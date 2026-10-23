# Ember's memory model: what the research found (2026-10-24)

The owner, 2026-10-23: the language needs a rework of its memory model. Three goals at once:
**C-like performance** (speed and memory against plain C), **memory safety without Rust's annoying
parts**, **Python-like syntax**. An annoying part can go in two ways: (a) a more Pythonic spelling of
the same guarantee, or (b) a smarter solution that makes the danger impossible or harmless.

This note sums up the three-phase research. **Nothing is decided and nothing in the language or the
compiler was changed.** The full reports are beside this file; every number below comes from them.

| Phase | Report | What it is |
|---|---|---|
| 1 | [phase1-borrowing.md](phase1-borrowing.md), [phase1-sharing.md](phase1-sharing.md) | Rust's annoying parts, each run through real `rustc` and real `ember check` |
| 2 | [phase2-swift-mojo.md](phase2-swift-mojo.md), [phase2-hylo-csharp.md](phase2-hylo-csharp.md), [phase2-vale-austral-pony.md](phase2-vale-austral-pony.md), [phase2-rc-elision.md](phase2-rc-elision.md) | How other languages avoid them, and what it costs them |
| 3 | [phase3-design-D1.md](phase3-design-D1.md), [-D2](phase3-design-D2.md), [-D3](phase3-design-D3.md) | Three candidate memory models |
| 3 | [phase3-break-D1.md](phase3-break-D1.md), [-D2](phase3-break-D2.md), [-D3](phase3-break-D3.md) | One adversary per design |
| 3 | [phase3-measurements.md](phase3-measurements.md) | Every design timed against plain C |

Probe programs, C models, plain C baselines and raw results are kept in [artifacts/](artifacts/), a copy
of `target/redesign/` without build files. Paths in the reports say `target/redesign/...`: the same files are under
`artifacts/...`. The harness, `bench/harness.h`, prints time, peak memory and a checksum.

## 1. Phase 1: how much of Rust's annoyance Ember has today

Every row was checked with the real compiler (`ember check`) and the real `rustc` (1.98.1).

