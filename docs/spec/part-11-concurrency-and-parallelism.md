# Part XI — Concurrency and Parallelism

## XI.1 Thread safety markers

* `Send`: a value may be moved to another thread. Auto-derived (`[TYP-*]`). Not `Send`: `*T`, `*mut T`, `ref`/`ref mut`/views (in v1 — scoped threads relax this, §3), non-`Sync` class handles, `Shared[T]` where `T: !Sync`.
* `Sync`: a value may be *shared* (borrowed) by multiple threads simultaneously. Auto-derived when all fields are `Sync`. `ref mut` is never `Sync`. Interior mutability primitives (`Cell[T]`, `RefCell[T]`) are `!Sync`; `Atomic[T]`, `Mutex[T]`, `RwLock[T]` are `Sync` (when `T: Send`).
* `[THR-1]` A class is `Sync` iff every field is `Sync` **and** every non-`let` field's type is itself an interior-synchronised type (`Atomic`, `Mutex`, `RwLock`, channel end) — because handles alias, a plain mutable field shared across threads would be a data race. `@sync class` asserts `Sync` and is `E7001` if the rule is violated; `@thread_local class` forces `!Sync` even if the fields would allow it (to get non-atomic counts).
* `[THR-2]` Class handles are `Send` iff the class is `Sync` (a sent handle can be copied on both sides).

## XI.2 Primitives (`std.sync`, `std.thread`)

```ember
t = thread.spawn(owned fn() -> R: ...)      # requires captures Send; returns JoinHandle[R]
r = t.join()                                # -> Result[R, PanicPayload]

m = Mutex[Array[i32]](Array())              # Sync
with g = m.lock():                          # g: MutexGuard (view, ref mut Array[i32] via auto-deref)
    g.push(1)

a = Atomic[u32](0)
a.fetch_add(1, Ordering.Relaxed)            # orderings: Relaxed, Acquire, Release, AcqRel, SeqCst (default SeqCst)

tx, rx = channel[Message](capacity=1024)    # bounded MPSC; unbounded via channel_unbounded
tx.send(msg)?                               # Result; Err if receiver dropped
```

* `[THR-3]` `Mutex[T]` is a struct; locking returns a guard whose region borrows the mutex; the guard is `!Send`. Poisoning is not modelled (panic = abort by default); with unwinding (v2) the lock is released during unwind.
* `[THR-4]` Deadlock detection in debug: the runtime records lock ordering per thread and reports order inversions (`W-runtime: lock order inversion`).

## XI.3 Structured (scoped) concurrency

```ember
with scope = thread.scope():                       # all spawned tasks joined at block end
    scope.spawn(fn(): process(left))               # non-owned closures allowed: borrows outlive the scope's join
    scope.spawn(fn(): process(right))
```

`[THR-5]` Inside a scope, closures may borrow locals of the enclosing function because the scope's drop joins all tasks; the borrow checker treats the scope's region as covering the spawned closures. Views are thus `Send` *within a scope* when their pointee type is `Sync`. This is the primary API for fork-join engine work.

## XI.4 `@parallel for`

```ember
@parallel(chunk=256)
for i in 0..count:
    positions[i] += velocities[i] * dt
```

* `[PAR-1]` The loop body is compiled as a closure invoked over index chunks by the job system (Part XI §5). `[PAR-2]` The body MUST be free of `break`/`continue`-to-outer/`return`; iterations MUST be independent: the closure captures are checked for `Send` and, for mutable captures, the compiler requires that each mutable place written in the body be indexed by the loop variable through a `MutSpan`/`SoA` column with a **disjointness proof** (index expression is exactly `i` or `i + const`); otherwise `E7010 parallel loop writes to a shared place` with a suggestion (`Atomic`, `chunks_mut`, reduction).
* `[PAR-3]` Reductions: `@parallel(reduce=[sum: +, best: max])` declares variables combined with an associative operator after the loop.
* `[PAR-4]` `@parallel` loops carry the `Sync` effect (job submission) and are therefore illegal in `@nosync` functions; they are `@noalloc`-clean when the job system is initialised with preallocated queues (the default).

## XI.5 Job system (`std.jobs`)

```ember
jobs = JobSystem.init(worker_count = cpu.logical_cores() - 1)   # once, in main

h1 = jobs.submit(fn(): build_visibility(scene, mut visible))    # returns JobHandle
h2 = jobs.submit_after([h1], fn(): build_commands(visible, mut cmds))
jobs.wait(h2)
```

* `[JOB-1]` Work-stealing deques per worker; jobs are `owned fn() -> void` closures stored inline in a fixed-size slot (≤ 64 bytes captured, else boxed — reported by `ember inspect`).
* `[JOB-2]` Job closures may borrow outer state **only** through a `JobScope` (`with s = jobs.scope(): s.submit(...)`), which joins at block end — same rule as `[THR-5]`.
* `[JOB-3]` **Access-set scheduling.** A job may declare `jobs.submit_with_access(reads=[positions], writes=[velocities], f)`; the scheduler orders jobs whose declared write sets intersect any other's read/write sets, giving access-set-driven parallelism without whole-program analysis. `[JOB-4]` Access sets are checked in debug builds by recording actual `Span` base pointers touched (the runtime instruments `Span` creation inside jobs when `-Cjobs-verify` is on).

## XI.6 Job-local memory

`[JOB-5]` Each worker owns a `ThreadArena`; `jobs.local_arena()` returns a `ScopedArena` view for the current job that is reset when the job completes. Allocations from it cannot escape the job (their region is the job closure's), so temporary allocations are cheap and deterministic without threading an arena parameter through every function.

## XI.7 Async

Reserved for v2. `async`/`await` keywords are reserved; the design will follow "structured async over the job system", with the rule that borrows may not cross `await` points unless the future is scoped. Nothing in v1 precludes this.

---
