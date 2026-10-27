---

# Part VII — Ownership and Views

This Part specifies ownership and **views** for every type: structs, enums, collections, strings and
class objects alike. A value has one owner unless a line says otherwise; a name that points into a
place someone else owns is a view, and what a view may do is said by the word in front of it. Everything
in this Part is checked when compiling: nothing here runs while the program runs except a value's own
clean-up. How a class object is laid out, how `shares` owners are counted and how `weakref` links work
is Part VIII.

## VII.1 Terms

These words are used with these meanings throughout the specification.

- **owner**: the one place (a local, a field, a list element, a temporary, a `static`) that ends a value
  when it ends. A value made shared has several owners (`shares`, `[RC-1]`).
- **handed over**: the giver's name ends and the receiver becomes the owner, for every type, numbers
  included, wherever a line hands a value over (`snapshot = items`; an argument for an `owns`
  parameter; `return a`; a function's capture `owns count`; `[OWN-3]`, `[OWN-9]`).
- **copy**: a new value equal to the old one; nothing done to one is seen through the other. `copy of`
  copies the thing itself, never the link it is written before (`[OWN-8]`).
- **`Copy` type**: a type whose `=` makes such a copy bit for bit: the numbers, `bool`, `ref T`,
  `Span[T]`, `StrView`, and structs made only of these. Class objects, `shares` values, `weakref` and
  `sharedref mut` links, `ref mut` views and `MutSpan`s are not `Copy`.
- **place**: where a value lives: a local, a field, a list element, an enum variant's payload.
- **part**: a field (`p.name`), an element (`enemies[0]`), or a payload (`State.Walking(path)`).
- **view**: a name or value that points into a place someone else owns and may not outlive it:
  `ref T`, `ref mut T`, `sharedref mut T`, `Span[T]`, `MutSpan[T]`, `StrView`, a plain parameter, a
  plain loop or pattern name, a name opened with `exists as` (not `exists as copy of`, which is a copy
  of your own) or named by a kind test (`is T as`), a name a function captured with `ref`, a name for a
  part. A **read-only view** is every view except `ref mut T`, `sharedref mut T` and a `MutSpan[T]`
  held by a `ref mut` name. A **kept view** is a view stored in a struct or a list.
- **into X**: a view points into X when it points at X or at anything X owns, all the way down (X's
  fields, the elements of X's lists, X's `owns` parts). It does not go through links: what a link
  reaches is another object.
- **link**: a field or value that names an object it does not own alone: `weakref T` (does not keep the
  object alive), `shares T`, `shares mut T` and `thread_shared T` (one of several owners), and
  `sharedref mut T` (a view several holders may share, `[LT-46]`).
- **position**: a view of a list element that finds the element again at each use (`[BRW-16]`).
- **reshape**: a change that can free or move memory: replacing a value that owns memory; `push`,
  `pop`, `insert`, `remove`, `clear`, `sort`, `retain`, `extend` on a list, map, set or string;
  changing an enum's variant; ending an object's last owner. Writing a number or a `bool` is not a
  reshape. A reshape moves or frees only what it may move or free: adding to a list (`push`, `extend`)
  may move its buffer and every value stored inline in it, but not a class object, which lives in its
  own block and is not moved when the list grows.
- **ended**: an object is ended the moment its last owner lets it go, before its `drop` runs
  (`[WK-3]`); ending it counts as a call of its `drop` (`[DRP-8]`).
- **summary**: what the compiler works out from a function's code, with no word written (`[EFF-1]`):
  what it may change, read and end, and through which calls (the chain of calls a message names); which
  elements of a list it touches; which of its view parameters, `this` included, it uses after a line
  that may end or move what they view; and what it reaches by other names while a `ref mut` parameter
  of it is used. Across a package's edge it is recorded in the package's interface file (`[EFF-4]`).

## VII.2 Ownership

* `[OWN-1]` *(changed in 0.9.15)* Every value has exactly one owner: a local, a field of an owned value,
  an element of an owned collection, a temporary, or a `static`, unless a line makes it shared
  (`shares x`, `thread_shared x`, `[RC-1]`). This holds for every type: a class object has one owner,
  as a struct value does.
* `[OWN-2]` *(changed in 0.9.15)* A value is **dropped** (ended) when its owner goes out of scope (block
  end, reverse declaration order), when its owner is overwritten, or at the end of the statement that
  created it as a temporary. Dropping runs the value's `drop` method, if any, then drops its fields in
  reverse declaration order. A value always ends with its clean-up: nothing ends an owner without
  running it, and there is no `mem.forget` (`[OWN-6]`); a value given to C is handed over to it
  (`[OWN-12]`).
* `[OWN-3]` *(changed in 0.9.15)* **`=` with a whole name hands it over.** `b = a`, where `a` is a whole
  name that owns a value whose type is not `Copy`, hands `a` over: `a` ends. Using it afterwards,
  whether it is read, handed over again or viewed (a method call, `print`), is `E3040`, labelled at the
  hand-over and at the use, and the message names both ways out: "`items` was handed over to
  `snapshot` on line 3 and is used on line 4; to keep both, write `snapshot = copy of items` (a copy of
  your own) or `snapshot = shares items` (two owners of one list)". Where `a` is itself a view (a plain
  parameter, a loop, pattern or opened name, a local holding a view), `b = a` makes `b` a read-only view
  of what `a` views, and `ref` may say so (`cur = ref p`); a view hands nothing over. For a `Copy` type
  `=` copies. A value handed over on only some paths is tracked with a hidden drop flag; a hand-over
  on some paths is legal, and a use a hand-over may have reached is `E3040` too.
* `[OWN-4]` *(changed in 0.9.15)* A loop body that hands over a value declared outside the loop is
  `E3040`, unless every path gives it a value again before the next round.
* `[OWN-5]` Assigning to a place that holds a live value evaluates the new value, then drops the old
  one, then stores the new one. `Cell.set` stores first and drops after (`[CELL-1]`); `MaybeUninit.write`
  stores without dropping (`[ARN-8]`). These three orders are distinct and none generalises the others.
* `[OWN-6]` *(changed in 0.9.9)* *(changed in 0.9.15)* `mem.take(ref mut place: T) -> T` (leaves
  `Default`), `mem.replace(ref mut place: T, owns new: T) -> T`, `mem.swap(ref mut a: T, ref mut b: T)`
  and `mem.drop(owns x: T)` (ends `x` now: the name given to it ends, for every type, numbers included,
  as for every `owns` parameter, and a later use is `E3040`, ODR-065) are the operations that take
  values out of places a hand-over may not otherwise leave empty. A call writes `ref mut` before an
  argument the callee may change (`[FN-2a]`): `mem.take(ref mut this.items)`. There is no
  `mem.forget`: it is `E1010`, "`mem.forget` is gone: a value always ends with its clean-up; to give
  the widget to a C function that keeps it, hand it to a parameter its declaration says it takes
  (`widget_take(first)`), or, inside `unsafe:`, give it `first.into_raw()`".
