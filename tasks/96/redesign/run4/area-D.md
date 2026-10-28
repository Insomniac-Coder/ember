# Area D: linked objects, callbacks, interface types (run 4, phase 1; agent 05)

Every answer of area D (AN-53 to AN-70) brought to the run-4 rules (`RUN4/rules.md`) and words
(`RUN4/vocabulary.md`). `RUN3` means `tasks/96/redesign/run3/`, `RUN4` means `tasks/96/redesign/run4/`.
"R1" .. "R18" are the rules of `RUN4/rules.md`; "row N" is the owner's agreed row N in `inputs/JOB.md`;
G-01 .. G-11 are the problems of `RUN3/22-re-attacker.md`; "gap 1", "gap 2" are those of
`RUN3/24-faithfulness.md`.

Power: `PowerLineStatus` printed `Online` at the start of this work and again before the one `ember check`
run. No compiler change, no git command, no Ember program run. Scratch files:
`C:\Users\ism19\Code\ember\target\redesign4\05-area-D\` (`template.md` and `build.py`, which copy every
OLD part from `RUN3/combined-v4.md` unchanged; `m55_for_each.em` and `check_output.txt`, the one new
`ember check`).

## Terms used in this area

- **function value**: a function or lambda kept in a variable, a list or a field (a callback, a
  handler): two words, its code and its **state** (what it captured).
- **capture**: a name a lambda uses from the function that made it. **The real variable** (row 6): that
  very variable, not a copy; a write in the lambda is seen by the function, and back.
- **capture list**: what follows `with` after a lambda's parameters (`fn() with copied score:`).
- **box**: a small counted heap block holding one captured variable, shared by every user of the
  variable (Python's "cell").
- **holder**: the object or list whose field or element holds a function value or another object.
- **opening**: R13's form `if p = cam.target:` (or `while p = a:`): the block runs when the `Option` or
  the `weak` link holds something, and names it `p` there.
- **count pair**: one +1 and one -1 of an object's count around a use, to keep it alive across it.
- **deferred-drop region**: a stretch of code (a loop) during which an object whose last owner ends is
  set aside whole, marked dead, and freed when the stretch ends (R10 point 2).
- **unknown call**: a call through a function value or a `dyn` value, or into code whose effects the
  compiler cannot see; it is taken to do everything its type allows.
- **set aside**: kept, with its fields and memory untouched, until the region ends.
- **AddressSanitizer** (ASan): a clang option that stops a program with a report the moment it reads or
  writes memory it must not.
- **today**: `ember check <file>` with today's compiler (`target/redesign3/bin/ember.exe` in run 3,
  `target/redesign4/bin/ember.exe` in this run). Every claim about today has its output pasted.

Run 3's area rules D1 to D8 are replaced by the run-4 rules: D1 and D4 by R12 (where a real variable
lives; a stored function value is two words); D2 by R3 and R12 (the capture words); D3 by R12 (what a
call that may run a function value changes); D5 by R10; D6 by R11 and R13 (`weak`, the opening); D7 by
R11 (`set by`); D8 by R3 and R14 (interface values). The answers below cite only the R rules.

The OLD code of every answer is unchanged, so every OLD part, with its `ember check` output, is copied
unchanged from `RUN3/combined-v4.md` (file names in it are relative to `target/redesign3/11-area-D/`).

## Answers whose meaning changed under the run-4 rules

| answer | what changed, and how |
|---|---|
| AN-58 | run 3's NEW `for h in hs: h()` is refused by R12 (a call through a stored function value needs a name that may change its holder, E3023); NEW writes `for ref mut h in hs:`. What the program does (returns 1) is unchanged |
| AN-60 | the `for` form's second way out (read-only `fn(Ui)` handlers walked by `for ref mut h`) now holds only where no function value of type `fn(Ui)` can reach a Ui through what it captured and change its `handlers` (R12, G-05); the weak handler is written with the opening form |
| AN-62 | the walk keeps views (`cur = ref n`, `while p = cur.parent:`, `cur = ref p`) instead of copies of the link (run 3's `a = copied n.parent`): no weak count and no header test per step (R11, gap 1). One case now ends differently: if the callback ends an ancestor above the node the walk is on, the walk still visits that ancestor once (it is set aside, not freed), where run 3's header test stopped before it. Memory stays safe; see "For the owner" 1 |
| AN-68 | the programmer's first spelling, `if c = cast s if Circle:` (the OLD line), is now the accepted one (R13); run 3's NEW `if Some(c) = cast s if Circle:` is refused (E0115). Run 3's refusal of the OLD line is withdrawn |

Spelling changes that do not change meaning: `this` for `self` (in the code of AN-53, AN-60, AN-61,
AN-62, AN-67, AN-70 and AN-69's message; in the owner's-test tables of AN-63, AN-64, AN-65, AN-66);
`ref mut this` for `mut self` (AN-60, AN-61, AN-70's `Door`);
`ref mut` at calls (AN-54 `run(ref mut cb)`, AN-60 and AN-61 `this.handlers[i](ref mut this)`, AN-67
`x.inspect(ref mut s)`); the opening form for `if Some(...)` and `while Some(...)` (AN-60, AN-62, AN-63,
AN-65, AN-67); AN-55's iterator line (`for_each` for `inspect(...).count()`, because `count` has no row
in `RUN4/vocabulary.md`; see "Words needed"); AN-70's override says `changes angle`.

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

    fn on_click(owned f: fn()) changes handlers:
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
    b.on_click(fn() with copied score: score += 1)   # row 6's copy: the handler's own `score`
    b.click()
    return score                       # 0: only the handler's copy went up
```

