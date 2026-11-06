# Report: pilot (made by deep/v3/make_report.py; nothing here is in the specification)

| cluster | status | agents | decisions | holes left | tests |
|---|---|---:|---:|---:|---:|
| K04-weakref-outlives-arena | holds | 4 | 0 | 0 | 44 |
| K05-thread-shared-using | holds | 7 | 1 | 0 | 87 |
| K10-queued-drop-check-points | unresolved | 6 | 2 | 5 | 42 |

## K04-weakref-outlives-arena (P02; J5-02): **holds**, 4 agents

Outcome: `decided`. Intent: The camera follows the level's boss with a link that does not keep it alive, and says "gone" once the boss is no more.
The rule that decides it: LT-4 (tasks/spec-0.9.9/parts/p07-ownership.md 613-615 (with p08-classes.md 328 [WK-11] and 85 [OBJ-3])): "LT-4, p07 613-615: 'A value made in a memory manager's instance (`owns Mesh(points=[0.0, 1.0]) using current`, `[ALC-1]`), and every value that holds one, lives no longer than that instance, wherever it is kept'. WK-11 (p08 328) uses the same verb for a link and its target: 'A link may hold no target' (and WK-12, p08 349: 'the link holds nothing'). OBJ-3 (p08 85): 'the block is freed when the weak count reaches 0', so a link keeps the block, which is memory the instance gave out. Verdict 24 states the property: 'no value outlives the arena it came from'. Verdict 107 says 'the checks that no value outlives its manager or is used after a reset run when compiling'. So the property already reaches a link. The words do not say so plainly, because a link does not keep its target alive. ALC-5 (p09 242-243) guards a reset only 'While a value made in a memory manager's instance lives', which misses a block that is still kept after its value ends."

### Edit: LT-4 (change, tasks/spec-0.9.9/parts/p07-ownership.md 614-615)

**Before:**

> and every value that holds one, lives no longer than that instance, wherever it is kept

**After:**

> and every value that holds one or keeps its block (a weak count keeps the block after the value ends, `[OBJ-3]`), lives no longer than that instance, wherever it is kept

Why general: The property: whatever keeps memory an instance gave out lives no longer than that instance. It is stated for the block, not for one construct. So it reaches every holder of a weak count (a link in a field, in a list, in a returned struct, in a static, in a captured state) and every target kind (a class object, a `shares` or `thread_shared` value). It reaches every manager kind: under one that frees nothing one by one, and under a FoundByAddress one, the last link now always ends while the instance lives. The check is LT-4's own E3063 when compiling, so nothing new runs.

### Edit: ALC-5 (change, tasks/spec-0.9.9/parts/p09-memory.md 242-243)

**Before:**

> While a value made in a memory manager's instance lives, the instance is changed only by what allocates from it

**After:**

> While a value made in a memory manager's instance lives, or its block is kept after it ends (`[OBJ-3]`), the instance is changed only by what allocates from it

Why general: The property: an instance is not rewound, cleared or reused while any block it gave out is still kept. Today it holds only while the value lives. It now also holds while a weak count keeps an ended value's block. This reaches every change that is not an allocation or a give-back (`reset()`, `clear`, a program's own rewinding method), through any name, in every manager kind. The block's own give-back, when the last link ends, is still allowed: it is the value's end, which gives a block back. It is ALC-5's own E3021 when compiling.

Readability: Unchanged, word for word: the ordinary level-arena program with a camera link compiles and reads as it does today, printing 500, then gone. No new word, annotation or ceremony. Only a program whose link would outlive its arena, or whose arena is reset under a kept block, is refused, and its message gives the line to write.
Cost: none: decided when compiling. These are the same lifetime check (E3063, LT-4) and reset check (E3021, ALC-5) the compiler already runs for values made in an instance, now applied to the holders of weak counts as well. Nothing runs, and no byte is added to any object or link. C++ pays nothing either, and its version is undefined behaviour. The weak count's existing costs (one compare per +1, one header test per opening, WK-11 and WK-12) are unchanged.

