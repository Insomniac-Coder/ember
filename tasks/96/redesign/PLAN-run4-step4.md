# Run 4, step 4: bring every design file to verdicts 1 to 97, once (PLAN, for the owner's approval)

## Goal

One update that makes the rules, the keyword list, the 84 worked examples and the 830 tests agree with
every owner decision in `JOB.md` (1 to 97), then a check that nothing disagrees. Every open question is
answered, so this is the last rewrite before the compiler work.

## Phase 0, the lead, no agents

- Copy `run4/v4` to `run4/v5`.
- A script (Python, mine) makes the spelling changes that need no judgement, only in program code: the
  test programs, and the NEW code blocks of the five example files (never OLD code, today's outputs or
  "means exactly" programs, which are today's Ember):
  - `changes ...` and `reads ...` taken off `fn` lines and function types (verdict 86);
  - `with` taken off what a function keeps: `fn() with copy of score` becomes `fn() copy of score` (83).
  It writes a report: every line it changed (before and after), every `changes`/`reads`/`with` it could
  not handle, and the 72 tests whose expected error belonged to the removed words (E2105, E2107, E2113,
  E2122, E2085, E0118). The script carries its own checks on sample lines.
- The agents' inputs: the shared brief, the duties, `JOB.md`, and a worklist: what verdicts 83 to 97 and
  the corrections to 53 and 58 change, and every fix item from step 3's checks (readability failures,
  wrong tests, unsettled points, the hole 3 and 6a remnants, the agents' message problems), each with the
  verdict that settles it or "the lead's: messages and codes".
- Merge step 3's six safety tables into one.

## Agents

| # | job | why its own job | model | effort |
|---|---|---|---|---|
| 1 | rules and keyword list | removing `changes`/`reads` reshapes the core rules; everything else reads them | Opus 5.5 | xhigh |
| 2 | examples A and B (AN-01 to AN-33) | 33 answers, light on the removed words | Opus 5.5 | high |
| 3 | examples C and D (AN-34 to AN-70) | 37 answers; links and callbacks | Opus 5.5 | high |
| 4 | examples E (AN-71 to AN-84) | threads: verdicts 89, 92, 93, 94 land here | Opus 5.5 | xhigh |
| 5 | tests R1-R6, R13, R17 (326) | 23 tests lose their error; `owns` verdicts 84, 85, 97 | Opus 5.5 | high |
| 6 | tests R7-R11 (216) | the loop rule (95) and the two-names copy (88) | Opus 5.5 | xhigh |
| 7 | tests R12 (118) | function values: 31 tests lose their error | Opus 5.5 | xhigh |
| 8 | tests R14-R16 (170) | threads and C: 17 lose their error; 89, 91-94, 96 | Opus 5.5 | xhigh |
| 9 | check: every verdict applied, no old spelling, every message to the message rule | | Opus 5.5 | high |
| 10 | check: every refusal and stop has a test, every test expects what the rules say | | Opus 5.5 | xhigh |

10 agents (step 3 had 14). Worst case 20 with one retry each. No Fable.

## Phases

1. Phase 0 (the lead), then agent 1.
2. Agents 2 to 8 together, after agent 1.
3. Agents 9 and 10 together.

Then the lead merges the four test agents' tables and reports to the owner.

## Cost

Step 3: 14 agents, 6.52 million agent tokens, 90 minutes (its own report). This step: 10 agents, with the
mechanical part done by the script; estimate 4.5 to 5.5 million tokens (step 3's average of 466 thousand
per agent, with more for agent 1), about the same time.

## Safeguards (as before)

The same brief for every agent; a form with `finished` from each; one retry, then the run halts; a safety
filter stop halts and is reported, never reworded; no compiler changes, no git writes, nothing compiled or
run; scratch files under `target/redesign7/`; mains power checked; the laptop kept awake during the run.

## Outputs

`run4/v5/`: rules, keyword list, changes, the five example files, `conformance/`, `safety/matrix.md`,
`readability.md`, `safety/recheck.md`, the script and its report; a short report for the owner.
