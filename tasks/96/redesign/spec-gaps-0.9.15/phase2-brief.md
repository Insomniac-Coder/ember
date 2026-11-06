# Phase 2: check each finding, and draft the survivors as questions for the owner

Read `brief.md` first (what a gap is and is not; read-only). Then your job's findings in
`phase1.json` (the entry whose `job` is yours).

## 1. Try to disprove each finding

For every finding, try to show it is **not** a gap: find the rule, table, example or owner verdict
(`tasks/96/redesign/JOB.md`, verdicts 1 to 163) that settles it, or show that the finder misread the
text. Search the whole specification (`docs/spec-source/Ember_v0.9.15_Hardened_11.md`), not only the
part it cites. Give each finding one `verdict`:

* `refuted`: the specification settles it, or the finder misread it. Quote what settles it, with lines.
* `already_decided`: an owner verdict settles it. Name the verdict and quote it. If the specification
  does not carry that verdict, or carries it differently, route it `lead_fix` (the lead writes the
  verdict in; nothing for the owner to decide).
* `confirmed`: you could not disprove it. When the evidence is thin, say so in `confidence`; do not
  confirm what you cannot show.

Two findings that are one question are merged: keep the first, list the other in `merged_with`.
You may add a finding the finder missed, marked `new: true`, held to the same bar.

## 2. Route each confirmed finding

* `owner_odr`: the owner decides it: whether a program compiles, or what it does.
* `lead_fix`: the text is stale against a decision already made (a verdict, or a rule the newer text
  plainly supersedes), so writing it in changes nothing the owner has not decided. Say which decision.

## 3. Draft each `owner_odr` as the owner reads it

What has worked with this owner, every time; what fails, every time:

* **A realistic program from a real kind of software** (a game engine, a renderer, a server, a parser),
  never toy names (`Foo`, `A`, `Mine`). The owner writes C++ game engines: name the danger or the idea in
  C++ terms in one phrase where it helps ("a `std::span` into a `std::vector` that `push_back` may move").
* **The whole program**, in the specification's 0.9.15 spelling, every name it uses defined in it, a
  comment on each line that matters. No fragments, no "as above".
* **`title`**: the question in at most 15 plain words. **`problem`**: two or three plain sentences: what
  the programmer writes, and what the specification does not say (or says twice).
* **`spec_today`**: the words of each rule involved, quoted with their lines.
* **`options`**: two or three, each as the code the programmer would write and what then happens
  (compiles or which error; what it prints or where it stops). Keep the letters fixed.
* **`recommendation`**: the option, and why, in one or two lines, checked against
  `docs/OWNER-DIRECTION.md`'s checklist: the language's own spelling reused, no exemption for missing
  syntax, no hidden special path, one rule wherever the same thing happens, safe and fast as C. Say which
  checklist items it meets. It must not contradict another of your recommendations.

Plain words throughout: no compiler internals, no verdict numbers in the `problem` or `options` (say what
a verdict decided instead), no unexplained terms.
