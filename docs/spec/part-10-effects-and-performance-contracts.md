# Part X — Effects and Performance Contracts

## X.1 Effects

The compiler infers, for every function, an **effect set** ⊆ `{Alloc, Sync, Panic, Unsafe, FFI, Block, RuntimeCheck(k)}`:

| Effect | Introduced by |
|---|---|
| `Alloc` | any call to `ember_alloc`/`realloc`, class instantiation, `Box`, `Shared`, container growth, `String` formatting, f-strings, `Arena` growth, boxed closures |
| `Sync` | atomic RMW ops with ordering stronger than relaxed, `Mutex`/`RwLock` lock, channel send/receive, thread spawn/join, `Sync`-class retain/release (atomic) |
| `Panic` | `panic`, `assert`, bounds checks, overflow checks (debug), `unwrap`, exclusivity checks, division |
| `Unsafe` | body contains an `unsafe` block or the function is `unsafe fn` |
| `FFI` | calls to `extern` functions |
| `Block` | `Mutex.lock`, `join`, `sleep`, channel blocking receive, `File.read` — any call the runtime marks blocking |
| `RuntimeCheck(k)` | a safety check the compiler emitted rather than proved away, for kind `k ∈ {Aliasing, Bounds, Stale, Overflow}` — see §X.1.1 |

