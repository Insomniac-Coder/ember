---

# Part XI — Concurrency and Parallelism

Ember has OS threads, scoped tasks that may borrow the caller's data, a job system, and parallel
loops. Safe Ember has no data races (`[PHIL-10]`): two threads touch one memory location at the same
time only when both accesses read it, or when the location is an `Atomic`, or is protected by a
`Mutex`, `RwLock` or channel. The rules below are how that holds without a garbage collector and
without a check on every access.

```ember
import std.thread
from std.sync import Atomic, Mutex

@sync
class Stats:
    words: Atomic[int] = Atomic(0)
    tags: Mutex[Array[String]] = Mutex([])

fn scan(stats: Stats, text: str):
    for w in text.split_whitespace():
        stats.words.fetch_add(1)
        if w.starts_with("#"):
            with tags = stats.tags.lock():
                tags.push(w.to_string())

fn main():
    stats = Stats()
    texts = ["#ember is fast", "no tags here", "#python #c"]
    with scope = thread.scope():
        for t in texts:
            scope.spawn(fn() => scan(stats, t))
    println(stats.words.load(), "words")
```

## XI.1 `Send` and `Sync`

* `[THR-8]` *(new in 0.9.9)* A type is **`Send`** when a value of it may be moved to another thread.
  Structs, enums, tuples, fixed arrays and collections are `Send` when every component is. Raw
  pointers are not `Send`. `ref T` and `Span[T]` are `Send` when `T` is `Sync`; `ref mut T` and
  `MutSpan[T]` when `T` is `Send`. A class handle, and `Weak` of one, is `Send` only if the class is
  `@sync` (`[THR-2]`). `Shared[T]` is never `Send` (`[HEAP-10]`); `SyncShared[T]` and its `Weak` are.
* `[THR-9]` *(new in 0.9.9)* A type is **`Sync`** when several threads may read one value of it at
  the same time. Structural types are `Sync` when every component is. `Cell`, `RefCell`, `UnsafeCell`,
  raw pointers, `Shared[T]`, and handles (and `Weak`s) of classes that are not `@sync` are not `Sync`.
  `Atomic[T]`, `Once`, `SyncShared[T]`, `Mutex[T]`/`RwLock[T]` for a `Send` `T`, and handles of `@sync`
  classes are `Sync`.
* `[THR-1]` *(changed in 0.9.9)* **A class is `Sync` only when it is declared `@sync class`.** Every
  other class is neither `Sync` nor `Send`, whatever its fields, and uses plain (non-atomic) counts.
  In a `@sync` class:
  * every field's type is `Sync` (`E7001`, naming the field and the component that is not);
  * **every field is immutable after `init`.** Assigning a field, or beginning any write access to one
    (a `mut self` method on it, `ref mut`, `iter_mut`, passing it to a `mut` parameter), outside `init`
    is `E7003`; inside `init`, a field assignment after `self` has been used as a whole is `E7003`;
  * a method may not be declared `mut self` (`E7003`);
  * a base class and every derived class are `@sync` too (`E7001`).

  Mutation of a `@sync` object's state therefore happens only through its fields' own
  synchronisation — `Atomic`, `Mutex`, `RwLock`, `Once`, channel ends, or handles to other `@sync`
  objects. Every access to a `@sync` object is a read, so it needs no exclusivity check and its fields
  have no access words (`[EXC-19]`). The error's help names the field to wrap:
  `wrap it in Atomic[int]` for an integer, `Mutex[…]` otherwise.
* `[THR-2]` *(changed in 0.9.9)* Handles of a `@sync` class are `Send` and `Sync`. A handle of any other
  class cannot reach another thread by any Safe route — moving, capturing or borrowing into a task
  is `E7004` — so its objects need no atomic counts and no cross-thread exclusivity checks. The
  diagnostic offers `@sync` when the class could satisfy `[THR-1]`, and names the disqualifying field
  when it could not.
* `[THR-7]` `@sync` means exactly two things: the class's handles may cross threads, and its
  reference counts are atomic (`[RC-4]`). It does not make methods atomic: two threads calling
  `stats.words.fetch_add(1)` are each atomic, but a read followed by a write through two fields is
  two operations. A diagnostic MUST NOT describe `@sync` or `Sync` as "thread-safe".
* `[THR-13]` *(new in 0.9.9)* **The data-race guarantee.** In Safe Ember a memory location is reachable
  from two threads only through (a) a `@sync` object, whose fields are immutable after construction
  (`[THR-1]`); (b) a `static`, which is immutable and `Sync` (`[STA-1]`); (c) a `SyncShared[T]`,
  which gives only shared access; (d) a borrow handed to a scoped task, which the borrow rules make either shared and of a
  `Sync` type, or exclusive (`[THR-11]`); (e) a value moved to the other thread, which the sender no
  longer has. In each case concurrent mutation goes through `Atomic`, `Mutex`, `RwLock` or a channel.

