# Helper T — C++ interop: D-808 and `[FFI-17a]`'s bridges (notes)

Helper T, 2026-10-22. Branch `worktree-agent-ac63b86829223e483`, fast-forwarded to `main` at
`7b4a5854` (the brief's corpus and D-808 are there; the worktree had been made at `b8c4df20`).
Target 0.9.10_Hardened_33.

## T1. D-808 — an overlay's contract words on a C++ function reached through a wrapper

**Done.** An entry that states every fact (no `TODO(…)`, no `unsafe`) now contracts the thunk's own
declaration, as a C function's is: `render_contracted` (cimport.rs) writes it `safe fn` with
`@ffi(param(…), result(…))`, so the checker's existing lowering does the work (a `count(n)` witness
and an `out` leave the callable signature; a `Span` gives the pointer and length, with its length
checks). What the importer passes itself is a `borrowed one` reference of the same declaration:

| slot | declared as | the wrapper passes |
|---|---|---|
| receiver | `ember_self: ref T` / `ref mut T` (`from(self)` in a result word becomes `from(ember_self)`) | `ref self` / `ref mut self`; a trampoline's `ember_cpp_ref(self.ember_cpp_base.object)` |
| opaque `const T&` / `T&` | `p: ref T` / `ref mut T` | `p` |
| opaque `T&&` | `p: ref mut T` | `p.get_mut()` (the `ForeignBox` frees what is left, as before) |
| C++ `T&` of a type by value | `mut p: T` with `param(p, borrowed, one, exclusive)` | `p` |
| bridge value (`std::vector&` …) | `p: ref CppObject` / `ref mut CppObject` | `ember_cpp_ref[_mut](p.as_ptr() as *CppObject)` |
| trampoline object, `Retained` token | `p: ref T` / `ref mut T` of the opaque C++ class | `ember_cpp_ref[_mut](….ember_cpp_base.object)` |
| class result by value | `mut ember_out: T` with `param(ember_out, out, one)` | (returned by the call) |
| where the thunk says it caught | `mut ember_failed: c_int`, `borrowed one exclusive` | the wrapper's local |

The wrapper's parameters are then the safe types (`safe_parameter`), and it needs `unsafe` only to
name an object by its address (a bridge value's, a trampoline's) or to own a new object the thunk
gives (`adopt`, `from_thunks`). Outputs follow the checker's rule (C++ result first, then each `out`
or the `inout_count`'s count; a `status` result is `Result[outs, N]`); a class by value, which the
thunk writes last, is put first. An entry's own `@ffi(…)` arguments (`effects=[…]`) now reach the
declaration too (they were dropped for wrapped functions). A `class C:` method line is compared with
the contracted wrapper's signature when an entry contracts the method (class sections are now applied
after the overlay's other items).

An entry that leaves a pointer without facts is now `E5002` on a wrapped C++ function, as on a C
function and a `noexcept` free one (before, its wrapper silently stayed `unsafe`).

Tests: `tests/conformance/FFI-17/accept_contract_words_reach_a_wrapped_function.em` (run-pass, no
`unsafe`: throwing functions and methods with `count(n)` spans, a `MutSpan`, a `cstr`, a `from(self)`
reference result, a `T&` beside a span, a bridge value lent mutably beside a span, a class by value,
an opaque class by value and a `std::string` as results, two spans sharing a length, a constructor,
and a `class` method line restating a contracted method), `run_fail_two_counted_views_of_unequal_lengths.em`
(the length check before C++ runs), `reject_a_count_naming_no_length.em` (`E5002`); `texts.hpp`,
`overlays/texts.em`, `overlays/texts_unmatched.em`. Break-tested: the method line written with the
raw signature is `E5010`, naming the contracted one.

