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

## N2 — `extern class` (formerly D3)

| | |
|---|---|
| **Rule** | `[FFI-39]`, and the production added as amendment A9 |
| **The feature** | a declared foreign C++ base class: sized, known layout, implicitly `open`, inheritable under `@ffi(trampoline, virtuals=[…])` |
| **Requires** | the C++ importer, its thunks and the trampoline (Phase 7), none of which exists |
| **Meanwhile** | it parses, then is refused with `E1010` "not supported yet in this phase". Refused rather than ignored: a declaration that is stored and then ignored would build a program that is wrong |
| **Can a program notice** | yes, as a rejection |
| **Plan** | Phase 7, with the importer. ERR-037 |

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

## N9 — `assert_disjoint`'s `RuntimeCheck(Aliasing)` record

| | |
|---|---|
| **Rule** | `[DSJ-5]`: "Both forms record a `RuntimeCheck(Aliasing)` site whose reason is `establishes_static_fact` (`[EFF-11]`), unless the compiler already knows the ranges are disjoint, in which case no comparison is emitted" |
| **The feature** | the safety side table's entry for each comparison, and leaving the comparison out where the ranges are known apart |
| **Requires** | `RuntimeCheck(k)` effects and reason codes (`[EFF-9..11a]`, Phase 4) |
| **Meanwhile** | every call compares (two comparisons per pair); nothing is recorded |
| **Can a program notice** | no: `ember inspect --safety` lists no aliasing checks for any program yet |
| **Plan** | record the site in the side table with `establishes_static_fact`; skip the comparison when range facts prove the views apart |

## N7 — a default doing float work that its mode's closure cannot hold

| | |
|---|---|
| **Rule** | `[TYP-9]` (ODR-090): "a parameter default keeps its declaring function's mode ... wherever ... evaluated" |
| **The feature** | a default of another float mode than the calling function's that does float work and cannot be the body of a closure: a `mut` parameter's default (a place, which a closure cannot hand back), or one using `self` or an earlier parameter in a way a borrowed closure parameter cannot (a `mut self` method) |
| **Requires** | evaluating a place, or a mutating default, out of line in another C unit: a function of the declaration's mode taking the earlier parameters in their own modes |
| **Meanwhile** | `E0900` at the call, naming the parameter and the mode (`TYP-9/reject_a_mut_default_doing_float_work_from_another_mode`); a default doing no float work, or called from its own mode, is not affected |
| **Can a program notice** | yes: such a call does not compile |
| **Plan** | give each such default a function of its declaration's mode whose parameters take the earlier parameters' modes |
