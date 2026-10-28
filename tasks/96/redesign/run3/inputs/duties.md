# Duties, one section per agent (read `inputs/brief.md` and `inputs/JOB.md` first)

`RUN3` = `tasks/96/redesign/run3/`. Your report is `RUN3/<number>-<name>.md`; your code goes under
`target/redesign3/<number>-<name>/`. The spec is `docs/spec-source/Ember_v0.9.10_Hardened_35.md`.

## 01-04 Spec readers

| # | name | slice (spec lines) |
|---|---|---|
| 01 | reader-1 | 1-1491: Parts I-IV (overview, lexical incl. the keyword list II.4, grammar, types) |
| 02 | reader-2 | 1491-3177: Parts V-IX (declarations, expressions, ownership and borrowing, classes, memory facilities) |
| 03 | reader-3 | 3177-5015: Parts X-XVI (effects, threads, data-oriented, errors, compile-time, standard library, calling C) |
| 04 | reader-4 | 5015-6930: Parts XVII-XVIII, Annexes A-D, Appendices E-F (toolchain and diagnostics, requirements, syntax summary, hot reload, C++, GPU, coming from Python, glossary) |

Read your whole slice. List every word in it that a programmer writes to say something: keywords,
contextual keywords, `@` attributes, and types and functions that act as language words. Start from
`inputs/must-cover.md` (every word on it that appears in your slice must be judged) and add any other
such word you find. For each word, in your report:
- what it means at each place it is used in your slice, with line numbers;
- whether that is the same meaning everywhere, and the same for every type;
- whether it says in plain English what it does;
- whether it sits on the thing it describes;
- verdict: keep / rename / merge / remove, with the reason. Keep is the default; rename, merge or remove
  only when one of the owner's tests fails, and say which test and why a programmer would care.

04 only, in addition: go through the error-message table and every error example in your slice; list
every message that does not say what to write instead, with its code, line and a proposed message
written to the error-message rule.

Do not judge whether the memory model is good; that is phase 2. Judge the words.

## 05 Builder

Inputs: the four readers' reports `RUN3/01-reader-1.md` .. `RUN3/04-reader-4.md` (and their forms, given
in your prompt), `inputs/JOB.md` (the agreed table and keyword rules), `inputs/must-cover.md`, and for the
annoyance list `tasks/96/redesign/phase1-borrowing.md`, `phase1-sharing.md`, `FINDINGS.md`,
`FINDINGS-phase3b.md` (section 5) and the `phase3b-<area>.md` reports.

Write two files:
1. `RUN3/vocabulary-draft.md`: one table, one row per word, columns: word; today it means; it will mean;
   where it is written; which of today's words it replaces (if any); what stays refused; verdict and
   reason. Keep is the default. Settle disagreements between readers (one word, two meanings across
   slices) and say how. Every word on `inputs/must-cover.md` gets a row (judge yourself any word no
   reader judged; your prompt lists them). Every word used by the agreed rows 3-9 keeps its agreed
   spelling and meaning.
2. `RUN3/annoyances.md`: the full list of annoyances in today's Ember, each with an id (AN-01, AN-02, ...),
   a one-line title, the program's intent in one line, where it was found (file and line), and exactly
   one design area: A lists, loops, temporary values; B references kept or returned, locks, objects
   still being built; C ownership and sharing, `owned` replacing `Box`, copies, hand-over, state
   machines; D linked objects, callbacks, interface types; E threads, locks across threads, `unsafe`.
   Agreed rows go to: 5 and "delete after the loop" A; 3, 4, 7 B; 8, 9 and `owned` replacing `Box` C;
   6 D. Mark each annoyance that an agreed row already settles.

## 06 Checker

Inputs: `RUN3/vocabulary-draft.md`, `inputs/JOB.md`. Put every word through the owner's tests (brief):
does the line say what the name is and whether it may change it; does the word mean the same for every
type; does any word mean two things, or two words mean one thing; does it read right where it sits.
Fix or reject each failing row and say why. Write the approved table to `RUN3/vocabulary.md` (same
columns). Agreed rows 3-9 stay exactly as agreed.

## 07 Word planner

Inputs: `RUN3/vocabulary.md`, `RUN3/annoyances.md`, `inputs/JOB.md`. Before any design, decide the new
words the five areas will need, in one place, so the areas start from one vocabulary. For each new word:
spelling, exact meaning, where it is written, which annoyances (ids) need it, why no existing word
does, and an example line that passes the owner's test. Prefer no new word where an existing one says
it. Write `RUN3/planned-words.md`.

