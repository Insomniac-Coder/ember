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

## 3. `@soa(flatten)` and `ArenaSoA[T]` (`[SOA-1]`, `[SOA-4]`)

**`@soa(flatten)`.** The attribute is built (`@soa` takes only `flatten`, `E0104` otherwise); a
struct's flattened fields are recorded in the type table (`mark_soa_flatten`), and
`TypeTable::soa_columns` gives an `SoA`'s columns: one per field, a flattened struct field split
into one per field of its own, recursively; each column has the path of field indices from the
element, a name (`pos__x`, the C field `col_pos__x`) and a type. Everything that took columns to be
the element's fields now goes through that list: the `SoA` struct's pointer fields, column views,
element and proxy places, gathers (`get`, `load`, a flattened field read whole), scatters
(`ps[i] = v`), and the C helpers (`push`, `pop`, `swap_remove` reach a column's value at `v.pos.x`).
In the source, `ps.pos.x` is a column (`MutSpan[f32]`), `ps[i].pos.x` and `p.pos.x` places in it,
`ps[i].pos` a value gathered from the three, and `ps[i].pos = v` (or `op=`) writes the three. A
flattened field reached as the base of a further field access is a group (`soa_groups`, by the
span of the expression standing for it); `ps.pos` used whole has no column (`E2020`). Flattening a
field that is not a struct is `E0104` where an `SoA` of it is made. Two columns of one flattened
field are distinct places, borrowed mutably at once, and distinct columns for `[SIMD-3]`.

**`ArenaSoA[T]`** (`std.collections`, not in the prelude, as `[ARN-5]` puts its arena
containers): the same compiler-built struct as `SoA[T]` with the arena it borrows as a last field
(so it is a view and cannot outlive the arena: `E3061`, `E3021`), no drop, and origin `ArenaSoA`.
`ArenaSoA[T].with_capacity(arena, n)` (`SoAOp::WithCapacityIn`) takes one 64-byte-aligned block for
`n` elements from the arena (`ember_arena_alloc_uninit`); `push` (`SoAOp::TryPush`) returns
`Err(CapacityError.Full)` past capacity and changes nothing; `capacity()`; no `reserve`; elements
that need drop are `E3090` at construction. Columns, elements, proxies, `get`, `pop`,
`swap_remove`, `clear` are an `SoA`'s. `std`'s `retain` and `sort_by_key` stay `SoA`'s only (they
are written over `SoA[T]`; the spec names no operation list for `ArenaSoA`).

**Tests.** `SOA-1/accept_a_flattened_field_is_a_column_per_field.em`, `SOA-1/reject_flatten_misused.em`,
`SOA-4/accept_an_arena_soa_has_a_fixed_capacity.em`, `SOA-4/reject_an_arena_soa_misused.em`,
`SOA-4/reject_an_arena_soa_outliving_its_arena.em`. `SOA-1` and `SOA-4` leave the `unfinished` list.
Speed: a flattened column is a column like any other (the same views and loops); not timed apart.

## 4. Defects found on the way to the ECS, fixed (not numbered: the coordinator numbers them)

- **A generic struct could not name a generic class** declared after it, or in a module collected
  later (`struct Holder[T]: store: Store[T]` was "cannot find type `Store`"): generic structs were
  collected before generic classes, and only structs were collected on demand. Generic classes are
  now noted with the structs and collected on demand where a type names them
  (`collect_generic_class_on_demand`). Test: `tests/run-pass/a_generic_struct_may_name_a_generic_class_declared_after_it.em`.
- **An `SoA` as a class's field** did not compile (its C helpers were written after the class's
  drop glue that calls them) and, once it did, a column of it panicked with an exclusivity
  violation (the column's mutable borrow began before its `SoA`'s length was read). The `SoA` and
  `std.simd` helpers are now written before the closure values and drop glue, and a column of an
  `SoA` reached through a class reads the length first. Test: `tests/run-pass/an_soa_in_a_class_field.em`.
- **A tuple could not be a generic extension's target** (`extend[A: P, B: P] (A, B) implements Q:`
  did not parse, and was not matched): the parser takes it where the line ends with its `:` (a call
  `extend[i](x)` stays a call), and the type checker keeps it under `$tupleN` with the elements as
  its arguments, so every tuple of that arity whose elements meet the bounds takes it, as a generic
  type's instance does (only with `implements`, `[IFC-2]`). Test:
  `tests/run-pass/a_tuple_of_parameters_implements_an_interface.em`. This is what a `Query[(A,
  Mut[B])]` needs to map each parameter to its item type (`[ECS-3]`).

## 5. `[STD-1]`: `std.core`, `std.mem`, `std.math`, `std.simd`, `std.arena` are `@noalloc`

