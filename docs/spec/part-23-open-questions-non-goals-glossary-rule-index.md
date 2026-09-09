# Part XXIII — Open Questions, Non-Goals, Glossary, Rule Index

## XXIII.1 Owner decisions (`OQ-n`)

Every question this document has put to the owner carries a **stable `OQ-n` identifier**.
Identifiers are never reused and never renumbered: 0.3 renumbered this list and thereby broke
ADR-009's citation of "#9", which is why the numbering is now fixed. **Every question 0.4 raised is answered.** Three errata rulings inherited from 0.2 remain outstanding and are
recorded below as `OQ-24`..`OQ-26`; the implementation has followed all three since Phase 0, so each is a
confirmation rather than a design choice. Part XXI §1 ground rule 3 continues to apply: an implementing agent
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
| `OQ-27` | Should the verification layer ship as 0.6 at all, ahead of Phase 1? | **Decided: no.** The verification layer does not ship. Range types, the foreign boundary and the `Io`/`Lock` effects ship as 0.6; contracts and verification are removed rather than deferred, because Ember eliminates bad values by construction rather than asserting rules about them. |
| `OQ-28` | What is `--contracts`' default in `release`, and is `@requires` evaluated in the caller or the callee? | **Closed** — moot: `--contracts` no longer exists (owner decision, 0.6.2). |
| `OQ-29` | Is `@ensures` checked on the `?` error path? | **Closed** — moot: `@ensures` no longer exists (owner decision, 0.6.2). |
| `OQ-30` | Where does the SMT solver come from? | **Closed** — moot: no prover ships, so there is nothing to obtain (owner decision, 0.6.2). |
| `OQ-31` | Do `[CTR-11]`'s quantifiers ship in v1? | **Closed** — moot: `[CTR-11]` no longer exists (owner decision, 0.6.2). |
| `OQ-32` | Who owns the soundness of `std` contracts? | **Closed** — moot: the `verify` std layer no longer exists (owner decision, 0.6.2). |

## XXIII.2 Non-goals (v1)

No tracing GC; no exceptions; no implicit numeric narrowing; no function overloading; no macros; no named lifetimes; no specialisation/HKT; no shader/kernel compilation; no dynamic typing; no whole-program-required optimisations for correctness; no stable Ember-to-Ember ABI (C ABI is the stable boundary); **C++ subclassing from Ember was a non-goal through 0.7.1 and is no longer one**: `[FFI-39]` specifies it for a base declared `@ffi(trampoline)` with an enumerated virtual set, which is the bounded form. Unrestricted subclassing — multiple inheritance, virtual bases, overriding a virtual the overlay did not name — remains a non-goal (`[FFI-39]`, `E5056`). **no relaxed or unchecked safety mode** — `Cell`/`RefCell` (Part IX §7) and classes (Part VIII) are the sanctioned ways to express aliased mutation, and each keeps its check rather than removing it; **no assumption that is checked in one profile and assumed in another** (`[DSJ-8]`) — the one exception the owner has granted is dynamic class exclusivity under `exclusivity = "unchecked"` in `shipping` (ADR-004), and it does not extend to any other mechanism; **no cycle collector** — reclaiming reference cycles requires root identification, graph traversal and mutator cooperation, which is a tracing collector over the RC subgraph and reintroduces exactly what Part 0 removed; `Weak`, the leak report (`[WK-1]`) and `L3001` are the whole answer.

## XXIII.3 Glossary

