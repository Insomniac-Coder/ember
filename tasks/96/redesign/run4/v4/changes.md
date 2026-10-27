# What changed from v3, and why (run 4, step 3, v4; agent 01, rule writer)

Every change from the v3 rules and vocabulary (`V3/rules.md`, `V3/vocabulary.md`, copied into `V4` at the
start of this step) to `V4/rules.md` and `V4/vocabulary.md`, each with the owner's verdict that caused it;
every error-code change; the stop texts; the messages corrected; and, at the end, what the verdicts leave
open for the owner. The rules and the vocabulary carry no history: it is all here. What changed before
(verdicts 1 to 52) is in `V2/changes.md` and `V3/changes.md`.

`V3` means `tasks/96/redesign/run4/v3/`, `V4` means `tasks/96/redesign/run4/v4/`, `RUN3` means
`tasks/96/redesign/run3/`. "Verdict N" is item N of `inputs/JOB.md`, section "Owner verdicts on run 4's
open questions"; "R1" .. "R18" are the v4 rules; "row N" is the owner's agreed row N; "worklist" is
`V4/inputs/worklist.md`. Terms are defined in `V4/rules.md` section 0.

No compiler change, no git write, no Ember program compiled or run. Commands run: a lookup of every
section-4 code in `compiler/ember_diag/src/codes.rs`, a search of the repository for the six new codes and
for `mem.forget`, and a scan of the area files (section 10). Power was on mains at the start
(`PowerLineStatus` printed `Online`).

## 1. Verdicts 53 to 82, and where each is applied

