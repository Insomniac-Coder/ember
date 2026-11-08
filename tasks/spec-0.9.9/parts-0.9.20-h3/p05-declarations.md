---

# Part V — Declarations

## V.1 Modules, packages, imports, visibility

* `[MOD-1]` *(changed in 0.9.10)* *(changed in 0.9.15)* A **package** is a directory tree with an
  `ember.toml` at its root. A **module** is one `.em` file. A module's path is the package name
  followed by its path under `src/`, with `/` replaced by `.` and the extension removed;
  `src/lib.em` or `src/main.em` is the package's root module, and `src/a/mod.em` is module `a` when
  `a/` has submodules. A single `.em` file with no manifest is a package of its own (`[CLI-4]`), and
  the directory containing it is its source root. A module and an item of one name in one scope are
  `E1032`: an item whose path is a module's (the root module's `fn grid()` beside module `grid`; a
  dependency's root module has its package's name), and an item named like a module its module
  imports (`fn grid()` beside `import paths.grid`). A type named like a built-in type (`struct i32`)
  is `E1032` too, except a name `[MOD-5]` lets a declaration shadow (`int`, `float`, `Never`,
  `StrView`) and a declaration `import c` takes from a header, which keeps the header's name. Every
  item thus has one path (`[MNG-1]`). The modules compared are those of the build, each imported by
  some module, not every file under the source root.
* `[MOD-2]` *(changed in 0.9.9)* Items are private to their module unless marked. `pub(package)`
  makes an item visible within the package; `pub` makes it visible to dependants. Using an item,
  field or constructor that is not visible is `E1052`, naming where it is declared and its
  visibility; `E1020` means only a name declared twice in one block (`[GRM-4]`).
* `[MOD-3]` *(changed in 0.9.9)* `import a.b.c` binds the name `c` to module `a.b.c`, and `import
  a.b.c as d` binds `d`. `from a.b import x, y as z` binds items. A standard module may be named
  without `std.`, as in Python: `import math` is `import std.math`, and `from random import Rng` is
  `from std.random import Rng`; when the package itself or one of its dependencies has that name,
  the package wins, with warning `W1003` naming both. A module path that names no module is `E1060
  no module named 'a.b.c'` at the import, whose help lists the nearest existing module paths; an
  item missing from an existing module is `E1061`. Neither is ever deferred to the first use
  (`[PHIL-12]`).
