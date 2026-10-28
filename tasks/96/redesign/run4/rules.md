# The memory model's rules (run 4, phase 1; agent 01, rule writer)

The rules of Ember's memory model, rewritten whole from run 3's rules R1 to R17
(`RUN3/combined-v4.md` sections 0 to 9) with the owner's four decisions of 2026-10-25 applied. They are
written clean: what changed from run 3, and why, is only in `RUN4/changes.md`; the words are in
`RUN4/vocabulary.md`. `RUN3` means `tasks/96/redesign/run3/`, `RUN4` means `tasks/96/redesign/run4/`.
"Row N" is the owner's agreed row N in `inputs/JOB.md`.

Power: `PowerLineStatus` printed `Online` at the start of this work and again before writing this file.
No compiler change, no git command, no Ember program run.

## 0. Terms

- **owner**: the one place (a local, a field, a list element) that ends a value when it ends.
- **handed over**: the giver's name ends and the receiver becomes the owner (row 9: `snapshot = items`).
- **copy** (`copied`): a new value equal to the old one; nothing done to one is seen through the other.
- **`Copy` type**: a type whose `=` makes such a copy bit for bit: numbers, `bool`, `ref T`, `Span[T]`,
  and structs made only of these. Class objects, `shared` values, `weak` and `sharedref mut` links and
  `ref mut` views are not `Copy`.
- **place**: where a value lives: a local, a field, a list element, an enum variant's payload.
- **part**: a field (`p.name`), an element (`enemies[0]`), or a payload (`State.Walking(path)`).
- **view**: a name or value that points into a place someone else owns and may not outlive it:
  `ref T`, `ref mut T`, `sharedref mut T`, `Span[T]`, `MutSpan[T]`, `StrView`, a plain parameter, a
  plain loop or pattern name, a name for a part. A **kept view** is a view stored in a struct or a list.
- **into X**: a view points into X when it points at X or at anything X owns, all the way down (X's
  fields, the elements of X's lists, X's `owned` parts). It does not go through links: what a link
  reaches is another object (R9, R10).
- **link**: a field or value that names an object it does not own alone: `weak T` (does not keep the
  object alive), `shared T` and `thread_shared T` (one of several owners), `sharedref mut T` (a view
  several holders may share, R7).
- **position**: a view of a list element that finds the element again at each use (R8 (a)).
- **reshape**: a change that can free or move memory: replacing a value that owns memory; `push`,
  `pop`, `insert`, `remove`, `clear`, `sort`, `retain`, `extend` on a list or map; changing an enum's
  variant; ending an object's last owner. Writing a number or a `bool` is not a reshape.
- **`this`**: inside a method, the object the method was called on.
- **first line**: a function's signature with its clauses: what a method does to `this` (`reads`,
  `changes`, `ref mut this`, `owned this`), what a function changes in module variables (`changes`), the
  result type, `from`.
- **`shared` / `thread_shared` value**: a value with several owners, all on one thread / on any
  threads, freed when its last owner ends. Its owners are **counted**: a number in the value's header,
  changed with a plain add (one thread) or an atomic add (any threads).
- **weak count**: a second number in an object's header; it keeps the object's memory block (not the
  object) until the last `weak` link to it ends, so a link can still test whether the object lives.
- **count pair**: one +1 and one -1 of a count around a use, to keep an object alive across it.
- **deferred-drop region**: a loop during which an object whose last owner ends is set aside whole,
  marked dead, and freed when the loop ends.
- **function value**: a function or lambda kept in a variable, a list or a field: two words, its code
  and its **state** (what it captured). A **capture** is a name a lambda uses from the function that
  made it; **the real variable** (row 6) is that very variable, not a copy. A **box** is a small counted
  heap block holding one captured variable, shared by everyone who uses the variable.
- **unknown call**: a call through a function value or a `dyn` value, or into code whose effects the
  compiler cannot see; it is taken to do everything its type allows.
- **summary**: what the compiler works out from a function's body (what it may change, whether it may
  run a function value); across a package's edge it is recorded in the package's interface file (R17).
- **scope**, **task**, **join**: the block `with scope = thread.scope():`; the threads started in it;
  the end of the block, which waits for every task.
- **noalias**: C's `restrict`, the promise to the C compiler that two pointers never point at the same
  memory, which lets it keep values in registers and use vector instructions.
- **the owner's test**: for every name a line declares, the word that says what it is (the owner's own
  thing, a copy, or handed over) and the word that says whether it may change it; a name with no word
  only where the stated default never changes anything, never aliases silently, never copies silently.

## 1. The rules

**R1. One owner; `=` with a whole name hands it over.** Every value has one owner unless a line says
otherwise. `b = a`, where `a` is a whole name whose type is not `Copy`, hands `a` over: `a` ends, and a
later use of it is refused (E3040, naming both ways out, `copied` and `shared`; row 9). For a `Copy`
type `=` copies. A `Copy` struct copied with `=` from a name that is still used afterwards is not then
changed through the new name (E3023): the line says `copied` (a copy of your own) or `ref mut` (the
original), so no copy is changed by mistake.

**R2. A name for a part is a read-only view; `=` puts a value in the place a name stands for.** A name
is declared by the line that first gives it a value; a line inside a loop's body declares it once per
round (`[GRM-4]`). A name bound to a part (`n = p.name`, `boss = enemies[0]`), or to a call whose
result is a view or a `ref` place (`word = line.to_lower().trim()`, row 5; `best = a.as_span()`), is a
read-only view of it: never a hand-over, and never a silent copy, except that a part with no fields of
its own (a number, a `bool`, a variant with no payload) is copied, as Python binds a number. A word
before the part says otherwise: `ref mut` makes a name for that very place that may change it
(`boss = ref mut enemies[0]`, `r = ref mut first_even(xs.as_mut_span())`); `copied` makes a copy of your
own (rows 6, 9); the link words of R3 make a link; `ref` says the default, and before a part it is
needed only to view a number (`r = ref counts[0]`). A part stored somewhere else (`push`, `return`, a field, an `owned`
argument) is handed over, which leaves its owner partly handed over.
`=` on a name that already has a value puts the new value in the place the name stands for. A name
declared with `=` stands for its own place: its old value is replaced (and ends, if it owned
something), and if the value is a view, the name now views something else (`rest = tail.trim()`),
checked by R8 against its new target. A `ref mut` name stands for the place it was made from:
`r = ref mut a` then `r = 20` writes `a`, and `r = ref mut b` is refused, since it would write `b`'s
value into `a` (E3023). A parameter, a loop name, a pattern name and a name opened by
`if` or `while` (R13) stand for what the caller, the loop or the test gave, and keep it for their whole
scope: `=` on one is refused (E3023), unless it is `ref mut`, which writes the caller's place.

