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

## 7. Overlay files (`[GRM-35]`, `E5067`; `@grade` and `W5050` of `[TCB-1]`/`[TCB-2]`)

`ast::ItemKind::Overlay(OverlayDecl)`: `[unsafe] overlay c|cpp "header":` with `fn` entries whose
parameters take contract words (`ast::Contract`: a word, optional arguments) in place of types,
`-> words`, `type`, `enum`, `rename … as …`, `hide …` and `extend` blocks; `overlay` is a keyword
only where an item begins with `overlay c "…"` (or `unsafe overlay`), `rename`/`hide` only inside
one. The checker (`check_overlay`): `E5067` when an overlay states a fact (a function entry, a
contract word, an `@ffi` attribute) and is not `unsafe overlay`; each contract word is the
vocabulary's (`E0104`); `@ffi` applies to `fn`/`type`/`enum` entries; `@grade(asserted |
instrumented | checked | proven)` on an entry, a grade above asserted with no evidence `W5050`
(an overlay entry has no evidence record to name until the importer turns it into a declaration;
section 12's records are for hand declarations), the fact staying asserted.
Nothing applies an overlay: that is `import c … with (overlay=…)`, the importer's. A module may
hold an overlay beside other items (the grammar describes an overlay file; tests need a `main`
for the harness's run); deciding to refuse the mix was not needed.
`spec_check`'s baseline shrank by one (Part XVI's overlay example parses now).
Pages `docs/errors/E5067.md`, `W5050.md`. Tests: `tests/conformance/GRM-35/`,
`tests/conformance/TCB-2/accept_a_written_grade_stays_asserted.em`.

## 8. An object that is not Sync is its maker's (`[FFI-33a]`)

In the debug profile (`EMBER_RT_DEBUG_ACCESS`) `ember_obj_new` writes the making thread's number
into the high 16 bits of a non-Sync object's `flags` (the layout table assigns only bits 0 and 1),
and `ember_retain_plain`/`ember_release_plain` (inline) and `ember_weak_retain`/`_release` call
`ember_obj_check_thread`, which panics naming the class and both threads (`thread 1 (main)`).
Thread numbers are a thread-local counter, 1 for the thread that initialises the runtime; past 65535
threads two can share a number, which can miss a report but never make a false one. Release
profiles neither record nor check. Safe Ember never moves a non-Sync handle to another thread (it is
not `Send`), so the check fires only for foreign threads that attached and touch Ember objects, which
is the rule's subject. Test: `runtime/ember_rt/templates/tests/object_creator_thread.c.in`, run by
`ember_build`'s `an_object_that_is_not_sync_is_its_makers` (debug and release, a C thread that
attaches and retains, releases or weakly retains; a Sync object; the maker itself).

## 9. `ForeignBox`, `std.ffi.adopt` (`[FFI-36a]`, `[FFI-36b]`, part of `[FFI-33b]`)

`std.ffi.ForeignBox[T]` is written in Ember: `pointer: *mut T`, `destructor: extern "C" fn(*mut T)`,
`creator: u32`; `as_ptr()`; its `drop` calls the compiler-known `foreign_release` then the
destructor. It is `Send` (`unsafe extend`): a box's thread rule is its contract's, checked at its end.
`std.ffi.adopt[T](h)` is compiler-known (`synth_foreign_adopt`): `T` must be an opaque foreign type
whose declaration names its destructor, `@ffi(destroys(f))` (`f` a foreign function of the same
module taking one `*mut T`); `@ffi(adopt=false)` is `E5052`; no destructor or not a foreign type is
`E5002`; outside `unsafe` `E3100`. It builds the box as a struct literal: `Builtin::ForeignAdopt`
(the runtime's `ember_ffi_adopt`, recording the address with the call's place),
`Builtin::ForeignDestructor` (the function itself when it returns nothing, else a C thunk
`ember_ffi_destroy_<f>` that drops its status), and `Builtin::ForeignThread` when the type says
`@ffi(threads(creator))`. The runtime's record (an open-addressing table under a spin lock, in every
profile as `[FFI-36b]` says) panics at a second owner of a live address naming both places;
`ember_ffi_release` takes the address out and, in debug, checks a creator box ends on its thread.
A foreign type's `@ffi` takes `handle`, `destroys(f)`, `adopt=…`, `threads(any|main|creator)`
(`E0104` otherwise). Tests: `tests/conformance/FFI-36a/`, `FFI-36b/`, milestone
`adopted_foreign_objects_are_freed_by_their_destructor` (fixture `tests/fixtures/foreign_objects.c`:
frees counted, a status-returning destructor, double adoption in debug and release, a creator box
dropped on a spawned thread panicking in debug only).

## 10. `owned` parameters and `returns_owned` results (`[FFI-11]`, `[FFI-36a]`)

On a `safe fn`: `@ffi(param(p, owned, one[, nullable]))` takes `owned p: ForeignBox[T]` (`Option` of
one with `nullable`), and `@ffi(result(returns_owned(destructor=f), one[, nullable][,
threads(creator)]))` gives a `ForeignBox[T]` (`Option`, `None` for null). They use the existing C
wrapper (`hir::FfiCounted`, new `FfiAbiParam::Owned` and `FfiOwnedResult`): a taken box leaves the
runtime's record (`ember_ffi_release`, its creator checked in debug) and C gets its pointer; a
returned pointer is recorded (`ember_ffi_adopt`, the declaration's line as the place) and boxed with
`f` (or its thunk) and its creator. `check_foreign_ownership` gives `E5002` for an `owned` contract
on anything but an owned box, a box crossing without a contract, a `returns_owned` result that is not
a box, and an `f` that is not a foreign function taking one `*mut T`. `owned` and `unsafe` are
contract words inside `@ffi(…)` (the parser takes the keywords there as words, as it took
`static`). Tests: `FFI-36a/accept_returns_owned_and_owned_parameters.em` (run by the milestone of
§9), `FFI-36a/reject_ownership_contract_mismatches.em`.

## 11. `Retained` and `retained` parameters (`[FFI-23]`, `[FFI-11]`, `[FFI-22]`)

`std.ffi.Retained[T]` holds the pinned value itself (`value: T`): a class handle's strong count, or
the `Box`. `Retained.pin(v)` (Ember) calls the compiler-known `retained_pin` (a class object's
header gets `EMBER_OBJ_PINNED` through the new `ember_obj_pin`; a `Box` nothing) and `ptr()` the
compiler-known `retained_address` (`*mut u8`: the handle, the `Box`'s `T*`, or a `Box[dyn]`'s
data). Dropping the token drops the value, which releases the object or frees the box; no `drop` of
its own. A `T` that is neither a class nor a `Box` is refused where `Retained[T]` is instantiated
(`E2020` at the user's `pin`), so the std bodies never see one. The pinned flag is set and not
cleared: a single bit cannot count tokens, and keeping it is the safe side for `[HR-20]` (an object
foreign code may hold is never moved). A safe declaration's `param(p, retained, one[,
threads(main | creator)])` takes a borrowed `Retained[T]` and gives C the address (`FfiAbiParam::
Retained`); without a thread word `T` must be `Send` (`E7004`, `[FFI-22]`). `E5002` for the contract
on another type, or a token crossing without it. `ptr()`'s type (`*mut u8` for every `T`) is a
choice the spec leaves open (owner question 4). Tests: `tests/conformance/FFI-23/`, milestone
`retained_tokens_let_c_keep_ember_objects` (a class object outliving the handle it was pinned from
until its token drops, the kept address, C reading a pinned `Box`'s value).

## 12. `ember test --instrument-ffi` and evidence records (`[FFI-37]`–`[FFI-37d]`, `[TCB-5]`, `[TCB-6]`)

The code generator (only when the flag is given) wraps every call to a foreign declaration in the
test binary's main unit with `ember_ffi_enter(function, site)` / `ember_ffi_leave(previous)` (a
thread-local "foreign function running now"), numbers the call sites, and links a unit of its own,
`compiler/ember_codegen_c/src/ffi_instrument.c`. That unit intercepts the allocation, lock, wait and
I/O entry points: on Windows by patching the import table of every module loaded at the test's start
(`malloc`/`calloc`/`realloc`/`_aligned_malloc` of the UCRT, `HeapAlloc`, `VirtualAlloc`, critical
sections and SRW locks, `Sleep`/`WaitFor*`/condition variables, `ReadFile`/`WriteFile`/`CreateFile`/
`WriteConsole`); on Linux (glibc) by defining the C library's names in the executable (`malloc` via
`__libc_malloc`, the rest via `dlsym(RTLD_NEXT)`). Each test writes its counts to the file the runner
names (`EMBER_FFI_EVIDENCE`); the runner merges them, grades each declared absence (`@ffi(effects=
[…])` without `Alloc`, `Lock`, `Block` or `Io`): reached is `E5053` and fails the run; a function no
test called is `W5054`, `unexercised`; otherwise `instrumented`, `partial` when a call site never
ran. The report prints the intercepted set, what is structurally unobservable, and that the other
fact kinds stay asserted. The evidence record (`<[ffi] evidence>/<entry>.json`, default
`.ember/ffi-evidence`) holds the intercepted set, the identities with hashes (toolchain, test binary;
per function its declaring source and the library its address is in), the environment (machine,
time) and per function its calls, its sites with hits and its graded facts. `ember tcb` reads it:
graded facts per declaration with "no counterexample was observed on the paths exercised, which is
not that the fact holds"; stale (facts stay asserted) when an identity input's hash changed; a
record from another machine is only noted. Off Windows and Linux the flag is `E0900` (an empty
intercepted set would grade without observing). `E5053`/`W5054` keep no page: their examples need a
test run, and `error_pages.py` checks a page with `ember check` (they stay in the baseline).
Test: milestone `instrumented_tests_grade_foreign_effect_facts` (MSVC and Clang on Windows; GCC on
Linux not run here: no Linux toolchain reachable from this worktree).

Known limit: a foreign function that calls back into Ember has the callback's allocations counted
as its own (the "running now" mark is not cleared on entry to an Ember function called from C).
Callbacks with `user_data` are not built (`[FFI-11]`); when they are, the trampoline should clear it.

## 13. `inout_count` (`[FFI-11b]`, `[FFI-11]`)

`@ffi(param(count, inout_count(items)), param(items, borrowed, count(count), nullable, exclusive))`
over `safe fn f(mut count: u32, items: Option[MutSpan[T]])`: the count is written at its C ABI
position as `mut` (C's `uint32_t*`, the hand-declaration form of a pointer to one value) and hidden
from the callable signature as ODR-074's `count(n)` witness is; the callable is
`f(items: Option[MutSpan[T]]) -> u32`. The C wrapper (`FfiAbiParam::Count { inout: true }`) holds
the capacity (0 for `None`, the span's length otherwise, `Some(empty)` a non-null sentinel per
ODR-079) in a local, passes its address, and returns what C wrote back. `check_inout_counts`: `E5002`
for a count that is not `mut` of an integer type, an `items` not `nullable count(count)`
`exclusive`, or two `inout_count`s; a C result beside it is `E0900` (with a status it would be
`Result[count, <E>Error]`, `[FFI-16]`, whose status enum is an overlay's). Tests:
`tests/conformance/FFI-11b/`.

## Rules done (each with its test)

| Rule | Test |
|---|---|
| `FFI-10` | `tests/conformance/FFI-10/*`, milestones `foreign_statics_are_places_in_c_storage`, `vectorcall_declarations_compile_with_the_convention`, `tcb_lists_every_foreign_fact_with_its_source_and_grade` |
| `EFF-3` | `tests/conformance/EFF-3/*` |
| `TCB-2`, `TCB-3` | milestone `tcb_lists_every_foreign_fact_with_its_source_and_grade`; `TCB-2/accept_a_written_grade_stays_asserted.em` |
| `GRM-35` | `tests/conformance/GRM-35/*` |
| `FFI-33a` | `ember_build` test `an_object_that_is_not_sync_is_its_makers` |
| `FFI-36a` | `tests/conformance/FFI-36a/*`, milestone `adopted_foreign_objects_are_freed_by_their_destructor` |
| `FFI-23` | `tests/conformance/FFI-23/*`, milestone `retained_tokens_let_c_keep_ember_objects` |
| `CLI-13` | milestone `calls_foreign_lists_what_an_item_reaches_with_each_contract` |
| `DIA-18` | `tests/conformance/DIA-18/*` |
| `FFI-37`, `FFI-37a`–`FFI-37d`, `TCB-6` | milestone `instrumented_tests_grade_foreign_effect_facts` |
| `FFI-11b` | `tests/conformance/FFI-11b/*` |

## Left, and why

* `[TCB-1]` (in `unfinished`): a written grade backed by evidence. `@grade(…)` is an overlay item's
  attribute (the attribute table), overlays reach a program only through `import c … with
  (overlay=…)`, and the C header importer is outside this job. Evidence records exist now (section
  12) and `ember tcb` shows them; W5050/E5050 for a claimed grade and `--require` wait for overlays.
* `[TCB-5]` (in `unfinished`): the record holds every identity there is now except the header's;
  hand declarations have none. Add its hash when `import c` lands.
* `[CLI-6]`, `[CLI-7]`, `[FFI-12]`, `[FFI-13]`: about what the C header importer produces
  (skipped declarations, an overlay checked against its header, `extend` blocks over a foreign
  module); outside this job.
* `[FFI-16]`, `[ERR-6]`: the status enum is marked in an overlay (`@ffi` sits on extern and overlay
  items only, the attribute table; a hand declaration has no enum), so `-> status` waits for the
  importer, or for an owner decision on a hand-declared form (question 5). `[ERR-6]`'s example also
  names the error type differently from `[FFI-16]` (question 6).
* `[FFI-11]` (in `unfinished`): `out`, `status`, `callback(…)` with `user_data` (`[FFI-21]`'s
  trampolines), `threads(…)`/`destroys(f)` on functions, `unsafe`.
* `[FFI-33b]` (in `unfinished`): callbacks carrying `threads(creator)` wait for callbacks.
* `[FFI-36b]` (in `unfinished`): the compile-time half needs a code (question 2).

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

1. **How a foreign type's declaration names its destructor (`[FFI-36a]`).** "`adopt` … reading its
   destructor from the contract of `T`'s declaration": the spec gives no spelling for a type's
   contract. Built as `@ffi(destroys(f))` on `type T` (the vocabulary's `destroys(f)`), with
   `@ffi(adopt=false)` (`[FFI-36b]`'s words) and `@ffi(threads(creator))` beside it. If the owner
   wants `destructor=f` (the `returns_owned` spelling), it is a rename.
2. **The code for two `adopt`s of one local in one function (`[FFI-36b]`).** The rule says it is
   caught at compile time, and the registry has no code for it (`E5052` is `adopt=false`). Which
   code? Until then only the run-time half is built.
3. **The type of `Retained.ptr()` (`[FFI-23]`).** "`tok.ptr()` is the raw pointer" names no type.
   Built as `*mut u8` for every `T` (a class object has no Ember pointer type of its own; C
   usually keeps a `void*`). A typed `*mut U` for `Retained[Box[U]]` would need the two kinds of
   token told apart in the type.
4. **Calling a `static` of function type by name** (`HOOK(7)`) is `E1010 cannot find`: a defect
   outside this job, noted under "Defects found".
5. **A status enum without an overlay (`[FFI-16]`).** `@ffi(status, ok=X)` marks an enum, and
   `@ffi` applies to extern and overlay items only; extern blocks hold `fn`, `static` and `type`.
   Should a hand declaration get a form (an `@repr(i32) @ffi(status, ok=X) enum` at module level, or
   an `enum` inside `unsafe extern`), or is `-> status` an overlay's only?
6. **The error type's name (`[ERR-6]` against `[FFI-16]`).** `[FFI-16]` generates `<E>Error` (for
   `VkResult`, `VkResultError`); `[ERR-6]`'s example turns a `VkResult` return into
   `Result[void, VkError]`, and spells the overlay `@ffi(status=VkResult, ok=VK_SUCCESS)` where
   `[FFI-16]` writes `@ffi(status, ok=X)` on the enum. Which name and which spelling?
7. **`inout_count` beside a C result (`[FFI-11b]`).** Built for C functions returning `void` (the
   call returns the count). With a status result it is `Result[count, <E>Error]` once `[FFI-16]` is
   built; with any other result (an `int` that is not a status), what does the call return?
