# 02 Today's spec: C calls, `unsafe`, the runtime, build and command-line flags

Agent 02 (spec-ffi) of the unsafe-features inventory. What I read, all of it: `inputs/brief.md`,
`tasks/96/redesign/JOB.md` (verdicts 1 to 98), `inputs/duties.md` section 02; in `SPEC`
(`docs/spec-source/Ember_v0.9.10_Hardened_35.md`): Versioning (102-127), the tiers and the Safe Ember
invariant (143-155, 216-227), IV.5 raw types (1147-1158), `[FN-6]` (1589-1600), `[STA-1]` (1803-1807),
IX.4 `[UNS-*]` (3005-3051), Part XVI `[FFI-*]` and `[BLD-FFI-*]` (4581-5012), Part XVII `[CLI-*]`,
`[MAN-*]`, `[VER-5]`, `[BLD-*]`, `[PRF-*]`, `[CONF-*]`, `[ABI-*]` (5017-5288, 5589-5617), Part XVIII
`[CG-C-*]`, `[RT-*]`, `[CG-LL-*]` (5902-6123), Annex C (6451-6655); and a search of `SPEC` for "off",
"disable", "unchecked", "trust", "assume", "release", "profile". In `V5`: `vocabulary.md` sections 3.1 to
3.5 (the rows of every word in my families), `rules.md` R12, R15, R16 and section 2 (662-1029),
`area-E.md` where it treats C callbacks, exports and `static mut` (1405-1430, 1820-1920, 2600-2700).
Where a claim rests on the compiler, runtime, library or tests, the file and line are given. Nothing was
compiled or run; "when run" says what the rules and the code I read make it do.

`V5` spellings are used in programs wherever the feature exists in `V5` (`this`, `owns`, `exists as`,
`copy of`, `CStrView` for `cstr`, no `Box`). A feature `V5` does not mention keeps today's spelling.

