# Deep review phases 2-4: notes during the run (wf_28e6a473-206, task wx8qfqlbw)

Owner, 2026-11-02 14:35: "Let it run but note things down and the fact that things are breaking again
means there's something clearly wrong".

## Events

* 08:21 launched (28 groups, Opus 5.5 xhigh, 7 at once). All 28 designs finished; all 56 first breakers
  finished and every one broke its design (518 high/medium holes; 4 to 16 per breaker, median 9).
* The limiter is first in, first out: all 28 first repairs ran before any re-breaker (my 13:00 estimate
  of the first re-break results was wrong; they came ~14:15).
* **14:3x, classifier stop:** `rebreak1-safety:P06` (agent a4ed23a338553a6d2) was cut off by a safety
  classifier partway through. It returned `finished: false`, empty matrix and holes, and said so in its
  notes ("broken=false here means nothing"); it also ran one read-only Python one-liner against the
  no-scripts instruction. Owner told (rule: stop and report, no reworded retry); owner: let it run. P06's
  safety lens gets a fresh agent only through the pipeline's own round 2 (same prompt as every
  re-breaker); if that is stopped too, P06 ends `unverified`, reported, not retried.

## Round 1 so far (14:32): every repaired group still breaks

| Group | Safety high/medium (first -> after repair 1) | Engineering high/medium |
|---|---|---|
| P02 memory managers | 2/6 -> 4/4 | 2/10 -> 0/12 |
| P03 one value, one place | 5/7 -> 6/5 | 4/4 -> 1/9 |
| P05 where `drop` runs | 3/7 -> 2/7 | 0/6 -> 0/7 |

## What looks wrong (measured; to settle before any later batch)

1. **Repairs patch, they do not generalise.** One repair nearly doubles a design: P02 41.6k -> 81.2k
   characters (+95%), P03 +57%, P05 +65%. The repair prompt hands the designer ~20 holes and asks it to
   close each; it adds a clause per hole. More text, more surface: 12 of the 57 round-1 serious holes
   name text the repair changed or added. This is the "specific fix" the owner's bar forbids.
2. **The hole count does not move.** A breaker reports a median of 9 serious holes on the first design
   and 9.5 on the repaired one (min 7, max 12). Either the repairs gain nothing, or the count measures
   the breaker's effort more than the design: "find every program" over a 14,000-line spec has no end,
   and "zero high or medium" may be a stop rule no design can meet.
3. **One cause shows up across groups:** a rule that needs a fact about the caller, where one compiled
   body serves many callers (a method in a table, a `dyn` call, a function compiled once per kind of
   memory manager, not per instance). P02 round 0 (`Enemy.listen` in a method table), P03 round 0
   (`Shop.show` compiled once for every `Shop`), P02 round 1 ("compiled once per kind, not per
   instance"), P05 round 1 (`subscribe` compiled with `events`, which knows no implementation). A crude
   keyword count: 6% of round-0 serious holes, 19% of round 1's. If it holds, this is one language
   decision (how a caller's fact crosses a compiled boundary: in the type, as Rust's lifetimes do;
   checked while running; or refused), not 28 patches, and it is the owner's to make.

## Stopped (owner, 14:38: "Yes stop now")

The owner asked why the workflow does not give general solutions ("that is EXACTLY what I asked for").
The lead's answer: the repair step's shape. It hands the designer ~20 separate broken programs to close,
so it adds a clause per hole; nothing rejects a repair that grew or added a case clause before the
breakers spend credits; and the designer has no way to hand a decision that is the owner's (a word, a
run-time check, a refusal) back as an ODR, so it patches. Round 2 used the same prompt, so the run was
stopped (option a of three; recommended).

Saved: `deep/phase2-4-partial.json` (stop_and_tally.py): 120 finished agents: 28 designs, 56 first
breakers, 28 first repairs, 8 round-1 re-breakers (P02, P03, P05, P06; P06 safety not finished),
584 high/medium holes. Lost: the 7 agents in flight (re-breakers of P01, P04, P07, P08 safety).