New gaps found on the way (not this edit's): 4

Tests rejected by the script, not written (1):
* mixed_manager_link_list: the expected output is a description, not the exact text: , with `a`'s block back to the built-in manager and `b`'s to `level`, and no sanitizer or 

To verify on the compiler:
* J5-02's example: today it compiles, and opening `cam.target` in `main` reads freed memory (show it under a sanitizer, or with an arena whose end poisons its buffer). After the edit it is E3063 at line 10, `cam.target = weakref boss`.
* The option-A version (arena made in `main` above `cam`, passed as `ref mut level`) compiles and prints 500, then gone, with no sanitizer report. `cam` ends before `level`, as DRP-2 orders the locals.
* With `cam = Camera()` written above `level = Arena.with_capacity(...)` in `main`, the same call is E3063: the link would outlive `level`.
* The server sibling: E3063 at `ACTIVE.push(weakref s)`; it compiles when the session is made without `using`.
* The parser sibling compiles and prints 1, both before and after the edit (no needless refusal for links held by values made in the same instance).
* The renderer sibling: `hud.focus = weakref s` inside `draw_frame` is accepted through the summary of the two `ref mut` parameters, checked at the call in `main` (`hud` is made after `frame`). `frame.reset()` is E3021. With `hud.focus = None` above the reset it compiles, which shows that the compiler's tracking is flow-sensitive for a link that has been cleared. If it is not, report the exact fix the message would have to give.
* The game sibling: E3063 at `return t`.
* The generated code for every accepted program is identical before and after the edit (no run-time cost added).
* A link to a value held `shares` made in an arena, and to a `thread_shared` value made in one, are bounded the same way (E3063 when kept past the instance).

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

## K10-queued-drop-check-points (P05; J7-01, J7-02): **unresolved**, 6 agents

Outcome: `decided_by_parameters`. Intent: J7-01: a connection-pool server. Its main loop lists the free connection ids under the pool's lock while a worker finishes a batch, and each connection gives its id back to the pool when it is destroyed. J7-02: a game loads textures on job workers and frees each texture's GPU memory as soon as main lets go of it.
The rule that decides it: RC-4 (points 2, 3 and 5), read with DRP-8 (tasks/spec-0.9.9/parts/p08-classes.md 166-172 (points 2 and 3), 179-181 (point 5); DRP-8 is p07-ownership.md 865-868): "RC-4 point 2 (p08 166-168): "The making thread empties its queue at each of its check points: it adds each value's home count into its away count, and ends the value when the sum is zero. A queued value is ended by nothing else." Point 3 (p08 169-172): "Check points. The compiler places them: at every loop's back edge, before every call that may block the thread (waiting for a lock, `join`, a channel's receive, a sleep) and when the thread ends." Point 5 (p08 179-181): "it is ended at its making thread's next check point." DRP-8 (p07 865-868): "Anything that may end a value whose type has a `drop` ... makes the changes that `drop` makes, wherever a value of its type may end, for every rule that needs them." A check point is a place where a queued value may end, so DRP-8 already counts it as a call of that value's drop. Verdict 101 leaves where the check points go to the compiler ("placed by the compiler; always before the thread ends"). The places listed in point 3 are the lead's choice (decisions.md section 6, "the owner may overturn any"), and the lead made it without thinking of this clash."

### Edit: RC-4 (change, tasks/spec-0.9.9/parts/p08-classes.md 169-171)

**Before:**

> **Check points.** The compiler places them: at every loop's back edge, before every call that may block the thread (waiting for a lock, `join`, a channel's receive, a sleep) and when the thread ends.

**After:**

> **Check points.** The compiler places them at every loop's back edge, before every call that may block the thread (waiting for a lock, `join`, a channel's receive, a sleep) and when the thread ends; the runtime where a thread waits for work or returns to C from its outermost Ember call. Each may end any value queued to its thread, with what it owns, for every rule (`[DRP-8]`), except inside a queued value's `drop` or a place, open in any function under way, where that end may be refused (a lock held, a walk), marked by the code that opens it in a count of the thread's own. Outside them, a thread in one of the waits named, or waiting for work, is woken by a value put in its queue.

Why general: It states one property of every check point. A check point may end any value given back to its thread, together with what that value owns. That end is the thread's own code at that point, for every rule. This includes the rules that compare one thread with another (a scope's tasks and their parent, a join or a scope's end under a lock, the main thread's module variables) and a reload's compare. The one exception is a place the thread has open in any function under way where that end would be refused: a held lock, a walk, a paused generator's walk, a scope block. A given-back drop already running is the other exception, so ends never nest. The code that opens such a place knows it, so it marks it. Then a check point anywhere under it ends nothing, a library's check point included, and the library needs to know nothing of the program. The runtime adds only places where the thread has nothing in progress (its wait for work, its return to C). It wakes only waits it performs itself, never a call into C or a read. No type, lock kind or construct is named. So with nothing added it decides Lock with guarded by, Mutex, RwLock, channels, scopes, joins, sleeps, generators, callbacks, spawned threads, job workers and C threads. The thread's end is never inside a place, which keeps verdict 101's 'always before the thread ends'.

### Decision (made by the owner's parameters): Where the making thread may end a given-back value (J7-01)

**A destructor (~Conn()) that another thread gave back to the thread that made the object, run later by that thread: may it run inside that thread's lock_guard block or range-for, or inside a function called from there?**

