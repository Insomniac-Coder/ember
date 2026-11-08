# The 0.9.15 gap review: everything found, done and learned (2026-11-02 to 2026-11-06)

Written for the next agent by the lead of the local session (the owner: "prepare detailed notes of all your
finding ... including the new Hardened version you made and the findings of the bugs. The other agent will
read all of that"). Read this first, then `deep/run-notes.md` (the day-by-day record) and
`deep/check-0.9.20-H2.md` (how this work stands against the cloud session's 0.9.20).

## 1. What is on this branch that the 0.9.20 branch does not have

`origin/spec-0.9.20-hardening` (the cloud session, 2026-11-03) was built from `5001ef9e` (0.9.15_Hardened_10).
It does not have:

1. **0.9.15_Hardened_11** (commit `4de795c4`): the owner's verdicts 161 to 163 written into the spec
   (`docs/spec-source/Ember_v0.9.15_Hardened_11.md`, parts frozen in `tasks/spec-0.9.9/parts-0.9.15-h11/`,
   `development-target.json` re-pinned to SHA `516ED28D5E1244861E7620F4ABA31D649BAA07A2115ECF0ED33D8A773FA39E9C`,
   §H.51 in `tools/appx_h.py`; E3021's help in `tools/p17a_0915.py`).
2. **Verdicts 164 to 166** in `tasks/96/redesign/JOB.md` (the gap sweep's first three questions, 2026-11-02);
   its JOB.md ends at 160.
3. **This folder**: the gap sweep (76 open questions), four methods of agent review, their results, the
   tests they wrote, and the tools.
4. **Three open compiler defects** (D-966, D-967, D-968 in `docs/DEFECTS.md`), D-966 with its fix ready.

### The owner's verdicts 161 to 166 (verbatim in JOB.md; short here)

| # | Verdict | In 0.9.20_H2? |
|---|---|---|
| 161 | `owns` at a call binds as `-` does; an argument of more than one name, number or call takes brackets: `tx.send(owns (i * i))`; `tx.send(owns i * i)` is `(owns i) * i`, refused | yes (`[GRM-47]`, code `E0119`; H11 used `E3028`) |
| 162 | `runtime` goes before a class type too: `fn pick(a: runtime A):` | yes (`[LT-48]`) |
| 163 | `runtime` goes on a variable too, right after the colon: `renderer: runtime Box[dyn Renderer] = ...` | yes (`[LT-48]`, a local declared with its type) |
| 164 | a `sharedref mut` link never goes to another thread: a type holding one is neither `Send` nor `ThreadShared` | yes (`[THR-8]`, `[THR-9]`) |
| 165 | `thread_shared` is refused **where the value is made** when something in it is not `ThreadShared` (`stats = thread_shared FrameStats()` with `draws: RwLock[Cell[int]]` is `E7015` at that line) | **no**: 0.9.20's `[THR-1]` asks only `Send` at the making line |
| 166 | a scoped task neither makes nor ends a `shares` owner, nor ends an object `weakref` links point at (`E7015` at the spawn) | yes ("A task touches no one-thread count") |

0.9.20 also, by its own record, reverses the written-manifest half of verdict 135 (its N83) and touches
verdicts 101 and 114 (its N87). Those are the owner's to accept.

## 2. The gap sweep (worked)

| Phase | What | Output |
|---|---|---|
| 1 | 8 finders over Hardened_11, one part group each (`brief.md`) | `phase1.json`: 137 findings (rules, quote, program, severity) |
| 2 | 8 checkers: each finding refuted / already decided / confirmed, and drafted as an owner question (`phase2-brief.md`) | `phase2.json` |
| 3 | 1 consistency critic over all drafts | `phase3.json`: **76 clusters** (K01 to K76; 17 high, 42 medium, 17 low), conflicts, verdict clashes |
| - | ranked for the owner | `questions.md`; K01-K03 answered (verdicts 164-166) |

Note: `phase2.json` dropped the `rules` field of some items (J8-08, J8-09, J6-16, J8-04, ...); the finding's
own rules are in `phase1.json` (the v3 input maker falls back to them).

## 3. Answering the 76 with agents: four methods, what each showed

Credits: the owner's weekly meter went 13% -> 47% on 2026-11-02 (the sweep and method 1, about 155
agents), 47% -> 51% for method 3's trial (17 agents plus the Fable design agent); later runs not measured.
About 0.22% of a week per Opus 5.5 agent at effort xhigh, as an upper bound.

| Method | Shape | Runs | Result |
|---|---|---|---|
| 1 | 28 principle groups (`deep/phase1-groups.json`): a designer writes ONE rule per group; 2 breakers (safety, engineering); up to 2 repairs | `wf_28e6a473-206`, 127 agents started, 120 finished; stopped by the owner | every one of 28 designs broke (584 high/medium holes, `deep/phase2-4-partial.json`); repairs grew 57-95% and broke again (median 9 -> 9.5 holes per breaker) |
| 2 | per group: a checker sorts the holes into real ones and root causes, a redesigner writes one rule per cause under a length cap, a gate reviews before any breaker | `wf_015776c2-f98`, pilot P02 P03 P05, 15 finished | the checker worked (26-34 real holes per group from 5-8 causes); every redesign rejected twice by the gate; no breaker ran |
| 3 | per question (Fable 5.1 design, `deep/v3/WORKFLOW.md`): the smallest edit within the rules the question was filed under; a script checks scope, quotes and size; 2 breakers; 1 repair; later a final checker and a `whole_feature` outcome | trial `wf_0bc36743-b6c` (17), batch 1 `wf_ec3baefe-0c1` (48 finished, stopped), test 2 `wf_c2916bd3-4a0` (21) | trial: K04 and K05 hold, K10 unresolved; batch 1: 6 of 6 finished unresolved; test 2: K07, K20, K21 unresolved, the final checker confirmed 17 of 18 last holes real, K21 not routed as a whole feature |
| 4 | a team per question (the lead's design, `deep/v4/TEAM-WORKFLOW.md`): designer, safety adversary, engineer, spec integrator, arbiter; the fix goes to its cause anywhere, with why_here and an impact report; up to 3 rounds; only a safety hole (S), the question undecided (U) or a new contradiction (X) blocks | pilot `wf_13565f06-13c`, K07 K20 K10, 29 agents | all three **rejected, each down to one remaining cause**; 38 objections fixed in the debate |

### Why methods 1 to 3 failed (measured in `deep/run-notes.md`)

* **Method 1 and 2: the job was wrong.** "One general rule for the group, settling every member and the
  cases nobody asked about" made agents rewrite whole areas: about 325 existing rules for 91 member
  questions. The method-2 checker found that 12 of its 18 root causes were already answered by the spec
  or a verdict as it stood: the rewrites broke working rules. Designs that rewrote 15 or more rules drew
  21.9 serious holes on average, those with 8 or fewer 13.0.
* **Method 3: the fix could not go where the cause was.** The scope check kept each edit inside the
  rules the question was filed under, which is where the problem showed, not where it is caused (K07:
  the danger is safe code writing a raw pointer into a struct, decided in §IV.5, not in the FFI rules;
  K20: the fix is in how generic types and interfaces carry `Send`). The pass mark "no high or medium
  hole" also failed fixes on medium engineering points (25 of 34 holes left after repair were medium).
* **Method 4 fixed both** (the fix goes to its cause; only S, U, X block) and was the first to converge:
  K10 went from 9 blocking objections to 1.

### The concrete spec results worth keeping

Full text, every edit before/after, decisions with options, objections and rulings: the report files.

| Question | Status | Where | What is left |
|---|---|---|---|
| K04 weakref link outliving an arena | **holds** (method 3) | `deep/v3/results/pilot-report.md` | nothing: LT-4 "or keeps its block (a weak count keeps the block after the value ends)", ALC-5 "or its block is kept after it ends"; 0.9.20 fixed it too (`[ALC-5a]`) |
| K05 `thread_shared` from an arena | **holds** (method 3), one decision by the parameters: no `using` names the block of a `thread_shared` header (E2114) | same | 0.9.20 decided it differently (`thread_shared T using A` refused unless `A` is `ThreadShared`): the owner chooses |
| K07 raw pointer inside a struct to a safe C call | rejected (method 4) | `deep/v4/results/pilot-report.md` | the fix (a limit on raw pointers in `[FFI-2]`, the safe-to-call fact, for every spelling that asserts it) holds on safety; one X left: `@ffi(std_function)` read two ways (the designer left it as decision 4 for the owner). 0.9.20 does not fix K07 |
| K20 `AnyError` across threads | rejected (method 4) | same | decided `Error: needs (Debug, Display, Send, ThreadShared)`; one S left (two reviewers): a library error type holding an object of an open class, judged `Send` in the library's build, while another package derives a subclass holding a `shares` owner, which then crosses threads inside an `AnyError`. This is separate compilation: 0.9.20's `[BLD-16]` (no whole-program fact in a package's code) may be the tool. 0.9.20 does not fix K20 |
| K10 where a queued `thread_shared` drop runs | rejected (method 4) | same | from 9 blocking to 1 X: a wait inside a lock in a `with scope` block, meant to be refused (verdict 43, THR-19), can hang when a task makes a value another thread lets go of. 0.9.20 changed RC-4 point 3 but no answer to this was found |
| K21 C++ overloads | unresolved twice (method 3) | `deep/v3/results/test2-report.md` | a whole missing feature; 0.9.20 touched none of its rules |
| K06, K12, K31, K08, K18 | unresolved or stopped (method 3, batch 1) | `deep/v3/results/batch1-partial.json` | answers, holes and repairs saved |

**Gaps that exist without any fix**, found by reviewers on the way: 10 (trial), 21 (test 2), 16 (method 4
pilot), in each result's `new_gaps`. They are new entries for the list, not yet de-duplicated.

**Tests**: every reviewer's must-refuse / must-compile / must-run program is kept in the results; the
report makers write them as `.em` files with `#$` headers (`python -I -X utf8 deep/v4/make_report.py
deep/v4/results/pilot.json` -> `pilot-tests/`, 186 tests). On today's compiler 189 of the trial's 210
stopped at E0900 (`thread_shared`, `weakref`, capture words not built): they wait for the compiler.

## 4. Bugs found

### In the compiler (docs/DEFECTS.md, all open)

* **D-966** `owns v as u8` reads as `owns (v as u8)` (the parser takes the value at `BP_CAST`); verdict 161 /
  `[GRM-15]` say `(owns v) as u8`, `E3028`. **Fix ready, not applied**: `pending/D-966-owns-binds-as-minus.py`
  (two `BP_CAST` -> `BP_UNARY + 1` in `compiler/ember_parser/src/body.rs`, the test
  `tests/conformance/GRM-15/reject_owns_binds_as_minus_before_as.em`, the DEFECTS row to fixed). Then build
  and run the suites (`cargo test --workspace --profile ci`; no edits while it runs).
* **D-967** a `StrView` parameter stays `E3021` under `runtime` (`fn title(default: StrView) -> StrView` with
  a `String` argument).
* **D-968** the `runtime` stop message says "list" for text.

### In the review tooling (all fixed in `deep/v4/workflow.js` and the report makers)

* **Ids matched exactly**, in four places (member answers, responses to objections, reviewers' statuses, the
  arbiter's rulings). Agents write ids with words around them ("S1-7: ..."), so good work was thrown away:
  5 wasted resends in the method-4 pilot and a first result of "unverified" for all three. Now one
  function, `mentions`, finds an id as a whole token anywhere ("S1-1" never inside "S1-10"); where an entry
  stands for one objection, the id it names first (`firstId`); a ruling that names none is paired in order.
  **The owner's rule: an id or a form problem is corrected and noted, never a reason to reject** ("make ID
  as something that can be corrected or fixable not a rejection criteria, emphasis on the solution");
  only the substance of a change sends it back (`formProblems` vs `formNotes`).
* **Tests as descriptions**: agents wrote "J7-01's program, verbatim" or "No freeze..." where code or exact
  output belongs. The report makers reject a program that is not code; an output written inside a
  sentence (`prints "3 1"`) is salvaged from its quotes, with the agent's words kept in a note.
* **C/C++ headers**: a test may begin with the header it imports (`// renderer.hpp`); an earlier check
  rejected 132 such tests by mistake.
* **Encoding**: `python -I` ignores `PYTHONIOENCODING`; Windows wrote `…` and `§` in cp1252 and broke the
  quote check. Use `python -I -X utf8`.
* **Contradictory rules**: a hard character cap in the prompt and "never count characters" in the rules.
* **Long input lines** (up to 17,687 characters) cut by the Read tool made agents run `node -e`; the
  owner chose to keep the no-scripts rule strict: plain-text inputs with short lines (`v2/make_text.py`,
  `v3/make_inputs.py`).

## 5. The owner's rules for this work (from his words; they bind any next run)

* Safety is the only part never compromised; everything else is weighed and shown to him (method 4, C).
* The fix goes where the problem is caused, with its reasoning and its impact; the smallest change there.
* The workflow decides choices by his parameters and records each for his later review; nothing enters
  the spec before he has reviewed it; a departure from a verdict is always listed for him.
* Never decide outside the parameters he sets: never drop, add or reinterpret a step he approved; ask.
* Pilot 2-3 items with the exact prompts before any full run; stop at the first sign a method does not
  converge; state the cost as a share of his week and wait for his yes; he sets how many run at once.
* Agents read only; no scripts (`node`, `python`, `wc`) ever; plain-text inputs.
* Explaining to him: plain words, his world (C++ game engines), one concrete program, a step table, one
  line per option, the recommendation; never pile on explanations; use the names of the thing he saw.

## 6. Where everything is

```
tasks/96/redesign/spec-gaps-0.9.15/
  FINDINGS.md              this file
  brief.md, phase2-brief.md, phase1.json, phase2.json, phase3.json, questions.md     the sweep
  pending/D-966-owns-binds-as-minus.py                                             the ready compiler fix
  deep/run-notes.md        the day-by-day record and every diagnosis, with numbers
  deep/check-0.9.20-H2.md  this work against 0.9.20_Hardened_2
  deep/owner-bar.md, RULES-FOR-EVERY-AGENT.md   method 1-2 rules;  deep/phase1-groups.json  the 28 groups
  deep/phase2-4-partial.json, stop_and_tally.py  method 1's results (584 holes)
  deep/v2/                 method 2 (workflow.js, pilot-result.json, make_inputs.py, make_text.py, scope_check.py)
  deep/v3/                 method 3 (WORKFLOW.md, RULES-FOR-EVERY-AGENT.md, workflow*.js, make_inputs.py,
                           make_report.py, run_tests_today.py, results/: pilot, batch1-partial, test2 + reports)
  deep/v4/                 method 4 (TEAM-WORKFLOW.md, RULES-FOR-THE-TEAM.md, workflow.js, workflow_pilot.js,
                           make_report.py, results/pilot.json + pilot-report.md)
```

Not committed (regenerated by one command each): `deep/v2/text/` (`python -I -X utf8 deep/v2/make_text.py
<groups>`), the `*-tests/` folders (`make_report.py` on the result), `__pycache__`.

## 7. What is next (the owner: the full check "next week")

1. Read each of the 76 clusters against 0.9.20_Hardened_2: answered, not answered, or answered
   differently (`deep/check-0.9.20-H2.md` has the first pass: 62 had a rule changed, 13 none).
2. Bring verdict 165 into 0.9.20; settle K07, K20, K10 (method 4's last causes above), K21 (a design
   question for the owner).
3. Decide how the two lines come together: 0.9.20 lacks H11's commit, JOB.md 161-166 and this folder;
   H11 lacks 0.9.20's 209 rule changes. The owner decides; verdicts are never overridden by a merge.
4. Apply D-966 (pending script), fix D-967 and D-968, with the suites.
5. If agents are used again: method 4 with its fixes (`deep/v4/workflow.js`; two reviewers; the arbiter only
   for a disputed blocking objection), a pilot first, the cost stated, the owner's yes.
