# K10: Where and when does a queued thread_shared value's drop run, and how is a wait inside a held lock in a with scope block kept from hanging when a task makes a value another thread lets go of?

### Design 1 summary
The hang is closed when compiling, at its cause: a task's (or job's, or `@parallel` chunk's) thread empties its own queue at its end, before its end is let go to the parent, so `[THR-11]` now says the task's function counts as possibly running the `drop` of every `thread_shared` value made on its thread and not handed over, from the making line to the task's end; `[THR-19]`'s existing clause ("the end of a `with scope` block, whose task changes what the held lock guards") then refuses the scope's end inside `with pool.lock:` as `E7018`, and `[THR-20]`'s `E7012` list stays as it is (a task is not "another thread"), which removes the contradiction method 4 left. Nothing runs for that rule. For the rest of K10 the ruling keeps method 4's decisions (no queued drop runs inside a lock, a walk, a view or a contract; a sleeping job worker is woken) but makes the mechanism `[BLD-16]`-clean and cheaper: one plain thread-local hold count maintained only by the places that keep drops out (a lock block or guard life, a walk or view rooted in a `static mut`, a function with a contract, a scope whose task touches an unguarded `static mut`), a check point empties the queue only when the hold is zero, the leaving that returns it to zero is a check point, back-edge check points exist only in loops that call or read an `Atomic`, and a worker ending a value outside a job whose `drop` reaches an unguarded `static mut` hands it, dead, to the main thread's queue instead of counting such values atomically from birth.

### Design 2 summary
The queue is made harmless instead of the check points being made clever: a `thread_shared` value is marked handed over at the line that makes it (the mark `[RC-4]` point 4 already sets for a value made `using` an instance) in two more cases, decided by a property, never by a program's shape: (A) its type is not plain (its `drop`, or that of anything it owns all the way down, `thread_shared` owners included, has an effect a contract may forbid, reaches a `static mut`, or runs code the compiler cannot read whole), and (B) it is made on any thread but the main thread or a `thread.spawn` thread (a scope's task or job, a worker's chunk, a host thread), whose end no `join` waits for and whose claims the rules judge only until the join. So the queue holds only a value made on main or a spawned thread whose `drop` frees memory and ends what it owns and nothing else; such a drop breaks no rule wherever it runs, so a check point may stand anywhere (inside a held lock, a walk, a view, a `with scope` block), the thread's last one runs before its `join` returns, and every drop with outside effects runs where its last owner ends, as a written call the existing rules (`[THR-19]`, `[THR-20]`, `[THR-11]`, `[BRW-18]`, `[RC-5]`) already judge. E3-1's program compiles and prints `0` then `1` (the closer thread runs the Conn's drop and waits for the pool's lock until main's block ends); nothing new is refused, nothing is checked while running beyond one load and one store inside the allocation of a `thread_shared` block, and no copies, per-thread numbers or worker wakes are needed.

