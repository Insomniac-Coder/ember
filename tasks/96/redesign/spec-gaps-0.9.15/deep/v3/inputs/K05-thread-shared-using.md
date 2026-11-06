# K05-thread-shared-using (input for method 3; made by deep/v3/make_inputs.py from phase2.json, phase3.json, deep/phase1-groups.json, JOB.md and the part files)

## cluster
**cid:**
K05-thread-shared-using

**title:**
May a thread_shared value take its memory from an arena or other manager?

### members
#### [1]
J5-05

**primary:**
J5-05

**why_together:**
Single question, but it must be answered before K34: its answer decides whether TYP-46's list may be restated as a property, which would bring thread_shared back in.

**severity:**
high

**impact_rank:**
5

**notes:**
Ask before K34 (J2-02/J5-09). If B, OBJ-1's spare header word gets a meaning (verdict 101, fix 1).

## questions
### [1]
**id:**
J5-05

**verdict:**
confirmed

**route:**
owner_odr

**severity:**
high

**confidence:**
0.65

#### rules
##### [1]
TYP-46

##### [2]
ALC-1

##### [3]
OBJ-1

##### [4]
RC-4

##### [5]
LT-4

##### [6]
LT-46

**reason:**
[TYP-46] (p04 433-434) and [ALC-1] allow `using` after `thread_shared`. [OBJ-1] (p08 69-71) reads the making thread only from the header of the runtime's memory chunk (verdict 101, fix 1), and a manager's block has no such header. After a hand-over ([RC-4] point 4), the last owner, and with it `drop` and the manager's `deallocate`, may run on another thread. [LT-4]'s instance word is a kept `sharedref mut` link, which may be changed on two threads only under a lock ([LT-46]). No rule refuses this or says how it runs (grep 'memory chunk', 'making thread': OBJ-1 only). Whether it compiles is the owner's question.

**title:**
May a `thread_shared` value take its memory from an arena or other manager?

**problem:**
The list of types that `using` may follow names `thread_shared`, so on paper `stats = thread_shared Stats() using level` is allowed. But a `thread_shared` value finds the thread that made it from the header of the runtime's own memory chunk, and a block from an arena has no such header. Once the value is handed to another thread, its last owner, and so the arena's give-back, may run on that thread, while an `Arena` is a one-thread struct. The specification says neither whether this compiles nor how it runs.

**example:**
from std.sync import Atomic
import std.thread

class Stats:
    hits: Atomic[int] = Atomic(0)

fn main():
    level = Arena.with_capacity(64 * 1024)               # a one-thread arena
    stats = thread_shared Stats() using level             # the list of types `using` may follow names `thread_shared`
    with scope = thread.scope():
        scope.spawn(owns fn() owns stats: stats.hits.fetch_add(1))   # the owner goes to a task on another thread
    println("done")                                       # the last owner ended on that thread, giving the block back to `level` there

**spec_today:**
p04-types.md 433-434 [TYP-46]: "It may follow an `owns`, `shares`, `shares mut` or `thread_shared` type, a class type, or an `Array`, a `Map` or a `Set`". p08-classes.md 69-71 [OBJ-1]: "Which thread made a value is read from the header of the memory chunk its block came from, written once when the chunk is made, never from the value." p08-classes.md 176-178 [RC-4] point 4: "from then on every thread, the making thread included, counts in the away count, and a later end on another thread never goes through the queue." p07-ownership.md 621-622 [LT-4]: "that address is a kept `sharedref mut` link to the instance (`[LT-46]`)". p07-ownership.md 688 [LT-46]: "on two threads it is changed only under a lock, through `Atomic`, or by a reduction".

#### options
##### [1]
**letter:**
A

**summary:**
`using` may not follow `thread_shared`: such a value always takes its block from the built-in manager.

**code:**
from std.sync import Atomic
import std.thread

class Stats:
    hits: Atomic[int] = Atomic(0)

