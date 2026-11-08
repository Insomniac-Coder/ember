# K19-written-init-call-words (input for method 3; made by deep/v3/make_inputs.py from phase2.json, phase3.json, deep/phase1-groups.json, JOB.md and the part files)

## cluster
**cid:**
K19-written-init-call-words

**title:**
Does a call of a class's own init say owns before an owns parameter's argument?

### members
#### [1]
J1-07

**primary:**
J1-07

**why_together:**
Single question.

**severity:**
high

**impact_rank:**
19

**notes:**
Every class with an init meets this. The draft must quote verdict 159 ('A constructor's argument takes no word: it works as `=` does') and say that A narrows it to constructors made from the fields (verdict clash). CLS-2's E2108 help and I.5's `Emitter("fountain")` depend on the answer.

## questions
### [1]
**id:**
J1-07

**verdict:**
confirmed

**route:**
owner_odr

**severity:**
high

**confidence:**
0.75

**reason:**
Verdict 159 was decided on a struct's memberwise constructor (`Words(names=xs)`): 'A constructor's argument takes no word: it works as `=` does (verdict 147 does not reach a constructor)'. FN-2a (p05 131-132) and OWN-9 (p07 93-94) say it of every constructor. CLS-2 (p05 313) and STR-6 (p05 253) call a written `fn init(...)` 'the constructor'. But a written init has a first line with its own parameter words, and FN-2a and GRM-15 require `owns` before an argument for an `owns` parameter. Neither rule says which wins for `Emitter(...)` when `init(owns name: String)` exists. With a plain init parameter, the 'as `=`' reading would hand over a value the init only reads, against FN-2 (p05 112: 'A parameter with no word reads'). I.5 (p01 195) writes `Emitter("fountain")` for `init(owns name: String)`, so it depends on the answer.

**title:**
Does a call of a class's own init say owns before an owns parameter's argument?

**problem:**
A class writes its own `init` with an `owns` parameter, and the program builds an object with `Emitter(title)`. Every function call says `owns` before an argument its `owns` parameter takes over, while a constructor's argument takes no word and works as `=` does. A written `init` is both, so the specification says two things for this line, and nothing for an `init` whose parameter only reads.

**example:**
class Emitter:
    name: String
    rate: f32 = 100.0

    fn init(owns name: String):          # this init takes the name over
        this.name = name

class Label:
    text: String

    fn init(text: String):               # this init only reads what it is given
        this.text = copy of text

fn main():
    title = String("fountain")
    fountain = Emitter(title)            # no word, as for a struct's constructor?
    caption = String("smoke")
    sign = Label(caption)                # does `caption` end here, as `=` would end it?
    println(fountain.name)
    println(sign.text)
    println(caption)

**spec_today:**
FN-2a (p05 115-118): "A call says `ref mut` before an argument the callee may change, and `owns` before one it takes over ... `take(owns xs)` for `fn take(owns xs: Array[int]):`, whatever is given"; FN-2a (p05 131-132): "`return a` and a constructor's argument take no word: they hand a value over as `=` does (`[OWN-9]`)". OWN-9 (p07 93-94): "and so does a constructor's argument, as `=` does: `w = Words(names=xs)` hands `xs` over". CLS-2 (p05 313): "`fn init(…)` is the constructor". FN-2 (p05 112): "A parameter with no word reads." I.5 (p01 176, 195): "fn init(owns name: String):" ... "fountain = Emitter(\"fountain\")".

#### options
##### [1]
**letter:**
a

**summary:**
A written init is called like any function, with its parameters' words

**code:**
fountain = Emitter(owns title)       # init says `owns name`
sign = Label(caption)                # init only reads `text`
w = Words(names=xs)                  # a struct with no init: as `=`, no word

**outcome:**
Compiles. `title` ends at the call, and `caption` stays the program's, so `println(caption)` prints smoke. `Emitter(title)` is E3028 ('say so at the call: `Emitter(owns title)`'). The overview's line becomes `Emitter(owns "fountain")`. A constructor made from the fields with no init keeps no word, as decided.

##### [2]
**letter:**
b

**summary:**
No owns at any constructor call; the init's parameter word decides what happens

**code:**
fountain = Emitter(title)            # handed over: init says `owns name`
sign = Label(caption)                # lent: init only reads `text`

**outcome:**
Compiles. `title` ends at the call and `caption` stays, so `println(caption)` prints smoke. `Emitter(owns title)` is refused ('remove `owns`'). A `ref mut` parameter of an init is still written `ref mut` at the call, as at every call.

##### [3]
**letter:**
c

**summary:**
Every constructor argument works as `=`, whatever the init's first line says

**code:**
fountain = Emitter(title)
sign = Label(caption)

**outcome:**
`title` and `caption` both end at their calls, although `Label`'s init only reads `text`, so `println(caption)` is E3040 ('`caption` was handed to `Label`').

#### recommendation
**letter:**
a

**reason:**
A written init has a first line with words, like any function, and the rule that the call repeats the parameter's word (`take(owns xs)`) was chosen so that each line shows what happens to each name. A constructor made from the fields has no first line to repeat, which is why its argument works as `=`.

**checklist:**
5 (one word, one meaning at every call), 6 (the exception holds only where there is no first line to repeat), 11, 15, 2

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
CLS-2

### [2]
FN-2a

### [3]
GRM-15

### [4]
OWN-9

### [5]
STR-6

## rule_text
**CLS-2:**
p05-declarations.md lines 313-330
* `[CLS-2]` *(changed in 0.9.9)* *(changed in 0.9.10)* *(changed in 0.9.15)* `fn init(…)` is the
  constructor; it sets the fields of the object it builds through `this`, with no word. Before any
  `init` body runs, every field default of the class and of its bases is evaluated and stored. Every
  field without a default MUST then be set on every path before `this` is used as a whole (`E2100`):
  `this` is used as a whole when it is passed, stored, returned or captured, when a link or an owner of
  it is made (`weakref this`, `shares this`), and by a call of a method whose body lets `this` out in
  any of those ways, which the compiler works out from that body (`[EFF-24]`). Before then `init` may
  call a method of its object once every field that method's body reads is set: after
  `this.renderer = r`, `this.assets = this.load_assets()` is allowed when `load_assets` reads only
  `this.renderer`, and is `E2100` when it reads a field not yet set ("`init` calls `load_assets()` (line
  6), which reads `this.assets` (its line 14), and `assets` is set only on line 7; give it a value
  first: write `this.assets = []` above line 6"); a method that lets `this` out is called only after
  every field is set ("`enrol` keeps a link to the player (`ROSTER.push(weakref this)`, its line 2), and
  `name` is set only on line 9; set every field first: move line 6 below line 9"). A `return` from
  `init`, like the end of its body, hands the object to its caller, so every such field is set on that
  path too. `owns this` on `init` is `E2108` ("`init` builds the object, so it cannot take it over;
  write `fn init(name: String):`"). Several constructors are written as `static fn`s that call
  `Self(…)`.

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

**STR-6:**
p05-declarations.md lines 253-256
* `[STR-6]` *(new in 0.9.9)* *(changed in 0.9.15)* `fn init(…)` on a struct is its constructor; it sets
  the fields through `this`, with no word. Every field without a default MUST be set on every path
  before `this` is used as a whole (`E2100`), as `[CLS-2]` says for a class. `owns this` on an `init` is
  `E2108`.

## rules_not_found