### Design 3 summary
The newcomer's rule: a `thread_shared` value made by `main` or by a `thread.spawn` thread goes home to be ended, at that thread's next check point where it holds no lock; a value made anywhere else (a scoped task's thread, a job worker, a `@parallel` chunk, a thread C owns) is counted with atomics from its making line and is ended wherever its last owner is let go. So nothing is ever queued to a thread that ends at a join or sleeps in the pool, which removes the scope hang (E3-1) and the idle-worker leak (J7-02) with no check at the join, no wake and no new refusal; it is the step `[RC-4]` point 4 already takes for a value made `using` an instance, and `[THR-11]`'s own sentence "a task touches no one-thread count" now covers the home count too (verdict 166). For the threads that keep a home count, a check point empties the queue only while the thread holds no lock, kept by a plain per-thread count of held locks whose release to zero is itself a check point (J7-01), a check point is added where a callback returns to C and none stands inside a function carrying a contract; a `thread.spawn` thread's function counts the `drop`s of the `thread_shared` values made on it, so a `join` of it inside a lock those `drop`s take is `E7018` when compiling, as verdict 43 and `[THR-19]` promise.


## Judgement 1

**winner**: 2

**scores**: [
 {
  "design": 1,
  "safety": 5,
  "speed": 8,
  "readability": 3,
  "consistency": 6,
  "generality": 6,
  "holes": "(1) A pool worker's queue is not partitioned by job. Design 1's point 3 drains 'at the end of every task, job and `@parallel` chunk a thread runs, before its end is let go to what waits for it'. `[JOB-2]`: '`jobs.scope()` follows `[THR-5]` and `[THR-11]`', and \u00a7XI.5: 'The job system is a pool of worker threads'. So a `jobs.scope()` job's end drains the worker's whole queue, which may hold a value an earlier job of another scope made on that worker (point 2 queues to 'the making thread'). Program: job A (scope 1, earlier) makes `thread_shared Conn(pool=...)` inside a plain `Batch` pushed to an inbox; later `with pool.lock:` / `with s = jobs.scope(): s.submit(owns fn(): sum(xs))`; a closer thread clears the inbox during the second scope; the Conn is queued to the worker; the worker running `sum` reaches its job's last check point (hold zero) and runs `Conn.drop`, which waits for `pool.lock` held by main at the scope's end: hang. The job's function is charged only with 'the `drop` of every `thread_shared` value made on its thread, by its body or a function it runs' (design 1's THR-11 text); `sum` makes none, so nothing is refused, and verdict 43's 'nothing is checked while running' is broken by a hang. The design's remaining doubt (1) admits the thread model is unpinned; the text does not hold 'either way'. (2) The hold count is a run-time branch inside every lock block that may reach a check point, a cost the owner may read as a check (the design's doubt 2), and the 'may travel' over-approximation refuses scope ends and joins that cannot hang (doubt 3). (3) `E7007` for a parent that reads an unguarded `static mut` a task-made value's `drop` changes refuses programs the current text and verdict 104 accept, since the drop's time (the task's last point) is invented by the design. (4) Readability: six mechanisms (hold count, loop-kind placement, wake, dead-value hand to main, made-type charge, contract hand-over) for one question."
 },
 {
  "design": 2,
  "safety": 9,
  "speed": 7,
  "readability": 7,
  "consistency": 8,
  "generality": 9,
  "holes": "(1) The plainness definition contradicts itself as written: \u00a7X.1's row '`Sync` | an atomic read-modify-write stronger than relaxed, a lock, a channel operation, thread spawn or join, making or ending a `thread_shared` owner or a `weakref thread_shared T` link'. A type that owns a `thread_shared` child ends that owner in its `drop`, so its effect set has `Sync`, and the design's 'has none of the effects a contract may forbid (... `Sync` ...)' makes every such type non-plain while the same sentence says 'every type the value owns is plain, all the way down, `thread_shared` owners included'. Fixed in the merged edit: the `Sync` of ending the `thread_shared` owners the value holds is excepted (that end is the runtime's step wherever the value ends). (2) Hot reload: the design's `[HR-44]` edit marks live values handed over when a type stops being plain, but a value already in a home thread's queue (queued mark set, counts at zero) is then ended by that thread's next check point, which may stand inside a lock the new `drop` takes. `[HR-12]`: 'Plan and prepare walk the list only while every other registered thread is at depth zero', and a thread blocked in a lock wait is at depth zero while holding another lock. Fixed in the merged edit: plan refuses such a reload (`E9030`) while a value of the type is queued. (3) The thread model is unpinned (doubt 4): if a scope's task could run on the thread that opened the scope, the by-thread mark would be missed. Fixed by pinning it in point 4. (4) A plain value queued to a `thread.spawn` thread blocked for its life in `rx.recv()` holds memory until the call returns or the thread ends; no resource and no hang, named in point 5. Nothing else broke: a plain `drop` frees memory and ends what it owns, so no lock (`[THR-19]`), walk (`[BRW-18]`), view (`[RC-5]`: what the queue ends has no owner and no opened `weakref thread_shared T` name, which 'holds one count for its life'), contract (\u00a7X.2) or task window (`[THR-11]`) can see it."
 },
 {
  "design": 3,
  "safety": 4,
  "speed": 8,
  "readability": 8,
  "consistency": 6,
  "generality": 7,
  "holes": "(1) Walks and views have no keep-out, on the claim that a module variable changed by a queued drop 'is already `E7012` (`[THR-20]`) or `E7007` (`[THR-11]`)'. False for a scoped task that lets go of a value the main thread made. Program: `static mut LOG: Array[String] = []`; `class Conn: fn drop(): LOG.push(owns String(\"closed\"))`; in `main`: `b = Batch(conns=[thread_shared Conn()])`; `with s = thread.scope(): s.spawn(owns fn() owns b: b.conns.len())` (the task ends the Conn's last owner: point 2 queues it to main, since `[THR-11]`'s window ends at the join and the task touched `LOG` at no moment the parent used it); then `for line in LOG: helper()`. Design 3 keeps 'every loop's back edge' as a check point, main holds no lock, so the drain runs `Conn.drop` inside the walk and pushes to `LOG`, which `[BRW-18]` promises 'nothing changes ... whatever the change': the walked array may reallocate under its view, a use after free. `E7012` needs 'a function that may run on another thread' from `[THR-20]`'s list, which excludes scoped tasks; `E7007` needs a conflict inside the scope, and there is none. (2) Contracts: 'none stands in the lines of a function that carries a contract' leaves the check points of its callees: a `@noblock` function calling a looping callee drains a `Block`ing drop inside its promise (`[EFF-5]`, `E4003` promised when compiling, broken while running). (3) Design B (hand the queue to the parent) is offered to the lead without text, and the lock count is a run-time branch at every lock release (one load and a test), the kind of check verdict 43 may not accept. (4) The by-thread mark itself is right and is the part the merged ruling keeps."
 }
]

**grafts**: From design 3: the `[WK-3]` cross-reference edit (design 2 left `[WK-3]` claiming 'is ended at its making thread's next check point' for every value) and the `[OBJ-1]` sentence that says no thread asks which thread made a value marked at its making (so the chunk record is unused for it); its one-sentence newcomer rule is used as the shape of `[RC-4]` point 5. From design 1: 'before its end is let go to what waits for it' as the words for a thread's last check point (design 2 wrote 'before its `join` returns', which does not name a scope's end), and the statement that the queue's own atomic step and the ends of what the value owns are the runtime's, counted in no function's effect set. Taken from neither, added by the judge: the `Sync` exception in the plainness definition (hole 1 of design 2); the `E9030` refusal while a value of a type that stops being plain is queued (hole 2); the pin that a scope's task or job runs on a thread the runtime owns, never on the thread that opened the scope (hole 3 of design 2, doubt 1 of design 1). Rejected from design 1: the hold count, loop-kind placement, worker wake, dead-value hand to main, made-type charge (`[EFF-24]`) and the `E7007`/`E7018` refusals of task-made drops, all unnecessary once nothing a task makes is queued and only a plain drop waits. Rejected from design 3: the held-lock count and the contract exclusion, both run-time or incomplete, and design B.

- FINAL EDIT rule=RC-4 point 4
  - old:      (`[ALC-5]`, `[LT-4]`), which an end delayed to a check point would miss, so every owner of such
     a value is counted in its away count and its end is never delayed; a value made in `Global` or
     `System` keeps the two counts, its block waits for the check point and nothing reuses it.
  - new:      (`[ALC-5]`, `[LT-4]`), which an end delayed to a check point would miss, so every owner of such
     a value is counted in its away count and its end is never delayed; a value made in `Global` or
     `System` keeps the two counts, its block waits for the check point and nothing reuses it. Two
     more values are marked handed over at the line that makes them, for the same reason: an end
     delayed to a check point would come at a place, or on a thread, where its `drop` may not run.
     **By its type.** A type is plain when it has no `drop`, or its `drop`'s effect set (`[EFF-1]`)
     has none of the effects a contract may forbid (§X.2: `Alloc`, `Sync`, `Block`, `Io`, `Lock`,
     `Panic(Explicit)`, `RuntimeCheck(Aliasing)`, `Nondet`), the `Sync` of ending the
     `thread_shared` owners the value itself holds alone excepted (ending what a value owns is part
     of its own end, and runs where that end runs), its summary (`[EFF-24]`) reaches no `static
     mut`, it runs no code the compiler cannot read whole (a call through a function value, a `dyn`
     or `virtual` method, a foreign call, which carries every effect unless its declaration states
     them, `[EFF-3]`, and runs the callbacks C keeps unless declared `@ffi(no_callbacks)`), and
     every type the value owns is plain, all the way down, `thread_shared` owners included (a `dyn
     I`, a function value and an object of an `open` or `abstract` class it owns are not). The
     package that declares the type decides, from its own text and its imports' interface files,
     and records the fact in its interface file beside the type's layout (`[BLD-14]`, `[BLD-16]`),
     as it records the weak count (`[OBJ-1]`); a generic instance's plainness is decided by the
     package that makes the instance (`[MONO-1]`). A `thread_shared` value of a type that is not
     plain is marked handed over at the line that makes it, so its `drop` never waits in a queue and
     runs where its last owner ends, as a written call there (`[DRP-8]`, `[THR-20]`). **By its
     thread.** A `thread_shared` value made on any thread other than the main thread or a
     `thread.spawn` thread is marked handed over at its making: the thread of a scope's task or job,
     which the runtime runs on a thread it owns and never on the thread that opened the scope
     (`[THR-5]`, `[JOB-2]`), a job worker running `jobs.submit` or a chunk of a `@parallel` loop (a
     chunk the calling thread runs itself, `[PAR-4]`, is made on that thread), and a host thread
     attached by `[FFI-22]`. Those threads run Ember code only while given a task, a job or a call
     from C, no `join` of the program waits for their end, and a task's claims are checked only
     until its join (`[THR-11]`), so a queue to them would have no bound and could run a `drop`
     after the window the rules checked. Which kind of thread runs the making line is known only
     while the program runs, so the runtime, which lays out the block and knows its thread's kind,
     sets the mark in the header it writes: one load and one store inside the allocation (§X.4,
     `[BLD-16]`). So the queue of point 2 holds only a value of a plain type made on the main thread
     or a `thread.spawn` thread.
  - why: Point 4 is the rule that moves a value's count into the away count, and its `using`-instance sentence already does this at the making line (N84). The two marks remove both causes of K10 by a property each: a drop with outside effects never waits for a hidden point (J7-01, method 4's decision 3), and no value waits for a thread whose end nothing joins or whose claims end at the join (J7-02, method 4's decision 4, E3-1). The `Sync` exception is needed because §X.1 lists 'ending a `thread_shared` owner' under `Sync`, which would otherwise make every type owning a `thread_shared` child non-plain while the sentence says such owners are included. The pin on where a scope's task runs is what the by-thread mark rests on.

- FINAL EDIT rule=RC-4 point 3
  - old:      a channel's receive, a sleep) and when the thread ends; the one after a `join` or a scope's end
     makes a value a joined task ended on its own thread end before the parent's next line.
  - new:      a channel's receive, a sleep) and when the thread ends, before its end is let go to what waits
     for it (its `join`); the one after a `join` or a scope's end makes a value a joined task ended
     on its own thread end before the parent's next line. What a check point ends is a value this
     thread made whose type is plain (point 4): its `drop` has no effect a contract forbids (§X.2),
     takes no lock, reaches no `static mut` and runs only code the compiler read whole, and the
     values it ends have no owner left and no name opened from a `weakref thread_shared T` link
     (`[RC-5]` point 2 holds a count for the name's life), so no view, walk or task of this thread
     reads them. A check point is therefore not a line or a call for the rules that judge where it
     stands (`[THR-19]`, `[BRW-18]`, `[RC-5]`, `[THR-11]`, `[EFF-1]`, §X.2), adds nothing to its
     function's effects or summary (`[EFF-24]`, `[DRP-8]`), and may stand inside a lock's `with`
     block or a guard's life, a walk, a view's life, a `with scope` block or a function with a
     contract; the atomic step that ends a queued value (point 2) and the ends of what it owns are
     the runtime's, counted in no function's effect set.
  - why: Point 3 is where hidden ends are placed, so it must say what they may do there. Saying what the queue can hold (point 4) makes the placement free: no per-place judgement, no copies, no per-thread number, and the question the arbiter's E3-1 left open (does the last point come before the join is let go?) is answered with nothing at stake, since the drops it runs are plain. 'Before its end is let go' makes `join` mean done, as verdict 101's 'always before the thread ends' intends.

- FINAL EDIT rule=RC-4 point 5
  - old:   5. **So a value's end is delayed only** when its last owner ends on a thread that did not make it
     and the value was not handed over at a place of point 4 (it travelled inside another object):
     it is ended at its making thread's next check point.
  - new:   5. **So a value's end is delayed only** for a value of a plain type made on the main thread or a
     `thread.spawn` thread (point 4), when its last owner ends on a thread that did not make it and
     the value was not handed over at a place of point 4 (it travelled inside another object): it is
     ended at its making thread's next check point, and its `drop`, which frees memory and ends what
     the value owns and nothing else, runs there; a making thread that waits in one call for its
     life (`rx.recv()`) ends it when the call returns or at its own end, holding only memory
     meanwhile. Every other `thread_shared` value is ended where its last owner ends.
  - why: Point 5 is the summary a reader and `[WK-3]` rely on; it must name the only values that can wait, and the one bound the owner may want to see (memory held by a thread blocked for its life, never a resource or a hang).

- FINAL EDIT rule=THR-11
  - old: A value a task or a job takes with `owns` is handed to
  another thread: `[THR-2]`'s compare and move happen at the spawn, so every link on this thread
  then finds it ended, and the task ends it with no header this thread reads.
  - new: A value a task or a job takes with `owns` is handed to
  another thread: `[THR-2]`'s compare and move happen at the spawn, so every link on this thread
  then finds it ended, and the task ends it with no header this thread reads. A `thread_shared`
  value a task or a job makes has no one-thread count either: it is marked handed over at its making
  (`[RC-4]` point 4, by the runtime on its thread), so nothing is ever queued to the task's thread to
  be ended after the join, and its `drop` runs where its last owner ends, which this rule judges
  until the join and `[THR-20]` after it. So the end of a `with scope` block waits for no `drop` but
  those the tasks' own lines run, and is `E7018` inside a lock's `with` block exactly when those
  change what the lock guards (`[THR-19]`, `[THR-5]`).
  - why: `[THR-11]`'s own principle, 'A task touches no one-thread count', is completed for the home count (verdict 166 in the same words). This is the sentence E3-1 asked for: a task's thread is never asked to run a `drop` on the task's behalf, before or after the join, so the scope's end inside a held lock is judged by the task's visible summary alone and can neither hang nor need a new refusal.

- FINAL EDIT rule=THR-20
  - old: **A
  `drop` runs where the last owner ends.** The last owner of a `thread_shared` value may end on any
  thread that holds a count of it:
  - new: **A
  `drop` runs where the last owner ends**, or, for a value of a plain type queued back to the thread
  that made it (`[RC-4]` points 2 and 4), at that thread's next check point, where a plain `drop`
  reaches nothing this Part judges; a `drop` that is not plain never waits in a queue, so it runs
  where its last owner ends, on the thread this rule judges. The last owner of a `thread_shared`
  value may end on any thread that holds a count of it:
  - why: The headline was untrue for every queued value. With the plain-type rule it is true in the only way this Part needs: every `drop` that reaches a module variable, a lock or anything another thread's rules judge runs where the last owner ends, which the next sentence ('a function that may run on another thread and holds such a count counts as possibly running that value's `drop`', `E7012`) already judges.

- FINAL EDIT rule=DRP-8
  - old:   A `drop` may change things outside its object (a log line, leaving a list).
  - new:   A `drop` may change things outside its object (a log line, leaving a list). The one end no line
  writes, a queued `thread_shared` value's at a check point, is the end of a plain type (`[RC-4]`
  point 4), whose `drop` changes nothing outside its object, and `[RC-4]` point 3 says what it is
  for the rules that judge the lines around it.
  - why: `[EFF-1]`, `[EFF-24]`, `[BRW-18]` and `[RC-5]` point 1 reach every end through this rule; without the sentence a check point inside a function would be read as a call with effects, or as nothing, and the text would not say which.

- FINAL EDIT rule=WK-3
  - old: A `thread_shared` value whose last owner ends on a thread that did not make
  it, and that was not handed over, is ended at its making thread's next check point (`[RC-4]` point
  5), and its `drop` runs there.
  - new: A `thread_shared` value of a plain type, made on the main thread or a `thread.spawn`
  thread, whose last owner ends on a thread that did not make it, and that was not handed over, is
  ended at its making thread's next check point (`[RC-4]` points 4 and 5), and its `drop` runs there;
  every other `thread_shared` value is ended where its last owner ends (`[THR-20]`).
  - why: Consistency: the cross-reference must claim what `[RC-4]` says.

- FINAL EDIT rule=OBJ-1
  - old: A `thread_shared` value made `using` an instance (`[ALC-1]`) lies in memory the
  runtime did not lay out, so it is marked handed over at the line that makes it (`[RC-4]` point 4)
  and no thread asks which thread made it.
  - new: A `thread_shared` value made `using` an instance (`[ALC-1]`) lies in memory the
  runtime did not lay out, so it is marked handed over at the line that makes it (`[RC-4]` point 4)
  and no thread asks which thread made it; the same holds for a value of a type that is not plain
  and for one made on a thread that has no queue (`[RC-4]` point 4), whose chunk record is written
  and never read.
  - why: `[OBJ-1]` says where the making thread is recorded; it must say for which values the record is never consulted, so an implementer does not look for a chunk header on a marked value.

- FINAL EDIT rule=HR-44
  - old: (`[HR-14]`), and is computed again on the next call. Nothing of this runs while the program runs.
  - new: (`[HR-14]`), and is computed again on the next call. Whether a type is plain (`[RC-4]` point 4) is
  such a fact for every line that makes a `thread_shared` value of it: when a reload takes it away,
  plan refuses the reload (`E9030`) while a value of the type is in a thread's queue ("a `Conn` is
  waiting in a thread's queue, and `Conn`'s `drop` now takes a lock; let that thread reach a check
  point and reload again"), and otherwise prepare sets the handed-over mark on every live
  `thread_shared` value of the type while it walks the list (`[HR-12]`), one store each, so none is
  queued later to a check point that may not run its new `drop`. Nothing else of this runs while the
  program runs.
  - why: A `drop` that stops being plain after a reload would otherwise run at a check point of a value made unmarked; a value already queued is ended by its home thread's next check point, which may stand inside a lock the new `drop` takes (that thread may be at depth zero in a lock wait while holding another lock), so the reload is refused for that rare state and the mark covers the rest at one store per live value, at the reload only.

- FINAL EDIT rule=COST-3 row "A `thread_shared` value's two counts"
  - old: a test of the thread's queue at each check point (`[RC-4]`), and the queue's work for a value whose last owner ends on another thread without a hand-over.
  - new: a test of the thread's queue at each check point (`[RC-4]`), one thread-local load and a branch, placed as before and never kept out of a lock block, a walk or a contract function, and the queue's work for a value whose last owner ends on another thread without a hand-over; a value of a type that is not plain, and a value made on a thread other than the main thread or a `thread.spawn` thread (`[RC-4]` point 4), count every owner atomically as if handed over at the making line, the first decided when compiling from the type's interface file, the second marked by the runtime in the header it writes (one load of the thread's kind and one store, inside the allocation): for such a value this row's measurements read the other way, one atomic count against two (a pipeline, made on one thread and ended on another, 27% faster on one thread and the same on 2 to 8; a thread's own values 25% slower), a cost paid only by a type whose `drop` already takes a lock, does I/O or calls code the compiler cannot read, and by what a task, a job, a chunk or a C callback makes; at a hot reload that makes a type stop being plain, one store per live value of that type during the walk prepare already makes (`[HR-44]`).
  - why: `[COST-5]`: the two marks change which values pay the atomic path and add one load and one store to an allocation; the row says what is paid, by which values, and what is not (nothing new per check point, no per-thread number, no copies, no wake).