## XI.2 Threads and synchronisation (`std.thread`, `std.sync`)

```ember
import std.thread
from std.sync import Sender, channel

fn produce(tx: Sender[int]):
    for i in 0..10:
        tx.send(i * i).expect("receiver is gone")

fn main():
    tx, rx = channel[int](capacity=64)
    producer = thread.spawn(owned fn() => produce(tx))
    total = 0
    for v in rx:
        total += v
    producer.join()
    println(total)
```

* `[THR-10]` *(new in 0.9.9)* `thread.spawn(owned f: fn() -> R) -> JoinHandle[R]` runs `f` on a new
  thread. The closure's captured state and `R` MUST be `Send` (`E7004`, naming the capture) and MUST
  carry only the static region: a thread may outlive every local of its creator, so a captured view of
  a local is `E3063`, whose help is `thread.scope()` (`[THR-5]`). `h.join() -> R` waits and returns the
  result. Dropping a `JoinHandle` detaches the thread.
* `[THR-16]` *(new in 0.9.9)* When `main` returns, or `process.exit` is called, the process ends: other
  threads stop without running destructors, no static is freed and no drop runs. The `debug` leak
  report (`[WK-15]`) runs only if no other thread is still running, and otherwise says how many were.
* `[THR-3]` *(changed in 0.9.9)* `Mutex[T]` is a value type. `m.lock()` blocks and returns a
  `MutexGuard[T]`, a view borrowing `m` that reads through to `ref mut T` (`[TYP-14]`); the mutex is
  released when the guard is dropped, which `with g = m.lock():` places at the end of the block.
  `m.try_lock() -> Option[MutexGuard[T]]` never blocks. A guard is not `Send`. There is no poisoning:
  a panic aborts the process (`[PAN-1]`). `RwLock[T]` has `read()` (shared guard, `ref T`) and
  `write()` (exclusive guard). `Once` runs an initialiser exactly once (`[STA-3]` is built on it).
* `[THR-14]` *(new in 0.9.9)* `Atomic[T]` exists for the integer types, `bool` and raw pointers.
  Every operation takes `order: MemoryOrder = MemoryOrder.SeqCst` (`Relaxed`, `Acquire`, `Release`,
  `AcqRel`, `SeqCst`); an order an operation does not support (`Acquire` on a store) is `E7006`.
  `fetch_add`/`fetch_sub` **panic when the result overflows**, like `+` (`[TYP-8]`); the overflowed
  value has been stored, and another thread may observe it before the process aborts.
  `checked_fetch_add` never stores an overflowed value (it is a compare-exchange loop and returns
  `Option`), and `wrapping_fetch_add` wraps without a check.
* `[THR-15]` *(new in 0.9.9)* `channel[T](capacity=n) -> (Sender[T], Receiver[T])` is a bounded
  many-producer, one-consumer queue; `channel_unbounded[T]()` has no bound and allocates as it grows.
  `Sender` is `Send`, `Sync` and `Clone`; `Receiver` is `Send`. `tx.send(v) -> Result[void, SendError[T]]`
  blocks while the queue is full and returns the value when the receiver is gone; `rx.recv() ->
  Option[T]` blocks and returns `None` once every sender has been dropped and the queue is empty;
  `try_send`/`try_recv` never block. A `Receiver` is iterable; the loop ends at `None`.
* `[THR-4]` In the `debug` profile the runtime records the order in which each thread takes locks and
  prints `warning: lock order inversion` with both acquisition sites when two threads take two locks
  in opposite orders. It never changes a program's behaviour.
* `[THR-6]` *(changed in 0.9.9)* A type whose `drop` a safety guarantee depends on is `@must_drop`. A
  `@must_drop` value MUST NOT be passed to `mem.forget`, stored in a class field, `Shared`,
  `SyncShared`, `Box`, a collection or an `owned fn` capture, held across a `yield` (`[CORO-13]`), or returned from the function that created it (`E3015`). The standard library
  applies it to `Scope` and `JobScope` only.

## XI.3 Scoped tasks

A scoped task may borrow the caller's locals, because the scope does not end until every task in it
has finished.

```ember
import std.thread

fn sum_halves(xs: Span[int]) -> int:
    left, right = xs.split_at(xs.len() // 2)
    with scope = thread.scope():
        a = scope.spawn(fn() => left.iter().sum())
        b = scope.spawn(fn() => right.iter().sum())
        return a.join() + b.join()
```

