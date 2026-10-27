# Verdict and readability check of run 4, step 4, v5 (agent 09)

Agent 09 (verdict and readability checker), run 4, step 4, phase 3. This file checks every file of `V5`
(the rules, the vocabulary, the five answer files and every conformance test) against `inputs/JOB.md`
verdicts 1 to 97 (27 replaces 25, 26 replaces 1, 95 reverses 90, 86 voids the spellings it names), with
the corrected records of verdicts 53 and 58: no old spelling left, every verdict applied, the owner's
test on every new or changed block, and the message rule and verdict 73 on every message. It fixes
nothing. `V4` means `tasks/96/redesign/run4/v4/`, `V5` means `tasks/96/redesign/run4/v5/`; "R1" .. "R18"
are the rules of `V5/rules.md`; "verdict N" is item N of `inputs/JOB.md`; "row N" is the owner's agreed
row N; "worklist" is `V5/inputs/worklist.md`. Line numbers are lines of the file named, as it stood when
this check ran (27 October 2026, from 09:12, after agents 01 to 08 had written their files).

No compiler change, no git command, no Ember program compiled or run. Power was on mains at the start and
at the end (`PowerLineStatus` printed `Online`).

## 0. Terms

- **NEW block**: code an answer file proposes under the v5 rules: any fenced `ember` block outside an
  answer's parts 2 (OLD) and 5 ("means exactly"), plus AN-53's part 5 block (`V5/area-D.md` 297-308,
  a NEW program in v5 spelling). OLD code and "means exactly" programs are today's Ember and are not
  judged.
- **message**: the text of a refusal, a warning or a run-time stop: an answer's part 3 or 6 (inline or
  as a `text` block), a row of `V5/rules.md` section 3, and a test's `#$ error[...]`, `#$ warning[...]`,
  `#$ help:`, `#$ note:` and `#$ panics:` lines.
- **old spelling**: a form a verdict removed (for this step: a `changes` or `reads` clause, `captured`,
  `captured...`, a change list, `changelist`, `with` before what a function keeps, `mem.forget`,
  `ForeignBox`, E2085's refusal of `owns` on a number, a collection changed through its own name inside
  its own loop, `into_raw()` outside `unsafe:`; from earlier steps: `holds`, `cast x if`/`when`,
  `.copied()`, `owned`, `shared` as an owner word, `copied`, `self`, `weak`, `set by`, `Some(...)` in an
  `if`/`while`, type parameters after a name, `as?`/`as!`, `Box[`, `RefCell[`, `Weak[`, `Shared[`,
  `SyncShared[`, `Mut[`, `.upgrade(`, `.clone()`, run 4's `if p = x:`, a plain `MutSpan` that writes,
  `global`, the removed attributes, `str`/`cstr`). A **hit** is an old spelling found; it **fails** only
  where it is presented as current Ember (not in a `reject_` test's refused line, a message that refuses
  it, a "today"/OLD column, or prose that says what was removed).
- **the owner's test, the message rule, verdict 73**: as `inputs/brief.md` states them (R18).
- **CONFIRMED**: settled by the text, no reading needed. **PLAUSIBLE**: rests on a reading.

## 1. What was checked, and how

