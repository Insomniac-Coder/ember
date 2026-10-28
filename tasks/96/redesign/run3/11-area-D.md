# 11 area D: linked objects, callbacks, interface types (run 3, phase 2)

Agent 11 of run 3. Duty: `inputs/duties.md`, "08-12 Area designers A-E", area D. Inputs read in full:
`inputs/brief.md`, `inputs/JOB.md`, `RUN3/planned-words.md`, `RUN3/annoyances.md`, and the parts of
`RUN3/vocabulary.md` this area uses (sections 1, 3.1-3.6, 4, 6.1); last run's
`phase3b-graphs-callbacks.md` and `phase3b-graphs-callbacks-attack.md` (repaired here), and
`phase3b-combined.md` sections 0, 3.4, 4.3, 4.4, 4.7-4.9 (what last run's combiner decided for this area).

Power: `PowerLineStatus` printed `Online` at the start, and again before the one timing batch.

Annoyances answered: AN-53 .. AN-70 (18). Agreed row kept: 6, exactly as in `inputs/JOB.md`.

## 0. The short version

| id | annoyance | answer in one line | kind |
|---|---|---|---|
| AN-53 | a closure changes a variable of the function that made it | row 6: `b.on_click(fn(): score += 1)` changes `game`'s real `score`; the compiler keeps the variable where the function value can reach it (its address when the function value provably ends first, a counted box otherwise), which never changes what the program does | smarter compile-time |
| AN-54 | `owned fn` moves a `String` and copies an `int` | `owned fn` is removed; what a function owns or copies is named on each captured name: `fn() with owned name, copied score:` | readable spelling |
| AN-55 | the callee decides whether a capture is moved (`[CLO-15]`) | a capture not named is the real variable, whatever the callee; a thread that may outlive it is refused with the two lines to write (`with owned xs`, or `thread.scope()`) | readable spelling |
| AN-56 | a lambda parameter takes `owned` from the expected type | the mode is what is written on the parameter: `fn(owned s) => names.push(s)`; a plain `s` only reads | readable spelling |
| AN-57 | E3026 inside the callee; `Callable0` | every refusal about a function value is reported at the line that wrote it, in the program's names (`fn()`, never `Callable0`) | understandable limitation kept |
| AN-58 | a closure in a local list that ends first is E3026 | accepted: the list provably ends first, so the function value holds the variable's address (cost: none) | smarter compile-time |
| AN-59 | a closure that gives a capture away gets E3013 with ten options | `fn() with owned s: consume(s)` is a run-once function (`once fn()`); passing it where it may run twice gets "declare the parameter `once fn()`" | readable spelling |
| AN-60 | a callback receiving its holder; a handler that removes itself | handlers are `fn(ref mut Ui)`; dispatch by position (`self.handlers[i](self)`) runs added handlers and lets one remove itself, as Python does; the `for` form stays refused (agreed limitation) with that line as its advice | smarter compile-time |
| AN-61 | unsound today: dispatch through `get_mut(i)` writes freed memory | refused at compile time (a view into the list may not live while the handler may change the list); the position call copies two words out and keeps no pointer into the list | smarter compile-time |
| AN-62 | a walk calling an unknown stored callback paid a count pair per step | a **deferred-drop region** around the loop: an object of the walked class that dies during the loop is set aside whole and dropped when the loop ends; the step's code is the unprotected step's | new mechanism |
| AN-63 | trees with parent pointers, graphs, cycles | `parent: weak Node set by children` (the language sets and clears it; reading is a plain load); `target: weak Player` for other links (reads as `Option`); a one-owner `children` list cannot form a cycle | new mechanism |
| AN-64 | L3001 tells the programmer to make `children` weak | no cycle warning for one-owner links (they cannot form a cycle); the Python spelling gets an error naming the back link: "write `parent: weak Node set by children`" | smarter compile-time |
| AN-65 | `upgrade` | no method: `if Some(p) = cam.target:` | readable spelling |
| AN-66 | a bare interface type is a counted shared handle with no word | the type says who owns it: `owned dyn Shape` (one owner) or `shared dyn Shape` (several); bare `Shape` is refused with both lines; a parameter `s: dyn Shape` is a read-only view | readable spelling |
| AN-67 | downcasting a `Box[dyn]` pool; generic methods through `dyn`; `as?`/`as!` | `cast` works on an `owned dyn` place and gives a view (`if Some(ref mut pool) = cast self.pools[k] if Pool[T]:`); a generic method whose type parameter is only a `ref`/`ref mut` parameter gets a `dyn` entry; the spec's `as?`/`as!` become `cast` | smarter compile-time |
| AN-68 | `if c = cast s if Circle:` gets a help that is not what was meant | the help gives the line that works today: `if Some(c) = cast s if Circle:` | understandable limitation kept |
| AN-69 | generics friction | bounds of a function that is not `pub` are worked out from its body; a `pub` one writes them and the error gives the exact line; a program nobody depends on may implement any interface for any type; overlapping implementations stay refused with a message | smarter compile-time |
| AN-70 | `virtual` in a final class is a warning | refused: "write `open class Door:`, or remove `virtual`" | understandable limitation kept |

**New words: none.** Every answer uses words of `RUN3/vocabulary.md` and `RUN3/planned-words.md`
(`weak`, `set by`, the capture list `with ...`, `owned`, `shared`, `copied`, `ref mut`, `once`, `dyn`,
`cast`, `open`, `virtual`, `changes`). Section 5 lists the positions used.

**What runs while the program runs, compared with C** (section 3 has the sources): a capture is an
address (as C) or a counted box (one allocation per variable instance, as C's heap callback environment;
measured nil last run); a stored function value is two words `{code, state}`; a handler that uses its
own state after a call that may remove it holds one count across that call (rare path); a `weak` read is
one test of the target's header; a `set by` read is a plain load and its upkeep is one store per
insert/remove; a walk that calls unknown code pays one store on entering and leaving the loop instead of
a count pair per step; `owned dyn`/`shared dyn` cost what Rust's `Box<dyn>`/`Rc<dyn>` cost.

## 1. Terms used here

- **function value**: a function or lambda kept in a variable, list or field (a callback, a handler):
  two words, its code and a pointer to its **state** (what it captured).
- **capture**: a name a lambda uses from the function that made it. **The real variable** (row 6): the
  capture is that very variable, not a copy; a write in the lambda is seen by the function, and back.
- **capture list**: what follows `with` after a lambda's parameters (`fn() with copied score:`).
- **box**: a small counted heap block holding one captured variable, shared by the function that owns
  the variable and every function value that captures it, freed with the last of them (Python's "cell").
- **count pair**: one increment and one decrement of an object's count around something.
- **view**: a name for something owned elsewhere that may not outlive it (`ref T`, a read-only
  parameter, a pattern name).
- **holder**: the object whose field holds another (a parent for its children).
- **deferred-drop region** (new mechanism, AN-62): a stretch of code during which an object whose count
  reaches zero is set aside whole (its fields and its memory untouched) and dropped when the stretch ends.
- **unknown call**: a call through a function value, a `dyn` value or another package's function whose
  effects are not known, which may therefore change anything a program may change through it.
- **AddressSanitizer** (ASan): a clang option that stops a program with a report the moment it reads or
  writes memory it must not.
- **today**: `target/redesign3/bin/ember.exe check <file>`. Every "today" claim has its output pasted.

## 2. Evidence

All under `C:\Users\ism19\Code\ember\target\redesign3\11-area-D\`:

- `old/*.em`: the OLD programs. 16 are last run's probes, copied unchanged from
  `target/redesign2/graphs-callbacks/probes/` (file names changed to the annoyance ids), 3 are the
  readers' and builder's (`04-reader-4/t13_o5.em`, `t10_w2111.em`, `05-builder/d1_cast_forms.em`), the
  rest are new. `old/check_output.txt` is made by `run_checks.sh old` (every file, `ember check`, exit
  code).
- `means/*.em`: every "means exactly" translation into today's Ember; `means/check_output.txt` the same
  way.
- `c/`: the C translations, each with its output file:
  `an53_captures.c` (`an53_captures_output.txt`), `an61_dispatch.c` (`an61_dispatch_output.txt`, three
  modes), `an62_defer_asan.c` (`an62_defer_asan_output.txt`, three modes), `an63_links.c`
  (`an63_links_output.txt`). ASan build: `clang -O1 -g -fsanitize=address -I target/redesign/bench`,
  run with the ASan DLL directory first on `PATH`, `ASAN_OPTIONS=abort_on_error=0`.
- `c/an62_walk_bench.c` (last run's `graphs-callbacks-attack2/bench/w2v_backlink.c` with one variant
  added, `-DDEFER`), `c/an62_walk_plain.c` (`w2v_plain.c`, unchanged), `c/ember_model.h` (unchanged);
  built `clang -O2 -DSLAB [-DARRAY24|-DRETAIN|-DDEFER]` into `c/bin/`; assembly `c/bin/walk_*.s`;
  timing `c/walk_runs.txt` by `target/redesign2/bench/run.ps1 -Rounds 5`; medians `c/walk_summary.txt`.

No Ember program was run. No compiler, spec or standard-library file was changed.

---

## 3. The rules of area D in one place

The answers in section 4 refer to these by number.

**D1. A capture not named is the real variable** (row 6). `fn(): score += 1` reads and writes the
`score` of the function that wrote the lambda, wherever the function value goes. Its place is chosen
when compiling and never changes what the program does:
- **an address** when the compiler proves the function value ends before the variable does: it is kept
  only in locals, local lists, or objects that never leave the function, and every method it is handed
  to says what it changes (row 3's `changes handlers`, or `mut self`, on an object that itself stays in
  the function);
- **a box** otherwise (the function value is returned, stored in something that outlives the function,
  or handed to code whose first line does not limit where it goes).
`ember inspect --alloc` lists each box, as it lists every allocation today (`[CLO-10]`).

**D2. Named captures** (planned-words section 4): `with copied x` (the function's own copy), `with
owned x` (handed over; refused for a `Copy` type with "write `with copied x`"), `with shared x`, `with
thread_shared x`, `with weak x` (each with its meaning before a value). Each name carries its own word.

**D3. What a call changes.** Calling a function value changes what its body changes: the real variables
it captured (through their address or box) and its own named state (`copied`, `owned`). A function
value whose body changes **its own** state is called through a name that may change it (the owner, or a
`ref mut` parameter); one that changes only real variables may be called through any name. A variable
that a living function value changes is treated as changed by every call that may run that function
value: no view into it is kept across such a call (the attack on last run's pointer captures,
`phase3b-graphs-callbacks-attack.md` 5.2, closed at compile time).

**D4. A stored function value is an object** (last run's A32 answer, kept): its state is in its own
block, so the two words `{code, state}` may move with the list. A call through a list element or field
copies the two words out and keeps no pointer into the list. The body of a function value whose state
may be dropped by a call it makes (it removes itself), and which uses that state during or after the
call, holds one count on its state across that call (last run's two attack fixes, 6.2 and 6.3, kept).

**D5. Unknown calls and the objects a loop holds.** A handle read from a place and used after an unknown
call is protected against that call ending its object only if the call could end it: if the object's
class is the target of a `weak` link, or is held `shared` or `thread_shared`, anywhere the call can
reach. A one-owner object lent read-only cannot be ended by anyone during the loan (the borrow rule).
Where protection is needed: a **deferred-drop region** around the loop (AN-62) when the class has no
`drop` method of its own; a count pair across the call otherwise.

**D6. `weak`** (planned-words 3.2): a link that does not keep its target alive; reading it gives
`Option`. Reading tests the target's header (last run's combined choice, `phase3b-combined.md` 4.3:
Rust's mechanism; the registration alternative needed three fixes and costs 1.09x to 1.27x on churn).
A name bound from the `Some` is kept alive until its last use under D5's rule. `Some(ref mut p)` binds a
view that may change the target; the target then counts as "reachable by an unknown call" (D5) and
under area C's rule for objects with several ways in (AN-48). A `weak` link to a `thread_shared` value
uses the atomic count its header already says it has (no word).

**D7. `set by`** (planned-words 3.3): `parent: weak Node set by children`. Rules, from last run's back
links as saved by both attacks, rewritten for one-owner lists:
1. The named field (`children`) belongs to the target's type and holds this type with one owner
   (`Array[Node]`, `Option[Node]`) for a single link, or weak elements for a list link
   (`into: Array[weak City] set by roads`).
2. Putting an object into the holder field sets its link; taking it out clears it. One membership change,
   in this order: unlink the old value, link the new one, release the old one (attack 3.2's fix: a field
   assigned the value it already holds).
3. A holder that ends clears its elements' links before releasing them.
4. The link is never assigned by the program; the holder field is changed only through its own
   operations (`push`, `insert`, `remove`, `pop`, `clear`, assigning the whole field), never lent as a
   changeable list or span to a function that does not keep the links (last run's rule 6).
5. Reading the link is a plain load; no test: the holder cannot end while it holds this object without
   clearing the link first (rule 3).

**D8. Interface values say who owns them**: `owned dyn I` (one owner; replaces `Box[dyn I]`), `shared
dyn I` (several owners, counted), `dyn I` as a parameter's type (a read-only view; `ref mut v: dyn I`
for a changeable one), `some I` in a return type (as today), `[T: I]` for a generic. A bare `I` as a
type is refused.

---

## 4. The answers

### AN-53 A closure that changes a variable of the function that made it (A12; agreed row 6)

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

    fn on_click(self, owned f: fn()) changes handlers:
        self.handlers.push(f)

    fn click(self):
        for h in self.handlers:
            h()

fn game() -> int:
    score = 0
    b = Button()
    b.on_click(fn(): score += 1)       # row 6: game's real `score`
    b.click()
    return score                       # 1
```

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `handlers` (field) | none (default: the Button's own list; each element its own function value) | `changes handlers` on `on_click` |
| `self` in `on_click` | `self`: the Button the method was called on | `changes handlers` |
| `f` | `owned`: handed over to `on_click` | its new owner may; it is handed on to the list |
| `self` in `click` | `self` | no (plain `self`, read-only) |
| `h` | none (default: a read-only view of an element of `handlers`; calling it changes only what its body changes, D3) | no |
| `score` in `game` | none (default: `game`'s own variable, made by `=`) | its owner `game` may, by assignment |
| `score` in the lambda | none (agreed row 6: the real `score` of `game`) | yes, as row 6 says (`score += 1`) |
| `b` | none (default: `game`'s own Button, made by `=`) | through methods that say `changes` |

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

That is D1's box. Here `b` never leaves `game` and `on_click` says it changes only `handlers`, so the
compiler proves the handler ends with `b`, before `score`, and passes `score`'s address instead: the C is
`c/an53_captures.c` case 1, `on_click(&b, (Fn0){bump, &score})`, run under ASan:

```text
game=1
counter=2
an53_captures ms=0.040 peak_kb=8176 checksum=0
```

(`counter=2` is case 2, a counter factory whose variable lives in a box because the function value is
returned: D1's other place.) No ASan report.

**6. Still refused.**
- A capture of someone else's place in a function value that may outlive it (AN-57's example):
  "the click handler changes `total`, which belongs to `register`'s caller, and `b` keeps the handler
  after `register` returns; keep the total where the handler can own a share of it:
  `count = shared Counter()` and `fn() with shared count: count.n += 1`".
- A capture used by a thread that may outlive it (AN-55's messages).
- A loop that walks a variable a living function value changes, around a call that may run it (D3):
  "`hs[0]()` may run the function made on line 3, which changes `xs`, while this loop walks `xs`;
  collect the changes and make them after the loop".

**7. Cost.** An address: none (the C passes `&score`). A box: one allocation per variable instance and
one load per use; last run measured the callback workload at 1.018 and 1.052 of plain C with the shared
total boxed, against 1.045 and 1.037 with it by address
(`target/redesign2/graphs-callbacks-attack2/bench/runs/w4_a.txt`, `w4_b.txt`, summarised in
`phase3b-graphs-callbacks-attack.md` 6.4), and a boxed variable written in a hot loop at 0.990 and 0.998
of a C local (`runs/micro_a.txt`, `micro_b.txt`, same report 5.3). Rust's spelling of the same program
is `Rc<Cell<i32>>`, the same box.

**8. Kind:** a smarter compile-time solution (Rust refuses the program; here the compiler proves the
cheaper place and otherwise uses the box Rust makes the programmer write).

---

### AN-54 `owned fn` moves a `String` capture and copies an `int` one (agreed row 6)

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
    name = "ann".to_string()
    score = 0
    cb = fn() with owned name, copied score:
        score += 1                    # its own copy
        println(name)                 # its own String
    run(cb)
    println(score)                    # 0, as `copied` says
```

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `f` | `ref mut`: `main`'s function value, lent | yes (`ref mut`): `cb` changes its own copy when called (D3) |
| `name` in `main` | none (default: `main`'s own String, made by `=`) | its owner may |
| `score` in `main` | none (default: `main`'s own `int`) | its owner may |
| `cb` | none (default: `main`'s own function value) | its owner may (it is lent `ref mut` to `run`) |
| `name` in the lambda | `owned`: handed over to the function | its new owner may |
| `score` in the lambda | `copied`: the function's own copy | yes, its own copy (`score += 1`) |

**5. Means exactly.** Today's `owned fn` does exactly this (moves the `String`, copies the `int`); NEW
only writes it on each name (`means/m54_named_captures.em`):

```text
=== m54_named_captures.em
exit=0
```

**6. Still refused.**
- `owned fn(): ...`: "name what the function takes or copies on each name: `fn() with owned name,
  copied score:`; a name not listed is the real variable".
- `fn() with owned score:` for an `int`: "an `int` is copied, not handed over; write `fn() with copied
  score:`" (vocabulary C-2).
- `println(name)` after `with owned name`: "`name` was handed to the function on line 7; to keep
  using it here, give the function a copy, `with copied name`, or share it, `with shared name`".
- `fn() with copied name, score:`: "say what the function does with `score`: `with copied name,
  copied score`, or leave `score` out to use the real `score`" (planned-words section 4).
- `run(cb)` with `fn run(f: fn()):`: "`cb` changes its own copy of `score` each time it runs, and
  `run` takes it read-only; write `fn run(ref mut f: fn()):`" (D3).

**7. Cost.** None beyond C: the `String`'s three words move into the function's state, the `int` is
copied, as a Rust `move` closure.

**8. Kind:** a readable spelling of Rust's guarantee (`move`, written per name).

---

### AN-55 The compiler decides a capture is moved from which function receives the lambda (`[CLO-15]`)

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
    _n = xs.iter().inspect(fn(v) => seen.push(v)).count()     # the real `seen`
    h = thread.spawn(fn() with owned xs: work(xs))           # `xs` handed to the thread
    println(seen.len())
    println(h.join())
```

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `xs` in `main` | none (default: `main`'s own list) | its owner may |
| `seen` in `main` | none (default: `main`'s own list) | its owner may |
| `v` | none (default: a read-only view of the element; an `int`, read) | no |
| `seen` in the lambda | none (row 6: the real `seen`) | yes (`push`), as row 6 says |
| `_n` | none (default: `main`'s own `int`, the result of `count()`) | its owner may |
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

**6. Still refused.** `thread.spawn(fn() => work(xs))`: "the thread may still run after `main` ends,
and `xs` belongs to `main`; hand it to the thread, `thread.spawn(fn() with owned xs: work(xs))`, give it
a copy, `with copied xs`, or start the thread inside `with scope = thread.scope():`, which waits for it".
A box (D1) is never used across threads: its count is a one-thread count (area E's rule; `with
thread_shared x` is the shared form).

**7. Cost.** None beyond C: the view is an address, the hand-over moves three words.

**8. Kind:** a readable spelling of Rust's guarantee (`move` and scoped threads), with no rule that
depends on the callee.

---

### AN-56 A lambda parameter without a mode takes `owned` from the expected callable type (`[TYP-23]`)

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
fn each_name(owned f: fn(owned String)):
    f("ann".to_string())

fn main():
    names: Array[String] = []
    each_name(fn(owned s) => names.push(s))       # takes `s` over, keeps it
    each_name(fn(s) => println(s.len()))          # only reads `s`
```

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

**6. Still refused.** `each_name(fn(s) => names.push(s))`: "`s` is only read here, so it cannot be kept
in `names`; to take it over write `fn(owned s) => names.push(s)`". `fn(owned s)` where the expected type
is `fn(String)` (a read-only parameter): "`each_name` lends the name and keeps it; to keep your own,
write `fn(s) => names.push(copied s)`".

**7. Cost.** None: the same code as today.

**8. Kind:** a readable spelling of Rust's guarantee (the parameter's mode is written, as on every other
parameter, vocabulary C-1).

---

### AN-57 E3026 is reported inside the callee; a stored lambda gets E2020 naming `Callable0`

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

**3. NEW.** The button program of AN-53 compiles (D1). What is still refused is reported at the lambda,
in the program's names:

```ember
fn register(ref mut b: Button, ref mut total: int):
    b.on_click(fn(): total += 1)
```

```text
error: the click handler changes `total`, which belongs to `register`'s caller
 --> app.em:2:16
  |
2 |     b.on_click(fn(): total += 1)
  |                ^^^^^^^^^^^^^^^^ written here, kept in `b`
  |
  = why: `b` keeps the handler after `register` returns, and `total` may have ended by then
  = help: keep the total where the handler can own a share of it:
          `count = shared Counter()` in the caller, and `b.on_click(fn() with shared count: count.n += 1)`
```

Callable types are printed as written: `fn()`, `fn(ref mut Ui)`, never `Callable0`.

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `b` | `ref mut`: the caller's Button | yes (`ref mut`) |
| `total` | `ref mut`: the caller's `int` | yes (`ref mut`) |
| `total` in the lambda | none (row 6: the real `total`, here the caller's place) | yes, as row 6 says; refused because it may outlive it |

**5. Means exactly.** Nothing compiles for the refused program, which is the point. The accepted form in
the help is AN-53's box (`means/m53_button_box.em`, `exit=0`, pasted under AN-53); its `shared Counter`
is today's class `Score` there.

**6. Still refused.** The program above, with the message above. A lambda given to a callee that keeps
it while the lambda captures a view (a `ref`, `ref mut`, pattern or `for` name): the same message shape,
at the lambda's line, naming the captured view and the holder that keeps the lambda.

**7. Cost.** None (compile time only).

**8. Kind:** an understandable limitation kept (a function value may not outlive a place it does not
own; Rust's E0373), with the message at the right line and in the program's names.

---

### AN-58 A closure stored in a local list that ends first is still E3026

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
    for h in hs:
        h()
    return score                       # 1
```

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `score` | none (default: `game`'s own `int`) | its owner may |
| `hs` | none (default: `game`'s own list; its elements its own function values) | its owner may |
| `score` in the lambda | none (row 6: the real `score`) | yes, as row 6 says |
| `h` | none (default: a read-only view of an element; calling it changes what its body changes, D3) | no |

**5. Means exactly.** `hs` is declared after `score` and never leaves `game`, so it ends first and the
lambda holds `score`'s address (D1): the C of `c/an53_captures.c` case 1 (the same `{code, &score}` pair,
there kept in a local `Button`), clean under ASan, output pasted under AN-53 (`game=1`).

**6. Still refused.** The loop of D3: `for x in xs: hs[0]()` where `hs[0]` changes `xs`: "`hs[0]()` may
run the function made on line 3, which changes `xs`, while this loop walks `xs`; collect the changes and
make them after the loop". (Last run's `attacks2.exe 11` showed the use after free this refusal
prevents.)

**7. Cost.** None (the C passes an address).

**8. Kind:** a smarter compile-time solution (the list's end is proved; Rust needs the same proof through
lifetimes, and accepts this program).

---

### AN-59 A closure that gives a capture away gets E3013 with ten options instead of "declare the parameter `once fn()`"

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
fn call_once(f: once fn()):
    f()

fn call_twice(f: fn()):
    f()
    f()

fn main():
    s = "a".to_string()
    call_once(fn() with owned s: consume(s))      # the function owns `s` and gives it away: once
    t = "b".to_string()
    call_twice(fn(): consume(copied t))           # each run hands over a new copy
```

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `f` in `call_once` | `once`: a function the call uses up | it is used up by the call |
| `f` in `call_twice` | none (default: a read-only function value; calling it changes only what its body changes, D3) | no |
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

**6. Still refused.**
- `call_twice(fn() with owned s: consume(s))`: "this function gives `s` away, so it can run only once,
  and `call_twice` may run it twice; if `call_twice` runs it once, declare the parameter
  `f: once fn()`; otherwise hand over a copy each time: `fn(): consume(copied s)`".
- `call_twice(fn(): consume(s))` (no word): "`consume(s)` would hand `main`'s `s` over from inside a
  function that `main` keeps; give the function its own `s` to hand over, `fn() with owned s:
  consume(s)` (then it runs once), or hand over a copy, `consume(copied s)`".

**7. Cost.** None: the same code as today's `once fn` and `.clone()`.

**8. Kind:** a readable spelling of Rust's guarantee (`FnOnce`), with the message the annoyance asks for.

---

### AN-60 A callback receiving the object that holds it, and a handler that removes itself (A32)

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

    fn fire(mut self):
        self.current = 0
        while self.current < len(self.handlers):
            self.handlers[self.current](self)       # the handler may add or remove handlers
            self.current += 1

    fn remove_current(self) changes handlers, current:
        self.handlers.remove(self.current)
        self.current -= 1

fn count(ref mut ui: Ui):
    ui.clicks += 1

fn add_once(ref mut ui: Ui):                      # adds a handler, then removes itself
    ui.handlers.push(count)
    ui.remove_current()
```

Python's list iterator is this loop (a position checked against the length at every step), so a handler
added during the dispatch runs in it and a handler that removes itself makes the next one move into its
place, as in Python. A handler that must not keep the UI alive captures it weakly:
`ui.button.on_click(fn() with weak ui: if Some(ref mut me) = ui: me.clicks += 1)` (planned-words 3.2).

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `handlers` | none (default: the Ui's own list of function values) | `changes handlers`, or `mut self` |
| element type `fn(ref mut Ui)` | `ref mut` on its parameter: a handler gets the caller's Ui | yes, the Ui (`ref mut`) |
| `current`, `clicks` | none (default: the Ui's own `int`s) | `changes current`, `mut self` |
| `self` in `fire` | `self` | yes (`mut self`) |
| `self` in `remove_current` | `self` | `changes handlers, current` |
| `ui` in `count` | `ref mut`: the caller's Ui | yes |
| `ui` in `add_once` | `ref mut`: the caller's Ui | yes |
| `count` given to `push` | none (default: a function with no state, copied like a number) | no |
| `ui` in the `with weak ui` lambda | `weak`: a link that does not keep the Ui alive | no (a link); `me` may (below) |
| `me` | `ref mut` on the pattern name: a view of the Ui that may change it | yes |

**5. Means exactly.** Today's Ember accepts only the unsound form (AN-61), so the translation is C:
`c/an61_dispatch.c` mode 2, the code D4 emits for `fire`, a handler that pushes 8 handlers (the list's
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

(line 93 is `u->total += st->counter;`, the handler reading its own state after removing itself.)

**6. Still refused.**
- `for h in self.handlers: h(self)` with `fn(ref mut Ui)` handlers (the agreed limitation: a list may
  not change while a loop walks it): "a handler gets `self` and may change `self.handlers`, which this
  loop walks; walk by position, as Python's list does:
  `while self.current < len(self.handlers): self.handlers[self.current](self)`, or, if the handlers
  never change the list, give them a read-only Ui: `handlers: Array[fn(Ui)]`".
- Not refused, warned: a handler that captures the UI that holds it with `with shared ui` makes a cycle
  of owners that is never freed (a leak, not a memory error, as in Rust); AN-64's L3001 names it: "the
  handler is kept by `ui` and keeps `ui`; capture it weakly, `with weak ui`, or take the Ui as the
  handler's parameter".

With `handlers: Array[fn(Ui)]` (read-only Ui) on a Ui with one owner, the `for` loop is accepted and
nothing protects anything: no handler can reach the list to change it.

**7. Cost.** A call copies two words out of the list (plain C copies the same `{fn, env}` pair). A
handler's body holds one count on its own state only across a call that may remove it and only when it
uses its state after it (a rare path). Last run measured this dispatch at 1.045 and 1.037 of plain C
doing exactly the same work with the same allocator, safe Rust at 1.490 and 1.472
(`target/redesign2/graphs-callbacks-attack2/bench/runs/w4_a.txt`, `w4_b.txt`;
`phase3b-graphs-callbacks-attack.md` 6.4); the measured program is this one (phase 3's W4).

**8. Kind:** a smarter compile-time solution (where each count is needed is decided when compiling;
the `for` refusal is the agreed limitation).

---

### AN-61 Unsound today: a handler list dispatched by index is accepted and writes freed memory

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

**3. NEW.** The position call of AN-60:

```ember
    fn fire(mut self):
        i = 0
        while i < len(self.handlers):
            self.handlers[i](self)
            i += 1
```

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `self` | `self` | yes (`mut self`) |
| `i` | none (default: `fire`'s own `int`) | its owner may |
| `self.handlers[i]` | none (default: the element is read, two words, and called; nothing points into the list during the call, D4) | no |

**5. Means exactly.** `c/an61_dispatch.c` mode 2, pasted under AN-60: clean under ASan.

**6. Still refused.**
- `h = self.handlers.get_mut(i).unwrap()` then `h(self)`: "`h` points into `self.handlers`, and
  `h(self)` lets the handler change `self.handlers` while `h` points into it; call it by position:
  `self.handlers[i](self)`". (`get_mut` returns a `ref mut` view, the vocabulary's `get_mut` row; a
  `ref mut` view into a place may not live across a call given a changeable name that reaches the same
  place: Rust's rule, E0499.)

**7. Cost.** As AN-60. The refused form had no cheaper code: the copy back it needs is what made it
unsafe.

**8. Kind:** a smarter compile-time solution (the hole is closed when compiling; the form that works
costs what C costs).

---

### AN-62 A loop calling a stored callback with unknown effects pays a count pair per step (last run's limit)

**1. Intent:** walk from a node up to the root and report every ancestor to a stored callback.

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

    fn depth_total(self) -> int:
        depth = 0
        for leaf in self.leaves:
            if Some(n) = leaf:
                a = n.parent
                while Some(p) = a:
                    self.on_visit(p)          # unknown: may end `p` (Node is a `weak` target)
                    depth += 1
                    a = p.parent
        return depth
```

The program writes nothing for the protection. D5: `Node` is the target of `weak` links (`parent`,
`leaves`), so an unknown call may end a `Node`; `Node` has no `drop` method of its own, so the compiler
puts a **deferred-drop region** around the loop instead of a count pair per step: entering the loop adds
1 to a per-thread counter and leaving subtracts 1; a `Node` whose count reaches zero while the counter
is above zero is set aside whole (marked dead, so `weak` reads of it give `None`) and dropped when the
loop ends. `p` therefore stays readable after the call; if the callback took `p` out of the tree,
`p.parent` was cleared by `set by` and the walk stops there.

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `value` | none (default: the node's own `int`) | `changes value`, `mut self` |
| `parent` | `weak` (does not keep its node alive), `set by children` (only `children` sets it) | no: the program never assigns it |
| `children` | none (default: the node's own list; each element has this one owner) | `changes children`, `mut self` |
| `root` | none (default: the tree's own Node) | `changes root`, `mut self` |
| `leaves` | `weak` on its elements: links that do not keep the leaves alive | `changes leaves`, `mut self` |
| `on_visit` | none (default: the tree's own function value) | `changes on_visit`, `mut self` |
| `self` | `self` | no (read-only) |
| `depth` | none (default: the method's own `int`) | its owner may |
| `leaf` | none (default: a read-only view of an element of `leaves`, a `weak` link) | no |
| `n`, `p` | none (default: a read-only view of the node the link points at, kept alive until its last use, D6) | no |
| `a` | none (default: the method's own `Option` copy of a link, as `n.parent` gives it) | its owner may (re-pointed each step) |

**5. Means exactly.** C, `c/an62_defer_asan.c`: a leaf, its parent `mid`, and a callback that takes
`mid` out of the tree and drops its last owner while the walk is on `mid`. Three modes, under ASan
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
(the region) and mode 3 (last run's count pair) are clean and give the same result.

The per-step code is the unprotected step's. The workload's walk (`c/an62_walk_bench.c`, 1,000,000
nodes, 10 passes, a call through a function pointer the C compiler cannot see on every step) built
`-DSLAB` and `-DSLAB -DDEFER`; the whole difference between the two assembly files (`c/bin/walk_noprot.s`,
`c/bin/walk_defer.s`), apart from the program's name string and the counter's declaration:

```text
205a206
> 	incq	g_defer(%rip)
233a235
> 	decq	g_defer(%rip)
```

one increment before each pass's loop and one decrement after it; nothing inside the loop.

**6. Still refused.** Nothing new is refused. What stays: a class with its own `drop` method keeps the
count pair (deferring it would move a visible action); a view into a `Node`'s contents (an element of
`p.children`) across the unknown call stays refused by the loop rule ("the callback may change
`p.children`, which this loop walks; ...", area A's message).

**7. Cost.** Timed with `target/redesign2/bench/run.ps1 -Rounds 5` (pinned to the fast cores, mains
checked each round; `c/walk_runs.txt`, medians in `c/walk_summary.txt`). Every run printed checksum
722575054.

| program | median ms | ratio to plain C, same layout | spread |
|---|---|---|---|
| plain C, same layout (`walk_plain.exe`, `-DSLAB -DARRAY24`) | 1905.5 | 1.000 | 30% |
| set by, no protection (`walk_noprot.exe`) | 2068.5 | 1.086 | 29% |
| set by, a count pair per step (`walk_retain.exe`, last run's rule) | 2444.0 | 1.283 | 34% |
| set by, deferred-drop region (`walk_defer.exe`) | 1797.9 | 0.944 | 37% |

**PROVISIONAL**: other agents of this phase were running; the spread (fastest to slowest) is 29% to 37%,
far over the 10% the timer accepts, so these numbers rank nothing finely. What they agree with is the
assembly above: the region's walk is the unprotected walk plus two instructions per pass, and the count
pair is again about 1.2x to 1.3x (last run: 1.27x to 1.32x). Peak memory: 84,544 KB for every `set by`
variant against 76,688 KB for plain C (the 8-byte header, `phase3b-combined.md` 4.4's choice; not this
answer's). A region holds the memory of the objects set aside until the loop ends: in this workload none.
Phase 4 measures the final form.

**8. Kind:** a new mechanism (clever: the protection moves from every step to the rare death; readable:
nothing is written; as fast as the unprotected walk inside the loop; as safe as the count pair, shown
under ASan).

---

### AN-63 Trees with parent pointers, graphs, cycles (A8)

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
    if Some(p) = n.parent:
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

Moving a node is written: taken out of one list, pushed into another (planned-words 3.3). For links
that are not the reverse of an owning field (a camera's target, an observer list), `weak` alone:
`target: weak Player`, `observers: Array[weak Observer]`. For a graph, each city's roads and the roads
into it: `roads: Array[weak City]` and `into: Array[weak City] set by roads`, the cities owned by one
list (last run's rules 4.4, with D7).

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `name` | none (default: the node's own String) | `changes name`, `mut self` |
| `parent` | `weak`: does not keep its node alive; `set by children`: only `children` sets it | no: never assigned by the program |
| `children` | none (default: the node's own list; one owner per element) | `changes children`, `mut self` |
| `n` | none (default: a read-only view, a parameter) | no |
| `p` | none (default: a read-only view of the parent, kept alive until its last use) | no |
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
- `child.parent = Some(self)` (the Python spelling, a strong field): "`child.parent = Some(self)` would
  make `parent` a second owner of this node, and `self` is not yours to hand over; a link back to the
  node whose `children` holds this one is written `parent: weak Node set by children`, and `push` sets
  it".
- `leaf.parent = root` with `set by`: "`parent` is set by `children`; to give `leaf` a parent, write
  `root.children.push(leaf)`" (planned-words 3.3).
- `parent: Node set by children` without `weak`: planned-words' message.
- Lending a `set by` holder as a changeable list (`shuffle(ref mut xs: Array[Node])` called with
  `p.children`): "`children` keeps every child's `parent` in step, so it is changed only by its own
  methods; pass it read-only, or write the change with `push`, `insert`, `remove`".
- A cycle of `shared` owners (`a: shared Node` holding `b` holding `a`) is not refused (it is a leak,
  not a memory error, as in Rust); AN-64's warning names it.

**7. Cost.** `set by`: reading is a plain load; an insert or remove writes the link (one store). Last run
measured the same mechanism on the tree workload, the link's own work, at 1.025 and 1.149 of plain C laid
out the same way, and safe Rust's index arena at 1.519 and 1.357
(`graphs-callbacks-attack2/bench/runs/w2_a.txt`, `w2_b.txt`); with a 4-byte header 0.964 and 1.024
(`phase3b-combined.md` 4.4). A graph with edges kept both ways: 0.969 and 1.081 of plain C keeping the
same reverse lists (`runs/graph_a.txt`, `graph_b.txt`). `weak`: one test of the target's header per read,
as Rust's `Weak`, which on churn costs 0.919 and 0.935 of plain C (`runs/weak_g.txt`, `weak_h.txt`).
Last run's push also searched the node's old list; here the program writes the `remove`, so no hidden
search.

**8. Kind:** a new mechanism (`set by`), with `weak` as a readable spelling of Rust's `Weak`.

---

### AN-64 L3001 tells the programmer to make the owning edge weak

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
`parent: weak Node set by children`.

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `name` | none (default: the person's own String) | `changes name`, `mut self` |
| `friends` | `shared` on its elements: one of several owners of each person | `changes friends`, `mut self` |
| `best` | `shared`: one of several owners of that person | `changes best`, `mut self` |

**5. Means exactly.** The NEW tree is AN-63's C (`c/an63_links.c`): no ASan report, and its line
`4 leaf parent=None` (pasted under AN-63) shows the root's end clearing its child's link before
releasing it. (ASan on Windows has no leak checker, so "every node is freed" is read from the code's
drops, not from a leak report.) The warning itself is a message; the program it is given for is a leak,
not something to translate.

**6. Still refused.** Nothing new; the Python spelling's error is AN-63's.

**7. Cost.** None (compile time).

**8. Kind:** a smarter compile-time solution (one-owner links are proved acyclic, so the warning is only
given where a cycle can exist, and it names the right link).

---

### AN-65 `upgrade` is Rust's word for "the object, if it is still alive"

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
    if Some(p) = cam.target:
        println(p.health)
    else:
        println("gone")
```

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `target` | `weak`: a link that does not keep the player alive | `changes target`, `mut self` (re-pointing) |
| `player` | none (default: `main`'s own Player) | its owner may |
| `cam` | none (default: `main`'s own Camera) | its owner may |
| `weak player` | `weak`: a link, not a hand-over; `player` stays `main`'s | no |
| `p` | none (default: a read-only view of the player, kept alive until its last use) | no |

**5. Means exactly.** `means/m65_weak_read.em` (`if Some(p) = cam.target.upgrade():`):

```text
=== m65_weak_read.em
exit=0
```

In C, `c/an63_links.c` part 2: a read is `(w && !w->dead) ? w : NULL`; lines `5 100` and `6 gone` of
its output (pasted under AN-63) are the read before and after the player ends, clean under ASan.

**6. Still refused.** `cam.target.upgrade()`: "a `weak` link is read directly: `if Some(p) =
cam.target:`"; `Weak[Player]`, `Weak(player)`: "write `weak Player`", "write `weak player`";
`cam.target.health`: "`cam.target` may be gone; write `if Some(p) = cam.target:`"; `cam.target =
player`: planned-words 3.2's message.

**7. Cost.** One test of the target's header per read; a count pair only where D5 needs one (Rust's
`upgrade` always makes one).

**8. Kind:** a readable spelling of Rust's guarantee.

---

### AN-66 A bare interface type is a counted, shared, dynamically dispatched handle with no word

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
| `shapes` | `owned` on its elements: each shape has this one owner | `changes shapes`, `mut self` |
| `shown` | `shared` on its elements: one of several owners | `changes shown`, `mut self` |
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

**6. Still refused.** `shapes: Array[Shape]`: "say who owns the shapes: `Array[owned dyn Shape]` (the
list is their only owner) or `Array[shared dyn Shape]` (other names own them too)". `b = a` for an
`owned dyn Shape`: hands it over (row 9); for a second owner, `b = shared a`. `Box[dyn Shape]`: "write
`owned dyn Shape`".

**7. Cost.** `owned dyn`: one heap block per value and a method-table pointer beside it (Rust's
`Box<dyn>`); `shared dyn`: the same plus a count (Rust's `Rc<dyn>`); a call is one indirect call.

**8. Kind:** a readable spelling of Rust's guarantee.

---

### AN-67 Trait objects and downcasting (A13)

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

    fn get_mut[T](mut self, k: u128, i: int) -> ref mut T:
        if Some(ref mut pool) = cast self.pools[k] if Pool[T]:
            return ref mut pool.dense[i]
        panic("no pool of this type at this key")

interface Inspect:
    fn inspect[V: Visitor](self, ref mut v: V)

fn walk(xs: Array[owned dyn Inspect]) -> int:
    s = Sum(total=0)
    for x in xs:
        x.inspect(s)                # through `dyn`: the entry made with V = dyn Visitor
    return s.total
```

`cast <place> if D` on an `owned dyn` (or `shared dyn`) place tests the value's type and gives a view of
it as a `D`: `Some(ref mut pool)` binds a changeable view because the place is changeable (`mut self`);
`Some(pool)` a read-only one. A generic method whose type parameter appears only as the type of a `ref`
or `ref mut` parameter, with a `dyn`-compatible bound, gets one method-table entry, the generic body
compiled with `V = dyn Visitor` (last run's answer, saved by its attack, 7.1: checked per
implementation). The spec's `as?`/`as!` text becomes `cast x if D` / `cast x as D` (the compiler's,
ODR-280).

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `pools` | `owned` on its values: each pool has the map as its one owner | `changes pools`, `mut self` |
| `self` in `get_mut` | `self` | yes (`mut self`) |
| `k`, `i` | none (default: read-only numbers) | no |
| `pool` | `ref mut` on the pattern name: a view of the registry's pool that may change it (row 8's words) | yes |
| result `-> ref mut T` | `ref mut`: a view into the registry's element | yes |
| `v` | `ref mut`: the caller's visitor | yes |
| `xs` | `owned` on its elements (one owner, the list); the parameter itself none (a read-only view) | no |
| `s` | none (default: `walk`'s own Sum) | its owner may (lent `ref mut` to `inspect`) |
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

**6. Still refused.**
- A generic method whose type parameter is returned or stored by value (`fn make[V: Visitor](self) ->
  V`) through `dyn`: "`Inspect` cannot be used as `dyn Inspect`: `make` returns its `V`, and a `dyn` value
  has one entry per method; take the visitor as `ref mut v: V`, or use a type parameter
  `[I: Inspect]` instead of `dyn Inspect`" (E2050 with the line).
- `cast x as D` / `cast x if D` between unrelated types (E2020), with "`Box[dyn AnyPool]` is now
  `owned dyn AnyPool`" where that is the cause.
- `as?` / `as!`: today's message (it already gives the line).

**7. Cost.** A `cast`: one load of the method table's type id and one compare (last run's references
attack fix, `phase3b-combined.md` 3.4); safe Rust's `downcast_mut` was measured slower (1.710x and 1.606x
against the pool table's 1.208x and 1.084x of plain C, `phase3b-references.md` 3.4 as quoted in
`phase3b-graphs-callbacks.md` 6.3). The `dyn` entry: the indirect calls Rust's hand-written workaround
makes.

**8. Kind:** a smarter compile-time solution.

---

### AN-68 `if c = cast s if Circle:` gets E2036 with a help that is not what was meant

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
        if Some(c) = cast s if Circle:
            t += c.r
```

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `s` | none (default: a read-only view of an element) | no |
| `c` | none (default: a read-only view of the same object as a `Circle`) | no |
| `t` | none (default: the function's own `f64`) | its owner may |

**5. Means exactly.** This is today's Ember (`means/m68_if_some.em`):

```text
=== m68_if_some.em
exit=0
```

**6. Still refused.** `if c = cast s if Circle:`, with the message:
"`c = ...` names `c` but tests nothing; to run the block only when `s` is a `Circle`, write
`if Some(c) = cast s if Circle:`".

**7. Cost.** None (one type-id compare, as AN-67).

**8. Kind:** an understandable limitation kept (`=` in an `if` binds only through a pattern), with the
message giving the line that works.

---

### AN-69 Generics friction (A16)

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
    acc = copied xs[0]
    for x in xs[1..]:
        acc = acc + x
    return acc

pub fn total[T: Add[T, Output = T] + Copy](xs: Span[T]) -> T:    # `pub`: the bound is written
    ...

extend Array[Vec3] implements Neg:           # accepted in a program (a package nothing depends on)
    ...
```

Rules (last run's, with one change): a function that is not `pub` gets its bounds from its body (every
operation on `T` maps to the one interface in scope that provides it; recorded, printed by `ember inspect
--bounds`); a `pub` function writes its bounds, because they are its promise to other packages, and the
error gives the exact line; a package that nothing can depend on (a program or a test) may implement any
interface for any type (two implementations of one interface for one type anywhere in the program stay
refused). The change from last run: inferred bounds stop at `pub`, so nothing another package relies on
is decided by the compiler from a body it cannot see.

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `xs` | none (default: a read-only view, a parameter) | no |
| `acc` | `copied`: the function's own copy of the first element | its owner may |
| `x` | none (default: a read-only view of an element) | no |

**5. Means exactly.** The bound the compiler infers for `sum`, written out (`means/m69_bounds_written.em`,
`fn sum[T: Add[T, Output = T] + Copy](xs: Span[T]) -> T:`):

```text
=== m69_bounds_written.em
exit=0
```

The orphan exemption is a permission, not code; the overlap stays refused, so it needs no translation.

**6. Still refused.**
- A `pub` generic with an operation its bounds lack: "`total` is `pub`, so its bound is written: write
  `pub fn total[T: Add[T, Output = T] + Copy](xs: Span[T]) -> T:`" (E2040 with the line; a non-`pub`
  function never gets it).
- E2121 in a library package: "`Array[Vec3]` and `Neg` both belong to other packages, and packages
  that depend on this one could see a second `Neg` for `Array[Vec3]`; wrap it: `struct Vecs(Array[Vec3])`
  and `extend Vecs implements Neg:`".
- E2041, overlapping implementations (the most specific one winning is unproven for Ember, last run's
  open item): "`i32` already gets `Describe` from `extend[T: Debug] T implements Describe` on line 5,
  and two implementations may not apply to one type; narrow the general one's bound so it leaves `i32`
  out, or give `i32` its own function: `fn describe_int(x: i32) -> String:`".
- A non-`pub` body calling a method no interface in scope declares: "`T` has no `weight`: no interface
  in scope declares `fn weight`; declare one (`interface Weighed: fn weight(self) -> int`) and the
  bound follows" (last run's duck-typing extension is not taken: it would let a typo surface at a call
  far away).

**7. Cost.** None at run time: every instance compiles to the code written bounds give.

**8. Kind:** a smarter compile-time solution (bounds and the orphan exemption); the overlap is an
understandable limitation kept, with its message.

---

### AN-70 `virtual` in a class that cannot be subclassed is accepted with a warning (W2111)

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
    virtual fn on_update(self, dt: float):
        pass
```

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `angle` | none (default: the door's own `float`) | `changes angle`, `mut self` |
| `self` | `self` | no (read-only) |
| `dt` | none (default: a read-only number) | no |

**5. Means exactly.** `means/m70_open_class.em`:

```text
=== m70_open_class.em
exit=0
```

**6. Still refused.** `virtual` in a class that is not `open` or `abstract`, now an error: "`Door`
cannot be subclassed, so `virtual` on `on_update` changes nothing; to let subclasses replace it write
`open class Door:`, or remove `virtual`" (vocabulary row `virtual`).

**7. Cost.** None.

**8. Kind:** an understandable limitation kept (only an `open` class has subclasses), now refused with
the line to write instead of accepted with a warning.

---

## 5. Words used, and their positions

No new word. Positions used, all planned (planned-words section 4) or in the vocabulary:

| word | where it is used here | answers |
|---|---|---|
| `with owned x`, `with copied x`, `with shared x`, `with weak x` (capture list) | after a lambda's parameters | AN-53, AN-54, AN-55, AN-57, AN-59, AN-60 |
| `weak` | in a type (`target: weak Player`, `Array[weak Node]`), before a value (`weak player`), in a capture list | AN-60, AN-62, AN-63, AN-64, AN-65 |
| `set by` | after a `weak` field's type | AN-62, AN-63 |
| `owned dyn I`, `shared dyn I`, `dyn I` (parameter) | in a type | AN-66, AN-67 |
| `once fn()` | a parameter's type | AN-59 |
| `ref mut` | a parameter's name, a callable type's parameter (`fn(ref mut Ui)`), a pattern name | AN-54, AN-57, AN-60, AN-67 |
| `changes` | a method's first line | AN-53, AN-60 |
| `cast x if D`, `cast x as D` | before a value | AN-67, AN-68 |
| `copied` | before a value | AN-59, AN-63, AN-69 |

Two rulings planned-words left to area D (its section 8): **`weak`'s count while a bound name lives**:
none when nothing in the name's life can end the target (D5), a deferred-drop region when a loop calls
unknown code and the class has no `drop` method, a count pair otherwise; **`set by`'s full rules**: D7.
**A `weak` link to a `thread_shared` value**: no word; the target's header says whether its counts are
atomic (as planned-words proposed).

One question for the combiner, not a word: the call-site spelling of an argument given to a `ref mut`
parameter (`self.handlers[i](self)` here, plain as today's `mut` parameters are). Area B or C may
settle it; these answers follow whatever is settled.

## 6. What this area needs from the others, and clashes seen

| from | what | used by | if not given |
|---|---|---|---|
| A | the loop rule (a list may not change while a loop walks it) applied to calls through function values and to variables a living function value changes (D3) | AN-53, AN-58, AN-60, AN-62 | the attack 5.2 use after free returns |
| B | `changes` on a method limiting where an `owned` function-value parameter can go (D1's proof) | AN-53 | the button's capture uses a box instead of an address (no meaning change; one allocation) |
| C | `shared` objects lent read-only while unknown code runs (AN-48), which D5 treats like `weak` targets | AN-60, AN-62 | D5 must assume every class may be ended by an unknown call |
| C | `Copy` excludes class handles (vocabulary C-3): a capture-free function stays `Copy`; a function value with state is not | AN-60 | none |
| E | a box (D1) has a one-thread count and never crosses threads; `with thread_shared x` for the shared form | AN-55 | not memory-safe across threads |
| phase 4 | the C writer's W2 needs nodes held by both a list and a parent; under D7 a node is owned by its parent's `children` and the index list holds `weak` links (`leaves`/`nodes: Array[weak Node]`), as AN-62's NEW does | W2 | a node in two owning lists cannot have a single `set by` link (planned-words 3.3's refusal) |

## 7. Last run's answers: repaired, kept, and the good parts of what was dropped

| last run's piece (`phase3b-graphs-callbacks.md`, as saved by its attack) | here | why | good part kept |
|---|---|---|---|
| captures as in Python, the compiler picks copy/move/pointer/box | kept as D1 under row 6's words; `copied`/`owned` only when named | row 6 is that rule; a copy is never made silently (a read-only `Copy` capture may still be copied when that cannot be seen, `[CLO-13]`) | the box is free on the measured workloads |
| `owned fn` with a warning (discarded last run) | removed (`owned fn` refused) | the owner's named failure (JOB) | — |
| a stored closure is an object; two words copied out; body holds its state | kept as D4 | measured 1.04x of plain C, ASan-clean (AN-60) | — |
| the count pair per step of a walk calling a stored callback | replaced by the deferred-drop region (D5) | the count pair cost 1.27x to 1.32x; the region adds two instructions per loop | the count pair stays for classes with a `drop` method |
| a contract on the callback's type ("drops no `Node`", proposed `@keeps(Node)`) | not taken | needed a new word and every callback type to carry it; the region needs nothing written | the idea that what a callback cannot do removes the protection (D5's one-owner case) |
| back links, `parent: Option[Node] = backlink(children)` | respelled `parent: weak Node set by children` (planned-words) | the old spelling read as a default value computed by a call | the mechanism, its membership-change order, rule 6 |
| a push takes the node out of its old parent first | dropped | a one-owner list element is taken out by a written `remove` (planned-words 3.3); no hidden search | Python's `appendChild` meaning is one written line more |
| `weak` cleared by registration | replaced by a test on read | last run's combiner's choice (`phase3b-combined.md` 4.3): three fixes, 1.09x to 1.27x on churn | registration's plain-load read is what `set by` gives for back links |
| `isinstance(s, Circle)` narrowing | not taken | planned-words: `cast` and `if Some(c) = cast s if Circle:` already exist and compile today | narrowing after a test: the attack's rules (ends at writes, at calls running a closure that writes the local) apply if the combiner adds it |
| a generic method through `dyn` | kept (AN-67) | — | — |
| bounds inferred from the body | kept for functions that are not `pub` | a `pub` function's bounds are its promise; the owner's rule that the compiler does not decide what the code does not say | the inferred bound printed by `ember inspect --bounds` |
| the program exemption from the orphan rule | kept | — | — |
| most specific implementation wins | not taken (open, unproven) | soundness not shown for Ember | Python's `singledispatch` meaning, if it is ever proved |
| duck-typed method names, `acc = 0` typed by use (attack 8.2, 8.3) | not taken | a typo would surface far from its line; the literal rule was unverified | the error now names the interface to declare |
| loop variable per pass (`[CLO-13]`) | kept | with row 6 each pass's variable is the real variable of that pass | — |

## 8. Files

`C:\Users\ism19\Code\ember\target\redesign3\11-area-D\`: `run_checks.sh`; `old/` (28 `.em`,
`check_output.txt`); `means/` (16 `.em`, `check_output.txt`); `c/an53_captures.c`, `c/an61_dispatch.c`,
`c/an62_defer_asan.c`, `c/an63_links.c` with their `_output.txt`; `c/an62_walk_bench.c`,
`c/an62_walk_plain.c`, `c/ember_model.h`, `c/bin/walk_*.exe`, `c/bin/walk_*.s`, `c/walk_runs.txt`,
`c/walk_summary.txt`.
