# K12-function-lent-to-std-and-kept (input for method 3; made by deep/v3/make_inputs.py from phase2.json, phase3.json, deep/phase1-groups.json, JOB.md and the part files)

## cluster
**cid:**
K12-function-lent-to-std-and-kept

**title:**
A function from a variable given to std that keeps it (thread.spawn, push, a returned adapter): lent, handed over, or owns?

### members
#### [1]
J3-02

#### [2]
J6-12

**primary:**
J3-02

**why_together:**
Both ask what verdict 150's loan means when the std function keeps the function past the call (spawn in J3-02, push in J6-12). Their recommendations draw the line in different places (see conflicts), so one rule must answer both.

**severity:**
high

**impact_rank:**
12

**notes:**
Show J6-12's `b.handlers.push(beep)` as the second line of J3-02's program. J3-02 A turns verdict 51's own line `a = thread.spawn(push_a)` into `thread.spawn(owns push_a)`, so say so. Option C reverses verdict 51 ('both ways should be permissible'), so label it that way. J3-03 (the owner's own std/program split, verdicts 147 and 150) may be shown beside it. K31 (J8-01) follows this answer.

## questions
### [1]
**id:**
J3-02

**verdict:**
confirmed

**route:**
owner_odr

**severity:**
high

**confidence:**
0.7

**reason:**
Nothing settles it. FN-2a (p05:132-134) and STD-19 (p15:316-317) lend a function from a variable to any standard-library function, with no word. THR-10 (p11:173-175) says the spawned thread "may outlive the function that started it", so a loan of `main`'s `push_a` to it cannot be kept, and CLO-4/CLO-7 refuse a function value that outlives what it uses. No rule lets `thread.spawn(owns push_a)` hand a named function over: FN-2a says such a function "takes no word". Yet CLO-12 (p06:333) and GRM-28 (p03:253) show `a = thread.spawn(push_a)` as working code, and verdict 51 says both forms are allowed. So no spelling is decided to compile. The same gap hits a std adapter kept past its line (`return xs.iter().map(h)`). Verdict 150 decided the loan for `for_each`, which runs the function during the call. It does not say what happens when the std function keeps it.

**title:**
Does `thread.spawn(push_a)` lend a local function, hand it over, or need `owns`?

**problem:**
The spec lets a long thread body be written as a local function and started by name, `a = thread.spawn(push_a)`. It also says a function from a variable given to a standard-library function is lent and takes no word. But `thread.spawn` keeps the function on a thread that may outlive `main` (C++: a `std::thread` given a reference to a local lambda). A loan cannot be kept, and the spec never says that `thread.spawn(owns push_a)` is allowed. So the spec decides no spelling that compiles. The same goes for an adapter kept past its line, `return xs.iter().map(h)`.

**example:**
import std.thread
from std.sync import Mutex

class Log:                                    # a log any thread may add to
    lines: Mutex[Array[String]] = Mutex([])

    fn add(line: StrView):
        with g = this.lines.lock():           # the lock guards the list
            g.push(owns String(line))

fn main():
    log = thread_shared Log()                 # one owner per thread that holds it
    fn push_a() thread_shared log:            # a local function holding its own owner of `log`
        log.add("a")
    a = thread.spawn(push_a)                  # lent? handed over? or must it say `owns push_a`?
    push_a()                                  # may main still run it here?
    a.join()

**spec_today:**
FN-2a (p05:132-134): "A function from a variable given to a standard-library function that takes one is lent and takes no word: after `xs.iter().for_each(h)`, `h` is the caller's still". STD-19 (p15:316-317): "A function from a variable is lent to the call and takes no word". THR-10 (p11:173-175): "`thread.spawn` runs a function on a new thread, which may outlive the function that started it. It takes ... `pub fn spawn(owns f: thread_shared fn() -> R) -> JoinHandle[R]:`". GRM-15 (p03:394-395): `owns` is required "before an argument for an `owns` parameter, whatever is given". CLO-12 (p06:331-333): "may be passed by its name ... (`fn push_a() thread_shared log:` with the body `log.push("a")`, then `a = thread.spawn(push_a)`)". GRM-28 (p03:251-253) gives the same `a = thread.spawn(push_a)`.

#### options
##### [1]
**letter:**
A

**summary:**
A loan stays the default; `owns` before a named function hands it over to any `owns` parameter, the library's included.

**code:**
    a = thread.spawn(owns push_a)             # handed over: the thread owns it now
    a.join()

# for_each still borrows with no word:
#     h = fn(x: int): println(x)
#     [1, 2].iter().for_each(h)
#     h(5)

**outcome:**
Compiles and prints `a`. A later `push_a()` is E3040, because `push_a` was handed over. `thread.spawn(push_a)` with no word is refused, because the thread may outlive `push_a`, and the help gives `thread.spawn(owns push_a)`. `for_each(h)` keeps borrowing and `h(5)` prints 5. `return xs.iter().map(owns h)` works the same way.

##### [2]
**letter:**
B

**summary:**
With no word, a named function is handed over when the library function keeps it past the call.

**code:**
    a = thread.spawn(push_a)                  # no word: handed over, since spawn keeps it
    a.join()

