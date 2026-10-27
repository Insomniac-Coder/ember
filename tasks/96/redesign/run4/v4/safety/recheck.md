# Safety re-check, run 4 step 3 (agent 14)

`V4` = `tasks/96/redesign/run4/v4/`; "rules" = `V4/rules.md` as the rule writer left it (verdicts 1 to 82);
"line N" in a rule citation is a line of `V4/rules.md`. Nothing was compiled or run; nothing was fixed.

**tl;dr.** Every refusal of rules section 3 (86 rows) and every code of section 4 (69 error codes, 5
warnings) has at least one test that shows it; every check of section 2 that stops a program has a `run_`
test with its stop text. Of the 826 tests, **801 expect what the rules say**, **7 are wrong** (2 have a
second fault, 3 give a fix that misses its import line, 1 names a name the program does not have, 1
contradicts section 0) and **18 are unsettled** (the rules give two readings, or none, or two groups
of tests disagree). The six holes of `V4/safety/findings.md` section 1 are each refused or made safe, with
tests; but **hole 3 is closed only for a function that ends an object when it runs**: two neighbouring
programs the rules as written still accept would read freed memory (section 6). Those, and the unsettled
points, are for the owner (section 8).

## 0. How it was checked

- Every test file was read whole: 826 tests in `V4/conformance/R1` .. `R17` (340 `accept_`, 424 `reject_`,
  62 `run_`), the 3 module files of `R12` (`logbook.em`, `log_widgets.em`, `widgets.em`), the `ledger`
  package of `R17`, and the C stubs `R15/kept_events.c`, `R16/wide_sum.c`.
- Each expected result (accepted or refused, the code, the stdout, the stop text) was checked against the
  rule that decides it, and each fix a message gives was checked to compile under the rules (verdict 73).
- Every "line N" a message or a stop text names was checked against the file
  (`target/redesign6/14/lines.py`, which prints the line each number points at): all are right.
- Format (`target/redesign6/14/index.py`): every `reject_` is `compile-fail` with one `#$ error` (one
  exception, wrong item 2); every `run_fail_` has one `#$ panics:`; every `accept_` is `compile-pass` or
  `run-pass`. 109 refused tests give the fix inside the `#$ error` text and no `#$ help:` line; today's
  runner (`compiler/ember_driver/tests/milestones.rs` lines 225-292) takes both forms. Several `#$ stdout:`
  lines in one file, and an empty `#$ stdout:` followed by `#$ ` lines, are both today's format.

## 1. Every refusal of section 3 has a `reject_` test

Section 3 has 86 rows (lines 977-1062). Each has at least one refused test; where a row gives several
messages, the tests named cover each one.

