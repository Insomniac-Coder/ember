# The shared brief for run 4, phase 1 (every agent reads this first, all of it)

You are one agent of a team of 7 revising a design for the owner of the Ember programming language. Every
agent has the same goal; you only have a different duty. Your duty is in `inputs/duties.md`, in the
section with your number. Read this brief, then `inputs/JOB.md` in full, then your duty. All paths are
under `C:\Users\ism19\Code\ember\` unless they start with a drive. `RUN4` means `tasks/96/redesign/run4/`
and `RUN3` means `tasks/96/redesign/run3/`.

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

## Where this run starts

Run 3 produced a proposal: `RUN3/combined-v4.md` (rules R1-R17 in its sections 0-9, then 84 answers in
five areas), with the vocabulary `RUN3/vocabulary-final.md`. Its findings are in
`tasks/96/redesign/FINDINGS-run3.md`. What went wrong in run 3, and what this run does about it:
- **Patching did not settle.** Each round of fixes added special cases to the rules and opened new
  problems (48, then 12, then 11). In this run a rule is never patched with one more special case: it
  is rewritten as a whole, simpler, or the problem is written down for the owner.
- **The rules grew unreadable**, with "Was: ..." history inside them. In this run the rules are written
  clean; history goes only in `RUN4/changes.md`.
- **Five programs the rules wrongly accept are known** (G-01 to G-05 in `RUN3/22-re-attacker.md`), and six
  readability problems (G-06 to G-11, same file). The revised rules must refuse those five programs and
  settle those six problems.
- **The owner decided four things on 2026-10-25** (in `inputs/JOB.md`, sections "Readability verdicts on
  run 3's syntax" and "Owner verdicts on run 3's open word questions"): a call that changes an argument
  says so, `bump(ref mut a)`; there is no `self`: inside a method the object is `this`, and a method that
  changes its object says `ref mut this`; a changeable link that other fields may also hold gets its own
  word (the owner suggested `sharedref mut` or `sref mut`); `if Some(ref mut me) = ui:` needs a major
  rework because it is not readable. These are binding.

## The owner's test (apply it to every line of NEW code you write or judge)

For every name in a line where the name is declared (a variable, a field, a parameter, a name bound in a
`match` or a loop, a captured variable), point at **the word in the line that says what it is** (the
owner's own thing, a copy, or handed over) and **the word that says whether it may change it**. If no word
says it, the line hides meaning and fails. A name with no word is allowed only where the default is
stated and cannot surprise a reader (it never changes anything, never aliases, never copies silently).
A call that changes an argument also says so (`bump(ref mut a)`); a method called on an object
(`a.reset()`) is not marked, because the method's first line says what it changes.

Keywords: a keyword says exactly what it does, means the same for every type and everywhere, and sits on
the thing it describes. `owned` means ownership handed over and replaces `Box`; a copy is `copied`;
`ref mut` means "only this name may touch it until it is done". "Keep" is the default for an existing
word; a rename needs a failed test and a reason a programmer would care about.

Error messages are part of readability: every refusal names what was refused in the program's own
names, why in one line, and the exact line to write instead.

The owner's agreed examples (the table in `inputs/JOB.md`, rows 3 to 9) are binding. Rows 3 and 7 lose
`self` under the owner's 2026-10-25 decision; their meaning is unchanged.

## The team

- 01 Rule writer: rewrites the rules and the vocabulary with the owner's decisions; makes the five known
  programs refused by simpler rules; settles the readability problems; writes `RUN4/rules.md`,
  `RUN4/vocabulary.md`, `RUN4/changes.md`.
- 02-06 Area updaters A-E: bring every answer of their area to the new rules and words.
- 07 Readability checker: checks every NEW block against the owner's tests and verdicts.

After this phase, the lead checks the revised rules for safety, and a later run measures speed.

## Machine limits

- No compiler changes. No git writes at all (no add, commit, reset, checkout, stash). Write only your own
  files: the files your duty names, and scratch files under `target/redesign4/<your number>-<your name>/`.
  Never edit a file another agent wrote.
- Today's Ember: `target/redesign4/bin/ember.exe check <file.em>` only (compiles, does not run). Never run
  an Ember program. Every claim about what today's Ember accepts or refuses carries the real output of
  that command, pasted (run 3's pasted outputs may be carried over unchanged when the OLD code is unchanged).
- Never `cmd`, never `start`, never anything that opens a window.
- Every number names its source (a log line, or a file and line). No invented numbers.
- If `powershell -NoProfile -Command "Add-Type -AssemblyName System.Windows.Forms;
  [System.Windows.Forms.SystemInformation]::PowerStatus.PowerLineStatus"` prints anything but `Online`,
  stop at once and say so in your form.
- Plain words in everything you write: the owner reads the results. Define every term you use.
- When you finish, fill in the form exactly. `finished` is true only if you did your whole duty; if you
  stopped early for any reason, set it false and say why in `reason`.
