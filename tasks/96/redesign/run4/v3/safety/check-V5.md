# Check V5 of writer W5 (R12, function values)

Checker V5, safety validation (b). Inputs: `inputs/brief.md`, `V3/rules.md`, `V3/vocabulary.md`,
`V3/inputs/JOB.md`, `S/matrix-W5.md` and the 96 files in `V3/conformance/R12/` (95 tests and the module
`widgets.em`), with the answers they start from (AN-05, AN-53 to AN-61 in `V3/area-D.md`, AN-75, AN-76,
AN-78 in `V3/area-E.md`). Nothing was compiled or run. "line N" with no file means `V3/rules.md` line N.
Today's runner is `compiler/ember_driver/tests/milestones.rs`: a diagnostic no annotation claims fails the
test (`assert_exact_diagnostics`, lines 334-421), and each `#$ help:` must be found in the `= help:` lines
(`assert_diagnostic_helps`, lines 5404-5426). Every message the writer calls "word for word" was matched
against `rules.md` and the area files with whitespace normalised (`target/redesign5/V5/msgs2.py`); all
those claims hold.

Totals: 96 files (81 right, 12 wrong, 3 unsettled); 26 matrix rows (19 right, 6 wrong, 1 unsettled).

## The tests

### Accepted programs (40)

