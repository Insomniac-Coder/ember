---

# Part VIII — Classes, Shared Owners and Links

Classes give Ember the object graphs Python programmers expect (objects with identity, parent links,
observers, objects several places own) with deterministic destruction and no garbage collector. A class
object has one owner unless a line says `shares`, as every value does (`[OWN-1]`); a link that does not
keep its target alive is a `weakref`, which the program sets like any field. Everything here is checked
when compiling, except what §VIII.2 and §VIII.5 name: a count, a count's limit, and the test a `weakref`
link makes when it is opened.

```ember
class Node:
    name: String
    children: Array[Node] = []
    parent: weakref Node = None

    fn add(owns child: Node):
        child.parent = weakref this
        this.children.push(child)

fn main():
    root = Node("root")
    root.add(Node("leaf"))
    for c in root.children:
        if c.parent exists as p:
            println(f"{c.name} -> {p.name}")
        else:
            println("orphan")
```

## VIII.1 Object representation

Every class object, and every value held `shares`, `shares mut` or `thread_shared`, is one heap block
with a header:

```text
offset  size  field
0       4     count   u32   its owners
4       4     type    u32   its type index: its type information, and the interface tables reached
                            through it
8       4     weak    u32   its `weakref` links, plus one for all its owners; present only in a value
                            a `weakref` link may reach
8 or 12 …     base-class fields, then own fields (or the shared value), naturally aligned
```

A `thread_shared` value's header (`[RC-4]`):

```text
offset  size  field
0       4     home    u32   the owners its making thread holds; read and written only by that thread,
                            with plain instructions
4       4     type    u32   its type index
8       4     weak    u32   its `weakref` links, plus one for all its owners; atomic
12      4                   unused
16      8     away    i64   atomic: the owners the other threads hold, times 8, and three marks: handed
                            over (the making thread counts here too), queued (it waits in the making
                            thread's queue), ended
24      8     next    ptr   its place in the making thread's queue
32      …     the shared value, naturally aligned
```

* `[OBJ-1]` *(changed in 0.9.9)* *(changed in 0.9.15)* The header is 8 bytes: a 32-bit count and a
  32-bit type index. A value a `weakref` link may reach has a third 32-bit number there, its weak
  count, making 12 bytes: every object of a class family (the classes that share one root base class)
  of which some `weakref` link in the program may name an object, and every shared value of a type some
  `weakref` link may name. A `thread_shared` value's header is 32 bytes (verdict 101): its making
  thread's count of its owners, the type index, the weak count, the other threads' count with its
  marks, and its place in the making thread's queue (`[RC-4]`). Which thread made a value is read from
  the header of the memory chunk its block came from, written once when the chunk is made, never from
  the value. The header is part of the runtime ABI (`[VER-4]`). Foreign code never reads it; it calls
  `ember_rt`. An object carries nothing else for the memory model: no access words, and no flags but a
  `thread_shared` value's three marks.
* `[OBJ-2]` *(changed in 0.9.10)* *(changed in 0.9.15)* An owner, a link or a view of a class object
  is one pointer, to offset 0. An object's interface tables are reached through its type index, so a
  `dyn I` value of a class object, and an `Option` of one, is one pointer wide. A bare interface name in
  a type position (`s: Shape`, `Array[Shape]`) does not say who owns the value, and is `E2173`:
  "`Shape` is an interface, so `Array[Shape]` does not say who owns the shapes; say who owns them: write
  `shapes: Array[owns dyn Shape]` in place of line 19 (the list is their only owner), and
  `scene = Scene(shapes=[owns Square(side=2)])` in place of line 22" (with `Array[shares dyn Shape]`,
  other names own them too, and each shape is made `shares`). `dyn I` as a parameter's type is a
  read-only view (`ref mut v: dyn I` may change it).
* `[OBJ-3]` *(changed in 0.9.15)* The weak count starts at 1 on behalf of all owners. When the count
  of owners reaches 0 the object is ended (`[WK-3]`): its `drop` chain and its field drops run, and the
  weak count is decremented; the block is freed when the weak count reaches 0, which for a value no
  `weakref` link may reach is at once.
* `[OBJ-4]` *(changed in 0.9.10)* *(changed in 0.9.15)* Objects are allocated through the runtime
  allocator (`[RT-1]`), or from the memory manager the class names with `using` (`[ALC-1]`), except as
  `[OPT-1]` permits.
