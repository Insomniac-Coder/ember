---

# Part XI — Concurrency and Parallelism

Ember has OS threads, scoped tasks that may use the caller's own variables, a job system, and parallel
loops. Safe Ember has no data races (`[PHIL-10]`): two threads touch one memory location at the same
time only when both accesses read it, or when the location is an `Atomic`, or is protected by a lock
(a `Mutex`, an `RwLock`, or a `Lock` that fields are `guarded by`) or a channel. The rules below are how
that holds without a garbage collector: every one is checked when compiling, and nothing runs for them
but the locks, atomics and counts the program's own words ask for.

```ember
import std.thread
from std.sync import Atomic, Lock

class Stats:
    words: Atomic[int] = Atomic(0)
    lock: Lock = Lock()
    tags: Array[String] guarded by lock = []

fn scan(stats: Stats, text: StrView):
    for w in text.split_whitespace():
        stats.words.fetch_add(1)
        if w.starts_with("#"):
            with stats.lock:
                stats.tags.push(String(w))

fn main():
    stats = Stats()
    texts = [String("#ember is fast"), String("no tags here"), String("#python #c")]
    with scope = thread.scope():
        for t in texts:
            scope.spawn(fn() ref t: scan(stats, t))
    println(stats.words.load(), "words")
```

Each task uses `main`'s real `stats` and looks at its round's text with `ref t`; `scan` changes
`stats.words` through its `Atomic` and `stats.tags` under the lock that guards it, so the tasks never
change one place unordered.

## XI.1 What may reach another thread

