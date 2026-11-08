# Spec gap sweep of 0.9.15_Hardened_11: the brief every agent reads

The owner (2026-11-02): "use a workflow if needed that goes through the spec and finds all gaps and
tries to fix them with consistency in mind". The owner decides the language; each gap found here goes
to them as a question (an ODR) with options and the lead's recommendation. **Nothing found here is
written into the specification without the owner's verdict.**

## What you read

* The specification: `docs/spec-source/Ember_v0.9.15_Hardened_11.md` (14,165 lines), assembled from
  `tasks/spec-0.9.9/parts/p*.md` (one file per part; quote line numbers from the part files).
* The owner's verdicts, 1 to 163, in their own words: `tasks/96/redesign/JOB.md`. A question a verdict
  already answers is not a gap (but see `verdict_missing` below).
* The lead's rulings and the open and answered questions: `tasks/96/redesign/compiler-0.9.15/decisions.md`.
* The owner's direction and consistency checklist: `docs/OWNER-DIRECTION.md` (read "The principles"
  and "Before ruling anything: the checklist"). Consistency is the yardstick for every fix you propose:
  reuse the language's own spelling, no exemptions for missing syntax, no hidden special paths, a
  written declaration means only what it says, one rule wherever the same thing happens.

## What a gap is (`kind`)

* `undecided_case`: a program the rules do not decide: whether it compiles, or what it does when run.
* `contradiction`: two rules (or a rule and a table) that say different things about the same program.
* `example_mismatch`: an example whose stated result does not follow from its own rule.
* `inconsistency`: the same thing done, spelled or allowed differently in two places with no reason
  given (a word allowed on parameters but not on fields, one kind of value treated unlike its siblings).
* `verdict_missing`: an owner verdict in `JOB.md` that the specification does not carry, or carries
  differently. Name the verdict number and quote both.

## What is NOT a gap (never report these)

* Wording, style, typos, a message's words, which error code a mistake gets, help text: the lead's.
* The toolchain (`ember` commands and flags), the compiler's internals, tests, build or CI matters.
* What the compiler builds or does not build yet ("not built by this compiler yet", `E0900`): an
  implementation gap, not a specification gap.
* Anything a rule elsewhere settles. Before you report, search the whole specification (grep the rule
  ids your finding cites, its key words and the types involved) and `JOB.md`; say in `searched` where
  you looked. If you are not sure it is unsettled, lower `confidence`; do not pad the list.

## Each finding

* `rules`: the rule ids (`LT-48`); `file` and `lines`: the part file and its line numbers; `quote`: the
  words that leave it open (at most three lines, exact).
* `question`: the question in program terms ("does this compile?", "what does it print?").
* `example`: a small, realistic Ember program in the specification's own 0.9.15 spelling (`owns` and
  `ref mut` written at calls, `this`, `exists as`, ...) that the rules do not decide. Prefer code from a
  real kind of program (a game engine, a parser, a server) over toy names.
* `why_unsettled` and `searched`; `severity`: `high` (memory safety, or what an ordinary program does),
  `medium`, `low` (a rare corner); `confidence` from 0 to 1.
* `fix_idea`: one or two lines: the fix most consistent with the rest of the language.

## How you work

* Read your assigned parts in full, every rule, and check each against the rest of the document.
* **Read-only.** Do not edit, create or delete any file. Do not run the compiler, `cargo`, Python or any
  build or test. Reading files and searching them is all you do.
