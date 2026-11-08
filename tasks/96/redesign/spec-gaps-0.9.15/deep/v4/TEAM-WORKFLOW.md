# Method 4: a team per question (the lead's design; not launched)

The owner, 2026-11-03: "design a workflow yourself which you think will meet the goals the best way
possible ... safety remains the only no compromise part and the complexities should mostly be abstracted
in compiler ... sometimes the fix is somewhere else and not where it was reported in that case let the
agents use the best judgement but let them also provide the reasoning and impact of the change. Design a
team that works towards a common goal and something that beats each other down to death."

## The team (one per question; every member Opus 5.5 at extra high effort; all read-only)

| Role | Who they are | Their job | Can block? |
|---|---|---|---|
| Designer | the question's lead | finds the cause wherever it is, writes the smallest change there, with its impact report; answers every objection | - |
| Safety adversary | found use-after-free and races in C++ engines and unsafe Rust | breaks the change's safety: freed memory, races, double ends, undefined behaviour, unclear crashes, a guarantee taken away | yes (S) |
| Language engineer | built a borrow checker, kept a big codebase readable | correct programs refused, readability, cost over C, words the compiler could work out, special cases, verdict clashes | no: weighed |
| Spec integrator | kept a standard of a thousand rules consistent | every rule that cites or is cited by a changed rule; the quoted words are real; the question is decided (U); no two rules now disagree (X); the impact report is true | yes (U, X) |
| Arbiter | from outside the debate | judges every open objection (real, refuted, already in today's spec) and every trade-off | rules |

## How one question goes

| Step | Who | What happens |
|---|---|---|
| 1 | Designer | the cause, the change at the cause, the impact report, decisions by the owner's parameters, tests |
| 2 | script | form only: every field there, every test real code, every objection answered (one resend) |
| 3 | 3 reviewers at once | attack it; every objection is a whole program walked through the rules |
| 4 | Designer | answers every objection: fixed, rebutted (quoting rules) or trade-off kept (reasoning and impact) |
| 5 | reviewers again | each marks its old objections resolved, withdrawn or still open, and adds new ones |
| 6 | Arbiter | rules on what is still open |

Steps 4 and 5 repeat up to three review rounds in all. The debate stops early when nothing blocks, or when
a round does not reduce the blocking objections (no endless loop). A whole missing feature, or a question
only the owner can answer, stops at step 1 and goes to the owner as a design question.

## The pass mark

Blocks (each needs a whole program walked through the rules, confirmed by the arbiter): **S** a safety hole
the change causes; **U** the question's own program still undecided; **X** a contradiction the change
creates. Everything else is weighed: the designer answers it, the arbiter judges the trade-off against the
goal, the owner reads it. A departure from a verdict is always listed for the owner.

Results: accepted; accepted, the owner must see trade-offs; rejected (with the program that blocks it);
whole_feature or owner_question (to the owner); unverified (a member was stopped: reported, never retried).

## What it fixes from the earlier runs

| Earlier failure | Method 4 |
|---|---|
| Fixes rewrote whole areas (method 1) | one question; the smallest change at the cause; the integrator checks the impact |
| Fixes stuck where the problem was reported (method 3, K07, K20) | the fix goes to the cause, wherever it is, with why_here and an impact report |
| Patches, one clause per hole | every revision rewrites the change as a whole; no clause per objection |
| One repair could not close real problems | up to three rounds, with every objection answered, and a stop when no progress |
| Every medium problem failed a fix | only safety, an undecided question or a new contradiction blocks; the rest is weighed and shown |
| Holes counted unchecked | the arbiter judges every open objection |
| Tests that were descriptions | the script rejects them before review |
| Areas inconsistent | the questions of one area run in order, each seeing the earlier answers |

## Agents and cost

Per question: 5 (nothing blocks in round 1), 9 (one revision), 13 (two revisions), plus up to three resends
on form. A whole feature costs 1.

Pilot: K07, K20 (both failed under method 3 on problems that were not safety) and K10 (the hardest,
unsolved since the first trial): 15 to 39 agents, likely about 25, about 3% to 9% of the week at the
trial's measured rate (about 0.22% per agent, taken from 47% -> 51% for 17 agents).

The rest, only after the pilot and the owner's word: about 70 questions, likely about 600 agents, more than
a week; the pace is the owner's.

Files: deep/v4/RULES-FOR-THE-TEAM.md (every agent reads it first), deep/v4/workflow.js (the method),
deep/v4/workflow_pilot.js (the pilot, inputs written in by code), deep/v4/make_report.py.
