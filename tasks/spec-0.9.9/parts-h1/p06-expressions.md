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
* `[EXP-4]` *(changed in 0.9.9)* A temporary created while evaluating an expression statement is
  dropped at the end of that statement, in reverse order of creation. A temporary created while
  evaluating the **condition** of an `if`, `elif` or `while` is dropped before the chosen block runs,
  unless the condition is a pattern test whose bindings refer to it, in which case it lives to the end
  of that block only — never into an `elif` or `else`. A temporary bound by `with` lives to the end of
  the `with` block; one created by a `for` iterable lives to the end of the loop.

## VI.2 Places, values and moves

A **place** denotes memory: a local, a field of a place, an element of a place, the target of a
reference or `Box`, a field of a class object reached through a handle, a `static`. Everything else is
a value.

* `[EXP-5]` Mutation, a mutable borrow and a move require a place; `f(x).y = 1` is `E2140` unless `f`
  returns `ref mut`.
* `[EXP-6]` A place of a non-`Copy` type used as a value (assigned, passed to `owned`, returned,
  captured by `owned fn`) is **moved**, and is uninitialised until assigned again. Moving out of a
  field of a type with `drop` is `E3010`; out of an element of an array, `Array` or `Span` is `E3011`
  (use `mem.take`, `mem.replace`, `swap`, `pop`, `remove`); out of a class field is `E3012`; out of
  anything reached through a reference is `E3013`. Moving out of a plain struct's field leaves the
  struct partially moved; it cannot be used whole until the field is assigned again (`E3042`).

## VI.3 Operators and literals

| Expression | Meaning | Notes |
|---|---|---|
| `a + b`, `a - b`, `a * b` | arithmetic | overflow panics (`[TYP-8]`) |
| `a / b` | true division | floats only; integers are `E2240` (`[TYP-28]`) |
| `a // b`, `a % b` | floor division and modulo | Python meaning |
| `a ** b` | power | `[TYP-30]` |
| `a == b`, `a != b` | `Eq` | built in for scalars |
| `a < b` etc. | comparison | scalars built in; generic code through `Ord.cmp`; chains (`[GRM-25]`) |
| `x in c`, `x not in c` | `c.contains(x)` | `Contains` (`[STD-8]`) |
| `a is b`, `a is not b` | handle identity | class handles and references (`[EXP-9]`) |
| `x is None`, `x is not None` | option test | `Option` (`[EXP-9]`) |
| `a[i]` | `Index` / `IndexMut` / `IndexSet` | read; written in place; `a[i] = v` per `[STD-17]` |
| `a[i..j]`, `a[..j]`, `a[i..]` | slice | `Span`/`MutSpan`/`str`, bounds-checked |
| `a?.f`, `a?.m()` | optional chaining | on `Option`: `None` propagates |
| `e?` | early return | on `Option` or `Result` (`[ERR-2]`) |
| `x as T`, `h as? D`, `h as! D` | conversion | `[TYP-6]` |
| `f"…{e}…"` | formatting | allocates a `String`; `E4001` in `@noalloc` (use `format_to`) |
| `[a, b]`, `{k: v}`, `{a, b}` | collection literals | `[TYP-38]` |

* `[EXP-9]` *(new in 0.9.9)* *(changed in 0.9.9)* `a is b` compares two class handles (or two references) for identity.
  `x is None` and `x is not None` test an `Option` for absence and presence, and are the only uses of
  `is` on a non-handle type. Any other operand of `is` is `E2150`, whose help names `==`.
