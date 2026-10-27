# Check V6 of writer W6 (rules R14, R15, R16, and the C side)

Checker V6, safety validation (b). Inputs: `inputs/brief.md`, `V3/rules.md`, `V3/vocabulary.md`,
`V3/inputs/JOB.md`, `S/matrix-W6.md`, and the 149 tests in `V3/conformance/R14/` (38), `R15/` (97, with
the C library `kept_events.c` / `kept_events.h`) and `R16/` (14). The answers W6 quotes were read where a
message or a program hangs on them: AN-22 (area B), AN-66 to AN-69 (area D), AN-71 to AN-84 (area E).
Nothing was compiled or run. Checked mechanically (scratch under `target/redesign5/V6/`): every one of
the 218 test paths the matrix names exists on disk (today's under `tests/conformance/`, the new ones under
`V3/conformance/`); every one of the 149 tests is named in the matrix; every source file the matrix names
exists; the codes it calls new are absent from `compiler/ember_diag/src/codes.rs`; every "line N" inside an
expected message points at the line the message means. The texts W6 calls today's were found in today's
source: `invalid downcast` (`ember_codegen_c/src/lib.rs:8718`), the `threads(creator)` stop
(`runtime/ember_rt/src/ember_rt.c:2597`), the bounds text (`ember_rt.c:9135`), W3012's text
(`ember_driver/src/lib.rs:7365`), the two E3100 texts (`ember_typeck/src/errors.rs:257`,
`ember_typeck/src/lib.rs:14011`). A `#$ warning` in the header with no line is today's form
(`tests/conformance/ERR-5/warning_a_dropped_result.em`).

Verdicts: **right** (the rules settle it as the test or row says), **wrong** (why, and the fix),
**unsettled** (the rule text that leaves it open). Rule lines are `V3/rules.md` lines.

Summary: 149 tests: 134 right, 7 wrong, 8 unsettled. 42 matrix rows: 37 right, 5 unsettled; the
matrix's section-2 table has one wrong entry. The C library is right.

## Open points that come back below

- **U1** (R15, rules.md:837 against 710-715 and 963). A call through a `fn() changes Log` inside a walk of
  `log.lines` under `log`'s lock: section 3 gives it E3020; R15's fourth point and E7018's meaning give
  every such call inside the lock E7018; R18 (line 758) allows one message.
- **U2** (R15, rules.md:696 against 360 and 938). A scoped task that changes the list its parent walks:
  R15 says E7007; R8 (c) and E3020's meaning list "a `spawn` whose thread changes it".
- **U3** (R15, rules.md:701 against 874). `stats.total += e` from two tasks: R15 lists `total += e`
  among the reductions; section 3's E7007 row refuses `stats.total` changed by two tasks "each without a
  lock". Whether a field of a captured object is a reduction place is not said.
- **U4** (section 4, rules.md:914). E1030's one meaning is "two items with the same name in one scope";
  AN-76 gives E1030 to `FRAME = 0` at module level, a statement.
- **U5** (rules.md:680, 726-727). No text for the stop of a failed `cast`, of a trampoline call after
  `unregister`, or of a wrong-thread call; no word on whether the thread check runs in every profile
  (today it is debug-only, `tests/conformance/FFI-33b`). The tag value `0x4080` is AN-78's only.
- **U6** (rules.md:689-690). A `pub` generic whose body has two operations its missing bound would cover
  (`copy of xs[0]` needs `Clone`, `acc + x` needs `Add`): which line carries the one E2040.