| row (line) | code | refused test(s) |
|---|---|---|
| hand-over, part handed over (977) | E3040 | `R1/reject_a_list_used_after_it_was_handed_over`, `R2/reject_a_field_used_after_it_was_handed_over_on_its_own`, `R2/reject_a_struct_used_whole_after_a_field_was_handed_over` |
| change through a name that only reads (978) | E3023 | `R2/reject_a_change_through_a_plain_pattern_name`, `R4/reject_a_plain_shares_mut_parameter_sets_a_cell` |
| `=` on a parameter, loop, pattern, opened name; `const ref mut` re-pointed (979) | E3023 | `R2/reject_equals_on_a_view_parameter`, `R2/reject_equals_on_a_number_parameter`, `R2/reject_equals_on_a_loop_name`, `R2/reject_equals_on_a_pattern_name`, `R2/reject_a_change_of_a_plain_opened_name`, `R2/reject_another_ref_given_to_a_const_ref_mut_name` |
| changing method through a read-only name (980) | E3023 | `R5/reject_a_changing_method_called_through_a_plain_loop_name`, `R5/reject_a_changing_method_called_on_a_plain_parameter` |
| changing owner made from a reading owner (981) | E3023 | `R3/reject_a_changing_owner_made_from_a_reading_owner` |
| changing owner from a plain `shares mut` parameter (982) | E3023 | `R4/reject_an_owner_that_changes_made_from_a_plain_parameter` |
| write through a plain `MutSpan` (983) | E3023 | `R4/reject_a_plain_mutspan_parameter_writes`, `R2/reject_a_write_through_a_plain_name_holding_a_mut_span` |
| change of what a read-only name reads (984) | E3021 | `R8/reject_a_change_while_a_read_only_name_is_used` |
| walk with views across a node-changing call (985) | E3021 | `R11/reject_a_walk_with_views_across_a_visit_that_may_change_nodes` |
| body does what its first line does not say, or the reverse (986) | E2105 | `R5/reject_a_caller_does_not_name_what_its_callee_changes`, `R5/reject_a_method_changes_a_field_with_no_clause`, `R5/reject_a_reads_method_reads_a_field_it_does_not_name`, `R5/reject_a_first_line_names_a_field_the_body_never_changes` |
| change through a link not named (987) | E2105 | `R5/reject_a_change_through_a_ref_mut_link_parameter_not_named`, `R5/reject_a_change_through_an_own_link_not_named` |
| may end an object whose `drop` changes something (988) | E2105 | `R8/reject_a_function_that_ends_an_enemy_without_naming_log`, `R1/reject_a_function_whose_local_ends_with_a_drop_that_pushes_to_log_without_naming_it`, `R2/reject_replacing_an_object_whose_drop_pushes_to_log_without_naming_it` |
| lambda, or a function that runs one (989, seven messages) | E2105 | `R12/reject_a_lambda_that_pushes_to_log_without_saying_so`, `R12/reject_row_6_handler_without_changes_score`, `R12/reject_for_each_given_a_lambda_that_does_not_say_it_changes_log`, `R12/reject_a_function_that_ends_the_players_it_owns_without_naming_player`, `R12/reject_a_thread_function_that_pushes_under_the_log_lock_without_changes_log`, `R12/reject_click_without_log`, `R12/reject_click_without_captured` |
| `captured` on a lambda's own line (990) | E2105 | `R12/reject_captured_on_a_lambdas_own_first_line` |
| thread started or callback handed to C, not named (991) | E2105 | `R12/reject_main_starts_a_thread_that_changes_results_without_naming_it`, `R15/reject_setup_hands_c_a_callback_without_naming_its_changes` |
| `changes this`/`owns this` beside others (992) | E2107 | `R5/reject_changes_this_beside_a_field`, `R5/reject_changes_this_beside_reads`, `R5/reject_owns_this_beside_changes_this` |
| `changes this`/`owns this` on `init` (993) | E2108 | `R5/reject_changes_this_on_init` (the `owns this` form has no test) |
| `changes this` on a `drop` (994) | E2122 | `R5/reject_changes_this_on_a_drop`, `R5/reject_changes_this_on_a_drop_that_changes_nothing_outside` |
| first line out of order (995) | E0118 | `R5/reject_a_first_line_out_of_order` |
| `this` given another object (996) | E2103 | `R5/reject_this_given_another_object` |
| `self` (997) | E1010 | `R5/reject_self_names_a_methods_object` |
| `this` with no object (998) | E1010 | `R5/reject_this_in_a_free_function`, `R5/reject_this_in_a_static_fn` |
| module-level statement, `global` (999) | E0100 | `R15/reject_a_module_level_assignment`, `R15/reject_python_global` |
| `@view` on a type with no view (1000) | E2030 | `R7/reject_view_on_a_type_that_holds_no_view` |
| `pub` generic without bounds (1001) | E2040 | `R14/reject_a_pub_generic_without_its_written_bound`, `R17/reject_a_pub_generic_without_its_bounds` |
| interface not `dyn`-usable (1002) | E2050 | `R14/reject_a_generic_method_that_returns_its_type_parameter_through_dyn` |
| `static fn` outside a type (1003) | E2115 | `R5/reject_static_fn_outside_a_type` |
| `mut` alone (1004, three messages) | E0113 | `R2/reject_mut_alone_on_a_pattern_name`, `R4/reject_mut_alone_on_a_parameter`, `R5/reject_mut_this_on_a_methods_first_line` |
| `ref`/`ref mut` in a parameter's type (1005) | E0117 | `R4/reject_ref_mut_in_a_parameters_type`, `R4/reject_ref_in_a_parameters_type` |
| call's word does not match (1006, five messages) | E3028 | `R4/reject_a_call_without_ref_mut_for_a_ref_mut_parameter`, `R4/reject_ref_mut_at_a_call_to_a_const_ref_mut_parameter`, `R4/reject_a_mutspan_argument_without_ref_mut`, `R4/reject_ref_mut_at_a_call_to_a_plain_parameter`, `R4/reject_ref_mut_before_a_mutex_argument` |
| `ref mut` before what may not change (1007) | E3027 | `R4/reject_ref_mut_given_from_a_plain_parameter`, `R4/reject_ref_mut_on_a_query_item_the_query_only_reads`, `R4/reject_ref_mut_before_a_new_number` |
| two changeable names of one place (1008) | E3022 | `R4/reject_the_same_body_given_twice`, `R4/reject_a_function_value_that_changes_the_calls_object`, `R12/reject_a_handler_stored_in_the_button_it_changes` (its run-time stop: section 2 below) |
| changeable argument and one reaching into it (1009) | E3021 | `R4/reject_an_argument_reaching_into_what_a_changeable_one_owns`, `R4/reject_two_tracks_of_one_map_not_proved_apart` |
| part of what the callee changes (1010) | E3031 | `R4/reject_a_view_argument_into_what_the_callee_clears_and_then_uses`, `R5/reject_a_method_called_on_a_part_of_what_it_clears` |
| `ref mut` place used through another name (1011) | E3022, E3021 | `R7/reject_two_kept_ref_mut_fields_of_one_player`, `R8/reject_a_read_through_another_name_while_a_ref_mut_name_is_used`, `R8/reject_a_link_that_may_be_the_ref_mut_parameters_object` |
| `sharedref` without `mut` (1012) | E0114 | `R7/reject_sharedref_without_mut` |
| collection changed during a walk (1013, six messages) | E3020 | `R8/reject_a_push_while_a_for_loop_walks_the_list`, `R8/reject_an_insert_while_a_loop_walks_a_maps_keys`, `R8/reject_ending_an_object_whose_drop_pushes_to_the_walked_list`, `R15/reject_a_task_changes_the_list_its_parent_walks`, `R12/reject_clicking_a_log_button_inside_a_loop_over_log`, `R12/reject_a_for_walk_of_handlers_whose_type_changes_ui` |
| library call or C call during a walk (1014) | E3020 | `R8/reject_a_library_call_given_a_function_that_changes_the_walked_list`, `R15/reject_a_c_call_inside_a_walk_a_kept_callback_may_change`, `R15/reject_an_unmarked_c_function_inside_the_walk` |
| list a generator walks (1015) | E3020 | `R8/reject_a_change_to_a_list_a_stored_generator_walks` |
| removal in a counting loop (1016) | E3020 | `R8/reject_a_removal_inside_a_counting_loop` |
| view used after a reshape, end or removal (1017, five messages) | E3060 | `R8/reject_a_removal_ends_a_position`, `R10/reject_a_view_from_a_link_used_after_a_call_that_may_end_its_player`, `R12/reject_a_name_opened_from_a_link_used_after_a_call_that_may_end_players`, `R4/reject_a_view_put_in_through_ref_mut_used_after_its_source_grows`, `R8/reject_a_map_insert_ends_a_view_of_its_keys` |
| change to a temporary would be lost (1018) | E2140 | `R8/reject_a_change_to_a_temporary_that_would_be_lost` |
| view of a temporary (1019) | E3063 | `R8/reject_returning_a_view_of_a_temporary` |
| view out of a lock's block (1020) | E3063 | `R8/reject_a_view_returned_out_of_a_lock` |
| callee's local put in through `ref mut` (1021) | E3063 | `R4/reject_a_view_of_the_callees_own_local_put_in_through_ref_mut` |
| kept view outlives its target (1022) | E3063 | `R7/reject_a_sharedref_mut_holder_returned_past_its_target`, `R7/reject_a_list_of_views_returned_past_what_it_views` |
| value outlives its arena (1023) | E3063 | `R3/reject_a_value_returned_past_the_end_of_its_arena` |
| result outside its `from` (1024) | E3062 | `R6/reject_a_result_outside_what_its_from_names` |
| view field of a class, bad `from` (1025, four messages) | E2118 | `R7/reject_a_view_field_of_a_class_without_from`, `R7/reject_from_naming_a_field_that_is_not_let`, `R7/reject_from_naming_a_fixed_array_field`, `R6/reject_from_names_a_parameter_and_a_field` |
| `copy of` what it cannot copy (1026) | E2116 | `R3/reject_copy_of_a_weakref_link`, `R11/reject_copy_of_a_link`, `R3/reject_copy_of_a_view`, `R3/reject_copy_of_a_value_whose_views_point_into_its_own_let_field`, `R3/reject_copy_of_a_type_with_no_clone` |
| `@cached` (1027, five messages) | E2117 | `R5/reject_cached_without_reads`, `..._with_another_parameter`, `..._reading_a_shares_field`, `..._with_changes`, `..._reading_a_field_other_threads_change` |
| `Some(...)` in a header (1028) | E0115 | `R13/reject_some_in_an_if_header`, `R13/reject_some_in_a_while_header` |
| opening that always holds; kind test that always passes (1029) | E2036 | `R13/reject_exists_as_on_a_value_that_always_holds_one`, `R13/reject_exists_as_on_a_result`, `R14/reject_a_kind_test_that_always_passes` |
| opening or kind test naming a name in use (1030) | E2106 | `R13/reject_an_opened_name_already_in_use`, `R14/reject_a_kind_test_naming_a_name_already_in_use` |
| name with no word after one with a word (1031) | E0116 | `R3/reject_a_loop_name_with_no_word_after_one_with_a_word`, `R3/reject_a_captured_name_with_no_word_after_one_with_a_word`, `R3/reject_a_lambda_parameter_with_no_word_after_one_with_a_word` |
| `ref mut` in a capture list (1032) | E0120 | `R12/reject_ref_mut_in_a_capture_list` |
| `weakref` of a part (1033) | E2119 | `R11/reject_a_weakref_of_a_struct_stored_inline_in_a_list` (the field form has no test) |
| `owns` missing on the making line (1034) | E2123 | `R3/reject_a_value_made_for_an_owns_field_without_owns` |
| type containing itself (1035) | E2200 | `R3/reject_a_type_that_contains_itself_with_no_block_between`, `R3/reject_a_struct_holding_a_view_of_its_own_type` |
| `owns` where it can only copy (1036) | E2085 | `R4/reject_owns_on_an_int_parameter`, `R3/reject_owns_on_a_captured_number` |
| `owns` already handed over, before `fn` (1037) | E0109 | `R3/reject_owns_before_a_value_its_place_already_hands_over`, `R3/reject_owns_before_fn`, `R4/reject_owns_written_before_an_argument` |
| type word that cannot apply (1038) | E2114 | `R3/reject_owns_on_a_class_type`, `R3/reject_using_after_a_type_with_no_heap_memory` |
| bare interface name (1039) | E2173 | `R3/reject_a_bare_interface_name_used_as_a_type` |
| function value or override changing too much (1040, five messages) | E2113 | `R12/reject_a_log_handler_in_a_list_of_plain_functions`, `R12/reject_a_log_handler_in_a_button_that_allows_only_captured`, `R12/reject_a_two_variable_handler_in_a_captured_list`, `R12/reject_an_inspect_adapter_that_changes_log_made_a_dyn_iterator_that_says_nothing`, `R5/reject_an_override_changes_more_than_its_virtual` |
| lambda parameter mode; link used unopened; object into a link (1041) | E2020 | `R12/reject_a_lambda_that_takes_over_what_each_name_lends`, `R12/reject_a_plain_lambda_parameter_where_each_name_hands_the_name_over`, `R13/reject_a_weakref_link_used_as_its_target_without_opening_it`, `R3/reject_an_object_stored_into_a_weakref_place_without_weakref` |
| value from another memory manager (1042) | E2020 | `R3/reject_a_value_from_the_built_in_manager_given_to_an_arena_place`, `R3/reject_a_value_from_another_kind_of_arena_given_to_an_arena_place` |
| `no_callbacks` on a callback-taking function (1043) | E5069 | `R15/reject_no_callbacks_on_a_function_that_takes_a_callback` |
| function value outliving what it uses (1044) | E3026 | `R12/reject_a_handler_that_outlives_the_callers_total`, `R12/reject_a_returned_function_that_changes_its_makers_local` |
| hand-over from a non-owner (1045, four messages) | E3013 | `R1/reject_a_plain_parameter_pushed_into_a_list`, `R12/reject_a_function_that_gives_s_away_passed_to_call_twice`, `R12/reject_a_plain_once_fn_parameter`, `R2/reject_a_field_taken_out_of_a_class_object` |
| stored function called from a read-only holder (1046) | E2105, E3023 | `R12/reject_click_walks_its_handlers_without_changes_handlers`, `R12/reject_a_stateful_callback_given_to_a_read_only_parameter` |
| `virtual` in a closed class (1047) | E2112 | `R5/reject_virtual_in_a_class_that_cannot_be_subclassed` |
| method no bound declares (1048) | E2072 | `R14/reject_a_generic_body_calling_a_method_no_interface_declares` |
| wait for a held lock (1049, three messages) | E7018 | `R15/reject_a_join_inside_the_lock_its_thread_needs`, `R15/reject_a_call_through_a_function_value_that_may_take_the_held_lock`, `R15/reject_a_walk_under_the_lock_across_a_function_value_that_changes_log` |
| `thread_shared` on a value not made so (1050) | E7017 | `R3/reject_thread_shared_on_a_value_not_made_thread_shared`, `R15/reject_thread_shared_from_a_plain_parameter` |
| `thread_shared` value changed directly (1051) | E7014 | `R9/reject_a_thread_shared_object_changed_directly` |
| thread or kept callback, name with no word (1052) | E7004 | `R15/reject_an_unscoped_thread_using_a_name_with_no_capture_word`, `R15/reject_a_kept_callback_using_a_name_with_no_capture_word` |
| one-thread owner reaching another thread (1053) | E7015 | `R12/reject_a_shares_mut_owner_held_by_a_function_any_thread_may_run`, `R15/reject_a_callback_c_may_run_on_any_thread_holding_a_shares_owner` |
| two tasks change one place (1054) | E7007 | `R15/reject_a_task_reads_a_field_another_task_adds_to` |
| thread-bound value sent (1055) | E7008 | `R15/reject_a_lock_guard_sent_to_a_task` |
| parallel chunks writing one place (1056) | E7010 | `R15/reject_a_parallel_loop_over_shares_mut_objects` |
| main-thread module variable from another thread (1057) | E7012 | `R15/reject_a_thread_changes_an_unguarded_static_mut` |
| guarded field without its lock (1058) | E7013 | `R15/reject_a_guarded_field_changed_outside_its_lock` |
| `@parallel` on what it cannot split (1059) | E7016 | `R15/reject_a_parallel_while_loop` |
| unsafe step outside `unsafe:`; hand-written `Send` (1060, three messages) | E3100 | `R16/reject_an_unsafe_operation_in_an_unsafe_fn_body_outside_a_block`, `R16/reject_calling_an_unsafe_fn_outside_unsafe`, `R15/reject_implementing_send_without_unsafe` |
| run-time alias check in `@no_alias_checks` (1061) | E4030 | `R16/reject_an_index_compare_inside_no_alias_checks` |
| removed words (1062) | E1010, E0104 | 28 E1010 and 4 E0104 tests, one per word, among them `R1/reject_mem_forget_is_not_a_word_of_ember`, `R3/reject_box_is_not_a_word_of_ember`, `R3/reject_box_with_a_manager_is_owns_using`, `R3/reject_an_array_with_a_manager_is_array_using`, `R3/reject_copied_is_now_copy_each`, `R14/reject_cast_when_is_not_ember`, `R6/reject_borrows_is_now_from`, `R15/reject_the_sync_class_mark`. Two words of the row have no test: `@fp(contract)` (write `@fp(allow_fma)`) and `fn drop(mut self)` (write `fn drop():`) |

