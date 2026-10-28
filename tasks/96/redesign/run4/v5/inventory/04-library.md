# 04 The standard library, built-in functions and the runtime

Agent 04 (library), 2026-10-27. Read: `std/src/*.em` and `std/src/math`, `std/src/ser` (the smaller
modules in full; `core.em`, `collections.em`, `ecs.em`, `simd.em`, `math/*`, `ser/*` in full where they
hold an `unsafe` block, a raw pointer, a hand-written `Send`, a compiler-known function or a type built
on one, and by search for everything else); the compiler's built-ins (`compiler/ember_hir/src/lib.rs`
1526-1700 `RuntimeFn`, 1706-2330 `Builtin`; where `ember_typeck` resolves them and asks for `unsafe`,
every `E3100` site); `runtime/ember_rt/include/ember_rt.h` and the parts of `src/ember_rt.c` behind
the functions a program reaches. Nothing was compiled or run: every "what it does" below is read from
the code named, and says so. `V5` = `tasks/96/redesign/run4/v5/`, `SPEC` =
`docs/spec-source/Ember_v0.9.10_Hardened_35.md`.

Ten findings, in order of harm. Then: every `unsafe` item of the library and the built-ins with its
`@safety` text (section B), what was checked and is not a finding (section C), and what I could not
settle (section D).

---

## A. Findings

### L1. `Allocator.allocate`, `deallocate` and `reallocate` can be called without `unsafe`

- **feature**: the interface `pub unsafe interface Allocator` (`std/src/mem.em:121-124`), its methods
  `fn allocate(self, layout: Layout) -> *mut u8`, `fn deallocate(self, p: *mut u8, layout: Layout)`,
  `fn reallocate(...)`; implemented for `Global` (`mem.em:135-146`, runtime `free`/`ember_free`) and
  `System` (`mem.em:167-179`, runtime `ember_system_alloc/free/realloc`, `ember_rt.c:2827-2846`). Only
  *implementing* the interface needs `unsafe` (`SPEC` 1269-1270, `[ALC-2]` 2989-2997; checked at
  `ember_typeck/src/lib.rs:14068-14083`). The methods are not `unsafe fn`, and the only call check is
  for a callee marked `unsafe` (`errors.rs:257`, `lib.rs:16478`, `47712-47719`); `Layout` has public
  fields (`mem.em:110-112`), and a raw pointer is taken safely (`[SPN-8]`, `SPEC` 2550; `as_mut_ptr`,
  `lib.rs:55442-55470`).
- **how it breaks safety**: kinds 1 and 6 (a double free, a read of freed memory; an alignment that is
  not a power of two or a `free` that does not match the allocation is undefined in C). In `V5`'s
  spelling (`std.mem` is unchanged there):

  ```ember
  from std.mem import Global, Layout

  fn main():
      xs: Array[u8] = [1, 2, 3, 4]
      d = ref mut xs.as_mut_span()
      p = d.as_mut_ptr()                                  # safe: taking a raw pointer
      Global().deallocate(p, Layout(size=4, align=1))     # no `unsafe`: frees the list's buffer
      println(xs[0])                                      # reads freed memory
  ```                                                     # xs's end frees the buffer again

  What it does when run: `Global.deallocate` is the runtime's `free` (`ember_hir/src/lib.rs:1661`), so
  `xs`'s buffer is freed, `xs[0]` reads freed memory, and `xs`'s end frees it a second time (heap
  corruption). `System().deallocate(p, Layout(size=1, align=64))` on a block `malloc` made calls
  `_aligned_free` on it under MSVC (`ember_rt.c:2672-2684`), and `System().allocate(Layout(size=8,
  align=3))` calls `aligned_alloc(3, 9)` (`ember_rt.c:2655-2660`), undefined in C. A program's own
  memory manager is reached the same way (`current.deallocate(...)` on a `LevelArena`, verdict 24).
  Sure: read from the declarations and the call check; not compiled.
