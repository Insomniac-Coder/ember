# Spec gap review, method 3: one question, the smallest change, tested on the compiler

Written 2026-11-02 by a Fable 5.1 agent at the owner's request, from the files only. Nothing in the lead's
brief was taken on trust; section 2 says what the files confirm and what they cannot.

## 1. What it is

Each open question about the specification (73 clusters, 88 member questions, from `questions.md`) is
answered on its own: one agent finds the rule that already decides it and makes the smallest edit that
makes the words say so, or records the choice it had to make by the owner's parameters. A script checks
the edit touches only the rules the question was filed under, quotes real text, and is small. Two
breakers (safety, engineering) attack that edit, not the whole specification. One repair, no growth, then
stop. Every program anyone wrote on the way becomes a test file for the real compiler. The owner reviews
the decisions afterwards; nothing enters the specification before that.

In C++ terms: a bug report names the functions it is about; the fix may touch only those; CI rejects a
pull request that rewrites the module; two reviewers try to break the diff, not the codebase; one
follow-up commit at most; every reviewer's test case lands in the test suite.

## 2. The lead's account, checked against the files

| Claim in the brief | Checked | What the files say |
|---|---|---|
| Method 1 designs rewrote whole areas (~325 rules for 91 questions) | partly | The opening line of every design in `deep/phase2-4-partial.json` lists what it adds and changes: between 2 and about 25 rules each (one FFI design changes 22, one types design 20). The total of 325 I did not count; the shape is confirmed. |
| Method 2's checker found most causes already answered by the specification or a verdict | yes | `deep/v2/pilot-result.json`: 18 root causes; 17 name an existing rule or verdict as the answer (fully or in part); one says "None decides it". Of 176 holes judged, 167 real, 9 refuted; 135 caused by the rule, 24 only by the repair's patch text, 17 by another group. |
| Every redesign rejected twice by the gate; no breaker ran | yes | 3 results, all `unresolved: method`; 19 gate problems: choices not recorded, a clause for one case, untrue self-attestations, a verdict widened, 3 about counting characters. |
| The rules contradicted each other on character counting | yes | Rule G says "never count characters" while the prompt gave a hard cap; the gate rejected designers for `wc -m` and PowerShell length counts. |
| The first run: 127 agents started, 120 finished; the pilot: 18 started, 15 finished; 8+1+1 in the morning sweep | yes | The run journals under the session's `subagents/workflows/`. 155 workflow agents today in all. |
| One safety breaker was cut off by a classifier | yes | `deep/run-notes.md` 14:3x; it also ran a Python one-liner. |
| Holes per breaker did not fall after repair (9 -> 9.5) | not recomputed | Taken from `deep/run-notes.md`; not needed for the design. |
| The correlation 0.51 between rules rewritten and holes | no | The lead's own computation; not needed for the design. |
| The owner kept two breakers per item; 47% of the week used, 13% before today | no (chat, not files) | Kept as given, in the safe direction: both are kept, nothing is dropped. |
| `scope_check.py` rejects 8 of 13 rules on P05's design | no (needs running) | Not needed: the new script reads the edits as data, not by regex over prose. |

One slip of mine: while sampling files I ran `python -c "print(1)"` once by accident (it printed 1 and
nothing else). The brief forbade running Python; I report it here.

## 3. The unit of work and the job

* **Unit: a cluster** (`phase3.json`): one or more member questions on one point, as the owner was
  asked them in `questions.md`. 76 clusters, 3 answered (verdicts 164 to 166), 73 open. The 28
  principle groups of method 1 are kept only for ordering and consistency: the clusters of one group run
  one after another, each seeing the earlier answers of its group; different groups run in parallel.
* **The job: settle this question with the smallest change, from the rules and verdicts that exist.**
  Four outcomes and only these: `already_decided` (the text decides it; quote the lines; no edit),
  `decided` (an existing rule's property reaches the case; extend its words so they say so; no new
  choice), `decided_by_parameters` (a choice was needed; made by the owner's parameters in his order;
  recorded with its options for his review), `owner_question` (his parameters cannot all hold, verdicts
  clash, or a verdict would have to be reversed or widened; options and one recommendation; nobody
  decides it).
