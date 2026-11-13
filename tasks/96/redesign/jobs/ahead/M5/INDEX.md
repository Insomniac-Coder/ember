# M5 ahead: tests and spec questions, written before the builders

Written from `docs/spec-source/Ember_v0.9.20_Hardened_14.md` alone (`355cf49f`, tag `spec-h14`) by the
Opus deputy, one milestone ahead of the builders. **The pilot** (2026-11-13, merged `ed092043`) covered
A2-2 and A2-3: 8 rules. **Batch 1 of the full run** (2026-11-13, this file's second version) ruled the
pilot's Q2 to Q13, probed every file with the existing compiler, fixed the files that failed for a
reason other than their rule, and added the next 10 rules in the milestone's item order: SPN-1 (the
last of A2-3), then A2-4's BRW-1, BRW-2, BRW-4, BRW-5, BRW-16, BRW-17, LT-5, LT-18, LT-25. Nothing
was built: every expectation is the spec's words, never the compiler's. These files live here, not
under `tests/`, because they fail until M5 is built. When a rule is built, its directory moves to
`tests/conformance/<RULE>/` and is checked with `python tasks/impl-0.9.9/annotations.py
tests/conformance/<RULE>`. **No hardening is cut here**: the P-rows of §3 go into Hardened_15, cut once
at the end of M5's run.

**Conventions.** Each file starts with `# Clause <RULE>.<n>: "<the clause, quoted>"`. One file per
clause, or per clause and outcome (`accept_`, `reject_`); a file holds several cases as functions. A
refusal's text is the spec's own example text where the spec gives one, with the file's own line
numbers in place of the example's; where a ruling of §3 decides the text, the ruling's; where neither
gives one, the expectation is `#$ error[CODE]` with no text ("none" below). Line numbers in the
expectations were resolved by a generator from markers, not by hand. A **case** is one expectation:
one `#$ error[...]` or one `#$ stdout:` line.

