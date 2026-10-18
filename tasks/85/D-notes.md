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

## 2. `std.simd` (`[SIMD-1]`, `[SIMD-4]`, `[SIMD-8]`, `[SIMD-9]`)

**Built.**
- `std/src/simd.em` declares the 30 vector types (`f32x4/8/16`, `f64x2/4/8`, and for each of 8, 16,
  32, 64-bit lanes the signed and unsigned vectors of 128, 256 and 512 bits) and the 12 mask types
  (`mask{bits}x{lanes}`), each a `@derive(Copy)` struct of one private `lanes: [T; N]` array with
  `LANES`, and `from_array` / `to_array` written in Ember; `native_lanes[T]()` is
  `16 // size_of[T]()` (every C-backend target has 16-byte vector registers; the C is built for the
  baseline instruction set). The compiler knows the types by name (`ember_types::SimdShape::of`).
- Type checking (`synth_simd_*` in typeck): `V.splat`, `V.load(_unchecked)`, `V.gather(_unchecked)`
  (any integer index vector of as many lanes), `v.store(_unchecked)`, `v.scatter(_unchecked)`,
  `reduce_add/min/max`, `shuffle([...])` (a list literal of constants below the lane count, else
  `E2020`), `fma`, `min`, `max`, `abs`, `sqrt`, `rsqrt_approx`, `rcp_approx`, `wrapping_add/sub/mul/neg`,
  the operators `+ - *` (and `/` on floats, `& | ^` on integers and masks), unary `-`, the six
  comparisons into the mask type, and on masks `any`, `all`, `select`, `M.splat`. Each is one
  `Builtin::Simd { op, vector }`. Integer lanes take the function's overflow policy
  (`active_overflow`: checked, wrap or saturate), so `@overflow(wrap)` reaches vector operators as
  `[SIMD-8]` says; float lanes carry `Wrap` (they check nothing). The unchecked forms need `unsafe`
  (`E3100`). Effects: checked memory operations `RuntimeCheck(Bounds)`, unchecked ones `Unsafe`,
  checked integer arithmetic `RuntimeCheck(Arithmetic)`.
- C: `runtime/ember_rt/templates/ember_simd.h.in` (rendered to `include/ember_simd.h` by
  `generate_runtime.py`, which now renders every further `*.h.in`; `check_branding.py` treats it as
  generated). Each vector struct is emitted as `struct V { union { _Alignas(size) <lanes array>
  lanes; <registers> v[size/16]; }; }`, and after the structs one macro per type the program
  operates on instantiates its operations as `V_<op>` static inline functions (masks first; one
  gather/scatter pair per vector and index type used). With SSE2 (every x86-64), splat, loads,
  stores, float `+ - * /`, `min`, `max`, `sqrt` (IEEE-exact), the approximations (`rsqrtps`/`rcpps`),
  integer `& | ^` and wrapping `+ -` are written a register at a time, each register's statement
  written out (no loop: MSVC keeps a rolled two-register loop's vectors in memory, 3.4x the C);
  everything else, and every operation elsewhere, works lane by lane. Checked integer operations
  compute every lane and panic once after (`ember_panic_overflow` naming the operator); a checked
  load/store/gather/scatter panics through the new `ember_panic_lane_bounds` ("lane 2: index 4 is
  out of bounds for a length of 4"). Reductions add lane 0 first, then each next lane, so every
  target gives the same answer. `fma` is C's `fmaf`/`fma` (one rounding). Layout: `size_of`/
  `align_of` give the size for both (`ember_types::layout`).
- `[SPN-2]` slicing a `MutSpan` (was `E0900`, needed by Part XII.2's own `out[i..i + 8]`): the part
  is a mutable reborrow of the base, built as `Slice` over `reborrow_span`.
- `[SIMD-9]`'s "a load or store at a loop's induction variable is covered by the entry test of
  `[OPT-2]`": `loop_version.rs` now versions (1) a slice's end check (`end <= len`, recognised from
  the assert's comparison, taken as `end - 1 < len`), resolving the length of the view copy or
  reborrow the slice makes in the loop to the view's own place (`copied_view_len`), and (2) a
  stepped loop's value (`for i in (a..b).step_by(k)`: `RangeNth(start, step, k)`,
  `IndexBound::Stepped`), whose entry test checks `step >= 1`, the first index, and the last
  value `start + step * (limit - 1)` (computed in `u64`, exact because it is one of the range's
  values) plus the offset against the length. In the unchecked copy, a slice whose end check was
  removed is marked `nonempty` (`Builtin::Slice { nonempty }`): its end is at least 1, so its view
  is not empty and its pointer is not null, and the C takes `ptr + lo` with no test for an empty
  view's null pointer. `Builtin::Simd`, `RangeNth` and `RangeCount` write no view header
  (`loop_writes`).

**Speed** (Part XII.2's `scale`, 4096 floats eight at a time, 200 thousand times:
`bench/probes/simd_scale.em` against `simd_scale.c`'s plain loop, release, performance cores, mains
power; other helper agents were building and testing on the machine at the same time, so these
are within a few percent): clang **x1.03**, MSVC **x1.09** (from x1.14 and x3.40 before the work
above). The fast loop is two loads, two multiplies, two stores and the counter on both compilers;
MSVC's twin unrolls twice as far because the twin's length is a constant it can see (Ember's is a
list's, built by `push`). Not chased further: an owner-question-free next step would be to give the
fast copy's trip count to MSVC as a multiple of 2.

**Tests.** `SIMD-1/accept_vector_and_mask_types.em` (Part XII.2's `scale`, every type's `LANES` /
`reduce_add`, `native_lanes`, `size_of`/`align_of`), `SIMD-4/accept_reductions_shuffles_gathers_fma_and_approximations.em`,
`SIMD-4/reject_a_shuffle_that_is_not_a_constant_list.em`, `SIMD-8/accept_lane_wise_operators_follow_the_scalar_rules.em`
(debug, release, shipping; wrap, saturate, `wrapping_add`, comparisons, `fma` against `*` then `+`),
`SIMD-8/run_fail_an_overflowing_integer_lane_panics.em`, `SIMD-9/accept_checked_and_unchecked_memory_operations.em`,
`SIMD-9/run_fail_a_gather_names_its_lane.em`, `SIMD-9/run_fail_a_short_store_panics.em`,
`SIMD-9/reject_the_unchecked_forms_outside_unsafe.em`,
`SIMD-9/accept_a_load_at_the_induction_variable_needs_no_check_in_the_loop.em` (release:
`assert-c-count` two loads, two slice-end checks: the versioned copy has none). gcc (WSL) and clang
compile the instantiated header for all 42 types with `-std=c11 -Wall -Wextra` clean.

**Left.** Shifts on integer vectors; integer vector division; a vector type in generic code
through `Add` and the other operator interfaces (the operators are the compiler's, not interface
implementations); `select` and comparisons are lane by lane in C (no SSE path).
`STD-1`'s entry now names only `std.arena`.
