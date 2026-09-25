---

# Part X — Effects, Contracts, Determinism and Cost

## X.1 Effects

The compiler infers, for every function, an **effect set** drawn from:

| Effect | Introduced by |
|---|---|
| `Alloc` | a heap allocation: constructing an allocating type, container growth, an allocating literal or f-string, a boxed callable value, arena growth |
| `Sync` | an atomic read-modify-write stronger than relaxed, a lock, a channel operation, thread spawn or join, retaining or releasing a `Sync` handle |
| `Lock` | acquiring a `Mutex`/`RwLock` (including `try_lock`) |
| `Block` | an operation that may wait: a blocking lock, `join`, `sleep`, a blocking receive or read |
| `Io` | file, socket and console operations |
| `Panic(Explicit)` | `panic`, `assert*`, `unwrap`, `expect`, `todo`, `unreachable`, `as!`, allocation failure |
| `RuntimeCheck(k)` | a safety check the compiler emitted rather than proved away, for `k ∈ {Aliasing, Bounds, Stale, Arithmetic}` |
| `Unsafe` | the body contains `unsafe` or the function is `unsafe fn` |
| `FFI` | a call to an `extern` function |
| `Nondet` | a result that may differ between runs or machines (`[DET-2]`) |

* `[EFF-1]` Effects are computed from each function's body and its callees', over the call graph, with
  recursive groups taking the union of their members.
* `[EFF-2]` *(changed in 0.9.9)* A call through a callable **parameter** (a generic bound, `[CLO-3]`)
  contributes the effects of the function actually passed, per instantiation. A call through an owned
  callable value, a `dyn` interface or a function pointer contributes the effects declared on its type
  (`@noalloc fn(A) -> R`), and all effects if none are declared. An interface method may declare
  contracts; its implementations MUST satisfy them (`E4010`).
* `[EFF-3]` An `extern` function carries the effects its contract declares (`@ffi(effects=[…])`), `FFI`
  at least.
* `[EFF-4]` Effects are part of a function's interface for incremental builds: a change to a callee's
  effect set re-checks its callers' contracts.
* `[EFF-9]` *(changed in 0.9.9)* `RuntimeCheck(k)` enters a function's effect set when a check of kind
  `k` survives the compiler's profile-independent proofs in its body. `Arithmetic` covers integer
  overflow, division by zero, shift amounts and negative integer exponents; `Bounds` covers indices,
  slices and chunk sizes; `Aliasing` covers class exclusivity and `RefCell`; `Stale` covers handle
  generations and `Weak.upgrade`.
* `[EFF-10]` The effect set records only which kinds occur. Per-site detail — every emitted check with
  its kind, location, mechanism and reason, and every removed check with the proof that removed it — is
  written to the safety side table that `ember inspect --safety` reads.
* `[EFF-11]` Every emitted check carries one reason, and diagnostics use it:

  | Reason | Meaning | What the programmer can do |
  |---|---|---|
  | `not_provable_in_principle` | the property depends on run-time data | nothing; the check is permanent |
  | `not_proven_by_analysis` | the property may hold but was not proved | restructure per the hint, or report the gap |
  | `requested_by_type` | the programmer chose a checked type (`RefCell`) | change the type to change the cost |
  | `inherent_to_mechanism` | the check is what the mechanism is (a generation compare) | use another mechanism |
  | `establishes_static_fact` | the check proves a property once and returns proof-carrying values (`[DSJ-1]`) | nothing |

* `[EFF-18]` Effects are independent: a blocking `Mutex.lock` has `Sync + Lock + Block`; `try_lock` has
  `Sync + Lock`; a blocking read has `Io + Block`.

## X.2 Contracts

A contract attribute states an effect the function must not have.