* `[TYP-38]` *(new in 0.9.9)* **Collection literals.**
  * A list literal `[a, b, c]` has the type its context expects: `Array[T]` (allocates), `[T; N]` (no
    allocation), `Span[T]` (a view of a temporary fixed array), or `Set[T]` when written `{a, b}`.
    With no context it is `Array[T]`. `[v; N]` is always the fixed array of `N` copies of `v`.
  * A map literal `{k: v, …}` is a `Map[K, V]` (insertion-ordered, `[STD-11]`); a set literal
    `{a, …}` is a `Set[T]`.
  * `[]` and `{}` need their element type from the context or from later uses in the function
    (`[TYP-23]`); left open, they are `E2060`.
  * Every element converts to the element type by `[TYP-5]`, so `["ann", "bob"]` in an
    `Array[String]` position is two `String`s, and in an `Array[str]` position is two static `str`s.
  * A literal that allocates carries `Alloc`; in a `@noalloc` function it is `E4001`, whose help names
    the fixed-array form.

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
* `[CTL-1]` *(changed in 0.9.9)* `for pattern in e:` iterates:
  * a place `e` whose type is `Iterable`: borrows `e` for the loop and calls `e.iter()`; elements are
    `ref T` (read through wherever a `T` is expected);
  * `owned e`, or a value `e` whose type is `IntoIterator`: consumes `e`;
  * an `Iterator` value: calls `next` until `None`.
  `Array`, `[T; N]`, `Span`, `MutSpan`, `Set`, ranges and generators are iterable; a `Map` yields
  `(key, value)` pairs, destructured by `for k, v in m:`; a `str` yields its `char`s. `for x in
  xs.iter_mut():` yields `ref mut T`.
* `[CTL-2]` The iterated place is borrowed for the whole loop; mutating it inside the loop is `E3020`,
  whose help names `retain`, `drain`, collecting first, or an index loop.
* `[CTL-3]` `a..b` (`Range`), `a..=b` (`RangeInclusive`) and `a..` (`RangeFrom`) over integers are
  iterable and compile to a counted loop with no iterator object. `a..=MAX` terminates after `MAX`.
* `[CTL-3b]` Iteration over `Span`, `MutSpan`, `Array`, `[T; N]`, `SoA` columns, and `enumerate`,
  `zip`, `take`, `skip`, `copied` and `rev` composed over them, MUST compile to an induction-variable
  loop over base pointers and lengths, with no iterator object in memory and no call per element,
  independent of the host C compiler's optimiser. `tests/conformance/CTL-3b/` asserts it on the
  emitted C.
* `[CTL-4]` The `else` of `while` or `for` runs when the loop ends without `break`.
* `[CTL-5]` A `match` tests arms top to bottom; the first that matches runs; a guard is evaluated after
  binding; the arms MUST be exhaustive (`E2090`); an unreachable arm is `W2091`.
* `[CTL-6]` `with a = e1, b = e2:` binds `a` and `b` for the block and drops them in reverse order when
  it ends. `with e:` keeps the temporary alive for the block (a guard).
* `[CTL-7]` `defer:` registers a block to run when the enclosing block exits, last registered first.
  It may refer to locals declared before it (borrowing them until the block ends). It cannot `return`,
  `break` or `continue` (`E2160`).
* `[CTL-8]` On every exit from a block, its `defer` blocks run first and then its locals are dropped in
  reverse declaration order.
* `[CTL-9]` `pass` does nothing; an empty block is written `pass`.
* `[CTL-10]` *(new in 0.9.9)* **Names assigned in every branch.** In an `if`/`elif`/`else` that has an `else`, or an
  exhaustive `match` statement, a name that is not in scope before the statement and is declared (by
  `x = e`) in **every** arm that completes normally, at the **same type**, is declared in the enclosing
  block and is initialised after the statement. An arm that ends in `return`, `break`, `continue` or a
  panic does not need to declare it. Different types in different arms are `E2230`, naming each arm.

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

## VI.5 Closures and callable values

```ember
fn apply_twice(f: fn(int) -> int, x: int) -> int:
    return f(f(x))

fn make_adder(n: int) -> fn(int) -> int:
    return owned fn(x: int) => x + n

struct Button:
    label: String
    on_click: fn()

fn main():
    step = 3
    println(apply_twice(fn(x) => x + step, 1))
    add5 = make_adder(5)
    b = Button(label="ok", on_click=owned fn() => println("clicked"))
    b.on_click()
    println(add5(1))
```