fn main():
    stats = thread_shared Stats()                         # its block from the built-in manager
    with scope = thread.scope():
        scope.spawn(owns fn() owns stats: stats.hits.fetch_add(1))
    println("done")

**outcome:**
The example's `thread_shared Stats() using level` is E2114: "a `thread_shared` value takes its memory from the built-in manager, which every thread may give it back to; remove `using level`". This version compiles and prints done.

##### [2]
**letter:**
B

**summary:**
Allowed when the manager kind may be used from any thread and has one instance. The value then keeps its making thread in its header.

**code:**
from std.sync import Atomic
import std.thread

class Stats:
    hits: Atomic[int] = Atomic(0)

fn main():
    stats = thread_shared Stats() using System            # the platform's manager: one instance, any thread may give memory back to it
    with scope = thread.scope():
        scope.spawn(owns fn() owns stats: stats.hits.fetch_add(1))
    println("done")

**outcome:**
The example's `using level` is refused, because an `Arena` is used by one thread (E7015, with that reason). `using System`, or any manager kind with one instance that implements `ThreadShared`, compiles and prints done. Such a value keeps its making thread in the 4 unused bytes of its header, since its block has no runtime chunk to read it from.

#### recommendation
**letter:**
A

**reason:**
It keeps one way to find a shared value's making thread, the memory chunk that the two-count design chose, and adds no second header path. A mistake becomes a refusal at the line, not a race at run time. The program loses nothing it needs: its block can come from the built-in manager.

**checklist:**
3, 5 (one place for the making thread), 8 (a mistake is a refusal, not a race), 13 (no special path for values made in a manager)

## group
**gid:**
P02

**principle:**
Everything that points into or is served by a memory manager's memory (a value made in it, anything holding one, a view, a weakref link, a count in one of its blocks) ends before the manager gives that memory back, and nothing else is ended; `using` follows every type whose value owns heap memory of its own, a program's own types included, wherever the manager can serve that memory for the value's whole life, on the thread where the value may end.

**shared_cause:**
ALC-1, TYP-46 and LT-4 tie to a manager's life only 'a value made in an instance, and every value that holds one', and give a closed list of the types `using` may follow. They do not state the property that whatever lives in or points into the manager's memory is bounded by it and given back on its thread. So a weakref link (which holds no value), a scope's views (only part of the memory is released), a thread_shared value (released on another thread, with no runtime chunk header) and String, Deque and the rest (left off the list) each fall outside, and the list contradicts its own property sentence.

### members
#### [1]
J5-02

#### [2]
J5-05

#### [3]
J5-19

#### [4]
J2-02

#### [5]
J5-09

### clusters
#### [1]
K04-weakref-outlives-arena

#### [2]
K05-thread-shared-using

#### [3]
K34-which-types-take-using

#### [4]
K61-arena-scope-views

### answered
### verdicts_involved
#### [1]
19: using names a manager

#### [2]
24: the field names the kind, the making line names which; no value outlives its arena

#### [3]
47: an arena-made value carries a pointer to its arena

#### [4]
101: fix 1, the maker's id is read from the memory chunk

#### [5]
107: no manager word when the manager can do without it

**other_places:**
sharedref mut links and shares owners whose count block sits in an arena. StrView or Span into an arena String or ArenaArray. Iterators over arena lists. into_raw and C-kept addresses of arena objects. A value made in an arena and handed to another thread (verdict 114 moves it). reset() (ALC-5) and nested scope(). Pools and their handles after the pool ends. A program's own Allocator implementations and containers (a SlotMap). Closure state blocks and AnyError's block made under a manager. Arena values reached from statics.

**severity:**
high

**rank:**
2

### depends_on
#### [1]
P01