* **General, here, means:** the answer comes from the property that already covers the case, written as
  a property, never as a list of cases; three sibling programs (other types, constructs, kinds of
  software) must be decided by the same words with nothing added. This is the owner's "generalized
  solution" without the first run's "rewrite the area". The breakers' `special_case` and `not_general`
  hole kinds enforce it; the script enforces the size.

## 4. The steps

| Step | Who | Model, effort | Reads | Must output | Checked by | Stop rule |
|---|---|---|---|---|---|---|
| 0. Inputs | the lead, `make_inputs.py` | script | phase2/3 json, groups, JOB.md, part files | one plain-text file per cluster (lines <= 900 chars); the `args` JSON: allowed rules, their exact text, cap, edits limit | the printed table (rules whose text was not found) | a missing rule text is fixed by hand before launch |
| A. Answer | 1 agent per cluster | Opus 5.5 (inherited), xhigh | rules file, owner bar, direction, verdicts, the cluster file, the spec parts | `outcome`, `existing_rule`, `edits[]` (rule, kind, before, after, why_general), `answers[]` (one per member, whole program, today/after, the deciding line), `sibling_cases[]` (>= 3), `readability` before/after, `cost`, `verdicts_kept[]`, `decisions[]` (options as code, chosen, parameters in order, `owner_short`), `to_verify[]` | the script: outcome/edits consistent; every edited rule allowed; `before` verbatim in the rule; <= 3 edits; new text <= 1,200 chars; decisions shaped as required; a sibling case count; an answer per member | one resend on a script rejection, then `rejected_by_script`; a stopped agent is `stopped`, never retried |
| B. Break | 2 agents per cluster (safety, engineering), only for `decided` / `decided_by_parameters` | Opus 5.5, xhigh | the same files; the answer (JSON, small) | `checked[]`, `matrix[]` (guarantee, how kept, must-refuse, must-compile), `holes[]` (kind, caused_by `this_edit`/`pre_existing`, severity, whole program, today, with_edit, what goes wrong, rule lines, fix hint), `tests[]`, `broken`, `finished` | the script counts high/medium holes caused by the edit; `pre_existing` holes go to the new-gap list, never to a repair | no holes: `holds` (both finished) or `unverified` (one cut off); holes: step C |
| C. Repair | 1 agent per cluster with holes | Opus 5.5, xhigh | the answer, every hole with its id | `triage[]` (real / refuted / pre_existing, with lines), `causes[]`, then the whole answer again (`edits` replaced as a whole) or `owner_question` | the same script checks as A, plus: new text <= the first answer's new text (no growth) | one resend on a script rejection; a rejected repair ends `unresolved`; `owner_question` ends there |
| D. Re-break | 2 agents per cluster repaired | Opus 5.5, xhigh | the repair with its triage; refuted holes are re-tested first | as B | the script | any high/medium hole caused by the edit: `unresolved` (final; the holes go to the owner); none: `holds` / `unverified` |
| E. Report | the lead, `make_report.py` | script | the run's result | `<batch>-report.md` in the owner's shape; `<batch>-tests/<cluster>/*.em` with `#$` headers | the lead reads it; the owner gets the report | the owner says go on, stop, or change something |
| F. Compiler | the lead, by hand | the built compiler (`target/`, never `%TEMP%`) | the test files | per file: the compiler agrees, disagrees, or does not build that feature yet (`E0900`) | the harness's expectations (`#$ error[E…]`, `#$ stdout`) | a disagreement is reported next to the item; it is evidence for the owner, not a repair |

Agents per cluster: 1 (`already_decided` or `owner_question`), 3 (holds at once), up to 8 (two sends of
the answer, two breakers, two sends of the repair, two re-breakers). Nothing in the script can skip a
step; the only paths out are the stop rules above.

## 5. What each step must not do (the prompts say it; the schemas make it checkable)

