# Duties for run 4, phase 1 (read `inputs/brief.md` and `inputs/JOB.md` first)

`RUN4` = `tasks/96/redesign/run4/`, `RUN3` = `tasks/96/redesign/run3/`. Scratch files go under
`target/redesign4/<number>-<name>/`: `01-rule-writer`, `02-area-A` .. `06-area-E`, `07-readability`.

Settled for everyone (from the smoke test's questions):
- **Owner questions:** a subagent cannot reach the owner. Anything left for the owner goes in a section
  "For the owner" (the rule writer: in `RUN4/changes.md`; the others: at the end of their own report);
  the lead raises it in chat.
- **`ember check` output** is needed only for claims about what **today's** Ember accepts or refuses. The
  new rules are not built, so a statement of what the new rules do needs no output.
- **Error codes:** keep run 3's number where the meaning stays; a new meaning gets a new number; the rule
  writer lists every code change in `RUN4/changes.md`.

## 01 Rule writer

Inputs: `inputs/JOB.md`; `RUN3/combined-v4.md` sections 0 to 9 (lines 1-843: terms, rules R1-R17, what
runs while the program runs, what stays refused, word rulings, clashes, error codes, open points);
`RUN3/vocabulary-final.md`; `tasks/96/redesign/FINDINGS-run3.md`; `RUN3/22-re-attacker.md` (G-01 to G-11:
the five programs the rules wrongly accept and six readability problems); `RUN3/20-re-attacker.md` and
`RUN3/21-fixer.md` (what round 2 changed and why); `RUN3/18-readability.md` (R18-01 to R18-22);
`RUN3/24-faithfulness.md` (two contradictions in the model, gaps 1 and 2).

Write three files:
1. `RUN4/rules.md`: the memory model's rules, rewritten clean and as simple as they can be while keeping
   every guarantee. Sections: terms; the rules (one short paragraph each, no history inside); what
   still runs while the program runs, against C; what stays refused, each with its error message
   written to the error-message rule; the error codes, one meaning each.
   - Apply the owner's four decisions of 2026-10-25 everywhere (brief, "Where this run starts").
   - Settle the spellings the decisions leave open, each passing the owner's test, with the reason:
     where `ref mut this` goes on a method's first line and how it sits with `changes <field>` (row 3);
     the word for a changeable link other fields may also hold (choose one of the owner's two,
     `sharedref mut` or `sref mut`; propose a third only if both fail the owner's test, with the reason,
     and list it under "For the owner"); a readable replacement for `if Some(ref mut me) = ui:` (opening a
     `weak` link or an `Option` and naming what is inside). These three were ordered by the owner, so the
     brief's "a rename needs a failed test" bar does not apply to them; it applies to every other word.
   - The five programs G-01 to G-05: rewrite the rules behind them (run 3's R2, R4, R5, R10, R11, R12,
     whichever are involved) as a whole, so that each of the five is refused. For each, name the rule
     that refuses it and give the error message. No rule gets a special case added for one program; if
     a program cannot be refused without one, say so and leave it for the owner.
   - The six readability problems G-06 to G-11: settle each in the rules, or leave it for the owner
     with the reason.
   - The two contradictions in `RUN3/24-faithfulness.md` (gap 1: the test on every step through a
     `weak` link in W2, R11 against AN-62; gap 2: one box per captured variable in W4, D1's text against
     the handler's state block): settle each in one rule.
2. `RUN4/vocabulary.md`: the word table updated (columns as `RUN3/vocabulary-final.md`), with every
   word the rules **and the 84 answers** use (carry over run 3's rows for words still used, changed where
   the new rules change them); `self` gone, `this` added.
3. `RUN4/changes.md`: every change from run 3's rules and words, each with its reason and the problem
   it settles (G-xx, R18-xx, owner decision); every error-code change; run 3's word rulings, clashes and
   open points that still stand; and the section "For the owner".

## 02-06 Area updaters A-E

| # | name | area | lines of `RUN3/combined-v4.md` | answers |
|---|---|---|---|---|
| 02 | area-A | lists, loops, temporary values | 844-3700 | AN-01 to AN-16 |
| 03 | area-B | references kept or returned, locks, objects still being built | 3701-5655 | AN-17 to AN-33 |
| 04 | area-C | ownership and sharing, `owned` replacing `Box`, copies, hand-over, state machines | 5656-7304 | AN-34 to AN-52 |
| 05 | area-D | linked objects, callbacks, interface types | 7305-9280 | AN-53 to AN-70 |
| 06 | area-E | threads, locks across threads, `unsafe` | 9281-11342 | AN-71 to AN-84 |

Inputs: `RUN4/rules.md`, `RUN4/vocabulary.md`, `RUN4/changes.md`, your area's lines of
`RUN3/combined-v4.md`, `inputs/JOB.md`.

Write `RUN4/area-<letter>.md`: every answer of your area, in the eight-part form (intent; OLD with its
`ember check` output; NEW; the owner's test for every declared name; means exactly; still refused with
its message; cost; kind), brought to the new rules and words: `this` for `self`, `ref mut` at calls that
change an argument, the new link word, the new replacement for `if Some(...)`, and any rule the rule
writer changed. The OLD code and its output carry over unchanged unless the OLD code changes. Mark every
answer whose meaning changed under the new rules, and say how. Use only words in `RUN4/vocabulary.md`;
if an answer needs another word, list it in a "words needed" table and do not use it. Agreed rows
exactly as in `inputs/JOB.md` (rows 3 and 7 without `self`).

## 07 Readability checker

Inputs: `RUN4/rules.md`, `RUN4/vocabulary.md`, the five `RUN4/area-<letter>.md`, `inputs/JOB.md`.

Write `RUN4/readability.md`: for every NEW block, the owner's test on every declared name, the
error-message rule on every message, the owner's verdicts of 2026-10-25 (no `self`; `ref mut` at calls;
the new link word; no `if Some(...)` form), and whether the agreed rows 3 to 9 are exactly as agreed.
List every failure with the block, the test it fails and why. Do not fix anything.