* `[MOD-8]` *(new in 0.9.9)* *(changed in 0.9.20)* `from m import *` binds every `pub` item of `m`.
  A name bound by two glob imports and then used is `E1031`, naming both sources; an explicit import
  or a local declaration shadows a glob. The prelude (`[MOD-5]`) counts as a glob import of `std`
  written first: a name bound by a glob import and by the prelude, and then used, is `E1031` naming
  both ("`len` is bound by `from util import *` (line 1) and by the prelude; write `from util import
  len` to take `util`'s, or `std.len` for the prelude's"), so no package's update can take a prelude
  name in silence.
* `[MOD-4]` Import cycles within a package are allowed; cycles between packages are `E1041`.
* `[MOD-7]` A field declared `pub(read)` (or `pub(package, read)`) may be read wherever a `pub`
  (`pub(package)`) field could be, and written only from its declaring module (`E1050` otherwise).
  Construction is a write, so a `pub(read)` field makes the memberwise constructor private to the
  module.
* `[MOD-9]` *(new in 0.9.15)* *(changed in 0.9.20)* **A package's edge.** What a dependant may rely
  on is what the package's `pub` items write and what the compiler works out from their code. Where
  a word exists to write a fact, only the written word counts across the edge: a `pub` generic
  function writes its bounds (`[IFC-6]`). Every other fact the compiler works out (what each
  function may change, read and end, with the chain of calls; which view parameters it uses after a
  line that may end what they view, which it makes a one-thread `weakref` link to, which it stores
  an owner of into a place of which other, and what it needs of the kinds that reach a `dyn`
  parameter; for a class, whether its methods say its objects are on one thread or `thread_shared`;
  the locks it may take; the function values it stores and the callbacks it hands to C; each type's
  layout, with whether its values carry the weak count, `[OBJ-1]`) is recorded in the package's
  interface file, and a change to it is a change of the package's interface (`[EFF-4]`): an edit
  inside one function, or a library update, can make a dependant's line stop compiling, and the
  message names the chain of calls. A fact worked out from a body never changes what a line does.
* `[MOD-9a]` *(new in 0.9.20)* **A library's call through what a dependant may fill.** For each line
  of a package's functions that calls through a `virtual` method of a `pub` class, a method of a
  `pub` interface used as `dyn`, or a function-typed place another package can fill (`[EFF-2a]`),
  the package records in its interface file, beside the function's summary (`[BLD-14]` item 12): the
  place it calls through; the collections walked across it and the generators still used after it
  (`[BRW-18]`); the read-only views, `ref mut` names and positions live across it, with what each
  points into (`[BRW-1]`, `[BRW-17]`); the names opened from links live across it, with their class
  families (`[RC-5]`); the parameters and `this` used after it (`[BRW-21]`); and the locks held at
  it (`[THR-19]`). A package that adds an override, an implementation or a stored function value
  checks it, at its declaration or at the storing line, against every recorded line it may be called
  from, with the chain named: a body whose summary changes what such a line walks is `E3020`
  ("`Door.on_update` clears `WORLD.scripts` (line 9), which `Script.run_all` walks while it calls
  `on_update` (its line 4); clear after the walk: give `run_all`'s caller the list of doors to
  remove"), one that changes what it holds or has lent is `E3022`, one that may end what a name
  opened from a link reads `E3060`, and one that takes a lock held there `E7018`, each at the line
  that adds the member ("`b.on(...)` (line 7) stores a function that pushes to `b2.log`, and
  `Bus.flush` (`bus`, line 10) walks `this.log` while it runs the handlers (its line 12); push to a
  list of your own in the handler and move it into the log after `flush`"). A member no recorded
  line refuses is accepted; nothing runs for it while the program runs.
* `[MOD-5]` *(changed in 0.9.9)* *(changed in 0.9.15)* **The prelude.** Every module implicitly
  imports these names from `std`, and this list is the only definition of the prelude:

  | Group | Names |
  |---|---|
  | Scalar aliases | `int` (`i64`), `float` (`f64`), `Never` |
  | Options and errors | `Option`, `Some`, `None`, `Result`, `Ok`, `Err`, `Error` (interface), `AnyError`, `RangeError` |
  | Text | `String`, `StrView`, `format` |
  | Collections | `Array`, `Map`, `Set`, `Span`, `MutSpan` |
  | Ranges | `Range`, `RangeInclusive`, `RangeFrom`, `RangeTo` |
  | Cells | `Cell` |
  | Interfaces | `Copy`, `Clone`, `Drop`, `Default`, `Eq`, `Ord`, `Ordering`, `Hash`, `Debug`, `Display`, `Iterator`, `Iterable`, `IntoIterator`, `Generator`, `Contains`, `From`, `Send`, `ThreadShared`, and the operator interfaces `Add`, `Sub`, `Mul`, `Div`, `FloorDiv`, `Rem`, `Pow`, `Neg`, `Not`, `BitAnd`, `BitOr`, `BitXor`, `Shl`, `Shr`, `Index`, `IndexMut`, `IndexSet`, with the `…Assign` forms of the arithmetic and bitwise interfaces |
  | Functions | `print`, `println`, `eprint`, `eprintln`, `input`, `assert`, `assert_eq`, `assert_ne`, `debug_assert`, `panic`, `todo`, `unreachable`, `min`, `max`, `abs`, `clamp`, and Python's `len`, `range`, `sum`, `sorted`, `enumerate`, `zip`, `reversed`, `any`, `all` (`[STD-26]`) |
  | Modules | `mem` (the module `std.mem`) |

  A local declaration or explicit import shadows a prelude name. Everything else in the standard
  library is imported explicitly (`from std.math import Vec3`). `owns`, `shares`, `thread_shared`,
  `weakref` and `sharedref mut` are words of the language, not library types (`[GRM-3]`).

## V.2 Functions

```ember
pub fn blend(dst: Array[f32], ref mut out: Array[f32], owns scratch: Array[f32], weight: f32 = 0.5) -> int:
    for i in 0..out.len():
        out[i] = dst[i] * weight
    return scratch.len()
```

* `[FN-1]` *(changed in 0.9.9)* *(changed in 0.9.15)* *(changed in 0.9.20)* **Parameter words.** What a function may do
  with a parameter is set by the word in front of its name, for every type; the type after the colon
  says what the caller's value is, never what the function may do with it.
  * `a: A`: **reads**. The callee reads the caller's `a` (a number is copied in); it changes nothing
    through `a` (`[BRW-15]`) and hands nothing over out of it (`[OWN-11]`), and what `a` reads is
    not ended or moved while `a` is used (`[BRW-1]`, `[BRW-21]`); a number another name of the same
    place writes across the call (a module variable, a name opened `ref mut` from a link, `[WK-11]`)
    shows through `a`, since the compiler never assumes a value unchanged where it cannot prove the
    names apart (`[EXC-15]`). Some parameters are passed as a copy instead, such as a small `Copy`
    value in registers (`[BRW-8]`); the meaning is the same.
  * `ref mut b: B`: **in-out**. `b` names the caller's place: every change through `b` reaches the
    caller's value, `b = v` puts `v` there, and while `b` is used nothing else the function uses
    reaches that place (`[BRW-22]`). Where the caller's value is a link or a view, the place holds
    it: `ref mut p: shares mut Player` may point the caller's link at another player and change the
    player, and `ref mut best: Span[int]` may point the caller's view elsewhere. `const ref mut b:
    B` changes the caller's value but is never given another ref (`[BRW-14]`).
  * `owns c: C`: **handed over**, for every type, numbers included. The caller's name ends, and a
    later use is `E3040`; a caller that keeps its own passes `copy of x` (`[OWN-9]`). `owns c:
    shares C` makes the callee one more owner that reads a shared value, and `owns c: shares mut C`
    one that may also change it.

  So a plain `game: shares mut Game` reads the caller's shared value, with no count, and an owner
  that may change the game is not made from it: `fn() shares mut game` is `E3023` ("`game` only
  reads the `Game` that `main` lends to `setup` (`shares mut Game` says what `main`'s game is, not
  what `setup` may do with it), so an owner made from it only reads too; to hand on an owner that
  may change the game, take one: write `fn setup(owns game: shares mut Game):`, call it
  `setup(shares mut game)`, and capture it `owns game`"). Only a `Mutex`, `RwLock`, `Atomic` or
  channel end, and the `guarded by` fields of an object inside their lock, are changed through a
  plain parameter, by their own operations (`[BRW-15]`, Part XI). `mut` alone on a parameter is
  `E0113`, and `ref` or `ref mut` written in a parameter's type is `E0117` (`[GRM-41]`).
* `[FN-1a]` *(changed in 0.9.15)* A `MutSpan[T]` parameter that writes the elements it views says
  `ref mut`, as every name that may change what it does not own does: `fn blend(ref mut dst:
  MutSpan[float], src: Span[float]):`. Through it the callee may also point the caller's span
  elsewhere; `const ref mut dst: MutSpan[float]` writes the elements and never does. A plain `dst:
  MutSpan[float]` only reads them, and a write through it is `E3023` ("`dst[i] += src[i]` writes an
  element of the caller's span, but `dst` only reads it; write `fn blend(ref mut dst:
  MutSpan[float], src: Span[float]):` and call it `blend(ref mut d, s)`"). At the call the word
  stands before the span's elements, so it may stand before a span made in the call (`integrate(ref
  mut p.as_mut_span(), v.as_span(), 0.5)`); a callee that points such a span elsewhere changes only
  that span, which ends with the line (`[SPN-3]`).
* `[FN-2]` *(changed in 0.9.15)* A parameter with no word reads. There is no by-copy mode: a callee
  that needs a value of its own takes it `owns`, and the caller passes `owns copy of x` to keep its
  own, or `owns x` to hand it over (`[FN-2a]`).
* `[FN-2a]` *(changed in 0.9.15)* *(changed in 0.9.20)* **A call says `ref mut` before an argument
  the callee may change, and `owns` before one it takes over**, with the same words as the
  parameter, `const` included: `bump(ref mut a)` for `fn bump(ref mut c: Counter):`, `blend(const
  ref mut d, s)` for `fn blend(const ref mut dst: MutSpan[float], src: Span[float]):`, `mem.take(ref
  mut this.items)`; `take(owns xs)` for `fn take(owns xs: Array[int]):`, whatever is given:
  `take(owns copy of xs)` gives a copy and keeps `xs`, `take(owns [1, 2])` a list made in the call,
  `m.insert(owns (i * 7), owns i)` a value computed in the call, in brackets (`[GRM-43]`,
  `[GRM-47]`), `names.push(owns p.name)` a part (`[OWN-11]`), and a function written in the call is
  given the same way, `xs.iter().for_each(owns fn(x: int): println(x))` (`[STD-19]`). A missing
  word, or one written where the parameter does not say it, is `E3028` ("`bump` may change `a` (its
  first line says `ref mut c: Counter`); say so at the call: `bump(ref mut a)`"; "`show` only reads
  its argument; remove `ref mut`: `show(a)`"; "`take` takes `xs` over (its first line says `owns xs:
  Array[int]`); say so at the call: `take(owns xs)`", with the help "to keep `xs`, give it a copy:
  `take(owns copy of xs)`"; for a computed argument, "`insert` takes this argument over (its first
  line says `owns key: K`); say so at the call: write `owns` before it, in brackets: `m.insert(owns
  (i * 7), owns i)`"; "`show` only reads its argument; remove `owns`: `show(xs)`"), and `ref mut`
  before something this function may not change (a plain parameter, a temporary, a place reached
  through a read-only name) is `E3027` ("`p` only reads the caller's player, so it cannot be given
  to `heal` as `ref mut`; take it as `ref mut p: Player` on `revive`'s first line, or walk the list
  with `for ref mut p in players:`"). `return a` and a constructor's argument take no `owns`: a
  value given to a memberwise constructor, or to an `owns` parameter of a written `init`, is handed
  over as `=` does (`[OWN-9]`; `Emitter(label)` for `fn init(owns name: String):`, after which
  `label` is `E3040`; `Emitter(copy of label)` keeps it; `Emitter(owns label)` is `E0109`,
  "`Emitter(...)` already hands `label` over; remove `owns`"); a value given to an `init` parameter
  with no word is read for the call and stays the caller's, as at every call (`Label(caption)` for
  `fn init(name: String):`, then `println(caption)` prints); and `ref mut` is written before an
  argument for an `init`'s `ref mut` parameter as at every call (`Widget(ref mut reg, label)` for
  `fn init(ref mut registry: Registry, owns name: String):`, `E3028` without it); nor does a value
  made in the call with a word that makes an owner or a link (`setup(shares mut game)`,
  `ROSTER.push(weakref this)`, `thread_shared x`, `sharedref mut x`), since that word says what is
  given (verdicts 147 and 160; `owns shares mut game` is `E0109`). A function from a variable given
  to a parameter whose type is a type parameter bounded by a function type (`owns f: F` under `[F:
  needs fn(int)]`, `[CLO-14]`), in any package, is lent for the call: `F` names that very function,
  the callee runs it and keeps nothing of it, so after `xs.iter().for_each(h)` and after `apply(h)`
  alike `h` is the caller's still. It takes no word when its functions keep no state or only read
  it; one whose functions may change their own state is given `ref mut h`, since the call changes it
  (`[CLO-18]`, `E3023` without the word: "`counter` changes its own `n` each time it runs, and
  `apply` runs it; say so at the call: `apply(ref mut counter)`"). `owns h` before such an argument
  is `E0109` ("`apply` runs `h` and keeps nothing of it; remove `owns`: `apply(h)`"); a function
  written in the call is given with `owns` (verdict 158), and a value the call gives back that keeps
  the lent function (an iterator adapter, `[STD-19]`) is a value holding a view of `h`, which lives
  no longer than `h` (`[LT-45]`). A parameter of a callable type (`owns f: fn(int)`) keeps what it
  is given and takes `owns` as any `owns` parameter does. An argument for a `Mutex`, `RwLock` or
  `Atomic` parameter, or a channel end, takes no word, since its own "only me" is the lock's block
  (`E3028`: "`top` takes the lock of `scores` itself; a `Mutex` takes no word at the call:
  `top(scores)`"); an `owns` parameter of one of these types takes `owns` as every such argument
  does (`start_worker(owns rx)` for `fn start_worker(owns rx: Receiver[Job]):`). The object a method
  is called on is not marked (`a.reset()`): what the method does to it is worked out from its body
  (`[EXC-15]`). The arguments of one call do not overlap a changeable one (`[BRW-11]`), and a call
  is checked against what the callee does with each view argument (`[BRW-21]`). The editor shows a
  parameter's word as an inlay hint where the call does not write one.
* `[FN-9]` *(new in 0.9.9)* *(changed in 0.9.15)* A parameter of a class type takes its word as one
  of any type does, since a class object has one owner unless a line says `shares` (`[CLS-1]`): `p:
  Player` reads the object, which neither changes nor ends while `p` is used; `ref mut p: Player`
  may change the object, and `p = Player()` gives the caller's place another object (the old one
  ends); `owns p: Player` hands the object over, so the caller's name ends. A method that may change
  its object is not called through a `p` that only reads it (`E3023`, `[EXC-15]`).
* `[FN-3]` *(changed in 0.9.9)* *(changed in 0.9.15)* *(changed in 0.9.20)* A function hands its
  result over to its caller. A result that is a view points into what the function's code shows it
  points into: its view arguments and, for a method, `this` (`[LT-1]`); `-> T from name` after the
  result type narrows it to one parameter or one field of `this` written bare (`-> StrView from
  src`) (`[LT-1]`, `[LT-1a]`); a view of text written in the program has the `static` region
  (`[LT-3]`). A view of the function's own locals or of a temporary is never returned (`E3063`).
* `[FN-4]` *(changed in 0.9.15)* **Methods and `this`.** Inside a method its object is `this`; a
  method does not list it among its parameters (`fn reset():` with `this.count = 0`). `owns this`
  after the parameters takes the object over, so the caller's name ends (`fn finish() owns this ->
  u64:`); what else a method does to its object is worked out from its body (`[EXC-15]`). A struct's
  method may give `this` a whole new value (`this = Point(x=0.0, y=0.0)`); a class's may not
  (`[CLS-7]`). A function in a type body is a method unless it is written `static fn`: it then has
  no object and is called on the type (`static fn starting_at(k: int) -> Counter:`, called
  `Counter.starting_at(5)`). `static fn` outside a type body is `E2115`; `this` in a `static fn` or
  a free function is `E1010` ("`this` can only be used in a method, and `starting_at` is a `static
  fn`, not a method", with the help "build the counter from `k`: `return Counter(n=k)`"; "`this` can
  only be used in a method, and `double` is not a method", with the help "to make `double` a method,
  move it into the struct or class it belongs to"). The help that builds the value is given where
  the function returns its own type and sets each field it names through `this` from its parameters;
  `self` is `E1010`, whose help gives `this` (`[GRM-41]`). A function's first line says its
  parameters, `owns this`, its result and `from`, in that order (`[GRM-41]`), and nothing about what
  the function changes or reads, which the compiler works out from its code (`[EFF-24]`).
* `[FN-5]` *(changed in 0.9.20)* A default is the argument the caller did not write: it is evaluated
  at the call, in the caller, after every written argument and in parameter order, as an argument
  expression is (`[EXP-1]`, `[EFF-1]`), and it takes no word, since it makes the value it gives: a
  default is written only for a plain or an `owns` parameter (`fn take(owns xs: Array[int] = []):`,
  each call makes its own list), and one on a `ref mut` parameter is `E3027` ("`ref mut xs` names
  the caller's place, and a default would be a temporary that ends with the call; take the list
  over, `owns xs: Array[int] = []`, or let the caller pass one: `reset(ref mut items)`"). It may
  read an earlier parameter and neither changes it nor hands it over (`E3023`, `E3013`, as a plain
  name): `t: String = s` reads `s`, which stays the function's own. Names and type expressions
  resolve in the function's declaration scope, with generic parameters substituted at instantiation.
  Defaults use that function's effective overflow policy; a caller's scope or policy does not
  replace these declaration bindings (ODR-084). Supplied arguments retain their own lexical overflow
  policy (`[TYP-8]`).
* `[FN-6]` *(changed in 0.9.9)* *(changed in 0.9.15)* *(changed in 0.9.20)* **Callable types.** A
  function is a value. A callable type is written `fn(A, ref mut B, owns C) -> R`, with the same
  parameter words as a declaration. It says the parameters, the result, and whether any thread may
  run its functions (`thread_shared fn(…)`, `[CLO-16]`), and nothing about what they change
  (`[CLO-17]`). `once fn(…) -> R` is a callable that may be called at most once (`[CLO-6]`), and
  `unsafe fn(…) -> R` one whose call is an `unsafe` operation (`[UNS-1]`, ODR-115); the two combine
  with `extern "C"` as the grammar orders them, `once unsafe extern "C" fn(…)`. Callable types are
  compared by their parameter types and words, their result, `once`, `unsafe`, `extern "C"` and
  `thread_shared`; `fn(…)` converts to `once fn(…)` and to `unsafe fn(…)`, never the reverse
  (`E3013` for a `once fn` where a function may run twice, `[CLO-6]`; `E2020` for an `unsafe fn`
  where a safe one is wanted). A function coerces to any callable type it satisfies, and to its
  `unsafe` form; an `unsafe fn` coerces only to an `unsafe` callable type (`E2020` otherwise). Where
  an `unsafe fn(…)` is wanted, any function or lambda that meets `fn(…)` may be given, and so may an
  `unsafe fn`. A function that captures nothing also coerces to `extern "C" fn(…)` when every type
  is FFI-safe: its parameters and result are those `[FFI-31b]` admits in an export's signature,
  written with the same words and lowered to the same carriers (`ref mut x: T` as `T*`, `Span[T]` as
  a pointer and a `size_t`), and any other is `E2020`, naming the parameter; it then gets a C entry
  that attaches the calling thread (`[FFI-22]`) and runs it, and its body is checked as an export
  under `threads=any` is (`[FFI-21]`, `[FFI-33]`), since the compiler cannot see which thread C
  calls it from. A view that a call through a callable value gives points into that call's own view
  arguments (`[LT-7]`). What a callable type means in each position is `[CLO-3]`.
* `[FN-7]` Recursion is permitted; tail calls are not guaranteed to be eliminated.
* `[FN-8]` *(changed in 0.9.9)* *(changed in 0.9.10)* *(changed in 0.9.15)* `main` is `fn main()`,
  `fn main() -> Result[void, E]` for any `E: needs Display`, or either form with a first parameter
  `args: Array[String]`. `args` holds the command line, decoded as UTF-8 with invalid bytes replaced
  by U+FFFD; `std.process.args_os()` returns the raw bytes. An `Err` returned from `main` is printed
  with `Display` to standard error and the process exits with status 1. The entry module's `main` in
  any other form (another parameter or more than one, another return type, or type parameters) is
  `E2183`, reported at its name; an `E` that is not `Display` is `E2040`, an unmet bound (ODR-101).
  **Scripts:** the statements at file scope of the entry file (`[GRM-2]`) form, in source order, the
  body of an implicit `fn main()`, or `fn main() -> Result[void, AnyError]` when one of them uses
  `?`; items anywhere in the file are declared as usual. A file with such statements that also
  declares `main` has two items named `main` (`E1030`). The names the statements bind are locals of
  the implicit `main`: a function of the file that uses one is `E1010`, whose help names a
  parameter, a `const`, a `static`, or a `static mut` for a module variable that functions change
  (`[STA-1]`) (unlike Python, where it would read a global). `return` and `yield` at file scope are
  `E0100`, as in Python; `process.exit(code)` ends a script early.
* `[FN-10]` *(new in 0.9.9)* A function whose return type is `Result[void, E]` returns `Ok(())` when
  control reaches the end of its body, as a `void` function returns; an explicit `return Ok(())` is
  still allowed. No other return type has an implicit value: a body that can reach its end is
  `E2182`, reported at the function's name (ODR-023).

## V.3 Structs

```ember
@derive(Copy)
pub struct Vec2:
    pub x: f32
    pub y: f32

    const ZERO: Vec2 = Vec2(0, 0)

    fn length() -> f32:
        return (this.x * this.x + this.y * this.y).sqrt()

    fn scale(k: f32):
        this.x *= k
        this.y *= k
```

* `[STR-1]` *(changed in 0.9.15)* *(changed in 0.9.20)* Every struct has a **memberwise
  constructor** `Name(field0, field1, …)` taking positional or named arguments in field order; a
  field with a default may be omitted. Its visibility is the narrowest among its fields', since
  construction writes every field: `pub` when every field is `pub` (a struct with no fields
  included), `pub(package)` when every field is `pub` or `pub(package)`, and private to its module
  otherwise (a private, `pub(read)` or `pub(package, read)` field, `[MOD-7]`); a call from where it
  is not visible is `E1052`, naming the field that narrows it ("`Packet(...)` is not visible here:
  `seq` is `pub(package)`, so a `Packet` is built only inside the `net` package"). A declared `fn
  init(…)` replaces it (`[STR-6]`). An argument for an `owns` field is made with `owns` on its line
  (`mesh=owns Mesh(points=[0.0])`, `[OWN-10]`).
* `[STR-2]` *(changed in 0.9.9)* *(changed in 0.9.20)* A field default is any expression; at a
  construction that omits the field, the written arguments are evaluated first, left to right
  (`[EXP-1]`, `[FN-5]`), then the omitted fields' defaults in field order, each at that construction
  (every field's, before the body, where the type declares an `init`, `[STR-6]`, `[CLS-2]`), and a
  default may allocate (`items: Array[int] = []`). Names and type expressions in a field default
  resolve in the type's declaration scope, with the type's generic parameters substituted at
  instantiation, as `[FN-5]`'s do; a field is named only through `this`, which a default has not,
  since it runs before the value exists (`E1010`: "`this` can only be used in a method, and a field
  default runs before `init`, when there is no object yet; set `on_tick` in `init`: write
  `this.on_tick = fn(): ...` below `this.name = n`"; "`cap`'s default reads `n`, a field of the same
  `Buf`, which has no value when defaults run; set `cap` in an `init`: `fn init(n: int):` with
  `this.n = n` and `this.cap = n * 2`").
* `[STR-3]` *(changed in 0.9.15)* *(changed in 0.9.20)* A struct with a `drop` method, or with a
  field that is not `Copy`, is not `Copy`: `=` from a whole name hands it over (`[OWN-3]`). `Copy`
  is never implicit: `@derive(Copy)` requests it, and is `E2080` if a field is not `Copy` or the
  type declares `drop` (`[DRV-1]`: "`Handle` has a `drop` (line 5), which would run once per copy;
  remove `@derive(Copy)` (line 1), and hand a `Handle` over or write `copy of h` with a `clone` of
  its own"); it gives `Clone` too, so `copy of` works on every `Copy` struct (`[OWN-8]`). Class
  objects, `shares` values, `weakref` and `sharedref mut` links, `ref mut` views and `MutSpan`s are
  not `Copy` (`[OWN-7]`), so a struct holding one is not.
* `[STR-4]` A struct may have no fields (`struct Marker: pass`).
* `[STR-5]` *(changed in 0.9.9)* *(changed in 0.9.15)* *(changed in 0.9.20)* **Implicit derives.** A
  struct or enum implements `Eq`, `Debug` and `Clone` field-wise, automatically, when every field
  implements the interface; `@no_derive(Eq)` (or `Debug`, `Clone`) opts out, and a hand-written
  implementation replaces the implicit one. A field-wise `Clone` copies what the value owns alone
  itself, and each link as one more link to the same target: a `shares` field as one more owner, a
  `weakref` field as one more link, a `sharedref mut` field as one more holder (`[OWN-8]`). A type
  that declares `drop` is not implicitly `Clone`: a field-wise copy would release twice what its
  `drop` releases, so it is `Clone` through `@derive(Clone)` or a written `clone` (ODR-026). `Copy`,
  `Ord`, `Hash`, `Default` and `Display` are never implicit, except that a unit-only enum is `Copy`,
  `Ord` and `Hash` automatically (`[ENM-3]`), and are requested with `@derive(…)` or written by
  hand.
* `[STR-7]` *(new in 0.9.9)* *(changed in 0.9.15)* Inside the body of a `struct`, `enum`, `class` or
  `extend` block, `Self` names the type being declared with its own generic parameters, and the
  type's name may be written with any arguments: with `[T]` above `struct Pair:`, a method may
  return `Self`, `Pair[T]` or `Pair[int]`.
* `[STR-6]` *(new in 0.9.9)* *(changed in 0.9.15)* *(changed in 0.9.20)* `fn init(…)` on a struct is
  its constructor; it sets the fields through `this`, with no word. Before its body runs, every
  field default is evaluated and stored, as `[CLS-2]` says for a class. Every field without a
  default MUST be set on every path before `this` is used as a whole (`E2100`), as `[CLS-2]` says
  for a class. `owns this` on an `init` is `E2108`.

## V.4 Enums

```ember
@repr(u8)
enum RenderMode:
    Forward = 0
    Deferred = 1

enum Shape:
    Circle(radius: f32)
    Rect(w: f32, h: f32)
    Empty

    fn area() -> f32:
        return match this:
            Circle(r) => 3.14159 * r * r
            Rect(w, h) => w * h
            Empty => 0.0
```

* `[ENM-2]` A `match` on an enum MUST be exhaustive (`E2090` lists the missing variants); `_` covers
  the rest.
* `[ENM-3]` A unit-only enum is `Copy`, `Eq`, `Ord` (declaration order), `Hash` and `Debug`
  automatically, converts to its representation with `as`, and converts back with
  `Mode.from_repr(x) -> Option[Mode]`.
* `[ENM-4]` A payload enum is `Copy` only by `@derive(Copy)` with every payload `Copy`; `[STR-5]`'s
  implicit derives apply to it.

## V.5 Classes

```ember
open class Script:
    let entity: int
    pub(read) enabled: bool = true

    fn init(entity: int):
        this.entity = entity

    virtual fn on_update(dt: f32):
        pass

class Door(Script):
    angle: f32 = 0.0

    override fn on_update(dt: f32):
        this.angle = min(this.angle + 90.0 * dt, 90.0)
```

* `[CLS-1]` *(changed in 0.9.10)* *(changed in 0.9.15)* A class object lives in its own heap block
  with the header of `[OBJ-1]`, except as `[OPT-1]` permits; the block comes from the built-in
  memory manager, or from the instance the line that makes the object names with `using`
  (`Enemy(hp=5) using current`, `[ALC-1]`). `Name(args)` allocates, runs the constructor and gives
  the object to the place it is stored in, its one owner: a class object has one owner unless a line
  says `shares` (`[OWN-1]`, `[RC-1]`), and `=` from a whole name hands it over (`[OWN-3]`). `owns`
  on a class type is `E2114` ("a `Player` object always lives in its own block; remove `owns`: `p:
  Player`").
* `[CLS-2]` *(changed in 0.9.9)* *(changed in 0.9.10)* *(changed in 0.9.15)* *(changed in 0.9.20)*
  `fn init(…)` is the constructor; it sets the fields of the object it builds through `this`, with
  no word. Before any `init` body runs, every field default of the class and of its bases is
  evaluated and stored. Every field without a default MUST then be set on every path before `this`
  is used as a whole (`E2100`): `this` is used as a whole when it is passed, stored, returned or
  captured, when a link or an owner of it is made (`weakref this`, `shares this`), and by a call of
  a method whose body lets `this` out in any of those ways, which the compiler works out from that
  body, and for a `virtual` method from every override's body the program has (`[DSP-5]`,
  `[EFF-24]`), since a base `init` cannot know which kind it builds. In a class that is `pub` and
  `open` or `abstract`, an override of a `virtual` method may be any dependant's, which this package
  never sees (`[BLD-16]`): its `init` calls a `virtual` method only after every field is set, and a
  call before that is `E2100` ("`init` calls `on_make()` (line 7) before `name` is set (line 8), and
  `Script` is `pub` and `open`, so any package may override `on_make` with a body that reads `name`
  or keeps a link to the object; set every field first: move line 7 below line 8"); in any other
  class every override is the package's own, and the check reads their bodies. Before then `init`
  may call a method of its object once every field that method's body (every override's, for a
  `virtual` method) reads is set: after `this.renderer = r`, `this.assets = this.load_assets()` is
  allowed when `load_assets` reads only `this.renderer`, and is `E2100` when it reads a field not
  yet set ("`init` calls `load_assets()` (line 6), which reads `this.assets` (its line 14), and
  `assets` is set only on line 7; give it a value first: write `this.assets = []` above line 6"); a
  method that lets `this` out is called only after every field is set ("`enrol` keeps a link to the
  player (`ROSTER.push(weakref this)`, its line 2), and `name` is set only on line 9; set every
  field first: move line 6 below line 9"). A `return` from `init`, like the end of its body, hands
  the object to its caller, so every such field is set on that path too. `owns this` on `init` is
  `E2108` ("`init` builds the object, so it cannot take it over; write `fn init(name: String):`").
  Several constructors are written as `static fn`s that call `Self(…)`.
* `[CLS-3]` *(changed in 0.9.20)* A class with no `init` and no base class gets a memberwise
  constructor as a struct does (`[STR-1]`), and that constructor is its `init` for every purpose: a
  derived `init` calls it as `super.init(…)`, with positional or named arguments in field order, a
  field with a default left out keeping the default `[CLS-2]` has already stored
  (`super.init(name=name)` for a base with `name: String` and `visible: bool = true`); `[CLS-11]`'s
  two phases hold around that call as around any `super.init(…)`; and a derived class with no `init`
  gets it as its base's constructor (`[CLS-10]`: `Panel(name=String("side"))`), and passes it on the
  same way to a class derived from it. The arguments of `super.init(…)` take the words a call of
  that constructor takes: none for a memberwise one, whose values are handed over as to every
  constructor (`[OWN-9]`, so the derived `init` holds them `owns`: `fn init(owns name: String, owns
  label: String):`), and, for a written base `init`, the words a call of that constructor takes
  (`[FN-2a]`: none before an argument for its `owns` parameter, `ref mut` before one for its `ref
  mut` parameter).
* `[CLS-10]` *(new in 0.9.9)* *(changed in 0.9.20)* A derived class with no `init` gets its base's
  constructor: `Door(entity)` evaluates `Door`'s field defaults and then runs `Script.init(entity)`
  (the base's memberwise constructor where it declares no `init`, `[CLS-3]`). A derived class with a
  field that has no default MUST declare an `init` (`E2101`).
* `[CLS-4]` *(changed in 0.9.9)* *(changed in 0.9.15)* *(changed in 0.9.20)* A class is final unless
  declared `open` or `abstract`. An `open` or `abstract` class that another package may derive from
  says in its header whether any thread may hold its objects, `implements Send`, `implements
  ThreadShared`, or an interface that needs them, a promise every class derived from it keeps
  (`[THR-8]`, `[THR-9]`); a final class's markers are read from its fields. Methods are non-virtual
  unless `virtual`; replacing one requires `override`; overriding a non-virtual method is `E2110`. A
  method named like an inherited one replaces it (ODR-028): over a virtual method without `override`
  it is `E2111`, and over a non-virtual one it is `E2110`, `override` or not; an `override` has the
  first line of the method it replaces after its name: the same parameter names, words and types (a
  generic base's type parameters standing for the arguments the derived class gives them), the same
  words on `this` and the same result, since a call through the base's slot (`[DSP-2]`) passes what
  the base declared and reads what it promised, and one that differs is `E2113` ("`KeyLog.on` takes
  `e: KeyEvent`, and the `Handler.on` it overrides takes `e: Event`; write `override fn on(e:
  Event):` and test the kind inside: `if e is KeyEvent as k:`"); `init`, `drop` and `static fn`s are
  each class's own. An `override` is itself virtual, so a further subclass may override it again.
  `virtual` in a class that cannot be subclassed (neither `open` nor `abstract`) is `E2112` ("`Door`
  cannot be subclassed, so `virtual` on `on_update` changes nothing; to let subclasses replace it
  write `open class Door:`, or remove `virtual`"). A call through a `virtual` method counts what
  every override the program has changes, worked out from their bodies (`[DSP-5]`), so nothing is
  written about what an override changes. A derived `init` MUST call `super.init(…)` exactly once
  (`[CLS-11]`). Fields cannot be overridden. Base fields are laid out first; there is at most one
  base.
* `[CLS-11]` *(new in 0.9.9)* *(changed in 0.9.15)* Construction is two-phase. In a derived `init`,
  the code before `super.init(…)` MUST set every field the class itself declares without a default,
  and MUST NOT read an inherited field or use `this` as a whole; the code after it may do anything
  (`E2102`, naming the field or the use). So when any method runs, even a `virtual` method called
  from a base `init`, every field of the object holds a value.
* `[CLS-5]` *(changed in 0.9.15)* Interface calls on a class object are dispatched statically unless
  the object is a `dyn` value.
* `[CLS-6]` *(changed in 0.9.15)* *(changed in 0.9.20)* Destruction runs the derived `drop`, then
  the base's, then ends the fields in reverse declaration order, then frees the block once no
  `weakref` link to it remains (`[OBJ-3]`). The object is ended before its `drop` runs (`[WK-3]`).
  The kept results of the object's `@cached` methods (`[CLS-12]`) end with its fields.
* `[CLS-7]` *(changed in 0.9.9)* *(changed in 0.9.15)* **`this` in a class method.** Inside a class
  method `this` is the object the method was called on, for the whole call. What a method does to it
  is worked out from its body, and `this` is never the only name of its object (`[EXC-15]`): a
  method whose body changes `this` may change it and is not called through a name that only reads it
  (`E3023`), and one whose body neither changes nor ends anything of `this` only reads it. Giving
  `this` another object in a class method is `E2103` ("`this = Node()` cannot replace the `Node`
  object `reset` was called on; set its fields instead: `this.v = 0`"); a function that gives a
  caller's place another object takes it as a `ref mut` parameter (`[FN-9]`) (ODR-072). A method
  changes its object through `this` with no word: `mut this` is `E0113` and `mut self` `E1010`, each
  with the help `fn reset():`.
* `[CLS-7a]` *(changed in 0.9.15)* Inside `drop`, `this` MUST NOT be stored anywhere that outlives
  the call, and no owner of it is made (`shares this`, `thread_shared this`): the object has already
  ended (`E3016`, `[OBJ-5]`). A `drop`'s first line is `fn drop():`; it changes its object as a
  given (`[DRP-1]`).
* `[CLS-9]` *(changed in 0.9.10)* *(changed in 0.9.15)* A `let` field is set only in `init`
  (`E2104`).
* `[CLS-9a]` *(changed in 0.9.15)* *(changed in 0.9.20)* After `init`, nothing is changed through a
  `let` field, for every type, inside `drop` included (`[DRP-1]` waives only the word `changes
  this`; a view field `from` a `let` field is still read in `drop`, `[GRM-46]`, and a `let` field
  that must be flushed or closed does that in its own type's `drop`, which runs when the field ends,
  after the object's, `[CLS-6]`): a number or struct field never changes, a list field is never
  added to, and a class-object field is never pointed at another object nor its object changed
  through it (`E2104`: "`items` is `let`: it is set in `init` and never changed after; to add to it,
  write the field without `let`: `items: Array[int]`"). Other owners of a `shares` object change it
  as `shares` allows (`[RC-7]`); `let` speaks only of the field it is written on.
* `[CLS-12]` *(new in 0.9.15)* *(changed in 0.9.20)* **`@cached`.** `@cached` above a class method
  with no parameters keeps the method's result after the first call, in the object (a struct or enum
  has no hidden state, `[PHIL-9]`, `[TYP-11]`), and gives it again until a change, through any name,
  of a field the method's body reads clears it (`[EFF-27]`). A `Copy` result is returned by value,
  any other as a `ref T` view into the kept value, which clearing it ends (a later use is `E3060`).
  On an object of a class whose objects several threads may reach, read from the class's declaration
  (it promises `ThreadShared` or implements an interface that needs it, `[THR-8]`, or its methods
  pin it `thread_shared`, `[OWN-13]`; never from which values a program makes `thread_shared`,
  `[BLD-16]`), the kept result is filled once under a once-flag. A class with a `@cached` method
  that declares none of these is not `ThreadShared` (`[THR-9]`): no object of it is made
  `thread_shared` or held by a `static` (`E7015`, `E7002`, naming the `@cached` line), since the
  stores below would race. It costs one flag test per call and one store per change of a field the
  body reads. `@cached` is `E2117`, each message giving the `@cached` line to remove: on a method of
  a struct or an enum ("a `Vec2` has no hidden state to keep a result in; keep it in a field of a
  class, or remove `@cached` (line 2)"); on a method with parameters ("`@cached` keeps one result,
  and `radius` takes `scale`; remove `@cached` (line 2), or keep the results in a field: `radii:
  Map[f64, f64]`"); on one whose body changes anything ("a cached method keeps one result, so it
  only reads, and `radius` changes `this.verts` (its line 4); remove `@cached` (line 2)"); and on
  one whose body reads a `shares`, `shares mut`, `thread_shared`, `weakref`, `sharedref mut` or
  `Cell` field, or a field another thread may change by its own operations, under a lock or not (an
  `Atomic`, a `Mutex`, an `RwLock`, a channel end, a field `guarded by` a lock): on an object of
  such a class a kept result is filled under a once-flag and never cleared, so the method reads only
  fields that never change after `init` ("`radius` reads `verts`, which other threads change under
  `lock`; a kept result could be stale; remove `@cached` (line 18)"; "`summary` reads `hits`, an
  `Atomic` other threads change, and a kept result on a shared object is never cleared; remove
  `@cached` (line 6)"); and on a `virtual` method or an `override` ("`label` is `virtual`, and a
  kept result follows one body; keep it in a method no subclass replaces, or remove `@cached` (line
  3)"). For a method that reads a value its other owners may change: E2117 ("`radius` reads `mesh`,
  which its other owners may change without `Mesh` seeing it, so a kept result could be stale;
  remove `@cached` (line 13)"). A change made by foreign code, or through a raw pointer, passes
  through no name the compiler sees: unsafe code that changes a field a `@cached` method reads
  answers for it, as for every obligation of `unsafe` (`[UNS-4]`). A `@cached` body reads nothing a
  link may change without passing through the holder: reading a field of a class object reached
  through a field or element of `this` (not `this` itself), of a class that carries a weak count
  (`[OBJ-1]`), is `E2117` ("`first_name` reads `items[0].name`, and a `Book` may be changed through
  a `weakref Book` link, made from any name before or after the book entered `shelf` (`[WK-11]`) and
  opened `ref mut` (`[GRM-44]`), without `Shelf` seeing it, so a kept result could be stale; remove
  `@cached` (line 7), or keep the names in `Shelf` itself: `names: Array[String]`"); a class that
  carries no weak count is named by no link, and a `sharedref mut` link or `shares mut` owner into
  the read set is refused where it is made (`[EFF-27]`). `@cached` is `E2117` too on a method whose
  body reads anything not reached through `this` that may change (a `static mut`, or a `static`
  `Mutex`, `RwLock`, `Atomic`, `Cell`, channel end or `guarded by` variable, `[STA-1]`, directly or
  through a call, `[EFF-24]`), since a change of it passes through no name of the object and clears
  nothing ("`area` reads `SCALE`, a `static mut` that may change without `Circle` seeing it, so a
  kept result could be stale; remove `@cached` (line 5), or keep the scale in the object: `scale:
  float`").

## V.6 Interfaces and `extend`

```ember
pub interface Shape2D:
    fn area() -> float
    fn describe() -> String:
        return f"shape with area {this.area():.2f}"

struct Square:
    side: float

extend Square implements Shape2D:
    fn area() -> float:
        return this.side * this.side
```

* `[IFC-1]` *(changed in 0.9.9)* `extend T:` without `implements` adds inherent methods to `T`; it
  is legal in any module of `T`'s package.
* `[IFC-2]` Adding inherent methods to a type from another package is `E2120`; use an interface or a
  wrapper type.
* `[IFC-3]` *(changed in 0.9.9)* *(changed in 0.9.20)* `interface Ord: needs Eq` requires
  implementers of `Ord` to implement `Eq`: an implementation meets its interface's parents for every
  type it covers, checked where it is written (`E2040`, naming the part that does not and giving the
  line that compiles). A generic implementation covers every instance its brackets allow, so it
  writes the needs the parents put on its parameters (`extend[T: needs (Send, ThreadShared)]
  PoolFull[T] implements Error:`; `extend[T] PoolFull[T] implements Error:` is `E2040` with that
  line, never read as holding for some arguments only), as a `pub` generic function writes its
  bounds (`[IFC-6]`). A class that implements an interface with marker parents (`Send`,
  `ThreadShared`) promises them for every class derived from it, each checked where it is declared
  (`[THR-8]`), and the implementation itself is never checked again. So a bound brings its parents:
  `T: needs IndexMut[int]` provides `Index[int]`'s methods too, and a binding it writes (`Output =
  f32`) names the parent's associated type (ODR-042). An implementation writes each of the
  interface's methods with the interface's first line after the name: the same parameter names,
  words and types (with `Self`, the interface's type parameters and its associated types standing
  for the implementing type and what the `implements` clause and the `type` lines give them, so `fn
  eq(other: Point)` meets `fn eq(other: Self)`), the same words on `this` and the same result, since
  a call through `dyn I` (`[DSP-3]`) passes what the interface declared; one that differs is
  `E2040`, naming the method and the interface's line ("`Sq.scale` takes `k: i64`, and
  `Scaler.scale` takes `k: f64`; write `fn scale(k: f64) -> f64:`").
* `[IFC-4]` *(changed in 0.9.9)* *(changed in 0.9.20)* Interfaces may declare associated types and
  constants, not statics. An associated type may carry bounds (`type Real: needs Float`), and a type
  implementing the interface states it (`type Real = f64`) with a value that meets them (`E2040` if
  it is missing or does not). `T.Name` names the associated type `Name` of a type parameter `T`
  whose bound declares it: inside the generic body it has what its declared bounds provide, and each
  call takes it from what the argument's type states (ODR-038). An `extend` block that implements
  several interfaces states an associated type once, for each of them that declares it (`extend i32
  implements Add, Sub: type Output = i32`). `Name = T` inside an interface's brackets is written
  only in a bound (`T: needs Add[Output = T]`), and is `E2020` in an `implements` list. A bound that
  leaves an associated type unsaid (`T: needs Add`) makes `a + b` a `T.Output`, which only a
  signature naming it (`-> T.Output`) can hold (`E2040` otherwise) (ODR-040). An interface may give
  an associated type a **default** (`type Out = Self`): an implementation that does not state it
  takes the default, read with `Self` as the implementing type, and the default must meet the
  associated type's bounds (`E2040`). A default is not an equality: a bound `T: needs I` leaves
  `T.Out` abstract whatever the default says, and a default method body sees `Out` as itself, never
  as its default, since an implementation may override it (ODR-049). A default that leads back to
  itself, directly or through other defaults, is `E2043`.
* `[IFC-5]` *(new in 0.9.15)* *(changed in 0.9.20)* **Interface values say who owns them.** `owns
  dyn I` is a value of some type implementing `I`, held alone in a block of its own; `shares dyn I`
  and `shares mut dyn I` are one of several owners of such a value, reading it or changing it too;
  `dyn I` as a parameter's type is a read-only view (`ref mut v: dyn I` may change it), and `dyn I`
  with no owner word is written only as a parameter's type: a view of an interface value anywhere
  else (a local, a field, an element, a result) is `ref dyn I`, or `ref mut dyn I` to change it,
  which `[TYP-15]` and `[TYP-34]` govern as every view (`s: ref dyn Shape = ref sq`; `-> ref dyn
  Shape from xs`), and a bare `dyn I` in such a position is `E2173` ("`main: dyn Shape` does not say
  whether `Scene` owns or views its shape; write `main: owns dyn Shape` to own it, or `main: ref dyn
  Shape` for a view"). A bare interface name used as a type is `E2173` ("`Shape` is an interface, so
  `Array[Shape]` does not say who owns the shapes; say who owns them: write `shapes: Array[owns dyn
  Shape]` in place of line 19 (the list is their only owner), and `scene = Scene(shapes=[owns
  Square(side=2)])` in place of line 22"). A generic method whose type parameter is only the type of
  a `ref` or `ref mut` parameter gets one entry for `dyn` use; an interface whose generic method
  returns or stores its type parameter is not usable as `dyn` (`E2050`, `[TYP-22]`: "`Inspect`
  cannot be used as `dyn Inspect`: `make` returns its `V`, and a `dyn` value has one entry per
  method; take the visitor in instead of returning it, `[V: needs Visitor]` above `fn make(ref mut
  v: V)`, or keep each list's own type with a type parameter instead of `dyn Inspect`: `[I: needs
  Inspect]` above `fn walk(xs: Array[I]) -> int:`"). A value of an interface type is tested for its
  kind with `is` and cast with `cast x as T` (`[DSP-4]`, `[DSP-6]`). What a `dyn` value's methods
  change is worked out from every value the program makes into that type (`[EFF-2]`).
* `[IFC-6]` *(new in 0.9.15)* **Bounds of a generic function.** A function that is not `pub` writes
  all of its type parameters' bounds or none. With none, they are worked out from its body, and a
  method that no interface in scope declares is `E2072` ("`T` has no `weight`: no interface in scope
  declares `fn weight`; declare one, `interface Weighed:` with `fn weight() -> int`, and write
  `struct Crate implements Weighed:` for the type `heaviest` is given; the bound follows"). A `pub`
  function writes them (`[MOD-9]`): a missing one is `E2040`, one message, on the first line of the
  body that needs a bound the function does not write, giving every bound the body needs ("`total`
  is `pub`, so its bound is written: write `[T: needs Add[T, Output = T]]` in place of `[T]` above
  `pub fn total(a: T, b: T) -> T:`"). A function that is not `pub` and writes some of them writes
  every one its body needs: a list that leaves one out is `E2040`, whose message gives the whole
  line (`[T: needs Lookalike]` above `fn direct_add(a: T, b: T) -> T:` with `return a + b`, where
  `Lookalike` declares an ordinary `fn add` and is not `Add`, gets `[T: needs (Lookalike, Add[T,
  Output = T])]`); nothing is added to a written list (verdict 113).

## V.7 Constants and statics

* `const NAME: T = e` is a compile-time constant, inlined at each use; `e` MUST be evaluable at
  compile time (Part XIV). Written without `: T` and with an untyped numeric literal as `e`, it is
  untyped (ODR-037): each use is that literal and takes its type as the literal would there
  (`[TYP-23]`), `int` or `float` where nothing fixes it. `T` MUST be `Copy`, `StrView`, a `bytes`
  literal type, or a fixed array of those (`E2130` otherwise, whose help suggests a `static`). A
  `const` in the body of a `struct`, `enum`, `class` or `extend` block belongs to the type: it is
  named `T.NAME`, or `Self.NAME` inside the type, and is private to its module unless `pub`
  (`[MOD-2]`). Constants may name one another in any order; one whose value depends on itself is
  `E6001`, and a panic while one is evaluated is `E6004` (`[CT-7]`) (ODR-045).
* `[STA-1]` *(changed in 0.9.9)* *(changed in 0.9.15)* *(changed in 0.9.20)* `static NAME: T = e` is
  one value per program with a stable address, and it never changes; shared state that changes goes
  through an `Atomic`, `Mutex`, `RwLock` or `Once` in it. An assignment to it (`HOOK = v` in a
  function) is `E3023`, whose help names `static mut HOOK` or another name for a local; it never
  makes a local that hides the `static`. `T` MUST be `ThreadShared` (`E7002`). `static mut NAME: T =
  e` is a module variable that may change, written with no `unsafe`: a function changes it by its
  name, with no word (Python's `global` is `E0100`: "`global` is Python's; in Ember a function
  changes a module variable by its name, with no word: remove line 9"), and what a function changes
  is worked out from its code (`[EFF-24]`). It is reached only by the main thread and the tasks and
  jobs of the scopes it opens (`[THR-20]`) unless it is `guarded by` a `static` `Lock` (`static mut
  LOG: Array[String] guarded by LOG_LOCK = []`); another thread that reaches it otherwise is `E7012`
  (Part XI). A `guarded by` `static mut` is a `Mutex[T]` written as a module variable (`[THR-18]`),
  so its `T` MUST be `Send` (`E7002`: "`CACHE` is guarded by `L`, so any thread may reach it under
  the lock, and a `shares mut Table` stays on one thread; share it between threads: write `static
  mut CACHE: thread_shared Table guarded by L = thread_shared Table()`"); an unguarded `static mut`
  asks nothing of `T`, since one thread and its scoped tasks reach it. A statement at module level
  outside the entry file (where statements form the implicit `main`, `[FN-8]`) that would give a
  module variable a value is `E0100`, whose help gives the declaration ("`FRAME = 0` is a statement,
  and a module holds declarations; for a module variable that may change write `static mut FRAME:
  int = 0`"). A foreign `static mut` (declared in an `extern` block) is reached only inside `unsafe`
  (`[UNS-1]`). A foreign static read only in `unsafe` need not be `ThreadShared`: the programmer
  answers for sharing it between threads; one read without `unsafe`, under `@ffi(immutable)`
  (`[FFI-10]`) or as a header's `const` object (`[FFI-8]`, `[FFI-41]`), must be (ODR-163).
* `[STA-3]` *(new in 0.9.9)* *(changed in 0.9.15)* *(changed in 0.9.20)* A static whose initialiser
  can be evaluated at compile time is placed in the image with no initialisation code, except a
  value that owns heap memory (an `Array`, `String`, `Map`, `Set`, an `owns T`, a class object) in a
  `static mut`, or in a `static` whose type is one a plain name may change (`[CT-5]`: a `Mutex`,
  `RwLock`, `Cell`, `Atomic`, `Once`, channel end or `guarded by` field on the path to it, or a
  class with a `@cached` method): that value is built on first access, never in read-only memory, so
  that nothing grows, changes or frees constant data. Any other initialiser runs **once**, on first
  access, exactly once even under concurrent first access; the first access blocks until it
  completes. A panic during initialisation aborts. An initialiser that reaches its own static is
  `static initialisation cycle`, a panic naming the chain.

```ember
from std.sync import Mutex

static GREETING: StrView = "hello"
static REGISTRY: Mutex[Map[String, int]] = Mutex({})
static mut FRAME: int = 0

fn tick():
    FRAME += 1
```

* `[STA-2]` There is no static-initialisation-order problem: statics are initialised at compile time
  or on first access, never by a global constructor.

## V.8 Attributes

* `[ATT-1]` *(changed in 0.9.9)* *(changed in 0.9.15)* An attribute that is neither in the table below nor a visible
  `@attribute` struct (`[RFL-3]`) is `E0104`; one that Ember no longer has is `E0104` with the one to
  write (`@sync`: "`@sync` is gone: a class needs no mark to be shared between threads; remove `@sync`",
  and the value says `thread_shared`, each field that changes `guarded by` or `Atomic`;
  `@static_safe`: `@no_alias_checks`; `@fp(contract)`: `@fp(allow_fma)`; `@borrows(p)`: `from p` after
  the result type, `[LT-1a]`; `@component`: any type is a component, with no mark (Part XII); `@no_runtime_checks`: no switch turns a safety check off, `[UNS-11]`). An
  attribute on a declaration it does not apply to is `E0104` naming the positions it does apply to.
* `[ATT-4]` One attribute per line.
  A module attribute uses a `#! module` directive (`[GRM-37]`). Its effect and any item-level
  override follow the attribute's own rule: overflow uses `[TYP-8]`, deterministic applies to
  every function without opt-out (`[DET-8]`), and reload attributes retain `[HR-25]`'s item priority.
* `[ATT-6]` *(new in 0.9.9)* Every attribute in the table has exactly the effect its rule gives. An implementation that
  has not built an attribute's effect rejects the attribute with `E0900` (`[PHIL-12]`); it never
  accepts and ignores it.
* `[ATT-2]` *(changed in 0.9.15)* A statement may carry only `@parallel`, `@unroll`, `@simd` and `@allow`; the first three
  only on a `for` statement (`E0108`).

| Attribute | Applies to | Rule |
|---|---|---|
| `@derive(A, …)`, `@no_derive(A, …)` | struct, enum, class | `[STR-5]`, `[DRV-1]` |
| `@layout(c)`, `@packed`, `@align(N)` | struct, enum | `[LAY-2]` |
| `@repr(T)` | enum | `[LAY-2]` |
| `@gpu_layout(std140\|std430\|scalar)` | struct | Annex D |
| `@noalloc`, `@nosync`, `@noblock`, `@noio`, `@nolock`, `@nopanic(explicit)`, `@no_alias_checks`, `@deterministic`, `@realtime` | fn, interface method, and a `type` line naming a function type (`@noalloc` above `type Handler = fn()`, `[EFF-2]`); `@deterministic` also module | Part X |
| `@overflow(panic\|wrap\|saturate)` | fn, module | `[TYP-8]` |
| `@fastmath`, `@fp(allow_fma)` | fn | `[TYP-9]` |
| `@inline`, `@noinline`, `@cold`, `@hot` | fn | `[CG-C-3a]` |
| `@simd`, `@simd(assert)`, `@parallel(…)`, `@unroll(N)` | `for` statement | Part XII |
| `@must_use` | fn, type | `[ERR-5]` |
| `@deprecated(since, note)` | any item | `[VER-3]` |
| `@unsafe(export("symbol"))`, `@unsafe(export_table("Name", protocol=N))` (the plain `@export(...)` and `@export_table(...)` are `E3100`) | fn, static; struct | `[FFI-26]` |
| `@ffi(…)` | extern item, overlay item | Part XVI |
| `@test`, `@bench`, `@should_panic` | fn | `[TST-3]` |
| `@view` | struct, enum | `[TYP-34]` (optional; checked: `E2030` on a type that holds no view) |
| `@cached` | class method with no parameters | `[CLS-12]` |
| `@reflect` | type | `[RFL-2]` |
| `@safety("…")` | unsafe fn | `[UNS-7]` |
| `@must_drop` | struct, class | `[THR-6]` |
| `@allow(code, …)` | any item or statement | suppresses the named `W`/`L` diagnostics |
| `@non_exhaustive` | enum | `[FFI-8]` |
| `@soa(flatten)` | field | Part XII |
| `@always_specialize`, `@never_specialize` | generic fn or type | `[MONO-7]` |
| `@reloadable`, `@noreload`, `@renamed_from("n")`, `@reinit_on_reload`, `@allow_reload_terminate` | see Annex B | Annex B |
| `@prelude` | module (standard library only) | `[MOD-5]` |
| `@attribute` | struct | `[RFL-3]` |
| `@error("…")`, `@from` | variant of an enum deriving `Error` | `[ERR-3]` |
| `@default` | variant of an enum deriving `Default` | `[DRV-1]` |
| `@ser(skip\|rename="…"\|default\|since=N)` | field of a type deriving `Serialize` | `[SER-1]` |
| `@grade(…)` | overlay item | `[TCB-1]` |
| `@assume_noalloc(call)` | a call inside `unsafe` (an expression) | `[EFF-7]` |

*Reserved* (recognised and rejected with `E0104` naming the version): `@nopanic` (without
`(explicit)`), `@allocator(Name)`, `@gpu`.
