# The shared brief for the unsafe-features inventory (every agent reads this first, all of it)

You are one agent of a team of 6. Every agent has the same goal; you only have a different duty, in
`inputs/duties.md` in the section with your number. Read this brief, then `tasks/96/redesign/JOB.md` in
full (the owner's 98 verdicts on the new memory model), then your duty. Paths are under
`C:\Users\ism19\Code\ember\` unless they start with a drive. `V5` = `tasks/96/redesign/run4/v5/`, the new
design (`V5/rules.md`, `V5/vocabulary.md`, the answers `V5/area-*.md`, the tests `V5/conformance/`).
`SPEC` = `docs/spec-source/Ember_v0.9.10_Hardened_35.md`, today's spec.

## The language's goal

Ember is to be, all at once: as fast as C and Rust; as memory-safe as Rust or safer, checked when
compiling wherever possible; readable like Python, with what the code means written in words on the thing
it describes. Safety is never traded for looking like Python.

## This task (the owner's verdict 60)

"Anything that can break a safety promise is possible only inside an `unsafe` block." Find every feature
of Ember that can break a safety promise, and say whether it is possible outside `unsafe`. A feature
breaks a safety promise when, used in a program, it can:
1. **corrupt or misuse memory**: use after free, a double free, out-of-bounds access, reading memory not
   yet written, a view that outlives what it points into, a value of the wrong type read from memory;
2. **race**: two threads touching one place, at least one changing it, with nothing ordering them;
3. **leak**: memory never freed while the program runs, though nothing uses it any more (a count left too
   high, a loop of shared owners that keep each other alive);
4. **skip a check the rules promise**: bounds, overflow, aliasing, locks, threads, a callback's checks;
5. **reach memory the rules cannot see**: C code, raw pointers, untyped bytes, a cast that reinterprets
   memory, code loaded while the program runs;
6. **make the generated C undefined**: anything the C compiler is allowed to treat as impossible
   (signed overflow, a shift by the width or more, a misaligned access, `restrict` broken).

Decided already, do not reopen: `mem.forget` is removed (verdict 58); `into_raw()` exists only inside
`unsafe` (91); every count is checked before it rolls over (59); verdict 98 removed the run-time index
compare. A feature the new design (`V5`) removes or replaces (its keyword list says which) is not a
finding, unless the replacement has the same problem.

## What to write for each finding

In your findings file (your duty names it), one numbered entry per feature:
- **feature**: its name, and where it is defined (file and line);
- **how it breaks safety**: which of the six kinds above, and a minimal complete program that shows it
  (in the new design's spelling, `V5`, where the feature exists there; every name defined; say what it
  would do when run);
- **fenced?**: is it possible only inside `unsafe` today (`SPEC`, the compiler) and in the new design
  (`V5`)? Quote the line that fences it, or say that none does;
- **Rust**: what Rust does with the same thing (with its name in Rust);
- **recommendation**: fence it inside `unsafe` (and the error a use outside gets), make it safe with a
  rule (which rule, at what cost in speed), or remove it (and what to write instead);
- **settled?**: if a verdict in `JOB.md` already settles it, name the verdict and apply it; mark "for the
  owner" only if no verdict settles it AND the recommendation changes how an ordinary program is
  written or what it does.
Every claim names its source line. Say how sure you are; do not present a guess as a finding.

## Machine limits

- Read-only, except your own findings file and scratch files under `target/redesign8/<your number>/`.
  No compiler changes, no git writes, nothing compiled or run (you may run `grep`, `python` or `rg` to
  search). Never `cmd`, never `start`, never anything that opens a window. Do not check the laptop's
  power; set `on_mains` to true in your form.
- Plain words: the owner reads the results.
- When you finish, fill in the form exactly. `finished` is true only if you did your whole duty; if you
  were stopped before the end, `finished` is false and `reason` says why.
