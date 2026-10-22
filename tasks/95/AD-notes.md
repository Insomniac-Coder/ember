# Helper AD — the language and std rules the owner has answered, 2026-10-23

Branch `worktree-agent-adbe030e8d426272c`, from `main` at `3922e110`. One commit per piece.
Baseline before any change: the whole conformance tree (every `tests/conformance/*`,
`tests/run-pass`, `compile-pass`, `compile-fail`, `run-fail`) failing 0.

## Done (tests)

* **ODR-254 `[STD-14]`, `[ERR-13]`** — the accessors' twins: `Mat2`/`Mat3`/`Mat4` `try_col(i)` and
  `try_row(i)` (`None` for a column or row the matrix does not have), `CppShared.try_get()`
  (`None` for an empty `std::shared_ptr`); `col`, `row` and `get` now name them in their panics
  (`; use .try_col(i) for an Option`), as plain text, so they stay `@noalloc`. The names follow
  std's twin habit for a method (`try_inverse`, `try_insert`). The other caller's bugs ODR-254
  lists (`stepped(0)`, `BitSet.insert`, `Entity.new`, `World.add`, `normalize`, `split`) are
  settled in `milestones.rs`'s list (`std_panics_name_their_twins_and_allocations_are_documented`),
  whose open list is now empty. `phase-status.json`: `STD-14` removed.
  * `STD-21/accept_a_matrix_column_or_row_has_a_non_panicking_twin.em` (three profiles),
    `STD-21/run_fail_a_matrix_column_past_the_matrix_names_its_twin.em`,
    `FFI-17a/accept_an_empty_shared_pointer_has_a_non_panicking_get.em` (`gauges.hpp`).

