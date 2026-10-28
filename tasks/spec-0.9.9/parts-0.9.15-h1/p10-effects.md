---

# Part X — Effects, Contracts, Determinism and Cost

## X.1 Effects

The compiler infers, for every function, an **effect set** drawn from:

| Effect | Introduced by |
|---|---|
| `Alloc` | a heap allocation: constructing an allocating type (a class object, a value put in its own block by `owns`, `shares` or `thread_shared`), container growth, an allocating literal or f-string, a function value's state placed on the heap (`[CLO-10]`), arena growth |
| `Sync` | an atomic read-modify-write stronger than relaxed, a lock, a channel operation, thread spawn or join, making or ending a `thread_shared` owner or a `weakref` link to a `thread_shared` value, and opening such a link (the view it gives holds an atomic count) |
| `Lock` | acquiring a `Mutex`/`RwLock` (including `try_lock`), or holding a `Lock` with `with x.lock:` |
| `Block` | an operation that may wait: a blocking lock (`with x.lock:` included), `join` and the end of a `with scope` block, `sleep`, a blocking receive or read |
| `Io` | file, socket and console operations |
| `Panic(Explicit)` | `panic`, `assert*` (`assert_disjoint` included), `unwrap`, `expect`, `todo`, `unreachable`, `cast x as T`, allocation failure |
| `RuntimeCheck(k)` | a safety check the compiler emitted rather than proved away, for `k ∈ {Aliasing, Bounds, Stale, Arithmetic}` |
| `Unsafe` | the body contains `unsafe` or the function is `unsafe fn` |
| `FFI` | a call to an `extern` function |
| `Nondet` | a result that may differ between runs or machines (`[DET-2]`) |

* `[EFF-1]` *(changed in 0.9.15)* Effects, and the function's **summary** (what it may change, read and
  end, `[EFF-24]`), are computed from each function's body and its callees', over the call graph, with
  recursive groups taking the union of their members. An operation's effects are its own and those of
  everything it runs where it runs: the methods and function values it calls (a `Display` a formatter
  invokes, a function an adapter applies), the copies, count changes and drops it performs (ending an
  object counts as a call of its `drop`, `[DRP-8]`), and the argument and default expressions it
  evaluates. Calling a generator runs none of its body; each `next` carries the effects of the body up
  to the next `yield`, and dropping a suspended one those of the drops it performs (ODR-057).
* `[EFF-2]` *(changed in 0.9.9)* *(changed in 0.9.15)* A call through a function value contributes the
  effects of every function the program can put there. For a type parameter bounded by a function type
  (`[F: fn(Item)]` above `fn for_each(owns f: F) owns this:`) that is the function given, per instance.
  For a parameter, field, list element or local of a function type, it is every lambda and named
  function stored into that type anywhere in the program, a library's included. For an interface, `dyn`
  or `virtual` method, it is every implementation and override the program has. A foreign function
  pointer (`extern "C" fn`) contributes the effects its type declares, and all effects if none are
  declared. A contract written on a function type is checked against every function stored into it,
  with the contract's code. An interface method may declare contracts; its implementations MUST satisfy
  them (`E4010`).
* `[EFF-3]` *(changed in 0.9.15)* An `extern` function carries the effects its contract declares
  (`@ffi(effects=[…])`), `FFI` at least. What a call of one may change through the callbacks C keeps is
  `[EFF-26]`'s.
