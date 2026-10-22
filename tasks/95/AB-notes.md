# Helper AB — C and C++ interop rules not yet done (notes)

Helper AB, 2026-10-22. Branch `helper-AB`, from `main` at `f00f2d61`. Target 0.9.10_Hardened_33.

## AB1. `[FFI-11]`, `[FFI-11b]` — `inout_count` beside `out` or a status result

**Done.** The count an `inout_count` gets back is now an output like an `out` parameter, in its C
position: the call returns C's result first (as its contract makes it), then the outputs in C's
order; with `-> status`, `Result[T, N]`, `T` the outputs, the count among them (`[FFI-11b]`'s last
sentence, `[FFI-16]`). ODR-125's form (`void` → the count, a result → `(result, count)`) is the
special case with no `out`, so the separate tuple path is gone: typeck makes one list of outputs
(`out_tys`, C order) and codegen one list of output locals (`_ffi_count` among the `_ffi_out<k>`).

A C result with a contract of its own (`returns_owned`, a counted or fixed view, a reference, a
`cstr`) beside outputs is now returned as that contract makes it: `emit_ffi_result_value`
(codegen) gives C's result as its Ember value in one place, and the output tuple is built around
it (`ffi_tuple_of`); the status path is `emit_ffi_status`. `check_inout_counts`'s `E0900` for an
`inout_count` beside a contracted result is gone.

Tests: `tests/conformance/FFI-11b/accept_inout_count_beside_out_and_status.em` (an `out` and the
count, a status with the count, a `cstr` result beside an `out` and the count),
`tests/conformance/FFI-11/accept_out_parameters_beside_a_contracted_result.em` (with `tokens.c`,
`tokens.h`, `overlays/tokens.em`: `returns_owned` and `returns_owned … nullable` beside an `out`,
through an overlay).

**Defect found and fixed (for the lead to number):** an `out` parameter beside a `returns_owned`
result crashed the program (access violation): the wrapper returned C's raw pointer in the tuple's
`ForeignBox` slot, never adopting it (`emit_ffi_outputs` ignored `result_owned`, and typeck's
metadata read the result contract from the tuple). Probe: the FFI-11 test above with `main`'s
compiler.

### Compiler changes for AB1

- `compiler/ember_typeck/src/lib.rs` — signature collection: one output list (`out` parameters and
  the `inout_count`'s count, C order); `check_foreign_signature` reads C's result from
  `foreign_c_results` only; `foreign_counted_metadata` reads the result contracts against C's
  result type; `check_inout_counts` loses its `E0900` and its `attrs` parameter.
- `compiler/ember_codegen_c/src/lib.rs` — `ffi_counted_signature` (C's result type from the
  outputs), `emit_ffi_status`, `ffi_tuple_of`, `emit_ffi_result_value`; the wrapper's end is one
  path.
- `compiler/ember_mir/src/lib.rs` — re-exports `FfiCounted`.