- `accept_a_button_keeps_captured_and_log_handlers.em`: **right**. Lines 527-528, 535 (`LOG` beside `captured`), 564-569 (`main` wrote the lambdas, names only `LOG`); output `1 1`.
- `accept_a_button_of_another_module_keeps_a_log_handler_under_captured.em`: **right**. Verdict 40, lines 527-534: `widgets` cannot see `LOG`, so `captured` covers it there; `main` names `LOG` (a module variable, not its own local). Output `1`.
- `accept_a_dyn_iterator_that_says_changes_log.em`: **right**. Verdict 46, lines 618-623. Output `1`, `2`, `2` (inspect runs before each item is yielded).
- `accept_a_handler_called_by_position_adds_one_and_removes_itself.em`: **right**. AN-60's NEW; lines 637-647, line 646-647 (the holder is not an argument). Traced: `add_once` pushes `count`, removes itself (`current` -1 then 0), `count` runs once: `1 1`.
- `accept_a_lambda_that_only_reads_a_captured_variable_is_a_plain_fn.em`: **right**. Line 512 (a plain `fn()` changes nothing), 537-542. Output `0`.
- `accept_a_lambda_that_says_it_pushes_to_log.em`: **right**. Lines 519-522; `click` is `main`'s own local, so the call changes a place `main` may change (line 641-643). Output `1`.
- `accept_a_local_function_names_the_mutex_it_changes.em`: **right**. Verdicts 29, 30, 51; lines 509-512, 570-576. Output `1`.
- `accept_a_local_list_names_the_variable_its_functions_change.em`: **right**. AN-58's NEW, verdict 40. Output `1`.
- `accept_a_plain_function_fits_a_list_of_log_handlers.em`: **right**. Lines 522-523, 537-541. Output `1`.
- `accept_a_programs_own_function_takes_its_function_as_a_template_parameter.em`: **right**. Verdicts 38, 50; lines 543-552; `C` filled from `main`'s own `total`/`seen` (line 286-287: no clause). Output `8`, `2`.
- `accept_a_ref_mut_name_from_get_mut_calls_the_handler.em`: **right**. Lines 382-385; AN-61's two NEW forms; vocabulary `get_mut` row. Output `2`.
- `accept_a_two_variable_handler_in_a_captured_dots_list.em`: **right**. Verdicts 22, 35. Output `1 2`.
- `accept_a_weakref_handler_says_changes_ui.em`: **right**. AN-60's `watch`/`UiButton`; lines 507-509, 583-588; R11 line 450 (a weakref from any name). `main` names `Ui` as AN-75 makes `main` name `Log` (area-E.md lines 1069-1070). Output `1`.
- `accept_add_line_changes_a_guarded_field_through_a_plain_parameter.em`: **right**. Verdict 44's own line; R4 lines 212-216. Output `1`.
- `accept_an_arrow_lambda_says_what_it_changes_after_its_parameters.em`: **right**. Verdict 45, lines 520-521. Traced `2 1`.
- `accept_call_once_holds_an_owns_once_fn_and_call_twice_hands_over_copies.em`: **right**. AN-59's NEW, word for word; line 644-646 refuses only a function value that changes its own state given to a read-only parameter, so `call_twice(f: fn())` may call `f`. Output `a`, `b`, `b`.
- `accept_clear_the_handlers_after_the_click.em`: **right**. The line-832 message's way out. Output `1 0`.
- `accept_click_says_changes_handlers_and_walks_ref_mut.em`: **right**. Lines 641-644, 635-636. Output `1`, `2`.
- `accept_clicking_after_the_loop_over_the_list_a_handler_changes.em`: **right**. R8 (c): no call in the walk; lines 626-628. Output `a`, `2`.
- `accept_clicking_plain_handlers_inside_a_loop_over_log.em`: **right**. Lines 541-542, verdict 17. Output `a`, `click`, `b`, `click`.
- `accept_each_name_takes_a_change_list.em`: **unsettled**. AN-56's NEW, word for word, passes a read-only lambda `fn(s) => println(s.len())` where the type is `fn(owns String) changes C`; AN-56 part 5 accepts it ("as if the function had taken it and dropped it"), but section 4 line 915 gives E2020 for "a lambda parameter whose mode differs from the expected function type", both directions, and line 861's message covers only the other direction. The rules do not say a plain lambda parameter fits an `owns` one.
- `accept_for_each_after_the_loop_over_log.em`: **right**. Lines 546-548. Output `1`, `3`.
- `accept_for_each_given_a_lambda_that_says_it_changes_log.em`: **right**. Verdicts 21, 38. Output `2`.
- `accept_inspect_keeps_the_function_and_count_runs_it.em`: **right**. Lines 583-588, 612-615; reading `seen` between `inspect` and `count()` is allowed (lines 553-556 restrict only views across the call; area-D.md line 43, row 6). Output `0`, `3`.
- `accept_main_names_results_for_the_threads_it_starts.em`: **right**. AN-76's NEW, verdict 30; `RESULTS` is `guarded by` a static lock (no E7012). Output `1 1`, `2`.
- `accept_press_says_captured_and_game_names_nothing.em`: **right**. Lines 564-569 ("itself or through a call whose first line says it"). Output `1`.
- `accept_read_only_handlers_walked_with_for.em`: **right**. AN-60 part 6 (area-D.md lines 1180-1183); R8 (f) lines 382-385 let `h(this)` reach `h`'s place. Output `3`, `3`.
- `accept_report_clicks_after_the_loop_over_log.em`: **right**. Verdicts 17, 48; G-06. Traced `start`, `start`, `clicked`.
- `accept_retain_then_push_the_notes.em`: **right**. The lines line 832's message gives. Output `2`.
- `accept_row_6_handler_changes_the_real_score.em`: **right**. AN-53's NEW. Output `1`.
- `accept_run_takes_the_stateful_callback_ref_mut.em`: **right**. AN-54; lines 635-636, 641-646. Output `ann 1`, `ann 2`, `0`.
- `accept_the_function_value_called_after_the_with_block.em`: **unsettled**. R8 (c) is met, but `dump(log, fn() with thread_shared log changes Log: ...)` (line 27) gives one call a read-only `Log` argument and a function value that "counts as changing every place its type names" (line 235, R12 lines 556-558): line 232-241 refuses an argument that "may reach into what the other owns" (E3021) and makes no exception for a `Log` whose changed fields are `guarded by` its lock. AN-75 (area-E.md line 1074) writes the same call without discussing it.
- `accept_the_handler_owns_a_share_of_a_counter.em`: **right**. The program line 864's message gives; verdict 32; `main` names `Counter` because `click` says it. Output `1`.
- `accept_the_handler_takes_a_copy_of_the_loop_name.em`: **right**. Lines 635-636. Output `ann`, `bo`.
- `accept_the_stored_function_called_after_the_loop_as_many_times.em`: **right**. The program AN-58's E3020 message leads to. Output `1`, `2`, `4`.
- `accept_tick_changes_world.em`: **right**. Verdict 41, lines 607-611; `main` changes both names (through `tick` and `note`). Output `1 2`.
- `accept_tick_names_what_the_kept_callback_changes.em`: **right**. Verdicts 30, 31, 39; AN-78's shape (`return register_cb(...)` gives the `Retained` token, area-E.md lines 1748-1749) with `threads(main)` so the callback may push to `LOG`.
- `accept_two_threads_push_to_one_log_and_say_changes_log.em`: **right**. AN-75's NEW; `with thread_shared log` from a plain `thread_shared` parameter (vocabulary `thread_shared` row, "AN-75's `start`"). Output `2`.
- `accept_update_takes_a_death_handler_type_with_a_change_list.em`: **right**. Verdict 49, AN-05. Traced: `kills` 2, `es` [1] (the second `update` removes indexes 2 then 0): `2 1`.
- `accept_views_used_before_the_call_that_changes_their_place.em`: **right**. Lines 553-556: only views that live across the call are restricted; R2 (`ref` before a number part). Output `0`, `11`.

