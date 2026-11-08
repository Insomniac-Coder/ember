## K05-thread-shared-using (P02; J5-05): **holds**, 7 agents

Outcome: `decided_by_parameters`. Intent: Count hits on one Stats object that main and a task share, with its memory taken from a one-thread arena, then print done.
The rule that decides it: OBJ-1 (p08-classes.md 69-71): "Which thread made a value is read from the header of the memory chunk its block came from, written once when the chunk is made, never from the value."

### Edit: TYP-46 (change, p04-types.md 433-434)

**Before:**

> `owns`, `shares`, `shares mut` or `thread_shared` type

**After:**

> `owns`, `shares` or `shares mut` type

Why general: In a type, `using` applies to the whole type before it (GRM-3, p03 189-190: "`owns Mesh using LevelArena` is an `owns Mesh` whose block comes from a `LevelArena`"). So after `thread_shared T` it always names the block that holds the value's header, which the next sentence refuses. The entry comes off the list so that the list and that sentence never disagree. A `thread_shared fn()` type, which has no such header, then gets the general E2114, as a plain `fn()` does.

### Edit: TYP-46 (add, p04-types.md 435)

**Before:**

> `E2114` ("an `int` has no memory of its own to take from `LevelArena`; remove `using`").

**After:**

> No `using` names the block that holds a `thread_shared` value's header: the thread that made the value is read from the memory chunk that block came from (`[OBJ-1]`), and only a block the runtime takes for a value made with no `using` lies in such a chunk. So `thread_shared Stats using System` and `thread_shared Stats() using level` are `E2114` too ("`stats` is shared between threads, so its memory comes from Ember itself, which records the thread that made it; remove `using level`").

Why general: The property is about one block, not about where words stand. It is the block that holds a counted `thread_shared` value's 32-byte header, and the making thread is read from that block's chunk. These words reach every `using` that names that block. In a type: a field, a parameter, a result, or generic code over `A: needs Allocator` (GRM-3). On a making line: a class object made `thread_shared` (HEAP-3: it "already has its block") and a struct that the word puts in a block. They reach every kind of manager the same way: an arena, a pool, `FoundByAddress`, `System`, and `Global`, which is a call to the configured allocator (ALC-2 p09 224-225), not the runtime's chunk path. They do not reach a `using` that names some other block. That covers a list's own buffer (`thread_shared [..] using frame`, which GRM-43 binds to the list, decided by Send and ThreadShared as `new_in(frame)` is), a field's memory, a list of shared values made in an arena, and a function type, which has no such header. TYP-46's existing sentence "what it holds comes from where its own fields' types say" still decides those. The two examples show the type form and the making-line form, and nothing is added per case. The exception gives its reason, as principle 6 asks.

### Edit: ALC-1 (change, p09-memory.md 185-186)

**Before:**

> `shares`, `shares mut` or `thread_shared` value, a class object, an `Array`, `Map` or `Set`)

**After:**

> `shares` or `shares mut` value, a class object, an `Array`, `Map` or `Set`; never the block that holds a `thread_shared` value's header, `[TYP-46]`)

Why general: ALC-1 repeats TYP-46's list, so it gets the same change and the same property, and points to TYP-46 for the reason. The two rules then never disagree. A class object made `thread_shared` is still in the list as a class object. The clause decides its block by the property, not by its spelling.

### Decision (made by the owner's parameters): No manager is named for the block that holds a thread_shared value's header

**Can a thread-shared object (a C++ `std::shared_ptr` made with `std::allocate_shared`) take its own memory from a custom allocator, for example a one-thread arena like `std::pmr::monotonic_buffer_resource`?**

| step | who | what happens | result |
|---|---|---|---|
| 1 | programmer | writes `stats = thread_shared Stats() using level`, where `level` is a one-thread arena | |
| 2 | the spec today | lets the shared object's own memory come from `level`, but finds which thread made a shared object only in Ember's own memory | undefined: nothing says how its counts or its end run |
| 3 | compiler, after | refuses at `using level`: a shared object's own memory always comes from Ember itself | an error at that line: "remove `using level`" |
| 4 | programmer | writes `stats = thread_shared Stats()`; a list or field inside it may still use the arena | compiles, prints done |

Today: The spec lets a shared object's own memory come from an arena, but the only way it gives to find which thread made the object needs Ember's own memory, so such a program has no defined behaviour.

* A: refuse naming an allocator for a thread-shared object's own memory, which always comes from Ember itself; what the object holds can still come from any allocator.
* B: allow an allocator that has one instance and that any thread may use; the object stores its making thread in its own header (a second way, and reading it from the object measured 13% to 44% slower for cross-thread copying).
* C: allow such an allocator; count such an object with one atomic count from birth (a second counting path, slower on its own thread).

Recommendation: A: one way to find the making thread, the mistake refused at its line, and arenas stay usable for everything the shared object holds.

