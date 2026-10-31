# M2 — whole-program facts, with nothing new refused: work plan (lead, 2026-11-01)

MILESTONES.md: A4-1 (the call graph, done), A4-2 (the summary engine, with A2-9's borrow facts in it,
C2), A4-3 (summaries in interface files), A9-11 (the "stored into a callable type" index), A1-17's first
half (`static mut` as a place, Y2); D-920 (fixed 2026-10-29). Nothing is refused by M2: every fact it
builds is read by a later milestone's check (M3 on), and until then the program means what it means
today. Each slice lands on `main` on its own, as M2 allows ("analysis that only adds facts"), with the
fast check and the Rust suite green, and the push gate before each push.

What exists: `ember_analysis::summaries` (A4-2's first slice, `06fb0bbc`): changes, reads and ends
per place a parameter reaches, through links, with the chain of calls; a fixed point over all bodies;
printed by `ember build --emit mir`. It is not stored on a body, not computed before the borrow check,
not kept by incremental builds and not in interface files. SD1 (`gap/04`) says what a summary holds
and where it lives; this plan follows it.

## Slices, in order

Slice 1 is built last, with the first check that reads a summary (M3's), not first (lead, 2026-11-01):
made before the borrow check of every build, it is a whole-program fixed point each build pays and
nothing reads until then, and the compile-time budgets (`[BUD-*]`) count it. Until then the facts are
made where they are printed (`--emit mir`) and tested there, slices 2 to 4 grow them, and slice 1's
placement, keeping and verifying land when a check needs them.

1. **Where the summary lives (SD1).** A `FunctionSummary` on each MIR `Body`, beside
   `callable_regions`, serde like it; made by the driver after drop elaboration and before the borrow
   check (the order SD1 gives, since every borrow rule at a call reads it from M3 on); a kept body
   keeps its own (as `install_callable_regions_kept` does), and the verifier makes it again from the
   MIR and compares (`[VERIFY-3]`'s pattern), so a stale one is a compiler failure, not a wrong fact.
   `--emit mir` prints the stored one. Tests: the existing `--emit mir` summary test, and an
   incremental one (a kept module's summaries equal a clean build's).
2. **Reshape** (may move or free memory: `push`, `clear`, `remove`, a hand-over of an owner): the
   fourth kind beside change, read, end; built-in operations given theirs. Consumer: `[BRW-1]`/`[BRW-18]`
   (M3+). Tests: `--emit mir` output for `push`, `clear`, a chain through a call.
3. **The abstract places SD1 adds:** `Static(symbol, path)` (with slice 7), `Captured(k, path)` mapped
   to the maker's variable, `HeldBy(i, path)` for function values stored under a parameter, `OwnState`.
   `Linked` waits for M4's links (`shares`, `weakref`), which do not exist yet. **`Static` built**
   (2026-11-01): a fact's place starts at a parameter or a static (`Root`), a pointer the static
   built-ins give points at the static, and a callee's facts about one reach the caller unchanged (a
   function with no parameters recorded nothing of a static before). Open: a change made through a
   read-only view of an `Atomic` or a `Cell` (interior mutability) is a read in the summary; whether
   the rules reading it count it as a change is settled with them (`[THR-*]`, `[CELL-*]`).
4. **Per-parameter facts:** which view parameters (`this` included) it uses after a line that may end
   or move what they view (`[BRW-21]`, `E3031`'s fact); what it reaches by other names while a `ref mut`
   parameter is live; "only adds" for a `ref mut` list parameter; the elements it touches; where it
   keeps a view or function-value parameter (the `StoreSummary` of `regions.rs` folded in, A2-9's
   facts, C2); whether it lets `this` out and which fields of `this` it reads; whether a function
   value's body may end its own state and use it after. Slots, empty until their milestones: area 3's
   "sets the parent link before reading it", area 5's locks taken and needed (`[THR-18]`).
5. **A4-3: summaries in interface files.** `.emif` schema 15: each callable's summary with each chain
   step as file and line; a changed summary is an interface change, so dependants are checked again
   (`[BLD-14]`, `[EFF-4]`, `[MOD-9]`). Test: the `[EFF-4]` milestone test with a library's two lines
   swapped.
6. **A9-11: the callable-type index (SD2).** Every function converted into each callable type, from
   the function items' conversions (`Cast`/`Const::Fn`), replacing `plain_function_values`' guess by
   arity and result (`effects.rs`); the summary's calls through a function value read it. Tests: the
   three `EFF-1` tests of a call through a variable, with a callable parameter that is not a type
   parameter (`[EFF-2]`).
7. **A1-17's first half: `static mut` as a place.** A function's `X = v` / `X.push(…)` on a module
   variable assigns it by name (`[GRM-4]`); a `static` that is not `mut` assigned by name is refused;
   the help for a module-level assignment outside the entry file; codegen: a plain mutable C global for
   one with no heap memory, the first-access accessor on writable storage for one that owns heap
   memory (`[CT-5]`). Tests: `STA-1`, `STA-3`, `FN-8`'s help, `FN-6`'s static function value.
   **It lands with `E7012`, not before it** (lead, 2026-11-01): `[STA-1]` lets only the main thread
   and its scoped tasks reach a `static mut` not `guarded by` a lock, and accepting `static mut` before
   that check would let two threads change one in safe code for as long as the window lasted
   (decisions.md §3's split put the thread check after A5-18). So slice 7 comes after slice 3's
   `Static` places and the call graph's spawn and job edges (A4-2's, `decisions.md` EFF-2 row), and
   refuses, conservatively, a `static mut` any function a non-scoped thread may run reaches; `guarded
   by` (the threads area's) lifts it later.

Each slice's facts are printed by `--emit mir` and tested there; no slice changes what a program
compiles to or whether it compiles (slice 7 accepts a form 0.9.15 writes, and refuses one 0.9.15
refuses, `[STA-1]`).

## Owner questions

None expected: M2 builds facts the spec defines. A choice that changes what a program means (slice 7's
refusal of a non-`mut` `static` assigned by name is the spec's, Q3) is the spec's, not a ruling.
