# The shared brief for the spec Ember_v0.9.15_Hardened_1 (every agent reads this first, all of it)

You are one agent of a team of 10 writing a new memory model into Ember's specification. Every agent has
the same goal; you only have a different duty, in `inputs/duties.md` in the section with your number. Read
this brief, then `tasks/96/redesign/JOB.md` in full (the owner's 99 verdicts), then `inputs/crosswalk.md`,
then your duty. Paths are under `C:\Users\ism19\Code\ember\` unless they start with a drive.
- `V5` = `tasks/96/redesign/run4/v5/`: the new design. `V5/rules.md` (R1 to R18, and sections 0 to 4:
  terms, run-time checks, refusals with messages, codes), `V5/vocabulary.md` (every word, today's and
  new), `V5/changes.md`, the 84 worked answers `V5/area-*.md`, the 842 tests `V5/conformance/`, and the
  unsafe-features inventory `V5/unsafe-inventory.md` (details and programs in `V5/inventory/06-checked.md`).
- `PARTS` = `tasks/spec-0.9.9/parts/`: today's spec (`Ember_v0.9.10_Hardened_35`), 28 parts joined in
  name order into one file. Its tools are in `tasks/spec-0.9.9/tools/`.
- `OUT` = `tasks/96/redesign/spec-0.9.15/`: your report goes to `OUT/reports/<number>.md`.

## The language's goal

Ember is to be, all at once: as fast as C and Rust; as memory-safe as Rust or safer, checked when compiling
wherever possible; readable like Python: what the code means is written in words on the thing it
describes, and the compiler never decides what the code does not say. Safety is never traded for looking
like Python. Error messages are part of readability: they name what was refused in the program's words,
say why in one line, and give the exact line to write, which must compile (verdict 73).

## What this step does

The new design replaces today's memory model: ownership, borrows and regions, `Box`, `Shared`, `Weak`,
`Cell`, `RefCell`, `SyncShared`, `ForeignBox`, `@sync`, `@borrows`, `owned`, `mem.forget`, effects written
on a function's first line, and the exclusivity checks that ran while the program ran. Write the new model
into the spec parts your duty names, so that the spec says exactly what `V5/rules.md` says, with every
verdict 1 to 99 and every fix of the inventory, and nothing of the replaced model is left. The result is
cut as `docs/spec-source/Ember_v0.9.15_Hardened_1.md`.

- **The source of truth** is, in order: `JOB.md`'s verdicts; `V5/rules.md` (with the lead's changes at
  the end of `V5/changes.md`); `V5/unsafe-inventory.md`. Where today's spec says otherwise, the new design
  wins. Where the new design says nothing, today's spec stays as it is.
- **Keep the spec's form**: Parts, sections (`## VII.1 ...`), rules as `* \`[FAM-n]\` text`, normative
  words (MUST, MUST NOT, MAY), examples as fenced `ember` blocks, the same plain register. Look at how a
  part is written before you change it. Write it clean: no history inside a rule, no "previously", no
  "v5", no verdict numbers in rule text (the lead writes the history into Appendix H).
- **Rule IDs**: a rule whose meaning stays keeps its ID, with its text brought to the new model. A rule
  whose subject is gone is removed (list it in your report). A new rule takes the next free number of a
  family your duty owns (check every part with `grep` first; never reuse a removed number). If a new rule
  belongs in a family another agent owns, write it in your part with an ID from a family you own, or list
  it in your report for the lead.
- **Codes**: refusals use the codes of `V5/rules.md` section 4 (one meaning each); codes it removes are
  gone. Run-time stops use section 2's stop texts.
- **Examples** use the new spelling and must be correct under the new rules (complete, every name defined).
  Today's compiler does not parse the new spelling: that is expected.
- Record, in your report, the mapping from each `V5/rules.md` rule (and section 2, 3 and 4 row) you wrote
  to the spec rule IDs that now carry it.

## Machine limits

- Edit only the parts your duty names, and your report. Scratch under `target/redesign9/<your number>/`.
  No compiler changes, no git writes. Nothing is compiled or run: never run the compiler (`ember`, `cargo`)
  or a program; the spec's tools `check_ids.py` and `check_citations.py` are plain Python and may be run
  (`python -I tasks/spec-0.9.9/tools/check_ids.py`); `check_examples.py` runs the compiler and may not.
  Never `cmd`, never `start`, never anything that opens a window. Do not check the laptop's power; set
  `on_mains` to true in your form.
- Plain words: the owner reads the results.
- When you finish, fill in the form exactly. `finished` is true only if you did your whole duty; if you
  were stopped before the end, `finished` is false and `reason` says why. A point no verdict settles that
  changes how a program is written goes in `for_owner` (expect none); a technical point you settle safely
  yourself and record in your report.
