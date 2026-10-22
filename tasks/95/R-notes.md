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
    `ember_typeck::ExportIndex` trait (+ `Exported`, `NoExports`), a new `exports` argument to
    `ember_typeck::check`, and `add_import_suggestions`: up to three `from M import name` fix-its,
    inserted after the module's last import (or at the top of the file), of the right kind for the
    position. Used for unknown values, unknown callees and unknown types.
  - `compiler/ember_driver/src/main.rs` — `Exports`, the index: every `pub` item of std's modules and
    every `pub`/`pub(package)` item of the root package (its tree when it has a manifest, else the
    files beside the root file), parsed (never type checked) lazily — only the first time a name is
    not found, so a program that compiles never builds it. Passed at all three `check` calls.
- Ceiling: dependency packages' modules are not in the index (their import paths are rewritten per
  package); add them when a dependency's names need offering.
