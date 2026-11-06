# K18-string-plus (input for method 3; made by deep/v3/make_inputs.py from phase2.json, phase3.json, deep/phase1-groups.json, JOB.md and the part files)

## cluster
**cid:**
K18-string-plus

**title:**
Does String + read its left side and make a new String, or hand it over with a word?

### members
#### [1]
J2-04

#### [2]
J6-14

**primary:**
J2-04

**why_together:**
The same question, found twice. J6-14 says the 'outside Add' half is settled by TYP-21's exception, and J2-04 asks to remove that exception. Both recommend A.

**severity:**
high

**impact_rank:**
18

**notes:**
Very common code. Use J6-14's short program to open and J2-04's generic `with_suffix` line to show the Add bound. One B spelling: `(owns label) + "!"` (by verdict 161 it is the same as `owns label + "!"`, but it reads unambiguously). See the recommendation change: A should reuse a temporary left operand.

## questions
### [1]
**id:**
J2-04

**verdict:**
confirmed

**route:**
owner_odr

**severity:**
medium

**confidence:**
0.8

**reason:**
TYP-21 (p04 745-746) says a type has an operator only by implementing its interface, and (p04 751-752) that String implements none of the operator interfaces, while TXT-11 (p15 163-164) gives String a `+` that 'takes `a` over' and a `+=`. So String's `+` is a path no interface declares, a String fails an `Add` bound while `a + b` on it compiles, and TYP-21's own desugaring (p04 741-742: 'both operands read in place') is broken by a `+` that takes its left side over with no `owns` written, which verdicts 84, 145 and 147 require of every hand-over. No verdict decides String's `+`.

**title:**
Is String's `+` an ordinary `Add`, and does it take the left text over?

**problem:**
A HUD builds labels with `label + "!"` and `label += "10"`, and a generic helper bounded by `Add` adds a suffix to any text type. Today String's `+` is built in, outside the operator interfaces every other type goes through, so the helper refuses a String; and `label + "!"` silently hands `label` over, so a later `println(label)` is refused though the line never says `owns`.

**example:**
[T: needs Add[StrView, Output = T]]
fn with_suffix(a: T, b: StrView) -> T:          # any type that `+` can extend with text
    return a + b

fn main():
    label = String("hp: ")
    label += "10"                                # appends in place
    shown = label + "!"                           # today: takes `label` over, with no word
    println(label)                               # today: E3040, `label` was handed over
    tagged = with_suffix(shown, "?")              # today: E2040, String implements no Add
    println(tagged)

**spec_today:**
TYP-21 (p04 741-742): "Operators desugar to these interfaces for non-scalar operands; `a + b` calls `Add.add(a, b)` with both operands read in place." TYP-21 (p04 745-746): "A type has an operator only by implementing its interface (`[TYP-40]`); a method that merely shares the name is not one." TYP-21 (p04 751-752): "`bool`, `char` and `String` implement none of them (`String`'s `+` takes its left operand over, `[TXT-11]`)." TXT-11 (p15 163-164): "`a + b` for `a: String, b: StrView` takes `a` over and returns it extended; `s += t` appends. `StrView + StrView` is `E2040` whose help is an f-string."

#### options
##### [1]
**letter:**
A

