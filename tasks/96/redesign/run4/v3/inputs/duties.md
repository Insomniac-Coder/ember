# Duties for run 4, step 2 (read `inputs/brief.md` and `inputs/JOB.md` first)

`V3` = `tasks/96/redesign/run4/v3/`, `V2` = `tasks/96/redesign/run4/v2/`. Scratch files go under
`target/redesign4/v3-<number>-<name>/`: `v3-01-rule-writer`, `v3-02-area-A` .. `v3-06-area-E`,
`v3-07-readability`. Error codes: keep a code's number where its meaning stays; a new meaning gets a new
number; a code whose rule is gone is retired (listed in `V3/changes.md`). Every new or changed refusal has
a message that names what was refused in the program's own names, says why in one line, and gives the
exact line to write.

## 01 Rule writer

Inputs: `inputs/JOB.md` (verdicts 21 to 34; 1 to 20 are already in `V2`); `V3/rules.md` and
`V3/vocabulary.md` (phase 0 copies of `V2`'s, with `exists as` already in code); `V2/changes.md` (its
section 11, "For the owner", is answered by verdicts 21, 22, 23, 24 and 27); `V2/readability.md`
(F-12 is settled by 33 and F-14 by 34; F-01 to F-11 and F-13, error messages, are the lead's, phase 3:
leave them unless a rule you change rewrites that message anyway); the "For the owner" sections of
`V2/area-D.md` and `V2/area-E.md` (section 4.2; settled by verdicts 28 to 32 and 34).

Edit `V3/rules.md` and `V3/vocabulary.md` in place, and write `V3/changes.md` new. Write them clean: no
history inside a rule. In particular:
- 21: R12 states it; the vocabulary gives the library's first lines of `for_each`, `retain`, `inspect`
  and `sort_by` in that form.
- 22: R12: a function names every captured real variable it changes (`fn() changes score:`); plain `fn()`
  changes nothing at all; `captured` (one) and `captured...` (any number) in a field's or list's function
  type, with globals named (`Array[fn() changes captured..., LOG]`); the refusal of a two-variable
  function stored where `captured` allows one ("changes 2 captured variables, the list allows 1"). Agreed
  row 6 takes the new spelling.
- 23: R3: state it as settled.
- 24: R3 and the `using` row: the field names the kind, the making line names the arena; checked: the
  arena is of that kind, and no value outlives the arena it came from.
- 26: the prose, rows and messages that still say `holds` (the script changed code only; see
  `V3/spelling-changes.txt`).
- 27: rows `cast`, `is`, `as`, `if`; R13/R14: `if x is T as name:` (a place stays a place:
  `if this.pools[k] is Pool[T] as ref mut pool:`), `if x is T:`; `cast x as T` stays; `cast x if T` and
  `cast x when T` refused with the line to write; a `match` on a maybe-downcast becomes `if`/`else`.
- 28: R10/R11 and the parent-link text: the deleting walk's form, the refusal of a plain `ref` walk
  across a call that may delete, and the honest cost (one check and one count update per step), with
  where compile-time proof removes them.
- 29: R12's "need no word" for lock types is withdrawn for function types (verdict 3's call-site rule,
  no `ref mut` at a call for them, is unchanged); a call through such a function value inside a loop
  over the guarded list is then refused, with the message.