**final_cost_row**: | A `thread_shared` value's two counts | guaranteed present | blocks made in `Global` come in chunks of the configured allocator, each chunk's header naming its thread, read with one load on the away path (`[OBJ-1]`); a value made `using` an instance counts every owner atomically, as if handed over at its making line (`[RC-4]` point 4); on the making thread, one load and one branch on the handed-over mark before each count operation and each opening of a `weakref thread_shared T` link (`[RC-4]` point 4); a 32-byte header, 16 bytes more than one atomic count's (`[OBJ-1]`); a test of the thread's queue at each check point (`[RC-4]`), one thread-local load and a branch, placed as before and never kept out of a lock block, a walk or a contract function, and the queue's work for a value whose last owner ends on another thread without a hand-over; a value of a type that is not plain, and a value made on a thread other than the main thread or a `thread.spawn` thread (`[RC-4]` point 4), count every owner atomically as if handed over at the making line, the first decided when compiling from the type's interface file, the second marked by the runtime in the header it writes (one load of the thread's kind and one store, inside the allocation): for such a value this row's measurements read the other way, one atomic count against two (a pipeline, made on one thread and ended on another, 27% faster on one thread and the same on 2 to 8; a thread's own values 25% slower), a cost paid only by a type whose `drop` already takes a lock, does I/O or calls code the compiler cannot read, and by what a task, a job, a chunk or a C callback makes; at a hot reload that makes a type stop being plain, one store per live value of that type during the walk prepare already makes (`[HR-44]`). Measured against one atomic count (`tasks/96/redesign/measure-0.9.15/brc/REPORT.md`, 15 runs per cell): a thread's own values 25% faster on 1 to 8 threads, the camera program 89 ms against 544 ms; reading another thread's values 3% slower on 1 thread and 17% to 28% faster on 2 to 8; one thread handing copies to all 0.93x to 1.05x; a random mix 9% to 13% slower; a pipeline (made on one thread, ended on the next) 27% slower on 1 thread and the same on 2 to 8 |

**owner_verdicts_touched**: Verdict 101 (touched, within fix 2, to be reported as N84 was): 'The thread that made the object counts its owners in one number, A' no longer holds for a value of a type that is not plain nor for a value made on a scoped task's or job's thread, a job worker or a C-owned thread, which count in B from the making line; this is fix 2's step ('a hand-over the compiler sees moves A into B at that moment') taken at the making line, the step N84 already took for a value made `using` an instance and reported as touching 101. Everything else of 101 is kept word for word: two numbers for a plain type made on main or a `thread.spawn` thread, the queue, check points placed by the compiler, deletion at the next check point and always before the thread ends (now also before its join is let go), the ended-thread path, rule Q. Verdict 43 kept in full: nothing is checked while running (the by-thread mark is one load and one store inside an allocation, run-time data of the thread's kind, `[BLD-16]`), no hidden end can wait for a lock its thread holds because a `drop` that takes a lock never waits in a queue, and the join and scope-end refusals of `[THR-19]` are unchanged. Verdict 57 kept: every end with outside effects runs as a written call where the last owner ends, and `[BRW-18]`, `[RC-5]`, `[EFF-24]` see it there; the only end no line writes changes nothing outside its object, which the verdict's own words ('a `drop` may still change things outside its object') do not forbid saying. Verdict 104 kept: a task's claims end at its join and nothing runs on its behalf after it. Verdict 114 kept: a one-thread object is never queued. Verdict 166 followed into the home count ('a scoped task neither makes nor ends' a count without atomics). Verdicts 56, 86, 88, 122 untouched. Method 4's decisions 1 (placement by judgement), 2 (worker wake), 3 (atomic from the making line for drops reaching main's statics in jobs) and 4 (host thread, left for the owner) are superseded by the two marks; the owner may still prefer decision 4's option b, which is unneeded once a host thread's values are never queued.

