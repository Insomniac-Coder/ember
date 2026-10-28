# The shared brief for run 4, step 1 (every agent reads this first, all of it)

You are one agent of a team of 7 bringing a language design up to date with the decisions its owner
made. Every agent has the same goal; you only have a different duty. Your duty is in
`inputs/duties.md`, in the section with your number. Read this brief, then `inputs/JOB.md` in full,
then your duty. All paths are under `C:\Users\ism19\Code\ember\` unless they start with a drive.
`V2` means `tasks/96/redesign/run4/v2/` and `RUN4` means `tasks/96/redesign/run4/`.

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

## Where this step starts

Run 4's first phase wrote the current design: `RUN4/rules.md`, `RUN4/vocabulary.md`, `RUN4/changes.md`
and the 84 answers in `RUN4/area-A.md` .. `RUN4/area-E.md`; its readability check is
`RUN4/readability.md`. The owner then answered every open question: **20 decisions, in
`inputs/JOB.md`, section "Owner verdicts on run 4's open questions"** (and the two sections before it).
These decisions are binding. Where an older section of `JOB.md` or a run-4 file says otherwise (for
example `owned`, `copied`, `self`, `weak`, `set by`, `if p = x:`), the decisions win:
- `holds` opens an `Option` or a `weakref`: `if cam.target holds p:`, `while cur.parent holds p:`.
- `weak` is `weakref`; another link to the same target is `a = weakref self.keep`.
- `copied` is `copy of`: `snapshot = copy of items`, `fn() with copy of score:`.
- `owned` is `owns`, `shared` is `shares`, the verb everywhere: `mesh: owns Mesh`,
  `fn add(owns child: Node)`, `other = shares items` (read only), `other = shares mut items`.
  `thread_shared` and `sharedref mut` keep their spelling.
- a memory manager is named with `using`: `mesh: owns Mesh using LevelArena`.
- there is no `self`; inside a method the object is `this`; `fn reset() changes this:` marks a method
  that may change its object (it promises nothing about other names, so no exclusivity, no noalias);
  `fn note(msg: String) changes log:` the narrower form; `static fn` for a function with no object.
- `set by` is removed: a parent link is an ordinary field set by the program
  (`child.parent = weakref this`); make that natural way fast by compile-time proof where it can be
  proved, and say honestly what it costs where it cannot.
- the other decisions (calls that change an argument say `ref mut`; `MutSpan` too, `Mutex` and friends
  do not; `ref mut` keeps out other reads and writes; a read-only name promises no change while in use;
  a lost change on a temporary is refused; one rule for every loop, generators included; functions and
  lists of stored functions say `changes X`; and the rest) are in `JOB.md`.

## The owner's test (apply it to every line of NEW code you write or judge)

For every name in a line where the name is declared (a variable, a field, a parameter, a name bound in a
`match`, a loop or a `holds`, a captured variable), point at **the word in the line that says what it
is** (the owner's own thing, a copy, or handed over) and **the word that says whether it may change
it**. If no word says it, the line hides meaning and fails. A name with no word is allowed only where
the default is stated and cannot surprise a reader (it never changes anything, never aliases, never
copies silently). Keywords say exactly what they do, mean the same everywhere, and sit on the thing
they describe. Error messages name what was refused in the program's own names, say why in one line,
and give the exact line to write instead. The agreed examples (rows 3 to 9 of the table in `JOB.md`)
stay as agreed, in the new spellings.

## The team

- 01 Rule writer: brings the rules and the vocabulary up to date with the 20 decisions; writes
  `V2/rules.md`, `V2/vocabulary.md`, `V2/changes.md`.
- 02-06 Area updaters A-E: bring every answer of their area to the updated rules and words.
- 07 Readability checker: checks every NEW block against the owner's tests and decisions.

After this step, the lead fixes the flagged error messages, and then plans a safety check of the rules.

## Machine limits

- No compiler changes. No git writes at all. Write only the files your duty names, and scratch files
  under `target/redesign4/v2-<your number>-<your name>/`. Never edit a file another agent wrote.
- Today's Ember: `target/redesign4/bin/ember.exe check <file.em>` only (compiles, does not run). Never run
  an Ember program. Claims about what today's Ember accepts or refuses carry the real output (run 4's
  pasted outputs carry over unchanged when the OLD code is unchanged). Statements about the new rules
  need no output (they are not built).
- Never `cmd`, never `start`, never anything that opens a window. Every number names its source.
- If `powershell -NoProfile -Command "Add-Type -AssemblyName System.Windows.Forms;
  [System.Windows.Forms.SystemInformation]::PowerStatus.PowerLineStatus"` prints anything but `Online`,
  stop at once and say so in your form.
- Plain words: the owner reads the results. Anything left for the owner goes in a "For the owner"
  section; the lead raises it.
- When you finish, fill in the form exactly. `finished` is true only if you did your whole duty.