* The answer agent: no rule for the area; no edit outside the allowed rules; no clause for the example;
  no type or construct named as a case where a property would do; no choice left open; no verdict
  reversed or widened; no counting; nothing run. Earlier answers of the same group are given to it as a
  small JSON (edits and decisions only) with "stay consistent; if one is wrong, say so, do not undo it".
* The breakers: bounded to the edit. (1) Walk the answer's own programs line by line and confirm or
  refute each stated outcome with lines. (2) Fill a matrix row per guarantee or promise with a
  must-refuse and a must-compile program. (3) Write programs inside the edit's reach: decided differently
  than today, left undecided, or breaking a row. A program that goes wrong the same way without the edit
  is `pre_existing`: a new gap for the list, never a hole of this edit. (4) Stop when every program is
  checked, every row has its two programs, and two further programs in a row found nothing new. The
  safety breaker uses the owner's defensive-validation framing (`SAFETY-VALIDATION-TASK.md`) and says
  that nothing is run; a classifier stop ends that lens as `unverified`, reported, never reworded.
* The repair agent: triage first (default `refuted` when not shown, with the lines), group the real
  holes by cause, replace the edit as a whole; a clause per hole is impossible because the text may not
  grow. A cause that needs the owner ends the cluster as `owner_question`.

Dropped on purpose: the gate agent (a second judge of the same model; the owner: "no guarantee"; its
substance is in the engineering breaker and the script) and the `rules_checked` self-attestation (it
produced "untrue entry" rejections, nothing else; the required fields are the compliance now).

## 6. What the owner gets, per cluster

| Status | Meaning | What he sees |
|---|---|---|
| `already_decided` | the specification already says it | the lines, the program, and the compiler's answer on that program |
| `holds` | the edit survived both lenses (after one repair at most) | the edit as before/after, every decision in his shape (one line in C++ terms, step table, today, options, recommendation), the tests, the compiler's answers |
| `owner_question` | nobody may decide it | the problem, options as code, one recommendation |
| `unresolved` | the repaired edit still has a real hole | the edit, the holes left with their programs, as a question for him |
| `unverified` | a breaker was cut off; no hole known | the edit, marked "not fully checked", never retried |
| `stopped` / `rejected_by_script` / `nothing` | the answer or repair agent was stopped, rejected twice for form, or died | reported as such; re-queued only if he says so |

Plus one list of **new gaps** the breakers found that this edit does not own (pre-existing), for a later
batch. Nothing is written into the specification; the lead applies an edit only after the owner's verdict,
as a new hardening, with the tests moved into the suite.

## 7. Models and effort

All agents inherit Opus 5.5 (no `model` set) at effort `xhigh`. Reason: none of the three jobs is easy
(finding the rule that already decides a case in a 14,000-line specification; breaking an edit with whole
programs; a no-growth repair), and the owner's lean rule sizes effort to the job. No cheaper model is
proposed for any step; the only cheap steps are scripts. Fable needs the owner's approval and is not used.

## 8. The pilot

Three clusters, two groups, chosen so that the method is tested on an easy case, a decision case and the
hardest case, with the earlier runs' results to compare against:

| Cluster | Group | Members | Why |
|---|---|---|---|
| K04 "Can a weakref link to an arena-made object outlive the arena?" | P02 | J5-02 | high, crisp; the finder's recommendation is already "apply the rule the specification has to the one holder it misses": the ideal `decided` case; method 1 and 2 data exist |
| K05 "May a thread_shared value take its memory from an arena?" | P02 | J5-05 | high; the critic says "ask this first" before K34; a real choice (refuse, or allow with a condition): tests `decided_by_parameters` and the in-group consistency with K04 (both touch the same rule) |
| K10 "Where and when does a queued thread_shared drop run?" | P05 | J7-01, J7-02 | the hardest open item (P05 never resolved in method 1; the pilot checker found 5 owner decisions); if the method cannot bound it into `holds` or a clean `owner_question`, better to learn it here |