| step | who | what happens | result |
|---|---|---|---|
| 1 | main | makes a Conn inside a Batch and hands the batch to a worker | the Conn was made on main |
| 2 | worker | lets go of the last owner of the Conn | the Conn goes back to main |
| 3 | main | walks the pool's free list inside `with pool.lock:` | a loop turn is where main may run given-back destructors |
| 4 | main (today) | would run ~Conn(), which takes the pool's lock and pushes to the walked list | refused at a line the programmer never wrote, or frozen (m.lock(); m.lock();) |
| 5 | compiler (B) | marks the block in a per-thread count; nothing given back is destroyed under it, in main or in anything main calls | compiles; ~Conn() runs at the next loop turn outside; later frames print 1 |

Today: The text either refuses the program at a line it never wrote, or lets main freeze holding its own lock.

* A: refuse it; the programmer copies the list out of the lock and walks the copy.
* B: compile it; the compiler marks each block where a given-back destructor would clash (one thread-local add in, one out), and nothing given back is destroyed inside it, in any function it calls either.
* C: compile it; objects whose destructor reaches outside them are counted atomically from the start, so the worker runs the destructor itself (slower counts for those types).
* D: compile it with nothing at run time: no destruction point where it would clash, judged with what the callers hold (needs a copy of each function per caller, impossible for a library compiled once).
* E: compile it; every lock block and every loop over a list is marked, whatever the destructors do (two thread-local adds per loop in every program).

Recommendation: B: nothing to write, nothing correct refused, two thread-local adds only around the blocks that clash; ~Conn() runs at the first point after main lets go.

Chosen: B. By: 1. The three goals at once. A refuses correct programs (ease). C gives up measured speed and verdict 101's plain count for those types. D cannot be built where a function is compiled once for every caller, so it either freezes there or never ends given-back values in library loops. E puts two adds on every loop of every program. B keeps all three. 2. Safety. Under B a given-back value ends only where its end passes every rule. That includes the rules comparing this thread with other threads, tasks and a reloaded image. It never ends inside something this thread has open, or inside another given-back drop. Nothing waits for a held lock, and nothing changes a walked list. 3. Compile time first. Where to mark is decided when compiling. Only run time knows whether a caller has such a place open while code compiled once is running. One thread-local count is the cheapest way to tell, and it is paid only where a clash is possible. 4. Written word: none. 5. Ordinary program: unchanged. 6. Consistency. Verdict 57 is applied in full: a check point that may end a Conn counts as ending it, for every rule. Verdict 43 holds. Verdict 101's 'next check point, placed by the compiler; always before the thread ends' is kept. The critic's bound is the first check point after the thread leaves the marked place, and always the thread's end, where nothing is open. A thread that holds the clashing lock for its whole loop keeps the value until it lets go. That is the earliest that drop can run without waiting for ever.

Option A:

```
# the program is refused at the inner `for` line (E7018):
#   "this loop's back edge may end a `Conn` another thread let go of, and `Conn`'s drop takes the
#    `Pool` lock this thread holds (line 12); walk a copy after the lock"
    for frame in 0..1000:
        with pool.lock:
            ids = copy of pool.free
        for id in ids:
            println(id)
```

The original does not compile; the rewrite does. Every loop under such a lock, or call of a function that has a loop, is refused the same way: correct programs refused for a point the compiler chose.

Option B:

```
# the program, unchanged
# ending a given-back Conn inside `with pool.lock:` would be refused (its drop takes a Pool's lock), so
# the compiler marks the block: one add to a count of main's own as it opens, one as it closes; a check
# point under it, in main or in any function it calls (std's included), ends nothing; the outer back
# edge and the check point before the wait for `pool.lock` end the Conn, counted for every rule
```

Compiles. Conn's drop runs at the next outer back edge, takes the lock and pushes 1. Later frames print 1, and `h.join()` prints 1. A helper with a loop, or `n.to_upper()`, called inside the block compiles and ends nothing there. The bound: the first check point after main leaves the block, and always the thread's end.

Option C:

```
# the program, unchanged
# a thread_shared type whose drop reaches outside its own fields is counted for every thread from
# the start (all counts atomic), so the worker runs Conn's drop when it ends b
```

Compiles. The worker runs Conn's drop and waits for the pool's lock until main's `with` block ends. Every count change of such a type is atomic, on main too. That gives up verdict 101's measured 25% for a thread's own values of those types. This is the same idea as J7-02's B.

Option D:

```
# the program, unchanged
# (the first answer) no check point is placed where ending a given-back Conn would be refused, judged
# with what the callers hold; nothing runs for it, and check points add nothing to what a function
# is worked out to do
```

