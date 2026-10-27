# 01: the new design (`V5`) against verdict 60 (agent 01, design)

Verdict 60: "Anything that can break a safety promise is possible only inside an `unsafe` block." This file
checks the new design itself: `V5/rules.md` (read in full, 1219 lines) and `V5/vocabulary.md` (read in full,
595 lines), with the answers and tests a rule points to (`V5/area-E.md` AN-71, AN-74, AN-78; `V5/safety/recheck.md`;
the R3, R12, R15 and R16 tests), and today's spec (`SPEC`) where a rule says "as today". Line numbers are of
those files. `V5` = `tasks/96/redesign/run4/v5/`. Nothing was compiled or run.

How sure: **sure** = the rule text, read word for word, allows the program; **likely** = the rule text allows
it, but another answer may already mean to cover it; **unsure** = it depends on a fact the design does not
state (said each time).

## Summary

| # | feature | kind (brief) | outside `unsafe` today / in V5 | how sure | recommendation | for the owner |
|---|---|---|---|---|---|---|
| 1 | C calling back into Ember outside the kept-callback list: `@export` functions, Ember overrides of C++ virtuals | 1, 6 | yes / yes | sure | count them at every foreign call, as kept callbacks are (verdict 31) | no |
| 2 | hot reload against the compile-time facts | 1, 2, 5, 6 | yes / yes | sure | reload re-runs the facts' checks and refuses an edit that would now be refused | no |
| 3 | an arena's instance reached through a `sharedref mut` link: `reset()` while values made in it live | 1 | in part (the `Allocator` implementation is `unsafe extend`) | likely | while a value made in an instance lives, only `allocate`/`reallocate`/`deallocate` change the instance | no |
| 4 | a loop of `shares` owners (and of `thread_shared` owners, function values' state, channels) | 3 | yes / yes | sure | make the cycle a refusal unless a link in it is `weakref`; or keep Rust's choice and say leaks are allowed | **yes** |
| 5 | the trampoline's 16-bit generation rolling over | 4 | yes / yes | unsure (slot reuse not stated) | retire a slot whose generation is used up, as `[HND-3]` does | no |
| 6 | the weak count of a `thread_shared` object not said to be atomic | 2 (then 1) | yes / yes | likely (wording) | say both counts of a `thread_shared` object are atomic | no |
| 7 | "static re-entry" of a lock whose object is not known to be the held one | 2, 4 | yes / yes | unsure (wording) | enter past the lock only when the compiler proves it is the same object; otherwise E7018 | no |
| 8 | a scoped task reading a place the parent (or another task) writes, no reduction | 2 | yes / yes | likely (wording) | write area E's rule into R15 and E7007's meaning | no |
| 9 | `static mut` without `unsafe` | 1, 2 | fenced today / not fenced in V5 | sure | keep (verdict 8); close 1 and 2, which are its holes | no |
| 10 | C writing into Ember memory it keeps (`Retained`, a `retained` mutable pointer) during a later foreign call | 5, 6 | in part / in part | unsure | a foreign call counts as possibly changing every object C keeps a changeable pointer to | no |
| 11 | `thread_shared fn()` "may hold ... copies": a copy of a struct holding `shares` links | 2 | yes / yes | likely (wording) | R12's sentence: copies with nothing one-thread in them | no |
| 12 | `@parallel`: R15 and E7010 name only two chunks writing one place | 2 | no (`[PAR-2]`) / wording | likely (wording) | keep `[PAR-2]`'s "reads included" and E7011 in V5 | no |

Twelve entries; 1 to 4 are the ones that matter most. Entry 9 is listed because V5 removes a fence today has.

---

## 1. C calling back into Ember outside the kept-callback list

- **feature**: R12's foreign-call rule (`V5/rules.md` 745-756): "a call of a foreign function counts as
  possibly running every callback C keeps (those the program hands over through a `callback(retained)` or
  `callback(once)` parameter ...)", and section 0's "unseen code" (86-89). Two other ways C runs Ember code
  are not in that list: `@export` functions (`SPEC` `[FFI-26]` 4945, `[FFI-33]` 4964, `[FFI-33c]` 4981),
  which C may call during any foreign call on a thread their contract allows; and Ember overrides of a C++
  class's virtual methods (`SPEC` `[FFI-39d]` 6584: "Re-entrant calls from C++ into the same Ember object are
  expected"). `V5/rules.md` never mentions `@export` or C++ overrides (searched: no match).
- **how it breaks safety**: kind 1 (use after free) and kind 6 (`restrict` broken on a `ref mut` parameter).
  Today these re-entries were caught by run-time checks (`[FFI-39d]`: "an override that begins a conflicting
  access panics (`[EXC-1]`)"), and a `static mut` needed `unsafe` (`[UNS-1]` 3023). V5 removes both: "Nothing
  is checked while running: no access words, no borrow flags" (R9, 549-550; section 2, 1023), and `static mut`
  needs no `unsafe` (vocabulary line 134). The counting rule that replaces them misses these callers.

  ```ember
  #! threads main

  static mut LOG: Array[String] = []

  @export("on_event")
  extern "C" fn on_event():
      LOG.clear()

  unsafe extern "C":
      safe fn poll_events()          # C: calls on_event() when an event is queued

  fn main():
      LOG.push(String("start"))
      LOG.push(String("tick"))
      for line in LOG:               # walks LOG
          poll_events()              # C runs on_event(): LOG.clear() frees both strings and the buffer
          println(line)              # reads freed text
  ```

  It compiles under V5: no callback is kept, so "With no kept callback ... a foreign call changes nothing"
  (756); `on_event` is a `threads(main)` export, which `[FFI-33c]` lets use thread-confined state, and E7012
  applies only to `threads=any|creator` exports (area E 1416-1420). When run, `println(line)` reads freed
  memory. With a `ref mut c: Counter` parameter alive across `poll_events()` and an export that changes the same
  counter through a module variable, the parameter's noalias promise (R4, 346-348) is false: undefined C.
  The C++ form is the same with an Ember `override fn` of a C++ class, called by C++ inside any C++ call.
  (A third possible way, a capture-free Ember function given to a C function-pointer parameter that has no
  `callback(...)` word, `[FFI-21]` 4894, depends on whether `[FFI-10]` lets such a declaration be `safe fn`:
  unsure.)
- **fenced?**: no, in V5. `@export` needs no `unsafe`; `poll_events` is `safe fn`, whose assertion
  (`[FFI-2]` 4610-4616: "a call within its contract cannot break memory safety") is about its own pointers, not
  a promise never to call an export. Today it is caught while running (`[FFI-39d]`, `[EXC-1]`) and the
  `static mut` access is `unsafe` (`[UNS-1]`).
- **Rust**: `#[no_mangle] pub extern "C" fn` is safe to write, but a global it changes is either `static mut`
  (every access `unsafe`; Rust 2024 refuses references to it) or a `Mutex` / `thread_local!` `RefCell`, whose
  re-entrant use deadlocks or panics (`BorrowMutError`) while running: never undefined behaviour in safe Rust.
- **recommendation**: make the rule complete, at no cost while running: the callbacks a foreign call may run
  are every function value C keeps (as now), every `@export` (and `@export_table`) function whose thread
  contract allows the calling thread, every Ember override of a C++ virtual, and every Ember function whose
  address C was given as an `extern "C" fn`. A foreign call not marked `@ffi(no_callbacks)` makes their
  changes (R5, R8, R4's call check). The message names the export: "`poll_events()` (line 14) is a C
  function, which may call the export `on_event` (line 5), and `on_event` clears `LOG`, which this loop walks
  (line 13); call `poll_events()` before or after the loop: move line 14 below the loop; if `poll_events`
  never calls back into Ember, say so on its declaration: `@ffi(no_callbacks) safe fn poll_events()`".
- **settled?**: verdict 31 ("A C call counts as possibly running the callbacks C keeps"): this is that verdict
  applied to every way C keeps Ember code. Not for the owner.

## 2. Hot reload against the compile-time facts

- **feature**: Annex B's hot reload (`SPEC` `[HR-3]` 6264, `[HR-3b]` 6272-6282, `[HR-5]` 6298, `[HR-17a]`
  6392). V5 moves every borrow check to compile time and keeps the facts per function: what it changes, reads
  and ends, which view parameters it uses after a change (E3031), which locks it takes, parent-link proofs,
  thread facts (R5 355-386, R11 607-624, R17 930-948). Nothing in `V5/rules.md` mentions reload (searched: no
  match), and the spec's reload plan checks only type schemas: "Regions (`[LT-30]`) are compile-time facts and
  play no part in schemas or migration" (`SPEC` 6395).
- **how it breaks safety**: kinds 1, 2, 5, 6. Code loaded while the program runs changes a fact that compiled
  code relies on.

  ```ember
  static mut LOG: Array[String] = []

  fn note():
      println("note")              # edited while the program runs to: LOG.clear()

  fn main():
      LOG.push(String("a"))
      LOG.push(String("b"))
      for line in LOG:
          std.hot.checkpoint()     # the reload is applied here; main's frame keeps running its old code
          note()                   # a call through note's thunk: the new body clears LOG
          println(line)            # reads freed text
  ```

  Compiled, the loop is accepted (`note` changes nothing). At the checkpoint `[HR-3b]` lets the frames below
  run on with their old code and checks only "the types whose layout the rest of the function depends on";
  `note()` then goes through its thunk to the new body (`[HR-5]`), which clears `LOG` while the old loop walks
  it. Other facts break the same way: a parent-link read compiled with no header test (R11 609-624) in an
  unchanged function, after an edit moves a child without resetting its link (the proof's condition 1 no
  longer holds, the read lands in a freed parent); a thread function edited to touch a `static mut` (E7012 was
  decided on the old body: a race); a `@reinit_on_reload` static (`[HR-17a]` "discards the old value") while a
  kept view in another static or object points into it.
- **fenced?**: no. A reloadable build and `std.hot.checkpoint()` need no `unsafe`; V5 has no rule for it.
- **Rust**: no hot reload in the language; loading code while running goes through `unsafe`
  (`libloading::Library::new` is an `unsafe fn`, as are the crates built on it).
- **recommendation**: the reload's plan step (`[HR-2a]`, which runs no user code) compares each changed
  function's recorded facts (R17's interface-file facts) with the old image's and re-runs the compile-time
  checks of every function that relies on them, the frames below a checkpoint included; a reload after which a
  line would be refused is refused (E9030, `[HR-18]`), the report naming the line and the chain, as a compile
  error would ("`note()` (line 11) now clears `LOG` while `main`'s loop walks it (line 9); restart to apply
  this edit"). `@reinit_on_reload` is refused on a static that a kept view may point into. Cost: plan time
  only; nothing while running. Some edits then need a restart; only edits that would corrupt memory.
- **settled?**: no verdict. Not for the owner: it refuses only reloads that would break a safety promise and
  changes no program.

## 3. An arena's instance reached through a `sharedref mut` link

- **feature**: R3, `V5/rules.md` 226-229: "A value whose manager may have several instances keeps its
  instance's address ... That address is a kept `sharedref mut` link to the instance (R7): the program may use
  the instance while values made in it live". R7 (443-444): through `sharedref mut`, "On one thread any of them
  may change it at any time". Section 0 (48): "Writing a number or a `bool` is not a reshape."
- **how it breaks safety**: kind 1. An arena's `reset()` rewinds a number, so by the rules it ends nothing,
  and it may run through the owner's name while values made in the arena live:

  ```ember
  from std.mem import Allocator, Layout, Global

  struct FrameArena:
      base: *mut u8
      size: usize
      used: usize = 0

      fn reset():
          this.used = 0                        # a number written: not a reshape (section 0)

  # SAFETY: each block is cut from `base` once, and is not given out again while it is in use.
  unsafe extend FrameArena implements Allocator:
      fn allocate(layout: Layout) -> *mut u8:
          start = (this.used + layout.align - 1) // layout.align * layout.align
          if start + layout.size > this.size:
              return null
          this.used = start + layout.size
          # SAFETY: start + layout.size <= size, inside the block `base` points to.
          unsafe:
              return this.base.add(start)
      fn deallocate(p: *mut u8, layout: Layout):
          pass
      fn reallocate(p: *mut u8, old: Layout, new_size: usize) -> *mut u8:
          return null

  fn main():
      g = Global()
      frame = FrameArena(base=g.allocate(Layout(size=4096, align=16)), size=4096)
      names = [String("ann")] using frame       # the list's buffer comes from `frame`
      frame.reset()                              # allowed as written: R3 226-229, R7 443-444
      tags = [String("bob")] using frame         # gets the same bytes as `names`'s buffer
      println(names[0])                          # prints "bob": `names` and `tags` share one string
  ```

  When `tags` and `names` end, the string `"bob"` is freed twice. The arena author vouched for `allocate`
  ("not given out again while it is in use"), but `reset` is ordinary safe code, and V5 gives the author no
  way to make it safe: today `reset()` takes `mut self`, "so no view survives them" (`SPEC` `[ARN-1]` 2933-2935,
  `[ARN-7]` 2967), and Rust's `Bump::reset(&mut self)` cannot run while a `Box<T, &Bump>` borrows it. V5's
  own test reads the link as exclusive ("`current` is read only after `lvl`'s end, so no view of it is in use
  then", `conformance/R3/run_a_mesh_made_using_an_arena_takes_its_block_from_that_arena.em` line 9), which the
  rule does not say.
- **fenced?**: in part. Implementing `Allocator` is `unsafe extend` (`SPEC` 1269, `[ALC-2]` 2989,
  `std/src/mem.em` 114-121), but the method that breaks the promise is outside it, and R3 is what lets it run.
- **Rust**: `bumpalo::Bump::reset(&mut self)`; while a `Box<T, &Bump>` or `Vec<T, &Bump>` lives, `reset` is
  refused (E0502).
- **recommendation**: make it a rule, at no cost while running: while a value made in an instance lives, the
  instance is changed only through the `Allocator` methods that value's growth and end call; any other change
  through any name is refused (E3021: "`frame.reset()` (line 26) changes the arena `names` was made in (line
  25), and `names` is used on line 28; reset after `names`'s last use: move line 26 below line 28"). Reading
  the instance stays allowed. Also say in `[ALC-2]`'s promise that a block stays valid until it is given back
  or the instance ends.
- **settled?**: verdicts 24 and 47 settle where the instance is named and that a value keeps its address, not
  the kind of that link. Not for the owner: it refuses only programs that corrupt memory.

## 4. A loop of shared owners (a leak)

- **feature**: `shares` / `shares mut` owners (R9 537-558), and the cycle warning: "The cycle warning
  (L3001) is given for a type whose `shares` fields can form a cycle of owners" (R11 658-660); "Warned, not
  refused" (section 3, 1123-1125); today `[WK-6]` 2822: "It never changes whether a program is accepted."
- **how it breaks safety**: kind 3 (a leak: a count left too high for good, memory nothing uses).

  ```ember
  class Node:
      name: String
      next: Option[shares mut Node] = None

  fn make_pair():
      a = shares mut Node(name=String("a"))
      b = shares mut Node(name=String("b"))
      a.next = Some(shares mut b)
      b.next = Some(shares mut a)
      # a and b end here; each node keeps one owner of the other, so neither count reaches 0

  fn main():
      for i in 0..1000000:
          make_pair()            # two nodes leaked each round; only L3001 is given
  ```

  It compiles with warning L3001 and leaks 2,000,000 nodes. The warning's wording also misses cycles that do
  not go through a `shares` field: `thread_shared` owners, a function value whose state owns a share of the
  object that keeps the function (L3001's own example message names this case, section 3 1124, but the rule
  says "`shares` fields"), `Array[shares dyn I]`, and a channel whose queue holds a `Sender` of itself.
- **fenced?**: no (today and V5).
- **Rust**: allowed; `Rc`/`Arc` cycles compile and leak, and leaking is "safe" in Rust (`std::mem::forget` is
  safe; `Rc::new_cyclic` and `Weak` are the way out).
- **recommendation**, for the owner, one of:
  (a) make a possible cycle of owners a refusal (a new error code at the field that closes it, as `[WK-6]`
  places the warning today: "`Node.next` and `Node.next` can make a loop of owners that keep each other alive
  for good; make one of them a link that does not keep its target alive: `next: Option[weakref Node] =
  None`"), extended to `thread_shared`, function values' state, `shares dyn` and channels; cost nothing while
  running, but some programs Rust accepts are refused (a doubly linked list of `shares` nodes needs one
  `weakref` direction, as Rust's idiom does);
  (b) keep it a warning, as Rust, and write in R16 that a leak through a cycle is the one safety promise not
  kept outside `unsafe` (verdict 60's sentence would then have an exception);
  (c) keep the warning and add `unsafe` only where a program silences it (`@allow(L3001)` inside `unsafe`).
  Recommended: (a), matching verdict 58's reason ("what is so good about leaving a count higher and causing a
  leak").
- **settled?**: no verdict settles cycles; verdict 58 settled only `mem.forget`. It changes how ordinary
  programs are written: **for the owner**.

## 5. The trampoline's generation rolling over

- **feature**: R15, `V5/rules.md` 906-911: "C holds a tagged number, never an address, with the callback's
  generation in its top 16 bits and its slot in its low 48 ... a call after the token ended ... stop[s] the
  program"; area E 1904-1907: the trampoline "checks the slot ..., then the generation and the function's type
  in one compare". No rule says what happens when a slot's 16-bit generation is used up.
- **how it breaks safety**: kind 4 (a check the rules promise is skipped). If a slot is reused (a freed slot
  given to the next registration), after 65,536 registrations in it a stale tag held by C matches the new
  callback of the same function type:

  ```ember
  unsafe extern "C":
      fn unregister(cb: extern "C" fn(*mut void), ud: *mut void)
      @ffi(param(cb, callback(retained, user_data=ud), threads(creator)), destroys(unregister))
      safe fn register_cb(cb: extern "C" fn(*mut void), ud: *mut void) -> Retained[ForeignCallback]
      safe fn call_first()        # C: calls `cb` with the first `ud` it was ever given

  fn main():
      first = register_cb(fn(): println("first"))
      mem.drop(first)                                   # unregistered: C's first tag is now stale
      for i in 0..65535:
          t = register_cb(fn(): println("later"))       # each reuses the slot, generation + 1
          mem.drop(t)
      keep = register_cb(fn(): println("kept"))        # generation back to the first one's
      call_first()                                      # prints "kept"; the rules promise the stop
  ```

  The rules promise "C called the callback registered on line 5 after its token ended" (section 2, 1011);
  the program prints `kept`. The state it runs with is the new callback's own, so no memory is read wrongly;
  the promised check is skipped.
- **fenced?**: no. Unsure: if slots are never reused (48 bits of slots), there is no hole; the design does not
  say.
- **Rust**: nothing like it: a `Box::into_raw` user-data pointer called after its box is freed is undefined
  behaviour, unchecked.
- **recommendation**: as `Pool` does (`SPEC` `[HND-3]` 3095: "A slot whose generation is exhausted is retired
  and never reused"): retire a trampoline slot when its generation reaches 65,535. One compare per
  registration, nothing per call.
- **settled?**: verdict 59 (a count is checked before it rolls over), applied to this count. Not for the owner.

## 6. The weak count of a `thread_shared` object

- **feature**: R9 551: "`shares` counts are plain adds ...; `thread_shared` counts are atomic"; R11 591-595
  ("Every `weakref` link holds one weak count ... making one ... adds one, and its end ... subtracts one");
  section 2 984 (no "atomic" for the weak count). R12 696-697 lets a `thread_shared fn()` hold "a `weakref`
  link" of a `thread_shared` value (it refuses only one of a `shares` value), so links to one `thread_shared`
  object are made and ended on several threads.
- **how it breaks safety**: kind 2, then kind 1. If "`thread_shared` counts" is read as the owner count only,
  two threads add to and subtract from the weak count with plain adds:

  ```ember
  from std.sync import Atomic
  import std.thread

  class Player:
      hp: Atomic[int] = Atomic(10)

  fn main():
      p = thread_shared Player()
      with scope = thread.scope():
          for k in 0..2:
              scope.spawn(fn() copy of k, thread_shared p:
                  for i in 0..1000000:
                      w = weakref p          # +1 to p's weak count
                      # w ends here: -1
              )
  ```

  A lost +1 lets the weak count reach 0 while a link still holds it; when the object ends, its block is freed
  under a live link, and the next `exists as` reads freed memory.
- **fenced?**: no.
- **Rust**: `Arc`'s weak count is atomic (`Arc::downgrade` is an atomic add, with the same overflow abort as
  the strong count).
- **recommendation**: say it: "both counts of a `thread_shared` object, its owners and its `weakref` links,
  are atomic" in R9 and R11, and in section 2's row. Cost: an atomic add where a plain one was never safe.
- **settled?**: verdict 59 ("The weak count ... gets the same check") and R9's intent. Wording only. Not for
  the owner.

## 7. "Static re-entry" when the lock's object is not known

- **feature**: R15 886-887: "A direct call whose body the compiler sees enters the callee past the lock it
  already holds (static re-entry)"; area E 906-909: "A `with` on a lock this thread is known to hold calls the
  callee's entry that skips taking it again"; test `conformance/R15/accept_a_direct_call_that_takes_the_held_lock_enters_past_it.em`.
  For function values R15 refuses by kind: "The compiler cannot tell which `Log` a stored function holds, so a
  function that takes another `Log`'s lock is refused too" (894-895). For a direct call whose lock may or may not
  be the held one, the rule says nothing.
- **how it breaks safety**: kind 2 / kind 4, if the compiler matches by kind as R15 does for function values:

  ```ember
  from std.sync import Lock
  import std.thread

  class Log:
      lock: Lock = Lock()
      lines: Array[String] guarded by lock = []

      fn push(s: StrView):
          with this.lock:
              this.lines.push(String(s))

  fn add_to(held: Log, other: Log):
      with held.lock:
          other.push("x")            # past `other.lock`, or through it?

  fn main():
      a = thread_shared Log()
      b = thread_shared Log()
      with scope = thread.scope():
          scope.spawn(fn(): add_to(a, b))     # pushes to b.lines
          scope.spawn(fn(): b.push("y"))      # pushes to b.lines under b's lock
  ```

  Entered past the lock, the first task changes `b.lines` without `b.lock` while the second holds it: two
  threads change one list (a race; the list's buffer may be freed under the other). Taking the lock is safe
  here, and waits for ever only for `add_to(a, a)` (no memory error).
- **fenced?**: no. Unsure: area E's "known to hold" suggests the safe reading; R15's sentence does not say it.
- **Rust**: `Mutex` is not re-entrant; taking it twice deadlocks or panics; there is no static re-entry.
- **recommendation**: R15: a direct call enters past a lock only when the compiler proves it is the very lock
  this thread holds (the same name, or `this` of the object whose lock is held); where it may be the same and
  may not, the call is refused (E7018, "`other.push` takes `other`'s lock, and `other` may be `held`, whose lock
  this thread holds; write `if other is not held:` above line 13 and indent line 13 under it"), as verdict 43
  refuses a function value that may take another `Log`'s lock. The same for a helper "compiled as needs
  `this.lock` held" (area E 904-906).
- **settled?**: verdict 43 ("A function that changes a different `Log` is refused too (the compiler cannot tell
  which)"), applied to direct calls. Not for the owner.

## 8. A scoped task reading a place the parent writes

- **feature**: R15 851-854: "Every task's effects ... are checked against the other tasks and the parent until
  the join (E7007 for two of them changing one place other than by a reduction ...)"; E7007's meaning (1200):
  "two tasks of one scope change one place". A read in one and a write in the other is not named. Area E's
  answer has the rule (AN-71, `V5/area-E.md` 160: "a place a scoped task reads: the parent and the other tasks
  may not change it until the join"); the tests have the reduction case only
  (`R15/reject_a_task_reads_total_before_the_join.em`, `R15/reject_a_task_reads_a_field_another_task_adds_to.em`).
  A real variable a task uses with no word is not in section 0's list of views (30-36), so R8 (g) does not
  cover it either.
- **how it breaks safety**: kind 2.

  ```ember
  import std.thread

  fn main():
      total = 0
      with scope = thread.scope():
          scope.spawn(fn(): println(total))   # reads `total` on another thread
          total = 5                           # the parent writes it before the join
  ```

  As `rules.md` is worded, nothing refuses it: a data race (for an `int`, a torn or stale read; for a `String`
  replaced the same way, a read of freed text).
- **fenced?**: no.
- **Rust**: the scoped closure borrows `&total`, and `total = 5` before the scope ends is E0506.
- **recommendation**: write AN-71's point into R15 and E7007's meaning: "a place a task reads is not changed by
  the parent or another task until the join, other than through a lock or an `Atomic`" (E7007: "the task on
  line 6 reads `total`, and line 7 changes it before the `with` block ends; change it after the block").
- **settled?**: area E's AN-71 (point 1). Wording. Not for the owner.

## 9. `static mut` without `unsafe`

- **feature**: R15 899-901: "`static mut NAME: T = e` is a module variable that may change ...; it is reached
  only by the main thread and its scoped tasks unless it is `guarded by` a `static` lock (E7012)"; vocabulary
  line 134: "with no `unsafe` (a foreign `static mut` keeps `[STA-1]`'s `unsafe`)". Today: "An `unsafe` context
  is required to: ... access a `static mut`" (`SPEC` `[UNS-1]` 3023).
- **how it breaks safety**: kinds 1 and 2, only through entries 1 and 2: re-entry from C into a function that
  changes it during a walk (entry 1's program), and a reloaded body that changes it (entry 2's program). Inside
  Ember code, R4's call check, R8 (c) and (g) and E7012 cover it as any variable (the tests
  `R15/reject_a_thread_changes_an_unguarded_static_mut.em`, `R15/reject_an_any_thread_export_changes_a_static_mut.em`).
- **fenced?**: today yes; in V5 no, by design.
- **Rust**: every `static mut` access is `unsafe`; Rust 2024 denies references to one (`static_mut_refs`);
  safe globals are `Mutex`, atomics or `thread_local!`.
- **recommendation**: keep it safe (verdict 8, area E); its safety then rests on entries 1 and 2, which must be
  closed.
- **settled?**: verdict 8 ("`static mut` stays"). Not for the owner.

## 10. C writing into Ember memory it keeps

- **feature**: `Retained.pin(v)` and `tok.ptr()` (`SPEC` `[FFI-23]` 4923-4929: "It is the only way to let
  foreign code keep Ember memory"; a `*mut U` for an `owns U`) and the `retained` contract word (`[FFI-11]`
  4826). R12 (745-756) counts only the callbacks C keeps at a foreign call; nothing says a foreign call may
  change an object whose changeable address C keeps.
- **how it breaks safety**: kind 5 (memory changed where the rules cannot see) and kind 6:

  ```ember
  @layout(c)
  struct Stats:
      hits: i32 = 0

  unsafe extern "C":
      @ffi(param(s, retained, one))
      safe fn watch(s: *mut Stats)       # C keeps `s`; tick() adds 1 to s->hits
      safe fn tick()

  fn bump_twice(ref mut s: Stats):     # compiled with noalias (R4 346-348)
      s.hits += 1
      tick()                             # C writes s->hits through the pointer it kept
      s.hits += 1                        # the C compiler may keep s.hits in a register: C's write is lost
  ```

  With `tok = Retained.pin(owns Stats())`, `watch(tok.ptr())`, and `bump_twice` given the pinned stats, the
  noalias promise on `s` is false (undefined C), and a read-only view of the stats held across `tick()`
  breaks R8 (g)'s promise. The exact way the program reaches the pinned value in V5 spelling is not stated
  (the `Retained` row, vocabulary 268, keeps today's API): unsure of the spelling, sure of the gap in R12.
- **fenced?**: in part: `watch` is a `safe fn` in an `unsafe extern` block, so the programmer asserted its
  contract; but the contract says plainly that C keeps a changeable pointer, and the rules do not use that fact.
- **Rust**: every extern call is `unsafe` unless declared `safe fn` in an `unsafe extern` block (Rust 2024);
  C writing through a pointer while Rust holds `&mut` to the same memory is undefined behaviour, which the
  author of the `safe` declaration must rule out.
- **recommendation**: a foreign call not marked `@ffi(no_callbacks)` counts as possibly changing every object
  C keeps a changeable pointer to (a live `Retained` token of an `owns` value, a `retained` mutable parameter),
  for R8 and R4, as it counts the kept callbacks' changes. Nothing while running.
- **settled?**: verdict 31's principle. Not for the owner.

## 11. `thread_shared fn()` "may hold ... copies"

- **feature**: R12 696-697: "A function of such a type may hold `thread_shared` values or copies, never a
  `shares` or `shares mut` value or a `weakref` link of one". Verdict 79 / R3 177-181: `copy of` a struct
  that holds a `shares` field makes "one more owner of the same player".
- **how it breaks safety**: kind 2 (two threads change one plain count), then kind 1:

  ```ember
  import std.thread

  class Player:
      hp: int = 10

  struct Team:
      captain: shares Player

  fn main():
      p = shares Player()
      t = Team(captain=shares p)
      h = thread.spawn(fn() copy of t: println(t.captain.hp))   # "copies" are allowed by R12's words
      q = shares p                    # this thread adds to the plain count while the other may subtract
      h.join()
  ```

  R12's sentence allows it; R15's general sentence refuses it ("A one-thread owner or a lent value reaching
  another thread is E7015", 896-898), and so does the `Send` row (vocabulary 271: "nothing reachable from it is
  a one-thread owner"). So the design refuses it; only R12's wording leaves the gap.
- **fenced?**: refused by R15 (E7015); R12's words disagree.
- **Rust**: a closure holding a struct with an `Rc` is not `Send`; `thread::spawn` refuses it (E0277).
- **recommendation**: R12: "may hold `thread_shared` values, and copies in which nothing is a one-thread owner
  or a `weakref` link of one".
- **settled?**: verdict 61 (as Rust's `Send`). Wording. Not for the owner.

## 12. `@parallel`: reads across chunks

- **feature**: R15 902-903: "A `@parallel` loop splits a range or a list walked by element into chunks (E7016
  for anything else; E7010 where chunks may write one place)"; E7010's meaning (1202). Today's `[PAR-2]`
  (`SPEC` 3583-3589) collects index expressions "**reads included**", and `[PAR-2a]` gives E7011 for a
  loop-carried dependency. V5's code table has no E7011 and does not list it as removed (1216-1218).
- **how it breaks safety**: kind 2, if R15's wording replaced `[PAR-2]`:

  ```ember
  fn main():
      xs = [0] * 100000
      @parallel
      for i in 1..xs.len():
          xs[i] = xs[i - 1] + 1      # a chunk reads the element the previous chunk writes
  ```

- **fenced?**: today refused (E7011). In V5, refused only if `[PAR-2]` still stands; the rule's words name
  writes only.
- **Rust**: `par_iter_mut` gives each closure its own `&mut` element; reading a neighbour needs a shared
  borrow of the slice, refused while it is lent mutably (E0502).
- **recommendation**: keep `[PAR-2]` and `[PAR-2a]` as they are, and say so in R15 ("E7010 where chunks may
  write one place, E7011 where one reads a place another writes"); list E7011 in section 4.
- **settled?**: today's spec. Wording. Not for the owner.

---

## The escapes the duty names, one by one

| escape | V5 status | the line |
|---|---|---|
| `unsafe` blocks and `unsafe fn` | fenced: an `unsafe fn` body puts every unsafe step in its own `unsafe:` block (E3100); `@safety` on every `unsafe fn` (L3015) | R16 913-920 |
| raw pointers (`*T`, `*mut T`, `read`, `add`), `transmute`, `asm`, `get_unchecked`, `assume_disjoint`, `MaybeUninit.assume_init`, `UnsafeCell.get`, union field reads, pointer casts | fenced as today | vocabulary 335-339, 446; `SPEC` `[UNS-1]` 3020-3024, `[TYP-7]` 947, `[FFI-8]` 4727 ("fields are read only in `unsafe`") |
| `Zeroable` implemented for a struct | fenced: an `unsafe` marker, the derive proved field by field | `SPEC` `[ARN-11]` 2976 |
| `@no_alias_checks` | safe: it only forbids `assert_disjoint` inside (E4030); bounds and overflow checks still run | R16 917-918; vocabulary 183 |
| `assert_disjoint` / `check_disjoint` | safe: a run-time compare that stops the program, after which the two are noalias | R4 290-298; section 2, 977, 1007 |
| `@ffi(...)` promises (`no_callbacks`, `callback(borrowed\|retained\|once)`, `threads(...)`, `owned`, `retained`), `safe fn` | fenced: written only in an `unsafe extern` block or an `unsafe overlay`; listed by `ember tcb` | `SPEC` `[FFI-10]` 4631-4637, `[FFI-2]` 4610-4616; R12 749-751. R12 could say where `no_callbacks` may be written. Entry 1 is what the rules miss beside them |
| `Send` declared by hand | fenced: only by `unsafe extend` (E3100) | R15 896-898; section 3, 1113 |
| `ThreadShared` | inferred, written only in bounds; no way to declare it by hand (no code says what `extend T implements ThreadShared:` gets; a wording gap that refuses more, not less) | R15 896-897 |
| `into_raw()` | fenced (E3100) | R16 920-927, verdict 91 |
| `mem.forget` | removed (E1010) | R1 114-115, verdict 58 |
| arenas and memory managers | implementing `Allocator` fenced (`unsafe extend`); the link a value keeps to its instance: entry 3 | `SPEC` 1269; R3 210-230 |
| `static mut` | safe by E7012 and the compile-time checks; entries 1, 2, 9 | R15 899-901 |
| generators | a paused generator is a walk (R8 (c)); no view into its own frame across `yield` (E2221); its result is a view (R6). Not found: whether a generator's body may reach and resume its own frame through the object that holds it; today a `gen fn` method's accesses were checked while running (`[CORO-12]` 2199), V5 drops those checks; R6 and R7 seem to make a generator stored in its own object a view of itself (refused), but no rule says so: **unsure, not a finding** | R6 422-426; R15 904-906 |
| `drop` | ended before its `drop` runs (R11 601-606); ending counts as a call of `drop` (R5 369-374); a `drop` cannot keep its object (E3013); a stop inside `drop` aborts with no unwinding (`SPEC` `[PAN-1]` 2229-2231) | verdicts 56, 57 |
| hot reload | entry 2 | — |
| C callbacks | the checked trampoline in every profile; a callback ending its own token (verdict 62); entries 1, 5, 10 | R12 773-780; R15 906-911 |

## The six kinds, one by one

| kind | what stops it in V5 | not stopped (entry) |
|---|---|---|
| 1. memory misused | bounds checks as today (section 2, 976; `SPEC` `[TYP-31]`); a name used after hand-over (E3040); views ended by reshapes (R8 (a), (b)); a read-only view's promise (R8 (g)); the call check (R4, E3031); views from links across calls that may end their object (R10); function values' state (R12); kept views and arenas outliving what they point into (R7, R3, E3063); `weakref` opened before reading (R13); fields set before a method reads them (E2100; with `SPEC` `[CLS-11]` 1731 for subclasses); `copy of` refused where it would copy a view (E2116); a downcast checked (R14); stack overflow faults (`SPEC` `[RT-12]` 6089) | 1, 2, 3, 10 (and 6 through a lost count) |
| 2. races | `thread_shared` changed only through locks, atomics, channels, `guarded by` (E7014, E7013); one-thread owners never reach another thread (E7015, `thread_shared fn()`); `static mut` (E7012); scoped tasks' changes (E7007) and reductions; `@parallel` (`[PAR-2]`) | 6, 7, 8, 11, 12 (wording), 1 and 2 through `static mut` |
| 3. leaks | `mem.forget` removed; counts checked before they roll over (R9 555-558, R11 593-595); a value always ends with its clean-up (R1 114-115). An arena that never frees single values keeps them until it ends, by its own design (not a finding) | 4 |
| 4. checks skipped | no switch turns a check off (R16 918-919; `@no_runtime_checks` removed); the trampoline's checks in every profile (R15 906-911); overflow and shift checks in every profile (`SPEC` `[TYP-8]` 949, `[TYP-10]` 985); a call that would take a held lock (E7018) | 5, 7 |
| 5. memory the rules cannot see | raw pointers, `transmute`, unions, `asm` inside `unsafe`; C calls need a complete contract or `unsafe` (`SPEC` `[FFI-2]`); kept callbacks counted (R12) | 1, 2, 10 |
| 6. undefined C | wrapping is never undefined (`SPEC` `[TYP-8]` 959); shifts checked (`[TYP-10]`); zero divisor and `MIN // -1` (`[TYP-28]` 978); float to integer saturates (`[TYP-6]` 940); a NaN never makes behaviour undefined, `@fastmath` included (`[TYP-9]` 1008-1010); references to packed fields refused (E2170); `restrict` only on a `ref mut` parameter whose call is checked (R4 346-348), never on `this` (R5 395-396) | 1, 2, 10 (a `ref mut` parameter changed by C or by reloaded code during the call) |

## Looked at, not findings

- Panics while a list is half-changed, inside a lock, inside a callback C runs, inside `drop`: a stop aborts the
  process with no unwinding (`SPEC` `[PAN-1]` 2229-2231; `[FFI-25]` 4954), so no half-changed state is seen.
- `Pool` / `Handle` generations: retired when used up (`SPEC` `[HND-3]` 3095).
- A `ref mut` parameter reached through a link of the same type inside the callee: R8 (f) refuses it ("a link
  of the same type included", 508).
- `@cached` results after a change through a link: E2117 refuses a cached method that reads `shares`,
  `weakref`, `sharedref mut`, `Cell` or lock-guarded fields; its `ref T` result points into `this` (R6), so
  R8 (g) keeps it.
- A `thread_shared` object's `drop` that changes a `static mut`, run on whichever thread ends it last: ending
  counts as a call of `drop` (R5 369-374), so the thread that may end it changes what the `drop` changes (E7012).
- A `weakref` link on this thread to an object handed to another thread: refused (R12 704-707, E7015; the hole
  `V5/safety/recheck.md` section 6 found is now written in).
