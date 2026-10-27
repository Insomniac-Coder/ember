# 06 The checker: every finding of 01 to 05, tried against the rules

Agent 06 (checker), phase 2 of the unsafe-features inventory (verdict 60). Read in full: `inputs/brief.md`,
`JOB.md` (verdicts 1 to 98), `inputs/duties.md`, and `INV/01-design.md` to `INV/05-attacks.md`. To check
them: `V5/rules.md` in full; `V5/vocabulary.md` sections 3.1 to 3.8 in full; the parts of `V5/area-E.md`
the findings lean on (154-180, 898-912, 1405-1425, 1896-1918); the `V5/conformance/R15` test list and two
tests; `SPEC` Annex B (6240-6425), `[CT-5]` (4030-4042), `[ARN-1]`..`[ARN-8]`, `[ALC-2]` (2986-2998),
`[PAR-2]`/`[PAR-2a]`, `[HND-3]`, `[TIER-1]`, `[PHIL-11]`, `[CLS-2]`, `[OBJ-5]`, `[RT-10]`, `[FFI-11]`,
`[FFI-21]`..`[FFI-23]`, `[FFI-26]`..`[FFI-33]`, the import table (4720-4740), `[GPU-*]`; the library lines
`std/src/mem.em` 105-180, `std/src/sync.em` 207-261, `std/src/jobs.em` 23, `std/src/ffi.em` 160-190,
`std/src/ecs.em` 347-446; the compiler and runtime lines `ember_types/src/lib.rs` 1870-1882,
`ember_typeck/src/lib.rs` 54933-54995 (`alloc_nodrop`), 49930-49960 (`MaybeUninit.write`), 21340-21365
(the export boundary), the `unsafe interface` checks (7720-7729, 13981-13992, 14075-14082),
`ember_driver/src/cimport.rs` 1924-1932, `ember_rt.c` 3323-3340 (`ember_weak_retain`),
`ember_build/src/lib.rs` 1978-1988. Nothing was compiled or run. `INV` = `tasks/96/redesign/run4/v5/inventory/`.

Marks: **confirmed** (the rule lines allow the program and nothing found fences it; the reason is given),
**refuted** (the line that refuses it or makes it safe), **unsure** (what would decide it).

## 1. Where every finding went

| finding | merged into | mark |
|---|---|---|
| 01-1 C calling back outside the kept-callback list | C02 | confirmed |
| 01-2 hot reload against the compile-time facts | C08 | confirmed (its program does not reach the reload: see C08) |
| 01-3 arena instance reached through its `sharedref mut` link | C14 | confirmed |
| 01-4 loop of `shares` owners | C19 | confirmed, for the owner |
| 01-5 trampoline generation rolls over | C30 | confirmed |
| 01-6 weak count of a `thread_shared` object not said atomic | C22 | confirmed (wording) |
| 01-7 static re-entry when the lock's object is not known | C24 | confirmed (wording) |
| 01-8 scoped task reads a place the parent writes | X1 | refuted |
| 01-9 `static mut` without `unsafe` | X4 | refuted as a finding of its own (its holes are C02 and C08) |
| 01-10 C writing into Ember memory it keeps | C16 (part b) | unsure inside a confirmed entry |
| 01-11 `thread_shared fn()` "may hold copies" | X2 | refuted |
| 01-12 `@parallel` reads across chunks | X3 | refuted |
| 02-F1 foreign call re-enters uncounted Ember code | C02 | confirmed |
| 02-F2 `@export` name another symbol already has | C12 | confirmed |
| 02-F3 C function pointer callable with no `unsafe` | C07 | confirmed |
| 02-F4 borrowed callback C runs on several threads | C10 | confirmed |
| 02-F5 `Retained` token dropped while C holds the address | C16 (part a) | confirmed |
| 02-F6 exported function's `char` parameter | C28 | confirmed |
| 02-F7 stack probes only on Linux, none with `--emit c` | C29 | confirmed for `--emit c`; unsure for macOS |
| 02-F8 another package's overlay | C26 | confirmed |
| 02-F9 C source compiled into the program | C25 | confirmed |
| 02-F10 thread checks only in `debug` | C31 | confirmed |
| 02-F11 C++ header handle keyed to `@sync` | C34 | confirmed (stale text) |
| 02-B1..B19 the fenced features | not findings | (B12 `into_raw()` is fenced by verdict 91; B19 is X4) |
| 03-1 `[CT-5]` puts a `static mut`'s heap value in read-only memory | C17 | confirmed (wording) |
| 03-2 hot reload swaps a function whose facts changed | C08 | confirmed |
| 03-3 hot reload's migration predates V5's links and `using` | C18 | confirmed |
| 03-4 atomic owner count compared, then added | C23 | confirmed |
| 03-5 a draw call's counts unchecked | C27 | confirmed |
| 03-6 `alloc_nodrop` | C20 | confirmed |
| 03-7 `MaybeUninit.write` | C20 | confirmed |
| 03-8 a channel that holds one of its own ends | C21 (a `Sender`), C19 (a `Receiver`) | confirmed |
| 03-9 `[PHIL-11]` lists a loop of owners as an accepted leak | C19 | confirmed, for the owner |
| 03-10 `@reinit_on_reload` under kept views | U2 | unsure |
| 03-11 a suspended generator across a reload | U1 | unsure |
| 03-12 `[HASH-3]` names `RefCell` | X5 | refuted (03 said so too) |
| 04-L1 `Allocator` methods callable without `unsafe` | C01 | confirmed |
| 04-L2 hand-written query parameter makes `run_parallel` race | C09 | confirmed |
| 04-L3 a struct named `Cell_...` judged `Send` | C06 | confirmed |
| 04-L4 `jobs.submit` has no "any thread" word | C11 | confirmed |
| 04-L5 `Arena.alloc_nodrop` | C20 | confirmed |
| 04-L6 `MaybeUninit.write`, `write_at` | C20 | confirmed |
| 04-L7 a channel that holds one of its own `Sender`s | C21 | confirmed |
| 04-L8 runtime weak retain compares, then adds | C23 | confirmed |
| 04-L9 `Pool` keeps retired slots | C33 | confirmed (an understandable limitation) |
| 04-L10 `Volatile` texts omit alignment | C32 | confirmed |
| 05-F1 reload at a checkpoint | C08 | confirmed (its program is the one kept) |
| 05-F2 `static mut` from jobs, exports, a `drop` on another thread | X6 (jobs, exports), U3 (the `drop`) | refuted in part, unsure in part |
| 05-F3 `MaybeUninit.write`, `alloc_nodrop` | C20 | confirmed |
| 05-F4 loop of `shares` owners | C19 | confirmed, for the owner (its program is kept) |
| 05-F5 `init` lets `this` out before its fields are set | C05 | confirmed |
| 05-F6 two scoped tasks change one weak count | C03 | confirmed |
| 05-F7 the uncounted link of a walk, `thread_shared` nodes | C13 (and C22 for its wording note) | confirmed |
| 05-F8 `@cached` not cleared through a link | C15 | confirmed |
| 05-F9 a `drop` that makes a new owner of its own object | C04 | confirmed |
| 05-F10 trampoline generation rolls over | C30 | confirmed |
| 05-F11 generator across a reload | U1 | unsure |
| 05-F12 reload rewriting misses V5's links | C18 | confirmed |
| 05-F13 `@export("free")` | C12 | confirmed |

