# 05 The attacker: ordinary programs, no `unsafe`, that break a safety promise anyway

Agent 05 of the unsafe-features inventory (verdict 60). Each program below is written in the new
design's spelling (`V5` = `tasks/96/redesign/run4/v5/`), with no `unsafe`, and tried against the rules
as written (`V5/rules.md`, `V5/vocabulary.md`, the answers `V5/area-*.md`) and today's spec where V5
keeps it (`SPEC` = `docs/spec-source/Ember_v0.9.10_Hardened_35.md`). Paths are under
`C:\Users\ism19\Code\ember\`. Nothing was compiled or run: "what it does when run" is what the rules
and the spec say it does, and every claim names the line it rests on. Section 1 lists every attack
the duty names (and a few more) with the rule that refuses it or makes it safe. Section 2 holds the
attacks that nothing stops: the findings, in the brief's form, most harmful first. Section 3 is the
short list for the form.

## 1. Every attack tried, and what stops it

"Refused" means a rule refuses the program when compiling; "safe" means it runs and does what it says.
A row that says "finding" is in section 2.

| # | attack (no `unsafe`) | result | the line that stops it |
|---|---|---|---|
| 1 | a loop of `shares` owners: `a.next = Some(shares mut b)`, `b.next = Some(shares mut a)` | **finding F4**: accepted with warning L3001, leaks | `V5/rules.md:658-660`, `:1123-1125`, `:1210` ("a warning: a leak, not a memory error"); `SPEC:2801-2803` `[WK-1]`, `:2822-2824` `[WK-6]` ("never changes whether a program is accepted") |
| 2a | `static mut COUNT` changed by `thread.spawn(fn(): COUNT += 1)` and by `main` | refused, E7012 | `V5/rules.md:899-901`; the spawned function is a `thread_shared fn()`, `:845-847` |
| 2b | the same from two scoped tasks | refused, E7007 (a module variable is not "a local or a field", so `COUNT += 1` is not a reduction) | `V5/rules.md:852-856` |
| 2c | the same from a `@parallel` loop's body | refused, E7010 (a written place must be a captured `MutSpan` or `SoA` column) | `V5/rules.md:902-903`; `SPEC:3583-3592` `[PAR-2]` |
| 2d | the same from a job (`jobs.submit`), an `@export(threads=any)` function, or the `drop` of a `thread_shared` object whose last count ends on another thread | **finding F2** | see F2 |
| 3a | a stop (panic) inside `drop` | safe: the process aborts, no unwinding, no destructor runs | `SPEC:2229-2231` `[PAN-1]`, `:2505` `[DRP-4]`, `:2234` `[PAN-3]` |
| 3b | a stop inside a lock (`with g = m.lock():`) | safe: abort; the lock is never released to another thread, there is no poisoning | `SPEC:3511-3516` `[THR-3]`. Note: `Atomic.fetch_add` stores the overflowed value before the stop and another thread may read it (`SPEC:3520-3522` `[THR-14]`): a wrong number, not a memory error (indexing stays checked) |
| 3c | a stop inside a callback C runs | safe: abort at the boundary, nothing unwinds into C | `SPEC:4621` `[FFI-4]`, `:4954-4956` `[FFI-25]` |
| 3d | a stop while a list is half-changed (a `sort_by` comparator or a `retain` test that panics) | safe: abort, nothing is dropped or run afterwards (the host's C panic hook, `SPEC:6069-6070` `[RT-4]`, is the host's code) | `SPEC:2229-2231` |
| 4 | deep recursion (`fn f(n: int) -> int: return f(n + 1)`; a long `owns` list dropped recursively) | safe: "stack overflow in f", abort, every profile | `SPEC:6089-6093` `[RT-12]`; `compiler/ember_build/src/lib.rs:1983-1985` (`-fstack-clash-protection` on Linux; MSVC probes by default). Threads a C host makes keep the host's guard, as in Rust |
| 5a | `+ - * // % **`, unary `-`, compound assignments past the type's range | safe: stop, every profile; `@overflow(wrap)` is two's complement and "never undefined behaviour in the generated code" | `SPEC:951-968` `[TYP-8]` |
| 5b | `x << 64`, `x >> -1` | safe: stop, every profile | `SPEC:985-989` `[TYP-10]` |
| 5c | `x // 0`, `MIN // -1`, `MIN % -1` | safe: stop, or `MIN`/`MAX` under wrap/saturate; `x % -1 == 0` (the runtime's floor modulo returns 0 for a divisor of -1 before C's `%` runs, `runtime/ember_rt/include/ember_rt.h:659`) | `SPEC:969-980` `[TYP-28]` |
| 5d | `1e300 as i32`, `nan as int`, `300 as u8` | safe: saturate, NaN is 0, low bits kept; the runtime tests NaN by its bits so `@fastmath` cannot drop the test (`ember_rt.h:725-740`; `compiler/ember_build/src/lib.rs:1502-1508`: `@fastmath` never gets `-ffinite-math-only`) | `SPEC:938-946` `[TYP-6]`, `:1001-1009` `[TYP-9]` |
| 5e | a capacity whose byte size overflows; 2^32 owners; 2^32 links | safe: stop | `SPEC:2888-2890` `[HEAP-8]`; `V5/rules.md:555-558` (verdict 59), `:593-595` |
| 6a | a field read before `init` sets it, by `init`'s own lines | refused, E2100 | `SPEC:1712-1718` `[CLS-2]`; `V5/rules.md:397-401` |
| 6b | the same through a method `init` may now call, which lets `this` out | **finding F5** | see F5 |
| 6c | a list made with a size, read before it is written | refused or safe: no constructor makes unwritten elements; `alloc_array` needs `Default`, `alloc_zeroed` needs `Zeroable` (an `unsafe` marker the compiler proves field by field), `alloc_uninit` gives `MaybeUninit`, whose `assume_init` is `unsafe` | `SPEC:2938-2944` `[ARN-3]`, `:2969-2973` `[ARN-8]`, `:2976-2982` `[ARN-11]`; a local used before set, E3050, `SPEC:2321` `[BRW-7]` |
| 6d | a struct, `bool`, `char` or union from C | fenced: only through declarations in an `unsafe extern "C":` block; a union's fields and a flexible tail only in `unsafe` | `SPEC:4631-4637` `[FFI-10]`, `:879-881` `[TYP-2]`/`[TYP-3]`, `:4727`, `:4736` |
| 7a | a generator kept after the list it walks changes or ends (`g = d.replay()`, `d.log.push(...)`, `g.next()`; `g` returned past `d`) | refused, E3020 / E3063 | `V5/rules.md:422-426` (a generator's result is a view), `:428-431` (R7), `:478-486` (R8 (c)), `:904-905` (E2221); `SPEC:2185` `[CORO-6]` |
| 7b | a suspended generator whose `gen fn` body is changed by a hot reload | **finding F11** (unsure) | see F11 |
| 8a | hot reload changing a type's layout under live objects | safe for what today's spec names: migrated by name or refused, references rewritten or the reload refused | `SPEC:6339-6361` `[HR-14]`, `:6363-6369` `[HR-15]`/`[HR-15a]`, `:6264-6268` `[HR-3]`, `:6272-6282` `[HR-3b]`, `:6289` `[HR-4]` |
| 8b | a reload at `std.hot.checkpoint()` that changes what a function does (not its types) under a frame that still runs old code | **finding F1** | see F1 |
| 8c | a reload that moves an object V5's new links point at (the arena-instance word, a `sharedref mut` held by a static) | **finding F12** (unsure) | see F12 |
| 9a | scoped tasks with views (`scope.spawn(fn() ref t: ...)`; the parent changing `texts` inside the scope) | refused where they clash (E3026 for `thread.spawn`; E7007/E3021 against the parent until the join) | `V5/rules.md:852-853`, `:866-876` (verdict 93) |
| 9b | reductions (`total += x`, `names.push(...)`) and a task or the parent reading the place before the join | safe (private sum per task, combined at the join in spawn order) / refused, E7007 | `V5/rules.md:854-865` |
| 9c | a `@parallel` loop writing through a link, or two spans of one list | refused, E7010 / E3021 (`assert_disjoint`) | `SPEC:3583-3592` `[PAR-2]`; `V5/rules.md:287-298`, `:902-903` |
| 10a | a `weakref` of a one-thread object handed to `thread.spawn` (`fn() owns w:`), or the object handed over while a link on this thread points at it | refused, E7015 | `V5/rules.md:704-707`, `:1106`; `V5/area-E.md:170-172`, `:490-493` |
| 10b | a `weakref` to a `thread_shared` object opened on another thread | safe: the view holds one atomic count for its life | `V5/rules.md:576-578` |
| 10c | `weakref` links made or ended by two scoped tasks at once | **finding F6** | see F6 |
| 10d | a walk up a tree of `thread_shared` nodes with the uncounted region link | **finding F7** | see F7 |
| 11 | a function value stored and replaced while it runs (it replaces its own slot, by its holder or through a `sharedref mut` link to the holder, then uses what it captured) | safe: the call copies `{code, state}` out and keeps no pointer into the slot; a body that may end its own state holds one count across the call; storing it where a real variable it changes owns it is refused, E3022 | `V5/rules.md:773-780`, `:727-730`. Wording: "may end its own state (it removes itself)" should say "through any name", so the may-alias rule of R8 (b) (`:462-464`) is the one used |
| 12 | interfaces and casts of the wrong kind (`cast s as Circle` on a `Square`; a `dyn` of the wrong interface) | safe: one type-id compare, stop; a test that can never pass is refused, E2037; pointer casts need `unsafe` | `V5/rules.md:811-833`, `:1010`; `SPEC:947` `[TYP-7]` |
| 13a | a value made in an arena instance kept past the instance (returned, put into the caller through `ref mut`, the arena handed over while the value lives) | refused, E3063 / E3060 | `V5/rules.md:219-229` (R3), `:349-353` (R4); conformance `V5/conformance/R3/reject_a_value_returned_past_the_end_of_its_arena.em` |
| 13b | an arena with a safe `reset()` that rewinds while values made in it live | fenced: only an `unsafe extend ... implements Allocator` author can write such an arena, and the promise is theirs (`SPEC:2997-2998` `[ALC-2]`). Note: V5 lets the program change the instance while values made in it live (`V5/rules.md:228-229`), so no safe `reset()` can be sound; `std/src/mem.em:115-121`'s promise should say no method may reuse a block while a value made in it lives (a resetting arena takes `owns this`: `current = current.reset()`, which R3 refuses while values made in it live) | as said |
| 14a | a C call to a callback after its `Retained` token ended; a second call of a `callback(once)`; a call on the wrong thread | safe: the checked trampoline stops the program, every profile | `V5/rules.md:906-911`, `:1011-1013` |
| 14b | the same after the slot's generation has rolled over | **finding F10** | see F10 |
| 15a | `@cached` after a change to a field of `this`, or reading a link field | safe (the kept result is cleared) / refused, E2117 | `V5/area-B.md:129-134`; `V5/rules.md:1174` |
| 15b | `@cached` after a change made through a `sharedref mut` link (or a `shares mut` owner) into an object the method reads | **finding F8** | see F8 |
| 16 | (more) a field handed over from a struct whose type has a `drop` | refused, E3010; V5's R2 (`V5/rules.md:133-140`) does not repeal it, but should name it | `SPEC:1907-1910` `[EXP-6]` |
| 17 | (more) a value given away without its clean-up: `MaybeUninit.write`, `Arena.alloc_nodrop` | **finding F3** | see F3 |
| 18 | (more) a `drop` that makes a new owner of its own object | **finding F9** | see F9 |
| 19 | (more) `@export("free")` | **finding F13** | see F13 |
| 20 | (more) a `ref mut` parameter (compiled `restrict`) reached through a `sharedref mut` link of the same type inside the callee | refused in the callee, E3022/E3021 | `V5/rules.md:506-511` (R8 (f): "a link of the same type included") |
| 21 | (more) `sort_by` with a comparator that is not a total order | safe: the C introsort that skips bounds tests runs only on the compiler's own orders; a program's own `cmp` goes to the library's checked sort | `runtime/ember_rt/include/ember_rt.h:2083-2094`; `compiler/ember_typeck/src/lib.rs:32612-32614`, `:33028-33030` |

## 2. Findings: what nothing stops

### F1. Hot reload at a checkpoint lets old code call new code that breaks the proof the old code was compiled under

- **feature**: `std.hot.checkpoint()` and `[HR-3b]` (`SPEC:6272-6282`; `std/src/hot.em:35`): "At a
  checkpoint the frames below it run their old code on after the swap", refused only when a frame
  depends on a changed type's **layout**. V5 moves every alias and loop check to compiling time
  (`V5/rules.md:375-381`: "Nothing is checked while the program runs") and says nothing about reload.
- **how it breaks safety**: kind 1 (use after free). Version 1:

  ```ember
  import std.hot

  static mut LOG: Array[String] = [String("a"), String("b")]

  fn note():
      println("tick")

  fn run():
      for line in LOG:
          std.hot.checkpoint()        # `ember run --hot` applies the edit here
          note()
          println(line)

  fn main():
      run()
  ```

  The programmer then edits, while `run` waits at the checkpoint: `fn note(): LOG.push(String("c"))`,
  and, because version 2 must compile (E3020: "`note()` adds to `LOG` while this loop walks `LOG`"),
  moves `note()` below the loop in `run`. Version 2 compiles and changes no type, so `[HR-3b]` applies it
  at the checkpoint. The old `run` frame goes on with its old code; its `note()` goes through `note`'s
  thunk (`SPEC:6298-6301` `[HR-5]`, `[HR-6]`) to version 2, which pushes to `LOG` while the old loop
  walks it. The push may move `LOG`'s buffer; `println(line)` and the loop's next step read the freed
  buffer. The same holds for every fact R17 lists (`V5/rules.md:930-948`): a parent-link read compiled
  with no header test (R11 condition 1, `:609-624`) in an old frame, under a new function that moves
  children without clearing their links, reads a freed parent; a view parameter the new body uses after
  a `clear` (R4's call check, `:316-345`).
- **fenced?** No, in both. Today the walk of a class field is a long-term access checked while running
  (`SPEC:2682-2700`, VIII.3), so the reloaded callee's push is stopped, and a module variable that changes
  exists only as `static mut` inside `unsafe` (`SPEC:1805-1806` `[STA-1]`); V5 removes the run-time
  access checks (`V5/rules.md:1023`: "Gone ... the access words checked on class fields") and makes
  `static mut` safe (`V5/vocabulary.md:134`). No line in V5 or in `[HR-3b]` refuses this reload.
- **Rust**: no hot reload in the language; loading new code goes through `libloading`, whose `Symbol`
  is `unsafe`.
- **recommendation**: make it safe with a rule, nothing paid while running: `[HR-3b]`'s record for each
  call lists, beside the types, the R17 facts of every function the old frame may still call after the
  checkpoint (what it may change, read and end; which view parameters it uses after a change; the locks
  it takes; the parent-link facts). A reload whose new code changes one of them for such a function is
  refused at that checkpoint, naming the function and the line, as a layout change is: "`note` now adds
  to `LOG`, which `run` walks below this checkpoint (line 9); the change waits for a checkpoint outside
  the loop". A function value called by an old frame counts by every function the new program can put
  there (R12).
- **settled?** No verdict settles it; not for the owner (no ordinary program changes; some reloads wait).
- **how sure**: high that the rules allow it as written (`[HR-3b]`'s text names only layouts); the harm
  follows from V5's own statement that nothing is checked while running.

### F2. `static mut` reached from threads the rules do not name: jobs, exports, a `drop` on another thread

- **feature**: `static mut` made safe (verdict 8; `V5/vocabulary.md:134`), "reached only by the main
  thread and its scoped tasks unless it is `guarded by` a `static` lock (E7012)" (`V5/rules.md:899-901`).
  The rules name as "a call that runs a function on its own" only `thread.spawn`, `scope.spawn` and a C
  callback kept by C (`V5/rules.md:731-735`), and give a type to only `thread.spawn`'s function
  (`thread_shared fn()`, `:845-847`). Not named: `std.jobs` (`jobs.submit(owned f: fn())` and
  `JobScope.submit`, `SPEC:3626-3629` `[JOB-2]`; `V5/vocabulary.md:356`: `JobScope` "same"), an
  `@export(threads=any)` function (`SPEC:4964-4968` `[FFI-33]`, whose check speaks of `static`, not
  `static mut`), and the thread on which a `thread_shared` object's `drop` runs.
- **how it breaks safety**: kind 2 (race), and through C's rules kind 6 (a data race on a plain `int` is
  undefined in C11).

  ```ember
  import std.jobs

  static mut COUNT: int = 0

  fn main():
      h = jobs.submit(fn(): COUNT += 1)    # runs on a job worker
      COUNT += 1                           # main changes it at the same time
      jobs.wait(h)
      println(COUNT)                       # 1 or 2
  ```

  The lambda holds nothing (a module variable needs no capture word, `V5/rules.md:876`), so the
  inferred `Send` (`V5/vocabulary.md:271`) lets it go to the worker, and nothing tells E7012 that the
  worker is another thread. The same with `@export(threads=any) pub extern "C" fn on_audio(): COUNT += 1`
  called by an audio library on its own thread. And with
  `class Res:` whose `fn drop(): COUNT += 1`, made `r = thread_shared Res()`, a thread that only opens
  `w = weakref r` (`if w exists as x:`) holds the count that may be the last one (`V5/rules.md:576-578`)
  and then runs `drop` on its own thread, while `main` changes `COUNT`; R5's list of what may end an
  object (`:369-371`) does not include the end of that count, so the thread's function is not counted as
  changing `COUNT`.
- **fenced?** Today yes: every access to a `static mut` is in `unsafe` (`SPEC:1805-1806`). V5: no
  line fences these three ways in.
- **Rust**: `static mut` access is `unsafe`; `std::thread::spawn`, rayon and every job system require
  `F: Send + 'static`; a type whose `Drop` touches a `static mut` needs `unsafe` there.
- **recommendation**: settled by verdicts 61 and 94, applied to the library's other thread starts:
  `jobs.submit(owns f: thread_shared fn())`, counting as `thread.spawn` for R5, R12 and R15;
  `JobScope.submit` as `scope.spawn`; E7012's "another thread" includes job workers, an
  `@export(threads=any)` body (and `creator` when it is not the thread that runs `main`), a C callback
  whose contract allows another thread, and the `drop` of a `thread_shared` object: a class whose
  `drop` reaches a main-thread `static mut` cannot be made `thread_shared` (E7012 at the making line:
  "`Res`'s `drop` adds to `COUNT`, a module variable of the main thread, and a `thread_shared Res` may
  end on any thread; guard `COUNT` with a lock, or make it an `Atomic`"). Nothing paid while running.
- **settled?** Verdicts 61 and 94 (apply them); not for the owner.
- **how sure**: high that V5 names none of the three; medium on the `drop` case (an implementer might
  count the opening's count as "may end").

### F3. Ending a value without its clean-up is still possible without `unsafe`: `MaybeUninit.write` and `Arena.alloc_nodrop`

- **feature**: `MaybeUninit[T].uninit()` and `write` are safe, and "dropping it never drops a `T`"
  (`SPEC:2969-2973` `[ARN-8]`; the compiler asks `unsafe` only for `assume_init`:
  `compiler/ember_typeck/src/lib.rs:49935-49945` against `:49947-49954`). `alloc_nodrop` puts a value
  that needs `drop` in an arena "which acknowledges that its `drop` will never run" (`SPEC:2936-2937`
  `[ARN-2]`). V5 keeps both (`V5/vocabulary.md:326` `MaybeUninit` "same", only `assume_init` unsafe;
  `:321` `Arena` "same"), against its own R1: "nothing ends an owner without it (`mem.forget` is not
  Ember, E1010)" (`V5/rules.md:114-115`).
- **how it breaks safety**: kind 3 (leak), and every guarantee that rests on a `drop` (a `Retained`
  token never ends, so C keeps the callback; a count never comes down).

  ```ember
  from std.mem import MaybeUninit

  class Player:
      hp: int = 10

      fn drop():
          println("player freed")

  fn main():
      p = shares Player()
      for i in 0..3:
          slot = MaybeUninit[shares Player].uninit()
          slot.write(shares p)          # one more owner; `slot` ends without ending it
      println("end of main")
  ```

  It prints "end of main" and never "player freed": the player's count stays at 3 after `p` ends, and
  the player is never freed while the program runs, verdict 58's own case ("what is so good about leaving
  a count higher and causing a leak"). Writing twice into one slot loses the first value the same way.
  `frame.alloc_nodrop(shares p)` does the same in an `Arena`.
- **fenced?** No, today or in V5 (quoted above).
- **Rust**: `MaybeUninit::write`, `ManuallyDrop::new`, `Box::leak` and `mem::forget` are all safe; Rust
  counts a leak as safe.
- **recommendation**: settled by verdict 58 (and 60): remove `alloc_nodrop` (a value that needs `drop`
  goes in an arena as `owns T using frame`, whose end runs its `drop` and the arena's `deallocate`, R3);
  `MaybeUninit[T]` for a `T` that needs `drop` only inside `unsafe` (E3100 at `uninit()` outside:
  "`MaybeUninit` never runs a `Player`'s clean-up, so a value written into it may never be freed;
  write it inside `unsafe:`, or keep the value in an `Option[shares Player]`"); for a `T` with nothing
  to drop it stays safe. Nothing paid while running. The vocabulary's `mem.forget` row is due anyway
  (verdict 58's "to do").
- **settled?** Verdict 58 (the same problem as the removed word); not for the owner.
- **how sure**: high (spec text and compiler lines quoted).

### F4. A loop of `shares` owners leaks, with only a warning

- **feature**: `shares` / `shares mut` fields (and function values holding them, and `thread_shared`
  owners inside a lock): `V5/rules.md:658-660` gives "the cycle warning (L3001)", `:1123-1125` lists it
  under "Warned, not refused", `:1210`: "a warning: a leak, not a memory error". Today the same:
  `SPEC:2801-2803` `[WK-1]` ("it leaks. This is a documented property"), `:2822-2824` `[WK-6]` ("It never
  changes whether a program is accepted").
- **how it breaks safety**: kind 3 (leak: "a loop of shared owners that keep each other alive", the
  brief's own example).

  ```ember
  class Node:
      name: String
      next: Option[shares mut Node] = None

  fn make_pair():
      a = shares mut Node(name=String("a"))
      b = shares mut Node(name=String("b"))
      a.next = Some(shares mut b)
      b.next = Some(shares mut a)       # each node's count is 2; when `a` and `b` end, 1, never 0

  fn main():
      for i in 0..1000000:
          make_pair()
      println("done")
  ```

  It prints "done" with two million nodes and their texts never freed while it runs; L3001 warns, the
  debug leak report lists them at exit (`SPEC:2833-2834` `[WK-15]`), release says nothing. The same
  through a function value: `ui.handlers.push(fn() shares mut ui2: ui2.n += 1)` with
  `ui2 = shares mut ui` (L3001's own message, `V5/rules.md:1124`); and through `thread_shared` objects
  linked inside a `Mutex`, which L3001 does not cover (it speaks of "`shares` fields", `:1210`).
- **fenced?** No, today or in V5.
- **Rust**: `Rc`/`Arc` cycles leak in safe Rust; Rust treats leaks as safe.
- **recommendation**: make it safe with a rule, nothing paid while running: in L3001's ownership graph
  (with every function value's state, found from the whole program, R12, and `thread_shared` owners), a
  cycle of owner edges may be closed only by a `let` field, set once when the object is made (it can name
  only objects that already exist, so it cannot close a loop), and `init` makes no owner of `this`; a
  changeable field that can close a cycle is refused, with "`Node.next` can make a loop of owners that
  never ends (`a.next` to `b`, `b.next` to `a`); make the link back `weakref`: `next: Option[weakref
  Node]`, or make `next` a `let` field set when the node is made". Cost: programs with changeable shared
  back-links write them as `weakref` (the parent-link pattern, R11) or `let`. The other two choices: keep
  the warning (Rust's choice, against verdict 60's wording), or a cycle collector while running (Python's;
  not C's speed).
- **settled?** No verdict settles it (58 and 59 are about counts left high by `forget` and roll-over).
  **For the owner**: the rule changes how ordinary programs with changeable shared back-links are
  written.
- **how sure**: high.

### F5. `init` may call a method that lets `this` out before its fields are set

- **feature**: V5's relaxation of `[CLS-2]`: "`init` ... may call a method of it once every field that
  method's body reads is set" (`V5/rules.md:397-401`; E2100 `:1049`). Today any method call on `self`
  before every field is set is E2100 (`SPEC:1712-1718`: "`self` is used as a whole by a method call on
  it, by passing, storing or returning it, and by a closure capturing it"). A method that stores a link to
  `this`, and reads no field, now passes the test: making a link reads nothing (`V5/rules.md:595-597`).
- **how it breaks safety**: kind 1 (reading memory not yet written).

  ```ember
  static mut ROSTER: Array[weakref Player] = []

  class Player:
      ready: bool
      name: String

      fn init(r: bool):
          this.ready = r
          this.enrol()              # `enrol` reads no field, so R5 lets `init` call it now
          print_roster()            # reads `name` of every enrolled player, this one's included
          this.name = String("ana")

      fn enrol():
          ROSTER.push(weakref this)

  fn print_roster():
      for w in ROSTER:
          if w exists as p:
              println(p.name)       # `name` of the player being built: never written

  fn main():
      p = Player(true)
  ```

  `print_roster` reads the `String` header of a field `init` has not written (the object's block comes
  from the allocator, unwritten), follows its pointer and prints whatever it finds, or faults.
- **fenced?** Today yes (E2100 at `this.enrol()`, `SPEC:1714-1715`). V5: no line refuses it.
- **Rust**: an object exists only once every field has a value; no partly built object can be reached.
- **recommendation**: make it safe with a rule, nothing paid while running: the summary R5 works out also
  records whether a method lets `this` out (stores a link or an owner of it, passes it on, captures it,
  returns it), and such a method is called in `init` only after every field is set (E2100: "`enrol` keeps
  a link to the player (`ROSTER.push(weakref this)`, its line 2), and `name` is set only on line 9; set
  every field first: move line 6 below line 9").
- **settled?** No verdict; not for the owner (row 7's program still compiles).
- **how sure**: medium: the rules do not say that letting `this` out counts as a read; an implementer
  might keep `[CLS-2]`'s list for the method's body too.

### F6. Two scoped tasks change one object's weak count at the same moment

- **feature**: R11, "making or ending a link is not a change of the target: any name may make one"
  (`V5/rules.md:595-597`); scoped tasks use the parent's variables with no word (`:848-851`); E7007 is
  about changes (`:852-853`); E7015 names `shares` owners and their links, and functions any thread may
  run (`:1106`). Nothing says a weak count of a one-thread object is changed atomically (R9 makes only
  `thread_shared` counts atomic, `:551`).
- **how it breaks safety**: kind 2, then kind 1.

  ```ember
  import std.thread

  class Player:
      hp: int = 10

  fn main():
      players = [Player()]
      keep: Array[weakref Player] = []
      with scope = thread.scope():
          scope.spawn(fn(): keep.push(weakref players[0]))   # +1 on the player's weak count, task 1's thread
          scope.spawn(fn(): keep.push(weakref players[0]))   # +1 on the same count, task 2's thread
      players.clear()        # the player ends; its block stays while its weak count is above 0
      keep.pop()             # -1
      if keep[0] exists as p:
          println(p.hp)
  ```

  Both tasks only read `players` (allowed), and `keep.push` is a reduction (`V5/rules.md:854-856`). The
  two plain adds can lose one: the count is then 1 for two links, the `pop` frees the block, and the
  test of `keep[0]` reads freed memory (and its end subtracts from it). Also unsure, same family: a task
  that opens a parent's `weakref` (`scope.spawn(fn(): if w exists as p: println(p.hp))`) while the
  parent runs `others.clear()` in the scope; R10's rule is stated for one thread's lines (`:565-575`) and
  R15's "checked against the parent" (`:852`) does not say it applies R10.
- **fenced?** Today yes: a scoped task's shared borrow needs `Sync` (`SPEC:3564-3569` `[THR-11]`), and
  a non-`@sync` class handle is not `Sync`. V5 removes `@sync` (`V5/vocabulary.md:178`) and adds no rule
  for weak counts.
- **Rust**: `Rc::downgrade` needs an `Rc`, which is neither `Send` nor `Sync`; scoped threads refuse it.
- **recommendation**: make it safe with a rule, nothing paid while running: for the thread rules, making,
  ending or opening a `weakref` link of an object counts as touching its header; a scoped task may do it
  only for a `thread_shared` object (atomic counts) (E7015: "`players[0]` counts its links without
  atomics, and the task on line 9 makes one too; make the links after the scope, or make the player
  `thread_shared`"); and a parent's line that may end an object of a type a task reads through a link is
  refused until the join, as R10 point 1 refuses it on one thread.
- **settled?** No verdict; not for the owner.
- **how sure**: medium-high for the first program (R11's sentence is explicit); unsure for the second.

### F7. The uncounted link of a walk's deferred-drop region, with `thread_shared` nodes

- **feature**: R11's cost paragraph: a walk whose node class has no `drop` "is placed in a deferred-drop
  region ... (one add and one subtract of a per-thread counter per region ...); a link that lives only
  inside the region and is given only objects that live at that moment (`up = weakref p` from the view
  `p`) then holds no weak count" (`V5/rules.md:651-657`). The only condition stated is "no `drop` method
  of its own" (`:652`); the parent-link proof excludes `thread_shared` objects (`:617`), this paragraph
  does not. The region is per thread, so it cannot hold back a block another thread frees.
- **how it breaks safety**: kind 1 (use after free).

  ```ember
  import std.thread
  from std.sync import Mutex

  class Node:
      name: String
      parent: weakref Node = None
      kids: Mutex[Array[thread_shared Node]] = Mutex([])

  fn prune(n: Node):                    # may change nodes: empties n's list of children
      with g = n.kids.lock():
          g.clear()

  fn walk_up(leaf: Node):
      up = weakref leaf
      while up exists as n:             # tests the header `up` names
          if n.parent exists as p:      # holds an atomic count on `p` for this block (R10 point 2)
              up = weakref p            # in the region: no weak count
              prune(p)
          else:
              break

  fn main():
      root = thread_shared Node(name=String("root"))
      mid = thread_shared Node(name=String("mid"), parent=weakref root)
      leaf = thread_shared Node(name=String("leaf"), parent=weakref mid)
      with g = root.kids.lock():
          g.push(thread_shared mid)
      with g = mid.kids.lock():
          g.push(thread_shared leaf)
      mem.drop(mid)                     # `root.kids` holds `mid`'s only owner now
      t = thread.spawn(fn() thread_shared root: with g = root.kids.lock(): g.clear())
      walk_up(leaf)
      t.join()
  ```

  When thread `t` empties `root.kids` after the walk lets go of `p`'s count and before the next
  `while up exists as n:`, `mid` ends on `t`, outside this thread's region, and with no weak count held by
  `up` its block is freed; the test reads it. (`import std.mem` as in
  `V5/conformance/R3/run_a_value_ends_in_the_arena_instance_it_came_from.em`.) Related wording gap: R11
  and section 2 (`:984`) never say that a `thread_shared` object's weak count is changed atomically.
- **fenced?** No: V5 is new here, and no line excludes `thread_shared` targets.
- **Rust**: a `Weak` of an `Arc` always holds an atomic weak count; there is no uncounted kind.
- **recommendation**: make it safe with a rule: the region and its uncounted link apply only to objects
  no other thread can end (not `thread_shared`, not reachable from one); say in R11 that a `thread_shared`
  object's weak count is atomic, as its owner count is. Cost: none for one-thread trees; a `thread_shared`
  walk pays the count moves.
- **settled?** No verdict (28 asks for compile-time proof, not this); not for the owner.
- **how sure**: medium (the program needs a timing; the rule text plainly allows the link).

### F8. `@cached` keeps a result a change through a link does not clear

- **feature**: `@cached` (new, `V5/vocabulary.md:409`): "cleared by every change, through any name, to
  a field its body reads"; "a `Copy` result is returned by value" (`V5/area-B.md:129-134`), and `StrView`
  is `Copy` (`V5/rules.md:25-27`); the clear is "one store (clear the flag) at each place that may change"
  a field read (`V5/area-B.md:877`). E2117 refuses reading a link field of `this` (`V5/rules.md:1174`),
  not reading what `this` owns that another name may change through a `sharedref mut` link (verdict 81
  allows one to a list's class object, `V5/rules.md:472-476`) or a `shares mut` owner made from it.
- **how it breaks safety**: kind 1 with a view result, kind 4 with a number.

  ```ember
  class Item:
      name: String

  struct Shelf:
      items: Array[Item]

      @cached
      fn first_name() -> StrView:
          return this.items[0].name.trim()

  struct Tag:
      target: sharedref mut Item

  fn main():
      s = Shelf(items=[Item(name=String("apple"))])
      t = Tag(target=sharedref mut s.items[0])
      println(s.first_name())          # computes a view of "apple" and keeps it in `s`
      t.target.name = String("pear")   # replaces the item's text: "apple"'s block is freed
      println(s.first_name())          # gives the kept view: reads the freed block
  ```

  The write `t.target.name = ...` changes an `Item` reached through a link; the compiler can clear a
  slot only in an object the written path goes through, and this path does not go through `s`, so the
  kept view is returned again. With `-> int: return this.items[0].name.len()` the second call prints
  the stale 5, a check the rules promise skipped. A reload that changes `first_name`'s body to read
  another field is the same: the clears were placed for the old body's fields, and a reload keeps the
  instances ("methods added, removed or changed: no instance change", `SPEC:6354`).
- **fenced?** No: `@cached` is new in V5 (today E0104).
- **Rust**: no such attribute; a cache kept by hand in a `Cell` cannot hold a `&str` into its own object.
- **recommendation**: make it safe with a rule, nothing paid while running: a `sharedref mut` link, or a
  `shares mut` owner, made into anything a `@cached` method of its holder reads is refused (E2117 at the
  link: "`s.items[0]` is read by `Shelf.first_name`, which is `@cached` and would not see a change made
  through this link; link the shelf instead, `sharedref mut s`, or remove `@cached`"); a link to the
  whole `Shelf` is fine, since every write through it passes through `s`. And a reload that changes a
  `@cached` method's body clears every kept result of its type.
- **settled?** No verdict; not for the owner (it narrows a new word).
- **how sure**: medium: whether `sharedref mut s.items[0]` may sit beside a `@cached` reader is not said
  anywhere; nothing refuses it.

### F9. A `drop` that makes a new owner of its own object

- **feature**: an object "is ended the moment its last owner lets it go, before its `drop` runs"
  (`V5/rules.md:601-606`); `shares x` makes "one more owner" (`:167-168`); the 8-byte header holds a count
  and a type index (`:552-554`). Nothing refuses `shares this` in a `drop`, and the run-time check today
  makes (`SPEC:2640-2642` `[OBJ-5]`: a count above zero after the `drop` chain "panics in every profile")
  is not among V5's run-time checks (`V5/rules.md:974-1000`).
- **how it breaks safety**: kind 1 (use after free, then a second free).

  ```ember
  import std.mem

  static mut KEEP: Array[shares Player] = []

  class Player:
      name: String

      fn drop():
          KEEP.push(shares this)        # one more owner of the player being ended

  fn main():
      p = Player(name=String("ana"))
      mem.drop(p)                       # ends the player: its drop runs, then its block is freed
      println(KEEP[0].name)             # reads the freed block
  ```

  When `KEEP` ends, the count goes from 1 to 0 a second time and the block is freed again.
- **fenced?** Today: stopped while running (`[OBJ-5]`). V5: nothing refuses it or checks it.
- **Rust**: `Drop::drop` gets `&mut self` and cannot make an `Rc` of it.
- **recommendation**: make it safe with a rule, nothing paid while running: inside a `drop`, and in what
  it calls (by its summary), an owner made from `this` (`shares`, `shares mut`, `thread_shared`) is
  refused ("`drop` ends this `Player`; an owner made from it would point at a freed object; keep a copy
  of what you need: `NAMES.push(copy of this.name)`"); a `weakref this` stays allowed (the weak count keeps
  the block, and opening it finds the object ended). Or keep `[OBJ-5]`'s one compare after each `drop`.
- **settled?** No verdict (56 covers opening a `weakref` in a `drop`, not making an owner); not for the
  owner.
- **how sure**: medium: V5 never says whether `shares this` is allowed in any method; if it is, nothing
  stops this.

### F10. The callback trampoline's 16-bit generation rolls over

- **feature**: the checked trampoline: "C holds a tagged number ... with the callback's generation in its
  top 16 bits and its slot in its low 48 ... a call after the token ended ... stop[s] the program"
  (`V5/rules.md:906-911`; `V5/area-E.md:1903-1915`, which also fixes `0x4080` as this callback's top
  bits; the layout is unsettled, `V5/safety/check-V6.md:183`). Nothing says what happens when a slot's
  generation reaches its last value; `Pool` handles retire such a slot (`SPEC:3095-3097` `[HND-3]`).
- **how it breaks safety**: kind 4 (a check the rules promise is skipped). With AN-78's declarations
  (`V5/area-E.md:1876-1882`):

  ```ember
  class Game:
      score: i32 = 0

      fn add_point():
          this.score += 1

  unsafe extern "C":
      fn unregister(cb: extern "C" fn(*mut void), ud: *mut void)
      @ffi(param(cb, callback(retained, user_data=ud), threads(creator)), destroys(unregister))
      safe fn register_cb(cb: extern "C" fn(*mut void), ud: *mut void)
      safe fn poll_events()

  fn main():
      game = shares mut Game()
      for i in 0..65537:
          token = register_cb(fn() shares mut game: game.add_point())   # ends with each round
      poll_events()
      println(game.score)
  ```

  Each round's token ends, freeing slot 0, and the next round reuses it with the generation one higher.
  A C library with the bug the trampoline exists for (it kept the first `ud` and delivers an event to it
  in `poll_events()`) presents round 0's number; after 65,536 rounds slot 0's generation has come back to
  round 0's value and the function type is the same, so the trampoline runs the current callback instead
  of stopping with "C called the callback registered on line 16 after its token ended". The slot holds a
  live callback of the right type, so no memory is corrupted.
- **fenced?** No (V5's design; today's trampoline has no generations).
- **Rust**: no such feature; a C callback's `user_data` is a raw pointer used in `unsafe`.
- **recommendation**: settled by verdict 59's principle and `[HND-3]`: a slot whose generation reaches its
  last value is retired and never used again (48 bits of slots never run out); nothing paid per call.
- **settled?** Verdict 59 (as applied to `Pool` by `[HND-3]`); not for the owner.
- **how sure**: high that the rules say nothing; the harm is a wrong callback run, not memory.

### F11. A suspended generator across a hot reload of its body (unsure)

- **feature**: a generator's frame holds the locals that live across a `yield` and the point it is
  suspended at (`SPEC:2180-2181` `[CORO-4]`); a reload of "function-body changes" is admitted even in the
  `bodies` tier (`SPEC:6417-6419` `[HR-19]`); a reloadable function's address, method tables included, is
  its thunk's (`SPEC:6299-6303`). Neither Annex B nor `[CORO-*]` says what a suspended frame does after
  its `gen fn`'s body changes; the compiler's generator lowering (`compiler/ember_mir/src/lower/coroutines.rs`,
  `compiler/ember_analysis/src/coroutines.rs`) has no reload handling, and the reload code tracks
  closures' states (`compiler/ember_codegen_c/src/reload.rs:713`) but not generator frames.
- **how it breaks safety**: kind 1 (memory not yet written read as a value):

  ```ember
  from std.coroutine import Wait

  gen fn door_sequence() -> Generator[Wait]:
      yield Wait.NextFrame
      a = 1
      yield Wait.NextFrame
      println(a)
  ```

  A `Scheduler` (`SPEC:2206-2209` `[CORO-11]`) holds a frame suspended at the first `yield`. The edit
  puts `name = String("door")` above the first `yield` and `println(name)` after the second. If the
  frame's next resume runs the new code through the thunk, it reads `name` from a frame field the old
  code never wrote.
- **fenced?** Not that any line says, today or in V5.
- **Rust**: no hot reload in the language.
- **recommendation**: a frame keeps calling the resume code it was made with (old images stay mapped,
  `SPEC:6289` `[HR-4]`), or a reload that changes a `gen fn` body while frames of it live is refused in
  plan, naming the generator and the count, as `[HR-14]` does for a class removed while instances live.
  The same question for a lambda whose capture list changes while function values of it live.
- **settled?** No verdict; not for the owner.
- **how sure**: unsure: decided by whether a suspended frame resumes through the `gen fn`'s thunk.

### F12. Hot reload's reference rewriting does not list V5's new links (unsure)

- **feature**: `[HR-15]` rewrites, when an instance must move, "strong handles in live instances and
  registered containers, `Weak` handles, interior references in registered containers, and handles inside
  tuples, `Box` contents and closure captures" (`SPEC:6363-6366`); a reference it cannot enumerate refuses
  the reload (`:6367-6368`). V5 adds pointers this list does not name: the instance address a value made
  in an arena keeps, "a kept `sharedref mut` link to the instance" (`V5/rules.md:226-229`), and
  `sharedref mut` links held by values that live across a poll (a static's value).
- **how it breaks safety**: kind 1. `static mut GAME: Game` holds `current: LevelArena` inline, and
  `static mut LEVEL: Level` holds a mesh made `using GAME.current` (the arena of
  `V5/conformance/R3/accept_a_level_made_in_an_arena_the_caller_keeps.em`). An edit adds a field to
  `Game`; the static migrates (`SPEC:6390-6391` `[HR-17]`) and, if it must move, the mesh's instance word
  still names the old `GAME.current`; the mesh's next growth or its end calls `allocate`/`deallocate` on
  freed memory.
- **fenced?** Not that any line says.
- **recommendation**: `[HR-15]` lists every V5 link (`sharedref mut`, the arena-instance word, kept `ref`
  views of a static) among what commit rewrites, or plan refuses a reload that moves an instance or an
  object such a link may name; `@reinit_on_reload` (`SPEC:6392-6394`) on a static that such a link names
  is refused the same way.
- **settled?** No verdict; not for the owner.
- **how sure**: unsure: decided by whether a static may hold a value with a `sharedref mut` link into
  another static, and whether the runtime finds the instance word through the field's schema.

### F13. `@export("name")` can replace a C function the process already uses

- **feature**: `@export("symbol")` gives a function "a stable C symbol" (`SPEC:4945-4946` `[FFI-26]`),
  with no `unsafe`; the compiler checks clashes only among Ember's own symbols
  (`compiler/ember_typeck/src/lib.rs:786-788`). V5 keeps it (`V5/vocabulary.md:194`, "same").
- **how it breaks safety**: kind 5 and 6: C code, the runtime included, reaches an Ember function the
  rules do not see it calling, with arguments of another type.

  ```ember
  @export("free")
  pub extern "C" fn my_free(p: u64):
      println("free called")

  fn main():
      names = [String("a")]
      names.clear()      # the runtime's `ember_free` calls C's `free`, now bound to `my_free`
  ```

  On Linux with a shared C library, the program's `free` takes the place of the C library's for every
  caller in the process: no block is freed again, and the C library's own internal `free` calls run
  Ember code. `@export("memcpy")` with other behaviour corrupts memory. (With a static C runtime the
  linker may refuse the duplicate instead.)
- **fenced?** No, today or in V5.
- **Rust**: since edition 2024, `#[unsafe(no_mangle)]` and `#[unsafe(export_name = "...")]`: a symbol
  that clashes with another is undefined behaviour, so the attribute must say `unsafe`.
- **recommendation**: settled by verdict 60: `@export(...)` and `@export_table(...)` only on a declaration
  that vouches (the spelling, for example `unsafe @export("game_on_update")`, is the vocabulary step's), with
  E3100 otherwise; a check of the name against the C library's and the runtime's symbols is a useful
  extra but not general (a library loaded while running). Nothing paid while running.
- **settled?** Verdict 60; not for the owner (one word on each export, spelling for the vocabulary step).
- **how sure**: medium-high (the linker behaviour is the platforms'; the missing fence is certain).

## 3. Summary

Thirteen findings, most harmful first by what they break and how sure: F1 (reload at a checkpoint, use
after free), F2 (`static mut` from jobs, exports and a `drop` on another thread: races), F3 (clean-up
skipped by `MaybeUninit.write` and `alloc_nodrop`), F4 (loops of `shares` owners: leaks; for the owner),
F5 (`init` lets `this` out: unwritten memory read), F6 (weak counts changed by two scoped tasks), F7 (the
uncounted region link with `thread_shared` nodes), F8 (`@cached` not cleared through a link), F9 (`drop`
makes an owner of its own object), F13 (`@export` symbol clash), F10 (the trampoline's generation rolls
over: a check skipped), and two unsure ones, F11 (generator frames across a reload) and F12 (V5's new
links not rewritten by a reload). Every other attack the duty names is refused when compiling or made
safe, with its line, in section 1.