Next: a new method, as a plan for the owner before anything is spawned: holes grouped by cause first;
one rule per cause, no longer than the first design; a gate that rejects a grown or case-specific
repair before any breaker; a designer that needs the owner's decision writes an ODR instead of a patch;
each hole checked as real and as caused by this rule before it counts.

## Method 2 pilot (wf_015776c2-f98)

* 15:10 launched (owner: "Go ahead all at once"); 15:25 stopped and resumed after the owner's change
  ("I want the workflow to make the decision based on the parameters that I set ..."): rules D4/D5
  rewritten, designs carry `decisions_taken`; the first check came from the cache, the three running
  redesigns restarted (none had finished).
* 16:05 rule break found: the gates of P02 and P03 and two redesigns ran `node -e` (reading JSON; one
  `wc -m`); no file was written. Cause: the inputs' long lines (up to 17,687 characters) are cut by the
  Read tool. Owner chose "b, keep the rule strict": rule G now names the allowed reads and forbids any
  script; plain-text copies in `deep/v2/text/` (make_text.py, lines <= 900). Before every later batch:
  `python deep/v2/make_inputs.py <groups>` then `python deep/v2/make_text.py <groups>`.

## Pilot result (17:00): all three "unresolved: method" (deep/v2/pilot-result.json)

15 agents (3 checks, 6 redesigns, 6 gates), ~80 minutes; no breaker ran, because no design passed the
gate. Each redesign was rejected twice (P02 4 then 5 problems, P03 4 then 5, P05 3 then 2). Of the 19
problems: about half a choice the design made without recording it, or left open (D4); about a third an
exception or a clause for one case with no reason given (C1, principle 6); several untrue entries of
rules_checked, mostly following from those; 2 words the compiler could work out (C3); 1 verdict widened
(114, D5); 3 about counting characters, a conflict of the lead's own making (a hard character cap in the
prompt, "never count characters" in rule G). Rule G after the strict change: kept (the only shell
commands since were `cat` and `wc -c` on files). The check step worked: 26 to 34 real holes per group
from 5 to 8 causes. The gate worked: it stopped flawed designs before any breaker spent credits. The
designers did not converge in two tries. Rule G's counting sentence reworded (the workflow measures).

## Why the workflow failed (owner: "find the problem with the workflow why is it failing"; lead, solo)

The job was wrong. The design prompt asked for "ONE general rule that establishes the group's principle:
it settles every member question and every case in other_places, and the cases nobody has asked about".
The agents took that as "rewrite the area": 28 first designs rewrote about 325 existing spec rules to
answer 91 member questions (bullet and "Adds/Changes" count of each spec_text; P05: 2 questions, 13
rules; P13: 2 questions, 16 rules). Rewritten rules lose what made them sound, so the breakers found holes
in the new text. Evidence: the method-2 checker (independent) found that 12 of the pilot's 18 root
causes are already answered by the specification or a verdict as it stands (P02 A, C, E, F, G, H; P03
C1, C3, C4, C5; P05 C1, C5): the design changed or ignored a working rule. Designs that rewrote 8 rules
or fewer drew 13.0 serious holes on average, those that rewrote 15 or more 21.9 (correlation 0.51 over
28). Method 2 kept the same job ("write the group's rule again"), so it failed the same way, at the gate.
Not the whole story: even small rewrites drew ~13 holes. The fix: the job is "settle this question with
the smallest change, using the rules and verdicts that exist; change nothing else"; general means the
answer comes from the rule that already covers the case.

## Method 3 (Fable 5.1 design, deep/v3/): the owner's answers to its questions (18:35)

1 "all at once"; 2 "go for hard ones" (asked: keep K04, K05, K10, or swap the easy K04 for a hard one);
3 "will decide later"; 4 "a formality is not a repar"; 5 "deep/v3/results"; 6 "ask later after this is
done"; 7 "no clue what you mean" (explained again); 8 "highest". Nothing launched yet.

## Method 3: trial result, compiler check, batch 1 (2026-11-02 evening)