**remaining_doubt**: (1) The pin that a scope's task or job runs on a thread the runtime owns, never on the thread that opened the scope, is written into `[RC-4]` point 4 here; `[THR-5]` and §XI.5 do not say it, so the lead should check the runtime's design (M9) and, if a scope's task may run on the opening thread, have the runtime clear that thread's home flag for the task's duration instead. (2) The `Sync` exception in plainness lets a plain `drop`'s atomic subtracts of its `thread_shared` children, and the queue's own compare-exchange, run inside a `@nosync` function at a check point; the current text already runs the queue's step there, and `@nosync` forbids the function's own effect set, not the runtime's steps, but the owner may want the word 'runtime's steps' said once in §X.2. (3) Plainness excludes `Io`, so a `drop` that writes a log counts atomically; allowing `Io` would need check points kept out of `@noio` functions transitively, which a separately compiled callee cannot know (`[BLD-16]`), so the stricter set was chosen; the cost is on types that already pay a syscall. (4) The 25% and 27% figures are the owner's measurements of other shapes; the atomic cost on task-made and non-plain values should be measured once on the performance cores before the hardening is cut. (5) A plain value queued to a `thread.spawn` thread blocked for its life in one call holds memory until the call returns or the thread ends; point 5 names it; a wake (one futex per first queued value) would remove it at a cost the owner may or may not want. (6) The refused program of the ruling is refused by the current text already (`[THR-19]`'s join clause with `[THR-20]`); the ruling adds no refusal, and the lead's test set should carry the arbiter's E3-1 program as a run-pass test printing `0` then `1`, J7-01's as frames then `1`, J7-02's as two `freed` lines during the clear, and a break test that removes the by-thread mark (E3-1's program then hangs or races) and one that removes the by-type mark (J7-01's program then hangs at the inner walk). (7) The old strings are copied with CURRENT-SPEC.md's wrapping; the lead's asserting script should match them as substrings, each occurring once (checked for `[DRP-8]`'s and `[HR-44]`'s).

## Judgement 2

**winner**: 1

**scores**: [
 {
  "design": 0,
  "safety": 8,
  "speed": 6,
  "readability": 3,
  "consistency": 6,
  "generality": 7,
  "holes": "No accepted program found that breaks a promise: the hold count is dynamic (a callee's back edge inside a contract function's extent sees hold > 0), a walk or view rooted in a `static mut` holds it across a generator's pause, and a worker's drop reaching an unguarded `static mut` is handed to main. The holes are over-refusal and weight. (1) It refuses programs that cannot hang: its THR-11 text counts \"the `drop` of every such value, from its making line to the task's end\" and its summary says \"when unsure, it may travel\", so a scoped task that makes `thread_shared Conn(id=1, pool=thread_shared pool)` into a `Batch` it pushes onto the parent's own list `made` (a real variable, no third thread), while the parent holds `with pool.lock:` around the scope, is `E7018` at the scope's end, though nothing ends the `Conn` during the task and the current text's THR-19 clause (\"the end of a `with scope` block, whose task changes what the held lock guards\") was meant for a change the task's lines make. Likewise its E7007 extension (\"what those `drop`s change is the task's own change, checked ... against the parent\") refuses a parent that reads an unguarded `static mut` the `Conn`'s `drop` writes during the scope, a race that needs a third thread the program has not got. (2) The THR-19 edit keeps \"changes what the held lock guards\" while its message reads by kind (\"takes a `Pool`'s lock (line 14)\"); the two readings differ for `this.pool.lock` against main's `pool.lock`, and the text does not say which. (3) Seven mechanisms (hold count with four kinds of place, back-edge placement rule, worker wake, dead hand to main, contract-based point-4 mark, made-type summary, last-point counting) for one question; the owner's condition is the elegant shape, and a newcomer cannot hold this in a sentence. (4) Its remaining doubt (1) is real: whether a scope's task runs on its own thread or a worker changes which thread runs a `drop` queued after the join, and the text is written to hold either way at the cost of the dead-hand path."
 },
 {
  "design": 1,
  "safety": 9,
  "speed": 8,
  "readability": 9,
  "consistency": 7,
  "generality": 9,
  "holes": "No accepted program found that breaks a promise: every `drop` with an outside effect runs where its last owner ends, as a written call the existing rules judge (`[DRP-8]`: \"Anything that may end a value whose type has a `drop` ... makes the changes that `drop` makes, wherever a value of its type may end, for every rule that needs them\"); what the queue ends frees memory only. The holes are in the letter. (1) Cost, not safety: the effect table says `Sync` is \"making or ending a `thread_shared` owner or a `weakref thread_shared T` link\", so by the design's own words (\"its `drop`'s effect set (`[EFF-1]`) has none of the effects a contract may forbid (\u00a7X.2: `Alloc`, `Sync`, ...)\") a `Conn` with `pool: thread_shared Pool` and no `drop` at all is not plain, and every object graph of `thread_shared` objects pays atomics; the count steps of ending what the value holds must be carved out (grafted below). (2) Hot reload: its HR-44 edit \"sets the handed-over mark on every live `thread_shared` value of the type ... one store each\" loses the home count: point 4's hand-over \"first adds its own count of the value into the away count and marks the value handed over\"; and a value already queued (queued mark set) when the type stops being plain would have its new, non-plain `drop` run at the home thread's next check point, where no rule judged it (fixed below). (3) `[WK-3]`'s last sentence (\"is ended at its making thread's next check point (`[RC-4]` point 5), and its `drop` runs there\") is left unqualified, true but no longer the whole story (fixed below). (4) A `@nosync` function's check point still performs the queue's acquire and compare-exchange when a value is queued, `Sync` by the table's letter; the design calls a check point \"not a line or a call\" for \u00a7X.2, which is the right reading (the handed-over test before a count operation is likewise the runtime's), but the owner may want it said. (5) E3-1's program with a stream of batches (an `open_some` loop in the task) can still deadlock: the closer runs `Conn`'s `drop` under `inbox.lock` and waits for `pool.lock`, the task waits for `inbox.lock`, main holds `pool.lock` waiting for the task; this is a lock-order cycle across three threads, written in the source (`Conn.drop` takes `pool.lock`, called by `inbox.items.clear()`), which `[THR-19]` never promised to refuse (\"A wait for a lock this thread holds\"), so it is no promise broken, but the lead should know the design closes the hidden hang, not every hang."
 },
 {
  "design": 2,
  "safety": 5,
  "speed": 8,
  "readability": 7,
  "consistency": 6,
  "generality": 7,
  "holes": "Two accepted programs break promises, both from \"Walks and views need no keep-out: besides its own object a queued drop can change only guarded places (the lock count), atomics, and module variables, and a module variable changed by the drop of a value another thread may end is already `E7012` (`[THR-20]`) or `E7007` (`[THR-11]`, through `[DRP-8]`)\" and from \"none stands in the lines of a function that carries a contract\". (1) `static mut LOG: Array[String] = []`; `class Conn: fn drop(): LOG.push(owns String(\"closed\"))`; `gen fn lines(): for l in LOG: yield l`; in `main`: `b = Batch(conns=[thread_shared Conn()])` (made on main), `with pool.lock:`, inside it `with s = thread.scope(): s.spawn(owns fn() owns b: b.conns.len())` (the task ends `b`, so the `Conn` is queued to main), then still inside the lock block `g = lines()`, `first = g.next()`; after the block `println(g.next())`. `[THR-11]` accepts it (the parent reads `LOG` only after the join, verdict 104; the task was not started inside a walk), `[THR-20]` accepts it (a scoped task may reach a `static mut`). Under this design the check point after the scope's end does not drain (\"only while the thread holds no lock\"), the release of `pool.lock` is a check point and runs `Conn`'s `drop`, which pushes to `LOG` while `g` is paused in a walk of `LOG`: `[BRW-18]` (\"or a generator that walks it ... will still be used, nothing changes the collection\") is broken, and a reallocation of `LOG` under the generator's element view is a use after free. (2) `@noblock fn spin(): while not READY.load(): step()` with `fn step():` a plain function containing a loop: the design keeps check points out of `spin`'s own lines only, so `step`'s back edge drains main's queue inside `spin`'s run and a queued `Conn`'s `drop` takes `pool.lock`, a `Block` inside a `@noblock` function; `[EFF-5]` (\"Each contract in the table forbids its effect in the function's effect set\") is broken by code no effect set holds. Design 1's dynamic hold and design 2's plain-type rule both close these; design 3's static rule does not. Also, its EFF-24 \"made types\" summary and THR-20 count exist only to make `h.join()` under a lock `E7018`, which under design 2 needs nothing (a plain `drop` takes no lock)."
 }
]

**grafts**: From design 3 (index 2): (a) the `[OBJ-1]` sentence that a chunk of a thread without a home count names no thread and its blocks are laid out marked handed over, which is how \"marked at its making\" costs nothing at the making line beyond the header the allocation writes anyway; (b) the check point where an exported function or trampoline returns to C, with its two §X.4 rows, so a main thread driven by a C event loop frees plain values queued to it between callbacks (design 2's queue holds memory only, but a loop-free callback program would hold it until exit); (c) its honesty about verdict 101: the ruling narrows which threads keep the home count A, and is reported as touching 101, as N84 was. From design 1 (index 0): (d) no check point at the back edge of a loop of plain operations (one whose body makes no call and no operation on an `Atomic` or a channel end): the per-iteration thread-local load and branch leaves every leaf loop, which keeps C's code and vectorisation where it matters, and is safe under design 2 because a plain end owes nothing but memory and a finite leaf loop reaches its end, the next point; (e) `jobs.wait` named among the blocking calls. Not taken: design 1's hold count, worker wake, dead hand to main and task-function counting (none is needed once nothing queued has an outside effect and nothing is queued to a task's thread); design 3's lock count, made-type summary and `[THR-19]`/`[THR-20]` join clauses (a plain `drop` takes no lock, so a join under a lock cannot hang through the queue and the existing join clause suffices). The task's two suggested alternatives, weighed and not taken: \"a scoped task's thread is the parent's for counting\" degenerates to the by-thread mark: the task's own count steps cannot be plain adds on the parent's home count (the parent may write it at the same moment, `[THR-11]` \"A task touches no one-thread count\"), so they must go to the away count, and a value made with home 0 must be marked handed over at birth or an away subtract reaching zero ends nobody; that is design 2's rule with the chunk in the parent's pool and no gain. \"A check point at the join that the lock holder runs\" (the task hands its queue to the parent) runs a lock-taking `drop` on a thread holding that lock unless deferred to the lock's release, which needs a hold count and a third thread running the `drop`, and refuses nothing a reader could see; rejected. Fixes applied to the winner in the merge: the `Sync` carve-out for the count steps of ending what the value holds; `[HR-44]`'s hand-over moves the home count into the away count and ends an already-queued value with the old image's plain `drop`; `[WK-3]` names the only values that wait; the cross-reference for the return to C claims what `[FFI-22]` says (the entries C calls), not a word it has not got.

- FINAL EDIT rule=RC-4 point 3
  - old:   3. **Check points.** The compiler places them: at every loop's back edge, before and after every
     call that may block the thread (waiting for a lock, a `join`, the end of a `with scope` block,
     a channel's receive, a sleep) and when the thread ends; the one after a `join` or a scope's end
     makes a value a joined task ended on its own thread end before the parent's next line.
  - new:   3. **Check points.** The compiler places them: at the back edge of every loop whose body makes a
     call or an operation on an `Atomic` or a channel end (a loop that may wait for another thread,
     which may be ending values this thread made; a loop of plain operations has none, and its end
     is the next point), before and after every call that may block the thread (waiting for a lock,
     a `join`, `jobs.wait`, the end of a `with scope` block, a channel's receive, a sleep), at the
     return of every exported function and trampoline (the entries C calls, `[FFI-22]`), and when
     the thread ends, before its `join` returns; the one after a `join` or a scope's end makes a
     value a joined task ended on its own thread end before the parent's next line. What a check
     point ends is a value this thread made whose type is plain (point 4): its `drop` frees memory
     and ends what the value owns, and does nothing else; the values it ends have no owner left, so
     no view, walk or task of this thread reads them (a name opened from a `weakref thread_shared T`
     link holds a count, `[RC-5]` point 2, and a `weakref T` link reaches no `thread_shared` value,
     `[THR-2]`). A check point is therefore not a line or a call for the rules that judge where it
     stands (`[THR-19]`, `[BRW-18]`, `[RC-5]`, `[THR-11]`, `[EFF-1]`, §X.2), adds nothing to its
     function's effects or summary (`[EFF-24]`, `[DRP-8]`), and may stand inside a lock's `with`
     block or a guard's life, a walk, a view's life, a `with scope` block or a function with a
     contract; its own work (the test of the queue, the merge of point 2, a plain `drop`) is the
     runtime's, as the handed-over test before a count operation is (point 4).
  - why: Point 3 is where hidden ends are placed, so it must say what they may do there. Instead of moving the points away from locks, walks, views, scopes and contracts (a per-place judgement across calls, copies or a per-thread number, which left E3-1 open and, in design 3, a generator and a contract callee broken), the text says what the queue can hold, and from that the placement is free; "before its `join` returns" makes `join` mean done, as verdict 101's "always before the thread ends" intends and method 4's decision 5 chose. The back-edge rule removes the per-iteration thread-local test from every leaf loop (graft from design 1), and the return to C gives a main thread driven by a C event loop a bound (graft from design 3).

- FINAL EDIT rule=RC-4 point 4
  - old: `System` keeps the two counts, its block waits for the check point and nothing reuses it.
  - new: `System` keeps the two counts, its block waits for the check point and nothing reuses it. Two
     more values are marked handed over at the line that makes them, for the same reason: an end
     delayed to a check point would come at a place, or on a thread, where its `drop` may not run.
     **By its type.** A type is plain when it has no `drop`, or its `drop`, taken with the `drop`s
     of everything the value owns, all the way down, has an effect set (`[EFF-1]`) with none of the
     effects a contract may forbid (§X.2: `Alloc`, `Sync`, `Block`, `Io`, `Lock`, `Panic(Explicit)`,
     `RuntimeCheck(Aliasing)`, `Nondet`) apart from the count steps of ending the owners and
     `weakref thread_shared T` links the value holds, which every end performs; its summary
     (`[EFF-24]`) reaches no `static mut`; it runs no code its package cannot read whole (a call
     through a function value, a call through a `dyn`, `virtual` or interface method that a package
     it does not see may implement, `[EFF-6]`, a foreign call not declared `@ffi(no_callbacks)`);
     and every type the value owns is plain, the target of a `thread_shared` owner it holds included
     (a `dyn I`, a function value and an object of an `open` or `abstract` class it owns are not
     plain). The package that declares the type decides, from its own text and its imports'
     interface files, and records the fact beside the type's layout in its interface file and in
     its `TypeInfo` (`[BLD-14]`, `[BLD-16]`, `[RT-3]`), as it records the weak count (`[OBJ-1]`). A
     `thread_shared` value of a type that is not plain is marked handed over at the line that makes
     it, so its `drop` never waits in a queue and runs where its last owner ends, as a written call
     there (`[DRP-8]`, `[THR-20]`). **By its thread.** A `thread_shared` value made on any thread
     other than the main thread or a `thread.spawn` thread (the thread of a scope's task or job,
     `[THR-11]`, `[JOB-2]`; a job worker running `jobs.submit` or a chunk of a `@parallel` loop,
     §XI.5; a host thread attached by `[FFI-22]`; a chunk the calling thread runs itself, `[PAR-4]`,
     is made on that thread) is marked handed over at its making: those threads run Ember code only
     while given a task, a job or a call from C, no `join` of the program waits for their end, and a
     task's claims are checked only until its join (`[THR-11]`), so a queue to them would have no
     bound and could run a `drop` after the window the rules checked. The runtime, which lays out
     the block and knows its thread's kind, sets the mark in the header it writes (`[OBJ-1]`,
     §X.4). So the queue of point 2 holds only a value of a plain type made on the main thread or a
     `thread.spawn` thread.
  - why: Point 4 is the rule that moves a value's count into the away count, and its `using`-instance sentence already does exactly this at the making line (N84). The two new cases are the two causes of K10: a `drop` with outside effects waiting for a hidden point (J7-01, method 4's decision 3), and a queue to a thread whose end nothing waits for or whose claims end at the join (J7-02, method 4's decision 4, E3-1). Both are properties, decided by the package that declares the type or by the runtime from a fact that exists only while the program runs (`[BLD-16]`: "where none of these holds, the instruction is emitted and the test is paid", here one load inside an allocation). The `Sync` carve-out is needed because the effect table makes "making or ending a `thread_shared` owner" a `Sync` effect, which would make every object holding another `thread_shared` object non-plain.

- FINAL EDIT rule=RC-4 point 5
  - old:   5. **So a value's end is delayed only** when its last owner ends on a thread that did not make it
     and the value was not handed over at a place of point 4 (it travelled inside another object):
     it is ended at its making thread's next check point.
  - new:   5. **So a value's end is delayed only** for a value of a plain type made on the main thread or a
     `thread.spawn` thread (point 4), when its last owner ends on a thread that did not make it and
     the value was not handed over at a place of point 4 (it travelled inside another object): it is
     ended at its making thread's next check point, and its `drop`, which is plain, runs there.
  - why: Point 5 is the summary a reader and `[WK-3]` rely on; it must name the only values that can wait.

- FINAL EDIT rule=THR-11
  - old: new value after the `with scope` block"). A value a task or a job takes with `owns` is handed to
  - new: new value after the `with scope` block"). A `thread_shared` value a task or a job makes has no
  one-thread count either: it is marked handed over at its making (`[RC-4]` point 4, by its thread),
  so nothing is ever queued to the task's thread to be ended after the join, and its `drop` runs
  where its last owner ends, which this rule judges until the join and `[THR-20]` after it. A value
  a task or a job takes with `owns` is handed to
  - why: The rule's own principle, "A task touches no one-thread count", is completed for the home count (verdict 166 in the same words). This is the sentence E3-1 asked for: a task's thread is never asked to run a `drop` on the task's behalf, before or after the join, so the end of a `with scope` block inside a held lock is judged by the task's summary alone (`[THR-19]`, `[THR-5]`) and can neither hang through a hidden end nor need a new refusal.

- FINAL EDIT rule=THR-20
  - old: `drop` runs where the last owner ends.** The last owner of a `thread_shared` value may end on any
  - new: `drop` runs where the last owner ends**, or, for a value of a plain type queued back to the thread
  that made it (`[RC-4]` points 2 and 4), at that thread's next check point, where a plain `drop`
  reaches nothing this Part judges; a `drop` that is not plain never waits in a queue, so it runs
  where its last owner ends, on the thread this rule judges. The last owner of a `thread_shared`
  value may end on any
  - why: The headline was untrue for every queued value (its `drop` ran later, on the making thread). With the plain-type rule it is true in the only way that matters to this Part: every `drop` that reaches a module variable, a lock or anything another thread's rules judge runs where the last owner ends, which is what the next sentence (a function holding a count counts as running the `drop`; `E7012`) already judges.

- FINAL EDIT rule=DRP-8
  - old: A `drop` may change things outside its object (a log line, leaving a list).
  - new: A `drop` may change things outside its object (a log line, leaving a list). The one end no line
  writes, a queued `thread_shared` value's at a check point, is the end of a plain type (`[RC-4]`
  point 4), whose `drop` changes nothing outside its object, and `[RC-4]` point 3 says what it is
  for the rules that judge the lines around it.
  - why: `[DRP-8]` is where every rule (`[EFF-1]`, `[EFF-24]`, `[BRW-18]`, `[RC-5]` point 1) is told that an end is a call of the `drop`; without this sentence a check point inside a function would be read as a call with effects, or as nothing, and the text would not say which. It points to the one place that decides.

- FINAL EDIT rule=WK-3
  - old: it, and that was not handed over, is ended at its making thread's next check point (`[RC-4]` point
  5), and its `drop` runs there.
  - new: it, and that was not handed over (a value of a plain type made on the main thread or a
  `thread.spawn` thread, `[RC-4]` point 4), is ended at its making thread's next check point
  (`[RC-4]` point 5), and its `drop`, which frees memory and does nothing else, runs there.
  - why: Consistency: the cross-reference claims what `[RC-4]` now says, and names the only values that wait.