Read in full: `inputs/brief.md`, `inputs/JOB.md`, `inputs/worklist.md`, `inputs/duties.md` (section 09),
`V5/rules.md`, the head of `V5/changes.md` (sections 1 and 2), and every one of the 484 tests added or
changed in this step. Read by part: parts 3, 4 and 6 of all 84 answers (the NEW blocks, every
owner's-test table, every refusal and its message), each answer's "v5" line, and each area file's "What
step 4 changed" table and "For the owner" section. Read by row: the vocabulary's rows for every word
verdicts 83 to 97 touch (`with`, `for`, `owns`, `copy of`, `thread.spawn`, `Scope`, `into_raw()`,
`mem.forget`, `ForeignBox`, `changes`, `captured`, the change list, `reads`, `this`, `fn`, the iterator
row, the agreed rows of section 1).

Scripts, all under `target/redesign7/09/`:
- `extract.py`: labels every line of the five area files by answer, part and block kind (`labels.json`,
  `blocks.json`, `blocks.txt`): 491 fenced blocks; the only changed OLD-part block is AN-53's part 5
  program, which is NEW code (above); every other OLD and "means exactly" block is v4's, byte for byte.
- `scan.py`: every NEW block (code and comment spans), every backticked span of the answers' prose and
  text blocks, of the rules and of the vocabulary, and every line of all 840 conformance files (code,
  `#$` lines, `##` comments) searched for the old spellings of section 0 (`scan.txt`, `scan.json`). Each
  hit was read in its sentence; the hits on lines this step changed (compared with `V4` by `difflib`)
  were read one by one.
- `loopscan.py`: every `for` loop in every test and NEW block whose body changes the walked collection
  through its own name (element write, value under a key, `push`, `insert`, `remove`, `clear` and the
  rest): every hit is a `reject_` test's marked line or an answer's part 6.
- `ownsscan.py`: every value made for an `owns` field, an `Option[owns T]` or an `Array[owns ...]`
  without `owns` on the making line (verdicts 74, 85, 97): every hit is a `reject_` test's marked line.
- `msgscan.py`: every compile-fail test has one error mark, a code that is live in `V5/rules.md`
  section 4, and a message (with its help) that gives a line to write; every run-fail test has its stop
  text; no message asks for a word on a first line.
- `added.txt`, `changed.txt`, `removed.txt`, `same.txt`: the tests against `V4/conformance` (134 added,
  350 changed, 130 removed, 356 the same); `read_R*.txt`: the 484 added and changed tests as read;
  `ans346_*.txt`: parts 3, 4 and 6 of every answer as read.

## 2. Results at a glance

| test | result |
|---|---|
| no old spelling left as current Ember | **passes**: 0 left. Every hit is a refused line of a `reject_` test (`R1/reject_mem_forget_is_not_a_word_of_ember.em`, `R3/reject_foreign_box_is_now_owns.em`, `R3/reject_with_before_a_capture_list.em`, `R5/reject_changes_written_after_a_functions_brackets.em`, `R5/reject_reads_written_after_a_methods_brackets.em`, `R12/reject_a_function_type_written_with_changes_log.em`, `R12/reject_row_6_handler_written_with_changes_score.em`, `R12/reject_an_arrow_lambda_written_with_changes_removed.em`, `R12/reject_with_before_a_lambdas_captures.em`, `R16/reject_into_raw_outside_unsafe.em`, and the earlier steps' removed-word tests), a message that refuses it, a "today" column, or prose that says what was removed. The removed codes E2085, E2105, E2107, E2113, E2122 appear only in such prose and in comments of tests that replace them. One stale meaning of a removed verdict stays (F-02) |
| `into_raw()` only inside `unsafe:` (verdict 91) | **passes**: its two code uses are `R16/accept_into_raw_inside_unsafe_gives_c_the_address.em` line 17 (inside `unsafe:`) and `R16/reject_into_raw_outside_unsafe.em` line 18 (the refused line) |
| a collection changed through its own name inside its own loop (verdict 95) | **passes**: none outside `reject_` tests' marked lines and answers' part 6 (AN-12, `V5/area-A.md` 2694-2719) |
| `owns` on every making line for an `owns` place (verdicts 74, 85, 97) | **passes**: none missing outside `reject_` tests' marked lines |
| verdicts 83 to 97 and the corrected records of 53 and 58 applied | **yes, but for one point of verdict 88** (F-01, section 5) |
| verdicts 1 to 82 applied | **yes**, but one owner's-test cell keeps verdict 22's meaning, which verdict 86 voids (F-02) |
| the owner's test on every new or changed block | **passes**: every declared name of the NEW blocks and of the 484 tests has its word, or a stated default that never changes, aliases or copies silently; the tables match the code |
| the message rule | **3 failures** (F-03, F-04, F-05): a way out given without the lines it replaces, and two messages that give no line |
| verdict 73 | **passes** in every message read; the one CONFIRMED rule conflict (F-01) is in the rules, not in a message |
| agreed rows 3 to 9 | **as agreed**, in the verdicts' spellings (section 6) |
| claims with a source | **2 failures** (F-06, F-07): four tests added in this step state sanitizer results nothing produced; ten changed tests cite `V4`'s rules and vocabulary as their source |

## 3. Failures

Most severe first. Each with where it is, the test it fails, and why.

### F-01. R4 says two ways what happens to two `ref mut` elements of one list at unknown indexes (CONFIRMED; verdict 88; for the owner)

- **Where.** `V5/rules.md` 305-306 (R4, overlap rule, exception 1): "Parts whose positions are not known
  when compiling (`ref mut log[i]` beside `ref mut log[j]`, or beside `log`) stay refused (E3022)",
  taken from verdict 88's record ("items whose positions are not known (`ref mut log[i]` beside
  `ref mut log[j]`, or beside `log` itself) stay refused") and from the worklist's row 88 ("`reject_`:
  `ref mut xs[i]` beside `ref mut xs[j]` stays"). Against it, in the same rules: R4 292-294 ("Where the
  two differ only by an index of one list (`bodies[i]`, `bodies[j]`), one compare before the call stops
  the program if they are equal"), R8 (f) 512-513, section 2's rows 972 and 1003 (the index compare and
  its stop text), section 3's E3022 row 1060 ("Where not proved, a stop while running"), AN-03
  (`V5/area-A.md` 601-615, `step`'s `collide(ref mut bodies[i], ref mut bodies[j])` with "one compare:
  i != j"; 671-681, which quotes exception 1's "stay refused" as if it covered the compare), and the
  test `R4/run_the_same_body_given_twice_stops_the_program.em`, which compiles that very shape.
- **Test:** every verdict applied, one meaning per rule; the worklist's required `reject_` test for
  `ref mut xs[i]` beside `ref mut xs[j]` does not exist (the nearest, `R4/reject_an_element_given_as_ref_mut_beside_its_whole_list.em`,
  is the "beside `log`" half, which both readings refuse).
- **Why:** one program, two outcomes. `fn step(ref mut bodies: Array[Body], i: int, j: int):` with
  `collide(ref mut bodies[i], ref mut bodies[j])` compiles with one compare by R4 292-294 and is refused
  by R4 305-306. `V5/changes.md` records verdict 88 (line 33) but settles nothing about the compare. The
  verdict's word "stay" presupposes the line was already refused, which under v4 it was not (it was
  compared while running), so the record does not settle which the owner meant. Section 5 gives the two
  readings.

