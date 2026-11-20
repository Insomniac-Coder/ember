# M9 ahead: tests and spec questions for threads, written before the builders

Written from `docs/spec-source/Ember_v0.9.20_Hardened_14.md` alone (`355cf49f`, tag `spec-h14`) by the
Opus deputy D2 on 2026-11-20, on `deputy` after `main` (`3656833c`). M9 is MILESTONES.md's "Threads:
`thread_shared` values, spawn, scoped tasks, reductions, locks, module variables": A5-11 to A5-20,
A1-17's second half, A6-13, A6-14 and A9-9, with verdicts 104 (after `h.join()` the parent may use
what the task used), 105 (a split sum stops where a task's own total overflows) and 114 (an object
handed to another thread leaves its `weakref` links behind). Nothing was built: every expectation
is the spec's words and codes. The files fail until M9 is built; a built rule's directory moves to
`tests/conformance/<RULE>/`. No hardening is cut here.

**Skipped.** No rule of the milestone is credited (`built_for_0915`). Clauses already tested in
`tests/conformance` and not written again: THR-3's lock and guard forms, THR-5's `with` binding and
its every-exit join, THR-6's `@must_drop` places, THR-10's spawn, join and detach, THR-14's
`Atomic`, THR-15's bounds and disconnection, THR-16, JOB-1, JOB-5, PAR-2's `?` and outer
`continue`, PAR-4, PAR-5, WK-11's links to `thread_shared` values, STA-1's `E7002` on plain
statics, FFI-33's thread policies. Not written: A6-13 and A9-9 (a `thread_shared` class crossing to
C++ as a handle: a `--cpp` build and both backends, the FFI milestone's harness), A6-14's run-time
thread checks of FFI-33a/33b/53 (the runtime's, in every profile: a milestone test), THR-4 (the
debug lock-order report), THR-13 (the guarantee, which the files of the rules it lists test),
RC-4's points 1, 2, 4 and 5 beyond what a program can see (the counting itself), PAR-2a and PAR-2b.

**Conventions** (as the other banks). Each file starts with `# Clause <RULE>.<n>: "<the clause,
quoted>"`; one file per clause and outcome; a refusal's text is the spec's own with the file's line
numbers, else code only; line numbers come from markers. An expectation on its own line claims a
diagnostic anywhere in the file, used where the spec does not say on which line the refusal stands
(Q3). A case is one `#$ error[...]`, `#$ stdout:`, `#$ panics:` or `#$ not-help:` line. `[THR-7]`
(no diagnostic calls `thread_shared` "thread-safe") is the `not-help` of THR-1's E7014 file.

