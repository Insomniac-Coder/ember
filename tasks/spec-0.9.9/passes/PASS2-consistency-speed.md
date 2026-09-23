# Pass 2 — Consistency and C-like speed of 0.9.9_Hardened_1

**Questions:** (1) where do two parts of 0.9.9_Hardened_1 disagree, or leave a case where two
conforming implementations could accept different programs or give different results? (2) where does
the text cost speed that equivalent C would not pay, without naming the cost or offering the fast
form?

**Method.** Rules that interact across Parts were read against each other: profiles against every
debug-only behaviour; contract verdicts against the proofs they rely on; grammar against the examples;
the prelude against every name the examples use without an import; each `*(changed)*` rule against the
rules citing it. For speed, every construct in a hot loop was followed to its emitted C: arithmetic,
indexing, iteration, calls, allocation, floating point and dynamic checks. Automated scans looked for
leftovers of 0.9.8 (`::`, `Callable[…]`, `PartialOrd`, call-site modes, version words, engine names).

**Result.** 16 consistency findings (5 of which let implementations disagree on accepted programs) and 9
speed findings.

## Consistency

| Id | Sev | Where | Finding | Fix |
|---|---|---|---|---|
| CS-01 | S1 | `[VER-9]` vs this revision's purpose | `[VER-9]` says a hardening changes no accepted program, yet closing the holes of Pass 1 must reject programs H1 accepts | a hardening may reject a program only to close a soundness hole, and lists each such change |
| CS-02 | S1 | `[EFF-15]` vs `[EXC-3]`, `[SIMD-7]` | a contract verdict relies on proofs that are only permissions (`[EXC-3]` "MAY remove", `[SIMD-7]` "MAY check once per group"), so two implementations can disagree on `@static_safe` and `@simd(assert)` | verdicts are computed as if every listed proof were applied wherever its conditions hold, whether or not the emitted code keeps the check |
| CS-03 | S1 | `[SIMD-5]` vs `[SIMD-7]` | vectorisable form requires arithmetic checks to be "removed or grouped", but grouping is optional | follows from CS-02: the form is defined by what may be grouped |
| CS-04 | S2 | `[PRF-1]` vs `debug_assert` (§VI.6, `[PRF-3]`) | `[PRF-1]` says profiles panic in the same places "with no exceptions", but `debug_assert` is checked only in `debug` | `[PRF-1]` names `debug_assert` as the one check that is profile-dependent by definition (side-effect-free by `W2016`), so it never changes a correct program's result |
| CS-05 | S2 | `[BRW-3]` vs `[FN-2a]` | the two-phase-borrow example writes `f(mut v, v[0])`, a call-site mode `[FN-2a]` forbids | `f(v, v[0])` with the mode in `f`'s signature |
| CS-06 | S2 | `[MOD-5]` vs Parts VI, XV, Annex A | `Generator` is used in signatures without an import and is not in the prelude | add `Generator` to the prelude |
| CS-07 | S2 | `[LEX-17a]` vs `[LEX-17]` | `W2015` fires on any `f32` literal not exactly representable (`0.1f32`), even when the program chose `f32`; 0.9.8 warned only for a default-chosen `f32` or visibly lost digits | warn only when a literal has more significant digits than the type keeps (more than 9 for `f32`) |
| CS-08 | S2 | `[THR-8]`, `[THR-9]` | `Weak` of a class handle has no stated `Send`/`Sync` | `Weak[C]` is `Send` and `Sync` exactly when `C` is `@sync` |
| CS-09 | S2 | `[PAR-3]` vs `[STD-5]`, `[TYP-9]` | a float reduction in `@parallel(reduce=…)` combines chunk results in chunk order, which differs from the serial left-to-right sum; the text calls the operator "associative" | state the result: per-chunk left-to-right, then chunks in order; deterministic, not equal to the serial sum for floats |
| CS-10 | S2 | `[CLO-2]`, `[CLO-3]` | an owned callable value that writes its captured state needs "a mutable place to be called", but `fn(A) -> R` does not say whether a value writes its captures, so a caller cannot know | calling any owned callable value needs a mutable place (a class field call is a write access, MS-12); a parameter-position callable keeps `[CLO-2]` |
| CS-11 | S3 | `[HND-2]` | names an engine ("RageV's") in a rule, against the document's scope | "a 20-bit index and 12-bit generation in a `u32`" |
| CS-12 | S3 | `[OWN-6]` | `mem.take(mut place)` reads like a call with a mode | write signatures: `mem.take(mut place: T) -> T` |
| CS-13 | S3 | `[SOA-6]`, `[ECS-3]` | proxies and queries yield `ref`/`ref mut` items; what a query iteration borrows is stated for `iter_mut` only | a query's `iter` borrows `q` shared and needs only reads; `iter_mut` needs `mut q` |
| CS-14 | S3 | §VIII.3 | messages name a field (`write access to Node.children while a read access is active`) while the access word is per object | per-field state (PASS3 E-01) makes the message exact |
| CS-15 | S3 | `[STD-10]` vs `[ERR-13]` | console functions panic on end of input, an explicit exception to "world failures return `Result`" | keep, and cite `[STD-10]` in `[ERR-13]` as the one exception |
| CS-16 | S3 | Annex A vs `[TST-6]` | Annex A is marked `ember,fragment` while `[TST-6]` says its fixture compiles | the fixture declares the items the fragment names and is compiled; the printed block stays a fragment |

