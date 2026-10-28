# Check V4 of writer W4 (R9, R10, R11)

Checker V4, safety validation (b). Inputs: `inputs/brief.md`, `V3/rules.md`, `V3/vocabulary.md`,
`V3/inputs/JOB.md`, `S/matrix-W4.md`, and the 61 tests in `V3/conformance/R9`, `R10` and `R11`; the
answers they come from (AN-34, AN-36, AN-41, AN-48, AN-49, AN-51, AN-52, AN-62 to AN-65, AN-71). Nothing
was compiled or run. "line N" with no file means `V3/rules.md` line N. Today's test runner is
`compiler/ember_driver/tests/milestones.rs`.

Totals: 61 tests (56 right, 3 wrong, 2 unsettled); 27 matrix rows (25 right, 2 unsettled).

## The tests

### R9 (32)

- `accept_a_change_through_a_shares_mut_owner.em`: **right**. Lines 400-402: a `shares mut` owner changes the value; R3 line 147-148 (`items` may change, so `shares mut items` is allowed).
- `accept_a_field_change_through_a_shares_mut_field.em`: **right**. AN-48 (a)'s `shake`; R5 line 257-258: `changes target` covers "anything reached through" the field.
- `accept_a_number_changed_through_one_owner_while_a_loop_walks_the_other.em`: **right**. R8 (c) line 357 refuses only a change of the walked list; `hp` is a number (not a reshape, line 41) and `log` is another field; `item` views an inventory element, which neither change reaches (R8 (g)). AN-48 (a).
- `accept_a_number_in_a_sharedref_target_changed_while_another_holder_walks_its_list.em`: **right**. R7 lines 335-338, AN-48 (b).
- `accept_a_ref_mut_name_of_a_shares_mut_link_changes_the_player.em`: **right**. Line 405 ("Any `ref mut` name of it"), R10 line 421 names this very view; AN-51. The "no count" claim is in the `##` comment only (see the general note at the end).
- `accept_a_scene_two_panels_own_and_change.em`: **right**. The program AN-49's E1010 help gives, word for word; R9, R5 (`changes entities`).
- `accept_a_shared_list_element_changed_through_its_ref_mut_name.em`: **right**. R8 (f) lines 376-379: `other.len()` is after `boss`'s last use.
- `accept_a_thread_shared_object_filled_where_it_is_made.em`: **right**. AN-71's E7014 help applied; R3 line 143-145; R15 line 704-705 (capture word given).
- `accept_a_thread_shared_owner_reaching_another_thread.em`: **right**. AN-71's E7015 help applied; `thread_shared c` on a value already `thread_shared` (R3 line 144-145).
- `accept_a_view_used_before_another_owner_removes_its_element.em`: **right**. R8 (a), (g): the view's life ends at line 10.
- `accept_moving_items_between_two_shared_players_proved_apart.em`: **right**. AN-48's `transfer`; R8 (b) line 352-353 (`if a is not b:`). The call's two arguments are two different links (`x.target`, `y.target`); "into" does not go through links (section 0 line 30-32), so R4 line 232-239 does not refuse the call.
- `accept_one_scoped_task_sets_a_player_through_a_sharedref_link.em`: **right**. Lines 408-410: one task, and the parent reads `p` only after the join. `changes Player` on the lambda (R12 line 508-509) and on `main` (R12 line 570-573, `spawn` counts).
- `reject_a_change_through_a_shares_owner.em`: **right**. Line 401; E3023 (section 4 line 941). Message and help are AN-34's and the `shares` row of `V3/vocabulary.md`, word for word; the line it gives compiles (it is the accept test).
- `reject_a_field_change_through_a_shares_field.em`: **right**. R5 line 294-296; E3023; AN-48's message word for word; the two lines it gives make the accept test. (It names the two lines by their text, not by number, as AN-48 does.)
- `reject_a_list_changed_through_one_owner_while_a_loop_walks_the_other.em`: **right**. R8 (c), line 402-403; E3020; AN-48 (a)'s message with this file's lines (24, 25) correctly renumbered; the help's lines compile. Note: today's runner ignores `#$ note:` (milestones.rs, the `starts_with("note")` branch), so the note line asserts nothing; the help is checked.
- `reject_a_list_in_a_sharedref_target_changed_while_another_holder_walks_it.em`: **right**. R7 line 336-337, R8 (c); E3020; lines 23, 24 correct.
- `reject_a_plain_parameter_changes_a_player_behind_a_shares_link.em`: **right**. R5 line 294-296 ("an object behind a `shares` link"); E3023; the message has the section 3 E3023 row's shape ("... but `s` only reads ...; write `fn ...` and call ..."), names both lines it replaces, and the fixed program is AN-51's.
- `reject_a_shared_list_element_changed_while_a_view_reads_it.em`: **right**. R8 (g) line 386-390 and R8 (b) line 351-353 (one path is the `shares mut` link `other`); E3021 at the change; the section 3 E3021 row in this program's names, both ways out; the second way out is the accept test.
- `reject_a_shares_owner_reaching_another_thread.em`: **right**. Line 403-404, E7015 (section 4 line 960); AN-71's message with lines 16 and 17 correctly renumbered (AN-71 had 9 and 10); the fix is the accept test.
- `reject_a_thread_shared_object_changed_directly.em`: **right**. Line 407-408, E7014; AN-71's message, lines 12 and 13 correct.
- `reject_a_view_used_after_another_owner_removed_its_element.em`: **right**. R8 (a) line 346-347; E3060 at the later use; the section 3 E3060 row's shape; `first = names[0]` after line 10 re-points the view (R2 line 121-124) and compiles.
- `reject_an_unscoped_thread_holding_a_sharedref_link.em`: **right**. Line 411-412 names E7015 for this case; the help replaces lines 17 and 18 and gives the accept test.
- `reject_moving_items_between_two_shared_players_that_may_be_one.em`: **right**. R8 (b) line 351-353, R8 (c); E3020; AN-48's message word for word.
- `reject_refcell_is_not_ember.em`: **right**. Line 412-413; E1010 (section 4 line 913 lists `RefCell`); AN-49 / vocabulary `RefCell` row word for word.
- `reject_two_tasks_set_one_player_through_two_sharedref_links.em`: **right**. Line 409-411; E7007; `=` is not in R15's reductions (line 701). The fixed program (every changed line named by number) compiles: `Atomic` operations through any name (R5 line 300-304), `changes Atomic[int]` (R12 line 511-512).
- `run_a_shared_object_ends_with_its_last_owner.em`: **right**. Output traced: `consumed 1`, `still 1`, `end 1`, `done` (lines 58-61; R4 line 205, hand-over to `owns t: shares Token` with no word, line 230-231). The `drop` line follows vocabulary line 548 (see `for_owner` on E2105).
- `run_a_shares_owner_sees_what_the_first_owner_changes.em`: **right**. AN-34's `owner_that_reads`, verdict 23; prints `4`.
- `run_a_thread_shared_object_read_by_a_thread_and_its_maker.em`: **right**. Prints `1 1` (today's `println(a, b)` separates by a space, e.g. `tests/conformance/ATT-1/accept_the_listed_attributes.em`).
- `run_a_tree_child_with_two_owners_changed_through_either.em`: **right**. AN-36's second form; prints `5`, `6`; L3001 text is AN-36's word for word; R11 line 501-502 (a `shares mut` field of its own type).
- `run_two_sharedref_mut_holders_and_the_owner_change_one_player.em`: **right**. Traced 100 -1 -3 +10 = 106; R7 line 335-336 lets the owner change it while the holders live.
- `run_two_shares_mut_owners_both_change_one_list.em`: **right**. AN-34's `two_owners`; `4`, `5`.
- `run_two_shares_mut_owners_change_one_counter_with_no_run_time_check.em`: **right**. `shares mut` before a new value (vocabulary `shares` row: `count = shares mut Counter()`); traced 1 + 1 + 1 = 3 through either owner, `3 + 3 = 6`.

### R10 (6)

- `accept_a_view_from_a_link_used_after_a_call_that_changes_nothing.em`: **right** for compile-pass (R10 line 439-441 refuses only across `changes T`; R12 line 512 "a plain `fn()` changes nothing at all"; `changes on_hit` for the call through a stored function, R12 line 641-643). The `##` claim "costs nothing" rests on R10 point 1, which is open for a function value whose own state owns a `Player` (see `for_owner`).
- `reject_a_view_from_a_link_used_after_a_call_that_may_change_players.em`: **wrong** (the message). Result, place and code are right (R10 line 439-441, R8 (g), E3021). But the first way out, "write `println(p.hp)` above line 16", leaves line 17 in place, which is still refused, so the line it gives does not make the program compile (R18 line 759-760: name every line it replaces). Fix: "use `p` before the call: move `println(p.hp)` (line 17) above line 16".
- `run_a_view_kept_across_a_loop_of_calls_that_may_end_players.em`: **unsettled**. The trace under R10 point 2 gives `100`, `gone`, but R8 (b) line 348-351 says a view reached through a link "ends at any change that may reshape what it points into" (E3060 at a later use), and section 0 line 40-41 counts "ending an object's last owner" as a reshape; `drop_last` may end the player `p` views, and `p` is used on line 33. R10 lines 419-439 say the compiler keeps it alive instead. The rules do not say which wins (W4's own `for_owner` item for R10-3).
- `run_a_view_of_a_thread_shared_object_through_a_link_holds_an_atomic_count.em`: **right**. R10 point 4 line 436-438 ("always"); R11 line 445 (a `thread_shared` value may be a `weakref` target); prints `3`.
- `run_a_view_opened_from_a_link_keeps_its_player_across_a_call_that_ends_it.em`: **unsettled**, the same rule text as above (R8 (b) line 348-351 and section 0 line 40-41 against R10 point 3 line 435).
- `run_a_view_opened_from_a_link_needs_no_protection_when_nothing_can_end_a_player.em`: **right**. R10 point 1; AN-65; prints `100`.

Note on the two R10 run tests above: `if cam.target exists as q:` never reads `q`, and `for i in 0..2:` never reads `i`. If the built compiler reports an opened or loop name nobody reads as today's L1001 does for locals (`docs/spec/part-20-toolchain.md` `[LNT-1]`, on by default), the runner fails the test on an unclaimed warning ("unexpected and missing diagnostics both fail", milestones.rs `[TST-1]`). Safer: `exists as _q` and `for _ in 0..2:`.

### R11 (23)

- `accept_a_copy_of_a_links_target.em`: **right**. R11 line 453-454, verdict 10; `@derive(Clone)` makes `copy of p` legal (section 3 E2116 row's fourth message).
- `accept_a_cycle_of_shares_owners_is_warned.em`: **right**. R11 line 501-502; AN-64's warning and help word for word; today's runner fails a test on any unclaimed or missing warning, so the expectation is enforced.
- `accept_a_link_made_from_a_read_only_name_changes_nothing.em`: **right**. R11 line 449-451 covers both cases it names; a `weakref` is a link, not a view (section 0 lines 24-35), so returning one from a plain parameter is not E3063.
- `accept_a_link_opened_before_its_target_is_used.em`: **right**. AN-65's NEW.
- `accept_a_parent_link_set_by_the_program.em`: **right**. R11 line 455-456; AN-41, AN-63.
- `accept_a_tree_with_parent_links_gets_no_cycle_warning.em`: **right**. R11 line 501-502 (no `shares` field); AN-64; the runner's exact-warnings rule makes "no warning" a real check.
- `accept_a_walk_with_views_when_the_visit_changes_no_node.em`: **right** for compile-pass (AN-62 (a); R11 line 500-501; verdict 28). The "no header test, no count inside the `while`" claim rests on R10 point 1 too (see `for_owner`: a `fn(Node)` whose own state owns the walked nodes).
- `accept_an_object_stored_into_a_link_with_weakref.em`: **right**.
- `accept_another_link_to_the_same_target.em`: **right**. R3 line 145-146, R11 line 447.
- `reject_a_link_used_as_its_target_without_opening.em`: **right**. E2020, section 3 line 861 word for word.
- `reject_a_parent_field_that_owns_its_parent.em`: **right**. E3013 (section 4 line 937), AN-63's message word for word; the fix is the accept test. `Option[Node]` in a class is not E2200 (a class object has its own block, R3 line 158).
- `reject_a_struct_node_keeping_a_view_of_its_parent.em`: **right**. E2200, section 3 line 855 word for word.
- `reject_a_walk_with_views_across_a_visit_that_may_change_nodes.em`: **right**. R11 line 474-477, verdict 28; E3021; AN-62's version of the section 3 row (with `if leaf exists as n:` in "instead of", which the rules' row leaves out, AN-62 note 4), lines 28 and 30 correct; following it gives the run test's walk.
- `reject_an_object_stored_into_a_link_without_weakref.em`: **right**. E2020, section 3 line 861.
- `reject_copy_of_a_link.em`: **wrong** (the program). Code and message are right (R11 line 453, E2116, AN-65 word for word), but `Player` here has no `Clone`, so the message's second way out, `c = copy of p`, is itself refused in this program (E2116, "`Player` has no `Clone`", section 3 line 849), against R18 line 759 ("The line a message gives compiles ... in that program"). Fix: add `@derive(Clone)` above `class Player` (as `accept_a_copy_of_a_links_target.em` has); see `for_owner` for the message in programs without `Clone`.
- `reject_upgrade_is_not_ember.em`: **wrong** (the program). Code and message are right (E1010; vocabulary `upgrade` row; AN-65). But line 16, `println(p)`, prints a `Player`, which has no `Display` (`docs/spec/part-04-type-system.md` `interface Display`), so the program the message leads to (`if cam.target exists as p:` with `println(p)` under it) is still refused. Fix: `println(p.hp)` on line 16.
- `run_a_parent_link_that_does_not_follow_ownership_finds_its_target_gone.em`: **right**. R11 condition 1 line 459-461 fails on line 29; traced `other/leaf`, then `other = ...` ends the old node (R2 line 122-123), `leaf`.
- `run_a_proved_parent_walk_prints_the_path.em`: **right**. AN-63's NEW; condition 1 and 2 hold as the comment says; traced `root/a/leaf`, `leaf`, `root/leaf`.
- `run_a_second_link_finds_its_target_gone_after_the_first_link_and_the_target_end.em`: **right**. R11 line 446-448, verdict 11; traced `7`, `gone`.
- `run_a_walk_follows_a_node_moved_under_another_parent.em`: **right**. Traced: visit `mid` moves it under `root` (`take` then `add`), next step finds `mid` alive with parent `root`, visits `root`, then `root.parent` is empty: `visit mid`, `visit root`, `depth 2`. `with weakref root` is a named capture (own state), so R12 line 560-563 does not refuse storing it in `t` (AN-62's `prune` is the same shape).
- `run_a_walk_over_nodes_with_a_drop_method_stops_at_the_first_ended_parent.em`: **right**. No region (R10 point 2 needs no `drop`), every link counted (R11 line 496-500); traced `visit mid`, `end mid` (from `clear`), `depth 1`.
- `run_a_walk_whose_visit_may_end_nodes_stops_at_the_first_ended_parent.em`: **right**. AN-62 (b) and its C (`visited=m depth=1`); verdicts 15, 28.
- `run_an_empty_link_opens_to_its_else_branch.em`: **right**. R11 line 445, R13; traced `gone`, `100`, `gone`.

## The matrix rows

- R9-1: **right**. Enforcing parts plausible; the today's tests are about today's `Shared` and named owners, related. Missing as listed.
- R9-2: **right**. Also missing: AN-36's E3023 through a `shares` element (`root.children[0].value = 6` with `children: Array[shares Node]`), whose message AN-36 gives; W4 lists it but wrote no test.
- R9-3: **right**.
- R9-4: **right**. Also missing: AN-48's E3020 for `cam.shake()` (`changes target`) called inside the loop, whose message AN-48 gives word for word; listed, no test.
- R9-5: **right**. E3021 and E3060 in `compiler/ember_analysis/src/borrows.rs` (grep confirms both codes there).
- R9-6: **right**. Missing as listed (E3060 through another holder, R7 line 336-337).
- R9-7: **right**. Missing: E7015 for a `weakref` link of a one-thread object (section 4 line 960, AN-71's note); listed, no test.
- R9-8: **right**.
- R9-9: **right**. Missing: E7007 for "a task and the parent changing one place" (line 410), named in the rule itself; listed, no test.
- R9-10: **right**. Missing as listed (`Ref`, `RefMut`, `borrow`, `borrow_mut` each E1010).
- R9-11: **right**.
- R9-12: **right**: a layout, not a refusal or a run-time check, so no test is owed.
- R10-1: **right**. Its missing edge case (a function value whose own state owns a `Player`) is a possible hole, not only a missing test: see `for_owner`.
- R10-2: **unsettled**: R8 (b) line 348-351 with section 0 line 40-41 ("ending an object's last owner" is a reshape) against R10 point 2 lines 428-434.
- R10-3: **unsettled**: the same text against R10 point 3 line 435 (W4 flags it too).
- R10-4: **right**.
- R10-5: **right** as a row; its reject test's message needs the fix above.
- R11-1: **right**.
- R11-2: **right**.
- R11-3: **right**.
- R11-4: **right** as a row; `reject_upgrade_is_not_ember.em` needs the fix above. Also missing: `Weak[T]` and `Weak(x)` (E1010, vocabulary `Weak` row) and `weakref` of a part (`weakref pos`, vocabulary `weakref` row, R11 line 445 "its target lives on its own"), neither tested.
- R11-5: **right** as a row; `reject_copy_of_a_link.em` needs `@derive(Clone)`. Its missing case (`copy of` a value holding a `weakref` field) is open in the rules: see `for_owner`.
- R11-6: **right**. E2200 today means "a generator frame that holds itself" (`compiler/ember_analysis/src/coroutines.rs`), the nearest work, not the type checker named in R11-7; plausible either way.
- R11-7: **right**.
- R11-8: **right**.
- R11-9: **right**.
- R11-10: **right**.
- Section 2 table and "Refusals covered": **right**: every section 2 row that belongs to R9-R11 (lines 775-781) has a run test, none stops the program, and every refusal listed has a `reject_` and a nearest `accept_`.

## General note on the run tests

Every claim about the emitted C (no count change, one region counter add and subtract, an atomic count,
no header test on a proved parent read, a weak-count move per step) is in the `##` comments only. Today's
runner already reads `#$ assert-c: contains(...)`, `#$ assert-c: !contains(...)` and
`#$ assert-c-count: contains(...) == N` (milestones.rs, the parser's key list). Once the model's C names
exist (for example the region counter and the weak-count calls), these claims should become such lines,
or they are checked by hand only.
