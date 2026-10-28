# The lead's rulings for the 0.9.15 compiler work (2026-10-27)

These settle what the gap map and its critic (`gap/10-critic.md`) left open. The authority for each
ruling is one of:
- the spec's own text;
- an owner verdict (`tasks/96/redesign/JOB.md`);
- the measuring phase;
- the owner's delegation: elegant, high-performance answers only, never widening an owner verdict.

None is an owner verdict, and the owner may overturn any. Rulings that change spec text go into
0.9.15_Hardened_2, with the milestone that needs them.

## 1. Who does which work (the critic's C1 to C10)

| work | owner item | the others |
|---|---|---|
| deleting the run-time access machinery (`insert_shared_accesses_all`, `access.rs`, `loop_access.rs`, `long_access_lint.rs`, `BeginAccess`/`EndAccess`, the runtime's access helpers) | A4-8 | A2-11 keeps only the `RefCell` guard loans and `check_refcell_call`; A6-8 and A9-5 depend on A4-8 |
| the summary's facts (per parameter: ends before last use, `ref mut` reach, "only adds", fields of `this`, stored-function fault lines) | A4-2 | A2-9 supplies the borrow transfer functions inside A4-2 |
| raising the codes through calls | A2-10, through A4-4's mapping | |
| E3031 | A4-5 | |
| `noalias`/`restrict` on `ref mut` parameters | A2-14 | A7-19 is folded in; A9-10 adapts AI's code only |
| kind tests, `is T as name`, `cast`, E2036/E2037/E2106 | A4-14 (checker) | A7-7 parses; A1-20 and A7-12's checker half are merged into A4-14 |
| the type representation of the ownership words | A7-10 | built in area 7's shape (SD1), which has the `using` slot; area 3 gives `shares`/`weakref` their meaning, area 1 `owns` |
| `into_raw()` | A1-11 | A5-4 is the `ember tcb` listing only |
| `this`, `static fn`, parameter and call words, `copy of`, `owns` | the parse is area 7's (M1); the meaning is area 1's | A7-8 does not build what A1-5, A1-7 and A1-8 build |
| views kept anywhere, re-pointing | area 2 (A2-3, A2-8) | A7-26 and A7-27 keep only their TYP texts |
| every mechanical respell of `std/` and `tests/` | A7-9 (the migrator, M1) | area items keep only rewrites and retires |
| HEAP-3, -4, -5, -6, -7 | area 3 (A3-2, A3-3) | |
| HEAP-10 | area 5 (A5-8) | |
| CLS-6 | area 3 (A3-6) | |
| FN-3 | area 2 (A2-2) | |
| EXC-18's ten `run_fail_*` files | rewritten under BRW-17 (area 2) | |
| EXC-18's five `accept_*` files | kept, under BRW-17 or LT-1 | |
| CELL-10's six E3023/E3022 files | area 2 | |
| DRV-1 | area 7 | |

## 2. Shared representations

- **The header (S-1, OBJ-1)** carries no flags.
  - A `thread_shared` object is told apart by a twin type index (area 3's D6).
  - The pin of an object given to C is the low bit of the reload-only `live_prev` pointer, which is
    aligned. Builds without reload never move objects.
- **Kind tests (S-2)** stay one compare: twins are numbered inside their class's pre-order span. A class
  family that a reload extends falls back to the base-chain walk; that happens only in reload builds.
- **The weak count (S-3).**
  - Whether a family gets the 12-byte header is a whole-program fact, as OBJ-1 says ("some `weakref`
    link in the program"). It is computed with the summaries, and interface records carry it (they exist:
    `ember_build/src/interface.rs`, schema 14).
  - A dependant that adds the first `weakref` to a library's class rebuilds the library's modules, as
    a changed summary does.
- **`owns dyn I` (S-5)** is two words, `{data, vtable}`, with the type index in the vtable. A kind test
  is one load and one compare, and boxed structs (ECS pools, `Array[owns dyn Shape]`) carry no 8-byte
  header.
- **The interface schema (S-4).** The summary goes in the schema after AK's and AJ's bumps (16 or later).
- **The summary's contents (S-7).** Area 4 adds three facts:
  - "may make an owner of parameter k or of `this`" (OBJ-5);
  - "opens the owner edge `T.f`" (WK-7);
  - "what it hands to C".
- **A function value's possible targets (S-8)** are found by area 4's SD2: the exact callable type with
  its words, result, `once`, `unsafe` and `thread_shared`, over AJ's conversion sites. For `dyn`, the
  implementations in the program's upcasts.
- **The function value's first word (S-9)** is whatever area 4's SD3 settles. Areas 6 and 9 cite it.
- **Spelling transition (S-10).** No window with both spellings: M1 refuses and migrates in one commit.
  A feature's forerunner stays only until the milestone that builds its replacement.

## 3. Cycles (Y1 to Y7)

- A1-7 depends on A4-2, not A4-7.
- A1-17 is split: (a) `static mut` as a place, before A4-2; (b) its per-call and thread checks, after
  A5-18.
- A1-8 is the words and E3028 only; BRW-15 and BRW-22 depend on A1-8.
- A1-20 is merged into A4-14.
- A1-9 depends on A3-2 only.
- A3-4 and A4-8 land together (M6b).
- M1 is parse, refusals and the migrator: the object's change-mode is read from today's `mut self`
  until A4-2 lands.
- A1-10 lands with A1-11 (M4).

## 4. Questions the areas raised that are settled here

| question | ruling | why |
|---|---|---|
| area 2 Q1: may `from` name any field of `this`? | Yes. `from log` with `log: Array[int]` points into that field and what it owns | LT-1's own example and seven design tests write it; LT-1a's "view-typed field only" is a slip, fixed in H2 |
| area 2 Q2: is the list of index proofs closed? | Yes: two different constants, `j` from `i + 1..`, inside `if i != j:` | XVIII.4: every implementation accepts the same programs; verdict 98 names exactly these |
| area 2 Q3: changing elements through `enumerate` | `for i, ref mut x in xs.enumerate():` | verdict 95's own map form, `for k, ref mut v in counts:`; `iter_mut` stays gone (SPN-4) |
| area 2 Q4: a plain parameter across calls | across a call, only "not ended or moved" is checked (BRW-21); a change through a link is allowed, and the compiler never assumes the value unchanged (verdict 18, the turret) | verdict 18 |
| area 3 Q1: when does a line close a loop of owners (E3032)? | refuse any store whose owner type can reach the holder's type through owner edges, unless the compiler proves it unreachable or opened again; the list of proofs goes into the spec (H2) | verdict 99: "only accepted if the compiler can prove that they get freed" |
| area 3 Q3: drops in the deferred-drop region | drops run at once; only the blocks' memory waits for the region's end | the output order is then the same as with no region; M2's `freeonly` salvage measures this form |
| area 1 Q2: which managers have several instances? | a manager type with no fields has one instance (no word, no instance on the making line); every other has several | the same test as Rust's zero-sized allocators; `Global` and `System` already qualify |
| area 1 Q3: `HOOK = v` for a plain `static` | E3023, with help naming `static mut` or another local name | STA-1: a `static` never changes; today's shadowing (D-692) is a trap |
| area 5 Q1: E3100 or E5002 for an unchecked foreign call outside `unsafe`? | E3100 (UNS-1); E5002 stays for contract errors | UNS-1 is the newer, specific rule |
| area 4 Q2 / area 7 Q1 (bounds): the case where two interfaces both supply a method | this half is settled: refused with both bounds in the message | Rust's ambiguity rule; the "some written, rest inferred" half was the owner's (Q-H): verdict 113, a full list or none |
| area 9 Q1: does a named function have a type of its own? | yes (ODR-278, the owner's "yes", 2026-10-23); `if c: f = a else: f = b` gives `f` the callable type both convert to | the owner's ruling; the spec's CLO-1 is widened in H2 to say so |
| area 9 Q2: is a variable holding a function item a "real variable" for capture? | a zero-sized function item holds nothing to capture, so naming it captures nothing (AJ's rule stays) | assigning the same item again changes nothing, so no program can tell |
| area 9 Q3 / area 6 Q3: cdylib exports for `thread_shared` class handles | a cdylib also exports the functions its header declares for exported class handles | BLD-FFI-5a is the specific rule; FFI-28's list gains that line in H2 |
| area 6 Q1: is a `Retained` token an owner edge to the callback's state? | yes; verdict 62's test is rewritten with a `weakref` capture | WK-5 says so; verdict 99 refuses the loop it closes |
| area 6 Q2: destructors of one foreign type | one destructor per foreign type; a mismatch is E5002 | a foreign `owns T` stays one word |
| area 6 Q4: the trampoline's tagged number on 32-bit targets | the 48/16 layout is 64-bit; 32-bit targets use 16/16 without the "no user-space address" promise | the compiler builds 64-bit only today |
| area 8 Q2: `ecs.type_hash` needs a type's name | a general compiler-known `std.core.type_name[T]()` | general, as Rust's `std::any::type_name`; no ECS special case |
| area 8 Q3: `@component(layout=...)` | its row is removed; `@component` is E0104 | verdict 100 (EnTT, no SoA pools) |
| area 8 Q4: `Pool[T]` or `Storage[T]` | one type, `Storage[T]` (EnTT's `storage<T>`); `std.collections.Pool` keeps its name | no collision |
| area 8 Q5: CLI-20's migrate list | add the bracket lines (GRM-40), `x: ref T` parameters, `T: Sync` bounds, `iter_mut()` loops | the list must cover what M1's migrator does |
| area 1 Q1: `owned` inside `@ffi(...)` | **overturned by the owner (verdict 102, Q-A): the contract word is `owns`**, and `owned` is refused everywhere | the owner: "owned is now owns" |

## 5. For 0.9.15_Hardened_2: the owner's verdict 101 (two numbers for `thread_shared` counts) — written into Hardened_2 (2026-10-28)

To write into the spec with the other rulings above, in one hardening:
- `[OBJ-1]`: a `thread_shared` object's header (A, the type index, the weak count, B with its flags, the
  queue link);
- `[RC-4]`, `[RC-5]` point 2, `[WK-12]`, `[RT-8]`, `[RT-10]`: which number each +1 and −1 goes to; the
  hand-over to the making thread; rule Q;
- the check points (the compiler places them; always before a thread ends) and the delayed deletion they
  allow, for an object that travels inside another object;
- the publish points of fix 2 (lock-guarded `thread_shared` containers, channels, a spawned thread's or a
  kept callback's capture);
- the cost table (`[COST-3]`): the numbers measured in `measure-0.9.15/brc/REPORT.md`.

The compiler work falls in milestone M9 (threads), with A3-4's header in M6b.

## 6. Choices made while writing 0.9.15_Hardened_2 (lead, 2026-10-28)

Each is the smallest sound reading of a verdict or ruling above; the owner may overturn any.

| where | choice | why |
|---|---|---|
| `[OBJ-1]` (verdict 101) | a `thread_shared` value's header is 32 bytes: home count, type index, weak count, an unused word, the away word (count × 8 and three marks), the queue link | the measured layout (`measure-0.9.15/brc/brc.h`); the type index and the weak count stay at the offsets every other header uses |
| `[RC-4]` point 3 (verdict 101) | check points at every loop's back edge, before every call that may block the thread, and when it ends | the report left placement "to design"; these bound how long a queued value waits by the thread's own progress, and each is one test of the queue |
| `[RT-7]` (verdict 101) | the home count keeps the exact compare at 4,294,967,295; the away and weak counts the half-range stop | the home count is one thread's, as a `shares` count is |
| `[WK-7]` (area 3 Q1) | the proofs that a store closes no loop: (1) the stored value was made in this function and nothing but such values were stored into what it owns; (2) the holder was made in this function and no owner of it was stored anywhere yet | both are sound and checkable within one function; (1) is the design tests' tree built top down |
| `[PAR-3]` (verdict 105) | a `@parallel` whole-number reduction's chunk totals stop on overflow too | the same rule as `[THR-21]`'s task totals; chunk boundaries do not depend on the machine (`[PAR-5]`), so neither does the stop |
| `[FFI-11]` (verdict 102) | `returns_owned(destructor=f)` keeps its name | it is one word, not the removed `owned`; the owner may rename it |
| `E2122` (verdict 103) | the new code is `E2122`, next to `E2119` | never used in any spec or in the compiler |

## 7. Choices made while writing 0.9.15_Hardened_3 (lead, 2026-10-28)

Each carries out an owner verdict where the verdict leaves a detail open; each is told to the owner.

| Rule | Choice | Why |
|---|---|---|
| `[TYP-17]`, Part III (verdicts 109-112) | `needs` before every bound, a function-shaped one too: `[F: needs fn(Item)]`, `[U, F: needs once fn(owns T) -> U]` | verdict 109: what a type parameter needs is written after `needs`; one spelling for every bound |
| Part III (verdicts 109-112) | `dyn` and `some` keep `+` (`dyn Display + Send`) | they are types, not a type parameter's needs; no verdict names them, and the spec writes none today |
| `[TYP-17]` (verdict 109) | a bound without `needs`, or joined with `+`, is `E0100`, whose message gives the line with `needs` | the old spelling is refused with the new one in its message, as every removed spelling is |
| `[ALC-2]` (verdict 107) | the promise is the interface `std.mem.FoundByAddress` (no methods, `unsafe` to implement) with `const PIECE: usize = 65536`; the instance's address is the piece's first word | the verdict gives the layout; a name was needed |
| `[FFI-35a]` (verdict 114) | an object given to a `retained` parameter is marked in its header as kept by C | the hand-over must know it cannot move; the header has the room (`[OBJ-1]`) |
| `[TYP-47]` (verdict 116) | following C++: `Cs...` among type arguments spells a pack out, `values...` among a call's arguments passes the values on, `values.len()` counts them | the verdict chose C++'s spelling; these are C++'s uses of it |
| `[TYP-47]`, `[STD-9]` (verdict 117) | parameters after a pack's values have defaults and are passed by name only (`println(a, b, sep=", ")`), as after Python's `*args` | `print` keeps `sep` and `end` as a library function |
| `[STD-9]` (verdict 117) | the compiler implements `Printable` (`std.core`, prelude) for every type with `Display` or `Debug`: `Display`'s text, else `Debug`'s; no program implements it by hand | Ember has no specialisation (`[TYP-19]`), so the library cannot write "Display, else Debug" itself; a program's own functions use the same `[Ts...: needs Printable]` |

## 8. Rulings made on autopilot, night of 2026-10-28 (lead, under the delegation)

Each is reported to the owner in the morning with the real code it concerns and why. A question that is
really the owner's is listed under "For the owner", not ruled.

| Rule | Ruling | Why |
|---|---|---|
| UNS-1, UNS-12 (A5-1) | The type checker reads each module's source text (`ast::Module::source`, set by the parser) to quote code as written and name its lines. | The spec's messages do both (about 100 of H3's name a line: "`read_u32_le(buf.as_ptr())` (line 9) calls an `unsafe fn`, …"), and the checker had neither. Keeping the text costs one copy of each loaded file's text for the build: at most 458 KB for the standard library's 26 files, plus the program's own. One general facility rather than a placeholder the renderer fills in. |
| UNS-12 (A5-1) | In an `unsafe fn` body, the unsafe steps of one line are one `E3100` (the line's first diagnostic is amended as more steps arrive: `Sink::message_mut`), naming them all: "the reads through `p` on line 4 are this function's own unsafe steps". | The spec's message is per line ("put line 4 in an `unsafe:` block"); seven diagnostics for one line, each saying to put it in a block, is noise. It is merged as it is emitted, never by a pass afterwards: an incremental build keeps diagnostics by their place in the sink (`[BLD-1]`), so removing one would shift the others. |
| UNS-12 (A5-1) | A pointer's reads and writes name what its offsets are for: `p.add(1).read()` is a read through `p`; offsets are named only when the line has nothing else on that pointer ("the offset of `p`"). | The spec's own example line has four reads and three offsets and says "the reads through `p`". |
| UNS-1 (A5-1) | A refused raw pointer step (`p.read()`, `p.write(v)`, `p.add(n)`) is typed on instead of becoming an error expression. | Before, `p.add(1).read()` reported only the `add`: the `read` on its error result was never seen. What is done with a refused step's result is checked as well now. |
| UNS-12 (A5-1) | A closure written in an `unsafe fn` body is that function's: its steps are named as the function's own. A local function is its own: an `unsafe` one's steps are its own, a safe one's are safe code's. | A closure is the function's text, run as part of what the function does; a local function is a declaration with its own `unsafe` or not. |
| UNS-1 (A5-1) | A call of a declared `unsafe fn` outside `unsafe` gives the spec's message with the call as written. A call the code does not spell (a `for` loop's `iter`) names its callee: "`row` (line 12) calls `iter`, an `unsafe fn` whose callers must make true …". Every other step outside `unsafe` keeps its message and gives the block to write as a help ("put line 9 in an `unsafe:` block with a `# SAFETY:` note …"), which replaces the bare "wrap the call in `unsafe:`". | `[UNS-1]`: each message "says what the code must vouch for and gives the block to write". Quoting `row` as if it were a call would have said something untrue. |

| UNS-8 (A5-2) | `W3013` is for a `# SAFETY:` note in either place `[LEX-23]` attaches one: alone on the line before the `unsafe fn` ("… remove this line", the spec's) or at the end of its line ("… remove this note", since removing that line would remove the function). On a function with no `@safety` it says "… with `@safety("...")` above it, not with a `# SAFETY:` note, …: say this in `@safety` and remove …". | The spec's message presumes a `@safety` that is there and a note on a line of its own; the two other cases needed words that are true of them. |
| UNS-7 (A5-2) | `L3015` is reported when the body's check ends, naming the first unsafe step it took, inside `unsafe:` or not ("… that reads through the raw pointer `p`, … with what `p` must point to"); a body-less `unsafe fn` gets the message without a step. A foreign declaration in an `unsafe extern` block is not an `unsafe fn` written, so it gets none; a malformed `@safety` is its own `E0104` and no `L3015` besides. | The spec's message names the body's step, which only the body's check knows. The report sits inside the body's part of the check, so an incremental build that keeps the body replays it (`[BLD-1]`). |
| UNS-7 (A5-2) | A local `unsafe fn` may carry `@safety` (the parser allowed no attribute but `@simd`, `@parallel`, `@unroll` and `@allow` on a statement), and gets `L3015` without it. An `unsafe gen fn`'s lifted body keeps its `@safety`. | "Every `unsafe fn`" includes them; refusing the attribute while warning for its absence would leave no way to be clean. |
| UNS-8 (A5-2) | `ember tcb` keeps listing blocks and `unsafe fn`s as before; the `unsafe` implementations, now checked for notes, are listed by A5-5 (`[TCB-4]`'s sections). | One change at a time; A5-5 reshapes the listing. |

### For the owner

1. **`E5013`'s suggested conversion for a narrow integer** (`[FFI-8]`, built in A6-12). The spec's message is
   "C passes `narrow` as an `int` through `...`; if `sum_wide` reads another type, convert it first:
   `v.sum_wide(1, narrow as i64)`". `sum_wide` reads `int64_t`, so `as i64` is right there, and the compiler
   always writes `as i64`, whatever the C function reads. For `printf(c"%d\n", small)` it says
   `printf(c"%d\n", small as i64)`, which is wrong: `%d` reads an `int`. The compiler cannot see the `va_arg`.
   Options: (a) keep `as i64` as written (the spec's wording); (b) show the type C's promotion gives, `as c_int`,
   which is right whenever the function reads what C passes (the `f32` case already shows `as f64` this way);
   (c) show no type: "convert it to the type `sum_wide` reads: `narrow as T`". Changing the text is a new
   hardening. Nothing is changed until you choose. The error page `docs/errors/E5013.md` uses `sum_wide`, where
   `as i64` is right.

