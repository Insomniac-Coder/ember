# 03 Today's spec: everything else the redesign keeps

Agent 03 (spec-rest) of the unsafe-features inventory. `SPEC` = `docs/spec-source/Ember_v0.9.10_Hardened_35.md`,
`V5` = `tasks/96/redesign/run4/v5/`, `JOB` = `tasks/96/redesign/JOB.md`. Line numbers are SPEC's unless a file is
named. Nothing was compiled or run; the compiler, runtime and `std` were only read, to see whether a rule is fenced
today.

Families read in full: types and casts `[TYP]`, ranges `[RNG]`, layout `[LAY]`, statics `[STA]`, attributes
`[ATT]`, control flow `[CTL]`, generators `[CORO]`, panics `[PAN]`, destruction `[DRP]`, spans `[SPN]`, the heap
`[HEAP]`, arenas `[ARN]`, allocators `[ALC]`, handles `[HND]`, `Cell` `[CELL]`, disjointness `[DSJ]`, effects and
contracts `[EFF]`, determinism `[DET]`, cost `[COST]`/`[OPT]`, threads `[THR]`, parallel loops `[PAR]`, jobs
`[JOB]`, `SoA` `[SOA]`, SIMD `[SIMD]`, ECS `[ECS]`, errors `[ERR]`, compile time `[CT]`, reflection `[RFL]`,
derives `[DRV]`, serialisation `[SER]`, the standard library's spec `[STD]`, text `[TXT]`, hashing `[HASH]`,
structs/enums/classes `[STR]`/`[ENM]`/`[CLS]`, mangling and monomorphisation `[MNG]`/`[MONO]`, hot reload `[HR]`
(Annex B), GPU `[GPU]` (Annex D), and the principles `[PHIL]`/`[TIER]`. Left to others: `[FFI]`, `[UNS]`, `[RT]`,
`[BLD]`, `[CLI]`, `[VER]` and Annex C's C++ rules (agent 02); `std/src/*.em` code (agent 04); the families V5
replaces (`[OWN]`, `[BRW]`, `[LT]`, `[RC]`, `[WK]`, `[CLO]`, `[OBJ]`, `[EXP]`, `[EXC]` exclusivity, `RefCell`) are
not findings unless the replacement has the same problem.

Eleven findings below (1-11), most harmful first, and one item checked and probably safe (12, kept for the
checker); then what was checked and found fenced or safe; then three things seen in passing that belong to other
duties. Only finding 9 is for the owner.

---

## 1. A `static mut` whose starting value owns heap memory is placed in read-only memory

- **feature**: `[CT-5]`, lines 4030-4042: a static whose value is computed when compiling is "materialised in
  read-only memory" when "its type gives Safe code no way to obtain `ref mut` to the value (no `Mutex`, `RwLock`,
  `Cell`, `RefCell` or `Atomic` on the path to it)"; its heap parts are constant arrays "which nothing grows or
  frees" (compiler: `compiler/ember_codegen_c/src/lib.rs` 12651-12655, `emit_constant_statics`). V5 adds the
  safe `static mut` (`V5/rules.md` 899-901; `V5/vocabulary.md` 134, verdict 8), which gives `ref mut` with no type
  on the path. `[CT-5]`'s test looks only at types, so it still sends such a static to read-only memory.
- **how it breaks safety**: kind 1 (freeing or growing memory the heap never gave out), kind 6 (`realloc`/`free`
  of a non-heap pointer is undefined in C).
  ```ember
  static mut LOG: Array[String] = [String("boot")]

  fn main():
      LOG.push(String("start"))   # grows a buffer that lives in read-only constant data
      LOG[0] = String("hi")       # drops the old String: frees bytes the heap never gave out
      println(LOG[0])
  ```
  When run under `[CT-5]` as written: `push` calls `ember_realloc` on a pointer into constant data, and the
  assignment calls `ember_free` on another; the C library's behaviour is undefined (usually a crash, possibly heap
  corruption). The same holds for any V5 operation that writes a static's own memory without a type on the path
  (a `shares` or `thread_shared` count +1 on an object held by a static, if V5 allows taking one).