## 2. Every code of section 4 has a test

All 74 codes of section 4 (lines 1088-1163) are expected by at least one test. Each of the 69 error codes
has at least one `reject_` test; the 5 warnings (L3001, L3015, W2190, W3012, W3013) cannot have one, since a
warned program compiles, and each has a `compile-pass` or `run-pass` test that expects it
(`R11/accept_a_cycle_of_shares_owners_is_warned`, `R9/run_a_tree_child_with_two_owners_changed_through_either`;
`R16/accept_an_unsafe_fn_without_safety_is_warned`; `R4/accept_an_unused_check_disjoint_result_is_warned`;
`R16/accept_an_unsafe_block_without_a_safety_note_is_warned`;
`R16/accept_a_safety_note_on_an_unsafe_fn_line_is_warned`). The codes not in section 3's table (E0005,
E1030, E2104, E2221) have theirs: `R15/reject_async_is_reserved`, `R15/reject_a_second_main_in_one_module`,
`R7/reject_changing_the_let_field_views_point_into`, `R15/reject_a_view_of_the_frames_own_local_across_yield`.
No test uses a code that section 4 does not list. The full list of which test expects which code is
`target/redesign6/14/codes_table.md`.

## 3. Every check of section 2 that stops the program has a `run_` test with its text

| check (section 2's stop table, lines 951-960) | test | stop text expected |
|---|---|---|
| one compare of two indexes (R4, R8 (f)) | `R4/run_the_same_body_given_twice_stops_the_program`; `R8/run_two_ref_mut_names_of_one_list_are_compared_once`; `R8/run_a_ref_mut_name_and_a_read_only_name_of_one_list_are_compared_once` | "`a` and `b` name the same element of `bodies` (index 1); `collide` needs two different bodies", and the same words in the other two programs' names |
| `assert_disjoint` (R4) | `R4/run_assert_disjoint_stops_on_one_track` | "`assert_disjoint(d, s)` on line 18: the two spans overlap (`dst_name` and `src_name` name one track)" (line 18 checked) |
| owner count at its maximum (R9, verdict 59) | `R9/run_fail_too_many_owners_of_one_player_stop_the_program` | "too many owners of a `Player` (4,294,967,295)" |
| weak count at its maximum (R11, verdict 59) | `R11/run_fail_too_many_weakref_links_to_one_player_stop_the_program` | "too many `weakref` links to a `Player` (4,294,967,295)" |
| `cast s as Circle` (R14) | `R14/run_fail_a_cast_to_the_wrong_kind_stops_the_program` | "`cast b as Circle` on line 28: `b` is a `Square`, not a `Circle`" (line 28 checked) |
| trampoline, call after the token ended (R12, R15) | `R15/run_fail_c_runs_a_callback_after_it_was_unregistered` | "C called the callback registered on line 34 after its token ended; it was unregistered" (line 34 is the `register_cb` line) |
| trampoline, `threads(creator)` callback on another thread (R15) | `R15/run_fail_a_creator_callback_called_on_another_thread` (`#$ profiles: debug, development, release`) | "C called the callback registered on line 35 on another thread, but its contract says `threads(creator)`" |
| bounds checks | `R8/run_a_while_loop_index_is_checked`; `R16/run_fail_a_bounds_check_still_runs_inside_no_alias_checks` | "index 1 is out of bounds for a length of 1"; "index 3 is out of bounds for a length of 3" |

The rows of section 2's first table that do not stop the program (counts, header tests, the weak-count
moves, the deferred-drop region, two words copied out, the count on a handler's own state, the
type-id compare, locks, reductions, the trampoline, the generator protocol, arenas) each have a `run_` or
`run-pass` test that exercises them (for example `R11/run_a_second_link_finds_its_target_gone_after_the_first_link_and_the_target_end`,
`R11/run_a_walk_whose_visit_may_end_nodes_stops_at_the_first_ended_parent`,
`R12/run_a_handler_that_grows_the_list_during_dispatch_keeps_its_state`,
`R10/run_a_view_of_a_thread_shared_object_through_a_link_holds_an_atomic_count`,
`R3/run_a_value_ends_in_the_arena_instance_it_came_from`).