Chosen: A. By: 1. The three goals at once. A keeps every thread_shared value on the measured two-count path. Its cost is the block itself: one runtime allocation and one free per shared object, as C++'s std::make_shared pays one heap allocation. Against std::allocate_shared over a std::pmr::monotonic_buffer_resource, which bumps a pointer, that is one allocation and one free more per object (to measure). B and C instead slow every count of these values: B reads the maker from the object (cross-thread copying 13% to 44% slower, measured), and C uses one atomic count (it loses the measured 25% for a thread's own objects). Shared objects are made far less often than they are counted, so A is the cheaper side. 2. Safety. All three are safe, and A turns undefined behaviour into a refusal at the line. 3. Compile time first. A is decided when compiling. B or C would be needed only if refusing lost a correct program, and none is lost: the object's meaning (shared between threads) stays without `using`, and its lists and fields can still take their memory from any manager, the arena's speed included (the Level and keyword-list siblings). 4. A written word only for meaning. A removes a word that cannot do what it says and adds nothing to write. 5. The ordinary program is unchanged. 6. Consistency. There is one way to find the making thread (OBJ-1, verdict 101 fix 1) and no second header path (checklist 13). The refusal sits at the making line or the type, as verdict 165's does, and a list's buffer is decided by verdict 164 in both of its spellings.

Option A:

```
from std.sync import Atomic
import std.thread

class Stats:
    hits: Atomic[int] = Atomic(0)

fn main():
    stats = thread_shared Stats()
    with scope = thread.scope():
        scope.spawn(owns fn() owns stats: stats.hits.fetch_add(1))
    println("done")
```

`thread_shared Stats() using level` is E2114, and so is any `using` naming the block that holds a shared object's header, in a type or on the making line, with any manager (`System`, `Global`, a pool). The message: "`stats` is shared between threads, so its memory comes from Ember itself, which records the thread that made it; remove `using level`". This version compiles and prints done. A `using` naming other memory, such as a list's buffer or a field's memory, is decided as before.

Option B:

```
from std.sync import Atomic
import std.thread

class Stats:
    hits: Atomic[int] = Atomic(0)

fn main():
    stats = thread_shared Stats() using System
    with scope = thread.scope():
        scope.spawn(owns fn() owns stats: stats.hits.fetch_add(1))
    println("done")
```

Compiles and prints done with a manager that has one instance and that any thread may use. `using level` is refused (E7015: an `Arena` is used by one thread). The object records its making thread in its header's 4 unused bytes, a second way beside fix 1's chunk header. Fix 1 removed exactly that reading because reading the maker from the object made cross-thread copying 13% to 44% slower (measure-0.9.15/brc/REPORT.md line 143). The cost for these values is to measure.

Option C:

```
from std.sync import Atomic
import std.thread

class Stats:
    hits: Atomic[int] = Atomic(0)

fn main():
    stats = thread_shared Stats() using System
    with scope = thread.scope():
        scope.spawn(owns fn() owns stats: stats.hits.fetch_add(1))
    println("done")
```

Compiles and prints done. The object is marked handed over when it is made, so every thread counts it with atomic adds, its making thread included, and nothing reads the making thread. This is a second counting path for objects made in a manager, and their own thread loses the two-count gain (25% for a thread's own objects, brc REPORT line 129). The cost for these values is to measure.

Readability: Identical. An ordinary program never names a manager for a shared object's own block, and the edit adds nothing anyone has to write. A `using` that names a list's buffer or a field's memory on such a line still reads and works as today. The only form refused is one that had no defined behaviour, or that repeated a word for no effect (`using Global`).
Cost: Decided when compiling, and the check costs nothing at run time. Every thread_shared value keeps verdict 101's two-count path, with its making thread read from its chunk header. The price is the block: each thread_shared value takes one runtime allocation and one free (the free possibly through the making thread's queue, RC-4 point 2). That is what every thread_shared value made with no `using` pays today, and what C++'s std::make_shared pays (one heap allocation). Against C++'s std::allocate_shared over a std::pmr::monotonic_buffer_resource, which bumps a pointer and frees nothing, it is one allocation and one free more per shared object (to measure). An arena-made shared object had no defined behaviour before, so no defined fast path is lost. What such an object holds (lists, fields) can still come from any arena, at the arena's bump cost.

New gaps found on the way (not this edit's): 4

To verify on the compiler:
* The J5-05 program gives E2114 at line 9 (`using level`) with the message above. With `stats = thread_shared Stats()` it compiles and prints done.
* E2114 for every `using` naming the block that holds a thread_shared header: a struct field type (`counters: thread_shared Counters using System`), a parameter type (`owns log: thread_shared Log using FixedArena`), a generic result type (`-> thread_shared Log using A`) with `return thread_shared Log() using mem`, a struct made shared (`thread_shared Mesh(points=[0.0]) using level`), and `thread_shared Stats() using Global`.
* Not reached, compiling as today: `thread_shared Level(mesh=owns Mesh(points=[0.0, 1.0]) using frame)` prints 2; `thread_shared [String("let"), String("fn")] using System` prints 2 then 2; `Batch(jobs=[thread_shared Job(), thread_shared Job()] using level)` with `jobs: Array[thread_shared Job] using Arena`; the AssetCache sibling prints 1; `shares mut Mesh(...) using level` prints 2.
* `thread_shared [..] using frame` and `thread_shared Array[String].new_in(frame)` with a FixedArena get the same code (E7015 by verdicts 164 and 165), not E2114.
* A field `on_tick: thread_shared fn() using Arena` gets the general E2114 message, as `fn() using Arena` does.
* What the compiler does today with `thread_shared X() using inst` (accepts, refuses or crashes), so the lead knows which tests change.
* The runtime gives every thread_shared block a chunk carrying the making-thread header, also when the package names its own global allocator (`[ALC-3]`) or a host passes allocation functions (`[FFI-27]`). This is a pre-existing gap (S1-5, E1-5).
* Measure one thread_shared object's runtime allocation and free against an arena bump (E1-4).

