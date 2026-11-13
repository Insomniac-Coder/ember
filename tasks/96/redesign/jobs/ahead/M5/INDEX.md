# M5 ahead: tests and spec questions for A2-2 and A2-3 (pilot)

Written 2026-11-13 by the Opus deputy, one milestone ahead of the builders, from
`docs/spec-source/Ember_v0.9.20_Hardened_14.md` alone (`355cf49f`, tag `spec-h14`). Nothing was built
or run (the lead was timing a benchmark): every expectation is the spec's words, not the compiler's.
These files live here, not under `tests/`, because they fail until M5 is built. When a rule is built,
its directory moves to `tests/conformance/<RULE>/` and is checked with
`python tasks/impl-0.9.9/annotations.py tests/conformance/<RULE>`.

**Conventions.** Each file starts with `# Clause <RULE>.<n>: "<the clause, quoted>"`. One file per
clause; where a clause both accepts and refuses, it has an `accept_` and a `reject_` file. A refusal's
text is the spec's own example text where the spec gives one, with the file's own line numbers in
place of the example's (a file has a header, so its lines differ from the example's); where the spec
gives none, the expectation is `#$ error[CODE]` with no text, marked "none" below. Line numbers in
the expectations were resolved by a generator, not by hand.

**Rules.** A2-2 names BRW-8 (its E3063), LT-4 (its code), BCK-4 (its second half); A2-3 names
BRW-12, BRW-13, BRW-14, BRW-15, CTL-11 and SPN-1. Nine rules; the pilot's cap is eight, so the ninth
in order, **SPN-1, is not covered** (its four clauses are small; it goes with the full run). Each rule
was split whole, not only the part its item completes, since a job finishes whole rules.

**Totals.** 8 rules, 45 clauses, 63 test files (38 accept, 25 reject), 56 error expectations, of
which 37 have no text because the spec gives none. Of the 15 clauses that refuse something, 8 give no
message at all (BRW-8.5, LT-4.5, BCK-4.1, BCK-4.2, BRW-12.4, BRW-14.2, CTL-11.2, CTL-11.4) and 4 give
one for some cases only (LT-4.1, BRW-14.1, BRW-15.1, BRW-15.2).

**Other milestones these tests also need** (they turn green only when those have landed): the call
words `ref mut`/`owns`/`const ref mut` at calls (M3); a class object has one owner, `copy of`,
`weakref` (M4); the method part of BRW-15 (BRW-15.2), by summaries (M6a, A2-10); threads, `Atomic`,
`Mutex`, channels (M9: LT-4.2, BRW-15.3); `using` and `Allocator` (M10: LT-4.1, LT-4.3, LT-4.4);
kind tests `is T as c` (M11: BRW-14.1, CTL-11); positions (A2-4/A2-5, same milestone).

## 1. Rules, clauses and tests

Message column: "spec" = the spec's text; "derived" = the text the clause's own form gives for that
program (pinned only as far as the form decides); "none" = code only.

### BRW-8 (VII.4) — parameters by address or as a copy

| clause | what it says | test | message |
|---|---|---|---|
| 8.1 | plain and `ref mut` parameters are passed by address: the callee reads or changes the caller's place | `accept_plain_and_ref_mut_parameters_are_the_callers_place` | — |
| 8.2 | a view is passed as itself and keeps the regions it carries | `accept_a_view_parameter_keeps_the_regions_it_carries` | — |
| 8.3 | a small `Copy` value without a `Cell` goes in registers, except the object of a view-returning method | `accept_a_small_copy_parameter_and_the_exceptions` | — |
| 8.4 | an `owns` parameter receives the value | `accept_an_owns_parameter_receives_the_value` | — |
| 8.5 | a view into a copy parameter's own storage ends with the call: returning it is E3063 | `reject_a_view_into_a_copy_parameters_own_storage` | none |
| 8.6 | the choice depends only on the first line and cannot be observed | `accept_the_passing_choice_cannot_be_observed` (**pending Q1**) | — |

### LT-4 (VII.6) — values made in a memory manager's instance