Totals after merging: 34 confirmed, 3 unsure, 6 refuted.

## 2. Confirmed, most harmful first

Harm is judged by what breaks (memory misused before a race, a race before a leak, a leak before a skipped
check that corrupts nothing), by how ordinary the program is, and by how sure the finding is.

### C01. `Allocator.deallocate` and `reallocate` are callable with no `unsafe` (04-L1)
- **Checked**: `std/src/mem.em` 121-124 declares `pub unsafe interface Allocator` with plain `fn allocate`,
  `fn deallocate`, `fn reallocate`; the compiler checks only that an `unsafe interface` is *implemented* in
  an `unsafe extend` (`ember_typeck` 7720-7729, 14075-14082), never a call of its methods. `Global` and
  `System` implement it (mem.em 134-179). A raw pointer is taken safely (`as_mut_ptr`). V5 keeps the
  interface "same" (vocabulary 340) and R3 only adds the compiler's own calls. **Confirmed, sure.**
- **Program kept**: 04-L1's (`Global().deallocate(p, Layout(size=4, align=1))` on a list's own buffer, then
  `xs[0]`, then the list's end frees it again). Kinds 1 and 6.
- **Rust**: `GlobalAlloc::alloc`, `dealloc`, `realloc` are `unsafe fn`.
- **Recommendation**: fence. The three methods become `unsafe fn` in the interface (E3100 outside: "`deallocate`
  gives memory back that Ember may still use, and a wrong `Layout` is undefined in C; write it inside `unsafe:`
  with a `# SAFETY:` note"); the calls the compiler makes for `using` are its own. `Layout` gets a checked
  constructor (align a power of two). No cost while running.
- **Settled by** verdict 60. Not for the owner (ordinary programs never call these).

### C02. C runs Ember code the foreign-call rule does not count (01-1, 02-F1)
- **Checked**: R12 (`rules.md` 745-756) and section 0 (86-89) count, at a foreign call, only callbacks handed
  through `callback(retained)` or `callback(once)`. Not counted: an `@export` function (kept "same",
  vocabulary 194; area E 1416-1420 limits only `threads=any|creator` exports, and only for `static mut`); an
  Ember override of a C++ virtual that C++ keeps (`SPEC` `[FFI-39d]` relied on run-time access checks, which
  V5 removes, `rules.md` 1023); a `callback(borrowed)` function still running (C may call it again from a
  nested foreign call); a capture-free Ember function given to a C function-pointer parameter with no
  `callback(...)` word (`[FFI-21]`). V5 makes `static mut` safe (vocabulary 134), so the re-entry needs no
  `unsafe` anywhere. **Confirmed, sure for exports, fairly sure for the other three.**
- **Program kept**: 02-F1's (`host_poll` calls the export `log_line` three times inside `for line in LOG:`; the
  pushes move `LOG`'s buffer and `println(line)` reads the freed block). 01-1's `LOG.clear()` form is the same.
  Kind 1; kind 6 when a `ref mut` parameter is live across the call (its noalias promise is false).
- **Rust**: a re-entrant global is a `static mut` (every access `unsafe`) or a `RefCell`/`Mutex`, which panics
  or deadlocks while running; never undefined behaviour in safe code.
- **Recommendation**: make it safe by a rule, nothing paid while running: a foreign call not marked
  `@ffi(no_callbacks)` counts as possibly running every `@export`/`@export_table` function whose thread
  contract allows this thread, every override of a C++-derived class C++ may hold, every `callback(borrowed)`
  function still running on this thread, and every Ember function whose address was given to C as an
  `extern "C" fn`; E3020/E3031 then refuse as for kept callbacks, the message naming the export.
- **Settled by** verdicts 31 and 39 (this is their rule, applied to every way C keeps Ember code). Not for the owner.

### C03. Two scoped tasks change a one-thread object's weak count at once (05-F6)
- **Checked**: making a link "is not a change of the target: any name may make one" (R11, 595-597); scoped
  tasks use the parent's variables with no word (R15 851); E7007 is about changes; E7015 covers `shares`
  owners and their links, not a one-owner class object; only `thread_shared` counts are atomic (R9 551).
  Area E's own thread argument (`area-E.md` 157-180) lists every place two threads can reach, and a one-thread
  object's header count is not among them. Today `[THR-11]` needed `Sync`, which a non-`@sync` handle lacked;
  V5 removes `@sync`. **Confirmed, fairly sure.**
- **Program kept**: 05-F6's first program (two tasks each `keep.push(weakref players[0])`; a lost +1 lets the
  `pop` free the block a remaining link still names). Kinds 2 then 1.
