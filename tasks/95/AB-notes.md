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

## AB2. `[FFI-11]`, `[FFI-21]`, `[FFI-22]`, `[FFI-33b]` — callbacks with user data, `destroys(f)`, `unsafe`

**Built.** `param(p, callback(borrowed | once | retained[, user_data=q])[, nullable][, threads(…)])`
on a `safe fn`, hand-declared or through an overlay entry (`visit: callback(borrowed, user_data=data)`).

*The declaration.* The callback is written as C has it, its function pointer type with the user
data among the parameters (`visit: extern "C" fn(*mut void, c_int)`, `data: *mut void`; any
calling convention, `extern "system" fn` for Win32's `WNDENUMPROC`). The user data's slot in the
callback is its one parameter of `q`'s type (a pointer or an integer: `LPARAM` is `isize`); none
or several is `E5002` (spec question 1). In the callable signature the parameter becomes the
callable a caller passes — a `[CLO-3]` bound of the other parameters (`fn(c_int)`), lent for the
call under `borrowed`, taken over under `once` (a `once fn`) and `retained` (a `fn`) — and `q`
leaves it, as a `count(n)` witness does (ODR-074's shape one level in).

*The wrapper (why it is fast).* The callable is a bound, so the safe declaration is generic and
**each instance has a C wrapper of its own**: `check_instantiations` makes one for a foreign
declaration (`foreign_instance`), with the `FfiCounted` metadata of that instance's callable
(`hir::FfiCallback`). The wrapper passes C a trampoline (`<wrapper>_cb<k>`) and the callable's
address as the user data; the trampoline attaches the thread (`[FFI-22]`), and calls a lambda's
body **directly** with its environment (a callable value through its entry). Under `borrowed` the
callable stays where the caller has it: **no allocation, no copy** (the spec's "box" is the
caller's frame for the call). Under `once` and `retained` the callable moves into one allocation
(`<wrapper>_cb<k>_record`, `ember_box_new_copy`); `once`'s trampoline ends it after its call
(unless the lambda took its environment over) and frees it.

*`retained` and `destroys(f)`.* The call returns a `Retained[ForeignCallback]` token
(`std.ffi.ForeignCallback`: the allocation's address and its release), an output in the
callback's C position (C's own result first, as for `out`). Its end calls the generated release,
which calls the function's `destroys(f)`, ends the callable and frees the allocation. `f` is a
foreign function of the module whose parameters are each named as one of the registering
function's: the callback (given the trampoline, as `Option[…]` too), its user data (the
allocation's address), or a scalar parameter (what the call passed, kept in the allocation).
Without `destroys(f)`: `E5041`; a parameter of `f` that is none of those: `E5002` (spec question
2). `tok.ptr()` of such a token is the user data C keeps. One `retained` callback per function.

*What a callback captures* (`foreign_callback_refused`, at the call): `Send` (`E7004`) unless
`threads(main)` or `threads(creator)` (`[FFI-22]`); under `once`/`retained` a borrowing lambda is
`E3026` (write `owned fn`; the region analysis keeps an `owned fn`'s views static, `E3063`). A
callable value of unknown captures is not `Send`.

*`threads(creator)` / `threads(main)`* (`[FFI-33b]`): the record keeps the creating thread; in
`debug` the trampoline (and a `retained` release) calls `ember_ffi_callback_thread`, which panics
naming both threads when C calls on another (new runtime function, `EMBER_FFI_MAIN_THREAD`).

*Without user data*, `callback(…)` on an `extern "C" fn` parameter (`Option` with `nullable`) makes
the plain function pointer safe to pass; a capturing lambda there is now `E5040` (was `E2020`,
`FN-6/reject_capturing_closure_as_extern_pointer.em` updated to the code the spec's table gives,
with a help naming `callback(…, user_data=p)`).

*`unsafe`* (`[FFI-11]`: "leave the call unsafe"): `@ffi(unsafe)`, `param(p, unsafe)`,
`result(unsafe)` on a hand declaration; on a `safe fn` `E5002`. The call's `E5002` then says the
contract leaves it unsafe on purpose instead of asking for facts; an overlay's `unsafe` word now
reaches the declaration as `@ffi(unsafe)`.

*Elsewhere*: the effects analysis follows a callback to the lambda it calls (`[EFF-2]`; a callable
value's are every effect), `reached_bodies` keeps the lambda, `ember inspect --cost` names each
callback's handling, the importer gives a function pointer parameter its function type
(`CType.function`, `.embind` version 11) and imports `ForeignCallback` for a module with a
`retained` callback.

Tests: `tests/conformance/FFI-21/accept_a_borrowed_callback_takes_a_lambda.em` (hand declarations,
`visits.c`: user data first and last, an integer user data, local functions writing the caller's
locals, a function), `accept_callbacks_c_keeps_past_the_call.em` (overlay `overlays/events.em`,
`events.c`: `once` and `retained` with `destroys(unlisten)` given the kind, trampoline and data;
strings moved into the callables; tokens ended), `reject_callback_contract_misuse.em`,
`reject_what_a_callback_captures.em`; `tests/conformance/FFI-33b/accept_a_creator_callback_on_its_thread.em`,
`run_fail_a_creator_callback_called_on_another_thread.em` (`ticks.c`: a C thread calls it, debug
panics); `FFI-11/reject_a_contract_left_unsafe_asks_for_no_fact.em`. The emitted C of the run-pass
tests and the C sources compiled with gcc 15 in WSL (`-Wall -Wextra -Werror`); MSVC and clang-cl
with the C gate on.

**Not built:** a `nullable` callback with user data (`E0900`: the callable bound has no `Option`
form); callbacks of a C++ function reached through a wrapper (see AB-later).

**Defect found and fixed:** a header typedef of a function pointer naming a C alias (`typedef void
(*done_fn)(void*, int)`) was `E1010 cannot find type c_int` whenever the importing file itself
imported from `std.ffi` (`main`'s compiler too): `alias_body_is_resolvable` did not look inside a
function type, so the alias was resolved before `std.ffi`'s `c_int` was. Probe: a file with `from
std.ffi import Retained` and `import c "./events.c"`.

### Compiler changes for AB2

- `ember_hir`: `FfiCounted.callbacks`, `FfiCallback`, `FfiCallbackMode`, `FfiUnregister`,
  `FfiUnregisterArg`; `FfiAbiParam::Callback`, `CallbackData`.
- `ember_typeck`: `ffi_callback_param_arg` (and `ffi_threads_word`, `ffi_destroys_arg`,
  `ffi_unsafe_slot_arg`, `ffi_left_unsafe`), `collect_foreign_callbacks`, `check_foreign_callbacks`,
  `foreign_unregister`, `foreign_callback_token`, `foreign_instance`, `foreign_callback_refused`;
  `foreign_counted_metadata` gains the callbacks (now `&mut self`); `closure_symbols`;
  `Retained[ForeignCallback]` is allowed; `alias_body_is_resolvable` looks into function types.
- `ember_codegen_c`: `emit_ffi_callback` (record, trampoline, release), `emit_ffi_callback_data`.
- `ember_analysis::effects`, `ember_mir::reached_bodies`, `ember_driver` (`inspect`, importer).
- runtime: `ember_ffi_callback_thread`, `EMBER_FFI_MAIN_THREAD`; `std.ffi.ForeignCallback`.

## AB3. `[FFI-1]` — a foreign declaration's contract carries its effects and its thread rule

**Done** (`FFI-1`'s and `FFI-33b`'s unfinished entries are gone; `FFI-11`'s says what is left).

*Effects, derived where the C declaration gives them.* The importer reads `__attribute__((const))`
(no effects, and the same result for the same arguments: `@ffi(effects=[], deterministic)`) and
`__attribute__((pure))` (no effects, it only reads memory: `@ffi(effects=[])`) on a C function,
with a `# derived:` comment, unless the overlay entry states its own (`Derived`, `.embind` version
12). The facts make no call safe (`[FFI-2]`). With no statement the effects stay every effect
(`[EFF-3]`).

*The thread rule.* `@ffi(threads(any | main))` on a foreign function is its thread rule
(`hir::Contracts::foreign_threads`; unknown when the contract states none: no C declaration says
one). Under `threads(main)` the debug profile checks each call is on the main thread before C runs
(`ember_ffi_check_thread`, which the callback checks now share), naming the function and both
threads. `threads(creator)` on a function is `E0900`: a function has no creator of its own (spec
question 3). `ember tcb` lists the rule ("its thread rule: `threads(main)`") and, where the
contract states none, "its thread rule: none stated, so unknown [FFI-1]".

Tests: `tests/conformance/FFI-1/accept_a_declaration_states_its_effects.em`,
`reject_a_declaration_that_states_nothing_has_every_effect.em` (`maths.c`),
`accept_a_thread_rule_is_part_of_the_contract.em`, `run_fail_a_main_thread_function_called_elsewhere.em`
(`ui.c`), `reject_a_function_has_no_creator.em`; milestone
`tcb_lists_every_foreign_fact_with_its_source_and_grade` (the thread rule lines).
`FFI-49/reject_unimplemented_ffi_contract.em` took `threads(main)` as its example of a word not
built; now `handle` on a function is.

## AB4. `[FFI-24c]` (ODR-220, the owner's answer) — catching C++ calls on `@noalloc` and main-thread paths

**Done** (`FFI-24c`'s unfinished entry is gone). `ember inspect --cost`'s row of each catching C++
call says when it is on a `@noalloc` path or on the main thread's, and where the path starts:
`catching; on a \`@noalloc\` path, from \`steady\`; on the main thread's path, from \`main\``. A
`@noalloc` path is what a function contracted `@noalloc` reaches by direct calls (a catching call
compiles there only under `@assume_noalloc`); the main thread's is what `main` and an export
contracted `threads=main` reach. A spawned thread's or a job's function is reached through the
runtime, so it is on neither. `ember bind --report` reads a header and lists none (as ODR-220
says; nothing to change there). Test: milestone
`inspect_marks_catching_cpp_calls_on_noalloc_and_main_thread_paths`. Compiler change:
`safety_costs_json` (`ember_driver/src/main.rs`).

## AB5. `[FFI-39]`, `[FFI-44]` (ODR-221, the owner's answer) — deriving from a C++ class with no overlay entry

**Done.** An Ember class may derive from a C++ class with no overlay entry. The importer reads
the program's classes (`cimport::program_module`, `program_trampolines`): every class whose base
is a class of the header (`class Mine(cpp.Stage)`, or through another class of its module) asks
the header's class for a trampoline, which passes on the virtuals those classes declare
`override`. To see them, `load_package_modules` now imports each `import c`/`import cpp` once
every module of the program is read (the worklist holds the prelude's alone): the generated
modules still come before the prelude's in the module list, as before (they must: see the defect
below). An overlay's `@ffi(trampoline, virtuals=[…], owner=…)` entry stays accepted (its names and
`owner` are merged in); nothing depends on it. Every pure virtual is passed on, and a C++ class
with one is an Ember `abstract class` whose pure virtuals are bodiless `virtual fn`s, so a class
deriving from it that does not implement one is the abstract-class error (`E2020`, was `E5056` at
the import). A base that cannot be derived from is refused where the class names it, saying why
(`E5059` "`Deep` cannot derive from `Flat` of `panels.hpp`: `Flat` has no virtual destructor…";
`E5034` for `final`). The trampolines the program asks for are part of the `.embind` key.

*Exactly what each class overrides.* The trampoline of a C++ class overrides the union of what the
program's classes override, and each object passes on exactly what its own class overrides: the
base's generated `init` gives the trampoline an entry only for a virtual its object's class
overrides (`ember_cpp_overrides[C](self, "name")`, a new compiler-known builtin of `import cpp`
modules, `Builtin::CppOverrides`: the class's method-table slot compared with the base's own
method, one load and compare per virtual when the object is made), and the trampoline calls the
C++ implementation directly where it has none. In a reloadable build every entry is given (a
reload may add an override, `[HR-9a]`).

`reject_an_override_of_a_virtual_the_overlay_does_not_name.em` is gone (ODR-221 removes its
premise: the override is now passed on, `accept_overrides_beyond_an_overlay_entry_are_passed_on.em`);
`E5056` stays only where the importer could not see a class (a base reached through another
module's class, or a header the two modules spell differently).

Tests: `tests/conformance/FFI-39/accept_an_ember_class_derives_with_no_overlay_entry.em` (no
overlay; `stages.hpp`: overrides reaching C++, a class overriding another virtual, one overriding
none, an abstract C++ class implemented), `accept_overrides_beyond_an_overlay_entry_are_passed_on.em`,
`reject_a_class_that_does_not_implement_a_pure_virtual.em`, `reject_a_cpp_base_that_cannot_be_derived_from.em`;
the thunk file compiled with g++ 15 and clang++ 21 in WSL (`-Wall -Wextra -Werror`).
`FFI-44`'s unfinished entry no longer lists inheritance; `FFI-39b`'s says it is built as ODR-219
rules (it goes with the next cut).

**Defect found (not fixed, typeck):** checking depends on the order of the module list: with
`std.ffi` after `std.core` (as the first form of the deferral put it), every program importing a
header failed with "`std.ffi.CppError` implements `Error` but does not define `source`" — a pass
over the modules in reverse visits an `extend … implements Error` before `Error`'s default methods
are known. The importer's modules are kept where they were; the order dependence itself is open.

## AB6. `[FFI-8]` — the type-mapping rows (first part), and `[ENM-3]`'s `from_repr`

**Built so far** (each with its test; the rows below still open are listed in `FFI-8`'s entry):

- *Enum-typed bit-fields.* A bit-field of a C enum has `get_<f>`/`set_<f>` of the enum, through
  the enum's integer type (sign-extended where it is signed: MSVC's enums are `int`), the
  getter turning the integer back with `from_repr` (every value is a C enum's, so it is always
  `Some`). Test: `FFI-8/accept_enum_bit_fields_are_read_and_written_by_accessors.em`
  (`enum_bits.h`).
- *`[ENM-3]` `E.from_repr(x) -> Option[E]`* was missing (the rule had no `unfinished` entry and a
  test naming it, but `from_repr` was "no variant `from_repr`"): built in typeck
  (`synth_enum_from_repr`, HIR only: `Some` of the cast when a variant's discriminant is `x`,
  else `None`; a `@non_exhaustive` enum's is always `Some`, with no test). Test:
  `ENM-3/accept_from_repr_converts_back.em`. **Defect, fixed** (for the lead to number).
- *`long double`* is `std.ffi.c_longdouble`, a new `TyKind::CLongDouble`: opaque bytes of the
  target's layout (`c_long_double_layout`: 8 on Windows and Apple ARM, 16 on x86-64 and AArch64
  Linux, 12/4 on 32-bit x86), emitted as C's `long double` so it is passed and returned as C
  passes it (System V's x87 class), `Copy`, FFI-safe, no arithmetic, comparison or `as`. The
  importer maps `long double` to it. Tests: `FFI-8/accept_long_double_is_c_longdouble.em`
  (`precise.c`, a struct holding one), `reject_c_longdouble_has_no_arithmetic.em`.
- *Unions.* A C union is `@layout(c, union) @derive(Copy) struct` (spec question 4 on the
  spelling): `LayoutAttrs.union`, every field at offset zero (`aggregate_layout`), emitted as a C
  `union`; made with one field by name (`Builtin::UnionLit`, a designated initialiser, the rest of
  its storage zero); a field read is `unsafe` (`E3100`, `check_union_read`), a store into one
  (`u.f = v`, the whole assignment target) is not; no implicit `Eq`/`Debug` (they would read), its
  `Clone` a copy; each field `Copy` (`E0104` otherwise). An anonymous union member of a struct is a
  generated union type read through, in `unsafe`. A union with bit-fields stays opaque. Tests:
  `FFI-8/accept_unions_are_layout_c_unions.em` (`unions.h`), `reject_a_union_field_is_read_in_unsafe.em`.

`FFI-20a`'s header gave a union, an enum bit-field and `long double` as what cannot be represented:
now a union with bit-fields and a function with no prototype (its tests are updated).

**Built next** (AB6 continued):

- *Flexible array members.* A struct a flexible array member ends is `@layout(c, flexible)` (no
  `Copy`; spelling as in spec question 4): unsized, so it is used by value nowhere (`E5017`, the
  spec's "cannot be used by value": a parameter, a result, a local, a field, an element); held
  behind a reference or a pointer, its elements reached by the generated
  `unsafe fn tail(self, n) -> Span[T]` (the elements follow the struct at the member's offset,
  `@safety` "the allocation holds at least `n` elements"). A member after it, or one in a union,
  stays opaque. Tests: `FFI-8/accept_a_flexible_array_member_is_reached_by_tail.em` (`packets.c`,
  an overlay giving the C allocator `returns_owned`/`destroy`),
  `reject_a_flexible_struct_is_never_by_value.em`.
- *`volatile`.* `std.mem.Volatile[P]` (`Copy`, `new`, `as_ptr`; `read_volatile`/`write_volatile`
  on `Volatile[*mut T]`, `read_volatile` on `Volatile[*T]`, all `unsafe`), each access a
  compiler-known `std.mem.volatile_read`/`volatile_write` emitted as C's
  `*(volatile T*)p`. To C it is the pointer (one field, as `ObjectPtr` is): C's `volatile T*` is
  `Volatile[*mut T]`, `const volatile T*` `Volatile[*T]`, as a parameter, a result or a field.
  Assumption to confirm: a one-pointer struct is passed as the pointer by every ABI Ember targets
  (it is for System V, Win64 and AAPCS64; `ObjectPtr` already relies on it). Test:
  `FFI-8/accept_volatile_pointers_are_volatile.em` (`registers.h`).
- *`_Atomic T`* is `std.sync.Atomic[T]` when `T` is one `Atomic` holds (`[THR-14]`: an integer,
  `bool` or a pointer) and Clang's size and alignment of `_Atomic T` are `T`'s; else what uses it
  is skipped with `E5016` (reported where it is named; libclang 11's `clang_Type_getValueType`,
  so `clang-sys`'s feature is now `clang_11_0`). A record holding an `Atomic` is written without
  `@derive(Copy)`. **Build change:** cl compiles `<stdatomic.h>`/`_Atomic` only with
  `/experimental:c11atomics` (Visual Studio 17.5 on), so `compiler_command` passes it to cl
  (not clang-cl, which has C11 atomics and warns on the switch); an older cl warns D9002 and
  ignores it. A skip reason's code tag now leads the "parameter `p`:"/"its result:"/"field `f`:"
  context (`tagged`), and the W5002 text drops it. Tests: `FFI-8/accept_atomic_is_std_sync_atomic.em`
  (`atomics.h`), `reject_an_atomic_ember_cannot_hold.em` (`atomic_levels.h`).

## AB7. `[FFI-8]` — `va_list` and variadic calls; `@ffi(newtype)` left as a question

**Built.**

- *`VaList`* (`std.ffi.VaList`, a compiler type `TyKind::VaList` like `c_longdouble`): C's
  `va_list` as a parameter holds it, emitted as the runtime header's `ember_va_list` — `va_list`
  itself, except where `va_list` is an array (x86-64 System V), where it is what a parameter of it
  decays to (`__typeof__(&(*(va_list*)0)[0])`), so a `VaList` is moved and passed as C passes the
  parameter. It is a view (alive for the call that passed it), not `Copy` and not `Send`/`Sync`;
  never constructed: `VaList(…)`, `ffi.VaList(…)` are `E5018`. The importer maps `va_list` (by its
  written name: on Windows it is a `char*`; a function pointer's parameters are read through their
  written names too) to `owned VaList` — whoever is passed a `va_list` reads it, so it is passed on
  once (`E3040` on a second use). A callback receiving one declares `owned arguments: VaList`.
- *Variadic C functions* are imported with `@ffi(variadic)` (a hand declaration writes the same):
  the prototype ends `...`, a call is `unsafe` (`E5002`, its own message; never a `safe fn`), the
  declared parameters are checked as any call's, and each further argument must have the type the
  program gives it and that C passes unchanged — a C `int` or wider integer up to 64 bits, `f64`,
  `c_longdouble`, a raw pointer, `cstr`, an `extern fn`; an unsuffixed literal, `bool`, the
  integers narrower than `int` and `f32` (which C's default argument promotions change) are
  `E5015` with the cast to write. The function is never a value (`E5015`). A contract that keeps
  what C is passed (`cstr`, `ref T`, `Option[…]`) types the declared parameters (an overlay entry
  does so for an imported one, which stays `unsafe`); one that needs a wrapper (a count, `out`, a
  status, a callback) is `E0104`, as a wrapper cannot pass `...` on. A variadic `static inline`
  function (or a C++ header's inline `extern "C"` one) is still skipped (`[FFI-29c]`), as is C23's
  `f(...)` and a variadic C++ function; `FFI-20a`'s header and `docs/errors/W5002.md` now show the
  `static inline` case. `.embind` version 14.
- Tests: `FFI-8/accept_variadic_calls_and_va_list.em` (`variadic.c`/`.h`, `overlays/variadic.em`:
  further arguments, `snprintf`/`vsnprintf` by other names, a logger handing its sink a `va_list`
  the sink passes on), `reject_variadic_calls_are_unsafe_and_typed.em`,
  `reject_a_va_list_is_never_constructed.em`, `reject_a_va_list_is_passed_on_once.em`. The C of the
  accept test compiles with gcc 15 in WSL; the FFI-8, FFI-20a, FFI-21 and FFI-39 directories pass in
  WSL through the harness with the compiler built there (Linux libclang 21, System V's array
  `va_list`). Two importer fixes that run found: a `va_list` parameter is read by its written name
  before an array parameter decays (`parameter_type`), and a callback parameter's function type is
  read through its typedefs rather than canonically, which had imported Clang's builtin
  `__va_list_tag` as a struct whose layout checks the shim could not compile.

**Error pages** for the codes this work emits: `E5016`, `E5017`, `E5018`, `E5040`, `E5041`
(`tools/error_pages.py` passes).

**`@ffi(newtype)` — spec question (not built).** `[FFI-8]`'s row says a `typedef` is "a type alias,
or a new type with `@ffi(newtype)`", and nothing says what the new type is. Options: (1) a one-field
`@layout(c) @derive(Copy) struct T: pub value: U` — simple, but Win64 passes a one-`double` struct in
an integer register and a `double` in `xmm0`, so a foreign call would need the field passed, not the
struct; (2) a nominal type the compiler gives the typedef's representation (as `[RNG-1]`'s range
types are nominal over theirs), made by `T(x)` and read by `x.value`, emitted as the C type itself:
no ABI question. Recommendation: (2). Where the entry goes is not in question: an overlay's
`@ffi(newtype) type fd_t`.

**ODR-221 completed across modules.** The importer now follows a class's base through every module
of the program (`header_base`): a class of the module, an item imported from another module
(`from m import Base`, `*`), a module's class (`m.Base`), a type alias of one, and a module's
re-export of the header (`m.stages.Stage`, `[FFI-30b]`). With that, an override is never left out of
its trampoline, so `E5056` no longer arises: its emission is removed, its page says it is retired by
ODR-221 (the spec's code table and the `[FFI-39]` row "overriding a virtual not named in
`virtuals=[…]`" need the lead's cut); a base the importer still could not follow would be `E0900`
(no such form is known). Test: `FFI-39/accept_a_class_derives_through_another_module.em`
(`support/stage_kinds.em`).

**Defect found (not fixed):** two modules that each `import cpp` one header (`import cpp
"../stages.hpp"` in one, `import cpp "stages.hpp"` in another) both write the module `cpp.h_stages`,
and every member of its classes is then `E1030` ("already has a method named `tick`"); main's
compiler does the same. `[FFI-30]` makes them one entity. The re-export form above avoids it.

## AB8. `[FFI-48]` — the unsupported constructs, each `E5034` naming its row

**Done** (the `FFI-48` entry is removed). Beside multiple and virtual inheritance and
allocator-parameterised containers (built before):

- *C++20 modules:* a header with a line that declares or imports a module (`export module m;`,
  `module;`, `import m;`, `import <h>;`) is not read: the import is `E5034` ("a compiled module
  interface has no stable AST; parse the headers instead"). An import refused for an `[E5034]`
  reason is now reported with that code rather than `E9002`.
- *Coroutines:* a function whose result's class declares `promise_type`, or whose body awaits,
  yields or returns as a coroutine (`co_await`, `co_yield`, `co_return`), is not imported; naming it
  is `E5034`.
- *Overloads differing only in their return type:* of two methods of a class with the same
  parameters and different results (`int& value()`, `const int& value() const`), the first is
  imported and the other is skipped with that row's reason ("rename one in the overlay").
- *Casts across a hierarchy the importer did not model:* `as?`/`as!` where either side is a C++
  object Ember holds as a value or behind a pointer (not as a class of its own) is `E5034`.
- *Non-type template parameters of class type:* a specialization of such a template is opaque with
  that reason (so a function taking or returning one by value is not imported, `E5034` where it is
  named), and `instantiate=[…]` naming one refuses the import with `E5034`.
- *Overriding a virtual not named in `virtuals=[…]`:* retired by ODR-221 (AB7).

Tests: `FFI-48/reject_a_cpp20_module_is_not_read.em` (`modular.hpp`),
`reject_unsupported_cpp_constructs.em` (`unsupported.hpp`, `nttp.hpp` as the C++20 project `nttp`
of the directory's new `ember.toml`); they pass on Windows and in WSL. The C++ corpus records are
unchanged on both platforms.