### F-02. A stated default keeps verdict 22's "a plain `fn()` changes nothing at all" (CONFIRMED; verdict 86)

- **Where.** `V5/area-C.md` 1633 (AN-47's owner's-test table, `task`): "none (default: a plain `fn()`
  changes nothing at all, R12, verdict 22)".
- **Test:** verdicts 1 to 82 as verdict 86 leaves them; the owner's test (a stated default must be
  true).
- **Why:** verdict 86 voids verdict 22's spelling ("the ones only about how to write it (22's `changes
  captured` ...) are void") and R12 now says the opposite: "`fn()`, `fn(Node)`, `fn(owns String) -> int`,
  `thread_shared fn()` and `owns dyn I` say nothing about what their functions or methods change"
  (`V5/rules.md` 662-664). Row 6's `fn(): score += 1` is a plain `fn()` that changes `score`. The cell's
  conclusion holds for `task` (it captures nothing and changes nothing, by its body), but the default it
  states is gone. No other file states it (searched in all of `V5`).

### F-03. E7008's way out is one line, not the lines it replaces (CONFIRMED; the message rule, R18)

- **Where.** `V5/rules.md` 1105 (section 3, E7008): "take the lock inside the task: `with g =
  m.lock():`"; its twin in `V5/area-E.md` 512-514 (AN-71 part 6), word for word.
- **Test:** R18: "the exact line to write ... where it replaces several lines the message names every one
  it replaces"; F-12 of step 3's check, which the worklist sends to the rules and the answers.
- **Why:** the program refused (`R15/reject_a_lock_guard_sent_to_a_task.em`) needs a helper and a
  changed `spawn` line, not only a `with` line; the test's own message gives them ("write `fn
  add_one(m: Mutex[Array[int]]):` with the body `with g = m.lock():` and, inside it, `g.push(1)`, above
  line 11, and `scope.spawn(fn(): add_one(m))` in place of line 16"), so the rules' row and AN-71 lag
  their twin.

### F-04. E7010 on a scattered write gives no line (CONFIRMED; the message rule)

- **Where.** `V5/conformance/R15/reject_a_parallel_loop_writing_at_another_index.em` lines 3 and 8
  (unchanged since `V4`): error "the parallel loop writes `xs`, and reaches it here at an index that is
  not `i` plus a constant", help "index it by the loop's own index, `i + k`, or make it a reduction".
- **Test:** the message rule (the exact line to write); worklist F-12 ("every message gives the exact
  line to write ... [rules, answers, tests]").
- **Why:** neither way out is a line of this program: `xs[idx[i]] = i` has no `i + k` form, and "make
  it a reduction" names no line. A way out that compiles here exists (remove `@parallel`, line 7, which
  makes the loop run on one thread), and the rules' own E7010 row (`V5/rules.md` 1106) covers only the
  `shares mut` case, so this message has no section-3 form to follow.

### F-05. Two more ways out given without their line (PLAUSIBLE; the message rule)

- **E2100's second way out**, "or in `load_assets` stop reading `this.assets` (its line 14)":
  `V5/rules.md` 397-400 and 1046, `V5/area-B.md` 2321-2325 (AN-31),
  `R5/reject_init_calls_a_method_whose_body_reads_a_field_not_yet_set.em` line 3. "Stop reading" names
  no line to write; the first way out (`this.assets = []` above the call) does, so the message is not
  without a fix, but its second half is advice, not a line.
- **AN-55's third way out**, "or start it inside `with scope = thread.scope():`, which waits for it"
  (`V5/area-D.md` 603-604): the `with` line alone does not make the program (the `spawn` line becomes
  `scope.spawn(...)` and `h.join()` goes); the first two ways out give whole lines.
- **Test:** the message rule. PLAUSIBLE: R18 can be read as asking only that the message give one line
  that compiles, which both do.

### F-06. Four tests added in this step state sanitizer results that nothing produced (CONFIRMED; the brief's "every claim names its source")

- **Where.** `R4/run_one_variable_given_twice_as_ref_mut_is_one_variable_in_the_call.em` line 8 and
  `R4/run_a_module_list_given_to_a_ref_mut_parameter_its_function_also_names.em` line 11 ("Verified by
  its output and a clean AddressSanitizer run"); `R3/run_a_new_value_made_later_for_an_owns_field_says_owns_and_ends_the_old_one.em`
  lines 7-8 ("Verified by its output and a clean AddressSanitizer and leak run (each block freed
  once)"); `R10/accept_a_view_of_a_node_used_before_the_function_value_that_owned_it_is_replaced.em`
  line 9 ("Checked under AddressSanitizer: no report.").
- **Test:** `inputs/brief.md`, Machine limits: "Nothing is compiled or run (the new spellings are not
  built) ... Every claim names its source."
- **Why:** the four files are new in this step (`target/redesign7/09/added.txt`), in spellings no
  compiler builds (verdicts 88, 97 and hole 3 part 2), and no step-4 agent ran anything; the sentences
  read as results. Older tests that carry such sentences cite run 3's C builds, which did run.

### F-07. Ten changed tests cite `V4`'s rules and vocabulary as their source (CONFIRMED, minor)

- **Where.** `R2/reject_mut_alone_on_a_pattern_name.em` 5-6; in `R3/`:
  `reject_a_value_from_another_kind_of_arena_given_to_an_arena_place.em` 4,
  `reject_an_array_with_a_manager_is_array_using.em` 4-5, `reject_box_with_a_manager_is_owns_using.em`
  4-5, `reject_clone_is_not_a_word_of_ember.em` 4, `reject_using_after_a_type_with_no_heap_memory.em` 5,
  `run_a_mesh_made_using_an_arena_takes_its_block_from_that_arena.em` 5,
  `run_a_value_ends_in_the_arena_instance_it_came_from.em` 5,
  `run_a_value_stored_into_an_owns_place_gets_its_own_block.em` 4,
  `run_copy_of_a_struct_copies_the_links_it_holds.em` 7.
- **Test:** every claim names its source (the rules a v5 test checks are `V5/rules.md`).
- **Why:** each was edited in this step and still quotes `V4/rules.md` section 2 or 3, or a
  `V4/vocabulary.md` row; the quoted text still exists in v5 (in reworded rows), so nothing is wrong
  but the citation.

Not a failure, noted: AN-16 says every refusal of area A carries a code of `V5/rules.md` section 4
(`V5/area-A.md` 3481-3482), and AN-14 gives today's E3095 (3132), which section 4 does not list; the
area says so itself (3619-3620) and leaves it to the rule writer.

## 4. Verdicts 83 to 97, and the corrected records of 53 and 58

| verdict | rules and vocabulary | answers | tests | applied? |
|---|---|---|---|---|
| 53 (record corrected) | R4 319-323, 339-342 (whole program; only C unseen); section 0 86-89; E3031's storing-line message (1062) | AN-62 (b): `this.on_visit(p)` compiles; storing `prune` that reads its node is E3031 at the store (`V5/area-D.md` 1670-1674) | `R11/reject_a_visit_function_that_reads_its_node_after_ending_it.em` (E3031 at line 53, the storing line); the two walk run tests stay `run-pass` | yes |
| 58 (record corrected) | R1 115-117, R16 915-923 (`Retained` gives no object away); vocabulary `mem.forget`, `ForeignBox`, `into_raw()` | AN-79 (`V5/area-E.md` 2139-2143) | `R1/reject_mem_forget_is_not_a_word_of_ember.em` | yes |
| 83 | R3 230-245; E0100 (1051, 1143); vocabulary `with` (146): blocks and imports only | every capture list (`fn() copy of score:`, `fn() owns players:`); AN-53's part 5 | `R3/reject_with_before_a_capture_list.em`, `R12/reject_with_before_a_lambdas_captures.em`; no `with` before a capture anywhere else | yes |
| 84 | R1 110-111, R3 186-200, R4 256-257, 284-285; E2085 removed (1213); E3040's number message (1035) | AN-43, AN-53 case 2, AN-54 (E3040 for `owns score`) | `R3/accept_a_number_taken_over_by_a_function_with_owns.em`, `R3/reject_a_number_used_after_a_function_took_it_over_with_owns.em`, `R4/accept_a_number_given_to_an_owns_parameter_or_a_copy_of_one.em`, `R4/reject_a_number_used_after_it_was_handed_to_an_owns_parameter.em`, `R12/accept_a_returned_function_owns_the_variable_it_keeps.em` | yes |
| 85 | R3 186-195; E2123 (1087, 1174) | AN-40 (`r.pools.push(owns IntPool(...))`), AN-41 (`left=Some(owns Tree(...))`), AN-66, AN-77 | `R3/reject_a_value_made_inside_some_for_an_option_of_owns_without_owns.em`, `R3/reject_a_value_pushed_into_a_list_of_owns_values_without_owns.em`, their `accept_`/`run_` neighbours, and every list of `owns dyn` values in R14 | yes |
| 86 | R5 353-410 rewritten; R12 662-680; codes E2105, E2107, E2113, E2122 removed; no message asks for a first-line word (R18 953-954); vocabulary rows `changes`, `captured`, change list, `changelist`, `reads` say "removed" | every answer: no clause; rows 3 and 7 as `fn note(msg: String):` and `fn load_assets() -> Array[String]:`; messages name the chain | the 72 tests of the spelling report turned into chain refusals, `accept_` or `run_` tests (130 v4 tests removed, nearly all of them tests of a written clause; whether each removal is listed in its table is agent 10's and the test updaters' check, not this one's); `R5/reject_changes_written_after_a_functions_brackets.em`, `R5/reject_reads_written_after_a_methods_brackets.em` | yes, but F-02 |
| 87 | R4 332-334; E3031's `rename` message | none needed | `R4/reject_a_ref_mut_argument_into_what_the_callee_clears.em`, `R4/accept_a_ref_mut_argument_of_your_own_given_to_a_function_that_clears_log.em` | yes |
| 88 | R4 297-306, 344-346; R8 (f) 509; section 2's "nothing" row | AN-03 part 6 | `R4/run_one_variable_given_twice_as_ref_mut_is_one_variable_in_the_call.em` (prints 2), `R4/run_a_module_list_given_to_a_ref_mut_parameter_its_function_also_names.em` (prints 1), `R4/reject_an_element_given_as_ref_mut_beside_its_whole_list.em` | **in part**: the whole-variable case is applied; the unknown-indexes half contradicts the rest of R4 (F-01) |
| 89 | R4 307-313; R15 879-880 | AN-75 (`V5/area-E.md` 1202-1205) | `R4/accept_a_log_changed_only_under_its_lock_reaches_one_call_through_two_names.em`, `R15/accept_one_call_reaches_a_lock_guarded_log_through_two_names.em`; E7018 stays in `R15/reject_a_walk_under_the_lock_across_a_function_value_that_changes_log.em` | yes |
| 91 | R1 115-117, R16 915-923; E3100 (1110, 1192); vocabulary `into_raw()`, `ForeignBox` | AN-79 | `R16/reject_into_raw_outside_unsafe.em`, `R16/accept_into_raw_inside_unsafe_gives_c_the_address.em`, `R16/accept_a_c_function_declared_owned_takes_the_widget_with_no_word.em`, `R16/reject_a_widget_used_after_it_was_handed_to_c.em` | yes |
| 92 | R15 853-858; section 2 row 990 | AN-71 (`V5/area-E.md` 516-532) | `R15/run_a_float_sum_added_more_than_once_per_task_is_a_reduction.em` (prints `0.6`), replacing v4's refusal | yes |
| 93 | R3 230-233; R12 759-760; R15 861-871; E3026's two messages (1095) | AN-71, AN-72, AN-74 (`ref t`) | `R15/accept_a_scoped_task_looks_at_the_loops_text_with_ref.em`, `R15/reject_a_thread_given_the_loops_text_with_ref.em`, `R15/reject_a_task_uses_the_loop_name_without_a_copy.em` (both words) | yes |
| 94 | R15 840-845; E7015 (1103, 1203); vocabulary `thread.spawn`, `Scope` | AN-55, AN-71, AN-75 | `R15/reject_a_thread_captures_a_one_thread_owner.em` and `R15/reject_a_thread_takes_a_shares_mut_bus_to_store_a_handler.em` now expect E7015 | yes |
| 95 | R8 (c) 477-494; E3020's two messages (1065); vocabulary `for` | AN-12 (`V5/area-A.md` 2705-2719) | `R8/reject_a_write_of_an_element_while_a_for_loop_walks_the_list.em`, `R8/reject_a_value_under_a_key_written_while_a_loop_walks_the_maps_keys.em`, `R8/accept_the_loops_own_handle_writes_the_element_it_is_on.em`, `R8/accept_the_loops_own_handle_changes_the_value_under_each_key.em` | yes |
| 96 | section 2, 1013-1018 | none | `R9/run_fail_too_many_owners_of_one_player_stop_the_program.em` and `R11/run_fail_too_many_weakref_links_to_one_player_stop_the_program.em` (`#$ count_bits: 8`, stop texts with `(255)`; the round counts check: 1 + 254 owners, then the 255th add stops; 255 links, then the 256th), `R9/run_real_programs_keep_32_bit_counts.em` | yes |
| 97 | R3 193, 197-200; E0109 (1089, 1145); E2123 (1087) | none reassigns | `R3/reject_a_new_value_made_later_for_an_owns_field_without_owns.em`, `R3/run_a_new_value_made_later_for_an_owns_field_says_owns_and_ends_the_old_one.em`, `R3/accept_an_existing_mesh_handed_to_an_owns_field_with_no_word.em` | yes |

