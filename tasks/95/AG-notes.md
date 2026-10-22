# Helper AG — the C interop rulings, D-879, the first week, the debugger's views, BEN-4 (notes)

Helper AG, 2026-10-23. Branch `worktree-agent-a81bfc704dc1489ec`, from `main` at `a8540ad5`.
Target 0.9.10_Hardened_35 (rulings ODR-245..ODR-253 in the spec since cut 35).

## AG1. The C/C++ contract rulings (spec cut 35) — done

### ODR-245 `[FFI-21]` — `user_data=p@N`

**Done.** Inside `@ffi(…)` and an overlay's contract words, `p@N` is one word (the parser keeps it
as the name `p@N`, so `ember fmt` writes it back as written). `callback(…, user_data=p@N)` names
the callback's `N`th parameter (1-based, as the C signature writes them) as the one that receives
the user data; the callable a caller passes takes the others, in their order. Without `@N`, the
user data is the callback's one parameter of its type; several are `E5002`, whose help names the
`@N` form and the places it may be (`user_data=data@N`, `N` … `(1 or 2)`). A place the callback
does not have, or one of another type, is `E5002`; `@0` is `E0104`. The importer's C++ wrapper
drops the parameter `@N` names (`callable_of`).

### ODR-246 `[FFI-11]` — `destroys(f)` given the registering call's result