**Built.** The type checker gives every function of those five modules (and `std.math`'s
submodules) an implied `@noalloc` (`std_contracts`) unless its documentation says it
`allocates`; the effects pass then checks it as a written contract, `E4001` naming the chain to
the allocation. So a function of these modules that allocates with nothing saying so stops every
build, which is the strongest form of the check (the dependency map asked for "something that
checks"). A generic function, a generic type's or generic extension's method and an interface's
provided method are exempt where written (`std_generic_members`): their effects are each
instance's (`[EFF-2]`), which no declaration can promise. `std/src/arena.em` is new (the arenas
are prelude, compiler-known types; the module says what they are and holds the rule).

**What the check found, and what was done.** 26 functions of `std.core` and 12 of `std.math`
allocated undocumented. `std.math`'s `Mat2/3/4.col` and `row` panicked with an f-string
(`[PAN-2]`: that allocates), and `transpose`, `inverse`, `determinant`, `try_inverse`,
`from_view_projection` reach them: their panics are now plain text ("a Mat3 has columns 0 to 2
only"), so the whole of the maths is `@noalloc`, which game code wants. In `std.core`, the error
report and `AnyError`'s `fmt`, `fmt_debug` and `source` (which write text, or call through
`dyn Error`), `String`'s `default`, and the 128/256-bit count helpers (`of_wide`, `to_wide`,
`to_int`, …, whose panics name the value in an f-string) now say they allocate; a derived
`Display`/`Debug`'s `fmt` says so too (`errors.rs`'s `member`). *Choice made:* the count helpers
keep their informative panic messages (documented as allocating) rather than losing the number.

**Tests.** `STD-1/accept_core_math_and_simd_functions_are_noalloc.em` (a `@noalloc` function using
vector and matrix maths, `transpose().inverse()`, and an `i32x4`), and the milestone
`std_functions_are_noalloc_unless_documented` (a copy of `std` whose `core` gains an undocumented
allocating function is `E4001`; documented, generic, or in `std.thread`, it is accepted).
`STD-1` leaves the `unfinished` list.

## 6. `[CG-C-4]`: views proved apart get a `restrict` pointer and a hoisted length per loop

**Built.** Before, `restrict` reached the C only through ADR-079/098's kernels (a loop moved into a
function of its own whose views are `restrict` parameters), which each C compiler gets on its own
conditions, so most loops over views proved apart had none. Now every counted loop written as a C
`for` (`emit_for_loop`) asks `ember_analysis::restrict_views` (kernels.rs, beside `[SIMD-3]`'s
`origin`/`apart`) which of its lists and views qualify: indexed in the loop, named there only by
element and length (never whole, so a view reassigned in the loop is out; never borrowed), the
loop calling nothing (a call could grow a list or write through a class handle: `[SIMD-3]`'s
handle condition, kept conservatively for every view), the loop reaching no memory any other way
(no other reference followed, no other local indexed), and `[SIMD-3]` proving it apart from every
other list or view the loop names. Each is written in a C block around the loop as
`T* restrict _N_r = (T*)<view>.ptr; size_t _N_rn = <view>.len;`, and the loop's element accesses
and length reads (its bounds checks) go through them (`place_in`). A kernel's views are already
its `restrict` parameters. *Limits:* a loop that is not a C `for` (a body with branches, or with
a call) gets none; views reached through a class handle never get one (their origin is not
derived). Both are on the safe side of the rule ("exactly when … proves").

**Speed.** A/B on mains power, performance cores, the 13 benchmark programs whose C changed
(a11, b13-b16, p1, w13, w1; MSVC and clang): every row within noise of before (0.97-1.04x C both
ways); no regressions. The compilers mostly ignore a block-scope `restrict` (ADR-079/098 measured
that MSVC and gcc take it only from parameters), so the kernels stay the fast path.

**Tests.** `CG-C-4/accept_views_proved_apart_get_restrict_pointers_and_hoisted_lengths.em` (two
owned lists, `[SIMD-3]` (b); two `SoA` columns, (c)) and
`CG-C-4/accept_views_not_proved_apart_or_reassigned_keep_plain_pointers.em` (two parameter views;
a view set in its loop): `assert-c` on the emitted C, checked by the conformance milestone.

## 7. `[BLD-2]`, `[CG-C-3b]`: effect sets, exported inline bodies and layouts in the interface hash

