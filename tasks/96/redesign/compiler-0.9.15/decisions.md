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
| GRM-35, FFI-40a (verdict 131) | A class section holds entries in the form of every other entry (names and contract words; `@ffi(const)` above a method that only reads its object); the Ember method line with types (`fn lookups() -> c_int`) goes, and with it the line asserting `mut self`. An entry without `@ffi(const)` leaves the importer's reading of the body (corrected in place the same evening: Hardened_5 first said such an entry imports the method as changing its object). | One way to write an overlay line (checklist 11); the types are the header's, which the entry is checked against. 0.9.15 has no word for "changes its object" (`[FN-4]`: worked out from the body), and writing an entry to add `@safety` must not change what a method may do. |
| MAN-9 (verdict 127) | The default `ember.toml` is `[package]` with the folder's name and `version = "0.1.0"`; Ember says that it wrote it. | Nothing is written behind the user's back; the least a package needs. |
| GRM `lambda` (verdict 126) | `@safety("…")` is the only attribute written before a lambda. | No other attribute was asked for. |
| CLI-5 (verdict 130) | `ember bind --init` writes `unsafe fn` lines with no `@safety` text; the first call's `L3015` says what is missing. | A placeholder text would silence the warning without anyone saying anything. |
| EFF-2 (ODR 15.3's knock-on fix) | Dropping a function value has the effects of dropping the functions that reach its place. | The owner was told with ODR 15.3; without it no `@noalloc` function may own a stored function. |

## 11. Rulings made on autopilot, night of 2026-10-29 (lead, the owner asleep)

The owner: "now you work autonomously ... in the morning I want you to report all ODRs to me, you can use one
helper agent and make sure you ruling pattern is followed". Each ruling below was run through
`docs/OWNER-DIRECTION.md`'s checklist, items 11 to 14 included, and is put to the owner in the morning.

| # | Rule | Ruling | Why |
|---|---|---|---|
| N1 | UNS-7 (verdict 126) | A lambda written in unsafe code without `@safety` is `L3015` at the lambda; `@safety` on a lambda outside unsafe code is `E0104` ("applies only to an `unsafe fn`", as on a named function); `@safety` written before an expression that is no lambda is `E0104`. | The same three answers Ember gives a named function (checklist 11). |
| N2 | MAN-9 (verdict 127) | Every command that compiles a file (build, check, run, test, lint) writes the default `ember.toml` first, in the file's folder, and says so on standard error; `ember bind` of a header no package holds writes one too. Finding a manifest starts from the file's absolute folder, so a bare `main.em` sees one above the working folder (it did not). | One path for every program (checklist 13); a manifest above the folder must never be missed and then duplicated. |
| N3 | MAN-9, the test corpus | Each test folder is its own package: the 574 default manifests the suite's first run wrote are committed, with the records (`ember.overlays`) their overlays need, as `FFI-17`'s and `FFI-17a`'s already were. One `tests/ember.toml` was rejected: it would make the whole corpus one package, whose `[DIA-12]` import suggestions read every file of it. | The model the owner chose, unchanged for tests; a test's package is the files beside it, as before. |
| N4 | CLI-7, FFI-55 (ODR 14.2) | `E5010` for a changed declaration names `ember bind <header> --check <overlay> --merge` with the import's own spellings when the overlay is the package's own; for another package's overlay it says that package checks and records it again. | The help's line must work where it is given (principle 12). |
| N5 | EFF-2 (verdicts 128, 129, ODR 15.3) | A parameter holds what each direct call passes it; a function whose address is taken (a value, a table's slot, a virtual method) has parameters that may hold any function of their types. The list operations (`Array(…)` of a literal, `push`, `insert`, `extend`, `clone`, `sorted`, `pop`, `remove`, `swap_remove`, `drain`) move what they hold between the list, the values given and the result, `Some`'s payload included; `[f; 3]` holds `f`. Any other value the analysis cannot follow holds any function of each callable type within its type (a list's, an `Option`'s, a tuple's). Dropping a function value has the effects of dropping the functions stored into its type: none for a named function, the captured values' for a lambda. | Sound everywhere the program runs a function it did not see stored (checklist 14); exact where it can be followed. The whole program is one compilation, so every direct call is seen. |
| N6 | EFF-2 messages | In an effect chain a lambda is "a lambda", and the message points at where it is written ("the lambda the call runs"), instead of its symbol (`em_closure_4main_c1`). | A message names what the program wrote (principle 12). |
| N7 | Hardened_5, in place | Two sentences I wrote that evening were corrected before any push: an entry without `@ffi(const)` leaves the importer's reading (not "changing"), and `unsafe` stops being a contract word in overlays only (`[FFI-11]`'s vocabulary is shared with `@ffi(…)` on hand declarations, which verdict 130 did not mention). | Never widen an owner answer; my own text of an hour before, nothing pushed. |
| N8 | The watcher | "if any of those change you wait for 5 minutes and hibernate": a change that is still there after 5 minutes hibernates; one that comes back within them does not. | A power blip should not stop the night's work; a real cut or a lost network does. |

**The read-only review of the night's compiler changes** (the one helper agent; findings fixed by the lead):
a `class` section's line that begins no entry looped the parser for ever (now skipped past, as the overlay's
own loop does); `L3015` and a lambda's `E0104` were lost when a quiet check rolled back, since the once-only
sets outlived the rollback (now recorded while checking and reported once at the end); `ember bind
--check calc.overlay.em --merge` with a bare name recorded into Ember's cache (now from the absolute folder);
an overlay in no package (`../vendor/x.overlay.em`) got a manifest of its own and became another package's
(N10 below); `@safety` text with `"` or `\` broke the declarations the importer writes (now escaped);
`ember fmt` dropped a lambda's `@safety` (now kept as written); `ember bind --init --merge` of a C++ header
wrote members at the top and again on every merge (now under `class X:`, by `Class.member`; and a C++
header's starter is an `overlay cpp`, which it was not); an `override` entry kept an earlier `@ffi(const)`;
`bind --check` counted an `unsafe fn` entry as safe; a manifest being written could be read empty by a
build beside it (now written aside and linked in). Left, recorded: a renamed function's `L3015` names its
Ember name, not the overlay's; `@grade(instrumented)` on a member's entry is never backed.

| # | Rule | Ruling | Why |
|---|---|---|---|
| N9 | GRM-35 | A line under a `class` section that begins no entry is `E0100`, and parsing goes on at the next line. | As at the top of an overlay. |
| N10 | FFI-55, MAN-9 | An overlay that no package's folder holds is the importing program's own: its record is in that program's `ember.overlays`, named by a path from the package (`../vendor/x.overlay.em`). Only `ember bind`, with no program, makes the header's folder a package. | An overlay nobody packaged is the program's to vouch for; a manifest written beside it would have made it another package's and refused it. |

| # | Rule | Ruling | Why |
|---|---|---|---|
| N11 | BLD-15 (verdict 124, ODR 16) | Each Ninja link runs under a guard, `ember __link <seconds> <program> <command file> -- <linker> …` (an internal command of the compiler); a link outside Ninja is guarded in the compiler itself. At the limit the guard takes the stacks of every thread of the linker's processes with the Windows debugger `cdb` when it is installed (attached without stopping them, no symbols fetched, a minute at most each), into `<command file>.stacks.txt`, then ends the whole process tree through WMI's `Terminate` with exit code 1, then `taskkill`; the half-written program is deleted and the message names the command file and the stacks. Every helper of the guard runs under a limit of its own. | The stop must hold where `TerminateProcess` did not (the recorded hangs), never read as a success (exit code 0 did, D-928), and leave the evidence the cause needs: verdict 124's first half. |
| N12 | TST-4 | `E9042` has no conformance test: no test can make a linker hang. The build crate's unit test ends a process tree past its limit, and the guard was run by hand on a 30-second command (E9042 after 2 s, the output removed, the command file and stacks kept); E9042 is in `tools/rule_index_baseline.json`'s codes without tests. | A test that waits for a real hang would hang the suite. |
| N13 | MAN-9, FFI-55 (in place in Hardened_5) | The spec text says what N2 and N10 decided: every command that compiles writes the default manifest; an overlay in no package is the importing program's. | The compiler follows the spec; neither was pushed. |

| # | Rule | Ruling | Why |
|---|---|---|---|
| N14 | EFF-24 (A4-2, first slice) | A summary's fact is a place a parameter reaches (`this` the first of a method): the parameter and its steps (fields, an enum variant, a list's elements as one step, and each step through a reference, pointer or object handle, at least one), covering everything inside it; changes, reads and ends, each with the first call chain that makes it. A value the analysis cannot follow (a pointer read through a raw pointer, a C function's result, a function with no body here) may be anything any parameter reaches. Paths are cut at 8 steps, a shorter path covering longer ones, so recursion through links ends. The summary is computed by `ember_analysis::summaries` and printed by `--emit mir`; it is put on `Body` (and in interface files, A4-3) with its first consumer, M1's `this`. | Sound before precise: a fact too many refuses a correct program later, a fact missing would accept a wrong one (checklist 7, 14); nothing is built before something uses it. |

| # | Rule | Ruling | Why |
|---|---|---|---|
| N15 | UNS-7, BLD-1 (verdict 125, ODR 19; D-931) | `L3015` for a C or C++ function comes once in a program, at its first call by place: in the module loaded first, then the earliest in it. An incremental check reports it from a kept part as a clean check does: the after-body warnings' range holds it (it was before the range, so a kept part's was lost), and a kept part's takes the place of one this check made at a later call, or is left out beside one at an earlier. | "Once per function, at its first call" must not depend on which modules a build checks again; "first" by place is the one order both kinds of check share. |

**For the owner (open, not ruled):** verdict 130 retired `-> unsafe` in overlays. A hand declaration in an
`unsafe extern` block can still write `@ffi(unsafe)` (and `param(p, unsafe)`, `result(unsafe)`): its call is
unsafe on purpose, and `E3100` then names that instead of asking for facts. A plain `fn` there is unsafe to call
already. Should `@ffi(unsafe)` go too, as a second spelling, or stay?

### Owner verdicts on §11's rulings, 2026-10-30 (morning)

The owner: "ODRs are language related decisions" — N3, N8, N11 to N14, N16, N17, N19 to N21 are not ODRs
(tooling, tests, the watcher, planning, internals) and were not asked; N8's reading was confirmed ("yes no
hibernate if the conditions return back to normal"). N18 (what changes `this`) and N22's codes are the
spec's own rules, not rulings; N22's wording was approved.

- **Approved as ruled:** N1 (a lambda's `@safety`), N2 (the default manifest, with the note reworded:
  JOB.md verdict 135), N4 (`E5010`'s help), N5 (functions followed into and out of lists; an unknown one
  is any of its type), N6 ("a lambda" in a chain; the `^` landing on the last argument is to be fixed),
  N9 (a stray line in an overlay's `class` section), N10 (an overlay in no package is the program's), N15
  with an addition (JOB.md verdict 134: every other call named), N22's wording.
- **Reversed:** N7's first half — an overlay entry without `@ffi(const)` makes the method changing
  (JOB.md verdict 132).
- **The open question, answered:** `@ffi(unsafe)`, `param(p, unsafe)` and `result(unsafe)` go; a
  hand-written declaration unsafe to call is `@safety` above `unsafe fn` (JOB.md verdict 133).

### Writing 0.9.15_Hardened_6, 2026-10-30 (evening; the owner: "go ahead")

Verdicts 132 to 135 as written in JOB.md, and the LLVM backend the default, which `[CG-LL-1]` already said
would happen once it passed the conformance and performance suites. The lead's choices in writing them:

| Rule | Choice | Why |
|---|---|---|
| FFI-10 (verdict 133) | A plain `fn` in an `unsafe extern` block stays as it was: unsafe to call until its facts are complete and it is `safe fn`, its `E3100` as before. Only the word `unsafe` inside `@ffi(…)` goes, and `unsafe fn` takes its meaning (unsafe on purpose, the `E3100` any `unsafe fn` call gets). | The verdict named `@ffi(unsafe)`, `param(p, unsafe)` and `result(unsafe)`; a plain `fn` is the hand-written twin of a header function with no overlay entry, which also stays unsafe without a word. |
| CLI-20 (verdicts 130, 133) | `ember fmt --migrate` rewrites both retired spellings: `@ffi(unsafe)` and its per-slot forms, and an overlay entry's `unsafe` contract words, to `unsafe fn`; it writes no `@safety` text. | `[VER-9]`: a hardening that changes the language comes with `--migrate` rewriting every program it can; no placeholder text (§10, `CLI-5`). |
| E0104 | The retired word is `E0104` in both places, with the `unsafe fn` line to write. | Verdict 130's overlay refusal already is; one code for one mistake. |
| UNS-7 (verdict 134) | "first call by place": the file built first, then its imports, the earliest line in each; each other call a note `also called at file:line:col`. | The verdict's words; N15's order. |

Not rulings (recorded so the counts are not a surprise): `docs/phase-status.json` takes `UNS-7` off
`built_for_0915` until verdict 134's notes are built, and puts `CG-LL-1` on it with a test of the default
(`cg_ll_1_the_llvm_backend_is_the_default`); `appx_h.py` reads a rule past a blank line inside it, which gave
the older change logs 25 rows they had missed.

#### Building verdicts 132 to 135 and the effect caret (lead, night of 2026-10-30; none changes what the verdicts say)

| Rule | What was built | Why |
|---|---|---|
| FFI-40a (132) | Every method entry sets its method's receiver: with `@ffi(const)` read-only, without it changing (`cimport.rs`, `applied.receivers`). It reaches the free operator functions C++ declares outside the class and Ember imports as the class's methods (`operator+(const Money&, const Money&)` as `Money.add`): their entries need `@ffi(const)` like any method's. 30 entries in 14 test overlays, for methods that only read, now say `@ffi(const)` (none of their classes has `mutable` state). | The verdict: an entry means what it says, whatever the importer reads (checklist 15). |
| FFI-40a (132) | `E3021`/`E3023` at a call of a C++ method imported as changing, through a name that only reads, gets a help naming the line: "if it only reads its object, write `@ffi(const)` above its entry". | Checklist 9: the message gives the line to write. |
| CLI-5 (132) | `ember bind --init` writes `@ffi(const)` (with a `# derived:` comment) above each method the importer reads as only reading its object. | The starter compiles unchanged and keeps what the import did; without it adopting the starter would make every read-only method changing. |
| FFI-10 (133) | `safe unsafe fn` is parsed whole and refused, `E5002` ("declared both `safe` and `unsafe`"), as `safe` with `@ffi(unsafe)` was; it was a parse error with no place. | The old refusal carried to the new spelling. |
| CLI-20 (130, 133) | `ember fmt --migrate` rewrites both: the word out of `@ffi(…)` (an empty one removed) and `unsafe fn`; in an overlay the word out, a slot left bare losing its `:` or `->`, and `unsafe fn`. Neither writes `@safety` text. | `[VER-9]`; §10's no placeholder text. |
| UNS-7 (134) | The type check gives one `L3015` for each call of a C or C++ function nothing states (each call once, whichever checks see it), and where diagnostics are shown (`Sink::ordered`, text and `--json`) one function's are one: the first by place (the file read first, then the offset), with `also called at <file>:<line>:<col>` for each other, in that order, before the warning's own note. An incremental build keeps each call's warning with its call's part, so a kept part and a part checked again add up to what a clean build reports; D-931's special case in `report_again` goes. | The verdict's words; the kept parts record warnings by the part their place is in, so one merged warning would carry stale calls of parts checked again. |
| EFF-2 (N6) | An effect's chain is reported at the whole call: MIR keeps a call's own expression (`BasicBlock::call_span`, set for direct, interface and indirect calls) beside its place, which stays its last argument. Moving the place itself was tried and undone: it moved a move's "moved here" from the moved argument to the call, `explain`'s "used at" likewise, and a backtrace frame's column. | The owner's "yes" to N6 named the caret as the flaw to fix, nothing else. |
| MAN-9 (135) | The note's words as approved; the error beside it reworded the same way ("cannot create `…`: …; every program needs an `ember.toml` in its folder or a folder above"). | Verdict 135's lesson (checklist 18). |

### Rulings made on autopilot, night of 2026-10-30 (lead, the owner asleep; for the morning)

The owner: "back to autonomous mode ... I'll ask for ODRs in the morning", "and you go solo tonight". Only
language decisions are listed here; each is put to the owner one at a time, with the code.

| # | Rule | Ruling | Why |
|---|---|---|---|
| N23 | FFI-10 (verdict 133) | A plain `fn` in an `unsafe extern` block stays as it was: unsafe to call until it is written `safe fn` with its facts complete, its call refused with the facts it lacks; only `unsafe` inside `@ffi(…)` became `unsafe fn`. | The verdict named the `@ffi(…)` spellings; a plain `fn` is a declaration nothing is said of yet, as a header's function with no overlay entry is. Checklist 16 asks whether `fn` and `unsafe fn` there are two spellings of one idea: they differ in what the call's error asks for (the facts, or nothing), so I left it and ask. |
| N24 | FFI-10 | `safe unsafe fn` on one declaration is `E5002` ("declared both `safe` and `unsafe`"), with both lines to write. | It was a parse error with no place; `safe` with `@ffi(unsafe)` was `E5002` before verdict 133. |
| N25 | FFI-40a (verdict 132) | A free C++ operator function Ember imports as a class's method (`operator+(const Money&, const Money&)` as `Money.add`) is a method like any other: its entry without `@ffi(const)` makes it changing, though C++ passes the object by `const&`. | Verdict 132's words ("its method"); an entry means what it says. |

**Owner verdicts, 2026-10-31 (morning).** N23 reversed: a plain `fn` in an `unsafe extern` block is refused
(JOB.md verdict 136; the lead recommended it when the question was put a third time, plainly: a plain `fn`
is safe to call everywhere else). N24 approved ("perfect, I like this decision"). N25 approved ("damn we
are slowly aligning more and more, approved"). What worked: the owner's own example (`lib_step`, verdict
133), the question in one sentence, the two answers one line each; two longer versions did not ("UH
WHAT?", "I dont get it").

**How verdict 136 is built (Hardened_7, `[FFI-10]`).** The type checker refuses a plain `fn` in a
hand-written `unsafe extern` block with `E5002` ("`lib_step` does not say whether it is safe to call"),
its help naming both lines to write; `ember fmt --migrate` writes `unsafe fn`. Not widened: a function
`import c` reads from a header is not a hand-written declaration, so the importer still shows one with no
overlay entry as a plain `fn` and its call keeps `[DIA-18]`'s error naming each missing fact; the thunks
the importer writes itself (`@safety` lines it supplies) say `unsafe fn`, as any hand-written one must.
A declaration with the retired `@ffi(unsafe)` keeps its own `E0104` and migration: it said `unsafe`, in
the old spelling, and the two migrations inserting `unsafe ` at one `fn` wrote `unsafe unsafe fn`.

**The defects the push gate found, 2026-10-31 (D-936 to D-939).** The CI-style run (Clang, the LLVM
backend, Microsoft's linker forced, `EMBER_CACHE=target/ci-cache`, relative, so each folder its own cache):
- D-936: 8 reload tests failed, two threads of the test binary building the shared runtime into one
  temporary folder named by the process id. `ember_build::temp_suffix()` (process id and a count) names
  all 11 such temporaries; the publish already copes with losing the race.
- D-937: `a_build_is_reproducible_and_named_by_its_inputs` failed. Not the relative cache itself: the
  runtime's object, linked from the cache by its absolute path and carrying its temporary name, reached
  the program database, which `lld-link` hashes into the program (Microsoft's linker does not). Chosen:
  link it from the objects' folder by its name, as Ninja already spells the build's own (`from_dir`),
  and fix its object name at compile; not a path map in the linker (`lld-link` has none for module
  names) nor compiling the runtime with each program (145 ms a build, the reason the cache exists).
- D-938: the C warning sweeps had swept nothing since the LLVM default: `--emit c` is the C backend's.
- D-939: the reload benchmark failed, the runtime beside the running program replaced from another cache (the host linked from the test's folder, a relative cache): compared by its inputs' key now, not its bytes.

### Rulings made on autopilot, night of 2026-10-31 (lead, the owner asleep; for the morning)

The owner, evening: build `[LT-22]` (A2-1's other half) in M1 so every `@borrows` can be refused ("building
the dependencies first ... so build that too"); at night: "run autonomously ... ask for ODR updates" in the
morning. Only language decisions are listed; each is put to the owner one at a time, with the code.

| # | Rule | Ruling | Why |
|---|---|---|---|
| N26 | LT-22, LT-1, LT-34 | A result of a view type (`[TYP-34]`: a struct, enum or tuple holding views, `Option[StrView]` among them) points where the body's summary says, within the first line; a plain view result (`StrView`, `Span[T]`, `ref T`) is the first line's rule at the caller. So `Words.next() -> Option[StrView]` (a non-`Copy` struct holding `rest: StrView`) gives items that point into the text `rest` views, and a second `w.next()` while the first item lives compiles (`LT-22/accept_items_of_a_non_copy_iterator_point_into_what_it_reads`); the same method returning a plain `StrView` keeps `w` viewed (`LT-1/reject_a_view_a_non_copy_object_returns_keeps_it_viewed`, E3025). | `[LT-22]` says it of "a function returning a view type"; `[LT-1]` rule 3 "may view more than the function needs", so a plain view does not read the body. Without the summary for view types, std's adapters stop compiling once `@borrows(self.a, self.b)` is gone: `Filter.next ... from inner` over `Nested` or `Chain` (whose `next` names no field) is `E3062` (GRM-38, CTL-1). `gap/02` had planned E3021 for the `Option` iterator; the spec's words decide it the other way. |
| N27 | LT-1, LT-22 | A method whose body returns a view of a source parameter other than `this` compiles; its callers then view `this` and every source parameter (rule 3). Only what `from` does not allow is `E3062`. | `[LT-22]`: "where the summary would need more than the first line allows, the result points into every source parameter (`[LT-1]` rule 3)". `[LT-1]`'s "a returned view that points into any other parameter is refused" read as any parameter that is not a source (a `Copy` value, an `owns` one). `gap/02` had planned `accept_an_owned_receiver_is_under_rule_3` as E3062 without `from a`; `[LT-22]`'s fallback is the later, more specific sentence. |
| N28 | LT-1, LT-1a | A result that points into one of two `ref mut` parameters of a `Copy` type (`fn choose_mut(ref mut first: Shared[Counter], ref mut second: Shared[Counter], ...) -> ref mut Counter`, 0.9.10's `@borrows(first, second)`) has no 0.9.15 first line: `from` names one, and such parameters are not source parameters, so rule 3 leaves them out: `E3062`. HEAP-5's test now chooses in the caller (`open(first) if choose_first else open(second)`, each `-> ref mut Counter from owner`). | `[LT-1]` says it in so many words ("`from` is needed ... to let the result point into a `ref mut` parameter of a `Copy` type"; "a `Copy` parameter that is not a view is not a source parameter"). Asked because 0.9.10 could write it and 0.9.15 cannot: the other way is to make a `ref mut` parameter a source of rule 3 whatever its type. |
| N29 | LT-1a, LT-23 | Minor change, for readability: where the result may point into what `from` names and into another parameter (`-> Pair from a` over `return make(a, b)` through a function value), E3062's help is "remove `from a`: the result may point into `a` and `b`, and with no `from` it points into each of them"; one wrong parameter only keeps "return a view of `a`, or write `from b` after the result type". Spec first (next hardening), then built with a UI case. | The spec's E3062 example names two ways out, and here both fail (`from b` is as false as `from a`). Built by the lead without an ODR and reverted; the owner: a quality-of-life change that "needs to go through a minor change ODR". |
| N30 | CLI-20, OWN-9 | **Answered 2026-11-01 evening (below the table).** Was: 0.9.10's `owned x: int` copied a number; 0.9.15's `owns` hands it over (`[OWN-9]`). `[CLI-20]` migrates the mode `owned` to `owns`, so a 0.9.10 caller that gives a number to such a parameter and uses it again (`m.insert(key, v)` then `m[key]`) migrates into `E3040`. Should `ember fmt --migrate` write `copy of` at such a call (`m.insert(copy of key, v)`), keeping the 0.9.10 meaning, or list the line as one it could not rewrite? `[CLI-20]` says neither. The repository's own code was given `copy of` by hand where it reuses the number (`std/src/ecs.em` twice; three tests). | Found building A1-2 (M3), 2026-11-01. |
| N31 | GRM-15, OWN-9 | **Answered 2026-11-01 evening (below the table).** Was: `b = owns n` hands a number over, and `owns` before a name whose `=` already hands it over (`b = a` for a value that is not a number) is `E0109`. A `bool`, a `char`, a `StrView`, a `ref T`, a `Span[T]` or a struct made only of these is no number, yet `=` copies it as it does a number (VII.1's `Copy` types), so `b = owns flag` fits neither sentence: hand `flag` over as a number is, or `E0109`, whose message ("`b = flag` already hands `flag` over") would then not be true? Until answered the compiler keeps `E0900` for it. | Found building A1-2 (M3), 2026-11-01. |
| N32 | GRM-15, OWN-9 | **Answered 2026-11-01 evening (below the table).** Was: `E0109` covers `owns` before "an argument" and before `return a`, and `[OWN-9]` gives its words only for `e.mesh = m`, `b = a` and `owns fn`. (1) For an argument of an `owns` parameter and for `return a` the same words fit ("`f(a)` already hands `a` over; remove `owns`", "`return a` already hands `a` over; remove `owns`"): are these the messages? (2) An argument of a parameter that only views it (`fn show(x: Mesh)`, `show(owns m)`) is not handed over by `show(m)`: `E0109` there too, and with what words? Until answered the compiler keeps `E0900` for both. | Found building A1-2 (M3), 2026-11-01. |
| N33 | FN-4 | **Answered 2026-11-01 evening (below the table).** Was: `this` in a `static fn` or a function standing alone is `E1010`, and `[FN-4]` gives its words for its one example only ("`starting_at` is a `static fn`: it is called on the type, `Counter.starting_at(5)`, and has no object, so no `this`; build the counter from `k`: `return Counter(n=k)`"). What is the message in general: for any `static fn` (its call with what arguments, and is the help "build the ... from ..." given only where the compiler can see it), and for a function standing alone? Until answered the compiler keeps "`self` outside a method" (which names a word the program did not write). | Found fixing D-951, 2026-11-01. |
| N34 | LT-1a | **Answered 2026-11-01 evening (below the table).** Was: `E2118`'s help gives two first lines with the parameter renamed ("rename the parameter: for a view of the field write `fn next_word(line: StrView) -> StrView from src:`, for a view of the parameter `fn next_word(line: StrView) -> StrView from line:`"); the example's new name `line` is the example's. What name does the compiler write in general (the parameter's name with a word added, such as `src_param`, or a placeholder the user replaces)? Until answered the message is the spec's ("`src` is both a parameter of `next_word` and a field of `Parser`, so `from src` could mean either") and the help today's ("rename the parameter: then `from src` names the field, and `from` with the new name names the parameter"). | Found fixing the verification run's LT-1a findings, 2026-11-01. |
| N35 | LT-1a | **Answered 2026-11-01 evening (below the table).** Was: `E3062`'s help is "return a view of `src`, or write `-> StrView from buf`": the `from` that names what the result points into. Where that is the whole object (`return ref this` under `from src`), no `from` names it (`from` takes a parameter or a field, `[GRM-41]`). Is the help then "return a view of `src`" alone, or "return a view of `src`, or remove `from src`" (rule 1 then points it into `this`)? Built for now: the first, which says only what the spec's form says. | Found fixing the verification run's LT-1a findings, 2026-11-01. |
| N38 | LT-48, LT-1 | **Answered 2026-11-02: verdict 162, (a), `runtime` before a class type too ("yes a").** Was: Verdict 152 put `runtime` before a parameter's `dyn` type. A `virtual` call on a class value has the same question and no `dyn` to write it before: with `open class A` (`virtual fn first(other: Array[int]) -> Span[int]: return this.items`), `class B(A)` (`override ... return other`) and `fn show(a: A):` doing `r = a.first(xs)`, `xs.push(owns 2)`, `println(r)`, `show(B())` is refused when compiling (verdict 151), but where `a`'s class is chosen while running, built today: `E3021` "`a` may be a `B`, and which kind it is is only known while running" with no way to say "check it while running". Does `runtime` also go before a class type (`fn show(a: runtime A):`), or is there no run-time form for classes? | Found by the verdict-151 helper, 2026-11-02 (`dyn-per-kind-design.md` question 6). |
| N39 | LT-48, LT-1 | **Answered 2026-11-02: verdict 163, (b), `runtime` on a variable too ("yes b").** Was: Where the kind is chosen while running inside one function, with no parameter in between: `q: Box[dyn Picker] = Box(Mine(items=[9])) if input() == "m" else Box(Yours(items=[9]))`, then `r = q.pick(xs)`, `xs.push(owns 3)`, `println(r)`, all in `main`. Built today: `E3021` "`q` may be a `Yours`, ..." with no help, since `runtime` goes on a parameter. Is that right (move the work into a function with a `runtime` parameter), or does `runtime` go somewhere on such a line too? | Found by the verdict-151 helper, 2026-11-02 (`dyn-per-kind-design.md` question 7). |
| N40 | GRM-15, FN-2a | **Answered 2026-11-02: verdict 161, (a), the brackets stay ("yes a").** Was: The grammar makes `owns` bind as `-` does (`unary := value_word unary`), so with verdict 147 a computed argument takes brackets: `m.insert(owns (i * 7), owns i)`, `tx.send(owns (i * i))`; `m.insert(owns i * 7, owns i)` is `(owns i) * 7`, a value made after `i` is handed over, and is `E3028` with the help `owns (i * 7)`. Built that way (the benchmark programs and tests were respelled so). Should `owns` written at a call cover the whole argument instead (`m.insert(owns i * 7, owns i)`), as a call word, while `owns` elsewhere keeps the grammar's binding? | Found respelling the repository for verdict 147, 2026-11-02. |
| N37 | HASH-1 | **Owner, 2026-11-01.** Copying a hasher is no issue: `[HASH-1]`'s "a hashing state that is not `Copy`" loses "that is not `Copy`" in the next hardening. The words were never the owner's: ODR-033 (2026-09-25, delegated) wrote "move-only", respelled "not `Copy`" in 0.9.15_Hardened_1 (`e7a010fd`). The compiler already compiles `@derive(Copy)` on a type that implements `Hasher` (`target/verify-0915/probe/hcopy2` prints `7 12`); `DefaultHasher` and `RandomState` carry no `Copy` derive. | Owner: "COPYING SHOULD NOT BE AN ISSUE MAKE IT AN ODR", after `Sum` (`@derive(Copy)`, `e = h`, `e.write_u64(5)`) printed `7 12`. |
| N36 | STD-19, OWN-9 | **Answered 2026-11-01 evening (below the table).** Was: ODR-240 (still in Hardened_9's table) says the standard library's functions that take a function borrow a named one; `[STD-19]`'s 0.9.15 text says "the function is handed over as every argument for an `owns` parameter is". So after `xs.iter().for_each(h)`, is `h()` allowed (ODR-240) or `E3040` (`[STD-19]`)? Built for now: ODR-240, for the standard library's functions only; a program's own `owns` parameters and `mem.drop` take a function value over (D-963). | Found by the verification run (hand-over finding 7), 2026-11-01. |

The owner's answers (2026-11-01, morning): N26 is verdict 137 (the code of the function called is read for every result type, plain views too: "yes"); N27 is verdict 138 (a method may return a view of a source parameter with nothing written: "I am cool with that so yes"); N28 is verdict 139 (`choose_mut` over two `Shared` handles stays `E3062`, the caller chooses: "keep it", after the run-time locking alternative was shown); N29 is verdict 144 ("yes").

The owner's answers (2026-11-01, evening; every one in JOB.md in his words): N31 is verdict 145 (`owns` hands over every value `=` copies); N32 is verdict 146 (`show(owns xs)` is an error), and verdict 147 replaces the rest of N32: an argument for an `owns` parameter says `owns` at the call (`take(owns xs)`, `take(owns copy of xs)`, `take(owns [1, 2])`), verdict 148 `return xs` with no word and `return owns xs` an error, verdict 149 `push` too; N36 is verdict 150 (ODR-240 stands, `[STD-19]`'s hand-over sentence goes); N37 is verdict 153; N33, N34 and N35 are worded in verdict 157. N30 is tooling, not an ODR (owner rule of 2026-10-30): the lead's ruling, `ember fmt --migrate` writes `owns copy of` where 0.9.10 code uses a number again after giving it to an `owns` parameter, keeping the 0.9.10 meaning. Also new: verdicts 151-152 (`dyn` calls checked per kind; `runtime` on a parameter), 154-156 (`[GRM-40]` x2, `[LEX-14]`). From now on a message's words and help, and every test, are the lead's (JOB.md, "Messages and tests").


### The lead's rulings building verdicts 145-160 (2026-11-02, night; messages and tests are the lead's, owner 2026-11-01 evening)

| # | Rule | Ruling | Why |
|---|---|---|---|
| N41 | FN-2a, OWN-9 | Call words are checked only for the arguments of a call the program writes: a call or argument the compiler makes (a node with no parser id: a `?`'s conversion, a map or set literal's inserts, a comprehension, a `yield`, `x.map(f)` routed to `option_map`), a constructor's argument (verdict 159, a class built through its `init` and an enum's constructor alike), and the files an `import c`/`import cpp` module is written into take none. | Verdict 147 is about what a program writes at its calls ("a call says ..."); the compiler's own glue has no reader. Verdict 159: "a constructor's argument takes no word: it works as `=` does"; a class's `Device(b.widget_new(7))` is a constructor as `Words(names=xs)` is. |
| N42 | FN-2a, GRM-15 | `owns` binds as `-` does (`unary := value_word unary`), so a computed argument takes brackets: `m.insert(owns (i * 7), owns i)`; E3028's help writes them (`owns (i * 7)`), and an argument written over several lines is "`take` takes this argument over", help "say so at the call: write `owns` before it". | The grammar, word for word; OPEN N40 asks the owner whether `owns` at a call should cover the whole argument instead. |
| N43 | FN-2a, STD-15 | `Array`'s built-in `push` takes `owns value: T`: `xs.push(3)` is E3028 "`push` takes `3` over (its first line says `owns value: T`)", help `xs.push(owns 3)`. `String.push(c: char)` and `push_str` read theirs. | Verdict 149 ("`Array`'s built-in `push` included"); the spec's `push(c: char)` for `String`. |
| N44 | FN-2a | E3027's help and note for a `ref mut` argument that is no place: "bind the value to a local first, or pass the owner place directly (`f(ref mut obj.field)` rather than `f(ref mut obj.get_field())`)", note "the callee changes what it is given, so it needs a place it can change". | The old note said the call writes no word, false since A1-8. |
| N45 | DIA-21 | The Python habits' fix-its write the call words: P6 `xs.append(x)` is `xs.push(owns x)` (brackets for a compound value), P12 `raise E("m")` is `return Err(AnyError.msg(owns "m"))`, P21 `Err("m")` is `Err(AnyError.msg(owns "m"))`. A UI case writes everything else the Ember way (`map(owns lambda x: x * 2)` isolates P13). | A fix-it gives a line that compiles (`ui.rs` checks it); a case isolates one error code. |
| N46 | GRM-40 | Above an `extend` block the help is "write the brackets after `extend`: `extend[T] Shape:`", from the program's own line. | The spec's form ("after the name") fits `enum`/`interface`/`type`; an `extend` writes them after `extend` (`[GRM-34]`). |
| N47 | LEX-14 | A keyword where the grammar expects that keyword is not a keyword used as a name: `for in xs:` keeps `E0100 expected a pattern`; keywords that may follow an operand (`and`, `or`, `is`, `in`, `as`, `if`, `else`, `for`) keep `E0100 expected an expression`; a stray `else:`/`struct S:` in a body keeps `E0100`. | "Where a name is expected" ([LEX-14]): there the grammar expects the keyword, not a name. |
| N48 | LT-48 | The run-time stop names the kind as the compiler shows types: another module's is qualified ("`p` is a `support.pickers.Yours`"); "gives back the list it was handed" for an `Array`/`Span`, "the text it was handed" for text, "the `T` it was handed" otherwise, "what it holds" for a view of the receiver. | The spec's one example is a list in one module. |
| N49 | LT-48 | The kind check compares a kind word every `dyn` table ends with (`ember_mir::kind_identity`, FNV-1a of the canonical type name), not the table's address: one compare, the same in every unit of a multi-module build. | Tables are per unit (C `static const`, LLVM `internal`), so an address compare never matched a value boxed in another module on the C backend (found by the review). |
| N50 | LEX-14, DIA-1 | `owns`, `shares`, `shares mut`, `thread_shared`, `weakref`, `sharedref` and `sharedref mut` before a type (`parent: weakref Node`, `Option[owns Mesh]`), and the value words before a value where this compiler does not build them (`child.parent = weakref this`), are `E0900` "`weakref` before a type is not built by this compiler yet", and the type after the word is still read. | Verdict 156 makes a keyword used as a name `E0005`, but these are the spec's own words where the spec puts them: the compiler lacks them, the program is not wrong. |
| N51 | CLI-20, OWN-9 | N30 built: `ember fmt --migrate` writes `owns` at each call for an `owns` parameter (the checker's `E3028` migration), and, once the file type-checks, runs drop elaboration: where a number handed over so is used again (`E3040` after the call, `E3041` on a loop's next pass), the migration gives each call that handed it over a copy, `twice(owns copy of b)`; a number not used again stays `twice(owns a)`. Lowering records each such argument's own span (`Body::handed_at`), since a call's move is spanned by the whole call. No message changes (a migration is not printed). | N30's ruling, "keeping the 0.9.10 meaning", where 0.9.10 code uses the number again, and only there. |

### `a10_recursion` on the LLVM backend, 2026-10-30 (owner: "yes build it")

The LLVM backend lost `a10_recursion` (1.71x C): not the backend's doing. The C backend built by clang
lost it too (1.72x) and by MSVC passed (1.02x): clang turns the C twin's `fib(n - 1) + fib(n - 2)`
into a loop with a running total, one call a turn, because C's `+` cannot fail, and does not turn
Ember's checked `+`. ODR-277 (2026-10-23) had settled it as the overflow check's price, after a rewrite
with a 128-bit total measured slower (131.5 ms against 116.1 ms). The owner, shown the loop written by
hand with every `+` still checked (Ember 61.26 ms, C 64.79 ms with clang): "yes build it".

| Where | Choice | Why |
|---|---|---|
| `ember_analysis::accumulate`, after inlining and before the range facts remove checks | A function whose recursive case returns a call of itself added to a value runs that call as a loop's turn adding into a running total, every addition checked with the plain 64-bit check, when the range facts prove no added value negative (or the type unsigned): the arguments the function passes itself bound its parameters, and within the bounds every other value it returns is at least 0. | With every value at least 0, one grouping of the additions overflows exactly when the other does, so the program panics in exactly the cases it did; the 128-bit total of ODR-277 was what a rewrite needing no proof had to pay. |
| The same | On an overflow the turn makes the call it stands for, then panics with the written `+`'s message: after the function's last call, as the function would have. | The program's output and the moment it stops are the function's own whatever its calls print; and a path ending in the runtime's cold panic costs the loop one jump, as the written check did. Noting the overflow in a flag instead cost a register and two instructions a turn (72.5 ms with clang). |
| The same | The frame holds only numbers, `bool`s and `void`: nothing to drop. | A drop would happen at another time. |
| The same | For clang and the LLVM backend the turn ends with the head's test; for MSVC the loop is tested at its start. | Measured: the C backend's C built by clang ran `fib(38)` in 85 ms tested at the start and 61 ms at the end (medians of nine runs on one core); MSVC's in 65 ms at the start and 80 at the end (`ember bench`). |
| The same (found by the README's benchmark run, 2026-10-30) | Not when gcc compiles the C. | gcc does better with the written recursion: `bench/programs/a10_recursion`, development, medians of 11 in WSL: 3 ms as written, 52 ms as the loop tested at its start, 38 ms at its end; its C twin 31 ms. The README's run had shown 2.70 times the C (0.02 on 2026-10-16). |

Result (`ember bench --compare --core 1`, mains): MSVC 65.94 ms against C 104.58 (0.63x); the C backend by
clang 61.70 against 64.71 (0.95x); the LLVM backend 61.97 against 64.55 (0.96x). Tests:
`TYP-8/accept_a_recursive_sum_run_as_a_loop_gives_what_it_did_as_written`,
`TYP-8/run_fail_a_recursive_sum_run_as_a_loop_panics_where_it_did` (every profile, both backends, and the
evaluator agreeing), `milestones.rs` `a_recursive_sum_never_negative_runs_as_a_loop`.

### The LLVM backend the default, 2026-10-30 (owner: "yes, could you add it to readme benchmarks too?", then "a")

`[CG-LL-1]` makes the LLVM backend the default once it passes the conformance and the performance suites;
both did. Switched, the driver's tests showed nine things the C backend's build did and the LLVM
backend's did not (the driver returned before them). The owner, asked whether to build them first or
keep the C backend the default meanwhile: "a" (build them first). None is a language decision.

| Where | Choice | Why |
|---|---|---|
| `llvm::selected` | `--backend`, then `[build] backend`, then `EMBER_BACKEND`, then LLVM. The C backend stays one flag away; its errors (`--emit c`, `--cc msvc`, no Clang) say how. | `EMBER_BACKEND` is CI's matrix, as `EMBER_CC` is; four platforms keep the C backend tested with each C compiler. |
| Tests | A test that reads the C backend's output (`--emit c`, `--emit header`, C units, the C natvis) names `--backend c`; the rest build with the default, or `EMBER_BACKEND`'s. | They test the C backend whatever the default is. |
| `--build-id`, `--report=engine`, `--emit-optimization-report` | The same functions for both backends (`build_id` hashes the backend too; `print_optimization_report`, `print_host_vectorisation_report`); the LLVM backend records the virtual calls it makes direct as the C backend does. | One answer whichever backend built. |
| `[BLD-13]` with Clang on Windows | Debug information without the command line or the working folder, linked by `lld-link /pdbsourcepath:.` where LLD is found (`clang_windows_debug_flags`), as clang-cl already was; the runtime object compiled without the working folder. The LLVM program built through Ninja, each unit an object of its own under `obj` (Clang's one-call build named its object after a temporary file). | Microsoft's linker writes absolute paths into the program database and hashes them into the program: one program built in two folders differed. |
| `[ALC-3]` | The entry stores the two glue functions in the runtime's `alloc` and `free`, as the C entry does. | It was refused. |
| `[BLD-FFI-1b]`, `[BLD-FFI-4]` | `inherit_cpp_projects`, the C path's, runs for the LLVM backend too (Clang's side). | The LLVM build never inherited the C++ projects' settings. |
| `[BLD-FFI-2]` | The CMake module passes `--backend c` with any compiler but Clang. | Only Clang compiles the LLVM backend's IR; the module builds with the project's compiler. |
| `[CG-C-9]`, `[CG-LL-3]` | The LLVM backend's visualisers (`ember_codegen_llvm/src/visualisers.rs`), from the types its debug information describes: a `.natvis` with one entry for every `Array`, `Span` and `MutSpan` and the texts, one per enum, struct and class; GDB and LLDB scripts by the DWARF names. An enum with payloads described as its `tag` and the union of its variants (it was an empty structure). In CodeView a name is written as C++ writes a template's (`Array<i64>`, `tuple$<i64,bool>`, `a::B`), as Rust's debug information for Windows does; DWARF keeps the Ember name. | cdb refuses a `.natvis` naming `Array[int]` ("Failed to register NatVis for type"). Checked with cdb: `numbers` shows its elements, `word` `"local"`, `maybe` `Some(7)`, `b` `Bag { items: [0x2], label: "hi", shape: Circle(r: 1.500000) }`. |
| `[FFI-37]` | The LLVM backend counts each call to a foreign function at its site (`ffi_enter`/`ffi_leave`), writes the table of foreign functions and links the C backend's instrumentation unit; `ember tcb` and `--version` resolve the backend's compiler as the build does (`resolved_backend`). | Nothing was instrumented, and evidence an LLVM test wrote was "stale" against the C compiler. |
| `[BLD-4]` (ODR-244) | A `debug` program a unit per module, as the C backend's (a `development` one too until D-932), written on the `-j` threads; in units every definition the IR makes `internal` is `weak_odr` in a COMDAT of its name, so each unit makes the glue and tables it needs and the link keeps one, and each unit declares the bodies it names that another defines. | Measured on `bench/bigpkg`, a body edited, debug, medians of five: 6.00 s before (the whole IR compiled again, 3.02 s), 2.99 s after (0.50 s), against the C backend's 3.32 s; a clean build 7.05 s against 8.53 s. `linkonce_odr` may drop a definition only another unit uses. |

| `[RT-4]` | The entry's call of the Ember `main` is `noinline`. | In units, ThinLTO inlined `main` into the C entry, whose frame the backtrace does not name: a `development` panic printed "no backtrace" (`RT-4/run_fail_a_release_panic_names_its_frames`). |
| `[TOOL-2]` | A build asking for `clang` (the LLVM backend's) takes the archive's Clang before one on `PATH`. | The default backend would otherwise need a Clang the archive already carries. |
| `[DSP-3]` (found by the README's benchmark run) | An interface call on a class handle first tries the program's classes that implement the interface, at most four, by their type information, each called through its own table (a constant the optimiser inlines through), as the C backend does; the runtime's out-of-line search stays behind them. | The LLVM backend searched the tables and called through them for every element: `bench/programs/a07_virtual_calls`, release, 1.41 times its C++ twin (the C backend by clang 0.98); with the tests 1.01. The performance suite has no interface call. |

Also found: `--version` and `ember tcb` said `backend: c` whatever the backend, and `ember bench` used the
C backend unless told; each now resolves the backend as a build does.

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


## 12. Rulings writing 0.9.20_Hardened_1 (lead, 2026-11-03, a cloud session, under the delegation)

The owner's task (2026-11-03): read the latest 0.9.15 spec, find the security holes, memory-safety issues,
consistency problems and missing detail, and fix them, solo, as `0.9.20_Hardened_1`, with fixes that are general.
Every rule below is a refusal of a program that was unsafe, a fact the text left a compiler to guess, or a
correction of the words; the change log is §H.51. Each was judged by the owner's condition on the delegation
(`docs/AUTOPILOT.md` §3: the better shape, never the cheaper one), and none takes a feature away.

| # | Rule | Ruling | Why |
|---|---|---|---|
| N52 | THR-8, THR-9, LT-46, TYP marker table | A `sharedref mut` link is neither `Send` nor `ThreadShared`; it never reaches another thread, and only the scoped tasks of its thread change its target (under a lock, an `Atomic` or a reduction). | `[THR-13]`'s last bullet said an unscoped thread never holds one, but `[THR-8]`'s definition of `Send` and the marker table listed every other one-thread word and not this one, so a struct holding such a link was `Send` by omission: its holder on another thread would change the target while this thread's holders do. |
| N53 | CLO-17, CLO-18, COST-3, shape B1 | A function value is not stored in a place its body may reshape or end through a name it captured with a word (a `shares mut`, `thread_shared` or `sharedref mut` owner or link, a `weakref` link opened `ref mut`), read through its calls: `E3022` where it is stored. `[CLO-18]`'s "holds one count on its state across the call" is gone. | A state block is not counted (`[CLO-10]`), so no count could keep it alive; a handler stored in `bus.handlers` that captured `weakref bus` (no loop of owners) could run `b.handlers.clear()` and free its own state while running. The rule extends `[CLO-17]`'s real-variable rule to every captured way in, and the check is made at the store, as the existing one is. |
| N54 | LT-48, COST-3 | The `runtime` check reads, from the kind's own method table, which of the method's parameters its result may point into (one bit per source parameter and one for `this`, the fact `[LT-22]` records for each implementation), and stops the program when the result may point into what the line changes: one load and one bit test. | A compare of kind identities (the lead's N49) is right only when every kind is known when the function is compiled; a `pub` function of a package with a `runtime dyn` parameter cannot know the kinds a dependant declares, and an unseen kind would pass the check into a use after free. The fact in the table holds for a kind declared anywhere. `dyn_kinds.rs` changes with the milestone that builds it. |
| N55 | VII.1 terms, OWN-3, THR-2, glossary | A hand-over is a reshape of the giver's place: a view into a value still used after it is handed over (to a name, an `owns` parameter, a return, or another thread) is `E3060`, and a value is not handed over while a `ref mut` name of it is used. | `[BCK-4]` said a view of a place whose owner is handed over is `E3060`; the language Parts said nothing, and `[THR-2]` covered only a view opened from a link. A `sharedref mut` link into `players[0]` after `players` went to a thread would race with that thread. |
| N56 | RC-5 point 1 | "an object of that type" is the view's class family, held directly or through anything that owns it, all the way down. | A `Player` derived from `Entity` is ended by clearing an `Array[Entity]`, and by clearing a list of teams that own players; `[EXC-3]` already reasons by class family. |
| N57 | RC-1 | `shares x` of a local that is not a class object makes the value in a block of its own from the line that made it, as if that line had said `shares`, so nothing moves on the `shares` line; only a local that owns its value may be shared. | "a local of this function (which becomes one of the owners)" moved an inline value into a heap block, which would dangle every view of it and change the local's type on one path only. Allocating from the making line keeps every view and one type; the allocation is still shown by the `shares` line. |
| N58 | ARN-4, ARN-7 | A growing `Arena` grows by taking a new chunk and never moves what it has given out. | `[ARN-1]` promised outstanding views live until a reset; a growth that reallocated one buffer would move them. |
| N59 | CLS-2 | A `virtual` method called from `init` is judged by every override's body the program has. | A base `init` cannot know which kind it builds; the one body the text named would miss an override that lets `this` out or reads an unset field. `[DSP-5]` already counts every override for calls. |
| N60 | CTL-7 | A `defer` block's uses of the locals it names are uses at every exit of the block: one handed over below the `defer` line is `E3040`, a view it uses ended before the exit `E3060`. | Nothing said what a hand-over after `defer: println(xs)` meant; the block would read a moved value. |
| N61 | RC-4 | A thread's end empties and closes its queue in one atomic step; a subtract that finds the queue closed ends the value itself. | Point 2 queued a value for a making thread that might be ending at that moment, so the value would be ended by nobody (`[OWN-2]`). |
| N62 | THR-20 | A `static mut` the tasks of a scope reach is a place `[THR-11]` checks as any other (`E7007` other than by a reduction, a lock or an `Atomic`). | `[STA-1]` let the main thread's scoped tasks reach a `static mut` and no rule said two tasks may not both change it. |
| N63 | SIMD-7 | A grouped overflow check needs a loop body with no operation on an `Atomic`. | A relaxed atomic store is not `Sync`, and another thread could read the wrapped value the group stores before the process aborts, so "not observable" was false for it. |
| N64 | ECS-8 | The registry's signal listeners run before a method's result view is made; `emplace`'s `ref mut T` is found again after them. | A listener that reshapes the pool would leave a view made before it dangling. |
| N65 | FFI-10 | The sentinel for an empty span is read-only memory for a `Span[T]` and a slot of the call's own frame for a `MutSpan[T]`. | One shared static `T` written by a C function that breaks its count contract is a corruption every caller shares and a race between threads. |
| N66 | CT-2, MAN-8 | `comptime.read_file` resolves every symbolic link before it checks the path lies under the package root; `comptime.env` in a dependency reads only the variables the root package's manifest lists for it (`[comptime] env`, by default none). | A link inside the package escaped "only inside the package directory"; a dependency's compile-time code could read the builder's environment into the program it builds. |
| N67 | BRW-16 | A position keeps the index it was made with; a later change of the index moves no position. | "found again at each use" did not say with which index. |
| N68 | OWN-8 | `copy of` before a `shares`, `shares mut`, `thread_shared` or `sharedref mut` link makes a `T` with one owner. | `[TYP-36]` and §IV.6 said "a copy of its target" without its type. |
| N69 | CLS-12, EFF-27, the attribute table | `@cached` is for a class method (`E2117` on a struct's or enum's); a change made by foreign code or through a raw pointer is unsafe code's obligation. | A kept result is hidden state, which `[PHIL-9]` and `[TYP-11]` deny a struct; a change the compiler cannot see cannot clear it. |
| N70 | ALC-1 | A memory manager's instance held inline in a struct beside values made in it is `[LT-25]`'s shape: it is held in a block of its own, or the values are made in an instance the struct does not hold. | A value keeps its instance's address; moving the struct would break it. |
| N71 | CT-5 | A `comptime(e)` that owns heap memory has the type of a read-only view of the value (`Span[T]`, `StrView`, `ref T`) with the `static` region. | The text said "a read-only view of it" and never its type. |
| N72 | THR-19 | An `RwLock` is among the locks a thread may not wait for while it holds them (`write()` under `read()` and the reverse). | The text named only "a lock of the kind this thread holds" with a `Log`'s lock as the example. |
| N73 | TYP-13 | A `shares`, `shares mut` or `thread_shared` value, a `sharedref mut` link, a `ref mut` and a `MutSpan` have a niche. | Each is a pointer that is never null; the list left them out. |
| N74 | FN-3, §IV.5, TYP-15a, TYP-47, STD-9, §II.3, §XVII.2 | `[FN-3]` follows `[LT-1]` (verdicts 137, 138); `ref_to_ptr` is one built-in whose result follows its argument's word, not two functions of one name (`[TYP-26]`); `BorrowList[T]` is no longer named (nothing declared it); the `"\n"` of `println`'s first line is written as text; the examples name the current version. | Consistency, each a contradiction or an undefined name in the text. |
| N75 | VER-9, CLI-20 | The version is 0.9.20_Hardened_1 (the owner's name for it); the migrator writes the `#! language` directive and rewrites nothing else, since every change of 0.9.20 is a refusal. The compiler's `LANGUAGE_VERSION` moves with it, so `[VER-8]` and the document agree, as M1 did for 0.9.15. | `[VER-9]`: the accepted set changes. |


## 13. Rulings writing 0.9.20_Hardened_2 (lead, 2026-11-03, a cloud session, under the delegation)

The owner's task (2026-11-03, after Hardened_1): "the final mile ... make it as foolproof as possible ... remove all the
gaps, flaws and memory issues. Make the spec near 100% complete so that any agent can pick it and start implementing
without any disruptions. Also answer all the open questions in this pass with the best judgement." A workflow of 29
agents ran (`tasks/96/redesign/audit-0.9.20/round1/`): nine readers over the Parts, nine refuters, and for the open
questions O1, O2 and N38 to N40 three designers and three judges apiece; the lead ruled from the judged set, ruled
the smaller questions alone, and applied everything with asserting scripts. The change log is §H.52; the disposition
of every finding is `round1/applied.md`. Each ruling was judged by the owner's condition on the delegation (the
better shape, never the cheaper one); the one that touches an owner verdict says so.

| # | Rule | Ruling | Why |
|---|---|---|---|
| N76 | BLD-16 (new), OBJ-1, THR-2, WK-18, WK-16, RT-3, MOD-9, BLD-14, COST-3 (O1) | No fact of the whole program decides a package's code or layout. A type carries a weak count when it can be named from outside its package (`pub`, or named in a `pub` signature) or a `weakref` type in its own package names it, decided by the declaring package; a hand-over to another thread walks the owned objects by their declared layouts, with a hand-over routine in `TypeInfo` for a layout the handing package cannot see; the deferred-drop region is read from the class and the link's type; a `weakref` of a `dyn I` or `some I` value is refused (`E2119`). | A package compiled once must lay out its objects and test what it tests the same in every program (`[BLD-1]`, hot reload, `ember check` alone). The judges preferred the declaring-package rule over "every class carries the count" (8 or 12 bytes wasted on every private object) and over a link-time relayout (which changes a compiled package's layout). The cost is one header word on a type named from outside its package, nothing else. |
| N77 | GRM-3a (new), TYP-45, OWN-13, WK-11, WK-12, WK-16, WK-17, WK-18, RC-5, THR-1, THR-2, THR-8, THR-9, THR-11, THR-17, CLO-16, HEAP-7, HEAP-10, EFF-4, BCK-7, MNG-1, RT-10, the grammar, the marker table, Appendix F (O2) | `weakref thread_shared T` is a type of its own, the link to a `thread_shared` value, whose opening holds a count; `weakref T` links a value on one thread and is made only to a value no `thread_shared` value reaches (`E7015` at the line or at a call by the callee's summary); the kind of a class's objects is read from its own methods (`weakref this` one-thread, `thread_shared this` shared); `weakref shares T` and the other stacked words are `E0100`. | The opening of a link compiled two ways on one type, choosing by a flag read while running (O2); with the kind in the type, opening a `weakref T` is one load and one compare, no flag, no branch, and the plain weak count is never touched from two threads (a `weakref T` names an object this thread owns, as Rust's `Rc` never sits inside an `Arc`). The judges' grafts: the kind read from the path's words so `weakref x` needs no second word; the class pinned by its methods so a line that makes an object of the other kind is refused. |
| N78 | LT-48, LT-1, LT-22, DSP-2, DSP-3, LEX-15, the grammar, X.4 (N38, N39) | `runtime` is written on a parameter or a local declared with its type, before `dyn` or an `open`/`abstract` class name, under any holding word; the fact is read at the call from the called method's entry in the kind's table (bit 0 `this`, bit k parameter k, 64-bit words sized by the declaring interface or base), kept with the result, tested at each guarded line; a result that leaves the function carries no fact. | One word with one meaning wherever it stands, and a `virtual` call on a class value checked as a `dyn` call is (the slot already loaded holds the fact beside the code). A `runtime` type or field was refused: the function whose line pays says so. |
| N79 | GRM-43, GRM-47 (new), GRM-15, OWN-9, FN-2a, E0119 (new) (N40) | The value words bind the same everywhere, as a prefix `-`; a computed argument handed over goes in brackets (`m.insert(owns (i * 7), owns i)`); `owns` as an operand of an operator is `E0119`, naming the operator and the line to write; `owns` before a value made on the line for a non-`owns T` place is `E0109`. | One binding rule is what a reader can hold; "a call's `owns` covers the whole argument" would make `owns i * 7` mean one thing in an argument and another on the right of `=`. `owns` is the one word that ends what it stands before, so it is never an operand; the other words make a value and keep their meaning. |
| N80 | THR-15, THR-9 (O3) | `Sender[T]` and `Receiver[T]` are `Send`, and `Sender[T]` is `ThreadShared`, exactly when `T` is `Send`; an end of a channel of another `T` given to another thread is `E7015`. | A channel of a one-thread owner would carry it across threads. |
| N81 | EXP-6 (O4) | A hand-out of the value inside an `owns T` block (`m: Mesh = b`) is `E3013`, with `mem.take(ref mut b)` as the line to write. | The value belongs to its block; reading through it stays a view. |
| N82 | JOB-5 (O5) | `jobs.local_arena()` is a `ref mut ThreadArena` whose region is the job function's; a value made in it that would leave is `E3063`; a call outside a job stops the program. | The arena is reset when the job ends, so nothing made in it may outlive the job's function. |
| N83 | MAN-9, FFI-55 (O6; **reverses half of verdict 135**, reported) | No compile command writes a file the program did not ask for: a file with no manifest is compiled as a package named after its folder with default settings, and a note says so once; `ember.overlays` is kept only beside a manifest that exists. Verdict 127 (every program is a package) stands. | Verdict 135 had the manifest written on the first compile, so `ember check` left a file behind; the owner (2026-11-03): "it's all good if in the end the spec turns out to be better than what it is now, better solutions are welcomed". The owner may reinstate the written file. |
| N84 | OBJ-1, RC-4, OPT-1, OBJ-4, ALC-1, COST-3 (O8, with G3-11 and G4-04) | The runtime takes the memory for `thread_shared` blocks made in `Global` in chunks from the configured allocator, writes the making thread into each chunk's header once, and a chunk belongs to that thread for the life of the process; a `thread_shared` value made `using` an instance is marked handed over at its making line, so no thread asks which thread made it; a `thread_shared` value is never placed in a frame or a manager's instance. | A block from `malloc`, a host allocator or a `using` manager has no chunk header to read; the making thread must come from memory the runtime laid out, or not be needed. Verdict 101's header stays 32 bytes. |
| N85 | STD-11 (O9) | A map or set keyed by what an outside party chooses is made with `RandomState`, said where `Map` is defined. | A fixed seed lets that party choose colliding keys. |
| N86 | FFI-53 (O10, with G7-20) | The tagged number's generation always has its bits 0, 7 and 15 set, so the number is non-canonical on x86-64 under 48- and 57-bit addressing and on AArch64 under top-byte-ignore. | A generation whose top byte was zero made the number a user-space address on AArch64. |
| N87 | OBJ-1, FFI-35a (G7-08; **touches verdicts 101 and 114**, reported) | The kept-by-C mark is the top bit of the type index, set by the call's wrapper through a `retained` pointer to a class type; a type index is below 2^31. | Verdict 114 asked for the stop at a hand-over of a marked object, and `[OBJ-1]` held no bit for the mark; the header's size is unchanged. |
| N88 | the 153 findings and 37 missed items of round 1 (`round1/applied.md`) | Each applied as its refuter's better fix where one was given, else as the reader proposed, merged where two covered one rule; the 10 refuted not applied. | §H.52 lists each rule with its reason; `round1/G*.json` hold the readers' programs, the refuters' arguments and the better fixes. |

## 14. Rulings writing 0.9.20_Hardened_3 (lead, 2026-11-08, a cloud session, under the delegation)

The second round of the 0.9.20 hardening combined the gap review of 0.9.15_Hardened_11 (76 clusters, 47
new gaps, `tasks/96/redesign/spec-gaps-0.9.15/`) with Hardened_2's text: 10 readers, 10 refuters, 14
designers, 9 judges and a critic (44 agents), records in `tasks/96/redesign/audit-0.9.20/round2/`
(`results/`, `applied.md`). Every edit was applied by an asserting script against the text at HEAD. Owner
verdicts touched are marked; none is overridden. The five design rulings are also OPEN ODRs 260–266.

| # | Rule(s) (source; touches) | Ruling | Why |
|---|---|---|---|
| N89 | TXT-11, TYP-21, §X.4 (K18) | `a + b` on a `String` reads both operands and makes a new `String`; `s += t` appends in place. Every operator reads its operands. | The one operator that took its left side over did so with no word on the line (the owner's test of 2026-10-24), and GRM-47 (N79) forbids `owns` before an operand; no verdict covers text `+`. |
| N90 | TYP-14, TYP-5, TYP-36, EXP-9, §VI.3, E2150 (K30) | A `shares`, `shares mut` or `thread_shared` owner is read through and has its value's `Eq`, `Ord`, `Hash`, `Display`, `Debug`; `a == b` compares values, `a is b` identity, on owners and links alike. | The rules named only `ref` and `owns T`; the E2150 row and EXP-9 decided links two ways. |
| N91 | TYP-5 (K41) | Rule 7 (a read-through) composes before any rule; rule 11 (`Some`) after any one rule, one level. | "rule 11 after rules 1–4" predated rules 8–12. |
| N92 | GRM-13, the grammar (K64; ODR-264) | List patterns defined, nested or on a `for` line; `[..rest]` producible; a top-level arm `[x]:` stays `E0100` (verdict 155). | The grammar produced them and nothing said what they match. |
| N93 | TYP-17, TYP-22, TYP-32, the grammar (K65, NG-23; **extends verdict 109** to `dyn`/`some`; ODR-265) | Several interfaces after `dyn` or `some` are written in round brackets; `dyn (A, B)` is one data pointer and one table pointer with member-table pointers; a marker in the list gives the marker; `+` is `E0100`. | One spelling for a list of interfaces; the set's layout keeps `[DSP-3]`'s call. |
| N94 | TYP-47, the grammar (K68) | A pack is spelled out in a tuple type and a function type; a tuple over a pack is formed once spelled out (one element is the type itself, none is `void`); `Option[Cs]...` is `E0100`. | Verdict 115's uses could not be written. |
| N95 | FN-2a, OWN-9 (K19; **touches verdict 159** in a case it did not consider) | A memberwise constructor's and an `owns` `init` parameter's value is handed over with no `owns`; a plain `init` parameter reads for the call; `ref mut` is written. | Two sentences decided `Label(caption)` two ways; the alternative (hand over and drop) is in the digest for the owner. |
| N96 | CLS-3, CLS-10 (K32) | A memberwise constructor is the base's `init` for every purpose (`super.init(name=name)`); its arguments take the constructor's words. | `super.init` on a base with no `init` had no target. |
| N97 | CLO-10, §X.4, CLO-18 (K40, R2 missed) | The real variables a function value uses are reached through one frame address in the state word; a word-capture beside real variables makes a block; a one-word owner capture whose target the body changes through it needs no block. | `E4001` under `@noalloc` varied between implementations. |
| N98 | STR-1 (K66) | A memberwise constructor's visibility is the narrowest of its fields' across `[MOD-2]`'s three levels. | The rule knew two levels. |
| N99 | UNS-12, E2020 (K11) | `drop`, an `override` of a safe method and an interface method's implementation are never `unsafe fn`. | Safe code ran them with nobody vouching. |
| N100 | FN-1, BRW-21 (K17; **narrows verdict 5** across a call, the lead's area-2 Q4 ruling) | Across a call a plain parameter promises only that its value is not ended or moved; a scalar another name writes shows through. | The promise was wider than what `[BRW-21]` checks. |
| N101 | EXP-4, THR-19, E7018 (K23) | A `match` subject's temporaries follow the condition rule; a lock guard that is a temporary holds its lock for its life. | `match owns JOBS.lock().pop(): Some(job): job.run()` deadlocked under a promise to refuse it. |
| N102 | BRW-14, BRW-15, CTL-1 (K24) | A name handed its value (an `owns` parameter, `for x in owns e`, `match owns e`) owns it and may be changed and reassigned. | "plain" was read two ways. |
| N103 | GRM-15, OWN-11, TYP-44, DSP-6 (K25) | `body = owns req.body` hands a part over on `=`; the words before a value act on a cast after the kind test. | Three lines no rule decided. |
| N104 | BRW-12, OWN-8, TYP-36, E2116 (K26; **brings the text to verdict 68**) | A part of a `Copy` type is copied by `=`; a read-only view is `Copy`, so `copy of w` is one more view; `E2116` only for `ref mut`, `MutSpan` and a struct holding one. | OWN-8 gave `E2116` for `StrView`, which the tables made `Copy`. |
| N105 | SPN-3, BRW-12 (K42) | `left, right = ref mut xs.split_at(mid)` gives each name the right to write its half. | No line could write both halves. |
| N106 | CLS-12, EFF-27, E2117 (K43, R3 missed) | A `@cached` body reads no field of a weak-counted class object reached through `this`'s fields, and no `static mut`. | A `weakref` opened `ref mut` changed what a kept result read. |
| N107 | EFF-9, EFF-12, DSJ-6, LT-48, COST-6, PRF-4, HR-35, E4030 (K58) | A `runtime` name's test is a `RuntimeCheck(Aliasing)`, refused under `@no_alias_checks`, admitted under `@nopanic(explicit)`. | The test had no effect kind. |
| N108 | THR-15, E7015 (K01; **restores verdict 164's letter**, correcting N80) | A channel of a `T` that is not `Send` is `E7015` where it is made. | The verdict showed the channel line refused. |
| N109 | THR-1, THR-9, the marker table, E7015 (K02, NG-24; verdict 165) | A `thread_shared` value's type is `Send` and `ThreadShared`, judged by components; a `Receiver` is one consumer's and not `ThreadShared`; a `thread_shared fn()`'s state is `ThreadShared` unless `once fn`. | The making-line check asked `Send` alone. |
| N110 | TYP-43, FFI-8, E7008 (K36, R5 missed) | A class may vouch `Send` for its raw pointers; `ThreadShared` is vouched only on a foreign type, never one bound to a thread. | TYP-43 and FFI-8 contradicted each other. |
| N111 | PAR-2, PAR-2b, PAR-3, THR-21, E7007, E7010 (K37, K38, K39) | A `@parallel` range loop writes an `Array` the body does not reshape; a body's reductions are declared and `[THR-11]`'s exceptions hold; a reduction's accumulator starts from the operator's neutral value, `-=` gathers what is taken away, one operator per place. | A `u8` place stopped the program on the rule's own example. |
| N112 | THR-2, CT-5, OBJ-1, E7015 (K13) | A `weakref T` link never names what every thread may reach (a `static`, a guarded `static mut`, a `comptime(e)`); a `thread_shared` value in a `static` is placed writable, marked handed over. | A plain weak count in read-only memory, written from two threads. |
| N113 | STD-8 (K29) | `x in c` for collections of `String` takes a `StrView` needle; a literal needle is a `StrView` where an implementation takes one. | `"bob" in names` was typed two ways. |
| N114 | SER-1, §XIV.2, E2080 (K49) | A `@ser(skip)` field is given what `@derive(Default)` gives a field; none is `E2080`. | The attribute had no reading side. |
| N115 | DET-2, DET-4, STD-27, E4070, E1010 (K55, K73, R6 missed) | `Nondet` is a platform function whose result IEEE 754 does not fix, `**` on a float and a SIMD `_approx`; `std.math.det` has a twin for each under `std.math`'s names (`ln`, not `log`). | `cbrt` differed between C libraries and was admitted. |
| N116 | CT-1, CT-2, E6010, E6005 (K56, K72) | `unsafe` code runs at compile time under the evaluator's checks; `comptime.read_file -> StrView` and `comptime.env -> Option[StrView]` run only while compiling, carry no `Io`. | Two rules decided one write two ways; the inputs had no type. |
| N117 | SOA-1, SOA-6 (K69) | A flattened field is a proxy over its columns; a proxy's compound assignment is the gather, the operation and the scatter. | The write side was unsaid. |
| N118 | FFI-31b, FFI-56, FFI-2a, FFI-17a, FFI-11a, FFI-41, FFI-8, STA-1, UNS-1, FFI-51, TCB-4, LEX-23, TIER-1, FFI-21, FFI-10, FFI-52, FFI-11, FFI-42, FFI-36, FFI-36a, GRM-35, FFI-55, FFI-53, COST-6, FFI-40, FFI-17f, FN-6 (K08, K09, K15, K16, K31, K46, K47, K50, K53, K75, K76, NG-29, R7/R8/R10 missed) | The Part XVI items of `applied.md`: a function pointer C hands an export; a result that points somewhere; a header's `const` object; `unsafe import cpp` by the property; `owns cb: fn()`; a handle's words; operators named by their method; `adopt` with `delete`; a registering call's result; the overlay line's header; a full callback table; `T&&`. | Each an implementer's undecidable line, recorded in the digests. |
| N119 | TYP-46a, TYP-46, ALC-1, OBJ-1, OBJ-4, OPT-1, RC-4 point 4, LT-4, the marker table, COST-3, E2114, E7015 (the K05 ruling; **touches verdict 101** as N84 did, 47's word placed in the header's queue place; ODR-260) | Shape B: `thread_shared T using A` allowed, counted away from its making line, refused only where the value would hold a link to an instance not every thread may call. | The better shape, no feature dropped; the method-3 team's shape A not taken. |
| N120 | RC-4 points 3–5, THR-11, THR-20, DRP-8, WK-3, OBJ-1, HR-44, THR-5, COST-3 (the K10 ruling; **touches verdict 101**: which threads keep a home count; method 4's decisions 1–4 superseded) | Plain types and home threads: a value of a type that is not plain, or made on a task's, job's, worker's or host thread, is marked handed over at its making; the queue holds only plain values made on main or a spawned thread; a check point may stand anywhere, at the return of every export and trampoline too. | J7-01's hang and J7-02's lost clean-up closed with nothing checked while running. |
| N121 | FFI-2, FFI-10, FFI-11, FFI-21, FFI-56, FFI-17a, TCB-4, CLI-5, the Vulkan example, E5002, E3100 (the K07 ruling; ODR-199's shape changes; verdicts 60, 120, 125, 130, 133, 136 applied) | The limit on raw pointers under the safe-to-call fact, for its four spellings; a `std::function` value is `unsafe fn(A…) -> R` until its entry says `safe`. | Safe code handed C an address it made up under an assertion of safety. |
| N122 | UNS-1, FFI-8, FFI-31b, FFI-39 (NG-11..21, NG-32..41, the R10 refuter's form) | Giving foreign code a raw pointer is an unsafe operation, for every route no declaration covers. | One property by the operation, no provenance. |
| N123 | THR-8, THR-9, TYP-43, CLS-4, OWN-13, IFC-3, DRV-1, ERR-3, ERR-8, BLD-14, BLD-16, Annex B, the marker table, E7015, E2040, E3100, E9030 (the K20 ruling; **extends verdict 61**, changes verdict 110's example; ODR-261) | An `open` class's header promise; `Error` needs `Send` and `ThreadShared`. | A base-class value's markers were read from fields a dependant could add to. |
| N124 | FFI-57, FFI-40b, TYP-26, §I.6, FFI-17, FFI-17b, FFI-44, FFI-39, FFI-39b, FFI-40a, FFI-12, FFI-30c, GRM-35, FFI-55, FFI-48, CLI-5, COST-3, MNG-1, §XVII.6.2, E5010, E5035, E5056 (the K21 ruling; verdict 86 kept; ODR-262) | A C++ name declared several times; a `const`/non-`const` pair as two names. | Every C++ class with two constructors was `E1030`. |
| N125 | TYP-46, ALC-5a, OWN-13, GRM-3, ALC-1 (NG-03, R9 missed) | `weakref T using A`: the give-back is read from the link's type; a class made `using A` is a type of its own and `this` carries the kind. | A link's end had four give-backs and no datum. |
| N126 | CLS-4, E2113; IFC-3, E2040 (NG-30, R10 missed) | An override, and an interface implementation's method, has the first line of the method it replaces. | A slot call read a `MouseEvent` as a `KeyEvent`. |
| N127 | DIA-18, E0119, THR-2's message, FFI-10, DSP-4, TYP-45, FFI-48, 26 registry rows | Words made consistent (critic's list and the missed items). | Every cross-reference claims what its rule says. |
| N128 | the 123 clusters and gaps and 34 missed items of round 2 (`round2/applied.md`) | Each applied as its refuter's better fix where one was given, else as the reader proposed, merged where two covered one rule or a ruling covered it; the 3 refuted and the duplicates of Hardened_2 not applied. | §H.54 lists each rule with its reason. |

N83 and N87, still for the owner after Hardened_2, are ruled in §15.

## 15. Rulings closing the open ODRs (lead, 2026-11-08, a cloud session, on the owner's instruction: "go for the best solution possible, that matches all the goals"; solo, no agents)

Each open owner-decision record (ODR-260 to ODR-266, ODR-238) and the new ODR-267 was weighed against the four goals and the owner's verdicts and closed in `docs/OWNER-QUEUE.md`; the record of what was weighed is `tasks/96/redesign/audit-0.9.20/round3/rulings.md`. Every edit was applied by an asserting script (`scratchpad/edit33.py`, `edit34.py`, `registry4.py`) against the text of 0.9.20_Hardened_3 and cut as 0.9.20_Hardened_4. Owner verdicts touched are marked; one is departed from (155), one restored (110).

| # | Rule(s) (source; touches) | Ruling | Why |
|---|---|---|---|
| N129 | TYP-46a, LT-4, ALC-5, OBJ-1, RC-4 point 4, ALC-1, the two [COST-3] rows, E3063 row; FFI-54, E2114 row (ODR-260) | Shape B of `thread_shared T using A` stands; its text made exact: growth and end count for `[THR-11]` as the calls they make, by the methods' summaries; the hand-over to an unscoped thread, a job outside a scope or a kept callback is from a `static` instance only; the queue place is free for a value counted away from its making line, so the instance word sits there at no byte and one method body serves every manager; `using x` goes through any name where `allocate` reads the instance; `into_raw()` on a value made `using` another manager is `E2114`. | A `static mut` instance may be reset by a line of the main thread while the thread runs, a fact of the whole program (`[BLD-16]`); `ember_rt_object_release` gives back only blocks the runtime laid out. Verdicts 19, 47, 101, 107, 164, 165 kept. Two designers' results that finished before the workflow was stopped agreed (`round3/results/odr-260-designs.json`); their doubt, a generic body over an `Allocator` parameter, is decided by `[TYP-17]`: refused at the definition, since no bound provides the property. |
| N130 | ERR-3, ERR-8, ERR-1, DRV-1, THR-8's example, THR-9, TYP-17, §IV.8; E2040, E2080 rows (ODR-261; **restores verdict 110's letter**) | `interface Error: needs (Debug, Display):`; `AnyError` holds `owns dyn (Error, Send, ThreadShared)` and its `From[E]` needs `E: needs (Error, Send, ThreadShared)`, so a one-thread error is `E2040` at its conversion into `AnyError`, not at `implements Error`; the open-class header promise of Hardened_3 stands. | Rust's shape (`std::error::Error` needs neither marker; `anyhow::Error` does): refuses no one-thread program, keeps std's declaration, costs nothing at run time; `Result[T]` still crosses threads with nothing written. |
| N131 | FFI-57, FFI-40b, FFI-42, FFI-8 aliases (ODR-262) | Stands: one Ember name of several members, `at`/`at_mut`, transparent C aliases with `E5035` on a tie. | `_mut` is the language's own spelling everywhere; a nominal `c_int` would tax every C call for one case a `rename` settles. Verdicts 131, 132, 60 kept. |
| N132 | RC-4 point 3, the [COST-3] rows (ODR-263) | Stands: a check point at every return into C and around a foreign call. | One load and a branch at a boundary that already costs a call; an away mark on entering C needs an atomic protocol against re-entry and runs a plain `drop` on a foreign thread. Verdict 101 kept. |
| N133 | GRM-40, GRM-13, E0100 row (ODR-264; **departs from verdict 155** in a case it did not consider) | A list pattern may open a `match` arm (`["quit"]:`); every other line beginning with `[` stays `E0100`. | An arm's line is a pattern ending in `:` where no bound list and no statement can stand; the verdict's targets were a misplaced bound list and a list as a statement. Reported; the owner may reinstate the refusal. |
| N134 | TYP-17, TYP-22, TYP-32 (ODR-265) | Stands: `dyn (A, B)`, `some (A, B)`. | Verdict 109's spelling; a named interface is no substitute, since interfaces are nominal (`[TYP-40]`). |
| N135 | FFI-53, E4040 row (ODR-266) | Stands; the stop on a full table is the failure `Panic(Explicit)` lists, on every target, so `@nopanic(explicit)` code registering a kept callback is `E4040`. | 16 bits of slot on a 32-bit target is the width's limit; slots are reused through their generations; a `Result` would tax every 64-bit program for a case it cannot reach. Verdicts 114, 130 kept. |
| N136 | CLO-4, CLO-7, EFF-24, BLD-14 (ODR-238) | Answered by the text: a kept lambda's captures are checked at its maker through the callee's summary (`E3026`), a registry takes an `owns` function value; no new word. | `[CLO-15]` was removed in 0.9.10_Hardened_35; ODR-100 untouched. |
| N137 | FFI-8, E3063 row (ODR-267, new) | A `@ffi(handle)` value read out of an `owns T` is a view of the owner: it lives no longer than the owner, whose end destroys the object, and is `E3063` where kept beyond it; a handle no `owns T` gave stays the plain `Copy` value. | A stale copy is not a fact an overlay author can assert away; a run-time check per call is refused by §4. Verdicts 130, 132, 136 kept. |
| N138 | MAN-9, FFI-55 (N83 closed) | Stands: no compile command writes a file the program did not ask for. | `cargo`, `go` and `zig` build a plain file without writing a manifest; the owner welcomed better solutions (2026-11-03). Half of verdict 135 remains departed from, reported. |
| N139 | OBJ-1, TYP-44a, the §X.4 kind-test row (N87 closed; touches verdicts 101 and 114 as N87 did) | Stands, one fact stated: a kind test, a cast and a table load read the type index with the kept-by-C mark cleared, one `and` on the loaded word. | Without it a marked object (its top bit set, `[FFI-35a]`) failed every `is` test; one `and` costs nothing measurable. |

## 16. Rulings writing 0.9.20_Hardened_5 (lead, 2026-11-10, the local session, attended, solo; the owner: "read the parts you haven't read and fix them too, before creating a new hardened spec")

The findings are in `tasks/96/redesign/spec-gaps-0.9.15/deep/check-0.9.20-H4.md` (1 to 23, T1 to T9); the owner's words are verdicts 167 to 169 in `JOB.md`. Each ruling below is the lead's under the delegation, weighed against `docs/OWNER-DIRECTION.md`, and the owner's to overturn. None adds a code or a word.

| N | Rules | Ruling | Why |
|---|---|---|---|
| N140 | GRM-15, OWN-10, E0109 row (finding 6) | Where the parameter's own type is an `owns T`, one `owns` before a value made in the call says both that it has a block of its own and that the call takes it over; a second `owns` is `E0109`. | Two rules each asked for the word and the examples wrote one. Verdict 160 already says that the word which makes the value says what is given (`setup(shares mut game)`); `owns` is such a word. `owns owns` would say one thing twice. |
| N141 | BRW-16 (finding 7, corrected) | A position is made by indexing an `Array` (and an `SoA` proxy is one, `[SOA-6]`); a name for an element of any other container is made by a call of its `index` and is an address. | `Deque`, `SmallArray` and `BitSet` are plain structs of `std/src/collections.em`, normative by Part XV's preamble, so their operations were not missing (the finding's first half is withdrawn); only "a list" in `[BRW-16]` was unsaid. An address is the safe reading: a container that grows at the front, or moves from inline to heap, ends it. |
| N142 | WK-7, OWN-13, MOD-9a, BLD-14 items 11 and 12, EFF-2a, EFF-4, BCK-8, E3032 and E7015 rows (finding 17) | What a function another package adds (an implementation, an override, a stored function) keeps of what a library's call gives it, an owner stored or a `weakref T` link made, is judged where the function is seen: for a kind the caller gives through a `dyn`, interface or base-class parameter, the function's summary records the call with which of its parameters the object and each argument are, and the caller applies the kind's own records (items 9 and 10) as the function's; for a kind a place holds, the line's record names what owns what there and what a `thread_shared` value reaches, and the member is checked where it is added. A kind the judging package cannot see counts as doing both. | The library could not see the member and no record carried these two facts, so a loop of owners compiled (`h` → plugin → `h`) and a one-thread link could be made to a value another thread may end, against `[WK-6]`, `[OWN-13]` and `[PHIL-10]`. Weighed: (a) the library assumes every unseen member keeps every owner it can make: sound, but refuses every plugin that is given a shared host and then stored in it, innocent ones too; (b) a word on the interface's parameter saying "not kept": a new word, and a fact the compiler can work out (owner direction 1 and 2); (c) this, the shape items 11, 12, 15 and 16 already have for every other fact: precise, decided when compiling, nothing while running. `[BCK-8]` already promised it ("a line checked this way by no build is a defect of the build"). |
| N143 | RC-1, OWN-13, THR-1, CLS-2, CLS-7a, OBJ-5, ALC-1, E2122 row (finding 18) | `thread_shared this` is written in a method: it pins the class, so `this` is a `thread_shared` value and the word makes one more owner. `shares this` and `shares mut this` are `E2122` in every method. | `[OWN-13]` built a class's kind on these words and `[RC-1]`'s list of where they may stand left `this` out. A one-thread object may have one owner, which ends it with no count to test, so an owner made inside a method could outlive it; a `thread_shared` class is pinned, so every object of it is counted. For the owner, later: a pin for one-thread shared classes (every object made `shares`) would make `shares this` sound; it is a new restriction on how objects are made and was not added. |
| N144 | JOB-5, E3022 row (finding 19) | The arena `jobs.local_arena()` gives is the job's own: a scope of its worker thread's arena, opened when the job starts and ended when it ends, which gives back only the job's bytes; one place for each job, so a second name of it while the first is used is `E3022`, across calls by the summary. | The rule did not say whether the arena was the thread's or the job's; with the thread's and a full reset, a job run inside another job's wait on the same thread (`[PAR-4]`) would reset memory the waiting job still views. A scope is last in first out, as the calls are, and costs one mark per job. |
| N145 | section VI.6, PHIL-13, PRF-1, E3023 and W2016 rows (finding 21) | A call in `debug_assert`'s argument of a function that changes or ends anything that outlives the argument, by its summary, is `E3023`; an effect that changes nothing stays `W2016`. | `[PHIL-13]` said the argument "may not have side effects", the check was a warning, and it read the effect set, which does not hold a change of a module variable: `debug_assert(bump())` gave `debug` and `release` different values with no message. The summary is where changes are (`[EFF-24]`). |
| N146 | MAN-9, FFI-55, CLI-4, the XVII.1 listing (verdict 168) | Verdicts 127 and 135 restored: the default manifest is written, and `ember.overlays` sits beside it for every program. N83 and N138 are withdrawn. | The owner's word. With no file, a single file's overlay was never checked against its header (`[FFI-55]`). |

Text repaired with no ruling (findings 1, 4, 5, 20, 22, 23, T1 to T9): two example blocks whose line breaks Hardened_2's re-wrap removed; the front matter; thirteen table rows without their table's indent; `[LT-4]`'s help line; `split_at_mut` in `[DSJ-11]`; `ember audit` in the listing; `[CLI-20]`'s change log; a sentence of `[TYP-32]`; a citation written twice in section I.6; three lines of the glossary (a `Copy` type, a hand-over, a position) that said other than `[STR-5]`, `[OWN-9]` and `[BRW-16]`; unit-only enums in three lists of `Copy` types (`[ENM-3]`); a parenthesis of `[TYP-36]` that read against `[OWN-8]`; a name split in two in `[HASH-2]`; a blank line at the end of every Part, the three generated ones included. `tools/check_examples.py` now reports a code fence that shares its line with other text and exits non-zero on one.

Not changed, and reported to the owner as readability costs of his own verdicts (findings 13 to 16): the hand-over word at a call beside none at `=`, `return`, a constructor and a list literal (verdicts 147 to 149, 158 to 161); `ref mut` with `MutSpan` and `as_mut_span()`; a scoped task's `total += x` as a reduction (verdicts 104, 105).

## 17. Rulings writing 0.9.20_Hardened_6 (lead, 2026-11-10, the local session, attended, solo; the owner: "Fix them and cut a new hardened version")

The findings are §8 of `tasks/96/redesign/spec-gaps-0.9.15/deep/review-0.9.20-H5.md` (another agent's review of Hardened_5); the lead's check of each, by runs, is `docs/AUDIT-0.9.20.md` §11; the owner's word is verdict 170 in `JOB.md`. Each ruling below is the lead's under the delegation, weighed against `docs/OWNER-DIRECTION.md`, and the owner's to overturn. None adds a code or a word.

| N | Rules | Ruling | Why |
|---|---|---|---|
| N147 | HR-12, HR-13, the COST-3 row (hole 1, note 2) | In a reloadable build an object is registered by its own type: not `Send`, plain stores in its own thread's list; `Send`, ordered by the runtime as a `thread_shared` object's is. A registered container the same. Adding to the list is four stores. | The text gave every object that is not `thread_shared` the plain stores and said they touch nothing another thread touches; a one-owner object of a `Send` type is ended by another thread (a task's result, a channel), which unlinks it from a list its maker is changing. The runtime today takes one lock for every object (`rl_live_insert`, four stores), so the hole was in the text only. Nearly every class is `Send`, so this is close to what the runtime does; the reviewer's cheaper variant (an end on another thread queued to the maker) is not written, since nothing has measured it and it needs the maker known for every block. |
| N148 | FFI-51, TCB-4, E3100 row (hole 2) | A header imported with no `unsafe` only declares: types, macros, declarations, `inline` and `static` functions with no assembly, and `static`, `inline` or `constexpr` variables with a constant initialiser, no destructor and no named section. Anything else counts as running. A library a header names (`#pragma comment(lib, …)`) is an entry of `[link] libs`; `#pragma detect_mismatch` is nothing; the toolchain's own headers are the toolchain. | The rule listed two ways to run code before `main`; the reviewer found a third (a pointer in a start-up section), and the lead two more by running them (a destructor attribute; `#pragma comment(lib/linker)`, which put `WINMM.dll` into a program's imports). A list of tricks leaks, so the rule states what a declaring header is and counts the rest (`[FFI-38]`'s own principle). The pilot shaped the two allowances: `windows.h` asks for `uuid.lib`, every MSVC C++ standard header asks for `libcpmt` and passes `/alternatename`, and `<wchar.h>` defines a symbol; without them every such import would need the word and it would mean nothing. With them, none of the repository's 49 C headers needs it, nor does `vulkan.h`. The toolchain is already trusted: its start-up runs in every program. |
| N149 | MAN-10, E9001 row (hole 3) | The compile of a dependency's C reads only files under its own root and the toolchain's headers, checked against what the C compiler reports it read, and holds no assembly; `native` in the root's manifest lifts both. Each flag is one argument, never through a shell. | `[CT-2]` promises a dependency cannot read the builder's files and `[MAN-10]` limits its `-I` paths for that, but a header names any path itself: `#embed "C:/Windows/win.ini"` compiled on Clang 22 with a warning. Clang's `-MD` names an embedded file, so the check is exact there. The reviewer's "file-scope `asm`" is widened to any assembly: `.incbin` in a function body's `__asm__` read a file too, and no report named it. MSVC 14.51 has neither route. The reviewer's "the root package's include paths" is left out: `[BLD-FFI-1a]` gives a dependency's compile only its own flags. |
| N150 | EFF-2, EFF-2a (note 1; verdict 38) | A parameter of a function type is the shared type: one body, one indirect call, and what it may do is every function its package gives it, with a dependant's checked at the dependant's line. Only a function given through a type parameter is compiled once for each function given. | Not a ruling of the lead's own: verdict 38 says both forms exist and differ ("the shared type written on the parameter ... (C++'s `std::function`), is legal too"), with the owner's measurement (73 ms against 266 ms). 0.9.20_Hardened_2 made the parameter per function given in `[EFF-2]` and `[EFF-2a]`, left `[COST-3]`'s row ("a parameter ... one indirect call") and `[BLD-14]` item 16 ("or passes to it") saying the other, and did not report it. Judged under verdict 167: not the better solution (two spellings with one meaning, no way left to ask for one body, and items 12, 13 and 16 already make the shared form sound across packages). |
| N151 | FFI-57 (note 3) | Among the members a call fits, number types rank as in C++: the argument's own type, then a promotion C++ makes (an integer narrower than `c_int` to `c_int`, `f32` to `f64`), then every other number type alike. | The rule promises "where C++ chooses one Ember chooses the same", and "the narrowest number type" chose `set(short)` for an `i8` where C++ chooses `set(int)` (run on Clang 22 and MSVC 14.51). The reviewer's own example, `i64` against `f64` for an `i32`, cannot arise: `[TYP-5]` turns no integer into a float. |
| N152 | BLD-16, TYP-46a, ALC-5a (note 4) | A rule new in 0.9.20 carries "new in 0.9.20" alone; `[BLD-16]` no longer cites itself. | Form. The reviewer named one rule; three carried both marks. |

## 18. 0.9.20_Hardened_7: the owner's verdict 171 applied (lead, 2026-11-10, the local session, attended, solo)

| N | Rules | Ruling | Why |
|---|---|---|---|
| N153 | FFI-51, TCB-4, MAN-10 (verdict 171; withdraws part of N148) | N148's two allowances are withdrawn: a header that has the linker add a library does more than declare, and the toolchain's own headers are judged as every other header is. `#pragma detect_mismatch`, which can only stop a link, stays outside the rule: it brings no code. `[MAN-10]` keeps the toolchain's headers among the files a dependency's C may read, and defines them itself. | The owner's word: "ANY code and I mean ANY code that we can NOT guarantee is safe needs a unsafe keyword". The lead's reason for the allowance, that a word written on most imports tells a reader nothing, weighed convenience against the promise that safe code holds nothing unvouched; `docs/OWNER-DIRECTION.md` principle 11 already says which wins. Not touched, and put to the owner: a library linked from the manifest (`[link] libs`, `[BLD-FFI-3]`) is listed by `ember tcb` and has no `unsafe` word of its own, as before 0.9.20. |

## 19. 0.9.20_Hardened_8: the owner's verdict 172 applied (lead, 2026-11-10, the local session, attended, solo)

| N | Rules | Ruling | Why |
|---|---|---|---|
| N154 | BLD-FFI-3, FFI-49, FFI-51, MAN-8, MAN-10, E3100 row (verdict 172) | The owner's two lines as he wrote them: `[link] unsafe_libs = [...]` in the manifest, `unsafe import` with its note in the code. What the lead added to make them a rule, each his to overturn: (1) the key `libs` is refused with the line to write, since two keys for one thing would leave the unmarked one in use; (2) an import's own `link = [...]` option (`[FFI-49]`) is gone, since it linked a library the manifest did not name; (3) which imports need the word is worked out by the build, from the names each library defines: an import of a header that declares one of them. | The owner's word: "I want the toml and the code both to be explicit, the toml says we are using unsafe libraries and the code imports unsafe libraries". (3) is the compiler doing the bookkeeping (`docs/OWNER-DIRECTION.md` principle 1): tried with `llvm-nm` on `vulkan-1.lib`, which lists `vkCreateInstance`. It is decided by what the header declares, not by which calls a program makes, so the import line does not change when a first call is added. Not decided, and put to the owner: whether a library a header itself asks for (`#pragma comment(lib, "uuid.lib")`, which already makes its import `unsafe`, verdict 171) must also be listed under `unsafe_libs`. |

## 20. 0.9.20_Hardened_9: the owner's verdict 173 applied (lead, 2026-11-10, the local session, attended, solo)

| N | Rules | Ruling | Why |
|---|---|---|---|
| N155 | FFI-51, BLD-FFI-3, MAN-10, E9001 row (verdict 173) | A library a header asks for is named under `[link] unsafe_libs`; an import whose header asks for one the manifest does not name is `E9001`, with the line to add. | The owner's word: "Yes, list it too". The code is the manifest's (`E9001`), since what is missing is a manifest line; the wording is the lead's. Seen and put to the owner, not decided: the manifest has no way to name a library for one target only, so a program that must name `libcpmt` for MSVC names it for every target, where no such library exists. The same was already true of any program that links different libraries on different systems. |

## 21. 0.9.20_Hardened_10: the owner's verdict 174 applied (lead, 2026-11-10, the local session, attended, solo)

| N | Rules | Ruling | Why |
|---|---|---|---|
| N156 | BLD-FFI-3, MAN-8, MAN-10, FFI-51 (verdict 174) | `[link.windows]`, `[link.linux]` and `[link.macos]` each take `unsafe_libs`; a build links `[link]`'s list and then the list of the system it is for (that of `--target` or `[build] target`, the host's otherwise) and does not look for another system's. Filled in by the lead, his to overturn: a library named in both lists is linked once; the import of its header is `unsafe import` on every system; a dependency's per-system lists are limited as its `[link]` list is; `E9001`'s help for a library a header asks for names the system's section first. | The owner's word: "Yes, add them". The section form is the manifest's own (`[profiles.debug]`); the three names are the three systems Ember's toolchain is published for (`[TOOL-1]`). No rule warns of an `unsafe` word that is not needed, so one import line serves every system. |

## 22. 0.9.20_Hardened_11: the eight fixes of the review of Hardened_10 (lead, 2026-11-11, the local session, attended, solo; the owner: "fix everything")

The fixes, and the test behind each, are `tasks/96/redesign/spec-gaps-0.9.15/deep/fixes-0.9.20-H10.md` (another agent's; Clang 22.1.8 and clang-cl on Windows, GCC 15.2 and Clang 21.1.8 with glibc 2.43 in WSL). The lead applied them as written, except where a row says otherwise, and did not run the tests again but for item 2.

| N | Rules | Ruling | Why |
|---|---|---|---|
| N157 | MAN-10 (item 1) | A file a dependency's C reads is judged by its real path, links resolved; one with more than one hard link is refused. | A junction inside the dependency reached a file outside it, and the compiler's report named the path inside. |
| N158 | FFI-6, Annex A, its fixture (item 2) | `link` is no option of `import c`; the message says where libraries go. Annex A's line is `import c "zlib.h" as zlib`. | The lead's miss in Hardened_8 (a search for `link = [` did not find `link=[`). **Differs from the fix as written**, which shows `unsafe import c`: the fixture is compiled by `annex_a_is_generated_from_a_fixture_that_compiles`, and today's compiler refuses `unsafe import`; a plain import of a header is valid Ember and compiles today, where teaching the test to drop the word would have been a test hack. The line becomes the `unsafe import` form when the compiler builds it. `spec_check.py` now fails on a removed spelling left in the text. |
| N159 | FFI-51, MAN-10 (item 3) | An `asm` label that is one plain symbol name only renames its declaration; any other label is assembly. A dependency's C holds no `asm` statement, and where the compiler writes assembly text (GCC) the build reads it for directives that read files or make directives. | glibc's `<stdio.h>` holds five renaming labels, so every Linux import of it was `unsafe import`; with GCC a label, a section name or a `symver` put a file into the object with no report. |
| N160 | MAN-10 (item 4) | The toolchain's headers are the folders the C compiler searches with no manifest flag and none of the environment variables that add folders, as it lists them, each by its real path. | The text never said which folders; `INCLUDE` or `CPATH` could add one. |
| N161 | BLD-FFI-3, E9001 row (item 5; verdict 175) | A library an object of a named library asks for is named under `unsafe_libs` too; the build links besides only the runtime's own libraries; shims are compiled without default-library names (`/Zl`); what a shared library loads is vouched for with it. | The owner's word. Every MSVC-built static library asks for `libcmt.lib`; without `/Zl` every shim would. |
| N162 | FFI-54, FFI-27, a COST-3 row (item 6) | An object whose type is not `Send` is ended by C, and taken back, on the thread that made it, checked in every profile; after that thread ends it can only be ended. | Nothing checked it. The cost row is the lead's addition: `[COST-3]` lists everything that runs. Not run: the runtime does not build it. |
| N163 | FFI-57, E5035 row (item 7; verdict 175) | A call of a C++ name with several members stands for the member Clang's overload resolution chooses; a member reached only through a conversion Ember does not make is `E5035`. Replaces the K21 ranking and N151. | The owner's word. The ranking chose otherwise in 2 of 4 cases tried (a plain function against a template; two templates). The sentence on two members alike on a target (`put(int)`, `put(long)`) stays. |
| N164 | BLD-FFI-3, E9001 row (item 8; verdict 175) | A library named under `unsafe_libs` that nothing uses is `E9001`. **The lead added** "and which no header and no named library asks for": as written, a library that N161 makes the manifest name, because another library asks for it, would have been refused by this rule. | The owner's word. Its start-up code runs although nothing calls it (LLD and GNU ld, without `--as-needed`). |

## 23. 0.9.20_Hardened_12: the six fixes of the check of Hardened_11 (2026-11-11, the local session, attended, solo; the owner: "option A and then fix the rest yourself")

The check, and the test behind each fix, are `tasks/96/redesign/spec-gaps-0.9.15/deep/check-0.9.20-H11.md` (the session that wrote `fixes-0.9.20-H10.md`; Clang 22.1.8 on Windows, GCC 15.2 and Clang 21.1.8 in WSL). Applied as written there, by that session.

| N | Rules | Ruling | Why |
|---|---|---|---|
| N165 | FFI-51, MAN-10 (item 1) | Where the C compiler writes assembly text (GCC), the shim's and the thunk unit's text is read as a dependency's is; a directive there, outside a quoted name, that reads a file or makes directives is assembly the header holds, `E3100` without `unsafe`. | With GCC 15.2 a `weakref` target and an `#ident` line in a header that only declares put a file into the object; Clang quotes both. Fix 3 of the review of Hardened_10 covered `asm` labels only. Reading GCC's own output covers any string it writes as given, not a list of attributes. |
| N166 | FFI-57 (item 2) | A call between members alike on this target is decided by Clang as any other (`put(int)` for an `i32`, whose C++ type is `int32_t`, and for `7`); only a use without a call, one callable type for both, is `E5035`; a call of two instances differing only in the result fits neither. An override is matched by its first line, not by overload resolution. | The paragraph kept from the ranking said such a call fits both members, which contradicted "the member C++ chooses" (Clang 22.1.8 on Windows chooses `put(int)`). |
| N167 | RT-1, BLD-FFI-3 (item 3) | The runtime's own libraries are listed for each system in `[RT-1]`: the C runtime the profile chooses with what it asks for itself, the C compiler's support library, and the system libraries `ember_rt` calls (on Windows `synchronization`; on Linux `libc`, `libm`, `libpthread`; on macOS `libSystem`). `[BLD-FFI-3]` cites `[RT-1]`. | `[BLD-FFI-3]` cited `[TOOL-2]` for the list, which names none, so which libraries were exempt was the implementation's to choose. The list is what the build links today (`ember_build`: `-lm -pthread`; the MSVC runtimes, which ask for `kernel32`, `ucrt` and `vcruntime` themselves; `ember_rt.c`'s `#pragma comment(lib, "synchronization.lib")`). |
| N168 | Annex A, its fixture (item 4; verdict 176) | Annex A's C import is `unsafe import c "zlib.h" as zlib`, with a `# SAFETY:` note that names `unsafe_libs = ["z"]`. | The owner's choice of two shown to him. The plain line was legal only while nothing linked zlib. The fixture test needs no change: it already marks the fixture as one the compiler does not build (`FIXTURE_KNOWN_TO_FAIL`). |
| N169 | `tools/spec_check.py` (item 5) | The removed `link` option is found in any place among an import's options (`\bwith\s*\([^)]*\blink\s*=`). | The pattern found `with (link=` only, not `with (defines=[…], link=[…])`, `with ( link =` or `with(link=`. |
| N170 | FFI-57, E9001 row (item 6) | An untyped literal reaches C++ as C++ spells its value (`1_000` as `1000`); the `E9001` row says a library of which no import and no `unsafe extern` block declares a name, and which nothing asks for. | `3000000000` is C++'s `long long` on Windows and `long` on Linux, not `int`; the row said "uses" where the rule judges by what is declared. |