Agents: 9 at least (three answers, six breakers) and 24 at most; typical 12 to 15. K04 and K05 run one
after the other (same group); K10 in parallel. Wall clock at 2 agents at once: about 2 to 3 hours.

**Pass criterion (recommended; the owner may set another):** the method converges when every cluster ends
in a bounded state (`holds`, `already_decided` or `owner_question`, or `unresolved` with at most 3 holes
left), no repair was rejected for growth, no edit was rejected twice for scope, and the per-agent cost
measured on the meter is at or under the estimate below. One cluster `unresolved` with a clear owner
question is a pass; two or more `unresolved`, or any repair that grew, is a fail and the method changes
before any other batch.

## 9. Cost, as a share of the week, and its basis

* Basis: today's 155 workflow agents plus the lead's own session took the owner's meter from 13% to 47%
  (the owner's figures, via the lead): **at most 0.22% of the week per agent**, the lead's session
  included in that. Those agents each read about 300k tokens of context per turn (one redesign agent's
  transcript: 305k to 314k cached tokens per turn, about 20k output tokens per turn, 1.7 to 2.4 MB of
  transcript). The method-3 agents read a bounded cluster file (one question, its rules, its group) and
  write far less, so **0.1 to 0.2% per agent** is the working estimate.
* Pilot: 9 to 24 agents: **1% to 5% of the week**. The pilot is also the measurement: meter before and
  after, divided by the agent count, gives the real figure for every later estimate.
* Full scale: 70 more clusters at 3 to 8 agents each: 210 to 560 agents, **21% to 112% of the week** at
  the estimate. That does not fit the 53% left this week at the top end. It is not a reason to cut
  breakers or effort; it is a pacing question for the owner (question 3).

## 10. Between phases, and after the per-item work

1. **Before the pilot:** the lead runs `make_inputs.py`, shows the owner this plan with the agent range,
   the cost, and the concurrency question; waits for his answer; launches with his `limit`.
2. **After the pilot:** `make_report.py`; the lead runs every test file on the compiler and writes the
   compiler's answer next to each item; the report goes to the owner in the shape of section 6; the
   lead waits. Nothing else is launched.
3. **Batches:** only if the owner says the pilot passed. Batch size and order are his (question 3); the
   recommended order is `questions.md`'s, with the critic's "ask X first" notes kept (a cluster that
   another cluster's answer depends on runs first, in the same group). After every batch: report, tests on
   the compiler, wait.
4. **Phase 5, the whole-language check** (`deep/phase5-plan.md`, approved by the owner): runs after the
   last batch, with the final edits as its input, and the owner asked for its concurrency first, as he
   said. Its inputs are now small (73 edits of at most 1,200 characters each, about 25k tokens in all,
   against 28 designs of 300k), so the agent count he approved (8 certain, about 18 at most) may be more
   than it needs; question 6.
5. **Then the ODRs, one at a time**, each with the lead's recommendation, in his shape; and the edits the
   owner approves go into the next hardening with their tests.

## 11. Blunder -> how this workflow makes it impossible

| # | Blunder | Made impossible by |
|---|---|---|
| 1 | All 28 groups at once, no pilot | The script runs only the clusters in `args`; the plan fixes the pilot at three; every later batch needs the owner's go after a report; the lead cannot launch 70 clusters without producing the `args` for 70 and stating the cost (21 to 112%). |
| 2 | "General" turned into "rewrite the area" | The job is one question; `edits` may touch only the rules the question was filed under (script, data not regex); at most 3 edits and 1,200 characters of new text (script); `existing_rule` is a required field; a `new_rule` is allowed only with a recorded decision. |
| 3 | Repair = a clause per hole | The repair replaces the edit as a whole and may not have more new text than the first answer (script); holes are grouped by cause first; a cause the owner must decide ends as `owner_question`. |
| 4 | Unreachable stop rule; holes never checked | Only holes caused by the edit (not pre-existing gaps) count; breakers are bounded to the edit's reach with a dry-run stop; the repair agent triages every hole with lines and defaults to refuted; re-breakers re-test refuted holes first. |
| 5 | FIFO limiter ran every repair before any re-break; wrong timing claims | Each cluster flows answer -> break -> repair -> re-break on its own (`pipeline` per group); the limiter only bounds how many agents run at once; no timing claims in this document beyond "about 2 to 3 hours at 2 at once", to be replaced by the pilot's measured time. |
| 6 | Method 2 kept the same job | The job changed: "answer this question with the smallest change" replaces "write the group's rule again". |
| 7 | Contradictory rules; inputs the Read tool cut; agents ran scripts | One rules file, no character cap in any prompt, "the script measures"; inputs with lines <= 900 characters made by `make_inputs.py`; rule G lists the four shell commands allowed and forbids `wc`, `node`, `python`. |
| 8 | A breaker cut off by a classifier | Defensive framing kept; "nothing is run" stated; a stop ends that lens as `unverified`, reported, never reworded; its holes found so far still count. |
| 9 | A step dropped when the owner did not understand it | The steps are in the script; none can be skipped; a change to a step is a question to the owner before launch (section 14), explained in C++ terms first. |
| 10 | Explanations that confused him; 34% of the week with no usable rule | Every decision carries `owner_short` in the shape that works (schema-enforced); the report is generated in that shape; the pilot is sized at 1 to 5% and measures the cost before anything larger. |

## 12. Owner requirement -> where the workflow meets it

| Requirement (his words) | Where |
|---|---|
| "perfectly engineering no compromise, safe, high readability, high performance, goal compliant ... in line with the ODRs I have approved unless you can do something really REALLY better" | Rules file C1 to C6; required fields `readability`, `cost`, `verdicts_kept`; two lenses; a "really better" departure is an `owner_question` with evidence, never applied. |
| "no specific solutions ... generalized solution that meet all the other parameters" | Rules D1, D5; `why_general` per edit; three sibling cases required; `special_case` / `not_general` hole kinds; a clause per hole impossible (no growth). |
| "make sure the language does not lose its ease of use and readability"; "the complexities can be abstracted in the compiler" | `readability` before/after required (the after no longer or harder); parameter order puts "a written word only for meaning" before everything but safety and speed; hole kind `programmer_must_write_what_compiler_could_infer`. |
| "Why is the workflow not giving general solution" | The job is the existing property applied, not a new area; see section 3. |
| "make the decision based on the parameters that I set, then later ... discuss the changes" | `decided_by_parameters`: the parameter order in rule D2; every decision recorded with options as code and `owner_short`; nothing enters the specification; the report lists them for his review. |
| "all agents HAVE to abide by those rules" | One rules file, read first by every agent; the script checks what a script can; the breakers check the rest. |
| Agents read-only, no scripts ("b, keep the rule strict") | Rule G: Read/Grep/Glob, `cat`/`sed -n`/`grep`/`ls` only; nothing to count; inputs readable by the Read tool. |
| "a single repair round if that fails then no more tries" | Step C once; a rejected or failing repair ends the cluster. (A resend for a script formality, scope or quote, is not a repair; question 4 asks him to confirm.) |
| Two breakers per item, safety and engineering | Steps B and D, always both lenses. |
| Cost stated as a share of the week before a run | Section 9, with its basis; the pilot measures. |
| He decides how many agents run at once | `args.limit`, asked before launch (question 1); the script defaults to 2 only if the field is missing, and says so in its first log line. |
| "you and your agents are the same so there is no guarantee" | No same-model gate or vote; every program becomes a compiler test (steps E, F); the compiler's answer is reported next to each item. |
| "YOU ARE FUCKING NOBODY TO DECIDE OUTSIDE THE PARAMETERS I DEFINE" | Nothing he set is dropped or reinterpreted (the kept items are listed in section 2); what he has not decided is in section 14 with options, decided by nobody. |
| Explanations: plain, short, C++ terms, step table, today, options, recommendation | Rule H; the `owner_short` schema; `make_report.py` renders exactly that. |

## 13. What is different from today's failures, in five lines

1. The job is one question, not a group's rule; the edit is bounded by a script before any agent reads it.
2. Breakers test the edit, not the specification, and only holes the edit causes count.
3. The repair cannot grow, so it cannot patch; what it cannot answer goes to the owner as a question.
4. No second judge of the same model; the compiler judges, through test files written by every agent.
5. Three clusters first, 1 to 5% of the week, measured; nothing larger without his go.

## 14. Files and how the lead runs it

```
deep/v3/RULES-FOR-EVERY-AGENT.md   every agent reads it first
deep/v3/WORKFLOW.md                this file
deep/v3/workflow.js                the Workflow tool script (pilot and batches alike; the clusters come from args)
deep/v3/make_inputs.py             python deep/v3/make_inputs.py K04 K05 K10  -> deep/v3/inputs/<cid>.md and the ARGS line
deep/v3/make_report.py             python deep/v3/make_report.py deep/v3/results/pilot.json -> report + test files
```

1. `python deep/v3/make_inputs.py K04 K05 K10` (from the repository, with `-I` if it is run on anything
   downloaded; here it is not). Read the printed table; if a rule's text was not found, fix the input by
   hand before launching. Paste the `ARGS:` object as the Workflow tool's `args`, with `limit` set to the
   owner's number.
2. Launch `deep/v3/workflow.js` with that `args`. Save the returned value as `deep/v3/results/pilot.json`.
3. `python deep/v3/make_report.py deep/v3/results/pilot.json`.
4. Run each `.em` under `deep/v3/results/pilot-tests/` on the built compiler from the repository's
   `target/` (never a `%TEMP%` copy: Defender flags it), and write the compiler's answer next to the item
   in the report: agrees, disagrees (with the diagnostic), or `E0900` (not built yet). A file may be
   copied into `tests/compile-fail` or `tests/run-pass` for one harness run and removed again; none stays
   there before the owner's verdict.
5. Give the owner the report; wait.

## 15. Questions for the owner (decided by nobody else)

1. **How many agents at once?** (a) 2, (b) 3, (c) more. The lean rule says 2 to 3; the pilot has at most
   two clusters running at a time anyway (K04 then K05, K10 beside them), so 2 loses almost nothing.
   Recommendation: (a) 2.
2. **The pilot's three clusters: K04, K05, K10 as in section 8?** (a) yes, (b) swap K10 for an easier
   high one (K12, which function a std call keeps), (c) his own three. Recommendation: (a), because a
   pilot that avoids the hard case proves nothing.
3. **Pace after the pilot:** 70 clusters are 21% to 112% of a week at the estimate. (a) batches of about
   10 clusters, one per report, as the meter allows, over two to three weeks; (b) one batch of everything
   when a new week starts; (c) stop after the pilot and decide then. Recommendation: (c) now, then (a).
4. **One resend for a script rejection (scope, a misquoted rule, size) before the cluster is given up:**
   is that within "a single repair round"? (a) yes, a formality is not a repair; (b) no, one send only.
   Recommendation: (a); it costs at most one agent and saves a cluster lost to a typo.
5. **Where the proposed tests live until his verdict:** (a) `deep/v3/results/<batch>-tests/` only, run by
   hand; (b) a new `tests/proposed/` folder the harness does not scan. Recommendation: (a), nothing new
   in `tests/` before a verdict.
6. **Phase 5's agent count** was approved for 28 large designs; the inputs will be 73 small edits.
   (a) keep the approved count (8 certain, about 18 at most); (b) let the lead propose a smaller count
   with the overlap map when the time comes. Recommendation: (b), asked then, as he said.
7. **Should the review agents skip the repository's start-up reading** (`CLAUDE.md` sends every agent to
   `docs/AUTOPILOT.md` and the handoff's start-here section, meant for compiler work)? (a) skip it for
   read-only review agents (saves reading per agent), (b) keep it. Recommendation: (a), stated in the
   prompt; the files they must read are named there.
8. **The effort of the breakers:** (a) xhigh as planned, (b) high, to cut cost after the pilot's
   measurement. Recommendation: (a) for the pilot; revisit only with the measured figure.