Step 3's readability failures (`V4`'s file), where each stands now: F-01 applied (the rules' E3060
messages open the link as `q`: `V5/rules.md` 572-574, 1069; AN-62, AN-65); F-02, F-03 settled by verdicts
84 and 85; F-04, F-05, F-07, F-08 void by verdict 86; F-06 settled by verdict 53's corrected record;
F-09 by 87; F-10 applied (E7007 gives only the lock: `V5/rules.md` 1104, AN-74,
`R15/reject_a_task_reads_a_field_another_task_adds_to.em`); F-11 by 89; F-12 applied but for F-03 to
F-05 above; F-13 applied (`R4/reject_mut_alone_on_a_parameter.em`: "to change the caller's `a`");
F-14 applied (`V5/rules.md` 1009, `R15/run_fail_c_calls_a_once_callback_twice.em`); F-15 applied
(`V5/area-D.md` 2680-2683); F-16 applied (`alias` in
`R9/run_two_shares_mut_owners_change_one_counter_with_no_run_time_check.em`).

## 5. For the owner

One point, F-01, which no verdict settles in the text the rules can follow and which changes whether a
program compiles. The rule is R4's overlap rule (`V5/rules.md` 286-306); the line is a call that gives
two elements of one list, at indexes known only when running, to two `ref mut` parameters:

