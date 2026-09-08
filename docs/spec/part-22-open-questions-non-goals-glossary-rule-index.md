# Part XXII — Open Questions, Non-Goals, Glossary, Rule Index

## XXII.1 Owner decisions (`OQ-n`)

Every question this document has put to the owner carries a **stable `OQ-n` identifier**.
Identifiers are never reused and never renumbered: 0.3 renumbered this list and thereby broke
ADR-009's citation of "#9", which is why the numbering is now fixed. **Every question 0.4 raised is answered.** Three errata rulings inherited from 0.2 remain outstanding and are
recorded below as `OQ-24`..`OQ-26`; the implementation has followed all three since Phase 0, so each is a
confirmation rather than a design choice. Part XX §1 ground rule 3 continues to apply: an implementing agent
records a provisional choice and flags it, and does not decide silently.

`OQ-1`..`OQ-8` were confirmed on 2026-09-07 and are recorded as ADR-001..ADR-008. `OQ-9`..`OQ-23`
were answered for 0.5.

| Id | Question | Decision |
|---|---|---|
| `OQ-1` | Float literals default to `f32` (`[LEX-17]`) | confirmed — ADR-001 |
| `OQ-2` | Block scoping, not Python function scoping | confirmed — ADR-002 |
| `OQ-3` | `pub` for visibility | confirmed — ADR-003 |
| `OQ-4` | Class exclusivity checks stay on in `release` (`[EXC-1]`) | confirmed — ADR-004; cost re-derived per `OQ-16` |
| `OQ-5` | Generic arguments use `[]`, not `<>` (`[GRM-8]`) | confirmed — ADR-005 |
| `OQ-6` | C11 backend first, LLVM in v2 | confirmed — ADR-006 |
| `OQ-7` | Reference cycles leak, with tooling rather than a collector | confirmed — ADR-007 |
| `OQ-8` | RageV ECS integration shape (a) or (b) | deferred to measurement — ADR-008 |
| `OQ-9` | Language and CLI naming | **Keep `Ember` as the language name.** Language identity is separated from CLI, package, registry and runtime-ABI identity; the runtime symbol prefix is generated from `EMBER_SYMBOL_PREFIX` (`[RT-5]`, `[MNG-5]`, `[MAN-4]`), so a later rename of any one of them is a build-record change rather than a source break. |
| `OQ-10` | `Cell`/`RefCell` in the prelude (`[CELL-11]`) | **In the prelude.** `Shared[T]` was already there and is heavier on every axis; taxing the zero-cost facility inverted the gradient. `[DIA-9]` still bars them as a *first* suggestion. |
| `OQ-11` | `@static_safe` scope (`[EFF-12]`) | **Excludes only non-proof-establishing `RuntimeCheck(Aliasing)`.** A check whose reason is `establishes_static_fact` is permitted, because it returns proof-carrying values the borrow checker and backend consume. |
| `OQ-12` | Is ERR-005 decided? | **Confirmed.** `[LEX-11a]` is normative and unqualified. `docs/spec-source/` is authoritative; `docs/spec/` is generated and never hand-edited. |
| `OQ-13` | Does a call site write `mut`? | **No.** The callee's declaration determines the mode; `f(x)` is written for every mode (`[FN-2a]`). Part 0 row 3's "call sites never write `&`" is preserved verbatim. |
| `OQ-14` | Are `return`, `break`, `continue` expressions? | **Yes** — expressions of type `!` at the lowest precedence (`[GRM-16]`), which is what Part IV §2's `!` producer list already implied. |
| `OQ-15` | Does `::` stay? | **Stays**, as the qualified module/type/namespace path separator. `.` remains instance and member access. |
| `OQ-16` | Re-confirm ADR-004's access cost | **Decision kept; the number is not.** No fixed figure is normative. The cost is re-measured under `[BEN-1]`–`[BEN-7]` and published, and `ember inspect --safety` reports each access as `STATIC`, `ELIDABLE` or `DYNAMIC`. |
| `OQ-17` | Late-bound callback regions (`[LT-7]`) | **Adopted.** One level of higher-ranked quantification at callback boundaries; region variables stay compiler-internal and appear in no source, generic argument or ABI-visible name. `[THR-5]` and `[JOB-2]` become instances of it rather than bespoke exceptions. |
| `OQ-18` | What does `let` freeze? | **The binding, not the value** (resolution A). `[EXC-4]` narrows to instantaneous reads; a long-term access to a `let` field registers exactly as for any other field. `[CLS-8]`/`[THR-1]` restated so a `let Array[T]` field does not by itself make a class `Sync`. |
| `OQ-19` | Containers of view types | **Arbitrary owning containers stay rejected.** `[TYP-15a]` admits `BorrowList[T]`/`ViewList[T]` under one compiler-inferred region, which is never written by the programmer. |
| `OQ-20` | Demote `class` in diagnostics? | **No.** `[DIA-16]` discloses its cost instead — heap allocation, header, RC traffic, dynamic exclusivity, and ineligibility for `@static_safe` — so the trade is visible rather than steered. |
| `OQ-21` | Ship `@nopanic(explicit)`? | **Ship it** (`[EFF-17]`), with `NonZero[T]` in `std.core` (`[STD-4]`). `[EFF-16]` enumerates `Panic(Explicit)`. Bare `@nopanic` stays reserved for v2. |
| `OQ-22` | Source compatibility at 1.0 | **Guaranteed** within a major language version; a breaking language change requires a new major. Language, compiler, standard-library and runtime-ABI versions remain independent (`[VER-1]`..`[VER-7]`). |
| `OQ-23` | Bundle a C toolchain? | **Bundled** — a pinned Clang and `lld` (`[TOOL-2]`), which is what makes Part 0 row 7's promise true. A system compiler may be selected explicitly and is recorded in the build record. |
| `OQ-24` | ERR-002 — is `1f32` a float literal? | **Decided: allowed.** `1f32` and `1.0f32` are both float literals. `1.f32` is not: `float_lit`'s `dec_lit "."` form requires that no identifier character follow, which is what keeps `1.max(2)` parsing as a method call. |
| `OQ-25` | ERR-003 — is `;` punctuation? | **Decided: yes, but not as a statement separator.** `;` stays in the II.6 table because `[T; N]` and `[v; N]` require it. `simple_stmt` loses its `{";" small_stmt}` tail, so `a = 1; b = 2` is `E0105` and one line carries one statement (`[GRM-18]`) — the Python reading. |
| `OQ-26` | ERR-004 — is `let` contextual? | **Decided: fully reserved.** `let` joins the reserved set (48 entries, after `type` joins under ERR-009 and `from` leaves under ERR-017) and is a keyword in every position; `r#let` is required to use it as a name. Chosen over the contextual reading so that `let name: T` never depends on position, at the cost of one identifier nobody can use unescaped. |

