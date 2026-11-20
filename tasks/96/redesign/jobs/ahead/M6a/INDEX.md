# M6a ahead: tests and spec questions, written before the builders

Written from `docs/spec-source/Ember_v0.9.20_Hardened_14.md` alone (`355cf49f`, tag `spec-h14`, sha256
`F88C7977…`) by the Opus deputy D2, one milestone ahead of the builders, on 2026-11-20. M6a is
MILESTONES.md's "Calls checked by the summaries; one name for one variable; messages that name the
chain": A4-4 to A4-7, A2-10, A2-13, A1-21, A1-22, A3-6 to A3-8, A7-14. **Batch 1** (merged) covered 15
of the rules those items complete; **batch 2** (the same day) the 29 the lead listed after it: RC-5,
BCK-7, the EXC rules of A4-6, WK-3, BCK-1 to BCK-3, BCK-5, BCK-6, BCK-8, LT-35, LT-47, BRW-22's copy
clause and the DIA rules of A7-14. Rules already credited (`docs/phase-status.json`, `built_for_0915`)
and tested in `tests/conformance` were skipped: WK-2, CLS-7a, DIA-16; BCK-4 is M5's (`ahead/M5/BCK-4`).
§2 says what is left. Nothing was built:
every expectation is the spec's words and codes, never the compiler's. The files fail until M6a is
built; when a rule is built, its directory moves to `tests/conformance/<RULE>/` and is checked with
`python tasks/impl-0.9.9/annotations.py tests/conformance/<RULE>`. No hardening is cut here.

