# Pass 1 — Memory safety of 0.9.9_Hardened_1

**Question:** where can a program with no `unsafe` block reach freed, uninitialised or concurrently
mutated memory, or undefined behaviour in the generated C, under the rules of 0.9.9_Hardened_1?

**Method.** Each guarantee `[PHIL-10]` lists (no invalid ownership operation, no use-after-free, no
double release, no data race, no out-of-bounds access, no invalid value) was traced through every rule
that can create an alias, end a lifetime, cross a thread or cross the FFI boundary: Parts VII–IX, XI,
XII, XVI, XVIII and Annexes B–C. Each candidate hole was checked against the exact rule text before
being recorded.

**Result.** 8 holes of severity S1 (Safe code reaches freed, uninitialised or raced memory), 7 of S2
(a hole through an under-specified rule, or undefined behaviour in the generated code), 6 of S3
(clarifications that close a misreading). Each has a proposed fix; `CHANGES-H2.md` adopts them.

| Id | Sev | Where | Hole |
|---|---|---|---|
| MS-01 | S1 | `[THR-10]`, `[JOB-2]` | an unscoped thread or job may capture a view of the caller's locals |
| MS-02 | S1 | `[HEAP-4]`, `[HEAP-5]`, `[THR-8]` | `Shared[T]` shared across threads keeps a non-atomic `get_mut` check; `get` takes no access |
| MS-03 | S1 | `[CLS-2]`, `[CLS-4]`, `[CLS-10]` | a base `init` can call a virtual method that reads a derived field before it is initialised |
| MS-04 | S1 | `[FFI-21]`, `[FFI-22]` | a retained C callback may capture borrows, and may be called on another thread |
| MS-05 | S1 | `[FFI-10]` | a scalar-only foreign function is safe to call whatever it does |
| MS-06 | S1 | `[THR-6]`, `[CORO-6]` | a generator suspended inside a `thread.scope()` block can be forgotten, so its tasks outlive their borrows |
| MS-07 | S1 | `[RT-10]`, `[WK-12]` | `@sync` counts are "one `fetch_add`, never a compare-exchange", which `Weak.upgrade` cannot be |
| MS-08 | S1 | `[FFI-8]` | C's `char32_t` maps to `char`, so an invalid scalar from C becomes an invalid `char` in Safe code |
| MS-09 | S2 | `[SIMD-4]`, `[SIMD-1]` | gathers, scatters and `load`/`store` have no stated bounds rule |
| MS-10 | S2 | `[CG-C-4]`, `[SIMD-3]`, §VIII.3 | `restrict` on views reached through a class handle, while unchecked instantaneous writes through another handle can modify them |
| MS-11 | S2 | §VIII.3 table, `[RC-5]`, `[BCK-1]` | a view of a class field returned from a method: where its dynamic access ends is unspecified |
| MS-12 | S2 | §VIII.3 table, `[CLO-11]` | calling a callable stored in a class field is not listed as an access, so the closure can be replaced (freed) while it runs |
| MS-13 | S2 | `[FFI-39d]` | "the trampoline takes no long-term access" does not stop a re-entrant C++ call from freeing what the Ember caller still views |
| MS-14 | S2 | nowhere | stack overflow is not required to fault; a large frame can step over a guard page |
| MS-15 | S2 | `[ECS-4]` | `run_parallel` over systems whose access sets are not known at compile time has no rule |
| MS-16 | S3 | `[SOA-6]` | what an element proxy borrows is unstated |
| MS-17 | S3 | `[HASH-3]`, `[STD-15]` | an incoherent `Hash`, a key mutated through a `Cell`, or an inconsistent `Ord` in `sort` must stay memory-safe |
| MS-18 | S3 | `[THR-10]`, `[FFI-27]`, `[STD-24]` | the leak report and at-exit hooks can run while detached threads still run |
| MS-19 | S3 | `[RC-3]` | early deinitialisation and raw pointers into an object |
| MS-20 | S3 | `[TYP-35]` | a phantom parameter is always `Send`/`Sync`, so a library cannot mark a type thread-confined without a field |
| MS-21 | S3 | `[CT-2]` | `comptime.read_file` has no restriction to the package directory |

## S1 holes

### MS-01 — an unscoped thread may capture a view of the caller's locals

`[THR-10]` requires the captured state of `thread.spawn`'s closure to be `Send`; `[THR-8]` makes
`Span[T]` `Send` when `T` is `Sync`. Nothing requires the captured state to carry only the static
region, so:

```ember
import std.thread

fn start() -> thread.JoinHandle[int]:
    local = [1, 2, 3]
    s = local.as_span()                        # Span[int]: Copy, and Send
    return thread.spawn(owned fn() => s[0])    # copies the view into the thread
                                               # `local` is freed on return; the thread reads it
```

`[JOB-2]`'s unscoped `jobs.submit` has the same hole. **Fix:** the closure passed to `thread.spawn` or
unscoped `jobs.submit`, and its result type, must carry only the static region (`[TYP-15]`'s storage
rule applied to a thread transfer), `E3063` naming the captured view, whose help is `thread.scope()`.

