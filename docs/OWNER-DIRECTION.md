# The owner's direction for Ember: what every decision so far points at

Written 2026-10-29 at the owner's request ("understand the pattern of decisions that I have made so far
including the ones that moved the version from 0.9.10 to 0.9.15 ... each decision should tell you the
direction we are trying to go in"), from the redesign's verdicts 1 to 117 (`tasks/96/redesign/JOB.md`),
the answers of 2026-10-19 (`tasks/85/owner-answers.md`), `docs/AUTOPILOT.md`, and the ODR review of
2026-10-29 (`tasks/96/redesign/compiler-0.9.15/decisions.md`, "Owner verdicts, 2026-10-29").
**Read it before ruling anything.** Verdict numbers below are JOB.md's.

## The goal (non-negotiable)

**A robust, production-ready, easy-to-use language with memory safety.** In the owner's three parts
(JOB.md, "The goals, all at once"):

1. Speed on par with C and Rust.
2. Memory safety the same as Rust's, or better.
3. Python-like: easy to read and easy to write (readability, not Python's meaning).

All three at once. The redesign began because the first attempt "replaced compile-time checking with
run-time checks, so every design came out slower than C and still not safe". A solution that gives up
one of the three, or that is a shortcut, is not a solution.

## The principles

### 1. The compiler does the bookkeeping, behind the scenes; the programmer never does it by hand

Anything the compiler can work out, it works out, and the programmer neither writes nor sees it.

- Verdict 86: `changes` and `reads` removed; the compiler works out what every function changes from
  its code ("THIS CHANGES THING IS ANNOYING AS FUCK I THINK IT NEEDS TO GO AWAY"). The costs were stated
  and accepted (the `fn` line no longer shows it; a far edit can break a line, the error naming the chain).
- 2026-10-29, FFI-55: the header hash leaves the overlay line; Ember writes and checks a package file
  of its own, `ember.overlays`. The owner's two reasons: users don't see it, and Ember handles it
  behind the scenes elegantly. Then: every overlay is checked, the package's own included.
- Verdict 107: a value made by an allocator carries no hidden word when the allocator can do without one.
- Verdict 88: the same variable under two names is allowed; the compiler makes a copy of the function.
- Verdict 113 / `[IFC-6]`: a private generic function may leave its `needs` out; the compiler works it out.

### 2. What the programmer *means* stays written, in the line, in readable words

The boundary of principle 1. The owner's test (JOB.md, "Keywords that make sense"): for every name, point
at the word in the line that says what it is (yours, a copy, handed over, shared) and whether it may
change it. If no word says it, the design hides meaning and fails.

- Written: handing over (`owns`, verdicts 74, 84, 85, 97), copying (`copy of`, 12), changing an argument
  (`ref mut` at the call, 1), sharing (`shares`, `shares mut`, 4, 19), a link that keeps nothing alive
  (`weakref`, 9), what a function keeps when it outlives its maker (`owns n`, 80: over the compiler
  moving it in by itself "as Python does"), who may run a function (`thread_shared fn()`, 61: over the
  compiler finding it from the whole program).
- Inferred: what a function changes or reads (86), what its private generics need (113).
- The difference: a word that states a permission or an intent about a value (who owns it, who may
  change it through this name, which threads may run it) is written; a fact derivable from bodies that
  changes nothing about what you may do with the value is the compiler's. `pub` interfaces are written
  out (108): a library's promise does not depend on reading its body.
- "The compiler does not silently decide what the programmer did not say" (JOB.md). No silent copy, no
  silent aliasing (5). A change silently lost is refused (13).

### 3. A marking is said once, where it is true; never twice

- 2026-10-29: `[UNS-12]` reversed: "the function literally says that it is unsafe". The body of an
  `unsafe fn` is unsafe as a whole; code to be checked as safe is opted *into* checking with `safe:`.
  Then "anything inside unsafe block is considered unsafe by default even functions", closures too, and
  such a function, being an `unsafe fn`, carries `@safety` like any other.
- Verdict 69: `drop` never writes `changes this`: "it's a given".
- `# SAFETY:` explains a block; `@safety` states a function's precondition; one kind of note per kind
  of thing (2026-10-29, `W3013` kept).

### 4. One word, one meaning, for every type; one rule, with no exception by type

- Verdict 84: "EVERY OTHER THING HAS TO BE OWNED TO TRANSFER WHY THE FUCK INT GETS THE SPECIAL
  TREATMENT?" `owns` hands over every type the same way.
- Verdict 34: `MutSpan`'s exception by type withdrawn ("one rule"). Verdict 32: the word in front of a
  parameter's name sets the rights, for every type.
- Verdict 25 then 27: `cast s if Circle` rejected because "if has two meanings"; `is` may take a second
  job only because it reads naturally there (Swift, Kotlin, C#).
- Verdict 42: `const` means one thing: the name itself stays as it is.
- Verdict 26: `holds` "fails the readability test miserably"; replaced by `exists as`.

### 5. Consistency decides when two designs are otherwise equal

- Verdict 22: "for consistency sake lets go with b". Verdict 16: "same rule for every loop". Verdict 29:
  "one rule with no exception". Verdicts 63, 64: a change through a link is named wherever the link
  lives. Verdicts 109 to 112: `needs` everywhere a list of needs is written. Verdict 95: Rust's way "for
  both lists and maps".
- 2026-10-29: a C function declared in `unsafe extern` carries `@safety` "for consistency sake and it
  makes sense"; closures are covered like functions; every overlay is checked, your own included.

### 6. An exception exists only where the general rule would say something untrue, and it says why

- Verdict 3: `Mutex`, `RwLock`, `Atomic` and channel ends take no `ref mut` at the call, because "only
  me for the whole call" would be untrue for them.
- Verdict 69: `drop` and `changes this` (a given). Verdict 94: `scope.spawn` keeps a plain `fn()` (its
  tasks end with the scope).
- Verdict 90 was an exception (changing values inside a map loop); verdict 95 reversed it once it was
  clear it changed something being read. An exception that confuses is removed.

### 7. Refuse what is wrong, accept what is correct; the program's intent decides

- "Understand the intent of the program before deciding anything" (JOB.md). A rule that makes code
  compile but do something not meant is wrong, however safe (`[CLO-15]`'s copied `score`).
- Correct programs are made to work, not refused: verdict 11 (G-01 "made safe, not refused"), 88, 93
  ("both ways should be permissible ... as long as there is no clash"), 51, 66, 114 (Rust's compile-time
  way refuses fine programs, so Rust's run-time way, one compare, was taken), 2026-10-19's A6 (search
  more so reloads succeed instead of being refused).
- An understandable limitation is kept and written clearly ("deleting the values later on is fine");
  only what is annoying is fixed. A list does not change while a loop walks it (16, 72, 95).

### 8. Compile time first; run time only where compile time cannot decide without refusing correct programs, and then the cheapest measured way

- Refused when compiling, never checked while running: 43 (a lock taken twice; Rust hangs), 52, 55, 82,
  98 (the run-time `i == j` compare removed), 99 (a loop of owners proven freed or refused; the warning
  became an error), 100 (ECS clashes).
- Run time where it must be, at the lowest cost, measured: 59 (one compare before a count rolls over,
  over 64-bit counts that make every object bigger), 101 (two counts, measured: 25% faster for a
  thread's own objects, 9 to 13% slower when passed back and forth, 16 bytes more), 114 (one compare at a
  hand-over).
- AUTOPILOT §4: correct and fast together (run-time variant picking over always-wide types); the
  smallest type that holds the values; step up by need.

### 9. Robust and general, never a shortcut, never a test hack

- AUTOPILOT §4: "specific solutions are like ticking timebombs"; every solution general; build the better
  implementation, never the easier one; never hide a feature to get past a check (ODR-163).
- Verdict 14: `set by` removed: "all of that just because you couldn't optimise the existing way of
  writing". Make the natural way fast; don't add a feature to avoid the work.
- Verdict 96: the count-limit tests use a real 8-bit counter, not a test-only shortcut in the language.
- Verdict 115: "this is not a good limit to have what if I want to access and object with 10
  components?" Typed variadic generics instead of a tuple capped at 8.
- 2026-10-29: by type for stored functions was "just dumb"; the compiler follows each field.
- "Before considering anything as a failure ... maybe the idea was good but the implementation was not
  optimal" (JOB.md, last section). A failure is never presented as a design choice.

### 10. Readable like Python, spelled like what people already know

- Python's readability, not its meaning (JOB.md: "not an exact 1 to 1 copy").
- Familiar spellings win when they read well: `this` (the owner hates `self`), `static fn` (Java, C#,
  C++), C++'s pack dots (116: "the to be continued... vibe"), Rust's rules where proven (78, 95, 59),
  `ref mut` at the call after Rust's `&mut` (1).
- Words that read as English: `copy of`, `exists as`, `is Circle as c`, `needs (A, B)`, `weakref`.
- Long things named once (`type`, 48, 49); type parameters on their own line (50).
- Python habits beat Rust-style ownership where both are sound (std functions taking ownership of a
  callable was "THE DUMBEST DECISION EVER").

### 11. Anything that can break a safety promise lives only inside `unsafe`, and whoever writes it vouches

- Verdict 60: "for unsafe stuff it should only be possible in an unsafe block". 58: `mem.forget` removed.
  91: `into_raw()` only inside `unsafe`.
- 2026-10-29: a function C slips into an Ember field can only arrive through `unsafe` code, so the
  compiler does not refuse correct programs for it; the `unsafe` block's note carries the promise.
- Where a mark can be forgotten, the design makes forgetting it safe: verdict 31 chose
  `@ffi(no_callbacks)` (a missing mark only refuses a loop) over `@ffi(runs_callbacks)` ("a forgotten
  mark lets a crash through").

### 12. Error messages are part of readability

- Every refusal names what, why in one line, and the exact line to write (JOB.md, "Error messages are
  part of readability").
- The line a message gives must compile; between two errors on one line, show the one whose fix makes
  the program compile (73: "a rule is supposed to tell how to fix an issue not how to run into another").
- 2026-10-29: advice must be followable: "put lines 9 to 12" (the whole `if`) over "put line 11"; "remove
  this note" over "remove this line" when the line is the function.

## What is never mine to decide

- **Any change to what the language does.** It goes to the owner, with real code, one question at a
  time. Rules that agents, critics or I wrote into the spec without a verdict were the problem of
  2026-10-29: `[UNS-12]` (redesign AN-79), `[FFI-55]`'s hash (critic C26), `[IFC-6]` (108), `+` between
  bounds (109). When I find one, I tell the owner what it rests on and keep the question open.
- Reversing an owner's verdict; widening one; anything touching RageV.
- Under the delegation I rule only what the spec leaves open, and only the robust, general, measured
  answer that follows these principles.

## Before ruling anything: the checklist

1. What is the program meant to do, in one line? Does the rule make the code do exactly that?
2. Does it accept every correct program and refuse only what is really wrong (or an understandable
   limitation, written clearly)?
3. Is it decided when compiling? If anything runs, could compile time decide without refusing correct
   programs? If not, is it the cheapest way, measured against C or C++?
4. Does the programmer write only meaning (ownership, copy, sharing, change through a name, permission)?
   Is everything else done by the compiler, unseen?
5. Is anything said twice? Is every word one meaning for every type?
6. Is it consistent with the nearest existing rule? If it is an exception, is the general rule's
   wording untrue for this case, and does the exception say why?
7. Is it general, for every program and not one test or pattern? Is it production-ready, with no known
   ceiling left in?
8. If a mark is forgotten, is the result a refusal and not a crash?
9. Does every message give a line that compiles?
10. Does it change what the language does? Then it is the owner's question, not my ruling.