- 30: R5, R12, R15: `changes` is everything a function causes to change, whenever; `spawn`,
  `scope.spawn` and a `callback(retained ...)` parameter count as making the given function's changes
  (library first lines in the vocabulary); callers list them; a `spawn` inside a loop over what it
  changes is refused ("start the thread after the loop"). Say where the line falls between storing a
  function value (not a change: `setup` storing a button's handler names nothing, G-06) and handing it to
  something that will run it on its own (a thread, C).
- 31: R12/R15: every foreign call on a thread where kept callbacks may run counts as making the changes
  their first lines list; `@ffi(no_callbacks)` on a declaration exempts it (an asserted fact, listed by
  `ember tcb`); the E3020 message for `poll_events()` in a loop over `game.names`, and the message for an
  unmarked C function that never calls back, naming the mark.
- 32: R4: a parameter's rights come from the word in front of its name, for every type; E3023 for
  `with shares mut game` made from a plain `game: shares mut Game`, giving the `owns` lines.
- 33: the `.copy_each()` row (replacing `.copied()`), E3060's help.
- 34: R4/R5: R5's exception by type is withdrawn; a `MutSpan` name or parameter that writes says
  `ref mut`; the refusal of a plain one, with the line to write.
- New codes where needed; the error-code table with one meaning each.

`V3/changes.md`: every change from `V2/rules.md` and `V2/vocabulary.md`, each with the verdict that
caused it; every code change; "For the owner" (anything a verdict leaves open, with the reason).

## 02-06 Area updaters A-E

| # | name | area | answers | verdicts that land most |
|---|---|---|---|---|
| 02 | area-A | lists, loops, temporary values | AN-01 to AN-16 | 34 (`MutSpan` in most answers), 30 (`scope.spawn`, parallel loops), 21 |
| 03 | area-B | references kept or returned, locks, objects still being built | AN-17 to AN-33 | 29 (locks), 27 (downcasts), 34 |
| 04 | area-C | ownership and sharing, `owns`, copies, hand-over, state machines | AN-34 to AN-52 | 24 (arenas), 32, 33 (`.copy_each()`, AN-46), 23, 27 |
| 05 | area-D | linked objects, callbacks, interface types | AN-53 to AN-70 | 22 (agreed row 6, AN-53), 27 (AN-67, AN-68), 28 (AN-62), 31, 32, 21 |
| 06 | area-E | threads, locks across threads, `unsafe` | AN-71 to AN-84 | 29 (AN-75, AN-80), 30 (AN-76), 31 (AN-78), 32 (AN-78), 34 |

Inputs: `V3/rules.md`, `V3/vocabulary.md`, `V3/changes.md` (the rule writer's, done before you start),
your area's `V3/area-<letter>.md` (phase 0's copy of `V2`'s), `V3/spelling-changes.txt`, `inputs/JOB.md`.

Edit `V3/area-<letter>.md` in place: every answer of your area in the eight-part form, brought to the v3
rules and words and to verdicts 21 to 34. The OLD code, today's outputs and the "means exactly" programs
stay unchanged. Check every phase 0 line in your file (a wrong replacement is put right; an English
"holds" stays). Mark every answer whose meaning changed and say how ("**Meaning changed:** yes ..." as in
`V2`), naming the verdict. Use only words in `V3/vocabulary.md`; anything else goes in the "words needed"
table and is not used. The error messages listed as F-01 to F-11 in `V2/readability.md` are the lead's
(phase 3): leave them unless your update rewrites that message anyway, and list in your form every
message you think still breaks the message rule. Agreed rows exactly as agreed, in the verdicts'
spellings (row 6: `b.on_click(fn() changes score: score += 1)`).

## 07 Readability checker (phase 4, after the lead's message fixes)

Inputs: `V3/rules.md`, `V3/vocabulary.md`, `V3/changes.md`, the five `V3/area-<letter>.md`,
`inputs/JOB.md` (verdicts 1 to 52: the owner decided 35 to 52 during this step, after phases 1 and 2),
`V2/readability.md` (the failures the lead fixed in phase 3) and `V3/phase3-messages.md` (the lead's
log of every message fixed in phase 3; F-01 was settled by verdict 42 instead of rewritten).

Write `V3/readability.md`: for every NEW block, the owner's test on every declared name, the
error-message rule on every message, every one of the owner's verdicts 1 to 52 (25 excepted, replaced by
27) applied everywhere with no old spelling left (`holds`, `cast x if T`, `cast x when T`, `.copied()`, a
plain `MutSpan` name or parameter that writes, `owned`, `shared` as an owner word, `copied`, `self`,
`weak` alone, `set by`, `Some(...)` in an `if` or `while`, a function type that leaves out a change made
under a lock, a function that starts a thread without listing its changes, a send or a receive through
a given channel end not named (36, 37), a function that calls C without naming what its kept callbacks
change (39), a `=>` lambda that changes something without saying so (45), type parameters in brackets
after a generic function's, method's, struct's or class's name instead of on a line of their own above
it (50; enums, interfaces and `type` aliases keep the brackets), a `ref mut` name refused another ref
(42)). Verdict 50 covers every generic function, so it also applies to the examples written in
verdicts 38 and 49. Whether `V2/readability.md`'s F-01 to F-11 and F-13
are fixed, and whether the agreed rows are exactly as agreed. List every failure with the block, the test
it fails and why. Do not fix anything.