### MS-02 — `Shared[T]` across threads

`[THR-8]` makes `Shared[T]` `Send` when `T` is `Sync`. `[HEAP-5]`'s `get_mut` uses the class
exclusivity word, which is plain (non-atomic) memory for anything but a `@sync` class. Two threads
calling `get_mut` on copies of one `Shared[Atomic[int]]` race on the word and both get `ref mut`, and
`mem.replace` through either is then a non-atomic write racing the other. Separately, `[HEAP-4]`'s
`get` begins no dynamic access, so `get_mut` through a second copy succeeds while a `ref T` from `get`
is live; a `T` with an `Array` field can then have its buffer freed under the reader.
**Fix:** `get` begins a dynamic read access for the loan's life and `get_mut` a write access (both
checked); `Shared[T]` is never `Send`; a new `std.sync.SyncShared[T]` (`T: Sync`) has atomic counts
and `get` only, so cross-thread sharing goes through it or a `@sync` class.

### MS-03 — virtual calls in `init` read uninitialised fields

`[CLS-4]` requires a derived `init` to call `super.init(…)` before using inherited fields, but not to
initialise its own fields first; `[CLS-10]` initialises a derived class's defaulted fields after the
base constructor. A base `init` that calls a virtual method overridden in the derived class therefore
runs the override on an object whose derived fields hold no value:

```ember
open class Widget:
    fn init(self):
        self.layout()                          # virtual call from the base constructor
    virtual fn layout(self):
        pass

class ListView(Widget):
    items: Array[String] = []
    override fn layout(self):
        println(self.items.len())              # reads `items` before it has a value
```

