# The shared brief for run 4, step 2 (every agent reads this first, all of it)

You are one agent of a team of 7 bringing a language design up to date with the decisions its owner
made. Every agent has the same goal; you only have a different duty. Your duty is in
`inputs/duties.md`, in the section with your number. Read this brief, then `inputs/JOB.md` in full,
then your duty. All paths are under `/home/user/ember/` (the checkout's root) unless they start with `/`.
`V3` means `tasks/96/redesign/run4/v3/` and `V2` means `tasks/96/redesign/run4/v2/`.

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

Step 1 wrote the current design in `V2`: `V2/rules.md`, `V2/vocabulary.md`, `V2/changes.md` and the 84
answers in `V2/area-A.md` .. `V2/area-E.md`; its readability check is `V2/readability.md`. Step 1 applied
the owner's verdicts **1 to 20** only. The owner has since decided **21 to 34** (`inputs/JOB.md`, section
"Owner verdicts on run 4's open questions", items 21 to 34). This step applies them. **Verdict 25
(`cast s when Circle`) is replaced by verdict 27 and is not applied.** The verdicts are binding; where an
older section of `JOB.md` or a `V2` file says otherwise, the verdicts win:
- 21: a library function that runs a function it is given accepts one that changes things, and the call
  counts as making those changes: `fn for_each[C](f: fn(T) changes C) changes C:`.
- 22: a function that changes a variable it captured says so (`b.on_click(fn() changes score: score +=
  1)`; agreed row 6 takes this spelling); a list or field of such functions says `captured` (exactly one
  captured variable) or `captured...` (any number), besides globals: `Array[fn() changes captured...,
  LOG]`.
- 23: after `other = shares mut items`, `items` keeps what it could do; both owners may change it.
- 24: the field says which kind of arena (`mesh: owns Mesh using LevelArena`); the line that makes the
  value says which arena (`owns Mesh(points=[0.0, 1.0]) using current`), which calls
  `current.allocate(...)`.
- 26: `holds` is replaced by `exists as`: `if cam.target exists as p:`, `while up exists as n:`.
- 27: a "not sure" downcast is `if s is Circle as c:` (`if s is Circle:` tests only); `cast s as Circle`
  stays for "sure" (stops the program if it is not one); `cast x if D` and `cast x when D` are gone.
- 28: a walk whose visit function may delete nodes holds a `weakref` and checks it at every step
  (`up = weakref leaf`, `while up exists as n:`, `if n.parent exists as p:`, `up = weakref p`); a plain
  `ref` walk across such a call is refused; the cost is accepted, and the next run still looks for
  compile-time proof that removes it.
- 29: a function that changes something under a lock says so in its type, as every function does
  (`fn() with thread_shared log changes Log: log.push("a")`), for `Mutex`, `RwLock` and `Atomic` too.
- 30: `changes` lists everything a function causes to change, whenever that happens. Starting a thread
  counts: `pub fn spawn[R, CHANGED](owns f: fn() changes CHANGED -> R) changes CHANGED ->
  JoinHandle[R]:`, so `main` lists what its threads change (`fn main() changes FRAME, LOG, RESULTS:`).
- 31: every C call counts as possibly running the callbacks C keeps, and makes the changes their first
  lines list, unless the C function's declaration says it never calls back: `@ffi(no_callbacks) safe fn
  sqrt(x: f64) -> f64`.
- 32: what a function may do with a parameter is set by the word in front of the parameter's name, for
  every type: `fn setup(game: shares mut Game)` only reads; to hand on an owner that may change the
  game, `fn setup(owns game: shares mut Game)`, called `setup(shares mut game)`.
- 33: `.copied()` is renamed `.copy_each()`.
- 34: a name or parameter holding a `MutSpan` says `ref mut` in front, like every other name that may
  change something it does not own: `d = ref mut tracks[dst_name].as_mut_span()`,
  `fn blend(ref mut dst: MutSpan[float], src: Span[float]):`.

**Phase 0, already done by the lead:** `V3/rules.md`, `V3/vocabulary.md` and `V3/area-A.md` ..
`V3/area-E.md` are copies of `V2` in which two scripts changed spellings in code only. First `holds`
became `exists as` (verdict 26: inline code spans, and `if`/`elif`/`while` lines in code blocks). Then
v2's opening `if cast X if T exists as n:` became `if X is T as n:` (verdict 27, in all code, since it
was never today's Ember), and `.copied()` became `.copy_each()` (verdict 33, only in an answer's parts 3
NEW, 4 the owner's test, 6 Still refused and 7 Cost). `.copied()` and a bare `cast x if D` are today's
Ember, so they stay in OLD code, today's outputs and the "means exactly" programs; where they are left
in rule text or in a sentence about v2's choice, the rule writer or the area updater rewords it.
`V3/spelling-changes.txt` lists every replaced line, every line still containing `holds`, and every line
still containing `.copied()` or `cast ... if`, with the reason it was left. Nothing else was changed.

## The owner's test (apply it to every line of NEW code you write or judge)

For every name in a line where the name is declared (a variable, a field, a parameter, a name bound in a
`match`, a loop or an `exists as`, a captured variable), point at **the word in the line that says what
it is** (the owner's own thing, a copy, or handed over) and **the word that says whether it may change
it**. If no word says it, the line hides meaning and fails. A name with no word is allowed only where the
default is stated and cannot surprise a reader (it never changes anything, never aliases, never copies
silently). Keywords say exactly what they do, mean the same everywhere, and sit on the thing they
describe. Error messages name what was refused in the program's own names, say why in one line, and give
the exact line to write instead. The agreed examples (rows 3 to 9 of the table in `JOB.md`) stay as
agreed, in the verdicts' spellings (row 6 now reads `b.on_click(fn() changes score: score += 1)`).

## The team

- 01 Rule writer: brings the rules, the vocabulary and the change list up to date with verdicts 21-34.
- 02-06 Area updaters A-E: bring every answer of their area to the updated rules and words.
- 07 Readability checker (later, after the lead fixes the error messages flagged in `V2/readability.md`):
  checks every NEW block against the owner's tests and verdicts 1 to 52 (the owner decided 35 to 52
  during this step, after phases 1 and 2; they are in `inputs/JOB.md` and already applied by the lead).

## Machine limits

- No compiler changes. No git writes at all. Write only the files your duty names, and scratch files
  under `target/redesign4/v3-<your number>-<your name>/`. Never edit a file another agent wrote.
- Today's Ember: `target/redesign4/bin/ember check <file.em>` only (compiles, does not run). Never run
  an Ember program. Claims about what today's Ember accepts or refuses carry the real output (pasted
  outputs carry over unchanged when the OLD code is unchanged). OLD code, today's outputs and "means
  exactly" programs (today's Ember) are never changed to the new spellings. Statements about the new
  rules need no output (they are not built).
- The probe files the answers cite (`old/...`, `means/...`, `q1_holds.em`) were in the laptop's ignored
  `target/` and are not in this checkout. Their pasted outputs carry over as above; a new claim about
  today's Ember needs a new probe in your scratch folder, checked with the command above.
- Never `cmd`, never `start`, never anything that opens a window. Every number names its source.
- Plain words: the owner reads the results. Anything left for the owner goes in a "For the owner"
  section; the lead raises it.
- When you finish, fill in the form exactly. `finished` is true only if you did your whole duty. If you
  were stopped for any reason before the end, `finished` is false and `reason` says why.