| clause | what it says | test | message |
|---|---|---|---|
| 4.1 | a value made in an instance, and every value holding one, lives no longer than it: kept or returned past it is E3063 | `reject_a_value_made_in_an_instance_kept_past_it`; `accept_the_instance_the_caller_keeps` (the message's fix, DIA-25) | spec (returned); none (kept in the caller's list) |
| 4.2 | to an unscoped thread, a job outside a scope or a kept callback only from a `static` instance; else E3063 at the hand-over | `reject_an_instance_value_handed_to_an_unscoped_thread`; `accept_the_ways_out_for_a_thread` | spec |
| 4.3 | a value of a several-instance manager keeps its instance's address, a kept `sharedref mut` link: the instance stays usable, growth and end go through it | `accept_the_instance_is_used_while_its_values_live`; `reject_growth_through_the_link_while_a_ref_mut_name_is_used` (E3022) | none (see Q8) |
| 4.4 | a one-instance manager (`Global`, `System`) costs no word | `accept_a_one_instance_manager_costs_no_word` (`size_of`) | — |
| 4.5 | arena allocations point into the arena | `reject_an_arena_allocation_outliving_the_arena` (E3063, E3060); `accept_an_arena_allocation_within_the_arena` | none |

### BCK-4 (XVIII.4) — storage end