* `[THR-5]` *(changed in 0.9.9)* `thread.scope()` returns a `Scope`, which MUST be bound by a `with`
  (`E3014` otherwise). The `with` block is ordinary code of the enclosing function: `return`,
  `break`, `continue` and `?` inside it mean what they mean anywhere else. On **every** exit from the
  block — falling off its end, a jump, or `?` — the scope first joins every task spawned in it that
  has not been joined, and then the exit completes; the value of a `return` is computed before the
  join. The `Scope` binding is a view whose region is the block and is `@must_drop` (`[THR-6]`): it
  cannot be moved, stored, returned, captured by an `owned fn` or forgotten.
* `[THR-11]` *(new in 0.9.9)* `scope.spawn(f: fn() -> R) -> ScopedJoinHandle[R]` accepts a closure
  that borrows any place whose storage outlives the `with` block. Each borrow a spawned closure makes
  is live until the end of the block, so the ordinary borrow rules (`[BRW-2]`) reject two tasks that
  write one place, or one that writes a place another reads (`E3021`/`E3022`, naming both spawns). A
  shared borrow requires the borrowed type to be `Sync`, a mutable borrow requires it to be `Send`,
  and captured values and `R` MUST be `Send` (`E7004`/`E7005`). `h.join() -> R` waits early.
* `[THR-12]` *(new in 0.9.9)* Tasks that must run in sequence and share data use two scopes in
  sequence: the join at the end of the first is the ordering, and the borrows of the first end there.

## XI.4 Parallel loops

```ember
fn integrate(pos: MutSpan[float], vel: Span[float], dt: float):
    @parallel(chunk=4096)
    for i in 0..pos.len():
        pos[i] += vel[i] * dt
```

* `[PAR-1]` A `@parallel` loop's body runs as a closure over index chunks on the job system (§XI.5).
* `[PAR-2]` **Iterations MUST be independent.** The body MUST NOT contain `break`, `return`, `yield`,
  or a `continue` naming an outer loop, and its captures are `Send`. For each place `P` written in the
  body, the compiler collects every index expression through which `P` is accessed in the body,
  **reads included**. The loop is accepted only if (a) each has the form `i + k` for a constant `k`;
  (b) all of them share **one** `k`; and (c) `P` is a `MutSpan` or an `SoA` column captured by the
  loop, not a place reached through a class handle, `Cell`, `RefCell` or raw pointer. Otherwise
  `E7010`, with the alternatives `Atomic`, `chunks_mut` and `@parallel(reduce=…)`.
* `[PAR-2a]` A violation of clause (b) is `E7011`, `parallel loop has a loop-carried dependency on
  <place>`, naming both offsets and offering: two `@parallel` loops, a reduction, or a serial loop.
* `[PAR-2b]` The analysis runs after the body's calls are inlined. A call it cannot see into that
  receives a captured `MutSpan` or column defeats the proof for everything reachable through that
  argument (`E7010`).
* `[PAR-3]` *(changed in 0.9.9)* `@parallel(reduce=[total: +, best: max])` declares variables combined
  after the loop; inside the body each chunk has its own copy. Each chunk reduces its iterations left
  to right, and the chunk results are then combined in chunk order, so the result is the same on
  every machine (`[PAR-5]`); for floats it can differ from the serial left-to-right sum, which is
  why a float reduction must be declared.
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

fn step(pos: MutSpan[Vec3], vel: Span[Vec3], life: MutSpan[float], dt: float):
    with s = jobs.scope():
        s.submit(fn() => integrate(pos, vel, dt))
        s.submit(fn() => age(life, dt))
```

* `[JOB-1]` A job's captured state is stored inline in its queue slot when it fits in 64 bytes and in
  one heap allocation otherwise (`Alloc`, reported by `ember inspect --alloc`).
* `[JOB-2]` *(changed in 0.9.9)* `jobs.scope()` follows `[THR-5]` and `[THR-11]`: jobs submitted to a `JobScope` may borrow
  the caller's data, and the block's exit joins them. `jobs.submit(owned f: fn())` outside a scope
  takes an owned, `Send` closure carrying only the static region (as `[THR-10]`) and returns a
  `JobHandle`; `jobs.wait(h)` waits for it.
* `[JOB-3]` *(changed in 0.9.9)* `s.submit_after(deps, f)` starts `f` only after the jobs in `deps` have
  finished. It orders execution; it does not relax the borrow rules, which treat every job of a scope
  as running at once. (The 0.9.8 declared read/write access sets, verified only in debug builds, are
  removed: a safety property checked in one profile is not a guarantee, `[PHIL-13]`.)
* `[JOB-5]` `jobs.local_arena()` returns an arena view for the current job, reset when the job ends;
  allocations from it cannot escape the job.

## XI.6 Async

`async` and `await` are reserved words (§II.4). Ember's workloads are covered by threads, scoped tasks,
jobs and generators (§VI.5a); a later version may add structured async over the job system, in which
a borrow may not cross an `await` unless the future is scoped. Generators are designed so that this
remains possible: a generator frame is a sized value, and the same frame lowering serves an
`async fn`.
