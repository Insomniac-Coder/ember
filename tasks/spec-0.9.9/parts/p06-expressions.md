---

# Part VI — Expressions and Statements

## VI.1 Evaluation order

* `[EXP-1]` Operands, arguments, and the elements of tuple, list, map and set literals are evaluated
  left to right, completely, before the operation or call. Named arguments are evaluated in the order
  written.
* `[EXP-2]` *(changed in 0.9.9)* An assignment evaluates its right side first, into a temporary if it
  has several elements, then the target places, then stores left to right. `a[i], a[j] = a[j], a[i]`
  swaps. An augmented assignment `a[i] += x` evaluates `x`, then the place once, then reads, operates
  and writes.
* `[EXP-3]` `and` and `or` short-circuit. `x if c else y` evaluates `c` and then exactly one branch. A
  comparison chain stops at the first false link (`[GRM-25]`).
* `[EXP-4]` *(changed in 0.9.9)* *(changed in 0.9.15)* A temporary created while evaluating an
  expression statement is dropped at the end of that statement, in reverse order of creation, unless a
  view made from it is still to be used: it then lives until the last use of every view made from it,
  lists of views included (`word = line.to_lower().trim()` keeps the lowered string until `word`'s last
  use), and a view of it that would outlive it is refused (`[BRW-19]`, `E3063`). A temporary created
  while evaluating the **condition** of an `if`, `elif` or `while` is dropped before the chosen block
  runs, unless the condition names what it holds (`exists as`, a kind test, a pattern) and the name
  refers to it, in which case it lives to the end of that block only — never into an `elif` or `else`.
  A temporary bound by `with` lives to the end of the `with` block; one created by a `for` iterable
  lives to the end of the loop.

## VI.2 Places, values and hand-overs

A **place** denotes memory: a local, a field of a place, an element of a place, an enum variant's
payload, the place a `ref` or `ref mut` view names, the value an `owns` field holds, a field of a class
object, a `static`. A call whose result type is `ref T` or `ref mut T` is a place too
(`first(ref mut xs) += 1`, `counts.entry(k).or_default() += 1`, `[LT-1]`). Everything else is a value.

* `[EXP-5]` *(changed in 0.9.15)* A change, a `ref mut` name and a hand-over require a place;
  `f(x).y = 1` is `E2140` unless `f` returns `ref mut`. A method that may change its object, called on a
  temporary that ends with the line, is `E2140` too when that change is the call's only effect, since
  the change would be lost (`enemy_at(es, 0).heal()`, `[BRW-19]`); a call whose result is used stays
  allowed (`line.split(" ").next()`).
* `[EXP-6]` *(changed in 0.9.15)* A whole name of a type that is not `Copy`, used as a value (given with
  `=`, given to an `owns` parameter, returned, captured with `owns`), is **handed over**: it ends, and is
  uninitialised until it is given a value again (`[OWN-3]`); `owns` hands over a name of every type,
  numbers included (`[OWN-9]`). A part of a struct value is handed over on its own, and neither the part
  nor the struct used whole is used until the part has a new value (`E3040`, `[OWN-11]`). Handing over
  a field of a type with `drop` is `E3010`; an element of an array, `Array` or `Span` is `E3011` (use
  `mem.take`, `mem.replace`, `mem.swap`, `pop`, `remove`); a field of a class object, which other names
  may reach and would see the hole, and anything reached through a name that does not own it, are
  `E3013` (`[OWN-11]`).

## VI.3 Operators and literals