* `[EFF-1]` Effects are computed per function from its body and the (already computed) effects of its callees, over the call graph, with recursion handled by fixpoint (recursive SCCs are assumed to have the union of their members' direct effects).
* `[EFF-2]` Calls through `dyn` or function values contribute the effects declared on the interface method or function type; a method in an interface may declare `@noalloc` and implementers MUST satisfy it (`E4010`). A `fn(A) -> R` parameter type is assumed to carry all effects unless written `@noalloc fn(A) -> R`.
* `[EFF-3]` `extern` functions carry effects declared in their contract (`@ffi(effects=[FFI])` by default; `@ffi(effects=[FFI, Alloc, Block])` if the binding says so).
* `[EFF-4]` Effects are part of a function's public interface for the purpose of caching: a change to a callee's effect set invalidates callers' contract checks (Part XIX build graph).

### X.1.1 The `RuntimeCheck` effect

`RuntimeCheck` records that Safe code obtained one of its guarantees by the runtime arm of `[PHIL-8]` rather than the static arm. Its four kinds:

| Kind | Emitted for |
|---|---|
| `Aliasing` | dynamic class exclusivity (`[EXC-1/2]`), `RefCell.borrow`/`borrow_mut` state checks (`[CELL-5]`) |
| `Bounds` | an index or slice check the compiler could not prove redundant |
| `Stale` | generational handle validation (`[HND-1]`, `[GPU-1]`), `Weak.upgrade` |
| `Overflow` | checked arithmetic under `@overflow(panic)` or the `debug` profile |

* `[EFF-9]` `RuntimeCheck(k)` enters a function's **contract** effect set when a check of kind `k` in that function's own body survives the profile-independent elision passes of `[EFF-15]`, and propagates through the call graph like every other effect (`[EFF-1..3]`). It is a statement about the code the contract profile would generate, not about source syntax and not about the selected profile.
* `[EFF-10]` **The effect is coarse; the site record is not.** The effect set carries only the kinds present, so that contracts can be checked cheaply and transitively. Separately, codegen emits a **safety-check side table** (`target/<profile>/inspect/<module>.safety.json`) with one entry per emitted check: `{kind, source span, function, mechanism, reason}`, plus one entry per *elided* check with the analysis that removed it. The side table is what `ember inspect --safety` reads (`[CLI-3]`). Implementations MUST NOT push per-site data into the effect lattice.
* `[EFF-11]` **Reason codes.** Every emitted-check entry carries exactly one reason, and diagnostics MUST use its wording rather than a generic "could not prove" message:

  | Reason | Meaning | What the programmer can do |
  |---|---|---|
  | `not_provable_in_principle` | no static analysis could establish the property — the value is genuinely runtime data (an index read from a file, a handle from a scene) | nothing; the check is correct and permanent |
  | `not_proven_by_analysis` | the property may hold, but this compiler's analysis did not establish it | restructure per the hint, or file a compiler issue — this is the only reason that is a candidate for future elision |
  | `requested_by_type` | the programmer chose a dynamically checked type (`RefCell`, `Cell`-free aliasing through a class) | the check is the type's purpose; change the type to change the cost |
  | `inherent_to_mechanism` | the check is what the mechanism *is* (a generation compare in a generational handle) | use a different mechanism (a direct reference, an index) if the cost matters |
  | `establishes_static_fact` | the check verifies a property once and returns proof-carrying values, so the property is static from there on (`[DSJ-1]`, `[DSJ-5]`) | nothing; the check is what makes the code after it checkable, and `[EFF-12]` permits it under `@static_safe` |

  `[EFF-11a]` Reporting `not_proven_by_analysis` where `not_provable_in_principle` is correct is a diagnostic bug: it tells the programmer to restructure code that cannot be improved. The conformance suite fixes the expected reason for each check site in `tests/safety/reasons/`.

## X.2 Hard contracts

* `@noalloc fn`: `[EFF-5]` MUST NOT have `Alloc` in its effect set. Violations report the **full call chain** to the allocation site: `error[E4001]: @noalloc function `cull` reaches an allocation: cull → collect_visible → Array.push → ember_alloc`.
* `@nosync fn`: MUST NOT have `Sync`.
* `@noblock fn`: MUST NOT have `Block`.
* `@nopanic fn` **(v2)**: MUST NOT have `Panic` — requires proving away bounds checks; reserved.
* `@static_safe fn`: `[EFF-12]` MUST NOT have `RuntimeCheck(Aliasing)` in its effect set **except from sites whose reason code is `establishes_static_fact`** (owner decision `OQ-11`). That is: every aliasing property the function *relies on* is either proven statically or established by a verifying operation whose result carries the proof. It forbids dynamic class exclusivity, `RefCell` borrow checks, and any callee that carries a non-establishing `Aliasing` check. It does **not** forbid `RuntimeCheck(Bounds)`, `(Stale)` or `(Overflow)`: an in-bounds index and a live handle are different guarantees from aliasing, are frequently not provable in principle (`[EFF-11]`), and lumping them together would make the contract unusable. `@no_runtime_checks` — the stronger form excluding all four kinds — is **reserved for v2**, once bounds-check elimination is strong enough for it to be satisfiable.
* `[EFF-13]` **`@static_safe` is, in practice, a value-type contract.** Any long-term access through a class handle that was loaded from memory (a field, an array element, a `dyn` receiver) is dynamically checked by construction (`[EXC-3]`), so most code written against class handles cannot satisfy the contract. This is correct behaviour, not a limitation to be worked around, and the diagnostic MUST say so specifically rather than reporting a bare contract violation:

  ```
  error[E4030]: @static_safe function `integrate` performs a dynamically checked access
    --> src/systems.em:14:9
     |
  14 |         self.target.transform.position += v
     |         ^^^^^^^^^^^ handle loaded from field `self.target`, so exclusivity is checked at runtime
     |
     = help: hoist the handle to a local first, so accesses through it are statically ordered:
             `t = self.target` then `t.transform.position += v`
     = help: or operate on value types — a `Query[(mut Transform,)]` yields `ref mut Transform` with no handle indirection
     = note: accesses through a handle held in a local are proven statically (EXC-3); accesses through a
             handle re-read from memory cannot be, because another handle to the same object may exist
  ```
* `[EFF-14]` Contract attributes compose: `@static_safe @noalloc @nosync` is the inner-loop set. Each is checked independently against the same effect set.
* `[EFF-6]` A contract applies to the whole reachable call graph, including drop glue for locals, default arguments, and operator impls.
* `[EFF-6a]` `@static_safe` additionally applies to any check the *caller* would have to emit on this function's behalf — a `@static_safe` function may not take a parameter whose type forces a dynamic check at the call site (`RefCell[T]` by value, `Ref[T]`/`RefMut[T]` guards).
* `[EFF-7]` `unsafe: @assume_noalloc(expr)` overrides the analysis for one call (e.g. a C function known not to allocate); it is `unsafe` because the compiler cannot verify it.
* `[EFF-8]` Contracts are inherited by overriding methods (an `override` of a `@noalloc virtual fn` must be `@noalloc`).
* `[EFF-15]` **The contract profile.** Contract checking (`[EFF-5]`, `[EFF-12]`, `@nosync`, `@noblock`, and `@nopanic` in v2) MUST use an effect set computed once per build against the **contract profile**: bounds checks enabled, `overflow = "panic"`, `exclusivity = "checked"`, and only those elisions that are independent of profile settings (`[EXC-3]`, `[RC-2/3]`, target-independent bounds-check elimination). The contract profile pins the **set of elision passes** as well as the profile keys: adding or strengthening an elision pass changes which programs satisfy a contract, and is therefore a versioned change under `[VER-2]`. A profile setting MUST NOT remove an effect from the contract set. Consequently a program that satisfies its contracts under one profile satisfies them under all, and `E4001`/`E4030` are reported identically by `ember build --profile debug|release|shipping`.
* `[EFF-16]` **`Panic` is refined.** `Panic(Explicit)` is introduced by: `panic`, `assert`/`assert_eq`/`assert_ne` (every profile), `unwrap`, `expect`, `todo`, `unreachable`, `as!` downcast, allocation failure (`[ALC-4]`), integer division or remainder by a divisor not proven non-zero, `i32.MIN / -1` where the operands are not proven safe, and a shift whose amount is not proven in range. Every other panic is recorded by the `RuntimeCheck(k)` kind that already covers it: bounds panics by `Bounds`, overflow panics by `Overflow`, exclusivity and `RefCell` borrow failures by `Aliasing`, generational-handle and `Weak.upgrade` failures by `Stale`. `Panic`, where this document uses it unqualified, means the union. `[EFF-9]`'s post-elision rule applies to `Panic(Explicit)` unchanged, computed **under `[EFF-15]`'s contract profile**, so the set does not vary with the optimisation level.
* `[EFF-17]` **`@nopanic(explicit) fn` (v1).** MUST NOT have `Panic(Explicit)` in its contract effect set. Violations report the full call chain in `[EFF-5]`'s shape: `error[E4040]: @nopanic(explicit) function `resolve` reaches a panic: resolve → Option.unwrap → ember_panic_unwrap`. Bare `@nopanic` remains reserved for v2 and is defined as forbidding `Panic(Explicit)` together with `RuntimeCheck(Bounds)` and `RuntimeCheck(Overflow)`; `[OPT-2]` is what makes it satisfiable. `[EFF-6]`, `[EFF-8]` and `[EFF-2]` apply to it exactly as to the other contracts. `[EFF-14]`'s inner-loop set becomes `@static_safe @noalloc @nosync @nopanic(explicit)`.

## X.3 Inspection

`ember inspect path.to.fn` prints the effect set, the allocation sites reachable, inlining decisions, whether loops vectorised, the chosen ABI for each parameter, and `size_of`/`align_of`/field offsets for types. This is the primary tool for making the compiler's allocation, layout, dispatch and vectorisation decisions visible rather than implicit.

`ember inspect --safety <path>` reads the side table of `[EFF-10]` and reports every runtime safety check the function emits and every one it elided:

```
$ ember inspect --safety game.world.World.update

Function: game.world.World.update
Effects: Alloc, Panic, RuntimeCheck(Aliasing), RuntimeCheck(Bounds)

Runtime checks emitted:
  Aliasing   2
    src/world.em:88:9   dynamic exclusivity on `Entity`
                        reason: not_proven_by_analysis — handle loaded from `self.active[i]`
                        help:   hoist to a local, or iterate a Query yielding `ref mut Transform`
    src/world.em:141:5  RefCell borrow_mut on `RefCell[Array[Event]]`
                        reason: requested_by_type
  Bounds     1
    src/world.em:96:22  index `ids[j]` where `j` is read from `pending`
                        reason: not_provable_in_principle
  Stale      0
  Overflow   0

Elided:
  Aliasing   6   (EXC-3: accesses ordered through the same handle local)
  Bounds    23   (range analysis over `for i in 0..len`)
  RC        14   (RC-2a borrowed parameter, RC-2c retain/release pair)

Estimated cost of emitted checks: ~7 ns/call at the measured call count
```

* `[CLI-3]` `--safety` accepts `--json`, and `--safety --elided-only` reports just what was removed, which is the form used when investigating why a `@static_safe` function fails to compile.
* `[TOOL-1]` Each release publishes a self-contained toolchain archive per supported host (`x86_64-pc-windows-msvc`, `x86_64-unknown-linux-gnu`, `aarch64-apple-darwin`) containing `ember`, the `ember_rt` libraries, `std` sources and the editor packages, plus a one-line installer (`install.ps1`/`install.sh`) that unpacks it and places `ember` on `PATH`.
* `[TOOL-3]` When no C compiler is found the driver emits `E9001 no C compiler found`, whose `help` names the remedies verbatim for the host — on Windows, Visual Studio Build Tools with the "Desktop development with C++" workload. **A missing `cl.exe`/`clang` path MUST NOT be the primary message.**
* `[TOOL-4]` `ember --version` prints compiler version, language version, backend, and the resolved C compiler and linker, so a bug report carries its environment.  **M0 — the first hour** (Phase 0 exit criterion; `tests/milestones/m0_first_hour.md` plus a CI job on a clean image with no C toolchain beyond what `[TOOL-1]`/`[TOOL-3]` provide, no Rust and no editor): the recorded script `install → ember new hello → cd hello → ember run` MUST print `hello, world` in under five minutes wall time and at most six typed commands, on Windows and on Linux. A regression in M0 blocks a release.
* `[TOOL-2]` `ember toolchain install cc` downloads a pinned Clang + `lld` into the toolchain directory and selects it via `[build] c_compiler = "bundled"`. `lld` is the default linker where available. **A working Ember installation MUST NOT require a separately installed C toolchain.** This is a committed deliverable (owner decision `OQ-23`), not a conditional one; a system compiler MAY be selected explicitly and its full configuration is recorded in the build record.

---