| Contract | Forbids | Code |
|---|---|---|
| `@noalloc` | `Alloc` | `E4001` |
| `@nosync` | `Sync` | `E4002` |
| `@noblock` | `Block` | `E4003` |
| `@noio` | `Io` | `E4041` |
| `@nolock` | `Lock` | `E4042` |
| `@nopanic(explicit)` | `Panic(Explicit)` | `E4040` |
| `@static_safe` | `RuntimeCheck(Aliasing)`, except from sites whose reason is `establishes_static_fact` | `E4030` |
| `@deterministic` | `Nondet` (Part X.3) | `E4070` |
| `@realtime` | the manifest's set, by default `@noalloc @nolock @noblock @nopanic(explicit)` | per contract |

* `[EFF-5]` Each contract in the table forbids its effect in the function's effect set; a violation is
  the listed error.
* `[EFF-6]` A contract is checked over the whole reachable call graph — callees, drop glue, default
  arguments, operator and interface implementations reached statically — and a violation names the full
  chain to the offending operation:

  ```text
  error[E4001]: @noalloc function `cull` reaches an allocation
    cull -> collect_visible -> Array.push -> ember_alloc
  ```

* `[EFF-12]` `@static_safe` permits `RuntimeCheck(Bounds)`, `(Arithmetic)` and `(Stale)`; it forbids
  only dynamic aliasing checks, because those are the checks a data-oriented inner loop can always be
  restructured to avoid.
* `[EFF-17]` `@nopanic(explicit)` is spelled with its argument; bare `@nopanic` is `E0104` with a fix-it,
  so that no reader takes it to mean "cannot abort" (`[EFF-16]`).
* `[EFF-20]` `@noio` forbids `Io`, including console output; `println` in a `@noio` function is `E4041`.
* `[EFF-21]` `@nolock` forbids `Lock`, including `try_lock`, which never blocks but still takes a lock.
* `[EFF-6a]` A `@static_safe` function may not take a parameter whose type forces a dynamic check at
  the call site (a `RefCell` by value, a `Ref`/`RefMut` guard).
* `[EFF-7]` `unsafe: @assume_noalloc(expr)` overrides the analysis for one call the compiler cannot see
  into.
* `[EFF-8]` An `override` inherits its base method's contracts.
* `[EFF-13]` A long-term access through a class handle loaded from memory is dynamically checked
  (`[EXC-3]`), so most class-heavy code cannot be `@static_safe`; the diagnostic says why and names the
  fixes (hoist the handle into a local; use value types or a query yielding `ref mut`).
* `[EFF-14]` Contracts compose; the usual inner-loop set is `@static_safe @noalloc @nosync
  @nopanic(explicit)`, one per line.
* `[EFF-15]` *(changed in 0.9.9)* Effects and contract verdicts are computed once, after the
  profile-independent proofs, and are identical in every profile (`[PHIL-13]`). The proofs that count
  are exactly those this document specifies — `[EXC-3]`, `[EXC-8]`, `[RC-2]`, `[OPT-2]`, `[RNG-4]`,
  `[SIMD-7]` and `[DSJ-1]` — and a verdict is computed **as if every one of them were applied wherever
  its conditions hold**, whether or not the emitted code keeps the check a proof would remove. So every
  implementation reaches the same verdict; an implementation may remove further checks from the code it
  emits, but that never changes a verdict. Adding a proof to the list is a language revision
  (`[VER-9]`) and makes more programs satisfy a contract, never fewer.
* `[EFF-16]` *(changed in 0.9.9)* `@nopanic(explicit)` forbids only the panics the programmer writes and
  allocation failure. Arithmetic, bounds, aliasing and stale checks are `RuntimeCheck` kinds and are
  allowed under it; the diagnostics and error page for `@nopanic(explicit)` MUST say so. A function that
  can abort in no way at all is `@nopanic(explicit)` **and** free of `RuntimeCheck` — two facts
  `ember inspect --safety` reports together. (`NonZero[T]` divisors and range facts remove division
  checks entirely, `[STD-4]`.)