This program compiles. A function compiled once for every caller (std's `to_upper`, called inside the block) has two choices. It keeps its check point, which ends the Conn under the lock and freezes (m.lock(); m.lock();). Or it has none anywhere, and a thread whose loop is in a library never ends given-back values. And with the ends left out of what functions do, three places stop seeing them: a scope's end under the lock, a task beside its parent, and a reload. The result is a freeze or a race.

Option E:

```
# the program, unchanged
# every `with` block of a lock and every walk is marked, whatever the given-back drops do
```

Compiles, as B. Every walk and every lock block in every program pays two thread-local adds, also where nothing can be given back. Body's Atomic drop also waits until the block ends.

### Decision (made by the owner's parameters): When a waiting or C-side making thread ends a given-back value (J7-02)

**A destructor given back to a thread that is idle (a pool worker waiting for work, a thread blocked in recv, or a C thread back in C code): when does it run, and what may it touch there?**

| step | who | what happens | result |
|---|---|---|---|
| 1 | worker | loads a Texture inside a Bundle that goes to main | made on the worker |
| 2 | main | clears the bundles | each Texture goes back to its worker |
| 3 | worker (today) | is idle, waiting for work | ~Texture() never runs; the GPU memory is held until the process ends |
| 4 | runtime (A) | wakes the waiting worker | it runs ~Texture() and waits again |
| 5 | compiler (A) | treats that ~Texture() as the worker's code, beside main | one that changes main's globals is refused; one that prints or frees compiles |

Today: A value given back to an idle worker or to a C thread waits until that thread runs Ember code again, which for an idle worker may be never.

* A: a waiting thread is woken to run what was given back; a C thread runs it when its outermost call into Ember returns; the destructor counts as that thread's code, so one that changes main's globals is refused.
* B: objects made on workers or C threads are counted atomically from the start, so whoever lets go last destroys them at once (slower counts).
* C: keep today's text: the destructor waits until that thread runs Ember code again, possibly never.
* D: while the making thread is blocked or in C, the thread that lets go last destroys the object itself (two atomic steps on every call from C into Ember).

Recommendation: A: every given-back destructor runs promptly on a waiting thread, the fast counts stay, and the cost falls only on the rare given-back object; a C thread's bound is its next call into Ember.

Chosen: A. By: 1. The three goals at once. C keeps a resource for ever (not robust). B gives up speed and verdict 101's plain count. D puts two atomic steps on every call from C into Ember and changes who deletes. A keeps speed and ease and bounds the wait. 2. Safety. A value ended at a worker's wait or at a return to C is that thread's code there, beside main, judged by every rule. So a drop that changes one of main's module variables is refused when compiling, as it is for any other thread. A thread inside a call into C or an OS read is never interrupted. A thread is never woken from inside a given-back drop, so drops never nest. 3. Run time only where compile time cannot decide (when a value arrives), and then the cheapest way. Nothing is added on the normal path: one store beside each runtime wait, and one load and one wake on the rare delayed path. 4. Written word: none. 5. Ordinary program: unchanged. 6. Consistency. Verdict 101's 'the making thread deletes it at its next check point' is kept, with one queue model for compiled code, the runtime's waits and C threads. The runtime cannot run Ember code while a thread is in C, so a C thread's bound is the return of its next outermost call into Ember. Only D closes that rest, and D is the owner's because it changes who deletes.

Option A:

```
# the program, unchanged
# the runtime has a check point where a worker waits for work and where a thread returns to C from its
# outermost Ember call; a thread waiting there, or for a lock, a join, a channel's receive or a sleep,
# with nothing open, is woken by a value put in its queue and ends it; a thread inside a call into C is
# never interrupted; an end at a worker's wait is that worker's code, beside main, for every rule
```

Compiles. The workers print 'texture 1 freed' and 'texture 2 freed' soon after `bundles.clear()`. A drop that changed one of main's module variables would be refused when compiling (E7012), since a worker ends it beside main. A C thread's bound is the return of its next outermost call into Ember.

Option B:

```
# the program, unchanged
# a thread_shared value made on a job worker or a C thread is counted for every thread from the
# start (all counts atomic), so whichever thread ends its last owner ends it at once
```

Compiles. Main prints both lines during `clear()`. Every count change of anything a worker or a C thread makes is atomic. That gives up verdict 101's plain count on the making thread for those threads. This is the same idea as J7-01's C.

Option C:

```
# the program, unchanged; today's text: the drop waits until that thread next runs Ember code
```

Compiles. While no job is submitted, the textures are never freed: the GPU memory is held for the life of the process.

Option D:

```
# the program, unchanged
# while the making thread is blocked or back in C, the thread that ends the last owner ends the
# value at once; the making thread, on waking or on every call from C into Ember, first waits for
# such an end to finish (one atomic step in, one out)
```

Compiles. Main prints both lines during `clear()`, and a C thread's values are ended at once. Every call from C into Ember pays two atomic steps (to measure). The other thread deletes in a case that verdict 101 gives to the making thread.

Readability: The text is identical and compiles before and after; nothing is written. Nothing here can be given back, so nothing is marked and nothing runs for it. Check points and marks stay the compiler's and the runtime's, unseen. In programs where a given-back drop would clash, the programmer still writes nothing. The compiler marks its own block instead of refusing a line the programmer never wrote. No program gets longer or harder to read.
Cost: J7-01 part, run time, all to measure.
- Each marked place costs one thread-local add when it opens and one when it closes; C pays nothing here.
- A place is marked only where ending a value that may be given back to this thread would be refused, and the compiler knows that from the program. In most programs no thread has such values, so nothing is marked.
- A library marks only its own places that a given-back drop may reach through its functions: its own locks, and walks of what it keeps.
- A check point's normal path is unchanged: one test of the queue. The count is read only when the queue is not empty.

J7-02 part, all to measure.
- Around each wait the runtime performs, one store before and one after, next to its system call. It says the thread is waiting with nothing open.
- On the delayed path only: one load of that mark and, if it is set, one OS wake.
- On a C thread, one test of its queue when its outermost Ember call returns: a thread-local load and compare, like FFI-22's attach check.

C pays none of this, but C has no two counts. A shared_ptr runs its deleter at once wherever the last owner ends and pays one atomic on every count change. Verdict 101's measurements beat that by 25% for a thread's own values.

### Holes left (5)

* **undecided**, high: Main freezes while it holds the pool's lock. In C++ terms that is m.lock(); m.lock(); on one thread: the freeze J7-01 was meant to remove, which verdict 43 says is refused when compiling. If the check points test 'count is not zero', main also never ends a given-back value outside a marked place. The first walk it then marks ends one inside it, and a drop that grows the walked list is a use of freed memory (push_back during a range-for). The worker never ends or wakes for anything given back to it again, so J7-02's leak returns for that worker for the life of the process.

```
import std.jobs
import std.thread
from std.sync import Lock

static mut ROSTER: Array[String] = []          # player names: main's own list (std::vector<std::string>)

class Badge:                                   # a player's badge, made on main
    name: String
    fn drop():                                 # a dropped badge puts its name back on the roster
        ROSTER.push(owns copy of this.name)

class Award:                                   # a plain object; its badge travels inside it
    badges: Array[thread_shared Badge] = []

class Pool:                                    # a connection pool: free ids behind a lock (std::mutex + std::vector)
    lock: Lock = Lock()
    free: Array[int] guarded by lock = []

class Conn:                                    # one connection, made on main
    id: int
    pool: thread_shared Pool
    fn drop():                                 # gives its id back to the pool
        with this.pool.lock:
            this.pool.free.push(owns copy of this.id)

class Batch:                                   # a plain object; its connections travel inside it
    conns: Array[thread_shared Conn] = []

gen fn roster() -> Generator[String]:          # walks the roster one name at a time
    for n in ROSTER:                           # a Badge's drop pushes to ROSTER, so this walk is a marked place
        yield copy of n

[I: needs Iterator[Item = String]]
fn print_next(ref mut it: I):                  # prints the next name, if there is one
    if it.next() exists as n:
        println(n)

fn main():
    ROSTER.push(owns String("ann"))
    ROSTER.push(owns String("bo"))
    aw = Award(badges=[thread_shared Badge(name=String("cy"))])
    with s = thread.scope():
        s.spawn(owns fn() owns aw: aw.badges.len())    # the task ends aw: the Badge goes back to main
    g = roster()
    with s = jobs.scope():
        s.submit(owns fn(): print_next(ref mut g))     # a worker starts g: g's walk opens there, and g pauses inside it
    for i in 0..3:
        print_next(ref mut g)                  # main finishes g: g's walk closes here, on main
    pool = thread_shared Pool()
    b = Batch(conns=[thread_shared Conn(id=1, pool=thread_shared pool)])
    h = thread.spawn(owns fn() owns b: b.conns.len())  # that thread ends b: the Conn goes back to main
    for frame in 0..1000:
        with pool.lock:                        # a marked place: main's count should go to 1
            println(pool.free.len())           # println's own loop is a check point
    println(h.join())
```

* **fault_not_tolerated**, high: The server freezes in its first frame. Main holds the pool's lock and waits for a job, while every worker sits in the job queue's way, waiting for that same lock. In C++ terms: a std::lock_guard held across a wait on a thread pool whose threads are all blocked on that mutex. THR-19 promises that such a wait is refused when compiling ('each would wait for the other for ever'), but the worker's new check point slips past it. The answer's verdicts_kept for verdict 43 says 'a join or a scope's end under a lock, whose thread may end such a value, is still refused'. That does not hold for job workers.

```
import std.jobs
from std.sync import Atomic, Lock

class Pool:                                   # a connection pool: free ids behind a lock (std::mutex + std::vector)
    lock: Lock = Lock()
    free: Array[int] guarded by lock = []
    served: Atomic[int] = Atomic(0)           # frames served, counted by a job

class Conn:                                   # a connection a job opens
    id: int
    pool: thread_shared Pool
    fn drop():                                # gives its id back to the pool
        with this.pool.lock:
            this.pool.free.push(owns copy of this.id)

class Batch:                                  # a plain object, combined into main's list
    conns: Array[thread_shared Conn] = []

fn main():
    pool = thread_shared Pool()
    batches: Array[Batch] = []
    with s = jobs.scope():
        for i in 0..64:
            s.submit(owns fn() copy of i: batches.push(owns Batch(conns=[thread_shared Conn(id=i, pool=thread_shared pool)])))   # each Conn is made on a worker
    batches.clear()                           # each Conn goes back to the worker that made it; the idle workers wake to end them
    for frame in 0..1000:
        with pool.lock:                       # main holds the pool's lock ...
            with s = jobs.scope():
                s.submit(owns fn(): pool.served.fetch_add(1))   # ... while it waits for a job that only counts
            println(pool.free.len())
    println(pool.served.load())
```

* **cross_rule_clash**, medium: The audio thread waits for the UI's lock inside the audio callback. The listener hears a dropout, and the audio thread suffers a priority inversion. In C++ terms, ~Voice() locks the UI's std::mutex inside the CoreAudio render callback. That is exactly what @realtime exists to refuse. The edit's own principle (where the end may be refused, it is not run) stops at the function's last line, while C's call goes on.

```
from std.sync import Lock

static MIXER_LOCK: Lock = Lock()
static mut FREE_SLOTS: Array[i32] guarded by MIXER_LOCK = []      # voice slots given back (std::mutex + std::vector)
static PENDING_LOCK: Lock = Lock()
static mut PENDING: Array[Started] guarded by PENDING_LOCK = []   # voices the audio thread started, for the UI

class Voice:                                       # a playing voice, made on the host's audio thread
    slot: i32
    fn drop():                                     # gives its slot back: takes the mixer's lock
        with MIXER_LOCK:
            FREE_SLOTS.push(owns copy of this.slot)

class Started:                                     # a plain object; its voices travel inside it to the UI
    voices: Array[thread_shared Voice] = []

# SAFETY: the host defines no other symbol named `audio_start`
@unsafe(export("audio_start", threads=any))
pub extern "C" fn audio_start(slot: i32) -> i32:  # the host's audio thread starts a voice
    with PENDING_LOCK:
        PENDING.push(owns Started(voices=[thread_shared Voice(slot=slot)]))
    return 0

# SAFETY: the host defines no other symbol named `audio_render`
@realtime
@unsafe(export("audio_render", threads=any))
pub extern "C" fn audio_render(frames: i32) -> i32:   # the host's audio thread, every block: no lock, no wait
    return frames

# SAFETY: the host defines no other symbol named `ui_frame`
@unsafe(export("ui_frame", threads=main))
pub extern "C" fn ui_frame() -> i32:              # the host's UI thread, every frame
    with PENDING_LOCK:
        PENDING.clear()                           # ends the voices: each goes back to the audio thread
    with MIXER_LOCK:                              # redraws the slot list under the mixer's lock
        for s in FREE_SLOTS:
            println(s)
    return 0

# the C host: its audio thread calls audio_start(3) once, then audio_render(256) every 5 ms;
# its UI thread calls ui_frame() every 16 ms
```

* **fault_not_tolerated**, medium: A program that runs today (with a leak) now freezes. THR-19 promises that such mutual waits are refused when compiling ('each would wait for the other for ever'), and that promise fails for waits on the job system. In C++ terms: every pool thread is blocked in a destructor on a std::mutex held by the thread that called pool.wait(), as in `std::lock_guard g(m); pool.wait_all();`. The cause: the new point, a worker's wait for work, belongs to no function of the program, so the rules that judge a wait by what its tasks do never see the ends made there.

```
import std.jobs
from std.sync import Lock

static ASSETS_LOCK: Lock = Lock()
static mut FREED_IDS: Array[int] guarded by ASSETS_LOCK = []   # ids of textures whose GPU memory went back

class Texture:                                # a GPU texture a worker loaded
    id: int
    fn drop():                                # records, under the asset lock, that its memory went back
        with ASSETS_LOCK:
            FREED_IDS.push(owns copy of this.id)

class Bundle:                                 # a plain object, combined into main's list
    textures: Array[thread_shared Texture] = []

fn load_all(ids: Span[int]) -> Array[Bundle]:
    out: Array[Bundle] = []
    with s = jobs.scope():
        for id in ids:
            s.submit(owns fn() copy of id: out.push(owns Bundle(textures=[thread_shared Texture(id=id)])))   # each Texture is made on a worker
    return out

fn main():
    ids: Array[int] = []
    for i in 0..256:
        ids.push(owns i)
    bundles = load_all(ids.as_span())         # 256 textures, made across every worker
    bundles.clear()                           # each Texture goes back to the worker that made it
    total = 0
    with ASSETS_LOCK:                         # rebuild the atlas index under the asset lock
        with s = jobs.scope():
            for i in 0..4:
                s.submit(owns fn() copy of i: total += i * i)   # straight-line jobs: no loop, no blocking call
        println(FREED_IDS.len())
    println(total)
```

* **verdict_clash**, medium: The specification now says two contrary things, and an implementer must break one of them. Verdict 86's accepted cost (a far edit can stop a line compiling; nothing runs at run time) is widened to 'a far edit can change what a line does at run time', which the owner has never ruled on. In C++ terms: editing a library's ~Conn() changes what your own lock_guard block compiles to.

```
import std.thread
from std.sync import Lock

class Pool:                                   # a connection pool: free ids behind a lock (std::mutex + std::vector)
    lock: Lock = Lock()
    free: Array[int] guarded by lock = []

class Conn:                                   # one connection, from a networking library
    id: int
    pool: thread_shared Pool
    fn drop():                                # version 1 (the library today): println("conn", this.id, "closed")
        with this.pool.lock:                  # version 2 (the library's next release): give the id back
            this.pool.free.push(owns copy of this.id)

class Batch:                                  # a plain object, handed to the worker whole
    conns: Array[thread_shared Conn] = []

fn main():
    pool = thread_shared Pool()
    b = Batch(conns=[thread_shared Conn(id=1, pool=thread_shared pool)])
    h = thread.spawn(owns fn() owns b: b.conns.len())
    for frame in 0..1000:
        with pool.lock:                       # what this line does now depends on Conn.drop's body
            for id in pool.free:
                println(id)
            println("frame", frame)
    println(h.join())
```

New gaps found on the way (not this edit's): 2

Tests rejected by the script, not written (32):
* j7_01_pool_walk_under_lock: the expected output is a description, not the exact text: Compiles, never freezes, ends. Every line printed is 1. The last line, from h.join(), is 1
* sibling_mutex_hub: the expected output is a description, not the exact text: Compiles, never freezes. Every line before the last is 'closed 7'. The last line is 1.
* sibling_rwlock_assets: the expected output is a description, not the exact text: Compiles, never freezes. Every line before the last is 'rock'. The last line is 1.
* sibling_atomic_physics: the expected output is a description, not the exact text: Compiles. The step lines are some 1s and then 0s, never below 0. The last line is 1. Check
* sibling_network_thread_woken_in_recv: the expected output is a description, not the exact text: The first line is 1. 'buffer 1 returned' comes before 'stop 0' and is expected among the f
* panic_in_given_back_drop: the expected output is a description, not the exact text: The process aborts. stderr starts with 'panic at <file>:7:' and names the out-of-range ind
* pa_callee_loop_under_callers_lock: the expected output is a description, not the exact text: Compiles, not E7018, never freezes. The lines before the last are '<frame> 1'. The last li
* t01_j701_pool_walk_under_lock: the expected output is a description, not the exact text: Compiles with no E7018 and never freezes (run it 550 times). Every line printed is `1`: fr
* t02_j702_idle_workers_free_textures: the expected output is a description, not the exact text: Prints `frame 0` to `frame 599`. `texture 1 freed` and `texture 2 freed` each appear once,
* t04_hud_atomic_on_worker: the expected output is a description, not the exact text: Compiles. The gpu bytes column goes from 8192 to 0 within the first frames. The last line 
* t07_e11_no_nested_drops: the expected output is a description, not the exact text: Prints `100000`. No stack overflow: main runs at most one given-back drop at a time.
* t08_e13_no_wake_inside_c_read: the expected output is a description, not the exact text: With `a` typed on stdin: `1` first. Then `key 97` and `frame 1 recycled`, the second only 
* t10_generator_mark_crosses_threads: the expected output is a description, not the exact text: Never freezes (run it 550 times). First the names `ann`, `bo`, `cy`, then 1000 counts (0, 
* t12_realtime_export_return: the expected output is a description, not the exact text: A C harness calls audio_start(3) on its audio thread, then ui_frame() on its main thread, 
* t13_c_sleep_not_interrupted: the expected output is a description, not the exact text: `1`, then `slept 0` about two seconds later (usleep is never cut short), and `frame 1 recy
* t16_atomic_drop_keeps_inner_check_point: the expected output is a description, not the exact text: Compiles. Nothing is marked (check with `ember inspect`: no count add). The printed live c
* j7_01_pool_free_list_under_lock: the program is a description, not code: J7-01's program in the answer, verbatim (Pool, Conn with a drop that takes the pool's lock
* j7_02_idle_workers_free_textures: the program is a description, not code: J7-02's program in the answer, with `while true:` replaced by `while frame < 600:` so that
* mutex_sibling: the program is a description, not code: The answer's Mutex sibling (Hub, Socket, Accepted, 100 ticks walking hub.closed under its 
* rwlock_sibling: the program is a description, not code: The answer's RwLock sibling (Assets, Mesh, Chunk, 100 frames walking a read guard), verbat
* atomic_sibling: the program is a description, not code: The answer's Atomic sibling (World with an Atomic live count, Body, Island, 100 steps), ve
* net_thread_buffer: the program is a description, not code: The answer's network-thread sibling (Buffer, Request, net(), main receiving one request th
* hud_atomic: the program is a description, not code: The answer's HUD sibling with `static GPU_BYTES: Atomic[int]`, verbatim.
* hud_static_mut: the program is a description, not code: The answer's HUD sibling with `static mut GPU_BYTES: int = 0` and `GPU_BYTES -= this.bytes
* s1_2_loader_and_parent_push: the program is a description, not code: First-round S1-2's program, verbatim.
* s1_3_scope_end_under_pool_lock: the program is a description, not code: First-round S1-3's program, verbatim.
* e1_1_burst_of_returns_during_health_check: the program is a description, not code: First-round E1-1's program, verbatim.
* readability_board: the program is a description, not code: The answer's readability program (Board with a guarded score list, one spawned thread addi
* hole1_atlas_scope_under_asset_lock: the program is a description, not code: Hole 1's program (ASSETS_LOCK, FREED_IDS, Texture whose drop pushes under the lock, load_a
* hole2_drop_body_changes_main_line: the program is a description, not code: Hole 2's program, built twice: once with Conn's drop as version 1 (println only), once as 
* hole3_unrelated_buffer_under_long_lock: the program is a description, not code: Hole 3's program, as given in holes (Pool, Conn, Buffer, Batch, a million frames inside `w
* hole4_ui_thread_in_c_loop: the program is a description, not code: Hole 4's program, as given in holes, with a C harness: on_frame stores the callback, and u

To verify on the compiler:
* Today's compiler on J7-01's program: does it refuse it (E7018 at the inner walk), compile it and freeze, or compile and run?
* After the edit, J7-01's program compiles and runs: frames print 1 once the worker has ended b, `h.join()` prints 1, and there is no freeze. Run it in a stress loop like verdict 101's 550 runs.
* Emitted C / `ember inspect` for J7-01: one count add when main's `with pool.lock:` block opens and one when it closes (on every exit: falling off, break, return, ?); the inner walk's check point tests the queue, then the count; no add anywhere in the Atomic sibling.
* A helper with its own loop (`with pool.lock: log_frame()`) and a std call with a loop (`println(n.to_upper())`) inside the marked block compile with no E7018 and end nothing there; std's code is unchanged by the program.
* S1-2's program is refused (E7007: the loader's check points may run Texture's drop, which pushes to FREED, which main also pushes to in the scope block); S1-3's is E7018 at the scope's end inside `with pool.lock:`; each message names the given-back drop and the line that makes the value.
* S1-1's and E1-2's programs (`static mut` changed by a Texture made in a job) are E7012, naming the drop and the job that makes the Texture; the HUD sibling with `static GPU_BYTES: Atomic[int]` compiles and shows 0.
* E1-1's program: main's stack never holds more than one given-back drop; it prints 100000.
* E1-3's program: the input thread is never interrupted inside getchar; Frame 1 ends after the call returns (at the thread's next check point).
* E1-5's program under `ember run --hot`: the reload that makes Item.drop take World's lock is refused with E9030 naming run and Item.drop; an incremental debug build recompiles run's module and marks the block.
* E1-7's program: Lease's drop, run by ending a queued Conn, marks the `with pool.lock:` block as a drop of Conn's own would; frames print 1.
* J7-02's program prints 'texture 1 freed' and 'texture 2 freed' soon after `bundles.clear()` while the frames run; measure the time from the hand-back to the drop.
* The network-thread sibling prints 'buffer 1 returned' before main sends stop.
* Runtime: a thread woken in a lock wait, a join, a channel receive or a sleep leaves that wait cleanly before it runs Ember code, and waits again after; a given-back drop run at a worker's wait that calls jobs.submit or thread.spawn finds no runtime operation half done.
* A C test harness: a callback on a C thread makes a thread_shared value inside a plain object that reaches main; main ends it; the drop runs when that thread's outermost call into Ember returns, or at the end of its next one if the value was queued while it was in C.
* Generators: a paused generator that walks a list a given-back drop changes keeps its holder's place marked until its last use; a generator dropped while paused inside a marked place lowers the count.
* Costs, all to measure: the two count adds per marked place (and how many places std marks); the store around each runtime wait; the load and wake on the delayed path; the queue test on return to C. Check that verdict 101's benchmark set (measure-0.9.15/brc) still shows its figures, including a thread's own values 25% faster.

