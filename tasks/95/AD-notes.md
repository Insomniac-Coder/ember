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
