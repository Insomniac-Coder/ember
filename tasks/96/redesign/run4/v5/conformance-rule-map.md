# The design tests mapped to spec 0.9.15_Hardened_1's rule IDs

841 files: 410 precise (by error code), 84 by their refused twin, 347 by their design rules' main spec ID (to refine when the tests move into tests/conformance).

| test | design rules | spec IDs | how |
|---|---|---|---|
| `R1/accept_a_child_set_before_the_push_and_changed_in_the_list.em` | R1, R2 | OWN-1, BRW-12 | main |
| `R1/accept_a_copy_of_a_copy_struct_is_changed_on_its_own.em` | R1, R3 | OWN-1, OWN-8 | main |
| `R1/accept_a_copy_of_a_list_stays_as_it_was.em` | R1, R3 | OWN-1, OWN-8 | main |
| `R1/accept_a_copy_of_the_template_pushed_each_round.em` | R1, R3 | OWN-1, OWN-8 | main |
| `R1/accept_a_copy_on_one_branch_keeps_the_list.em` | R1, R3 | OWN-1, OWN-8 | main |
| `R1/accept_a_copy_struct_whose_source_is_not_used_again_may_change.em` | R1 | OWN-1 | main |
| `R1/accept_a_drop_method_that_keeps_a_number_of_its_object.em` | R1, R5 | OWN-1, FN-4 | main |
| `R1/accept_a_function_whose_local_ends_with_a_drop_called_after_the_loop.em` | R1, R5, R8 | OWN-1, FN-4, BRW-16 | main |
| `R1/accept_a_handed_over_list_is_used_through_its_new_name.em` | R1 | OWN-1 | main |
| `R1/accept_a_handed_over_object_is_ended_once_by_its_new_owner.em` | R1 | OWN-1 | main |
| `R1/accept_a_name_used_before_mem_drop.em` | R1 | OWN-1 | main |
| `R1/accept_a_number_given_with_equals_is_copied.em` | R1 | OWN-1 | main |
| `R1/accept_a_value_ended_now_runs_its_clean_up.em` | R1 | OWN-1 | main |
| `R1/accept_a_view_given_with_equals_hands_nothing_over.em` | R1 | OWN-1 | main |
| `R1/accept_an_object_read_before_it_is_handed_over.em` | R1, R4 | OWN-1, FN-1 | main |
| `R1/accept_ref_mut_given_with_equals_changes_through_the_new_name.em` | R1, R2 | OWN-1, BRW-12 | main |
| `R1/accept_ref_mut_to_a_copy_struct_changes_the_original.em` | R1, R2 | OWN-1, BRW-12 | main |
| `R1/reject_a_change_through_a_view_given_with_equals.em` | R1, R5 | BRW-15 | precise |
| `R1/reject_a_copy_struct_copied_with_equals_then_changed.em` | R1 | BRW-14, BRW-15 | precise |
| `R1/reject_a_drop_method_that_keeps_its_object.em` | R1, R5 | OWN-11, EXP-6, CLO-6 | precise |
| `R1/reject_a_field_set_on_a_child_after_it_was_pushed.em` | R1, R2 | OWN-3 | precise |
| `R1/reject_a_list_used_after_a_hand_over_on_one_branch.em` | R1 | OWN-3 | precise |
| `R1/reject_a_list_used_after_it_was_handed_over.em` | R1 | OWN-3 | precise |
| `R1/reject_a_loop_over_log_calls_a_function_whose_local_ends_with_a_drop_that_pushes_to_log.em` | R1, R5, R8 | BRW-18, CTL-2 | precise |
| `R1/reject_a_name_used_after_mem_drop.em` | R1 | OWN-3 | precise |
| `R1/reject_a_plain_parameter_pushed_into_a_list.em` | R1, R2, R4 | OWN-11 | precise |
| `R1/reject_an_object_read_after_it_was_handed_to_an_owns_parameter.em` | R1, R4 | OWN-3 | precise |
| `R1/reject_an_owned_object_handed_over_in_every_round_of_a_loop.em` | R1, R2 | OWN-3 | precise |
| `R1/reject_mem_forget_is_not_a_word_of_ember.em` | R1 | MOD-3 | precise |
| `R10/accept_a_link_opened_again_after_a_call_that_ended_its_player_finds_it_gone.em` | R10, R11, R13 | BRW-17, LT-3 | twin |
| `R10/accept_a_view_from_a_link_used_after_a_call_that_changes_nothing.em` | R10, R8, R12 | BRW-17 | twin |
| `R10/accept_a_view_from_a_link_used_before_a_function_that_ends_the_players_it_owns.em` | R10, R12, R13 | BRW-17, LT-3 | twin |
| `R10/accept_a_view_from_a_link_used_before_a_loop_of_calls_that_end_its_player.em` | R10, R11, R13 | BRW-17, LT-3 | twin |
| `R10/accept_a_view_of_a_node_used_before_the_function_value_that_owned_it_is_replaced.em` | R10, R12, R5 | BRW-17, LT-3 | twin |
| `R10/reject_a_view_from_a_link_used_after_a_call_that_may_change_players.em` | R10, R8, R12, R18 | BRW-17 | precise |
| `R10/reject_a_view_from_a_link_used_after_a_call_that_may_end_its_player.em` | R10, R13, R18 | BRW-17, LT-3 | precise |
| `R10/reject_a_view_from_a_link_used_after_a_function_that_ends_the_players_it_owns.em` | R10, R12, R18 | BRW-17, LT-3 | precise |
| `R10/reject_a_view_from_a_link_used_after_a_loop_of_calls_that_may_end_players.em` | R10, R13, R18 | BRW-17, LT-3 | precise |
| `R10/reject_a_view_of_a_node_used_after_the_function_value_that_owned_it_is_replaced.em` | R10, R12, R5 | BRW-17, LT-3 | precise |
| `R10/run_a_view_of_a_thread_shared_object_through_a_link_holds_an_atomic_count.em` | R10, R11, R9 | RC-5, WK-11, RC-1 | main |
| `R10/run_a_view_opened_from_a_link_needs_no_protection_when_nothing_can_end_a_player.em` | R10, R11, R13 | RC-5, WK-11, GRM-44 | main |
| `R11/accept_a_copy_of_a_links_target.em` | R11, R3 | OWN-8 | twin |
| `R11/accept_a_cycle_of_shares_owners_is_warned.em` | R11 | WK-11 | main |
| `R11/accept_a_link_made_from_a_read_only_name_changes_nothing.em` | R11, R5 | WK-11, FN-4 | main |
| `R11/accept_a_link_opened_before_its_target_is_used.em` | R11, R13 | WK-11, GRM-44 | main |
| `R11/accept_a_parent_link_set_by_the_program.em` | R11 | OWN-11, EXP-6, CLO-6, TYP-14 | twin |
| `R11/accept_a_tree_with_parent_links_gets_no_cycle_warning.em` | R11 | WK-11 | main |
| `R11/accept_a_walk_with_views_when_the_visit_changes_no_node.em` | R11, R10, R12 | WK-11, RC-5, CLO-1 | main |
| `R11/accept_a_weakref_of_a_class_object_in_a_list.em` | R11, R8 | WK-16 | twin |
| `R11/accept_a_weakref_of_the_object_that_holds_the_field.em` | R11, R13 | WK-16 | twin |
| `R11/accept_an_object_stored_into_a_link_with_weakref.em` | R11, R1 | WK-11, OWN-1 | main |
| `R11/accept_another_link_to_the_same_target.em` | R11, R3 | OWN-8 | twin |
| `R11/reject_a_link_used_as_its_target_without_opening.em` | R11, R13 | TYP-4, CLO-3 | precise |
| `R11/reject_a_parent_field_that_owns_its_parent.em` | R11, R1 | OWN-11, EXP-6, CLO-6 | precise |
| `R11/reject_a_struct_node_keeping_a_view_of_its_parent.em` | R11, R7 | TYP-14 | precise |
| `R11/reject_a_visit_function_that_reads_its_node_after_ending_it.em` | R11, R4, R12 | BRW-21, FFI-52 | precise |
| `R11/reject_a_walk_with_views_across_a_visit_that_may_change_nodes.em` | R11, R8, R10, R12 | BRW-1 | precise |
| `R11/reject_a_weakref_of_a_struct_field_of_an_object.em` | R11 | WK-16 | precise |
| `R11/reject_a_weakref_of_a_struct_stored_inline_in_a_list.em` | R11 | WK-16 | precise |
| `R11/reject_an_object_stored_into_a_link_without_weakref.em` | R11, R1 | TYP-4, CLO-3 | precise |
| `R11/reject_copy_of_a_link.em` | R11, R3 | OWN-8 | precise |
| `R11/reject_upgrade_is_not_ember.em` | R11, R13 | MOD-3 | precise |
| `R11/run_a_childs_drop_finds_its_parent_ended.em` | R11, R13, R5 | WK-11, GRM-44, FN-4 | main |
| `R11/run_a_copy_of_a_struct_holding_links_adds_an_owner_and_a_link.em` | R11, R3, R9 | WK-11, OWN-8, RC-1 | main |
| `R11/run_a_link_opened_during_its_targets_drop_finds_it_ended.em` | R11, R13, R5 | WK-11, GRM-44, FN-4 | main |
| `R11/run_a_parent_link_that_does_not_follow_ownership_finds_its_target_gone.em` | R11, R13 | WK-11, GRM-44 | main |
| `R11/run_a_proved_parent_walk_prints_the_path.em` | R11, R13 | WK-11, GRM-44 | main |
| `R11/run_a_second_link_finds_its_target_gone_after_the_first_link_and_the_target_end.em` | R11, R13 | WK-11, GRM-44 | main |
| `R11/run_a_walk_follows_a_node_moved_under_another_parent.em` | R11, R12, R13 | WK-11, CLO-1, GRM-44 | main |
| `R11/run_a_walk_over_nodes_with_a_drop_method_stops_at_the_first_ended_parent.em` | R11, R10, R12, R13 | WK-11, RC-5, CLO-1, GRM-44 | main |
| `R11/run_a_walk_whose_visit_may_end_nodes_stops_at_the_first_ended_parent.em` | R11, R10, R12, R13 | WK-11, RC-5, CLO-1, GRM-44 | main |
| `R11/run_an_empty_link_opens_to_its_else_branch.em` | R11, R13 | WK-11, GRM-44 | main |
| `R11/run_fail_too_many_weakref_links_to_one_player_stop_the_program.em` | R11 | WK-11 | main |
| `R12/accept_a_bus_with_thread_shared_fn_handlers_made_thread_shared.em` | R12, R15 | THR-1, THR-2, THR-10, CLO-16, FFI-22 | twin |
| `R12/accept_a_button_keeps_a_score_handler_and_a_log_handler.em` | R12, R5 | CLO-1, FN-4 | main |
| `R12/accept_a_button_of_another_module_clicked_after_the_loop_over_log.em` | R12, R17 | BRW-18, CTL-2 | twin |
| `R12/accept_a_function_that_ends_the_players_it_owns.em` | R12, R5, R13 | BRW-17, LT-3 | twin |
| `R12/accept_a_handler_called_by_position_adds_one_and_removes_itself.em` | R12, R8 | BRW-18, CTL-2 | twin |
| `R12/accept_a_handler_that_changes_the_ui_through_its_link_walked_with_for.em` | R12, R8, R11, R13 | BRW-18, CTL-2 | twin |
| `R12/accept_a_handler_that_changes_two_variables_it_uses.em` | R12, R5 | CLO-1, FN-4 | main |
| `R12/accept_a_lambda_changes_an_atomic_it_uses.em` | R12, R5 | CLO-1, FN-4 | main |
| `R12/accept_a_lambda_changes_the_real_variable.em` | R12, R3 | GRM-42, CLO-2 | twin |
| `R12/accept_a_lambda_keeps_a_copy_of_what_each_name_lends.em` | R12, R4 | CLO-3 | twin |
| `R12/accept_a_lambda_takes_the_name_it_uses_with_owns.em` | R12, R3 | MOD-3 | twin |
| `R12/accept_a_lambda_that_only_reads_a_variable_it_uses.em` | R12 | CLO-1 | main |
| `R12/accept_a_lambda_that_pushes_to_log_called_after_the_loop.em` | R12, R5, R8 | BRW-18, CTL-2 | twin |
| `R12/accept_a_local_function_that_changes_a_mutex_passed_to_spawn_by_name.em` | R12, R5, R15 | CLO-1, FN-4, THR-1 | main |
| `R12/accept_a_local_list_of_functions_that_change_score.em` | R12, R8 | CLO-1, BRW-16 | main |
| `R12/accept_a_name_opened_from_a_link_used_before_the_call_that_may_end_players.em` | R12, R10, R13 | BRW-17, LT-3 | twin |
| `R12/accept_a_plain_loop_name_calls_a_function_with_no_state_of_its_own.em` | R12, R5 | BRW-15 | twin |
| `R12/accept_a_programs_own_function_takes_its_function_as_a_template_parameter.em` | R12, R5 | CLO-1, FN-4 | main |
| `R12/accept_a_ref_mut_name_from_get_mut_calls_the_handler.em` | R12, R2, R8 | BRW-14, BRW-15 | twin |
| `R12/accept_a_returned_function_owns_the_variable_it_keeps.em` | R12, R3 | CLO-4 | twin |
| `R12/accept_a_thread_shared_owner_held_by_a_function_any_thread_may_run.em` | R12, R15 | THR-1, THR-2, THR-10, CLO-16, FFI-22 | twin |
| `R12/accept_a_view_of_a_node_used_before_the_function_value_owning_it_is_replaced.em` | R12, R10, R11, R13 | BRW-17, LT-3 | twin |
| `R12/accept_a_view_through_a_link_used_before_a_stored_function_replaces_its_text.em` | R12, R8, R10 | BRW-17 | twin |
| `R12/accept_a_weakref_handler_changes_the_ui_and_a_copy_keeps_the_count.em` | R12, R3, R11, R13 | BRW-1 | twin |
| `R12/accept_add_line_changes_a_guarded_field_through_a_plain_parameter.em` | R12, R4, R5, R15 | CLO-1, FN-1, FN-4, THR-1 | main |
| `R12/accept_an_arrow_lambda_changes_a_variable_it_uses.em` | R12, R5 | CLO-1, FN-4 | main |
| `R12/accept_call_once_holds_an_owns_once_fn_and_call_twice_hands_over_copies.em` | R12, R4 | CLO-1, FN-1 | main |
| `R12/accept_clear_the_handlers_after_the_click.em` | R12 | BRW-11 | twin |
| `R12/accept_click_walks_stateful_handlers_with_ref_mut.em` | R12, R5 | BRW-15 | twin |
| `R12/accept_clicking_after_the_loop_over_the_list_a_handler_changes.em` | R12, R8 | BRW-18, CTL-2 | twin |
| `R12/accept_clicking_plain_handlers_inside_a_loop_over_log.em` | R12, R8 | CLO-1, BRW-16 | main |
| `R12/accept_driving_a_dyn_iterator_that_pushes_to_log_after_the_loop.em` | R12, R14 | BRW-18, CTL-2 | twin |
| `R12/accept_each_name_hands_each_name_to_a_lambda_that_owns_it.em` | R12, R4, R5 | CLO-3, BRW-18, CTL-2 | twin |
| `R12/accept_for_each_after_the_loop_over_log.em` | R12, R8 | BRW-18, CTL-2 | twin |
| `R12/accept_inspect_keeps_the_function_and_count_runs_it.em` | R12 | CLO-1 | main |
| `R12/accept_main_starts_threads_that_push_to_results.em` | R12, R5, R15 | CLO-1, FN-4, THR-1 | main |
| `R12/accept_named_functions_kept_in_a_list_of_fn.em` | R12, R5 | CLO-1, FN-4 | main |
| `R12/accept_press_after_the_loop_over_the_list_its_handler_changes.em` | R12, R5, R8 | BRW-18, CTL-2 | twin |
| `R12/accept_read_only_handlers_walked_with_for.em` | R12, R8 | CLO-1, BRW-16 | main |
| `R12/accept_report_clicks_after_the_loop_over_log.em` | R12, R5, R8 | BRW-18, CTL-2 | twin |
| `R12/accept_retain_then_push_the_notes.em` | R12, R4 | BRW-11 | twin |
| `R12/accept_row_6_handler_changes_the_real_score.em` | R12, R5, R8 | CLO-1, FN-4, BRW-16 | main |
| `R12/accept_run_takes_the_stateful_callback_ref_mut.em` | R12, R4, R3 | BRW-14, BRW-15 | twin |
| `R12/accept_setup_stores_a_log_handler_inside_a_loop_over_log.em` | R12, R5, R8 | BRW-18, CTL-2 | twin |
| `R12/accept_the_function_value_called_after_the_with_block.em` | R12, R8, R15 | THR-19 | twin |
| `R12/accept_the_handler_owns_a_share_of_a_counter.em` | R12, R3, R4, R9 | CLO-4 | twin |
| `R12/accept_the_handler_takes_a_copy_of_the_loop_name.em` | R12, R3 | CLO-4, GRM-2 | twin |
| `R12/accept_the_stored_function_called_after_the_loop_as_many_times.em` | R12, R8 | BRW-18, CTL-2 | twin |
| `R12/accept_tick_after_the_loop_over_log_may_run_the_callback_c_keeps.em` | R12, R5, R15 | BRW-18, CTL-2 | twin |
| `R12/accept_two_threads_push_to_one_log.em` | R12, R5, R15 | CLO-1, FN-4, THR-1 | main |
| `R12/accept_update_takes_a_death_handler_type.em` | R12, R4 | CLO-1, FN-1 | main |
| `R12/accept_views_used_before_the_call_that_changes_their_place.em` | R12, R8 | BRW-1 | twin |
| `R12/reject_a_bus_with_plain_fn_handlers_made_thread_shared.em` | R12, R15 | THR-1, THR-2, THR-10, CLO-16, FFI-22 | precise |
| `R12/reject_a_for_walk_of_handlers_given_the_ui_ref_mut.em` | R12, R8 | BRW-18, CTL-2 | precise |
| `R12/reject_a_for_walk_of_handlers_one_of_which_clears_them_through_a_weakref.em` | R12, R8, R11, R13 | BRW-18, CTL-2 | precise |
| `R12/reject_a_function_that_gives_s_away_passed_to_call_twice.em` | R12 | CLO-6 | precise |
| `R12/reject_a_function_type_written_with_changes_log.em` | R12, R3 | GRM-2 | precise |
| `R12/reject_a_function_value_that_changes_log_called_in_a_walk_of_log_lines.em` | R12, R15, R18 | THR-19 | precise |
| `R12/reject_a_handler_kept_past_the_loop_name_it_uses.em` | R12, R2 | CLO-4 | precise |
| `R12/reject_a_handler_stored_in_the_button_it_changes.em` | R12, R4 | BRW-11 | precise |
| `R12/reject_a_handler_that_outlives_the_callers_total.em` | R12, R4 | CLO-4 | precise |
| `R12/reject_a_lambda_that_pushes_to_log_called_inside_a_loop_over_log.em` | R12, R5, R8 | BRW-18, CTL-2 | precise |
| `R12/reject_a_lambda_that_takes_over_what_each_name_lends.em` | R12, R4 | CLO-3 | precise |
| `R12/reject_a_name_opened_from_a_link_used_after_a_call_that_may_end_players.em` | R12, R10, R13 | BRW-17, LT-3 | precise |
| `R12/reject_a_once_fn_called_twice.em` | R12, R1 | OWN-3 | precise |
| `R12/reject_a_plain_lambda_parameter_where_each_name_hands_the_name_over.em` | R12, R4 | CLO-3 | precise |
| `R12/reject_a_plain_name_from_get_mut_calls_the_handler.em` | R12, R2, R8 | BRW-14, BRW-15 | precise |
| `R12/reject_a_plain_once_fn_parameter.em` | R12, R4 | CLO-6 | precise |
| `R12/reject_a_read_only_view_across_a_call_that_changes_what_it_reads.em` | R12, R8 | BRW-1 | precise |
| `R12/reject_a_ref_mut_name_across_a_call_that_changes_its_place.em` | R12, R8 | BRW-1 | precise |
| `R12/reject_a_returned_function_that_changes_its_makers_local.em` | R12 | CLO-4 | precise |
| `R12/reject_a_shares_mut_owner_held_by_a_function_any_thread_may_run.em` | R12, R15 | THR-1, THR-2, THR-10, CLO-16, FFI-22 | precise |
| `R12/reject_a_stateful_callback_given_to_a_read_only_parameter.em` | R12, R4 | BRW-14, BRW-15 | precise |
| `R12/reject_a_stored_function_that_pushes_to_the_walked_list_called_in_the_loop.em` | R12, R8 | BRW-18, CTL-2 | precise |
| `R12/reject_a_view_of_a_node_owned_by_a_replaced_dyn_value.em` | R12, R10 | BRW-17, LT-3 | precise |
| `R12/reject_a_view_of_a_node_used_after_the_function_value_owning_it_is_replaced.em` | R12, R10, R5 | BRW-17, LT-3 | precise |
| `R12/reject_a_view_of_the_ui_across_a_handler_that_changes_it_through_a_weakref.em` | R12, R8, R11, R13 | BRW-1 | precise |
| `R12/reject_a_view_through_a_link_used_after_a_stored_function_replaces_its_text.em` | R12, R8, R10 | BRW-17 | precise |
| `R12/reject_an_arrow_lambda_written_with_changes_removed.em` | R12, R5 | GRM-2 | precise |
| `R12/reject_call_twice_given_a_lambda_that_hands_over_the_real_s.em` | R12, R2 | OWN-11, CLO-6 | precise |
| `R12/reject_click_walks_stateful_handlers_with_a_plain_name.em` | R12, R5 | BRW-15 | precise |
| `R12/reject_clicking_a_button_of_another_module_inside_a_loop_over_log.em` | R12, R8, R17 | BRW-18, CTL-2 | precise |
| `R12/reject_clicking_a_log_button_inside_a_loop_over_log.em` | R12, R8 | BRW-18, CTL-2 | precise |
| `R12/reject_clicking_inside_a_loop_over_a_list_a_handler_changes.em` | R12, R8 | BRW-18, CTL-2 | precise |
| `R12/reject_driving_a_dyn_iterator_that_pushes_to_log_inside_a_loop_over_log.em` | R12, R8, R14 | BRW-18, CTL-2 | precise |
| `R12/reject_each_name_inside_a_loop_over_the_list_its_function_pushes_to.em` | R12, R5, R8 | BRW-18, CTL-2 | precise |
| `R12/reject_for_each_inside_a_loop_over_log.em` | R12, R8 | BRW-18, CTL-2 | precise |
| `R12/reject_inspect_count_inside_a_loop_over_seen.em` | R12, R8 | BRW-18, CTL-2 | precise |
| `R12/reject_owned_fn_names_its_captures.em` | R12, R3 | MOD-3 | precise |
| `R12/reject_press_inside_a_loop_over_the_list_its_handler_changes.em` | R12, R5, R8 | BRW-18, CTL-2 | precise |
| `R12/reject_ref_mut_in_a_capture_list.em` | R12, R3 | GRM-42, CLO-2 | precise |
| `R12/reject_replacing_a_function_whose_node_drop_pushes_to_the_walked_list.em` | R12, R8, R5 | BRW-18, CTL-2 | precise |
| `R12/reject_retain_given_a_function_that_changes_the_list.em` | R12, R4 | BRW-11 | precise |
| `R12/reject_row_6_handler_written_with_changes_score.em` | R12, R5 | GRM-2 | precise |
| `R12/reject_the_handler_name_used_again_after_the_call.em` | R12, R8 | BRW-17 | precise |
| `R12/reject_tick_inside_a_loop_over_log_may_run_the_callback_c_keeps.em` | R12, R5, R8, R15 | BRW-18, CTL-2 | precise |
| `R12/reject_with_before_a_lambdas_captures.em` | R12, R3 | GRM-2 | precise |
| `R12/run_a_counter_made_by_a_function_keeps_its_variable.em` | R12 | CLO-1 | main |
| `R12/run_a_handler_that_grows_the_list_during_dispatch_keeps_its_state.em` | R12, R8 | CLO-1, BRW-16 | main |
| `R12/run_a_handler_that_removes_itself_then_uses_its_state.em` | R12, R8 | CLO-1, BRW-16 | main |
| `R12/run_a_named_copy_is_the_handlers_own_state.em` | R12, R3 | CLO-1, OWN-8 | main |
| `R12/run_a_real_variable_is_seen_by_the_handler_and_back.em` | R12 | CLO-1 | main |
| `R12/widgets.em` | R12 | BRW-18, CTL-2 | twin |
| `R13/accept_a_variant_of_a_result_tested_with_a_pattern.em` | R13 | GRM-44 | main |
| `R13/accept_an_opened_name_that_is_new.em` | R13 | GRM-44 | main |
| `R13/accept_exists_as_copy_of_gives_a_value_of_your_own.em` | R13, R3 | BRW-14, BRW-15 | twin |
| `R13/accept_exists_as_opens_an_option_and_else_runs_when_it_is_empty.em` | R13 | GRM-44 | main |
| `R13/accept_exists_as_opens_an_option_of_a_pair.em` | R13, R3 | GRM-44, OWN-8 | main |
| `R13/accept_exists_as_ref_mut_changes_what_the_link_names.em` | R13, R5 | GRM-44, FN-4 | main |
| `R13/accept_exists_as_then_copy_of_on_its_own_line.em` | R13, R3 | GRM-44, OWN-8 | main |
| `R13/accept_some_makes_a_value_and_names_a_match_branch.em` | R13 | GRM-44 | main |
| `R13/accept_while_exists_as_repeats_while_it_holds_something.em` | R13, R2 | GRM-44, BRW-12 | main |
| `R13/reject_a_change_through_a_plain_opened_name.em` | R13, R5 | BRW-14, BRW-15 | precise |
| `R13/reject_a_weakref_link_used_as_its_target_without_opening_it.em` | R13, R11 | TYP-4, CLO-3 | precise |
| `R13/reject_an_opened_name_already_in_use.em` | R13 | GRM-44 | precise |
| `R13/reject_an_opened_pair_name_with_no_word_after_one_with_a_word.em` | R13, R3 | GRM-42 | precise |
| `R13/reject_exists_as_on_a_result.em` | R13 | GRM-19 | precise |
| `R13/reject_exists_as_on_a_value_that_always_holds_one.em` | R13 | GRM-19 | precise |
| `R13/reject_some_in_a_while_header.em` | R13 | GRM-44 | precise |
| `R13/reject_some_in_an_if_header.em` | R13 | GRM-44 | precise |
| `R13/reject_upgrade_is_not_a_word_of_ember.em` | R13, R11 | MOD-3 | precise |
| `R13/run_an_opened_weakref_tests_whether_its_target_still_lives.em` | R13, R11 | GRM-44, WK-11 | main |
| `R14/accept_a_bound_follows_from_a_declared_interface.em` | R14 | TYP-44 | main |
| `R14/accept_a_dyn_iterator_whose_steps_push_to_log_needs_no_word.em` | R12, R14, R5 | CLO-1, TYP-44, FN-4 | main |
| `R14/accept_a_generic_method_taking_its_visitor_by_ref_mut_has_one_dyn_entry.em` | R14 | TYP-44 | main |
| `R14/accept_a_kind_test_declares_a_new_name.em` | R14 | TYP-44 | main |
| `R14/accept_a_kind_test_on_a_place_names_it_ref_mut.em` | R14, R5 | TYP-44, FN-4 | main |
| `R14/accept_a_method_whose_kind_test_changes_pools_called_through_a_ref_mut_name.em` | R14, R5, R4 | TYP-44, FN-4, FN-1 | main |
| `R14/accept_a_name_for_a_value_already_of_its_kind.em` | R14, R2 | TYP-44, BRW-12 | main |
| `R14/accept_a_plain_dyn_iterator_is_driven_inside_a_walk_of_log.em` | R12, R14, R8 | BRW-18, CTL-2 | twin |
| `R14/accept_a_private_generic_function_works_its_bound_out_from_its_body.em` | R14, R17 | TYP-44, BLD-14 | main |
| `R14/accept_a_pub_generic_writes_its_bound.em` | R14, R17 | TYP-44, BLD-14 | main |
| `R14/accept_a_ref_mut_cast_of_a_place_changes_the_registry.em` | R14, R2, R6 | TYP-44, BRW-12, LT-1 | main |
| `R14/accept_a_ref_mut_dyn_parameter_changes_the_shape.em` | R14, R4, R5 | TYP-44, FN-1, FN-4 | main |
| `R14/accept_a_ref_mut_kind_test_name_of_a_ref_mut_parameter.em` | R14, R4 | TYP-44, FN-1 | main |
| `R14/accept_owns_dyn_and_shares_dyn_say_who_owns_the_shapes.em` | R14, R3 | TYP-44, OWN-8 | main |
| `R14/accept_the_kind_test_and_the_sure_cast_replace_the_old_forms.em` | R14 | TYP-44 | main |
| `R14/reject_a_bare_interface_name_as_a_type.em` | R14, R3 | TYP-45, IFC-5 | precise |
| `R14/reject_a_call_that_drives_a_dyn_pushing_to_log_inside_a_walk_of_log.em` | R12, R14, R8 | BRW-18, CTL-2 | precise |
| `R14/reject_a_change_through_a_plain_dyn_parameter.em` | R14, R5 | BRW-15 | precise |
| `R14/reject_a_change_through_the_read_only_name_a_kind_test_gives.em` | R14, R5 | BRW-15 | precise |
| `R14/reject_a_dyn_that_pushes_to_log_driven_inside_a_walk_of_log.em` | R12, R14, R8 | BRW-18, CTL-2 | precise |
| `R14/reject_a_generic_body_calling_a_method_no_interface_declares.em` | R14 | STD-13 | precise |
| `R14/reject_a_generic_method_that_returns_its_type_parameter_through_dyn.em` | R14 | IFC-5, TYP-22 | precise |
| `R14/reject_a_kind_test_naming_a_name_already_in_use.em` | R14 | GRM-45, DSP-4 | precise |
| `R14/reject_a_kind_test_that_always_passes.em` | R14 | GRM-19 | precise |
| `R14/reject_a_kind_test_that_can_never_pass.em` | R14 | TYP-44, DSP-4 | precise |
| `R14/reject_a_match_on_a_maybe_downcast.em` | R14 | MOD-3 | precise |
| `R14/reject_a_method_whose_kind_test_changes_pools_called_through_a_read_only_name.em` | R14, R5 | BRW-15 | precise |
| `R14/reject_a_plain_name_from_a_cast_of_a_place_returns_ref_mut.em` | R14, R2 | FN-2a | precise |
| `R14/reject_a_pub_generic_without_its_written_bound.em` | R14, R17 | IFC-6, TYP-17, BLD-14 | precise |
| `R14/reject_a_ref_mut_kind_test_name_of_a_read_only_parameter.em` | R14, R4 | FN-2a | precise |
| `R14/reject_as_bang_is_not_ember.em` | R14 | MOD-3 | precise |
| `R14/reject_as_question_is_not_ember.em` | R14 | MOD-3 | precise |
| `R14/reject_cast_if_is_not_ember.em` | R14 | MOD-3 | precise |
| `R14/reject_cast_when_is_not_ember.em` | R14 | MOD-3 | precise |
| `R14/run_a_kind_test_and_a_cast_on_a_base_class.em` | R14 | TYP-44 | main |
| `R14/run_a_kind_test_picks_each_shape_by_its_kind.em` | R14 | TYP-44 | main |
| `R14/run_fail_a_cast_to_the_wrong_kind_stops_the_program.em` | R14 | TYP-44 | main |
| `R14/run_is_before_a_value_asks_for_the_same_object.em` | R14 | TYP-44 | main |
| `R14/run_is_without_a_name_only_tests_the_kind.em` | R14 | TYP-44 | main |
| `R15/accept_a_borrowed_callback_that_changes_the_list_after_the_walk.em` | R15, R12, R8 | THR-1, CLO-1, BRW-16 | main |
| `R15/accept_a_borrowed_callback_uses_the_real_variable.em` | R15, R12 | THR-1, CLO-1 | main |
| `R15/accept_a_c_call_inside_a_walk_of_names_when_the_kept_callback_changes_only_the_score.em` | R15, R12, R8, R5 | THR-1, CLO-1, BRW-16, FN-4 | main |
| `R15/accept_a_c_call_inside_a_walk_when_no_callback_is_kept.em` | R15, R12, R8 | THR-1, CLO-1, BRW-16 | main |
| `R15/accept_a_c_call_inside_a_walk_when_the_kept_callback_changes_nothing.em` | R15, R12, R8 | THR-1, CLO-1, BRW-16 | main |
| `R15/accept_a_c_function_marked_no_callbacks_inside_a_walk.em` | R15, R12, R5 | THR-1, CLO-1, FN-4 | main |
| `R15/accept_a_callback_c_may_run_on_any_thread_holds_a_thread_shared_owner.em` | R15, R9, R12, R5 | THR-1, RC-1, CLO-1, FN-4 | main |
| `R15/accept_a_class_with_atomic_fields_is_shared_without_a_mark.em` | R15, R9, R12 | THR-1, RC-1, CLO-1 | main |
| `R15/accept_a_creator_thread_callback_holds_a_weakref.em` | R15, R11, R12, R13 | THR-1, WK-11, CLO-1, GRM-44 | main |
| `R15/accept_a_direct_call_that_takes_the_held_lock_enters_past_it.em` | R15 | THR-1 | main |
| `R15/accept_a_float_sum_added_once_per_task.em` | R15 | THR-1 | main |
| `R15/accept_a_generator_views_what_its_caller_keeps.em` | R15, R6 | THR-1, LT-1 | main |
| `R15/accept_a_generator_yields_its_own_number.em` | R15 | THR-1 | main |
| `R15/accept_a_parallel_loop_over_a_range.em` | R15, R4 | THR-1, FN-1 | main |
| `R15/accept_a_parallel_loop_over_one_owner_objects.em` | R15, R1 | THR-1, OWN-1 | main |
| `R15/accept_a_scope_started_after_the_lock_block.em` | R15 | THR-1 | main |
| `R15/accept_a_scoped_task_changes_an_unguarded_static_mut.em` | R15, R5 | THR-1, FN-4 | main |
| `R15/accept_a_scoped_task_looks_at_the_loops_text_with_ref.em` | R15, R12, R3 | THR-1, CLO-1, OWN-8 | main |
| `R15/accept_a_task_reads_and_adds_to_a_guarded_field_under_its_lock.em` | R15, R12 | THR-1, CLO-1 | main |
| `R15/accept_a_task_started_after_the_walk_changes_the_list.em` | R15, R8 | THR-1, BRW-16 | main |
| `R15/accept_a_task_takes_a_copy_of_a_round_local.em` | R15, R12, R5 | THR-1, CLO-1, FN-4 | main |
| `R15/accept_a_task_takes_the_lock_itself.em` | R15, R12, R4, R5 | THR-1, CLO-1, FN-1, FN-4 | main |
| `R15/accept_a_thread_given_a_copy_of_an_object_a_weakref_points_at.em` | R15, R12, R11 | THR-1, THR-2, THR-10, CLO-16, FFI-22 | twin |
| `R15/accept_a_thread_shared_object_is_read_by_a_thread_and_its_maker.em` | R15, R3, R9 | THR-1, OWN-8, RC-1 | main |
| `R15/accept_a_thread_started_in_a_walk_under_the_lock_it_needs.em` | R15, R8 | THR-1, BRW-16 | main |
| `R15/accept_a_thread_started_under_the_lock_is_joined_after_it.em` | R15 | THR-1 | main |
| `R15/accept_a_thread_stores_a_handler_in_a_bus_threads_share.em` | R15, R12 | THR-1, CLO-1 | main |
| `R15/accept_a_thread_takes_a_copy_of_the_loops_text.em` | R15, R12 | THR-1, CLO-1 | main |
| `R15/accept_a_thread_takes_a_list_of_its_own_strings.em` | R15 | THR-1 | main |
| `R15/accept_an_any_thread_export_changes_a_guarded_static.em` | R15 | THR-1 | main |
| `R15/accept_an_unscoped_thread_takes_its_object_over.em` | R15, R1 | THR-1, OWN-1 | main |
| `R15/accept_guarded_statics_changed_by_a_thread.em` | R15, R5 | THR-1, FN-4 | main |
| `R15/accept_handlers_that_change_a_bus_run_after_its_lock.em` | R15, R12 | THR-1, CLO-1 | main |
| `R15/accept_init_sets_guarded_fields_without_the_lock.em` | R15 | THR-1 | main |
| `R15/accept_main_receives_through_its_own_end_while_a_thread_sends.em` | R15, R12, R5 | THR-1, CLO-1, FN-4 | main |
| `R15/accept_main_starts_threads_with_no_word_on_its_first_line.em` | R15, R5, R12 | THR-1, FN-4, CLO-1 | main |
| `R15/accept_one_call_reaches_a_lock_guarded_log_through_two_names.em` | R15, R4, R12 | THR-19 | twin |
| `R15/accept_send_written_by_an_unsafe_extend_author.em` | R15, R16 | THR-1, TIER-1 | main |
| `R15/accept_tasks_change_stats_under_its_lock.em` | R15, R5, R12 | THR-1, FN-4, CLO-1 | main |
| `R15/accept_the_function_value_called_after_the_lock_is_let_go.em` | R15, R12 | THR-1, CLO-1 | main |
| `R15/accept_thread_shared_as_a_bound.em` | R15 | THR-1 | main |
| `R15/accept_threads_share_a_log_and_push_under_its_lock.em` | R15, R12, R5, R3 | THR-1, CLO-1, FN-4, OWN-8 | main |
| `R15/accept_tick_calls_poll_events_with_no_word_on_its_first_line.em` | R15, R12, R5 | BRW-18, CTL-2 | twin |
| `R15/reject_a_borrowed_callback_that_changes_the_walked_list.em` | R15, R12, R8 | BRW-18, CTL-2 | precise |
| `R15/reject_a_c_call_inside_a_walk_a_kept_callback_may_change.em` | R15, R12, R8 | BRW-18, CTL-2 | precise |
| `R15/reject_a_call_through_a_function_value_that_may_take_the_held_lock.em` | R15, R12 | THR-19 | precise |
| `R15/reject_a_callback_c_may_run_on_any_thread_holding_a_shares_owner.em` | R15, R9, R12 | THR-1, THR-2, THR-10, CLO-16, FFI-22 | precise |
| `R15/reject_a_function_value_that_changes_another_log_inside_the_lock.em` | R15, R12 | THR-19 | precise |
| `R15/reject_a_generator_walk_held_by_views_of_its_own_frame.em` | R15 | CORO-6 | precise |
| `R15/reject_a_guarded_field_changed_outside_its_lock.em` | R15 | THR-18 | precise |
| `R15/reject_a_guarded_field_read_outside_its_lock.em` | R15 | THR-18 | precise |
| `R15/reject_a_guarded_field_under_another_objects_lock.em` | R15 | THR-18 | precise |
| `R15/reject_a_guarded_list_pushed_to_outside_its_lock.em` | R15 | THR-18 | precise |
| `R15/reject_a_handler_any_thread_may_run_holding_a_shares_mut_owner.em` | R15, R12 | THR-1, THR-2, THR-10, CLO-16, FFI-22 | precise |
| `R15/reject_a_join_inside_the_lock_its_thread_needs.em` | R15 | THR-19 | precise |
| `R15/reject_a_kept_callback_using_a_name_with_no_capture_word.em` | R15, R12 | THR-17, THR-10, FFI-21 | precise |
| `R15/reject_a_lock_guard_sent_to_a_task.em` | R15 | THR-8, THR-3 | precise |
| `R15/reject_a_module_level_assignment.em` | R15 | GRM-2 | precise |
| `R15/reject_a_parallel_loop_over_shares_mut_objects.em` | R15, R9 | PAR-2 | precise |
| `R15/reject_a_parallel_loop_writing_at_another_index.em` | R15 | PAR-2 | precise |
| `R15/reject_a_parallel_while_loop.em` | R15 | PAR-1 | precise |
| `R15/reject_a_round_local_used_by_a_task_without_a_copy.em` | R15, R12 | CLO-4 | precise |
| `R15/reject_a_scope_that_ends_inside_the_lock_its_task_needs.em` | R15 | THR-19 | precise |
| `R15/reject_a_second_main_in_one_module.em` | R15 | TYP-26 | precise |
| `R15/reject_a_task_changes_the_list_its_parent_walks.em` | R15, R8 | BRW-18, CTL-2 | precise |
| `R15/reject_a_task_reads_a_field_another_task_adds_to.em` | R15, R12 | THR-11, THR-21 | precise |
| `R15/reject_a_task_reads_total_before_the_join.em` | R15 | THR-11, THR-21 | precise |
| `R15/reject_a_task_update_that_uses_the_old_value.em` | R15 | THR-11, THR-21 | precise |
| `R15/reject_a_task_uses_the_loop_name_without_a_copy.em` | R15, R12 | CLO-4 | precise |
| `R15/reject_a_thread_captures_a_one_thread_owner.em` | R15, R9, R12 | THR-1, THR-2, THR-10, CLO-16, FFI-22 | precise |
| `R15/reject_a_thread_changes_an_unguarded_static_mut.em` | R15, R5 | THR-20 | precise |
| `R15/reject_a_thread_given_the_loops_text_with_ref.em` | R15, R12 | CLO-4 | precise |
| `R15/reject_a_thread_started_in_a_walk_uses_the_loop_name_without_a_copy.em` | R15 | THR-17, THR-10 | precise |
| `R15/reject_a_thread_takes_a_list_of_views.em` | R15, R7 | THR-1, THR-2, THR-10 | precise |
| `R15/reject_a_thread_takes_a_shares_mut_bus_to_store_a_handler.em` | R15, R9, R12 | THR-1, THR-2, THR-10, CLO-16, FFI-22 | precise |
| `R15/reject_a_thread_takes_over_a_lent_log.em` | R15, R3 | OWN-11, EXP-6, CLO-6 | precise |
| `R15/reject_a_thread_using_the_log_with_no_capture_word.em` | R15 | THR-17, THR-10 | precise |
| `R15/reject_a_view_of_the_frames_own_local_across_yield.em` | R15 | CORO-6 | precise |
| `R15/reject_a_walk_under_the_lock_across_a_function_value_that_changes_log.em` | R15, R8, R12 | THR-19 | precise |
| `R15/reject_an_any_thread_export_changes_a_static_mut.em` | R15 | THR-20 | precise |
| `R15/accept_an_object_handed_to_another_thread_leaves_its_weakref_links_behind.em` | R15, R12, R11 | THR-1, THR-2, THR-10, CLO-16, FFI-22, WK-12 | precise |
| `R15/reject_an_unmarked_c_function_inside_the_walk.em` | R15, R12, R8 | BRW-18, CTL-2 | precise |
| `R15/reject_an_unscoped_thread_using_a_name_with_no_capture_word.em` | R15 | THR-17, THR-10 | precise |
| `R15/reject_async_is_reserved.em` | R15 | LEX-14 | precise |
| `R15/reject_handing_c_the_callback_inside_the_walk.em` | R15, R12, R8 | BRW-18, CTL-2 | precise |
| `R15/reject_implementing_send_without_unsafe.em` | R15, R16 | UNS-1, UNS-12, FFI-54 | precise |
| `R15/reject_no_callbacks_on_a_function_that_takes_a_callback.em` | R15, R12 | FFI-52 | precise |
| `R15/reject_python_global.em` | R15, R5 | GRM-2 | precise |
| `R15/reject_ref_mut_before_a_channel_end_at_a_call.em` | R15, R4 | FN-2a | precise |
| `R15/reject_running_handlers_that_change_their_bus_under_its_lock.em` | R15, R12 | THR-19 | precise |
| `R15/reject_sync_as_a_bound.em` | R15 | MOD-3 | precise |
| `R15/reject_the_sync_class_mark.em` | R15 | ATT-1, PHIL-12 | precise |
| `R15/reject_thread_shared_from_a_plain_parameter.em` | R15, R3 | THR-1 | precise |
| `R15/reject_thread_shared_on_a_plain_local.em` | R15, R3 | THR-1 | precise |
| `R15/reject_tick_called_inside_a_walk_of_log_may_run_the_kept_callback.em` | R15, R12, R8 | BRW-18, CTL-2 | precise |
| `R15/reject_using_a_name_after_it_was_handed_to_a_thread.em` | R15, R1 | OWN-3 | precise |
| `R15/run_a_float_sum_added_more_than_once_per_task_is_a_reduction.em` | R15 | THR-1 | main |
| `R15/run_a_generator_walks_a_list_by_position.em` | R15, R6 | THR-1, LT-1 | main |
| `R15/run_a_kept_callback_that_ends_its_own_token_then_uses_its_state.em` | R15, R12 | THR-1, CLO-1 | main |
| `R15/run_a_producer_thread_sends_and_a_consumer_thread_receives.em` | R15, R12, R5, R4 | THR-1, CLO-1, FN-4, FN-1 | main |
| `R15/run_a_thread_runs_a_handler_holding_a_thread_shared_owner.em` | R15, R12, R9 | THR-1, CLO-1, RC-1 | main |
| `R15/run_c_holds_a_tagged_number_not_an_address.em` | R15 | THR-1 | main |
| `R15/run_c_runs_a_kept_callback_through_the_trampoline.em` | R15, R12 | THR-1, CLO-1 | main |
| `R15/run_fail_a_creator_callback_called_on_another_thread.em` | R15 | THR-1 | main |
| `R15/run_fail_c_calls_a_once_callback_twice.em` | R15, R12 | THR-1, CLO-1 | main |
| `R15/run_fail_c_runs_a_callback_after_it_was_unregistered.em` | R15 | THR-1 | main |
| `R15/run_scoped_tasks_add_their_parts_to_a_total.em` | R15, R12 | THR-1, CLO-1 | main |
| `R15/run_tasks_add_to_a_field_of_an_object_they_reach.em` | R15 | THR-1 | main |
| `R15/run_tasks_push_their_parts_in_spawn_order.em` | R15 | THR-1 | main |
| `R15/run_two_threads_update_a_guarded_pair_under_one_lock.em` | R15, R9, R12 | THR-1, RC-1, CLO-1 | main |
| `R16/accept_a_blend_from_a_copy_inside_no_alias_checks.em` | R16, R4 | TIER-1, FN-1 | main |
| `R16/accept_a_c_function_declared_owned_takes_the_widget_with_no_word.em` | R16, R4 | TIER-1, FN-1 | main |
| `R16/accept_a_further_argument_passed_as_c_reads_it.em` | R16 | TIER-1 | main |
| `R16/accept_a_loop_over_the_view_itself.em` | R16 | TIER-1 | main |
| `R16/accept_a_safety_note_on_an_unsafe_fn_line_is_warned.em` | R16 | UNS-8 | precise |
| `R16/accept_an_unsafe_block_without_a_safety_note_is_warned.em` | R16 | UNS-8 | precise |
| `R16/accept_an_unsafe_fn_vouches_block_by_block.em` | R16 | TIER-1 | main |
| `R16/accept_an_unsafe_fn_without_safety_is_warned.em` | R16 | UNS-7 | precise |
| `R16/accept_into_raw_inside_unsafe_gives_c_the_address.em` | R16 | TIER-1 | main |
| `R16/accept_no_alias_checks_with_the_indexes_proved_apart.em` | R16, R4 | TIER-1, FN-1 | main |
| `R16/reject_a_narrow_further_argument_c_would_promote.em` | R16 | FFI-8 | precise |
| `R16/reject_a_widget_used_after_it_was_handed_to_c.em` | R16, R4, R1 | OWN-3 | precise |
| `R16/reject_an_unsafe_operation_in_an_unsafe_fn_body_outside_a_block.em` | R16 | UNS-1, UNS-12, FFI-54 | precise |
| `R16/reject_assert_disjoint_inside_no_alias_checks.em` | R16, R4 | DSJ-6, EFF-12 | precise |
| `R16/reject_calling_an_unsafe_fn_outside_unsafe.em` | R16 | UNS-1, UNS-12, FFI-54 | precise |
| `R16/reject_into_raw_outside_unsafe.em` | R16 | UNS-1, UNS-12, FFI-54 | precise |
| `R16/reject_no_runtime_checks.em` | R16 | ATT-1, PHIL-12 | precise |
| `R16/reject_static_safe_is_now_no_alias_checks.em` | R16 | ATT-1, PHIL-12 | precise |
| `R16/run_fail_a_bounds_check_still_runs_inside_no_alias_checks.em` | R16 | TIER-1 | main |
| `R17/accept_a_body_fact_is_used_inside_its_package.em` | R17, R5, R8 | BLD-14, FN-4, BRW-16 | main |
| `R17/accept_a_call_from_another_package_given_what_its_recorded_fact_allows.em` | R17, R4 | BLD-14, FN-1 | main |
| `R17/accept_a_ledger_generic_whose_instance_changes_log_after_the_loop.em` | R17, R12, R8 | BLD-14, CLO-1, BRW-16 | main |
| `R17/accept_a_librarys_button_clicked_after_the_loop_over_log.em` | R17, R12, R8 | BLD-14, CLO-1, BRW-16 | main |
| `R17/accept_a_private_generic_infers_its_bounds.em` | R17, R14 | BLD-14, TYP-44 | main |
| `R17/accept_a_pub_methods_recorded_reads_cross_the_package_edge.em` | R17, R5, R8 | BLD-14, FN-4, BRW-16 | main |
| `R17/accept_an_only_adds_summary_crosses_the_package_edge.em` | R17, R8 | BLD-14, BRW-16 | main |
| `R17/reject_a_call_from_another_package_hands_a_part_of_what_it_clears.em` | R17, R4 | BRW-21 | precise |
| `R17/reject_a_call_from_another_package_that_may_do_more_than_add.em` | R17, R8 | BRW-17 | precise |
| `R17/reject_a_ledger_generic_whose_instance_changes_log_in_a_loop_over_log.em` | R17, R12, R8 | BRW-18, CTL-2 | precise |
| `R17/reject_a_librarys_button_runs_a_programs_log_handler_in_a_loop_over_log.em` | R17, R12, R8 | BRW-18, CTL-2 | precise |
| `R17/reject_a_pub_generic_without_its_bounds.em` | R17, R14 | IFC-6, TYP-17, BLD-14 | precise |
| `R17/reject_a_pub_method_whose_recorded_reads_include_the_walked_list.em` | R17, R5, R8 | BRW-1 | precise |
| `R2/accept_a_call_that_makes_a_new_value_hands_it_to_the_name.em` | R2 | BRW-12 | main |
| `R2/accept_a_const_ref_mut_name_writes_its_place.em` | R2 | BRW-12 | main |
| `R2/accept_a_field_handed_over_on_its_own_leaves_the_rest_usable.em` | R2, R1 | BRW-12, OWN-1 | main |
| `R2/accept_a_field_of_a_class_object_taken_with_mem_take.em` | R2 | BRW-12 | main |
| `R2/accept_a_function_that_replaces_an_object_whose_drop_pushes_to_log_called_after_the_loop.em` | R2, R5, R8 | BRW-12, FN-4, BRW-16 | main |
| `R2/accept_a_mut_span_name_is_pointed_at_another_span.em` | R2 | BRW-12 | main |
| `R2/accept_a_name_declared_in_a_loop_body_is_new_each_round.em` | R2, R1 | OWN-3 | twin |
| `R2/accept_a_name_for_a_part_is_a_view_and_its_owner_stays_whole.em` | R2, R4 | BRW-12, FN-1 | main |
| `R2/accept_a_name_for_a_view_a_call_gives_is_read_only.em` | R2, R8 | BRW-12, BRW-16 | main |
| `R2/accept_a_number_of_your_own_made_from_a_parameter.em` | R2 | BRW-12 | main |
| `R2/accept_a_number_part_is_copied.em` | R2 | BRW-12 | main |
| `R2/accept_a_number_viewed_with_ref_changed_after_its_last_use.em` | R2, R8 | BRW-12, BRW-16 | main |
| `R2/accept_a_ref_mut_name_holding_a_mut_span_writes_its_elements.em` | R2 | BRW-12 | main |
| `R2/accept_a_ref_mut_name_is_given_another_ref.em` | R2 | BRW-12 | main |
| `R2/accept_a_ref_mut_view_parameter_given_another_view_points_the_callers_view.em` | R2, R4 | BRW-12, FN-1 | main |
| `R2/accept_a_view_name_is_pointed_at_another_view.em` | R2 | BRW-12 | main |
| `R2/accept_a_view_name_pointed_elsewhere_frees_its_old_target.em` | R2, R8 | BRW-12, BRW-16 | main |
| `R2/accept_a_view_of_your_own_made_from_a_parameter.em` | R2 | BRW-12 | main |
| `R2/accept_copy_of_a_part_is_a_value_of_your_own.em` | R2, R3 | BRW-12, OWN-8 | main |
| `R2/accept_equals_on_a_name_replaces_and_ends_its_old_value.em` | R2 | BRW-12 | main |
| `R2/accept_equals_on_a_ref_mut_loop_name_writes_the_element.em` | R2 | BRW-12 | main |
| `R2/accept_equals_on_a_ref_mut_parameter_writes_the_callers_place.em` | R2, R4 | BRW-12, FN-1 | main |
| `R2/accept_equals_on_a_ref_mut_pattern_name_replaces_the_payload.em` | R2 | BRW-12 | main |
| `R2/accept_ref_mut_names_a_part_that_may_change.em` | R2 | BRW-12 | main |
| `R2/accept_ref_mut_pattern_name_changes_the_payload.em` | R2 | BRW-12 | main |
| `R2/reject_a_change_of_a_plain_opened_name.em` | R2, R13 | BRW-14, BRW-15 | precise |
| `R2/reject_a_change_through_a_name_for_a_part.em` | R2, R5 | BRW-14, BRW-15 | precise |
| `R2/reject_a_change_through_a_plain_pattern_name.em` | R2, R5 | BRW-14, BRW-15 | precise |
| `R2/reject_a_field_taken_out_of_a_class_object.em` | R2 | OWN-11 | precise |
| `R2/reject_a_field_used_after_it_was_handed_over_on_its_own.em` | R2, R1 | OWN-3 | precise |
| `R2/reject_a_loop_over_log_calls_a_function_that_replaces_an_object_whose_drop_pushes_to_log.em` | R2, R5, R8 | BRW-18, CTL-2 | precise |
| `R2/reject_a_number_viewed_with_ref_changed_while_viewed.em` | R2, R8 | BRW-1 | precise |
| `R2/reject_a_read_only_ref_given_to_a_ref_mut_name.em` | R2 | BRW-14, BRW-15 | precise |
| `R2/reject_a_struct_used_whole_after_a_field_was_handed_over.em` | R2, R1 | OWN-3 | precise |
| `R2/reject_a_view_name_pointed_elsewhere_is_checked_against_its_new_target.em` | R2, R8 | BRW-17 | precise |
| `R2/reject_a_view_of_a_part_used_after_its_owner_was_handed_over.em` | R2, R8 | BRW-17 | precise |
| `R2/reject_a_write_through_a_plain_name_holding_a_mut_span.em` | R2, R5 | BRW-14, BRW-15 | precise |
| `R2/reject_another_ref_given_to_a_const_ref_mut_name.em` | R2 | BRW-14, BRW-15 | precise |
| `R2/reject_equals_on_a_loop_name.em` | R2 | BRW-14, BRW-15 | precise |
| `R2/reject_equals_on_a_number_parameter.em` | R2 | BRW-14, BRW-15 | precise |
| `R2/reject_equals_on_a_pattern_name.em` | R2 | BRW-14, BRW-15 | precise |
| `R2/reject_equals_on_a_view_parameter.em` | R2 | BRW-14, BRW-15 | precise |
| `R2/reject_mut_alone_on_a_pattern_name.em` | R2, R3 | GRM-42 | precise |
| `R2/run_a_partly_handed_over_struct_ends_each_part_once.em` | R2, R1 | BRW-12, OWN-1 | main |
| `R3/accept_a_capture_list_names_what_it_takes_over.em` | R3, R12 | OWN-8, CLO-1 | main |
| `R3/accept_a_captured_copy_is_the_function_values_own.em` | R3, R12 | OWN-8, CLO-1 | main |
| `R3/accept_a_class_object_held_by_a_field.em` | R3 | OWN-8 | main |
| `R3/accept_a_class_with_a_weakref_link_to_its_parent.em` | R3, R11, R13 | OWN-8, WK-11, GRM-44 | main |
| `R3/accept_a_document_made_again_from_its_text.em` | R3, R7 | OWN-8, LT-45 | main |
| `R3/accept_a_for_loop_over_owns_hands_each_element_over.em` | R3 | OWN-8 | main |
| `R3/accept_a_hand_over_written_with_no_word.em` | R3, R1 | OWN-8, OWN-1 | main |
| `R3/accept_a_level_made_in_an_arena_the_caller_keeps.em` | R3, R7 | OWN-8, LT-45 | main |
| `R3/accept_a_list_field_names_its_arena_and_the_making_line_its_instance.em` | R3 | OWN-8 | main |
| `R3/accept_a_list_that_owns_its_interface_values.em` | R3, R14 | OWN-8, TYP-44 | main |
| `R3/accept_a_manager_with_one_instance_needs_none_on_the_making_line.em` | R3 | OWN-8 | main |
| `R3/accept_a_match_over_owns_hands_the_payload_over.em` | R3 | OWN-8 | main |
| `R3/accept_a_number_handed_over_with_owns.em` | R3 | OWN-8 | main |
| `R3/accept_a_number_taken_over_by_a_function_with_owns.em` | R3, R12 | OWN-8, CLO-1 | main |
| `R3/accept_a_parameter_list_with_types_is_not_such_a_list.em` | R3, R12 | OWN-8, CLO-1 | main |
| `R3/accept_a_plain_value_stored_into_an_owns_field.em` | R3 | OWN-8 | main |
| `R3/accept_a_reading_owner_made_from_a_reading_owner.em` | R3 | OWN-8 | main |
| `R3/accept_a_string_of_your_own_made_from_a_view.em` | R3 | OWN-8 | main |
| `R3/accept_a_value_pushed_into_a_list_of_owns_values_says_owns.em` | R3, R14 | OWN-8, TYP-44 | main |
| `R3/accept_a_weakref_link_leaves_its_target_with_its_owner.em` | R3, R11, R13 | OWN-8, WK-11, GRM-44 | main |
| `R3/accept_an_existing_mesh_handed_to_an_owns_field_with_no_word.em` | R3, R1 | OWN-8, OWN-1 | main |
| `R3/accept_another_link_and_a_copy_of_the_target.em` | R3, R11, R13 | OWN-8, WK-11, GRM-44 | main |
| `R3/accept_copy_of_a_shares_link_copies_its_target.em` | R3 | OWN-8 | main |
| `R3/accept_each_captured_name_says_its_own_word.em` | R3 | OWN-8 | main |
| `R3/accept_each_loop_name_says_its_own_word.em` | R3 | OWN-8 | main |
| `R3/accept_fp_allow_fma_lets_the_compiler_fuse.em` | R3, R16 | OWN-8, TIER-1 | main |
| `R3/accept_owns_on_the_line_that_makes_a_value_for_an_owns_field.em` | R3 | OWN-8 | main |
| `R3/accept_the_first_owner_keeps_what_it_could_do.em` | R3, R9 | OWN-8, RC-1 | main |
| `R3/accept_thread_shared_on_a_value_made_thread_shared.em` | R3, R15 | OWN-8, THR-1 | main |
| `R3/accept_two_owners_that_may_change_one_list.em` | R3, R9 | OWN-8, RC-1 | main |
| `R3/reject_a_bare_interface_name_used_as_a_type.em` | R3, R14 | TYP-45, IFC-5 | precise |
| `R3/reject_a_captured_name_with_no_word_after_one_with_a_word.em` | R3 | GRM-42 | precise |
| `R3/reject_a_change_through_a_shares_owner.em` | R3, R9 | BRW-14, BRW-15 | precise |
| `R3/reject_a_changing_owner_made_from_a_reading_owner.em` | R3 | BRW-14, BRW-15 | precise |
| `R3/reject_a_lambda_parameter_with_no_word_after_one_with_a_word.em` | R3, R12 | GRM-42 | precise |
| `R3/reject_a_list_used_after_a_for_loop_over_owns.em` | R3, R1 | OWN-3 | precise |
| `R3/reject_a_loop_name_with_no_word_after_one_with_a_word.em` | R3 | GRM-42 | precise |
| `R3/reject_a_new_value_made_later_for_an_owns_field_without_owns.em` | R3 | OWN-10, TYP-46 | precise |
| `R3/reject_a_number_used_after_a_function_took_it_over_with_owns.em` | R3, R12, R1 | OWN-3 | precise |
| `R3/reject_a_number_used_after_it_was_handed_over_with_owns.em` | R3, R1 | OWN-3 | precise |
| `R3/reject_a_plain_loop_name_pushed_into_another_list.em` | R3, R2 | OWN-11 | precise |
| `R3/reject_a_struct_holding_a_view_of_its_own_type.em` | R3, R7 | TYP-14 | precise |
| `R3/reject_a_type_that_contains_itself_with_no_block_between.em` | R3 | TYP-14 | precise |
| `R3/reject_a_value_from_another_kind_of_arena_given_to_an_arena_place.em` | R3 | TYP-4, CLO-3 | precise |
| `R3/reject_a_value_from_the_built_in_manager_given_to_an_arena_place.em` | R3 | TYP-4, CLO-3 | precise |
| `R3/reject_a_value_made_for_an_owns_field_without_owns.em` | R3 | OWN-10, TYP-46 | precise |
| `R3/reject_a_value_made_inside_some_for_an_option_of_owns_without_owns.em` | R3 | OWN-10, TYP-46 | precise |
| `R3/reject_a_value_pushed_into_a_list_of_owns_values_without_owns.em` | R3, R14 | OWN-10, TYP-46 | precise |
| `R3/reject_a_value_returned_past_the_end_of_its_arena.em` | R3, R7 | LT-45, ALC-1 | precise |
| `R3/reject_an_array_with_a_manager_is_array_using.em` | R3 | MOD-3 | precise |
| `R3/reject_an_object_stored_into_a_weakref_place_without_weakref.em` | R3, R11 | TYP-4, CLO-3 | precise |
| `R3/reject_box_is_not_a_word_of_ember.em` | R3 | MOD-3 | precise |
| `R3/reject_box_with_a_manager_is_owns_using.em` | R3 | MOD-3 | precise |
| `R3/reject_clone_is_not_a_word_of_ember.em` | R3 | MOD-3 | precise |
| `R3/reject_copied_is_now_copy_each.em` | R3 | MOD-3 | precise |
| `R3/reject_copy_of_a_type_with_no_clone.em` | R3 | OWN-8 | precise |
| `R3/reject_copy_of_a_value_whose_views_point_into_its_own_let_field.em` | R3, R7 | OWN-8 | precise |
| `R3/reject_copy_of_a_view.em` | R3 | OWN-8 | precise |
| `R3/reject_copy_of_a_weakref_link.em` | R3, R11 | OWN-8 | precise |
| `R3/reject_cstr_is_now_cstrview.em` | R3 | MOD-3 | precise |
| `R3/reject_foreign_box_is_now_owns.em` | R3 | MOD-3 | precise |
| `R3/reject_fp_contract_is_now_fp_allow_fma.em` | R3, R16 | ATT-1, PHIL-12 | precise |
| `R3/reject_owned_is_now_owns.em` | R3, R4 | MOD-3 | precise |
| `R3/reject_owns_before_a_returned_value.em` | R3 | OWN-9 | precise |
| `R3/reject_owns_before_a_value_its_place_already_hands_over.em` | R3 | OWN-9 | precise |
| `R3/reject_owns_before_fn.em` | R3, R12 | OWN-9 | precise |
| `R3/reject_owns_on_a_class_type.em` | R3 | TYP-45, TYP-46, ALC-1 | precise |
| `R3/reject_shared_is_now_shares.em` | R3 | MOD-3 | precise |
| `R3/reject_str_is_now_strview.em` | R3 | MOD-3 | precise |
| `R3/reject_sync_shared_is_now_thread_shared.em` | R3, R15 | MOD-3 | precise |
| `R3/reject_thread_shared_on_a_value_not_made_thread_shared.em` | R3, R15 | THR-1 | precise |
| `R3/reject_using_after_a_type_with_no_heap_memory.em` | R3 | TYP-45, TYP-46, ALC-1 | precise |
| `R3/reject_weak_is_not_a_word_of_ember.em` | R3, R11 | MOD-3 | precise |
| `R3/reject_with_before_a_capture_list.em` | R3, R12 | GRM-2 | precise |
| `R3/run_a_mesh_made_using_an_arena_takes_its_block_from_that_arena.em` | R3 | OWN-8 | main |
| `R3/run_a_new_value_made_later_for_an_owns_field_says_owns_and_ends_the_old_one.em` | R3, R2 | OWN-8, BRW-12 | main |
| `R3/run_a_value_ends_in_the_arena_instance_it_came_from.em` | R3 | OWN-8 | main |
| `R3/run_a_value_stored_into_an_owns_place_gets_its_own_block.em` | R3 | OWN-8 | main |
| `R3/run_copy_of_a_struct_copies_the_links_it_holds.em` | R3, R9, R11 | OWN-8, RC-1, WK-11 | main |
| `R3/run_copy_of_makes_an_independent_object.em` | R3 | OWN-8 | main |
| `R3/run_the_last_of_two_owners_ends_the_value.em` | R3, R9 | OWN-8, RC-1 | main |
| `R4/accept_a_callable_type_writes_the_parameters_word.em` | R4, R12 | FN-1, CLO-1 | main |
| `R4/accept_a_const_ref_mut_parameter_writes_the_callers_elements.em` | R4, R2 | FN-1, BRW-12 | main |
| `R4/accept_a_copy_given_to_a_function_that_clears_log.em` | R4, R3 | FN-1, OWN-8 | main |
| `R4/accept_a_copy_of_its_own_beside_a_changeable_list.em` | R4 | FN-1 | main |
| `R4/accept_a_copy_of_its_own_beside_a_changeable_span.em` | R4 | FN-1 | main |
| `R4/accept_a_log_changed_only_under_its_lock_reaches_one_call_through_two_names.em` | R4, R15, R12 | THR-19 | twin |
| `R4/accept_a_method_given_another_object_as_ref_mut.em` | R4, R5 | FN-1, FN-4 | main |
| `R4/accept_a_mutex_argument_takes_no_word.em` | R4, R5 | FN-1, FN-4 | main |
| `R4/accept_a_number_given_to_an_owns_parameter_or_a_copy_of_one.em` | R4, R1 | FN-1, OWN-1 | main |
| `R4/accept_a_plain_parameter_takes_no_word_at_the_call.em` | R4 | FN-1 | main |
| `R4/accept_a_ref_mut_argument_of_your_own_given_to_a_function_that_clears_log.em` | R4, R3 | FN-1, OWN-8 | main |
| `R4/accept_a_ref_mut_argument_read_through_its_parameter.em` | R4 | BRW-21 | twin |
| `R4/accept_a_ref_mut_loop_name_over_a_ref_mut_list.em` | R4, R5 | FN-1, FN-4 | main |
| `R4/accept_a_ref_mut_mutspan_local_given_as_ref_mut.em` | R4, R2 | FN-1, BRW-12 | main |
| `R4/accept_a_ref_mut_mutspan_parameter_writes_the_callers_elements.em` | R4, R6, R2 | FN-1, LT-1, BRW-12 | main |
| `R4/accept_a_ref_mut_name_given_another_ref.em` | R4, R2 | FN-1, BRW-12 | main |
| `R4/accept_a_ref_mut_parameter_changes_the_callers_counter.em` | R4 | FN-1 | main |
| `R4/accept_a_ref_mut_parameter_handed_on_as_ref_mut.em` | R4 | FN-1 | main |
| `R4/accept_a_ref_mut_shares_mut_parameter_sets_a_cell.em` | R4 | FN-1 | main |
| `R4/accept_a_ref_mut_view_parameter_points_the_callers_view_elsewhere.em` | R4, R6, R2 | FN-1, LT-1, BRW-12 | main |
| `R4/accept_a_view_argument_used_before_the_callee_clears.em` | R4, R8 | FN-1, BRW-16 | main |
| `R4/accept_a_view_put_in_through_ref_mut_used_before_its_source_grows.em` | R4, R6, R8 | FN-1, LT-1, BRW-16 | main |
| `R4/accept_an_owns_parameter_hands_on_a_changing_owner.em` | R4, R3 | FN-1, OWN-8 | main |
| `R4/accept_an_owns_parameter_takes_a_node_over.em` | R4, R1 | FN-1, OWN-1 | main |
| `R4/accept_an_unused_check_disjoint_result_is_warned.em` | R4 | ERR-5 | precise |
| `R4/accept_check_disjoint_proves_apart_in_its_ok_branch.em` | R4 | FN-1 | main |
| `R4/accept_drop_the_lines_first_and_push_the_notes_after.em` | R4, R12 | FN-1, CLO-1 | main |
| `R4/accept_every_pair_proved_apart_by_its_loop.em` | R4 | FN-1 | main |
| `R4/accept_one_game_passed_once_as_ref_mut.em` | R4, R5 | FN-1, FN-4 | main |
| `R4/accept_ref_mut_on_the_parameters_name.em` | R4 | FN-1 | main |
| `R4/accept_text_written_in_the_program_given_to_a_function_that_clears_log.em` | R4 | FN-1 | main |
| `R4/accept_two_bodies_inside_if_i_is_not_j.em` | R4 | FN-1 | main |
| `R4/accept_two_tracks_proved_apart_by_assert_disjoint.em` | R4 | FN-1 | main |
| `R4/reject_a_call_without_ref_mut_for_a_ref_mut_parameter.em` | R4 | FN-2a | precise |
| `R4/reject_a_const_ref_mut_name_given_another_ref.em` | R4, R2 | BRW-14, BRW-15 | precise |
| `R4/reject_a_const_ref_mut_parameter_given_another_span.em` | R4, R2 | BRW-14, BRW-15 | precise |
| `R4/reject_a_function_value_that_changes_the_calls_object.em` | R4, R12 | BRW-11 | precise |
| `R4/reject_a_method_given_its_own_object_as_ref_mut.em` | R4, R5 | BRW-11 | precise |
| `R4/reject_a_mutspan_argument_without_ref_mut.em` | R4 | FN-2a | precise |
| `R4/reject_a_name_used_after_it_was_handed_to_an_owns_parameter.em` | R4, R1 | OWN-3 | precise |
| `R4/reject_a_number_used_after_it_was_handed_to_an_owns_parameter.em` | R4, R1 | OWN-3 | precise |
| `R4/reject_a_plain_mutspan_local_given_as_ref_mut.em` | R4, R2 | FN-2a | precise |
| `R4/reject_a_plain_mutspan_parameter_writes.em` | R4, R5 | BRW-15 | precise |
| `R4/reject_a_plain_parameter_changes_the_callers_counter.em` | R4, R5 | BRW-15 | precise |
| `R4/reject_a_plain_shares_mut_parameter_sets_a_cell.em` | R4, R5 | BRW-15 | precise |
| `R4/reject_a_read_and_a_change_of_one_game_in_one_call.em` | R4 | BRW-11 | precise |
| `R4/reject_a_ref_mut_argument_into_what_the_callee_clears.em` | R4, R8 | BRW-21 | precise |
| `R4/reject_a_ref_mut_argument_the_callee_also_reads_by_its_own_name.em` | R4 | BRW-21 | precise |
| `R4/reject_a_ref_mut_loop_name_over_a_list_the_function_only_reads.em` | R4, R5, R8 | FN-2a | precise |
| `R4/reject_a_view_argument_into_what_a_callee_of_the_callee_clears.em` | R4, R5, R8 | BRW-21 | precise |
| `R4/reject_a_view_argument_into_what_the_callee_clears_and_then_uses.em` | R4, R8, R17 | BRW-21 | precise |
| `R4/reject_a_view_of_the_callees_own_local_put_in_through_ref_mut.em` | R4, R6 | LT-45, BRW-19, ALC-1 | precise |
| `R4/reject_a_view_put_in_through_ref_mut_used_after_its_source_grows.em` | R4, R6, R8 | BRW-17 | precise |
| `R4/reject_a_write_through_a_span_parameter.em` | R4, R5 | BRW-15 | precise |
| `R4/reject_an_argument_reaching_into_what_a_changeable_one_owns.em` | R4 | BRW-11 | precise |
| `R4/reject_an_element_given_as_ref_mut_beside_its_whole_list.em` | R4, R8 | BRW-1, BRW-11 | precise |
| `R4/reject_an_ignored_check_disjoint_proves_nothing.em` | R4 | BRW-11 | precise |
| `R4/reject_an_owner_that_changes_made_from_a_plain_parameter.em` | R4, R3 | BRW-14, BRW-15 | precise |
| `R4/reject_const_at_a_call_to_a_plain_ref_mut_parameter.em` | R4 | FN-2a | precise |
| `R4/reject_mut_alone_on_a_parameter.em` | R4 | FN-1 | precise |
| `R4/reject_mut_in_a_query_is_now_ref_mut.em` | R4, R3 | MOD-3 | precise |
| `R4/reject_owns_written_before_an_argument.em` | R4, R3 | OWN-9 | precise |
| `R4/reject_ref_b_given_to_a_ref_mut_name.em` | R4, R2 | BRW-14, BRW-15 | precise |
| `R4/reject_ref_in_a_parameters_type.em` | R4 | FN-1 | precise |
| `R4/reject_ref_mut_at_a_call_to_a_const_ref_mut_parameter.em` | R4 | FN-2a | precise |
| `R4/reject_ref_mut_at_a_call_to_a_plain_parameter.em` | R4 | FN-2a | precise |
| `R4/reject_ref_mut_before_a_mutex_argument.em` | R4, R5 | FN-2a | precise |
| `R4/reject_ref_mut_before_a_new_number.em` | R4 | FN-2a | precise |
| `R4/reject_ref_mut_given_from_a_plain_parameter.em` | R4, R5 | FN-2a | precise |
| `R4/reject_ref_mut_in_a_parameters_type.em` | R4 | FN-1 | precise |
| `R4/reject_ref_mut_on_a_query_item_the_query_only_reads.em` | R4 | FN-2a | precise |
| `R4/reject_the_same_body_given_twice.em` | R4 | BRW-11 | precise |
| `R4/reject_two_bodies_whose_indexes_may_be_equal.em` | R4 | BRW-11 | precise |
| `R4/reject_two_spans_of_one_list_one_changeable.em` | R4 | BRW-11 | precise |
| `R4/reject_two_tracks_of_one_map_not_proved_apart.em` | R4 | BRW-11 | precise |
| `R4/run_a_module_list_given_to_a_ref_mut_parameter_its_function_also_names.em` | R4, R8 | BRW-1, BRW-11, BRW-21 | twin |
| `R4/run_assert_disjoint_stops_on_one_track.em` | R4 | FN-1 | main |
| `R4/run_one_variable_given_twice_as_ref_mut_is_one_variable_in_the_call.em` | R4, R8 | FN-1, BRW-16 | main |
| `R5/accept_a_change_through_a_link_needs_no_word_whatever_holds_it.em` | R5, R4 | FN-4, FN-1 | main |
| `R5/accept_a_change_to_an_own_local_or_a_ref_mut_parameter_that_is_no_link_needs_no_clause.em` | R5, R4 | FN-4, FN-1 | main |
| `R5/accept_a_changing_method_called_through_a_ref_mut_loop_name.em` | R5 | FN-4 | main |
| `R5/accept_a_drop_that_changes_nothing_outside_writes_no_changes.em` | R5 | FN-4 | main |
| `R5/accept_a_drop_that_pushes_to_log_writes_no_word.em` | R5 | FN-4 | main |
| `R5/accept_a_first_line_in_its_order.em` | R5, R6 | FN-4, LT-1 | main |
| `R5/accept_a_function_changes_a_module_variable_with_no_word.em` | R5 | FN-4 | main |
| `R5/accept_a_loop_calls_a_virtual_that_changes_one_field.em` | R5, R8 | BRW-18, CTL-2 | twin |
| `R5/accept_a_method_changes_its_object_and_a_module_variable.em` | R5 | FN-4 | main |
| `R5/accept_a_method_changes_its_object_with_no_word.em` | R5 | FN-4 | main |
| `R5/accept_a_method_that_adds_to_the_log_called_after_the_loop_over_it.em` | R5, R8 | FN-4, BRW-16 | main |
| `R5/accept_a_method_that_uses_this_before_it_clears_what_owns_it.em` | R5, R4 | FN-4, FN-1 | main |
| `R5/accept_a_static_fn_is_called_on_its_type.em` | R5 | FN-4 | main |
| `R5/accept_a_struct_method_replaces_its_whole_value.em` | R5 | FN-4 | main |
| `R5/accept_an_atomic_changed_through_a_plain_name.em` | R5, R4, R12 | FN-4, FN-1, CLO-1 | main |
| `R5/accept_an_implementation_may_change_its_object.em` | R5 | FN-4 | main |
| `R5/accept_an_interface_method_whose_implementation_pushes_to_log_called_before_the_loop.em` | R5, R8, R14 | FN-4, BRW-16, TYP-44 | main |
| `R5/accept_an_override_may_change_what_its_virtual_leaves_alone.em` | R5 | FN-4 | main |
| `R5/accept_init_calls_a_method_once_the_fields_its_body_reads_are_set.em` | R5 | FN-4 | main |
| `R5/accept_init_sets_its_fields_with_no_word.em` | R5 | FN-4 | main |
| `R5/accept_owns_this_takes_the_object_over.em` | R5 | FN-4 | main |
| `R5/accept_tick_called_after_the_loop_over_log.em` | R5, R8 | FN-4, BRW-16 | main |
| `R5/accept_tick_calls_note_inside_a_loop_over_players.em` | R5, R8, R17 | BRW-18, CTL-2 | twin |
| `R5/reject_a_changing_method_called_on_a_plain_parameter.em` | R5, R4 | BRW-15 | precise |
| `R5/reject_a_changing_method_called_through_a_plain_loop_name.em` | R5 | BRW-15 | precise |
| `R5/reject_a_first_line_out_of_order.em` | R5 | GRM-41 | precise |
| `R5/reject_a_loop_calls_a_virtual_whose_override_pushes_to_the_walked_log.em` | R5, R8 | BRW-18, CTL-2 | precise |
| `R5/reject_a_loop_over_log_calls_an_interface_method_whose_implementation_pushes_to_log.em` | R5, R8, R14 | BRW-18, CTL-2 | precise |
| `R5/reject_a_loop_over_log_calls_tick_which_adds_to_log_through_note.em` | R5, R8 | BRW-18, CTL-2 | precise |
| `R5/reject_a_method_called_on_a_part_of_what_it_clears.em` | R5, R4, R8 | BRW-21 | precise |
| `R5/reject_a_method_that_adds_to_the_log_called_in_a_loop_over_the_log.em` | R5, R8 | BRW-18, CTL-2 | precise |
| `R5/reject_a_name_used_after_owns_this.em` | R5, R1 | OWN-3 | precise |
| `R5/reject_a_plain_loop_name_changes_a_player.em` | R5 | BRW-15 | precise |
| `R5/reject_cached_reading_a_field_other_threads_change.em` | R5, R15 | CLS-12 | precise |
| `R5/reject_cached_reading_a_shares_field.em` | R5, R9 | CLS-12 | precise |
| `R5/reject_cached_with_another_parameter.em` | R5 | CLS-12 | precise |
| `R5/reject_cached_with_changes.em` | R5 | CLS-12 | precise |
| `R5/reject_changes_written_after_a_functions_brackets.em` | R5 | GRM-2 | precise |
| `R5/reject_init_calls_a_method_before_the_field_its_body_reads_is_set.em` | R5 | CLS-2 | precise |
| `R5/reject_init_calls_a_method_whose_body_reads_a_field_not_yet_set.em` | R5 | CLS-2 | precise |
| `R5/reject_mut_self_is_not_ember.em` | R5 | MOD-3 | precise |
| `R5/reject_mut_self_on_a_drop_is_not_ember.em` | R5 | MOD-3 | precise |
| `R5/reject_mut_this_on_a_methods_first_line.em` | R5 | GRM-42, FN-1 | precise |
| `R5/reject_owned_self_is_owns_this.em` | R5 | MOD-3 | precise |
| `R5/reject_owns_this_on_init.em` | R5 | CLS-2 | precise |
| `R5/reject_reads_written_after_a_methods_brackets.em` | R5 | GRM-2 | precise |
| `R5/reject_self_names_a_methods_object.em` | R5 | MOD-3 | precise |
| `R5/reject_static_fn_outside_a_type.em` | R5 | GRM-41, FN-4 | precise |
| `R5/reject_this_given_another_object.em` | R5 | CLS-7 | precise |
| `R5/reject_this_in_a_free_function.em` | R5 | MOD-3 | precise |
| `R5/reject_this_in_a_static_fn.em` | R5 | MOD-3 | precise |
| `R5/reject_virtual_in_a_class_that_cannot_be_subclassed.em` | R5 | CLS-4 | precise |
| `R5/run_a_cached_result_is_cleared_by_a_change_to_what_it_reads.em` | R5 | FN-4 | main |
| `R5/run_a_turret_aimed_at_itself_changes_itself.em` | R5, R4, R11, R13 | FN-4, FN-1, WK-11, GRM-44 | main |
| `R6/accept_a_call_returning_ref_mut_is_a_place.em` | R6, R2 | LT-1, BRW-12 | main |
| `R6/accept_a_push_after_the_generators_last_use.em` | R6, R8 | LT-1, BRW-16 | main |
| `R6/accept_from_narrows_a_function_result_to_one_parameter.em` | R6, R8 | LT-1, BRW-16 | main |
| `R6/accept_from_src_lets_the_parser_move_on.em` | R6, R8 | LT-1, BRW-16 | main |
| `R6/accept_the_argument_changes_after_the_views_last_use.em` | R6, R8 | LT-1, BRW-16 | main |
| `R6/reject_a_change_to_any_part_of_the_argument_while_its_view_is_used.em` | R6, R8 | BRW-1 | precise |
| `R6/reject_a_function_result_ends_when_either_view_argument_grows.em` | R6, R8 | BRW-17 | precise |
| `R6/reject_a_method_result_points_into_its_whole_object.em` | R6, R8 | BRW-1 | precise |
| `R6/reject_a_push_to_the_list_a_stored_generator_walks.em` | R6, R8 | BRW-18, CTL-2 | precise |
| `R6/reject_a_reshape_of_a_list_the_argument_owns_ends_the_view.em` | R6, R8 | BRW-17 | precise |
| `R6/reject_a_result_outside_what_its_from_names.em` | R6 | LT-1a | precise |
| `R6/reject_a_word_of_a_local_parser_returned_without_from.em` | R6, R8 | BRW-19 | precise |
| `R6/reject_borrows_is_now_from.em` | R6 | ATT-1, PHIL-12 | precise |
| `R6/reject_from_names_a_parameter_and_a_field.em` | R6, R7 | LT-45, TYP-15 | precise |
| `R6/reject_the_element_written_through_its_list_while_a_ref_mut_result_is_used.em` | R6, R8 | BRW-1 | precise |
| `R7/accept_a_camera_owning_a_share_may_be_returned.em` | R7, R9, R5 | LT-45, RC-1, FN-4 | main |
| `R7/accept_a_class_keeps_views_into_its_own_let_field.em` | R7 | LT-45 | main |
| `R7/accept_a_class_view_field_from_static.em` | R7 | LT-45 | main |
| `R7/accept_a_list_of_views_of_an_argument_may_be_returned.em` | R7, R6 | LT-45, LT-1 | main |
| `R7/accept_a_local_list_of_views.em` | R7, R8 | LT-45, BRW-16 | main |
| `R7/accept_a_method_that_changes_a_number_of_the_target_inside_a_loop_over_its_list.em` | R7, R8, R5 | BRW-18, CTL-2 | twin |
| `R7/accept_a_push_keeps_a_kept_link_to_an_object_in_its_own_block.em` | R7, R8 | LT-45, BRW-16 | main |
| `R7/accept_a_shared_holder_of_a_view_of_an_argument_may_be_returned.em` | R7, R9, R6 | LT-45 | twin |
| `R7/accept_a_struct_holding_a_view_needs_no_view_mark.em` | R7 | LT-45 | main |
| `R7/accept_a_struct_holding_a_view_of_an_argument_may_be_returned.em` | R7, R6 | LT-45 | twin |
| `R7/accept_a_struct_whose_view_field_is_from_static_is_not_a_view.em` | R7 | LT-45 | main |
| `R7/accept_from_naming_a_field_after_renaming_the_parameter.em` | R7, R6 | LT-45, LT-1 | main |
| `R7/accept_from_naming_a_let_array_field.em` | R7 | LT-45, TYP-15 | twin |
| `R7/accept_from_naming_a_let_owns_field.em` | R7, R3 | LT-45, TYP-15 | twin |
| `R7/accept_handing_over_an_object_while_a_copy_of_its_word_is_used.em` | R7, R1 | BRW-17 | twin |
| `R7/accept_reading_again_after_another_holder_reshapes.em` | R7, R8 | LT-45, BRW-16 | main |
| `R7/accept_removing_after_the_kept_links_last_use.em` | R7, R8 | LT-45, BRW-16 | main |
| `R7/accept_sharedref_mut_holders_and_the_owner_all_change_one_player.em` | R7, R8 | LT-45, BRW-16 | main |
| `R7/accept_text_written_in_the_program_in_a_from_field.em` | R7 | LT-45 | twin |
| `R7/accept_the_owner_after_the_kept_ref_mut_holders_last_use.em` | R7, R8 | LT-45, BRW-16 | main |
| `R7/accept_the_registry_changes_after_the_kept_views_last_use.em` | R7, R8 | LT-45, BRW-16 | main |
| `R7/reject_a_change_of_what_kept_ref_views_read.em` | R7, R8 | BRW-1 | precise |
| `R7/reject_a_list_of_views_returned_past_what_it_views.em` | R7 | LT-45 | precise |
| `R7/reject_a_method_that_may_change_the_target_inside_a_loop_over_it.em` | R7, R8, R5 | BRW-18, CTL-2 | precise |
| `R7/reject_a_push_moves_the_inline_players_a_kept_link_points_at.em` | R7, R8 | BRW-17 | precise |
| `R7/reject_a_push_through_one_holder_while_a_loop_walks_through_another.em` | R7, R8 | BRW-18, CTL-2 | precise |
| `R7/reject_a_read_through_the_owner_while_a_kept_ref_mut_field_is_used.em` | R7, R8 | BRW-1 | precise |
| `R7/reject_a_shared_holder_of_a_view_returned_past_its_text.em` | R7, R9 | LT-45 | precise |
| `R7/reject_a_sharedref_mut_field_of_a_class_without_from.em` | R7 | LT-45, TYP-15 | precise |
| `R7/reject_a_sharedref_mut_holder_returned_past_its_target.em` | R7 | LT-45 | precise |
| `R7/reject_a_struct_holding_a_view_returned_past_its_source.em` | R7 | LT-45 | precise |
| `R7/reject_a_view_field_of_a_class_without_from.em` | R7 | LT-45, TYP-15 | precise |
| `R7/reject_a_view_into_a_part_used_after_another_holder_reshapes_it.em` | R7, R8 | BRW-17 | precise |
| `R7/reject_a_view_of_caller_text_in_a_from_static_field.em` | R7 | LT-45 | precise |
| `R7/reject_changing_the_let_field_views_point_into.em` | R7 | CLS-9 | precise |
| `R7/reject_from_naming_a_field_that_is_not_let.em` | R7 | LT-45, TYP-15 | precise |
| `R7/reject_from_naming_a_fixed_array_field.em` | R7 | LT-45, TYP-15 | precise |
| `R7/reject_from_naming_an_inline_struct_field.em` | R7 | LT-45, TYP-15 | precise |
| `R7/reject_from_naming_both_a_parameter_and_a_field.em` | R7, R6 | LT-45, TYP-15 | precise |
| `R7/reject_handing_over_an_object_while_a_view_read_out_of_it_is_used.em` | R7, R8, R1 | BRW-17 | precise |
| `R7/reject_removing_a_player_a_kept_link_points_at.em` | R7, R8 | BRW-17 | precise |
| `R7/reject_sharedref_without_mut.em` | R7 | LT-46 | precise |
| `R7/reject_storing_a_view_of_other_text_in_a_from_field.em` | R7 | LT-45 | precise |
| `R7/reject_two_kept_ref_mut_fields_of_one_player.em` | R7, R8, R5 | BRW-1 | precise |
| `R7/reject_view_on_a_struct_whose_only_view_field_is_from_static.em` | R7 | TYP-34 | precise |
| `R7/reject_view_on_a_type_that_holds_no_view.em` | R7 | TYP-34 | precise |
| `R8/accept_a_change_after_the_generators_last_use.em` | R8, R6 | BRW-16, LT-1 | main |
| `R8/accept_a_change_after_the_read_only_names_last_use.em` | R8 | BRW-16 | main |
| `R8/accept_a_change_reaches_the_enemy_the_line_names.em` | R8, R6 | BRW-16, LT-1 | main |
| `R8/accept_a_change_through_the_ref_mut_name.em` | R8 | BRW-16 | main |
| `R8/accept_a_copy_of_your_own_is_not_restricted.em` | R8, R3 | BRW-16, OWN-8 | main |
| `R8/accept_a_copy_out_of_a_guarded_field.em` | R8, R15 | BRW-19 | twin |
| `R8/accept_a_counting_loop_may_add.em` | R8 | BRW-16 | main |
| `R8/accept_a_function_that_ends_an_enemy_called_after_a_walk_of_log.em` | R8, R5 | BRW-18, CTL-2 | twin |
| `R8/accept_a_function_value_that_changes_nothing_runs_inside_a_loop.em` | R8, R12 | BRW-18, CTL-2 | twin |
| `R8/accept_a_library_call_that_changes_the_list_after_the_loop.em` | R8, R12 | BRW-16, CLO-1 | main |
| `R8/accept_a_link_opened_in_a_walk_and_the_loop_name_both_read.em` | R8, R13 | BRW-1 | twin |
| `R8/accept_a_link_opened_in_a_walk_proved_different_from_the_loop_name.em` | R8, R13, R5 | BRW-16, GRM-44, FN-4 | main |
| `R8/accept_a_link_proved_different_from_the_ref_mut_parameter.em` | R8, R13, R5 | BRW-16, GRM-44, FN-4 | main |
| `R8/accept_a_list_of_views_of_a_temporary.em` | R8, R7 | BRW-16, LT-45 | main |
| `R8/accept_a_method_on_a_name_opened_from_a_link_changes_this_before_clearing.em` | R8, R4, R5 | BRW-21 | twin |
| `R8/accept_a_method_on_a_temporary_that_also_changes_a_module_variable.em` | R8, R5 | BRW-16, FN-4 | main |
| `R8/accept_a_method_on_a_temporary_whose_result_is_used.em` | R8 | BRW-16 | main |
| `R8/accept_a_method_that_changes_another_field_inside_the_loop.em` | R8, R5 | BRW-16, FN-4 | main |
| `R8/accept_a_method_that_changes_hp_while_a_ref_mut_name_of_name_is_used.em` | R8, R5 | BRW-16, FN-4 | main |
| `R8/accept_a_method_that_reads_its_object_before_clearing_the_list_holding_it.em` | R8, R4, R5 | BRW-21 | twin |
| `R8/accept_a_payload_view_used_before_the_variant_changes.em` | R8, R13 | BRW-16, GRM-44 | main |
| `R8/accept_a_position_is_found_again_after_a_push.em` | R8, R4 | BRW-16, FN-1 | main |
| `R8/accept_a_position_kept_across_a_call_that_only_adds.em` | R8, R17 | BRW-16, BLD-14 | main |
| `R8/accept_a_position_named_again_after_a_removal.em` | R8, R2 | BRW-16, BRW-12 | main |
| `R8/accept_a_push_after_a_loop_over_a_strings_characters.em` | R8 | BRW-18, CTL-2 | twin |
| `R8/accept_a_push_through_a_shares_mut_local_after_the_ref_mut_names_last_use.em` | R8, R9 | BRW-1 | twin |
| `R8/accept_a_read_only_position_is_found_again_after_a_push.em` | R8 | BRW-16 | main |
| `R8/accept_a_shares_owner_sees_a_change_through_another_owner.em` | R8, R9, R3 | BRW-16, RC-1, OWN-8 | main |
| `R8/accept_a_span_given_to_a_function_that_reads_it_before_growing_what_it_shows.em` | R8, R4 | BRW-21 | twin |
| `R8/accept_a_temporary_lives_until_its_views_last_use.em` | R8 | BRW-16 | main |
| `R8/accept_a_view_survives_a_push_to_a_fresh_object.em` | R8, R9, R5 | BRW-16, RC-1, FN-4 | main |
| `R8/accept_a_view_survives_a_push_to_an_object_known_to_differ.em` | R8, R9, R5 | BRW-16, RC-1, FN-4 | main |
| `R8/accept_a_view_survives_a_push_to_an_object_of_another_type.em` | R8, R9, R5 | BRW-16, RC-1, FN-4 | main |
| `R8/accept_a_view_whose_last_use_is_an_argument_does_not_live_across_the_call.em` | R8, R4 | BRW-16, FN-1 | main |
| `R8/accept_a_while_loop_on_the_current_length_may_add_and_remove.em` | R8 | BRW-16 | main |
| `R8/accept_an_add_after_a_loop_over_a_set.em` | R8 | BRW-18, CTL-2 | twin |
| `R8/accept_an_insert_after_a_loop_over_a_maps_keys.em` | R8 | BRW-16 | main |
| `R8/accept_an_opened_name_used_before_its_option_is_replaced.em` | R8, R13 | BRW-16, GRM-44 | main |
| `R8/accept_another_field_changes_while_a_from_log_generator_is_used.em` | R8, R6 | BRW-16, LT-1 | main |
| `R8/accept_another_name_after_the_ref_mut_names_last_use.em` | R8 | BRW-16 | main |
| `R8/accept_changes_go_after_the_loop.em` | R8, R12 | BRW-16, CLO-1 | main |
| `R8/accept_copied_keys_outlive_a_change_of_the_map.em` | R8 | BRW-16 | main |
| `R8/accept_copies_out_of_a_lock.em` | R8, R4, R5 | BRW-16, FN-1, FN-4 | main |
| `R8/accept_ecs_changes_are_queued_while_a_query_walks.em` | R8, R12 | BRW-16, CLO-1 | main |
| `R8/accept_ending_an_object_whose_drop_pushes_to_log_after_the_walk.em` | R8, R5 | BRW-18, CTL-2 | twin |
| `R8/accept_reading_the_list_again_after_a_push.em` | R8, R7 | BRW-16, LT-45 | main |
| `R8/accept_returning_your_own_copy_of_a_temporarys_view.em` | R8 | BRW-16 | main |
| `R8/accept_the_loops_own_handle_changes_the_value_under_each_key.em` | R8, R3 | BRW-18, CTL-2 | twin |
| `R8/accept_the_loops_own_handle_writes_the_element_it_is_on.em` | R8, R2 | BRW-18, CTL-2 | twin |
| `R8/accept_two_ref_mut_names_proved_apart.em` | R8, R4 | BRW-1, BRW-11 | twin |
| `R8/accept_your_own_copy_of_a_strings_text_outlives_its_replacement.em` | R8 | BRW-16 | main |
| `R8/accept_your_own_copy_outlives_a_push.em` | R8, R6 | BRW-16, LT-1 | main |
| `R8/accept_your_own_text_outlives_a_function_value_that_renames_its_node.em` | R8, R10, R12 | BRW-17 | twin |
| `R8/reject_a_call_through_a_function_value_given_the_list_ends_a_position.em` | R8, R12 | BRW-17 | precise |
| `R8/reject_a_call_through_a_function_value_that_changes_the_walked_list.em` | R8, R12 | BRW-18, CTL-2 | precise |
| `R8/reject_a_change_through_another_name_while_a_ref_mut_name_is_used.em` | R8 | BRW-1 | precise |
| `R8/reject_a_change_to_a_list_a_generator_walking_by_position_walks.em` | R8, R6 | BRW-18, CTL-2 | precise |
| `R8/reject_a_change_to_a_list_a_stored_generator_walks.em` | R8, R6 | BRW-18, CTL-2 | precise |
| `R8/reject_a_change_to_a_temporary_that_would_be_lost.em` | R8, R6 | BRW-19 | precise |
| `R8/reject_a_change_to_the_list_a_loop_over_a_generator_walks.em` | R8, R6 | BRW-18, CTL-2 | precise |
| `R8/reject_a_change_while_a_read_only_name_is_used.em` | R8 | BRW-1 | precise |
| `R8/reject_a_change_while_a_ref_view_of_a_number_is_used.em` | R8, R2 | BRW-1 | precise |
| `R8/reject_a_function_that_ends_an_enemy_called_inside_a_walk_of_log.em` | R8, R5 | BRW-18, CTL-2 | precise |
| `R8/reject_a_helper_that_changes_an_element_while_a_ref_mut_position_is_used.em` | R8 | BRW-1 | precise |
| `R8/reject_a_helper_that_may_remove_ends_a_position.em` | R8, R17 | BRW-17 | precise |
| `R8/reject_a_helper_that_reads_and_removes_ends_a_position.em` | R8 | BRW-17 | precise |
| `R8/reject_a_library_call_given_a_function_that_changes_the_walked_list.em` | R8, R12 | BRW-18, CTL-2 | precise |
| `R8/reject_a_link_opened_in_a_ref_mut_walk_reads_the_loop_names_player.em` | R8, R13 | BRW-1 | precise |
| `R8/reject_a_link_opened_in_a_walk_changes_what_the_loop_name_reads.em` | R8, R13, R5 | BRW-1 | precise |
| `R8/reject_a_link_that_may_be_the_ref_mut_parameters_object.em` | R8, R13, R5 | BRW-1 | precise |
| `R8/reject_a_map_insert_ends_a_view_of_its_keys.em` | R8 | BRW-17 | precise |
| `R8/reject_a_method_on_a_name_opened_from_a_link_that_clears_its_own_list.em` | R8, R4, R5 | BRW-21 | precise |
| `R8/reject_a_method_that_changes_hp_while_a_ref_mut_name_of_hp_is_used.em` | R8, R5, R4 | BRW-1, BRW-11 | precise |
| `R8/reject_a_method_that_changes_the_walked_field_inside_the_loop.em` | R8, R5 | BRW-18, CTL-2 | precise |
| `R8/reject_a_method_that_may_grow_the_list_ends_a_view.em` | R8, R5, R9 | BRW-17 | precise |
| `R8/reject_a_method_that_reads_its_object_after_clearing_the_list_holding_it.em` | R8, R4, R5 | BRW-21 | precise |
| `R8/reject_a_push_ends_a_kept_view.em` | R8, R7 | BRW-17 | precise |
| `R8/reject_a_push_ends_a_view_a_call_gave.em` | R8, R6 | BRW-17 | precise |
| `R8/reject_a_push_through_a_shares_mut_local_while_a_ref_mut_name_of_its_member_is_used.em` | R8, R9 | BRW-1 | precise |
| `R8/reject_a_push_through_another_link_of_the_same_type_ends_a_view.em` | R8, R9, R5 | BRW-17 | precise |
| `R8/reject_a_push_while_a_for_loop_walks_the_list.em` | R8 | BRW-18, CTL-2 | precise |
| `R8/reject_a_push_while_a_loop_walks_a_strings_characters.em` | R8 | BRW-18, CTL-2 | precise |
| `R8/reject_a_read_through_another_name_while_a_ref_mut_name_is_used.em` | R8 | BRW-1 | precise |
| `R8/reject_a_ref_mut_name_beside_a_read_only_name_whose_indexes_may_be_equal.em` | R8, R4 | BRW-1, BRW-11 | precise |
| `R8/reject_a_removal_ends_a_position.em` | R8 | BRW-17 | precise |
| `R8/reject_a_removal_inside_a_counting_loop.em` | R8 | BRW-18, CTL-2 | precise |
| `R8/reject_a_removal_while_a_for_loop_walks_the_list.em` | R8 | BRW-18, CTL-2 | precise |
| `R8/reject_a_reorder_ends_a_position.em` | R8 | BRW-17 | precise |
| `R8/reject_a_span_given_to_a_function_that_grows_what_it_shows.em` | R8, R4 | BRW-21 | precise |
| `R8/reject_a_value_under_a_key_written_while_a_loop_walks_the_maps_keys.em` | R8 | BRW-18, CTL-2 | precise |
| `R8/reject_a_view_of_a_guard_kept_past_its_block.em` | R8, R2 | BRW-19 | precise |
| `R8/reject_a_view_of_a_guarded_field_returned_out_of_its_lock.em` | R8, R15 | BRW-19 | precise |
| `R8/reject_a_view_of_text_used_after_a_function_value_renames_its_node.em` | R8, R10, R12 | BRW-17 | precise |
| `R8/reject_a_view_returned_out_of_a_lock.em` | R8 | BRW-19 | precise |
| `R8/reject_a_view_returned_out_of_a_read_lock.em` | R8 | BRW-19 | precise |
| `R8/reject_a_write_of_an_element_while_a_for_loop_walks_the_list.em` | R8 | BRW-18, CTL-2 | precise |
| `R8/reject_an_add_while_a_loop_walks_a_set.em` | R8 | BRW-18, CTL-2 | precise |
| `R8/reject_an_insert_while_a_loop_walks_a_maps_keys.em` | R8 | BRW-18, CTL-2 | precise |
| `R8/reject_an_insertion_ends_a_position.em` | R8 | BRW-17 | precise |
| `R8/reject_an_opened_name_used_after_its_option_is_replaced.em` | R8, R13 | BRW-17 | precise |
| `R8/reject_another_field_changed_while_a_generator_without_from_is_used.em` | R8, R6 | BRW-1 | precise |
| `R8/reject_changing_the_variant_ends_a_view_of_its_payload.em` | R8, R13 | BRW-17 | precise |
| `R8/reject_clearing_ends_a_position.em` | R8 | BRW-17 | precise |
| `R8/reject_ending_an_object_whose_drop_pushes_to_the_walked_list.em` | R8, R5 | BRW-18, CTL-2 | precise |
| `R8/reject_keeping_a_view_of_a_temporary_past_its_loop_step.em` | R8, R7 | LT-45, BRW-19 | precise |
| `R8/reject_replacing_a_string_ends_a_view_of_its_text.em` | R8 | BRW-17 | precise |
| `R8/reject_replacing_the_list_ends_a_position.em` | R8, R2 | BRW-17 | precise |
| `R8/reject_returning_a_list_of_views_of_a_temporary.em` | R8, R7 | LT-45, BRW-19 | precise |
| `R8/reject_returning_a_view_of_a_temporary.em` | R8 | BRW-19 | precise |
| `R8/reject_two_ref_mut_names_of_one_element.em` | R8 | BRW-1 | precise |
| `R8/reject_two_ref_mut_names_of_one_list_whose_indexes_may_be_equal.em` | R8, R4 | BRW-1, BRW-11 | precise |
| `R8/run_a_while_loop_index_is_checked.em` | R8 | BRW-16 | main |
| `R9/accept_a_change_through_a_shares_mut_owner.em` | R9, R3, R5 | BRW-15 | twin |
| `R9/accept_a_field_change_through_a_shares_mut_field.em` | R9, R5 | RC-1, FN-4 | main |
| `R9/accept_a_number_changed_through_one_owner_while_a_loop_walks_the_other.em` | R9, R8, R5 | RC-1, BRW-16, FN-4 | main |
| `R9/accept_a_number_in_a_sharedref_target_changed_while_another_holder_walks_its_list.em` | R9, R7, R8, R5 | RC-1, LT-45, BRW-16, FN-4 | main |
| `R9/accept_a_ref_mut_name_of_a_shares_mut_link_changes_the_player.em` | R9, R10, R5 | BRW-15 | twin |
| `R9/accept_a_scene_two_panels_own_and_change.em` | R9, R5 | MOD-3 | twin |
| `R9/accept_a_shared_list_element_changed_through_its_ref_mut_name.em` | R9, R8, R5 | RC-1, BRW-16, FN-4 | main |
| `R9/accept_a_thread_shared_object_filled_where_it_is_made.em` | R9, R15 | RC-1, THR-1 | main |
| `R9/accept_a_thread_shared_owner_reaching_another_thread.em` | R9, R15 | RC-1, THR-1 | main |
| `R9/accept_a_view_used_before_another_owner_removes_its_element.em` | R9, R8, R5 | RC-1, BRW-16, FN-4 | main |
| `R9/accept_moving_items_between_two_shared_players_proved_apart.em` | R9, R8, R4, R5 | RC-1, BRW-16, FN-1, FN-4 | main |
| `R9/accept_one_scoped_task_sets_a_player_through_a_sharedref_link.em` | R9, R15 | THR-11, THR-21, THR-1, THR-2, THR-10 | twin |
| `R9/reject_a_change_through_a_shares_owner.em` | R9, R3, R5 | BRW-15 | precise |
| `R9/reject_a_field_change_through_a_shares_field.em` | R9, R5 | BRW-15 | precise |
| `R9/reject_a_list_changed_through_one_owner_while_a_loop_walks_the_other.em` | R9, R8 | BRW-18, CTL-2 | precise |
| `R9/reject_a_list_in_a_sharedref_target_changed_while_another_holder_walks_it.em` | R9, R7, R8 | BRW-18, CTL-2 | precise |
| `R9/reject_a_plain_parameter_changes_a_player_behind_a_shares_link.em` | R9, R5, R4 | BRW-15 | precise |
| `R9/reject_a_shared_list_element_changed_while_a_view_reads_it.em` | R9, R8 | BRW-1 | precise |
| `R9/reject_a_shares_owner_reaching_another_thread.em` | R9, R15 | THR-1, THR-2, THR-10 | precise |
| `R9/reject_a_thread_shared_object_changed_directly.em` | R9, R15 | THR-1 | precise |
| `R9/reject_a_view_used_after_another_owner_removed_its_element.em` | R9, R8 | BRW-17 | precise |
| `R9/reject_an_unscoped_thread_holding_a_sharedref_link.em` | R9, R15, R7 | THR-1, THR-2, THR-10 | precise |
| `R9/reject_moving_items_between_two_shared_players_that_may_be_one.em` | R9, R8, R4 | BRW-18, CTL-2 | precise |
| `R9/reject_refcell_is_not_ember.em` | R9 | MOD-3 | precise |
| `R9/reject_two_tasks_set_one_player_through_two_sharedref_links.em` | R9, R15 | THR-11, THR-21 | precise |
| `R9/run_a_shared_object_ends_with_its_last_owner.em` | R9 | RC-1 | main |
| `R9/run_a_shares_owner_sees_what_the_first_owner_changes.em` | R9, R3, R8 | RC-1, OWN-8, BRW-16 | main |
| `R9/run_a_thread_shared_object_read_by_a_thread_and_its_maker.em` | R9, R15 | RC-1, THR-1 | main |
| `R9/run_a_tree_child_with_two_owners_changed_through_either.em` | R9, R3 | RC-1, OWN-8 | main |
| `R9/run_fail_too_many_owners_of_one_player_stop_the_program.em` | R9 | RC-1 | main |
| `R9/run_real_programs_keep_32_bit_counts.em` | R9, R11 | RC-1, WK-11 | twin |
| `R9/run_two_sharedref_mut_holders_and_the_owner_change_one_player.em` | R9, R7 | RC-1, LT-45 | main |
| `R9/run_two_shares_mut_owners_both_change_one_list.em` | R9, R3 | RC-1, OWN-8 | main |
| `R9/run_two_shares_mut_owners_change_one_counter_with_no_run_time_check.em` | R9 | RC-1 | main |
| `R9/run_two_tasks_take_hp_off_one_player_through_two_sharedref_links.em` | R9, R15, R7 | THR-11, THR-21 | twin |
