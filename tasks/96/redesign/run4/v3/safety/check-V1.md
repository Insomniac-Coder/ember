# Check V1 of writer W1 (rules R1, R2, R3, R13)

Checker V1, safety validation (b). Inputs: `inputs/brief.md`, `V3/rules.md`, `V3/vocabulary.md`,
`V3/inputs/JOB.md`, `S/matrix-W1.md`, and the 133 tests in `V3/conformance/R1/`, `R2/`, `R3/`, `R13/`.
The answers W1 quotes were read where a message hangs on them (AN-28, AN-34 to AN-38, AN-40, AN-41,
AN-43 to AN-45, AN-47, AN-65). Nothing was compiled or run. Today's test files the matrix names were all
found on disk. The source files it names were all found too, and the codes it calls "new" are absent from
`compiler/ember_diag/src/codes.rs`.

Verdicts: **right** (the rules settle it as the test or row says), **wrong** (why, and the fix),
**unsettled** (the rule text that leaves it open). Rule lines are `V3/rules.md` lines.

The open points that come back below:
- **U1**: may an opened name written `exists as copy of v` be changed (`v += 1`)? R2 lines 130-133 refuse
  `=` on "a name opened with `exists as`" unless it is `ref mut`, with no exception for `copy of`.
  R13 line 660 and the `exists as` row of the vocabulary call it "a copy of your own".
- **U2**: does `@derive(Copy)` alone give `copy of`? R3 line 141 says `copy of` is made "by its type's
  `Clone`", and E2116 refuses "a type with no `Clone`" (lines 151-153). R1 lines 101-102 name `copy of`
  as the way out for a `Copy` struct. Today's spec `[OWN-8]` keeps `Clone` for non-`Copy` types, and
  `tests/conformance/ATT-1/accept_the_listed_attributes.em` writes `@derive(Copy, Clone)`. Writing
  `@derive(Copy, Clone)` in the tests makes them right under either reading.
- **U3**: R1 lines 97-98 say a `b = a` hand-over is refused "naming both ways out, `copy of` and
  `shares`", and R18 line 759 says the line a message gives must compile in that program. Where the
  receiving name already has a declared type (`kept: Array[int] = []`), `kept = shares items` does not
  compile without changing that declaration.
- **U4**: section 3 line 852 gives E2036's help "to compare write `if v == n:`". In that program `v`
  does not exist, so the line breaks R18 line 759.