* `[CLO-1]` A lambda or local function has a unique anonymous type implementing its callable type. If
  it captures anything by reference it is a view type; if it is `owned fn` or captures nothing it is a
  plain value.
* `[CLO-2]` Captures are inferred per variable: read only ⇒ shared borrow; written ⇒ mutable borrow
  (and the closure needs a mutable place to be called: `mut f`); `owned fn` ⇒ each captured variable
  is moved in (copied if `Copy`, retained if a handle).
* `[CLO-13]` *(new in 0.9.9)* A read-only capture of a `Copy` variable whose storage would end while
  the closure is still live, and which is not assigned after the closure is created, is captured by
  **copy** instead of by borrow (for a handle, a retain). The difference cannot be observed; it lets a
  closure created in a loop use the loop variable after the iteration ends. A captured variable that is
  assigned later keeps the borrow rule, and its error's help is `owned fn`.
* `[CLO-3]` *(changed in 0.9.9)* **What `fn(A) -> R` means depends on where it is written.**
  * As a **parameter type** it is a bound, not a representation: the parameter is an implicit generic
    constrained by the callable type, each argument monomorphises the callee, and a call through it is
    a direct call. This is the zero-cost form. It accepts any lambda, local function or function,
    including non-`owned` lambdas that borrow the caller's locals.
  * In **any other position** — a field, a local annotation, a return type, a collection element, a
    `static` — it is an **owned callable value**: a function pointer plus the callee's captured state,
    owning that state. A value of this kind may be stored anywhere an owned value may. It is filled
    from a function, a capture-free lambda, or an `owned fn` lambda; a lambda that borrows is a view
    and may only be stored where `[TYP-15]` permits (`E3063` otherwise, whose help is `owned fn`).
  * `extern "C" fn(A) -> R` is a C function pointer: capture-free functions only.
* `[CLO-14]` *(new in 0.9.9)* A callable type may also be written as an explicit generic bound,
  `fn apply[F: fn(int) -> int](f: F, x: int)`, with the meaning of the parameter form of `[CLO-3]`;
  a parameter so bounded is called like a function. `Callable[…]` is not source syntax.
* `[CLO-10]` *(new in 0.9.9)* An owned callable value holds up to **three pointer-sized words** of captured state inline
  and makes no allocation; larger captured state is placed in one heap allocation, which carries the
  `Alloc` effect and is reported by `ember inspect --alloc`. Calling one is one indirect call.
* `[CLO-4]` A non-`owned` lambda cannot outlive what it borrows: storing it, returning it or passing it
  to an `owned` parameter follows the view rules (`[TYP-15]`).
* `[CLO-5]` A closure capturing a class handle holds a strong reference; the usual cycle caution
  applies (`[WK-1]`).
* `[CLO-6]` *(changed in 0.9.9)* A lambda that moves one of its captures out of itself (into an
  `owned` parameter, a return, a field) can be called only once. Its type satisfies `once fn(A) -> R`
  but not `fn(A) -> R`; supplying it where `fn` is required is `E3030`, shape O5. A parameter `f: once
  fn(A) -> R` accepts both kinds and calling `f` consumes it; a second call is `E3040`.
* `[CLO-6a]` *(changed in 0.9.9)* An owned `once fn` value, including one inside a `Box` or a
  collection, is callable: the call moves the value out of its place (the place becomes empty, and a
  collection element is removed by the calling API, e.g. `jobs.pop_front()`).
* `[CLO-7]` Standard-library APIs that store or send a callback (`thread.spawn`, `jobs.submit`,
  `Option.map` returning a value computed later, event registries) declare it `owned fn` or
  `once fn` in an owned-value position, so the call site passes an `owned fn` lambda.
* `[CLO-11]` *(new in 0.9.9)* A call `recv.name(args)` where `recv`'s type has no method `name` but has a field `name`
  of callable type calls that field. A method of that name takes precedence.
