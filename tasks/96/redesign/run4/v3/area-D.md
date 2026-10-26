# Area D: linked objects, callbacks, interface types (run 4, step 1, v2; agent 05)

Every answer of area D (AN-53 to AN-70) brought to the v2 rules (`V2/rules.md`) and words
(`V2/vocabulary.md`), which carry the owner's 20 decisions of 2026-10-25 (`inputs/JOB.md`). `RUN3` means
`tasks/96/redesign/run3/`, `RUN4` means `tasks/96/redesign/run4/`, `V2` means
`tasks/96/redesign/run4/v2/`. "R1" .. "R18" are the rules of `V2/rules.md`; "row N" is the owner's agreed
row N and "decision N" the owner's decision N in `inputs/JOB.md` ("Owner verdicts on run 4's open
questions"); G-01 .. G-11 are the problems of `RUN3/22-re-attacker.md`.

Power: `PowerLineStatus` printed `Online` at the start of this work, before the `ember check` run and the C
run, and before writing this file. No compiler change, no git command, no Ember program run. Scratch
files: `C:\Users\ism19\Code\ember\target\redesign4\v2-05-area-D\` (`template.md` and `build.py`, which
copy every OLD part, and every pasted output that carries over, from `RUN4/area-D.md` unchanged;
`m55_inspect_count.em` and `check_output.txt`, the one `ember check`; `c\an62_an63_v2.c` and
`c\an62_an63_v2_output.txt`, the C of AN-62 (b) and AN-63 under AddressSanitizer).

## Terms used in this area

- **function value**: a function or lambda kept in a variable, a list or a field (a callback, a
  handler): two words, its code and its **state** (what it captured with a word). Its type says what it
  may change outside itself (R12): `fn()` nothing, `fn() changes LOG` the module variable `LOG`,
  `fn() changes Ui` objects of type `Ui` it reaches through a link it holds.
- **capture**: a name a lambda uses from the function that made it. **The real variable** (row 6): that
  very variable, not a copy; a write in the lambda is seen by the function, and back. A captured real
  variable is part of the function value itself (R12).
- **capture list**: what follows `with` after a lambda's parameters (`fn() with copy of score:`).
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
- **AddressSanitizer** (ASan): a clang option that stops a program with a report the moment it reads or
  writes memory it must not.
- **today**: `ember check <file>` with today's compiler (`target/redesign3/bin/ember.exe` in run 3,
  `target/redesign4/bin/ember.exe` in run 4 and here). Every claim about today has its output pasted.

The OLD code of every answer is unchanged, so every OLD part, with its `ember check` output, is copied
unchanged from `RUN4/area-D.md` (file names in it are relative to `target/redesign3/11-area-D/`).

## Answers whose meaning changed from run 4

| answer | what changed, and how |
|---|---|
| AN-53 | run 4's stated limitation (G-07: inside `click` every `h()` was taken to make every change any `fn()` of the program makes) is gone: a plain `fn()` changes nothing outside itself (decision 17), so a handler that changes a module variable says so (`fn() changes LOG`) and is kept only in a list that says so (E2113 otherwise). The program and row 6 are unchanged |
| AN-57 | the way out the E3026 message gives now names what the handler changes: `fn() with shares mut count changes Counter:`, kept in a list of `fn() changes Counter` (decision 17); the refused program is unchanged |
| AN-60 | the second way out (read-only `fn(Ui)` handlers walked with `for`) is now accepted for every program, by the handlers' type: a `fn(Ui)` changes no `Ui` (decision 17); run 4's whole-program search is gone. G-05's handler has the type `fn(Ui) changes Ui` and is refused in that list (E2113). `watch`'s handler changes a `Ui` through its `weakref` capture, so it says `changes Ui` and is kept in a button whose list says so |
| AN-61 | the programmer's own shape, a handler named by `get_mut` and then called with the Ui, is now accepted when the name says `ref mut` (`h = ref mut this.handlers.get_mut(i).unwrap()`, `h(ref mut this)`; R8 (f)); run 4 refused it (E3022) |
| AN-62 | rewritten without `set by` (decision 14): the program sets each parent link itself (`child.parent = weakref this`). With a plain `fn(Node)` callback (which changes no node, decision 17) the walk keeps views, needs no protection and no header test per step (R10 point 1, R11's proof). A callback that may change or end nodes says `changes Node`; then no view of a node may live across the call (decision 5), so the walk keeps a link (`up = weakref p`): one header test and one weak-count subtract and add per step, and it stops at the first ended parent (decision 15). Run 4's question (a node visited once after its parent was removed) is gone with `set by` |
| AN-63 | rewritten without `set by` (decision 14): `add` and `take` set and clear the parent link; reads need no header test where R11 proves the link follows ownership, and every read tests where it cannot. The costs now include a weak count per link |
| AN-64 | the warning's advice names a `weakref` field, and the parent pair's error names `parent: weakref Node = None` set by the program (no `set by`) |
| AN-68 | the programmer's line `if c = cast s if Circle:` is refused again (decision 1 opens with `exists as`), but with a message that gives the line meant, `if s is Circle as c:`, instead of today's "did you mean `c == ...`" |

Spelling changes that do not change meaning (every answer): `owns` for `owned`, `shares` / `shares mut`
for `shared`, `copy of` for `copied`, `weakref` for `weak`, `if x exists as p:` and `while x exists as p:` for
run 4's `if p = x:` and `while p = x:`, `changes this` for `ref mut this` after the brackets (AN-60, AN-61,
AN-70), and the messages in those spellings. AN-55's iterator line is the programmer's own again
(`.inspect(...).count()`), since `count` now has a row (`V2/vocabulary.md`, the iterator row). AN-70's
override no longer uses `min` (see "Words needed").

---

## AN-53: A closure that changes a variable of the function that made it (A12; agreed row 6)

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
    handlers: Array[fn()] = []

    fn on_click(owns f: fn()) changes handlers:
        this.handlers.push(f)

    fn click() changes handlers:
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
    b.on_click(fn() with copy of score: score += 1)   # row 6's copy: the handler's own `score`
    b.click()
    return score                       # 0: only the handler's copy went up
```