- **U5**: what kind of view is the arena address a value keeps (R3 lines 174-182: "lives no longer than
  that instance (R7's rule for a value that holds views)")? A `ref`, a `ref mut` or a `sharedref mut`?
  If it is `ref mut`, R8 (f) refuses reading the arena (`current.made`) while the value lives (E3021).
  If it is `ref`, R8 (g) leaves open whether the value's end, which calls `deallocate`, a `changes this`
  method, counts as a change made through a read-only view.
- **U6**: a partial hand-over (R2 line 120, "leaves its owner partly handed over"). Nothing says what is
  then refused (the whole owner used, the handed-over part used) or under which code. Section 4 has no
  E3042.

## Tests

### R1

| test | verdict | why / fix |
|---|---|---|
| accept_a_child_set_before_the_push_and_changed_in_the_list | right | AN-36's first NEW form; stdout `6`. |
| accept_a_copy_of_a_copy_struct_is_changed_on_its_own | unsettled | U2: `Vec2` is `@derive(Copy)` with no `Clone`, and `q = copy of p` needs `Clone` (R3 line 141, E2116). Fix whatever the ruling: `@derive(Copy, Clone)`. |
| accept_a_copy_of_a_list_stays_as_it_was | right | R1, R3; stdout `3 4`. |
| accept_a_copy_of_the_template_pushed_each_round | right | AN-35's NEW `spawn_all`; stdout `3` / `ann`. |
| accept_a_copy_on_one_branch_keeps_the_list | right | stdout `3 3`. |
| accept_a_copy_struct_whose_source_is_not_used_again_may_change | right | R1 lines 100-103 apply only when the source "is still used afterwards"; stdout `2`. |
| accept_a_handed_over_list_is_used_through_its_new_name | right | stdout `4`. |
| accept_a_handed_over_object_is_ended_once_by_its_new_owner | right | `fn drop() changes this:` is the vocabulary's spelling (line 548). stdout `handed over` / `end one`. `b` is never read: today that is warning L1001, not an error. |
| accept_a_name_used_before_mem_drop | right | `mem.drop` row (vocabulary line 254). |
| accept_a_number_given_with_equals_is_copied | right | R1 line 100; stdout `1 2`. |
| accept_a_view_given_with_equals_hands_nothing_over | right | R1 lines 98-100; stdout `30` / `10`. |
| accept_an_object_read_before_it_is_handed_over | right | AN-43; stdout `1` / `1`. |
| accept_ref_mut_given_with_equals_changes_through_the_new_name | right | R2 line 114; stdout `9`. |
| accept_ref_mut_to_a_copy_struct_changes_the_original | right | R1 line 102; `p` is read after `q`'s last use (R8 (f)); stdout `2`. |
| reject_a_change_through_a_view_given_with_equals | right | `p`, a `ref mut` parameter, is a view (section 0, lines 24-26), so `cur = p` gives a read-only view (R1 lines 98-100). A change through it is E3023 (R5 lines 294-296; section 4 line 941). The help compiles: it is the accept test above. |
| reject_a_copy_struct_copied_with_equals_then_changed | unsettled | The code and message are right (R1 lines 100-103, E3023). The help's `q = copy of p` hangs on U2. Fix: `@derive(Copy, Clone)`. |
| reject_a_field_set_on_a_child_after_it_was_pushed | right | AN-36's message and help word for word, with this file's lines (15, 16). |
| reject_a_list_used_after_a_hand_over_on_one_branch | unsettled | E3040 is right. The help names only `copy of`, while R1 lines 97-98 want `shares` named too. Under R18, `kept = shares items` does not compile here, because `kept: Array[int]` is declared on line 11 (U3). |
| reject_a_list_used_after_it_was_handed_over | right | Section 3 row 1 word for word (line 809), lines 10/11. |
| reject_a_name_used_after_mem_drop | right | AN-43's message; E3040 (section 4 line 945: "or ended"). |
| reject_a_plain_parameter_pushed_into_a_list | right | E3013, section 3 line 865 word for word. |
| reject_an_object_read_after_it_was_handed_to_an_owns_parameter | wrong | The code and message are right (AN-43). The help's first way out, "`v = a.v` above line 16", still leaves line 17 (`println(a.v)`) refused. R18 line 760 wants every line it replaces named. Fix the help: "read it before the call: write `v = a.v` above line 16 and `println(v)` in place of line 17, or ..." (the same flaw is in AN-43's text). |
| reject_an_owned_object_handed_over_in_every_round_of_a_loop | right | E3040 is settled by section 4 line 945 ("a name used after it was handed over"), so this does not need to be unsettled as the matrix says. The help compiles: `Player` has `@derive(Clone)`. |

### R2

| test | verdict | why / fix |
|---|---|---|
| accept_a_call_that_makes_a_new_value_hands_it_to_the_name | right | R2 lines 111-113; stdout `2`. |
| accept_a_const_ref_mut_name_writes_its_place | right | R2 lines 128-130, verdict 42; stdout `20`. |
| accept_a_mut_span_name_is_pointed_at_another_span | right | R2 lines 127-128, verdicts 34 and 42; stdout `10 30`. |
| accept_a_name_for_a_part_is_a_view_and_its_owner_stays_whole | right | AN-35's NEW `log_and_keep`; stdout `ann` / `0`. |
| accept_a_name_for_a_view_a_call_gives_is_read_only | right | Row 5; `"  ab "` after `trim` has length 2. |
| accept_a_number_of_your_own_made_from_a_parameter | right | Section 3 line 811's way out; stdout `2`. |
| accept_a_number_part_is_copied | right | R2 lines 110-111; stdout `2 9`. |
| accept_a_number_viewed_with_ref_changed_after_its_last_use | right | R8 (g); stdout `1` / `9`. |
| accept_a_ref_mut_name_holding_a_mut_span_writes_its_elements | right | Verdict 34; stdout `0`. |
| accept_a_ref_mut_name_is_given_another_ref | right | AN-28, verdict 42; stdout `20 30`. |
| accept_a_ref_mut_view_parameter_given_another_view_points_the_callers_view | right | R2 lines 133-135, R4 lines 245-249, R6; stdout `3`. |
| accept_a_view_name_is_pointed_at_another_view | right | R2 lines 122-124; stdout `3`. |
| accept_a_view_name_pointed_elsewhere_frees_its_old_target | right | R2 line 124, "checked by R8 against its new target"; `p` and `q` are different locals; stdout `bo cy`. |
| accept_a_view_of_your_own_made_from_a_parameter | right | Section 3 line 811's way out. Its `## ` comment also says the name "may later view something else", which this program does not do. That is harmless. |
| accept_copy_of_a_part_is_a_value_of_your_own | right | Rows 6 and 9, R8 (g) line 395; stdout `7 1`. |
| accept_equals_on_a_name_replaces_and_ends_its_old_value | right | R2 lines 122-123; stdout `end one` / `middle` / `end two`. |
| accept_equals_on_a_ref_mut_loop_name_writes_the_element | right | R2 lines 132-133; stdout `0 0`. |
| accept_equals_on_a_ref_mut_parameter_writes_the_callers_place | right | R4 lines 199-200; stdout `0`. |
| accept_equals_on_a_ref_mut_pattern_name_replaces_the_payload | right | R2 lines 132-133; stdout `0`. |
| accept_ref_mut_names_a_part_that_may_change | right | AN-44's NEW; stdout `2`. |
| accept_ref_mut_pattern_name_changes_the_payload | right | AN-38's NEW; stdout `1`. |
| reject_a_change_of_a_plain_opened_name | unsettled | The refusal itself is right (R2 lines 130-133, E3023). The help gives `exists as copy of v`, then keeps `v += 1`. Under R2 lines 130-133 that line is refused too (U1). If U1 goes against it, fix the help to "for a number of your own write `count = v` and change `count`", as section 3 line 811 does for `n`. |
| reject_a_change_through_a_name_for_a_part | unsettled | E3023 and the message are right (AN-44 word for word). The help's `pos = copy of p.pos` on a `@derive(Copy)`-only `Vec2` hangs on U2. Fix: `@derive(Copy, Clone)`. |
| reject_a_change_through_a_plain_pattern_name | right | AN-38's message word for word (it says "the walk list in `this.state`"; section 3 line 810's example has `n.state`); row 8's help. |
| reject_a_number_viewed_with_ref_changed_while_viewed | right | Writing a number is not a reshape (section 0 line 41), so the change is E3021 (R8 (g) lines 386-390). The help compiles. |
| reject_a_read_only_ref_given_to_a_ref_mut_name | right | AN-28's E3023 and message word for word. Section 4's E3023 entry does not list this case, but its heading, "a change the name does not allow", covers it. |
| reject_a_view_name_pointed_elsewhere_is_checked_against_its_new_target | wrong | `q.name = String("cy")` replaces a value that owns memory, which is a reshape (section 0 lines 39-40). R8 (g) lines 389-391 say a reshape "ends it instead (E3060, at its later use)"; E3021 is only for a change that does not reshape. Fix: put the mark on line 16, `#$ error[E3060]: `q.name = String("cy")` (line 15) replaces the text `n` views (`n = q.name`, line 14), and `n` is used on line 16`, with help "use `n` before line 15, or name it again after it: `n = q.name`". To test E3021 instead, change a number field the view reads. |
| reject_a_view_of_a_part_used_after_its_owner_was_handed_over | right | AN-35's message and help word for word, with this file's lines. |
| reject_a_write_through_a_plain_name_holding_a_mut_span | right | Section 3 line 815, in this program's names. |
| reject_another_ref_given_to_a_const_ref_mut_name | right | Section 3 line 811 word for word. |
| reject_equals_on_a_loop_name | right | R2 lines 130-133, E3023. |
| reject_equals_on_a_number_parameter | right | Section 3 line 811 word for word. |
| reject_equals_on_a_pattern_name | right | R2 lines 130-133, E3023. |
| reject_equals_on_a_view_parameter | right | Section 3 line 811 word for word. |
| reject_mut_alone_on_a_pattern_name | right | E0113, section 3 line 828 word for word. |

