# Check of writer W2 (R4, R5, R6, R17), checker V2

Agent V2 of safety validation (b). Checked: `S/matrix-W2.md` (35 rows) and its 121 tests in
`V3/conformance/R4/` (56), `R5/` (42), `R6/` (14), `R17/` (9, with `R17/ember.toml` and the package
`R17/ledger/`). Against `V3/rules.md` (`rules:N` is its line), `V3/vocabulary.md`, `V3/inputs/JOB.md`
(verdict N) and the answers in `V3/area-*.md` the tests start from. Nothing was compiled or run. Every
line number a message gives was counted in the file. Every today's test the matrix names exists
(checked with `ls`); every compiler file it names exists.

Totals: tests 98 right, 15 wrong, 8 unsettled; rows 29 right, 1 wrong, 5 unsettled.

## Two faults that recur

1. **Multi-line output is written in a form today's runner does not read.** Eight run-pass tests write
   the second printed line as `#$ 4` (or `#$ dropped an empty line`). Today's format, and the brief's
   "`#$ stdout:` (each printed line)", is one `#$ stdout:` header per line (for example
   `tests/conformance/BLD-FFI-5a/accept_a_sync_class_object_crosses_an_export.em`, lines 4 to 6; no
   today's test has a `#$ <text>` continuation line). Fix in each: write the second line as
   `#$ stdout: <text>`.
2. **Messages whose line does not make the program compile** (R18, rules:759: "The line a message gives
   compiles under these rules in that program, and where it replaces several lines the message names
   every one it replaces"). Five tests copy a message from the rules or an answer whose fix leaves
   another line of the test's program refused. Each is listed below with the line to add; the same gap
   in the rules' own examples is a question for the owner.

## Tests, R4 (56)

| test | verdict |
|---|---|
| accept_a_callable_type_writes_the_parameters_word | **right**: rules:217-218 (a callable type writes the parameter's words), rules:219-221; `bump` changes nothing beyond its `ref mut` parameter, so R4's overlap rule (rules:233-235) sees no place in `apply(bump, ref mut counter)`; prints `1` |
| accept_a_const_ref_mut_parameter_writes_the_callers_elements | **right**: rules:128-137, verdict 42; `0.5*1+0.5*3 = 2.0`, `0.5*2+0.5*6 = 4.0` |
| accept_a_copy_of_its_own_beside_a_changeable_list | **right**: rules:232-236; `first` is a `String` of its own (rules:117-118), prints `ann` |
| accept_a_copy_of_its_own_beside_a_changeable_span | **right**: rules:240-241; `1+(-1)*0.5 = 0.5`, `1+1*0.5 = 1.5` |
| accept_a_mutex_argument_takes_no_word | **unsettled**: the program is AN-30's NEW word for word, and AN-30 (`area-B.md` part 3) says `main` names nothing because `m` is its own local (rules:286 "a change ... to an own local needs no clause"); but rules:267 ("What the functions it calls change counts") and verdict 36 / rules:515-516 make a function that calls `produce(tx)` on its own channel end say `changes Sender[int]`. If verdict 36's reading holds, `fn main()` here is refused (E2105) and needs `changes Mutex[Array[String]]` |
| accept_a_plain_parameter_takes_no_word_at_the_call | **right**: rules:196-199, 230; prints `0` |
| accept_a_ref_mut_mutspan_local_given_as_ref_mut | **right**: rules:114-117, verdict 34; prints `0.0` |
| accept_a_ref_mut_mutspan_parameter_writes_the_callers_elements | **right**: rules:222-225, 316-319, 366 (the temporary span lives until `r`'s last use); `p[0] = 0.5`, `a[1] = 7` |
| accept_a_ref_mut_name_given_another_ref | **right**: rules:124-127, verdict 42, AN-28; prints `20 30` |
| accept_a_ref_mut_parameter_changes_the_callers_counter | **right**: rules:199-201, 219-221; prints `1` |
| accept_a_ref_mut_parameter_handed_on_as_ref_mut | **right**: rules:219-227; prints `100` |
| accept_a_ref_mut_shares_mut_parameter_sets_a_cell | **unsettled**: rules:201 says `ref mut b: shares mut Player` "may ... change the player", and rules:286 says a change through a `ref mut` parameter needs no clause; but rules:264 says `changes T` names "objects of type `T` it changes through a link", and the clock is reached through the `shares mut` link the caller's place holds. Whether `tick` (and `main`) must say `changes Clock` is open (the writer's own for-owner 1 and 2) |
| accept_a_ref_mut_view_parameter_points_the_callers_view_elsewhere | **right**: rules:133-136, 202, 245-247; `best = other` is the form rules:844 gives; prints `3` |
| accept_a_view_put_in_through_ref_mut_used_before_its_source_grows | **wrong**: the second printed line is written `#$ 4`, which no runner reads (fault 1). Fix: `#$ stdout: 4`. Otherwise right (rules:245-247, R8 (b)) |
| accept_an_owns_parameter_hands_on_a_changing_owner | **right**: rules:204-209, verdict 32, AN-78; prints `5` |
| accept_an_owns_parameter_takes_a_node_over | **right**: rules:198, 204, 230-231; prints `1 2` |
| accept_check_disjoint_proves_apart_in_its_ok_branch | **wrong**: second line written `#$ 1.0` (fault 1). Fix: `#$ stdout: 1.0`. Otherwise right: AN-14 part 3 ("in its `Ok` branch the two are known apart, as inside `if i != j:`"); `2.0`, then the `Err` branch leaves `1.0` |
| accept_drop_the_lines_first_and_push_the_notes_after | **wrong**: second line written `#$ dropped an empty line` (fault 1). Fix: `#$ stdout: dropped an empty line`. Otherwise right: the lines rules:832 gives; `LOG` ends `["a", "dropped an empty line"]` |
| accept_every_pair_proved_apart_by_its_loop | **right**: rules:236-237 (`j` from `i + 1..`); prints `-1.0` |
| accept_one_game_passed_once_as_ref_mut | **right**: AN-18's NEW; `note` changes only `log` (rules:256-258), the argument is a new `String`; prints `orc moved` |
| accept_ref_mut_on_the_parameters_name | **right**: AN-27; prints `2` |
| accept_two_bodies_inside_if_i_is_not_j | **right**: rules:237; the call is skipped, prints `1.0` |
| accept_two_tracks_proved_apart_by_assert_disjoint | **right**: rules:237, 772, AN-14's `mix`; prints `2.0` |
| reject_a_call_without_ref_mut_for_a_ref_mut_parameter | **right**: E3028 (rules:226, 944), rules:830's first message in this program's names; AN-27 word for word |
| reject_a_const_ref_mut_name_given_another_ref | **right**: E3023 (rules:129-130, 941), rules:811's third message word for word |
| reject_a_const_ref_mut_parameter_given_another_span | **right**: E3023 (rules:136-137, 941); message follows rules:811's third; its line compiles (the span made in the call ends with the line, rules:224-225) |
| reject_a_function_value_that_changes_the_calls_object | **right**: E3022, rules:557-559 and rules:832's message word for word; the lines it gives compile (`accept_drop_the_lines_first...` is that program) |
| reject_a_mutspan_argument_without_ref_mut | **right**: E3028, rules:830's second message word for word |
| reject_a_name_used_after_it_was_handed_to_an_owns_parameter | **wrong** (message, fault 2): E3040 is right (rules:945) and the message is AN-43's, but its fix "read it before the call (`v = a.v` above line 15)" leaves line 16 `println(a.v, x)` reading `a` after the hand-over, still E3040. Fix: "... (`v = a.v` above line 15, and `println(v, x)` as line 16)" |
| reject_a_plain_mutspan_local_given_as_ref_mut | **right**: E3027 (rules:226-227, 943; a plain name holding a `MutSpan` only reads, rules:115-117); line 13 counted right; its line compiles |
| reject_a_plain_mutspan_parameter_writes | **wrong**: the program has a second refusal the test does not expect: `main` calls `integrate(ref mut p.as_mut_span(), ...)` while `pos` is plain, which rules:226 refuses (E3028, "one written where the parameter is not `ref mut`"). Fix: write the call `integrate(p.as_mut_span(), v.as_span(), 0.5)`, and end the message as rules:815's first does: "... write `fn integrate(ref mut pos: MutSpan[float], vel: Span[float], dt: float):` and call it `integrate(ref mut p.as_mut_span(), v.as_span(), 0.5)`" |
| reject_a_plain_parameter_changes_the_callers_counter | **right**: E3023 (rules:294-296), rules:810's second message's form |
| reject_a_plain_shares_mut_parameter_sets_a_cell | **unsettled**: E3023 is settled (rules:294-296 name "a `Cell` field, an object behind a `shares` link"), but the line the message gives, `fn tick(ref mut s: shares mut Clock):`, compiles only if no `changes Clock` is needed (see `accept_a_ref_mut_shares_mut_parameter_sets_a_cell`) |
| reject_a_read_and_a_change_of_one_game_in_one_call | **wrong** (message, fault 2): E3021 is right (rules:239). The message is AN-18's, but in this program `tick`'s body is `other.score = g.players.len()`, so `fn tick(ref mut g: Game):` leaves `other` undefined. Fix: add "and write its body `g.score = g.players.len()`" |
| reject_a_view_of_the_callees_own_local_put_in_through_ref_mut | **right**: E3063 (rules:247-249), rules:844 word for word |
| reject_a_view_put_in_through_ref_mut_used_after_its_source_grows | **right**: E3060 (rules:245-246, 348-351); lines 15, 16, 17 counted right |
| reject_a_write_through_a_span_parameter | **right**: E3023 (rules:294-296, 941); both lines named |
| reject_an_argument_reaching_into_what_a_changeable_one_owns | **right**: E3021 (rules:232-239, 833); both lines named |
| reject_an_ignored_check_disjoint_proves_nothing | **right**: E3021; rules:236-237 list `assert_disjoint`, not a bound `check_disjoint` result, as a proof; lines 15 to 18 counted right |
| reject_an_owner_that_changes_made_from_a_plain_parameter | **right**: E3023 (rules:207-209, 814), every replaced line named |
| reject_mut_alone_on_a_parameter | **wrong** (message, fault 2): E0113 right (rules:828, 908), message AN-27's; but with `fn bump(ref mut n: int):` line 13 `bump(a)` is refused (E3028, rules:226). Fix: add "and call it `bump(ref mut a)`", as rules:810 does |
| reject_owns_on_an_int_parameter | **right**: E2085 (rules:204, 856) |
| reject_owns_written_before_an_argument | **right**: E0109 (rules:154-155, 857, 907) |
| reject_ref_b_given_to_a_ref_mut_name | **right**: E3023, rules:126-127 ("says so with `ref mut` again, for every type"), AN-28's message word for word |
| reject_ref_in_a_parameters_type | **wrong** (message): E0117 right (rules:829), but the message (AN-27's) says "a plain parameter already reads the caller's value without copying it" for an `int`, and R4 (rules:198) says "a number is copied in": R18 (rules:761) "A message says only what the compiler knows". Fix: use the case rules:829 gives, `fn show(c: ref Counter):` with a `Counter` class, message "the mode goes on the name: write `c: Counter`"; or for the `int` drop "without copying it" |
| reject_ref_mut_at_a_call_to_a_plain_parameter | **right**: E3028, rules:830's third message |
| reject_ref_mut_before_a_mutex_argument | **right**: E3028, rules:830's fourth message, AN-30 |
| reject_ref_mut_before_a_new_number | **right**: E3027 (rules:226-227), AN-27's message; AN-27 reads `ref mut a + 1` as `ref mut (a + 1)` (the rules give no precedence; parentheses would remove the doubt) |
| reject_ref_mut_given_from_a_plain_parameter | **right**: E3027, rules:831's first message, both lines named |
| reject_ref_mut_in_a_parameters_type | **right**: E0117, rules:829 / AN-27 word for word |
| reject_the_same_body_given_twice | **right**: E3022, rules:832's first message word for word |
| reject_two_spans_of_one_list_one_changeable | **right**: E3021 (rules:240-241), both lines named |
| reject_two_tracks_of_one_map_not_proved_apart | **right**: E3021, rules:833's second message with lines 14 to 16 counted right |
| run_assert_disjoint_stops_on_one_track | **unsettled**: that it stops is settled (rules:772, vocabulary `assert_disjoint`); the panic's words are not given anywhere in the rules (today's `assert_disjoint_or_panic` panics "assert_disjoint_or_panic: the views overlap", `tests/conformance/DSJ-5/run_fail_or_panic_on_overlap.em`) |
| run_the_same_body_given_twice_stops_the_program | **right**: rules:237-239, 771; panic is rules:832's run-time message with `bodies[1]` |
| run_two_bodies_compared_once_at_the_call | **right**: rules:237-239; prints `-1.0 1.0` |

## Tests, R5 (42)

| test | verdict |
|---|---|
| accept_a_changing_method_called_through_a_ref_mut_loop_name | **right**: rules:229-231, 812; prints `100 100` |
| accept_a_loop_calls_a_virtual_that_changes_one_field | **right**: rules:288-293, R8 (c): `changes angle` does not reach `d.log`; prints `45.0` |
| accept_a_method_names_the_field_it_changes | **right**: rules:257-258, AN-19; prints `90.0` |
| accept_a_static_fn_is_called_on_its_type | **right**: rules:252-254, verdict 7; prints `5` |
| accept_a_struct_method_replaces_its_whole_value | **right**: vocabulary `this` row ("A struct method with `changes this` may replace the whole value"); prints `0.0 0.0` |
| accept_a_virtual_says_more_than_its_body | **right**: rules:290-293, AN-70; prints `45.0` |
| accept_an_atomic_changed_through_a_plain_name | **unsettled**: as `accept_a_mutex_argument_takes_no_word`: AN-30's reading (own local, rules:286) against rules:267 and verdict 36 / rules:515-516 |
| accept_an_implementation_within_its_interface_methods_line | **right**: rules:288-290 (`changes calls` is within `changes this`); prints `4.0 1` |
| accept_changes_this_beside_a_module_variable | **right**: rules:260-261 (`fn fire() changes this, LOG:`); prints `0 1` |
| accept_init_calls_a_reads_method_once_its_fields_are_set | **right**: rules:279-280, AN-31; prints `atlas@800px` |
| accept_init_sets_its_fields_with_no_word | **wrong**: second line written `#$ 90.0` (fault 1). Fix: `#$ stdout: 90.0`. Otherwise right (rules:279-281) |
| accept_main_names_the_module_variable_its_callee_changes | **right**: rules:263, 282-285; prints `1` |
| accept_owns_this_takes_the_object_over | **right**: rules:259; prints `7` |
| accept_tick_names_what_note_changes | **right**: rules:283-284 (row 3), R8 (f), R17 rules:741 (inside the package `note`'s reads come from its body); prints `[0] orc died` |
| reject_a_caller_does_not_name_what_its_callee_changes | **right**: E2105, rules:818's first message word for word |
| reject_a_changing_method_called_through_a_plain_loop_name | **right**: E3023, rules:812 word for word |
| reject_a_first_line_names_a_field_the_body_never_changes | **right**: E2105 both ways (rules:282); message in rules:610's form |
| reject_a_helper_with_no_reads_called_before_every_field_is_set | **right**: E2100 (rules:280-281, 919; a method with no `reads` may read any field, vocabulary `reads`) |
| reject_a_loop_calls_a_virtual_that_changes_this | **right**: E3020 (rules:288-289, 357-361); line 20 counted right |
| reject_a_method_changes_a_field_its_line_does_not_name | **right**: E2105, rules:818's second message's form |
| reject_a_method_changes_a_field_with_no_clause | **right**: E2105, rules:818's second message word for word |
| reject_a_mutex_change_not_named_on_the_first_line | **unsettled**: E2105 and the message are AN-30's (line 11 counted right); whether the fixed program compiles depends on whether `main` must then say `changes Mutex[Array[String]]` (the question of `accept_a_mutex_argument_takes_no_word`) |
| reject_a_name_used_after_owns_this | **wrong** (message, fault 2): E3040 right (rules:259, 945); "read it before the call (`id = j.id` above line 14)" leaves line 15 `println(j.id, n)` reading `j`. Fix: add "and `println(id, n)` as line 15" |
| reject_a_plain_loop_name_changes_a_player | **right**: E3023 (rules:294-296) |
| reject_a_reads_method_called_before_its_field_is_set | **right**: E2100 (rules:280), AN-31 |
| reject_a_reads_method_reads_a_field_it_does_not_name | **right**: E2105 (rules:282, row 7) |
| reject_an_atomic_change_not_named_on_the_first_line | **unsettled**: as `reject_a_mutex_change_not_named_on_the_first_line` |
| reject_an_implementation_changes_more_than_its_interface_method | **right**: E2113 (rules:288-290, 927), rules:860's fourth message's form |
| reject_an_override_changes_more_than_its_virtual | **right**: E2113, rules:860's fourth message word for word |
| reject_changes_this_beside_a_field | **right**: E2107, rules:822 word for word |
| reject_changes_this_beside_reads | **right**: E2107 (rules:259-261, 924) |
| reject_changes_this_on_init | **right**: E2108, rules:823 |
| reject_main_does_not_name_a_module_variable_its_callee_changes | **right**: E2105, rules:818's first message's form |
| reject_mut_this_on_a_methods_first_line | **right**: E0113, rules:828's third message word for word |
| reject_owns_this_beside_changes_this | **right**: E2107 (rules:259-261) |
| reject_reads_names_a_field_the_body_never_reads | **right**: E2105 both ways (rules:282) |
| reject_self_names_a_methods_object | **right**: E1010, rules:825 word for word |
| reject_static_fn_outside_a_type | **right**: E2115, rules:827 word for word |
| reject_this_given_another_object | **right**: E2103, rules:824 word for word |
| reject_this_in_a_static_fn | **wrong** (message, fault 2): E1010 right (rules:826, 913) and the message is rules:826's word for word, but its way out, "remove `static`", leaves `main`'s `Counter.starting_at(5)` (line 13) calling a method on the type, refused. Fix for this program: "... has no object; write `return Counter(n=k)`" (or name the call to change as well); the gap is in rules:826 itself (for the owner) |
| reject_virtual_in_a_class_that_cannot_be_subclassed | **right**: E2112, rules:867 word for word |
| run_a_turret_aimed_at_itself_changes_itself | **right**: rules:275-279, verdict 18; `fire` and `main` name `Turret` (rules:264, 267); prints `9` |

## Tests, R6 (14)

| test | verdict |
|---|---|
| accept_a_call_returning_ref_mut_is_a_place | **wrong**: second line written `#$ 12` (fault 1). Fix: `#$ stdout: 12`. Otherwise right (rules:316-319, AN-23) |
| accept_a_push_after_the_generators_last_use | **wrong**: second line written `#$ 2` (fault 1). Fix: `#$ stdout: 2`. Otherwise right (rules:319-321, verdict 16: `from log` leaves `angle` out) |
| accept_from_narrows_a_function_result_to_one_parameter | **right**: rules:312-313; `w` points into `x` only; prints `2` |
| accept_from_src_lets_the_parser_move_on | **right**: AN-22's NEW (`area-B.md` from line 829); prints `let x let` |
| accept_the_argument_changes_after_the_views_last_use | **wrong**: second line written `#$ 2 1` (fault 1). Fix: `#$ stdout: 2 1`. Otherwise right (rules:314-316) |
| reject_a_change_to_any_part_of_the_argument_while_its_view_is_used | **right**: E3021 (rules:314-316, 386-391; writing a number is not a reshape, rules:41); lines 17 to 19 counted right |
| reject_a_function_result_ends_when_either_view_argument_grows | **right**: E3060 (rules:311-312, 348-351); lines 15 to 17 counted right |
| reject_a_method_result_points_into_its_whole_object | **right**: E3021 (rules:311, 386-391), AN-22; its line compiles (`accept_from_src...`) |
| reject_a_push_to_the_list_a_stored_generator_walks | **right**: E3020, rules:838's message with lines 18 to 20 |
| reject_a_reshape_of_a_list_the_argument_owns_ends_the_view | **right**: E3060 (rules:314-315) |
| reject_a_result_outside_what_its_from_names | **right**: E3062, rules:847 word for word in this program's names |
| reject_a_word_of_a_local_parser_returned_without_from | **right**: E3063 (rules:311, 366-368), AN-22 |
| reject_from_names_a_parameter_and_a_field | **right**: E2118, rules:848's fourth message word for word |
| reject_the_element_written_through_its_list_while_a_ref_mut_result_is_used | **right**: E3022 (rules:376-380), AN-23's message word for word with lines 12 to 14 |

## Tests, R17 (9)

| test | verdict |
|---|---|
| accept_a_body_fact_is_used_inside_its_package | **right**: rules:741 ("Inside a package such facts are used freely"); prints `[0] orc died` |
| accept_a_librarys_button_clicked_after_the_loop_over_log | **wrong**: second line written `#$ 2` (fault 1). Fix: `#$ stdout: 2`. Otherwise right (rules:524-535, verdict 40) |
| accept_a_private_generic_infers_its_bounds | **right**: rules:689-690; `Add[T, Output = T]` is valid (`std/src/core.em:267`, `interface Add[Rhs = Self]`); prints `5 5` |
| accept_a_pub_method_that_says_what_it_reads_from_another_package | **right**: rules:741-743; `note` says `reads frame`; prints `[0] orc died` |
| accept_an_only_adds_summary_crosses_the_package_edge | **right**: rules:744-748, AN-02's NEW across the edge; prints `2 80` |
| reject_a_call_from_another_package_that_may_do_more_than_add | **right**: E3060, as AN-02 part 6 refuses a call whose body the caller cannot see (`on_hit`), with its message's form; lines 11, 13, 14 counted right |
| reject_a_librarys_button_runs_a_programs_log_handler_in_a_loop_over_log | **right**: E3020, rules:530-533 and rules:836's second message's form, lines 15 to 17 |
| reject_a_pub_generic_without_its_bounds | **unsettled**: rules:690 ("a `pub` one writes them") refuses it, but no code in rules section 4 is given for a missing written bound; the test uses today's E2040, whose meaning is "unsatisfied interface bound" (`compiler/ember_diag/src/codes.rs:208`), not "bound not written" |
| reject_a_pub_method_with_no_reads_from_another_package | **right**: E3021 (rules:742-743, 376-380), AN-17; the lines it gives compile |

## Matrix rows (35)

| row | verdict |
|---|---|
| R4-1 | **right**: enforcing part plausible (today's E3023 is in `ember_typeck/src/lib.rs`); FN-9/CLS-7/FN-1 tests are related (today they accept what R4 refuses). Missing too: a plain parameter calling a `changes this` method (AN-18's `g.reset()`, E3023); `=` on a plain parameter (rules:811's first two messages, E3023) unless W1 tests it |
| R4-2 | **right** |
| R4-3 | **right**. Missing: a plain parameter's value handed to an `owns` parameter or pushed (E3013, rules:865's first message), no test anywhere in W2's set |
| R4-4 | **right**. Missing as listed, plus a lambda parameter whose mode differs from the expected function type (E2020, rules:861) |
| R4-5 | **right** |
| R4-6 | **right** as a row; two of its tests are wrong (`reject_ref_in_a_parameters_type`, `reject_mut_alone_on_a_parameter`) |
| R4-7 | **right** |
| R4-8 | **right** |
| R4-9 | **right**; `DSJ-4/reject_two_references_are_e3095.em` is only loosely related (it is about `assert_disjoint`'s argument types, not the overlap rule). One test wrong (`reject_a_read_and_a_change_of_one_game_in_one_call`) |
| R4-10 | **right** (`ember_mir/src/lower.rs:1733` is `lower_assert_disjoint`) |
| R4-11 | **unsettled**: the expected panic start `assert_disjoint` is not given by the rules (rules:772 gives the check, not its words) |
| R4-12 | **right**: a C check only; no test can observe it without `#$ assert-c:`, as the row says |
| R4-13 | **unsettled**: its accepts assume `main` names nothing for a callee's `changes Mutex[...]` / `changes Atomic[int]` on its own local (AN-30), against rules:267 and verdict 36 |
| R5-1 | **right** (today's E1010 is emitted in `ember_typeck`) |
| R5-2 | **right** |
| R5-3 | **right**. Missing: `@cached` with no `reads` (E2117, rules:850), which leans on `reads`; no writer's set names it |
| R5-4 | **right** |
| R5-5 | **unsettled**: the same question as R4-13 for three of its tests |
| R5-6 | **right** |
| R5-7 | **right**. Missing: a plain parameter calling a `changes this` method (see R4-1) |
| R5-8 | **right** |
| R5-9 | **right** |
| R5-10 | **unsettled**, as the writer says: rules:271 gives an order and no code for a line in another order |
| R6-1 | **right** |
| R6-2 | **right** |
| R6-3 | **right** |
| R6-4 | **right** |
| R6-5 | **right** |
| R6-6 | **right** (points to R4-8) |
| R17-1 | **right** |
| R17-2 | **right** |
| R17-3 | **unsettled**: no code for an unwritten `pub` bound (see `reject_a_pub_generic_without_its_bounds`) |
| R17-4 | **right** |
| R17-5 | **right**: observable only in the C (a header test removed), same output; no test, a C check once spelled |
| R17-6 | **wrong**: a guarantee of R17 (rules:752-754) with no test, though the row names the program. Fix: add `R17/reject_a_ledger_generic_whose_instance_changes_log_in_a_loop_over_log.em` (a `ledger` generic `[C, F: fn(int) changes C]` above `pub fn each(xs: Array[int], owns f: F) changes C:` called inside `for line in LOG:` with `fn(x) changes LOG: LOG.push(String("x"))`, E3020 at the call) and its accept with the call after the loop |
