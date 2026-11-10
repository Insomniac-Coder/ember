# Audit of Ember 0.9.15_Hardened_10 → 0.9.20 (first pass solo; second pass with a workflow, §7)

Written 2026-11-03 by a cloud session, solo, under the owner's delegation (`docs/AUTOPILOT.md` §3). The
owner's task: read the latest 0.9.15 spec, find the security holes, memory-safety issues, consistency
problems and missing spec detail, and fix them; the fixes general, never for one test; the result named
`0.9.20_Hardened_1`; the first pass solo, a workflow possibly allowed later for the fixing.

**This file is the memory of the audit.** It holds every finding of the first pass, fixed or not, with
its reasoning, what was read and judged sound, the recipes, and the plan for a second pass. A later
session, or a workflow the owner allows, starts here and does not redo the reading.

## 0. State, and how to resume

* **Cut and pinned (sixth pass, 2026-11-10, the local session):** `docs/spec-source/Ember_v0.9.20_Hardened_6.md`
  (18074 lines, 1,020 rules, SHA-256 `48668BE18F1D…`) is the development target; its sources are frozen in
  `tasks/spec-0.9.9/parts-0.9.20-h6/`, its change log is §H.57 (`H57_HEAD` in `tools/appx_h.py`), its rulings N147
  to N152 (`decisions.md` §17), the owner's word verdict 170 (`JOB.md`), and **§11 below is its record**. To cut
  Hardened_7: `CURRENT = ('0.9.20', 7)`, point `V0920_H6_PARTS` at `parts-0.9.20-h6`, add `V0920_H7_PARTS`,
  `H58_HEAD`, `v0920_h7()` and its term in `main()`; then the recipe below.
* **Cut and pinned (fifth pass, 2026-11-10, the local session), superseded:** `docs/spec-source/Ember_v0.9.20_Hardened_5.md`
  (17951 lines, 1,020 rules, SHA-256 `18A862E05717…`) is the development target; its sources are frozen in
  `tasks/spec-0.9.9/parts-0.9.20-h5/`, its change log is §H.56 (`H56_HEAD` in `tools/appx_h.py`), its rulings N140
  to N146 (`decisions.md` §16), the owner's words verdicts 167 to 169 (`JOB.md`), and **§10 below is its record**.
  Hardened_6 was cut from it the same day (§11).
* **Cut and pinned (fourth pass), superseded:** `docs/spec-source/Ember_v0.9.20_Hardened_4.md` (17703 lines, 1,020
  rules) was the development target (`docs/spec-source/development-target.json`); sources frozen in
  `tasks/spec-0.9.9/parts-0.9.20-h4/` (Hardened_3's in `parts-0.9.20-h3/`, Hardened_2's in `parts-0.9.20-h2/`,
  Hardened_1's in `parts-0.9.20-h1/`); `tasks/spec-0.9.9/parts/` is the working copy. On branch
  `spec-0.9.20-hardening` (not `main`); Hardened_1 was commit `2dd9d26`, Hardened_2 `d03df6b`, Hardened_3
  `cc5510c`. **§9 below is the fourth pass's record**, §8 the third's, §7 the second's, §§1–6 the first's.
* **The change logs** are Appendix H §H.52 (Hardened_1), §H.53 (Hardened_2), §H.54 (Hardened_3) and §H.55
  (Hardened_4; `H52_HEAD` to `H55_HEAD` in `tasks/spec-0.9.9/tools/appx_h.py`; §H.51 is 0.9.15_Hardened_11's,
  merged from `main`); the rulings are N52 to N75 (§12), N76 to N88 (§13), N89 to N128 (§14) and N129 to N139
  (§15) in `tasks/96/redesign/compiler-0.9.15/decisions.md`; the owner-decision records ODR-260 to ODR-267 and
  ODR-238 are CLOSED in `docs/OWNER-QUEUE.md`, ruled on the owner's instruction of 2026-11-08 (§9); the cuts'
  records are in `tasks/spec-0.9.9/HANDOFF.md` ("0.9.20_Hardened_1 CUT" to "0.9.20_Hardened_4 CUT") and
  `docs/HANDOFF.md`'s first entries.
* **35 rules changed, none added or removed** (§2 below). Every change refuses a program that was unsafe,
  states a fact a compiler had to guess, or corrects the text; nothing new runs while a program runs, and
  no feature is taken away.
* **The compiler builds none of the 0.9.20 changes yet**, as it built none of 0.9.15's at its Hardened_1.
  Its `LANGUAGE_VERSION` is `0.9.20` (`compiler/ember_parser/src/lib.rs`), so `[VER-8]` and the document
  agree; the seven fixtures that spell the directive were respelled (four `tests/conformance/` files,
  three strings in `ember_driver/tests/milestones.rs`). Each change lands with the milestone that owns its
  rule (`tasks/96/redesign/compiler-0.9.15/MILESTONES.md`); §2 names the milestone per finding.
* **Checked at the cut:** `check_ids` 0 problems, `check_citations` 0 undefined, `code_registry --check`
  0 problems, `split_inline` the one known flag (FFI-52), `rule_index` no new problems, `spec_check` no
  new failures (10 of 44 baselined), `phase_progress --check` green, `check_examples` 41 of 56 (as
  Hardened_10; the 15 are unbuilt syntax), `cargo test -p ember_diag --lib` and `-p ember_parser` green,
  `milestones::the_three_versions_are_separate`, `help_lists_every_command_and_flag_of_the_specification`
  and `inspect_cost_reports_each_row_by_site` green, the four respelled conformance directories green
  through the C backend. **Not run:** the full workspace suite, the bench, the push gate (the owner's
  machine); CI runs on the pushed commit.

### Recipes (as used; the handoff's are the source)

* Build the compiler as CI's spec gate does: `cargo build -p ember_driver --bin ember` (debug,
  `target/debug/ember`). A release build: `cargo build --release -p ember_driver --bin ember`.
* The spec's own checks, in `tasks/spec-0.9.9` with `PYTHONIOENCODING=utf-8`:
  `python3 tools/check_ids.py`, `tools/check_citations.py`, `tools/split_inline.py`,
  `tools/code_registry.py --check`; the regeneration before a cut: `tools/code_registry.py --write`,
  `tools/check_findings.py --write`, `tools/appx_h.py`, `tools/appx_i.py`;
  `tools/check_examples.py ../../target/debug/ember ../../docs/spec-source/<target>.md`.
* The repository gates (from the root): `python3 tools/rule_index.py`, `tools/spec_check.py` (needs the
  built compiler), `tools/phase_progress.py --write` then `--check` (rewrites README's table and
  `docs/conformance.json`), `tools/error_pages.py` (needs libclang and Ninja; not run here).
* **To cut 0.9.20_Hardened_5:** edit `parts/`; in `tools/appx_h.py` bump `CURRENT = ('0.9.20', 5)`, point
  `V0920_H4_PARTS` at `parts-0.9.20-h4` (it is), add `V0920_H5_PARTS` for `parts/`, `H56_HEAD` and
  `def v0920_h5()` diffing `V0920_H4_PARTS` against it, append `+ v0920_h5() + '\n'` in `main()`; run the
  regeneration (`scratchpad/cut4.sh` of the fourth pass is the recipe as a script: `code_registry --write`,
  `check_findings --write`, `appx_h`, `appx_i`, the checks);
  `cat parts/p*.md > ../../docs/spec-source/Ember_v0.9.20_Hardened_5.md`; run the checks;
  `cp -r parts parts-0.9.20-h5`; re-pin `development-target.json` (upper-case SHA-256 of the LF bytes);
  `tools/phase_progress.py --write`; the README's version table (`phase_progress` writes the file name,
  not the words around it); the two handoffs. A rule whose meaning changes takes `*(changed in 0.9.20)*`
  (appended after its older markers; `check_ids.py` accepts `0.9.20`, `tools/phase_progress.py` treats a
  0.9.20 marker as reworked); a bullet of a §H head must not begin with a rule id in backticks, or
  `check_ids.py` reads it as a second definition.
