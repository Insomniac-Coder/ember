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