`tests/cxx-corpus/program/main.em` now passes its text through `json_api.overlay.em` (`parse`,
`count`, and nlohmann's `is_object`, `size`, `dump`): the program has no `unsafe:` left.
`the_cpp_corpus_binds_as_recorded` copies the overlay into its package (package.rs). D-808's words
are out of `[FFI-11]`'s `unfinished` entry.

### Compiler changes for T1

- `compiler/ember_typeck/src/lib.rs` — `foreign_referent`, `foreign_mut_one`: a `one` contract's
  reference may refer to an incomplete type (`extern type`), and `borrowed one exclusive` is
  `ref mut T` (the `[FFI-11]` table's own form) as well as `mut p: T`. **This fixes a C gap too:**
  a C header's opaque handle (`typedef struct Ctx Ctx; int ctx_get(const Ctx*)`) contracted
  `borrowed one` was `E5015`/`E5002` (`ref Ctx` "refers to a type with no foreign
  representation", `mut ctx: Ctx` "incomplete"). Test: `tests/conformance/FFI-11/accept_an_opaque_handle_is_borrowed_as_one.em`
  (with `handles.c`, `handles.h`, `overlays/handles.em`): `ref`, `ref mut`, nullable of each, a
  result `from(counter)`, no `unsafe`.
- `compiler/ember_driver/src/cimport.rs`:
  - `CType.opaque` (a data pointer to an incomplete type), set in `c_type`; `safe_parameter` gives
    such a pointer's exclusive `one` as `ref mut T` (an incomplete type takes no parameter mode).
    `.embind` version 10 (the field is recorded).
  - `canonical`, `slot_words`: an overlay's words are written into `param(…)`/`result(…)` in the
    order the checker reads them (ownership, count, the rest). **Defect found:** an overlay
    writing `nullable borrowed one` (any order but ownership-count-rest) gave `E0900` "this
    `param(…)` contract is not implemented yet": the checker's parsers are positional
    (`ffi_param_contract_arg` and its neighbours), and the importer copied the overlay's order. The
    importer now canonicalises; hand declarations are still positional (see Defects).
  - `slot_words` also derives `borrowed one exclusive` for a C++ `T&` of a type by value (`mut p:
    T`) when the entry writes nothing: a `noexcept` free C++ function with an `int&` and an entry
    was `E5002` before ("safe fn needs an @ffi contract for mutable parameter").
  - `ADDRESS` (`std::uintptr_t`): a thunk gives the address of an object the caller comes to own
    (an opaque class by value or from its constructor, a bridge value's object, a released
    `std::unique_ptr`, an iteration's range) as an integer, `usize` to Ember, which the wrapper
    turns into the owner (`adopt[T](… as *mut T)`, `CppVector.from_thunks(… as *mut u8, …)`). Why:
    a contracted (`safe`) declaration may not return a raw pointer (`[FFI-10]`), and the only safe
    pointer results are `ref`/`Span`/`cstr`/`ForeignBox` — a `ForeignBox` for a bridge value would
    add the runtime's adoption record (`ember_ffi_adopt`) to every `std::string` result, which a
    C++ caller does not pay; a `ref … from(static)` would be a false declaration. An integer is the
    honest type ("a scalar can be an address", `[FFI-2]`) and costs nothing (one register, a cast
    the C compiler drops). The thunk declares the same type, so C and C++ agree under LTO
    (`-Wlto-type-mismatch` would flag a pointer/integer pair). The thunk prelude includes
    `<cstdint>`.
  - `render_contracted` (new), `contracts_everything`, `signature_of`; `render_wrapper` dispatches
    to it; `render` finds a status enum's error type for it; `method_value` gives `ForeignBox[T]`
    for an opaque class by value.
- `std/src/ffi.em` — `CppObject`, an `extern type` in `std.ffi`: the C++ object a bridge value
  owns, as a contracted declaration takes it (only behind a reference).
- `tests/cxx-corpus/nlohmann_json/*.thunks.cpp` — re-recorded for the `ADDRESS` change (6 thunks and
  the include). The Windows record was written by `record.py`; the Linux one (no libclang here) by
  the same textual rewrite, which reproduces the Windows record byte for byte from its old text; the
  two diffs are identical. **The lead should re-record on Linux** (`python tests/cxx-corpus/record.py
  <linux ember>`) if CI's Linux run disagrees.

## T2. `[FFI-17a]`'s bridges

(below)

## Defects found (for the lead to number)

1. **Hand declarations read contract words by position.** `@ffi(param(p, nullable, borrowed,
   one))` is `E0900` ("this `param(…)` contract is not implemented yet"); only `param(p, borrowed,
   one, nullable)` is read. `[FFI-10]`/`[FFI-11]` give no order (an overlay's words are "separated
   by spaces", any order). The importer now writes overlays' words in the checker's order, so
   imports are fixed; a hand declaration is not. Fix: read each `param(…)`/`result(…)` as a set of
   words (`ffi_param_contract_arg`, `ffi_result_contract_arg`, `ffi_owned_param_arg`,
   `ffi_retained_param_arg`, `ffi_owned_result_arg`, `ffi_missing_count_arg`,
   `ffi_result_missing_count_arg` in ember_typeck). Open.
2. **An opaque handle could not be borrowed `one`** (C and C++): fixed (T1, typeck).
3. **A `noexcept` free C++ function with a `T&` of a type by value and an entry was `E5002`**:
   fixed (T1, `slot_words`).
4. **An entry's `@ffi(effects=[…])` was dropped for a C++ function reached through a wrapper**:
   fixed for contracted entries (T1); an entry with `TODO`/`unsafe` words still drops it (its
   raw declaration is written without the entry, as before).
5. **Names the importer gives its own slots could collide with a C++ parameter's** (`this`, `out`,
   `failed`, and the wrapper's locals `failed`, `slot`, `result`): a throwing C++ function with a
   parameter named `failed` or `result` declared it twice. Fixed: `ember_self`, `ember_out`,
   `ember_failed`, `ember_slot`, `ember_result` in both the contracted and the uncontracted
   declaration and wrapper (`fill(int* out, …)` in `texts.hpp` takes a parameter named `out`).

## Spec questions

None for T1.
