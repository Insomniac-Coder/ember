# Part VIII — Classes and Reference Counting (the object world)

## VIII.1 Object representation

Every class instance is a heap block:

```
offset  size   field
0       4      strong_count   u32   (non-atomic if the class is !Sync, atomic if Sync)
4       4      weak_count     u32   (same atomicity)
8       4      access_state   u32   (dynamic exclusivity: bit31 = writer, bits0..30 = reader count; !Sync classes only)
12      4      flags          u32   (bit0 = is_deinitialising, bit1 = pinned/foreign-retained, rest reserved)
16      8      type_info      *const TypeInfo   (vtable, size, align, drop fn, class name, base chain, interface tables)
24      …      base-class fields, then own fields, each at its natural alignment
```

* `[OBJ-1]` The header is 24 bytes on 64-bit targets and is ABI-stable within a compiler minor version. Foreign code never inspects it; it goes through `ember_rt` functions.
* `[OBJ-2]` A handle is a pointer to offset 0. A `dyn I` handle to a class is *not* a fat pointer: the vtable is reached through `type_info` (this keeps `Option[Handle]` one pointer wide and makes class handles ABI-compatible with `void*`).
* `[OBJ-3]` `weak_count` starts at 1 for the "strong side"; when `strong_count` hits 0 the object is deinitialised (`drop` chain + field drops) and `weak_count` is decremented; the block is freed when `weak_count` hits 0. (Rust `Rc` scheme; avoids a separate control block.)
* `[OBJ-4]` Allocation uses the runtime's global allocator (`ember_alloc(size, align)`), default `mimalloc`; the class attribute `@allocator(Name)` selects an alternative for all instances of that class (v2).
* `[OBJ-5]` `ember_rt_deinit(obj)` MUST set `flags.is_deinitialising` before invoking the drop chain and MUST, after the drop chain returns and **again after every field's drop glue has run**, re-read `strong_count`. If it is not zero the object has been resurrected: the runtime MUST panic `E-panic: object resurrected during drop: <Class>` in every profile. No profile setting may remove this check (`[PRF-1]`). The second read is required because a field's drop glue runs after the user `drop` returns and can itself reach the object; it cannot succeed by any other route, because at `strong_count == 0` no strong handle exists and `[WK-3]` blocks `Weak.upgrade`.

## VIII.2 Reference-count operations and their elision

* `[RC-1]` Copying a handle emits `retain`; dropping a handle emits `release`; `release` reaching zero calls `ember_rt_deinit(obj)`.
* `[RC-2]` **Guaranteed elisions** (conformance-tested): (a) passing a handle to a `borrowed` parameter emits no RC ops; (b) a handle read from a place and used only within a single expression emits no RC ops when the place is not written during the expression; (c) `retain` immediately followed by `release` on the same handle with no intervening call or store is removed; (d) a handle stored into a field from a temporary is moved, not retained+released. Its lettered clauses are individually citable as `[RC-2a]`..`[RC-2d]` in the order written.
* `[RC-3]` Additional elisions (`retain` sinking, `release` hoisting across calls proven not to release the object) are permitted only when semantics-preserving; `[PHIL-5]` applies. Elisions never change *when* `drop` runs relative to observable side effects except that an object may be deinitialised **earlier** than the last syntactic use of a handle whose value is provably not needed and no loan derived from that object is in scope (`[RC-5]`) — the compiler treats a handle exactly like any other `Copy` value with a `drop`. A borrow *into* the object is a use of the handle (`[RC-5]`); `mem.keep_alive(h)` is therefore needed only to extend liveness past the last borrow, never to protect a live borrow. Programs MUST NOT depend on an object staying alive past the last use of its handles (identical to Swift; a `with h:` or `mem.keep_alive(h)` pins it explicitly).
* `[RC-4]` Non-`Sync` classes use plain loads/stores for counts; `Sync` classes use relaxed-increment / acquire-release-decrement atomics.
* `[RC-5]` A loan whose place chain contains a `Deref` of a class handle constitutes a **shared loan of the handle operand itself**. `ref h.f`, `ref mut h.f`, a `Span`/`MutSpan` derived from `h.f`, a `Ref[T]`/`RefMut[T]` guard obtained from a `RefCell` field of `h`, and any view struct built from these, all keep `h` borrowed for `[BRW-1]` purposes until the last use of any value derived from them. The compiler MUST NOT kill that loan earlier, MUST NOT sink the handle's `release` above it, and MUST report `E3060` (shape B7) if the handle's storage ends first. XVIII §4.7 **step 2** generates the extra loan when lowering a projection through a handle.
* `[RC-2e]` **Loop-borrowed handles.** Iterating a place whose element type is a class handle, `Shared[T]`, or a value type containing either — `for h in xs`, `for h in xs.iter()`, `for a, b in q` — emits **no** retain and no release on the yielded handles, provided the container is borrowed for the whole loop per `[CTL-2]` and the loop body does not store the handle into memory, return it, pass it to an `owned` parameter, or capture it in an `owned fn`. The yielded value is a borrowed handle whose region is the loop's borrow of the container, exactly as if it had been passed to a borrowed parameter under `[RC-2a]`. Where the body does one of the excluded things, the retain is emitted at that use, not at the top of the iteration. Like `[RC-2a]`..`[RC-2d]`, this elision is guaranteed and conformance-tested by counting `ember_retain` in the emitted C.
* `[RC-6]` **RC operations are optimisation barriers, and this MUST be visible.** `ember_retain` and `ember_release` write through the object pointer; a host C compiler cannot prove the header word distinct from the object's fields, so a surviving RC operation inside a loop invalidates loop-invariant loads of that object's fields and prevents vectorisation. `ember inspect` MUST list, per function, every retain and release that survives inside a loop body, with the source span, the loop, and the reason the elision analysis did not remove it, drawn from `[EFF-11]`'s vocabulary. The `Elided: RC` line of `ember inspect --safety` gains a corresponding `Surviving: RC` section.