| Expression | Meaning | Notes |
|---|---|---|
| `a + b`, `a - b`, `a * b` | arithmetic | overflow panics (`[TYP-8]`) |
| `a / b` | true division | floats only; integers are `E2240` (`[TYP-28]`) |
| `a // b`, `a % b` | floor division and modulo | Python meaning |
| `a ** b` | power | `[TYP-30]` |
| `a == b`, `a != b` | `Eq` | built in for scalars |
| `a < b` etc. | comparison | scalars built in, in generic code too (a float's is IEEE, `[TYP-37]`); other types through `Ord.cmp`; chains (`[GRM-25]`) |
| `x in c`, `x not in c` | `c.contains(x)` | `Contains` (`[STD-8]`) |
| `a is b`, `a is not b` | the same object | class objects and views (`[EXP-9]`) |
| `x is None`, `x is not None` | option test | `Option` (`[EXP-9]`) |
| `x is T`, `if x is T as name:` | kind test | a value of an interface or base-class type (`[EXP-9]`, `[DSP-4]`) |
| `a[i]` | `Index` / `IndexMut` / `IndexSet` | read; written in place; `a[i] = v` per `[STD-17]` |
| `a[i..j]`, `a[..j]`, `a[i..]` | slice | `Span`/`MutSpan`/`StrView`, bounds-checked |
| `a?.f`, `a?.m()` | optional chaining | on `Option`: `None` propagates (`[EXP-10]`) |
| `e?` | early return | on `Option` or `Result` (`[ERR-2]`) |
| `x as T` | conversion | `[TYP-6]` |
| `cast x as T` | downcast | stops the program if `x` is not a `T` (`[DSP-6]`) |
| `copy of x` | a copy of your own | by the type's `Clone` (`[OWN-8]`) |
| `owns x`, `shares x`, `shares mut x`, `thread_shared x`, `weakref x`, `sharedref mut x`, `ref x`, `ref mut x` | the words before a value | `[OWN-13]` |
| `f"…{e}…"` | formatting | allocates a `String`; `E4001` in `@noalloc` (use `format_to`) |
| `[a, b]`, `{k: v}`, `{a, b}` | collection literals | `[TYP-38]` |

* `[EXP-9]` *(new in 0.9.9)* *(changed in 0.9.9)* *(changed in 0.9.15)* `a is b` tests whether two class
  objects (or two views) are the same object, by comparing addresses. `x is None` and `x is not None`
  test an `Option` for absence and presence. Before a type name `is` asks for a kind: `x is T` is a
  `bool` that says whether `x`, a value of an interface type (`dyn I`, `owns dyn I`, `shares dyn I`) or
  of a base class, is a `T`, and `if x is T as name:` names it there (`[GRM-45]`, `[DSP-4]`); each is one
  type-id compare. A kind test that always passes is `E2036`, and one that can never pass, of a type
  that does not implement the interface or derive from the base class, is `E2037` (`[DSP-4]`). Any other
  operand of `is` is `E2150`, whose help names `==`.
* `[EXP-10]` *(new in 0.9.10)* *(changed in 0.9.15)* **Optional chaining.** `a?.f` and `a?.m[T](…)` take an `Option`
  (or a view of one; anything else is `E2020`): `None` when `a` is `None`, and otherwise the
  field or the method's result of the value inside, wrapped in `Some`, unless it is an `Option`
  itself, which is then the result (a chain is one `Option`, never an `Option` of an `Option`), or
  nothing, which is then what the chain is. `a` is read as `[GRM-13]` reads a place it does not
  hand over: a `Copy` value inside is copied and any other viewed, so a field that is not `Copy` comes
  as a view (`p?.name` is an `Option[ref String]`) and `a` keeps its value; from a temporary it is
  handed over (ODR-107).
* `[TYP-38]` *(new in 0.9.9)* *(changed in 0.9.15)* **Collection literals.**
  * A list literal `[a, b, c]` has the type its context expects: `Array[T]` (allocates), `[T; N]` (no
    allocation) or `Span[T]` (a view of a temporary fixed array). With no context it is `Array[T]`.
    `[v; N]` is always the fixed array of `N` copies of `v`. A list literal is never a `Set`: in a
    `Set` position it is `E2020`, whose help is `{a, b}`.
  * A map literal `{k: v, …}` is a `Map[K, V]` (insertion-ordered, `[STD-11]`); a set literal
    `{a, …}` is a `Set[T]`. A repeated key keeps its first position and its last value, as Python's
    `dict` does; `{1, 1}` is `{1}`. With no context a text key, value or element is a `String`, since
    a map written with no type owns its text; it holds `StrView` only where the program says so
    (`labels: Map[StrView, int]`, `[STD-11]`).
  * `[]` and `{}` need their element type from the context or from later uses in the function
    (`[TYP-23]`); left open, they are `E2060`.
  * Every element converts to the element type by `[TYP-5]`, so `["ann", "bob"]` in an
    `Array[String]` position is two `String`s, and in an `Array[StrView]` position is two static
    `StrView`s. An element for a list of `owns` values is made with `owns` (`[owns Circle(r=1.0)]`,
    `[OWN-10]`).
  * A literal that allocates carries `Alloc`; in a `@noalloc` function it is `E4001`, whose help names
    the fixed-array form. A generator expression (`[GRM-38]`) is not a collection and never allocates.
    A literal followed by `using m` takes its memory from the memory manager's instance `m`
    (`[0.0, 1.0] using current`, `[ALC-1]`).

```ember
fn main():
    counts: Map[String, int] = {}
    for word in "the cat and the hat".split(" "):
        counts[word] = counts.get_or(word, 0) + 1
    squares = [n * n for n in 0..10 if n % 2 == 0]
    seen = {"ann", "bob"}
    if "ann" in seen and 0 <= squares[1] < 10:
        println(counts, squares)
```

## VI.4 Control flow

* `[CTL-0]` The condition of `if`, `elif`, `while` and a match guard MUST be a `bool`; there is no
  truthiness. `E2035` carries a type-directed fix-it: `not xs.is_empty()` for a container or string,
  `x is not None` for an `Option`, `x != 0` for a number.
* `[CTL-1]` *(changed in 0.9.9)* *(changed in 0.9.15)* `for names in e:` walks:
  * a place `e` whose type is `Iterable`: calls `e.iter()`; each loop name is a read-only view of the
    element it is on, and what it reads does not change while it is used (`[BRW-1]`);
    `for ref mut x in e:` names the element so that it may change it, the only name for it while it
    is used;
  * `for x in owns e:`, or a value `e` (a temporary) whose type is `IntoIterator`: hands `e` over, and
    each element to the loop's name;
  * an `Iterator` value: calls `next` until `None`.
  `Array`, `[T; N]`, `Span`, `MutSpan`, `Set`, `Map`, ranges and generators are iterable; a `Map`
  yields its keys, as Python's `dict` does, and with two names its keys and values (`for k, v in m:`,
  as `m.items()` gives them; `for k, ref mut v in m:` may change each value; `m.values()` the values),
  and `for k in owns m:` hands its keys over, ending the values (`into_items()` keeps both); a `String`
  or `StrView` yields its `char`s. A loop's names follow the comma rule
  (`for ref mut pos, ref vel in q:`; a name with no word after one with a word is `E0116`,
  `[OWN-13]`). A plain loop name changes
  nothing: `for v in xs.iter_mut(): v += 1` is `E3023`, whose help gives `for ref mut v in xs:`.