**Conventions** (M5's). Each file starts with `# Clause <RULE>.<n>: "<the clause, quoted>"`. One file
per clause and outcome (`accept_`, `reject_`); a file holds several cases. A refusal's text is the
spec's own example text where the spec gives one for that program, with the file's own line numbers
in place of the example's; where it gives none, the expectation is `#$ error[CODE]` with no text
("none" below). Line numbers in expectations were resolved by a generator from markers, not by hand.
A **case** is one expectation: one `#$ error[...]` or one `#$ stdout:` line. A file whose expectation
rests on a reading of §3 says so in its header comment.

**The probe** (`target/release/ember.exe check`, one file at a time, no cargo). `static mut` with no
`unsafe` is not built and stops the check of what follows, so each file was also checked as a copy
with `static mut` read as `static`, to show the faults it hides. Faults found and fixed before each
commit, keeping what the spec says:
- `Tree(owns root)`: a constructor's argument takes no `owns` (`[FN-2a]`, `E0109`): `Tree(root)`.
- BRW-11: `keep`'s parameter was a `StrView` where `retain` gives a `String`; a pair of list-element
  names made without a proof (refused at the second name, M5's Q17) was replaced by the `Map` case of
  the spec's own `blend`; `owns a` of a `ref mut int` became `owns 0`.
- EFF-26: unused loop names (`L1001`); one local given two adapters of different lambda types.
- CLO-18: block-bodied lambdas inside call arguments (no such form in the spec) became one-line bodies.

Every other failure is a feature not built: `static mut` with no `unsafe` (M2, A1-17); the summaries
and the call check (this milestone); a class method being a change of its whole object today (A4-7);
`ref mut this` given to a call (E2103 today, W8's case); one whole variable under two names (E3022
today, A1-22/A2-13); the `i + 1..` and `if i != j:` proofs (M5's A2-4); `assert_disjoint` as `[DSJ-5]`
says (M6c); a function lent to an `F`-bounded parameter (`retain(keep)`, `[FN-2]`); capture words and
`[CLO-4]`'s "provably ends before" (M8, E0900 and E3026 today); and `this.handlers[i](...)` read as a
call of an element (today a method with a type argument, E2072: Q8). Three accept files check clean
today (`FN-9/accept_a_ref_mut_class_parameter_may_change_and_replace`,
`FN-9/accept_a_changing_method_through_a_ref_mut_p`,
`BRW-21/accept_a_call_through_a_function_value_whose_functions_keep_the_promise`); none was run.

**Batch 2's probe.** One fault fixed (`copy of` a span returned as an `Array`, LT-47); every other
failure is not built: the compile-time checks for links and class objects (today run-time, or E3021
where the spec says E3060 for a reshape), positions across a push, a method's object reserved, a
`Map` walked `for k, ref mut v in`, `sharedref mut` (M5's A2-8), the message texts (E3040's label,
E0100's and E0113's texts, `int` printed `i64`: Q10), and M6c's `assert_disjoint`. WK-3's file checks
clean today; none was run.

**Totals.** 44 rules, 77 clauses, 119 files (54 accept, 65 reject), 285 cases. Batch 1: 15 rules, 42
clauses, 69 files (34 accept, 35 reject), 159 cases: 68 error expectations (16 with the spec's text,
52 code only) and 91 output lines. Batch 2: 29 rules, 35 clauses, 50 files (20 accept, 30 reject),
126 cases: 64 error expectations (18 with the spec's whole text; 9 pinned in part, to the words a DIA
rule names, such as "used here" or `` `i64` ``; 37 code only), 56 output lines and 6 `#$ help:` or
`#$ not-help:` lines, which batch 2 counts as cases.

## 0. Rules covered, and what else each needs before it can be credited

### Batch 1

| rule | clauses | files | cases | also needs (beyond M6a's own items) |
|---|---|---|---|---|
| BRW-21 | 9 | 14 | 34 | M2 (`static mut`); not written: the package edge (a two-package test, `[EFF-4]`), a foreign call's open lines (M12, `[FFI-52]`), leaving a lock's `with` block (M9) |
| BRW-11 | 4 | 8 | 20 | M2; M5's A2-4 (`i + 1..`, `if i != j:`); M6c (`assert_disjoint`, `[DSJ-5]`); `[FN-2]`'s lent function (`LOG.retain(keep)`); not written: exception 2 (M9) |
| MONO-10 | 4 | 5 | 12 | M2; not written: an `extern` function (M12), `ember inspect`'s list, a `pub` function's body in the interface file |
| BRW-10 | 2 | 3 | 8 | none; not written: the package edge (two-package test) |
| EFF-25 | 3 | 4 | 11 | M2, M4 (`weakref`); not written: `E7018` and tasks (M9), `E2100` and `@cached` (M11) |
| EFF-26 | 4 | 4 | 11 | M2; not written: `spawn`, scopes, jobs (M9), `callback(retained|once)` (M12) |
| CLO-17 | 2 | 5 | 10 | M2; M8 (`[CLO-4]`'s keeps facts: a lambda holding a real variable stored in its holder is E3026 today); not written: the `weakref bus` capture (M8) |
| CLO-18 | 2 | 3 | 4 | M8 (capture words); `[FN-2]`'s lent function |
| DRP-8 | 1 | 3 | 7 | M2; not written: a function value or `owns dyn I` replaced (M8, M11) |
| BRW-18 | 1 | 1 | 2 | M2 (the call clause only; the own-name cases are A2-5's) |
| EXC-15 | 1 | 2 | 4 | M2 (the call clause; its `E3023` is M5's BRW-15.2 and CLS-7.1 here; the turret is BRW-21.4) |
| CLS-7 | 2 | 5 | 15 | M2; Q8 (an element call through a field) |
| FN-9 | 4 | 7 | 14 | M2 |
| OBJ-5 | 2 | 3 | 3 | M2, M4 (`weakref`); M9 (`thread_shared this`, its own file) |
| DSP-5 | 1 | 2 | 4 | M2; not written: through `dyn` and interface values (M11). Its first sentence is built (`tests/conformance/DSP-5`) |

### Batch 2

| rule | clauses | files | cases | also needs (beyond M6a's own items) |
|---|---|---|---|---|
| RC-5 | 1 | 3 | 8 | M8 (the stored-function file's capture words); not written: point 2 (`weakref thread_shared`, M9), the hand-over to a thread (M9) |
| BCK-7 | 2 | 4 | 7 | none |
| EXC-1 | 1 | 2 | 6 | none |
| EXC-2 | 1 | 2 | 3 | none |
| EXC-3 | 1 | 2 | 5 | none |
| EXC-16 | 1 | 2 | 6 | none |
| EXC-17 | 2 | 2 | 4 | M5's A2-8 (`sharedref mut`) |
| EXC-19 | 1 | 1 | 4 | none (the method case is A4-7's summary) |
| WK-3 | 1 | 1 | 3 | not written: a `thread_shared` value ended at its making thread's check point (M9) |
| BCK-1 | 1 | 2 | 6 | none (positions are M5's BRW-16) |
| BCK-2 | 1 | 2 | 5 | none (a view last used as an argument: BRW-21.1) |
| BCK-3 | 1 | 2 | 6 | M2 (`static mut`); the rest of the clause is M5's BRW-1, BRW-17, BRW-18 files |
| BCK-5 | 1 | 2 | 5 | none |
| BCK-6 | 1 | 1 | 6 | M5's A2-5 (a position across a push) |
| BCK-8 | 2 | 3 | 3 | M2; M5's A2-5; not written: which build checks a line (`[EFF-2a]`, two packages) |
| LT-35 | 1 | 2 | 3 | not written: a foreign function's every field used and kept (M12) |
| LT-47 | 2 | 3 | 6 | none |
| BRW-22 | 1 | 1 | 3 | M2; BRW-22.1 (noalias on every `ref mut`) is M6c's |
| DIA-2 | 1 | 1 | 4 | Q10 |
| DIA-3 | 1 | 1 | 3 | none |
| DIA-7 | 1 | 1 | 6 | M2 |
| DIA-9 | 1 | 1 | 4 | M2 |
| DIA-10 | 1 | 1 | 4 | M8 (B9's message offers a capture word) |
| DIA-11 | 1 | 1 | 4 | M6c (`assert_disjoint`, `@no_alias_checks`' check) |
| DIA-22 | 1 | 1 | 3 | M2 |
| DIA-25 | 2 | 2 | 3 | not written: the runner applying every machine-applicable fix (A7-14's tooling), PAR-1's several-line replacement (M9) |
| DIA-26 | 1 | 1 | 1 | none |
| DIA-27 | 2 | 2 | 3 | M2; Q9 |
| DIA-28 | 1 | 1 | 2 | M2; not written: `E7018` over `E3020` and `E3020` over `E7007` (M9) |

## 1. Rules, clauses and tests

Message column: "spec" = the spec's text; "none" = code only.

### BRW-21 (VII.4) — the call check

| clause | what it says | test | message |
|---|---|---|---|
| 21.1 | a view last used as an argument does not live across the call; the callee is held to the parameter's promise | `accept_a_view_whose_last_use_is_an_argument_does_not_live_across_the_call` | — |
| 21.2 | a plain or `Span` parameter's target is not ended or moved: E3031 at the call | `reject_a_plain_or_span_parameter_whose_target_the_callee_ends_or_moves` (`note(LOG[0])`; a push that may move a span); `accept_a_call_the_callee_keeps_its_promise_for` (`note("bye")`, print-then-clear) | spec (`note`); none (span) |
| 21.3 | the fact comes through callees, function values, overrides and ends of objects, with the body's precise facts | `reject_the_fact_through_callees_function_values_overrides_and_ends` (4); `accept_a_change_inside_is_not_is_not_one_for_its_name` | none |
| 21.4 | writing a number or `bool` ends and moves nothing | `accept_writing_a_number_ends_and_moves_nothing` (the turret; a write showing through, M5's Q9) | — |
| 21.5 | the object of a method counts the same way | `reject_a_method_whose_object_it_may_end` (`ENEMIES[0].hit()`); `accept_a_method_that_uses_this_only_before_the_end` | spec |
| 21.6 | a `ref mut` parameter counts the same way | `reject_a_ref_mut_parameter_whose_place_the_callee_ends` (`rename(ref mut LOG[0])`) | spec |
| 21.7 | a `ref mut` parameter's place is reached by no other name while it is used | `reject_a_ref_mut_argument_the_callee_reaches_by_another_name` (`heal(ref mut PARTY[0])`; a write by name); `accept_a_ref_mut_parameter_reached_by_no_other_name_while_used` | spec (`heal`); none |
| 21.8 | unless one whole variable (BRW-11 exception 1) | `accept_the_same_whole_variable_is_compiled_against_a_copy` (`bump(ref mut COUNT)` prints 2) | — |
| 21.9 | a stored function's fault is refused where it is stored, not at the call through the value | `reject_a_stored_function_is_refused_where_it_is_stored` (`t.on_visit = prune`); `accept_a_call_through_a_function_value_whose_functions_keep_the_promise` | none (the spec's text is for its own tree program) |

### BRW-11 (VII.4) — arguments do not overlap a changeable one

| clause | what it says | test | message |
|---|---|---|---|
| 11.1 | two arguments, one changeable (`ref mut`, a changing method's object, a function value), name one place or reach into each other only where proved apart; else E3022 (both change) or E3021 | `reject_two_arguments_one_changeable_that_may_name_one_place` (`f(ref mut v, v[0])`, `w.add_copy(w.names[0])`, a function value, a captured real variable, unproved indexes); `accept_two_arguments_proved_different` (constants, `i + 1..`, `if i != j:`) | none |
| 11.2 | two spans of one list, one changeable: proved apart or refused | `reject_two_spans_of_one_list_not_proved_apart` (`blend`); `accept_two_spans_of_one_list_proved_apart_or_copied` | spec |
| 11.3 | a function value given to a call that changes the call's object | `reject_a_function_value_that_changes_the_calls_object` (`LOG.retain(keep)`, E3022); `accept_the_lines_the_retain_message_gives` | spec |
| 11.4 | exception 1: one whole variable under two names, compiled against a copy; unknown positions stay E3022 | `accept_the_same_whole_variable_under_two_names` (`add_both` 2, `rename(ref mut LOG)`); `reject_parts_whose_positions_are_not_known` | none |

### MONO-10 (XVII) — one whole variable under two names

| clause | what it says | test | message |
|---|---|---|---|
| 10.1 | the call is compiled against a copy in which the two names are one place | `accept_a_copy_computes_what_its_lines_say_for_one_variable` (a module variable twice; `tally(ref mut TOTAL)` prints 11) | — |
| 10.2 | other calls use the ordinary body | `accept_other_calls_use_the_ordinary_body` | — |
| 10.3 | the copy is made down the chain | `accept_the_copy_is_made_down_the_chain` | — |
| 10.4 | where no body can be had (function value, `virtual`, `extern`): E3022 | `reject_one_variable_twice_where_no_body_can_be_had`; `accept_the_ways_out_of_the_function_value_message` | spec (function value); none (`virtual`) |

### BRW-10 (VII.4) — what a method does to its object

| clause | what it says | test | message |
|---|---|---|---|
| 10.1 | a call changes only the fields its body and its callees change, private or `pub` | `accept_a_method_call_changes_only_the_fields_its_body_changes` | — |
| 10.2 | `this.note(...)` in a loop over `this.log` is E3020, over `this.players` allowed | `reject_a_method_that_changes_the_walked_field` (through `this` and a name); `accept_a_method_that_changes_another_field_than_the_walked_one` | none |

### EFF-25 (X) — the summary decides

| clause | what it says | test | message |
|---|---|---|---|
| 25.1 | a message names the chain of calls; no message asks for a word on a first line | `reject_a_change_a_call_makes_names_the_chain` (the spec's program); `accept_the_line_the_chain_message_gives` | spec |
| 25.2 | it decides E3020, E3021, E3031, E3060 (a link's object), E3022 (a stored function), … | `reject_the_summary_decides_each_rule_through_a_call` (one per code) | none |
| 25.3 | a fact from a body never changes what a line does | `accept_a_fact_from_a_body_changes_only_whether_a_line_compiles` | — |

### EFF-26 (X) — a call that runs a function makes its changes

| clause | what it says | test | message |
|---|---|---|---|
| 26.1 | a call through a function value makes its functions' changes | `reject_a_call_through_a_function_value_makes_its_functions_changes` (a local, a field, a parameter) | none |
| 26.2 | a call that runs a function by itself: `for_each`, whatever drives an adapter | `reject_a_call_that_runs_the_function_it_is_given` | spec (`for_each`); none (adapters) |
| 26.3 | storing a function value changes nothing | `accept_storing_a_function_value_changes_nothing` (a local, a field, a list, an adapter) | — |
| 26.4 | a generic body makes its type parameters' changes, per instance | `reject_a_generic_body_makes_its_type_parameters_changes_per_instance` (`run(Quiet())` allowed, `run(Loud())` E3020) | none |

### CLO-17 (VI) — what a function value does

| clause | what it says | test | message |
|---|---|---|---|
| 17.1 | a call through a function value makes what every function the program can put there does; a library's `Button` with a handler that pushes to `LOG` | `reject_a_handler_a_button_holds_makes_its_click_a_change`; `accept_the_click_after_the_loop` | spec (`[BRW-18]`'s) |
| 17.2 | not stored in a place its body may reshape or end, or owned all the way down by one, through any name | `reject_a_function_value_stored_where_its_run_would_change_its_holder` (`on_click`, `BUS`, `ref mut u`, owned all the way down through a call); `reject_a_tree_visit_that_clears_the_roots_children` (Q3); `accept_a_function_value_whose_run_leaves_its_holder_alone` | spec (`on_click`); none |

### CLO-18 (VI) — a function value that changes its own state

| clause | what it says | test | message |
|---|---|---|---|
| 18.1 | a call through a stored stateful function is a change of its holder: a plain `h` is E3023 | `reject_a_stateful_handler_called_through_a_read_only_name` | spec |
| 18.2 | such a value given to a read-only parameter is E3023 (and `[FN-2]`'s `apply`) | `reject_a_stateful_function_value_given_read_only`; `accept_the_lines_the_messages_give` (both clauses) | spec (both) |

### DRP-8 (VII.8) — ending a value counts as a call of its `drop`

| clause | what it says | test | message |
|---|---|---|---|
| 8.1 | anything that may end a value with a `drop` makes that `drop`'s changes | `reject_a_clear_makes_the_changes_of_the_drops_it_runs` (the spec's `enemies.clear()`); `reject_every_way_a_value_may_end_makes_its_drops_changes` (a replacement, a local ending in a callee, a function given the list); `accept_the_clear_after_the_loop` | spec; none |

### BRW-18 (VII.5) — the call clause

| clause | what it says | test | message |
|---|---|---|---|
| 18.1 | nothing changes a walked collection through any call that may change it; a stored function that may change the walked list | `reject_a_stored_function_that_may_change_the_walked_list` (`h(this)`, W8's line in the message; the store is E3022 by Q4) | spec (E3020); none (E3022) |

### EXC-15 (VIII.3) — the call clause

| clause | what it says | test | message |
|---|---|---|---|
| 15.1 | a call whose object is part of what the method may end before its last use of `this` is E3031 | `reject_a_call_whose_object_the_method_may_end_before_its_last_use_of_this` (through callees; directly and through a part name); `accept_a_call_whose_object_is_not_part_of_what_the_method_ends` | none |

### CLS-7 (VIII) — `this` in a class method

| clause | what it says | test | message |
|---|---|---|---|
| 7.1 | a method whose body changes `this` is not called through a name that only reads (E3023); one that neither changes nor ends only reads | `reject_a_changing_method_through_a_name_that_only_reads` (the change through a callee; a parameter, a part name, a `ref` name, a loop name); `accept_a_reading_method_through_a_name_that_only_reads` | none |
| 7.2 | `this` is the object for the whole call: W8's case | `accept_ref_mut_this_given_to_calls` (`bump(ref mut this)`, `this.handlers[this.current](ref mut this)`); `reject_ref_mut_this_given_to_a_call_that_may_end_its_object` (E3031 at `UIS[0].shut()`, Q6); `reject_ref_mut_this_given_to_a_call_that_gives_it_another_object` (E2103, Q1) | none |

### FN-9 (VI) — a class-type parameter's word

| clause | what it says | test | message |
|---|---|---|---|
| 9.1 | `p: Player` reads; the object neither changes nor ends while `p` is used | `reject_a_plain_class_parameter_is_neither_changed_through_nor_ended` (E3023, E3031); `accept_a_write_by_another_name_shows_through_p` (Q5) | none |
| 9.2 | `ref mut p` may change it, and `p = Player()` gives the caller's place another object, the old one ending | `accept_a_ref_mut_class_parameter_may_change_and_replace` (a local and a list element; `drop` prints) | — |
| 9.3 | `owns p` hands it over; the caller's name ends | `reject_an_owns_class_parameter_ends_the_callers_name` (E3040); `accept_an_owns_class_parameter_keeps_the_object` | none |
| 9.4 | a changing method is not called through a `p` that only reads | `reject_a_changing_method_through_a_p_that_only_reads`; `accept_a_changing_method_through_a_ref_mut_p` | none |

### OBJ-5 (VIII.1) — everything a `drop` calls

| clause | what it says | test | message |
|---|---|---|---|
| 5.1 | inside a `drop` and everything it calls, no owner of the object is made and it is not stored (E3016) | `reject_a_drop_that_calls_what_stores_this` (Q7); `reject_a_drop_that_calls_what_makes_an_owner_of_this` (`thread_shared this`, M9) | none |
| 5.2 | `weakref this` stays allowed; opening it finds the object ended | `accept_a_drop_that_calls_what_keeps_a_weakref` | — |

### DSP-5 (VIII.4) — the second sentence

| clause | what it says | test | message |
|---|---|---|---|
| 5.2 | what a `virtual` call may change is worked out from every override | `reject_a_virtual_call_makes_what_every_override_changes`; `accept_a_virtual_call_whose_overrides_change_nothing_walked` | none |

### Batch 2: RC-5 (VIII.2) — a view of an object reached through a link

| clause | what it says | test | message |
|---|---|---|---|
| 5.1 | no line or call in the view's life may end an object of its class family, held directly or through owners, nor a place of a function type whose state owns one: E3060 at the later use | `reject_a_line_or_call_that_may_end_the_object_a_link_view_reads` (`remove_all`; a base-class list; a list of teams); `reject_a_stored_function_that_may_end_the_object_a_link_view_reads` (`reset()`, `h.keep = fn(): pass`; M8); `accept_the_ways_out_and_ends_of_other_families` | spec (all three); none |

### BCK-7 (XVIII.4) — links

| clause | what it says | test | message |
|---|---|---|---|
| 7.1 | two paths through links follow BCK-3's rule for links | `reject_two_paths_through_links_may_reach_one_object`; `accept_two_paths_through_links_proved_apart` | none |
| 7.2 | a link view lives while nothing may end its family (E3060); across a call that may change one it does not live (E3021); a walk keeps a `weakref` | `reject_a_link_view_across_a_call_that_may_change_or_end_its_object`; `accept_a_walk_across_such_a_call_keeps_a_weakref_link` | none |

### EXC-1, EXC-2, EXC-3, EXC-16, EXC-17, EXC-19 (VIII.3) — changes through several names

| clause | what it says | test | message |
|---|---|---|---|
| EXC-1.1 | a field changed through another name while viewed: E3021; reshaped: E3060; while a `ref mut` name of it is used: E3022 | `reject_a_field_changed_through_another_name_while_viewed`; `accept_a_field_changed_after_the_views_last_use` | none |
| EXC-2.1 | a field read through another name while a `ref mut` name of it is used: E3021 | `reject_a_field_read_through_another_name_while_a_ref_mut_name_is_used`; `accept_a_field_read_through_its_ref_mut_name_or_after_it` | none |
| EXC-3.1 | one class family and a link or kept view: may be one unless known different; refused with the `is not` line | `reject_a_link_that_may_reach_the_only_name_of_its_object` (the turret, E3022; a base-class link, E3021); `accept_paths_the_compiler_knows_differ` (`is not`, unrelated classes, a fresh object) | spec (turret); none |
| EXC-16.1 | a memory-owning field given a new value is a reshape: E3020 in a walk of it, E3060 for a view of the old value | `reject_giving_a_memory_owning_field_a_new_value_is_a_reshape` (directly and through `reset`); `accept_a_new_value_after_the_walk_and_the_views_last_use` | none |
| EXC-17.1 | a number or `bool` write is not a reshape | `accept_writing_a_number_field_is_not_a_reshape` | — |
| EXC-17.2 | such a write through another name while a read-only view of the field is used: E3021, a `sharedref mut` link's included | `reject_a_number_written_through_another_name_while_viewed` | none |
| EXC-19.1 | fields are separate places; a method touches the fields its body touches | `accept_fields_are_separate_places` (the spec's loop, and through a method) | — |

### WK-3 (VIII.5) — ended before its `drop`

| clause | what it says | test | message |
|---|---|---|---|
| 3.1 | a link opened in the object's own `drop` or its child's finds it ended; ending the link stays allowed | `accept_an_object_is_ended_before_its_drop_runs` | — |

### BCK-1, BCK-2, BCK-3, BCK-5, BCK-6, BCK-8 (XVIII.4) — the checking algorithm

| clause | what it says | test | message |
|---|---|---|---|
| BCK-1.1 | values carry the views they may hold, through copies, projections, results and views put in through `ref mut` | `reject_a_value_carries_the_views_it_may_hold` (4); `accept_a_value_carries_only_the_views_it_may_hold` (`from a`) | none |
| BCK-2.1 | live where a value carrying it is used later on some path, with no assignment between; a result's views on the paths to the `return` only | `accept_liveness_follows_uses_and_paths`; `reject_a_view_live_on_some_path` | none |
| BCK-3.1 | a read or change made by a call, through its summary, is checked against the live views | `reject_a_read_or_change_made_by_a_call_is_checked_against_live_views` (E3021, E3060, E3021, E3022); `accept_a_call_that_reaches_no_live_views_place` | none |
| BCK-5.1 | a changeable argument is reserved, then activated; then the arguments are checked against each other | `accept_a_changeable_argument_is_reserved_then_activated` (`v.push(owns v.len())`); `reject_a_change_of_a_reserved_place_and_arguments_that_overlap` | none |
| BCK-6.1 | the four required acceptances | `accept_the_required_acceptances` (`first_even`; the conditional store, Q11) | — |
| BCK-8.1 | a list parameter only added to keeps a position usable | `accept_a_list_parameter_it_only_adds_to_keeps_a_position`; `reject_a_callee_that_removes_ends_the_position` | none |
| BCK-8.2 | the least fixed point covers recursion | `reject_a_recursive_group_is_covered_by_the_fixed_point` | none |

### LT-35, LT-47 (VII.6) — regions through calls

| clause | what it says | test | message |
|---|---|---|---|
| LT-35.1 | a call needs only the regions of the fields its callee uses; through a function value, every function put there counts | `reject_a_call_through_a_function_value_needs_what_every_function_uses`; `accept_a_call_needs_only_the_regions_of_the_fields_it_uses` | none |
| LT-47.1 | only a view argument, a part of one or program text may be put in through `ref mut`; a callee's local is E3063 | `reject_a_view_of_the_callees_own_local_put_in_through_ref_mut`; `accept_the_views_a_callee_may_put_in` | spec |
| LT-47.2 | the put view may point into any other argument; used after one changed: E3060 | `reject_a_use_of_the_put_view_after_an_argument_changed` | spec |

### BRW-22 (VII.4) — the copy clause

| clause | what it says | test | message |
|---|---|---|---|
| 22.2 | a call compiled against a copy has one name for the place | `accept_a_call_against_a_copy_has_one_name_for_the_place` (every profile) | — |

### DIA-2, 3, 7, 9, 10, 11, 22, 25, 26, 27, 28 (XVII.6) — the message framework

| clause | what it says | test | message |
|---|---|---|---|
| DIA-2.1 | names as the program wrote them: `i64`, `[int; 3]`, `int` for `len()`, the caller's variable in the line to write | `reject_a_refusal_names_what_the_program_wrote` | the names only (Q10) |
| DIA-3.1 | view errors label the later use, "used here" | `reject_a_view_error_labels_the_later_use` (E3060, E3040, E3021) | the label only |
| DIA-7.1 | each error emits its shape's help | `reject_each_shape_emits_its_help` (B1, B2, B3, B4, B7, B16) | spec (all six) |
| DIA-9.1 | no `unsafe`, no `shares` alone, where a safe one-owner fix compiles | `reject_no_help_suggests_unsafe_or_shares_where_a_safe_one_owner_fix_compiles` | none; `not-help` |
| DIA-10.1 | B1, B2, B4, B5, B9 helps name a construct, never "consider restructuring" | `reject_a_help_names_a_concrete_construct` (B2, B4, B9) | spec (three); `not-help` |
| DIA-11.1 | S1 names the `assert_disjoint` call, gives the copy or removing `@no_alias_checks` | `reject_assert_disjoint_inside_no_alias_checks_gives_the_lines_that_need_no_check` | the call's name; `help` |
| DIA-22.1 | a callee's fact is reported at the call | `reject_a_callees_fact_is_reported_at_the_call` (E3031, E3020, E3028) | none |
| DIA-25.1 | the line a message gives compiles | `accept_the_line_a_message_gives_compiles` | — |
| DIA-25.2 | a form that would not do what it says is refused with the line that does | `reject_a_form_that_would_not_do_what_its_words_say` (E0113) | spec |
| DIA-26.1 | one mistake, one message; a stored function's at the store | `reject_one_mistake_in_a_stored_function_gives_one_message_at_the_store` | none |
| DIA-27.1 | a message names the chain of calls | `reject_a_message_names_the_chain_of_calls` (one link; two links in part, Q9) | spec; in part |
| DIA-27.2 | no message asks for a word on a first line | `reject_no_message_asks_for_a_word_on_a_first_line` (E0100) | spec (`[EFF-24]`'s) |
| DIA-28.1 | where both fixes compile, the rule that names the case: E3060 over E3022, E3031 over E3021 | `reject_two_errors_on_one_line_show_the_one_the_rule_names` | none |

## 2. Not written, and old tests seen

Skipped as credited and tested: WK-2, CLS-7a, DIA-16; BCK-4 is M5's (`ahead/M5/BCK-4`, its two
clauses). Still not written, from the rules M6a's items complete: BRW-1, BRW-15 and BRW-17's call
clauses, which M5's ahead files already cover (`BRW-1/accept_a_ref_mut_parameter_and_a_module_variable_are_checked_at_the_call`,
the parameter case of `BRW-15/reject_a_changing_method_through_a_read_only_name`, BRW-17.3 and 17.4's
files); BRW-22.1 (noalias, M6c); every clause whose row in §0 says "not written" (other milestones'
features, or a two-package test); A7-14's tooling (the runner that applies each fix and checks
again) and the shape helps of DIA-7 for the shapes not in DIA-7's or DIA-10's files (O1, O2, O6 to
O9, B5, B6, B10 to B12, B17, A1, U1, R1), most of them other milestones' codes.

Old tests seen while writing (not a sweep): `tests/conformance/BRW-11/reject_overlapping_mut_arguments.em`
(`update_both(ref mut value, ref mut value)`, E3022) is accepted by exception 1 (a run-pass);
`CLS-7/reject_repointing_self_through_a_mut_parameter.em` and `CLS-7/reject_repointing_self_through_mem_swap.em`
(E2103) agree with Q1's reading (A), their texts not the spec's; `BRW-10/reject_a_method_using_the_iterated_field.em`
and `BRW-18/reject_a_call_that_grows_the_list_its_loop_walks.em` keep E3020, with BRW-18's text form.

## 3. Spec questions

None is decided here. Each has its readings as code and the one recommended; the files follow the
recommended reading unless the row says otherwise, and say so in their header.

| # | rule | the question | readings | recommended |
|---|---|---|---|---|
| Q1 | CLS-7, FN-9, BRW-21 | `ref mut this` given to a call whose callee gives its parameter another object: `fn renew(ref mut u: Ui): u = Ui()`, and in a method `renew(ref mut this)`. W8's case allows `ref mut this` at calls; the brief's "a call that may end the object it was given is E3031" covers `close` (BRW-21's letter: an end, then a use); this one replaces | (A) `E2103` at `renew(ref mut this)`, by `renew`'s summary: `[CLS-7]` and E2103's row, "`this` given another object in a class method"; (B) `E3031` at the call: the old object ends, though BRW-21's letter needs a use of `u` after the end and `u = Ui()` is the last; (C) `E3031` at the method's callers (`ui.reset()`), through `reset`'s summary | **(A)**: E2103's row names the case; `[CLS-7]`'s own help sends a replacing function to a `ref mut` parameter of a caller's place, which `this` is not; the old E2103 tests agree. File: `CLS-7/reject_ref_mut_this_given_to_a_call_that_gives_it_another_object`. P-row: CLS-7 adds "(also through a call, when the callee may give its `ref mut` parameter another object, by its summary)" |
| Q2 | CLO-17, CLO-18, BRW-18, RC-5, WK-18 | "the handler stored on line N": which line? `b.on_click(owns log_click)` in `main`, with `fn on_click(owns f: fn()): this.handlers.push(owns f)` | (a) the program's line that puts the function where the call runs it (an assignment, a `push`, the call a lambda is written in); (b) the storing method's own line (`this.handlers.push(owns f)`); (c) the function's first line. CLO-18's "stored on line 20" against its walk on line 9 points to (a); WK-18's "`prune`, stored on line 40" beside BRW-21's lines 41 and 42 for its body fits (c), or a lambda stored on its first line | **(a)**, a lambda written in a call being stored on that call's line. The files put the handler straight into the list, so they hold under (a) and (b). P-row: DIA-27 adds the sentence |
| Q3 | CLO-17 | `t.on_visit = fn(n: Node): t.root.children.clear()` is one of the clause's `E3022` examples, but by its letter `t.on_visit` is neither a place the body reshapes or ends nor owned by one (`t.root.children` does not own `t.on_visit`), and the stated reason ("the run would end or move the function's own state") does not hold: the state is a reference to `t` | (A) the example stands, `E3022`; (B) the letter stands: accepted (a function that then used its node would be BRW-21.9's E3031 at the store) | **(B)**, with the example replaced by one that ends its holder (`t.on_visit = fn(n: Node): t.on_visit = look`). The file `CLO-17/reject_a_tree_visit_that_clears_the_roots_children` follows the spec's example (A), alone so it can become a run-pass |
| Q4 | CLO-17, BRW-18, CLO-18, DIA-25 | a stored function with no state of its own whose body changes the list that holds it: `UI.handlers.push(owns close_all)` with `fn close_all(u: Ui): UI.handlers.clear()`. CLO-17's letter and its `BUS` example refuse the store (`E3022`); BRW-18's E3020 message for `h(this)` gives a fix (walk by position, `handlers: Array[fn(ref mut Ui)]`) whose program still stores such a function, so the fix does not compile | (A) the letter: refused for every function whose run may reshape or end its holder, state or none; BRW-18's message is repaired to CLO-18's own way ("a handler that must go hands its removal to its holder": a result or a list of removals applied after the walk); (A') refused only where the function's state is a block the run may end or move (`[CLO-10]`: more than one word): a stateless or one-word function is copied out whole before the call (`[BRW-1]`, `[CLO-10]`), so its holder may change; the `BUS` and `on_click` examples are repaired to ones with a block | **(A)**: the rule's letter and examples, CLO-18's last sentence, and the simpler check to state and explain; W8's std.ecs handlers, which change fields only, compile under both. Files: `BRW-18/reject_a_stored_function_that_may_change_the_walked_list` (E3020 and the store's E3022) and `CLO-17/reject_a_function_value_stored_where_its_run_would_change_its_holder` follow (A). P-row: BRW-18's `h(this)` message's fix |
| Q5 | FN-9, BRW-21, FN-1 | `[FN-9]`: "`p: Player` reads the object, which neither changes nor ends while `p` is used"; `[BRW-21]` holds a plain parameter's callee only to not ending or moving what it views, and `[FN-1]`'s "a number another name writes ... shows through" (M5's Q9, the lead's Q1). `fn weaken_and_show(p: Player): PLAYERS[0].hp -= 1` then `println(p.hp)`, called `weaken_and_show(PLAYERS[0])` | (A) FN-9's letter: refused; (B) accepted, prints 4: "neither changes" means not through `p` | **(B)**, as the lead ruled for Q1/Q9. File: `FN-9/accept_a_write_by_another_name_shows_through_p`. P-row: FN-9's words become "which is not changed through `p` and does not end while `p` is used (a write by another name shows through, `[FN-1]`)" |
| Q6 | BRW-21, CLS-7, BRW-1 | where is `ref mut this` (or `this`) checked: `close(ref mut this)` inside `shut`, where `this` may be any `Ui` and `close` clears `UIS` then writes `u.count`? | (A) at `shut`'s callers through its summary (`this` is used after a line that may end it): `UIS[0].shut()` is E3031, `mine.shut()` allowed, `shut` compiles; (B) at `close(ref mut this)`: `this` may be in `UIS` ("`this` is never the only name of its object"), so `shut` is refused in every program | **(A)**: `[BRW-21]` counts `this` among the view parameters whose fact is kept and checked at each call, and `[BRW-1]` keeps the function itself compiling for a `ref mut` parameter and a module variable. File: `CLS-7/reject_ref_mut_this_given_to_a_call_that_may_end_its_object` |
| Q7 | OBJ-5, CLS-7a, E3013 | a `drop` that calls an `owns this` method that stores the object (`this.retire()`, `fn retire() owns this: this.archive()`, `fn archive() owns this: ARCHIVE.push(owns this)`): E3016 (OBJ-5, CLS-7a: stored from a `drop` or anything it calls) or E3013 (a hand-over out of a name that does not own the value; today's compiler says "cannot move `this` out of `drop`") | (A) E3016; (B) E3013 | **(A)**: the rule for the object being ended is the specific one, and its message's fix (keep a copy of a field) compiles where E3013's cannot (`[DIA-28]`). File: `OBJ-5/reject_a_drop_that_calls_what_stores_this` |
| Q8 | GRM, BRW-18, CLS-7 | `this.handlers[this.current](ref mut this)` (the spec's line) calls an element of a field; `x.f[T](...)` is also a method call with a type argument (`"4".parse[int]()`). Today's compiler reads the first as the second (E2072, "`Ui` has no method named `handlers`") | (a) the text decides it: what `f` names chooses (a field: an element, then a call through it; a method: type arguments) | (a); an implementation gap, no P-row. File: `CLS-7/accept_ref_mut_this_given_to_calls` |
| Q9 | DIA-27, EFF-25, BRW-18 | the chain's form beyond one call: with `step()` calling `tick()` calling `note`, which pushes to `LOG`, inside a walk of `LOG`. The spec's forms are one link ("(through `note`, line 4)") and a stored handler ("(through `click`, which runs the handler stored on line 9, `log_click`, which pushes to `LOG`)") | (a) every link with its line: "(through `tick`, line A, and `note`, line B)"; (b) the first and the last: "(through `tick`, line A, … `note`, line B)"; (c) as the handler form: "(through `tick`, which calls `note`, line B, which pushes to `LOG`)" | **(c)**: it is the spec's own form for a chain of more than one step, and it names every function with the one line that changes, which is the line DIA-27's fix acts on. File: `DIA-27/reject_a_message_names_the_chain_of_calls` claims only "`step()` (line N) adds to `LOG` (through `tick`", true under all three. P-row: DIA-27 adds the two-step example |
| Q10 | DIA-2 | the lead ruled (review.md, W7: "`int` is the prelude's alias of `i64`, and DIA-2 says nothing of it") that `int` may be printed `i64`; Hardened_14's DIA-2 (changed in 0.9.20) says "`int`, not `i64`" and "`int` for an untyped literal's default and for `len()`'s result", and `[int; 3]` where the program writes it | (A) the text: a type is named as the program spells it, else as its declaration does, and `int` for literals and `len()`; (B) the ruling: one type, printed `i64` | **(A)**, the text of the cut the build follows; the ruling may predate it. Raised, not decided: the lead's ruling stands until the lead says. File: `DIA-2/reject_a_refusal_names_what_the_program_wrote` follows (A) |
| Q11 | BCK-6 | "a loop that conditionally stores a view in a local declared outside it and changes the place it views on rounds where it did not": the place changed is (a) the place the view will view, on rounds before it is stored (`while`: change `xs[i]`, or store `ref xs[i]` and `break`); or (b) on any round where nothing is stored, while an earlier round's view is still live (`for i in 0..n:` storing `ref xs[i]` on some rounds and writing `xs[i] = 0` on the others, the view used after the loop) | (a), (b) | **(a)**: under (b) the stored view and the later round's write are two elements of one list whose indexes the rules of `[BRW-5]` do not prove apart (each round's `i` against an earlier round's), so (b) would need a proof the spec does not state. File: `BCK-6/accept_the_required_acceptances` (accepted under both). P-row: BCK-6 adds the example |
