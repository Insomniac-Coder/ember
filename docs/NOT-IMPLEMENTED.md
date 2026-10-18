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
