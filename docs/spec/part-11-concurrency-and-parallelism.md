# Part XI — Concurrency and Parallelism

## XI.1 Thread safety markers

* `Send`: a value may be moved to another thread. Auto-derived (`[TYP-*]`). Not `Send`: `*T`, `*mut T`, `ref`/`ref mut`/views (in v1 — scoped threads relax this, §3), non-`Sync` class handles, `Shared[T]` where `T: !Sync`.
* `Sync`: a value may be *shared* (borrowed) by multiple threads simultaneously. Auto-derived when all fields are `Sync`. `ref mut` is never `Sync`. Interior mutability primitives (`Cell[T]`, `RefCell[T]`; Part IX §7) are `!Sync`; `Atomic[T]`, `Mutex[T]`, `RwLock[T]` are `Sync` (when `T: Send`).
* `[THR-1]` A class is `Sync` iff **every** field's type is itself an interior-synchronised type (`Atomic`, `Mutex`, `RwLock`, channel end) or a deeply immutable value type. **There is no `let` exemption** (`OQ-18`): `let` restrains rebinding, not mutation through the field (`[CLS-9a]`), so exempting `let` fields would make a class `Sync` whose contents a `mut self` method can still mutate — a data race in Safe code. A class that needs a shared mutable field uses `Mutex[T]`/`RwLock[T]`; one that needs a frozen field uses a type with no `mut self` API. — because handles alias, a plain mutable field shared across threads would be a data race. `@sync class` asserts `Sync` and is `E7001` if the rule is violated; `@thread_local class` forces `!Sync` even if the fields would allow it (to get non-atomic counts).
* `[THR-2]` Class handles are `Send` iff the class is `Sync` (a sent handle can be copied on both sides).
* `[THR-7]` **What `Sync` does and does not mean.** `Sync` on a class means two things and no more: its handles may cross threads (`[THR-2]`), and its reference count is atomic so the *lifetime* is safe under concurrent handle traffic. It does **not** mean the object's fields may be mutated concurrently. Field-level race freedom comes from the ordinary rules — a `mut self` method needs exclusive access, which across threads means a `Mutex`/`RwLock` or an `Atomic` field — and `[PHIL-10]`'s data-race guarantee holds because those rules apply to a `Sync` class exactly as to any other, not because `Sync` waives them. The derivation is written so this stays true: a `let` field is not exempt from the `Sync` requirement, precisely because `let` restrains rebinding and not mutation through the field (`[CLS-9a]`), and exempting it would produce a `Sync` class whose contents a `mut self` method could still race on.

## XI.2 Primitives (`std.sync`, `std.thread`)

```ember
t = thread.spawn(owned fn() -> R: pass)      # requires captures Send; returns JoinHandle[R]
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
* `[THR-6]` A type whose `drop` is load-bearing for a borrow guarantee is marked `@must_drop` (Part III §7). A `@must_drop` value MUST NOT be (a) passed to `mem.forget`, (b) stored in a `class` field, `Shared[T]`, `Box[T]`, or any container or closure capture that can participate in a reference cycle, or (c) returned from the function that created it. Violations are `E3015 value whose drop is required may not be leaked`, whose help names the closure-scoped form. In v1 the standard library applies `@must_drop` to exactly `Scope` and `JobScope`; adding it to any other type requires an entry in `docs/DECISIONS.md`.

## XI.3 Structured (scoped) concurrency

```ember
with scope = thread.scope():                       # all spawned tasks joined at block end
    scope.spawn(fn(): process(left))               # non-owned closures allowed: borrows outlive the scope's join
    scope.spawn(fn(): process(right))
```

`[THR-5]` `thread.scope(f)` takes the scope body as a closure and returns only after every task spawned inside it has been joined, on both normal and panicking exit. The `Scope` value is a parameter of `f`; it is a view type whose region is `f`'s body, so it cannot be moved out, stored, returned, captured by an `owned fn`, or passed to `mem.forget`. Inside the scope, closures may borrow locals of the enclosing function; the borrow checker treats the scope's region as covering the spawned closures. The surface form `with scope = thread.scope():` is sugar for `thread.scope(fn(scope): …)`; the binding it introduces is not a movable place (`E3014 scope binding may not be moved`). `[JOB-2]` is the same rule for `jobs.scope`.

## XI.4 `@parallel for`

```ember
@parallel(chunk=256)
for i in 0..count:
    positions[i] += velocities[i] * dt
```

* `[PAR-1]` The loop body is compiled as a closure invoked over index chunks by the job system (Part XI §5).
* `[PAR-2]` **Iterations MUST be independent.** The body MUST be free of `break`, `continue`-to-outer and `return`; captures are checked for `Send`. For each place `P` written in the body, the compiler collects the set of index expressions through which `P` is accessed anywhere in the body, **reads included**. The loop is accepted only if: (a) every such expression has the form `i + k` with `k` a compile-time constant (`k = 0` permitted); (b) **all of them have the same `k`**; and (c) `P` is a `MutSpan` or an `SoA` column captured by the loop, not a place reached through a class handle, a `RefCell`, a `Cell`, or a raw pointer. Otherwise `E7010 parallel loop writes to a shared place`, with the suggestion set (`Atomic`, `chunks_mut`, `@parallel(reduce=…)`).
* `[PAR-3]` Reductions: `@parallel(reduce=[sum: +, best: max])` declares variables combined with an associative operator after the loop.
* `[PAR-4]` `@parallel` loops carry the `Sync` effect (job submission) and are therefore illegal in `@nosync` functions; they are `@noalloc`-clean when the job system is initialised with preallocated queues (the default).
* `[PAR-2a]` Violation of clause (b) specifically is `E7011 parallel loop has a loop-carried dependency on <place>: written at index `i + <k1>`, also accessed at index `i + <k2>``, whose help MUST name a concrete alternative: split into two `@parallel` loops separated by a join, express it as `@parallel(reduce=…)` (`[PAR-3]`), or run the loop serially.
* `[PAR-2b]` The analysis MUST run on MIR **after inlining** of the body's calls. A call in the body that the compiler cannot see through and that receives a captured `MutSpan` or `SoA` column defeats the proof for every place reachable through that argument (`E7010`).

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