```ember
fn collide(ref mut a: Body, ref mut b: Body):
    t = a.vel
    a.vel = b.vel
    b.vel = t

fn step(ref mut bodies: Array[Body], i: int, j: int):
    collide(ref mut bodies[i], ref mut bodies[j])
```

- **Reading (a), run 3's and v4's, kept in R4 292-294, R8 (f), sections 2 and 3, AN-03 and the test
  `R4/run_the_same_body_given_twice_stops_the_program.em`:** it compiles; one compare of `i` and `j`
  runs before the call, and when they are equal the program stops with "`a` and `b` name the same
  element of `bodies` (index 1); `collide` needs two different bodies". Where the compiler proves them
  apart (`j` from `i + 1..`, inside `if i != j:`) nothing runs.
- **Reading (b), verdict 88's record as written ("`ref mut log[i]` beside `ref mut log[j]` ... stay
  refused, since they may lead to the `LOG[0]` problem"), R4 305-306 and the worklist:** it is refused
  when compiling, E3022, until the program proves the two apart itself:

  ```ember
  fn step(ref mut bodies: Array[Body], i: int, j: int):
      if i != j:
          collide(ref mut bodies[i], ref mut bodies[j])
  ```

Both are as safe as Rust (Rust refuses (a)'s line, E0499, and offers `get_disjoint_mut`, which checks
while running as (a) does). Your words, "log[i], log[j] still remains an error as we don't know the
values of i and j", fit (b) if "error" means "refused when compiling" and (a) if a stop while running
counts; the call that prompted them was the `LOG[0]` case, where `rename` also clears `LOG` by name, and
that case is refused under both readings (E3031, verdict 87).