- **fenced?**: no, today or in `V5`. `SPEC` fences only the implementing (`[TYP-43]`, 1269: "An `unsafe
  interface` (`Allocator`, `[ALC-2]`) is implemented only by an `unsafe extend`"); `V5/vocabulary.md`
  row `Allocator` (line 340) keeps it "as today"; `V5/rules.md` 210-230 has the compiler call
  `allocate`/`deallocate` for `using`, and says nothing of a program's own calls.
- **Rust**: `GlobalAlloc::alloc`, `dealloc` and `realloc` are `unsafe fn`; the unstable `Allocator`
  has `deallocate` and `grow`/`shrink` as `unsafe fn` (only `allocate` is safe, and it gives a
  `NonNull<[u8]>`, never frees).
- **recommendation**: fence. Declare the three methods `unsafe fn` in the interface (the `using`
  calls the compiler writes are its own and unaffected). A call outside `unsafe:` gets E3100:
  "`deallocate` gives memory back that Ember may still use, and a wrong `Layout` is undefined; write it
  inside `unsafe:` with a `# SAFETY:` note". Also check `Layout`'s `align` (a power of two) in a
  constructor, so a wrong one cannot reach C even inside `unsafe`. No cost while running.
- **settled?**: verdict 60 ("anything that can break a safety promise is possible only inside an
  `unsafe` block"). Not for the owner: ordinary programs never call these.

### L2. A query parameter written by hand makes `run_parallel` race

- **feature**: `pub interface ParamOps` and `pub interface QueryParam` (`std/src/ecs.em:410-446`), which
  any program may implement (no `unsafe`); `World.run_parallel` (`ecs.em:336-371`) checks the systems'
  access sets from `Q.Ops.accesses(...)` at run time, and the compiler's checks (`E7020`, and `E7004` at
  `std.ecs.system`, `ember_typeck/src/lib.rs:38960-39021`) read only the associated types `Reads` and
  `Writes`. Each system's query then goes to a job through `SystemWork`, which is `Send` by hand
  (`ecs.em:1271-1276`); its `# SAFETY:` note relies on those access sets being true. Nothing checks
  that `Fetch` and `View` are `Send` or that the access sets match what `get_mut` touches.
- **how it breaks safety**: kind 2 (two threads change one `World` and one storage with nothing
  ordering them), then kind 1 (a `Map` and non-atomic counts of class objects changed by two threads
  at once). In `V5`'s spelling:

  ```ember
  from std.ecs import World, Query, ParamOps, QueryParam, Access, Entity, system

  @derive(Component)
  struct Pos:
      x: int

  struct Sneak:
      pass

  struct SneakOps:
      pass

  extend SneakOps implements ParamOps:
      type Item = ()
      type ReadItem = ()
      type Fetch = World          # a one-thread class object
      type View = World
      type Reads = ()             # says: reads nothing
      type Writes = ()            # says: writes nothing
      static fn fetch(world: World) -> World:
          return world
      static fn view(fetch: ref World) -> World:
          return fetch
      static fn driver(view: World) -> Option[Span[Entity]]:
          return None
      static fn get(view: ref World, e: Entity) -> Option[()]:
          return None
      static fn get_mut(ref mut view: World, e: Entity) -> Option[()]:
          view.add(e, Pos(x=1))   # changes the world's storages
          return Some(())
      static fn accesses(ref mut out: Array[Access]):
          pass                    # reports no access

  extend Sneak implements QueryParam:
      type Ops = SneakOps

  fn touch(ref mut q: Query[Sneak]):
      for _ in q.iter_mut():
          pass

  fn main():
      w = World()
      for _ in 0..1000:
          _e = w.create()
      w.run_parallel([system(touch, "a"), system(touch, "b")])
  ```

  What it does when run: both systems pass the compile-time check (`Reads`/`Writes` empty) and the
  run-time check (no accesses), run on two jobs at once, and both call `World.add`, which changes
  `world.storages` (a `Map`) and an `AosStorage`'s lists from two threads, and retains and releases
  the non-`@sync` `World` and storage objects with plain (non-atomic) counts (`ember_rt.h:1528-1548`).
  The map's buckets and the counts can be torn: lost entries, a count off by one, an object freed while
  the other thread uses it. Fairly sure (the checks read are the only ones; not compiled).
- **fenced?**: no. `SPEC` `[ECS-4]` (3827-3836) states the checks; none covers a hand-written
  `ParamOps`. `V5` keeps `std.ecs` as is (`vocabulary.md` 440: `World`, `Query` ... "same").
- **Rust**: no ECS in the standard library; Bevy marks the same traits `unsafe trait WorldQuery` and
  `unsafe trait SystemParam`, so a hand-written one needs `unsafe impl`.
- **recommendation**: fence. Make `ParamOps` and `QueryParam` `unsafe interface`s (implemented only by
  `unsafe extend`, E3100 otherwise, "a query parameter promises its access sets; `@derive(Component)`
  writes them for you"); `@derive(Component)` and the library's own `Read`/`Write`/`Option`/`Not`/
  `Entity` implementations write them as today. Belt and braces: `std.ecs.system` also checks
  `Q.Ops.Fetch` and `Q.Ops.View` for `Send` (E7004). No cost while running.
- **settled?**: verdict 60. Not for the owner: only a program that writes its own query parameter is
  touched.

### L3. A struct named `Cell_...` in the main file is treated as a `Cell` when deciding `Send`

- **feature**: the compiler's thread-properties rule, `ember_types/src/lib.rs:1875-1879`: a struct with
  no generic origin whose name starts with `Cell_` or `RefCell_` is `Send` when its *first field* is,
  and never `Sync`. Compiler-made `Cell` instances carry that prefix, but so may a program's own
  struct: names in the root module are stored unqualified (`ember_typeck/src/lib.rs:1736-1742`,
  `6242-6249`). The C backend guards its own use of the prefix with `declaring_module == usize::MAX`
  ("not a spelling convention a program can forge", `ember_codegen_c/src/lib.rs:5424-5433`); the
  thread rule does not.
- **how it breaks safety**: kind 2. In `V5`'s spelling (main file):

  ```ember
  from std import thread

  class Counter:
      n: int = 0

  struct Cell_pair:            # the name starts with "Cell_"
      pad: int                 # first field: an int, which is Send
      c: shares mut Counter    # a one-thread owner

  fn main():
      k = shares mut Counter()
      b = Cell_pair(pad=0, c=shares mut k)
      h = thread.spawn(fn() owns b: for _ in 0..1000000: b.c.n += 1)
      for _ in 0..1000000:
          k.n += 1
      h.join()
      println(k.n)
  ```

  What it does when run: `Cell_pair` is judged `Send` from `pad` alone, so the capture passes E7004
  (and `V5`'s E7015 if it uses the same rule); two threads then add to one `n` and change the
  counter's plain owner count with nothing ordering them: the total printed is below 2000000, and the
  count can end wrong (the counter freed while used, or never freed). Renaming the struct `Pair` gets
  E7004. Fairly sure (read from the two places above; not compiled).
- **fenced?**: no; it is a defect of today's compiler, not a rule. `V5` infers `Send` and
  `ThreadShared` "from the words" (`vocabulary.md` 271-272) and would keep the defect if the same code
  is used.
- **Rust**: `Send` and `Sync` are decided from the fields, and `Cell<T>` is `!Sync` by an `impl` on
  that exact type, never by its name.
- **recommendation**: make it safe: recognise the compiler's `Cell`/`RefCell` structs by
  `declaring_module == usize::MAX` (as the C backend does), and do the same in the other prefix tests
  (`holds_a_cell`, `ember_types/src/lib.rs:1710-1716`; `has_implicit_eq_in`,
  `ember_typeck/src/lib.rs:15991-15999`; `is_compiler_known_struct`, `60373-60378`). No cost.
- **settled?**: settled by the rules as written (`Send` is what a type holds, vocabulary row `Send`);
  a compiler fix. Not for the owner.

### L4. `V5` gives `jobs.submit` no "any thread may run it" word

- **feature**: `pub fn submit(owned f: fn()) -> JobHandle` (`std/src/jobs.em:63`): the job runs on a
  pool thread and may outlive its caller. Today the compiler checks what it captures (`Send`, no view)
  in `synth_jobs_submit` (`ember_typeck/src/lib.rs:52510-52548`). `V5`'s R15 changes only
  `thread.spawn`'s line to `thread_shared fn()` (`rules.md` 844-849) and keeps `scope.spawn` plain; it
  names neither `jobs.submit` nor `JobScope.submit` nor the function `@parallel` and `run_parallel`
  hand to the pool.
- **how it breaks safety**: kind 2, if the library line is carried over as written:

  ```ember
  import std.jobs

  class Counter:
      n: int = 0

  fn main():
      count = shares mut Counter()
      h = jobs.submit(fn() shares mut count: count.n += 1)   # a one-thread owner on a pool thread
      count.n += 1
      jobs.wait(h)
      println(count.n)
  ```

  What it would do: the job thread and `main` change `n` and the counter's plain owner count at once
  (prints 1 or 2; the count can be left wrong). Today it is refused by the built-in check; under `V5`'s
  rules as written, only R15's "an unscoped thread ... say in the capture list" (rules.md 876-879) might
  catch it, and it does not say a job is an unscoped thread. Sure that `V5` does not name it; not sure
  the gap is real once R15 is read generously.
- **fenced?**: today yes (E7004 in the built-in); in `V5` not by a written line.
- **Rust**: `std::thread::spawn` and Rayon's `rayon::spawn` both require `F: Send + 'static`.
- **recommendation**: make it safe with verdict 61's word: `pub fn submit(owns f: thread_shared fn())
  -> JobHandle:`, refused with E7015 as for `thread.spawn`; `JobScope.submit` stays a plain `fn()` as
  `scope.spawn` does (verdict 94's reason applies). No cost.
- **settled?**: verdicts 61 and 94 (`thread.spawn`'s line); apply the same to `jobs.submit`. Not for the
  owner.

### L5. `Arena.alloc_nodrop` leaks what the value owns, without `unsafe`

- **feature**: `arena.alloc_nodrop(owned value)` (`SPEC` `[ARN-2]` 2936-2937: "Allocating a type that
  needs drop is `E3090`, unless through `alloc_nodrop`, which acknowledges that its `drop` will never
  run"; built-in `ArenaAlloc`, `ember_hir/src/lib.rs:2265-2270`). `V5` keeps `Arena` "as today"
  (`vocabulary.md` 321).
- **how it breaks safety**: kind 3 (the same harm verdict 58 removed `mem.forget` for):

  ```ember
  class Player:
      hp: int = 10

  fn main():
      frame = Arena.with_capacity(4096)
      p = shares Player()
      for _ in 0..1000000:
          _note = frame.alloc_nodrop(String("frame text"))   # its text buffer is never freed
          _keep = frame.alloc_nodrop(shares p)               # p's owner count never comes down
          frame.reset()
  ```

  What it does when run: each round leaks one `String` buffer, and `p`'s owner count rises by one per
  round and never falls, so the player is never freed (verdict 58's "leaving a count higher and causing
  a leak"); after 4,294,967,295 rounds verdict 59's check stops the program. Sure (spec text).
- **fenced?**: no; nothing in `SPEC` or `V5` puts it inside `unsafe`.
- **Rust**: `bumpalo::Bump::alloc` never runs `Drop` and is safe; Rust counts a leak as safe.
- **recommendation**: fence: `alloc_nodrop` only inside `unsafe:` (E3100: "`alloc_nodrop` never runs
  the value's clean-up, so what it owns is never freed; write it inside `unsafe:`, or allocate a value
  that needs no clean-up with `alloc`"). Optional later: an arena that runs the drops of what it holds
  at `reset()` (as `typed_arena::Arena` does), which makes the ordinary case need no `unsafe`; its cost
  is a list of drop calls per arena, only for types that need one.
- **settled?**: verdicts 58 and 60 (leaving a clean-up undone is the broken promise; verdict 60 puts
  it inside `unsafe`). Not for the owner by the brief's test, though a game that keeps `String`s in a
  frame arena gains an `unsafe:` line.

### L6. `MaybeUninit.write` and `write_at` leak the value they overwrite or hold

- **feature**: `MaybeUninit[T].uninit()` (safe), `write(mut self, owned v) -> ref mut T` "stores without
  dropping previous bytes", and "dropping it never drops a `T`" (`SPEC` `[ARN-8]` 2969-2973);
  `write_at(i, v)` on a `MutSpan[MaybeUninit[T]]` the same (`[ARN-9]`; built-ins
  `MaybeUninitWrite`, `MaybeUninitWriteAt`, `ember_hir/src/lib.rs:2169-2185`). Only `assume_init` needs
  `unsafe` (`ember_typeck/src/lib.rs:49949-49960`, `55677-55688`). `V5` keeps it "as today (`unsafe`)"
  (`vocabulary.md` 326), which is true of `assume_init` only.
- **how it breaks safety**: kind 3:

  ```ember
  fn main():
      slot = MaybeUninit[String].uninit()
      _a = slot.write(String("first"))
      _b = slot.write(String("second"))   # "first" is never freed
  ```                                     # slot ends: "second" is never freed either

  What it does when run: both strings' buffers are never freed. Sure (spec text).
- **fenced?**: no.
- **Rust**: `MaybeUninit::write` is safe and leaks the same way.
- **recommendation**: fence the leaking case: `write` and `write_at` of a `T` that needs clean-up only
  inside `unsafe:` (E3100: "`MaybeUninit` never frees what it holds; write it inside `unsafe:` and end
  the value with `assume_init`"); a `T` that needs none (numbers, `Copy` structs, the case arenas use)
  stays safe. No cost.
- **settled?**: verdict 60, by verdict 58's reason. Not for the owner.

### L7. A channel that holds one of its own `Sender`s is never freed

- **feature**: `Receiver.drop` (`std/src/sync.em:255-261`) marks the receiver gone and wakes senders,
  but leaves the queued values in the queue; the channel's state (`SyncShared[ChannelState[T]]`,
  `sync.em:157-168, 278`) is freed only when its last owner ends, and every `Sender`, queued ones
  included, is an owner.
- **how it breaks safety**: kind 3 (a loop of owners):

  ```ember
  from std.sync import channel, Sender

  fn main():
      for _ in 0..1000000:
          (tx, rx) = channel[Sender[int]](capacity=1)
          _ = tx.send(copy of tx)      # a second sender, waiting in its own channel
  ```                                  # tx and rx end; the queued sender keeps the channel alive

  What it does when run: every round leaves a channel (its lock, ring, counters and the queued
  `Sender`) that nothing can reach; none is ever freed. Sure (read from `sync.em`).
- **fenced?**: no.
- **Rust**: `std::sync::mpsc` (crossbeam-based since 1.67) discards the queued messages when the
  `Receiver` is dropped, so this program frees everything; sending the `Receiver` into its own channel
  still leaks in Rust.
- **recommendation**: make it safe: `Receiver.drop` takes every queued value out under the lock and
  ends them after releasing it (no cost but at the receiver's end). What remains, a `Receiver` sent
  into its own channel, is the general loop of shared owners (agent 05's case), not a library one.
- **settled?**: no verdict names channels; the fix only frees values no program can receive, so it does
  not change what an ordinary program does. Not for the owner.

### L8. The weak count of a `thread_shared` object is compared, then added to, in two steps

- **feature**: `ember_weak_retain` (`runtime/ember_rt/src/ember_rt.c:3323-3340`): it loads the weak
  count, refuses `UINT32_MAX`, then for a Sync object does `fetch_add` separately. Two threads at
  `UINT32_MAX - 1` both pass and the count wraps to 0. The strong count's Sync path
  (`ember_rt.h:1915-1922`) adds first and compares the old value after, so it wraps to 0 for the
  instant before the panic (no free follows: a retain or release that then sees 0 panics too).
- **how it breaks safety**: kinds 1 and 3, verdict 59's check skipped by a race. Not reachable at the
  real size (4 billion links, about 32 GB); reachable in verdict 96's 8-bit test build:

  ```ember
  from std import thread

  class Node:
      v: int = 0

  fn main():                                    # built with the 8-bit test count (verdict 96)
      n = thread_shared Node()
      links: Array[weakref Node] = []
      for _ in 0..253:
          links.push(weakref n)                 # weak count 254 (one held by the owner)
      with scope = thread.scope():
          scope.spawn(fn(): _a = weakref n)     # both read 254, both pass, count wraps to 0
          scope.spawn(fn(): _b = weakref n)
  ```

  What it may do: instead of stopping with "too many `weakref` links", the count wraps; a later link's
  end then panics on a zero count or, with an unlucky order, frees the header while links remain. Sure
  of the code; the harm needs the race.
- **fenced?**: no; it is verdict 59's check, done in two steps.
- **Rust**: `Arc::downgrade` uses a compare-exchange loop; `Weak::clone` adds and then aborts if the
  old count is above `MAX_REFCOUNT` (`isize::MAX`), half the range away from wrapping.
- **recommendation**: make it safe: one `fetch_add`, then compare the value it returns (as the strong
  path does), with the stop at a limit below the wrap (for example `UINT32_MAX - 2^16`), so other
  threads' adds before the stop cannot reach 0. One compare, as now.
- **settled?**: verdicts 59 and 96. Not for the owner.

### L9. A `Pool` keeps every retired slot for the rest of the program

- **feature**: `Pool.remove` retires a slot whose generation is used up and never reuses it
  (`std/src/collections.em:1464-1476`, `[HND-3]`); the slot's `Option[T]` and generation stay in the
  pool's lists. With `Split20` (12-bit generations, `collections.em:1335-1353`) that is one slot per
  4096 reuses.
- **how it breaks safety**: kind 3, bounded:

  ```ember
  from std.collections import Pool, Split20

  fn main():
      pool = Pool[int, Split20]()
      for i in 0..100000000:
          h = pool.insert(i)
          _ = pool.remove(h)
  ```

  What it does when run: about 24,400 slots (about 600 KB) end up retired, held and never used; after
  about 4.3 billion removals all 2^20 indices are retired and `insert` stops the program. With the
  default `Split32` it is one slot per 4 billion reuses. Sure (read from the code).
- **fenced?**: no (it is the design).
- **Rust**: no pool in the standard library; generational arenas in Rust crates either retire the slot
  or let the generation wrap (an old handle can then match a new value).
- **recommendation**: keep as an understandable limitation (JOB.md, "Annoying, or an understandable
  limitation"): reusing the slot would let an old handle reach a new value. Say it in `Pool`'s
  documentation and in `Split20`'s ("one slot per 4096 removals is kept").
- **settled?**: no verdict; the recommendation changes no program. Not for the owner.

### L10. `Volatile.read_volatile` and `write_volatile` do not say the pointer must be aligned

- **feature**: `@safety("the pointer is valid for a read of a `T`")` and "valid for a write"
  (`std/src/mem.em:27-40`); the C a volatile access becomes is undefined on a misaligned pointer.
- **how it breaks safety**: kind 6, inside `unsafe`, through a promise the text does not ask for:

  ```ember
  from std.mem import Volatile

  fn main():
      bytes: Array[u8] = [0, 0, 0, 0, 0, 0, 0, 0]
      p = bytes.as_mut_span().as_mut_ptr()
      unsafe:  # SAFETY: the pointer is valid for a read of a u32 (all the @safety text asks)
          q = Volatile.new(p.add(1) as *mut u32)
          println(q.read_volatile())             # a misaligned 4-byte read: undefined in C
  ```

  What it may do: work on x86-64, fault on a strict-alignment target, or be miscompiled. Sure.
- **fenced?**: yes, inside `unsafe` (`ember_typeck/src/lib.rs:16650-16655`); the gap is the text.
- **Rust**: `ptr::read_volatile`'s safety section: "src must be valid for reads" and "src must be
  properly aligned".
- **recommendation**: change the three texts to "the pointer is valid for a read (write) of a `T` and
  aligned to `T`'s alignment".
- **settled?**: verdict 60 (an `unsafe` operation's promise is the caller's, so it must be stated). Not
  for the owner.

---

## B. Every `unsafe` item of the library and the built-ins, and its `@safety` text

| item | where | needs `unsafe`? | its text says what the caller must make true? |
|---|---|---|---|
| `Volatile.read_volatile`, `write_volatile` (x2 read, 1 write) | `mem.em:27-40` | yes (E3100 at `lib.rs:16654`) | partly: no alignment (L10) |
| `assume_disjoint` | `mem.em:90-92` | yes (`lib.rs:16259-16265`) | yes: "The two views' address ranges do not overlap." |
| `Allocator` (implementing) | `mem.em:121` | implementing yes; calling no (L1) | the doc comment states the implementer's promises (`mem.em:114-120`) |
| `unsafe extend Global/System implements Allocator` | `mem.em:134-179` | written by the library | `# SAFETY:` comments; fine |
| `std.ffi.adopt` | `ffi.em:194-196` | yes (`lib.rs:16477-16480`, `16540-16542`) | yes (live, owned by nothing else, `T`'s destructor frees it) |
| `CppVector/CppFunction/CppString/CppShared/CppWeak.from_thunks` | `ffi.em:288, 339, 366, 443, 500` | yes | yes (the object is live, owned by nothing else, the functions are its thunks) |
| `unsafe extend ForeignBox implements Send` | `ffi.em:188` | written by the library | relies on a thread check that runs only in `debug` (`SPEC` 4930-4932); `V5` replaces `ForeignBox` by `owns T` and refuses a `threads(creator)` handle sent to another thread when compiling (E7008, `rules.md` 1201), so not a finding |
| `dyn_payload` | `core.em:247` | yes (`errors.rs:256-258`), private to `std.core` | no `@safety`, only a `# SAFETY:` comment; its one caller checks the type first (`core.em:134-139`). Add the `@safety` text for consistency |
| `Pool.get_unchecked` | `collections.em:1445-1452` | yes | yes ("`h` is live"); its body checks anyway (the index through `slots[...]`, the empty slot by a stop), so it is safe even when the promise is broken |
| `unsafe extend AddChange implements Send` | `ecs.em:1188-1192` | written by the library | relies on `Commands.add`'s run-time stop for a non-`Send` component in a parallel system (`ecs.em:1226-1229`): sound, but checked while running, not when compiling |
| `unsafe extend SystemWork implements Send` | `ecs.em:1271-1276` | written by the library | relies on access sets a program may write by hand (L2) |
| `unsafe:` blocks in `QueryIter.next`, `QueryIterMut.next` | `ecs.em:1122, 1145` | — | read `entities` through a raw pointer held for the run (section D, item 1) |
| `unsafe:` blocks in `CppVector`, `CppString`, `CppShared`, `CppWeak`, `CppFunction` | `ffi.em:298-534` | — | each calls the object's own thunk; sound given `from_thunks`' promise |
| built-ins `alloc`, `free`, `read`, `write` | `lib.rs:16790-16825` | yes | spec `[UNS-1]`/`[UNS-5]` |
| raw pointer `read`, `write`, `add` | `lib.rs:45795-45805` | yes | `[UNS-1]` |
| raw pointer to integer, pointer to pointer casts | `lib.rs:37580-37589` | yes (except a constant address) | `[UNS-1]` |
| a C union's field read | `lib.rs:47575-47582` | yes | `[FFI-8]` |
| `Span.get_unchecked` | `lib.rs:55524-55534` | yes | `[SPN-2]` |
| SIMD `load/store/gather/scatter_unchecked` | `lib.rs:51676-51684` | yes | `[SIMD-9]` |
| `MaybeUninit.assume_init` (value and span) | `lib.rs:49949-49960`, `55677-55688` | yes | `[ARN-8]`, `[ARN-9]` |
| `UnsafeCell.get` | `lib.rs:50015-50023` | yes | `[UNS-10]` |
| `T.new_unchecked(v)` of a range type | `lib.rs:44563-44575` | yes | `[RNG-10]` |
| a call of an `unsafe fn` value | `lib.rs:47712-47719` | yes | `[UNS-1]` |
| C++ handle from an address, C++ span from a raw pointer | `lib.rs:17356-17357`, `17415-17417` | yes | `[FFI-39]`, `[FFI-17e]` |
| implementing `Send`/`Sync` by hand | `lib.rs:14005-14015` | yes (`unsafe extend`) | `[THR-8]` |

Safe built-ins that touch raw memory, and why they are safe: `as_ptr`/`as_mut_ptr` (a raw pointer is
only an address until read, `[SPN-8]`); `is_null`; `null[*T]()`; `size_of`, `align_of`, `offset_of`;
`Retained.pin`, `tok.ptr()` and `ObjectPtr` (an address for C; turning it back into an object needs
`unsafe`); `type_is_send`/`type_is_sync`; `format_to` (its text never grows past the caller's buffer,
`ember_hir/src/lib.rs:1879-1886`); SIMD `load`/`store` (every lane checked, unaligned loads,
`ember_simd.h:73-78`).

---

## C. Checked, not findings

- `mem.forget` (`mem.em:97`) and `ForeignBox.into_raw` (`ffi.em:177-181`, which calls `mem.forget`) are
  safe today; verdict 58 removes `forget` and verdict 91 makes `into_raw()` the compiler's, only inside
  `unsafe` (`vocabulary.md` 309, 313). Not findings by the brief; the next library must replace
  `ffi.em:180`'s use.
- `Shared`, `SyncShared`, `Weak`, `RefCell`, `Box`, `ForeignBox` are replaced in `V5` (`vocabulary.md`
  247-251, 309); their replacements are agent 01's.
- `mem.drop`, `take`, `replace`, `swap`, `keep_alive`, `assert_disjoint*`: safe and sound (`swap` of
  one place twice is E3022).
- `Cell` (`CellSet` stores before it drops, `CellUpdateDefault` puts a default in first, so a
  re-entering `drop` or function never sees a torn cell, `ember_hir/src/lib.rs:2134-2166`).
- `Mutex`, `RwLock`, `Atomic`, `Once`, channels' thread properties (`ember_types/src/lib.rs:1836-1866`):
  match Rust's (a channel's `Receiver` is also `Sync` here, but every receiving method takes it
  changeable, so sharing it reads nothing). `Atomic(v)` refuses anything but an integer, `bool` or raw
  pointer (`lib.rs:52845`). A panic aborts, so no lock is left half-changed (`sync.em:7`).
- `Sender.clone`'s `fetch_add` on the sender count panics after the wrapped value is stored
  (`sync.em:18-19, 218`); since a panic aborts, nothing runs on the wrapped count.
- Every collection in `collections.em` (`Map`, `Set`, `Deque`, `RingBuffer`, `SmallArray`,
  `FixedArray`, `FixedString`, `BitSet`, `Pool`) is written in checked Ember: no `unsafe` but
  `get_unchecked`; a lying `Hash` or `Eq` gives wrong answers, never bad memory. The arena collections
  are built-ins with checked indices (`[ARN-5c]`).
- A program's own `Component` with a wrong `component_hash` or `component_soa`: the storage is taken
  with a checked `cast ... as AosStorage[T]` (`ecs.em:255-268`), which stops the program.
- `World.destroy`'s generation wraps after 4096 reuses of an index (`ecs.em:241`), so an old `Entity`
  can name a new one: a wrong entity, not bad memory (no promise like `[HND-3]` is made for entities).
- `std.fs.File` holds a C `FILE*` in a private `int` (`fs.em:78-79`, `ember_rt.c:10740-10758`): it
  cannot be made or copied outside `std.fs`, and its end closes it once.
- `std.process.exit` and `abort` skip destructors (`process.em:1-9`); the process ends, so nothing
  leaks while it runs. Detached threads at `main`'s end: the runtime skips the leak report and frees no
  table while another thread runs (`ember_rt.c:1071-1091`).
- `@overflow(wrap)` in the library (hashing, `random.em`) is done in unsigned C
  (`ember_codegen_c/src/lib.rs:2497-2502, 11573-11581`), never signed overflow.
- `AnyError.downcast_ref` compares a 128-bit hash of the type's qualified name before its `unsafe`
  read (`core.em:134-139`, `errors.rs:248-254`); within one program the names are unique
  (`lib.rs:18600-18626`). See D, item 2, for separately compiled code.
- `std.coroutine.Scheduler` stores generators as `Box[dyn Sequence]`; a paused generator walking a list
  is verdict 16's rule, not the library's.

## D. Could not settle (what would decide each)

1. **`World.run` with the world as its argument** (`ecs.em:289-305`). The query's entity list is read
   through a raw pointer (`q.entities`, `ecs.em:1118-1124`) taken before the system runs; a system
   given the world itself (`w.run(fn(ref mut q: Query[Pos], w: World): ... w.create() ..., w)`) could
   grow `world.live` or a storage's `entities` (a public field, `ecs.em:91`) while it iterates, and the
   next `read` would see freed memory. `_held = live.len()` keeps the view of `world.live` alive across
   the call, but the `Some(list)` branch keeps no such use. Decides it: whether today's checker (and
   `V5`'s R8 and verdicts 53/82, which work out what a call changes) refuses a change through another
   handle to a class object while a view of its field is alive. If not, it is a kind 1 finding inside
   `std`'s own `unsafe` block.
2. **`type_identity` across separately compiled code.** The hash is of the qualified name only
   (`ember_types/src/lib.rs:3067-3192`), and a generic instance's name gets a number when another name
   already holds its spelling (`lib.rs:18604-18626`), so two images (a reload, a binary package) could
   give one name to two layouts and `downcast_ref` would read one as the other. Decides it: whether an
   `AnyError` can cross from one compiled image to another (agent 03, hot reload; agent 02, packages).
3. For agent 02: `CppShared.get() -> ref T` (`ffi.em:452-460`) promises a read-only view of an object
   C++ code may change; `[FFI-33a]`'s check that a non-`@sync` object is not touched from an attached C
   thread runs only in `debug` (`ember_rt.h:1515-1526`).

## Scratch

None needed; nothing under `target/redesign8/04/` but the empty folder.
