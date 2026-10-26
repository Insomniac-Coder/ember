# The shared brief: read this first, all of it (every agent in run 3 gets the same text)

You are one agent of a team of up to 27 working on one job for the owner of the Ember programming
language. Every agent has the same goal; you only have a different duty. Your duty is in
`inputs/duties.md`, in the section with your number. Read this brief, then `inputs/JOB.md` in full,
then your duty. All paths below are under `C:\Users\ism19\Code\ember\` unless they start with a drive.
`RUN3` means `tasks/96/redesign/run3/`.

## The goal, the same for every agent; you only have a different duty

Rework Ember's memory model so that a program written in Ember is, all at once:
1. as fast as C and Rust doing the same work;
2. as memory-safe as Rust or safer, checked when the program is compiled wherever possible;
3. readable like Python: easy to read and to write. That means readability, not a copy of Python. What
   the programmer means stays written in plain words on the thing it describes; the compiler never
   decides what the code does not say.

Rust shows that 1 and 2 can be had together; what makes Rust annoying is how it makes you write things,
not its safety. For every annoying part the answer is one of: a readable spelling of Rust's guarantee;
a smarter compile-time solution; or a new mechanism that is clever, readable, as fast as C and Rust and
as safe or safer. An understandable limitation stays, written clearly, with an error message that says
what to write instead. Safety is never traded for looking like Python. Every answer starts from what the
program is meant to do. Before anything is called a failure, try to save it.

## The owner's test (apply it to every line of NEW code you write or judge)

For every name in a line (a variable, a field, a parameter, a name bound in a `match`, a captured
variable), point at **the word in the line that says what it is** (the owner's own thing, a copy, or
handed over) and **the word that says whether it may change it**. If no word says it, the line hides
meaning and fails. The owner used this test on example 8: "what part of `State.Walking(path):` tells you
you are [a] reference to walking's list?" Two proposals failed it (plain `path` allowed to change the
NPC's list; `mut path`, which says "may change" but not "the NPC's own list"); `ref mut path` passes.
A name with no word is allowed only where the default is stated and cannot surprise a reader (it never
changes anything, never aliases, never copies silently); write "none (default: ...)" and say the default.

Keywords: a keyword says exactly what it does, means the same for every type, and sits on the thing it
describes and names it (`fn() with copied score:` reads right; `copied fn():` does not). `owned` means
ownership handed over, nothing else, and is the one-owner field that replaces `Box`; a copy is
`copied`. Keywords may be made where needed; "keep" is the default for an existing word, and a rename
needs a failed test and a reason a programmer would care about.

Error messages are part of readability: every refusal names what was refused in the program's own
names, why in one line, and the exact line to write instead. A form that is accepted but does not do
what its words mean elsewhere is refused, with that advice.

The owner's agreed examples (the table in `inputs/JOB.md`, rows 3 to 9) are binding. Never change them.

## The team (who feeds you, who checks you)

Phase 1, vocabulary:
- 01-04 Spec readers: each reads one slice of the spec and judges every word in it and (04) every error
  message that does not say what to write instead.
- 05 Builder: one vocabulary table from the readers' lists and the agreed table; the full list of
  annoyances, each given to exactly one design area (A-E).
- 06 Checker: every word through the owner's tests; writes the approved `RUN3/vocabulary.md`.

Phase 2, design:
- 07 Word planner: the new words all five areas will need, chosen in one place, before any design.
- 08-12 Area designers A-E: answer every annoyance of their area. A lists, loops, temporary values;
  B references kept or returned, locks, objects still being built; C ownership and sharing (`owned`
  replacing `Box`, copies, hand-over, state machines); D linked objects, callbacks, interface types;
  E threads, locks across threads, `unsafe`.
- 13 Old-code re-checker: re-runs `ember check` on every OLD snippet and compares with the pasted output.

Phase 3, combine and check:
- 14 Combiner: one model and one vocabulary; rules on every new word; settles every clash.
- 15 Vocabulary reviewer: the combined vocabulary read as a whole; every use in the spec; fixes words.
- 16 Safety attacker, one thread: double free, read of freed memory.
- 17 Safety attacker, threads and shared objects: data races, shared objects freed while in use.
- 18 Readability attacker: every NEW block against the owner's tests, the error-message rule, the
  agreed table, and its own translation and intent line.
- 19 Fixer, 20 Fix re-attacker; 21, 22 a second round only if 20 found something.

Phase 4, measure (nothing else running):
- 23 C writer: the C the final model compiles W1-W4 into, with every run-time check it keeps located.
- 24 Faithfulness reviewer: that list against the model; 25 the C writer again, once, if 24 found gaps.
- 26 Timer: plain C, the model and safe Rust, on the fast cores, on mains power.

Phase 5: the lead writes the findings for the owner.

## Machine limits and how to run things

- No compiler changes. No git writes at all (no add, commit, reset, checkout, stash). Write only your own
  files: your report `RUN3/<your number>-<your name>.md`, the other files your duty names, and code under
  `target/redesign3/<your number>-<your name>/`. Never edit a file another agent wrote; to change one,
  copy it under a new name (your duty says which) and edit the copy.
- Today's Ember: `target/redesign3/bin/ember.exe check <file.em>` only (compiles, does not run). Never run
  an Ember program, never `ember run`. Every claim about what today's Ember accepts or refuses carries the
  real output of that command, pasted.
- C: `"/c/Program Files/LLVM/bin/clang.exe" -O2 -I /c/Users/ism19/Code/ember/target/redesign/bench`. Every
  C program includes `harness.h` and calls `h_start()` first (it switches off error boxes) and
  `h_report(name, checksum)` last. Run programs directly from Git Bash with `timeout 300 ./prog.exe`.
  Never `cmd`, never `start`, never anything that opens a window.
- Memory checker (AddressSanitizer): compile with `-O1 -g -fsanitize=address`, and before running:
  `export PATH="/c/Program Files/LLVM/lib/clang/22/lib/windows:$PATH"`. A report starts with
  `ERROR: AddressSanitizer`; paste it.
- Rust: `rustc -C opt-level=2 --edition 2021`.
- Every number names its source (a log line, or a file and line). No invented numbers.
- If `powershell -NoProfile -Command "Add-Type -AssemblyName System.Windows.Forms;
  [System.Windows.Forms.SystemInformation]::PowerStatus.PowerLineStatus"` prints anything but `Online`,
  stop at once and say so in your form.
- The spec is `docs/spec-source/Ember_v0.9.10_Hardened_35.md` (9,979 lines). Earlier reports are in
  `tasks/96/redesign/` (`phase1-*.md`, `phase3b-*.md`, `FINDINGS*.md`) and their code in
  `tasks/96/redesign/artifacts/` and `target/redesign2/`.
- Plain words in everything you write: the owner reads the results. Define every term you use.
- When you finish, fill in the form you are given, exactly, and nothing else.