* `[CTL-2]` *(changed in 0.9.15)* While a `for` loop walks a collection, nothing changes it but the
  loop's own handle on the item it is on, whatever the change and through whatever name or call
  (`[BRW-18]`): `xs[0] = 5` inside `for x in xs:` is `E3020`, whose help gives `for ref mut x in xs:`
  and `x = 5`, and a collection that must gain or lose items does so after the loop (`retain`,
  `extend`, a list of the program's own). A counting loop `for i in 0..xs.len():` does not walk `xs`: it
  may write its elements and add to it, but a removal is `E3020`; a `while` loop that tests its position
  against the current length at every step may add and remove.
* `[CTL-3]` *(changed in 0.9.10)* `a..b` (`Range`), `a..=b` (`RangeInclusive`) and `a..` (`RangeFrom`) over integers are
  iterable and compile to a counted loop with no iterator object. `a..=MAX` terminates after `MAX`;
  `a..` has no end: it gives its type's maximum, and the value after it is an overflow (`[TYP-8]`),
  found when the loop or the iterator asks for it. A range is a
  value (ODR-027): `Range[T]`, `RangeInclusive[T]`, `RangeFrom[T]` and `RangeTo[T]` (`..b`) are
  prelude structs with public bounds, `start` and `end` (a `RangeFrom` has only `start`, a `RangeTo`
  only `end`), and are `Copy` when `T` is. A `for` over a range value counts over a copy of its
  bounds, so the range is unchanged and can be iterated again, as Python's `range` can.
* `[CTL-3b]` *(changed in 0.9.10)* *(changed in 0.9.15)* Iteration over ranges, `Span`, `MutSpan`, `Array`, `[T; N]`, `SoA`
  columns, and `enumerate`, `zip`, `take`, `skip`, `step_by`, `copy_each` and `rev` composed over them,
  MUST compile to an induction-variable loop over base pointers and lengths, with no iterator object in
  memory and no call per element, independent of the host C compiler's optimiser. `step_by(k)` becomes
  the increment `k` (a `k` that is not a positive constant is checked once before the loop). An
  `enumerate` numbers in the type the known-numbers rule gives it (`[STD-19]`), which holds every
  number it can give, so the loop computes each number with no check: over a range of 64-bit
  numbers the compiler knows nothing of, an `i128`. The loop has a copy that computes numbers wider
  than 64 bits as `int`s, taken after one test before it when every number fits one, and computes
  them in their own type otherwise (G8-4 decision C); the numbers are the same either way.
  `tests/conformance/CTL-3b/` asserts it on the emitted C.
* `[CTL-4]` The `else` of `while` or `for` runs when the loop ends without `break`.
* `[CTL-5]` A `match` tests arms top to bottom; the first that matches runs; a guard is evaluated after
  binding; the arms MUST be exhaustive (`E2090`); an unreachable arm is `W2091`.
* `[CTL-6]` `with a = e1, b = e2:` binds `a` and `b` for the block and drops them in reverse order when
  it ends. `with e:` keeps the temporary alive for the block (a guard).
* `[CTL-7]` *(changed in 0.9.15)* `defer:` registers a block to run when the enclosing block exits, last registered first.
  It may refer to locals declared before it, which it uses as they are when it runs; they are not
  dropped before it. It cannot `return`, `break` or `continue` (`E2160`).
* `[CTL-8]` On every exit from a block, its `defer` blocks run first and then its locals are dropped in
  reverse declaration order.
* `[CTL-9]` `pass` does nothing; an empty block is written `pass`.
* `[CTL-10]` *(new in 0.9.9)* **Names assigned in every branch.** In an `if`/`elif`/`else` that has an `else`, or an
  exhaustive `match` statement, a name that is not in scope before the statement and is declared (by
  `x = e`) in **every** arm that completes normally, at the **same type**, is declared in the enclosing
  block and is initialised after the statement. An arm that ends in `return`, `break`, `continue` or a
  panic does not need to declare it. Different types in different arms are `E2230`, naming each arm.
* `[CTL-11]` *(new in 0.9.15)* **Headers that name what they test.** An `if` or `while` header that
  opens an `Option` or a `weakref` link (`if cam.target exists as p:`, `while cur.parent exists as p:`,
  `[GRM-44]`), and an `if` or `elif` header that tests a value's kind (`if s is Circle as c:`,
  `[DSP-4]`), run their block as those rules say, and the name they declare is in scope in that block
  only, never in an `elif` or `else`: `else:` runs when the link or `Option` holds nothing, or when the
  value is not of that kind. The name keeps its place for the whole block (`[BRW-14]`): a plain one
  only reads, and `exists as copy of v` is a copy of the program's own.