- **U7** (R18, rules.md:759). Section 3's E7012, E7007 and E7010 messages give `Lock` and `Atomic`
  lines; `Lock` is a new type (none in today's `std/src/`), and every accepted program imports it
  (`from std.sync import Lock`). A program that lacks the import makes the message's lines fail to compile.

## Tests, R14

| test | verdict |
|---|---|
| `accept_a_bound_follows_from_a_declared_interface.em` | right: R14 lines 689-690; `Crate implements Weighed`, so `heaviest` gets `T: Weighed`; prints 5. |
| `accept_a_dyn_iterator_type_says_what_its_steps_change.em` | right: R12 lines 619-623 and verdict 46's own line; `inspect` runs `f` before each step: 1, 2, then `LOG.len()` 2. |
| `accept_a_generic_method_taking_its_visitor_by_ref_mut_has_one_dyn_entry.em` | right: R14 line 688-689, AN-67's NEW made whole; 2 + 3 = 5. |
| `accept_a_kind_test_declares_a_new_name.em` | right: R14 lines 672-676; prints 2. |
| `accept_a_kind_test_on_a_place_names_it_ref_mut.em` | right: R14 lines 673-675, AN-22's `put` (`changes pools`); prints `ok`. |
| `accept_a_name_for_a_value_already_of_its_kind.em` | right: the line E2036's message gives (rules.md:852); 1 + 2 = 3. |
| `accept_a_plain_dyn_iterator_is_driven_inside_a_walk_of_log.em` | right: R12 line 623 ("a `dyn` type with no `changes` changes nothing"), R8 (c); (10+1)+(20+2) = 33. |
| `accept_a_private_generic_function_works_its_bound_out_from_its_body.em` | right: R14 line 689, AN-69's `sum`; prints 6. |
| `accept_a_pub_generic_writes_its_bound.em` | right: AN-69's written bound; prints 6. |
| `accept_a_ref_mut_cast_of_a_place_changes_the_registry.em` | right: R14 lines 680-682, R2 line 113-119 (`ref mut` before a call that gives `ref mut T`), AN-22's `get_mut` and its `reg.get_mut[int](...)`; 10 + 1 = 11. |
| `accept_a_ref_mut_dyn_parameter_changes_the_shape.em` | right: R14 line 669-670, R4 line 219-221; prints 2. |
| `accept_a_ref_mut_kind_test_name_of_a_ref_mut_parameter.em` | right: R14 lines 669, 672-675; prints 2. |
| `accept_owns_dyn_and_shares_dyn_say_who_owns_the_shapes.em` | right: R3 lines 161-162, R14 line 669; prints 1, 2, 7, 7. |
| `accept_the_kind_test_and_the_sure_cast_replace_the_old_forms.em` | right: R14 lines 672-680, verdict 27; prints `5 0 5`. |
| `reject_a_bare_interface_name_as_a_type.em` | right: E2173 (rules.md:934), section 3's message word for word (rules.md:859), on the field's line. |
| `reject_a_change_through_a_plain_dyn_parameter.em` | right: R14 line 669, R5 lines 294-300; E3023 is section 3's code for a `changes this` method through a read-only name (rules.md:812); the message names both lines it changes. |
| `reject_a_change_through_the_read_only_name_a_kind_test_gives.em` | right: R14 line 673; E3023 with AN-22's part-6 message word for word. |
| `reject_a_dyn_iterator_whose_steps_change_more_than_its_type_says.em` | right: E2113 (rules.md:927), section 3's third E2113 message (rules.md:860) with this program's line 15. |
| `reject_a_dyn_that_changes_log_driven_inside_a_walk_of_log.em` | right: R12 lines 619-622 with R8 (c) lines 357-361 give E3020; message written to R18 in the form of section 3's `poll_events` one. |
| `reject_a_function_driving_a_dyn_that_changes_log_without_saying_so.em` | right: R12 lines 620-622, E2105 (rules.md:922); the message gives verdict 46's own first line. |
| `reject_a_generic_body_calling_a_method_no_interface_declares.em` | right: E2072 with section 3's message word for word (rules.md:868). The message's own way out also needs `Crate` to implement the interface (for the owner, item 8). |
| `reject_a_generic_method_that_returns_its_type_parameter_through_dyn.em` | right: AN-67 part 6 gives E2050 and this message word for word; rules.md:688-689 says only what gets an entry. |
| `reject_a_kind_test_between_unrelated_types.em` | **wrong**: the code is AN-67 part 6's (E2020), but the message, written by W6, gives `if s is Circle as c:` as the line to write, and line 21 (`return l.text.len()`) then uses `l`, which no longer exists, on a `Circle` that has no `text`: the line does not compile in this program (R18, rules.md:759-760). Fix: name both lines, "instead of lines 20 and 21 write `if s is Circle as c:` and `return c.r`", or give the way out that keeps the body: "if a `Label` is a shape, write `struct Label implements Shape:` with `fn size() -> int: return this.text.len()`". |
| `reject_a_kind_test_naming_a_name_already_in_use.em` | right: E2106 (rules.md:923) with section 3's second message (rules.md:853), line 14 correct. Section 3's line leaves `return s.r` on line 16, which does not compile (for the owner, item 8). |
| `reject_a_kind_test_that_always_passes.em` | right: R14 lines 670-679 (a struct value is no interface or base-class value), E2036 with section 3's message (rules.md:852). |
| `reject_a_kind_test_that_changes_pools_in_a_method_that_does_not_say_so.em` | right: R5 lines 298-300 make a change through `this` in a method with no `changes` E2105, not E3027; AN-22's part-6 message word for word. |
| `reject_a_match_on_a_maybe_downcast.em` | right: R14 lines 676-678, section 3's removed words (rules.md:882), AN-68's message word for word. |
| `reject_a_plain_name_from_a_cast_of_a_place_returns_ref_mut.em` | right: R14 lines 680-682; AN-22 part 6 gives E3023 and this message word for word. |
| `reject_a_pub_generic_without_its_written_bound.em` | **unsettled** (U6): code and message are AN-69's; whether E2040 sits on line 8 (`copy of xs[0]`, the first operation the empty bound lacks) or on line 10 is not settled by rules.md:689-690 or AN-69. |
| `reject_a_ref_mut_kind_test_name_of_a_read_only_parameter.em` | right: R14 line 669, 673-675; E3027's meaning includes "a loop or pattern name over something it only reads" (rules.md:943); message to R18, both lines named. |
| `reject_as_question_and_as_bang.em` | right: section 3 removed words (rules.md:882), AN-67's two messages word for word; two mistakes, two messages. The `as?` message leaves line 17 (`if a exists as c:`) behind (for the owner, item 8). |
| `reject_cast_if_is_not_ember.em` | right: R14 lines 686-687, AN-68's OLD line and its part-6 message word for word (one message for the line, AN-68). |
| `reject_cast_when_is_not_ember.em` | right: R14 line 686, AN-68's message, both replaced lines named. |
| `run_a_kind_test_and_a_cast_on_a_base_class.em` | right: R14 lines 670-682 on a base class; prints 7, 3, 7. |
| `run_a_kind_test_picks_each_shape_by_its_kind.em` | right: R14 lines 672-678; prints `circle 3`, `square 4`, `other`. |
| `run_fail_a_cast_to_the_wrong_kind_stops_the_program.em` | **unsettled** (U5): the stop is settled (rules.md:680), the text `invalid downcast` is today's and the rules give none. |
| `run_is_before_a_value_asks_for_the_same_object.em` | right: R14 line 683-684; prints `true false`, `true`. |
| `run_is_without_a_name_only_tests_the_kind.em` | right: R14 line 676; prints 2. |

## Tests, R15

| test | verdict |
|---|---|
| `accept_a_borrowed_callback_uses_the_real_variable.em` | right: AN-78 ("A plain capture ... only for `callback(borrowed)`"), R12 line 576, verdict 51 (a named local function); `count` is `main`'s own (R5 line 286-287); `ke_each` runs it 3 times; prints 3. |
| `accept_a_c_call_inside_a_walk_when_no_callback_is_kept.em` | right: R12 line 606. |
| `accept_a_c_call_inside_a_walk_when_the_kept_callback_changes_nothing.em` | right: R12 line 606, verdict 31's own example. Not minimal: `sqrt`, `add_point` and `score` are unused (harmless). |
| `accept_a_c_function_marked_no_callbacks_inside_a_walk.em` | right: AN-78's NEW word for word, R12 lines 592-595. |
| `accept_a_callback_c_may_run_on_any_thread_holds_a_thread_shared_owner.em` | right: the lines section 3's E7015 message gives (rules.md:873), and AN-78 part 6 keeps `fn add_point() changes score:` for an `Atomic` field. |
| `accept_a_class_with_atomic_fields_is_shared_without_a_mark.em` | right: R5 lines 300-304, R12 line 508-512 (`changes Stats` as section 3's E7015 message names `Game` for an `Atomic` field); 1 + 2 = 3. |
| `accept_a_creator_thread_callback_holds_a_weakref.em` | right: R15 lines 704-707, the second way out of AN-78's E7004 message, verdict 51. `sqrt` unused (harmless). |
| `accept_a_direct_call_that_takes_the_held_lock_enters_past_it.em` | right: R15 line 711 (static re-entry); prints 1. |
| `accept_a_float_sum_added_once_per_task.em` | right: the line AN-71's E7007 message gives; 0.75 + 1.0 = 1.75. |
| `accept_a_generator_views_what_its_caller_keeps.em` | right: the lines AN-77's E2221 message gives; prints 2, 1. |
| `accept_a_generator_yields_its_own_number.em` | right: AN-77's E2221 fix; prints 1, 5. |
| `accept_a_parallel_loop_over_a_range.em` | right: R15 line 722, AN-84's `place`; prints `[2, 0, 1]`. |
| `accept_a_parallel_loop_over_one_owner_objects.em` | right: AN-74's `hurt_all`, R1; hp 9, x = 9 & 1 = 1. |
| `accept_a_scoped_task_changes_an_unguarded_static_mut.em` | right: R15 lines 720-721; prints 1. |
| `accept_a_task_takes_a_copy_of_a_round_local.em` | right: AN-73's NEW, R15 line 703; 2 + 1 = 3. |
| `accept_a_task_takes_the_lock_itself.em` | right: R4 lines 227-229, R12 line 511; prints 2. |
| `accept_a_thread_shared_object_is_read_by_a_thread_and_its_maker.em` | right: AN-71's E7014 fix; prints `1 1`. |
| `accept_a_thread_started_in_a_walk_under_the_lock_it_needs.em` | right: verdict 52, R8 (c) line 360, R15 line 694-695; `with copy of r` is what R15 lines 704-705 require (verdict 52's own line lacks it: for the owner, item 3); RESULTS [1] then one more 1: prints 2. |
| `accept_a_thread_started_under_the_lock_is_joined_after_it.em` | right: the line E7018's message gives (rules.md:869); prints 1. |
| `accept_an_any_thread_export_changes_a_guarded_static.em` | right: AN-84's NEW lines. |
| `accept_an_own_local_end_needs_no_word.em` | right: R12 lines 517-518, verdict 37; sum of i*i for 0..9 = 285. |
| `accept_an_unscoped_thread_takes_its_object_over.em` | right: AN-71's `share_plain`; prints 1. |
| `accept_guarded_statics_changed_by_a_thread.em` | right: the lines section 3's E7012 message gives (rules.md:877); prints `1 1`. |
| `accept_init_sets_guarded_fields_without_the_lock.em` | right: R15 line 708-709 ("outside `init`"); prints 5. |
| `accept_main_names_what_its_threads_change.em` | right: R15 line 693-694, AN-76's NEW; prints `1 1 2`. |
| `accept_send_written_by_an_unsafe_extend_author.em` | right: R15 line 717, R16 line 733-734 (a `# SAFETY:` note above, so no W3012). |
| `accept_tasks_change_stats_under_its_lock.em` | right: AN-74's NEW; 2 + 1 = 3. |
| `accept_the_function_value_called_after_the_lock_is_let_go.em` | right: the line E7018's message gives; prints 0, 1. Relies on the call `flush(log, fn() ... changes Log)` not being an R4 overlap (for the owner, item 10); AN-75 calls `dump` the same way. |
| `accept_thread_shared_as_a_bound.em` | right: R15 line 716, AN-80; prints 2. |
| `accept_threads_share_a_log_and_push_under_its_lock.em` | right: AN-75's NEW, verdicts 29, 30, 44; prints 2. |
| `accept_tick_names_what_the_kept_callback_changes.em` | right: R12 lines 596-601, verdict 39. |
| `kept_events.c` / `kept_events.h` (support) | right: the C side does what the four trampoline tests need (keeps one callback, runs it here or on its own thread and waits, keeps a stale copy, reports the top 16 bits of the user data, runs a borrowed one n times); builds on Windows and POSIX. |
| `reject_a_c_call_inside_a_walk_a_kept_callback_may_change.em` | right: R12 lines 589-592, 602; section 3's message (rules.md:837) with lines 30, 32, 25 correct. |
| `reject_a_call_through_a_function_value_that_may_take_the_held_lock.em` | right: R15 lines 710-715, E7018 with section 3's message word for word. |
| `reject_a_callback_c_may_run_on_any_thread_holding_a_shares_owner.em` | right: R15 lines 704-707, E7015 with section 3's message word for word (rules.md:873). |
| `reject_a_float_sum_added_more_than_once_per_task.em` | right: AN-71 part 6's program and message, line 12. |
| `reject_a_function_value_that_changes_another_log_inside_the_lock.em` | right: R15 lines 714-715 ("a function that changes another `Log` is refused too"). |
| `reject_a_generator_walk_held_by_views_of_its_own_frame.em` | right: R15 lines 724-726, AN-77's program and message; lines 12, 14, 10, 11 correct. |
| `reject_a_guarded_field_changed_outside_its_lock.em` | right: E7013 with section 3's message word for word (rules.md:878). |
| `reject_a_guarded_field_read_outside_its_lock.em` | right: R15 line 708-709 ("every read and change"); message written to R18. |
| `reject_a_guarded_field_under_another_objects_lock.em` | right: AN-74 part 6's message word for word. |
| `reject_a_guarded_list_pushed_to_outside_its_lock.em` | right: AN-75 part 6's message word for word. `import std.thread` unused (harmless). |
| `reject_a_join_inside_the_lock_its_thread_needs.em` | right: verdict 52, E7018 with section 3's message (rules.md:869), lines 20 and 19. |
| `reject_a_kept_callback_that_does_not_say_it_changes_game.em` | right: AN-78 part 6, word for word. |
| `reject_a_kept_callback_using_a_name_with_no_capture_word.em` | right: R15 lines 704-705, AN-78's E7004 message word for word; both ways out compile (the second is `accept_a_creator_thread_callback_holds_a_weakref.em`). |
| `reject_a_lock_guard_sent_to_a_task.em` | **wrong**: E7008 and section 3's message are right (rules.md:875), but in this program the message's way out does not compile: the task would take `m`'s lock while `main` holds it around the whole `with scope` block (line 10), so the scope's end would wait for a task that needs a lock this thread holds, which is E7018 (rules.md:963; R18 line 759). Fix: open the scope first and take the guard inside it, `with scope = thread.scope():` then `with g = m.lock():` then the spawn, so that the task taking the lock itself (`fn() changes m: add_one(m)`, as in `accept_a_task_takes_the_lock_itself.em`) waits only until the `with g` block ends and the join comes after it. |
| `reject_a_module_level_assignment.em` | **unsettled** (U4): AN-76's message word for word, but E1030's one meaning (rules.md:914) is "two items with the same name in one scope". |
| `reject_a_parallel_loop_over_shares_mut_objects.em` | **wrong** (U7): code and message are section 3's (rules.md:876), but the program does not import `Lock`, so the message's `lock: Lock = Lock()` does not compile in it (R18, rules.md:759). Fix: add `from std.sync import Lock`, as AN-74's program has it. |
| `reject_a_parallel_loop_writing_at_another_index.em` | right: E7010's meaning (rules.md:956), today's message and help kept by AN-84. |
| `reject_a_parallel_while_loop.em` | right: E7016 with section 3's message (rules.md:879), line 9. |
| `reject_a_receive_through_a_given_end_not_named.em` | right: R12 lines 517-518, verdict 37, E2105; message to R18. |
| `reject_a_round_local_used_by_a_task_without_a_copy.em` | right: AN-73 part 6's E3026 message, line 13. |
| `reject_a_scope_that_ends_inside_the_lock_its_task_needs.em` | **wrong**: E7018 is settled by its meaning ("a scope's end included", rules.md:963), but the message's way out ("write the `with scope` block after the `with RESULTS_LOCK:` block") leaves `with RESULTS_LOCK:` (line 18) with no body, which does not parse (R18, rules.md:759-760). Fix: say "instead of lines 18 to 20 write `with scope = thread.scope():` and `scope.spawn(fn() changes RESULTS: worker(2))`", or give the lock block a line of its own before the scope (`println(RESULTS.len())`) so that moving the scope leaves a program that compiles. |
| `reject_a_scoped_task_that_changes_total_without_saying_so.em` | right: AN-71 part 6 word for word. |
| `reject_a_send_not_named_on_the_first_line.em` | right: R12 lines 514-516, verdict 36, E2105; message to R18. |
| `reject_a_task_changes_the_list_its_parent_walks.em` | **unsettled** (U2): E7007 by R15 line 696, E3020 by rules.md:360 and 938. The message's lines compile. |
| `reject_a_task_changing_an_atomic_without_saying_so.em` | right: AN-73 part 6 word for word. |
| `reject_a_task_reads_total_before_the_join.em` | right: R15 line 702 ("nothing reads the place before the join"), AN-71's E7007 message with lines 9 to 12 and 13. |
| `reject_a_task_update_that_uses_the_old_value.em` | right: AN-72's program, message and help word for word. |
| `reject_a_thread_captures_a_one_thread_owner.em` | right: R15 lines 706-707, AN-75's E7015 message word for word. |
| `reject_a_thread_changes_an_unguarded_static_mut.em` | **wrong** (U7): code, placement and message are section 3's (rules.md:877, lines 8 and 9 right), but the program does not import `Lock`, so `static TICK_LOCK: Lock = Lock()` does not compile in it. Fix: add `from std.sync import Lock`. |
| `reject_a_thread_takes_over_a_lent_log.em` | right: R3, E3013 (rules.md:937), AN-75's message word for word. |
| `reject_a_thread_using_the_log_with_no_capture_word.em` | right: AN-75's E7004 message word for word. |
| `reject_a_threads_function_changing_the_log_without_saying_so.em` | right: section 3's E2105 message (rules.md:819) word for word. |
| `reject_a_threads_function_that_does_not_say_what_it_changes.em` | right: AN-76 part 6 word for word. |
| `reject_a_threads_function_that_sends_without_saying_so.em` | right: verdict 36's line; message to R18. |
| `reject_a_view_of_the_frames_own_local_across_yield.em` | right: AN-77's program and message, lines 7, 8, 9. |
| `reject_a_walk_under_the_lock_across_a_function_value_that_changes_log.em` | **unsettled** (U1): section 3's E3020 message word for word, but E7018 claims the same call. |
| `reject_an_any_thread_export_changes_a_static_mut.em` | **wrong** (U7): AN-84's message and help, but the help's `Lock` and `Atomic` lines do not compile without an import the program lacks. Fix: add `from std.sync import Atomic, Lock`. |
| `reject_an_unmarked_c_function_inside_the_walk.em` | right: section 3's fourth E3020 message (rules.md:837), lines 29, 30, 24. |
| `reject_an_unscoped_thread_using_a_name_with_no_capture_word.em` | right: section 3's E7004 message word for word (rules.md:872). |
| `reject_async_is_reserved.em` | right: E0005 (rules.md:905), AN-77's message. |
| `reject_handing_c_the_callback_inside_the_walk.em` | right: R12 lines 570-578, R8 (c); AN-78's message, lines 28, 29. |
| `reject_implementing_send_without_unsafe.em` | right: R15 line 717, today's E3100 text (`ember_typeck/src/lib.rs:14011`), which gives no exact line (for the owner, item 9). |
| `reject_main_calls_setup_without_naming_game.em` | right: AN-78 part 6, line 23 named. |
| `reject_main_starts_a_sending_thread_without_naming_it.em` | right: verdicts 30 and 36; message in the form of section 3's (rules.md:821). |
| `reject_main_starts_a_thread_without_naming_what_it_changes.em` | right: section 3's E2105 message (rules.md:821), line 18. |
| `reject_no_callbacks_on_a_function_that_takes_a_callback.em` | right: E5069 with section 3's message word for word (rules.md:863). |
| `reject_python_global.em` | right: AN-76 part 6's message with this program's lines 9 and 8. |
| `reject_ref_mut_before_a_channel_end_at_a_call.em` | right: R4 lines 227-229, E3028; message in the form of section 3's `Mutex` one (rules.md:830). |
| `reject_setup_hands_c_a_callback_without_naming_its_changes.em` | right: section 3's E2105 message word for word (rules.md:821). |
| `reject_start_without_naming_what_its_threads_change.em` | right: AN-75 part 6 word for word. |
| `reject_sync_as_a_bound.em` | right: AN-80's E1010 message word for word. |
| `reject_the_sync_class_mark.em` | right: AN-80's E0104 message, line 8. |
| `reject_thread_shared_from_a_plain_parameter.em` | right: R3 lines 143-145, AN-75's E7017 message. |
| `reject_thread_shared_on_a_plain_local.em` | right: section 3's E7017 message word for word (rules.md:870). |
| `reject_tick_calls_a_c_function_without_naming_the_kept_callbacks_changes.em` | right: R12's own message (rules.md:598-601) word for word. |
| `reject_two_tasks_change_a_field_without_a_lock.em` | **wrong**, and **unsettled** underneath (U3). Wrong: with either way out the message gives, line 16 (`return stats.total`) no longer compiles (an `Atomic` is read with `.load()`; a guarded field read outside its lock is E7013), and the program imports neither `Atomic` nor `Lock` (U7); R18 (rules.md:759-760) asks the message to name every line it replaces. Fix: as AN-74's program does, read something else after the join (or name line 16 in the message: `return stats.total.load()`, or `with stats.lock: return stats.total`), and add `from std.sync import Atomic, Lock`. Unsettled: whether `stats.total += t.len()` is a reduction (rules.md:701) or E7007 (rules.md:874). |
| `reject_using_a_name_after_it_was_handed_to_a_thread.em` | right: R1, AN-71's E3040 message, line 12. |
| `run_a_generator_walks_a_list_by_position.em` | right: R15 lines 724-725, AN-77's NEW; prints a, b, 4, 5. |
| `run_a_producer_thread_sends_and_a_consumer_thread_receives.em` | right: verdicts 36, 37, 30; prints 285. |
| `run_c_holds_a_tagged_number_not_an_address.em` | **unsettled** (U5): rules.md:726-727 says only "a tagged `{slot, generation}` number"; 16512 (`0x4080`) is AN-78's. |
| `run_c_runs_a_kept_callback_through_the_trampoline.em` | right: R15 line 726-727, AN-78; two polls, prints 2. |
| `run_fail_a_creator_callback_called_on_another_thread.em` | **unsettled** (U5): the stop is AN-78's; its text is today's and the rules say neither the text nor the profile (today's test runs in `debug` only). |
| `run_fail_c_runs_a_callback_after_it_was_unregistered.em` | **unsettled** (U5): the stop is AN-78's ("stops with the callback's name"); the expected start is written by W6 and the rules give no text. stdout 1 is right. |
| `run_scoped_tasks_add_their_parts_to_a_total.em` | right: AN-71's `count_words`; prints 3. |
| `run_tasks_push_their_parts_in_spawn_order.em` | right: R15 line 702 (spawn order); prints 2, 1, then (0*2+2)*2+1 = 5. |
| `run_two_threads_update_a_guarded_pair_under_one_lock.em` | right: AN-80's NEW; prints `8 5`. |

## Tests, R16

| test | verdict |
|---|---|
| `accept_a_blend_from_a_copy_inside_no_alias_checks.em` | right: R4 lines 236-241 (a fresh copy is proved apart), R16 line 735-736. |
| `accept_a_loop_over_the_view_itself.em` | right: AN-82's NEW; prints 6. |
| `accept_a_safety_note_on_an_unsafe_fn_line_is_warned.em` | right: W3013 (rules.md:968), section 3's text (rules.md:886-888), AN-79's OLD note above the line. |
| `accept_an_unsafe_block_without_a_safety_note_is_warned.em` | right: W3012 (rules.md:967), today's text. |
| `accept_an_unsafe_fn_vouches_block_by_block.em` | right: AN-79's NEW; [1, 0, 0, 0] read little-endian is 1. |
| `accept_an_unsafe_fn_without_safety_is_warned.em` | right: L3015 on every `unsafe fn` (rules.md:731-733, 965), section 3's text word for word; that text names a promise the compiler cannot know (for the owner, item 7). |
| `accept_no_alias_checks_with_the_indexes_proved_apart.em` | right: R4 line 237, R16; the swap prints `2 1`. |
| `reject_an_index_compare_inside_no_alias_checks.em` | right: E4030 with section 3's message (rules.md:881), line 16. |
| `reject_an_unsafe_operation_in_an_unsafe_fn_body_outside_a_block.em` | right: E3100 with section 3's message (rules.md:880), line 8. |
| `reject_assert_disjoint_inside_no_alias_checks.em` | right: R16 line 735-736; message to R18, lines 12 to 15 named, its lines compile (`accept_a_blend_from_a_copy_inside_no_alias_checks.em`). |
| `reject_calling_an_unsafe_fn_outside_unsafe.em` | right: E3100, today's text kept by AN-79 (it gives no exact line: for the owner, item 9). |
| `reject_no_runtime_checks.em` | right: AN-82's E0104 message word for word. |
| `reject_static_safe_is_now_no_alias_checks.em` | right: AN-81's E0104 message word for word. |
| `run_fail_a_bounds_check_still_runs_inside_no_alias_checks.em` | right: R16 line 736-737, section 2 line 770; stdout 6, then today's bounds text for index 3 of 3. |

## Matrix rows

| row | verdict |
|---|---|
| R14-1 | right: guarantee (rules.md:669, R5), enforcer plausible, today's `TYP-22` tests touch `mut self` through `dyn`. Missing too: a `shares mut dyn I` element changed through a `ref mut` kind-test name. |
| R14-2 | right. Missing too: a `shares mut dyn I` changed through each of its two owners. |
| R14-3 | right. Missing too: a kind test on an element of `Array[shares dyn I]` inside a walk of that list. |
| R14-4 | right; the edge case (`is` inside an expression) is a real open point. |
| R14-5 | right. Missing too: a `ref mut` kind-test name held while the registry's map is changed (R8 (f), E3022). |
| R14-6 | right. |
| R14-7 | right. |
| R14-8 | right as a row (AN-67 part 6 gives E2020); its test's message is wrong (above). |
| R14-9 | **unsettled** (U5): the panic text. |
| R14-10 | right. |
| R14-11 | right; `GRM-30/reject_the_old_downcast_spellings.em` is related. |
| R14-12 | right. |
| R14-13 | right (E2050 is AN-67's; not in section 4, which lists today's codes only where the model changes them). |
| R14-14 | right; U6 is the one open point of its reject test. |
| R14-15 | right. Missing too: a value that changes less than its `dyn` type says, given to it (accepted, R12 line 540-541 by analogy). |
| R15-1 | right. |
| R15-2 | right. |
| R15-3 | right. |
| R15-4 | right as a row; `reject_two_tasks_change_a_field_without_a_lock.em` is wrong and unsettled (above, U3). |
| R15-5 | right. |
| R15-6 | **unsettled** (U2). |
| R15-7 | right. Missing too: `with weakref` and `with shares mut` captures in an unscoped thread (E7015; only `with shares` is tested); a class that is the target of a `weakref` field handed `with owns` (AN-71 part 6, E7015). |
| R15-8 | right (W4's tests carry it). |
| R15-9 | right. |
| R15-10 | **unsettled** (U1), for its walk test. |
| R15-11 | right. Missing too: E7008 for a `Scope` or a raw pointer sent to a task (the row lists it). |
| R15-12 | **unsettled** (U4), for `reject_a_module_level_assignment.em`; the rest right. |
| R15-13 | right. Missing too: E7016 for a loop over an iterator that is neither a range nor a list (`for x in xs.iter().filter(...)`); E7010 for a scalar captured and written by every chunk. |
| R15-14 | right. |
| R15-15 | right. |
| R15-16 | right. |
| R15-17 | **unsettled** (U5): the stop texts, the profile, the tag value. Missing too: a `threads(main)` callback called elsewhere. |
| R15-18 | right. |
| R15-19 | right. |
| R15-20 | right. |
| R15-21 | right. |
| R16-1 | right. |
| R16-2 | right. |
| R16-3 | right. |
| R16-4 | right. |
| R16-5 | right. |
| R16-6 | right. |
| section-3 table | right: each refusal's code and test match. |
| section-2 table | **wrong** in one entry: "an atomic count pair for `thread_shared` (rules.md:775)" lists `R15/run_a_producer_thread_sends_and_a_consumer_thread_receives.em`, which has no `thread_shared` value (its threads take `with owns tx`, `with owns rx`). Fix: list `R15/accept_threads_share_a_log_and_push_under_its_lock.em` and `R15/run_two_threads_update_a_guarded_pair_under_one_lock.em`. |

## For the owner

1. U1: E3020 or E7018 for a call through a `fn() changes Log` inside a walk of `log.lines` under its lock (rules.md:837 against 710-715 and 963; R18 line 758: one message).
2. U2: E7007 or E3020 for a scoped task that changes the list its parent walks (rules.md:696 against 360 and 938).
3. Verdict 52's line, `thread.spawn(fn() changes RESULTS: worker(r))` inside `for r in RESULTS:`, has no capture word for `r`, which R15 (rules.md:704-705) requires of an unscoped thread (E7004); W6's test writes `with copy of r`.
4. U3 and U7: is `stats.total += e` on a field of a captured object a reduction (rules.md:701) or E7007 (rules.md:874)? And section 3's E7012, E7007 and E7010 messages give `Lock`/`Atomic` lines without the import they need, so in a program without it the line does not compile (R18, rules.md:759).
5. U4: E1030 means "two items with the same name" (rules.md:914); AN-76 gives it to a module-level statement.
6. U5: the texts of the stops of a failed `cast` (rules.md:680), of a trampoline call after `unregister`, and of a wrong-thread call (rules.md:726-727); whether the thread check runs in every profile (today `debug` only); the tag `0x4080` is AN-78's alone.
7. L3015's message (rules.md:884-885) gives `@safety("p points to four readable bytes")`, a promise the compiler cannot know (R18, rules.md:761).
8. Lines given by rules-text messages that do not compile in the program they are for (R18, rules.md:759-760): E2106 (rules.md:853) leaves `s.r` in the block; AN-67's `as?` message leaves `if a exists as c:`; E2072's (rules.md:868) "declare one and the bound follows" also needs the type to implement it (`struct Crate implements Weighed`).
9. Today's E3100 texts kept by AN-79 and R15 ("calling an `unsafe fn` requires an `unsafe` block"; "implementing `Send` by hand requires `unsafe extend`") give no exact line to write (R18, rules.md:757).
10. R4's overlap rule (rules.md:232-241) does not say whether a plain `thread_shared Log` argument and a `fn() changes Log` argument of one call overlap (`flush(log, fn() with thread_shared log changes Log: ...)`); verdict 44 makes a guarded field a `Mutex`'s value, but R4 names no exception for it.
