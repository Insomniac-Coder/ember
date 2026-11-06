# K04-weakref-outlives-arena (input for method 3; made by deep/v3/make_inputs.py from phase2.json, phase3.json, deep/phase1-groups.json, JOB.md and the part files)

## cluster
**cid:**
K04-weakref-outlives-arena

**title:**
Can a weakref link to an arena-made object outlive the arena?

### members
#### [1]
J5-02

**primary:**
J5-02

**why_together:**
Single question.

**severity:**
high

**impact_rank:**
4

**notes:**
Use after free as written today. A extends LT-4/BCK-4/ALC-5 to weakref links.

## questions
### [1]
**id:**
J5-02

**verdict:**
confirmed

**route:**
owner_odr

**severity:**
high

**confidence:**
0.75

#### rules
##### [1]
ALC-1

##### [2]
ALC-5

##### [3]
LT-4

##### [4]
WK-11

##### [5]
OBJ-3

**reason:**
No rule ties a `weakref` link to the instance its target was made in. [ALC-1], [TYP-46] and [LT-4] bind only "a value made in an instance, and every value that holds one". [WK-11] says a link "holds one weak count", not the value. [OBJ-3] keeps the block, which sits in the arena's memory, until the weak count reaches 0. [ALC-1] (1) frees all of an arena's memory at once. [ALC-5] guards a reset only "while a value made in it lives". Grepping `weakref` with arena, instance and manager finds nothing more. [PHIL-10] demands an answer (no use-after-free), but more than one safe answer exists, so the choice is the owner's.

**title:**
Can a weakref link to an arena-made object outlive the arena?

**problem:**
A level loader makes the boss `using` a level arena, and the camera follows it with a `weakref` link. The link keeps the boss's memory block, but the arena frees all its memory when it ends. Once the level is gone, the camera's link points into freed memory: in C++ terms, a `weak_ptr` whose control block lived in a `std::pmr` arena that was released. The specification ties to an arena's life only the values made in it and the values that hold one. A link does not hold its target, so nothing says whether the camera line compiles, or what opening the link does later.

**example:**
class Enemy:
    hp: int

class Camera:
    target: weakref Enemy = None             # follows an enemy without keeping it alive

fn load_level(ref mut cam: Camera):
    level = Arena.with_capacity(64 * 1024)   # the level's memory, freed when load_level returns
    boss = Enemy(hp=500) using level         # the boss's block is taken from `level`
    cam.target = weakref boss                # the link's weak count keeps the boss's block
    println(boss.hp)
    # here boss ends (its block kept by the link), then `level` ends and frees all of its memory

fn main():
    cam = Camera()
    load_level(ref mut cam)
    if cam.target exists as e:               # tests the boss's header, in memory the arena gave back
        println(e.hp)
    else:
        println("gone")

**spec_today:**
p09-memory.md 199-200 [ALC-1]: "a value made in an instance, and every value that holds one, lives no longer than that instance (`E3063`, `[LT-4]`)". p09-memory.md 207-208 [ALC-1]: "all of it is freed together when the instance is cleared or ends". p08-classes.md 331 [WK-11]: "Every `weakref` link holds one weak count on its target". p08-classes.md 85 [OBJ-3]: "the block is freed when the weak count reaches 0". p09-memory.md 242-243 [ALC-5]: "While a value made in a memory manager's instance lives, the instance is changed only by what allocates from it and gives blocks back to it".

#### options
##### [1]
**letter:**
A

**summary:**
The link counts as holding the boss, so it lives no longer than the arena, like the values made in it.

**code:**
class Enemy:
    hp: int

class Camera:
    target: weakref Enemy = None

fn load_level(ref mut cam: Camera, ref mut level: Arena):   # the caller keeps the arena
    boss = Enemy(hp=500) using level
    cam.target = weakref boss
    println(boss.hp)

fn main():
    level = Arena.with_capacity(64 * 1024)   # made before cam, so it ends after cam
    cam = Camera()
    load_level(ref mut cam, ref mut level)
    if cam.target exists as e:               # the boss ended with load_level; its block is still in `level`
        println(e.hp)
    else:
        println("gone")

**outcome:**
The example is E3063 at `cam.target = weakref boss`: `cam` outlives `level`, so make the arena in the caller. This version compiles and prints 500, then gone. `level.reset()` while `cam` still holds the link is E3021, just as a reset under a live value is.

##### [2]
**letter:**
B

**summary:**
A `weakref` link may only name an object from the built-in memory manager.

**code:**
class Enemy:
    hp: int

class Camera:
    target: weakref Enemy = None

fn load_level(ref mut cam: Camera):
    boss = Enemy(hp=500)                     # no `using`: from the built-in manager
    cam.target = weakref boss
    println(boss.hp)

fn main():
    cam = Camera()
    load_level(ref mut cam)
    if cam.target exists as e:
        println(e.hp)
    else:
        println("gone")

**outcome:**
Every `cam.target = weakref boss` with a boss made `using` an arena is refused: "a `weakref` link needs an object from the built-in memory manager; make the boss without `using`". This version prints 500, then gone. A camera can never follow an arena-made enemy, even while the arena lives.

#### recommendation
**letter:**
A

**reason:**
It applies the rule the specification already has for values that hold an arena-made value to the one holder that rule misses. It is checked when compiling, nothing runs, and the common layout of a level arena with links into it keeps working.

**checklist:**
2 (accepts the level-arena program, refuses only the dangling one), 3 (decided when compiling, nothing runs), 6 (the nearest rule: what holds a value made in an instance lives no longer than the instance), 7 (every link, every manager)

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
ALC-5

### [3]
LT-4

### [4]
OBJ-3

### [5]
WK-11

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

**ALC-5:**
p09-memory.md lines 242-249
* `[ALC-5]` *(new in 0.9.15)* **An instance stays as its values need it.** While a value made in a memory
  manager's instance lives, the instance is changed only by what allocates from it and gives blocks back
  to it: the `Allocator` methods that value's growth and end call, the making of other values in it, and
  an `Arena`'s `alloc*` (`[ARN-1]`), none of which touches a block already given out. Reading it stays
  allowed (`current.made`); any other change, through any name, is
  `E3021` ("`frame.reset()` (line 26) changes the arena `names` was made in (line 25), and `names` still
  lives; end it first: write `mem.drop(owns names)` above line 26"). So no safe method of a memory manager
  rewinds it, frees it or gives its bytes out again under a value made in it.

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

**OBJ-3:**
p08-classes.md lines 83-86
* `[OBJ-3]` *(changed in 0.9.15)* The weak count starts at 1 on behalf of all owners. When the count
  of owners reaches 0 the object is ended (`[WK-3]`): its `drop` chain and its field drops run, and the
  weak count is decremented; the block is freed when the weak count reaches 0, which for a value no
  `weakref` link may reach is at once.

**WK-11:**
p08-classes.md lines 326-339
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

## rules_not_found
