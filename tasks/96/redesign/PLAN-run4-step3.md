# Run 4, step 3: write verdicts 53 to 82 into the design, with the messages and the tests (PLAN, for the owner's approval)

## Goal

Bring `run4/v3/` up to every owner verdict, 1 to 82, as `run4/v4/`: the rules, the word list, the 84
answers, and the safety table and its 682 tests, with every error code, stop message and refusal message
written to the message rule (verdict 73: a message's fix must not run into another error).

## What goes in

- Verdicts 53 to 82 (`JOB.md`): the six safety holes (53 to 62), the open questions (63 to 82), `forget`
  removed (58), unsafe things only inside `unsafe` (60).
- The writing the owner delegated ("you can handle this"): the missing error codes, the run-time stop
  texts, and the messages that break the message rule (`run4/v3/safety/findings.md` section 3). The lead
  writes these in phase 0 as a worklist with every code and text, so the agents only place them.
- The checkers' 62 test fixes (`run4/v3/safety/check-V1.md` .. `check-V6.md`), the completeness checker's
  26 missing tests (`completeness.md`), and tests for verdicts 53 to 82.

## Phases

**Phase 0, the lead, no agents.** Copy `v3` to `v4`. Write `v4/inputs/`: the shared brief, the duties,
`JOB.md` with 82 verdicts, and `worklist.md`: for each verdict, the rules, rows, answers and tests it
touches; every new error code and stop text, and every corrected message, written out in full.

**Phase 1, one agent.** The rule writer: `v4/rules.md`, `v4/vocabulary.md`, `v4/changes.md`.

**Phase 2, eleven agents, all after phase 1.**
- Five area updaters (A to E): the 84 answers.
- Six test updaters (one per safety group, W1 to W6's rules): the checkers' fixes, the missing tests, new
  tests for verdicts 53 to 82, each group's table rows; the lead merges the six tables into one at the end.

**Phase 3, two agents.**
- The readability checker: every new block, every message, every verdict 1 to 82, no old spelling.
- A safety re-check: every refusal and run-time check in the rules has a test, and every test's
  expected result matches the rules (the checkers' bar from the safety run).

## Agents

| # | job | model | effort |
|---|---|---|---|
| 1 | rule writer | Opus 5.5 | xhigh |
| 2-6 | area updaters A-E | Opus 5.5 | high (E xhigh) |
| 7-12 | test updaters, one per group of rules | Opus 5.5 | high (R7-R8, R12, R14-R16+C: xhigh) |
| 13 | readability checker | Opus 5.5 | high |
| 14 | safety re-check | Opus 5.5 | xhigh |

14 agents; worst case 28 with one retry each. No Fable.

## Safeguards (as in steps 1 and 2 and the safety run)

The same shared brief for every agent; `finished` in every form, one retry, then the run halts; a safety
filter stop halts and is reported, no rewording; no compiler changes, no git writes, nothing compiled or
run; scratch under `target/redesign6/`; power checked.

## Outputs

`run4/v4/` (rules, vocabulary, changes, the five areas, `safety/matrix.md`, `conformance/`,
`readability.md`, `safety/recheck.md`), and a short report for the owner.

## After it

The inventory of escape hatches and leaky features against verdict 60, planned separately.
