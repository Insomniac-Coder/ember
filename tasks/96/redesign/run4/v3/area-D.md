# Area D, v3: linked objects, callbacks, interface types (AN-53 .. AN-70)

Agent 05 (area-D), run 4, step 2 (v3). Every answer of area D brought to the v3 rules and words, which
apply all of the owner's verdicts 1 to 34 (`inputs/JOB.md`, "Owner verdicts on run 4's open questions";
27 replaces 25): `V3/rules.md` (rules R1 to R18, the codes), `V3/vocabulary.md` (the words) and
`V3/changes.md` (what changed from v2). `RUN3` means `tasks/96/redesign/run3/`, `RUN4` means
`tasks/96/redesign/run4/`, `V2` means `tasks/96/redesign/run4/v2/`, `V3` means
`tasks/96/redesign/run4/v3/`. "R1" .. "R18" are the v3 rules; "row N" is the owner's agreed row N and
"verdict N" the owner's verdict N in `inputs/JOB.md` (v2 wrote "decision N" for verdicts 1 to 20; this
file says "verdict N" throughout); G-01 .. G-11 are the problems of `RUN3/22-re-attacker.md`.

No compiler change, no git command, no Ember program run, and no new `ember check`: this file makes no
new claim about what today's Ember accepts or refuses, and where it cites today's compiler on a new
spelling it cites the rule writer's probes (`V3/changes.md` section 7, `r1` .. `r8`). This file is phase
0's copy of `V2/area-D.md` (`V3/spelling-changes.txt` lists its replaced lines), edited in place: every
OLD block and every "means exactly" block in today's Ember, with its pasted output, is v2's byte for byte
(checked by `target/redesign4/v3-05-area-D/check_carried.py`, output in `check_carried.txt`); only the
prose, the NEW blocks, the owner's-test tables, the refusals and the costs changed. One new C check was
run in this step, on Linux in a cloud container: `target/redesign4/v3-05-area-D/c/an62b_region.c`, AN-62
(b) in a deferred-drop region, built with gcc 13.3 and AddressSanitizer (output in
`c/an62b_region_output.txt`; no timing). The C files and outputs v2 and run 3 cite (`c/...`,
`c\an62_an63_v2.c`) were on the owner's laptop and are not in this checkout; their pasted outputs carry
over unchanged. What changed from v2 is said once per answer, in its "v3" line, and in the table below;
what changed before that is in `V2/area-D.md`.