* `[THR-8]` *(new in 0.9.9)* *(changed in 0.9.10)* *(changed in 0.9.15)* A type is **`Send`** when a value
  of it may be handed over to another thread: nothing reachable from it (its fields, its elements, what
  its `owns` parts hold, a function value's state) is a one-thread owner (`shares T`, `shares mut T`), a
  `weakref` link to a value that is not `thread_shared`, or bound to the thread that made it (a lock's
  guard, a `Scope` or `JobScope`, a raw pointer, a `threads(creator)` handle). Structs, enums, tuples,
  fixed arrays, collections and class objects are `Send` when every component is; an `owns T` when `T`
  is; `ref T` and `Span[T]` when `T` is `ThreadShared`, `ref mut T` and `MutSpan[T]` when `T` is
  `Send`; a `thread_shared T`, and a `weakref` link to one, when `T` is `Send` and `ThreadShared`;
  `dyn I` when `I` requires it (`interface I: needs Send:`). Of the function types, `thread_shared fn()` is
  `Send` (`[THR-1]`). A `Send` view still goes only where it cannot outlive what it views: to a scoped
  task or a job of a scope, never to an unscoped thread (`[THR-2]`). `Send` is inferred from the
  words and written only in bounds; the author of a type that holds a raw
  pointer declares it with `unsafe extend T implements Send:` and a `# SAFETY:` note (`E3100` without
  `unsafe`: "`Counter` holds a raw pointer, so the compiler cannot tell that another thread may have
  it; say that you vouch for it: write `unsafe extend Counter implements Send:` with a `# SAFETY:` note
  above it"). A thread-bound value sent to another thread is `E7008`.
* `[THR-9]` *(new in 0.9.9)* *(changed in 0.9.15)* A type is **`ThreadShared`** when several threads may
  hold and read one value of it at once. It is inferred like `Send` and written only in bounds
  (`[T: needs ThreadShared]` above `fn keep(xs: Array[T]) -> int:`) and in messages; `Sync` in a bound is
  `E1010` ("`Sync` is the effect `@nosync` forbids, not a property of a type; for 'threads may share
  it' write `[T: needs ThreadShared]` above `fn keep(x: T)`"). Structural types are `ThreadShared` when every
  component is. `ref mut T`, `Cell`, `UnsafeCell`, raw pointers, `shares T`, `shares mut T`, and a
  `weakref` link to a value that is not `thread_shared` are not. `Atomic[T]`, `Once`, `Lock`,
  `Mutex[T]` for a `Send` `T` (one thread at a time holds its `ref mut T`), `RwLock[T]` and
  `thread_shared T` for a `T` that is `Send` and `ThreadShared` (several threads hold a `ref T` at
  once: `RwLock[Cell[int]]` is not `ThreadShared`), and a channel's `Sender` are.
* `[THR-1]` *(changed in 0.9.9)* *(changed in 0.9.15)* **A value several threads own is made
  `thread_shared`.** `thread_shared x` makes one more owner of `x`, which may be on any thread; it is
  written before a new value (`st = thread_shared Stats()`) or on a value already `thread_shared`
  (`fn() thread_shared st:`), and on a value made otherwise it is `E7017` ("`names` is
  `share_and_keep`'s own object, which it may change; to share it between threads, create it shared:
  `names = thread_shared Names()`"). A `thread_shared` value lives until its last owner ends. Its
  owners are counted in two numbers, its making thread's and the other threads' (atomic), and its weak
  count is atomic (`[RC-4]`); each is compared with its limit before it grows (Part VIII). Three rules keep it safe:
  * **no owner changes a `thread_shared` value directly** (`E7014`: "`names` is shared between threads
    (line 5), so no owner changes it directly, and line 6 pushes to its list; fill the list where the
    object is made: write `names = thread_shared Names(items=[copy of s])` instead of lines 5 and 6");
    its state changes only through an `Atomic`, a `Mutex`, an `RwLock`, a channel end, or a field
    `guarded by` a lock, under that lock (`[THR-18]`);
  * everything reachable from it is `Send` (`[THR-8]`): a one-thread owner, a `weakref` link to a value
    that is not `thread_shared`, or a plain `fn()` value reachable from it is `E7015` at the line that
    makes it `thread_shared` ("`Bus.handlers` holds plain `fn()` values, and a `thread_shared Bus` may
    run them on any thread; write `handlers: Array[thread_shared fn()] = []` in `Bus`");
  * **`thread_shared fn()`** is a function any thread may run. It holds only `thread_shared` owners,
    values handed over to it and copies, never a one-thread owner or a `weakref` link to one (`E7015`:
    "`on_event` keeps functions any thread may run (`thread_shared fn()`), and this one holds `count`, a
    `shares mut` owner, which stays on one thread; share the counter between threads: write
    `count = thread_shared Counter()` in place of line 12, give `Counter` the field
    `n: Atomic[int] = Atomic(0)` (with `from std.sync import Atomic` at the top), and pass
    `fn() thread_shared count: count.n.fetch_add(1)`"). A list, field or parameter whose functions may
    run on any thread says so in its type: `handlers: Array[thread_shared fn()] guarded by lock = []`.

  As a parameter's type, `log: thread_shared Log` reads the caller's value with no count, and
  `owns log: thread_shared Log` takes one owner. A class needs no mark to be shared between threads:
  `@sync` is not Ember (`E0104`: "`@sync` is gone: a class needs no mark to be shared between threads,
  and `Stats`'s fields are `Atomic`, which threads may change as they are; remove `@sync` (line 1)").
* `[THR-2]` *(changed in 0.9.9)* *(changed in 0.9.15)* **A one-thread value never reaches another
  thread.** A `shares` or `shares mut` owner counts its owners with plain adds, and every object that is
  not `thread_shared` counts its `weakref` links with plain adds. So no such owner, and no `weakref`
  link to such a value, reaches another thread by any route: captured or held by a function any thread
  may run (`thread_shared fn()`, the function of `thread.spawn` or `jobs.submit`, a callback C may run on
  another thread), handed to an unscoped thread or a job, or reachable from a `thread_shared` value
  (`[THR-1]`); and neither does a view of a place this thread owns, which an unscoped thread may outlive
  (`E7015`, whose message gives the words to write: the value made `thread_shared` from the start, or a
  copy). **Making, ending or opening a `weakref` link touches its target's header** (its weak count, and
  the test that it still lives): in a scoped task, a job, or any function that may run on another thread
  than its target's owner, it is allowed only for a `thread_shared` target, whose counts are kept for
  several threads and whose opened view holds a count (`[RC-4]`, `[RC-5]` point 2; `E7015`: "`players[0]` counts its links without atomics, and
  the task on line 9 makes one too; make the links after the scope, or make the player
  `thread_shared`"). **An object handed whole to another thread leaves its `weakref` links behind**
  (verdict 114; Rust's `Rc::try_unwrap`). At the hand-over the program reads the object's weak count,
  one compare. With no link, the object goes as it is. With a link, its contents move to a new block
  for the other thread, and the old block is marked ended (its `drop` does not run; the new block's
  runs once, on the other thread) and stays on this thread until its last link ends, so every link here
  finds it ended (`[WK-12]`): after `w = weakref names` and
  `h = thread.spawn(fn() owns names: names.items.len())`, `if w exists as p:` is skipped and
  `h.join()` gives 0. The same is done for every object the handed object owns whose class a `weakref`
  is made to anywhere in the program (the compiler knows those classes; one compare each), so after
  `cam.target = weakref level.enemies[0]`, handing `level` over leaves `cam.target` finding enemy 0
  ended. An object whose address C keeps (`[FFI-35a]`) never moves: a link to it at the hand-over
  stops the program ("`names` goes to another thread while `w`, a `weakref` link on this thread, points
  at it, and C keeps its address (`register(names)`, line 4), so it cannot move; end the link first:
  `w = None`"). A view opened from a link is not used after a hand-over that may move its object
  (`[RC-5]` point 1). The other thread never touches this thread's counts: the old block stays here
  with the links, and the new one starts with none. So an object that stays on one thread needs no
  atomic count.
* `[THR-7]` *(changed in 0.9.15)* `thread_shared` means exactly: the value may have owners on several
  threads, its counts are kept for several threads (`[RC-4]`), and no owner changes it directly (`[THR-1]`). It does not make a method
  atomic: two operations on `Atomic` fields, or a read and a store of one
  (`if n > this.longest.load(): this.longest.store(n)`), are two steps another thread may come between;
  a read and a change that must stay together are made under one lock (`[THR-18]`). A diagnostic MUST
  NOT describe `thread_shared` or `ThreadShared` as "thread-safe".
* `[THR-13]` *(new in 0.9.9)* *(changed in 0.9.15)* **The data-race guarantee.** In Safe Ember every
  place two threads can reach is one of these, and each is ordered or only read:
  * a place a scoped task reads: neither the parent nor another task changes it until the join
    (`[THR-11]`); the loop's own variable reaches a task only as the task's own copy, or as a view of the
    element it is on, which the walked collection keeps unchanged until the join (`[THR-17]`);
  * a place tasks change by a reduction: each task has its own accumulator, and the join orders them
    (`[THR-21]`);
  * an `Atomic`, `Mutex`, `RwLock`, channel end or `guarded by` field: ordered by the atomic or by the
    lock, which the thread that touches it holds (`[THR-3]`, `[THR-18]`);
  * a value handed over to the other thread, which the giver no longer has, and from which no one-thread
    value is reachable (`[THR-2]`, `[THR-8]`);
  * a `thread_shared` value: no owner changes it directly, and its counts are kept for several threads (`[THR-1]`, `[RC-4]`);
  * a module variable: a `static` never changes (`[STA-1]`), and a `static mut` is reached only by the
    main thread and its scoped tasks, or is `guarded by` a lock (`[THR-20]`);
  * a `sharedref mut` target: a scoped task changes it directly only where neither another task nor the
    parent may touch it before the join, or by a reduction (`E7007`); an unscoped thread never holds one
    (`E7015`).

## XI.2 Threads and synchronisation (`std.thread`, `std.sync`)

```ember
import std.thread
from std.sync import Sender, channel

fn produce(tx: Sender[int]):
    for i in 0..10:
        tx.send(i * i).expect("receiver is gone")

fn main():
    tx, rx = channel[int](capacity=64)
    producer = thread.spawn(fn() owns tx: produce(tx))
    total = 0
    for v in rx:
        total += v
    producer.join()
    println(total)
```

* `[THR-10]` *(new in 0.9.9)* *(changed in 0.9.10)* *(changed in 0.9.15)* `thread.spawn` runs a function on
  a new thread, which may outlive the function that started it. It takes a function any thread may run,
  `[R]` above `pub fn spawn(owns f: thread_shared fn() -> R) -> JoinHandle[R]:`, so a function that holds
  a one-thread value is `E7015` (`[THR-1]`, `[THR-2]`), and `R` MUST be `Send` (`E7015`, `E7008`). What
  the function holds from its maker is written in its capture list (`[THR-17]`). `h.join() -> R` waits
  and returns the result. Dropping a `JoinHandle` detaches the thread. **Starting a thread counts as
  making everything its function changes, now or later** (`[EFF-26]`): a `thread.spawn` inside a walk of
  what the thread changes is `E3020` ("start it after the loop"), except where the thread makes that
  change under a lock the walk holds, when the new thread waits for the lock until the loop ends; joining
  such a thread inside that lock is `E7018` (`[THR-19]`).
* `[THR-17]` *(new in 0.9.15)* **What an unscoped thread holds, and the loop's own variable.** An unscoped
  thread (`thread.spawn`), a job submitted outside a scope (`[JOB-2]`) and a callback C keeps (Part XVI)
  say in their capture list everything of their maker's that they use: `owns x` (handed over; the
  maker's `x` ends), `thread_shared x` (one more owner), or `copy of x` (a copy of their own); a name
  used with no word is `E7004` ("this thread may outlive `share_plain`, so it cannot use
  `share_plain`'s `names`; say what the thread holds: `fn() owns names:` (handed over), or create it
  shared, `names = thread_shared Names()`, and write `fn() thread_shared names:` (one more owner)").
  `shares` and `shares mut` captures, and a `weakref` capture of a value that is not `thread_shared`,
  are allowed only where the function stays on this thread, which a callback whose contract says
  `threads(creator)` or `threads(main)` does, checked by the trampoline in every profile (Part XVI). A
  module variable (`RESULTS`) needs no word. **The loop's own variable**, which the loop gives another
  element every round, and a local of one round, which ends with it, are given to a task or a thread
  with a word. `copy of t` gives it its own copy (for a `String`, one allocation and copy per task):
  `thread.spawn(fn() copy of r: worker(r))`. For a scoped task, `ref t` lets it look at the element `t`
  is on in that round, with no copy, since the walked collection cannot change until every task has
  ended at the end of the `with scope` block (`scope.spawn(fn() ref t: total +=
  t.split_whitespace().count())`). `ref t` where the function may still run after the element changes
  or ends is `E3026`: a `thread.spawn` thread may outlive the loop ("the thread may still run after the
  loop has moved on and `texts` is gone; give it its own copy: `thread.spawn(fn() copy of t: ...)`").
  The loop's name used with no word is `E3026` for a scoped task, whose message gives both words ("the
  task may still run in a later round, when `t` names another text; give it its own copy,
  `scope.spawn(fn() copy of t: ...)`, or let it look at this round's text, `scope.spawn(fn() ref t:
  ...)`"), and `E7004` for a `thread.spawn` thread, whose message gives `copy of t`.
* `[THR-16]` *(new in 0.9.9)* When `main` returns, or `process.exit` is called, the process ends: other
  threads stop without running destructors, no static is freed and no drop runs. The `debug` leak
  report (`[WK-15]`) runs only if no other thread is still running, and otherwise says how many were.
* `[THR-3]` *(changed in 0.9.9)* *(changed in 0.9.15)* `Mutex[T]` is a value type. `m.lock()` blocks and
  returns a `MutexGuard[T]`, a view of `m` that reads through to `ref mut T` (`[TYP-14]`); the mutex is
  released when the guard ends, which `with g = m.lock():` places at the end of the block.
  `m.try_lock() -> Option[MutexGuard[T]]` never blocks. A guard belongs to the thread that took the
  lock: sending it to another thread is `E7008` ("`g` is the guard of `m`'s lock, which belongs to the
  thread that took it; take the lock inside the task: write `fn add_one(m: Mutex[Array[int]]):` with the
  body `with g = m.lock():` and, inside it, `g.push(1)`, above line 11, and `scope.spawn(fn():
  add_one(m))` in place of line 16"). A view made through a guard (`g[0]`, `ref g[0]`, a slice of `g`)
  does not leave the `with` block (`E3063`: "this points into the locked list, which can change once the
  lock ends; return a copy, or do the work inside the `with` block"), and the same holds for `RwLock`'s
  guards and for a `guarded by` field inside its lock (`[BRW-20]`). There is no poisoning: a panic aborts
  the process (`[PAN-1]`). `RwLock[T]` has `read()` (a shared guard, `ref T`) and `write()` (an exclusive
  guard). `Once` runs an initialiser exactly once (`[STA-3]` is built on it). A `Mutex`, `RwLock`,
  `Atomic` or channel end is changed through any name, a plain one included, by its own operations, and
  an argument given to one takes no word at the call (`ref mut` before it is `E3028`: "`top` takes the
  lock of `scores` itself; a `Mutex` takes no word at the call: `top(scores)`").
* `[THR-18]` *(new in 0.9.15)* **`Lock` and `guarded by`.** A `Lock` is a lock with no value inside;
  `with x.lock:` holds it for the block. `guarded by lock` after a field's type (or a `static mut`'s,
  `[THR-20]`) names the `Lock` that must be held for every read and change of that field outside `init`:
  the field is used only inside `with x.lock:` on the same object `x` (the same name, not given another
  object in the block; `this.lock` in a method), on the thread that holds it (`E7013`: "`words` is
  guarded by `lock`; change it inside `with stats.lock:`"; "`b.words` is guarded by `b.lock`, and this
  block holds `a.lock`; write `with b.lock:`"). Several fields may name one lock, and a class may have
  several locks; `init` sets guarded fields without the lock.
  * A `guarded by` field is a `Mutex` written as ordinary fields, and follows a `Mutex`'s rules: it is
    changed under its lock through any name that reaches its object, a plain one included, with no word
    at the call (`fn add_line(log: Log):` whose body is `with log.lock:` and, inside it,
    `log.lines.push(String("hi"))`). An object whose every field that may change is `guarded by` its
    lock, or is an `Atomic`, `Mutex` or `RwLock`, may reach one call through any number of names, a plain
    argument and a function value that holds it included (`dump(log, fn() thread_shared log:
    log.push("seen"))`): the lock keeps the ways in apart (`[BRW-11]`).
  * A function whose body uses a guarded field without taking its lock is worked out to need that lock
    held (its summary, `[EFF-24]`), and is called only where the lock is held (`E7013`).
  * **A direct call that takes a lock this thread already holds enters the callee past it**, without
    taking it again, only where the compiler proves it is the very lock held: the same name, or `this`
    of the object whose lock is held (`with log.lock: log.push("a")`, where `push` takes `this.lock`).
    Where the callee's lock may be the held one and may not be, the call is `E7018` ("`other.push` takes
    `other`'s lock, and `other` may be `held`, whose lock this thread holds; write `if other is not
    held:` above line 13 and indent line 13 under it").

  ```ember
  from std.sync import Lock

  class Log:
      lock: Lock = Lock()
      lines: Array[String] guarded by lock = []

      fn push(s: StrView):
          with this.lock:
              this.lines.push(String(s))

  fn main():
      log = Log()
      with log.lock:
          log.push("a")
          println(log.lines.len())
  ```

* `[THR-19]` *(new in 0.9.15)* **A wait for a lock this thread holds is refused when compiling.** Inside a
  lock's `with` block, these are `E7018`, with no loop needed and nothing checked while the program runs:
  a call through a function value, or through a `dyn` or `virtual` method, that may run a function taking
  a lock of the kind this thread holds (a `Log`'s lock, while it holds `log`'s: the compiler cannot tell
  which object's lock a stored function takes, so one that takes another `Log`'s lock is refused too, and
  the way out moves one line); a join of a
  thread, or the end of a `with scope` block, whose task changes what the held lock guards; and a direct
  call that may take the held lock and is not proved to enter past it (`[THR-18]`). The messages: "`note()`
  may run the function stored on line 12, which takes a `Log`'s lock (`log.push`, under `lock`), and this
  thread holds `log`'s lock, so taking it again would wait for ever; call `note()` after the
  `with log.lock:` block: move line 16 below the block"; "`t.join()` (line 16) waits for the thread
  started on line 15, whose function changes `RESULTS` under `RESULTS_LOCK`, and this thread holds that
  lock, so each would wait for the other for ever; join after the `with RESULTS_LOCK:` block: move line
  16 below the block". Inside a walk under that lock, such a call is `E7018`, not `E3020`, whose fix
  (after the loop, still inside the lock) would still wait for ever ("this loop walks `log.lines` (line
  9) under `log`'s lock, and `note()` (line 11) may run the function stored on line 4, which takes a
  `Log`'s lock, so it would wait for the lock this thread holds; call `note()` after the `with log.lock:`
  block: move line 11 below the block").
* `[THR-20]` *(new in 0.9.15)* **Module variables and threads.** A `static mut` (`[STA-1]`) is reached only
  by the main thread and the tasks of scopes it opens, unless it is `guarded by` a `static` `Lock` and
  used inside `with` that lock: a function that may run on another thread (reachable from
  `thread.spawn`, from a job submitted outside a scope, from a callback C may run on a thread other than
  the main one, or from an `@unsafe(export("name", threads=any))` or `@unsafe(export("name", threads=creator))`
  function) and reads or
  changes it is `E7012` ("`tick` changes `FRAME` and `LOG`, module variables of the main thread, and this
  thread runs beside it; guard both with one lock: write `from std.sync import Lock` at the top,
  `static TICK_LOCK: Lock = Lock()`, then `static mut FRAME: int guarded by TICK_LOCK = 0` instead of
  line 1 and `static mut LOG: Array[String] guarded by TICK_LOCK = []` instead of line 2, and put
  `tick`'s body inside `with TICK_LOCK:`"). A `static` of a type that changes only through itself
  (`Lock`, `Atomic`, `Mutex`, `RwLock`, a channel end) is usable from any thread. **A `drop` runs where
  the last owner ends.** The last owner of a `thread_shared` value may end on any thread that holds a
  count of it: an owner, or a view opened from a `weakref` link, which holds one for its life. So a
  function that may run on another thread and holds such a count counts as possibly running that
  value's `drop`, and the `drop` of everything it owns: what they change is changed on that thread, and
  a module variable of the main thread they reach is `E7012`.

  ```ember
  import std.thread
  from std.sync import Lock

  static RESULTS_LOCK: Lock = Lock()
  static mut RESULTS: Array[int] guarded by RESULTS_LOCK = []

  fn worker(x: int):
      r = x * x
      with RESULTS_LOCK:
          RESULTS.push(r)

  fn main():
      worker(1)
      handles: Array[thread.JoinHandle[void]] = []
      with RESULTS_LOCK:
          for r in RESULTS:
              handles.push(thread.spawn(fn() copy of r: worker(r)))
      for h in owns handles:
          h.join()
      with RESULTS_LOCK:
          println(RESULTS.len())
  ```

* `[THR-14]` *(new in 0.9.9)* `Atomic[T]` exists for the integer types, `bool` and raw pointers.
  Every operation takes `order: MemoryOrder = MemoryOrder.SeqCst` (`Relaxed`, `Acquire`, `Release`,
  `AcqRel`, `SeqCst`); an order an operation does not support (`Acquire` on a store) is `E7006`.
  `fetch_add`/`fetch_sub` **panic when the result overflows**, like `+` (`[TYP-8]`); the overflowed
  value has been stored, and another thread may observe it before the process aborts.
  `checked_fetch_add` never stores an overflowed value (it is a compare-exchange loop and returns
  `Option`), and `wrapping_fetch_add` wraps without a check.
* `[THR-15]` *(new in 0.9.9)* *(changed in 0.9.15)* `channel[T](capacity=n) -> (Sender[T], Receiver[T])` is
  a bounded many-producer, one-consumer queue; `channel_unbounded[T]()` has no bound and allocates as it
  grows. `Sender` is `Send`, `ThreadShared` and `Clone` (`copy of tx` is another sender of the same
  channel); `Receiver` is `Send`. `tx.send(v) -> Result[void, SendError[T]]` blocks while the queue is
  full and returns the value when the receiver is gone; `rx.recv() -> Option[T]` blocks and returns
  `None` once every sender has ended and the queue is empty; `try_send`/`try_recv` never block. A
  `Receiver` is iterable; the loop ends at `None`. **When the `Receiver` ends, so do the values still
  queued**: it takes them out under the channel's lock and ends them after letting the lock go, so a
  `Sender` queued in its own channel (`tx.send(copy of tx)`) does not keep the channel alive.
* `[THR-4]` In the `debug` profile the runtime records the order in which each thread takes locks and
  prints `warning: lock order inversion` with both acquisition sites when two threads take two locks
  in opposite orders. It never changes a program's behaviour.
* `[THR-6]` *(changed in 0.9.9)* *(changed in 0.9.15)* A type whose `drop` a safety guarantee depends on is
  `@must_drop`. A `@must_drop` value MUST NOT be stored in a class field, a `shares` or `thread_shared`
  value, an `owns` block, a collection or a function value's state (a capture word), held across a
  `yield` (`[CORO-13]`), or returned from the function that created it (`E3015`); every value's `drop`
  runs at its end. The standard library applies it to `Scope` and `JobScope` only. It is structural: a
  struct, enum, tuple or generic type that owns a `@must_drop` value is `@must_drop` too, whatever its
  own `drop` does, so wrapping one hides nothing; a view of one is not, since it lives no longer than the
  value it views. A `Scope` or `JobScope` cannot be put in a wrapper at all (`[THR-5]`) (ODR-054).

## XI.3 Scoped tasks

A scoped task may use the caller's own variables, because the scope does not end until every task in it
has finished.

```ember
import std.thread

fn sum_halves(xs: Span[int]) -> int:
    left, right = xs.split_at(xs.len() // 2)
    with scope = thread.scope():
        a = scope.spawn(fn() => left.iter().sum())
        b = scope.spawn(fn() => right.iter().sum())
        return a.join() + b.join()

fn count_words(texts: Array[String]) -> int:
    total = 0
    with scope = thread.scope():
        for t in texts:
            scope.spawn(fn() ref t: total += t.split_whitespace().count())
    return total
```

* `[THR-5]` *(changed in 0.9.9)* *(changed in 0.9.15)* `thread.scope()` returns a `Scope`, which MUST be
  bound by a `with` (`E3014` otherwise). The `with` block is ordinary code of the enclosing function:
  `return`, `break`, `continue` and `?` inside it mean what they mean anywhere else. On **every** exit
  from the block — falling off its end, a jump, or `?` — the scope first joins every task spawned in it
  that has not been joined, and then the exit completes; the value of a `return` is computed before the
  join. The `Scope` binding is a view whose life is the block and is `@must_drop` (`[THR-6]`): it cannot
  be handed over, stored, returned or held in a function value's state. A `with scope` block that ends
  inside a lock's `with` block, when a task's function changes what that lock guards, is `E7018`
  (`[THR-19]`).
* `[THR-11]` *(new in 0.9.9)* *(changed in 0.9.15)* `scope.spawn` takes a plain function, `[R]` above
  `fn spawn(owns f: fn() -> R) -> ScopedJoinHandle[R]:`. A task needs no capture words but for the
  loop's own variable and a local of one round (`[THR-17]`): it uses the caller's real variables, which
  outlive it. What each task changes and reads is worked out from its body and every function it may run
  (`[EFF-24]`), and is checked against the other tasks and the parent until the task's join: `h.join()`
  on its handle, or the end of the `with scope:` block for a task not joined before it (verdict 104).
  After `h.join()` the parent may change or read what that task used, in the same block
  (`counts[0] += 1` after `h.join()`), checked only against the tasks not yet joined: two of them
  changing one place, or one reading a place another changes, is `E7007`, other than by a reduction
  (`[THR-21]`), under a lock, or through an `Atomic`. A task that reads a place before changing it needs
  the lock, which keeps the read and the change together ("two tasks change `stats.total` (line 19), and
  each reads it first (`if stats.total < 10:`, line 18), so each may see a total the other is changing;
  guard it, which keeps the read and the add together: write `from std.sync import Lock` at the top, in
  `Stats` `lock: Lock = Lock()`, `total: int guarded by lock = 0` and `fn add_below(limit: int, n:
  int):` whose body is `with this.lock:`, inside it `if this.total < limit:` and inside that
  `this.total += n`, and call `stats.add_below(10, x)` in place of lines 18 and 19"). A task started
  inside a walk of what it changes is `E3020` ("`scope.spawn(...)` (line 6) starts a task that pushes to
  `names` (`fn(): names.push(String("x"))`), which this loop walks (line 5); start it after the loop:
  move line 6 below the loop"), not `E7007`, whose fix would leave the walk refused. A task makes, ends
  or opens a `weakref` link only to a `thread_shared` target, and makes no `shares` owner (`[THR-2]`).
  `R` MUST be `Send`. `h.join() -> R` waits early.
* `[THR-21]` *(new in 0.9.15)* **Reductions.** A task's `total += e` (and `-=`, `*=`, `|=`, `&=`, `^=`,
  `max`, `min`, `push`, `extend`, `add`, `m[k] op= e`) on a local, or on a field of an object the tasks
  reach (`scope.spawn(fn() copy of x: stats.total += x)`), a field reached through two `sharedref mut`
  links to one object included, is a reduction: each task adds into a private accumulator, and the
  accumulators are combined into the place at the join, in spawn order. Nothing reads the place before
  the join (`E7007`: "tasks add their parts to `total`, which is known only once the `with` block (lines
  11 to 14) ends; read it after the block: remove line 14 and write `println(total)` above line 15").
  Each task's accumulator has the place's type, and its operations are checked as the place's are, and
  so are the combining operations at the join (verdict 105): a whole-number total is the one adding one
  by one gives whenever no task's own total overflows (`a.target.hp -= 1` and `b.target.hp -= 2` from
  two tasks, `a` and `b` linking one player whose `hp` is 10, leave 7 every time), and a task whose own
  total overflows stops the program there (`[TYP-8]`), even where adding one by one would not
  (`[[-100], [100, 100]]` summed into an `i8` by two tasks stops in the second, at 100 + 100, where
  adding one by one gives 100); a decimal total may differ from it in the last digit (`for part in [[0.1], [0.2, 0.3]]:` with
  `scope.spawn(fn() copy of part: for x in part: total += x)` gives `0.6`, where adding one by one gives
  `0.6000000000000001`), and is the same on every run, so it is not `Nondet` (`[DET-2]`). A plain
  assignment from a task (`a.target.hp = 5`) and a task that reads the current value
  (`if stats.total < 10: stats.total += x`) need a lock (`E7007`). A `thread.spawn` thread has no
  reductions: it changes what other threads reach under a lock, or, where it only adds, through an
  `Atomic`.
* `[THR-12]` *(new in 0.9.9)* *(changed in 0.9.15)* Tasks that must run in sequence and share data use two
  scopes in sequence: the join at the end of the first is the ordering, and the views of the first end
  there.

## XI.4 Parallel loops

```ember
class Player:
    hp: int = 10

fn integrate(ref mut pos: MutSpan[float], vel: Span[float], dt: float):
    @parallel(chunk=4096)
    for i in 0..pos.len():
        pos[i] += vel[i] * dt

fn hurt_all(ref mut players: Array[Player]):
    @parallel
    for ref mut p in players:
        p.hp -= 1
```

* `[PAR-1]` *(changed in 0.9.15)* A `@parallel` loop splits a range (`for i in 0..n:`) or a list walked by
  element (`for ref mut p in players:`) into chunks, and runs its body over each chunk on the job system
  (§XI.5). Over anything else it is `E7016` ("`@parallel` splits a range or a list into chunks, and
  `while i < idx.len():` (line 4) in `place` is neither; instead of `i = 0`, the `while` line and
  `i += 1`, write `for i in 0..idx.len():`").
* `[PAR-2]` *(changed in 0.9.15)* **Iterations MUST be independent.** The body MUST NOT contain `break`,
  `return`, `yield`, or a `continue` naming an outer loop, and what it uses from outside follows the
  rules of a scoped task (`[THR-11]`, `[THR-2]`). In a loop over a range, for each place `P` written in
  the body the compiler collects every index expression through which `P` is accessed in the body,
  **reads included**. The loop is accepted only if (a) each has the form `i + k` for a constant `k`;
  (b) all of them share **one** `k`; and (c) `P` is the elements of a `MutSpan` or an `SoA` column the
  loop uses, not a place reached through a link (`shares`, `shares mut`, `sharedref mut`,
  `thread_shared`), a `Cell` or a raw pointer. A loop walked by element hands each chunk its own elements
  through the loop's `ref mut` name, and writes nothing else but declared reductions (`[PAR-3]`); its
  list holds each element once, so a list whose objects may be in it twice (`Array[shares mut Player]`)
  is refused. Otherwise `E7010`, whose message gives the lines that make the loop compile ("the parallel
  loop writes `xs[idx[i]]`, and two rounds may have the same `idx[i]`, so two threads could write one
  element; run the loop on one thread: remove `@parallel` (line 7)"; "`players` holds `shares mut
  Player` objects, and one object may be in the list twice, so two chunks could change it at once; give
  each player one owner: write `players: Array[Player] = []` in place of line 3, and
  `players.push(Player(hp=10))` in place of `players.push(shares mut p)` on line 5"); where an `Atomic`,
  `chunks_mut` or `@parallel(reduce=…)` makes the loop compile, the message gives that line.
* `[PAR-2a]` A violation of clause (b) is `E7011`, `parallel loop has a loop-carried dependency on
  <place>`, naming both offsets and offering: two `@parallel` loops, a reduction, or a serial loop.
* `[PAR-2b]` The analysis reads through the body's calls as if inlined: each call to a function
  `[CG-C-3]` places in the inline header, recursively until a call is recursive — never according to
  what the C compiler chose to inline. A call it cannot see into that receives a captured `MutSpan`
  or column defeats the proof for everything reachable through that argument (`E7010`) (ODR-062).
* `[PAR-3]` *(changed in 0.9.9)* `@parallel(reduce=[total: +, best: max])` declares variables combined
  after the loop; inside the body each chunk has its own copy. Each chunk reduces its iterations left
  to right, and the chunk results are then combined in chunk order, so the result is the same on
  every machine (`[PAR-5]`); for floats it can differ from the serial left-to-right sum, which is
  why a float reduction must be declared. A whole-number reduction's chunk results have the variable's
  type and stop the program on overflow, as `[THR-21]`'s task totals do (verdict 105): the result is the
  serial one whenever no chunk's own total overflows, and since chunk boundaries depend only on the trip
  count and `chunk`, whether it stops does not depend on the machine.
* `[PAR-4]` *(changed in 0.9.9)* A `@parallel` loop has the effects `Sync` and `Block` (it waits for
  its chunks) and not `Alloc`: its chunk descriptors live in the calling frame and the job queues have
  a fixed capacity; when they are full the calling thread runs the chunk itself.
* `[PAR-5]` *(new in 0.9.9)* **Parallel results do not depend on the machine.** Chunk boundaries are a
  function of the trip count and `chunk` only (a default `chunk` is a function of the trip count
  only), and reductions combine per-chunk results in chunk order. A `@parallel` loop therefore
  computes the same bits whatever the worker count and scheduling, and is not `Nondet` (`[DET-2]`).

## XI.5 The job system (`std.jobs`)

The job system is a pool of worker threads with work-stealing queues. The runtime starts it before
`main` in any program that uses `@parallel` or `std.jobs`, with the worker count from the manifest
(`[jobs] workers = "auto"`, meaning logical cores minus one).

```ember
import std.jobs
from std.math import Vec3

fn integrate(ref mut pos: MutSpan[Vec3], vel: Span[Vec3], dt: f32):
    for i in 0..pos.len():
        pos[i] += vel[i] * dt

fn age(ref mut life: MutSpan[f32], dt: f32):
    for i in 0..life.len():
        life[i] -= dt

fn step(ref mut pos: MutSpan[Vec3], vel: Span[Vec3], ref mut life: MutSpan[f32], dt: f32):
    with s = jobs.scope():
        s.submit(fn(): integrate(ref mut pos, vel, dt))
        s.submit(fn(): age(ref mut life, dt))
```

* `[JOB-1]` A job's captured state is stored inline in its queue slot when it fits in 64 bytes and in
  one heap allocation otherwise (`Alloc`, reported by `ember inspect --alloc`).
* `[JOB-2]` *(changed in 0.9.9)* *(changed in 0.9.15)* `jobs.scope()` follows `[THR-5]` and `[THR-11]`:
  `s.submit` on a `JobScope` takes a plain function (`owns f: fn()`), whose jobs may use the caller's own
  variables, and the block's exit joins them. `jobs.submit` outside a scope takes a function any thread
  may run, `pub fn submit(owns f: thread_shared fn()) -> JobHandle:`, as `thread.spawn` does
  (`[THR-10]`, `[THR-17]`): a function that holds a one-thread value is `E7015`, and a name used with no
  capture word `E7004`. `jobs.wait(h)` waits for it.
* `[JOB-3]` *(changed in 0.9.9)* *(changed in 0.9.15)* `s.submit_after(deps, f)` starts `f` only after
  the jobs in `deps` have finished. It orders execution; it does not relax `[THR-11]`, which treats
  every job of a scope as running at once. (The 0.9.8 declared read/write access sets, verified only in
  debug builds, are removed: a safety property checked in one profile is not a guarantee, `[PHIL-13]`.)
* `[JOB-5]` `jobs.local_arena()` returns an arena view for the current job, reset when the job ends;
  allocations from it cannot escape the job.

## XI.6 Async

`async` and `await` are reserved words (§II.4). Ember's workloads are covered by threads, scoped tasks,
jobs and generators (§VI.5a); a later version may add structured async over the job system, in which
a view may not cross an `await` unless the future is scoped. Generators are designed so that this
remains possible: a generator frame is a sized value, and the same frame lowering serves an
`async fn`.