| Id | Annoyance | Severity | Ember today |
|---|---|---|---|
| A1 | a method borrowing the whole object | high | partly |
| A2 | can't add to a list while holding a reference into it | high | same as Rust |
| A3 | two references into one list | medium | same |
| A4 | changing a list while looping over it (spawn/despawn in a game loop) | high | same |
| A5 | returning references, lifetimes | medium | partly (no lifetimes written; ODR-286's E3060 is worse than Rust) |
| A6 | structs holding references | medium | partly |
| A7 | self-referential data | medium | same |
| A8 | trees with parent pointers, graphs, cycles | high | partly |
| A9 | shared mutable state | high | partly |
| A10 | choosing among ownership kinds | high | partly (eleven mechanisms remain) |
| A11 | moves, use after move, clone | high | same |
| A12 | closures capturing mutable state | high | **worse** (a plain lambda is refused inside the callee, E3026; the suggested `owned fn` then changes a copy silently) |
| A13 | trait objects, downcasting | medium | partly |
| A14 | async | medium | partly (async is reserved) |
| A15 | threads | medium | partly |
| A16 | generics friction (orphan rule etc.) | medium | same |
| A17 | error messages that don't say what to do | medium | partly (most help texts name a working fix) |
| A18 | initialisation | low | partly |
| A19 | global mutable state | medium | same (and `FRAME += 1` gives a misleading "cannot find FRAME") |
| A20 | unsafe and FFI | low | partly |
| A21 | updating a map entry in place | low | partly |
| A22 | a view of a temporary | medium | same |
| A23 | moving a field out from behind `mut self` (state machines) | medium | same |
| A24 | struct, Box or class decides what compiles and what panics | high | **worse** |
| A25 | a local list of string views (`words = text.split()`) | high | **worse** (Rust accepts it) |
| A31 | caching inside a read-only method | medium | partly |
| A32 | a callback receiving the object that holds it | high | same |
| A33 | references out of a lock or RefCell guard | medium | **worse: unsound** (see section 5) |
| A34 | the clone-before-move dance for shared handles | medium | absent |

In short: Ember already dropped most of Rust's *syntax* (no lifetimes, no call-site modes, NLL
problem case #3 accepted), but kept nearly all of Rust's *rules*, and is worse than Rust in four places.

## 2. Phase 2: what the other languages do, and what it costs them

| Language | How it avoids Rust's annoyances | What it pays | Maturity |
|---|---|---|---|
| Swift | values with copy-on-write, counted classes, run-time exclusivity checks | counting: Swift client programs spent 42% of their time in reference counting (BRC paper, PACT 2018); a run-time call per class-property access | production |
| Mojo | ownership with inferred origins, ASAP destruction | leaves shared mutation unchecked (a safety hole) | 1.0 shipped 2026-08 |
| Hylo | no references at all: values, and accessors that `yield` a place | cannot express shared mutable objects | experimental |
| C# | second-class references (`ref`, `Span`), garbage collector | the GC; stale references write to dead buffers with no error | production |
| Vale | generational references: each object has a generation, each use compares it | +10.8% on one published program vs unsafe; RC was +25.3% | experimental; last release v0.2.0, the regions prototype never merged, the author moved to Mojo |
| Pony | reference capabilities, garbage collector | six capability keywords; footprint worse than the JVM's C4 collector | pre-1.0 |
| Lobster, Koka, Roc, Nim | reference counting, mostly removed at compile time | Koka within 10% of C++ on rbtree; atomic counts cost 5% to 59% | mixed |

Costs the phase 2 agents measured on this laptop (clang -O2, P-core, mains):
- A generation check per use of a stored reference: +0.57 ns in order (+23%), +4.61 ns random (+45%), 8 bytes per reference. A non-atomic count update per use: +14% / +19%.
- Pools whose pages never move: appending was faster (24.975 ms vs 34.974 ms) at the same peak memory; in-order sums about 25% slower with 4096-element pages, close to equal with 65536.
- Elements stored as counted objects instead of inline structs: 3.56x slower iteration (9.74x when scattered), 2.44x memory.
- Copy-on-write: +10% per write for the uniqueness test, plus 17.8 ms and 2x peak memory per 64 MiB when a shared buffer is copied.
- An atomic count pair 5.8 ns, against 0.84 ns non-atomic.

## 3. Phase 3: the three designs

**D1, values and projections** (Hylo, Swift). There are no references a programmer can store. Lists and maps are values that copy on write, and classes are counted. Parts of a value are reached through "projections": `mut` parameters, `for`/`with` bindings, and accessors that `yield`. One run-time rule replaces the borrow checker: an address is re-fetched whenever code may have changed what it points into.

**D2, generational references and stable stores** (Vale, EnTT). Every object lives in a slot that never moves, and the slot carries a generation number. An object has one owner, and everything else holds a reference `{slot, generation}`, any number of them, storable anywhere. Using a reference compares the generation where that can't be proven safe: a stale reference reads `None`, or panics. There is no borrow checker, no lifetimes, no counting and no exclusivity checks. Loops see the list as it was when they began.

**D3, static borrowing with automatic fallbacks.** The borrow checker stays as the fast path. Wherever Ember refuses a program today (E3020, E3022, E3040, E3060, E3063, E3026), the compiler inserts one of five cheap run-time mechanisms instead:
- **pin:** a list holds still while referenced;
- **watch:** a stored reference to an object carries a generation;
- **copy;**
- **counted cell:** for captured variables;
- **type check:** for downcasts.

`ember inspect --checks` lists them, and `@static` turns them back into errors. One object kind replaces class, Box, Weak, Shared and Handle.

### What the adversaries found

**None of the three is memory-safe as written.** Every hole was judged fixable, but the fixes cost something each design set out to remove.

| Design | Critical holes | Major holes | Annoyances judged NOT fixable |
|---|---|---|---|
| D1 | 6 (e.g. nested lends clear a one-bit pin; captured locals; a count race in its own Mutex example) | 3 | lists and dicts as values **silently lose updates in normal Python code**; threads can't share closure state; struct vs class still decides what compiles |
| D2 | 2 (references to stack values unspecified; references not tagged with their thread's store) | 6, each with a cheap fix | none (worst: `old = p.name; p.name = new; print(old)` panics, because strings and lists have one owner) |
| D3 | 5 (an object destroyed by a call while a parameter still refers to it; `pop()` under a pin; copying a value that holds an object's home) | 3 | struct vs class still changes what panics; a distant edit changes class layouts (whole-program analysis on every build) |

### Measured against plain C

Laptop on mains power, clang 22.1.8 -O2, pinned to the P-cores, median of 5 runs. Every model printed the same checksum as plain C on every run. The table gives time ratio to plain C, with the memory ratio in brackets.

| | W1 ECS vs archetype C | W1 vs EnTT-layout C | W2 tree | W3 game objects | W4 callbacks |
|---|---|---|---|---|---|
| plain C | 42.4 ms | 96.6 ms | 535.6 ms | 435.5 ms | 23.1 ms |
| D1 | 3.05 (2.19) | 1.34 (0.96) | 1.97 (1.69) | 1.16 (1.14) | 1.43 (1.11) |
| D2, the breaker's faithful versions | 5.27 (2.44) | 2.31 (1.07) | 1.45 (1.68) | 1.15 (0.95) | 1.40 (1.07) |
| D3 | 3.95 (2.17) | 1.73 (0.95) | 1.13 (1.39) | 0.97 (0.84) | 1.07 (1.05) |

Every design row is a **best case**. None includes the fixes for its safety holes. D1's W1 and D3's W1, W2 and W4 rely on compiler analyses the designs haven't defined yet.

**W1 note:** EnTT's layout (one sparse set per component) costs 2.28x by itself in plain C, compared with an archetype layout (all of an entity's components side by side). So in W1, the library's data layout costs more than any memory model.

## 4. The lead's reading (not decided: for the discussion)

1. **D1 conflicts with Python.** Lists and dicts as values lose updates in ordinary Python idioms, and its adversary judged that not fixable. It survives only if value semantics is wanted.
2. **D2 and D3 converge on the same ingredients.** Each of these appears in both, and in phase 2:
   - **Storage that never moves** (pages or stable slots) for lists and pools that are referenced. (b) It makes A2 and A4 harmless: you can add inside a loop.
   - **Checked stored references** (a generation compare) for parent pointers, targets, observers and ECS references, instead of Rc, Weak and RefCell. (b) It removes A6 to A10.
   - **`b = a` aliases, as in Python.** (a) It removes A11.
   - **Loops see the list as it began,** with changes applied at the loop's end. (b)
   - **One object kind** instead of class, Box, Weak and Shared; a smaller header; no per-field access words. (a) It removes A10 and A24.
   - **Threads: "move it or freeze it",** inferred with no Send/Sync words. (a)
   - **Accessors that `yield` a place** (Hylo, Swift) for returning parts of objects. (b) This removes ODR-286's E3060.
3. **D3 is the fastest** (W2 1.13x, W3 0.97x, W4 1.07x) but has the most critical holes, and its numbers rest on whole-program analyses that don't exist yet. **D2 is the simplest:** no borrow checker at all, two critical holes, and honest numbers of 1.15x to 1.45x on W2 to W4.
4. **The central choice for the owner is what happens to an object that is still referenced when its one owner lets go.**
   - D2 and D3: it dies, and the references read `None` or panic. That's fast, and nothing is counted.
   - Python: it stays alive. That needs counting or a garbage collector, which costs speed.
5. **For the ECS**, the layout (archetype vs EnTT sparse sets) matters more than the memory model in W1. The owner chose EnTT, and that choice deserves another look with these numbers.
6. **Proposed next step, pending the owner:**
   - a design D4: D2's model, plus D3's compile-time proofs to remove checks, plus yield-accessors, plus the breakers' cheap fixes;
   - then break it again and measure it again, with models that include the safety fixes;
   - prove it before pitching it.

## 5. Defects found in today's compiler (independent of the rework)

- **Unsound: a reference taken inside a `RefCell` or `Mutex` guard outlives the guard (A33).** Four probes pass `ember check`, including a use after free and a data race: `target/redesign/phase1/sharing/a33_lock.em`, `a33_lock_uaf2.em`, `a33_lock_race.em`, `a09_refcell_escape.em`.
- A struct field of type `Option[ref Node]` crashes the checker with a stack overflow (`phase1/borrowing/e08d_ref_self_type.em`).
- Moving out of an array element gives E3040; the spec (`[EXP-6]`) says E3011.
- `counts.entry(k).or_default() += 1` gives E2140, although `or_default` returns `ref mut`.
- The L3001 warning on a tree advises making the owning `children` edge weak, which is the wrong edge.
- `BorrowList` (`[TYP-15a]`) isn't implemented.
- E3026 is reported inside the callee, not at the call.
- `fn() => total.fetch_add(n)` reports "cannot find `n`".
- `target/analysis/ecs4/accept_change_during_loop.em` is marked run-pass, but by `[EXC-18]` it must panic.

## 6. Where everything stands (for resuming)

- `main` holds everything locally, and none of it is pushed: 88 commits since `46e05ba5`, including the AF, AH and AE merges, the D-907 to D-910 fixes, ODR-280's `cast x as D` / `cast x if D`, and the saved helper work (`d73a9e81`).
- The helpers are stopped. Their work is on `stopped/AI-llvm-backend`, `stopped/AJ-function-items` and `stopped/AK-compiler-memory`; see [../helpers-stopped.md](../helpers-stopped.md).
- ODR-286's owner verdict: the EnTT registry keeps its pools in a `Box`. Recorded, not to be built.
- On hold until the rework is decided: the ECS (`tasks/95/ecs-entt-design.md`), the spec cut for ODR-261 to 286, the 24-agent analysis workflow approved earlier, and the push (it needs the full check and the laptop bench first).
