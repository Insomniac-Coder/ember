# Safety matrix, writer W5: R12 (function values)

Run 4, step 4 (agent 07, test updater for group T3), brought to verdicts 1 to 97. Rule: R12,
`V5/rules.md` lines 661-786, with the section 3 refusals and the section 2 run-time rows that belong to
it. Verdicts from `V5/inputs/JOB.md`; settled points from `V5/inputs/worklist.md` and `V5/changes.md`.
Tests: `V5/conformance/R12/`, 100 tests (43 `reject_`, 52 `accept_`, 5 `run_`) and one module the tests
import (`widgets.em`). Nothing was compiled or run (brief, "Machine limits"). `V5` =
`tasks/96/redesign/run4/v5/`; "line N" with no file is a line of `V5/rules.md`. Step 3's matrix and the
R12 folder as this step found them are kept at `target/redesign7/07/orig/`.

What this step changed, in short: verdict 86 removed `changes`, `reads`, `captured` and change lists, so
the 31 R12 tests of the spelling report's section 1 (their expected error, E2105 or E2113, was a missing
or wrong clause) were each made the nearest test of what the program now does (12) or deleted where the
program had become an existing `accept_` test (19); every other test lost its clauses (the script's
work) and got comments and messages in the v5 words, line numbers and chain messages (R18, F-12); four
tests whose refusal came from a function type now read the stored functions instead (the handler that
causes the refusal is now in the program); verdict 83 (no `with` before captures) and verdict 84 (`owns`
on a number) have their tests; step 3's hole 3 (parts 1 and 2) and hole 6a's original form have a
`reject_` and an `accept_` each; the rows are renumbered to follow v5's R12. Full list at the end.

## How to read this

- One section per guarantee, `R12-1` .. `R12-26`, with the seven fields of the brief.
- **today's tests** are `tests/conformance/<RULE>/<file>.em`. Those that expect the opposite of the new
  rule are marked **(superseded)**.