## 08-12 Area designers A-E

| # | name | area | last run's reports to repair | agreed rows to keep |
|---|---|---|---|---|
| 08 | area-A | lists, loops, temporary values | `phase3b-containers.md`, `phase3b-containers-attack.md` | 5; "delete after the loop" |
| 09 | area-B | references kept or returned, locks, objects still being built | `phase3b-references.md`, `phase3b-references-attack.md` | 3, 4, 7 |
| 10 | area-C | ownership and sharing, `owned` replacing `Box`, copies, hand-over, state machines | `phase3b-ownership.md`, `phase3b-ownership-attack.md` | 8, 9 |
| 11 | area-D | linked objects, callbacks, interface types | `phase3b-graphs-callbacks.md`, `phase3b-graphs-callbacks-attack.md` | 6 |
| 12 | area-E | threads, locks across threads, `unsafe` | `phase3b-threads.md`, `phase3b-threads-attack.md` | |

Inputs: `RUN3/vocabulary.md`, `RUN3/planned-words.md`, `RUN3/annoyances.md` (answer every annoyance of
your area; your prompt lists their ids), `inputs/JOB.md`, and last run's reports for your area (repair
what is there; anything thrown away gets its reason and its good parts listed in a "good parts" section).

For every annoyance, one answer in your report with these parts, in this order:
1. **Intent:** what the program is meant to do, one line.
2. **OLD:** today's Ember code, a real program, and the real output of `ember check` on it, pasted.
3. **NEW:** the proposed code, labelled NEW.
4. **The owner's test:** for every name in the NEW code, the word that says what it is and the word that
   says whether it may change it (or "none (default: ...)", only where the brief allows it).
5. **Means exactly:** the NEW code translated into today's Ember that compiles (with its `ember check`
   output) or, where today's Ember cannot say it, into C. The translation must do what the intent says.
6. **Still refused:** the code that stays refused, each with its error message written to the
   error-message rule.
7. **Cost:** what runs while the program runs, if anything, compared with C.
8. **Kind:** a readable spelling of Rust's guarantee; a smarter compile-time solution; a new mechanism;
   or an understandable limitation kept.

Use only words in `RUN3/vocabulary.md` and `RUN3/planned-words.md`. If you need another word, put it
in a "new words" table (spelling, meaning, where written, reason, example) and say which answers use it;
never use a word silently. Keep the agreed rows exactly as agreed.

## 13 Old-code re-checker

Inputs: the five area reports `RUN3/08-area-A.md` .. `RUN3/12-area-E.md`. For every OLD snippet and every
"means exactly" translation in today's Ember: save it under `target/redesign3/13-old-code-recheck/`, run
`target/redesign3/bin/ember.exe check` on it, and compare with the output pasted in the report. List
every mismatch (area, annoyance id, what was claimed, what really happens).

## 14 Combiner

Inputs: `RUN3/vocabulary.md`, `RUN3/planned-words.md`, the five area reports, `RUN3/13-old-code-recheck.md`,
`inputs/JOB.md`. Merge the five areas into one memory model, `RUN3/combined.md`, and one vocabulary,
`RUN3/vocabulary-combined.md`. Rule on every new word from every area (accept, reject, or merge into
another word) and rewrite the answers that used a rejected one. Where two areas solved one thing
differently, pick one, say why, and rewrite the other. Correct every mismatch the re-checker found.
List every clash and how it was settled. `combined.md` keeps the eight-part form for every answer,
grouped by area, with a summary at the top: the model in plain words, what still runs while the
program runs, and what stays refused. Agreed rows 3-9 stay exactly as agreed.

## 15 Vocabulary reviewer

Inputs: `RUN3/combined.md`, `RUN3/vocabulary-combined.md`, the readers' reports (their line-by-line lists
of every use of every word in the spec), `inputs/JOB.md`. Copy `combined.md` to `combined-v2.md` and
`vocabulary-combined.md` to `vocabulary-final.md` (with `cp`) and edit only the copies. Check:
- does every word read right where it sits, as plain English (as `fn() with copied score:` reads right
  and `copied fn():` does not)? does it name what it describes?
- does every word mean one thing, everywhere, for every type: in the NEW code, and at every use in the
  spec that the readers listed (a word whose meaning changes must not leave old uses meaning something
  else)?