**summary:**
String implements `Add[StrView, Output = String]`, reading both sides and giving a new String (as Python and C++'s `std::string` `+` do), and `AddAssign[StrView]`, appending in place. Its `+` goes through the interfaces like every other type's.

**code:**
shown = label + "!"          # a new String; `label` stays
label += "10"                # appends in place, no new String

**outcome:**
Compiles; prints `hp: 10`, then `hp: 10!?`. `with_suffix` accepts a String. Each `+` makes one new String (one allocation and copy); `+=` in a loop costs what C++'s `append` costs.

##### [2]
**letter:**
B

**summary:**
String implements `Add` with its `add` taking its object over (Rust's way, no copy), and the line says so with the word every hand-over uses.

**code:**
shown = (owns label) + "!"   # `label` handed over and extended in place
label = String("hp: ")       # `label` must be given a value again before use

**outcome:**
`shown = label + "!"` without `owns` is refused, its help giving `(owns label) + "!"`; `println(label)` after it is E3040. No copy is made. `with_suffix` must take `owns a: T`.

##### [3]
**letter:**
C

**summary:**
Keep String's `+` built in, outside the operator interfaces, taking its left side over with no word.

**code:**
shown = label + "!"          # `label` silently handed over

**outcome:**
As today: `println(label)` after it is E3040 and `with_suffix(shown, "?")` is E2040.

#### recommendation
**letter:**
A

**reason:**
It removes the only operator that bypasses its interface and the only hand-over not written with `owns`, and matches TYP-21's own 'both operands read in place'. `+=` keeps the fast in-place path; B is faster for long `+` chains but makes every label line read `(owns label) + ...`.

**checklist:**
2 (correct generic code accepted), 4 (nothing handed over without a word), 5 (one operator rule for every type), 10 (Python's and C++'s meaning), 13 (no hidden built-in path)

### [2]
**id:**
J6-14

**verdict:**
confirmed

**reason:**
TYP-21 (p04 741-742; spec 1841-1842) settles that String's `+` is not `Add` ("`String`'s `+` takes its left operand over, `[TXT-11]`"), so the 'outside Add' part is refuted. Still open: TXT-11 (p15 163-164) hands the left operand over with no word on the line, while every other hand-over is written with `owns` (FN-2a, the owner's rule that a word in the line says what each name is), and every other operator reads both operands in place (TYP-21). So `println(first)` after `first + " Lovelace"` is E3040 with no word to point at, and `return name + "!"` on a read-only parameter is refused with no stated reason. No verdict speaks of operators.

**route:**
owner_odr

**severity:**
medium

**confidence:**
0.7

**title:**
Should String + read its left side, or hand it over with a word?

**problem:**
A programmer builds a name with `full = first + " Lovelace"`. Today `+` on a String silently takes `first` over, so a later `println(first)` is an error although no word on the line says `first` was handed over, and `+` on a read-only parameter is refused. Every other operator in the language only reads its operands.

**example:**
fn greet(name: String) -> String:
    return name + "!"                  # `name` only reads the caller's string: may `+` take it over?

fn main():
    first = String("Ada")
    full = first + " Lovelace"         # no word says `first` is handed over
    println(full)
    println(first)                     # E3040, or "Ada"?
    println(greet(full))

**spec_today:**
p15 163-164 (TXT-11): "`a + b` for `a: String, b: StrView` takes `a` over and returns it extended; `s += t` appends." Spec 1831-1832 (TYP-21): "Operators desugar to these interfaces for non-scalar operands; `a + b` calls `Add.add(a, b)` with both operands read in place." Spec 1841-1842: "`bool`, `char` and `String` implement none of them (`String`'s `+` takes its left operand over, `[TXT-11]`)." p05 115-116 (FN-2a): "A call says ... `owns` before one it takes over".

#### options
##### [1]
**letter:**
A

**summary:**
`+` reads both sides and makes a new String

**code:**
full = first + " Lovelace"
return name + "!"

**outcome:**
Both compile; prints "Ada Lovelace", "Ada", "Ada Lovelace!". One allocation, as an f-string makes; adding in place stays `s += t`. String then implements `Add` like other types, and the exception in the operator rule goes.

##### [2]
**letter:**
B

**summary:**
`+` takes its left side over, written with `owns`

**code:**
full = owns first + " Lovelace"

**outcome:**
Compiles and reuses `first`'s buffer; `first + " Lovelace"` without `owns` is an error giving this line; `println(first)` after it is E3040; `greet` must take `owns name: String` and be called `greet(owns full)`.

##### [3]
**letter:**
C

**summary:**
As today: taken over with no word

**code:**
full = first + " Lovelace"

**outcome:**
Compiles; `println(first)` is E3040; `return name + "!"` is refused because a read-only parameter cannot be handed over.

#### recommendation
**letter:**
A

**reason:**
A makes `+` mean the same for every type (operands read in place), needs no hidden hand-over, and costs what C++'s `a + b` on two named strings costs; the in-place append is already spelled `s += t`.

**checklist:**
4 (no hidden hand-over), 5, 6 (the operator rule without an exception), 7, 11, 13

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
J2-04

##### [2]
J6-14

**conflict:**
The same option B is spelled two ways: J2-04 `shown = (owns label) + "!"` and J6-14 `full = owns first + " Lovelace"`. By verdict 161 ('`owns` at a call binds as `-` does') these mean the same, but the owner would see two spellings of one option.

**resolution:**
Merge into K18 with one spelling, the bracketed `(owns label) + "!"`, which cannot be misread as `owns (first + ...)`.

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
FN-2a

### [2]
OWN-9

### [3]
TXT-11

### [4]
TYP-21

### [5]
TYP-40

## rule_text
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

**TXT-11:**
p15-stdlib.md lines 160-180
* `[TXT-11]` *(new in 0.9.9)* *(changed in 0.9.15)* **`String` operations.** Everything `StrView` has (by read-through),
  plus `String()`, `String(s)` (a `String` of your own made from text), `with_capacity(n)`,
  `push(c: char)`, `push_str(s)`, `insert`, `remove`, `truncate`, `clear`, `capacity`, `reserve`, and
  `as_str` (a `StrView` of the whole text). `a + b` for `a: String, b: StrView` takes `a` over and
  returns it extended; `s += t` appends. `StrView + StrView` is `E2040` whose help is an f-string.
  ODR-098 defines the mutation and capacity methods: `String.with_capacity(n) -> String` makes an
  empty string with room for at least `n` bytes; zero allocates nothing. `capacity() -> int` reads
  the allocated byte capacity. `reserve(additional)` makes room for at least `len() + additional`
  bytes, without changing the text, and does nothing when that capacity already exists.
  `insert(i, c: char)` inserts one character at byte offset `i`, which must be in `0..=len()` and
  on a character boundary. `remove(i) -> char` removes and returns the character starting at byte
  offset `i`, which must be in `0..len()` and on a character boundary. Both close up or move the
  suffix as necessary; neither scans the prefix for a character position. `truncate(n)` retains
  the first `n` bytes when `n < len()` (then `n` must be a character boundary), and otherwise does
  nothing. `clear()` removes all text. `remove`, `truncate` and `clear` keep the capacity.
  These change the string, through a name that may change it, and return `void` unless a result is
  named above; each is a reshape, which ends every view into the text. Counts and
  offsets accept any integer type; a negative literal is `E2011`, and a negative run-time value or
  one not representable as a native length panics before conversion. An invalid offset or character
  boundary panics in every profile. Growth uses the checked allocation rules of `[HEAP-8]`.
  The arguments are evaluated before the string is changed, as for `Array`'s methods that change it.

**TYP-21:**
p04-types.md lines 741-753
* `[TYP-21]` *(changed in 0.9.9)* *(changed in 0.9.15)* Operators desugar to these interfaces for non-scalar operands; `a + b` calls
  `Add.add(a, b)` with both operands read in place. `a += b` calls `AddAssign.add_assign` if implemented,
  else `a = a + b`. Scalar operators are built in. The interfaces' methods are `add`, `sub`, `mul`,
  `div`, `floordiv`, `rem`, `pow`, `bitand`, `bitor`, `bitxor`, `shl`, `shr`, `neg` and `not`, and
  each `…Assign` form's is its operator's followed by `_assign` (ODR-040). A type has an operator
  only by implementing its interface (`[TYP-40]`); a method that merely shares the name is not one.
  Each number type implements the interface of each operator it has, with `Rhs = Self` and `Output =
  Self`: every integer type `Add`, `Sub`, `Mul`, `FloorDiv`, `Rem`, `Pow`, `Neg`, `Not`, `BitAnd`,
  `BitOr`, `BitXor`, `Shl`, `Shr` and their `…Assign` forms, and every float type `Add`, `Sub`,
  `Mul`, `Div`, `FloorDiv`, `Rem`, `Pow`, `Neg` and theirs, so `[TYP-17]`'s `sum` takes numbers.
  Such a method is the operator: `3.add(4)` is `3 + 4`. `bool`, `char` and `String` implement none
  of them (`String`'s `+` takes its left operand over, `[TXT-11]`). Likewise a type is indexed only
  through `Index`, `IndexMut` and `IndexSet` (ODR-042).

**TYP-40:**
p04-types.md lines 478-481
* `[TYP-40]` *(new in 0.9.9)* **Interfaces are nominal.** A type implements an interface only through an
  `implements` clause or an `extend … implements` block; having methods of the right names and types is
  not enough. This is what gives every implementation one place of declaration (`[TYP-20]`) and one
  method table for `dyn`.

## rules_not_found
