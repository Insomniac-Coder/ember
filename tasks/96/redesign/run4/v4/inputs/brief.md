# The shared brief for run 4, step 3 (every agent reads this first, all of it)

You are one agent of a team of 14 bringing a language design up to date with the decisions its owner
made. Every agent has the same goal; you only have a different duty. Your duty is in `inputs/duties.md`,
in the section with your number. Read this brief, then `inputs/JOB.md` in full, then `inputs/worklist.md`
(written by the lead: what each new verdict changes and where; the new error codes; the run-time stop
texts; the messages to correct), then your duty. All paths are under `C:\Users\ism19\Code\ember\` unless
they start with a drive. `V4` means `tasks/96/redesign/run4/v4/`.

## The goal, the same for every agent

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
program is meant to do.

## Where this step starts

`V4` holds copies of the current design: `V4/rules.md`, `V4/vocabulary.md`, `V4/changes.md`, the 84
answers in `V4/area-A.md` .. `V4/area-E.md` (all at verdicts 1 to 52), and the safety validation of those
rules: `V4/safety/matrix-W1.md` .. `matrix-W6.md` (the safety table, by group), `V4/conformance/R1/` ..
`R17/` (682 tests in today's conformance format), `V4/safety/check-V1.md` .. `check-V6.md` (the checkers'
results: 62 test mistakes with their fixes, 32 unsettled points), `V4/safety/completeness.md` (26 missing
tests) and `V4/safety/findings.md` (the summary). The owner has since decided **verdicts 53 to 82**
(`inputs/JOB.md`): the six safety holes the validation found, and every unsettled point. They are binding;
where any file says otherwise, the verdicts win. This step brings everything to verdicts 1 to 82.

## The owner's test (apply it to every line of NEW code you write or judge)

For every name in a line where the name is declared (a variable, a field, a parameter, a name bound in a
`match`, a loop or an `exists as`, a captured variable), point at **the word in the line that says what
it is** (the owner's own thing, a copy, or handed over) and **the word that says whether it may change
it**. If no word says it, the line hides meaning and fails, unless the default is stated and cannot
surprise a reader. Keywords say exactly what they do, mean the same everywhere, and sit on the thing they
describe. **Error messages** name what was refused in the program's own names, say why in one line, and
give the exact line(s) to write; the fix a message gives must not run into another error, and where two
errors fit one line the one whose fix makes the program compile is shown (verdict 73). The agreed
examples (rows 3 to 9 of the table in `JOB.md`) stay as agreed, in the verdicts' spellings.

## Machine limits

- No compiler changes. No git writes at all. Write only the files your duty names, and scratch files
  under `target/redesign6/<your number>/`. Never edit a file another agent wrote.
- Nothing is compiled or run in this step (the new spellings are not built). OLD code, today's outputs
  and "means exactly" programs stay unchanged. Every claim names its source.
- Never `cmd`, never `start`, never anything that opens a window.
- If `powershell -NoProfile -Command "Add-Type -AssemblyName System.Windows.Forms;
  [System.Windows.Forms.SystemInformation]::PowerStatus.PowerLineStatus"` prints anything but `Online`,
  stop at once and say so in your form (`on_mains`: false).
- Plain words: the owner reads the results. Anything a verdict leaves open goes in your form's
  `for_owner`, with the rule and the line; do not decide it yourself.
- When you finish, fill in the form exactly. `finished` is true only if you did your whole duty. If you
  were stopped for any reason before the end, `finished` is false and `reason` says why.