- FINAL EDIT rule=OBJ-1
  - old: never changes. A `thread_shared` value made `using` an instance (`[ALC-1]`) lies in memory the
  - new: never changes. A chunk taken by a thread that keeps no home count (`[RC-4]` point 4: a scoped
  task's or job's thread, a job worker, a host thread) names no thread, and a `thread_shared` block
  from it is marked handed over in the header the runtime writes as it lays the block out, so no
  thread ever finds itself its maker and every count operation on it takes the away path; a block
  of a type that is not plain is marked the same by the line that makes it. A `thread_shared` value
  made `using` an instance (`[ALC-1]`) lies in memory the
  - why: Where the making thread is recorded: the no-thread chunk is how "marked at its making" is implemented with nothing at the making line beyond the header's initial marks and no change to `[RT-10]`'s inlined count operations (graft from design 3).

- FINAL EDIT rule=HR-44
  - old: (`[HR-14]`), and is computed again on the next call. Nothing of this runs while the program runs.
  - new: (`[HR-14]`), and is computed again on the next call. Whether a type is plain (`[RC-4]` point 4) is
  such a fact for every line that makes a `thread_shared` value of it; when a reload takes it away,
  prepare, which walks the live instances while every registered thread is at depth zero (`[HR-3]`,
  `[HR-12]`), hands over every live `thread_shared` value of the type as `[RC-4]` point 4 does (its
  home count moved into its away count, the handed-over mark set), and ends one it finds queued
  whose two counts add up to zero with the old image's `drop`, which is plain, so none waits in a
  queue whose check points may no longer run its `drop`. Nothing else of this runs while the program
  runs.
  - why: Hot reload is in scope: a `drop` that becomes non-plain after a reload would otherwise run at a check point of a value made unmarked, or of a value already queued. The commit already walks the live-instance list with every thread at depth zero, so the fix costs one hand-over per live value of that type, at the reload only; a bare store of the mark (design 2's words) would have lost the home count.

- FINAL EDIT rule=COST-3 row "A `thread_shared` value's two counts"
  - old: a test of the thread's queue at each check point (`[RC-4]`), and the queue's work for a value whose last owner ends on another thread without a hand-over.
  - new: a test of the thread's queue at each check point (`[RC-4]` point 3), one thread-local load and a branch, at the back edge of a loop whose body makes a call or an operation on an `Atomic` or a channel end and never in a loop of plain operations, around a blocking call, at the return of an export or a trampoline and at the thread's end; the queue's work, out of line, for a value whose last owner ends on another thread without a hand-over, a plain `drop` only; a value of a type that is not plain, and a value made on a thread other than the main thread or a `thread.spawn` thread (`[RC-4]` point 4), count every owner atomically as if handed over at the making line, the second marked by the runtime in the header it writes (one load of the thread's kind and one store, inside the allocation), and for such a value this row's measurements read the other way (a pipeline, made on one thread and ended on another, 27% faster on one thread and the same on 2 to 8; a thread's own values 25% slower); no copy of a function per call place, no per-thread count of held locks and no wake of a worker is paid.
  - why: `[COST-5]`: every implicit cost has a row. The two marks change which values pay the atomic path and add one load and one store to an allocation; the back-edge rule and the return to C change where the queue test stands; what is not paid is said so the lead can price this ruling against the two designs not taken.