| Term | Meaning |
|---|---|
| Value world / object world | code over `struct`s, views and containers (statically checked) / code over `class` handles (RC + dynamic exclusivity) |
| Handle (class) | pointer to a counted heap object; `Copy`; retain on copy, release on drop |
| Handle (resource) | `Handle[Tag]`: 20-bit index + 12-bit generation, plain `Copy` value |
| Coroutine | a `gen fn`'s suspended frame as a value; resumed to run the body up to the next `yield` (`[CORO-1]`) |
| Safe point | a call to `ember_reload_poll()` at which the Ember-depth counter is zero on every registered thread — the only point a reload may be applied (`[HR-3]`) |
| Permanent thunk | the fixed address that stands for a reloadable function everywhere in the process; a reload rewrites its target, never its address (`[HR-6]`) |
| Reload manifest | the image section listing every reloadable function, type schema and static, read instead of the platform export directory (`[HR-8]`) |
| Trampoline subclass | the generated C++ subclass that lets an Ember class extend a foreign base and receive its virtuals (`[FFI-39]`) |
| Safe Ember | a program with no `unsafe` block, no `unsafe fn` and no unbacked foreign fact; `[PHIL-10]` states exactly what it guarantees and `[PHIL-11]` what it does not |
| Reload transaction | PLAN and PREPARE together — the part of a reload that may fail, made panic-free by `[HR-34]` so that failing returns to the old image rather than aborting |
| Conformance profile | Core / Systems / Native / Dynamic; what an implementation declares it implements (`[CONF-1]`) |
| Zero-cost | for a *use* whose dynamic checks are statically discharged, emitted code containing no instruction the equivalent C would not (`[COST-1]`) — never a property of an abstraction in general |
| Requirement category | what a rule binds: language, ABI, toolchain, reference implementation, RageV, or nothing (`[CAT-1]`) |
| `Nondet` | the effect carried by an operation whose result may differ between runs or machines; `@deterministic` forbids it (`[DET-1]`, `[DET-2]`) |
| Shareable generic | one whose every use of `T` is a call to a method of `T`'s bounds, making one shared function plus per-type witness tables legal in place of one function per type (`[MONO-5]`) |
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
| Diagnostic shape | one of the classified ownership/borrow error patterns in §XX.6.1, each with a mandated concrete fix |

## XXIII.4 Rule index

Rule ID prefixes and where they are specified: `PHIL` (I.3), `TIER` (I.2), `LEX` (II), `GRM` (III, and V §8 for `[GRM-20]`..`[GRM-23]`, whose productions are stated beside the declaration rules that constrain them), `TYP` (IV), `MOD` `FN` `STR` `ENM` `CLS` `IFC` `STA` `ATT` (V), `EXP` `CTL` `CLO` `PAN` (VI), `OWN` `BRW` `LT` `DRP` `SPN` (VII), `OBJ` `RC` `EXC` `DSP` `WK` `OPT` (VIII), `HEAP` `ARN` `ALC` `UNS` `HND` `CELL` `DSJ` (IX), `EFF` (X), `THR` `PAR` `JOB` (XI), `SOA` `SIMD` `ECS` (XII), `ERR` (XIII), `CT` `RFL` `DRV` (XIV), `STD` (XV), `FFI` `BLD-FFI` (XVI), `GPU` (XVII), `RNG` (IV.2a), `CORO` (VI.5a), `DET` (X.2a), `TCB` (XX.6a), `CMP` `AST` `HIR` `MIR` `MONO` `CG-C` `MNG` `RT` (XIX), `CLI` `MAN` `BLD` `PRF` `TST` `DIA` `FMT` `LNT` `BUD` `CONF` `ABI` `CXX` `IDE` `DOC` `VER` `TOOL` (XX), `BEN` `GATE` (XXI), `RV` (XXII), `CAT` (XXIII.5). `HR` (XVIII), `TXT` (XV.4a), `SEL` (IX.0), `COST` (X.4). Every normative rule MUST have a unique rule ID, a conformance test directory, a diagnostic mapping where it rejects source, and an entry discoverable by the extractor. **Rule-ID uniqueness is a hard invariant**: `tools/rule_index.py` MUST fail CI when the same rule ID is defined more than once in the active index, regardless of section or prefix. A *reference* to an existing rule is not a definition and MUST NOT create a second index entry; historical RFC or provenance prose may mention an existing ID without being indexed. **A superseded ID named only in an earlier revision's change log is likewise not an index entry**: `tools/rule_index.py` MUST ignore rule ids occurring inside a `## Change log — <version>` section for a version other than the current one, so that the record of what a past revision did — `[HOT-1]`..`[HOT-10]`, replaced by Part XVIII in 0.7.1, and the `[CTR-*]`/`[PRV-*]` rules removed in 0.6.2 — neither demands a conformance test nor silently resurrects a deleted rule. A superseded id MUST NOT be reused for a new rule. No new normative rule may be introduced without a test mapping. `tools/rule_index.py` extracts every rule id matching `\[([A-Z][A-Z0-9-]*-[0-9]+[a-z]?)\]` from the specification — the trailing letter admits amendment ids such as `[LEX-11a]`, and the hyphenated class admits `[BLD-FFI-1]` and `[CG-C-3]` — and cross-checks `tests/conformance/`. It fails CI on a duplicate id, on an id with no test directory, and on a code named in a diagnostic shape but absent from the registry in XX §6.

