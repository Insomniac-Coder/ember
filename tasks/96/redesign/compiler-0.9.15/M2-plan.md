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
   value's body may end its own state and use it after. **"Only adds" built** (2026-11-01): the
   facts *adds* (`push`, `extend`) and *removes* (every other reshape), and `Summary::only_adds`. **"Used after a line that may end or move what it views" built**
   (2026-11-01, `[BRW-21]`, `E3031`'s fact): `Summary::used_after`, per view parameter (`this` included),
   the places a reshape or an end may reach on a line after which the parameter, or a view made from it, is
   still used (its regions live at the line's successor: `borrows::regions_all`, the borrow check's own
   regions from the installed contracts); a callee's own fact passed on where the view went into its
   argument (the operand's origins at the call). A change made through the parameter itself is not one
   (`[BRW-1]`: a `ref mut` name is alone): the parameter's own place, through no handle or link past its
   reference. The `if a is not b:` precision of `[BRW-21]` waits for the disjointness facts (M6c).
   **"What it reaches by other names while a `ref mut` parameter is live" built** (2026-11-01):
   `Summary::read_while` and `changed_while`, per `ref mut` parameter (a region slot holding a changeable
   view), the reads and changes at a line where it is still used (a static's place among them, `[BRW-1]`'s
   check at each call); at a call, only when it is still used after the call, since one the call is its
   last use of goes to the callee, whose own fact about that parameter is passed on. A parameter's own
   place is what its leading reference and handle reach (every `Deref` before its first field, element
   or variant step), so a `ref mut n: Node`'s own object is its own. While 0.9.10's run-time access
   brackets stand (M6b), an `end_access_write` after a call keeps a parameter live a line longer.
   **Not folded yet:** where it keeps a view or function-value parameter stays the borrow check's
   `StoreSummary` (what the checks read today) until slice 1 puts the summary on the body; it moves then,
   not copied before. **Found while building it, D-944:** a task a built-in runs (a scoped or pooled
   task, a `@parallel` loop's chunks) was not followed; its facts are now the starting function's.
   **"Lets `this` out" built** (2026-11-01, `[CLS-2]`'s `init` check, A4-12's input):
   `Summary::lets_this_out`, a method's object as a whole (or a copy or view of all of it) passed,
   stored into a field, an element, a struct or a closure's state, or returned; a method called on it
   lets it out as that method does. **"Fields of `this` read"** are the summary's reads of `this` (its
   paths' first field), so nothing new is stored. **"A function value's body may end its own state and
   use it after"** is `used_after` of a closure's environment parameter: what the caller checks it
   against (the place it is called from) is the call check's, M6a.
   Slots, empty until their milestones: area 3's
   "sets the parent link before reading it", area 5's locks taken and needed (`[THR-18]`).
5. **A4-3: summaries in interface files.** `.emif` schema 15: each callable's summary with each chain
   step as file and line; a changed summary is an interface change, so dependants are checked again
   (`[BLD-14]`, `[EFF-4]`, `[MOD-9]`). Test: the `[EFF-4]` milestone test with a library's two lines
   swapped.
6. **A9-11: the callable-type index (SD2).** *Measured 2026-11-01:* of 3,293 test programs, 102 make a
   loose function value: a closure a built-in runs itself (a scoped task, `run_staged`'s, a `@parallel`
   loop's), a C function given an `extern "C"` type, a C++ bridge function. None is a value of an Ember
   callable type, so none should count for one; but a loose value reaches a check only where a value
   cannot be followed at all (a read through a raw pointer), and no program here shows the difference
   yet: the change waits for a test that fails without it. *Mostly built with A4-1* (`targets.rs`: the place a value
   is read from, verdicts 122, 128, 129, and every function stored into a callable type where nothing
   can be followed; `plain_function_values` is gone). Left: the *loose* functions, those that become a
   value where no callable type is known (a closure's captured state, an operand of an operator, an
   argument of a built-in or of a function with no body here), which count for every callable type of
   their shape; each such place's type is to be read from the operation itself. Every function converted into each callable type, from
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

## Design notes for slice 4's flow-sensitive facts

**Used after a line that may end or move what it views** (`[BRW-21]`, `E3031`'s fact). Per view
parameter `p` (`this` included), the places of the other parameters the body may reshape or end at a
point after which `p`, or a view made from it, is still used. Built on the borrow check's regions, not
beside them: `regions.rs` already knows, for each point, which region slots are live and which locals
hold each parameter's regions (`local_regions`, `values_at`). For each point where the summary's
facts say a reshape or an end happens (a statement's own, or a call's callee's facts mapped onto its
arguments, as `body_facts` maps them), each view parameter whose region slots are live at that point
gets the fact's place in its list. The caller's check (M3) refuses a call that passes, for `p`, a view
into a place its list names. The fact is per parameter pair and place, so a recursive group's fixed
point stays finite (paths are cut at `MAX_PATH`).

**What it reaches by other names while a `ref mut` parameter is live.** Per `ref mut` parameter `q`,
the places of other parameters (and statics) the body reads or changes while `q`'s region is live:
the same points, the same liveness, the read and change facts in place of reshapes. Consumer: the
call check of two arguments that may overlap (`[BRW-5]`, `[BRW-22]`'s noalias).

## Owner questions

None expected: M2 builds facts the spec defines. A choice that changes what a program means (slice 7's
refusal of a non-`mut` `static` assigned by name is the spec's, Q3) is the spec's, not a ruling.
