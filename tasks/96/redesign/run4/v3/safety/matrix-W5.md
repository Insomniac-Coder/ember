# Safety matrix, writer W5: R12 (function values)

Writer W5 of safety validation (b). Rule: R12, `V3/rules.md` lines 504-652, with the section 3 refusals
and the section 2 run-time rows that belong to it. Verdicts from `V3/inputs/JOB.md`. Tests:
`V3/conformance/R12/` (95 tests: 51 `reject_`, 40 `accept_`, 4 `run_`, and `widgets.em`, a module two
of them import). Nothing was compiled or run (brief, "Machine limits"). `V3` = `tasks/96/redesign/run4/v3/`.

## How to read this

- One section per guarantee, `R12-1` .. `R12-26`, with the seven fields of the brief.
- **today's tests** are `tests/conformance/<RULE>/<file>.em`. Several of them expect the opposite of
  the new rule; those are marked **(superseded)**.
- **enforced by** names today's source file that does this kind of work now. Short names used below:
  - `typeck`: `compiler/ember_typeck/src/lib.rs` (the type checker; captures are resolved at
    `resolve_capture`, about line 25452, and checked from about line 49099).
  - `effects`: `compiler/ember_analysis/src/effects.rs` (today's effect sets, `[EFF-1]`..`[EFF-6]`; it
    says itself that a call through a function value "contributes every effect: no function type
    declares its effects yet").
  - `borrows`: `compiler/ember_analysis/src/borrows.rs` (the borrow check, the MIR pass that today
    checks borrows and regions; it emits E3020, E3021, E3022, E3060; captures from about line 2257).
  - `stores`: `compiler/ember_analysis/src/stores.rs` (emits E3026 today).
  - `drops`: `compiler/ember_analysis/src/drops.rs` (emits E3013 today).
  - `parser`: `compiler/ember_parser/src/body.rs` and `decls.rs`.
  - `codegen`: `compiler/ember_codegen_c/src/lib.rs` and `compiler/ember_codegen_llvm/src/closures.rs`
    (today's function value is `ember_fn_value`, `{native, descriptor, w[3]}`, 40 bytes).
  - `runtime`: `runtime/ember_rt/src/ember_rt.c`.
  - `interface`: `compiler/ember_build/src/interface.rs` (a package's interface file, R17).
  E2105 and E2113 are not in today's compiler (`grep` over `compiler/` finds neither): the check is new
  work, most naturally beside `effects`.
- **How a test is verified once built**: a `compile-fail` test by the runner's expected code and
  message on the marked line (and the `#$ help:` line); `run-pass` by `#$ stdout:`; the four `run_`
  tests, and the `accept_` tests that dispatch handlers, also by a build of the emitted C under
  AddressSanitizer with no report, as `V3/area-D.md` AN-60, AN-61 and AN-62 checked the same shapes in C.
- Where I put an E2105 for a named function, the marker is on its first line (the line the message asks
  to change). The rules do not say on which line E2105 is reported (`for_owner` item 9).

---

### R12-1. A plain `fn()` changes nothing at all; a lambda names the module variables it changes

| field | value |
|---|---|
| guarantee | A lambda whose body changes a module variable says so on its first line (`fn() changes LOG:`), so no function value hides a change of `LOG` behind a type that says it changes nothing. |
| source | R12, rules.md lines 505-506, 512, 519-522; verdict 17; section 3 line 819 (E2105) |
| enforced by | type checker (`typeck`), with the change sets of `effects`; parser for `changes` after a lambda's parameters |
| today's tests | `EFF-1/accept_a_call_through_a_plain_fn_value_has_the_effects_of_its_functions.em` (effects of a call through a function value are those of the functions of its shape: the opposite approach, inferred, not written) |
| missing edge cases | a lambda that changes a module variable only through a call it makes (`fn(): tick()`), covered by R5's "the calls it makes included" but not tested here; a lambda inside a lambda |
| tests | `reject_a_lambda_that_pushes_to_log_without_saying_so.em`, `accept_a_lambda_that_says_it_pushes_to_log.em` |
| expected result | reject: compile-fail E2105 "this function pushes to `LOG`; say so on its first line: `fn() changes LOG: LOG.push(String("clicked"))`" (rules.md line 819, word for word); accept: run-pass, stdout `1` |

### R12-2. A lambda names the real variables it captures and changes; the change reaches the real variable

| field | value |
|---|---|
| guarantee | A capture with no word is the real variable (row 6); a lambda that changes it names it (`fn() changes score:`), and the change is seen by the function that made it, and a change by that function is seen by the lambda. |
| source | R12 lines 505-507, 624-628; row 6; verdict 22; `V3/area-D.md` line 43 ("a write in the lambda is seen by the function, and back"); AN-53; section 3 line 819 |
| enforced by | type checker (`typeck`, capture resolution) for the clause; borrow check (`borrows`) for what the place that holds the lambda reaches |
| today's tests | `CLO-2/accept_mutating_a_captured_local.em` (a capture written in place); `CLO-2/reject_outer_write_while_mutating_capture_is_live.em` and `CLO-12/reject_a_capture_written_while_the_local_function_lives.em` **(superseded)**: today a write by the maker while the lambda lives is E3021, under row 6 it is seen by the lambda |
| missing edge cases | a real variable that is a list read by the lambda while the maker reshapes it between calls (the lambda holds no view across calls, so nothing to refuse, but untested); two lambdas sharing one real variable |
| tests | `reject_row_6_handler_without_changes_score.em`, `accept_row_6_handler_changes_the_real_score.em`, `run_a_real_variable_is_seen_by_the_handler_and_back.em` |
| expected result | reject: compile-fail E2105 "this function changes `score`, a variable it uses from `game`; say so on its first line: `b.on_click(fn() changes score: score += 1)`" (line 819); accept: run-pass `1`; run: run-pass `1`, `11`, ASan clean |

### R12-3. A lambda's `changes` is checked both ways

| field | value |
|---|---|
| guarantee | A lambda that names a variable it never changes is refused, so `changes` on a lambda is never a guess. |
| source | R12 lines 519-520 ("checked against its body both ways (E2105)"); section 4 E2105 |
| enforced by | type checker (`typeck`) with `effects` |
| today's tests | none |
| missing edge cases | a lambda that names a module variable it never changes; one that names `Ui` and never changes a `Ui` |
| tests | `reject_a_lambda_that_names_a_variable_it_never_changes.em`, `accept_a_lambda_that_only_reads_a_captured_variable_is_a_plain_fn.em` |
| expected result | reject: compile-fail E2105 "this function says `changes score` and never changes it; remove `score` from its first line: `show = fn(): println(score)`" (my wording, in the shape of AN-76 part 6's `tick` message; the rules give none for a lambda); accept: run-pass `0` |

### R12-4. A lambda written with `=>` says what it changes after its parameters

| field | value |
|---|---|
| guarantee | `fn(e) changes removed => keep(e, ref mut removed)`: the `=>` form takes the clause, so a short lambda cannot hide a change. |
| source | R12 lines 520-521; verdict 45 |
| enforced by | parser (`changes` before `=>`); type checker (`typeck`) for the check |
| today's tests | none (today's lambdas have no clause) |
| missing edge cases | a `=>` lambda with a capture list and a clause (`fn(e) with copy of k changes removed => ...`) |
| tests | `reject_an_arrow_lambda_that_changes_removed_without_saying_so.em`, `accept_an_arrow_lambda_says_what_it_changes_after_its_parameters.em` |
| expected result | reject: compile-fail E2105 "this function changes `removed`, a variable it uses from `update`; say so on its first line: `enemies.retain(fn(e) changes removed => keep(e, ref mut removed))`"; accept: run-pass `2 1` |

### R12-5. Changes through a link are named by the type of what changes

| field | value |
|---|---|
| guarantee | A function that changes an object through a link it holds (`with weakref ui`, `shares mut`, `sharedref mut`, `thread_shared`) names the object's type (`changes Ui`). |
| source | R12 lines 507-509; verdict 17; AN-60 part 6 (E2105) |
| enforced by | type checker (`typeck`), `effects` |
| today's tests | `CLO-5/accept_a_closure_keeps_a_captured_object_alive.em` (a captured object, no clause today) |
| missing edge cases | a change through a captured `sharedref mut` link; through a `shares mut` capture (tested only inside R12-21's accept) |
| tests | `reject_a_weakref_handler_that_changes_the_ui_without_changes_ui.em`, `accept_a_weakref_handler_says_changes_ui.em` |
| expected result | reject: compile-fail E2105 "this function changes `me`, a `Ui` reached through its `weakref ui`; say so on its first line: `fn() with weakref ui changes Ui:`" (AN-60, word for word); accept: run-pass `1` |

### R12-6. Changes under a lock, in a `Mutex`, are named like every change (verdicts 29, 44)

| field | value |
|---|---|
| guarantee | A function, a lambda or a local function that changes a `guarded by` field under its lock, or the value in a `Mutex`, names it by type (`changes Log`, `changes Mutex[Stats]`); a `guarded by` field is changed through a plain name inside its lock with no word at the call. |
| source | R12 lines 509-512; R4 lines 212-216; R5 lines 300-308; verdicts 29, 44, 51; AN-75; section 3 line 819 |
| enforced by | type checker (`typeck`), `effects`; the lock rules themselves are R15's (W6) |
| today's tests | `EFF-21/accept_nolock_without_a_lock.em`, `EFF-21/reject_a_lock_under_nolock.em` (a lock is an effect today); `THR-10/accept_threads_spawn_join_and_detach.em` (a `Mutex` changed from threads, nothing named) |
| missing edge cases | `changes RwLock[T]`, `changes Atomic[int]` (for_owner item 8: by name or by type for a captured `Atomic`), `changes Sender[int]` / `Receiver[int]` (verdicts 36, 37: W6's channels) |
| tests | `reject_a_thread_function_that_pushes_under_the_log_lock_without_changes_log.em`, `accept_two_threads_push_to_one_log_and_say_changes_log.em`, `reject_add_line_changes_a_guarded_field_without_changes_log.em`, `accept_add_line_changes_a_guarded_field_through_a_plain_parameter.em`, `reject_a_local_function_that_changes_a_mutex_without_naming_it.em`, `accept_a_local_function_names_the_mutex_it_changes.em` |
| expected result | rejects: compile-fail E2105, the first word for word from rules.md line 819 / AN-75, the other two in its shape; accepts: run-pass `2`, `1`, `1` (each reads the count under the lock after the joins) |

### R12-7. What a function captured with a word is its own state, and changing it needs no word

| field | value |
|---|---|
| guarantee | A named capture (`copy of`, `owns`, `shares`, `shares mut`, `thread_shared`, `weakref`) is the function value's own state; a function that changes only its own state is a plain `fn()`, and the maker's variable is untouched. |
| source | R12 lines 512-514, 635-636; row 6; AN-53 (`game_copy`), AN-54 |
| enforced by | type checker (`typeck`) for the type; code generator (`codegen`) for the state block |
| today's tests | `CLO-2/accept_owned_closure_mutates_its_moved_capture.em`, `CLO-10/accept_an_owned_callable_value_keeps_its_state.em`, `CLO-13/accept_a_lambda_copies_what_it_only_reads.em` (today's silent copy for a `Copy` read: **superseded**, a read is now of the real variable) |
| missing edge cases | `with shares x` and `with weakref x` state ending when the function value ends (counts given back): a run test under ASan with leak detection |
| tests | `run_a_named_copy_is_the_handlers_own_state.em`, `accept_run_takes_the_stateful_callback_ref_mut.em`, `accept_the_handler_takes_a_copy_of_the_loop_name.em` |
| expected result | run-pass `1`, `2`, `0`; run-pass `ann 1`, `ann 2`, `0`; run-pass `ann`, `bo`; ASan clean |

### R12-8. `captured` and `captured...` in the type of stored functions; a lambda never writes `captured`

| field | value |
|---|---|
| guarantee | A field, list or parameter that keeps functions covers the variables they captured with `captured` (each changes one) or `captured...` (several); a function changing two variables does not fit `captured`; a lambda names its own variables and never writes `captured`. |
| source | R12 lines 524-536, 537-540; verdicts 22, 35; AN-53 part 6; section 3 lines 820 (E2105), 860 (E2113) |
| enforced by | parser (`captured`, `captured...` in a function type); type checker (`typeck`) |
| today's tests | none |
| missing edge cases | a parameter (not a field) of type `fn() changes captured` given a two-variable lambda; `captured...` beside a module variable (`Array[fn() changes captured..., LOG]`) |
| tests | `reject_captured_on_a_lambdas_own_first_line.em`, `reject_a_two_variable_handler_in_a_captured_list.em`, `accept_a_two_variable_handler_in_a_captured_dots_list.em` (and `accept_row_6_handler_changes_the_real_score.em`) |
| expected result | compile-fail E2105 (line 820, word for word); compile-fail E2113 (AN-53's message, word for word, which names all three lines it replaces); run-pass `1 2` |

### R12-9. A name visible where a function type is written is written by its name

| field | value |
|---|---|
| guarantee | `captured` covers only what the line cannot see: a local list in the function that owns `score` says `Array[fn() changes score]`, and a `Button` in the module of `LOG` says `changes captured, LOG`; a `fn() changes LOG` does not fit a plain `captured` list there. |
| source | R12 lines 527-530; verdict 40; AN-58; AN-53 part 6 |
| enforced by | type checker (`typeck`, name resolution of the change list) |
| today's tests | `CLO-4/reject_a_borrowing_lambda_stored_in_a_list.em` **(superseded)**: a lambda kept in a local list that ends first is now accepted |
| missing edge cases | a local list naming a variable, given to a function outside (it would have to leave the function: E3026) |
| tests | `reject_a_local_list_of_plain_functions_given_a_lambda_that_changes_score.em`, `accept_a_local_list_names_the_variable_its_functions_change.em`, `reject_a_log_handler_in_a_button_that_allows_only_captured.em`, `accept_a_button_keeps_captured_and_log_handlers.em` |
| expected result | compile-fail E2113 (AN-58's message, word for word); run-pass `1`; compile-fail E2113 (AN-53's message, word for word; for_owner item 6); run-pass `1 1` |

### R12-10. What `captured` stands for is filled in from the whole program

| field | value |
|---|---|
| guarantee | A module (or library) that cannot see a program's `LOG` covers it with `captured`; a `b.click()` that runs a program's `LOG` handler is then a change of `LOG` in that program, refused inside a loop over `LOG`. |
| source | R12 lines 530-534; verdict 40; R17 lines 750-751 (the function values a package stores into a `captured` type, recorded in its interface file) |
| enforced by | type checker (`typeck`) over the whole program; across packages the interface file (`interface`) |
| today's tests | `MOD-2/accept_a_pub_method_and_an_interface_method_anywhere.em` (only that a module's method is callable) |
| missing edge cases | the same across a package edge (two packages; today's runner has no package fixture in `tests/conformance/`); a library `pub fn wait_frame() changes captured...:` filled from a program's kept C callbacks |
| tests | `widgets.em` (module), `accept_a_button_of_another_module_keeps_a_log_handler_under_captured.em`, `reject_clicking_a_button_of_another_module_inside_a_loop_over_log.em` |
| expected result | run-pass `1`; compile-fail E3020 (my wording in the shape of rules.md line 836). Read with for_owner item 7 |

### R12-11. A function value goes only where its type allows; one that changes less fits

| field | value |
|---|---|
| guarantee | A named function is a value of the type its first line gives; a function value that may change more than the place, list or parameter allows is refused, and one that changes less fits; so calling what an `Array[fn()]` holds inside a loop over `LOG` is safe. |
| source | R12 lines 522-523, 537-542; verdict 17; AN-60 part 6 (G-05); AN-76 part 6; section 3 line 860 (E2113) |
| enforced by | type checker (`typeck`) |
| today's tests | `CLO-1/reject_another_lambda_in_a_place_holding_one.em`, `CLO-14/reject_a_callable_argument_whose_signature_misses_the_bound.em` (shape, not changes) |
| missing edge cases | a function value assigned to a field (`t.on_visit = prune`, AN-62) rather than pushed; a function value returned where the result type says less |
| tests | `reject_a_log_handler_in_a_list_of_plain_functions.em`, `accept_a_plain_function_fits_a_list_of_log_handlers.em`, `accept_clicking_plain_handlers_inside_a_loop_over_log.em`, `reject_a_ui_changing_handler_in_a_list_of_read_only_handlers.em`, `accept_read_only_handlers_walked_with_for.em` |
| expected result | compile-fail E2113 (line 860, word for word); run-pass `1`; run-pass `a`, `click`, `b`, `click`; compile-fail E2113 (AN-60, word for word); run-pass `3`, `3` |

### R12-12. A change list: a function that runs a function it is given makes its changes

| field | value |
|---|---|
| guarantee | `[C, F: fn(Item) changes C]` (or a parameter `fn(...) changes C`) takes a function that changes things if it says so, and the call counts as making those changes; a function type without a change list takes only functions that change nothing; a program's own function may write the bound or the function value. |
| source | R12 lines 543-552; verdicts 21, 38, 49, 50; AN-05, AN-56; section 3 lines 819, 860 |
| enforced by | type checker (`typeck`, generic instances); `effects` per instance (today's `[EFF-2]` already gives a callable parameter the passed function's effects) |
| today's tests | `EFF-2/accept_a_callable_parameter_has_the_passed_functions_effects.em`, `EFF-2/reject_a_callable_parameter_given_an_allocating_function.em`, `CLO-14/accept_an_explicit_callable_bound_is_called_like_a_function.em`, `CLO-14/accept_a_callable_type_alias_as_a_bound.em`, `STD-19/accept_a_named_callable_is_lent_and_kept.em` |
| missing edge cases | one instance per function given (verdict 38's speed): no test reads the emitted C (an `#$ assert-c-count:` on `apply_each`'s instances, once the new C names are known); `changes T` where `T` is also a parameter's type (R12 line 551) |
| tests | `reject_for_each_given_a_lambda_that_does_not_say_it_changes_log.em`, `accept_for_each_given_a_lambda_that_says_it_changes_log.em`, `reject_each_name_without_a_change_list_given_a_changing_lambda.em`, `accept_each_name_takes_a_change_list.em`, `reject_a_counting_handler_given_to_a_plain_death_handler.em`, `accept_update_takes_a_death_handler_type_with_a_change_list.em`, `accept_a_programs_own_function_takes_its_function_as_a_template_parameter.em` |
| expected result | compile-fail E2105 (line 819's shape, in this program's names: for_owner item 4a); run-pass `2`; compile-fail E2113 (AN-56, word for word); run-pass `3`, `1`; compile-fail E2113 (AN-05, word for word); run-pass `2 1`; run-pass `8`, `2` |

### R12-13. A call through a function value makes the changes its type says; the caller names them

| field | value |
|---|---|
| guarantee | A function that runs function values names what their type says (`changes LOG`), or the word their type uses (`captured`, `captured...`), itself or through a call whose first line says it; the function that wrote the lambdas names nothing for their captured variables. |
| source | R12 lines 553-556, 563-569; R5 lines 282-288; verdict 35; AN-53, AN-76; section 3 line 819 |
| enforced by | type checker (`typeck`) with `effects` (E2105) |
| today's tests | `EFF-1/reject_an_effect_through_callees_and_drops.em` (effects through callees, inferred today) |
| missing edge cases | a `virtual` method that runs stored handlers (R5's one-way check) |
| tests | `reject_click_without_log.em`, `accept_report_clicks_after_the_loop_over_log.em`, `reject_click_without_captured.em`, `reject_click_says_captured_for_a_list_of_captured_dots.em`, `reject_press_runs_handlers_without_captured.em`, `accept_press_says_captured_and_game_names_nothing.em` |
| expected result | compile-fail E2105 (line 819, word for word, twice); run-pass `start`, `start`, `clicked`; compile-fail E2105 (my wording, twice); run-pass `1` |

### R12-14. Across a call through a function value: no walk, no read-only view, no `ref mut` name of what it changes

| field | value |
|---|---|
| guarantee | A call that may run a function value is a change of everything its type names and every real variable it changes: a list it changes is not walked across the call (E3020), a read-only view of what it changes does not live across it (E3021), nor a `ref mut` name of such a place (E3022); a walk of a list under its lock does not call a function value that changes that list's type. |
| source | R12 lines 553-556, 626-628; R8 (c), (f), (g) (rules.md lines 357-365, 376-385, 386-397); verdict 29 (its gain); AN-53, AN-58, AN-60, AN-75, AN-76; section 3 lines 816, 834, 836, 837 |
| enforced by | borrow check (`borrows`), reading the callee's function type instead of today's "unknown call" |
| today's tests | `BRW-1/reject_mutating_a_viewed_container.em`, `CTL-1/accept_an_adapters_items_borrow_what_it_reads.em`; `CLO-15/reject_what_an_adapter_holds_is_still_borrowed_or_moved.em` (a lambda's borrow held while an adapter lives: **superseded**, the borrow is now at the call that drives it) |
| missing edge cases | a position (`boss = enemies[0]`) across a call through a function value that changes `enemies` (E3060 at the later use, R8 (a)); a generator paused in a list across such a call; a `sharedref mut` holder |
| tests | `reject_clicking_a_log_button_inside_a_loop_over_log.em`, `reject_for_each_inside_a_loop_over_log.em`, `accept_for_each_after_the_loop_over_log.em`, `reject_a_stored_function_that_pushes_to_the_walked_list_called_in_the_loop.em`, `accept_the_stored_function_called_after_the_loop_as_many_times.em`, `reject_clicking_inside_a_loop_over_a_list_a_handler_changes.em`, `accept_clicking_after_the_loop_over_the_list_a_handler_changes.em`, `reject_a_read_only_view_across_a_call_that_changes_what_it_reads.em`, `reject_a_ref_mut_name_across_a_call_that_changes_its_place.em`, `accept_views_used_before_the_call_that_changes_their_place.em`, `reject_a_function_value_that_changes_log_called_in_a_walk_of_log_lines.em`, `accept_the_function_value_called_after_the_with_block.em`, `reject_a_for_walk_of_handlers_given_the_ui_ref_mut.em`, `reject_a_for_walk_of_handlers_whose_type_changes_ui.em`, `accept_a_handler_called_by_position_adds_one_and_removes_itself.em` |
| expected result | compile-fail E3020 (line 836 and 837, AN-58, AN-60, word for word in this program's lines; one in my wording); compile-fail E3021 and E3022 (my wording, in the shape of lines 816 and 834); run-passes `1`,`3` / `1`,`2`,`4` / `a`,`2` / `0`,`11` / `a`,`2` / `1 1`. The walk-under-lock pair reads with for_owner item 2 |

### R12-15. A function value given to a call counts as an argument that changes what its type names

| field | value |
|---|---|
| guarantee | `LOG.retain(keep)`, where `keep` changes `LOG`, names one place twice and is refused; the lambda that changes nothing overlaps nothing. |
| source | R12 lines 556-559; R4 lines 232-241; section 3 line 832 (E3022) |
| enforced by | borrow check (`borrows`, the two-place overlap of a call) |
| today's tests | `BRW-3/accept_two_phase_borrows_across_conditional_arguments.em` (overlap of arguments, no function value) |
| missing edge cases | a function value that captures the real variable (not a module variable) given beside it: `xs.retain(fn(x) changes xs => ...)` |
| tests | `reject_retain_given_a_function_that_changes_the_list.em`, `accept_retain_then_push_the_notes.em` |
| expected result | compile-fail E3022 (line 832, word for word); run-pass `2` |

### R12-16. A function value is not stored in a place that a real variable it changes owns

| field | value |
|---|---|
| guarantee | A handler that changes `b` is not kept in `b.handlers`, so no function changes its own holder while it runs (Rust refuses the same). |
| source | R12 lines 560-563; AN-53 part 6, AN-62 part 6; section 3 line 832 (E3022) |
| enforced by | borrow check (`borrows`) at the store, with R4's overlap of `b` and the argument |
| today's tests | none |
| missing edge cases | the store through a field assignment (`t.on_visit = fn(n: Node) changes t: ...`, AN-62), all the way down (`b.inner.handlers`) |
| tests | `reject_a_handler_stored_in_the_button_it_changes.em`, `accept_clear_the_handlers_after_the_click.em` |
| expected result | compile-fail E3022 (line 832, word for word); run-pass `1 0` |

### R12-17. Running a function on its own counts as making its changes; storing it does not

| field | value |
|---|---|
| guarantee | `thread.spawn`, `scope.spawn` and a `callback(retained)` / `callback(once)` parameter count as making what the function changes, now or later, and the caller names it; storing a function value (a field, a list, an adapter) changes nothing, and a function that only stores one names nothing. |
| source | R12 lines 570-588; R15 lines 693-697; verdicts 30, 52; G-06; AN-76; section 3 line 821 (E2105) |
| enforced by | type checker (`typeck`) with `effects`; the library's first lines (`std/src/thread.em`) |
| today's tests | `THR-10/accept_threads_spawn_join_and_detach.em`, `CLO-7/accept_spawn_takes_its_callback_over.em` (nothing named today) |
| missing edge cases | `scope.spawn` inside a loop over what the task changes (E3020, AN-55) and the join inside a lock (E7018, verdict 52): W6's threads |
| tests | `reject_main_starts_a_thread_that_changes_results_without_naming_it.em`, `accept_main_names_results_for_the_threads_it_starts.em`, `reject_setup_says_changes_log_for_a_handler_it_only_stores.em`, `accept_report_clicks_after_the_loop_over_log.em` (its `setup`), `accept_inspect_keeps_the_function_and_count_runs_it.em` |
| expected result | compile-fail E2105 (line 821, in this program's lines); run-pass `1 1`, `2`; compile-fail E2105 (my wording); run-pass as R12-13; run-pass `0`, `3` |

### R12-18. A foreign call may run the callbacks C keeps; its caller names their changes

| field | value |
|---|---|
| guarantee | Every foreign call not marked `@ffi(no_callbacks)` counts as possibly running every callback C keeps, so a function that makes one names what they change, as every function that calls it does. |
| source | R12 lines 589-606; verdicts 31, 39; AN-78 |
| enforced by | type checker (`typeck`, which reads today's `callback(...)` contracts) with `effects`; the trampoline in `compiler/ember_codegen_c/src/lib.rs` and `compiler/ember_codegen_llvm/src/ffi.rs` |
| today's tests | `FFI-21/accept_callbacks_c_keeps_past_the_call.em`, `FFI-21/reject_what_a_callback_captures.em` |
| missing edge cases | the walk across a C call (E3020), `@ffi(no_callbacks)` and E5069 are W6's (verdicts 31, 39 in W6's duty; W6 writes them in `V3/conformance/R15/`, among them `reject_a_c_call_inside_a_walk_a_kept_callback_may_change.em` and `accept_a_c_function_marked_no_callbacks_inside_a_walk.em`); a library's `pub fn wait_frame() changes captured...:` |
| tests | `reject_tick_calls_poll_events_without_naming_what_the_kept_callback_changes.em`, `accept_tick_names_what_the_kept_callback_changes.em` |
| expected result | compile-fail E2105 (rules.md lines 598-601, word for word); compile-pass. Both use a `threads(main)` callback that pushes to `LOG` (for_owner item 4b) |

### R12-19. A named change list is strict

| field | value |
|---|---|
| guarantee | `world: changelist = (FRAME, LOG)` stands for its names; a function says `changes world` only if it changes everything in it. |
| source | R12 lines 607-611; verdict 41 |
| enforced by | parser (`changelist`); type checker (`typeck`) |
| today's tests | none |
| missing edge cases | a change list that names a type (`Game`) and one used in a function type (`fn() changes world`) |
| tests | `reject_a_function_that_says_changes_world_but_never_changes_frame.em`, `accept_tick_changes_world.em` |
| expected result | compile-fail E2105 (rules.md line 610, in this program's names); run-pass `1 2` |

### R12-20. A type that keeps a function carries its change list; a `dyn` type says what it changes

| field | value |
|---|---|
| guarantee | An adapter (`inspect`) keeps the function and its change list, so driving it (`count()`, a `for`) is the call that makes the changes; a `dyn` type with no `changes` changes nothing, and a value whose methods change more is refused. |
| source | R12 lines 612-623; verdicts 21, 46; AN-55; section 3 line 860 (E2113) |
| enforced by | type checker (`typeck`, generic instances and `dyn` coercion) |
| today's tests | `CLO-15/accept_a_lambda_at_an_adapter_keeps_its_borrows.em`, `STD-19/accept_the_closure_adapters.em`, `CTL-3b/accept_closure_adapters_are_stages_of_a_counted_loop.em` |
| missing edge cases | `map` and `filter` adapters; an adapter kept in a struct field and driven later in a loop over what it changes |
| tests | `reject_an_inspect_lambda_without_changes_seen.em`, `reject_inspect_count_inside_a_loop_over_seen.em`, `accept_inspect_keeps_the_function_and_count_runs_it.em`, `reject_an_inspect_adapter_that_changes_log_made_a_dyn_iterator_that_says_nothing.em`, `accept_a_dyn_iterator_that_says_changes_log.em` |
| expected result | compile-fail E2105 (AN-55, word for word); compile-fail E3020 (AN-55, in this program's lines); run-pass `0`, `3`; compile-fail E2113 (line 860, in this program's lines); run-pass `1`, `2`, `2` |

### R12-21. A function value may not outlive a place it uses and does not own

| field | value |
|---|---|
| guarantee | A handler that changes the caller's `total`, or that uses a loop name, and is kept by an object that outlives it, is refused at the lambda's line; a handler that owns a share, or a copy, outlives nothing. |
| source | R12 lines 624-627, 651-652; AN-57; section 3 line 864 (E3026) |
| enforced by | `stores` (E3026 today), reading where each function value goes in the function that wrote it |
| today's tests | `CLO-3/reject_a_borrowing_lambda_stored_as_a_value.em`, `CLO-7/reject_a_callback_that_borrows.em`, `TYP-15/reject_a_closure_storing_a_parameter_in_a_capture.em` (all E3026 today, reported inside the callee) |
| missing edge cases | a function value that captures a real local of its maker and leaves the maker (for_owner item 1); a lambda capturing an opened name (`exists as p`) kept past the block |
| tests | `reject_a_handler_that_outlives_the_callers_total.em`, `accept_the_handler_owns_a_share_of_a_counter.em`, `reject_a_handler_kept_past_the_loop_name_it_uses.em`, `accept_the_handler_takes_a_copy_of_the_loop_name.em` |
| expected result | compile-fail E3026 (line 864, word for word); run-pass `1`; compile-fail E3026 (my wording, the shape AN-57 part 6 gives); run-pass `ann`, `bo` |

### R12-22. Where a captured real variable lives never changes what the program does (section 2, line 783)

| field | value |
|---|---|
| guarantee | A real variable lives at its address when the function value provably ends first, else in the function value's state, else in a box; the choice is made when compiling and the program prints the same either way. |
| source | R12 lines 628-634; section 2 line 783 (`nothing / nothing beyond the function value's own block / one box per variable`; W4: 1.091x, 1.255x, safe Rust 1.358x) |
| enforced by | code generator (`codegen`) for the placement; `runtime` for the box's count |
| today's tests | `CLO-10/accept_wide_captured_state_is_held_inline.em`, `CLO-10/accept_small_captured_state_in_noalloc_code.em` (today's state layout) |
| missing edge cases | the state and box placements of a **real** variable: both arise only when the function value may outlive its maker, which R12 line 626 refuses (E3026) while lines 631-634 and AN-53's counter factory assume it; no test until the owner rules (for_owner item 1) |
| tests | `run_a_real_variable_is_seen_by_the_handler_and_back.em` (address), `run_a_named_copy_is_the_handlers_own_state.em` (state of a named capture) |
| expected result | run-pass `1`, `11`; run-pass `1`, `2`, `0`; each also under AddressSanitizer with leak detection: no report |

### R12-23. A stored function value is two words, copied out before the call (section 2, line 784)

| field | value |
|---|---|
| guarantee | A call through a list element or a field copies `{code, state}` out before the arguments are given and keeps no pointer into the list, so a handler may grow or shrink the list it is in while it runs, and nothing is written back. |
| source | R12 lines 637-638, 646-647; R8 (f) lines 382-385; section 2 line 784; AN-60, AN-61 (today's write-back through a stale pointer, `c/an61_dispatch.c` mode 1, heap-use-after-free) |
| enforced by | code generator (`codegen`, the call of a stored function value); borrow check (`borrows`) for R8 (f)'s exception |
| today's tests | `CLO-11/accept_a_callable_field_is_called.em`; none tests a handler that moves its own list's buffer (AN-61's unsound programs are accepted today) |
| missing edge cases | a field (not a list element) that a handler replaces while it runs (`this.on_visit = other` inside `on_visit`) |
| tests | `run_a_handler_that_grows_the_list_during_dispatch_keeps_its_state.em`, `accept_a_handler_called_by_position_adds_one_and_removes_itself.em`, `accept_a_ref_mut_name_from_get_mut_calls_the_handler.em` |
| expected result | run-pass `1`, `2`, `16`, `0` under AddressSanitizer: no report (the run-time proof); run-pass `1 1`; run-pass `2` |

### R12-24. A handler that removes itself and then uses its state holds one count on it (section 2, line 785)

| field | value |
|---|---|
| guarantee | A body that may end its own state (it removes itself) and uses that state afterwards holds one count on it across that call, so the state is not freed under it. |
| source | R12 lines 639-640; section 2 line 785; AN-60 part 5 (`c/an61_dispatch.c` mode 2 clean, mode 3 without the count a heap-use-after-free) |
| enforced by | code generator (`codegen`) with the `runtime` count on the state block |
| today's tests | none |
| missing edge cases | a handler that removes itself through a call several levels down (the count is decided from callees' first lines and summaries) |
| tests | `run_a_handler_that_removes_itself_then_uses_its_state.em` |
| expected result | run-pass `once`, `8`, `8` under AddressSanitizer: no report |

### R12-25. A call through a stored function value is a change of its holder

| field | value |
|---|---|
| guarantee | A call may change the function's own state, so the holder is reached through a name that may change it (`changes handlers`, `for ref mut h`, `h = ref mut ...`), and a function value that changes its own state is not given to a parameter that only reads; a later use of the `ref mut` name is checked against what the call may have done. |
| source | R12 lines 641-647; R8 (a), (f); verdict 42; AN-54, AN-58, AN-61; section 3 line 866 (E2105, E3023) |
| enforced by | type checker (`typeck`, E3023 and E2105); borrow check (`borrows`, E3060) |
| today's tests | `CLO-2/reject_a_callable_value_called_through_a_shared_reference.em` (E3021 today, help `.iter_mut()`), `CLO-2/reject_mutating_capture_through_borrowed_callable_parameter.em`, `CLO-2/accept_mutating_capture_through_mut_callable_parameter.em`, `CLO-2/reject_calling_an_owned_callable_value_from_a_borrowed_place.em` |
| missing edge cases | a read-only parameter given a function that changes nothing of its own state (allowed: `call_twice` in R12-26's accept); a field holder (`fn depth_total() changes on_visit`, AN-62) |
| tests | `reject_click_walks_its_handlers_without_changes_handlers.em`, `accept_click_says_changes_handlers_and_walks_ref_mut.em`, `reject_a_plain_loop_name_calls_a_stored_function.em`, `reject_a_plain_name_from_get_mut_calls_the_handler.em`, `accept_a_ref_mut_name_from_get_mut_calls_the_handler.em`, `reject_the_handler_name_used_again_after_the_call.em`, `reject_a_stateful_callback_given_to_a_read_only_parameter.em`, `accept_run_takes_the_stateful_callback_ref_mut.em` |
| expected result | compile-fail E2105 (line 866, word for word); run-pass `1`, `2`; compile-fail E3023 (AN-58, AN-61, word for word); run-pass `2`; compile-fail E3060 (AN-61, in this program's lines); compile-fail E3023 (line 866, word for word); run-pass `ann 1`, `ann 2`, `0` |

### R12-26. `once fn`: a function that gives a capture away runs once

| field | value |
|---|---|
| guarantee | A function value that gives a capture away is a `once fn()`; it is held by `owns f: once fn()`, refused where it may run twice, a plain `f: once fn()` is refused, a lambda cannot hand over a real variable it does not own, and a second call of a `once fn` is refused. |
| source | R12 lines 648-650; section 3 line 865 (E3013); section 4 E3040 (line 945) for the second call; AN-59 |
| enforced by | `drops` (E3013 today); type checker (`typeck`) for `once` |
| today's tests | `CLO-6/reject_a_once_fn_parameter_called_twice.em` (E3040), `CLO-6/reject_a_once_closure_where_fn_is_required_names_once_fn.em`, `CLO-6/accept_a_once_fn_parameter_takes_either_kind.em`, `CLO-6a/reject_a_once_fn_value_called_twice_or_from_a_borrowed_place.em` |
| missing edge cases | a `once fn` kept in a field and taken out to be called (`mem.take`); `callback(once)` (W6) |
| tests | `reject_a_function_that_gives_s_away_passed_to_call_twice.em`, `reject_a_plain_once_fn_parameter.em`, `reject_call_twice_given_a_lambda_that_hands_over_the_real_s.em`, `reject_a_once_fn_called_twice.em`, `accept_call_once_holds_an_owns_once_fn_and_call_twice_hands_over_copies.em` |
| expected result | compile-fail E3013 (three, line 865 and AN-59, word for word); compile-fail E3040 (my wording); run-pass `a`, `b`, `b` |

---

## Section 3 refusals and section 2 rows that belong to R12, and where each is tested

| rules.md line | refusal or row | tested by |
|---|---|---|
| 819 | E2105, a lambda or a function that runs one | R12-1, R12-2, R12-4, R12-5, R12-6, R12-12, R12-13, R12-20 |
| 820 | E2105, `captured` on a lambda | R12-8 |
| 821 | E2105, a thread or a kept C callback not named | R12-17, R12-18 (W6 too) |
| 832 | E3022, `retain` with a changing function; a function stored where a variable it changes owns it | R12-15, R12-16 |
| 836, 837 | E3020 through a function value, a library call, a lock, a foreign call | R12-14, R12-10, R12-20 (the foreign-call walk is W6's) |
| 860 | E2113 | R12-8, R12-9, R12-11, R12-12, R12-20 |
| 863 | E5069 `@ffi(no_callbacks)` | W6: `V3/conformance/R15/reject_no_callbacks_on_a_function_that_takes_a_callback.em` (not tested here) |
| 864 | E3026 | R12-21 |
| 865 | E3013 | R12-26 |
| 866 | E2105 / E3023, a call through a stored function value | R12-25 |
| 869 | E7018 | W6 (verdicts 43, 52): `V3/conformance/R15/reject_a_call_through_a_function_value_that_may_take_the_held_lock.em` (not tested here) |
| section 2, 783 | where a captured variable lives | R12-22 (address and named-capture state; real-variable state and box: for_owner item 1) |
| section 2, 784 | two words copied out | R12-23 |
| section 2, 785 | one count on a handler's own state | R12-24 |
| section 2, 794 | the change words cost nothing while running | compile-time only; no run test (nothing to run) |