```ember
fn describe(n: int) -> String:
    if n < 0:
        kind = "negative"
    elif n == 0:
        kind = "zero"
    else:
        kind = "positive"
    return f"{n} is {kind}"
```

## VI.5 Lambdas and function values

```ember
fn apply_twice(f: fn(int) -> int, x: int) -> int:
    return f(f(x))

fn make_adder(n: int) -> fn(int) -> int:
    return fn(x: int) copy of n => x + n

struct Button:
    label: String
    on_click: fn()

fn main():
    step = 3
    println(apply_twice(fn(x) => x + step, 1))
    add5 = make_adder(5)
    b = Button(label="ok", on_click=fn() => println("clicked"))
    b.on_click()
    println(add5(1))
```

* `[CLO-1]` *(changed in 0.9.15)* A named function, a lambda and a local function each have a type of
  their own (ODR-278), which implements its callable type; one that keeps no state is a value of size
  zero. Given to a type parameter (`[CLO-14]`) it keeps that type, and a call through it is direct;
  wherever a callable type is written (`[CLO-3]`) it becomes a **function value** of that type
  (`[CLO-10]`). Where two different functions meet in one place (`if c: f = a else: f = b`, a second
  assignment, a conditional's two branches, a collection's elements), the place has the callable type
  both convert to. A local holding a function that keeps no state is not captured: a lambda that uses
  it keeps nothing for it (`[CLO-2]`), since giving it the same function again changes nothing.