- **fenced?** Today yes, by refusal: the compiler rejects every Ember `static mut` ("`static mut` needs `unsafe`,
  which is not in this phase", `compiler/ember_typeck/src/lib.rs` 10259-10265), and `[STA-1]` (1805) keeps it
  for foreign statics, behind `unsafe`. In V5 no: `static mut` is safe and no V5 line says where its value lives.
- **Rust**: `static mut` is `unsafe` to touch; a `static` with interior mutability (`static LOG: Mutex<Vec<String>>
  = Mutex::new(Vec::new())`) cannot hold heap memory made at compile time at all (const heap allocation does not
  exist), so the question never arises.
- **recommendation**: make it safe with a rule, at no run-time cost beyond what `[CT-5]`'s second bullet already
  pays: add "a `static mut`" (and any static whose memory a safe operation writes, such as a count) to the
  second bullet, so its starting value is built on first access into ordinary heap memory (`[STA-3]`). Only a plain
  `static` whose value nothing can change stays in read-only memory.
- **settled?** Yes: verdict 8 made `static mut` safe, verdict 60 requires that it not break a promise; this only
  places it correctly. Not for the owner (no program is written differently).
- **sure?** High that the rule text has the gap (`[CT-5]` predates a safe `static mut`); the crash itself follows
  from `[CT-5]`'s own words.

## 2. Hot reload swaps in a function whose worked-out facts changed, under code that was checked against the old ones

- **feature**: `[HR-3b]`, lines 6272-6282 (a reload at `std.hot.checkpoint()` while the frames below it "run
  their old code on after the swap"; the only thing checked is "the types whose layout the rest of the function
  depends on"), and `[HR-14]`/`[HR-41]`, lines 6339-6361, 6403-6415 (plan compares schemas only; "methods added,
  removed or changed: no instance change"). V5 moves the safety of loops, views and calls onto facts the compiler
  works out from each body ("what a function does", `V5/vocabulary.md` 33-35; R4, R5, R17 in `V5/rules.md`
  930-948) and removes every run-time check that used to catch a mismatch ("Nothing is checked while running",
  `V5/rules.md` 549-550). R17 handles a changed fact at build time ("a change to it is reported as an interface
  change"), but hot reload replaces a body without re-checking the frames still running old code, and (at a
  poll) without rebuilding other packages that call it.
- **how it breaks safety**: kind 5 (code loaded while the program runs) leading to kind 1 (use after free).
  Version 1 of a reloadable program:
  ```ember
  import std.hot

  static mut LOG: Array[String] = []

  fn note():
      println("noted")

  fn main():
      LOG.push(String("a"))
      LOG.push(String("b"))
      while true:
          for line in LOG:
              note()
              hot.checkpoint()
              println(line)
  ```
  Version 2 changes `note`'s body to `LOG.push(String("c"))` and `main`'s loop to `for line in copy of LOG:`, so
  version 2 compiles. At the checkpoint, no type changed, so `[HR-3b]` applies the reload. The old `main` frame
  goes on walking `LOG` by its old pointer and length (`[CTL-3b]`); its next `note()` goes through the thunk to the
  new body, whose `push` may move `LOG`'s buffer; `println(line)` then reads the freed buffer. With version 1
  checked, the loop was legal because `note` did not change `LOG`; nothing checks that fact again. The same
  happens at an ordinary poll when the caller is in a package that is not rebuilt (a host package looping over a
  list a reloadable function can reach).
- **fenced?** No, today or in V5. Hot reload needs no `unsafe` (it is a build mode, `[PRF-2]`, `[CONF-5]`).
  Today's spec at least had the run-time exclusivity checks (`[EXC-1]`) that V5 removes.
- **Rust**: no hot reload in the language. Loading code while running goes through `libloading`, whose
  `Library::new` and `Library::get` are `unsafe fn`, because loaded code can break any promise.
- **recommendation**: make it safe with a rule, no run-time cost: the reload image carries each function's
  worked-out facts (R17 already records them across a package's edge), and plan compares them with the old ones;
  a reload that changes a fact of a function that code not rebuilt was checked against (another package's code,
  or an old frame live below a checkpoint, which `[HR-3b]` already lists per call) is refused, naming the function
  and the changed fact ("`note` now adds to `LOG`, which `main` (line 12, still running old code) walks; restart,
  or reload at a poll where no frame is live"). At a poll with every caller in the rebuilt package, nothing
  changes.
- **settled?** Settled by verdict 60 with R17's own rule (a changed fact is an interface change). Not for the
  owner: ordinary programs are unchanged; some reloads are refused instead of crashing.
- **sure?** High for the checkpoint case (the spec text checks only layouts); medium for the poll case across
  packages (depends on whether a reloadable package's callers are always rebuilt, which `[HR-1]`'s "the unit of
  reload is the package" suggests they are not).

## 3. Hot reload's migration predates V5's kept views, links and `using`

- **feature**: `[HR-15]`, lines 6363-6366: when a migrated object must move, commit rewrites "strong handles in
  live instances and registered containers, `Weak` handles, interior references in registered containers, and
  handles inside tuples, `Box` contents and closure captures". `[HR-15a]` refuses only references "in foreign
  memory, behind a raw pointer". `[HR-2a]`/`[HR-36]`, lines 6258-6263, 6383-6384: prepare allocates every new
  instance in the transaction arena (in the runtime, ordinary heap blocks tracked by address,
  `runtime/ember_rt/src/ember_rt.c` 2784, 11938-12057). V5 adds heap-held links and views the list does not name
  (`sharedref mut T`, `ref T` and `ref mut T` kept in fields, R7, `V5/rules.md` 428-450; `shares`/`weakref` are
  today's handles renamed), and values made from a named memory manager that "keep a pointer to `current`, so
  [they go] back to the right arena when [they grow] or end" (verdicts 24, 47).
- **how it breaks safety**: kind 1. (a) With `class Watcher: target: sharedref mut Player` (wherever R7 lets a
  heap object hold one) and a reload that adds a field to `Player` so it no longer fits, the player moves; the
  watcher's link is not on `[HR-15]`'s list, so it still names the old block, which reclaim frees; the next
  `w.target.hp -= 1` writes freed memory. (b) With verdict 24's `boss = Enemy(hp=500, mesh=owns Mesh(points=[0.0,
  1.0]) using current)` and a reload that adds a field to `Mesh`, prepare builds the new mesh in heap memory;
  when `boss` ends, the mesh is handed to `current.deallocate`, a block `current` never gave out (a bump arena
  ignores it and leaks it; a free-list arena corrupts its list).
- **fenced?** No (hot reload is not behind `unsafe`); the gap exists only with V5's additions.
- **Rust**: no counterpart (see finding 2).
- **recommendation**: make it safe with a rule, no run-time cost: (a) add every V5 link and kept view to
  `[HR-15]`'s list (they are typed fields of schemas, so the runtime can enumerate them), or refuse in plan any
  move of an object a kept view or link may name; (b) build a migrated value made with `using` from its own memory
  manager (its header names it), or refuse moving one in plan.
- **settled?** Settled by verdicts 24, 47 and 60 (the redesign keeps hot reload: `V5/vocabulary.md` 192, 213-235).
  Not for the owner.
- **sure?** Medium. Whether R7 lets a heap object hold a `sharedref mut` field without `from` is for agent 01 to
  confirm; part (b) is certain if a value made with `using` can live in a reloadable package.

## 4. An atomic owner count checked "before each +1" can be pushed past its maximum by two threads

- **feature**: `[THR-14]`, lines 3517-3523: "`fetch_add`/`fetch_sub` panic when the result overflows ... the
  overflowed value has been stored, and another thread may observe it before the process aborts". V5's counts
  (`V5/rules.md` 555-558, verdict 59): "before each +1 of an owner count (a `shares`, `shares mut` or
  `thread_shared` owner made) it is compared with its maximum". A compare followed by an add is two steps; for an
  atomic count two threads can both pass the compare.
- **how it breaks safety**: kind 1 (an object freed while owners still point at it).
  ```ember
  import std.thread

  class Config:
      level: int = 1

  fn main():
      c = thread_shared Config()
      keep: Array[thread_shared Config] = []
      for i in 0..4294967293:          # the count is now 4,294,967,294
          keep.push(thread_shared c)
      with scope = thread.scope():
          scope.spawn(fn() thread_shared c: println(c.level))
          scope.spawn(fn() thread_shared c: println(c.level))
      d = thread_shared c
      println(keep[0].level)
  ```
  The two tasks each make an owner at the same moment: both read 4,294,967,294, both pass the compare, both add,
  and the 32-bit count rolls over to 0. When one task's owner ends and `d` is made and ends, the count reaches 0
  and the `Config` is freed while four billion owners in `keep` still point at it; `keep[0].level` reads freed
  memory. (It needs about 32 GB; with verdict 96's 8-bit test counter the same race needs 254 owners.)
- **fenced?** Not applicable today (today's counts are `[RC]`'s); in V5 nothing fences it, and the rule's wording
  invites the racy order.
- **Rust**: `Arc::clone` does one `fetch_add` and then aborts if the old value exceeds `isize::MAX`; the room
  above `isize::MAX` is larger than any number of threads that could race past it.
- **recommendation**: make it safe with verdict 59's own check done in Rust's order: one `fetch_add`, then stop the
  program if the old value is above a limit set at half the range (2,147,483,647 for the 32-bit count, 127 for
  the 8-bit test counter), so no number of racing threads can reach the roll-over. Same cost as written (one add
  and one compare), no extra memory. The weak count the same. For `shares` (one thread) the order does not
  matter.
- **settled?** Settled by verdict 59 (this is how to apply it). Not for the owner; only the limit in the message
  changes.
- **sure?** High that "compare, then add" races for an atomic count; it depends on how the implementation is
  written, which the rule's wording does not pin down. (Overlaps agent 01's R9.)

## 5. A draw call's counts are not checked against the buffers it reads

- **feature**: Annex D, `[GPU-5]` and the example at lines 6690-6697: `cmd.bind_vertex_buffer(0, vbo)`,
  `cmd.draw_indexed(index_count)`, all safe; only `cmd.native()` needs `unsafe` (6708). `[GPU-1]` checks handle
  generations and `[GPU-4]` writes to buffers in flight; no rule compares a draw's vertex or index range with the
  sizes of the bound buffers. (`std.gpu` is not built yet: there is no `std/src/gpu.em`.)
- **how it breaks safety**: kind 5 (memory the rules cannot see: the GPU's), leading to out-of-bounds reads.
  ```ember
  fn draw(ref mut frame: Frame, pipeline: PipelineHandle, vbo: BufferHandle):
      with cmd = frame.command_list():
          cmd.begin_render_pass(RenderPassBegin(target=None))
          cmd.bind_pipeline(pipeline)
          cmd.bind_vertex_buffer(0, vbo)    # vbo holds 3 vertices
          cmd.draw_indexed(1000000)         # reads far past both buffers
          cmd.end_render_pass()
  ```
  Under Vulkan without `robustBufferAccess` (and D3D12/Metal alike) the GPU reads past the buffers: undefined
  behaviour, from garbage on screen to a device loss or a read of another process's memory on some drivers.
- **fenced?** No; nothing in Annex D needs `unsafe` except `cmd.native()`.
- **Rust**: `wgpu` (the safe API) checks every draw's ranges against the bound buffers and refuses one that runs
  past them; `ash` (raw Vulkan) makes `cmd_draw_indexed` an `unsafe fn`.
- **recommendation**: make it safe with a rule: the command list checks each draw's and dispatch's ranges against
  the bound buffers' sizes before recording it (one compare per draw, as `wgpu`), panicking with the numbers; the
  device asks for robust buffer access where the API has it, for what shaders index themselves. Shader code is
  foreign code (`[GPU-11]`) and stays the device's responsibility, as `[PHIL-11]` says for foreign code.
- **settled?** Settled by verdict 60. Not for the owner (nothing ordinary changes; a wrong draw panics).
- **sure?** High for the missing rule; the harm depends on the GPU API.

## 6. `alloc_nodrop` ends a value without its clean-up: verdict 58's `mem.forget` by another name

- **feature**: `[ARN-2]`, lines 2936-2937: "Allocating a type that needs drop is `E3090`, unless through
  `alloc_nodrop`, which acknowledges that its `drop` will never run." V5 keeps `Arena` unchanged
  (`V5/vocabulary.md` 321-323: "same ... as today").
- **how it breaks safety**: kind 3 (leak), and the count left too high that verdict 58 removed `mem.forget` for.
  ```ember
  from std.arena import Arena

  class Player:
      hp: int = 10

  fn main():
      frame = Arena.with_capacity(1024)
      p = Player()
      for i in 0..1000000:
          frame.alloc_nodrop(shares p)           # one more owner of p, never ended
          frame.alloc_nodrop(String("lost"))     # a heap buffer never freed
          frame.reset()
      println(p.hp)
  ```
  The arena's own bytes are reused every round, but each round leaves `p`'s count one higher and one string buffer
  unfreed: the heap grows by a million buffers, and `p` is never freed when `main` ends. V5's own claim that
  "every value's `drop` runs at its end, since `mem.forget` is gone" (`V5/vocabulary.md` 193, the `@must_drop`
  row) is false while this exists.
- **fenced?** No. Today the compiler gives `E3090` only for `alloc` (`compiler/ember_typeck/src/lib.rs`
  54979-54992: `if method == "alloc" && self.types.needs_drop(...)`), and `alloc_nodrop` passes with no `unsafe`
  check. V5: no line fences it.
- **Rust**: `bumpalo::Bump::alloc` never runs `Drop` and is safe (Rust counts leaks as safe); `typed_arena::Arena`
  runs every value's `Drop` when the arena ends.
- **recommendation**: fence it: `alloc_nodrop` only inside `unsafe:` (E3100: "`alloc_nodrop` stops Ember from
  running the clean-up of the `String`, so its memory is never freed; write it inside `unsafe:`, or let the arena
  clean it up: `frame.alloc(...)`"). Optionally make the safe way exist: `alloc` of a type that needs drop records
  the value in the arena's clean-up list, run at `reset` and at the arena's end, as `typed_arena` does (one list
  entry per such value; nothing for types that need no drop).
- **settled?** Settled by verdicts 58 and 60 ("REMOVE FORGET"; "only possible in an unsafe block"). Not for the
  owner.
- **sure?** High (compiler line quoted).

## 7. `MaybeUninit.write` can overwrite or abandon a value that needs drop

- **feature**: `[ARN-8]`, lines 2969-2973: "`MaybeUninit[T].uninit()` is safe; `write(mut self, owned v) -> ref
  mut T` stores without dropping previous bytes ... dropping it never drops a `T`"; `write_at(i, v)` on
  `MutSpan[MaybeUninit[T]]` the same. V5 keeps it (`V5/vocabulary.md` 326: "as today (`unsafe`)", while today
  only `assume_init` is `unsafe`).
- **how it breaks safety**: kind 3 (leak).
  ```ember
  from std.mem import MaybeUninit

  fn main():
      for i in 0..1000000:
          slot = MaybeUninit[String].uninit()
          slot.write(String("lost"))     # never dropped: a MaybeUninit's end drops no String
  ```
  A million string buffers are never freed. With a `shares` value written in, its count stays too high, as in
  finding 6.
- **fenced?** No today: `write` is checked with no `unsafe` test, only `assume_init` has one
  (`compiler/ember_typeck/src/lib.rs` 49934-49946 against 49948-49958). V5: unclear ("as today (`unsafe`)").
- **Rust**: `MaybeUninit::write` is safe; writing twice or never calling `assume_init_drop` leaks, which Rust
  counts as safe.
- **recommendation**: fence it: `write` and `write_at` of a `T` that needs drop only inside `unsafe:` (E3100).
  Nothing ordinary changes: a `MaybeUninit` is only useful together with `assume_init`, which is already behind
  `unsafe`; for a `T` that needs no drop, `write` stays safe.
- **settled?** Settled by verdicts 58 and 60. Not for the owner.
- **sure?** High (compiler lines quoted).

## 8. A channel that holds one of its own ends keeps itself alive

- **feature**: `[THR-15]`, lines 3524-3529: `channel[T]` and `channel_unbounded[T]`; `Sender` is `Send`, `Sync`
  and `Clone`, `Receiver` is `Send`, so either can be sent as a message. In `std/src/sync.em` (167-261) both ends
  hold one `SyncShared[ChannelState[T]]`; the queue lives as long as that count, and `Receiver.drop` (255-261)
  marks the receiver gone but does not empty the queue.
- **how it breaks safety**: kind 3 (a loop of owners that keep each other alive).
  ```ember
  from std.sync import channel, Receiver

  struct Msg:
      back: Option[Receiver[Msg]]

  fn main():
      for i in 0..1000000:
          tx, rx = channel[Msg](capacity=1)
          tx.send(Msg(back=Some(rx))).expect("receiver is gone")
          # tx ends here; the queue holds rx, rx holds the queue: nothing reaches it, nothing frees it
  ```
  A million channel states are never freed. A `Sender` sent into its own channel (a "reply to me" message whose
  receiver ends first) leaks the same way, because the receiver's end does not empty the queue.
- **fenced?** No, today or in V5.
- **Rust**: `std::sync::mpsc` leaks the same program (Rust counts leaks as safe); its channel does drop queued
  messages when the receiver disconnects, which breaks the `Sender` case but not the `Receiver` one.
- **recommendation**: make it safe with two rules: (a) the receiver's end drops every value still queued (the
  work happens anyway when the state is freed; queued values end earlier, when the receiver ends, rather than when
  the last sender ends); (b) a channel whose element type can hold, through what it owns, a `Receiver` of that same
  channel type is refused when compiling ("a `Msg` can hold the `Receiver[Msg]` of its own channel, which would
  keep the channel alive for ever; send the receiver over a channel of another type"). The compiler knows every
  type a `dyn` value can be (R5), so (b) is exact. No run-time cost.
- **settled?** Verdict 60 sets the goal (leaks only inside `unsafe`). Not for the owner as far as (b) goes (it
  refuses only self-holding channels); (a) moves when queued values are dropped, which a program can see only
  through a `drop` that prints; the checker may raise it with finding 9.
- **sure?** High for the leak (code read); the Rust comparison is from memory of `std`'s channel, medium.

## 9. The spec still lists a loop of shared owners as an accepted leak

- **feature**: `[PHIL-11]`, lines 224-228: "What remains possible, and is therefore not a defect of the
  guarantee: ... a reference cycle leaking (`[WK-1]`)". V5 replaces counted class handles with `shares` /
  `shares mut` / `thread_shared` owners (R9), which can form the same loops; nothing in `V5/rules.md` refuses one
  (its "What stays refused", section 3, has no such row).
- **how it breaks safety**: kind 3 (the brief's own example: "a loop of shared owners that keep each other
  alive").
  ```ember
  class Node:
      next: Option[shares mut Node] = None

  fn main():
      for i in 0..1000000:
          a = Node()
          b = Node()
          a.next = Some(shares mut b)
          b.next = Some(shares mut a)
          # a and b end; each still has one owner, the other's `next`: neither is ever freed
  ```
- **fenced?** No, today (by design, `[PHIL-11]`) or in V5.
- **Rust**: `Rc`/`Arc` loops leak, and Rust counts leaks as safe; `Weak` is the way out.
- **recommendation**: make it safe with a rule, at compile time: a type whose owning links (`shares`, `shares mut`,
  `thread_shared`, and channel ends, finding 8) can lead back to itself is refused unless one link on the loop is a
  `weakref` ("`Node.next` can lead back to its own `Node` through owners only, which can keep both alive for ever;
  write `next: Option[weakref Node]`, and keep the nodes in a list that owns them"). No run-time cost. And remove
  "a reference cycle leaking" from `[PHIL-11]`.
- **settled?** Verdict 60 sets the goal, but no verdict settles refusing such types, and it changes how an
  ordinary program is written (a graph of nodes that share each other). **For the owner.**
- **sure?** High for the leak; agent 05's loop-of-owners attack is the same finding and may hold the better
  program.

## 10. `@reinit_on_reload` drops a static's old value while views may still point into it

- **feature**: `[HR-17a]`, lines 6392-6394: "`@reinit_on_reload` ... re-runs the initialiser and discards the
  old value". Today `[LT-3]` (2399) gives views of `static` items the static region, and `[TYP-15]` (1125-1132)
  lets a class field hold any view whose region is static. V5 narrows a kept view `from static` to "text written
  in the program" (`V5/rules.md` 434) and keeps `@reinit_on_reload` (`V5/vocabulary.md` 214).
- **how it breaks safety**: kind 1 (a view that outlives what it points into). Today's spelling:
  ```ember
  @reinit_on_reload
  static NAMES: Array[String] = [String("ann")]

  class Badge:
      name: str = "?"

  fn make() -> Badge:
      b = Badge()
      b.name = NAMES[0]        # a view of a static: static region, so a class field may hold it
      return b
  ```
  Editing `NAMES`' initialiser and reloading drops the old array and its strings; every live `Badge` still points
  into the freed text.
- **fenced?** No today. In V5 probably closed for class fields (only literal text may be kept `from static`), but
  no line says whether a `static` may itself hold a view of another static's heap parts, which would reopen it.
- **Rust**: no counterpart.
- **recommendation**: keep V5's narrowing and state it for statics too (a static's type holds views only of text
  written in the program); and have plan refuse `@reinit_on_reload` on a static that any kept view or link may
  point into. No run-time cost.
- **settled?** Settled by verdict 60. Not for the owner.
- **sure?** High for today's spec; medium for V5 (depends on the statics rule above).

## 11. A suspended generator across a reload (unsure)

- **feature**: `[CORO-3]`/`[CORO-4]` (2174-2181): each `gen fn` has its own frame type, laid out by its body,
  resumed by a jump on its state; `[HR-19]` (6417-6419): `reload = "bodies"` "admits only function-body changes
  and changes that need no instance migration; it needs no schemas"; `[HR-6a]` (6302-6303): method tables and
  code pointers are thunks, so a stored generator's `next` reaches the new code.
- **how it breaks safety**: kind 1 (a frame of one layout read as another). With `g = Door().swing_open()` kept
  in a field (a `Generator[Wait]` in `std.coroutine`'s `Scheduler`, `[CORO-11]`), editing `swing_open`'s body
  (a new local kept across a `yield`) changes its frame layout and state numbering; under `"bodies"` no schema is
  compared, and the next `g.next()` runs the new state machine on the old frame.
- **fenced?** Not by any spec line. The compiler's reload code refuses values it cannot reach
  (`compiler/ember_codegen_c/src/reload.rs` 696-760) but has no mention of generator frames.
- **Rust**: no counterpart.
- **recommendation**: treat a `gen fn`'s frame as a type with a schema: a body change that changes its frame
  layout or its states is a schema change, refused while any frame of it is live (in every reload tier), or the
  old resume code is kept for old frames.
- **settled?** Settled by verdict 60 if confirmed. Not for the owner.
- **sure?** Low to medium: what decides it is whether a generator's resume function is reached through a thunk and
  whether `"bodies"` mode checks frame layouts; neither is stated.

## 12. `[HASH-3]`'s safety argument names `RefCell`, which V5 removes (checked: probably safe)

- **feature**: `[HASH-3]`, lines 1424-1429: a `hash` or `eq` "that reaches the map again can do so only through a
  `RefCell`, whose borrow panics". V5 removes `RefCell` (`V5/vocabulary.md` 251) and adds `static mut` and
  `shares mut`, so a key's `hash` can reach the map it is being inserted into by name:
  ```ember
  static mut SEEN: Map[Key, int] = {}

  struct Key:
      n: int

  extend Key implements Hash:
      [H: Hasher]
      fn hash(ref mut h: H):
          SEEN = {}                  # replaces the map being inserted into
          h.write_i64(this.n)

  fn main():
      SEEN.insert(Key(n=1), 1)
  ```
- **fenced?** V5's call check (R4, `V5/rules.md` 316-338) should refuse `SEEN.insert(...)` with E3031: `insert`
  uses `this` after a call (`hash`) that may end what `this` views, and `this` is `SEEN`, as in verdict 54's
  `ENEMIES[0].hit()`. So this is not counted as a finding; listed so the checker can confirm, and so `[HASH-3]`'s
  text is updated to name R4 instead of `RefCell`.
- **Rust**: the key's `Hash` cannot reach the map mutably without `RefCell` or `unsafe`; `RefCell` panics.
- **recommendation**: none beyond the text; confirm E3031 covers a method's `this` reached through a type
  parameter's method (`hash`) per instance (R17's last sentence).
- **sure?** Medium.

---

## Checked and found fenced or safe (not findings)

- `[TYP-8]` `@overflow(wrap)`, `#! module overflow(wrap)`: wrapping is defined, two's complement, "never undefined
  behaviour in the generated code" (959); indices made from wrapped values are still bounds-checked. Stricter than
  Rust, whose release builds wrap by default.
- `[TYP-9]` `@fastmath`: the build passes every relaxation but the finite-only one, because "under it clang makes a
  NaN argument undefined behaviour, and a `@fastmath` function called with one could then skip a bounds check"
  (`compiler/ember_build/src/lib.rs` 1502-1517, 1480-1499; the LLVM backend drops `nnan`/`ninf`,
  `compiler/ember_codegen_llvm/src/func.rs` 2086-2087). Plain MSVC gets `/fp:fast` (1467), which may mishandle NaN
  but has no "undefined" meaning in MSVC. Safe.
- `[TYP-6]` float to integer `as` saturates, NaN to 0 (runtime helper, `runtime/ember_rt/include/ember_rt.h` 723,
  811); `[TYP-10]` shifts out of range panic; `[TYP-28]`/`[TYP-30]` division and powers checked.
- `[TYP-2]`, `[TYP-3]`, `[RNG-9]`: invalid `bool`, `char`, range values only in `unsafe` (`T.new_unchecked`).
  `[RNG-10]`, `[SER-2]`, `[HR-14]`: every other way to make a range value checks it.
- `[TYP-43]`, `[ARN-11]`: `Send`, `Sync`, `Zeroable` and `Allocator` by hand only with `unsafe extend`;
  `@derive(Zeroable)` is proved field by field.
- `[LAY-2]` `@packed`: a reference to a field is E2170, including the borrows a parameter mode or a receiver makes
  (`compiler/ember_analysis/src/packed.rs` 1-5, 16-24); V5's plain-parameter views must keep going through it.
- `[SIMD-9]`: unchecked loads, stores, gathers and scatters are `unsafe fn`; the checked ones use unaligned
  intrinsics (`runtime/ember_rt/include/ember_simd.h` 73-78). `[SIMD-7]` grouped overflow checks write only
  memory the thread owns and abort before anyone reads it.
- `[HND-1]`, `[SPN-2]`, `[DSJ-7]`, `[EFF-7]`: the unchecked and assuming forms (`get_unchecked`,
  `assume_disjoint`, `@assume_noalloc`) are `unsafe` only. `[HND-3]` retires a slot whose generation runs out.
- `[ARN-1]`, `[ARN-6]`, `[ARN-7]`, `[JOB-5]`, `[GPU-3]`: arena resets and scopes take the arena changeably, so no
  view survives them.
- `[STA-3]`: lazy statics run once, a self-reaching initialiser panics. `[THR-16]`, `[STD-24]`: `process.exit`
  uses `_Exit`, so no `atexit` handler runs beside other threads (`runtime/ember_rt/src/ember_rt.c` 10352-10357).
  No `env.set_var` exists (`std/src/env.em` has only `var`), so the `setenv`-beside-`getenv` race cannot arise.
- `[PAN-1]`, `[DRP-4]`, `[THR-3]`, `[HASH-3]`, `[ARN-10]`: every panic aborts, so no half-changed map, lock or
  span is ever seen; this is also what keeps scoped tasks sound without unwinding (`[THR-5]`).
- `[PAR-2]`: a `@parallel` body may write only `MutSpan`s and `SoA` columns at one offset; a write to anything else
  (a V5 `static mut` included) is E7010.
- `[ECS-2]`: a component's identity is a 64-bit FNV-1a hash of its name, but storages are fetched with a checked
  downcast (`std/src/ecs.em` 264-278, `cast found as AosStorage[T]`), so a collision stops the program rather than
  confusing types (assuming the downcast compares the exact instance).
- `[MNG-1]`, `[MONO-1]`: link-time de-duplication by a 64-bit BLAKE3 prefix; an accidental collision across
  separately built packages is not checked, but its odds are negligible and one version of each package is
  resolved (5184).
- `[CLS-2]`, `[CLS-11]`, `[STR-6]`: every field holds a value before any method runs, a virtual one from a base
  `init` included.
- `[TIER-1]`, `[ATT-6]`, `[EFF-23]`, `[PHIL-12]`, `[PHIL-13]`: no attribute, profile or manifest key adds a way
  around `unsafe` or turns a check off (agent 02 covers the flags themselves).

## Seen in passing, for other duties

- `[FFI-39d]` (6584-6589): re-entry from C++ into an Ember object derived from a C++ class relied on the run-time
  exclusivity checks (`[EXC-1]`) that V5 removes; whether R12's "every foreign call may run the callbacks C keeps"
  covers a trampoline's overrides is not stated. For agent 02.
- A `thread_shared` object's last owner may end on another thread, running its `drop` there (`[THR-8]`, ODR-052);
  a `drop` that changes a `static mut` (verdict 57 allows a `drop` to change things outside its object) would then
  touch a main-thread module variable from another thread, which E7012 must catch. For agents 01 and 05.
- `[CORO-12]`: a `gen fn` method of a class "retains the handle" today; with V5's one-owner objects, what keeps
  `this` alive in a frame stored in `std.coroutine`'s `Scheduler` needs a rule. For agents 01 and 05.
