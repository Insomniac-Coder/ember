# Readability check of run 4, step 3, v4 (agent 13)

Agent 13 (readability checker), run 4, step 3, phase 3. This file checks every NEW block of the five v4
area files and every conformance test changed or added in this step against the owner's test, the
error-message rule, verdict 73, and the owner's verdicts 1 to 82 (27 replaces 25, 26 replaces 1) with no
old spelling left. It fixes nothing. `V3` means `tasks/96/redesign/run4/v3/`, `V4` means
`tasks/96/redesign/run4/v4/`; "R1" .. "R18" are the rules of `V4/rules.md`; "verdict N" is item N of
`inputs/JOB.md`, "Owner verdicts on run 4's open questions"; "row N" is the owner's agreed row N. Line
numbers are lines of the file named, as it stood when this check ran (26 October 2026, from 22:53).

No compiler change, no git command, no Ember program run. Power was on mains at the start and at the end
(`PowerLineStatus` printed `Online`).

## 0. Terms

- **NEW block**: code an area file proposes under the v4 rules: any fenced `ember` block of an answer
  outside parts 2 (OLD) and 5 ("means exactly"), the code blocks outside the answers, and one block this
  step added inside a part 5 (AN-53's `make_counter`, `V4/area-D.md` 298-309, which is v4 spelling, not
  today's Ember). OLD code and "means exactly" programs are today's Ember and are not judged.
- **message**: the text of a refusal, a warning or a run-time stop: an answer's part 6 (inline or as a
  `text` block), its part 3 where it shows one, and a test's `#$ error[...]`, `#$ help:`, `#$ note:` and
  `#$ panics:` lines.
- **test changed or added**: a file of `V4/conformance/` that is not byte for byte its `V3/conformance/`
  copy (296 changed .em files), or that `V3` does not have (150 added .em files): 446 tests. Three v3
  tests are gone (two R10 `run_` tests that relied on R10's old keep-alive, verdict 82; one R15 refusal
  that verdict 77 turned into an acceptance).
- **the owner's test**: for every name a line declares, the word on the line that says what it is (the
  owner's own thing, a copy, or handed over) and the word that says whether it may change it; a name
  with no word passes only where the stated default never changes anything, never aliases silently and
  never copies silently.
- **the message rule**: a message names what was refused in the program's own names, says why in one
  line, and gives the exact line(s) to write, which compile in that program (R18).
- **verdict 73**: a message's fix never runs into another error; where two errors fit one line, the one
  whose fix makes the program compile is shown.
- **twin**: the same message or line in another file (the rules, the vocabulary, an answer, a test).
- **CONFIRMED**: settled by the rule text, no reading needed. **PLAUSIBLE**: depends on a point the
  verdicts leave open; it is listed for the owner and decided here by no one.

## 1. What was checked, and how

Read in full: `inputs/brief.md`, `inputs/JOB.md`, `inputs/worklist.md`, `inputs/duties.md` (section 13),
`V4/changes.md`, `V4/rules.md`, `V3/readability.md` (sections 0 to 3, for the shape), every part of the
five area files but the OLD and "means exactly" parts (read through stripped copies that keep the line
numbers), and all 446 changed or added tests. Where a message pointed at an OLD program, that program was
read too.

Scripts, all under `target/redesign6/13/`:
- `tdiff.py`: lists the tests added (`added.txt`), changed (`changed.txt`) and removed against `V3`.
- `extract.py`: pulls every fenced block out of the five area files, labels it by answer, part and kind,
  and marks whether its text is new in this step (`blocks.txt`, `blocks.json`); writes the stripped
  copies `new_area-A.md` .. `new_area-E.md` and the compact reading copies `compact_A.txt` ..
  `compact_E.txt`. Result: 117 NEW `ember` blocks (A 26, B 20, C 25, D 21, E 25), 25 of them changed in
  this step; 51 NEW `text` blocks holding messages (C 44, D 2, E 5), 31 changed in this step.
