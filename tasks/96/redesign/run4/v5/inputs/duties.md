# Duties for run 4, step 4 (read `inputs/brief.md`, `inputs/JOB.md` and `inputs/worklist.md` first)

`V5` = `tasks/96/redesign/run4/v5/`. "The report" is `V5/tools/step4_spell_report.md`.

## 01 Rule writer

Edit `V5/rules.md` and `V5/vocabulary.md` in place, and write `V5/changes.md` new: every change from v4's
files, each with its verdict; every code removed, added or changed; a section "Settled in step 4" (the
technical points you settled, with reasons); and "For the owner" (expected empty). Apply verdicts 83 to
97 and the corrections to 53 and 58 as `inputs/worklist.md` sections 1 to 3 say: above all section 2,
verdict 86 in the rules (R5 rewritten; every rule, row, message and example that uses `changes`, `reads`,
`captured`, change lists or `with` before captures brought to the new spelling, since the script did not
touch these two files). Then every worklist section 4 item marked **rules**. Section 4 of the rules: one
meaning per code; a code with no case left is listed as removed; a new code is checked unused in the
rules AND in today's compiler (`grep -rn "E1234" compiler/`) before you use it. Section 2 of the rules:
every run-time check with its stop text. Every message to the message rule and verdict 73. Write it clean:
no history inside a rule.

## 02-04 Answer updaters

| # | answers | file(s) |
|---|---|---|
| 02 | AN-01 to AN-33 | `V5/area-A.md`, `V5/area-B.md` |
| 03 | AN-34 to AN-70 | `V5/area-C.md`, `V5/area-D.md` |
| 04 | AN-71 to AN-84 | `V5/area-E.md` |

Inputs: `V5/rules.md`, `V5/vocabulary.md`, `V5/changes.md` (the rule writer's, done before you start),
your file(s), the report, `inputs/worklist.md`. Edit your files in place:
1. check every line the script changed in your files (the report's section 3) and correct any it got
   wrong; finish what it could not do: change-list type parameters left without a use (`type
   DeathHandler[C] = fn(...)`, `[C]`, `CHANGED`), prose, the owner's-test tables, messages and `text`
   blocks, comments;
2. bring every answer to verdicts 83 to 97 and the corrections (worklist section 1 names the answers
   each touches; check them all), and apply every worklist section 4 item that names your answers;
3. every message in your answers to the message rule and verdict 73, in the new spelling;
4. mark every answer whose meaning changed and say how, naming the verdict.
OLD code, today's outputs and "means exactly" programs stay unchanged. Add to your file(s) a short
section "What step 4 changed".

## 05-08 Test updaters

| # | group | tests in | tables |
|---|---|---|---|
| 05 | T1 | `V5/conformance/R1`, `R2`, `R3`, `R13`, `R4`, `R5`, `R6`, `R17` | `V5/safety/matrix-W1.md`, `matrix-W2.md` |
| 06 | T2 | `R7`, `R8`, `R9`, `R10`, `R11` | `matrix-W3.md`, `matrix-W4.md` |
| 07 | T3 | `R12` | `matrix-W5.md` |
| 08 | T4 | `R14`, `R15`, `R16` and the C side | `matrix-W6.md` |

Inputs: `V5/rules.md`, `V5/vocabulary.md`, `V5/changes.md`, the report, `inputs/worklist.md`, your tests
and tables, `V5/safety/recheck.md`, `V5/readability.md`. Do, in your tests and tables:
1. check every line the script changed in your tests and correct any it got wrong; finish what it could
   not do (`#$ error` and `#$ help:` messages, `##` comments, change-list type parameters);
2. every test in the report's section 1 that is yours (its expected error belonged to the removed words):
   make it the nearest test of what the program now does under the rules (the inferred check with its
   chain message, an `accept_`, or a `run_`), or delete it if nothing is left to test; list every
   deletion with its reason in your table;
3. verdicts 83 to 97 in your rules (worklist section 1, "tests"): for every new refusal one `reject_` and
   one `accept_` of the nearest allowed program; for every new behaviour a `run_` test; for every
   refusal a verdict removed (E2085, the map value writes of verdict 90, AN-71's float sum) the test
   turned to what the program now does;
4. every worklist section 4 item that names your tests (step 3's wrong tests, refusals without a test,
   the hole 3 and 6a remnants, message fixes);
5. your tables updated to match: every row, today's tests by file name, deletions listed.
Test format as in step 3: `#$ test:`, `#$ rules:`, `#$ stdout:`, `#$ panics:`, `#$ error[CODE]: message`
on the refused line, `#$ help:`; a `## ` comment quoting the guarantee; complete, minimal programs, every
name defined, one fault per refused test. Verdict 96's two tests add the header `#$ count_bits: 8`.

## 09 Verdict and readability checker (phase 3)

Write `V5/readability.md` new: every file (rules, vocabulary, the five answer files, every test) checked
against `inputs/JOB.md` verdicts 1 to 97: no old spelling left (`changes`/`reads` clauses, `captured`,
change lists, `with` before what a function keeps, `mem.forget`, `ForeignBox`, `owns` refused on a
number, a map or list changed by its own name inside its loop, `into_raw()` outside `unsafe`), every
verdict applied, the owner's test on every new or changed block, the message rule and verdict 73 on
every message. List every failure with the file and line, the test it fails and why. Do not fix anything.

## 10 Safety re-check (phase 3)

Write `V5/safety/recheck.md` new: every refusal in `V5/rules.md` section 3 and every code in section 4
has at least one `reject_` test; every run-time check in section 2 has a `run_` test with its stop text;
every test's expected result is what the rules say (name the rule and line; mark **wrong** or
**unsettled** otherwise); the six safety holes of `V5/safety/findings.md` section 1, and the two hole 3
remnants and hole 6a's form from step 3's re-check (worklist section 4), are each refused or made safe by
the rules, with the test that shows it. Do not fix anything.