**outcome:**
Compiles and prints `a`. A `push_a()` after the spawn is E3040, though nothing in that line says `push_a` was given away. `for_each(h)` still borrows, so the same-looking call means two things depending on what the library function does with it.

##### [3]
**letter:**
C

**summary:**
A named function is always lent, so `thread.spawn` cannot take one; write the function inside the call.

**code:**
    a = thread.spawn(owns fn() thread_shared log: log.add("a"))   # the only form that compiles
    a.join()

**outcome:**
`thread.spawn(push_a)` is refused, because the thread may outlive `push_a`, and its help writes the function into the call. A local function can never start a thread, and the spec's `thread.spawn(push_a)` examples go.

#### recommendation
**letter:**
A

**reason:**
One rule: a lent function is like a view and cannot outlive the variable, and `owns` at a call hands over, as it does for every other argument. Both ways of writing a long thread body keep working, and the hand-over is written in the line.

**checklist:**
2 (the local-function form, allowed by the owner, compiles); 4 and 5 (the hand-over is written, and `owns` means one thing at every call); 6 (no new exception: the loan with no word stays as decided); 9 (the refusal gives a line that compiles); 13 (no hidden rule that depends on what the callee does with the function).

### [2]
**id:**
J6-12

**verdict:**
confirmed

**reason:**
Part decided: the owner ruled that a function from a variable given to a standard-library function is lent (verdict 150, ODR-240, FN-2a p05 132-134, STD-19 p15 315-317), and that the program's own `owns` parameters take `owns` at the call (verdict 147); that split is the owner's and is not reopened here. Still open: std's `push` keeps what it is given and takes `owns` at the call (verdict 149: "`push` too ... `names.push(p.name)` is an error"), while verdict 150 lends a function from a variable to std. For `handlers.push(beep)` the two verdicts give different answers, and a list cannot hold a lent function past the call. ODR-240's 'std's callable-taking functions' does not say whether push on a list of functions is one.

**route:**
owner_odr

**severity:**
high

**confidence:**
0.7

**title:**
Does handlers.push(beep) hand over a function kept in a variable?

**problem:**
A GUI program keeps a click handler in a variable and adds it to a button's list of handlers with the standard `push`. One ruling says a function from a variable given to a standard-library function is only lent; another says `push` takes over what it is given and needs `owns` at the call. A list keeps its elements after the call, so the two cannot both hold, and the specification does not say which one wins.

**example:**
type Handler = fn()

class Button:
    handlers: Array[Handler] = []       # called on each click

fn main():
    b = Button()
    beep = fn(): println("beep")        # a function kept in a variable
    b.handlers.push(beep)               # std's push keeps it: lent, handed over, or an error?
    beep()                              # is `beep` still the caller's here?
    for h in b.handlers:
        h()

**spec_today:**
p05 132-134 (FN-2a): "A function from a variable given to a standard-library function that takes one is lent and takes no word: after `xs.iter().for_each(h)`, `h` is the caller's still (`[STD-19]`)." p05 119-121 (FN-2a): "`take(owns xs)` for `fn take(owns xs: Array[int]):`, whatever is given ... `names.push(owns p.name)` a part". p15 305-306 (STD-19): "**A function is taken as a type parameter**, by the adapters that apply one and the consumers alike". p15 311-312: "the shared type, `owns f: fn(Item)`, is legal too, and is what keeps functions of different kinds in one list (`handlers: Array[fn()]`)".

#### options
##### [1]
**letter:**
A

**summary:**
push keeps it, so it is handed over like any value; lending stays for functions given as a function parameter

**code:**
b.handlers.push(owns beep)              # beep is the list's now
b.handlers.push(owns copy of beep)      # or keep beep and give the list a copy

**outcome:**
`b.handlers.push(beep)` is an error: "`push` keeps what it is given; hand `beep` over: `push(owns beep)`, or keep it: `push(owns copy of beep)`". With `owns beep`, a later `beep()` is E3040; with `owns copy of beep`, it prints "beep" and the loop prints "beep". A function from a variable is still lent, with no word, to a standard function that takes a function as its function parameter (`for_each(h)`, `map(h)`, `retain(keep)`).

##### [2]
**letter:**
B

**summary:**
Lent, and the list holds a view of it

**code:**
b.handlers.push(beep)

**outcome:**
Compiles; the list holds a view of `beep` and may not outlive it (as a list of `StrView`s may not outlive their text): returning or storing `b` past `beep`'s scope is E3063. `beep()` prints "beep".

#### recommendation
**letter:**
A

**reason:**
A list owns its elements, so its `push` hands over a function as it does every other value, with `owns` written at the call; lending stays exactly where the library already takes a function as a function parameter (STD-19), so no line keeps a value silently.