## VIII.3 Exclusivity (aliased mutation)

Because handles alias, two references to the same object may be live at once. Ember enforces Swift's **law of exclusivity** dynamically for the accesses that can actually cause memory unsafety:

* An **instantaneous access** — reading or writing a single scalar/`Copy` field through a handle (`h.x = 1.0`, `y = h.x`) — is performed directly with no check. It cannot leave a dangling reference.
* A **long-term access** — any of: calling a `mut self` method, passing `h.field` to a `mut` parameter, taking `ref`/`ref mut` to `h.field` or to a sub-place of it, iterating `h.field` with `for` — begins an access on the object at its start and ends it at its last use (NLL), and:
  * `[EXC-1]` beginning a **write** access while any access (read or write) is active on that object panics with `E-panic: exclusivity violation: overlapping mutable access to <Class>.<field>` in every profile unless the package sets `exclusivity = "unchecked"` (shipping only, and then it is UB). The setting reaches dynamic **class** exclusivity and nothing else (`[CELL-9]`).
  * `[EXC-2]` beginning a **read** access while a write is active panics likewise;
  * `[EXC-3]` The compiler MUST NOT elide the dynamic check on the ground that all *statically visible* accesses to an object go through one handle local. An access A on object O, opened at point p and closed at point q, MAY be elided only if **both**: (a) every long-term access to O in [p, q) that the compiler can see goes through the same handle local as A, that local is not reassigned in [p, q), and conflicting accesses among them are rejected at compile time (`E3080`); **and** (b) no point in [p, q) can begin an access to O that the compiler cannot see. (b) holds when either (b1) the interval contains no call, no virtual or interface dispatch, and no call through a function value; or (b2) escape analysis (`[OPT-1]`) proves that at p, O is reachable from exactly one live handle, that handle is the local named in (a), and no statement in [p, q) stores a handle to O into memory or passes one to a call. Otherwise the compiler MUST emit `begin_access`/`end_access` even though the check in (a) succeeded.
  * `[EXC-4]` An **instantaneous** read of a `let` field — one that copies the field's value, so that no reference to it outlives the read — never begins an access. A **long-term** access to a `let` field begins an access exactly as for a non-`let` field: taking `ref`/`ref mut` to it or to a sub-place of it, iterating it, passing it to a `mut` parameter, or calling a `mut self` method on it. `let` restricts assignment to the field; it does not freeze the value the field holds (owner decision `OQ-18`, resolution A).
  * `[EXC-5]` Nested accesses through the *same* method's `mut self` are statically permitted (they are reborrows).
  * `[EXC-3a]` Every elision MUST be recorded in the safety side table (`[EFF-10]`) with reason `closed_interval` (b1) or `unique_handle` (b2). `ember inspect --safety --elided-only` MUST report it. An implementation that cannot name the condition MUST NOT elide.
  * `[EXC-6]` **Both ends of a conflict MUST be reported.** In the `debug` profile the runtime maintains a per-thread stack of active long-term accesses, each entry `{object, kind, source location}`, pushed by `begin_access` and popped by `end_access`. The panic message required by `[EXC-1]`/`[EXC-2]` MUST name the location of both the offending access and the active access it conflicts with, and MUST carry a `help`:  ``` E-panic: exclusivity violation: write access to Node.children at src/scene.em:31:9 while a read access begun at src/scene.em:22:13 is still active help: hoist the handle to a local so the accesses are ordered statically (EXC-3), or defer the mutation to a command buffer applied after the walk (see ECS-5) ```  In `release`, the mandated `help` text is required unconditionally, and the second location is required only where the implementation can carry it at **no additional per-access cost** — a single static location pointer stored alongside `access_state`, never a stack. Recording the full stack in `release` is an opt-in profile key whose cost MUST be measured under `[BEN-*]` before it is made mandatory. In `shipping` with `exclusivity = "unchecked"` nothing is recorded and no check is performed. **The header layout of `[OBJ-1]` is unchanged.**
  * `[EXC-7]` The lint `L3013 long-term access held across a call` fires when a long-term access on a class object is live across a **virtual or `dyn`** call within one `open class` hierarchy whose reachable call graph (`[EFF-1]`) is not proven free of accesses to the same class type. It names the field and the call, mirroring `L3011`. It is a lint, never an error, and is **off by default**.

