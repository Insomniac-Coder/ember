# Features not implemented because they require another feature

Each entry is a spec feature the compiler does not have yet because another
feature it is built on does not exist yet. None of these is a deviation: the
compiler does not do something *different* from the rule, it does not do the
rule yet, and it says so (an error, or nothing observable). When the feature it
waits on lands, the entry is built and removed.

Moved here from `docs/DEVIATIONS.md` on 2026-09-27 (owner: they are not
deviations). Their old numbers are kept in brackets so earlier records can be
followed.

---

## N1 — generated operator implementations for range types (formerly D1)

| | |
|---|---|
| **Rule** | `[RNG-5a1]` |
| **The feature** | for every range type `T` over `R`, the compiler generates `T: Add[T, Output = R]`, `T: Add[R, Output = R]`, `R: Add[T, Output = R]` and the `Sub`/`Mul`/`Div`/`Rem`/`Neg`/`PartialEq`/`PartialOrd` forms, in `T`'s declaring module |
| **Requires** | the operator interfaces (`Add`, `Sub`, …) in `std`, which do not exist yet |
| **Meanwhile** | operators on range types are decided in the type checker, with the results the rule requires: `r + 0.25` is `f32`, `Roughness + Metallic` is `E2214` (`tests/conformance/RNG-5a1/`) |
| **Can a program notice** | yes, since the operator interfaces exist (ODR-040): `fn twice[T: Add[T]](x: T)` refuses a `Unit` with `E2040` (probed 2026-10-01) |
| **Plan** | build it together with the operator interfaces, before user generics can be bounded on them (Phase 3, block D). ADR-016 |

## N2 — `extern class` (formerly D3)

| | |
|---|---|
| **Rule** | `[FFI-39]`, and the production added as amendment A9 |
| **The feature** | a declared foreign C++ base class: sized, known layout, implicitly `open`, inheritable under `@ffi(trampoline, virtuals=[…])` |
| **Requires** | the C++ importer, its thunks and the trampoline (Phase 7), none of which exists |
| **Meanwhile** | it parses, then is refused with `E1010` "not supported yet in this phase". Refused rather than ignored: a declaration that is stored and then ignored would build a program that is wrong |
| **Can a program notice** | yes, as a rejection |
| **Plan** | Phase 7, with the importer. ERR-037 |

## N3 — `once fn` callbacks of the `Option`/`Result` helpers (formerly D6)

| | |
|---|---|
| **Rule** | `[ERR-4]` (ODR-025): `map`, `map_err`, `and_then`, `or_else`, `unwrap_or_else`, `ok_or_else` and `filter` take `f: once fn(…)` |
| **The feature** | a `once fn` is called at most once, so the closure passed may give away a capture; a lambda written at the call site keeps the captures it infers |
| **Requires** | `once fn` as a parameter type: a bound that consumes the callee value when called, without `owned`'s storage rule, carried to MIR so `check_owned_closure_argument_regions` does not apply |
| **Meanwhile** | the `std.core` helpers take `f: fn(…)`, `[CLO-3]`'s bound: a closure that moves a capture out is `E3030`, with a help naming the method. The compiler refuses more, never less |
| **Can a program notice** | yes: `name.map(owned fn(s) => join(s, tail))` is `E3030` |
| **Plan** | build `once fn` parameter types, then change the helpers' `fn(` to `once fn(`. Target M1. ODR-025 |

## N4 — a call inside a loop in vectorisable form

| | |
|---|---|
| **Rule** | `[SIMD-5]`: "every call in the body is to a function `[CG-C-3]` places in the inline header, read through as `[PAR-2b]` says"; `[SIMD-7]` then groups the loop's overflow checks, the callee's included |
| **The feature** | a loop that calls a small function (`@inline`, an operator of a type of at most 64 bytes, a view or container accessor, or any function of at most 40 statements with no foreign call) can still be in vectorisable form, the call read through as if inlined |
| **Requires** | `[CG-C-3]`'s per-package inline header, which is not built, and the read-through (a MIR inliner) |
| **Meanwhile** | a loop with any call is not in vectorisable form: its overflow checks stay one per operation. `[OPT-2]` still versions it; its bounds checks are removed when the callee is proved not to change the view |
| **Can a program notice** | no, except as speed |
| **Plan** | build `[CG-C-3]`'s inline header and the read-through, then admit those calls in `loop_version.rs`'s `vectorisable` |

## N5 — a floating-point running total in a loop in vectorisable form

| | |
|---|---|
| **Rule** | `[SIMD-5]`: "every floating-point reduction is declared by `@parallel(reduce=…)` or the function is `@fastmath`" |
| **The feature** | a loop keeping a floating-point running total is in vectorisable form when it declares the reduction |
| **Requires** | `@parallel(reduce=…)`, which is not built (`[ATT-6]` rejects it with `E0900`). `@fastmath` is built (ADR-085) and admits the loop |
| **Meanwhile** | outside `@fastmath` such a loop is not in vectorisable form; it has no integer overflow check to group unless it also does integer arithmetic, which stays checked one operation at a time |
| **Can a program notice** | no, except as speed |
| **Plan** | admit the declared reduction once `@parallel` is built |

## N6 — `@fastmath` and `@fp(contract)` in a static library

| | |
|---|---|
| **Rule** | `[CG-C-11]`: "A `@fastmath` or `@fp(contract)` function is emitted in a separate translation unit compiled with the relaxed flags" |
| **The feature** | a static library whose package has a relaxed function |
| **Requires** | library builds of more than one C unit: a library keeps its functions internal to its one unit (`static`), so two Ember libraries linked into one host cannot clash, and a relaxed unit could not call them. Package-private external names are the missing piece; the owner deferred further FFI work (`docs/AUTOPILOT.md` §2) |
| **Meanwhile** | such a library is `E0900` (`staticlib.rs`'s `a_static_library_with_a_relaxed_function_is_refused`); the attribute is never dropped. Programs build as `[CG-C-11]` says |
| **Can a program notice** | yes: the library does not build |
| **Plan** | name a library's functions with its package prefix (as its public types are, `package_type_namespace`) and give the ones a relaxed unit shares external linkage; then build the relaxed objects into the archive |

## N7 — a default doing float work that its mode's closure cannot hold

| | |
|---|---|
| **Rule** | `[TYP-9]` (ODR-090): "a parameter default keeps its declaring function's mode ... wherever ... evaluated" |
| **The feature** | a default of another float mode than the calling function's that does float work and cannot be the body of a closure: a `mut` parameter's default (a place, which a closure cannot hand back), or one using `self` or an earlier parameter in a way a borrowed closure parameter cannot (a `mut self` method) |
| **Requires** | evaluating a place, or a mutating default, out of line in another C unit: a function of the declaration's mode taking the earlier parameters in their own modes |
| **Meanwhile** | `E0900` at the call, naming the parameter and the mode (`TYP-9/reject_a_mut_default_doing_float_work_from_another_mode`); a default doing no float work, or called from its own mode, is not affected |
| **Can a program notice** | yes: such a call does not compile |
| **Plan** | give each such default a function of its declaration's mode whose parameters take the earlier parameters' modes |
