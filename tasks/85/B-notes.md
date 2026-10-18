# Helper B — C interop part 1 (85% push, 2026-10-18)

Job sheet: FFI contracts and `@ffi` attributes beyond link names, overlays, `ForeignBox`/`Retained`,
foreign statics (`[FFI-10]`), the trusted base (`[TCB-*]`), FFI instrumentation, `ember bind`/`ember
calls`. This file is the record of what was built, why, what is left, and the owner's questions.

## 1. Foreign statics are places (`[FFI-10]`, `[STA-1]`)

**Before.** A foreign static was read through `Builtin::ForeignStaticRead` (a value) and written
through `ForeignStaticWrite`/`ForeignStaticFieldWrite` (a dotted field path in a symbol). That shape
could not hold a record that is not `Copy`, could not write an element of an array inside a static
(D1: refused with `E0900`), and admitted only scalars, pointers and structs.

**Now.** One builtin, `Builtin::ForeignStatic { symbol, ty, immutable }`, is the C global's address
as `ref T` (`ref mut T` for a `static mut`), and every use is `Deref` of it — the shape `ConstStatic`
and `LazyStatic` already have. So a foreign static is an ordinary place: a field read or write, an
element write with its bounds check, a whole assignment, `+=` with `[EXP-2]`'s operand first (the
place computes, so the generic path holds the operand and hoists the place), and a whole read of a
record with drop glue is `E3013` (moving out of a reference). The checks that stay special are the
two the spec names: only `static mut` is written (`E2140`), and only inside `unsafe` (`E5002`).
Removed: the three old builtins, the field-path symbol and its two root finders (one finder now walks
fields and indices). The C is `((T*)&symbol)`, folded by the C compilers.

Any FFI-safe `Sync` type is a foreign static now: arrays, function pointers, records not `Copy`. A
type with no C layout (`str`, `String`, a record holding one) is `E5050` "has no foreign
representation" (was `E0900`); a record with a `cstr` field has a layout but `cstr` is not `Sync`, so
`[STA-1]`'s `E7002`. An `@ffi(immutable)` pointer global is declared `T* const sym` (the object is
const, not the pointee).

Tests: `tests/conformance/FFI-10/accept_foreign_statics_as_places.em` with the milestone
`foreign_statics_are_places_in_c_storage` (links `tests/fixtures/foreign_statics.c`, C reads back
every write), the four `reject_*` files that were `E0900`.

## 2. Non-C declaration blocks (`[FFI-10]`, `[FFI-9]`)

`unsafe extern "system" | "fastcall" | "vectorcall"` blocks are accepted. The C prototype (and a
function value's typedef) carries `EMBER_CC_SYSTEM`/`EMBER_CC_FASTCALL`/`EMBER_CC_VECTORCALL`,
defined once per generated file when used: `system` is `__stdcall` on 32-bit Windows and empty
elsewhere; `fastcall` is `__fastcall`/`__attribute__((fastcall))` on 32-bit x86 and empty on x86-64
(both compilers ignore it there: one convention); `vectorcall` is `__vectorcall` (MSVC, clang-cl) or
`__attribute__((vectorcall))` (Clang) on x86. Where the convention does not exist (GCC's vectorcall, a
non-x86 target) the macro is an undefined name that stops the C build naming the reason, never a call
with another convention. `"C++"` stays `E0900` (Annex C); any other string is `E0104`.
Tests: `FFI-10/accept_system_and_fastcall_declaration_blocks.em` (run-pass, calls `abs`/`llabs`
and a function value), `accept_vectorcall_declaration_block.em`, milestone
`vectorcall_declarations_compile_with_the_convention` (compiles a call; GCC is told it has none),
`reject_non_c_manual_abi.em`.

## 3. `@ffi(effects=[…])` and `@ffi(deterministic)` (`[EFF-3]`, `[FFI-10]`, `[DET-2]`)

`hir::Contracts::foreign` holds a foreign declaration's effects: those its `@ffi(effects=[…])`
lists, plus `FFI`; every effect when it lists none; `Nondet` unless `@ffi(deterministic)`. The
effects analysis takes them from there (before: `FFI | Nondet` for every foreign function whatever it
did, so `@noalloc` callers of an allocating C function passed; and a counted `safe fn` wrapper, whose
body has no blocks and no ABI, got no foreign effect at all). Names are Part X.1's (`Panic` is
`Panic(Explicit)`); an unknown name, a duplicate, or `Nondet` beside `deterministic` is `E0104`.
Tests: `tests/conformance/EFF-3/`.