## C-like speed

| Id | Sev | Where | Finding | Fix |
|---|---|---|---|---|
| SP-01 | S2 | `[CTL-3]` | the guaranteed counted-loop lowering covers `enumerate`, `zip`, `take`, `skip`, `copied`, `rev` but not `step_by`, so §XII.2's own SIMD example (`(0..n).step_by(8)`) iterates through `next` calls | add `step_by` over ranges and views to the guaranteed lowering |
| SP-02 | S2 | `[RNG-4]`, `[TYP-8]` | nothing guarantees that the induction variable of `for i in a..b` carries the range `a..b`, which is what removes the overflow check on its increment and the bounds checks it feeds | the loop variable of a range loop carries the range fact `a <= i < b` |
| SP-03 | S2 | `[SIMD-7]` | grouped overflow checks are optional; without them, checked integer loops never vectorise, so `int` code is slower than C by an implementation's choice | grouping is required for loops in vectorisable form (CS-02/CS-03) |
| SP-04 | S2 | `[COST-3]` | the cost table has no row for floating-point contraction being off: `a * b + c` is two roundings and two instructions where C compilers default to one on FMA hardware | add the row, naming `fma`, `mul_add` and `@fp(contract)` as the fast forms |
| SP-05 | S2 | `[COST-3]`, `[TYP-1]` | `int` is 64-bit; an `Array[int]` of dense data moves twice the bytes of C's 32-bit `int` | add the row: dense numeric data should use `i32`/`f32` element types; `ember lint` gets `L4003` for a large `Array[int]` in a hot loop whose values fit 32 bits |
| SP-06 | S3 | §VIII.3 | the per-object access word turns a read of one field and a write of another into a panic, forcing programs into copies and restructuring (see PASS3 E-01) | per-field access state for non-`Copy` fields |
| SP-07 | S3 | `[HASH-2]` | the default hasher's speed is unconstrained | require a hasher at least as fast as FxHash-class hashing on integer keys, still fixed-seed |
| SP-08 | S3 | `[TYP-37]` | sorting floats by totalOrder is a bit-twiddled compare, slightly slower than `<`; unnamed | add to the cost table (not observable per element; stated for completeness) |
| SP-09 | S3 | `[MS-14]` in Pass 1 | stack probes cost one touch per page of a large frame | named in the cost table when adopted |

## Checked and consistent

Profiles against safety checks (`[PRF-1]`, `[PRF-3]`); every `(changed)` rule against its citations
(`tools/check_citations.py`); rule ids against 0.9.8 (`tools/check_ids.py`); diagnostic codes both ways
(§XVII.9); every example's syntax (`tools/check_examples.py`); the prelude against the names the
examples use (one gap, CS-06); the cost table against every implicit cost introduced by this revision
(`[COST-5]`), apart from SP-04 and SP-05.
