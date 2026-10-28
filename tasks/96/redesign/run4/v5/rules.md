# The memory model's rules (run 4, step 4, v5; agent 01, rule writer)

The rules of Ember's memory model: run 4's rules (`RUN4/rules.md`) brought up to date with every verdict
the owner has given on them (`inputs/JOB.md`: the verdicts before run 4, and verdicts 1 to 97 on run 4's
open questions, where 27 replaces 25, 26 replaces 1 and 95 reverses 90, with the corrected records of
verdicts 53 and 58), with the rule and code items the area files and the checks left for the rule writer
(`V5/inputs/worklist.md`). They are written clean: what changed, and why, is only in `V5/changes.md`; the
words are in `V5/vocabulary.md`. `RUN3` means `tasks/96/redesign/run3/`, `RUN4` means
`tasks/96/redesign/run4/`, `V5` means `tasks/96/redesign/run4/v5/`. "Row N" is the owner's agreed row N in
`inputs/JOB.md`.

No compiler change, no git command, no Ember program run.

## 0. Terms

- **owner**: the one place (a local, a field, a list element) that ends a value when it ends. A value
  made shared has several owners (`shares`, below).
- **handed over**: the giver's name ends and the receiver becomes the owner (row 9: `snapshot = items`),
  for every type, numbers included, wherever a line hands a value over (`fn() owns count`; an argument
  for an `owns` parameter; R3, R4).
- **copy** (`copy of`): a new value equal to the old one; nothing done to one is seen through the other.
  The copy word copies the thing itself, never the link it is written before: `copy of` a link copies
  its target, or is refused; a value that holds links among its fields is copied with them, each one more
  link to the same target (R3).
- **`Copy` type**: a type whose `=` makes such a copy bit for bit: numbers, `bool`, `ref T`, `Span[T]`,
  `StrView`, and structs made only of these. Class objects, `shares` values, `weakref` and
  `sharedref mut` links, `ref mut` views and `MutSpan`s are not `Copy`.
- **place**: where a value lives: a local, a field, a list element, an enum variant's payload.
- **part**: a field (`p.name`), an element (`enemies[0]`), or a payload (`State.Walking(path)`).
- **view**: a name or value that points into a place someone else owns and may not outlive it:
  `ref T`, `ref mut T`, `sharedref mut T`, `Span[T]`, `MutSpan[T]`, `StrView`, a plain parameter, a
  plain loop or pattern name, a name opened with `exists as` (not `exists as copy of`, a copy of your
  own) or named by a kind test (`is T as`), a name a function captured with `ref` (R12), a name for a
  part. A **read-only view** is every view except `ref mut T`, `sharedref mut T` and a `MutSpan[T]` held
  by a `ref mut` name (a plain name holding a `MutSpan` only reads its elements, R5). A **kept view** is a
  view stored in a struct or a list.
- **into X**: a view points into X when it points at X or at anything X owns, all the way down (X's
  fields, the elements of X's lists, X's `owns` parts). It does not go through links: what a link
  reaches is another object (R9, R10).
- **link**: a field or value that names an object it does not own alone: `weakref T` (does not keep the
  object alive), `shares T`, `shares mut T` and `thread_shared T` (one of several owners),
  `sharedref mut T` (a view several holders may share, R7).
- **parent link**: a `weakref` field that names the object that owns its holder (`parent: weakref
  Node`), which the program gives its target like any field (`child.parent = weakref this`) (R11).
- **position**: a view of a list element that finds the element again at each use (R8 (a)).
- **reshape**: a change that can free or move memory: replacing a value that owns memory; `push`,
  `pop`, `insert`, `remove`, `clear`, `sort`, `retain`, `extend` on a list, map, set or string; changing
  an enum's variant; ending an object's last owner. Writing a number or a `bool` is not a reshape. A
  reshape moves or frees only what it may move or free: adding to a list (`push`, `extend`) may move its
  buffer and every value stored inline in it, but not a class object, which lives in its own block and
  is not moved when the list grows (R8 (b)).
- **`this`**: inside a method, the object the method was called on.
- **type parameters**: written on a line of their own, directly above the function, method, struct or
  class they belong to, at its indentation: `[F: fn(Item)]` over `fn for_each(owns f: F) owns this:`;
  `[T]` over `struct Pool:`. A type is used as today, `Pool[int]`, and a lambda has none (verdict 50). In
  prose, a span cannot hold two lines, so it is written "`[T]` above `fn f(x: T)`".
- **first line**: a function's signature: its parameters, each with the word in front of its name (R4);
  `owns this`, for a method that takes its object over; the result type; `from` (R5, R6). It says
  nothing about what the function changes or reads: the compiler works that out from the code (R5).
- **summary**: what the compiler works out from a function's code, with no word written (R5): what it
  may change, read and end, and through which calls (the chain of calls a message names); which elements
  of a list it touches; whether it sets the link of an object it is given before reading it (R11); which
  of its view parameters, `this` included, it uses after a line that may end or move what they view, and
  what it reaches by other names while a `ref mut` parameter of it is used (R4); which locks it may take
  (R15). Across a package's edge it is recorded in the package's interface file (R17).
- **`shares` / `thread_shared` value**: a value with several owners, all on one thread / on any
  threads, freed when its last owner ends. Its owners are **counted**: a number in the value's header,
  changed with a plain add (one thread) or an atomic add (any threads). A `shares` owner reads the
  value; a `shares mut` owner may also change it (R9).
- **weak count**: a second number in an object's header; it keeps the object's memory block (not the
  object) until the last `weakref` link to it ends, so a link can still test whether the object lives.
- **ended**: an object is ended the moment its last owner lets it go, before its `drop` runs; from then
  on a `weakref` link to it finds nothing (R11). Ending an object counts as a call of its `drop` (R5).
- **count pair**: one +1 and one -1 of a count around a use, to keep an object alive across it (R10
  point 2, R12).
- **deferred-drop region**: a loop during which an object whose last owner ends is set aside whole:
  it and everything it owns are marked ended at once, and their memory is freed when the loop ends (R11's
  walk).
- **function value**: a function or lambda kept in a variable, a list or a field: two words, its code
  and its **state** (what it captured with a word). Its type says its parameters, its result and whether
  any thread may run it (`thread_shared fn()`); what it changes is worked out from every function the
  program can put there (R12). A **capture** is a name a lambda uses from the function that made it;
  **the real variable** (row 6) is that very variable, not a copy.
- **kept callback**: a function value C keeps after the call that hands it over (a parameter whose
  contract says `callback(retained)` or `callback(once)`), and runs during later foreign calls (R12).
- **unseen code**: code whose body the compiler cannot read, which is only a foreign (C) function: a call
  of one counts as possibly running every callback C keeps, unless its declaration says
  `@ffi(no_callbacks)` (R12). Every Ember function is read, a library's included, and so is every
  function the program can put in a function value or behind an interface, `dyn` or virtual method (R5).
- **kind test**: `x is T`, which asks whether a value of an interface or base-class type is a `T`
  (R14).
- **memory manager**: where a value's heap memory comes from: the built-in one, or one named with
  `using` (a type implementing `Allocator`, `[ALC-1]`): its kind in a type, its instance on the line
  that makes the value (R3).
- **scope**, **task**, **join**: the block `with scope = thread.scope():`; the threads started in it;
  the end of the block, which waits for every task.
- **noalias**: C's `restrict`, the promise to the C compiler that two pointers never point at the same
  memory, which lets it keep values in registers and use vector instructions.
- **the owner's test**: for every name a line declares, the word that says what it is (the owner's own
  thing, a copy, or handed over) and the word that says whether it may change it; a name with no word
  only where the stated default never changes anything, never aliases silently, never copies silently.

## 1. The rules

**R1. One owner; `=` with a whole name hands it over.** Every value has one owner unless a line says
otherwise (`shares`, R3). `b = a`, where `a` is a whole name that owns a value whose type is not
`Copy`, hands `a` over: `a` ends, and a later use of it is refused (E3040, naming both ways out,
`copy of` and `shares`; row 9). Where `a` is itself a view (a plain parameter, a loop, pattern or opened
name, a local holding a view), `b = a` makes `b` a read-only view of what `a` views, and `ref` may say
so (`cur = ref p`); a view hands nothing over. For a `Copy` type `=` copies; the word `owns` hands over
every type the same way, numbers included (R3, R4, R12). A `Copy` struct copied with `=` from a name that
is still used afterwards is not then changed through the new name (E3023): the line says `copy of` (a copy
of your own; every `Copy` type has `Clone`, R3) or `ref mut` (the original), so no copy is changed by
mistake. A value always ends with its clean-up: when its last owner ends, its `drop` runs, and nothing
ends an owner without it (`mem.forget` is not Ember, E1010). An object a C function keeps is handed to it
as to any `owns` parameter where the C function's declaration says it takes the object; for a C function
whose declaration cannot say so, `into_raw()` gives C the object's address, only inside `unsafe:` (R16).