**The probe** (`target/release/ember.exe check`, the Hardened_14 tree's build; no cargo): every file
was run. Each failure was read and sorted: the rule (or another milestone's feature) not built, or a
fault in the test. Faults found and fixed, keeping what the spec says:
- 5 LT-4 files: their `unsafe fn deallocate` and `unsafe fn reallocate` had no `@safety`, which every
  `unsafe fn` carries (`[UNS-7]`, `L3015`, and an unclaimed warning fails a test, `[TST-1]`); added,
  and the one line number this moved in a message.
- `BRW-13/accept_the_old_value_is_replaced_and_ends`: `t` was never read (`L1001`, `[LNT-1]`); the last
  line reads it.
- `BRW-12/accept_a_value_a_call_made_is_owned_by_the_name`: `-> Array[StrView] from s` used a side
  form the clause does not need (rule 2 gives the same); removed. Its standing is Q15.
- `BRW-17/reject_a_reshape_worked_out_through_a_function_value` (new): the stored function written as
  a block did not parse; it is the spec's one line, `rename = fn() owns n: n.name = String("b")`.
- `BCK-4/reject_a_view_kept_or_returned_past_its_target`: its instance case, which needs `using`
  (M10), stopped the parser for the whole file; moved to its own file, so the M5 cases can be
  credited when M5 lands.

Every other failure is a feature not built: `static mut` with no `unsafe` (M2), part names as views,
re-pointing, `const ref mut` (A2-3), `ref mut` before a span made in the call (A2-7), `shares`,
`weakref`, `sharedref mut` and `exists as` on links (M4, A2-8), capture words (M8), `Lock` and
`thread_shared` (M9), `using` and the `Allocator` interface with its `unsafe` methods (M10), kind
tests and `dyn` from a struct value (M11), and the rules themselves.

**Totals.** 18 rules, 85 clauses, 115 test files (63 accept, 52 reject), 293 cases (130 error
expectations, of which 98 have no text; 163 output lines). The pilot's 8 rules: 45 clauses, 64 files
(one split), 151 cases. Batch 1's 10 rules: 40 clauses, 51 files, 142 cases.

## 0. Rules covered, and what else each needs before it can be credited

A rule is credited (`built_for_0915`) only when every clause is built and its tests pass, and most of
these rules have a clause that waits for another milestone. "M5" items are this milestone's own.

| rule | clauses | files | cases | also needs (beyond M5's own A2-2 to A2-8) |
|---|---|---|---|---|
| BRW-8 | 6 | 6 | 14 | M2 (`static mut` with no `unsafe`, A1-17; the summary facts Q1 and Q14 need), M3 (`owns` at calls) |
| LT-4 | 5 | 9 | 12 | M8 (capture words `owns st`), M9 (threads, `thread_shared`, scoped tasks), M10 (`using`, `Allocator`, arenas) |
| BCK-4 | 2 | 5 | 13 | M3 (hand-over by `=`, `owns`), M10 (its instance case) |
| BRW-12 | 10 | 14 | 30 | M3 (hand-over by `=`, call words), M4 (`copy of`, `weakref`, `exists as`) |
| BRW-13 | 6 | 8 | 14 | M3 (`ref mut` and `const ref mut` call words, A1-8) |
| BRW-14 | 9 | 11 | 31 | M3 (`owns` parameters, `for x in owns e`, `match owns e`), M4 (`exists as copy of`), M11 (kind tests `is T as c`) |
| BRW-15 | 3 | 6 | 20 | M4 (`shares mut` links), M6a (a changing method through a read-only name, by its summary, A2-10), M9 (`Mutex`, `Lock`, `Atomic`, channels) |
| CTL-11 | 4 | 5 | 17 | M4 (`exists as` on links, `weakref`, `None` as an empty link), M11 (kind tests, `dyn` values) |
| SPN-1 | 3 | 4 | 18 | M3 (`ref mut`, `owns` at calls) |
| BRW-1 | 10 | 14 | 33 | M2 (`static mut`), M4 (links, `exists as`, `is not`, `copy of`, `@derive(Clone)`), M6a (BRW-11's one whole variable `add_both`, A2-13; the call check of BRW-1.4, A2-10), M8 (a stored function's two words, BRW-1.7) |
| BRW-2 | 2 | 2 | 7 | M3 (`owns` at calls) |
| BRW-4 | 2 | 3 | 7 | M4 (links, BRW-4.2); M6b removes the run-time checks on class fields (A2-11), which these tests do not observe |
| BRW-5 | 5 | 6 | 20 | M3 (call words), M6a (two arguments of one call, `[BRW-11]`'s check, A2-10) |
| BRW-16 | 6 | 6 | 20 | M2 (`static mut`), M4 (`shares mut` links, BRW-16.5); M5's A2-5 (positions at run time) and A2-7 (SoA proxies) |
| BRW-17 | 7 | 10 | 23 | M2 (`static mut`), M4 (links, `weakref`, `exists as`), M6a (a reshape through a function value and the call check, A2-10, BRW-21), M8 (capture words); M5's A2-8 (`sharedref mut`) |
| LT-5 | 2 | 2 | 3 | M3 (`ref mut` at calls) |
| LT-18 | 2 | 2 | 7 | M5's A2-8 (a kept `ref mut` field) only |
| LT-25 | 1 | 2 | 4 | none |

## 1. Rules, clauses and tests

Message column: "spec" = the spec's text; "ruled" = the text a ruling of §3 gives; "derived" = the text
the clause's own form gives for that program (pinned only as far as the form decides); "none" = code
only.

### BRW-8 (VII.4) — parameters by address or as a copy

| clause | what it says | test | message |
|---|---|---|---|
| 8.1 | plain and `ref mut` parameters are passed by address: the callee reads or changes the caller's place | `accept_plain_and_ref_mut_parameters_are_the_callers_place` | — |
| 8.2 | a view is passed as itself and keeps the regions it carries | `accept_a_view_parameter_keeps_the_regions_it_carries` | — |
| 8.3 | a small `Copy` value without a `Cell` goes in registers, except the object of a view-returning method | `accept_a_small_copy_parameter_and_the_exceptions` | — |
| 8.4 | an `owns` parameter receives the value | `accept_an_owns_parameter_receives_the_value` | — |
| 8.5 | a view into a copy parameter's own storage ends with the call: returning it is E3063 | `reject_a_view_into_a_copy_parameters_own_storage` (with Q7's cases: `ref h.s` refused, `ref h.b.x` accepted, `owns h` refused) | none |
| 8.6 | the choice depends only on the first line and cannot be observed | `accept_the_passing_choice_cannot_be_observed` (Q1, the lead's ruling: prints 9 and 9; see Q14) | — |

### LT-4 (VII.6) — values made in a memory manager's instance

| clause | what it says | test | message |
|---|---|---|---|
| 4.1 | a value made in an instance, and every value holding one, lives no longer than it: kept or returned past it is E3063 | `reject_a_value_made_in_an_instance_kept_past_it`; `accept_the_instance_the_caller_keeps` (the message's fix, DIA-25) | spec (returned); none (kept in the caller's list) |
| 4.2 | to an unscoped thread, a job outside a scope or a kept callback only from a `static` instance; else E3063 at the hand-over | `reject_an_instance_value_handed_to_an_unscoped_thread` (Q11: `join` changes nothing); `accept_the_ways_out_for_a_thread` | spec |
| 4.3 | a value of a several-instance manager keeps its instance's address, a kept `sharedref mut` link: the instance stays usable, growth and end go through it | `accept_the_instance_is_used_while_its_values_live`; `reject_growth_through_the_link_while_a_ref_mut_name_is_used` (E3022) | none (Q8) |
| 4.4 | a one-instance manager (`Global`, `System`) costs no word | `accept_a_one_instance_manager_costs_no_word` (`size_of`) | — |
| 4.5 | arena allocations point into the arena | `reject_an_arena_allocation_outliving_the_arena` (E3063, E3060); `accept_an_arena_allocation_within_the_arena` | none |

### BCK-4 (XVIII.4) — storage end

| clause | what it says | test | message |
|---|---|---|---|
| 4.1 | a place's storage ends (scope exit, handed over, replaced, ended) while a view of it is live: E3060 at the view's later use | `reject_a_view_used_after_its_place_ends` (Q2: (a), E3060 for a local of an outer block); `accept_a_view_whose_last_use_comes_first` | none |
| 4.2 | a view, or a value holding views, kept or returned past what it points into (an instance's value included): E3063 | `reject_a_view_kept_or_returned_past_its_target`; `reject_a_value_made_in_an_instance_returned_past_it` (split out: needs M10); `accept_a_view_returned_into_what_outlives_the_call` | none |

### BRW-12 (VII.4) — a name for a part is a read-only view

| clause | what it says | test | message |
|---|---|---|---|
| 12.1 | a name is declared by the line that first gives it a value; once per round in a loop body | `accept_a_name_in_a_loop_body_is_declared_each_round` | — |
| 12.2 | a name bound to a part or to a view-giving call is a read-only view: never a hand-over, never a silent copy | `accept_a_part_name_is_a_view_not_a_hand_over`; `reject_a_change_while_a_part_name_reads_it` | spec (BRW-1's `boss` text) |
| 12.3 | a part of a `Copy` type is copied (`w = this.rest`; `this.rest = w[i + 1..]` changes nothing `w` reads) | `accept_a_part_of_a_copy_type_is_copied` | — |
| 12.4 | a copied `Copy` struct is not changed through the new name while the source is used (E3023) | `reject_a_copied_copy_struct_changed_while_its_source_is_used`; `accept_a_copied_copy_struct_changes_once_its_source_is_done` | none |
| 12.5 | a call's new value is handed to the name, which owns it | `accept_a_value_a_call_made_is_owned_by_the_name` (its `from s` removed; Q15) | — |
| 12.6 | `ref mut` before a part names that very place | `accept_ref_mut_before_a_part_names_that_very_place` | — |
| 12.7 | `ref mut` before a `MutSpan` (or tuple of them) a call gives lets the bound names write | `accept_ref_mut_before_a_call_that_gives_spans` (Q6: (a)); `reject_a_plain_name_bound_from_a_call_giving_spans` | help only (SPN-3's line); text none |
| 12.8 | `copy of` makes a copy of your own | `accept_copy_of_a_part_is_a_copy_of_your_own` | — |
| 12.9 | the link words make a link | `accept_a_link_word_before_a_part_makes_a_link` | — |
| 12.10 | `ref` states the default, needed before a part only to view a number | `accept_ref_before_a_part_states_the_default`; `reject_a_ref_view_of_a_number_while_it_changes` | none |

### BRW-13 (VII.4) — `=` on a name that has a value

| clause | what it says | test | message |
|---|---|---|---|
| 13.1 | `=` puts the new value in the name's place; a name declared with `=` has its old value replaced (and ended) | `accept_the_old_value_is_replaced_and_ends` | — |
| 13.2 | a view name given a view now views that, checked by BRW-1 against the new target | `accept_a_view_name_given_a_new_view_views_that`; `reject_a_re_pointed_view_is_checked_against_its_new_target` | none |
| 13.3 | a `ref mut` name writes its place until given another ref (`r = ref mut b`) | `accept_a_ref_mut_name_writes_until_given_another_ref` | — |
| 13.4 | the line giving another ref says `ref mut` again; a read-only `ref b` given to it is E3023 | `reject_a_read_only_ref_given_to_a_ref_mut_name` | spec (for `r`/`b`); ruled, Q4 (`tsecond`) |
| 13.5 | a name holding a `MutSpan` writes elements and is pointed at another span | `accept_a_name_holding_a_mutspan_writes_and_is_pointed_elsewhere` | — |
| 13.6 | `const ref mut` is never given another ref (E3023) | `reject_a_const_ref_mut_name_given_another_ref`; `accept_a_const_ref_mut_name_writes_its_place` | spec |

### BRW-14 (VII.4) — names that keep their place

| clause | what it says | test | message |
|---|---|---|---|
| 14.1 | a parameter not `owns`, a loop or pattern name that views, an opened or kind-tested name keep what they were given: `=` is E3023 | `reject_eq_on_a_parameter_gives_the_local_of_your_own`; `reject_eq_on_a_loop_pattern_opened_or_tested_name` (Q13: (a)) | spec (both parameter texts); none (the other names) |
| 14.2 | so is any other change through one that is plain | `reject_a_change_through_a_plain_name_that_keeps_its_place` | none |
| 14.3 | the local is `new_` + the name, else the first free `new_n_2`, `new_n_3`, … | `reject_the_local_of_your_own_takes_a_free_name` | derived |
| 14.4 | made by the refused line if a top-level plain `=` with no read above; else a new first line (`new_n = n`, `new_s = copy of s`); never in a loop or branch | `reject_where_the_local_of_your_own_is_made` (whole texts; a `Handle` with no `Clone`) | derived; ruled, Q5 (nouns, the no-`Clone` offer) |
| 14.5 | every later use is written with the local, each line named, as Python's rebinding | `accept_the_local_of_your_own_does_what_rebinding_does` | — |
| 14.6 | a name handed its value owns it: `owns` parameter, `for x in owns e:`, `match owns e:` names may change; `kept = job` hands over | `accept_a_name_handed_its_value_may_change` | — |
| 14.7 | `exists as copy of v` may change; the map is untouched | `accept_exists_as_copy_of_may_change` | — |
| 14.8 | a `ref mut` one writes the caller's place and may be given another ref (span: the caller's `d`; `Counter`: for the rest of the call) | `accept_a_ref_mut_parameter_given_another_ref` (Q10: (a)) | — |
| 14.9 | `const ref mut dst` writes the caller's elements, never given another ref | `accept_a_const_ref_mut_parameter_keeps_the_callers_span`; `reject_a_const_ref_mut_parameter_given_another_ref` | none |

### BRW-15 (VII.4) — a plain name never changes anything

| clause | what it says | test | message |
|---|---|---|---|
| 15.1 | a plain parameter, loop, pattern or opened name, and a plain local holding a view, change nothing they name or lead to (`Cell`, `shares` target, `MutSpan` elements): E3023 with the line to write; an iterator a call made is owned and may advance | `reject_a_change_through_a_plain_name_gives_the_line` (the four texts, and `tock` called twice: Q12); `reject_a_change_behind_a_plain_name` (`r = 99`: Q3); `accept_the_lines_the_messages_give` | spec (four); ruled (Q12's `tock`, Q3's `r = 99`); none (`Cell`, `shares mut`, part name) |
| 15.2 | a method that may change its object, called through a read-only name, is E3023 | `reject_a_changing_method_through_a_read_only_name`; `accept_a_reading_method_through_a_read_only_name` | spec (EXC-15's `heal`); none (through a parameter) |
| 15.3 | `Mutex`, `RwLock`, `Atomic`, channel ends and `guarded by` fields under their lock are used through any name; handed to a call they take no word | `accept_types_shared_between_threads_change_through_plain_names` | — |

### CTL-11 (VI.4) — headers that name what they test

| clause | what it says | test | message |
|---|---|---|---|
| 11.1 | `if`/`while ... exists as p:` and `if`/`elif ... is T as c:` run their block as GRM-44 and DSP-4 say | `accept_headers_that_name_what_they_test` | — |
| 11.2 | the name is in scope in that block only, never in an `elif` or `else` | `reject_the_name_used_outside_its_block` (E1010) | none |
| 11.3 | `else:` runs when nothing is held or the kind differs | `accept_else_runs_when_nothing_is_held_or_the_kind_differs` | — |
| 11.4 | the name keeps its place: a plain one only reads; `exists as copy of v` is a copy of your own | `reject_a_change_through_a_plain_header_name`; `accept_a_copy_opened_from_a_header_changes` | none |

### SPN-1 (VII.7) — arrays and strings convert to views

| clause | what it says | test | message |
|---|---|---|---|
| 1.1 | an `Array[T]`, a `[T; N]` or a `String` converts to `Span[T]` or `StrView` where one is expected | `accept_an_array_a_fixed_array_and_a_string_convert_to_views` (a parameter, a typed local, a field) | — |
| 1.2 | and to `MutSpan[T]` where one is expected: a `ref mut` parameter, a `ref mut` name, a `MutSpan` field made with `ref mut` | `accept_a_mutspan_where_ref_mut_says_so` | — |
| 1.3 | the conversion makes a view of the source under the ordinary rules: not changed through another name, handed over or ended while used, however it is written (implicitly, `as_span()`/`as_mut_span()`, by a call) | `reject_the_source_changed_handed_over_or_ended_while_the_view_is_used` (9 cases: E3021 for each spelling, E3060 for a push, a text added to, a hand-over, an end; E3021 and E3022 beside `ref mut ... as_mut_span()`); `accept_the_source_changes_after_the_views_last_use` | none |

### BRW-1 (VII.4) — one name changes, or any number read

| clause | what it says | test | message |
|---|---|---|---|
| 1.1 | a place is reached through any number of read-only views or through exactly one name that may change it; all checked when compiling, nothing while running | `accept_many_read_only_views_or_one_changing_name` | — |
| 1.2 | while a `ref mut` name (parameter, local, loop/pattern/opened name, kept `ref mut` field, a `MutSpan` holder) is used, its place is read (E3021) or changed (E3022) through no other name, a link of the same type included | `reject_a_ref_mut_name_is_alone` (5 cases); `reject_a_link_of_the_same_type_may_name_the_place` | spec (`boss` read); none (the rest) |
| 1.3 | unless proved different (`if t is not a:`) or one whole variable given to one call (BRW-11) | `accept_unless_proved_apart_or_one_whole_variable` (`add_both` prints 2) | — |
| 1.4 | for a `ref mut` parameter and a module variable the check is at each call (BRW-21), so the function itself compiles | `accept_a_ref_mut_parameter_and_a_module_variable_are_checked_at_the_call` | — |
| 1.5 | two names of elements of one list, one `ref mut`, indexes not proved different, are refused as two such arguments are | `reject_two_element_names_one_ref_mut_not_proved_different` (locals are not constants); `accept_two_element_names_proved_different` (literals, `const` items, `i + 1..`, `if i != j:`) | none |
| 1.6 | a reshape through another name while a `ref mut` name that is not a position is used is E3022 at the reshape; a position is BRW-16's | `reject_a_reshape_while_a_ref_mut_address_is_used`; `accept_a_position_is_ruled_by_brw_16` (BCK-6's example) | none |
| 1.7 | a call through a `ref mut` name of a stored function copies its two words out first; its arguments may reach that place; a later use is checked against the call | `accept_a_call_through_a_ref_mut_name_of_a_stored_function` | — |
| 1.8 | `this` is not such a name (EXC-15) | `accept_this_is_not_the_only_name_of_its_object` (the turret, verdict 18) | — |
| 1.9 | while a read-only view (part name, `ref` name, plain or `Span` parameter, plain `MutSpan` holder, loop/pattern/opened name, a view a call gave) is used, what it reads is not changed through another name (E3021 at the change); a reshape ends it (E3060 at its later use); the refusal gives the ways out | `reject_a_change_of_what_a_read_only_name_reads` (5 kinds); `reject_a_reshape_ends_a_read_only_name` | spec (`boss`); none (the rest) |
| 1.10 | a copy of your own is restricted by nothing; a `shares` owner is an owner and sees the other owners' changes | `accept_a_copy_of_your_own_and_a_shares_owner` | — |

### BRW-2 (VII.4) — liveness

| clause | what it says | test | message |
|---|---|---|---|
| 2.1 | a view is in use to the last use of anything made from it: itself, a view made from it, a value returned from a call it was given to | `reject_a_change_while_anything_made_from_the_view_is_used`; `accept_a_view_ends_at_its_last_use_not_its_scope` | none |
| 2.2 | scope does not matter: `n = v.len(); v.push(owns n)` is legal | `accept_a_view_ends_at_its_last_use_not_its_scope` (the spec's line first) | — |

### BRW-4 (VII.4) — disjoint fields

| clause | what it says | test | message |
|---|---|---|---|
| 4.1 | `ref mut a.x` and `ref mut a.y` together for different fields, at any depth, for struct values and class objects alike | `accept_different_fields_at_any_depth_struct_and_class`; `reject_one_field_or_a_prefix_twice` (the same field; a prefix and its extension, `[BCK-3]`) | none |
| 4.2 | what two paths reach through links is BRW-17's | `reject_different_fields_that_reach_one_object_through_links` | none |

### BRW-5 (VII.4) — indexes are not disjoint unless proved

| clause | what it says | test | message |
|---|---|---|---|
| 5.1 | `ref mut a[i]` beside `ref mut a[j]`, as names or arguments, is E3022 unless proved: two different constants, `j` from `i + 1..`, inside `if i != j:`; nothing at run time | `reject_two_ref_mut_elements_not_proved_different`; `accept_indices_proved_different` | spec (`collide`); none (the names) |
| 5.2 | the same position twice: "both are `bodies[i]`" | `reject_the_same_position_twice` | spec (the fragment) |
| 5.3 | `get_pair_mut`, `split_at`, `chunks_mut`, `for ref mut x in xs:`, `swap(i, j)` and SoA columns are the library's ways to two changeable views | `accept_the_librarys_ways_to_two_changeable_views` (Q18 for `swap`) | — |
| 5.4 | `get_pair_mut(i, j)` on an `Array` or a `MutSpan` checks range and difference, else `None` with nothing changed; the pair points into the list (LT-1 rule 1) | `accept_get_pair_mut_checks_and_gives_two_views`; `reject_a_pair_used_across_a_reshape_of_its_list` | none |
| 5.5 | zero-sized elements need not have distinct addresses; nothing about addresses follows from indexes | `accept_get_pair_mut_checks_and_gives_two_views` (a type with no fields) | — |

### BRW-16 (VII.5) — positions

| clause | what it says | test | message |
|---|---|---|---|
| 16.1 | a name for a list element made by indexing (three spellings), of a list reached through owning locals, parameters, `this`, one-owner fields, module variables, is a position: the index is read once | `accept_a_position_keeps_the_index_it_was_made_with` | — |
| 16.2 | the element is found again at each use; the list may grow while the name is used | `accept_the_list_may_grow_while_a_position_is_used` (100 pushes; through a parameter, `this` and a field, a `static mut`) | — |
| 16.3 | remove, insert, reorder, clear, replace end it: E3060 at a later use, a call that reads and removes included | `reject_a_removal_insertion_reorder_clear_or_replace_ends_a_position` (6 ways) | spec (`remove`); none (the rest) |
| 16.4 | growing the list is not a use of the element | `accept_the_list_may_grow_while_a_position_is_used` | — |
| 16.5 | a list reached through a link gives addresses (BRW-17) | `reject_a_list_reached_through_a_link_gives_addresses` (E3060 read-only; E3022 `ref mut`) | none |
| 16.6 | an `Array`'s `xs[i]`/`xs.get(i)` and an SoA proxy are positions; any other container's element (a `Map`'s value, an own `Index`) is an address that adding ends | `accept_an_array_and_an_soa_give_positions`; `reject_other_containers_give_addresses_that_adding_ends` | none |

### BRW-17 (VII.5) — addresses

| clause | what it says | test | message |
|---|---|---|---|
| 17.1 | every other view (kept view, through a link, into a `String`'s text, an `Array` field's elements, an enum's payload, a view a call gives) ends at any reshape through any name: E3060 at a later use | `reject_a_reshape_ends_every_other_view` (6 kinds) | none |
| 17.2 | two paths of one type, one through a link or kept view, may reach one object unless known different (types, a fresh object, `if a is not b:`) | `reject_two_paths_through_links_may_reach_one_object`; `accept_paths_known_to_differ` | none |
| 17.3 | what may reshape is worked out from the code, function values and `dyn` included: the `rename()` message | `reject_a_reshape_worked_out_through_a_function_value` | spec |
| 17.4 | a view parameter or `this` and what its function may end or move: checked at each call (BRW-21) | `accept_a_view_parameter_is_checked_at_each_call`; `reject_a_call_whose_callee_ends_what_its_argument_views` (E3031) | spec (BRW-21's `note`) |
| 17.5 | adding moves only what is stored inline: a view of a class object survives a push or extend; one of a struct element does not | `accept_adding_keeps_views_of_class_objects` (the spec's `Watcher`); `reject_adding_ends_a_view_of_an_inline_element` | none |
| 17.6 | removing, clearing or replacing still ends every view of what it may end | `reject_removing_clearing_or_replacing_ends_class_object_views` | none |
| 17.7 | keys viewed across a change of their map: E3060 | `reject_map_keys_viewed_across_a_change` | spec |

### LT-5 (VII.6) — regions by liveness

| clause | what it says | test | message |
|---|---|---|---|
| 5.1 | regions of locals are inferred by non-lexical liveness | `accept_regions_follow_liveness_not_blocks` (a path with no later use; BCK-6's `first_even`) | — |
| 5.2 | diagnostics describe them in source terms ("the view of `x` on line 12 is still used on line 19") | `reject_a_diagnostic_names_the_region_by_its_lines` | derived (BRW-1's form: the two lines only) |

### LT-18 (VII.6) — view fields are ordinary views

| clause | what it says | test | message |
|---|---|---|---|
| 18.1 | every view field is an ordinary view under BRW-1 to BRW-22 | `reject_a_view_field_is_an_ordinary_view` (E3021, E3060, and a kept `ref mut` field's E3021) | none |
| 18.2 | several regions add no aliasing rule and weaken none | `accept_several_regions_add_no_rule` | — |

### LT-25 (VII.6) — no view into a field of its own value

| clause | what it says | test | message |
|---|---|---|---|
| 25 | a view field may not point into another field of the value that holds it (shape B5, E3024) | `reject_a_view_field_into_another_field_of_its_value` (by the constructor; by a later line); `accept_a_view_field_into_what_lives_outside_the_value` | none |

## 2. Old tests the target contradicts

Under `tests/conformance/` unless said; line numbers are the files' own. "Consistent" files are listed
after each part.

### The pilot's rules

| file:line | pins | Hardened_14 says instead (rule) |
|---|---|---|
| `BRW-8/accept_a_copy_struct_holding_a_cell_is_passed_by_address.em:13` | `t.count.set(...)` through plain `t` runs | E3023: a plain parameter changes nothing behind a `Cell` (BRW-15); by-address passing is observed through a module variable instead (BRW-8.3's test) |
| `FN-1/accept_cell_and_refcell_through_borrowed_parameters.em:13,19` | `s.frame.set` through plain `s`; `RefCell` | E3023 at line 13 (BRW-15); `RefCell` is gone (E1010) |
| `BRW-1/accept_writing_through_a_mut_ref.em:12` | `m = ref mut y` writes `y` into `x`, prints 5 5 | re-points `m` (BRW-13.3): prints 1 5 |
| `BRW-1/reject_a_reference_is_not_re_pointed.em:17` | E3021 for `hottest = r` | accepted, re-points `hottest` (BRW-13.2); run-pass printing 3.0 |
| `BRW-1/reject_writing_through_a_shared_ref.em:16` | E3021 for `r = 99` on a `ref` name | E3023 with Q3's line: "`r = 99` changes `x`, but `r` only reads it; to change `x` through `r`, write `r = ref mut x` in place of line 15" |
| `DIA-9/reject_a_diagnostic_never_suggests_unsafe_while_a_safe_fix_compiles.em:3` | E3023 "cannot mutate borrowed parameter `values`" | E3023 stays; the text is BRW-15's form, with the line to write (`fn append_one(ref mut values: Array[i32]):`, call `append_one(ref mut values)`) |
| `FN-2/reject_a_plain_parameter_is_not_changed.em:6` | same old text | as DIA-9 (BRW-15) |
| `LT-44/accept_one_arena_parameter_needs_no_borrows.em:8`; `LT-44/accept_a_fixed_or_scoped_arena_parameter_is_a_source.em:12,15,18` | `arena.alloc` through a plain `arena` | E3023: `alloc*` changes the arena, so through a plain parameter it is BRW-15.2's refusal (ARN-1) |
| `LT-4a/*.em` (4 files, `allocate_one(arena: Arena)` lines 4-7), `TYP-15/reject_arena_backed_view_in_box.em:7` | the same plain-arena `alloc` | E3023 (BRW-15.2, ARN-1); each needs `ref mut arena` and `ref mut` at its calls |
| `LT-4a/reject_reset_while_a_wrapped_allocation_is_live.em:10` | E3021 at `arena.reset()` | `reset()` ends every view into the arena: E3060 at `println(value)`, line 11 (LT-4.5, ARN-1) |
| `LT-1/reject_a_reference_to_a_view_parameter_itself.em:9`; `LT-1a/reject_a_reference_to_a_named_view_parameters_slot.em:8` | E3060 | E3063 (BRW-8.5) |
| `LT-1/reject_a_reference_into_a_view_parameters_own_storage.em:21,24` | E3060 at both | line 21 (`owns h`): E3063 (LT-1, BCK-4.2); line 24 (plain `h`, `ref h.b.x`): accepted (Q7) |
| `SPN-5/accept_views_of_a_view_parameters_target.em:19,30` | `t[0] = 20` through a plain name from `split_at` | E3023 (BRW-12.7, BRW-15): `parts = ref mut s.split_at(1)` and `t = ref mut tail(...)`; also `tail(ref mut ...)` at the call (FN-2a) |
| 31 files with "cannot write through a shared …" or "cannot mutate borrowed parameter" (`grep -rln "cannot write through a shared\|cannot mutate borrowed parameter" tests/`) | E3021 for a write through a read-only name (15 files), or E3023 with the old text (the rest) | candidates for BRW-15: a write through a read-only name is E3023 with the line to write; some are methods (BRW-15.2, M6a), some `FFI-40a` const methods, some ECS queries |

Other rules' changes seen in the same files: `EXP-4/*outliving_its_temporary.em` (2) and `TXT-10/reject_*`
(3) refuse a view of a temporary used on the next line; BRW-19/EXP-4 keep the temporary until the
view's last use, so all five are run-pass (A1-18, also M5). `EXC-18/accept_a_*getter*.em` (2) alias a
class object with `q = p` / `alias = p`, now a hand-over (OWN-1, OWN-3: E3040 at the later use of
`p`). `ARN-4/reject_using_fixed_backing_while_the_arena_is_live.em:7` is E3022, not E3021, with
`Arena.fixed(ref mut bytes)`. Most arena tests lack `owns` at `alloc(…)` (ARN-3, FN-2a: E3028).
Consistent with the target: `BRW-14/*` (both), `BCK-4/*` (both; Q2 confirms the scope-exit one),
`BRW-8/accept_small_copy_parameter_by_value.em`, `BRW-8/reject_an_instance_value_that_takes_by_address.em`,
`LT-1/accept_rule_1_takes_any_borrowed_receiver.em`, `LT-1/accept_a_call_returning_a_ref_is_a_place.em`,
`LT-1a/reject_borrows_names_a_copy_struct_holding_a_cell.em`, `OWN-8/accept_derived_enum_clone.em`.

### Batch 1's rules

| file:line | pins | Hardened_14 says instead (rule) |
|---|---|---|
| `SPN-1/accept_as_mut_span_writes_through.em:12,14` | `s = a.as_mut_span()` then `s[0] = 40` | E3023 at line 14: a plain name holding a `MutSpan` only reads (BRW-15, SPN-3); `s = ref mut a.as_mut_span()` |
| `SPN-1/reject_a_view_returned_from_a_call.em:15`; `SPN-1/reject_as_span_still_borrows.em:11`; `BRW-1/reject_mutating_a_viewed_container.em:13` | E3021 at the push | the push reshapes the list and ends the view: E3060 at `println(v[0])`, lines 16, 12, 14 (BCK-3, BRW-17) |
| `SPN-1/reject_mutating_around_a_live_str_view.em:12`; `SPN-1/reject_mutating_around_an_implicit_str_view.em:11` | E3021 at `push_str` | E3060 at the later use, lines 13 and 12 (BRW-17: a `String`'s text) |
| `SPN-1/reject_mutating_while_a_slice_lives.em:3` | E3021 (header) | E3060 at `println(view)`, line 11 |
| `SPN-1/reject_writing_through_a_shared_span.em:3` | E3021 "cannot write through a shared `Span`" | E3023 with the line to write (BRW-15: through the plain parameter `xs`) |
| `SPN-1/reject_writing_through_a_slice_inside_its_loop.em:3` | E3021 | E3020 (Q16: `[DIA-28]` shows the walk's error, whose fix compiles) |
| `SPN-1/run_fail_string_field_view_keeps_exclusivity.em:18-21` | an exclusivity panic while running | nothing is checked while running (BRW-1); `alias = box` hands the object over: compile-fail, E3040 at line 19 (OWN-3, A1-9) |
| `BRW-1/reject_assigning_an_owning_field_while_borrowed_once.em:17,18` | E3021 at `h.arr = ...` | as written, run-pass printing 1 (`r = first(ref mut h)` copies the number, BRW-12); with `r = ref mut first(ref mut h)` it is E3022 at line 18 (BRW-1.6) |
| `BRW-2/reject_a_loop_writes_the_element_it_keeps.em:12` | E3021 at `xs[0] = "z"` | replacing a `String` element reshapes the text `best` views: E3060 at `best`'s next use (line 10 on the next round, or line 13) (BRW-17) |
| `BRW-2/reject_a_write_before_the_last_use.em:9`; `BRW-4/reject_a_method_that_changes_a_borrowed_field.em:18`; `BRW-4/reject_two_mutable_borrows_of_one_field.em:16` | old texts | the codes stand; the texts are BRW-1's form (the method's case needs BRW-10's summary, M6a) |
| `BRW-5/accept_a_mutable_view_splits_in_two.em:17-20` | `parts = values.as_mut_span().split_at(1)`, `left[0] = 10` | E3023 at line 20: names bound from a split without `ref mut` only read (BRW-12.7, SPN-3); `left, right = ref mut values.as_mut_span().split_at(1)` |
| `BRW-5/accept_get_pair_mut_gives_two_elements.em:16-19` (and its other matches); `BRW-5/accept_get_pair_mut_of_a_mutable_view.em:12-13,22-23` | plain pattern names `Some((a, b)):` written through | E3023 (GRM-13, BRW-14): `Some((ref mut a, ref mut b)):`, or `if ... exists as (ref mut a, ref mut b):`; the second file's `xs.as_mut_span().get_pair_mut(...)` needs `ref mut` before the span or the call on `xs` |
| `BRW-5/reject_a_pair_used_after_its_array_changes.em:8,11,20` | `.clone()`; E3060 for `return a` | `.clone()` is E1010 (`copy of xs`); returning a view of the local `tmp` is E3063 (BCK-4.2); `a = 5` needs `ref mut a`; line 18's E3022 stands |
| `BRW-5/reject_growing_while_an_element_is_borrowed.em:12` | E3022 at the push | a position survives a push (BRW-16, BCK-6): run-pass printing 10 |
| `BRW-5/reject_mutating_array_while_split_view_lives.em:3` | E3022 (header) | the push ends the read-only span `left`: E3060 at `println(left[0])`, line 16 |
| `BRW-5/reject_the_same_constant_index_twice.em:10` | "`v[0]` is already mutably borrowed" | E3022 stands; the text says "both are `v[0]`" (BRW-5.2) |
| `LT-5/reject_a_view_still_used_later_holds_its_owner.em:10` | E3021 at the push | `v = ref xs[0]` is a position, which a push keeps: run-pass printing 1 (BRW-16) |
| `LT-18/reject_mutation_while_the_matching_field_region_is_live.em:15` | E3021 at the push | E3060 at `println(pair.left[0])`, line 16 (BRW-17) |

Consistent with the target: `SPN-1/accept_a_view_that_ends_inside_the_call.em`, `accept_array_and_fixed_array_coerce.em`,
`accept_returning_an_implicit_str_of_a_borrowed_parameter.em`, `accept_string_coerces_to_str.em`;
`BRW-1/accept_the_view_ends_before_the_write.em`; `BRW-2/accept_a_borrow_ends_at_its_last_use.em`,
`accept_a_loop_keeps_an_element_across_its_turns.em` (`best = x` re-points `best`, BRW-13);
`BRW-4/accept_*.em` (2); `BRW-5/accept_two_different_constant_indices.em`, `reject_two_variable_indices.em`
(locals are not constants), `run_fail_split_boundary_past_length.em`; `LT-5/accept_*.em`;
`LT-18/accept_*.em`; `LT-25/*.em` (2). `SPN-1/accept_string_descriptor_fields.em` was not judged
beyond SPN-1 (it leans on `String.from` and `as_str`, which this batch did not check).

## 3. Spec questions and their rulings

Class: (a) the text decides it; (b) a text repair (wording, no change of meaning); (c) an ODR ruled
under the owner's delegation; (d) OPEN for the owner. None is (d). No owner verdict is touched. The
P column is the row of the Hardened_15 cut, made once at the end of M5's run.

| # | rule | the question | class | ruling | P |
|---|---|---|---|---|---|
| Q1 | BRW-8, FN-1 | a plain `Copy` parameter given a module variable that the callee writes by its own name: the value at the call, or what the other name wrote? | (a), **ruled by the lead** (review.md, round 7) | a plain parameter reads the caller's place, and a number another name writes across the call shows through it (`[FN-1]`); a copy in registers only where the summaries prove no other name writes it (`decisions.md` §4, area 2 Q4, on verdict 18). `accept_the_passing_choice_cannot_be_observed` prints 9 and 9 | P1: `[FN-1]`'s "(a number is copied in)" becomes "(a number may be copied in where no other name can change it while `a` is used)" |
| Q2 | BCK-4 | a view held by a local of an outer block whose target ends with an inner block: E3060 (clause 1) or E3063 (clause 2)? | (a) | E3060 at the later use. Clause 1 names "scope exit" as a storage end; were a local of an outer block clause 2's, clause 1's scope exit would have no case left, since every view live after an inner block is held in an outer local. Clause 2's "kept or returned" is a value leaving the function's locals (returned, put in through `ref mut`, a `static`, a field reached through a parameter or a link, a capture or thread that outlives the call). The old test and F-188 agree | P2: BCK-4 adds "(a value that a local of the same function holds is clause 1's: `E3060` at its later use)" |
| Q3 | BRW-13, BRW-15, TYP-14 | `r = ref x` then `r = 99`: E3023 (a change through a read-only name) or E2020 (an `int` given to a `ref int` place)? | (c) | E3023, with the line to write: "`r = 99` changes `x`, but `r` only reads it; to change `x` through `r`, write `r = ref mut x` in place of line N". One meaning of `=` for every reference name (principle 4): with a value of the referred type it writes what the name refers to, on a `ref mut` name (TYP-14) as on a `ref` one; on a read-only one that write is BRW-15's refusal. E2020 would read `=` on a `ref` name differently from `=` on a `ref mut` one, and its message could not give the line the program means. Test: `BRW-15/reject_a_change_behind_a_plain_name` | P3: BRW-13 and TYP-14 add the sentence and the message; `E3023`'s registry row adds "`=` with a value on a name that holds a read-only `ref`" |
| Q4 | BRW-13 | the E3023 messages name a new read-only name `rb = ref b`: how is it spelled for other names? | (b) | the refused name followed by the target's root name (`r` and `b`: `rb`; `t` and `second`: `tsecond`; `t` and `p.pos`: `tp`), or, where the function has that name, the first free of `rb_2`, `rb_3`, … as BRW-14's locals | P4: BRW-13 adds the form |
| Q5 | BRW-14 | (i) the message's noun for types other than a `StrView` and an `int`; (ii) what is offered, in the first-line case, for a type with no `Clone` | (i) (b); (ii) (c) | (i) "text" for `StrView` and `String`; "number" for every integer and float type; "list" for `Array` and `[T; N]`; "span" for `Span` and `MutSpan`; "map"; "set"; for any other struct, class or enum its own name in lower case, words apart (`GameClock`: "game clock"), as BRW-15's "the caller's clock" and EXC-15's "the player"; for a link or `owns T`, its target's; "value" for the rest (`bool`, `char`, a tuple, a type parameter). (ii) `[FN-2]`'s own advice, "a callee that needs a value of its own takes it `owns`": "for a handle of your own take it over: write `fn reopen(owns h: Handle) -> int:` and call it `reopen(owns first)` on line N", every call named (verdict 134), offered where each caller can hand its argument over (`[DIA-25]`); and `@derive(Clone)` above the type, with the first-line `new_h = copy of h`, where the type is the program's own and every field has `Clone`. Neither hides the parameter or copies silently. Test: `BRW-14/reject_where_the_local_of_your_own_is_made` | P5: BRW-14 adds the noun list and the no-`Clone` offer |
| Q6 | BRW-12, SPN-5 | BRW-12's `left, right = ref mut xs.split_at(mid)` against SPN-5's "an `Array` is split for changing as `a.as_mut_span().split_at(i)`" | (a) | no conflict: BRW-12's `xs` is undeclared and is a `MutSpan`, as SPN-3's `samples` is; on an `Array` the line is `ref mut xs.as_mut_span().split_at(mid)` | P6: BRW-12 adds "(`xs` a `MutSpan`, `[SPN-5]`)" |
| Q7 | BRW-8, LT-1 | a view-type parameter passed as a copy that also owns blocks: is `ref h.b.x` (into its `owns P`) a view into "the own storage of a parameter passed as a copy" (E3063)? | (a) | accepted. LT-1 itself names the two apart, "into its own storage or into storage it owns"; BRW-8's E3063 is for the copy's own bytes (`ref h.s`, `ref w`); the blocks the caller's value owns outlive the call, and the caller's argument stays viewed while the result lives (LT-1, "into"). `owns h` frees them at the return: E3063. Tests: `BRW-8/reject_a_view_into_a_copy_parameters_own_storage` | P7: BRW-8 adds "(its own bytes; what it owns is the caller's, `[LT-1]`)" |
| Q8 | LT-4, ALC-1 | LT-4's "keeps its instance's address, one word" against ALC-1's two kinds of manager that keep none (verdict 107) | (b) | LT-4 follows ALC-1, which states the owner's verdict: "one word, except in the two kinds of manager of `[ALC-1]` (verdict 107), whose values are still checked as holding the link" | P8 |
| Q9 | BRW-1, BRW-21, FN-1 | BRW-1 lists "a plain parameter" among views whose target no other name changes; inside `report(s: Score)` given `BEST`, `BEST.points = 5` changes what `s` reads | (a) | accepted, prints 5: BRW-21 ("for a plain or `Span` parameter, that what it views is not ended or moved") and FN-1 ("shows through"); `decisions.md` §4 settled it (area 2 Q4, verdict 18) | P9: BRW-1's bullet adds "(across a call, `[BRW-21]` says what is checked)" |
| Q10 | BRW-14, BRW-13, SPN-3 | `dst = ref mut other.as_mut_span()` on `ref mut dst: MutSpan` changes the caller's `d`; `c = ref mut spare` on `ref mut c: Counter` re-points `c` for the call only: one rule? | (a) | one rule, read by the right side: `ref mut` before a span names its elements (SPN-3), a value of the place's type, written into the caller's place as `r = 20` is (FN-1: "`ref mut best: Span[int]` may point the caller's view elsewhere"); `ref mut spare` is a ref to a `Counter` place, which gives the name another ref | — |
| Q11 | LT-4 | `thread.spawn(...)` then `t.join()` in the same function: still E3063? | (a) | yes: "may outlive every instance but a `static`", by the kind of thread, with no reasoning about `join` | — |
| Q12 | BRW-15 | in "`s` only reads the caller's clock; write `fn tick(ref mut s: Clock):` and call `tick(ref mut clock)`", is "clock" the type or the variable, and which call is named when there are several? | (b) | the noun is Q5's (the type's name in lower case); the call part gives each call as the caller writes it: with one call, the spec's form; with several, each with its line ("call `tock(ref mut clock)` on line 12 and `tock(ref mut spare)` on line 14"), since the new first line needs all of them (`[DIA-25]`, verdict 134). Test: `tock` in `reject_a_change_through_a_plain_name_gives_the_line` | P10 |
| Q13 | BRW-14, GRM-13 | a loop or pattern name bound to a number is "copied, as Python binds a number": does `=` on it keep its place (E3023)? | (a) | yes: GRM-13 says "and nothing is changed through the name (`E3023`)", and CTL-1 refuses `for v in xs.iter_mut(): v += 1`; the copy is how it is held, not a name of one's own | — |
| Q14 | BRW-8 (from Q1) | **new.** Under Q1, a register copy cannot show a write through `P`, so whether a small `Copy` parameter goes in registers depends on what the callee does; BRW-8 says "The choice depends only on the declared first line, never on the body". Options as code: `fn show_pair(p: Pair):` with `P.a = 9` then `println(p.a)`, called `show_pair(P)` and through `f: fn(Pair)`. (A) the ABI is by address for every plain parameter; registers only as the compiler's own optimisation where every call is seen and the summaries prove no other name writes the argument while it is used (Rust's `&Pair`, promoted by LLVM inside a package); cost: a store and a load per argument at a call the compiler cannot see through (a `pub` function across packages, a function value). (B) registers by the first line wherever the callee's summary (`[EFF-4]`, kept in the interface file) shows that nothing it reaches writes, while the parameter is used, a place the argument may be; a callee whose summary says one may (`show_pair`) also gets an entry by address, which the caller uses for such an argument (`show_pair(P)`); a function value or `dyn` call passes by address, so every function of one type has one ABI; cost: none where the summary clears the call, one more entry for the few functions that write a module variable or a link's field of the parameter's type | (c), for the lead to confirm (it completes the lead's Q1; it changes what BRW-8 plainly says, so it is not a repair) | **(B)**: it keeps register passing for every call the summary clears, across packages too (principle 8, speed on par with C), uses the summary the compiler already keeps (BRW-21's facts are body-derived and in the interface; a change of them is a change of the interface, as BRW-10 says for `pub` methods), and leaves the ABI of a function type uniform. (A) is simpler but puts every small plain `int` of a `pub` function behind a pointer | P11: BRW-8's "goes in registers when it is no larger than two pointers" and "The choice depends only on the declared first line, never on the body" become: "may go in registers when it is no larger than two pointers, where the callee's summary shows that no name it reaches writes the argument while the parameter is used (`[EFF-4]`); a call through a function value, or a `dyn`, interface or `virtual` method, passes it by address. Whether a parameter is passed by address or as a view's copy depends only on the declared first line; the choice cannot be observed" |
| Q15 | LT-1a, LT-45 | may `from` stand on a function whose result holds views but is not a view type (`-> Array[StrView] from s`)? | (a) | yes: BCK-1 says a call's result that may hold views carries the views of the arguments it may point into "(`[LT-1]`, `from`)", and TYP-15 makes a list of views a value holding views; E2031 is only for the names LT-1a lists. Today's compiler refuses it (E2031): an implementation gap for LT-1a's job | P12: LT-1a adds "(a result that holds views, `-> Array[StrView] from s`, included)" |
| Q16 | DIA-28, BRW-15, BRW-18 | `xs[..][2] = 100` inside `for x in xs:` (old `SPN-1/reject_writing_through_a_slice_inside_its_loop`): E3023 (a write through a read-only slice) or E3020 (a change of the walked list)? | (a) | E3020, by DIA-28's own precedent ("a scoped task started inside a walk of what it changes is `E3020` ... not `E7007`, whose fix ... leaves the walk refused"): E3023's fix (`xs[2] = 100`) stays refused inside the walk, and E3020's message can give the line that compiles ("write `xs[2] = 100` after the loop") | — |
| Q17 | BCK-3, BRW-1, BRW-5 | where is a conflict between two views reported: at the line that makes the second view, or at the read or change through it? | (c) | two names of elements of one list are refused at the line that makes the second (BRW-5: the names are E3022; BRW-1.5: "refused as two such arguments are"), unless that line is under a proof; any other conflict is reported at the read or change that the live view forbids (BCK-3: "a read or a change of a place ... is checked"), so that a proof may stand between the making and the use, as the spec's own `if t is not a:` needs (BRW-1); making a read-only view, or opening a link, is not itself such a read. Tests: BRW-1's element and link files | P13: BCK-3 adds the sentence |
| Q18 | BRW-5, STD-15 | BRW-5 names `swap(i, j)` among the library's ways to two changeable views; STD-15's list of `Array` methods has no `swap` | (b) | `Array` and `MutSpan` have `swap(i, j)`, which BRW-5 states; STD-15's list adds it | P14 |

Q1 to Q13 are the pilot's (Q1 ruled by the lead); Q14 to Q18 are this batch's.
