# Review of 0.9.20_Hardened_5 (the local session's lead, solo, 2026-11-10)

The owner: "see whether it has any holes or not and whether it addressed the problems the better way even if
it goes against some of my ODRs ... you have to work solo". Target: `docs/spec-source/Ember_v0.9.20_Hardened_5.md`
(commit `8e58dafd`, branch `spec-0.9.20-hardening`, local, not pushed; SHA-256 `18A862E0…`). Rules quoted
from `tasks/spec-0.9.9/parts/` at that commit.

## 1. The owner's verdicts 161 to 169

| Verdict | In Hardened_5 |
|---|---|
| 161 `owns` binds as `-` | yes: `[GRM-47]` (`owns i * 7` refused, `E0119`) |
| 162 `runtime` before a class type | yes: `[LT-48]` (`open` or `abstract` classes) |
| 163 `runtime` on a variable | yes: `[LT-48]` (a local declared with its type) |
| 164 `sharedref mut` neither Send nor ThreadShared | yes: `[THR-8]`, `[THR-9]` |
| 165 `thread_shared` refused where made unless ThreadShared | yes now: `[THR-1]` p11 148-157 ("verdict 165" quoted) |
| 166 a scoped task touches no one-thread count | yes: p11 563 |
| 167 departures that are better stand | applied: ODR-264, ODR-261, N87 stand |
| 168 a compile writes the default `ember.toml` | yes: `[MAN-9]` p17 198-202 |
| 169 read the rest and fix | the cut itself |

## 2. The problems the earlier runs left open, tried against Hardened_5