| verdict | what the owner decided | applied in |
|---|---|---|
| 53 | a function that uses a view parameter after changing what it changes is caught at the call, by a fact worked out from its body, kept in the interface file | R4 (new bullet "A view argument and what the callee changes"), R8 (b) and (g), R17, section 0 (summary), section 2 (the "nothing" row), section 3 (E3031 row), section 4 (**E3031**); vocabulary `changes`, `ref mut`, section 0 (view) |
| 54 | the same for a method's object, `this` | R4 (same bullet), R5 (`this` paragraph), section 3 (E3031's second message); vocabulary `this` |
| 55 | a function that may destroy class objects it owns names their type (`fn() with owns players changes Player`) | R12 (first bullet), R5 (`changes T` includes them), R10 point 1, section 3 (E2105 lambda row, E3060 `reset()` message), section 4 (E2105); vocabulary `fn`, `changes` |
| 56 | an object is ended the moment its last owner lets it go, before its `drop` runs | R11 (new paragraph), section 0 (ended), section 2 (the header-test row: nothing more runs); vocabulary `drop`, `weakref`, section 0 (class object), section 6.2 (`[DRP-1]`) |
| 57 | ending an object counts as a call of its `drop` | R5 ("Ending an object counts as a call of its `drop`"), R8 (c), R10 point 1, section 3 (E2105 row, E3020 message), section 4 (E2105, E3020); vocabulary `drop`, `changes`, `for` |
| 58 | `mem.forget` is removed | R1, R12 (C callback), section 3 (removed words, with E1010's message), section 4 (E1010); vocabulary `mem.forget` (removed, with today's uses listed), `@must_drop`, `Retained`, section 6.2 (`[OWN-6]`, `[THR-6]`) |
| 59 | an owner count, and the weak count, are compared with their maximum before each +1 | R9, R11, section 2 (two rows and two stop texts); vocabulary `shares`, `weakref`, `thread_shared`, `Shared`, section 6.1 and 6.2 |
| 60 | anything that can break a safety promise is possible only inside `unsafe` | R16 (one sentence); vocabulary `unsafe` |
| 61 | `thread_shared fn()`: a function type says whether any thread may run its functions | R12 (new bullet), R15, section 2, section 3 (E7015 message), section 4 (E7015); vocabulary `fn`, `thread_shared`, section 4 (placeholders) |
| 62 | a C callback that ends its own token stays alive until it finishes | R12 (stored function bullet), R15, section 2 (the count-on-own-state row); vocabulary `Retained`, `ForeignCallback` |
| 63 | a change through a link is named with `changes T` even when the link came in as a `ref mut` parameter | R5 (narrowed "needs no clause"), R4 (`ref mut b: shares mut Player`), section 3 (new E2105 row), section 4 (E2105); vocabulary `changes`, `shares` |
| 64 | a change through a link that is the function's own local is named too | R5, section 3 (same E2105 row); vocabulary `changes`, `shares` |
| 65 | verdict 52's line takes its own copy of the loop's variable | R15 (scoped-task bullet); vocabulary `thread.spawn`, `Scope` |
| 66 | `exists as copy of v` stays, and the two-line form is valid | R13; vocabulary `exists as`, `copy of` |
| 67 | the `v` of `exists as copy of v` may be changed | R2, R13, section 0 (view), section 3 (E3023 row title), section 4 (E3023); vocabulary `exists as`, `copy of`, section 0 |
| 68 | a `Copy` type always works with `copy of` (`@derive(Copy)` gives `Clone`) | R1, R3, section 3 (E2116 note); vocabulary `Copy`, `Clone`, `copy of`, section 0 |
| 69 | `drop` never writes `changes this` | R5 (new bullet), section 3 (new E2122 row; the removed-words row gives `fn drop():` for `fn drop(mut self)`), section 4 (**E2122**); vocabulary `drop`, `this`, `mut self`, `changes`, section 6.1 (`mut self`), 6.2 (`[DRP-1]`) |
| 70 | a call to a `const ref mut` parameter writes `const ref mut` | R4, section 3 (E3028 message), section 4 (E3028); vocabulary `const`, `ref mut` |
| 71 | a lambda's parameter says what it receives, matching the function type | R4, R12, section 3 (E2020 message), section 4 (E2020); vocabulary `fn` |
| 72 | the loop rule covers lists, maps, sets and strings, refused at the change | R8 (c), section 0 (reshape), section 3 (E3020 row and message), section 4 (E3020); vocabulary `for` |
| 73 | a message's fix must not run into another error; where two errors fit one line, the one whose fix compiles is shown | R18 (rewritten end), R8 (c), (f), R10 (closing), R15, section 3 (the lock message moved from E3020 to E7018), section 4 (E7018); every message corrected in section 8 |
| 74 | `owns` is written on the line that makes a value for an `owns` field | R3 (narrowed "no `owns` before a value"), section 3 (new E2123 row), section 4 (**E2123**); vocabulary `owns`, `using` |
| 75 | a captured `Atomic` is named by its variable | R12 (first bullet); vocabulary `Atomic`, `fn` |
| 76 | "can see by name", for `captured`, means declared or imported in that file | R12 (second bullet); vocabulary `captured` |
| 77 | adding to a field from several scoped tasks is a reduction too | R15, section 3 (E7007's example changed to a task that reads first), section 4 (E7007); vocabulary `Scope` |
| 78 | a field of a value may be handed over on its own, as in Rust | R2, section 3 (E3040 message, E3013 class-field message), section 4 (E3040, E3013) |
| 79 | `copy of` a struct that holds links copies the links | R3, section 0 (copy); vocabulary `copy of` |
| 80 | a function that outlives its maker takes what it keeps with `with owns` | R12 (placement bullet rewritten, new bullet), section 0 (box removed), section 2 (row), section 3 (E3026 message), section 4 (E3026); vocabulary `fn`, `with` |
| 81 | adding to a list does not end a link to an object in its own block | R8 (b), section 0 (reshape); vocabulary `sharedref` |
| 82 | a name opened from a link is not used across a call that may destroy what it reads | R10 (rewritten), R13, section 2 (rows), section 3 (E3060 messages), section 4 (E3060); vocabulary `exists as`, `weakref` |

## 2. The rules that were rewritten, and how

### 2.1 A call is held to what the callee's body does (verdicts 53, 54)

R4 has a new bullet. A view whose last use is an argument does not live across the call (R8 (g), as
before); the callee makes the promise, and now the compiler holds it to it: from each body it works out
which view parameters, `this` included, the function uses after a change it makes (through its own lines
and its callees' same fact; a call it cannot see counts as doing so for every change its first line or
type allows), keeps that fact with the function (the package's interface file, R17), and refuses a call
that hands such a parameter a part of what the function changes (E3031). The verdict's examples are the
rule's: `note(LOG[0])` for a `note` that clears `LOG` and then prints `s` is refused, `note("bye")` and a
print-then-clear `note` are allowed, and `ENEMIES[0].hit()` for a `hit` that clears `ENEMIES` and then
changes `this.hp` is refused. R8 (b) says where the two-paths test of v3 fell short (a parameter and a
module variable are neither a link nor a kept view) and points at the new check. **One reading taken**:
the verdict says "a view parameter"; section 0 of the rules calls a `ref mut` parameter a view too, and a
`ref mut` parameter handed a part of what its function frees has the same hole (and breaks the noalias
promise of R4), so the check covers plain, `Span` and `ref mut` parameters. Listed for the owner (section
11, item 2).

### 2.2 The end of an object (verdicts 55, 56, 57, 69)

- R5: "everything else it causes to change" now includes the class objects a function's own state may
  end (verdict 55) and the changes of the `drop` of every object it may end (verdict 57: "Ending an object
  counts as a call of its `drop`", with the verdict's `Enemy` and `LOG`). A new bullet: a `drop` names only
  what it changes outside its object, never `changes this` (verdict 69, E2122).
- R12: "its own state is itself, and a change to it needs no word" keeps one exception, ending a class
  object (verdict 55), with E2105's message.
- R11: an object is ended the moment its last owner lets it go, before its `drop` runs; a link opened in
  its `drop`, or the `drop` of anything it owns, finds it ended; `this.parent = None` stays allowed (verdict
  56). Section 0 has the term **ended**.
- R8 (c) lists the end of an object whose `drop` changes the walked list among the calls that change it.

### 2.3 A view from a link across a call that may end its object (verdict 82)

R10 is rewritten. v3 protected such a view in "the cheapest of" four ways (nothing; a deferred-drop
region; a count pair; an atomic count for a `thread_shared` object). Now it is checked when compiling: no
line or call in the view's life may end an object of that type, and after one the view is not used
(E3060, the verdict's `remove_all(ref mut players)` message). What may end one is read from the call's
first line, summary, arguments (`ref mut players`) or type (`changes Player`, verdict 55) and from `drop`s
(verdict 57). The atomic count stays only for a `thread_shared` object reached through a `weakref` (point
2). The deferred-drop region stays where R11 uses it (the walk), and section 0 now says so; the count pair
stays for that atomic count and for R12's count on a function value's own state. R13's "kept alive for its
last use by R10" says what R10 now does. A call that both changes and may end the object (`reset()`, a
`fn() changes Player`) fits R10 point 1 (E3060 at the later use) and R8 (g) (E3021 at the call); R10's
closing paragraph applies verdict 73's test: E3060 where using the name before the call or opening the
link again after it compiles, R11's E3021 for a walk, where it would not.

Section 2 changes with it: the v3 row "nothing, or a deferred-drop region, or a count pair (R10)" becomes
"nothing" for the view and a new row "one atomic count for the view's life" for point 2; run 3's region
number (0.944 against 1.283 for a count pair, AN-62) moves to the walk's row, where a region is still
placed.

### 2.4 Counts, `mem.forget`, `unsafe` (verdicts 58, 59, 60)

- R1: "A value always ends with its clean-up ... (`mem.forget` is not Ember, E1010; a value handed to C
  goes through the `Retained` token)". The removed-words row of section 3 gives E1010's message, word for
  word from the worklist.
- R9: before each +1 of an owner count it is compared with its maximum, and the program stops ("too many
  owners of a `Player` (4,294,967,295)"); R11 the same for the weak count. Section 2: both count rows say
  so; the stop texts are in its new table.
- R16: "Anything that can break a safety promise is possible only inside an `unsafe` block." The verdict's
  inventory is not part of a rule.

### 2.5 Threads and C (verdicts 61, 62, 65, 77)

- R12, new bullet: `thread_shared fn()` in a function type (the verdict's `Bus` lines); such a function
  holds `thread_shared` values or copies, never a `shares` or `shares mut` value or a `weakref` link of one
  (E7015, the worklist's widening of the verdict's "never a `shares` or `shares mut` value" to a one-thread
  `weakref`, which E7015 already refused). R15 points to it.
- R12, stored function bullet: a C callback that ends its own `Retained` token is unregistered at once
  and its block freed only when it returns (the verdict's `my_game` example). Section 2's row for the
  count on a handler's own state covers it (the trampoline's bookkeeping, not measured).
- R15: the loop's own variable, and a local of one round, are given to a task or a thread with
  `with copy of` (verdict 65's corrected line, `thread.spawn(fn() with copy of r changes RESULTS:
  worker(r))`); "scoped tasks need no capture words" now says "but for a loop's own variable".
- R15: a reduction on a field of an object the tasks reach (`stats.total += x`) is a reduction; a task
  that reads the value first, or a `thread.spawn` thread, still needs an `Atomic` or a lock (verdict 77).
  Section 3's E7007 example, which refused `stats.total` changed by two tasks "each without a lock", is
  now such a reduction, so the example is changed to a task that reads the total first, with lines that
  compile (imports included).
- R15: the `threads(creator)` check runs in every profile (the worklist's stop text); section 2's
  trampoline row says what that adds in the release profile (one compare per such callback, not
  measured; today the check is debug-only, `tests/conformance/FFI-33b`).

### 2.6 A change through a link is named, whatever holds the link (verdicts 63, 64), and verdict 36's example

R5's "A change through a `ref mut` parameter or to an own local needs no clause" is narrowed: "unless the
change is made through a link", with the verdicts' `tick(ref mut c: shares mut Clock) changes Clock` and
`main() changes Clock`. R4's `ref mut b: shares mut Player` says the change is named. **A sentence added
from verdict 36's own example**: "A change a callee's first line names by a type counts for its caller
whatever it was given, its own local included" (`fn() with owns tx changes Sender[int]: produce(tx)`,
where `tx` is the lambda's own state). It settles the safety check's most repeated unsettled point after
verdicts 63/64, AN-30's `main` with its own `Mutex` (check-V2, R4-13 and R5-5): `main` names `changes
Mutex[...]`. Listed for the owner as a confirmation (section 11, item 3), since it changes AN-30's NEW.

### 2.7 Copies, parts and `owns` (verdicts 66, 67, 68, 74, 78, 79)

- R2: a name opened `exists as copy of v` may be changed (verdict 67); R13 says the two-line form is valid
  (verdict 66). Section 0's "view" leaves `exists as copy of` out.
- R1, R3: `copy of` works on every `Copy` type, `@derive(Copy)` giving `Clone` (verdict 68).
- R3: a value that holds links is copied field by field, each link copied as a link to the same target
  (verdict 79's `Team`); section 0's "copy" says what "never a link" means now (the link `copy of` is
  written before, not a link inside a struct it copies). Every link of section 0 is copied this way,
  `sharedref mut` included (one more holder); listed for the owner (section 11, item 4).
- R3: `owns` is written, and required, on the line that makes a value for an `owns` field, with or without
  an arena (verdict 74, E2123); the v3 sentence "it is written only where the default is a view" is
  narrowed.
- R2: a part is handed over on its own (`names.push(p.name)`), with E3040's message from the worklist; a
  field of a class object is not handed out (E3013), `mem.take` or `mem.replace` is the way (verdict 78).

### 2.8 Smaller rules (verdicts 70, 71, 72, 75, 76, 80, 81)

- R4: a call writes `const ref mut` for a `const ref mut` parameter (verdict 70, E3028).
- R4, R12: a lambda's parameter matches the function type, `fn(owns s)` for `fn(owns String)` (verdict
  71, E2020).
- R8 (c), section 0: the loop rule covers maps, sets and strings, at the change (verdict 72).
- R12: a captured `Atomic` is named by its variable (verdict 75).
- R12: "can see by name" is "declared or imported in the file" (verdict 76, the `game/button.em` example).
- R12: the v3 sentence that chose where a real variable lives (its address, the function value's state,
  or a box) is replaced: without a word a function only borrows, the real variable stays at its address,
  and a function value that may outlive it is refused (E3026); a new bullet says that a function that
  outlives its maker takes what it keeps with `with owns` (`return fn() with owns n: n += 1`, `fn
  make_counter() -> fn():`) (verdict 80, over the compiler moving `n` in by itself). Section 0's "box"
  term goes; section 2's row keeps run 3's measured state-block number and says the box is no longer made.
- R8 (b), section 0 (reshape): a push does not end a link to a class object, which lives in its own
  block; a link to an inline element is still ended (verdict 81).

### 2.9 Messages and the tie-breaker (verdict 73)

R18 now says that a message's fix never runs into another error and that, where two errors fit one line,
the one whose fix makes the program compile is shown, with the cases this step settled by that test:
- E7018 over E3020 for a call through a function value that may take a lock this thread holds, inside a
  walk under that lock (the verdict's own case). Section 3's E3020 row lost that message; it is now the
  E7018 row's "in a walk" message, and the E3020 row's title says where it went.
- E3020 over E7007 for a scoped task started inside a walk of what it changes (the worklist's "E7007
  against E3020": E3020's fix, "start it after the loop", compiles; E7007's, a lock or an `Atomic`, leaves
  the walk refused). R15's v3 sentence "a scoped task's change to what its parent walks is E7007" is
  changed accordingly. This answers area E's note (`V4/area-E.md`, the note on verdict 52).
- E3060 against E3022 (a position used after a removal of its list while it is a `ref mut` name): both
  fixes compile, so the test does not choose; R8 (a), the rule that names the case, already says E3060,
  and R8 (f) now defers to it.
- E3060 against E3021 for a call that may change and end an object a name opened from a link reads (2.3).

## 3. Every change from v3's rules

| v3 | v4 | what changed | why |
|---|---|---|---|
| title, opening | title, opening | step 3, v4; verdicts 1 to 82; `V4` paths; the safety validation named | — |
| section 0 | section 0 | copy (links in a copied value); view (`exists as copy of` is not one); reshape (maps, sets, strings; a push does not move a class object); **ended** (new); count pair and deferred-drop region (where each is still used); summary (the view-parameter fact); box (removed) | 53, 56, 67, 72, 79, 80, 81, 82 |
| R1 | R1 | every `Copy` type has `Clone`; a value always ends with its clean-up, `mem.forget` gone | 58, 68 |
| R2 | R2 | a part handed over on its own, E3040's message, a class field not handed out; `exists as copy of v` may be changed | 67, 78 |
| R3 | R3 | `copy of` on `Copy` types; a value holding links copied with its links; `owns` required on the making line of an `owns` field (E2123); `ref mut` in a capture list refused (E0120) | 68, 74, 79; worklist section 2 (E0120) |
| R4 | R4 | a lambda parameter matches the function type; `const ref mut` at the call; new bullet: a view argument and what the callee changes (E3031); `ref mut b: shares mut Player` names its change | 53, 54, 63, 70, 71 |
| R5 | R5 | class objects its own state may end; ending an object counts as a call of its `drop`; new `drop` bullet (E2122); the order refused with E0118; `this` checked at the call (E3031); "needs no clause" narrowed for links; a callee's type-named change counts for its caller | 53, 54, 55, 57, 63, 64, 69; 36's example; worklist section 2 (E0118) |
| R6, R7 | R6, R7 | unchanged | — |
| R8 | R8 | (b) the parameter-and-module-variable case points at R4; a push keeps a link to a class object; (c) maps, sets, strings, refused at the change; `drop`; the lock case is E7018; (f) a reshape of a position's list is E3060; (g) the call checked by the callee's body | 53, 57, 72, 73, 81 |
| R9 | R9 | the count checked before it rolls over | 59 |
| R10 | R10 | rewritten: checked when compiling, a view not used after a line or call that may end its object (E3060); the region and the count pair no longer used for it; the atomic count for a `thread_shared` object through a `weakref` stays; the tie-break with R8 (g) | 55, 57, 73, 82 |
| R11 | R11 | `weakref` of a part refused (E2119); the weak count checked; ended before `drop`; the walk's region named from section 0, with its cost | 56, 59; worklist section 2 (E2119) |
| R12 | R12 | a captured `Atomic` named by its variable; ending class objects of its own state named; a lambda parameter matches the type; `captured` and "declared or imported in the file"; new bullet `thread_shared fn()`; `LOG.push(copy of s.name)`; `add_point` pushes to `LOG`, so the callback's `changes Game, LOG` is true; real variables only borrowed, the box gone; new bullet `with owns` for a function that outlives its maker; a C callback ending its own token | 55, 61, 62, 71, 75, 76, 80; worklist section 4 items 13, 15 |
| R13 | R13 | `exists as copy of v` may change, the two-line form; what R10 does for an opened name | 66, 67, 82 |
| R14 | R14 | unchanged (its stop text is in section 2) | — |
| R15 | R15 | a scoped task in a walk of what it changes is E3020; reductions on a field; the loop's own variable with `with copy of`; the thread check in every profile; `thread_shared fn()`; a callback ending its token; the trampoline's stops | 61, 62, 65, 73, 77 |
| R16 | R16 | the `unsafe` principle | 60 |
| R17 | R17 | the view-parameter fact recorded in the interface file, a reordering reported | 53 |
| R18 | R18 | a fix never runs into another error; the tie-breaker, with its cases | 73 |
| section 2 | section 2 | count rows with the compare; R10's row split in two; the walk's row carries run 3's region number; the real-variable row without the box; the own-state count row covers the C callback; the trampoline row's thread check; the "nothing" row lists the new compile-time checks; **new table of stop texts**; "Gone" lists the region and count pair under views, and the box | 59, 61, 62, 80, 82; worklist section 3 |
| section 3 | section 3 | rows added: E2105 for a change through a link and for a `drop`'s changes; E2122; E0118; E3031; E0120; E2119; E2123; and today's codes the tests use, E0100 (a module-level statement, `global`), E2030, E2040, E2050, with the messages the tests and answers give (`R15/reject_python_global.em`, `R7/reject_view_on_a_type_that_holds_no_view.em`, `R17/reject_a_pub_generic_without_its_bounds.em`, `R14/reject_a_generic_method_that_returns_its_type_parameter_through_dyn.em`), so that every code of section 4 that a program can meet has its row; messages added: E3040 (part), E3028 (`const`), E3022's run-time stop (the lead's text), E3020 (map keys, `drop`, scoped task), E3060 (`remove_all`, `reset()`), E2020 (`fn(owns s)`), E3026 (`make_counter`), E3013 (class field), E7018 (walk under the lock), E7015 (`thread_shared fn()`), E3100 (two more), the removed words (`mem.forget`, `as?`, `fn drop():`); messages corrected (section 8) | 53-82; worklist sections 2, 4 |
| section 4 | section 4 | the note rewritten; every code `codes.rs` does not define in bold; six new codes; E0100, E2030, E2040, E2050 added with today's meanings; E1030 kept to one meaning; meanings widened (section 6); "Not used: E3041" | worklist section 2 |

## 4. Word changes

| rows of `V4/vocabulary.md` | change | caused by |
|---|---|---|
| title, opening, section 0, section 2 | step 3, v4; verdicts 1 to 82; `V4` paths; "this step ran no probe"; view (`exists as copy of` is not one; the call check); `Copy` (has `Clone`); class object (ended before `drop`); section 2 points to `V3/changes.md` sections 4 and 10a and this file | 53, 56, 67, 68 |
| `const` | the call writes `const ref mut`, E3028's message | 70 |
| `fn` | a captured `Atomic` by its variable; ending a class object of its own state named; `thread_shared fn()`; real variables only borrowed (no box); `with owns` for a function that outlives its maker; a lambda parameter matches the type; E0120 for `with ref mut x`; E3026, E2105, E2020, E7015 messages | 55, 61, 71, 75, 80; worklist section 2 |
| `for` | every collection, refused at the change; `drop`; the map-keys message | 57, 72 |
| `if`, `is` | E2036's and E2106's messages corrected | worklist section 4 items 1, 7 |
| `mut self` | a `drop` writes `fn drop():` | 69 |
| `this` | checked at the call like a view parameter (E3031); `drop` says neither; E2122 | 54, 69 |
| `with` | a name not listed is only borrowed; `with owns` for a function that outlives its maker; E0120; E3026 | 80; worklist section 2 |
| `unsafe` | the principle; E3100's two messages | 60; worklist section 4 item 9 |
| `@must_drop` | "no `forget`" goes | 58 |
| `@borrows` | its refusal is E0104 (section 4's E0104 lists it) | completeness check, item 10 |
| `Shared` | the count's compare stays | 59 |
| `Retained`, `ForeignCallback` | the one way to hand a value to C; a callback ending its own token | 58, 62 |
| `Copy`, `Clone` | `@derive(Copy)` gives `Clone`; E2116's message gives "or keep a view of it" | 68; worklist section 4 item 2 |
| `drop` | rewritten: names only what it changes outside its object; ending an object is a call of it; ended before it runs; E2122, E3020, E2105 messages | 56, 57, 69 |
| `mem.forget` | **removed**, with E1010's message and today's uses in std, tests and the compiler | 58 |
| `Atomic` | named by its variable where visible, by type through a link | 75 |
| `as?`, `cast` | the whole lines AN-67's program needs; the failed cast's stop text | worklist section 4 item 8; section 3 |
| `Scope`, `thread.spawn` | field reductions; a task started in a walk (E3020); a read-then-change (E7007); the loop variable `with copy of` | 65, 73, 77 |
| `copy of` | `exists as copy of v` may change, the two-line form; links copied as links; `Copy` types; E2116's messages | 66, 67, 68, 79; worklist section 4 item 2 |
| `shares` | the count's check; a change through it named | 59, 63, 64 |
| `sharedref` | a push keeps a link to a class object; the word on the making line | 81 |
| `changes` | links whatever holds them; class objects of own state; `drop`s; on a `drop` only what is outside; the body's view-parameter fact (E3031); E2105, E2122, E0118 messages | 53, 54, 55, 57, 63, 64, 69 |
| `captured` | "declared or imported in the file"; E2113's whole lines | 76; worklist section 4 item 5 |
| a change list, the iterator row | `LOG.push(copy of s.name)` | worklist section 4 item 13 |
| `exists as` | the copy may change; the two-line form; an opened name not used after a call that may end its object (E3060) | 66, 67, 82 |
| `owns`, `using` | required on the making line of an `owns` field (E2123); verdict 47 cited on `using` | 74 |
| `thread_shared` | the count's check; `thread_shared fn()`; E7015's message | 59, 61 |
| `weakref` | the weak count's check; ended before `drop`; R10's check for an opened name; E2119 | 56, 59, 82; worklist section 2 |
| `ref mut` | `const` at the call; E3031 for a `ref mut` argument | 53, 70 |
| section 4 (placeholders) | `thread_shared fn()` added | 61 |
| section 5 | no word left open by verdicts 53 to 82 | — |
| section 6.1 (`mut self`), 6.2 (`[DRP-1]`, `[OWN-6]`, new `[THR-6]` row, `[RC-*]`), 6.1 (`Shared`'s cost note) | `fn drop():`; ended before `drop`; `mem.forget` removed from `[OWN-6]`, `[THR-6]` 3534 and `std.mem`'s list 4169; the counts' compare | 56, 57, 58, 59, 69 |

Every other row is v3's text.

## 5. The safety validation's findings, and how each is settled

| finding (`V4/safety/findings.md`) | settled |
|---|---|
| section 1, hole 1: a view given to a function that clears what it points into | verdict 53: R4's call check (E3031) |
| hole 2: a method that destroys its own object | verdict 54: the same check on `this` |
| hole 3: a stored function that owns what a view reads | verdict 55: ending a class object is named in the function's type, so R10 sees the call; R10 point 1 |
| hole 4: `drop` and `weakref` | verdicts 56, 57: ended before `drop` (R11); ending is a call of `drop` (R5, R8 (c)) |
| hole 5: count overflow | verdicts 58, 59: `mem.forget` removed (R1); the compare before each +1 (R9, R11) |
| hole 6: threads and C | verdict 61 (`thread_shared fn()`, R12, R15), verdict 62 (the callback's block freed after it returns, R12) |
| section 2: `changes T` through a link held by a `ref mut` parameter or own local | verdicts 63, 64 (R5) |
| one mistake, two codes (E3020/E7018, E7007/E3020, E3060/E3022) | verdict 73 (R18, section 2.9 above) |
| R8 (b) against R10 | verdict 82 (R10 rewritten) |
| `copy of` a struct holding `shares` or `weakref` fields | verdict 79 (R3) |
| verdict 52's line without a capture word | verdict 65 (R15) |
| the smaller ones: `exists as copy of v`; `Copy` and `Clone`; a partial hand-over; maps, sets, strings; a push and a link in its own block; `stats.total += e`; `owns` on a making line; a call to `const ref mut`; a function value that outlives its maker; a sibling module and `captured`; a captured `Atomic`; a plain lambda parameter for an `owns` type; `drop` and `changes this` | verdicts 66/67, 68, 78, 72, 81, 77, 74, 70, 80, 76, 75, 71, 69 |
| section 3: codes used but not in section 4; E1030's second meaning; no code for a partial hand-over, `weakref` of a part, a first line in the wrong order, `with ref mut x`; the "bold = new" note | section 4 (section 6 below): E0100, E2030, E2040, E2050 added; E1030 one meaning (a module-level statement is E0100); E3040 for a part; E2119; E0118; E0120; the note rewritten |
| section 3: run-time stops with no text | section 2's new table (the lead's texts) |
| section 3: messages whose line does not compile | section 8 below |
| section 3: test runner limits (R17's package edge) | not the rule writer's; left to the test updaters |
| `completeness.md` item 10: E0104's list does not name `@borrows` | added to E0104's meaning and the `@borrows` row |
| `completeness.md` item 12: a list of views given to an unscoped thread (E7015 or E3063) | already decided by R15's text ("a lent value reaching another thread is E7015"); no change |
| `check-V6.md` item 10: a plain `thread_shared Log` argument beside a `fn() changes Log` argument | not settled by a verdict: section 11, item 5 |

## 6. Error-code changes

| code | change | reason |
|---|---|---|
| **E0118** | **new**: a first line whose parts are out of order | worklist section 2; R5 |
| **E0120** | **new**: `ref mut` in a lambda's capture list | worklist section 2; R3 (the vocabulary had the message with no code) |
| **E2119** | **new**: a `weakref` of a part | worklist section 2; R11 (the vocabulary had the message with no code) |
| **E2122** | **new**: `changes this` on a `drop` | 69 |
| **E2123** | **new**: a value made for an `owns` field without `owns` | 74 |
| **E3031** | **new**: a call that hands a function a part of what it changes, where its body uses that parameter after the change | 53, 54 |
| E0100 | **added** with today's meaning (unexpected token), for `global` and a module-level statement such as `FRAME = 0` (the message gives `static mut FRAME: int = 0`; AN-76's E1030 for it becomes E0100) | worklist section 2 |
| E2030, E2040, E2050 | **added** with today's meanings (`@view` on a non-view; an unsatisfied bound, a `pub` generic's unwritten bounds; a non-`dyn`-compatible interface) | worklist section 2 |
| E1030 | **kept to one meaning**, two items with one name; "nothing else" written | worklist section 2 |
| E0104 | **widened**: `@borrows` | completeness item 10 |
| E1010 | **widened**: `mem.forget` | 58 |
| E2020 | **widened**: a plain lambda parameter where the type hands the value over | 71 |
| E2105 | **widened**: a change through a link not named; the end of a class object by a function's own state; a `drop`'s changes | 55, 57, 63, 64 |
| E3013 | **widened**: a field of a class object | 78 |
| E3020 | **widened**: maps, sets, strings; a scoped task started in the walk; a `drop`'s changes. **Narrowed** by verdict 73: a call through a function value that may take a held lock, in a walk under it, is E7018 | 57, 72, 73 |
| E3023 | **narrowed**: a name opened `exists as copy of` may be changed | 67 |
| E3026 | **widened** in its text: a returned function that changes its maker's local without `with owns` | 80 |
| E3028 | **widened**: no `const` before an argument for a `const ref mut` parameter | 70 |
| E3040 | **widened**: a part handed over, or its owner used whole, before the part has a new value | 78 |
| E3060 | **widened**: a name opened from a link after a line or call that may end its object | 82 |
| E3100 | **widened**: `Send` declared by hand without `unsafe extend` (today's second E3100 text) | worklist section 4 item 9 |
| E7007 | **narrowed**: a field reduction from scoped tasks is not E7007; a task that reads the place first is | 77 |
| E7015 | **widened**: a one-thread value held by a `thread_shared fn()` | 61 |
| E7018 | **widened**: a call through a function value that may take a held lock, inside a walk under it | 73 |
| E3041 | **not used**: a hand-over in a loop's later round is E3040 (a line under section 4's table) | worklist section 2 |

**The note at the top of section 4**, corrected: v3 said "a code in bold is the model's own: it is not in
today's compiler", and bolded only E2116, E2117, E2118, E5069 and E7018. Every code of the table was
looked up in `compiler/ember_diag/src/codes.rs` (`target/redesign6/01/free_codes.txt`): 36 codes of the
table are not defined there, and all are now bold: E0113 to E0118, E0120, E2085, E2105 to E2108, E2112 to
E2119, E2122, E2123, E3028, E3031, E5013, E5069, E7007, E7008, E7012 to E7018 and W3013. The worklist's
list (E0113-E0117, E2085, E2105-E2108, E2112-E2115, E3028, E7017 and the new ones) missed E5013, E7007,
E7008, E7012 to E7016 and W3013, which a search of every `.rs` file under `compiler/` also does not find.
The six new codes appear nowhere in the repository outside `target/` but the lead's worklist. No code is
retired in this step. (Not changed, for the record: today's `codes.rs` gives E7004 the meaning "value
crossing a thread boundary is not `Send`", near the model's "a name with no capture word"; no row was
asked to change it.)

## 7. Stop texts (rules section 2)

Section 2 has a new table, "The checks above that stop the program say this when they do", with the
lead's texts word for word (worklist section 3): the index compare, `assert_disjoint`, the two count
compares, the failed `cast s as Circle`, the trampoline's two stops (after the token ended; a
`threads(creator)` callback on another thread, checked in every profile), and today's bounds text. Section
3's E3022 row, which gave v3's own stop text for the index compare ("`collide` was given `bodies[3]` as
both `a` and `b`; ..."), now gives the lead's, so one check has one text. R14 and the vocabulary's `cast`
row point to the cast's text; R15 to the trampoline's.

## 8. Messages corrected (worklist section 4), where each stands

| item | message | done in |
|---|---|---|
| 1 | E2036: no `if v == n:` | rules section 3, vocabulary `if`; the worklist's "write `v = n` on the line before" left the refused `if` line in place, so the line given is "write `v = n` in place of the `if` line, and use `v` in its block, moved out one level"; the kind test's E2036 the same way (vocabulary `is`) |
| 2 | E2116: `copy of` offered only where the type has `Clone` | rules section 3 (the `weakref` message says when its second way out is offered and what is said otherwise; the `Player` message: "write `@derive(Clone)` above `class Player`, or keep a view of it: `p2 = ref p`"), vocabulary `copy of`, `Clone` |
| 3 | E2118: `let buf: String` | rules section 3 |
| 4 | E3026: every line it replaces, the `Counter` class, the list type, `main`'s `changes Counter` | rules section 3 (`class Counter:` with `n: int = 0`; `main`'s three lines and its first line; `register`'s two lines; `Button`'s three lines) |
| 5 | E2113: every line, `click`'s included | rules section 3 (three messages: `fn() changes LOG` into `Array[fn()]`; into `Array[fn() changes captured]`, giving `fn click() changes handlers, captured, LOG:`; two captured variables), vocabulary `captured` |
| 6 | AN-59's third message | an answer's (area D); the rules' own AN-59 message (E3013) is corrected: "hand over a copy each time instead: `call_twice(fn(): consume(copy of s))`", since its v3 first way out (`owns f: once fn()` in a `call_twice` that calls twice) was refused itself |
| 7 | E2106: no `s.r` left in the block | rules section 3 ("and `c.r` for `s.r` in its block"; the opening's message "and `t` for `target`"), vocabulary `if`, `is` |
| 8 | AN-67's `as?` message | rules section 3's removed-words examples and vocabulary `as?`, `cast`: the lines replaced are named (`a = s as? Circle` and `if a exists as c:`; `found = cast s if Circle` and `if found exists as c:`) |
| 9 | E3100: the exact line | rules section 3 (the `unsafe fn` body message in the worklist's words; two added: a call of an `unsafe fn`, and `unsafe extend Counter implements Send:`), vocabulary `unsafe` |
| 10 | the generator's E3020 | rules section 3: "push after `g`'s last use: move `d.log.push("open")` below line 4" (the rules' example has its last use on line 4, not the worklist's line 9) |
| 11 | L3015: no made-up promise | rules section 3's warnings: "`read_u32_le` is an `unsafe fn` that reads through the raw pointer `p`, and nothing says what its callers must make true for that; say it above the function: `@safety("...")`, with what `p` must point to". The worklist's text ("add `# SAFETY:` above the read") was not used: it is W3012's fix for an `unsafe:` block, and on an `unsafe fn` it would run into W3013 or leave L3015 (R16), against verdict 73 |
| 12 | E5013's wording | the message is area E's (AN-84); the rules have only E5013's meaning, unchanged |
| 13 | `LOG.push(copy of s.name)` | R12, rules section 3 (E2105, E3020), vocabulary (change list, iterator row) |
| 14 | E7012, E7007, E7010: the import | rules section 3 (`from std.sync import Lock` and, for `Atomic`, `from std.sync import Atomic`); also AN-78's E7015 message, which gives an `Atomic` field in a program with no import (area E's AN-78 has none) |
| 15 | `register_cb(... changes Game, LOG ...)` | R12: kept as verdict 39 writes it, with `add_point` now also writing a line to the log (`fn add_point() changes score, LOG:`), so the callback's `changes Game, LOG` is true |
| 16 | "F-01 to F-15 of `check-V*.md`" | the check files number no item "F-"; this step took the checkers' findings on messages that stand in the rules or the vocabulary: check-V1's U4 (item 1), check-V6's U7 (item 14), its item 8 (E2106, AN-67, and E2072, whose message now also says `struct Crate implements Weighed:`), its item 7 (L3015) and item 9 (E3100), check-V3's list of source messages (the generator's and the counting loop's E3020: "instead of `if enemies[i].hp <= 0:` (line 3) and line 4, write `enemies.retain(fn(e) => e.hp > 0)` after the loop (and remove the `for` line if nothing is left in it)"), check-V5's E2113 and E3026 (items 4, 5). Left as they are: row 4's message (the owner agreed it word for word), AN-24's and AN-52's (the areas' messages; the rules' AN-52 E3063 row follows area C) |

## 9. What this step does not change

- Today's outputs, OLD code and "means exactly" programs are not in these two files.
- No number in section 2 was measured again; three costs are marked "not measured" (the counts'
  compare, the thread check in the release profile, the callback's block held to its return).
- No answer and no test is edited: they belong to agents 02 to 12.

## 10. Answers and tests the changes touch (for agents 02 to 12)

A pointer, not a ruling. The worklist's "answers" column names the likely ones; a crude scan of the area
files (`target/redesign6/01/touched_v4.py`, output `touched_v4.txt`; prose is searched too) adds:

| verdict | answers the scan finds |
|---|---|
| 53, 54 | AN-04, AN-05, AN-16, AN-53, AN-70, AN-76 |
| 55 | AN-22, AN-29, AN-30, AN-51, AN-53, AN-62, AN-77 (a `clear()` or a `with owns` list) |
| 63, 64 | AN-20, AN-48, AN-70, and every answer that changes an object through a `shares mut` local or `ref mut` parameter |
| 68 | AN-01, AN-06, AN-10, AN-13, AN-15, AN-44 |
| 70 | AN-11, AN-16, AN-28, AN-33 |
| 74 | AN-39, AN-40, AN-52 (and AN-24, by the worklist) |
| 75 | AN-71, AN-73, AN-74, AN-78, AN-80, AN-84 |
| 76 | AN-53, AN-54, AN-57, AN-60, AN-62, AN-70 |
| 81 | AN-52 |
| 82 | AN-51 (R10 points 2-3), AN-62, AN-63, AN-65 |
| 36's example (2.6) | AN-30's `main` |

Two consequences the area and test updaters will meet: R10 no longer keeps an object alive under a view
across a call that may end it, so an answer or test that relied on R10's region or count pair for such a
call (W4's `R10` tests) now expects E3060 or a reordered program; and E7007's section-3 example is now a
task that reads the total first, since verdict 77 makes AN-74's `stats.total += 1` from scoped tasks a
reduction.

## 11. Files

- `tasks/96/redesign/run4/v4/rules.md`, `vocabulary.md` (edited in place), `changes.md` (this file, new).
- Under `target/redesign6/01/`: `rules_v3.md`, `vocabulary_v3.md`, `changes_v3.md` (the files as this step
  found them); `free_codes.txt` (every section-4 code looked up in `codes.rs`, and the search for the new
  codes); `section4_table.md` (the new table of section 4, spliced in by a script); `touched_v4.py`,
  `touched_v4.txt` (section 10).

## 12. For the owner

What verdicts 53 to 82 leave open, one at a time, each with what this step does meanwhile.

1. **How does a one-owner foreign value give its object to C, now that `mem.forget` is gone?** Today's
   `ForeignBox.into_raw` (`std/src/ffi.em:177-181`) releases its record and calls `mem.forget(self)` so
   that C owns the object and the box frees nothing; the `owns T` that replaces `ForeignBox[T]` needs the
   same for a parameter contracted `owned` and for `std::unique_ptr` parameters (`[FFI-11]`, `[FFI-17a]`).
   Verdict 58 says handing an object to C goes through the `Retained` token, which keeps the object on the
   Ember side; giving it away is a different case. Meanwhile the rules have no way to do it. Should it be
   an `unsafe` library operation (verdict 60), or something else?
2. **Verdict 53 for a `ref mut` parameter.** The verdict says "a view parameter"; the rules' section 0
   calls a `ref mut` parameter a view, and `f(ref mut LOG[0])` into a `fn f(ref mut s: String) changes
   LOG:` that clears `LOG` and then writes `s` is the same freed-memory hole (it also breaks the noalias
   promise R4 compiles every `ref mut` parameter with). This step applies the call check to plain, `Span`
   and `ref mut` parameters (R4). Is that the intended reach?
3. **A callee's change named by a type counts for its caller, even on the caller's own local** (R5).
   Verdict 36's own example writes `fn() with owns tx changes Sender[int]: produce(tx)` for a function
   whose `tx` is its own state, so this step wrote that as the rule; it makes AN-30's `main`, which gives
   its own `Mutex` to a function that changes it, say `changes Mutex[Array[String]]` (AN-30 says it names
   nothing, as an own local). Is that right?
4. **`copy of` a struct holding a `sharedref mut` field.** Verdict 79 copies a struct's links; the rules'
   "link" includes `sharedref mut`, so this step copies such a field as one more holder of the same
   changeable link (R3). A struct holding a `ref mut` field (not a link, and alone by R8 (f)) is a view
   type, which E2116 refuses. Both as intended?
5. **A plain `thread_shared Log` argument beside a `fn() changes Log` argument of one call**
   (`check-V6.md` item 10). R4's overlap rule counts a function value as changing every place its type
   names, and verdict 44 makes a guarded field a `Mutex`'s value, which takes no word; R4 does not say
   whether the two arguments overlap (E3021) or whether the lock makes it allowed. No verdict settles it;
   the rules are unchanged.
6. **The walk's compile-time saving** (verdict 28, still open from v3): R11 places a walk across a
   deleting call in a deferred-drop region where the node class has no `drop`, which removes the
   weak-count moves and leaves one header test per step; nothing is measured.
