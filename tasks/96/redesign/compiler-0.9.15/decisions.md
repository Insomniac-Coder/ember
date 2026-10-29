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
| UNS-12 (A5-1; **superseded** by the review's rows below: per statement, through a map) | In an `unsafe fn` body, the unsafe steps of one line are one `E3100` (the line's first diagnostic is amended as more steps arrive: `Sink::message_mut`), naming them all: "the reads through `p` on line 4 are this function's own unsafe steps". | The spec's message is per line ("put line 4 in an `unsafe:` block"); seven diagnostics for one line, each saying to put it in a block, is noise. It is merged as it is emitted, never by a pass afterwards: an incremental build keeps diagnostics by their place in the sink (`[BLD-1]`), so removing one would shift the others. |
| UNS-12 (A5-1) | A pointer's reads and writes name what its offsets are for: `p.add(1).read()` is a read through `p`; offsets are named only when the line has nothing else on that pointer ("the offset of `p`"). | The spec's own example line has four reads and three offsets and says "the reads through `p`". |
| UNS-1 (A5-1; **narrowed** by the review's rows below: only in an `unsafe fn`) | A refused raw pointer step (`p.read()`, `p.write(v)`, `p.add(n)`) is typed on instead of becoming an error expression. | Before, `p.add(1).read()` reported only the `add`: the `read` on its error result was never seen. What is done with a refused step's result is checked as well now. |
| UNS-12 (A5-1; the local function's `unsafe:` block: see the review's first row) | A closure written in an `unsafe fn` body is that function's: its steps are named as the function's own. A local function is its own: an `unsafe` one's steps are its own, a safe one's are safe code's. | A closure is the function's text, run as part of what the function does; a local function is a declaration with its own `unsafe` or not. |
| UNS-1 (A5-1) | A call of a declared `unsafe fn` outside `unsafe` gives the spec's message with the call as written. A call the code does not spell (a `for` loop's `iter`) names its callee: "`row` (line 12) calls `iter`, an `unsafe fn` whose callers must make true …". Every other step outside `unsafe` keeps its message and gives the block to write as a help ("put line 9 in an `unsafe:` block with a `# SAFETY:` note …"), which replaces the bare "wrap the call in `unsafe:`". | `[UNS-1]`: each message "says what the code must vouch for and gives the block to write". Quoting `row` as if it were a call would have said something untrue. |