* `[EFF-4]` *(changed in 0.9.15)* **Effects and summaries are part of a function's interface.** Inside a
  package they are used freely. Across a package's edge, a fact for which a word exists counts only as
  written (a `pub` function's bounds, `E2040`), and every other worked-out fact is recorded in the
  package's interface file: each function's effect set and summary (`[EFF-24]`), with the chains of
  calls that messages name; the effects of the scoped tasks it starts; and the function values the
  package stores into a function type and the callbacks it hands to C, which a program that uses the
  package counts wherever such a value is called and at every foreign call (`[EFF-26]`). A change to any
  of them is reported as an interface change and re-checks every caller: an edit inside one function,
  or a library update (two lines of a function swapped, so that a use of a parameter falls below a line
  that clears what it views), can make a line elsewhere stop compiling, and the message names the chain
  of calls. A parent link proves itself (Part VIII) only when no other package can change it or the
  places that own its holders. What a generic function changes through its type parameters' methods is
  worked out for each instance, by the package that compiles that instance.
* `[EFF-9]` *(changed in 0.9.9)* *(changed in 0.9.15)* `RuntimeCheck(k)` enters a function's effect set
  when a check of kind `k` survives the compiler's profile-independent proofs in its body. `Arithmetic`
  covers integer overflow, division by zero, shift amounts and negative integer exponents, and the
  compare of an owner count or a weak count with its limit before it grows; `Bounds` covers indices,
  slices and chunk sizes; `Aliasing` covers the compare of `assert_disjoint` and `check_disjoint`
  (`[DSJ-1]`), the one run-time alias check there is; `Stale` covers handle generations and the test of
  a target's header that opens a `weakref` link (`exists as`).
* `[EFF-10]` The effect set records only which kinds occur. Per-site detail — every emitted check with
  its kind, location, mechanism and reason, and every removed check with the proof that removed it — is
  written to the safety side table that `ember inspect --safety` reads.
* `[EFF-11]` *(changed in 0.9.15)* Every emitted check carries one reason, and diagnostics use it:

  | Reason | Meaning | What the programmer can do |
  |---|---|---|
  | `not_provable_in_principle` | the property depends on run-time data | nothing; the check is permanent |
  | `not_proven_by_analysis` | the property may hold but was not proved | restructure per the hint, or report the gap |
  | `requested_by_type` | the programmer chose a type whose use is checked (a `weakref` link, each opening of which tests its target's header) | change the type to change the cost (a view, where the target outlives it) |
  | `inherent_to_mechanism` | the check is what the mechanism is (a handle's generation compare) | use another mechanism |
  | `establishes_static_fact` | the check proves a property once, and the names it checked are treated as having it afterwards (`assert_disjoint`, `[DSJ-1]`) | nothing |

* `[EFF-18]` *(changed in 0.9.15)* Effects are independent: a blocking `Mutex.lock`, and `with x.lock:` on
  a `Lock`, have `Sync + Lock + Block`; `try_lock` has `Sync + Lock`; a blocking read has `Io + Block`.

### What a function does

* `[EFF-24]` *(new in 0.9.15)* **What a function does is worked out from its code.** Beside its effect
  set the compiler works out, for every function, its summary: what it may change, read and end, and
  through which chain of calls. It is computed over the same call graph as `[EFF-1]`, from the
  function's own lines; from the functions it calls; for a call through a function value, an interface,
  a `dyn` or a `virtual` method, from every function the program can put there (`[EFF-2]`); from what
  the threads, tasks and jobs it starts and the callbacks it hands to C change, now or later
  (`[EFF-26]`); and from the `drop` of every object it may end (`[DRP-8]`), a function value or an
  `owns dyn I` value replaced or ended counting the `drop`s of what its state owns. A `drop` changes its
  own object as a given. The summary also records which elements of a list the function touches;
  whether it only adds at the end of a `ref mut` list parameter (`push`, `extend`) and reads or writes no
  element already in it; whether it gives the link of an object it is given a target before reading it
  (Part VIII); which of its view parameters, `this` included, it uses after a line that may end or move
  what they view; what it reaches by other names while a `ref mut` parameter is used; and which locks it
  may take, or needs held (`[THR-18]`). No word on a function's first line says any of it: `changes`,
  `reads` or another clause after a function's brackets is `E0100` ("`changes FRAME` is not Ember: the
  compiler works out what `tick` changes from its body; write `fn tick():`").
* `[EFF-25]` *(new in 0.9.15)* **The summary decides every rule that needs it, and nothing is checked
  while the program runs.** It decides: a walk of a collection and a call that may change it (`E3020`,
  `[BRW-18]`); a read-only name and a call that may change what it reads (`E3021`); a call that hands a
  function a part of what it may end or move before its last use of that parameter (`E3031`,
  `[BRW-21]`); a view of an object reached through a link and a call that may end the object (`E3060`);
  a function value stored where it would change its holder (`E3022`); a call that may take a lock this
  thread holds (`E7018`, `[THR-19]`); what tasks and threads change (`[THR-11]`, `[THR-20]`); `init`
  calling a method before every field its body reads is set (`E2100`); and what a `@cached` method's
  result depends on (`[EFF-27]`). A fact worked out from a body never changes what a line does: it
  decides only whether a line compiles, or removes a test that would always pass. A message about a
  change a call makes names the chain of calls that makes it, and no message asks for a word on a first
  line:

  ```ember
  static mut LOG: Array[String] = []

  fn note(owns msg: String):
      LOG.push(msg)

  fn tick():
      note(String("tick"))

  fn main():
      LOG.push(String("start"))
      for line in LOG:
          println(line)
          tick()
  ```

  ```text
  error[E3020]: `tick()` (line 13) adds to `LOG` (through `note`, line 4) while this loop walks `LOG` (line 11); call `tick()` after the loop: move line 13 below the loop
  ```

* `[EFF-26]` *(new in 0.9.15)* **A call that runs a function makes its changes; storing one does not.** A
  call through a function value makes the changes of every function it may run, and a function value
  given to a call counts as an argument that changes every place its functions change and every real
  variable they capture (`[BRW-11]`). A call that runs a function by itself counts as making everything
  that function changes, now or later: `thread.spawn`, `scope.spawn`, `jobs.submit` and a job scope's
  `submit` (Part XI); a library function that calls the function it is given
  (`scores.for_each(fn(s): LOG.push(copy of s.name))` changes `LOG`); whatever drives an iterator
  adapter that keeps one (`for x in it:`, `it.sum()`); and a foreign function's parameter whose contract
  says `callback(retained)` or `callback(once)`, through which C keeps the function and runs it during
  later foreign calls (a `callback(borrowed)` parameter runs it during the call, as `for_each` does).
  Putting a function value in a field, a list, a local or an iterator adapter changes nothing the
  function changes; the call that later runs it does. **A foreign call may run the callbacks C keeps**:
  C code cannot be read, so a call of a foreign function counts as possibly running every callback C
  keeps, on the threads their contracts allow, and makes the changes they make, for its caller and every
  function that calls it, unless the foreign function's declaration says `@ffi(no_callbacks)`, a fact
  the programmer vouches for (`[TCB-1]`; Part XVI says which Ember code C may run). A generic body's
  calls of its type parameters' methods make the changes those methods make for the types it is given,
  worked out for each instance.
  Its message: E3020 ("`scores.for_each(...)` (line 4) pushes to `LOG` (through the function it is given) while this loop walks `LOG` (line 3); call it after the loop: move line 4 below the loop").
* `[EFF-27]` *(new in 0.9.15)* **What a `@cached` result depends on.** A `@cached` method's result
  depends on the fields its body reads, worked out from its body (`[EFF-24]`), and the kept result is
  cleared by every change, through any name, to one of them: the compiler places the clear at each line
  that may make such a change. A change that does not pass through the method's object cannot be cleared
  that way, so a `sharedref mut` link, or a `shares mut` owner, made into anything a `@cached` method of
  its holder reads is `E2117` at the line that makes it ("`s.items[0]` is read by `Shelf.first_name`,
  which is `@cached` and would not see a change made through this link; link the shelf instead,
  `sharedref mut s`, or remove `@cached`"); a link to the whole holder is allowed, since every change
  through it passes through the holder. A reload that changes a `@cached` method's body clears every
  kept result of its type (Annex B).

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
| `@no_alias_checks` | `RuntimeCheck(Aliasing)` | `E4030` |
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

* `[EFF-12]` *(changed in 0.9.15)* `@no_alias_checks` permits `RuntimeCheck(Bounds)`, `(Arithmetic)` and
  `(Stale)`; it forbids only `RuntimeCheck(Aliasing)`, the run-time compare of `assert_disjoint` or
  `check_disjoint`, because a data-oriented inner loop can always do without it: spans the compiler
  proves apart need no compare, and a copy needs none (`E4030`: "`mix` is `@no_alias_checks`, and
  `assert_disjoint(d, s)` (line 18) compares the two spans while running; blend from a copy, which needs
  no check: instead of lines 16 to 19 write `old = copy of tracks[src_name]` and
  `blend(ref mut tracks[dst_name].as_mut_span(), old.as_span())`, or remove `@no_alias_checks`").
  `@static_safe` is `E0104` ("`@static_safe` is now `@no_alias_checks`: it forbids run-time alias checks
  only; bounds and overflow checks still run; write `@no_alias_checks`").
* `[EFF-17]` `@nopanic(explicit)` is spelled with its argument; bare `@nopanic` is `E0104` with a fix-it,
  so that no reader takes it to mean "cannot abort" (`[EFF-16]`).
* `[EFF-20]` `@noio` forbids `Io`, including console output; `println` in a `@noio` function is `E4041`.
* `[EFF-21]` `@nolock` forbids `Lock`, including `try_lock`, which never blocks but still takes a lock.
* `[EFF-7]` `@assume_noalloc(call)`, an expression admitted only inside `unsafe` (`E0104` elsewhere),
  has the call's type and value and overrides the analysis for that one call, which the compiler
  cannot see into: the call is taken to carry no `Alloc`, and the programmer is answerable for it, as
  for any `unsafe` obligation (`[UNS-7]`) (ODR-060).
* `[EFF-8]` An `override` inherits its base method's contracts.
* `[EFF-14]` *(changed in 0.9.15)* Contracts compose; the usual inner-loop set is `@no_alias_checks
  @noalloc @nosync @nopanic(explicit)`, one per line.
* `[EFF-15]` *(changed in 0.9.9)* *(changed in 0.9.15)* Effects and contract verdicts are computed once,
  after the profile-independent proofs, and are identical in every profile (`[PHIL-13]`). The proofs
  that count are exactly those this document specifies — `[OPT-2]`, `[RNG-4]`, `[SIMD-7]` and the
  parent-link proof of Part VIII, which removes the header test of a parent-link read — and a verdict is
  computed **as if every one of them were applied wherever its conditions hold**, whether or not the
  emitted code keeps the check a proof would remove. So every implementation reaches the same verdict;
  an implementation may remove further checks from the code it emits, but that never changes a verdict.
  Adding a proof to the list is a language revision (`[VER-9]`) and makes more programs satisfy a
  contract, never fewer.
* `[EFF-16]` *(changed in 0.9.9)* *(changed in 0.9.15)* `@nopanic(explicit)` forbids only the panics the
  programmer writes and allocation failure. Arithmetic, bounds, aliasing and stale checks, and a count's
  compare with its limit, are `RuntimeCheck` kinds and are allowed under it; the diagnostics and error
  page for `@nopanic(explicit)` MUST say so. A function that is `@nopanic(explicit)` **and** free of
  `RuntimeCheck` — two facts `ember inspect --safety` reports together — has no panic the effect model
  knows of; it can still stop on what the model does not cover (`[PHIL-11]`: exhaustion outside
  `Alloc`, a hardware fault, foreign code breaking its contract) (ODR-057). (`NonZero[T]` divisors and
  range facts remove division checks entirely, `[STD-4]`.)
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
* `[DET-2]` *(changed in 0.9.9)* *(changed in 0.9.15)* `Nondet` is introduced by exactly: floating-point
  contraction, reassociation or any `@fastmath` relaxation; a transcendental function from the platform
  library instead of `std.math.det`; observing a pointer or an object's address as an integer, or
  hashing one; making a `RandomState`, directly or as a `Map`'s or `Set`'s hasher; the wall clock, the
  monotonic clock, the system random source, thread and job completion order; reading padding bytes,
  recorded where a pointer to a type with padding is reinterpreted as a pointer to another type; any
  `extern` function not declared `@ffi(deterministic)`. Iterating a `Map` or `Set` is **not** `Nondet`:
  their order is insertion order (`[STD-11]`). A reduction of scoped tasks, combined in spawn order, is
  not `Nondet` either (`[THR-21]`). Reading uninitialised memory is not `Nondet` but undefined, outside
  Safe Ember (`[PHIL-10]`): an `unsafe` obligation (ODR-136).
* `[DET-3]` *(changed in 0.9.15)* A `@deterministic` function may call only `Nondet`-free functions; a
  call through a function value counts every function the program can put there, and a function given
  through a type parameter is checked per instance (`[EFF-2]`).
* `[DET-4]` `std.math.det` provides `sin`, `cos`, `tan`, `exp`, `log`, `pow`, `atan2` and `sqrt` for
  `f32` and `f64` returning the same bits on every supported target. Each takes either type and
  answers in it (`det.sin(x)` of an `f32` is an `f32`); an `f32`'s result is its `f64` result
  rounded once. Each is within one unit in the last place of the exact result, `atan2` within 1.3;
  `sqrt` is IEEE 754's, correctly rounded. An integer is converted by the program (ODR-046).
* `[DET-5]` *(changed in 0.9.15)* Inside a `@deterministic` function the backend never contracts,
  reassociates or fuses; a `@fastmath` or `@fp(allow_fma)` function there is `E4072` ("`dot` is
  `@deterministic`, and a fused multiply-add rounds once where two operations round twice; remove
  `@fp(allow_fma)`"). `@fp(contract)` is `E0104` ("`@fp(contract)` is now `@fp(allow_fma)`: it lets the
  compiler fuse `a * b + c` into one fused multiply-add; write `@fp(allow_fma)`").
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
* `[COST-3]` *(changed in 0.9.9)* *(changed in 0.9.10)* *(changed in 0.9.15)* **The costs.** Everything
  the memory model checks is checked when compiling; what runs is only what this table lists.

| Cost | Class | Condition, and what the programmer can do |
|---|---|---|
| Bounds check on `a[i]` | conditionally elided | removed by loop versioning (`[OPT-2]`) and range facts; `get_unchecked` in `unsafe` |
| Overflow check on integer arithmetic | conditionally elided, **every profile** | removed by range facts; checked once per group in vectorised loops (`[SIMD-7]`); `@overflow(wrap)` or `wrapping_*` for code that wants wrapping |
| A second pass of a loop, then `retain` | conditionally fused | a loop that queues removals and the `retain` that follows it are fused into one pass where nothing between them reads the list; otherwise two passes (1.71x of one-pass C as written, 1.00x fused) |
| Floor `//` and `%` on signed integers | guaranteed present, small | one compare-and-adjust over C's truncation; free for a positive constant divisor; `div_trunc`/`rem_trunc` give C's |
| Division-by-zero and shift-range checks | conditionally elided | removed for constants, `NonZero` divisors and range facts |
| Owner count +1 / -1 | guaranteed present | only where a line makes or ends a `shares`, `shares mut` or `thread_shared` owner (`[THR-1]`), and nowhere else; a plain add for `shares`, an atomic one for `thread_shared`; each +1 is checked against the count's limit, one compare and a branch never taken (`[COST-6]`) |
| `weakref` link made, given another target, or ended | guaranteed present | one add or one subtract of the target's weak count, the add checked against its limit; atomic for a `thread_shared` target; a parent link pays it too, one add when `child.parent = weakref this` runs and one subtract when the link changes or the child ends |
| Opening a `weakref` link (`if w exists as p:`) | conditionally elided | one test of the target's header (during the target's `drop` the object is already ended, and the same test finds it gone); a parent-link read the compiler proves (Part VIII) is a load and a test for no link |
| A view of a `thread_shared` object opened from a `weakref` link | guaranteed present | one atomic count held for the view's life, since another thread may end the last owner at any moment |
| A walk up a tree across a call that may change nodes | guaranteed present | per step, one test of the node's header and one move of the link's weak count (a subtract and an add); where the node's class has no `drop` of its own and no other thread can end its nodes, the walk runs in a deferred-drop region and the count moves go, leaving the header test; a walk whose visit function changes no node keeps its views and pays nothing per step |
| A view of an object reached through a link; a `sharedref mut` link | not observable | checked when compiling: no count and no deferred-drop region around the view |
| Stale-handle check | guaranteed present, **every profile** | the guarantee is the check; `get_unchecked` in `unsafe` |
| Kind test or cast | guaranteed present | one type-id compare per `if s is Circle as c:` and per `cast s as Circle` (Part VIII) |
| `assert_disjoint` | guaranteed present | two compares where the program writes it (two spans of one list given to one call, one of them changeable); the call is then compiled with noalias; none when the compiler knows the spans apart |
| `Cell` access | not observable | a load or a store |
| Interface call through `dyn` | guaranteed present | one indirect call |
| Virtual method call | conditionally elided | a load of the slot and one indirect call (`[DSP-2]`); direct when one implementation is reachable (`[DSP-5]`) |
| Generic call, monomorphised | not observable | a direct, inlinable call |
| Generic call, shared (`[MONO-6]`) | guaranteed present | one indirect call per bound method; only above a declared instantiation budget |
| A function given through a type parameter (`[F: fn(Item)]`) | not observable | a direct, inlinable call; one instance per function given |
| A call through a function value (a parameter, field, list element or local of a function type) | guaranteed present | one indirect call; a call through a list element or a field first copies the value's two words, its code and its state, out of it |
| A function value's state | guaranteed present | a variable kept with a capture word (`owns`, `copy of`, `shares`, `thread_shared`, `weakref`) lives in the function value's own state (`[CLO-10]`), and costs nothing beyond it; a captured real variable stays at its own address and costs nothing; a function that ends its own state while it runs and then uses it holds one count on that state across the call |
| `some I` return | not observable | the concrete type is known |
| Generator resume | guaranteed present | one jump on the state; the frame never allocates; a `for` over a list inside the generator reads its position again after each resume |
| Collection literal producing a heap collection | guaranteed present | one allocation (`Alloc`); a fixed-array context allocates nothing |
| String literal producing a `String` | guaranteed present | one allocation and copy; a `StrView` context allocates nothing |
| f-string | guaranteed present | allocates; `format_to` into a buffer does not |
| `?` converting an error into `AnyError` | guaranteed present, failure path only | one allocation (`[ERR-11]`); a concrete error type moves without allocating |
| `copy of`; `owns`, `shares` or `thread_shared` before a new value | guaranteed present | `copy of` makes a copy by the type's `Clone`; `owns`, `shares` and `thread_shared` put a new value in its own block, one allocation; nothing is copied or allocated where no line says so |
| Contract and range-type annotations | not observable | compile time only |
| Floating-point contraction off | guaranteed present | `a * b + c` is two roundings and two instructions, where C compilers often fuse them; `fma(a, b, c)`, `x.mul_add(b, c)` or `@fp(allow_fma)` give one (`[TYP-9]`) |
| 64-bit `int` | guaranteed present | `int` is `i64`, so an `Array[int]` moves twice the bytes of C's 32-bit `int`; dense numeric data uses `i32`/`f32` elements (`L4003`) |
| Sorting floats | not observable | `sort` on floats compares by IEEE totalOrder (`[TYP-37]`), a few integer instructions per compare |
| Arena allocation | guaranteed present | a pointer bump |
| A value made `using` a memory manager | guaranteed present | each allocation and free of the value is the manager's own work (an arena: a pointer bump, nothing freed one by one); a value whose manager may have several instances keeps one word, its instance's address, so that its growth and its end go back to that instance; none for `Global` or `System` |
| Allocation | guaranteed present | a call that allocates (a collection growing, a `String` built), reported per site; `@noalloc` refuses it (`[EFF-1]`) |
| `contains` | guaranteed present | the `contains` an `x in c` selects: linear for an array or view, constant for a set or map (`[STD-8a]`) |
| A lock | guaranteed present | where the program takes one (`with x.lock:`, `m.lock()`): the lock C takes; nothing per use of a `guarded by` field (`[THR-18]`) |
| Scoped tasks and their reductions | guaranteed present | one private accumulator per task and one combine per task at the join, as C's per-thread counter; no atomic per change (`[THR-21]`) |
| `@cached` method | guaranteed present | one flag test per call, and one store, clearing the flag, at each line that may change a field its body reads (`[EFF-27]`); on an object several threads reach, the result is filled once under a once-flag |
| Stack probe | guaranteed present | one touch per page of a frame larger than a page (`[RT-12]`); none for smaller frames |
| FFI call | guaranteed present | one direct call for an ABI-direct C function |
| A C callback | guaranteed present | the checked trampoline (Part XVI): C holds a tagged number, never an address; each call compares the callback's generation, and, in every profile, the thread of a `threads(creator)` callback and the used-up mark of a `callback(once)` one |
| What a function changes, reads and ends; the thread rules; the arena checks | not observable | all checked when compiling (`[EFF-24]`); a library function given a function is one instance per function, as Rust's generic `F: FnMut`; a call that names one whole variable twice is compiled against its own copy of the function (`[BRW-11]`), one more copy of its code and nothing while running |

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
* `[COST-6]` *(new in 0.9.15)* **What stops the program, and what it says.** Of the costs above, these
  may stop the program, in every profile (`[PRF-4]`); each says so in the names of the program that
  stopped (the example's names in quotes):

  | Check | The stop's text |
  |---|---|
  | `assert_disjoint` (`[DSJ-1]`) | "`assert_disjoint(d, s)` on line 11: the two spans overlap (`dst_name` and `src_name` name one track)" |
  | a one-thread owner count at its maximum | "too many owners of a `Player` (4,294,967,295)" |
  | a one-thread weak count at its maximum | "too many `weakref` links to a `Player` (4,294,967,295)" |
  | a `thread_shared` owner count at its limit (`[RC-4]`) | "too many owners of a `Log` (2,147,483,647)" |
  | a `thread_shared` weak count at its limit (`[RC-4]`) | "too many `weakref` links to a `Log` (2,147,483,647)" |
  | the type-id compare of `cast s as Circle` | "`cast s as Circle` on line 9: `s` is a `Square`, not a `Circle`" |
  | the checked trampoline: a call after the token ended | "C called the callback registered on line 16 after its token ended; it was unregistered" |
  | the checked trampoline: a `callback(once)` function called a second time | "C called the callback registered on line 29 a second time; it was registered `callback(once)`, so it may run only once" |
  | the checked trampoline: a `threads(creator)` callback on another thread, checked in every profile | "C called the callback registered on line 16 on another thread, but its contract says `threads(creator)`" |
  | a bounds check | "index 3 is out of bounds for a length of 3; use `.get(i)` for an `Option`" |
  | adopting one live object twice (`[FFI-36b]`) | a panic naming both places that adopted it |
  | a GPU draw reading past what is bound (`[GPU-12]`) | the text `[GPU-12]` gives |

  The number a count's text names is the limit that count reached: for a one-thread count, its
  maximum; for a `thread_shared` count, the limit Part VIII keeps below its maximum. The two count
  limits are tested with a count that is easy to fill (`[TST-31]`); the language has no test-only
  shortcut.