Each answer has the eight parts: 1 intent; 2 OLD with today's output; 3 NEW; 4 the owner's test for
every declared name; 5 means exactly (today's Ember or C doing what NEW means); 6 still refused, each with
its code and message; 7 cost; 8 kind. In the owner's-test tables "none (default: ...)" means no word in
the line answers that question and the stated default does (allowed only where the default never changes
anything, never aliases silently and never copies silently).

## Terms used in this area

- **function value**: a function or lambda kept in a variable, a list or a field (a callback, a
  handler): two words, its code and its **state** (what it captured with a word). Its type says
  everything it may change but its own state (R12): `fn()` nothing at all, `fn() changes LOG` the module
  variable `LOG`, `fn() changes score` the variable `score` it captured, `fn() changes Ui` objects of type
  `Ui` it reaches through a link it holds, and, in the type of a field, list or parameter that keeps
  functions, `fn() changes captured` (each function may change the one variable it captured) or
  `fn() changes captured...` (each may change several at once) (verdict 22); a function that runs them
  repeats that word on its first line (verdict 35).
- **capture**: a name a lambda uses from the function that made it. **The real variable** (row 6): that
  very variable, not a copy; a write in the lambda is seen by the function, and back. A lambda that
  changes a real variable it captures names it on its first line (`fn() changes score:`, verdict 22); it
  is checked in the function that writes the lambda (R12).
- **capture list**: what follows `with` after a lambda's parameters (`fn() with copy of score:`).
- **change list**: a generic parameter written after `changes` (`[C]` above `fn each_name(owns f: fn(owns
  String) changes C) changes C:`, verdict 50): it stands for what the function given to the call changes, and the call
  counts as making those changes (R12, verdict 21).
- **kind test**: `if s is Circle as c:` (runs its block when `s` is a `Circle`, and names it `c` there)
  and `if s is Circle:` (tests only); the "not sure" downcast. `cast s as Circle` is the "sure" one: it
  stops the program if `s` is not a `Circle` (R14, verdict 27).
- **box**: a small counted heap block holding one captured variable, shared by every user of the
  variable (Python's "cell").
- **holder**: the object or list whose field or element holds a function value or another object.
- **opening**: R13's `if cam.target exists as p:` (or `while cur.parent exists as p:`): the block runs when the
  `Option` or the `weakref` link holds something, and names it `p` there.
- **parent link**: a `weakref` field naming the object that owns its holder, set by the program
  (`child.parent = weakref this`, then `this.children.push(child)`; R11).
- **header test**: the read of a `weakref` link's target's header to see whether the object still lives.
- **count pair**: one +1 and one -1 of an object's count around a use, to keep it alive across it.
- **deferred-drop region**: a stretch of code (a loop) during which an object whose last owner ends is
  set aside whole (it and everything it owns marked ended) and freed when the stretch ends (R10 point 2).
- **AddressSanitizer** (ASan): a clang and gcc option that stops a program with a report the moment it
  reads or writes memory it must not.
- **today**: `ember check <file>` with today's compiler (`target/redesign3/bin/ember.exe` in run 3,
  `target/redesign4/bin/ember.exe` in run 4 and v2, on the owner's laptop; `target/redesign4/bin/ember
  check` in this checkout). Every claim about today has its output pasted.

The OLD code of every answer is unchanged, so every OLD part, with its `ember check` output, is copied
unchanged from `RUN4/area-D.md` through `V2/area-D.md` (file names in it are relative to
`target/redesign3/11-area-D/`).

## What v3 changed in each answer

"Meaning changed" compares with `V2/area-D.md`; what changed from run 4 to v2 is `V2/area-D.md`'s table
"Answers whose meaning changed from run 4". The verdicts that land in area D are 22 (what a lambda and a
list of functions say they change: AN-53, row 6, and AN-55 to AN-58), 21 (a function handed a function:
AN-55, AN-56), 27 (kind tests: AN-67, AN-68), 28 (the walk that may delete: AN-62), 30 (starting a
thread counts: AN-55) and 32 (the word on the name: AN-57). Verdicts 23, 24, 26 (done in phase 0), 29,
31, 33 and 34 change no line of area D's code: no answer of area D hands on a `shares` owner, makes a
value in an arena, takes a lock, calls C, uses `.copied()` or holds a `MutSpan` (verdict 31 is area E's,
AN-78). Every answer now cites "verdict N" and the v3 files.

| answer | what changed | meaning changed? |
|---|---|---|
| AN-53 (row 6) | row 6's handler says `changes score` (verdict 22); `Button`'s list is `Array[fn() changes captured]`, `on_click` takes `fn() changes captured`, `click` says `changes handlers, captured` (R12; verdict 35: the word its list's type uses); refusals added: row 6's v2 spelling (E2105), a two-variable handler (E2113), a handler stored in the button it changes (E3022), `click` without `captured` (E2105); the `LOG` handler's list becomes `Array[fn() changes captured, LOG]` | **yes**: v2's `handlers: Array[fn()]` with `b.on_click(fn(): score += 1)` is refused; the program does what it did, and row 6 reads as the owner agreed it |
| AN-54 | the v3 line; one refusal added: a handler that changes the real `score` without naming it (E2105) | no |
| AN-55 | the lambda given to `inspect` says `changes seen` (verdict 22), and `inspect`'s change list makes `count()`, which drives the adapter, the call that changes `seen` (verdict 21); starting the thread counts as making what its function changes (verdict 30), here nothing, so `main` names nothing; refusals added: the lambda without `changes seen` (E2105), the iterator line inside a loop over `seen` (E3020), a scoped task started inside a loop over what it changes (E3020) | **yes**, for one line: v2's `inspect(fn(v) => seen.push(v))` is refused (E2105); the program does what it did |
| AN-56 | the first lambda says `changes names` (verdict 22), and `each_name` takes a change list, `[C]` above `fn each_name(owns f: fn(owns String) changes C) changes C:` (verdict 21); the E2020 way out gives `fn(s) changes names: names.push(copy of s)` | **yes**: v2's `fn each_name(owns f: fn(owns String)):` no longer takes the first lambda (E2113); the program does what it did |
| AN-57 | the refused lambda says `changes total` (verdict 22), still E3026; one refusal added: the help followed with a plain `count: shares mut Counter` parameter (E3023, verdict 32) | no |
| AN-58 | the lambda says `changes score`, and the local list, written where `score` is visible, names it: `Array[fn() changes score]` (verdicts 22 and 40) | **yes**, for two lines: v2's `hs: Array[fn()]` with `fn(): score += 1` is refused; the program does what it did |
| AN-59 | the v3 line only: none of its lambdas changes a variable it captured | no |
| AN-60 | the E2113 message for `watch` with AN-53's `Button` names that button's v3 type; phase 0's wrong replacement in an E2113 message (an English "holds" turned into `exists as`) put right | no |
| AN-61 | the v3 line only | no |
| AN-62 | (b) is verdict 28's walk, word for word; its view walk is refused with R11's message (E3021); its cost follows R11's region (one header test per step; the weak-count moves go, since `Node` has no `drop` method), checked in C under ASan in this step, not timed; a refusal added: a function stored in the tree it changes (E3022, v2's note 1); v2's question for the owner answered by verdict 28 | no |
| AN-63 | the v3 line only | no |
| AN-64 | the v3 line only | no |
| AN-65 | the v3 line only | no |
| AN-66 | the v3 line only | no |
| AN-67 | the kind test's prose and owner's-test row (verdict 27); refusals rewritten: today's `cast ... if Pool[T]` and the removed `cast ... when Pool[T]`, a `match` on a maybe-downcast, `as?` and `as!` (E1010); added: a plain name from the kind test whose element is returned `ref mut` (E3023) | no |
| AN-68 | the OLD line is now refused as `cast s if Circle` (E1010, verdict 27) with the line meant, instead of v2's E2036; refusals rewritten and added: a test that always passes (E2036), `cast s when Circle` (E1010), `match cast s if Circle:` (E1010) | no |
| AN-69 | the v3 line only | no |
| AN-70 | the v3 line only | no |

---

## AN-53: A closure that changes a variable of the function that made it (A12; agreed row 6)

**v3:** row 6 reads as the owner agreed it in verdict 22, `b.on_click(fn() changes score: score +=
1)`. A plain `fn()` now changes nothing at all, captured variables included, so `Button`'s list covers
the variable each handler captured with `captured`, as the owner settled it (`handlers: Array[fn()
changes captured]`), and `click`, which runs them, says `changes handlers, captured`, the word its list's
type uses (R12; verdict 35). **Meaning changed:** yes, for v2's lines:
`handlers: Array[fn()]` with `b.on_click(fn(): score += 1)` is refused (E2105 for the lambda, then
E2113 for the list, part 6); the program does what it did: the button adds 1 to `game`'s real `score`.

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
    handlers: Array[fn() changes captured] = []      # each may change the one variable it captured

    fn on_click(owns f: fn() changes captured) changes handlers:
        this.handlers.push(f)

    fn click() changes handlers, captured:
        for ref mut h in this.handlers:
            h()

fn game() -> int:
    score = 0
    b = Button()
    b.on_click(fn() changes score: score += 1)       # row 6: game's real `score`
    b.click()
    return score                       # 1

fn game_copy() -> int:
    score = 0
    b = Button()
    b.on_click(fn() with copy of score: score += 1)   # row 6's copy: the handler's own `score`
    b.click()
    return score                       # 0: only the handler's copy went up
```

Changed from v2 (verdict 22): the lambda says `changes score`, the list's type says `changes captured`,
`on_click`'s parameter says the same, and `click` says `changes handlers, captured`. The calls write
no `ref mut`: `b.on_click(...)` and `b.click()` are methods called on `b`, whose first lines say what
they change (R4); `f` is an `owns` parameter, and a hand-over is written with no word (R4).

Row 6's handler changes `score`, a real variable of `game` that it captures, and says so on its first
line (`fn() changes score:`, R12; checked against its body both ways, E2105). The `Button` cannot name
`game`'s `score` in its own type, so its list covers it with `captured`: each function in it may change
the one variable it captured (a function that changes none fits too). `click` says `changes handlers`
and walks with `ref mut h` because a call through a stored function value may change its own state, so
it is a change of the place that holds it (R12), and `captured` because it runs functions of the type
`fn() changes captured`, and says what they change in their own word, as it would say `LOG` for
functions that change `LOG` (R12, verdict 35). In
`game`, the function that wrote the lambda, those variables are its own locals (`score`), so `game` names
nothing (R5); the compiler checks them there: `b` holds a function value that reaches `score`, and
`b.click()` counts as a change of `score` (R12), so `return score` reads it after the click. Where
`game`'s real `score` lives (R12): at its own address, since `b` never leaves `game` and both methods it
is handed to say what they change, so the handler provably ends first. In `game_copy` the captured
`score` is named (`copy of`), so it is the handler's own state: that handler changes nothing outside
itself, a plain `fn()`, which fits the same list (it changes fewer captured variables than `captured`
allows, R12).

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `handlers` (field) | none (default: the Button's own list; each element its own function value); its type `fn() changes captured`: each function may change the one variable it captured | the list: only in a method whose first line says `changes handlers` (or `changes this`); each function: `changes captured`, its own captured variable |
| `this` in `on_click` | `this`: the Button the method was called on | `changes handlers`: that field only |
| `f` | `owns`: handed over to `on_click`; its type `fn() changes captured` | its new owner may; it hands it on to the list (storing it changes nothing it changes, R12) |
| `this` in `click` | `this` | `changes handlers` (R12: calling a stored function value may change it) |
| what `click` runs | the handlers' captured variables | `changes captured`: the word of its list's type; each handler may change the one variable it captured (R12, verdict 35) |
| `h` | `ref mut`: an element of `handlers`, the only name for it while used (R8 (f)) | yes; calling it changes what its body changes, its own state included (R12) |
| `score` in `game` | none (default: `game`'s own `int`, made by `=`) | its owner may |
| `b` in `game` | none (default: `game`'s own Button, made by `=`) | through methods whose first line says `changes` |
| `score` in the lambda | none (row 6: the real `score` of `game`) | `changes score` on the lambda's first line: yes, the real one (row 6, verdict 22) |
| `score`, `b` in `game_copy` | none (default: `game_copy`'s own `int` and Button) | as in `game` |
| `score` in `game_copy`'s lambda | `copy of`: the handler's own copy (row 6) | yes, its own copy (its own state, which needs no word, R12), through `click`'s `changes handlers` |

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

That is the box of R12. Here R12 places `score` at its address instead (the handler provably ends first,
above): the C is `c/an53_captures.c` case 1, `on_click(&b, (Fn0){bump, &score})`, run under ASan:

```text
game=1
counter=2
an53_captures ms=0.040 peak_kb=8176 checksum=0
```

(`counter=2` is case 2, a counter factory whose variable lives in a box because the function value is
returned.) No ASan report. `game_copy` means exactly the OLD's `old/an53_button_owned_fn.em`
(`b.on_click(owned fn(): score += 1)` with `.iter_mut()`, `exit=0`, pasted in part 2): today's `owned fn`
copies the `int`, which is what `with copy of score` names.

**6. Still refused.**
- A capture of someone else's place in a function value that may outlive it (AN-57), E3026: "the click
  handler changes `total`, which belongs to `register`'s caller, and `b` keeps the handler after
  `register` returns; keep the total in a value the handler can own a share of: declare `struct Counter:` with `n: int = 0`; in the caller write `count = Counter()` and call `register(ref mut b, shares mut count)`; write `fn register(ref mut b: Button, owns count: shares mut Counter):` and `b.on_click(fn() with owns count changes Counter: count.n += 1)`; and in `Button` keep such handlers as `handlers: Array[fn() changes Counter]`, with `fn on_click(owns f: fn() changes Counter) changes handlers:` and `fn click() changes handlers, Counter:`".
- A capture used by a thread that may outlive it (AN-55), E7004.
- Row 6's v2 spelling, `b.on_click(fn(): score += 1)`, E2105 (verdict 22: a plain `fn()` changes nothing
  at all): "this function changes `score`, a variable it uses from `game`; say so on its first line:
  `b.on_click(fn() changes score: score += 1)`" (R12).
- v2's `handlers: Array[fn()]`, with row 6's handler as written above, E2113: "this function changes
  `score`, a variable it captured (`fn() changes score`), and `on_click` takes `fn()`, functions that
  change nothing; keep it in a list that says so: `handlers: Array[fn() changes captured]`, with
  `fn on_click(owns f: fn() changes captured) changes handlers:`" (R12).
- A handler that changes two variables it captured, `b.on_click(fn() changes score, lives: ...)` whose
  body adds to `score` and takes from `lives`, E2113 (the owner's refinement of verdict 22): "`fn()
  changes score, lives` changes 2
  captured variables, and `handlers` allows 1 (`Array[fn() changes captured]`); for functions that change
  several at once write `handlers: Array[fn() changes captured...]`, `fn on_click(owns f: fn() changes
  captured...) changes handlers:` and `fn click() changes handlers, captured...:`" (R12).
- A handler that changes the button that keeps it, `b.on_click(fn() changes b: b.handlers.clear())`,
  E3022: "this function changes `b`, and `on_click` would keep it in `b.handlers`, so it would change the
  button that holds it while the button runs it; clear the handlers after the click instead:
  `b.click()`, then `b.handlers.clear()`" (R12: a function value is not stored in a place that a real
  variable it changes owns, as Rust refuses).
- `fn click():` with `for ref mut h in this.handlers: h()`, E2105: "`click` calls the handlers stored in
  `this.handlers`, which may change their own state and the variable each captured; say so on its first
  line, `fn click() changes handlers, captured:`, and walk them with `for ref mut h in
  this.handlers:`" (R12). With `changes handlers` alone, E2105: "`click` runs the handlers in
  `this.handlers`, whose type says `changes captured`; add it to its first line: `fn click() changes
  handlers, captured:`".
- A loop that walks a list a function value changes, around a call that may run it (R12, R8 (c)),
  E3020: "this loop walks `xs` (line 5), and `hs[0]()` (line 6) may run the function made on line 3,
  which pushes to `xs`; call it after the loop, as many times as the loop would: write `n = xs.len()` before the loop, and `for k in 0..n: hs[0]()` below it instead of `hs[0]()` in it". The
  same holds for `b.click()` inside a loop over a list of `game`'s that a handler in `b` changes: in
  `game`, which wrote the lambda, the call counts as each change the lambda names (R12).
- A handler that changes a module variable says so, and is kept only where the type says so (R12,
  verdict 17): `b.on_click(fn() changes LOG: LOG.push(String("clicked")))` with this `Button`, E2113:
  "this function changes `LOG` (`fn() changes LOG`), and `on_click` takes `fn() changes captured`,
  functions that change at most the one variable they captured; keep it in a list that says so:
  `handlers: Array[fn() changes captured, LOG]`, with `fn on_click(owns f: fn() changes captured, LOG)
  changes handlers:`". With such a list, `click` says `changes handlers, captured, LOG`, the words of
  its list's type (E2105 otherwise: "`click` runs the handlers in `this.handlers`, whose type says
  `changes LOG`; add it to its first line: `fn click() changes handlers, captured, LOG:`"), and
  `b.click()` inside a loop over
  `LOG` is refused, E3020: "this loop walks `LOG` (line 16), and `b.click()` (line 17) changes `LOG`:
  `click` runs the handlers of `b`, an `Array[fn() changes captured, LOG]` (its first line says `changes
  handlers, captured, LOG`); click after the loop". A button whose handlers change only the variables
  they captured, as here, is clicked inside a loop over `LOG` with no refusal (G-07 settled by the type,
  not by a search of the program).

**7. Cost.** At an address: none (the C passes `&score`). In the function value's own state
(`game_copy`): nothing beyond the function value's own block. In a box: one allocation per variable
and one load per use. Run 3's W4 (callbacks that remove themselves) measured the state placement at
1.091x of plain C, a box per captured variable at 1.255x, safe Rust at 1.358x (`RUN3/26-timer.md` lines
29, 31, 32: `rust_w4`, `model_w4`, `model_w4_state_block`); the phase before measured the callback
workload at 1.018 and 1.052 of plain C with the shared total boxed, against 1.045 and 1.037 with it by
address (`target/redesign2/graphs-callbacks-attack2/bench/runs/w4_a.txt`, `w4_b.txt`, summarised in
`phase3b-graphs-callbacks-attack.md` 6.4). Rust's spelling of the same program is `Rc<Cell<i32>>`, the
same box. The words verdict 22 adds (`changes score`, `changes captured`, `captured...`) cost nothing
while the program runs: they are checked when compiling (`V3/rules.md` section 2).

**8. Kind:** a smarter compile-time solution (Rust refuses the program; here the compiler proves the
cheaper place and otherwise uses the box Rust makes the programmer write), with what each handler
changes written on it, as verdict 22 asks.

---

## AN-54: `owned fn` moves a `String` capture and copies an `int` one (agreed row 6)

**v3:** NEW as in v2. Under verdict 22 the handler is still a plain `fn()`: what it changes is its own
state (its own `name` and its own copy of `score`), which needs no word (R12). One refusal added: the
same handler with `score` left out of its capture list changes `main`'s real `score` and must say so.
Meaning unchanged.

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
    cb = fn() with owns name, copy of score:
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
  `fn() with owns name, copy of score:`; a name not listed is the real variable".
- `owns fn(): ...`, E0109: "`owns fn` does not say what the function takes or copies; name it on each
  captured name: `fn() with owns name, copy of score:`".
- `fn() with owns score:` for an `int`, E2085: "an `int` is copied, not handed over; write
  `with copy of score`".
- `println(name)` after `with owns name`, E3040: "`name` was handed to the function on line 7; to keep
  using it here, give the function a copy, `with copy of name`, or share it, `with shares name`".
- `fn() with copy of name, score:`, E0116: "`copy of` here belongs to `name` only; say what the function
  does with `score`: `with copy of name, copy of score`, or leave `score` out to use the real `score`".
- `cb = fn() with owns name:` with the same body (no word for `score`), E2105 (verdict 22): "this
  function changes `score`, a variable it uses from `main`; say so on its first line: `cb = fn() with
  owns name changes score:`, or give it a copy of its own: `cb = fn() with owns name, copy of score:`"
  (R12). Written with `changes score`, it no longer fits `run`'s `fn()`, E2113: "`cb` changes `score`, a
  variable it captured (`fn() changes score`), and `run` takes `fn()`, functions that change nothing;
  take it as one that may, and say that `run` runs it: `fn run(ref mut f: fn() changes captured)
  changes captured:`" (R12: a function that runs one function whose type says `captured` says
  `captured`).
- `run(cb)` with `fn run(f: fn()):`, E3023: "`cb` changes its own copy of `score` each time it runs, and
  `run` takes it read-only; write `fn run(ref mut f: fn()):` and call `run(ref mut cb)`" (R12).
- `run(cb)` with `fn run(ref mut f: fn()):`, E3028: "`run` may change `cb` (its first line says
  `ref mut f: fn()`); say so at the call: `run(ref mut cb)`" (R4).

**7. Cost.** None beyond C: the `String`'s three words move into the function's state, the `int` is
copied, as a Rust `move` closure.

**8. Kind:** a readable spelling of Rust's guarantee (`move`, written per name).

---

## AN-55: The compiler decides a capture is moved from which function receives the lambda (`[CLO-15]`)

**v3:** the lambda given to `inspect` changes `seen`, a variable it captured, so it says so on its first
line, `fn(v) changes seen: seen.push(v)` (verdict 22), and `inspect` keeps it with its change list, so
`count()`, which drives the adapter, is the call that changes `seen` (verdict 21; the iterator row of
`V3/vocabulary.md`). Starting a thread now counts as making what its function changes (verdict 30); the
thread's function here changes only its own state (it owns `xs`), so `main` names nothing. **Meaning
changed:** yes, for one line: v2's `inspect(fn(v) => seen.push(v))` is refused (E2105, part 6); the
program does what it did.

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
    _n = xs.iter().inspect(fn(v) changes seen: seen.push(v)).count()   # the real `seen`
    h = thread.spawn(fn() with owns xs: work(xs))                      # `xs` handed to the thread
    println(seen.len())
    println(h.join())
```

(`work` is the OLD's: it reads the list it is given and returns a number.) Changed from v2: `changes
seen` on the lambda (verdict 22). The library's first line is `[C, F: fn(Item) changes C]` above `fn
inspect(owns f: F) owns this -> Inspect[Self, C, F]:` (verdicts 21 and 50, `V3/vocabulary.md`'s iterator row): `inspect`
keeps the function in the adapter, which changes nothing yet (storing a function is not a change, R12),
and the adapter's `fn next() changes this, C -> Option[Item]` runs it, so `count()`, which calls `next`
until the end, is the call that pushes to `seen` (`C` is `seen` here). In `main`, which wrote the
lambda, `seen` is its own local, so `main` names nothing (R5). One instance of `inspect` is made for
this lambda (the bound `F`), so the call through `f` is direct, as Rust's `F: FnMut`.

The thread: `thread.spawn`'s first line is `[R, CHANGED]` above `pub fn spawn(owns f: fn() changes
CHANGED -> R) changes CHANGED -> JoinHandle[R]:` (verdicts 30 and 50): starting the thread counts as making what its
function changes, now or later (R5, R12, R15). The thread's function holds `xs` (`with owns xs`, R15)
and changes nothing but its own state, so `CHANGED` is empty and `main` names nothing; its first line
stays `fn main():`.

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `xs` in `main` | none (default: `main`'s own list) | its owner may |
| `seen` in `main` | none (default: `main`'s own list) | its owner may |
| `v` | none (default for a lambda parameter: it reads; an `int`, copied in) | no |
| `seen` in the lambda | none (row 6: the real `seen`) | `changes seen` on the lambda's first line: yes (`push`), the real one (verdict 22) |
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

**6. Still refused.** `thread.spawn(fn() => work(xs))`, E7004: "this thread may outlive `main`, so it
cannot use `main`'s `xs`; say what the thread holds: `fn() with owns xs:` (handed over) or
`fn() with copy of xs:` (a copy of its own), or start it inside `with scope = thread.scope():`, which
waits for it" (R15). `fn() with shares xs:` on `thread.spawn`, E7015: "this thread runs beside `main`,
and `shares` owners stay on one thread; hand `xs` over, `fn() with owns xs:`, or give the thread a copy,
`fn() with copy of xs:`". A box (R12) is never used across threads: its count is a one-thread count.
And, under verdicts 21, 22 and 30:
- v2's iterator line, `xs.iter().inspect(fn(v) => seen.push(v)).count()`, E2105: "`inspect` keeps this
  function and `count()` runs it, and it pushes to `seen`, a variable it uses from `main`; say so on its
  first line: `xs.iter().inspect(fn(v) changes seen: seen.push(v)).count()`" (R12).
- The iterator line inside a loop over `seen`, `for s in seen:` (line 3) with the line above as line 4,
  E3020: "this loop walks `seen` (line 3), and `xs.iter().inspect(...).count()` (line 4) changes `seen`
  (`count` runs the function `inspect` keeps, which says `changes seen`); count after the loop: write
  `_n = xs.iter().inspect(fn(v) changes seen: seen.push(v)).count()` after it" (R8 (c), R12).
- A scoped task started inside a loop over what its function changes, in the lines `with scope =
  thread.scope():` (line 3), `for s in seen:` (line 4), `scope.spawn(fn() changes seen: seen.push(1))`
  (line 5), E3020 (verdict 30): "this loop walks `seen` (line 4), and `scope.spawn(...)` (line 5) starts
  a task whose function changes `seen` (`fn() changes seen: seen.push(1)`); start the task after the
  loop: write `scope.spawn(fn() changes seen: seen.push(1))` after it" (R8 (c), R15).

**7. Cost.** None beyond C: the lambda given to `inspect` holds `seen`'s address and is called directly
(one instance of `inspect` per function given), the hand-over moves three words. The change lists of
`inspect` and `spawn` cost nothing while running (`V3/rules.md` section 2).

**8. Kind:** a readable spelling of Rust's guarantee (`move` and scoped threads), with no rule that
depends on the callee.

---

## AN-56: A lambda parameter without a mode takes `owned` from the expected callable type (`[TYP-23]`)

**v3:** the first lambda changes `names`, a variable it captured, so it says so on its first line,
`fn(owns s) changes names: names.push(s)` (verdict 22; written with `:`, since the vocabulary shows
`changes` only on a lambda written with `:`, "Notes for the lead", note 3). A plain `fn(owns String)`
now changes nothing at all, so `each_name`, a function that runs a function it is given, takes a change
list, `[C]` above `fn each_name(owns f: fn(owns String) changes C) changes C:`, the spelling of verdict 21, and a
call of it counts as making what its function changes. **Meaning changed:** yes: v2's `fn
each_name(owns f: fn(owns String)):` no longer takes the first lambda (E2113, part 6); the program does
what it did.

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
[C]
fn each_name(owns f: fn(owns String) changes C) changes C:
    f(String("ann"))

fn main():
    names: Array[String] = []
    each_name(fn(owns s) changes names: names.push(s))   # takes `s` over, keeps it in the real `names`
    each_name(fn(s) => println(s.len()))                  # only reads `s`, changes nothing
```

Changed from v2: `each_name`'s first line (verdict 21) and `changes names` on the first lambda (verdict
22). `f(String("ann"))` and `each_name(...)` hand their arguments over to `owns` parameters, written with
no word (R4). `C` is filled at each call from the function given (R12): the first call changes `names`,
which in `main`, the function that wrote the lambda, is its own local, so `main` names nothing (R5); the
second lambda is a plain function type, which changes nothing, so that call changes nothing. Written with
a function value as the verdict writes it, `f` is called through its two words; the library's form takes
it as a bound (`[C, F: fn(owns String) changes C](owns f: F)`, `V3/changes.md` section 11, question 4),
which makes one instance per function given.

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `f` | `owns`: handed over to `each_name`; its type `fn(owns String) changes C`: a function that may change what the function given changes | its new owner may; `each_name`'s own `changes C` says that calling it makes those changes |
| `names` | none (default: `main`'s own list) | its owner may |
| `s` in the first lambda | `owns`: handed over to the lambda | its owner may (it hands it on to `names`) |
| `names` in the first lambda | none (row 6: the real `names`) | `changes names` on the lambda's first line: yes (`push`), the real one (verdict 22) |
| `s` in the second lambda | none (default for a parameter: a read-only view) | no |

**5. Means exactly.** `means/m56_lambda_owned_param.em` (`fn(owned s) => names.push(s)`) and
`means/m56_lambda_read_param.em` (`fn(s: String) => println(s.len())` where `fn(owned String)` is
expected):

```text
=== m56_lambda_owned_param.em
exit=0
=== m56_lambda_read_param.em
exit=0
```

A read-only lambda where a taking one is expected: the caller hands the `String` over, the lambda reads
it, and it ends after the call, as if the function had taken it and dropped it.

**6. Still refused.**
- `each_name(fn(s) => names.push(s))` (no word on `s`, and none for `names`), E3013: "`s` is only read
  here, so it cannot be kept in `names`; to take it over, and say that the function changes `names`,
  write `each_name(fn(owns s) changes names: names.push(s))`".
- `each_name(fn(owns s) => names.push(s))`, v2's line, E2105 (verdict 22): "this function pushes to
  `names`, a variable it uses from `main`; say so on its first line: `each_name(fn(owns s) changes names:
  names.push(s))`" (R12).
- v2's `fn each_name(owns f: fn(owns String)):`, given the first lambda, E2113 (verdict 21): "this
  function changes `names`, a variable it captured (`fn(owns String) changes names`), and `each_name`
  takes `fn(owns String)`, functions that change nothing; let it take one that changes things, and say
  that a call makes those changes: `[C]` above `fn each_name(owns f: fn(owns String) changes C) changes C:`"
  (R12).
- `fn(owns s)` where the expected type is `fn(String)` (a read-only parameter), E2020 (a lambda
  parameter whose mode differs from the expected function type): "`each_name` gives each name read-only,
  so this function cannot take it over; to keep your own, write `fn(s) changes names: names.push(copy of
  s)`".

**7. Cost.** None: the same code as today. The change list costs nothing while running (`V3/rules.md`
section 2).

**8. Kind:** a readable spelling of Rust's guarantee (the parameter's mode is written, as on every other
parameter, R4).

---

## AN-57: E3026 is reported inside the callee; a stored lambda gets E2020 naming `Callable0`

**v3:** the refused lambda says what it changes, `fn() changes total: total += 1` (verdict 22), and is
still refused for the same reason (E3026): it would outlive `total`. It fits AN-53's v3 `Button`
(`fn() changes captured`), so E3026 is the only refusal. The E3026 message was fixed by the lead in phase 3
(`V2/readability.md` F-03): it now gives every line, `struct Counter:` with `n: int = 0` included. One refusal added, for verdict 32: the
help followed in `register` with a plain `count: shares mut Counter` parameter (E3023). Meaning
unchanged.

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
in the program's names:

```ember
fn register(ref mut b: Button, ref mut total: int):
    b.on_click(fn() changes total: total += 1)
```

```text
error[E3026]: the click handler changes `total`, which belongs to `register`'s caller
 --> app.em:2:16
  |
2 |     b.on_click(fn() changes total: total += 1)
  |                ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ written here, kept in `b`
  |
  = why: `b` keeps the handler after `register` returns, and `total` may have ended by then
  = help: keep the total in a value the handler can own a share of:
          declare `struct Counter:` with `n: int = 0`;
          in the caller write `count = Counter()` and call `register(ref mut b, shares mut count)`;
          write `fn register(ref mut b: Button, owns count: shares mut Counter):`
          and `b.on_click(fn() with owns count changes Counter: count.n += 1)`;
          and in `Button` keep such handlers as `handlers: Array[fn() changes Counter]`,
          with `fn on_click(owns f: fn() changes Counter) changes handlers:` and `fn click() changes handlers, Counter:`
```

Callable types are printed as written: `fn()`, `fn(ref mut Ui)`, `fn() changes Counter`, never
`Callable0` (R18). Changed from v2: `changes total` on the lambda (verdict 22). The handler in the help
changes a `Counter` through an owner it holds (`with owns count`: the owner `register` was handed,
now the handler's own state; one more owner of the counter, beside the caller's `count`), so its type
says `changes Counter` (R12, verdict 17): a change of an object through a link,
named by its type, not a change of a variable it captured, so AN-53's `Button`, whose list holds
`fn() changes captured`, could not keep it (E2113); the help names both lines (R18). The message is
`V3/rules.md` section 3's E3026 row.

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `b` | `ref mut`: the caller's Button | yes (`ref mut`) |
| `total` | `ref mut`: the caller's `int` | yes (`ref mut`) |
| `total` in the lambda | none (row 6: the real `total`, here the caller's place) | `changes total` on the lambda's first line: yes, as row 6 says; refused because the handler may outlive it |

**5. Means exactly.** Nothing compiles for the refused program, which is the point. The accepted form in
the help is AN-53's box (`means/m53_button_box.em`, `exit=0`, pasted under AN-53); its
`shares mut Counter` is today's class `Score` there.

**6. Still refused.** The program above, with the message above (E3026). A lambda given to a callee that
keeps it while the lambda captures a view (a `ref`, `ref mut`, pattern, opened or `for` name): the same
message shape, at the lambda's line, naming the captured view and the holder that keeps the lambda.

The help followed inside `register` instead of in the caller, with the counter taken as a plain
parameter: `fn register(ref mut b: Button, count: shares mut Counter):` with `b.on_click(fn() with
shares mut count changes Counter: count.n += 1)`, E3023 (verdict 32: what a function may do with a
parameter is set by the word in front of its name, R4): "`count` only reads the `Counter` that
`register`'s caller lends it (`shares mut Counter` says what the caller's counter is, not what
`register` may do with it), so an owner made from it only reads too; to hand on an owner that may change
the counter, take one: write `fn register(ref mut b: Button, owns count: shares mut Counter):`, call it
`register(ref mut b, shares mut count)`, and capture it `with owns count`". (`register` names nothing it
changes for the handler: it only stores it, which is not a change, R12.)

**7. Cost.** None (compile time only).

**8. Kind:** an understandable limitation kept (a function value may not outlive a place it does not
own; Rust's E0373), with the message at the right line and in the program's names.

---

## AN-58: A closure stored in a local list that ends first is still E3026

**v3:** the lambda changes `score`, a variable it captured, so it says `changes score`, and the local
list, written where `score` is visible, names it: `hs: Array[fn() changes score] = []` (verdict 40: a
name visible where the line is written is written by its name; `captured` is for what it cannot see).
`game`, which wrote the lambda and runs it, names nothing: `score` is its own local (R5). **Meaning changed:** yes, for two lines: v2's `hs: Array[fn()] = []` with
`hs.push(fn(): score += 1)` is refused (part 6); the program does what it did.

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
    hs: Array[fn() changes score] = []
    hs.push(fn() changes score: score += 1)
    for ref mut h in hs:
        h()
    return score                       # 1
```

A call through a stored function value may change its own state, so its holder is reached through a
name that may change it (R12): the walk is `for ref mut h in hs:`, as AN-53's `click`. `hs` holds
functions that change `score`, and its type says so, `Array[fn() changes score]` (verdict 40); the lambda changes `score`, a real
variable of `game`, and says so (R12, verdict 22). `h()` changes `score`, which the loop does not walk,
so the loop is allowed (R8 (c)), and `game` names nothing on its first line, since `score` is its own
local (R5).

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `score` | none (default: `game`'s own `int`) | its owner may |
| `hs` | none (default: `game`'s own list; its elements its own function values); its type `fn() changes score`: each may change `game`'s `score` | its owner may |
| `score` in the lambda | none (row 6: the real `score`) | `changes score` on the lambda's first line: yes, the real one (verdict 22) |
| `h` | `ref mut`: an element of `hs`, the only name for it while used | yes; calling it changes what its body changes (R12) |

**5. Means exactly.** `hs` is declared after `score` and never leaves `game`, so the function value
provably ends first and R12 places `score` at its address: the C of `c/an53_captures.c` case 1 (the same
`{code, &score}` pair, there kept in a local `Button`), clean under ASan, output pasted under AN-53
(`game=1`).

**6. Still refused.**
- `for h in hs: h()`, E3023: "`h` only reads the function value in `hs`, and calling a stored function
  value may change its own state; walk the list with `for ref mut h in hs:`" (R12).
- A loop over a list that a stored function value changes, around a call that may run it (R12,
  R8 (c)): `for x in xs: hs[0]()` where `hs[0]` pushes to `xs`, E3020: "this loop walks `xs` (line 5),
  and `hs[0]()` (line 6) may run the function made on line 3, which pushes to `xs`; call it after the loop, as many times as the loop would: write `n = xs.len()` before the loop, and `for k in 0..n: hs[0]()` below it instead of `hs[0]()` in it". (The phase before's `attacks2.exe 11` showed
  the use after free this refusal prevents.)
- v2's lines, `hs: Array[fn()] = []` and `hs.push(fn(): score += 1)`: first E2105 (verdict 22): "this
  function changes `score`, a variable it uses from `game`; say so on its first line: `hs.push(fn()
  changes score: score += 1)`"; then, with that line, E2113: "this function changes `score`, a variable
  it captured (`fn() changes score`), and `hs` holds `fn()`, functions that change nothing; keep it in a
  list that says so: `hs: Array[fn() changes score] = []`" (R12).

**7. Cost.** None (the C passes an address).

**8. Kind:** a smarter compile-time solution (the list's end is proved; Rust needs the same proof through
lifetimes, and accepts this program).

---

## AN-59: A closure that gives a capture away gets E3013 with ten options instead of "declare the parameter `once fn()`"

**v3:** as in v2. Under verdict 22 both lambdas stay plain `fn()`: the first changes only its own
state (it owns `s` and gives it away), the second only reads the real `t` (`copy of t` makes the value
it hands over), so neither changes a variable it captured, and neither needs `changes`. Verdict 33 does
not touch this answer: `copy of` is the copy word, and no `.copied()` appears. Meaning unchanged.

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
    call_once(fn() with owns s: consume(s))      # the function owns `s` and gives it away: once
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
- `call_twice(fn() with owns s: consume(s))`, E3013: "this function gives `s` away, so it can run only
  once, and `call_twice` may run it twice; if `call_twice` runs it once, declare the parameter
  `owns f: once fn()`; otherwise hand over a copy each time: `fn(): consume(copy of s)`".
- `fn call_once(f: once fn()):`, E3013: "`call_once` uses `f` up, so it must hold it: write
  `owns f: once fn()`" (R12).
- `call_twice(fn(): consume(s))` (no word), E3013: "`consume(s)` would hand `main`'s `s` over from inside
  a function that `main` keeps; give the function its own `s` to hand over, `fn() with owns s:
  consume(s)` (then it runs once), or hand over a copy, `consume(copy of s)`".

**7. Cost.** None: the same code as today's `once fn` and `.clone()`.

**8. Kind:** a readable spelling of Rust's guarantee (`FnOnce`), with the message the annoyance asks for.

---

## AN-60: A callback receiving the object that holds it, and a handler that removes itself (A32)

**v3:** NEW as in v2. Verdict 22 changes nothing in it: the handlers `fn(ref mut Ui)` change the Ui
through their `ref mut` parameter, which needs no `changes` (R12), and `bump_ui` changes a `Ui` through
the link it holds and already says `changes Ui`. Two messages put right: the E2113 message for `watch`
with AN-53's `Button` names that button's v3 type, `fn() changes captured`, which a function that
changes a `Ui` through a link does not fit; and phase 0 had turned an English "holds" in an E2113
message into `exists as` (`V3/spelling-changes.txt`, area-D.md:982), now "holds" again. Meaning
unchanged.

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

    fn fire() changes this:
        this.current = 0
        while this.current < len(this.handlers):
            this.handlers[this.current](ref mut this)   # the handler may add or remove handlers
            this.current += 1

    fn remove_current() changes handlers, current:
        this.handlers.remove(this.current)
        this.current -= 1

fn count(ref mut ui: Ui):
    ui.clicks += 1

fn add_once(ref mut ui: Ui):                       # adds a handler, then removes itself
    ui.handlers.push(count)
    ui.remove_current()

class UiButton:                                    # a button whose handlers may change a Ui
    handlers: Array[fn() changes Ui] = []

    fn on_click(owns f: fn() changes Ui) changes handlers:
        this.handlers.push(f)

fn watch(ref mut ui: Ui, ref mut b: UiButton):
    bump_ui = fn() with weakref ui changes Ui:     # a link that does not keep the Ui alive
        if ui exists as ref mut me:                # the Ui still lives: call it `me`, which may change it
            me.clicks += 1
    b.on_click(bump_ui)
```

Python's list iterator is this loop (a position checked against the length at every step), so a handler
added during the dispatch runs in it and a handler that removes itself makes the next one move into its
place, as in Python.

`fn fire() changes this:` (R5, verdict 6), the opening `if ui exists as ref mut me:` (R13, verdict
26) and `with weakref ui` (verdict 9) are the owner's words. `fire` says `changes this`, not `changes handlers, current`, because it gives the whole Ui to a
handler that may change any field (R4: `ref mut` only before what this function may change, E3027). The
call says `ref mut this` (R4: the handler's parameter is `ref mut Ui`); it copies the handler's two words
out of the list first, so the list is not an argument of the call (R12) and nothing points into it while
the handler runs. The handlers' type `fn(ref mut Ui)` names no change outside the handler: what it
changes, it changes through its `ref mut` parameter (R12).

`watch` (v2's change from run 4, verdict 17): `bump_ui` changes a `Ui` through a link it holds (`with
weakref ui`), which is outside itself, so its first line says `changes Ui` (R12; E2105 otherwise), and
it is kept only in a list whose type says so. AN-53's `Button` holds `fn() changes captured`, functions
that change at most the one variable they captured, so `b: Button` would be refused (E2113); `watch`
takes a `UiButton`, whose list says `changes Ui`. `watch` itself names nothing: `b.on_click(bump_ui)`
stores the handler, which is not a change of what it changes (R12).

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `handlers` (in `Ui`) | none (default: the Ui's own list of function values) | `changes handlers`, or `changes this` |
| element type `fn(ref mut Ui)` | `ref mut` on its parameter: a handler gets the caller's Ui | yes, the Ui (`ref mut`) |
| `current`, `clicks` | none (default: the Ui's own `int`s) | `changes current`, or `changes this` |
| `this` in `fire` | `this`: the Ui `fire` was called on | yes: `changes this` (any field) |
| `this` in `remove_current` | `this` | `changes handlers, current`: those two fields |
| `ui` in `count` | `ref mut`: the caller's Ui | yes |
| `ui` in `add_once` | `ref mut`: the caller's Ui | yes |
| `handlers` (in `UiButton`) | none (default: the button's own list); its type `fn() changes Ui`: each function may change a `Ui` | `changes handlers`, or `changes this` |
| `this` in `on_click` | `this` | `changes handlers` |
| `f` | `owns`: handed over to `on_click` | its new owner may; it hands it on to the list |
| `ui` in `watch` | `ref mut`: the caller's Ui | yes |
| `b` in `watch` | `ref mut`: the caller's UiButton | yes (its `on_click` changes `handlers`) |
| `bump_ui` | none (default: `watch`'s own function value, made by `=`); `changes Ui` on its first line | its owner may; handed over to `on_click` with no word (an `owns` parameter) |
| `ui` in the `with weakref ui` lambda | `weakref`: a link that does not keep the Ui alive | no (a link); `me` may (below) |
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
keeps the handler's state in a box (`box_new`); R12 places a variable that only this handler uses in the
handler's own state block instead, which run 3's W4 measured (part 7). `watch`'s handler is the
`weakref` read of AN-65 (`c/an63_links.c` part 2: a counted link, read with a test of the header).

**6. Still refused.**
- `fn fire() changes this:` with `for ref mut h in this.handlers:` (line 7) and `h(ref mut this)`
  (line 8), with `fn(ref mut Ui)` handlers (the agreed limitation: a list may not change while a loop
  walks it), E3020: "this loop walks `this.handlers` (line 7), and `h(ref mut this)` (line 8) gives each
  handler the whole Ui, so a handler may add to or remove from `this.handlers`; walk by position, as
  Python's list does: instead of the `for` line and `h(ref mut this)`, write `this.current = 0`,
  `while this.current < len(this.handlers):`, `this.handlers[this.current](ref mut this)` and
  `this.current += 1`".
- v2's change from run 4 (verdict 17): the second way out, read-only handlers walked with `for`
  (`handlers: Array[fn(Ui)]`, `fn fire() changes handlers:`, `for ref mut h in this.handlers: h(this)`),
  is accepted for every program: a `fn(Ui)` changes nothing outside itself, so no handler in that list
  can change any Ui's `handlers` while the loop walks them. G-05's handler, which captures
  `with weakref ui` and pushes to `me.handlers`, has the type `fn(Ui) changes Ui` (R12); given to that
  list it is refused, E2113: "this handler changes a `Ui` through `with weakref ui` (`fn(Ui) changes
  Ui`), and `handlers` holds `fn(Ui)`, functions that change nothing outside themselves; give it the Ui
  to change as its parameter, `fn(ref mut Ui)`, or keep it in a list that says so,
  `handlers: Array[fn(Ui) changes Ui]`". Kept in a list that says so (with `fn fire() changes handlers,
  Ui:`), the `for` walk is refused, E3020: "this loop walks `this.handlers` (line 7), and `h(this)` may
  change any `Ui`, this one included (the handlers' type says `changes Ui`); give each handler the Ui to
  change, `handlers: Array[fn(ref mut Ui)]`, and walk by position instead of the `for` line:
  `this.current = 0`, `while this.current < len(this.handlers):`,
  `this.handlers[this.current](ref mut this)`, `this.current += 1`, with `fn fire() changes this:`".
- `if Some(ref mut me) = ui:`, E0115: "an `Option` or a `weakref` link is opened with `exists as`: write
  `if ui exists as ref mut me:`" (R13).
- `bump_ui = fn() with weakref ui:` (no `changes Ui`) with `me.clicks += 1` in its body, E2105: "this
  function changes `me`, a `Ui` reached through its `weakref ui`; say so on its first line:
  `fn() with weakref ui changes Ui:`" (R12).
- `watch(ref mut ui: Ui, ref mut b: Button)` with AN-53's `Button`, E2113: "`bump_ui` changes a `Ui`
  through its link `ui` (`fn() changes Ui`), and `on_click` takes `fn() changes captured`, functions
  that change at most the one variable they captured; keep it in a list that says so: `handlers:
  Array[fn() changes Ui]`, with `fn on_click(owns f: fn() changes Ui) changes handlers:`".
- Not refused, warned: a handler that captures the Ui that holds it with `with shares ui` makes a cycle
  of owners that is never freed (a leak, not a memory error, as in Rust); L3001: "the handler is kept by
  `ui` and keeps `ui`; capture it as a link that does not keep it alive, `with weakref ui`, or take the
  Ui as the handler's parameter".

**7. Cost.** A call copies two words out of the list (plain C copies the same `{fn, env}` pair). A
handler's body holds one count on its own state only across a call that may remove it and only when it
uses its state after it (a rare path). The phase before measured this dispatch at 1.045 and 1.037 of
plain C doing exactly the same work with the same allocator, safe Rust at 1.490 and 1.472
(`target/redesign2/graphs-callbacks-attack2/bench/runs/w4_a.txt`, `w4_b.txt`;
`phase3b-graphs-callbacks-attack.md` 6.4). Run 3's W4, this program's workload: 1.091x of plain C with a
captured variable in the handler's own state block (R12's placement here), 1.255x with a box per
captured variable, safe Rust 1.358x (`RUN3/26-timer.md` lines 29, 31, 32). The `weakref` capture: one
add of the Ui's weak count when `bump_ui` is made, one subtract when it ends, and one header test per
run (R11).

**8. Kind:** a smarter compile-time solution (where each count is needed is decided when compiling; which
handlers may change a Ui is read from their type; the `for` refusal is the agreed limitation).

---

## AN-61: Unsound today: a handler list dispatched by index is accepted and writes freed memory

**v3:** as in v2; verdicts 21 to 34 change nothing here (the handlers are AN-60's `fn(ref mut Ui)`).
The E3060 message in part 6 is v2's, and `V2/readability.md` F-01 is settled by verdict 42: its line,
`h = ref mut this.handlers.get_mut(i).unwrap()`, gives `h` another ref, which now compiles. Meaning
unchanged.

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
    fn fire() changes this:
        i = 0
        while i < len(this.handlers):
            this.handlers[i](ref mut this)
            i += 1
```

and (v2's change from run 4) the OLD's own shape, accepted once the name says `ref mut`:

```ember
    fn fire() changes this:
        i = 0
        while i < len(this.handlers):
            h = ref mut this.handlers.get_mut(i).unwrap()   # the only name for this handler while used
            h(ref mut this)                                 # the call first copies its two words out
            i += 1
```

Changed from run 4: `changes this` (was `ref mut this` after the brackets, R5); the second form is now
accepted (run 4 refused it, E3022). A call through a `ref mut` name of a stored function value uses the
name only to copy the function's two words out, before the arguments are given, so `ref mut this`, which
reaches `h`'s place, may be given to it (R8 (f), R12; the `get_mut` row of `V3/vocabulary.md`). Nothing is
written back into the list after the call (the handler's state is behind the function value's own
pointer, R12), so a handler may grow or shrink the list while it runs. `h` is a position (R8 (a)); it is
not used after the call, and a use there would be checked against what the call may have done (part 6).

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `this` | `this`: the Ui `fire` was called on | yes: `changes this` |
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

**v3:** NEW as in v2: (b) is the walk the owner ruled in verdict 28, line for line (`up = weakref
leaf`, `while up exists as n:`, `if n.parent exists as p:`, `up = weakref p`, `else: break`), and (a)
is the plain walk verdict 28 keeps for a visit function that changes nothing (`fn(Node)`). What changed
from v2: (b)'s view walk is refused with R11's message (E3021), which gives the link walk line by line;
(b)'s cost follows v3's R11: since `Node` has no `drop` method, the walk runs in a deferred-drop
region, and its link holds no weak count after its first target, so each step pays one header test
(the weak-count moves of v2 go); a C check of that saving under ASan was run in this step (part 5; not
timed); a refusal added for a function stored in the tree it changes (E3022), the hole v2's note 1
found, now in R12. v2's question for the owner is answered by verdict 28 ("the cost is justified however
we will still try to optimise it if possible"). Meaning unchanged.

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

**3. NEW.** A callback's effects are no longer unknown: its type says them (R12, verdict 17). The usual
callback only looks at the nodes (a), and a callback that may change or remove nodes says so (b). In both,
a parent link is an ordinary `weakref` field the program sets (verdict 14).

(a) The callback changes no node:

```ember
class Node:
    value: int = 0
    parent: weakref Node = None
    children: Array[Node] = []

    fn add(owns child: Node) changes children:
        child.parent = weakref this            # the link back, which the program sets
        this.children.push(child)              # then the child is put where its parent owns it

class Tree:
    root: Node
    leaves: Array[weakref Node] = []
    on_visit: fn(Node)                         # changes nothing outside itself

    fn depth_total() changes on_visit -> int:
        depth = 0
        for leaf in this.leaves:
            if leaf exists as n:               # the leaf still lives: call it `n`
                cur = ref n                    # a view of that node
                while cur.parent exists as p:  # its parent, while there is one
                    this.on_visit(p)           # changes no node: its type is `fn(Node)`
                    depth += 1
                    cur = ref p                # `cur` now views the parent
        return depth
```

(b) The callback may change nodes, and so remove and end them (`Node` as in (a)):

```ember
class Tree:
    root: Node
    leaves: Array[weakref Node] = []
    on_visit: fn(Node) changes Node            # may change nodes, remove them included

    fn depth_total() changes on_visit, Node -> int:
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
number in its header no name reads, so it is not a change of `this` and `add`'s line names only
`children` (R11). `depth_total` calls the stored function value, a change of the field that holds it
(`changes on_visit`, R12), and in (b) names what that function changes (`changes Node`, R5).

What (a) costs, by the rules: `on_visit`'s type says it changes no node, so no line in the walk can end a
`Node` (R10 point 1): the views `n`, `cur` and `p` need no protection (no region, no count pair). The
parent read `cur.parent` needs no header test (R11): (1) the link follows ownership in this package (the
only line that gives `parent` a target is in `add`, followed by the `push` into the parent's own list;
nothing takes a node out of its list; no `children` list is handed over or replaced whole; nodes have one
owner and `Node` has no `drop` method; the fields are not `pub`), and (2) nothing in the walk can end a
node. So each step is a plain load of `cur.parent` and a test for no link. The one header test is
`if leaf exists as n:`, once per leaf (`leaves` holds ordinary links, not parent links). `cur = ref n` and
`cur = ref p` say what `cur` is, a read-only view (R1); `=` on `cur` gives it a new view (R2).

What (b) costs, and why it is written with a link: across a call that may change a `Node`, no read-only
view of a node lives (R8 (g), the owner's verdict 5; R10's last paragraph, R12), so (a)'s loop with (b)'s
callback type is refused (part 6), and the walk keeps a `weakref` link instead (`up`), which is not a
view and may outlive its target (verdict 28). `n` and `p` are views, but neither lives across the call:
`n`'s last use is `n.parent`, and `p`'s is the call itself, whose argument it is (a view whose last use
is an argument of the call does not live across it, R8 (g); inside the call the parameter makes the
promise). `up = weakref p` makes a link from the read-only `p`, which changes no node (R11). Each step
opens the link (`while up exists as n:`, one header test), reads `n.parent` with no header test (R11:
`n` is known to live at the test, and nothing between the test and the read can end a node), and moves
the link to the parent before the call. If the callback ends `p`, or a node above it with everything
under it, the link's target is ended and its block is kept, so the next `while up exists as n:` finds it
ended and the walk stops: it never visits an ended node, and nothing stops the program (verdicts 15,
28). If the callback moves `p` under another parent (taking it out and giving it a new link, as AN-63's
`take` and `add`), the walk goes on up the new parents.

What keeps the ended node's block until the test, by R11: every `weakref` link holds a weak count on
its target, so `up = weakref p` is one subtract on the node left and one add on its parent; and, since
`Node` has no `drop` method of its own, the compiler places the walk in a deferred-drop region (R10
point 2), which keeps every block it sets aside until the region ends. A link that lives only inside
the region and is given only objects that live at that moment (`up = weakref p`, from the view `p`)
then holds no weak count: the region keeps its target's block. Only the first link of each walk,
`up = weakref leaf` (made from another link, whose target may already be gone), keeps its count. So
each step pays one header test, and nothing else (part 7). This never changes what the program does
(R17): with or without the region the walk stops at the same node.

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `value` | none (default: the node's own `int`) | `changes value`, or `changes this` |
| `parent` | `weakref`: a link that does not keep its node alive | `changes parent` or `changes this` in `Node`; and any owner of the node (`add` sets `child.parent` on the node it was handed) |
| `children` | none (default: the node's own list; each element has this one owner) | `changes children`, or `changes this` |
| `this` in `add` | `this`: the node `add` was called on | `changes children`: that field only (`weakref this` makes a link to it and changes no field) |
| `child` | `owns`: handed over to `add` | its owner may (it sets `child.parent`, then hands it to the list) |
| `root` | none (default: the tree's own Node) | `changes root`, or `changes this` |
| `leaves` | `weakref` on its elements: links that do not keep the leaves alive | `changes leaves`, or `changes this` |
| `on_visit` in (a) | none (default: the tree's own function value); its type `fn(Node)` changes nothing outside itself | `changes on_visit`, or `changes this` |
| `on_visit` in (b) | none (default: the tree's own function value); its type `fn(Node) changes Node` may change nodes | `changes on_visit`, or `changes this` |
| `this` in `depth_total` | `this`: the Tree `depth_total` was called on | `changes on_visit` (calling the stored function value may change it, R12); in (b) also `changes Node` |
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
a freed block, so the weak count is what makes the link's test safe (or, in v3, the region, below).

The deferred-drop region's code, from run 3's bench (`c/bin/walk_noprot.s` against `c/bin/walk_defer.s`,
the whole difference apart from the program's name string and the counter's declaration):

```text
205a206
> 	incq	g_defer(%rip)
233a235
> 	decq	g_defer(%rip)
```

one increment before each pass's loop and one decrement after it; nothing inside the loop. (a) needs no
region (no call in it can end a node). In v3 a region serves (b) in a second way (R11): it keeps the
block of every node set aside during the walk until the walk ends, so the walk's link needs no weak
count after its first target.

(b) in that region, checked in this step: `target/redesign4/v3-05-area-D/c/an62b_region.c`, the same
tree and visit function as v2's part 2 above (`root`, `top`, `mid`, `leaf`; the visit function, on
`mid`, takes `top` out of `root` and ends it, with `mid` and `leaf` under it). Mode 1 is v3's R11: a
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
- (a)'s loop (views `cur` and `p` kept across the call) with (b)'s callback type, the plain-`ref` walk
  verdict 28 refuses, E3021: "`this.on_visit(p)` (line 21) may change any `Node`, and so end one
  (`on_visit` is a `fn(Node) changes Node`), and `p` reads a node and is used again on line 23; walk with
  a link, which may outlive its node: instead of `if leaf exists as n:`, `cur = ref n`, the `while` line
  and `cur = ref p`, write `up = weakref leaf`, then `while up exists as n:`, inside it `if n.parent
  exists as p:` with `up = weakref p` and `this.on_visit(p)`, and `else: break`" (R8 (g), R11). Lines 21
  and 23 are those of (a)'s file, `Node` and `Tree` as shown. This is `V3/rules.md` section 3's E3021
  row with one line more in "instead of": the row leaves out `if leaf exists as n:`, which would then
  declare `n` twice (E2106), so the line it gives would not compile in this program (R18; notes for the
  lead, note 4).
- A visit function stored in the tree it changes, with a field that allows a captured variable
  (`on_visit: fn(Node) changes captured`): `t.on_visit = fn(n: Node) changes t: t.root.children.clear()`,
  E3022: "this function changes `t`, and `t.on_visit = ...` keeps it in `t`, so it would change the tree
  that holds it while `depth_total` runs it; clear the children after the walk instead:
  `d = t.depth_total()`, then `t.root.children.clear()`" (R12: a function value is not stored in a
  place that a real variable it changes owns, as Rust refuses; this was v2's note 1, a walk whose views
  of `t`'s nodes would live across a call that ends them). With (b)'s field the same line is refused
  before that, E2113: "this function changes `t`, a variable it captured (`fn(Node) changes t`), and
  `on_visit` holds `fn(Node) changes Node`, functions that change nodes and no variable they captured;
  clear the children after the walk instead: `d = t.depth_total()`, then `t.root.children.clear()`";
  with (a)'s field the message says "`on_visit` holds `fn(Node)`, functions that change nothing" and
  gives the same two lines.
- A function that changes nodes stored in (a)'s field: `t.on_visit = prune` where `prune` is
  `fn(n: Node) with weakref r changes Node:`, E2113: "`prune` changes a `Node` through `with weakref r`
  (`fn(Node) changes Node`), and `on_visit` holds `fn(Node)`, functions that change nothing outside
  themselves; keep it in a field that says so: `on_visit: fn(Node) changes Node`".
- (b)'s field with (a)'s first line, `fn depth_total() changes on_visit -> int:`, E2105:
  "`depth_total` calls `this.on_visit`, whose type says `changes Node`; add it to its first line:
  `fn depth_total() changes on_visit, Node -> int:`" (R5).
- `while Some(p) = a:`, E0115: "an `Option` or a `weakref` link is opened with `exists as`: write
  `while a exists as p:`" (R13).
- Run 3's walk by copies of the link, `a = copy of n.parent`, E2116: "`copy of n.parent` would copy the
  link, not the node; for another link to the same node write `a = weakref n.parent`; for a copy of the
  node itself write `if n.parent exists as q:` and then `c = copy of q`" (R3, R11).
- A view into a `Node`'s contents (an element of `p.children`) kept across a call that may change it:
  refused by the loop rule when a loop walks it (E3020) and by R8 (b), (g) otherwise.
- Not refused: a `Node` class with its own `drop` method. R11's proof then fails (a child's `drop` could
  read its link while its parent is ending), so every `parent` read tests the header; a view of a node
  kept across a call that may end nodes without changing one gets a count pair instead of a region (R10
  point 3); and (b)'s walk gets no region, so its link keeps a weak count on every target, one subtract
  and one add per step (R11).

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
| a deferred-drop region (`walk_defer.exe`; R10 point 2, not needed in (a)) | 1797.9 | 0.944 | 37% |

**PROVISIONAL**: other agents were running; the spread (fastest to slowest) is 29% to 37%, far over the
10% the timer accepts, so these numbers rank nothing finely. Run 3's W2 (the same parent walk without a
callback) measured the read with no header test at 1.176x of plain C, against 1.579x for a header test
on every hop and 1.603x for safe Rust (`RUN3/26-timer.md` lines 20 to 22: `diag_w2_hop_proved`,
`model_w2`, `rust_w2`; `diag_w2_hop_proved` spread 14.8%). The parent links' weak counts are paid where
a link is made or ended (`add`, a node's end), not in the walk; they were not measured.

(b), by v3's R11 (`Node` has no `drop` method): per step, one header test (`while up exists as n:`) and
the parent read with no header test; the link's moves after its first target (`up = weakref p`) touch
no count; per walk from a leaf, one add and one subtract of `leaf`'s weak count (the first link); per
region, one add and one subtract of the per-thread counter (the two instructions above). Where the class
has a `drop` method, add one subtract and one add of weak counts per step (no region). **Not measured**
(this step ran in a cloud container, where no timing was taken; the C check in part 5 tests safety
only). The nearest numbers: a header test on every hop, 1.579x of plain C (`model_w2`, uncounted links,
the same work per step as (b) in a region), and safe Rust's walk, 1.603x (`rust_w2`), which upgrades a
`Weak` on every step (a count pair) (`RUN3/26-timer.md` lines 20 to 22). The owner accepted this cost
(verdict 28) and asked that the next run still look for proof that removes the rest; the region
removes the weak-count moves, and the header test stays because the visit function is a stored
function value, an unknown call, of which only its type is known, and `changes Node` allows it to end
the very node the test is for (notes for the lead, note 2). Peak
memory: 84,544 KB for every variant of run 3's bench against 76,688 KB for plain C (the object header);
a node's header now holds its weak count too (R9). In (b) an ended node's block is kept until the region
ends (or, without one, until the walk's link moves on or ends).

**8. Kind:** (a) a smarter compile-time solution: the callback's type and R11's proof remove every
per-step check, as fast as the unprotected walk; (b) a readable spelling of Rust's guarantee (a link
kept across a call that may end its target, Rust's `Weak` on every step), with the walk stopping at the
first ended parent as verdicts 15 and 28 say, and a smarter compile-time saving (the region) that leaves
one header test per step where Rust pays a count pair.

---

## AN-63: Trees with parent pointers, graphs, cycles (A8)

**v3:** as in v2; verdicts 21 to 34 change nothing here (no function value, kind test or deleting
walk). `add`'s `child.parent = weakref this` under a line that names only `children` is now R11's text
(making a link changes only the target's weak count, so it is not a change of `this`). Meaning
unchanged.

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

    fn add(owns child: Node) changes children:
        child.parent = weakref this            # the link back, which the program sets
        this.children.push(child)              # then the child is put where its parent owns it

    fn take(i: int) changes children -> Node:
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
| `name` | none (default: the node's own String) | `changes name`, or `changes this` |
| `parent` | `weakref`: a link that does not keep its node alive | `changes parent` or `changes this` in `Node`; and any owner of the node (`add` and `take` set it on the node they hold) |
| `children` | none (default: the node's own list; one owner per element) | `changes children`, or `changes this` |
| `this` in `add` | `this`: the node `add` was called on | `changes children`: that field only (`weakref this` makes a link to it and changes no field) |
| `child` in `add` | `owns`: handed over to `add` | its owner may (it sets `child.parent`, then hands it to the list) |
| `this` in `take` | `this` | `changes children` |
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
(R11). A read: where R11 proves it, a plain load and a test for no link; where it cannot, one header test
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

**v3:** as in v2; verdicts 21 to 34 change nothing here. Meaning unchanged.

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
| `name` | none (default: the person's own String) | `changes name`, or `changes this` |
| `friends` | `shares` on its elements: one of several owners of each person, reading it | `changes friends`, or `changes this` (the list); not the persons, which a `shares` owner only reads |
| `best` | `shares`: one of several owners of that person, reading it | `changes best`, or `changes this` (the link); not the person |

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

**v3:** as in v2; the opening `if cam.target exists as p:` is verdict 26's (phase 0 changed the code;
the prose now cites it). Meaning unchanged.

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
| `target` | `weakref`: a link that does not keep the player alive | `changes target`, or `changes this` (re-pointing) |
| `player` | none (default: `main`'s own Player) | its owner may |
| `cam` | none (default: `main`'s own Camera) | its owner may |
| `weakref player` | `weakref`: a link, not a hand-over; `player` stays `main`'s | no |
| `p` | none (default for an opened name: a read-only view of the player, kept alive until its last use) | no |

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
- `cam.target.upgrade()`, E1010: "a `weakref` link is opened with `exists as`: `if cam.target exists as p:`".
- `Weak[Player]`, `Weak(player)`, E1010: "write `weakref Player`", "write `weakref player`".
- `if Some(p) = cam.target:`, E0115: "an `Option` or a `weakref` link is opened with `exists as`: write
  `if cam.target exists as p:`".
- `cam.target.health`, E2020: "`cam.target` may be gone; open it: `if cam.target exists as p:`".
- `cam.target = player`, E2020: "`=` alone would hand `player` over; write `cam.target = weakref player`".
- `a = copy of cam.target`, E2116: "`copy of cam.target` would copy the link, not the player; for another
  link to the same player write `a = weakref cam.target`; for a copy of the player itself write
  `if cam.target exists as p:` and then `c = copy of p`" (verdict 10).

**7. Cost.** One add of the player's weak count when `cam.target = weakref player` makes the link and
one subtract when it ends (R11); one test of the target's header per read (the camera's target is not a
parent link). Nothing keeps `p` alive: no line in its life can end a Player (R10 point 1). Rust's
`upgrade` always makes a count pair.

**8. Kind:** a readable spelling of Rust's guarantee.

---

## AN-66: A bare interface type is a counted, shared, dynamically dispatched handle with no word

**v3:** as in v2; verdicts 21 to 34 change nothing here. The E3040 message's second way out
(`a: shares mut dyn Shape = Circle()` and `b = shares mut a`) is two owners that may both change the
shape, which verdict 23 now states (the first owner, `a`, keeps what it could do). Meaning unchanged.

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
    scene = Scene(shapes=[Circle(r=1.0), Square(side=2.0)], shown=[])
    for s in scene.shapes:
        draw(s)
```

`owns dyn` and `shares dyn` say who owns each shape (verdict 19's words).

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `shapes` | `owns` on its elements: each shape has this one owner | `changes shapes`, or `changes this` |
| `shown` | `shares` on its elements: one of several owners, reading the shape | `changes shown`, or `changes this` (the list); not the shapes |
| `s` in `draw` | none (default: a read-only view, a parameter) | no |
| `scene` | none (default: `main`'s own Scene) | its owner may |
| `Circle(...)`, `Square(...)` in the list | none (default: new values handed over to the list, row 9) | their owner may |
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
- The OLD's `b = a` then `b.grow()` and `a.size()`, with `a: owns dyn Shape`: `b = a` hands the shape
  over (row 9), and the later use of `a` is E3040: "`a` was handed over to `b` on line 2 and is used on
  line 4; to keep both, write `b = copy of a` (a shape of its own, where its type has `Clone`), or make
  the shape shared: `a: shares mut dyn Shape = Circle()` and `b = shares mut a` (two owners that may
  change it)".
- `Box[dyn Shape]`, E1010: "write `owns dyn Shape`".

**7. Cost.** `owns dyn`: one heap block per value and a method-table pointer beside it (Rust's
`Box<dyn>`); `shares dyn`: the same plus a count (Rust's `Rc<dyn>`); a call is one indirect call.

**8. Kind:** a readable spelling of Rust's guarantee.

---

## AN-67: Trait objects and downcasting (A13)

**v3:** NEW as phase 0 left it: the "not sure" downcast is the kind test `if this.pools[k] is Pool[T] as
ref mut pool:` (verdict 27). The prose and the owner's-test row now say what the kind test is (v2's prose
described today's maybe-downcast, which verdict 27 removes), and the refusals give
v3's lines: today's `cast ... if Pool[T]`, the removed `cast ... when Pool[T]`, a `match` on a
maybe-downcast, `as?` and `as!` (all E1010); one refusal added, a plain name from the kind test whose
element is returned `ref mut` (E3023). Meaning unchanged.

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
    fn get_mut(k: u128, i: int) changes pools -> ref mut T:
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
since `get_mut` may change `pools` (`changes pools`); plain `pool` would be read-only (R14; part 6).
`is Pool[T]` says what `pool` is (the pool at `k`, seen as a `Pool[T]`), and `ref mut` that it may change
it. When it is not a `Pool[T]` the `if` block does not run and `get_mut` stops the program with its
message; the "sure" form, `pool = ref mut cast this.pools[k] as Pool[T]`, stops it in the cast instead
(area B's AN-22). A generic method whose type parameter appears only as the type of a `ref` or `ref mut`
parameter, with a `dyn`-compatible bound, gets one method-table entry, the generic body compiled with
`V = dyn Visitor` (R14).

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `pools` | `owns` on its values: each pool has the map as its one owner | `changes pools`, or `changes this` |
| `this` in `get_mut` | `this`: the Reg `get_mut` was called on | `changes pools` |
| `k`, `i` | none (default: read-only numbers) | no |
| `pool` | `is Pool[T]`: the pool at `k`, seen as a `Pool[T]`; `ref mut` on the name the kind test gives: a view of the registry's own pool | yes (`ref mut`, in a method that says `changes pools`) |
| result `-> ref mut T` | `ref mut`: a view into the registry's element | yes |
| `this` in `inspect` | `this` | no: its first line says neither `changes` nor `owns this` |
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
- `cast x as D`, and a kind test `x is D`, between unrelated types, E2020, with "`Box[dyn AnyPool]` is
  now `owns dyn AnyPool`" where that is the cause.
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
  test: instead of `a = s as? Circle`, write `if s is Circle as a:` and use `a` inside it"; `as!`,
  E1010: "`as!` is not Ember: to stop the program when `s` is not a `Circle`, write `a = cast s as
  Circle`" (the vocabulary's `as?` and `as!` rows, in the names of part 2's probe).

**7. Cost.** A `cast`: one load of the method table's type id and one compare; safe Rust's
`downcast_mut` was measured slower (1.710x and 1.606x against the pool table's 1.208x and 1.084x of plain
C, `phase3b-references.md` 3.4 as quoted in `phase3b-graphs-callbacks.md` 6.3); `V3/rules.md` section 2
gives 1.223 to 1.244 of plain C against `downcast_mut` 1.571 to 1.635 (AN-22, PROVISIONAL). The kind
test is the same one compare. The `dyn` entry: the indirect calls Rust's hand-written workaround makes.

**8. Kind:** a smarter compile-time solution.

---

## AN-68: `if c = cast s if Circle:` gets E2036 with a help that is not what was meant

**v3:** NEW as phase 0 left it, the kind test `if s is Circle as c:` (verdict 27). The programmer's
line, `if c = cast s if Circle:`, is now refused because `cast s if Circle` is not Ember (E1010), with a
message that gives the line meant; v2 refused it with E2036. Refusals added: a kind test that always
passes (E2036), `cast s when Circle` (E1010), `match cast s if Circle:` (E1010). Meaning unchanged (the
same program, the same refused line, with another code and the same line to write).

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
- `if s exists as c:`, where `s` always holds a value, E2036: "`s` is a shape, which always holds a value, so
  this `if` would always run; to test whether it is a Circle write `if s is Circle as c:`; to
  name it write `c = s` on the line before".
- A kind test that always passes, where `s` is already a `Circle` (`for s in circles:`), E2036: "`s` is a
  `Circle` already, so this `if` would always run; to name it write `c = s` on the line before" (R14).
- `if s is Circle as s:`, E2106: "`s` already names a value (line 1); `as` declares a new name: `if s
  is Circle as c:`" (R14).

**7. Cost.** None beyond the one type-id compare of the kind test (AN-67).

**8. Kind:** a readable spelling of Rust's guarantee (`if let Some(c) = s.downcast_ref::<Circle>()`),
written as Swift, Kotlin and C# write it, with today's unhelpful help replaced by the line meant.

---

## AN-69: Generics friction (A16)

**v3:** as in v2; verdicts 21 to 34 change nothing here. The E2041 message in part 6 gives both ways out
as lines (phase 3: "narrow the general one's bound" could not be written, Ember having no bound that
leaves one type out). Meaning unchanged.

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
- A `pub` generic with an operation its bounds lack, E2040: "`total` is `pub`, so its bound is written:
  write `[T: Add[T, Output = T] + Copy]` above `pub fn total(xs: Span[T]) -> T:`" (a non-`pub` function never
  gets it).
- E2121 in a library package: "`Array[Vec3]` and `Neg` both belong to other packages, and packages that
  depend on this one could see a second `Neg` for `Array[Vec3]`; wrap it: `struct Vecs(Array[Vec3])`
  and `extend Vecs implements Neg:`".
- E2041, overlapping implementations (the most specific one winning is unproven for Ember): "`i32`
  already gets `Describe` from `extend[T: Debug] T implements Describe` on line 5, and two
  implementations may not apply to one type; give `i32` its own description through a function of its
  own, `fn describe_int(x: i32) -> String:`, or remove line 9 so that `i32` uses the general one".
- A non-`pub` body calling a method no interface in scope declares, E2072: "`T` has no `weight`: no
  interface in scope declares `fn weight`; declare one (`interface Weighed: fn weight() -> int`) and the
  bound follows" (a duck-typing extension is not taken: it would let a typo surface at a call far away).

**7. Cost.** None at run time: every instance compiles to the code written bounds give.

**8. Kind:** a smarter compile-time solution (bounds and the orphan exemption); the overlap is an
understandable limitation kept, with its message.

---

## AN-70: `virtual` in a class that cannot be subclassed is accepted with a warning (W2111)

**v3:** as in v2; verdicts 21 to 34 change nothing here. Meaning unchanged.

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
    virtual fn on_update(dt: float) changes this:
        pass

class SwingDoor(Door):
    override fn on_update(dt: float) changes angle:
        this.angle = min(this.angle + 90.0 * dt, 90.0)
```

Changed from run 4: the virtual says `changes this` (was `ref mut this` after the brackets, R5, verdict
6). Its first line is what every override must keep to, and it may say more than its own body does
(R5: a virtual's body is checked one way, so `pass` under `changes this` is accepted; this settles run 4's
note 5). The override says `changes angle`, the one field its body changes: no more than the virtual's
line allows (E2113), and exactly what its body does (E2105). The override's body no longer calls `min`,
which has no row as a function of two numbers (see "Words needed"); it does the same: the angle grows and
stops at 90.

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `angle` | none (default: the door's own `float`) | `changes angle`, or `changes this` |
| `this` in `Door.on_update` | `this`: the door the method was called on | `changes this`: any field (the promise for its overrides) |
| `this` in `SwingDoor.on_update` | `this` | `changes angle`: that field only |
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
- An override that changes more than the virtual's first line says (`virtual fn on_update(dt: float):`
  in `Door`), E2113: "`SwingDoor.on_update` changes `angle`, but it overrides `Door.on_update`, whose
  first line changes nothing, and callers of `Door` rely on that; write
  `virtual fn on_update(dt: float) changes this:` in `Door`".

**7. Cost.** None.

**8. Kind:** an understandable limitation kept (only an `open` class has subclasses), now refused with the
line to write instead of accepted with a warning.

---

## Words needed

Words an answer would need that `V3/vocabulary.md` has no row for. None is used in the NEW code above.

| word | why |
|---|---|
| `min` as a function of two values (`min(a, b)`) | AN-70's override (run 3's and run 4's) wrote `this.angle = min(this.angle + 90.0 * dt, 90.0)`. `V3/vocabulary.md` has `min` only in the iterator row, as a method (`xs.iter().min()`), and its prelude row lists `len`, `println`, `panic`, `sorted`, `abs`. The NEW body writes the same with `+=` and an `if`; a row for the prelude function would let the line stay as the programmer wrote it. **Settled by the lead: `min`, `max` and `clamp` are prelude functions of two values in today's spec (`[STD-26]`, spec 1538 and 4157); the vocabulary's prelude row now lists them, and the NEW body writes `min` again.** |
| `changes` on a lambda written with `=>` (`fn(owns s) changes names => names.push(s)`) | AN-56's first lambda, and AN-55's, were written with `=>` in v2. Under verdict 22 they say what they change, and the vocabulary shows `changes` only on a lambda written with `:` (`fn(s) changes LOG: LOG.push(s.name)`). This file writes both with `:`, which says the same; a line in the `fn` row would say whether `=>` takes the clause too (notes, note 3) **Settled by verdict 45: yes, after the parameters.** |

## For the owner

1. **Answered by verdict 40: yes, a name visible where the list is written is written by its name
   (`hs: Array[fn() changes score] = []`, as AN-58 now writes it).** The question as it was put: **may a
   list of functions that a function keeps for itself name the variable they change?** (AN-58,
   verdict 22.) A field's or a parameter's type cannot name a variable of whoever calls it, which is why
   you chose `captured`. A local list, kept and run inside the one function that wrote the lambdas,
   could name that function's own variable. Meanwhile AN-58 writes the rule as it stands, which allows
   only `captured` in a type:

   ```ember
   hs: Array[fn() changes captured] = []      # meanwhile: the word for "the variable each captured"
   hs.push(fn() changes score: score += 1)
   ```

   The other spelling, naming it:

   ```ember
   hs: Array[fn() changes score] = []         # the list says which variable: only this function's `score`
   hs.push(fn() changes score: score += 1)
   ```

   The second says more (a reader sees which variable), and costs a second rule: a type may name a
   variable only where that variable is in scope, and such a list may not leave the function (E3026
   already refuses that). Both are checked when compiling and cost nothing while running. Which?

The questions this area's answers meet that the rule writer already raised are in `V3/changes.md`
section 11 and are not repeated: question 1, what `click` says about its handlers' captured variables
(answered by verdict 35: the word their type uses, `fn click() changes handlers, captured:`, as AN-53 now
writes it); question 4, a function value or a bound in a
function that is handed a function (answered by verdict 38: the library uses the bound, a program's own
function may write either; AN-56 writes verdict 21's function-value form, AN-55 uses the library's
bound); question 7, timing AN-62 (b)'s region (this step checked it for safety only, part 5).

## Notes for the lead and the rule writer

Points where the v3 rules did not settle what an answer needed, or where an answer found a gap; what
this area wrote meanwhile.

1. **v2's six notes, and where they went** (`V3/changes.md` section 5). Note 1 (a function value stored
   inside the variable it changes) is now refused (E3022, R12): AN-53 and AN-62 part 6 show it. Note 2
   (R11's region walk against R8 (g)) is settled by verdict 28: AN-62 (b) is its walk. Note 3 (a view as
   the argument of a call that may change it) is R8 (g)'s sentence on a view whose last use is an
   argument. Note 4 (a link inside a region needs no weak count) is R11's saving: AN-62's part 5 checks
   it in C and part 7 uses it. Note 6 (`weakref this` in a method that names only a field) is R11's
   sentence on making a link. **Note 5 is still open**: R11's parent-link proof is package-wide, and one
   line elsewhere (a link given to a node its target does not then own, a `remove` with no clearing)
   makes every `parent` read in the package test the header, 1.579x of plain C instead of 1.176x on W2
   (`RUN3/26-timer.md` lines 20 to 22), with no word in the program and no message. A note from the
   compiler naming the line that stops the proof, or an `ember inspect` view of it, would let the
   programmer see it; `V3/changes.md` section 5 leaves it to the lead.
2. **The header test AN-62 (b) still pays per step** (verdict 28 asks the next run to look for proof
   that removes what it can). The region removes the weak-count moves; no v3 rule can remove the header
   test, because `on_visit` is a stored function value, an unknown call known only by its type, and
   `fn(Node) changes Node` allows it to end the node the next test is for. A proof would need to know
   which nodes the visit function may end. Two routes for the next run, neither in the rules: a function
   type that says more than `changes Node` (for example that it never ends a node above the one it is
   given), which would be a word for the owner; or, where every line that sets `on_visit` is in the
   package, the summary of each function given to it (R17), as R11 already reads summaries for parent
   links. Neither was priced.
3. **`changes` on a `=>` lambda, and a message in `V3/rules.md` that breaks verdict 22.** The vocabulary
   shows a `changes` clause only on a lambda written with `:`; this file writes every changing lambda
   that way (AN-55, AN-56; "Words needed"). `V3/rules.md` section 3's E2020 row gives "to keep your own,
   write `fn(s) => names.push(copy of s)`", a lambda that changes the captured `names` without saying so,
   which R12 refuses (E2105); AN-56's own message gives `fn(s) changes names: names.push(copy of s)`.
   Area B found the same row (`V3/area-B.md` section 6, note 6).
4. **`V3/rules.md` section 3's E3021 row for the view walk gives a line that does not compile in its own
   program** (R18). The row says "instead of `cur = ref n`, the `while` line and `cur = ref p`, write
   `up = weakref leaf`, then `while up exists as n:` ...". AN-62 (a)'s walk, which the row's line numbers
   (21, 23) come from, also has `if leaf exists as n:` above `cur = ref n`; kept, it and the new `while up
   exists as n:` declare `n` twice (E2106). AN-62 part 6 adds `if leaf exists as n:` to the "instead of"
   list; the row needs the same.
5. **Two readings of R12 this file takes.** (a) R12 says a function that runs a function whose type
   says `captured` writes "`captured` where it runs the one function a field holds"; AN-54's E2113
   message applies that to a parameter, `fn run(ref mut f: fn() changes captured) changes captured:`.
   (b) R12 shows `captured...` beside a module variable (`Array[fn() changes captured..., LOG]`); AN-53's
   E2113 message writes `captured` beside one, `Array[fn() changes captured, LOG]`, so that the button
   that keeps row 6's handlers can keep `LOG` handlers too. One sentence in R12 would say each.
6. **Messages in area D that still break the message rule** (left as v2 wrote them: the lead's, phase 3,
   except where this step rewrote a message, which is said in the answer):
   - AN-61 part 6, E3060: "name it again after the call: `h = ref mut this.handlers.get_mut(i).unwrap()`"
     gives a new view to a `ref mut` name, which R2 refuses (E3023); the line that compiles uses a new
     name (`V2/readability.md` F-01).
   - AN-53 part 6 and AN-57 part 3, E3026: the help gives `count = shares mut Counter()` and `count.n +=
     1`, which need a class `Counter` with a field `n` that the program does not have and the message
     does not give; AN-53's bullet only describes the list ("kept in a list of `fn() changes Counter`")
     (F-03). AN-57's new E3023 message (verdict 32) is about that help followed, so its `Counter` is the
     program's own.
   - AN-53 and AN-58 part 6, E3020: "or collect what to add and push it after the loop" is a way out in
     words (F-06).
   - AN-67 part 6, E2050: "or use a type parameter `[I: Inspect]` instead of `dyn Inspect`" gives a
     bound, not the line (`[I: Inspect]` above `fn walk(xs: Array[I]) -> int:`) (found in this step).
   - AN-69 part 6, E2041: "narrow the general one's bound so it leaves `i32` out" is a way out in words;
     its second way out is an exact line (found in this step).
   **Phase 3 (the lead):** all of these are settled. AN-61's line compiles under verdict 42 (a `ref mut`
   name may be given another ref) and stays; AN-53's and AN-57's E3026, AN-53's and AN-58's E3020, AN-67's
   E2050 and AN-69's E2041 now give the exact lines; the rules' E3021 walk row (note 5's AN-62 line) lists
   `if leaf exists as n:` among the lines it replaces. Each fix is in `V3/phase3-messages.md`.
7. **Verdicts with no code in area D.** No answer of area D calls a C function, so verdict 31 (a C call
   may run the callbacks C keeps; `@ffi(no_callbacks)`) changes none of its lines; area E's AN-78 carries
   it. Verdict 32 lands only in AN-57's added refusal; no NEW block of area D takes a plain parameter of a
   `shares` type. Verdicts 23, 24, 29, 33 and 34 have no construct in area D's NEW code (AN-66's message
   names two `shares mut` owners, which verdict 23 confirms).