- `scan.py`, `scan2.py`: every NEW block and every test (code with comments stripped, and every
  backticked span in a comment or message) searched for the old spellings: `holds`, `cast x if`/`when`,
  `.copied()`, `owned`, `owned fn`, `shared` as an owner word, `copied`, `self`, `mut self`, `weak` alone,
  `set by`, `Some(...)` in an `if`/`while` line, type parameters in brackets after a function's, struct's
  or class's name, `mem.forget`, `ref mut this` as a parameter, `as?`/`as!`, `Box[`, `RefCell[`,
  `Weak[`, `Shared[`, `SyncShared[`, `ForeignBox[`, `Mut[`, `.upgrade(`, `.clone()`, run 4's `if p = x:`,
  a plain `MutSpan` parameter, a plain `= ....as_mut_span()`, `changes this` on a `drop`, `with ref mut`,
  `global`, `@sync`/`@nosync`/`@static_safe`/`@borrows`/`@no_runtime_checks`/`@fp(contract)`, `str`
  and `cstr` as types (`scan_blocks.txt`, `scan_tests_code.txt`, `scan_tests_spans.txt`).
- `scan3.py`, `addedlines.py`: the same search on every backticked span of the stripped area files, the
  rules and the vocabulary, kept to the lines this step changed (`scan_spans_added.txt`; every hit read
  in its sentence).
- `linkscan.py`: every function of every test (changed or not) and of every NEW block that holds a link
  (`shares mut`, `sharedref mut`, `thread_shared`, an opened `ref mut` name) and changes through it, with
  no `changes` on its first line (verdicts 63, 64); 10 hits, every one a change inside a lambda that names
  it, or a test's intended refusal (`linkscan.txt`).
- `review_list.txt`, `tests_R1.txt` .. `tests_R17.txt`: the 446 tests, numbered, as read.
- A check that every test named in a test's comment as its nearest allowed or refused program exists:
  all do but three names of today's `tests/conformance/` (`refs.txt`, `names.txt`).
- By hand: every owner's-test table of the 84 answers against its NEW code; every declared name of the
  NEW blocks and of the 446 tests; every message in parts 3 and 6 and every `#$` message line; every
  `run_` test's stop text against `V4/rules.md` section 2's table.

## 2. Results at a glance