**R3. Words before a value, in a type, and in a list of names.**
- Before a value, a word acts on the object the value names (for a link or an owner, on its target):
  `copied x` (a copy, made by the type's `Clone`); `shared x` (one more owner, on this thread);
  `thread_shared x` (one more owner, on any thread: only before a new value, `names = thread_shared
  Names()`, or on a value already `thread_shared`, else E7017); `weak x` (a link that does not keep it
  alive, R11); `sharedref mut x` (a changeable link other holders may share, R7). `owned x` before a
  value is refused: `b = a` already hands over (E0109).
- In a type: `T` (held alone: a struct's value inside its owner, a class object in its own block);
  `owned T` (held alone in its own heap block, replacing `Box[T]`, `Box[dyn I]` and `ForeignBox[T]`;
  refused on a class type, which always has its own block, E2114); `shared T`; `thread_shared T`;
  `weak T` and `weak T set by f` (R11); `ref T`, `ref mut T`, `sharedref mut T` (R7); `owned dyn I` and
  `shared dyn I` (a bare interface name used as a type is refused, E2173).
- In a lambda's capture list, after its parameters: `fn() with copied a, owned b, shared c,
  thread_shared d, weak e:`; each word means what it means before a value, and a name not listed is the
  real variable (row 6, R12).
- In a list of names declared without types of their own (a capture list, a loop's names, a pattern's
  names, a lambda's parameters written without types) each word belongs to the one name after it, and a
  name with no word may not follow a name with a word, so that no word can be read as covering the next
  name (E0116): `for ref mut pos, ref vel in q:`; `with copied name, copied score`;
  `fn(ref mut ui, ref dx)`. `ref` says a name's default (a read-only view) and may be written on any
  declared name. A parameter list with types, and a call's arguments, are not such lists.

**R4. Parameters and calls.**
- `x: T` reads the caller's value (a read-only view; a number is copied in). `ref mut x: T` names the
  caller's place: every change through `x` reaches the caller's value, and `x = v` puts `v` there;
  where the value is a link or a view, the place holds the link or the view (`ref mut b: shared Player`
  may re-point the caller's link and change the player; `ref mut best: Span[int]` may point the caller's
  view elsewhere). `owned x: T` takes the value over (refused where `T` is known to be `Copy`, E2085).
  The mode is on the name, and the type says what the value is: `x: shared T` reads the caller's shared
  value with no count, `owned x: shared T` makes the callee one more owner. A lambda parameter's mode is
  what is written on it; a callable type writes the same words (`fn(ref mut Ui)`).
- **A call says `ref mut` where its parameter does**, with the same words: `bump(ref mut a)` for
  `fn bump(ref mut c: Counter):`; `collide(ref mut bodies[i], ref mut bodies[j])`;
  `this.handlers[k](ref mut this)`. A missing `ref mut`, or one written for a parameter that does not
  say it, is refused (E3028), and so is `ref mut` before something this function may not change
  (E3027). The object a method is called on is not marked (`a.reset()`): the
  method's first line says what it does to it (R5). A hand-over to an `owned` parameter is written as
  `=` writes one, with no word (row 9). An argument whose type is made to be changed through any name
  (`MutSpan`, `Mutex`, `RwLock`, `Atomic`, a channel end; R5) needs no word: its type says it.
- **Arguments that may be changed do not overlap.** Two arguments of one call (the object a method is
  called on counts as one), of which at least one may be changed (a `ref mut` argument, the object of a
  `ref mut this` method, or a `MutSpan` whose elements own memory), may not name one place, and neither
  may reach into what the other owns. Where the two differ only by an index of one list
  (`bodies[i]`, `bodies[j]`) and the compiler cannot prove the indexes differ (different constants, `j`
  from `i + 1..`, inside `if i != j:`), one compare before the call stops the program if they are equal.
  Any other possible overlap is refused when compiling: E3022 where both may be changed, E3021
  otherwise. A `MutSpan` of elements that own no memory (numbers, `bool`, `Copy` structs) makes no such
  promise: two spans of one list of numbers may be given to one call, which is then compiled without
  noalias for them (AN-14's meaning and cost).
- Every `ref mut` parameter, `ref mut this` included, is compiled with noalias, except where a link the
  function reads (`weak`, `shared`, `thread_shared`, `sharedref mut`) may reach the same object (R9);
  there no such promise is made, and R8 (b) keeps every view safe either way.
- **A view put in through `ref mut`.** After the call, a caller's place given to a `ref mut` parameter
  whose value is a view may point into what it pointed into before, or into any of the call's other
  arguments (R6). Inside the callee only such a view may be put there: another view argument or a part
  of one, or text written in the program; a view of the callee's own locals or of a temporary is
  refused (E3063).

**R5. What a method's and a function's first line says.** Inside a method its object is `this`; a
method does not list it among its parameters. A function in a type body is a method unless it is
written `static fn`: it then has no `this` and is called on the type (`Pool.make(3)`); `static fn`
outside a type body is refused (E2115). After the parameters the first line says what the method does
to `this`: nothing (it only reads `this`); `reads a, b` (it reads exactly these fields; row 7);
`changes a, b` (it may change exactly these fields and anything reached through them; row 3);
`ref mut this` (it may
change any field, or the whole value); `owned this` (it takes the object over, and the caller's name
ends). `reads` and `changes` may stand together; `ref mut this` and `owned this` stand alone (E2107).
The order is: parameters, these words, `-> T`, `from ...`, `:` (`fn note(msg: String) changes log:`;
`fn load_assets() reads renderer -> Array[String]:`; `fn reset() ref mut this:`;
`fn next_word() changes pos -> StrView from src:`). `init` sets the fields of the object it builds
through `this` with no word, and may call a `reads` method once the fields it names are set; `ref mut
this` or `owned this` on `init` is refused (E2108). A free function that changes a module variable says
`changes NAME`.
Every clause is checked against the body both ways, the calls it makes included (E2105): a function
that calls one whose first line changes something it can reach names that too (`tick` calls
`this.note(...)`, so `fn tick() changes players, log:`; `main` calls `tick()`, which changes `FRAME`,
so `fn main() changes FRAME:`). A change through a `ref mut` parameter or to an own local needs no
clause, and a thread that `thread.spawn` starts is not a call (R15). What function values change is the
one thing first lines do not show, because the type `fn()` has no word for it; R12 says how it is
checked. A call through a `virtual` or interface method is checked against the first line of the method
it names, so an override says no more than that line (E2113); a base whose overrides change their
object says so: `virtual fn on_update(dt: float) ref mut this:`.
**A plain name never changes anything**: a plain parameter and a plain loop, pattern or opened name
never change what they name or what it leads to (a `Cell` field, an object behind a `shared` link):
E3023, with the line to write. Nor does `this` in a method whose line says neither `changes` nor
`ref mut this`: a change through it is a body that does not match its first line (E2105). The
one exception is by type: a plain name may change what a `MutSpan`, `Mutex`, `RwLock`, `Atomic` or
channel end leads to, with that type's own operation written at the use (`pos[i] = ...`;
`with g = m.lock():`; `total.fetch_add(n)`; `tx.send(v)`). These are the types made to be changed
through read-only names.

**R6. Views a call gives.** A call's result that is a view, and a view a call puts in through a
`ref mut` parameter, point into the arguments they may come from: a method's into `this`, a function's
into every view argument (into the one, if only one could be the source). `-> T from name` narrows it to
one parameter, or to one field of `this` written bare (`-> StrView from src`); the body may then give
only a view of what `from` names (E3062). Pointing into an argument means pointing into it and into
everything it owns (section 0), so a reshape of anything it owns ends the view (R8 (b)), as Rust's
borrow of the whole argument does. A call whose result type is `ref mut T` or `ref T` is a place
(`first(xs) += 1`; `counts.entry(k).or_default() += 1`), and a name bound to it follows R2
(`r = ref mut first_even(xs.as_mut_span())`). A generator method's result is such a view: after
`g = d.replay()`, nothing in `d` (or in `d.log`, with `-> Generator[int] from log`) may be reshaped while
`g` is still used (E3060), however `replay` walks its list.

**R7. Kept views.** A struct that holds a view (`ref T`, `ref mut T`, `sharedref mut T`, `Span[T]`,
`MutSpan[T]`, `StrView`, or a struct that holds one) is itself a view: it lives no longer than what its
view fields point into (`[TYP-34]`); `@view` may say so and is checked. A local list of views is itself
a view. A class object is never a view: a view field of a class says what it points into, `from <let
field>` (a `let` field of the same object whose contents live in their own heap block) or `from static`
(text written in the program; on a struct's field too, which then does not make the struct a view).
The three kinds of kept view, by their word:
- `ref T` reads; any number of names may view one place.
- `ref mut T` may change, and it is alone: while its holder is still used, the place is not read or
  changed through any other name (R8 (f)).
- `sharedref mut T` may change, and several holders, and the owner, may name one object. On one thread
  any of them may change it at any time; every view into it ends at a reshape made through any of them,
  and a list in it may not change while a `for` loop walks it, whichever holder reaches it (R8); on two
  threads it is changed only under a lock or through `Atomic` (R9). It is written the same way before a
  value (`Watcher(target=sharedref mut players[r])`), and `sharedref` is always followed by `mut`
  (E0114): a link that only reads is `ref T`, which any number of names may hold already.

**R8. Views and changes, checked when compiling.**
- **(a) Positions.** A name for a list element made by indexing (`boss = ref mut enemies[0]`,
  `p = ps.get(0).unwrap()`), of a list reached through locals, parameters, `this` and one-owner fields,
  is a position: the element is found again at each use. The list may grow while the name is used (its
  buffer is read again after the add); anything that may remove, insert, reorder, clear or replace the
  list ends the name (E3060 at a later use). A read-only position sees, at its next use, plain data
  written through another name.
- **(b) Addresses.** Every other view (a kept view; a view reached through a link; a view into a
  value's own contents: a `String`'s text, an `Array` field's elements, an enum's payload; every view a
  call gives, R6) ends at any change that may reshape what it points into, made through any name that
  may reach it (E3060 at a later use). Two paths may reach the same object when they lead to values of
  one type and at least one goes through a link or a kept view, unless the compiler knows they differ
  (different types, a fresh object, inside `if a is not b:`). What may reshape is read from the function
  itself, from callees' first lines and summaries, and from R12 for calls that may run a function value;
  unknown code may do everything its type allows.
- **(c) Loops.** A list may not change while a `for` loop walks it, directly or through any call that
  may change it (R5, R12) (E3020); the change goes after the loop (`retain`, `extend`, `Commands`). A
  counting loop `for i in 0..xs.len():`, whose range is fixed when it starts, may add to `xs` but not
  remove from it (E3020). A `while` loop that tests its position against the list's current length at
  every step may add and remove: it is not a walk, and each index is checked.
- **(d) Temporaries** (row 5). A temporary lives until the last use of every view made from it, lists
  of views included. Returning such a view, or storing it where it would outlive the temporary, is
  refused with row 5's message (E3063).
- **(e) Guards** (row 4). A view made through a lock's guard (`g[0]`, `ref g[0]`, a slice of `g`) points
  into the guard and does not leave its `with` block (E3063, row 4's message); the same for `RwLock`'s
  `read()` and `write()` and for a `guarded by` field inside `with x.lock:`.
- **(f) A `ref mut` name is alone.** While a `ref mut` name (a parameter, a local, a loop or pattern or
  opened name, a kept `ref mut` field, `ref mut this`) is still used, its place is read or changed only
  through it: another name that may name the same place is refused (E3022 to change it, E3021 to read
  it) unless the compiler proves the two differ. Two `ref mut` names of elements of one list that differ
  only by an index are compared once, as R4 compares two arguments. Growing the list that holds a
  position (`push`) is not a use of the element (rule (a) says what it may do). Objects reachable
  through links are also reachable through those links, as their types say (R9).

**R9. Several owners and shared links; nothing checked while running.** Any owner of a `shared` value,
any `ref mut` name of it and any holder of a `sharedref mut` link to it may change it, written as any
change is written (R5). A `thread_shared` value is never changed directly by any owner: only through
`Atomic`, `Mutex`, `RwLock`, a channel, or a `guarded by` field under its lock (E7014). A scoped task
that reaches a `sharedref mut` target changes it directly only where neither another task nor the
parent may touch it before the join; otherwise only as a `thread_shared` value is changed (E7007, two
tasks or a task and the parent changing one place). An unscoped thread never holds one (E7015: a lent
value reaching another thread). Nothing is checked while running: no access words, no borrow flags;
`RefCell`, `Ref`, `RefMut`, `borrow` and `borrow_mut` are removed. `shared` counts are plain adds and
never cross threads (E7015); `thread_shared` counts are atomic. A class object has an 8-byte header (a
32-bit count and a 32-bit type index); an object that `weak` links may reach has a third 32-bit number
there, its weak count (run 3's C used this 12-byte header and found the objects' packed sizes unchanged,
`RUN3/24-faithfulness.md` section 3).

**R10. Keeping an object alive under a view.** A view of an object reached through a `shared`,
`thread_shared` or `weak` link (a name opened from a link, `if p = cam.target:`; a view made through a
`shared` field, `t = ref mut ws[i].target`; a parameter `x: shared T`, whose life is the call) must not
outlive the object. The compiler protects it in the cheapest of these ways:
1. Nothing, when no line in the view's life can end an owner of an object of that type: no write to a
   place that owns one (`T`, `owned T`, `shared T`, `thread_shared T`), no removal from a list that owns
   them, no end of a local that owns one, no call whose first line or summary may do so. A one-owner
   object lent read-only cannot be ended during the loan.
2. A deferred-drop region, where the view's life contains a loop that makes an unknown call and the
   class has no `drop` method of its own. The region lasts as long as the view: it is placed around the
   loop when the view ends inside it, and around the view's whole life when the view is used after the
   loop, so nothing set aside is freed while the view can still be used. One add and one subtract of a
   per-thread counter per region, nothing per step.
3. Otherwise a count pair around the view's life or around the call.
4. A view of a `thread_shared` object reached through a `weak` link always holds one atomic count for
   its life, as Rust's `Weak::upgrade` does: another thread may end the last owner at any moment, and a
   region belongs to one thread.

**R11. `weak` links.** `weak T` is a link that does not keep its target alive; its target lives on its
own (a class object, or a value held `shared` or `thread_shared`). Every `weak` link holds one weak
count on its target, with no exception: making one (`weak x`) or copying one (`copied w`) adds one, and
its end subtracts one, as Rust's `Weak`. Reading one tests the target's header (does the object still
live?) and is written as an opening (R13): `if p = cam.target:`. A field `weak T set by f` is kept by
the language equal to the object whose field `f` holds this one: set when the object is put into `f`,
cleared when it leaves, never assigned by the program; `f` is changed only through its own operations
(`push`, `insert`, `remove`, `pop`, `clear`, assigning the whole field) and is never lent as a
changeable list (E2109 for both). Such a link holds no count, and reading it is a plain load and a test
for no link, with no header test, because its target lives while it holds this object. A copy of it
(`copied n.parent`) is an ordinary `weak` link. So a walk along `set by` links that keeps views
(`cur = p`, R2) instead of copies pays no count and no header test per step. The cycle warning (L3001) is
given only for a cycle of `shared` owners, and names the link to make `weak`.

**R12. Function values.**
- A capture not named is the real variable (row 6). Where it lives is chosen when compiling and never
  changes what the program does: at its own address, when the function value provably ends first (it is
  kept only in locals, local lists or objects that never leave the function, and every method it is
  handed to says what it changes); else inside the function value's own state, when that function value
  is the only user of the variable from the moment it is made (no other function value captures it,
  the function that made it does not use it afterwards, and the function value is never copied); else
  in a box shared by every user.
- A named capture (`copied`, `owned`, `shared`, `thread_shared`, `weak`) is the function value's own
  state.
- A stored function value is two words, `{code, state}`. A call through a list element or a field
  copies the two words out and keeps no pointer into the list or field. A body that may end its own
  state (it removes itself) and uses that state afterwards holds one count on it across that call.
- A call may change the function value's own state, so a call through a stored function value is a
  change of the place that holds it: the holder is reached through a name that may change it
  (`fn click() changes handlers:` with `for ref mut h in this.handlers: h()`; E2105 or E3023
  otherwise), and a function value that changes its own state is refused where it is given to a
  parameter that only reads (`run(cb)` with `fn run(f: fn()):`; write `ref mut f: fn()` and
  `run(ref mut cb)`). The holder does not become an argument of the call: R4 sees only the call's
  arguments.
- **A call that may run a function value**, that is, a call through one, an unknown call, or a call of
  a function whose summary says it may run a function value it is given or reaches (a library
  `retain`, `sort_by` or `inspect` included), is taken to make every change that the function values
  which may run there make: changes to the real variables they capture (module variables included),
  and changes to any object that their captured links (`weak`, `shared`, `thread_shared`,
  `sharedref mut`) can reach, and to what that object owns. "Which may run there" is what the compiler
  can see: the function values the call is given; where it cannot see which, every function value of
  the type called (of any type, for a `dyn` call). R8 then applies: no `ref mut` name of such a place, no
  view into it that a reshape ends, and no `for` walk of a list in it lives across the call. This is a
  stated limitation: the type `fn()` does not say what a function value changes, so the compiler does
  not follow which handlers each object holds.
- `once fn()` is a function a call uses up; a function value that gives a capture away is one. A
  parameter that takes one holds it, `owned f: once fn()`. A plain `f: once fn()`, and a function value
  that gives a capture away passed where it may run twice, are refused (E3013).
- A function value may not outlive a place it uses and does not own (E3026): it names a word for that
  place (`with shared count`, `with owned xs`), or it runs inside a scope.

**R13. Opening an `Option` or a `weak` link.** `if p = cam.target:` runs its block when `cam.target`
holds something (an `Option` holds a value; a `weak` link's target still lives) and names that thing `p`
inside the block; `else:` runs when it holds nothing; `while p = a:` repeats while it holds something.
The name is declared there with its words, as a pattern name is (R2): plain `p` is a read-only view;
`if ref mut me = ui:` may change what the link points at; `if copied v = scores.get(name):` is a copy
of your own; `if (ref mut a, ref b) = pair:` opens an `Option` of a pair. The right side must be able
to hold nothing: an `Option`, a `weak` link, or `cast x if D` (R14); otherwise the block would always
run, which is
refused (E2036). The name is new, and a name already in use is refused (E2106), so `=` here never
hides a comparison. `Some(...)` is written as a value and in a `match` arm, not in an `if` or `while`
header (E0115). A name opened from a `weak` link is kept alive for its last use by R10.

**R14. Interface values and downcasts.** `dyn I` as a parameter's type is a read-only view
(`ref mut v: dyn I` may change it). `cast p as D` names the place `p` as a `D` and stops the program if it
is not one; `cast p if D` gives it only if it is one and is opened as R13 says
(`if c = cast s if Circle:`; `if ref mut pool = cast this.pools[k] if Pool[T]:`). A cast of a place is a
place: the name bound from it is read-only unless written `ref mut`
(`pool = ref mut cast this.pools[k] as Pool[T]`). Each cast is one type-id compare. A generic method
whose type parameter is only the type of a `ref` or `ref mut` parameter gets one `dyn` entry. A
non-`pub` function's bounds are worked out from its body; a `pub` one writes them.

**R15. Threads.**
- Scoped tasks need no words: every task's effects are checked against the other tasks and the parent
  until the join (E7007 for two of them changing one place other than by a reduction or under a lock).
  Reductions (`total += e`, `-=`, `*=`, `|=`, `&=`, `^=`, `max`, `min`, `push`, `extend`, `add`,
  `m[k] op= e`) get one private accumulator per task, combined at the join in spawn order; nothing reads
  the place before the join. A local of one loop round is given to a task with `with copied n`.
- An unscoped thread, and a C callback that C keeps, say in the capture list what they hold
  (`with owned`, `with thread_shared`, `with copied`; E7004 for a name with no word); `with shared` and
  `with weak` only where the function stays on this thread (`threads(creator)` or `threads(main)`,
  checked by the trampoline).
- `guarded by lock` after a field's (or a `static mut`'s) type names the `Lock` that must be held for
  every read and change outside `init` (E7013).
- What may go to another thread is inferred from the words: `ThreadShared` and `Send` are inferred and
  written only in bounds; `Send` is declared only by an `unsafe extend` author. A one-thread owner or a
  lent value reaching another thread is E7015; a thread-bound value sent away is E7008.
- `static mut NAME: T = e` is a module variable that may change; a function that changes it says
  `changes NAME` (R5); it is reached only by the main thread and its scoped tasks unless it is
  `guarded by` a `static` lock (E7012).
- A `@parallel` loop splits a range or a list walked by element into chunks (E7016 for anything else;
  E7010 where chunks may write one place).
- A generator's `for` over a list walks by position across `yield`, so it holds no address into its
  own frame (E2221 for a view into its frame held across `yield`); what the list may do while the
  generator lives is R6's rule. `async` stays reserved (E0005). C callbacks go through the checked
  trampoline: C holds a tagged `{slot, generation}` number, never an address.

**R16. `unsafe`.** One word with one meaning by position: before a declaration it says "whoever uses
this must vouch", with `@safety` saying what they must make true (L3015 when missing, on every
`unsafe fn`); before a block or an implementation it says "this code vouches", with a `# SAFETY:` note
saying why (W3012 when missing; W3013 for a `# SAFETY:` note on an `unsafe fn` line). Inside an
`unsafe fn` body every unsafe operation sits in its own `unsafe:` block (E3100). `@no_alias_checks`
forbids the run-time alias checks that are left, R4's index compare and `assert_disjoint` (E4030). No
switch turns safety checks off.

**R17. Inferred facts at a package's edge.** A fact the compiler works out from a body never changes
what a line does; it only decides whether a line compiles. Inside a package such facts are used
freely. Across a package's edge, where a word exists to write the fact, only the written word counts:
a `pub` function's bounds are written, and a `pub` method with no `reads` is taken to read every field.
Where no word exists, the fact is recorded in the package's interface file (as today's effect sets,
`[EFF-4]`), and a change to it is reported as an interface change: area A's "only adds" summary of a
`ref mut` list parameter, the effects of scoped tasks, whether a function may run a function value it is
given or reaches, and what the function values a call may run change (R12).

**R18. Messages.** Every refusal names what was refused in the program's own names (`enemies[0]`, not
`*enemies[0]`; `line.to_lower()`, not `_5`; `int`, not `i64`; `fn()`, not `Callable0`), says why in one
line, and gives the exact line to write. One mistake gives one message, at the program's line, on the
name that caused it. The line a message gives compiles under these rules in that program, and where it
replaces several lines the message names every one it replaces ("instead of `i = 0`, the `while` line
and `i += 1`, write `for i in 0..idx.len():`"). A message says only what the compiler knows; where it
cannot see which function value runs, it says so (R12).

## 2. What still runs while the program runs, compared with C

Every number is from run 3, with its source; "PROVISIONAL" marks numbers measured while other agents
ran, as run 3 marked them. Nothing in this table was measured again in this run.

| what runs | when | cost against C, and the source |
|---|---|---|
| bounds checks | indexing, as today | as today (not this rework's) |
| one compare of two indexes (R4, R8 (f)) | two changeable names or arguments of one list's elements whose indexes are not proved different | one compare and a branch never taken: 89.9 ms with it against safe Rust's `get_disjoint_mut` 87.6 ms (`tasks/96/redesign/phase3b-containers.md:301-306`, cited in `RUN3/combined-v4.md` line 533) |
| two compares (`assert_disjoint`) | where the program writes it | buys noalias for two spans of one list of numbers: 1.53x without, 1.00x to 1.07x with (`RUN3/combined-v4.md` line 534, AN-14) |
| a second pass of a loop, then `retain` | only where AN-04's fusion conditions fail | 1.71x of one-pass C as written, 1.00x fused; safe Rust's idiom 1.77x (`RUN3/combined-v4.md` line 535, PROVISIONAL) |
| one queued record per `Commands` change | ECS systems that destroy, create or add | applied after the system, as Bevy (AN-06) |
| a count +1 / -1 | a `shared` owner made or ended; atomic for `thread_shared` | Rust's `Rc` / `Arc`; re-pointed shared targets 1.13x to 1.25x of plain C, safe Rust's `Rc` 1.48x to 1.57x (`RUN3/combined-v4.md` line 537, AN-52 (a), PROVISIONAL) |
| nothing | a `sharedref mut` link (R7) | 1.00x to 1.06x of plain C and the same memory, measured in run 3 as kept `ref mut` views that name one player (`RUN3/combined-v4.md` line 7298, AN-52 (b), PROVISIONAL) |
| nothing, or a deferred-drop region, or a count pair (R10) | a view of an object reached through a link | region 0.944 of plain C against 1.283 for a count pair per step (`RUN3/combined-v4.md` line 538, AN-62, PROVISIONAL, spread 29% to 37%) |
| one add and one subtract of the target's weak count | every `weak` link made, copied or ended (R11) | Rust's `Weak::clone` and its drop |
| one test of the target's header | each read of a `weak` link that is not `set by` | Rust's `Weak` on churn 0.919 and 0.935 of plain C (`RUN3/combined-v4.md` line 539, AN-63) |
| one store to put in or take out; a plain load to read | a `set by` link (R11) | tree workload 1.025 and 1.149 of plain C in the same layout, safe Rust's index arena 1.519 and 1.357 (`RUN3/combined-v4.md` line 541, AN-63). W2, the parent walk: with no header test per step (the form R11 gives a walk that keeps views) 1.176x of plain C, against 1.579x with a test per step and safe Rust's 1.603x (`RUN3/26-timer.md` lines 20 to 22: `diag_w2_hop_proved`, `model_w2`, `rust_w2`) |
| one flag test per call, one store per change of a `reads` field | a `@cached` method | the hand-written C cache (`RUN3/combined-v4.md` line 542, AN-21) |
| nothing / nothing beyond the function value's own block / one box per variable | a captured real variable at its address / in the function value's state / in a box (R12) | W4, callbacks that remove themselves: 1.091x of plain C with the state placement, 1.255x with a box per captured variable, safe Rust 1.358x (`RUN3/26-timer.md` lines 29, 31, 32: `rust_w4`, `model_w4`, `model_w4_state_block`) |
| two words copied out of the list | each call of a stored function value | dispatch 1.045 and 1.037 of plain C, safe Rust 1.490 and 1.472 (`RUN3/combined-v4.md` line 544, AN-60) |
| one count on a handler's own state | a handler that removes itself and then uses its state | a rare path (AN-60) |
| one type-id compare | each `cast` | 1.223 to 1.244 of plain C, safe Rust's `downcast_mut` 1.571 to 1.635 (`RUN3/combined-v4.md` line 546, AN-22, PROVISIONAL) |
| a lock | where the program takes one (`with x.lock:`) | the lock C takes (AN-74) |
| nothing beyond C's per-thread counter | scoped tasks and reductions | 1.00 against an atomic add per word 10.22x (`RUN3/combined-v4.md` line 548, AN-71, PROVISIONAL) |
| the checked trampoline | each C callback | 1.02x to 1.05x, today's 1.47x (`RUN3/combined-v4.md` line 549, AN-78) |
| the generator protocol | each `yield` | position re-read 1.18 of a plain C loop (`RUN3/combined-v4.md` line 550, AN-77) |
| a copy, an allocation | where a line says `copied`, or `owned` / `shared` puts a value in its own block | Rust's `.clone()`, `Box::new`, `Rc::new` |

Gone, compared with today's Ember: the access words checked on class fields (`[EXC-19]`), `RefCell`'s
borrow flag, the count pair per loop step on views of shared objects (AN-51), `@sync`'s frozen fields,
the 24-byte class header (now 8 bytes), and today's 1.47x trampoline. Gone, compared with run 3: the
two-compare overlap test before a call that passes a `MutSpan` of elements that own memory (now a
compile-time rule, R4).

## 3. What stays refused

Each refusal with its code and its message, in the names of an example program (the answer it comes
from in brackets, where there is one). Every message follows R18.

| refused | code | message |
|---|---|---|
| using a name after it was handed over (row 9) | E3040 | "`items` was handed over to `snapshot` on line 3 and is used on line 4; to keep both, write `snapshot = copied items` (a copy of your own) or `snapshot = shared items` (two owners of one list)" |
| a change through a name that only reads (row 8; AN-20) | E3023 | "`path.pop()` changes the list in `n.state`, but `path` only reads it; to change that list through `path`, write `State.Walking(ref mut path):`"; "`tick` sets `s.frame`, but `s` only reads the caller's clock; write `fn tick(ref mut s: Clock):` and call `tick(ref mut clock)`" |
| `=` on a parameter, a loop, pattern or opened name, or a new view given to a `ref mut` name (R2) | E3023 | "`line` is the text the caller passed and stays that text for the whole call; for a view of your own write `rest = line.trim()`"; "`n` is the number the caller passed; for a number of your own write `count = n` and change `count`"; "`r` always refers to `a`, so `r = ref mut b` would put `b`'s value into `a`; to refer to `b` use a new name: `rb = ref mut b`" |
| a `ref mut this` method called through a read-only name (R5) | E3023 | "`heal` changes the player (its first line says `ref mut this`), and `p` only reads it; write `for ref mut p in players:`" |
| a body that changes or reads what its first line does not name, or names what it does not do (rows 3, 7; AN-17, AN-31) | E2105 | "`tick` calls `note`, which changes `log`; add it to `tick`'s first line: `fn tick() changes players, log:`"; "`open` changes `this.angle`; write its first line as `fn open() changes angle:`"; "`load_assets` reads `assets`, which its first line does not name, and `init` calls it on line 6 before `assets` is set; set `assets` first, or write `fn load_assets() reads renderer -> Array[String]:` and stop reading `assets`" |
| `ref mut this` or `owned this` beside `reads`, `changes` or each other (R5) | E2107 | "`reset` says `ref mut this`, which already lets it change every field; write `fn reset() ref mut this:` or `fn reset() changes count:`" |
| `ref mut this` or `owned this` on `init` (R5; AN-32) | E2108 | "`init` may always set the fields of the object it builds; write `fn init(name: String):`" |
| `this` given another object in a class method (AN-50) | E2103 | "`this = Node()` cannot replace the `Node` object `reset` was called on; set its fields instead: `this.v = 0`, with `fn reset() changes v:`" |
| `self` (owner, 2026-10-25) | E1010 | "`self` is not a word of Ember: inside a method its object is `this`; write `fn reset() ref mut this:` for a method that changes it, and `this.count = 0` in its body" |
| `this` where there is no object (a free function, a `static fn`) | E1010 | "`make` is a `static fn`: it is called on the type, `Pool.make(3)`, and has no object; to give it one, remove `static`" |
| `static fn` outside a type body (R5) | E2115 | "`helper` is not in a type, so it already has no object; remove `static`: `fn helper(n: int) -> int:`" |
| `mut` alone on a name (row 8; AN-27, AN-38) | E0113 | "`mut path` says `path` may change but not whose list it is; to change that list through `path`, write `State.Walking(ref mut path):`"; "`mut b` on a parameter: to change the caller's `b` through `b`, write `ref mut b: Body`"; "`mut this`: write `ref mut this` after the parameters: `fn reset() ref mut this:`" |
| `ref` or `ref mut` in a parameter's type (AN-27) | E0117 | "the mode goes on the name: write `ref mut c: Counter`" (or "`c: Counter`" for `c: ref Counter`) |
| a call whose word does not match the parameter (owner, 2026-10-25) | E3028 | "`bump` may change `a` (its first line says `ref mut c: Counter`); say so at the call: `bump(ref mut a)`"; "`show` only reads its argument; remove `ref mut`: `show(a)`" |
| `ref mut` before an argument that this function may not change (a plain parameter, a loop name, a temporary) | E3027 | "`p` only reads the caller's player, so it cannot be given to `heal` as `ref mut`; take it as `ref mut p: Player` on `revive`'s first line, or walk the list with `for ref mut p in players:`" |
| two changeable arguments, or two `ref mut` names, of one place (AN-03, AN-18) | E3022 | "`collide` changes `a` and `b` as two different bodies, and both are `bodies[i]`; pass two different bodies, or write a one-body version of `collide`". Where not proved, a stop while running: "`collide` was given `bodies[3]` as both `a` and `b`; test `if i != j:` before the call to skip it" |
| a changeable argument and another argument that may reach into it (R4) | E3021 | "`refill` may replace the lists of `w1.lists` through `dst` while `src` shows numbers inside one of them, and `w1` and `w2` may be the same `Wrapper`; give `src` a list of its own: `first = copied w2.lists[0]`, then `refill(w1.lists.as_mut_span(), first.as_span())`"; "`refill` may replace strings of `names` through `dst` while `src` shows them; give it two different lists, or a copy: `old = copied names`, then `refill(names.as_mut_span(), old.as_span())`" |
| a `ref mut` name's place used through another name while the `ref mut` name is still used (R7, R8 (f)) | E3022 (a change), E3021 (a read) | "`p` is lent to `cam.target`, a `ref mut` field, until its last use on line 17, and line 16 lends it to `hud.target` too; if both may change the player, write `target: sharedref mut Player` in `Camera` and in `Hud`"; "`boss` is the only name for `enemies[0]` until line 9, and line 7 reads `enemies[0].hp`; read it through `boss`: `boss.hp`" |
| `sharedref` without `mut` (R7) | E0114 | "a link that only reads may be held by any number of names: write `target: ref Player`; `sharedref` is for a link that may change its target: `target: sharedref mut Player`" |
| a list changed while a `for` loop walks it, directly, through a call that may change it, or through a call that may run a function value that changes it (R8 (c), R12; AN-04, AN-76) | E3020 | "`enemies.push(...)` adds to `enemies`, which this loop walks (line 3); collect the new enemies in `spawned` and write `enemies.extend(spawned)` after the loop"; "this loop walks `LOG` (line 8), and `run(log_click)` (line 9) runs `log_click`, which pushes to `LOG` (line 7); call it after the loop, or collect what to add and push it after the loop"; "this loop walks `this.handlers` (line 7), and `h(this)` may run the handler made on line 15, which adds to the `handlers` of the `Ui` it links to (`with weak ui`), and that may be this one; walk by position instead of the `for` line: `this.current = 0`, `while this.current < len(this.handlers):`, `this.handlers[this.current](this)`, `this.current += 1`, with `fn fire() changes handlers, current:`" |
| a removal inside a counting loop (R8 (c)) | E3020 | "`enemies.remove(i)` shortens `enemies`, which `for i in 0..enemies.len():` counts over with an end fixed when it started; remove after the loop: `enemies.retain(fn(e) => e.hp > 0)`" |
| a view used after a change that may reshape what it points into, a position after a removal or reorder (R8 (a), (b); AN-01, AN-35, AN-48, AN-52, AN-77) | E3060 | "`enemies.remove(0)` (line 8) may put another enemy in the place `boss` names (line 6), and `boss` is used on line 9; use `boss` before line 8, or name the enemy again after it: `boss = ref mut enemies[0]`"; "`bag.items.push(k)` (line 15) may move the list that `h.view` points into (`pick` may have put a view of `bag` there on line 13), and `h.view` is used on line 16; use `h.view` before line 15"; "`d.log.push(...)` (line 2) changes `d.log`, which `g` views (`d.replay()`, line 1), and `g` is used on line 3; change `d.log` after the last use of `g`" |
| a view of a temporary returned or kept too long (row 5; AN-09) | E3063 | "`word` is a view of the lowered copy that `line.to_lower()` made on line 2, which ends when `clean` returns; return `String(...)` to hand back your own copy: `return String(word)`, with the result type `String`" |
| a view leaving a lock's `with` block (row 4; AN-30) | E3063 | "this points into the locked list, which can change once the lock ends; return a copy, or do the work inside the `with` block" |
| a view of the callee's own locals put into the caller's view through `ref mut` (R4) | E3063 | "`best` is the caller's view and lives on after `pick_longer` returns, but `mine` ends with `pick_longer`; put a view of what the caller gave there (`best = other`), or return a list of your own: `-> Array[int]`" |
| a kept view that outlives what it views (AN-52) | E3063 | "`ws` holds views of `players`, so it may not outlive it (returned here); return positions instead (`target: int`), or let the watchers own a share: `target: shared Player`" |
| a result that points into something its `from` does not name (R6) | E3062 | "`next_word` returns a view of `this.buf`, but its first line says `from src`; return a view of `src`, or write `-> StrView from buf`" |
| `Some(...)` in an `if` or `while` header (R13; owner, 2026-10-25) | E0115 | "an `Option` or a `weak` link is opened by naming what it holds: write `if ref mut me = ui:`" |
| an opening whose right side always holds something (R13) | E2036 | "`n` always holds a value, so this `if` would always run; to compare write `if v == n:`, to name it write `v = n` on the line before"; for a `Result`: "to run the block when `r` succeeded write `if Ok(v) = r:`" |
| an opening that reuses a name already in use (R13) | E2106 | "`target` already names a value (line 2); to compare write `if target == cam.target:`, to open the link use a new name: `if t = cam.target:`" |
| a name with no word after a name with a word, in a list of names without types (R3) | E0116 | "`ref mut` here belongs to `pos` only; say what `vel` is: `for ref mut pos, ref vel in q:`, or `ref mut vel` to change it too"; "`copied` here belongs to `name` only; say what the function does with `score`: `with copied name, copied score`, or leave `score` out to use the real `score`"; "`ref mut` here belongs to `ui` only; say what `dx` is: `fn(ref mut ui, ref dx)`, or give both their types: `fn(ref mut ui: Ui, dx: float)`" |
| a type that contains itself without `owned` (AN-41) | E2200 | "`Tree` contains itself through `left`, so a `Tree` would have no end; hold the child in its own block: `left: Option[owned Tree]`" |
| `owned` where it can only copy (AN-43) | E2085 | "an `int` is copied, not handed over; remove `owned`"; in a capture list: "write `with copied score`" |
| `owned` before a value or before `fn` (AN-47, AN-54) | E0109 | "`b = a` already hands `a` over; remove `owned`"; "`owned fn` does not say what the function takes or copies; name it on each captured name: `fn() with owned report:`" |
| `owned` on a class type (AN-40) | E2114 | "a `Player` object always lives in its own block; remove `owned`: `p: Player`" |
| a bare interface name used as a type (AN-66) | E2173 | "say who owns the shapes: `Array[owned dyn Shape]` (the list is their only owner) or `Array[shared dyn Shape]` (other names own them too)" |
| a function value that may outlive a place it uses and does not own (AN-55, AN-57) | E3026 | "the click handler changes `total`, which belongs to `register`'s caller, and `b` keeps the handler after `register` returns; keep the total where the handler can own a share of it: `count = shared Counter()` and `fn() with shared count: count.n += 1`" |
| a hand-over out of a name or place that does not own the value, a function value's own capture given away where it may run twice included (AN-35, AN-37, AN-59) | E3013 | "`v.push(template)` would hand over the caller's player, but `template` only reads it; to put a player of your own in the list write `v.push(copied template)`"; "this function gives `s` away, so it can run only once, and `call_twice` may run it twice; if `call_twice` runs it once, declare the parameter `owned f: once fn()`; otherwise hand over a copy each time: `fn(): consume(copied s)`"; "`call_once` uses `f` up, so it must hold it: write `owned f: once fn()`" |
| a call through a stored function value from a name that may not change its holder (R12; AN-53, AN-62) | E2105 (the first line), E3023 (the name) | "`click` calls the handlers stored in `this.handlers`, which may change their own state; say so on its first line, `fn click() changes handlers:`, and walk them with `for ref mut h in this.handlers:`"; "`cb` changes its own copy of `score` each time it runs, and `run` takes it read-only; write `fn run(ref mut f: fn()):` and call `run(ref mut cb)`" |
| a `set by` link assigned, or its holder lent as a changeable list (R11; AN-63) | E2109 | "`parent` is set by `children`; to give `leaf` a parent, write `root.children.push(leaf)`"; "`children` keeps every child's `parent` in step, so it is changed only by its own methods; pass it read-only, or write the change with `push`, `insert`, `remove`" |
| `virtual` in a class that cannot be subclassed (AN-70) | E2112 | "`Door` cannot be subclassed, so `virtual` on `on_update` changes nothing; to let subclasses replace it write `open class Door:`, or remove `virtual`" |
| an override that changes more than the method it overrides says (R5; AN-70) | E2113 | "`SwingDoor.on_update` changes `angle`, but it overrides `Door.on_update`, whose first line changes nothing, and callers of `Door` rely on that; write `virtual fn on_update(dt: float) ref mut this:` in `Door`" |
| `thread_shared` on a value not created `thread_shared` (R3; AN-71, AN-75) | E7017 | "`names` is `share_and_keep`'s own object, which it may change; to share it between threads, create it shared: `names = thread_shared Names()`" |
| a `thread_shared` value changed directly (R9; AN-71) | E7014 | "`names` is shared between threads (line 17), so no owner changes it directly; keep the list in a field `guarded by` a lock and change it inside `with names.lock:`" |
| an unscoped thread or a kept callback using a name with no capture word (R15; AN-71, AN-78) | E7004 | "this thread may outlive `share_plain`, so it cannot use `share_plain`'s `names`; say what the thread holds: `fn() with owned names:` (handed over), or create it shared, `names = thread_shared Names()`, and write `fn() with thread_shared names:` (one more owner)" |
| a one-thread owner or a lent value reaching another thread (R15; AN-78) | E7015 | "C may call this function from any thread (`threads(any)`), and `shared` owners stay on one; create the game shared between threads, `game = thread_shared Game()`, take it as `game: thread_shared Game` in `setup`, write `with thread_shared game`, and, since `add_point` changes `score`, declare `score: Atomic[i32]` and write `this.score.fetch_add(1)` in `add_point`" |
| two tasks changing one place (R15; AN-71, AN-74) | E7007 | "two tasks change `stats.total` (line 19); guard it: `total: int guarded by lock`, and change it inside `with this.lock:`; or make it `Atomic[int]`" |
| a thread-bound value sent to another thread (R15) | E7008 | "`g` is the guard of `m`'s lock, which belongs to the thread that took it; take the lock inside the task: `with g = m.lock():`" |
| chunks of a parallel loop that may write one place (R15; AN-74, AN-84) | E7010 | "`players` holds `shared Player` objects, and one object may be in the list twice, so two chunks could change it at once; give each object one owner (`Array[Player]`), or change it under its lock (`hp: int guarded by lock`)" |
| a module variable of the main thread reached from another thread (R15; AN-76, AN-84) | E7012 | "`tick` changes `FRAME`, a module variable of the main thread, and this thread runs beside it; guard it: `static mut FRAME: int guarded by FRAME_LOCK` and change it inside `with FRAME_LOCK:`, or make it `Atomic[int]`" |
| a guarded field used without its lock (R15; AN-74) | E7013 | "`words` is guarded by `lock`; change it inside `with stats.lock:`" |
| a `@parallel` loop over something it cannot split (R15; AN-84) | E7016 | "`@parallel` splits a range or a list into chunks, and `while i < idx.len():` (line 4) in `place` is neither; instead of `i = 0`, the `while` line and `i += 1`, write `for i in 0..idx.len():`" |
| an unsafe operation outside an `unsafe:` block, an `unsafe fn` body included (R16; AN-79) | E3100 | "`read_u32_le` is an `unsafe fn`, which says its callers must vouch; the read at line 4 is this function's own unsafe step: put it in an `unsafe:` block with a `# SAFETY:` note saying why it holds here" |
| a run-time alias check inside `@no_alias_checks` (R16; AN-81) | E4030 | "`step` is `@no_alias_checks`, and `collide(ref mut bodies[i], ref mut bodies[j])` (line 3) compares `i` and `j` while running; prove them different (`if i != j:` around the call), or remove `@no_alias_checks`" |
| the removed words (AN-40, AN-45, AN-47, AN-49, AN-54, AN-65, AN-80 to AN-83) | E1010, E0104 | each with the line to write: `owned T` for `Box`, `shared T` for `Shared`, `thread_shared T` for `SyncShared`, `weak T` for `Weak`, `if p = w:` for `upgrade`, `copied x` for `.clone()`, `StrView` for `str`, `CStrView` for `cstr`, `ref mut T` for `Mut[T]`, `from x` for `@borrows`, `@no_alias_checks` for `@static_safe`, `@fp(allow_fma)` for `@fp(contract)`, the capture list for `owned fn`, `cast` for `as?` and `as!`, `this` for `self`, `ref mut this` for `mut self`, `owned this` for `owned self`; `RefCell` and `@sync` with the lines of AN-49 and AN-80; `@no_runtime_checks` with AN-82's |

Warned, not refused: an `unsafe fn` with no `@safety` (L3015: "say what callers of `read_u32_le` must
make true: `@safety("p points to four readable bytes")`"); an `unsafe:` block with no `# SAFETY:` note
(W3012, today's message); a `# SAFETY:` note on an `unsafe fn` line (W3013: "`read_u32_le` says what
its callers must make true with `@safety`; a `# SAFETY:` note says why one block is sound and goes above
each `unsafe:` block inside it: remove this line"); a cycle of `shared` owners (L3001: "the handler is
kept by `ui` and keeps `ui`; capture it weakly, `with weak ui`, or take the Ui as the handler's
parameter").

## 4. Error codes, one meaning each

"Today's" means the meaning today's compiler gives the code. Every new code was checked free: none of
`E0114`, `E0115`, `E0116`, `E0117`, `E2106`, `E2107`, `E2108`, `E2109`, `E2112`, `E2113`, `E2114`,
`E2115`, `E3028` appears in the compiler's sources (`compiler/**/*.rs`) or in the spec
(`docs/spec-source/Ember_v0.9.10_Hardened_35.md`) (`grep -w`; output in
`target/redesign4/01-rule-writer/free_codes.txt`).

| code | means |
|---|---|
| E0005 | `async` is reserved (today's) |
| E0104 | an attribute that is gone, with the one to write (`@sync`, `@static_safe`, `@no_runtime_checks`, `@fp(contract)`) |
| E0109 | `owned` before a value or before `fn` |
| E0113 | `mut` alone on a name: a pattern name, a parameter, a lambda parameter, `this` (`static mut` is a declaration and is not this) |
| **E0114** (new) | `sharedref` not followed by `mut` |
| **E0115** (new) | `Some(...)` in an `if` or `while` header |
| **E0116** (new) | in a list of names declared without types, a name with no word after a name with a word |
| **E0117** (new) | `ref` or `ref mut` written in a parameter's type instead of on its name |
| E1010 | a name the program may not use here: not in scope (`this` outside a method or in a `static fn`), gone from Ember (`self`, `Box`, `RefCell`, `Sync` as a property), or written with a word instead (`a.clone()` is `copied a`) |
| E1030 | a second `main`: statements at file level in a file that has `fn main` (today's) |
| E2036 | this pattern always matches: an `if` or `while` opening whose right side always holds a value (today's) |
| E2085 | `owned` where the type is known to be `Copy` (a parameter, a capture) |
| E2103 | `this` given another object in a class method |
| E2105 | a `changes` or `reads` clause, or its absence, does not match the body, the calls it makes included |
| **E2106** (new) | an `if` or `while` opening that names a name already in use |
| **E2107** (new) | `ref mut this` or `owned this` together with `reads`, `changes` or each other |
| **E2108** (new) | `ref mut this` or `owned this` on `init` |
| **E2109** (new) | a `set by` link assigned by the program, or its holder field lent as a changeable list |
| **E2112** (new) | `virtual` in a class that cannot be subclassed (today the warning W2111) |
| **E2113** (new) | an override or implementation whose first line lets it change more than the method it overrides or implements |
| **E2114** (new) | `owned` on a class type |
| **E2115** (new) | `static fn` outside a type body |
| E2173 | a bare interface name used as a type |
| E2200 | a type that contains itself without `owned` |
| E2221 | a view into a generator's own frame held across `yield` (today's) |
| E3013 | a hand-over out of a name or place that does not own the value (a plain parameter's value; a function value's own capture where it may run twice; a plain `once fn()` parameter) |
| E3020 | a list changed while a `for` loop walks it: directly, through a generator the loop consumes, or through a call that may change it, a call that may run a function value included; a removal inside a counting loop |
| E3021 | a place read through one name while another name may change it: an argument that may reach into a changeable argument of the same call; a `ref mut` name's place read through another name while it is still used (today's meaning, "a shared and a mutable borrow overlap") |
| E3022 | one place changed through two names: two changeable arguments of one call naming one place; a `ref mut` name's place changed through another name while it is still used (today's meaning, "two mutable borrows of the same place") |
| E3023 | a change the name does not allow: a change through a name that only reads (a plain parameter, a loop, pattern, opened or part name, a value made when compiling); `=` on a name that keeps its place for its scope (a parameter, a loop, pattern or opened name, a `ref mut` name given a new view); a `ref mut this` method called on a read-only name |
| E3026 | a function value that may outlive a place it uses and does not own (today's) |
| E3027 | `ref mut` before an argument that is not a place this function may change (today's meaning, "a `mut` argument is not a mutable place") |
| **E3028** (new) | a call's word does not match its parameter: no `ref mut` before an argument for a `ref mut` parameter, or `ref mut` before one for a parameter that does not say it |
| E3040 | a name used after it was handed over or ended |
| E3060 | a view used after what it points into was ended, handed over, replaced or reshaped: a position after a removal or reorder; an address after a reshape; a stored generator after its list changed; a view put in through `ref mut` after one of the call's arguments changed |
| E3062 | a result that points into something its first line's `from` does not name (today's) |
| E3063 | a view, or a value holding views, kept or returned where it would outlive what it points into: a view of a temporary, a view out of a lock's `with` block, a view of the callee's own locals put into the caller's view through `ref mut`, a kept view outliving its target |
| E3100 | an unsafe operation outside an `unsafe:` block, an `unsafe fn` body included |
| E4030 | a run-time alias check (R4's index compare, `assert_disjoint`) inside `@no_alias_checks` |
| E5013 | a variadic further argument C would promote (was E5015) |
| E7004 | an unscoped thread or a kept callback uses a name with no capture word |
| E7007 | two tasks of one scope change one place other than by a reduction or under a lock |
| E7008 | a thread-bound value (a guard, a `Scope`, a raw pointer, a `threads(creator)` handle) sent to another thread |
| E7010 | a parallel loop's chunks may write one place |
| E7012 | a `static mut` not `guarded by` a lock reached from another thread |
| E7013 | a `guarded by` field used without holding its lock |
| E7014 | a `thread_shared` value changed directly |
| E7015 | a one-thread owner (`shared`, a `weak` link of one) or a lent value reaches another thread |
| E7016 | a `@parallel` loop over something it cannot split into chunks: neither a range nor a list walked by element |
| E7017 | `thread_shared` on a name whose value was not created `thread_shared` |
| L3001 | a cycle of `shared` owners (a warning: a leak, not a memory error) |
| L3015 | an `unsafe fn` with no `@safety` (every one, not only `pub`) |
| W3012 | an `unsafe:` block with no `# SAFETY:` note (today's) |
| W3013 | a `# SAFETY:` note on an `unsafe fn` line, where `@safety` says it |

Retired: E7001, E7003, E7005 (with `@sync`), and run 3's proposed E3045 (its case is E3023).