- **enforced by** names today's source file that does this kind of work now. Short names used below:
  - `typeck`: `compiler/ember_typeck/src/lib.rs` (the type checker; captures are resolved at
    `resolve_capture`, about line 25452, and checked from about line 49099).
  - `effects`: `compiler/ember_analysis/src/effects.rs` (today's effect sets, `[EFF-1]`..`[EFF-6]`,
    worked out from bodies, as v5 does; it says itself that a call through a function value
    "contributes every effect: no function type declares its effects yet". v5 instead counts every
    function the program stores into that place, the whole program and its libraries: new work here).
  - `borrows`: `compiler/ember_analysis/src/borrows.rs` (the borrow check, the MIR pass that today
    checks borrows and regions; it emits E3020, E3021, E3022, E3060; captures from about line 2257).
  - `stores`: `compiler/ember_analysis/src/stores.rs` (emits E3026 today).
  - `drops`: `compiler/ember_analysis/src/drops.rs` (emits E3013 today).
  - `parser`: `compiler/ember_parser/src/body.rs` and `decls.rs`.
  - `codegen`: `compiler/ember_codegen_c/src/lib.rs` and `compiler/ember_codegen_llvm/src/closures.rs`
    (today's function value is `ember_fn_value`, `{native, descriptor, w[3]}`, 40 bytes).
  - `runtime`: `runtime/ember_rt/src/ember_rt.c`.
  - `interface`: `compiler/ember_build/src/interface.rs` (a package's interface file, R17).
  E0120, E7015 and E2020's lambda-parameter case are not in today's compiler (step 3's search; E7015 is
  a code this model made, rules section 4's note).
- **How a test is verified once built**: a `compile-fail` test by the runner's expected code and
  message on the marked line and its `#$ help:` line (`compiler/ember_driver/tests/milestones.rs`:
  `assert_exact_diagnostics`, a diagnostic no annotation claims fails the test; `assert_diagnostic_helps`,
  each `#$ help:` is text of the diagnostic's help); `run-pass` by `#$ stdout:`; the five `run_` tests,
  and the `accept_` tests that dispatch handlers, also by a build of the emitted C under
  AddressSanitizer with no report, as `V3/area-D.md` AN-60, AN-61 and AN-62 checked the same shapes in C.
- Every message was checked against R18 and verdict 73: the lines it gives compile in that program (the
  `accept_` test named beside each `reject_` test is, in most rows, that program after the fix); every
  line number a message names was checked against its file by a script
  (`target/redesign7/07/check_lines.py`); every `#$ help:` is text of its message. A message about a
  change a call makes names the chain of calls (R18 lines 952-954); none asks for a word on a first line.

---

### R12-1. A lambda writes nothing about what it changes; the change is worked out and checked where it runs

| field | value |
|---|---|
| guarantee | What a lambda changes (a module variable, a real variable it uses) is worked out from its body, with no word on the lambda or on the functions that run it; a call that runs it inside a loop over what it changes is refused, the message naming the chain. A `changes` clause on a lambda, in the `:` form or the `=>` form, is not Ember (E0100) and the message gives the line without it. |
| source | R12 lines 662-672; R5 lines 362-384; R8 (c) lines 477-494; R18 lines 945-962; verdicts 17, 45, 86; section 3 lines 1051 (E0100, third message's shape), 1065 (E3020) |
| enforced by | `effects` for the change, extended to every function the program stores in the place called; `borrows` for E3020; `parser` for E0100 (today's "unexpected token") |
| today's tests | `EFF-1/accept_a_call_through_a_plain_fn_value_has_the_effects_of_its_functions.em` (effects of a call through a function value worked out, not written: the same approach, by shape today) |
| missing edge cases | a lambda that changes `LOG` only through a call it makes (`fn(): tick()`), covered by the chain but not tested here; a lambda inside a lambda; a named local function (verdict 51) that pushes to `LOG`, run inside a loop over `LOG` |
| tests | `reject_a_lambda_that_pushes_to_log_called_inside_a_loop_over_log.em`, `accept_a_lambda_that_pushes_to_log_called_after_the_loop.em`, `reject_row_6_handler_written_with_changes_score.em`, `reject_an_arrow_lambda_written_with_changes_removed.em`, `accept_an_arrow_lambda_changes_a_variable_it_uses.em`, `accept_a_lambda_changes_an_atomic_it_uses.em`, `accept_a_lambda_that_only_reads_a_variable_it_uses.em` |
| expected result | compile-fail E3020 "`click()` (line 16) pushes to `LOG` (it runs the function made on line 13) while this loop walks `LOG` (line 14); call it after the loop: move line 16, `click()`, below the loop"; run-pass `start`, `2`; compile-fail E0100 "`changes score` is not Ember: the compiler works out what this function changes from its body; write `b.on_click(fn(): score += 1)`" (row 6's own line, nearest allowed `accept_row_6_handler_changes_the_real_score.em`); compile-fail E0100 for `changes removed` before `=>`; run-pass `2 1`; run-pass `2`; run-pass `0` |

### R12-2. A function type says its parameters, its result and `thread_shared`, nothing about changes

| field | value |
|---|---|
| guarantee | `fn()`, `fn(Node)`, `fn(owns String) -> int` say nothing about what their functions change; what a call through one changes is worked out from every function the program stores there. A long type is named once with `type`; a named function is a value of its type; a local function captures as a lambda does and may be passed by its name. A `changes` clause in a function type is E0100. |
| source | R12 lines 662-665, 672-676; R3 lines 201-208; verdicts 48, 49, 51, 86; section 3 line 1051 |
| enforced by | `parser` (E0100); `typeck` (function types, `type` aliases, local functions) |
| today's tests | `CLO-14/accept_a_callable_type_alias_as_a_bound.em`, `STD-19/accept_a_named_callable_is_lent_and_kept.em` |
| missing edge cases | a `type` alias with type parameters (`type Pair[T] = fn(T, T)`); a named function assigned to a field (`t.on_visit = prune`, AN-62) |
| tests | `reject_a_function_type_written_with_changes_log.em`, `accept_named_functions_kept_in_a_list_of_fn.em`, `accept_report_clicks_after_the_loop_over_log.em` (`type Handler = fn()`), `accept_update_takes_a_death_handler_type.em` (`type DeathHandler = fn(ref mut Array[Enemy], int)`, verdict 49 without its change list), `accept_a_local_function_that_changes_a_mutex_passed_to_spawn_by_name.em` (verdict 51) |
| expected result | compile-fail E0100 "`changes LOG` is not Ember: a function type says its parameters and result, and the compiler works out what the functions kept here change; write `handlers: Array[fn()] = []`"; run-pass `1`; run-pass `start`, `start`, `clicked`; run-pass `2 1`; run-pass `1` |

### R12-3. A capture with no word is the real variable (row 6)

| field | value |
|---|---|
| guarantee | A name a lambda uses and does not list is the real variable: a change by the lambda reaches it, a change by the function that made it is seen by the lambda, and nothing on any line says so. One `Array[fn()]` keeps handlers that change different things. |
| source | R12 lines 670-671, 752-759; R3 lines 230-234; row 6; verdicts 22 (meaning kept), 80, 86; `V3/area-D.md` line 43; AN-53 |
| enforced by | `typeck` (capture resolution); `borrows` for what the place that holds the lambda reaches |
| today's tests | `CLO-2/accept_mutating_a_captured_local.em` (a capture written in place); `CLO-2/reject_outer_write_while_mutating_capture_is_live.em` and `CLO-12/reject_a_capture_written_while_the_local_function_lives.em` **(superseded)**: today a write by the maker while the lambda lives is E3021; under row 6 it is seen by the lambda |
| missing edge cases | a real variable that is a list read by the lambda while the maker reshapes it between calls (the lambda holds no view across calls, so nothing to refuse, but untested); two lambdas sharing one real variable |
| tests | `accept_row_6_handler_changes_the_real_score.em`, `run_a_real_variable_is_seen_by_the_handler_and_back.em`, `accept_a_lambda_changes_the_real_variable.em`, `accept_a_handler_that_changes_two_variables_it_uses.em`, `accept_a_button_keeps_a_score_handler_and_a_log_handler.em`, `accept_a_local_list_of_functions_that_change_score.em` |
| expected result | run-pass `1`; run-pass `1`, `11`, AddressSanitizer clean; run-pass `1`; run-pass `1 2`; run-pass `1 1`; run-pass `1` |

### R12-4. What a function captured with a word is its own state; a change to it and the end of what it owns are worked out (hole 3)

| field | value |
|---|---|
| guarantee | A named capture (`copy of`, `owns`, ...) is the function value's own state. A change to that state, and the end of what it owns, are worked out from the body like every change: a view of a node's text reached through a link is not used after a call that replaces that text (hole 3, part 1), and a view of a node is not used after the function value that owns it is replaced (hole 3, part 2). Nothing runs for either while the program runs. |
| source | R12 lines 681-691, 766-767; R8 (b) lines 458-476 (466-468); R10 point 1 lines 564-574; R5 lines 362-372; section 3 line 1069 (E3060, fourth and fifth messages); `V5/safety/recheck.md` sections 6.1, 6.2; worklist section 4 ("Hole 3, part 1 ... Part 2") |
| enforced by | `effects` (what the stored function changes and ends); `borrows` (E3060 at the later use); `codegen` for the state block |
| today's tests | `CLO-10/accept_an_owned_callable_value_keeps_its_state.em`; `CLO-13/accept_a_lambda_copies_what_it_only_reads.em` (today's silent copy for a `Copy` read: **superseded**, a read is now of the real variable) |
| missing edge cases | the `drop` of a node held in a function value's state counted where the value is replaced, inside a walk of what that `drop` changes (E3020, R12 lines 690-691); an `owns dyn I` value replaced (line 691); `shares x` and `weakref x` state ending with the function value (counts given back): a run test under ASan with leak detection |
| tests | `run_a_named_copy_is_the_handlers_own_state.em`, `accept_the_handler_takes_a_copy_of_the_loop_name.em`, `accept_a_lambda_takes_the_name_it_uses_with_owns.em`, `reject_a_view_through_a_link_used_after_a_stored_function_replaces_its_text.em`, `accept_a_view_through_a_link_used_before_a_stored_function_replaces_its_text.em`, `reject_a_view_of_a_node_used_after_the_function_value_owning_it_is_replaced.em`, `accept_a_view_of_a_node_used_before_the_function_value_owning_it_is_replaced.em` |
| expected result | run-pass `1`, `2`, `0`; run-pass `ann`, `bo`; run-pass `ann`; compile-fail E3060 "`rename()` (line 20) replaces the node's name (`n.name = String("b")`, line 17), and `s` views that name's text (line 19) and is used on line 21; use `s` before line 20, or keep your own text: `s = String(p.name.trim())` in place of line 19"; run-pass `a`, `b`; compile-fail E3060 "`h.keep = fn(): pass` (line 22) ends the function stored there on line 20 and the node it owns (`owns n`), and `p` reads that node (`if w exists as p:`, line 21) and is used on line 23; use `p` before line 22"; run-pass `a`, `gone` |

### R12-5. A change through a link a function value holds is worked out (run 3's G-05)

| field | value |
|---|---|
| guarantee | A handler that changes an object through a link it holds (`fn() weakref ui`) is seen to change it, with no word: a read-only view of that object does not live across a call that may run the handler (E3021), and a `for` walk of a list the handler may clear through the link is refused (E3020); a handler that changes another field through the link leaves the walk allowed. |
| source | R12 lines 717-720; R8 (b) lines 461-463, (c) lines 477-494, (g) lines 521-534; verdict 86 (verdicts 63 and 64 void in spelling); section 3 lines 1043 (E3021), 1065 (E3020, eighth message) |
| enforced by | `effects` (the stored handler's change through its link); `borrows` |
| today's tests | none that sees a change through a captured link; `CLO-5/accept_a_closure_keeps_a_captured_object_alive.em` is R12-17's |
| missing edge cases | a change through a captured `sharedref mut` link alone; through a captured `shares mut` link alone (only R12-16's `Counter`) |
| tests | `reject_a_view_of_the_ui_across_a_handler_that_changes_it_through_a_weakref.em`, `accept_a_weakref_handler_changes_the_ui_and_a_copy_keeps_the_count.em`, `reject_a_for_walk_of_handlers_one_of_which_clears_them_through_a_weakref.em`, `accept_a_handler_that_changes_the_ui_through_its_link_walked_with_for.em` |
| expected result | compile-fail E3021 at `h()`: "... `before` reads `ui.clicks` (line 30) and is used on line 33; read it before the handlers run: move line 33, `println(before)`, above line 31, or keep a copy of your own: `before = copy of ui.clicks` in place of line 30"; run-pass `0 1` (the copy way out); compile-fail E3020 at `h(this)`, the rules' eighth message in this program's lines, also naming `close_all`'s lines (a `fn(Ui)` does not fit `Array[fn(ref mut Ui)]`; R18); run-pass `1` |

### R12-6. A change under a lock is worked out; a call that may take a lock this thread holds is refused

| field | value |
|---|---|
| guarantee | A function, a lambda or a local function that changes a `guarded by` field under its lock, or the value in a `Mutex`, writes nothing about it; a `guarded by` field is changed through a plain name inside its lock with no word at the call. A call through a function value that may run a function taking a lock of the kind this thread holds, inside a walk under that lock, is E7018 (not E3020, verdict 73). |
| source | R4 lines 265-269; R5 lines 401-410; R15 lines 835-842, 881-890; R8 (c) lines 492-494; R18 lines 954-957; verdicts 29 (meaning kept, spelling void), 43, 44, 73, 86, 89; section 3 line 1099 (E7018, third message) |
| enforced by | `effects` (what each function changes, the locks it may take); the lock check of R15; `typeck` for `guarded by` |
| today's tests | `EFF-21/accept_nolock_without_a_lock.em`, `EFF-21/reject_a_lock_under_nolock.em` (a lock is an effect today, only loosely related); `THR-10/accept_threads_spawn_join_and_detach.em` |
| missing edge cases | a `RwLock`; a channel end (verdicts 36, 37: W6's); the plain call and the join inside the lock (E7018's first two messages) are W6's |
| tests | `accept_add_line_changes_a_guarded_field_through_a_plain_parameter.em`, `accept_two_threads_push_to_one_log.em`, `accept_a_local_function_that_changes_a_mutex_passed_to_spawn_by_name.em`, `reject_a_function_value_that_changes_log_called_in_a_walk_of_log_lines.em`, `accept_the_function_value_called_after_the_with_block.em` |
| expected result | run-pass `1`; run-pass `2`; run-pass `1`; compile-fail E7018 "this loop walks `log.lines` (line 25) under `log`'s lock, and `note()` (line 27) may run the function stored on line 23, which takes a `Log`'s lock (`log.push`, under `lock`), so it would wait for the lock this thread holds; call `note()` after the `with log.lock:` block: move line 27 below the block"; run-pass `a`, `2`. `note` is made inside `dump`, so the test keeps one fault; passing it beside `log` is allowed by verdict 89 (R4 exception 2), whose `accept_` is W6's |

### R12-7. A module's stored function values are counted where they run (the whole program)

| field | value |
|---|---|
| guarantee | A module (or library) that neither declares nor imports a program's `LOG` keeps that program's handler that pushes to `LOG`; `b.click()` is then a change of `LOG` in the program, refused inside a loop over `LOG`, the message naming `click`'s line in the module. |
| source | R12 lines 665-670; R5 lines 362-366; R17 lines 925-943 (938-939: "the function values a package stores into a function type ... which a program that uses the package counts wherever such a value is called"); verdict 86 (verdicts 40 and 76 void in spelling) |
| enforced by | `effects` over the whole program; across packages the interface file (`interface`) |
| today's tests | `MOD-2/accept_a_pub_method_and_an_interface_method_anywhere.em` (only that a module's method is callable) |
| missing edge cases | the same across a package edge (two packages; today's runner has no package fixture in `tests/conformance/` for this); a library whose own body changes the program's `LOG` it imports (R5's, W2) |
| tests | `widgets.em` (module, parse-pass), `reject_clicking_a_button_of_another_module_inside_a_loop_over_log.em`, `accept_a_button_of_another_module_clicked_after_the_loop_over_log.em` |
| expected result | parse-pass; compile-fail E3020 "`b.click()` (line 21) adds to `LOG` (through `click` in `widgets.em`, line 14, which runs the handler stored on line 18, which pushes to `LOG`) while this loop walks `LOG` (line 19); click after the loop: move line 21, `b.click()`, below the loop"; run-pass `start`, `2` |

### R12-8. Calling what a list holds is allowed in a loop when no function stored there changes what the loop walks

| field | value |
|---|---|
| guarantee | Where every function the program stores in a list changes nothing the loop walks (it prints, it reads, it changes another field), running them inside that loop is allowed, with no word anywhere: the precision comes from the bodies, not from a type. |
| source | R12 lines 665-670 ("calling what a list holds whose functions change nothing is allowed inside such a loop"); R8 (c) lines 477-494; G-07 (AN-76 part 6) |
| enforced by | `effects`; `borrows` |
| today's tests | `CLO-11/accept_a_callable_field_is_called.em` |
| missing edge cases | a list whose functions are stored by another module, one of which changes the walked list (the refusal then names the module's line) |
| tests | `accept_clicking_plain_handlers_inside_a_loop_over_log.em`, `accept_read_only_handlers_walked_with_for.em`, `accept_a_handler_that_changes_the_ui_through_its_link_walked_with_for.em` (R12-5) |
| expected result | run-pass `a`, `click`, `b`, `click`; run-pass `3`, `3`; run-pass `1` |

### R12-9. A function that runs the function it is given makes its changes

| field | value |
|---|---|
| guarantee | A library function that calls a function right away takes it as a template parameter (`[F: fn(Item)]`), a program's own function may write either that or a function value, and the call counts as making what the given function changes, worked out from it; inside a loop over what that function changes the call is refused. |
| source | R12 lines 703-711; verdicts 21 and 38 (meaning kept), 50, 86; section 3 line 1066 (E3020, first message) |
| enforced by | `typeck` (generic instances); `effects` per instance (today's `[EFF-2]` already gives a callable parameter the passed function's effects) |
| today's tests | `EFF-2/accept_a_callable_parameter_has_the_passed_functions_effects.em`, `EFF-2/reject_a_callable_parameter_given_an_allocating_function.em`, `CLO-14/accept_an_explicit_callable_bound_is_called_like_a_function.em`, `STD-19/accept_a_named_callable_is_lent_and_kept.em` |
| missing edge cases | one instance per function given (verdict 38's speed): no test reads the emitted C (an `#$ assert-c-count:` on `apply_each`'s instances, once the new C names are known); `sort_by` |
| tests | `reject_for_each_inside_a_loop_over_log.em`, `accept_for_each_after_the_loop_over_log.em`, `reject_each_name_inside_a_loop_over_the_list_its_function_pushes_to.em`, `accept_each_name_hands_each_name_to_a_lambda_that_owns_it.em`, `accept_a_programs_own_function_takes_its_function_as_a_template_parameter.em`, `accept_update_takes_a_death_handler_type.em` (R12-2), `accept_an_arrow_lambda_changes_a_variable_it_uses.em` (R12-1, `retain`) |
| expected result | compile-fail E3020 "`scores.iter().for_each(...)` (line 19) pushes to `LOG` (through the function it is given) while this loop walks `LOG` (line 17); call it after the loop: move line 19 below the loop"; run-pass `1`, `3`; compile-fail E3020 "`each_name(...)` (line 17) adds to `names` (`each_name` runs the function it is given, line 11, which pushes to `names`) while this loop walks `names` (line 15); call it after the loop: move line 17 below the loop"; run-pass `a`, `3`, `2`; run-pass `8`, `2` |

### R12-10. An adapter keeps the function; driving it makes the changes, through a `dyn` iterator too

| field | value |
|---|---|
| guarantee | Storing a function in an adapter changes nothing; whatever drives the adapter (`count()`, a `for`) makes its changes, and so does driving an `owns dyn Iterator`, whose type says nothing about changes: the compiler counts every adapter the program can put there. |
| source | R12 lines 711-716, 735-739; verdict 46 (meaning kept), 86; AN-55 |
| enforced by | `typeck` (generic instances and `dyn` coercion); `effects` |
| today's tests | `CLO-15/accept_a_lambda_at_an_adapter_keeps_its_borrows.em`, `STD-19/accept_the_closure_adapters.em`, `CTL-3b/accept_closure_adapters_are_stages_of_a_counted_loop.em` |
| missing edge cases | `map` and `filter` adapters; an adapter kept in a struct field and driven later in a loop over what it changes |
| tests | `accept_inspect_keeps_the_function_and_count_runs_it.em`, `reject_inspect_count_inside_a_loop_over_seen.em`, `reject_driving_a_dyn_iterator_that_pushes_to_log_inside_a_loop_over_log.em`, `accept_driving_a_dyn_iterator_that_pushes_to_log_after_the_loop.em` |
| expected result | run-pass `0`, `3`; compile-fail E3020 "`xs.iter().inspect(...).count()` (line 14) adds to `seen` (`count` drives the adapter, which runs the function `inspect` keeps, which pushes to `seen`) while this loop walks `seen` (line 12); count after the loop: move line 14 below the loop"; compile-fail E3020 "`show(...)` (line 21) adds to `LOG` (`show` drives the iterator it is given, line 13, ...) ... move line 21 below the loop"; run-pass `0`, `1`, `2`, `3` |

### R12-11. Across a call through a function value: no walk, no read-only view, no `ref mut` name of what it changes

| field | value |
|---|---|
| guarantee | A call that may run a function value is a change of everything the functions stored there change, the real variables they use included, through every call on the way (`press` → `click` → the handler): a list they change is not walked across the call (E3020), a read-only view of what they change does not live across it (E3021), nor a `ref mut` name of such a place (E3022). A handler given the whole Ui `ref mut` that adds to or removes from its list is not run from a `for` walk of that list. |
| source | R12 lines 717-720, 752-756; R8 (c) lines 477-494, (f) lines 505-520, (g) lines 521-534; R18 lines 945-962; verdict 86; AN-53, AN-58, AN-60, AN-76; section 3 lines 1043 (E3021), 1063 (E3022), 1065 (E3020) |
| enforced by | `borrows`, reading what the stored functions change instead of today's "unknown call" (`effects`) |
| today's tests | `BRW-1/reject_mutating_a_viewed_container.em`, `CTL-1/accept_an_adapters_items_borrow_what_it_reads.em`; `CLO-15/reject_what_an_adapter_holds_is_still_borrowed_or_moved.em` (a lambda's borrow held while an adapter lives: **superseded**, the borrow is now at the call that drives it) |
| missing edge cases | a position (`boss = enemies[0]`) across a call through a function value that removes from `enemies` (E3060 at the later use, R8 (a)); a generator paused in a list across such a call; a `sharedref mut` holder |
| tests | `reject_clicking_a_log_button_inside_a_loop_over_log.em`, `accept_report_clicks_after_the_loop_over_log.em`, `reject_a_stored_function_that_pushes_to_the_walked_list_called_in_the_loop.em`, `accept_the_stored_function_called_after_the_loop_as_many_times.em`, `reject_clicking_inside_a_loop_over_a_list_a_handler_changes.em`, `accept_clicking_after_the_loop_over_the_list_a_handler_changes.em`, `reject_press_inside_a_loop_over_the_list_its_handler_changes.em`, `accept_press_after_the_loop_over_the_list_its_handler_changes.em`, `reject_a_read_only_view_across_a_call_that_changes_what_it_reads.em`, `reject_a_ref_mut_name_across_a_call_that_changes_its_place.em`, `accept_views_used_before_the_call_that_changes_their_place.em`, `reject_a_for_walk_of_handlers_given_the_ui_ref_mut.em`, `accept_a_handler_called_by_position_adds_one_and_removes_itself.em` |
| expected result | compile-fail E3020 at `b.click()` (line 25), the chain through `click` (line 18) to the handler stored on line 30, "click after the loop: move line 25, `b.click()`, below the loop" (F-12); run-pass `start`, `start`, `clicked`; compile-fail E3020 at `hs[0]()` (line 16), "write `n = xs.len()` above line 14, remove line 16, and below the loop write `for k in 0..n:` with `hs[0]()` under it"; run-pass `1`, `2`, `4`; compile-fail E3020 at `b.click()` (line 27), "move line 27, `b.click()`, below the loop"; run-pass `a`, `2`; compile-fail E3020 at `press(ref mut b)` (line 29), the chain `press` (line 21), `click` (line 16), the handler stored on line 26; run-pass `a`, `2`; compile-fail E3021 "... move line 18, `println(r)`, above line 17" (one way out: R8 (g)'s `ref mut` way out cannot apply to a change a call makes, verdict 73); compile-fail E3022 "... move line 17, `w += 10`, above line 16"; run-pass `0`, `11`; compile-fail E3020 at `h(ref mut this)` (line 17), the chain to `add_once` (stored on line 32; its lines 27, 28), "walk by position ... instead of lines 16 and 17"; run-pass `1 1` |

### R12-12. A function value given to a call counts as an argument that changes what its functions change

| field | value |
|---|---|
| guarantee | `LOG.retain(keep)`, where `keep`'s body pushes to `LOG`, names one place twice and is refused; a lambda that changes nothing overlaps nothing. |
| source | R12 lines 720-722; R4 lines 286-296; section 3 line 1060 (E3022, the `retain` message) |
| enforced by | `borrows` (the two-place overlap of a call), with `effects` for what `keep` changes |
| today's tests | `BRW-3/accept_two_phase_borrows_across_conditional_arguments.em` (overlap of arguments, no function value) |
| missing edge cases | a function value that uses the real variable given beside it (`xs.retain(fn(x) => ... xs.push(...))`); R4's exception 2 (verdict 89: a thing changed only through its lock, `dump(log, fn() thread_shared log: ...)`, allowed), whose `accept_` is W6's (worklist, verdict 89) |
| tests | `reject_retain_given_a_function_that_changes_the_list.em`, `accept_retain_then_push_the_notes.em` |
| expected result | compile-fail E3022 "`LOG.retain(keep)` changes `LOG` while it walks it, and `keep` adds to `LOG` too (line 15); drop the lines first and push the notes after: ..." (the rules' message, word for word but for the line); run-pass `2` |

### R12-13. A function value is not stored in a place that a real variable it changes owns

| field | value |
|---|---|
| guarantee | A handler that clears `b.handlers` is not kept in `b.handlers`, so no function changes its own holder while it runs (Rust refuses the same). |
| source | R12 lines 722-725; AN-53 part 6, AN-62 part 6; section 3 line 1060 (E3022, last message) |
| enforced by | `borrows` at the store, with R4's overlap of `b` and the argument |
| today's tests | none |
| missing edge cases | the store through a field assignment (`t.on_visit = fn(n: Node): t.root.children.clear()`, the rules' second example), all the way down (`b.inner.handlers`) |
| tests | `reject_a_handler_stored_in_the_button_it_changes.em`, `accept_clear_the_handlers_after_the_click.em` |
| expected result | compile-fail E3022 "this function clears `b.handlers`, and `on_click` would keep it in `b.handlers`, so it would change the button that holds it while the button runs it; clear the handlers after the click instead: remove line 22 and write `b.handlers.clear()` below line 23, `b.click()`" (F-12: the lines); run-pass `1 0` |

### R12-14. Running a function on its own counts as making its changes; storing it does not

| field | value |
|---|---|
| guarantee | `thread.spawn` (which takes a `thread_shared fn()`), `scope.spawn` and a `callback(retained)` / `callback(once)` parameter count as making what the function changes, now or later, worked out from it, with no word on the caller; storing a function value (a field, a list, an adapter) changes nothing, so a function that only stores a `LOG` handler may run inside a loop over `LOG`. |
| source | R12 lines 726-739; R15 lines 835-845; verdicts 30 (meaning kept), 52, 86, 94; G-06; AN-76 |
| enforced by | `effects`; the library's first lines (`std/src/thread.em`) |
| today's tests | `THR-10/accept_threads_spawn_join_and_detach.em`, `CLO-7/accept_spawn_takes_its_callback_over.em` |
| missing edge cases | the refusal itself: a `scope.spawn` inside a loop over what the task changes (E3020) and the join inside the lock (E7018, verdict 52) are W6's (`R15/reject_a_task_changes_the_list_its_parent_walks.em`) |
| tests | `accept_main_starts_threads_that_push_to_results.em`, `accept_two_threads_push_to_one_log.em` (R12-6), `accept_setup_stores_a_log_handler_inside_a_loop_over_log.em`, `accept_inspect_keeps_the_function_and_count_runs_it.em` (R12-10) |
| expected result | run-pass `1 1`, `2`; run-pass `2`; run-pass `start`, `2`; run-pass `0`, `3` |

### R12-15. A foreign call may run the callbacks C keeps

| field | value |
|---|---|
| guarantee | Every foreign call not marked `@ffi(no_callbacks)` counts as possibly running every callback C keeps, so a function that makes one (`tick`, through `poll_events()`) changes what those callbacks change, worked out with no word, and is refused inside a loop over it; the message names the chain and the mark. |
| source | R12 lines 740-751; verdicts 31, 39 (meaning kept), 86; AN-78; section 3 line 1066 (E3020, the `poll_events()` message) |
| enforced by | `effects` (which reads today's `callback(...)` contracts); the trampoline in `compiler/ember_codegen_c/src/lib.rs` and `compiler/ember_codegen_llvm/src/ffi.rs` |
| today's tests | `FFI-21/accept_callbacks_c_keeps_past_the_call.em`, `FFI-21/reject_what_a_callback_captures.em` |
| missing edge cases | `@ffi(no_callbacks)` and E5069 are W6's (`V5/conformance/R15/`) |
| tests | `reject_tick_inside_a_loop_over_log_may_run_the_callback_c_keeps.em`, `accept_tick_after_the_loop_over_log_may_run_the_callback_c_keeps.em` |
| expected result | compile-fail E3020 "`tick()` (line 44) may add to `LOG` (through `poll_events()`, line 36, a C function that may run the callback C keeps from `register_cb`, line 33, which pushes to `LOG`, line 32) while this loop walks `LOG` (line 42); call `tick()` after the loop: move line 44 below the loop; if `poll_events` never calls back into Ember, say so on its declaration: `@ffi(no_callbacks) safe fn poll_events()`"; compile-pass (the C library is not part of the test). The callback is `threads(main)`, so it may change the main thread's `LOG` |

### R12-16. A function value may not outlive a place it uses and does not own

| field | value |
|---|---|
| guarantee | A handler that changes the caller's `total`, or uses the loop's name, and is kept by an object that outlives it, is refused at the lambda's line; a handler that owns a share, or a copy, outlives nothing. |
| source | R12 lines 752-760, 785-786; AN-57; section 3 line 1095 (E3026, first message) |
| enforced by | `stores` (E3026 today), reading where each function value goes in the function that wrote it |
| today's tests | `CLO-3/reject_a_borrowing_lambda_stored_as_a_value.em`, `CLO-7/reject_a_callback_that_borrows.em`, `TYP-15/reject_a_closure_storing_a_parameter_in_a_capture.em` (all E3026 today, reported inside the callee) |
| missing edge cases | a lambda capturing an opened name (`exists as p`) kept past the block; `ref t` for a handler (R12 lines 759-760; its scoped-task use is W6's) |
| tests | `reject_a_handler_that_outlives_the_callers_total.em`, `accept_the_handler_owns_a_share_of_a_counter.em`, `reject_a_handler_kept_past_the_loop_name_it_uses.em`, `accept_the_handler_takes_a_copy_of_the_loop_name.em` (R12-4) |
| expected result | compile-fail E3026, the rules' first message word for word (it declares `class Counter:`, names `main`'s three lines and `register`'s two, and gives `b.on_click(fn() owns count: count.n += 1)`); run-pass `1`; compile-fail E3026 "... give the handler its own copy: `b.on_click(fn() copy of name: println(name))`"; run-pass `ann`, `bo` |

### R12-17. A function that outlives the function that made it takes what it keeps with `owns`, a number too

| field | value |
|---|---|
| guarantee | Without a word a function only borrows: a returned function that changes its maker's local is refused (E3026); with `owns` the variable, an `int` included, is handed over and becomes its own state, it outlives the maker, and its type is a plain `fn()`. |
| source | R12 lines 761-765; R3 lines 230-236; R4 lines 256-257; section 2 line 985; verdicts 80, 84; section 3 line 1095 (E3026, second message); worklist F-02 (settled by verdict 84) |
| enforced by | `stores` (E3026) and `typeck` for the result type |
| today's tests | `CLO-5/accept_a_closure_keeps_a_captured_object_alive.em` (**superseded**: `return fn() -> int => c.value` reads its maker's local `c`; it now needs `owns c`); `CLO-3/reject_a_borrowing_lambda_stored_as_a_value.em` |
| missing edge cases | `copy of` a number for a returned function (`return fn() copy of n:`), the maker keeping its own; the maker using `n` after `owns n` (E3040: R3's test, W1's) |
| tests | `reject_a_returned_function_that_changes_its_makers_local.em`, `accept_a_returned_function_owns_the_variable_it_keeps.em`, `run_a_counter_made_by_a_function_keeps_its_variable.em` |
| expected result | compile-fail E3026 "the returned function changes `n`, which ends when `make_counter` returns; let the function own it: write `return fn() owns n:` in place of this line" (verdict 80's own line, `n` an `int`); run-pass `1`, `2`; run-pass `1`, `2`, `1` under AddressSanitizer with leak detection |

### R12-18. Where a captured variable lives never changes what the program does (section 2, line 985)

| field | value |
|---|---|
| guarantee | A real variable captured without a word stays at its own address, and the function value provably ends first; a variable a function keeps with `owns`, or any named capture, lives in the function value's own block. No box is made (verdict 80). |
| source | R12 lines 752-767; section 2 line 985 ("nothing / nothing beyond the function value's own block": W4, 1.091x of plain C, safe Rust 1.358x; run 3's box, 1.255x, no longer made) |
| enforced by | `codegen` for the placement |
| today's tests | `CLO-10/accept_wide_captured_state_is_held_inline.em`, `CLO-10/accept_small_captured_state_in_noalloc_code.em` (today's state layout) |
| missing edge cases | none left by the rules |
| tests | `run_a_real_variable_is_seen_by_the_handler_and_back.em` (address), `run_a_named_copy_is_the_handlers_own_state.em` (state of a named capture), `run_a_counter_made_by_a_function_keeps_its_variable.em` (an `owns` variable that outlives its maker) |
| expected result | run-pass `1`, `11`; run-pass `1`, `2`, `0`; run-pass `1`, `2`, `1`; each also under AddressSanitizer with leak detection: no report |

### R12-19. A stored function value is two words, copied out before the call (section 2, line 986)

| field | value |
|---|---|
| guarantee | A call through a list element or a field copies `{code, state}` out before the arguments are given and keeps no pointer into the list, so a handler may grow or shrink the list it is in while it runs, and nothing is written back. |
| source | R12 lines 768-770; R8 (f) lines 516-519; section 2 line 986; AN-60, AN-61 (today's write-back through a stale pointer, `c/an61_dispatch.c` mode 1, heap-use-after-free) |
| enforced by | `codegen` (the call of a stored function value); `borrows` for R8 (f)'s exception |
| today's tests | `CLO-11/accept_a_callable_field_is_called.em`; none tests a handler that moves its own list's buffer (AN-61's unsound programs are accepted today) |
| missing edge cases | a field (not a list element) that a handler replaces while it runs (`this.on_visit = other` inside `on_visit`) |
| tests | `run_a_handler_that_grows_the_list_during_dispatch_keeps_its_state.em`, `accept_a_handler_called_by_position_adds_one_and_removes_itself.em` (R12-11), `accept_a_ref_mut_name_from_get_mut_calls_the_handler.em` (R12-21) |
| expected result | run-pass `1`, `2`, `16`, `0` under AddressSanitizer: no report (the run-time proof); run-pass `1 1`; run-pass `3` |

### R12-20. A handler that removes itself and then uses its state holds one count on it (section 2, line 987)

| field | value |
|---|---|
| guarantee | A body that may end its own state (it removes itself) and uses that state afterwards holds one count on it across that call, so the state is not freed under it. |
| source | R12 lines 770-771; section 2 line 987; AN-60 part 5 (`c/an61_dispatch.c` mode 2 clean, mode 3 without the count a heap-use-after-free) |
| enforced by | `codegen` with the `runtime` count on the state block |
| today's tests | none |
| missing edge cases | a handler that removes itself through a call several levels down (decided from the callees' bodies); a C callback that ends its own token (verdict 62, R12 lines 771-775) is W6's |
| tests | `run_a_handler_that_removes_itself_then_uses_its_state.em` |
| expected result | run-pass `once`, `8`, `8` under AddressSanitizer: no report |

### R12-21. A call through a stored function value that may change its own state is a change of its holder

| field | value |
|---|---|
| guarantee | Where a function the program stores in a place may change its own state, the place is reached through a name that may change it (`for ref mut h`, `h = ref mut ...`, the list by position), and such a function value is not given to a parameter that only reads; a function with no state of its own (it changes a real variable) may be called through a plain name. A later use of the `ref mut` name is checked against what the call may have done. |
| source | R12 lines 776-781; R8 (a) lines 451-457, (f) lines 505-520; `V5/changes.md` section 8, item 4; AN-54, AN-58, AN-61; section 3 lines 1042 (E3023), 1069 (E3060) |
| enforced by | `typeck` (E3023), with `effects` for whether a stored function changes its own state; `borrows` (E3060) |
| today's tests | `CLO-2/reject_a_callable_value_called_through_a_shared_reference.em` (E3021 today, help `.iter_mut()`), `CLO-2/reject_mutating_capture_through_borrowed_callable_parameter.em`, `CLO-2/accept_mutating_capture_through_mut_callable_parameter.em`, `CLO-2/reject_calling_an_owned_callable_value_from_a_borrowed_place.em` |
| missing edge cases | a field holder (`depth_total` running `this.on_visit`, AN-62) |
| tests | `reject_click_walks_stateful_handlers_with_a_plain_name.em`, `accept_click_walks_stateful_handlers_with_ref_mut.em`, `accept_a_plain_loop_name_calls_a_function_with_no_state_of_its_own.em`, `reject_a_plain_name_from_get_mut_calls_the_handler.em`, `accept_a_ref_mut_name_from_get_mut_calls_the_handler.em`, `reject_the_handler_name_used_again_after_the_call.em`, `reject_a_stateful_callback_given_to_a_read_only_parameter.em`, `accept_run_takes_the_stateful_callback_ref_mut.em` |
| expected result | compile-fail E3023 "`h` only reads the handler in `this.handlers`, and calling it may change its own state (`tick`, stored on line 27, adds to its own copy of `n`, line 25); walk the handlers with `for ref mut h in this.handlers:` in place of line 18" (the rules' first line-1042 message); run-pass `1`, `2`; run-pass `1`; compile-fail E3023 (the same shape, with both ways out: by position in place of lines 17 and 18, or `h = ref mut ...` in place of line 17); run-pass `3`; compile-fail E3060 "`h(ref mut this)` (line 17) may remove handlers (it may run `close_all`, stored on line 26, which clears `ui.handlers`, line 22), which may put another handler in the place `h` names (line 16), or none, and `h` is used on line 18; name it again after the call: ..."; compile-fail E3023 (the rules' second line-1042 message, word for word); run-pass `ann 1`, `ann 2`, `0` |

### R12-22. `once fn`: a function that gives a capture away runs once

| field | value |
|---|---|
| guarantee | A function value that gives a capture away is a `once fn()`; it is held by `owns f: once fn()`, refused where it may run twice, a plain `f: once fn()` is refused, a lambda cannot hand over a real variable it does not own, and a second call of a `once fn` is refused. |
| source | R12 lines 782-784; section 3 line 1096 (E3013); section 4 line 1188 (E3040) for the second call; AN-59 |
| enforced by | `drops` (E3013 today); `typeck` for `once` |
| today's tests | `CLO-6/reject_a_once_fn_parameter_called_twice.em` (E3040), `CLO-6/reject_a_once_closure_where_fn_is_required_names_once_fn.em`, `CLO-6/accept_a_once_fn_parameter_takes_either_kind.em`, `CLO-6a/reject_a_once_fn_value_called_twice_or_from_a_borrowed_place.em` |
| missing edge cases | a `once fn` kept in a field and taken out to be called (`mem.take`); `callback(once)` (W6) |
| tests | `reject_a_function_that_gives_s_away_passed_to_call_twice.em`, `reject_a_plain_once_fn_parameter.em`, `reject_call_twice_given_a_lambda_that_hands_over_the_real_s.em`, `reject_a_once_fn_called_twice.em`, `accept_call_once_holds_an_owns_once_fn_and_call_twice_hands_over_copies.em` |
| expected result | compile-fail E3013 (three: line 1096's second message, word for word; its third, word for word; AN-59's third with the one way out that compiles: `call_twice(fn(): consume(copy of s))`); compile-fail E3040 ("call it once: remove line 10; or ... write `fn call_once(f: fn()):` in place of line 8 and `call_once(fn(): consume(copy of s))` in place of line 17"); run-pass `a`, `b`, `b` |

### R12-23. A function that may end class objects it owns: worked out, and a view is not used after its call

| field | value |
|---|---|
| guarantee | `reset = fn() owns players: players.clear()` may end players, worked out from its body with no word on it or on `main`; a name opened from a link is not used after a call of it (E3060), the message's second way out opening the link under a new name (F-01); nothing runs for it while the program runs. |
| source | R12 lines 681-685; R10 point 1 lines 564-574; R13 (E2106 for a name in use); verdicts 55 and 82 (meaning kept), 86; section 3 line 1069 (E3060, third message); worklist F-01 |
| enforced by | `effects` (what the stored function ends); `borrows` for the opened name across the call |
| today's tests | none (today a captured list's end is not an effect) |
| missing edge cases | a function value stored in a field and run through it (`b.reset()`) across an opened name; a `shares` owner whose last share the function ends |
| tests | `accept_a_function_that_ends_the_players_it_owns.em`, `reject_a_name_opened_from_a_link_used_after_a_call_that_may_end_players.em`, `accept_a_name_opened_from_a_link_used_before_the_call_that_may_end_players.em` |
| expected result | run-pass `gone`; compile-fail E3060 "`reset()` (line 24) may end `Player` objects (it runs the function stored on line 22, which clears the players it owns), and `p` reads one (`if cam.target exists as p:`, line 23) and is used on line 25; use `p` before line 24, or open the link again after it: write `if cam.target exists as q:` above line 25, indent line 25 under it and use `q` there"; run-pass `10`, `gone` |

### R12-24. A function type says whether any thread may run its functions: `thread_shared fn()` (hole 6a)

| field | value |
|---|---|
| guarantee | A list, field or parameter whose functions any thread may run says `thread_shared fn()`, and a function of that type holds `thread_shared` values or copies, never a `shares` or `shares mut` value or a `weakref` link of one (as Rust's `Send`); a class whose objects are made `thread_shared` holds functions only in such types, refused at the line that makes the object (hole 6a's original form). |
| source | R12 lines 692-702; R15 lines 840-845, 872-877; verdicts 61, 94; section 3 line 1103 (E7015, first and second messages); worklist section 3 (E7015's class message) and section 4 (hole 6a's form) |
| enforced by | `typeck` (the function type's `thread_shared` mark and its state's types); today's `Send` inference for closures |
| today's tests | `THR-1/reject_a_sync_class_holding_an_owned_callable_value.em` (E7001 today: a `@sync` class may not hold any callable; **superseded**, `@sync` is gone and a `thread_shared fn()` field is allowed), `THR-10/reject_what_a_thread_may_not_take.em` |
| missing edge cases | a `weakref` link of a one-thread owner held by such a function; the `threads(any)` callback message (W6) |
| tests | `reject_a_shares_mut_owner_held_by_a_function_any_thread_may_run.em`, `accept_a_thread_shared_owner_held_by_a_function_any_thread_may_run.em`, `reject_a_bus_with_plain_fn_handlers_made_thread_shared.em`, `accept_a_bus_with_thread_shared_fn_handlers_made_thread_shared.em` |
| expected result | compile-fail E7015, line 1103's first message in this program's lines (`count = thread_shared Counter()` in place of line 26, the field `n: Atomic[int] = Atomic(0)` in place of line 12, the import, `fn() thread_shared count: count.n.fetch_add(1)`); run-pass `1`; compile-fail E7015 at `bus = thread_shared Bus()` "`Bus.handlers` holds plain `fn()` values, and a `thread_shared Bus` may run them on any thread; write `handlers: Array[thread_shared fn()] guarded by lock = []` in `Bus` in place of line 17, and `fn on_event(owns f: thread_shared fn()):` in place of line 19" (the rules' message, with `on_event`'s line so the fix compiles, R18); run-pass `event` (the fixed program, its handlers run by a thread `spawn` starts) |

### R12-25. A lambda's parameter says what it receives, matching the function type (verdict 71)

| field | value |
|---|---|
| guarantee | Where the type hands the value over (`fn(owns String)`), the lambda writes `fn(owns s)`; where it lends it (`fn(String)`), a plain `fn(s)`, and a lambda that takes over what it is only lent is refused, so it never hands over the caller's value. |
| source | R12 lines 676-680; R4 lines 269-271; verdict 71; section 3 line 1092 (E2020, first and second messages) |
| enforced by | `typeck` (a lambda's parameter modes against the expected function type) |
| today's tests | `CLO-1/accept_a_lambda_giving_a_borrowed_parameter_gives_its_reference.em` (a lambda parameter's mode written as a type, `fn(w: ref String)`) |
| missing edge cases | `fn(ref mut ui)` against `fn(Ui)` (a lambda that would change what it is lent read-only) |
| tests | `reject_a_plain_lambda_parameter_where_each_name_hands_the_name_over.em`, `accept_each_name_hands_each_name_to_a_lambda_that_owns_it.em` (R12-9), `reject_a_lambda_that_takes_over_what_each_name_lends.em`, `accept_a_lambda_keeps_a_copy_of_what_each_name_lends.em` |
| expected result | compile-fail E2020 "`each_name` hands each name over (`fn(owns String)`), so this function receives it as its own: write `fn(owns s) => println(s.len())`" (word for word); run-pass `a`, `3`, `2`; compile-fail E2020 "`each_name` gives each name read-only, so this function cannot take it over; to keep your own, write `fn(s): names.push(copy of s)`" (word for word); run-pass `1` |

### R12-26. Capture words: after the brackets, with no `with`; `ref mut` is not one; `owned fn` is gone

| field | value |
|---|---|
| guarantee | A function says what it holds on each captured name, after its brackets (`fn() copy of name:`, `fn() owns name`); `with` before the captures is E0100 (verdict 83); `ref mut` in the capture list is E0120 (a name not listed is already the real variable); today's `owned fn`, which handed some captures over and silently copied others, is E1010. |
| source | R3 lines 230-245; R12 lines 752-753, 766-767; verdict 83; section 3 lines 1051 (E0100, fourth message), 1085 (E0120), 1113 (E1010, the capture words for `owned fn`); vocabulary `with`, `fn`, `owned fn` rows |
| enforced by | `parser` (the capture list); `typeck` |
| today's tests | `CLO-2/accept_owned_closure_mutates_its_moved_capture.em` (**superseded**: `owned fn`) |
| missing edge cases | `copy of a, b` (E0116, R3, W1's) |
| tests | `reject_with_before_a_lambdas_captures.em`, `accept_the_handler_takes_a_copy_of_the_loop_name.em` (R12-4), `reject_ref_mut_in_a_capture_list.em`, `accept_a_lambda_changes_the_real_variable.em` (R12-3), `reject_owned_fn_names_its_captures.em`, `accept_a_lambda_takes_the_name_it_uses_with_owns.em` (R12-4) |
| expected result | compile-fail E0100 "a function's captures follow its brackets with no `with`: write `b.on_click(fn() copy of name: println(name))`"; run-pass `ann`, `bo`; compile-fail E0120 "remove `ref mut score`: without a word the function already uses the real `score`, and may change it: `bump = fn(): score += 1`"; run-pass `1`; compile-fail E1010 "`owned fn` is not Ember: say what the function takes or copies on each name it uses: `f = fn() owns name => println(name)` (it takes `name` over), or `f = fn() copy of name => println(name)` (a copy of its own)"; run-pass `ann` |

---

## Section 3 refusals and section 2 rows that belong to R12, and where each is tested

| rules.md line | refusal or row | tested by |
|---|---|---|
| 1042 | E3023, a call through a stored function value that may change its own state, from a name that only reads it (two messages) | R12-21 |
| 1043 | E3021, a change of what a read-only name reads, made by a call through a function value | R12-5, R12-11 |
| 1051 | E0100, a clause after a function's brackets (`changes ...`) and `with` before captures | R12-1, R12-2, R12-26 |
| 1060 | E3022, `retain` with a function that changes the list; a function stored where a variable it changes owns it | R12-12, R12-13 |
| 1063 | E3022, a `ref mut` name's place changed by a call through a function value | R12-11 |
| 1065, 1066 | E3020 through a function value, a library call, an adapter, a `dyn` iterator, a foreign call | R12-1, R12-5, R12-7, R12-9, R12-10, R12-11, R12-15 |
| 1069 | E3060: a name opened from a link after a call that may end its object (`reset()`); a view after a stored function replaces its text (`rename()`); a view after the function value owning its node is replaced (`h.keep = ...`); a `ref mut` name of a stored function after the call | R12-23, R12-4, R12-21 (the `remove_all` message is W4's, verdict 82) |
| 1085 | E0120, `ref mut` in a capture list | R12-26 |
| 1092 | E2020, a lambda parameter's mode (two messages) | R12-25 |
| 1094 | E5069 `@ffi(no_callbacks)` | W6 (`V5/conformance/R15/`) |
| 1095 | E3026 (the button handler, the returned function, the loop's name) | R12-16, R12-17 (the task and thread loop-name messages are W6's) |
| 1096 | E3013 | R12-22 |
| 1099 | E7018 (the walk-under-the-lock message) | R12-6 (the join and the plain call are W6's) |
| 1103 | E7015 (the `thread_shared fn()` message and the class message) | R12-24 (the `threads(any)` callback message is W6's) |
| 1113 | E1010, `owned fn` | R12-26 |
| section 4, 1188 | E3040, a `once fn` called again | R12-22 |
| section 2, 985 | a captured variable at its address / an `owns` variable in the function value's own block | R12-18, R12-17 |
| section 2, 986 | two words copied out | R12-19 |
| section 2, 987 | one count on a handler's own state (a C callback ending its own token: W6) | R12-20 |
| section 2, 996 | what every function changes, worked out; `thread_shared fn()` | compile-time only; no run test (nothing to run) |

---

## What step 4 changed (every test, by file name)

### The report's section 1: the 31 R12 tests whose expected error belonged to the removed words

Twelve were made the nearest test of what the program now does:

| step 3's test (expected) | now | why |
|---|---|---|
| `reject_a_lambda_that_pushes_to_log_without_saying_so` (E2105) | `reject_a_lambda_that_pushes_to_log_called_inside_a_loop_over_log` (E3020) | the change the clause named is worked out; the refusal it guarded is the call inside a loop over `LOG`, with the chain |
| `reject_row_6_handler_without_changes_score` (E2105) | `reject_row_6_handler_written_with_changes_score` (E0100) | the program is now row 6's accepted one; the line its message gave (`fn() changes score:`) is the one now refused |
| `reject_an_arrow_lambda_that_changes_removed_without_saying_so` (E2105) | `reject_an_arrow_lambda_written_with_changes_removed` (E0100) | the same for the `=>` form (verdict 45's line) |
| `reject_a_log_handler_in_a_list_of_plain_functions` (E2113) | `reject_a_function_type_written_with_changes_log` (E0100) | the same for a function type (its fix, `Array[fn() changes LOG]`) |
| `reject_a_weakref_handler_that_changes_the_ui_without_changes_ui` (E2105) | `reject_a_view_of_the_ui_across_a_handler_that_changes_it_through_a_weakref` (E3021) | the change through the link is worked out; what it protects is a view of the Ui across the call |
| `reject_a_ui_changing_handler_in_a_list_of_read_only_handlers` (E2113) | `accept_a_handler_that_changes_the_ui_through_its_link_walked_with_for` (run-pass `1`) | the handler changes `clicks`, not `handlers`, so the walk it was refused for is safe and now compiles |
| `reject_an_inspect_adapter_that_changes_log_made_a_dyn_iterator_that_says_nothing` (E2113) | `reject_driving_a_dyn_iterator_that_pushes_to_log_inside_a_loop_over_log` (E3020) | a `dyn` type says nothing; driving it inside a loop over `LOG` is what is refused |
| `reject_click_walks_its_handlers_without_changes_handlers` (E2105) | `reject_click_walks_stateful_handlers_with_a_plain_name` (E3023) | the holder rule stays; its one message is E3023's (`V5/changes.md` section 8, item 4) |
| `reject_press_runs_handlers_without_captured` (E2105) | `reject_press_inside_a_loop_over_the_list_its_handler_changes` (E3020) | the chain `press` → `click` → handler is worked out; refused inside a loop over what the handler changes |
| `reject_each_name_without_a_change_list_given_a_changing_lambda` (E2113) | `reject_each_name_inside_a_loop_over_the_list_its_function_pushes_to` (E3020) | a function that runs the function it is given makes its changes (verdict 21's meaning) |
| `reject_setup_says_changes_log_for_a_handler_it_only_stores` (E2105) | `accept_setup_stores_a_log_handler_inside_a_loop_over_log` (run-pass `start`, `2`) | "storing is not a change" is now shown by what compiles: `setup` inside a loop over `LOG` |
| `reject_tick_calls_poll_events_without_naming_what_the_kept_callback_changes` (E2105) | `reject_tick_inside_a_loop_over_log_may_run_the_callback_c_keeps` (E3020) | `tick`'s change through C is worked out; refused inside a loop over `LOG` |

Nineteen were deleted: after the script took their clause off, each program is word for word an `accept_`
test already in this folder, and nothing else is left to test (the guarantee each guarded is tested by
the row named):

| deleted | why |
|---|---|
| `reject_a_counting_handler_given_to_a_plain_death_handler` (E2113) | a function type says nothing, so the counting handler fits `fn(ref mut Array[Enemy], int)`; the program is the first call of `accept_update_takes_a_death_handler_type` (R12-2) |
| `reject_a_function_that_ends_the_players_it_owns_without_naming_player` (E2105) | the end is worked out; the program is `accept_a_function_that_ends_the_players_it_owns`; the refusal it led to is R12-23's E3060 |
| `reject_a_function_that_says_changes_world_but_never_changes_frame` (E2105) | the named change list is gone with `changes` (verdict 86 voids verdict 41's form; `changelist` is not a word) |
| `reject_a_lambda_that_changes_a_captured_atomic_without_naming_it` (E2105) | verdict 75's spelling is void; the program is `accept_a_lambda_changes_an_atomic_it_uses` |
| `reject_a_lambda_that_names_a_variable_it_never_changes` (E2105) | with no clause there is nothing to be wrong; the program is `accept_a_lambda_that_only_reads_a_variable_it_uses`; a lambda still writing a clause is R12-1's E0100 |
| `reject_a_local_function_that_changes_a_mutex_without_naming_it` (E2105) | the change under the lock is worked out; the program is `accept_a_local_function_that_changes_a_mutex_passed_to_spawn_by_name` |
| `reject_a_local_list_of_plain_functions_given_a_lambda_that_changes_score` (E2113) | the program is `accept_a_local_list_of_functions_that_change_score` |
| `reject_a_log_handler_in_a_button_of_a_module_that_imports_log` (E2113) | verdict 76 (what a file can see by name, for `captured`) is void; the program now compiles (prints 2); the cross-module guarantee is R12-7's |
| `reject_a_log_handler_in_a_button_that_allows_only_captured` (E2113) | the program is `accept_a_button_keeps_a_score_handler_and_a_log_handler` without the score handler |
| `reject_a_thread_function_that_pushes_under_the_log_lock_without_changes_log` (E2105) | the program is `accept_two_threads_push_to_one_log` without the final count; the lock refusal is R12-6's E7018 |
| `reject_a_two_variable_handler_in_a_captured_list` (E2113) | `captured` is gone; the program is `accept_a_handler_that_changes_two_variables_it_uses` |
| `reject_add_line_changes_a_guarded_field_without_changes_log` (E2105) | the program is `accept_add_line_changes_a_guarded_field_through_a_plain_parameter` |
| `reject_an_inspect_lambda_without_changes_seen` (E2105) | the program is `accept_inspect_keeps_the_function_and_count_runs_it` in one line; the refusal is R12-10's E3020 |
| `reject_captured_on_a_lambdas_own_first_line` (E2105) | `captured` is gone; the program is `accept_row_6_handler_changes_the_real_score` |
| `reject_click_says_captured_for_a_list_of_captured_dots` (E2105) | the program is `accept_a_handler_that_changes_two_variables_it_uses` |
| `reject_click_without_captured` (E2105) | the program is `accept_row_6_handler_changes_the_real_score` |
| `reject_click_without_log` (E2105) | the program compiles; the refusal it led to is `reject_clicking_a_log_button_inside_a_loop_over_log` (R12-11) |
| `reject_for_each_given_a_lambda_that_does_not_say_it_changes_log` (E2105) | the program is `accept_for_each_after_the_loop_over_log` without the loop; the refusal is `reject_for_each_inside_a_loop_over_log` (R12-9) |
| `reject_main_starts_a_thread_that_changes_results_without_naming_it` (E2105) | the program is `accept_main_starts_threads_that_push_to_results` without the prints; the refusal of a task started inside a walk of what it changes is W6's (R15) |

### Other deletions

| deleted | why |
|---|---|
| `accept_tick_changes_world` | it declared `world: changelist = (FRAME, LOG)`, which is no longer Ember (verdict 86); without that line it would test nothing R5 does not |
| `accept_for_each_given_a_lambda_that_says_it_changes_log` | without its clause it is `accept_for_each_after_the_loop_over_log` without the loop |
| `accept_a_log_handler_in_a_button_of_a_module_that_does_not_import_log` | verdict 76 is void; without it the test is `accept_a_button_of_another_module_clicked_after_the_loop_over_log` with `LOG` imported instead of declared |
| `log_widgets.em` (module) | used only by the deleted `reject_a_log_handler_in_a_button_of_a_module_that_imports_log` |
| `logbook.em` (module) | used only by the two deleted verdict-76 tests |

### Tests whose refusal came from a function type (verdict 86) and now from the stored functions

| test | change |
|---|---|
| `reject_a_for_walk_of_handlers_given_the_ui_ref_mut` | its only handler (`count`) changes `clicks`, so the walk would now compile; `add_once` (which pushes and removes) is stored instead, and the E3020 names it |
| `reject_a_for_walk_of_handlers_whose_type_changes_ui` → `reject_a_for_walk_of_handlers_one_of_which_clears_them_through_a_weakref` | it stored no handler (the type said `changes Ui`); now G-05's handler, which clears the list through its `weakref ui`, is stored |
| `reject_a_plain_loop_name_calls_a_stored_function` → `accept_a_plain_loop_name_calls_a_function_with_no_state_of_its_own` | its handler has no state of its own, so the plain name is now allowed (`V5/changes.md` section 8, item 4); the stateful case is `reject_click_walks_stateful_handlers_with_a_plain_name` |
| `reject_a_plain_name_from_get_mut_calls_the_handler`, `accept_a_ref_mut_name_from_get_mut_calls_the_handler` | their handler `count` had no state, so the plain name would compile; both now store `step`, which keeps its own copy of `n` (the accept prints `3`) |
| `reject_the_handler_name_used_again_after_the_call` | its handler could not remove handlers; `close_all` is stored instead, so `h` may name another handler after the call |

### Renamed, with the program changed to be its `reject_` neighbour's fix, or only its words

`accept_a_lambda_that_says_it_pushes_to_log` → `accept_a_lambda_that_pushes_to_log_called_after_the_loop` (a loop
over `LOG` first); `accept_an_arrow_lambda_says_what_it_changes_after_its_parameters` →
`accept_an_arrow_lambda_changes_a_variable_it_uses`; `accept_a_plain_function_fits_a_list_of_log_handlers` →
`accept_named_functions_kept_in_a_list_of_fn`; `accept_a_lambda_that_only_reads_a_captured_variable_is_a_plain_fn`
→ `accept_a_lambda_that_only_reads_a_variable_it_uses`; `accept_a_lambda_names_the_captured_atomic_it_changes` →
`accept_a_lambda_changes_an_atomic_it_uses`; `accept_a_two_variable_handler_in_a_captured_dots_list` →
`accept_a_handler_that_changes_two_variables_it_uses`; `accept_a_button_keeps_captured_and_log_handlers` →
`accept_a_button_keeps_a_score_handler_and_a_log_handler`; `accept_a_local_list_names_the_variable_its_functions_change`
→ `accept_a_local_list_of_functions_that_change_score`; `accept_a_weakref_handler_says_changes_ui` →
`accept_a_weakref_handler_changes_the_ui_and_a_copy_keeps_the_count` (keeps `copy of ui.clicks`, prints `0 1`);
`accept_two_threads_push_to_one_log_and_say_changes_log` → `accept_two_threads_push_to_one_log`;
`accept_a_local_function_names_the_mutex_it_changes` →
`accept_a_local_function_that_changes_a_mutex_passed_to_spawn_by_name`;
`accept_a_button_of_another_module_keeps_a_log_handler_under_captured` →
`accept_a_button_of_another_module_clicked_after_the_loop_over_log` (a loop over `LOG` first);
`accept_press_says_captured_and_game_names_nothing` → `accept_press_after_the_loop_over_the_list_its_handler_changes`
(the `names` program); `accept_each_name_takes_a_change_list` → `accept_each_name_hands_each_name_to_a_lambda_that_owns_it`
(a loop over `names` first; `[C]` gone); `accept_update_takes_a_death_handler_type_with_a_change_list` →
`accept_update_takes_a_death_handler_type` (`type DeathHandler = fn(ref mut Array[Enemy], int)`, `[C]` gone);
`accept_main_names_results_for_the_threads_it_starts` → `accept_main_starts_threads_that_push_to_results`;
`accept_a_dyn_iterator_that_says_changes_log` → `accept_driving_a_dyn_iterator_that_pushes_to_log_after_the_loop`;
`accept_tick_names_what_the_kept_callback_changes` → `accept_tick_after_the_loop_over_log_may_run_the_callback_c_keeps`;
`accept_click_says_changes_handlers_and_walks_ref_mut` → `accept_click_walks_stateful_handlers_with_ref_mut`;
`accept_a_function_that_ends_the_players_it_owns_names_player` → `accept_a_function_that_ends_the_players_it_owns`;
`accept_a_lambda_says_it_changes_the_real_variable` → `accept_a_lambda_changes_the_real_variable`;
`accept_a_lambda_says_it_takes_the_name_it_uses` → `accept_a_lambda_takes_the_name_it_uses_with_owns`.

### Changed in place

- Verdict 84 (worklist F-02): `reject_a_returned_function_that_changes_its_makers_local`,
  `accept_a_returned_function_owns_the_variable_it_keeps` and `run_a_counter_made_by_a_function_keeps_its_variable`
  keep an `int` `n` with verdict 80's own line (`return fn() owns n:`), where step 3 kept a `struct Tally` to avoid
  E2085, now removed.
- Change-list type parameters left by the script: `[C]` removed from `reject_a_lambda_that_takes_over_what_each_name_lends`,
  `accept_a_lambda_keeps_a_copy_of_what_each_name_lends` and `reject_a_plain_lambda_parameter_where_each_name_hands_the_name_over`;
  `[C, F: fn(int)]` became `[F: fn(int)]` and `[C]` went in
  `accept_a_programs_own_function_takes_its_function_as_a_template_parameter`.
- `reject_ref_mut_in_a_capture_list`: `fn() with ref mut score:` (the script, rightly, left a line that is no
  capture it keeps) became `fn() ref mut score:` (verdict 83), so the line has one fault, E0120.
- `reject_a_plain_lambda_parameter_where_each_name_hands_the_name_over`: the same `main` as its accept (a loop
  over `names` first).
- Every message to the v5 words and R18: chains of calls in every E3020, E3021, E3022 and E3060 message
  about a call; the line to write and where (F-12: the three `b.click()` messages and the others that
  said "after the loop" with no line); the rules' v5 text where it gives one (E3022 `retain`, E3026, E3013,
  E2020, E7015, E3023, E7018), in this program's lines; F-01's `q` kept in the `reset()` message. Every
  `##` comment cites v5's line numbers and quotes the guarantee in v5's words; no comment or message
  writes a `changes`, `reads` or `captured` clause, a change list, or `with` before captures, except the
  three E0100 tests and `reject_with_before_a_lambdas_captures`, whose refused line is that very spelling.

### Added

| test | why |
|---|---|
| `reject_with_before_a_lambdas_captures` | verdict 83's new refusal (E0100); its accept is `accept_the_handler_takes_a_copy_of_the_loop_name` |
| `reject_a_view_through_a_link_used_after_a_stored_function_replaces_its_text`, `accept_a_view_through_a_link_used_before_a_stored_function_replaces_its_text` | step 3's hole 3, part 1 (worklist section 4) |
| `reject_a_view_of_a_node_used_after_the_function_value_owning_it_is_replaced`, `accept_a_view_of_a_node_used_before_the_function_value_owning_it_is_replaced` | step 3's hole 3, part 2 (worklist section 4) |
| `reject_a_bus_with_plain_fn_handlers_made_thread_shared`, `accept_a_bus_with_thread_shared_fn_handlers_made_thread_shared` | hole 6a's original form (worklist sections 3 and 4) |

### The script's lines

The report's section 3 lists 295 changed lines in R12's files; each was checked against the test it
sits in. None was wrong: every `changes ...`, `reads ...` and `with` it took off a code line was a clause
or a capture's `with`. What it could not do (messages, `##` comments, `#$ help:` lines, change-list type
parameters, the tests of its section 1) is done above.

### Worklist section 4 items for T3

| item | where |
|---|---|
| F-01 (`q` in E3060's "open the link again") | `reject_a_name_opened_from_a_link_used_after_a_call_that_may_end_players` (R12-23) |
| F-02 (settled by 84) | R12-17's three tests use an `int` |
| F-03 (settled by 85) | no R12 test makes a value for an `Option[owns T]` or an `owns` list element |
| F-12 (the exact line in every message) | every message of this folder; the three `b.click()` tests by name (R12-7, R12-11) |
| hole 3, parts 1 and 2 | R12-4 |
| hole 6a's form | R12-24 |

## For the owner

None. Every point met in this folder is settled by a verdict, by the worklist or by `V5/changes.md`
section 8 (item 4 decides which stored functions make their holder change).

## Added or changed by the lead after step 4's checks

- `R12/reject_a_view_of_a_node_owned_by_a_replaced_dyn_value.em` (E3060, an `owns dyn I` value replaced)
- `R12/reject_replacing_a_function_whose_node_drop_pushes_to_the_walked_list.em` (E3020, a node's `drop` run by replacing a function value inside a walk)