| test | result |
|---|---|
| no old spelling left in NEW code and in the changed and added tests | **passes**: every hit is a refused word in a message or a refused line of a `reject_` test, an English word ("holds", "copied", "owned" in a file name), today's code quoted, or a carried OLD or C block. One identifier reads like a removed keyword (F-16) |
| the owner's test on every declared name | **passes** in every NEW block and every test: each declared name has its word, or a stated default that never changes, aliases or copies silently; the tables match the code. Three spellings the verdicts leave open make the same thing read two ways (F-03, F-05, F-07) |
| the message rule | **4 failures** (F-01, F-12, F-13, F-14): a fix that names a name already in use (five places), messages that give no line or a help line that repeats the program (five kinds), the caller named by the callee's parameter, a stop with no text; and one answer's prose that contradicts its code (F-15) |
| verdict 73 (a fix never runs into another error) | **1 CONFIRMED failure** (F-01, five places, the tests right), **4 PLAUSIBLE** (F-02, F-04, F-06, F-10), each resting on a point already listed for the owner |
| verdicts 53 to 82 applied everywhere | **yes**, with the open points of section 4: verdict 74 is spelled two ways for an `owns` inside an `Option` (F-03); verdict 80's own line meets E2085 (F-02); verdict 53's call check meets verdict 28's walk (F-06) |
| verdicts 1 to 52 applied everywhere | **yes** (no regression of `V3/readability.md`'s results in the blocks this step touched) |
| agreed rows 3 to 9 | **as agreed**, in the verdicts' spellings (section 5); row 9's message gains a first line in AN-34 only (F-04) |

## 3. Failures

Most severe first. Each with the blocks it is in, the test it fails, and why.

### F-01. "Open the link again" reuses the name the open block already holds (CONFIRMED; verdict 73, E2106)

- **Blocks.** The E3060 way out for a name opened from a link and used after a call that may end its
  object writes the opening again with the same name, inside the block where that name is in use:
  - `V4/rules.md` 515-516 (R10 point 1): "write `if cam.target exists as p:` above line 13 and indent line
    13 under it", where line 13 is inside the block opened `if cam.target exists as p:` on line 10;
  - `V4/rules.md` 1017 (section 3, E3060 row): the same in the `remove_all` message, and again in the
    `reset()` message ("write `if cam.target exists as p:` above line 8");
  - `V4/vocabulary.md` 384 (the `exists as` row): the `remove_all` message, word for word;
  - `V4/area-D.md` 2152-2156 (AN-65 part 6): "write `if cam.target exists as p:` above line 10", inside
    the block opened on line 8;
  - `V4/area-D.md` 1771-1775 (AN-62 part 6): "write `if leaf exists as n:` above `println(n.value)`",
    inside the block opened `if leaf exists as n:`.
- **Test:** verdict 73 and R18 (the line a message gives compiles in that program).
- **Why:** R13: "The name is new: a name already in use is refused (E2106)". The opened name is still in
  scope where the new opening goes, so the line given is refused itself. The tests of this step write it
  right, with a new name: `R10/reject_a_view_from_a_link_used_after_a_call_that_may_end_its_player.em`
  line 22, `R10/reject_a_view_from_a_link_used_after_a_function_that_ends_the_players_it_owns.em` line 20,
  `R10/reject_a_view_from_a_link_used_after_a_loop_of_calls_that_may_end_players.em` line 27,
  `R10/reject_a_view_from_a_link_used_after_a_call_that_may_change_players.em` line 18 and
  `R12/reject_a_name_opened_from_a_link_used_after_a_call_that_may_end_players.em` line 23 ("write
  `if cam.target exists as q:` above line 23, indent line 23 under it and use `q` there"; its comment
  names E2106 as the reason). The five places above are the tests' twins and disagree with them.

### F-02. Verdict 80's `with owns` on an `int` against E2085 (PLAUSIBLE; verdict 73; for the owner)

- **Blocks.** `V4/rules.md` 749-755 (R12) and 1044 (section 3, E3026's second message): "write `return
  fn() with owns n: n += 1`", `n` an `int`; `V4/area-D.md` 298-309, the block this step added to AN-53's
  part 5 (`return fn() with owns count -> int:`, `count = 0`), and its owner's-test rows 260-262 and
  part 6's E3026 message (336). Against: `V4/rules.md` 1036 (E2085 row: "in a capture list: write
  `with copy of score`"), 239 (R4: `owns` "refused where `T` is known to be `Copy`, E2085"),
  `V4/area-D.md` 507-508 (AN-54 part 6, `with owns score` on an `int`, E2085) and
  `R3/reject_owns_on_a_captured_number.em` line 9 (E2085, help `with copy of score`).
- **Test:** verdict 73 (the E3026 line given must not run into E2085); keywords mean the same everywhere.
- **Why:** for an `int` the rules give two opposite lines for one capture: E3026's fix writes
  `with owns n`, which E2085 refuses with "write `with copy of`". The tests avoid the clash by keeping a
  `struct Tally` (`R12/accept_a_returned_function_owns_the_variable_it_keeps.em`,
  `R12/reject_a_returned_function_that_changes_its_makers_local.em`,
  `R12/run_a_counter_made_by_a_function_keeps_its_variable.em`), so no test shows verdict 80's own line.
  Area D lists it for the owner (its "For the owner", item 2). The block in AN-53's part 5 also adds a
  NEW-spelling program to a "means exactly" part, which the brief keeps for today's Ember (the carried
  program itself is unchanged).

### F-03. `owns` on a making line inside an `Option`, a list or a local: two spellings (PLAUSIBLE; verdict 74; for the owner)

- **Blocks.** With `owns`: `R15/accept_a_generator_views_what_its_caller_keeps.em` line 22 and
  `R15/reject_a_generator_walk_held_by_views_of_its_own_frame.em` line 11 (`next=Some(owns
  Link(value=1))`, their comments citing verdict 74). Without: the same refused generator in
  `V4/area-E.md` 1816 (AN-77 part 6, `next=Some(Link(value=1))`), whose E2221 message the R15 test
  copies; `R3/run_a_value_stored_into_an_owns_place_gets_its_own_block.em` line 15 and `V4/area-C.md`
  1044 (AN-41, `left=Some(Tree(value=2))`); `V4/area-C.md` 881 (AN-40, `r.pools.push(IntPool(items=[1,
  2]))` into `Array[owns dyn Pool]`); `V4/area-D.md` 2225 (AN-66, `shapes=[Circle(r=1.0),
  Square(side=2.0)]` for `Array[owns dyn Shape]`); `R14/run_fail_a_cast_to_the_wrong_kind_stops_the_program.em`
  lines 24 and 27 (`a: owns dyn Shape = Circle(r=3)`) and `R14/reject_cast_if_is_not_ember.em` line 30.
- **Test:** the owner's test (a word on the line says the value is put in a block of its own, or a
  stated default does) and "keywords mean the same everywhere".
- **Why:** verdict 74 and R3 name an `owns` *field*. Whether an `owns` inside an `Option`, a list's
  element type or a local's type takes the word on its making line is open (area C's "For the owner"
  item 3, area D's item 3, area E's item 5); meanwhile the same refused program is written both ways in
  an answer and in its test, so a reader meets `Some(owns Link(...))` and `Some(Link(...))` for one
  field. Either spelling passes the owner's test (the default "a plain value stored into an `owns T`
  place is put in its block at that store" is stated in the vocabulary's `owns` row); the two together do
  not read as one rule.

### F-04. Row 9's message, and a first owner that pushes after sharing (PLAUSIBLE; verdict 73; for the owner)

- **Blocks.** The tests of this step read the first owner of a shared value as a link, so a change
  through it is named: `R8/accept_a_shares_owner_sees_a_change_through_another_owner.em` line 9,
  `R9/run_a_shares_owner_sees_what_the_first_owner_changes.em` line 9,
  `R9/run_two_shares_mut_owners_both_change_one_list.em` line 9,
  `R9/run_a_tree_child_with_two_owners_changed_through_either.em` line 16 (`fn main() changes
  Array[int]:`, `changes Node`), as `V4/area-C.md` 160-170 (AN-34's `owner_that_reads`) does. Under that
  reading, row 9's `shares` way out (`snapshot = shares items`, then `items.push(4)`) needs `fn main()
  changes Array[int]:` too. `V4/area-C.md` 193-197 (AN-34 part 6) adds it; `V4/rules.md` 977 (section 3,
  E3040) and `R1/reject_a_list_used_after_it_was_handed_over.em` lines 3 and 11 do not.
- **Test:** verdict 73 (the fix compiles); twins say the same.
- **Why:** one reading makes the rules' and the R1 test's way out run into E2105; the other makes the
  four tests above and AN-34 name a change the rules do not ask for. Area C lists the point for the owner
  (item 1); the twins should follow whichever the owner picks. Row 9's agreed content (both ways out
  named) is kept in every version.

### F-05. A method changing a player through its own field says `changes target`, its caller `changes Player` (PLAUSIBLE; for the owner)

- **Blocks.** `fn shake() changes target:` with `this.target.hp -= 1` through a `shares mut` or
  `sharedref mut` field: `V4/area-C.md` 1729 and 1762 (AN-48), `R7/accept_a_camera_owning_a_share_may_be_returned.em`
  line 14, `R7/accept_sharedref_mut_holders_and_the_owner_all_change_one_player.em` line 19,
  `R7/reject_a_method_that_may_change_the_target_inside_a_loop_over_it.em` line 15,
  `R9/accept_a_field_change_through_a_shares_mut_field.em` line 13,
  `R9/run_two_sharedref_mut_holders_and_the_owner_change_one_player.em` line 16; their callers say
  `fn main() changes Player:` for `cam.shake()`.
- **Test:** the owner's test on the first line (the word that says what may change) and "keywords mean
  the same everywhere" (verdicts 63, 64: "a change through a link always says `changes T`, whatever holds
  the link").
- **Why:** one change of one player is named `changes target` on the method and `changes Player` on its
  caller. R5's "`changes a, b` ... and anything reached through them" allows the first; verdict 64's
  wording asks for `changes target, Player`. Area C lists it (item 2).

### F-06. R4's call check refuses verdict 28's own walk, and the E3021 message that leads to it (PLAUSIBLE; verdict 73; for the owner)

- **Blocks.** `V4/rules.md` 281-283 (R4: "a call whose body it cannot see counts as using every such
  parameter after every change its first line or type allows"), against `this.on_visit(p)` with
  `on_visit: fn(Node) changes Node` in `V4/rules.md` 573-581 (R11's walk), `V4/area-D.md` 1543-1554
  (AN-62 (b)), `R11/run_a_walk_whose_visit_may_end_nodes_stops_at_the_first_ended_parent.em` line 35 and
  `R11/run_a_walk_over_nodes_with_a_drop_method_stops_at_the_first_ended_parent.em` line 39 (both
  `run-pass`); and the E3021 walk message (`V4/rules.md` 985, `R11/reject_a_walk_with_views_across_a_visit_that_may_change_nodes.em`
  line 29), whose fix is that walk.
- **Test:** verdict 73; the tests' expected results.
- **Why:** read as written, `p` is a view argument of a call through a function value whose type may
  change every `Node`, so the call is E3031, and the two `run-pass` tests and the E3021 message's line
  would be refused. Area D lists it for the owner (item 1); the rules and the tests disagree until it is
  settled.

### F-07. A change of an `Atomic` inside an object reached through a link: two names (PLAUSIBLE; for the owner)

- **Blocks.** By the object's type: `R12/accept_a_thread_shared_owner_held_by_a_function_any_thread_may_run.em`
  line 26, `R12/reject_a_shares_mut_owner_held_by_a_function_any_thread_may_run.em` line 25,
  `R15/run_a_thread_runs_a_handler_holding_a_thread_shared_owner.em` line 30 (`fn() with thread_shared
  count changes Counter: count.n.fetch_add(1)`, verdict 61's own spelling). By the `Atomic`'s type:
  `R9/reject_two_tasks_set_one_player_through_two_sharedref_links.em` line 21, E7007's way out
  (`fn() changes Atomic[int]: a.target.hp.store(1)` and `fn main() changes Atomic[int]:`).
- **Test:** "keywords mean the same everywhere"; R12 ("named by the type of what changes ...
  `changes Atomic[int]`"; "naming by type is for what is reached through a link").
- **Why:** both are a change of an `Atomic` field of an object reached through a link, and one is named
  by the object, the other by the field's type. R12's words fit both. The R9 message is the one that
  differs from verdict 61's example.

### F-08. A callee's change named by a type, on the caller's own local (PLAUSIBLE; for the owner)

- **Blocks.** `fn main() changes Mutex[Array[String]]:` for `main`'s own `m`: `V4/area-B.md` 2127 (AN-30,
  meaning changed), `R4/accept_a_mutex_argument_takes_no_word.em` line 23,
  `R8/accept_copies_out_of_a_lock.em` line 22, `R5/reject_a_mutex_change_not_named_on_the_first_line.em`
  line 11; `fn main() changes Atomic[int]:` for `main`'s own `total`, `R5/accept_an_atomic_changed_through_a_plain_name.em`
  line 13; and a task that changes the captured `m` through `add_one` names the type,
  `fn() changes Mutex[Array[int]]: add_one(m)` (`R15/accept_a_task_takes_the_lock_itself.em` line 21,
  `R15/reject_a_lock_guard_sent_to_a_task.em` line 15's way out), where verdict 75 names a captured
  `Atomic` by its variable (`fn() changes hits:`).
- **Test:** the owner's test on the first line: `main` says it changes a `Mutex[...]` that is its own
  local, which R5 elsewhere says needs no clause; a captured variable is named by its name (verdict 75)
  or by a type (this rule) depending on whether the change is direct.
- **Why:** this is the rule writer's reading (R5, from verdict 36's example), listed for the owner
  (`V4/changes.md` section 12, item 3; area B's "For the owner", item 1). The lines are consistent with
  the rules as written; they are listed here because the same captured `m` reads two ways.

### F-09. E3031 on a `ref mut` parameter rests on the rule writer's reading (PLAUSIBLE; for the owner)

- **Blocks.** `R4/reject_a_ref_mut_argument_into_what_the_callee_clears.em` (E3031 for
  `rename(ref mut LOG[0])`) and its allowed neighbour `R4/accept_a_ref_mut_argument_of_your_own_given_to_a_function_that_clears_log.em`.
- **Test:** every verdict applied, no wider.
- **Why:** verdict 53 says "a view parameter"; the rules widen it to `ref mut` (`V4/changes.md` 2.1,
  section 12 item 2). The test's message is clear and its fix compiles; the refusal itself waits on the
  owner.

### F-10. E7007's atomic way out changes what the program says (PLAUSIBLE; for the owner)

- **Blocks.** `V4/rules.md` 1054 (E7007 row: "count it with an atomic: ... `if stats.total.load() < 10:`
  ... `stats.total.fetch_add(x)`"), `R15/reject_a_task_reads_a_field_another_task_adds_to.em` line 18
  (the same way out), and its allowed neighbour `R15/accept_a_task_reads_an_atomic_field_another_task_adds_to.em`
  (`if stats.total.load() < 10: stats.total.fetch_add(x)` in each task). Against: `V4/area-E.md` 1025-1036
  (AN-74 part 6), which gives only the lock and says why.
- **Test:** "a rule that makes code compile but do something the programmer did not mean is wrong"
  (`inputs/JOB.md`, "First, the intent of the program"); twins say the same.
- **Why:** with two separate atomic steps, two tasks can both read 9 and both add, so the total can pass
  10, which the line `if stats.total < 10:` says it must not. No memory error, but not the program's
  meaning; the lock way keeps the read and the add together. Area E lists it (item 3).

### F-11. A refused test that depends on an unsettled overlap (PLAUSIBLE; for the owner)

- **Blocks.** `R15/reject_a_walk_under_the_lock_across_a_function_value_that_changes_log.em` lines 18
  and 27: `dump(log, fn() with thread_shared log changes Log: ...)`, a plain `thread_shared Log` argument
  beside a `fn() changes Log` argument, the question of `V4/changes.md` section 12, item 5 (area E's
  item 6, AN-75 part 6).
- **Test:** one fault per refused test, its message the one shown (verdict 73).
- **Why:** if the owner rules that the two arguments overlap (E3021), the test has a second fault on
  line 27 before line 22's E7018. Its R12 twin (`R12/reject_a_function_value_that_changes_log_called_in_a_walk_of_log_lines.em`)
  makes `note` inside `dump` for exactly this reason; this one does not.

### F-12. Messages that give no line to write (CONFIRMED; R18)

- **Blocks.**
  - "click after the loop", with no line: `V4/rules.md` 1013 (E3020 row, the `b.click()` message),
    `R12/reject_clicking_a_button_of_another_module_inside_a_loop_over_log.em` line 18,
    `R12/reject_clicking_a_log_button_inside_a_loop_over_log.em` line 23,
    `R12/reject_clicking_inside_a_loop_over_a_list_a_handler_changes.em` line 25 (and their `#$ help:
    click after the loop`). Their twins in `V4/area-D.md` 386-388 (AN-53) and `V4/area-E.md` 1594 (AN-76)
    say it: "click after the loop: move `b.click()` below the loop".
  - "read `total` after it, as line 13 does": `R15/reject_a_task_reads_total_before_the_join.em` line 12,
    and `V4/area-E.md` 434-436 (AN-71, "as line 6 does"). The line to write, or to remove, is not said
    (the refused task line is the one to drop or move below the `with` block).
  - A help line that repeats the program's own line: `R5/reject_cached_reading_a_field_other_threads_change.em`
    line 3 and `R5/reject_cached_reading_a_shares_field.em` line 3 (`#$ help: fn radius() reads verts ->
    f64:` / `... reads shape -> f64:`, already line 19 / 14), where the fix is to remove `@cached`
    (line 18 / 13), as the error line itself says.
  - Row 4's text on cases beyond row 4: `R8/reject_a_view_returned_out_of_a_read_lock.em` line 11 ("this
    points into the read-locked list ...") and `R8/reject_a_view_of_a_guarded_field_returned_out_of_its_lock.em`
    line 15, and `V4/area-B.md` 2195-2197 (AN-30's `RwLock` variant), give "return a copy" with no line.
    Row 4's own message is agreed word for word and stays; its variants are new text and could give the
    line (`return copy of g[0]` with `-> String`), as AN-30's third message does.
  - "test `if t is not a:` first": `R8/reject_a_link_that_may_be_the_ref_mut_parameters_object.em` line
    15 and `V4/rules.md` 1011, which do not say where (the R8 tests of the same kind say "above line 20
    and indent line 20 under it").
- **Test:** the message rule (the exact line to write).
- **Why:** each names what to do but not the line, where its twin or its neighbours do.

### F-13. E0113 calls the caller's variable by the parameter's name (CONFIRMED; R18, the program's own names)

- **Blocks.** "`mut n` on a parameter: to change the caller's `n` through `n`, write `ref mut n: int`, and
  call it `bump(ref mut a)`": `R4/reject_mut_alone_on_a_parameter.em` line 7, `V4/area-B.md` 1819-1820
  (AN-27), 719-720 (AN-20, "the caller's `c`", whose variable is `counter`); the template is
  `V4/rules.md` 1004 ("to change the caller's `b` through `b`").
- **Test:** the message rule (what was refused, in the program's own names).
- **Why:** the caller has no `n` (or `c`); its variable is `a` (or `counter`), which the same message
  names a few words later. "to change the caller's number through `n`" (or "`a`, through `n`") says it.

### F-14. A run-time stop with no text (CONFIRMED; worklist section 3)

- **Blocks.** `R15/run_fail_c_calls_a_once_callback_twice.em` line 4: `#$ panics: C called the callback
  registered on line 29`, a prefix only; its comment says `V4/rules.md` section 2 gives no text for this
  stop (a `callback(once)` function called a second time by C).
- **Test:** every stop says the rules' text in the program's names (worklist section 3).
- **Why:** the check exists (R12, R15: the trampoline finds the slot ended) and the test pins it, but no
  text says what happened ("C called the callback registered on line 29 a second time, but its contract
  says `callback(once)`" would follow the other two trampoline texts). For the lead or the owner.

### F-15. AN-70's prose says the override no longer calls `min`; its code calls `min` (CONFIRMED)

- **Blocks.** `V4/area-D.md` 2754-2756 ("The override's body no longer calls `min`, which has no row
  ... it does the same") against 2747 (`this.angle = min(this.angle + 90.0 * dt, 90.0)`). Carried from
  `V3/area-D.md` 2588 and 2581; the lead settled `min` as a prelude function (`V3/area-D.md` 2639), and
  the code went back to `min` while the sentence stayed.
- **Test:** an answer says what its program does.
- **Why:** a reader is told the opposite of the line above it.

### F-16. A local named `shared` (minor; readability)

- **Blocks.** `R9/run_two_shares_mut_owners_change_one_counter_with_no_run_time_check.em` lines 14 to 18:
  `shared = shares mut Counter(value=1)`, `alias = shares mut shared`.
- **Test:** no old spelling left (verdict 19 renamed `shared` to `shares`).
- **Why:** not a keyword now, but `shares mut shared` reads as the old and the new word side by side; a
  name such as `counter` says what it is.

## 4. Verdicts 53 to 82, where each was looked for

| verdict | applied in the NEW blocks and the changed or added tests |
|---|---|
| 53, 54 | yes: E3031 tests in R4, R5, R8, R17 with their allowed neighbours; area A, B, C and D state why each of their calls is not refused; open: F-06 (verdict 28's walk), F-09 (`ref mut`) |
| 55 | yes: `fn() with owns players changes Player: players.clear()` (R10, R12), AN-47's `job` |
| 56 | yes: `R11/run_a_link_opened_during_its_targets_drop_finds_it_ended.em`, `R11/run_a_childs_drop_finds_its_parent_ended.em`, AN-62 |
| 57 | yes: R1, R2, R5, R8 tests (`spawn_one`, `swap_in`, `reset`, `enemies.clear()` in a walk of `LOG`), AN-04's note |
| 58 | yes: `R1/reject_mem_forget_is_not_a_word_of_ember.em`; no `mem.forget` in any NEW block |
| 59 | yes: both `run_fail_too_many_...` tests with section 2's texts; the costs in areas C, D, E |
| 60 | stated (R16, AN-79); nothing to apply in code |
| 61 | yes: R12 and R15 `thread_shared fn()` tests; AN-06's note; open: area E's item 4 (`thread.spawn`'s parameter type) |
| 62 | yes: `R15/run_a_kept_callback_that_ends_its_own_token_then_uses_its_state.em`, AN-78's rule |
| 63, 64 | yes in every function that changes through a link (`linkscan.py`: no unnamed change); open: F-04, F-05 |
| 65 | yes: every task and thread inside a loop takes `with copy of` its loop variable (AN-71, AN-72, AN-74, AN-76; R15 tests); the refusals give it |
| 66, 67 | yes: `R13/accept_exists_as_copy_of_gives_a_value_of_your_own.em`, `R13/accept_exists_as_then_copy_of_on_its_own_line.em`, `R2/reject_a_change_of_a_plain_opened_name.em` |
| 68 | yes: `copy of` on `@derive(Copy)` and number types (AN-10, AN-13, AN-44, AN-50, AN-69) |
| 69 | yes: every `drop` writes `fn drop():` or `fn drop() changes X:`; `R5/reject_changes_this_on_a_drop*.em` |
| 70 | yes: `blend(const ref mut d, ...)` (R4 tests, AN-11, AN-14) |
| 71 | yes: `each_name(fn(owns s) => ...)` (AN-56, R12 tests) |
| 72 | yes: map keys, sets and a string's characters (R8 tests, AN-12) |
| 73 | yes in most messages; failures F-01, F-02, F-04, F-06, F-10 |
| 74 | yes for `owns` fields (R3 tests, AN-40, AN-39's table); open for an `owns` inside another type: F-03 |
| 75 | yes: `count_hit = fn() changes hits:` (R12), AN-73, AN-74's `changes stats`; see F-07, F-08 |
| 76 | yes: `widgets.em`, `log_widgets.em`, `logbook.em` and their tests; AN-53 |
| 77 | yes: `R15/run_tasks_add_to_a_field_of_an_object_they_reach.em`, `R9/run_two_tasks_take_hp_off_one_player_through_two_sharedref_links.em`, AN-74; see F-10 |
| 78 | yes: R2 tests (a field handed over, the owner used whole, a class field with `mem.take`), AN-35 |
| 79 | yes: `R3/run_copy_of_a_struct_copies_the_links_it_holds.em`, `R11/run_a_copy_of_a_struct_holding_links_adds_an_owner_and_a_link.em` |
| 80 | yes: `return fn() with owns t:` (R12 tests), AN-53 part 5; see F-02 |
| 81 | yes: `R7/accept_a_push_keeps_a_kept_link_to_an_object_in_its_own_block.em` and its refused inline twin, AN-52 |
| 82 | yes: R10 and R12 tests (E3060 after a call that may end the player), AN-51, AN-62, AN-65; see F-01 |

## 5. The agreed rows

| row | where | as agreed |
|---|---|---|
| 3 | `fn note(msg: String) changes log:` (AN-17; `R5/accept_a_first_line_in_its_order.em` with `reads frame` before it) | yes |
| 4 | AN-30's `return ref g[0]` refusal, `R8/reject_a_view_returned_out_of_a_lock.em` | yes, word for word |
| 5 | AN-08, AN-09 (`word = line.to_lower().trim()`; "return `String(...)` to hand back your own copy") | yes |
| 6 | AN-53 (`b.on_click(fn() changes score: score += 1)`, `fn() with copy of score:`), `R12/accept_row_6_handler_changes_the_real_score.em` | yes |
| 7 | `fn load_assets() reads renderer -> Array[String]:` (AN-31) | yes |
| 8 | `State.Walking(ref mut path):` and its two messages (AN-37, AN-38, `R2/reject_mut_alone_on_a_pattern_name.em`) | yes |
| 9 | `snapshot = copy of items`, `snapshot = items`, both ways out named (AN-34, AN-44, `R1/reject_a_list_used_after_it_was_handed_over.em`) | yes; AN-34's message also gives a first line for the `shares` way out (F-04) |

## 6. For the owner

Each point is already in an area file's or the rule writer's "For the owner" list; this check found
that the files disagree on it, which is why it is repeated here with the lines.

1. **`with owns` on an `int`** (F-02): verdict 80's own `return fn() with owns n: n += 1` against E2085
   ("write `with copy of score`"). Which word does a function that keeps an `int` it outlives write?
2. **`owns` on a making line inside an `Option`, a list or a local** (F-03): `next=Some(owns
   Link(value=1))` (two R15 tests) or `next=Some(Link(value=1))` (AN-77, AN-41, R3, R14 tests)?
3. **The first owner of a shared value is a link?** (F-04): if yes, row 9's message, in the rules and in
   `R1/reject_a_list_used_after_it_was_handed_over.em`, needs AN-34's `fn main() changes Array[int]:`; if
   no, four tests and AN-34 name a change the rules do not ask for.
4. **`changes target` or `changes target, Player`** for a method changing a player through its own link
   field (F-05).
5. **R4's call check and verdict 28's walk** (F-06): exempt a call through a function value, or change the
   walk?
6. **An `Atomic` inside an object reached through a link** (F-07): `changes Counter` (verdict 61's
   example) or `changes Atomic[int]` (R9's E7007 way out)?
7. **E7007's atomic way out for a task that reads first** (F-10): drop it, as AN-74 does, or keep it as
   the rules and the R15 tests do?
8. **A once callback called twice by C** (F-14): its stop text.

## 7. Files

- `tasks/96/redesign/run4/v4/readability.md` (this file).
- Under `target/redesign6/13/`: `tdiff.py`, `added.txt`, `changed.txt`, `removed.txt`, `review_list.txt`,
  `tests_R1.txt` .. `tests_R17.txt`; `extract.py`, `blocks.txt`, `blocks.json`, `new_area-A.md` ..
  `new_area-E.md`, `compact_A.txt` .. `compact_E.txt`; `scan.py`, `scan2.py`, `scan3.py`,
  `addedlines.py`, `added_lines.json`, `scan_blocks.txt`, `scan_tests.txt`, `scan_tests_code.txt`,
  `scan_tests_spans.txt`, `scan_spans.txt`, `scan_spans_added.txt`; `linkscan.py`, `linkscan.txt`;
  `refs.txt`, `names.txt`; `notes.md` (working notes).