- **Rust**: `Rc::downgrade` needs an `Rc`, which is neither `Send` nor `Sync`.
- **Recommendation**: for the thread rules, making, ending or opening a `weakref` link of an object counts as
  touching its header; a scoped task may do it only for a `thread_shared` object (E7015, 05-F6's message). The
  same for the parent making one while a task may. Nothing paid while running.
- **Settled by** verdict 61's reason (what may run on another thread is said by the words); no verdict names it.
  Not for the owner.

### C04. A `drop` that makes a new owner of its own object (05-F9)
- **Checked**: an object is ended before its `drop` runs (R11 601-606); `shares x` makes one more owner (R3);
  nothing refuses `shares this`. Today `[OBJ-5]` panics on resurrection in every profile, and `[RT-10]`
  (6085) still names that check, but `[OBJ]` is a family V5 replaces and V5's list of what runs while the
  program runs (`rules.md` section 2) does not have it; verdict 56's "nor counted again" is stated only for a
  link opened during `drop`. **Confirmed, medium** (decided finally by whether V5 keeps `[OBJ-5]`'s compare).
- **Program kept**: 05-F9's (`KEEP.push(shares this)` inside `drop`, then `KEEP[0].name` reads the freed block
  and `KEEP`'s end frees it twice). Kind 1.
- **Rust**: `Drop::drop` gets `&mut self` and cannot make an `Rc` of it.
- **Recommendation**: refuse, when compiling, an owner made from `this` inside a `drop` or anything it calls
  (by its summary), with 05-F9's message; `weakref this` stays allowed. Nothing paid while running.
- **Settled by** verdict 56's intent; no verdict names this case. Not for the owner.

### C05. `init` may call a method that lets `this` out before its fields are set (05-F5)
- **Checked**: R5 (397-401) lets `init` call a method once every field *that method's body reads* is set, and
  E2100's V5 meaning (1165) is only that. Today `[CLS-2]` (1712-1718) refuses any use of `self` as a whole
  (a method call, passing, storing, capturing) before every field is set. A method that stores `weakref this`
  reads no field, so V5 lets `init` call it; a free function `init` then calls can read the unset field
  through that link. **Confirmed, medium** (an implementer might keep `[CLS-2]`'s list for the body).
- **Program kept**: 05-F5's (`enrol` pushes `weakref this` to `ROSTER`; `print_roster` reads `p.name` before
  `init` sets it). Kind 1 (memory not yet written; for a field of a class or range type, an invalid value).
- **Rust**: no object exists until every field has a value.
- **Recommendation**: the summary also records whether a method lets `this` out (stores, passes, captures or
  returns it, or a link or owner of it); such a method is called in `init` only after every field is set
  (E2100, 05-F5's message). Row 7's program still compiles.
- **Settled by** row 7 and `[CLS-2]`'s rule, which the relaxation must keep. Not for the owner.

### C06. A struct whose name starts with `Cell_` is judged `Send` by its first field (04-L3)
- **Checked**: `ember_types/src/lib.rs` 1875-1879 decides `Send` for any struct named `Cell_...` or `RefCell_...`
  from `fields[0]` alone; root-module names are unqualified, so a program's own struct matches. The C backend
  guards the same prefix by `declaring_module`; the thread rule does not. **Confirmed, sure (code read).**
- **Program kept**: 04-L3's (`struct Cell_pair:` with `pad: int` first and a `shares mut Counter`; two
  threads change the counter and its plain count). Kind 2. A compiler defect, which V5 keeps if the code
  is reused for E7015.
- **Rust**: `Send`/`Sync` come from the fields; `Cell` is `!Sync` by an `impl` on that type, not by its name.
- **Recommendation**: recognise compiler-made `Cell`/`RefCell` structs by `declaring_module == usize::MAX`, in
  every prefix test 04-L3 lists. No cost.
- **Settled by** the `Send` row (vocabulary 271: inferred from what a type holds). Not for the owner.

### C07. A C function pointer from a header is callable with no `unsafe` (02-F3)
- **Checked**: the importer maps every C function pointer to `Option[extern "C" fn(...)]` (`cimport.rs`
  1926-1929; `SPEC` 4733); calling an `extern "C" fn` value needs no `unsafe` (the library itself does,
  `std/src/ffi.em` 186, `destroy(self.pointer)` in a plain `drop`). `[UNS-1]` fences only a call of an
  `extern` function without a verified contract, and a pointer read from a struct, a result or a callback
  argument has no contract. V5 keeps `extern "C" fn` "same" (vocabulary 107). **Confirmed, sure.**
- **Program kept**: 02-F3's (`hooks.release(4096)` frees an address never allocated). Kinds 5 and 1.
- **Rust**: bindgen gives `Option<unsafe extern "C" fn(...)>`; a call needs `unsafe`.
- **Recommendation**: fence: an imported C function pointer is an `unsafe extern "C" fn(...)` (E3100 at a call
  outside `unsafe:`, 02-F3's message); an overlay may assert one safe, as `safe fn` does; a function pointer made
  from an Ember function stays a safe `extern "C" fn`; the destructor call of an `owns` foreign value is
  generated by the compiler from its asserted `destroys(f)`.
- **Settled by** verdict 60. Not for the owner.

### C08. Hot reload swaps a function whose worked-out facts changed, under code checked against the old ones (01-2, 03-2, 05-F1)
- **Checked**: `[HR-3b]` (`SPEC` 6272-6282) lets the frames below a checkpoint run their old code and checks
  only "the types whose layout the rest of the function depends on"; a call from an old frame to a reloadable
  function goes through its thunk to the new body (`[HR-5]`, `[HR-6]`); `[HR-14]` makes "methods added, removed
  or changed: no instance change". V5 puts every loop, view and call check on facts worked out from bodies (R5,
  R17) and checks nothing while running (R9 549-550, section 2's "Gone"). Hot reload needs no `unsafe` and
  V5 keeps it (vocabulary 192, 213-235). Today the same program is stopped while running (access words) and its
  `static mut` needs `unsafe`. **Confirmed, sure for the checkpoint; fairly sure for a poll whose callers sit
  in a package that is not rebuilt.**
- **Program kept**: 05-F1's. 01-2's program does not reach the reload as written: editing only `note` to
  `LOG.clear()` leaves `main`'s loop calling `note()` inside `for line in LOG:`, which version 2 refuses
  (E3020), and a version that does not compile is never loaded (`[HR-41]`, first row). 05-F1 and 03-2 also move
  the call out of the loop in version 2, so version 2 compiles while the old frame keeps walking `LOG`:

  ```ember
  import std.hot

  static mut LOG: Array[String] = [String("a"), String("b")]

  fn note():
      println("tick")              # version 2: LOG.push(String("c"))

  fn run():
      for line in LOG:
          std.hot.checkpoint()     # the reload is applied here
          note()                   # version 2 moves this line below the loop; the old frame still runs it
          println(line)            # reads LOG's old buffer, freed by the push

  fn main():
      run()
  ```
  Kind 1 (code loaded while running, kind 5, then use after free); the same for a parent-link read compiled
  with no test, a view parameter's call check, a lock's static re-entry, and a thread function's E7012.
- **Rust**: no hot reload in the language; `libloading` is `unsafe`.
- **Recommendation**: make it safe by a rule, nothing paid while running: `[HR-3b]`'s record for each call lists,
  beside the types, the R17 facts of every function the old frame may still call (what it may change, read
  and end; view parameters used after a change; locks; parent-link facts; thread facts). A reload that changes
  one of them for such a function is refused at that checkpoint (E9030, naming the function, as a layout
  change is); at a poll, a fact change of a function that code not rebuilt was checked against is refused the
  same way.
- **Settled by** verdict 60 and R17 (a changed fact is an interface change). Not for the owner (no program is
  written differently; some reloads wait or need a restart).

### C09. A hand-written query parameter makes `World.run_parallel` race (04-L2)
- **Checked**: `pub interface QueryParam` and `pub interface ParamOps` (`std/src/ecs.em` 417, 421) are ordinary
  interfaces any program may implement; `run_parallel` (347) trusts their `Reads`/`Writes` and `accesses()`; each
  system goes to a job through `SystemWork`, `Send` by hand, whose `# SAFETY:` note rests on those sets. V5 keeps
  `std.ecs` "same" (vocabulary 440). **Confirmed, fairly sure.**
- **Program kept**: 04-L2's (a `ParamOps` reporting no access whose `get_mut` adds components; two systems
  change one `World` from two jobs). Kinds 2 then 1.
- **Rust**: Bevy's `WorldQuery` and `SystemParam` are `unsafe trait`s.
- **Recommendation**: fence: `ParamOps` and `QueryParam` become `unsafe interface`s (implemented only by
  `unsafe extend`, E3100); `@derive(Component)` and the library's own parameters write them as today; the
  system check also asks `Fetch` and `View` for `Send`. No cost.
- **Settled by** verdict 60. Not for the owner.

### C10. A borrowed callback C may run on several threads at once changes a real variable (02-F4)
- **Checked**: a callback's thread contract defaults to any thread (`[FFI-22]`); R12's `thread_shared fn()` rule
  and R15's capture rule (877-882) are written for kept callbacks; area E (1900-1902) allows a plain capture of
  the real variable for `callback(borrowed)`, and V5's own test
  `conformance/R15/accept_a_borrowed_callback_uses_the_real_variable.em` accepts one that changes `count` with no
  `threads(...)` word. Nothing asks whether C may call it on several threads at once. **Confirmed, fairly sure.**
- **Program kept**: 02-F4's (`visit_each` calls the callback from four threads; `seen.push` races and the
  buffer is freed under another push). Kinds 2 then 1.
- **Rust**: a safe wrapper of such a C function takes `F: Fn(i32) + Sync`; `seen.push` in an `Fn` is refused.
- **Recommendation**: a callback whose contract lets C call it from any thread is a `thread_shared fn()`
  (verdict 61): it changes real variables only through `Atomic`s or locks; with `threads(creator)` or
  `threads(main)`, checked by the trampoline in every profile, it may change them. V5's test above then writes
  `threads(creator)` in its `@ffi(...)`. E7015 with 02-F4's message. No cost.
- **Settled by** verdict 61. Not for the owner (only the asserted declaration gains a word).

### C11. `jobs.submit` takes a plain `fn()`, so a one-thread owner can reach a pool thread (04-L4)
- **Checked**: `std/src/jobs.em` 23 is `pub fn submit(owned f: fn()) -> JobHandle`. Today the compiler checks the
  captures in a built-in (`synth_jobs_submit`, E7004). V5 changes only `thread.spawn`'s line to
  `thread_shared fn()` (R15 845-847; verdict 94) and says nothing of `jobs.submit` or the pool functions of
  `@parallel` and `run_parallel`. Area E 1416-1420 does say a job is another thread for `static mut` (see X6),
  so the design knows; the library line does not. **Confirmed, likely** (a generous reading of R15's
  "a one-thread owner reaching another thread is E7015" might cover it).
- **Program kept**: 04-L4's (`jobs.submit(fn() shares mut count: count.n += 1)` beside `count.n += 1` in
  `main`). Kind 2.