Kinds (the brief's six): 1 memory, 2 race, 3 leak, 4 skipped check, 5 memory the rules cannot see,
6 undefined C.

## Summary

| # | feature | kind | fenced today | fenced in V5 | for the owner |
|---|---|---|---|---|---|
| F1 | a foreign call that re-enters Ember code the rules do not count (an `@export`ed function, a C++ override, a borrowed callback still running) | 1 | yes (path 1: `static mut` needs `unsafe`); path 2 checked while running | **no** | no (verdicts 31, 39) |
| F2 | `@export("name")` with a name the C library or the runtime already defines | 1, 3, 5 | no | no | no (verdict 60) |
| F3 | a C function pointer imported as `extern "C" fn`, callable with no `unsafe` | 5, 1 | no | no | no (verdict 60) |
| F4 | a borrowed callback that C may run on several threads at once changes a real variable | 2 | no | no | no (verdict 61) |
| F5 | dropping a `Retained` token of an object while C still holds its address | 1 | partly | unclear | no (verdict 91) |
| F6 | an exported function's `char` parameter (C passes any `uint32_t`) | 1 | no | no | no (verdict 60) |
| F7 | stack probes only on Linux, and none in `--emit c` output | 1 | no | no | no |
| F8 | another package's overlay makes calls safe in a package with no `unsafe` | 5, 1 | no | no | no (verdict 60) |
| F9 | C source compiled into the program (`import c "./x.c"`, `implementation=`) runs without a call | 5 | no | no | no (verdict 60) |
| F10 | thread checks that run only in the `debug` profile (`[FFI-33a]`, `[FFI-33b]`) | 2, 4 | backs a fenced fact | partly | no |
| F11 | the C++ header's copyable handle is keyed to `@sync`, which V5 removes | 2 | n/a | stale rule | no |
| B1-B19 | the features that are fenced: raw pointers, casts, `transmute`, `asm`, `unsafe fn`, uncontracted and variadic foreign calls, asserted foreign facts, unions, foreign statics, `adopt`, `into_raw()`, `get_unchecked`, `assume_init`, `assume_disjoint`, `@assume_noalloc`, `UnsafeCell.get`, `Send` by hand, `Volatile`, and `static mut` (fenced today, made safe by a rule in V5) | | yes | yes | no |

Section C lists what the search found that turns no check off.

---

## Part A. Features that can break a safety promise outside `unsafe`

### F1. A foreign call that re-enters Ember code the rules do not count

- **Feature.** V5's rule for foreign calls, R12 (`V5/rules.md` 745-756): "a call of a foreign
  function counts as possibly running every callback C keeps (those the program hands over through a
  `callback(retained)` or `callback(once)` parameter ...)". Three other ways C runs Ember code during a
  foreign call are not in that list:
  1. an `@export`ed function (`[FFI-26]` SPEC 4945; `V5/vocabulary.md` 194 "same"), which C can call
     at any time by its symbol;
  2. an override of a class derived from a C++ class (`[FFI-39]` 6564, `[FFI-39c]` 6574, C++ keeping the
     object), which C++ calls through its virtual table;
  3. a `callback(borrowed)` function whose own call is still running, which C may call again from a
     foreign call made inside it.
  `rules.md` never mentions `@export` (searched); `area-E.md` 1416-1420 treats exports only for which
  thread may touch a `static mut` (E7012).
- **How it breaks safety.** Kind 1: a list changed while a loop walks it, so a view into its old block
  is read after the block is freed. Program (path 1, V5 spelling):

  `host.h`
  ```c
  void host_poll(void);
  ```
  `host.c`
  ```c
  #include "host.h"
  void log_line(void);                                   /* the Ember export below */
  void host_poll(void) { log_line(); log_line(); log_line(); }
  ```
  `host.overlay.em`
  ```ember
  unsafe overlay c "host.h":
      fn host_poll()
  ```
  `main.em`
  ```ember
  import c "./host.c" with (overlay = "host.overlay.em") as host

  static mut LOG: Array[String] = []

  @export("log_line", threads=main)
  fn log_line():
      LOG.push(String("event"))

  fn main():
      LOG.push(String("start"))
      for line in LOG:
          host.host_poll()
          println(line)
  ```
  The overlay's assertion is true: `host_poll` itself breaks nothing. When run, the loop's `line` is a
  view of `LOG[0]` in `LOG`'s block; `host_poll` runs `log_line` three times, the pushes outgrow the
  block (one element, then two, then four), the list moves to a new block and the old one is freed;
  `println(line)` reads the freed block (garbage or a crash). `threads=main` is allowed to change
  `LOG` (`area-E.md` 1416-1420 refuses only `threads=any|creator`).

  Path 2 (C++): a `class Button(cpp.Widget)` whose `override fn on_event(code: i32)` does
  `this.codes.push(code)`, handed to C++ to keep (`[FFI-39c]`'s `Retained` token), and
  `for c in b.codes: cpp.dispatch_all()` where `dispatch_all` calls `on_event` on every kept widget:
  the same use after free. Today this is stopped while running: the access words make the override's
  push panic (`[FFI-39d]` 6584-6589; `tests/conformance/FFI-39d/run_fail_a_conflicting_reentrant_access_panics.em`).
  V5 removes the access words ("Gone ... the access words checked on class fields", `rules.md`
  1023) and puts nothing in their place. Where the object is the call's own argument
  (`cpp.fire(ref mut this, 4)` inside `for c in this.codes:`), R8 (f) already refuses it; the gap is an
  object C++ keeps and a call that does not name it.

  Path 3: inside a `callback(borrowed)` function that pushes to `seen`, a loop `for v in seen:` that
  calls a foreign function which C uses to run the same callback again.
- **Fenced?** Today path 1 is fenced: every access to a `static mut` needs `unsafe` (`[UNS-1]` 3023,
  `[STA-1]` 1805-1806). V5 makes `static mut` safe (`vocabulary.md` 134, R15 899-901) and counts only
  kept callbacks at a foreign call (R12 745-756): no line fences or refuses any of the three paths.
- **Rust.** A `static mut` is reached only in `unsafe`; the safe form, a `thread_local!` `RefCell`,
  panics at the re-entrant `borrow_mut` while running. For C++ subclasses, as far as I know, autocxx's
  subclass support keeps the Rust object behind `Rc<RefCell<…>>`, again a run-time panic. Rust has no
  compile-time rule for this.
- **Recommendation.** Make it safe with a rule, at no run-time cost: extend R12's foreign-call bullet
  so a foreign call not marked `@ffi(no_callbacks)` counts as possibly running (1) every `@export`
  function whose thread contract lets it run on this thread, (2) every override of every class derived
  from a C++ class that C++ may be holding, (3) every `callback(borrowed)` function whose call is still
  running on this thread. The program above is then refused, E3020: "`host_poll()` may run `log_line`
  (exported, line 5), which adds to `LOG` (line 7) while this loop walks `LOG` (line 11); call
  `host_poll()` before or after the loop, or, if `host_poll` never calls back into Ember, mark it
  `@ffi(no_callbacks)` in `host.overlay.em`". Same speed as C (the checks happen when compiling).
- **Settled?** Yes, by verdicts 31 and 39: "A C call counts as possibly running the callbacks C keeps,
  unless its declaration says it never calls back"; an export, an override C++ keeps and a running
  borrowed callback are Ember code C can reach, so the rule only needs to name them. Not for the owner.
- **How sure.** Path 1: sure (the rule text lists only `callback(retained)`/`callback(once)`, and the
  program uses only allowed lines). Path 2: fairly sure (V5 says nothing about classes derived from C++
  classes). Path 3: likely, not checked against a test.

### F2. `@export("name")` with a name something else already defines

- **Feature.** `@export("symbol")` "gives a function a stable C symbol" (`[FFI-26]` 4945); "sets the
  symbol exactly" (`[MNG-2]` 5972). It is an ordinary attribute (attribute table 1853, `fn, static`),
  needing no `unsafe`. The compiler refuses only `main` and a symbol two Ember functions define
  (`tests/conformance/FFI-26/reject_export_named_as_program_entry.em`,
  `reject_duplicate_export_symbols.em`). V5 keeps it unchanged (`vocabulary.md` 194).
- **How it breaks safety.** Kinds 1, 3, 5: the program's function replaces a C library or runtime
  function for the whole process. Program (V5 spelling):
  ```ember
  @export("realloc")
  pub extern "C" fn same_block(p: *mut void, n: usize) -> *mut void:
      return p

  fn main():
      xs: Array[int] = [1]
      for i in 0..1000:
          xs.push(i)
      println(xs.len())
  ```
  Returning a raw pointer it was given is safe Ember. When run (on Linux at least, where a symbol the
  executable defines takes the place of the C library's), the runtime grows lists with `realloc`
  (`[RT-1]` 6062-6065), gets the old one-element block back each time, and the pushes write past it:
  heap corruption, a crash or wrong numbers. `@export("free")` with an empty body leaks every
  allocation (kind 3); `@export("memset")` returning at once leaves every new object's fields
  unwritten (the runtime zeroes objects with `memset`, `runtime/ember_rt/src/ember_rt.c` 3148).
  Exporting a `static` (admitted by the attribute table) would also let C write a module variable
  behind V5's thread rule; no test exports a static, so this part is unverified.
- **Fenced?** No, today or in V5: no line puts `@export` inside `unsafe`.
- **Rust.** Edition 2024 makes `#[no_mangle]` and `#[export_name = "…"]` unsafe attributes, written
  `#[unsafe(no_mangle)]`, for exactly this reason: a symbol name can replace another one.
- **Recommendation.** Fence it, as Rust does: an `@export` names its symbol only in an unsafe form of
  the attribute with a `# SAFETY:` note (spelling for the vocabulary step, for example
  `@unsafe(export("game_on_update"))`). Outside it, E3100: "`@export("realloc")` gives `same_block` the
  C name `realloc`, which the C library already defines, so every `realloc` in the program, the
  runtime's included, would run `same_block`; if this is what you mean, say that you vouch: write the
  attribute as `@unsafe(export("realloc"))` with a `# SAFETY:` note above it". Also refuse outright the
  names the compiler knows collide (the runtime's `ember_` prefix, `[RT-5]` 6072; the C standard
  library's names), as `main` is refused today. No run-time cost.
- **Settled?** Verdict 60. The spelling is the vocabulary step's. Not for the owner.
- **How sure.** Sure that nothing fences it. Fairly sure the example links and interposes on Linux;
  not built. On Windows with MSVC the CRT's `dllimport` declarations may make it a compile error
  instead.

### F3. A C function pointer imported as a safe callable value

- **Feature.** The importer maps every C function pointer, a struct field's, a typedef's or a result's,
  to `Option[extern "C" fn(…)]` (`[FFI-8]` 4733; `compiler/ember_driver/src/cimport.rs` 1926-1930,
  1985, 2271-2275). `extern "C" fn(…)` is a safe callable type (IV.5, 1157; `[FN-6]` 1589-1597), and
  calling a value of it needs no `unsafe`
  (`tests/conformance/FN-6/accept_capture_free_function_as_extern_pointer.em`; the standard library
  calls one in safe code, `std/src/ffi.em` 167, 184-185). V5 keeps `extern "C" fn` unchanged
  (`vocabulary.md` 107).
- **How it breaks safety.** Kinds 5 and 1: a call into C code with no contract. Program:

  `hooks.h`
  ```c
  #include <stdint.h>
  typedef struct { void (*release)(uintptr_t address); } Hooks;
  Hooks get_hooks(void);              /* release(a) frees the block at address a */
  ```
  `hooks.overlay.em`
  ```ember
  unsafe overlay c "hooks.h":
      fn get_hooks()
  ```
  `main.em`
  ```ember
  import c "hooks.h" with (overlay = "hooks.overlay.em") as h

  fn main():
      hooks = h.get_hooks()
      if hooks.release exists as release:
          release(4096)
  ```
  The overlay's assertion is true: calling `get_hooks` breaks nothing. When run, `release(4096)` frees
  an address that was never allocated: heap corruption or a crash. The same happens with a pointer read
  from an `@ffi(immutable)` extern static, or one C passes to a callback.
- **Fenced?** No. `[UNS-1]` (3022) fences a call of "an `extern` function without a verified
  contract", and `[FFI-2]` (4612-4613) says why the safe-to-call fact is never derived ("a scalar can be
  an address (`void free_addr(uintptr_t)`)"), but a call through an imported pointer is neither: the
  pointer's target has no contract and the call needs nothing.
- **Rust.** bindgen imports a C function pointer as `Option<unsafe extern "C" fn(…)>`; calling it
  needs `unsafe`.
- **Recommendation.** Fence it: import a C function pointer as `unsafe extern "C" fn(…)` (the type
  `[FN-6]` already has), so a call through it needs `unsafe:` (E3100: "`release` is a C function
  pointer from `hooks.h`, with no contract; put line 6 in an `unsafe:` block with a `# SAFETY:` note
  saying why the call is safe"); an overlay may assert one safe, as `safe fn` does for a function
  (spelling for the vocabulary step, for example an `@ffi(safe)` on the typedef or field). A function
  pointer made from an Ember function stays a safe `extern "C" fn`. `std.ffi`'s `ForeignBox` drop then
  calls its destructor inside `unsafe:` (the destructor came from an asserted `destroys(f)`). No
  run-time cost.
- **Settled?** Verdict 60. Not for the owner (only programs that use C function pointers change).
- **How sure.** Sure of the mapping and the safe call; the program is built from both.

### F4. A borrowed callback C may run on several threads at once

- **Feature.** A `callback(borrowed)` parameter (`[FFI-21]` 4894-4898). With no thread word its
  contract is "any thread" (`[FFI-22]` 4918-4922: captures must be `Send` "unless the contract says
  `threads(main)` or `threads(creator)`"). V5: "A plain capture (the real variable) is allowed only for
  `callback(borrowed)`, which C calls only during the call" (`area-E.md` 1900-1902); the
  `thread_shared fn()` rule (R12 693-707) and the capture words for C callbacks (R15 877-882) are written
  for kept callbacks only.
- **How it breaks safety.** Kind 2, then 1. Program (V5 spelling):

  `pvisit.h`
  ```c
  #include <stddef.h>
  /* Calls visit(data, values[i]) for every i, spread over four threads that run at once. */
  void visit_each(const int* values, size_t count, void (*visit)(void* data, int value), void* data);
  ```
  `pvisit.overlay.em`
  ```ember
  unsafe overlay c "pvisit.h":
      fn visit_each(values: borrowed count(count), count, visit: callback(borrowed, user_data=data), data)
  ```
  `main.em`
  ```ember
  import c "pvisit.h" with (overlay = "pvisit.overlay.em") as pv
  from std.ffi import c_int

  fn main():
      values: Array[c_int] = [1, 2, 3, 4, 5, 6, 7, 8]
      seen: Array[c_int] = []
      pv.visit_each(values.as_span(), fn(v: c_int): seen.push(v))
      println(seen.len())
  ```
  Every fact the overlay states is true (C does call from any thread; nothing false is asserted).
  When run, four threads push to `seen` at once: pushes are lost, and when two grow the list together
  one frees the block the other is writing: a race, then heap corruption; it prints a number below 8
  or crashes.
- **Fenced?** No, today or in V5. Today's "captures must be `Send`" does not stop it: a real variable
  lent to the callback may be sent, but four threads using it at once need more than that.
- **Rust.** A safe wrapper of such a C function takes `F: Fn(i32) + Sync`; `seen.push` inside an `Fn`
  closure is refused, and the program needs a `Mutex` or an atomic.
- **Recommendation.** Make it safe with a rule: a callback whose contract lets C call it from any
  thread (`threads(any)`, the default) is a `thread_shared fn()` (verdict 61): it holds only
  `thread_shared` values or copies and changes nothing but through `Atomic`s or locks; a borrowed one
  may change real variables only when its contract says `threads(creator)` or `threads(main)`, which
  the trampoline checks in every profile (`rules.md` 995). Here E7015: "C may call this function from
  any thread at once (`visit_each`'s `visit` has no `threads(...)` word), and it changes `seen`, a list
  of this thread; count with an atomic, or, if C calls it only on this thread, say so in the overlay:
  `visit: callback(borrowed, user_data=data) threads(creator)`". No run-time cost.
- **Settled?** Verdict 61 ("A function type says whether any thread may run its functions"),
  applied to borrowed callbacks. Not for the owner.
- **How sure.** Fairly sure: no rule I found looks at the thread contract of a borrowed callback's
  captures.

### F5. Dropping a `Retained` token of an object while C still holds its address

- **Feature.** `Retained.pin(v)` "hands an Ember-owned object to C for keeping ... Dropping the token
  releases the object. It is the only way to let foreign code keep Ember memory" (`[FFI-23]` 4923-4929;
  `std/src/ffi.em` 199-221). A `retained` parameter gives C the address (`[FFI-11]` 4826). For a
  callback the token's drop first tells C to forget it, through the contract's `destroys(f)`
  (`[FFI-21]` 4900-4901, E5041 if missing); for an object, nothing tells C. V5's row for `Retained`
  (`vocabulary.md` 268) says it "keeps an object on Ember's side while C holds a callback; it gives no
  object away", but keeps `Retained.pin(v)` and does not say whether today's object form stays.
- **How it breaks safety.** Kind 1. Program (today's spelling; it is today's test
  `tests/conformance/FFI-23/accept_retained_lets_foreign_code_keep_ember_memory.em` with two lines
  swapped):
  ```ember
  from std.ffi import Retained
  from std.mem import drop

  struct Point:
      x: i64
      y: i64

  unsafe extern "C":
      @ffi(param(point, retained, one), link_name="keep")
      safe fn keep_point(point: Retained[Box[Point]])
      @ffi(effects=[])
      safe fn kept_point_sum() -> i64

  fn main():
      token = Retained.pin(Box(Point(x = 3, y = 4)))
      keep_point(token)
      drop(token)
      println(kept_point_sum())
  ```
  When run, `drop(token)` frees the point; C still holds its address and `kept_point_sum()` reads the
  freed block (a wrong number or a crash).
- **Fenced?** Partly. Both C functions are asserted in `unsafe extern`, but the vocabulary has no word
  for "C uses this address until the token ends", so even a careful, true contract lets safe code free
  the object under C.
- **Rust.** Handing C an object is `Box::into_raw` (safe; a leak is safe in Rust); freeing it again is
  `Box::from_raw`, which is `unsafe`. Ember's token drop is the `from_raw` step without `unsafe`.
- **Recommendation.** Remove the object form, as verdict 91 says (`Retained` "keeps an object on
  Ember's side while C holds a callback, it does not give an object away"): C keeps a foreign object by
  an `owns` hand-over to a parameter declared `owned`, or any object by `into_raw()` inside `unsafe:`;
  it gives one back through `adopt` inside `unsafe:`. If the object form must stay, it needs the
  callback's rule: a `retained` object parameter's contract names `destroys(f)`, which the token's drop
  calls before releasing (E5041 without it). No run-time cost beyond that call.
- **Settled?** Verdict 91. V5's text must say so explicitly. Not for the owner.
- **How sure.** Sure for today (the test shows `keep_point` then a later `drop(token)`); unsure what V5
  keeps.

### F6. An exported function's `char` parameter

- **Feature.** An exported function's parameters may be any FFI-safe type (`[FFI-31b]` 4957-4960);
  `char` is one (`compiler/ember_types/src/lib.rs` 2756-2760), crossing as `uint32_t`
  (`compiler/ember_codegen_c/src/lib.rs` 12146). The boundary refuses only range types (E5054,
  `compiler/ember_typeck/src/lib.rs` 21345-21362; `tests/conformance/FFI-26/reject_a_range_type_at_the_boundary.em`),
  for the reason "Admitting one here would let a foreign caller manufacture a range value that passed
  no check".
- **How it breaks safety.** Kind 1, a value of the wrong type. Program (V5 spelling):
  ```ember
  @export("letter_len")
  pub extern "C" fn letter_len(c: char) -> int:
      return c.to_string().len()
  ```
  C side: `int64_t letter_len(uint32_t c);` called as `letter_len(0xD800)`, valid C. When run, Ember
  holds a `char` that `[UNS-4]` (3028-3033) says cannot exist (a surrogate), and `c.to_string()` puts
  bytes that are not UTF-8 into a `String`, which `[UNS-4]` also says cannot be; what code that
  relies on UTF-8 then does is undefined by the spec. The same holds for an `@layout(c)` struct with a
  `char` field passed by value. (A `CStrView` parameter, which C may pass as null, is the weaker
  cousin: Rust also trusts C not to pass null for a reference parameter, so I do not count it.)
- **Fenced?** No, today or in V5.
- **Rust.** `char` in an `extern "C"` signature draws the `improper_ctypes_definitions` warning ("the
  `char` type has no C equivalent"; it suggests `u32`).
- **Recommendation.** Make it safe with a rule: refuse `char` at a foreign boundary as range types
  are, E5054: "`char` has no C equivalent, and C may pass any `uint32_t`; take `c: u32` and make the
  character with `char.from_u32(c)`, which gives `None` for a value that is not one" (`[FFI-8]` 4722
  already maps `char32_t` to `u32` this way). No run-time cost beyond the program's own check.
- **Settled?** Verdict 60, and the spec's own reason for E5054. Not for the owner.
- **How sure.** Sure it is accepted (the code above); I did not check what `to_string` does with the
  bad value.

### F7. Stack probes depend on build flags Ember passes only on Linux

- **Feature.** `[RT-12]` (6089-6093): "The backend compiles with stack probes
  (`-fstack-clash-protection` where the C compiler has it; MSVC probes by default) ... A stack write
  can never land beyond the guard page." The C backend passes the flag only on Linux
  (`compiler/ember_build/src/lib.rs` 1983-1985). `ember build --emit c` writes the C with no flags at
  all, for build systems that compile it themselves (`[CLI-2]` 5054;
  `compiler/ember_driver/src/lib.rs` 5857-5872).
- **How it breaks safety.** Kind 1. Program:
  ```ember
  fn fill(depth: int) -> int:
      pad: [int; 4096] = [0, 0, 0, ...]          # 4,096 zeros written out: a 32 KB frame
      pad[depth % 4096] = depth
      if depth == 0:
          return pad[0]
      return fill(depth - 1) + pad[(depth * 7) % 4096]

  fn main():
      println(fill(1000000))
  ```
  On Linux it stops with "stack overflow in fill". Where nothing probes, a 32 KB frame is larger than
  the guard page (16 KB pages on macOS arm64), so the frame can start below the guard page without
  touching it and its writes land in whatever memory lies below the stack.
- **Fenced?** No.
- **Rust.** rustc emits stack probes itself (`probe-stack=inline-asm`) on x86 and x86-64, and on
  AArch64 in recent versions; I did not check `aarch64-apple-darwin`.
- **Recommendation.** Make it hold everywhere, as `[RT-12]` already requires: pass the probe flag on
  every target whose compiler accepts it, refuse a toolchain that cannot probe (as E9011 refuses one
  that cannot turn contraction off, `[TYP-9a]`), and with `--emit c` write the required flags beside the
  sources. No cost beyond the probes themselves (one touch per page of a large frame).
- **Settled?** No verdict needed: `[RT-12]` and `[PHIL-10]` (218-223, "This holds in every build
  profile and under every manifest setting") already say it must hold. Not for the owner.
- **How sure.** Unsure whether Apple's Clang probes without being asked. Decided by compiling `fill`
  with Apple Clang `-O2` and looking for a probe loop in the assembly, or running it on the macOS
  release machine (`[TOOL-1]` 5086-5090).

### F8. Another package's overlay makes calls safe in a package with no `unsafe`

- **Feature.** "A package may distribute an overlay for a foreign module ... `overlay` takes one file
  or a list, applied first to last (a dependency's before the package's own)" (`[FFI-30c]` 4760-4766).
  `[TIER-1]` (151-155): safe code may call such an operation only "if an `unsafe` boundary in the same
  package discharges it". The overlay's checks against the header are by name and parameter count
  only (`[FFI-12]` 4853).
- **How it breaks safety.** Kinds 5 and 1. A dependency ships `unsafe overlay c "lib.h":` written for
  version 1 of `lib.h`, where `fn name_of(h: borrowed one) -> nul_terminated from(static)` was true;
  the program imports its own `vendor/lib.h`, version 2, where the text now lives in the handle, with
  `import c "vendor/lib.h" with (overlay = [<the dependency's overlay file>]) as lib`, then
  `n = lib.name_of(h)`, `lib.close(h)`, `println(n)`: a read of freed text in a package that contains no
  `unsafe`. (How the path to a dependency's overlay is written was not checked.)
- **Fenced?** Not in the importing package, contrary to `[TIER-1]`'s "in the same package".
- **Rust.** A `-sys` crate builds against its own pinned header or library in its build script, so
  its safe wrappers and the C they describe come together; a crate cannot apply its bindings to
  another crate's header.
- **Recommendation.** An overlay records the header (by its hash, which `.embind` already holds,
  `[FFI-14]` 4857) it was written for; applying it to another header is an error naming both. Or the
  import line that names another package's overlay says `unsafe`. No run-time cost.
- **Settled?** Verdict 60 and `[TIER-1]`. Not for the owner.
- **How sure.** Fairly sure of the rule conflict; the version mismatch is a scenario, not a test.

### F9. C source compiled into the program runs without being called

- **Feature.** `import c "./bridge.c"` compiles `bridge.c` into the program (`[BLD-FFI-1]` 5002-5004);
  `implementation = ["MINIAUDIO_IMPLEMENTATION"]` compiles a whole library (`[FFI-29b]` 4745);
  `link = [...]` and `[link] libs` link libraries (`[BLD-FFI-3]` 5011).
- **How it breaks safety.** Kind 5. Program:

  `bridge.c`
  ```c
  #include <string.h>
  __attribute__((constructor)) static void early(void) { memset((void*)0x10000, 0, 64); }
  int add(int a, int b) { return a + b; }
  ```
  `main.em`
  ```ember
  import c "./bridge.c" as b

  fn main():
      println("hello")
  ```
  When run, `early` runs before `main` and writes to an address it does not own (a crash, or memory
  corrupted), though the program calls nothing of `bridge.c` and has no `unsafe`.
- **Fenced?** No. `[TIER-1]`'s three boundaries are `unsafe:`/`unsafe fn`, `unsafe extern` and
  `unsafe overlay`; compiling C source is none of them.
- **Rust.** Not fenced either: a build script compiles C with the `cc` crate and no `unsafe`.
- **Recommendation.** Read verdict 60 as written: the two forms that compile C into the program say
  `unsafe` (`unsafe import c "./bridge.c"`, and an `unsafe import c ... with (implementation = [...])`),
  as `unsafe extern` and `unsafe overlay` already mark the other places foreign code is trusted;
  `link = [...]` and `[link] libs`, which cannot be fenced in the source, stay listed by `ember tcb`
  (`[TCB-4]` 6633, "external library"). An import of a header alone stays as it is: its functions run
  only when called, and those calls are fenced.
- **Settled?** Verdict 60. Not for the owner (only programs that compile C change). Rust does less.
- **How sure.** Sure.

### F10. Thread checks that run only in the `debug` profile

- **Feature.** `[FFI-33a]` (4933-4935): "in `debug` a retain or release of a non-`@sync` object from a
  thread other than its creator panics"; `[FFI-33b]` (4930-4932): a foreign object contracted
  `threads(creator)` "in `debug`, panics when dropped on another" thread. The runtime compiles both
  checks only with `EMBER_RT_DEBUG_ACCESS` (`runtime/ember_rt/include/ember_rt.h` 1515-1526;
  `runtime/ember_rt/src/ember_rt.c` 2553-2560), which only `debug` defines
  (`compiler/ember_build/src/lib.rs` 1835 for MSVC, 1973 for Clang and GCC). Today the trampoline's check of a `threads(creator)`
  callback is debug-only too (`V5/rules.md` 995, `tests/conformance/FFI-33b`).
- **How it breaks safety.** Kinds 2 and 4. A C library that calls a `threads(creator)` callback from
  its own worker thread, in a `release` build: nothing stops it today, and the callback changes a
  one-thread value from two threads.
- **Fenced?** The fact they back (the thread contract) is asserted in an `unsafe` overlay or extern
  block. But `[PRF-1]` (5257-5262) promises "Every profile ... performs the same safety checks", and
  the table of `[PRF-3]` (5263-5286) does not list these. V5 makes the trampoline's check run in every
  profile (`rules.md` 995, 1013) and replaces `@sync` with compile-time rules (E7008, E7015), but says
  nothing about the other two checks.
- **Rust.** No such run-time check; `Send`/`Sync` are compile-time only, and a false C thread
  contract is the `unsafe` code's fault.
- **Recommendation.** One rule for every profile: either run them everywhere (one compare per retain of
  an object C may hold), or say they are debug diagnostics backing an asserted fact and that the
  compile-time rules (E7008, E7015) are what keep threads apart; `[PRF-3]` then lists them.
- **Settled?** By `[PRF-1]` and `[PHIL-10]`; no verdict needed. Not for the owner.
- **How sure.** Sure of the code; low harm (it backs a fenced fact).

### F11. The C++ header's copyable handle is keyed to `@sync`, which V5 removes

- **Feature.** `[BLD-FFI-5a]` (6475-6477): "The C++ header gives a copyable smart handle only to
  `@sync` classes, whose counts are atomic"; ODR-162 (8398) lets an `@export` function hand C++ an
  object of a `@sync` class. V5 removes `@sync` (`vocabulary.md` 178): whether an object's count is
  atomic is said on the value (`thread_shared T`, R9), not on the class.
- **How it breaks safety.** Kind 2. If the `--cpp` header gave every class a copyable handle, C++
  copying one from two threads would change a one-thread (non-atomic) count at once: a lost count,
  then an object freed while still used.
- **Fenced?** Not applicable today; in V5 the rule refers to a word that no longer exists. (Today's
  spec also contradicts itself: `[FFI-31b]` 4957-4960 forbids class handles in exported signatures.)
- **Rust.** cxx shares Rust objects with C++ only as `Box`/references; an `Arc` crosses only through
  hand-written code.
- **Recommendation.** Restate it for V5: a copyable C++ handle only for a `thread_shared` value's type;
  any other class object is not handed to C++ (keep `[FFI-31b]`), and decide ODR-162 against
  `[FFI-31b]`.
- **Settled?** Follows from area E's removal of `@sync` (`vocabulary.md` 178) and verdict 61. Not for
  the owner.
- **How sure.** Sure the rule is stale; no program can be written until V5 restates it.

---

## Part B. Features that are fenced (inside `unsafe` today and in V5)

Each is possible only inside `unsafe` (or an `unsafe extern` / `unsafe overlay`, `[TIER-1]` 151-155).
Recommendation for all: keep the fence. Settled by verdict 60 unless said otherwise. None is for the
owner.

**B1. Reading, writing and offsetting through a raw pointer** (`*T`, `*mut T`; IV.5 1149-1158). Kinds
1, 5. Program: `xs = [1, 2, 3]`, `p = xs.as_ptr()` (safe: making a pointer), then
`unsafe:` / `println(p.add(10).read())`: reads past the list. Fence: `[UNS-1]` 3020-3021 "An `unsafe`
context is required to: dereference, read or write through `*T`/`*mut T`; offset a pointer"; outside it
E3100 "reading through a raw pointer requires an `unsafe` block"
(`compiler/ember_typeck/src/lib.rs` 17416); V5 R16 913-919 and E3100 (`rules.md` 1113). Rust: making a
raw pointer is safe, dereferencing and `add` are `unsafe`.

**B2. Casts between pointers, or a pointer and an integer** (`[TYP-7]` 947). Kinds 1, 5. Program:
`unsafe:` / `p = 4096 as *mut u8` / `p.write(1)`: writes to an address the program does not own.
Fence: `[TYP-7]` "requires `unsafe`"; `[UNS-1]` 3021-3022; V5 "as today". Rust: the cast is safe, the
write `unsafe`; Ember is stricter.

**B3. `transmute`** (`[UNS-5]` 3034). Kinds 1, 5. Program: `unsafe:` / `b = transmute[u8, bool](2)`: a
`bool` that is neither true nor false. Fence: `[UNS-1]` 3023; V5 `vocabulary.md` 338 ("outside
`unsafe`" refused). Rust: `mem::transmute` is `unsafe`.

**B4. Inline assembly** (`[UNS-6]` 3037). Kinds 5, 6. Program: `unsafe asm("mov qword ptr [0], 0")`
writes to address 0. Fence: `[UNS-6]` "`unsafe asm(…)`", E5090 on the C backend; V5 `vocabulary.md`
339. Rust: `asm!` is `unsafe`.

**B5. `unsafe fn` and values of `unsafe fn(…)` types** (`[FN-6]` 1589-1595, `[UNS-7]` 3039). Any kind,
by what the function does. Program: B1's `read_u32_le` (SPEC 3008-3010) called with a pointer to two
bytes reads two bytes past them. Fence: `[UNS-1]` 3021-3022; E3100 "calling an `unsafe fn` requires an
`unsafe` block" (`compiler/ember_typeck/src/lib.rs` 22807); an `unsafe fn` coerces only to an `unsafe`
callable type (35979-35984). V5 R16 913-917 is stricter than today: inside an `unsafe fn` body each
unsafe step sits in its own `unsafe:` block (E3100), and every `unsafe fn` carries `@safety` (L3015).
Rust: the same, with `unsafe_op_in_unsafe_fn` a warning in edition 2024.

**B6. A foreign call whose contract is not complete** (`[FFI-2]` 4610-4617, `TODO(...)` words
`[FFI-11c]` 4849-4852, `[FFI-38]` 4710-4712). Kinds 1, 5. Program: `import c "string.h" as cs` with no
overlay, then `unsafe:` / `cs.memset(p, 0, 1000)` on a 10-byte block. Fence: `[FFI-2]` "A call to a
foreign function whose contract contains an `unknown` fact requires `unsafe`"; E5002 "a foreign call
without a complete safe contract requires `unsafe`" (`compiler/ember_typeck/src/lib.rs` 22795); V5
keeps it (R12 745-756 counts these calls too). Rust: every foreign call is `unsafe` unless the
declaration says `safe fn` inside `unsafe extern` (edition 2024).

**B7. Facts asserted about foreign code.** `safe fn` in an extern block (`[FFI-10]` 4631-4638; a plain
`extern` block is refused, `compiler/ember_typeck/src/lib.rs` 20378, 58879); overlay entries
(`[FFI-2]` 4613-4615; `[GRM-35]` 4813-4818, E5067 for an overlay that states facts without `unsafe`);
the contract words (`borrowed`, `owned`, `retained`, `returns_owned`, `exclusive`, `aliased`, `count`,
`nullable`, `from(...)`, `threads(...)`, `destroys`, `[FFI-11]` 4819-4840; a parameter with no word
"does not retain", `[FFI-35a]` 4860); `@ffi(no_callbacks)` (V5 R12 749-751, E5069 on one that takes a
callback); `@ffi(immutable)` (`[FFI-8]` 4731); `@ffi(link_name=...)` (`[FFI-49]` 4675); C++ overlay
method lines (`[FFI-40a]` 6600-6601) and `owner="foreign"` (`[FFI-17d]` 6581);
`[[clang::lifetimebound]]` (`[FFI-11f]` 6525, an asserted fact, and its calls still need an asserted
safe-to-call fact); `@ffi(noexcept)` (`[FFI-24b]` 6546, whose thunk still catches, so a false one only
panics). Kinds 1, 2, 5. Program (V5 spelling):
```ember
unsafe extern "C":
    @ffi(result(borrowed, nul_terminated, from(static)))
    safe fn last_error() -> CStrView          # false: C frees this text at the next call
    safe fn clear_error()

fn main():
    e = last_error()
    clear_error()
    println(e)                                # reads freed text
```
Fence: the false fact is written inside `unsafe extern "C":` ("`unsafe` records that the programmer
asserts the signatures", `[FFI-10]` 4631-4632) or an `unsafe overlay`, and `ember tcb` lists each
(`[TCB-1]` 6622-6629). V5 keeps all of them (`vocabulary.md` 159, 166, 179, 367-371). Rust: the same
model (`unsafe extern` blocks with `safe fn`, and safe wrappers vouched for by their authors).

**B8. Variadic C calls** (`[FFI-8]` 4738). Kinds 1, 6: C reads further arguments by the types its
format names. Program: `unsafe:` / `v.sum_wide(1, narrow as c_int)` where C reads an `int64_t`
(`area-E.md` 2537, 2556-2561, 2603-2605). Fence: `[FFI-8]` "variadic calls only in `unsafe`"; E5002 "a
call of a variadic C function requires `unsafe`" (`compiler/ember_typeck/src/lib.rs` 22788); V5 adds E5013
for C's promotions (`rules.md` 1114). Rust: calling a variadic C function is `unsafe`.

**B9. Reading a field of a C union** (`@layout(c, union)`, `[FFI-8]` 4727). Kind 1. Program: a union
of `i: i32` and `f: f32`, write `i`, then `unsafe:` / read `f`. Fence: `[FFI-8]` "whose fields are read
only in `unsafe`" (`tests/conformance/FFI-8/reject_a_union_field_is_read_in_unsafe.em`); V5 keeps
`@layout` "same". Rust: reading a union field is `unsafe`.

**B10. Foreign statics** (an `extern static` read without `@ffi(immutable)`, a foreign `static mut`;
`[FFI-8]` 4731, `[STA-1]` 1803-1807, `[FFI-41]` 6603-6608). Kinds 2, 5. Program: `unsafe extern "C":`
/ `static mut counter: c_int`, then `counter += 1` from two threads. Fence: "reading it needs `unsafe`
unless `@ffi(immutable)` asserts ...", and every foreign `static mut` access needs `unsafe`; V5 "a
foreign `static mut` keeps `[STA-1]`'s `unsafe`" (`vocabulary.md` 134). Rust: extern statics are
`unsafe` to touch.

**B11. `adopt[T](p)`** (`[FFI-36a]` 4883-4887; `std/src/ffi.em` 195). Kind 1 (a double free if two
owners). Program: `unsafe:` / `a = ffi.adopt[Widget](p)` / `b = ffi.adopt[Widget](p)`: two owners of
one C object. Fence: `pub unsafe fn adopt`; E3100 "adopting a foreign object requires an `unsafe`
block" (`compiler/ember_typeck/src/lib.rs` 16541); the second adopt also panics in every profile
(`[FFI-36b]` 4888-4890). V5 keeps it (`vocabulary.md` 310). Rust: `Box::from_raw` is `unsafe`.

**B12. `into_raw()`.** Kind 3 (a leak), 1 (a double free if given back wrongly). Today it is **not
fenced**: `pub fn into_raw(owned self) -> *mut T` calls `mem.forget` (`std/src/ffi.em` 177-181). V5
fences it: "Outside `unsafe:` it is refused (E3100: "`into_raw()` stops Ember from freeing the widget,
so a mistake leaks it or frees it twice; write it inside `unsafe:`")" (`rules.md` 920-927,
`vocabulary.md` 314). Program: `unsafe:` / `p = second.into_raw()` / `set_user_data(p)`. Settled by
verdict 91. Rust: `Box::into_raw` is safe (Rust counts a leak as safe); Ember is stricter.

**B13. `get_unchecked` and `assume_init`** (`[UNS-1]` 3023; 2538, 2971-2972, 3091). Kind 1. Program:
`unsafe:` / `println(xs.get_unchecked(10))` on a 3-element list. Fence: `[UNS-1]` "call
`get_unchecked` or `assume_init`"; V5 `vocabulary.md` 326, 335. Rust: `get_unchecked`,
`assume_init` are `unsafe`.

**B14. `assume_disjoint`** (`[DSJ-7]` 3171). Kinds 1, 6: the two views are then compiled with
`restrict`/`noalias` (`[CG-C-4]` 5935-5938), and C may reorder writes to one place. Program: `unsafe:` /
`(a, b) = assume_disjoint(xs.as_mut_span(), xs.as_mut_span())`. Fence: "`unsafe assume_disjoint(a, b)`";
E3100 "`assume_disjoint` requires an `unsafe` block" (`compiler/ember_typeck/src/lib.rs` 16261); V5
`vocabulary.md` 337; `@no_alias_checks` forbids even the checked `assert_disjoint` (R16 917-918).
Rust: `split_at_mut` is checked; the unchecked forms are `unsafe`.

**B15. `@assume_noalloc(call)`** (`[EFF-7]` 3269). Kind 4 (skips the `@noalloc` contract's check; not a
memory break). Program: inside a `@noalloc fn`, `unsafe:` / `@assume_noalloc(xs.push(1))`. Fence: "an
expression admitted only inside `unsafe` (`E0104` elsewhere)"; V5 `vocabulary.md` 217. Rust: no
equivalent contract.

**B16. `UnsafeCell.get`** (`[UNS-10]` 3045-3049). Kinds 1, 2. Program: `unsafe:` / two `*mut` from
one `UnsafeCell` written through as two `ref mut`. Fence: "`unsafe get(self) -> *mut T`", never
`Sync`; V5 `vocabulary.md` 303. Rust: `UnsafeCell::get` is safe, using the pointer is `unsafe`.

**B17. `Send` declared by hand, and implementing an `unsafe interface`** (`[UNS-1]` 3023-3024). Kind 2.
Program: `unsafe extend Counter implements Send:` for a `Counter` holding a raw pointer to a one-thread
count, then the counter sent to a thread. Fence: E3100 "implementing `Send` by hand requires
`unsafe extend`" (`compiler/ember_typeck/src/lib.rs` 14011); V5 R15 896-897, `rules.md` 1113. Rust:
`unsafe impl Send`.

**B18. `Volatile[*T]`** (`[UNS-5]` 3034, `[FFI-8]` 4723). Kind 5. Program: `unsafe:` /
`Volatile(reg).write_volatile(1)` on an address from an integer. Fence: E3100 "a volatile access through
a raw pointer requires an `unsafe` block" (`compiler/ember_typeck/src/lib.rs` 16654); V5 keeps it.
Rust: `ptr::read_volatile` is `unsafe`.

**B19. `static mut` (a module variable that may change).** Kind 2. Today fenced: `[UNS-1]` 3023
"access a `static mut`", `[STA-1]` 1805-1806 "exists only for foreign interfaces and every access to it
requires `unsafe`". V5 removes the fence (verdict 8; `vocabulary.md` 134) and makes it safe by a rule:
it is reached only by the main thread and its scoped tasks unless `guarded by` a static lock (E7012,
R15 899-901; `area-E.md` 1416-1420, which also covers exports and C callbacks by their thread
contract). The one gap I found in that rule is F1 (a foreign call re-entering an export). Rust:
`static mut` needs `unsafe`, and edition 2024 refuses references to one.

---

## Part C. Searched, and nothing turns a check off

- **Manifest keys.** "No manifest key turns off a safety check or changes what a program means"
  (`[MAN-8]` 5177-5178); an unknown key is E9001. `[c] flags` and `--cc-flags-from-target` reach only
  the shims and C sources, not Ember's own C (`compiler/ember_driver/src/lib.rs` 5015-5020, 7153).
- **Profiles.** `[PRF-1]` 5257-5262 and `[PRF-3]` 5263-5286: what differs is optimisation, debug
  information, stripping, backtraces, `debug_assert` (side effects are W2016), the leak and lock-order
  reports, sanitizers (added checks), LTO ("No correctness property depends on LTO", `[BLD-6]` 5215)
  and Ember's own rewrites ("checks a proof removes"). The exceptions are F10's debug-only checks.
- **`@no_runtime_checks`**: removed in V5 with a message (`vocabulary.md` 237); **`@static_safe`** becomes
  `@no_alias_checks`, which forbids run-time alias checks and turns none off (`vocabulary.md` 183).
- **`@fastmath` and `@fp(allow_fma)`** (`[TYP-9]` 1001-1013, `[CG-C-11]` 5916-5922): the relaxed C
  file is compiled without `-ffinite-math-only` on Clang and GCC
  (`compiler/ember_build/src/lib.rs` 1500-1517); with MSVC it gets `/fp:fast`, but the conversions
  and range tests a NaN could break are done by bit tests or out of line in strict code
  (`runtime/ember_rt/include/ember_rt.h` 722-740, 2575-2583). Not a finding.
- **`@overflow(wrap|saturate)`** (`[TYP-8]`): defined results, never undefined C (`[CG-C-1]` 5906-5911).
- **Command-line flags.** `--no-leak-check` (a report), `-D warnings` and `--require-evidence`
  (stricter), `--interp` (no foreign calls or threads, `[CLI-21]` 5070-5072), `--report=engine` (a
  report, `[CLI-10]` 5108-5111), `ember bind --flag` (the shim's `_Static_assert`s re-check every
  layout, `[FFI-5a]` 4625-4627). `--emit c` is F7.
- **`restrict` and `noalias`** are derived, never assumed (`[CG-C-4]` 5935-5938, `[CG-LL-3]` 6119-6123,
  `[SIMD-3]`); V5 never compiles `this` with noalias (verdict 18).
- **`@grade`** changes only the trust report, never a check (`[TCB-2]` 6630-6631).
- **Counts** never wrap (`[RT-7]` 6075-6076; verdict 59 and V5 R9 keep the check).
- **Versions.** `[VER-*]` change nothing a program does; the runtime ABI is checked at link or load
  (`[VER-4]` 116-119, `[ABI-1]`); V5's 8-byte header is a new runtime ABI and must bump
  `EMBER_RUNTIME_ABI`.
- **Exports and callbacks that panic** abort at the boundary (`[FFI-25]` 4954-4956, `[FFI-39e]` 6557);
  a `count(n)` length that does not fit C's type panics before the call (`[FFI-10]` 4649-4651).