* **D-880 `[CLO-3]`, `[CLO-6]`, `[DRP-1]`** — helper AB's form (an owned callable value given to an
  `owned f: fn(A) -> R` parameter) drops on `main` as it stands (fixed before this branch). The
  form still leaking: an owned callable value given to a `once fn` parameter. The call consumes the
  callee (the MIR moves it in, `consumes_callee`), but its type is the plain `fn` the instance
  made, so both backends called its `native` entry and nothing ended it. The root: a call whose
  callee operand is moved ends the value whatever its type says, as a `once fn` call does: the C
  backend calls the descriptor's `once` entry (call, then drop), and the evaluator ends it after
  the call. The evaluator also ended a `once` call's state only when the closure's environment
  parameter was a reference, where C decides by the parameter being `owned` (a closure that moves
  a capture out takes its state whole); a closure reading captures from a copied environment kept
  its state un-dropped there. Both now agree.
  `CLO-3/accept_an_owned_callable_given_to_an_owned_parameter_is_dropped.em` (three profiles: a
  written `owned fn` parameter, one stored in a struct, a `once fn` parameter called and one never
  called, an `owned fn` lambda moved into a `once fn` parameter, and a named one lent to a generic
  parameter, dropped at its scope's end).

* **ODR-240 `[STD-19]`** — built by helper AA (`1792f2f3`, merged at `52bc34b6`; ADR-269) with the
  lead's mechanism (`lend_named_callable`: a named callable given to a parameter `owned f: F`,
  `F` bounded by a callable type, is passed `ref mut`; a lambda written at the call by value), and
  tested as the brief asks: `STD-19/accept_a_named_callable_is_lent_and_kept.em` (a named callable
  reused after `any` and `map`, counting callables read after their last use, every consumer,
  `retain`, `sort_by_key`, an adapter made of a lambda returned from a function) and
  `STD-19/reject_reading_what_a_lent_callable_changes_before_its_last_use.em` (`E3021`).
  `odr-pending.md`'s "NOT BUILT" for ODR-240 is out of date. What ODR-210 left that contradicted
  ODR-240 was std's documentation (`map`, `fold`, `for_each` said they take the callable over,
  citing ODR-210) and two tests' comments; rewritten. What I added, the std functions that took a
  callable and did not lend a named one:
  * the `Option` and `Result` methods (`map`, `and_then`, `filter`, `or_else`, `unwrap_or_else`,
    `ok_or_else`, `map_err`, `with_context`) and `Once.call_once` took `f: once fn(…)`, which
    `[CLO-6]` makes `owned`, so `o.map(inc)` moved `inc`: each is now `[Call: once fn(…)](…,
    owned f: Call)`, and `lend_named_callable` lends to a `once fn` bound too (a named callable
    that may be called again; one whose call gives a capture away is a `once` closure and is
    given, as a `once fn` value is). A parameter written as a callable type (`owned f: fn(A) ->
    R`, `f: once fn(A) -> R`) still takes over what it is given (ODR-189, `[CLO-7]`'s APIs).
  * `World.run` took `system: fn(mut Query[Q], A)` borrowed: a system that counts (`seen += 1`)
    was `E3023` in its instance. It is now `[S: fn(mut Query[Q], A)](…, owned system: S, …)`.
  * Not changed: `std.ecs.system(body, name)` (it keeps `body` in a `System`, an owned callable
    value; see `[ECS-4]` below) and `World.run_query` (its caller).
  Tests: `STD-19/accept_a_named_callable_is_lent_to_the_option_and_result_methods.em` (three
  profiles), `STD-19/reject_a_callable_whose_call_gives_a_capture_away_is_given_not_lent.em`
  (`E3040` for the `once` closure; the lent one passes twice),
  `ECS-4/accept_a_named_system_is_lent_to_run_and_kept.em` (three profiles).
  Note for the lead: a result whose type holds a view (`Result[int, str]`) is taken to borrow from
  every view argument, the lent `ref mut` callable among them, so `r.map(inc)` with an `E = str`
  keeps `inc`'s captures lent while the result lives (the elision rule, `[LT-1]`), as for any
  function given a reference.

* **ODR-218, ODR-255 `[THR-8]`, `[THR-9]`, `[CLO-10]`, `[ECS-4]`.**
  * `dyn I` is `Send` when `I` requires `Send` and `Sync` when it requires `Sync`, itself or
    through a parent (`TypeTable::interface_parents`, filled where an interface or an instance of a
    generic one is declared; `thread_properties` of `Dyn`). An implementation of such an
    interface by a type that is not `Send` was `E2040` already.
  * A type parameter is `Send` (`Sync`) when its bounds require it: every thread check of the
    checker goes through `thread_facts` (`TypeTable::thread_properties_with`, consulting the
    current generics' bounds). Before, `T: Send` was never `Send` inside its function (a thread
    capturing a `T: Send` was `E7004`, and `need_send[T](x)` forwarding it `E2040`).
  * An owned callable value whose type is its closure's (a lambda written or named) is `Send` when
    its captures are, structurally, as before; with the bounds above, a generic `F: fn() + Send`
    is too.
  * `[ECS-4]` — `run_parallel` runs its systems at once, on jobs of the pool (`jobs.scope()`).
    std.ecs reworked so nothing a job reaches is a class handle of the calling thread: a `Query`
    holds its entities as a raw view (`entities`, `count`: the driving storage's list, which its
    view holds, or the world's live list, which the run holds for a query requiring no component)
    instead of the world's handle; the `Commands` buffer is a `@sync class` holding
    `Mutex[Array[Box[dyn Change]]]`, `interface Change: Send` (ODR-255), its changes structs
    (`DestroyChange`, `RemoveChange` by the type's identity, `AddChange[T]`); a `System` keeps an
    erased `run_nested(world, rest, commands)`: each system's storages are taken and its query made
    in a frame of the calling thread, its function submitted as a job while the next systems are
    staged inside that frame's scope, so the scopes end innermost first; the `Commands` are applied
    in the systems' order once all have returned. `World.run` is unchanged in what it borrows (the
    world's live list only for a query requiring no component). Checks: `std.ecs.system(body,
    name)` (and the literal's systems, which the compiler makes with it) is `E7004` for a capture
    that is not `Send` and for a component its query reads or writes that is not `Send`
    (`check_ecs_system_sendable`); two systems of a `run_parallel` literal that both read a
    component that is not `Sync` are `E7005` naming both (`check_system_conflicts`); systems
    passed as values carry `Access.sync` and panic so before any starts. A system `run_parallel`
    runs that adds through its `Commands` a component that is not `Send` panics before it queues it
    (`AddChange[T]` is `Send` by hand; `World.run` adds any). New compiler-known
    `std.ecs.type_is_send[T]()`/`type_is_sync[T]()` (`Builtin::TypeIsSend`/`TypeIsSync`, a
    constant per instance, in both backends).
  * Tests: `THR-8/accept_a_dyn_of_an_interface_requiring_send_moves_to_a_thread.em`,
    `THR-8/reject_a_dyn_of_an_interface_not_requiring_send_or_sync.em` (`E7004`, `E7005`,
    `E2040`), `CLO-10/accept_an_owned_callable_value_is_send_when_its_captures_are.em`,
    `CLO-10/reject_an_owned_callable_value_capturing_what_is_not_send.em`,
    `ECS-4/accept_run_parallel_runs_its_systems_at_once_on_threads.em` (two systems that each wait
    up to ten seconds for the other to start see each other; commands applied in the systems'
    order; lambda systems capturing a `@sync` object), `ECS-4/reject_a_system_or_component_run_parallel_cannot_send.em`,
    `ECS-4/run_fail_systems_passed_as_values_share_a_component_that_is_not_sync.em`,
    `ECS-4/run_fail_a_parallel_system_adds_a_component_that_is_not_send.em`;
    `ECS-4/accept_run_parallel_runs_systems_whose_sets_do_not_conflict.em` changed: its two
    readers printed from two threads at once, which interleaves (below), so they record what they
    saw for `main` to print. `phase-status.json`: `ECS-4` removed; `THR-8`, `CLO-10` say what is
    left (question 1).

* **ODR-241 `[TYP-36]`, `[TYP-22]`, IX.1** — two boxes compare with `==` when what they hold does.
  `Box(3) == Box(3)` compared before, by reading through both boxes; `Box[T]` was no `Eq` type
  (`T: Eq` bounds, `Array[Box[int]]`, `Option[Box[int]]` and a recursive `@derive(Eq)` through a
  `Box` were `E2040`). Now `has_implicit_eq` gives a box `Eq` when `T` has it (a recursive type
  is the largest set, as the struct case already was), a held type's written `eq` is called
  (`needs_written_eq`, `synth_eq_of` read through the box), and both backends compare a box's
  payload (C's equality function, the evaluator's `equal`). `Box[dyn I]` when `I` requires `Eq`:
  `Eq` is no slot (as `Clone`, ODR-237: `dyn_methods_in_order`, `dyn_incompatibility_inner`);
  each type a table of such an interface is made for gets std's `dyn_eq[T]` instance
  (`core.em`: `a == b` as the type writes it; made in `dyn_concrete_adapter`, its symbol given
  to the backends through `TypeTable::declare_dyn_eq_symbol`), the table an `eq` entry after
  `access` (and `clone`), and two boxes compare their tables first (two kinds are never equal),
  then call it (C: `dyn_eq_header` / `dyn_clone_eq_header`; the evaluator: `invoke`). `==` on two
  `Box[dyn I]` no longer reads through the boxes (`synth_binary`). MIR reachability and std's
  body retention keep the `dyn_eq` instances a table holds. Effects: comparing a value holding a
  `Box[dyn I]` is every effect, as a call through a `dyn` value is (`holds_dyn_box`).
  Tests: `TYP-36/accept_two_boxes_compare_what_they_hold.em`,
  `TYP-22/accept_a_box_dyn_of_an_interface_requiring_eq_compares.em` (three profiles each, shown
  under development and release too, and with the C gate under cl and clang-cl),
  `TYP-22/reject_eq_where_an_interface_or_a_box_payload_is_not_eq.em`,
  `TYP-22/reject_noalloc_compares_dyn_boxes_through_their_table.em` (`E4001`).

* **ODR-215, ODR-213 `[MNG-1]`, `[MNG-3]`, `[MNG-6]`.** A built-in type's path is under `std.core`
  (`TypeTable::type_path`: a scalar, `str`, `cstr`, `void`, `Never`), so `str.chars` is
  `em_3std4core3str5chars` and `i64.of` `em_3std4core3i642of`; a scalar's spelling as a generic
  argument stays its keyword (`[MNG-6]`). The root module's own `int`, `float` or `str`, which the
  checker keys `root.<name>` (D-860), has the name as its path (`item_path`), the root package
  having no component of its own (`em_3str13is_alphabetic`; it was `em_4root3str…`). `[MNG-3]`:
  `transliterated` writes an ASCII `_` that begins what would read as an escape (`_u`, four or
  more hex digits, `_`) as `_u005F_`, a component it changes is marked `u`, and C member names and
  locals (`c_identifier`) are spelled the same: `caf_u00E9_` is `em_u16caf_u005F_u00E9_` and the C
  member `caf_u005F_u00E9_`; `demangle` reads it back. Every existing program's names: only those
  holding a built-in type's methods or an escape-looking ASCII name change; the whole conformance
  tree passes before and after (behaviour unchanged).
  Tests: `MNG-1/accept_a_programs_own_str_and_the_built_in_one_are_two_symbols.em`,
  `MNG-3/accept_a_field_beyond_ascii_and_its_ascii_spelling_are_two_c_members.em` (one struct
  holding `café` and `caf_u00E9_`, a C member declared twice before), the existing
  `MNG-3/accept_a_transliterated_name_and_its_ascii_spelling_are_two_symbols.em` re-recorded to
  the rule, `ember_branding` `a_name_beyond_ascii_is_transliterated_before_its_length`,
  `ember_types` `the_canonical_spelling_of_instance_arguments` (names `[MNG-6]`, and the
  built-in paths); `milestones.rs` `inline_header_functions_and_inline_attributes_reach_the_c`
  names `std.core.str.chars`. `phase-status.json`: `MNG-1`, `MNG-3` removed.

* **D-876 `[CLO-1]`, `[OWN-3]`** — a lambda with no result type written whose body is a value read
  through a reference that only a copy could give by value gives the reference
  (`check_lambda_body`): `fn(w: ref String) => w` is `fn(ref String) -> ref String`, as with the
  type written; a `Copy` value is still given by value. It moved the `String` out (`E3013`).
  `CLO-1/accept_a_lambda_giving_a_borrowed_parameter_gives_its_reference.em` (three profiles).
* **D-877 `[EFF-1]`** — a call through a `plain fn` value (a function, a lambda capturing nothing)
  runs one of the functions the program uses as values (`Const::Fn`) of its shape (parameter count,
  result type), so its effects are theirs (`plain_function_values`, the `Indirect` call sites of
  the effect graph): a stored generator expression, an unfused adapter chain and a later-`for`
  generator holding capture-free lambdas run in `@noalloc`. One allocating lambda of the same shape
  anywhere makes the shared adapter instance's call allocate, which is right: the instance runs
  whichever it holds (an instance per lambda would be more precise and cost code size; not built).
  `EFF-1/accept_a_call_through_a_plain_fn_value_has_the_effects_of_its_functions.em` (three
  profiles), `EFF-1/reject_a_plain_fn_value_of_a_shape_some_function_allocates_in.em`.

* **D-878 `[TYP-24]`, `[BLD-13]`** — an implementation got its interface's default methods module by
  module (`register_defaults` at the end of each module's `collect_methods`), so one in a module
  the list holds after the interface's had none: `std.ffi`'s `extend CppError implements Error`
  had no `source` once `std.ffi` came after `std.core`. The defaults are now given once every
  module's implementations are collected (a pass of its own after `collect_methods`).
  `TYP-24/accept_a_default_method_reaches_an_implementation_in_a_later_module.em` (three
  profiles; support `TYP-24/order/named.em`, `order/circle.em`, imported in the order that puts
  the interface's module first; `E2040` and `E2072` before).
* **D-881 — not a compiler defect; a question.** `{name: name.char_count() for _, name in pairs}`
  with `name: str` is `E2020` because `[TYP-38]` makes a map written with no type own its text
  (its key type `String`) and `[TXT-9]` never converts a `str` value implicitly; a map literal
  `{name: 1}` is the same error, and both written forms work (`sizes: Map[str, int] = {…}`, or
  `{name.to_string(): …}`). The diagnostic is the defect's real part: it names `Map.insert`'s
  parameter in `std/src/collections.em`, the comprehension's desugaring, and does not offer the
  map's type. Options: (a) as the spec reads, a better `E2020` at the key (help: `.to_string()`, or
  name the type `Map[str, V]`; no std location); (b) a key whose type is `str` (not a literal)
  makes `Map[str, V]`, as a list comprehension's elements stay `str`, only a literal defaulting to
  `String`; (c) convert implicitly, copying (contradicts `[TXT-9]`). Recommend (a). Not built.

* **`[GRM-28]`, `[CLO-12]` — generic local functions.** `fn ident[T](x: T) -> T:` in a block
  (`check_generic_local_fn`): its type parameters follow the enclosing function's own; its body
  is checked once with them opaque (`[TYP-17]`, the check that reports) as the named closure it
  is, which gives its environment (what it captures, `[CLO-2]`; an empty one if nothing) and the
  value its name holds. A call (`synth_generic_local_call`) is a generic call of a signature with
  the environment first, its type arguments inferred or written (`ident[int](9)`, `[TYP-18]`), the
  enclosing function's being what they are where the call is (themselves in its opaque check, its
  instance's arguments in an instance: a generic local in a generic function is made for each of
  that function's instances). Each new set of type arguments makes its instance's body
  (`generic_local_instance`): the closure's body checked with them concrete, capturing what the
  declaration's closure does through the one environment, its diagnostics (only an instance's own)
  noting `in \`f\` used with \`T\` = …`; it may call itself (with the same arguments) and a closure
  in the block calls it through its capture. Its name read anywhere but as a callee is `E2060`,
  as a module-level generic's is. Tests:
  `GRM-28/accept_a_generic_local_function_is_made_for_each_use.em` (three profiles: inferred and
  written arguments, capturing reads and writes from two instances, a bound, recursion, a closure
  calling it, one inside a generic function), `GRM-28/reject_a_generic_local_function_used_as_a_value.em`
  (`E2060` three ways, an opaque-body `E2040`, a missed bound `E2040`);
  `GRM-28/reject_the_local_function_forms_not_built.em` keeps the local generator.
  Not built: a call of itself inside its body is checked against its own signature (as a local
  function's reserved one is), so one with other type arguments (`deep(n - 1, 5)` in
  `deep[T](n: int, label: T)` called with a `str`), which a module-level generic allows, is
  `E2020`. Building it: the self-call made as a generic call with the body's environment parameter
  first, which the opaque check, still discovering the environment, does not have.
* **`[CLO-2]` — found on the way: a capture shadows an item of its name.** In a closure, a local of
  the enclosing function read as a value was the module's item of its name when there was one
  (`twice = 5` then `fn() => twice + 1` was `E2020`, `+` on `fn(i64) -> i64`), though called it was
  the capture; the name's read now tries the capture before the items. A generic local function
  named as a module's function, called from a closure, needed it.
  `CLO-2/accept_a_capture_shadows_an_item_of_its_name.em` (three profiles).
  `CLI-19/reject_an_unimplemented_construct_by_name.em` named the generic local function as its
  unbuilt construct; it now names the local generator.
* **`[GRM-28]` local generators — not built** (still `E0900`, named). The generators are written
  out before the checker as module-level declarations (ADR-193, `coroutines.rs`): a frame struct
  holding the parameters, its `Iterator` implementation, the body as a method of the frame. A
  local one captures enclosing locals, whose types and needed access (`ref`, `ref mut`) only the
  checker learns, at the declaration. In every form the declaration is checked as a closure (its
  `yield`s as the transform's calls) to find the captures and their access, as `[CLO-2]` decides
  them, and the frame holds a reference to that environment (`ref` when it only reads, `ref mut`
  when it writes, so the local is borrowed while a generator of it lives); what differs is how the
  lifted declaration learns the environment's type:
  (a) lifted generic over it, `gen fn count__local[E](owned env: E, n: int)`, each call passing
  `ref count` or `ref mut count` (`E` a reference type, as ODR-240 lends a callable), its body
  checked for that one instance with free names read through `env` as a closure's captures are:
  needs a generator whose body is not checked opaquely (one is today: `yield x + i` with `x: T` is
  `E2040`), only quietly per instance, the declaration's check being the one that reports;
  (b) lifted with a placeholder environment struct the desugaring declares empty and the checker
  fills at the declaration (fields, `Copy`), the parameter's mode and the frame's field settled
  then, as `settle_coroutine_frame` settles a frame's `ref T` fields: a type changed after
  collection;
  (c) the frame, its `Iterator` implementation and the body method made by the checker at the
  declaration: a type, an implementation and methods registered after collection.
  Recommend (a): no type changes after collection, and the result carrying the borrow of what it
  is given is the path ODR-240's adapters already take. A design of its own (an ADR), so left for
  the owner.

## Not done

* A local generator (`[GRM-28]`, above): `E0900`; three designs, (a) recommended.
* A generic local function calling itself with other type arguments (above): `E2020`.
* `[THR-8]`, `[CLO-10]`: a value of a written callable type (`fn(A) -> R`) is never `Send`
  (question 1); the entries in `phase-status.json` say so.
* D-881: the diagnostic option (a) is not built (a question: which option).

## Compiler changes (besides each item's own above)

* `ember_typeck`: `lend_named_callable` lends to a `once fn` bound; thread facts through the
  current generics' bounds (`thread_facts`); `check_ecs_system_sendable`, `E7005` in
  `check_system_conflicts`; `Box` `Eq` and `Box[dyn I]` `eq` table entries (`dyn_eq_instances`);
  D-876 in `check_lambda_body`; D-878's defaults after every module's `collect_methods`; generic
  local functions (`check_generic_local_fn`, `synth_generic_local_call`, `generic_local_instance`,
  `generic_local_named`, `generic_local_value`).
* `ember_types`: interface parents and `Dyn`'s thread properties, `thread_properties_with`;
  `dyn_eq` symbols; built-in types' paths under `std.core`.
* `ember_hir`/`ember_mir`/`ember_analysis`: `Builtin::TypeIsSend`/`TypeIsSync` (pure); the
  `dyn_eq` instances reachable; a comparison holding a `Box[dyn I]` has every effect; D-877's
  `plain_function_values`.
* `ember_codegen_c`, `ember_interp`: a call moving its callee in ends it (D-880);
  `TypeIsSend`/`TypeIsSync` constants; box and `Box[dyn I]` equality.
* `ember_branding`: the `_u005F_` escape of an escape-looking ASCII `_`.
* `ember_driver`: std bodies kept for the `dyn_eq` instances.
* std: `math` (`try_col`, `try_row`), `ffi` (`CppShared.try_get`), `core` (the `Option`/`Result`
  methods' lent callables, `dyn_eq`), `sync` (`Once.call_once`), `ecs` (the `run_parallel`
  rework).

## Defects found (not fixed)

* **The evaluator reads a null storage when two lambda systems capturing one `@sync` object run
  in one `run_parallel`.** `ECS-4/accept_run_parallel_runs_its_systems_at_once_on_threads.em`
  with its last two lines written
  `w.run_parallel([counted, system(owned fn(mut q: Query[Pos]): tally.total.fetch_add(2), "two")])`
  and `println(tally.total.load(), w.has[Mark](a))` (stdout `7 true`): the C program is right;
  the evaluator panics `invalid downcast` in `std.ecs.aos_storage` (`world.storages[key]` reads
  a null handle while `contains_key(key)` is true) at the program's *first* `run_parallel`
  (`[left, right]`, which runs before the lambdas exist); without that first run it panics at the
  last one, in `Read[Vel].fetch`. The same with the second lambda over `Query[Mark]`. With one
  lambda system in that run, or the second capturing nothing (the test as committed), both agree.
  So something static in the program changes the evaluator's run of code that runs earlier: not
  a duplicate symbol (the MIR has none), not the callable descriptors (keyed by body and
  environment type). Not found.
* Two threads' `println`s with several arguments interleave their pieces (each argument is
  written on its own): the ECS-4 test records what its systems saw for `main` to print.
* D-877's precision: one adapter instance runs every lambda of its shape, so one that allocates
  makes every call of the instance allocate (an instance per lambda: code size; not built).

## Questions for the owner

1. `[THR-8]`, `[CLO-10]` — a value of a written callable type (`fn(A) -> R`: a field, an
   annotation, a result) hides what it captured, so it is never `Send` and cannot move to a
   thread. Options: (a) `fn(A) -> R + Send`, the bound form a generic already writes (`F: fn() +
   Send`), checked where a closure becomes one; (b) leave it (only a closure's own type, or a
   generic, is `Send`). Recommend (a).
2. ODR-255 — "a component two systems read must be `Sync`, one a system writes `Send`
   (`E7004`)": built as `E7004` for not `Send` and `E7005` for not `Sync`, `E7005` being the
   thread checks' code for `Sync` elsewhere. Is the `E7004` the ruling's for both?
3. ODR-218 — a component a system `run_parallel` runs adds through its `Commands` must be
   `Send`; `Commands.add[T]` serves `World.run` too, where any component may be added, so a
   parallel system's adding one that is not panics before it is queued (tested), rather than a
   static bound. Options: (a) as built; (b) a `T: Send` bound on `add` everywhere (refuses
   `World.run` adding a class handle); (c) a parallel system's query hands out a `Commands` type
   of its own whose `add` requires `Send` (static; a second type in the API). Recommend (c) if the
   static check is wanted, else (a).
4. D-881 — options (a)/(b)/(c) above; recommend (a).
