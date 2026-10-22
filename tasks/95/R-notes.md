# Helper R — notes (2026-10-22)

Rules: DIA-15, CG-C-9, PHIL-1, CG-C-7, VER-3, CG-C-6, CG-C-10, DIA-5, LT-39, SEL-2, ALC-2, OPT-1.

## SEL-2 — done
- Tests: `tests/conformance/SEL-2/reject_shared_and_cpp_shared_never_convert.em` (argument, return,
  annotated local, `as` cast; `Shared` and `Weak` to `CppShared` and back),
  `tests/conformance/SEL-2/accept_shared_and_cpp_shared_each_keep_their_own_count.em`.
- Compiler: `compiler/ember_typeck/src/lib.rs` — `counted_family` / `counted_family_mismatch`;
  `coerce` and the `as` cast now report `E5065` (was a plain `E2020`) when one side is Ember's
  `Shared`/`SyncShared`/`Weak` and the other C++'s `std.ffi.CppShared`/`std.ffi.CppWeak`.
- `docs/errors/E5065.md` written (the code had no page; `ember test --doc` passes both examples).
- Gap outside SEL-2: `std.ffi` has no `CppWeak` (the `[FFI-17a]` table maps `std::weak_ptr<T>` to
  it). The check already names `std.ffi.CppWeak`, so it covers it once it exists.

## DIA-15 — done
- Tests: `tests/conformance/DIA-15/reject_suggestions_come_from_scopes_types_and_exports.em`
  (scope table: local; type information: field and method, only the receiver's own methods; index
  of exported names: a value and a type), `reject_an_import_line_is_of_the_right_kind.em` (a
  function is not offered where a type is expected; a type is where a value is),
  `accept_offered_import_line_compiles.em`.
- Compiler:
  - `compiler/ember_typeck/src/lib.rs` — `method_name_suggestions`: shape N3's "nearest member" was
    missing (`p.length_squard()` offered nothing); now ranked from the method table as `[DIA-24]`
    ranks names, receiver and its bases only, invisible methods left out.
  - Shape N2 ("name exported elsewhere: the import line, as a fix-it") was not built. New
    `ember_typeck::ExportIndex` trait (+ `Exported`), a new `exports` argument to
    `ember_typeck::check`, and `add_import_suggestions`: up to three `from M import name` fix-its,
    inserted after the module's last import (or at the top of the file), of the right kind for the
    position. Used for unknown values, unknown callees and unknown types.
  - `compiler/ember_driver/src/main.rs` — `Exports`, the index: every `pub` item of std's modules and
    every `pub`/`pub(package)` item of the root package (its tree when it has a manifest, else the
    files beside the root file), parsed (never type checked) lazily — only the first time a name is
    not found, so a program that compiles never builds it. Passed at all three `check` calls.
- Ceiling: dependency packages' modules are not in the index (their import paths are rewritten per
  package); add them when a dependency's names need offering.

## LT-39 — done (with a spec question)
- Tests: `tests/conformance/LT-39/reject_a_field_selected_at_run_time_needs_every_region.em`,
  `tests/conformance/LT-39/accept_regions_a_selection_cannot_reach_may_end.em`.
- No compiler change: a run-time choice between fields (a conditional expression, or a function
  choosing by an argument) already holds every region it could select, and only those, and they
  end at the selection's last use.
- The rule's examples, reflection and field descriptors, have no run-time operation that selects a
  field in safe Ember: `TypeInfo`'s `FieldDesc` (`[RFL-2]`) gives an offset, which reaches a field
  only through a raw pointer in `unsafe`, outside region tracking. So every operation the rule can
  apply to today is covered.
- Spec question: the rule says "`E3065` if a region it needs has ended", but `E3065` is defined
  (registry, `[LT-35]`, shape B14) as "multi-region result provenance cannot be inferred". Ending a
  region a live selection needs is an ordinary loan conflict, reported `E3021` (as every other
  ended-region use is, `[LT-18]`). Options: (a) change LT-39's parenthesis to name the loan
  conflict (`E3021`/`E3010` family, shape B1); (b) keep `E3065` and make the compiler report a
  run-time selection's ended region as B14. Recommendation: (a) — B14's help ("return an owned
  value, or the views as separate results") does not fit a local conditional, and the region is
  known, not uninferable.