**The probe** (the release `ember.exe` of 2026-11-19, one file at a time; `static mut` read as
`static` in a copy). That build predates M10's std, so every file also shows the std's own
`Arena has no method named allocate`: set aside. Three faults were fixed: `copy of` a `StrView`
pushed into an `Array[String]` (THR-1's E7014 program takes a `String`); `weakref thread_shared x`
on a name already `thread_shared` (`E0109`, `[OWN-13]`: it is `weakref x`); a handle declared inside
a `with` block and used after it (the fix's handles are kept in a list declared before, as the
spec's own program does). Everything else is not built: capture words (E0900), `Lock` and `guarded
by` (`std.sync` has no `Lock` yet), the effect-based task check (today's borrow check of tasks),
reductions, E7007/E7012/E7013/E7014/E7015/E7017/E7018 in their 0.9.20 forms, `sharedref mut`, a
`@parallel` element walk and `Array` writes (E7010 today), `@parallel` on a `while` (E0108 today).
THR-3's and BRW-20's accepts check clean today; none was run.

**Totals.** 22 rule directories (THR-7 inside THR-1's), 35 clauses, 47 files (18 accept, 27
reject, 2 run-fail), 105 cases: 54 error expectations (33 with the spec's whole text, 2 in part, 19
code only), 48 output lines, 2 panics and 1 `not-help`.

## 0. Rules covered, and what else each needs

| rule | clauses | files | cases | also needs |
|---|---|---|---|---|
| THR-1 (and THR-7) | 3 | 4 | 8 | M8 (capture words) |
| THR-2 | 3 | 3 | 9 | M8; verdict 114's hand-over in the runtime |
| THR-10 | 1 | 2 | 2 | M8; Q1, Q2 |
| THR-17 | 2 | 3 | 7 | M8 |
| THR-11 | 3 | 4 | 8 | M6a's summaries (EFF-24); Q3 |
| THR-21 | 3 | 3 | 9 | THR-11's check; Q4 (the spec's inline task lines) |
| THR-18 | 3 | 3 | 10 | M6a (summaries: "needs lock held"); Q3 |
| THR-19 | 1 | 2 | 10 | THR-18; Q3 |
| THR-20 | 2 | 3 | 6 | M2 (`static mut`); M6a; Q3 |
| STA-1 | 1 | 1 | 2 | M2 (its thread clauses only) |
| FFI-33 | 1 | 1 | 1 | THR-20; Q3 |
| THR-3 | 1 | 2 | 5 | none |
| BRW-20 | 1 | 2 | 5 | THR-18 for the `guarded by` case |
| THR-15 | 1 | 2 | 4 | M5's A2-8 (`sharedref mut`) for the reject; M8 |
| JOB-2 | 1 | 2 | 4 | M8 |
| JOB-3 | 1 | 1 | 1 | THR-11; Q3 |
| PAR-1 | 1 | 2 | 2 | none |
| PAR-2 | 1 | 1 | 3 | Q3 |
| PAR-3 | 1 | 2 | 2 | none |
| TYP-46a | 1 | 1 | 1 | M10 (`using`, the `Allocator` facts) |
| RC-4 | 1 | 1 | 3 | the two-number count (verdict 101) |
| WK-12 | 2 | 2 | 3 | M8; Q5 |

## 1. Rules, clauses and tests

Message column: "spec" = the spec's text; "part" = claimed up to a point; "none" = code only.

| rule.clause | what it says | test | message |
|---|---|---|---|
| THR-1.1 | `thread_shared` on a value made otherwise: E7017 | `reject_thread_shared_on_a_value_made_otherwise` | spec |
| THR-1.2 | no owner changes a `thread_shared` value directly (E7014); its state changes through `Atomic`, locks, `Sender`, guarded fields | `reject_an_owner_changes_a_thread_shared_value_directly` (and THR-7's `not-help`); `accept_thread_shared_owners_on_several_threads` | spec |
| THR-1.3 | its type is `Send` and `ThreadShared`; a `thread_shared fn()` holds no one-thread owner: E7015 | `reject_what_a_thread_shared_value_may_not_reach` (`FrameStats`, `Bus`, `on_event`) | spec (three) |
| THR-2.1 | no one-thread owner or `weakref T` reaches another thread; links in a task only through `weakref thread_shared T`; no `weakref` of a `static` | `reject_a_one_thread_value_reaching_another_thread` | spec (two); none |
| THR-2.2 | an object handed whole to another thread leaves its links behind (verdict 114), all the way down | `accept_an_object_handed_to_another_thread_leaves_its_links_behind` (the spec's two cases) | — |
| THR-2.3 | a hand-over is a reshape: no view into it after (E3060) | `reject_a_view_used_after_a_hand_over` | none |
| THR-10.1 | `R` is `Send`; a spawn in a walk under the lock the thread needs is allowed | `reject_a_thread_whose_result_is_not_send` (Q1); `accept_a_thread_started_in_a_walk_under_the_lock_it_needs` (the spec's program) | none |
| THR-17.1 | an unscoped thread names its captures: E7004 | `reject_an_unscoped_thread_names_its_captures` | spec |
| THR-17.2 | the loop's own variable and a round's local take a word: E3026 for a task (both words), E3026 for `ref t` in a thread, E7004 for a thread | `reject_the_loops_own_variable_given_without_a_word`; `accept_the_words_a_thread_and_a_task_take` | spec (two); part |
| THR-11.1 | tasks are checked against each other until their join (E7007); a `Receiver` is one consumer; after a join the parent may use what the task used (verdict 104) | `reject_tasks_checked_against_each_other_until_their_join`; `accept_after_a_join_the_parent_may_use_what_the_task_used` | spec (Receiver); none |
| THR-11.2 | a task started in a walk of what it changes: E3020, not E7007 | `reject_a_task_started_in_a_walk_of_what_it_changes` | spec |
| THR-11.3 | a task touches no one-thread count: E7015 at the spawn | `reject_a_task_touches_a_one_thread_count` | spec (two) |
| THR-21.1 | reductions gathered per task, combined at the join in spawn order | `accept_reductions_are_combined_at_the_join` (sum, push, max, `u8` taken from, the float `0.6`) | — |
| THR-21.2 | one operator per place; nothing reads it before the join; plain assignments need a lock (E7007) | `reject_a_read_before_the_join_and_mixed_or_plain_changes` | spec (the read); none |
| THR-21.3 | a task's own total overflowing stops the program (verdict 105) | `run_fail_a_tasks_own_total_overflows` | — |
| THR-18.1 | `with x.lock:` holds a `Lock`; a direct call taking the held lock enters past it | `accept_the_log_under_its_lock` (the spec's program) | — |
| THR-18.2 | a guarded field only under its lock on its object (E7013); a function needing the lock called only under it; an unproved re-entry is E7018; the lock not replaced in its block (E3060) | `reject_a_guarded_field_used_without_its_lock` | spec (four); none |
| THR-18.3 | `init` sets guarded fields; changed through any name under the lock; one call reaches a guarded object through several names | `accept_guarded_fields_changed_through_any_name_under_their_lock` | — |
| THR-19.1 | a wait for a lock this thread holds is E7018: a function value, a walk under the lock, a join, a scope's end, a second `read()` | `reject_a_wait_for_a_lock_this_thread_holds`; `accept_the_lines_the_messages_give` | spec (four); none |
| THR-20.1 | a `static mut` is the main thread's and its scopes' unless guarded: E7012, a `drop` on another thread included; a `static` of a self-changing type is anyone's | `reject_another_thread_reaches_a_module_variable_of_the_main_thread`; `accept_what_threads_and_tasks_may_reach` | spec; none |
| THR-20.2 | two tasks changing a `static mut`: E7007 | `reject_two_tasks_change_a_module_variable` | none |
| STA-1.1 | a guarded `static mut`'s `T` is `Send` (E7002); a `static`'s is `ThreadShared` | `reject_a_guarded_module_variable_of_a_type_that_is_not_send` (`CACHE`; a `Receiver`) | spec; none |
| FFI-33.1 | an any-thread export reaching an unguarded `static mut`: E7012 | `reject_an_any_thread_export_reaches_an_unguarded_module_variable` | none |
| THR-3.1 | a guard sent to another thread: E7008; no word before a lock's argument (E3028); locks change through plain names | `reject_a_guard_sent_to_another_thread_and_a_word_before_a_lock`; `accept_locks_change_through_plain_names` | spec (two) |
| BRW-20.1 | a view through a guard does not leave its `with` block (E3063) | `reject_a_view_through_a_guard_leaves_its_with_block`; `accept_a_copy_leaves_the_block` | spec (two); none |
| THR-15.1 | a channel of a type that is not `Send`: E7015; `copy of tx`; a queued `Sender` ends with the receiver | `reject_a_channel_of_a_type_that_is_not_send`; `accept_the_producer_and_the_senders` (§XI.2's program) | spec |
| JOB-2.1 | `jobs.submit` outside a scope names its captures (E7004) and holds no one-thread value (E7015); scoped jobs use the caller's variables | `reject_an_unscoped_job_names_its_captures_and_holds_no_one_thread_value`; `accept_scoped_jobs_use_the_callers_variables` | none |
| JOB-3.1 | `submit_after` orders but does not relax THR-11 | `reject_ordering_does_not_relax_the_task_check` | none |
| PAR-1.1 | a range or a list walked by element; else E7016 | `reject_a_parallel_loop_over_what_it_cannot_split`; `accept_a_list_walked_by_element` (`hurt_all`) | spec |
| PAR-2.1 | E7010 with the lines that compile: an index written twice, an undeclared total, a list that may hold one object twice | `reject_iterations_that_are_not_independent` | spec (two); part |
| PAR-3.1 | declared reductions; a chunk's own total overflowing stops (verdict 105) | `accept_declared_reductions_combine_in_chunk_order`; `run_fail_a_chunks_own_total_overflows` | — |
| TYP-46a.1 | `thread_shared` made in an instance no thread may call: E7015 | `reject_a_thread_shared_value_made_in_an_instance_no_thread_may_call` | none |
| RC-4.3 | a drop that is not plain runs where its last owner ends, before `join` returns | `accept_a_drop_runs_where_the_last_owner_ends` | — |
| WK-12.1 | a name opened from a `weakref thread_shared T` link holds a count | `accept_a_thread_opens_a_weakref_thread_shared_link` | — |
| WK-12.2 | `ref mut` after `exists as` on such a link: E7014 | `reject_ref_mut_after_opening_a_weakref_thread_shared_link` (Q5) | none |

## 2. Old tests the target contradicts (the gap map's and those seen, not a sweep)

- `tests/conformance/THR-1/accept_generic_sync_class_of_int.em`, `accept_sync_class_uses_atomic_reference_counts.em`,
  `reject_generic_sync_class_of_non_sync_handle.em`, `reject_a_sync_class_holding_an_owned_callable_value.em`:
  0.9.10's `@sync` classes; a value several threads own is now `thread_shared` (THR-1).
- `THR-2/accept_a_sync_class_handle_crosses_threads.em`: the same.
- `THR-11/accept_scoped_tasks_borrow_the_block.em`: tasks are judged by their effects now, not by
  borrows (the gap map's rewrite).
- `THR-10/*` (3 respell, 1 rewrite) and `JOB-2/*` (3 respell): the capture words of THR-17.

## 3. Spec questions

None is decided here; each has its readings and the one recommended.

| # | rule | the question | readings | recommended |
|---|---|---|---|---|
| Q1 | THR-10, THR-8 | "`R` MUST be `Send` (`E7015`, `E7008`)": which code for which `R`? `thread.spawn(owns fn(): shares Player())` | (a) E7015 for a one-thread owner or a `weakref T` (E7015's row), E7008 for a thread-bound value (a guard, a `Scope`, a raw pointer: E7008's row); (b) one code for both | **(a)**, each by its row. File: `THR-10/reject_a_thread_whose_result_is_not_send` (E7015) |
| Q2 | THR-10 | "a `thread.spawn` inside a walk of what the thread changes is `E3020`, except where the thread makes that change under a lock the walk holds": an unscoped thread changes only `Atomic`s, lock contents, guarded fields and `thread_shared` values through their own operations (`[THR-21]`'s last sentence, `[THR-1]`, `[THR-20]`), and a walk of a guarded field or a lock's content holds its lock (E7013 otherwise), which is the exception. No program was found that the `E3020` clause refuses | (a) a clause with no case left: repaired to say only the exception (and `E7018` for joining inside the lock); (b) a case exists that this deputy missed | **(a)**, unless the lead finds the case. Not written |
| Q3 | THR-11, THR-20, THR-18, THR-19, PAR-2, JOB-3, FFI-33 | where E7007, E7012, E7010, THR-18's E3060 and THR-19's E7018 for a scope's end stand: the spec gives the code and often the text, not the line | (a) E7007 on the spawn of the second task whose claim conflicts; E7012 at the call that starts the other thread (or the export's first line), with the chain (`[DIA-22]`, `[DIA-27]`); E7010 on the write or the total named; E3060 on the replacing line; the scope's E7018 on its `with` line; (b) at the line the message names first | **(a)**, the line the fix acts on. Files claim these for the file |
| Q4 | GRM-17, THR-21, THR-17, CLO-17 | `[GRM-17]` (0.9.20): "A lambda with a `:` body inside brackets holds exactly one simple statement (...). More is `E0106`"; `[THR-21]` writes `scope.spawn(owns fn() copy of part: for x in part: total += x)` and `[CLO-17]` `fn() weakref bus: if bus exists as ref mut b: b.handlers.clear()`, compound statements | (A) GRM-17 stands: the examples are `E0106` and are repaired with a local function (`[GRM-28]`); (B) a compound statement whose body is one simple statement on the same line counts as one statement | **(A)**: GRM-17 was changed in 0.9.20 to say "simple", and its help gives the local function. The THR-21 files keep the spec's own line until it is ruled; under (A) they take a local function `fn add_part():` with its capture words |
| Q5 | WK-12, THR-1, E7014 | where E7014 stands for `ref mut` after `exists as` on a `weakref thread_shared T` link: the opening line (E7014's row: "`ref mut` after `exists as` (...) included") or the change through the name (today's compiler) | (a) the `exists as ref mut` line; (b) the first change | **(a)**, the word the fix removes. File follows (a) |
