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

## VIII.2 Reference-count operations and their elision

* `[RC-1]` Copying a handle emits `retain`; dropping a handle emits `release`; `release` reaching zero calls `ember_rt_deinit(obj)`.
* `[RC-2]` **Guaranteed elisions** (conformance-tested): (a) passing a handle to a `borrowed` parameter emits no RC ops; (b) a handle read from a place and used only within a single expression emits no RC ops when the place is not written during the expression; (c) `retain` immediately followed by `release` on the same handle with no intervening call or store is removed; (d) a handle stored into a field from a temporary is moved, not retained+released.
* `[RC-3]` Additional elisions (`retain` sinking, `release` hoisting across calls proven not to release the object) are permitted only when semantics-preserving; `[PHIL-5]` applies. Elisions never change *when* `drop` runs relative to observable side effects except that an object may be deinitialised **earlier** than the last syntactic use of a handle whose value is provably not needed — the compiler treats a handle exactly like any other `Copy` value with a `drop`. Programs MUST NOT depend on an object staying alive past the last use of its handles (identical to Swift; a `with h:` or `mem.keep_alive(h)` pins it explicitly).
* `[RC-4]` Non-`Sync` classes use plain loads/stores for counts; `Sync` classes use relaxed-increment / acquire-release-decrement atomics.

## VIII.3 Exclusivity (aliased mutation)

Because handles alias, two references to the same object may be live at once. Ember enforces Swift's **law of exclusivity** dynamically for the accesses that can actually cause memory unsafety:

* An **instantaneous access** — reading or writing a single scalar/`Copy` field through a handle (`h.x = 1.0`, `y = h.x`) — is performed directly with no check. It cannot leave a dangling reference.
* A **long-term access** — any of: calling a `mut self` method, passing `h.field` to a `mut` parameter, taking `ref`/`ref mut` to `h.field` or to a sub-place of it, iterating `h.field` with `for` — begins an access on the object at its start and ends it at its last use (NLL), and:
  * `[EXC-1]` beginning a **write** access while any access (read or write) is active on that object panics with `E-panic: exclusivity violation: overlapping mutable access to <Class>.<field>` in every profile unless the package sets `exclusivity = "unchecked"` (shipping only, and then it is UB);
  * `[EXC-2]` beginning a **read** access while a write is active panics likewise;
  * `[EXC-3]` the compiler **elides** the runtime check when it can prove statically that no conflicting access to the *same object* is live in the interval: two accesses through the *same handle local* in one function are checked statically exactly like borrows (`[BRW-*]`) and produce compile-time errors (`E3080`) rather than runtime panics; accesses through *different* handle locals or through handles loaded from memory are dynamically checked.
  * `[EXC-4]` Reads of `let` (immutable) fields never begin an access.
  * `[EXC-5]` Nested accesses through the *same* method's `mut self` are statically permitted (they are reborrows).

Cost: an uncontended `begin_access` is a load, a compare and a store on a header word (~2 ns); `end_access` a decrement. Classes are not intended for inner loops; see the DOD facilities (Part XII) for those.

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

## VIII.6 Stack promotion (optimisation)

`[OPT-1]` If escape analysis proves that no handle to an object outlives the function that created it (no store into memory, no `owned` argument, no return, no capture by `owned fn`, no `spawn`), the compiler MAY allocate the object in the function's frame with the same header and run `drop` at scope end. This is unobservable (identity comparisons, `drop` order and exclusivity semantics are preserved).

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