### R3

| test | verdict | why / fix |
|---|---|---|
| accept_a_capture_list_names_what_it_takes_over | right | AN-47's NEW; stdout `done` / `0`. `job` is a `once fn` and runs once (R12 line 648). |
| accept_a_captured_copy_is_the_function_values_own | right | Row 6; stdout `1` / `5`. |
| accept_a_class_object_held_by_a_field | right | R3 line 158; stdout `10`. |
| accept_a_class_with_a_weakref_link_to_its_parent | right | AN-41's `Node`, R11 lines 449-451; stdout `7`. |
| accept_a_document_made_again_from_its_text | right | AN-25's way out; stdout `3`. |
| accept_a_for_loop_over_owns_hands_each_element_over | right | R3 lines 155-157; stdout `2`. |
| accept_a_hand_over_written_with_no_word | right | stdout `2`. |
| accept_a_level_made_in_an_arena_the_caller_keeps | right | AN-40's help, R3 lines 174-176. Nothing reads `current` while `lvl` lives. |
| accept_a_list_field_names_its_arena_and_the_making_line_its_instance | wrong | The result is right (stdout `0 1`), but the program is not minimal: `struct Mesh` and `struct Level` (lines 29-33) are never used. Fix: remove them. |
| accept_a_list_that_owns_its_interface_values | right | AN-66. A plain `Square` stored into an `owns dyn Shape` place is put in its block (vocabulary line 381). stdout `13`. |
| accept_a_manager_with_one_instance_needs_none_on_the_making_line | right | R3 lines 176-177; stdout `1`. |
| accept_a_match_over_owns_hands_the_payload_over | right | R3 lines 155-157; stdout `1`. |
| accept_a_parameter_list_with_types_is_not_such_a_list | right | R3 lines 192-193. `ref dx` is the default, so it does not differ from the type's mode (no E2020). stdout `5`. |
| accept_a_plain_value_stored_into_an_owns_field | right | Vocabulary line 381 and AN-40's `Entity(mesh=Mesh(...))`; stdout `3`. |
| accept_a_reading_owner_made_from_a_reading_owner | right | R3 lines 147-148; stdout `0`. |
| accept_a_string_of_your_own_made_from_a_view | right | stdout `hi`. |
| accept_a_weakref_link_leaves_its_target_with_its_owner | right | AN-65's NEW; stdout `100` / `100`. |
| accept_another_link_and_a_copy_of_the_target | right | R3 lines 145-146, R11 lines 453-454; stdout `100 5`. |
| accept_copy_of_a_shares_link_copies_its_target | right | R3 lines 141-142; stdout `2 2 3`. |
| accept_each_captured_name_says_its_own_word | right | Section 3 line 854's way out; stdout `ann 3`. |
| accept_each_loop_name_says_its_own_word | right | R3 lines 191-192; the pairs become `(3, 2)` and `(7, 4)`; stdout `3` / `7`. |
| accept_the_first_owner_keeps_what_it_could_do | right | Verdict 23, R3 lines 148-150; stdout `4`. |
| accept_thread_shared_on_a_value_made_thread_shared | right | R3 lines 143-145; a read, not a change (R9 line 407); stdout `0`. |
| accept_two_owners_that_may_change_one_list | right | AN-34; stdout `4`. |
| reject_a_bare_interface_name_used_as_a_type | right | E2173; section 3 line 859's message is the help, and the error line is written in this program's names (R18). |
| reject_a_captured_name_with_no_word_after_one_with_a_word | right | E0116, section 3 line 854 word for word. |
| reject_a_change_through_a_shares_owner | right | AN-34's message word for word, E3023. |
| reject_a_changing_owner_made_from_a_reading_owner | right | Section 3 line 813 word for word, line 11. |
| reject_a_lambda_parameter_with_no_word_after_one_with_a_word | right | Section 3 line 854, with `int` for this program. |
| reject_a_list_used_after_a_for_loop_over_owns | right | E3040; the help compiles. |
| reject_a_loop_name_with_no_word_after_one_with_a_word | right | E0116; section 3's loop message in this program's names. |
| reject_a_plain_loop_name_pushed_into_another_list | right | E3013 (section 4 line 937); both helps compile. |
| reject_a_struct_holding_a_view_of_its_own_type | right | E2200, section 3 line 855's second message, split. |
| reject_a_type_that_contains_itself_with_no_block_between | right | E2200, section 3 line 855's first message, split. |
| reject_a_value_from_another_kind_of_arena_given_to_an_arena_place | right | E2020, section 3 line 862's second message in this program's names. |
| reject_a_value_from_the_built_in_manager_given_to_an_arena_place | right | E2020, AN-40's message word for word. |
| reject_a_value_returned_past_the_end_of_its_arena | right | E3063, AN-40's message, with lines 37/38. |
| reject_an_object_stored_into_a_weakref_place_without_weakref | right | E2020, section 3 line 861. |
| reject_box_is_not_a_word_of_ember | wrong | Line 15, `Entity(mesh=Box(Mesh(verts=[1])))`, is a second use of `Box`, which is a second mistake (R18 line 758: one mistake, one message). The test marks only line 12, so a correct compiler fails it. Fix: write line 15 as `e = Entity(mesh=Mesh(verts=[1]))`, or replace `main`'s body with `println(0)`. |
| reject_clone_is_not_a_word_of_ember | right | AN-45's message word for word. |
| reject_copy_of_a_type_with_no_clone | right | E2116, section 3 line 849 word for word. |
| reject_copy_of_a_value_whose_views_point_into_its_own_let_field | right | E2116, AN-25. |
| reject_copy_of_a_view | right | E2116, section 3 line 849. |
| reject_copy_of_a_weakref_link | right | E2116, AN-65's message word for word. |
| reject_owns_before_a_returned_value | right | E0109 (section 4 line 907: "after `return`"). |
| reject_owns_before_a_value_its_place_already_hands_over | right | AN-47's message word for word. |
| reject_owns_before_fn | right | AN-47's message word for word. |
| reject_owns_on_a_captured_number | right | E2085, section 3 line 856. |
| reject_owns_on_a_class_type | right | E2114, section 3 line 858. |
| reject_thread_shared_on_a_value_not_made_thread_shared | right | E7017, AN-71's message (section 3 line 870, with this program's function name). |
| reject_using_after_a_type_with_no_heap_memory | wrong | The code and message are right (E2114, section 3 line 858), but the program is not minimal: `struct Mesh` and `struct Level` (lines 29-33) are never used. Fix: remove them. |
| reject_weak_is_not_a_word_of_ember | wrong | `cam = Camera()` (line 14) makes a `Camera` whose `target` has no default, which is a second fault next to the marked one (R18 line 758). The help's line also adds `= None`. Fix: replace line 14 with `println(0)`, or write line 11 so that `Weak` is its only fault. |
| run_a_mesh_made_using_an_arena_takes_its_block_from_that_arena | unsettled | The counting is right (one `allocate`, then one `deallocate` at `mem.drop(lvl)`; stdout `1 2` / `1`). But line 44 reads `current.made` while `lvl`, which keeps `current`'s address, is used on line 45. U5 decides whether that is E3021. Fix if needed: read `lvl.mesh.verts.len()` first, `mem.drop(lvl)`, then print both counts. |
| run_a_value_ends_in_the_arena_instance_it_came_from | unsettled | The same U5: line 45 reads `next.ended` while `b`, made in `next`, is used on line 46. Fix if needed: print `b.mesh.verts.len()` before reading the counts. |
| run_a_value_stored_into_an_owns_place_gets_its_own_block | right | AN-41's NEW and vocabulary line 381; stdout `3`. |
| run_copy_of_makes_an_independent_object | right | AN-45; stdout `100 5`. |
| run_the_last_of_two_owners_ends_the_value | right | R3 lines 142, 148-150 and section 2 line 775: `mem.drop(a)` subtracts one, and `mem.drop(b)` ends the object. |