* `[OBJ-5]` *(changed in 0.9.15)* **An ended object is never brought back.** Inside a `drop`, and in
  everything it calls (by their summaries, `[EFF-1]`), no owner is made of the object being ended: an
  owner made from `this` (`shares this`, `shares mut this`, `thread_shared this`) is `E3016`, as storing
  `this` anywhere that outlives the call is (`[CLS-7a]`): "`drop` ends this `Player`; an owner made from
  it would point at a freed object; keep a copy of what you need: `NAMES.push(copy of this.name)`".
  `weakref this` stays allowed: the weak count keeps the block, and opening the link finds the object
  ended (`[WK-3]`). Nothing is checked for it while the program runs.

## VIII.2 Owners and counts

* `[RC-1]` *(changed in 0.9.15)* **Several owners.** `shares x` makes one more owner of what `x`
  names, on this thread, that reads it; `shares mut x` one more that may also change it; and
  `thread_shared x` one more on any thread (`[OWN-13]`). A value made shared lives in a block of its own
  with the header of `[OBJ-1]` (a class object already does), and its owners are counted: making an
  owner adds one to the count, ending one subtracts one, and the subtraction that reaches zero ends the
  value. A value with one owner is not counted: its owner ends it. **Only a whole value is shared**
  (verdict 103): `shares x`, `shares mut x` and `thread_shared x` are written before a new value, a
  local of this function (which becomes one of the owners) or a name that is already such an owner. A
  place that has one owner and is not a local of this function, a list element (`shares mut players[1]`
  of an `Array[Player]`), a field (`shares this.items`) or a plain parameter (`fn keep(p: Player):
  LIST.push(shares p)`), is `E2122`: "`players[1]` has one owner, the list `players`, and no second
  owner can be made of one element; to let other names own the players, make the list share them from
  the start: write `players = [shares mut Player(), shares mut Player()]` in place of line 15". So a
  value in a one-owner place is ended by that place at once, with no count to test. **A count is checked
  before it rolls over:** before each +1 of an owner count it is compared with its maximum, and at the maximum the
  program stops, in every profile: "too many owners of a `Player` (4,294,967,295)" (`[RT-7]`), as
  Rust's `Rc` and `Arc` stop (a `thread_shared` count stops at the lower limit of `[RC-4]`). That is one
  compare per added owner and no extra memory.
