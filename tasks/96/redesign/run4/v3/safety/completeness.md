# Completeness check of safety validation (b) (agent C1)

What no writer covered. Inputs: `V3/rules.md` (cited as "line N"), the six `S/matrix-W<n>.md`, and every
file under `V3/conformance/`. `V3` = `tasks/96/redesign/run4/v3/`, `S` = `V3/safety/`. Nothing was
compiled or run. The index of the tests (kind, rules, the codes on their marked lines, the panic) was
made by `target/redesign5/C1/index.py` into `target/redesign5/C1/index.txt`; the code table below is
`target/redesign5/C1/codes_table.md`.

**What is there.** 687 files: 681 tests (346 `compile-fail`, 35 `compile-pass`, 291 `run-pass`, 9
`run-fail`; 346 `reject_`, 285 `accept_`, 50 `run_`), the module `R12/widgets.em`, the package
`R17/ledger/` (`src/lib.em` and its `ember.toml`) with `R17/ember.toml`, and the C side of the
trampoline tests, `R15/kept_events.c` and `R15/kept_events.h`. Many `accept_` files are `run-pass` and
print what they did.

**How to read a gap.** Each gap (G1 to G27, section 4; there is no G9: the `@cached` run test is
G1's) gives the rule and its line, why it matters, the
test to add (its name, the folder it belongs in, and the program), and the expected result. Where the
rules settle the result the gap says so; where they do not, it says **unsettled** and the question is in
section 5 (and in the form's `for_owner`), not decided here. Messages are given in R18's form without
line numbers: the writer adds the finished file's line numbers.

## 1. Every error code of section 4 against the `reject_` tests

Columns: the code, how many `reject_` tests mark it, how many other tests (warnings are written as
`accept_..._is_warned`, `compile-pass`), and one test that has it.

| code | `reject_` | other | one test |
|---|---|---|---|
| E0005 | 1 | 0 | `R15/reject_async_is_reserved.em` |
| E0104 | 3 | 0 | `R15/reject_the_sync_class_mark.em` |
| E0109 | 4 | 0 | `R3/reject_owns_before_a_returned_value.em` |
| E0113 | 3 | 0 | `R2/reject_mut_alone_on_a_pattern_name.em` |
| E0114 | 1 | 0 | `R7/reject_sharedref_without_mut.em` |
| E0115 | 2 | 0 | `R13/reject_some_in_a_while_header.em` |
| E0116 | 4 | 0 | `R13/reject_an_opened_pair_name_with_no_word_after_one_with_a_word.em` |
| E0117 | 2 | 0 | `R4/reject_ref_in_a_parameters_type.em` |
| E1010 | 13 | 0 | `R11/reject_upgrade_is_not_ember.em` |
| E1030 | 1 | 0 | `R15/reject_a_module_level_assignment.em` (**not section 4's meaning**: G4) |
| E2020 | 7 | 0 | `R11/reject_a_link_used_as_its_target_without_opening.em` |
| E2036 | 3 | 0 | `R13/reject_exists_as_on_a_result.em` |
| E2072 | 1 | 0 | `R14/reject_a_generic_body_calling_a_method_no_interface_declares.em` |
| E2085 | 2 | 0 | `R3/reject_owns_on_a_captured_number.em` |
| E2100 | 2 | 0 | `R5/reject_a_helper_with_no_reads_called_before_every_field_is_set.em` |
| E2103 | 1 | 0 | `R5/reject_this_given_another_object.em` |
| E2104 | 1 | 0 | `R7/reject_changing_the_let_field_views_point_into.em` |
| E2105 | 45 | 0 | `R12/reject_a_function_that_says_changes_world_but_never_changes_frame.em` |
| E2106 | 2 | 0 | `R13/reject_an_opened_name_already_in_use.em` |
| E2107 | 3 | 0 | `R5/reject_changes_this_beside_a_field.em` |
| E2108 | 1 | 0 | `R5/reject_changes_this_on_init.em` |
| E2112 | 1 | 0 | `R5/reject_virtual_in_a_class_that_cannot_be_subclassed.em` |
| E2113 | 11 | 0 | `R12/reject_a_counting_handler_given_to_a_plain_death_handler.em` |
| E2114 | 2 | 0 | `R3/reject_owns_on_a_class_type.em` |
| E2115 | 1 | 0 | `R5/reject_static_fn_outside_a_type.em` |
| E2116 | 5 | 0 | `R11/reject_copy_of_a_link.em` |
| **E2117** | **0** | 0 | **none: G1** |
| E2118 | 6 | 0 | `R6/reject_from_names_a_parameter_and_a_field.em` |
| E2140 | 1 | 0 | `R8/reject_a_change_to_a_temporary_that_would_be_lost.em` |
| E2173 | 2 | 0 | `R14/reject_a_bare_interface_name_as_a_type.em` |
| E2200 | 3 | 0 | `R11/reject_a_struct_node_keeping_a_view_of_its_parent.em` |
| E2221 | 2 | 0 | `R15/reject_a_generator_walk_held_by_views_of_its_own_frame.em` |
| E3013 | 7 | 0 | `R1/reject_a_plain_parameter_pushed_into_a_list.em` |
| E3020 | 32 | 0 | `R8/reject_a_push_while_a_for_loop_walks_the_list.em` |
| E3021 | 19 | 0 | `R8/reject_a_change_while_a_read_only_name_is_used.em` |
| E3022 | 11 | 0 | `R4/reject_the_same_body_given_twice.em` |
| E3023 | 34 | 0 | `R1/reject_a_change_through_a_view_given_with_equals.em` |
| E3026 | 3 | 0 | `R12/reject_a_handler_that_outlives_the_callers_total.em` |
| E3027 | 4 | 0 | `R4/reject_ref_mut_given_from_a_plain_parameter.em` |
| E3028 | 5 | 0 | `R4/reject_a_call_without_ref_mut_for_a_ref_mut_parameter.em` |
| E3040 | 11 | 0 | `R1/reject_a_list_used_after_it_was_handed_over.em` |
| E3060 | 26 | 0 | `R8/reject_a_removal_ends_a_position.em` |
| E3062 | 1 | 0 | `R6/reject_a_result_outside_what_its_from_names.em` |
| E3063 | 15 | 0 | `R8/reject_returning_a_view_of_a_temporary.em` |
| E3100 | 3 | 0 | `R16/reject_an_unsafe_operation_in_an_unsafe_fn_body_outside_a_block.em` |
| E4030 | 2 | 0 | `R16/reject_an_index_compare_inside_no_alias_checks.em` |
| **E5013** | **0** | 0 | **none: G2** |
| E5069 | 1 | 0 | `R15/reject_no_callbacks_on_a_function_that_takes_a_callback.em` |
| E7004 | 3 | 0 | `R15/reject_an_unscoped_thread_using_a_name_with_no_capture_word.em` |
| E7007 | 6 | 0 | `R15/reject_two_tasks_change_a_field_without_a_lock.em` |
| E7008 | 1 | 0 | `R15/reject_a_lock_guard_sent_to_a_task.em` |
| E7010 | 2 | 0 | `R15/reject_a_parallel_loop_over_shares_mut_objects.em` |
| E7012 | 2 | 0 | `R15/reject_a_thread_changes_an_unguarded_static_mut.em` |
| E7013 | 4 | 0 | `R15/reject_a_guarded_field_changed_outside_its_lock.em` |
| E7014 | 1 | 0 | `R9/reject_a_thread_shared_object_changed_directly.em` |
| E7015 | 4 | 0 | `R15/reject_a_thread_captures_a_one_thread_owner.em` |
| E7016 | 1 | 0 | `R15/reject_a_parallel_while_loop.em` |
| E7017 | 3 | 0 | `R3/reject_thread_shared_on_a_value_not_made_thread_shared.em` |
| E7018 | 4 | 0 | `R15/reject_a_join_inside_the_lock_its_thread_needs.em` |
| L3001 (warning) | 0 | 2 | `R11/accept_a_cycle_of_shares_owners_is_warned.em` |
| L3015 (warning) | 0 | 1 | `R16/accept_an_unsafe_fn_without_safety_is_warned.em` |
| **W2190 (warning)** | 0 | **0** | **none: G3** |
| W3012 (warning) | 0 | 1 | `R16/accept_an_unsafe_block_without_a_safety_note_is_warned.em` |
| W3013 (warning) | 0 | 1 | `R16/accept_a_safety_note_on_an_unsafe_fn_line_is_warned.em` |

Codes the tests use that section 4 does not list (it says "one meaning each", line 894): E0100
(`R15/reject_python_global.em`, AN-76), E2030 (`R7/reject_view_on_a_type_that_holds_no_view.em`),
E2040 (`R14/reject_a_pub_generic_without_its_written_bound.em`,
`R17/reject_a_pub_generic_without_its_bounds.em`), E2050
(`R14/reject_a_generic_method_that_returns_its_type_parameter_through_dyn.em`). Not a gap in the tests:
a gap in section 4 (section 5, item 10).

## 2. Every refusal of section 3 against the `reject_` tests

One line per row of the table (lines 809 to 882) and the warnings (884 to 892). "Covered" names one
`reject_` test per message family; a message with no test is a gap.

| line | refusal | code | covered by | gap |
|---|---|---|---|---|
| 809 | a name used after a hand-over | E3040 | `R1/reject_a_list_used_after_it_was_handed_over.em` | — |
| 810 | a change through a name that only reads | E3023 | `R2/reject_a_change_through_a_plain_pattern_name.em`, `R4/reject_a_plain_parameter_changes_the_callers_counter.em` | — |
| 811 | `=` on a parameter, loop, pattern or opened name; another ref to a `const ref mut` | E3023 | `R2/reject_equals_on_a_view_parameter.em`, `R2/reject_equals_on_a_number_parameter.em`, `R2/reject_equals_on_a_loop_name.em`, `R2/reject_equals_on_a_pattern_name.em`, `R2/reject_a_change_of_a_plain_opened_name.em`, `R2/reject_another_ref_given_to_a_const_ref_mut_name.em` | — |
| 812 | a changing method through a read-only name | E3023 | `R5/reject_a_changing_method_called_through_a_plain_loop_name.em` | — |
| 813 | a changing owner made from a reading owner | E3023 | `R3/reject_a_changing_owner_made_from_a_reading_owner.em` | — |
| 814 | a changing owner made from a plain `shares mut` parameter | E3023 | `R4/reject_an_owner_that_changes_made_from_a_plain_parameter.em` | — |
| 815 | a write through a plain name holding a `MutSpan` | E3023 | `R4/reject_a_plain_mutspan_parameter_writes.em`, `R2/reject_a_write_through_a_plain_name_holding_a_mut_span.em` | — |
| 816 | a change of what a read-only name reads | E3021 | `R8/reject_a_change_while_a_read_only_name_is_used.em` | — |
| 817 | a walk with views across a call that may change nodes | E3021 | `R11/reject_a_walk_with_views_across_a_visit_that_may_change_nodes.em` | — |
| 818 | a body that does not match its first line | E2105 | `R5/reject_a_caller_does_not_name_what_its_callee_changes.em`, `R5/reject_a_method_changes_a_field_its_line_does_not_name.em`, `R5/reject_a_reads_method_reads_a_field_it_does_not_name.em` | — |
| 819 | a lambda or a function running one that does not name a change | E2105 | `R12/reject_a_lambda_that_pushes_to_log_without_saying_so.em`, `R12/reject_row_6_handler_without_changes_score.em`, `R12/reject_for_each_given_a_lambda_that_does_not_say_it_changes_log.em`, `R12/reject_a_thread_function_that_pushes_under_the_log_lock_without_changes_log.em`, `R12/reject_click_without_log.em`, `R12/reject_click_without_captured.em` | — |
| 820 | `captured` on a lambda's own first line | E2105 | `R12/reject_captured_on_a_lambdas_own_first_line.em` | — |
| 821 | a thread or a kept C callback whose changes are not named | E2105 | `R15/reject_main_starts_a_thread_without_naming_what_it_changes.em`, `R15/reject_setup_hands_c_a_callback_without_naming_its_changes.em` | — |
| 822 | `changes this` / `owns this` beside `reads`, a field, or each other | E2107 | `R5/reject_changes_this_beside_reads.em`, `R5/reject_changes_this_beside_a_field.em`, `R5/reject_owns_this_beside_changes_this.em` | — |
| 823 | `changes this` on `init` | E2108 | `R5/reject_changes_this_on_init.em` | — |
| 824 | `this` given another object | E2103 | `R5/reject_this_given_another_object.em` | — |
| 825 | `self` | E1010 | `R5/reject_self_names_a_methods_object.em` | — |
| 826 | `this` where there is no object: a free function, a `static fn` | E1010 | `R5/reject_this_in_a_static_fn.em` (the `static fn` only) | **G7** (a free function) |
| 827 | `static fn` outside a type | E2115 | `R5/reject_static_fn_outside_a_type.em` | — |
| 828 | `mut` alone on a name | E0113 | `R2/reject_mut_alone_on_a_pattern_name.em`, `R4/reject_mut_alone_on_a_parameter.em`, `R5/reject_mut_this_on_a_methods_first_line.em` | — |
| 829 | `ref` / `ref mut` in a parameter's type | E0117 | `R4/reject_ref_mut_in_a_parameters_type.em`, `R4/reject_ref_in_a_parameters_type.em` | — |
| 830 | a call's word that does not match its parameter | E3028 | `R4/reject_a_call_without_ref_mut_for_a_ref_mut_parameter.em`, `R4/reject_a_mutspan_argument_without_ref_mut.em`, `R4/reject_ref_mut_at_a_call_to_a_plain_parameter.em`, `R4/reject_ref_mut_before_a_mutex_argument.em` | — |
| 831 | `ref mut` before what this function may not change: a plain parameter, a temporary, **a query item it only reads** (and, by section 4, a loop or pattern name over what it only reads) | E3027 | `R4/reject_ref_mut_given_from_a_plain_parameter.em`, `R4/reject_ref_mut_before_a_new_number.em` | **G6** (the query item; a loop name) |
| 832 | two changeable arguments or `ref mut` names of one place; its run-time stop; `retain` with a changing function; a function stored where a variable it changes owns it | E3022 | `R4/reject_the_same_body_given_twice.em` (stop: `R4/run_the_same_body_given_twice_stops_the_program.em`), `R12/reject_retain_given_a_function_that_changes_the_list.em`, `R12/reject_a_handler_stored_in_the_button_it_changes.em` | — |
| 833 | a changeable argument and another that may reach into it | E3021 | `R4/reject_an_argument_reaching_into_what_a_changeable_one_owns.em`, `R4/reject_two_tracks_of_one_map_not_proved_apart.em` | — |
| 834 | a `ref mut` name's place used through another name | E3022, E3021 | `R7/reject_two_kept_ref_mut_fields_of_one_player.em`, `R8/reject_a_read_through_another_name_while_a_ref_mut_name_is_used.em`, `R8/reject_a_link_that_may_be_the_ref_mut_parameters_object.em` | — |
| 835 | `sharedref` without `mut` | E0114 | `R7/reject_sharedref_without_mut.em` | — |
| 836 | a list changed while a `for` loop walks it, directly or through a call or a function value | E3020 | `R8/reject_a_push_while_a_for_loop_walks_the_list.em`, `R12/reject_clicking_a_log_button_inside_a_loop_over_log.em`, `R12/reject_a_for_walk_of_handlers_whose_type_changes_ui.em` | — |
| 837 | the same through a library call, a function value under a held lock, a foreign call | E3020 | `R8/reject_a_library_call_given_a_function_that_changes_the_walked_list.em`, `R15/reject_a_walk_under_the_lock_across_a_function_value_that_changes_log.em`, `R15/reject_a_c_call_inside_a_walk_a_kept_callback_may_change.em`, `R15/reject_an_unmarked_c_function_inside_the_walk.em` | (G23: the same through a `callback(borrowed)` C function) |
| 838 | a list changed while a generator walks it | E3020 | `R8/reject_a_change_to_a_list_a_stored_generator_walks.em` | — |
| 839 | a removal inside a counting loop | E3020 | `R8/reject_a_removal_inside_a_counting_loop.em` | — |
| 840 | a view used after a reshape; a position after a removal or reorder | E3060 | `R8/reject_a_removal_ends_a_position.em`, `R4/reject_a_view_put_in_through_ref_mut_used_after_its_source_grows.em`, `R8/reject_a_map_insert_ends_a_view_of_its_keys.em` | — |
| 841 | a change to a temporary that would be lost | E2140 | `R8/reject_a_change_to_a_temporary_that_would_be_lost.em` | — |
| 842 | a view of a temporary returned or kept | E3063 | `R8/reject_returning_a_view_of_a_temporary.em` | — |
| 843 | a view leaving a lock's block | E3063 | `R8/reject_a_view_returned_out_of_a_lock.em` | — |
| 844 | a view of the callee's locals put in through `ref mut` | E3063 | `R4/reject_a_view_of_the_callees_own_local_put_in_through_ref_mut.em` | — |
| 845 | a kept view that outlives what it views | E3063 | `R7/reject_a_list_of_views_returned_past_what_it_views.em` | (G20: a counted owner of a value holding a view) |
| 846 | a value made in an arena kept past the arena | E3063 | `R3/reject_a_value_returned_past_the_end_of_its_arena.em` | — |
| 847 | a result outside what its `from` names | E3062 | `R6/reject_a_result_outside_what_its_from_names.em` | — |
| 848 | a view field of a class without a usable `from` | E2118 | `R7/reject_a_view_field_of_a_class_without_from.em`, `R7/reject_from_naming_a_field_that_is_not_let.em`, `R7/reject_from_naming_a_fixed_array_field.em`, `R6/reject_from_names_a_parameter_and_a_field.em` | — |
| 849 | `copy of` what it cannot copy as itself | E2116 | `R3/reject_copy_of_a_weakref_link.em`, `R3/reject_copy_of_a_view.em`, `R3/reject_copy_of_a_value_whose_views_point_into_its_own_let_field.em`, `R3/reject_copy_of_a_type_with_no_clone.em` | (G17: a value holding a link) |
| 850 | `@cached` on a method whose result it cannot keep safely (five messages) | E2117 | **none** | **G1** |
| 851 | `Some(...)` in an `if` or `while` header | E0115 | `R13/reject_some_in_an_if_header.em` | — |
| 852 | an opening or kind test that always passes | E2036 | `R13/reject_exists_as_on_a_value_that_always_holds_one.em`, `R13/reject_exists_as_on_a_result.em`, `R14/reject_a_kind_test_that_always_passes.em` | — |
| 853 | an opening or kind test naming a name in use | E2106 | `R13/reject_an_opened_name_already_in_use.em`, `R14/reject_a_kind_test_naming_a_name_already_in_use.em` | — |
| 854 | a name with no word after one with a word | E0116 | `R3/reject_a_loop_name_with_no_word_after_one_with_a_word.em`, `R3/reject_a_captured_name_with_no_word_after_one_with_a_word.em`, `R3/reject_a_lambda_parameter_with_no_word_after_one_with_a_word.em` | — |
| 855 | a type that contains itself | E2200 | `R3/reject_a_type_that_contains_itself_with_no_block_between.em`, `R11/reject_a_struct_node_keeping_a_view_of_its_parent.em` | — |
| 856 | `owns` where it can only copy | E2085 | `R4/reject_owns_on_an_int_parameter.em`, `R3/reject_owns_on_a_captured_number.em` | — |
| 857 | `owns` before a value already handed over, or before `fn` | E0109 | `R3/reject_owns_before_a_value_its_place_already_hands_over.em`, `R3/reject_owns_before_fn.em` | — |
| 858 | `owns` on a class type; `using` with no heap memory | E2114 | `R3/reject_owns_on_a_class_type.em`, `R3/reject_using_after_a_type_with_no_heap_memory.em` | — |
| 859 | a bare interface name as a type | E2173 | `R3/reject_a_bare_interface_name_used_as_a_type.em` | — |
| 860 | a function value or override that changes more than its type allows | E2113 | `R12/reject_a_log_handler_in_a_list_of_plain_functions.em`, `R12/reject_a_two_variable_handler_in_a_captured_list.em`, `R14/reject_a_dyn_iterator_whose_steps_change_more_than_its_type_says.em`, `R5/reject_an_override_changes_more_than_its_virtual.em` | — |
| 861 | **a lambda parameter whose mode differs from the expected type**; a link used unopened; an object stored into a link without `weakref` | E2020 | `R13/reject_a_weakref_link_used_as_its_target_without_opening_it.em`, `R3/reject_an_object_stored_into_a_weakref_place_without_weakref.em` | **G5** (the lambda parameter) |
| 862 | a value given to a `using A` place not made in an `A` | E2020 | `R3/reject_a_value_from_the_built_in_manager_given_to_an_arena_place.em`, `R3/reject_a_value_from_another_kind_of_arena_given_to_an_arena_place.em` | — |
| 863 | `@ffi(no_callbacks)` on a callback-taking function | E5069 | `R15/reject_no_callbacks_on_a_function_that_takes_a_callback.em` | — |
| 864 | a function value that may outlive a place it uses | E3026 | `R12/reject_a_handler_that_outlives_the_callers_total.em` | — |
| 865 | a hand-over out of a name that does not own the value | E3013 | `R1/reject_a_plain_parameter_pushed_into_a_list.em`, `R12/reject_a_function_that_gives_s_away_passed_to_call_twice.em`, `R12/reject_a_plain_once_fn_parameter.em` | (G13: out of `this` in a `drop`) |
| 866 | a call through a stored function value from a name that may not change its holder | E2105, E3023 | `R12/reject_click_walks_its_handlers_without_changes_handlers.em`, `R12/reject_a_stateful_callback_given_to_a_read_only_parameter.em` | — |
| 867 | `virtual` in a class that cannot be subclassed | E2112 | `R5/reject_virtual_in_a_class_that_cannot_be_subclassed.em` | — |
| 868 | a generic body calling a method no bound declares | E2072 | `R14/reject_a_generic_body_calling_a_method_no_interface_declares.em` | — |
| 869 | a wait for a lock this thread holds | E7018 | `R15/reject_a_join_inside_the_lock_its_thread_needs.em`, `R15/reject_a_call_through_a_function_value_that_may_take_the_held_lock.em` | — |
| 870 | `thread_shared` on a value not made so | E7017 | `R3/reject_thread_shared_on_a_value_not_made_thread_shared.em` | — |
| 871 | a `thread_shared` value changed directly | E7014 | `R9/reject_a_thread_shared_object_changed_directly.em` | — |
| 872 | no capture word | E7004 | `R15/reject_an_unscoped_thread_using_a_name_with_no_capture_word.em` | — |
| 873 | a one-thread owner or lent value reaching another thread | E7015 | `R15/reject_a_callback_c_may_run_on_any_thread_holding_a_shares_owner.em` | (G21, G26: inside a list of views, inside a stored function value) |
| 874 | two tasks changing one place | E7007 | `R15/reject_two_tasks_change_a_field_without_a_lock.em` | — |
| 875 | a thread-bound value sent away | E7008 | `R15/reject_a_lock_guard_sent_to_a_task.em` | — |
| 876 | parallel chunks writing one place | E7010 | `R15/reject_a_parallel_loop_over_shares_mut_objects.em` | — |
| 877 | a main-thread module variable reached from another thread | E7012 | `R15/reject_a_thread_changes_an_unguarded_static_mut.em` | — |
| 878 | a guarded field without its lock | E7013 | `R15/reject_a_guarded_field_changed_outside_its_lock.em` | — |
| 879 | `@parallel` over what it cannot split | E7016 | `R15/reject_a_parallel_while_loop.em` | — |
| 880 | an unsafe operation outside `unsafe:` | E3100 | `R16/reject_an_unsafe_operation_in_an_unsafe_fn_body_outside_a_block.em` | — |
| 881 | a run-time alias check inside `@no_alias_checks` | E4030 | `R16/reject_an_index_compare_inside_no_alias_checks.em` | — |
| 882 | the removed words | E1010, E0104 | `Box[T]`, `.clone()`, `Weak[T]`, `upgrade`, `RefCell`, `cast if`, `cast when`, a `match` on a downcast, `as?`/`as!`, `Sync`, `@sync`, `@static_safe`, `@no_runtime_checks`, `self`, `if Some(x) = e:` (E0115) | **G8**: `Box[T, A]`, `Array[T, A]`, `Shared[T]`, `SyncShared[T]`, `ForeignBox[T]`, `str`, `cstr`, `Mut[T]`, `@borrows`, `@fp(contract)`, the mode `owned`, `owned fn`, `.copied()`, `mut self`, `owned self` |
| 884-892 | warnings | L3015, W3012, W3013, L3001, W2190 | the four `accept_..._is_warned` files (`R16/` three, `R11/` one) | **G3** (W2190) |

## 3. Every run-time check of section 2 against the `run_` tests

| line | what runs | `run_` test | gap |
|---|---|---|---|
| 770 | bounds checks | `R8/run_a_while_loop_index_is_checked.em`, `R16/run_fail_a_bounds_check_still_runs_inside_no_alias_checks.em` | — |
| 771 | one compare of two indexes | `R4/run_the_same_body_given_twice_stops_the_program.em`, `R8/run_two_ref_mut_names_of_one_list_are_compared_once.em`, `R8/run_a_ref_mut_name_and_a_read_only_name_of_one_list_are_compared_once.em` | — |
| 772 | two compares, `assert_disjoint` | `R4/run_assert_disjoint_stops_on_one_track.em` (the passing case: `R4/accept_two_tracks_proved_apart_by_assert_disjoint.em`, run-pass) | — |
| 773 | a second pass, then `retain` | no `run_` file; `R8/accept_changes_go_after_the_loop.em` is run-pass with the same stdout fused or not | covered under `accept_` |
| 774 | one queued record per `Commands` change | no `run_` file; `R8/accept_ecs_changes_are_queued_while_a_query_walks.em`, run-pass | covered under `accept_` |
| 775 | a count +1 / -1 (atomic for `thread_shared`) | `R3/run_the_last_of_two_owners_ends_the_value.em`, `R9/run_a_shared_object_ends_with_its_last_owner.em`, `R9/run_a_thread_shared_object_read_by_a_thread_and_its_maker.em` | (G16: the count's overflow) |
| 776 | nothing, `sharedref mut` | `R9/run_two_sharedref_mut_holders_and_the_owner_change_one_player.em` | — |
| 777 | nothing / a deferred-drop region / a count pair | the four `R10/run_*.em` | **G15** (a `weakref` read inside a region finds the set-aside object ended, line 432-433) |
| 778 | one add and one subtract of the weak count | `R11/run_a_second_link_finds_its_target_gone_after_the_first_link_and_the_target_end.em` | — |
| 779 | one header test per `weakref` read | `R13/run_an_opened_weakref_tests_whether_its_target_still_lives.em`, `R11/run_an_empty_link_opens_to_its_else_branch.em` | (G14: a read during the target's own `drop`) |
| 780 | a load and a test for no link (proved parent link) | `R11/run_a_proved_parent_walk_prints_the_path.em` | — |
| 781 | a header test and a weak-count move per step | `R11/run_a_walk_whose_visit_may_end_nodes_stops_at_the_first_ended_parent.em`, `R11/run_a_walk_over_nodes_with_a_drop_method_stops_at_the_first_ended_parent.em` | — |
| 782 | a flag test per call, a store per change of a `reads` field (`@cached`) | **none** | **G1** |
| 783 | a captured real variable at its address / in the function value's state / in a box | `R12/run_a_real_variable_is_seen_by_the_handler_and_back.em` (address), `R12/run_a_named_copy_is_the_handlers_own_state.em` (a named capture's state) | **G10** (a real variable in the state, and the box: unsettled) |
| 784 | two words copied out of the list | `R12/run_a_handler_that_grows_the_list_during_dispatch_keeps_its_state.em` | — |
| 785 | one count on a handler's own state | `R12/run_a_handler_that_removes_itself_then_uses_its_state.em` | (G27: the same for a kept C callback) |
| 786 | one type-id compare | `R14/run_a_kind_test_picks_each_shape_by_its_kind.em`, `R14/run_fail_a_cast_to_the_wrong_kind_stops_the_program.em` | — |
| 787 | a lock | `R15/run_two_threads_update_a_guarded_pair_under_one_lock.em` | — |
| 788 | scoped tasks and reductions | `R15/run_scoped_tasks_add_their_parts_to_a_total.em`, `R15/run_tasks_push_their_parts_in_spawn_order.em` | — |
| 789 | the checked trampoline | `R15/run_c_runs_a_kept_callback_through_the_trampoline.em`, `R15/run_c_holds_a_tagged_number_not_an_address.em`, `R15/run_fail_c_runs_a_callback_after_it_was_unregistered.em`, `R15/run_fail_a_creator_callback_called_on_another_thread.em` | **G24** (`callback(once)` called twice), **G27** (a callback that removes itself) |
| 790 | the generator protocol | `R15/run_a_generator_walks_a_list_by_position.em` | — |
| 791 | a copy, an allocation | `R3/run_copy_of_makes_an_independent_object.em`, `R3/run_a_value_stored_into_an_owns_place_gets_its_own_block.em` | — |
| 792 | the memory manager's own work | `R3/run_a_mesh_made_using_an_arena_takes_its_block_from_that_arena.em` | — |
| 793 | one word: the instance's address | `R3/run_a_value_ends_in_the_arena_instance_it_came_from.em` | — |
| 794 | nothing (compile-time facts) | nothing to run | — |

## 4. The gaps, and the tests to add

### A. Codes and refusals with no `reject_` test

**G1. `@cached` (E2117, five messages; section 2 line 782).** Rule: section 3 line 850, section 4 line
931, AN-21 (`V3/area-B.md` lines 688-816). Why it matters: a kept result read after what it depends on
changed is a stale value, and a kept `ref T` result read after the clear is a view of freed memory; the
five refusals are what keeps a cache from going stale, and the flag test is a run-time check with no
test at all. Folder `V3/conformance/R5/` (a first-line word). Tests, each a complete program:

`run_a_cached_result_is_cleared_by_a_change_to_what_it_reads.em` (run-pass, stdout `2.0`, `2.0`,
`5.0`; AN-21's NEW program):

```ember
struct Mesh:
    verts: Array[f64]

    @cached
    fn radius() reads verts -> f64:
        r = 0.0
        for v in this.verts:
            r = max(r, abs(v))
        return r

fn main():
    m = Mesh(verts=[1.0, -2.0])
    println(m.radius())      # computes 2.0 and keeps it
    println(m.radius())      # the kept 2.0
    m.verts.push(-5.0)       # clears the kept result
    println(m.radius())      # computes 5.0
```

The five `reject_` tests are this program with one line changed (and the change marked):
- `reject_cached_without_reads.em`: `fn radius() -> f64:`, E2117 "say what `radius` depends on, so a
  change can clear it: write `fn radius() reads verts -> f64:` under `@cached`".
- `reject_cached_with_another_parameter.em`: `fn radius(scale: f64) reads verts -> f64:` (and
  `m.radius(1.0)` in `main`), E2117 "`@cached` keeps one result, and `radius` takes `scale`; remove
  `@cached`, or keep the results in a field: `radii: Map[f64, f64]`".
- `reject_cached_with_changes.em`: `fn radius() reads verts changes verts -> f64:`, E2117 "a cached
  method only reads; remove `changes verts`".
- `reject_cached_reading_a_shares_field.em`: a class `Shape: verts: Array[f64] = []` and
  `struct Model: shape: shares Shape` with `@cached fn radius() reads shape -> f64:` walking
  `this.shape.verts`, E2117 "`radius` reads `shape`, which its other owners may change without `Model`
  seeing it, so a kept result could be stale; remove `@cached`".
- `reject_cached_reading_a_field_other_threads_change.em`: `class Mesh:` with `lock: Lock = Lock()`,
  `verts: Array[f64] guarded by lock = []` and the body inside `with this.lock:`, E2117 "`radius` reads
  `verts`, which other threads change under `lock`; a kept result could be stale; remove `@cached`".

AN-21 (line 758-760) also says a change "ends every view of the kept result" (a `-> ref T` cached
method, E3060 at a later use); AN-21 shows no body for such a method, so that test waits for one.

**G2. E5013, a variadic further argument C would promote.** Rule: section 4 line 951; AN-84
(`V3/area-E.md` lines 2438-2446). Why it matters: C's default promotions change how many bytes C reads
for a further argument; reading the wrong width is undefined behaviour at the C boundary. Folder
`V3/conformance/R16/` beside the C-side tests, with `variadic.c`, `variadic.h` and
`overlays/variadic.em` copied from `tests/conformance/FFI-8/`.
`reject_a_narrow_further_argument_c_would_promote.em` (compile-fail E5013):

```ember
import c "./variadic.c" with (overlay = "overlays/variadic.em") as v

fn main():
    narrow: i16 = 3
    unsafe:  # SAFETY(ffi): sum_wide reads each further argument as an i64
        println(v.sum_wide(1, narrow))    # E5013
```

Expected: E5013 on the call line. The nearest accepted program, `accept_a_further_argument_passed_as_c_reads_it.em`,
writes `v.sum_wide(1, narrow as i64)` (stdout `3`). AN-84's message says "C reads the further argument
`narrow` as an `int`" and helps `narrow as c_int`; `sum_wide` reads an `int64_t` (`variadic.c` line 9),
so that help would be wrong here (section 5, item 11).

**G3. W2190, an unused `check_disjoint` result.** Rule: line 890-892, section 4 line 966. Why it
matters: a check whose answer is dropped proves nothing (the `R4` test
`reject_an_ignored_check_disjoint_proves_nothing.em` binds it to `_checked`, which is not the unused
case). Folder `V3/conformance/R4/`. `accept_an_unused_check_disjoint_result_is_warned.em`
(compile-pass with the warning):

```ember
from std.mem import check_disjoint

fn main():
    a = [1.0, 2.0]
    b = [3.0]
    check_disjoint(a.as_span(), b.as_span())    # W2190
    println(a.len())
```

Expected: warning W2190 "to stop when they overlap, write `assert_disjoint(a.as_span(), b.as_span())`;
to handle both cases, write `match check_disjoint(a.as_span(), b.as_span()):`" (line 891's text in
this program's names).

**G4. E1030 in section 4's meaning.** Rule: line 914 ("two items with the same name in one scope, such
as a second `main`"). Why it matters: the one `E1030` test uses the code for a module-level assignment
(AN-76), a meaning section 4 does not give; the meaning it does give has no test. Folder
`V3/conformance/R15/` (beside that test). `reject_a_second_main_in_one_module.em`:

```ember
fn main():
    println(1)

fn main():    # E1030
    println(2)
```

Expected: E1030 "`main` is already declared in this module" (`V3/area-E.md` line 1190) on the second
`fn main():`. Which meaning E1030 has is section 5, item 10.

**G5. E2020, a lambda parameter whose mode differs from the expected function type.** Rule: section 3
line 861, section 4 line 915; AN-56 part 6 (`V3/area-D.md` lines 684-687). Why it matters: a lambda
that takes over (`owns s`) what its caller only lends would hand over the caller's value. Folder
`V3/conformance/R12/`. `reject_a_lambda_that_takes_over_what_each_name_lends.em`:

```ember
[C]
fn each_name(owns f: fn(String) changes C) changes C:
    f(String("ann"))

fn main():
    names: Array[String] = []
    each_name(fn(owns s) changes names: names.push(s))    # E2020
    println(names.len())
```

Expected: E2020 "`each_name` gives each name read-only, so this function cannot take it over; to keep
your own, write `fn(s) changes names: names.push(copy of s)`" (section 3 word for word). The nearest
allowed: `accept_a_lambda_keeps_a_copy_of_what_each_name_lends.em`, the same with
`each_name(fn(s) changes names: names.push(copy of s))`, run-pass stdout `1`.

**G6. E3027 before a loop name over something the function only reads, and before a query item.**
Rule: section 3 line 831, section 4 line 943; AN-15 part 6 (`V3/area-A.md` lines 3304-3306). Why it
matters: `for ref mut p in players:` over a list the function only reads would change the caller's
players behind a read-only promise (R4 line 198-199, R8 (g)). Folder `V3/conformance/R4/`.
`reject_a_ref_mut_loop_name_over_a_list_the_function_only_reads.em`:

```ember
class Player:
    hp: int = 10

fn heal_all(players: Array[Player]):
    for ref mut p in players:    # E3027
        p.hp += 1

fn main():
    team = [Player(), Player()]
    heal_all(team)
    println(team[0].hp)
```

Expected: E3027 "`players` only reads the caller's list, so `p` cannot change its players; take it as
`ref mut players: Array[Player]` on `heal_all`'s first line and call `heal_all(ref mut team)`". The
nearest allowed: `accept_a_ref_mut_loop_name_over_a_ref_mut_list.em` with those two lines, run-pass
stdout `11`. And `reject_ref_mut_on_a_query_item_the_query_only_reads.em` (AN-15):

```ember
from std.ecs import World, Query, Component

@derive(Copy, Component)
struct Position:
    x: float

@derive(Copy, Component)
struct Velocity:
    dx: float

fn slow(ref mut q: Query[(ref mut Position, Velocity)], dt: float):
    for ref mut pos, ref mut vel in q:    # E3027
        vel.dx *= 0.5
        pos.x += vel.dx * dt

fn main():
    w = World()
    e = w.create()
    w.add(e, Position(x=0.0))
    w.add(e, Velocity(dx=2.0))
    w.run(slow, 1.0)
```

Expected: E3027 "the query reads `Velocity` only; to change it, ask for it: `Query[(ref mut Position,
ref mut Velocity)]`" (section 3 word for word).

**G7. E1010, `this` in a free function.** Rule: section 3 line 826 ("a free function, a `static fn`"),
section 4 line 913. Only the `static fn` case has a test. Folder `V3/conformance/R5/`.
`reject_this_in_a_free_function.em`:

```ember
struct Counter:
    n: int = 0

fn bump() -> int:
    return this.n + 1    # E1010

fn main():
    println(bump())
```

Expected: E1010 "`bump` is not a method, so it has no object and no `this`; take the counter as a
parameter: `fn bump(c: Counter) -> int:` and `return c.n + 1`" (written to R18; section 3 gives only
the `static fn` message).

**G8. The removed words that have no test.** Rule: section 3 line 882, section 4 lines 906 and 913,
the vocabulary rows named. Why it matters: each old word, if it still compiled, would bring back an old
meaning the model removed (a run-time-checked `Shared`, a silent copy by `owned fn`, `mut self`'s
access on every field). Each is one `reject_` file of a few lines, expected E1010 (or E0104 for an
attribute) with the vocabulary's line to write. Folder `V3/conformance/R3/` unless named.
- `reject_shared_is_now_shares.em`: `items: Shared[Array[int]] = [1, 2]` in `main`; E1010 "write
  `shares T`, or `shares mut T` for an owner that changes it" (vocabulary line 240), with the line
  `items = shares mut [1, 2]`.
- `reject_sync_shared_is_now_thread_shared.em`: `fn start(log: SyncShared[Array[String]]):`; E1010
  "write `thread_shared T`" (line 299): `fn start(log: thread_shared Array[String]):`.
- `reject_foreign_box_is_now_owns.em`: `struct Handle:` with `n: int`, then `fn keep(h: ForeignBox[Handle]):`;
  E1010 "write `owns T`" (line 300): `fn keep(h: owns Handle):`.
- `reject_box_with_a_manager_is_owns_using.em`: the program of
  `R3/run_a_mesh_made_using_an_arena_takes_its_block_from_that_arena.em` with its field written
  `mesh: Box[Mesh, LevelArena]`; E1010 with the `Box` row's text (line 238): `mesh: owns Mesh using LevelArena`.
- `reject_an_array_with_a_manager_is_array_using.em`: the same program with
  `names: Array[String, LevelArena]`; E1010 "write `names: Array[String] using LevelArena`" (line 882).
- `reject_str_is_now_strview.em`: `fn first(s: str) -> str:` returning `s.trim()`; E1010 "`str` is
  now `StrView`, a view into someone else's text; for your own text write `String`" (line 246).
- `reject_cstr_is_now_cstrview.em`: `from std.ffi import c_int` and an `unsafe extern "C":` block
  declaring `safe fn puts(s: cstr) -> c_int`;
  E1010 "write `CStrView`" (line 292).
- `reject_mut_in_a_query_is_now_ref_mut.em` (folder `R4/`): G6's program with
  `Query[(Mut[Position], Velocity)]`; E1010 "`Mut[Position]` is now `ref mut Position`: write
  `Query[(ref mut Position, Velocity)]`" (AN-15 part 6).
- `reject_borrows_is_now_from.em` (folder `R6/`): `@borrows(src)` above
  `fn first_word(src: StrView, sep: StrView) -> StrView:`; E0104 "write `from src` after the result
  type: `fn first_word(src: StrView, sep: StrView) -> StrView from src:`" (line 182; E0104's list in
  section 4 does not name `@borrows`: section 5, item 10).
- `reject_fp_contract_is_now_allow_fma.em` (folder `R16/`): `@fp(contract)` above
  `fn dot(a: f64, b: f64, c: f64) -> f64: return a * b + c`; E0104 "`@fp(contract)` is now
  `@fp(allow_fma)`" (line 181).
- `reject_owned_is_now_owns.em`: `fn take(owned x: String):`; E1010 "`owned` is now `owns`: write
  `owns x: String`" (line 115).
- `reject_owned_fn_names_its_captures.em` (folder `R12/`): `name = String("ann")`,
  `f = owned fn() => println(name)`; E1010 "name what the function copies or takes:
  `f = fn() with copy of name => println(name)`" (line 116).
- `reject_copied_is_now_copy_each.em`: `m = Map[String, int]()`, `names = sorted(m.keys().copied())`;
  E1010 "`.copied()` is now `.copy_each()`: write `names = sorted(m.keys().copy_each())`" (line 281).
- `reject_mut_self_is_changes_this.em` (folder `R5/`): `fn reset(mut self):` in a `struct Counter`;
  E1010 "`self` is not a word of Ember ...: write `fn reset() changes this:`" (vocabulary line 111;
  one message for the whole old form, R18).
- `reject_owned_self_is_owns_this.em` (folder `R5/`): `fn finish(owned self) -> int:`; E1010 "write
  `fn finish() owns this -> int:`" (line 524).

### B. Section 2 rows with no `run_` test

G1 covers `@cached` (line 782). G15, G24 and G27 add run tests to lines 777, 789 and 785.

**G10. A real variable in the function value's state, or in a box (line 783; R12 lines 628-634).**
Why it matters: section 2 gives three placements and their costs; only the first has a program. W5 found
(its R12-22) that the other two arise only when the function value outlives its maker, which line 651
refuses (E3026). Folder `V3/conformance/R12/`. `run_a_counter_made_by_a_function_keeps_its_variable.em`
(AN-53 part 5's case 2, "a counter factory whose variable lives in a box"):

```ember
fn make_counter() -> fn() changes captured -> int:
    n = 0
    return fn() changes n -> int:
        n += 1
        return n

fn main():
    next = make_counter()
    next()
    println(next())
```

Expected: **unsettled**: compile-fail E3026 by line 651 ("a function value may not outlive a place it
uses and does not own"), or run-pass stdout `2` with `n` in the function value's state by lines 631-633
(it is the only user from the moment it is made). Section 5, item 8.

### C. The task's priorities

**Ownership and lifetime**

**G11. A part handed out of its owner (R2 line 120, "which leaves its owner partly handed over").**
W1 left it untested (its R2-14). Why it matters: the owner must not end the handed-over part a second
time, and must not be used whole afterwards. Folder `V3/conformance/R2/`.
`run_a_partly_handed_over_struct_ends_each_part_once.em` (run-pass; settled):

```ember
class Res:
    name: String

    fn drop() changes this:
        println("end", this.name)

struct Pair:
    left: Res
    right: Res

fn main():
    kept: Array[Res] = []
    p = Pair(left=Res(name=String("a")), right=Res(name=String("b")))
    kept.push(p.left)       # hands `p.left` over: `p` is partly handed over
    println("pushed")
```

Expected: stdout `pushed`, `end b`, `end a` (each `Res` ended once; the order is today's, the later
local first); AddressSanitizer: no double free. And `reject_a_partly_handed_over_struct_used_whole.em`:
the same `Pair` with `left: String`, `right: String`, then `kept.push(p.left)`, `println(p.right)`
(allowed) and `q = p` (refused): **unsettled code** (E3040 by section 4 line 945 is the nearest; the
rules give no message). And `reject_a_part_handed_out_of_a_class_object.em`: `class Player: name:
String`, `p = Player(name=String("ann"))`, `w = weakref p`, `names: Array[String] = []`,
`names.push(p.name)`: a class object has other names (`w` here) that would see the hole:
**unsettled** (today's `EXP-6/reject_moving_out_of_a_class_field.em` refuses it). Section 5, item 9.

**Object destruction**

**G12. A `drop` method that changes a module variable, ended inside a walk of it.** Rule: R5 line
282-283 ("the calls it makes included"), R8 (c) line 357-361; the rules never name `drop` among the
calls. Why it matters: `enemies.clear()` runs each enemy's `drop`; a `drop` that pushes to `LOG` while
a loop walks `LOG` moves the list under the loop: the use after free of R8 (c), reached through an
implicit call. Folder `V3/conformance/R8/`. `reject_ending_an_object_whose_drop_pushes_to_the_walked_list.em`:

```ember
static mut LOG: Array[String] = []

class Enemy:
    name: String

    fn drop() changes this, LOG:
        LOG.push(String("gone"))

fn main() changes LOG:
    enemies: Array[Enemy] = [Enemy(name=String("a"))]
    LOG.push(String("start"))
    for line in LOG:
        enemies.clear()      # ends `enemies[0]`, whose `drop` pushes to `LOG`
        println(line)
```

Expected: **unsettled** (if an end counts as a call of `drop`: E3020 "this loop walks `LOG`, and
`enemies.clear()` ends an `Enemy`, whose `drop` pushes to `LOG`; clear after the loop"). The nearest
allowed, `accept_ending_an_object_whose_drop_pushes_to_log_after_the_walk.em`: `enemies.clear()` after
the loop, run-pass stdout `start`. And `reject_a_function_that_ends_an_enemy_without_naming_log.em`:
`fn reset(ref mut es: Array[Enemy]): es.clear()` with no `changes LOG`: **unsettled** (E2105 if a
`drop` counts). Section 5, item 2.

**G13. A `drop` method that keeps its own object.** Rule: section 4 line 937 (E3013, "a hand-over out
of a name or place that does not own the value"); R5 line 258-259 (`changes this` does not own `this`;
`owns this` would). Why it matters: an object handed somewhere during its own end lives on in a list
after its memory is freed (the resurrection bug of finalizers). Folder `V3/conformance/R1/`.
`reject_a_drop_method_that_keeps_its_object.em`:

```ember
static mut KEPT: Array[Enemy] = []

class Enemy:
    hp: int = 1

    fn drop() changes this, KEPT:
        KEPT.push(this)        # E3013

fn main() changes KEPT:
    e = Enemy()
    println(e.hp)
```

Expected: E3013 "`KEPT.push(this)` would hand over the enemy `drop` is ending, but `drop` only changes
it; keep its number instead: declare `static mut HPS: Array[int] = []` and write `HPS.push(this.hp)`"
(settled code; whether `main` must name `KEPT` is G12's question).

**G14. A `weakref` link opened while its target's `drop` runs.** Rule: R11 lines 444-453 (a read tests
"does the object still live?"), R10 point 3 line 435 (a count pair). The rules do not say whether an
object is ended before or after its `drop` runs. Why it matters: if the opening succeeds, the view's
count pair takes the count from 0 to 1 and back to 0, which ends the object a second time (a double
free); Rust's `Weak::upgrade` fails once the strong count is 0, before `drop`. Folder
`V3/conformance/R11/`. `run_a_link_opened_during_its_targets_drop_finds_it_ended.em`:

```ember
class Player:
    hp: int = 100
    me: weakref Player = None

    fn drop() changes this:
        if this.me exists as p:
            println("open", p.hp)
        else:
            println("ended")

fn main():
    p = Player()
    p.me = weakref p
    println("made")
```

Expected: **unsettled** (stdout `made`, `ended` if an object is ended before its `drop`, as Rust; under
AddressSanitizer, no report either way). Section 5, item 3.

**G15. A `weakref` read inside a deferred-drop region (R10 point 2, lines 432-433: "a `weakref` read of
any of them finds it ended").** W4 listed it as missing (R10-2). Why it matters: the set-aside object's
memory is kept for the view, but no new name may open it as alive. Folder `V3/conformance/R10/`.
`run_a_link_read_inside_a_region_finds_the_set_aside_player_ended.em` (settled):

```ember
class Player:
    hp: int = 100

class Camera:
    target: weakref Player = None

fn drop_last(ref mut ps: Array[Player]):
    if ps.len() > 0:
        ps.pop()

fn main():
    players: Array[Player] = []
    cam = Camera()
    first = Player()
    cam.target = weakref first
    players.push(first)
    players.push(Player(hp=5))
    if cam.target exists as p:
        for i in 0..2:
            drop_last(ref mut players)  # the second call ends the player `p` views
        if cam.target exists as q:      # inside the region: set aside, so ended
            println("alive")
        else:
            println("gone")
        println(p.hp)                   # 100: its memory is kept until the region ends
```

Expected: run-pass, stdout `gone`, `100`; AddressSanitizer: no report.

**Reference counting**

**G16. A count that overflows.** Rule: R9 line 414-415 (a 32-bit count), `mem.forget` (vocabulary line
304, kept). Why it matters: 2^32 forgotten owners wrap the count to 0, and the next end frees a value
its real owners still use. Rust's `Rc` stops the program on overflow; section 2 names no check. Folder
`V3/conformance/R9/`. `run_fail_more_owners_than_the_count_holds_stop_the_program.em`:

```ember
import std.mem

class Big:
    n: int = 0

fn main():
    b = shares Big()
    for i in 0..4294967296:
        mem.forget(shares b)
    println(b.n)
```

Expected: **unsettled** (run-fail with a stop, as Rust's `Rc`; the rules give neither the check nor its
text). The same for the weak count with `mem.forget(weakref b)`. Section 5, item 4.

**G17. `copy of` a value that holds a `shares` field or a `weakref` field.** Rule: R3 line 141-142
("for a `shares` or `sharedref mut` link, a copy of its target"), section 0 line 17-18 ("The copy word
copies the thing itself, never a link"), E2116 line 930 (a `weakref` link). W1 (R3-1) and W4 (R11-5)
both found it unstated. Why it matters: one answer adds an owner (+1), the other makes a new target;
a copy that copies the pointer without either is a double free. Folder `V3/conformance/R3/`.
`run_a_copy_of_a_struct_holding_a_shared_mesh.em`:

```ember
class Mesh:
    n: int = 1

struct Entity:
    mesh: shares mut Mesh

fn main():
    a = Entity(mesh=shares mut Mesh())
    b = copy of a
    b.mesh.n = 5
    println(a.mesh.n)
```

Expected: **unsettled** (stdout `1` if the copy copies the mesh, `5` if it adds an owner, as today's
`OWN-7/accept_a_copy_holding_handles_retains_them.em`); AddressSanitizer: no report either way. And
`reject_or_accept_a_copy_of_a_struct_holding_a_link.em`: `class Player: hp: int = 10`,
`struct Watch: target: weakref Player = None`, `p = Player()`, `a = Watch(target=weakref p)`,
`b = copy of a`: **unsettled** (E2116, or a second link with one more weak count). Section 5, item 5.

**Mutable aliasing**

**G18. A method's object and a `ref mut` argument of the same call that are one object.** Rule: R4
line 232-239 ("the object a method is called on counts as one"; E3022 where both may change). W2 listed
it (R4-9). Why it matters: `merge` pushes to the list it walks when both are `a`. Folder
`V3/conformance/R4/`. `reject_a_method_given_its_own_object_as_ref_mut.em`:

```ember
class Bag:
    items: Array[int] = []

    fn merge(ref mut other: Bag) changes items:
        for x in other.items:
            this.items.push(x)
        other.items.clear()

fn main():
    a = Bag(items=[1, 2])
    a.merge(ref mut a)    # E3022
    println(a.items.len())
```

Expected: E3022 "`merge` changes `a` (its first line says `changes items`) and `other`, and both are
`a`; give it another bag: `a.merge(ref mut b)`". The nearest allowed,
`accept_a_method_given_another_object_as_ref_mut.em`: `b = Bag(items=[3])`, `a.merge(ref mut b)`,
`println(a.items.len(), b.items.len())`, run-pass stdout `3 0`.

**G19. A method that changes the module variable which owns its own object.** Rule: R4 line 232-241
(only a call's arguments are compared), R5 line 275-279 (`this` is never alone), R8 (b) line 351-353
(two paths count only through a link or a kept view). Why it matters: nothing in the rules refuses it,
and `this` is freed under the method (a use after free). W3's item 9 is the same shape with a plain
parameter. Folder `V3/conformance/R8/`. `reject_a_method_that_clears_the_list_holding_its_object.em`:

```ember
static mut ENEMIES: Array[Enemy] = []

class Enemy:
    hp: int = 10

    fn hit() changes hp, ENEMIES:
        ENEMIES.clear()       # ends every enemy, this one included
        this.hp -= 1          # this one is gone

fn main() changes ENEMIES:
    ENEMIES.push(Enemy())
    ENEMIES[0].hit()
```

Expected: compile-fail, **unsettled code and line** (at the call, E3022 "`hit` changes `ENEMIES`, which
owns `ENEMIES[0]`, the enemy it is called on", in the way R12 line 560-563 refuses a function stored in
what it changes; or in the body, E3060 at `this.hp`). The same with a `ref mut` parameter,
`fn hurt(ref mut e: Enemy) changes ENEMIES:` called `hurt(ref mut ENEMIES[0])`, as
`reject_a_ref_mut_parameter_whose_list_the_callee_clears.em`. Section 5, item 1.

**Views kept in lists and structs**

**G20. A counted owner of a value that holds a view.** Rule: R7 line 325-326 ("A value that holds views
... lives no longer than what they point into, whoever owns it"). W1 listed it (R3-3). Why it matters:
a `shares` owner's life is counted at run time, so the compile-time bound must still hold. Folder
`V3/conformance/R7/`. `reject_a_shared_holder_of_a_view_returned_past_its_text.em`:

```ember
struct Holder:
    word: StrView

fn keep(line: StrView) -> shares Holder:
    text = String(line)
    h = shares Holder(word=text.trim())
    return h    # E3063

fn main():
    h = keep(" a ")
    println(h.word)
```

Expected: E3063 "`h` holds a view of `text`, which ends when `keep` returns, and an owner made with
`shares` does not keep `text` alive; give the holder its own text: `word: String` in `Holder` and
`Holder(word=String(text.trim()))`" (settled code). The nearest allowed,
`accept_a_shared_holder_of_a_view_of_an_argument_may_be_returned.em`: `return shares
Holder(word=line.trim())`, run-pass stdout `a`.

**G21. A list of views given to a thread.** Rule: R15 line 717 ("a lent value reaching another thread
is E7015"), R7 line 325-326. W3 listed it (R7-3). Folder `V3/conformance/R15/`.
`reject_a_thread_takes_a_list_of_views.em`:

```ember
import std.thread

fn main():
    text = String(" a ")
    words: Array[StrView] = [text.trim()]
    t = thread.spawn(fn() with owns words: println(words.len()))    # refused
    t.join()
```

Expected: compile-fail, **code to confirm** (E7015 "a lent value reaching another thread", or E3063 "a
value holding views kept where it would outlive what it points into"; section 5, item 12); help: give the
thread text of its own, `words: Array[String] = [String(text.trim())]`. The nearest allowed,
`accept_a_scoped_task_reads_a_list_of_views.em`: `with scope = thread.scope():` and
`scope.spawn(fn(): println(words.len()))`, run-pass stdout `1`.

**Exclusive changes of class fields**

**G22. A field-precise method beside a `ref mut` name of a field of the same object.** Rule: R5 line
256-258 (`changes a, b`: exactly those fields), R8 (f) line 376-380, R17 line 742-743 (a `pub` method
with no `reads` reads every field across a package's edge). No test pairs a method's field list with a
`ref mut` name of a field (today's `[EXC-19]` did this while running). Folder `V3/conformance/R8/`.
`accept_a_method_that_changes_hp_while_a_ref_mut_name_of_name_is_used.em`:

```ember
class Player:
    name: String
    hp: int = 10

    fn heal() changes hp:
        this.hp += 1

fn main():
    p = Player(name=String("ann"))
    n = ref mut p.name
    p.heal()                 # changes `p.hp` only
    n = String("bo")         # writes `p.name` through `n`
    println(p.name, p.hp)
```

Expected: run-pass, stdout `bo 11` (by R5's field list, line 256-258, and line 741: `heal`'s body reads
and changes `hp` only, and inside one package that fact is used; the same reading as W3's
`R8/accept_a_method_that_changes_another_field_inside_the_loop.em`). And `reject_a_method_that_changes_hp_while_a_ref_mut_name_of_hp_is_used.em`:
`h = ref mut p.hp`, `p.heal()`, `h = 5`: E3022 "`p.heal()` changes `p.hp` (its first line says
`changes hp`), and `h` is the only name for `p.hp` until its last use; call `p.heal()` after it, or
change it through `h`" (settled).

**Contracts at the C boundary**

**G23. A `callback(borrowed)` C function given a function that changes the walked list.** Rule: R12
line 575-576 ("a `callback(borrowed)` parameter runs it during the call, as `for_each` does"), R8 (c)
line 358-361. Why it matters: the C code runs the function inside the loop; the list moves under the
walk. Folder `V3/conformance/R15/`, using `kept_events.c`. `reject_a_borrowed_callback_that_changes_the_walked_list.em`:

```ember
import c "./kept_events.c" as raw

unsafe extern "C":
    @ffi(param(cb, callback(borrowed, user_data=ud)), link_name="ke_each")
    safe fn each(cb: extern "C" fn(*mut void), ud: *mut void, times: i32)

fn main():
    names: Array[String] = [String("a")]
    fn grow() changes names:
        names.push(String("b"))
    for n in names:
        each(grow, 1)    # E3020
    println(names.len())
```

Expected: E3020 "this loop walks `names`, and `each(grow, 1)` runs `grow` during the call
(`callback(borrowed)`), which pushes to `names`; call `each(grow, 1)` after the loop" (settled). The
nearest allowed, `accept_a_borrowed_callback_that_changes_the_list_after_the_walk.em`, run-pass
stdout `2`.

**G24. A `callback(once)` that C calls twice.** Rule: R12 line 574-575 (`callback(once)`), R15 line
726-727 (the trampoline), section 2 line 789. Not one test uses `callback(once)`. Why it matters: a
once callback's state is used up by its call; a second call from C must not run freed state. Folder
`V3/conformance/R15/`, using `kept_events.c` (`ke_poll` runs the kept callback each time).
`run_fail_c_calls_a_once_callback_twice.em`:

```ember
from std.ffi import ForeignCallback, Retained

import c "./kept_events.c" as raw

class Game:
    score: i32 = 0

    fn add_point() changes score:
        this.score += 1

unsafe extern "C":
    @ffi(link_name="ke_unregister")
    fn unregister(cb: extern "C" fn(*mut void), ud: *mut void)
    @ffi(param(cb, callback(once, user_data=ud), threads(creator)), destroys(unregister), link_name="ke_register")
    safe fn register_cb(cb: extern "C" fn(*mut void), ud: *mut void)
    @ffi(link_name="ke_poll")
    safe fn poll_events()

fn setup(owns game: shares mut Game) changes Game -> Retained[ForeignCallback]:
    return register_cb(fn() with owns game changes Game: game.add_point())

fn main() changes Game:
    game = shares mut Game()
    token = setup(shares mut game)
    poll_events()          # the one call the contract allows
    println(game.score)
    poll_events()          # C calls it again
```

Expected: run-fail, stdout `1`, then the trampoline's stop (its text is not in the rules, as for W6's
`run_fail_c_runs_a_callback_after_it_was_unregistered.em`); AddressSanitizer: no read of the used-up
state.

### D. Where two rules meet

**G25. A `weakref` opened inside a loop over the list that holds its target.** Rules: R8 (b) line
351-353 (two paths of one type, one through a link), R8 (f) line 376-380, R8 (g) line 386-390, R13 line
654-667. No test opens a link inside a walk of its target's list (the `R10`/`R11` walks loop over a
counter or up a tree). Why it matters: the opened name and the loop name may be one object; one of them
changing it while the other reads it is the aliasing R8 forbids. Folder `V3/conformance/R8/`.

`reject_a_link_opened_in_a_walk_changes_what_the_loop_name_reads.em`:

```ember
class Player:
    hp: int = 10

fn main() changes Player:
    players: Array[Player] = []
    first = Player(hp=10)
    w = weakref first
    players.push(first)
    players.push(Player(hp=20))
    for p in players:
        if w exists as ref mut q:
            q.hp -= 1          # E3021
        println(p.hp)
```

Expected: E3021 "`q.hp -= 1` changes a player that `p` may read (`q` is opened from `w`, a link to a
`Player`, and `p` is a `Player` of `players`, used again on the next line); test first, `if q is not
p:`, or change it after the loop" (settled). The nearest allowed,
`accept_a_link_opened_in_a_walk_proved_different_from_the_loop_name.em`: the same with
`if q is not p:` above `q.hp -= 1`, and after the loop `if w exists as r: println(r.hp)`; run-pass
stdout `10`, `20`, `9`.

`reject_a_link_opened_in_a_ref_mut_walk_reads_the_loop_names_player.em`: the same `Player`, list and
link, under `fn main():` (here nothing is changed through the link, so `changes Player` would be
E2105), then

```ember
    for ref mut p in players:
        p.hp += 1
        if w exists as q:
            println(q.hp)      # E3021
        p.hp += 1
```

Expected: E3021 "`p` is the only name for its player while it is used, and `q`, opened from `w`, may be
that player; test `if q is not p:` first" (section 3 line 834's third message, in this program's
names). The nearest allowed, `run_a_link_opened_in_a_walk_reads_what_the_loop_name_reads.em`: `for p in
players:` with `if w exists as q: println(q.hp, p.hp)`, run-pass stdout `10 10`, `10 20` (both names
only read; nothing in the loop can end a player, so no count, R10 point 1).

**G26. A function value stored in a shared object that a thread changes or runs.** Rules: R15 line
706-707 (`with shares mut` only where the function stays on this thread), R15 line 716-718 (what may go
to another thread is inferred from the words), R12 line 641-644 (a call through a stored function value
is a change of its holder), R15 line 710-715 (E7018). No test stores a function value in an object
another thread reaches (only `R12/accept_main_names_results_for_the_threads_it_starts.em` has threads
and a function list, unshared). Why it matters: a function value carries its state; a `shares` owner
inside that state crosses to another thread unseen, and its plain count races. Folder
`V3/conformance/R15/`.

`reject_a_thread_takes_a_shares_mut_bus_to_store_a_handler.em` (the duty's case as written):

```ember
import std.thread

class Bus:
    handlers: Array[fn()] = []

    fn on_event(owns f: fn()) changes handlers:
        this.handlers.push(f)

fn ping():
    println("ping")

fn main() changes Bus:
    bus = shares mut Bus()
    t = thread.spawn(fn() with shares mut bus changes Bus: bus.on_event(ping))    # E7015
    t.join()
```

Expected: E7015 "`with shares mut bus` makes an owner of `bus` that stays on one thread, and this
function runs on a new one; share the bus between threads: `bus = thread_shared Bus()`, with
`lock: Lock = Lock()` and `handlers: Array[fn()] guarded by lock = []` in `Bus`" (settled code, as
`R15/reject_a_thread_captures_a_one_thread_owner.em`).

`reject_a_handler_holding_a_one_thread_owner_stored_in_a_bus_threads_share.em` (the one-thread owner
hidden in a function value's state):

```ember
import std.thread
from std.sync import Lock

class Counter:
    n: int = 0

class Bus:
    lock: Lock = Lock()
    handlers: Array[fn() changes Counter] guarded by lock = []

    fn on_event(owns f: fn() changes Counter) changes handlers:
        with this.lock:
            this.handlers.push(f)

    fn fire() changes handlers, Counter:
        with this.lock:
            for ref mut h in this.handlers:
                h()

fn main() changes Bus, Counter:
    bus = thread_shared Bus()
    count = shares mut Counter()
    bus.on_event(fn() with shares mut count changes Counter: count.n += 1)    # refused
    t = thread.spawn(fn() with thread_shared bus changes Bus, Counter: bus.fire())
    t.join()
    println(count.n)
```

Expected: compile-fail E7015, **line unsettled** (at `bus.on_event(...)`, where the value goes into an
object every thread may reach, or at `thread.spawn`): "this function holds `count` with `shares mut`,
an owner that stays on one thread, and `bus` is shared between threads, so another thread may run it;
share the counter between threads: `count = thread_shared Counter()` with `n: Atomic[int] =
Atomic(0)` in `Counter`, and `fn() with thread_shared count changes Atomic[int]: count.n.fetch_add(1)`".
Section 5, item 6. The nearest allowed, `run_a_thread_runs_a_handler_holding_a_thread_shared_owner.em`:
`from std.sync import Lock, Atomic`; `class Counter: n: Atomic[int] = Atomic(0)`; the `Bus` above with
`fn() changes Atomic[int]` for its handlers and `fn fire() changes handlers, Atomic[int]:`;
`fn main() changes Bus, Atomic[int]:` with `count = thread_shared Counter()`,
`bus.on_event(fn() with thread_shared count changes Atomic[int]: count.n.fetch_add(1))`, the thread
`fn() with thread_shared bus changes Bus, Atomic[int]: bus.fire()`, and `println(count.n.load())`;
run-pass stdout `1`, ThreadSanitizer: no report.

`reject_running_handlers_that_change_their_bus_under_its_lock.em`: the `Bus` above with
`handlers: Array[fn() changes Bus] guarded by lock = []` and `fn fire() changes handlers, Bus:`, whose
`h()` inside `with this.lock:` is refused: E7018 "`h()` may change any `Bus`, this one included (the
handlers' type says `changes Bus`), and this thread holds `this.lock`, so a handler that adds a handler
would wait for the lock for ever; take the handlers out under the lock and run them after the
`with this.lock:` block" (settled code, line 710-715; the exact lines of the way out are the writer's).

**G27. A C callback that removes itself.** Rules: R12 line 639-640 ("A body that may end its own state
(it removes itself) and uses that state afterwards holds one count on it across that call"), R15 line
726-727 (the trampoline), section 2 lines 785 and 789. The `R12` test of a handler that removes itself
is an Ember list; no test has a kept C callback end its own registration. Why it matters: ending the
`Retained` frees the trampoline's slot, and with it the callback's state, while the callback still
runs: the next line reads freed memory. Folder `V3/conformance/R15/`, using `kept_events.c`.
`run_a_kept_callback_that_unregisters_itself_then_uses_its_state.em`:

```ember
from std.ffi import ForeignCallback, Retained

import c "./kept_events.c" as raw

class Game:
    score: i32 = 0
    token: Option[Retained[ForeignCallback]] = None

    fn add_point() changes score:
        this.score += 1

unsafe extern "C":
    @ffi(link_name="ke_unregister")
    fn unregister(cb: extern "C" fn(*mut void), ud: *mut void)
    @ffi(param(cb, callback(retained, user_data=ud), threads(creator)), destroys(unregister), link_name="ke_register")
    safe fn register_cb(cb: extern "C" fn(*mut void), ud: *mut void)
    @ffi(link_name="ke_poll")
    safe fn poll_events()

fn setup(owns game: shares mut Game) changes Game:
    me = shares mut game
    leave = fn() with owns me changes Game:
        me.token = None      # ends the `Retained`: C forgets this callback, and its state, `me`, ends
        me.add_point()       # uses `me` after that line
    game.token = Some(register_cb(leave))

fn main() changes Game:
    game = shares mut Game()
    setup(shares mut game)
    poll_events()            # runs `leave`: it unregisters itself, then adds a point
    poll_events()            # C keeps no callback now: nothing runs
    println(game.score)
```

Expected: run-pass, stdout `1`; AddressSanitizer: no report. **Unsettled** whether line 639-640's
count covers a kept callback's slot, and how the trampoline frees a slot whose callback is running on
this thread (AN-78's "unregister waits" cannot wait for itself). Section 5, item 7.

## 5. For the owner

Each item is a place where the rules leave a result open; nothing here decides it.

1. **A method that changes what owns its own object** (G19). `ENEMIES[0].hit()` where `hit` says
   `changes hp, ENEMIES` and clears `ENEMIES`: R4 (lines 232-241) compares only a call's arguments, R5
   (lines 275-279) makes `this` never alone, and R8 (b)'s two-path test (lines 351-353) counts only
   paths through a link or a kept view, so no sentence refuses it, and the object is freed under
   `this`. Which rule refuses it, at which line, with which code? (W3's item 9 is the same shape with a
   plain parameter.)
2. **Is the end of an object a call of its `drop`?** (G12, G13.) R5's "the calls it makes included"
   (line 282) and R8 (c)'s "any call that may change it" (lines 358-361) never name `drop`. If a `drop`
   that pushes to `LOG` counts, ending an `Enemy` inside a walk of `LOG` is E3020 and every function
   that may end one says `changes LOG`.
3. **When is an object ended, before or after its `drop` runs?** (G14.) R11 (lines 444-453) tests
   "does the object still live?"; if a `weakref` opened during the drop finds it alive, R10's count
   pair (line 435) takes the count 0 -> 1 -> 0 and ends it twice.
4. **Count overflow.** (G16.) The count is 32 bits (R9 line 415) and `mem.forget` is kept (vocabulary
   line 304): 2^32 forgotten owners wrap it, and the next end frees a value in use. No check is named in
   section 2, and none for the weak count.
5. **`copy of` a value holding a `shares` or a `weakref` field.** (G17.) R3 line 141-142 and section 0
   line 17-18 say a copy copies "the thing itself, never a link"; today's copy adds an owner. And a
   held `weakref` field: E2116, or a second link? (W1's R3-1, W4's R11-5.)
6. **A function value with one-thread state stored in a `thread_shared` object.** (G26.) R15 lines
   716-718 infer what may go to another thread "from the words", and a function type
   (`fn() changes Counter`) has no word for it. Is it E7015 at the store, at the `spawn`, or is a
   function-typed field refused in a class made `thread_shared`?
7. **A kept C callback that ends its own registration.** (G27.) Does R12 lines 639-640's count on a
   stored function value's state cover the trampoline's slot, and must the trampoline delay freeing the
   slot of a running callback (a `threads(creator)` callback cannot wait for itself)?
8. **The state and box placements of a real variable** (section 2 line 783, R12 lines 631-634) need a
   function value that outlives its maker, which line 651 refuses (E3026). Is a returned counter
   (AN-53 part 5's case 2) allowed, and what does its result type say it changes? (W5's item 1.) (G10.)
9. **A part handed out of its owner** (R2 line 120): which code for the owner used whole afterwards,
   and may a part be handed out of a class object, which other names still reach? (W1's R2-14.) (G11.)
10. **Codes section 4 does not give.** The tests use E0100 (`global`), E2030 (`@view`), E2040 (a `pub`
    generic without its bounds) and E2050 (a `dyn`-incompatible generic method), none in section 4;
    E1030 means "two items with the same name" (line 914) but AN-76 and
    `R15/reject_a_module_level_assignment.em` use it for a module-level assignment; E0104's list (line
    906) does not name `@borrows`, which the vocabulary (line 182) refuses. (G4, G8.)
11. **E5013's message names what the compiler does not know** (R18 line 761): "C reads the further
    argument `narrow` as an `int`" (`V3/area-E.md` line 2446) describes C's promotion, not what the C
    function reads; for `sum_wide`, which reads an `int64_t`, the help `narrow as c_int` would pass 4
    bytes where C reads 8. (G2.)
12. **Codes left open in proposed tests:** a list of views given to an unscoped thread (G21: E7015 "a
    lent value" or E3063 "a value holding views"); a partly handed-over struct used whole (G11).