## 4. `ember tcb` lists the foreign facts (`[FFI-10]`, `[TCB-2]`, `[TCB-3]`, part of `[TCB-1]`)

`compiler/ember_driver/src/foreign.rs`: every declaration of an `unsafe extern` block in the
program's modules, with each fact it asserts — its signature (and C symbol), `safe fn`, each `@ffi`
clause, `immutable`, `effects`, `deterministic` — its source and its grade. The report now opens with
what the language guarantees, then the facts supplied from outside it (the unsafe section, each
entry graded too, then "C FFI" by module); `--json` gains `guaranteed` and `foreign`. Every grade is
**asserted**: a hand declaration has no way to claim more, and nothing written raises one.
Test: milestone `tcb_lists_every_foreign_fact_with_its_source_and_grade`.

## 5. `ember calls --foreign <item>` (`[CLI-13]`)

`ember_analysis::effects::foreign_reach`: breadth-first over the effects' call graph (calls, virtual
slots, drops, what the runtime is handed) plus the foreign functions whose value a reached function
takes; each foreign function with the chain to it, and its declaration's facts (section 4) as its
contract, including "effects: none stated, so every effect". A call through a function or `dyn`
value is named as beyond the list (its callee is not known here). Test: milestone
`calls_foreign_lists_what_an_item_reaches_with_each_contract`.

## 6. `E5002` names each unknown fact and the line that states it (`[DIA-18]`)

A call to a foreign function not declared `safe fn` lists, as notes, each unknown fact (not asserted
safe to call; per raw pointer parameter its ownership, count and nullability, or the `TODO` axes
already written; for a pointer result also its lifetime) and, as help, the declaration line that
states them: `@ffi(param(p, TODO(ownership), TODO(count), TODO(nullable)), result(…))` over
`safe fn f`, each `TODO` to replace. The printed line is accepted on the unsafe declaration as
written (`[FFI-11c]`), which `DIA-18/accept_printed_contract_line_compiles.em` checks. A header's
functions (overlays) will print the overlay form of the same words; the importer does not exist.
Tests: `tests/conformance/DIA-18/`.

## Rules done (each with its test)

| Rule | Test |
|---|---|
| `FFI-10` | `tests/conformance/FFI-10/*`, milestones `foreign_statics_are_places_in_c_storage`, `vectorcall_declarations_compile_with_the_convention`, `tcb_lists_every_foreign_fact_with_its_source_and_grade` |
| `EFF-3` | `tests/conformance/EFF-3/*` |
| `TCB-2`, `TCB-3` | milestone `tcb_lists_every_foreign_fact_with_its_source_and_grade` |
| `CLI-13` | milestone `calls_foreign_lists_what_an_item_reaches_with_each_contract` |
| `DIA-18` | `tests/conformance/DIA-18/*` |

## Left, and why

* `[TCB-1]` (now in `unfinished`): grades above asserted. `@grade(…)` is an overlay item's attribute
  (the attribute table), overlays reach a program only through `import c … with (overlay=…)`, and the
  C header importer is outside this job. No evidence records, W5050/E5050, `--require`.

## Defects found

* **`E5050` means two things.** The registry gives `E5050` to "a foreign fact claims a grade whose
  evidence is absent or stale" (`[TCB-1]`); the compiler emits it for "type has no foreign
  representation" (its page says so too). The right code for that is `E5015` ("type may not cross the
  boundary"). Moving it now leaves `E5050` with no producer, page or test, which the `rule_index`
  ratchet refuses (it grows only with a new specification); so it stays until grades are built, and
  the foreign-static error uses `E5050` too, consistently. Fix with `[TCB-1]`'s grade check.
* **A `static` of function type cannot be called by name** (`static HOOK: extern "C" fn(i32) -> i32
  = f` then `HOOK(7)` is `E1010 cannot find`); a foreign one neither. Binding it to a local first
  works. Not fixed (outside this job).

## Owner questions

(none yet)
