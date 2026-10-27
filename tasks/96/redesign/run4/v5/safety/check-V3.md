# Check V3 of writer W3 (R7, R8)

Checker V3, safety validation (b). Inputs: `inputs/brief.md`, `V3/rules.md`, `V3/vocabulary.md` (the
`for`, `ref`, `ref mut`, `from`, `@view`, `Span`, `MutSpan`, `StrView`, `range`, `sharedref`, `Array`,
reshape and `get` rows), `V3/inputs/JOB.md`, `S/matrix-W3.md`, and the 122 tests in
`V3/conformance/R7/` (40) and `V3/conformance/R8/` (82). The answers the tests start from were read where
a program or a message hangs on them: AN-01, AN-02, AN-04, AN-06, AN-07, AN-10 (area A), AN-17, AN-22,
AN-24, AN-25, AN-26, AN-30 (area B), AN-48, AN-52 (area C). Nothing was compiled or run. "line N" with no
file means `V3/rules.md` line N. Today's runner is `compiler/ember_driver/tests/milestones.rs`: a
`#$ help:` line must be found in one of the diagnostic's `= help:` texts.

Mechanical checks (scripts in `target/redesign5/V3/`): every `#$ help:` text was searched for in its
file's `#$ error` message (`helpcheck.py`: three are not there); every "`X` (line N)" and "used on line
N" in a message was matched against the file (`linecheck.py`: all match); every today's test the matrix
names exists under `tests/conformance/`; every source file it names exists, and the cited lines
(`stores.rs:462`, `:522`; `borrows.rs:1549`, `:3419`, `:4683`, `:4758`, `:4792`, `:4808-4832`;
`ember_typeck/src/lib.rs:4428`, `:10817`, `:16395`; `ember_mir/src/lib.rs:134`) hold the code the
matrix says; every test file is named by some row; the counts (57 `accept_`, 62 `reject_`, 3 `run_`)
match the matrix's header. No test uses a removed spelling (`self`, `Box`, `.clone()`, `str`,
`Some(...)` in a header, `mut x:`, `owned`, `shared`, `weak`, `.copied()`).

Totals: 122 tests (110 right, 4 wrong, 8 unsettled); 31 matrix rows (26 right, 3 wrong, 2 unsettled).

Where a test's message is section 3's or an answer's word for word, as the brief asks, it is marked
right on the message even where that source message itself falls short of R18 (line 757-760); those
source messages are listed once, at the end, for the owner.

## The tests

### R7 (40)