**checklist:**
1, 4 (the word at the call says it), 5, 6 (push's existing rule), 9, 11

## group
**gid:**
P06

**principle:**
A call does to each argument exactly what the word at the call says (no word lends it for the call, owns hands it over), for every callee (a function, a method, an operator, a written init, a standard-library or C function) and every argument (a function held in a variable included), and a callee keeps past the call only what was handed over.

**shared_cause:**
FN-2a and GRM-15 give each kind of callee its own call-word rule, with no single rule tying the word to what the callee does and keeps. A function from a variable is lent to std (verdict 150). A constructor's argument takes no word (159). String's + is built in outside the operator interfaces (TYP-21, TXT-11) and silently takes its left side. So a std function that keeps a lent function (spawn, push, a returned adapter), an init with owns parameters, and String + each get two answers or none.

### members
#### [1]
J3-02

#### [2]
J6-12

#### [3]
J2-04

#### [4]
J6-14

#### [5]
J1-07

### clusters
#### [1]
K12-function-lent-to-std-and-kept

#### [2]
K18-string-plus

#### [3]
K19-written-init-call-words

### answered
### verdicts_involved
#### [1]
3: Mutex, RwLock, Atomic and channel ends take no ref mut at the call

#### [2]
51: a named local function or a lambda in the call, both allowed (thread.spawn(push_a))

#### [3]
80: a function that outlives its maker takes what it keeps with owns

#### [4]
84: owns hands over every type the same way

#### [5]
94: scope.spawn keeps a plain fn()

#### [6]
145

#### [7]
146: owns before a read-only parameter's argument is an error

#### [8]
147: an owns parameter's argument says owns at the call

#### [9]
148: return takes no word

#### [10]
149: push too

#### [11]
150: a function from a variable given to std is borrowed

#### [12]
158: a function written in a std call says owns

#### [13]
159: a constructor's argument takes no word, as =

#### [14]
160: a value made with shares or weakref in the call takes no owns

#### [15]
161: owns binds as unary minus

**other_places:**
thread.spawn, scope.spawn (verdict 94's exception), push, insert and extend, iterator adapters returned or stored in fields, Option.map, event-handler registration, C functions that keep callbacks (P08, J8-01), every operator of every type (+, ==, [], +=, in, comparisons) as a call of its interface, From conversions at `?`, channel send, methods taking owns this, and a lock or channel-end owns parameter (lead fix J3-04: verdict 3 against 147, one-line confirmation).

**severity:**
high

**rank:**
6

### depends_on
**notes:**
Phase 3's unified rule for K12: a function from a variable passed with no word is lent; a loan the callee keeps past the call is refused at the call with help `owns h` or `owns copy of h`; for_each(h) and xs.iter().map(h).sum() stay loans. Show J6-12's `b.handlers.push(beep)` as the second line of J3-02's program. J3-02 A rewrites verdict 51's `thread.spawn(push_a)` as `thread.spawn(owns push_a)`, and option C reverses verdict 51: say both. J1-07 A narrows verdict 159 to memberwise constructors: quote it. K18: one B spelling, `(owns label) + "!"`. A is refined with C++'s rvalue rule, so `a + b + c` makes one String. Show J3-03 (verdicts 147/150's std/program split) beside it. J1-06's widening of verdict 160 to thread_shared and sharedref mut is a one-line confirmation under the same rule.

## conflicts_naming_a_member
### [1]
#### ids
##### [1]
J3-02

##### [2]
J6-12

**conflict:**
Where a function's loan ends. J3-02 A: 'A loan stays the default; `owns` before a named function hands it over to any `owns` parameter, the library's included'. `thread.spawn(push_a)` is refused because the function is kept, and 'return xs.iter().map(owns h) works the same way'. J6-12 A instead draws the line by parameter kind: 'A function from a variable is still lent, with no word, to a standard function that takes a function as its function parameter (`for_each(h)`, `map(h)`, `retain(keep)`)'. By that criterion `thread.spawn(push_a)` (a function parameter) would be lent, and so would a returned `map(h)`, against J3-02.

**resolution:**
One rule for both: a function from a variable passed with no word is lent (verdict 150). A loan the callee keeps past the call is refused at the call, with the help `owns h` or `owns copy of h`. The kept cases are a thread (`thread.spawn`), an element (`push`) and an adapter returned or stored (`return xs.iter().map(h)`). `owns` hands the function over (verdict 147). `for_each(h)` and `xs.iter().map(h).sum()` stay loans. Reword J6-12 A's sentence to 'lent to a call that runs it during the call'.

## verdicts_involved
**3:**
JOB.md 1330-1331: 3. **If it cannot be saved, discard it**, and say why: the measurement or the counterexample that no
   fix removes.

**51:**
JOB.md 408-412: 51. **A long lambda may be written inside the call or as a named local function passed by its name; both
    are allowed** (the survey: thread and C lambdas). `a = thread.spawn(fn() with thread_shared log
    changes Log: log.push("a"))`, or `fn push_a() with thread_shared log changes Log:` / `log.push("a")`
    then `a = thread.spawn(push_a)`; a local function captures as a lambda does. Owner: "I think both ways
    should be permissible".

**80:**
JOB.md 599-605: 80. **A function that outlives the function that made it takes the variables it keeps with `with owns`**
    (open question 17: R12's placement of a real variable in the function's own state, against E3026).
    `return fn() changes n: n += 1` in `make_counter`, whose local `n` ends when it returns, is refused
    (E3026), the message giving `return fn() with owns n:`. With `with owns`, `n` is the function's own
    state, and changing its own state needs no `changes` word (R12), so the returned type is a plain
    `fn()`: `fn make_counter() -> fn():`. Without `with`, a function only borrows (the owner's own
    summary of `with`). Owner: "b", over (a), the compiler moving `n` in by itself as Python does.

**84:**
JOB.md 627-633: 84. **`owns` hands over every type the same way, numbers included** (step 3's readability check: verdict
    80's `owns n` on an `int` against E2085). After `owns count`, the one who gave it can no longer use
    `count`; the receiver uses it under the same name. `copy of` copies, and the giver keeps its own, for
    every type. E2085 (`owns` on a `Copy` type) and its "write `copy of`" messages are removed; verdict 80's
    `return fn() owns count -> int:` stands as written. The machine copies the number either way, as fast
    as C. Owner: "EVERY OTHER THING HAS TO BE OWNED TO TRANSFER WHY THE FUCK INT GETS THE SPECIAL
    TREATMENT?", then "yes".

**94:**
JOB.md 725-730: 94. **`thread.spawn` takes a `thread_shared fn()`** (area E's item 4; verdict 61 applied to the library's
    own line). `pub fn spawn[R](owns f: thread_shared fn() -> R) -> JoinHandle[R]:`; calls are written as
    before (`h = thread.spawn(fn() copy of r: worker(r))`), and a function holding a one-thread owner is
    refused by verdict 61's general rule (E7015) instead of a rule of `spawn`'s own. `scope.spawn` keeps
    a plain `fn()`: its tasks may use the caller's own variables (`total`, `ref t`), since they all end
    with the `with scope` block. Owner: "yes".

**145:**
JOB.md 1141-1144: 145. **`owns` hands over every value `=` copies, as it does a number** (N31): `flag = true` then `f = owns
    flag` gives `f` the value and `flag` ends, so `println(flag)` is `E3040`; the same for `char`, `StrView`,
    `ref T`, `Span[T]` and a struct made only of these. Owner (2026-11-01), shown it for a `bool` beside
    E0109 "remove `owns`": "yes noext question".

**146:**
JOB.md 1145-1148: 146. **`owns` before an argument for a parameter that only reads is an error** (N32): with `fn show(xs:
    Array[int])`, `show(owns xs)` is refused, "remove `owns`". Owner (2026-11-01): "yes pick a, next
    question". (The reason the lead gave with it, that `owns` is never written before an argument, is
    overturned by verdict 147.)

**147:**
JOB.md 1149-1156: 147. **An argument for an `owns` parameter says `owns` at the call, whatever is passed**: with `fn take(owns
    xs: Array[int])`, `take(owns xs)` hands `xs` over and `take(xs)` is an error; a copy is `take(owns copy
    of xs)`, which keeps `xs`; a list made in the call is `take(owns [1, 2])`. Owner (2026-11-01), shown his
    rule of 2026-10-24 ("For every name, point at the word in the line that says what it is") and verdict 3
    (`bump(ref mut a)`): "YES"; of `owns copy of xs`: "I think owns copy of will make things more clear"; of
    `take(owns [1, 2])`: "yes". The spec's "An argument for an `owns` parameter is handed over with no word"
    (`[FN-2a]`, `[OWN-9]`) and "the caller passes `copy of x` to keep its own, or `x` to hand it over"
    (`[FN-2]`) were never the owner's and go.

**148:**
JOB.md 1157-1160: 148. **`return xs` takes no word; `return owns xs` is an error, "remove `owns`"; a function that returns a
    copy keeps its plain result type**: `fn dup(xs: Array[int]) -> Array[int]: return copy of xs`. Owner
    (2026-11-01): "just return"; of the result type, beside `-> ref mut int` and `-> shares Array[int]`:
    "keep it Array[int]"; of `return owns xs`: "remove owns".

**149:**
JOB.md 1161-1163: 149. **`push` too**: `names.push(owns p.name)` (verdict 78's line), `Array`'s built-in `push` included;
    `names.push(p.name)` is an error. Owner (2026-11-01), shown `SmallArray.push(owns value: T)`: "yes owns
    at the call for push too".

**150:**
JOB.md 1164-1168: 150. **A function passed from a variable to a standard-library function is borrowed; the caller keeps it**
    (ODR-240 of 2026-10-22 stands): `h = fn(x: int): println(x)`, `[1, 2].iter().for_each(h)` prints 1, 2,
    and `h(5)` prints 5. `[STD-19]`'s "The function is handed over as every argument for an `owns`
    parameter is" was the lead's (0.9.15_Hardened_1, drawn from verdicts 38 and 84) and goes. Owner
    (2026-11-01): "SO IT BORROWS RIGHT?", then "YES GOOD".

**158:**
JOB.md 1210-1215: 158. **A function written in a call to a standard-library function says `owns` too** (verdict 147 for a new
    value, beside verdict 150 for a named one): `fn for_each(owns f: F) owns this` is called
    `[1, 2].iter().for_each(owns fn(x: int): println(x))`, `evens = xs.iter().filter(owns fn(x: int) -> bool:
    x % 2 == 0)`, `return xs.iter().map(owns fn(x: int) -> int: x * 2)`; a function from a variable stays
    `for_each(h)`, borrowed. Owner (2026-11-01), shown the three lines and (b) no word for a function written
    in the call: "a".

**159:**
JOB.md 1217-1221: 159. **A constructor's argument takes no word: it works as `=` does** (verdict 147 does not reach a
    constructor): with `struct Words:` holding `names: Array[String]`, `w = Words(names=xs)` hands `xs` over
    (a later use is `E3040`), as `snapshot = items` does, and `Point(x=a)` copies a number; `owns` stays
    for a value made for an `owns` place (verdicts 74, 85, 97: `Entity(mesh=owns Mesh(...))`). Owner
    (2026-11-01), shown `Words(names=xs)` beside `Words(names=owns xs)`: "b".

**160:**
JOB.md 1223-1226: 160. **A value made with `shares` or `weakref` in the call takes no `owns` before it**: with `fn
    setup(owns game: shares mut Game):`, the call stays `setup(shares mut game)`, and a list of links takes
    `ROSTER.push(weakref this)`; the word that makes the value says what is given. Owner (2026-11-01), shown
    `setup(owns shares mut game)` and `ROSTER.push(owns weakref this)` beside them: "yes go with b".

**161:**
JOB.md 1228-1232: 161. **`owns` at a call binds as `-` does; an argument of more than one name, number or call takes brackets**:
    `tx.send(owns (i * i))`, `m.insert(owns (i * 7), owns i)`; `tx.send(owns i * i)` is `(owns i) * i` and is
    refused, with the bracketed line as its help. The grammar already says so (`unary := value_word unary`).
    Owner (2026-11-02), shown (a) brackets and (b) `owns` covering the whole argument at a call, with the
    recommendation (a): "yes a".

## allowed_rules
### [1]
CLO-12

### [2]
CLO-7

### [3]
FN-2a

### [4]
GRM-15

### [5]
GRM-28

### [6]
OWN-9

### [7]
STD-19

### [8]
THR-10

## rule_text
**CLO-12:**
p06-expressions.md lines 329-333
* `[CLO-12]` *(new in 0.9.9)* *(changed in 0.9.15)* A local function (`[GRM-28]`) is a named lambda: it takes the same
  capture words after its brackets (`fn push_a() thread_shared log:`), follows `[CLO-1]` to `[CLO-6]`,
  and may be passed by its name, so a long lambda may be written inside the call
  (`a = thread.spawn(owns fn() thread_shared log: log.push("a"))`) or as a local function
  (`fn push_a() thread_shared log:` with the body `log.push("a")`, then `a = thread.spawn(push_a)`).

**CLO-7:**
p06-expressions.md lines 318-326
* `[CLO-7]` *(changed in 0.9.9)* *(changed in 0.9.10)* *(changed in 0.9.15)* Standard-library functions
  that keep a function beyond the call, or run it on another thread, take it `owns`: `thread.spawn` and
  `jobs.submit` take `owns f: thread_shared fn() -> R` (`[THR-10]`, `[JOB-2]`), a callback registry an
  `owns` function value, and `scope.spawn` a plain `owns f: fn() -> R`, whose tasks may use the
  caller's own variables since they end with the `with scope` block (`[THR-11]`). Nothing is captured by
  a rule of the call: the function given says after its brackets what it holds
  (`thread.spawn(owns fn() owns tx: produce(tx))`), a name with no word is the real variable, and a function
  that may outlive a real variable it uses is `E3026` (`E7004` for an unscoped thread or a kept C
  callback, whose message names the words, Part XI).

**FN-2a:**
p05-declarations.md lines 115-140
* `[FN-2a]` *(changed in 0.9.15)* **A call says `ref mut` before an argument the callee may change,
  and `owns` before one it takes over**, with the same words as the parameter, `const` included:
  `bump(ref mut a)` for `fn bump(ref mut c: Counter):`, `blend(const ref mut d, s)` for
  `fn blend(const ref mut dst: MutSpan[float], src: Span[float]):`, `mem.take(ref mut this.items)`;
  `take(owns xs)` for `fn take(owns xs: Array[int]):`, whatever is given: `take(owns copy of xs)` gives a
  copy and keeps `xs`, `take(owns [1, 2])` a list made in the call, `names.push(owns p.name)` a part
  (`[OWN-11]`), and a function written in the call is given the same way,
  `xs.iter().for_each(owns fn(x: int): println(x))` (`[STD-19]`). A missing word, or one written where
  the parameter does not say it, is `E3028` ("`bump` may change `a` (its first line says
  `ref mut c: Counter`); say so at the call: `bump(ref mut a)`"; "`show` only reads its argument;
  remove `ref mut`: `show(a)`"; "`take` takes `xs` over (its first line says `owns xs: Array[int]`); say
  so at the call: `take(owns xs)`", with the help "to keep `xs`, give it a copy:
  `take(owns copy of xs)`"; "`show` only reads its argument; remove `owns`: `show(xs)`"), and `ref mut`
  before something this function may not change (a plain parameter, a temporary, a place reached
  through a read-only name) is `E3027` ("`p` only reads the caller's player, so it cannot be given to
  `heal` as `ref mut`; take it as `ref mut p: Player` on `revive`'s first line, or walk the list with
  `for ref mut p in players:`"). `return a` and a constructor's argument take no word: they hand a
  value over as `=` does (`[OWN-9]`). A function from a variable given to a standard-library function
  that takes one is lent and takes no word: after `xs.iter().for_each(h)`, `h` is the caller's still
  (`[STD-19]`). An argument for a `Mutex`, `RwLock` or `Atomic` parameter, or a channel end, takes no word, since its own "only me" is
  the lock's block (`E3028`: "`top` takes the lock of `scores` itself; a `Mutex` takes no word at the
  call: `top(scores)`"). The object a method is called on is not marked (`a.reset()`): what the method
  does to it is worked out from its body (`[EXC-15]`). The arguments of one call do not overlap a
  changeable one (`[BRW-11]`), and a call is checked against what the callee does with each view
  argument (`[BRW-21]`). The editor shows a parameter's word as an inlay hint where the call does not
  write one.

**GRM-15:**
p03-grammar.md lines 389-405
* `[GRM-15]` *(changed in 0.9.10)* *(changed in 0.9.15)* `owns e` hands `e` over. It is written, and required, on the line
  that makes a value for a place that owns it alone (`mesh=owns Mesh(points=[0.0])`,
  `left=Some(owns Tree(value=2))`, `e.mesh = owns Mesh(points=[1.0, 2.0])`; `[TYP-45]`, `E2123`);
  before the iterable of a `for` or of a comprehension's `for` clause (`[GRM-27]`) and before the
  scrutinee of a `match`, where the default is a view, so that the loop or the branches take the value
  over (`for x in owns xs:`, `match owns e:`); before an argument for an `owns` parameter, whatever is
  given (`take(owns xs)`, `take(owns copy of xs)`, `take(owns [1, 2])`,
  `xs.iter().for_each(owns fn(x: int): println(x))`; `[FN-2a]`, `E3028` without it); and in
  `b = owns n` for a value `=` would copy (a number, a `bool`, a `StrView`, `[OWN-9]`), so that `n` ends
  there. Before a name whose place already hands the value over with no word (`e.mesh = m`, `b = a` for
  a value `=` hands over, `return a`), and before `fn` anywhere but an argument for an `owns` parameter,
  it is `E0109`, whose message says to remove it (for `owns fn`, to name what the function takes on each
  captured name: `fn() owns report:`). Before an argument for a parameter that only reads
  (`show(owns xs)`) it is `E3028`, "`show` only reads its argument; remove `owns`: `show(xs)`". `owns`
  binds as `-` does (`unary := value_word unary`), so an argument of more than one name, number or call
  is written in brackets, `tx.send(owns (i * i))`: `tx.send(owns i * i)` is `(owns i) * i`, a value made
  after `i` is handed over, and is `E3028`, whose help writes the brackets (`tx.send(owns (i * i))`).

**GRM-28:**
p03-grammar.md lines 248-253
* `[GRM-28]` *(new in 0.9.9)* *(changed in 0.9.15)* A `fn_decl` inside a block declares a **local function**. It may capture locals of the
  enclosing function exactly as a lambda does (`[CLO-2]`), with the same capture words after its
  brackets (`fn push_a() thread_shared log:`), may call itself recursively, and is in scope from its
  declaration to the end of the block. It is a value of its function type and may be passed by its
  name, so a long lambda may be written inside the call or as a local function passed by its name
  (`a = thread.spawn(push_a)`).

**OWN-9:**
p07-ownership.md lines 91-103
* `[OWN-9]` *(new in 0.9.15)* **`owns` hands over, for every type, numbers included.** An argument for
  an `owns` parameter says `owns` at the call (`[FN-2a]`): `f(owns n)` ends `n` when `f` takes
  `owns x: int`, and `f(owns copy of n)` keeps the caller's own. `return a` hands `a` over with no word,
  and so does a constructor's argument, as `=` does: `w = Words(names=xs)` hands `xs` over, `Point(x=a)`
  copies `a`. A value `=` copies (a number, a `bool`, a `char`, a `StrView`, a `ref T`, a `Span[T]`, a
  struct made only of these) is copied with `=` too, so `b = owns n` is how `=` hands one over: `n` ends
  there, and a later use is `E3040` (`f = owns flag` ends `flag`). `owns` is not written before a name
  whose place already hands the value over, after `return`, nor before `fn` written on its own
  (`E0109`: "`e.mesh = m` already hands `m` over; remove `owns`"; "`b = a` already hands `a` over; remove
  `owns`"; "`return` always hands the value over; remove `owns`: `return xs`"; "`owns fn` does not say
  what the function takes or copies; name it on each captured name: `fn() owns report:`"); before a
  function written as an argument for an `owns` parameter it is the call's word
  (`xs.iter().for_each(owns fn(x: int): println(x))`).

**STD-19:**
p15-stdlib.md lines 289-391
* `[STD-19]` *(new in 0.9.9)* *(changed in 0.9.10)* *(changed in 0.9.15)* Every `Iterator` has the adapters `map`, `filter`, `filter_map`,
  `enumerate`, `zip`, `chain`, `take`, `skip`, `take_while`, `skip_while`, `step_by`, `flat_map`,
  `flatten`, `peekable`, `copy_each`, `inspect`, and `rev` where the iterator can run backwards;
  and the consumers `count`, `sum`, `product`, `min`, `max`, `min_by_key`, `max_by_key`, `fold`,
  `reduce`, `any`, `all`, `find`, `position`, `last`, `nth`, `min_by`, `max_by`, `for_each`,
  `collect[C]()`, `to_array()`,
  and `join(sep)` for `Display` items. `copy_each()` gives, for each item, what `copy of` the item
  would make (for `Item: needs Clone`): `names = sorted(m.keys().copy_each())` is a list of strings of its
  own, kept past changes to `m`; `.copied()` and `.cloned()` are not Ember (`E1010`, "write
  `.copy_each()`"). Adapters are lazy and allocate nothing themselves; what the functions they apply
  do counts where they run (`[EFF-1]`). A chain of adapters in a
  `for` loop compiles to one loop with no iterator object in memory (`[CTL-3]`). The adapters and
  consumers can also be called directly on an `Iterable` value, reading it as `iter()` would:
  `xs.enumerate()` is `xs.iter().enumerate()`, and a method of the collection's own of the same name
  (`Array.join`) takes precedence. An adapter takes the iterator it wraps over (`owns this`), and so
  does a consumer; `zip` and `chain` take their other iterator over too, so a named iterator is given
  to `zip` with `owns` (`a.zip(owns it)`; `a.zip(owns copy of it)` keeps the caller's). **A function is taken as a type
  parameter**, by the adapters that apply one and the consumers alike (`any`, `all`, `find`,
  `position`, `fold`, `reduce`, `for_each`, the `…_by` and `…_by_key` forms): `[F: needs fn(Item)]` above
  `fn for_each(owns f: F) owns this:`, `[F: needs fn(Item) -> bool]` above `fn any(owns f: F) owns this ->
  bool:`, `[F: needs fn(Item)]` above `fn inspect(owns f: F) owns this -> Inspect[Self, F]:`. `F` names the
  exact function given, so the adapter or consumer is compiled once for each function given to it and
  the call through `f` is direct and may be inlined; the shared type, `owns f: fn(Item)`, is legal too,
  and is what keeps functions of different kinds in one list (`handlers: Array[fn()]`). A function
  written in the call is given to it, with `owns` as every argument for an `owns` parameter is
  (`[FN-2a]`), so an adapter made from a lambda can be kept or returned
  (`return xs.iter().map(owns fn(x) => x * 2)`), what the lambda holds with it. A function from a
  variable is lent to the call and takes no word: after `xs.iter().for_each(h)`, `h` is the caller's
  still and `h(5)` runs it (ODR-240). A function given may change
  things: a consumer's call counts as making what the function changes, now
  (`scores.for_each(owns fn(s): LOG.push(owns copy of s.name))` changes `LOG`, so inside a loop over `LOG` it is
  `E3020`), and an adapter keeps the function and runs it at each step, so whatever drives the adapter
  (`for x in it:`, `it.sum()`) makes its changes; driving an `owns dyn Iterator[Item = T]` makes the
  changes of every adapter the program can put there. What a caller
  passes is an `int` (ODR-089): `take(n)` gives at most `n` items and
  `skip(n)` leaves out the first `n`, `nth(n)` is the item at index `n` (`None` past the end),
  `step_by(k)` gives the first item and then every `k`-th, and `enumerate(start=0)` numbers from
  `start` as `[STD-26]`'s `enumerate` does. A negative count or index, or a `k` of zero or less,
  is the caller's bug and panics when the adapter is made or the consumer called (`[STD-14]`),
  never partway through. **Counts and numbers** (the owner's design G8-4, 0.9.10): what Ember
  hands back is in the smallest type that holds every value the iterator can give. `Iterator` has
  the associated types `Count`, what `len()`, `count()` and `position()` give, and `Position`, what
  `enumerate` numbers its items with; each is `int` unless the iterator says otherwise, and each
  implements `std.core.ItemCount` (`int`, `u64`, `u128`, `u256`, `i128` and `i256` do), which
  gives the count operations (`of`, `plus`, `less`, `exceeds`, `to_int`, and `as_count[C]()` from
  one count type to another that holds it). Stored items (arrays, views, text, maps, sets) and
  ranges of 8-, 16- or 32-bit numbers count and number in `int`; a `..` range of 64-bit numbers
  counts in `u64` and a `..=` one in `u128`, a `..` range of 128-bit numbers in `u128` and a `..=`
  one in `u256` (`[TYP-42]`); ranges of 64-bit numbers number in `i128` and of 128-bit numbers in
  `i256`, from a start that may be negative. An adapter that never gives more items than it takes
  counts and numbers as what it wraps. `chain` counts its two sides together, stepping up only as
  far as needed: two `int` counts stay an `int` (stored items and small ranges cannot come near its
  top), otherwise the count is one level above the wider side's (`u64`, then `u128`, then `u256`),
  and it numbers in the type that holds both sides' numbers; `flatten` and `flat_map` count in the
  level that holds the outer count times the inner one (an `int` by an `int` stays an `int`).
  `u256` is the limit: a count past it panics. In generic code these are the counts of the types
  involved, filled in for each use. **The known-numbers rule** (the owner's rulings of 2026-10-06,
  which widen the visible-numbers rule): where `enumerate` is called, and where `len` or `count` is
  called on an iterator, the numbers it gives, and the count, are in the smallest of `int`, `u64`,
  `i128`, `u128` and `i256` that holds every value it can give, read from the numbers the compiler
  knows. It knows a number written as one; a name set once and never changed anywhere in its
  function (no assignment, augmented assignment, `ref mut` name or argument of it, or call of a method
  that changes it, in the function or a lambda in it) holds what its value held, and a range or an `Array`
  literal so set has its bounds or its length; a loop's counter (`for i in a..b`, `range(a, b,
  step)`, and a range's value through `rev`, `skip`, `take` and `step_by`) holds its range's
  numbers; `+`, `-`, `*`, `//` and `%` give what their known operands allow; a list's, a view's or
  a text's length is at most what its items' bytes allow (`[HEAP-8]`: at most `PTRDIFF_MAX` bytes,
  an item of no size counting as one) and a fixed array's is its `N`; `take`, `skip`, `step_by`,
  `zip` and `chain` give what their known arguments allow. Anything else may be any value of its
  type, and an iterator gives at most what its `Count` holds. `enumerate` numbers from `start` to
  `start + n - 1` over at most `n` items, so its type steps up from the iterator's own `Position`
  where the numbers could pass that type's top (`ys.enumerate(start=int.MAX - 1)` over three items
  numbers in `u64`): no number of an `enumerate` overflows. In generic code an `enumerate` numbers
  in its iterator's `Position`; where what it numbers in depends on the item type (a list of `T`,
  whose most items depend on `T`'s size), the function's own check leaves it, and every line that
  uses those numbers, to the check of each use (`[TYP-17]`), which numbers by what it knows. A use
  that a line does not fit is refused there, naming the use (`in labels used with T = u8`); with
  `[TYP-5]` rule 12, `last = i` takes the numbers of a list of `int`s from 2 whether `last` is an
  `int` or a `u64`. An iterator can run backwards when it implements
  `DoubleEndedIterator`, whose method `fn next_back() -> Option[Item]` gives the last item not yet
  given from either end (with `next`, the two ends meet and no item comes twice) and whose method
  `rev` is, so `rev` is exactly where an iterator can run backwards; `ExactSizeIterator`'s
  `fn len() -> Count` is how many items are left (ODR-091). `skip_front(n)` and `skip_back(n)`
  leave out up to `n` items (a `Count`) from either end and give how many of the `n` they could
  not, and `nth_back(n)` leaves out `n` from the back and gives the one before them; a range does
  each at once, and `rev`, `copy_each`, `take`, `skip`, `zip`, `chain`, and `enumerate` from the back
  pass them down, so none of them passes a gigantic run of items one at a time. `ItemCount`,
  `DoubleEndedIterator` and `ExactSizeIterator` are in `std.core` and are not prelude names
  (`[MOD-5]`). The iterators of the ranges with an end and the element iterators of the views run
  backwards and know their length; `copy_each` and `chain` run backwards when what they wrap
  does, and `take`, `skip`, `step_by`, `enumerate` and `zip` when it also knows its length, giving
  the items `next` would, last first; each knows its length when what it wraps does. Run backwards,
  `enumerate` numbers its last item `start + len - 1` first, and the items `skip_back` leaves out
  need no numbers. `collect[C]()` builds any `C` that implements `FromIterator[Item]` (ODR-094), an interface of
  `std.core` that is not a prelude name: `interface FromIterator[T]:` declares
  `[I: needs Iterator[Item = T]]` above `static fn from_iter(owns it: I) -> Self`. `Array[T]`, `Set[T]`, `Map[K, V]` (from `(K, V)` pairs, a later pair's
  value replacing an earlier one's for the same key) and `String` (from `char`s and from `String`s)
  implement it, and so may a program's own types. `C` is written (`it.collect[Array[int]]()`) or is
  the type the call is expected to give (`a: Array[int] = it.collect()`). `flatten` is for items that
  are themselves iterable (`IntoIterator`). `peekable()` gives the same items with
  `fn peek() -> Option[ref Item]` and `fn peek_mut() -> Option[ref mut Item]`, which take the next item early
  and keep it where it is until `next` gives it; `next_if(pred)` gives the next item when `pred` holds
  for it and leaves it otherwise, and `next_if_eq(v)` when it equals `v` (`Item: needs Eq`).

**THR-10:**
p11-concurrency.md lines 173-182
* `[THR-10]` *(new in 0.9.9)* *(changed in 0.9.10)* *(changed in 0.9.15)* `thread.spawn` runs a function on
  a new thread, which may outlive the function that started it. It takes a function any thread may run,
  `[R]` above `pub fn spawn(owns f: thread_shared fn() -> R) -> JoinHandle[R]:`, so a function that holds
  a one-thread value is `E7015` (`[THR-1]`, `[THR-2]`), and `R` MUST be `Send` (`E7015`, `E7008`). What
  the function holds from its maker is written in its capture list (`[THR-17]`). `h.join() -> R` waits
  and returns the result. Dropping a `JoinHandle` detaches the thread. **Starting a thread counts as
  making everything its function changes, now or later** (`[EFF-26]`): a `thread.spawn` inside a walk of
  what the thread changes is `E3020` ("start it after the loop"), except where the thread makes that
  change under a lock the walk holds, when the new thread waits for the lock until the loop ends; joining
  such a thread inside that lock is `E7018` (`[THR-19]`).

## rules_not_found