## 6. Agreed rows 3 to 9

| row | where it stands in v5 | as agreed? |
|---|---|---|
| 3 | `fn note(msg: String):` (AN-17, `V5/area-B.md` 259-260; vocabulary 62): a loop over `this.log` that calls it is refused (E3020, chain message, `R5/reject_a_method_that_adds_to_the_log_called_in_a_loop_over_the_log.em`), a loop over `this.players` is not (`R5/accept_tick_calls_note_inside_a_loop_over_players.em`) | yes; `self` and `changes log` gone (owner 2026-10-25, verdict 86) |
| 4 | "this points into the locked list, which can change once the lock ends; return a copy, or do the work inside the `with` block" (`V5/rules.md` 1072; AN-30, `V5/area-B.md` 2193-2195) | yes, word for word |
| 5 | `word = line.to_lower().trim()` kept until `word`'s last use; returning it refused with "return `String(...)` to hand back your own copy" (`V5/rules.md` 1071; AN-08, AN-09) | yes |
| 6 | `b.on_click(fn(): score += 1)` changes `game`'s real `score`; the copy is `fn() copy of score:` (AN-53; `R12/accept_row_6_handler_changes_the_real_score.em`, `R12/run_a_named_copy_is_the_handlers_own_state.em`) | yes, in verdicts 12, 83 and 86's spelling |
| 7 | `fn load_assets() -> Array[String]:`, which `init` may call once `renderer` is set (AN-31; `R5/accept_init_calls_a_method_once_the_fields_its_body_reads_are_set.em`) | yes; `self` and `reads renderer` gone |
| 8 | `State.Walking(ref mut path):`; plain `path` then `path.pop()` and `State.Walking(mut path):` get "to change that list through `path`, write `State.Walking(ref mut path):`" (AN-38, `V5/area-C.md` 656-666; `R2/reject_a_change_through_a_plain_pattern_name.em`, `R2/reject_mut_alone_on_a_pattern_name.em`) | yes, word for word |
| 9 | `snapshot = copy of items` copies; `snapshot = items` hands over; E3040 names `copy of` and `shares` (AN-34, `V5/area-C.md` 148-200; `V5/rules.md` 1035) | yes, in verdicts 12 and 19's spelling |