- scan every NEW block in `combined.md` for words that are not in the vocabulary.
Fix the words and the code that uses them in the copies, and list every change with its reason.

## 16 Safety attacker, one thread

Input: `RUN3/combined-v2.md`, `RUN3/vocabulary-final.md`. Try to write programs the model accepts that
free memory twice or read freed memory, in a single thread. For each one: the NEW-syntax program, why
the model accepts it (quote the rule), and the proof: the C the compiler would emit for it, run under
AddressSanitizer, with the report pasted. A suspicion without an AddressSanitizer report is listed
separately as unproven. Look hardest where values are handed over, kept past a call, kept past a
change to their container, captured by callbacks, or freed while a name still points in.

## 17 Safety attacker, threads and shared objects

As 16, but for two or more threads and for shared (counted) objects: data races, a shared object freed
while another thread or name still uses it, a lock bypassed. Prove each with C under
AddressSanitizer (or, for a race, a deterministic interleaving forced in the C, with its wrong output).

## 18 Readability attacker

Input: `RUN3/combined-v2.md`, `RUN3/vocabulary-final.md`, `inputs/JOB.md`. For every NEW block: apply
the owner's test to every name; check its error messages against the error-message rule; compare it
with its "means exactly" translation and its intent line (does it do what the program means?); check it
reads like Python (easy to read and write) without hiding meaning; check every agreed row 3-9 is still
exactly as agreed. List every failure with the block, the test it fails, and why.

## 19 Fixer (and 21, round 2)

Inputs: the latest model (`RUN3/combined-v2.md` for 19, `RUN3/combined-v3.md` for 21), the problem
lists of 16, 17, 18 (for 21: of 20). Copy the model to the next version (`combined-v3.md` for 19,
`combined-v4.md` for 21) and edit only the copy. For every problem: fix it, or save what can be saved,
or discard the answer with its reason and its good parts. Show that each fix passes the test that
caught it (rerun the AddressSanitizer proof against the fixed C; reapply the readability test). You may
not add or change words: a fix that needs one is flagged "needs the owner" and left open.

## 20 Fix re-attacker (and 22, round 2)

Input: the Fixer's report and the model version it wrote. Attack only the fixes, with the same proofs
as 16-18. List new problems; an empty list means the fixes hold.

## 23 C writer (and 25, once, if 24 found gaps)

Input: the final model (the path is in your prompt). For W1-W4 (the workloads of
`tasks/96/redesign/phase3b-measurements.md`: W1 the ECS, W2 the tree with parent links, W3 game
objects referring to each other, W4 callbacks that remove themselves; their plain C is in
`target/redesign2/bench/`), write the Ember program in the model's NEW syntax and the C its compiler
would emit, under `target/redesign3/23-c-writer/` (25 writes to `target/redesign3/25-c-writer-fix/`).
Where the model's data layout differs from the plain C baseline, also write plain C in the model's
layout. Use the same random source and seed as the baselines, so the checksums must match. List every
run-time check the model keeps for each program and the line in the C where it is. Build each program
once with clang -O2 and run it once through the harness to show its checksum equals plain C's.
25 only: fix exactly the gaps 24 found.

## 24 Faithfulness reviewer

Input: the final model and the C writer's report and code. For each program: is every run-time check
the model keeps present in the C, and is nothing left out that the model would emit (no cheating in the
model's favour)? Is anything present that the model would not emit (unfair to it)? List each gap.

## 26 Timer

Inputs: the C writer's report (the final C, from 23 or 25) and the baselines in
`target/redesign2/bench/` (plain C `w1_group.c`, `w1_arch.c`, `w2.c`, `w3.c`, `w4.c`; safe Rust
`rust_w1.rs` .. `rust_w4.rs`, `rust_w4_return.rs`). Build everything fresh into
`target/redesign3/26-timer/bin/`. Time with
`powershell -NoProfile -ExecutionPolicy Bypass -File target/redesign2/bench/run.ps1 -Out <file> -Rounds 5 -Exes <list>`
(it pins to the fast cores, checks mains power before every round, opens no window). Report medians
and peak memory per program, the ratio to plain C, and the spread. If one program's runs differ by more
than 10% between fastest and slowest, run that set again (at most twice). Every checksum must equal plain
C's for the same workload. Run nothing else while timing.
