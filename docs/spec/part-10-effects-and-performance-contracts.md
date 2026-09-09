# Part X — Effects and Performance Contracts

## X.1 Effects

The compiler infers, for every function, an **effect set** ⊆ `{Alloc, Sync, Lock, Io, Panic, Unsafe, FFI, Block, Nondet, RuntimeCheck(k)}`. Effects are orthogonal: one operation MAY introduce several effects at once.

| Effect | Introduced by |
|---|---|
| `Alloc` | any call to `ember_alloc`/`realloc`, class instantiation, `Box`, `Shared`, container growth, `String` formatting, f-strings, `Arena` growth, boxed closures |
| `Sync` | atomic RMW ops with ordering stronger than relaxed, lock acquisition, channel send/receive, thread spawn/join, `Sync`-class retain/release (atomic) |
| `Lock` | acquisition of a `Mutex`/`RwLock` or equivalent synchronisation primitive; `try_lock` and non-blocking lock acquisition carry `Lock` even when they do not carry `Block` |
| `Io` | file, socket, console and other external-I/O operations, whether or not they block |
| `Panic` | `panic`, `assert`, bounds checks, overflow checks (debug), `unwrap`, exclusivity checks, division |
| `Unsafe` | body contains an `unsafe` block or the function is `unsafe fn` |
| `FFI` | calls to `extern` functions |
| `Block` | `Mutex.lock`, `RwLock` operations that may wait, `join`, `sleep`, channel blocking receive, `File.read`, and any call the runtime marks as potentially waiting |
| `RuntimeCheck(k)` | a safety check the compiler emitted rather than proved away, for kind `k ∈ {Aliasing, Bounds, Stale, Overflow}` — see §X.1.1 |
| `Nondet` | an operation whose result may differ between two runs of the same program over the same inputs, or between two machines running the same binary — enumerated exhaustively in `[DET-2]` |