- `accept_a_camera_owning_a_share_may_be_returned.em`: **right**. A `shares mut` owner is not a view
  (lines 30-35, R9 lines 399-401); `shares mut p` from an own local (R3 lines 147-148); `changes target`
  covers the player reached through the link (R5 line 257, AN-48 (a)'s `shake`). Prints `99`.
- `accept_a_class_keeps_views_into_its_own_let_field.em`: **right**. AN-25's NEW program verbatim; R7
  lines 326-331.
- `accept_a_class_view_field_from_static.em`: **right**. AN-26's NEW program plus a `main`; `from static`
  (lines 328-329); a `String` argument reads as a `StrView` (AN-29).
- `accept_a_list_of_views_of_an_argument_may_be_returned.em`: **right**. The line AN-24's E3063 message
  gives; R6 lines 313-314 (`from reg`), R7 lines 325-326.
- `accept_a_local_list_of_views.em`: **right**. AN-24's NEW program verbatim.
- `accept_a_push_keeps_a_kept_link_to_an_object_in_its_own_block.em`: **unsettled**. AN-52 (area C,
  point 3, (b)'s rules) lets the owning list grow while kept `sharedref mut` links are used, but section
  0 (lines 30-32: a view points into X when it points "at anything X owns ... the elements of X's
  lists") makes `w.target` point into `players`, and R8 (b) (lines 348-351) ends every kept view "at any
  change that may reshape what it points into"; a push is a reshape (lines 39-41). The writer's item 3.
- `accept_a_struct_holding_a_view_needs_no_view_mark.em`: **right**. Lines 323-325; `@view` optional
  (vocabulary `@view` row).
- `accept_a_struct_holding_a_view_of_an_argument_may_be_returned.em`: **right**. R6 lines 311-312: a
  function's result points into its view argument; R7 line 326.
- `accept_a_struct_whose_view_field_is_from_static_is_not_a_view.em`: **right**. Lines 328-329.
- `accept_from_naming_a_field_after_renaming_the_parameter.em`: **right**. AN-22's program with the
  parameter E2118's message gives; `from src` on a view field means "into what `src` views" (AN-22 part
  3). Prints `let x`.
- `accept_from_naming_a_let_array_field.em`: **right**. Lines 327-328; area B ruling 2 (a `String` or an
  `Array`).
- `accept_handing_over_an_object_while_a_copy_of_its_word_is_used.em`: **right**. `String(...)` is a
  copy of your own (R8 (g) line 395-396); `e = d` hands a class object over (R1 lines 96-97).
- `accept_reading_again_after_another_holder_reshapes.em`: **right**. The second way out of AN-24's
  E3060; R7 lines 335-338.
- `accept_removing_after_the_kept_links_last_use.em`: **right**. The first way out of AN-52's E3060.
- `accept_sharedref_mut_holders_and_the_owner_all_change_one_player.em`: **right**. AN-48 (b) verbatim
  plus a `println`; 100 - 1 - 3 + 10 = 106. (The matrix's claim that its C output is checked for "no
  count" has no `#$ assert-c-count:` line here: see row R7-9.)
- `accept_text_written_in_the_program_in_a_from_field.em`: **right**. AN-25 rule 3.
- `accept_the_owner_after_the_kept_ref_mut_holders_last_use.em`: **right**. R8 (f) lines 376-378: alone
  only while still used.
- `accept_the_registry_changes_after_the_kept_views_last_use.em`: **right**. The first way out of
  AN-24's E3021.
- `reject_a_change_of_what_kept_ref_views_read.em`: **right**. E3021, R8 (g) lines 386-390; AN-24's
  message with this file's lines (19, 18, 20); its second way out compiles (a number write is not a
  reshape, lines 39-41).
- `reject_a_list_of_views_returned_past_what_it_views.em`: **right**. E3063 (line 845); AN-24's message
  word for word. Its tail (`vs = views(reg)`) names no line of this `main`, which calls
  `println(views().len())`: see "Source messages" below.
- `reject_a_method_that_may_change_the_target_inside_a_loop_over_it.em`: **right**. E3020, R8 (c) lines
  357-359 ("by its first line"), R5 line 257 ("anything reached through them"); AN-48 point 6's message.
- `reject_a_push_moves_the_inline_players_a_kept_link_points_at.em`: **right**. E3060, R8 (b) lines
  348-351: a kept view is an address; inline structs move with the buffer (AN-52 needs `owns` elements).
- `reject_a_push_through_one_holder_while_a_loop_walks_through_another.em`: **right**. E3020, R7 lines
  336-338, R8 (c); AN-48's E3020 message (its note and help joined).
- `reject_a_read_through_the_owner_while_a_kept_ref_mut_field_is_used.em`: **right**. E3021, R8 (f) line
  376-380 (verdict 2: reads kept out too); message in section 3's E3021 form (line 834).
- `reject_a_sharedref_mut_field_of_a_class_without_from.em`: **right**. E2118, R7 lines 326-331 (a
  `sharedref mut` field is a view field, lines 24-29); the message is the writer's, in R18's form, and
  its two lines compile with `class Player`.
- `reject_a_sharedref_mut_holder_returned_past_its_target.em`: **right**. E3063; AN-48 point 6's message
  with `make_camera`.
- `reject_a_struct_holding_a_view_returned_past_its_source.em`: **right**. E3063 (lines 323-325, 948);
  the writer's message names both lines it replaces.
- `reject_a_view_field_of_a_class_without_from.em`: **right**. E2118; AN-26's message (line 848) word
  for word.
- `reject_a_view_into_a_part_used_after_another_holder_reshapes_it.em`: **right**. E3060 at the use, R8
  (g) lines 389-390 (a reshape ends a read-only view); AN-24's message.
- `reject_a_view_of_caller_text_in_a_from_static_field.em`: **right**. E3063; AN-26's message.
- `reject_changing_the_let_field_views_point_into.em`: **right**. E2104 (line 921); AN-25's message.
- `reject_from_naming_a_field_that_is_not_let.em`: **right**. E2118; AN-25's message (line 848).
- `reject_from_naming_a_fixed_array_field.em`: **right**. E2118 (line 932: "a fixed array"). The message
  departs from line 848's (`buf: String`, which E2118 would refuse again for lacking `let`) to keep R18
  line 759; the writer's item 6 asks the owner.
- `reject_from_naming_both_a_parameter_and_a_field.em`: **right**. E2118; AN-22's message (line 848)
  word for word.
- `reject_handing_over_an_object_while_a_view_read_out_of_it_is_used.em`: **right**. E3060 (line 946,
  "handed over"); AN-25 rule 4. The way out is the accepted neighbour above.
- `reject_removing_a_player_a_kept_link_points_at.em`: **right**. E3060, R8 (b); AN-52's message. Its
  second way out (`players: Array[shares mut Player]`) is not a whole line of this program, whose
  `players` has no written type: see "Source messages".
- `reject_sharedref_without_mut.em`: **right**. E0114 (lines 339-340, 909); section 3's message (line
  835) word for word.
- `reject_storing_a_view_of_other_text_in_a_from_field.em`: **right**. E3063; AN-25's message word for
  word.
- `reject_two_kept_ref_mut_fields_of_one_player.em`: **right**. E3022 (R8 (f)); section 3's message
  (line 834) with this file's lines (20, 19).
- `reject_view_on_a_type_that_holds_no_view.em`: **unsettled**. R7 line 325 says `@view` "is checked"
  and names no code; the vocabulary's `@view` row keeps today's E2030, but section 4 (lines 903-968),
  "one meaning each", has no E2030. The writer's item 5.

### R8 (82)

- `accept_a_change_after_the_generators_last_use.em`: **right**. R8 (c) lines 357-361 ("after the
  generator's last use"); 4, 4, 6.
- `accept_a_change_after_the_read_only_names_last_use.em`: **right**. R8 (g)'s first way out.
- `accept_a_change_reaches_the_enemy_the_line_names.em`: **right**. AN-10's second NEW program verbatim;
  `copy of` a `Copy` struct is R1's own way out (lines 100-103). 21 31.
- `accept_a_change_through_the_ref_mut_name.em`: **right**. R8 (g)'s second way out; `boss.hp` is copied
  into `println`'s first argument before `enemies[0].hp` is read (R4 line 198, R8 (g) lines 390-392).
- `accept_a_copy_of_your_own_is_not_restricted.em`: **right**. Line 395-396; `Enemy` of one `int` is a
  `Copy` type (lines 19-20), so `copy of` applies.
- `accept_a_copy_out_of_a_guarded_field.em`: **right**. R8 (e); a guarded field changed in `main`'s own
  local needs no clause (R5 line 286; AN-30 `main`).
- `accept_a_counting_loop_may_add.em`: **right**. Lines 362-363.
- `accept_a_function_value_that_changes_nothing_runs_inside_a_loop.em`: **right**. R12 line 512 ("a plain
  `fn()` changes nothing at all"), verdict 17.
- `accept_a_library_call_that_changes_the_list_after_the_loop.em`: **right**. R12 lines 543-548;
  verdict 21.
- `accept_a_link_proved_different_from_the_ref_mut_parameter.em`: **unsettled**. `hit` changes another
  `Turret` through the `weakref` link `a.target` with no `changes Turret` on its first line. R5 lines
  263-265 ask for "`changes T` for objects of type `T` it changes through a link"; line 286-287 says "a
  change through a `ref mut` parameter ... needs no clause", and section 0 (lines 30-32) says "into" does
  not go through links. If the first reading holds, `hit` is E2105. Fix that holds under both readings:
  `fn hit(ref mut a: Turret) changes Turret:`.
- `accept_a_list_of_views_of_a_temporary.em`: **right**. AN-07's NEW program; R8 (d) lines 366-367.
- `accept_a_method_on_a_temporary_that_also_changes_a_module_variable.em`: **right**. Line 370 ("it
  changes nothing else, through its first line").
- `accept_a_method_on_a_temporary_whose_result_is_used.em`: **right**. AN-10's first NEW program;
  verdict 13.
- `accept_a_method_that_changes_another_field_inside_the_loop.em`: **right**. AN-17's NEW program
  verbatim.
- `accept_a_payload_view_used_before_the_variant_changes.em`: **right**. R8 (g): `t` is not used after
  line 15.
- `accept_a_position_is_found_again_after_a_push.em`: **right**. AN-01's NEW program verbatim.
- `accept_a_position_kept_across_a_call_that_only_adds.em`: **right**. AN-02's NEW program; R17 lines
  746-748.
- `accept_a_position_named_again_after_a_removal.em`: **right**. Section 3's E3060 way out (line 840),
  verdict 42.
- `accept_a_read_only_position_is_found_again_after_a_push.em`: **right**. Line 392-393.
- `accept_a_shares_owner_sees_a_change_through_another_owner.em`: **right**. Lines 396-397, verdict 23.
- `accept_a_temporary_lives_until_its_views_last_use.em`: **right**. AN-08, row 5.
- `accept_a_view_survives_a_push_to_a_fresh_object.em`: **right**. Line 353 ("a fresh object").
- `accept_a_view_survives_a_push_to_an_object_known_to_differ.em`: **right**. Line 353 (`if a is not
  b:`).
- `accept_a_view_survives_a_push_to_an_object_of_another_type.em`: **right**. Line 353 ("different
  types"): the links lead to a `Bag` and a `Crate`; the `items` lists are parts each owns alone.
- `accept_a_view_whose_last_use_is_an_argument_does_not_live_across_the_call.em`: **right**. Lines
  390-392. (The callee side is the writer's item 9; see "For the owner".)
- `accept_a_while_loop_on_the_current_length_may_add_and_remove.em`: **right**. Lines 363-365; traced:
  `[1, 3, 4, 5]`.
- `accept_an_insert_after_a_loop_over_a_maps_keys.em`: **right** (accepted whatever rule governs maps).
- `accept_an_opened_name_used_before_its_option_is_replaced.em`: **right**. R8 (g).
- `accept_another_field_changes_while_a_from_log_generator_is_used.em`: **right**. R6 lines 319-321.
- `accept_another_name_after_the_ref_mut_names_last_use.em`: **right**. R8 (f).
- `accept_changes_go_after_the_loop.em`: **right**. AN-04's NEW program; traced `2 2 5`.
- `accept_copied_keys_outlive_a_change_of_the_map.em`: **right**. Verdict 33.
- `accept_copies_out_of_a_lock.em`: **right**. AN-30's NEW program verbatim (its `main` names nothing for
  `clear(m)`, own local).
- `accept_ecs_changes_are_queued_while_a_query_walks.em`: **right**. AN-06's NEW system; plain `e` before
  `ref mut h` is allowed (R3 lines 188-191).
- `accept_reading_the_list_again_after_a_push.em`: **right**. Both ways out of the kept-view E3060.
- `accept_returning_your_own_copy_of_a_temporarys_view.em`: **right**. AN-09, row 5.
- `accept_two_ref_mut_names_proved_apart.em`: **wrong**. Its `## ` comment says "nothing is compared
  while running", and the matrix (R8-16) says the C output is checked for no compare with
  `#$ assert-c-count:`, but the file has no such line, so the test checks only the stdout. Fix: add an
  `#$ assert-c-count:` line counting the compare's stop call (0 here), or drop the claim from the
  comment and the matrix.
- `accept_your_own_copy_of_a_strings_text_outlives_its_replacement.em`: **right**. AN-01 point 6's way
  out.
- `accept_your_own_copy_outlives_a_push.em`: **right**. AN-26 point 6's way out.
- `reject_a_call_through_a_function_value_given_the_list_ends_a_position.em`: **right**. E3060 with
  AN-02 point 6's message (AN-02 settles E3060 here, though R8 (f) line 379 would also give E3022 at the
  call: the writer's item 8).
- `reject_a_call_through_a_function_value_that_changes_the_walked_list.em`: **right**. E3020, R8 (c)
  line 359, R12 lines 553-555; the writer's message gives a compiling line.
- `reject_a_change_through_another_name_while_a_ref_mut_name_is_used.em`: **right**. E3022; AN-01's
  message (line 834).
- `reject_a_change_to_a_list_a_generator_walking_by_position_walks.em`: **right**. E3020, R6 line 321
  ("however `replay` walks its list"); section 3's message (line 838) word for word (its R18 shortfall:
  "Source messages").
- `reject_a_change_to_a_list_a_stored_generator_walks.em`: **right**. E3020; section 3's message (line
  838) with this file's lines.
- `reject_a_change_to_a_temporary_that_would_be_lost.em`: **right**. E2140; section 3's message (line
  841) word for word.
- `reject_a_change_to_the_list_a_loop_over_a_generator_walks.em`: **right**. E3020, R8 (c).
- `reject_a_change_while_a_read_only_name_is_used.em`: **right**. E3021; section 3's message (line 816)
  with this file's lines.
- `reject_a_change_while_a_ref_view_of_a_number_is_used.em`: **right**. E3021; `r = 5` on a `ref mut`
  name writes the place (R2 lines 124-125), so the way out compiles.
- `reject_a_helper_that_changes_an_element_while_a_ref_mut_position_is_used.em`: **right**. E3022;
  AN-02 point 6's message.
- `reject_a_helper_that_may_remove_ends_a_position.em`: **unsettled**. `kill_weak` also reads elements
  already in the list (`enemies[i].hp`, line 13), which R8 (f) (lines 376-380) refuses at the call while
  `boss` is used (E3021 for a read; AN-02 point 6's third bullet gives E3022 for `heal_all`), and R18 line
  758 allows one message; R8 (a) gives E3060 at line 22. Fix: a helper that removes without reading an
  element, `fn drop_last(ref mut enemies: Array[Enemy]):` with `enemies.remove(enemies.len() - 1)`.
- `reject_a_library_call_given_a_function_that_changes_the_walked_list.em`: **right**. E3020; line 837's
  message on `scores.iter()` with `copy of s.name` (writer's item 11).
- `reject_a_link_that_may_be_the_ref_mut_parameters_object.em`: **unsettled**. E3022 is right by R8 (f)
  line 379 ("a link of the same type included") and section 3's message is word for word, but `hit` has
  the same missing `changes Turret` as the accepted neighbour above; if R5 lines 263-265 apply, a second
  error (E2105) comes on line 11. Fix: `fn hit(ref mut a: Turret) changes Turret:`.
- `reject_a_map_insert_ends_a_view_of_its_keys.em`: **right**. E3060; section 3's message (line 840).
- `reject_a_method_that_changes_the_walked_field_inside_the_loop.em`: **right**. E3020 by the callee's
  first line (R8 (c) line 359).
- `reject_a_method_whose_first_line_changes_the_list_ends_a_view.em`: **right**. E3060, R8 (b) lines
  353-355.
- `reject_a_push_ends_a_kept_view.em`: **right**. E3060, R8 (b) line 348.
- `reject_a_push_ends_a_view_a_call_gave.em`: **right**. E3060; AN-26 point 6's message.
- `reject_a_push_through_another_link_of_the_same_type_ends_a_view.em`: **right**. E3060, lines
  351-353.
- `reject_a_push_while_a_for_loop_walks_the_list.em`: **right**. E3020; section 3's message (line 836).
- `reject_a_read_through_another_name_while_a_ref_mut_name_is_used.em`: **right**. E3021; section 3's
  message form.
- `reject_a_removal_ends_a_position.em`: **right**. E3060; AN-01's message (line 840).
- `reject_a_removal_inside_a_counting_loop.em`: **right**. E3020; section 3's message (line 839) word
  for word (its R18 shortfall: "Source messages").
- `reject_a_removal_while_a_for_loop_walks_the_list.em`: **right**. E3020; AN-04 point 6's message.
- `reject_a_reorder_ends_a_position.em`: **right**. E3060 (R8 (a) line 346, "reorder"; AN-01 point 6).
- `reject_a_view_of_a_guard_kept_past_its_block.em`: **right**. E3063, R8 (e); AN-30's message with the
  way out that compiles here (`best` holds a view, so `best = copy of g[0]` would not).
- `reject_a_view_of_a_guarded_field_returned_out_of_its_lock.em`: **wrong**. The code and the row-4
  message are right (line 375), but `#$ help: return String(log.lines[0])` is not in that message, which
  says only "return a copy"; the runner would require a help line the rules never give. Fix: drop the
  `#$ help:` line, or make it `return a copy, or do the work inside the \`with\` block`.
- `reject_a_view_returned_out_of_a_lock.em`: **wrong**. Same: row 4's message (line 843) gives no
  `return copy of g[0]`, which `#$ help:` requires. Same fix.
- `reject_a_view_returned_out_of_a_read_lock.em`: **wrong**. Same, with AN-30's `RwLock` message. Same
  fix.
- `reject_an_insert_while_a_loop_walks_a_maps_keys.em`: **unsettled**. R8 (c) line 357 and E3020's
  meaning (line 938) say "a list"; a map is not one, and for a view a call gave (`counts.keys()`, R6)
  R8 (g) lines 389-390 give E3060 at the next use instead. The comment's "R15 names lists, strings,
  maps and sets" is the vocabulary's `yield` row, not R15 (line 724 says "a list"). Today's
  `CTL-1/reject_writing_a_map_while_a_kept_key_lives.em` gives E3020 inside such a loop.
- `reject_an_insertion_ends_a_position.em`: **right**. E3060 (line 346).
- `reject_an_opened_name_used_after_its_option_is_replaced.em`: **right**. E3060 (replacing a value that
  owns memory is a reshape, lines 39-40; R8 (g) line 390).
- `reject_changing_the_variant_ends_a_view_of_its_payload.em`: **right**. E3060 (lines 40, 349-350).
- `reject_clearing_ends_a_position.em`: **right**. E3060 (line 346); its way out moves one line.
- `reject_keeping_a_view_of_a_temporary_past_its_loop_step.em`: **right**. E3063 (lines 367-368);
  AN-07's message with the list's own line added (R18 line 759).
- `reject_replacing_a_string_ends_a_view_of_its_text.em`: **right**. E3060; AN-01's message.
- `reject_replacing_the_list_ends_a_position.em`: **right**. E3060 (line 347, "replace"; R2 lines
  121-123).
- `reject_returning_a_list_of_views_of_a_temporary.em`: **right**. E3063; AN-07's message word for word.
- `reject_returning_a_view_of_a_temporary.em`: **right**. E3063; section 3's message (line 842) word for
  word.
- `reject_two_ref_mut_names_of_one_element.em`: **right**. E3022 (line 832, "two `ref mut` names, of one
  place"); the way out names `bodies[1]`, a different constant (R4 line 237).
- `run_a_ref_mut_name_and_a_read_only_name_of_one_list_are_compared_once.em`: **unsettled**. The stop is
  settled (section 2 line 771: "two names ... at least one changeable"), the panic text is not: section
  3 (line 832) gives only the call's ("`collide` was given `bodies[3]` as both `a` and `b`"). The
  writer's item 4. Also its comment cites "R8-17"; the row is R8-16.
- `run_a_while_loop_index_is_checked.em`: **right**. Section 2 line 770; traced: prints `2`, then index 1
  of a list of length 1, today's message (`OPT-2/run_fail_an_index_past_the_length_still_panics.em`).
- `run_two_ref_mut_names_of_one_list_are_compared_once.em`: **unsettled**. As above (lines 380-381, 771,
  832); comment cites "R8-17" for R8-16.

## The matrix rows

- R7-1: **right**. Enforcing parts plausible (today's view inference, `stores.rs` E3063); today's tests
  related.
- R7-2: **unsettled**. E2030 is not in section 4 (lines 903-968); R7 line 325 names no code.
- R7-3: **right**. The `TYP-15` tests are related as the old refusal v3 lifts.
- R7-4: **right**.
- R7-5: **right**. `LT-1a/reject_borrows_names_a_field_that_is_not_a_view.em` is R6's `from` rather than a
  field's, but close enough.
- R7-6: **right**.
- R7-7: **right**.
- R7-8: **right**.
- R7-9: **wrong**. It says section 2's "nothing" (line 776) is checked on the C output with
  `#$ assert-c-count:`, but `accept_sharedref_mut_holders_and_the_owner_all_change_one_player.em` has no
  such line. Fix: add the line (the count call `== 0`), or say the claim is not yet tested. It also
  carries the unsettled `accept_a_push_keeps_...` (item 3).
- R7-10: **right**.
- R7-11: **right** (a placeholder for C1, no claim to test).
- R8-1: **right**.
- R8-2: **right** (one of its tests is unsettled: `reject_a_helper_that_may_remove_ends_a_position.em`).
- R8-3: **right**.
- R8-4: **right**.
- R8-5: **right**.
- R8-6: **unsettled**. "one rule for every loop, a map's keys included": verdict 16 is about generators,
  and R8 (c) (line 357) and E3020 (line 938) name a list.
- R8-7: **right**.
- R8-8: **right**.
- R8-9: **right**.
- R8-10: **right**.
- R8-11: **right**.
- R8-12: **right**.
- R8-13: **wrong**. `GRM-36/reject_ref_of_a_value.em` is not related: it refuses `ref` of a call's value
  (E0111, its comment says so), not a change that would be lost; `DRP-6/reject_get_from_temporary_box.em`
  is a view of a temporary (row R8-12's guarantee). Fix: keep `EXP-5/reject_a_slice_as_an_assignment_target.em`,
  drop `GRM-36`, move `DRP-6` to R8-12.
- R8-14: **right**.
- R8-15: **right** (two of its tests are unsettled on `changes Turret`).
- R8-16: **wrong**. "the C output inside `if i != j:` has no compare (runner's `#$ assert-c-count:`)" is
  not in `accept_two_ref_mut_names_proved_apart.em`; same fix as that test. Its two panic texts are
  unsettled (writer's item 4).
- R8-17: **right** (placeholder for C1).
- R8-18: **right**.
- R8-19: **right**; its missing case is a real hole, below.
- R8-20: **right**.

## Edge cases no test covers yet (beyond the matrix's own lists)

- A removal inside `for i in range(xs.len()):` (the vocabulary's `range` row makes it a counting loop,
  E3020): no test of the second spelling.
- A `for` loop over a `sharedref mut` target's list while the owner itself (not another holder) pushes
  to it (`for item in cam.target.inventory:` with `p.inventory.push(...)`, R7 line 336: "whichever
  holder", and the owner).
- A view into a `sharedref mut` target's part used after the owner reshapes it (the R7 test covers only
  another holder).
- A generator kept in a struct field or a list (a stored walk) while the list it walks changes.
- The callee side of R8 (g) lines 390-392 with a removal: `fn note(s: StrView) changes LOG:` whose body
  is `LOG.clear()` then `LOG.push(String(s))`, called `note(LOG[0])`: frees the text `s` reads; no test
  says where it is refused.
- `insert` (an add that shifts) inside a counting loop; `write()` of an `RwLock` and a slice `g[0..1]`
  returned from a guard (the matrix lists these; still no tests).
- `@view` on a struct whose only view field is `from static`.

## Source messages that fall short of R18 (used word for word, as the brief asks)

- Section 3 line 838 (generator): "push after `g`'s last use" names no line to write (R18 line 757).
- Section 3 line 839 (counting loop): "remove after the loop: `enemies.retain(...)`" leaves
  `if enemies[i].hp <= 0:` with no body and does not name the lines it replaces (line 759-760).
- Row 4 / line 843 and AN-30's `RwLock` form: "return a copy" gives no line (the owner agreed this
  message exactly).
- AN-24's E3063: "call it `reg = Registry(hp=[10, 20])`, then `vs = views(reg)`" names a caller line that
  may not exist (here `main` writes `println(views().len())`).
- AN-52's E3060 second way out: "`players: Array[shares mut Player]`" is not a whole line where `players`
  has no written type, and does not say whether the list literal's elements need `shares mut`.