### R13

| test | verdict | why / fix |
|---|---|---|
| accept_a_variant_of_a_result_tested_with_a_pattern | right | R13 line 666; stdout `1`. |
| accept_an_opened_name_that_is_new | right | stdout `100` / `1`. |
| accept_exists_as_copy_of_gives_a_value_of_your_own | unsettled | U1: line 12, `v += 1`, is `=` on a name opened with `exists as`, which R2 lines 130-133 refuse unless it is `ref mut`. R13 line 660 says only "a copy of your own". |
| accept_exists_as_opens_an_option_and_else_runs_when_it_is_empty | right | stdout `4` / `none`. |
| accept_exists_as_opens_an_option_of_a_pair | right | R13 line 661; stdout `3 2`. |
| accept_exists_as_ref_mut_changes_what_the_link_names | right | R13 line 659. `changes Player` is needed because the change goes through a link (R5 line 264). stdout `99`. |
| accept_some_makes_a_value_and_names_a_match_branch | right | R13 lines 663-664; stdout `5`. |
| accept_while_exists_as_repeats_while_it_holds_something | right | R11 line 471's walk shape; stdout `5`. |
| reject_a_change_through_a_plain_opened_name | right | R13 lines 659-660, E3023; the help compiles (it is the accept test above). |
| reject_a_weakref_link_used_as_its_target_without_opening_it | right | E2020, section 3 line 861 word for word. |
| reject_an_opened_name_already_in_use | right | E2106, section 3 line 853 word for word, line 14. |
| reject_an_opened_pair_name_with_no_word_after_one_with_a_word | right | E0116 (R3 lines 188-192 and R13 line 661). |
| reject_exists_as_on_a_result | right | E2036; section 3 line 852's `Result` help. |
| reject_exists_as_on_a_value_that_always_holds_one | unsettled | Section 3 line 852 word for word, but its "to compare write `if v == n:`" names a `v` that does not exist in this program (U4, R18 line 759). |
| reject_some_in_a_while_header | right | E0115, section 3 line 851. |
| reject_some_in_an_if_header | right | E0115, AN-65's message. |
| reject_upgrade_is_not_a_word_of_ember | right | E1010; AN-65. The help names the lines it replaces (R18 line 760). |
| run_an_opened_weakref_tests_whether_its_target_still_lives | right | R11 lines 446-452, R13 lines 654-656, section 2 line 779; stdout `gone` / `100` / `gone`. |