* `[EFF-19]` `@realtime` names the contract set declared by the manifest of the package that declares
  the function (`[realtime] contracts = […]`), by default `@noalloc @nolock @noblock
  @nopanic(explicit)`. It is a marker for that set and not a timing guarantee. A diagnostic arising from
  it names both the failed contract and `@realtime`.
* `[EFF-23]` *(new in 0.9.9)* A contract attribute whose checking an implementation has not built is rejected with
  `E0900` (`[PHIL-12]`); it is never accepted unchecked.

## X.3 Determinism

Lock-step networking, replays and golden-image tests need two runs of a program on the same input to
compute the same bits, on one machine and across machines running the same binary.

* `[DET-1]` `@deterministic` on a function or module is a contract: `Nondet` MUST NOT appear in its
  effect set (`E4070`, naming the operation and the chain).
* `[DET-8]` `@deterministic` on a module applies to every function in it, with no opt-out. A foreign
  function is deterministic only if its declaration says `@ffi(deterministic)`, which is an asserted
  fact reported by `ember tcb`; `@deterministic` on an `extern` block is `E0104`.
* `[DET-9]` `ember inspect --deterministic <item>` prints whether the item satisfies `[DET-1]` and, if
  not, the shortest chain to each `Nondet` source; a function that relies on asserted foreign facts is
  reported as such, not as verified.
* `[DET-2]` *(changed in 0.9.9)* `Nondet` is introduced by exactly: floating-point contraction,
  reassociation or any `@fastmath` relaxation; a transcendental function from the platform library
  instead of `std.math.det`; observing a pointer or handle as an integer, or hashing one; a
  `RandomState` hasher; the wall clock, the monotonic clock, the system random source, thread and job
  completion order; reading uninitialised or padding bytes; any `extern` function not declared
  `@ffi(deterministic)`. Iterating a `Map` or `Set` is **not** `Nondet`: their order is insertion order
  (`[STD-11]`).
* `[DET-3]` A `@deterministic` function may call only `Nondet`-free functions; a callable parameter is
  checked per instantiation (`[EFF-2]`).
* `[DET-4]` `std.math.det` provides `sin`, `cos`, `tan`, `exp`, `log`, `pow`, `atan2` and `sqrt` for
  `f32` and `f64` returning the same bits on every supported target.
* `[DET-5]` Inside a `@deterministic` function the backend never contracts, reassociates or fuses; a
  `@fastmath` or `@fp(contract)` function there is `E4072`.
* `[DET-6]` `@deterministic` constrains results, not timing.
* `[DET-7]` The cross-machine claim holds for one binary. `ember build --build-id` prints a hash over the
  inputs of a reproducible build (`[BLD-13]`) so peers can confirm they run the same one.
* `[DET-10]` *(new in 0.9.9)* A deterministic program runs with a defined floating-point environment: round to nearest,
  ties to even; no flush-to-zero or denormals-are-zero; no trapping exceptions. The runtime establishes
  it at start-up and on every thread it creates, and foreign code that changes it is outside the
  guarantee.

## X.4 The cost model

* `[COST-1]` **Zero cost, defined.** An abstraction is zero-cost *for a use* whose dynamic checks are
  all proved away when the emitted code contains no instruction that equivalent hand-written C —
  C upholding the same invariant — would not contain.
* `[COST-2]` Every implicit cost is one of: **guaranteed elided** (emitting it is a defect);
  **guaranteed present**; **conditionally elided** (the compiler MAY remove it with a proof and reports
  whether it did); **implementation-defined** (not constrained, such as code size; two conforming
  implementations may differ); **not observable** (erased).
* `[COST-3]` *(changed in 0.9.9)* **The costs.**