| UNS-8 (A5-2) | `W3013` is for a `# SAFETY:` note in either place `[LEX-23]` attaches one: alone on the line before the `unsafe fn` ("… remove this line", the spec's) or at the end of its line ("… remove this note", since removing that line would remove the function). On a function with no `@safety` it says "… with `@safety("...")` above it, not with a `# SAFETY:` note, …: say this in `@safety` and remove …". | The spec's message presumes a `@safety` that is there and a note on a line of its own; the two other cases needed words that are true of them. |
| UNS-7 (A5-2) | `L3015` is reported when the body's check ends, naming the first unsafe step it took, inside `unsafe:` or not ("… that reads through the raw pointer `p`, … with what `p` must point to"); a body-less `unsafe fn` gets the message without a step. A foreign declaration in an `unsafe extern` block is not an `unsafe fn` written, so it gets none; a malformed `@safety` is its own `E0104` and no `L3015` besides. | The spec's message names the body's step, which only the body's check knows. The report sits inside the body's part of the check, so an incremental build that keeps the body replays it (`[BLD-1]`). |
| UNS-7 (A5-2) | A local `unsafe fn` may carry `@safety` (the parser allowed no attribute but `@simd`, `@parallel`, `@unroll` and `@allow` on a statement), and gets `L3015` without it. An `unsafe gen fn`'s lifted body keeps its `@safety`. | "Every `unsafe fn`" includes them; refusing the attribute while warning for its absence would leave no way to be clean. |
| UNS-8 (A5-2) | `ember tcb` keeps listing blocks and `unsafe fn`s as before; the `unsafe` implementations, now checked for notes, are listed by A5-5 (`[TCB-4]`'s sections). | One change at a time; A5-5 reshapes the listing. |

| UNS-12, UNS-11 (review of A5-1) | A local function's body starts outside `unsafe`, even written inside an `unsafe:` block; a closure keeps the block's `unsafe`. | A local function runs wherever it is called from and can leave the block (returned, stored): before, `fn poke(q: *mut u8): q.write(7)` written in an `unsafe:` block and returned was called from safe code unchecked, a hole in `[UNS-11]`. Rust does the same (a nested `fn` does not inherit `unsafe`; a closure does). |
| UNS-1 (review of A5-1) | Area 5 Q1's ruling is built: a call of a C function without a verified contract, variadic or not, and a read or write of a foreign `static mut`, outside `unsafe`, are `E3100` ("`abs(x)` (line 6) calls the C function `abs`, whose contract is not verified, so its callers must make true what C requires of the call; put line 6 …"), the `[DIA-18]` facts kept as notes; `E5002` stays for contracts that cannot stand. In an `unsafe fn` they are its own steps like any other. | The ruling was made (decisions.md, area 5 Q1) and not yet built; an `unsafe fn` doing FFI is the common case of UNS-12. |
| UNS-1, UNS-12 (review of A5-1) | The block an `E3100` gives to write goes around the statement the step is in: the whole compound statement for a step in its header (`elif p.read() > 0:` is the `if`'s, "lines 8 to 11"), every line of a statement written over several ("lines 7 to 8"), "put the statement on line 7 on a line of its own, in an `unsafe:` block …" for one sharing its line with a header, and none outside a function's body (a constant's value), where no block can be written. In an `unsafe fn` the steps are merged per statement, not per line. | "Indent line 5 under it" cannot be done for an `elif` or half a statement; the spec's examples are one-line statements, for which the text is the spec's. |
| UNS-1 (review of A5-1) | The merge finds a statement's diagnostic through a map checked against the sink, and lines through a line index made once per file. | The first form scanned the body's diagnostics and counted newlines from the file's start for each step: 300 refused lines took 4.96 s to check; now 0.13 s, as safe code takes (measured twice each, `target/review_a51/perf_*.em`). |
| UNS-1 (review of A5-1) | Outside an `unsafe fn` a refused pointer step is an error expression again (one `E3100` for each step the code shows, as before A5-1), and a cast of an operand already refused says nothing more; inside one it is typed on, so the statement's steps are named together. A `write` through a `*T` is only `E2020`. | Typing on everywhere had made the spec's one line 7 diagnostics in safe code; returning an error had made "`<error>` cannot be cast to `u32`" four times. |
| UNS-1 (review of A5-1) | A call is "spelled" when the code quotes the callee's name followed by `(` or `[`; otherwise the message names the callee ("`make()` (line 7) calls `iter`, …"). A local function called by its name is a call of it, not "a value of type `unsafe fn(…)`". Each step's own helps and notes join the statement's diagnostic in an `unsafe fn`; the old "wrap … in `unsafe:`" clauses are gone, the block to write saying it. Several "other" steps are counted ("the casts between raw pointers and integers"). A generator is named as written (`walk`, not its lifted `body`). | Each said something untrue or lost what the step's own refusal said (`get(i)` returns an `Option`, …). |
| UNS-1 (review of A5-1) | The call message keeps the spec's "whose callers must make true what its `@safety` says" when the callee has no `@safety`. | `L3015` now asks for one on every `unsafe fn`; the wording is the spec's. |

| FFI-55 (A6-16) | An overlay is "another package's" when the package it lies in (the nearest folder at or above it with an `ember.toml`) is neither the importing module's nor the header's; only then is its `header_hash` required and compared. | The spec exempts "an overlay applied to a header of its own package"; a package that ships a header and its overlay together cannot let them drift apart, and one's own overlay is vouched for by its own package (`[TIER-1]`). A manifest is what makes a folder a package everywhere else in the build. |
| FFI-55 (A6-16) | The hash is the header file's own content, written `blake3:<hex>` (the `.embind` record's entry for that file), not its includes'. A refused overlay is not applied, so its entries cause nothing further. | "The hash of the header it was written for" is the header's; the spec writes `blake3:9f2c…`. The tree is LF everywhere (`.gitattributes`), so the hash is the same on every platform. |
| FFI-55 (A6-16) | `ember bind --init` writes the hash on the starter overlay's first line, with a `# SAFETY(ffi): TODO(…)` note (`[UNS-8]` asks every `unsafe overlay` for one); `--merge`, which brings the overlay to the header as it is now, records the new hash. | An overlay is distributable as written; and since A5-2 a starter overlay without a note would warn on its first check. |

| Order (milestones) | M2's A4-1 and A4-2 (the call graph and the summaries) are built before M1's switch, and M1 after them. | M1's A7-8 needs "the EFF area's first summary slice" (whether a method changes `this`, which 0.9.15 infers and no longer writes as `mut self`); the critic's Y7 noted it. The owner's standing rule (2026-10-15): "if something is a dependency for something else then build that first". M2 refuses nothing new, so each piece lands and is pushed on its own. |
| EFF-2 (A4-1) | A call through a function value has the effects of every function the program stores into that exact callable type: a `Const::Fn` stored, passed or built into a value of it, a `Callable` made at it, and what callable types converted to it hold (to a fixed point); a function whose type is not known where it becomes a value counts for every callable type of its shape, which only adds. A `dyn` call has the effects of what the program's casts put in that table's slot, and of the overrides below a class method there. Where the program stores nothing of the type (a value from elsewhere), the call keeps every effect. | `[EFF-2]` as written: "every lambda and named function stored into that type anywhere in the program"; "every implementation and override the program has". Before, a callable with state and a `dyn` call had every effect; D-877's guess by parameter count and result is gone. |
| EFF-2 (A4-1) | The graph's spawn and job edges are added with A4-2, their first user; effects keep counting a spawn's own allocation and synchronisation, not what the spawned function does on its thread. | Nothing reads them before the summaries. |

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

## 9. Rulings made on autopilot, 2026-10-29 (lead, under the delegation), writing 0.9.15_Hardened_4

The owner's verdicts below are JOB.md's 118 to 124. Each ruling here was checked against
`docs/OWNER-DIRECTION.md`'s checklist.

| Rule | Ruling | Why |
|---|---|---|
| UNS-12 (verdict 118) | `safe` is a contextual keyword at the start of a statement before `:`; a `safe:` block needs no note; inside it an `unsafe:` block is unsafe again, and a function or lambda written in it is safe. | `safe:` asks for checking and vouches for nothing, so a note would say nothing (principle 3); nesting both ways keeps one rule: the innermost mark says what the code is. |
| UNS-12, UNS-3 | A `safe:` block outside unsafe code is `L3020` (new, a lint: "this `safe:` block is not inside unsafe code, so it changes nothing: remove `safe:`"); an `unsafe:` block directly inside unsafe code is `L3010`, whose description widens to say so. | A mark said where it changes nothing is said twice (principle 3); a lint, not an error, as `L3010` already was. |
| UNS-12 | `E3100` inside a `safe:` block: "`p.write(1)` (line 7) is in a `safe:` block, which is checked as safe code; move it out of the `safe:` block, or put it in an `unsafe:` block with a `# SAFETY:` note". | Both ways out compile (principle 12). |
| UNS-7 (verdict 119) | A lambda written inside unsafe code gets no `L3015`: it has no line for `@safety`; whoever calls it vouches in the note of the block the call is in. | Warning for a missing attribute that cannot be written leaves no way to be clean (the reason ODR 5 part 4, approved, allows `@safety` on a local `unsafe fn`). |
| UNS-1 (ODR 1, 2) | The statement rules (one `E3100` per statement; the block named is the whole statement) are written into `[UNS-1]` for all safe code. | They were the lead's in decisions.md §8; the owner approved ODR 2; ODR 1 is to hold in all safe code. |
| FFI-55 (verdict 121) | `ember.overlays`'s fingerprint is of each declaration the overlay describes, as the importer binds it (the `.embind` declaration with its layout); the record is written the first time the package's own build applies the overlay; a dependency's overlay with no record is `E5010`; its text form is TOML (`[[overlay]]`, `file`, `header`, one key per declaration). | What the overlay relies on is the binding, so a header edit that changes nothing Ember binds changes nothing; TOML as `ember.toml` and `ember.lock` are. |
| EFF-2 (verdict 122) | The places: each field of each struct and class apart; a list's elements with the place holding the list; a parameter with every argument given to it; a place a value is copied into gets the functions of the place it came from; a value read through a raw pointer or given by C has every function stored into its type. | The field-based flow the owner approved, with copies followed so nothing slips through, and the by-type fallback only where nothing can be followed. |
| BLD-15 (verdict 124) | The limit is 30 minutes by default, `[link] timeout` (seconds) sets another; the error is `E9042` (new); the link's command file is kept; nothing is retried. | The hangs ran 10 to 43 minutes before they were ended; a whole `bigpkg` build takes 16.6 s, so 30 minutes ends no real link of today's programs, and a project with a longer one sets the key. A retry would hide the problem (the owner's choice). |
| FFI-8 (verdict 123) | For an `f32`, `E5013`'s message names the one right type: "C passes `single` as an 8-byte `double` through `...`; write `single as f64`". | C never passes a `float` through `...`, so `va_arg` can only read a `double`. |

**For A5-5 (recorded, not done):** the C++ importer's generated `unsafe fn` wrappers say what their callers keep in a `# SAFETY:` note at the end of their `fn` line, `W3013`'s position, and `ember tcb` reads an `unsafe fn`'s note from there; since 0.9.15_Hardened_4 their bodies hold no `unsafe:` block, so the asserted `[[clang::lifetimebound]]` lifetime moved onto that line. A5-5, which reshapes `ember tcb` (`[TCB-4]`), moves those notes into the wrappers' `@safety` and lists an `unsafe fn` by its `@safety`.

**For the owner (open, not ruled):** a function imported from a header (`import c`) is declared by the
header; verdict 120 covers declarations written in an `unsafe extern` block. Should an imported C function
that is unsafe to call need `@safety` too (in its overlay entry), or not? Today it does not; nothing was
changed.

### Owner verdicts, 2026-10-29

1. **`[UNS-12]` is reversed** (owner: "the whole unsafe function has to be considered unsafe unless someone
   marks a piece of code safe"; "if you want it to be treated as safe you have to write it in safe block").
   The body of an `unsafe fn` is an `unsafe` context as a whole, as in 0.9.9. Code in it that should be
   checked as safe code is written in a `safe:` block (spelling chosen by the owner: "yes"), where an unsafe
   operation is `E3100` as in any safe code. `[UNS-12]` came from the redesign's AN-79 and was never put to
   the owner. To do: a new hardening; in the compiler, A5-1's body rule reverted and `safe:` added; the
   `unsafe:` blocks added inside `unsafe fn`s in std and the tests removed. ODR 1 (one `E3100` per
   statement) then concerns safe code: normal functions and `safe:` blocks; as built it merges only in an
   `unsafe fn` body, so it is to be made to hold in all safe code.
2. **ODR 2 approved** (the block an `E3100` gives to write goes around the whole statement: "lines 9 to 12"
   for a step in an `elif`; owner: "yes fair enough").
3. **ODR 3 replaced** (owner: "apply my ruling anything inside unsafe block is considered unsafe by default
   even functions"; earlier: "so poke is considered unsafe"). A function written inside an `unsafe:` block
   (or inside an `unsafe fn` body) is unsafe: it counts as an `unsafe fn`, its body is unsafe as a whole,
   and calling it needs `unsafe:`; a value of it has type `unsafe fn(…)` (so `make` in
   `UNS-12/reject_a_local_function_in_an_unsafe_block_vouches_for_nothing.em` must return
   `unsafe fn(*mut u8)`). Code in it to be checked as safe goes in `safe:`. D-926's fix (the local
   function's body starting outside `unsafe`) is replaced by this. A closure written inside an `unsafe:`
   block is covered the same way (owner: "yes cover this too"): it is unsafe, its value has type
   `unsafe fn(…)`, and calling it needs `unsafe:`.
4. **ODR 4 approved** (area 5 Q1, which I had wrongly called the owner's ruling: a C call without `unsafe:`
   is `E3100`, not `E5002`, since the declaration is fine and the call is the mistake; owner: "yup that
   makes sense").
5. **ODR 5 part 1 approved** (`W3013` for a `# SAFETY:` note at the end of an `unsafe fn`'s own line says
   "remove this note", not the spec's "remove this line"; owner: "okay fine", then "yes").
6. **ODR 5 part 2 approved** (`W3013` on an `unsafe fn` with no `@safety` says "… should say what its
   callers must make true with `@safety("...")` above it, not with a `# SAFETY:` note …: say this in
   `@safety` and remove this line"; owner: "yes").
7. **ODR 5 part 3 rejected** (that a C function declared in an `unsafe extern "C":` block needs no
   `@safety` and gets no `L3015`; owner: "no"). Replaced (owner: "yes that is my decision for consistency
   sake and it makes sense"): a C function declared in an `unsafe extern` block, which callers may call
   only inside `unsafe`, carries `@safety` like any `unsafe fn`, saying what its callers must make true
   (`@safety("x is not the smallest i32")` on `abs`), and gets `L3015` without it; the block's
   `# SAFETY(ffi):` note keeps saying that the declarations match C. To settle in the hardening: what
   `ember bind --init` writes for each declaration.
8. **ODR 5 part 4 approved** (a local `unsafe fn` may carry `@safety`; an `unsafe gen fn` keeps it when
   lifted; owner: "yes").
9. **A function written inside an `unsafe:` block needs `@safety`** like any `unsafe fn` (it counts as
   one by verdict 3) and gets `L3015` without it (owner: "yes").
10. **ODR 5 last part approved** (`ember tcb` keeps listing `unsafe:` blocks and `unsafe fn`s; the
    `unsafe extern`/`extend`/`overlay` places are added by A5-5, not before; owner: "yes", after "leave
    them just for now right? you will add them later?").
11. **`[FFI-55]`'s `with (header_hash = "...")` is to go** (owner: "find a better way to deal with it
    don't make the author or user work with it in code files"; the rule came from the critic's C26 and
    was never put to the owner). Directed (owner: "instead of using a lock file create another package
    config file for this and make it mandatory"): a package file of its own, written by Ember and never
    by hand, records for each of the package's overlays the header and a fingerprint of each C
    declaration the overlay describes (my proposal: per declaration, not the whole header, so an
    unrelated edit breaks nothing); a third-party overlay whose described declaration differs, or whose
    package has no such file, is `E5010`. Confirmed as written (owner: "yes"): the file is
    **`ember.overlays`**, beside `ember.toml` in every package with an overlay of a C header; Ember writes
    it on the package's first build and `ember bind --merge` rewrites it; it is mandatory (importing an
    overlay of a package without it is an error); it holds per overlay `file`, `header` and one
    `blake3:` fingerprint per C declaration the overlay describes; the overlay line carries no hash;
    `ember.lock` is unchanged. **Every overlay is checked** against its record, the package's own included
    (owner: "b"): a described declaration that changed stops the build, naming it, until `ember bind
    --merge` rewrites the record; so ODR 6 part 1 ("another package's" = the nearest `ember.toml` is
    neither the importer's nor the header's) is no longer needed.
12. **ODR 7, `dyn` calls, approved** (a call through `dyn` has the effects of the implementations the
    program passes as that `dyn`, with overrides below class methods; owner: "yes").
13. **ODR 7, stored functions: by field, not by type** (owner on the by-type rule: "that is just dumb";
    then "yes" to following each field). A call through a field has the effects of the functions the
    program stores into that field (`Scaler.by` apart from `Logger.format`); a store from another field
    carries that field's functions; where a function cannot be followed (from C), the call falls back to
    every function stored into its type. Changes `[EFF-2]`'s "stored into that type anywhere in the
    program": a new hardening first; A4-1's by-type `Targets::values` to be refined; not built or measured.
14. **ODR 7 part 3 replaced** (owner: "no find a better way to deal with C problem", then "yes"): when the
    compiler finds no function the program puts into a field (`Scaler.by`), a call through it counts as
    doing nothing, so `@noalloc fn scaled` is allowed. Without code marked `unsafe`, every function in a
    field gets there through Ember code the compiler sees; one C slips in (C's `lib_make_scaler` filling a
    `Scaler` with its own `triple`, read with `p.read()`) arrives only through an `unsafe:` block or an
    unsafe C declaration, whose note vouches for it. My first answer (refuse) and my overlay design (C
    functions stored in a field with overlay-stated effects) are not taken.
15. **ODR 8 approved** (A4-2, the summaries, before M1, whose `fn reset():` needs them to tell whether a
    method changes `this`, by the owner's verdict 86; owner: "oh yes of course").
18. **The MSVC `link.exe` hang ("For the owner" 2): find the cause and fix it, and a time limit** (owner:
    "duh ofcourse it's an issue so you have to find the cause and fix it, can't leave it like that and a
    time limit"). Next hang: save a dump of the spinning `link.exe` and fix the cause if it is ours (how
    the linker is called, where the files are); meanwhile `ember build` gives a link a generous limit,
    ends a link past it and stops with an error saying the linker hung, with the details; no automatic
    retry (it would hide the problem).
17. **E5013 answered: stop and ask, with a message that does not guess** (owner: "okay cool no overlay bs
    right?", then "yes"). A value C's default promotions would change (`i16`, `bool`, `f32`) passed
    through `...` stays `E5013`, for every variadic C function, one type or mixed; nothing is added to
    overlays. The message no longer names `as i64` for every function: "C passes `narrow` as a 4-byte
    `int`; write the size `sum_wide` expects: `narrow as c_int` for 4 bytes, or `narrow as i64` for 8"
    (for an `f32`, the `double` C passes). Spec text change: the next hardening (`[FFI-8]`'s message,
    `docs/errors/E5013.md`, whose "See also" also still names `E5002` for a variadic call outside
    `unsafe`, now `E3100` by verdict 4 above).
16. **WITHDRAWN by the owner** ("I take my approval back for the previous question, nothing makes sense
    here"); E5013 is open again, nothing of 16 stands. Was: **E5013 ("For the owner" 1): an overlay states what a `...` function reads** (owner: "okay so
    basically declaring it in the overlay itself what the function expects?", then "yes"). `sum.h`'s
    overlay says `sum_wide`'s further values are `i64`; each call converts a value that becomes one
    without loss (`narrow: i16`), and refuses one that does not (`u64` above `i64.MAX`, `f64`). Without
    the line `E5013` stays and its message gives the line to add. Spelling suggested, `fn sum_wide(count,
    values: i64...)` (verdict 116's pack dots), to confirm in the hardening. Options (a)-(c) (which type the
    advice names) are superseded: no fixed type is right for every function. A `...` that mixes types
    (`printf`): asked next.

## 10. Rulings writing 0.9.15_Hardened_5 (lead, 2026-10-29 evening)

Verdicts 125 to 131 (JOB.md). Each ruling was run through `docs/OWNER-DIRECTION.md`'s checklist, its
items 11 to 14 included.

| Rule | Ruling | Why |
|---|---|---|
| GRM-35, FFI-40a (verdict 131) | A class section holds entries in the form of every other entry (names and contract words; `@ffi(const)` above a method that only reads its object); the Ember method line with types (`fn lookups() -> c_int`) goes. | One way to write an overlay line (checklist 11); the types are the header's, which the entry is checked against. |
| MAN-9 (verdict 127) | The default `ember.toml` is `[package]` with the folder's name and `version = "0.1.0"`; Ember says that it wrote it. | Nothing is written behind the user's back; the least a package needs. |
| GRM `lambda` (verdict 126) | `@safety("…")` is the only attribute written before a lambda. | No other attribute was asked for. |
| CLI-5 (verdict 130) | `ember bind --init` writes `unsafe fn` lines with no `@safety` text; the first call's `L3015` says what is missing. | A placeholder text would silence the warning without anyone saying anything. |
| EFF-2 (ODR 15.3's knock-on fix) | Dropping a function value has the effects of dropping the functions that reach its place. | The owner was told with ODR 15.3; without it no `@noalloc` function may own a stored function. |

### Owner verdicts on §9's rulings, 2026-10-29 (afternoon; asked as ODR 9 onward)

The rulings of §9 and of verdict 125, put to the owner one at a time, numbered on from the morning's
ODRs 1 to 8.

19. **C++ follows verdict 125** (owner: "yes C++ too, same rule"): a function `import cpp` writes,
    members included, says what its callers must make true in its overlay entry. How a member's entry
    is written is still to settle (two classes' `new` are one entry by name today).
20. **ODR 9 approved: a `safe:` block takes no `# SAFETY:` note** (owner: "no comments for safe, safe is
    safe so nothing to worry about").
21. **ODR 10, first half, approved: an `unsafe:` block inside a `safe:` block is unsafe code again**
    (owner: "I accept your decision").
22. **ODR 10, second half, approved: a function written inside a `safe:` block is a safe function**
    (owner, after the plain re-explanation: "yes I see your point and I agree").
23. **ODR 11 approved: `L3020` and `L3010` warn of a `safe:` or `unsafe:` block that changes nothing**
    (owner: "warning is fine").
24. **ODR 12 approved: `E3100`'s text in a `safe:` block** (owner: "yes that message is fine"). The note
    under it, "`[UNS-1]` lists the operations that do", is cut short: to read "… that need `unsafe`".
25. **ODR 13 replaced: a lambda carries `@safety`** (owner, after the probe showed today's compiler refuses it:
    "yes build it"). `set = @safety("…") fn(v: int) => p.write(v)`: `@safety` written just before a
    lambda; a lambda written in unsafe code without it is `L3015` as a named function is; the text is kept
    with the lambda (its line, `ember tcb`). A value's type `unsafe fn(…)` carries no text, as for a named
    function. To do: a new hardening, the parser, typeck.
26. **ODR 14.1 approved: `ember.overlays` fingerprints each declaration an overlay describes, not the
    header file** (owner: "I agree with your decision", after the probe: a function added to the header
    changes no record and stays unsafe, with verdict 125's `L3015`, until an entry states it).
27. **ODR 14.2 approved: the package's own build adds a missing record line; a changed declaration
    stops the build until a person checks the overlay and re-records** (owner: "yes I agree"). Defect found
    in the probe and to fix: the help's `ember bind --merge` is refused alone, and `--init --merge` records
    only the starter overlay (`<header>.overlay.em`); `ember bind lib.h --check overlays/lib.em --merge`
    is to check that overlay and, when it passes, rewrite its record line, and `E5010`'s help is to print
    that command with the real file names.
28. **ODR 14.3 approved: another package's overlay with no line in that package's `ember.overlays` is
    `E5010`** (owner, after `ember.overlays` was explained again from his own request: "error is fine").
29. **ODR 14.4 approved: `ember.overlays` is TOML, as `ember.toml` and `ember.lock` are** (owner: "keep
    this format"). Its second comment line is to name the re-record command once ODR 14.2's fix lands.
30. **ODR 14.5 replaced: every project has an `ember.toml`** (owner: "ember.tomp is a must for all
    projects, so if it doesn't exist ember should build a default one for you and then create the overlays
    file"). A build of a program with an overlay and no `ember.toml` writes a default `ember.toml`, then
    `ember.overlays` beside it; no record in the cache. Every build: asked whether a loose file with no
    overlay gets one too, the owner: "always, every project needs an ember.toml". So a build or run of a
    file with no `ember.toml` in its folder or above writes a default one (`[package]`, `name` the file's
    folder, `version = "0.1.0"`). To do: a new hardening (MAN, BLD), the driver, the test harness.
31. **ODR 15.1 replaced: a callable parameter holds what its callers pass** (owner: "yes follow what
    callers pass"). §9's table said so, but `targets.rs` maps a parameter to its type: `apply(s.by, x)`
    with `fn apply(mut f: fn(int) -> int, x: int)` reached `Logger.format`'s allocating lambda. To build:
    each argument flows into the callee's parameter (direct calls, and calls through values to each
    target), to the fixed point. Also: `E4001`'s chain named `em_closure_4main_c1`; it is to name the
    lambda by its line.
32. **ODR 15.2: the exact way for lists** (owner: "the exact way"), after the probe found D-930 (a
    `@noalloc` function called `widen` through `Bus.handlers`, accepted and run). Putting a function into a
    list (a list literal, `push`, `insert` and std's other list functions, marked for the analysis) puts it
    in the list's place; anything else writing a list's memory (a raw pointer) falls back to the type.
33. **ODR 15.3 approved: a local a function is copied or lent into holds what its place holds** (owner:
    "yes treat it like s.by"). Found with it, told the owner, to fix (lead, a knock-on fix): dropping a
    stored function counts as a drop not known (`owned s: Scaler` then `f = s.by` refuses `@noalloc` even
    for `double`); the functions reaching the place give the drop: none for a named function, the
    captured state's for a lambda.
34. **ODR 15.4 approved: a function read through a raw pointer may be any function of its type** (owner,
    after the plain re-explanation: "yes any function of that type").
35. **ODR 16 approved: a link's time limit is 30 minutes, `[link] timeout` (seconds) sets another,
    `E9042`, the command file kept, nothing retried** (owner, after asking whether the hang was the
    compiler's: the limit is only the safety net; finding the cause stays to do: "yes 30 minutes is fine").
36. **ODR 17 approved: an `f32` through `...` names the one fix, `as f64`, not both sizes** (owner, after
    the background was explained again: "keep the exception for floats"; verdict 123 names both sizes
    for integers, which C may read at either).
37. **ODR 18 replaced: an overlay line for a C function unsafe to call is `@safety("…")` above
    `unsafe fn name(…)`, as in Ember code** (owner: "why is it not unsafe fn lib_step(x)", then "yes unsafe
    fn, replace -> unsafe"). `unsafe fn` without `@safety` is verdict 125's `L3015`; `@safety` on a plain
    `fn` line is `E0104` as in Ember; the `-> unsafe` contract word is replaced by `unsafe fn` (the
    grammar, `ember bind --init`'s starter, the tests). To check: the `unsafe` word in a parameter's
    place. Found in the probe, to fix: today `@safety` on an overlay line is `E0104` ("does not apply to
    a overlay item"), and `E3100`'s note blames "its `unsafe` word".
38. **ODR 19 approved: verdict 125's `L3015` comes once per C function, at its first call in unsafe code**
    (owner: "once per function is fine"). Its help is to name `unsafe fn name(…)` (ODR 18).
39. **ODR 20 approved: an overlay line for a C++ class member goes under its class** (owner: "yes member
    lines under their class"). Found in the probe: today a top-level `fn new(x, y)` in an overlay of
    `handles.hpp` made both `Handle.new` and `MutHandle.new` safe to call (entries are found by name).
    A top-level line names only a function outside any class; one naming only members is an error that
    names the classes to write it under.

**The work these verdicts leave (2026-10-29, in order of the push):** verdict 125 for C and C++
(items 19, 37, 38, 39: `@safety` above `unsafe fn` in an overlay, under the class for a member; `-> unsafe`
replaced; the help and note texts); a lambda's `@safety` (25); every project has an `ember.toml` (30);
`ember bind <header> --check <overlay> --merge` re-records, with `E5010`'s help naming it (27, 29); EFF-2:
a parameter holds what callers pass (31), lists the exact way (32, D-930), a stored function's drop, the
chain's lambda by its line (31, 33); `E3100`'s cut-short note (24); then a new hardening for all of it,
the link time limit (35) and the hang's cause, the full check, the bench, the push.