- **Rust**: `rayon::spawn` and `thread::spawn` need `F: Send + 'static`.
- **Recommendation**: `pub fn submit(owns f: thread_shared fn()) -> JobHandle:` (E7015 as for `thread.spawn`);
  `JobScope.submit` stays a plain `fn()` as `scope.spawn` does.
- **Settled by** verdicts 61 and 94. Not for the owner.

### C12. `@export("name")` can replace a C library or runtime function (02-F2, 05-F13)
- **Checked**: `@export` is an ordinary attribute (`SPEC` `[FFI-26]` 4945), kept "same" (vocabulary 194); the
  compiler checks clashes only among Ember's own symbols (`ember_typeck` 786-792) and refuses `main`.
  **Confirmed, sure that nothing fences it; fairly sure of the link behaviour** (Linux interposes; a static C
  runtime may refuse the duplicate).
- **Program kept**: 02-F2's (`@export("realloc")` returning its argument: every list growth writes past a
  one-element block). 05-F13's `@export("free")` is the same family. Kinds 5, 6, 1, 3.
- **Rust**: edition 2024 makes `no_mangle` and `export_name` unsafe attributes (`#[unsafe(no_mangle)]`).
- **Recommendation**: fence: an export names its symbol only in an unsafe form of the attribute with a
  `# SAFETY:` note (spelling for the vocabulary step, for example `@unsafe(export("game_on_update"))`), E3100
  otherwise; also refuse outright the names the compiler knows collide (`ember_`-prefixed runtime names, the C
  standard library's), as `main` is today. Covers `@export_table` too.
- **Settled by** verdict 60. Not for the owner (one word on each export; the spelling is the vocabulary step's).

### C13. The uncounted link of a walk's deferred-drop region, over `thread_shared` nodes (05-F7)
- **Checked**: R11 651-657 lets a walk whose node class has no `drop` run in a per-thread deferred-drop region
  where `up = weakref p` holds no weak count; the only condition stated is "no `drop` method of its own". The
  parent-link proof excludes `thread_shared` objects (617), this paragraph does not, and a per-thread region
  cannot hold back a block another thread frees. **Confirmed, medium** (the program needs a timing; the text
  plainly allows the link).
- **Program kept**: 05-F7's (another thread empties `root.kids` between the walk's steps; `mid` is freed with
  no weak count held by `up`; the next `while up exists as n:` reads the freed header). Kind 1.
- **Rust**: a `Weak` of an `Arc` always holds an atomic weak count.
- **Recommendation**: the region and its uncounted link apply only to objects no other thread can end (not
  `thread_shared`, not reachable from one). A `thread_shared` walk pays the count moves. See C22 for the
  atomic weak count.
- **Settled by** R11's own reason (the region keeps what this thread ends); no verdict. Not for the owner.

### C14. An arena's instance may be changed through its owner while values made in it live (01-3)
- **Checked**: R3 (226-229) makes the instance address each value keeps "a kept `sharedref mut` link to the
  instance ...: the program may use the instance while values made in it live"; through `sharedref mut` any
  holder may change it (R7 443-444), and writing a number is not a reshape (section 0, 48). So a safe method that
  rewinds the arena's counter runs while values made in it live. Today `[ARN-1]`/`[ARN-7]` make every rewinding
  operation take `mut self`, "so no view survives them"; V5 has no exclusive receiver (`this` is never alone, R5)
  and the only sound spelling left to an arena author is `owns this`. Implementing `Allocator` is `unsafe extend`,
  so the author vouched; but `[ALC-2]`'s promise (2997-2998: "valid, aligned, unaliased memory") does not say
  until when, and the method that breaks it is ordinary safe code. **Confirmed, likely.**
- **Program kept**: 01-3's (`frame.reset()` between two lists made `using frame`; both get one buffer; the
  string is freed twice). Kind 1.
- **Rust**: `bumpalo::Bump::reset(&mut self)` cannot run while a `Box<T, &Bump>` borrows it.
- **Recommendation**: a rule, nothing paid while running: while a value made in an instance lives, the instance
  is changed only through the `Allocator` methods that value's growth and end call (E3021 at any other change,
  01-3's message); reading it stays allowed. `[ALC-2]`'s promise says a block stays valid until it is given back
  or the instance ends.
- **Settled by** verdicts 24 and 47 (where the instance is named, that a value keeps its address), not the kind of
  that link; verdict 60. Not for the owner.

### C15. A `@cached` result is not cleared by a change made through a link (05-F8)
- **Checked**: `@cached` is "cleared by every change, through any name, to a field its body reads" (vocabulary
  409), by "one store (clear the flag) at each place that may change" such a field (`area-B.md` 877, as 05 quotes).
  A change through `t.target`, a `sharedref mut` link to an item of the shelf's list (allowed for a class object
  in a list, verdict 81), does not pass through the shelf, so no clear can be placed for it; E2117 refuses only a
  cached method that *reads* a link field. 01's dismissal (`@cached` "after a change through a link" not a
  finding) covers links the method reads, not a link another object holds. **Confirmed, medium.**
- **Program kept**: 05-F8's (the kept `StrView` of `"apple"` is returned after the text is replaced and freed;
  with an `int` result, a stale 5). Kind 1 for a view result; for a number a wrong value only.
