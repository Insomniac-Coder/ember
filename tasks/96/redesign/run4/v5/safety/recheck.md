# Safety re-check, run 4 step 4 (agent 10)

`V5` = `tasks/96/redesign/run4/v5/`; "rules" = `V5/rules.md` as the rule writer left it (verdicts 1 to 97, with
the corrected records of 53 and 58); "line N" is a line of `V5/rules.md` unless another file is named.
Nothing was compiled or run; nothing was fixed. Step 3's re-check, which this file replaces, is kept as found
in `target/redesign7/10/recheck_step3_as_found.md`.

**In short.** Every refusal of rules section 3 (79 rows) has at least one `reject_` test, and every code of
section 4 (65 error codes, 5 warnings) has a test; every check of section 2 that stops a program has a `run_`
test with its stop text, the two count limits now runnable (verdict 96's 8-bit counter). Of the 832 tests,
**828 expect what the rules say**, **1 is wrong** (a message's fix runs into verdict 85's `owns`), and **3 are
unsettled** by one pair of sentences in R4 that say two things about `ref mut xs[i]` beside `ref mut xs[j]`.
The six holes of `V5/safety/findings.md` section 1, the two hole 3 neighbours of step 3's re-check and hole
6a's original form are each refused or made safe, with tests. Two things are not done: one refusal message
has no test (a `ref mut` argument the callee also reaches by its own name, `heal(ref mut PARTY[0])`), and,
found on the way, the rules do not refuse handing to another thread a class object that `weakref` links on
this thread still point at (AN-71 says E7015; section 6). Nothing needs the owner.

## 0. How it was checked

- Every test file was read whole: 832 tests in `V5/conformance/R1` .. `R17` (367 `accept_`, 398 `reject_`, 67
  `run_`), the module `R12/widgets.em`, the `ledger` package of `R17` (`ledger/src/lib.em` and both
  `ember.toml`), and the C stubs' function list (`R15/kept_events.c`, `R16/wide_sum.c`).
- Each expected result (accepted or refused, the code, the line the error stands on, the stdout, the stop
  text) was checked against the rule that decides it, and each fix a message or a `#$ help:` line gives was
  checked to compile in that program (R18, verdict 73).
- Every "line N" a message, a help line or a stop text names was printed against the file
  (`target/redesign7/10/lines.py`, output `lines.txt`): each points at the line it means; the one reference
  into another file (`widgets.em`, line 14, in `R12/reject_clicking_a_button_of_another_module_inside_a_loop_over_log`)
  is right.
- Format (`target/redesign7/10/index.py`): every `reject_` is `compile-fail` with exactly one `#$ error`; every
  `run-fail` has one `#$ panics:`; every `accept_` is `compile-pass` or `run-pass`; five `run-fail` tests keep
  the `run_` prefix without `fail_` (`R4/run_assert_disjoint_stops_on_one_track`,
  `R4/run_the_same_body_given_twice_stops_the_program`, `R8/run_a_while_loop_index_is_checked`, and R8's two
  `..._are_compared_once`), as in step 3; the two count-limit tests carry `#$ count_bits: 8`.
- Old spellings in test code (`changes`, `reads`, `with` before captures, `captured`, `mem.forget`,
  `ForeignBox`, `Box[`, `owned`, `self`, `holds`, `.copied()`): each appears only in the `reject_` test that
  refuses that word. No test expects a code section 4 lists as removed (E2085, E2105, E2107, E2109, E2113,
  E2122, E7001, E7003, E7005).

## 1. Every refusal of section 3 has a `reject_` test

Section 3 has 79 rows (lines 1035-1113). Each has at least one refused test; where a row gives several
messages, each message has its test, except the three named in the last column (section 7).

| row (line) | code | refused test(s) | message without a test |
|---|---|---|---|
| hand-over: a name, a number with `owns`, a part (1035) | E3040 | `R1/reject_a_list_used_after_it_was_handed_over`, `R3/reject_a_number_used_after_a_function_took_it_over_with_owns`, `R4/reject_a_number_used_after_it_was_handed_to_an_owns_parameter`, `R2/reject_a_field_used_after_it_was_handed_over_on_its_own`, `R2/reject_a_struct_used_whole_after_a_field_was_handed_over` | |
| change through a name that only reads (1036) | E3023 | `R2/reject_a_change_through_a_plain_pattern_name`, `R4/reject_a_plain_shares_mut_parameter_sets_a_cell` | |
| `=` on a parameter, loop, pattern, opened name; `const ref mut` given another ref; `ref b` given to a `ref mut` name (1037) | E3023 | `R2/reject_equals_on_a_view_parameter`, `R2/reject_equals_on_a_number_parameter`, `R2/reject_equals_on_a_loop_name`, `R2/reject_equals_on_a_pattern_name`, `R2/reject_another_ref_given_to_a_const_ref_mut_name`, `R4/reject_a_const_ref_mut_parameter_given_another_span`, `R2/reject_a_read_only_ref_given_to_a_ref_mut_name`, `R4/reject_ref_b_given_to_a_ref_mut_name` | |
| changing method through a read-only name (1038) | E3023 | `R5/reject_a_changing_method_called_through_a_plain_loop_name`, `R5/reject_a_changing_method_called_on_a_plain_parameter`, `R14/reject_a_method_whose_kind_test_changes_pools_called_through_a_read_only_name` | |
| changing owner from a reading owner (1039) | E3023 | `R3/reject_a_changing_owner_made_from_a_reading_owner` | |
| changing owner from a plain `shares mut` parameter (1040) | E3023 | `R4/reject_an_owner_that_changes_made_from_a_plain_parameter` | |
| write through a plain `MutSpan` (1041) | E3023 | `R4/reject_a_plain_mutspan_parameter_writes`, `R2/reject_a_write_through_a_plain_name_holding_a_mut_span` | |
| stateful stored function from a read-only name (1042) | E3023 | `R12/reject_click_walks_stateful_handlers_with_a_plain_name`, `R12/reject_a_stateful_callback_given_to_a_read_only_parameter`, `R12/reject_a_plain_name_from_get_mut_calls_the_handler` | |
| change of what a read-only name reads (1043) | E3021 | `R8/reject_a_change_while_a_read_only_name_is_used`, `R2/reject_a_number_viewed_with_ref_changed_while_viewed`, `R12/reject_a_read_only_view_across_a_call_that_changes_what_it_reads` | |
| walk with views across a node-changing call (1044) | E3021 | `R11/reject_a_walk_with_views_across_a_visit_that_may_change_nodes` | |
| `owns this` on `init` (1045) | E2108 | `R5/reject_owns_this_on_init` (new in step 4) | |
| `init` calls a method before its fields are set (1046) | E2100 | `R5/reject_init_calls_a_method_whose_body_reads_a_field_not_yet_set`, `R5/reject_init_calls_a_method_before_the_field_its_body_reads_is_set` | |
| first line out of order (1047) | E0118 | `R5/reject_a_first_line_out_of_order` | |
| `this` given another object (1048) | E2103 | `R5/reject_this_given_another_object` | |
| `self` (1049) | E1010 | `R5/reject_self_names_a_methods_object` | |
| `this` with no object (1050) | E1010 | `R5/reject_this_in_a_free_function`, `R5/reject_this_in_a_static_fn` | |
| module statement, `global`, a clause Ember does not have (1051, four messages) | E0100 | `R15/reject_a_module_level_assignment`, `R15/reject_python_global`, `R5/reject_changes_written_after_a_functions_brackets`, `R5/reject_reads_written_after_a_methods_brackets`, `R12/reject_a_function_type_written_with_changes_log`, `R12/reject_an_arrow_lambda_written_with_changes_removed`, `R12/reject_row_6_handler_written_with_changes_score`, `R3/reject_with_before_a_capture_list`, `R12/reject_with_before_a_lambdas_captures` | |
| `@view` on a type with no view (1052) | E2030 | `R7/reject_view_on_a_type_that_holds_no_view`, `R7/reject_view_on_a_struct_whose_only_view_field_is_from_static` | |
| `pub` generic without bounds (1053) | E2040 | `R14/reject_a_pub_generic_without_its_written_bound`, `R17/reject_a_pub_generic_without_its_bounds` | |
| interface not `dyn`-usable (1054) | E2050 | `R14/reject_a_generic_method_that_returns_its_type_parameter_through_dyn` | |
| `static fn` outside a type (1055) | E2115 | `R5/reject_static_fn_outside_a_type` | |
| `mut` alone (1056, three messages) | E0113 | `R2/reject_mut_alone_on_a_pattern_name`, `R4/reject_mut_alone_on_a_parameter`, `R5/reject_mut_this_on_a_methods_first_line` | |
| `ref`/`ref mut` in a parameter's type (1057) | E0117 | `R4/reject_ref_mut_in_a_parameters_type`, `R4/reject_ref_in_a_parameters_type` | |
| call's word does not match (1058, five messages) | E3028 | `R4/reject_a_call_without_ref_mut_for_a_ref_mut_parameter`, `R4/reject_ref_mut_at_a_call_to_a_const_ref_mut_parameter`, `R4/reject_a_mutspan_argument_without_ref_mut`, `R4/reject_ref_mut_at_a_call_to_a_plain_parameter`, `R4/reject_ref_mut_before_a_mutex_argument`, `R4/reject_const_at_a_call_to_a_plain_ref_mut_parameter`, `R15/reject_ref_mut_before_a_channel_end_at_a_call` | |
| `ref mut` before what may not change (1059, three messages) | E3027 | `R4/reject_ref_mut_given_from_a_plain_parameter`, `R4/reject_ref_mut_on_a_query_item_the_query_only_reads`, `R14/reject_a_plain_name_from_a_cast_of_a_place_returns_ref_mut`, `R4/reject_ref_mut_before_a_new_number`, `R4/reject_a_plain_mutspan_local_given_as_ref_mut`, `R4/reject_a_ref_mut_loop_name_over_a_list_the_function_only_reads`, `R14/reject_a_ref_mut_kind_test_name_of_a_read_only_parameter` | |
| two changeable names of one place (1060, three messages and a stop) | E3022 | `R4/reject_the_same_body_given_twice`, `R4/reject_a_function_value_that_changes_the_calls_object`, `R12/reject_retain_given_a_function_that_changes_the_list`, `R12/reject_a_handler_stored_in_the_button_it_changes`, `R4/reject_an_element_given_as_ref_mut_beside_its_whole_list`, `R4/reject_a_method_given_its_own_object_as_ref_mut` (the stop: section 3 below) | |
| changeable argument and one reaching into it (1061, two messages) | E3021 | `R4/reject_an_argument_reaching_into_what_a_changeable_one_owns`, `R4/reject_two_tracks_of_one_map_not_proved_apart`, `R4/reject_two_spans_of_one_list_one_changeable`, `R4/reject_a_read_and_a_change_of_one_game_in_one_call`, `R4/reject_an_ignored_check_disjoint_proves_nothing` | |
| the call check (1062, five messages) | E3031 | `R4/reject_a_view_argument_into_what_the_callee_clears_and_then_uses`, `R5/reject_a_method_called_on_a_part_of_what_it_clears`, `R4/reject_a_ref_mut_argument_into_what_the_callee_clears`, `R11/reject_a_visit_function_that_reads_its_node_after_ending_it`, `R4/reject_a_view_argument_into_what_a_callee_of_the_callee_clears`, `R8/reject_a_method_that_reads_its_object_after_clearing_the_list_holding_it`, `R8/reject_a_span_given_to_a_function_that_grows_what_it_shows`, `R17/reject_a_call_from_another_package_hands_a_part_of_what_it_clears` | **the fourth**: `heal(ref mut PARTY[0])` for a `heal` that reads `PARTY[0]` by its own name (section 7) |
| a `ref mut` name's place through another name (1063, three messages) | E3022, E3021 | `R7/reject_two_kept_ref_mut_fields_of_one_player`, `R8/reject_a_read_through_another_name_while_a_ref_mut_name_is_used`, `R8/reject_a_link_that_may_be_the_ref_mut_parameters_object`, `R8/reject_a_push_through_a_shares_mut_local_while_a_ref_mut_name_of_its_member_is_used` | |
| `sharedref` without `mut` (1064) | E0114 | `R7/reject_sharedref_without_mut` | |
| a collection changed during a walk (1065, eight messages) | E3020 | `R8/reject_a_push_while_a_for_loop_walks_the_list`, `R8/reject_a_write_of_an_element_while_a_for_loop_walks_the_list`, `R8/reject_a_value_under_a_key_written_while_a_loop_walks_the_maps_keys`, `R8/reject_an_insert_while_a_loop_walks_a_maps_keys`, `R8/reject_ending_an_object_whose_drop_pushes_to_the_walked_list`, `R15/reject_a_task_changes_the_list_its_parent_walks`, `R12/reject_clicking_a_log_button_inside_a_loop_over_log`, `R12/reject_a_for_walk_of_handlers_one_of_which_clears_them_through_a_weakref` (sets and strings: `R8/reject_an_add_while_a_loop_walks_a_set`, `R8/reject_a_push_while_a_loop_walks_a_strings_characters`) | |
| library call or C call during a walk (1066, three messages) | E3020 | `R12/reject_for_each_inside_a_loop_over_log`, `R15/reject_a_c_call_inside_a_walk_a_kept_callback_may_change`, `R15/reject_an_unmarked_c_function_inside_the_walk` | |
| list a generator walks (1067) | E3020 | `R8/reject_a_change_to_a_list_a_stored_generator_walks`, `R6/reject_a_push_to_the_list_a_stored_generator_walks` | |
| removal in a counting loop (1068) | E3020 | `R8/reject_a_removal_inside_a_counting_loop` | |
| a view used after a reshape or an end (1069, seven messages) | E3060 | `R8/reject_a_removal_ends_a_position`, `R10/reject_a_view_from_a_link_used_after_a_call_that_may_end_its_player`, `R12/reject_a_name_opened_from_a_link_used_after_a_call_that_may_end_players`, `R8/reject_a_view_of_text_used_after_a_function_value_renames_its_node`, `R12/reject_a_view_of_a_node_used_after_the_function_value_owning_it_is_replaced`, `R4/reject_a_view_put_in_through_ref_mut_used_after_its_source_grows`, `R8/reject_a_map_insert_ends_a_view_of_its_keys` | the `owns dyn I` form of the row's last case (section 7) |
| change to a temporary would be lost (1070) | E2140 | `R8/reject_a_change_to_a_temporary_that_would_be_lost` | |
| view of a temporary (1071) | E3063 | `R8/reject_returning_a_view_of_a_temporary`, `R8/reject_returning_a_list_of_views_of_a_temporary`, `R8/reject_keeping_a_view_of_a_temporary_past_its_loop_step` | |
| view out of a lock's block (1072) | E3063 | `R8/reject_a_view_returned_out_of_a_lock`, `R8/reject_a_view_returned_out_of_a_read_lock`, `R8/reject_a_view_of_a_guarded_field_returned_out_of_its_lock`, `R8/reject_a_view_of_a_guard_kept_past_its_block` | |
| callee's local put in through `ref mut` (1073) | E3063 | `R4/reject_a_view_of_the_callees_own_local_put_in_through_ref_mut` | |
| kept view outlives its target (1074) | E3063 | `R7/reject_a_sharedref_mut_holder_returned_past_its_target`, `R7/reject_a_list_of_views_returned_past_what_it_views`, `R7/reject_a_struct_holding_a_view_returned_past_its_source`, `R7/reject_a_shared_holder_of_a_view_returned_past_its_text` | |
| value outlives its arena (1075) | E3063 | `R3/reject_a_value_returned_past_the_end_of_its_arena` | |
| result outside its `from` (1076) | E3062 | `R6/reject_a_result_outside_what_its_from_names` | |
| view field of a class, bad `from` (1077, five messages) | E2118 | `R7/reject_a_view_field_of_a_class_without_from`, `R7/reject_from_naming_a_field_that_is_not_let`, `R7/reject_from_naming_a_fixed_array_field`, `R7/reject_from_naming_an_inline_struct_field`, `R6/reject_from_names_a_parameter_and_a_field`, `R7/reject_a_sharedref_mut_field_of_a_class_without_from` | |
| `copy of` what it cannot copy (1078, four messages) | E2116 | `R3/reject_copy_of_a_weakref_link`, `R11/reject_copy_of_a_link`, `R3/reject_copy_of_a_view`, `R3/reject_copy_of_a_value_whose_views_point_into_its_own_let_field`, `R3/reject_copy_of_a_type_with_no_clone` | |
| `@cached` (1079, four messages) | E2117 | `R5/reject_cached_with_another_parameter`, `R5/reject_cached_reading_a_shares_field`, `R5/reject_cached_with_changes`, `R5/reject_cached_reading_a_field_other_threads_change` | |
| `Some(...)` in a header (1080) | E0115 | `R13/reject_some_in_an_if_header`, `R13/reject_some_in_a_while_header` | |
| opening that always holds; kind test that always passes (1081, three messages) | E2036 | `R13/reject_exists_as_on_a_value_that_always_holds_one`, `R13/reject_exists_as_on_a_result`, `R14/reject_a_kind_test_that_always_passes` | |
| kind test that can never pass (1082) | E2037 | `R14/reject_a_kind_test_that_can_never_pass` (new code, U5) | |
| opening or kind test naming a name in use (1083) | E2106 | `R13/reject_an_opened_name_already_in_use`, `R14/reject_a_kind_test_naming_a_name_already_in_use` | |
| name with no word after one with a word (1084, three messages) | E0116 | `R3/reject_a_loop_name_with_no_word_after_one_with_a_word`, `R3/reject_a_captured_name_with_no_word_after_one_with_a_word`, `R3/reject_a_lambda_parameter_with_no_word_after_one_with_a_word`, `R13/reject_an_opened_pair_name_with_no_word_after_one_with_a_word` | |
| `ref mut` in a capture list (1085) | E0120 | `R12/reject_ref_mut_in_a_capture_list` | |
| `weakref` of a part (1086) | E2119 | `R11/reject_a_weakref_of_a_struct_field_of_an_object` (new in step 4), `R11/reject_a_weakref_of_a_struct_stored_inline_in_a_list` | |
| `owns` missing on the making line (1087, four messages) | E2123 | `R3/reject_a_value_made_for_an_owns_field_without_owns`, `R3/reject_a_value_made_inside_some_for_an_option_of_owns_without_owns`, `R3/reject_a_value_pushed_into_a_list_of_owns_values_without_owns`, `R3/reject_a_new_value_made_later_for_an_owns_field_without_owns` | |
| type containing itself (1088) | E2200 | `R3/reject_a_type_that_contains_itself_with_no_block_between`, `R3/reject_a_struct_holding_a_view_of_its_own_type`, `R11/reject_a_struct_node_keeping_a_view_of_its_parent` | |
| `owns` before an existing name or before `fn` (1089, four messages) | E0109 | `R3/reject_owns_before_a_value_its_place_already_hands_over`, `R3/reject_owns_before_a_returned_value`, `R4/reject_owns_written_before_an_argument`, `R3/reject_owns_before_fn` | the number form, "`b = n` gives `b` the number; remove `owns`" (section 7) |
| type word that cannot apply (1090) | E2114 | `R3/reject_owns_on_a_class_type`, `R3/reject_using_after_a_type_with_no_heap_memory` | |
| bare interface name (1091) | E2173 | `R14/reject_a_bare_interface_name_as_a_type`, `R3/reject_a_bare_interface_name_used_as_a_type` (wrong, section 4) | |
| lambda parameter mode; link used unopened; object into a link (1092, four messages) | E2020 | `R12/reject_a_lambda_that_takes_over_what_each_name_lends`, `R12/reject_a_plain_lambda_parameter_where_each_name_hands_the_name_over`, `R13/reject_a_weakref_link_used_as_its_target_without_opening_it`, `R11/reject_a_link_used_as_its_target_without_opening`, `R3/reject_an_object_stored_into_a_weakref_place_without_weakref`, `R11/reject_an_object_stored_into_a_link_without_weakref` | |
| value from another memory manager (1093, two messages) | E2020 | `R3/reject_a_value_from_the_built_in_manager_given_to_an_arena_place`, `R3/reject_a_value_from_another_kind_of_arena_given_to_an_arena_place` | |
| `no_callbacks` on a callback-taking function (1094) | E5069 | `R15/reject_no_callbacks_on_a_function_that_takes_a_callback` | |
| function value outliving what it uses (1095, four messages) | E3026 | `R12/reject_a_handler_that_outlives_the_callers_total`, `R12/reject_a_returned_function_that_changes_its_makers_local`, `R15/reject_a_task_uses_the_loop_name_without_a_copy`, `R15/reject_a_thread_given_the_loops_text_with_ref`, `R12/reject_a_handler_kept_past_the_loop_name_it_uses`, `R15/reject_a_round_local_used_by_a_task_without_a_copy` | |
| hand-over from a non-owner (1096, four messages) | E3013 | `R1/reject_a_plain_parameter_pushed_into_a_list`, `R12/reject_a_function_that_gives_s_away_passed_to_call_twice`, `R12/reject_a_plain_once_fn_parameter`, `R2/reject_a_field_taken_out_of_a_class_object` | |
| `virtual` in a closed class (1097) | E2112 | `R5/reject_virtual_in_a_class_that_cannot_be_subclassed` | |
| method no bound declares (1098) | E2072 | `R14/reject_a_generic_body_calling_a_method_no_interface_declares` | |
| wait for a held lock (1099, three messages) | E7018 | `R15/reject_a_join_inside_the_lock_its_thread_needs`, `R15/reject_a_call_through_a_function_value_that_may_take_the_held_lock`, `R15/reject_a_walk_under_the_lock_across_a_function_value_that_changes_log`, `R15/reject_a_scope_that_ends_inside_the_lock_its_task_needs` | |
| `thread_shared` on a value not made so (1100) | E7017 | `R3/reject_thread_shared_on_a_value_not_made_thread_shared`, `R15/reject_thread_shared_from_a_plain_parameter` | |
| `thread_shared` value changed directly (1101) | E7014 | `R9/reject_a_thread_shared_object_changed_directly` | |
| thread or kept callback, name with no word (1102, two messages) | E7004 | `R15/reject_an_unscoped_thread_using_a_name_with_no_capture_word`, `R15/reject_a_thread_started_in_a_walk_uses_the_loop_name_without_a_copy`, `R15/reject_a_kept_callback_using_a_name_with_no_capture_word` | |
| one-thread owner or lent value reaching another thread (1103, three messages) | E7015 | `R12/reject_a_shares_mut_owner_held_by_a_function_any_thread_may_run`, `R12/reject_a_bus_with_plain_fn_handlers_made_thread_shared`, `R15/reject_a_callback_c_may_run_on_any_thread_holding_a_shares_owner` | |
| two tasks change one place (1104) | E7007 | `R15/reject_a_task_reads_a_field_another_task_adds_to` | |
| thread-bound value sent (1105) | E7008 | `R15/reject_a_lock_guard_sent_to_a_task` | |
| parallel chunks writing one place (1106) | E7010 | `R15/reject_a_parallel_loop_over_shares_mut_objects` | |
| main-thread module variable from another thread (1107) | E7012 | `R15/reject_a_thread_changes_an_unguarded_static_mut` | |
| guarded field without its lock (1108) | E7013 | `R15/reject_a_guarded_field_changed_outside_its_lock` | |
| `@parallel` on what it cannot split (1109) | E7016 | `R15/reject_a_parallel_while_loop` | |
| unsafe step outside `unsafe:`, `into_raw()`, hand-written `Send` (1110, four messages) | E3100 | `R16/reject_an_unsafe_operation_in_an_unsafe_fn_body_outside_a_block`, `R16/reject_calling_an_unsafe_fn_outside_unsafe`, `R16/reject_into_raw_outside_unsafe` (new, verdict 91), `R15/reject_implementing_send_without_unsafe` | |
| variadic argument C would promote (1111) | E5013 | `R16/reject_a_narrow_further_argument_c_would_promote` | |
| run-time alias check in `@no_alias_checks` (1112) | E4030 | `R16/reject_an_index_compare_inside_no_alias_checks`, `R16/reject_assert_disjoint_inside_no_alias_checks` | |
| removed words (1113) | E1010, E0104 | 30 E1010 and 5 E0104 tests, one per word, among them `R1/reject_mem_forget_is_not_a_word_of_ember`, `R3/reject_foreign_box_is_now_owns`, `R3/reject_fp_contract_is_now_fp_allow_fma` and `R5/reject_mut_self_on_a_drop_is_not_ember` (both new: step 3's two words with no test), `R14/reject_as_question_is_not_ember`, `R14/reject_as_bang_is_not_ember` | |

## 2. Every code of section 4 has a test

All 70 codes of section 4 (lines 1140-1211) are expected by at least one test. Each of the 65 error codes has
at least one `reject_` test; the 5 warnings cannot have one, since a warned program compiles, and each has a
`compile-pass` or `run-pass` test that expects it (L3001: `R11/accept_a_cycle_of_shares_owners_is_warned`,
`R9/run_a_tree_child_with_two_owners_changed_through_either`; L3015:
`R16/accept_an_unsafe_fn_without_safety_is_warned`; W2190: `R4/accept_an_unused_check_disjoint_result_is_warned`;
W3012: `R16/accept_an_unsafe_block_without_a_safety_note_is_warned`; W3013:
`R16/accept_a_safety_note_on_an_unsafe_fn_line_is_warned`). The codes not in section 3's table have theirs:
E0005 `R15/reject_async_is_reserved`, E1030 `R15/reject_a_second_main_in_one_module`, E2104
`R7/reject_changing_the_let_field_views_point_into`, E2221 `R15/reject_a_view_of_the_frames_own_local_across_yield`.
No test uses a code that section 4 does not list, nor one of the removed codes (line 1213). The count of tests per
code, with one test each, is `target/redesign7/10/codes_table.md`; every refused test by code,
`target/redesign7/10/rejects_by_code.txt`.

## 3. Every check of section 2 that stops the program has a `run_` test with its text

| check (section 2's stop table, lines 1001-1011) | test | stop text expected |
|---|---|---|
| one compare of two indexes (R4, R8 (f)) | `R4/run_the_same_body_given_twice_stops_the_program`; `R8/run_two_ref_mut_names_of_one_list_are_compared_once`; `R8/run_a_ref_mut_name_and_a_read_only_name_of_one_list_are_compared_once` | "`a` and `b` name the same element of `bodies` (index 1); `collide` needs two different bodies", and the same words in the other two programs' names (but see section 4, unsettled U1) |
| `assert_disjoint` (R4) | `R4/run_assert_disjoint_stops_on_one_track` | "`assert_disjoint(d, s)` on line 18: the two spans overlap (`dst_name` and `src_name` name one track)" (line 18 is the call) |
| a count at its maximum (R9) | `R9/run_fail_too_many_owners_of_one_player_stop_the_program` (`#$ count_bits: 8`) | "too many owners of a `Player` (255)": one owner and 254 rounds reach 255, round 255 stops (lines 1013-1018, verdict 96) |
| a weak count at its maximum (R11) | `R11/run_fail_too_many_weakref_links_to_one_player_stop_the_program` (`#$ count_bits: 8`) | "too many `weakref` links to a `Player` (255)": 255 links, round 256 stops |
| the 32-bit count real programs keep | `R9/run_real_programs_keep_32_bit_counts` | runs 300 owners and 300 links to the end; the emitted C holds both 4,294,967,295 texts (`#$ assert-c`) |
| `cast s as Circle` (R14) | `R14/run_fail_a_cast_to_the_wrong_kind_stops_the_program` | "`cast b as Circle` on line 29: `b` is a `Square`, not a `Circle`" (line 29 is the cast) |
| trampoline: a call after the token ended | `R15/run_fail_c_runs_a_callback_after_it_was_unregistered` | "C called the callback registered on line 34 after its token ended; it was unregistered" (line 34 is `register_cb`) |
| trampoline: a `callback(once)` function called a second time | `R15/run_fail_c_calls_a_once_callback_twice` | "C called the callback registered on line 29 a second time; it was registered `callback(once)`, so it may run only once" (the whole text now, step 3's U9) |
| trampoline: a `threads(creator)` callback on another thread | `R15/run_fail_a_creator_callback_called_on_another_thread` (`#$ profiles: debug, development, release`) | "C called the callback registered on line 35 on another thread, but its contract says `threads(creator)`" |
| bounds checks | `R8/run_a_while_loop_index_is_checked`; `R16/run_fail_a_bounds_check_still_runs_inside_no_alias_checks` | "index 1 is out of bounds for a length of 1"; "index 3 is out of bounds for a length of 3" |

The rows of section 2's first table that do not stop the program each have a `run_` or `run-pass` test that
exercises them, among them `R3/run_the_last_of_two_owners_ends_the_value` (counts),
`R10/run_a_view_opened_from_a_link_needs_no_protection_when_nothing_can_end_a_player` (nothing),
`R10/run_a_view_of_a_thread_shared_object_through_a_link_holds_an_atomic_count`,
`R11/run_a_second_link_finds_its_target_gone_after_the_first_link_and_the_target_end` (weak counts),
`R11/run_a_proved_parent_walk_prints_the_path`, `R11/run_a_walk_whose_visit_may_end_nodes_stops_at_the_first_ended_parent`,
`R5/run_a_cached_result_is_cleared_by_a_change_to_what_it_reads`, `R12/run_a_counter_made_by_a_function_keeps_its_variable`,
`R12/run_a_handler_that_grows_the_list_during_dispatch_keeps_its_state` (two words copied out),
`R12/run_a_handler_that_removes_itself_then_uses_its_state` (one count on its state),
`R15/run_a_float_sum_added_more_than_once_per_task_is_a_reduction` (decimal reductions, verdict 92),
`R15/run_c_holds_a_tagged_number_not_an_address` (the tag layout, step 3's U8),
`R4/run_one_variable_given_twice_as_ref_mut_is_one_variable_in_the_call` (one more copy of a function, verdict 88)
and `R3/run_a_value_ends_in_the_arena_instance_it_came_from` (the instance's address).

## 4. Tests whose expected result is not what the rules say

### Wrong (1)

| test | why | rule |
|---|---|---|
| `R3/reject_a_bare_interface_name_used_as_a_type` | The error (E2173 at line 18) is right, but the fix its message and its help give, `Array[owns dyn Shape]`, does not compile in this program: line 21 makes the shape for that list without `owns` (`Scene(shapes=[Square(side=2)])`), which verdict 85 then refuses (E2123: "`shapes=[owns Circle(r=1.0), owns Square(side=2.0)]` for `Array[owns dyn Shape]`"). Its twin `R14/reject_a_bare_interface_name_as_a_type` is right (its making line is `Scene(shapes=[])`). Either make line 21 `Scene(shapes=[])`, or have the message name line 21 too (`shapes=[owns Square(side=2)]`). The rules' own E2173 message (line 1091) gives only the type, so it has the same gap in any program that also makes the shapes. | R18 lines 948-950 ("its fix never runs into another error"); verdict 73; R3 lines 186-195 (verdict 85) |

### Unsettled (3 tests, 1 point)

| point | tests | the rule text that leaves it open |
|---|---|---|
| U1. Two `ref mut` elements of one list whose indexes are not known when compiling, given to one call: one compare before the call, or refused? | `R4/run_the_same_body_given_twice_stops_the_program`, `R4/run_two_bodies_compared_once_at_the_call` (both expect the compare), `R16/reject_an_index_compare_inside_no_alias_checks` (E4030 exists only if the compare does) | R4 lines 292-293 (and section 2, lines 972 and 1003): "Where the two differ only by an index of one list (`bodies[i]`, `bodies[j]`), one compare before the call stops the program if they are equal"; against R4's exception 1, lines 305-306: "Parts whose positions are not known when compiling (`ref mut log[i]` beside `ref mut log[j]`, or beside `log`) stay refused (E3022)". The worklist's row for verdict 88 asks for a `reject_` of `ref mut xs[i]` beside `ref mut xs[j]`; the test writer did not write one and recorded the reading that keeps the compare (`matrix-W2.md` line 204): exception 1's sentence means "not given the one-variable copy", which keeps the compare for `collide`, and E3022 for an element beside its whole list. That reading matches section 2, AN-03 and verdict 88's own reason ("since they may lead to the `LOG[0]` problem", a callee that also reaches the list by its own name, which E3031 refuses). The tests are right under it; lines 305-306 should say so, for example "a part beside its whole list stays refused (E3022); two parts of one list keep the compare above, and a callee that also reaches the list by its own name is checked at the call (E3031)". |

### What happened to step 3's list

- Wrong 1 (`R3/accept_the_first_owner_keeps_what_it_could_do`): void by verdict 86; the test is right as it stands.
- Wrong 2: split into `R14/reject_as_question_is_not_ember` and `R14/reject_as_bang_is_not_ember`, one fault each.
- Wrong 3 to 6: `R16/reject_assert_disjoint_inside_no_alias_checks` imports `assert_disjoint`; the messages of
  `R4/reject_an_ignored_check_disjoint_proves_nothing`, `R4/reject_two_tracks_of_one_map_not_proved_apart` and
  W2190's in `R4/accept_an_unused_check_disjoint_result_is_warned` give the import; so do the rules (lines 1061, 1124).
- Wrong 7: replaced by `R14/reject_a_call_that_drives_a_dyn_pushing_to_log_inside_a_walk_of_log` (E3020, the
  message names `show(...)`, the line's own call).
- U1 settled by verdict 85 (every `owns dyn` list and `Option[owns T]` making line in the suite says `owns`,
  but for the wrong test above); U2 and U3 void by verdict 86; U4 in E3023's row (line 1037); U5 the new E2037;
  U6 in E3027's row (line 1059); U7 verdict 92 (`R15/run_a_float_sum_added_more_than_once_per_task_is_a_reduction`);
  U8 the layout in R15 (lines 901-903); U9 the stop text (line 1009); U10 R9's "other than by a reduction"
  (lines 546-547) and R15 (lines 849-855).

The rest of the suite, 828 tests, expects what the rules say: the code, the line the error stands on, the
stdout, the stop text, and a fix that compiles.

## 5. The safety holes

The six holes of `V5/safety/findings.md` section 1 (6 in its two parts), the two neighbours of hole 3 that step
3's re-check found (its section 6), and hole 6a's original form.

| hole | now | rule (lines) | refused or run tests | allowed neighbours |
|---|---|---|---|---|
| 1. a view given to a function that then clears what it points into | **refused** when compiling (E3031): the compiler works out from the body which view parameters (`ref mut` ones included, verdict 87) the function uses after a line that may end or move what they view, through its callees and through every function the program can put in a function value it calls, and checks each call; across a package's edge the fact is in the interface file; a faulty function stored in a function value is refused where it is stored | R4 314-343; R8 (g) 525-528; R17 934-937; verdicts 53 (corrected), 87 | `R4/reject_a_view_argument_into_what_the_callee_clears_and_then_uses`, `R4/reject_a_view_argument_into_what_a_callee_of_the_callee_clears`, `R4/reject_a_ref_mut_argument_into_what_the_callee_clears`, `R8/reject_a_span_given_to_a_function_that_grows_what_it_shows`, `R17/reject_a_call_from_another_package_hands_a_part_of_what_it_clears`, `R11/reject_a_visit_function_that_reads_its_node_after_ending_it` | `R4/accept_a_view_argument_used_before_the_callee_clears`, `R4/accept_a_copy_given_to_a_function_that_clears_log`, `R4/accept_text_written_in_the_program_given_to_a_function_that_clears_log`, `R4/accept_a_ref_mut_argument_of_your_own_given_to_a_function_that_clears_log`, `R8/accept_a_view_whose_last_use_is_an_argument_does_not_live_across_the_call`, `R4/run_a_module_list_given_to_a_ref_mut_parameter_its_function_also_names` (verdict 88), `R17/accept_a_call_from_another_package_given_what_its_recorded_fact_allows` |
| 2. a method that destroys its own object | **refused** (E3031): `this` counts as a view parameter | R4 330-332; R5 395; verdict 54 | `R5/reject_a_method_called_on_a_part_of_what_it_clears`, `R8/reject_a_method_that_reads_its_object_after_clearing_the_list_holding_it` | `R5/accept_a_method_that_uses_this_before_it_clears_what_owns_it`, `R8/accept_a_method_that_reads_its_object_before_clearing_the_list_holding_it`, `R5/run_a_turret_aimed_at_itself_changes_itself` (a number written, nothing ended) |
| 3. a stored function that owns what a view reads | **refused**: what a function value's call changes and ends is worked out from every function the program stores there, so a view of an object reached through a link does not outlive a call that may end it (E3060), and no walk keeps views across a call that may change nodes (E3021) | R12 662-672, 681-691; R10 564-574, 578-584; verdicts 55, 82, 86 | `R12/reject_a_name_opened_from_a_link_used_after_a_call_that_may_end_players`, `R10/reject_a_view_from_a_link_used_after_a_function_that_ends_the_players_it_owns`, `R10/reject_a_view_from_a_link_used_after_a_call_that_may_change_players`, `R11/reject_a_walk_with_views_across_a_visit_that_may_change_nodes` | `R12/accept_a_function_that_ends_the_players_it_owns`, `R10/accept_a_view_from_a_link_used_before_a_function_that_ends_the_players_it_owns`, `R10/accept_a_view_from_a_link_used_after_a_call_that_changes_nothing`, `R11/accept_a_walk_with_views_when_the_visit_changes_no_node` |
| 3, neighbour 1 (step 3's 6.1): a **change** of an object a function value's state owns, read through a `weakref` view | **refused** (E3060): the change is worked out from the stored function's body, and a view into what it replaces is not used after the call | R8 (b) 463-468; R12 684-687 | `R8/reject_a_view_of_text_used_after_a_function_value_renames_its_node`, `R12/reject_a_view_through_a_link_used_after_a_stored_function_replaces_its_text` | `R8/accept_your_own_text_outlives_a_function_value_that_renames_its_node`, `R12/accept_a_view_through_a_link_used_before_a_stored_function_replaces_its_text` |
| 3, neighbour 2 (step 3's 6.2): the **end** of a function value (or an `owns dyn I` value) whose state owns an object | **refused** (E3060): replacing or ending such a value ends what its state owns, counted as R10's end of an object and as a call of that object's `drop` | R12 687-691; R10 point 1 564-570; R5 367-370; section 4 E3060 (line 1189) | `R10/reject_a_view_of_a_node_used_after_the_function_value_that_owned_it_is_replaced`, `R12/reject_a_view_of_a_node_used_after_the_function_value_owning_it_is_replaced` | `R10/accept_a_view_of_a_node_used_before_the_function_value_that_owned_it_is_replaced`, `R12/accept_a_view_of_a_node_used_before_the_function_value_owning_it_is_replaced`. Not tested: the `owns dyn I` form and the `drop` form (section 7) |
| 4. `drop` and `weakref` | **made safe**: an object is ended before its `drop` runs, so a link opened in it finds it ended and nothing counts it again; ending an object counts as a call of its `drop` (E3020 inside a walk of what it changes); a `drop` cannot keep its object (E3013) | R11 600-605; R5 367-372; verdicts 56, 57 | `R11/run_a_link_opened_during_its_targets_drop_finds_it_ended` (stdout `made`, `ended`), `R11/run_a_childs_drop_finds_its_parent_ended`, `R8/reject_ending_an_object_whose_drop_pushes_to_the_walked_list`, `R8/reject_a_function_that_ends_an_enemy_called_inside_a_walk_of_log`, `R1/reject_a_loop_over_log_calls_a_function_whose_local_ends_with_a_drop_that_pushes_to_log`, `R2/reject_a_loop_over_log_calls_a_function_that_replaces_an_object_whose_drop_pushes_to_log`, `R1/reject_a_drop_method_that_keeps_its_object` | `R8/accept_ending_an_object_whose_drop_pushes_to_log_after_the_walk`, `R8/accept_a_function_that_ends_an_enemy_called_after_a_walk_of_log`, `R1/accept_a_drop_method_that_keeps_a_number_of_its_object`, `R11/run_a_walk_over_nodes_with_a_drop_method_stops_at_the_first_ended_parent` |
| 5. count overflow | **made safe**: `mem.forget` is gone (E1010) and `into_raw()` is only inside `unsafe:` (E3100), so nothing ends an owner without its clean-up outside `unsafe`; every +1 of an owner or weak count is compared with its maximum (a stop with section 2's text). Both stops are now runnable on the laptop: their tests use the real counter at 8 bits (verdict 96) | R1 114-117; R9 553-557; R11 590-594; R16 915-923; section 2 1005-1006, 1013-1018; verdicts 58, 59, 91, 96 | `R1/reject_mem_forget_is_not_a_word_of_ember`, `R16/reject_into_raw_outside_unsafe`, `R9/run_fail_too_many_owners_of_one_player_stop_the_program`, `R11/run_fail_too_many_weakref_links_to_one_player_stop_the_program` | `R1/accept_a_value_ended_now_runs_its_clean_up`, `R16/accept_into_raw_inside_unsafe_gives_c_the_address`, `R16/accept_a_c_function_declared_owned_takes_the_widget_with_no_word`, `R9/run_real_programs_keep_32_bit_counts` |
| 6a. a stored function holding a one-thread owner, run by another thread | **refused** (E7015): a function type says whether any thread may run its functions (`thread_shared fn()`), such a function holds no `shares`, `shares mut` or `weakref` of one, and `thread.spawn` takes one | R12 692-702; R15 840-845; section 3 line 1103; section 4 line 1203; verdicts 61, 94 | `R12/reject_a_shares_mut_owner_held_by_a_function_any_thread_may_run`, `R15/reject_a_handler_any_thread_may_run_holding_a_shares_mut_owner`, `R15/reject_a_thread_takes_a_shares_mut_bus_to_store_a_handler`, `R15/reject_a_thread_captures_a_one_thread_owner` | `R12/accept_a_thread_shared_owner_held_by_a_function_any_thread_may_run`, `R15/run_a_thread_runs_a_handler_holding_a_thread_shared_owner`, `R15/accept_a_thread_stores_a_handler_in_a_bus_threads_share` |
| 6a, its original form (step 3's re-check): a class with a plain `fn()` field made `thread_shared` | **refused** (E7015, at the line that makes the object `thread_shared`), with the lines that make the field `thread_shared fn()` | R12 698-702; section 3 line 1103 (second message); section 4 line 1203 | `R12/reject_a_bus_with_plain_fn_handlers_made_thread_shared` | `R12/accept_a_bus_with_thread_shared_fn_handlers_made_thread_shared` |
| 6b. a C callback that ends its own registration while it runs | **made safe**: C forgets it at once; its block, where its captures live, is freed when it returns | R12 770-775; R15 876-877; section 2 line 987; verdict 62 | `R15/run_a_kept_callback_that_ends_its_own_token_then_uses_its_state` (stdout `1`), `R15/run_fail_c_runs_a_callback_after_it_was_unregistered` | `R15/run_c_runs_a_kept_callback_through_the_trampoline` |

## 6. Found on the way: a hole next to 6a

**A class object that `weakref` links on this thread point at, handed whole to another thread.** The rules
refuse a function any thread may run that holds a one-thread owner or a `weakref` link of one (R12 lines
695-697, E7015), and the vocabulary's `Send` row (`V5/vocabulary.md` line 271) says a value handed over may go
to another thread "when nothing reachable from it is a one-thread owner, a `weakref` link of one, or bound to
one thread". Both look at what the handed value **holds**. Neither looks at links that point **at** it. A class
object has a plain (one-thread) owner count and weak count in its header (R9 lines 550-553), and only a
`thread_shared` object's view through a link holds an atomic count (R10 point 2, lines 575-577). So the rules
as written accept:

```ember
class Names:
    items: Array[String] = []

fn main():
    names = Names(items=[String("ann")])
    w = weakref names                                      # a link on this thread
    h = thread.spawn(fn() owns names: names.items.len())   # the object goes to another thread
    if w exists as p:                                      # tests the header the other thread may be ending
        println(p.items.len())                             # may read `items` after the other thread freed it
    h.join()
```

`if w exists as p:` reads the owner count while the other thread may set it to zero and run the object's end;
`p.items` may then be freed text (the block is kept by the weak count, its contents are not). Rust cannot
write this program: an object a `Weak` points at lives in an `Rc`, which is not `Send`. AN-71 already gives the
answer (`V5/area-E.md` lines 490-493): "The same for a class that is the target of a `weakref` field anywhere
in the build (its `weakref` links stay on this thread)", E7015, with the line `names = thread_shared Names()`.
What is missing is that sentence in the rules (R12's `thread_shared fn()` point or R15's capture point, and
E7015's row and meaning, lines 1103 and 1203) and a `reject_`/`accept_` pair. No verdict is needed: AN-71 says
what is refused and with which code. It belongs to the rule writer (R12, R15) and test group T4 (R15).

A related sentence the rules have only in general: a `thread_shared` value whose class holds a one-thread
owner (`class Names:` with `cache: shares Cache`, made `thread_shared Names(cache=shares c)`) reaches another
thread with that owner inside it. R15's "A one-thread owner or a lent value reaching another thread is E7015"
(lines 892-893) covers it, and `ThreadShared` is inferred "as Rust infers `Sync`" (vocabulary line 272), but no
row of section 3 says where the error stands and no test shows it; hole 6a's form was written for `fn()`
fields only (R12 lines 698-702).

## 7. Refusals and run-time checks without a test

Every row of section 3 has a test (section 1), and every stop of section 2 (section 3). These cases inside a
row have none:

1. **E3031 for a `ref mut` argument the callee also reaches by its own name** (R4 lines 336-339; section 3,
   line 1062, fourth message): with `fn heal(ref mut p: Player):` whose body is `println(PARTY[0].hp)` then
   `p.hp += 10`, `heal(ref mut PARTY[0])` is refused; without it every `ref mut` parameter is compiled with
   `restrict` while a second pointer reaches the same player (undefined behaviour in C; `V5/changes.md`
   section 8 item 5). `matrix-W2.md` row R4-14 lists it as "no test here". It needs a `reject_` and the
   `accept_` of the message's line (`println(p.hp)` in `heal`). Group T1 (R4).
2. **E0109 before a number name** (section 3, line 1089, third message; `V5/changes.md` section 8 item 2):
   `b = owns n` for an `int` (the message "`b = n` gives `b` the number; remove `owns`"). E2085, which refused
   `owns` on a number before verdict 84, is gone, so this is the one line that still refuses `owns` before a
   number name. Group T1 (R3).
3. **E3060 after an `owns dyn I` value whose state owns an object is replaced or ended** (R12 line 691; R5
   lines 368-369; section 3 line 1069's case list): only the function-value form is tested. Group T3 (R12) or
   T2 (R10).
4. **A `drop` run by the end of a function value's state, inside a walk of what that `drop` changes** (R12
   lines 689-691: "that node's `drop` counts as called there (inside a walk of what it changes, E3020)"):
   step 3's re-check named this case (its section 6.2, last paragraph); the `drop` is tested only through
   lists cleared and locals ended. Group T3 (R12).
5. Not a row of its own, still missing from step 3's list: hole 2 reached through a link, a method called on a
   name opened from a `weakref` whose body clears the list that owns the object (R8 (b) lines 461-463 make the
   two paths one place; R4 lines 330-332 refuse it, E3031).

## 8. Notes for the rule writer (no test is wrong because of them)

- R8 (f) (lines 505-509), read word for word, refuses `s = tracks[src_name].as_span()` while
  `d = ref mut tracks[dst_name].as_mut_span()` is still used (a read of a place `d` may name); R4 (lines 290-296)
  and section 3's E3021 message for the same program (line 1061, "`assert_disjoint(d, s)` before line 12") put the
  check at the call. The six tests of AN-14's `mix` follow R4 (`R4/accept_two_tracks_proved_apart_by_assert_disjoint`,
  `R4/accept_check_disjoint_proves_apart_in_its_ok_branch`, `R4/run_assert_disjoint_stops_on_one_track`,
  `R4/reject_two_tracks_of_one_map_not_proved_apart`, `R4/reject_an_ignored_check_disjoint_proves_nothing`,
  `R16/reject_assert_disjoint_inside_no_alias_checks`); R8 (f) could say that two spans of one list are checked
  where they meet in a call (R4).
- R3 (lines 186-195) lists where `owns` is required on the making line (an `owns` field, an `Option` or a list of
  `owns` values, a value made later for one); a local or a parameter typed `owns T` or `owns dyn I` is not named.
  The tests write it for a local (`R14/run_fail_a_cast_to_the_wrong_kind_stops_the_program`,
  `a: owns dyn Shape = owns Circle(r=3)`, as AN-66 does, `V5/area-D.md` line 2193) and not for an argument made in
  the call for an `owns it: owns dyn Iterator[...]` parameter (`show(xs.iter().inspect(...))` in four R12 and R14
  tests, `add_each(xs.iter())` in two R14 tests), which R4's "an argument for an `owns` parameter is handed over
  with no word" (lines 284-285) allows. No test depends on the other choice; one sentence in R3 would settle both.
- Section 3's E2173 message (line 1091) should add that each value made for the new `owns dyn` list says `owns`
  (the wrong test of section 4).
- R4 lines 305-306: the sentence of section 4's U1.

## 9. For the owner

Nothing. The one point with two readings (U1) is settled by verdict 88's own reason and section 2; the hole of
section 6 is settled by AN-71's words; the rest are missing tests and sentences.

## Files

- This file (new). Scratch, under `target/redesign7/10/`: `index.py` (format and kinds, `index.json`),
  `codes.py` and `codes_table.md` (codes of section 4 against the tests), `rejects_by_code.txt`, `lines.py` and
  `lines.txt` (every line a message names), `dump.py` (the tests as read, with line numbers), `notes.md` (the
  notes taken while reading), `recheck_step3_as_found.md` (step 3's file as this step found it).