---

## XXIII.5 Requirement categories

This document deliberately covers the language, the reference compiler, the
toolchain and one integration. That is useful to an implementing agent and dangerous
to a second implementation, because it makes *what Ember means* and *how this
compiler does it* look alike. Every normative rule therefore carries a category.

* `[CAT-1]` **The categories.**
  * `LANGUAGE-NORMATIVE` — changes what a program means or whether it is accepted.
    Any conforming implementation must obey it. Most of Parts I–XVIII.
  * `ABI-NORMATIVE` — fixes a representation two artifacts must agree on. Binding
    on anything that interoperates, including a future implementation.
    `[FFI-28]`, `[ABI-1..5]`, `[MNG-*]`, `[OBJ-1]`.
  * `TOOLCHAIN-NORMATIVE` — fixes a command, a manifest key, a diagnostic code or an
    output format that users and tooling depend on. Part XX.
  * `REFERENCE-IMPLEMENTATION` — describes how *this* compiler is built. Another
    implementation may differ freely and still conform. Part XIX's pass structure,
    the MIR verifier, the crate layout.
  * `RageV-INTEGRATION` — Part XXII. Binding on nothing outside that project.
  * `NON-NORMATIVE` — rationale, guidance, examples. Part 0's rejected
    alternatives, §IX.0's decision table, this sentence.
* `[CAT-2]` A rule's category is recorded in the rule index and emitted by
  `tools/rule_index.py`. **The categorisation is assigned by a one-time mechanical
  pass, not by hand**: every rule takes the default for the Part it lives in —
  Parts I–XVIII `LANGUAGE-NORMATIVE`, Part XIX `REFERENCE-IMPLEMENTATION`, Part XX
  `TOOLCHAIN-NORMATIVE`, Part XXII `RageV-INTEGRATION`, Part 0 and the glossary
  `NON-NORMATIVE` — and the pass then overrides the rules `[CAT-1]` names
  individually as `ABI-NORMATIVE`. Only once that pass has run and its output is
  committed does `tools/rule_index.py` fail CI on an uncategorised rule; requiring
  it before then would fail every build against 810 rules that predate this
  section, which is not a transition, it is a wall.
* `[CAT-3]` **A `REFERENCE-IMPLEMENTATION` rule MUST NOT be cited as the reason for
  a `LANGUAGE-NORMATIVE` one.** Where a language rule exists because the reference
  compiler is shaped a certain way, either the language rule is wrong or the
  constraint is real and belongs in `LANGUAGE-NORMATIVE` with its own justification.
  `tools/rule_index.py` reports such citations; they are reviewed, not auto-failed,
  because a few are legitimate cross-references rather than dependencies.
* `[CAT-4]` `ember explain <rule-id>` prints the category alongside the rule, so a
  reader asking "must my implementation do this" gets an answer without inferring it
  from which Part the rule lives in.
* `[CAT-5]` Categorisation is **descriptive, not a new constraint**: it records what
  each existing rule already was. A revision that changes a rule's category is
  changing what the rule binds and MUST have a change-log row saying so.

---