* `[OWN-9]` *(new in 0.9.15)* **`owns` hands over, for every type, numbers included.** An argument for
  an `owns` parameter is handed over with no word: `f(n)` ends `n` when `f` takes `owns x: int`, and
  `f(copy of n)` keeps the caller's own. `return a` hands `a` over too. A number given with `=` is
  copied, so `b = owns n` is how `=` hands a number over: `n` ends there, and a later use is `E3040`.
  `owns` is not written before a name whose place already hands the value over, nor before `fn`
  (`E0109`: "`e.mesh = m` already hands `m` over; remove `owns`"; "`b = a` already hands `a` over; remove
  `owns`"; "`owns fn` does not say what the function takes or copies; name it on each captured name:
  `fn() owns report:`").
* `[OWN-10]` *(new in 0.9.15)* **`owns` on the line that makes a value for a place that owns it
  alone.** An `owns` field, an `Option[owns T]` and a list of `owns` values each hold their value in a
  block of its own, and the line that makes a value for such a place says `owns`, at first and later:
  `e = Entity(mesh=owns Mesh(points=[0.0]))`; `boss = Enemy(hp=500, mesh=owns Mesh(points=[0.0, 1.0])
  using current)` (`[ALC-1]`); `t = Tree(value=1, left=Some(owns Tree(value=2)))` for
  `left: Option[owns Tree]`; `r.pools.push(owns IntPool(items=[1, 2]))` for
  `pools: Array[owns dyn Pool]`; `e.mesh = owns Mesh(points=[1.0, 2.0])`, which frees the old mesh. The
  line without it is `E2123` ("`mesh` owns its mesh; say so where it is made:
  `mesh=owns Mesh(points=[0.0, 1.0]) using current`"; "`left` holds its tree in a block of its own; say
  so where it is made: `left=Some(owns Tree(value=2))`"). `owns` is also written where the default is
  a view, before a `for` loop's or a `match`'s source (`for x in owns xs:`, `match owns e:`), which then
  hands the value over. `owns` before a class type, whose object always lives in its own block, is
  `E2114` ("a `Player` object always lives in its own block; remove `owns`: `p: Player`").
* `[OWN-11]` *(new in 0.9.15)* **A part handed over on its own.** A part of a struct value stored
  somewhere else (`push`, `return`, a field, an `owns` argument) is handed over on its own: for a struct
  value `p`, `names.push(p.name)` moves `p`'s name into the list, and `p.hp` stays usable; `p.name`,
  and `p` used whole, are `E3040` until `p.name` is given a new value (`p.name = String("")`): "`p.name`
  was handed over to `names` on line 4; give it a new value first, or keep a copy:
  `names.push(copy of p.name)`". A field of a class object is not handed out this way, since other names
  may reach the object and would see the hole (`E3013`: "`q` is a `Player` object, which other names
  may reach, so `q.name` cannot be taken out and leave a hole; take it and leave a value in its place:
  `names.push(mem.take(ref mut q.name))`"). A hand-over out of a name or place that does not own the
  value is `E3013` too: "`v.push(template)` would hand over the caller's player, but `template` only
  reads it; to put a player of your own in the list write `v.push(copy of template)`".
* `[OWN-12]` *(new in 0.9.15)* **Giving an object to C.** A C function whose declaration says it keeps
  the object it is given (`[FFI-11]`) takes it by an ordinary hand-over, `widget_take(first)`: the name
  ends and Ember frees nothing. For a C function whose declaration cannot say so (a plain pointer
  parameter, a function with `...`), `second.into_raw()` gives the object's address and stops Ember
  from freeing it; the compiler makes that step itself, so nothing else can skip a value's clean-up.
  `into_raw()` outside `unsafe:` is `E3100` ("`into_raw()` stops Ember from freeing the widget, so a
  mistake leaks it or frees it twice; write it inside `unsafe:`", `[UNS-1]`).
* `[OWN-13]` *(new in 0.9.15)* **Words before a value.** Before a value, a word acts on the object the
  value names (for a link or an owner, on its target): `copy of x` (`[OWN-8]`); `owns x`
  (`[OWN-9]`); `shares x`, one more owner, on this thread, that reads it, and `shares mut x`, one more
  owner that may also change it (`[RC-1]`); `thread_shared x`, one more owner on any thread, written only
  before a new value (`names = thread_shared Names()`) or a value already `thread_shared` (`E7017`
  otherwise: "`names` is `share_and_keep`'s own object, which it may change; to share it between
  threads, create it shared: `names = thread_shared Names()`"); `weakref x`, a link that does not keep it
  alive (`[WK-11]`); `sharedref mut x`, a changeable link other holders may share (`[LT-46]`); and
  `ref x` and `ref mut x`, views (`[BRW-12]`). An owner made from another may do no more than it:
  `shares mut x` needs `x` to be a name that may change the value (`E3023`: "`g2 = shares mut game`
  would change the `Game` through `game`, which only reads it (`game = shares Game()`, line 1), and an
  owner made from it only reads too; write `g2 = shares game`, or, for two owners that both change the
  game, write `game = shares mut Game()` instead of line 1"). A plain parameter of a `shares mut` type
  only reads the caller's value (`[FN-1]`), so an owner made from it only reads too; a function that
  hands on an owner that may change takes one: `fn setup(owns game: shares mut Game)`, called
  `setup(shares mut game)`. In a lambda's capture list each word means
  what it means before a value (`[CLO-2]`). In a list of names declared without types of their own (a
  capture list, a loop's names, a pattern's names, a lambda's parameters written without types) each
  word belongs to the one name after it, and a name with no word may not follow a name with a word, so
  that no word can be read as covering the next name (`E0116`: "`ref mut` here belongs to `pos` only;
  say what `vel` is: `for ref mut pos, ref vel in q:`, or `ref mut vel` to change it too").

## VII.3 Copy and `copy of`

* `[OWN-7]` *(changed in 0.9.15)* A value of a `Copy` type is duplicated bit for bit by `=` and on use,
  and the source stays valid; the duplicate is independent, and no copy runs any code. A `Copy` type
  holds no owner and no link: class objects, `shares` values, `weakref` and `sharedref mut` links,
  `ref mut` views and `MutSpan`s are not `Copy`, and a type with a `drop` of its own is never `Copy`
  (ODR-051). A `Copy` struct copied with `=` from a name that is still used afterwards is not then
  changed through the new name (`E3023`): the line says `copy of` (a copy of your own) or `ref mut`
  (the original), so that no copy is changed by mistake.
* `[OWN-8]` *(changed in 0.9.15)* **`copy of x`** makes a copy of the thing itself, by its type's
  `Clone` (`fn clone() -> Self`, the explicit deep copy): of a class object, a new object
  (`p2 = copy of player`); for a `shares` or `sharedref mut` link, a copy of its target. Every `Copy` type
  has `Clone` (`@derive(Copy)` gives it), so `b = copy of a` works for a `Point` marked only `Copy`. A
  value that holds links is copied field by field, as a C++ struct holding a `shared_ptr` and a
  `weak_ptr` is: what it owns alone is copied itself, and each link is copied as a link to the same
  target. For `struct Team:` with `captain: shares Player` and `scout: weakref Player`, `t2 = copy of t`
  is a new `Team` whose `captain` is one more owner of the same player and whose `scout` is one more
  link to it; a `sharedref mut` field is copied as one more holder of the same link. `copy of` is
  `E2116` before what it cannot copy as itself:
  * a `weakref` link, whose target may be gone: "`copy of this.keep` would copy the link, not the node;
    for another link to the same node write `a = weakref this.keep`; for a copy of the node itself write
    `if this.keep exists as n:` and then `c = copy of n`";
  * a value of a view type (`StrView`, `Span[T]`, a struct holding a `ref mut` field): "`w` is a view of
    `line`'s text; to make your own copy write `String(w)`";
  * a type with no `Clone`: "`Player` has no `Clone`, so `copy of` cannot make one; write
    `@derive(Clone)` above `class Player`, or keep a view of it: `p2 = ref p`";
  * a value whose views point into its own `let` field: "`d.words` points into `d.text`, so a copy's
    words would point into `d`'s text; make the copy from its text: `d2 = Document(String(d.text))`".

## VII.4 Views: names, parameters and calls

A **view** is made where a name is bound to a part or to a call that gives a view, where a place is
given to a plain or `ref mut` parameter, walked by a loop, opened or tested, or converted to a `Span`,
`MutSpan` or `StrView`, and where a line writes `ref`, `ref mut` or `sharedref mut` before a place. A
view hands nothing over and makes no copy.

* `[BRW-12]` *(new in 0.9.15)* **A name for a part is a read-only view.** A name is declared by the line
  that first gives it a value; a line inside a loop's body declares it once per round (`[GRM-4]`). A
  name bound to a part (`n = p.name`, `boss = enemies[0]`), or to a call whose result is a view
  (`ref T`, `StrView`, `Span[T]`, a `ref` or `ref mut` place, `[LT-1]`), such as
  `word = line.to_lower().trim()` or `best = a.as_span()`, is a read-only view of it: never a hand-over,
  and never a silent copy, except that a part with no fields of its own (a number, a `bool`, a variant
  with no payload) is copied, as Python binds a number. A call whose result is a new value the callee
  made (a list it built, a list of views included) hands that value to the name, which owns it
  (`[LT-45]` bounds the life of a value that holds views). A word before the part says otherwise:
  `ref mut` makes a name for that very place that may change it (`boss = ref mut enemies[0]`,
  `r = ref mut first_even(ref mut xs)`), and before a `MutSpan` a call gives it says the name may write
  the span's elements (`d = ref mut tracks[dst_name].as_mut_span()`); `copy of` makes a copy of your own
  (`boss = copy of enemies[0]`); the link words make a link (`[OWN-13]`); and `ref` states the default,
  needed before a part only to view a number (`r = ref counts[0]`).
* `[BRW-13]` *(new in 0.9.15)* **`=` on a name that has a value** puts the new value in the place the
  name stands for. A name declared with `=` stands for its own place: its old value is replaced (and
  ends, if it owned something), and if the value is a view, the name now views something else
  (`rest = tail.trim()`, `cur = p`), checked by `[BRW-1]` against its new target. A `ref mut` name stands
  for the place it was made from until it is given another ref: `r = ref mut a` then `r = 20` writes
  `a`, and `r = ref mut b` points `r` at `b`, after which `r = 30` writes `b`. The line that gives it
  another ref says `ref mut` again, for every type, and a read-only `ref b` given to it is `E3023`
  ("`r` may change what it refers to, and `ref b` only reads `b`; to point `r` at `b` write
  `r = ref mut b`, or for a read-only name `rb = ref b`"). A name that holds a `MutSpan` likewise:
  `d[i] = v` writes an element through it, and `d = ref mut other.as_mut_span()` points it at another
  span. A name written **`const ref mut`** is never given another ref, as `const` on a value means it
  never changes: `r = const ref mut a` then `r = ref mut b` is `E3023` ("`r` is `const ref mut a`, so it
  always refers to `a`; to refer to `b` use a new name, `rb = ref mut b`, or make `r` with plain
  `ref mut`: `r = ref mut a`").
* `[BRW-14]` *(new in 0.9.15)* **Names that keep their place.** A parameter, a loop name, a pattern name
  and a name opened with `exists as` (`[GRM-44]`) or named by a kind test (`[DSP-4]`) stand for what the
  caller, the loop, the opening or the test gave, and keep it for their whole scope: `=` on one is
  `E3023` ("`line` is the text the caller passed and stays that text for the whole call; for a view of
  your own write `rest = line.trim()`"; "`n` is the number the caller passed; for a number of your own
  write `count = n` and change `count`"), and so is any other change through one that is plain
  (`[BRW-15]`). The one exception is a name opened `exists as copy of v`, a copy of your own, which may be
  changed (`if scores.get(name) exists as copy of v:` then `v += 1`; the map is untouched). A `ref mut`
  one writes the caller's place, and may be given another ref, as any `ref mut` name may: a
  `ref mut dst: MutSpan[float]` parameter given another span points the caller's `d` at it, and
  `ref mut c: Counter` given another ref points `c`, for the rest of the call, at that place.
  `const ref mut dst: MutSpan[float]` writes the caller's elements and is never given another ref, so
  the caller's `d` is the same span after the call.
* `[BRW-15]` *(new in 0.9.15)* **A plain name never changes anything.** A plain parameter, a plain loop,
  pattern or opened name, and a plain local that holds a view never change what they name or what it
  leads to (a `Cell` field, an object behind a `shares` link, the elements of a `MutSpan`). A change
  through one is `E3023`, with the line to write: "`path.pop()` changes the list in `n.state`, but
  `path` only reads it; to change that list through `path`, write `State.Walking(ref mut path):`";
  "`tick` sets `s.frame`, but `s` only reads the caller's clock; write `fn tick(ref mut s: Clock):` and
  call `tick(ref mut clock)`"; "`dst[i] += src[i]` writes an element of the caller's span, but `dst`
  only reads it; write `fn blend(ref mut dst: MutSpan[float], src: Span[float]):` and call it
  `blend(ref mut d, s)`"; "`d[i] = 0.0` writes an element of `tracks[dst_name]`, but `d` only reads it;
  write `d = ref mut tracks[dst_name].as_mut_span()`". A method that may change its object, called
  through a name that only reads it, is `E3023` too (`[EXC-15]`). The types made to be shared between
  threads are used through any name, a plain one included, by their own operations: a `Mutex`,
  `RwLock`, `Atomic` or channel end, and the `guarded by` fields of an object under their lock
  (`with g = m.lock():`; `with log.lock: log.lines.push(...)`; `total.fetch_add(n)`; `tx.send(v)`;
  `rx.recv()`). Each orders its own change (the lock's block, an atomic instruction), and handed to a
  call it takes no word (`[FN-2a]`, Part XI).
* `[BRW-1]` *(changed in 0.9.15)* **One name changes, or any number read.** At every point of the
  program a place is reached through any number of read-only views, or through exactly one name that
  may change it, as `[BRW-17]` to `[BRW-20]` and the two rules below say; every one of them is checked
  when compiling, and nothing is checked while the program runs.
  * **A `ref mut` name is alone.** While a `ref mut` name (a parameter, a local, a loop, pattern or
    opened name, a kept `ref mut` field; one that holds a `MutSpan` included) is still used, its place is
    read or changed only through it: another name that may name the same place, a link of the same type
    included, is refused, `E3022` to change it and `E3021` to read it ("`boss` is the only name for
    `enemies[0]` until line 9, and line 7 reads `enemies[0].hp`; read it through `boss`: `boss.hp`"),
    unless the compiler proves the two differ (`if t is not a:`), or proves them the same whole variable
    given to one call (`[BRW-11]`). For a `ref mut` parameter and a place its function reaches by its
    own name (a module variable), the check is made at each call, by what the body does (`[BRW-21]`),
    so the function itself compiles. Two names of elements of one list, at least one `ref mut`, whose
    indexes are not proved different are refused as two such arguments are (`[BRW-5]`). A reshape
    through another name while a `ref mut` name that is not a position is still used is `E3022`, at the
    reshape; a position is ruled by `[BRW-16]`. A call through a `ref mut` name of a stored function
    value uses the name only to copy the function's two words out, before the arguments are given
    (`[CLO-10]`), so the call's arguments may reach the place it names; a later use of the name is
    checked against what the call may have done. `this` is not such a name (`[EXC-15]`).
  * **A read-only name promises that what it reads does not change while it is used.** While a
    read-only view (a plain name for a part, a `ref` name, a plain parameter, a `Span` parameter, a
    plain name holding a `MutSpan`, a plain loop, pattern or opened name, a view a call gave) is still
    used, what it points into is not changed through any other name (`E3021`, at the change) unless the
    compiler proves the two differ; a change that may reshape it ends it instead (`E3060`, at its later
    use). The refusal gives the ways out that compile in that program: "`enemies[0].hp = 1` (line 7)
    changes the enemy `boss` reads (`boss = enemies[0]`, line 5), and `boss` is used on line 9; change
    it after `boss`'s last use, or write `boss = ref mut enemies[0]` and change it through `boss`:
    `boss.hp = 1`". A copy of your own (`boss = copy of enemies[0]`) is restricted by nothing. A `shares`
    owner is an owner, not a view: it sees the changes the value's other owners make (`[RC-1]`).
* `[BRW-2]` *(changed in 0.9.15)* **Liveness.** A view is in use from where it is made to the last use
  of anything made from it: the view itself, a view made from it, a value returned from a call it was
  given to. Scope does not matter: `n = v.len(); v.push(n)` is legal.
* `[BRW-3]` *(changed in 0.9.9)* *(changed in 0.9.15)* **Reading before the call.** For a method call
  whose object is a place, or an argument given to a `ref mut` parameter, the place may be read while
  the call's arguments are evaluated, and values computed from it may be passed (`v.push(v.len())`); a
  view of the place still in use at the call, because it is itself an argument or is held by one,
  overlaps the changeable argument and is `E3021` (`[BRW-11]`). For
  `fn f(ref mut v: Array[String], s: StrView)` the call `f(ref mut v, v[0])` is therefore refused; for
  `fn f(ref mut v: Array[int], x: int)` the call `f(ref mut v, v[0])` is accepted, because `v[0]` is
  copied in before the call.
* `[BRW-11]` *(new in 0.9.9)* *(changed in 0.9.15)* **Arguments do not overlap a changeable one.** Two
  arguments of one call (the object a method is called on counts as one), of which at least one may be
  changed (a `ref mut` argument, a `MutSpan` one included; the object of a method that may change it;
  or a function value, which counts as changing every place the functions it may run change and every
  real variable they capture, `[CLO-2]`), may not name one place, and neither may reach into what the
  other owns, unless the compiler proves they differ: different constants, `j` from `i + 1..`, inside
  `if i != j:`, or after `assert_disjoint(a, b)` (with `from std.mem import assert_disjoint`,
  `[DSJ-5]`). Any other possible overlap is refused when compiling: `E3022` where both may be changed,
  `E3021` otherwise. This holds for spans of numbers too: two spans of one list given to one call, one
  of them changeable, are proved apart (`assert_disjoint`, two compares) or refused ("`blend` changes
  the elements of `d` (`tracks[dst_name]`, line 10) while `s` shows `tracks[src_name]` (line 11), and
  the two are one list when `dst_name` equals `src_name`; prove them apart first,
  `assert_disjoint(d, s)` before line 12, with `from std.mem import assert_disjoint` at the top, or
  blend from a copy: instead of lines 10 to 12 write `old = copy of tracks[src_name]` and
  `blend(ref mut tracks[dst_name].as_mut_span(), old.as_span())`"). A function value given to a call
  that changes the call's object is refused likewise ("`LOG.retain(keep)` changes `LOG` while it walks
  it, and `keep` adds to `LOG` too (its line 3); drop the lines first and push the notes after:
  `before = LOG.len()`, `LOG.retain(fn(s) => s != "")`, `dropped = before - LOG.len()`, then
  `for k in 0..dropped: LOG.push(String("dropped an empty line"))`"). Two exceptions:
  1. **The same whole variable under two names.** Where the compiler proves when compiling that two
     `ref mut` arguments, or a `ref mut` argument and a variable the callee uses by its own name (a
     module variable), are one whole variable, the call is allowed and compiled against a copy of the
     function in which the two names are one. `fn add_both(ref mut a: int, ref mut b: int):` whose
     body is `a += 1`, `b += 1`, `println(a)`, called `add_both(ref mut x, ref mut x)`, prints 2;
     `fn rename(ref mut s: Array[String]):` whose body is `LOG.clear()` then
     `s = [String("renamed")]`, called `rename(ref mut LOG)`, leaves `LOG` holding one string (it frees
     nothing: `clear()` empties the list, `s = ...` fills it again). Other calls use the normal copy,
     where a `ref mut` parameter is the only way in. Parts whose positions are not known when compiling
     (`ref mut log[i]` beside `ref mut log[j]`, or beside `log`) stay refused (`E3022`).
  2. **A thing whose changes go only through its lock.** A `thread_shared` object whose every field
     that may change is `guarded by` its lock (or is an `Atomic`, `Mutex` or `RwLock`) may reach one
     call through any number of names, a plain argument and a function value that holds it included:
     `dump(log, fn() thread_shared log: log.push("seen"))`, whose `dump` reads the lines inside
     `with log.lock:` and calls `note()` after the block, is allowed; the lock keeps the two ways in
     apart. A call inside the lock that may take it again is `E7018` (§XI.2).
  For two spans whose lists may be one: E3021 ("`refill` may replace the lists of `w1.lists` through `dst` while `src` shows numbers inside one of them, and `w1` and `w2` may be the same `Wrapper`; give `src` a list of its own: `first = copy of w2.lists[0]`, then `refill(ref mut w1.lists.as_mut_span(), first.as_span())`").
* `[BRW-21]` *(new in 0.9.15)* **The call check: a view argument and what the callee does.** A view
  whose last use is an argument of a call (of a plain, `Span` or `ref mut` parameter) does not live
  across the call: inside it, the parameter makes the promise of `[BRW-1]`, and the compiler holds the
  callee to it. It works out from each function's body which of its view parameters, `this` included,
  it uses after a line that may end or move what they view (a reshape; the end of an object,
  `[RC-5]`), through its own lines and the same fact of the functions it calls (a call through a
  function value, or through an interface, `dyn` or virtual method, by every function the program can
  put there, `[EFF-1]`), with the body's precise facts (a change inside `if a is not b:` is not one for
  `a`). It keeps that fact with the function (in the package's interface file across a package's edge,
  `[EFF-4]`) and checks it at each call: an argument that points into what the callee may end or move
  before its last use of that parameter is `E3031`. Writing a number or a `bool` ends and moves nothing,
  so a method that writes `this.health` through a link that may lead to its own object (a turret aimed
  at itself) is allowed; leaving a lock's `with` block ends nothing either. With `fn note(s: StrView):`
  whose body is `LOG.clear()` then `println(s)`, `note(LOG[0])` is refused ("`note` clears `LOG` (its
  line 3) and then uses `s` (its line 4), and `LOG[0]` is part of `LOG`; give it a copy:
  `note(copy of LOG[0])`, or in `note` use `s` before `LOG.clear()`"); `note("bye")` is allowed, and so
  is `note(LOG[0])` for a `note` that prints before it clears. The object a method is called on counts
  the same way: with `fn hit():` whose body is `ENEMIES.clear()` then `this.hp -= 1`, `ENEMIES[0].hit()`
  is refused ("`hit` clears `ENEMIES` (its line 3) and then changes `this.hp` (its line 4), and
  `ENEMIES[0]` is part of `ENEMIES`; in `hit`, change `this.hp` before `ENEMIES.clear()`"), and a `hit`
  that uses `this` only before the clear is allowed. A `ref mut` parameter counts the same way: with
  `fn rename(ref mut s: String):` whose body is `LOG.clear()` then `s = String("renamed")`,
  `rename(ref mut LOG[0])` is refused ("`rename` clears `LOG` (its line 2) and then writes `s` (its line
  3), and `LOG[0]` is part of `LOG`, so `s` would point at freed memory; rename a copy of your own:
  write `mine = copy of LOG[0]` and `rename(ref mut mine)` in place of this line, or in `rename` write
  `s` before `LOG.clear()`"). Since a `ref mut` parameter is compiled with noalias (`[BRW-22]`), the check
  is wider for it: while it is used, the callee reaches its place through no other name, so an argument
  for it that points into a place the callee reads or changes by another name while the parameter is
  used is refused too (with `fn heal(ref mut p: Player):` whose body is `println(PARTY[0].hp)` then
  `p.hp += 10`, `heal(ref mut PARTY[0])`: "`heal` reads `PARTY[0]` by its own name (its line 2) while
  `p` is still used (its line 3), and `p` is `PARTY[0]`; in `heal`, read it through `p`:
  `println(p.hp)`"), unless it is that same whole variable, proved when compiling (`[BRW-11]`,
  exception 1). Where the fault is in one function stored in a function value (it ends what its
  parameter views and then uses the parameter), the refusal stands on the line that stores it, not on
  the call through the value, which is right for every other function ("`on_visit` is called with a
  node of the tree (line 21), and this function removes its node from the parent's list (line 41) and
  then reads it (line 42); read it before removing it: move line 42 above line 41"). Only a C function's
  code is unseen; `[FFI-52]` covers it. Nothing is checked while the program runs.
* `[BRW-22]` *(new in 0.9.15)* **Noalias.** Every `ref mut` parameter, a `MutSpan` one included, is
  compiled with noalias (C's `restrict`): while it is used, nothing else the function uses reaches its
  place (`[BRW-1]`, `[BRW-21]`); a call compiled against a copy with one name (`[BRW-11]`, exception 1)
  has one name for that place. `this` is never compiled with noalias (`[EXC-15]`), and neither is a
  `sharedref mut` link (`[LT-46]`).
* `[BRW-4]` *(changed in 0.9.9)* *(changed in 0.9.15)* **Disjoint fields.** `ref mut a.x` and
  `ref mut a.y` may be in use together when `x` and `y` are different fields, through any depth of field
  path, for a struct value's fields and a class object's alike (`[EXC-19]`). What two different paths
  may both reach through links is `[BRW-17]`'s.
* `[BRW-10]` *(new in 0.9.9)* *(changed in 0.9.15)* **What a method does to its object is worked out
  from its body,** for every method, private or `pub` (`[EXC-15]`): a call `w.bump()` changes only the
  fields `bump`'s body changes, and those its callees' bodies change, so it may be called while a loop
  walks `w.names` if `bump` touches only `w.count`. `fn note(msg: String):` whose body is
  `this.log.push(msg)` changes `this.log`, so `this.note(...)` inside a loop over `this.log` is refused
  (`E3020`) and inside a loop over `this.players` is allowed. Across a package's edge the fact is
  recorded in the package's interface file (`[EFF-4]`), so a change to a `pub` method's body that
  changes what it touches is reported as a change of its interface.
* `[BRW-5]` *(changed in 0.9.9)* *(changed in 0.9.15)* **Indices are not disjoint unless proved.**
  `ref mut a[i]` and `ref mut a[j]`, as two names or as two arguments of one call, are `E3022` unless
  the compiler proves `i` and `j` differ: both constants and different, `j` from `i + 1..`, or the line
  inside `if i != j:`. Nothing is compared while the program runs. With
  `fn collide(ref mut a: Body, ref mut b: Body):`, `collide(ref mut bodies[i], ref mut bodies[j])` in a
  function given `i` and `j` is refused ("`collide` changes `a` and `b` as two different bodies, and
  `bodies[i]` and `bodies[j]` are the same body when `i == j`; write `if i != j:` above this line and
  indent it under it"), and compiles under `if i != j:` or in `for j in i + 1..n:`. The same position
  twice is refused with "both are `bodies[i]`". `get_pair_mut(i, j)`, `split_at`, `chunks_mut`,
  `for ref mut x in xs:`, `swap(i, j)` and `SoA` columns are the library's ways to hold two changeable
  views of one container. `get_pair_mut(i, j) -> Option[(ref mut T, ref mut T)]`, on an `Array` or a
  `MutSpan`, views the container once, checks that both indices are in range and differ, and only then
  makes the two views; otherwise it is `None` and the container is unchanged. The pair points into the
  list it was called on (`[LT-1]` rule 1). Distinct elements of a zero-sized type need not have
  distinct addresses, and nothing about addresses follows from the indices (ODR-068).
* `[BRW-6]` *(changed in 0.9.15)* **Lending a `ref mut` name.** From a `ref mut` name `r`, a read-only
  view of `r.f` keeps `r` from changing `r.f` while it is used, and `ref mut r.f` lends that part, so `r`
  does not reach it meanwhile. Passing a `ref mut` name to a `ref mut` parameter (`bump(ref mut r)`)
  lends it for the call; it does not hand it over.
* `[BRW-7]` *(changed in 0.9.15)* Using a place that is uninitialised on some path, one never given a value there, is `E3050`;
  using one a hand-over may have reached is `[OWN-3]`'s `E3040` (ODR-114).
* `[BRW-8]` *(changed in 0.9.9)* *(changed in 0.9.15)* A plain or `ref mut` parameter is passed by
  address: the callee reads, or changes, the caller's place (`[FN-1]`, ODR-024). Two kinds are passed as
  a copy instead:
  * a view (a `ref`, `Span`, `MutSpan`, `StrView` or `@view` struct), which is passed as itself and keeps
    the regions it carries;
  * a plain parameter of a `Copy` type that holds no `Cell` or `UnsafeCell`, which goes in registers when
    it is no larger than two pointers, except the object of a method whose result is a view (`[LT-1]`
    rule 1).
  An `owns` parameter receives the value. A view into the own storage of a parameter passed as a copy
  ends with the call: returning it is `E3063`. The choice depends only on the declared first line, never
  on the body; it is an ABI decision and cannot be observed. An instance of a generic function, method
  or type has the generic's declared first line: a parameter its result may point into (the object, or
  a source parameter of `[LT-1]`'s second kind) is passed as the declaration passes it, whatever the
  type arguments, so `Holder[StrView]`'s object is passed by address as `Holder[T]`'s is (ODR-048).
* `[BRW-9]` *(changed in 0.9.15)* A view in Safe code is never null, dangling or unaligned.

## VII.5 Views and changes

* `[BRW-16]` *(new in 0.9.15)* **Positions.** A name for a list element made by indexing
  (`boss = enemies[0]`, `boss = ref mut enemies[0]`, `p = ps.get(0).unwrap()`), of a list reached
  through locals that own it, parameters, `this` and one-owner fields, is a position: the element is
  found again at each use. The list may grow while the name is used (its buffer is read again after the
  add); anything that may remove, insert, reorder, clear or replace the list ends the name, and a later
  use is `E3060` ("`enemies.remove(0)` (line 8) may put another enemy in the place `boss` names (line
  6), and `boss` is used on line 9; use `boss` before line 8, or name the enemy again after it:
  `boss = ref mut enemies[0]`"), a call that reads elements and removes included. Growing the list is
  not a use of the element. A list reached through a link (a local or field holding a `shares`,
  `shares mut` or `sharedref mut` link) gives addresses, `[BRW-17]`.
* `[BRW-17]` *(new in 0.9.15)* **Addresses.** Every other view (a kept view; a view reached through a
  link; a view into a value's own contents: a `String`'s text, an `Array` field's elements, an enum's
  payload; every view a call gives, `[LT-1]`) ends at any change that may reshape what it points into,
  made through any name that may reach it, and a later use is `E3060`. Two paths may reach the same
  object when they lead to values of one type and at least one goes through a link or a kept view,
  unless the compiler knows they differ (different types, a fresh object, inside `if a is not b:`).
  What may reshape is worked out from the code (`[EFF-1]`): the function itself, the functions it
  calls, every function the program can put in a function value or behind a `dyn` method it calls, and,
  at a foreign call, the callbacks C keeps (`[FFI-21]`). So a call through a function value whose stored
  function replaces the text of an object it owns (`rename = fn() owns n: n.name = String("b")`) ends a
  view of that text reached through a link (`s = p.name.trim()` under `if w exists as p:`): `s` is not
  used after `rename()` ("`rename()` (line 9) replaces the node's name (`n.name = String("b")`, line
  6), and `s` views that name's text (line 8) and is used on line 10; use `s` before line 9, or keep your
  own text: `s = String(p.name.trim())`"). A view parameter (a plain, `Span` or `ref mut` one), or
  `this`, and what its function may end or move may be one place too; that is checked at each call
  (`[BRW-21]`). Adding to a list moves only what it stores inline: a view of a class object (a link, a
  kept view, or an address reached through a link), which lives in its own block, is not ended by a
  `push` or an `extend` of the list that owns it (with `players: Array[Player]` and
  `w = Watcher(target=sharedref mut players[0])`, `w.target.hp -= 1` after `players.push(Player())` is
  allowed); one to a value stored inline in the list (a struct element) is. Removing, clearing or
  replacing still ends every view of what it may end.
  For keys viewed across a change of their map: E3060 ("`m.insert(String("z"), 1)` (line 8) changes `m`, whose keys `ks` (line 7) still views on line 9; use `ks` before line 8, or keep your own strings: write `ks = sorted(m.keys().copy_each())` in place of line 7").
* `[BRW-18]` *(new in 0.9.15)* **Loops, generators included.** While a `for` loop walks a collection (a
  list, a map, a set or a string), or a generator that walks it (a `gen fn` paused in the middle of it,
  `[LT-1]`) will still be used, nothing changes the collection, whatever the change: nothing is added or
  removed, no element is written or replaced and no field of one changed (`xs[0] = 5`,
  `enemies[0].hp -= 1`), no value under a key replaced (`counts[k] *= 2`, `names[k] = String("bob")`),
  through the collection's own name or through any call that may change it (`[EFF-1]`: a call through
  a function value, a library call given a function that changes it, a `spawn` whose thread changes it
  other than under a lock the loop holds, a foreign call while a callback C keeps changes it, and the
  end of an object whose `drop` changes it, `[DRP-8]`). It is `E3020`, at the change:
  "`enemies.push(...)` adds to `enemies`, which this loop walks (line 3); collect the new enemies in a
  list of your own: write `spawned: Array[Enemy] = []` before the loop,
  `spawned.push(Enemy(hp=5, splits=false))` in place of this line, and `enemies.extend(spawned)` after
  the loop"; "`counts.insert(...)` (line 4) adds to `counts`, whose keys this loop walks (line 3);
  collect the new keys in a list of your own: write `upper: Array[String] = []` before the loop,
  `upper.push(k.to_upper())` in place of this line, and `for u in owns upper: counts.insert(u, 0)` after
  the loop"; "`b.click()` (line 17) adds to `LOG` (through `click`, which runs the handler stored on
  line 9, `log_click`, which pushes to `LOG`) while this loop walks `LOG` (line 16); click after the
  loop: move line 17, `b.click()`, below the loop"; "`d.log.push("open")` (line 3) changes `d.log`,
  which `g` walks (`g = d.replay()`, line 1), and `g` is used again on line 4; push after `g`'s last use:
  move `d.log.push("open")` below line 4". **The loop's own handle may change the item it is on**:
  `for ref mut x in xs: x = 5` gives `[5, 5, 5]`; `for k, ref mut v in counts: v *= 2`;
  `for k, ref mut v in names: v = String("bob")`; and the message gives that line: "`xs[0] = 5` (line
  3) writes an element of `xs`, which this loop walks (line 2); to change the element the loop is on,
  write `for ref mut x in xs:` in place of line 2 and `x = 5` in place of line 3"; "`counts[k] *= 2`
  (line 4) changes `counts`, whose keys this loop walks (line 3); change each value through the loop's
  own handle: write `for k, ref mut v in counts:` in place of line 3 and `v *= 2` in place of line 4".
  Anything else goes after the loop or after the generator's last use (`retain`, `extend`, `Commands`).
  A counting loop `for i in 0..xs.len():`, whose range is fixed when it starts, does not walk `xs`: it
  may write its elements and add to it, but not remove from it (`E3020`: "`enemies.remove(i)` (line 4)
  shortens `enemies`, which `for i in 0..enemies.len():` (line 2) counts over with an end fixed when it
  started; remove after the loop: instead of `if enemies[i].hp <= 0:` (line 3) and line 4, write
  `enemies.retain(fn(e) => e.hp > 0)` after the loop (and remove the `for` line if nothing is left in
  it)"). A `while` loop that tests its position against the list's current length at every step may
  add and remove: it is not a walk, and each index is checked. A call through a function value that may
  take a lock this thread holds, inside a walk under that lock, is `E7018`, not `E3020` (§XI.2):
  the message whose fix compiles is the one shown (`[DIA-28]`).
  For a call through a stored function that may change the walked list: E3020 ("`h(this)` (line 8) may clear `this.handlers` (through `close_all`, stored on line 20) while this loop walks `this.handlers` (line 7); give each handler the Ui to change, `handlers: Array[fn(ref mut Ui)]`, and walk by position instead of the `for` line: `this.current = 0`, `while this.current < len(this.handlers):`, `this.handlers[this.current](ref mut this)`, `this.current += 1`").
* `[BRW-19]` *(new in 0.9.15)* **Temporaries.** A temporary lives until the last use of every view made
  from it, lists of views included (`word = line.to_lower().trim()` keeps the lowered string until
  `word`'s last use, `[EXP-4]`). Returning such a view, or storing it where it would outlive the
  temporary, is `E3063`: "`word` is a view of the lowered copy that `line.to_lower()` made on line 2,
  which ends when `clean` returns; return `String(...)` to hand back your own copy:
  `return String(word)`, with the result type `String`". A method that may change its object, called on
  a temporary that ends with the line, is refused when the call's only effect is that change (its body
  changes nothing else, through its arguments or by any other name, and nothing it gives back is used),
  since the change would be lost: `E2140`, "`heal` changes the enemy that `enemy_at(es, 0)` returns, a
  copy that ends on this line, so the change would be lost; to heal the enemy in `es` write
  `es[0].heal()`, or let `enemy_at` give back the enemy itself:
  `fn enemy_at(ref mut xs: Array[Enemy], i: int) -> ref mut Enemy:` with `return ref mut xs[i]`,
  called `enemy_at(ref mut es, 0).heal()`". A call whose result is used stays allowed
  (`line.split(" ").next()`).
* `[BRW-20]` *(new in 0.9.15)* **Guards.** A view made through a lock's guard (`g[0]`, `ref g[0]`, a
  slice of `g`) points into the guard and does not leave its `with` block (`E3063`: "this points into
  the locked list, which can change once the lock ends; return a copy, or do the work inside the `with`
  block"); the same for `RwLock`'s `read()` and `write()` and for a `guarded by` field inside
  `with x.lock:` ("this points into the read-locked list, which can change once the lock ends; return a
  copy: `return copy of g[0]`, with the result type `String`").

## VII.6 Regions and their inference

Ember has **no lifetime syntax**. Every view carries a **region**, the set of program points at which
it must be valid, and the compiler infers every region.

* `[LT-5]` *(changed in 0.9.15)* Regions of locals are inferred by non-lexical liveness (§XVIII.4). Diagnostics describe them
  in source terms: "the view of `x` on line 12 is still used on line 19".
* `[LT-1]` *(changed in 0.9.9)* *(changed in 0.9.15)* **Where a call's view points.** A call's result
  that is a view, and a view a call puts in through a `ref mut` parameter (`[LT-47]`), point into the
  arguments they may come from. A **source parameter** (ODR-024) is:
  * a parameter whose type is a view, in any mode; or
  * a plain or `ref mut` parameter whose type is not `Copy` even when each of its type parameters is
    taken to be `Copy`: an `Array[T]`, `String`, `Map`, an `owns T` or an arena, a struct or enum without
    `@derive(Copy)`, or a tuple or fixed array holding one of these.
  A source parameter of the second kind is the caller's place (`[FN-1]`, `[BRW-8]`): a result that points
  into it may point anywhere in it, into its own storage or into storage it owns, and the call's view is
  on the argument (`[BCK-1]`), read-only for a plain parameter and changeable for a `ref mut` one. A
  `Copy` parameter that is not a view is not a source parameter (so neither is a plain `x: T`, nor a
  function-type parameter, `[CLO-3]`), and nor is an `owns` parameter that is not a view. Which
  parameters are sources depends only on the declared first line. In an instance of a generic, a
  parameter whose type there is a view is a source of that instance too, of the first kind:
  `Tree[StrView].leaf(v: T)` may return a view of what `v` views, and its callers, which call that
  instance, know it (ODR-048). Where rule 1 below ties the result to `this`, such a parameter is a
  source alongside it when its declared type names a type parameter the declared result names too: the
  result's type says it can hold one, so `Map[StrView, V].entry(k) -> MapEntry[K, V, H]` returns an
  entry holding `k`, while `get(q) -> Option[ref V]` points only into the map (ODR-069).
  For a function whose result is a view:
  1. if it is a method, the result points into `this`;
  2. else, if exactly one parameter is a source parameter, the result points into it;
  3. else, the result points into **all** source parameters together (the caller keeps all of them
     viewed while the result lives).
  Rule 3 always type-checks; it may view more than the function needs. Pointing into an argument means
  pointing into it and into everything it owns (**into**, §VII.1), so a reshape of anything it owns ends
  the view and any other change of it while the view is used is refused (`[BRW-1]`). A returned view
  that points into any other parameter is refused: `E3063` when that parameter is `owns` or is passed as
  a copy (`[BRW-8]`), whose storage ends with the call, and `E3062` otherwise. A call whose result type
  is `ref mut T` or `ref T` is a place (`first(ref mut xs) += 1`; `counts.entry(k).or_default() += 1`),
  and a name bound to it follows `[BRW-12]` (`r = ref mut first_even(ref mut xs)`). A generator method's
  result is such a view, and it walks what it views: after `g = d.replay()`, the list it walks
  (`d.log`) does not change while `g` will still be used (`[BRW-18]`, `E3020`), however `replay` walks
  it, and any other change of `d` while `g` will still be used (`d.angle = 90.0`) is a change of what a
  view a call gave points into (`[BRW-1]`, `E3021`), unless `-> Generator[int] from log` narrows the
  view to `d.log`.
* `[LT-1a]` *(changed in 0.9.9)* *(changed in 0.9.10)* *(changed in 0.9.15)* **`from`.** `-> T from
  name`, written after the result type on the function's first line, replaces the rule the compiler
  would apply: the result points into exactly the named parameter, or into the named field of `this`
  written bare (`-> StrView from src`). The body may then give only a view of what `from` names, and
  any other is `E3062` ("`next_word` returns a view of `this.buf`, but its first line says `from src`;
  return a view of `src`, or write `-> StrView from buf`"). A named parameter is a source parameter
  (`[LT-1]`) or a `ref mut` parameter; naming a plain `Copy` parameter, or an `owns` parameter that is
  not a view, is `E2031`. A bare field name names a field of `this` whose type is a view, whatever the
  method does with its object (`owns this`, or passed as a copy, `[BRW-8]`): the result points into
  what the field points into, and the call's view of the object ends with the call. A result read
  through a `ref mut` field points into `this` as well, so it cannot name the field alone (`E3062`):
  an iterator gives no two changeable items. The field may instead be one whose type is a type
  parameter, with a result whose type names one (`Option[I.Item]`); each instance is checked, as a
  source is (`[LT-1]`, ODR-048): where the instance's field and result are views the result points into
  what the field points into, and where they are not it points into nothing. Naming a field when there
  is no `this`, or a field of neither kind, is `E2031`; a name that is both a parameter and a field of
  `this` is `E2118` ("`src` is both a parameter of `next_word` and a field of `Parser`, so `from src`
  could mean either; rename the parameter: for a view of the field write
  `fn next_word(line: StrView) -> StrView from src:`, for a view of the parameter
  `fn next_word(line: StrView) -> StrView from line:`"). The standard library's iterators that read
  through a read-only field say so on `next`, and its iterator adapters on the iterator they wrap, so
  `for k in m:` and `m.keys()` give views that point into the map (`[CTL-1]`), which a program may keep
  across rounds and after the loop. `from` is needed only where rule 1 or 3 points into more than the
  caller can afford, or to let the result point into a `ref mut` parameter of a `Copy` type:

```ember
fn find_word(haystack: StrView, needle: StrView) -> Option[StrView] from haystack:
    i = haystack.find(needle)?
    return Some(haystack[i..i + needle.len()])
```

* `[LT-3]` String literals, `bytes` literals, `static` items and views of them have the `static`
  region, which outlives everything.
* `[LT-4]` *(changed in 0.9.9)* *(changed in 0.9.15)* A value made in a memory manager's instance
  (`owns Mesh(points=[0.0, 1.0]) using current`, `[ALC-1]`), and every value that holds one, lives no
  longer than that instance, wherever it is kept: keeping or returning it where it would outlive the
  instance is `E3063` ("`boss` holds a mesh made in `current` (line 12), and `current` ends when
  `load_level` returns, so `boss` cannot be returned; make it in an arena the caller keeps:
  `fn load_level(ref mut arena: LevelArena) -> Enemy:`, with `mesh=owns Mesh(points=[0.0, 1.0]) using
  arena`, and in the caller `current = LevelArena()` and `boss = load_level(ref mut current)`"). A value
  whose manager may have several instances keeps its instance's address, one word, so that its growth
  and its end go back to the instance it came from; that address is a kept `sharedref mut` link to the
  instance (`[LT-46]`): the program may use the instance while values made in it live, and each value's
  growth and end change the instance through it. A manager with one instance (`Global`, `System`) costs
  no word. Arena allocations point into the arena (`[ARN-1]`).
* `[LT-44]` *(new in 0.9.9)* *(changed in 0.9.15)* A plain or `ref mut` `Arena`, `FixedArena` or
  `ScopedArena` parameter is a source parameter because it is not `Copy` (`[LT-1]`): a function whose
  only source parameter is one arena returns views that point into it, with no word. `from arena` is
  needed only when the function has other source parameters too.
* `[LT-6]` *(changed in 0.9.9)* *(changed in 0.9.15)* Named lifetimes are not part of Ember and will
  not be added. Where a relationship between regions cannot be inferred, the diagnostic names the
  restructuring that expresses it: an owned result, an index instead of a view, a view struct, or
  `from`.
* `[LT-7]` *(changed in 0.9.9)* *(changed in 0.9.15)* **Function types.** Each call through a value or parameter of function
  type `fn(P1, …, Pn) -> R` gets fresh regions for its source parameters (`[LT-1]`). `R` may point into
  them only as `[LT-1]` would allow for a function declared with that first line, and then only for the
  time the caller keeps the arguments. So a callee may lend a function value views of its own locals,
  and a function value may return a view of an argument to its caller:

```ember
fn with_local(f: fn(Span[int]) -> int) -> int:
    tmp = [41]
    return f(tmp)

fn first_of(a: Span[int], b: Span[int], pick: fn(Span[int], Span[int]) -> Span[int]) -> int:
    return pick(a, b)[0]

fn main():
    println(with_local(fn(s) => s[0] + 1))
    xs = [1, 2]
    ys = [3]
    println(first_of(xs, ys, fn(a, b) => a))
```

* `[LT-45]` *(new in 0.9.15)* **Kept views.** A struct that holds a view (`ref T`, `ref mut T`,
  `sharedref mut T`, `Span[T]`, `MutSpan[T]`, `StrView`, or a struct that holds one) lives no longer than
  what its view fields point into (`[TYP-34]`); `@view` may say so and is checked (`@view` on a type
  that holds no view is `E2030`: "`@view` on `Point`, which holds no view; remove `@view`"). A value that
  holds views (a local list of views, a struct made by a call) lives no longer than what they point
  into, whoever owns it; keeping or returning it where it would outlive that is `E3063` ("`ws` holds
  views of `players`, so it may not outlive it (returned here); let the watchers keep their players
  alive: write `target: shares mut Player` in `Watcher`,
  `players = [shares mut Player(), shares mut Player()]` in place of line 3 and
  `Watcher(target=shares mut players[r])` in place of `Watcher(target=sharedref mut players[r])` on line
  6; or keep positions: write `target: int` in `Watcher`, `Watcher(target=r)` on line 6, and
  `return (players, ws)` with the result type `(Array[Player], Array[Watcher])`"). A class object is
  never a view: a view field of a class says what it points into, `from <let field>` (a `let` field of
  the same object whose contents live in their own heap block: a `String`, an `Array` or an `owns T`)
  or `from static` (text written in the program; on a struct's field too, which then does not make the
  struct a view). A view field of a class without such a `from`, or with one that names what a view
  cannot point into for the object's life (a field that is not `let`, a fixed array, a struct stored
  inline), is `E2118`: "a `Player` keeps `name` after the text it came from may be gone; to keep its own
  copy write `name: String` and `this.name = String(name)`"; "`words` points into `text`, so `text` must
  never change after `init`; write `let text: String`"; "`pair` is stored inside the object, so moving
  the object moves it; keep it in a block of its own: write the field as `let pair: owns Pair`". A
  `from static` field given a view of anything else is `E3063` ("`kind` holds only text written in the
  program, and `name` comes from the caller; write `this.kind = String(name)` with `kind: String`").
  For a view into a field stored inside the value: E2118 ("`buf` is stored inside the value, so moving the value moves it; keep the text in a `String`, which has a block of its own: write the field as `let buf: String`").
* `[LT-46]` *(new in 0.9.15)* **The three kinds of kept view**, by their word:
  * `ref T` reads; any number of names may view one place.
  * `ref mut T` may change, and it is alone: while its holder is still used, the place is not read or
    changed through any other name (`[BRW-1]`); a second holder is `E3022` ("`p` is lent to
    `cam.target`, a `ref mut` field, until its last use on line 17, and line 16 lends it to `hud.target`
    too; if both may change the player, write `target: sharedref mut Player` in `Camera` and in `Hud`,
    and `sharedref mut p` in place of `ref mut p` on lines 15 and 16").
  * `sharedref mut T` may change, and several holders, and the owner, may name one object. On one thread
    any of them may change it at any time; every view into it ends at a reshape made through any of
    them, and a list in it may not change while a loop walks it, whichever holder reaches it
    (`[BRW-17]`, `[BRW-18]`); on two threads it is changed only under a lock, through `Atomic`, or by a
    reduction from scoped tasks (§XI.3, `[THR-13]`). It is written the same way before a value
    (`Watcher(target=sharedref mut players[r])`), it costs nothing while the program runs, and its
    holders get no noalias (`[BRW-22]`). `sharedref` is always followed by `mut` (`E0114`: "a link that
    only reads may be held by any number of names: write `target: ref Player`; `sharedref` is for a link
    that may change its target: `target: sharedref mut Player`").
* `[LT-47]` *(new in 0.9.15)* **A view put in through `ref mut`.** After a call, a caller's place given
  to a `ref mut` parameter whose value is a view may point into what it pointed into before, or into any
  of the call's other arguments (`[LT-1]`). Inside the callee only such a view may be put there: another
  view argument or a part of one, or text written in the program; a view of the callee's own locals or of
  a temporary is `E3063` ("`best` is the caller's view and lives on after `pick_longer` returns, but
  `mine` ends with `pick_longer`; put a view of what the caller gave there (`best = other`), or return a
  list of your own: `-> Array[int]`"). A use of such a view after one of the call's arguments changed is
  `E3060`.
  Its message: E3060 ("`bag.items.push(k)` (line 15) may move the list that `h.view` points into (`pick` may have put a view of `bag` there on line 13), and `h.view` is used on line 16; use `h.view` before line 15").

### Views with several regions

A struct, enum or tuple holding views (a **view type**, `[TYP-34]`) may hold views of independent
sources. The compiler keeps one region per view field, a **region vector**, and never forces them to be
equal.

```ember
struct Bodies:
    positions: MutSpan[f32]
    velocities: Span[f32]

fn integrate(ref mut b: Bodies, dt: f32):
    for i in 0..b.positions.len():
        b.positions[i] += b.velocities[i] * dt

fn main():
    pos: Array[f32] = [0.0, 1.0]
    vel: Array[f32] = [1.0, 1.0]
    b = Bodies(positions=ref mut pos, velocities=vel)
    integrate(ref mut b, 0.5)
    println(pos)
```

* `[LT-14]` *(changed in 0.9.15)* A view type carries a compiler-internal region vector with one slot per view field (fields
  proven to share a source may share a slot). It is never written in source (`[LT-6]`).
* `[LT-16]` Constructing a view value keeps each field's region; it never replaces them with their
  intersection.
* `[LT-17]` **Validity is conjunctive.** A view value is usable only while every region a used field
  needs is valid. A field whose source has ended cannot be used even if other fields are still valid.
* `[LT-18]` *(changed in 0.9.15)* Every view field is an ordinary view under `[BRW-1]` to `[BRW-22]`;
  multiple regions add no new aliasing rule and weaken none.
* `[LT-20]` Moving, copying, destructuring, passing and returning a view preserves each field's region.
  Projecting one field carries only that field's region.
* `[LT-21]` Assigning a new view into a field recomputes that field's region; the old source is
  released on that path and the new one constrained. Merges of control flow keep every region that may
  reach a field.
* `[LT-22]` *(changed in 0.9.15)* A function returning a view type gets, from its body, a summary of
  which result field points into which parameter (and which parameter fields each call uses,
  `[LT-35]`). Callers use the summary; source never states it. Where the summary would need more than
  the first line allows, the result points into every source parameter (`[LT-1]` rule 3), or into what
  `from` names.
* `[LT-23]` *(changed in 0.9.15)* With `from` on a function returning a multi-region view, every field
  of the result points into what `from` names; a field that points into anything else is `E3062`.
* `[LT-24]` A region slot ends at the last use of its own field, not of the whole value.
* `[LT-25]` *(changed in 0.9.15)* A view field may not point into another field of the value that contains it (shape B5).
* `[LT-26]` *(changed in 0.9.15)* A view extends no lifetime: constructing, copying or storing it
  counts nothing. A view of an object reached through a link does not keep the object alive either; it
  is checked when compiling not to outlive it (`[RC-5]`).
* `[LT-27]` *(changed in 0.9.15)* Different regions never prove that two views do not overlap in
  memory; noalias comes only from a `ref mut` parameter (`[BRW-22]`), `[DSJ-*]` and `[SIMD-3]`.
* `[LT-30]` Regions exist only at compile time: two values of one view type with different regions have
  one layout, one ABI, one symbol and one type identity, so region inference causes no code
  duplication.
* `[LT-34]` *(changed in 0.9.15)* A view type whose fields all point into one source behaves exactly as a single-region view.
* `[LT-35]` *(changed in 0.9.15)* For each function that can receive a view value, the compiler knows
  which fields the body may read, write, hand over, return or publish. A call requires only the regions
  of those fields. A call through a function value, or through an interface, `dyn` or virtual method,
  counts every function the program can put there (`[EFF-1]`); where the target is a foreign function,
  every field is assumed used and kept.
* `[LT-36]` *(changed in 0.9.15)* Treating the view as a whole (passing it where every field may be used, comparing, hashing
  or formatting it, copying or moving it) requires every region to be valid.
* `[LT-38]` Moving one field out moves only that field's constraint; the remaining fields keep theirs.
* `[LT-39]` *(changed in 0.9.10)* *(changed in 0.9.15)* An operation that selects a field by a run-time value (reflection, a
  field descriptor) requires every region it could select; a region it needs that has ended is a
  conflict, `E3021`.
* `[LT-42]` *(changed in 0.9.15)* A function value that uses a real variable holding a view, or
  captures a view with `ref`, records the regions of the fields it uses and may not outlive them
  (`E3026`, `[CLO-4]`); one that takes a view into its own state with `owns` requires every region to be
  `static` (`[TYP-15]`).
* `[LT-43]` Region tracking never lets a view survive a `yield` that `[CORO-6]` forbids.

## VII.7 Views over containers: `Span` and `MutSpan`

```ember
fn normalize(ref mut xs: MutSpan[f32]):
    total = xs.iter().sum()
    for ref mut x in xs:
        x /= total

fn main():
    buf: Array[f32] = [1, 2, 3]
    normalize(ref mut buf)
    d = ref mut buf.as_mut_span()
    d[0] = d[2]
    println(buf)
```

* `[SPN-1]` *(changed in 0.9.15)* An `Array[T]`, a `[T; N]` or a `String` converts to `Span[T]` or
  `StrView` where one is expected, and to `MutSpan[T]` where one is expected (a `ref mut` parameter, a
  `ref mut` name, or a `MutSpan`-typed field made with `ref mut`, `[BRW-12]`). **The conversion makes a
  view** of the source place with the ordinary rules: the source is not changed through another name,
  handed over or ended while the view is used (`[BRW-1]`), whether the conversion is written implicitly,
  as `as_span()`/`as_mut_span()`, or produced by a call.
* `[SPN-2]` Indexing a view is bounds-checked; `get(i) -> Option[ref T]` does not panic;
  `unsafe: s.get_unchecked(i)` does not check.
* `[SPN-3]` *(changed in 0.9.15)* `Span[T]` is `Copy`; `MutSpan[T]` is not. A name or parameter that
  writes a `MutSpan`'s elements says `ref mut` (`d = ref mut a.as_mut_span()`,
  `fn blend(ref mut dst: MutSpan[float], src: Span[float]):`), and a plain one only reads them
  (`E3023`, `[BRW-15]`). A `MutSpan` is lent to a call with `ref mut` before the argument
  (`blend(ref mut d, s)`), which before a span names its elements, so it may stand before a span made in
  the call (`integrate(ref mut p.as_mut_span(), v.as_span(), 0.5)`); a callee that points such a span
  elsewhere changes only that span, which ends with the line. Where a `Span[T]` is expected, a
  `MutSpan[T]` converts to one: a read-only view, so the `MutSpan` is not written while the `Span` lives
  (`[TYP-5]` rule 6, ODR-116).
* `[SPN-4]` *(changed in 0.9.15)* `iter()` on a `Span` or `MutSpan` gives read-only views of its
  elements; `for ref mut x in s:` walks a `MutSpan` a `ref mut` name holds and may change each element
  it is on (`[BRW-18]`). `chunks(n)` and `chunks_mut(n)` give consecutive non-overlapping views of `n`
  elements (the last may be shorter); `n == 0` panics in every profile.
* `[SPN-5]` *(changed in 0.9.9)* *(changed in 0.9.15)* `split_at(i)` on a `Span` returns two `Span`s; on
  a `MutSpan` it lends the view and returns two disjoint `MutSpan`s. `i > len` panics. (There is no
  separate `split_at_mut`: an `Array` is split for changing as `a.as_mut_span().split_at(i)`.) A
  `MutSpan` also has `get_pair_mut(i, j)` (`[BRW-5]`).
* `[SPN-8]` `as_ptr()` and `as_mut_ptr()` return raw pointers; extracting one is safe, using one needs
  `unsafe`, and extraction extends no region.

## VII.8 Destruction

* `[DRP-1]` *(changed in 0.9.15)* `fn drop():` runs exactly once per value, at the end of its life.
  It changes its own object as a given, since it ends it, and its first line says nothing about that.
  Calling it explicitly is `E3070`; `mem.drop(x)` ends a value early.
* `[DRP-2]` Locals drop at the end of their block in reverse declaration order; fields after their
  owner's `drop`, in reverse declaration order; collection elements in index order; the active variant
  of an enum; tuple elements in reverse order.
* `[DRP-3]` Temporaries drop at the end of the statement that created them (`[EXP-4]`).
* `[DRP-4]` *(changed in 0.9.9)* A panic inside `drop` aborts the process (`[PAN-1]`). A `drop` SHOULD
  NOT block; `@noalloc` on a type's `drop` is honoured wherever that type is dropped.
* `[DRP-5]` *(changed in 0.9.15)* A `drop` body may not take fields out of `this` (`[EXP-6]`); it uses
  `mem.take(ref mut this.f)` on `Option` or `Default` fields. It makes no owner of its own object
  (`[OBJ-5]`).
* `[DRP-6]` *(changed in 0.9.15)* Ending an `owns T` ends the `T` and frees its block; ending a class
  object's owner ends the object (its `drop`, then its fields) and frees its block unless `weakref`
  links still hold it (`[OBJ-3]`); ending a `shares`, `shares mut` or `thread_shared` owner subtracts
  one from the count, and the subtraction that reaches zero ends the value (`[RC-1]`); ending a
  `weakref` link subtracts one from its target's weak count (`[WK-11]`); ending a plain view (`ref`,
  `Span`, `StrView`, a `sharedref mut` link) does nothing; ending a guard (`MutexGuard`, an `RwLock`'s)
  unlocks; and ending a view type ends whatever in it needs ending. Carrying a region and needing drop
  are independent properties (`[TYP-1]`) (ODR-053).
* `[DRP-7]` *(new in 0.9.9)* *(changed in 0.9.15)* A value whose `drop` may read through a view it holds must be dropped
  while that view's region is valid. A generic type parameter is assumed to be read by `drop` unless the
  type has no `drop` of its own. Violations are `E3060` (shape B7).
* `[DRP-8]` *(new in 0.9.15)* **Ending a value counts as a call of its `drop`.** Anything that may end a
  value whose type has a `drop` (a list cleared, a value replaced, a local ending with its function, a
  function value or an `owns dyn I` value replaced or ended with what its state owns, `[CLO-10]`) makes
  the changes that `drop` makes, wherever a value of its type may end, for every rule that needs them
  (`[EFF-1]`). With `class Enemy:` whose `fn drop():` pushes to `LOG`, `enemies.clear()` inside
  `for line in LOG:` is `E3020` ("`enemies.clear()` (line 5) ends every `Enemy` in `enemies`, and an
  `Enemy`'s `drop` pushes to `LOG` (its line 12), which this loop walks (line 4); clear after the loop:
  move line 5, `enemies.clear()`, below the loop"), and a function that may end an `Enemy` changes `LOG`.
  A `drop` may change things outside its object (a log line, leaving a list).

## VII.9 What a newcomer sees

```ember
class Player:
    name: String
    health: float = 100.0

fn total_health(players: Array[Player]) -> float:      # reads the list
    total = 0.0
    for p in players:
        total += p.health
    return total

fn heal_all(ref mut players: Array[Player]):           # may change the players
    for ref mut p in players:
        p.health = 100.0

fn take(owns players: Array[Player]) -> int:           # takes the list over
    return players.len()

fn main():
    ps: Array[Player] = [Player("a"), Player("b")]
    println(total_health(ps))
    heal_all(ref mut ps)
    n = take(ps)
    println(n)
```

Each name says what it is: `players` in `total_health` reads the caller's list, `ref mut players` in
`heal_all` may change it, and `owns players` in `take` takes it over. The call `heal_all(ref mut ps)`
says that `ps` may be changed; `take(ps)` hands `ps` over, and using `ps` after it is `E3040`,
labelled at the hand-over. Nothing in this program names a lifetime, and nothing in it is checked while
it runs.
