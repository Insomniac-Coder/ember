# K10: Where and when does a queued thread_shared value's drop run, and how is a wait inside a held lock in a with scope block kept from hanging when a task makes a value another thread lets go of?

## Design 1

**question**: K10: where and when does a queued `thread_shared` value's drop run, and how is a wait inside a held lock in a `with scope` block kept from hanging when a task's thread made a value (a `Conn` travelling inside a plain `Batch`) that another thread lets go of, so that the value is queued for the task's thread, whose last check point is reached while the parent waits at the scope's end holding the lock (method 4's open objection E3-1)? Angle: speed first, then safety.

**summary**: The hang is closed when compiling, at its cause: a task's (or job's, or `@parallel` chunk's) thread empties its own queue at its end, before its end is let go to the parent, so `[THR-11]` now says the task's function counts as possibly running the `drop` of every `thread_shared` value made on its thread and not handed over, from the making line to the task's end; `[THR-19]`'s existing clause ("the end of a `with scope` block, whose task changes what the held lock guards") then refuses the scope's end inside `with pool.lock:` as `E7018`, and `[THR-20]`'s `E7012` list stays as it is (a task is not "another thread"), which removes the contradiction method 4 left. Nothing runs for that rule. For the rest of K10 the ruling keeps method 4's decisions (no queued drop runs inside a lock, a walk, a view or a contract; a sleeping job worker is woken) but makes the mechanism `[BLD-16]`-clean and cheaper: one plain thread-local hold count maintained only by the places that keep drops out (a lock block or guard life, a walk or view rooted in a `static mut`, a function with a contract, a scope whose task touches an unguarded `static mut`), a check point empties the queue only when the hold is zero, the leaving that returns it to zero is a check point, back-edge check points exist only in loops that call or read an `Atomic`, and a worker ending a value outside a job whose `drop` reaches an unguarded `static mut` hands it, dead, to the main thread's queue instead of counting such values atomically from birth.

- EDIT rule=THR-5
  - old: A `with scope` block that ends inside a lock's `with` block, when a task's function changes
  what that lock guards, is `E7018` (`[THR-19]`).
  - new: A `with scope` block that ends inside a lock's `with` block, when a task's function changes
  what that lock guards, the `drop`s its thread may run at its last check point counted as the task's
  (`[THR-11]`), is `E7018` (`[THR-19]`).
  - why: The rule that defines the scope's end names the refusal; it must say that the task's queued drops are among the task's changes, or a reader of `[THR-5]` alone compiles E3-1's program.

- EDIT rule=THR-19
  - old: a join of a thread, or the
  end of a `with scope` block, whose task changes what the held lock guards; a direct call that may
  - new: a join of a thread, a
  `jobs.wait`, or the end of a `with scope` block, whose thread, task or job changes what the held lock
  guards, the `drop`s it runs at its last check point, before its end is let go, included (`[RC-4]`
  point 3, `[THR-11]`, `[THR-20]`); a direct call that may
  - why: The clause that judges a wait for another thread must see what that thread does at its last check point; `jobs.wait` is a wait for a job the text did not name.

- EDIT rule=THR-19
  - old: for ever; join after the `with RESULTS_LOCK:` block: move line 16 below the block". Inside a walk
  - new: for ever; join after the `with RESULTS_LOCK:` block: move line 16 below the block"; "the `with scope`
  block (line 40) ends inside `with pool.lock:` (line 39), and its task makes a `Conn` (line 33, in
  `open_one`) whose `drop` it may run before the scope's end is let go, which takes a `Pool`'s lock
  (line 14), so each would wait for the other for ever; end the scope after the `with pool.lock:`
  block: move lines 40 and 41 above line 39". Inside a walk
  - why: Every refusal gives the line to write (verdicts 43 and 157); the queued-drop case names the making line and the drop's lock, since neither is on the refused line.

- EDIT rule=THR-11
  - old: 
  - new: **A task's queue.** A `thread_shared` value made on a task's or a job's thread, by its body or a
  function it runs, and not handed over (`[RC-4]` point 4) is ended at that thread's check points, the
  last of them at the task's end, before its end is let go to the parent (`[RC-4]` point 3). So the
  task's function counts as possibly running the `drop` of every such value, from its making line to
  the task's end (`[DRP-8]`), and what those `drop`s change is the task's own change, checked as any
  other: against the parent and the other tasks until its join (`E7007`), at a start inside a walk of
  it (`E3020`), and at a wait for the task inside a lock those `drop`s take (`E7018`, `[THR-19]`). It is
  not a change made on another thread: `[THR-20]`'s `E7012` does not apply to it, since the task runs
  beside the main thread only until its join, and a value queued to its thread after that is ended by
  the thread that ends its last owner or, on a job worker, as `[RC-4]` point 3 says. A `jobs.scope()`
  job (`[JOB-2]`) and a `@parallel` chunk (`[PAR-2]`) are tasks for this rule.
  - why: This is where E3-1 is caused: method 4 placed the task's last check point before its join is let go and relied on `[THR-20]` to count it, but `[THR-20]`'s list of functions that may run on another thread rightly excludes scoped tasks (they may reach main's `static mut`). The count belongs with the rule that defines what a task does and checks it until its join (verdict 104), so `[THR-19]`, `[THR-5]`, `E7007` and `E3020` all read it by their existing words, and `E7012` keeps its list. It is a property (what the task's thread may end), not a shape; nothing runs for it.

- EDIT rule=THR-20
  - old: `drop` runs where the last owner ends.** The last owner of a `thread_shared` value may end on any
  - new: `drop` runs where the last owner ends**, or, for a value queued back to the thread that made it
  (`[RC-4]` point 2), at a check point of that thread, where its hold is zero (`[RC-4]` point 3). The
  function of a thread other than the main one that makes such a value, by its lines or a function it
  runs, holds a count of it from the making line to the thread's last check point; a scoped task's or
  job's function the same, judged by `[THR-11]`. The last owner of a `thread_shared` value may end on any
  - why: The headline was untrue for every queued value; the added clause makes the next sentence ("a function that may run on another thread and holds such a count counts as possibly running that value's `drop`") reach a spawned thread's own values, which closes method 4's E2-1 and lets `[THR-19]` judge `h.join()` under a lock, while the scoped case is sent to `[THR-11]` and not to this rule's list.

- EDIT rule=DRP-8
  - old: A `drop` may change things outside its object (a log line, leaving a list).
  - new: A `drop` may change things outside its object (a log line, leaving a list). The ends at a check
  point in a function's lines (`[RC-4]` point 3) run only where the thread's hold is zero, outside every
  place that keeps them out, and are not part of that function's effects (`[EFF-1]`) or summary
  (`[EFF-24]`); what a thread's, task's or job's function may end at its last check point is counted
  in it as `[THR-20]` and `[THR-11]` say.
  - why: `[EFF-1]` and `[EFF-24]` reach every drop through this rule; without the sentence every function with a check point would count the drops of every type its thread may queue (a whole-program fact, `[BLD-16]`), making walks across a looping call `E3020` and `@noblock` loops `E4003`. The last clause names both counting rules, so no function is counted by one and uncounted by the other (E3-1's contradiction).

- EDIT rule=RC-4 point 4
  - old: `System` keeps the two counts, its block waits for the check point and nothing reuses it.
  - new: `System` keeps the two counts, its block waits for the check point and nothing reuses it. A
     `thread_shared` value made inside a function with a contract that forbids an effect of its type's
     `drop` (`@noblock` and a `drop` that takes a lock, `@noalloc` and one that allocates, `[EFF-5]`) is
     marked handed over at its making line too, decided from that function's own declaration: its
     thread's check points are kept out of the function (point 3), which may never return, so its
     `drop` runs where its last owner ends, on the thread that ends it.
  - why: Point 4 is the rule that moves the making thread's count into the away count; a value whose maker can never run its drop belongs there. Method 4's second condition (a drop reaching a module variable of the main thread, made in a job) is no longer needed: point 3 hands such a value to the main thread's queue instead, with no atomic count from birth.

- EDIT rule=RC-4 point 3
  - old:   3. **Check points.** The compiler places them: at every loop's back edge, before and after every
     call that may block the thread (waiting for a lock, a `join`, the end of a `with scope` block,
     a channel's receive, a sleep) and when the thread ends; the one after a `join` or a scope's end
     makes a value a joined task ended on its own thread end before the parent's next line.
  - new:   3. **Check points.** The compiler places them: at the back edge of every loop whose body makes a
     call or reads an `Atomic` (a loop that may wait for another thread; a loop of plain operations on
     this thread's own values has none, and its end is the next point), before and after every call
     that may block the thread (waiting for a lock, a `join`, `jobs.wait`, the end of a `with scope`
     block, a channel's receive, a sleep), when a call C made into Ember returns to C, at the end of
     every task, job and `@parallel` chunk a thread runs, before its end is let go to what waits for
     it (its `join`, its scope's end), and when the thread ends; a job worker waiting for work waits
     at one, and a value put in its empty queue wakes it. Emptying the queue ends, on this thread, the
     values whose counts add up to zero (point 2) and runs their `drop`s there: it is a call of the
     `drop`s of every value that may be in this thread's queue, the `thread_shared` values made on this
     thread, by its own lines or a function it runs, that were not handed over (point 4) (`[DRP-8]`,
     `[WK-3]`). **Where the queue is emptied.** Each thread keeps one plain thread-local count, its
     hold: +1 where it enters a place a queued `drop` may not run inside and -1 where it leaves it, the
     places being a lock's `with` block or a guard's life (`[THR-18]`, `[THR-3]`: the `drop` may take
     that lock), a walk or a view rooted in a `static mut`, from its start to its last use (the `drop`
     may change it, `[BRW-18]`, `[THR-20]`), a function with a contract, from its entry to its return
     (`[EFF-5]`), and a `with scope` block whose task may read or change a `static mut` not guarded by a
     lock (`[THR-11]`); a check point empties the queue only when the hold is zero, and the leaving
     that takes it back to zero is a check point, so a queued value is ended at the thread's first
     point outside every such place, and never waits for a lock this thread holds, changes what it
     walks or views, or breaks a contract it is under. The add and subtract are omitted for a place
     that can reach no check point (a lock block of field accesses, a leaf function with a contract),
     decided from the place's own lines and its callees' summaries (`[EFF-24]`); nothing here is
     decided by a caller or by the program as a whole (`[BLD-16]`). What the queue ends has no owner
     and no opened name left (a name opened from a `weakref thread_shared T` link holds a count,
     `[RC-5]` point 2, and a `weakref T` link reaches no `thread_shared` value, `[THR-2]`), so no view
     of this thread reads it, and `[RC-5]` point 1 does not count a check point as a call that may end
     what a view reads. **The last check point** of a task, a job or a chunk, and of a thread other
     than the main one, is always placed: the `drop`s it may run there are counted in its function,
     from the making line to that point (`[THR-11]`, `[THR-20]`), so every wait for it (`[THR-19]`,
     `[THR-5]`) and every use beside it of what they change (`[THR-11]`) is judged when compiling, and
     nothing of the kind is tested while the program runs; the one after a `join` or a scope's end
     ends a value a joined task let go before the parent's next line, or, where that line is inside a
     place the hold counts, at that place's end. A job worker outside a job (waiting for work, or
     after a job's end is let go) that finds a queued value's counts at zero ends it itself, unless
     its type's `drop` reaches a `static mut` not guarded by a lock (the type's summary, `[EFF-24]`):
     then it puts the value, marked dead, in the main thread's queue, whose next check point runs the
     `drop`, so the module variable stays the main thread's (`[THR-20]`).
  - why: Point 3 is the only place where hidden ends are placed, so it is where J7-01 (no drop inside a held lock or a walk), J7-02 (a sleeping worker is woken), E3-1 (the task's last point is counted in its function) and the worker-after-scope race (the dead value goes to main) are all decided. The hold count replaces method 4's per-call-place judgement and copies: a prebuilt library's loop cannot know whether its caller holds a lock (`[BLD-16]`), but the caller's own block can count. Back-edge points only in loops that call or read an `Atomic` keep a thread-local load and branch out of every leaf loop, so vectorisation and C speed are untouched where it matters.

- EDIT rule=COST-3 row "A `thread_shared` value's two counts"
  - old: a test of the thread's queue at each check point (`[RC-4]`), and the queue's work for a value whose last owner ends on another thread without a hand-over.
  - new: a test of the thread's queue at each check point (`[RC-4]` point 3: one thread-local load and a branch, at the back edge of a loop that calls or reads an `Atomic`, never in a loop of plain operations, around a blocking call and at an end), and the queue's work for a value whose last owner ends on another thread without a hand-over; one plain add and one subtract of the thread's hold count per lock block or guard life, walk or view rooted in a `static mut`, function with a contract, or `with scope` block whose task touches an unguarded `static mut`, only where the place may reach a check point (none for a block of plain field accesses), and one test of the hold where the queue is found non-empty; one wake (a system call) for the first value put in the empty queue of a job worker waiting for work, and one push onto the main thread's queue for a value a worker ends outside a job whose `drop` reaches an unguarded `static mut`; atomic counts from the making line for a value made `using` an instance, or inside a function whose contract forbids an effect of its `drop` (`[RC-4]` point 4), which for that value gives up what the two counts win on its making thread (the 25% below for a thread's own values); no copy of a function per call place and no fact of the whole program (`[BLD-16]`).
  - why: `[COST-5]`: every implicit cost has a row. The row now prices each thing point 3 adds and says what is not paid. The 25% figure is read from the row's own measurements (a thread's own values 25% faster with two counts), not from the 9% to 13% of the random mix, which method 4 misread.

**refused_program**: import std.thread
from std.sync import Atomic, Lock

static DONE: Atomic[int] = Atomic(0)

class Pool:                                   # a connection pool: a free list behind a lock
    lock: Lock = Lock()
    free: Array[int] guarded by lock = []

class Conn:                                   # one connection, made on a scoped task's thread
    id: int
    pool: thread_shared Pool
    fn drop():                                # gives its id back to the pool
        with this.pool.lock:
            this.pool.free.push(owns copy of this.id)

class Batch:                                  # a plain object: connections to close
    conns: Array[thread_shared Conn] = []

class Inbox:                                  # batches waiting for the closer thread
    lock: Lock = Lock()
    items: Array[Batch] guarded by lock = []

fn close_all(inbox: thread_shared Inbox):     # the closer thread lets the batches go
    while DONE.load() == 0:
        with inbox.lock:
            inbox.items.clear()               # may end the Conn's last owner: the Conn is queued to the thread that made it
    with inbox.lock:
        inbox.items.clear()

fn open_one(pool: thread_shared Pool, inbox: thread_shared Inbox):
    with inbox.lock:
        inbox.items.push(owns Batch(conns=[thread_shared Conn(id=1, pool=thread_shared pool)]))   # line 33: the Conn travels inside a plain Batch, so it is not handed over

fn main():
    pool = thread_shared Pool()
    inbox = thread_shared Inbox()
    closer = thread.spawn(owns fn() thread_shared inbox: close_all(inbox))
    with pool.lock:                           # line 39: main holds the pool's lock
        with s = thread.scope():              # line 40
            s.spawn(owns fn(): open_one(pool, inbox))   # line 41
        println(pool.free.len())              # line 42
    DONE.store(1)
    closer.join()
    with pool.lock:
        println(pool.free.len())

# Refused when compiling, E7018 at line 40 ([THR-19] with [THR-11]):
# "the `with scope` block (line 40) ends inside `with pool.lock:` (line 39), and its task makes a `Conn`
#  (line 33, in `open_one`) whose `drop` it may run before the scope's end is let go, which takes a
#  `Pool`'s lock (line 14), so each would wait for the other for ever; end the scope after the
#  `with pool.lock:` block: move lines 40 and 41 above line 39"
# Why it would hang under the current text: the closer's clear puts the Conn in the task thread's queue;
# the task's last check point runs Conn's drop, which waits for pool.lock; main holds it while waiting
# for the scope's end. The task's function now counts that drop (it changes `pool.free` under
# `pool.lock`), so [THR-19]'s clause "the end of a `with scope` block, whose task changes what the held
# lock guards" refuses it, with no loop needed and nothing checked while running.

**accepted_program**: import std.thread
from std.sync import Atomic, Lock

static DONE: Atomic[int] = Atomic(0)

class Pool:
    lock: Lock = Lock()
    free: Array[int] guarded by lock = []

class Conn:
    id: int
    pool: thread_shared Pool
    fn drop():
        with this.pool.lock:
            this.pool.free.push(owns copy of this.id)

class Batch:
    conns: Array[thread_shared Conn] = []

class Inbox:
    lock: Lock = Lock()
    items: Array[Batch] guarded by lock = []

fn close_all(inbox: thread_shared Inbox):
    while DONE.load() == 0:
        with inbox.lock:
            inbox.items.clear()
    with inbox.lock:
        inbox.items.clear()

fn open_one(pool: thread_shared Pool, inbox: thread_shared Inbox):
    with inbox.lock:
        inbox.items.push(owns Batch(conns=[thread_shared Conn(id=1, pool=thread_shared pool)]))

fn main():
    pool = thread_shared Pool()
    inbox = thread_shared Inbox()
    closer = thread.spawn(owns fn() thread_shared inbox: close_all(inbox))
    with s = thread.scope():                  # the scope ends outside the pool's lock
        s.spawn(owns fn(): open_one(pool, inbox))
    DONE.store(1)
    closer.join()
    with pool.lock:
        println(pool.free.len())

# Compiles and prints `1`, on every run. Two timings, one outcome:
# - the closer clears the inbox before the task ends: the Conn goes to the task thread's queue; the task's
#   last check point (hold zero, before its end is let go) runs Conn's drop, takes pool.lock (free), pushes 1;
# - the closer clears after the task ended: the task thread's queue is closed ([RC-4] point 3), so the
#   closer ends the Conn at once and its drop pushes 1 on the closer's thread; close_all holds a count of
#   the Conn through the inbox, so [THR-20] counts that drop in it: it takes a Pool's lock, not the Inbox's
#   lock the closer holds, and reaches no module variable, so nothing is refused.
# Also still accepted, unchanged: J7-01's program (main holds pool.lock while walking pool.free, and a
# worker lets go of a Conn main made): the inner walk's back edge is inside a lock block, where main's
# hold is one, so the queue is not emptied there; the end of the `with pool.lock:` block takes the hold to
# zero and is a check point: Conn's drop runs there, pushes 1, and later frames and `h.join()` print 1.

**runtime_cost**: For the rule that closes E3-1 (the task's function counts its thread's queued drops; `[THR-19]`, `[THR-5]`, `[THR-11]`, `[THR-20]`, `[DRP-8]`): nothing; it is decided when compiling. For the K10 mechanism carried with it, row "A `thread_shared` value's two counts" of §X.4 as edited: one thread-local load and a branch per check point, placed at the back edge of a loop that calls or reads an `Atomic` (none in a loop of plain operations, so leaf loops keep C's code and vectorise), around a blocking call, at a task's, job's or chunk's end before its end is let go, at a thread's end, at a return to C and at a worker's idle wait; one plain add and one subtract of the thread's hold count per lock block or guard life, walk or view rooted in a `static mut`, function with a contract, or scope block whose task touches an unguarded `static mut`, only where the place may reach a check point (omitted for a block of plain field accesses or a leaf contract function), and one test of the hold only on the path where the queue is found non-empty; one futex wake for the first value put in an idle worker's empty queue; one push onto the main thread's queue for a value a worker ends outside a job whose drop reaches an unguarded `static mut`; atomic counts from the making line only for a value made `using` an instance or inside a contract function whose contract forbids an effect of its drop. Removed against the current text: the load and branch at the back edge of every leaf loop. Removed against method 4: the copy of a function per call place and its per-place judgement.

**implementer_steps**: 1. Summaries (`[EFF-24]`, written to interface files `[EFF-4]`/`[MOD-9]`): for each class with a `drop`, record what the drop may take (locks), change (`static mut`s, guarded fields), end, block or allocate; emit one bit in the type's table, "drop reaches an unguarded `static mut`". For each function, record (a) the set of `thread_shared` types it makes whose owner may travel (a making line, or a callee that makes one, where not every path stores the owner at a point-4 place or ends it on this thread; when unsure, it may travel), and (b) "may reach a check point" (it has one, or calls a function whose summary has the bit).
2. Effects of a unit of another thread (`thread.spawn`'s and `jobs.submit`'s function, a scoped task's or job's body, a `@parallel` chunk): add the drops of its may-travel types (its own and its callees', `[DRP-8]`) to the unit's effect set, from the making line to the unit's end. `[EFF-26]` carries them to the spawn or submit; `[THR-19]` reads them at a join, `jobs.wait` or scope end under a lock (`E7018`), `[THR-11]` against the parent and the other tasks (`E7007`) and at a start inside a walk (`E3020`), `[THR-20]` for an unscoped unit's module variables (`E7012`), the contract checker for a wait inside a contract. In-body check points contribute nothing to any summary (`[DRP-8]`).
3. Check-point placement, per function, from its own lines only: a back edge of a loop whose body contains a call or an `Atomic` read; before and after a blocking call; the unit's end (emitted before the join's release: the task stores its result and empties its queue, then signals); the thread's end; the return of a C entry (`[FFI-22]`); the worker's idle wait. Each is `if (tls->queue) ember_rt_drain(tls);`.
4. Hold places, per function: at the entry of a lock block or guard life, a `static mut` walk or view (its root is a `static mut`, read from the expression and from a view's `static` region), a function with a contract, and a scope block whose tasks' summaries touch an unguarded `static mut`, emit `tls->hold++`; at the exit `if (--tls->hold == 0 && tls->queue) ember_rt_drain(tls);`. Omit both where the place's lines and its callees' "may reach a check point" bits show no point inside. A generator paused inside a `static mut` walk keeps the increment until the walk ends or the generator is ended.
5. Runtime (`ember_rt.h`, M9): `ember_rt_drain` returns at once if `hold != 0`; otherwise pops the queue (an acquire, `[RT-8]`) and for each value adds home into away; sum zero: if this thread is a job worker outside a job and the type bit is set, push the value marked dead onto the main thread's queue, else run the drop and free with point 2's compare-exchange; sum positive: move home into away, clear queued, set handed over. Main's drain runs dead entries' drops directly. A push onto a worker's queue that finds the idle mark in the queue word does one futex wake. A thread's end: exchange the queue head with CLOSED after a final drain (a task's end does this before signalling its join).
6. Point 4 at making lines: emit the handed-over mark at the making line for a value made `using` an instance (already) and for a value made inside a declared-contract function whose contract forbids an effect of the type's drop (the function's own attribute and the type's summary; no caller fact).
7. Separate compilation and hot reload: every decision above reads the function's own lines, the type's interface summary and callees' summaries (`[BLD-16]`); the queue, the hold and the idle mark live in the thread's runtime record (`[HR-3a]`), not in the image, so a reload keeps them; a frame below a reload checkpoint runs its old code to its exit (`[HR-3b]`), so each increment meets its decrement; a dead value's drop runs the current code from the type's table. FFI: a host thread's hold is zero at its return to C (Ember regions close before a return), so the return's check point drains; values let go while the thread is in C follow method 4's decision 4, left to the owner.
8. Tests: the refused program (E7018 at the scope's end, the message above); the accepted program (prints 1); the same with `h.join()` on the task's handle under the lock (E7018); a task whose made value's drop pushes to an unguarded `static mut` the parent reads during the scope (E7007); J7-01's program (compiles, drop at the lock block's end); J7-02's program (workers woken, two textures freed right after the clear); a worker-made value whose drop pushes main's `static mut`, let go by main after the scope (ended on main, counted once); break tests removing each of steps 2, 4 and 5.

**departs_from_verdict**: None. Verdict 101 is kept: the making thread deletes a queued value at a check point the compiler places, always before the thread ends (a task's or job's thread at its end, before its end is let go), or the thread that ends the last owner deletes it at once when the maker has ended; the hand-over at a making line under a contract is fix 2 taken where the compiler sees the maker can never end it; the dead value a worker hands to main is deleted by the maker's own drain, which only runs the drop elsewhere. Verdict 43 kept: every wait for a held lock is refused when compiling (the hold count tests no wait; it schedules the drain, as the queue test already does, and the hang program is E7018). Verdict 57 kept: an end counts as a call of the drop where it is placed, and a unit's last check point is in its function; in-body points are left out of a summary only because the hold keeps them where no rule's premise holds. Verdict 104 kept: a task's claims, its queued drops included, are checked until its join. Verdicts 56, 114, 122, 165, 166 untouched. Verdict 86 kept (nothing is written by the programmer). Verdict 88 is not needed here (no copies). Alternative considered and not applied, marked "departs from verdict 101": the task's end hands its non-empty queue to the parent (one atomic splice on that rare path), and the parent ends the values at its next check point outside its hold places; it refuses nothing, but a third thread, not the maker and not the one that ended the last owner, deletes the value, and not at once. The owner may prefer it for Python-like ease; its only run-time cost is that splice.

**trade_offs**: Speed: nothing new runs for the hang's closing; the carried mechanism costs a thread-local load and branch only in loops that already call or read an atomic (leaf loops lose the per-iteration test the current text charges them, a gain), two plain thread-local adds per lock block or contract function that may reach a check point (next to the lock's own atomic, or the call, these are noise), and one wake or one queue push on rare paths; given up: method 4's zero-instruction placement, which needed a copy of a function per call place and a whole-program fact a prebuilt library cannot have. Readability: one refusal where the program may hang (a scope's end or join under a lock the task's queued drops take), with a one-line fix in the message, like verdict 43's "a different Log is refused too"; a queued drop's visible effect may come a little later, at the end of the lock block, walk, view or contract the thread is in, and a task's drop at its end is now part of what the task does, so a parent that reads an unguarded `static mut` the drop changes during the scope is E7007 (a real race). Safety: never compromised; no drop runs inside a lock it takes, a walk or view it changes, or a contract it breaks; what the queue ends has no owner, so no view reads it; a worker's drop touching main's module variable runs on main. Consistency: one place (RC-4 point 3) places every hidden end, THR-11 and THR-20 together say which function is charged with a thread's last point, and DRP-8 names both, so no program is refused by one rule and compiled by another. Residual accepted: a value made by a helper called from a contract function that never returns waits for that function's return (the helper has no declared contract, and a caller fact may not decide its code).

**remaining_doubt**: (1) Whether `thread.scope()` tasks run on threads of their own that end at the join or on job workers is not stated; the text is written to hold either way (an ended thread's values go to whoever ends the last owner; a living worker's are drained at its idle wait, with the hand to main for module variables), but the lead should pin it, since the two paths run the drop on different threads. (2) Verdict 43's "nothing is checked while running": the hold test runs only on the path where the queue is non-empty and tests no wait, so I read it as kept, but the owner may see any run-time branch in a lock block as a check; if so, method 4's per-place copies are the only alternative and they conflict with `[BLD-16]` for prebuilt libraries. (3) The "may travel" judgement at a making line is an over-approximation: where the compiler cannot prove every owner reaches a point-4 place, the task's function is charged with the drop, which may refuse a scope's end under a lock that could not hang in practice. (4) A generator paused inside a walk of a `static mut` holds the count across its suspension; a `Send` generator moved to another thread would carry a hold it cannot release there, so such a generator should not be `Send`, which `[THR-8]` does not yet say. (5) Method 4's decision 4 (a value queued to a host thread that has returned to C) stays with the owner; the hold is zero at a return to C, so option a (freed at the next call into Ember) needs no new mechanism, and option b (whoever lets go frees it while the thread is in C) needs one atomic step per return to C. (6) The hand of a dead value to main's queue assumes main is alive; when main has returned the process ends (`[THR-16]`), so no drop is owed. (7) The cost of two thread-local adds in a lock block that calls a looping library function was not measured; it should be priced against the lock's own cost on the P-cores before the row's words are final.

## Design 2

**question**: K10: Where and when does a queued `thread_shared` value's drop run, and how is a wait inside a held lock in a `with scope` block kept from hanging when a task makes a value another thread lets go of (the method-4 design's open objection E3-1)? Angle: safety first, then cost.

**summary**: The queue is made harmless instead of the check points being made clever: a `thread_shared` value is marked handed over at the line that makes it (the mark `[RC-4]` point 4 already sets for a value made `using` an instance) in two more cases, decided by a property, never by a program's shape: (A) its type is not plain (its `drop`, or that of anything it owns all the way down, `thread_shared` owners included, has an effect a contract may forbid, reaches a `static mut`, or runs code the compiler cannot read whole), and (B) it is made on any thread but the main thread or a `thread.spawn` thread (a scope's task or job, a worker's chunk, a host thread), whose end no `join` waits for and whose claims the rules judge only until the join. So the queue holds only a value made on main or a spawned thread whose `drop` frees memory and ends what it owns and nothing else; such a drop breaks no rule wherever it runs, so a check point may stand anywhere (inside a held lock, a walk, a view, a `with scope` block), the thread's last one runs before its `join` returns, and every drop with outside effects runs where its last owner ends, as a written call the existing rules (`[THR-19]`, `[THR-20]`, `[THR-11]`, `[BRW-18]`, `[RC-5]`) already judge. E3-1's program compiles and prints `0` then `1` (the closer thread runs the Conn's drop and waits for the pool's lock until main's block ends); nothing new is refused, nothing is checked while running beyond one load and one store inside the allocation of a `thread_shared` block, and no copies, per-thread numbers or worker wakes are needed.

- EDIT rule=DRP-8
  - old:   A `drop` may change things outside its object (a log line, leaving a list).
  - new:   A `drop` may change things outside its object (a log line, leaving a list). The one end no line
  writes, a queued `thread_shared` value's at a check point, is the end of a plain type (`[RC-4]`
  point 4), and `[RC-4]` point 3 says what it is for the rules that judge the lines around it.
  - why: DRP-8 is where every rule (EFF-1, EFF-24, BRW-18, RC-5 point 1) is told that an end is a call of the drop; without this sentence a check point inside a function would be read as a call with effects, or as nothing, and the text would not say which. It points to the one place that decides.

- EDIT rule=THR-20
  - old: **A
  `drop` runs where the last owner ends.** The last owner of a `thread_shared` value may end on any
  thread that holds a count of it:
  - new: **A
  `drop` runs where the last owner ends**, or, for a value of a plain type queued back to the thread
  that made it (`[RC-4]` points 2 and 4), at that thread's next check point, where a plain `drop`
  reaches nothing this Part judges; a `drop` that is not plain never waits in a queue, so it runs
  where its last owner ends, on the thread this rule judges. The last owner of a `thread_shared`
  value may end on any thread that holds a count of it:
  - why: The headline was untrue for every queued value (its drop runs later, on the making thread). With the plain-type rule it is true in the only way that matters to this Part: every drop that reaches a module variable, a lock or anything another thread's rules judge runs where the last owner ends, which is exactly what the next sentence (a function holding a count counts as running the drop; E7012) already judges.

- EDIT rule=THR-11
  - old: A value a task or a job takes with `owns` is handed to
  another thread: `[THR-2]`'s compare and move happen at the spawn, so every link on this thread
  then finds it ended, and the task ends it with no header this thread reads.
  - new: A value a task or a job takes with `owns` is handed to
  another thread: `[THR-2]`'s compare and move happen at the spawn, so every link on this thread
  then finds it ended, and the task ends it with no header this thread reads. A `thread_shared` value
  a task or a job makes has no one-thread count either: it is marked handed over at its making
  (`[RC-4]` point 4, by its thread), so nothing is ever queued to the task's thread to be ended after
  the join, and its `drop` runs where its last owner ends, which this rule judges until the join and
  `[THR-20]` after it.
  - why: THR-11's own principle, 'A task touches no one-thread count', is completed: a task's values have no home count. This is the sentence E3-1 asked for: a task's thread is never asked to run a drop on the task's behalf, before or after the join, so the end of a `with scope` block inside a held lock is judged by the task's visible summary alone (THR-19, THR-5) and can neither hang nor need a new refusal.

- EDIT rule=HR-44
  - old: (`[HR-14]`), and is computed again on the next call. Nothing of this runs while the program runs.
  - new: (`[HR-14]`), and is computed again on the next call. Whether a type is plain (`[RC-4]` point 4) is
  such a fact for every line that makes a `thread_shared` value of it; when a reload takes it away,
  prepare sets the handed-over mark on every live `thread_shared` value of the type while it walks
  the list (`[HR-12]`), one store each, so none waits in a queue whose check points may no longer
  run its `drop`. Nothing else of this runs while the program runs.
  - why: Hot reload is in scope: a drop that becomes non-plain after a reload would otherwise run at a check point of a value made unmarked. The commit already walks the live-instance list with every thread at depth zero (HR-12, HR-3), so the fix costs one store per live value of that type, at the reload only.

- EDIT rule=RC-4 point 5
  - old:   5. **So a value's end is delayed only** when its last owner ends on a thread that did not make it
     and the value was not handed over at a place of point 4 (it travelled inside another object):
     it is ended at its making thread's next check point.
  - new:   5. **So a value's end is delayed only** for a value of a plain type made on the main thread or a
     `thread.spawn` thread (point 4), when its last owner ends on a thread that did not make it and
     the value was not handed over at a place of point 4 (it travelled inside another object): it is
     ended at its making thread's next check point, and its `drop`, which is plain, runs there.
  - why: Point 5 is the summary a reader and WK-3 rely on; it must name the only values that can wait.

- EDIT rule=RC-4 point 3
  - old:      a channel's receive, a sleep) and when the thread ends; the one after a `join` or a scope's end
     makes a value a joined task ended on its own thread end before the parent's next line.
  - new:      a channel's receive, a sleep) and when the thread ends, before its `join` returns; the one after
     a `join` or a scope's end makes a value a joined task ended on its own thread end before the
     parent's next line. What a check point ends is a value this thread made whose type is plain
     (point 4): its `drop` has no effect a contract forbids (§X.2), takes no lock, reaches no
     `static mut` and runs only code the compiler read whole, and the values it ends have no owner
     left, so no view, walk or task of this thread reads them. A check point is therefore not a line
     or a call for the rules that judge where it stands (`[THR-19]`, `[BRW-18]`, `[RC-5]`,
     `[THR-11]`, `[EFF-1]`, §X.2), adds nothing to its function's effects or summary (`[EFF-24]`,
     `[DRP-8]`), and may stand inside a lock's `with` block or a guard's life, a walk, a view's life, a
     `with scope` block or a function with a contract.
  - why: Point 3 is where hidden ends are placed, so it must say what they may do there. Instead of moving the points away from locks, walks, views and scopes (which needs a per-place judgement across calls, copies or a per-thread number, and still left E3-1 open), the text says what the queue can hold, and from that the placement is free. 'Before its `join` returns' makes `join` mean done, as verdict 101's 'always before the thread ends' intends.

- EDIT rule=RC-4 point 4
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
     `Panic(Explicit)`, `RuntimeCheck(Aliasing)`, `Nondet`), its summary (`[EFF-24]`) reaches no
     `static mut` and runs no code the compiler cannot read whole (a call through a function value,
     a `dyn` or `virtual` method, a foreign call not declared `@ffi(no_callbacks)`), and every type
     the value owns is plain, all the way down, `thread_shared` owners included (a `dyn I`, a
     function value and an object of an `open` or `abstract` class it owns are not). The package
     that declares the type decides, from its own text and its imports' interfaces, and records the
     fact in its interface file beside the type's layout (`[BLD-14]`, `[BLD-16]`), as it records the
     weak count (`[OBJ-1]`). A `thread_shared` value of a type that is not plain is marked handed
     over at the line that makes it, so its `drop` never waits in a queue and runs where its last
     owner ends, as a written call there. **By its thread.** A `thread_shared` value made on any
     thread other than the main thread or a `thread.spawn` thread (the thread of a scope's task or
     job, a job worker running `jobs.submit` or a chunk of a `@parallel` loop, a host thread attached
     by `[FFI-22]`; a chunk the calling thread runs itself, `[PAR-4]`, is made on that thread) is
     marked handed over at its making: those threads run Ember code only while given a task, a job
     or a call from C, no `join` of the program waits for their end, and a task's claims are checked
     only until its join (`[THR-11]`), so a queue to them would have no bound and could run a `drop`
     after the window the rules checked. The runtime, which lays out the block and knows its
     thread's kind, sets the mark in the header it writes (§X.4). So the queue of point 2 holds only
     a value of a plain type made on the main thread or a `thread.spawn` thread.
  - why: Point 4 is the rule that moves a value's count into the away count, and the `using`-instance sentence already does exactly this at the making line (N84). The two new cases are the two causes of K10: a drop with outside effects waiting for a hidden point (J7-01, decision 3's race), and a queue to a thread whose end nothing waits for or whose claims end at the join (J7-02, decision 4, E3-1). Both are properties, both decidable by the package that compiles the line or by the runtime from a fact that exists only while the program runs (BLD-16's 'the test is paid', here one load inside an allocation).

- EDIT rule=COST-3 row "A `thread_shared` value's two counts"
  - old: a test of the thread's queue at each check point (`[RC-4]`), and the queue's work for a value whose last owner ends on another thread without a hand-over.
  - new: a test of the thread's queue at each check point (`[RC-4]`), one thread-local load and a branch, and the queue's work for a value whose last owner ends on another thread without a hand-over; a value of a type that is not plain, and a value made on a thread other than the main thread or a `thread.spawn` thread (`[RC-4]` point 4), count every owner atomically as if handed over at the making line, the second marked by the runtime in the header it writes (one load of the thread's kind and one store, inside the allocation): for such a value this row's measurements read the other way, one atomic count against two (a pipeline, made on one thread and ended on another, 27% faster on one thread and the same on 2 to 8; a thread's own values 25% slower).
  - why: COST-5 asks for a row for every implicit cost; the two marks change which values pay the atomic path and add one load and one store to an allocation.

**refused_program**: import std.thread
from std.sync import Lock

class Pool:                                   # a connection pool: a free list behind a lock
    lock: Lock = Lock()
    free: Array[int] guarded by lock = []

class Conn:                                   # one connection
    id: int
    pool: thread_shared Pool
    fn drop():                                # gives its id back to the pool
        id = this.id
        with this.pool.lock:
            this.pool.free.push(owns id)

fn use_one(pool: thread_shared Pool):
    c = thread_shared Conn(id=1, pool=thread_shared pool)
    println(c.id)                             # `c` ends here, so `use_one` runs `Conn`'s drop

fn main():
    pool = thread_shared Pool()
    with pool.lock:
        t = thread.spawn(owns fn() thread_shared pool: use_one(pool))
        t.join()
        println(pool.free.len())

# E7018 at line 24: "`t.join()` (line 24) waits for the thread started on line 23, whose function
# changes `pool.free` under `pool.lock` (`use_one` ends `c` on line 18, and `Conn`'s `drop`, line 13,
# takes the lock), and this thread holds that lock, so each would wait for the other for ever; join
# after the `with pool.lock:` block: move line 24 below the block".
# Refused by the current text as well ([THR-19] "a join of a thread ... whose task changes what the
# held lock guards", with [THR-20] "a function that may run on another thread and holds such a count
# counts as possibly running that value's drop"). The ruling adds no refusal: every change is to
# which thread ends a value and when, never to which programs compile.

**accepted_program**: import std.thread
from std.sync import Atomic, Lock

static DONE: Atomic[int] = Atomic(0)

class Pool:                                   # a connection pool: a free list behind a lock
    lock: Lock = Lock()
    free: Array[int] guarded by lock = []

class Conn:                                   # one connection, made on a scoped task's thread
    id: int
    pool: thread_shared Pool
    fn drop():                                # gives its id back to the pool: takes a lock, so `Conn` is not plain
        id = this.id
        with this.pool.lock:
            this.pool.free.push(owns id)

class Batch:                                  # a plain object: connections to close
    conns: Array[thread_shared Conn] = []

class Inbox:                                  # batches waiting for the closer thread
    lock: Lock = Lock()
    items: Array[Batch] guarded by lock = []

fn close_all(inbox: thread_shared Inbox):     # the closer thread lets the batches go
    while DONE.load() == 0:
        with inbox.lock:
            inbox.items.clear()               # ends each Batch and its Conn: the Conn's drop runs here, on this thread
    with inbox.lock:
        inbox.items.clear()

fn open_one(pool: thread_shared Pool, inbox: thread_shared Inbox):
    with inbox.lock:
        inbox.items.push(owns Batch(conns=[thread_shared Conn(id=1, pool=thread_shared pool)]))

fn main():
    pool = thread_shared Pool()
    inbox = thread_shared Inbox()
    closer = thread.spawn(owns fn() thread_shared inbox: close_all(inbox))
    with pool.lock:
        with s = thread.scope():
            s.spawn(owns fn(): open_one(pool, inbox))
        println(pool.free.len())              # 0: the Conn's drop, on the closer, waits for this lock
    DONE.store(1)
    closer.join()
    with pool.lock:
        println(pool.free.len())              # 1

# This is E3-1's program. It compiles: the task's summary changes `inbox.items` under `inbox.lock`, not
# `pool.free`, so the scope's end under `pool.lock` is not [THR-19]'s case; the closer's `clear()` is a
# written end of the Conn ([DRP-8]), taking a `Pool`'s lock while holding an `Inbox`'s, two kinds.
# It runs: the Conn is marked handed over at its making twice over (a task's thread; a type that is
# not plain), so nothing is queued to the task's thread, the task ends with nothing hidden to run, the
# scope ends, main prints 0 and lets the lock go, the closer's waiting drop pushes 1, and after the
# join main prints 1. Output: 0 then 1, on every run.

**runtime_cost**: Row "A `thread_shared` value's two counts" (§X.4, [COST-3]). What runs: (1) inside the runtime's allocation of a `thread_shared` block, one load of the thread's kind (a thread-local the chunk allocation already consults) and one store of the handed-over mark into the header it is writing, for a value made on a thread other than main or a `thread.spawn` thread; (2) for a value of a type that is not plain, the making line emits the marked making the `using`-instance making already uses (no test: the fact is compiled in); (3) for every value so marked, each count operation is the existing atomic away-path operation instead of a plain add, measured as one atomic count against two: a pipeline 27% faster on one thread, a thread's own values 25% slower; (4) a check point stays one thread-local load and a branch, now also inside lock blocks, walks and scope blocks, with none moved and none added; (5) nothing per check point for placement, no per-thread hold number, no function copies, no worker wake, no step at a return to C; (6) at a hot reload that makes a type stop being plain, one store per live value of that type during the walk prepare already makes.

**implementer_steps**: 1. Interface fact "plain" per type: in the package that declares a type, compute from the drop's effect set (EFF-1) and summary (EFF-24) and from the plain bits of every owned type (imports' interface files; thread_shared owners included; `dyn I`, function values, `open`/`abstract` owned objects are not plain; a greatest fixpoint over the package's own type graph for cycles). Write the bit beside the layout in the interface file (BLD-14 item list) and in `TypeInfo` (RT-3) for `ember inspect --cost`. 2. Making lines: when the compiler lowers `thread_shared T(...)` (and generic instances, MONO-1), if T's plain bit is clear it emits the marked-making entry point already used for `using` an instance (N84): home 0, away 1, handed-over mark set. 3. Runtime thread kind: a thread-local flag "home thread" set for main at start-up and for threads `thread.spawn` starts; left clear for job workers, for any thread the runtime starts to run a scope's tasks, and for host threads at `ember_rt_thread_attach()`. The allocation of a `thread_shared` block in `Global` reads it: clear means mark set, away 1, home 0 (the chunk record is still written, unused for marked values). A `@parallel` chunk the calling thread runs itself inherits that thread's flag by construction. 4. Check points: keep the current placement (back edges, before and after blocking calls, the thread's end, after a join or a scope's end); remove nothing, add nothing, and emit them inside lock blocks, walks and scope blocks as anywhere else. No contribution to effects or summaries. 5. Thread end: the runtime's thread-exit path drains and closes the queue (RC-4 point 3) before it signals the join handle, so `join` returns after the last queued drop. 6. Hot reload: plan compares the plain bit as a fact (HR-44); when it goes from set to clear, prepare sets the handed-over mark on each live `thread_shared` instance of that type during its list walk (HR-12). 7. Diagnostics: no new code; THR-19's join and scope-end messages unchanged. 8. Tests: E3-1's program prints `0 1`; J7-01's program (main's Conn ended by a worker) prints frames then `1`, the drop running on the worker when main's lock is free; J7-02's program prints "texture 1 freed", "texture 2 freed" during `bundles.clear()` on main; a scoped job that makes a Texture whose drop writes main's `static mut LOG` and that main clears after the join writes LOG on main only (no race); a `thread.spawn` thread's own plain values keep the plain-add path (the 25% row); an `ember inspect --cost` run names the row for a marked making.

**departs_from_verdict**: None. Verdict 101 (two numbers; the making thread deletes at its next check point, always before the thread ends; fix 2 moves A into B at a hand-over the compiler sees) is kept: the two counts stay for every value made on main or a spawned thread with a plain type, the queue and check points stay, and the two new marks are fix 2's step taken at the making line, as the lead's N84 already took it for a value made `using` an instance; the owner should see that the set of values counted atomically from the start is wider than N84's (touches 101, within fix 2). Verdict 43 kept: nothing is checked while running and no hidden end can wait for a lock its thread holds, because a drop that takes a lock never waits in a queue. Verdict 57 kept: every end with outside effects runs as a written call where the last owner ends, so BRW-18, RC-5 and the summaries see it; the only end no line writes changes nothing outside its object. Verdict 104 kept: a task's claims end at its join, and nothing runs on its behalf after it. Verdict 114 kept: a one-thread object is never queued. Verdicts 86 and 88 untouched (no copies are needed). The method-4 team's decision 4, left for the owner (a value queued to a host thread gone back to C), is decided here by the by-thread mark, its option c narrowed to host-thread-made values, with no per-call step; the owner may still prefer their option b. The method-4 team's decisions 1 (placement by judgement), 2 (worker wake) and 3 (atomic from the making line only for drops reaching main's statics in jobs) are superseded by the same two sentences, which the arbiter's E3-1 showed those decisions could not close together.

**trade_offs**: Given up: the plain-add speed of the two-count design for two sets of values. (i) Values of types whose drop does anything but free memory and end what it owns (logs, takes a lock, writes an atomic, calls C with callbacks, reaches a `static mut`): every count operation on them is atomic, the cost the owner measured as 9% to 13% for a random mix against two counts; such a drop already pays a lock or a syscall, and the owner's measured gains (a thread's own values, the camera program) are on plain data. (ii) Values made in scoped tasks, jobs, worker-run chunks and C callbacks: atomic from the start; for the usual shape (made on a worker, consumed elsewhere) the measured pipeline row says this is 27% faster than two counts on one thread and equal on more, and only a task that makes and ends its own `thread_shared` values locally loses the 25%, a shape `owns` serves without any count. Against the four goals: speed, nothing new runs at check points and nothing is added to a hot loop (one load and one store move into an allocation that already writes the header); readability, nothing to write and nothing refused that compiles today, `join` means done, a queued drop's output may appear later only for a value whose drop prints nothing (it is plain); safety, no hidden end can ever run where its written call would be refused (lock, walk, view, task, contract) or after the window THR-11 checked (the post-join race of a worker-run drop on main's `static mut` is gone), and no value waits for a thread that sleeps in the pool or in C; consistency, THR-20's headline becomes true, THR-11's 'a task touches no one-thread count' becomes complete, verdict 43's promise holds with no list of rules, and the compiler needs no per-place judgement, copies or per-thread number, so BLD-16 is met by construction (the plain bit is a declaring-package fact in the interface file; the thread's kind is run-time data read where the block is laid out). Compared with the method-4 design: fewer mechanisms (no placement analysis, no wake, no return-to-C point), one honest cost row instead of three, and one closed contradiction instead of an open one.

**remaining_doubt**: (1) A `thread.spawn` thread blocked for its life in one call (a hand-written worker in `rx.recv()`) runs its check point before and after the call, so a plain value queued to it while it blocks is freed only when the call returns or the thread ends: memory held, never a hang or a resource (a resource-holding drop is not plain and never queues). The owner may want the text to name this bound in point 5 or to pay a wake (one futex per first queued value) as the method-4 team proposed for pool workers; this ruling does not. (2) The 25% and 27% figures are the owner's measurements of other shapes; the cost of atomics on task-made values should be measured once (the pipeline row is the nearest evidence). (3) Plainness excludes `Io`, so a drop that writes a log file counts atomically; the alternative, allowing `Io`, would need check points kept out of `@noio` functions transitively, which a separately compiled callee cannot know (BLD-16), so the stricter set was chosen. (4) The by-thread mark assumes the runtime starts every scope task's thread or runs it on a worker it owns; if an implementation ran a scope's task on a thread the program made with `thread.spawn`, the flag of that thread would have to be cleared for the task's duration; THR-5 and §XI.5 do not say where a `thread.scope()` task runs, and the lead may wish to pin it. (5) The wording 'a check point is not a line or a call for the rules that judge where it stands' relies on the argument that what the queue ends has no owner left so no view, walk or task reads it; a `weakref thread_shared T` name holds a count (RC-5 point 2), so its target is never at zero; I found no view that escapes this, but it is the one claim an attacker should try again.

## Design 3

**question**: K10: Where and when does a queued `thread_shared` value's `drop` run, and how is a wait inside a held lock in a `with scope` block kept from hanging when a task makes a value another thread lets go of? (Angle: the newcomer's one-sentence rule first, then safety and cost.)

**summary**: The newcomer's rule: a `thread_shared` value made by `main` or by a `thread.spawn` thread goes home to be ended, at that thread's next check point where it holds no lock; a value made anywhere else (a scoped task's thread, a job worker, a `@parallel` chunk, a thread C owns) is counted with atomics from its making line and is ended wherever its last owner is let go. So nothing is ever queued to a thread that ends at a join or sleeps in the pool, which removes the scope hang (E3-1) and the idle-worker leak (J7-02) with no check at the join, no wake and no new refusal; it is the step `[RC-4]` point 4 already takes for a value made `using` an instance, and `[THR-11]`'s own sentence "a task touches no one-thread count" now covers the home count too (verdict 166). For the threads that keep a home count, a check point empties the queue only while the thread holds no lock, kept by a plain per-thread count of held locks whose release to zero is itself a check point (J7-01), a check point is added where a callback returns to C and none stands inside a function carrying a contract; a `thread.spawn` thread's function counts the `drop`s of the `thread_shared` values made on it, so a `join` of it inside a lock those `drop`s take is `E7018` when compiling, as verdict 43 and `[THR-19]` promise.

- EDIT rule=RC-4 point 1
  - old:      Only the making thread reads both. When the making thread's own subtract takes the home count
  - new:      Only the making thread reads both. A home count belongs only to a thread of the program's own,
     the main thread or a `thread.spawn` thread, which lives until its own end: a value made on any
     other thread (a scoped task's, `[THR-11]`; a job worker's, a `@parallel` loop's chunk included,
     §XI.5; a thread C owns, `[FFI-22]`) is marked handed over at the line that makes it, as a value
     made `using` an instance is (point 4), so every owner of it is counted in its away count,
     nothing is ever queued to such a thread, and its `drop` runs where its last owner ends
     (`[THR-20]`). When the making thread's own subtract takes the home count
  - why: The cause of the scope hang: a task's thread had a home count and a queue, so a value it made and another thread let go of waited for the task thread's end, which the parent awaits inside its lock. A thread that is joined as a task, or that sleeps in the pool, can never be relied on to run a drop in time; the value is counted away from the start, the mechanism point 4 already uses for `using` an instance (N84).

- EDIT rule=RC-4 point 5
  - old:   5. **So a value's end is delayed only** when its last owner ends on a thread that did not make it
     and the value was not handed over at a place of point 4 (it travelled inside another object):
     it is ended at its making thread's next check point. Until then it is intact, and a `weakref`
  - new:   5. **So a value's end is delayed only** when its last owner ends on a thread that did not make it,
     the making thread is the main thread or a `thread.spawn` thread (point 1), and the value was not
     handed over at a place of point 4 (it travelled inside another object): it is ended at its
     making thread's next check point at which that thread holds no lock (point 3). Until then it is
     intact, and a `weakref`
  - why: Consistency with points 1 and 3: the delay exists only for values made on a thread that has a home count, and ends at a check point outside every held lock.

- EDIT rule=RC-4 point 3
  - old:   3. **Check points.** The compiler places them: at every loop's back edge, before and after every
     call that may block the thread (waiting for a lock, a `join`, the end of a `with scope` block,
     a channel's receive, a sleep) and when the thread ends; the one after a `join` or a scope's end
     makes a value a joined task ended on its own thread end before the parent's next line. A value
  - new:   3. **Check points.** The compiler places them: at every loop's back edge, before and after every
     call that may block the thread (waiting for a lock, a `join`, the end of a `with scope` block,
     a channel's receive, a sleep), where a call C made into Ember returns to C (`[FFI-22]`), and
     when the thread ends, before its `join` returns; none stands in the lines of a function that
     carries a contract (§X.2), whose promise the queued `drop`s could break, nor in the wrapper that
     returns such a function to C, so the queue waits there for the thread's next check point
     outside it. **A check point empties the queue only while the thread holds no lock**: each
     thread keeps a count of the locks it holds (a `with` block of a `Lock`, `[THR-18]`; a `Mutex`'s
     or `RwLock`'s guard from its taking to its last use, `[THR-3]`), one plain add at the taking
     and one plain subtract at the release, and the release that brings the count to zero is a
     check point. So a queued `drop` never waits for a lock its own thread holds and never changes a
     `guarded by` collection this thread walks under its lock; the lines the program writes are
     refused or accepted when compiling as `[THR-19]` says, and the count decides only when the
     queue is emptied (§X.4). The check point after a `join` or a scope's end makes a value a joined
     task ended on its own thread end before the parent's next line, or, where the parent holds a
     lock there, where it lets its last lock go. A value
  - why: J7-01's cause: check points were placed by position only, so a queued drop could run inside the lock it takes. Whether this thread holds a lock at a check point depends on the callers of the function, across packages, so it is run-time data (`[BLD-16]`): a per-thread count, maintained at the lock lines every package compiles alike. The return-to-C point gives a main thread driven by a C event loop a bound; a contract function is the one place whose promise hidden code could break, and the compiler sees the contract on the function it compiles. Walks and views need no keep-out: besides its own object a queued drop can change only guarded places (the lock count), atomics, and module variables, and a module variable changed by the drop of a value another thread may end is already `E7012` (`[THR-20]`) or `E7007` (`[THR-11]`, through `[DRP-8]`).

- EDIT rule=THR-11 (the sentence "A task touches no one-thread count")
  - old:   new value after the `with scope` block"). A value a task or a job takes with `owns` is handed to
  - new:   new value after the `with scope` block"). A `thread_shared` value's home count is a one-thread
  count too, so a task's thread has none: a `thread_shared` value a task or a job makes is counted
  in its away count from its making line (`[RC-4]` point 1), nothing is ever queued to a task's
  thread, which ends at its join, and its `drop` runs where its last owner ends (`[THR-20]`); so the
  end of a `with scope` block waits for no `drop` but those the tasks' own lines run. A value a task
  or a job takes with `owns` is handed to
  - why: The rule that states what a task may count; verdict 166 (a scoped task neither makes nor ends a `shares` owner) extends in the same words to the home count. `[PAR-2]` and `[JOB-2]` route jobs and chunks here, so one sentence covers all three.

- EDIT rule=THR-20 (the headline "A `drop` runs where the last owner ends")
  - old:   `drop` runs where the last owner ends.** The last owner of a `thread_shared` value may end on any
  thread that holds a count of it: an owner, or a view opened from a `weakref` link, which holds one
  for its life. So a function that may run on another thread and holds such a count counts as
  possibly running that value's `drop`, and the `drop` of everything it owns: what they change is
  changed on that thread, and a module variable of the main thread they reach is `E7012`.
  - new:   `drop` runs where the last owner ends**, or, for a value queued back to the main thread or the
  `thread.spawn` thread that made it (`[RC-4]` point 2), at that thread's next check point at which
  it holds no lock (`[RC-4]` point 3). The last owner of a `thread_shared` value may end on any
  thread that holds a count of it: an owner, or a view opened from a `weakref` link, which holds one
  for its life; and a `thread.spawn` thread holds the home count of every `thread_shared` value made
  on it from the making line until the value is handed over or the thread ends. So a function that
  may run on another thread and holds such a count, the function of a `thread.spawn` thread for
  every `thread_shared` type it may make (its summary, `[EFF-24]`) included, counts as possibly
  running that value's `drop`, and the `drop` of everything it owns: what they change is changed on
  that thread, a module variable of the main thread they reach is `E7012`, and a `join` of the
  thread inside a lock's `with` block, when such a `drop` takes a lock of that kind, is `E7018`
  (`[THR-19]`: "`opener.join()` (line 40) waits for the thread started on line 38, which makes
  `Conn` objects (line 32) and ends those another thread lets go of, and a `Conn`'s `drop` takes a
  `Pool`'s lock (line 14) while this thread holds `pool`'s lock, so each would wait for the other
  for ever; join after the `with pool.lock:` block: move line 40 below the block").
  - why: The headline was untrue for every queued value; the rule that says which thread runs a drop must name both places. Counting the values a spawn thread makes keeps verdict 43's promise for `join`: the thread's end empties its queue before the join returns, so a join under a lock those drops take would hang, and is refused when compiling with the one-line fix the existing join clause gives.

- EDIT rule=THR-19 (the join clause)
  - old:   another `Log`'s lock is refused too, and the way out moves one line); a join of a thread, or the
  end of a `with scope` block, whose task changes what the held lock guards; a direct call that may
  - new:   another `Log`'s lock is refused too, and the way out moves one line); a join of a thread, or the
  end of a `with scope` block, whose task changes what the held lock guards (for a `thread.spawn`
  thread the `drop`s of the `thread_shared` values made on it included, `[THR-20]`; a scoped task's
  thread has no such `drop`s, `[RC-4]` point 1); a direct call that may
  - why: The cross-reference must claim what `[THR-20]` now says, and must say plainly why the scope's end under a lock compiles: no drop is owed to a task's thread.

- EDIT rule=EFF-24 (the summary's list)
  - old:   what they view; what it reaches by other names while a `ref mut` parameter is used; and which locks it
  may take, or needs held (`[THR-18]`). No word on a function's first line says any of it: `changes`,
  - new:   what they view; what it reaches by other names while a `ref mut` parameter is used; which locks it
  may take, or needs held (`[THR-18]`); and which `thread_shared` types it may make, since a value
  made on a `thread.spawn` thread may come back to that thread to be ended (`[RC-4]` point 2,
  `[THR-20]`). No word on a function's first line says any of it: `changes`,
  - why: `[THR-20]`'s count of made values needs a fact the summary records; it is per function, composed through calls and carried in interface files like "what it may end", so a package compiled alone has it (`[BLD-16]`).

- EDIT rule=WK-3 (last sentence)
  - old:   count keeps in memory. A `thread_shared` value whose last owner ends on a thread that did not make
  it, and that was not handed over, is ended at its making thread's next check point (`[RC-4]` point
  5), and its `drop` runs there.
  - new:   count keeps in memory. A `thread_shared` value made on the main thread or a `thread.spawn` thread
  whose last owner ends on a thread that did not make it, and that was not handed over, is ended at
  its making thread's next check point at which it holds no lock (`[RC-4]` points 3 and 5), and its
  `drop` runs there; a value made on any other thread is ended where its last owner ends (`[RC-4]`
  point 1).
  - why: Consistency: the cross-reference claims what `[RC-4]` says.

- EDIT rule=OBJ-1 (the chunk header)
  - old:   another thread's allocator, and a block freed goes back to that thread's own pool, so the record
  never changes. A `thread_shared` value made `using` an instance (`[ALC-1]`) lies in memory the
  - new:   another thread's allocator, and a block freed goes back to that thread's own pool, so the record
  never changes. A chunk of a thread that has no home count (`[RC-4]` point 1: a scoped task's
  thread, a job worker, a thread C owns) names no thread, and a block from it is marked handed over
  when it is made, so no thread ever finds itself its maker and every count operation on it takes
  the away path. A `thread_shared` value made `using` an instance (`[ALC-1]`) lies in memory the
  - why: Where the making thread is recorded: the no-thread record is how "counted away from the making line" is implemented with no test at the making line and no change to the count operations.

- EDIT rule=COST-3 row "A `thread_shared` value's two counts" (first part)
  - old: a value made `using` an instance counts every owner atomically, as if handed over at its making line (`[RC-4]` point 4);
  - new: a value made `using` an instance counts every owner atomically, as if handed over at its making line (`[RC-4]` point 4), and so does a value made on a thread that has no home count, a scoped task's thread, a job worker (a `@parallel` chunk's included) or a thread C owns (`[RC-4]` point 1), with nothing at its making line beyond the header's initial marks;
  - why: `[COST-5]`: the atomic counts for those values are a cost this ruling adds.

- EDIT rule=COST-3 row "A `thread_shared` value's two counts" (second part)
  - old: a test of the thread's queue at each check point (`[RC-4]`), and the queue's work for a value whose last owner ends on another thread without a hand-over.
  - new: a test of the thread's queue at each check point, one thread-local load and one branch, and at each callback's return to C (`[RC-4]` point 3); the queue's work, out of line, for a value whose last owner ends on another thread without a hand-over, run only while the thread holds no lock, the release of its last lock being a check point (one plain add per lock taken and one plain subtract, with one test of the queue, per lock released).
  - why: `[COST-5]`: the lock count and the return-to-C test are costs this ruling adds; the check point's hot path is unchanged.

- EDIT rule=COST-3 row "A lock"
  - old: | A lock | guaranteed present | where the program takes one (`with x.lock:`, `m.lock()`): the lock C takes; nothing per use of a `guarded by` field (`[THR-18]`) |
  - new: | A lock | guaranteed present | where the program takes one (`with x.lock:`, `m.lock()`): the lock C takes, one plain add of the thread's count of held locks at the taking and one plain subtract at the release, with one test of the thread's queue when the count reaches zero (`[RC-4]` point 3); nothing per use of a `guarded by` field (`[THR-18]`) |
  - why: The cost sits on the lock lines, so the lock row names it.

- EDIT rule=COST-3 row "An Ember function called by C through a function pointer"
  - old: | An Ember function called by C through a function pointer (`[FFI-21]`) | guaranteed present | the thread-local attach check of its C entry (`[FFI-22]`), nothing else |
  - new: | An Ember function called by C through a function pointer (`[FFI-21]`) | guaranteed present | the thread-local attach check of its C entry (`[FFI-22]`) and one test of the thread's queue at its return (`[RC-4]` point 3; none for a function carrying a contract), nothing else |
  - why: The return-to-C check point is a cost on that path.

- EDIT rule=COST-3 row "An export's or a trampoline's entry"
  - old: | An export's or a trampoline's entry (`[FFI-22]`, `[FFI-33]`) | guaranteed present | the thread-local attach test, and under `threads=creator` or `main` one compare of the calling thread |
  - new: | An export's or a trampoline's entry (`[FFI-22]`, `[FFI-33]`) | guaranteed present | the thread-local attach test, under `threads=creator` or `main` one compare of the calling thread, and one test of the thread's queue at the return (`[RC-4]` point 3; none for a function carrying a contract) |
  - why: The same cost on the export path.

**refused_program**: ```ember
import std.thread                              # line 1
from std.sync import Atomic, Lock

static DONE: Atomic[int] = Atomic(0)

class Pool:                                    # a connection pool: a free list behind a lock
    lock: Lock = Lock()
    free: Array[int] guarded by lock = []

class Conn:                                    # one connection (line 10)
    id: int
    pool: thread_shared Pool
    fn drop():                                 # gives its id back to the pool
        with this.pool.lock:                   # line 14
            this.pool.free.push(owns copy of this.id)

class Batch:                                   # a plain object: connections to close
    conns: Array[thread_shared Conn] = []

class Inbox:                                   # batches waiting for the closer thread (line 20)
    lock: Lock = Lock()
    items: Array[Batch] guarded by lock = []

fn close_all(inbox: thread_shared Inbox):      # the closer thread lets the batches go
    while DONE.load() == 0:
        with inbox.lock:
            inbox.items.clear()

fn open_some(pool: thread_shared Pool, inbox: thread_shared Inbox):   # the opener thread makes connections
    for i in 0..1000:                          # line 30
        with inbox.lock:
            inbox.items.push(owns Batch(conns=[thread_shared Conn(id=i, pool=thread_shared pool)]))   # line 32

fn main():
    pool = thread_shared Pool()
    inbox = thread_shared Inbox()
    closer = thread.spawn(owns fn() thread_shared inbox: close_all(inbox))
    opener = thread.spawn(owns fn() thread_shared pool, thread_shared inbox: open_some(pool, inbox))   # line 38
    with pool.lock:
        opener.join()                          # line 40
        println(pool.free.len())
    DONE.store(1)
    closer.join()
```

Refused when compiling, E7018 at line 40 (`[THR-19]` with `[THR-20]`): "`opener.join()` (line 40) waits for the thread started on line 38, which makes `Conn` objects (line 32) and ends those another thread lets go of, and a `Conn`'s `drop` takes a `Pool`'s lock (line 14) while this thread holds `pool`'s lock, so each would wait for the other for ever; join after the `with pool.lock:` block: move line 40 below the block".

Why it would hang without the refusal: the `Conn`s are made on the opener thread (home count), travel inside plain `Batch`es into the inbox (not handed over, `[RC-4]` point 5), the closer lets them go (away count below zero, queued to the opener, point 2), and the opener's end empties its queue before its `join` returns (point 3): each `drop` takes `pool.lock`, which `main` holds while it waits in `opener.join()`. With line 40 moved below the block the program compiles and prints 1000 after the closer's join (not shown: the second read is after `closer.join()`).

**accepted_program**: ```ember
import std.thread                              # the arbiter's E3-1 program, unchanged
from std.sync import Atomic, Lock

static DONE: Atomic[int] = Atomic(0)

class Pool:                                    # a connection pool: a free list behind a lock
    lock: Lock = Lock()
    free: Array[int] guarded by lock = []

class Conn:                                    # one connection, made on a scoped task's thread
    id: int
    pool: thread_shared Pool
    fn drop():                                 # gives its id back to the pool
        with this.pool.lock:
            this.pool.free.push(owns copy of this.id)

class Batch:                                   # a plain object: connections to close
    conns: Array[thread_shared Conn] = []

class Inbox:                                   # batches waiting for the closer thread
    lock: Lock = Lock()
    items: Array[Batch] guarded by lock = []

fn close_all(inbox: thread_shared Inbox):      # the closer thread: lets the batches go
    while DONE.load() == 0:
        with inbox.lock:
            inbox.items.clear()                # ends each Batch and its Conn
    with inbox.lock:
        inbox.items.clear()

fn open_one(pool: thread_shared Pool, inbox: thread_shared Inbox):
    with inbox.lock:
        inbox.items.push(owns Batch(conns=[thread_shared Conn(id=1, pool=thread_shared pool)]))

fn main():
    pool = thread_shared Pool()
    inbox = thread_shared Inbox()
    closer = thread.spawn(owns fn() thread_shared inbox: close_all(inbox))
    with pool.lock:
        with s = thread.scope():
            s.spawn(owns fn(): open_one(pool, inbox))
        println(pool.free.len())
    DONE.store(1)
    closer.join()
    with pool.lock:
        println(pool.free.len())
```

Compiles: the task's function pushes to `inbox.items` under `inbox.lock` and ends no `Conn`, so it changes nothing `pool.lock` guards, and a scoped task's thread owes no `drop` (`[THR-11]`, `[RC-4]` point 1), so the scope's end inside `with pool.lock:` is not `E7018`. Runs: the `Conn` is made on the task's thread, so it is marked handed over at its making and counted in its away count; the closer's `clear` takes that count to zero and ends the `Conn` there, whose `drop` waits for `pool.lock` until `main`'s block ends (`main` waits for nothing the closer does inside that block). Prints `0` (the free list is empty while `main` holds the lock), then `1` (the closer's last `clear`, and so the `drop`, finish before `closer.join()` returns). The same every run. J7-02's program compiles too and prints `texture 1 freed`, `texture 2 freed` during `bundles.clear()` on `main`, then the frames. J7-01's program compiles; the `Conn`'s `drop` runs where `main`'s `with pool.lock:` block ends, later frames print `1`, and `h.join()` prints `1`.

**runtime_cost**: Row "A lock": one plain add of the thread's held-lock count at each lock taking and one plain subtract at each release, with one thread-local load and branch (the queue test) when the count reaches zero. Row "A `thread_shared` value's two counts": the check point stays one thread-local load and one branch; the queue's work (out of line) now runs only while the thread holds no lock; one queue test at each callback's return to C; atomic count operations from the making line for every `thread_shared` value made on a scoped task's thread, a job worker, a `@parallel` chunk or a thread C owns (the measured 25% of a thread's own values is lost for a task's own shared values; the pipeline shape, made on one thread and ended on the next, measured 27% slower under two counts on one thread, gains). Rows "An Ember function called by C through a function pointer" and "An export's or a trampoline's entry": one queue test at the return. Nothing new at any written line, at any check point's hot path, at a scope's end or at a thread's end.

**implementer_steps**: 1. Runtime, thread registration (`[HR-3a]`): each registered thread records whether it has a home count: the main thread and `thread.spawn` threads yes; scoped task threads (however the runtime runs them, own threads or pool workers), job workers and host threads attached by `ember_rt_thread_attach` no. Only a thread with a home count has a queue.
2. Runtime, allocation (`[OBJ-1]`): a chunk taken by a thread without a home count gets the no-thread id in its header; a `thread_shared` block from such a chunk is initialised with the handed-over mark set in its away word, exactly as a block made `using` an instance is, and its first owner goes to the away count. No change to the inlined count operations of `[RT-10]`: the id compare fails for every thread, the mark test takes the away path.
3. Runtime, locks: a thread-local `locks_held`; `ember_rt_lock_take` adds one, `ember_rt_lock_release` subtracts one and, when zero and the queue head is not null, calls the out-of-line drain (`ember_rt_checkpoint`). The drain itself returns at once when `locks_held` is not zero, so every existing check point keeps its one load and branch. The compiler may omit a check point it can prove lies under a lock in the same function; not observable.
4. Compiler, check points (`[RC-4]` point 3): as today at back edges, before and after blocking calls (a `join`, a scope's end, a receive, a sleep, a lock wait) and at the thread's end before the join is signalled; add one after the body in every export and trampoline wrapper (beside `[FFI-22]`'s attach test); emit none inside a function carrying a contract of §X.2, nor in its C wrapper. The scope's end waits for its tasks only (`[THR-5]`); tasks have no queue to drain.
5. Compiler, summaries (`[EFF-24]`): record the set of `thread_shared` types a function may make (its own lines plus callees, function values by `[EFF-2]`), written to the interface file like "what it may end". `[THR-20]`: the function given to `thread.spawn` counts the `drop` of each made type as possibly run on that thread (for `E7012` and for the lock kinds it may take); `[THR-19]`'s join clause reads it, so `h.join()` under a lock of a kind one of those `drop`s takes is `E7018`, with the message above.
6. Tests (break-tested): the accepted program prints `0` then `1` under a stress loop; the refused program is `E7018` at line 40 and compiles with the join moved; J7-02's program prints the two `freed` lines during the clear; J7-01's program prints `1` at the join; a program that holds a lock while a worker lets a main-made value go, with a loop back edge inside the lock, runs the drop exactly when the block ends (a counter of drop runs read under the lock inside the block is 0, after it 1); a contract function with a back edge drains nothing inside it (an `@noblock` spin loop with a queued value, the drop runs after the function returns).
7. Separate compilation and modules (`[BLD-16]`, `[ABI-1]`): every package emits the same lock-count operations and check-point tests; no package needs to know which `thread_shared` types exist or which threads make them; the made-type set travels in interface files as other summary facts do. Hot reload (Annex B): the queue, the held-lock count and the home-count flag are runtime thread state, not image state (`[HR-4a]`); a commit runs only with every thread's `[HR-42]` depth at zero, so never inside a drain. FFI: a host thread has no queue, so a value made in a callback on a C-owned thread is ended wherever its last owner ends; the main thread driven by a C event loop drains at each callback's return.

**departs_from_verdict**: Verdict 101 in its letter, for the owner (ODR): "The thread that made the object counts its owners in one number, A" no longer holds for a scoped task's thread, a job worker or a thread C owns, whose values count in B from the making line. The substance is kept: two numbers for the main thread and every `thread.spawn` thread, the queue, check points placed by the compiler, deletion always before the thread ends (a spawn thread's end empties its queue before its join returns; nothing is ever queued to a thread joined as a task), the ended-thread path word for word; counting away from the making line is fix 2's step at the making line, which N84 (decisions.md section 13) already took for a value made `using` an instance and reported as touching verdict 101. Verdict 43 kept in full: no written line is checked while running; the held-lock count decides only when hidden code runs and refuses nothing. Verdicts 56, 57 (ends still count as calls at the ender; `[DRP-8]`, `[EFF-24]` unchanged), 104, 114 untouched. Verdict 166 (a scoped task neither makes nor ends a `shares` owner, a count with no atomics) is followed into the home count.

**trade_offs**: Speed: a `thread_shared` value made inside a scoped task, a job or a `@parallel` chunk pays atomic count operations (verdict 101's measurement: a thread's own values 25% faster under two counts, lost for a task's own shared values), while the common shape, a value a task makes for its parent to end, gains (the pipeline row: two counts 27% slower on one thread); the lock count costs two plain instructions per lock, beside C's lock; the check point's hot path and every written line are unchanged. The alternative kept for the lead, design B: a scoped task's thread keeps a home count and at its end hands its queue, closed, to the parent, which drains it at its next check point outside every lock; it keeps the two-count speed for task-internal sharing at the price of a splice at each task's end, a parent that reads another thread's final home counts, a rule that a value already queued is ended only by the queue's taker (else a subtract that finds the queue closed and the parent would both end it), and nothing for job workers and C threads, which would still need the atomic rule or a wake of idle workers; it composes with everything else here. Readability: no word to write, no new refusal but the join under a lock whose fix moves one line (verdict 43's own shape); a newcomer holds one sentence: made by main or a spawned thread, ended at home when free of locks; made anywhere else, ended where let go. Safety: all atomic for the no-home-count values; for the others the drain runs only on the making thread (the sole writer of the home count), and the held-lock count keeps a queued drop out of the lock it takes; walks and views need no keep-out because every other place a queued drop could change is guarded, atomic, or a module variable whose change by a value another thread may end is already `E7012`/`E7007`. Consistency: `[THR-11]`'s "a task touches no one-thread count" becomes literally true of the home count; `[THR-20]`'s headline becomes true for queued values; `[THR-19]`'s and verdict 43's promise holds for `join` and for a scope's end. Given up: a thread that holds a lock, or sits in a contract function, for its whole life defers the drops queued to it until it releases, returns or ends (as any thread needing that lock would wait); a C-driven main thread drains at callback returns, not inside C.

**remaining_doubt**: (1) Whether the owner accepts narrowing verdict 101's A to threads the program owns; the lead may instead take design B for `thread.scope` tasks and the atomic rule for workers and C threads only. (2) `[RC-4]` point 4's "travelled inside another object": `[THR-2]` already walks a stored object's contents at a store under a lock and stops at a `thread_shared` value; letting that walk also hand over each `thread_shared` owner it meets when the storing thread is the maker would shrink the queue to objects stored by non-makers, within fix 2; not changed here, since verdict 101 accepted the delay for that shape. (3) The old text quoted in the edits is copied with CURRENT-SPEC.md's line wrapping; the lead's asserting script may need the unwrapped form. (4) The `E7018` message's wording for the join should be checked against the DIA rules. (5) A `Once` initialiser (`[STA-3]`) running at a check point whose queued drop reads the same static would recurse into the once-flag, exactly as a written call there would; the current text does not treat it either. (6) The atomic cost for task-internal shared values is inferred from verdict 101's rows, not measured for this shape; measure on the laptop's performance cores before the hardening is cut.

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