| clause | what it says | test | message |
|---|---|---|---|
| 4.1 | a place's storage ends (scope exit, handed over, replaced, ended) while a view of it is live: E3060 at the view's later use | `reject_a_view_used_after_its_place_ends` (**see Q2**); `accept_a_view_whose_last_use_comes_first` | none |
| 4.2 | a view, or a value holding views, kept or returned past what it points into (an instance's value included): E3063 | `reject_a_view_kept_or_returned_past_its_target`; `accept_a_view_returned_into_what_outlives_the_call` | none |

### BRW-12 (VII.4) — a name for a part is a read-only view

| clause | what it says | test | message |
|---|---|---|---|
| 12.1 | a name is declared by the line that first gives it a value; once per round in a loop body | `accept_a_name_in_a_loop_body_is_declared_each_round` | — |
| 12.2 | a name bound to a part or to a view-giving call is a read-only view: never a hand-over, never a silent copy | `accept_a_part_name_is_a_view_not_a_hand_over`; `reject_a_change_while_a_part_name_reads_it` | spec (BRW-1's `boss` text) |
| 12.3 | a part of a `Copy` type is copied (`w = this.rest`; `this.rest = w[i + 1..]` changes nothing `w` reads) | `accept_a_part_of_a_copy_type_is_copied` | — |
| 12.4 | a copied `Copy` struct is not changed through the new name while the source is used (E3023) | `reject_a_copied_copy_struct_changed_while_its_source_is_used`; `accept_a_copied_copy_struct_changes_once_its_source_is_done` | none |
| 12.5 | a call's new value is handed to the name, which owns it | `accept_a_value_a_call_made_is_owned_by_the_name` | — |
| 12.6 | `ref mut` before a part names that very place | `accept_ref_mut_before_a_part_names_that_very_place` | — |
| 12.7 | `ref mut` before a `MutSpan` (or tuple of them) a call gives lets the bound names write | `accept_ref_mut_before_a_call_that_gives_spans` (**see Q6**); `reject_a_plain_name_bound_from_a_call_giving_spans` | help only (SPN-3's line); text none |
| 12.8 | `copy of` makes a copy of your own | `accept_copy_of_a_part_is_a_copy_of_your_own` | — |
| 12.9 | the link words make a link | `accept_a_link_word_before_a_part_makes_a_link` | — |
| 12.10 | `ref` states the default, needed before a part only to view a number | `accept_ref_before_a_part_states_the_default`; `reject_a_ref_view_of_a_number_while_it_changes` | none |

### BRW-13 (VII.4) — `=` on a name that has a value

| clause | what it says | test | message |
|---|---|---|---|
| 13.1 | `=` puts the new value in the name's place; a name declared with `=` has its old value replaced (and ended) | `accept_the_old_value_is_replaced_and_ends` | — |
| 13.2 | a view name given a view now views that, checked by BRW-1 against the new target | `accept_a_view_name_given_a_new_view_views_that`; `reject_a_re_pointed_view_is_checked_against_its_new_target` | none |
| 13.3 | a `ref mut` name writes its place until given another ref (`r = ref mut b`) | `accept_a_ref_mut_name_writes_until_given_another_ref` | — |
| 13.4 | the line giving another ref says `ref mut` again; a read-only `ref b` given to it is E3023 | `reject_a_read_only_ref_given_to_a_ref_mut_name` (**see Q4**) | spec (for `r`/`b`); derived prefix (for `t`/`second`) |
| 13.5 | a name holding a `MutSpan` writes elements and is pointed at another span | `accept_a_name_holding_a_mutspan_writes_and_is_pointed_elsewhere` | — |
| 13.6 | `const ref mut` is never given another ref (E3023) | `reject_a_const_ref_mut_name_given_another_ref`; `accept_a_const_ref_mut_name_writes_its_place` | spec |

### BRW-14 (VII.4) — names that keep their place

| clause | what it says | test | message |
|---|---|---|---|
| 14.1 | a parameter not `owns`, a loop or pattern name that views, an opened or kind-tested name keep what they were given: `=` is E3023 | `reject_eq_on_a_parameter_gives_the_local_of_your_own`; `reject_eq_on_a_loop_pattern_opened_or_tested_name` | spec (both parameter texts); none (the other names) |
| 14.2 | so is any other change through one that is plain | `reject_a_change_through_a_plain_name_that_keeps_its_place` | none |
| 14.3 | the local is `new_` + the name, else the first free `new_n_2`, `new_n_3`, … | `reject_the_local_of_your_own_takes_a_free_name` | derived |
| 14.4 | made by the refused line if a top-level plain `=` with no read above; else a new first line (`new_n = n`, `new_s = copy of s`); never in a loop or branch | `reject_where_the_local_of_your_own_is_made` (**see Q5**) | derived |
| 14.5 | every later use is written with the local, each line named, as Python's rebinding | `accept_the_local_of_your_own_does_what_rebinding_does` | — |
| 14.6 | a name handed its value owns it: `owns` parameter, `for x in owns e:`, `match owns e:` names may change; `kept = job` hands over | `accept_a_name_handed_its_value_may_change` | — |
| 14.7 | `exists as copy of v` may change; the map is untouched | `accept_exists_as_copy_of_may_change` | — |
| 14.8 | a `ref mut` one writes the caller's place and may be given another ref (span: the caller's `d`; `Counter`: for the rest of the call) | `accept_a_ref_mut_parameter_given_another_ref` (see Q10) | — |
| 14.9 | `const ref mut dst` writes the caller's elements, never given another ref | `accept_a_const_ref_mut_parameter_keeps_the_callers_span`; `reject_a_const_ref_mut_parameter_given_another_ref` | none |

### BRW-15 (VII.4) — a plain name never changes anything

| clause | what it says | test | message |
|---|---|---|---|
| 15.1 | a plain parameter, loop, pattern or opened name, and a plain local holding a view, change nothing they name or lead to (`Cell`, `shares` target, `MutSpan` elements): E3023 with the line to write; an iterator a call made is owned and may advance | `reject_a_change_through_a_plain_name_gives_the_line` (the four texts); `reject_a_change_behind_a_plain_name` (**see Q3**); `accept_the_lines_the_messages_give` | spec (four); none (`Cell`, `shares mut`, part name, `r = 99`) |
| 15.2 | a method that may change its object, called through a read-only name, is E3023 | `reject_a_changing_method_through_a_read_only_name`; `accept_a_reading_method_through_a_read_only_name` | spec (EXC-15's `heal`); none (through a parameter) |
| 15.3 | `Mutex`, `RwLock`, `Atomic`, channel ends and `guarded by` fields under their lock are used through any name; handed to a call they take no word | `accept_types_shared_between_threads_change_through_plain_names` | — |

### CTL-11 (VI.4) — headers that name what they test

| clause | what it says | test | message |
|---|---|---|---|
| 11.1 | `if`/`while ... exists as p:` and `if`/`elif ... is T as c:` run their block as GRM-44 and DSP-4 say | `accept_headers_that_name_what_they_test` | — |
| 11.2 | the name is in scope in that block only, never in an `elif` or `else` | `reject_the_name_used_outside_its_block` (E1010) | none |
| 11.3 | `else:` runs when nothing is held or the kind differs | `accept_else_runs_when_nothing_is_held_or_the_kind_differs` | — |
| 11.4 | the name keeps its place: a plain one only reads; `exists as copy of v` is a copy of your own | `reject_a_change_through_a_plain_header_name`; `accept_a_copy_opened_from_a_header_changes` | none |

## 2. Old tests the target contradicts

Under `tests/conformance/` unless said. "Pilot rule" is the rule of this pilot whose Hardened_14 text
decides it; changes from other rules are listed after, in one line each.

| file:line | pins | Hardened_14 says instead (rule) |
|---|---|---|
| `BRW-8/accept_a_copy_struct_holding_a_cell_is_passed_by_address.em:13` | `t.count.set(...)` through plain `t` runs | E3023: a plain parameter changes nothing behind a `Cell` (BRW-15); by-address passing is observed through a module variable instead (this pilot's BRW-8.3) |
| `FN-1/accept_cell_and_refcell_through_borrowed_parameters.em:13,19` | `s.frame.set` through plain `s`; `RefCell` | E3023 at line 13 (BRW-15); `RefCell` is gone (E1010) |
| `BRW-1/accept_writing_through_a_mut_ref.em:12` | `m = ref mut y` writes `y` into `x`, prints 5 5 | re-points `m` (BRW-13.3): prints 1 5 |
| `BRW-1/reject_a_reference_is_not_re_pointed.em:17` | E3021 for `hottest = r` | accepted, re-points `hottest` (BRW-13.2); run-pass printing 3.0 (this pilot's BRW-13.2 test is that program) |
| `BRW-1/reject_writing_through_a_shared_ref.em:16` | E3021 for `r = 99` on a `ref` name | E3023 under the proposed Q3 ruling (BRW-15); the text does not settle it |
| `DIA-9/reject_a_diagnostic_never_suggests_unsafe_while_a_safe_fix_compiles.em:3` | E3023 "cannot mutate borrowed parameter `values`" | E3023 stays; the text is BRW-15's form, with the line to write (`fn append_one(ref mut values: Array[i32]):`, call `append_one(ref mut values)`); it is BRW-15's case, not BRW-14's |
| `FN-2/reject_a_plain_parameter_is_not_changed.em:6` | same old text | as DIA-9 (BRW-15) |
| `LT-44/accept_one_arena_parameter_needs_no_borrows.em:8`; `LT-44/accept_a_fixed_or_scoped_arena_parameter_is_a_source.em:12,15,18` | `arena.alloc` through a plain `arena` | E3023: `alloc*` changes the arena, so through a plain parameter it is BRW-15.2's refusal (ARN-1: "called through a name that may change the arena") |
| `LT-4a/*.em` (4 files, `allocate_one(arena: Arena)` lines 4-7), `TYP-15/reject_arena_backed_view_in_box.em:7` | the same plain-arena `alloc` | E3023 (BRW-15.2, ARN-1); every one needs `ref mut arena` and `ref mut` at its calls |
| `LT-4a/reject_reset_while_a_wrapped_allocation_is_live.em:10` | E3021 at `arena.reset()` | `reset()` ends every view into the arena: E3060 at `println(value)`, line 11 (LT-4.5, ARN-1) |
| `LT-1/reject_a_reference_to_a_view_parameter_itself.em:9`; `LT-1a/reject_a_reference_to_a_named_view_parameters_slot.em:8` | E3060 | E3063 (BRW-8.5) |
| `LT-1/reject_a_reference_into_a_view_parameters_own_storage.em:21,24` | E3060 at both | line 21 (`owns h`): E3063 (LT-1); line 24 (plain view-type `h`, `ref h.b.x`): E3063 or accepted, **Q7** |
| `SPN-5/accept_views_of_a_view_parameters_target.em:19,30` | `t[0] = 20` through a plain name from `split_at` | E3023 (BRW-12.7, BRW-15): `parts = ref mut s.split_at(1)` and `t = ref mut tail(...)`; also `tail(ref mut ...)` at the call (FN-2a) |
| 31 files with "cannot write through a shared …" or "cannot mutate borrowed parameter" (list: `grep -rln "cannot write through a shared\|cannot mutate borrowed parameter" tests/`) | E3021 for a write through a read-only name (15 files), or E3023 with the old text (the rest) | candidates for BRW-15: a write through a read-only name is E3023 with the line to write. Not each read in the pilot: some are methods (BRW-15.2, M6a), some `FFI-40a` const methods, some ECS queries |

Other rules' changes seen in the same files (not this pilot's rules): `EXP-4/*outliving_its_temporary.em`
(2) and `TXT-10/reject_*` (3) refuse a view of a temporary used on the next line; BRW-19/EXP-4 now
keep the temporary until the view's last use, so all five are run-pass (A1-18, also M5).
`EXC-18/accept_a_*getter*.em` (2) alias a class object with `q = p` / `alias = p`, now a hand-over
(OWN-1, OWN-3: E3040 at the later use of `p`). `ARN-4/reject_using_fixed_backing_while_the_arena_is_live.em:7`
is E3022, not E3021 (the fixed arena holds a kept `ref mut` view, ARN-4, BRW-1), with
`Arena.fixed(ref mut bytes)`. Most arena tests lack `owns` at `alloc(…)` (ARN-3, FN-2a: E3028).
Consistent with the target, no change: `BRW-14/*` (both), `BCK-4/*` (both, pending Q2),
`BRW-8/accept_small_copy_parameter_by_value.em`, `BRW-8/reject_an_instance_value_that_takes_by_address.em`,
`LT-1/accept_rule_1_takes_any_borrowed_receiver.em`, `LT-1/accept_a_call_returning_a_ref_is_a_place.em`,
`LT-1a/reject_borrows_names_a_copy_struct_holding_a_cell.em`, `OWN-8/accept_derived_enum_clone.em`.

## 3. Spec questions

Class: (a) the text decides it; (b) a text repair (wording, no change of meaning); (c) an ODR I would
rule; (d) OPEN for the owner. No hardening was cut: these are proposals for the lead.

| # | rule | the question | class | proposed answer |
|---|---|---|---|---|
| Q1 | BRW-8, FN-1 | `fn show(p: Pair)` (`Pair` `Copy`, two ints, in registers) given `P`, a `static mut`, whose body sets `P.a = 9` then prints `p.a`: BRW-8 says the passing "cannot be observed" and FN-1 says a number another name writes across the call "shows through `a`", yet FN-1 also says "(a number is copied in)". In registers it prints 1, by address 9; a four-int `Quad` (by address) would print the other, so the ABI choice is observed. | (c) — it decides what a program prints; the lead may prefer to send it to the owner (d) | **The value at the call, for every plain parameter of a `Copy` type holding no `Cell`, whatever its size**: OWN-7 already says a `Copy` value "is duplicated ... on use", and FN-1 "a number is copied in". FN-1's "shows through" is narrowed to parameters passed by address (not `Copy`, or holding a `Cell`). A large `Copy` value still goes by address; where the callee's summary (BRW-21's, which exists) changes the argument's place by another name while the parameter is used, the caller passes the address of a copy, so nothing is observed and nothing is paid elsewhere. Rejected: refusing the call (refuses a correct program, principle 7); by address always (registers would depend on the body). Test: `BRW-8/accept_the_passing_choice_cannot_be_observed` |
| Q2 | BCK-4 | A view stored in a local of an outer block whose target ends with an inner block (`r = Some(ref x)` inside `if`, `println(r)` after): clause 1 says E3060 at the later use ("scope exit"); clause 2 says a view "kept ... where it would outlive what it points into" is E3063. Both fit; DIA-26 wants one. The old test and F-188 (shape B7) say E3060. | (b) | E3063's "kept" means kept where the value leaves the function's locals (returned, put in through `ref mut`, a `static`, a field of something reached through a parameter or link, a capture or thread that outlives): there no later line of the function can be named. A value a local of the same function holds is judged by its uses: E3060 at the later use, which is the line to move. Add to BCK-4 clause 2: "(a value a local of the same function holds is clause 1's: `E3060` at its later use)". |
| Q3 | BRW-13, BRW-15 | `r = ref x` then `r = 99`: a value, not a view, given to a read-only `ref` name. BRW-13 says a name declared with `=` has its value replaced, which a `ref int` cannot take from an `int`; TYP-14 says `=` on a `ref mut` name writes its place. Is it E3023 (a change through a plain name) or E2020 (wrong type)? Old test: E3021. | (b) | E3023, by the symmetry with `ref mut` (`=` with a value writes the place) and BRW-15 (a plain name holding a `ref` changes nothing), with the line to write: "`r` only reads `x`; to change `x` through `r`, write `r = ref mut x` in place of line N". Add the sentence and the message to BRW-13. Test: `BRW-15/reject_a_change_behind_a_plain_name` |
| Q4 | BRW-13 | The E3023 messages name a new read-only name `rb = ref b` and a new changing one `rb = ref mut b`, for `r` and `b` only. How is it spelled for other names (`t`, `second`), and when it is taken? | (b) (message wording) | The `ref mut` name followed by the target's name (`rb`, `tsecond`), or the first free of `rb_2`, `rb_3`, … as BRW-14's `new_n_2`. The test pins only the prefix the spec decides. |
| Q5 | BRW-14 | (i) The message's noun is given for a `StrView` ("text") and an `int` ("number") only; "one form for every parameter" needs one per type. (ii) "`new_s = copy of s` ... offered only where `copy of` applies": what is offered for a type with no `Clone`, in the first-line case? | (i) (b); (ii) (c) | (i) The noun other messages use: "list" (`Array`), "span" (`Span`, `MutSpan`), "map", "set", "string" (`String`), and a struct or class's own name in lower case ("clock", "player"), as BRW-15's "the caller's clock" and EXC-15's "the player" do. (ii) Offer taking it over, which needs nothing the type lacks and is FN-2's own advice ("a callee that needs a value of its own takes it `owns`"): "for a `Handle` of your own take it over: write `fn f(owns h: Handle):` and call it `f(owns x)` on line N", naming every call (verdict 134); also offer `@derive(Clone)` where the type is the program's own. Not tested until ruled. |
| Q6 | BRW-12, SPN-5 | BRW-12 writes `left, right = ref mut xs.split_at(mid)`; SPN-5 says "an `Array` is split for changing as `a.as_mut_span().split_at(i)`" and `split_at` on a `Span` gives `Span`s. If `xs` is an `Array`, the two disagree. | (b) | BRW-12's example becomes `ref mut samples.split_at(mid)` as in SPN-3 (a `MutSpan`), or `ref mut xs.as_mut_span().split_at(mid)`. The test uses a `MutSpan` name. |
| Q7 | BRW-8, LT-1 | A view-type parameter passed as a copy that also owns blocks (`@view struct H` with `s: Span[int]`, `arr: Array[int]`, `b: owns P`): is `return ref h.arr[0]` / `ref h.b.x` a view into "the own storage of a parameter passed as a copy" (E3063) or into blocks the caller's value owns (accepted, the caller's argument kept viewed)? | (c) | Accepted. "Own storage" is the copy's own bytes (`ref h.s`, `ref h` are E3063); what its owners hold is the caller's and outlives the call, as LT-1's "into everything it owns" reads for any source. The old test's line 24 becomes run-pass, line 21 (`owns h`) stays E3063. |
| Q8 | LT-4, ALC-1 | LT-4: "A value whose manager may have several instances keeps its instance's address, one word". ALC-1 (verdict 107): no word for a manager whose `deallocate` does nothing (unless the value grows) or that implements `FoundByAddress`. LT-4 states no exception. | (b) | LT-4 adds "except in the two kinds of manager of `[ALC-1]` (verdict 107)". The owner ruled verdict 107, so LT-4 follows it. |
| Q9 | BRW-1, BRW-21, FN-1 | BRW-1's read-only bullet lists "a plain parameter" among views whose target no other name changes (E3021 at the change); inside `report(s: Score)` given `BEST`, `BEST.points = 5` changes what `s` reads. | (a) | Decided by BRW-21 ("for a plain or `Span` parameter, that what it views is not ended or moved") and FN-1 ("shows through"): accepted, prints 5. A cross-reference in BRW-1's bullet ("across a call, `[BRW-21]` says what is checked") would stop a builder refusing it. |
| Q10 | BRW-14, BRW-13, SPN-3 | `dst = ref mut other.as_mut_span()` on `ref mut dst: MutSpan` changes the caller's `d`; `c = ref mut spare` on `ref mut c: Counter` re-points `c` for the rest of the call only. One rule? | (a) | Yes, read by the right side: a `MutSpan` (what `ref mut` before a span gives, SPN-3: "before a span names its elements") is a value of the place's type and is written into the caller's place, as `r = 20` is; `ref mut spare` is a view of a `Counter` place and re-points. |
| Q11 | LT-4 | `t = thread.spawn(owns fn() owns st: ...)` followed by `t.join()` in the same function: still E3063? | (a) | Yes: "An unscoped thread ... may outlive every instance but a `static`", by its kind, with no reasoning about `join`; the message says "may outlive". The test keeps the `join`. |
| Q12 | BRW-15 | "`tick` sets `s.frame`, but `s` only reads the caller's clock; write `fn tick(ref mut s: Clock):` and call `tick(ref mut clock)`": is "clock" the type's name or the caller's variable, and which call is named when there are several? | (b) | The noun is the type's name in lower case (as EXC-15's "the player"); the call part names every call with its line ("call it `tick(ref mut clock)` on line 12 and `tick(ref mut spare)` on line 15"), since the signature change needs all of them (DIA-25, verdict 134). The test has one call, named `clock`. |
| Q13 | BRW-14, GRM-13 | A loop or pattern name bound to a number is "copied, as Python binds a number" (GRM-13): does `=` on it still keep its place (E3023)? | (a) | Yes: GRM-13 itself says "and nothing is changed through the name (`E3023`)", and CTL-1 refuses `for v in xs.iter_mut(): v += 1`. The copy is how it is held, not a name of your own; `exists as copy of v` is the way to one. |