* `[CLO-2]` *(changed in 0.9.9)* *(changed in 0.9.15)* **What a function captures is written after its
  brackets.** A name a lambda or a local function uses from the function that made it is the **real
  variable**, that very variable and not a copy, unless a capture word after its brackets names it:
  `fn() copy of a, owns b, shares c, shares mut d, thread_shared e, weakref f, ref g:`. Each word means
  what it means before a value (`[OWN-13]`): `copy of` a copy of its own; `owns` handed over, for every
  type, numbers included (after `fn() owns count`, the function that made the lambda no longer uses
  `count`, `E3040`, and the lambda uses it under the same name); `shares`, `shares mut` and
  `thread_shared` one more owner; `weakref` a link; `ref` a read-only view of what the name views at
  that moment, for a loop's name the element of that round. So `b.on_click(fn(): score += 1)` changes
  the real `score`, and `b.on_click(fn() copy of score: println(score))` keeps a copy of its own. What a
  function captured with a word, and what that owns alone, is its own state (`[CLO-10]`); a real
  variable stays at its own address, and the function only borrows it (`[CLO-4]`). A place that holds
  the function value counts as reaching every real variable it uses, and a call that may run it as a
  change of each one it changes (`[BRW-11]`, `[CLO-17]`). `ref mut` is not a
  capture word (`E0120`: "remove `ref mut x`: without a word the function already uses the real `x`,
  and may change it: `fn(): x += 1`"); a captured name with no word after one with a word is `E0116`
  ("`copy of` here belongs to `name` only; say what the function does with `score`:
  `fn() copy of name, copy of score:`, or leave `score` out to use the real `score`"); `with` before the
  capture words is `E0100` ("a function's captures follow its brackets with no `with`: write
  `fn() copy of score:`"); `owned fn` is `E1010` and `owns fn` `E0109`, each naming the words to write
  on each captured name (`fn() owns name, copy of score:`).
* `[CLO-3]` *(changed in 0.9.9)* *(changed in 0.9.10)* *(changed in 0.9.15)* **A callable type is one
  type wherever it is written.** `fn(A) -> R` as the type of a parameter, a field, a local, a result,
  a collection's element or a `static` is a function value of that type (`[CLO-10]`), filled from a
  named function (`fn log_click():` is a `fn()`), a local function or a lambda; one that captures
  nothing, or holds only its own state, may be stored anywhere a value may, and one that uses real
  variables may not outlive them (`[CLO-4]`). A parameter of callable type receives a function value
  and calls through it; a function that should get each function it is given as its own copy, called
  directly, takes it as a type parameter (`[CLO-14]`). A long callable type is named once with `type`
  (`type Handler = fn()`, then `handlers: Array[Handler] = []` and `fn on_click(owns f: Handler):`).
  `extern "C" fn(A) -> R` is a C function pointer: functions that capture nothing only. Converting a
  stored function value to one reads its C entry without handing it over; a value holding state has
  none, and the conversion panics (ODR-102).
* `[CLO-14]` *(new in 0.9.9)* *(changed in 0.9.15)* A callable type may be written as a type parameter's
  bound, on the line above the function: `[F: fn(int) -> int]` above `fn apply(f: F, x: int) -> int:`.
  `F` names the exact function given, so each function or lambda given gets its own instance of
  `apply`, and the call through `f` is direct and may be inlined, as C++'s `template <typename F>` and
  Rust's `F: FnMut`; a parameter so bounded is called like a function, and the call counts as making
  what the given function changes, worked out from it (`[EFF-2]`). A library function that calls a
  function right away takes it this way (`[F: fn(Item)]` above `fn for_each(owns f: F) owns this:`;
  `retain`, `inspect`, `sort_by`, `[STD-19]`). `Callable[…]` is not source syntax.
* `[CLO-10]` *(new in 0.9.9)* *(changed in 0.9.10)* *(changed in 0.9.15)* A function value is two words,
  `{code, state}`. Its state is what it needs besides its code: what it captured with a word, in one
  block of its own made with the value (it carries `Alloc` and is reported by `ember inspect --alloc`),
  except that a state of one word or less may be kept in the state word itself, with no block; and
  where the real variables it uses live, which costs nothing beyond their addresses. A named
  function, or a lambda that captures nothing, has no state and allocates nothing, and is `Copy`; a
  function value with state is not, and `=` hands it over. Calling one is one indirect call. A call
  through a list element or a field copies the two words out before the arguments are given and
  keeps no pointer into the list or field. Replacing or ending a function
  value ends what its state owns, as ending an object runs its `drop` (`[DRP-8]`): `h.keep = fn(): pass`
  ends the function stored there before and what its `owns` captures held, so a view of that is not
  used after that line (`E3060`, `[RC-5]`). Whether another thread may run it is said by its type,
  `thread_shared fn()` (`[CLO-16]`); two are equal (`==`) when they call the same function with the same
  state (ODR-102).
* `[CLO-4]` *(changed in 0.9.9)* *(changed in 0.9.10)* *(changed in 0.9.15)* **A function value does
  not outlive what it uses and does not own.** The real variables a lambda uses are checked in the
  function that writes it, which sees every place the function value goes: the value must provably end
  before them (it is kept only in locals, local lists or objects that never leave the function, and
  every function it is handed to keeps it no longer, by that function's summary, `[EFF-24]`), or it is
  `E3026`, whose message names a word for the place (`fn() shares mut count`, `fn() owns xs`) or a
  scope. A function that outlives the function that made it takes what it keeps with `owns`:
  `return fn(): n += 1` in `make_counter`, whose local `n` ends when it returns, is `E3026` ("the
  returned function changes `n`, which ends when `make_counter` returns; let the function own it: write
  `return fn() owns n: n += 1` in place of this line"); with `owns`, `n` is the function's own state,
  and the returned type is a plain `fn()`: `fn make_counter() -> fn():`. A name captured with `ref`
  follows the same rule (`[LT-42]`); a task's use of a loop's own name is Part XI's.
  Its message for a handler kept after its maker returns: E3026 ("the click handler changes `total`, which belongs to `register`'s caller, and `b` keeps the handler after `register` returns; keep the total in an object the handler can own a share of: declare `class Counter:` with the field `n: int = 0`; in `main`, instead of `total = 0`, `register(ref mut b, ref mut total)` and `println(total)`, write `count = Counter()`, `register(ref mut b, shares mut count)` and `println(count.n)`; and instead of `register`'s two lines write `fn register(ref mut b: Button, owns count: shares mut Counter):` and `b.on_click(fn() owns count: count.n += 1)`").
* `[CLO-5]` *(changed in 0.9.15)* A function value that captures an object with `shares` or
  `thread_shared` holds one more owner of it. A loop of owners through function values' state is
  refused as every loop of owners is (`[WK-1]`); the link back is captured `weakref`
  (`fn() weakref ui:`), or the object is given to the function as a parameter.
* `[CLO-6]` *(changed in 0.9.9)* *(changed in 0.9.15)* A function that gives one of its captures away
  (to an `owns` parameter, a return, a field) can be called only once. Its type is `once fn(A) -> R`,
  not `fn(A) -> R`; given where a function may run twice it is `E3013` ("this function gives `s` away,
  so it can run only once, and `call_twice` runs it twice; hand over a copy each time instead:
  `call_twice(fn(): consume(copy of s))`"). A parameter that takes a `once fn` holds it,
  `owns f: once fn(A) -> R`, and accepts both kinds; a plain `f: once fn(A) -> R` is `E3013`
  ("`call_once` uses `f` up, so it must hold it: write `owns f: once fn()`"). Calling `f` uses it up; a
  second call is `E3040`.
* `[CLO-6a]` *(changed in 0.9.9)* *(changed in 0.9.15)* A `once fn` value the program owns, including
  one in an `owns` block or a collection, is callable: the call hands the value out of its place (the
  place becomes empty, and a collection element is removed by the calling API, e.g.
  `jobs.pop_front()`).
* `[CLO-7]` *(changed in 0.9.9)* *(changed in 0.9.10)* *(changed in 0.9.15)* Standard-library functions
  that keep a function beyond the call, or run it on another thread, take it `owns`: `thread.spawn` and
  `jobs.submit` take `owns f: thread_shared fn() -> R` (`[THR-10]`, `[JOB-2]`), a callback registry an
  `owns` function value, and `scope.spawn` a plain `owns f: fn() -> R`, whose tasks may use the
  caller's own variables since they end with the `with scope` block (`[THR-11]`). Nothing is captured by
  a rule of the call: the function given says after its brackets what it holds
  (`thread.spawn(fn() owns tx: produce(tx))`), a name with no word is the real variable, and a function
  that may outlive a real variable it uses is `E3026` (`E7004` for an unscoped thread or a kept C
  callback, whose message names the words, Part XI).
* `[CLO-11]` *(new in 0.9.9)* A call `recv.name(args)` where `recv`'s type has no method `name` but has a field `name`
  of callable type calls that field. A method of that name takes precedence.
* `[CLO-12]` *(new in 0.9.9)* *(changed in 0.9.15)* A local function (`[GRM-28]`) is a named lambda: it takes the same
  capture words after its brackets (`fn push_a() thread_shared log:`), follows `[CLO-1]` to `[CLO-6]`,
  and may be passed by its name, so a long lambda may be written inside the call
  (`a = thread.spawn(fn() thread_shared log: log.push("a"))`) or as a local function
  (`fn push_a() thread_shared log:` with the body `log.push("a")`, then `a = thread.spawn(push_a)`).
* `[CLO-16]` *(new in 0.9.15)* **`thread_shared fn()`: a function any thread may run.** A callable
  type says whether any thread may run its functions. A list, field or parameter whose functions may
  run on any thread says so in its type (`handlers: Array[thread_shared fn()] guarded by lock = []`,
  `fn on_event(owns f: thread_shared fn()):`). A function of such a type holds `thread_shared` values or
  copies, never a `shares` or `shares mut` value or a `weakref` link of one (`E7015`: "`on_event` keeps
  functions any thread may run (`thread_shared fn()`), and this one holds `count`, a `shares mut` owner,
  which stays on one thread; share the counter between threads: write `count = thread_shared Counter()`
  in place of line 12, give `Counter` the field `n: Atomic[int] = Atomic(0)` (with
  `from std.sync import Atomic` at the top), and pass `fn() thread_shared count: count.n.fetch_add(1)`").
  A class whose objects are made `thread_shared` holds functions only in such types:
  `bus = thread_shared Bus()`, for a `class Bus:` with `handlers: Array[fn()] = []`, is `E7015` at that line
  ("`Bus.handlers` holds plain `fn()` values, and a `thread_shared Bus` may run them on any thread;
  write `handlers: Array[thread_shared fn()] = []` in `Bus`"). A function that captures nothing, or only
  `thread_shared` values and copies of values that may go to another thread, meets
  `thread_shared fn()`.
* `[CLO-17]` *(new in 0.9.15)* **What a function value does is worked out from the program.** A
  callable type says nothing about what its functions change. A call through a function value makes the
  changes, reads and ends of every function the program can put there (`[EFF-2]`), and so does a call
  that hands it to something that runs it on its own (`[EFF-26]`); storing a function value in a field,
  a list, a local or an iterator adapter changes nothing it changes. So a library's `Button` holding a
  program's handler that pushes to `LOG` makes `b.click()` a change of `LOG` in that program, inside a
  loop over `LOG` `E3020` (`[BRW-18]`). For the overlap of a call's arguments a function value counts as
  changing every place its functions change and every real variable they capture (`[BRW-11]`), and a
  function value is not stored in a place that a real variable it changes owns, all the way down
  (`b.on_click(fn(): b.handlers.clear())`, `t.on_visit = fn(n: Node): t.root.children.clear()`, `E3022`:
  "this function clears `b.handlers`, and `on_click` would keep it in `b.handlers`, so it would change
  the button that holds it while the button runs it; clear the handlers after the click instead:
  `b.click()`, then `b.handlers.clear()`"), since it would change its own holder while it runs.
* `[CLO-18]` *(new in 0.9.15)* **A function value that changes its own state.** A call through a stored
  function value whose functions may change their own state is a change of the place that holds it: the
  holder is reached through a name that may change it (`for ref mut h in this.handlers: h()`), and a
  plain `h` there is `E3023` ("`h` only reads the handler in `this.handlers`, and calling it may change
  its own state (the handler stored on line 20 adds to its own `count`); walk the handlers with
  `for ref mut h in this.handlers:` in place of line 9"). Such a value given to a parameter that only
  reads is `E3023` too ("`cb` changes its own copy of `score` each time it runs, and `run` takes it
  read-only; write `fn run(ref mut f: fn()):` and call `run(ref mut cb)`"). The holder does not become
  an argument of the call: `[BRW-11]` sees only the call's arguments. A body that may end its own state
  (a handler that removes itself) and uses that state afterwards holds one count on it across that
  call.
* `[CLO-19]` *(new in 0.9.15)* **A lambda's parameters say what they receive.** A lambda parameter's
  word is what is written on it, and matches the callable type it is given to: where the type hands the
  value over (`fn(owns String)`), the lambda writes `fn(owns s) => println(s.len())`, and a plain
  `fn(s)` there is `E2020` ("`each_name` hands each name over (`fn(owns String)`), so this function
  receives it as its own: write `fn(owns s) => println(s.len())`"); where the type gives it read-only,
  `fn(owns s)` is `E2020` ("`each_name` gives each name read-only, so this function cannot take it
  over; to keep your own, write `fn(s): names.push(copy of s)`"); `fn(ref mut ui)` changes the caller's
  value. A plain one reads. Written without types, the parameters follow the comma rule
  (`fn(ref mut ui, ref dx)`; `E0116`, `[OWN-13]`).

## VI.5a Generators

A **generator** is a function whose body runs step by step, producing values with `yield`. It is
Python's generator, and it is also how Ember writes frame-spanning gameplay sequences.

```ember
gen fn countdown(n: int) -> Generator[int]:
    i = n
    while i > 0:
        yield i
        i -= 1

gen fn evens(xs: Span[int]) -> Generator[int]:
    for x in xs:
        if x % 2 == 0:
            yield x

fn main():
    for t in countdown(3):
        println(t)
    data = [1, 2, 3, 4]
    total = evens(data).sum()
    println(total)
```

* `[CORO-1]` *(changed in 0.9.9)* A `gen fn` declares a generator. Calling it runs none of its body; it
  returns the generator's **frame**, a value holding the parameters and suspended state. The declared
  return type is `Generator[Y]` or `Generator[Y, R]`, where `Y` is the type of each yielded value and
  `R` (default `void`) the type of the final `return` value.
* `[CORO-2]` `yield e` suspends the generator and produces `e`. `yield` outside a `gen fn` is `E2220`.
  A `gen fn` with no `yield` is accepted and produces `L2004`.
* `[CORO-3]` *(changed in 0.9.9)* *(changed in 0.9.15)* `Generator[Y, R]` in a signature names the function's own frame type
  opaquely, as `some Iterator[Item = Y]` would: each `gen fn` has a distinct, sized frame type, which is
  not `Copy`, that implements `Iterator[Item = Y]`. `next()` resumes the body to the next `yield`
  (returning `Some(y)`) or to its end (returning `None`, after which `result() -> Option[R]` gives the
  returned value). Calling `next()` after the end returns `None` again. Generators of different
  functions are kept together as `owns dyn Iterator[Item = Y]`. `yield` as an expression has type
  `void`.
* `[CORO-4]` The compiler rewrites the body into a state machine over the frame: locals live across a
  `yield` become frame fields, others stay on the stack. No run-time machinery exists beyond the frame.
* `[CORO-5]` *(changed in 0.9.15)* **A generator allocates nothing.** The frame's size is a compile-time constant; it is an
  ordinary value, handed over by `=`, that may live in a local, a field, an `Array` or an arena. It is
  held `owns dyn` only to store generators of different functions together. A `gen fn` carries `Alloc`
  only if its body does.
* `[CORO-6]` *(changed in 0.9.9)* *(changed in 0.9.15)* A view **into the generator's own frame** (a view
  of one of its own locals) may not be held across a `yield`: moving the suspended frame would leave it
  pointing where the local was (`E2221`: "`r` (line 3) points into this generator's own frame, which may
  move while it is suspended at the `yield` on line 4; use `n` itself: remove line 3 and write
  `yield n` instead of line 5"). A `for` over a list, string, map or set whose body yields walks by
  position: after each resume it reads the collection again from where it lives now (the frame, or the
  object's field) and checks the position against its current length, so it holds no address across the
  suspension and the frame may move (`for s in buf: yield copy of s` over the generator's own `buf`
  compiles). Views that came in as parameters, or are made from them, may be held across `yield`; the
  frame is then a view of what they view, as a returned view would be (`[LT-1]`). `evens` above is such
  a generator.
* `[CORO-7]` Dropping a suspended generator drops exactly the locals live at its suspension point, in
  reverse declaration order.
* `[CORO-8]` A generator's effect set is the union over its whole body; contracts apply to it as to any
  function.
* `[CORO-9]` A frame never points into itself in Safe code; `unsafe` code that builds one states it in
  `@safety`.
* `[CORO-10]` A `gen fn` may not be `extern`, `@export`ed or passed to C (`E2222`).
* `[CORO-12]` *(new in 0.9.9)* *(changed in 0.9.15)* A `gen fn` **method of a class** has `this`, and its frame is a
  view of the object (`[CORO-14]`): the object does not end while the generator will still be used, and
  the body changes the object as any method's body does (`[EXC-15]`).
* `[CORO-14]` *(new in 0.9.15)* **A generator is a walk of what it views.** A generator's frame is a view
  a call gave (`[LT-1]`): a method's points into its object, and `from` narrows it
  (`gen fn replay() -> Generator[int] from log:`). While it will still be used, the collection it walks
  does not change, however the generator walks it, by position included (`E3020`, `[BRW-18]`:
  "`d.log.push("open")` (line 3) changes `d.log`, which `g` walks (`g = d.replay()`, line 1), and `g`
  is used again on line 4; push after `g`'s last use: move `d.log.push("open")` below line 4"); any
  other change of what its view points into is `E3021` (`d.angle = 90.0` while `g` will still be used),
  unless `from` narrows the view to the field it walks.
* `[CORO-13]` *(new in 0.9.9)* *(changed in 0.9.15)* A `yield` while a `@must_drop` value (`[THR-6]`) is live is `E2231`: a
  suspended generator may never be resumed, so the value's `drop`, on which a safety guarantee
  depends, would wait as long as the frame is kept (a `thread.scope()` whose tasks use the generator's
  parameters). A scope that closes before the next `yield` is unaffected.
* `[CORO-11]` `std.coroutine` builds gameplay sequencing on generators with no further compiler
  support: `Scheduler` resumes a set of `Generator[Wait]` once per frame and drops the finished ones;
  `wait(seconds)`, `wait_frames(n)` and `wait_until(pred)` produce `Wait` values.

```ember
from std.coroutine import Wait, wait

class Door:
    open_angle: float = 0.0

    gen fn swing_open() -> Generator[Wait]:
        yield wait(0.5)
        for _ in 0..60:
            this.open_angle += 1.5
            yield Wait.NextFrame
```

## VI.6 Assertions and panics

* `assert(cond)`, `assert(cond, msg)`, `assert_eq(a, b)`, `assert_ne(a, b)` are checked in every
  profile; `debug_assert(…)` only in `debug`, and it MUST NOT have side effects that change a program's
  result (`W2016` if its argument calls a function with effects other than `Panic`).
* `panic(msg)`, `todo()`, `unreachable()` have type `Never`.
* `[PAN-1]` A panic prints `panic at <file>:<line>:<col>: <message>` (and a backtrace outside
  `release`) to standard error and calls `abort()`. There is no unwinding in this version; `defer`
  blocks and destructors do not run on panic.
* `[PAN-2]` Formatting a panic message allocates only for an f-string; `panic("literal")` and
  `panic(some_str)` are `@noalloc`.
* `[PAN-3]` A panic inside a `drop` that runs while the process is already panicking aborts at once.