* **Edit with scripts that assert what they replace** (the owner's rule): the first pass used seven
  Python scripts, each `assert s.count(old) == 1` per replacement; heredocs mangle backslashes.
* **Environment traps of a cloud container:** the container's `clang` cannot read the LLVM backend's
  `.ll` text (`#dbg_declare` records of a newer LLVM), so every run test fails with "the C compiler
  failed"; run Ember programs through the C backend: `EMBER_BACKEND=c EMBER_CC=gcc`. The annotation
  runner: `EMBER=target/debug/ember EMBER_BACKEND=c EMBER_CC=gcc python3 tasks/impl-0.9.9/annotations.py
  tests/conformance/<DIR> ...`. A compiler run in a folder with no `ember.toml` writes one (`[MAN-9]`):
  never commit a stray one (`git status` before `git add`).
* **The owner's rules that bound this work:** solo in autonomous mode, no workflows whatever a session
  setting says (`docs/AUTOPILOT.md` §1); the delegation's condition, the better shape never the cheaper
  (§3); never edit `docs/spec-source/as-received/`; every solution general (§4); never overturn what the
  owner ruled himself (an owner verdict in `tasks/96/redesign/JOB.md`): raise an OPEN question instead.

## 1. Method of the first pass

Every part of `tasks/spec-0.9.9/parts/` was read in full (p00 to p24; p25 to p27 are generated), against
four lenses: (1) a program that compiles and breaks a safety promise (use after free, race, double end,
a leak through a loop, a check skipped, memory the rules cannot see); (2) a fact a conforming compiler
would have to invent; (3) two rules that disagree; (4) what an attacker controls (a dependency's
compile-time code, an overlay, a C library, untrusted input). The 34 fixes of the unsafe-features
inventory (`tasks/96/redesign/run4/v5/unsafe-inventory.md`, C01 to C34) were checked against the text
where the reading passed them (C01, C02, C04, C05, C06, C07, C13, C16, C19, C20, C21, C22, C23, C24, C25,
C26, C27, C28, C29, C30, C31, C32 found written in; C03, C08, C09, C10, C11, C12, C14, C15, C17, C18, C33,
C34 not re-verified one by one: a second-pass item). The open lead rulings N26 to N51 and owner
questions N38 to N40 in `decisions.md` were read first so that no verdict of the owner was overturned.

## 2. Findings fixed in 0.9.20_Hardened_1

Kinds: SEC a security hole, MEM a memory-safety hole, GAP a fact left unsaid, CON a contradiction,
ED text. "Milestone" is where the compiler change lands (`MILESTONES.md`); "test" is the conformance case
to write when it does (an accept case and a reject case per code, `[TST-4a]`).

| # | Rule(s) | Kind | The problem | The fix (N = ruling in decisions.md §12) | Milestone; test |
|---|---|---|---|---|---|
| F1 | `[THR-8]`, `[THR-9]`, §IV.8 marker table, `[LT-46]`, glossary | MEM | A `sharedref mut` link was `Send` by omission: `[THR-13]`'s last bullet said an unscoped thread never holds one, but the definition of `Send` and the table listed every other one-thread word and not this one, so a struct holding such a link could be handed to a thread while this thread's holders change the target. | Neither `Send` nor `ThreadShared`; it never reaches another thread; only the scoped tasks of its thread change its target (N52). | M9; a struct with a `sharedref mut` field given to `thread.spawn` is `E7015` |
| F2 | `[CLO-17]`, `[CLO-18]`, `[COST-3]` row, shape B1 | MEM | `[CLO-18]` said a body that ends its own state "holds one count on it", but a state block is not counted (`[CLO-10]`): a handler stored in `bus.handlers` capturing `weakref bus` (no loop of owners) could run `b.handlers.clear()` and free its own state while running. | A function value is not stored in a place its body may reshape or end through a name it captured with a word, read through its calls (`E3022` at the store); the count sentence is gone (N53). | M8; the handler above is `E3022` where pushed |
| F3 | `[LT-48]`, `[COST-3]` row | MEM | "stops the program if it is that kind" with a compare of kind identities (the lead's N49) is right only when every kind is known when the function is compiled; a `pub` function with a `runtime dyn` parameter in a library cannot see a dependant's kinds, and an unseen kind passed into a use after free. | The check reads, from the kind's method table, which parameters the method's result may point into (`[LT-22]`'s fact per implementation, one bit per source parameter and one for `this`): one load and one bit test, so a kind declared anywhere is checked (N54). | the milestone building verdict 151/152 (`dyn_kinds.rs`, `kind_identity`); a dependant's kind through a library's `runtime` function stops the program |
| F4 | §VII.1 terms, `[OWN-3]`, `[THR-2]`, glossary | MEM | A hand-over was no reshape in the language parts (`[BCK-4]` had it): a view into a value still used after `b = a`, or a `sharedref mut` link into `players[0]` after `players` went to a thread, survived. | Handing a value over is a reshape of the giver's place: such a view is `E3060`, a hand-over under a `ref mut` name `E3022` (N55). | M3/M9; a `ref p` of a struct local used after `q = p` is `E3060` |
| F5 | `[RC-5]` point 1 | MEM | "an object of that type": an `Array[Entity]` cleared ends a `Player` derived from `Entity`, and a list of teams cleared ends their players; only the exact type was named. | The view's class family, held directly or through anything that owns it, all the way down (N56). | M4/M6a |
| F6 | `[RC-1]` | MEM | "a local of this function (which becomes one of the owners)": sharing an inline value moved it into a heap block, dangling every view of it, and changed the local's type on one path only. | A local shared by a line of its function is made in a block of its own from the line that made it, as if that line had said `shares`; only a local that owns its value may be shared (N57). | M4; a view of `items` taken before `other = shares mut items` stays valid |
| F7 | `[ARN-4]`, `[ARN-7]` | MEM | A growing `Arena` that reallocated one buffer would move every outstanding view `[ARN-1]` promised. | Growth takes a new chunk and never moves what was given out (N58). | M10 |
| F8 | `[CLS-2]` | MEM | A `virtual` method called from `init` was judged by "that body"; a base `init` cannot know which kind it builds, so an override that lets `this` out or reads an unset field was missed. | Every override's body the program has (`[DSP-5]`) (N59). | M11 |
| F9 | `[CTL-7]` | GAP/MEM | Nothing said what a hand-over below `defer: println(xs)` meant. | A `defer` block's uses are uses at every exit: a hand-over below the line is `E3040`, a view ended before the exit `E3060` (N60). | M3/M5 |
| F10 | `[RC-4]` | GAP/MEM | Point 2 queued a value for a making thread that might be ending at that moment; the value would be ended by nobody (`[OWN-2]`). | A thread's end empties and closes its queue in one atomic step; a subtract that finds the queue closed ends the value itself (N61). | M9 (runtime) |
| F11 | `[THR-20]` | GAP/MEM | `[STA-1]` let the main thread's scoped tasks reach a `static mut`, and no rule said two tasks may not both change it. | A `static mut` the tasks of a scope reach is a place `[THR-11]` checks (`E7007`) (N62). | M9 |
| F12 | `[SIMD-7]` | MEM | "not observable" was false for a relaxed `Atomic` store in a grouped loop (not `Sync`): another thread could read the wrapped value before the abort. | No operation on an `Atomic` in a grouped loop (N63). | M15/opt |
| F13 | `[ECS-8]` | MEM | A listener that reshapes a pool could run after `emplace`'s `ref mut T` was made. | Listeners run before the method's result view is made; a listener keeps its `ref mut Registry` no longer than its call (N64). | M14 |
| F14 | `[FFI-10]` | MEM | One shared static `T` sentinel for every empty span: a C function breaking its count contract corrupts a shared object, a race between threads. | Read-only memory for a `Span[T]`; a slot of the call's own frame for a `MutSpan[T]` (N65). | M12 |
| F15 | `[CT-2]`, `[MAN-8]` | SEC | `comptime.read_file` "only inside the package directory" was escaped by a symbolic link; `comptime.env` let a dependency's compile-time code read the builder's environment into the program it builds. | Links are followed before the root check; a dependency reads only the variables the root package's manifest lists (`[comptime] env`, default none) (N66). | M15 (evaluator); `comptime.env("HOME")` in a dependency is `E6010` |
| F16 | `[BRW-16]` | GAP | "found again at each use" did not say with which index. | The index is read once, where the name is made, and kept with it (N67). | M5 |
| F17 | `[OWN-8]` | GAP | "a copy of its target" without its type. | A `T` with one owner (N68). | M4 |
| F18 | `[CLS-12]`, `[EFF-27]`, attribute table | GAP/CON | A kept result is hidden state, which `[PHIL-9]`/`[TYP-11]` deny a struct; a change by foreign code or through a raw pointer clears nothing. | Class methods only (`E2117` on a struct's); the unseen change is unsafe code's obligation (N69). | M11 |
| F19 | `[ALC-1]` | GAP | An instance held inline beside values made in it is `[LT-25]`'s shape and moves with the struct. | Held in a block of its own, or the values made in an instance the struct does not hold (N70). | M10 |
| F20 | `[CT-5]` | GAP | "a read-only view of it" without its type. | `Span[T]`, `StrView` or `ref T`, `static` region (N71). | M15 |
| F21 | `[THR-19]` | GAP | Only "a lock of the kind this thread holds", with a `Log`'s lock as the example. | `Mutex`, `Lock`, `RwLock` (`write()` under `read()` and the reverse) (N72). | M9 |
| F22 | `[TYP-13]` | GAP | The niche list left out every pointer-shaped word but three. | The shared owners, the links, `ref mut`, `MutSpan` (N73). | M4 (layout) |
| F23 | `[FN-3]` | CON | Said a method's result points into `this` only; `[LT-1]` since verdicts 137/138 says into `this` and its source parameters, read from the code. | Aligned with `[LT-1]` (N74). | none |
| F24 | §IV.5 | CON | `ref_to_ptr(r: T)` and `ref_to_ptr(ref mut r: T)` were two functions of one name, which `[TYP-26]` forbids; the compiler has one built-in (`ember_typeck`, "requires a shared or mutable reference"). | One built-in whose result follows the argument's word (N74). | none |
| F25 | `[TYP-15a]` | CON | `BorrowList[T]` was named and declared nowhere (the research note says unimplemented). | Removed (N74). | none |
| F26 | `[TYP-47]`, `[STD-9]` | ED | A literal line break inside `end: StrView = "..."`. | `"\n"` (N74). | none |
| F27 | §II.3, §XVII.2, `[CLI-20]`, `[VER-9]` | ED/CON | The directive and manifest examples named old versions; the migrator's duty for 0.9.20 unsaid. | The current version; the migrator writes the directive and rewrites nothing else (N74, N75). | `fmt --migrate` |

## 3. Findings recorded and NOT fixed: the second pass starts here

Each has the analysis the first pass reached and a proposed shape; none is in the text yet. O1 and O2
need the owner where marked (an owner verdict or a new word); the rest are delegated rulings a second
pass can make and cut as Hardened_2.

* **O1. Whole-program facts that decide a library's layout or code (MEM, the largest open item).**
  Three rules let a fact of the *whole program* decide what a *package* compiles, which separate
  compilation (`[BLD-1]`, `[MONO-1]`, `[EFF-4]`) cannot honour: (a) `[OBJ-1]`: an object has the 12-byte
  header (weak count) only when "some `weakref` link in the program may name an object" of its class
  family; a library compiled with 8-byte headers for its `pub` class `X`, then a dependant writing
  `weakref x`, is a layout mismatch and memory corruption. (b) `[THR-2]`: handing an object to another
  thread tests the weak count of "every object the handed object owns whose class a `weakref` is made to
  anywhere in the program"; a library's hand-over compiled without the compare for `X`, then a dependant's
  link to an `X` the handed object owns, leaves a one-thread link into another thread's object. (c)
  `[WK-18]`: a walk is compiled in a deferred-drop region (no count moves) when the node's objects are
  ones "no other thread can end (not `thread_shared`, and not reachable from a `thread_shared` value)";
  a dependant may make a `pub` class's object `thread_shared`. F3 (`[LT-48]`) was the same shape and was
  fixed by moving the fact into the type's table. Proposed general rule: *a per-program fact never
  decides a package's code or layout; the deciding fact is one the package can see*: for (a) and (b), a
  class family carries the weak count when the class is `pub` or some `weakref` in its own package names
  it (a private class is reachable from nowhere else), and (b)'s compare is over the classes that carry
  one; for (c), the region needs a class whose objects cannot be `thread_shared` at all, which the type
  can show (see O2: a class with a `weakref` field to a one-thread target cannot be made `thread_shared`,
  `[THR-8]`), or the region is dropped from a `pub` class's walk. Verdict 101's design (`[RC-4]`) is the
  owner's: ask before changing a header rule. Also check `[DSP-5]` (devirtualisation with the whole
  program visible: an optimisation, harmless), `[WK-17]`'s proof (already conditioned on "no other package
  can change it", `[EFF-4]`), `[CLS-12]`'s clearing stores (placed by the package that compiles the
  changing line, which sees the class's `@cached` methods in the interface: fine).
* **O2. Does `weakref T`'s type say whether its target is `thread_shared`? (GAP/MEM, needs the owner:
  it is a word.)** Opening a link compiles two ways (`[WK-12]`: a header test for a one-thread target, an
  atomic count held for the view's life for a `thread_shared` one, `[RC-5]` point 2), `[THR-8]` and
  `[THR-9]` speak of "a `weakref` link to a value that is not `thread_shared`" as a property a type has,
  and a scoped task may open a link "only for a `thread_shared` target" (`[THR-2]`). So the compiler must
  know the target's kind from the link's type, and no rule says how it is written. The grammar already
  parses `weakref thread_shared T` (`base_type := "weakref" base_type`, and `thread_shared T` is a
  `base_type`), and also `weakref shares T`, which nothing means. Proposed: `weakref T` links a one-thread
  target (a class object with one owner or `shares` owners); `weakref thread_shared T` links a
  `thread_shared` value, a different type, written so in a field (`parent: weakref thread_shared Node`
  inside a class whose objects are made `thread_shared`), and `weakref shares T` is `E0100` with the help
  `weakref T`. Then O1(c)'s condition can be read from types. A consequence to check with the owner: a
  class `Node` with `parent: weakref Node` cannot have `thread_shared` objects (its field is not `Send`,
  `[THR-8]`), which is right (its tree is one thread's), and the thread-shared tree is a class of its own
  with `weakref thread_shared` links.
* **O3. Channels and `Send` (MEM, delegated).** `[THR-15]` makes `Sender[T]` `Send` and `ThreadShared`
  and `Receiver[T]` `Send` unconditionally, and never asks `T: Send`; a `shares` value sent through a
  channel reaches another thread (`[THR-2]` says "by any route", `E7015`, but names channels nowhere).
  Proposed: `Sender[T]` and `Receiver[T]` are `Send`, and `Sender[T]` is `ThreadShared`, exactly when `T`
  is `Send` (Rust's rule); `tx.send(owns v)` of a `v` that is not `Send` is `E7015` where the channel's
  ends are not on one thread, which the same marker decides. The marker table of §IV.8 and `[THR-9]`'s
  list ("a channel's `Sender` are") change with it.
* **O4. A hand-over out of an `owns T` block (GAP, delegated).** `[TYP-14]`/`[TYP-5]` rule 7 read an
  `owns T` through "wherever a `T` is wanted", including a by-value site (`x: T = b` for `b: owns T`);
  `[EXP-6]` lists what cannot be handed out on its own (a field of a type with `drop`, an element, a class
  field, anything reached through a non-owning name) and not the value inside an `owns` block, which the
  block owns; moving it out leaves a block holding nothing. Proposed: a hand-over of the `T` out of an
  `owns T` place is `E3013` ("`b` holds its mesh in a block of its own; hand the block over, `m = b`, or
  take the value out and leave one: `mem.take(ref mut b)`"), as a class field is; read-through stays a
  view.
* **O5. `[JOB-5]` `jobs.local_arena()` (GAP, delegated).** "an arena view for the current job, reset when
  the job ends; allocations from it cannot escape the job" names no mechanism. Proposed: it gives a
  `ref mut ThreadArena` whose region is the job's function (a local of it), so a value made in it that
  leaves the function is `E3063` by `[LT-4]`, and a call of it outside a job is a panic naming it.
* **O6. `[MAN-9]` writes `ember.toml` into any folder a compile command runs in** (owner's verdicts 127
  and 135: an OPEN question, not a ruling). It never overwrites, but `ember check /tmp/x.em` leaves a
  file behind; the handoff records four strays left in `docs/` and the fixtures. Ask whether a single
  file with no manifest should instead be compiled as a package in memory, or the note printed and the
  file written only by `ember new`.
* **O7. Owner questions N38 to N40** (`runtime` before a class type; `runtime` for a local whose kind is
  chosen while running; whether `owns` at a call covers the whole argument): unchanged and open.
* **O8. `[RC-4]`'s "which thread made a value is read from the header of the memory chunk its block came
  from"** (GAP, runtime): say a chunk belongs to its thread for the process's life (never given to
  another thread's allocator), or the record moves with it; otherwise a block freed and the chunk reused
  by another thread names the wrong making thread. Check `runtime/ember_rt` for what it does.
* **O9. Hash flooding** (SEC, by design): `[HASH-2]`'s `DefaultHasher` is fixed-seed for determinism, and
  a map keyed by untrusted input is told to use `RandomState`. Nothing to fix; a second pass may want
  `[STD-11]` to say it beside `Map` (one sentence) and a lint (`L4004`?) for a `Map[String, …]` filled
  from `io`/`fs`/`args` with the default hasher.
* **O10. `[FFI-53]` on 32-bit targets** the tagged number may be a user-space address (acknowledged in
  the text); nothing to fix, noted for completeness.
* **O11. `[THR-14]` `fetch_add` stores the overflowed value before the abort** (acknowledged;
  `checked_fetch_add` exists); nothing to fix.
* **O12. The inventory items not re-verified one by one** (§1): C03, C08, C09, C10, C11, C12, C14, C15,
  C17, C18, C33, C34. Each names its rule; a second pass reads the rule against the fix.
* **O13. Examples:** 15 of 56 `ember` blocks do not compile with today's compiler (unbuilt syntax: value
  words before a value, `owns fn(...)`); `check_examples.py` lists them. They are not text defects.
* **O14. Annex A** is generated from `docs/spec-source/appendix-a.em` (`spec_check.py --emit-appendix`);
  it was not changed and still shows `second = shares mut other` on a local, which F6 keeps legal.

## 4. What was read and judged sound (so a second pass need not redo it)

Listed per Part with the specific points examined; "sound" means no hole or contradiction was found under
the four lenses of §1, not that every sentence was proved.

* **Part I.** `[TIER-1]`'s list of boundaries matches `[UNS-11]`/`[UNS-12]`/`[FFI-51]`/`[FFI-26]`/`[TYP-43]`;
  `[PHIL-2]`'s allocating forms match Part VII–IX; `[PHIL-8]`'s run-time checks match `[PRF-4]` and
  `[COST-3]`; the I.4 table's `[WK-17]` reference exists and says what the table says.
* **Part II.** The 52 keywords counted; `owned`/`self` reserved as `E1010`, consistent with `[LEX-14]`'s
  `E0005` for table keywords; `[LEX-23]` safety notes consistent with `[UNS-7]`/`[UNS-8]`.
* **Part III.** `fn_header`'s order matches `[GRM-41]`; `param := [param_word] IDENT ":" ["runtime"] type`;
  capture words exclude `sharedref mut` (consistent with `[GRM-42]`); `value_word` binds as `-`
  (N40/N42, open with the owner); `[GRM-44]`/`[GRM-45]`/`[GRM-19]` consistent with `[CTL-11]`,
  `[DSP-4]`, `[TYP-44]`; `[GRM-8a]`'s commit tokens cover every type start.
* **Part IV.** The IV.1 table's rows per word; `[TYP-5]`'s twelve coercions (rule 12 the owner's);
  `[TYP-6]`/`[TYP-7]`/`[TYP-8]`/`[TYP-10]`/`[TYP-30]` (`MIN // -1` under each policy); range types
  `[RNG-1]`–`[RNG-10b]`; `[TYP-14]`'s `ref mut` re-pointing versus write-through (`r = ref mut b` says
  `ref mut` again, N55 adds the hand-over side); `[TYP-15]` kept views and `from`; `[TYP-34]` view
  types; `[TYP-45]`/`[TYP-46]` (`using` after a type; a value keeps its instance word; one-instance
  managers keep none); `[TYP-17]`/`[IFC-6]` bounds (`pub` writes all, else all or none); `[TYP-19]`
  overlap; `[TYP-47]` packs; the marker table (F1 added `sharedref mut`); `[TYP-36]` (`copy of` through
  links, F17 added the type); `[TYP-39]`; `[HASH-1]`–`[HASH-4]` (a `hash` that changes its map is
  `E3031`); `[TYP-22]`/`[IFC-5]` `dyn` compatibility; `[TYP-44]` kind tests and casts; `[TYP-23]`
  inference; IV.11 resolution (`weakref` is not stripped; `sharedref mut` is).
* **Part V.** `[FN-1]`'s three words and the `Mutex`/`RwLock`/`Atomic`/channel/`guarded by` exception;
  `[FN-2a]` call words (verdicts 145–160); `[FN-6]` callable types; `[FN-8]` `main` and scripts;
  `[STR-1]`–`[STR-7]`; `[CLS-1]`–`[CLS-11]` (two-phase construction: derived fields before `super.init`,
  F8 covers the virtual case); `[CLS-7a]`/`[OBJ-5]` `drop` and `this`; `[CLS-9]`/`[CLS-9a]` `let`
  fields; `[STA-1]`/`[STA-3]` (first-access initialisation under concurrency; a `static mut` built on
  first access, C17); the attribute table.
* **Part VI.** `[EXP-4]` temporaries (conditions, `with`, `for`); `[EXP-5]`/`[EXP-6]` places and
  hand-overs (O4 is the one gap); `[EXP-9]`/`[EXP-10]`; `[TYP-38]` literals; `[CTL-1]`–`[CTL-11]`
  (a counting loop may add, not remove); `[CLO-1]`–`[CLO-19]` (real variables, capture words,
  `[CLO-4]`'s outliving check by the maker's view of every place the value goes, `[CLO-5]` loops through
  state, `[CLO-6]` `once fn`, `[CLO-10]` two words and the copy-out before a call, `[CLO-16]`
  `thread_shared fn()`, `[CLO-17]` extended by F2); generators `[CORO-1]`–`[CORO-14]` (`[CORO-6]` views
  into the frame, `[CORO-14]` a generator as a walk, `[CORO-13]` `@must_drop` across `yield`);
  `[PAN-1]`–`[PAN-3]` (abort, no unwinding).
* **Part VII.** The terms (F4 adds hand-over to "reshape"); `[OWN-1]`–`[OWN-13]`; `[BRW-12]`–`[BRW-15]`
  (names for parts, `=` on a name, names that keep their place, a plain name never changes); `[BRW-1]`
  (one changes or any number read; a `ref mut` name alone; a read-only promise; a reshape ends an
  address); `[BRW-2]` liveness; `[BRW-3]`/`[BRW-11]`/`[BCK-5]` arguments (two-phase: reserved then
  activated; exception 1 compiled against a copy, `[MONO-10]`; exception 2 the lock); `[BRW-21]` the call
  check by the callee's summary (`E3031`), with noalias widening the check for `ref mut`; `[BRW-22]`;
  `[BRW-4]`/`[BRW-5]` fields and indices; `[BRW-6]`–`[BRW-9]`; `[BRW-16]` positions (F16 adds the index),
  `[BRW-17]` addresses and when two paths may be one, `[BRW-18]` walks (including a call through a stored
  function, a spawn, a foreign call, a `drop`), `[BRW-19]` temporaries, `[BRW-20]` guards; `[LT-1]` (the
  three rules, the summary for every result type, `dyn`/`virtual` per kind, `runtime`), `[LT-1a]`
  `from`, `[LT-3]`/`[LT-4]`/`[LT-44]`; `[LT-7]` function types; `[LT-45]`/`[LT-46]`/`[LT-47]`; the region
  vector rules `[LT-14]`–`[LT-43]`; `[SPN-1]`–`[SPN-8]`; `[DRP-1]`–`[DRP-8]` (a `drop` counted as a call
  wherever a value may end).
* **Part VIII.** `[OBJ-1]` headers (O1 for the per-program decision), `[OBJ-2]`–`[OBJ-5]`; `[RC-1]`
  limits (one compare before each +1), `[RC-2a]`–`[RC-2e]` no-count cases, `[RC-3]` removals, `[RC-4]`
  biased counting (home/away, the queue, check points, hand-over at once; F10 adds the thread's end;
  the limit at half the range for racing adders), `[RC-5]` views through links (point 1 F5; point 2 the
  held count for a `thread_shared` target, the open-then-undo on another thread, the same-step ended
  mark), `[RC-6]`; `[EXC-1]`–`[EXC-3]`, `[EXC-15]`–`[EXC-19]` (`this` never noalias); `[DSP-1]`–`[DSP-6]`;
  `[WK-1]`–`[WK-18]` (`[WK-11]` link counts; `[WK-12]` opening; `[WK-3]` ended before `drop`; `[WK-17]`'s
  two conditions for a test-free parent read, conditioned on no other package changing it; `[WK-18]`'s
  walk with a link, the deferred-drop region: O1(c) for its whole-program condition; `[WK-5]`–`[WK-7]`
  the graph of owners and the two proofs that a store closes no loop); `[OPT-1]`.
* **Part IX.** IX.0's table; `[HEAP-1]`–`[HEAP-10]`; arenas `[ARN-1]`–`[ARN-11]` (F7 for growth;
  `MaybeUninit` unsafe for drop types, C20; `Zeroable` as an unsafe marker); `[ALC-1]`–`[ALC-5]`
  (`Layout` checked; `Allocator` unsafe to implement; `deallocate`/`reallocate` unsafe to call, C01;
  `FoundByAddress`'s piece promise; an instance changed only through its methods while values live);
  `[UNS-1]`–`[UNS-12]` (the list of unsafe operations matches Parts VII–XVI; `safe:` blocks; `@safety`
  on every `unsafe fn` and on lambdas in unsafe code); `[LAY-2]`; `[HND-1]`–`[HND-3]` (retired slots,
  C30); `[CELL-1]`–`[CELL-13]`; `[DSJ-1]`–`[DSJ-10]` (the one run-time alias check; making a span reads
  no element).
* **Part X.** The effect table; `[EFF-1]`–`[EFF-4]` (summaries, function values by every function the
  program puts in the place, interface files); `[EFF-9]`–`[EFF-11]`; `[EFF-24]`–`[EFF-27]` (F18 for
  `@cached`); contracts `[EFF-5]`–`[EFF-23]` (`[EFF-15]`'s verdict as if every proof applied);
  determinism `[DET-1]`–`[DET-10]`; the cost table (two rows changed, F2/F3) and `[COST-6]`'s stop texts.
* **Part XI.** `[THR-8]`/`[THR-9]` (F1; O2/O3 open); `[THR-1]` the three rules of `thread_shared`;
  `[THR-2]` (F4; O1(b) open); `[THR-7]`; `[THR-13]` the race guarantee's seven kinds of place;
  `[THR-10]`/`[THR-17]` unscoped threads and the loop's variable; `[THR-3]` locks and guards (no
  poisoning; a guard thread-bound); `[THR-18]` `guarded by` (a call entering past a held lock only when
  proved the same); `[THR-19]` (F21); `[THR-20]` (F11; a `drop` on another thread); `[THR-14]` atomics
  (O11); `[THR-15]` channels (O3 open; the receiver ends queued values, C21); `[THR-6]` `@must_drop`
  structural; `[THR-5]`/`[THR-11]`/`[THR-21]` scoped tasks and reductions (verdicts 104/105; a task's
  own total overflow stops); `[THR-12]`; `[PAR-1]`–`[PAR-5]`; `[JOB-1]`–`[JOB-5]` (O5 open).
* **Part XII.** `[SOA-1]`–`[SOA-7]` (proxies are positions; a length change reshapes all columns);
  `[SIMD-1]`–`[SIMD-9]` (F12; `[SIMD-3]`'s derived alias facts, `this` never; `[SIMD-9]` store says
  `ref mut`); `[ECS-1]`–`[ECS-9]` (F13; pools as one array, no stable address needed).
* **Part XIII.** `[ERR-1]`–`[ERR-13]`; `AnyError` not `Error` (so `From` is not doubled); `?` and
  `context`; the panic-or-error convention.
* **Part XIV.** `[CT-1]`–`[CT-7]` (F15; the evaluator has no `Nondet`, no I/O, no FFI: a dependency's
  compile-time code is sandboxed, O9's lens); `[CT-5]` (F20); reflection and derives; `std.ser`.
* **Part XV.** `[STD-6]` layers; `[STD-26]` Python built-ins; `[STD-9]`/`[STD-18]` printing (F26);
  text `[TXT-1]`–`[TXT-11]` (nothing becomes a `StrView` without validation; slicing panics on a
  boundary); collections `[STD-15]`–`[STD-17]`, `[STD-11]`/`[STD-12]` (lookups by a view, `to_key` only
  for a new key); `[STD-7]`, `[STD-8]`; iterators `[STD-19]` (adapters own what they wrap; functions as
  type parameters; the known-numbers rule and the count types, the owner's G8-4); `[STD-5]`; numbers
  `[STD-20]`–`[STD-28]`; I/O `[STD-22]`–`[STD-25]`.
* **Part XVI.** `[FFI-1]`/`[FFI-2]`/`[FFI-2a]` (safe-to-call never derived; `safe fn`/`unsafe fn`,
  verdicts 130–136); `[FFI-10]` hand declarations (F14; count witnesses, nullable carriers, result
  counts); `[FFI-8]` the type mapping (`char` never crosses, C28; `va_list`; variadics in `unsafe`);
  `[FFI-6]`–`[FFI-7]`, `[FFI-51]` (C compiled in is `unsafe import c`, C25), `[FFI-29]`–`[FFI-30c]`;
  overlays `[GRM-35]`, `[FFI-11]`–`[FFI-13]`, `[FFI-55]` (`ember.overlays` fingerprints, C26: a
  dependency cannot lie about the builder's header), `[FFI-14]`; `[FFI-35a]`, `[FFI-15]`, `[FFI-16]`,
  `[FFI-36a]`/`[FFI-36b]` (one destructor per foreign type; adopting twice caught); callbacks
  `[FFI-21]`/`[FFI-21a]`/`[FFI-22]`/`[FFI-52]`/`[FFI-53]` (the checked trampoline: tagged numbers never
  addresses, generation and type in one compare, thread checks in every profile, retired slots, a
  callback ending its own token; O10), `[FFI-23]`, `[FFI-54]` (`into_raw()` only in `unsafe`),
  `[FFI-56]`, `[FFI-33]`–`[FFI-33c]`; exports `[FFI-26]` (`@unsafe(export)`, known-taken names refused,
  C12), `[FFI-25]`, `[FFI-31b]`; `[FFI-27]`–`[FFI-31a]`; `[BLD-FFI-1]`–`[BLD-FFI-3]`.
* **Part XVII.** The command listing (tested against `--help`); `[CLI-20]` migration (F27); the
  manifest (`[MAN-8]` F15; `[MAN-9]` O6); builds `[BLD-1]`–`[BLD-15]` (`[BLD-14]`'s seven recorded facts);
  profiles `[PRF-1]`–`[PRF-4]` (no safety check differs); tests and gates `[TST-*]`, `[BEN-*]`;
  diagnostics `[DIA-*]` and the shape tables (B1 touched by F2); formatter, linter, docs, editor;
  `[CONF-*]`, `[ABI-*]`; the code registry (XVII.9: every code the text names is registered; no code was
  added by the first pass, so `codes.rs`'s test still holds).
* **Part XVIII.** `[CG-C-1]`–`[CG-C-11]`; `[MNG-1]`–`[MNG-6]`; `[MONO-1]`–`[MONO-10]`; `[BCK-1]`–`[BCK-8]`
  (`[BCK-4]` already treated a handed-over owner as ending its views: F4 made the language parts say it);
  the runtime `[RT-1]`–`[RT-12]` (counts inline; the atomic orders of `[RT-8]`; stack probes);
  `[IMP-11]`; the LLVM backend.
* **Annex A.** Generated; unchanged (O14). **Annex B.** `[HR-1]`–`[HR-47]`: the transaction, `[HR-44]`'s
  recorded facts, `[HR-3b]`'s frames below a checkpoint, `[HR-15]`/`[HR-15a]` rewriting what can be
  enumerated, `[HR-45]`–`[HR-47]`. **Annex C.** `[FFI-3]`, `[FFI-17]`–`[FFI-17f]`, `[FFI-24]`–`[FFI-24d]`,
  `[FFI-39]`–`[FFI-39e]`, `[FFI-40]`/`[FFI-40a]` (verdict 132), `[FFI-41]`, `[FFI-42a]`, `[TCB-1]`–`[TCB-6]`,
  `[FFI-37]`–`[FFI-37d]`, `[CXX-1]`; `[BLD-FFI-5a]`'s C++ handle counting on the away count (the biased
  scheme tolerates a making-thread copy counted away). **Annex D.** `[GPU-1]`–`[GPU-12]` (draws checked,
  C27). **Appendices E, F.** Non-normative; the glossary rows for "reshape" and `sharedref mut` changed
  with F4 and F1.

## 5. A plan for the second pass (a workflow, if the owner allows one)

The first pass was one reader with four lenses. The second pass should be adversarial and divided, each
finder blind to the others, and every finding refuted by two verifiers before it is fixed (the quality
patterns of the workflow reference):

1. **Finders, one per lens, over the whole document:** (a) *separate compilation*: every rule that uses
   "the program", "anywhere in the program", "every function the program can put there" and decides a
   layout, a code shape or a check at the package's compile time (O1 is the first such list);
   (b) *run-time races in the runtime's own mechanisms*: biased counting (`[RC-4]`, `[RT-8]`, `[RT-10]`),
   the trampoline (`[FFI-53]`), the queue (F10), the deferred-drop region (`[WK-18]`), `[STA-3]`'s
   first-access initialisation, hot reload's commit (`[HR-42]`); (c) *the FFI contracts*: each contract
   word's safe type against what C may do within the contract (`[FFI-10]`, `[FFI-11]`, `[FFI-21]`,
   `[FFI-54]`, `[FFI-56]`, Annex C's `[FFI-39c]`, `[FFI-40a]`); (d) *what an attacker controls*:
   dependencies' overlays, `unsafe import c`, compile-time code (F15), `@unsafe(export)`, `ember.overlays`,
   the lock file, `comptime.read_file`, hashing of untrusted keys (O9); (e) *the summary and call checks*
   (`[BRW-21]`, `[EFF-24]`–`[EFF-26]`, `[BCK-8]`): a program that a summary cannot capture (recursion
   through function values, a `dyn` method whose implementations the package never sees, a C callback
   kept across a package edge); (f) *words and types*: every place a word is written (`[GRM-42]`,
   `[GRM-43]`, `[OWN-13]`, `[FN-2a]`) against every place it is read (O2 is one); (g) *the inventory*
   C01–C34 against the text (O12); (h) *consistency*: every code against its registry row and shape
   (`[DIA-7a]`), every cross-reference's claim against the rule it cites.
2. **Verifiers:** for each finding, two independent refuters with the rule text and a program; a finding
   survives only when neither refutes it; a surviving finding gets a proposed rule in the document's own
   voice (no "you", the line to write, `[DIA-25]`), the compiler milestone and the test.
3. **The owner's questions first:** O1, O2, O6 and N38–N40 are put to the owner as language ODRs, one at
   a time with real code (`JOB.md`'s way), before a workflow fixes anything that depends on them.
4. **Then Hardened_2:** the delegated items (O3, O4, O5, O8, and what the finders confirm) cut with the
   recipe of §0, each change a row of §H.52 with its reason.

## 6. The first pass's raw notes (verbatim, from the session's scratchpad)

Kept as written during the reading, before the fixes were decided; the ids A01–A41 map to §2 and §3
above (A02, A03, A05, A18, A25, A28, A38, A41 were checked and found to need nothing).

```text
- A01 | p02 II.3 | text block | CON | `#! language "0.9.9"` example names an old version; make it the current one.
- A02 | p01 I.4 table | - | note | table says "except a parent-link read proved ([WK-17])": check WK-17 exists and says it. (it does)
- A03 | p03 | grammar `capture_word` | CON? | capture words exclude `sharedref mut`; GRM-42 agrees. (fine)
- A04 | p03 | fn_header `["->" type ["from" IDENT]]` | GAP? | LT-1a allows one parameter or one field: the grammar's one IDENT matches. (fine)
- A06 | p04 IV.5 | ref_to_ptr | CON | two functions of one name: overloading is a non-goal. (F24)
- A07 | p04 TYP-47, p15 STD-9 | - | ED | a literal line break inside the string. (F26)
- A08 | p04 marker table | Send/ThreadShared | MEM | `sharedref mut T` in neither row. (F1)
- A09 | p04 TYP-13 | niche | GAP | niche list omits the shared owners, links, `ref mut`, `MutSpan`. (F22)
- A10 | p04 TYP-15a | BorrowList | CON | named, declared nowhere. (F25)
- A11 | p05 FN-3 | - | CON | "a method's into `this`" against LT-1 after verdict 138. (F23)
- A12 | p05 CLS-2 | virtual in init | GAP/MEM | every override's body. (F8)
- A13 | p05 STA-1 / p11 THR-20 | static mut and scoped tasks | MEM? | two tasks writing it. (F11)
- A14 | p04 TYP-36 / IV.6 / OWN-8 | copy of a shares value | GAP | the result's type. (F17)
- A15 | p05 CLS-12 / EFF-27 | @cached and foreign changes | GAP | unsafe code's obligation. (F18)
- A16 | p06 CTL-7 | defer and hand-over | GAP/MEM | (F9)
- A17 | p06 CLO-18/CLO-17/CLO-10 | a function value ending its own holder | MEM | (F2)
- A19 | p07 BRW-16 | positions | GAP/MEM | the index read once. (F16)
- A20 | p07 LT-48 (+ X.4 row, N49) | runtime check and separate compilation | MEM/GAP | (F3)
- A21 | p07 LT-46 | sharedref on two threads | CON | only scoped tasks. (F1)
- A22 | p08 RC-4 point 2/3 | queue vs the making thread's end | GAP | (F10)
- A23 | p08 RC-5 point 1 | "an object of that type" | MEM | class family, everything that owns it. (F5)
- A24 | p08 RC-1 / OWN-13 | `shares x` of a local | MEM/GAP | made in a block from its making line. (F6)
- A26 | p09 ARN-4/ARN-7 | growing Arena | MEM/GAP | growth never moves. (F7)
- A27 | p09 ALC-1 / LT-4 | an instance held inline beside its values | GAP/ED | (F19)
- A29 | p10 EFF-27 / p05 CLS-12 / ATT table | @cached on a struct | CON | class only. (F18)
- A30 | p10 COST-3 | rows to update | CON | the runtime row and the state row. (F2, F3)
- A31 | VII.1 reshape / OWN-3 / THR-2 | a hand-over as a reshape | MEM | (F4)
- A32 | THR-19 | RwLock | ED | (F21)
- A33 | SIMD-7 | grouped overflow checks and relaxed atomics | GAP | (F12)
- A34 | ECS-8 | signals and a method's result view | MEM/GAP | (F13)
- A35 | CT-2 | comptime.read_file and symlinks | SEC | (F15)
- A36 | CT-2 | comptime.env from a dependency | SEC | (F15)
- A37 | CT-5 | type of `comptime(e)` owning heap memory | GAP | (F20)
- A39 | p17 manifest example | version strings | CON | (F27)
- A40 | p16 FFI-10 | the static `T` sentinel for an empty non-null span | ED/MEM | (F14)
```


## 7. The second pass: 0.9.20_Hardened_2 (2026-11-03, a workflow of 29 agents, applied by the lead)

The owner's task: "the final mile ... make it as foolproof as possible ... remove all the gaps, flaws and
memory issues. Make the spec near 100% complete so that any agent can pick it and start implementing without
any disruptions. Also answer all the open questions in this pass with the best judgement."

* **The workflow** (`tasks/96/redesign/PLAN-spec-0.9.20.md` is the original oversized plan; what ran was its
  reduction to 29 agents, Fable 5.1 at high effort, two at a time on 4 CPUs, 5.0M tokens, 2.9 h): nine
  readers, one per group of Parts (G1 I–IV, G2 V–VI, G3 VII–VIII, G4 IX–X, G5 XI and XVII, G6 XII–XV, G7
  XVI and Annex C, G8 XVIII and Annexes B and D, G9 the whole document as an implementer), each finding
  written with the breaking program, the sentence that allows it and the rule text to write; nine
  refuters, one per group, each finding refuted, narrowed or confirmed with a better fix where they had one
  and the items the reader missed; for O1, O2 and N38–N40 three designers (speed first, safety first, the
  newcomer first) and three judges apiece, the judges scoring and grafting. Results in
  `tasks/96/redesign/audit-0.9.20/round1/` (`G1.json`–`G9.json`, `ruling-*.json`, `digest.txt`,
  `counts.json`, `applied.md`).
* **Found:** 163 findings (108 confirmed, 45 narrowed, 10 refuted) and 37 missed items; 3 judged rulings.
  **Fixed:** the 153 non-refuted findings, the 37 missed items, the 3 judged rulings and the 7 smaller
  questions (O3, O4, O5, O6, O8, O9, O10) ruled by the lead: 222 rules changed or added (the rows of §H.52;
  18 new rules, 4 new codes, 3 retired), the document 1,000 → 1,018 rules. Every open item of §3 is now
  ruled or closed: O1 N76, O2 N77, O3 N80, O4 N81, O5 N82, O6 N83 (reverses half of verdict 135, reported),
  O7 N79, O8 N84, O9 N85, O10 N86, O11 by design (`[THR-14]`: `checked_fetch_add` exists; the text says
  it), O12 read by the readers of round 1 (every Part was read), O13 the examples are unbuilt syntax
  (41 of 54 blocks compile, as before), O14 Annex A unchanged (`second = shares mut other` on a local
  stays legal under `[RC-1]`).
* **Applied how:** one asserting script per group (the session scratchpad; the scripts assert the text they
  replace, re-wrap the bullet at 100 columns and add the `*(changed in 0.9.20)*` marker), the refuter's
  better fix where it gave one and the reader's text otherwise, merged where two findings or a ruling
  covered one rule (G5-03 is covered by N77; G9-04 by G3-02's rule; G9-05 by widening `[MOD-9a]`). The
  registry (`tools/p17a_0915.py`) gained the rows each change needs and `compiler/ember_diag/src/codes.rs`
  the four new codes (`E0012`, `E0119`, `E1042`, `E2074`) and the re-cited ones (`E5031` → `[FFI-32]`,
  `E5034` → `[FFI-48]`, `E5058` → `[FFI-39b]`, `E5059` → `[FFI-17d]`); `E5051`, `E5057` and `E5066` are
  retired (named by no rule that can emit them).
* **Checked at the cut:** `check_ids` 0 problems (1,018 rules), `check_citations` 0 undefined,
  `code_registry --check` 0 problems (269 live, 36 retired), `split_inline` two citation flags (FFI-52,
  UNS-8: citations after a semicolon, not definitions), `rule_index --write-baseline --allow-growth` then
  green, `spec_check` no new failures, `phase_progress --write` then `--check` green, `check_examples` 41
  of 54 (the rest unbuilt syntax), `cargo test -p ember_diag -p ember_parser` green (the registry test
  reads the pinned target). **The full milestones suite (186 tests) on this container, through the C backend:** 183 passed, 1 ignored by design (`the_compilers_speed_over_bigpkg`, the by-hand benchmark), 1 failed for the container alone (`the_llvm_backend_writes_debugger_visualisers_beside_the_program`: this container's clang rejects the backend's `.ll` with "expected instruction opcode" at `#dbg_declare`, as the earlier cloud session recorded; it passes with a current clang), and `the_conformance_suite_runs`, which the two-hour background allowance cut off twice a few directories from its end, run as four shards (`EMBER_TEST_SHARD=i/4`): all four passed, 0 failures, 33 + 31 + 42 + 32 minutes. So no test that reads the specification or runs the conformance programs changed its verdict with Hardened_2.
* **To resume or to cut Hardened_3:** the recipe of §0 with the numbers moved by one (`CURRENT`,
  `V0920_H2_PARTS`, `H53_HEAD`, `v0920_h3()`); the exit checks the plan named (an implementer and an
  attacker over the changed Parts) were not run, for the owner's credits: a small pass may run them.

## 8. The third pass: 0.9.20_Hardened_3 (2026-11-08, a workflow of 44 agents and two exit checks, applied by the lead)

The owner's task: take the gap review of 0.9.15_Hardened_11 that another session pushed to `main` (its findings
and its spec, which this branch had never seen), combine it with Hardened_2, verify and fix the other agent's
findings, and cut a newer 0.9.20, "a complete, without holes spec that takes care of speed, performance,
readability and memory safety", with a thorough analysis by up to 50 agents first.

* **What was combined.** `main` was merged (commit `d81c99b`): 0.9.15_Hardened_11 (its §H.51 kept as the log of
  verdicts 161–163, which Hardened_2 already carried; its `E3028` is 0.9.20's `E0119`), verdicts 164–166 of
  `JOB.md`, and `tasks/96/redesign/spec-gaps-0.9.15/` (76 clusters K01–K76 of open questions, 47 new gaps NG-01–NG-47,
  four review methods' notes, the pending compiler fix D-966). Verdict 165 and the review's 35 lead-fix items were
  applied first (commit `dd99380`, `scratchpad/edit19.py`).
* **The analysis** (`tasks/96/redesign/audit-0.9.20/round2/`, the workflow script and journal under the session's
  `workflows/`): 10 readers decided each cluster and gap against the current text (22 answered, 30 answered
  differently from the finder, 66 not answered, 4 for the owner, 1 not a defect; 198 edits), 10 refuters checked
  each (66 confirmed, 54 narrowed with 135 better edits, 3 refuted) and listed 34 missed items (39 edits), 14
  designers and 9 judges ruled K05, K07, K10, K20 and K21 (161 final edits), a critic read the whole (19 missing
  points, 15 contradictions between results, 10 verdict clashes); then two exit checks, an implementer's and an
  attacker's, over the changed Parts. The critic's first point stands as a lesson: the readers read a snapshot
  (`inputs/CURRENT-SPEC.md`) 44 rules behind HEAD, so every anchor was re-checked against HEAD before it was
  applied (`scratchpad/anchor_check.py`: 489 of 533 matched; the rest were re-anchored or found already applied).
* **Applied** (`round2/applied.md` has every item's disposition; `scratchpad/edit20.py` to `edit30.py`, each
  asserting what it replaces; `registry3.py` the registry): 160 rules changed or added (the rows of §H.54), 2 new
  rules (`[FFI-57]` a C++ name declared several times, `[FFI-40b]` a `const`/non-`const` pair), 2 new codes
  (`E2113` an override's first line, `E5035` a use of a C++ name of several members), 33 registry rows
  rewritten (26 by the rulings, 3 by the implementer's check, 4 by the attacker's), none retired; the rulings are N89–N128 (`decisions.md` §14). Verdicts touched, none overridden:
  159 (a plain `init` parameter lends), 150 and 51 (as Hardened_2 already did), 5 (across a call), 101 (which
  values keep a home count, as N84), 164 (its letter restored), 165 (extended to a type and a derived class), 61
  (extended to an open class's header), 109 (extended to `dyn`/`some`), 68 (brought to its letter); ODR-199's
  shape changes (`std_function` is `unsafe fn(A…) -> R` until its entry says `safe`).
* **For the owner** (OPEN ODRs 260–266, all applied under the delegation and reversible): K05 `thread_shared …
  using` (shape B), K20 the open-class header promise and `Error: needs (Send, ThreadShared)`, K21 overloads and
  the `_mut` split, NG-10 the check point at a return to C, K64 a list pattern as an arm, K65 `dyn (A, B)`, K76 the
  stop on a full callback table; and still N83 and N87 from Hardened_2. Not applied, by the refuters' verdicts:
  NG-08 (`using` binds the same on both sides already), NG-28 (the C aliases stay aliases), NG-39 (safe code
  cannot forge a C function pointer); NG-46/47's wake of a waiting maker is left to a later K10 round.
* **Exit checks:** the implementer's read found 11 defects of the text, each applied (`scratchpad/edit31.py`: `[CLS-3]`, `[TYP-32]`, `[IFC-3]`, `[CLS-4]`, `[GRM-13]`, `[TYP-43]`, `[DET-2]`/`[DET-4]`, `[TYP-46]`, §X.4, `[STD-16]`, three registry rows); the attacker's read against `[PHIL-10]` found two safe programs the text accepted, both fixed (`scratchpad/edit32.py`): an object of a class with a `@cached` method that declares nothing saying its objects are shared could be made `thread_shared` or held by a `static`, a data race on the kept result, and is now not `ThreadShared` (`[THR-9]`, `[CLS-12]`, `E7002`, `E7015`); a `thread_shared` value made in a local arena could be handed to an unscoped thread that outlives the arena, and is now `E3063` unless the instance is a `static` (`[LT-4]`, `[ALC-5]`, `[TYP-46a]`); with them an ambiguity (`[FFI-42]`: a class's one non-`const` `operator[]` is `index_mut` alone, `E2072`, since `Index.index` would promise a read) and two wordings (`[TYP-22]`, `[OBJ-1]`), four registry rows; the full record is `round2/applied.md`.
* **Checked at the cut:** `check_ids` 0 problems (1,020 rules), `check_citations` 0 undefined, `code_registry
  --check` 0 problems (271 live, 36 retired), `split_inline` the one known flag (UNS-8; the FFI-52 flag went with
  the example's rewrite), `rule_index` (baseline rewritten with growth), `spec_check`, `phase_progress --check`,
  `check_examples` (see the handoff for the counts); `ember_diag` (the registry test reads the pinned target) and
  `ember_parser` green after the pin; the three spec-tied milestone tests green. The full milestones suite was
  not re-run for this pass (nothing of the compiler changed but `codes.rs`'s two new entries; Hardened_2's run
  stands).
* **Known debts the text now states and the implementation does not have:** `std/src/math/det.em` declares `log`
  where `[DET-4]` now names `ln` (a defect of the file by Part XV's preamble, with the other `det` twins `[DET-4]`
  lists); the conformance tests the rulings name (the K10 ruling's run-pass and break tests, K07's must-refuse
  programs, `tests/conformance/`) are not written, since the compiler builds none of 0.9.20 yet; the compiler
  defects D-966–D-968 stay as `main` left them (D-966's fix in `spec-gaps-0.9.15/pending/`, not applied: compiler
  work was not asked for).
* **To resume:** the recipe of §0 ("To cut 0.9.20_Hardened_4"); `round2/results/critic.md` lists what no result
  took (missing 5's 33 missed items are applied; missing 15 is the exit checks, run; missing 18 the tests above).

## 9. The fourth pass: 0.9.20_Hardened_4 (2026-11-08, the open ODRs ruled for the best solution, by the lead alone)

The owner's instruction: "for the ODRs can you go for the best solution possible? that matches all the goals we are
trying to achieve?", then "go solo no agents" (a design workflow had been started and was stopped at that word; two
designer results for ODR-260 had finished and are kept in `tasks/96/redesign/audit-0.9.20/round3/results/`). Every
open owner-decision record was weighed against the four goals and the owner's verdicts and closed; the record of what
was weighed, shape by shape, is `tasks/96/redesign/audit-0.9.20/round3/rulings.md`, the rulings N129 to N139
(`decisions.md` §15), the queue entries ODR-260 to ODR-267 and ODR-238 (`docs/OWNER-QUEUE.md`, all CLOSED).

* **Stand as Hardened_3 applied them:** ODR-262 (one Ember name of several C++ members, `at`/`at_mut`, transparent C
  aliases), ODR-263 (a check point at every return into C), ODR-265 (`dyn (A, B)`, `some (A, B)`), ODR-238 (answered
  by `[CLO-4]` and `[CLO-7]`: a kept lambda's captures are checked at its maker through the callee's summary), ODR-266
  (the stop on a full callback table, now named as the failure `Panic(Explicit)` lists, on every target), N83 and N87
  (with one fact stated: a kind test reads the type index with the kept-by-C mark cleared, `[OBJ-1]`, `[TYP-44a]`).
* **ODR-260 stands** (shape B of `thread_shared T using A`) with its text made exact: growth and end count for
  `[THR-11]` as the calls they make; the hand-over to an unscoped thread, a job outside a scope or a kept callback is
  from a `static` instance only (a `static mut` may be reset by a line no package can see); why the queue place is
  free; `using x` through any name where `allocate` reads the instance; the §X.4 rows price the mark and the
  give-back; and an adjacent hole closed: `into_raw()` on a value made `using` another manager is `E2114` (`[FFI-54]`).
* **Two change the language.** ODR-261: `interface Error: needs (Debug, Display):` (verdict 110's line restored),
  `AnyError` holds `owns dyn (Error, Send, ThreadShared)` and its `From[E]` needs the markers, so a one-thread error
  is refused at its conversion into `AnyError` (`E2040`), never at `implements Error`; the open-class header promise
  stands. ODR-264: a list pattern may open a `match` arm (`["quit"]:`), every other `[` line staying `E0100`;
  **departs from verdict 155** in a case it did not consider, reported. New ODR-267: a `@ffi(handle)` value read out
  of an `owns T` is a view of the owner, `E3063` where kept beyond it (`[FFI-8]`).
* **Applied** by `scratchpad/edit33.py`, `edit34.py` and `registry4.py`, each asserting what it replaces: 19 rules
  changed (§H.55), registry rows E0100, E2040, E2080, E2114, E3063, E4040; no new rule, no new code, none retired.
* **Checked at the cut:** `check_ids` 0 problems (1,020 rules), `check_citations` 0 undefined, `code_registry
  --check` 0 problems (271 live, 36 retired), `split_inline` UNS-8 only, `rule_index`, `spec_check`, `phase_progress
  --check`, `check_examples` 41 of 54 as before; `ember_diag`, `ember_parser` and the three spec-tied milestone tests
  green against the new pin (SHA-256 `1CA5A9336C7C…`). The compiler is untouched.
* **For the owner:** the departure from verdict 155 (ODR-264) and the restoration of verdict 110 (ODR-261). No ODR
  is open: the designers' doubt under ODR-260 (a bound cannot say that a manager's `deallocate` reads `this`) is
  decided by `[TYP-17]`, which makes a `thread_shared T using a` line in a generic body a refusal at the definition.
* **To resume:** the recipe of §0 ("To cut 0.9.20_Hardened_5").

## 10. The fifth pass: 0.9.20_Hardened_5 (2026-11-10, the local session, attended, the lead solo)

The owner's task: pull the latest, and check the 0.9.20 spec "for gaps, memory safety flaws and implementation
discrepencies that can affect readability"; then "Yes, read the rest"; then "Yes, restore them and read the parts you
haven't read and fix them too, before creating a new hardened spec". No agents and no workflow.

* **The check.** The whole of Hardened_4 was read by one reader in three sittings (every hand-written line: Parts I to XVIII,
  Annexes A to D, Appendices E and F; not the three generated appendices G to I, but §H.52 to §H.55). Its record is
  `tasks/96/redesign/spec-gaps-0.9.15/deep/check-0.9.20-H4.md`: findings 1 to 23 and T1 to T9, what was attacked and
  held, the four departures from the owner's verdicts judged, and how to rerun the two measurements.
* **Found.** One hole (finding 17: an owner stored, or a `weakref T` link made, by a member another package adds and a
  library calls was judged by no build); four places the text left a fact unsaid or said it twice differently (one
  `owns` for two rules, `shares this`, the job's arena, `debug_assert`); two example blocks that Hardened_2's re-wrap
  script had turned into run-on lines, which hid three examples from `check_examples.py` for three cuts; front matter
  that described Hardened_1; thirteen table rows without their indent; three glossary lines that said other than the rules; smaller text. Finding 7's first half was wrong
  (the containers it called unspecified are in `std/src/collections.em`, normative by Part XV's preamble).
* **The owner's answers** (`JOB.md` verdicts 167 to 169): a departure stands where the lead judges it better (ODR-264,
  ODR-261 and N87 do); verdicts 127 and 135 are restored (N83 and N138 withdrawn); fix and cut.
* **Applied** by seven asserting scripts kept with the session (`edit_h5_1_text.py` to `edit_h5_7_records2.py`, each
  replacement asserting that its old text occurs once; no script re-wraps a code block or a table): 32 rules changed
  (the rows of §H.56), registry rows E0109, E2122, E3022, E3023, E3032, E7015, W2016; no new rule, no new code, none
  retired; `tools/check_examples.py` now fails on a code fence that shares its line with other text.
* **Checked at the cut:** `check_ids` 0 problems (1,020 rules), `check_citations` 0 undefined, `code_registry --check`
  0 problems (271 live, 36 retired), `split_inline` UNS-8 only, `rule_index` no new problems, `spec_check` no new
  failures (46 blocks, 36 parse; baseline 10), `spec_check --emit-appendix` no change, `phase_progress --write` then
  `--check` green (the percentages did not move), `check_examples` 41 of 56 with 0 malformed fences (the 15 are
  unbuilt syntax: the value words, capture words, `guarded by`, `using`, `unsafe import c`, `@unsafe(export(…))`);
  `cargo test --profile ci -p ember_diag --lib` and `-p ember_parser` green against the new pin. The compiler is
  untouched. **Not run:** the full workspace suite, the bench, the push gate.
* **Known, and not the text's:** `check_examples.py` runs `ember check --syntax-only` on every block where `[TST-7]`
  says `ember check`; with the whole check, 21 of the 45 complete programs of Hardened_4 passed. The compiler writes
  its messages in the words 0.9.15 retired ("borrowed", "moved", "clone"). `std` is still written with `Box`,
  `Shared`, `Weak` and `@sync`. Each lands with the milestone that owns its rule.
* **For the owner:** the rulings N140 to N145 are the lead's and his to overturn; N143 names a feature that was not
  added (a pin for one-thread shared classes); N139's "costs nothing measurable" is to be measured against the mark
  in the weak word when `[FFI-35a]` is built; the readability costs of findings 13 to 16 follow from his own verdicts.

## 11. The sixth pass: 0.9.20_Hardened_6 (2026-11-10, the local session, attended, the lead solo)

Another agent reviewed Hardened_5 (`tasks/96/redesign/spec-gaps-0.9.15/deep/review-0.9.20-H5.md`) and found three holes and four smaller notes, none in the
rules of ownership, threads or types. The owner asked the lead what he thought, then: "Fix them and cut a new
hardened version". No agents and no workflow.

* **Each finding was run before a line was written** (C backend with Clang 22.1.8; MSVC 14.51 for plain C and
  C++; the probe folders were under `target/` and are deleted):
  * Hole 1, `[HR-12]`: by reading. The runtime takes one lock for every object (`rl_live_insert` and
    `rl_live_remove` in `runtime/ember_rt/src/ember_rt.c`; the insert is four stores), so the hole was only in
    the cheaper design the text describes.
  * Hole 2, `[FFI-51]`: `import c "lib.h"` with no `unsafe`, the header holding a pointer in `.CRT$XCU`:
    `ember check` passed and the program printed the header's line before "main starts"; the same in plain C on
    MSVC at `/O2`. **The compiler builds none of the header check**: it accepted `__attribute__((constructor))`
    too. Two more routes passed: `__attribute__((destructor))`, and `#pragma comment(lib, "winmm.lib")` with
    `#pragma comment(linker, "/include:timeGetTime")`, after which the program imported `WINMM.dll`.
  * Hole 3, `[MAN-10]`: Clang compiled `#embed "C:/Windows/win.ini"` with a warning in its default mode and its
    `-MD` output named the file; `.incbin` inside a function body's `__asm__` read `C:/Windows/system.ini` and
    `-MD` did not name it. MSVC 14.51 rejects `#embed` (C1021) and has no inline assembly on x64.
  * Note 3, `[FFI-57]`: `f(signed char)` with `f(short)` and `f(int)` calls `f(int)` on both compilers;
    `g(int)` with `g(long long)` and `g(double)` is ambiguous on both.
* **The pilot for `[FFI-51]`'s new shape** (owner rule: pilot before scaling). Of 353 MSVC and 2,558 Windows SDK
  headers, 81 hold `#pragma comment(lib`, 6 `#pragma comment(linker`. After preprocessing: `<stdio.h>`,
  `<stdlib.h>`, `<string.h>`, `<math.h>` and `<time.h>` give none; `<windows.h>` asks for `uuid.lib`; every C++
  standard header asks for `libcpmt`, passes `/alternatename:…` and states `detect_mismatch`. Of the 49 C
  headers under `tests/`, `examples/`, `std/` and `bench/`, 48 define no symbol the linker sees and the other's
  one symbol is the platform kit's (`<wchar.h>`); `vulkan.h` defines none. So the rule counts a library named by
  a header as a `[link] libs` entry and the toolchain's own headers as the toolchain, and no header the
  repository imports newly needs `unsafe`.
* **Applied** by three asserting scripts kept with the session (`edit_h6_1_spec.py`, run first on a copy of
  `parts/`, `edit_h6_2_tools.py`, `edit_h6_3_records.py`): 12 rules changed (the rows of §H.57), registry rows
  E3100 and E9001; no new rule, no new code, none retired.
* **Found on the way, and the larger part of note 1:** `[EFF-2]` and `[EFF-2a]` had said since Hardened_2 that a
  parameter of a function type is compiled once for each function given, against verdict 38, `[COST-3]`'s row
  and `[BLD-14]` item 16. Verdict 38 is restored (N150). It is a fifth departure of the cloud session from the
  owner's verdicts, beside the four it reported.
* **Checked at the cut:** `check_ids` 0 problems (1,020 rules), `check_citations` 0 undefined, `code_registry
  --check` 0 problems (271 live, 36 retired), `split_inline` UNS-8 only, `rule_index` no new problems,
  `spec_check` no new failures, `spec_check --emit-appendix` no change, `phase_progress --write` then `--check`
  green, `check_examples` 41 of 56 with 0 malformed fences; `ember_diag`, `ember_parser` and the three milestone
  tests that read the pinned document green. The compiler is untouched. **Not run:** the full workspace suite,
  the bench, the push gate (the owner: "Not yet").
* **The README moved, by the tool:** phases 5, 7 and 7a each fell by one point, because `[HR-13]` is now marked
  changed in 0.9.20 and its new behaviour is not built (`docs/conformance.json`: 281 to 280 rules done).
* **For the compiler, when these rules are built:** the header check of `[FFI-51]` (none of it exists); the
  file check and the assembly refusal of `[MAN-10]`; the runtime's registration already orders every object and
  may take the plain path for a type that is not `Send`; the reviewer's queue variant is to be measured before
  it is considered.
* **Not covered by either review:** the generated appendices G and I, and the change log before §H.52.
