# M14 ahead: tests and spec questions for std.ecs and std's semantics, written before the builders

Written from `docs/spec-source/Ember_v0.9.20_Hardened_14.md` alone (`355cf49f`, tag `spec-h14`) by the
Opus deputy D2 on 2026-11-20, on `deputy` after `main`. M14 is MILESTONES.md's "std.ecs as EnTT's
design, and the rest of std's semantics": A8-1 (`type_name[T]()`, `ecs.type_hash[T]()`), A8-2
(one-element tuples, made moot for the ECS by verdict 115), A8-3 (the `Registry` and its views),
A8-4 (ECS-8), A8-5 (the 0.9.15 forms), A8-7 (STD-19, STD-26, STD-11), A8-8 (STD-15, STD-16), A8-11
(the three stale design tests) and A8-12 (the measurement against EnTT). Verdict 115 gives typed
variadic generics, so a view lists any number of types in its brackets (`reg.view[Position,
Velocity]()`, one alone `reg.view[Position]()`), with no tuple and no limit; verdict 116 writes a pack
with C++'s dots, which no file here needs to declare. Nothing was built: every expectation is the
spec's words and codes. No hardening is cut here.

**Skipped.** No rule of the milestone is credited. Covered by `tests/conformance` and not repeated:
STD-15 (26 files; `extend(owns items)` is left out besides, as the lead is building it), STD-19 (50,
the lending of a named function `[FN-2a]`/ODR-240 included), STD-26 (22), STD-11 (6, views as keys
and their `E3063`), STD-16's `key not found` and `or_insert_with`. Covered by the other banks:
`sorted(m.keys())` ended by an insert (`ahead/M6a/DIA-7`, `ahead/M5/BRW-17`), `LOG.retain(keep)`'s
`E3022` (`ahead/M6a/BRW-11`), `get_pair_mut` (`ahead/M5/BRW-5`), `counts[k] *= 2` in a map walk
(`ahead/M6a/DIA-10`). The gap map's A8-7 line says a named function given to `owns f: F` is handed
over; the spec now lends it (`[STD-19]`, `[FN-2a]`, ODR-240), and the existing STD-19 tests follow the
spec. Not written: ECS-8's `on_update`, `on_destroy`, `sort_as`, `Snapshot`, `SnapshotLoader`,
`ContinuousLoader` (an archive from `std.ser`), `RuntimeView` and `Handle` (a second batch if wanted);
A8-11's rewrites of the three design tests, which need the owner's sign-off (ECS-4's reject and
ECS-5's files give the new meanings: `ref mut` on a read-only view's item is `E3027`, a change during
a walk is collected first); A8-12, a measurement.

**Conventions** (as the other banks). Each file starts with `# Clause <RULE>.<n>: "<the clause,
quoted>"`; one file per clause and outcome; a refusal's text is the spec's own with the file's line
numbers, else code only; line numbers come from markers. A run-fail file whose stop text the spec
does not give claims the stop only. A case is one `#$ error[...]` or `#$ stdout:` line, or a
run-fail's stop.

**The probe** (the release `ember.exe` of 2026-11-19, one file at a time; the std's own `Arena`
error set aside). Today's `std.ecs` is the removed `World`/`Query` API, so every ECS file stops at
`module 'std.ecs' has no item named Registry`: not built. Two faults were fixed: a lambda's untyped
parameters follow the comma rule (`fn(ref mut r, ref e)`, not `fn(ref mut r, e)`: `E0116`,
`[GRM-42]`); a loop over a list literal whose `String`s are handed over is `for w in owns [...]`
(`E3013` otherwise). While writing this bank one fault was found in the merged M6a bank and fixed in
its own commit: `ahead/M6a/DIA-25`'s accept left `k` unread in `for k, ref mut v in counts:`, which
is `L1001` (`[LNT-1]`), an unclaimed warning; the loop prints `k` now. STD-16's accept checks clean
today; none was run.

