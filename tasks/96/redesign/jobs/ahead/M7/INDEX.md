# M7 ahead: tests and spec questions for loops of owners and parent links, written before the builders

Written from `docs/spec-source/Ember_v0.9.20_Hardened_14.md` alone (`355cf49f`, tag `spec-h14`) by the
Opus deputy D2 on 2026-11-20, on `deputy` after `main`. M7 is MILESTONES.md's "Loops of owners
refused unless opened (E3032); parent links and the walk": A3-9 (E3032 and the owner-edge graph),
A3-10 (the leak report and the cycle commands) and A3-11 (the parent-link proof, the walk with a
link, the deferred-drop region), "with the measured salvages" (`phase3b-salvage.md`: costs of the
region and the parent-link proof, which change no expectation here). `decisions.md` names them in
area 3 Q1 (a store whose owner type can reach the holder's type closes a loop unless proved
otherwise; the proofs went into `[WK-7]`), Q3 (a deferred-drop region runs the drops at once; only
the blocks' memory waits; now in `[WK-18]`) and the `[WK-7]` row. Nothing was built: every
expectation is the spec's words and codes. No hardening is cut here.

**Skipped.** Credited (`built_for_0915`): WK-2, WK-4, WK-8, WK-9 (A3-10's report and the cycle
commands), RC-2a, RC-6. Covered by the other banks and pointed to, not repeated: a loop of owners
through a function value's state (`ahead/M8/CLO-5/reject_a_loop_of_owners_through_a_functions_state`);
`[WK-3]` (`ahead/M6a/WK-3`); the store-line `E3031` for a stored `prune` (`ahead/M6a/BRW-21`'s
tree files) and the walk that keeps a `weakref` across a call (`ahead/M6a/BCK-7`); an object handed
to a thread leaving its links behind (`ahead/M9/THR-2`). Not written: `[WK-7]`'s cases across a
package's edge (`h.add(owns b)` with `lib`, `install` and its plugin, `[MOD-9a]`'s `Keeper.attach`:
each needs two packages); loops through `thread_shared` owners and channel ends (they need M9's
`guarded by` to change a `thread_shared` field, and a queue as an edge); `[WK-15]`'s debug report,
now only of `unsafe` or foreign leaks (`into_raw()`), which the runtime prints after the program:
a milestone test.

**Conventions** (as the other banks). Each file starts with `# Clause <RULE>.<n>: "<the clause,
quoted>"`; one file per clause and outcome; a refusal's text is the spec's own with the file's line
numbers, else code only; line numbers come from markers. A case is one `#$ error[...]` or `#$
stdout:` line.