### Refused programs (51)

- `reject_a_counting_handler_given_to_a_plain_death_handler.em`: **right**. E2113 (lines 537-539); AN-05's message word for word; the lines it gives compile (the accept test).
- `reject_a_for_walk_of_handlers_given_the_ui_ref_mut.em`: **right**. R8 (c) through a `ref mut this` argument; AN-60 part 6's E3020 message with this file's lines (14, 15).
- `reject_a_for_walk_of_handlers_whose_type_changes_ui.em`: **wrong**. The E3020 is right (line 836's third message, lines 14 and 15 correct), but `main` calls `ui.fire()`, whose first line says `changes handlers, Ui`, and `main` says nothing: R5 lines 282-285 give a second diagnostic (E2105) on `fn main():`, which the runner reports as unexpected. The writer's own `accept_a_weakref_handler_says_changes_ui.em` and AN-75 (area-E.md lines 1069-1070) name the type in `main` for its own object. Fix: `fn main() changes Ui:`.
- `reject_a_function_that_gives_s_away_passed_to_call_twice.em`: **wrong** (help). E3013 and line 865's message are right, but `#$ help: fn call_once(owns f: once fn()):` names a function this program does not have; the message asks for the parameter of `call_twice`. Fix: `#$ help: fn(): consume(copy of s)` (or `owns f: once fn()`).
- `reject_a_function_that_says_changes_world_but_never_changes_frame.em`: **right**. Line 610's message in this program's names.
- `reject_a_function_value_that_changes_log_called_in_a_walk_of_log_lines.em`: **unsettled**. The E3020 on line 22 is line 837's message (lines 20, 22 correct) and line 837 settles E3020 over E7018 inside a loop; but its line 27 is the same `dump(log, fn() ... changes Log ...)` call as the accept test above, which line 232-241 may refuse (E3021), giving a second diagnostic.
- `reject_a_handler_kept_past_the_loop_name_it_uses.em`: **right**. Line 651 (E3026); the loop name ends with its round (R2 line 106); AN-57 part 6's shape; the line it gives is the accept test.
- `reject_a_handler_stored_in_the_button_it_changes.em`: **wrong** (help). E3022 and line 832's message are right; `#$ help: call \`b.handlers.clear()\` after \`b.click()\`` is not text the message gives (it says "`b.click()`, then `b.handlers.clear()`"). Fix: `#$ help: \`b.click()\`, then \`b.handlers.clear()\``.
- `reject_a_handler_that_outlives_the_callers_total.em`: **wrong**. E3026 and line 864's message are right, but `main` (which did not write the lambda; `register` did) runs it through `b.click()`, whose first line says `captured`: lines 564-569 then require `captured` on `main`'s first line, a second E2105 the runner reports. Fix: `fn main() changes captured:`. (Following line 864's help, `main` would also need `changes Counter`, which the message does not name; see `for_owner`.)
- `reject_a_lambda_that_names_a_variable_it_never_changes.em`: **right**. Lines 519-520 ("both ways"); the writer's message follows R18.
- `reject_a_lambda_that_pushes_to_log_without_saying_so.em`: **right**. Line 819, word for word.
- `reject_a_local_function_that_changes_a_mutex_without_naming_it.em`: **right**. Verdicts 29, 51; message follows R18.
- `reject_a_local_list_of_plain_functions_given_a_lambda_that_changes_score.em`: **right**. AN-58 part 6, word for word.
- `reject_a_log_handler_in_a_button_that_allows_only_captured.em`: **wrong** (message). E2113 is right (lines 527-530: a visible `LOG` is written by its name). The message (AN-53 part 6, word for word) gives the list's type and `on_click`'s line but not `click`'s, which must then say `changes handlers, captured, LOG` (AN-53 itself, area-D.md lines 318-320, refuses it otherwise): R18 (line 759-760) wants every line it replaces named. Fix: end the message with "and `fn click() changes handlers, captured, LOG:`".
- `reject_a_log_handler_in_a_list_of_plain_functions.em`: **right**. Line 860, word for word; the line it gives compiles here.
- `reject_a_once_fn_called_twice.em`: **wrong** (message). E3040 fits (line 648; section 4 line 945). The message's line `fn call_twice(f: fn()):` renames `call_once`, so `call_once(...)` on line 17 no longer resolves, and "give it one that hands over a copy" does not name line 17, whose `fn() with owns s` would then be E3013: R18 (lines 759-760). Fix: "call it once (remove line 10); or, to run it twice, write `fn call_once(f: fn()):` and on line 17 `call_once(fn(): consume(copy of s))`"; help to match.
- `reject_a_plain_loop_name_calls_a_stored_function.em`: **right**. Lines 641-644; AN-58 part 6, word for word.
- `reject_a_plain_name_from_get_mut_calls_the_handler.em`: **right**. AN-61 part 6 and the vocabulary `get_mut` row, word for word.
- `reject_a_plain_once_fn_parameter.em`: **right**. Line 865's third message; the help is the full line it gives.
- `reject_a_read_only_view_across_a_call_that_changes_what_it_reads.em`: **right**. R8 (g), E3021 at the change (a number write is no reshape). The message gives one way out; R8 (g)'s second (`ref mut`) would not compile here (see `for_owner`).
- `reject_a_ref_mut_name_across_a_call_that_changes_its_place.em`: **right**. R8 (f), E3022.
- `reject_a_stateful_callback_given_to_a_read_only_parameter.em`: **right**. Line 866, word for word.
- `reject_a_stored_function_that_pushes_to_the_walked_list_called_in_the_loop.em`: **right**. AN-58 part 6's E3020 with lines 11, 12, 14.
- `reject_a_thread_function_that_pushes_under_the_log_lock_without_changes_log.em`: **right**. Line 819's lock message, word for word.
- `reject_a_two_variable_handler_in_a_captured_list.em`: **right**. AN-53 part 6, word for word (fuller than line 860's, and R18-complete).
- `reject_a_ui_changing_handler_in_a_list_of_read_only_handlers.em`: **wrong** (message). E2113 is right, but neither way out the message gives (AN-60 part 6, word for word) compiles in this program: `handlers: Array[fn(Ui) changes Ui]` makes `fire` (`changes handlers`) E2105 and its `for` walk E3020 (line 836); `fn(ref mut Ui)` needs the lambda, `fire`'s walk and first line changed too. R18 line 759. Fix: give the by-position lines (`handlers: Array[fn(ref mut Ui)]`, `bump = fn(ref mut seen: Ui): seen.clicks += 1`, and line 836's `while` form with `fn fire() changes this:`).
- `reject_a_weakref_handler_that_changes_the_ui_without_changes_ui.em`: **right**. AN-60 part 6, word for word.
- `reject_add_line_changes_a_guarded_field_without_changes_log.em`: **right**. Verdict 44.
- `reject_an_arrow_lambda_that_changes_removed_without_saying_so.em`: **right**. Verdict 45.
- `reject_an_inspect_adapter_that_changes_log_made_a_dyn_iterator_that_says_nothing.em`: **right**. Line 860's third message, line 16 correct.
- `reject_an_inspect_lambda_without_changes_seen.em`: **right**. AN-55 part 6, word for word.
- `reject_call_twice_given_a_lambda_that_hands_over_the_real_s.em`: **wrong** (message). E3013 fits; the message is AN-59 part 6's word for word, but its first way out, `fn() with owns s: consume(s)` given to `call_twice`, is itself refused (E3013, AN-59's first refusal), and "a function that `main` keeps" is untrue (`call_twice` is given it): R18 lines 759-761. Fix: "`consume(s)` would hand over `main`'s `s` from inside a function `call_twice` may run twice, and the function does not own `s`; hand over a copy each time: `call_twice(fn(): consume(copy of s))`".
- `reject_captured_on_a_lambdas_own_first_line.em`: **right**. Line 820, word for word.
- `reject_click_says_captured_for_a_list_of_captured_dots.em`: **right**. Verdict 35.
- `reject_click_walks_its_handlers_without_changes_handlers.em`: **right**. Line 866's E2105, one message for two mistakes (R18).
- `reject_click_without_captured.em`: **right**. Line 819, word for word.
- `reject_click_without_log.em`: **right**. Line 819, word for word.
- `reject_clicking_a_button_of_another_module_inside_a_loop_over_log.em`: **wrong** (help). E3020 is right (verdict 40); `#$ help: call \`b.click()\` after the \`for\` loop` is not text the message gives ("click after the loop"). Fix: `#$ help: click after the loop`.
- `reject_clicking_a_log_button_inside_a_loop_over_log.em`: **wrong** (help). E3020 and line 836's message (lines 21, 23) are right; the help is the same unmatched phrase. Fix as above.
- `reject_clicking_inside_a_loop_over_a_list_a_handler_changes.em`: **wrong** (help). E3020 is right (lines 626-628, AN-53 part 6); same help fix.
- `reject_each_name_without_a_change_list_given_a_changing_lambda.em`: **wrong** (help). E2113 and AN-56's message are right; the help drops the message's backticks (`[C] above fn each_name(...)`), so it is no text the diagnostic gives. Fix: `#$ help: fn each_name(owns f: fn(owns String) changes C) changes C:`.
- `reject_for_each_given_a_lambda_that_does_not_say_it_changes_log.em`: **right**. Line 819's third message in this program's names.
- `reject_for_each_inside_a_loop_over_log.em`: **right**. Line 837's first message, lines 16, 18.
- `reject_inspect_count_inside_a_loop_over_seen.em`: **right**. AN-55 part 6, lines 11, 13.
- `reject_main_starts_a_thread_that_changes_results_without_naming_it.em`: **right**. Line 821, line 28 correct.
- `reject_press_runs_handlers_without_captured.em`: **right**. Lines 564-569.
- `reject_retain_given_a_function_that_changes_the_list.em`: **right**. Line 832, word for word.
- `reject_row_6_handler_without_changes_score.em`: **right**. Line 819, word for word.
- `reject_setup_says_changes_log_for_a_handler_it_only_stores.em`: **right**. Lines 583-588, both ways.
- `reject_the_handler_name_used_again_after_the_call.em`: **right**. R8 (a), (f) lines 384-385; AN-61's E3060 with lines 15-17.
- `reject_tick_calls_poll_events_without_naming_what_the_kept_callback_changes.em`: **right**. Lines 598-601, word for word.

### Run tests and the module (5)

- `run_a_handler_that_grows_the_list_during_dispatch_keeps_its_state.em`: **right**. Lines 637-638; traced `1`, `2`, `16`, `0`; ASan as AN-61.
- `run_a_handler_that_removes_itself_then_uses_its_state.em`: **right**. Lines 639-640; traced `once`, `8`, `8`.
- `run_a_named_copy_is_the_handlers_own_state.em`: **right**. AN-53's `game_copy`. Output `1`, `2`, `0`.
- `run_a_real_variable_is_seen_by_the_handler_and_back.em`: **right**. Row 6, area-D.md line 43. Output `1`, `11`.
- `widgets.em`: **right**. A `parse-pass` helper module, as today's `DIA-12/support.em`.

## The matrix rows

- R12-1: **right**. Missing too: a named local function (verdict 51) that changes `LOG` without a clause.
- R12-2: **right** (CLO-2/CLO-12 superseded correctly). Missing: two lambdas sharing one real variable.
- R12-3: **right**.
- R12-4: **right**. Missing: a `=>` lambda with a capture list and a clause.
- R12-5: **wrong** (today's tests). `CLO-5/accept_a_closure_keeps_a_captured_object_alive.em` returns a lambda that captures a local class handle: a lifetime case (E3026 or a box, R12-21/R12-22, the writer's `for_owner` item 1), not a change through a link. Move it there. Missing: a change through a captured `sharedref mut` or `shares mut` link alone.
- R12-6: **right**. `EFF-21/accept_nolock_without_a_lock.em` is only loosely related. Missing: `changes Atomic[int]`, `changes RwLock[T]`.
- R12-7: **right** (CLO-13 superseded correctly).
- R12-8: **right**. Line 860's own message names one line only; the tests use AN-53's fuller one.
- R12-9: **wrong**. Its E2113 "word for word" message (AN-53) omits `click`'s line (see the test above).
- R12-10: **right**. Missing: two packages, and a library `pub fn wait_frame() changes captured...:`.
- R12-11: **wrong**. `reject_a_ui_changing_handler...`'s message gives lines that do not compile here (above). Missing: a function value stored by assigning a field, and one returned.
- R12-12: **unsettled**: `accept_each_name_takes_a_change_list.em` (E2020, line 915).
- R12-13: **right**. Missing: a `virtual` method that runs stored handlers.
- R12-14: **wrong**. `reject_a_for_walk_of_handlers_whose_type_changes_ui.em` lacks `changes Ui` on `main`; the walk-under-lock pair is unsettled (line 232-241). Missing: a position across such a call (E3060), a generator paused across it.
- R12-15: **right**. Missing: a real variable given beside a function value that changes it (`xs.retain(fn(x) changes xs => ...)`).
- R12-16: **right** (test's help aside). Missing: the store by field assignment (AN-62's `t.on_visit`).
- R12-17: **right**.
- R12-18: **right**.
- R12-19: **right**. Missing: a change list naming a type, and one used in a function type.
- R12-20: **right**. Missing: a `dyn` with no `changes` given an adapter whose function changes nothing (accepted).
- R12-21: **wrong**. `reject_a_handler_that_outlives_the_callers_total.em` needs `changes captured` on `main` (lines 564-569).
- R12-22: **right**; the box and state placements of a real variable stay open (writer's `for_owner` item 1).
- R12-23: **right**.
- R12-24: **right**.
- R12-25: **right**. Missing: a field holder (`changes on_visit`).
- R12-26: **wrong**. Two of its messages break R18 (`reject_a_once_fn_called_twice.em`, `reject_call_twice_given_a_lambda_that_hands_over_the_real_s.em`) and one help names a missing function. Missing: a `once fn` kept in a field and taken out.