* Trial (wf_0bc36743-b6c, 17 agents, meter 47% -> 51%): K04 holds, K05 holds (1 decision by parameters),
  K10 unresolved (9 -> 5 holes). deep/v3/results/pilot-report.md.
* Today's compiler on the 210 tests: 189 "not built yet" (thread_shared, weakref, capture words, ...);
  of the 21 left, K10's tests were often a description in place of the program or the exact output.
  Owner agreed to the lead's suggestion: a script check in make_report.py rejects such tests (34 of the
  trial's rejected, listed in the report); go on with the batches; keep the tests for when the compiler
  builds those features. Owner: "spawn it with no agent restriction as many concurrent as possible".
* make_inputs.py bug fixed: phase2.json dropped the `rules` of some questions (J8-08, J8-09, J6-16, J8-04
  and others); the input maker now falls back to the finding's own rules in phase1.json (the trial's
  three are unchanged).
* Batch 1 (the 11 high: K06 K07 K08 K09 K11 K12 K18 K19 K20 K21 K31) launched: workflow_batch1.js, run
  wf_ec3baefe-0c1, limit 200.

## Batch 1 stopped, the analysis, and the owner's decisions (2026-11-02 ~22:45)

Stopped by the owner after 6 of 11 ended `unresolved` (K06 4->8, K07 1->1, K12 5->3, K20 4->3, K21 8->8,
K31 6->6 serious holes, first answer -> after the repair); 47 agents had finished; K08 and K18 were in
their re-break (their repairs had finished), K09, K11, K19 not started. Results kept in deep/v3/results/batch1-partial.json.