- FINAL EDIT rule=COST-3 row "An Ember function called by C through a function pointer"
  - old: | An Ember function called by C through a function pointer (`[FFI-21]`) | guaranteed present | the thread-local attach check of its C entry (`[FFI-22]`), nothing else |
  - new: | An Ember function called by C through a function pointer (`[FFI-21]`) | guaranteed present | the thread-local attach check of its C entry (`[FFI-22]`) and one test of the thread's queue at its return (`[RC-4]` point 3), nothing else |
  - why: The return-to-C check point is a cost on that path (graft from design 3).

- FINAL EDIT rule=COST-3 row "An export's or a trampoline's entry"
  - old: | An export's or a trampoline's entry (`[FFI-22]`, `[FFI-33]`) | guaranteed present | the thread-local attach test, and under `threads=creator` or `main` one compare of the calling thread |
  - new: | An export's or a trampoline's entry (`[FFI-22]`, `[FFI-33]`) | guaranteed present | the thread-local attach test, under `threads=creator` or `main` one compare of the calling thread, and one test of the thread's queue at the return (`[RC-4]` point 3) |
  - why: The same cost on the export path (graft from design 3).

**final_cost_row**: | A `thread_shared` value's two counts | guaranteed present | blocks made in `Global` come in chunks of the configured allocator, each chunk's header naming its thread, read with one load on the away path (`[OBJ-1]`); a value made `using` an instance counts every owner atomically, as if handed over at its making line (`[RC-4]` point 4); on the making thread, one load and one branch on the handed-over mark before each count operation and each opening of a `weakref thread_shared T` link (`[RC-4]` point 4); a 32-byte header, 16 bytes more than one atomic count's (`[OBJ-1]`); a test of the thread's queue at each check point (`[RC-4]` point 3), one thread-local load and a branch, at the back edge of a loop whose body makes a call or an operation on an `Atomic` or a channel end and never in a loop of plain operations, around a blocking call, at the return of an export or a trampoline and at the thread's end; the queue's work, out of line, for a value whose last owner ends on another thread without a hand-over, a plain `drop` only; a value of a type that is not plain, and a value made on a thread other than the main thread or a `thread.spawn` thread (`[RC-4]` point 4), count every owner atomically as if handed over at the making line, the second marked by the runtime in the header it writes (one load of the thread's kind and one store, inside the allocation), and for such a value this row's measurements read the other way (a pipeline, made on one thread and ended on another, 27% faster on one thread and the same on 2 to 8; a thread's own values 25% slower); no copy of a function per call place, no per-thread count of held locks and no wake of a worker is paid. Measured against one atomic count (`tasks/96/redesign/measure-0.9.15/brc/REPORT.md`, 15 runs per cell): a thread's own values 25% faster on 1 to 8 threads, the camera program 89 ms against 544 ms; reading another thread's values 3% slower on 1 thread and 17% to 28% faster on 2 to 8; one thread handing copies to all 0.93x to 1.05x; a random mix 9% to 13% slower; a pipeline (made on one thread, ended on the next) 27% slower on 1 thread and the same on 2 to 8 |

**owner_verdicts_touched**: Verdict 101 is touched and must be reported (as N84 was, decisions.md §13): its letter, "The thread that made the object counts its owners in one number, A", no longer holds for a scoped task's or job's thread, a job worker or a host thread, nor for a value of a type that is not plain; those count in B from the making line. Its substance is kept word for word: two numbers for every plain value made on the main thread or a `thread.spawn` thread, the queue, check points placed by the compiler, "always before the thread ends" (a thread's last point runs before its `join` returns), the ended-thread path, and fix 2 ("a hand-over the compiler sees moves A into B at that moment") taken at the making line, which N84 already did for a value made `using` an instance and reported as touching 101. Verdict 43 kept in full: nothing is checked while running, no new refusal, and no hidden end can wait for a lock its thread holds, because a `drop` that takes a lock never waits in a queue; E3-1's program compiles and prints 0 then 1. Verdict 57 kept: every end with an outside effect runs as a written call where the last owner ends, so `[BRW-18]`, `[RC-5]`, `[THR-19]`, `[THR-20]` and the summaries see it; the only end no line writes changes nothing outside its object. Verdict 56 kept (the ended mark before the `drop`, unchanged). Verdict 104 kept: a task's claims end at its join, and nothing runs on its behalf after it. Verdict 114 kept: a one-thread object is never queued, and the hand-over walk is unchanged. Verdict 166 followed into the home count ("a task touches no one-thread count"). Verdicts 86 and 88 untouched (nothing written by the programmer, no copies). Method 4's decisions: 5 (a thread's last check point before its join is let go) kept; 1 (placement by judgement), 2 (worker wake) and 3 (atomic from the making line only for drops reaching main's statics in jobs) superseded by the two marks of point 4; 4 (a value queued to a host thread gone back to C), left for the owner, is decided by the by-thread mark (a host-thread-made value is never queued) and by the return-to-C check point (a main-thread-made plain value queued while main is in C is freed at main's next return from a callback); the owner may still prefer their option b.

**remaining_doubt**: (1) Where a `thread.scope()` task runs (its own thread or a pool worker) is still unstated in `[THR-5]` and §XI.5; the ruling holds either way (neither kind keeps a home count), but the lead should pin it for `[HR-3a]`'s registration and the "home thread" flag. (2) The `Sync` carve-out: a plain `drop` that ends a `thread_shared` owner whose target this thread made may end the target there (home to zero, away zero), running the target's `drop` at the check point; the recursive clause ("the target of a `thread_shared` owner it holds included") makes that `drop` plain too, so it is safe, but the implementer needs a greatest fixpoint over the package's type graph for cycles, and an imported type's plain bit read from its interface file. (3) `@nosync`: a check point's drain performs an acquire and a compare-exchange when a value is queued, `Sync` by the table's letter; the text now calls that the runtime's work, not the function's effect, as the handed-over test already is; the owner may instead want check points kept out of a `@nosync` function's own lines, which costs nothing in safety under this ruling. (4) A `thread.spawn` thread blocked for its life in one call (`rx.recv()`) holds plain values queued to it until the call returns or the thread ends: memory only, bounded by what that thread made before blocking; the text does not name this bound. (5) Plainness excludes `Io`, so a `drop` that writes a log file counts atomically; allowing `Io` would need check points kept out of `@noio` functions transitively, which a separately compiled callee cannot know (`[BLD-16]`), so the stricter set stands. (6) E3-1's program with a stream of batches can still deadlock by a lock-order cycle written in the source (the closer takes `pool.lock` inside `inbox.lock` through `Conn`'s `drop`; the task needs `inbox.lock`; main holds `pool.lock` across the scope's end); `[THR-19]` promises to refuse only a wait for a lock this thread holds, so no promise is broken, but the lead should not read the ruling as closing every hang. (7) The 25% and 27% figures are the owner's measurements of other shapes; the atomic cost for task-internal shared values and the one load and one store in the allocation should be measured once on the performance cores before the hardening is cut. (8) The back-edge rule reads "makes a call" from the body's lines; a loop whose only call is to a small inlined method still gets the test, which a later ruling could refine with a "may reach a check point" summary bit without changing meaning. (9) The old texts above are copied with CURRENT-SPEC.md's line wrapping, each verified to occur exactly once; the lead's asserting script should match them as written.