**Done.** `f`'s parameters are each named as the registering function's own (the callback, its
user data, a scalar parameter) or, when the name is none of them, given the registering call's
result: C's own result, when it is a number, a `bool`, a `char` or a C enum (an id or a handle C
counts by; not a status, which says only whether the call worked; not a pointer). The token keeps
it: the callback's record gets a `result` field, written once C returns (`emit_ffi_counted_wrapper`),
and the release passes it to `f`. A parameter of `f` that is neither is `E5002` (its help names the
result's type when there is one).

### ODR-247 `[FFI-1]`, `[FFI-11]` — `threads(creator)` and `handle` on a function

**Done.** Each is `E5002` on a function (was `E0900`), whose help says where the word goes
(`@ffi(threads(creator)) type T`, a `returns_owned` result, a callback; `@ffi(handle) type T`), on a
hand declaration and through an overlay entry (C and C++). On a type both stay accepted.

### ODR-249 `[FFI-8]` — `@ffi(newtype)`

**Done.** `@ffi(newtype) type T = R` makes a nominal type, `TyKind::Newtype(NewtypeId)`
(`ember_types::NewtypeDef`: name, representation): its layout, `Copy`, `Send`/`Sync`, FFI safety
and C type are `R`'s (so it is passed to C exactly as the typedef is: `seconds_t` over `double`
goes in the floating-point register), and `as` converts between `T` and `R` only (no bits change:
HIR `EraseRange`, which lowering drops; a literal takes `R`'s type). Any other cast to or from it is
`E2020`, whose help names the step through `R`; `T` and `R` do not coerce to each other; it has no
arithmetic. `R` must be a C type (`E5015` otherwise); `@ffi` on an alias takes `newtype` alone, on a
plain alias (no `in` clause, no parameters, `E0104`). The C-identity check (`c_call_shape`, `[FFI-30]`)
sees a newtype as its representation. An overlay's `@ffi(newtype) type T` entry names the header's
`typedef` (C) or `using`/`typedef` (C++); the module writes its alias with the attribute.
`FFI-8`'s `unfinished` entry is removed.

### ODR-253 `[FFI-21]`, `[FFI-11]` — a `nullable` callback with user data

**Done.** The parameter is `Option[fn(A…) -> R]` (`Option[once fn(…)]` for `once`): a callable
value, not a bound, so the declaration is not generic. `None` gives C a null function pointer and
null user data; `Some` the trampoline and the value's address (borrowed) or an allocation holding
it (`once`, `retained`), called through the value's entry. A `retained` one returns an
`Option[Retained[ForeignCallback]]`: `None` registered nothing. What the callable captures is checked
where the lambda is written (`Send` unless the contract names a thread; a callable value whose
captures are unknown is not, `E7004`): every call records what it gives a parameter that is an
`Option` of a callable value (`note_callable_value_args`), and once every body is checked
(`check_nullable_callback_args`) a function that passes its own parameter on to a `nullable`
callback takes that callback's contract, to a fixpoint — so the C++ import's wrapper (which passes
its parameter to the thunk) and a program's own forwarding function are checked at their callers,
whichever body is checked first.

### Tests

- `tests/conformance/FFI-21/accept_user_data_named_by_its_place.em` (hand declarations,
  `registry.c`/`.h`: `@2` of two `intptr_t`s, `@2` of two pointers, `@1`),
  `accept_a_nullable_callback_is_an_option_of_a_callable_value.em` (borrowed, `retained` with
  `destroys(unsubscribe)` given the id, `once`; `None` for each), `accept_callback_contracts_through_an_overlay.em`
  (`overlays/registry.em`: the three rulings through an overlay on a C header),
  `accept_cpp_callback_contracts.em` (`hooks.hpp`, `overlays/hooks.em`: `@2`, nullable on a function
  and a method, `destroys(cancel)` given the id, through the C++ wrapper),
  `reject_user_data_places_and_nullable_callbacks.em`, `reject_cpp_callback_contracts.em`
  (`overlays/hooks_on_a_function.em`: captures through the C++ wrapper and a method, `threads(creator)`
  and `handle` on a C++ function's entry).
- `tests/conformance/FFI-1/reject_a_function_has_no_creator.em` and
  `FFI-49/reject_unimplemented_ffi_contract.em` updated to ODR-247 (`E5002`).
- `tests/conformance/FFI-8/accept_a_newtype_is_nominal_with_its_typedefs_representation.em`
  (`newtypes.c`/`.h`, `overlays/newtypes.em`: an `int`, a `double`, a struct pointer, an alias left
  alone, a hand-declared newtype), `accept_a_cpp_alias_is_a_newtype.em` (`cpp_newtypes.hpp`: `using`
  and `typedef`, a parameter, a result, a constructor), `accept_a_newtype_is_copied_and_converted.em`
  (no foreign call, so the evaluator runs it too: a field, an `Option`, an array),
  `reject_a_newtype_converts_with_as_only.em`.
- Every run-pass case also ran in `development` and `release`, with the C gate on
  (`EMBER_C_WARNINGS_AS_ERRORS=1`).

### Compiler changes (AG1)

- `ember_parser` (`body.rs`): `p@N` in an `@ffi` context.
- `ember_hir`: `FfiCallback.nullable`, `FfiUnregister.output`, `FfiUnregisterArg::Result`.
- `ember_types`: `TyKind::Newtype`, `NewtypeId`, `NewtypeDef`, `add_newtype`, `newtype_def`,
  `newtype_repr`; layout, `Copy`, threads, FFI safety, zeroability, names and mangling.
- `ember_typeck`: `ffi_user_data_word`, `FfiCallbackContract.user_data_slot`, the slot choice and the
  nullable callable value in `collect_foreign_callbacks`; `foreign_unregister` (the result, `attrs`);
  `foreign_counted_metadata` (`Result`, nullable); `nullable_callbacks`, `callback_body`,
  `callable_value_args`, `CallableArg`, `begin_callback_body`, `note_callable_value_args`,
  `check_nullable_callback_args` (called after the instantiations); ODR-247's `E5002`s;
  `newtype_aliases`, `declare_newtype`, `newtype_cast`, the alias attribute check, `c_call_shape`.
- `ember_codegen_c`: `ffi_callback_result`, the record's `result`, nullable data/entry/token
  (`emit_ffi_callback_data`, `emit_ffi_counted_wrapper`), `c_type` and friends for `Newtype`.
- `ember_analysis` (`cycles.rs`): `Newtype` is a leaf.
- `ember_driver` (`cimport.rs`): `callback_user_data_place`, `callable_of` (place, nullable
  `Option[…]`, `owned` where C keeps it), the overlay's `@ffi(newtype) type` entry; a function
  pointer's parameters and result keep their written typedef names (`function_type`); `.embind` 15.

### Defect found and fixed (for the lead to number)

- **A callback parameter's typedef'd type was read canonically**: `typedef int (*pair_fn)(intptr_t,
  intptr_t, int)` imported its parameters as `i64` where `[FFI-8]` maps `intptr_t` to `isize`
  (everywhere else it did), so a `callback(…, user_data=context)` whose `context` is `intptr_t` was
  `E5002` ("a `i64`, not the user data's `isize`"). `function_type` now reads the prototype's
  parameters as written.

## AG2. D-879 — one C++ header imported by two modules — fixed

The two imports of `stages.hpp` were two modules (their key was the importing folder and the path
as written), and the second adopted the first's `Stage` (`[FFI-30]`: `type Stage = <first>.Stage`)
but still wrote `extend Stage:` with every member, so each method was the class's twice (`E1030`).

- An import's key is now the files as found (the header, the overlays and include folders,
  canonicalised; a header only an include path finds stays keyed by its folder): one header
  imported the same way is one module, whichever spelling (this also stops a `.c` imported from two
  folders being compiled twice).
- A record an earlier import of the build wrote is adopted members and all
  (`Imported.adopted`, `adopted_member`): a later import writes none of its methods, constructors
  or operators. An entry of the later import's overlay that names only such members states nothing:
  `E5010`, naming the import whose entity it is.

Tests: `tests/conformance/FFI-30/accept_one_cpp_header_imported_by_two_modules.em` (`classes.hpp`,
`helpers/counted.em`, `overlays/classes.em`, `overlays/classes_doubled.em`: the same overlay from a
folder below is one module; another overlay is a second module sharing `Counter`),
`reject_an_entry_for_a_member_another_import_wrote.em` (`overlays/classes_counted.em`). `main`'s
compiler gives the accept test six `E1030`s.

## Spec questions (options + recommendation)

- **Q-AG1. `handle` on a result** (`[FFI-11]`, ODR-247 says it is a word of a result's contract):
  nothing says what `result(handle)` asserts, and it is `E0900`. Options: (a) the result is an
  opaque handle C owns, needing no lifetime word (no `from(…)`), returned as the raw pointer
  typed by its opaque type, never dereferenced by Ember; (b) the result's type must be a type
  declared `@ffi(handle)`, and the word only restates it; (c) drop `handle` from the result's words.
  Recommendation: (b), with `@ffi(handle) type T` defined as (a) for every pointer to `T`. `FFI-11`'s
  entry keeps this alone.
- **Q-AG2. What a newtype implements** (`[TYP-36]` has no row for it): built as `Clone` and `Copy`
  as its representation, and nothing else (no `Eq`, `Hash`, `Debug`, `Display`, `Ord`, arithmetic),
  so comparing two descriptors is `(a as c_int) == (b as c_int)`. Options: (a) as built; (b) `Eq`,
  `Hash` and `Debug` as the representation (a handle is compared and keyed), the rest not;
  (c) every interface as the representation, as range types. Recommendation: (b).
- **Q-AG3. `destroys(f)` with a pointer the call was lent** (ODR-246 says "the registering call's
  own [parameters]"): built for the callback, its user data, a scalar parameter and the result; a
  parameter the call was lent (`bus: ref mut Bus`) is not kept, since the token outlives the loan
  and the bus may be gone when it ends. Options: (a) as built; (b) keep it, the token borrowing the
  parameter (a region on `Retained`). Recommendation: (a).