* `[CLO-12]` *(new in 0.9.9)* A local function (`[GRM-28]`) is a named closure: it captures like a lambda and follows
  `[CLO-1]`–`[CLO-6]`.

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
* `[CORO-3]` *(changed in 0.9.9)* `Generator[Y, R]` in a signature names the function's own frame type
  opaquely, as `some Iterator[Item = Y]` would: each `gen fn` has a distinct, sized, move-only frame
  type that implements `Iterator[Item = Y]`. `next()` resumes the body to the next `yield` (returning
  `Some(y)`) or to its end (returning `None`, after which `result() -> Option[R]` gives the returned
  value). Calling `next()` after the end returns `None` again. Generators of different functions are
  kept together as `Box[dyn Iterator[Item = Y]]`. `yield` as an expression has type `void`.
* `[CORO-4]` The compiler rewrites the body into a state machine over the frame: locals live across a
  `yield` become frame fields, others stay on the stack. No run-time machinery exists beyond the frame.
* `[CORO-5]` **A generator allocates nothing.** The frame's size is a compile-time constant; it is an
  ordinary move-only value that may live in a local, a field, an `Array` or an arena. `Box` it only to
  store generators of different functions together. A `gen fn` carries `Alloc` only if its body does.
* `[CORO-6]` *(changed in 0.9.9)* A reference or view **to a local of the generator's own frame** may
  not be live across a `yield`: moving the suspended frame would leave it dangling (`E2221`, whose help
  names iterating `owned` the collection, or iterating by index). References and views that came in
  **as parameters**, or were derived from them, may be held across `yield`; the frame is then a view
  type bound by their regions, exactly as a returned view would be (`[LT-1]`). `evens` above is such a
  generator.
* `[CORO-7]` Dropping a suspended generator drops exactly the locals live at its suspension point, in
  reverse declaration order.
* `[CORO-8]` A generator's effect set is the union over its whole body; contracts apply to it as to any
  function.
* `[CORO-9]` A frame never points into itself in Safe code; `unsafe` code that builds one states it in
  `@safety`.
* `[CORO-10]` A `gen fn` may not be `extern`, `@export`ed or passed to C (`E2222`).
* `[CORO-12]` *(new in 0.9.9)* A `gen fn` **method of a class** takes `self` (the frame retains the handle). Each access
  to the object inside it is checked on its own; a long-term access to the object (`[EXC-1]`) may not
  be live across a `yield` (`E2229`). `mut self` is not permitted on a `gen fn` (`E2229`).
* `[CORO-11]` `std.coroutine` builds gameplay sequencing on generators with no further compiler
  support: `Scheduler` resumes a set of `Generator[Wait]` once per frame and drops the finished ones;
  `wait(seconds)`, `wait_frames(n)` and `wait_until(pred)` produce `Wait` values.

```ember
from std.coroutine import Wait, wait

class Door:
    open_angle: float = 0.0

    gen fn swing_open(self) -> Generator[Wait]:
        yield wait(0.5)
        for _ in 0..60:
            self.open_angle += 1.5
            yield Wait.NextFrame
```

## VI.6 Assertions and panics

* `assert(cond)`, `assert(cond, msg)`, `assert_eq(a, b)`, `assert_ne(a, b)` are checked in every
  profile; `debug_assert(…)` only in `debug`, and it MUST NOT have side effects that change a program's
  result (`W2016` if its argument calls a function with effects other than `Panic`).
* `panic(msg)`, `todo()`, `unreachable()` have type `Never`.
* `[PAN-1]` A panic prints `panic at <file>:<line>:<col>: <message>` (and a backtrace outside
  `shipping`) to standard error and calls `abort()`. There is no unwinding in this version; `defer`
  blocks and destructors do not run on panic.
* `[PAN-2]` Formatting a panic message allocates only for an f-string; `panic("literal")` and
  `panic(some_str)` are `@noalloc`.
* `[PAN-3]` A panic inside a `drop` that runs while the process is already panicking aborts at once.
