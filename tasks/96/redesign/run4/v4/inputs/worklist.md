# Worklist for run 4, step 3 (written by the lead; every agent reads the parts for its duty)

`V4` = `tasks/96/redesign/run4/v4/`. Verdict numbers are those of `inputs/JOB.md`, section "Owner verdicts on
run 4's open questions". Every refusal message follows R18 and verdict 73: it names what was refused in the
program's own names, says why in one line, gives the exact line(s) to write, and its fix must not run into
another error; where two errors fit one line, the one whose fix makes the program compile is shown.

## 1. Verdicts 53 to 82: what each changes, and where

| verdict | the change | rules | vocabulary | answers | tests (group) |
|---|---|---|---|---|---|
| 53 | a fact worked out from a body: "uses view parameter `p` after changing X" (through its own lines and the same fact of its callees; an unseen call counts as doing so), kept with the function (R17 interface file), checked at each call: a call handing a part of X to such a parameter is refused, **E3031** | R4, R8 (b) (g), R17, section 3, section 4 | `changes` row | AN-01, AN-02 and any answer passing a part of a changed module variable to a function | W2 (R4, R17), W3 (R8) |
| 54 | the same for a method's object: `this` counts as a view parameter; `ENEMIES[0].hit()` with `hit` clearing `ENEMIES` then using `this` is refused, **E3031** | R5, R8 | `this` row | answers with methods changing what owns their object | W2, W3 |
| 55 | a function that may destroy a class object it owns (its own state included) names the object's type: `fn() with owns players changes Player: players.clear()`; R12's "own state needs no word" excludes ending class objects | R12, R5 | `fn`, `changes` rows | answers with `with owns` of lists of class objects | W5 |
| 56 | an object counts as ended the moment its last owner lets it go, before its `drop` runs; a `weakref` opened in its `drop` or in the `drop` of anything it owns finds it ended; ending the link itself (`this.parent = None`) only subtracts the weak count | R11, section 2 | `weakref`, `drop` rows | AN-62 and the tree answers | W4 |
| 57 | ending an object counts as calling its `drop`, with what `drop`'s line says it changes outside the object; `enemies.clear()` inside `for line in LOG:` with a `drop` that pushes to `LOG` is E3020; a function that may end such an object says `changes LOG` | R5, R8 (c) | `drop` row | answers with `drop` methods | W1, W3 |
| 58 | `mem.forget` removed: E1010 "`mem.forget` is gone: a value always ends with its clean-up; to hand a value to C, use the `Retained` token" | R1, section 3 | remove `mem.forget` row; `@must_drop`'s "no `forget`" | any use | W1 |
| 59 | before each +1 of an owner count, compare with the maximum; the program stops with the text in section 3 below; the weak count the same | R9, R11, section 2 | `shares`, `weakref` rows | — | W4 |
| 60 | anything that can break a safety promise is possible only inside `unsafe` (principle; the inventory comes later) | R16 (one sentence) | `unsafe` row | — | — |
| 61 | `thread_shared fn()` in function types: a function any thread may run; it may hold `thread_shared` values or copies, never a `shares`/`shares mut` value or a one-thread `weakref` (E7015); a `thread_shared` object's fields that hold functions use such types | R12, R15 | `fn`, `thread_shared` rows | AN-75, AN-80, AN-76 handler lists reached by threads | W5, W6 |
| 62 | a C callback that destroys its own token: C forgets it at once, its block (the captured variables) is freed only after the callback returns | R12, R15, section 2 | `Retained`, `ForeignCallback` rows | AN-78 | W6 |
| 63, 64 | a change through a link is named `changes T` whatever holds the link: a `ref mut` parameter (`fn tick(ref mut c: shares mut Clock) changes Clock:`) or the function's own local (`fn main() changes Clock:` for `clock = shares mut Clock()`); R5's "needs no clause" is narrowed to a `ref mut` parameter or own local that is not a link | R5 | `changes` row | every answer changing an object through a `shares`/`shares mut`/`thread_shared`/`weakref` link held in a parameter or local | W2 |
| 65 | verdict 52's line is `thread.spawn(fn() with copy of r changes RESULTS: worker(r))`; the loop's own variable needs `with copy of` for scoped tasks too | R15 | — | AN-76 | W6 |
| 66, 67 | `exists as copy of v` stays and `exists as v` + `mine = copy of v` is valid too; the `v` of `exists as copy of v` may be changed; a name made by `exists as` without `copy of` never changes | R2, R13 | `exists as`, `copy of` rows | — | W1 |
| 68 | a `Copy` type always works with `copy of` (`@derive(Copy)` gives `Clone`) | R1, R3 | `Copy`, `copy of` rows | — | W1 |
| 69 | `drop` never writes `changes this`; its line names only what it changes outside `this`; writing `changes this` on a `drop` is refused, **E2122** | R5 | `drop` row | answers with `drop` | W2 |
| 70 | a call to a `const ref mut` parameter writes `const ref mut`: `blend(const ref mut d, s)`; a call without `const` there, E3028 | R4 | `const` row | — | W2 |
| 71 | a lambda's parameter says what it receives: `fn(owns s) => ...` where the type is `fn(owns String)`; `fn(s)` there is E2020 | R12 | `fn` row | AN-56 (its second call) | W5 |
| 72 | the loop rule covers lists, maps, sets and strings: E3020 at the change, the list's message | R8 (c) | `for` row | — | W3 |
| 73 | a message's fix must not run into another error; two errors on one line: the one whose fix compiles is shown (E7018 over E3020 for a lock-taking call in a walk under the lock; for E7007 against E3020 and E3060 against E3022, the same test) | R18 | — | every message | all |
| 74 | `owns` is written on the line that makes a value for an `owns` field: `mesh=owns Mesh(points=[0.0, 1.0]) using current`; without it, **E2123**; with or without an arena (AN-40's `Entity(mesh=owns Mesh(...))`); R3's "no `owns` before a value its place hands over" is narrowed | R3 | `owns`, `using` rows | AN-40, AN-24 (verdict 24's), every making line of an `owns` field | W1 |
| 75 | a captured `Atomic` is named by the variable (`changes hits`); by type only when reached through a link | R12 | `Atomic` row | — | W5 |
| 76 | "can see by name" for `captured` = declared or imported in that file | R12 | `captured` row | — | W5 |
| 77 | `stats.total += x` from several scoped tasks is a reduction (a private sum per task, added at the end); reading the value in a task, or a `thread.spawn` thread, still needs an `Atomic` or a lock (E7007) | R15 | — | AN-74 | W6 |
| 78 | a field may be handed over on its own: `names.push(p.name)` moves the name; `p.hp` stays usable; `p.name` and `p` whole are refused until it gets a new value, E3040 (text below); a class object's field cannot be taken out (`mem.replace`/`mem.take`) | R1, R2 | — | — | W1 |
| 79 | `copy of` a struct holding links copies the links (`shares`: one more owner of the same object; `weakref`: one more link); fields the struct owns alone are copied | R3 | `copy of` row | — | W1, W4 |
| 80 | a function that outlives its maker takes what it keeps with `with owns`: `return fn() with owns n:` and `fn make_counter() -> fn():`; without it E3026 | R12 | `with` row | AN-53 part 5 case 2 | W5 |
| 81 | adding to a list does not end a link to an object in its own block (a class object): `w.target.hp -= 1` after `players.push(Player())` is allowed; a link to a value stored inline in the list is still ended | R8 (b), section 0 | `sharedref` row | AN-52 | W3 |
| 82 | a name opened from a link is not used across a call that may destroy what it reads: E3060; R10's keep-alive (region, count pair) is no longer used for such a call; it remains for a `thread_shared` object reached through a `weakref` | R10 | — | answers relying on R10 points 2-3 | W4 |

## 2. Error codes (section 4)

New, checked unused with `grep -rlw` over `compiler/`, `docs/spec-source/`, `std/`, `tests/` and
`tasks/96/redesign/` (no file): **E0118**, **E0120**, **E2119**, **E2122**, **E2123**, **E3031**.

| code | meaning | message (for the example the duty gives) |
|---|---|---|
| **E0118** | a first line whose parts are out of order (R5's order: parameters, `reads`, `changes` or `owns this`, `-> T`, `from`) | "the parts of `note`'s first line are out of order; write `fn note(msg: String) reads frame changes log:`" |
| **E0120** | `with ref mut x` in a capture list | "remove `ref mut x` from the capture list: without a word the function already uses the real `x`; it says it changes it with `changes x`: `fn() changes x: x += 1`" |
| **E2119** | `weakref` of a part (a field, an element stored inline) | "a `weakref` link needs an object that lives on its own; link to the player: `cam.target = weakref player`" |
| **E2122** | `changes this` on a `drop` (verdict 69) | "remove `changes this`: a `drop` always changes its own object; name only what it changes outside it: `fn drop() changes LOG:`" |
| **E2123** | a value made for an `owns` field without `owns` (verdict 74) | "`mesh` owns its mesh; say so where it is made: `mesh=owns Mesh(points=[0.0, 1.0]) using current`" |
| **E3031** | a call hands a part of X to a function (or a method's object) whose body uses that parameter after changing X (verdicts 53, 54) | "`note` changes `LOG` and uses `s` after `LOG.clear()` (its line 4), and `LOG[0]` is part of `LOG`; give it a copy: `note(copy of LOG[0])`, or in `note` use `s` before `LOG.clear()`"; for a method: "`hit` clears `ENEMIES` and then changes `this.hp`, and `ENEMIES[0]` is part of `ENEMIES`; in `hit`, change `this.hp` before `ENEMIES.clear()`" |

Today's codes used by the tests, added to section 4 with today's meaning: **E0100** (unexpected token;
`global`, and a statement at module level such as `FRAME = 0`, whose message gives `static mut FRAME: int
= 0`; AN-76's use of E1030 for it becomes E0100), **E2030** (`@view` on a type that is not a view),
**E2040** (unsatisfied interface bound: a `pub` generic whose bounds are not written), **E2050** (an
interface that is not `dyn`-compatible). **E1030** keeps one meaning, two items with one name. E3041
(a hand-over in a loop's later round) is not used: it is E3040. The note at the top of section 4 ("a code
in bold is the model's own") is corrected: list each code that today's `codes.rs` does not define, bold or
not (E0113-E0117, E2085, E2105-E2108, E2112-E2115, E3028, E7017 and the new ones).

## 3. Stop texts (section 2), written by the lead

| stop | text |
|---|---|
| `assert_disjoint` finds an overlap | "`assert_disjoint(d, s)` on line 11: the two spans overlap (`dst_name` and `src_name` name one track)" |
| `cast s as Circle` and `s` is not one | "`cast s as Circle` on line 9: `s` is a `Square`, not a `Circle`" |
| C calls a callback after its token ended | "C called the callback registered on line 16 after its token ended; it was unregistered" |
| C calls a `threads(creator)` callback on another thread | "C called the callback registered on line 16 on another thread, but its contract says `threads(creator)`" (checked in every profile) |
| two `ref mut` names of one list's elements are one element (R8 (f), compared once) | "`a` and `b` name the same element of `bodies` (index 3); `collide` needs two different bodies" |
| an owner count would roll over (verdict 59) | "too many owners of a `Player` (4,294,967,295)" |
| a weak count would roll over | "too many `weakref` links to a `Player` (4,294,967,295)" |

## 4. Messages to correct (from `V4/safety/findings.md` section 3 and the checkers)

Each is corrected where it stands (rules section 3, vocabulary rows, answers' part 6, tests), to the
message rule and verdict 73:

1. E2036 (rules section 3 and vocabulary `if` row): drop "to compare write `if v == n:`" (it uses an
   undeclared `v`); write: "`n` is an `int`, which always holds a value, so this `if` would always run;
   to name it write `v = n` on the line before and use `v` in the block".
2. E2116: offer `copy of` only where the type has `Clone` (every `Copy` type does, verdict 68); otherwise
   "`Document` has no `Clone`; write `@derive(Clone)` on it, or keep a view of it".
3. E2118 (fixed array): keep `let`: "write the field as `let buf: String`".
4. E3026 (rules row, AN-53, AN-57): every line it replaces, the `Counter` class with its field and the
   button's list type, and (after verdict 64) `main`'s `changes Counter`.
5. E2113 (AN-53 part 6, AN-60, rules row): give every line, including `click`'s new first line
   (`fn click() changes handlers, captured, LOG:`).
6. AN-59's third message: its first way out is refused itself; give a way out that compiles.
7. E2106 (rules section 3): the line given must not leave `s.r` in the block.
8. AN-67's message for `as?`: give the whole `if s is Circle as c:` line and its block.
9. E3100 (AN-79, R16): give the exact line, e.g. "put line 4 in an `unsafe:` block with a `# SAFETY:`
   note saying why the read is safe"; "write `unsafe extend Counter implements Send:`".
10. The generator's E3020 (rules section 3): give the line: "call `d.log.push("open")` after `g`'s last
    use, line 9".
11. L3015: drop the made-up promise; say "`p` is a raw pointer read without a note; say why it is safe:
    add `# SAFETY:` above the read".
12. E5013: say what C does with the argument, not what it reads: "C passes the extra argument `narrow` as
    an `int`; `sum_wide` reads an `int64_t`; pass it as one: `narrow as i64`".
13. R12's own example `scores.for_each(fn(s) changes LOG: LOG.push(s.name))` (and rules section 3 lines
    on it): `fn(s)` only reads `s`, so `LOG.push(s.name)` hands over a part of a read-only name; write
    `LOG.push(copy of s.name)`.
14. E7012, E7007, E7010 (rules section 3): add the import line their fixes need
    (`from std.sync import Lock, Atomic`).
15. The rules' `register_cb(fn() with owns game changes Game, LOG: ...)` (R12): it names `LOG` but never
    changes it (E2105 both ways): name only `Game`, unless the body pushes to `LOG`.
16. F-01 to F-15 of `V4/safety/check-V*.md` that are messages: as each checker's fix says.