Two notes on this section:
- `R15/run_fail_c_calls_a_once_callback_twice` is a run-time stop that section 2 has **no text for** (a
  `callback(once)` function called a second time); the test checks only the start of the text,
  "C called the callback registered on line 29" (unsettled item U9).
- The two count tests need about 32 GiB of memory to run (4,294,967,295 owners or links kept in a list,
  more while the list grows): correct by the rules, but not runnable on the laptop. With `mem.forget`
  gone (verdict 58) no smaller program reaches the maximum.

## 4. Tests whose expected result is not what the rules say

### Wrong (7)

| test | why | rule |
|---|---|---|
| `R3/accept_the_first_owner_keeps_what_it_could_do` | After `other = shares items`, `items` no longer owns the list alone, so it is a link, and `items.push(4)` is a change through a link, which `main`'s first line must name (`fn main() changes Array[int]:`); without it the program is refused (E2105). The same program is written with `changes Array[int]` in `R8/accept_a_shares_owner_sees_a_change_through_another_owner` and `R9/run_a_shares_owner_sees_what_the_first_owner_changes`, so the suite disagrees with itself. The W1 writer left this open for the owner (`matrix-W1.md` row R3-3); the rules' text decides it. | section 0 lines 37-39 ("link: ... names an object it does not own alone: ... `shares T` ... (one of several owners)"); R5 lines 349-354 |
| `R14/reject_as_question_and_as_bang` | Two faults, two `#$ error` lines ("Two mistakes, two messages"); the test format asks one fault per refused test. Split it into two tests, one for `as?`, one for `as!`. | `inputs/duties.md` line 59 |
| `R16/reject_assert_disjoint_inside_no_alias_checks` | `assert_disjoint` is used without `from std.mem import assert_disjoint`, which every other test imports (and today's `tests/conformance/DSJ-1`): a second fault beside E4030. | `V4/vocabulary.md` `assert_disjoint` row; one fault per refused test |
| `R4/reject_an_ignored_check_disjoint_proves_nothing` | The message's first way out, `assert_disjoint(d, s)` in place of line 17, does not compile: the program imports only `check_disjoint`. The fix needs `from std.mem import assert_disjoint` too. | verdict 73; R18 lines 903-904 |
| `R4/reject_two_tracks_of_one_map_not_proved_apart` | The same: `assert_disjoint(d, s)` before line 16, with no import of it. The rules' own message (line 1009) has the same gap. | verdict 73; R18 |
| `R4/accept_an_unused_check_disjoint_result_is_warned` | W2190's fix `assert_disjoint(a.as_span(), b.as_span())` needs the import line too; the rules' W2190 text (line 1072) has the same gap. | verdict 73 |
| `R14/reject_a_dyn_iterator_whose_steps_change_more_than_its_type_says` | The message begins "`it` runs a function that changes `LOG`", but the refused line in `main` has no `it` (the value is made inside the call; `it` is `show`'s parameter): the text was copied from `R12/reject_an_inspect_adapter_..._says_nothing`, where `it` is a local. Name the value the line makes, `xs.iter().inspect(...)`. | R18 lines 900-901 |

### Unsettled (18 tests, 10 points)

| point | tests | the rule text that leaves it open |
|---|---|---|
| U1. Does verdict 74's `owns` on the making line apply to a value made for an `Option[owns T]` field or an `Array[owns dyn I]` element? The suite answers both ways. | `R3/run_a_value_stored_into_an_owns_place_gets_its_own_block` (`left=Some(Tree(value=2))`, no `owns`) against `R15/accept_a_generator_views_what_its_caller_keeps` and `R15/reject_a_generator_walk_held_by_views_of_its_own_frame` (`next=Some(owns Link(value=1))`). Every list of `owns dyn I` (12 tests in R3 and R14) makes its elements without `owns`, and depends on the same answer. | R3 lines 184-188: "on the line that makes a value for an `owns` field" (a field of type `owns T`; nothing said of `Option[owns T]` or list elements) |
| U2. A method that changes an object through a link held in its own field: `changes target`, `changes Player`, or both? | `R7/accept_a_camera_owning_a_share_may_be_returned`, `R7/accept_sharedref_mut_holders_and_the_owner_all_change_one_player`, `R7/reject_a_method_that_may_change_the_target_inside_a_loop_over_it`, `R9/accept_a_field_change_through_a_shares_mut_field`, `R9/reject_a_field_change_through_a_shares_field`, `R9/run_two_sharedref_mut_holders_and_the_owner_change_one_player` (all write `fn shake() changes target:` and `fn main() changes Player:`) | R5 lines 310-311 ("`changes a, b` ... these fields and anything reached through them") against lines 316-318 and 351-354 ("`changes T` for objects ... it changes through a link"; "a change through a link always says `changes T`, whatever holds the link") |
| U3. An `Atomic` field of an object reached through a link: named by the object's type or by `Atomic[int]`? | `R9/reject_two_tasks_set_one_player_through_two_sharedref_links` (its fix writes `changes Atomic[int]` for `a.target.hp.store(1)`), against `R12/accept_a_thread_shared_owner_held_by_a_function_any_thread_may_run`, `R15/accept_a_class_with_atomic_fields_is_shared_without_a_mark`, `R15/run_a_thread_runs_a_handler_holding_a_thread_shared_owner` and the rules' own E7015 message (line 1053), which name the object (`changes Counter`, `changes Stats`) | R12 lines 605-611: "named by the type of what changes (`changes Mutex[Stats]` ...; `changes Atomic[int]`)", "naming by type is for what is reached through a link" |
| U4. Which code refuses `r = ref b` for a `ref mut` name? | `R2/reject_a_read_only_ref_given_to_a_ref_mut_name`, `R4/reject_ref_b_given_to_a_ref_mut_name` (both E3023) | R2 lines 143-146 say the line "says so with `ref mut` again" but no code; section 3 and E3023 (line 1135) do not list it |
| U5. A kind test that can never pass (a type that does not implement the interface) | `R14/reject_a_kind_test_between_unrelated_types` (E2020) | R14 lines 800-807 refuse only a test that always passes (E2036); no row of section 3, and E2020's meaning (line 1103) does not list it |
| U6. `return ref mut pool.dense[...]` through a read-only `pool`: E3023 or E3027? | `R14/reject_a_plain_name_from_a_cast_of_a_place_returns_ref_mut` (E3023) | E3027 (line 1137) is "`ref mut` before something this function may not change", but lists only an argument, a loop or a pattern name; E3023 (line 1135) is a change, and taking `ref mut` is not one |
| U7. A float reduction added more than once by one task | `R15/reject_a_float_sum_added_more_than_once_per_task` (E7007, from AN-71) | R15 lines 830-836: `+=` gets one private accumulator per task, "always the right total"; no float exception is written |
| U8. The trampoline's tag layout | `R15/run_c_holds_a_tagged_number_not_an_address` (stdout `16512`, the top 16 bits `0x4080`, from AN-78) | R15 lines 866-868: "a tagged `{slot, generation}` number"; no layout |
| U9. The stop text for a `callback(once)` called twice | `R15/run_fail_c_calls_a_once_callback_twice` | section 2 lines 951-960 have no row for it |
| U10. A reduction made through two `sharedref mut` links to one object | `R9/run_two_tasks_take_hp_off_one_player_through_two_sharedref_links` (accepted, stdout `7`) | R9 lines 489-492 ("otherwise only as a `thread_shared` value is changed (E7007)") against R15 lines 831-834 (verdict 77: a reduction "on a field of an object the tasks reach") |

The rest of the suite, 801 tests, expects what the rules say: the code, the line the error stands on, the
stdout, the stop text, and a fix that compiles.

## 5. The six safety holes of `findings.md` section 1

| hole | now | rule (lines) | refused or run tests | allowed neighbours |
|---|---|---|---|---|
| 1. a view given to a function that then clears what it points into | **refused**: the compiler works out from the body which view parameters are used after a change and checks each call (E3031), through callees and across a package's edge | R4 278-294; R8 (g) 471-473; R17 891-893; verdict 53 | `R4/reject_a_view_argument_into_what_the_callee_clears_and_then_uses`, `R4/reject_a_view_argument_into_what_a_callee_of_the_callee_clears`, `R4/reject_a_ref_mut_argument_into_what_the_callee_clears`, `R8/reject_a_span_given_to_a_function_that_grows_what_it_shows`, `R17/reject_a_call_from_another_package_hands_a_part_of_what_it_clears` | `R4/accept_a_view_argument_used_before_the_callee_clears`, `R4/accept_a_copy_given_to_a_function_that_clears_log`, `R4/accept_text_written_in_the_program_given_to_a_function_that_clears_log`, `R8/accept_a_view_whose_last_use_is_an_argument_does_not_live_across_the_call`, `R17/accept_a_call_from_another_package_given_what_its_recorded_fact_allows` |
| 2. a method that destroys its own object | **refused**: `this` counts as a view parameter (E3031) | R4 290-294; R5 337-342; verdict 54 | `R5/reject_a_method_called_on_a_part_of_what_it_clears`, `R8/reject_a_method_that_reads_its_object_after_clearing_the_list_holding_it` | `R5/accept_a_method_that_uses_this_before_it_clears_what_owns_it`, `R8/accept_a_method_that_reads_its_object_before_clearing_the_list_holding_it` |
| 3. a stored function that owns what a view reads | **refused when the function ends the object as it runs**: its type names the type it may end (E2105 without it), so a view opened from a link is not used after the call (E3060). **Not closed** for a change of that object, or the end of the function value itself: section 6 | R12 611-617; R10 508-516, 520-527; verdicts 55, 82 | `R12/reject_a_function_that_ends_the_players_it_owns_without_naming_player`, `R10/reject_a_view_from_a_link_used_after_a_function_that_ends_the_players_it_owns`, `R12/reject_a_name_opened_from_a_link_used_after_a_call_that_may_end_players`, `R10/reject_a_view_from_a_link_used_after_a_call_that_may_change_players`, `R11/reject_a_walk_with_views_across_a_visit_that_may_change_nodes` | `R12/accept_a_function_that_ends_the_players_it_owns_names_player`, `R10/accept_a_view_from_a_link_used_before_a_function_that_ends_the_players_it_owns`, `R10/accept_a_view_from_a_link_used_after_a_call_that_changes_nothing`, `R11/accept_a_walk_with_views_when_the_visit_changes_no_node` |
| 4. `drop` and `weakref` | **made safe**: an object is ended before its `drop` runs, so a link opened in it finds it ended (no second end); ending an object counts as a call of its `drop` (E3020 in a walk, E2105 unnamed); a `drop` cannot keep its object (E3013) | R11 543-548; R5 322-331; R10 point 1 511-512; verdicts 56, 57 | `R11/run_a_link_opened_during_its_targets_drop_finds_it_ended` (stdout `made`, `ended`), `R11/run_a_childs_drop_finds_its_parent_ended`, `R8/reject_ending_an_object_whose_drop_pushes_to_the_walked_list`, `R8/reject_a_function_that_ends_an_enemy_without_naming_log`, `R1/reject_a_drop_method_that_keeps_its_object` | `R8/accept_ending_an_object_whose_drop_pushes_to_log_after_the_walk`, `R1/accept_a_drop_method_that_keeps_a_number_of_its_object`, `R11/run_a_walk_over_nodes_with_a_drop_method_stops_at_the_first_ended_parent` |
| 5. count overflow | **made safe**: `mem.forget` is gone (E1010), and every +1 of an owner or weak count is compared with its maximum (a stop with section 2's text) | R1 114-116; R9 498-501; R11 535-537; verdicts 58, 59 | `R1/reject_mem_forget_is_not_a_word_of_ember`, `R9/run_fail_too_many_owners_of_one_player_stop_the_program`, `R11/run_fail_too_many_weakref_links_to_one_player_stop_the_program` (both need about 32 GiB to run, section 3) | `R1/accept_a_value_ended_now_runs_its_clean_up` |
| 6a. a stored function holding a one-thread owner, run by another thread | **refused where the function type says `thread_shared fn()`** (E7015). The original form, a class whose function field is a plain `fn() changes Counter` and which is made `thread_shared`, is covered only by R12's sentence "the fields of a `thread_shared` object that hold functions use such types", which names **no code**, and **no test** refuses it (`matrix-W6.md` row R15-22 says so too) | R12 651-658; R15 844-846; verdict 61 | `R12/reject_a_shares_mut_owner_held_by_a_function_any_thread_may_run`, `R15/reject_a_handler_any_thread_may_run_holding_a_shares_mut_owner`, `R15/reject_a_thread_takes_a_shares_mut_bus_to_store_a_handler` | `R12/accept_a_thread_shared_owner_held_by_a_function_any_thread_may_run`, `R15/run_a_thread_runs_a_handler_holding_a_thread_shared_owner`, `R15/accept_a_thread_stores_a_handler_in_a_bus_threads_share` |
| 6b. a C callback that ends its own registration while it runs | **made safe**: C forgets it at once, its block is freed when it returns | R12 760-765; R15 846-847; section 2 line 937; verdict 62 | `R15/run_a_kept_callback_that_ends_its_own_token_then_uses_its_state` (stdout `1`, AddressSanitizer clean once built), `R15/run_fail_c_runs_a_callback_after_it_was_unregistered` | `R15/run_c_runs_a_kept_callback_through_the_trampoline` |

## 6. Two programs next to hole 3 that the rules as written accept

Verdict 55 names the class objects a function value may **end** when it runs. Two neighbouring things are
not named anywhere, and each lets a view read freed memory. The compiler that follows the text accepts
both programs. I do not decide them; they are in `for_owner`.

**6.1. A change, not an end, of an object in a function's own state.** R12 lines 611-613: "Its own state ...
is itself, and a change to it needs no word, except ending a class object". A class object the function
holds alone may still be reached elsewhere through a `weakref` link, which is the very reason verdict 55
gave for naming its end. A change of it (here, replacing a `String`, which frees its text) is not named,
so R8 (b), which reads what may reshape "from the types of the function values a call may run" (lines
421-423), sees nothing, and neither does R10's "across a call that may change an object of the view's type
(its first line or its type says `changes T`)" (lines 521-523):

```ember
class Node:
    name: String

fn main():
    n = Node(name=String(" a "))
    w = weakref n
    rename = fn() with owns n: n.name = String("b")    # its own state: no word, a plain `fn()`
    if w exists as p:
        s = p.name.trim()          # a view into the node's text, reached through the link
        rename()                   # frees that text
        println(s)                 # reads freed memory; no rule refuses a line of this program
```

**6.2. The end of a value whose type hides what it owns.** R10 point 1 (lines 508-513) counts "a write to a
place that owns one (`T`, `owns T`, `shares T`, `shares mut T`, `thread_shared T`)". A place of a function
type (or of `owns dyn I`) whose value owns a class object is not in that list, and its type does not say
what it owns. Replacing or ending such a value ends the object:

```ember
class Node:
    name: String

class Holder:
    keep: fn()

fn main():
    n = Node(name=String("a"))
    w = weakref n
    h = Holder(keep=fn() with owns n: println(n.name))
    if w exists as p:
        h.keep = fn(): pass        # the old function value ends, and with its state the only owner of the node
        println(p.name)            # `p` is a view with no count (R10 point 1): reads an ended node
```

The same blind spot hides a `drop`'s changes (verdict 57): a node whose `drop` pushes to `LOG`, held in a
function value's state, ended by replacing that value inside `for line in LOG:`, is a change of the walked
list that R8 (c) does not see. Rust closes both cases by counting (`Weak::upgrade` gives an owner); the
rules removed that count (R10, verdict 82), so the compile-time list of "what may change or end an object"
has to cover type-erased owners too, or such values must say what they own.

## 7. Tests the duty did not ask for but which are missing

- E2108 for `owns this` on `init` (section 3 line 993 gives both words; only `changes this` is tested).
- E2119 for a `weakref` of a field (line 1033: "a field, or an element stored inline"; only the element is
  tested).
- The removed words `@fp(contract)` and `fn drop(mut self)` (line 1062).
- A method called through a name opened from a link, whose body clears the list that may own the object
  (hole 2 reached through a link: R8 (b)'s "two paths may reach the same object" makes it E3031, but no
  test shows it).
- Hole 6a's original form (section 5): a class with a plain `fn()` field made `thread_shared`.

## 8. For the owner

1. Section 6.1: should a change of a class object held in a function value's own state be named on its
   type (as verdict 55 names its end), or should a view reached through a `weakref` not live across a call
   to such a function?
2. Section 6.2: the end of a function value, or of an `owns dyn I` value, that owns a class object: how is
   it seen (by R10 point 1, and by verdict 57's `drop` counting)?
3. Hole 6a's original form: which code refuses a `thread_shared` object whose class has a plain `fn()`
   field (R12 lines 655-656 give the requirement, no code)?
4. U1: does verdict 74's `owns` on the making line reach `Option[owns T]` fields and `Array[owns dyn I]`
   elements?
5. U2: a method that changes an object through a link in its own field writes `changes target`,
   `changes Player`, or both; and what does its caller write?
6. U3: an `Atomic` field of an object reached through a link is named by the object's type or by
   `Atomic[int]`?
7. Wrong item 1 (the name a value was first shared from is a link): the rules' section 0 makes it one; the
   W1 writer asked the owner to confirm.
8. U7 (float reductions), U8 (tag layout), U9 (once-callback stop text), U10 (reduction through two
   `sharedref mut` links): rule text the rule writer can settle once the owner says which way.
9. Rule text gaps found on the way (no decision needed, for the rule writer): E7007's meaning (R15 line 830,
   section 4 line 1149) lists reductions and locks but not `Atomic`, which its own message offers and the
   tests accept; the E3021 message of line 1009 and W2190's of line 1072 need `from std.mem import
   assert_disjoint`; section 2 has no text for the once-callback stop; E2020 (line 1103) and E3023 (line
   1135) do not list the cases of U4 and U5.
10. The two count-overflow tests need about 32 GiB of memory; they cannot run on the laptop.