Cost: an uncontended `begin_access` is a load, a compare and a store on a header word; `end_access` a decrement. **No fixed figure is normative** (owner decision `OQ-16`): the cost is re-measured under `[BEN-1]`–`[BEN-7]` and published, because `[EXC-3]`'s closed-interval restriction, `[EXC-6]`'s location recording and `[FFI-33]`'s thread assertions all add work to this path. `ember inspect --safety` reports each access as `STATIC`, `ELIDABLE` or `DYNAMIC`. Classes are not intended for inner loops; see the DOD facilities (Part XII) for those.

## VIII.4 Inheritance and dispatch

* `[DSP-1]` Static dispatch is used whenever the receiver's static type is a `final` class or the method is non-`virtual`.
* `[DSP-2]` Virtual dispatch loads `type_info->vtable[slot]`; vtable slots are assigned in declaration order, base class first; `override` reuses the base slot.
* `[DSP-3]` Interface dispatch on a class handle typed as an interface `I` loads `type_info->itable(I)` via a small per-class array of `{interface_id, table*}` pairs searched linearly (classes implement few interfaces); the compiler caches the lookup in a hidden local when the same handle is used repeatedly.
* `[DSP-4]` `h as? D` walks `type_info->base` chain; `is` for classes compares pointers.
* `[DSP-5]` Devirtualisation: with whole-program knowledge (LTO / single package) the compiler may replace a virtual call with a direct call when exactly one implementation is reachable; this is an optimisation and must be reported in `--emit-optimization-report`.

## VIII.5 Weak handles and cycles

* `Weak[C]` does not keep the object alive. `w.upgrade() -> Option[C]`; `Weak(h)` creates one; `Weak[C].empty()`.
* `[WK-1]` Reference cycles between class instances leak (they are never collected). This is a documented property, not a bug. Mitigations the language provides: `Weak` for back-pointers; the debug runtime's **cycle report** (`ember run --leak-check`), which on process exit walks all live objects (the runtime keeps an intrusive list of live objects in debug builds) and prints any strongly connected components with the field names forming the cycle; and the lint `L3001 potential cycle: field <f> of class <A> holds <B> and <B>.<g> holds <A>` for statically visible cycles.
* `[WK-2]` `drop` runs when the strong count hits zero even if weak handles remain; those weak handles then fail to upgrade.
* `[WK-3]` `Weak.upgrade` MUST return `None` when `flags.is_deinitialising` is set on the target, and `retain` on such an object MUST NOT be treated by any optimisation as producing a usable handle. This is the mechanism behind `[WK-2]`.

## VIII.6 Stack promotion (optimisation)

`[OPT-1]` If escape analysis proves that no handle to an object outlives the function that created it (no store into memory, no `owned` argument, no return, no capture by `owned fn`, no `spawn`), the compiler MAY allocate the object in the function's frame with the same header and run `drop` at scope end. This is unobservable (identity comparisons, `drop` order and exclusivity semantics are preserved).
* `[OPT-2]` **Loop bounds-check versioning.** For a counted loop whose induction variable `i` ranges over `a..b` or `a..=b` with `a` and `b` loop-invariant, whose body indexes one or more views at `i`, `i + c` or `i − c` for compile-time constant `c`, and where each such view's base and length are loop-invariant across the loop, the compiler MUST emit: a single loop-entry test that every index the loop can produce lies in `0..len` for every indexed view; an **unchecked** body with those `Assert` terminators removed, taken when the test passes; and the ordinary **checked** body otherwise. Observable behaviour is unchanged in every profile: the unchecked body is entered only when no iteration of it could have failed, so `[PHIL-5]` and `[PRF-1]` are satisfied and no profile setting is involved.
* `[OPT-3]` The rule applies whether the bound is written `s.len()` (the case §4.12 already names) or is a separate loop-invariant local — the latter is the shape `[PAR-2]`'s disjointness proof requires, and MUST be covered.
* `[OPT-2a]` `tests/conformance/OPT-2/` MUST contain the `@parallel` example of Part XI §4 and the `integrate` example of Part I §5, each asserting that the emitted C contains no call to `ember_panic_bounds` inside the loop body.

## VIII.7 Class idioms for engine code

```ember
open class Component:
    let entity: Entity                     # immutable after init
    world: Weak[World]                     # back-pointer: weak

class Health(Component):
    value: f32 = 100.0
    on_death: Option[Box[dyn fn(Entity)]] = None   # owned callback; fine to store (owned fn)

    fn apply(mut self, dmg: f32):
        self.value -= dmg
        if self.value <= 0:
            if Some(cb) = self.on_death:  # borrows the callback for the call
                cb(self.entity)
```

---