- **Rust**: no such attribute; a hand cache in a `Cell` cannot hold a `&str` into its own object.
- **Recommendation**: refuse a `sharedref mut` link or a `shares mut` owner made into anything a `@cached` method
  of its holder reads (E2117 at the link, 05-F8's message); a link to the whole holder is fine. A reload that
  changes a `@cached` body clears its type's kept results (with C08).
- **Settled by** verdict 60 (the word is 07's, its refusals E2117); no verdict names this. Not for the owner.

### C16. C keeps the address of an Ember object that safe code may free or change (02-F5; 01-10)
- **Checked (a)**: `[FFI-23]` (4923-4929): `Retained.pin(v)` hands an object to C; "Dropping the token releases the
  object"; for an object no contract word says when C stops using it (callbacks have `destroys(f)`, objects
  nothing). Today's test `tests/conformance/FFI-23/accept_retained_lets_foreign_code_keep_ember_memory.em` with two
  lines swapped frees the point under C. V5 says `Retained` "keeps an object on Ember's side while C holds a
  callback; it gives no object away" (R16 927-928, verdict 91's correction) but keeps `Retained.pin(v)` "same"
  (vocabulary 268) and the `retained` contract word (369). **Confirmed for today, sure; V5's text leaves it open.**
- **(b) unsure**: 01-10's case, C writing through a kept changeable pointer during a later foreign call while
  Ember holds a `ref mut` (noalias) or read-only view of the same object, depends on whether V5 keeps the object
  form and how a program reaches the pinned value; decided by (a)'s fix.
- **Program kept**: 02-F5's. Kind 1 (and kinds 5, 6 for (b)).
- **Rust**: `Box::into_raw` is safe (a leak), `Box::from_raw` is `unsafe`; the token drop is the `from_raw`
  step without `unsafe`.
- **Recommendation**: as verdict 91 says, remove the object form: C keeps an object by an `owns` hand-over to a
  parameter declared `owned`, or by `into_raw()` inside `unsafe:`, and gives it back with `adopt` inside
  `unsafe:`; `Retained` stays for callbacks. If the object form stays, it takes the callback's rule: a
  `retained` object parameter names `destroys(f)`, which the token's end calls first (E5041 without it), and a
  foreign call counts as possibly changing every object C keeps a changeable pointer to (for R4 and R8).
- **Settled by** verdict 91. Not for the owner.

### C17. `[CT-5]` would put a `static mut`'s compile-time heap value in read-only memory (03-1)
- **Checked**: `[CT-5]` (`SPEC` 4030-4040) materialises a static's heap value in read-only memory when "its type
  gives Safe code no way to obtain `ref mut` ... (no `Mutex`, `RwLock`, `Cell`, `RefCell` or `Atomic` on the path)":
  a test of the type only. V5 makes `static mut LOG: Array[String] = [...]` safe (vocabulary 134), whose type
  has none of those, and no V5 line says where its value lives. Today the compiler refuses every Ember
  `static mut`, so the gap opens only with V5. **Confirmed (wording); high harm if implemented as written.**
- **Program kept**: 03-1's (`LOG.push` reallocates constant data; `LOG[0] = ...` frees it). Kinds 1 and 6.
- **Rust**: no compile-time heap values; a `static` with a `Mutex<Vec<_>>` starts empty.
- **Recommendation**: `[CT-5]`'s second bullet names "a `static mut`, and any static whose memory a safe
  operation writes": its value is built on first access into ordinary heap memory (`[STA-3]`). No cost beyond
  that bullet's.
- **Settled by** verdicts 8 and 60. Not for the owner.

### C18. Hot reload's migration does not know V5's `using` instance word or its new links (03-3, 05-F12)
- **Checked**: `[HR-2a]` builds every migrated instance in the transaction arena; `[HR-15]` rewrites, when an
  instance moves, a list that names today's handles only (strong, `Weak`, interior references, tuples, `Box`,
  captures); `[HR-15a]` refuses only references it "cannot enumerate". V5 adds a hidden instance address in every
  value made `using` an arena (R3 226-229, verdict 47) and `sharedref mut` links. A frame's references are
  covered (`[HR-3b]` refuses moving an object a frame references), and a class field cannot hold a
  `sharedref mut` without `from` (R7, E2118), so (a)'s field case is narrower than 03 wrote; what stays is (b): a
  migrated value made `using current` is rebuilt in heap memory, and its end hands `current.deallocate` a block
  `current` never gave out; and 05-F12's instance word of a value in a static whose arena lives in another
  migrating static. **Confirmed for (b), sure; 05-F12's path likely.**