| Cost | Class | Condition, and what the programmer can do |
|---|---|---|
| Bounds check on `a[i]` | conditionally elided | removed by loop versioning (`[OPT-2]`) and range facts; `get_unchecked` in `unsafe` |
| Overflow check on integer arithmetic | conditionally elided, **every profile** | removed by range facts; checked once per group in vectorised loops (`[SIMD-7]`); `@overflow(wrap)` or `wrapping_*` for code that wants wrapping |
| Floor `//` and `%` on signed integers | guaranteed present, small | one compare-and-adjust over C's truncation; free for a positive constant divisor; `div_trunc`/`rem_trunc` give C's |
| Division-by-zero and shift-range checks | conditionally elided | removed for constants, `NonZero` divisors and range facts |
| Retain / release pair | guaranteed elided | the cases of `[RC-2]` |
| Retain / release, surviving | guaranteed present | reported per site; atomic iff the class is `Sync` |
| Dynamic exclusivity check | conditionally elided | removed per `[EXC-3]`; hoisted per `[EXC-8]`; covered by `mut self` (`[EXC-15]`) |
| Per-field access word | guaranteed present | one `u32` per non-`Copy` field of a non-`@sync` class, and one check per such field on entry to a `mut self` method (`[EXC-19]`) |
| Stale-handle check | guaranteed present, **every profile** | the guarantee is the check; `get_unchecked` in `unsafe` |
| `RefCell` borrow | guaranteed present | one counter update |
| `Cell` access | not observable | a load or a store |
| Interface call through `dyn` | guaranteed present | one indirect call |
| Generic call, monomorphised | not observable | a direct, inlinable call |
| Generic call, shared (`[MONO-6]`) | guaranteed present | one indirect call per bound method; only above a declared instantiation budget |
| Callable parameter (`fn(A) -> R` as a parameter) | not observable | a direct call |
| Owned callable value | guaranteed present | one indirect call; one allocation if captures exceed three words (`[CLO-10]`) |
| `some I` return | not observable | the concrete type is known |
| Generator resume | guaranteed present | one jump on the state; the frame never allocates |
| Collection literal producing a heap collection | guaranteed present | one allocation (`Alloc`); a fixed-array context allocates nothing |
| String literal producing a `String` | guaranteed present | one allocation and copy; a `str` context allocates nothing |
| f-string | guaranteed present | allocates; `format_to` into a buffer does not |
| `?` converting an error into `AnyError` | guaranteed present, failure path only | one allocation (`[ERR-11]`); a concrete error type moves without allocating |
| Contract and range-type annotations | not observable | compile time only |
| Floating-point contraction off | guaranteed present | `a * b + c` is two roundings and two instructions, where C compilers often fuse them; `fma(a, b, c)`, `x.mul_add(b, c)` or `@fp(contract)` give one (`[TYP-9]`) |
| 64-bit `int` | guaranteed present | `int` is `i64`, so an `Array[int]` moves twice the bytes of C's 32-bit `int`; dense numeric data uses `i32`/`f32` elements (`L4003`) |
| Sorting floats | not observable | `sort` on floats compares by IEEE totalOrder (`[TYP-37]`), a few integer instructions per compare |
| Arena allocation | guaranteed present | a pointer bump |
| Stack probe | guaranteed present | one touch per page of a frame larger than a page (`[RT-12]`); none for smaller frames |
| FFI call | guaranteed present | one direct call for an ABI-direct C function |

* `[OPT-2]` For a counted loop over `a..b` that indexes views at `i + c` for constants `c`, with the
  views' bases and lengths invariant in the loop, the compiler MUST emit one entry test that all indices
  the loop can produce are in bounds, an unchecked body taken when it passes, and the checked body
  otherwise. Behaviour is unchanged in every case.
* `[OPT-3]` The loop bound in `[OPT-2]` may be written `s.len()`, a separate local the loop does not
  assign, or a constant; the three are treated alike.
* `[COST-4]` `ember inspect --cost <item>` prints every row that applies to an item, and for each
  conditionally elided cost whether it was removed and by which proof.
* `[COST-5]` A rule that introduces an implicit cost MUST add a row to this table; `rule_index.py` fails
  CI on a new implicit cost with no row.