**R2. A name for a part is a read-only view; `=` puts a value in the place a name stands for.** A name
is declared by the line that first gives it a value; a line inside a loop's body declares it once per
round (`[GRM-4]`). A name bound to a part (`n = p.name`, `boss = enemies[0]`), or to a call whose
result is a view (`ref T`, `StrView`, `Span[T]`, a `ref` or `ref mut` place; R6)
(`word = line.to_lower().trim()`, row 5; `best = a.as_span()`), is a read-only view of it: never a
hand-over, and never a silent copy, except that a part with no fields of its own (a number, a `bool`,
a variant with no payload) is copied, as Python binds a number. A call whose result is a new value the
callee made (a list it built, a list of views included) hands that value to the name, which owns it
(R7 bounds the life of a value that holds views). A word before the part says otherwise: `ref mut`
makes a name for that very place that may change it (`boss = ref mut enemies[0]`,
`r = ref mut first_even(ref mut xs)`), and before a `MutSpan` a call gives it says the name may write
the span's elements (`d = ref mut tracks[dst_name].as_mut_span()`; a plain
`d = tracks[dst_name].as_mut_span()` only reads them, R5); `copy of` makes a copy of your own (rows 6, 9:
`boss = copy of enemies[0]`); the link words of R3 make a link; `ref` says the default, and before a
part it is needed only to view a number (`r = ref counts[0]`). A part stored somewhere else (`push`,
`return`, a field, an `owns` argument) is handed over on its own, as in Rust: for a struct value `p`,
`names.push(p.name)` moves `p`'s name into the list, and `p.hp` stays usable; `p.name`, and `p` used
whole, are refused until `p.name` is given a new value (`p.name = String("")`) (E3040: "`p.name` was
handed over to `names` on line 4; give it a new value first, or keep a copy:
`names.push(copy of p.name)`"). A field of a class object is not handed out this way, since other names
may reach the object and would see the hole (E3013); `mem.take(ref mut q.name)` or
`mem.replace(ref mut q.name, String(""))` takes it and leaves a value in its place.
`=` on a name that already has a value puts the new value in the place the name stands for. A name
declared with `=` stands for its own place: its old value is replaced (and ends, if it owned
something), and if the value is a view, the name now views something else (`rest = tail.trim()`,
`cur = p`), checked by R8 against its new target. A `ref mut` name stands for the place it was made
from until it is given another ref: `r = ref mut a` then `r = 20` writes `a`, and `r = ref mut b` points
`r` at `b`, after which `r = 30` writes `b`; the line that gives it another ref says so with `ref mut`
again, for every type, and a read-only `ref b` given to it is refused (E3023: "`r` may change what it
refers to, and `ref b` only reads `b`; to point `r` at `b` write `r = ref mut b`, or for a read-only
name `rb = ref b`"). A name that holds a `MutSpan` likewise: `d[i] = v` writes an element through it,
and `d = ref mut other.as_mut_span()` points it at another span. A name written **`const ref mut`** is
never given another ref (`r = const ref mut a` then `r = ref mut b`, E3023), as `const` on a value
means it never changes: `const` says the name itself stays as it is. A parameter, a loop name, a pattern
name and a name opened with `exists as` (R13) or named by a kind test (R14) stand for what the caller,
the loop, the opening or the test gave, and keep it for their whole scope: `=` on one is refused
(E3023), and so is any other change through one that is plain (R5), with one exception: a name opened
`exists as copy of v` is a copy of your own and may be changed, as `boss = copy of enemies[0]` may
(`if scores.get(name) exists as copy of v:` then `v += 1`; the map is untouched). A `ref mut` one writes
the caller's place, and may be given another ref, as any `ref mut` name may: a `ref mut` parameter of a
view (`ref mut dst: MutSpan[float]`) given another span points the caller's `d` at it, and one of
another type (`ref mut c: Counter`) given another ref points `c`, for the rest of the call, at that
place. `const ref mut dst: MutSpan[float]` writes the caller's elements and is never given another ref,
so the caller's `d` is the same span after the call.

**R3. Words before a value, in a type, and in a list of names.**
- Before a value, a word acts on the object the value names (for a link or an owner, on its target):
  `copy of x` (a copy of the thing itself, made by its type's `Clone`; for a `shares` or
  `sharedref mut` link, a copy of its target); `shares x` (one more owner, on this thread, that reads
  it); `shares mut x` (one more owner that may also change it); `thread_shared x` (one more owner, on
  any thread: only before a new value, `names = thread_shared Names()`, or on a value already
  `thread_shared`, else E7017); `weakref x` (a link that does not keep it alive; before a `weakref`
  link, another link to the same target: `a = weakref this.keep`; R11); `sharedref mut x` (a
  changeable link other holders may share, R7). An owner made from another may do no more than it:
  `shares mut x` needs `x` to be a name that may change the value (E3023). The name a value is shared
  from keeps what it could do: after `other = shares mut items` (or `other = shares items`), `items` may
  still change the list if it could before, and so may every `shares mut` owner (R9). `copy of` works on
  every `Copy` type: `@derive(Copy)` gives `Clone` too, as in Rust (`b = copy of a` for a `Point` marked
  only `Copy`). A value that holds links is copied field by field, as a C++ struct holding a
  `shared_ptr` and a `weak_ptr` is: what it owns alone is copied itself, and each link is copied as a
  link to the same target: for `struct Team:` with `captain: shares Player` and
  `scout: weakref Player`, `t2 = copy of t` is a new `Team` whose `captain` is one more owner of the same
  player and whose `scout` is one more link to it; a `sharedref mut` field is copied as one more holder
  of the same link. `copy of` is refused before what it cannot copy as itself (E2116): a `weakref` link
  (its target may be gone: open it first, R13, or make another link with `weakref`), a value of a view
  type (`StrView`, `Span[T]`, a struct holding a `ref mut` field: write `String(w)`, or an `Array[T]` of
  a `Span[T]`), a type with no `Clone`, and a value whose views point into its own `let` field.
- **`owns` before a value hands it over, for every type, numbers included.** It is written, and
  required, on the line that makes a value for a place that owns it alone: an `owns` field, with or
  without a memory manager (`boss = Enemy(hp=500, mesh=owns Mesh(points=[0.0, 1.0]) using current)`,
  `e = Entity(mesh=owns Mesh(points=[0.0]))`); an `Option` or a list whose values each live in their own
  block (`left=Some(owns Tree(value=2))` for `left: Option[owns Tree]`;
  `r.pools.push(owns IntPool(items=[1, 2]))` for `pools: Array[owns dyn Pool]`;
  `shapes=[owns Circle(r=1.0), owns Square(side=2.0)]` for `Array[owns dyn Shape]`); and a value made
  later for such a place (`e.mesh = owns Mesh(points=[1.0, 2.0])`, which frees the old mesh). The line
  without it is refused (E2123: "`mesh` owns its mesh; say so where it is made:
  `mesh=owns Mesh(points=[0.0, 1.0]) using current`"). It is written where the default is a view, as a
  `for` loop's or a `match`'s source (`for x in owns xs:`, `match owns e:`), which then hands the value
  over. It is not written before a name whose place already hands it over, nor before `fn` (E0109):
  `e.mesh = m` hands `m` over with no word, as `b = a`, an argument for an `owns` parameter and
  `return a` do. A number given with `=` is copied, so `b = owns n` is where `owns` hands a number over
  by `=`: `n` ends there (E3040 if used after), as it would given to an `owns` parameter or captured
  with `owns` (verdict 84; R4, R12).
- In a type: `T` (held alone: a struct's value inside its owner, a class object in its own block);
  `owns T` (held alone in its own heap block, replacing `Box[T]`, `Box[dyn I]` and `ForeignBox[T]`;
  refused on a class type, which always has its own block, E2114); `shares T` and `shares mut T`;
  `thread_shared T`; `weakref T` (R11); `ref T`, `ref mut T`, `sharedref mut T` (R7); `owns dyn I`,
  `shares dyn I`, `shares mut dyn I` (a bare interface name used as a type is refused, E2173); a
  function type, which says its parameters and result (`fn(ref mut Ui)`, `fn(owns String) -> int`) and
  whether any thread may run its functions (`thread_shared fn()`), and nothing about what they change
  (R12).
- **Which memory manager: the place names the kind, the making line names the instance.** `using A`
  after a type whose value has heap memory of its own names the kind of memory manager its memory comes
  from, a type implementing `Allocator`: `mesh: owns Mesh using LevelArena`,
  `mesh: shares Mesh using LevelArena`, `names: Array[String] using LevelArena` (an `owns`, `shares`,
  `shares mut` or `thread_shared` value, a class object, an `Array`, `Map` or `Set`; after any other
  type it is refused, E2114); without `using`, the built-in one (`Global`). The line that makes such a
  value names the instance after the value it makes: `boss = Enemy(hp=500, mesh=owns Mesh(points=[0.0,
  1.0]) using current)` takes the mesh's block from the arena `current` by calling
  `current.allocate(...)`; a class object is made `Enemy(...) using current`, a list `[0.0, 1.0] using
  current` (`Array.new_in(current)` is the library's call for an empty one, `[ALC-1]`). Checked when
  compiling: the instance is of the kind the place names, and a value given to a `using A` place was
  made in an instance of `A` (E2020); a value made in an instance, and every value that holds one, lives
  no longer than that instance (R7's rule for a value that holds views, E3063). A manager with one
  instance (`Global`, `System`) needs none on the making line. `using` names where that value's own
  memory comes from; what the value holds comes from where its own fields' types say (the `points` list
  of that `Mesh` from the built-in manager, unless `Mesh` says otherwise). A value whose manager may have
  several instances keeps its instance's address, so that its growth and its end go back to the
  instance it came from (one word, as Rust's `Box<T, &A>` keeps its `&A`). That address is a kept
  `sharedref mut` link to the instance (R7): the program may use the instance while values made in it
  live, and each value's growth and end change the instance through it (`allocate`, `deallocate`). A
  manager with one instance costs no word.
- In a lambda's capture list, after its parameters: `fn() copy of a, owns b, shares c, shares mut d,
  thread_shared e, weakref f, ref g:`; each word means what it means before a value (`ref g`, a read-only
  view of what `g` names at that moment, for a loop's name the element it is on in that round, R15), and
  a name not listed is the real variable (row 6, R12): `b.on_click(fn(): score += 1)` changes `game`'s
  real `score`, and `b.on_click(fn() copy of score: println(score))` keeps a copy of its own. `owns`
  hands over every type the same way, numbers included: after `fn() owns count`, the function that made
  the lambda no longer uses `count` (E3040), and the lambda uses it under the same name. `ref mut` is
  not a capture word: a name not listed already is the real variable (`fn() ref mut x: x += 1` is
  refused, E0120: "remove `ref mut x`: without a word the function already uses the real `x`, and may
  change it: `fn(): x += 1`").
- In a list of names declared without types of their own (a capture list, a loop's names, a pattern's
  names, a lambda's parameters written without types) each word belongs to the one name after it, and a
  name with no word may not follow a name with a word, so that no word can be read as covering the next
  name (E0116): `for ref mut pos, ref vel in q:`; `fn() copy of name, copy of score:`;
  `fn(ref mut ui, ref dx)`. `ref` says a name's default (a read-only view) and may be written on any
  declared name. A parameter list with types, and a call's arguments, are not such lists.

**R4. Parameters and calls.**
- **What a function may do with a parameter is set by the word in front of its name, for every type;**
  the type after the colon says what the caller's value is, never what the function may do with it.
  `x: T` reads the caller's value (a read-only view; a number is copied in), and promises that it does
  not change while `x` is used (R8 (g)). `ref mut x: T` names the caller's place: every change through
  `x` reaches the caller's value, and `x = v` puts `v` there; where the value is a link or a view, the
  place holds the link or the view (`ref mut b: shares mut Player` may re-point the caller's link and
  change the player; `ref mut best: Span[int]` may point the caller's view elsewhere;
  `ref mut dst: MutSpan[float]` may write the elements the caller's span views, and point that span
  elsewhere). `owns x: T` takes the value over, for every type, numbers included: the caller's name ends
  (E3040 at a later use; the caller keeps its own with `copy of`: `f(copy of n)`). `owns x: shares T`
  makes the callee one more owner that reads the value, `owns x: shares mut T` one that may change it.
  So `x: shares T` and `x: shares mut T` both read the caller's shared value, with no count;
  `game: shares mut Game` cannot make an owner that changes the game (`fn() shares mut game`, E3023),
  and a function that hands one on takes one: `fn setup(owns game: shares mut Game)`, called
  `setup(shares mut game)`, with `fn() owns game`. In the same way a `MutSpan` parameter that writes its
  elements says `ref mut` (`fn blend(ref mut dst: MutSpan[float], src: Span[float]):`, called
  `blend(ref mut d, s)`); a plain `dst: MutSpan[float]` only reads them (E3023 at a write). Only a
  `Mutex`, `RwLock`, `Atomic` or channel end, and an object whose fields are `guarded by` a lock (for
  those fields, inside that lock), may be changed through a plain name, by its own operations (R5): a
  `guarded by` field is a `Mutex` written as ordinary fields, and follows the `Mutex`'s rules, no word in
  front of the parameter or at the call (`fn add_line(log: Log): log.push("hi")`;
  `fn start(log: thread_shared Log):`, whose threads push). A lambda parameter's mode is what is written
  on it, and matches the function type it is given to (R12); a callable type writes the same words
  (`fn(ref mut Ui)`, `fn(owns String)`).
- **A call says `ref mut` before an argument the callee may change**: an argument given to a `ref mut`
  parameter, with the same words as the parameter, `const` included (`bump(ref mut a)` for `fn bump(ref
  mut c: Counter):`; `collide(ref mut bodies[i], ref mut bodies[j])`; `this.handlers[k](ref mut this)`;
  `blend(ref mut d, s)`; `blend(const ref mut d, s)` for `fn blend(const ref mut dst: MutSpan[float],
  src: Span[float]):`). Before a `MutSpan` the word names its elements, so it may stand before a span
  made in the call (`integrate(ref mut p.as_mut_span(), v.as_span(), 0.5)` for `fn integrate(ref mut
  pos: MutSpan[float], vel: Span[float], dt: float):`); a callee that points such a span elsewhere
  changes only that span, which ends with the line. A missing `ref mut` or `const`, or one written where
  the parameter does not say it, is refused (E3028), and so is `ref mut` before something this function
  may not change (E3027). An argument given to a `Mutex`, `RwLock` or `Atomic` parameter, or a channel
  end, takes no word: these exist to be shared between threads, and their own "only me" is the lock's
  block (R5). The object a method is called on is not marked (`a.reset()`): what the method does to it
  is worked out from its body (R5). An argument for an `owns` parameter is handed over with no word, for
  every type (`f(n)` ends `n` when `f` takes `owns x: int`), as `=` hands over a list (row 9).
- **Arguments do not overlap a changeable one.** Two arguments of one call (the object a method is
  called on counts as one), of which at least one may be changed (a `ref mut` argument, a `MutSpan`
  one included, the object of a method that may change it, or a function value, which counts as
  changing every place the functions it may run change, R12), may not name one place, and neither may
  reach into what the other owns, unless the compiler proves they differ (different constants, `j` from
  `i + 1..`, inside `if i != j:`, after `assert_disjoint(a, b)`, with `from std.mem import
  assert_disjoint`). Two elements of one list whose positions are not proved different (`bodies[i]` beside
  `bodies[j]`) are refused like any other overlap; `if i != j:` above the call is the way out (verdict
  98). Any other possible overlap is refused
  when compiling: E3022 where both may be changed, E3021 otherwise. This holds for spans of numbers too:
  two spans of one list given to one call, one of them changeable, are proved apart (`assert_disjoint`,
  two compares) or refused. Two exceptions:
  1. **The same whole variable under two names.** Where the compiler proves when compiling that two
     `ref mut` arguments, or a `ref mut` argument and a variable the callee uses by its own name (a
     module variable), are one whole variable, the call is allowed and compiled against a copy of the
     function in which the two names are one: `fn add_both(ref mut a: int, ref mut b: int):` whose body
     is `a += 1`, `b += 1`, `println(a)`, called `add_both(ref mut x, ref mut x)`, prints 2; `fn
     rename(ref mut s: Array[String]):` whose body is `LOG.clear()` then `s = [String("renamed")]`,
     called `rename(ref mut LOG)`, leaves `LOG` holding one string (it frees nothing: `clear()` empties
     the list, `s = ...` fills it again). Other calls use the normal copy, where a `ref mut` parameter is
     the only way in. Parts whose positions are not known when compiling (`ref mut log[i]` beside
     `ref mut log[j]`, or beside `log`) stay refused (E3022).
  2. **A thing whose changes go only through its lock.** A `thread_shared` object whose every field that
     may change is `guarded by` its lock (or is an `Atomic`, `Mutex` or `RwLock`) may reach one call
     through any number of names, a plain argument and a function value that holds it included:
     `dump(log, fn() thread_shared log: log.push("seen"))`, whose `dump` reads the lines inside
     `with log.lock:` and calls `note()` after the block, is allowed, as Rust allows the same with `&Log`
     and a `Mutex`: the lock keeps the two ways in apart. A call inside the lock that may take it again
     is refused (E7018, R15).
- **A view argument and what the callee does** (the call check). A view whose last use is an argument
  of a call (of a plain, `Span` or `ref mut` parameter) does not live across the call (R8 (g)): inside
  it, the parameter makes the promise, and the compiler holds the callee to it. It works out from each
  function's body which of its view parameters, `this` included, it uses after a line that may end or
  move what they view (a reshape, R8 (b); the end of an object, R10), through its own lines and the same
  fact of the functions it calls (a call through a function value, or through an interface, `dyn` or
  virtual method, by every function the program can put there, R5), with the body's precise facts (a
  change inside `if a is not b:` is not one for `a`); it keeps that fact with the function (in the
  package's interface file across a package's edge, R17), and checks it at each call: an argument that
  points into what the callee may end or move before its last use of that parameter is refused (E3031).
  Writing a number or a `bool` ends and moves nothing, so a method that writes `this.health` through a
  link that may lead to its own object (a turret aimed at itself) is allowed; leaving a lock's `with`
  block ends nothing either. With `fn note(s: StrView):` whose body is `LOG.clear()` then `println(s)`,
  `note(LOG[0])` is refused ("`note` clears `LOG` (its line 3) and then uses `s` (its line 4), and
  `LOG[0]` is part of `LOG`; give it a copy: `note(copy of LOG[0])`, or in `note` use `s` before
  `LOG.clear()`"); `note("bye")` is allowed, and so is `note(LOG[0])` for a `note` that prints before it
  clears. The object a method is called on counts the same way: with `fn hit():` whose body is
  `ENEMIES.clear()` then `this.hp -= 1`, `ENEMIES[0].hit()` is refused (E3031), and a `hit` that uses
  `this` only before the clear is allowed. A `ref mut` parameter counts the same way: with
  `fn rename(ref mut s: String):` whose body is `LOG.clear()` then `s = String("renamed")`,
  `rename(ref mut LOG[0])` is refused (E3031). Since a `ref mut` parameter is compiled with noalias (next
  point), the check is wider for it: while it is used, the callee reaches its place through no other
  name, so an argument for it that points into a place the callee reads or changes by another name while
  the parameter is used is refused too (E3031: with `fn heal(ref mut p: Player):` whose body is
  `println(PARTY[0].hp)` then `p.hp += 10`, `heal(ref mut PARTY[0])`), unless it is that same whole
  variable, proved when compiling (exception 1 above). Where the fault is in one function stored in a
  function value (it ends what its parameter views and then uses the parameter), the refusal stands on
  the line that stores it, not on the call through the value, which is right for every other function
  (R12, R18). Only a C function's code is unseen; R12's rules for C cover it. Nothing is checked while
  the program runs.
- Every `ref mut` parameter, a `MutSpan` one included, is compiled with noalias: while it is used,
  nothing else the function uses may reach its place (R8 (f)); a call compiled against a copy with one
  name (exception 1) has one name for that place. `this` is never compiled with noalias (R5).
- **A view put in through `ref mut`.** After the call, a caller's place given to a `ref mut` parameter
  whose value is a view may point into what it pointed into before, or into any of the call's other
  arguments (R6). Inside the callee only such a view may be put there: another view argument or a part
  of one, or text written in the program; a view of the callee's own locals or of a temporary is
  refused (E3063).

**R5. A function's first line, and what the compiler works out from its body.** Inside a method its
object is `this`; a method does not list it among its parameters. A function in a type body is a method
unless it is written `static fn`: it then has no `this` and is called on the type (`static fn
starting_at(k: int) -> Counter:`, called `Counter.starting_at(5)`); `static fn` outside a type body is
refused (E2115). A function's first line says its parameters, each with the word in front of its name
(R4), then `owns this` for a method that takes its object over (the caller's name ends), then the result
type, then `from` (R6), in that order (`fn finish() owns this -> u64:`; `fn next_word() -> StrView from
src:`); a first line in another order is refused (E0118: "the parts of `finish`'s first line are out of
order; write `fn finish() owns this -> u64:`"). It says nothing about what the function changes or reads.
- **The compiler works out what every function may change, read and end from its code**: its own lines;
  the functions it calls; every function the program can put in a function value it calls, or behind an
  interface, `dyn` or virtual method it calls (it reads the whole program, the libraries included: every
  function value stored into that type, every override and implementation); what the threads it starts
  and the callbacks it hands to C change, now or later (R12, R15); and the `drop` of every object it may
  end. **Ending an object counts as a call of its `drop`**: anything that may end one (a list cleared, a
  value replaced, a local ending with its function, a function value or an `owns dyn I` value replaced
  or ended with what its state owns, R12) makes the changes its `drop` makes, so with `class Enemy:` whose
  `fn drop():` pushes to `LOG`, `enemies.clear()` inside `for line in LOG:` is refused (R8 (c), E3020). A
  `drop` changes its own object as a given, since it ends it; what it changes outside its object counts
  wherever an object of its type may end.
- **That fact decides every rule that needs it, with no word on any first line**: a loop and a call that
  may change what it walks (R8 (c)); a read-only name and a call that may change what it reads (R8 (g));
  a call that hands a function a part of what it may end (R4, E3031); a view of an object reached
  through a link and a call that may end the object (R10); a function value stored where it would change
  its holder (R12); a call that may take a lock this thread holds (R15); `init` calling a method before
  the fields its body reads are set (E2100); and what a `@cached` method's result depends on (E2117).
  Nothing is checked while the program runs. Across a package's edge the fact is kept in the package's
  interface file (R17): an edit inside one function, or a library update, that changes it can make a line
  elsewhere stop compiling, and the message names the chain of calls ("`tick()` (line 10) adds to `LOG`
  (through `note`, line 4) while this loop walks `LOG` (line 9); call `tick()` after the loop: move line
  10 below the loop"). C code cannot be read: a foreign call counts as possibly running the callbacks C
  keeps, unless its declaration says `@ffi(no_callbacks)` (R12).
- **What a method does to its object is worked out the same way.** `fn reset(): this.count = 0` changes
  it; `fn note(msg: String):` whose body is `this.log.push(msg)` changes `this.log`, so `this.note(...)`
  inside a loop over `this.log` is refused and inside a loop over `this.players` is allowed (row 3);
  `fn load_assets() -> Array[String]:` whose body reads only `this.renderer` reads that field (row 7); a
  method whose body neither changes nor ends anything of `this` only reads it. A method that may change
  its object is not called through a name that only reads it (E3023). That a method may change its
  object promises nothing about other names: the same object may be reached and changed through a link
  while the method runs (a turret aimed at itself: `if this.target exists as ref mut t: t.health -= 1`,
  accepted with no check), so `this` is never the only name of its object (R8 (f)) and is never compiled
  with noalias; R8 (b), (c) and (g) keep every view and every walk safe, and a call whose object is part
  of what the method may end before its last use of `this` is refused (R4, E3031). `init` sets the fields
  of the object it builds through `this` with no word, and may call a method of it once every field that
  method's body reads is set (row 7: `this.renderer = r` then `this.assets = this.load_assets()`; E2100
  before: "`init` calls `load_assets()` (line 6), which reads `this.assets` (its line 14), and `assets`
  is set only on line 7; give it a value first: write `this.assets = []` above line 6"); `owns this` on `init` is refused (E2108).
- **A plain name never changes anything**: a plain parameter, a plain loop, pattern or opened name, and
  a plain local that holds a view (R2) never change what they name or what it leads to (a `Cell` field,
  an object behind a `shares` link, the elements of a `MutSpan`): E3023, with the line to write
  (`d = ref mut tracks[dst_name].as_mut_span()`; `fn blend(ref mut dst: MutSpan[float], src:
  Span[float]):`). The types made to be shared between threads are used through any name, a plain one
  included, by their own operations: a `Mutex`, `RwLock`, `Atomic` or channel end, and the `guarded by`
  fields of an object under their lock (`with g = m.lock():`; `with log.lock: log.lines.push(...)`;
  `total.fetch_add(n)`; `tx.send(v)`; `rx.recv()`). Each orders its own change (the lock's block, an
  atomic instruction), as Rust's `&self` methods of these types do, and handed to a call it takes no word
  (R4).

**R6. Views a call gives.** A call's result that is a view, and a view a call puts in through a
`ref mut` parameter, point into the arguments they may come from: a method's into `this`, a function's
into every view argument (into the one, if only one could be the source). `-> T from name` narrows it to
one parameter, or to one field of `this` written bare (`-> StrView from src`); the body may then give
only a view of what `from` names (E3062). Pointing into an argument means pointing into it and into
everything it owns (section 0), so a reshape of anything it owns ends the view (R8 (b)) and any other
change of it while the view is used is refused (R8 (g)), as Rust's borrow of the whole argument is. A
call whose result type is `ref mut T` or `ref T` is a place (`first(ref mut xs) += 1`;
`counts.entry(k).or_default() += 1`), and a name bound to it follows R2
(`r = ref mut first_even(ref mut xs)`). A generator method's result is such a view, and it walks what
it views: after `g = d.replay()`, the list it walks (`d.log`) does not change while `g` will still be
used (R8 (c), E3020), however `replay` walks it; any other change of `d` while `g` will still be used
(`d.angle = 90.0`) is a change of what a view a call gave points into (R8 (g), E3021), unless
`-> Generator[int] from log` narrows the view to `d.log`.

**R7. Kept views.** A struct that holds a view (`ref T`, `ref mut T`, `sharedref mut T`, `Span[T]`,
`MutSpan[T]`, `StrView`, or a struct that holds one) lives no longer than what its view fields point
into (`[TYP-34]`); `@view` may say so and is checked. A value that holds views (a local list of views,
a struct made by a call) lives no longer than what they point into, whoever owns it. A class object
is never a view: a view field of a class says what it points into, `from <let field>` (a `let` field
of the same object whose contents live in their own heap block: a `String`, an `Array` or an `owns T`)
or `from static` (text written in the program; on a struct's field too, which then does not make the
struct a view); a view field of a class without such a `from`, or with one that names what a view
cannot point into for the object's life (a field that is not `let`, a fixed array, a struct stored
inline: "`pair` is stored inside the object, so moving the object moves it; keep it in a block of its
own: write the field as `let pair: owns Pair`"), is refused (E2118). The three kinds of kept view, by
their word:
- `ref T` reads; any number of names may view one place.
- `ref mut T` may change, and it is alone: while its holder is still used, the place is not read or
  changed through any other name (R8 (f)).
- `sharedref mut T` may change, and several holders, and the owner, may name one object. On one thread
  any of them may change it at any time; every view into it ends at a reshape made through any of them,
  and a list in it may not change while a loop walks it, whichever holder reaches it (R8); on two
  threads it is changed only under a lock, through `Atomic`, or by a reduction from scoped tasks (R9,
  R15). It is written the same way before a value (`Watcher(target=sharedref mut players[r])`), and
  `sharedref` is always followed by `mut` (E0114): a link that only reads is `ref T`, which any number
  of names may hold already.

**R8. Views and changes, checked when compiling.**
- **(a) Positions.** A name for a list element made by indexing (`boss = enemies[0]`,
  `boss = ref mut enemies[0]`, `p = ps.get(0).unwrap()`), of a list reached through locals that own it,
  parameters, `this` and one-owner fields, is a position: the element is found again at each use. The
  list may grow while the name is used (its buffer is read again after the add); anything that may
  remove, insert, reorder, clear or replace the list ends the name (E3060 at a later use). A list
  reached through a link (a local or field holding a `shares`, `shares mut` or `sharedref mut` link)
  gives addresses, rule (b).
- **(b) Addresses.** Every other view (a kept view; a view reached through a link; a view into a
  value's own contents: a `String`'s text, an `Array` field's elements, an enum's payload; every view a
  call gives, R6) ends at any change that may reshape what it points into, made through any name that
  may reach it (E3060 at a later use). Two paths may reach the same object when they lead to values of
  one type and at least one goes through a link or a kept view, unless the compiler knows they differ
  (different types, a fresh object, inside `if a is not b:`). What may reshape is worked out from the
  code (R5): the function itself, the functions it calls, every function the program can put in a
  function value or behind a `dyn` method it calls, and, at a foreign call, the callbacks C keeps (R12).
  So a call through a function value whose stored function replaces the text of an object it owns
  (`rename = fn() owns n: n.name = String("b")`) ends a view of that text reached through a link
  (`s = p.name.trim()` under `if w exists as p:`): `s` is not used after `rename()` (E3060). A view
  parameter (a plain, `Span` or `ref mut` one), or `this`, and what its function may end or move (a
  module variable, a list it reaches) may be one place too; that is checked at each call, by what the
  callee's body does (R4, E3031). Adding to a list moves only what it stores inline (section 0): a view
  of a class object (a link, a kept view, or an address reached through a link), which lives in its own
  block, is not ended by a `push` or an `extend` of the list that owns it (with `players:
  Array[Player]` and `w = Watcher(target=sharedref mut players[0])`, `w.target.hp -= 1` after
  `players.push(Player())` is allowed); one to a value stored inline in the list (a struct element) is.
  Removing, clearing or replacing still ends every view of what it may end.
- **(c) Loops, generators included.** While a `for` loop walks a collection (a list, a map, a set or a
  string), or a generator that walks it (a `gen fn` paused in the middle of it, R6) will still be used,
  nothing changes the collection, whatever the change: nothing is added or removed, no element is
  written or replaced and no field of one changed (`xs[0] = 5`, `enemies[0].hp -= 1`), no value under a
  key replaced (`counts[k] *= 2`, `names[k] = String("bob")`), through the collection's own name or
  through any call that may change it (R5: a call through a function value, a library call given a
  function that changes it, a `spawn` whose thread changes it other than under a lock the loop holds, a
  foreign call while a callback C keeps changes it, and the end of an object whose `drop` changes it)
  (E3020, at the change: `for k in counts.keys(): counts.insert(...)` is refused at the `insert`). **The
  loop's own handle may change the item it is on**, as Rust's `iter_mut()`: `for ref mut x in xs: x = 5`
  gives `[5, 5, 5]`; `for k, ref mut v in counts: v *= 2`; `for k, ref mut v in names: v =
  String("bob")`; the message gives that line. Anything else goes after the loop or after the
  generator's last use (`retain`, `extend`, `Commands`). A counting loop `for i in 0..xs.len():`, whose
  range is fixed when it starts, does not walk `xs`: it may write its elements and add to it, but not
  remove from it (E3020). A `while` loop that tests its position against the list's current length at
  every step may add and remove: it is not a walk, and each index is checked. A call through a function
  value that may take a lock this thread holds, inside a walk under that lock, is E7018, not E3020 (R15,
  R18).
- **(d) Temporaries** (row 5). A temporary lives until the last use of every view made from it, lists
  of views included. Returning such a view, or storing it where it would outlive the temporary, is
  refused with row 5's message (E3063). A method that may change its object, called on a temporary that
  ends with the line, is refused when the call's only effect is that change (its body changes nothing
  else, through its arguments or by any other name, and nothing it gives back is used): the change would
  be lost (E2140, `enemy_at(es, 0).heal()`). A call whose result is used stays allowed
  (`line.split(" ").next()`).
- **(e) Guards** (row 4). A view made through a lock's guard (`g[0]`, `ref g[0]`, a slice of `g`) points
  into the guard and does not leave its `with` block (E3063, row 4's message); the same for `RwLock`'s
  `read()` and `write()` and for a `guarded by` field inside `with x.lock:`.
- **(f) A `ref mut` name is alone.** While a `ref mut` name (a parameter, a local, a loop, pattern or
  opened name, a kept `ref mut` field; one that holds a `MutSpan` included) is still used, its place is
  read or changed only through it: another name that may name the same place, a link of the same type
  included, is refused (E3022 to change it, E3021 to read it) unless the compiler proves the two differ
  (`if t is not a:`), or proves them the same whole variable given to one call (R4, exception 1). For a
  `ref mut` parameter and a place its function reaches by its own name (a module variable), the check is
  made at each call, by what the body does (R4, E3031), so the function itself compiles. Two
  names of elements of one list, at least one `ref mut`, whose indexes are not proved different are
  refused as R4 refuses two such arguments (verdict 98; `if i != j:` is the way out). A position is ruled by (a): growing the list that holds it (`push`) is not a use
  of the element, and a removal, reorder or clear of the list ends the position, so a later use of the
  name is E3060, not E3022, a call that reads elements and removes included; a reshape through another
  name while a `ref mut` address (not a position) is still used is E3022, at the reshape (R18). A call
  through a `ref mut` name of a stored function value uses the name only to copy the function's two
  words out, before the arguments are given (R12), so the call's arguments may reach the place it names;
  a later use of the name is checked against what the call may have done (rule (a)). `this` is not such
  a name (R5).
- **(g) A read-only name promises that what it reads does not change while it is used.** While a
  read-only view (a plain name for a part, a `ref` name, a plain parameter, a `Span` parameter, a plain
  name holding a `MutSpan`, a plain loop, pattern or opened name, a view a call gave) is still used, what
  it points into is not changed through any other name (E3021, at the change) unless the compiler
  proves the two differ; a change that may reshape it ends it instead (E3060, at its later use). A view
  whose last use is an argument of a call does not live across that call: inside it, the parameter
  makes the promise, and the call is refused when the callee's body uses that parameter after a line
  that may end or move what the argument points into (R4, E3031). Adding to the list that holds a
  read-only position is allowed (the element is found again). The refusal gives the ways out that
  compile in that program: change it after the name's last use; or name it `ref mut` and change it
  through that name (`boss = ref mut enemies[0]`, then `boss.hp = 1`), where nothing else then reads it
  meanwhile. A copy of your own (`boss = copy of enemies[0]`) is not restricted by anything. A `shares`
  owner is an owner, not a view: it sees changes made through the value's other owners that may change
  it (R9).

**R9. Several owners and shared links; nothing checked while running.** A value made shared is changed
by its owners that may change it: an owner made with `shares mut`, and the name it was first shared
from, which keeps what it could do; a `shares` owner only reads it. So after `other = shares mut items`
both `items` and `other` may change the list. That is safe as it stands: one thread runs one line at a
time; a list does not change while a loop walks it, through whichever of its names (R8 (c)); and a
`shares` value never reaches another thread (E7015), which is what `thread_shared`, changed through a
lock, is for. Any `ref mut` name of it and any holder of a `sharedref mut` link to it may change it
too. A `thread_shared` value is never changed directly by any owner: only through `Atomic`, `Mutex`,
`RwLock`, a channel, or a `guarded by` field under its lock (E7014). A scoped task that reaches a
`sharedref mut` target changes it directly only where neither another task nor the parent may touch
it before the join, or by a reduction (R15); otherwise only as a `thread_shared` value is changed
(E7007, two tasks or a task and the parent changing one place other than by a reduction). An unscoped
thread never holds one (E7015: a lent value reaching another thread). Nothing is checked while running:
no access words, no borrow flags; `RefCell`, `Ref`, `RefMut`, `borrow` and `borrow_mut` are removed.
`shares` counts are plain adds and never cross threads (E7015); `thread_shared` counts are atomic. A
class object has an 8-byte header (a 32-bit count and a 32-bit type index); an object that `weakref`
links may reach has a third 32-bit number there, its weak count (run 3's C used this 12-byte header
and found the objects' packed sizes unchanged, `RUN3/24-faithfulness.md` section 3). **A count is
checked before it rolls over**: before each +1 of an owner count (a `shares`, `shares mut` or
`thread_shared` owner made) it is compared with its maximum, and at the maximum the program stops ("too
many owners of a `Player` (4,294,967,295)"), as Rust's `Rc` and `Arc` do; one compare per added owner, no
extra memory. The weak count gets the same check (R11).

**R10. A view of an object reached through a link.** A view of an object reached through a `shares`,
`shares mut`, `thread_shared` or `weakref` link (a name opened from a link,
`if cam.target exists as p:`; a view made through a `shares mut` field, `t = ref mut ws[i].target`; a
parameter `x: shares T`, whose life is the call) must not outlive the object. It is checked when
compiling, and nothing runs for it while the program runs, except for one kind of object (point 2):
1. **No line or call in the view's life may end an object of that type**: no write to a place that
   owns one (`T`, `owns T`, `shares T`, `shares mut T`, `thread_shared T`, and a place of a function type
   or of `owns dyn I` whose value's state owns one: `h.keep = fn(): pass` ends the function stored there
   before and the node its `owns n` held), no removal from a list that owns them, no end of a local that
   owns one, and no call that may do so, worked out from its code (R5): `remove_all(ref mut players)`,
   given the list that owns the players; a call through a function value that may run
   `fn() owns players: players.clear()`; the end of an object whose `drop` may end one. After such a line
   or call the view is not used (E3060: "`remove_all(ref mut players)` (line 12) may destroy the player
   `p` reads (`if cam.target exists as p:`, line 10), and `p` is used on line 13; use `p` before line
   12, or open the link again after it: write `if cam.target exists as q:` above line 13, indent line 13
   under it and use `q` there"). A one-owner object lent read-only cannot be ended during the loan.
2. A view of a `thread_shared` object reached through a `weakref` link holds one atomic count for its
   life, as Rust's `Weak::upgrade` does: another thread may end the last owner at any moment, which no
   line of this thread can be checked against.
Across a call that may change an object of the view's type (worked out from the functions it may run,
R5), a read-only view of a `T` does not live at all (R8 (g), E3021): a walk across such a call keeps a
`weakref` link and tests it at every step (R11). Where a call fits both this and point 1 (it may change
and end the object a name opened from a link reads), the message shown is the one whose fix compiles
(R18): point 1's E3060 where using the name before the call, or opening the link again after it,
compiles (`println(p.hp)` after `reset()`, which may run `fn() owns players: players.clear()`), and
R11's E3021 for a walk, where every view of the walk would need opening again.

**R11. `weakref` links, and parent links.** `weakref T` is a link that does not keep its target alive;
its target lives on its own (a class object, or a value held `shares` or `thread_shared`), so a
`weakref` of a part, a field or an element stored inline, is refused (E2119: "a `weakref` link needs an
object that lives on its own; link to the player: `cam.target = weakref player`"); it may hold no target
(`parent: weakref Node = None`). Every `weakref` link holds one weak count on its target, with no
exception: making one (`weakref x`, or `weakref this.keep` for another link to the same target) adds
one, and its end, or giving it another target, subtracts one, as Rust's `Weak`; before each +1 the
count is compared with its maximum, and at the maximum the program stops ("too many `weakref` links to
a `Player` (4,294,967,295)"). That count is a number in the target's header that no name reads, so
making or ending a link is not a change of the target: any name may make one (`up = weakref p` from a
read-only `p`; `child.parent = weakref this` in a method that otherwise changes only `this.children`).
Reading one tests the target's header (does the object still live?) and is written as an opening (R13):
`if cam.target exists as p:`. `copy of` a link is refused (E2116): another link to the same target is
`weakref this.keep`, a copy of the target itself is `if this.keep exists as n:` then `copy of n`.
**An object is ended the moment its last owner lets it go, before its `drop` runs**, as C++'s
`weak_ptr` and Rust's `Weak` see it: a `weakref` link opened during its `drop`, or during the `drop` of
anything it owns (a child's `drop` opening `this.parent` while the parent is being ended), finds it
ended, so the `if ... exists as p:` block is skipped and the half-ended object is never read, nor
counted again. Ending the link itself (`this.parent = None`) stays allowed: it only subtracts the link's
weak count in the target's header, which that count keeps in memory.
**Parent links** are ordinary `weakref` fields the program sets: `child.parent = weakref this`, then
`this.children.push(child)`. A read of one is compiled with no header test (a plain load and a test for
no link) where the compiler proves its target lives; it proves it when both of these hold:
1. **The link follows ownership.** In the package, every line that gives the field a target,
   `x.parent = weakref o`, is followed in the same function, before `x.parent` is read, by a line that
   puts `x` into a place `o` owns alone (`o.children.push(x)`, `o.left = x`); every line that takes
   such an object out of its place without ending it is followed, before that object's link is read
   (here, or in a function it is given to, by that function's summary), by a line that gives the link a
   target the same way or clears it (`c.parent = None`); no list or field holding such objects is
   handed over, replaced or exchanged whole while it holds them; the objects have one owner (not
   `shares`, not `thread_shared`) and their class has no `drop` method of its own; and no other
   package can change the link or those places (R17). Then an object's parent owns it, so while the
   object lives its parent lives.
2. **Nothing ends such an object meanwhile.** No line between the point where the object whose link is
   read is known to live (a view of it, R8, R10) and the read can end an object of the link's target
   type (R10 point 1).
A walk up a tree with views (`cur = ref n`, `while cur.parent exists as p:`, `cur = ref p`) is then
compiled with no test per step; where condition 1 fails, every read tests.

**A walk across a call that may change nodes holds a link and tests it at every step.** When the walk
calls a function that may change nodes (`this.on_visit(p)`, where the program stores in `on_visit: fn(Node)`
a function that removes nodes from their parents' lists, and so ends them), no view of a node lives
across the call (R8 (g)): the walk with views is refused (E3021), and it is written with a `weakref`
link, which may outlive its node:

```ember
up = weakref leaf
while up exists as n:            # the node still lives: call it `n`
    if n.parent exists as p:     # its parent, if it has one
        up = weakref p           # the link now names the parent
        this.on_visit(p)         # may end `p`, or a node above it with everything under it
    else:
        break
```

The call `this.on_visit(p)` itself compiles when no function the program stores in `on_visit` uses its
node after a line that may end it; one that does is refused where it is stored (E3031, R4). **A walk up
a tree stops at the first ended parent**: if the call ends `p`, or a node above it with everything under
it (all of it ended at once, held in memory until the walk ends where the walk runs in a deferred-drop
region, section 0), the link's target is ended and its block is kept by the link's weak count, so the
next `while up exists as n:` finds it ended and the walk stops; it never visits an ended node, and
nothing stops the program. **What it costs**: per step, one test of the node's header
(`while up exists as n:`) and one move of the link's weak count from the node to its parent
(`up = weakref p`: a subtract and an add); the read of `n.parent`, between the test and the call, needs
no header test where condition 1 holds (condition 2 holds there). **Where compile-time proof removes
part of it**: when the node's class has no `drop` method of its own, the walk is placed in a
deferred-drop region (section 0), which keeps every block it sets aside until it ends (one add and one
subtract of a per-thread counter per region, nothing per step); a link that lives only inside the region
and is given only objects that live at that moment (`up = weakref p` from the view `p`) then holds no
weak count, so each step pays only the header test (the first link of each walk, `up = weakref leaf`,
keeps its count). A walk whose visit function changes no node (every function the program stores in
`on_visit` only reads its node) keeps its views and pays nothing per step. The cycle warning (L3001) is
given for a type whose `shares` fields can form a cycle of owners, and names the link to make
`weakref`.

**R12. Function values.**
- **A function type says its parameters, its result, and whether any thread may run its functions;
  what they change is worked out from the program.** `fn()`, `fn(Node)`, `fn(owns String) -> int`,
  `thread_shared fn()` and `owns dyn I` say nothing about what their functions or methods change.
  Wherever a function value is called, the compiler counts the changes of every function the program can
  put in that place: every lambda and named function stored into that field, list, parameter or local's
  type anywhere in the program, a library included (R5). So a library's `Button` holding a program's
  handler that pushes to `LOG` makes `b.click()` a change of `LOG` in that program (inside a loop over
  `LOG`, E3020), and calling what a list holds whose functions change nothing is allowed inside such a
  loop. A lambda writes nothing about what it changes: `b.on_click(fn(): score += 1)` changes `game`'s
  real `score` (row 6); `enemies.retain(fn(e) => keep(e, ref mut removed))` adds to `update`'s
  `removed`. A long function type is named once with `type` (`type Handler = fn()`, then
  `handlers: Array[Handler] = []` and `fn on_click(owns f: Handler):`;
  `type DeathHandler = fn(ref mut Array[Enemy], int)`). A named function is a value of its type
  (`fn log_click():` is a `fn()`), and a local function captures as a lambda does, so a long lambda may
  be written inside the call or as a named local function passed by its name (verdict 51). A lambda's
  parameter says what it receives, matching the function type it is given to: where the type hands the
  value over (`fn(owns String)`), the lambda writes `fn(owns s) => println(s.len())`, and `fn(s)` there
  is refused (E2020: "`each_name` hands each name over (`fn(owns String)`), so this function receives it
  as its own: write `fn(owns s) => println(s.len())`").
- **Its own state.** What a function captured with a word (`copy of`, `owns`, `shares`, `shares mut`,
  `thread_shared`, `weakref`), and what that owns alone, is its own state; a capture with no word is the
  real variable, which it only borrows (below). A change to its own state, and the end of what its state
  owns, are worked out from its body like every change (R5): `reset = fn() owns players: players.clear()`
  may end players, so a view of a player opened from a link is not used after `reset()` (R10, E3060);
  `rename = fn() owns n: n.name = String("b")` replaces the node's text, so a view into that text
  reached through a link is not used after `rename()` (R8 (b), E3060). **Replacing or ending a function
  value ends what its state owns**, as ending an object runs its `drop` (R5): `h.keep = fn(): pass` ends
  the function stored there before and the node its `owns n` held, so a view of that node is not used
  after that line (R10, E3060), and that node's `drop` counts as called there (inside a walk of what it
  changes, E3020). An `owns dyn I` value replaced or ended does the same with what it owns.
- **A function type says whether any thread may run its functions: `thread_shared fn()`**, as Rust's
  `Send` does. A list, field or parameter whose functions may run on any thread says so in its type
  (`handlers: Array[thread_shared fn()] guarded by lock = []`,
  `fn on_event(owns f: thread_shared fn()):`). A function of such a type may hold `thread_shared` values
  or copies, never a `shares` or `shares mut` value or a `weakref` link of one (E7015):
  `bus.on_event(fn() shares mut count: count.n += 1)` is refused, since another thread running it would
  change `count`'s one-thread count and value. A class whose objects are made `thread_shared` holds
  functions only in such types: `bus = thread_shared Bus()`, for a `class Bus:` with
  `handlers: Array[fn()] = []`, is refused at that line (E7015: "`Bus.handlers` holds plain `fn()`
  values, and a `thread_shared Bus` may run them on any thread; write
  `handlers: Array[thread_shared fn()] = []` in `Bus`"). `thread.spawn` takes such a function (R15).
  A class object that a `weakref` link on this thread points at is not handed whole to another thread
  either: `w = weakref names`, then `thread.spawn(fn() owns names: names.items.len())`, is refused
  (E7015), since this thread could open `w` while the other thread ends `names` (a race on the object's
  one-thread header, and a read of freed contents); a copy (`fn() copy of names:`) is allowed.
- **A library function that calls a function right away takes it as a template parameter**:
  `[F: fn(Item)]` above `fn for_each(owns f: F) owns this:`, and `retain`, `inspect` and `sort_by` the
  same way (`[F: fn(T) -> bool]` above `pub fn retain(owns keep: F):`). `F` names the exact function
  given, so each lambda gets its own copy and the call is direct and may be inlined (C++'s
  `template <typename F> void for_each(F f)`, Rust's `F: FnMut`); the shared type written on the
  parameter, `owns f: fn(Item)` (C++'s `std::function`), is legal too and is what keeps functions of
  different kinds together (`handlers: Array[fn()]`), and a program's own functions may write either. The
  call counts as making what the function given changes: `scores.for_each(fn(s): LOG.push(copy of
  s.name))` changes `LOG`, so inside a loop over `LOG` it is refused (E3020). An iterator adapter keeps
  the function it is given and runs it at each step (`[F: fn(Item)]` above `fn inspect(owns f: F) owns
  this -> Inspect[Self, F]:`), so whatever drives the adapter (`for x in it:`, `it.sum()`) makes its
  changes; driving an `owns dyn Iterator[Item = int]` makes the changes of every adapter the program can
  put there. A generic body's calls of its type parameters' methods make the changes those methods make
  for the types it is given, worked out for each instance as the body is compiled for each.
- **A call through a function value makes the changes of the functions it may run.** R8 then applies: no
  `for` walk of a list they reach, no read-only view of what they change and no `ref mut` name of such a
  place lives across the call (a view whose last use is an argument of the call does not live across
  it, R8 (g)). For R4's overlap rule a function value given to a call counts as an argument that changes
  every place its functions change and every real variable they capture, so `LOG.retain(keep)`, where
  `keep` pushes a note to `LOG`, is refused (E3022). For the same reason a function value is not stored in
  a place that a real variable it changes owns, all the way down: `b.on_click(fn(): b.handlers.clear())`
  and `t.on_visit = fn(n: Node): t.root.children.clear()` are refused (E3022), since the function would
  change its own holder while it runs, as Rust refuses.
- **A call that runs a function on its own counts as making its changes, now or later.** Handing a
  function to something that will run it by itself counts as making everything the function changes:
  `thread.spawn` and `scope.spawn` (R15), and a foreign function's parameter whose contract says
  `callback(retained)` or `callback(once)` (C keeps the function and runs it during later foreign calls; a
  `callback(borrowed)` parameter runs it during the call, as `for_each` does). So a `spawn` inside a loop
  over what its thread changes is refused (E3020: start it after the loop), except where the thread makes
  that change under a lock the loop holds, as Rust allows it: the new thread waits for the lock until the
  loop ends; waiting for that thread inside the lock (`t.join()` in the `with` block) is refused (E7018),
  since each would wait for the other. The thread rules (R15: locks, `guarded by`, E7012) still keep
  threads apart. **Storing a function value is not a change**: putting it in a field, a list, a local or
  an iterator adapter (`b.on_click(f)`; `xs.iter().inspect(f)`) changes nothing the function changes; the
  call that later runs it does (`b.click()`; the adapter's `next`). The line falls at who runs it: a call
  that runs the function, itself or by handing it to a thread or to C, counts; a call that keeps it for
  its caller to run does not.
- **A foreign call may run the callbacks C keeps.** The compiler cannot see inside C, so a call of a
  foreign function counts as possibly running every callback C keeps (those the program hands over
  through a `callback(retained)` or `callback(once)` parameter, on the threads their contracts allow),
  and makes every change they make, for R8 and for every function that calls it. A foreign function that
  never calls back says so on its declaration, `@ffi(no_callbacks) safe fn sqrt(x: f64) -> f64`: a fact
  the programmer vouches for, listed by `ember tcb` with the other asserted facts (`[TCB-1]`); on a
  function whose own contract takes a callback it is refused (E5069). So with
  `register_cb(fn() owns game: game.add_point())` kept, `tick`, which calls `poll_events()`, changes what
  the callback changes, as does every function that calls `tick`, and `for name in game.names:
  poll_events()` is refused (E3020: "call `poll_events()` before or after the loop"); a missing mark
  only refuses such a loop, and the message names the mark. With no kept callback, or one that changes
  nothing (`fn(): println("event")`), a foreign call changes nothing.
- **The real variables a lambda captures** (a capture not named in its list is the real variable, row
  6) are checked in the function that writes the lambda, which sees every place the function value goes:
  the value may not outlive them (E3026). There a place that holds the function value counts as reaching
  every real variable it captures (R4, R8), and a call that may run it as a change of each one it
  changes. Without a word a function only borrows: a real variable stays at its own address, and the
  function value must provably end before it (it is kept only in locals, local lists or objects that
  never leave the function, and every function it is handed to keeps it no longer, by that function's
  summary), or it is refused (E3026). A name captured with `ref` (`fn() ref t:`) is a read-only view of
  what the name views at that moment, with the same rule (R15).
- **A function that outlives the function that made it takes what it keeps with `owns`.**
  `return fn(): n += 1` in `make_counter`, whose local `n` ends when it returns, is refused (E3026: "the
  returned function changes `n`, which ends when `make_counter` returns; let the function own it: write
  `return fn() owns n: n += 1` in place of this line"). With `owns`, `n` is handed over to the function
  and is its own state, so the returned type is a plain `fn()`: `fn make_counter() -> fn():`.
- A named capture (`copy of`, `owns`, `shares`, `shares mut`, `thread_shared`, `weakref`) is the
  function value's own state.
- A stored function value is two words, `{code, state}`. A call through a list element or a field
  copies the two words out before the arguments are given and keeps no pointer into the list or field.
  A body that may end its own state (it removes itself) and uses that state afterwards holds one count
  on it across that call. A C callback that ends its own `Retained` token while it runs (with
  `callback = fn() owns my_game:` whose body is `my_game.token = None` then `my_game.add_point()`) is the
  same case: the token's end unregisters the callback at once, so C never runs it again, but the block
  that holds its state, where `my_game` lives, is ended and freed only when the callback returns, so the
  rest of its body runs as written (R15).
- A call through a stored function value whose functions may change their own state is a change of the
  place that holds it: the holder is reached through a name that may change it (`for ref mut h in
  this.handlers: h()`; E3023 for a plain `h`), and such a function value is refused where it is given to
  a parameter that only reads (`run(cb)` with `fn run(f: fn()):`, E3023; write `ref mut f: fn()` and
  `run(ref mut cb)`). The holder does not become an argument of the call: R4 sees only the call's
  arguments.
- `once fn()` is a function a call uses up; a function value that gives a capture away is one. A
  parameter that takes one holds it, `owns f: once fn()`. A plain `f: once fn()`, and a function value
  that gives a capture away passed where it may run twice, are refused (E3013).
- A function value may not outlive a place it uses and does not own (E3026): it names a word for that
  place (`fn() shares mut count`, `fn() owns xs`), or it runs inside a scope.

**R13. Opening an `Option` or a `weakref` link: `exists as`.** `if cam.target exists as p:` runs its
block when `cam.target` holds something (an `Option` holds a value; a `weakref` link's target still
lives) and names that thing `p` inside the block; `else:` runs when it holds nothing;
`while cur.parent exists as p:` repeats while it holds something. `exists` says what the line checks,
and `as p` gives the name, as in `import x as y`. The name after `exists as` is declared there with its
words, as a pattern name is (R2): plain `p` is a read-only view; `if ui exists as ref mut me:` may
change what the link points at; `if scores.get(name) exists as copy of v:` is a copy of your own, which
may be changed (`v += 1`; the map is untouched), and the same may be written in two lines,
`if scores.get(name) exists as v:` then `mine = copy of v`; a name `exists as` makes without
`copy of` or `ref mut` never changes (E3023); `if pair exists as (ref mut a, ref b):` opens an `Option`
of a pair. The left side must be able to hold nothing, an `Option` or a `weakref` link; otherwise the
block would always run, which is refused (E2036). The name is new: a name already in use is refused
(E2106). `Some(...)` is written to make a value (`best = Some(5)`) and in a `match` branch, not in an
`if` or `while` header (E0115). A name opened from a link is a view of an object reached through a link,
checked by R10: nothing that may end its object runs while it is used. A test of another variant is
written as a pattern (`if Ok(v) = r:`, `if State.Walking(ref mut path) = n.state:`) or with `match`;
a test of a value's kind is written with `is` (R14).

**R14. Interface values, kind tests and casts.** `dyn I` as a parameter's type is a read-only view
(`ref mut v: dyn I` may change it). A value of an interface type (`dyn I`, `owns dyn I`,
`shares dyn I`) or of a base class is tested for its kind with `is`:
- **`if s is Circle as c:`** runs its block when `s` is a `Circle` and names it `c` there. The name is
  declared with its words, as a name opened with `exists as` is (R13): plain `c` is a read-only view,
  and since a value tested in its place stays a place, `if this.pools[k] is Pool[T] as ref mut pool:`
  names a view of the registry's pool that may change it (where the function may change the place). The
  name is new (E2106). `if s is Circle:` tests without a name. Other kinds are tested with
  `elif s is Square as q:` and `else:`, which is how a choice among kinds is written: there is no
  "maybe a `Circle`" value to `match` on. A test that always passes (`s` is already a `Circle`) is
  refused (E2036), and so is a test that can never pass, of a type that does not implement the
  interface or derive from the base class (E2037: "`s` is a `Shape`, which can never be a `Path`; remove
  the test: the `if` line and its block").
- **`cast s as Circle`** gives the `Circle` and stops the program if `s` is not one (section 2 gives the
  text). A cast of a place is a place: the name bound from it is read-only unless written `ref mut`
  (`pool = ref mut cast this.pools[k] as Pool[T]`), and `ref mut` taken through a plain one
  (`return ref mut pool.dense[pool.sparse[e]]`) is refused (E3027).
- `is` before a type name means "is of this kind", as in Swift, Kotlin and C#; before a value it means
  "the same object" (`a is b`, `x is None`). `as` has its two uses, naming (`import x as y`,
  `exists as p`, `is Circle as c`) and converting (`x as f32`, `cast s as Circle`). Today's
  `cast s if Circle`, and `cast s when Circle`, are not Ember (E1010, with the line to write:
  `if s is Circle as c:`).
Each test or cast is one type-id compare. A generic method whose type parameter is only the type of a
`ref` or `ref mut` parameter gets one `dyn` entry. A non-`pub` function's bounds are worked out from
its body (a method its bounds' interfaces do not declare is E2072); a `pub` one writes them (E2040: one
message, on the first line of the body that needs a bound the function does not write, giving every
bound the body needs).

**R15. Threads.**
- **Starting a thread counts as making what its function changes** (R5, R12). A `spawn` inside a loop
  over what the thread changes under a lock the loop holds is allowed, the thread waiting for the lock;
  joining it inside that lock is refused (E7018); a scoped task started inside a walk of what it changes
  is refused as any call that changes the walked list is (E3020: start it after the loop). The rules
  below keep the threads apart from each other.
- **`thread.spawn` takes a function any thread may run**: `[R]` above `pub fn spawn(owns f:
  thread_shared fn() -> R) -> JoinHandle[R]:`, so a function holding a one-thread value is refused by
  R12's rule (E7015); calls are written as any call (`h = thread.spawn(fn() copy of r: worker(r))`).
  `scope.spawn` takes a plain `fn()` (`[R]` above `fn spawn(owns f: fn() -> R) ->
  ScopedJoinHandle[R]:`): its tasks may use the caller's own variables, since they all end with the
  `with scope` block.
- Scoped tasks need no capture words but for a loop's own variable (below): they use the real variables.
  Every task's effects, worked out from its body (R5), are checked against the other tasks and the parent
  until the join (E7007 for two of them changing one place other than by a reduction, under a lock or
  through an `Atomic`). Reductions (`total += e`, `-=`, `*=`, `|=`, `&=`, `^=`, `max`, `min`, `push`,
  `extend`, `add`, `m[k] op= e`), on a local or on a field of an object the tasks reach
  (`scope.spawn(fn() copy of x: stats.total += x)`), a field reached through two `sharedref mut` links
  to one object included, get one private accumulator per task, combined at the join in spawn order;
  nothing reads the place before the join. A whole-number total is always the one adding one by one
  gives (`a.target.hp -= 1` and `b.target.hp -= 2` from two tasks, `a` and `b` linking one player whose
  `hp` is 10, leave 7 every time); a decimal total may differ from it in the last digit
  (`for part in [[0.1], [0.2, 0.3]]:` with `scope.spawn(fn() copy of part: for x in part: total += x)`
  gives `0.6`, where adding one by one gives `0.6000000000000001`) and is the same on every run, as
  Rust's Rayon and C's OpenMP give. A plain assignment from a task (`a.target.hp = 5`), a task that reads
  the current value (`if stats.total < 10: stats.total += x`), and a `thread.spawn` thread still need a
  lock (E7007); a `thread.spawn` thread that only adds may use an `Atomic`.
- **The loop's own variable**, which changes every round, and a local of one round, which ends with it,
  are given to a task or a thread with a word. `copy of t` gives it its own copy (for a `String`, one
  allocation and copy per task): `thread.spawn(fn() copy of r: worker(r))`. For a scoped task, `ref t`
  lets it look at the element `t` is on in that round, with no copy, as Rust's scoped threads borrow
  `&t`: the walked list cannot change until every task has ended at the end of the `with scope` block
  (`scope.spawn(fn() ref t: total += t.split_whitespace().count())`). `ref t` is refused where the
  function may still run after the element changes or ends: a `thread.spawn` thread may outlive the loop
  (E3026: "the thread may still run after the loop has moved on and `texts` is gone; give it its own
  copy: `thread.spawn(fn() copy of t: ...)`"). The loop's name used with no word is refused: E3026 for a
  scoped task, whose message gives both words; E7004 for a `thread.spawn` thread, whose message gives
  `copy of t`. A module variable such as `RESULTS` needs no word.
- An unscoped thread, and a C callback that C keeps, say in the capture list what they hold (`owns`,
  `thread_shared`, `copy of`; E7004 for a name with no word); `shares`, `shares mut` and `weakref` only
  where the function stays on this thread (`threads(creator)` or `threads(main)`, checked by the
  trampoline in every profile). A function type whose functions any thread may run says
  `thread_shared fn()` (R12), and such a function holds no one-thread value (E7015). A callback that
  ends its own token while it runs is unregistered at once and its block freed when it returns (R12).
- `guarded by lock` after a field's (or a `static mut`'s) type names the `Lock` that must be held for
  every read and change outside `init` (E7013). An object whose changes go only through its lock may
  reach one call through any number of names (R4).
- **A call that may take a lock this thread holds is refused when compiling.** A direct call whose body
  the compiler sees enters the callee past the lock it already holds (static re-entry). A call through a
  function value, or through a `dyn` or virtual method, that may run a function taking a lock of the
  kind this thread holds is refused, with no loop needed (E7018): with `note` holding
  `fn() thread_shared log: log.push("a")`, whose `push` takes a `Log`'s lock, `note()` inside
  `with log.lock:` is refused ("`note()` may run the function stored on line 12, which takes a `Log`'s
  lock (`log.push`, under `lock`), and this thread holds `log`'s lock, so taking it again would wait for
  ever; call `note()` after the `with log.lock:` block: move line 16 below the block"). The compiler
  cannot tell which `Log` a stored function holds, so a function that takes another `Log`'s lock is
  refused too; the way out moves one line.
- What may go to another thread is inferred from the words: `ThreadShared` and `Send` are inferred and
  written only in bounds; `Send` is declared only by an `unsafe extend` author. A one-thread owner or a
  lent value reaching another thread is E7015; a thread-bound value sent away is E7008.
- `static mut NAME: T = e` is a module variable that may change (`static` alone never changes); it is
  reached only by the main thread and its scoped tasks unless it is `guarded by` a `static` lock
  (E7012).
- A `@parallel` loop splits a range or a list walked by element into chunks (E7016 for anything else;
  E7010 where chunks may write one place).
- A generator's `for` over a list walks by position across `yield`, so it holds no address into its
  own frame (E2221 for a view into its frame held across `yield`); a generator paused in a list is a
  walk of it (R8 (c)). `async` stays reserved (E0005). C callbacks go through the checked trampoline: C
  holds a tagged number, never an address, with the callback's generation in its top 16 bits and its
  slot in its low 48 (AN-78's callback carries `0x4080`, 16512, in the top 16 bits); a call after the
  token ended, a second call of a `callback(once)` function, and a call on a thread the callback's
  contract does not allow stop the program (section 2). Every foreign call counts as possibly running the
  callbacks C keeps, unless the foreign function's declaration says `@ffi(no_callbacks)` (R12).

**R16. `unsafe`.** One word with one meaning by position: before a declaration it says "whoever uses
this must vouch", with `@safety` saying what they must make true (L3015 when missing, on every
`unsafe fn`); before a block or an implementation it says "this code vouches", with a `# SAFETY:` note
saying why (W3012 when missing; W3013 for a `# SAFETY:` note on an `unsafe fn` line). Inside an
`unsafe fn` body every unsafe operation sits in its own `unsafe:` block (E3100). `@no_alias_checks`
forbids the one run-time alias check that is left, `assert_disjoint` (E4030). No
switch turns safety checks off. Anything that can break a safety promise is possible only inside an
`unsafe` block. **Giving an object to C**: a C function declared as keeping an object
(`@ffi(param(widget, owned, one))` with `owns widget: owns Widget`) is given one by an ordinary
hand-over, `widget_take(first)`: the name ends and Ember frees nothing, with no word. For a C function
whose declaration cannot say so (a plain pointer, `fn set_user_data(p: *mut Widget)`, or a function with
`...`), `second.into_raw()` gives the object's address and stops Ember freeing it, as C++'s
`unique_ptr::release()` does; the compiler makes that step itself, so nothing else can skip a value's
clean-up. Outside `unsafe:` it is refused (E3100: "`into_raw()` stops Ember from freeing the widget, so a
mistake leaks it or frees it twice; write it inside `unsafe:`"). The `Retained` token keeps an object on
Ember's side while C holds a callback; it gives no object away (R12).

**R17. Inferred facts at a package's edge.** A fact the compiler works out from a body never changes
what a line does; it only decides whether a line compiles, or removes a test that would always pass.
Inside a package such facts are used freely. Across a package's edge, where a word exists to write the
fact, only the written word counts: a `pub` function's bounds are written. Every other fact is recorded
in the package's interface file (as today's effect sets, `[EFF-4]`), and a change to it is reported as an
interface change: what each function may change, read and end, with the chain of calls a message names
(R5); the "only adds" summary of a `ref mut` list parameter (the callee only adds at the end, `push` or
`extend`, and reads or writes no element already in the list, so a position of that list stays usable
across the call, R8 (a), (f)); the effects of scoped tasks; whether a function sets the parent link of an
object it is given before reading it (R11); which of a function's view parameters, `this` included, it
uses after a line that may end or move what they view, and what it reaches by other names while a
`ref mut` parameter is used (R4: so reordering two lines inside a library function, a use of `s` moved
below `LOG.clear()`, is reported as a change to its interface); the locks it may take (R15); and the
function values a package stores into a function type and the callbacks it hands to C, which a program
that uses the package counts wherever such a value is called and at every foreign call (R12). A library
update that changes such a fact can make a caller's line stop compiling; the message names the chain. A
parent link proves itself only when no other package can change it or the places that own its holders.
What a generic function changes through its type parameters' methods is worked out for each instance
(R12), whose body the instantiating package compiles.

**R18. Messages.** Every refusal names what was refused in the program's own names (`enemies[0]`, not
`*enemies[0]`; `line.to_lower()`, not `_5`; `int`, not `i64`; `fn()`, not `Callable0`; the caller's
variable at a call, not the callee's parameter), says why in one line, and gives the exact line to
write. One mistake gives one message, at the program's line, on the name that caused it. The line a
message gives compiles under these rules in that program: its fix never runs into another error, and
where it replaces several lines the message names every one it replaces ("instead of `i = 0`, the
`while` line and `i += 1`, write `for i in 0..idx.len():`"); a line it moves is named with where it goes
("move line 17, `b.click()`, below the loop"). A message about a change a call makes names the chain of
calls that makes it ("`tick()` adds to `LOG` (through `note`, line 4) while this loop walks `LOG`"); no
message asks for a word on a first line. Where two errors fit one line, the one whose fix makes the
program compile is shown: a call through a function value that may take a lock this thread holds,
inside a walk under that lock, is E7018 ("call `note()` after the `with log.lock:` block"), not E3020,
whose fix (after the loop, still inside the lock) would still wait for ever; a scoped task started inside
a walk of what it changes is E3020 ("start it after the loop"), not E7007, whose fix (a lock or an
`Atomic`) leaves the walk refused; and where both fixes compile, the rule that names the case decides (a
position used after a removal is E3060 by R8 (a), not E3022; a view used after a call whose callee ends
what it points into, where E3021 and E3031 both fit, is E3031). A message says only what the compiler
knows.

## 2. What still runs while the program runs, compared with C

Every number is from run 3, with its source; "PROVISIONAL" marks numbers measured while other agents
ran, as run 3 marked them. Nothing in this table was measured again in this run.

| what runs | when | cost against C, and the source |
|---|---|---|
| bounds checks | indexing, as today | as today (not this rework's) |
| two compares (`assert_disjoint`) | two spans of one list given to one call, one changeable (R4), where the program writes it | the call is then compiled with noalias: 1.00x to 1.07x of plain C, against 1.53x for the same call compiled without it (`RUN3/combined-v4.md` line 534, AN-14) |
| a second pass of a loop, then `retain` | only where AN-04's fusion conditions fail | 1.71x of one-pass C as written, 1.00x fused; safe Rust's idiom 1.77x (`RUN3/combined-v4.md` line 535, PROVISIONAL) |
| one queued record per `Commands` change | ECS systems that destroy, create or add | applied after the system, as Bevy (AN-06) |
| a count +1 / -1, and before each +1 one compare with the count's maximum | a `shares` or `shares mut` owner made or ended; atomic for `thread_shared` (R9) | Rust's `Rc` / `Arc`, which make the same compare; re-pointed shared targets 1.13x to 1.25x of plain C, safe Rust's `Rc` 1.48x to 1.57x (`RUN3/combined-v4.md` line 537, AN-52 (a), PROVISIONAL; measured without the compare, which adds one compare and a branch never taken per added owner, not measured) |
| nothing | a `sharedref mut` link (R7) | 1.00x to 1.06x of plain C and the same memory, measured in run 3 as kept `ref mut` views that name one player (`RUN3/combined-v4.md` line 7298, AN-52 (b), PROVISIONAL) |
| nothing | a view of an object reached through a link, across which no line or call may end its object, checked when compiling (R10 point 1) | the C as written: no count and no region around the view |
| one atomic count for the view's life | a view of a `thread_shared` object reached through a `weakref` link (R10 point 2) | Rust's `Weak::upgrade` and the drop of the `Arc` it gives; not measured |
| one add and one subtract of the target's weak count, and before each add one compare with its maximum | every `weakref` link made (`weakref x`, `weakref this.keep`), given another target or ended (R11), parent links included | Rust's `Weak::clone` and its drop. Parent links pay it too: one add when `child.parent = weakref this` runs, one subtract when the link changes or the child ends. Not measured: run 3's tree workloads kept parent links with no count (`set by`) |
| one test of the target's header | each read of a `weakref` link, except a parent-link read proved (R11); during the target's `drop` the object is already ended, so the same test finds it gone, with nothing more run | Rust's `Weak` on churn 0.919 and 0.935 of plain C (`RUN3/combined-v4.md` line 539, AN-63) |
| a load and a test for no link, no header test | each parent-link read proved (R11) | W2, the parent walk: 1.176x of plain C with no header test per step (`diag_w2_hop_proved`), against **1.579x where the proof fails** and every step tests (`model_w2`), and safe Rust's 1.603x (`rust_w2`) (`RUN3/26-timer.md` lines 20 to 22). Both model numbers were measured with uncounted parent links; the weak count above comes on top, unmeasured |
| one test of the node's header and one move of the link's weak count (a subtract and an add), per step | a walk up a tree across a call that may change nodes (R11) | **not measured.** The nearest numbers: a header test on every step, 1.579x of plain C (`model_w2`, uncounted links), and safe Rust's walk, which upgrades a `Weak` on every step (a count pair), 1.603x (`rust_w2`) (`RUN3/26-timer.md` lines 20 to 22). Where the node's class has no `drop` method the walk runs in a deferred-drop region and the count moves go, leaving the header test (R11); run 3 measured such a region, around a walk with views, at 0.944 of plain C against 1.283 for a count pair per step (`RUN3/combined-v4.md` line 538, AN-62, PROVISIONAL, spread 29% to 37%). A walk whose visit function changes no node pays nothing per step |
| one flag test per call, one store per change of a field the method's body reads | a `@cached` method | the hand-written C cache (`RUN3/combined-v4.md` line 542, AN-21) |
| nothing / nothing beyond the function value's own block | a captured real variable, at its own address / a variable a function keeps with `owns`, in its own state (R12) | W4, callbacks that remove themselves: 1.091x of plain C with the variable in the function value's own block, safe Rust 1.358x (`RUN3/26-timer.md` lines 29 and 32: `rust_w4`, `model_w4_state_block`); run 3's box per captured variable, 1.255x (line 31, `model_w4`), is no longer made |
| two words copied out of the list | each call of a stored function value | dispatch 1.045 and 1.037 of plain C, safe Rust 1.490 and 1.472 (`RUN3/combined-v4.md` line 544, AN-60) |
| one count on a handler's own state | a handler that removes itself and then uses its state; a C callback that ends its own token while it runs, whose block is freed when it returns (R12) | a rare path (AN-60); the callback's case is the trampoline's own bookkeeping, not measured |
| one type-id compare | each kind test (`if s is Circle as c:`) and each `cast` | 1.223 to 1.244 of plain C, safe Rust's `downcast_mut` 1.571 to 1.635 (`RUN3/combined-v4.md` line 546, AN-22, PROVISIONAL) |
| a lock | where the program takes one (`with x.lock:`) | the lock C takes (AN-74) |
| nothing beyond C's per-thread counter | scoped tasks and reductions, whole-number and decimal | 1.00 against an atomic add per word 10.22x (`RUN3/combined-v4.md` line 548, AN-71, PROVISIONAL) |
| the checked trampoline, with its thread check for a `threads(creator)` callback in every profile, and its used-up mark for a `callback(once)` one | each C callback | 1.02x to 1.05x, today's 1.47x (`RUN3/combined-v4.md` line 549, AN-78); today's thread check runs only in the debug profile (`tests/conformance/FFI-33b`), so the release profile adds one compare of the thread per such callback, not measured |
| the generator protocol | each `yield` | position re-read 1.18 of a plain C loop (`RUN3/combined-v4.md` line 550, AN-77) |
| a copy, an allocation | where a line says `copy of`, or `owns` / `shares` puts a value in its own block | Rust's `.clone()`, `Box::new`, `Rc::new` |
| the memory manager's own work | each allocation and free of a value with `using A` | what `A` does (an arena: a pointer bump, nothing freed one by one); not measured |
| one word in the value: its instance's address | each value whose memory manager may have several instances (R3) | Rust's `Box<T, &A>` and `Vec<T, &A>` keep the same word; none for `Global` or `System`; not measured |
| nothing | what every function changes, reads and ends, worked out from its code and the whole program's function values, overrides and kept callbacks (the chain of calls, a `drop`'s changes, the end of class objects and of function values' state), what a thread or a kept callback changes, `thread_shared fn()`, `@ffi(no_callbacks)`, the arena checks, which view parameters a function uses after a line that may end or move what they view (R3, R4, R5, R12) | all checked when compiling; a library function given a function is one instance per function, as Rust's generic `F: FnMut` (R12); a call that names one whole variable twice is compiled against its own copy of the function (R4), one more copy of its code and nothing while running |

The checks above that stop the program say this when they do, in the names of the program that stopped
(the example in brackets):

| check | the stop's text |
|---|---|
| `assert_disjoint` (R4) | "`assert_disjoint(d, s)` on line 11: the two spans overlap (`dst_name` and `src_name` name one track)" |
| a count's compare with its maximum (R9) | "too many owners of a `Player` (4,294,967,295)" |
| a weak count's compare with its maximum (R11) | "too many `weakref` links to a `Player` (4,294,967,295)" |
| the type-id compare of `cast s as Circle` (R14) | "`cast s as Circle` on line 9: `s` is a `Square`, not a `Circle`" |
| the checked trampoline: a call after the token ended (R12, R15) | "C called the callback registered on line 16 after its token ended; it was unregistered" |
| the checked trampoline: a `callback(once)` function called a second time (R12, R15) | "C called the callback registered on line 29 a second time; it was registered `callback(once)`, so it may run only once" |
| the checked trampoline: a `threads(creator)` callback on another thread, checked in every profile (R15) | "C called the callback registered on line 16 on another thread, but its contract says `threads(creator)`" |
| bounds checks | today's text ("index 3 is out of bounds for a length of 3") |

The two count limits are tested with a real counter that is easy to fill: those two test programs are
compiled with an 8-bit count in each object (the header `#$ count_bits: 8`), stored and compared as the
real one is, so 255 owners or links reach the limit and the next stops the program ("too many owners of
a `Player` (255)", "too many `weakref` links to a `Player` (255)"). Every other test and every real
program keeps the 32-bit count, which one more test checks (its limit, 4,294,967,295). The language has
no test-only shortcut.

Gone, compared with today's Ember: the access words checked on class fields (`[EXC-19]`), `RefCell`'s
borrow flag, the count pair per loop step on views of shared objects (AN-51), `@sync`'s frozen fields,
the 24-byte class header (now 8 bytes), and today's 1.47x trampoline. Gone, compared with run 3: the
two-compare overlap test before a call that passes a `MutSpan` of elements that own memory (now a
compile-time rule, R4); the deferred-drop region and the count pair that kept an object alive under a
view of it across a call that may end it (now refused when compiling, R10); the box per captured
variable (a function that outlives its maker owns what it keeps, `owns`, R12).

## 3. What stays refused

Each refusal with its code and its message, in the names of an example program (the answer it comes
from in brackets, where there is one). Every message follows R18.

| refused | code | message |
|---|---|---|
| using a name after it was handed over (row 9), a number handed over with `owns` included, and a part handed over on its own, or its owner used whole, before the part has a new value (R2, R3, R4) | E3040 | "`items` was handed over to `snapshot` on line 3 and is used on line 4; to keep both, write `snapshot = copy of items` (a copy of your own) or `snapshot = shares items` (two owners of one list)"; "`count` was handed over to the function on line 3 (`fn() owns count`) and is used on line 5; to keep your own, write `fn() copy of count` in place of `fn() owns count` on line 3"; "`p.name` was handed over to `names` on line 4; give it a new value first, or keep a copy: `names.push(copy of p.name)`" |
| a change through a name that only reads (row 8; AN-20) | E3023 | "`path.pop()` changes the list in `n.state`, but `path` only reads it; to change that list through `path`, write `State.Walking(ref mut path):`"; "`tick` sets `s.frame`, but `s` only reads the caller's clock; write `fn tick(ref mut s: Clock):` and call `tick(ref mut clock)`" |
| `=` on a parameter, a loop, pattern or opened name (one opened `exists as copy of` excepted); another ref given to a `const ref mut` name; a read-only `ref` given to a `ref mut` name (R2) | E3023 | "`line` is the text the caller passed and stays that text for the whole call; for a view of your own write `rest = line.trim()`"; "`n` is the number the caller passed; for a number of your own write `count = n` and change `count`"; "`r` is `const ref mut a`, so it always refers to `a`; to refer to `b` use a new name, `rb = ref mut b`, or make `r` with plain `ref mut`: `r = ref mut a`"; "`r` may change what it refers to, and `ref b` only reads `b`; to point `r` at `b` write `r = ref mut b`, or for a read-only name `rb = ref b`" |
| a method that may change its object called through a read-only name (R5) | E3023 | "`heal` changes the player (`this.hp += 10`, its line 3), and `p` only reads it; write `for ref mut p in players:` in place of line 7" |
| an owner made from a read-only owner that may change the value (R3) | E3023 | "`g2 = shares mut game` would change the `Game` through `game`, which only reads it (`game = shares Game()`, line 1), and an owner made from it only reads too; write `g2 = shares game`, or, for two owners that both change the game, write `game = shares mut Game()` instead of line 1" |
| an owner that may change made from a plain parameter of a `shares mut` type (R4; AN-78) | E3023 | "`game` only reads the `Game` that `main` lends to `setup` (`shares mut Game` says what `main`'s game is, not what `setup` may do with it), so an owner made from it only reads too; to hand on an owner that may change the game, take one: write `fn setup(owns game: shares mut Game) -> Retained[ForeignCallback]:` in place of line 8, call it `setup(shares mut game)`, and capture it `owns game`: `register_cb(fn() owns game: game.add_point())`" |
| a write through a plain name or parameter that holds a `MutSpan` (R2, R4, R5; AN-14) | E3023 | "`dst[i] += src[i]` writes an element of the caller's span, but `dst` only reads it; write `fn blend(ref mut dst: MutSpan[float], src: Span[float]):` and call it `blend(ref mut d, s)`"; "`d[i] = 0.0` writes an element of `tracks[dst_name]`, but `d` only reads it; write `d = ref mut tracks[dst_name].as_mut_span()`" |
| a call through a stored function value whose functions may change their own state, from a name that only reads it (R12; AN-53, AN-62) | E3023 | "`h` only reads the handler in `this.handlers`, and calling it may change its own state (the handler stored on line 20 adds to its own `count`); walk the handlers with `for ref mut h in this.handlers:` in place of line 9"; "`cb` changes its own copy of `score` each time it runs, and `run` takes it read-only; write `fn run(ref mut f: fn()):` and call `run(ref mut cb)`" |
| a change of what a read-only name reads while the name is still used (R8 (g)) | E3021 | "`enemies[0].hp = 1` (line 7) changes the enemy `boss` reads (`boss = enemies[0]`, line 5), and `boss` is used on line 9; change it after `boss`'s last use, or write `boss = ref mut enemies[0]` and change it through `boss`: `boss.hp = 1`" |
| a walk with views across a call that may change nodes (R8 (g), R11; AN-62) | E3021 | "`this.on_visit(p)` (line 21) may end a `Node` (it may run `prune`, stored on line 40, which removes a node from its parent's list), and `p` reads a node and is used again on line 23; walk with a link, which may outlive its node: instead of `if leaf exists as n:`, `cur = ref n`, the `while` line and `cur = ref p`, write `up = weakref leaf`, then `while up exists as n:`, inside it `if n.parent exists as p:` with `up = weakref p` and `this.on_visit(p)`, and `else: break`" |
| `owns this` on `init` (R5; AN-32) | E2108 | "`init` builds the object, so it cannot take it over; write `fn init(name: String):`" |
| `init` calling a method of the object before every field that method's body reads is set (R5; row 7) | E2100 | "`init` calls `load_assets()` (line 6), which reads `this.assets` (its line 14), and `assets` is set only on line 7; give it a value first: write `this.assets = []` above line 6" |
| a first line whose parts are out of order (R5) | E0118 | "the parts of `finish`'s first line are out of order; write `fn finish() owns this -> u64:`" |
| `this` given another object in a class method (AN-50) | E2103 | "`this = Node()` cannot replace the `Node` object `reset` was called on; set its fields instead: `this.v = 0`" |
| `self` (owner, 2026-10-25) | E1010 | "`self` is not a word of Ember: inside a method its object is `this`; write `fn reset():` in place of line 4, and `this.count = 0` in place of line 5" |
| `this` where there is no object (a free function, a `static fn`) | E1010 | "`starting_at` is a `static fn`: it is called on the type, `Counter.starting_at(5)`, and has no object, so no `this`; build the counter from `k`: `return Counter(n=k)`" |
| a statement at module level; Python's `global`; a clause after a function's brackets that Ember does not have (R5, R15; AN-76) | E0100 | "`FRAME = 0` is a statement, and a module holds declarations; for a module variable that may change write `static mut FRAME: int = 0`"; "`global` is Python's; in Ember a function changes a module variable by its name, with no word: remove line 9"; "`changes FRAME` is not Ember: the compiler works out what `tick` changes from its body; write `fn tick():`"; "a function's captures follow its brackets with no `with`: write `fn() copy of score:`" |
| `@view` on a type that holds no view (R7) | E2030 | "`@view` on `Point`, which holds no view; remove `@view`" |
| a `pub` generic whose bounds are not written (R14, R17; AN-69) | E2040 | "`total` is `pub`, so its bound is written: write `[T: Add[T, Output = T]]` in place of `[T]` above `pub fn total(a: T, b: T) -> T:`" |
| `dyn I` of an interface whose generic method returns or stores its type parameter (R14; AN-67) | E2050 | "`Inspect` cannot be used as `dyn Inspect`: `make` returns its `V`, and a `dyn` value has one entry per method; take the visitor in instead of returning it, `[V: Visitor]` above `fn make(ref mut v: V)`, or keep each list's own type with a type parameter instead of `dyn Inspect`: `[I: Inspect]` above `fn walk(xs: Array[I]) -> int:`" |
| `static fn` outside a type body (R5) | E2115 | "`helper` is not in a type, so it already has no object; remove `static`: `fn helper(n: int) -> int:`" |
| `mut` alone on a name (row 8; AN-27, AN-38) | E0113 | "`mut path` says `path` may change but not whose list it is; to change that list through `path`, write `State.Walking(ref mut path):`"; "`mut n` on a parameter: to change the caller's `a` through `n`, write `ref mut n: int`, and call it `bump(ref mut a)`"; "`mut this`: a method changes its object through `this` with no word; write `fn reset():`" |
| `ref` or `ref mut` in a parameter's type (AN-27) | E0117 | "the mode goes on the name: write `ref mut c: Counter`" (or "`c: Counter`" for `c: ref Counter`) |
| a call whose word does not match the parameter (R4) | E3028 | "`bump` may change `a` (its first line says `ref mut c: Counter`); say so at the call: `bump(ref mut a)`"; "`blend` takes `dst` as `const ref mut`, which writes the elements of `d` and never points it elsewhere; say so at the call: `blend(const ref mut d, s)`"; "`integrate` changes the elements of `p` (its first line says `ref mut pos: MutSpan[float]`); say so at the call: `integrate(ref mut p.as_mut_span(), v.as_span(), 0.5)`"; "`show` only reads its argument; remove `ref mut`: `show(a)`"; "`top` takes the lock of `scores` itself; a `Mutex` takes no word at the call: `top(scores)`" |
| `ref mut` before something this function may not change (a plain parameter, a temporary, a query item it only reads, a place reached through a read-only name) | E3027 | "`p` only reads the caller's player, so it cannot be given to `heal` as `ref mut`; take it as `ref mut p: Player` on `revive`'s first line, or walk the list with `for ref mut p in players:`"; "the query reads `Velocity` only; to change it, ask for it: `Query[(ref mut Position, ref mut Velocity)]`"; "`pool` only reads the pool in `this.pools`, so `ref mut pool.dense[...]` cannot be taken through it; name it so it may change: `pool = ref mut cast this.pools[k] as Pool[T]`" |
| two changeable arguments, or two `ref mut` names, of one place, other than one whole variable proved when compiling (R4; AN-03, AN-18) | E3022 | "`collide` changes `a` and `b` as two different bodies, and both are `bodies[i]`; pass two different bodies, under `if i != j:`: write `if i != j:` above this line and `collide(ref mut bodies[i], ref mut bodies[j])` under it". where the two indexes are not proved different (verdict 98): "`collide` changes `a` and `b` as two different bodies, and `bodies[i]` and `bodies[j]` are the same body when `i == j`; write `if i != j:` above this line and indent it under it"; a function value that changes the call's object (R12), with `fn keep(s: String) -> bool:` whose body pushes `String("dropped an empty line")` to `LOG` for each empty `s` and returns `s != ""`: "`LOG.retain(keep)` changes `LOG` while it walks it, and `keep` adds to `LOG` too (its line 3); drop the lines first and push the notes after: `before = LOG.len()`, `LOG.retain(fn(s) => s != "")`, `dropped = before - LOG.len()`, then `for k in 0..dropped: LOG.push(String("dropped an empty line"))`"; a function value stored where a variable it changes owns it (R12; AN-53, AN-62): "this function clears `b.handlers`, and `on_click` would keep it in `b.handlers`, so it would change the button that holds it while the button runs it; clear the handlers after the click instead: `b.click()`, then `b.handlers.clear()`" |
| a changeable argument and another argument that may reach into it (R4) | E3021 | "`refill` may replace the lists of `w1.lists` through `dst` while `src` shows numbers inside one of them, and `w1` and `w2` may be the same `Wrapper`; give `src` a list of its own: `first = copy of w2.lists[0]`, then `refill(ref mut w1.lists.as_mut_span(), first.as_span())`"; "`blend` changes the elements of `d` (`tracks[dst_name]`, line 10) while `s` shows `tracks[src_name]` (line 11), and the two are one list when `dst_name` equals `src_name`; prove them apart first, `assert_disjoint(d, s)` before line 12, with `from std.mem import assert_disjoint` at the top, or blend from a copy: instead of lines 10 to 12 write `old = copy of tracks[src_name]` and `blend(ref mut tracks[dst_name].as_mut_span(), old.as_span())`" (AN-14's `mix`) |
| a call that hands a parameter (a method's object included) a part of what the callee, by its body, may end or move before its last use of that parameter; a `ref mut` parameter a part of what the callee reaches by another name while the parameter is used (R4) | E3031 | "`note` clears `LOG` (its line 3) and then uses `s` (its line 4), and `LOG[0]` is part of `LOG`; give it a copy: `note(copy of LOG[0])`, or in `note` use `s` before `LOG.clear()`"; "`hit` clears `ENEMIES` (its line 3) and then changes `this.hp` (its line 4), and `ENEMIES[0]` is part of `ENEMIES`; in `hit`, change `this.hp` before `ENEMIES.clear()`"; "`rename` clears `LOG` (its line 2) and then writes `s` (its line 3), and `LOG[0]` is part of `LOG`, so `s` would point at freed memory; rename a copy of your own: write `mine = copy of LOG[0]` and `rename(ref mut mine)` in place of this line, or in `rename` write `s` before `LOG.clear()`"; "`heal` reads `PARTY[0]` by its own name (its line 2) while `p` is still used (its line 3), and `p` is `PARTY[0]`; in `heal`, read it through `p`: `println(p.hp)`"; at the line that stores a function into a function value: "`on_visit` is called with a node of the tree (line 21), and this function removes its node from the parent's list (line 41) and then reads it (line 42); read it before removing it: move line 42 above line 41" |
| a `ref mut` name's place used through another name while the `ref mut` name is still used, a reshape of a `ref mut` address included (R7, R8 (f)) | E3022 (a change), E3021 (a read) | "`p` is lent to `cam.target`, a `ref mut` field, until its last use on line 17, and line 16 lends it to `hud.target` too; if both may change the player, write `target: sharedref mut Player` in `Camera` and in `Hud`, and `sharedref mut p` in place of `ref mut p` on lines 15 and 16"; "`boss` is the only name for `enemies[0]` until line 9, and line 7 reads `enemies[0].hp`; read it through `boss`: `boss.hp`"; "`a` is the only name for its turret while `hit` runs, and `t`, opened from `a.target`, may be that turret; write `if t is not a:` above line 20 and indent line 20 under it" |
| `sharedref` without `mut` (R7) | E0114 | "a link that only reads may be held by any number of names: write `target: ref Player`; `sharedref` is for a link that may change its target: `target: sharedref mut Player`" |
| a collection (a list, a map, a set or a string) changed while a `for` loop walks it, whatever the change: directly through its own name, through a call that may change it, through a call through a function value whose functions may change it, through a scoped task started in the loop, or through the end of an object whose `drop` changes it (R8 (c), R5, R12, R15; AN-04, AN-12, AN-76) | E3020 | "`enemies.push(...)` adds to `enemies`, which this loop walks (line 3); collect the new enemies in a list of your own: write `spawned: Array[Enemy] = []` before the loop, `spawned.push(Enemy(hp=5, splits=false))` in place of this line, and `enemies.extend(spawned)` after the loop"; "`xs[0] = 5` (line 3) writes an element of `xs`, which this loop walks (line 2); to change the element the loop is on, write `for ref mut x in xs:` in place of line 2 and `x = 5` in place of line 3"; "`counts[k] *= 2` (line 4) changes `counts`, whose keys this loop walks (line 3); change each value through the loop's own handle: write `for k, ref mut v in counts:` in place of line 3 and `v *= 2` in place of line 4"; "`counts.insert(...)` (line 4) adds to `counts`, whose keys this loop walks (line 3); collect the new keys in a list of your own: write `upper: Array[String] = []` before the loop, `upper.push(k.to_upper())` in place of this line, and `for u in owns upper: counts.insert(u, 0)` after the loop"; "`enemies.clear()` (line 5) ends every `Enemy` in `enemies`, and an `Enemy`'s `drop` pushes to `LOG` (its line 12), which this loop walks (line 4); clear after the loop: move line 5, `enemies.clear()`, below the loop"; "`scope.spawn(...)` (line 6) starts a task that pushes to `names` (`fn(): names.push(String("x"))`), which this loop walks (line 5); start it after the loop: move line 6 below the loop"; "`b.click()` (line 17) adds to `LOG` (through `click`, which runs the handler stored on line 9, `log_click`, which pushes to `LOG`) while this loop walks `LOG` (line 16); click after the loop: move line 17, `b.click()`, below the loop"; "`h(this)` (line 8) may clear `this.handlers` (through `close_all`, stored on line 20) while this loop walks `this.handlers` (line 7); give each handler the Ui to change, `handlers: Array[fn(ref mut Ui)]`, and walk by position instead of the `for` line: `this.current = 0`, `while this.current < len(this.handlers):`, `this.handlers[this.current](ref mut this)`, `this.current += 1`" |
| a list changed while a `for` loop walks it through a library call given a function that changes it, or a foreign call that may run a kept callback (R8 (c), R12; AN-76, AN-78); a call through a function value that may take a lock this thread holds, inside a walk under that lock, is E7018 (R18) | E3020 | "`scores.for_each(...)` (line 4) pushes to `LOG` (through the function it is given) while this loop walks `LOG` (line 3); call it after the loop: move line 4 below the loop"; "`poll_events()` (line 21) is a C function, which may run the callback C keeps from `register_cb` (line 14), and that callback changes `game.names` (through `add_point`, line 6), which this loop walks (line 20); call `poll_events()` before or after the loop: move line 21 below the loop; if `poll_events` never calls back into Ember, say so on its declaration: `@ffi(no_callbacks) safe fn poll_events()`"; for a C function that never calls back, declared without the mark: "`sqrt(d)` (line 21) is a C function, which may run the callback C keeps from `register_cb` (line 14), and that callback changes `game.names`, which this loop walks (line 20); if `sqrt` never calls back into Ember, say so on its declaration: `@ffi(no_callbacks) safe fn sqrt(x: f64) -> f64`; otherwise move line 21 below the loop" |
| a list changed while a generator that walks it will still be used (R8 (c); AN-77) | E3020 | "`d.log.push("open")` (line 3) changes `d.log`, which `g` walks (`g = d.replay()`, line 1), and `g` is used again on line 4; push after `g`'s last use: move `d.log.push("open")` below line 4" |
| a removal inside a counting loop (R8 (c)) | E3020 | "`enemies.remove(i)` (line 4) shortens `enemies`, which `for i in 0..enemies.len():` (line 2) counts over with an end fixed when it started; remove after the loop: instead of `if enemies[i].hp <= 0:` (line 3) and line 4, write `enemies.retain(fn(e) => e.hp > 0)` after the loop (and remove the `for` line if nothing is left in it)" |
| a view used after a change that may reshape what it points into, a position after a removal or reorder, a name opened from a link used after a line or call that may end its object, a view of what a function value's state owns used after that value is replaced or ended (R8 (a), (b), R10, R12; AN-01, AN-35, AN-48, AN-52) | E3060 | "`enemies.remove(0)` (line 8) may put another enemy in the place `boss` names (line 6), and `boss` is used on line 9; use `boss` before line 8, or name the enemy again after it: `boss = ref mut enemies[0]`"; "`remove_all(ref mut players)` (line 12) may destroy the player `p` reads (`if cam.target exists as p:`, line 10), and `p` is used on line 13; use `p` before line 12, or open the link again after it: write `if cam.target exists as q:` above line 13, indent line 13 under it and use `q` there"; "`reset()` (line 7) may end `Player` objects (it runs the function stored on line 3, which clears the players it owns), and `p` reads one (`if cam.target exists as p:`, line 5) and is used on line 8; use `p` before line 7, or open the link again after it: write `if cam.target exists as q:` above line 8, indent line 8 under it and use `q` there"; "`rename()` (line 9) replaces the node's name (`n.name = String("b")`, line 6), and `s` views that name's text (line 8) and is used on line 10; use `s` before line 9, or keep your own text: `s = String(p.name.trim())`"; "`h.keep = fn(): pass` (line 11) ends the function stored there on line 9 and the node it owns (`owns n`), and `p` reads that node (`if w exists as p:`, line 10) and is used on line 12; use `p` before line 11"; "`bag.items.push(k)` (line 15) may move the list that `h.view` points into (`pick` may have put a view of `bag` there on line 13), and `h.view` is used on line 16; use `h.view` before line 15"; "`m.insert(String("z"), 1)` (line 8) changes `m`, whose keys `ks` (line 7) still views on line 9; use `ks` before line 8, or keep your own strings: write `ks = sorted(m.keys().copy_each())` in place of line 7" |
| a change to a temporary whose change would be lost (R8 (d); AN-10, AN-23) | E2140 | "`heal` changes the enemy that `enemy_at(es, 0)` returns, a copy that ends on this line, so the change would be lost; to heal the enemy in `es` write `es[0].heal()`, or let `enemy_at` give back the enemy itself: `fn enemy_at(ref mut xs: Array[Enemy], i: int) -> ref mut Enemy:` with `return ref mut xs[i]`, called `enemy_at(ref mut es, 0).heal()`" |
| a view of a temporary returned or kept too long (row 5; AN-09) | E3063 | "`word` is a view of the lowered copy that `line.to_lower()` made on line 2, which ends when `clean` returns; return `String(...)` to hand back your own copy: `return String(word)`, with the result type `String`" |
| a view leaving a lock's `with` block (row 4; AN-30) | E3063 | "this points into the locked list, which can change once the lock ends; return a copy, or do the work inside the `with` block" (row 4's own words); for a read lock or a guarded field, the same with the line: "this points into the read-locked list, which can change once the lock ends; return a copy: `return copy of g[0]`, with the result type `String`" |
| a view of the callee's own locals put into the caller's view through `ref mut` (R4) | E3063 | "`best` is the caller's view and lives on after `pick_longer` returns, but `mine` ends with `pick_longer`; put a view of what the caller gave there (`best = other`), or return a list of your own: `-> Array[int]`" |
| a kept view that outlives what it views (AN-52) | E3063 | "`ws` holds views of `players`, so it may not outlive it (returned here); let the watchers keep their players alive: write `target: shares mut Player` in `Watcher`, `players = [shares mut Player(), shares mut Player()]` in place of line 3 and `Watcher(target=shares mut players[r])` in place of `Watcher(target=sharedref mut players[r])` on line 6; or keep positions: write `target: int` in `Watcher`, `Watcher(target=r)` on line 6, and `return (players, ws)` with the result type `(Array[Player], Array[Watcher])`" |
| a value made in a memory manager's instance kept or returned where it would outlive the instance (R3; AN-40) | E3063 | "`boss` holds a mesh made in `current` (line 12), and `current` ends when `load_level` returns, so `boss` cannot be returned; make it in an arena the caller keeps: `fn load_level(ref mut arena: LevelArena) -> Enemy:`, with `mesh=owns Mesh(points=[0.0, 1.0]) using arena`, and in the caller `current = LevelArena()` and `boss = load_level(ref mut current)`" |
| a result that points into something its `from` does not name (R6) | E3062 | "`next_word` returns a view of `this.buf`, but its first line says `from src`; return a view of `src`, or write `-> StrView from buf`" |
| a view field of a class without a usable `from`, and a `from` that names what a view cannot point into for its life (R7; AN-22, AN-25, AN-26) | E2118 | "a `Player` keeps `name` after the text it came from may be gone; to keep its own copy write `name: String` and `this.name = String(name)`"; "`words` points into `text`, so `text` must never change after `init`; write `let text: String`"; "`buf` is stored inside the value, so moving the value moves it; keep the text in a `String`, which has a block of its own: write the field as `let buf: String`"; "`pair` is stored inside the object, so moving the object moves it; keep it in a block of its own: write the field as `let pair: owns Pair`"; "`src` is both a parameter of `next_word` and a field of `Parser`, so `from src` could mean either; rename the parameter: for a view of the field write `fn next_word(line: StrView) -> StrView from src:`, for a view of the parameter `fn next_word(line: StrView) -> StrView from line:`" |
| `copy of` before what it cannot copy as itself (R3, R11; AN-07, AN-25) | E2116 | "`copy of this.keep` would copy the link, not the node; for another link to the same node write `a = weakref this.keep`; for a copy of the node itself write `if this.keep exists as n:` and then `c = copy of n`" (offered where `Node` has `Clone`; where it has none, the second way out reads "`Node` has no `Clone`; write `@derive(Clone)` above `class Node`, or keep a view of it: `if this.keep exists as n:`"); "`w` is a view of `line`'s text; to make your own copy write `String(w)`"; "`d.words` points into `d.text`, so a copy's words would point into `d`'s text; make the copy from its text: `d2 = Document(String(d.text))`"; "`Player` has no `Clone`, so `copy of` cannot make one; write `@derive(Clone)` above `class Player`, or keep a view of it: `p2 = ref p`" (a `Copy` type always has `Clone`, R3) |
| `@cached` on a method whose result it cannot keep safely (AN-21) | E2117 | "`@cached` keeps one result, and `radius` takes `scale`; remove `@cached` (line 2), or keep the results in a field: `radii: Map[f64, f64]`"; "`radius` reads `mesh`, which its other owners may change without `Mesh` seeing it, so a kept result could be stale; remove `@cached` (line 13)"; "a cached method keeps one result, so it only reads, and `radius` changes `this.verts` (its line 4); remove `@cached` (line 2)"; "`radius` reads `verts`, which other threads change under `lock`; a kept result could be stale; remove `@cached` (line 18)" |
| `Some(...)` in an `if` or `while` header (R13; owner, 2026-10-25) | E0115 | "an `Option` or a `weakref` link is opened with `exists as`: write `if ui exists as ref mut me:`" |
| an opening whose left side always holds something, and a kind test that always passes (R13, R14) | E2036 | "`n` is an `int`, which always holds a value, so this `if` would always run; to name it write `v = n` in place of the `if` line, and use `v` in its block, moved out one level"; for a `Result`: "to run the block when `r` succeeded write `if Ok(v) = r:`"; "`s` is a `Circle` already, so this `if` would always run; to name it write `c = s` in place of the `if` line, and use `c` in its block, moved out one level" |
| a kind test that can never pass: a type that does not implement the interface or derive from the base class (R14; AN-67) | E2037 | "`s` is a `Shape`, which can never be a `Path`; remove the test: the `if` line and its block" |
| an opening or a kind test that names a name already in use (R13, R14) | E2106 | "`target` already names a value (line 2); `exists as` declares a new name: write `if cam.target exists as t:`, and `t` for `target` in its block"; "`s` already names a value (line 1); `as` declares a new name: write `if s is Circle as c:`, and `c.r` for `s.r` in its block" |
| a name with no word after a name with a word, in a list of names without types (R3) | E0116 | "`ref mut` here belongs to `pos` only; say what `vel` is: `for ref mut pos, ref vel in q:`, or `ref mut vel` to change it too"; "`copy of` here belongs to `name` only; say what the function does with `score`: `fn() copy of name, copy of score:`, or leave `score` out to use the real `score`"; "`ref mut` here belongs to `ui` only; say what `dx` is: `fn(ref mut ui, ref dx)`, or give both their types: `fn(ref mut ui: Ui, dx: float)`" |
| `ref mut` in a lambda's capture list (R3) | E0120 | "remove `ref mut x`: without a word the function already uses the real `x`, and may change it: `fn(): x += 1`" |
| a `weakref` of a part: a field, or an element stored inline (R11) | E2119 | "a `weakref` link needs an object that lives on its own; link to the player: `cam.target = weakref player`" |
| a value made for a place that owns it alone (an `owns` field, an `Option[owns T]`, a list of `owns` values), at first or later, without `owns` on the line that makes it (R3) | E2123 | "`mesh` owns its mesh; say so where it is made: `mesh=owns Mesh(points=[0.0, 1.0]) using current`"; "`left` holds its tree in a block of its own; say so where it is made: `left=Some(owns Tree(value=2))`"; "`pools` holds each pool in a block of its own; say so where it is made: `r.pools.push(owns IntPool(items=[1, 2]))`"; "`mesh` owns its mesh; say so where the new one is made: `e.mesh = owns Mesh(points=[1.0, 2.0])`" |
| a type that contains itself, or a view of its own type, with no block of its own between (AN-41) | E2200 | "`Tree` contains itself through `left`, so a `Tree` would have no end; hold the child in its own block: `left: Option[owns Tree]`"; "`Node.parent` would be a view of the `Node` that holds this one, which no struct can keep; for a link back to the parent make `Node` a class with `children: Array[Node]` and `parent: weakref Node = None`, set with `child.parent = weakref this` before `this.children.push(child)`" |
| `owns` before a name that already exists, or before `fn` (R3; AN-47, AN-54) | E0109 | "`e.mesh = m` already hands `m` over; remove `owns`"; "`b = a` already hands `a` over; remove `owns`"; "`owns fn` does not say what the function takes or copies; name it on each captured name: `fn() owns report:`" |
| a type word that cannot apply to its type: `owns` on a class type, `using` after a type with no heap memory of its own (R3; AN-40) | E2114 | "a `Player` object always lives in its own block; remove `owns`: `p: Player`"; "an `int` has no memory of its own to take from `LevelArena`; remove `using`" |
| a bare interface name used as a type (AN-66) | E2173 | "`Shape` is an interface, so `Array[Shape]` does not say who owns the shapes; say who owns them: write `shapes: Array[owns dyn Shape]` in place of line 19 (the list is their only owner), and `scene = Scene(shapes=[owns Square(side=2)])` in place of line 22" (with `Array[shares dyn Shape]` instead, other names own them too, and each shape is made `shares`) |
| a lambda parameter whose mode differs from the expected function type; a `weakref` link used as its target without opening it, or an object stored into a `weakref` place without `weakref` (AN-56, AN-65) | E2020 | "`each_name` gives each name read-only, so this function cannot take it over; to keep your own, write `fn(s): names.push(copy of s)`"; "`each_name` hands each name over (`fn(owns String)`), so this function receives it as its own: write `fn(owns s) => println(s.len())`"; "`cam.target` may be gone; open it: `if cam.target exists as p:`"; "`=` alone would hand `player` over; write `cam.target = weakref player`" |
| a value given to a `using A` place that was not made in an instance of `A` (R3; AN-40) | E2020 | "`mesh` takes its memory from a `LevelArena` (`mesh: owns Mesh using LevelArena`), and this `Mesh` comes from the built-in manager; say which arena: `mesh=owns Mesh(points=[0.0, 1.0]) using current`"; "`frame` is a `FrameArena`, and `mesh` takes its memory from a `LevelArena`; make it in one: `mesh=owns Mesh(points=[0.0, 1.0]) using current`" |
| `@ffi(no_callbacks)` on a foreign function whose contract takes a callback (R12; AN-78) | E5069 | "`register_cb` takes a callback (`param(cb, callback(retained, user_data=ud))`), so it may call back into Ember; remove `no_callbacks` from its `@ffi(...)`" |
| a function value that may outlive a place it uses and does not own, a loop's name a task uses with no word or with `ref` where it may outlive the element included (R12, R15; AN-53, AN-55, AN-57, AN-71, AN-73) | E3026 | "the click handler changes `total`, which belongs to `register`'s caller, and `b` keeps the handler after `register` returns; keep the total in an object the handler can own a share of: declare `class Counter:` with the field `n: int = 0`; in `main`, instead of `total = 0`, `register(ref mut b, ref mut total)` and `println(total)`, write `count = Counter()`, `register(ref mut b, shares mut count)` and `println(count.n)`; and instead of `register`'s two lines write `fn register(ref mut b: Button, owns count: shares mut Counter):` and `b.on_click(fn() owns count: count.n += 1)`"; "the returned function changes `n`, which ends when `make_counter` returns; let the function own it: write `return fn() owns n: n += 1` in place of this line"; "the task may still run in a later round, when `t` names another text; give it its own copy, `scope.spawn(fn() copy of t: ...)`, or let it look at this round's text, `scope.spawn(fn() ref t: ...)`"; "the thread may still run after the loop has moved on and `texts` is gone; give it its own copy: `thread.spawn(fn() copy of t: ...)`" |
| a hand-over out of a name or place that does not own the value, a function value's own capture given away where it may run twice included, and a field taken out of a class object (R2; AN-35, AN-37, AN-59) | E3013 | "`v.push(template)` would hand over the caller's player, but `template` only reads it; to put a player of your own in the list write `v.push(copy of template)`"; "this function gives `s` away, so it can run only once, and `call_twice` runs it twice; hand over a copy each time instead: `call_twice(fn(): consume(copy of s))`"; "`call_once` uses `f` up, so it must hold it: write `owns f: once fn()`"; "`q` is a `Player` object, which other names may reach, so `q.name` cannot be taken out and leave a hole; take it and leave a value in its place: `names.push(mem.take(ref mut q.name))`" |
| `virtual` in a class that cannot be subclassed (AN-70) | E2112 | "`Door` cannot be subclassed, so `virtual` on `on_update` changes nothing; to let subclasses replace it write `open class Door:`, or remove `virtual`" |
| a method called in a generic body that no bound's interface declares (AN-69) | E2072 | "`T` has no `weight`: no interface in scope declares `fn weight`; declare one, `interface Weighed:` with `fn weight() -> int`, and write `struct Crate implements Weighed:` for the type `heaviest` is given; the bound follows" |
| a wait for a lock this thread holds: a call through a function value that may take it, inside a walk under it too, or a join of a thread whose function needs it (R15, R18; AN-75, AN-76) | E7018 | "`t.join()` (line 16) waits for the thread started on line 15, whose function changes `RESULTS` under `RESULTS_LOCK`, and this thread holds that lock, so each would wait for the other for ever; join after the `with RESULTS_LOCK:` block: move line 16 below the block"; "`note()` may run the function stored on line 12, which takes a `Log`'s lock (`log.push`, under `lock`), and this thread holds `log`'s lock, so taking it again would wait for ever; call `note()` after the `with log.lock:` block: move line 16 below the block"; in a walk: "this loop walks `log.lines` (line 9) under `log`'s lock, and `note()` (line 11) may run the function stored on line 4, which takes a `Log`'s lock, so it would wait for the lock this thread holds; call `note()` after the `with log.lock:` block: move line 11 below the block" |
| `thread_shared` on a value not created `thread_shared` (R3; AN-71, AN-75) | E7017 | "`names` is `share_and_keep`'s own object, which it may change; to share it between threads, create it shared: `names = thread_shared Names()`" |
| a `thread_shared` value changed directly (R9; AN-71) | E7014 | "`names` is shared between threads (line 5), so no owner changes it directly, and line 6 pushes to its list; fill the list where the object is made: write `names = thread_shared Names(items=[copy of s])` instead of lines 5 and 6" (AN-71's refused `share_and_keep`) |
| an unscoped thread or a kept callback using a name with no capture word, a loop's name included (R15; AN-71, AN-78) | E7004 | "this thread may outlive `share_plain`, so it cannot use `share_plain`'s `names`; say what the thread holds: `fn() owns names:` (handed over), or create it shared, `names = thread_shared Names()`, and write `fn() thread_shared names:` (one more owner)"; "the thread may outlive the loop, and `t` names another text each round; give it its own copy: `thread.spawn(fn() copy of t: ...)`" |
| a one-thread owner or a lent value reaching another thread: a `shares` or `shares mut` value or a `weakref` link of one held by a function any thread may run, `thread.spawn`'s included; a `thread_shared` object whose class holds plain `fn()` values; a class object handed to another thread while a `weakref` link on this thread points at it (R12, R15; AN-78) | E7015 | "`names` goes to another thread, and `w` (line 15), a `weakref` link on this thread, still points at it, so this thread could read it while the other thread ends it; give the thread a copy of its own: write `h = thread.spawn(fn() copy of names: names.items.len())` in place of line 16" (R15/reject_an_object_a_weakref_points_at_handed_to_another_thread.em); "`on_event` keeps functions any thread may run (`thread_shared fn()`), and this one holds `count`, a `shares mut` owner, which stays on one thread; share the counter between threads: write `count = thread_shared Counter()` in place of line 12, give `Counter` the field `n: Atomic[int] = Atomic(0)` (with `from std.sync import Atomic` at the top), and pass `fn() thread_shared count: count.n.fetch_add(1)`"; "`Bus.handlers` holds plain `fn()` values, and a `thread_shared Bus` may run them on any thread; write `handlers: Array[thread_shared fn()] = []` in `Bus`"; "C may call this function from any thread (`threads(any)`), and `shares` owners stay on one thread; share the game between threads: write `game = thread_shared Game(names=[String("ann"), String("bo")])` and `token = setup(game)` in `main`, `fn setup(game: thread_shared Game) -> Retained[ForeignCallback]:` and `return register_cb(fn() thread_shared game: game.add_point())` in `setup`, and, since `add_point` changes `score`, `from std.sync import Atomic` at the top, `score: Atomic[i32] = Atomic(0)`, `this.score.fetch_add(1)` as `add_point`'s body, and `println(game.score.load())`" (AN-78) |
| two tasks changing one place other than by a reduction, under a lock or through an `Atomic` (R15; AN-71, AN-74) | E7007 | "two tasks change `stats.total` (line 19), and each reads it first (`if stats.total < 10:`, line 18), so each may see a total the other is changing; guard it, which keeps the read and the add together: write `from std.sync import Lock` at the top, in `Stats` `lock: Lock = Lock()`, `total: int guarded by lock = 0` and `fn add_below(limit: int, n: int):` whose body is `with this.lock:`, inside it `if this.total < limit:` and inside that `this.total += n`, and call `stats.add_below(10, x)` in place of lines 18 and 19" |
| a thread-bound value sent to another thread (R15) | E7008 | "`g` is the guard of `m`'s lock, which belongs to the thread that took it; take the lock inside the task: write `fn add_one(m: Mutex[Array[int]]):` with the body `with g = m.lock():` and, inside it, `g.push(1)`, above line 11, and `scope.spawn(fn(): add_one(m))` in place of line 16" (R15/reject_a_lock_guard_sent_to_a_task.em) |
| chunks of a parallel loop that may write one place (R15; AN-74, AN-84) | E7010 | "`players` holds `shares mut Player` objects, and one object may be in the list twice, so two chunks could change it at once; give each player one owner: write `players: Array[Player] = []` in place of line 3, and `players.push(Player(hp=10))` in place of `players.push(shares mut p)` on line 5"; "the parallel loop writes `xs[idx[i]]`, and two rounds may have the same `idx[i]`, so two threads could write one element; run the loop on one thread: remove `@parallel` (line 7)" (R15/reject_a_parallel_loop_writing_at_another_index.em) |
| a module variable of the main thread reached from another thread (R15; AN-76, AN-84) | E7012 | "`tick` changes `FRAME` and `LOG`, module variables of the main thread, and this thread runs beside it; guard both with one lock: write `from std.sync import Lock` at the top, `static TICK_LOCK: Lock = Lock()`, then `static mut FRAME: int guarded by TICK_LOCK = 0` instead of line 1 and `static mut LOG: Array[String] guarded by TICK_LOCK = []` instead of line 2, and put `tick`'s body inside `with TICK_LOCK:`" |
| a guarded field used without its lock (R15; AN-74) | E7013 | "`words` is guarded by `lock`; change it inside `with stats.lock:`" |
| a `@parallel` loop over something it cannot split (R15; AN-84) | E7016 | "`@parallel` splits a range or a list into chunks, and `while i < idx.len():` (line 4) in `place` is neither; instead of `i = 0`, the `while` line and `i += 1`, write `for i in 0..idx.len():`" |
| an unsafe operation outside an `unsafe:` block, an `unsafe fn` body and `into_raw()` included, and `Send` declared by hand without `unsafe extend` (R15, R16; AN-79, AN-81) | E3100 | "`read_u32_le` is an `unsafe fn`, which says its callers must vouch; the reads through `p` on line 4 are this function's own unsafe steps, and nothing says why they are sound; put line 4 in an `unsafe:` block with a `# SAFETY:` note saying why the read is safe: above line 4 write `# SAFETY:` and the reason, then `unsafe:`, and indent line 4 under it"; "`read_u32_le(buf.as_ptr())` (line 9) calls an `unsafe fn`, whose callers must make true what its `@safety` says; put line 9 in an `unsafe:` block with a `# SAFETY:` note saying why that holds here: above line 9 write `# SAFETY:` and the reason, then `unsafe:`, and indent line 9 under it"; "`into_raw()` stops Ember from freeing the widget, so a mistake leaks it or frees it twice; write it inside `unsafe:`"; "`Counter` holds a raw pointer, so the compiler cannot tell that another thread may have it; say that you vouch for it: write `unsafe extend Counter implements Send:` with a `# SAFETY:` note above it" |
| a further argument of a variadic C function that C would promote to another type (R16; AN-84) | E5013 | "C passes `narrow` as an `int` through `...`; if `sum_wide` reads another type, convert it first: `sum_wide(2, wide, narrow as i64)`" (it names only the promotion: what `sum_wide` reads is in C, which the compiler cannot see) |
| `assert_disjoint` inside `@no_alias_checks` (R16; AN-14, AN-81) | E4030 | "`mix` is `@no_alias_checks`, and `assert_disjoint(d, s)` (line 18) compares the two spans while running; blend from a copy, which needs no check: instead of lines 16 to 19 write `old = copy of tracks[src_name]` and `blend(ref mut tracks[dst_name].as_mut_span(), old.as_span())`, or remove `@no_alias_checks`" |
| the removed words (AN-40, AN-45 to AN-47, AN-49, AN-54, AN-65, AN-67, AN-68, AN-80 to AN-83) | E1010, E0104 | each with the line to write: `owns T` for `Box[T]` and for `ForeignBox[T]`, `owns T using A` for `Box[T, A]`, `Array[T] using A` for `Array[T, A]`, `shares T` or `shares mut T` for `Shared[T]`, `thread_shared T` for `SyncShared[T]`, `weakref T` for `Weak[T]`, `if w exists as p:` for `upgrade`, `copy of x` for `.clone()`, `StrView` for `str`, `CStrView` for `cstr`, `ref mut T` for `Mut[T]`, `from x` for `@borrows`, `@no_alias_checks` for `@static_safe`, `@fp(allow_fma)` for `@fp(contract)`, `owns` for the mode `owned` (`owns x: T`, `owns this`), the capture words after the brackets for `owned fn`, `if x is D as d:` for `as?`, `cast x as D` for `as!`, `if s is Circle as c:` for `cast s if Circle` (and an `if`/`elif`/`else` of kind tests for a `match` on one) and for `cast s when Circle`, `.copy_each()` for `.copied()`, `this` for `self`, `fn reset():` for `fn reset(mut self):` (the method changes its object through `this` with no word; `fn drop():` for `fn drop(mut self):`), `owns this` for `owned self`, `if e exists as x:` for `if Some(x) = e:`; `RefCell` and `@sync` with the lines of AN-49 and AN-80; `@no_runtime_checks` with AN-82's; `mem.forget`, which has no replacement but the value's own end (R1). For example: "`cast s if Circle` is not Ember: to run a block when `s` is a `Circle`, write `if s is Circle as c:` in place of `found = cast s if Circle` and `if found exists as c:`, keeping the block; to stop the program when it is not one, write `c = cast s as Circle`"; "a downcast that may fail is a kind test: instead of `a = s as? Circle` and `if a exists as c:`, write `if s is Circle as c:`, keeping its block"; "`upgrade` is not Ember: a `weakref` link is opened with `exists as`; write `if cam.target exists as p:` in place of line 16, and indent line 17 under it"; "`.copied()` is now `.copy_each()`: write `names = sorted(m.keys().copy_each())`"; "`mem.forget` is gone: a value always ends with its clean-up; to give the widget to a C function that keeps it, hand it to a parameter its declaration says it takes (`widget_take(first)`), or, inside `unsafe:`, give it `first.into_raw()`" |

Warned, not refused: an `unsafe fn` with no `@safety` (L3015: "`read_u32_le` is an `unsafe fn` that
reads through the raw pointer `p`, and nothing says what its callers must make true for that; say it
above the function: `@safety("...")`, with what `p` must point to"); an `unsafe:` block with no `# SAFETY:` note
(W3012, today's message); a `# SAFETY:` note on an `unsafe fn` line (W3013: "`read_u32_le` says what
its callers must make true with `@safety`; a `# SAFETY:` note says why one block is sound and goes above
each `unsafe:` block inside it: remove this line"); a type whose `shares` fields can form a cycle of
owners (L3001: "the handler is kept by `ui` and keeps `ui`; capture it as a link that does not keep it
alive, `fn() weakref ui:`, or take the Ui as the handler's parameter"); a `check_disjoint` result left
unused (W2190, today's "unused `Result`": "to stop when they overlap, write `assert_disjoint(dst, src)`,
with `from std.mem import assert_disjoint` at the top; to handle both cases, write
`match check_disjoint(dst, src):`").

## 4. Error codes, one meaning each

"Today's" means the meaning today's compiler gives the code (`compiler/ember_diag/src/codes.rs`). A
code in bold is one that file does not define, whether this model made it or an earlier run did:
E0113 to E0118, E0120, E2037, E2106, E2108, E2112, E2114 to E2119, E2123, E3028, E3031, E5013, E5069,
E7007, E7008, E7012 to E7018 and W3013. The codes this model made were checked unused before they were
taken: E2116, E2117 and E2118 by run 4's step 1 (`target/redesign4/v2-01-rule-writer/free_codes.txt`,
on the laptop), E5069 by step 2 (`target/redesign4/v3-01-rule-writer/free_codes.txt`), E7018 by the lead
for verdict 43, E0118, E0120, E2119, E2123 and E3031 by the lead for step 3 (`grep -rlw` over
`compiler/`, `docs/spec-source/`, `std/`, `tests/` and `tasks/96/redesign/`: no file), and E2037 by this
step (`grep -rn "E2037" compiler/`: no match; `grep -rlw E2037` over the repository outside `target/`
and `.git/`: no file; `target/redesign7/01/free_codes.txt`).

| code | means |
|---|---|
| E0005 | `async` is reserved (today's) |
| E0100 | an unexpected token (today's): `global`; a statement at module level, such as `FRAME = 0`, whose message gives the declaration to write, `static mut FRAME: int = 0`; a clause after a function's brackets that Ember does not have (`changes FRAME`, `reads renderer`, `with` before the captures), whose message gives the first line to write |
| E0104 | an attribute that is gone, with the one to write (`@sync`, `@static_safe`, `@no_runtime_checks`, `@fp(contract)`, `@borrows`) |
| E0109 | `owns` before a name whose place already hands the value over with no word (`e.mesh = m`, `b = a` for a value that is not a number, an argument, `return a`), or before `fn` |
| **E0113** | `mut` alone on a name: a pattern name, a parameter, a lambda parameter, `this` (`static mut` is a declaration and is not this) |
| **E0114** | `sharedref` not followed by `mut` |
| **E0115** | `Some(...)` in an `if` or `while` header |
| **E0116** | in a list of names declared without types, a name with no word after a name with a word |
| **E0117** | `ref` or `ref mut` written in a parameter's type instead of on its name |
| **E0118** | a first line whose parts are out of order (R5's order: parameters, `owns this`, `-> T`, `from`) |
| **E0120** | `ref mut` in a lambda's capture list (`fn() ref mut x:`) |
| E1010 | a name or form the program may not use here: not in scope (`this` outside a method or in a `static fn`), gone from Ember (`self`, `Box`, `ForeignBox`, `RefCell`, `Weak`, `Shared`, the mode `owned`, `Sync` as a property, `cast x if D`, `cast x when D`, `upgrade`, `mem.forget`), or written with a word instead (`a.clone()` is `copy of a`; `.copied()` is `.copy_each()`) |
| E1030 | two items with the same name in one scope, such as a second `main` (today's); nothing else (a statement at module level is E0100) |
| E2020 | mismatched types (today's): a lambda parameter whose mode differs from the function type it is given to (a plain `fn(s)` where the type hands the value over, `fn(owns String)`, included); a `weakref` link used as its target without opening it; an object stored into a `weakref` place without `weakref`; a value given to a `using A` place that was not made in an instance of `A` |
| E2030 | `@view` on a type that is not a view (today's) |
| E2036 | an `if` or `while` opening whose left side always holds a value, or a kind test that always passes (today's "this pattern always matches") |
| **E2037** | a kind test that can never pass: the type tested for does not implement the interface, or derive from the base class, of the value tested |
| E2040 | an unsatisfied interface bound (today's): a `pub` generic whose bounds are not written, one message on the first line of its body that needs a missing bound |
| E2050 | an interface that is not `dyn`-compatible (today's) |
| E2072 | no method or field of that name (today's): in a non-`pub` generic body, a method no bound's interface declares |
| E2100 | a field read before it is set (today's): `init` calling a method whose body reads a field not yet set |
| E2103 | `this` given another object in a class method |
| E2104 | a `let` field changed after `init`, for every type (today's) |
| **E2106** | an `if` or `while` opening, or a kind test, that names a name already in use |
| **E2108** | `owns this` on `init` |
| **E2112** | `virtual` in a class that cannot be subclassed (today the warning W2111) |
| **E2114** | a type word that cannot apply to its type: `owns` on a class type (it always has its own block); `using` after a type with no heap memory of its own |
| **E2115** | `static fn` outside a type body |
| **E2116** | `copy of` before what it cannot copy as itself: a `weakref` link, a value of a view type (a struct holding a `ref mut` field included), a type with no `Clone`, a value whose views point into its own `let` field |
| **E2117** | `@cached` on a method whose result it cannot keep safely: one with other parameters; one whose body changes anything, or reads a `shares`, `shares mut`, `thread_shared`, `weakref`, `sharedref mut` or `Cell` field, or a field other threads change under a lock |
| **E2118** | a `from` that is missing where a view needs one (a view field of a class), names what a view cannot point into for its life (a field that is not `let`, a fixed array or an inline struct field), or names two things (a parameter and a field of one name) |
| **E2119** | a `weakref` of a part (a field, an element stored inline), whose target does not live on its own |
| **E2123** | a value made for a place that owns it alone (an `owns` field, an `Option[owns T]`, a list of `owns` values), at first or later, without `owns` on the line that makes it |
| E2140 | a change to something that is not a place, which would be lost (today's "cannot assign to a value expression"): an assignment to a value; a method whose only effect is changing a temporary that ends with the line |
| E2173 | a bare interface name used as a type |
| E2200 | a type that contains itself, or holds a view of its own type, with no block of its own between (an `owns` field, a list, a `shares` block) |
| E2221 | a view into a generator's own frame held across `yield` (today's) |
| E3013 | a hand-over out of a name or place that does not own the value (a plain parameter's value; a function value's own capture where it may run twice; a plain `once fn()` parameter; a field of a class object, which other names may reach) |
| E3020 | a collection (a list, a map, a set or a string) changed while it is walked, whatever the change (an element written, a value under a key replaced, anything added or removed), by a `for` loop or by a generator that walks it and will still be used; through its own name, or through a call that may change it (a call through a function value or a library call by the functions it may run, a `spawn` by what its thread changes, a scoped task started in the walk, a foreign call by what the callbacks C keeps change, the end of an object by what its `drop` changes); a removal inside a counting loop |
| E3021 | a read and a change of one place overlap: a change of what a read-only name reads while it is still used; a `ref mut` name's place read through another name while it is still used; an argument that may reach into a changeable argument of the same call (today's meaning, "a shared and a mutable borrow overlap") |
| E3022 | one place changed through two names: two changeable arguments of one call naming one place, other than one whole variable proved when compiling (a function value that changes it counts as one); a function value stored in a place that a variable it changes owns; a `ref mut` name's place changed through another name while it is still used, a reshape of a `ref mut` address included (today's meaning, "two mutable borrows of the same place") |
| E3023 | a change the name does not allow: a change through a name that only reads (a plain parameter, a loop, pattern, opened or part name, a plain name holding a `MutSpan`, a value made when compiling); `=` on a name that keeps its place for its scope (a parameter, a loop, pattern or opened name, one opened `exists as copy of` excepted, a `const ref mut` name given another ref); a read-only `ref` given to a `ref mut` name; a method that may change its object, or a stored function that may change its own state, called through a read-only name; an owner that may change made from one that only reads (a plain parameter of a `shares mut` type included) |
| E3026 | a function value that may outlive a place it uses and does not own (today's): a returned function that changes its maker's local without `owns`; a task or thread given a loop's name with no word, or with `ref` where it may outlive the element |
| E3027 | `ref mut` before something this function may not change: an argument, a loop or pattern name over something it only reads, or a place reached through a read-only name (today's meaning, "a `mut` argument is not a mutable place") |
| **E3028** | a call's word does not match its parameter: no `ref mut` (or no `const` for a `const ref mut` parameter) before an argument for a `ref mut` parameter (a `MutSpan` one included), or `ref mut` before one for a parameter that is not `ref mut` |
| **E3031** | a call whose argument the callee's body reaches by another name in a way its parameter cannot allow: a part of what the callee may end or move before its last use of that parameter (a plain, `Span` or `ref mut` parameter, or a method's object); for a `ref mut` parameter, a part of what the callee reads or changes by another name while the parameter is used; at the line that stores a function into a function value, when that function does so with what the value's calls give it |
| E3040 | a name used after it was handed over or ended (a number handed over with `owns` included); a part handed over on its own, or its owner used whole, before the part is given a new value |
| E3060 | a view used after what it points into was ended, handed over, replaced or reshaped: a position after a removal or reorder; an address after a reshape; a view put in through `ref mut` after one of the call's arguments changed; a name opened from a link after a line or call that may end its object; a view of what a function value's or an `owns dyn I` value's state owns after that value is replaced or ended |
| E3062 | a result that points into something its first line's `from` does not name (today's) |
| E3063 | a view, or a value holding views, kept or returned where it would outlive what it points into: a view of a temporary, a view out of a lock's `with` block, a view of the callee's own locals put into the caller's view through `ref mut`, a kept view outliving its target; and a value made in a memory manager's instance, or one holding such a value, kept or returned where it would outlive that instance |
| E3100 | an unsafe operation outside an `unsafe:` block, an `unsafe fn` body and `into_raw()` included; `Send` declared by hand without `unsafe extend` |
| E4030 | `assert_disjoint`, the one run-time alias check left, inside `@no_alias_checks` |
| **E5013** | a variadic further argument C would promote (was E5015) |
| **E5069** | `@ffi(no_callbacks)` on a foreign function whose own contract takes a callback (`callback(...)`) |
| E7004 | an unscoped thread or a kept callback uses a name with no capture word (a loop's name included) |
| **E7007** | two tasks of one scope change one place other than by a reduction, under a lock or through an `Atomic` (a task that reads the place before changing it, and a plain assignment, included) |
| **E7008** | a thread-bound value (a guard, a `Scope`, a raw pointer, a `threads(creator)` handle) sent to another thread |
| E7010 | a parallel loop's chunks may write one place |
| **E7012** | a `static mut` not `guarded by` a lock reached from another thread |
| **E7013** | a `guarded by` field used without holding its lock |
| **E7014** | a `thread_shared` value changed directly |
| **E7015** | a one-thread owner (`shares`, `shares mut`, a `weakref` link of one) or a lent value reaches another thread: held by a function any thread may run (`thread_shared fn()`, `thread.spawn`'s included), or in a plain `fn()` field of a class whose object is made `thread_shared` (at that line) |
| **E7016** | a `@parallel` loop over something it cannot split into chunks: neither a range nor a list walked by element |
| **E7017** | `thread_shared` on a name whose value was not created `thread_shared` |
| **E7018** | a wait for a lock this thread holds, inside that lock's `with` block: a call through a function value, or a `dyn` or virtual method, that may run a function taking that lock (inside a walk under that lock too), or a join of a thread (a scope's end included) whose function changes what the lock guards |
| L3001 | a type whose `shares` fields can form a cycle of owners (a warning: a leak, not a memory error) |
| L3015 | an `unsafe fn` with no `@safety` (every one, not only `pub`) |
| W2190 | an unused `Result` (today's), `check_disjoint`'s included |
| W3012 | an `unsafe:` block with no `# SAFETY:` note (today's) |
| **W3013** | a `# SAFETY:` note on an `unsafe fn` line, where `@safety` says it |

Removed, with no case left (`V5/changes.md` says why): E2085, E2105, E2107, E2113, E2122, E2109; E7001,
E7003, E7005; run 3's proposed E3045 (its case is E3023). Not used: E3041 (a hand-over in a loop's later
round is E3040).