- **Program kept**: 03-3 (b) (verdict 24's `boss` with a field added to `Mesh`). Kind 1.
- **Rust**: no counterpart.
- **Recommendation**: a migrated value made `using` an instance is rebuilt by that instance (its header names
  it), or plan refuses moving one; `[HR-15]` lists the instance word and every V5 link and kept view among what
  commit rewrites, or plan refuses moving what one may name.
- **Settled by** verdicts 24, 47 and 60. Not for the owner.

### C19. A loop of shared owners leaks, with only a warning (01-4, 03-9, 05-F4; the `Receiver` half of 03-8)
- **Checked**: L3001 is "a warning: a leak, not a memory error" (`rules.md` 658-660, 1123-1125, 1210); today
  `[WK-6]` "never changes whether a program is accepted" and `[PHIL-11]` (224-228) lists "a reference cycle
  leaking" as accepted. L3001 speaks only of `shares` fields; a function value's state, `thread_shared` owners
  and a channel whose queue holds its own `Receiver` make the same loop. **Confirmed, sure.**
- **Program kept**: 05-F4's (`a.next = Some(shares mut b)`, `b.next = Some(shares mut a)`, a million times).
  Kind 3, the brief's own example.
- **Rust**: `Rc`/`Arc` cycles leak in safe Rust; Rust counts leaks as safe; `Weak` is the way out.
- **Recommendation, for the owner**, one of:
  (a) refuse a type whose owning links (`shares`, `shares mut`, `thread_shared`, function values' state, channel
  ends) can lead back to itself unless one link on the loop is `weakref` (or, 05's refinement, a `let` field set
  when the object is made, which can only name objects that exist already), at the field that closes it, with
  01-4's message; nothing paid while running, but a graph whose nodes share each other writes its back links as
  `weakref`;
  (b) keep the warning, as Rust, and write in R16 that a leak through a cycle is the one safety promise kept
  outside `unsafe` only as a warning (verdict 60 then has an exception, and `[PHIL-11]` keeps its line);
  (c) keep the warning, and require `unsafe` only to silence it.
  Recommended: (a), by verdict 58's reason ("what is so good about leaving a count higher and causing a leak").
- **Settled by**: no verdict (58 and 59 are about `forget` and roll-over). **For the owner**: it changes how
  ordinary programs with shared back links are written.

### C20. A value's clean-up is skipped with no `unsafe`: `Arena.alloc_nodrop`, `MaybeUninit.write` (03-6, 03-7, 04-L5, 04-L6, 05-F3)
- **Checked**: `ember_typeck` 54981 gives E3090 only `if method == "alloc"`, and `alloc_nodrop` has no `unsafe`
  test; `MaybeUninit.write` (49935-49947) has none, only `assume_init` does (49949-49959); `[ARN-2]`, `[ARN-8]`
  say the clean-up never runs. V5 keeps both "same" (vocabulary 321, 326), against its own R1 (114-115: "nothing
  ends an owner without it"). **Confirmed, sure.**
- **Program kept**: 05-F3's (`slot.write(shares p)` three times: "player freed" never prints). Kind 3, and the
  count left high that verdict 58 removed `mem.forget` for.
- **Rust**: `MaybeUninit::write`, `ManuallyDrop`, `mem::forget` are safe; Rust counts leaks as safe.
- **Recommendation**: fence, as verdict 91 fenced `into_raw()`: `alloc_nodrop`, and `write`/`write_at` of a `T`
  that needs clean-up, only inside `unsafe:` (E3100, 03-6's and 04-L6's messages); `write` of a `T` with
  nothing to clean up stays safe. Later, optionally, an arena that runs the drops of what it holds at `reset()`
  (as `typed_arena`), so the ordinary case needs no `unsafe`.
- **Settled by** verdicts 58 and 60. Not for the owner (04-L5 notes a game keeping `String`s in a frame arena
  gains an `unsafe:` line; verdict 58 already ruled on that trade).

### C21. A channel kept alive by a `Sender` queued in itself (03-8, 04-L7)
- **Checked**: `Receiver.drop` (`std/src/sync.em` 255-261) marks the receiver gone and wakes senders but leaves
  the queue full; the state is freed only when its last owner ends, and a queued `Sender` is one.
  **Confirmed, sure.**
- **Program kept**: 04-L7's (`tx.send(copy of tx)` then both ends end). Kind 3.
- **Rust**: `std::sync::mpsc` drops queued messages when the receiver disconnects.
- **Recommendation**: `Receiver.drop` takes every queued value out under the lock and ends it after the lock is
  let go. A `Receiver` sent into its own channel is C19's loop.
- **Settled by** verdict 60. Not for the owner (only values no program can receive end earlier).

### C22. The weak count of a `thread_shared` object is not said to be atomic (01-6; 05-F7's wording note)
- **Checked**: R9 551 ("`thread_shared` counts are atomic"), section 0 (66-68) and area E's item 5 (175-176) speak
  of the owner count; R11 591-595 and section 2's row (984) never say the weak count of a `thread_shared` object
  is changed atomically, while R12 (696-697) lets functions on any thread make and end such links. Today's runtime
  does it atomically for `@sync` objects (`ember_rt.c` 3335-3336). **Confirmed (wording); harm only if
  implemented as plain adds.**
- **Program kept**: 01-6's (two tasks make and end `weakref p` a million times). Kinds 2 then 1.
- **Rust**: `Arc`'s weak count is atomic.
- **Recommendation**: R9, R11 and section 2: "both counts of a `thread_shared` object, its owners and its
  `weakref` links, are atomic".
- **Settled by** verdict 59 and R9's intent. Not for the owner.

### C23. An atomic count compared with its maximum, then added to, can be raced past it (03-4, 04-L8)
- **Checked**: R9 555-558 says "before each +1 ... it is compared with its maximum"; for an atomic count a compare
  then an add are two steps. Today's `ember_weak_retain` (`ember_rt.c` 3323-3340) loads, refuses `UINT32_MAX`,
  then `fetch_add`s: two threads at `UINT32_MAX - 1` both pass and the count wraps to 0. `[RT-10]` requires one
  `fetch_add` for a strong copy, never a compare-exchange loop. **Confirmed for the wording and for today's weak
  path; needs about 4 billion links, or verdict 96's 8-bit test build.**
- **Program kept**: 04-L8's (in the 8-bit test build). Kind 1 through a check skipped.
- **Rust**: `Arc::clone`/`Weak::clone` do one `fetch_add`, then abort when the old value is above `isize::MAX`,
  half the range from wrapping; `Arc::downgrade` uses a compare-exchange loop.
- **Recommendation**: for `thread_shared` counts only, one `fetch_add`, then stop when the old value is at a limit
  kept below the maximum by more than any number of threads that could race (Rust's way, keeping `[RT-10]`'s
  single add); `shares` counts (one thread) keep the exact compare. The stop message of a `thread_shared` object
  then names that limit; verdict 96's 8-bit tests, if they use a one-thread object, are unchanged.
- **Settled by** verdicts 59 and 96 (how to apply them). Not for the owner (no ordinary program comes near).

### C24. "Static re-entry" when the held lock may or may not be the callee's (01-7)
- **Checked**: R15 886-887 lets a direct call enter "past the lock it already holds"; area E (906-909) says "a lock
  this thread is known to hold", and its test uses the same name (`log.push("a")` inside `with log.lock:`). For
  `other.push(...)` inside `with held.lock:`, where `other` may be `held`, neither text says what happens: entered
  past by kind, two threads change `b.lines` (a race); taken normally, `add_to(a, a)` waits for ever, which verdict
  43 promises to refuse when compiling. **Confirmed (wording), low to medium.**
- **Program kept**: 01-7's. Kind 2 (by-kind reading) or kind 4 (verdict 43's check skipped).
- **Rust**: no static re-entry; a `Mutex` taken twice deadlocks or panics.
- **Recommendation**: R15: a direct call enters past a lock only where the compiler proves it is the very lock held
  (the same name, or `this` of the object whose lock is held); where it may be and may not be, the call is refused
  (E7018, 01-7's message, the way out `if other is not held:`).
- **Settled by** verdict 43. Not for the owner.

### C25. C source compiled into the program runs with no `unsafe` (02-F9)
- **Checked**: `import c "./bridge.c"` and `implementation = [...]` compile C into the program (`[BLD-FFI-1]`,
  `[FFI-29b]`) with none of `[TIER-1]`'s three boundaries; a constructor there runs before `main`. **Confirmed, sure.**
- **Program kept**: 02-F9's. Kind 5.
- **Rust**: not fenced either (a build script with `cc`).
- **Recommendation**: `unsafe import c "./bridge.c"` and the same for an import with `implementation = [...]`;
  `link = [...]` stays listed by `ember tcb`. An import of a header alone is unchanged.
- **Settled by** verdict 60. Not for the owner (only programs that compile C gain one word).

### C26. Another package's overlay applied to a header it was not written for (02-F8)
- **Checked**: `[FFI-30c]` lets an import list a dependency's overlay; `[TIER-1]` (151-155) requires the `unsafe`
  boundary "in the same package"; `[FFI-12]` checks an overlay against a header by names and parameter counts only.
  **Confirmed (a spec conflict); the version-mismatch path is a scenario, not a test.**
- **Program kept**: 02-F8's description. Kinds 5 and 1.
- **Rust**: a `-sys` crate builds its bindings against its own header.
- **Recommendation**: an overlay records the hash of the header it was written for (`.embind` holds one); applying
  it to another header is an error naming both; or the import line naming another package's overlay says `unsafe`.
- **Settled by** `[TIER-1]` and verdict 60. Not for the owner.

### C27. A draw call's counts are not checked against the buffers it reads (03-5)
- **Checked**: Annex D's draw example (`SPEC` 6690-6697) is safe code; `[GPU-1]` checks handles, `[GPU-4]` writes
  in flight, `[GPU-5]` access declarations; no rule compares a draw's ranges with the bound buffers. `std.gpu`
  is not built (`std/src` has no `gpu.em`). **Confirmed (spec), sure.**
- **Program kept**: 03-5's (`cmd.draw_indexed(1000000)` on a three-vertex buffer). Kind 5.
- **Rust**: `wgpu` validates every draw's ranges; `ash` makes the raw call `unsafe`.
- **Recommendation**: the command list checks each draw's and dispatch's ranges against the bound buffers (one
  compare per draw, a stop with the numbers), and asks for robust buffer access where the API has it.
- **Settled by** verdict 60. Not for the owner.

### C28. An exported function's `char` parameter takes any `uint32_t` from C (02-F6)
- **Checked**: the export boundary refuses range types (E5054, `ember_typeck` 21348-21365) for the reason that C
  could "manufacture a range value that passed no check"; `char` crosses as `uint32_t` with no check, though
  `[UNS-4]` says a surrogate `char` cannot exist. **Confirmed, fairly sure.**
- **Program kept**: 02-F6's (`letter_len(0xD800)` from C). Kind 1 (a value of the wrong type).
- **Rust**: `char` in an `extern "C"` definition draws `improper_ctypes_definitions`.
- **Recommendation**: refuse `char` at a foreign boundary as range types are (E5054, 02-F6's message: take `u32`
  and convert with `char.from_u32`).
- **Settled by** verdict 60 and E5054's own reason. Not for the owner.

### C29. Stack probes: none with `--emit c`; macOS unknown (02-F7; 05's row 4 said safe)
- **Checked**: `ember_build/src/lib.rs` 1985 passes `-fstack-clash-protection` only `if cfg!(target_os = "linux")`
  (the comment: "MSVC and clang-cl probe by default"); `--emit c` writes the C with no flags, while `[RT-12]` and
  `[PHIL-10]` promise probes in every build. **Confirmed for `--emit c` (a build system using upstream GCC or
  Clang, which do not probe by default, loses the guarantee); unsure for macOS, decided by Apple Clang's `-O2`
  assembly of 02-F7's `fill` (a probe loop or `___chkstk_darwin`, or none).**
- **Program kept**: 02-F7's (a 32 KB frame per call). Kind 1.
- **Rust**: rustc emits inline stack probes itself on its tier-1 targets.
- **Recommendation**: pass the probe flag on every target whose compiler takes it, refuse a toolchain that cannot
  probe (as E9011 does for contraction), and write the required flags beside `--emit c` output.
- **Settled by** `[RT-12]`, `[PHIL-10]`. Not for the owner.

### C30. The trampoline's 16-bit generation can come round again (01-5, 05-F10)
- **Checked**: R15 906-911 and area E 1903-1907 put the generation in the top 16 bits and check it, and say nothing
  of a slot whose generation is used up; check-V6 183 marks the tag's layout unsettled. `[HND-3]` retires such a
  slot for `Pool`. **Confirmed (the rules are silent); the harm is a promised stop skipped, no memory misused.**
- **Program kept**: 05-F10's (65,536 registrations reusing slot 0, then C's stale tag runs the current callback).
  Kind 4.
- **Rust**: no counterpart (a raw `user_data` pointer).
- **Recommendation**: retire a slot whose generation reaches its last value (one compare per registration, nothing
  per call).
- **Settled by** verdict 59's principle and `[HND-3]`. Not for the owner.

### C31. Two thread checks run only in the `debug` profile (02-F10)
- **Checked**: `[FFI-33a]`, `[FFI-33b]` checks compiled only with `EMBER_RT_DEBUG_ACCESS`, which only `debug`
  defines; `[PRF-1]` promises the same safety checks in every profile. They back facts asserted in `unsafe`
  declarations. **Confirmed, low harm.**
- **Recommendation**: run them in every profile, or say they are debug diagnostics of an asserted fact and list
  them in `[PRF-3]`.
- **Settled by** `[PRF-1]`. Not for the owner.

### C32. `Volatile`'s `@safety` texts do not ask for alignment (04-L10)
- **Checked**: `std/src/mem.em` 27, 32, 38: "the pointer is valid for a read/write of a `T`". Inside `unsafe`;
  the gap is the text. **Confirmed.** Kind 6.
- **Recommendation**: "valid for a read (write) of a `T` and aligned to `T`'s alignment".
- **Settled by** verdict 60 (the caller's promise must be stated). Not for the owner.

### C33. A `Pool` keeps every retired slot (04-L9)
- **Checked**: `collections.em` 1464-1476 retires a slot whose generation runs out (`[HND-3]`), one per 4096 reuses
  with `Split20`. **Confirmed; bounded, and the retirement is itself the safety measure.** Kind 3, bounded.
- **Recommendation**: keep, as an understandable limitation (JOB.md), and say it in `Pool`'s and `Split20`'s
  documentation. C30's fix makes the trampoline the same.
- **Settled by** the JOB's "understandable limitation". Not for the owner.

### C34. The C++ header's copyable handle is keyed to `@sync`, which V5 removes (02-F11)
- **Checked**: `[BLD-FFI-5a]` (6475-6477) and ODR-162 name `@sync`; vocabulary 178 removes it. Stale text; no
  program can be written until it is restated. **Confirmed (text).**
- **Recommendation**: a copyable handle only for a `thread_shared` value's type; keep `[FFI-31b]` and decide
  ODR-162 against it.
- **Settled by** verdict 61 and the removal of `@sync`. Not for the owner.

## 3. Unsure

- **U1. A suspended generator across a reload of its body (03-11, 05-F11).** No line of Annex B or `[CORO-*]` says
  what a suspended frame does after its `gen fn` body changes; `reload = "bodies"` admits body changes with no
  schema (`[HR-19]`). Decided by: whether a stored generator resumes through its `gen fn`'s thunk (then the new
  state machine runs on the old frame: kind 1) or keeps the code it was made with (`[HR-4]` keeps old images
  mapped), and whether a generator's frame type has a schema plan compares. If confirmed: refuse in plan a body
  change that changes a live frame's layout or states, or keep old resume code for old frames (verdict 60; not for
  the owner).