**The probe** (the release `ember.exe` of 2026-11-19, one file at a time; `static mut` read as
`static` in a copy; the std's own `Arena` error set aside). One fault was fixed: a field named
`owned`, a word Ember no longer has (`E1010`), renamed `forward`. While writing this bank one fault
was found in the merged M9 bank and fixed in its own commit: `ahead/M9/WK-12`'s thread captured its
link `copy of w`, but a `weakref` link is not `Copy` and `copy of` before one is `E2116`
(`[WK-11]`); it now captures `fn() weakref st:`. Everything else is not built: today's `L3001`
warning stands where `E3032` refuses or nothing is said; re-pointing `cur = ref p` to a name opened
in the `while` header is `E3060` today, where `[WK-17]`'s own walk writes it; capture words. WK-18's
link walk checks clean today; none was run.

**Totals.** 6 rules, 11 clauses, 11 files (6 accept, 5 reject), 31 cases: 8 error expectations (2
with the spec's whole text, 6 code only) and 23 output lines.

## 0. Rules covered, and what else each needs

| rule | clauses | files | cases | also needs |
|---|---|---|---|---|
| WK-1 | 1 | 1 | 4 | none |
| WK-5 | 2 | 2 | 3 | none |
| WK-6 | 2 | 2 | 7 | none |
| WK-7 | 3 | 3 | 9 | M6a's summaries (a callee's stores, `[EFF-24]`); M8 (capture words, the panel's handler); M2 (`static mut`); Q2 |
| WK-17 | 1 | 1 | 2 | Q1 (the code check) |
| WK-18 | 2 | 2 | 6 | M6a (`[RC-5]`'s check through calls) |

## 1. Rules, clauses and tests

| rule.clause | what it says | test | message |
|---|---|---|---|
| WK-1.1 | no cycle collector; a link back that keeps nothing alive is a `weakref` | `accept_a_link_back_that_keeps_nothing_alive` (both `drop`s run) | — |
| WK-5.1 | what a type owns alone is an owner edge to what that owns | `reject_a_loop_through_what_its_objects_own_alone` (E3032) | none |
| WK-5.2 | links and views are no owner edges; a `let` field's line, at the making, closes none | `accept_links_views_and_let_fields_close_no_loop` | — |
| WK-6.1 | a line that closes a loop of owners is E3032 unless the loop is proved opened | `reject_a_loop_of_owners_nothing_opens` (the spec's program) | spec |
| WK-6.2 | the same function opening it is accepted; nothing runs for it | `accept_the_loop_opened_before_its_objects_end` (both ways out, with the `drop`s) | — |
| WK-7.1 | proved: opened in code the compiler follows, or by the owning object's `clear()`; refused: an opening known only while running, a loop handed out of reach | `accept_what_counts_as_proof` (a callee; the owner's `clear`, Q2); `reject_openings_the_compiler_cannot_follow` (a condition, returned, stored in a module variable) | none |
| WK-7.2 | a store closes none when the value was made here and cannot reach the holder, or the holder is fresh | `accept_what_counts_as_proof` (top-down tree; a fresh holder) | — |
| WK-7.3 | a store of a parameter is judged at each call; a function value's state owners are its edges | `reject_a_loop_closed_through_a_call` (`link`'s second call; the spec's panel and button) | none |
| WK-17.1 | a parent link read with no header test where both conditions prove its target lives; the walk up with views | `accept_a_walk_up_a_tree_with_views` (Q1) | — |
| WK-18.1 | a walk with views across a call that may end nodes is E3021 | `reject_a_walk_with_views_across_a_call_that_may_end_nodes` | spec |
| WK-18.2 | the walk with a link stops at the first ended parent; the drops of what an ended node owns run at once | `accept_a_walk_with_a_link_stops_at_the_first_ended_parent` | — |

## 2. Old tests the target contradicts (seen, not a sweep)

- `tests/conformance/WK-5/warning_*.em` (3), `WK-6/warning_direct_strong_self_cycle.em`,
  `WK-7/warning_generic_class_cycle_after_substitution.em`: `L3001` warnings; `L3001` is retired
  (its row: "a loop of owners is refused when compiling (`E3032`)"), so each is an `E3032` reject, or
  an accept where the loop is proved opened (the gap map's 5 rewrites).
- `WK-15/accept_a_debug_run_reports_a_leaked_cycle_by_default.em`: a loop of owners no longer leaks
  in Safe Ember; the report's leaks come from `unsafe` `into_raw()` or foreign code (the gap map's
  rewrite).

## 3. Spec questions

None is decided here.

| # | rule | the question | readings | recommended |
|---|---|---|---|---|
| Q1 | WK-17, EFF-10, OBS | "A read of one is compiled with no header test" is a fact of the emitted code, not of what the program prints. How is it tested? | (a) an `#$ assert-c` needle for the runtime's link test, named by the builder (C backend only); (b) `ember inspect --safety`, whose side table lists every emitted check with its kind and reason (`[EFF-10]`, `[EFF-11]`): no `Stale` check at the walk's reads, one at each read where condition 1 fails | **(b)**: it reads the spec's own report, the same on both backends, and also shows where a read keeps its test. A milestone test beside the run-pass file |
| Q2 | WK-7 | "opened by its owning object (a `clear()` it always calls before its last owner ends)": what makes a `clear()` "always called"? | (a) the owning class's `drop` calls it, since a `drop` runs whenever the last owner ends; (b) every path of the program that ends the owner's last owner calls it first | **(a)**, the one the compiler can see in one place. File: `WK-7/accept_what_counts_as_proof` (`Ring.drop` calls `clear`) |
