# Duties for run 4, step 3 (read `inputs/brief.md`, `inputs/JOB.md` and `inputs/worklist.md` first)

`V4` = `tasks/96/redesign/run4/v4/`.

## 01 Rule writer

Edit `V4/rules.md` and `V4/vocabulary.md` in place, and write `V4/changes.md` new (every change from the
v3 files, each with its verdict; every code change; "For the owner"). Apply verdicts 53 to 82 as
`inputs/worklist.md` section 1 says (the "rules" and "vocabulary" columns), the error codes of section 2
(section 4 of the rules: one meaning each, the new codes, today's codes added, the corrected note), the
stop texts of section 3 (section 2 of the rules, where each run-time check is listed), and the message
corrections of section 4 that stand in the rules or the vocabulary. Write it clean: no history inside a
rule.

## 02-06 Area updaters A-E

| # | area | answers |
|---|---|---|
| 02 | A: lists, loops, temporary values | AN-01 to AN-16 |
| 03 | B: references kept or returned, locks, objects still being built | AN-17 to AN-33 |
| 04 | C: ownership and sharing, `owns`, copies, hand-over, state machines | AN-34 to AN-52 |
| 05 | D: linked objects, callbacks, interface types | AN-53 to AN-70 |
| 06 | E: threads, locks across threads, `unsafe` | AN-71 to AN-84 |

Inputs: `V4/rules.md`, `V4/vocabulary.md`, `V4/changes.md` (the rule writer's, done before you start),
your area file, `inputs/worklist.md`. Edit `V4/area-<letter>.md` in place: every answer brought to
verdicts 53 to 82 (the worklist's "answers" column names the likely ones; check them all), every message
in your answers to the message rule and verdict 73 (worklist section 4 lists the known ones). OLD code,
today's outputs and "means exactly" programs stay unchanged. Mark every answer whose meaning changed and
say how, naming the verdict.

## 07-12 Test updaters

| # | group | rules | tests in |
|---|---|---|---|
| 07 | W1 | R1, R2, R3, R13 | `V4/conformance/R1`, `R2`, `R3`, `R13` |
| 08 | W2 | R4, R5, R6, R17 | `R4`, `R5`, `R6`, `R17` |
| 09 | W3 | R7, R8 | `R7`, `R8` |
| 10 | W4 | R9, R10, R11 | `R9`, `R10`, `R11` |
| 11 | W5 | R12 | `R12` |
| 12 | W6 | R14, R15, R16 and the C side | `R14`, `R15`, `R16` |

Inputs: `V4/rules.md`, `V4/vocabulary.md`, `V4/changes.md`, your group's `V4/safety/matrix-W<n>.md` and
`V4/safety/check-V<n>.md`, `V4/safety/completeness.md`, `inputs/worklist.md`, your tests.

Do, in your group's tests and table:
1. every fix your checker (`check-V<n>.md`) marked **wrong**;
2. every **unsettled** point, now settled by verdicts 53 to 82 (or left in `for_owner` if no verdict
   settles it);
3. the missing tests of `completeness.md` that belong to your rules;
4. new tests for verdicts 53 to 82 in your rules (the worklist's "tests" column): for every new refusal
   one `reject_` and one `accept_` of the nearest allowed program; for every new run-time check a `run_`
   test with its stop text (worklist section 3);
5. the spelling of every test to verdicts 1 to 82 (for example verdict 74's `owns` on a making line);
6. your table, `V4/safety/matrix-W<n>.md`, updated to match: every row, today's tests by file name.

Today's test format, as in the safety run: `#$ test:`, `#$ rules:`, `#$ stdout:`, `#$ panics:`,
`#$ error[CODE]: message` on the refused line, `#$ help:`; a `## ` comment quoting the guarantee; complete,
minimal programs, every name defined, one fault per refused test.

## 13 Readability checker (phase 3)

Write `V4/readability.md`: for every NEW block in the five areas and every test changed or added in this
step, the owner's test on every declared name, the message rule and verdict 73 on every message, every
verdict 1 to 82 applied everywhere with no old spelling left. List every failure with the block, the test
it fails and why. Do not fix anything.

## 14 Safety re-check (phase 3)

Write `V4/safety/recheck.md`: every refusal in `V4/rules.md` section 3 and every code in section 4 has at
least one `reject_` test; every run-time check in section 2 has a `run_` test with its stop text; every
test's expected result is what the rules say (name the rule and line; mark **wrong** or **unsettled**
otherwise); the six safety holes of `V4/safety/findings.md` section 1 are each refused or made safe by
the rules, with the test that shows it. Do not fix anything.