- **U2. `@reinit_on_reload` discards a static that a view may point into (03-10).** For V5, a class field's kept
  view `from static` is only text written in the program (R7 434), and a static's change waits for a poll, when no
  frame runs Ember code (`[HR-3]`, `[HR-3b]`'s last sentence), so 03-10's program is closed. Decided by: whether V5
  lets a static's value hold a view or a `sharedref mut` link into another static's heap parts. If it does: plan
  refuses `@reinit_on_reload` on a static such a link may name.
- **U3. A `thread_shared` object's `drop` run on another thread through a weakref-opened count (05-F2's third case;
  03's note in passing).** R5 (369-374) counts ending an object as a call of its `drop` "wherever an object of its
  type may end", which covers a thread that holds an owner; R10 point 2 gives a view opened from a `weakref` of a
  `thread_shared` object an atomic count whose end may be the last, and R5's list does not name that end. Decided
  by: whether the count a view holds (R10 point 2) counts as "may end" for R5. If not: 05-F2's E7012 at the line
  that makes a `thread_shared` object whose class's `drop` reaches a main-thread `static mut`.

## 4. Refuted

- **X1. A scoped task reads a place the parent writes (01-8).** R15 852-853: "Every task's effects, worked out from
  its body (R5), are checked against the other tasks and the parent until the join"; area E's safety argument,
  item 1 (`area-E.md` 160): "a place a scoped task reads: the parent and the other tasks may not change it until the
  join". Only E7007's one-line meaning (`rules.md` 1200) omits the read; adding "or one reads a place another
  changes" there is wording.
- **X2. `thread_shared fn()` "may hold copies" of a struct with a `shares` field (01-11).** R15 897-898: "A one-thread
  owner or a lent value reaching another thread is E7015"; the `Send` row (vocabulary 271): "nothing reachable from it
  is a one-thread owner". 01 says the design refuses it; R12's sentence may say "copies with nothing one-thread in
  them" (wording).
- **X3. `@parallel` reads across chunks (01-12).** The `@parallel` row is "same" (vocabulary 182), so `[PAR-2]` and
  `[PAR-2a]` stand: "A violation of clause (b) is `E7011`, `parallel loop has a loop-carried dependency`" (`SPEC`
  3590); E7011 is not in V5's removed list (`rules.md` 1216-1218).
- **X4. `static mut` without `unsafe` (01-9) as a finding of its own.** Verdict 8 keeps it; E7012 and the
  compile-time rules make it safe inside Ember code; its holes are C02 and C08, where they are counted.
- **X5. A key's `hash` that replaces the map it is inserted into (03-12).** R4 316-325 (the call check, a method's
  object included) with R12 720-721 (a generic body's calls of its type parameters' methods make their changes per
  instance): `SEEN.insert(Key(n=1), 1)` is E3031. `[HASH-3]`'s text should name R4 instead of `RefCell` (wording).