* `[EFF-1]` Effects are computed per function from its body and the (already computed) effects of its callees, over the call graph, with recursion handled by fixpoint (recursive SCCs are assumed to have the union of their members' direct effects).
* `[EFF-2]` Calls through `dyn` or function values contribute the effects declared on the interface method or function type; a method in an interface may declare `@noalloc` and implementers MUST satisfy it (`E4010`). A `fn(A) -> R` parameter type is assumed to carry all effects unless written `@noalloc fn(A) -> R`.
* `[EFF-3]` `extern` functions carry effects declared in their contract (`@ffi(effects=[FFI])` by default; `@ffi(effects=[FFI, Alloc, Block])` if the binding says so).
* `[EFF-4]` Effects are part of a function's public interface for the purpose of caching: a change to a callee's effect set invalidates callers' contract checks (Part XX build graph).

### X.1.1 The `RuntimeCheck` effect

`RuntimeCheck` records that Safe code obtained one of its guarantees by the runtime arm of `[PHIL-8]` rather than the static arm. Its four kinds:

| Kind | Emitted for |
|---|---|
| `Aliasing` | dynamic class exclusivity (`[EXC-1/2]`), `RefCell.borrow`/`borrow_mut` state checks (`[CELL-5]`) |
| `Bounds` | an index or slice check the compiler could not prove redundant |
| `Stale` | generational handle validation (`[HND-1]`, `[GPU-1]`), `Weak.upgrade` |
| `Overflow` | checked arithmetic under `@overflow(panic)` or the `debug` profile |

* `[EFF-9]` `RuntimeCheck(k)` enters a function's effect set when a check of kind `k` in that function's own body survives the profile-independent elision passes of `[EFF-15]`, and propagates through the call graph like every other effect (`[EFF-1..3]`). It is a statement about the code the contract profile would generate, not about source syntax and not about the selected profile.
* `[EFF-10]` **The effect is coarse; the site record is not.** The effect set carries only the kinds present, so that contracts can be checked cheaply and transitively. Separately, codegen emits a **safety-check side table** (`target/<profile>/inspect/<module>.safety.json`) with one entry per emitted check: `{kind, source span, function, mechanism, reason}`, plus one entry per *elided* check with the analysis that removed it. The side table is what `ember inspect --safety` reads (`[CLI-3]`). Implementations MUST NOT push per-site data into the effect lattice.
* `[EFF-11]` **Reason codes.** Every emitted-check entry carries exactly one reason, and diagnostics MUST use its wording rather than a generic "could not prove" message.

  | Reason | Meaning | What the programmer can do |
  |---|---|---|
  | `not_provable_in_principle` | no static analysis could establish the property — the value is genuinely runtime data (an index read from a file, a handle from a scene) | nothing; the check is correct and permanent |
  | `not_proven_by_analysis` | the property may hold, but this compiler's analysis did not establish it | restructure per the hint, or file a compiler issue — this is the only reason that is a candidate for future elision |
  | `requested_by_type` | the programmer chose a dynamically checked type (`RefCell`, `Cell`-free aliasing through a class) | the check is the type's purpose; change the type to change the cost |
  | `inherent_to_mechanism` | the check is what the mechanism *is* (a generation compare in a generational handle) | use a different mechanism (a direct reference, an index) if the cost matters |
  | `establishes_static_fact` | the check verifies a property once and returns proof-carrying values, so the property is static from there on (`[DSJ-1]`, `[DSJ-5]`) | nothing; the check is what makes the code after it checkable, and `[EFF-12]` permits it under `@static_safe` |

  `[EFF-11a]` Reporting `not_proven_by_analysis` where `not_provable_in_principle` is correct is a diagnostic bug: it tells the programmer to restructure code that cannot be improved. The conformance suite fixes the expected reason for each check site in `tests/safety/reasons/`.

* `[EFF-18]` **Effects are orthogonal.** `Io` (file, socket, console) and `Lock`
  (lock acquisition) are distinct from `Block` (may wait) and `Sync` (synchronisation
  semantics). One operation MAY carry several of them. In particular, a blocking
  `Mutex.lock` carries `Sync + Lock + Block`; a non-blocking `try_lock` carries
  `Sync + Lock`; a blocking file read carries `Io + Block`; and a non-blocking I/O
  operation carries `Io` without `Block`. `@noio`, `@nolock`, `@noblock` and
  `@nosync` therefore remain independent contracts. The full set is
  `{Alloc, Sync, Lock, Io, Panic, Unsafe, FFI, Block, RuntimeCheck(k)}`. `[EFF-18]`
  does not remove an effect previously attached to any operation; it refines the
  effect model so a single operation may report all applicable effects. `RuntimeCheck(k)`'s kinds are `{Bounds, Overflow, Aliasing, Stale}` — the four `[EFF-16]` assigns and `[EFF-22]` permits. `Contract` went with the contract prover (OQ-28..OQ-32, owner decision 0.6.2). *(0.6.2 leftover removed 2026-09-09; see `docs/spec-amendments.md`)*
* `[EFF-19]` `@realtime` on a function expands to a configured contract set,
  by default `@noalloc @nolock @noblock @nopanic(explicit)`. It is a **marker
  for that set and not a timing guarantee**: the compiler MUST NOT state or
  imply that a `@realtime` function meets a deadline, because hardware,
  scheduling, cache behaviour and foreign code decide that and none of them is
  visible to it. The set is named in the manifest so a project can widen or
  narrow it once rather than per function.

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
* `[EFF-22]` **`@nopanic(explicit)` does not mean "cannot panic", and the diagnostics MUST say so.** It forbids `Panic(Explicit)` — the panics the programmer writes — and permits `RuntimeCheck(Bounds)`, `RuntimeCheck(Overflow)`, `RuntimeCheck(Aliasing)` and `RuntimeCheck(Stale)`, every one of which can still abort. The name is retained rather than changed, because it is threaded through `[EFF-16]`'s refinement and renaming it is churn against a distinction the effect set already makes precisely; the price of retaining it is that every diagnostic naming the attribute, and its `docs/errors/` page, MUST state the permitted checks explicitly. A function that reaches no abort at all is `@nopanic(explicit)` **and** free of `RuntimeCheck` in its effect set — two facts, reported together by `ember inspect --safety`, and claimed by no single attribute.
* `[EFF-16]` **`Panic` is refined.** `Panic(Explicit)` is introduced by: `panic`, `assert`/`assert_eq`/`assert_ne` (every profile), `unwrap`, `expect`, `todo`, `unreachable`, `as!` downcast, allocation failure (`[ALC-4]`), integer division or remainder by a divisor not proven non-zero, `i32.MIN / -1` where the operands are not proven safe, and a shift whose amount is not proven in range. Every other panic is recorded by the `RuntimeCheck(k)` kind that already covers it: bounds panics by `Bounds`, overflow panics by `Overflow`, exclusivity and `RefCell` borrow failures by `Aliasing`, generational-handle and `Weak.upgrade` failures by `Stale`. `Panic`, where this document uses it unqualified, means the union. `[EFF-9]`'s post-elision rule applies to `Panic(Explicit)` unchanged, computed **under `[EFF-15]`'s contract profile**, so the set does not vary with the optimisation level. It is therefore outside `[EFF-17]`'s `@nopanic(explicit)` and outside `[EFF-19]`'s default `@realtime` set, so a frame-path function MAY carry contracts.
* `[EFF-17]` **`@nopanic(explicit) fn` (v1).** MUST NOT have `Panic(Explicit)` in its contract effect set. Violations report the full call chain in `[EFF-5]`'s shape: `error[E4040]: @nopanic(explicit) function `resolve` reaches a panic: resolve → Option.unwrap → ember_panic_unwrap`. Bare `@nopanic` remains reserved for v2 and is defined as forbidding `Panic(Explicit)` together with `RuntimeCheck(Bounds)` and `RuntimeCheck(Overflow)`; `[OPT-2]` is what makes it satisfiable. `[EFF-6]`, `[EFF-8]` and `[EFF-2]` apply to it exactly as to the other contracts. `[EFF-14]`'s inner-loop set becomes `@static_safe @noalloc @nosync @nopanic(explicit)`. Bare `@nopanic` (v2) forbids `Panic(Explicit)`, `RuntimeCheck(Bounds)` and `RuntimeCheck(Overflow)`; it does not forbid `RuntimeCheck(Aliasing)` or `RuntimeCheck(Stale)`. *(0.6.2 leftover removed 2026-09-09; see `docs/spec-amendments.md`)* `@no_runtime_checks` (v2, reserved) excludes all five kinds.
* `[EFF-20]` **`@noio fn`**: MUST NOT have `Io` in its contract effect set. Violations report the **full call chain** in `[EFF-5]`'s shape: `error[E4041]: @noio function `tick` reaches I/O: tick → log_frame → File.write → ember_io_write`. `[EFF-6]`, `[EFF-8]` and `[EFF-2]` apply exactly as to the other contracts.
* `[EFF-21]` **`@nolock fn`**: MUST NOT have `Lock` in its contract effect set, in the same shape (`E4042`). **`@nolock` does not imply `@noblock` and `@noblock` does not imply `@nolock`**: per `[EFF-18]` a `try_lock` carries `Lock` without `Block` and a blocking channel receive carries `Block` without `Lock`, so a function needing both writes both.
* `[EFF-19a]` **A `@realtime` function's contract set is fixed by the package that declares it.** The expansion is read from the manifest of the package **containing the function**, is recorded in that package's build record and in its interface metadata, and MUST NOT be altered by a consumer. A consumer wanting its own code checked against a different set writes that set with the individual contract attributes, which `[EFF-14]` already composes. The language default, applied when the declaring package's manifest names no set, is `@noalloc @nolock @noblock @nopanic(explicit)` and is fixed by the language version under `[VER-2]`. **Widening or narrowing a package's `realtime` set is a breaking change to that package under `[VER-2]`**, for the reason `[EFF-15]` already gives: it changes which programs satisfy a contract.
* `[EFF-19b]` A diagnostic arising from a contract in `@realtime`'s expansion MUST name **both** the expanded contract that failed and `@realtime` as the source of the obligation, so a user who wrote one attribute does not receive an error about a different one.

## X.2a Determinism

Lockstep networking, deterministic replay, and golden-image tests all want the same
thing: two runs of the same program over the same inputs compute bit-identical
results, on one machine and across machines. Ember does not promise that globally —
it would forbid optimisations every other workload wants — so it is a hard contract
over a `Nondet` effect, in the manner of `[EFF-12]`'s `@static_safe`.

```ember
@deterministic
fn step(mut w: World, input: InputFrame):
    integrate(w.bodies, FIXED_DT)
    resolve_contacts(w.bodies)
    w.tick += 1
```

* `[DET-1]` `@deterministic` on a function is a **hard contract**: `Nondet` MUST NOT appear in its effect set. Violation is `E4070`, which names the offending operation and the shortest call chain that reaches it, per `[EFF-10]`.
* `[DET-2]` `Nondet` is introduced by, and only by:
  * floating-point contraction, reassociation, or any `@fastmath` relaxation (`[TYP-9]`, `[TYP-9a]`, `[TYP-9b]`);
  * a transcendental (`sin`, `cos`, `tan`, `exp`, `log`, `pow`, `atan2` and their variants) taken from the platform's math library rather than from `std.math.det`, because no two libms agree in the last bits;
  * observing a pointer or handle as an integer, and any hash whose input includes one — which is why `Hash` for `*T` and for a class handle carries it;
  * iteration over a container whose order its type does not specify (`Map`, `Set`);
  * the wall clock, the monotonic clock, the system random source, thread and job completion order, and `[JOB-*]` work-stealing order;
  * reading uninitialised or padding bytes;
  * any `extern` function not declared `@ffi(deterministic)`.
* `[DET-3]` Propagation is `[EFF-1]`'s: a `@deterministic` function may call only functions free of `Nondet`. `[EFF-2]` applies unchanged — an interface method may declare `@deterministic` and implementers MUST satisfy it (`E4010`), and a `fn(A) -> R` parameter is assumed to carry `Nondet` unless written `@deterministic fn(A) -> R`.
* `[DET-4]` `std.math.det` provides `sin`, `cos`, `tan`, `exp`, `log`, `pow`, `atan2` and `sqrt` for `f32` and `f64`, specified to return the same bits on every supported target. `sqrt` is the one exemption from software emulation: IEEE-754 requires it to be correctly rounded and every supported target's instruction is. These are slower than the platform's; that is the price of the guarantee, and `ember inspect` reports which of them a call graph reaches.
* `[DET-5]` Inside a `@deterministic` function the backend MUST NOT contract, reassociate, or introduce an FMA the source did not write. `[TYP-9a]`'s translation-unit-wide `FP_CONTRACT OFF` already achieves this; `@fp(contract)` (`[TYP-9b]`) or `@fastmath` on a `@deterministic` function is `E4072`.
* `[DET-6]` `@deterministic` constrains **results, not timing**. A deterministic function may allocate, lock, block, and take a different amount of time on every run. It composes with `@noalloc` and `@realtime` and substitutes for neither.
* `[DET-7]` **Scope of the cross-machine claim.** Two machines running *the same binary* compute identical results for a `@deterministic` call graph. Two machines running binaries built from the same source by different toolchains, or for different targets, do not — `[DET-4]`'s guarantee is tied to emitted code — and this specification does not claim otherwise. Lockstep peers MUST therefore agree on the build, which `[BLD-13]`'s `--build-id` makes checkable in one comparison before the match starts.
* `[DET-8]` `@deterministic` on a module applies to every function it declares, and an individual function may not opt out. `@deterministic` on an `extern` block is `E0104`: a foreign function's determinism is *asserted*, with `@ffi(deterministic)`, and is an `asserted` fact in the sense of `[FFI-35]` that appears in `ember tcb` as one. `[PHIL-5]` applies — declining to look is not a proof, and an unbacked assertion is recorded as unbacked.
* `[DET-9]` `ember inspect --deterministic <path>` prints whether the named item satisfies `[DET-1]`, and if not, the shortest call chain to each `Nondet` source. A `@deterministic` function whose body reaches no `Nondet` source at all, in a build where every `@ffi(deterministic)` fact is `asserted`, is reported as such rather than as verified.

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

## X.4 The cost model

Ember claims C-like performance, and a claim like that is worth nothing unless it
says what it means. This section defines "zero-cost" for this document and classifies
every implicit cost the language can introduce, so that a programmer can answer
"what does this line cost" from the specification rather than from a disassembler.

* `[COST-1]` **Zero-cost, defined.** An abstraction is **zero-cost** when, for a use
  in which every dynamic check it implies has been statically discharged, the
  emitted code contains no instruction that the equivalent hand-written C would not
  contain, under the optimisation model of the selected backend and profile. Two
  consequences follow and are part of the definition: an abstraction is zero-cost
  *for a use*, never in general — the same generic is zero-cost where its checks are
  discharged and not where they are emitted; and "the equivalent C" means C that
  upholds the same invariant, not C that omits it. A bounds check the compiler
  cannot discharge is not a failure of zero-cost, it is the cost of the guarantee,
  and `[COST-3]` classifies it as such.
* `[COST-2]` **Five classes.** Every implicit cost in this document is classified as
  exactly one of:
  * **Guaranteed elidable** — an implementation MUST NOT emit it when the stated
    condition holds; emitting it anyway is a defect, not a quality-of-implementation
    matter.
  * **Guaranteed required** — always emitted; the semantics depend on it.
  * **Conditionally elidable** — an implementation MAY elide it when it can prove
    the condition; whether it did is reportable through `ember inspect`.
  * **Implementation-defined** — the specification does not constrain it; two
    conforming implementations may differ.
  * **Not observable** — no cost exists at runtime; the construct is erased.
* `[COST-3]` **The classification.**

| Cost | Class | Condition / note |
|---|---|---|
| Bounds check on `a[i]` | conditionally elidable | discharged by `[OPT-2]`'s range analysis or a `[RNG-4]` fact; `ember inspect --safety` reports each site |
| Overflow check | conditionally elidable | `debug` only under `[TYP-8]`; `release` wraps and emits nothing |
| Retain / release pair | guaranteed elidable | `[RC-2]`'s pairing rule — a retain immediately dominated by its release over a non-escaping handle MUST be removed |
| Retain / release, surviving | guaranteed required | reported per call site by `ember inspect`; atomic iff the class is `Sync` (`[RC-1]`) |
| Dynamic exclusivity check | conditionally elidable | discharged where `[EXC-3]` proves the access static; unchecked in `shipping` under `[PRF-1]` exception (2) |
| Stale-handle (generation) check | guaranteed required | `[HND-1]`'s guarantee is the check; `get_unchecked` is the `unsafe` opt-out |
| Interface dispatch through `dyn` | guaranteed required | one indirect call per method (`[TYP-22]`) |
| Generic call, specialised | not observable | direct call, inlinable (`[TYP-16]`) |
| Generic call, shared | guaranteed required | one indirect call per bound method (`[MONO-6]`), taken only under `[MONO-3]`'s ceiling |
| Class method dispatch | conditionally elidable | devirtualised where the concrete class is known (`[DSP-*]`) |
| Reload thunk indirection | guaranteed required in a reloadable build, not observable otherwise | `[HR-6]`, ≤ 3% by `[HR-9]`; absent in `shipping` |
| FFI thunk | guaranteed required | one call; inlinable across the boundary only under `[BLD-FFI-4]`'s shared-compiler case |
| Coroutine resume | guaranteed required | one indirect jump on the state discriminant (`[CORO-4]`); the frame itself does not allocate (`[CORO-5]`) |
| `str` ← foreign bytes | guaranteed required | O(n) UTF-8 validation (`[TXT-2]`); no allocation |
| `x in coll` | conditionally elidable · else guaranteed required | one `Contains.contains` call, inlinable when the impl is small; **the complexity is the impl's, not the operator's** — O(1) for `Map`/`Set`, O(n) for `Array`/`Span`/`str`, and `ember inspect --cost` names which (`[STD-8a]`) |
| `Cell` access | not observable | a load or store |
| `RefCell` borrow | guaranteed required | one word, every profile (`[CELL-9]`) |
| Effect annotations (`@noalloc`, `@deterministic`, …) | not observable | compile-time only |
| Range type (`[RNG-1]`) | not observable | erased to the representation; construction is where the check lives |
| Arena allocation | guaranteed required | a pointer bump; the reset is one store |

* `[COST-4]` `ember inspect --cost <path>` prints, for the named item, every row of
  `[COST-3]` that applies to it with its resolved class — elided or emitted, and for
  a conditionally elidable cost, which condition decided it. This is the same
  machinery `[EFF-10]`'s chains and `ember inspect --safety` already use, presented
  per item rather than per check.
* `[COST-5]` A rule **added or amended after 0.8** that introduces an implicit cost
  MUST add a row to `[COST-3]`, and `tools/rule_index.py` fails CI on such a rule
  with no row. For rules that predate this section the check is a **report, not a
  failure**: `[COST-3]` is asserted to be complete over 0.8's rule set, and any gap
  the report finds is an erratum against this section rather than a build break —
  the table is the claim being checked, so a check that failed the build would only
  measure how confident the claim was.

---