## Matrix rows

| row | verdict | why / fix |
|---|---|---|
| R1-1 | right | Guarantee, enforcing part (`drops.rs` raises E3040 today) and today's tests all hold. Missing edge case: a class object handed over by `p2 = player`, then used (AN-44's E3040 and its `shares mut` / `copy of` help). Every R1-1 test uses a list. |
| R1-2 | right | — |
| R1-3 | right | Also missing: the order of ends when the new owner is a field. |
| R1-4 | right | R1 line 99 does not list a `ref mut` parameter among the views, but section 0 (lines 24-26) makes it one, so the reject test holds. |
| R1-5 | right | — |
| R1-6 | unsettled | U2: the `copy of` way out and its accept test, on a `@derive(Copy)`-only struct. |
| R1-7 | wrong | The reject test's help leaves line 17 refused (see that test). Fix the help as given there. |
| R1-8 | right | The vocabulary's `mem.drop` row (line 254) settles the `int` case. |
| R1-9 | unsettled | U3: the help names only `copy of`. |
| R1-10 | right | The code is not unsettled, as the matrix says: section 4 line 945 makes the loop case E3040. |
| R1-11 | right | — |
| R2-1 | right | E3060, section 4 line 946 ("handed over"). |
| R2-2 | unsettled | U2, in the help's `copy of p.pos`. |
| R2-3 | right | — |
| R2-4 | right | — |
| R2-5 | right | — |
| R2-6 | wrong | Today's tests say only "`SPN-1`", with no file. The related ones are `SPN-1/accept_as_mut_span_writes_through.em` and `SPN-1/reject_writing_through_a_shared_span.em`. |
| R2-7 | right | — |
| R2-8 | right | — |
| R2-9 | unsettled | U1, in the opened-name reject test's help. The other eight tests are right. |
| R2-10 | right | — |
| R2-11 | wrong | The expected E3021 at the change should be E3060 at the later use (R8 (g) lines 389-391; section 0 line 39: replacing a `String` is a reshape). Fix the reject test as given above. |
| R2-12 | right | Also missing: the old value ended by `=` while a view of it is still used (`n = a.name`, `a = Res(...)`, `println(n)`: E3060, R8 (b)). |
| R2-13 | right | — |
| R2-14 | unsettled | U6. It has no test at all. The accept side can be written regardless: `kept.push(p.name)` with `p` owned, then `println(p.inventory.len())`, if the owner rules that the parts left stay usable. |
| R2-15 | wrong | "No observable difference" is not so. The per-round declaration is what lets a name declared in the loop body be handed over each round. Without it, line 15 in the test below is R1-10's E3040. Add `accept_a_name_declared_in_a_loop_body_is_new_each_round`: `v: Array[String] = []`, `for i in 0..3:` / `s = String("a")` / `v.push(s)`, then `println(v.len())`, stdout `3`. Its reject counterpart is `R1/reject_an_owned_object_handed_over_in_every_round_of_a_loop`. |
| R3-1 | right | — |
| R3-2 | right | — |
| R3-3 | right | Also missing: `shares mut` then a `for` loop over `items` that pushes through `other` (E3020; AN-34 names it; R8 (c) is W3's, so this is a meeting point). |
| R3-4 | right | — |
| R3-5 | right | — |
| R3-6 | right | — |
| R3-7 | right | — |
| R3-8 | right | — |
| R3-9 | right | — |
| R3-10 | wrong (minor) | The "enforced by" note says "today's E2200 is a generator's own frame". `compiler/ember_diag/src/codes.rs` line 250 gives E2200 as "type has infinite size (a recursive value type without `Box`)" (`[TYP-14]`). That is this rule's meaning already; `coroutines.rs` is one more user of it. Fix the note. The tests are right. |
| R3-11 | wrong (minor) | Both tests carry the unused `Mesh` and `Level` (see above). Also missing: `owns n: int` on a parameter (E2085, AN-43's second message). Only the capture case is tested, under R3-14. |
| R3-12 | right | — |
| R3-13 | wrong | Today's tests: `LT-1a/reject_borrows_names_an_owned_arena.em` is about `@borrows`, which is only loosely related. The direct one, `ARN-5a/reject_collection_escaping_its_arena.em` (today's E3061, "arena allocation cannot outlive `arena`"), is missing. The two `run_` tests are unsettled (U5). |
| R3-14 | right | — |
| R3-15 | right | — |
| R3-16 | wrong | `reject_box_is_not_a_word_of_ember` and `reject_weak_is_not_a_word_of_ember` each hold a second fault (see above). |
| R13-1 | right | — |
| R13-2 | right | — |
| R13-3 | right | — |
| R13-4 | unsettled | U1, in `accept_exists_as_copy_of_gives_a_value_of_your_own`. |
| R13-5 | unsettled | U4, in section 3's own message. |
| R13-6 | right | — |
| R13-7 | right | — |
| R13-8 | right | — |
| R13-9 | right | It is W4's (R10), as the row says. |

## Totals

Tests: 133 checked, 118 right, 6 wrong, 9 unsettled. Rows: 51 checked, 36 right, 8 wrong, 7 unsettled.

## For the owner

1. U1: may a name opened `exists as copy of v` be changed? R2 lines 130-133 against R13 line 660.
2. U2: does `@derive(Copy)` give `copy of`, or must a `Copy` struct also derive `Clone`? R1 lines
   101-102 against R3 line 141 and E2116 (lines 151-153).
3. U3: R1 lines 97-98 (E3040 names both `copy of` and `shares`) against R18 line 759 (the line given
   must compile), where the receiving name has a declared type.
4. U4: section 3 line 852's E2036 help, "to compare write `if v == n:`", uses a `v` the program does not
   have (R18 line 759).
5. U5: what kind of view is the instance address a value made `using` an arena keeps (R3 lines
   174-182)? May the program read the arena while the value lives, and is the value's end
   (`deallocate`, `changes this`) a change through that view?
6. U6: after a partial hand-over (R2 line 120), what is refused, and under which code (section 4 has no
   E3042)?
7. Section 4 (line 896) says a bold code "is not in today's compiler", but most of the codes without
   bold are not in today's `codes.rs` either: E0113 to E0117, E2085, E2106, E2107, E2108, E2114, E2115
   and E7017.