The lead's analysis (from the saved results, no agents):
* Ruled out: the allowed rules blocking the fix (1 of the 34 holes left needs a rule outside its
  question's list; a first count that said 33 was wrong: breakers cite rules for context); fixes growing
  (400-900 characters, repairs mostly shorter; K31 about 2,000).
* Cause 1, the pass mark: "no high or medium hole caused by the edit" while each new pair of breakers
  finds new cases (23 of the 34 holes left were never raised in round 1; 25 of the 34 are medium; K07
  failed on one medium hole).
* Cause 2, no check of the last round's holes: the repair triages round 1 and judged about a third not
  real or pre-existing (26 of 76); the re-breakers' holes count unchecked.
* Cause 3, whole missing features, not gaps: e.g. K21 (how C++ overloads import), 5 of its 8 holes left
  are cases the fix does not decide; the trial's two passes (K04, K05) were narrow questions.

**Owner's decisions: "both / just note it down"** (recorded, not built yet):
* (a) After the last breakers, one checker per question confirms each remaining hole is real and caused
  by the fix (the repair's own triage, applied to the final round); only confirmed holes count; the pass
  mark is unchanged. One more agent per question.
* (b) A question that is a whole missing feature (K21-like) goes to the owner as a design question, not
  through the small-fix workflow.

## Test run 2 (owner: "do another test run with the changes"; "go ahead after that hibernate the laptop")

Built (a) and (b): rules D2 has a fifth outcome `whole_feature` (a design question for the owner, no edit,
at least three undecided cases, options, one recommendation; the script checks it); D5a and a `check`
step after the re-break: one checker gives every last-round hole a verdict (real, refuted, pre_existing);
only confirmed real holes count; a missing verdict ends the question `unverified`. make_report.py shows
the check's counts. Fake-agent run passed every path. Test run 2: K07, K20, K21 (batch 1's results to
compare), workflow_test2.js, run wf_c2916bd3-4a0, all at once. After it: save, report, hibernate.

### Test run 2 result (2026-11-03 ~01:40; 21 agents; deep/v3/results/test2.json, test2-report.md)

All three `unresolved`, 7 agents each:
* K07: 2 serious holes -> 5 after the repair; the final check confirmed all 5 real (medium). Batch 1: 1 -> 1.
* K20: 4 -> 3; the check confirmed all 3 real (medium). Batch 1: 4 -> 3.
* K21: the answer agent did NOT take `whole_feature`; it chose `decided_by_parameters` with 4 decisions;
  8 -> 10; the check: 9 real (2 high, 7 medium), 1 pre-existing. Batch 1: 8 -> 8.
What it shows: (a) the final checker confirmed 17 of 18 last-round holes as real, so unchecked holes were
not what failed the fixes; the fixes really leave real holes, mostly medium. (b) the whole-feature path
did not trigger for K21, the question the owner named as the example.
Lead's bug, found and fixed: the description check rejected tests that begin with the C or C++ header
they import (132 of 135 in test 2); it now looks for Ember code anywhere and still rejects programs that
only point elsewhere. Reports regenerated: test 2 241 tests kept, 13 rejected; the trial 177 kept, 33.
Not done: the meter reading (ask the owner). Laptop hibernated afterwards (owner's instruction).

## Method 4 designed by the lead (2026-11-03; owner: "one last chance ... design a workflow yourself")

deep/v4/: TEAM-WORKFLOW.md (the design), RULES-FOR-THE-TEAM.md, workflow.js, workflow_pilot.js (K07, K20,
K10), make_report.py. A team per question: designer, safety adversary, language engineer, spec
integrator, arbiter; the fix goes to its cause anywhere, with why_here and an impact report; up to three
review rounds, stopping when nothing blocks or no progress; only S (safety hole), U (question undecided),
X (new contradiction) block, confirmed by the arbiter; the rest is weighed and shown to the owner.
Fake-agent run passed (accepted after a fix, accepted with weighed objections, rejected on no progress).
Not launched: shown to the owner first.

## Method 4 pilot result and the bug fixes (2026-11-03 morning)

Pilot (wf_13565f06-13c, 29 agents in all; the last recompute cost nothing): all three `rejected`, each
down to one remaining cause confirmed by the arbiter. K07: 13 objections fixed in the debate, 1 left
(X: `@ffi(std_function)` read two ways; the designer left that part as the owner's decision 4).
K20: 11 fixed, 1 cause left (S, high: a library's error type holding an object of an open class; a
subclass in another package holding a one-thread owner crosses threads inside AnyError). K10: 14 fixed,
from 9 blocking to 1 (X: a wait inside a lock in a scope, meant to be refused, can hang).
Owner then: "MAKE IT CHEAPER AND RUN THE TRAIL" (done: two reviewers from round 2, the arbiter only for
a disputed blocking objection, round 1 replayed) and "FIX THE BUG AND DONT RUN YET".
The lead's bugs and their fixes (workflow.js, make_report.py; nothing run):
* Exact-id matching, in four places (member answers, responses to objections, reviewers' statuses,
  the arbiter's rulings): agents write ids with words around them ("S1-7: ..."). It wasted 3 revision
  resends and 2 design resends and marked the first results "unverified". Now one function, `mentions`,
  finds an id as a whole token anywhere ("S1-1" never inside "S1-10"); where an entry stands for one
  objection, the id it names first (`firstId`). Tested on the agents' own formats.
* 53 run tests whose exact output sat inside a sentence (`prints "3 1"`) were rejected; make_report now
  takes the quoted output after "prints" and keeps the agent's words in a note: 53 of 53 kept, 186 tests.
* New runs start with the two-reviewer team (FIRST_ROUND); workflow_pilot.js stays as the record of
  what ran (it keeps the old exact matching for its replay).
* Owner: "make ID as something that can be corrected or fixable not a rejection criteria, emphasis on the
  solution". Done: formProblems() sends a design back only for its substance (no edit for a decided
  outcome, no why_here, a decision without its choice, fewer than three sibling cases); formNotes()
  corrects or notes ids and form (one answer, one member: it gets the member's id; an unanswered objection
  stays open and its reviewer judges it on the revised change; tests are salvaged or listed by the report);
  the arbiter's rulings take the id they name first, and rulings naming none are paired in order; an
  unjudged blocking objection is "open", never a silent pass. The report lists every correction. Dry run
  with agents that get ids wrong: nothing rejected, every path right. Nothing run.