Changed from run 4: `owns f` (was `owned f`) and `with copy of score` (was `with copied score`). The
calls write no `ref mut`: `b.on_click(...)` and `b.click()` are methods called on `b`, whose first lines
say what they change (R4); `f` is an `owns` parameter, and a hand-over is written with no word (R4).
`handlers` holds plain `fn()`: a function that changes nothing outside itself (R12). Row 6's
`fn(): score += 1` is one: `score` is a real variable it captures, which is part of the function itself
and is checked in `game`, the function that writes the lambda (R12; row 6 as agreed). `click` says
`changes handlers` and walks with `ref mut h` because a call through a stored function value may change
its own state, so it is a change of the place that holds it (R12); it names nothing more, since the
handlers' type says they change nothing outside themselves. Where `game`'s real `score` lives (R12): at
its own address, since `b` never leaves `game` and both methods it is handed to say what they change, so
the handler provably ends first. In `game_copy` the captured `score` is named (`copy of`), so it is the
handler's own state.

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `handlers` (field) | none (default: the Button's own list; each element its own function value, of type `fn()`: it changes nothing outside itself) | only in a method whose first line says `changes handlers` (or `changes this`) |
| `this` in `on_click` | `this`: the Button the method was called on | `changes handlers`: that field only |
| `f` | `owns`: handed over to `on_click` | its new owner may; it hands it on to the list |
| `this` in `click` | `this` | `changes handlers` (R12: calling a stored function value may change it) |
| `h` | `ref mut`: an element of `handlers`, the only name for it while used (R8 (f)) | yes; calling it changes what its body changes, its own state included (R12) |
| `score` in `game` | none (default: `game`'s own `int`, made by `=`) | its owner may |
| `b` in `game` | none (default: `game`'s own Button, made by `=`) | through methods whose first line says `changes` |
| `score` in the lambda | none (row 6: the real `score` of `game`) | yes, as row 6 says (`score += 1`) |
| `score`, `b` in `game_copy` | none (default: `game_copy`'s own `int` and Button) | as in `game` |
| `score` in `game_copy`'s lambda | `copy of`: the handler's own copy (row 6) | yes, its own copy, through `click`'s `changes handlers` |

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
  `register` returns; keep the total where the handler can own a share of it:
  `count = shares mut Counter()` and `fn() with shares mut count changes Counter: count.n += 1`, kept in
  a list of `fn() changes Counter`".
- A capture used by a thread that may outlive it (AN-55), E7004.
- `fn click():` with `for ref mut h in this.handlers: h()`, E2105: "`click` calls the handlers stored in
  `this.handlers`, which may change their own state; say so on its first line,
  `fn click() changes handlers:`, and walk them with `for ref mut h in this.handlers:`" (R12).
- A loop that walks a list a function value changes, around a call that may run it (R12, R8 (c)),
  E3020: "this loop walks `xs` (line 5), and `hs[0]()` (line 6) may run the function made on line 3,
  which pushes to `xs`; call it after the loop, or collect what to add and push it after the loop".
- A handler that changes something outside itself says so, and is kept only where the type says so
  (R12, decision 17): `b.on_click(fn() changes LOG: LOG.push(String("clicked")))` with this `Button`,
  E2113: "this function changes `LOG` (`fn() changes LOG`), and `on_click` takes `fn()`, functions that
  change nothing outside themselves; keep it in a list that says so: `handlers: Array[fn() changes LOG]`,
  with `fn on_click(owns f: fn() changes LOG) changes handlers:`". With such a list, `click` says
  `changes handlers, LOG` (E2105 otherwise), and `b.click()` inside a loop over `LOG` is refused, E3020:
  "this loop walks `LOG` (line 16), and `b.click()` (line 17) changes `LOG`: `click` runs the handlers of
  `b`, an `Array[fn() changes LOG]` (its first line says `changes handlers, LOG`); click after the loop".
  A button whose handlers are plain `fn()`, as here, is clicked inside such a loop with no refusal
  (G-07 settled by the type, not by a search of the program).

**7. Cost.** At an address: none (the C passes `&score`). In the function value's own state
(`game_copy`): nothing beyond the function value's own block. In a box: one allocation per variable
and one load per use. Run 3's W4 (callbacks that remove themselves) measured the state placement at
1.091x of plain C, a box per captured variable at 1.255x, safe Rust at 1.358x (`RUN3/26-timer.md` lines
29, 31, 32: `rust_w4`, `model_w4`, `model_w4_state_block`); the phase before measured the callback
workload at 1.018 and 1.052 of plain C with the shared total boxed, against 1.045 and 1.037 with it by
address (`target/redesign2/graphs-callbacks-attack2/bench/runs/w4_a.txt`, `w4_b.txt`, summarised in
`phase3b-graphs-callbacks-attack.md` 6.4). Rust's spelling of the same program is `Rc<Cell<i32>>`, the
same box.

**8. Kind:** a smarter compile-time solution (Rust refuses the program; here the compiler proves the
cheaper place and otherwise uses the box Rust makes the programmer write).

---

## AN-54: `owned fn` moves a `String` capture and copies an `int` one (agreed row 6)

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

Changed from run 4: `owns` and `copy of` (were `owned` and `copied`). The call says `run(ref mut cb)`, as
`run`'s first line says `ref mut f` (R4). `cb` is a plain `fn()`: it changes only its own state (R12).

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
- `run(cb)` with `fn run(f: fn()):`, E3023: "`cb` changes its own copy of `score` each time it runs, and
  `run` takes it read-only; write `fn run(ref mut f: fn()):` and call `run(ref mut cb)`" (R12).
- `run(cb)` with `fn run(ref mut f: fn()):`, E3028: "`run` may change `cb` (its first line says
  `ref mut f: fn()`); say so at the call: `run(ref mut cb)`" (R4).

**7. Cost.** None beyond C: the `String`'s three words move into the function's state, the `int` is
copied, as a Rust `move` closure.

**8. Kind:** a readable spelling of Rust's guarantee (`move`, written per name).

---

## AN-55: The compiler decides a capture is moved from which function receives the lambda (`[CLO-15]`)

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
    _n = xs.iter().inspect(fn(v) => seen.push(v)).count()   # the real `seen`
    h = thread.spawn(fn() with owns xs: work(xs))            # `xs` handed to the thread
    println(seen.len())
    println(h.join())
```

Changed from run 4: the iterator line is the programmer's own again. Run 4 wrote it with `for_each`
because `count` had no row; `V2/vocabulary.md` now has `count` in the iterator row. `inspect` takes a
plain `fn(int)`, and the lambda changes only `seen`, a real variable it captures, so it is such a
function (R12; `V2/changes.md` section 2.2). The thread's line says `with owns xs` (was `with owned
xs`). The thread is not a call of `main`'s (R5), and what it holds is in its capture list (R15).

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `xs` in `main` | none (default: `main`'s own list) | its owner may |
| `seen` in `main` | none (default: `main`'s own list) | its owner may |
| `v` | none (default for a lambda parameter: it reads; an `int`, copied in) | no |
| `seen` in the lambda | none (row 6: the real `seen`) | yes (`push`), as row 6 says |
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
iterator line as written, `thread.spawn(owned fn() => work(xs))`), checked in this step
(`target/redesign4/v2-05-area-D/check_output.txt`):

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

**7. Cost.** None beyond C: the lambda given to `inspect` holds `seen`'s address, the hand-over moves
three words.

**8. Kind:** a readable spelling of Rust's guarantee (`move` and scoped threads), with no rule that
depends on the callee.

---

## AN-56: A lambda parameter without a mode takes `owned` from the expected callable type (`[TYP-23]`)

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
    each_name(fn(owns s) => names.push(s))       # takes `s` over, keeps it
    each_name(fn(s) => println(s.len()))          # only reads `s`
```

Changed from run 4: `owns` (was `owned`). `f(String("ann"))` and `each_name(...)` hand their arguments
over to `owns` parameters, written with no word (R4). Both lambdas are plain function types: the first
changes only `names`, a real variable it captures (R12).

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `f` | `owns`: handed over to `each_name` | its new owner may |
| `names` | none (default: `main`'s own list) | its owner may |
| `s` in the first lambda | `owns`: handed over to the lambda | its owner may (it hands it on to `names`) |
| `names` in the first lambda | none (row 6: the real `names`) | yes (`push`), as row 6 says |
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
- `each_name(fn(s) => names.push(s))`, E3013: "`s` is only read here, so it cannot be kept in `names`;
  to take it over write `fn(owns s) => names.push(s)`".
- `fn(owns s)` where the expected type is `fn(String)` (a read-only parameter), E2020 (a lambda
  parameter whose mode differs from the expected function type): "`each_name` gives each name read-only,
  so this function cannot take it over; to keep your own, write `fn(s) => names.push(copy of s)`".

**7. Cost.** None: the same code as today.

**8. Kind:** a readable spelling of Rust's guarantee (the parameter's mode is written, as on every other
parameter, R4).

---

## AN-57: E3026 is reported inside the callee; a stored lambda gets E2020 naming `Callable0`

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
    b.on_click(fn(): total += 1)
```

```text
error[E3026]: the click handler changes `total`, which belongs to `register`'s caller
 --> app.em:2:16
  |
2 |     b.on_click(fn(): total += 1)
  |                ^^^^^^^^^^^^^^^^ written here, kept in `b`
  |
  = why: `b` keeps the handler after `register` returns, and `total` may have ended by then
  = help: keep the total where the handler can own a share of it:
          `count = shares mut Counter()` in the caller, and
          `b.on_click(fn() with shares mut count changes Counter: count.n += 1)`,
          kept in a list of `fn() changes Counter`: `handlers: Array[fn() changes Counter]` in `Button`
```

Callable types are printed as written: `fn()`, `fn(ref mut Ui)`, `fn() changes Counter`, never
`Callable0` (R18). Changed from run 4: the way out. The handler in the help changes a `Counter` through a
link it holds (`with shares mut count`), which is outside itself, so its type says `changes Counter`
(R12, decision 17), and AN-53's `Button`, whose list holds plain `fn()`, could not keep it (E2113); the
help names both lines (R18). The message is `V2/rules.md` section 3's E3026 row.

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `b` | `ref mut`: the caller's Button | yes (`ref mut`) |
| `total` | `ref mut`: the caller's `int` | yes (`ref mut`) |
| `total` in the lambda | none (row 6: the real `total`, here the caller's place) | yes, as row 6 says; refused because the handler may outlive it |

**5. Means exactly.** Nothing compiles for the refused program, which is the point. The accepted form in
the help is AN-53's box (`means/m53_button_box.em`, `exit=0`, pasted under AN-53); its
`shares mut Counter` is today's class `Score` there.

**6. Still refused.** The program above, with the message above (E3026). A lambda given to a callee that
keeps it while the lambda captures a view (a `ref`, `ref mut`, pattern, opened or `for` name): the same
message shape, at the lambda's line, naming the captured view and the holder that keeps the lambda.

**7. Cost.** None (compile time only).

**8. Kind:** an understandable limitation kept (a function value may not outlive a place it does not
own; Rust's E0373), with the message at the right line and in the program's names.

---

## AN-58: A closure stored in a local list that ends first is still E3026

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

**3. NEW.** (Unchanged from run 4.)

```ember
fn game() -> int:
    score = 0
    hs: Array[fn()] = []
    hs.push(fn(): score += 1)
    for ref mut h in hs:
        h()
    return score                       # 1
```

A call through a stored function value may change its own state, so its holder is reached through a
name that may change it (R12): the walk is `for ref mut h in hs:`, as AN-53's `click`. `hs` holds plain
`fn()`: the lambda changes only `score`, a real variable it captures (R12).

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `score` | none (default: `game`'s own `int`) | its owner may |
| `hs` | none (default: `game`'s own list; its elements its own function values, of type `fn()`) | its owner may |
| `score` in the lambda | none (row 6: the real `score`) | yes, as row 6 says |
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
  and `hs[0]()` (line 6) may run the function made on line 3, which pushes to `xs`; call it after the
  loop, or collect what to add and push it after the loop". (The phase before's `attacks2.exe 11` showed
  the use after free this refusal prevents.)

**7. Cost.** None (the C passes an address).

**8. Kind:** a smarter compile-time solution (the list's end is proved; Rust needs the same proof through
lifetimes, and accepts this program).

---

## AN-59: A closure that gives a capture away gets E3013 with ten options instead of "declare the parameter `once fn()`"

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

Changed from run 4: `owns` and `copy of` (were `owned` and `copied`), and `consume` written out (the
OLD's, with `owns`). The arguments of `call_once`, `call_twice` and `consume` are hand-overs to `owns`
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
        if ui exists as ref mut me:                    # the Ui still lives: call it `me`, which may change it
            me.clicks += 1
    b.on_click(bump_ui)
```

Python's list iterator is this loop (a position checked against the length at every step), so a handler
added during the dispatch runs in it and a handler that removes itself makes the next one move into its
place, as in Python.

Changed from run 4: `fn fire() changes this:` (was `ref mut this` after the brackets, R5); the opening
`if ui exists as ref mut me:` (was `if ref mut me = ui:`, R13); `with weakref ui` (was `with weak ui`); `owns
f`. `fire` says `changes this`, not `changes handlers, current`, because it gives the whole Ui to a
handler that may change any field (R4: `ref mut` only before what this function may change, E3027). The
call says `ref mut this` (R4: the handler's parameter is `ref mut Ui`); it copies the handler's two words
out of the list first, so the list is not an argument of the call (R12) and nothing points into it while
the handler runs. The handlers' type `fn(ref mut Ui)` names no change outside the handler: what it
changes, it changes through its `ref mut` parameter (R12).

**Meaning changed** for `watch` (decision 17): `bump_ui` changes a `Ui` through a link it holds
(`with weakref ui`), which is outside itself, so its first line says `changes Ui` (R12; E2105 otherwise),
and it is kept only in a list whose type says so. AN-53's `Button` holds plain `fn()`, so run 4's
`b: Button` would be refused (E2113); `watch` takes a `UiButton`, whose list says `changes Ui`.

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
- **Meaning changed** (decision 17): the second way out, read-only handlers walked with `for`
  (`handlers: Array[fn(Ui)]`, `fn fire() changes handlers:`, `for ref mut h in this.handlers: h(this)`),
  is accepted for every program: a `fn(Ui)` changes nothing outside itself, so no handler in that list
  can change any Ui's `handlers` while the loop walks them. G-05's handler, which captures
  `with weakref ui` and pushes to `me.handlers`, has the type `fn(Ui) changes Ui` (R12); given to that
  list it is refused, E2113: "this handler changes a `Ui` through `with weakref ui` (`fn(Ui) changes
  Ui`), and `handlers` exists as `fn(Ui)`, functions that change nothing outside themselves; give it the Ui
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
  (`fn() changes Ui`), and `on_click` takes `fn()`, functions that change nothing outside themselves;
  keep it in a list that says so: `handlers: Array[fn() changes Ui]`".
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

and, **meaning changed**, the OLD's own shape, accepted once the name says `ref mut`:

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
reaches `h`'s place, may be given to it (R8 (f), R12; the `get_mut` row of `V2/vocabulary.md`). Nothing is
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
  (R8 (a)).

**7. Cost.** As AN-60. The refused form today (a pointer into the list kept across the call and written
back to) had no cheaper code: the copy back it needs is what made it unsafe.

**8. Kind:** a smarter compile-time solution (the hole is closed when compiling; the forms that work cost
what C costs).

---

## AN-62: A loop calling a stored callback with unknown effects pays a count pair per step (the phase before's limit)

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

**3. NEW.** A callback's effects are no longer unknown: its type says them (R12, decision 17). The usual
callback only looks at the nodes (a), and a callback that may change or remove nodes says so (b). In both,
a parent link is an ordinary `weakref` field the program sets (decision 14).

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
            if leaf exists as n:                   # the leaf still lives: call it `n`
                cur = ref n                    # a view of that node
                while cur.parent exists as p:      # its parent, while there is one
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
            while up exists as n:                  # the node still lives: call it `n`
                if n.parent exists as p:           # its parent, if it has one
                    up = weakref p             # the link now names the parent
                    this.on_visit(p)           # may end `p`, or a node above it with all under it
                    depth += 1
                else:
                    break
        return depth
```

**Meaning changed** (decisions 14, 15, 17, 5). Run 4 wrote `parent: weak Node set by children`; `set by`
is gone, and `add` sets the link (`child.parent = weakref this`) before it puts the child where its parent
owns it (`this.children.push(child)`). `depth_total` calls the stored function value, a change of the
field that holds it (`changes on_visit`, R12), and in (b) names what that function changes
(`changes Node`, R5).

What (a) costs, by the rules: `on_visit`'s type says it changes no node, so no line in the walk can end a
`Node` (R10 point 1): the views `n`, `cur` and `p` need no protection (no region, no count pair). The
parent read `cur.parent` needs no header test (R11): (1) the link follows ownership in this package (the
only line that gives `parent` a target is in `add`, followed by the `push` into the parent's own list;
nothing takes a node out of its list; no `children` list is handed over or replaced whole; nodes have one
owner and `Node` has no `drop` method; the fields are not `pub`), and (2) nothing in the walk can end a
node. So each step is a plain load of `cur.parent` and a test for no link. The one header test is
`if leaf exists as n:`, once per leaf (`leaves` holds ordinary links, not parent links). `cur = ref n` and
`cur = ref p` say what `cur` is, a read-only view (R1); `=` on `cur` gives it a new view (R2).

What (b) costs, and why it is written with a link: a call that may change a `Node` may not have a
read-only view of a node living across it (R8 (g), the owner's decision 5; R12), so (a)'s loop with
(b)'s callback type is refused (part 6). The walk keeps a `weakref` link instead (`up`), which is not a
view and may outlive its target: each step opens it (`while up exists as n:`, one header test), reads
`n.parent` with no header test (R11: `n` is known to live at the test, and nothing between the test and
the read can end a node), and moves the link to the parent (`up = weakref p`: one subtract of the old
target's weak count, one add to the new one) before the call. If the callback ends `p`, or a node above
it with everything under it, the link's target is ended (its block kept by the link's weak count), so
the next `while up exists as n:` finds it ended and the walk stops: it never visits an ended node, and
nothing stops the program (decision 15). If the callback moves `p` under another parent (taking it out
and giving it a new link, as AN-63's `take` and `add`), the walk goes on up the new parents.

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
deferred-drop region) and mode 3 (a count pair) are clean. Neither is (b)'s code: under decision 5 a view
of a node does not live across a call that may change nodes, so (b) keeps a link.

(b), as the rules give it, is `c\an62_an63_v2.c` part 2 (this step,
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
a freed block, so the weak count is what makes the link's test safe.

The deferred-drop region's code, from run 3's bench (`c/bin/walk_noprot.s` against `c/bin/walk_defer.s`,
the whole difference apart from the program's name string and the counter's declaration):

```text
205a206
> 	incq	g_defer(%rip)
233a235
> 	decq	g_defer(%rip)
```

one increment before each pass's loop and one decrement after it; nothing inside the loop. Under v2 a
region protects a view of an object reached through a link across a call that may end such an object
(R10 point 2); neither (a) (no such call) nor (b) (no view across the call) needs one.

**6. Still refused.**
- (a)'s loop (views `cur` and `p` kept across the call) with (b)'s callback type, E3021: "`this.on_visit(p)`
  (line 21) may change any `Node` (`on_visit` is a `fn(Node) changes Node`), and `p` reads a node and is
  used again on line 23; keep a link across the call instead of a view: `up = weakref p` before the call,
  and `while up exists as n:` to go on from it" (R8 (g), R12).
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
  read its link while its parent is ending), so every `parent` read tests the header, and a view of a
  node kept across a call that may end nodes gets a count pair instead of a region (R10 point 3).

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

(b): per step, one header test (`while up exists as n:`), one subtract and one add of weak counts
(`up = weakref p`), and the parent read with no header test. **Not measured.** The nearest numbers: a
header test on every hop, 1.579x of plain C (`model_w2`, uncounted links), and safe Rust's walk, 1.603x
(`rust_w2`), which upgrades a `Weak` on every step (a count pair); (b)'s weak-count pair per step is the
same kind of work as Rust's. A compile-time saving is possible and not in the rules (notes for the lead,
4). Peak memory: 84,544 KB for every variant of run 3's bench against 76,688 KB for plain C (the object
header); a node's header now holds its weak count too (R9). In (b) an ended node's block is kept until
the walk's link moves on or ends.

**8. Kind:** (a) a smarter compile-time solution: the callback's type and R11's proof remove every
per-step check, as fast as the unprotected walk; (b) a readable spelling of Rust's guarantee (a link
kept across a call that may end its target, Rust's `Weak` on every step), with the walk stopping at the
first ended parent as decision 15 says.

---

## AN-63: Trees with parent pointers, graphs, cycles (A8)

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
    if n.parent exists as p:                       # `n` has a parent: call it `p`
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

**Meaning changed** (decision 14). Run 4 wrote `parent: weak Node set by children`, and `push` and
`remove` set and cleared the link. `set by` is gone: the link is an ordinary `weakref` field, and the
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
`parent: weakref Node = None`, set with `child.parent = weakref this`. Changed from run 4: `shares` and
`weakref` (were `shared` and `weak`); the parent pair's advice has no `set by` (decision 14).

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
    if cam.target exists as p:              # the player still lives: call it `p`
        println(p.health)
    else:
        println("gone")
```

Changed from run 4: `weakref` (was `weak`) and the opening `if cam.target exists as p:` (was
`if p = cam.target:`), R11, R13 (decisions 1, 9).

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
  `if cam.target exists as p:` and then `c = copy of p`" (decision 10).

**7. Cost.** One add of the player's weak count when `cam.target = weakref player` makes the link and
one subtract when it ends (R11); one test of the target's header per read (the camera's target is not a
parent link). Nothing keeps `p` alive: no line in its life can end a Player (R10 point 1). Rust's
`upgrade` always makes a count pair.

**8. Kind:** a readable spelling of Rust's guarantee.

---

## AN-66: A bare interface type is a counted, shared, dynamically dispatched handle with no word

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

Changed from run 4: `owns dyn` and `shares dyn` (were `owned dyn` and `shared dyn`).

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

    fn get_mut[T](k: u128, i: int) changes pools -> ref mut T:
        if this.pools[k] is Pool[T] as ref mut pool:   # the pool at `k` is a `Pool[T]`: call it `pool`
            return ref mut pool.dense[i]
        panic("no pool of this type at this key")

interface Inspect:
    fn inspect[V: Visitor](ref mut v: V)

fn walk(xs: Array[owns dyn Inspect]) -> int:
    s = Sum(total=0)
    for x in xs:
        x.inspect(ref mut s)        # through `dyn`: the entry made with V = dyn Visitor
    return s.total
```

Changed from run 4: `owns dyn` (was `owned dyn`) and the opening
`if this.pools[k] is Pool[T] as ref mut pool:` (was `if ref mut pool = cast ... if Pool[T]:`),
R13, R14 (decision 1).

`cast <place> if D` on an `owns dyn` (or `shares dyn`) place tests the value's type and gives a view of
it as a `D`, opened with `exists as`: `ref mut pool` names a changeable view because the place is changeable
(`changes pools`); plain `pool` would be read-only (R14). A generic method whose type parameter appears
only as the type of a `ref` or `ref mut` parameter, with a `dyn`-compatible bound, gets one method-table
entry, the generic body compiled with `V = dyn Visitor` (R14).

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `pools` | `owns` on its values: each pool has the map as its one owner | `changes pools`, or `changes this` |
| `this` in `get_mut` | `this`: the Reg `get_mut` was called on | `changes pools` |
| `k`, `i` | none (default: read-only numbers) | no |
| `pool` | `ref mut` on the opened name: a view of the registry's pool that may change it | yes |
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
- A generic method whose type parameter is returned or stored by value (`fn make[V: Visitor]() -> V`)
  through `dyn`, E2050: "`Inspect` cannot be used as `dyn Inspect`: `make` returns its `V`, and a `dyn`
  value has one entry per method; take the visitor as `ref mut v: V`, or use a type parameter
  `[I: Inspect]` instead of `dyn Inspect`".
- `cast x as D` / `cast x if D` between unrelated types, E2020, with "`Box[dyn AnyPool]` is now
  `owns dyn AnyPool`" where that is the cause.
- `if Some(ref mut pool) = cast this.pools[k] if Pool[T]:`, E0115: "an `Option` or a `weakref` link is
  opened with `exists as`: write `if this.pools[k] is Pool[T] as ref mut pool:`".
- `as?` / `as!`, E1010 (today E0100 with the line, pasted in part 2): "a downcast is written
  `cast x if D`, opened with `exists as` (or `cast x as D`, which stops the program if it is not one)".

**7. Cost.** A `cast`: one load of the method table's type id and one compare; safe Rust's
`downcast_mut` was measured slower (1.710x and 1.606x against the pool table's 1.208x and 1.084x of plain
C, `phase3b-references.md` 3.4 as quoted in `phase3b-graphs-callbacks.md` 6.3); `V2/rules.md` section 2
gives 1.223 to 1.244 of plain C against `downcast_mut` 1.571 to 1.635 (AN-22, PROVISIONAL). The `dyn`
entry: the indirect calls Rust's hand-written workaround makes.

**8. Kind:** a smarter compile-time solution.

---

## AN-68: `if c = cast s if Circle:` gets E2036 with a help that is not what was meant

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

**Meaning changed.** Run 4 accepted the programmer's own line, `if c = cast s if Circle:`, as its opening
form. The owner chose `exists as` for opening (decision 1), so the opening is
`if s is Circle as c:` (R13, R14), and the programmer's line is refused again (E2036, as today),
now with a message that gives the line meant (part 6) instead of today's "did you mean
`c == cast s if Circle`?". The two `if`s in the line are in `V2/changes.md` section 11, question 5, for
the owner.

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `s` | none (default: a read-only view of an element) | no |
| `c` | none (default for an opened name: a read-only view of the same object as a `Circle`) | no |

(`t` is declared earlier in the function, as the function's own `f64`; this fragment does not declare
it.)

**5. Means exactly.** Today's spelling of the same meaning (`means/m68_if_some.em`,
`if Some(c) = cast s if Circle:`):

```text
=== m68_if_some.em
exit=0
```

Today's compiler does not know the opening word (`V2/changes.md` section 7, `q1_holds.em`, which probed run 4's `holds`).

**6. Still refused.**
- `if c = cast s if Circle:` (the OLD line), E2036: "`c = cast s if Circle` always matches, since `c`
  takes any value, here an `Option[Circle]`; to run the block only when `s` is a Circle and call it `c`,
  write `if s is Circle as c:`".
- `if Some(c) = cast s if Circle:`, E0115: "an `Option` or a `weakref` link is opened with `exists as`: write
  `if s is Circle as c:`".
- `if s exists as c:`, where `s` always holds a value, E2036: "`s` is a shape, which always holds a value, so
  this `if` would always run; to test whether it is a Circle write `if s is Circle as c:`; to
  name it write `c = s` on the line before".
- `if s is Circle as s:`, E2106: "`s` already names a value (line 1); `exists as` declares a new
  name: `if s is Circle as c:`".

**7. Cost.** None beyond the one type-id compare of the cast (AN-67).

**8. Kind:** a readable spelling of Rust's guarantee (`if let Some(c) = s.downcast_ref::<Circle>()`),
with today's unhelpful help replaced by the line meant.

---

## AN-69: Generics friction (A16)

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
fn sum[T](xs: Span[T]) -> T:                 # not `pub`: its bound is worked out from the body
    acc = copy of xs[0]
    for x in xs[1..]:
        acc = acc + x
    return acc

pub fn total[T: Add[T, Output = T] + Copy](xs: Span[T]) -> T:    # `pub`: the bound is written
    ...

extend Array[Vec3] implements Neg:           # accepted in a program (a package nothing depends on)
    ...
```

Changed from run 4: `copy of` (was `copied`). Rules (R14, R17): a function that is not `pub` gets its
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
  write `pub fn total[T: Add[T, Output = T] + Copy](xs: Span[T]) -> T:`" (a non-`pub` function never
  gets it).
- E2121 in a library package: "`Array[Vec3]` and `Neg` both belong to other packages, and packages that
  depend on this one could see a second `Neg` for `Array[Vec3]`; wrap it: `struct Vecs(Array[Vec3])`
  and `extend Vecs implements Neg:`".
- E2041, overlapping implementations (the most specific one winning is unproven for Ember): "`i32`
  already gets `Describe` from `extend[T: Debug] T implements Describe` on line 5, and two
  implementations may not apply to one type; narrow the general one's bound so it leaves `i32` out, or
  give `i32` its own function: `fn describe_int(x: i32) -> String:`".
- A non-`pub` body calling a method no interface in scope declares, E2072: "`T` has no `weight`: no
  interface in scope declares `fn weight`; declare one (`interface Weighed: fn weight() -> int`) and the
  bound follows" (a duck-typing extension is not taken: it would let a typo surface at a call far away).

**7. Cost.** None at run time: every instance compiles to the code written bounds give.

**8. Kind:** a smarter compile-time solution (bounds and the orphan exemption); the overlap is an
understandable limitation kept, with its message.

---

## AN-70: `virtual` in a class that cannot be subclassed is accepted with a warning (W2111)

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
        this.angle += 90.0 * dt
        if this.angle > 90.0:
            this.angle = 90.0
```

Changed from run 4: the virtual says `changes this` (was `ref mut this` after the brackets, R5, decision
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

Words an answer would need that `V2/vocabulary.md` has no row for. None is used in the NEW code above.

| word | why |
|---|---|
| `min` as a function of two values (`min(a, b)`) | AN-70's override (run 3's and run 4's) wrote `this.angle = min(this.angle + 90.0 * dt, 90.0)`. `V2/vocabulary.md` has `min` only in the iterator row, "as a method" (`xs.iter().min()`). The NEW body writes the same with `+=` and an `if`; a row for the prelude function (as `len`, `abs`) would let the line stay as the programmer wrote it |

## For the owner

1. **A walk whose callback may remove nodes keeps a link, not a view (AN-62 (b)).** Your decision 5 (a
   read-only name promises that what it reads does not change while it is used) and decision 17 (a
   function's type says what it changes) together mean: when the callback's type says `changes Node`, the
   walk may not hold a read-only view of a node across the call, since the callback could change that
   node. The walk therefore names a link that may outlive its node: `up = weakref p` before the call and
   `while up exists as n:` after it. It reads plainly and stops at the first deleted parent, as decision 15
   says; it costs one weak-count subtract and add per step (Rust's walk pays a count pair per step for the
   same program). With a callback that only looks (`fn(Node)`, the usual case) the walk keeps views and
   pays nothing per step. Is the link spelling acceptable for the remove-nodes case? AN-62 writes it
   meanwhile.

## Notes for the lead and the rule writer

Points where the v2 rules did not settle what an answer needed, or where an answer found a gap; what this
area wrote meanwhile.

1. **A function value that changes a real variable, stored inside that variable (a possible hole in
   R12).** R12 counts the real variables a lambda captures as part of the function itself, checked in
   the function that writes the lambda. This program seems to pass every v2 rule:
   `t = Tree(...)`, `t.on_visit = fn(n: Node): t.root.children.clear()`, `d = t.depth_total()`. The lambda
   is a plain `fn(Node)` (it changes only `t`, a real variable it captures), it does not outlive `t`
   (E3026 does not apply), and `depth_total`'s first line (`changes on_visit`) matches its body. Inside
   `depth_total`, AN-62 (a)'s views of `t`'s nodes then live across a call that ends them (the case of
   run 3's `an62_defer_asan.c` mode 1, a read of freed memory). R12's "a call that may run it as a change
   of each one it changes" makes `t.depth_total()` a change of `t` in `main`, but no rule refuses a method
   call whose own object is changed by the call beyond the method's first line. A fix in Rust's terms: a
   function value that captures a real variable may not be stored in a place that variable owns (Rust
   refuses it as a borrow conflict); or a method call on `x` that may run a function value changing `x`
   is refused unless the method's first line says `changes this`. AN-62 (a) assumes such a refusal.
2. **R11's region walk and decision 5.** R11's last paragraph describes a walk whose views live across a
   call that may end nodes, protected by a deferred-drop region and stopping at the first ended parent.
   With R8 (g) and R12, a view of a node may not live across a call whose type says it changes nodes, and
   a callback that removes a node from a `children` list changes a `Node`. AN-62 (b) therefore keeps a
   link, not a view. R11's sentence covers only a call that may end nodes without changing any; which
   programs those are (a callback that changes only a `Tree` that owns the nodes?) is not said, nor
   whether changing an object owned by a `Tree` counts as changing the `Tree` for R12's `changes T`.
3. **A view given as the argument of a call that may change it.** In AN-62 (b), `this.on_visit(p)` gives
   `p`, a read-only view of a node, to a function whose type says `changes Node`. AN-62 reads R12's "no
   read-only view of what they change ... lives across the call" as not covering a view whose last use is
   that argument, and the callee's own body as checked by R4's promise and R8 (g) (it may not change the
   node it was given while it reads it, unless it proves the two differ). One sentence in R12 would say so.
4. **A compile-time saving for AN-62 (b)** (not in the rules, so not used in its cost): a `weakref` local
   whose whole life lies inside a deferred-drop region needs no weak count, since the region keeps every
   block set aside until it ends, after the link. Placing a region around (b)'s `for` loop would remove the
   per-step subtract and add, leaving one header test per step (run 3's W2 with a header test per hop:
   1.579x of plain C). Worth measuring in the next measuring phase.
5. **The parent-link proof fails silently.** R11's condition (1) is package-wide: one line elsewhere
   (a link given to a node that its target does not then own, a `remove` with no clearing) makes every
   `parent` read in the package test the header, 1.579x of plain C instead of 1.176x on W2, with no word
   in the program and no message. Decision 14 asks to say honestly what it costs; a note from the compiler
   naming the line that stops the proof (or an `ember inspect` view of it) would let the programmer see
   it.
6. **`weakref this` in a method whose first line names only a field.** AN-62's and AN-63's `add`
   (`changes children`) write `child.parent = weakref this`, which adds to `this`'s weak count (a number
   in its header) and changes no field. AN-62 and AN-63 read that as allowed; R5 does not say so.