**Built.** The module-interface record (`ember_build::interface`, schema 11) now carries, beside
each callable's signature and region contract, its effect set (by effect name, from
`ember_analysis::effect_sets`; `None` for a generic, whose effects are each instance's) and, for a
function the inline header exports (`ember_codegen_c::inline_header_symbols`, the `[CG-C-3]` set),
the hash of its MIR body; and, per module, the layout of each import-visible struct, class and
enum as declared (its placing attributes, its fields with their types and defaults, or its
variants). All of it is in the interface hash, so an importer's cache key changes exactly when one
of those facts does. *Choice made:* the layout is read from the declaration, not from the type
checker's table: a first version used the table's sizes and offsets, and `tools/error_pages.py`
caught that a type the checker makes only when a program names it (`std.thread.Scope`) then
appears in one program's record of `std.thread` and not another's, which the cache reports as a
stale summary. The declaration is the same in every program. *Limits:* the compiler still checks
the whole graph each run (the record is the dependency identity, as before; reuse of a cached
front-end result is not built); a size change that comes only from another module's type shows
through that module's own record, which the importer's key also holds. `BLD-2` leaves the `unfinished` list.

**Test.** Milestone `effects_inline_bodies_and_layouts_are_in_the_interface_hash`: a private
constant edit and a private struct edit change nothing; a private body gaining `Io` changes the
public caller's effects and the hash; an exported inline body edit changes the hash with the same
signature and effects; a field added to a public struct changes the layouts and the hash; each
change also changes the importer's cache key.

## 8. `std.ecs`: `Entity`, sparse-set storage, `@derive(Component)` and the registry (`[ECS-1]`, `[ECS-2]`, `[ECS-7]`)

**Built.** `std/src/ecs.em` (new; listed in `std/src/lib.em`):
- `Entity` (`[ECS-1]`): a 32-bit handle, `index()` the low 20 bits, `generation()` the high 12,
  `Entity.NULL` all ones, `Entity.new`. `World.create/destroy/is_alive/len`: a generation per
  index and a free list; a destroyed index comes back at its next generation (mod 2^12).
- Storage (`[ECS-2]`): one sparse set per component type in `World.storages: Map[u64, AnyStorage]`,
  keyed by `type_hash[T]()`. `AnyStorage` holds the entities in dense order and the sparse index
  (slot + 1 by entity index); `AosStorage[T]` (the default) adds an `Array[T]`, `SoaStorage[T]`
  (`@component(layout=soa)`) an `SoA[T]`. Removal swaps the last entity into the hole.
  `World.add/remove/has/storage/entities_with`.
- `@derive(Component)` (`[ECS-7]`, `derives.rs`): written out, as `Ord` and `Default` are, as
  `extend T implements std.ecs.Component` with `component_name()` (the fully qualified name),
  `component_hash()` (its 64-bit FNV-1a, computed when the derive is written) and
  `component_soa()`. `@component(layout=soa|aos)` is read and taken out there (`E0104` for any
  other layout, or `soa` on an enum, or `@component` with no `Component` derive). The registry:
  the derive collects every component of the program, in declaration order, and writes
  `std.ecs.components()`'s body as the array of `ComponentInfo(name, hash, soa)`.
  `type_hash[T]()` is `T.component_hash()`. *Choices made:* the identity is computed by the derive
  (ordinary Ember constants) rather than by a compiler-known `type_hash`, so the library needs no
  compiler support of its own beyond the derive (Part XII.3 says it has none); a generic component
  is `E0900` (its identity would be per instance); `components()` returns an `Array` (it
  allocates): a `static` cannot hold the `str` names (`E3063`).

**Tests.** `ECS-1/accept_an_entity_is_a_20_bit_index_and_a_12_bit_generation.em`,
`ECS-2/accept_each_component_type_is_a_sparse_set_keyed_by_its_type_hash.em`,
`ECS-2/reject_a_component_layout_that_is_not_soa_or_aos.em`,
`ECS-7/accept_derive_component_registers_the_type.em`. `CLI-19`'s unimplemented-construct test
now names `@derive(Serialize)` (Component is built).