Changed from run 3: `this` for `self` (R5). The calls write no `ref mut`: `b.on_click(...)` and
`b.click()` are methods called on `b`, and the first lines say what they change (R4); `f` is an `owned`
parameter, and a hand-over is written with no word (R4). `click` says `changes handlers` and walks with
`ref mut h` because a call through a stored function value may change its own state, so it is a change
of the place that holds it (R12). Where `game`'s real `score` lives (R12): at its own address, since
`b` never leaves `game` and both methods it is handed to say what they change, so the handler provably
ends first. In `game_copy` the captured `score` is named (`copied`), so it is the handler's own state.

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `handlers` (field) | none (default: the Button's own list; each element its own function value) | only in a method whose first line says `changes handlers` (or `ref mut this`) |
| `this` in `on_click` | `this`: the Button the method was called on | `changes handlers`: that field only |
| `f` | `owned`: handed over to `on_click` | its new owner may; it hands it on to the list |
| `this` in `click` | `this` | `changes handlers` (R12: calling a stored function value may change it) |
| `h` | `ref mut`: an element of `handlers`, the only name for it while used (R8 (f)) | yes; calling it changes what its body changes, its own state included (R12) |
| `score` in `game` | none (default: `game`'s own `int`, made by `=`) | its owner may |
| `b` in `game` | none (default: `game`'s own Button, made by `=`) | through methods whose first line says `changes` |
| `score` in the lambda | none (row 6: the real `score` of `game`) | yes, as row 6 says (`score += 1`) |
| `score`, `b` in `game_copy` | none (default: `game_copy`'s own `int` and Button) | as in `game` |
| `score` in `game_copy`'s lambda | `copied`: the handler's own copy (row 6) | yes, its own copy, through `click`'s `changes handlers` |

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
copies the `int`, which is what `with copied score` names.

**6. Still refused.**
- A capture of someone else's place in a function value that may outlive it (AN-57), E3026: "the click
  handler changes `total`, which belongs to `register`'s caller, and `b` keeps the handler after
  `register` returns; keep the total where the handler can own a share of it: `count = shared Counter()`
  and `fn() with shared count: count.n += 1`".
- A capture used by a thread that may outlive it (AN-55), E7004.
- `fn click():` with `for h in this.handlers: h()`, E2105: "`click` calls the handlers stored in
  `this.handlers`, which may change their own state; say so on its first line,
  `fn click() changes handlers:`, and walk them with `for ref mut h in this.handlers:`" (R12).
- A loop that walks a list a function value changes, around a call that may run it (R12, R8 (c)),
  E3020: "this loop walks `xs` (line 5), and `hs[0]()` (line 6) may run the function made on line 3,
  which pushes to `xs`; call it after the loop, or collect what to add and push it after the loop".
- The stated limitation of R12: inside `click` the compiler cannot see which handlers a Button holds,
  so `h()` is taken to make every change any `fn()` of the program makes. If some `fn()` anywhere adds
  a handler to a Button, `click`'s `for` walk is refused (E3020), and the message says the compiler
  cannot tell which handlers this Button holds (R18, G-07). In this program no `fn()` does, so it is
  accepted.

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
    cb = fn() with owned name, copied score:
        score += 1                    # its own copy
        println(name)                 # its own String
    run(ref mut cb)                   # `run` may change `cb`: its own copy of `score` goes up
    println(score)                    # 0, as `copied` says
```

Changed from run 3: the call says `run(ref mut cb)`, as `run`'s first line says `ref mut f` (R4, the
owner's decision 1).

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `f` | `ref mut`: `main`'s function value, lent | yes (`ref mut`): `cb` changes its own copy when called (R12) |
| `name` in `main` | none (default: `main`'s own String, made by `=`) | its owner may |
| `score` in `main` | none (default: `main`'s own `int`) | its owner may |
| `cb` | none (default: `main`'s own function value, made by `=`) | its owner may; it is lent `ref mut` to `run`, and the call says so |
| `name` in the lambda | `owned`: handed over to the function | its new owner may |
| `score` in the lambda | `copied`: the function's own copy | yes, its own copy (`score += 1`) |

**5. Means exactly.** Today's `owned fn` does exactly this (moves the `String`, copies the `int`); NEW
only writes it on each name (`means/m54_named_captures.em`):

```text
=== m54_named_captures.em
exit=0
```

**6. Still refused.**
- `owned fn(): ...`, E0109: "`owned fn` does not say what the function takes or copies; name it on each
  captured name: `fn() with owned name, copied score:`; a name not listed is the real variable".
- `fn() with owned score:` for an `int`, E2085: "an `int` is copied, not handed over; write
  `fn() with copied score:`".
- `println(name)` after `with owned name`, E3040: "`name` was handed to the function on line 7; to keep
  using it here, give the function a copy, `with copied name`, or share it, `with shared name`".
- `fn() with copied name, score:`, E0116: "`copied` here belongs to `name` only; say what the function
  does with `score`: `with copied name, copied score`, or leave `score` out to use the real `score`".
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
    xs.iter().for_each(fn(v) => seen.push(v))                # the real `seen`
    h = thread.spawn(fn() with owned xs: work(xs))           # `xs` handed to the thread
    println(seen.len())
    println(h.join())
```

Changed from run 3: the iterator line is `xs.iter().for_each(...)` (was
`_n = xs.iter().inspect(...).count()`): `count` has no row in `RUN4/vocabulary.md` (see "Words
needed"), and `for_each` (a row of section 3.8) is, like `inspect`, a library call that runs the function
it is given while it runs (R12). The line still does what the intent says: it records every value the
iterator sees in the real `seen`. The thread line is run 3's.

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `xs` in `main` | none (default: `main`'s own list) | its owner may |
| `seen` in `main` | none (default: `main`'s own list) | its owner may |
| `v` | none (default for a lambda parameter: it reads; an `int`, copied in) | no |
| `seen` in the lambda | none (row 6: the real `seen`) | yes (`push`), as row 6 says |
| `h` | none (default: the `JoinHandle` `thread.spawn` returns, `main`'s own) | its owner may (`join`) |
| `xs` in the thread's function | `owned`: handed over to the thread | its new owner may |

**5. Means exactly.** `means/m55_spawn_owned.em` (`thread.spawn(owned fn() => work(xs))`, `xs` not
used after) and the scoped form `means/m55_scope.em` (the thread uses the real `xs` inside
`with scope = thread.scope():`):

```text
=== m55_scope.em
exit=0
=== m55_spawn_owned.em
exit=0
```

and the whole NEW program in today's words (`target/redesign4/05-area-D/m55_for_each.em`: the
`for_each` line as written, `thread.spawn(owned fn() => work(xs))`), checked in this run
(`target/redesign4/05-area-D/check_output.txt`):

```text
=== m55_for_each.em
exit=0
```

**6. Still refused.** `thread.spawn(fn() => work(xs))`, E7004: "this thread may outlive `main`, so it
cannot use `main`'s `xs`; say what the thread holds: `fn() with owned xs:` (handed over) or
`fn() with copied xs:` (a copy of its own), or start it inside `with scope = thread.scope():`, which
waits for it" (R15). `fn() with shared xs:` on `thread.spawn`, E7015: "this thread runs beside `main`,
and `shared` owners stay on one thread; hand `xs` over, `fn() with owned xs:`, or give the thread a copy,
`fn() with copied xs:`". A box (R12) is never used across threads: its count is a one-thread
count.

**7. Cost.** None beyond C: the lambda given to `for_each` holds `seen`'s address, the hand-over moves
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

**3. NEW.** (Unchanged from run 3: it uses neither `self` nor a call to a `ref mut` parameter.)

```ember
fn each_name(owned f: fn(owned String)):
    f(String("ann"))

fn main():
    names: Array[String] = []
    each_name(fn(owned s) => names.push(s))       # takes `s` over, keeps it
    each_name(fn(s) => println(s.len()))          # only reads `s`
```

`f(String("ann"))` and `each_name(...)` hand their arguments over to `owned` parameters, written with
no word (R4).

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `f` | `owned`: handed over to `each_name` | its new owner may |
| `names` | none (default: `main`'s own list) | its owner may |
| `s` in the first lambda | `owned`: handed over to the lambda | its owner may (it hands it on to `names`) |
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
  to take it over write `fn(owned s) => names.push(s)`".
- `fn(owned s)` where the expected type is `fn(String)` (a read-only parameter), E2020 (today's code for
  a type that does not match): "`each_name` lends the name and keeps it; to keep your own, write
  `fn(s) => names.push(copied s)`".

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
          `count = shared Counter()` in the caller, and `b.on_click(fn() with shared count: count.n += 1)`
```

Callable types are printed as written: `fn()`, `fn(ref mut Ui)`, never `Callable0` (R18). Changed from
run 3: the code E3026 is written on the message (R12, `RUN4/rules.md` section 3).

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `b` | `ref mut`: the caller's Button | yes (`ref mut`) |
| `total` | `ref mut`: the caller's `int` | yes (`ref mut`) |
| `total` in the lambda | none (row 6: the real `total`, here the caller's place) | yes, as row 6 says; refused because the handler may outlive it |

**5. Means exactly.** Nothing compiles for the refused program, which is the point. The accepted form in
the help is AN-53's box (`means/m53_button_box.em`, `exit=0`, pasted under AN-53); its
`shared Counter` is today's class `Score` there.

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

**Meaning changed (the line, not the result).** Run 3's NEW walked with `for h in hs:`. Under R12 a call
through a stored function value may change its own state, so its holder is reached through a name that
may change it; a plain loop name only reads, so `for h in hs: h()` is refused (E3023, part 6). The walk
is `for ref mut h in hs:`, the form AN-53's `click` already used. `game` still returns 1.

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `score` | none (default: `game`'s own `int`) | its owner may |
| `hs` | none (default: `game`'s own list; its elements its own function values) | its owner may |
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

**3. NEW.** (Unchanged from run 3.)

```ember
fn call_once(owned f: once fn()):
    f()

fn call_twice(f: fn()):
    f()
    f()

fn main():
    s = String("a")
    call_once(fn() with owned s: consume(s))      # the function owns `s` and gives it away: once
    t = String("b")
    call_twice(fn(): consume(copied t))           # each run hands over a new copy
```

The arguments of `call_once`, `call_twice` and `consume` are hand-overs to `owned` parameters or plain
reads, so no call writes `ref mut` (R4). `call_twice` calls `f` through a read-only parameter: allowed,
since its argument changes nothing of its own state (R12 refuses only a function value that changes its
own state given to a parameter that only reads).

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `f` in `call_once` | `owned`: handed over to `call_once`; `once` in its type: a function the call uses up | it is used up by the call |
| `f` in `call_twice` | none (default: a read-only function value; calling it changes only what its body changes, R12) | no |
| `s` in `main` | none (default: `main`'s own String) | its owner may |
| `s` in the first lambda | `owned`: handed over to the function | its owner may (it hands it to `consume`) |
| `t` in `main` | none (default: `main`'s own String) | its owner may |
| `t` in the second lambda | none (row 6: the real `t`, read); `copied` before it makes the value handed over | no (read only) |

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
- `call_twice(fn() with owned s: consume(s))`, E3013: "this function gives `s` away, so it can run only
  once, and `call_twice` may run it twice; if `call_twice` runs it once, declare the parameter
  `owned f: once fn()`; otherwise hand over a copy each time: `fn(): consume(copied s)`".
- `fn call_once(f: once fn()):`, E3013: "`call_once` uses `f` up, so it must hold it: write
  `owned f: once fn()`" (R12).
- `call_twice(fn(): consume(s))` (no word), E3013: "`consume(s)` would hand `main`'s `s` over from inside
  a function that `main` keeps; give the function its own `s` to hand over, `fn() with owned s:
  consume(s)` (then it runs once), or hand over a copy, `consume(copied s)`".

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

    fn fire() ref mut this:
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

fn watch(ref mut ui: Ui, ref mut b: Button):       # `Button` is AN-53's
    bump_ui = fn() with weak ui:                   # a link that does not keep the Ui alive
        if ref mut me = ui:                        # the Ui still lives: call it `me`, which may change it
            me.clicks += 1
    b.on_click(bump_ui)
```

Python's list iterator is this loop (a position checked against the length at every step), so a handler
added during the dispatch runs in it and a handler that removes itself makes the next one move into its
place, as in Python.

Changed from run 3: `fn fire() ref mut this:` (was `fire(mut self)`) and `fn remove_current() changes
handlers, current:` (R5). The call says `ref mut this` (R4: the handler's parameter is `ref mut Ui`); it
copies the handler's two words out of the list first, so the list is not an argument of the call (R12)
and nothing points into it while the handler runs. `fire` needs `ref mut this`, not `changes ...`,
because it gives the whole Ui to a handler that may change any field (R4: `ref mut` only before what this
function may change, E3027). The handler that must not keep the Ui alive, run 3's one-line
`if Some(ref mut me) = ui:`, is written with the opening form (R13) as a whole function, `watch`.

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `handlers` | none (default: the Ui's own list of function values) | `changes handlers`, or `ref mut this` |
| element type `fn(ref mut Ui)` | `ref mut` on its parameter: a handler gets the caller's Ui | yes, the Ui (`ref mut`) |
| `current`, `clicks` | none (default: the Ui's own `int`s) | `changes current`, or `ref mut this` |
| `this` in `fire` | `this`: the Ui `fire` was called on | yes: `ref mut this` (any field) |
| `this` in `remove_current` | `this` | `changes handlers, current`: those two fields |
| `ui` in `count` | `ref mut`: the caller's Ui | yes |
| `ui` in `add_once` | `ref mut`: the caller's Ui | yes |
| `ui` in `watch` | `ref mut`: the caller's Ui | yes |
| `b` in `watch` | `ref mut`: the caller's Button | yes (its `on_click` changes `handlers`) |
| `bump_ui` | none (default: `watch`'s own function value, made by `=`) | its owner may; handed over to `on_click` with no word (an `owned` parameter) |
| `ui` in the `with weak ui` lambda | `weak`: a link that does not keep the Ui alive | no (a link); `me` may (below) |
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
handler's own state block instead, which run 3's W4 measured (part 7). `watch`'s handler is the `weak`
read of AN-65 (`c/an63_links.c` part 2).

**6. Still refused.**
- `for h in this.handlers: h(ref mut this)` with `fn(ref mut Ui)` handlers (the agreed limitation: a
  list may not change while a loop walks it), E3020: "this loop walks `this.handlers` (line 7), and
  `h(ref mut this)` (line 8) gives each handler the whole Ui, so a handler may add to or remove from
  `this.handlers`; walk by position, as Python's list does: instead of the `for` line and
  `h(ref mut this)`, write `this.current = 0`, `while this.current < len(this.handlers):`,
  `this.handlers[this.current](ref mut this)` and `this.current += 1`".
- **Meaning changed** (R12, G-05): the second way out, read-only handlers walked with `for`
  (`handlers: Array[fn(Ui)]`, `fn fire() changes handlers:`, `for ref mut h in this.handlers: h(this)`),
  is accepted only when no function value of type `fn(Ui)` in the program can reach a Ui through what it
  captured (a `weak`, `shared` or `sharedref mut` link, or a real variable) and change its `handlers`.
  Run 3 offered it for every program; G-05 is a `fn(Ui)` handler that captures `with weak ui` and pushes
  to `me.handlers`, which the `for` walk then reads from a freed buffer. That program is refused, E3020:
  "this loop walks `this.handlers` (line 7), and `h(this)` may run the handler made on line 15, which
  adds to the `handlers` of the `Ui` it links to (`with weak ui`), and that may be this one; walk by
  position instead of the `for` line: `this.current = 0`, `while this.current < len(this.handlers):`,
  `this.handlers[this.current](this)`, `this.current += 1`, with `fn fire() changes handlers, current:`".
  The E3020 message for the first bullet therefore offers only the position loop.
- `if Some(ref mut me) = ui:`, E0115: "an `Option` or a `weak` link is opened by naming what it holds:
  write `if ref mut me = ui:`" (R13).
- Not refused, warned: a handler that captures the Ui that holds it with `with shared ui` makes a cycle
  of owners that is never freed (a leak, not a memory error, as in Rust); L3001: "the handler is kept by
  `ui` and keeps `ui`; capture it weakly, `with weak ui`, or take the Ui as the handler's parameter".

**7. Cost.** A call copies two words out of the list (plain C copies the same `{fn, env}` pair). A
handler's body holds one count on its own state only across a call that may remove it and only when it
uses its state after it (a rare path). The phase before measured this dispatch at 1.045 and 1.037 of
plain C doing exactly the same work with the same allocator, safe Rust at 1.490 and 1.472
(`target/redesign2/graphs-callbacks-attack2/bench/runs/w4_a.txt`, `w4_b.txt`;
`phase3b-graphs-callbacks-attack.md` 6.4). Run 3's W4, this program's workload: 1.091x of plain C with a
captured variable in the handler's own state block (R12's placement here), 1.255x with a box per
captured variable, safe Rust 1.358x (`RUN3/26-timer.md` lines 29, 31, 32). The `weak` capture: one weak
count while `bump_ui` lives and one header test per run (R11).

**8. Kind:** a smarter compile-time solution (where each count is needed is decided when compiling; the
`for` refusal is the agreed limitation).

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
    fn fire() ref mut this:
        i = 0
        while i < len(this.handlers):
            this.handlers[i](ref mut this)
            i += 1
```

Changed from run 3: `ref mut this` (was `mut self`) and the call's `ref mut this` (R4, R5).

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `this` | `this`: the Ui `fire` was called on | yes: `ref mut this` |
| `i` | none (default: `fire`'s own `int`) | its owner may |

(`this.handlers[i]` declares no name: its two words are read and called, and nothing points into the
list during the call, R12.)

**5. Means exactly.** `c/an61_dispatch.c` mode 2, pasted under AN-60: clean under ASan.

**6. Still refused.**
- `h = ref mut this.handlers.get_mut(i).unwrap()` then `h(ref mut this)`, E3022 (R8 (f): `h` is the
  only name for that element while it is used, and `ref mut this` gives the handler the Ui that holds
  it): "`h` points into `this.handlers`, and `h(ref mut this)` lets the handler change
  `this.handlers`; call it by position: `this.handlers[i](ref mut this)`" (the `get_mut` row of
  `RUN4/vocabulary.md`).
- The same with a plain `h = this.handlers.get_mut(i).unwrap()`: `h` is then a read-only view (R2), so
  calling the stored handler through it is refused first, E3023: "`h` only reads `this.handlers[i]`,
  and calling a stored handler may change it; call it by position: `this.handlers[i](ref mut this)`"
  (R12).

**7. Cost.** As AN-60. The refused form had no cheaper code: the copy back it needs is what made it
unsafe.

**8. Kind:** a smarter compile-time solution (the hole is closed when compiling; the form that works
costs what C costs).

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

**3. NEW.**

```ember
class Node:
    value: int = 0
    parent: weak Node set by children
    children: Array[Node] = []

class Tree:
    root: Node
    leaves: Array[weak Node] = []
    on_visit: fn(Node)

    fn depth_total() changes on_visit -> int:
        depth = 0
        for leaf in this.leaves:
            if n = leaf:                      # the leaf still lives: call it `n`
                cur = ref n                   # a view of that node, not a copy of a link
                while p = cur.parent:         # its parent, while there is one
                    this.on_visit(p)          # unknown: may end a `Node`
                    depth += 1
                    cur = ref p               # `cur` now views the parent
        return depth
```

**Meaning changed.** Run 3's walk kept its own copy of the link (`a = copied n.parent`,
`while Some(p) = a:`, `a = copied p.parent`); each copy is an ordinary `weak` link, which by R11 now
always takes a weak count and is read with a test of its target's header. The walk now keeps a view of
the node it is on (`cur`) and reads `cur.parent` itself, a `set by` link, which R11 reads with a plain
load and a test for no link, with no header test and no count, because its target lives while it holds
this node (run 3's gap 1, settled in R11; `RUN4/changes.md` section 11 names this rewrite). `cur = ref
n` writes the word that says what `cur` is (a read-only view, R2): a plain `cur = n` would read as row
9's hand-over of `n`. `cur = ref p` points `cur` at the parent: `cur` was declared with `=`, so `=` gives
it a new view (R2), checked against its new target. `if n = leaf:` and `while p = cur.parent:` are the
opening form (R13). `this` for `self`, `changes on_visit` as run 3: `depth_total` calls the stored
function value `on_visit`, a change of the field that holds it (R12).

The program writes nothing for the protection. `Node` objects can be ended by `this.on_visit(p)` (an
unknown call, which may do anything a `fn(Node)` may do), and the views `n`, `cur` and `p` point at
`Node` objects reached through `weak` links, so R10 protects them; `Node` has no `drop` method of its
own, so the protection is a deferred-drop region around the `while` loop: entering it adds 1 to a
per-thread counter, leaving subtracts 1, and a `Node` whose last owner ends while the counter is above
zero is set aside whole (marked dead, so an ordinary `weak` read of it gives nothing) and freed when the
loop ends. If the callback takes `p` out of the tree, the removal clears `p.parent` (`set by`), so the
walk stops there, as run 3's C shows (part 5).

What changed in what the program does: if the callback ends an ancestor **above** `p` (say `q`, `p`'s
parent), `q` is set aside, not freed, and its children's `parent` links are cleared only when it is
freed at the end of the region; `p.parent` still names `q`, and the `set by` read has no header test,
so the walk visits `q` once more before it stops (`q.parent` was cleared when `q` was taken out of its
holder). Nothing is read from freed memory: `q`'s block is kept until the region ends. Run 3's walk
tested the header of every hop and stopped before `q`; Python, where `q` is freed at once and its
children's weak references die with it, stops before `q` too. This is a reading of R10 point 2 and R11
together, not a run C program; "For the owner" 1 asks which meaning is wanted and gives a fix that costs
nothing per step.

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `value` | none (default: the node's own `int`) | `changes value`, or `ref mut this` |
| `parent` | `weak` (does not keep its node alive), `set by children` (only `children` sets it) | no: the program never assigns it |
| `children` | none (default: the node's own list; each element has this one owner) | `changes children`, or `ref mut this` |
| `root` | none (default: the tree's own Node) | `changes root`, or `ref mut this` |
| `leaves` | `weak` on its elements: links that do not keep the leaves alive | `changes leaves`, or `ref mut this` |
| `on_visit` | none (default: the tree's own function value) | `changes on_visit`, or `ref mut this` |
| `this` | `this`: the Tree `depth_total` was called on | `changes on_visit`: calling the stored function value may change it (R12) |
| `depth` | none (default: the method's own `int`) | its owner may |
| `leaf` | none (default: a read-only view of an element of `leaves`, a `weak` link) | no |
| `n` | none (default for an opened name: a read-only view of the node `leaf` links to, kept alive by R10) | no |
| `cur` | `ref`: a read-only view of a node (first `n`, then each parent) | no, not the node; `=` points it at another node (R2) |
| `p` | none (default for an opened name: a read-only view of `cur`'s parent, kept alive by R10) | no |

**5. Means exactly.** C, `c/an62_defer_asan.c`: a leaf, its parent `mid`, and a callback that takes
`mid` out of the tree and drops its last owner while the walk is on `mid`. Its walk is a plain pointer
read with no header test and no weak count (`RUN3/24-faithfulness.md` gap 1: lines 82-88 of the C),
which is what R11 now gives a walk that keeps views. Three modes, under ASan
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

Mode 1 (no protection) reads `p.parent` from freed memory (line 89, `Node *next = a->parent;`); mode 2
(the region) and mode 3 (a count pair) are clean and give the same result. The case of an ended ancestor
above the walk (part 3) is not in this C.

The workload's walk (`c/an62_walk_bench.c`, 1,000,000 nodes, 10 passes, a call through a function
pointer the C compiler cannot see on every step) built `-DSLAB` and `-DSLAB -DDEFER`; the whole difference
between the two assembly files (`c/bin/walk_noprot.s`, `c/bin/walk_defer.s`), apart from the program's
name string and the counter's declaration:

```text
205a206
> 	incq	g_defer(%rip)
233a235
> 	decq	g_defer(%rip)
```

one increment before each pass's loop and one decrement after it; nothing inside the loop.

**6. Still refused.**
- `while Some(p) = a:`, E0115: "an `Option` or a `weak` link is opened by naming what it holds: write
  `while p = a:`" (R13).
- A view into a `Node`'s contents (an element of `p.children`) kept across the unknown call: refused by
  the loop rule when a loop walks it (E3020, "the callback may change `p.children`, which this loop
  walks; ...", area A's message), and by R8 (b) otherwise (E3060 at its next use).
- `for leaf in this.leaves:` itself, if some function value of type `fn(Node)` in the program can reach
  a `Tree` through what it captured and change its `leaves` (R12): E3020, naming that function value and
  saying the compiler cannot tell which callback `this.on_visit` holds (R18).
- Not refused: a class with its own `drop` method gets a count pair around each view instead of the
  region (deferring a `drop` would move a visible action; R10 point 3). Run 3's spelling
  `a = copied n.parent` is still accepted; it costs a weak count per copy and a header test per hop
  (R11), so the view walk above is the one to write.

**7. Cost.** Per step, none beyond the unprotected walk: the region is one add and one subtract of a
per-thread counter per run of the `while` loop; reading `cur.parent` is a plain load and a test for no
link. Timed by run 3 with `target/redesign2/bench/run.ps1 -Rounds 5` (pinned to the fast cores, mains
checked each round; `c/walk_runs.txt`, medians in `c/walk_summary.txt`). Every run printed checksum
722575054.

| program | median ms | ratio to plain C, same layout | spread |
|---|---|---|---|
| plain C, same layout (`walk_plain.exe`, `-DSLAB -DARRAY24`) | 1905.5 | 1.000 | 30% |
| set by, no protection (`walk_noprot.exe`) | 2068.5 | 1.086 | 29% |
| set by, a count pair per step (`walk_retain.exe`, the phase before's rule) | 2444.0 | 1.283 | 34% |
| set by, deferred-drop region (`walk_defer.exe`) | 1797.9 | 0.944 | 37% |

**PROVISIONAL**: other agents were running; the spread (fastest to slowest) is 29% to 37%, far over the
10% the timer accepts, so these numbers rank nothing finely. What they agree with is the assembly above:
the region's walk is the unprotected walk plus two instructions per pass. These walks read the link with
no header test, which is the run-4 rule. Run 3's W2 (the same parent walk without a callback) measured
the read with no header test at 1.176x of plain C, against 1.579x for a header test on every hop and
1.603x for safe Rust (`RUN3/26-timer.md` lines 20 to 22: `diag_w2_hop_proved`, `model_w2`, `rust_w2`;
`diag_w2_hop_proved` spread 14.8%). Peak memory: 84,544 KB for every `set by` variant against 76,688 KB
for plain C (the object header). Each `weak` link in `leaves` holds one weak count, taken once when it is
stored (R11). A region holds the memory of the objects set aside until the loop ends: in this workload
none.

**8. Kind:** a new mechanism (the protection moves from every step to the rare death; nothing is
written; as fast as the unprotected walk inside the loop; as safe as the count pair, shown under ASan).

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
    parent: weak Node set by children
    children: Array[Node] = []

fn path(n: Node) -> String:
    if p = n.parent:                     # `n` has a parent: call it `p`
        return f"{path(p)}/{n.name}"
    return copied n.name

fn main():
    root = Node(name="root")
    root.children.push(Node(name="a"))                   # a.parent is root
    root.children[0].children.push(Node(name="leaf"))    # leaf.parent is a
    println(path(root.children[0].children[0]))          # root/a/leaf
    moved = root.children[0].children.remove(0)          # leaf.parent is None
    root.children.push(moved)                            # leaf.parent is root
```

Changed from run 3: `if p = n.parent:` (was `if Some(p) = n.parent:`), the opening form (R13). Moving a
node is written: taken out of one list, pushed into another. For links that are not the reverse of an
owning field (a camera's target, an observer list), `weak` alone: `target: weak Player`,
`observers: Array[weak Observer]`. For a graph, each city's roads and the roads into it:
`roads: Array[weak City]` and `into: Array[weak City] set by roads`, the cities owned by one list.

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `name` | none (default: the node's own String) | `changes name`, or `ref mut this` |
| `parent` | `weak`: does not keep its node alive; `set by children`: only `children` sets it | no: never assigned by the program |
| `children` | none (default: the node's own list; one owner per element) | `changes children`, or `ref mut this` |
| `n` | none (default: a read-only view, a parameter) | no |
| `p` | none (default for an opened name: a read-only view of the parent, kept alive until its last use) | no |
| `root` | none (default: `main`'s own Node) | its owner may |
| `Node(name="a")` given to `push` | none (default: a new value handed over by `push`, row 9's hand-over) | its new owner may |
| `moved` | none (default: `main`'s own Node, handed over by `remove`) | its owner may |
| `moved` given to `push` | none (default: handed over, row 9) | its new owner may |

`copied n.name` makes the returned String (row 9's word).

**5. Means exactly.** C, `c/an63_links.c` part 1, the same steps plus a root that ends while a child is
held elsewhere; every link store is one store, every read a plain load; under ASan
(`c/an63_links_output.txt`):

```text
1 root/a/leaf
2 leaf
3 root/leaf
4 leaf parent=None
5 100
6 gone
an63_links ms=0.068 peak_kb=8264 checksum=0
```

No ASan report. (Lines 5 and 6 are part 2, AN-65.)

**6. Still refused.**
- `child.parent = Some(this)` with `parent: Option[Node]` (the Python spelling, a strong field), E3013:
  "`child.parent = Some(this)` would make `parent` a second owner of this node, and `this` is not yours
  to hand over; a link back to the node whose `children` holds this one is written
  `parent: weak Node set by children`, and `push` sets it".
- `leaf.parent = root` with `set by`, E2109: "`parent` is set by `children`; to give `leaf` a parent,
  write `root.children.push(leaf)`".
- `parent: Node set by children` without `weak` (no code in `RUN4/rules.md` section 4; see "Notes for
  the lead" 4): "`set by` keeps a link back to the node that holds this one, which must not own it;
  write `parent: weak Node set by children`".
- Lending a `set by` holder as a changeable list (`shuffle(ref mut xs: Array[Node])` called as
  `shuffle(ref mut p.children)`), E2109: "`children` keeps every child's `parent` in step, so it is
  changed only by its own methods; pass it read-only, or write the change with `push`, `insert`,
  `remove`".
- A cycle of `shared` owners (`a: shared Node` holding `b` holding `a`) is not refused (it is a leak,
  not a memory error, as in Rust); AN-64's warning names it.

**7. Cost.** `set by`: an insert or a remove writes the link (one store); a read is a plain load and a
test for no link, with no header test and no count (R11). The phase before measured the same mechanism
on the tree workload, the link's own work, at 1.025 and 1.149 of plain C laid out the same way, and safe
Rust's index arena at 1.519 and 1.357 (`graphs-callbacks-attack2/bench/runs/w2_a.txt`, `w2_b.txt`);
run 3's W2, the parent walk with no header test, at 1.176x of plain C against safe Rust's 1.603x
(`RUN3/26-timer.md` lines 20 and 22). A graph with edges kept both ways: 0.969 and 1.081 of plain C
keeping the same reverse lists (`runs/graph_a.txt`, `graph_b.txt`). A plain `weak` link: one add and one
subtract of the weak count when it is made and when it ends, and one test of the target's header per
read, as Rust's `Weak`, which on churn costs 0.919 and 0.935 of plain C (`runs/weak_g.txt`,
`weak_h.txt`). The program writes the `remove`, so no hidden search.

**8. Kind:** a new mechanism (`set by`), with `weak` as a readable spelling of Rust's `Weak`.

---

## AN-64: L3001 tells the programmer to make the owning edge weak

**1. Intent:** follow the warning to fix a tree's cycle.

**2. OLD.** `old/an63_tree_python.em`, `old/an63_tree_weak.em`, `old/an62_walk_callback.em`: all three
get the same advice, pasted under AN-63 and AN-62: `help: if \`Node.children\` is a back-reference, make
it hold a \`Weak\` handle`. `children` is the owning edge; making it weak would free every child. In the
second and third files `parent` is already `Weak`, and the warning is still given.

**3. NEW.** The NEW tree of AN-63 gets no warning: a list whose elements have one owner cannot form a
cycle (handing a node into its own subtree would hand over a value while a view into it lives, which is
refused when compiling). The warning remains only for a cycle through `shared` fields, and names the
link to make `weak`:

```ember
class Person:
    name: String
    friends: Array[shared Person] = []
    best: Option[shared Person] = None
```

```text
warning[L3001]: `Person.best` and `Person.friends` can hold each other: a cycle of `shared` owners is never freed
 --> app.em:4:5
  |
4 |     best: Option[shared Person] = None
  |     ^^^^ this link points at a person another field already owns
  |
  = help: a link that should not keep its person alive is `weak`: `best: weak Person`
```

The field named is the single-valued one when the cycle has one (a back link), otherwise the field the
cycle closes on; a parent-child pair in the Python spelling is the error of AN-63, which names
`parent: weak Node set by children`. (Unchanged from run 3: the code has no method.)

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `name` | none (default: the person's own String) | `changes name`, or `ref mut this` |
| `friends` | `shared` on its elements: one of several owners of each person | `changes friends`, or `ref mut this` |
| `best` | `shared`: one of several owners of that person | `changes best`, or `ref mut this` |

**5. Means exactly.** The NEW tree is AN-63's C (`c/an63_links.c`): no ASan report, and its line
`4 leaf parent=None` (pasted under AN-63) shows the root's end clearing its child's link before
releasing it. (ASan on Windows has no leak checker, so "every node is freed" is read from the code's
drops, not from a leak report.) The warning itself is a message; the program it is given for is a leak,
not something to translate.

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
    target: weak Player = None

fn main():
    player = Player()
    cam = Camera()
    cam.target = weak player
    if p = cam.target:                  # the player still lives: call it `p`
        println(p.health)
    else:
        println("gone")
```

Changed from run 3: `if p = cam.target:` (was `if Some(p) = cam.target:`), the opening form (R13).

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `target` | `weak`: a link that does not keep the player alive | `changes target`, or `ref mut this` (re-pointing) |
| `player` | none (default: `main`'s own Player) | its owner may |
| `cam` | none (default: `main`'s own Camera) | its owner may |
| `weak player` | `weak`: a link, not a hand-over; `player` stays `main`'s | no |
| `p` | none (default for an opened name: a read-only view of the player, kept alive until its last use) | no |

**5. Means exactly.** `means/m65_weak_read.em` (`if Some(p) = cam.target.upgrade():`, today's spelling):

```text
=== m65_weak_read.em
exit=0
```

In C, `c/an63_links.c` part 2: a read is `(w && !w->dead) ? w : NULL`; lines `5 100` and `6 gone` of
its output (pasted under AN-63) are the read before and after the player ends, clean under ASan.

**6. Still refused.**
- `cam.target.upgrade()`, E1010: "a `weak` link is read directly: `if p = cam.target:`".
- `Weak[Player]`, `Weak(player)`, E1010: "write `weak Player`", "write `weak player`".
- `if Some(p) = cam.target:`, E0115: "an `Option` or a `weak` link is opened by naming what it holds:
  write `if p = cam.target:`".
- `cam.target.health`, E2020 (today's code for a field the type does not have): "`cam.target` may be
  gone; write `if p = cam.target:`".
- `cam.target = player`, E2020 (today's code for a type that does not match): "`=` alone would hand
  `player` over; write `cam.target = weak player`".

**7. Cost.** One add of the player's weak count when `cam.target = weak player` makes the link and one
subtract when it ends (R11); one test of the target's header per read. Nothing keeps `p` alive: no line
in its life can end a Player (R10 point 1). Rust's `upgrade` always makes a count pair.

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

**3. NEW.** (Unchanged from run 3: no method, no call to a `ref mut` parameter.)

```ember
struct Scene:
    shapes: Array[owned dyn Shape]          # the list is each shape's one owner
    shown: Array[shared dyn Shape]          # shapes also held elsewhere

fn draw(s: dyn Shape):                      # a read-only view of any shape
    ...

fn main():
    scene = Scene(shapes=[Circle(r=1.0), Square(side=2.0)], shown=[])
    for s in scene.shapes:
        draw(s)
```

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `shapes` | `owned` on its elements: each shape has this one owner | `changes shapes`, or `ref mut this` |
| `shown` | `shared` on its elements: one of several owners | `changes shown`, or `ref mut this` |
| `s` in `draw` | none (default: a read-only view, a parameter) | no |
| `scene` | none (default: `main`'s own Scene) | its owner may |
| `Circle(...)`, `Square(...)` in the list | none (default: new values handed over to the list, row 9) | their owner may |
| `s` in the loop | none (default: a read-only view of an element) | no |

**5. Means exactly.** `owned dyn Shape` is today's `Box[dyn Shape]` (`means/m66_owned_dyn.em`):

```text
=== m66_owned_dyn.em
exit=0
```

`shared dyn Shape` over class objects is today's bare `Shape` handle (`old/an66_bare_interface_class.em`,
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
- `shapes: Array[Shape]`, E2173: "say who owns the shapes: `Array[owned dyn Shape]` (the list is their
  only owner) or `Array[shared dyn Shape]` (other names own them too)".
- `b = a` for an `owned dyn Shape` hands it over (row 9); a later use of `a` is E3040: "`a` was handed
  over to `b` on line 2 and is used on line 4; to keep both, write `b = shared a` (two owners of one
  shape)" (`copied a` only where the shape's type implements `Clone`).
- `Box[dyn Shape]`, E1010: "write `owned dyn Shape`".

**7. Cost.** `owned dyn`: one heap block per value and a method-table pointer beside it (Rust's
`Box<dyn>`); `shared dyn`: the same plus a count (Rust's `Rc<dyn>`); a call is one indirect call.

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
    pools: Map[u128, owned dyn AnyPool]

    fn get_mut[T](k: u128, i: int) changes pools -> ref mut T:
        if ref mut pool = cast this.pools[k] if Pool[T]:     # the pool at `k` is a `Pool[T]`: call it `pool`
            return ref mut pool.dense[i]
        panic("no pool of this type at this key")

interface Inspect:
    fn inspect[V: Visitor](ref mut v: V)

fn walk(xs: Array[owned dyn Inspect]) -> int:
    s = Sum(total=0)
    for x in xs:
        x.inspect(ref mut s)        # through `dyn`: the entry made with V = dyn Visitor
    return s.total
```

Changed from run 3: `this` for `self`, and no object in the parameter lists (R5); the opening form
`if ref mut pool = cast this.pools[k] if Pool[T]:` (was `if Some(ref mut pool) = ...`, R13, R14); the
call `x.inspect(ref mut s)` (R4: `inspect`'s parameter is `ref mut v`).

`cast <place> if D` on an `owned dyn` (or `shared dyn`) place tests the value's type and gives a view of
it as a `D`: `ref mut pool` names a changeable view because the place is changeable (`changes pools`);
plain `pool` would be read-only (R14). A generic method whose type parameter appears only as the type of
a `ref` or `ref mut` parameter, with a `dyn`-compatible bound, gets one method-table entry, the generic
body compiled with `V = dyn Visitor` (R14).

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `pools` | `owned` on its values: each pool has the map as its one owner | `changes pools`, or `ref mut this` |
| `this` in `get_mut` | `this`: the Reg `get_mut` was called on | `changes pools` |
| `k`, `i` | none (default: read-only numbers) | no |
| `pool` | `ref mut` on the opened name: a view of the registry's pool that may change it | yes |
| result `-> ref mut T` | `ref mut`: a view into the registry's element | yes |
| `this` in `inspect` | `this` | no: the first line says neither `changes` nor `ref mut this` |
| `v` | `ref mut`: the caller's visitor | yes |
| `xs` | `owned` on its elements (one owner, the list); the parameter itself none (a read-only view) | no |
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
  `owned dyn AnyPool`" where that is the cause.
- `if Some(ref mut pool) = cast this.pools[k] if Pool[T]:`, E0115: "an `Option` or a `weak` link is
  opened by naming what it holds: write `if ref mut pool = cast this.pools[k] if Pool[T]:`".
- `as?` / `as!`, E1010 (`RUN4/rules.md` section 3; today E0100 with the line, pasted in part 2): "a
  downcast is written `cast x if D`" (or `cast x as D`).

**7. Cost.** A `cast`: one load of the method table's type id and one compare; safe Rust's
`downcast_mut` was measured slower (1.710x and 1.606x against the pool table's 1.208x and 1.084x of plain
C, `phase3b-references.md` 3.4 as quoted in `phase3b-graphs-callbacks.md` 6.3); `RUN4/rules.md` section 2
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

**3. NEW.** The programmer's own line:

```ember
    for s in shapes:
        if c = cast s if Circle:          # `s` is a Circle: call it `c`
            t += c.r
```

**Meaning changed.** Run 3 kept today's refusal of this line and asked for `if Some(c) = cast s if
Circle:`. The owner's verdict of 2026-10-25 ("`if Some(ref mut me) = ui:` needs a major rework") led to
R13's opening form, which is this line: `if name = e:` runs the block when `e` (here the cast) holds
something and names it. The OLD program is now accepted with the meaning its programmer wrote, and
run 3's NEW spelling is refused (part 6).

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

Today's compiler refuses the NEW line itself (E2036, part 2).

**6. Still refused.**
- `if Some(c) = cast s if Circle:`, E0115: "an `Option` or a `weak` link is opened by naming what it
  holds: write `if c = cast s if Circle:`".
- `if c = s:` where `s` always holds a value, E2036: "`s` always holds a value, so this `if` would
  always run; to compare write `if c == s:`, to name it write `c = s` on the line before".
- `if s = cast s if Circle:`, E2106: "`s` already names a value (line 1); to compare write
  `if s == cast s if Circle:`, to open the cast use a new name: `if c = cast s if Circle:`".

**7. Cost.** None beyond the one type-id compare of the cast (AN-67).

**8. Kind:** a readable spelling of Rust's guarantee (`if let Some(c) = s.downcast_ref::<Circle>()`),
written the way the programmer first wrote it; run 3's refusal was an annoyance and is removed.

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

**3. NEW.** (Unchanged from run 3.)

```ember
fn sum[T](xs: Span[T]) -> T:                 # not `pub`: its bound is worked out from the body
    acc = copied xs[0]
    for x in xs[1..]:
        acc = acc + x
    return acc

pub fn total[T: Add[T, Output = T] + Copy](xs: Span[T]) -> T:    # `pub`: the bound is written
    ...

extend Array[Vec3] implements Neg:           # accepted in a program (a package nothing depends on)
    ...
```

Rules (R14, R17): a function that is not `pub` gets its bounds from its body (every operation on `T`
maps to the one interface in scope that provides it; recorded, printed by `ember inspect --bounds`); a
`pub` function writes its bounds, because they are its promise to other packages, and the error gives
the exact line; a package that nothing can depend on (a program or a test) may implement any interface
for any type (two implementations of one interface for one type anywhere in the program stay refused).
Inferred bounds stop at `pub`, so nothing another package relies on is decided by the compiler from a
body it cannot see.

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `xs` | none (default: a read-only view, a parameter) | no |
| `acc` | `copied`: the function's own copy of the first element | its owner may (`=` gives it a new value, R2) |
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
- A non-`pub` body calling a method no interface in scope declares, E2072 (today's code for a method a
  type does not have): "`T` has no `weight`: no interface in scope declares `fn weight`; declare one
  (`interface Weighed: fn weight() -> int`) and the bound follows" (`this` for run 3's `self` in the
  interface line; a duck-typing extension is not taken: it would let a typo surface at a call far
  away).

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
    virtual fn on_update(dt: float) ref mut this:
        pass

class SwingDoor(Door):
    override fn on_update(dt: float) changes angle:
        this.angle = min(this.angle + 90.0 * dt, 90.0)
```

Changed from run 3: the virtual says `ref mut this` (was `mut self`): its first line is what every
override must keep to, and an override may change the door (R5; the spec's own `Door`, spec 1704, sets
the angle in its override). The override says `changes angle` (was `mut self`), the one field its body
changes: no more than the virtual's line allows (R5, E2113), and exactly what its body does (E2105).

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `angle` | none (default: the door's own `float`) | `changes angle`, or `ref mut this` |
| `this` in `Door.on_update` | `this`: the door the method was called on | `ref mut this`: any field (for its overrides) |
| `this` in `SwingDoor.on_update` | `this` | `changes angle`: that field only |
| `dt` (both) | none (default: a read-only number) | no |

**5. Means exactly.** `means/m70_open_class.em`:

```text
=== m70_open_class.em
exit=0
```

and the override, `target/redesign3/19-fixer/m70_virtual_changes.em` (today a class method's plain
`self` may write fields, `[CLS-7]`; `SwingDoor(Door)` with `override fn on_update(self, dt: float):
self.angle = min(self.angle + 90.0 * dt, 90.0)`):

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
  `virtual fn on_update(dt: float) ref mut this:` in `Door`".

**7. Cost.** None.

**8. Kind:** an understandable limitation kept (only an `open` class has subclasses), now refused with the
line to write instead of accepted with a warning.

---

## Words needed

Words an answer would need that `RUN4/vocabulary.md` has no row for. None is used in the NEW code above.

| word | why |
|---|---|
| `count` (the iterator method, `xs.iter().count()`) | AN-55's OLD line counts what the iterator gives (`_n = xs.iter().inspect(...).count()`). Section 3.8 lists `iter`, `collect`, `inspect`, `for_each`, `sum`, `min`, `max`, `next`, not `count`, so the NEW line is written with `for_each`, which keeps the intent (record every value in `seen`) and drops the unused `_n`. A row for `count` (the same as Rust's `Iterator::count`) would let the line stay as the programmer wrote it |

## For the owner

1. **AN-62: a node the callback ended above the walk is still visited once.** The run-4 rules read a
   `set by` link (`cur.parent`) with no header test (R11, which makes the parent walk 1.176x of plain C
   instead of 1.579x, `RUN3/26-timer.md` lines 21-22), and inside a callback loop they set an ended
   node aside instead of freeing it (R10 point 2). Together: if the callback removes `p`'s parent `q`
   from the tree, `q` is set aside with its `children` untouched, so `p.parent` still names `q` and the
   walk visits `q` once more (memory stays safe; `q`'s block is kept until the loop ends). Python and run
   3 (which tested every hop) stop before `q`. A fix that keeps the speed: when a node is set aside, the
   `set by` links of the nodes it holds are cleared at once (the same stores its end makes anyway, moved
   earlier), so a `set by` read never gives a set-aside node and still needs no test; nothing is paid per
   step. This needs a sentence in R10 point 2, which is the rule writer's. Should the walk stop before
   `q`, as in Python (the fix), or is visiting `q` acceptable?

## Notes for the lead and the rule writer

Points where the run-4 rules or vocabulary did not settle what an answer needed; what this area wrote
meanwhile.

1. **`b = a` where `a` is a view name** (an opened, loop or parameter name). R1 says `b = a` with a whole
   name of a type that is not `Copy` hands `a` over; R2 lists parts and calls as what makes a read-only
   view, not names that are already views. `RUN4/changes.md` section 11 asks AN-62 for `cur = n` and
   `cur = p` as views. AN-62 writes `cur = ref n` and `cur = ref p`, which R2 states ("`ref` says the
   default ... may be written on any declared name") and which passes the owner's test with a word; a
   plain `cur = n` reads as row 9's hand-over.
2. **R8 (f) against R12 for AN-60's second way out.** `for ref mut h in this.handlers: h(this)` gives
   `this` (which holds `h`'s element) to a call while `ref mut h` is used; R8 (f) read alone refuses that
   (E3021). R12 ("the holder does not become an argument of the call"; the call copies the two words
   out) is read here as deciding it, as `RUN4/changes.md` section 11 does. One sentence in R8 (f) would
   say so.
3. **The `get_mut` row** (`RUN4/vocabulary.md`) gives E3022 for a plain `h = this.handlers.get_mut(i).
   unwrap()` then `h(ref mut this)`. By R2 a plain `h` is a read-only view, so the first refusal is R12's
   E3023; E3022 (R8 (f)) is the refusal of `h = ref mut ...`. AN-61 gives both.
4. **Refusals with no code in `RUN4/rules.md` section 4**: `set by` without `weak` (AN-63); a lambda
   parameter's mode that does not match the expected callable type (AN-56; AN-56 writes today's E2020);
   a field read on a `weak` link that was not opened, and `=` of an object into a `weak` field (AN-65;
   today's E2020); a non-`pub` generic body calling a method no interface declares (AN-69; today's
   E2072).
5. **A virtual's first line and E2105.** R5 checks every clause against the body both ways, so
   `virtual fn on_update(dt: float) ref mut this: pass` (the line E2113's own message asks for) names a
   change its body does not make. AN-70 reads the virtual's first line as the promise for its overrides,
   not a description of its own body; R5 does not say so.