**notes:**
J5-02 was shown to the owner and is open. Ask J5-05 before J2-02/J5-09 (phase 3 conflict 2): a property-worded `using` lets thread_shared back in unless the rule names the manager's thread. Phase 3 changes J2-02's recommendation: the property must reach a program's own containers, not just 'every standard collection'. J5-05 also touches P01 (an Arena is one-thread state) and P03 (OBJ-1 reads the maker from the chunk header). J5-19 is low: a correct program refused for a reason that is untrue for it (ARN-1 against ARN-6).

## conflicts_naming_a_member
### [1]
#### ids
##### [1]
J2-02

##### [2]
J5-05

**conflict:**
J2-02 A rewrites TYP-46's list as a property: '`using` follows every type whose value itself holds heap memory: the owner words, a class, and every standard collection ... The list becomes examples of the property'. 'The owner words' include thread_shared. J5-05 A says '`using` may not follow `thread_shared`', and J5's notes warn: 'Do not rephrase the list as "every type with heap memory of its own": that would let `thread_shared` back in'.

**resolution:**
Ask K05 (J5-05) first. Whatever K34 (J2-02/J5-09) decides is then written as the property minus thread_shared under J5-05 A, with the reason stated (principle 6: a thread_shared value's block must come from the built-in manager that every thread may give memory back to). Under J5-05 B, it is minus thread_shared with a one-instance manager that is not ThreadShared.

## verdicts_involved
**19:**
JOB.md 189-195: 19. **`owned` becomes `owns`, `shared` becomes `shares`, everywhere, and a memory manager is named with
    `using`.** The owner's own idea: `mesh: owns Mesh using LevelArena`, `mesh: shares Mesh using
    LevelArena`; also `fn add(owns child: Node)`, `other = shares items` (another owner, read only),
    `other = shares mut items` (may change), `fn() with shares count:`, `names: Array[String] using
    LevelArena`. Without `using`, the built-in memory manager. (Run 3's `Box[T, A]` and `Array[T, A]`
    allocator parameter become `using`.) Owner chose the verb everywhere ("go with a"), including in
    front of a value.

**24:**
JOB.md 216-221: 24. **Which arena: the field says the kind, the line that makes the value says which one.**
    `struct Enemy: mesh: owns Mesh using LevelArena` (from some `LevelArena`);
    `boss = Enemy(hp=500, mesh=owns Mesh(points=[0.0, 1.0]) using current)` (from the arena `current`),
    which calls `current.allocate(...)`; the compiler checks `current` is a `LevelArena` and that no
    value outlives the arena it came from. Owner, after the full program (a `LevelArena` implementing
    `Allocator`, two arenas `current` and `next`): "now I get the full picture and yes I agree".

**47:**
JOB.md 385-390: 47. **A value made in an arena carries a pointer to its arena, accepted for now; lists the same way** (the
    rule writer's question 6). `lvl = Level(mesh=owns Mesh(verts=[0.0]) using current)`: the mesh keeps a
    pointer to `current` (8 bytes per arena-made value, as C++'s `std::pmr` and Rust's `Box<T, &A>`), so it
    goes back to the right arena when it grows or ends; a list is made in an arena the same way,
    `[String("a")] using current`. Not measured; the measuring phase looks for ways to drop the pointer
    (an arena that never frees single values; memory laid out so a pointer finds its arena). Owner: "Yes".

**101:**
JOB.md 786-812: 101. **A `thread_shared` object's owners are counted in two numbers** (owner, 2026-10-27, after the
    measuring phase; `tasks/96/redesign/measure-0.9.15/brc/REPORT.md`).
    - **The two numbers.**
      - The thread that made the object counts its owners in one number, A, which only it ever reads or
        writes, with normal fast instructions.
      - Every other thread uses the second number, B, with locked instructions.
      - The object has A + B owners.
      - Only the making thread can read both, so when another thread takes the last owner it hands the
        object to the making thread. That thread deletes it at its next check point (placed by the
        compiler; always before the thread ends), or the other thread deletes it at once if the making
        thread has ended.
    - **The fixes.**
      - The maker's id is read from the object's memory chunk (fix 1).
      - A hand-over the compiler sees moves A into B at that moment (fix 2), so the delay remains only for
        an object that travels inside another object.
      - Rule Q: the drop that hands the object over marks it in the same atomic step, and only the making
        thread's check then deletes it.
      - A merged object's −1 is one locked add (the shortcut).
    - **Measured against one locked count**, 550 stress runs clean:
      - a thread's own objects, 25% faster;
      - the camera program, 89 ms against 544 ms;
      - several threads reading one thread's objects, up to 28% faster;
      - objects passed back and forth all the time, 9% to 13% slower;
      - each object 16 bytes bigger.
    - Owner, asked "Do you want the two-number design in the spec?" (with fix 2): "yes". The owner first
      kept the view's count on every read ("why the fuck would remove the counter for a read
      operation?"), and asked for a fix for the language, not for one benchmark.

**107:**
JOB.md 855-874: 107. **A value made with `using` carries no word naming its manager when the manager can do without
    it** (owner, 2026-10-28, gap map Q-F, (b) and (c) together, after the measured costs: "GOOD B AND C
    IT IS").
    - (b) A manager kind that frees nothing one by one (its `deallocate` does nothing, checked when
      compiling) gives values that carry no word: nothing is ever given back value by value, and
      everything is freed together when the manager is cleared or ends. A value that grows cannot be
      made in it this way.
    - (c) A manager that hands out its memory from pieces starting at a multiple of 65,536 bytes, with
      its own address in each piece's first 64 bytes (one piece of its own for a value bigger than a
      piece), promised by its writer in `unsafe`, gives values that carry no word: the manager is found
      from the value's address, as C's `free(p)` finds its heap.
    - Every other manager's values keep the word (verdict 47).
    - Safety is unchanged: the checks that no value outlives its manager or is used after a reset run
      when compiling and never read the word.
    - Measured (`measure-0.9.15/RESULTS.md` section 7, `measure-0.9.15/M3.md`), against plain C: a
      million small values 191 ms and 28.2 MB with (b), 241 ms and 44.7 MB with the word, C 191 ms and
      28.2 MB; growing lists 243 ms and 17.1 MB with (c), 248 ms and 19.1 MB with the word, C 217 ms and
      17.1 MB; a pool freeing values one by one 296 ms and 13.7 MB with (c), 384 ms and 15.8 MB with the
      word, C 290 ms and 13.7 MB.
    - Goes into the next hardening (`[ALC-1]`, `[ALC-2]`, `[OBJ-4]`, `[COST-3]`) with milestone M10.

## allowed_rules
### [1]
ALC-1

### [2]
LT-4

### [3]
LT-46

### [4]
OBJ-1

### [5]
RC-4

### [6]
TYP-46

## rule_text
**ALC-1:**
p09-memory.md lines 184-213
* `[ALC-1]` *(changed in 0.9.15)* **The place names the kind, the making line names the instance.**
  `using A` after a type whose value has heap memory of its own (an `owns`, `shares`, `shares mut` or
  `thread_shared` value, a class object, an `Array`, `Map` or `Set`) names the kind of memory manager
  that memory comes from, a type implementing `Allocator`: `mesh: owns Mesh using LevelArena`,
  `names: Array[String] using LevelArena`, `Map[K, V, H] using A`, `Set[T, H] using A`. Without `using`,
  it comes from the built-in manager, `Global`. The line that makes such a value names the instance
  after the value: `boss = Enemy(hp=500, mesh=owns Mesh(points=[0.0, 1.0]) using current)` takes the
  mesh's block from the arena `current` by calling `current.allocate(…)`; a class object is made
  `Enemy(hp=5) using current`, a list `[0.0, 1.0] using current`, and `Array.new_in(current)` makes an
  empty one. A manager type with no fields has one instance (`Global`, `System`, and any such type a
  program declares): it needs none on the making line, and its values keep no instance word; every
  other manager type may have several instances. Checked
  when compiling: the instance is of the kind the place names, and a value given to a `using A` place
  was made in an instance of `A` (`E2020`: "`mesh` takes its memory from a `LevelArena`
  (`mesh: owns Mesh using LevelArena`), and this `Mesh` comes from the built-in manager; say which
  arena: `mesh=owns Mesh(points=[0.0, 1.0]) using current`"); a value made in an instance, and every
  value that holds one, lives no longer than that instance (`E3063`, `[LT-4]`); `using` after any other
  type is `E2114` ("an `int` has no memory of its own to take from `LevelArena`; remove `using`").
  `using` names where that value's own memory comes from; what the value holds comes from where its own
  fields' types say (the `points` list of that `Mesh` from the built-in manager). A value whose manager
  may have several instances keeps its instance's address, one word, so that its growth and its end go
  back to the instance it came from (`[LT-4]`), except in two kinds of manager, whose values keep no
  word (verdict 107): (1) a manager that frees nothing one by one, whose `deallocate` does nothing (an
  empty body, checked when compiling): nothing it hands out is given back value by value, and all of it
  is freed together when the instance is cleared or ends; a value that grows (an `Array`, `Map`, `Set`
  or `String`) made in it keeps the word, since its growth goes back to the instance; (2) a manager that
  implements `FoundByAddress` (`[ALC-2]`), whose values' instance is found from their address, as C's
  `free(p)` finds its heap. Neither changes what is checked: that no value outlives its instance or is
  used after a reset is checked when compiling and never reads the word (`[LT-4]`). An allocator type parameter is not Ember: `Box[T, A]` and
  `Array[T, A]` are `E1010`, whose help gives `owns T using A` and `Array[T] using A`.

**LT-4:**
p07-ownership.md lines 613-624
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

**LT-46:**
p07-ownership.md lines 678-693
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

**OBJ-1:**
p08-classes.md lines 63-73
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

**RC-4:**
p08-classes.md lines 155-190
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

**TYP-46:**
p04-types.md lines 430-447
* `[TYP-46]` *(new in 0.9.15)* **Which memory manager.** `using A` after a type whose value has heap
  memory of its own names the kind of memory manager that memory comes from, a type implementing
  `Allocator` (`[ALC-2]`): `mesh: owns Mesh using LevelArena`, `mesh: shares Mesh using LevelArena`,
  `names: Array[String] using LevelArena`. It may follow an `owns`, `shares`, `shares mut` or
  `thread_shared` type, a class type, or an `Array`, a `Map` or a `Set`; after any other type it is
  `E2114` ("an `int` has no memory of its own to take from `LevelArena`; remove `using`"). Without
  `using`, the memory comes from the built-in manager (`Global`). The line that makes such a value
  names the instance after the value it makes (`owns Mesh(points=[0.0, 1.0]) using current`,
  `Enemy(hp=500) using current`, `[0.0, 1.0] using current`; `[GRM-43]`), and takes the value's block
  from it by calling `current.allocate(...)`; a manager with one instance (`Global`, `System`) needs
  none there. A value given to a `using A` place that was not made in an instance of `A` is `E2020`
  ("`mesh` takes its memory from a `LevelArena` (`mesh: owns Mesh using LevelArena`), and this `Mesh`
  comes from the built-in manager; say which arena: `mesh=owns Mesh(points=[0.0, 1.0]) using
  current`"); a value made in an instance, and every value that holds one, lives no longer than that
  instance (`E3063`, `[ALC-1]`). `using` names where that value's own memory comes from; what it holds
  comes from where its own fields' types say (the `points` list of that `Mesh` from the built-in
  manager). A value whose manager may have several instances keeps its instance's address, one word,
  so that its growth and its end go back to the instance it came from.

## rules_not_found