- **X6. `static mut` reached from a job or a `threads=any` export (05-F2, first two cases).** R15 899-901: "reached only
  by the main thread and its scoped tasks unless it is `guarded by` a `static` lock (E7012)"; `area-E.md` 1416-1420:
  "A function reachable from `thread.spawn`, an unscoped job of the job system, a C callback whose contract is not
  `threads(main)`, or an `@export(threads=any|creator)` may not read or change it (E7012)"; test
  `conformance/R15/reject_an_any_thread_export_changes_a_static_mut.em`. (A one-thread owner captured by a job is C11.)

## 5. The confirmed findings in order of harm, with recommendation and owner flag

| # | finding | kind | recommendation | settled by | for the owner |
|---|---|---|---|---|---|
| C01 | `Allocator.deallocate`/`reallocate` callable without `unsafe` | 1, 6 | fence: `unsafe fn` in the interface; checked `Layout` | 60 | no |
| C02 | C runs exports, C++ overrides, running borrowed callbacks, uncounted | 1, 6 | count them at every foreign call not `no_callbacks` | 31, 39 | no |
| C03 | two scoped tasks change a one-thread object's weak count | 2, 1 | making/opening a link touches the header: only `thread_shared` objects in tasks | 61 (reason) | no |
| C04 | `drop` makes a new owner of its own object | 1 | refuse an owner made from `this` in a `drop` | 56 (intent) | no |
| C05 | `init` calls a method that lets `this` out early | 1 | such a method only after every field is set (E2100) | row 7, `[CLS-2]` | no |
| C06 | `Cell_...` struct judged `Send` by its first field | 2 | recognise the compiler's structs by module, not name | `Send` row | no |
| C07 | imported C function pointer callable without `unsafe` | 5, 1 | import as `unsafe extern "C" fn` | 60 | no |
| C08 | hot reload swaps a body whose facts changed | 5, 1 | plan compares R17 facts for live frames and unrebuilt callers | 60, R17 | no |
| C09 | hand-written ECS query parameter races | 2, 1 | `unsafe interface` for `ParamOps`/`QueryParam` | 60 | no |
| C10 | borrowed callback on several threads changes a real variable | 2, 1 | any-thread callback is a `thread_shared fn()` | 61 | no |
| C11 | `jobs.submit` takes a plain `fn()` | 2 | `owns f: thread_shared fn()` | 61, 94 | no |
| C12 | `@export` name replaces a C library symbol | 5, 6, 1 | unsafe form of the attribute; refuse known names | 60 | no |
| C13 | uncounted walk link over `thread_shared` nodes | 1 | region only for objects no other thread can end | R11's reason | no |
| C14 | arena instance changed while values made in it live | 1 | only `Allocator` methods change it then | 24, 47, 60 | no |
| C15 | `@cached` not cleared through a link held elsewhere | 1 (4) | refuse such a link into what a cached method reads | 60 | no |
| C16 | `Retained` object form freed (or changed) under C | 1 (5, 6) | remove the object form, or `destroys(f)` for it | 91 | no |
| C17 | `[CT-5]` puts a `static mut`'s heap value in read-only memory | 1, 6 | build it on first access | 8, 60 | no |
| C18 | reload migration ignores `using` instances and V5 links | 1 | rebuild by its instance, or refuse in plan | 24, 47, 60 | no |
| C19 | loop of shared owners leaks with a warning | 3 | (a) refuse unless one link is `weakref`; (b) keep, as Rust; (c) `unsafe` to silence | none | **yes** |
| C20 | `alloc_nodrop`, `MaybeUninit.write` skip clean-up | 3 | fence inside `unsafe:` for types that need clean-up | 58, 60 | no |
| C21 | channel kept alive by a queued `Sender` | 3 | `Receiver.drop` ends the queued values | 60 | no |
| C22 | `thread_shared` weak count not said atomic | 2, 1 | say both counts are atomic | 59 | no |
| C23 | atomic count compared then added can pass its maximum | 1 (4) | one `fetch_add`, stop at a limit with headroom | 59, 96 | no |
| C24 | static re-entry where the held lock may not be the callee's | 2 (4) | enter past only when proved the same; else E7018 | 43 | no |
| C25 | C source compiled in runs without `unsafe` | 5 | `unsafe import c "./x.c"` | 60 | no |
| C26 | another package's overlay on another header | 5, 1 | overlay records its header's hash | `[TIER-1]`, 60 | no |
| C27 | draw ranges unchecked against bound buffers | 5 | check per draw, robust access | 60 | no |
| C28 | exported `char` parameter takes any `uint32_t` | 1 | refuse `char` at the boundary (E5054) | 60 | no |
| C29 | no stack probes with `--emit c` (macOS unsure) | 1 | probe everywhere; write the flags with `--emit c` | `[RT-12]` | no |
| C30 | trampoline generation comes round again | 4 | retire a used-up slot | 59, `[HND-3]` | no |
| C31 | two thread checks only in `debug` | 2, 4 | every profile, or list them | `[PRF-1]` | no |
| C32 | `Volatile` texts omit alignment | 6 | add "aligned" | 60 | no |
| C33 | `Pool` keeps retired slots | 3 (bounded) | keep and document | JOB: limitation | no |
| C34 | C++ handle keyed to removed `@sync` | 2 | restate for `thread_shared` | 61 | no |

For the owner: **C19 only** (a loop of shared owners: refuse it unless one link is `weakref`, keep Rust's warning,
or require `unsafe` to silence it).

Scratch: `target/redesign8/06/` was made and holds nothing; no other file was written.