**Totals.** 12 rules, 16 clauses, 20 files (10 accept, 7 reject, 3 run-fail), 57 cases: 12 error
expectations (3 with the spec's whole text, 1 in part, 8 code only), 42 output lines and 3 stops.

## 0. Rules covered, and what else each needs

| rule | clauses | files | cases | also needs |
|---|---|---|---|---|
| ECS-1 | 1 | 1 | 4 | A8-3 (the `Registry`) |
| ECS-2 | 1 | 1 | 3 | A8-1 (`type_id`, `type_name`, `type_hash`); M11 (`owns dyn`, a cast of a place) |
| ECS-3 | 3 | 4 | 14 | verdict 115 (variadic generics, `[TYP-47]`); Q2, Q3 |
| ECS-4 | 1 | 2 | 8 | verdict 115; Q4 |
| ECS-4a | 1 | 1 | 3 | verdict 115 |
| ECS-5 | 1 | 2 | 3 | M5 (`[BRW-18]` through calls, M6a) |
| ECS-6 | 1 | 1 | 2 | none |
| ECS-7 | 1 | 1 | 4 | none |
| ECS-8 | 2 | 2 | 7 | M8 (capture words); Q1 |
| ECS-9 | 1 | 2 | 2 | M9 (the task check, `[THR-11]`) |
| STD-14 | 1 | 1 | 1 | A8-3 |
| STD-16 | 2 | 2 | 6 | none |

## 1. Rules, clauses and tests

| rule.clause | what it says | test | message |
|---|---|---|---|
| ECS-1.1 | a 32-bit generational entity, 20-bit index and 12-bit generation; `Entity.NULL` | `accept_an_entity_is_a_32_bit_generational_handle` | — |
| ECS-2.1 | one pool per type keyed by `type_id`; `storage[T]()` as a place; `type_hash` stable | `accept_one_pool_per_type_keyed_by_the_type_itself` | — |
| ECS-3.1 | the registry's methods, any number of types in brackets | `accept_the_registrys_methods` | — |
| ECS-3.2 | `replace` and `erase` stop when the component is absent | `run_fail_replace_or_erase_of_an_absent_component_stops`; `run_fail_erase_of_an_absent_component_stops` | stop only |
| ECS-3.3 | a view a method gives points into the registry, which is not changed through another name while it is used | `reject_a_view_the_registry_gave_is_kept_across_a_change` (E3022, E3060) | none |
| ECS-4.1 | views read-only and `ref mut`, `exclude`, the view's methods, any number of types | `accept_views_read_change_and_leave_out` (the section's `integrate` under `@noalloc`; one type; four); `reject_a_read_only_view_changes_nothing` (E3027, E3023) | none |
| ECS-4a.1 | the types of one list are distinct: E2074, at each use of a generic | `reject_a_type_listed_twice_in_one_view` | spec; none; part |
| ECS-5.1 | changing the registry while a view walks it: E3020; collect first | `reject_changing_the_registry_while_a_view_walks_it`; `accept_collect_first_and_change_after` | spec; none |
| ECS-6.1 | insertion order with swap-remove holes | `accept_iteration_order_is_insertion_order_with_swap_remove_holes` | — |
| ECS-7.1 | any type is a component | `accept_any_type_is_a_component` (a number, a `String`, an enum, a plain struct) | — |
| ECS-8.1 | signals (`connect` with `owns`, `disconnect` by comparing; a listener may change the registry), `sort`, `ctx`, `group` | `accept_signals_sorting_context_and_groups` | — |
| ECS-8.2 | a listener viewing a local the registry outlives | `reject_a_listener_that_views_a_local_the_registry_outlives` (E3063 as ECS-8 says: Q1) | none |
| ECS-9.1 | no scheduler; two pieces of work at once on one pool, one changing it, are refused | `reject_two_tasks_that_may_touch_one_pool` (E7007); `accept_the_program_calls_its_own_functions_in_order` | none |
| STD-14.1 | `Registry.emplace` on a dead entity panics | `run_fail_emplace_on_a_dead_entity_stops` | stop only |
| STD-16.1 | `entry(k).or_default()` is a place; `for k, ref mut v in m:`; `values_mut()` | `accept_entries_as_places_and_values_changed_by_the_loops_name` | — |
| STD-16.2 | nothing changes a map while a loop walks it: E3020 | `reject_changing_a_map_while_a_loop_walks_it` (`[BRW-18]`'s `counts.insert` message) | spec |

## 2. Old tests the target contradicts

- Every file in `tests/conformance/ECS-1` to `ECS-7` (34) uses the removed `World`, `Query`, `System`,
  `run_parallel` and `Commands` API: the gap map's 1 respell, 10 rewrites and 16 retirements, landing
  with A8-3 (and `E7020` retired with them).
- `run4/v5/conformance/R4/reject_mut_in_a_query_is_now_ref_mut.em`,
  `R4/reject_ref_mut_on_a_query_item_the_query_only_reads.em`,
  `R8/accept_ecs_changes_are_queued_while_a_query_walks.em` (A8-11, the owner's sign-off).

## 3. Spec questions

None is decided here. M9's Q4 (`[GRM-17]` against bracketed lambdas with a compound statement) is
logged; no file here writes one.

| # | rule | the question | readings | recommended |
|---|---|---|---|---|
| Q1 | ECS-8, CLO-4, LT-42 | a listener that captures a local with `ref` and is kept by a registry that outlives the local: `[ECS-8]` says `E3063` ("a listener that views a local the registry outlives"); `[CLO-4]` and `[LT-42]` say `E3026` for a function value that captures a view with `ref` and outlives it | (a) `E3026`, shape B9, whose help gives the capture word that fixes it (`fn(ref mut r, ref e) copy of name:`); (b) `E3063`, shape B12, a value holding views kept past them | **(a)**: the function-value rule is the one that names the case, and its help compiles; ECS-8's parenthesis is repaired to `E3026`. The file follows ECS-8's text (b), and flips when ruled |
| Q2 | ECS-3 | `orphan(e)` has no result in the spec; EnTT's `orphan` tests whether an entity has no components | (a) `orphan(e) -> bool`, EnTT's test; (b) it removes every component of `e` | **(a)**, EnTT's meaning, as the section says it is EnTT's design. The file calls it and does not read a result |
| Q3 | ECS-3 | `create_at(hint)` when `hint`'s index is in use: the spec gives the method, not that case | (a) as EnTT, an entity of another index; (b) a stop (a caller's bug, `[STD-14]`) | **(a)**. The file gives a free hint |
| Q4 | ECS-4 | `each(f)`: what `f` receives is not written | (a) the entity and the items, with the view's words (`fn(e, p)` for `view`, `fn(e, ref mut p)` for `view_mut`); (b) the items alone; (c) either, by `f`'s arity, as EnTT | **(a)**, the shape of the view's own `for e, p in` loop. File: `ECS-4/accept_views_read_change_and_leave_out` |
