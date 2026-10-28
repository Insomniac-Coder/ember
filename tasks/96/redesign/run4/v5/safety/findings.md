# Safety validation (b): findings (2026-10-26)

**tl;dr:** all 13 agents finished; nothing failed. The new rules now have a safety table (203 rows) and
681 tests in today's test format (the writers' count; 682 test files are on disk). The checkers confirmed 769 of their checks and found 62 test mistakes
(fixes written down, **not applied**, as the owner asked). The important result: **the checkers found
about six places where the rules, as written, would let a program read memory that has already been
freed.** Those need the owner's decisions before anything else. Nothing was compiled, run or committed.

## Numbers

| writer | rules | table rows | tests (accept / reject / run) | checker: right | wrong | unsettled |
|---|---|---|---|---|---|---|
| W1 | R1, R2, R3, R13 (owners, hand-over, words, `exists as`) | 42 | 67 / 60 / 6 | 154 | 14 | 6 |
| W2 | R4, R5, R6, R17 (calls, first lines) | 35 | 47 / 70 / 4 | 127 | 16 | 5 |
| W3 | R7, R8 (kept views, the list rule) | 31 | 57 / 62 / 3 | 136 | 7 | 8 |
| W4 | R9, R10, R11 (shared owners, counts, `weakref`) | 27 | 22 / 21 / 18 | 81 | 3 | 2 |
| W5 | R12 (function values) | 26 | 40 / 51 / 4 | 100 | 14 | 3 |
| W6 | R14, R15, R16, the C side | 42 | 52 / 82 / 15 | 171 | 8 | 8 |
| **total** | | **203** | **285 / 346 / 50** | **769** | **62** | **32** |