Each was tried with the program that broke it before (method 4's last blocking objection, or the
question's own program).

| Problem | Hardened_5 | Verdict |
|---|---|---|
| K04 a `weakref` link outliving an arena | `[ALC-5a]`: the link carries the instance's region | **solved** |
| K05 `thread_shared … using A` | allowed, marked handed over at the making line (`[RC-4]` point 4, `[OBJ-1]`); only a manager "any thread may call" (`ThreadShared`, its `deallocate`/`reallocate` read `this`) lets the value cross threads, else it holds the `sharedref mut` link and is `E7015` (`[TYP-46a]`); the attacker's arena-outlives-thread hole closed in `[LT-4]` | **solved**, better than the trial's refusal (no correct program refused) |
| K07 a raw pointer inside a struct to a safe C call | the limit on raw pointers under the safe-to-call fact (`[FFI-2]` p16 43-58): every raw pointer, `ObjectPtr` and C function pointer the caller supplies, at any depth, in every spelling that asserts safety, **`@ffi(std_function, safe)` included** (method 4's last objection) | **solved** |
| K10 where a queued drop runs | only a value of a **plain** type (its `drop`s free memory and end owners; no `Lock`, `Block`, `Io`, `Alloc`, `Panic(Explicit)`, `RuntimeCheck(Aliasing)`, `Nondet`, no `static mut`, no code its package cannot read) made on the main or a `thread.spawn` thread is queued; every other value is handed over at its making and ends where its last owner ends (`[RC-4]` points 3-5). So a check point inside a lock, a walk or a `with scope` block cannot wait or change what is walked; method 4's last objection (a scope's task makes a value another thread lets go of) cannot arise: a task's values are never queued | **solved**; a strong, general design |
| K20 `AnyError` across threads | ODR-261: `Error` keeps verdict 110's line, `AnyError` holds `owns dyn (Error, Send, ThreadShared)` (`E2040` at the conversion); an `open` class another package may derive from is `Send` only when its header promises it, every derived class checked where declared (`[THR-8]`) | **solved**, method 4's cross-package hole included |
| NG-11 a callback returning a raw pointer to C | covered by the limit's "the result ... of a safe callable type" | **solved** (the FFI-21a rewrite was not needed) |

## 3. Holes looked for, and not found

* A queued `drop` that panics at a check point inside a lock: no unwinding (`[PAN-1]` aborts), so no
  half-changed state is ever seen.
* A `thread_shared` value in a manager whose `deallocate` changes it, ended on two threads at once:
  `[TYP-46a]` makes such a value hold a `sharedref mut` link, so it never crosses threads.
* A `thread_shared` value made `using` a local instance and handed to an unscoped thread: `E3063`
  (`[LT-4]`, the attacker's finding 2).
* A plain `drop` that calls a manager's `deallocate` at a check point under that manager's scope: a
  value that reaches another thread holds no instance a check point could change behind a scope
  (`[TYP-46a]`); a one-instance manager's `deallocate` reads `this`.
* A library's error holding an object of an `open` class, extended in another package with a `shares`
  owner: refused at the extending class's field, or the object never crosses (`[THR-8]`).

**Not covered by this review** (not read rule by rule): the grammar and type parts beyond the rules
named here, generics, compile-time code, hot reload beyond `[HR-20]`, SIMD/GPU, the toolchain, and
the remaining cluster fixes one by one (round 2 disposed of all 76; I checked the five rulings and
the method-4 counterexamples).

## 4. The departures from the owner's verdicts (verdict 167 let the better ones stand)

| Departure | My judgement |
|---|---|
| ODR-264, a list pattern may open a `match` arm (verdict 155) | agree: better; nothing ambiguous at arm level |
| ODR-261, `Error` keeps 110's line, `AnyError` carries the markers | agree: better; refuses only where the property is needed |
| N87/N139, the kept-by-C mark in the top bit of the type index (101, 114) | agree it stands: hot reload (`[HR-20]`) and the hand-over both need the mark on every object, and the type index is the only word every object has that no count operation touches; the cheaper weak-count word exists only on some objects. **Owed:** its cost, one `and` on every kind test, cast and dispatch-table load, is to be measured against C when built (the record says "not measured") |
| N83/N138 (127, 135) | withdrawn by verdict 168: agree |

## 5. Suggested improvements (language decisions: the owner's)

1. **N143, `shares this` refused.** Safe and right (a second owner made inside a method of an object
   in a one-owner place would outlive it, C++'s `shared_from_this` on an object no `shared_ptr`
   owns), but the help's way out is clumsy: `static fn enrol(owns me: shares mut Node):` called
   `Node.enrol(shares mut n)`. Ember already writes the object's word after a method's parameters
   (`fn finish() owns this -> u64:`); **`fn enrol() shares mut this:`** would reuse that spelling
   (checklist 11), be callable only on an object that has shared owners (`E2122` at a call on a
   one-owner object), and read `n.enrol()`, as Rust's `self: Rc<Self>`. Recommendation: adopt.
2. **Finding 13 of the other agent's check (the owner's verdicts 147-149, 158-161):** `Emitter(label)`
   hands `label` over with no word, while a function call needs `owns`. Whether a name survives a line
   then depends on whether the callee is a constructor, which the line does not show (principles 2 and
   4). Recommendation: a constructor argument that hands over takes `owns` as any call's does; it is
   the owner's to decide, since it changes his verdicts.
3. **Finding 16 (verdicts 104, 105):** `total += x` in a scoped task stops on a private total's overflow
   where the plain sum would not; verdict 104 was (b) "for now". Worth putting back to the owner,
   since `[PHIL-16]` says a program does what its words say.

## 6. Verdict

On everything this review tried, Hardened_5 holds: every owner verdict 161-169 is in, the five hard
questions the agent runs could not close are closed with general rules, and no hole was found. What
is left is the owner's: the lead's own rulings made under the delegation (decisions.md N89-N146,
OWNER-QUEUE ODR-260 to 267, K21's C++ overload design `[FFI-57]`/`[FFI-40b]`), the three suggestions
above, and the parts this review did not read rule by rule.

**After the rest (§7, §8):** every changed rule of every part read; three holes found, all outside
the core language: hot reload's live-instance list across threads (`[HR-12]`), code that runs
before `main` through a start-up section (`[FFI-51]`), and a dependency's C header reading the
builder's files (`[CT-2]`, `[MAN-10]`). Each has a general fix in §8.

## 7. The rest, part by part (the owner: "yes, check the rest too")

Method: every rule 0.9.20 changed or added (300 changed, 20 new, against `parts-0.9.15-h11`), read as
its word-level change, each tried for a program it lets through that breaks a promise, a correct
program it refuses, and a clash with another rule. The unchanged text was read by the sweep's
finders, the cloud session's round-1 and round-2 readers, and the H4 check before this.

* **Part II, lexical (8).** Sound. `[LEX-1a]` refuses bidirectional format characters anywhere, a
  comment and every literal included (the "Trojan Source" attack closed); the rest are precisions.
* **Part III, grammar (15).** Sound. `(e,)` and a one-element target list are `E0100` (Python's
  trap gone; a pack of one is its type, `[TYP-47]`); a lambda's body is one simple statement, the
  message naming a local function; `extend` adds no field; `[GRM-15]`/`[GRM-43]`/`[GRM-47]` carry
  verdicts 160 and 161 and K25 consistently.
* **Part IV, types (22).** Sound. `[TYP-5]` keeps the word on an upcast and refuses `ref mut Dog` to
  `ref mut Animal` (the covariance hole of Java's arrays: the wider name could write a `Cat` into a
  `Dog`'s place); owner words widen, since `=` on them swaps the owner and never writes the block.
  `=` on a `ref mut` name of a `dyn I` is `E3023` (the place's size is known only while running).
  `7 as Color` is refused for `Color.from_repr(7)`. A struct holding a raw pointer is never
  `ThreadShared`, the route being a `@ffi(handle)` type (`[TYP-43]`): a cost for C wrappers, not a
  hole.
* **Part V, declarations (26).** Sound. `[MOD-8]`: a name bound by a glob import and by the prelude
  is `E1031`, so a dependency's update cannot take a prelude name in silence; `[CLS-2]`: an `init`
  of a `pub open` class calls a `virtual` method only after every field is set (C++'s
  half-built-object bug); `[STR-1]`: a constructor is as visible as its narrowest field;
  `[FN-5]`: a default on a `ref mut` parameter is `E3027`; `[FN-6]`: `fn` converts to `once fn` and
  to `unsafe fn`, never back; `[CLS-4]`: an override's first line is the base's (`E2113`).
* **Part VI, expressions (18).** Sound. `[EXP-4]`: a `match` subject's lock guard is let go before
  the arm unless the pattern views it (K23's deadlock); `[CTL-7]`: no `yield` in a `defer`;
  `[CLO-6a]`: a `once fn` is called only out of a place no other name reaches. A readability cost,
  not a hole: `["ann", "bob"]` with no context is an `Array[StrView]` (`[TYP-38]`), so pushing a
  `String` into it is refused with a message naming `Array[String]`.
* **Part VII, ownership (24, `[LT-48]` read before).** Sound. `[OWN-3]`: a hand-over is a reshape
  (views into the giver end); `[BRW-16]`: positions only for `Array` and `SoA`, any other
  container's element an address ended by every reshape (C++'s `deque` invalidation closed);
  `[BRW-17]`: two names differ only when neither type can hold the other, derivation counted;
  `[BRW-21]`: a foreign call without `no_callbacks` is an open line resolved at each caller against
  its own callbacks (separate compilation); `[LT-3]`: a `static mut` is a one-owner place tracked
  in summaries; `[LT-46]`: a `sharedref mut` change while a read-only view made through another
  name is used is `E3021`.
* **Parts VIII-IX, classes and memory.** Sound. `[RC-1]`: a one-owner object that `weakref` links
  may reach has its count written 0 at its end, so a link sees it gone; `[ARN-3]`: `try_` forms for
  every arena allocation, and a full arena stops the program, never returns a bad block; `[ARN-4]`:
  an arena grows by chunks, so nothing in it ever moves; `[ARN-5]`: an arena's containers are kept
  `MutSpan` views; `[ALC-6]`: a making line carries its instance's effects; `[UNS-1]`: giving C a
  raw pointer is `unsafe`; `[OPT-1]`: a `thread_shared` value is never put in a frame.
* **Parts X-XIV, effects, threads, comptime, ECS.** Sound. `[ECS-4a]`: two components of one type
  refused; `[CT-1a]`: comptime reaches no runtime static; `[CT-3]`: the step count is
  deterministic; `[EFF-1]`: a static's first-access effects counted, `Block` included; `[DET-3]`:
  `dyn` under `@deterministic` needs a declaration; `[THR-18]`: a `with lock` block holds a view of
  the guarded value; `[THR-15]`: a channel's `T` must be `Send` where the channel is made;
  `[THR-5]`: a task never runs on its scope's own thread. **Cost to measure:** `[EFF-2a]` compiles a
  function-typed parameter's body once per function given (binary size).
* **Parts XV-XVI and I, standard library and C.** Sound. `[STD-11]`: `RandomState` for keys an
  outside party chooses; `[STD-6]`: layers, `E1042`; `[FFI-53]`: a token's bits are non-canonical
  on x86-64 and AArch64; `[FFI-23]`: a token is bound to its thread; `[FFI-54]`: `into_raw` only on
  runtime or C blocks; `[FFI-21]`: a function-pointer parameter needs a callback word; `[FFI-11]`: a
  C result valid only until the library's next call (`localtime`, `strerror`, `getenv`) gets no
  lifetime word, its function stays `unsafe fn`, and an `extend` wrapper copies it.
* **Parts XVII-XVIII, toolchain and implementation (24).** Sound, except holes 2 and 3 below.
  `[MAN-10]` (new): a dependency's manifest cannot run code on the builder's machine (its `[c]`
  flags limited to defines, includes under its root, a C standard, an optimisation level and
  warnings; the rest needs `native = [...]` in the root's `[build]`). `[BLD-16]` (new): no fact of
  the whole program decides a package's code or layout, so a package compiles the same in every
  program, in its tests, in a reload image and in a module loaded at run time. `[MNG-1]`: 128-bit
  instance hashes, the spellings compared before the link (`E9040`). `[BCK-8]`: a line whose
  verdict rests on what the whole program puts in a place is checked again by every package that
  puts something there. `[RT-7]`: the add of a home count into the away count is checked in 64
  bits. `[RT-8]`: closing a thread's queue is a release, the subtract that finds it closed an
  acquire. `[RT-12]`: a C-made thread's guard page is the host's, listed in `ember tcb`.
  Form notes only: `[BLD-16]` carries both "new" and "changed" tags and its second paragraph cites
  itself.
* **Annex B, hot reload (21).** Sound, except hole 1 below. `[HR-42]`: entering Ember code and a
  commit starting are ordered so at least one sees the other (the increment, then a full fence,
  then the read of the commit mark); `[HR-44a]` (new): a stored function value the new image does
  not count for its place refuses the reload; `[HR-15c]` (new): a view into a field or element is
  moved to the part's new place, or the reload is refused; `[HR-48]` (new): a lambda's captures
  have a schema and migrate as any value does; `[HR-7]`: a removed function's thunk still named
  anywhere refuses the reload; `[HR-37]`: a discarded reload ends what prepare built. Small cost
  precision: `[HR-12]` and the cost table say "two stores" to register an object; a doubly linked
  list takes four to add an object (two to remove it).
* **Annex C, C++ (19).** Sound. `[FFI-57]` (new, K21): a C++ name declared several times is one
  Ember name of several members, each with its own thunk, safety and overlay line; a call picks the
  member C++ would pick, a tie is `E5035` with the fix in the message, and the safety checks run on
  the member chosen, so a wrong pick can never skip a check. `[FFI-40b]` (new): a `const` and a
  non-`const` pair become `at` and `at_mut`. `[FFI-40]`: `T&&` is `owns`, and the moved-from object
  is ended by `T`'s destructor when the call returns. `[FFI-17d]`: a trampoline class is bound to
  its making thread by default, checked on every override call, in every profile. One precision:
  `[FFI-57]`'s "the nearer type" is not defined between an integer and a float type of one size
  (`i64` against `f64` for an `i32` argument); C++ calls that a tie, so the rule should too.
* **Annex D, GPU (4).** Sound. `[GPU-2]`: a `Frame` whose owner ends without `end_frame` is ended
  by its `drop`, submitting nothing, so no later frame waits for it; `[GPU-8]`: `prev` and `cur`
  are a `ReadHandle` and a `WriteHandle`, so using the wrong one is a compile error at no run-time
  cost.

## 8. Holes found in the rest

1. **Hot reload: an object ended on another thread than the one that made it (`[HR-12]`).** In a
   reloadable build every class instance is in a live-instance list (two pointers in its header),
   and the rule says registering an object that is not `thread_shared` "touches nothing another
   thread touches". That is false for any one-owner object that changes thread: a scoped task that
   returns a `Player` (`owns Player` is `Send`), or a value sent down a channel. The thread that
   ends it unlinks it from the list of the thread that made it, while that thread may be adding or
   removing its own objects in the same list: a data race inside the runtime, a broken list, and
   the next reload walks it. The same holds for a registered container (`[HR-13]`) and for the
   list of a thread that ends while its objects live on. **Fix (general, decided from the type, as
   `[BLD-16]` asks):** register an object of a `Send` type as a `thread_shared` one is already
   registered, ordered by the runtime (one lock or one atomic exchange per make and end), and keep
   the plain stores for types that are not `Send`; a thread's list is adopted by the runtime when
   the thread ends. Cost: reloadable (development) builds only. A cheaper variant: plain stores on
   the making thread, and an end on another thread pushes the object onto the making thread's
   queue (one atomic), which unlinks it at its next check point.
2. **Code that runs before `main` without `unsafe import` (`[FFI-51]`).** The rule makes an import
   `unsafe` when the header declares a function with a constructor attribute or a global whose
   initialiser is not constant. It misses the other common way to run code before `main`: a global
   with a **constant** initialiser placed in the start-up section, which the loader calls:
   ```c
   static void setup(void) { /* anything */ }
   __attribute__((used, section(".init_array"))) static void (*run_setup)(void) = setup;
   /* MSVC: __declspec(allocate(".CRT$XCU")) static void (*run_setup)(void) = setup; */
   ```
   `import c "lib.h"` with no `unsafe` compiles that into the shim, and `setup` runs before `main`
   with nothing vouching for it. On MSVC this is the usual way, since MSVC has no constructor
   attribute. **Fix:** count as running any declaration with a `section` attribute,
   `__declspec(allocate(...))`, `#pragma section` or `init_seg`, and any file-scope `asm`
   (`[FFI-38]`: what the importer cannot decide counts as running).
3. **A dependency's C header can read the builder's files (`[CT-2]`, `[MAN-10]`).** `[CT-2]`
   promises "a dependency's build cannot read the builder's files", and `[MAN-10]` limits a
   dependency's `-I` paths to its own root for that reason. But the C preprocessor reads any path
   written in the header itself: `#embed "/home/ci/.aws/credentials"` (C23; Clang 19+, GCC 15) into
   a constant array, or a file-scope `asm(".incbin \"...\"")` on any GCC or Clang, puts a file of
   the build machine into the program. A constant array needs no `unsafe import`, so this passes
   with no word anywhere. **Fix:** a dependency's C compile reads only files under its own root,
   under the root package's include paths and in the compiler's own system directories (checked
   from the compiler's own list of the files it read, `-MD` or `/showIncludes`, so `#include` and
   `#embed` are both covered), and a dependency's file-scope `asm` needs `native = [...]`, as its
   other build-machine powers do. Small precision of the same rule: `[MAN-10]` should say each flag
   reaches the C compiler as one argument, never through a shell, so a `-D` value holding a quote
   cannot add a second flag (on Windows the command line is one string the build must quote).

None of the three touches a program that uses neither hot reload nor C; 2 and 3 break
`[FFI-51]`'s and `[CT-2]`'s own promises. Not covered: the unchanged text of 0.9.20 (read by the
earlier runs) and the examples and appendices beyond the change log.
