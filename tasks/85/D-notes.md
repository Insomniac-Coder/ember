# Helper D (85% push, 2026-10-18): data-oriented design — notes

Job sheet: ECS, `std.simd` and the SIMD loop attributes, SoA, and the rules around them
(`BEN-6`, `BLD-2`, `CG-C-3b`, `CG-C-4`, `ECS-1`..`ECS-7`, `SIMD-1`, `SIMD-2`, `SIMD-4`..`SIMD-6`,
`SIMD-8`, `SIMD-9`, `SOA-1`, `SOA-4`, `STD-1`, `TST-6`). Each section below is one feature, written
like an ADR: what was built, the decisions and why, the tests, and what is left.

## 1. `@simd`, `@simd(assert)` and the per-loop verdict (`[SIMD-2]`, `[SIMD-5]`, `[SIMD-6]`)

**Before.** `@simd` on a loop was `E0900`. `[SIMD-5]`'s vectorisable form was computed
(`loop_version.rs`'s `vectorisable`) only to decide where `[SIMD-7]` groups overflow checks, and it
answered yes or no, never why. A loop calling any function was out of the form (NOT-IMPLEMENTED N4),
and so was a loop keeping a float total that `@parallel(reduce=…)` declares (N5).

**Built.**
- `@simd` and `@simd(assert)` are accepted on a `for` (`check_stmt` in typeck; anything else in the
  parentheses is `E0104`). The loop is lowered as written; the attribute marks the local that
  identifies it (the counter of each outermost counted loop it became, two for a loop picked by its
  range's width at run time, else its iterator) with `hir::LocalDecl::simd`. MIR carries the marks
  as `Body::loop_notes` (`LoopNotes { simd, declared_reductions, in_form }`); the MIR inliner carries a
  callee's notes into its caller.
- `vectorisable` now returns the first clause of `[SIMD-5]` a loop fails, in the rule's order
  (1 trip count, 2 memory accesses, 3 one offset per written view, 4 disjointness, 5 calls,
  6 effects/explicit panic/a drop that runs code, 7 ODR-088 division, 8 a bounds check left,
  9 an arithmetic check left, 10 one exit, 11 float reductions), with the place it failed and, for a
  check that remains, its `[EFF-11]` reason (`not_proven_by_analysis`). It collects every failure and
  reports the lowest-numbered, so the answer does not depend on the order the walk finds them.
  `group_overflow_checks` records a verdict on every counted loop copy (`note_verdict`); a loop's
  copies (the `[OPT-2]` checked and unchecked ones) count by the best one.
- After the loop pass the driver (`simd_verdicts`) turns each `@simd(assert)` loop not in the form
  into `E4020`, naming the clause (and the remaining check's reason), and
  `--emit-optimization-report` prints one line per `@simd` loop, in source order, after the direct
  calls (`[DSP-5]`).
- `[SIMD-6]`: a build that compiles the C then prints the host C compiler's own report, labelled
  "its opinion only, Ember's verdict is above": `ember_build::vectorisation_report` compiles the unit
  again to a scratch object with clang `-Rpass{,-missed,-analysis}=loop-vectorize`, gcc
  `-fopt-info-vec-all` or MSVC `/Qvec-report:2`, and keeps the remark lines placed in Ember source or
  in the unit (clang places a loop's remark at its function when the loop has no line of its own;
  MSVC follows the `#line`s). A failure to produce it prints "not available"; it is never an error.
  `--emit c` compiles nothing and prints no host report.
- **N4 built: calls read through.** `inline::read_through_callees` is the set `[CG-C-3]` puts in the
  inline header (`@inline`, or at most 40 statements and no foreign call, whose call boundary carries
  nothing but the call: the existing `copyable` test, so struct methods and operators are in it;
  class methods, closures with an environment and access-holding bodies are not).
  `inline::read_loop_calls` makes a copy of a body in which every call a counted loop makes to one of
  them (same float mode) is its body, through four levels. The loop pass runs on both; the copy gives
  the verdicts, and becomes the code only when it puts more loops in vectorisable form (then
  `[SIMD-7]`'s grouping covers the callee's checks too, as the rule requires). Otherwise the calls
  stay calls and the C compiler inlines them as it did. Decision: adopting the copy only when it
  gains a loop keeps every other program's C unchanged (the ADR-081 note: an inlined generic `larger`
  made clang pick a slower `max`), while the verdict always reads through, as ODR-062 says.
- **N5 built: declared reductions.** `@parallel(reduce=[s: +])` puts a compiler-only attribute
  `$reduce(s)` on its chunk's loop, which marks the chunk's copy of `s` `declared_reduction`; a float
  running total into such a local no longer fails clause 11. `@simd` written with `@parallel` moves to
  the chunk's loop.

**Tests.** `tests/conformance/SIMD-2/accept_simd_is_a_request_that_never_changes_meaning.em` (debug,
release, shipping: loops in and out of the form run as written),
`SIMD-2/reject_loops_not_in_vectorisable_form.em` (eight loops, each failing one clause: indirect
index, undeclared float total, an early exit, a text loop, a call through a function value, a
division by a variable, a view written at two offsets, a running total updated on some turns with its
`not_proven_by_analysis` reason), `SIMD-2/reject_simd_takes_no_argument_but_assert.em`,
`SIMD-5/accept_calls_and_declared_reductions_are_read_through.em` (`@simd(assert)` on loops calling a
helper twice, a `Vec3` operator, a struct method, and a declared parallel float total), and the
milestone `the_optimization_report_gives_each_simd_loop_its_verdict_then_the_host_compilers`
(`[SIMD-2]`, `[SIMD-6]`). Docs: NOT-IMPLEMENTED N4 and N5 removed; `SIMD-5`'s `unfinished` entry
removed.

**Left / not done.** A `@simd` loop that lowers to neither a counted loop nor an iterator local (none
found) gets no report line; with `assert` it is `E4020` at once (trip count). The verdict of a loop
in a generic function instantiated twice is said once, the failing instance's.
