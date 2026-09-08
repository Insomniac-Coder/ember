# Part XXII — Open Questions, Non-Goals, Glossary, Rule Index

## XXII.1 Questions requiring an owner decision (do not decide silently)

1. **Float literal default `f32` (`[LEX-17]`)** — confirm; the alternative is `f64` with pervasive suffixes in engine code.
2. **Block scoping vs Python function scoping (`Part 0 #13`)** — confirm block scoping.
3. **`pub` vs `export`** — confirm `pub`.
4. **Class exclusivity checks enabled in `release` (`[EXC-1]`)** — confirm (Swift's choice); alternative is debug-only with UB in release.
5. **Generic brackets `[T]` (`[GRM-8]`)** — confirm `[]` over `<>`; `[]` keeps Python's look, costs a resolution-time disambiguation.
6. **C backend first** — confirm; the alternative (LLVM first) delays RageV integration by an estimated one to two phases.
7. **Reference cycles leak (`[WK-1]`)** — confirm acceptance with tooling; the alternative (a backup cycle collector, à la CPython) is v2-possible but adds a tracing pass to the runtime.
8. **Stage 2 option (a) vs (b)** — to be decided by measurement.
9. **`Cell`/`RefCell` outside the prelude (`[CELL-11]`)** — confirm; keeping them out makes reaching for interior mutability a visible decision at the cost of one import line.
10. **Name of the language/CLI** — `ember`/`emberc` conflict check against existing packages on crates.io/PyPI/npm before publishing anything.

## XXII.2 Non-goals (v1)

No tracing GC; no exceptions; no implicit numeric narrowing; no function overloading; no macros; no named lifetimes; no specialisation/HKT; no C++ subclassing from Ember; no shader/kernel compilation; no dynamic typing; no whole-program-required optimisations for correctness; no stable Ember-to-Ember ABI (C ABI is the stable boundary); **no relaxed or unchecked safety mode** — `Cell`/`RefCell` (Part IX §7) and classes (Part VIII) are the sanctioned ways to express aliased mutation, and each keeps its check rather than removing it.

## XXII.3 Glossary

| Term | Meaning |
|---|---|
| Value world / object world | code over `struct`s, views and containers (statically checked) / code over `class` handles (RC + dynamic exclusivity) |
| Handle (class) | pointer to a counted heap object; `Copy`; retain on copy, release on drop |
| Handle (resource) | `Handle[Tag]`: 20-bit index + 12-bit generation, plain `Copy` value |
| View | a type carrying a borrow (`ref`, `Span`, `@view struct`, non-`owned` closure); has one region |
| Region | the set of program points where a borrow must remain valid; inferred by NLL |
| Loan | a borrow instance recorded by the borrow checker |
| Mode | parameter passing convention: borrowed (default), `mut`, `owned` |
| Effect | inferred property of a function: `Alloc`, `Sync`, `Panic`, `Unsafe`, `FFI`, `Block` |
| Contract | an attribute the compiler enforces (`@noalloc` …) as opposed to a hint (`@inline`) |
| Overlay | Ember file attaching FFI contracts/renames to an imported header |
| `.embind` | cached, content-addressed binding IR for a header + flags + overlay |
| Thunk | generated `extern "C"` C++ function bridging Ember to a C++ member/free function |
| Native island | a subsystem deliberately kept in C/C++ behind a typed Ember interface |
| Access state | GPU resource lifecycle: `CpuOwned → Recording → InFlight → CpuOwned` (+ `Retired`) |
| Interior mutability | mutation through a shared borrow, permitted by `Cell`/`RefCell`/`Atomic`/`Mutex`, which move the aliasing check to runtime (or make it unnecessary) |
| Diagnostic shape | one of the classified ownership/borrow error patterns in §XIX.6.1, each with a mandated concrete fix |

## XXII.4 Rule index

Rule ID prefixes and where they are specified: `PHIL` (I.3), `TIER` (I.2), `LEX` (II), `GRM` (III), `TYP` (IV), `MOD` `FN` `STR` `ENM` `CLS` `IFC` `STA` `ATT` (V), `EXP` `CTL` `CLO` `PAN` (VI), `OWN` `BRW` `LT` `DRP` `SPN` (VII), `OBJ` `RC` `EXC` `DSP` `WK` `OPT` (VIII), `HEAP` `ARN` `ALC` `UNS` `HND` `CELL` (IX), `EFF` (X), `THR` `PAR` `JOB` (XI), `SOA` `SIMD` `ECS` (XII), `ERR` (XIII), `CT` `RFL` `DRV` (XIV), `STD` (XV), `FFI` `BLD-FFI` (XVI), `GPU` (XVII), `CMP` `AST` `HIR` `MIR` `MONO` `CG-C` `MNG` `RT` (XVIII), `CLI` `MAN` `BLD` `PRF` `TST` `DIA` `FMT` (XIX), `RV` (XXI). `tools/rule_index.py` extracts every `[PREFIX-n]` from `docs/spec/` and cross-checks `tests/conformance/`.

---