**Fix (Swift's two-phase initialisation):** every field default, base and derived, is evaluated before
any `init` body runs; a derived `init` assigns all its own fields that have no default **before**
calling `super.init(…)`, and may not use `self` as a whole (a method call, passing `self`) until
`super.init` has returned. Then every field is initialised whenever any method can run.

### MS-04 — retained C callbacks

`[FFI-21]` boxes a capturing closure for `callback(retained)` without saying what it may capture, and
`[FFI-22]` requires `Send` only for closures that "reach a foreign thread", which the compiler cannot
see. A retained callback capturing a borrow of a local outlives the local; a callback capturing a
non-`@sync` handle can be invoked by C on another thread. **Fix:** a `callback(retained)` or
`callback(once)` closure must be an `owned fn` whose captures carry only the static region; every
callback's captures must be `Send` unless the contract says `threads(main)` or `threads(creator)`.

### MS-05 — scalar-only foreign functions are not safe merely by signature

`[FFI-10]` makes a function in an `unsafe extern` block safe to call when its parameters are scalars.
The signature can be exactly right and the function still unsafe: `void free_addr(uintptr_t)`,
`void memset_at(uintptr_t, int, size_t)`. Safe code can obtain an address as an integer (`[DET-2]`
only marks it `Nondet`), so such a call frees or overwrites Ember memory from Safe code.
**Fix:** as in Rust's `unsafe extern` blocks, each function is unsafe to call unless its declaration
says it is safe: `safe fn abs(x: c_int) -> c_int` (a contextual keyword inside `extern` blocks), an
asserted fact listed in the trusted-base report. Overlay contracts keep their meaning.

### MS-06 — forgetting a generator suspended inside a scope

`[THR-6]` stops a `Scope` itself from being forgotten, and `[CORO-6]` lets a generator hold views of
its **parameters** across a `yield`. A generator that opens `with scope = thread.scope():`, spawns tasks
that borrow a parameter, and yields inside the block can be suspended and passed to `mem.forget`: the
scope's drop, which joins the tasks, never runs, and the caller then frees what the tasks still use.
**Fix:** a `yield` while a `@must_drop` value is live is `E2231` (the same shape as a frame-local
borrow across `yield`). Generators remain free to use scopes that close before they yield.

### MS-07 — `Weak.upgrade` on a `@sync` object

`[RT-10]` says "a `@sync` retain is one atomic `fetch_add`, never a compare-exchange loop". Upgrading a
weak handle is also a retain, but it must not succeed once the strong count has reached zero on
another thread; a blind `fetch_add` from zero resurrects an object that thread is freeing.
**Fix:** state that `[RT-10]` covers copying an existing strong handle only; `Weak.upgrade` on a
`@sync` object is a compare-exchange that fails at zero.

### MS-08 — `char32_t` from C

`[FFI-8]` maps `char32_t` to `char`. A C function can return a surrogate or a value above U+10FFFF;
`[TYP-3]` makes an invalid `char` undefined behaviour producible only in `unsafe`, but here Safe code
receives one. **Fix:** `char32_t` maps to `u32`; `char.from_u32(v) -> Option[char]` converts.

## S2 holes

### MS-09 — SIMD memory operations

`[SIMD-4]` lists gathers and scatters with no bounds rule, and `[SIMD-1]`'s `load`/`store` take a view
of unstated length. **Fix:** `load(s)`/`store(s)` require `s.len() >= LANES` (checked, `Bounds`);
`gather(s, idx)`/`scatter(s, idx, v)` check every lane's index; unchecked forms exist only in `unsafe`.

### MS-10 — `restrict` on views reached through a class handle

§VIII.3 leaves instantaneous accesses — writing a `Copy` field, including an element of a fixed-array
field — unchecked, so another handle may write such a field while a view of it is live. `[CG-C-4]`
may still declare that view's base pointer `restrict` when `[SIMD-3]`(a) proves it disjoint from the
loop's other views; a write through another handle inside the loop (through a call) then violates
`restrict`, which is undefined behaviour in C. **Fix:** a view reached through a class handle or
`Shared` receives no alias fact unless the loop contains no call and no access through any other
handle (the conditions of `[EXC-3]`); and state that a reference into a class object's `Copy` field
may observe writes made through other handles.

### MS-11 — where a view's dynamic access ends

§VIII.3 gives a view of a class field the duration "the reference's liveness". When a method returns
such a view (`fn items(self) -> Span[int]: return self.items`), the access begins in the callee and
must end in the caller, possibly on several paths. **Fix:** state that the dynamic access a borrow
through a class handle begins ends where that borrow's loan dies (`[BCK-2]`), in whichever function
that is; a view returned by a method carries its access to the caller.

### MS-12 — calling a callable field

A callable stored in a class field (`[CLO-11]`) is called through the object, but calling it is not in
§VIII.3's table of long-term accesses. The closure can re-enter the object and replace the field
(`b.on_click = other`, a write access by `[EXC-16]`), dropping the running closure and its captures.
**Fix:** calling a callable stored in a class field is a long-term **write** access to that field for
the call; replacing it or calling it again from inside the call panics.

### MS-13 — re-entrant C++ calls

`[FFI-39d]` says re-entrant calls from C++ into an Ember object never panic because the trampoline
takes no long-term access. But the Ember caller may hold one — a `mut self` method, or a view of a
field — across the C++ call, and the override may then reassign that field, freeing what the caller
views. **Fix:** accesses held across a call into C++ stay active and are checked like any other: an
override that conflicts panics. `L3013` warns where a long-term access is held across a C++ call that
may call back; the supported idiom is to make such calls outside long-term accesses.

### MS-14 — stack overflow

Nothing requires stack overflow to fault. With a large frame and no stack probes, a function's stack
writes can land beyond the guard page, in another thread's stack or the heap. **Fix:** the backend
compiles with stack probes (`-fstack-clash-protection` where available; MSVC's probes are default), the
runtime gives every thread it creates a guard page, and overflow aborts with `stack overflow in
<function>`.

### MS-15 — dynamic ECS systems

`[ECS-4]` rejects two conflicting systems in `run_parallel` when both are known at compile time; with
systems passed as values it says nothing, and two systems writing one storage would race. **Fix:**
`run_parallel` checks the access sets of systems not known at compile time before starting them and
panics on a conflict, naming both.

## S3 clarifications

* **MS-16** — `ps[i]` as `SoAMut[T]` mutably borrows all of `ps` for the proxy's life; `SoARef[T]`
  shares it. A column borrowed separately conflicts with a live proxy.
* **MS-17** — an incoherent `Hash`, a key changed through a `Cell`, or an inconsistent `Ord` passed to
  `sort` gives wrong answers (any permutation, a missing key) but never undefined behaviour, an
  out-of-bounds access or a non-terminating operation.
* **MS-18** — at process exit the runtime frees no statics and runs no drops; the `debug` leak report
  runs only when no other thread is running, and otherwise says how many were still running.
  `process.exit` stops the process without running at-exit Ember code on other threads.
* **MS-19** — `unsafe` code that keeps a raw pointer into an object must keep a handle to it alive
  (`mem.keep_alive`), because `[RC-3]` may deinitialise the object after its last Safe use.
* **MS-20** — a phantom parameter contributes to `Send` and `Sync` as a field of that type would, so a
  marker such as `ThreadBound` can make a type thread-confined without a field.
* **MS-21** — `comptime.read_file` reads only files inside the package directory (`E6010` otherwise), so
  a dependency's build cannot read the builder's files.

## Checked and sound

`@sync` field immutability and the data-race argument (`[THR-1]`, `[THR-13]`); scoped-task borrows
(`[THR-11]`); location-sensitive borrow checking (§XVIII.4); generator frame-local borrows
(`[CORO-6]`); the arena rewind rule (`[ARN-7]`); compile-time data in read-only memory (`[CT-5]`);
view storage (`[TYP-15]`); two-phase borrows; `Pool` generations (`[HND-3]`); FFI layout assertions
(`[FFI-5a]`); grouped overflow checks (`[SIMD-7]`: writes are thread-private and the process aborts,
and bounds checks are never grouped); hot-reload publication (`[HR-42]`); resurrection (`[OBJ-5]`).