**Not built: `[ECS-3]`, `[ECS-4]`, `[ECS-5]`, `[ECS-6]`.** A `Query` was written and probed (the
`QueryParam` interface with `Item`, `ReadItem` and `State` associated types; `Mut[T]`, `Not[T]`,
`Option[T]` and tuples of 2 and 3 as parameters; `iter`/`iter_mut` iterators driving from the
narrowest required storage; `World.run(system, args)` applying a `Commands` buffer after the
system returns). It type-checks generically, but a concrete query fails in the type checker:
`QueryIterMut[(Mut[Pos], Vel, Not[Frozen])]` and `fn(mut Query[…], f32)` are each made as two
instances that differ only in their hidden projection parameter (`Q.State`, D-407's mechanism),
"expected `X`, found `X`". The fix belongs to how hidden parameters are filled for a tuple whose
associated types come from a generic tuple extension; it is a type-checker job, not this sheet's,
and was taken out of `ecs.em` rather than left half-working. Also needed: a plain component as a
query parameter needs a `QueryParam` implementation, which a blanket `extend[T: Component] T`
cannot give (`E2041` against `Option[T]`'s: no negative reasoning), so the derive would write one
too. `[ECS-4]`'s compile-time read/write sets and `E7020` need compile-time evaluation over the
query's type (or compiler support, which Part XII.3 rules out), and `run(system, args…)` is
variadic. `[ECS-6]` is about query iteration; storage order (insertion, swap-remove holes) is
built and tested through `entities_with`, but the rule is not claimed without queries.

## 9. Defects found on the way to the ECS (not numbered; the coordinator numbers them)

Fixed:
1. **An `override` in a generic class instance made early was refused (`E2110`).** A generic
   extension's associated type naming `Store[T]` (`Store[T](AnyStore)`) made the instance while
   extensions are collected, before any class's own methods are recorded, so the base had no
   virtual methods yet. Now such an instance's overrides are checked once every class's methods
   are recorded (`deferred_override_checks`). Test:
   `tests/run-pass/an_extension_may_name_a_generic_subclass_before_its_base_is_collected.em`
   (fails `E2110` before).
2. **A generic struct with a projection field, named before its declaration, was two types.**
   `Maker[Q]`'s `-> Holder[Q]` before `struct Holder[Q: P]: state: Q.State`: the signature's
   instance had no hidden parameter, the body's had one ("expected `Holder[Q]`, found
   `Holder[Q]`"). `induced_projections` now collects such a struct first. Test:
   `tests/run-pass/a_generic_struct_with_a_projection_field_may_be_named_before_it_is_declared.em`.
3. **A generic class instantiated from another module resolved its base there.** `AosStorage[T]
   (AnyStorage)` named in a user signature: `AnyStorage` was looked up in the user's module
   (`E1010`). The base is now resolved in the declaring module. Covered by
   `ECS-2/accept_each_component_type_is_a_sparse_set_keyed_by_its_type_hash.em` (`count`).
4. **A standard library override reached only through its base's slot was pruned.** A virtual call
   (`storage.remove_entity(e)` on `AnyStorage`) kept no `std` body of that slot, and the backend
   stopped ("virtual call has a collected vtable signature"). `retain_referenced_standard_bodies`
   now keeps every standard body in a called slot. Covered by the ECS-2 test (`remove`, `destroy`).

Found, not fixed (owner/coordinator decision):
5. **A `mut self` method calling a shared method on `self` panics.** `fn add(mut self): if not
   self.alive(i): …` panics "exclusivity violation: read access to W.gens while a write access to
   W.gens is active": the caller's whole-object write (`[EXC-15]`) covers the `mut self` method's
   own accesses, but the shared callee checks its reads against it. `[EXC-5]` ("accesses nested
   inside the same `mut self` method are reborrows") says this must not panic. A caller-side
   downgrade (end the write, begin a read around the call, and back) was tried and is refused by
   the MIR verifier, which requires each body to balance its own accesses; the fix needs a runtime
   downgrade operation or a covered variant of shared methods, which is EXC work. `std.ecs` avoids
   the pattern (its `mut self` methods read through free functions over the fields).
   Probe: a class with `fn alive(self, i)` reading a field and `fn add(mut self, i)` calling it.

## Owner questions

1. `Entity` is a standalone 32-bit struct; `[ECS-1]` says it is `Handle` with `Split20`
   (`[HND-2]`), which is not built. Should `Entity` become that once handles exist?
2. A query over a component stored as an `SoA` (`@component(layout=soa)`): an element is not one
   place, so `ref T`/`ref mut T` items cannot point at it. What should such a query yield
   (per-column views, a proxy, or copies)?
3. `world.run(system, args…)`: Ember has no variadic parameters. One `args` value (a tuple for
   several), or something else?
4. `ArenaSoA` was put in `std.collections` beside `ArenaArray`/`ArenaMap` (section 3): confirm.
5. Defect 5 above (shared call on `self` inside `mut self`): which fix (runtime downgrade, or
   shared methods compiled in a covered variant)?
6. `[BEN-6]` (thresholds) and `[TST-6]` (Appendix A fixture compiled) were not attempted: BEN-6's
   timings cannot be a test (benchmarks run on the owner's machine, mains power), and its
   `@noalloc` allocation counts and FFI instruction comparison need tooling that does not exist;
   TST-6 needs fixture items (CommandList, Formatter, Mutex, overlays) other jobs own, and the
   fixture's `#! language "0.5"` is `E0006`. Should they go to another job?
