# Duties for run 4, step 1 (read `inputs/brief.md` and `inputs/JOB.md` first)

`V2` = `tasks/96/redesign/run4/v2/`, `RUN4` = `tasks/96/redesign/run4/`. Scratch files go under
`target/redesign4/v2-<number>-<name>/`: `v2-01-rule-writer`, `v2-02-area-A` .. `v2-06-area-E`,
`v2-07-readability`. Error codes: keep run 4's number where the meaning stays; a new meaning gets a new
number; a code whose rule is gone is retired (listed in `V2/changes.md`).

## 01 Rule writer

Inputs: `inputs/JOB.md` (the decisions: sections "Readability verdicts on run 3's syntax", "Owner
verdicts on run 3's open word questions", "Owner verdicts on run 4's open questions"); `RUN4/rules.md`,
`RUN4/vocabulary.md`, `RUN4/changes.md`; `RUN4/readability.md` (failures F-01 to F-04 are settled by the
decisions; F-05 to F-09, error messages, are the lead's); the "For the owner" and "for the lead" items in
`RUN4/area-A.md` .. `RUN4/area-E.md` (section 5 or the end of each).

Write three files:
1. `V2/rules.md`: `RUN4/rules.md` brought up to date with every decision, written clean (no history inside
   a rule), same sections (terms; rules; what runs while the program runs, against C; what stays refused,
   each with its message; error codes, one meaning each). In particular:
   - every spelling the decisions changed (`holds`, `weakref`, `copy of`, `owns`, `shares`, `using`,
     `this`, `changes this`, `static fn`);
   - `set by` removed, with its rule, codes and costs; parent links as ordinary `weakref` fields; the
     rule (or the honest limit) for proving, at compile time, that reading a parent link needs no
     "still exists?" check;
   - `changes this` without exclusivity or noalias for `this`; `ref mut` names and parameters keep
     "only this name touches it" (reads and writes);
   - function types say what they change (`fn() changes LOG`; plain `fn()` changes nothing outside
     itself), and a list or field of functions says it in its type (`Array[fn() changes LOG]`);
   - calls: `ref mut` at a call that changes an argument, a `MutSpan` argument included; `Mutex`,
     `RwLock`, `Atomic` and channel ends take no word;
   - a read-only name promises its target does not change while the name is used;
   - a lost change on a temporary (a method whose only effect is changing a temporary thrown away at the
     end of the line) refused, with a code;
   - one rule for every loop, generators included;
   - a walk up through `weakref` parent links stops at the first deleted parent;
   - the "for the lead" items from the area files that are rules or codes (missing codes; the
     "only adds" summary of a call; virtual methods' first lines as promises for their overrides; and
     the others listed there): settle each, or list it under "For the owner" with the reason.
2. `V2/vocabulary.md`: the word table with every word the rules and the 84 answers use, in the new
   spellings; add rows for the words the area updaters needed (`count`, `find`, `_`, `i16`, raw pointer
   types and their `read`/`add`, `user_data`, `Vec3.ZERO`) unless there is a reason not to.
3. `V2/changes.md`: every change from `RUN4/rules.md` and `RUN4/vocabulary.md`, each with the decision or
   item that caused it; every code change; and "For the owner".

## 02-06 Area updaters A-E

| # | name | area | input | answers |
|---|---|---|---|---|
| 02 | area-A | lists, loops, temporary values | `RUN4/area-A.md` | AN-01 to AN-16 |
| 03 | area-B | references kept or returned, locks, objects still being built | `RUN4/area-B.md` | AN-17 to AN-33 |
| 04 | area-C | ownership and sharing, `owns` replacing `Box`, copies, hand-over, state machines | `RUN4/area-C.md` | AN-34 to AN-52 |
| 05 | area-D | linked objects, callbacks, interface types | `RUN4/area-D.md` | AN-53 to AN-70 |
| 06 | area-E | threads, locks across threads, `unsafe` | `RUN4/area-E.md` | AN-71 to AN-84 |

Inputs: `V2/rules.md`, `V2/vocabulary.md`, `V2/changes.md`, your area's run-4 file, `inputs/JOB.md`.

Write `V2/area-<letter>.md`: every answer of your area in the eight-part form, brought to the updated
rules and words. The OLD code and its output carry over unchanged. Mark every answer whose meaning
changed and say how (area D: every answer that used `set by` is rewritten with ordinary `weakref`
parent links set by the program, and says what the walk costs). Use only words in `V2/vocabulary.md`;
anything else goes in a "words needed" table and is not used. Agreed rows exactly as agreed, in the new
spellings.

## 07 Readability checker

Inputs: `V2/rules.md`, `V2/vocabulary.md`, the five `V2/area-<letter>.md`, `inputs/JOB.md`.

Write `V2/readability.md`: for every NEW block, the owner's test on every declared name, the
error-message rule on every message, every one of the owner's decisions (is each applied everywhere,
with no old spelling left: `owned`, `shared` as an owner word, `copied`, `self`, `weak` alone, `set by`,
`Some(...)` in an `if` or `while`), and whether the agreed rows are exactly as agreed. List every
failure with the block, the test it fails and why. Do not fix anything.