* `[RC-7]` *(new in 0.9.15)* **Changing a shared value.** A value made shared is changed by its owners
  that may change it: an owner made with `shares mut`, and the name it was first shared from, which
  keeps what it could do. After `other = shares mut items` (or `other = shares items`), `items` may
  still change the list if it could before, and so may `other`; a `shares` owner only reads it
  (`E3023`, `[OWN-13]`). Any `ref mut` name of it and any holder of a `sharedref mut` link to it may
  change it too. That is safe as it stands, with nothing checked while the program runs: one thread
  runs one line at a time; a list does not change while a loop walks it, through whichever of its names
  (`[BRW-18]`); a view of it is checked against every name that may reach it (`[BRW-17]`); and a
  `shares` value never reaches another thread (`E7015`), which is what `thread_shared`, changed through a
  lock, is for. A `thread_shared` value is never changed directly by any owner, only through `Atomic`,
  `Mutex`, `RwLock`, a channel, or a `guarded by` field under its lock (`E7014`: "`names` is shared
  between threads (line 5), so no owner changes it directly, and line 6 pushes to its list; fill the
  list where the object is made: write `names = thread_shared Names(items=[copy of s])` instead of lines 5
  and 6"). How scoped tasks and threads may reach a shared value or a `sharedref mut` link is §XI.3's.
  An object or a shared value carries no access words and no flags for any of this.
* `[RC-2]` *(changed in 0.9.15)* **No count where no owner is made.** No count operation is emitted in
  the cases `[RC-2a]` to `[RC-2e]`; each is tested by counting the count operations in the emitted C.
* `[RC-2a]` *(changed in 0.9.15)* Passing a shared value to a parameter that is not `owns`
  (`x: shares T` reads the caller's shared value for the call).
* `[RC-2b]` *(new in 0.9.9)* *(changed in 0.9.15)* A view of a shared value or an object: a name for a
  part, a loop name, a name opened from a `shares` link, a parameter. Only `[RC-5]` point 2 counts for a
  view.
* `[RC-2c]` *(new in 0.9.9)* *(changed in 0.9.15)* A +1 immediately followed by a -1 of the same count with no call or store
  between.
* `[RC-2d]` *(changed in 0.9.15)* An owner handed over (`b = a`, an `owns` argument, `return a`, a value
  stored into a field from a temporary): it moves, and nothing is counted.
* `[RC-2e]` *(changed in 0.9.15)* The loop names of a `for` over a list of `shares` owners, which are
  views, unless the body makes an owner of one (`shares p`).
* `[RC-3]` *(changed in 0.9.9)* *(changed in 0.9.15)* Further removals of count operations are allowed
  only when meaning is preserved (`[PHIL-5]`). A value is ended when its last owner ends as the source
  says (an owner's scope ending, its replacement, `mem.drop`) and never earlier: a removal takes away a
  +1 and a -1 only when it moves no `drop`, no outcome of an `exists as` opening and no foreign release
  across that point. A surviving owner in a field, a collection or a function value's state keeps the
  value alive. The last use of a view ends the view, not the owner (`[BRW-2]`) (ODR-063). `unsafe` code
  that keeps a raw pointer into an object keeps an owner of it alive for as long as it uses the pointer;
  `with h:` and `mem.keep_alive(h)` state that where no Safe use does.
* `[RC-4]` *(changed in 0.9.9)* *(changed in 0.9.15)* The counts of a value that is not
  `thread_shared` are changed with plain adds and subtracts, and never cross threads (`E7015`).
  **A `thread_shared` value's owners are counted in two numbers** (verdict 101; biased counting, Choi,
  Shull and Torrellas, 2018):
  1. **Home and away.** The thread that made the value counts the owners it holds in the value's home
     count, with plain adds and subtracts; every other thread counts its owners in the away count, with
     atomic ones (`[RT-8]`). The value lives while the two add up to more than zero. Only the making
     thread reads both.
  2. **The hand-over to the making thread.** When another thread ends an owner the making thread
     counted, the away count goes below zero: that thread puts the value in the making thread's queue,
     marking it queued in the same atomic step as its subtract, and only the making thread then ends
     it. The making thread empties its queue at each of its check points: it adds each value's home
     count into its away count, and ends the value when the sum is zero. A queued value is ended by
     nothing else.
  3. **Check points.** The compiler places them: at every loop's back edge, before every call that may
     block the thread (waiting for a lock, `join`, a channel's receive, a sleep) and when the thread
     ends. A value whose making thread has ended is ended at once by the thread that ends its last
     owner.
  4. **Handed over at once.** A thread that stores an owner of a `thread_shared` value where other
     threads can reach it (into a lock-guarded `thread_shared` container, through a channel, into the
     capture of a thread it spawns or of a callback C keeps; every such place is known when compiling)
     first adds its own count of the value into the away count and marks the value handed over; from
     then on every thread, the making thread included, counts in the away count, and a later end on
     another thread never goes through the queue.
  5. **So a value's end is delayed only** when its last owner ends on a thread that did not make it
     and the value was not handed over at a place of point 4 (it travelled inside another object): it
     is ended at its making thread's next check point. Until then it is intact, and a `weakref` link
     opened on another thread may still find it (`[WK-3]`).

  The weak count of a `thread_shared` value is atomic. The away count, and the weak count, are raised
  with one atomic add and their limit is checked on the value the add returns: when the count before
  the add was 2,147,483,647 or more, the program stops ("too many owners of a `Player`
  (2,147,483,647)"; "too many `weakref` links to a `Player` (2,147,483,647)"). That limit is half the
  count's range, so threads racing past it each stop before the count can roll over, whatever their
  number. The home count, and a count that is not `thread_shared`, keep the exact compare of `[RC-1]`
  and `[WK-11]`.
* `[RC-5]` *(changed in 0.9.15)* **A view of an object reached through a link.** A view of an object
  reached through a `shares`, `shares mut`, `thread_shared` or `weakref` link (a name opened from a link,
  `if cam.target exists as p:`; a view made through a `shares mut` field, `t = ref mut ws[i].target`; a
  parameter `x: shares T`, whose life is the call) must not outlive the object. It is checked when
  compiling, and nothing runs for it while the program runs, except for one kind of object (point 2):
  1. **No line or call in the view's life may end an object of that type**: no write to a place that
     owns one (`T`, `owns T`, `shares T`, `shares mut T`, `thread_shared T`, and a place of a function
     type or of `owns dyn I` whose value's state owns one: `h.keep = fn(): pass` ends the function
     stored there before and the node its `owns n` held), no removal from a list that owns them, no end
     of a local that owns one, and no call that may do so, worked out from its code (`[EFF-1]`):
     `remove_all(ref mut players)`, given the list that owns the players; a call through a function
     value that may run `fn() owns players: players.clear()`; the end of an object whose `drop` may end
     one (`[DRP-8]`). After such a line or call the view is not used (`E3060`: "`remove_all(ref mut
     players)` (line 12) may destroy the player `p` reads (`if cam.target exists as p:`, line 10), and
     `p` is used on line 13; use `p` before line 12, or open the link again after it: write
     `if cam.target exists as q:` above line 13, indent line 13 under it and use `q` there"; "`reset()`
     (line 7) may end `Player` objects (it runs the function stored on line 3, which clears the players
     it owns), and `p` reads one (`if cam.target exists as p:`, line 5) and is used on line 8; use `p`
     before line 7, or open the link again after it: write `if cam.target exists as q:` above line 8,
     indent line 8 under it and use `q` there"; "`h.keep = fn(): pass` (line 11) ends the function
     stored there on line 9 and the node it owns (`owns n`), and `p` reads that node
     (`if w exists as p:`, line 10) and is used on line 12; use `p` before line 11"). A one-owner object
     lent read-only cannot be ended while it is lent.
  2. A view of a `thread_shared` object reached through a `weakref` link holds one count for its life,
     as Rust's `Weak::upgrade` does (`[WK-12]`): another thread may end the last owner at any moment,
     which no line of this thread can be checked against. On the thread that made the object, until it
     is handed over (`[RC-4]` point 4), that count is a plain add to its home count; on another thread,
     one atomic add to its away count.
  Across a call that may change an object of the view's type (worked out from the functions it may run,
  `[EFF-1]`), a read-only view of such an object does not live at all (`[BRW-1]`, `E3021`): a walk
  across such a call keeps a `weakref` link and tests it at every step (`[WK-18]`). Where a call fits
  both this and point 1 (it may change and end the object a name opened from a link reads), the message
  shown is the one whose fix compiles (`[DIA-28]`): point 1's `E3060` where using the name before the
  call, or opening the link again after it, compiles (`println(p.hp)` after `reset()`, which may run
  `fn() owns players: players.clear()`), and `[WK-18]`'s `E3021` for a walk, where every view of the walk
  would need opening again.
* `[RC-6]` *(changed in 0.9.15)* `ember inspect` lists every count operation that survives inside a
  loop, with the reason it could not be removed, because a surviving count operation blocks
  vectorisation.

## VIII.3 Changes through several names

An object may be reached through several names: its owner, its `shares` owners, `sharedref mut` and
`weakref` links, views. Every rule of §VII.4 and §VII.5 applies to an object's fields as to a struct
value's; this section says what links add. Nothing is checked while the program runs: an object holds
no access words, and no change of a field begins or ends anything at run time.

* `[EXC-1]` *(changed in 0.9.9)* *(changed in 0.9.15)* A change of an object's field while a read-only
  view into that field, made through another name that may reach the object, is still used is refused
  when compiling (`E3021`, at the change), and a change that may reshape it ends the view instead
  (`E3060`, at its later use), by `[BRW-1]`'s promise; a change through another name while a `ref mut`
  name of the field is still used is `E3022`. There is no setting that turns these rules off
  (`[PHIL-13]`).
* `[EXC-2]` *(changed in 0.9.9)* *(changed in 0.9.15)* A read of an object's field through another
  name while a `ref mut` name of that field is still used is `E3021` (`[BRW-1]`).
* `[EXC-3]` *(changed in 0.9.9)* *(changed in 0.9.15)* **Which names may reach one object.** Two paths
  may reach the same object when they lead to objects of one class family (the classes that share one
  root base class) and at least one goes through a link or a kept view, unless the compiler knows they
  differ: different classes, a fresh object, or the line inside `if a is not b:`. A possible overlap the
  compiler cannot rule out is refused (`E3022` for a change, `E3021` for a read), with the test that
  proves them apart as the line to write ("`a` is the only name for its turret while `hit` runs, and
  `t`, opened from `a.target`, may be that turret; write `if t is not a:` above line 20 and indent line
  20 under it").
* `[EXC-19]` *(new in 0.9.9)* *(changed in 0.9.15)* **Fields are separate places.** A view of one field
  of an object and a change of another field never conflict: `for c in this.children:
  this.log.push(c.name)` is accepted. What a method's call touches is the fields its body touches
  (`[BRW-10]`).
* `[EXC-16]` *(new in 0.9.9)* *(changed in 0.9.15)* Giving a field that owns memory a new value
  (`h.items = []`, `h.name = other`) is a reshape: inside a `for` loop over the same field it is
  `E3020` (`[BRW-18]`), and a view into the old value is not used after it (`E3060`, `[BRW-17]`).
* `[EXC-17]` *(new in 0.9.9)* *(changed in 0.9.15)* Writing a number or a `bool` (a field of a `Copy`
  type) moves and frees nothing, so it is not a reshape and ends no view; a method that writes
  `this.health` through a link that may lead to its own object is allowed (`[EXC-15]`). While a
  read-only view of that field is still used, a write through another name that may reach it is
  `E3021` (`[BRW-1]`). Through a `sharedref mut` link, any holder may write it at any time on one thread
  (`[LT-46]`); such a view gets no alias fact another holder could break (`[SIMD-3]`).
* `[EXC-15]` *(new in 0.9.9)* *(changed in 0.9.15)* **A method and its object.** Inside a method its
  object is `this`. What a method does to it is worked out from its body (`[BRW-10]`): a method whose
  body changes `this` may change it, one whose body neither changes nor ends anything of `this` only
  reads it, and `owns this` after the parameters takes the object over. A method that may change its
  object is not called through a name that only reads it (`E3023`: "`heal` changes the player
  (`this.hp += 10`, its line 3), and `p` only reads it; write `for ref mut p in players:` in place of
  line 7"). That a method may change its object promises nothing about other names: the same object may
  be reached and changed through a link while the method runs (a turret aimed at itself:
  `if this.target exists as ref mut t: t.health -= 1`, accepted with no check), so `this` is never the
  only name of its object and is never compiled with noalias (`[BRW-22]`). `[BRW-1]`, `[BRW-17]` and
  `[BRW-18]` keep every view and every walk safe, and a call whose object is part of what the method may
  end before its last use of `this` is `E3031` (`[BRW-21]`). `init` sets the fields of the object it
  builds through `this`, as `[CLS-2]` says.

## VIII.4 Inheritance and dispatch

* `[DSP-1]` *(changed in 0.9.15)* A call is dispatched statically when the object's static type is a final class or the
  method is not `virtual`.
* `[DSP-2]` A virtual call loads its slot from the object's type table; slots are assigned in
  declaration order, base first, and an `override` reuses the base's slot.
* `[DSP-3]` *(changed in 0.9.15)* A call through a `dyn I` value of a class object finds the
  interface's table through the type index and caches the lookup for repeated calls on one value.
* `[DSP-4]` *(changed in 0.9.15)* **Kind tests.** A value of an interface type (`dyn I`, `owns dyn I`,
  `shares dyn I`) or of a base class is tested for its kind with `is`: `if s is Circle as c:` runs its
  block when `s` is a `Circle` and names it `c` there; `if s is Circle:` tests without a name; other
  kinds are tested with `elif s is Square as q:` and `else:`, which is how a choice among kinds is
  written (there is no "maybe a `Circle`" value to `match` on). The name is declared with its words, as
  a name opened with `exists as` is (`[BRW-14]`): plain `c` is a read-only view, and since a value
  tested in its place stays a place, `if this.pools[k] is Pool[T] as ref mut pool:` names a view of the
  registry's pool that may change it, where the function may change the place. `is` before a type name
  means "is of this kind"; before a value it means "the same object" (`a is b`, which compares
  addresses) or tests for absence (`x is None`). A test that always passes is `E2036` ("`s` is a
  `Circle` already, so this `if` would always run; to name it write `c = s` in place of the `if` line,
  and use `c` in its block, moved out one level"); a test of a type that does not implement the
  interface or derive from the base class, which can never pass, is `E2037` ("`s` is a `Shape`, which
  can never be a `Path`; remove the test: the `if` line and its block"); a name already in use is
  `E2106` ("`s` already names a value (line 1); `as` declares a new name: write `if s is Circle as c:`,
  and `c.r` for `s.r` in its block"). Each test is one type-id compare.
* `[DSP-6]` *(new in 0.9.15)* **Casts.** `cast s as Circle` gives the `Circle` and stops the program if
  `s` is not one, in every profile: "`cast s as Circle` on line 9: `s` is a `Square`, not a `Circle`".
  A cast of a place is a place: the name bound from it is read-only unless written `ref mut`
  (`pool = ref mut cast this.pools[k] as Pool[T]`), and `ref mut` taken through a plain one
  (`return ref mut pool.dense[pool.sparse[e]]`) is `E3027` ("`pool` only reads the pool in `this.pools`,
  so `ref mut pool.dense[...]` cannot be taken through it; name it so it may change:
  `pool = ref mut cast this.pools[k] as Pool[T]`"). Each cast is one type-id compare.
* `[DSP-5]` *(changed in 0.9.15)* With the whole program visible, a virtual call with exactly one
  reachable implementation may become a direct call; `--emit-optimization-report` lists each. What a
  call through a `virtual` or interface method may change, read and end is worked out from every
  override and implementation the program has (`[EFF-1]`).

## VIII.5 `weakref` links, parent links and loops of owners

* `[WK-1]` *(changed in 0.9.15)* Ember has no cycle collector: a loop of owners, objects that own each
  other through `shares`, `shares mut` or `thread_shared` fields, through the state of function values
  they hold, or through channel ends, would keep its objects alive for ever. So a loop of owners is
  allowed only where the compiler proves it is opened again before its objects lose their last owner
  from outside, and is otherwise refused (`[WK-6]`); a link back that does not keep its target alive is
  a `weakref` (`[WK-11]`).
* `[WK-11]` *(changed in 0.9.9)* *(changed in 0.9.15)* **`weakref` links.** `weakref T` is a link that
  does not keep its target alive. Its target lives on its own: a class object, or a value held `shares`
  or `thread_shared`. A link may hold no target (`parent: weakref Node = None`). `weakref x` makes a link
  to what `x` names; before a `weakref` link it makes another link to the same target
  (`a = weakref this.keep`). A `weakref` link is not `Copy`, and `copy of` before one is `E2116`
  (`[OWN-8]`). Every `weakref` link holds one weak count on its target: making one adds one, and its
  end, or giving it another target, subtracts one, as Rust's `Weak` (`[WK-18]` gives the one place a
  proved region keeps the block instead); before each +1 the count is compared with its maximum, and at
  the maximum the program stops, in every profile: "too many `weakref` links to a `Player`
  (4,294,967,295)" (`[RC-4]` for a `thread_shared` target). That count is a number in the target's
  header that no name reads, so making or ending a link is not a change of the target: any name may
  make one (`up = weakref p` from a read-only `p`; `child.parent = weakref this` in a method that
  otherwise changes only `this.children`). Making, ending or opening a link touches the target's
  header, so a scoped task does it only for a `thread_shared` target (§XI.3).
* `[WK-16]` *(new in 0.9.15)* A `weakref` of a part (a field, or an element stored inline), whose target
  does not live on its own, is `E2119`: "a `weakref` link needs an object that lives on its own; link to
  the player: `cam.target = weakref player`". A `weakref` link used as its target without opening it,
  and an object stored into a `weakref` place without `weakref`, are `E2020`: "`cam.target` may be gone;
  open it: `if cam.target exists as p:`"; "`=` alone would hand `player` over; write
  `cam.target = weakref player`".
* `[WK-12]` *(changed in 0.9.9)* *(changed in 0.9.15)* **Opening a link.** A `weakref` link is read by
  opening it with `exists as` (`[GRM-44]`): `if cam.target exists as p:` tests the target's header
  (does the object still live?) and runs its block, with `p` a view of the object, when it does;
  `else:` runs when the link holds nothing or its target is ended. Each opening is one test of the
  header, except a parent-link read proved by `[WK-17]`. For a `thread_shared` target the opening adds
  one owner only to a value that is not ended, so an object another thread is ending is never brought
  back, and the view holds that count for its life (`[RC-5]` point 2): on the making thread, until the
  value is handed over (`[RC-4]` point 4), it tests that the home and away counts add up to more than
  zero and adds one to the home count; on any other thread it is one atomic add to the away count,
  undone when the value is marked ended.
* `[WK-2]` *(changed in 0.9.15)* An object is ended when its count of owners reaches zero, whatever
  `weakref` links remain; those links then find nothing when opened.
* `[WK-3]` *(changed in 0.9.15)* **An object is ended the moment its last owner lets it go, before its
  `drop` runs**, as C++'s `weak_ptr` and Rust's `Weak` see it: a `weakref` link opened during its
  `drop`, or during the `drop` of anything it owns (a child's `drop` opening `this.parent` while the
  parent is being ended), finds it ended, so the `if ... exists as p:` block is skipped and the
  half-ended object is never read, nor counted again. Ending the link itself (`this.parent = None`)
  stays allowed: it only subtracts the link's weak count in the target's header, which that count keeps
  in memory. A `thread_shared` value whose last owner ends on a thread that did not make it, and that
  was not handed over, is ended at its making thread's next check point (`[RC-4]` point 5), and its
  `drop` runs there.
* `[WK-13]` *(changed in 0.9.15)* A `weakref` link to a value held `shares` names the same block as its
  owners.
* `[WK-14]` *(changed in 0.9.15)* Ember's `weakref` and `shares` never convert to or from C++'s
  `std::weak_ptr`/`std::shared_ptr` (`[SEL-2]`).
* `[WK-17]` *(new in 0.9.15)* **Parent links.** A parent link is an ordinary `weakref` field that names
  the object owning its holder (`parent: weakref Node`), which the program gives its target like any
  field: `child.parent = weakref this`, then `this.children.push(child)`. A read of one is compiled with
  no header test (a plain load and a test for no link) where the compiler proves its target lives; it
  proves it when both of these hold:
  1. **The link follows ownership.** In the package, every line that gives the field a target,
     `x.parent = weakref o`, is followed in the same function, before `x.parent` is read, by a line that
     puts `x` into a place `o` owns alone (`o.children.push(x)`, `o.left = x`); every line that takes
     such an object out of its place without ending it is followed, before that object's link is read
     (there, or in a function it is given to, by that function's summary), by a line that gives the link
     a target the same way or clears it (`c.parent = None`); no list or field holding such objects is
     handed over, replaced or exchanged whole while it holds them; the objects have one owner (not
     `shares`, not `thread_shared`) and their class has no `drop` method of its own; and no other
     package can change the link or those places (`[EFF-4]`). Then an object's parent owns it, so while
     the object lives its parent lives.
  2. **Nothing ends such an object meanwhile.** No line between the point where the object whose link
     is read is known to live (a view of it) and the read can end an object of the link's target type
     (`[RC-5]` point 1).
  A walk up a tree with views (`cur = ref n`, `while cur.parent exists as p:`, `cur = ref p`) is then
  compiled with no test per step; where condition 1 fails, every read tests.
* `[WK-18]` *(new in 0.9.15)* **A walk across a call that may change nodes holds a link and tests it at
  every step.** When a walk calls a function that may change nodes (`this.on_visit(p)`, where the
  program stores in `on_visit: fn(Node)` a function that removes nodes from their parents' lists, and so
  ends them), no view of a node lives across the call (`[RC-5]`): the walk with views is `E3021`
  ("`this.on_visit(p)` (line 21) may end a `Node` (it may run `prune`, stored on line 40, which removes
  a node from its parent's list), and `p` reads a node and is used again on line 23; walk with a link,
  which may outlive its node: instead of `if leaf exists as n:`, `cur = ref n`, the `while` line and
  `cur = ref p`, write `up = weakref leaf`, then `while up exists as n:`, inside it
  `if n.parent exists as p:` with `up = weakref p` and `this.on_visit(p)`, and `else: break`"), and it
  is written with a `weakref` link, which may outlive its node:

```ember,fragment
up = weakref leaf
while up exists as n:            # the node still lives: call it `n`
    if n.parent exists as p:     # its parent, if it has one
        up = weakref p           # the link now names the parent
        this.on_visit(p)         # may end `p`, or a node above it with everything under it
    else:
        break
```

  The call `this.on_visit(p)` itself compiles when no function the program stores in `on_visit` uses its
  node after a line that may end it; one that does is refused where it is stored (`E3031`, `[BRW-21]`).
  **A walk up a tree stops at the first ended parent**: if the call ends `p`, or a node above it with
  everything under it, the link's target is ended and its block is kept by the link's weak count, so
  the next `while up exists as n:` finds it ended and the walk stops; it never visits an ended node, and
  nothing stops the program. **What it costs**: per step, one test of the node's header
  (`while up exists as n:`) and one move of the link's weak count from the node to its parent
  (`up = weakref p`: a subtract and an add); the read of `n.parent`, between the test and the call, needs
  no header test where `[WK-17]`'s condition 1 holds (its condition 2 holds there). **Where proof removes
  part of it**: when the node's class has no `drop` method of its own and its objects are ones no other
  thread can end (not `thread_shared`, and not reachable from a `thread_shared` value), the walk is
  placed in a **deferred-drop region**: an object whose last owner ends during the walk is ended at
  once, as anywhere: the `drop`s of what it owns run then, in their usual order, so the program prints
  what it would with no region, and it and everything it owns are marked ended; only their blocks'
  memory waits, and is freed when the walk ends
  (one add and one subtract of a per-thread counter per region, nothing per step). A link that lives
  only inside the region and is given only objects that live at that moment (`up = weakref p` from the
  view `p`) then holds no weak count, so each step pays only the header test; the first link of each
  walk, `up = weakref leaf`, keeps its count. A walk over objects another thread may end pays the count
  moves. A walk whose visit function changes no node (every function the program stores in `on_visit`
  only reads its node) keeps its views and pays nothing per step.
* `[WK-5]` *(changed in 0.9.15)* **The graph of owners.** The compiler builds a graph of owner edges
  among types, after generic substitution, over the whole program: a field holding a `shares`,
  `shares mut` or `thread_shared` value, or a collection of them; a function value's state, from every
  function the program can put there (`[EFF-1]`); a channel end that its own queue can hold; a
  `Retained[ForeignCallback]` token, to the callback's state, which it keeps for C (`[FFI-23]`). `weakref`
  links, `sharedref mut` links, views and raw pointers are not owner edges. A `let` field, set when its
  object is made, can name only objects that exist already, so it never closes a loop.
* `[WK-6]` *(changed in 0.9.15)* **A loop of owners is refused unless it is proved opened.** A line that
  closes a loop of owners (`a.next = Some(shares mut b)` after `b.next` names `a`) is refused, unless the
  compiler proves that the loop is opened again before its objects lose their last owner from outside
  (`E3032`: "this closes a loop of owners (`a` → `b` → `a`) that nothing opens again before `a` and `b`
  end, so neither node is ever freed; open it before the function ends: `b.next = None`, or make the
  link back `weakref`: `next: Option[weakref Node]`") (a deliberate error example, so the block is not compiled):

```ember,ignore
class Node:
    name: String
    next: Option[shares mut Node] = None

fn make_pair():
    a = shares mut Node(name=String("a"))
    b = shares mut Node(name=String("b"))
    a.next = Some(shares mut b)
    b.next = Some(shares mut a)        # refused: nothing opens the loop before `a` and `b` end
```

  The same function with `b.next = None` before `a` and `b` end is accepted. Nothing runs for it while
  the program runs.
* `[WK-7]` *(changed in 0.9.15)* **What counts as proof.** Proved: a loop made and opened again in code
  the compiler can follow (the same function, the functions it calls), or opened by its owning object (a
  `clear()` it always calls before its last owner ends). Refused: an opening that depends on what is only
  known while running, and a loop handed out of reach (returned, stored where the compiler cannot follow
  it). The same holds for loops through function values' state, `thread_shared` owners and channel
  ends. A `weakref` link back always works. **When a line may close a loop.** A store of an owner
  (`shares`, `shares mut`, `thread_shared`, a value that holds one, or a function value whose state owns
  one) into a place of an object may close a loop when the stored value's type can reach the holder's
  type through owner edges (`[WK-5]`); such a line is `E3032` unless the loop it closes is proved opened
  (above) or one of these proves it closes none, and every implementation applies exactly these
  (XVIII.4): (1) the stored value was made in this function, on the line or earlier, and since then no
  owner has been stored into it or into anything it owns but values that themselves meet (1)
  (`root.children.push(shares mut child)` with `child = shares mut Node(...)` made above it: a tree
  built from the top down closes nothing); (2) the holder was made in this function and no owner of it
  has been stored anywhere yet (only this function's locals hold it), so nothing the stored value
  reaches can own it.
* `[WK-15]` *(new in 0.9.9)* *(changed in 0.9.15)* `ember run` and `ember test` in the `debug` profile
  report the objects still alive when the program exits, by default; `--no-leak-check` turns the report
  off. In Safe Ember nothing leaks through a loop of owners (`[WK-6]`) and nothing ends without its
  clean-up (`[OWN-2]`), so an object in the report was kept by `unsafe` code (`into_raw()`, `[OWN-12]`)
  or by foreign code.
* `[WK-8]` *(changed in 0.9.15)* The run-time report (`[WK-15]`) lists each group of leaked objects that
  own each other, with its classes and fields, and the link to make `weakref`.
* `[WK-4]` *(changed in 0.9.15)* The report names, for each leaked object on a loop of owners, the
  shortest loop through it as a path of `Type.field` edges and the link to make `weakref` (`L3017`).
* `[WK-9]` *(changed in 0.9.15)* `ember explain --cycle <path> <Class[.field]>` explains one class or
  field that can close a loop of owners; `ember inspect --cycle <path>` prints the whole graph. A loop
  through a generic container is shown with its instantiated types, and where no static loop can be
  established the command says so.
* `[CLI-18]` *(changed in 0.9.15)* For both commands `<path>` is a package directory (its manifest and
  imports) or a single `.em` file (`[CLI-4]`), and both analyse exactly the same graph of owners
  (`[WK-5]`), classifying each edge as an owner edge or not and showing the shortest loop when there is
  one.

## VIII.6 Stack promotion

* `[OPT-1]` *(changed in 0.9.10)* *(changed in 0.9.15)* When escape analysis proves that no owner,
  link or view of an object, a `weakref` link included, outlives the function that made it, the compiler
  MAY place the object in the function's frame with the same header and run its `drop` at scope end.
  This cannot be observed: how many times an embedding host's allocation functions (`[FFI-27]`) are
  called is not behaviour.