## XXII.2 Non-goals (v1)

No tracing GC; no exceptions; no implicit numeric narrowing; no function overloading; no macros; no named lifetimes; no specialisation/HKT; no C++ subclassing from Ember; no shader/kernel compilation; no dynamic typing; no whole-program-required optimisations for correctness; no stable Ember-to-Ember ABI (C ABI is the stable boundary); **no relaxed or unchecked safety mode** — `Cell`/`RefCell` (Part IX §7) and classes (Part VIII) are the sanctioned ways to express aliased mutation, and each keeps its check rather than removing it; **no assumption that is checked in one profile and assumed in another** (`[DSJ-8]`) — the one exception the owner has granted is dynamic class exclusivity under `exclusivity = "unchecked"` in `shipping` (ADR-004), and it does not extend to any other mechanism; **no cycle collector** — reclaiming reference cycles requires root identification, graph traversal and mutator cooperation, which is a tracing collector over the RC subgraph and reintroduces exactly what Part 0 removed; `Weak`, the leak report (`[WK-1]`) and `L3001` are the whole answer.

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
| Enforcement ladder | `[PHIL-8]`: prove statically, else enforce safely at runtime, else require `unsafe` |
| Reason code | why a runtime check exists (`[EFF-11]`): not provable in principle, not proven by this analysis, requested by type, or inherent to the mechanism |
| Proof-carrying value | a value returned by a verifying operation (`assert_disjoint`) that carries the established fact, so the fact cannot be separated from what it describes |
| Diagnostic shape | one of the classified ownership/borrow error patterns in §XIX.6.1, each with a mandated concrete fix |

## XXII.4 Rule index

Rule ID prefixes and where they are specified: `PHIL` (I.3), `TIER` (I.2), `LEX` (II), `GRM` (III), `TYP` (IV), `MOD` `FN` `STR` `ENM` `CLS` `IFC` `STA` `ATT` (V), `EXP` `CTL` `CLO` `PAN` (VI), `OWN` `BRW` `LT` `DRP` `SPN` (VII), `OBJ` `RC` `EXC` `DSP` `WK` `OPT` (VIII), `HEAP` `ARN` `ALC` `UNS` `HND` `CELL` `DSJ` (IX), `EFF` (X), `THR` `PAR` `JOB` (XI), `SOA` `SIMD` `ECS` (XII), `ERR` (XIII), `CT` `RFL` `DRV` (XIV), `STD` (XV), `FFI` `BLD-FFI` (XVI), `GPU` (XVII), `CMP` `AST` `HIR` `MIR` `MONO` `CG-C` `MNG` `RT` (XVIII), `CLI` `MAN` `BLD` `PRF` `TST` `DIA` `FMT` `LNT` `IDE` `DOC` `VER` `TOOL` (XIX), `BEN` (XX), `RV` (XXI). Every normative rule MUST have a unique rule ID, a conformance test directory, a diagnostic mapping where it rejects source, and an entry discoverable by the extractor. **Rule-ID uniqueness is a hard invariant**: `tools/rule_index.py` MUST fail CI when the same rule ID is defined more than once in the active index, regardless of section or prefix. A *reference* to an existing rule is not a definition and MUST NOT create a second index entry; historical RFC or provenance prose may mention an existing ID without being indexed. No new normative rule may be introduced without a test mapping. `tools/rule_index.py` extracts every rule id matching `\[([A-Z][A-Z0-9-]*-[0-9]+[a-z]?)\]` from the specification — the trailing letter admits amendment ids such as `[LEX-11a]`, and the hyphenated class admits `[BLD-FFI-1]` and `[CG-C-3]` — and cross-checks `tests/conformance/`. It fails CI on a duplicate id, on an id with no test directory, and on a code named in a diagnostic shape but absent from the registry in XIX §6.

---

