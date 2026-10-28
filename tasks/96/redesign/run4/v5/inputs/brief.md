# The shared brief for run 4, step 4 (every agent reads this first, all of it)

You are one agent of a team of 10 bringing a language design up to date with the decisions its owner
made. Every agent has the same goal; you only have a different duty. Your duty is in `inputs/duties.md`,
in the section with your number. Read this brief, then `inputs/JOB.md` in full, then `inputs/worklist.md`
(written by the lead: what each new verdict changes and where, and how every point left open by step 3 is
settled), then your duty. All paths are under `C:\Users\ism19\Code\ember\` unless they start with a
drive. `V5` means `tasks/96/redesign/run4/v5/`.

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

`V5` is a copy of step 3's result (`run4/v4`, at verdicts 1 to 82): `V5/rules.md`, `V5/vocabulary.md`,
`V5/changes.md`, the 84 answers in `V5/area-A.md` .. `V5/area-E.md`, the tests in `V5/conformance/R1/`
.. `R17/` (830 files), the safety tables `V5/safety/matrix-W1.md` .. `matrix-W6.md`, and step 3's checks
(`V5/readability.md`, `V5/safety/recheck.md`). The owner has since decided **verdicts 83 to 97** and
corrected the records of verdicts 53 and 58 (`inputs/JOB.md`); they are binding, and where any file says
otherwise, the verdicts win. **This is the last rewrite before compiler work: after it, every file agrees
with verdicts 1 to 97.**

Before you start, the lead's script (`V5/tools/step4_spell.py`) made the spelling changes that need no
judgement, in program code only (the test programs, and the NEW and Still refused code blocks of the
five answer files): it took `changes ...` and `reads ...` off `fn` lines and function types (verdict 86)
and `with` off what a function keeps (verdict 83). Its report, `V5/tools/step4_spell_report.md`, lists
every line it changed, and the 72 tests whose expected error belonged to the removed words. It did not
touch prose, messages, `#$` lines, comments, the rules or the keyword list: that is the agents' work.

## How open points are settled in this step

The owner has answered every open design question. So:
- Before you call anything open, look for it in `inputs/JOB.md` and in `inputs/worklist.md` section 4.
  If a verdict or the worklist settles it, apply that.
- A technical point that does not change how a program is written or what it means (which error code,
  which line carries an error, a table's row, a stop text's wording) you settle yourself, safely, and
  record it with your reason (the rule writer in `V5/changes.md` section "Settled in step 4"; the others
  in their report or table).
- Only a point that no verdict settles AND that changes how a program is written or what it does goes in
  your form's `for_owner`, with the rule, the line and the two readings in code. Expect none.

## The owner's test (apply it to every line of NEW code you write or judge)

For every name in a line where the name is declared (a variable, a field, a parameter, a name bound in a
`match`, a loop or an `exists as`, a captured variable), point at **the word in the line that says what
it is** (the owner's own thing, a copy, or handed over) and **the word that says whether it may change
it**. If no word says it, the line hides meaning and fails, unless the default is stated and cannot
surprise a reader. Keywords say exactly what they do, mean the same everywhere, and sit on the thing they
describe. **Error messages** name what was refused in the program's own names, say why in one line, and
give the exact line(s) to write; the fix a message gives must not run into another error, and where two
errors fit one line the one whose fix makes the program compile is shown (verdict 73). Since verdict 86,
what a function changes is not written: a message about a change a call makes names the chain of calls
that makes it ("`tick()` adds to `LOG` (through `note`, line 4) while this loop walks `LOG`"). The agreed
examples (rows 3 to 9 of the table in `JOB.md`) stay as agreed, in the verdicts' spellings.

## Machine limits

- No compiler changes. No git writes at all. Write only the files your duty names, and scratch files
  under `target/redesign7/<your number>/`. Never edit a file another agent owns.
- Nothing is compiled or run (the new spellings are not built); you may run Python or `grep` to check
  your own files. OLD code, today's outputs and "means exactly" programs stay unchanged. Every claim
  names its source.
- Never `cmd`, never `start`, never anything that opens a window.
- Do not check the laptop's power and do not stop because of it; the owner watches it. Set `on_mains` to
  true in your form.
- Plain words: the owner reads the results.
- When you finish, fill in the form exactly. `finished` is true only if you did your whole duty. If you
  were stopped for any reason before the end, `finished` is false and `reason` says why.
