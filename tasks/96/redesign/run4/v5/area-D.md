# Area D, v5: linked objects, callbacks, interface types (AN-53 .. AN-70)

Agent 03 (answers C and D), run 4, step 4 (v5). Every answer of area D brought to the v5 rules and words,
which apply all of the owner's verdicts 1 to 97 (`inputs/JOB.md`; 27 replaces 25, 26 replaces 1, 95
reverses 90, and 86 removes `changes` and `reads` and voids the verdicts that only said how to write
them): `V5/rules.md` (rules R1 to R18, the codes), `V5/vocabulary.md` (the words), `V5/changes.md` (what
changed from v4: verdicts 83 to 97 and the corrected records of verdicts 53 and 58) and
`V5/inputs/worklist.md` (what each new verdict changes, and how every point step 3 left open is
settled). `RUN3` means `tasks/96/redesign/run3/`, `RUN4` means `tasks/96/redesign/run4/`, `V2` .. `V5`
mean `tasks/96/redesign/run4/v2/` .. `v5/`. "R1" .. "R18" are the v5 rules; "row N" is the owner's
agreed row N and "verdict N" the owner's verdict N in `inputs/JOB.md`; G-01 .. G-11 are the problems of
`RUN3/22-re-attacker.md`.

No compiler change, no git command, no Ember program run, and no new `ember check`: this file makes no
new claim about what today's Ember accepts or refuses. This file is `V4/area-D.md` (copied into `V5`)
after the lead's spelling script (`V5/tools/step4_spell_report.md`, section 3), edited in place: every
OLD block and every "means exactly" block in today's Ember or C, with its pasted output, is v4's byte for
byte (checked by comparing every fenced block with the copy taken at the start,
`target/redesign7/03/check_blocks.py`, output in `target/redesign7/03/check_blocks.txt`); only the
prose, the NEW blocks (AN-53's part 5 block among them, a NEW program in the new spelling), the
owner's-test tables, the refusals and the costs changed. The C files and outputs v2, v3 and run 3 cite
are cited as they were; nothing was built or run in this step. What changed from v4 is said once per
answer, in its "v5" line, and in the table below; what changed before that is in `V4/area-D.md`,
`V3/area-D.md` and `V2/area-D.md`.

Each answer has the eight parts: 1 intent; 2 OLD with today's output; 3 NEW; 4 the owner's test for
every declared name; 5 means exactly (today's Ember or C doing what NEW means); 6 still refused, each with
its code and message; 7 cost; 8 kind. In the owner's-test tables "none (default: ...)" means no word in
the line answers that question and the stated default does (allowed only where the default never changes
anything, never aliases silently and never copies silently).

## Terms used in this area

- **function value**: a function or lambda kept in a variable, a list or a field (a callback, a
  handler): two words, its code and its **state** (what it captured with a word). Its type says its
  parameters, its result and whether any thread may run it (`thread_shared fn()`), and nothing about
  what it changes: the compiler works that out from every function the program can put there (R12,
  verdict 86).
- **capture**: a name a lambda uses from the function that made it. **The real variable** (row 6): that
  very variable, not a copy; a write in the lambda is seen by the function, and back. A lambda writes
  nothing about what it changes (`b.on_click(fn(): score += 1)`, row 6); it is checked in the function
  that writes the lambda (R12).
- **capture list**: the words after a lambda's brackets that name what it keeps (`fn() copy of score:`,
  `fn() owns count:`; verdict 83: no `with`).
- **worked out** (what a function, a method or a stored function changes, verdict 86): no word on a first
  line or in a function type says it; the compiler works it out from the body, the functions it calls,
  every function the program can put in a function value it calls and every override of a virtual it
  calls (R5, R12), and the rules that need it use that fact, their messages naming the chain of calls.
  In the owner's-test tables a field's "may it change it" is then "through a name that may change its
  holder", and a method's `this` "none written (worked out: ...)", with what its body changes.
- **kind test**: `if s is Circle as c:` (runs its block when `s` is a `Circle`, and names it `c` there)
  and `if s is Circle:` (tests only); the "not sure" downcast. `cast s as Circle` is the "sure" one: it
  stops the program if `s` is not a `Circle` (R14, verdict 27).
- **holder**: the object or list whose field or element holds a function value or another object.
- **opening**: R13's `if cam.target exists as p:` (or `while cur.parent exists as p:`): the block runs when the
  `Option` or the `weakref` link holds something, and names it `p` there.
- **parent link**: a `weakref` field naming the object that owns its holder, set by the program
  (`child.parent = weakref this`, then `this.children.push(child)`; R11).
- **header test**: the read of a `weakref` link's target's header to see whether the object still lives.
- **count pair**: one +1 and one -1 of an object's count around a use, to keep it alive across it (R12:
  a handler's own state across a call that may remove it).
- **deferred-drop region**: a stretch of code (a loop) during which an object whose last owner ends is
  set aside whole (it and everything it owns marked ended) and freed when the stretch ends (R11's walk).
- **ended**: an object is ended the moment its last owner lets it go, before its `drop` runs; from then
  on a `weakref` link to it finds nothing (R11, verdict 56).
- **AddressSanitizer** (ASan): a clang and gcc option that stops a program with a report the moment it
  reads or writes memory it must not.
- **today**: `ember check <file>` with today's compiler (`target/redesign3/bin/ember.exe` in run 3,
  `target/redesign4/bin/ember.exe` in run 4 and v2, on the owner's laptop; `target/redesign4/bin/ember
  check` in this checkout). Every claim about today has its output pasted.

The OLD code of every answer is unchanged, so every OLD part, with its `ember check` output, is copied
unchanged from `RUN4/area-D.md` through `V2/area-D.md` and `V3/area-D.md` (file names in it are relative
to `target/redesign3/11-area-D/`).

## What step 4 changed in each answer

"Meaning changed" compares with `V4/area-D.md`; what changed before is that file's table "What v4
changed in each answer". It says whether a program of the answer now does something else, or a program
it refused for more than a missing or mismatched word now compiles (or the reverse); a refusal that only
asked for a word verdict 86 removed (E2105, E2113) is a change of spelling, not of meaning. The verdicts
that land in area D: 86 (`changes`, `captured` and change lists removed; the compiler works out what
every function changes: every answer with a callback or a method), 83 (`with` dropped from capture
lists: AN-53, AN-54, AN-55, AN-57, AN-59, AN-60), 84 (`owns` hands over every type, numbers included:
AN-53's case 2, AN-54's E2085, step 3's question 2), 85 (`owns` on the making line inside a list:
AN-66, step 3's question 3), 53's corrected record (only C's code is unseen: AN-62 (b)'s
`this.on_visit(p)` compiles, step 3's question 1, worklist F-06), 94 (`thread.spawn` takes a
`thread_shared fn()`: AN-55). The worklist's section 4 items for area D: F-01 (E3060's way out opens the
link under a new name: AN-62, AN-65), F-06 (AN-62), F-15 (AN-70's prose and `min`), U5 (E2037 for a
kind test that can never pass: AN-67). Verdicts 87 to 89, 91 to 93 and 95 to 97 change no line of area
D: no answer hands a `ref mut` argument a callee clears, reaches one variable by two names or a locked
object through two, gives C an object, sums on threads, starts a task in a loop, writes a walked
collection, tests a count's limit or makes a value later for an `owns` place.

| answer | what changed | meaning changed? |
|---|---|---|
| AN-53 (row 6) | row 6 is `b.on_click(fn(): score += 1)` and its copy `fn() copy of score:` (verdicts 83, 86); `Button` holds `Array[fn()]`, `on_click` and `click` say nothing; part 5's case 2 is `return fn() owns count -> int:` (verdicts 83, 84); E3026's message is the rules' own; the E2105 and E2113 refusals go with the words; a plain `h` in `click` is E3023 | no: row 6's program adds 1 to `game`'s real `score`; the handlers v4 refused for a mismatched word (two captured variables, `LOG`) are kept in the plain list, and `b.click()` in a loop over what they change is still refused (E3020) |
| AN-54 (row 6) | `cb = fn() owns name, copy of score:` (verdict 83); E2085 removed (verdict 84); E2105 and E2113 go | **yes**, for one refused program (verdict 84): `fn() owns score:` on an `int` hands it over (E3040 on a later use) instead of being refused (E2085) |
| AN-55 | the lambda says nothing (verdict 86), `fn() owns xs:` (verdict 83), `spawn`'s first line `[R]` above `pub fn spawn(owns f: thread_shared fn() -> R) -> JoinHandle[R]:` (verdict 94); E7015 by verdict 61's rule; the E3020 messages name the chain | no |
| AN-56 | `[C]` and `changes C` off `each_name` (the script left `[C]`); the first lambda `fn(owns s): names.push(s)`; E2105 and E2113 go; E2020's messages without a clause | no |
| AN-57 | the refused lambda `fn(): total += 1`; E3026's message is the rules' own (no `changes` lines, no `Button` lines) | no |
| AN-58 | `hs: Array[fn()]`, `hs.push(fn(): score += 1)`; E2105 and E2113 go | **yes**, for one refused program (verdict 86; `V5/changes.md` section 8 item 4): `for h in hs: h()`, refused in v4 (E3023), compiles, since no function stored in `hs` changes its own state |
| AN-59 | `fn() owns s` in the NEW and the messages (verdict 83) | no |
| AN-60 | `fire`, `remove_current` and `UiButton`'s lines with no clause; `bump_ui = fn() weakref ui:`; G-05's handler refused at the `for` walk (E3020) instead of at the store (E2113, gone); E2105 and the `Button` E2113 go | **yes**, for one refused program (verdict 86): G-05's handler stored in a list of `fn(Ui)` handlers compiles, and the `for` walk over that list is refused (E3020) instead; the NEW does what it did |
| AN-61 | `fire` with no clause; E3031 reads every stored handler (53's corrected record) | no |
| AN-62 | `on_visit: fn(Node)` in both forms: which walk is written is decided by the functions the program stores in it (R11, R12); `this.on_visit(p)` compiles (53's corrected record, F-06, step 3's question 1); a stored function that ends its node and reads it is E3031 where it is stored (new); E2105 and E2113 go; E3060's way out with a new name (F-01) | **yes** (verdicts 53, 86): `this.on_visit(p)`, which v4's E3031 refused as written, compiles; storing a node-changing function in `on_visit` is no longer refused at the store (E2113), and (a)'s plain walk is refused instead (E3021) |
| AN-63 | `add` and `take` with no clause; the owner's test says "worked out" | no |
| AN-64 | the owner's test without `changes` | no |
| AN-65 | E3060's way out opens the link again as `q` (F-01); the owner's test without `changes` | no |
| AN-66 | `shapes=[owns Circle(r=1.0), owns Square(side=2.0)]` (verdict 85, step 3's question 3); E2123's message added; the E3040 example's `owns` making line | no |
| AN-67 | the kind test that can never pass is E2037 (U5); the owner's test without `changes` | no |
| AN-68 | the "v5" line only | no |
| AN-69 | the "v5" line only | no |
| AN-70 | `virtual fn on_update(dt: float):` and `override fn on_update(dt: float):`; E2113 goes; the prose about `min` corrected (F-15) | no: v4's refusal of the override (E2113) only asked for a word on the virtual |

Lines the spelling script changed in this file (the report's section 3), checked: 31 lines, all right
as far as they go; it could not finish three things, done here: AN-56's `[C]` line above `each_name`,
left without a use, is removed; AN-53's handler-list comment ("each may change the one variable it
captured") and AN-62's two `on_visit` comments, which described the removed types, are rewritten; AN-53's
part 5 block (`return fn() with owns count -> int:`, a NEW program) takes the new spelling.

---

## AN-53: A closure that changes a variable of the function that made it (A12; agreed row 6)

**v5:** Row 6 reads as the owner agreed it, `b.on_click(fn(): score += 1)`, now with no word on the
lambda (verdict 86 voids verdict 22's `changes score`), and its copy is `fn() copy of score:` (verdict
83). `Button` holds `Array[fn()]`, and `on_click` and `click` say nothing: wherever a stored handler runs,
the compiler counts what every handler the program stores there changes (R12); `captured` is gone.
Part 5's case 2 is `return fn() owns count -> int:` (verdicts 83, 84), which settles step 3's question
2. Part 6: E3026's message is the rules' own, with no `changes` line and no `Button` line; the E2105 and
E2113 refusals go with the words; `click` with a plain `h` is E3023. **Meaning changed:** no. Row 6's
program does what it did: the button adds 1 to `game`'s real `score`. The handlers v4 refused only for a
word their list did not have (one that changes two captured variables, one that changes `LOG`) are kept
in the plain list; a loop over what they change that clicks the button is still refused (E3020).

**1. Intent:** the button adds 1 to `game`'s score.

**2. OLD.** The Python spelling (`old/an53_button_plain.em`, last run's `a12_button_python.em`):

```ember
class Button:
    handlers: Array[fn()] = []

    fn on_click(self, f: fn()):
        self.handlers.push(f)

    fn click(self):
        for h in self.handlers:
            h()

fn game() -> int:
    score = 0
    b = Button()
    b.on_click(fn(): score += 1)
    b.click()
    return score
```

```text
error[E2020]: expected `fn() -> void`, found `Callable0`
 --> an53_button_plain.em:6:28
  |
6 |         self.handlers.push(f)
  |                            ^ this is `Callable0`
  |

error[E3026]: this lambda borrows what it captures, so it may not be stored as `fn() -> void`
 --> an53_button_plain.em:6:28
  |
6 |         self.handlers.push(f)
  |                            ^ stored here
  |
  = help: write it `owned fn`, which moves or copies each capture into the value
  = note: a `fn` type outside a parameter is an owned callable value; a lambda that borrows is a view, and this place has no bounding region (CLO-3, TYP-15)

error[E3021]: cannot write through a shared reference
  --> an53_button_plain.em:10:13
   |
10 |             h()
   |             ^^^ this is `ref fn() -> void`, which only reads
   |
   = help: a callable value may change what it captured, so it is called from a mutable place [CLO-2]: borrow it as `ref mut`, and iterate a collection of them with `.iter_mut()` (`for f in fs.iter_mut():`)
   = note: while a shared borrow is live the owner may read and copy but not write [BRW-1]

error: could not compile due to 3 error(s)
exit=1
```

Following the help (`old/an53_button_owned_fn.em`: `owned f`, `.iter_mut()`, `b.on_click(owned fn():
score += 1)`) is accepted, and by `[CLO-2]` the handler adds 1 to its own copy, so `game()` returns 0:

```text
=== an53_button_owned_fn.em
exit=0
```

**3. NEW.**

```ember
class Button:
    handlers: Array[fn()] = []      # what each handler changes is worked out (R12)

    fn on_click(owns f: fn()):
        this.handlers.push(f)

    fn click():
        for ref mut h in this.handlers:
            h()

fn game() -> int:
    score = 0
    b = Button()
    b.on_click(fn(): score += 1)       # row 6: game's real `score`
    b.click()
    return score                       # 1

fn game_copy() -> int:
    score = 0
    b = Button()
    b.on_click(fn() copy of score: score += 1)   # row 6's copy: the handler's own `score`
    b.click()
    return score                       # 0: only the handler's copy went up
```

Changed from v4 (verdicts 83, 86): no line says what a function changes. The lambda is row 6's own
`fn(): score += 1`, the list is `Array[fn()]`, `on_click` and `click` say nothing, and the copy is
`fn() copy of score:`. The calls write no `ref mut`: `b.on_click(...)` and `b.click()` are methods called
on `b` (R4); `f` is an `owns` parameter, and a hand-over is written with no word (R4).

Row 6's handler changes `score`, a real variable of `game` that it captures; the compiler sees it in the
lambda's body (R12, verdict 86). `Button`'s list says nothing about what its functions change; wherever
one is called, the compiler counts the changes of every function the program stores there (R12): here
row 6's handler (`game`'s `score`) and `game_copy`'s (its own copy). `click` walks with `ref mut h`
because a call through a stored function value whose functions may change their own state is a change of
the place that holds it (R12): `game_copy`'s handler changes its own copy of `score`. In `game`, the
function that wrote the lambda, `score` is its own local, and the compiler checks it there: `b` holds a
function value that reaches `score`, and `b.click()` counts as a change of `score` (R12), so
`return score` reads it after the click. Where `game`'s real `score` lives (R12): at its own address,
since `b` never leaves `game` and the methods it is handed to keep the handler no longer than `b` (by
their summaries), so the handler provably ends first. In `game_copy` the captured `score` is named
(`copy of`), so it is the handler's own state.

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `handlers` (field) | none (default: the Button's own list; each element its own function value; the type `fn()` says nothing about what they change, which is worked out from every function the program stores there, R12) | the list: through a name that may change the Button (`on_click`, by its body); each function: what its body changes |
| `this` in `on_click` | `this`: the Button the method was called on | none written (worked out: `on_click`'s body pushes to `this.handlers`, R5) |
| `f` | `owns`: handed over to `on_click` | its new owner may; it hands it on to the list (storing it changes nothing it changes, R12) |
| `this` in `click` | `this` | none written (worked out: calling a stored handler that changes its own state changes the list's element, and `click` makes what the handlers change, here `game`'s `score`, R12) |
| what `click` runs | the handlers stored in `this.handlers` | what their bodies change (R12): row 6's adds to `game`'s real `score`, `game_copy`'s to its own copy |
| `h` | `ref mut`: an element of `handlers`, the only name for it while used (R8 (f)) | yes; calling it changes what its body changes, its own state included (R12) |
| `score` in `game` | none (default: `game`'s own `int`, made by `=`) | its owner may |
| `b` in `game` | none (default: `game`'s own Button, made by `=`) | its owner may (through `on_click` and `click`) |
| `score` in the lambda | none (row 6: the real `score` of `game`) | none (row 6: a capture with no word is the real variable, which the lambda may change; what it changes is worked out, verdict 86): yes, the real one |
| `score`, `b` in `game_copy` | none (default: `game_copy`'s own `int` and Button) | as in `game` |
| `score` in `game_copy`'s lambda | `copy of`: the handler's own copy (row 6) | yes, its own copy (its own state, R12), through `click`'s `ref mut h` |
| `count` in `make_counter` (part 5's case 2) | none (default: `make_counter`'s own `int`, made by `=`) | its owner may; `owns count` hands it to the function (verdict 84: a number too) |
| `count` in case 2's lambda | `owns` (in `owns count`): the function's own state from that line on (verdicts 80, 84) | yes, its own state (`count += 1`), which needs no word (R12) |
| `c` in case 2's `main` | none (default: `main`'s own function value, made by `=`) | its owner may; calling it changes only its own state |

**5. Means exactly.** In today's Ember the real variable shared by `game` and the handler is a counted
object (`means/m53_button_box.em`):

```ember
class Score:
    n: int = 0
...
fn game() -> int:
    score = Score()
    b = Button()
    b.on_click(owned fn(): score.n += 1)
    b.click()
    return score.n
```

```text
=== m53_button_box.em
exit=0
```

That is a counted object today's Ember makes the programmer write. R12 makes none: without a word the
handler only borrows, and `score` stays at its address (the handler provably ends first, above): the C
is `c/an53_captures.c` case 1, `on_click(&b, (Fn0){bump, &score})`, run under ASan:

```text
game=1
counter=2
an53_captures ms=0.040 peak_kb=8176 checksum=0
```

(`counter=2` is case 2, a counter factory whose function value is returned; the C keeps its variable in
a counted heap block, run 3's box.) No ASan report. Under verdict 80 that factory says what its function
keeps, and the variable is the function value's own state, which needs no word (R12):

```ember
fn make_counter() -> fn() -> int:
    count = 0
    return fn() owns count -> int:         # the function owns `count` from here on
        count += 1
        return count

fn main():
    c = make_counter()
    c()
    println(c())                       # 2
```

`main` names nothing: `c` changes only its own state. The C of case 2 does the same work with `count` in
a separate counted box; R12 now puts it in the function value's own block instead (the placement run
3's W4 measured, part 7), which this step did not build in C. (`owns` on an `int` hands it over, as for
every type: verdict 84, which settles step 3's question 2.)

`game_copy` means exactly the OLD's `old/an53_button_owned_fn.em`
(`b.on_click(owned fn(): score += 1)` with `.iter_mut()`, `exit=0`, pasted in part 2): today's `owned fn`
copies the `int`, which is what `copy of score` names.

**6. Still refused.**
- A capture of someone else's place in a function value that may outlive it (AN-57's program), E3026,
  rules section 3's message: "the click handler changes `total`, which belongs to `register`'s caller,
  and `b` keeps the handler after `register` returns; keep the total in an object the handler can own a
  share of: declare `class Counter:` with the field `n: int = 0`; in `main`, instead of `total = 0`,
  `register(ref mut b, ref mut total)` and `println(total)`, write `count = Counter()`, `register(ref mut
  b, shares mut count)` and `println(count.n)`; and instead of `register`'s two lines write `fn
  register(ref mut b: Button, owns count: shares mut Counter):` and `b.on_click(fn() owns count: count.n
  += 1)`". `Button` stays as it is: its `Array[fn()]` holds any function, and `b.click()` then changes
  the `Counter`, which the compiler works out (R12).
- Part 5's case 2 with no capture word, `return fn() -> int:` with the same body in `make_counter`, E3026
  (verdict 80): "the returned function changes `count`, which ends when `make_counter` returns; let the
  function own it: write `return fn() owns count -> int:` in place of this line" (R12; the rules'
  `make_counter` message in this program's names).
- A capture used by a thread that may outlive it (AN-55), E7004.
- Not refused any more (verdict 86; v4 refused each only for a word): a handler that changes two
  variables it captured (a `fn():` whose block adds to `score` and takes from `lives`), or a module
  variable (`b.on_click(fn(): LOG.push(String("clicked")))`), is kept in the same `Array[fn()]` list,
  wherever `Button` is declared; v4 asked for `captured...` or for `LOG` in the list's type (E2113), and
  verdicts 35, 40 and 76, which said how to write those words, are void.
- A handler that changes the button that keeps it, `b.on_click(fn(): b.handlers.clear())`, E3022:
  "this function clears `b.handlers`, and `on_click` would keep it in `b.handlers`, so it would change
  the button that holds it while the button runs it; clear the handlers after the click instead:
  `b.click()`, then `b.handlers.clear()`" (R12: a function value is not stored in a place that a real
  variable it changes owns, as Rust refuses; the rules' message).
- `click` walking with a plain name, `for h in this.handlers:`, E3023: "`h` only reads the handler in
  `this.handlers`, and calling it may change its own state (the handler stored on line 21, `game_copy`'s,
  adds to its own `score`); walk the handlers with `for ref mut h in this.handlers:` in place of line 8"
  (R12; the rules' message in this program's lines).
- A loop that walks a list a function value changes, around a call that may run it (R12, R8 (c)),
  E3020: "this loop walks `xs` (line 5), and `hs[0]()` (line 6) may run the function made on line 3,
  which pushes to `xs`; call it after the loop, as many times as the loop would: write `n = xs.len()` before the loop, and `for k in 0..n: hs[0]()` below it instead of `hs[0]()` in it". The
  same holds for `b.click()` inside a loop over a list of `game`'s that a handler in `b` changes: in
  `game`, which wrote the lambda, the call counts as each change the lambda's body makes (R12).
- A handler that changes a module variable, `b.on_click(fn(): LOG.push(String("clicked")))`, makes
  `b.click()` a change of `LOG`, wherever `Button` is declared, a library included (R12: the compiler
  reads every function the program stores in `handlers`), so `b.click()` inside a loop over `LOG` is
  refused, E3020: "`b.click()` (line 17) adds to `LOG` (through `click`, which runs the handler stored on
  line 15, which pushes to `LOG`) while this loop walks `LOG` (line 16); click after the loop: move line
  17, `b.click()`, below the loop" (R8 (c), R12; the rules' message in this program's lines). A button
  where no handler the program stores changes `LOG` is clicked inside a loop over `LOG` with no refusal
  (G-07 settled by what the program stores into the list, not by a search of each call).

**7. Cost.** At an address: none (the C passes `&score`). In the function value's own state
(`game_copy`, and case 2's `owns count`): nothing beyond the function value's own block. No box is
made (verdict 80, R12): a function value that would outlive a variable it only borrows is refused
(E3026), and one that keeps it says `owns`. Run 3's W4 (callbacks that remove themselves) measured
the state placement at 1.091x of plain C, a box per captured variable (no longer made) at 1.255x, safe
Rust at 1.358x (`RUN3/26-timer.md` lines 29, 31, 32: `rust_w4`, `model_w4`, `model_w4_state_block`);
the phase before measured the callback workload at 1.018 and 1.052 of plain C with the shared total
boxed, against 1.045 and 1.037 with it by address
(`target/redesign2/graphs-callbacks-attack2/bench/runs/w4_a.txt`, `w4_b.txt`, summarised in
`phase3b-graphs-callbacks-attack.md` 6.4). Rust's spelling of the same program is `Rc<Cell<i32>>`, a
box the programmer writes. What each handler changes is worked out when compiling and costs nothing
while the program runs (`V5/rules.md` section 2).

**8. Kind:** a smarter compile-time solution (Rust makes the programmer write a shared box for the real
variable; here the compiler proves the handler ends first and keeps `score` at its address, and where
it cannot prove that it refuses the program with the lines to write, E3026), with what each handler
changes worked out from its body, no word on it (verdict 86).

---

## AN-54: `owned fn` moves a `String` capture and copies an `int` one (agreed row 6)

**v5:** the capture list loses `with` (verdict 83): `cb = fn() owns name, copy of score:`. Verdict 84
removes E2085: `fn() owns score:` on an `int` hands the number over, and a later use of `score` is E3040,
which settles step 3's question 2. Verdict 86: a callback that changes the real `score` needs no word and
fits `run`'s `fn()`; part 6's E2105 and E2113 go. **Meaning changed:** yes, for one refused program
(verdict 84): `fn() owns score:` is a hand-over, not an error. The NEW does what it did: it prints 0.

**1. Intent:** a callback that uses a name and a score.

**2. OLD.** `old/an54b_owned_fn_int.em` (only the `int`):

```ember
fn run(mut f: fn()):
    f()

fn main():
    score = 0
    cb = owned fn(): score += 1
    run(cb)
    println(score)
```

```text
=== an54b_owned_fn_int.em
exit=0
```

Accepted, and it prints 0 by `[CLO-2]` (the `int` is copied). The same word with a `String` too
(`old/an54_owned_fn_two_types.em`, `cb = owned fn(): score += 1; println(name)`, then
`println(score)` and `println(name)`): the `String` was handed over, the `int` was not:

```text
error[E3040]: `name` is borrowed after it has been moved out of
  --> an54_owned_fn_two_types.em:13:13
   |
 8 |     cb = owned fn():
   |          ----------- moved here
...
13 |     println(name)
   |             ^^^^ borrowed here after the move
   |
   = help: borrow before the move, or clone the value so the move takes the copy
   = note: the borrow would point at memory whose owner gave it away
   = note: assigning a list, a map or a set moves it; in Python both names would point at one list (Appendix E)

error: could not compile due to 1 error(s)
exit=1
```

**3. NEW.** The usual callback names nothing (row 6, the real variables). A callback that must own the
name and keep its own score says so on each name:

```ember
fn run(ref mut f: fn()):
    f()

fn main():
    name = String("ann")
    score = 0
    cb = fn() owns name, copy of score:
        score += 1                    # its own copy
        println(name)                 # its own String
    run(ref mut cb)                   # `run` may change `cb`: its own copy of `score` goes up
    println(score)                    # 0, as `copy of` says
```

The call says `run(ref mut cb)`, as `run`'s first line says `ref mut f` (R4). `cb` is a plain `fn()`:
it changes only its own state, which is the function itself (R12), so it fits `run`'s `fn()`.

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `f` | `ref mut`: `main`'s function value, lent | yes (`ref mut`): `cb` changes its own copy when called (R12) |
| `name` in `main` | none (default: `main`'s own String, made by `=`) | its owner may |
| `score` in `main` | none (default: `main`'s own `int`) | its owner may |
| `cb` | none (default: `main`'s own function value, made by `=`) | its owner may; it is lent `ref mut` to `run`, and the call says so |
| `name` in the lambda | `owns`: handed over to the function | its new owner may |
| `score` in the lambda | `copy of`: the function's own copy | yes, its own copy (`score += 1`) |

**5. Means exactly.** Today's `owned fn` does exactly this (moves the `String`, copies the `int`); NEW
only writes it on each name (`means/m54_named_captures.em`):

```text
=== m54_named_captures.em
exit=0
```

**6. Still refused.**
- Today's `owned fn(): ...`, E1010: "name what the function takes or copies on each name:
  `cb = fn() owns name, copy of score:` in place of line 7; a name not listed is the real variable".
- `owns fn(): ...`, E0109: "`owns fn` does not say what the function takes or copies; name it on each
  captured name: `fn() owns name, copy of score:`".
- `println(score)` (line 11) after `cb = fn() owns name, owns score:` (line 7), E3040 (verdict 84: `owns`
  hands a number over too; E2085 is removed): "`score` was handed over to the function on line 7
  (`owns score`) and is used on line 11; to keep your own, write `cb = fn() owns name, copy of score:` in
  place of line 7".
- `println(name)` (line 12) after `owns name`, E3040: "`name` was handed over to the function on line 7
  (`owns name`) and is used on line 12; to keep using it, give the function a copy of its own, `cb = fn()
  copy of name, copy of score:`, or share it, `cb = fn() shares name, copy of score:`, in place of line 7".
- `fn() copy of name, score:`, E0116: "`copy of` here belongs to `name` only; say what the function does
  with `score`: `fn() copy of name, copy of score:`, or leave `score` out to use the real `score`".
- Not refused any more (verdict 86): `cb = fn() owns name:` with the same body uses the real `score`
  (no word: row 6), and `run(ref mut cb)` changes it, so `println(score)` prints 1. v4 asked for
  `changes score` (E2105) and then refused that function in `run`'s `fn()` (E2113); both were words.
- `run(cb)` with `fn run(f: fn()):`, E3023: "`cb` changes its own copy of `score` each time it runs, and
  `run` takes it read-only; write `fn run(ref mut f: fn()):` and call `run(ref mut cb)`" (R12).
- `run(cb)` with `fn run(ref mut f: fn()):`, E3028: "`run` may change `cb` (its first line says
  `ref mut f: fn()`); say so at the call: `run(ref mut cb)`" (R4).

**7. Cost.** None beyond C: the `String`'s three words move into the function's state, the `int` is
copied, as a Rust `move` closure.

**8. Kind:** a readable spelling of Rust's guarantee (`move`, written per name).

---

## AN-55: The compiler decides a capture is moved from which function receives the lambda (`[CLO-15]`)

**v5:** the lambda given to `inspect` says nothing about what it changes (verdict 86: the compiler sees
that it pushes to `seen`, and `count()` is the call that runs it, R12), and the thread's function keeps
`xs` with `owns xs` (verdict 83). `thread.spawn`'s first line is `[R]` above `pub fn spawn(owns f:
thread_shared fn() -> R) -> JoinHandle[R]:` (verdict 94), so a function holding a `shares` owner is
refused by verdict 61's rule (E7015). Part 6's E2105 for v2's iterator line goes with the word; the
E7004, E7015 and E3020 messages give whole lines and name the chain. Meaning unchanged.

**1. Intent:** count `xs` on another thread, and record the values an iterator sees into `seen`.

**2. OLD.** `old/an55_spawn_moves.em`:

```ember
fn main():
    xs = [1, 2, 3]
    seen: Array[int] = []
    _n = xs.iter().inspect(fn(v) => seen.push(v)).count()
    h = thread.spawn(fn() => work(xs))
    println(seen.len())
    println(xs.len())
    println(h.join())
```

```text
error[E3026]: the thread's function borrows `xs`, which it may outlive
  --> an55_spawn_moves.em:11:22
   |
11 |     h = thread.spawn(fn() => work(xs))
   |                      ^^^^^^^^^^^^^^^^ the thread may outlive what it views
   |
   = help: write it `owned fn`, which moves or copies each capture into it; or borrow it inside `with scope = thread.scope():`, which joins its threads before the block ends [THR-5]
   = note: a thread may outlive every local of the code that started it [THR-10]

error: could not compile due to 1 error(s)
exit=1
```

The same lambda text views `seen` at `inspect` and, by `[CLO-15]`, would be moved at a registry: the
callee decides, nothing in the line says it.

**3. NEW.**

```ember
fn main():
    xs = [1, 2, 3]
    seen: Array[int] = []
    _n = xs.iter().inspect(fn(v): seen.push(v)).count()   # the real `seen`
    h = thread.spawn(fn() owns xs: work(xs))                      # `xs` handed to the thread
    println(seen.len())
    println(h.join())
```

(`work` is the OLD's: it reads the list it is given and returns a number.) Changed from v4 (verdicts 83,
86): the lambda says nothing about what it changes, and the thread's function keeps `xs` with
`owns xs`, no `with`. The library's first line is `[F: fn(Item)]` above `fn inspect(owns f: F) owns
this -> Inspect[Self, F]:` (verdicts 38 and 50; R12, `V5/vocabulary.md`'s iterator row): `inspect`
keeps the function in the adapter, which changes nothing yet (storing a function is not a change, R12),
and the adapter's `next` runs it, so `count()`, which calls `next` until the end, is the call that
pushes to `seen` (worked out from the function given, R12). In `main`, which wrote the lambda, `seen` is
its own local (R5). One instance of `inspect` is made for this lambda (the bound `F`), so the call
through `f` is direct, as Rust's `F: FnMut`.

The thread: `thread.spawn`'s first line is `[R]` above `pub fn spawn(owns f: thread_shared fn() -> R) ->
JoinHandle[R]:` (verdicts 50, 94). Its function may run on another thread, so it holds only what it was
handed, copies and `thread_shared` values (R12, verdict 61), and starting the thread counts as making
what its function changes, now or later (R12, R15; verdict 30). The thread's function holds `xs`
(`owns xs`, R15) and changes nothing but its own state, so starting it changes nothing of `main`'s.

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `xs` in `main` | none (default: `main`'s own list) | its owner may |
| `seen` in `main` | none (default: `main`'s own list) | its owner may |
| `v` | none (default for a lambda parameter: it reads; an `int`, copied in) | no |
| `seen` in the lambda | none (row 6: the real `seen`) | none (row 6: no word; what the lambda changes is worked out, verdict 86): yes (`push`), the real one |
| `_n` | none (default: `main`'s own `int`, the count `count` hands back) | its owner may |
| `h` | none (default: the `JoinHandle` `thread.spawn` returns, `main`'s own) | its owner may (`join`) |
| `xs` in the thread's function | `owns`: handed over to the thread | its new owner may |

**5. Means exactly.** `means/m55_spawn_owned.em` (`thread.spawn(owned fn() => work(xs))`, `xs` not
used after) and the scoped form `means/m55_scope.em` (the thread uses the real `xs` inside
`with scope = thread.scope():`):

```text
=== m55_scope.em
exit=0
=== m55_spawn_owned.em
exit=0
```

and the whole NEW program in today's words (`target/redesign4/v2-05-area-D/m55_inspect_count.em`: the
iterator line as written, `thread.spawn(owned fn() => work(xs))`), checked in v2's step, on the
owner's laptop (`target/redesign4/v2-05-area-D/check_output.txt`):

```text
=== m55_inspect_count.em
exit=0
```

**6. Still refused.** `thread.spawn(fn() => work(xs))` (line 5), E7004: "this thread may outlive `main`,
so it cannot use `main`'s `xs`; say what the thread holds: `h = thread.spawn(fn() owns xs: work(xs))`
(handed over) or `h = thread.spawn(fn() copy of xs: work(xs))` (a copy of its own) in place of line 5"
(R15). `thread.spawn(fn() shares xs:
work(xs))`, E7015 (verdicts 61, 94): "`thread.spawn` runs its function on another thread (`thread_shared
fn()`), and this one holds `xs`, a `shares` owner, which stays on one thread; hand `xs` over or give the
thread a copy: `h = thread.spawn(fn() owns xs: work(xs))` or `h = thread.spawn(fn() copy of xs:
work(xs))` in place of line 5". And:
- The iterator line inside a loop over `seen`, `for s in seen:` (line 3) with the iterator line as line
  4, E3020: "`xs.iter().inspect(...).count()` (line 4) pushes to `seen` (through `count`, which runs the
  function `inspect` keeps, which pushes to `seen`) while this loop walks `seen` (line 3); count after the
  loop: move line 4 below the loop" (R8 (c), R12).
- A scoped task started inside a loop over what its function changes, in the lines `with scope =
  thread.scope():` (line 3), `for s in seen:` (line 4), `scope.spawn(fn(): seen.push(1))` (line 5), E3020
  (verdict 30): "`scope.spawn(...)` (line 5) starts a task that pushes to `seen` (`fn(): seen.push(1)`),
  which this loop walks (line 4); start it after the loop: move line 5 below the loop" (R8 (c), R15; the
  rules' message; E3020, not E7007, since E3020's fix compiles, R18, verdict 73).

**7. Cost.** None beyond C: the lambda given to `inspect` holds `seen`'s address and is called directly
(one instance of `inspect` per function given), the hand-over moves three words. What the functions
given change is worked out when compiling and costs nothing while running (`V5/rules.md` section 2).

**8. Kind:** a readable spelling of Rust's guarantee (`move` and scoped threads), with no rule that
depends on the callee.

---

## AN-56: A lambda parameter without a mode takes `owned` from the expected callable type (`[TYP-23]`)

**v5:** `each_name` loses its change list (verdict 86: `[C]` and `changes C` go; the script left the `[C]`
line, removed here), and the first lambda its word for `names`: `each_name(fn(owns s): names.push(s))`.
The second call stays verdict 71's `each_name(fn(owns s) => println(s.len()))`. Part 6: E2105 and E2113
go with the words, and v2's line `each_name(fn(owns s) => names.push(s))` is accepted again (it is the
first NEW line written with `=>`); the two E2020 messages give lines with no clause. Meaning unchanged.

**1. Intent:** pass each name to a callback that keeps it in `names`.

**2. OLD.** `old/an56_lambda_param_owned.em`:

```ember
fn each_name(owned f: fn(owned String)):
    f("ann".to_string())

fn main():
    names: Array[String] = []
    each_name(fn(s) => names.push(s))
    println(names.len())
```

```text
=== an56_lambda_param_owned.em
exit=0
```

`s` has no word, yet it is handed over into `names` (`[TYP-23]`, ODR-025).

**3. NEW.**

```ember
fn each_name(owns f: fn(owns String)):
    f(String("ann"))

fn main():
    names: Array[String] = []
    each_name(fn(owns s): names.push(s))   # takes `s` over, keeps it in the real `names`
    each_name(fn(owns s) => println(s.len()))             # takes `s` over, reads it; it ends with the call
```

Changed from v4 (verdict 86): `each_name` has no change list (`[C]` and `changes C` are gone), and the
first lambda no word for `names`. `f(String("ann"))` and `each_name(...)` hand their arguments over to
`owns` parameters, written with no word (R4). Each call counts as making what the function given changes
(R12): the first changes `names`, `main`'s own local (R5); the second changes nothing outside itself (it
reads its own `s`, which ends when it returns). Written with a function type, `f` is called through its
two words; the library's form takes it as a template parameter (`[F: fn(owns String)]` above
`fn each_name(owns f: F):`, verdict 38), which makes one instance per function given.

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `f` | `owns`: handed over to `each_name`; its type `fn(owns String)` hands each name over | its new owner may; calling it makes what the function given changes (worked out, R12) |
| `names` | none (default: `main`'s own list) | its owner may |
| `s` in the first lambda | `owns`: handed over to the lambda | its owner may (it hands it on to `names`) |
| `names` in the first lambda | none (row 6: the real `names`) | none (row 6: no word; worked out, verdict 86): yes (`push`), the real one |
| `s` in the second lambda | `owns`: handed over to the lambda (verdict 71: the type `fn(owns String)` hands it over) | its owner may; it only reads it, and it ends when the lambda returns |

**5. Means exactly.** `means/m56_lambda_owned_param.em` (`fn(owned s) => names.push(s)`) and
`means/m56_lambda_read_param.em` (`fn(s: String) => println(s.len())` where `fn(owned String)` is
expected):

```text
=== m56_lambda_owned_param.em
exit=0
=== m56_lambda_read_param.em
exit=0
```

The second file is today's spelling of the second NEW lambda: the caller hands the `String` over, the
lambda reads it, and it ends after the call, which is what `fn(owns s) => println(s.len())` says on its
parameter.

**6. Still refused.**
- `each_name(fn(s) => names.push(s))` (no word on `s`), E2020 (verdict 71: the parameter's mode differs
  from the function type; the line also keeps a read-only `s`, E3013, and one message gives the line that
  settles both, R18, verdict 73): "`each_name` hands each name over (`fn(owns String)`), so this function
  receives it as its own, and it keeps it in `names`; write `each_name(fn(owns s): names.push(s))` in
  place of line 6".
- v3's second lambda, `each_name(fn(s) => println(s.len()))`, E2020 (verdict 71): "`each_name` hands
  each name over (`fn(owns String)`), so this function receives it as its own: write
  `each_name(fn(owns s) => println(s.len()))` in place of line 7" (R12; rules section 3's E2020 message,
  as a whole line).
- `fn(owns s)` where the expected type is `fn(String)` (a read-only parameter), E2020 (a lambda
  parameter whose mode differs from the expected function type; rules section 3's message): "`each_name`
  gives each name read-only, so this function cannot take it over; to keep your own, write `fn(s):
  names.push(copy of s)`".
- Not refused any more (verdict 86): v2's `each_name(fn(owns s) => names.push(s))` (v4's E2105 asked
  for `changes names`), and v2's `fn each_name(owns f: fn(owns String)):` given the first lambda (v4's
  E2113 asked for `[C]` and `changes C`), which is the NEW's own line.

**7. Cost.** None: the same code as today; what the function given changes is worked out when
compiling (`V5/rules.md` section 2).

**8. Kind:** a readable spelling of Rust's guarantee (the parameter's mode is written, as on every other
parameter, R4).

---

## AN-57: E3026 is reported inside the callee; a stored lambda gets E2020 naming `Callable0`

**v5:** the refused lambda is row 6's form, `fn(): total += 1` (verdict 86), still E3026: it would
outlive `total`, and without a word a function only borrows (verdict 80). E3026's message is the rules'
own: its help keeps the total in a `Counter` the handler owns a share of (`fn() owns count: count.n +=
1`, verdict 83), and no longer rewrites `main`'s first line or `Button`'s lines, since nothing says what
a function changes (verdict 86). Meaning unchanged.

**1. Intent:** find which call wrote the lambda that cannot be kept.

**2. OLD.** `old/an57_button_owned_param.em` (last run's `a12_button_owned.em`, the lambda written at
line 16, `b.on_click(fn(): score += 1)`):

```text
error[E3026]: this lambda borrows what it captures, so it may not be stored as `fn() -> void`
 --> an57_button_owned_param.em:6:28
  |
6 |         self.handlers.push(f)
  |                            ^ stored here
  |
  = help: write it `owned fn`, which moves or copies each capture into the value
  = note: a `fn` type outside a parameter is an owned callable value; a lambda that borrows is a view, and this place has no bounding region (CLO-3, TYP-15)

error: could not compile due to 1 error(s)
exit=1
```

The error points at line 6 inside `Button.on_click`, not at line 16. Without `owned` on the parameter
the first error names a compiler type (`old/an53_button_plain.em`, pasted under AN-53):
`error[E2020]: expected \`fn() -> void\`, found \`Callable0\``.

**3. NEW.** The button program of AN-53 compiles (R12). What is still refused is reported at the lambda,
in the program's names (with AN-53's `Button` declared in the same file, below these lines):

```ember
fn register(ref mut b: Button, ref mut total: int):
    b.on_click(fn(): total += 1)

fn main():
    total = 0
    b = Button()
    register(ref mut b, ref mut total)
    b.click()
    println(total)
```

```text
error[E3026]: the click handler changes `total`, which belongs to `register`'s caller
 --> app.em:2:16
  |
2 |     b.on_click(fn(): total += 1)
  |                ^^^^^^^^^^^^^^^^ written here, kept in `b`
  |
  = why: `b` keeps the handler after `register` returns, and `total` may have ended by then
  = help: keep the total in an object the handler can own a share of:
          declare `class Counter:` with the field `n: int = 0`;
          in `main`, instead of `total = 0`, `register(ref mut b, ref mut total)` and `println(total)`,
          write `count = Counter()`, `register(ref mut b, shares mut count)` and `println(count.n)`;
          and instead of `register`'s two lines write `fn register(ref mut b: Button, owns count: shares mut Counter):`
          and `b.on_click(fn() owns count: count.n += 1)`
```

Callable types are printed as written: `fn()`, `fn(ref mut Ui)`, `thread_shared fn()`, never
`Callable0` (R18). Changed from v4 (verdicts 83, 86): the refused lambda is `fn(): total += 1`; the
help's handler is `fn() owns count: count.n += 1` (it owns a share of the counter, the one `register`
was handed, now the handler's own state; one more owner beside the caller's `count`), and the help no
longer rewrites `main`'s first line or `Button`'s lines: `Button`'s `Array[fn()]` holds any function,
and what `b.click()` then changes, the `Counter`, is worked out (R12). The message is `V5/rules.md`
section 3's E3026 row, word for word.

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `b` | `ref mut`: the caller's Button | yes (`ref mut`) |
| `total` | `ref mut`: the caller's `int` | yes (`ref mut`) |
| `total` in the lambda | none (row 6: the real `total`, here the caller's place) | none (row 6: the real variable, which the lambda may change; worked out, verdict 86); refused because the handler may outlive it |
| `total` in `main` | none (default: `main`'s own `int`) | its owner may; lent `ref mut` to `register`, and the call says so |
| `b` in `main` | none (default: `main`'s own Button) | its owner may; lent `ref mut` to `register` |

**5. Means exactly.** Nothing compiles for the refused program, which is the point. The accepted form in
the help is a counted object with several owners, which today's Ember writes as a class
(`means/m53_button_box.em`, `exit=0`, pasted under AN-53); its `Counter`, shared `shares mut`, is today's
class `Score` there.

**6. Still refused.** The program above, with the message above (E3026). A lambda given to a callee that
keeps it while the lambda captures a view (a `ref`, `ref mut`, pattern, opened or `for` name): the same
message shape, at the lambda's line, naming the captured view and the holder that keeps the lambda.

The help followed inside `register` instead of in the caller, with the counter taken as a plain
parameter: `fn register(ref mut b: Button, count: shares mut Counter):` with `b.on_click(fn() shares mut
count: count.n += 1)`, E3023 (verdict 32: what a function may do with a
parameter is set by the word in front of its name, R4): "`count` only reads the `Counter` that
`register`'s caller lends it (`shares mut Counter` says what the caller's counter is, not what
`register` may do with it), so an owner made from it only reads too; to hand on an owner that may change
the counter, take one: write `fn register(ref mut b: Button, owns count: shares mut Counter):`, call it
`register(ref mut b, shares mut count)`, and capture it `owns count`: `b.on_click(fn() owns count:
count.n += 1)`". (Storing the handler changes nothing it changes, R12.)

**7. Cost.** None (compile time only).

**8. Kind:** an understandable limitation kept (a function value may not outlive a place it does not
own; Rust's E0373), with the message at the right line and in the program's names.

---

## AN-58: A closure stored in a local list that ends first is still E3026

**v5:** `hs: Array[fn()] = []` and `hs.push(fn(): score += 1)` (verdict 86: neither the lambda nor the
list says what it changes; the compiler sees that the handler changes `game`'s real `score`). Without a
word the lambda only borrows `score`, and `hs` provably ends first (R12, verdict 80), so `score` stays at
its address. Part 6's E2105 and E2113 go with the words. **Meaning changed:** yes, for one refused
program (verdict 86; `V5/changes.md` section 8 item 4): a plain `for h in hs: h()`, refused in v4
(E3023) because any stored function might change its own state, compiles, since the one function stored
in `hs` changes none. The NEW is unchanged and returns 1.

**1. Intent:** collect a few local callbacks in a list and run them.

**2. OLD.** `old/an58_local_list.em` (last run's `a12_local_list.em`):

```ember
fn game() -> int:
    score = 0
    hs: Array[fn()] = []
    hs.push(fn(): score += 1)
    for h in hs.iter_mut():
        h()
    return score
```

```text
error[E3026]: this lambda borrows what it captures, so it may not be stored as `fn() -> void`
 --> an58_local_list.em:5:13
  |
5 |     hs.push(fn(): score += 1)
  |             ^^^^^^^^^^^^^^^^ stored here
  |
  = help: write it `owned fn`, which moves or copies each capture into the value
  = note: a `fn` type outside a parameter is an owned callable value; a lambda that borrows is a view, and this place has no bounding region (CLO-3, TYP-15)

error: could not compile due to 1 error(s)
exit=1
```

**3. NEW.**

```ember
fn game() -> int:
    score = 0
    hs: Array[fn()] = []
    hs.push(fn(): score += 1)
    for ref mut h in hs:
        h()
    return score                       # 1
```

`hs` holds a function that changes `score`, a real variable of `game`, which the compiler sees in the
lambda's body (R12, verdict 86). `h()` changes `score`, which the loop does not walk, so the loop is
allowed (R8 (c)); `score` is `game`'s own local (R5). The walk is written `for ref mut h in hs:`, as
AN-53's `click`; since no function stored in `hs` changes its own state, a plain `for h in hs:` compiles
too (R12: only a call through a stored function that changes its own state changes its holder).

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `score` | none (default: `game`'s own `int`) | its owner may |
| `hs` | none (default: `game`'s own list; its elements its own function values); its type `fn()` says nothing about what they change; the one stored changes `game`'s `score` (worked out, R12) | its owner may |
| `score` in the lambda | none (row 6: the real `score`) | none (row 6: no word; worked out, verdict 86): yes, the real one |
| `h` | `ref mut`: an element of `hs`, the only name for it while used | yes; calling it changes what its body changes (R12) |

**5. Means exactly.** `hs` is declared after `score` and never leaves `game`, so the function value
provably ends first and R12 places `score` at its address: the C of `c/an53_captures.c` case 1 (the same
`{code, &score}` pair, there kept in a local `Button`), clean under ASan, output pasted under AN-53
(`game=1`).

**6. Still refused.**
- Not refused any more: `for h in hs: h()` (v4's E3023). No function stored in `hs` changes its own
  state, so calling one through a read-only name changes nothing its holder holds (R12; `V5/changes.md`
  section 8 item 4). With a handler that does, such as AN-53's `fn() copy of score: score += 1`, it is
  E3023: "`h` only reads the handler in `hs`, and calling it may change its own state (the handler
  stored on line 4 adds to its own `score`); walk the list with `for ref mut h in hs:` in place of line
  5".
- A loop over a list that a stored function value changes, around a call that may run it (R12,
  R8 (c)): `for x in xs: hs[0]()` where `hs[0]` pushes to `xs`, E3020: "this loop walks `xs` (line 5),
  and `hs[0]()` (line 6) may run the function made on line 3, which pushes to `xs`; call it after the loop, as many times as the loop would: write `n = xs.len()` before the loop, and `for k in 0..n: hs[0]()` below it instead of `hs[0]()` in it". (The phase before's `attacks2.exe 11` showed
  the use after free this refusal prevents.)
- v2's lines, `hs: Array[fn()] = []` and `hs.push(fn(): score += 1)`, refused in v4 for missing words
  (E2105, then E2113), are the NEW's own lines again (verdict 86).

**7. Cost.** None (the C passes an address).

**8. Kind:** a smarter compile-time solution (the list's end is proved; Rust needs the same proof through
lifetimes, and accepts this program).

---

## AN-59: A closure that gives a capture away gets E3013 with ten options instead of "declare the parameter `once fn()`"

**v5:** `call_once(fn() owns s: consume(s))` (verdict 83, the script's line); the messages of part 6
take the same spelling. Meaning unchanged.

**1. Intent:** `call_twice` runs a callback that hands a string to `consume`.

**2. OLD.** `old/an59_call_twice.em` (reader 4's `t13_o5.em`):

```ember
fn call_twice(f: fn()):
    f()
    f()

fn consume(owned s: String):
    println(s)

fn main():
    s = "a".to_string()
    call_twice(fn() => consume(s))
```

```text
error[E3013]: cannot move out of a reference
  --> an59_call_twice.em:10:32
   |
10 |     call_twice(fn() => consume(s))
   |                                ^ moved out here
   |
   = help: borrow it, `ref` (`ref mut` to change it where it is); move it out where its owner is held mutably, with `mem.take`, `mem.replace` or `swap`, or `pop`, `swap_remove`, `remove` or `drain` from a container; or copy it, `.clone()`, leaving the owner's value as it is
   = note: moving out of a `ref`/`ref mut` is forbidden [EXP-6]; the referent still owns the value and drops it

error: could not compile due to 1 error(s)
exit=1
```

**3. NEW.**

```ember
fn call_once(owns f: once fn()):
    f()

fn call_twice(f: fn()):
    f()
    f()

fn consume(owns s: String):
    println(s)

fn main():
    s = String("a")
    call_once(fn() owns s: consume(s))      # the function owns `s` and gives it away: once
    t = String("b")
    call_twice(fn(): consume(copy of t))         # each run hands over a new copy
```

`consume` is written out (the OLD's, with `owns`). The arguments of `call_once`, `call_twice` and `consume` are hand-overs to `owns`
parameters or plain reads, so no call writes `ref mut` (R4). `call_twice` calls `f` through a read-only
parameter: allowed, since its argument changes nothing of its own state (R12 refuses only a function
value that changes its own state given to a parameter that only reads).

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `f` in `call_once` | `owns`: handed over to `call_once`; `once` in its type: a function the call uses up | it is used up by the call |
| `f` in `call_twice` | none (default: a read-only function value; calling it changes only what its body changes, R12) | no |
| `s` in `consume` | `owns`: handed over to `consume` | its owner may |
| `s` in `main` | none (default: `main`'s own String) | its owner may |
| `s` in the first lambda | `owns`: handed over to the function | its owner may (it hands it to `consume`) |
| `t` in `main` | none (default: `main`'s own String) | its owner may |
| `t` in the second lambda | none (row 6: the real `t`, read); `copy of` before it makes the value handed over | no (read only) |

**5. Means exactly.** `means/m59_once.em` (`call_once(f: once fn())` with `owned fn() => consume(s)`)
and `means/m59_copy_each_time.em` (`call_twice(fn() => consume(s.clone()))`):

```text
=== m59_copy_each_time.em
exit=0
=== m59_once.em
exit=0
```

and `call_once` with `owned f: once fn()` (`target/redesign3/19-fixer/m59_owned_once.em`,
`call_once(owned fn() => consume(s))`):

```text
===== m59_owned_once.em
exit=0
```

**6. Still refused.**
- `call_twice(fn() owns s: consume(s))`, E3013: "this function gives `s` away, so it can run only
  once, and `call_twice` runs it twice; hand over a copy each time instead: `call_twice(fn():
  consume(copy of s))`, or, to hand over `s` itself, run it once: `call_once(fn() owns s:
  consume(s))`" (rules section 3's message, with the second way out this program has, `call_once`).
- `fn call_once(f: once fn()):`, E3013: "`call_once` uses `f` up, so it must hold it: write
  `owns f: once fn()`" (R12).
- `call_twice(fn(): consume(s))` (no word), E3013: "`consume(s)` would hand over `main`'s `s` from inside
  a function that `call_twice` may run twice, and the function does not own `s`; hand over a copy each
  time: `call_twice(fn(): consume(copy of s))`, or, to hand over `s` itself, run it once:
  `call_once(fn() owns s: consume(s))`" (`V4/safety/check-V5.md`'s fix, with the second way out).

**7. Cost.** None: the same code as today's `once fn` and `.clone()`.

**8. Kind:** a readable spelling of Rust's guarantee (`FnOnce`), with the message the annoyance asks for.

---

## AN-60: A callback receiving the object that holds it, and a handler that removes itself (A32)

**v5:** the methods say nothing about what they change (verdict 86: `fn fire():`,
`fn remove_current():`), and neither do the lists (`UiButton`'s `handlers: Array[fn()]`, `fn
on_click(owns f: fn()):`); `bump_ui = fn() weakref ui:` (verdicts 83, 86). With no word in any type,
`watch` could be given AN-53's `Button` too; G-05's handler, stored in a list of read-only `fn(Ui)`
handlers, is refused at the `for` walk (E3020) instead of at the store (E2113, gone); E2105 for
`bump_ui` and the `Button` E2113 go with the words; the first E3020 names the chain (`add_once`).
**Meaning changed:** yes, for one refused program (verdict 86): G-05's handler kept in a `fn(Ui)` list
compiles where nothing walks that list with `for`, and the `for` walk is refused (E3020). The NEW does
what it did.

**1. Intent:** the UI calls each handler with the UI; a handler may add handlers (they run in the same
dispatch, as in Python) or remove itself.

**2. OLD.** The Python spelling (`old/an60_fire_for.em`, last run's `a32_fire_python.em`):

```ember
class Ui:
    handlers: Array[fn(Ui)] = []
    clicks: int = 0

    fn fire(self):
        for h in self.handlers:
            h(self)
```

```text
error[E3021]: cannot write through a shared reference
 --> an60_fire_for.em:8:13
  |
8 |             h(self)
  |             ^^^^^^^ this is `ref fn(Ui) -> void`, which only reads
  |
  = help: a callable value may change what it captured, so it is called from a mutable place [CLO-2]: borrow it as `ref mut`, and iterate a collection of them with `.iter_mut()` (`for f in fs.iter_mut():`)
  = note: while a shared borrow is live the owner may read and copy but not write [BRW-1]

error: could not compile due to 1 error(s)
exit=1
```

Following the help (`old/an60_fire_itermut.em`):

```text
error[E3080]: `self.handlers` cannot be read while it is mutably borrowed
 --> an60_fire_itermut.em:8:15
  |
7 |         for h in self.handlers.iter_mut():
  |                  ------------- mutable borrow of `self.handlers` starts here
8 |             h(self)
  |               ^^^^ conflicting access here
  |
  = help: bind the borrow of `self.handlers` to a local and finish with it before this line, or copy the value out first
  = note: a borrow lasts until its last use, not to the end of the scope (BRW-2)

error: could not compile due to 1 error(s)
exit=1
```

**3. NEW.**

```ember
class Ui:
    handlers: Array[fn(ref mut Ui)] = []
    current: int = 0
    clicks: int = 0

    fn fire():
        this.current = 0
        while this.current < len(this.handlers):
            this.handlers[this.current](ref mut this)   # the handler may add or remove handlers
            this.current += 1

    fn remove_current():
        this.handlers.remove(this.current)
        this.current -= 1

fn count(ref mut ui: Ui):
    ui.clicks += 1

fn add_once(ref mut ui: Ui):                       # adds a handler, then removes itself
    ui.handlers.push(count)
    ui.remove_current()

class UiButton:                                    # a button whose handlers may change a Ui
    handlers: Array[fn()] = []

    fn on_click(owns f: fn()):
        this.handlers.push(f)

fn watch(ref mut ui: Ui, ref mut b: UiButton):
    bump_ui = fn() weakref ui:     # a link that does not keep the Ui alive
        if ui exists as ref mut me:                # the Ui still lives: call it `me`, which may change it
            me.clicks += 1
    b.on_click(bump_ui)
```

Python's list iterator is this loop (a position checked against the length at every step), so a handler
added during the dispatch runs in it and a handler that removes itself makes the next one move into its
place, as in Python.

The opening `if ui exists as ref mut me:` (R13, verdict 26) and `weakref ui` (verdicts 9, 83) are the
owner's words. `fire` gives the whole Ui to a handler (`ref mut this`), which may change any field; a
method may change its own object, and what `fire` changes is worked out from its body (R5, verdict 86).
The call says `ref mut this` (R4: the handler's parameter is `ref mut Ui`); it copies the handler's two
words out of the list first, so the list is not an argument of the call (R12) and nothing points into it
while the handler runs. The handlers' type `fn(ref mut Ui)` says nothing about what they change; what
each changes is worked out (R12), here through its `ref mut` parameter.

`watch`: `bump_ui` changes a `Ui` through a link it holds (`weakref ui`), which the compiler sees in its
body (R12, verdict 86). `UiButton` keeps it in a plain `Array[fn()]`; AN-53's `Button` would keep it too,
since no list says what its functions change. `b.on_click(bump_ui)` stores the handler, which is not a
change of what it changes (R12); a later `b.click()` is.

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `handlers` (in `Ui`) | none (default: the Ui's own list of function values) | through a name that may change the Ui (worked out, R5) |
| element type `fn(ref mut Ui)` | `ref mut` on its parameter: a handler gets the caller's Ui | yes, the Ui (`ref mut`) |
| `current`, `clicks` | none (default: the Ui's own `int`s) | through a name that may change the Ui (worked out, R5) |
| `this` in `fire` | `this`: the Ui `fire` was called on | none written (worked out: `fire`'s body changes `this.current` and gives the whole Ui to each handler, `ref mut this`, R5, verdict 86) |
| `this` in `remove_current` | `this` | none written (worked out: its body changes `this.handlers` and `this.current`) |
| `ui` in `count` | `ref mut`: the caller's Ui | yes |
| `ui` in `add_once` | `ref mut`: the caller's Ui | yes |
| `handlers` (in `UiButton`) | none (default: the button's own list); its type `fn()` says nothing about what its functions change (worked out, R12) | through a name that may change the button (`on_click`, by its body) |
| `this` in `on_click` | `this` | none written (worked out: its body pushes to `this.handlers`) |
| `f` | `owns`: handed over to `on_click` | its new owner may; it hands it on to the list |
| `ui` in `watch` | `ref mut`: the caller's Ui | yes |
| `b` in `watch` | `ref mut`: the caller's UiButton | yes (its `on_click` changes `handlers`) |
| `bump_ui` | none (default: `watch`'s own function value, made by `=`); what it changes, a `Ui` through its link, is worked out (R12) | its owner may; handed over to `on_click` with no word (an `owns` parameter) |
| `ui` in the `weakref ui` lambda | `weakref`: a link that does not keep the Ui alive | no (a link); `me` may (below) |
| `me` | `ref mut` on the opened name: the Ui the link points at, which it may change | yes |

**5. Means exactly.** Today's Ember accepts only the unsound form (AN-61), so the translation is C:
`c/an61_dispatch.c` mode 2, the code R12 gives `fire`: a handler that pushes 8 handlers (the list's
buffer moves) and then removes itself and reads its own state:

```c
for (u.current = 0; u.current < u.len && u.current < 12; u.current++) {
    Callable c = u.ptr[u.current];             /* two words copied out; no pointer into the list */
    c.fn(c.st, &u);
}
...
static void once_body(Box *st, Ui *u) {
    st->counter += st->m;
    for (int k = 0; k < 8; k++) push(u, (Callable){count_body, box_new(1)});   /* the buffer moves */
    if (g_mode == 2) st->rc++;                     /* hold: the state is used after a call that may drop it */
    remove_current(u);                             /* removes this very handler: drops the list's count */
    u->total += st->counter;                       /* uses its own state after the removal */
    if (g_mode == 2) box_release(st);
}
```

Under ASan (`c/an61_dispatch_output.txt`):

```text
--- mode 2
mode2 clicks=8 total=5
an61_dispatch ms=0.069 peak_kb=8232 checksum=2
```

The 8 handlers added during the dispatch ran in it (`clicks=8`), as in Python; no ASan report. Mode 3,
the same without the hold, proves the hold is needed:

```text
--- mode 3
=================================================================
==79044==ERROR: AddressSanitizer: heap-use-after-free on address 0x12032f7a0798 at pc 0x7ff7c24f36d6 bp 0x00c7c17ff550 sp 0x00c7c17ff598
READ of size 8 at 0x12032f7a0798 thread T0
    #0 0x7ff7c24f36d5 in once_body C:\Users\ism19\Code\ember\target\redesign3\11-area-D\c\an61_dispatch.c:93
```

(line 93 is `u->total += st->counter;`, the handler reading its own state after removing itself.) This C
keeps the handler's state in a separate counted block (`box_new`, run 3's box); R12 makes no box: what a
handler keeps with a word (`owns`, `copy of`) is in its own state block, which run 3's W4
measured (part 7). `watch`'s handler is the
`weakref` read of AN-65 (`c/an63_links.c` part 2: a counted link, read with a test of the header).

**6. Still refused.**
- `fire` with `for ref mut h in this.handlers:` (line 7) and `h(ref mut this)` (line 8), with
  `fn(ref mut Ui)` handlers (the agreed limitation: a list may not change while a loop walks it), E3020:
  "`h(ref mut this)` (line 8) may add to and remove from `this.handlers` (through `add_once`, which
  pushes a handler and calls `remove_current`) while this loop walks `this.handlers` (line 7); walk by
  position, as Python's list does: instead of the `for` line and `h(ref mut this)`, write
  `this.current = 0`, `while this.current < len(this.handlers):`, `this.handlers[this.current](ref mut
  this)` and `this.current += 1`" (R8 (c), R12; the rules' message in this program's names).
- Read-only handlers walked with `for` (`handlers: Array[fn(Ui)]`, `for ref mut h in this.handlers:
  h(this)`) compile where no handler the program stores in that list changes a Ui's `handlers` (R8 (c),
  R12). G-05's handler, which captures `weakref ui` and pushes to `me.handlers` inside
  `if ui exists as ref mut me:`, stored there, makes that walk refused, E3020: "`h(this)` (line 8) may
  add to `this.handlers` (through G-05's handler, which pushes to `me.handlers` of the Ui its link `ui`
  names, this one included) while this loop walks `this.handlers` (line 7); hand each handler the Ui and
  walk by position: write that handler as `fn(ref mut me: Ui):` with no `weakref ui` and no `if ui exists
  as ref mut me:` line (its block moved out one level), `handlers: Array[fn(ref mut Ui)] = []` instead
  of the `handlers` line, and `i = 0`, `while i < len(this.handlers):`, `this.handlers[i](ref mut this)`
  and `i += 1` instead of the `for` line and `h(this)`" (R18: every line it replaces; a local `i`, since
  the G-05 program's `Ui` need not have AN-60's `current`). v4 refused the store itself (E2113, the
  list's type said `fn(Ui)`); since verdict 86 no type says it, and the refusal is at the walk, the line
  that would break.
- `if Some(ref mut me) = ui:`, E0115: "an `Option` or a `weakref` link is opened with `exists as`: write
  `if ui exists as ref mut me:`" (R13).
- Not refused any more (verdict 86): `watch(ref mut ui: Ui, ref mut b: Button)` with AN-53's `Button`
  (v4's E2113 asked for a list that names `Ui`), and `bump_ui` with no word for what it changes (v4's
  E2105).
- Not refused, warned: a handler that captures the Ui that holds it with `shares ui` makes a cycle of
  owners that is never freed (a leak, not a memory error, as in Rust); L3001: "the handler is kept by
  `ui` and keeps `ui`; capture it as a link that does not keep it alive, `fn() weakref ui:`, or take the
  Ui as the handler's parameter" (rules section 3).

**7. Cost.** A call copies two words out of the list (plain C copies the same `{fn, env}` pair). A
handler's body holds one count on its own state only across a call that may remove it and only when it
uses its state after it (a rare path). The phase before measured this dispatch at 1.045 and 1.037 of
plain C doing exactly the same work with the same allocator, safe Rust at 1.490 and 1.472
(`target/redesign2/graphs-callbacks-attack2/bench/runs/w4_a.txt`, `w4_b.txt`;
`phase3b-graphs-callbacks-attack.md` 6.4). Run 3's W4, this program's workload: 1.091x of plain C with a
captured variable in the handler's own state block (R12's placement here), 1.255x with a box per
captured variable (no longer made, verdict 80), safe Rust 1.358x (`RUN3/26-timer.md` lines 29, 31, 32).
The `weakref` capture: one add of the Ui's weak count when `bump_ui` is made, after one compare with
the count's maximum (verdict 59; not measured), one subtract when it ends, and one header test per run
(R11).

**8. Kind:** a smarter compile-time solution (where each count is needed is decided when compiling; which
handlers may change a Ui is read from their type; the `for` refusal is the agreed limitation).

---

## AN-61: Unsound today: a handler list dispatched by index is accepted and writes freed memory

**v5:** `fire` says nothing about what it changes (verdict 86). R4's call check (E3031) reads every
handler the program stores (verdict 53's corrected record): `add_once` pushes and removes handlers
through the Ui it is given, but ends and moves nothing that `h(ref mut this)` hands it, so the call has
nothing to refuse. The E3060 message's line, `h = ref mut this.handlers.get_mut(i).unwrap()`, gives `h`
another ref, which compiles (verdict 42). Meaning unchanged.

**1. Intent:** dispatch handlers by position so each may change the list.

**2. OLD.** `old/an61_fire_index.em` (last run's `a32_fire_index.em`):

```ember
    fn fire(self):
        i = 0
        while i < self.handlers.len():
            h = self.handlers.get_mut(i).unwrap()
            h(self)
            i += 1
```

and `old/an61_remove_self.em` (a handler removes itself through `b.remove_current()`):

```text
=== an61_fire_index.em
exit=0
=== an61_remove_self.em
exit=0
```

Both accepted. The C that today's compiler emits for them keeps `h`, a pointer into the list's buffer,
across the call, and its trampoline copies the closure's state back to that address after the call
(last run's reading of the emitted C, `probes/a32_emit_uaf/c/release/c/main.c` lines 131-137, 233, 251;
`phase3b-graphs-callbacks.md` section 10; confirmed by the second attack, its section 9). The same shape,
written in C and run: `c/an61_dispatch.c` mode 1 (a handler pushes 8 handlers, so the buffer moves):

```text
--- mode 1
=================================================================
==43052==ERROR: AddressSanitizer: heap-use-after-free on address 0x11e85f1a0108 at pc 0x7ff90170b4dc bp 0x003b3fbeec50 sp 0x003b3fbeec98
WRITE of size 24 at 0x11e85f1a0108 thread T0
    #0 0x7ff90170b4db in _asan_memcpy+0x3db (C:\Program Files\LLVM\lib\clang\22\lib\windows\clang_rt.asan_dynamic-x86_64.dll+0x18004b4db)
    #1 0x7ff7c24f2bc6 in main C:\Users\ism19\Code\ember\target\redesign3\11-area-D\c\an61_dispatch.c:111
```

(the write is the trampoline's copy back, inlined into `main`; `c/an61_dispatch_output.txt` has the
whole report, with where the block was freed: the growth in `push1`).

**3. NEW.** The position call of AN-60 (in AN-60's `Ui`, `handlers: Array[fn(ref mut Ui)]`):

```ember
    fn fire():
        i = 0
        while i < len(this.handlers):
            this.handlers[i](ref mut this)
            i += 1
```

and (v2's change from run 4) the OLD's own shape, accepted once the name says `ref mut`:

```ember
    fn fire():
        i = 0
        while i < len(this.handlers):
            h = ref mut this.handlers.get_mut(i).unwrap()   # the only name for this handler while used
            h(ref mut this)                                 # the call first copies its two words out
            i += 1
```

Changed from run 4: the second form is now accepted (run 4 refused it, E3022); since verdict 86 `fire`
says nothing about what it changes (run 4's `ref mut this` after the brackets, v4's `changes this`). A
call through a `ref mut` name of a stored function value uses the name only to copy the function's two words out, before the arguments are given, so `ref mut this`, which
reaches `h`'s place, may be given to it (R8 (f), R12; the `get_mut` row of `V5/vocabulary.md`). Nothing is
written back into the list after the call (the handler's state is behind the function value's own
pointer, R12), so a handler may grow or shrink the list while it runs. `h` is a position (R8 (a)); it is
not used after the call, and a use there would be checked against what the call may have done (part 6).

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `this` | `this`: the Ui `fire` was called on | none written (worked out: its body gives the whole Ui to each handler, `ref mut this`, R5, verdict 86) |
| `i` | none (default: `fire`'s own `int`) | its owner may |
| `h` (second form) | `ref mut`: the handler at `i`, the only name for it while used (R8 (f)) | yes; calling it changes what its body changes (R12) |

(`this.handlers[i]` in the first form declares no name: its two words are read and called, and nothing
points into the list during the call, R12.)

**5. Means exactly.** `c/an61_dispatch.c` mode 2, pasted under AN-60: clean under ASan. Both forms are
that code: the handler's two words copied out of the list, then called.

**6. Still refused.**
- A plain `h = this.handlers.get_mut(i).unwrap()` then `h(ref mut this)`: `h` is then a read-only view
  (R2), and calling a stored handler through it is refused, E3023: "`h` only reads the handler, and
  calling it may change its own state; call it by position: `this.handlers[i](ref mut this)`, or name it
  `h = ref mut this.handlers.get_mut(i).unwrap()`" (R12; the `get_mut` row).
- `h` used again after `h(ref mut this)` (say `h(ref mut this)` twice), E3060: "`h(ref mut this)` (line
  5) lets the handler remove handlers, which may put another handler in the place `h` names (line 4),
  and `h` is used on line 6; name it again after the call: `h = ref mut this.handlers.get_mut(i).unwrap()`"
  (R8 (a); a `ref mut` name may be given another ref, verdict 42).

**7. Cost.** As AN-60. The refused form today (a pointer into the list kept across the call and written
back to) had no cheaper code: the copy back it needs is what made it unsafe.

**8. Kind:** a smarter compile-time solution (the hole is closed when compiling; the forms that work cost
what C costs).

---

## AN-62: A loop calling a stored callback with unknown effects pays a count pair per step (the phase before's limit)

**v5:** the NEW is v4's walk, line for line, with `on_visit: fn(Node)` in both forms (verdict 86): which
walk a program writes is decided by the functions it stores in `on_visit`, which the compiler reads (R11,
R12): (a) where every one only reads its node, (b), the walk the owner ruled in verdict 28, where one
may change or end nodes. Verdict 53's corrected record settles step 3's question 1 (worklist F-06):
`this.on_visit(p)` in (b) compiles, since no stored function uses its node after ending it, and a
function that does is refused where it is stored (E3031, a new refusal in part 6). The E2105 and E2113
refusals that came from the field's type and `depth_total`'s first line are gone; E3060's way out opens
the link under a new name (F-01); the comments on `on_visit` say what the program stores. **Meaning
changed:** yes (verdicts 53, 86): `this.on_visit(p)`, which v4's E3031 as written refused, compiles; a
function that may change nodes, stored in `on_visit`, is no longer refused at the store (E2113), and
(a)'s plain walk is refused instead (E3021), where it would break. The NEW does what it did.

**1. Intent:** walk from each leaf up to the root and report every ancestor to a stored callback.

**2. OLD.** Today's spelling (`old/an62_walk_callback.em`):

```ember
class Node:
    value: int = 0
    children: Array[Node] = []
    parent: Weak[Node] = Weak.empty()

class Tree:
    nodes: Array[Node] = []
    on_visit: fn(Node) = fn(n: Node): pass

    fn depth_total(self) -> int:
        depth = 0
        for n in self.nodes:
            a = n.parent.upgrade()
            while a is not None:
                p = a.unwrap()
                self.on_visit(p)
                depth += 1
                a = p.parent.upgrade()
        return depth
```

```text
warning[L3001]: potential reference cycle: Node.children -> Node
 --> an62_walk_callback.em:4:5
  |
4 |     children: Array[Node] = []
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^ this strong field closes the shortest statically visible cycle
  |
  = help: if `Node.children` is a back-reference, make it hold a `Weak` handle
  = note: every edge in this statically visible cycle is strong; the cycle may leak at run time

exit=0
```

Accepted; every `upgrade()` is a count pair (today's `Weak`, `[WK-*]`). In last run's model the same
walk paid a count pair per step because the callback's effects were unknown: 1.27x to 1.32x of the
same walk without it (`phase3b-graphs-callbacks-attack.md` 3.5, `runs/w2v_a.txt`, `w2v_b.txt`);
`FINDINGS-phase3b.md` section 5 item 6.

**3. NEW.** A callback's effects are no longer unknown: the compiler reads every function the program
stores in `on_visit` (R12; verdicts 53 and 86). Where every one only looks at its node, the walk is (a);
where one may change or remove nodes (`t.on_visit = prune`, with `prune` taking nodes out of their
parents' lists), it is (b). The field is `on_visit: fn(Node)` in both: its type says nothing about what
its functions change. In both, a parent link is an ordinary `weakref` field the program sets (verdict
14).

(a) No function the program stores in `on_visit` changes a node:

```ember
class Node:
    value: int = 0
    parent: weakref Node = None
    children: Array[Node] = []

    fn add(owns child: Node):
        child.parent = weakref this            # the link back, which the program sets
        this.children.push(child)              # then the child is put where its parent owns it

class Tree:
    root: Node
    leaves: Array[weakref Node] = []
    on_visit: fn(Node)                         # every function stored here only reads its node

    fn depth_total() -> int:
        depth = 0
        for leaf in this.leaves:
            if leaf exists as n:               # the leaf still lives: call it `n`
                cur = ref n                    # a view of that node
                while cur.parent exists as p:  # its parent, while there is one
                    this.on_visit(p)           # changes no node: no function stored in `on_visit` does
                    depth += 1
                    cur = ref p                # `cur` now views the parent
        return depth
```

(b) A function the program stores in `on_visit` may change nodes, and so remove and end them (`Node`
as in (a)):

```ember
class Tree:
    root: Node
    leaves: Array[weakref Node] = []
    on_visit: fn(Node)                         # a function stored here may change nodes, remove them included

    fn depth_total() -> int:
        depth = 0
        for leaf in this.leaves:
            up = weakref leaf                  # a link to the leaf, kept across the calls
            while up exists as n:              # the node still lives: call it `n`
                if n.parent exists as p:       # its parent, if it has one
                    up = weakref p             # the link now names the parent
                    this.on_visit(p)           # may end `p`, or a node above it with all under it
                    depth += 1
                else:
                    break
        return depth
```

v2's change from run 4 (verdicts 14, 15, 17, 5): run 4's parent link was set by the language when a
child was pushed; verdict 14 removed that, and `add` sets the link (`child.parent = weakref this`) before it puts the child where
its parent owns it (`this.children.push(child)`); making the link changes only `this`'s weak count, a
number in its header no name reads, so it is not a change of `this` (R11). What `add` and `depth_total`
change is worked out from their bodies (R5, verdict 86): in (b), `depth_total` makes the changes of the
functions stored in `on_visit`, nodes ended included (R12).

What (a) costs, by the rules: no function the program stores in `on_visit` changes a node (R12), so no
line in the walk can end a `Node` (R10 point 1): the views `n`, `cur` and `p` need no protection (no
region, no count pair). The parent read `cur.parent` needs no header test (R11): (1) the link follows ownership in this package (the
only line that gives `parent` a target is in `add`, followed by the `push` into the parent's own list;
nothing takes a node out of its list; no `children` list is handed over or replaced whole; nodes have one
owner and `Node` has no `drop` method; the fields are not `pub`), and (2) nothing in the walk can end a
node. So each step is a plain load of `cur.parent` and a test for no link. The one header test is
`if leaf exists as n:`, once per leaf (`leaves` holds ordinary links, not parent links). `cur = ref n` and
`cur = ref p` say what `cur` is, a read-only view (R1); `=` on `cur` gives it a new view (R2).

What (b) costs, and why it is written with a link: across a call that may change a `Node`, no read-only
view of a node lives (R8 (g), the owner's verdict 5; R10's last paragraph, R12), so (a)'s loop with (b)'s
stored function is refused (part 6), and the walk keeps a `weakref` link instead (`up`), which is not a
view and may outlive its target (verdict 28). `n` and `p` are views, but neither lives across the call:
`n`'s last use is `n.parent`, and `p`'s is the call itself, whose argument it is (a view whose last use
is an argument of the call does not live across it, R8 (g); inside the call the parameter makes the
promise). `up = weakref p` makes a link from the read-only `p`, which changes no node (R11). Each step
opens the link (`while up exists as n:`, one header test), reads `n.parent` with no header test (R11:
`n` is known to live at the test, and nothing between the test and the read can end a node), and moves
the link to the parent before the call. If the callback ends `p`, or a node above it with everything
under it, the link's target is ended and its block is kept, so the next `while up exists as n:` finds it
ended and the walk stops: it never visits an ended node, and nothing stops the program (verdicts 15,
28). The call `this.on_visit(p)` itself compiles: no function the program stores in `on_visit` uses its
node after a line that may end it (R4, R11; verdict 53's corrected record), and one that does is refused
where it is stored (E3031, part 6). If the callback moves `p` under another parent (taking it out and
giving it a new link, as AN-63's `take` and `add`), the walk goes on up the new parents.

What keeps the ended node's block until the test, by R11: every `weakref` link holds a weak count on
its target, so `up = weakref p` is one subtract on the node left and one add on its parent; and, since
`Node` has no `drop` method of its own, the compiler places the walk in a deferred-drop region (R11,
section 0), which keeps every block it sets aside until the region ends. A link that lives only inside
the region and is given only objects that live at that moment (`up = weakref p`, from the view `p`)
then holds no weak count: the region keeps its target's block. Only the first link of each walk,
`up = weakref leaf` (made from another link, whose target may already be gone), keeps its count. So
each step pays one header test, and nothing else (part 7). This never changes what the program does
(R17): with or without the region the walk stops at the same node.

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `value` | none (default: the node's own `int`) | through a name that may change the node (worked out, R5) |
| `parent` | `weakref`: a link that does not keep its node alive | through a name that may change the node: `add` sets `child.parent` on the node it was handed |
| `children` | none (default: the node's own list; each element has this one owner) | through a name that may change the node (worked out, R5) |
| `this` in `add` | `this`: the node `add` was called on | none written (worked out: its body changes `this.children`; `weakref this` makes a link to it and changes no field, R11) |
| `child` | `owns`: handed over to `add` | its owner may (it sets `child.parent`, then hands it to the list) |
| `root` | none (default: the tree's own Node) | through a name that may change the tree (worked out, R5) |
| `leaves` | `weakref` on its elements: links that do not keep the leaves alive | through a name that may change the tree |
| `on_visit` in (a) | none (default: the tree's own function value); every function the program stores there only reads its node (worked out, R12) | through a name that may change the tree |
| `on_visit` in (b) | none (default: the tree's own function value); a function the program stores there may change nodes (`prune`; worked out, R12) | through a name that may change the tree |
| `this` in `depth_total` | `this`: the Tree `depth_total` was called on | none written (worked out: it runs the stored function, R12; in (b) that may change and end nodes) |
| `depth` | none (default: the method's own `int`) | its owner may |
| `leaf` | none (default: a read-only view of an element of `leaves`, a `weakref` link) | no |
| `n` in (a) | none (default for an opened name: a read-only view of the node `leaf` links to) | no |
| `cur` | `ref`: a read-only view of a node (first `n`, then each parent) | no, not the node; `=` points it at another node (R2) |
| `p` in (a) | none (default for an opened name: a read-only view of `cur`'s parent) | no |
| `up` in (b) | `weakref`: the method's own link to a node, which does not keep it alive | not the node; `=` gives the link another target (R2) |
| `n` in (b) | none (default for an opened name: a read-only view of the node `up` links to, until its last use, `n.parent`) | no |
| `p` in (b) | none (default for an opened name: a read-only view of `n`'s parent, until its last use, the call) | no |

**5. Means exactly.** (a) is the walk of run 3's workload with no protection and no header test
(`c/an62_walk_bench.c` built `-DSLAB`, `walk_noprot`): a plain pointer read per step and a call through
a function pointer the C compiler cannot see (timings in part 7). Run 3's `c/an62_defer_asan.c` shows
what such a walk does when the callback may end nodes (a callback that takes `mid` out of the tree and
drops its last owner while the walk is on `mid`), the case (b)'s refusal is for. Three modes, under ASan
(`c/an62_defer_asan_output.txt`):

```text
--- mode 1
=================================================================
==71988==ERROR: AddressSanitizer: heap-use-after-free on address 0x1220ea4a0918 at pc 0x7ff6579d3211 bp 0x004fe739fa00 sp 0x004fe739fa48
READ of size 8 at 0x1220ea4a0918 thread T0
    #0 0x7ff6579d3210 in main C:\Users\ism19\Code\ember\target\redesign3\11-area-D\c\an62_defer_asan.c:89
...
--- mode 2
mode2 depth=1 leaf.parent=None
an62_defer_asan ms=0.075 peak_kb=8268 checksum=1
--- mode 3
mode3 depth=1 leaf.parent=None
an62_defer_asan ms=0.060 peak_kb=8280 checksum=1
```

Mode 1 (no protection) reads `p.parent` from freed memory (line 89, `Node *next = a->parent;`); mode 2 (a
deferred-drop region) and mode 3 (a count pair) are clean. Neither is (b)'s code: under verdict 5 a view
of a node does not live across a call that may change nodes, so (b) keeps a link.

(b), as v2's rules gave it, is `c\an62_an63_v2.c` part 2 (v2's step, on the owner's laptop,
`target/redesign4/v2-05-area-D/c/`, built `clang -O1 -g -fsanitize=address`): a tree `root`, `top`,
`mid`, `leaf`; the walk starts at `leaf`, and the callback, on `mid`, takes `top` out of `root` and ends
it, with `mid` and `leaf` under it. Mode 1 counts every link as R11 says; mode 2 is the same with
uncounted links (an ended node's block is freed at once). Output (`c\an62_an63_v2_output.txt`):

```text
--- mode 1
mode1 depth=1 visited=m root.children=0
an62b_v2 ms=0.070 peak_kb=8236 checksum=1
exit=0
--- mode 2
=================================================================
==51716==ERROR: AddressSanitizer: heap-use-after-free on address 0x1157c7aa08d4 at pc 0x7ff7b801362c bp 0x001e01cff980 sp 0x001e01cff9c8
READ of size 4 at 0x1157c7aa08d4 thread T0
    #0 0x7ff7b801362b in main C:\Users\ism19\Code\ember\target\redesign4\v2-05-area-D\c\an62_an63_v2.c:146
    #1 0x7ff7b801420e in __scrt_common_main_seh D:\a\_work\1\s\src\vctools\crt\vcstartup\src\startup\exe_common.inl:288
    #2 0x7ffa09a1dda6 in BaseThreadInitThunk+0x16 (C:\Windows\System32\KERNEL32.DLL+0x18002dda6)
    #3 0x7ffa0b35ee4b in RtlUserThreadStart+0x2b (C:\Windows\SYSTEM32\ntdll.dll+0x18007ee4b)

0x1157c7aa08d4 is located 4 bytes inside of 48-byte region [0x1157c7aa08d0,0x1157c7aa0900)
freed by thread T0 here:
    #0 0x7ff9358fc81f in _asan_wrap_memcpy+0x62f (C:\Program Files\LLVM\lib\clang\22\lib\windows\clang_rt.asan_dynamic-x86_64.dll+0x18004c81f)
    #1 0x7ff7b8012433 in free S:\compiler-rt\lib\asan\asan_malloc_win_thunk.cpp:52
exit=1
```

Mode 1: the walk visited `mid` once (`visited=m`), then `while up exists as n:` found `mid` ended and stopped
(`depth=1`); `top`, the ended node above, was never visited; no ASan report. Mode 2: the test itself reads
a freed block, so the weak count is what makes the link's test safe (or, since v3, the region, below).

The deferred-drop region's code, from run 3's bench (`c/bin/walk_noprot.s` against `c/bin/walk_defer.s`,
the whole difference apart from the program's name string and the counter's declaration):

```text
205a206
> 	incq	g_defer(%rip)
233a235
> 	decq	g_defer(%rip)
```

one increment before each pass's loop and one decrement after it; nothing inside the loop. (a) needs no
region (no call in it can end a node). Since v3 a region serves (b) in a second way (R11): it keeps the
block of every node set aside during the walk until the walk ends, so the walk's link needs no weak
count after its first target.

(b) in that region, checked in this step: `target/redesign4/v3-05-area-D/c/an62b_region.c`, the same
tree and visit function as v2's part 2 above (`root`, `top`, `mid`, `leaf`; the visit function, on
`mid`, takes `top` out of `root` and ends it, with `mid` and `leaf` under it). Mode 1 is v3's R11 (v4's is the same): a
region around the walk, the first link (`up = weakref leaf`) counted, every move after it uncounted; an
ended node set aside whole and freed when the region ends. Mode 2 is the same with no region (an ended
node freed at once) and no counts. Mode 3 has no region and counts every link (v2's R11). Built with
`gcc -O1 -g -fsanitize=address` (gcc 13.3, Linux, this cloud container), run with
`ASAN_OPTIONS=detect_leaks=1` (`c/an62b_region_output.txt`, cut after the first frames of mode 2's
report):

```text
--- mode 1
mode1 depth=1 visited=m root.children=0
exit=0
--- mode 2
=================================================================
==18989==ERROR: AddressSanitizer: heap-use-after-free on address 0x5060000000e8 at pc 0x558c62ba292e bp 0x7ffcbbd796e0 sp 0x7ffcbbd796d0
READ of size 4 at 0x5060000000e8 thread T0
    #0 0x558c62ba292d in main /home/user/ember/target/redesign4/v3-05-area-D/c/an62b_region.c:81
...
SUMMARY: AddressSanitizer: heap-use-after-free /home/user/ember/target/redesign4/v3-05-area-D/c/an62b_region.c:81 in main
exit=1
--- mode 3
mode3 depth=1 visited=m root.children=0
exit=0
```

Mode 1: the walk visited `mid` once and the next test found it ended (`depth=1`, `visited=m`), as with
counted links (mode 3, v2's mode 1 above); no ASan report and no leak report. Mode 2 shows what the region
or the count is for: line 81 is the header test, `while (up && !up->dead)`, reading `mid`'s freed
block.

**6. Still refused.**
- (a)'s loop (views `cur` and `p` kept across the call) in a program that stores `prune` in `on_visit`,
  the plain-`ref` walk verdict 28 refuses, E3021: "`this.on_visit(p)` (line 21) may end a `Node` (it may
  run `prune`, stored on line 40, which removes a node from its parent's list), and `p` reads a node and
  is used again on line 23; walk with a link, which may outlive its node: instead of `if leaf exists as
  n:`, `cur = ref n`, the `while` line and `cur = ref p`, write `up = weakref leaf`, then `while up exists
  as n:`, inside it `if n.parent exists as p:` with `up = weakref p` and `this.on_visit(p)`, and `else:
  break`" (R8 (g), R11). Lines 21 and 23 are those of (a)'s file, `Node` and `Tree` as shown, and line
  40 is the program's `t.on_visit = prune`. This is `V5/rules.md` section 3's E3021 row, word for word.
  The line it gives is (b)'s walk, which compiles under R11 and verdict 28; its `this.on_visit(p)`
  compiles too (verdict 53's corrected record: no function stored in `on_visit` uses its node after
  ending it).
- A function stored in `on_visit` that ends its node and then reads it, `prune`, whose body
  takes `n` out of its parent's `children` on line 41 and reads `n.value` on line 42, E3031
  at the line that stores it (verdict 53's corrected record; R4, R11): "`on_visit` is called with a node
  of the tree (line 21), and this function removes its node from the parent's list (line 41) and then
  reads it (line 42); read it before removing it: move line 42 above line 41" (rules section 3's message).
- A visit function stored in the tree it changes, `t.on_visit = fn(n: Node): t.root.children.clear()`,
  E3022: "this function clears `t.root.children`, and `t.on_visit = ...` keeps it in `t`, so it would
  change the tree that holds it while `depth_total` runs it; clear the children after the walk instead:
  `d = t.depth_total()`, then `t.root.children.clear()`" (R12: a function value is not stored in a place
  that a real variable it changes owns, as Rust refuses; this was v2's note 1, a walk whose views of
  `t`'s nodes would live across a call that ends them). v4's E2113 refusals of this line, by the field's
  type, are gone with the words (verdict 86).
- Not refused any more (verdict 86): storing a function that may change nodes in `on_visit`
  (`t.on_visit = prune`, v4's E2113 for (a)'s field); it makes `on_visit` a field whose functions may
  change nodes, so (a)'s walk is refused (above) and (b)'s is the one to write. v4's E2105 for (b)'s
  `depth_total` without `changes Node` is gone with the word.
- `while Some(p) = a:`, E0115: "an `Option` or a `weakref` link is opened with `exists as`: write
  `while a exists as p:`" (R13).
- Run 3's walk by copies of the link, `a = copy of n.parent`, E2116: "`copy of n.parent` would copy the
  link, not the node; for another link to the same node write `a = weakref n.parent`; `Node` has no
  `Clone`, so for a copy of the node itself write `@derive(Clone)` above `class Node`, or keep a view of
  it: `if n.parent exists as q:`" (R3, R11; rules section 3's message for a type with no `Clone`, since
  this `Node` has none, worklist section 4, item 2; where `Node` has `Clone`, the second way out is
  `if n.parent exists as q:` and then `c = copy of q`).
- A view into a `Node`'s contents (an element of `p.children`) kept across a call that may change it:
  refused by the loop rule when a loop walks it (E3020) and by R8 (b), (g) otherwise.
- Not refused, slower: a `Node` class with its own `drop` method. R11's proof then fails: a child's
  `drop` may open its `parent` link while the parent is being ended, and must then find it ended
  (verdict 56), which only the header test tells; so every `parent` read tests the header, and (b)'s
  walk gets no region, so its link keeps a weak count on every target, one subtract and one add per step
  (R11).
- A view of a node used after a call that may end nodes, with or without a `drop` method (`if leaf exists
  as n:`, then `prune(ref mut t)`, where `prune` takes nodes out of `t`, then `println(n.value)`), E3060
  (verdict 82): "`prune(ref mut t)` may end `Node` objects, and `n` reads one (`if leaf exists as n:`)
  and is used after it (`println(n.value)`); use `n` before `prune(ref mut t)`, or open the link again
  after it: write `if leaf exists as m:` above `println(n.value)`, indent that line under it and use `m`
  there" (worklist F-01: a new name, as the rules' E3060 message gives `q`; R10
  point 1; E3060 rather than E3021, since its fix compiles, R10's closing paragraph, verdict 73). v3
  kept the node alive with a count pair there; R10 no longer does.

**7. Cost.** (a): per step, a plain load of `cur.parent` and a test for no link; no protection; one
header test per leaf. Timed by run 3 with `target/redesign2/bench/run.ps1 -Rounds 5` (pinned to the fast
cores, mains checked each round; `c/walk_runs.txt`, medians in `c/walk_summary.txt`). Every run printed
checksum 722575054. The rows were measured with run 3's `set by` links (uncounted); the walk itself
touches no count, so (a)'s walk is the second row:

| program | median ms | ratio to plain C, same layout | spread |
|---|---|---|---|
| plain C, same layout (`walk_plain.exe`, `-DSLAB -DARRAY24`) | 1905.5 | 1.000 | 30% |
| (a): no protection, no header test (`walk_noprot.exe`) | 2068.5 | 1.086 | 29% |
| a count pair per step (`walk_retain.exe`, the phase before's rule) | 2444.0 | 1.283 | 34% |
| a deferred-drop region (`walk_defer.exe`; R11's walk, not needed in (a)) | 1797.9 | 0.944 | 37% |

**PROVISIONAL**: other agents were running; the spread (fastest to slowest) is 29% to 37%, far over the
10% the timer accepts, so these numbers rank nothing finely. Run 3's W2 (the same parent walk without a
callback) measured the read with no header test at 1.176x of plain C, against 1.579x for a header test
on every hop and 1.603x for safe Rust (`RUN3/26-timer.md` lines 20 to 22: `diag_w2_hop_proved`,
`model_w2`, `rust_w2`; `diag_w2_hop_proved` spread 14.8%). The parent links' weak counts are paid where
a link is made or ended (`add`, a node's end), not in the walk; they were not measured.

(b), by R11 (`Node` has no `drop` method): per step, one header test (`while up exists as n:`) and
the parent read with no header test; the link's moves after its first target (`up = weakref p`) touch
no count; per walk from a leaf, one add and one subtract of `leaf`'s weak count (the first link), the
add after one compare with the count's maximum (verdict 59); per region, one add and one subtract of the
per-thread counter (the two instructions above). Where the class
has a `drop` method, add one subtract and one add of weak counts per step (no region). **Not measured**
(this step ran in a cloud container, where no timing was taken; the C check in part 5 tests safety
only). The nearest numbers: a header test on every hop, 1.579x of plain C (`model_w2`, uncounted links,
the same work per step as (b) in a region), and safe Rust's walk, 1.603x (`rust_w2`), which upgrades a
`Weak` on every step (a count pair) (`RUN3/26-timer.md` lines 20 to 22). The owner accepted this cost
(verdict 28) and asked that the next run still look for proof that removes the rest; the region
removes the weak-count moves, and the header test stays because a function the program stores in
`on_visit` may end the very node the test is for (notes for the lead, note 3). Peak
memory: 84,544 KB for every variant of run 3's bench against 76,688 KB for plain C (the object header);
a node's header now holds its weak count too (R9). In (b) an ended node's block is kept until the region
ends (or, without one, until the walk's link moves on or ends).

**8. Kind:** (a) a smarter compile-time solution: what the program stores in `on_visit`, read when
compiling, and R11's proof remove every per-step check, as fast as the unprotected walk; (b) a readable spelling of Rust's guarantee (a link
kept across a call that may end its target, Rust's `Weak` on every step), with the walk stopping at the
first ended parent as verdicts 15 and 28 say, and a smarter compile-time saving (the region) that leaves
one header test per step where Rust pays a count pair.

---

## AN-63: Trees with parent pointers, graphs, cycles (A8)

**v5:** `add` and `take` say nothing about what they change (verdict 86: worked out from their bodies,
R5, the script's lines); the owner's-test table says so. Meaning unchanged.

**1. Intent:** a scene tree whose nodes know their parent.

**2. OLD.** The Python spelling (`old/an63_tree_python.em`, last run's `a8_tree_python.em`):

```ember
class Node:
    name: String
    children: Array[Node] = []
    parent: Option[Node] = None

    fn add(self, child: Node):
        child.parent = Some(self)
        self.children.push(child)
```

```text
warning[L3001]: potential reference cycle: Node.children -> Node
 --> an63_tree_python.em:4:5
  |
4 |     children: Array[Node] = []
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^ this strong field closes the shortest statically visible cycle
  |
  = help: if `Node.children` is a back-reference, make it hold a `Weak` handle
  = note: every edge in this statically visible cycle is strong; the cycle may leak at run time

exit=0
```

Accepted; every parent-child pair is a cycle of counted handles that leaks (`[WK-1]`). Today's working
spelling (`old/an63_tree_weak.em`: `parent: Weak[Node] = Weak.empty()`, `child.parent = Weak(self)`,
`match n.parent.upgrade():`) is accepted with the same warning, pasted in `old/check_output.txt`.

**3. NEW.**

```ember
class Node:
    name: String
    parent: weakref Node = None
    children: Array[Node] = []

    fn add(owns child: Node):
        child.parent = weakref this            # the link back, which the program sets
        this.children.push(child)              # then the child is put where its parent owns it

    fn take(i: int) -> Node:
        child = this.children.remove(i)        # out of its place, not ended
        child.parent = None                    # its link cleared before anything reads it
        return child

fn path(n: Node) -> String:
    if n.parent exists as p:                   # `n` has a parent: call it `p`
        return f"{path(p)}/{n.name}"
    return copy of n.name

fn main():
    root = Node(name="root")
    root.add(Node(name="a"))                         # a.parent is root
    root.children[0].add(Node(name="leaf"))          # leaf.parent is a
    println(path(root.children[0].children[0]))      # root/a/leaf
    moved = root.children[0].take(0)                 # leaf.parent is None
    root.add(moved)                                  # leaf.parent is root
```

v2's change from run 4 (verdict 14): in run 4 the language set and cleared the link when `push` and
`remove` ran. Verdict 14 removed that: the link is an ordinary `weakref` field, and the
program sets it, as Python does (`child.parent = weakref this`, then `this.children.push(child)`), and
clears it when it takes a node out (`child.parent = None`). `add` and `take` are the program's own
methods, not language words.

The link is read with no header test (a plain load and a test for no link) where the compiler proves its
target lives (R11): (1) **the link follows ownership** in this package: the only line that gives `parent`
a target, in `add`, is followed by `this.children.push(child)`, which puts the node into a list its new
parent owns alone; the only line that takes a node out without ending it, in `take`, is followed by
`child.parent = None`; no `children` list is handed over or replaced whole; nodes have one owner and
`Node` has no `drop` method; the fields are not `pub`, so no other package changes them. (2) **nothing
ends a node meanwhile**: `path`'s `n` is a parameter, which lives for the call, and nothing in `path` can
end a node (its call to `path` only reads). So `n.parent` is a plain load. Where (1) fails anywhere in the
package, every read of `parent` tests the target's header: the program stays safe and gets slower (part
7). Lines that make (1) fail: a link given to a node that the target does not then own
(`leaf.parent = weakref other`), a node taken out of a list with no line that clears or re-points its
link before it is read, a list of children handed over whole (`a.children = b.children`).

`return copy of n.name` makes the returned String (row 9's word; `n.name` is a part of the caller's node).
For links that are not the reverse of an owning field (a camera's target, an observer list), a `weakref`
field the same way, read with a test each time: `target: weakref Player = None`,
`observers: Array[weakref Observer]`. For a graph, each city's roads and the roads into it:
`roads: Array[weakref City]` and `into: Array[weakref City]`, both filled by the program when it adds a
road (`a.roads.push(weakref b)`, then `b.into.push(weakref a)`), the cities owned by one list; these are
not parent links, so each read tests.

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `name` | none (default: the node's own String) | through a name that may change the node (worked out, R5) |
| `parent` | `weakref`: a link that does not keep its node alive | through a name that may change the node: `add` and `take` set it on the node they hold |
| `children` | none (default: the node's own list; one owner per element) | through a name that may change the node (worked out, R5) |
| `this` in `add` | `this`: the node `add` was called on | none written (worked out: its body changes `this.children`; `weakref this` makes a link to it and changes no field, R11) |
| `child` in `add` | `owns`: handed over to `add` | its owner may (it sets `child.parent`, then hands it to the list) |
| `this` in `take` | `this` | none written (worked out: its body removes from `this.children`) |
| `i` | none (default: a read-only number) | no |
| `child` in `take` | none (default: `take`'s own Node, handed over by `remove`, R2) | its owner may (it clears `child.parent`, then hands it back) |
| `n` | none (default: a read-only view, a parameter) | no |
| `p` | none (default for an opened name: a read-only view of the parent, until its last use) | no |
| `root` | none (default: `main`'s own Node) | its owner may |
| `Node(name="a")`, `Node(name="leaf")` given to `add` | none (default: a new value handed over to an `owns` parameter, row 9's hand-over) | its new owner may |
| `moved` | none (default: `main`'s own Node, handed over by `take`) | its owner may; handed over to `add` |

**5. Means exactly.** C, `c\an62_an63_v2.c` part 1 (this step, `target/redesign4/v2-05-area-D/c/`): the
same steps with counted links (`add` sets the link and adds to the parent's weak count, `take` clears it
and subtracts), `path` reading `parent` as a plain load; then a list keeps a `weakref` link to `leaf` and
the root ends with its whole tree. Under ASan (`c\an62_an63_v2_output.txt`):

```text
--- mode 0
1 root/a/leaf
2 leaf
3 root/leaf
4 weak(leaf)=1 weak(root)=2
5 kept link: gone
an63_v2 ms=0.054 peak_kb=8204 checksum=0
exit=0
```

Line 4 shows the counts the links hold (one link to `leaf`, from the list; two to `root`, from `a` and
`leaf`); line 5 shows the kept link finding `leaf` ended after the root's end; `leaf`'s block was freed
when that link ended. No ASan report.

**6. Still refused.**
- `child.parent = Some(this)` with `parent: Option[Node]` (the Python spelling, a strong field), E3013:
  "`child.parent = Some(this)` would make `parent` a second owner of this node, and `this` is not yours
  to hand over; for a link back to the node whose `children` holds this one write
  `parent: weakref Node = None` and `child.parent = weakref this`".
- `child.parent = this` into the `weakref` field, E2020: "`=` alone would hand `this` over; write
  `child.parent = weakref this`".
- `struct Node` with a view of its parent, E2200: "`Node.parent` would be a view of the `Node` that holds
  this one, which no struct can keep; for a link back to the parent make `Node` a class with
  `children: Array[Node]` and `parent: weakref Node = None`, set with `child.parent = weakref this`
  before `this.children.push(child)`" (area C's AN-41 message).
- Not refused, slower: a program whose lines break R11's condition (1) (above) is accepted, and every
  `parent` read in the package tests the header (part 7).
- A cycle of `shares` owners (`a: shares Node` holding `b` holding `a`) is not refused (it is a leak, not
  a memory error, as in Rust); AN-64's warning names it.

**7. Cost.** A link made or changed: one store and one add or subtract of a weak count (`add`: the
parent's count +1; `take`: -1; a node's end: -1 on its parent), as Rust's `Weak::clone` and its drop
(R11); before each add, one compare with the count's maximum, which stops the program at it ("too many
`weakref` links to a `Node` (4,294,967,295)", verdict 59; not measured). A read: where R11 proves it, a plain load and a test for no link; where it cannot, one header test
as well. Run 3's W2, the parent walk: 1.176x of plain C with no header test per step
(`diag_w2_hop_proved`), 1.579x with a header test on every hop (`model_w2`), safe Rust 1.603x (`rust_w2`)
(`RUN3/26-timer.md` lines 20 to 22). Both model numbers were measured with uncounted links: the counts
above come on top, on every move of a node, and were not measured (`V2/changes.md` 2.1 asks the next
measuring phase to run W2 with them). An ordinary `weakref` link, read with a test each time, as Rust's
`Weak`: on churn 0.919 and 0.935 of plain C (`graphs-callbacks-attack2/bench/runs/weak_g.txt`,
`weak_h.txt`). Memory: each node's header holds a third 32-bit number, its weak count (R9). The program
writes the `remove`, so no hidden search.

**8. Kind:** a smarter compile-time solution (R11's proof removes the header test where the program's own
lines show that a node's parent owns it), with `weakref` a readable spelling of Rust's `Weak`.

---

## AN-64: L3001 tells the programmer to make the owning edge weak

**v5:** verdicts 83 to 97 change no line here; the owner's-test table no longer cites `changes`
(verdict 86). Meaning unchanged.

**1. Intent:** follow the warning to fix a tree's cycle.

**2. OLD.** `old/an63_tree_python.em`, `old/an63_tree_weak.em`, `old/an62_walk_callback.em`: all three
get the same advice, pasted under AN-63 and AN-62: `help: if \`Node.children\` is a back-reference, make
it hold a \`Weak\` handle`. `children` is the owning edge; making it weak would free every child. In the
second and third files `parent` is already `Weak`, and the warning is still given.

**3. NEW.** The NEW tree of AN-63 gets no warning: a list whose elements have one owner cannot form a
cycle (handing a node into its own subtree would hand over a value while a view into it lives, which is
refused when compiling). The warning remains only for a type whose `shares` fields can form a cycle of
owners (R11), and names the link to make `weakref`:

```ember
class Person:
    name: String
    friends: Array[shares Person] = []
    best: Option[shares Person] = None
```

```text
warning[L3001]: `Person.best` and `Person.friends` can hold each other: a cycle of `shares` owners is never freed
 --> app.em:4:5
  |
4 |     best: Option[shares Person] = None
  |     ^^^^ this link points at a person another field already owns
  |
  = help: a link that should not keep its person alive is `weakref`: `best: weakref Person = None`
```

The field named is the single-valued one when the cycle has one (a back link), otherwise the field the
cycle closes on; a parent-child pair in the Python spelling is the error of AN-63, which names
`parent: weakref Node = None`, set with `child.parent = weakref this`. The parent pair's
advice gives the program's own line, `child.parent = weakref this` (verdict 14).

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `name` | none (default: the person's own String) | through a name that may change the person (worked out, R5) |
| `friends` | `shares` on its elements: one of several owners of each person, reading it | the list: through a name that may change the person; not the persons, which a `shares` owner only reads |
| `best` | `shares`: one of several owners of that person, reading it | the link: through a name that may change the person; not the person |

**5. Means exactly.** The NEW tree is AN-63's C (`c\an62_an63_v2.c` part 1): no ASan report, and its
line `5 kept link: gone` (pasted under AN-63) shows the root's end ending the whole tree, every node with
it, while a link kept elsewhere finds its node ended. (ASan on Windows has no leak checker, so "every node
is freed" is read from the code's ends, not from a leak report.) The warning itself is a message; the
program it is given for is a leak, not something to translate.

**6. Still refused.** Nothing new; the Python spelling's error is AN-63's (E3013).

**7. Cost.** None (compile time).

**8. Kind:** a smarter compile-time solution (one-owner links are proved acyclic, so the warning is only
given where a cycle can exist, and it names the right link).

---

## AN-65: `upgrade` is Rust's word for "the object, if it is still alive"

**v5:** E3060's way out opens the link again under a new name, `q` (worklist F-01; the rules' message);
the owner's-test table no longer cites `changes` (verdict 86). Meaning unchanged. Nothing keeps `p` alive
(R10, verdict 82): no line or call in its life may end a `Player`, which is checked when compiling.

**1. Intent:** read a weak link's target.

**2. OLD.** `old/an65_upgrade.em`:

```ember
class Camera:
    target: Weak[Player] = Weak.empty()

fn main():
    player = Player()
    cam = Camera()
    cam.target = Weak(player)
    match cam.target.upgrade():
        Some(p) => println(p.health)
        None => println("gone")
```

```text
=== an65_upgrade.em
exit=0
```

**3. NEW.**

```ember
class Camera:
    target: weakref Player = None

fn main():
    player = Player()
    cam = Camera()
    cam.target = weakref player
    if cam.target exists as p:          # the player still lives: call it `p`
        println(p.health)
    else:
        println("gone")
```

The link is `weakref` (R11, verdict 9) and the opening is `if cam.target exists as p:` (R13, verdict
26).

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `target` | `weakref`: a link that does not keep the player alive | through a name that may change the camera (re-pointing: `cam.target = weakref player`) |
| `player` | none (default: `main`'s own Player) | its owner may |
| `cam` | none (default: `main`'s own Camera) | its owner may |
| `weakref player` | `weakref`: a link, not a hand-over; `player` stays `main`'s | no |
| `p` | none (default for an opened name: a read-only view of the player; nothing in its life may end a `Player`, R10) | no |

**5. Means exactly.** `means/m65_weak_read.em` (`if Some(p) = cam.target.upgrade():`, today's spelling):

```text
=== m65_weak_read.em
exit=0
```

In C, `c/an63_links.c` part 2: a counted link (one add when it is made), read as
`(w && !w->dead) ? w : NULL`; lines `5 100` and `6 gone` of its output are the read before and after the
player ends, clean under ASan (lines 1 to 4 are run 3's `set by` tree, which v2 replaces with AN-63's C):

```text
1 root/a/leaf
2 leaf
3 root/leaf
4 leaf parent=None
5 100
6 gone
an63_links ms=0.068 peak_kb=8264 checksum=0
```

**6. Still refused.**
- `cam.target.upgrade()`, E1010: "`upgrade` is not Ember: a `weakref` link is opened with `exists as`;
  instead of the `match` line and its `Some(p)` and `None` branches, write `if cam.target exists as p:`
  for the `Some(p)` branch and `else:` for the `None` branch" (R13, R18: the lines it replaces).
- `Weak[Player]`, `Weak(player)`, E1010: "write `weakref Player`", "write `weakref player`".
- `if Some(p) = cam.target:`, E0115: "an `Option` or a `weakref` link is opened with `exists as`: write
  `if cam.target exists as p:`".
- `cam.target.health`, E2020: "`cam.target` may be gone; open it: `if cam.target exists as p:`".
- `cam.target = player`, E2020: "`=` alone would hand `player` over; write `cam.target = weakref player`".
- `a = copy of cam.target`, E2116: "`copy of cam.target` would copy the link, not the player; for another
  link to the same player write `a = weakref cam.target`; for a copy of the player itself write
  `if cam.target exists as p:` and then `c = copy of p`" (verdict 10), the second way out given where
  `Player` has `Clone`; where it has none (as in a program that declares `class Player:` with no
  `@derive(Clone)`), it reads "`Player` has no `Clone`; write `@derive(Clone)` above `class Player`, or
  keep a view of it: `if cam.target exists as p:`" (rules section 3, worklist section 4, item 2).
- A line that may end the player while `p` is still used: in the NEW, `player = Player()` (line 9, which
  replaces `main`'s player and so ends it) above `println(p.health)` (now line 10), E3060 (verdict 82):
  "`player = Player()` (line 9) may end the player `p` reads (`if cam.target exists as p:`, line 8), and
  `p` is used on line 10; use `p` before line 9, or open the link again after it: write `if cam.target
  exists as q:` above line 10, indent line 10 under it and use `q` there" (R10 point 1, R13; rules
  section 3's message in this program's lines). v3's R10 kept the player alive for `p` there; nothing does now.

**7. Cost.** One add of the player's weak count when `cam.target = weakref player` makes the link and
one subtract when it ends (R11); one test of the target's header per read (the camera's target is not a
parent link); before the add, one compare with the count's maximum (verdict 59; not measured). Nothing
keeps `p` alive: no line in its life can end a Player, which is checked when compiling (R10 point 1,
verdict 82). Rust's
`upgrade` always makes a count pair.

**8. Kind:** a readable spelling of Rust's guarantee.

---

## AN-66: A bare interface type is a counted, shared, dynamically dispatched handle with no word

**v5:** the list's making line says `owns` before each element (verdict 85):
`shapes=[owns Circle(r=1.0), owns Square(side=2.0)]`, which settles step 3's question 3; the line
without it is refused (E2123, added to part 6), and part 6's E3040 example makes its `owns dyn Shape`
local with `owns` too. The owner's-test table no longer cites `changes` (verdict 86). Meaning unchanged:
the shapes are made and held as before.

**1. Intent:** store shapes of several kinds and draw them.

**2. OLD.** Over class objects, a bare interface type is accepted and its second name shares the object
silently (`old/an66_bare_interface_class.em`):

```ember
fn main():
    a: Shape = Circle()
    b = a
    b.grow()
    println(a.size())
```

```text
=== an66_bare_interface_class.em
exit=0
```

Over value types it is refused, and one owner needs `Box[dyn Shape]` (`old/an66_bare_interface.em`):

```text
error[E2173]: `Shape` is an interface, and the types implementing it are values, not classes
  --> an66_bare_interface.em:11:19
   |
11 |     shapes: Array[Shape]
   |                   ^^^^^
   |
   = help: hold such a value behind a pointer: `Box[dyn Shape]` in a field or a collection (`Box(v)` makes one), `ref dyn Shape` for a parameter; or take a type parameter, `[T: Shape]`, and pass the value itself
   = note: an interface's name as a type is a class handle typed by it [OBJ-2]; a value of a type implementing `Shape` has its own size, which `dyn Shape` keeps behind a pointer (ODR-182)

error: could not compile due to 1 error(s)
exit=1
```

**3. NEW.**

```ember
struct Scene:
    shapes: Array[owns dyn Shape]           # the list is each shape's one owner
    shown: Array[shares dyn Shape]          # shapes also held elsewhere, read here

fn draw(s: dyn Shape):                      # a read-only view of any shape
    ...

fn main():
    scene = Scene(shapes=[owns Circle(r=1.0), owns Square(side=2.0)], shown=[])
    for s in scene.shapes:
        draw(s)
```

`owns dyn` and `shares dyn` say who owns each shape (verdict 19's words), and `owns` on each making
line says that the list holds it alone in a block of its own (verdict 85).

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `shapes` | `owns` on its elements: each shape has this one owner | through a name that may change the scene (worked out, R5) |
| `shown` | `shares` on its elements: one of several owners, reading the shape | the list: through a name that may change the scene; not the shapes |
| `s` in `draw` | none (default: a read-only view, a parameter) | no |
| `scene` | none (default: `main`'s own Scene) | its owner may |
| `Circle(...)`, `Square(...)` in the list | `owns`: each made in a block of its own and handed to the list, its one owner (verdict 85) | their owner may |
| `s` in the loop | none (default: a read-only view of an element) | no |

**5. Means exactly.** `owns dyn Shape` is today's `Box[dyn Shape]` (`means/m66_owned_dyn.em`):

```text
=== m66_owned_dyn.em
exit=0
```

`shares dyn Shape` over class objects is today's bare `Shape` handle (`old/an66_bare_interface_class.em`,
`exit=0` above, with the sharing written). Over value types today has no spelling at all
(`means/m66_shared_dyn.em`, `a: Shared[dyn Shape] = Shared(Circle(r=1.0))`):

```text
error[E2020]: expected `dyn Shape`, found `Circle`
  --> m66_shared_dyn.em:11:35
   |
11 |     a: Shared[dyn Shape] = Shared(Circle(r=1.0))
   |                                   ^^^^^^^^^^^^^ this is `Circle`
   |
```

(two more errors follow in the file); in C it is a block `{count, Circle}` and a pair `{pointer, method
table}`, Rust's `Rc<dyn Shape>`.

**6. Still refused.**
- `shapes: Array[Shape]`, E2173: "say who owns the shapes: `Array[owns dyn Shape]` (the list is their
  only owner) or `Array[shares dyn Shape]` (other names own them too)".
- The OLD's `b = a` then `b.grow()` and `a.size()`, with `a: owns dyn Shape = owns Circle()` (verdict
  85: the place owns it alone): `b = a` hands the shape over (row 9), and the later use of `a` is E3040:
  "`a` was handed over to `b` (`b = a`) and is used after it (`println(a.size())`); to keep both names
  for one shape that both may change, make it shared: write `a: shares mut dyn Shape = Circle()` in
  place of `a: owns dyn Shape = owns Circle()`, and `b = shares mut a` in place of `b = a`" (R1, R3; two
  owners that may change it, verdict 23). A copy of
  the shape (`b = copy of a`) is offered only where the program makes `dyn Shape` values copyable, which
  this one does not (E2116 otherwise).
- `Box[dyn Shape]`, E1010: "write `owns dyn Shape`".
- The list made without `owns` on its elements, `Scene(shapes=[Circle(r=1.0), Square(side=2.0)],
  shown=[])` (v4's line), E2123 (R3, verdict 85): "`shapes` holds each shape in a block of its own; say
  so where it is made: write `scene = Scene(shapes=[owns Circle(r=1.0), owns Square(side=2.0)],
  shown=[])` in place of line 9" (the E2123 row of `V5/rules.md` section 3, in this program's names).

**7. Cost.** `owns dyn`: one heap block per value and a method-table pointer beside it (Rust's
`Box<dyn>`); `shares dyn`: the same plus a count (Rust's `Rc<dyn>`), whose every +1 is compared with
its maximum first (verdict 59; one compare, not measured); a call is one indirect call.

**8. Kind:** a readable spelling of Rust's guarantee.

---

## AN-67: Trait objects and downcasting (A13)

**v5:** a kind test that can never pass has its own code, E2037, with the rules' message (worklist
U5); the owner's-test table no longer cites `changes` (verdict 86), and `ref mut pool` is allowed because
a method may change its own object (R5). Meaning unchanged.

**1. Intent:** `get_mut[T]` finds the pool of `T` in the registry and returns element `i`.

**2. OLD.** `old/an67_box_cast.em` (last run's `a13_box_cast.em`, phase 1's registry):

```ember
struct Reg:
    pools: Map[u128, Box[dyn AnyPool]]

    fn get_mut[T](mut self, i: int) -> ref mut T:
        pool = cast self.pools[7u128] as Pool[T]
        return ref mut pool.dense[i]
```

```text
error[E2020]: `cast … as` and `cast … if` need related class handles, found `Box[dyn AnyPool]` and `Pool[T]`
  --> an67_box_cast.em:18:16
   |
18 |         pool = cast self.pools[7u128] as Pool[T]
   |                ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |

error: could not compile due to 1 error(s)
exit=1
```

A generic method through `dyn` (`old/an67_dyn_generic.em`, `fn inspect[V: Visitor](self, mut v: V)`):

```text
error[E2050]: interface `Inspect` is not `dyn`-compatible
  --> an67_dyn_generic.em:18:23
   |
18 | fn walk(xs: Array[Box[dyn Inspect]]) -> int:
   |                       ^^^^^^^^^^^ method `inspect` is generic
   |

error: could not compile due to 1 error(s)
exit=1
```

The spec's `as?` (`old/an67_cast_forms.em`, 05's probe):

```text
error[E0100]: `as?` is not Ember: a downcast is written `cast x if D`
  --> an67_cast_forms.em:10:13
   |
10 |     a = s as? Circle
   |             ^ write `cast` before the value and `if` here
   |
   = help: `cast x as D` gives the `D` and panics if `x` is not one; `cast x if D` gives `Some(d)` or `None`

error: could not compile due to 1 error(s)
exit=1
```

**3. NEW.**

```ember
struct Reg:
    pools: Map[u128, owns dyn AnyPool]

    [T]
    fn get_mut(k: u128, i: int) -> ref mut T:
        if this.pools[k] is Pool[T] as ref mut pool:   # the pool at `k` is a `Pool[T]`: call it `pool`
            return ref mut pool.dense[i]
        panic("no pool of this type at this key")

interface Inspect:
    [V: Visitor]
    fn inspect(ref mut v: V)

fn walk(xs: Array[owns dyn Inspect]) -> int:
    s = Sum(total=0)
    for x in xs:
        x.inspect(ref mut s)        # through `dyn`: the entry made with V = dyn Visitor
    return s.total
```

Changed from v2: the kind test
`if this.pools[k] is Pool[T] as ref mut pool:` (v2 opened today's `cast ... if Pool[T]` with verdict
1's opening word, two `if`s in one line), R14 (verdict 27).

The kind test `x is D as name` on an `owns dyn` (or `shares dyn`) place tests the value's type (one
type-id compare) and, when it is a `D`, names it there as a `D`. A value tested in its place stays a
place, so the name is a view of the registry's own pool: `ref mut pool` names a view that may change it,
since a method may change its own object (worked out from its body, R5); plain `pool` would be read-only
(R14; part 6).
`is Pool[T]` says what `pool` is (the pool at `k`, seen as a `Pool[T]`), and `ref mut` that it may change
it. When it is not a `Pool[T]` the `if` block does not run and `get_mut` stops the program with its
message; the "sure" form, `pool = ref mut cast this.pools[k] as Pool[T]`, stops it in the cast instead
(area B's AN-22). A generic method whose type parameter appears only as the type of a `ref` or `ref mut`
parameter, with a `dyn`-compatible bound, gets one method-table entry, the generic body compiled with
`V = dyn Visitor` (R14).

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `pools` | `owns` on its values: each pool has the map as its one owner | through a name that may change the registry (worked out, R5) |
| `this` in `get_mut` | `this`: the Reg `get_mut` was called on | none written (worked out: its body gives back a changeable element of a pool in `this.pools`, R5) |
| `k`, `i` | none (default: read-only numbers) | no |
| `pool` | `is Pool[T]`: the pool at `k`, seen as a `Pool[T]`; `ref mut` on the name the kind test gives: a view of the registry's own pool | yes (`ref mut`; a method may change its own object, R5) |
| result `-> ref mut T` | `ref mut`: a view into the registry's element | yes |
| `this` in `inspect` | `this` | none written (worked out from each implementation's body, R5; `owns this` would take it over) |
| `v` | `ref mut`: the caller's visitor | yes |
| `xs` | `owns` on its elements (one owner, the list); the parameter itself none (a read-only view) | no |
| `s` | none (default: `walk`'s own Sum) | its owner may; lent `ref mut` to `inspect`, and the call says so |
| `x` | none (default: a read-only view of an element) | no |

**5. Means exactly.** The cast, with the pools as class objects (today's `cast` needs class handles),
`means/m67_class_pools.em`:

```ember
class Reg:
    pools: Map[int, AnyPool] = Map[int, AnyPool]()

    fn bump(self, k: int, i: int):
        match cast self.pools[k] if Pool[int]:
            Some(p):
                p.dense[i] += 1
            None:
                pass
```

and the `dyn` entry, written by hand as Rust's workaround is, `means/m67_dyn_param.em`
(`fn inspect(self, v: ref mut dyn Visitor)`, `x.inspect(ref mut s)`):

```text
=== m67_class_pools.em
exit=0
=== m67_dyn_param.em
exit=0
```

(Both files are today's Ember, so they still write `self`.)

**6. Still refused.**
- A generic method whose type parameter is returned or stored by value (`[V: Visitor]` above `fn make() -> V`)
  through `dyn`, E2050: "`Inspect` cannot be used as `dyn Inspect`: `make` returns its `V`, and a `dyn`
  value has one entry per method; take the visitor in instead of returning it, `[V: Visitor]` above
  `fn make(ref mut v: V)`, or keep each list's own type with a type parameter instead of `dyn Inspect`:
  `[I: Inspect]` above `fn walk(xs: Array[I]) -> int:`".
- A kind test that can never pass, `if this.pools[k] is Label as l:`, where `Label` does not implement
  `AnyPool`, E2037 (worklist U5; R14; the rules' message in this program's names): "`this.pools[k]` is
  an `AnyPool`, which can never be a `Label`; remove the test: the `if` line and its block". A `cast x
  as D` between unrelated types stays E2020 (a type mismatch, today's code); where the cause is today's
  type, the message says "`Box[dyn AnyPool]` is now `owns dyn AnyPool`" with the line (`pools: Map[u128,
  owns dyn AnyPool]`).
- A plain name from the kind test, `if this.pools[k] is Pool[T] as pool:` with `return ref mut
  pool.dense[i]`, E3023: "`pool` reads the pool in `this.pools` and cannot change it; to return a
  changeable element, write `if this.pools[k] is Pool[T] as ref mut pool:`" (R14; area B's AN-22 gives
  the same message).
- Today's "not sure" downcast, `if Some(ref mut pool) = cast this.pools[k] if Pool[T]:`, E1010 (verdict
  27): "`cast this.pools[k] if Pool[T]` is not Ember: to run a block when the pool at `k` is a
  `Pool[T]`, write `if this.pools[k] is Pool[T] as ref mut pool:`; to stop the program when it is not
  one, write `pool = ref mut cast this.pools[k] as Pool[T]`". `cast this.pools[k] when Pool[T]`, verdict
  25's spelling, which verdict 27 removed, gets the same message with its own words in the first quote
  (R14).
- Today's `match cast this.pools[k] if Pool[T]:` with `Some(pool):` and `None:` branches (the form of
  part 5's `m67_class_pools.em`), E1010: "a choice among kinds is written with `is`: instead of the
  `match` line and its two branches, write `if this.pools[k] is Pool[T] as ref mut pool:` for the
  `Some(pool):` branch and `else:` for the `None:` branch" (R14: there is no "maybe a `Pool[T]`" value
  to `match` on).
- `as?`, E1010 (today E0100 with today's line, pasted in part 2): "a downcast that may fail is a kind
  test: write `if s is Circle as a:` in place of `a = s as? Circle`, and indent under it, as its block,
  the lines that use `a`" (where the program then opened `a`, as in rules section 3's example, the
  message names that line too: "instead of `a = s as? Circle` and `if a exists as c:`, write `if s is
  Circle as c:`, keeping its block"); `as!`,
  E1010: "`as!` is not Ember: to stop the program when `s` is not a `Circle`, write `a = cast s as
  Circle`" (the vocabulary's `as?` and `as!` rows, in the names of part 2's probe).

**7. Cost.** A `cast`: one load of the method table's type id and one compare; safe Rust's
`downcast_mut` was measured slower (1.710x and 1.606x against the pool table's 1.208x and 1.084x of plain
C, `phase3b-references.md` 3.4 as quoted in `phase3b-graphs-callbacks.md` 6.3); `V5/rules.md` section 2
gives 1.223 to 1.244 of plain C against `downcast_mut` 1.571 to 1.635 (AN-22, PROVISIONAL). The kind
test is the same one compare. The `dyn` entry: the indirect calls Rust's hand-written workaround makes.

**8. Kind:** a smarter compile-time solution.

---

## AN-68: `if c = cast s if Circle:` gets E2036 with a help that is not what was meant

**v5:** verdicts 83 to 97 change no line here. Meaning unchanged. (v4's line: the NEW is v3's, the kind
test `if s is Circle as c:` (verdict 27). Three messages of part 6 corrected (R18, verdict 73; worklist section 4, items 1 and 7): the two E2036 messages said "write
`c = s` on the line before", which left the refused `if` line in place, and now say to write it in
place of the `if` line, the block moved out one level (rules section 3); the E2106 message now also
replaces `s.r` in the block.)

**1. Intent:** do something only when a shape is a circle.

**2. OLD.** `old/an68_if_cast.em`:

```ember
    for s in shapes:
        if c = cast s if Circle:
            t += c.r
```

```text
error[E2036]: this pattern always matches
  --> an68_if_cast.em:12:12
   |
12 |         if c = cast s if Circle:
   |            ^^^^^^^^^^^^^^^^^^^^
   |
   = help: did you mean `c == cast s if Circle`?
   = note: to declare `c`, write `c = cast s if Circle` on the line before

error[E2020]: `Option[Circle]` has no field `r`
  --> an68_if_cast.em:13:18
   |
13 |             t += c.r
   |                  ^^^
   |

error: could not compile due to 2 error(s)
exit=1
```

**3. NEW.**

```ember
    for s in shapes:
        if s is Circle as c:       # `s` is a Circle: call it `c`
            t += c.r
```

`if s is Circle as c:` runs its block when `s` is a `Circle` and names it `c` there (R14). `is` before a
type name means "is of this kind", as in Swift, Kotlin and C#; `as c` gives the name, as in `import x as
y`; one `if`, which settles the two `if`s v2 left for the owner (`V2/changes.md` section 11, question 5).
v2's change from run 4 stays: run 4 accepted the programmer's own line as its opening form, and the
owner's opening word (verdict 1, then 26) made it a refused line; v3 refuses it as today's `cast s if
Circle` (E1010), with a message that gives the line meant (part 6) instead of today's "did you mean
`c == cast s if Circle`?".

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `s` | none (default: a read-only view of an element) | no |
| `c` | `is Circle`: `s` itself, seen as a `Circle` | none (default for the name a kind test gives: a read-only view, R14) |

(`t` is declared earlier in the function, as the function's own `f64`; this fragment does not declare
it.)

**5. Means exactly.** Today's spelling of the same meaning (`means/m68_if_some.em`,
`if Some(c) = cast s if Circle:`):

```text
=== m68_if_some.em
exit=0
```

Today's compiler does not know the opening word (`V2/changes.md` section 7, `q1_holds.em`, which probed run 4's `holds`), nor the kind test:
`if s is Circle as c:` is E1010 "cannot find `Circle` in this scope" and E2020 "cannot be cast ... with
`as`", since today's `is` takes a value and `as c` reads as a conversion (`V3/changes.md` section 7,
`r2_is_as.em`, output pasted there).

**6. Still refused.**
- `if c = cast s if Circle:` (the OLD line), E1010 (verdict 27): "`cast s if Circle` is not Ember: to
  run a block when `s` is a `Circle` and call it `c`, write `if s is Circle as c:`; to stop the program
  when it is not one, write `c = cast s as Circle`" (R14; one mistake, one message: the line's `=`
  needs no message of its own once the line is replaced).
- `if Some(c) = cast s if Circle:` (today's spelling, part 5), E1010, with the same message (the
  vocabulary's `cast` row).
- Verdict 25's spelling, which verdict 27 removed: `found = cast s when Circle` (line 2) then `if found
  exists as c:` (line 3), E1010: "`cast s when Circle` is not Ember: instead of `found = cast s when
  Circle` and `if found exists as c:`, write `if s is Circle as c:`" (R14, R18: the message names both
  lines it replaces).
- `match cast s if Circle:` with `Some(c):` and `None:` branches, E1010: "a choice among kinds is
  written with `is`: instead of the `match` line and its two branches, write `if s is Circle as c:` for
  the `Some(c):` branch and `else:` for the `None:` branch" (R14).
- `if s exists as c:`, where `s` always holds a value, E2036: "`s` is a shape, which always holds a
  value, so this `if` would always run; to test whether it is a Circle write `if s is Circle as c:` in
  place of this line; to name it write `c = s` in place of the `if` line, and use `c` in its block,
  moved out one level" (R13, R14).
- A kind test that always passes, where `s` is already a `Circle` (`for s in circles:`), E2036: "`s` is a
  `Circle` already, so this `if` would always run; to name it write `c = s` in place of the `if` line,
  and use `c` in its block, moved out one level" (R14; rules section 3's message).
- `if s is Circle as s:` with `t += s.r` in its block, E2106: "`s` already names a value (line 1); `as`
  declares a new name: write `if s is Circle as c:`, and `c.r` for `s.r` in its block" (R14; rules
  section 3's message).

**7. Cost.** None beyond the one type-id compare of the kind test (AN-67).

**8. Kind:** a readable spelling of Rust's guarantee (`if let Some(c) = s.downcast_ref::<Circle>()`),
written as Swift, Kotlin and C# write it, with today's unhelpful help replaced by the line meant.

---

## AN-69: Generics friction (A16)

**v5:** verdicts 83 to 97 change no line here. Meaning unchanged. (v4's line: verdict 68 confirms
`copy of xs[0]` for a `Copy` `T`: every `Copy` type has `Clone`.) Two messages of part 6 now give exactly what they replace
(R18; rules section 3): E2040 says the bound goes in place of `[T]`, and E2072 names the type that must
implement the interface it declares, without which the bound would still not hold
(`V4/safety/check-V6.md` item 8).

**1. Intent:** sum any addable values; print a list of vectors; give one type its own description.

**2. OLD.** `old/an69_bounds.em` (no bounds written):

```text
error[E2040]: `T`'s bounds do not provide a matching `Add` operator
 --> an69_bounds.em:5:15
  |
5 |         acc = acc + x
  |               ^^^^^^^
  |
  = help: add a `Add` bound with the required operand and output types
  = note: inside a generic body only the bounds' operations are available [TYP-17]

error: could not compile due to 1 error(s)
exit=1
```

`old/an69_orphan.em` (a one-file program implements `Neg` for `Array[Vec3]`):

```text
error[E2121]: `Array[Vec3]` and `Neg` are both another package's, so this package cannot implement one for the other
 --> an69_orphan.em:5:31
  |
5 | extend Array[Vec3] implements Neg:
  |                               ^^^
  |
  = help: implement it where `Neg` or `Array[Vec3]` is declared, or wrap the value in a type of this package
  = note: an implementation lives with its interface or its type, or with a type of its package among the interface's arguments [TYP-20]
```

(followed by `error[E2020]: \`-\` cannot be applied to \`Array[Vec3]\``.) `old/an69_overlap.em`:

```text
error[E2041]: `i32` already implements `Describe`
 --> an69_overlap.em:9:23
  |
5 | extend[T: Debug] T implements Describe:
  | --------------------------------------- every type implements it here
...
9 | extend i32 implements Describe:
  |                       ^^^^^^^^
  |
  = note: two implementations of one interface may not apply to one type [TYP-19]; the prelude converts every type from itself [ERR-7]

error: could not compile due to 1 error(s)
exit=1
```

**3. NEW.**

```ember
[T]
fn sum(xs: Span[T]) -> T:                 # not `pub`: its bound is worked out from the body
    acc = copy of xs[0]
    for x in xs[1..]:
        acc = acc + x
    return acc

[T: Add[T, Output = T] + Copy]
pub fn total(xs: Span[T]) -> T:    # `pub`: the bound is written
    ...

extend Array[Vec3] implements Neg:           # accepted in a program (a package nothing depends on)
    ...
```

Rules (R14, R17): a function that is not `pub` gets its
bounds from its body (every operation on `T` maps to the one interface in scope that provides it;
recorded, printed by `ember inspect --bounds`); a `pub` function writes its bounds, because they are its
promise to other packages, and the error gives the exact line; a package that nothing can depend on (a
program or a test) may implement any interface for any type (two implementations of one interface for
one type anywhere in the program stay refused). Inferred bounds stop at `pub`, so nothing another
package relies on is decided by the compiler from a body it cannot see.

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `xs` | none (default: a read-only view, a parameter) | no |
| `acc` | `copy of`: the function's own copy of the first element | its owner may (`=` gives it a new value, R2) |
| `x` | none (default: a read-only view of an element) | no |

**5. Means exactly.** The bound the compiler infers for `sum`, written out (`means/m69_bounds_written.em`,
`fn sum[T: Add[T, Output = T] + Copy](xs: Span[T]) -> T:`):

```text
=== m69_bounds_written.em
exit=0
```

The orphan exemption is a permission, not code; the overlap stays refused, so it needs no translation.

**6. Still refused.**
- A `pub` generic with an operation its bounds lack (`[T]` above `pub fn total(xs: Span[T]) -> T:` with
  `sum`'s body), E2040: "`total` is `pub`, so its bound is written: write `[T: Add[T, Output = T] +
  Copy]` in place of `[T]` above `pub fn total(xs: Span[T]) -> T:`" (a non-`pub` function never gets
  it).
- E2121 in a library package: "`Array[Vec3]` and `Neg` both belong to other packages, and packages that
  depend on this one could see a second `Neg` for `Array[Vec3]`; wrap it: `struct Vecs(Array[Vec3])`
  and `extend Vecs implements Neg:`".
- E2041, overlapping implementations (the most specific one winning is unproven for Ember): "`i32`
  already gets `Describe` from `extend[T: Debug] T implements Describe` on line 5, and two
  implementations may not apply to one type; give `i32` its own description through a function of its
  own, `fn describe_int(x: i32) -> String:`, or remove line 9 so that `i32` uses the general one".
- A non-`pub` body calling a method no interface in scope declares (`heaviest`, given `Crate`s), E2072:
  "`T` has no `weight`: no interface in scope declares `fn weight`; declare one, `interface Weighed:`
  with `fn weight() -> int`, and write `struct Crate implements Weighed:` for the type `heaviest` is
  given; the bound follows" (rules section 3's message; a duck-typing extension is not taken: it would
  let a typo surface at a call far away).

**7. Cost.** None at run time: every instance compiles to the code written bounds give.

**8. Kind:** a smarter compile-time solution (bounds and the orphan exemption); the overlap is an
understandable limitation kept, with its message.

---

## AN-70: `virtual` in a class that cannot be subclassed is accepted with a warning (W2111)

**v5:** `virtual fn on_update(dt: float):` and `override fn on_update(dt: float):` say nothing about what
they change (verdict 86, the script's lines); part 6's E2113, for an override that changes more than its
virtual's first line says, is gone (E2113 removed): a call through the virtual counts every override's
changes (R5). The prose no longer says the override avoids `min` (worklist F-15): it calls it. Meaning
unchanged: v4's refusal only asked for a word on the virtual.

**1. Intent:** declare a method that subclasses may replace.

**2. OLD.** `old/an70_virtual.em` (reader 4's `t10_w2111.em`):

```ember
class Door:
    angle: float = 0.0
    virtual fn on_update(self, dt: float):
        pass
```

```text
warning[W2111]: `virtual` has no effect in a final class
 --> an70_virtual.em:3:16
  |
3 |     virtual fn on_update(self, dt: float):
  |                ^^^^^^^^^
  |

exit=0
```

**3. NEW.**

```ember
open class Door:
    angle: float = 0.0
    virtual fn on_update(dt: float):
        pass

class SwingDoor(Door):
    override fn on_update(dt: float):
        this.angle = min(this.angle + 90.0 * dt, 90.0)
```

Changed from v4 (verdict 86): neither the virtual nor the override says what it changes. A call through
`Door.on_update` counts the changes of every override the program has, worked out from their bodies
(R5): here `SwingDoor`'s, which changes `this.angle`. The override calls `min`, a prelude function of
two values (`[STD-26]`): the angle grows by `90.0 * dt` and stops at 90.

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `angle` | none (default: the door's own `float`) | through a name that may change the door (worked out, R5) |
| `this` in `Door.on_update` | `this`: the door the method was called on | none written (worked out: its body changes nothing; a call through `Door` counts every override's changes, R5) |
| `this` in `SwingDoor.on_update` | `this` | none written (worked out: its body changes `this.angle`) |
| `dt` (both) | none (default: a read-only number) | no |

**5. Means exactly.** `means/m70_open_class.em`:

```text
=== m70_open_class.em
exit=0
```

and the override, `target/redesign3/19-fixer/m70_virtual_changes.em` (today a class method's plain
`self` may write fields, `[CLS-7]`; `SwingDoor(Door)` with `override fn on_update(self, dt: float):
self.angle = min(self.angle + 90.0 * dt, 90.0)`, the same result as the NEW body):

```text
===== m70_virtual_changes.em
exit=0
```

**6. Still refused.**
- `virtual` in a class that is not `open` or `abstract`, E2112: "`Door` cannot be subclassed, so
  `virtual` on `on_update` changes nothing; to let subclasses replace it write `open class Door:`, or
  remove `virtual`".
- Not refused any more (verdict 86): an override that changes more than its virtual's body does (v4's
  E2113 asked for `changes this` on the virtual). A call through a `Door` counts every override's
  changes, so a loop that a `SwingDoor`'s `on_update` could break is refused at the call, whatever the
  `Door` it is called on (R5, R8 (c), E3020).

**7. Cost.** None.

**8. Kind:** an understandable limitation kept (only an `open` class has subclasses), now refused with the
line to write instead of accepted with a warning.

---

## Words needed

None: every word the NEW code above uses has a row in `V5/vocabulary.md`, and no NEW block of this area
uses `changes`, `reads`, `captured` or a change list. `min` as a function of two values is a prelude
function (`[STD-26]`).

## For the owner

None. Step 3's three questions are settled: 1 (may a walk hand a node to a visit function that may change
nodes?) by verdict 53's corrected record: the compiler reads every function stored in `on_visit`, and
`this.on_visit(p)` compiles (AN-62); 2 (`owns` on an `int` a function keeps) by verdict 84: `owns` hands
over every type, numbers included, and E2085 is removed (AN-53, AN-54); 3 (`owns` on the elements of a
list made in place) by verdict 85: `shapes=[owns Circle(r=1.0), owns Square(side=2.0)]` (AN-66).

## Notes for the lead and the rule writer

Points where the v5 rules did not settle what an answer needed, or where an answer found a gap; what this
area wrote meanwhile. None changes how a program is written.

1. **Rules section 3's E3020 message for a `for` walk of read-only handlers** (the last E3020 example of
   the first E3020 row, `close_all`) gives `handlers: Array[fn(ref mut Ui)]` and the position walk, but
   not the handler's own line; a handler written for `fn(Ui)` (G-05's, which captures `weakref ui`) does
   not fit a list of `fn(ref mut Ui)` (its parameter's mode differs, E2020), so the message's fix runs
   into another error (verdict 73). AN-60 part 6 gives the handler's line too (`fn(ref mut me: Ui):`, no
   `weakref ui`, no `if ui exists as ref mut me:` line) and walks with a local `i`. v4's note 2, still
   open.
2. **The kind test between unrelated types** (AN-67): E2037 now; a `cast x as D` between unrelated types
   is left as today's E2020 (a type mismatch), which no rule names; R14 could say it.
3. **Still open from v3** (`V3/area-D.md` notes 1 and 2; v4's note 4): R11's parent-link proof is
   package-wide, and one line elsewhere makes every `parent` read in the package test the header (1.579x
   of plain C instead of 1.176x on W2, `RUN3/26-timer.md` lines 20 to 22) with no message; and AN-62 (b)
   still pays one header test per step, which no v5 rule removes (verdict 28 asks the next run to look
   for proof). Neither was priced in this step.
4. **Settled in this step** (no ruling needed): v4's note 1 (R4's E3031 against R11's walk) by verdict
   53's corrected record; v4's note 3 (E2085 against `owns n`) by verdict 84. AN-62's new E3031 refusal
   uses the rules' message and its lines (40 to 42: the program's `t.on_visit = prune` and `prune`'s
   body).
5. **What this file checked.** Every fenced block of the copy taken at the start of this step
   (`target/redesign7/03/area-D_start.md`) that is an OLD block, a pasted output or a "means exactly"
   block in today's Ember or C is still in this file, byte for byte (`target/redesign7/03/check_blocks.py`,
   output `check_blocks.txt`, which lists the blocks this step changed or added: NEW blocks and messages
   only).