"Wrong" is a mistake in a test or a table row (fix written in the checker's file). "Unsettled" means the
rules do not decide what the right result is. The completeness checker (C1) found 26 cases no test
covers. Questions raised for the owner: writers 56, checkers 44, C1 12, with many repeats; the main ones
are grouped below.

## 1. Places where the rules would let freed memory be read (most important)

Each is a program the rules, as written, accept, which would free something while it is still in use.

1. **A view given to a function that then clears what it points into** (V3, confirming W3). `note(LOG[0])`
   is allowed (R8 (g), lines 390-392, because the view's last use is the argument). Inside
   `fn note(s: StrView) changes LOG:`, `LOG.clear()` and then `String(s)` reads text that was freed. R8
   (b)'s two-paths test (lines 351-353) only counts paths through a link or a kept view; a plain
   parameter and a module variable are neither.
2. **A method that destroys its own object** (C1). `ENEMIES[0].hit()`, where `hit` says
   `changes hp, ENEMIES` and clears `ENEMIES`: the object `hit` runs on is freed while `hit` still runs.
   No sentence of R4, R5 or R8 refuses it. Same shape as 1.
3. **A stored function that owns what a view reads** (V4). R10 point 1 (lines 424-427) only counts calls
   whose first line, summary or function type may end an owner. A plain `fn(Node)` "changes nothing", yet
   it may change its own captured state, and that state may own the node a walk's view is reading.
4. **`drop` and `weakref`** (C1, W4). The rules never say whether ending an object counts as a call of
   its `drop`, nor whether a `weakref` opened while `drop` runs finds the object alive. If it finds it
   alive, R10's keep-alive count goes from 0 to 1 and back to 0, which ends the object a second time.
   (Rust's `Weak::upgrade` already fails before `drop` runs.)
5. **Count overflow** (C1). The `shares` count is 32 bits and `mem.forget` exists; 2^32 forgotten owners
   wrap the count to 0, and the next owner's end frees a value still in use. No check is named (Rust stops
   the program here).
6. **Threads and C** (C1):
   - a stored function holding a one-thread owner, put in a `thread_shared` object: `fn() changes
     Counter` has no word that tells the compiler, so the one-thread count could be changed from two
     threads;
   - a C callback that ends its own registration while it runs: nothing says the trampoline keeps its
     slot until the callback returns.

## 2. Places where two rules disagree, or none decides (questions for the owner, later, one at a time)

- **Does a function name `changes T` for a change through a link when it reached the object through a
  `ref mut` parameter or its own local?** R5 lines 263-265 say yes; lines 286-287 say no. Raised by W2,
  V2, V3, W4, V4, W5 and V5: the most repeated question.
- **One mistake, two codes:** E3020 or E7018 (a function value changing `Log` called while walking
  `log.lines` under its lock); E7007 or E3020 (a scoped task changing what its parent walks); E3060 or
  E3022 (a list reshaped while a `ref mut` name of it is used).
- **R8 (b) against R10:** does ending the last owner of an object end a view opened from a link to it
  (E3060), or does R10 keep the object alive for the view?
- **`copy of` a struct that holds a `shares` or `weakref` field:** a new object, one more owner, or
  refused?
- **Verdict 52's own line** (`thread.spawn(fn() changes RESULTS: worker(r))` in a loop) has no capture
  word for `r`, which R15 refuses (E7004); it needs `with copy of r`.
- Smaller ones: may a name opened `exists as copy of v` be changed; does `copy of` need `Clone` for a
  `Copy` struct; what a partial hand-over refuses and with which code; does the list rule cover maps,
  sets and strings; does a push end a kept link to an element in its own block; is `stats.total += e`
  from two tasks a reduction or E7007; is `owns` on a making line (`mesh=owns Mesh(...) using current`)
  required, allowed or refused; what a call writes for a `const ref mut` parameter; may a function value
  outlive its maker when it uses the maker's real locals; is a sibling module "not visible" for
  `captured`; is a captured `Atomic` named by variable or by type; may a lambda with a plain parameter be
  given where the type's parameter is `owns` (E2020 against AN-56); must a `drop` that only reads say
  `changes this`.

## 3. Missing codes, stop texts and messages

- **Codes used but not in section 4:** E0100, E2030, E2040, E2050; E1030 used with a second meaning;
  none for a partial hand-over, `weakref` of a part, a first line in the wrong order, `with ref mut x`.
  Section 4's "bold = new" note is wrong: many unbolded codes are not in today's compiler either.
- **Run-time stops with no text:** `assert_disjoint` finding an overlap, a failed `cast s as Circle`, a
  C call after `unregister`, a callback called on the wrong thread, two `ref mut` names of one element.
- **Messages in the rules that give a line which does not compile** (breaking R18): E2036's
  `if v == n:`, E2116's `copy of` for a type with no `Clone`, E2118 dropping `let`, E3026 and AN-53, 59,
  60's E2113 leaving out lines, E2106, AN-67, the E3100 texts, the generator's E3020, L3015's `@safety`
  promise, E5013's wording, and the rules' own example `LOG.push(s.name)` (R12 line 547), which hands
  over a part of a read-only parameter.
- **Test runner limits:** no way to put a test on the far side of a package's edge (R17), or to check an
  interface-change report.

## 4. Test mistakes the checkers found (62, not applied)

Mostly: a refused test with a second mistake in it (so a correct compiler would give two errors), a
program with unused parts, an expected code that another rule overrides, a message that leaves a line
refused, today's related tests cited without file names. Each, with its fix, is in `check-V1.md` ..
`check-V6.md`.

## 5. Tests still missing

C1's 26 gaps (`completeness.md`), among them: a `drop` that pushes to a list being walked, a `drop` that
saves its own object (resurrection), 15 removed words with no "this word is gone" test, `callback(once)`
called twice, and the places where rules meet (a `weakref` opened inside a loop over the list holding its
target; a stored function in a shared object changed by a thread). The checkers add 75 missing edge cases.

## Files

- `matrix-W1.md` .. `matrix-W6.md`: the safety table, by group (not yet merged into one file).
- `../conformance/R1/` .. `R17/`: the tests (682 `.em` files with a `#$ test:` header; one more `.em`
  is R17's second package, `ledger`, with its `ember.toml` files; R15 has a small C stub,
  `kept_events.c`/`.h`).
- `check-V1.md` .. `check-V6.md`: the checkers' line-by-line results.
- `completeness.md`: C1's gaps.
- The run's full return value: `target/redesign5/result.json` (not in git).

## What was not done (the owner's instruction: note the findings and wait)

No fixes applied, the six tables not merged, no questions asked yet, nothing committed. Suggested order
when the owner is back: the six safety holes in section 1 first (each is a rule decision), then the
repeated question about `changes T` through a link, then the test fixes and the merge.
