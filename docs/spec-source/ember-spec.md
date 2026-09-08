# Ember Programming Language — Design & Implementation Specification

**Version:** 0.5 (supersedes 0.4; see the change log at the end of Part 0)
**Authority:** This document is the sole normative source for Ember. It supersedes all earlier drafts, which are not required to implement anything described here.
**Status:** Implementation-ready specification for the reference compiler, runtime, toolchain and RageV integration
**Audience:** Implementing agents and engineers. This document is written to be executed against, not read for inspiration.
**Reference workload:** [RageV](https://github.com/Insomniac-Coder/RageV) (Windows C++ engine; Vulkan 1.3 + OpenGL 4.5 RHI; sparse-set ECS; render graph; C# scripting via a function-pointer table)

---

## How to use this document

1. **Part 0** records the foundational design decisions, the alternatives that were rejected, and why. Read it first; it explains the shape of everything after it.
2. **Parts I–XVII** are the *language* specification. Normative rules carry stable identifiers in the form `[XXX-n]` (e.g. `[OWN-4]`). Every rule with an identifier MUST have at least one test in the conformance suite that references it (see Part XIX §5).
3. **Part XVIII** specifies the compiler internals (IRs, passes, algorithms, ABI, mangling, backends).
4. **Part XIX** specifies the toolchain (CLI, manifest, build graph, test harness, diagnostics format, error-code registry).
5. **Part XX** is the implementation plan: phases, milestones, acceptance tests, repository layout, and operating instructions for an implementing agent.
6. **Part XXI** is the RageV integration plan, stage by stage, with concrete ABI sketches.
7. **Part XXII** lists questions that require an owner decision. An implementing agent MUST NOT silently choose an answer to these; it records a provisional choice in `docs/DECISIONS.md` and flags it.

The words **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, **MAY** are normative (RFC 2119). "v1" means the first stable release; "v2/v3" mean later releases. Anything marked **(v2)** or **(v3)** is specified so that v1 does not preclude it, but is not required for v1.

**Guiding sentence for the implementer:** when the specification is silent, choose the option that is (a) simplest to implement soundly, (b) most predictable at runtime, and (c) closest to what a C programmer would expect — in that order — and write the choice down in `docs/DECISIONS.md`.

---

# Part 0 — Foundational Decisions and Rejected Alternatives

The obvious way to design a language with Ember's goals is to include every mechanism a systems language can have — tracing GC, reference counting, ownership inference, arenas, borrow checking, escape-analysis-driven storage selection, dynamic types, a relaxed "experimental" safety mode — and let the compiler choose among them per value. That approach was evaluated against the RageV workload and rejected: it makes two similar pieces of source allocate, synchronise and destroy differently without the programmer noticing, and it requires every one of those mechanisms to be implemented before any of them is trustworthy.

Ember instead applies one principle everywhere:

> **The programmer's declared type determines the storage and lifetime model. The compiler may only perform optimisations that preserve that model's observable semantics.**

The concrete decisions, each paired with the alternative it replaces:

| # | Rejected alternative | Ember | Rationale |
|---|---|---|---|
| 1 | Tracing GC is the default heap for "managed" objects, with RC and ownership as alternatives chosen by the compiler. | **No tracing GC.** Value types use ownership + borrowing (checked statically, inferred). `class` instances use automatic reference counting with deterministic destruction. Arenas and handles cover the rest. | A GC cannot give deterministic destruction to the resources (files, GPU objects, locks) that need it most, requires write barriers and safepoints in *all* generated code, doubled the runtime, and was never wanted in RageV's hot paths. RC + ownership gives Python-like ergonomics for gameplay code with C-like predictability. A `Gc[T]` *library* heap can be added later without changing the language. |
| 2 | Storage strategy (stack / owned / shared / GC) chosen by escape analysis. | Storage strategy is a property of the **type**: `struct` = value, `class` = counted reference, `Box[T]` = unique heap, `Shared[T]` = counted heap struct, `Arena` = region. Escape analysis is now purely an optimisation (stack-promoting non-escaping class instances, eliding RC traffic). | Predictability. Two identical-looking declarations always behave identically. |
| 3 | Type-level modifiers `owned T`, `borrowed T`, `borrowed mut T`, `shared T`, `stack T`, `static T`. | **Parameter modes** (`x: T` borrow, `mut x: T` mutable borrow, `owned x: T` consume), one reference type family (`ref T`, `ref mut T`) for the rare cases that need a first-class reference, and explicit library types for the rest. `stack` removed (values are already on the stack). | Call sites never write `&`. Mode syntax mirrors Mojo/Swift, which have proven it readable. Removing `stack` removes a modifier whose only effect was "fail to compile if the compiler disagrees". |
| 4 | `soa Particle:` as a declaration form. | `SoA[Particle]` is a **container type**, generated by `@derive(SoA)` through compile-time reflection. The same struct may be stored AoS or SoA. | Layout is a property of the container, not the type. Avoids a second struct-like declaration form and lets one component type serve both the ECS and ordinary arrays. |
| 5 | `@experimental` relaxed-safety mode. | **Removed.** | A checked language with an unchecked-lite mode is a language with a hidden unsafe mode. Ergonomic friction is addressed with NLL, disjoint borrows, two-phase borrows, and diagnostics that name the fix. |
| 6 | `dynamic` values. | **Removed from core** (may return as a library `Any` type). | Not needed for the target workloads; complicates `@noalloc` reasoning. |
| 7 | Backend: LLVM first. | **Two backends over one MIR: a C11 backend for bootstrap and portability, LLVM for optimisation control.** C backend first. | A C backend gets real programs — and RageV integration through CMake/MSVC — running in weeks instead of months, needs no LLVM install on Windows, and is the only viable route to console toolchains. Without a GC there are no safepoints, so C loses nothing essential. LLVM is added when optimisation reports, PGO and precise vectorisation become the bottleneck. |
| 8 | Implementation language "C++ or Rust". | **Rust** for the compiler and tools; **C11** for the runtime library. | The compiler is security-critical (it enforces the safety story); Rust catches the compiler's own memory errors. The runtime is C so it links into any host (RageV is a static library built with MSVC) without a Rust toolchain at the consumer's site. |
| 9 | Operator overloading via `fn +(a, b)`. | Operator interfaces (`Add`, `Sub`, `Mul`, `Eq`, `Ord`, `Index`, …) implemented like any other interface. | One mechanism, not two; works with generic bounds. |
| 10 | `using` and `with` both present; `resource` declaration kind. | `with` only. Any type with a `drop` method is move-only and destroyed deterministically; no separate `resource` kind. | Fewer concepts. |
| 11 | `export`/`public` undecided. | `pub`. | Shorter, and reads naturally on fields and methods. |
| 12 | Float literal default unspecified (examples imply f64). | Integer literals default to `i32`, float literals default to **`f32`**. | Graphics/engine workloads are f32; mixing f64 temporaries into f32 math is the single most common numeric annoyance in game code. `1.0f64` or an annotation selects f64. (Precedent: Jai.) |
| 13 | Scoping unspecified (examples imply Python function scope). | **Block scoping**; drop at end of block in reverse declaration order; NLL for borrows. | Deterministic destruction requires it. |
| 14 | Class semantics vague ("reference-oriented, compiler decides"). | Classes are reference-counted heap objects with identity, `init` constructors with definite-initialisation analysis, optional single inheritance (`open class`, `virtual`/`override`), and Swift-style **dynamic exclusivity enforcement** for long-term mutable accesses through aliasing handles. | This is the model that gives Python/C# users the object graphs they expect (parent pointers, observers, editor panels) while keeping memory safety without a GC. |
| 15 | Thread-safety of `shared` decided per-object at runtime ("upgrade the control block"). | Reference counts are **atomic iff the class is `Sync`**; a non-`Sync` class is thread-confined and uses plain counters. Decided per type at compile time. | No runtime upgrade machinery; the count is atomic exactly when handles can cross threads. |
| 16 | GPU kernels as an eventual core language feature. | Shaders remain a separate language (as RageV already does with `.rvshader` → SPIR-V). Ember owns the **host-side** model: generational handles, command-scoped GPU ownership, deferred destruction, frame-in-flight tracking, and a typed shader interface generated from SPIR-V reflection. Kernel DSL is v3 and out of scope for this document beyond reservations. | Matches RageV's architecture exactly; keeps the core compiler small. |
| 17 | Specification written as prose. | Normative rules carry IDs; every ID maps to conformance tests; every compiler pass has an input/output contract. | So that an agent can implement and verify without reinterpreting. |

## Change log — 0.5

0.5 answers every question 0.4 put to the owner. It adds three rules, promotes three reserved
identifiers to normative text, and converts one open ambiguity into a decision. It removes no
guarantee and reopens no confirmed ADR.

**This change log is exhaustive for normative text: a revision that alters a rule without a row
here is a defect.**

| # | Change | Where |
|---|---|---|
| 1 | **All fifteen open questions answered.** Part XXII.1 becomes a decision record; `OQ-1`..`OQ-23` keep their permanent identifiers. No question is left for an implementing agent to decide silently. | XXII.1 |
| 2 | `[EXC-4]` **resolved** (`OQ-18`): `let` freezes the binding, not the value, so a long-term access to a `let` field registers exactly as any other field does and only an instantaneous read is exempt. This closes the aliasing hole the unconditional exemption left open, and `[CLS-9a]` states the mutate-through consequence. | VIII.3, V.5 |
| 3 | `[LT-7]` **adopted** (`OQ-17`): late-bound callback regions, one level of higher-ranked quantification at callback boundaries, with region variables compiler-internal and absent from source, generic arguments and ABI-visible names. `[THR-5]` and `[JOB-2]` stop being bespoke exceptions, and `[GPU-9]`'s `pass.native` becomes expressible. | VII.5 |
| 4 | `[TYP-15a]` **added** (`OQ-19`): `BorrowList[T]`/`ViewList[T]` may hold views under one inferred region, which the programmer never writes; arbitrary owning containers at view types stay rejected, and the new containers may not smuggle a view past `[TYP-15]`. | IV.4 |
| 5 | `[FFI-30c]` **added**: distributable, composable overlays with declared left-to-right order, explicit `override`, diagnosed conflicts, foreign type identity held stable, and the composed overlay identity folded into build invalidation. | XVI.4 |
| 6 | `Cell` and `RefCell` join the **prelude** (`OQ-10`), because `Shared[T]` was already there and heavier on every axis measured. `[DIA-9]` still bars them as a first suggestion. | IX.7, XV |
| 7 | **Call sites never write a mode** (`OQ-13`, `[FN-2a]`). Part 0 row 3's guarantee is preserved verbatim; the mode is read from the signature and surfaced by `ember inspect` and `[IDE-7]` inlay hints. | V.2 |
| 8 | `::` **kept** (`OQ-15`) with its purpose written down: the qualified module/type/namespace path separator, distinct from `.` for instance and member access. | III.5 |
| 9 | **No fixed exclusivity cost is normative** (`OQ-16`). The "~2 ns" figure is withdrawn as a promise and re-derived under `[BEN-1]`–`[BEN-7]`, because `[EXC-3]`, `[EXC-6]` and `[FFI-33]` all add work to that path. ADR-004's decision is unchanged. | VIII.3, I.4 |
| 10 | `[CLI-10]` `--report=engine` is a **reporting mode, not a profile**: it may not change type checking, acceptance, semantics or optimisation legality. `[PRF-1]` governs profiles and this is not one. | XIX.1 |
| 11 | The 1.0 promises become **normative rather than intent** (`OQ-22`), and the bundled Clang + `lld` toolchain becomes a **committed deliverable** (`OQ-23`, `[TOOL-2]`). | XIX.2, XIX.1 |
| 12 | **Rule-ID uniqueness is a hard invariant.** `tools/rule_index.py` fails CI when an id is defined twice in the active index; a reference is not a definition. This is the defect class that reached three drafts of the 0.5 proposal undetected. | XXII.4 |
| 13 | `[TST-11]` records the **v0.5 regression obligations** for `[EXC-4]`, `[LT-7]`, `[TYP-15]`/`[TYP-15a]`, `[FFI-30c]`, `[EFF-17]` and `[PRF-1]`, so every rule this revision adds or changes has a conformance mapping. | XIX.5 |
| 17 | `;` is **no longer a statement separator** (`OQ-25`, ERR-003). `simple_stmt` becomes `small_stmt`, `a = 1; b = 2` is `E0105`, and one line carries one statement. `;` stays punctuation only inside `[T; N]` and `[v; N]`. `[LEX-9]` and `[FMT-3]` follow. | III.4, II.2, XIX.7 |
| 16 | `1f32` is **confirmed legal** (`OQ-24`, ERR-002), alongside `1.0f32`. `1.f32` remains a method/field access on `1`, not a literal. | II.5 |
| 15 | `let` is **fully reserved** (`OQ-26`, ERR-004): it joins the reserved keyword set, which grows from 47 to 48 entries, and `r#let` is required to use the word as a name. `ember_lexer`'s keyword-count assertion moves from 47 to 48: `type` joins the set under ERR-009 and `from` leaves it under ERR-017. | II.4 |
| 14 | The **GPU host model is scheduled, not respecified**: Part XVII's `[GPU-*]` and Part XXI's `[RV-*]` remain the sole normative source, their implementation moves to v0.6, and v0.5 carries an explicit compatibility surface it may not break. The MIR interpreter becomes a supported restricted execution mode with mandatory differential testing. | XX.2 |

## Change log — 0.4

0.4 makes the document agree with itself and with the compiler that implements it. It adds no pillar and
removes none: every change is a reconciliation, a contradiction closed, a memory-safety hole in Safe code,
or a rule this document already implies and failed to state.

**This change log is exhaustive for normative text: a revision that alters a rule without a row here is a
defect.** 0.3 altered eight sites without a row, which is why the sentence is now normative.

| # | Change | Where |
|---|---|---|
| 0 | Carries the owner rulings ERR-001, ERR-005, ERR-006 and ERR-007 recorded in `docs/spec-errata.md`, which 0.3 reverted by being authored from an unpatched copy. The single-file source under `docs/spec-source/` is now stated to be normative and `docs/spec/` generated. | II.3, III.5, III.7, XIX.5, XX.1, XX.3 |
| 1 | The three still-open errata are written into the grammar: `;` joins the punctuation table (ERR-003), `let` joins the contextual keywords (ERR-004), `1f32` joins `float_lit` (ERR-002). Each is marked "implementation follows, owner ruling pending". | II.4, II.5, II.6 |
| 2 | Registry hygiene: the attribute table gains every attribute the document uses (`@static_safe`, `@borrows`, `@allow`, `@must_drop`, `@fp`, and the reserved forms); the rule index admits amendment and hyphenated ids and gains six prefixes; `type` becomes a v1 keyword. | III.7, II.4, XXII.4 |
| 3 | Contract effect sets are computed once under a fixed **contract profile** (`[EFF-15]`), so `@static_safe` no longer means one thing in `release` and the opposite in `shipping`. | X.1, X.2 |
| 4 | `[DSJ-6]` and `[EFF-12]` are made consistent: a check that *establishes* a static fact carries the new reason code `establishes_static_fact` and does not violate `@static_safe`. | IX.8, X.1, X.2 |
| 5 | `exclusivity = "unchecked"` reaches dynamic **class** exclusivity and nothing else; `[CELL-9]` keeps `RefCell`'s check in every profile, resolving the contradiction with XXII.2's own new non-goal. | VIII.3, IX.7, XIX.4, XXII.2 |
| 6 | Alias facts are derived, never assumed, and `[CG-C-4]` states how they reach the backend — without which `[SIMD-3]` and `[DSJ-3]` buy nothing. | IX.8, XII.2, XVIII.6 |
| 7 | Seven memory-safety holes in Safe code are closed: exclusivity elision only over a closed interval (`[EXC-3]`), resurrection during `drop` (`[OBJ-5]`, `[WK-3]`), borrows keeping objects alive (`[RC-5]`), leakable scope guards (`[THR-6]`, `@must_drop`), `@parallel` disjointness as a property of the place (`[PAR-2]`), `ScopedArena` rewind (`[ARN-7]`), and what `let` exempts (`[EXC-4]`, pending `OQ-18`). | VIII, IX.2, XI |
| 8 | The escape hatches the vision rests on are made usable: `@borrows` is registered and given a position (`[LT-1a]`), `?` can propagate an error of its own type (`[ERR-7]`), type arguments are admitted in expression position (`[GRM-8a]`–`[GRM-8c]`), and once-callable closures are selected by the existing parameter mode (`[CLO-6]`). | III, IV, VI, VII, XIII |
| 9 | The C backend gets the four things "speed of C" requires: cross-translation-unit inlining (`[CG-C-3]`), loop bounds-check versioning (`[OPT-2]`), guaranteed vectorisable loop shape (`[SIMD-5]`, `[CG-C-6]`), and enforced float control (`[TYP-9a]`–`[TYP-9c]`). | XVIII.6, VIII.6, XII, IV.2 |
| 10 | Grammar repairs for constructs the document uses and Part III did not define, and a rule that every fenced `ember` block in this specification is compile-checked (`[TST-7]`). | III, XIX.5 |
| 11 | The errors of the first hour get a mandated catalogue of their own (§XIX.6.2, shapes N1–N12), ownership shapes O5–O9, A1, B12, B13 are added, and `E3060` is split so one code no longer means two things. | XIX.6.1, XIX.6.2 |
| 12 | The editor becomes an architectural constraint rather than a later rewrite: an owned `Session` replacing the leaked interner, error tolerance past the parser, and item-granular re-checking (`[IDE-3]`, `[IDE-4]`, `[IDE-6]`, `[BLD-7]`–`[BLD-10]`); a new XIX §10 reserves the server itself. | XVIII.1, XIX.10 |
| 13 | Phase-5 completeness: the C importer imports what real headers contain (`[FFI-6]`, `[FFI-8]`), a generated shim translation unit handles `static inline` and single-header libraries (`[FFI-29]`), foreign contracts carry a count axis and an `unsafe overlay` boundary (`[FFI-11]`, `[FFI-2a]`, `[TIER-1]`), and imported entities have one identity (`[FFI-30]`). | XVI |
| 14 | The plan gains instruments that measure the **user** rather than the compiler: a benchmark protocol (`[BEN-1]`), a first-run milestone (`[TOOL-1]`–`[TOOL-4]`), and a corpus written by people who have not read this document (`[TST-8]`–`[TST-10]`). | XIX.1, XX.3, XX.4 |
| 15 | Part XXII.1 is renumbered once, permanently, under stable `OQ-n` identifiers, and twelve new questions are recorded rather than answered. | XXII.1 |

## Change log — 0.3

0.3 makes the enforcement model explicit and observable. It adds no new safety guarantee and removes none; every change either states a rule the compiler already followed, or makes an existing cost visible and restrictable.

| # | Change | Where |
|---|---|---|
| 1 | `[PHIL-8]` **the enforcement ladder** stated as a governing rule: prove statically → else enforce safely at runtime → else require `unsafe`. Rejection is correct only when a safe expression of the same intent exists, and the diagnostic must name it. | I.3 |
| 2 | `[PHIL-9]` **why classes carry dynamic checks and value types do not** — the class header already exists and is invisible to C; a `struct` has none because `[TYP-11]` guarantees C layout, and adding hidden state would break `size_of`, `@layout(c)`, `[FFI-5]` assertions and SoA/GPU bit-compatibility. The dichotomy is forced by the FFI guarantee, not by preference. | I.4 |
| 3 | A table of **which mechanism enforces which guarantee**, so the static/runtime split is documented rather than folklore. | I.4 |
| 4 | **`RuntimeCheck(k)` effect**, `k ∈ {Aliasing, Bounds, Stale, Overflow}`, computed *after* elision so it describes generated code. Coarse in the effect set; per-site detail lives in a codegen side table. | X.1.1 |
| 5 | **Reason codes** on every emitted check (`not_provable_in_principle`, `not_proven_by_analysis`, `requested_by_type`, `inherent_to_mechanism`), because "the compiler could not prove it" is false for a `RefCell` borrow or a generational compare and misleads the reader into restructuring code that cannot improve. | `[EFF-11]` |
| 6 | **`@static_safe`**, defined as "no `RuntimeCheck(Aliasing)`" rather than as a bespoke attribute. Deliberately excludes `Bounds`/`Stale`/`Overflow`; `@no_runtime_checks` reserved for v2. Documented as a value-type contract in practice, with a diagnostic that names the specific dynamic access. | X.2 |
| 7 | **`mem.assert_disjoint`** — verifies two view ranges do not overlap (two comparisons) and returns proof-carrying views the borrow checker and backend treat as disjoint. Paired with `unsafe assume_disjoint` for the unverifiable case. No profile-dependent third form. | IX.8 |
| 8 | **`ember inspect --safety`** — every check emitted, with reason, and every check elided, with the analysis that removed it. | X.3, `[CLI-3]` |
| 9 | Diagnostic shapes **B11** (disjointness not provable) and **S1** (`@static_safe` violated), plus `[DIA-11]`: when a check's reason is `not_provable_in_principle`, the suggestion must be to drop the contract or change the data structure, never to restructure. | XIX.6.1 |
| 10 | Non-goals extended: no profile-dependent assumptions, **no cycle collector** (it is a tracing collector over the RC subgraph and reintroduces what Part 0 removed). | XXII.2 |

Considered and rejected for 0.3: a fourth safety tier separating "statically proven" from "runtime enforced" — the existing tiers describe what the programmer *writes* (Safe / Contract / Unsafe), while static-vs-runtime describes how the compiler *enforces* within Safe; conflating them would imply a choice the programmer does not make. And limited RC cycle reclamation, per non-goal 10.

Every RageV-class requirement — first-class lifetime domains, borrow ergonomics for renderer code, command-scoped GPU ownership, deferred destruction, temporal history as a resource class, shader-language independence, native islands for backends and platform code, zero-cost FFI facades, transitive performance contracts, and visible allocation/effect decisions — is expressed with these primitives rather than as special cases. Part XXI.6 lists each requirement with the mechanism that satisfies it.

---

# Part I — Language Overview

## I.1 One-paragraph description

Ember is a statically typed, ahead-of-time compiled systems language with Python-style indentation syntax. Values have ownership; borrowing is checked by the compiler and inferred at call sites. Classes are reference-counted objects with deterministic destruction. There is no garbage collector, no exceptions, no null, no undefined behaviour outside `unsafe`. Performance-critical code uses value types, views, structure-of-arrays containers, arenas, SIMD and parallel loops, and can declare enforceable contracts (`@noalloc`, `@nosync`). C is imported directly from headers; C++ is bound through generated `extern "C"` thunks. The toolchain compiles Ember to C11 (bootstrap and portability backend) or LLVM IR.

## I.2 The three tiers

Every Ember program is written in one of three tiers, and the tier of any function is visible from its declaration:

| Tier | How you enter it | What the compiler guarantees | Typical use |
|---|---|---|---|
| **Safe** (default) | Nothing | Memory safety, null safety, bounds safety, lifetime safety, data-race freedom, deterministic destruction | Gameplay, editor, tools, orchestration |
| **Contract** | `@noalloc`, `@nosync`, `@simd`, `@parallel`, explicit `Arena`, `SoA`, `Span` | Everything in Safe, plus the declared performance contract is enforced at compile time | Culling, ECS systems, particle updates, command generation |
| **Unsafe** | `unsafe:` block / `unsafe fn` / `unsafe extern` | Type checking and diagnostics only; the programmer upholds the invariants listed in `[UNS-*]` | Backends, FFI shims, allocators, intrinsics |

There is no fourth tier. `[TIER-1]` Safe code MUST NOT invoke an operation with an unverifiable precondition unless an `unsafe` boundary in the same package discharges it. There are exactly three such boundaries: (1) an `unsafe:` block or `unsafe fn` lexically enclosing the call; (2) an `unsafe extern` declaration (`[FFI-10]`); (3) an `unsafe overlay` declaration (`[FFI-2]`). No other construct may make an unverified assertion, and no attribute, profile or contract may introduce a fourth.

## I.3 Design rules the compiler is held to

* `[PHIL-1]` Two syntactically identical declarations in the same context have identical storage, lifetime and synchronisation behaviour.
* `[PHIL-2]` No implicit heap allocation occurs except by constructing a type that is documented to allocate (`class` instances, `Array`, `String`, `Map`, `Box`, `Shared`, closures that escape).
* `[PHIL-3]` No implicit copy of a non-`Copy` value occurs. Copies of `Copy` values are bitwise.
* `[PHIL-4]` No implicit synchronisation occurs except through types documented to synchronise (`Mutex`, `Atomic`, channels, `Sync` class handles).
* `[PHIL-5]` Every safety check the compiler removes, it removes because it **proved** the check unnecessary. Declining to look is not a proof: an analysis that inspects only part of the program state that could invalidate a property has not established it, and removing the check on that basis is the same violation as changing a result (`[EXC-3]`, `[EFF-15]`). Optimisation never changes observable behaviour of safe code, and no profile, optimisation level or elision pass changes whether a program is accepted.
* `[PHIL-6]` Any expensive or dangerous conversion at an FFI boundary is either explicit in source or reported by the compiler.
* `[PHIL-7]` Panics are for programmer errors. Recoverable failures use `Result`.
* `[PHIL-8]` **The enforcement ladder.** Ember MUST guarantee memory safety, lifetime safety and data-race freedom for Safe code. The compiler SHOULD prove a required property statically whenever practical. Where static proof is unavailable but the property can be enforced safely at runtime, Ember MAY emit a runtime check instead of rejecting the program. An operation that can be made safe by neither static proof nor runtime enforcement MUST require an explicit `unsafe` boundary. Rejecting a program is correct only when no safe enforcement exists **and** the operation is expressible some other way — in which case the diagnostic MUST name that way (`[DIA-7]`).
* `[PHIL-8a]` **The ladder is enforced on every revision.** Every error code that rejects a program a previous language version accepted MUST have a diagnostic shape in §XIX.6.1 or §XIX.6.2 whose mandated `help`, applied literally to the rejected program, produces a program that compiles. `tools/rule_index.py` MUST verify this mechanically over `tests/ui/`: for each shape, the recorded "before" program fails with that code and the "after" program — obtained by applying the mandated fix — compiles. A shape whose fix does not compile is a defect of the same kind as `[DIA-11]`'s.

`[PHIL-8]` is the sentence that distinguishes Ember from a language whose only safety mechanism is static proof. It is a rule about **how** a guarantee is met, not about *which* guarantees hold: the guarantee list never shrinks, and no profile, contract or attribute weakens it outside `unsafe`.

## I.4 Static proof and runtime enforcement

Safe code obtains its guarantees from two mechanisms. Which one applies is decided per property and per program point by the compiler, never by the programmer — but it is always **visible** (`[EFF-9]`, `ember inspect --safety`) and always **restrictable** (`@static_safe`, `[EFF-12]`).

| Guarantee | Statically proven when | Runtime-enforced when | Mechanism and cost |
|---|---|---|---|
| No use-after-free, no dangling reference | always, for `ref`/`Span`/view types | never | borrow checker; zero cost |
| Value aliasing (`ref mut` XOR `ref`) | always, for value types | never | borrow checker; zero cost |
| Object aliasing (long-term access through a class handle) | accesses go through the same handle local and are statically ordered (`[EXC-3]`) | handles differ, or a handle is loaded from memory | header `access_state` word; ~2 ns per access pair |
| Interior mutability of a value type | never (the programmer opted in) | always | `Cell` (no check needed) / `RefCell` borrow state (`[CELL-5]`) |
| Index in bounds | index derived from the container's own length or a proven range | otherwise | compare + branch, predictable |
| Resource handle is live | never (generation is runtime data) | always outside `shipping` | generation compare (`[GPU-1]`, `[HND-1]`) |
| Object lifetime (class instances) | escape analysis promotes to the stack (`[OPT-1]`) | otherwise | reference counts, elided per `[RC-2/3]` |
| No data race | always, via `Send`/`Sync` | never — synchronisation is always explicit (`Mutex`, `Atomic`) | type system; zero cost |

`[PHIL-9]` **Why class instances can carry dynamic checks and value types cannot.** A class instance already has a 24-byte header (`[OBJ-1]`) holding its reference counts and type information; one more word carries access state at no layout cost, and the header is invisible to foreign code because handles are always passed as opaque pointers. A `struct` has no header by design: `[TYP-11]` guarantees C-compatible layout so that every plain struct can cross an FFI boundary unchanged, and `size_of`, `align_of` and field offsets are exactly what a C compiler would produce. Adding hidden runtime state to value types would change `size_of[T]()`, break `@layout(c)`, invalidate every `[FFI-5]` layout assertion, and make `SoA` columns and GPU-uploaded buffers no longer bit-identical to their C counterparts. The dichotomy is therefore forced by the FFI and layout guarantees, not by a judgement that one kind of code deserves more freedom than the other. Value types that genuinely need aliased mutation opt into a header-carrying type explicitly: `RefCell[T]` (Part IX §7) or a `class`.

## I.5 A complete small program

```ember
import std.io
from std.math import Vec3, sqrt

## A particle in a fountain simulation. Plain value type; 28 bytes.
@derive(Copy, Debug)
struct Particle:
    position: Vec3
    velocity: Vec3
    lifetime: f32

## Reference type with identity; handles are reference counted.
class Emitter:
    name: String
    particles: Array[Particle]
    spawn_rate: f32 = 100.0

    fn init(mut self, name: String):
        self.name = name
        self.particles = Array[Particle]()

    fn spawn(mut self, count: usize):
        for i in 0..count:
            self.particles.push(Particle(
                position=Vec3.ZERO,
                velocity=Vec3(0, 9.8, 0),
                lifetime=2.0))

## Contract tier: proven not to allocate, vectorisable.
@noalloc
@simd
fn integrate(mut particles: MutSpan[Particle], dt: f32):
    for p in particles.iter_mut():
        p.velocity.y -= 9.81 * dt
        p.position += p.velocity * dt
        p.lifetime -= dt

fn main() -> Result[void, io.Error]:
    fountain = Emitter("fountain")
    fountain.spawn(10_000)

    for frame in 0..600:
        integrate(fountain.particles.as_mut_span(), 1.0 / 60.0)
        fountain.particles.retain(fn(p) => p.lifetime > 0.0)

    io.println(f"{fountain.name}: {fountain.particles.len()} alive")
    return Ok(())
```

Everything in this program is specified precisely in the parts that follow.

---

# Part II — Lexical Structure

## II.1 Source encoding

* `[LEX-1]` Source files are UTF-8 without BOM. A BOM is accepted and ignored by the lexer; the formatter removes it. Any invalid UTF-8 sequence is a lexical error `E0001`.
* `[LEX-2]` Line endings are LF or CRLF; both are normalised to LF before tokenisation. Mixed endings are permitted (RageV's tree contains mixed files); the formatter normalises to the platform default unless `ember.toml` says otherwise.
* `[LEX-3]` File extension is `.em`. No other extension is a compilation unit.

## II.2 Indentation and line structure

Ember is indentation-sensitive in exactly the way Python 3 is, with stricter rules:

* `[LEX-4]` Indentation MUST use spaces. A tab character at the start of a logical line (outside a string) is a lexical error `E0002` with a fix-it that replaces it with 4 spaces.
* `[LEX-5]` The lexer emits `NEWLINE`, `INDENT`, `DEDENT` tokens using the Python algorithm: an indent stack starts at `[0]`; a logical line whose indentation exceeds the top pushes and emits `INDENT`; one that is smaller pops until equal, emitting one `DEDENT` per pop; an indentation that is not on the stack is `E0003 inconsistent dedent`.
* `[LEX-6]` Inside `(`, `[`, `{` and inside a triple-quoted string, newlines are not logical line ends (implicit line joining). Indentation of continuation lines inside brackets is not significant.
* `[LEX-7]` A backslash `\` at the end of a physical line joins it to the next physical line. The formatter never emits backslash continuations; it prefers bracketed continuation.
* `[LEX-8]` Blank lines and comment-only lines do not affect indentation.
* `[LEX-9]` An indented block MUST be introduced by a line ending in `:`. A `:` at the end of a line that is not followed by an `INDENT` (on the next non-blank line) is `E0004 expected an indented block` unless the block body is on the same line (`if x: return` — single simple statement only).

## II.3 Comments

```
# line comment to end of line
## doc comment (attaches to the next declaration); Markdown body
#! directive comment (first line of a file only, e.g. `#! language "0.5"`)
#$ test annotation, read by the test harness only; an ordinary comment to the compiler
```

* `[LEX-10]` There are no block comments. (A `###`-delimited block-comment form was considered and rejected: it is ambiguous with a doc comment followed by a `#`.)
* `[LEX-11]` A `##` comment attaches to the next declaration, ignoring blank lines. One that is not followed by a declaration documents nothing and is **discarded in silence**: a comment never affects compilation, and that includes producing a warning. (`W0001` remains registered because the registry lists every code the specification names, but nothing emits it.)
* `[LEX-11a]` A `##` comment on a comment-only line MUST be emitted after the `INDENT`/`DEDENT` tokens generated by the next line that carries content, so that a doc comment written inside a block attaches inside that block. `[LEX-8]` is unaffected: indentation still ignores comment-only lines completely.

## II.4 Identifiers and keywords

```
identifier  := XID_Start XID_Continue*        (Unicode; ASCII recommended)
```

* `[LEX-12]` Identifiers are NFC-normalised. Two identifiers are the same iff their NFC forms are byte-equal.
* `[LEX-13]` A single underscore `_` is the discard pattern/identifier, never a variable.
* `[LEX-14]` Raw identifiers `r#match` allow keywords as names (needed for imported C symbols such as a field called `type`).

**Reserved keywords (v1):**

```
and        as         break      class      comptime   const      continue
defer      dyn        elif       else       enum       extend     extern
false      fn         for        if         implements import     in
interface  is         let        match      mut        not        open
or         override   owned      pass       pub        ref        return
self       Self       static     struct     super      true       type
unsafe     virtual    void       where      while      with
```

**Reserved for future use (lexed as keywords, `E0005` if used):**

```
actor  async  await  macro  yield  move  trait  union  loop  unless
```

* `[LEX-15]` `abstract`, `final`, `lazy`, `test`, `bench` and `from` are **contextual**: they are keywords only in the positions specified in the grammar and identifiers elsewhere. `from` is a keyword only where it begins an import at item level, which is the one position an import may start; everywhere else it is an ordinary name, so that `interface From[T]` can declare `fn from(…)` as §8 writes it, and so that a user's `extend E implements From[io.Error]:` can too (owner decision, 2026-09-08; errata ERR-017). `let` is **fully reserved** (owner decision `OQ-26`), so it is a keyword everywhere and `r#let` (`[LEX-14]`) is required to use it as a name. `type` is fully reserved for the same reason: `[LEX-15a]` gives it three v1 meanings, so it cannot sit in the reserved-for-future list whose message `[LEX-14a]` requires to name a future version. `r#type` is the escape for the imported C field `[LEX-14]` names. The reserved set therefore has 48 entries.

## II.5 Literals

```
int_lit     := dec_lit | hex_lit | oct_lit | bin_lit
dec_lit     := digit (digit | "_")*
hex_lit     := "0x" hexdigit (hexdigit | "_")*
oct_lit     := "0o" octdigit (octdigit | "_")*
bin_lit     := "0b" bindigit (bindigit | "_")*
int_suffix  := "i8"|"i16"|"i32"|"i64"|"i128"|"u8"|"u16"|"u32"|"u64"|"u128"|"isize"|"usize"

float_lit   := dec_lit "." dec_lit exponent? | dec_lit exponent | dec_lit "." (not followed by identifier char)
             | dec_lit float_suffix                 # `1f32` (owner decision `OQ-24`)
exponent    := ("e"|"E") ("+"|"-")? dec_lit
float_suffix:= "f16" | "f32" | "f64"

char_lit    := "'" (char_body) "'"                 # exactly one Unicode scalar value
string_lit  := '"' string_body* '"'
raw_string  := 'r"' ... '"' | 'r#"' ... '"#' (up to 8 hashes)
fstring     := 'f"' (fstring_body | "{" expression (":" format_spec)? "}")* '"'
bytes_lit   := 'b"' ascii_body* '"'                 # type Span[u8] with static lifetime
cstr_lit    := 'c"' ascii_body* '"'                 # type cstr (NUL-terminated, static)
multiline   := '"""' ... '"""'                      # leading common indentation stripped

escape      := "\n" | "\r" | "\t" | "\0" | "\\" | "\"" | "\'" | "\x" hex hex | "\u{" hex{1,6} "}"
```

* `[LEX-16]` An integer literal without suffix has type **"untyped integer"** and takes the type expected by context; absent context it becomes `i32`. It may also take a float type by context (`Vec3(1, 2, 3)`). `E2010` if the value does not fit the target type.
* `[LEX-17]` A float literal without suffix is "untyped float"; absent context it becomes `f32`. `[LEX-16]`/`[LEX-17]` apply *before* overload resolution and are the only literal coercions.
* `[LEX-18]` `1.` is a float literal only when not followed by an identifier character; `1.abs()` is a method call on `1`.
* `[LEX-19]` f-string expressions are full expressions; `{{` and `}}` are literal braces; `format_spec` follows Python's mini-language subset: `[fill][align][sign][width][.precision][type]` with `type ∈ {d,x,X,b,o,e,f,g,s,?}` (`?` = Debug).
* `[LEX-20]` String literals have type `str` (a view with static lifetime). `String(literal)` or `literal.to_string()` produces an owned `String`.
* `[LEX-14a]` The words listed as reserved for future use are lexed as keywords. Using one as an identifier is `E0005`, whose message MUST name the version that will introduce it; `r#name` (`[LEX-14]`) is the escape.
* `[LEX-22]` A `'` followed by an identifier that is not closed by a second `'` lexes as a `Lifetime` token, reserved for v2 named lifetimes; the parser reports `E0007` (`[LT-6]`). This disambiguation is why `char_lit` requires exactly one Unicode scalar value followed by `'`.
* `[LEX-15a]` `type` is a v1 keyword: it introduces a type alias (`type_alias`), an associated type in an `interface` (`interface_member`), and an opaque foreign type in an `extern` block (`extern_item`). `union` remains reserved: v1 has no `union` declaration form in Ember source. The FFI importer materialises a C `union` into the synthetic binding module (`[FFI-8]`), which is generated rather than parsed from user source, so no hand-written program contains the token.
* `[LEX-17a]` A float literal with no suffix that receives type `f32` from `[LEX-17]`'s **default** (that is, no contextual type supplied it) and whose decimal form carries more than 9 significant decimal digits, or whose nearest `f32` differs from its nearest `f64` by more than one ulp of `f32`, MUST produce `W2015 float literal loses precision at f32`, showing the literal's value at both types and offering both `<literal>f64` and an `f32` annotation. A literal that receives `f32` from context is not diagnosed: the programmer chose the type. `W2015` is suppressible with `@allow(W2015)`.
* `[STD-5]` `Iterator.sum` and `fold` over `f32` accumulate in `f32`. `std.math` MUST additionally provide `sum_f64(self) -> f64` for `f32` iterators, `KahanSum[T]`, and `Vec2/3/4.sum_f64`, and their documentation MUST state the error bounds: naive summation of n terms carries a worst-case relative error of order n·ε; compensated summation of order 2·ε, independent of n. `std.math` reductions over more than a documented threshold — `mean`, `centroid`, `variance`, `Mat*` accumulation — MUST use compensated or `f64` accumulation internally and MUST say so, and the fast naive form MUST remain reachable.

## II.6 Operators and punctuation

```
Arithmetic:   +  -  *  /  %  **
Bitwise:      &  |  ^  ~  <<  >>
Comparison:   == != < > <= >=  is  is not
Logical:      and  or  not
Assignment:   =  +=  -=  *=  /=  %=  **=  &=  |=  ^=  <<=  >>=
Range:        ..  ..=
Access:       .  ?.  ::  (namespace path)  [  ]  (  )
Other:        ,  ;  :  ->  =>  @  ?  |  #  !
```

`[LEX-21]` Maximal-munch tokenisation. `..=` before `..`; `**=` before `**`; `?.` is a single token.
* `[LEX-6a]` Inside brackets, a lambda's `:` body is a single `small_stmt`, terminated by the enclosing closing bracket or by a `,` at the same bracket depth. No `NEWLINE` is required or emitted. This is the only construct for which a `block` may end without a `NEWLINE`. Consequently `f(fn(): g(), h())` passes two arguments; `h()` is not part of the lambda.

## II.7 Token stream contract

The lexer produces a `Vec<Token>` where each token has `{kind, span: (file_id, start_byte, end_byte), flags}`. `INDENT`/`DEDENT`/`NEWLINE` are real tokens. Doc comments are tokens (`DocComment(text)`); ordinary comments are discarded but their spans are recorded in a side table for the formatter. The lexer never fails fatally: unknown characters produce `Error` tokens and the parser continues.

---

# Part III — Grammar

This is the complete v1 grammar in EBNF. `{x}` = zero or more, `[x]` = optional, `|` = alternation. Terminal tokens are in quotes or UPPERCASE. The parser is a hand-written recursive-descent parser with Pratt-style expression parsing; the grammar is LL(2) except where noted.

## III.1 Compilation unit

```ebnf
file            := [directive] {NEWLINE} {import_decl} {item}
directive       := "#!" "language" string_lit NEWLINE

import_decl     := "import" module_path ["as" identifier] NEWLINE
                 | "from" module_path "import" import_list NEWLINE
                 | "import" "c" string_lit [ffi_opts] NEWLINE          (* C header or source *)
                 | "import" "cpp" string_lit [ffi_opts] NEWLINE        (* C++ header *)
module_path     := identifier {"." identifier}
import_list     := import_item {"," import_item} | "(" import_item {"," import_item} [","] ")"
import_item     := identifier ["as" identifier] | "*"
ffi_opts        := "with" "(" ffi_opt {"," ffi_opt} ")"
ffi_opt         := identifier "=" expression                          (* e.g. link="vulkan-1", overlay="vk.embind.em" *)

item            := {attribute} [visibility] item_body
visibility      := "pub" ["(" vis_args ")"]
vis_args        := "package" | "read" | "package" "," "read"      (* `read` is valid on fields only *)
item_body       := fn_decl | struct_decl | class_decl | enum_decl | interface_decl
                 | extend_decl | const_decl | static_decl | type_alias | extern_block
                 | comptime_block | test_decl
attribute       := "@" identifier ["(" [attr_args] ")"] NEWLINE?
attr_args       := attr_arg {"," attr_arg}
attr_arg        := expression | identifier "=" expression
```

## III.2 Declarations

```ebnf
fn_decl         := fn_header ":" block
                 | fn_header NEWLINE                                    (* only inside interface/extern *)
fn_header       := ["unsafe"] ["virtual" | "override"] "fn" identifier [generic_params]
                   "(" [param_list] ")" ["->" type] [where_clause]
generic_params  := "[" generic_param {"," generic_param} "]"
generic_param   := identifier [":" bound_list] ["=" type]
                 | "const" identifier ":" type                          (* const generic *)
bound_list      := type {"+" type}
where_clause    := "where" bound {"," bound}
bound           := type ":" bound_list
param_list      := param {"," param} [","]
param           := receiver | [mode] identifier ":" type ["=" expression]
receiver        := "self" | "mut" "self" | "owned" "self" | "self" ":" type
mode            := "mut" | "owned"

struct_decl     := "struct" identifier [generic_params] [implements_clause] [where_clause] ":" type_body
class_decl      := ["open" | "abstract"] "class" identifier [generic_params]
                   ["(" type ")"] [implements_clause] [where_clause] ":" type_body
implements_clause := "implements" type {"," type}
type_body       := NEWLINE INDENT {type_member} DEDENT | "pass" NEWLINE
type_member     := {attribute} [visibility] (field_decl | fn_decl | const_decl | type_alias | "pass" NEWLINE)
field_decl      := identifier ":" type ["=" expression] NEWLINE

enum_decl       := "enum" identifier [generic_params] [implements_clause] ":" NEWLINE INDENT {enum_member} DEDENT
enum_member     := {attribute} (variant | fn_decl | const_decl)
variant         := identifier ["(" variant_fields ")"] ["=" expression] NEWLINE
variant_fields  := (identifier ":" type | type) {"," (identifier ":" type | type)}

interface_decl  := "interface" identifier [generic_params] [":" bound_list] [where_clause] ":" NEWLINE
                   INDENT {interface_member} DEDENT
interface_member:= {attribute} (fn_decl | "type" identifier [":" bound_list] NEWLINE | const_decl)

extend_decl     := "extend" type [implements_clause] [where_clause] ":" type_body

const_decl      := "const" identifier [":" type] "=" expression NEWLINE
static_decl     := "static" ["mut"] identifier ":" type "=" expression NEWLINE
type_alias      := "type" identifier [generic_params] "=" type NEWLINE

extern_block    := ["unsafe"] "extern" string_lit ":" NEWLINE INDENT {extern_item} DEDENT
extern_item     := {attribute} (fn_header NEWLINE | static_decl | "type" identifier NEWLINE)

comptime_block  := "comptime" ":" block
test_decl       := "@test" NEWLINE fn_decl                              (* attribute form; no special syntax *)
```

Notes:
* A class's base class is written in parentheses: `class Door(Script):`. `[GRM-1]` At most one base class.
* `abstract class` may declare `virtual fn` without a body (abstract method); `[GRM-2]` a non-abstract class MUST implement all inherited abstract methods.
* `fn_decl` inside a `struct`/`class`/`enum`/`extend` body is a method if its first parameter is a receiver, otherwise an associated (static) function called as `Type.name(...)`.

## III.3 Types

```ebnf
type            := path_type | ref_type | ptr_type | tuple_type | fn_type | dyn_type | array_type | "Self" | "void" | "!"
path_type       := identifier {"." identifier} [generic_args]
generic_args    := "[" generic_arg {"," generic_arg} "]"
generic_arg     := type | expression                                    (* const generic argument *)
                 | identifier "=" type                                  (* associated type binding *)
ref_type        := "ref" ["mut"] type
ptr_type        := "*" ["mut"] type                                     (* raw pointer *)
tuple_type      := "(" ")" | "(" type "," [type {"," type}] ")"
fn_type         := ["extern" string_lit] "fn" "(" [type {"," type}] ")" ["->" type]
dyn_type        := "dyn" bound_list
array_type      := "[" type ";" expression "]"                          (* fixed-size inline array, e.g. [f32; 16] *)
```

`[GRM-3]` `Span[T]`, `MutSpan[T]`, `Array[T]`, `Option[T]`, `Result[T, E]`, `Box[T]`, `Shared[T]`, `Weak[T]` are ordinary library types spelled with `path_type`; the grammar does not special-case them.

## III.4 Statements

```ebnf
block           := NEWLINE INDENT {statement} DEDENT | simple_stmt NEWLINE
statement       := simple_stmt NEWLINE | compound_stmt
simple_stmt     := small_stmt                                           (* one statement per line; `;` is not a separator *)
small_stmt      := var_decl | assignment | expression | "return" [expression] | "break" [label]
                 | "continue" [label] | "pass" | "defer" ":" ...      (* defer is compound, see below *)
var_decl        := pattern ":" type ["=" expression]                    (* typed declaration, may be uninitialised *)
assignment      := target_list ("=" | augassign) expression
target_list     := target {"," target}                                  (* tuple destructuring *)
target          := identifier | postfix_expr | "_" | "(" target_list ")"
augassign       := "+=" | "-=" | "*=" | "/=" | "%=" | "**=" | "&=" | "|=" | "^=" | "<<=" | ">>="

compound_stmt   := if_stmt | while_stmt | for_stmt | match_stmt | with_stmt | defer_stmt
                 | unsafe_stmt | comptime_stmt | labeled_stmt
if_stmt         := "if" condition ":" block {"elif" condition ":" block} ["else" ":" block]
condition       := expression | pattern "=" expression                  (* `if Some(x) = opt:` *)
while_stmt      := "while" condition ":" block ["else" ":" block]
for_stmt        := "for" pattern "in" expression ":" block ["else" ":" block]
labeled_stmt    := identifier ":" (while_stmt | for_stmt)               (* `outer: for ...` *)
match_stmt      := "match" expression ":" NEWLINE INDENT {match_arm} DEDENT
match_arm       := pattern ["if" expression] ":" block
with_stmt       := "with" with_item {"," with_item} ":" block
with_item       := [pattern "="] expression
defer_stmt      := "defer" ":" block
unsafe_stmt     := "unsafe" ":" block
comptime_stmt   := "comptime" ":" block
```

Semantics notes tied to the grammar:
* `[GRM-4]` `x = expr` where `x` is not in scope declares `x` with the inferred type of `expr`. Where `x` is in scope, it assigns. `x: T = expr` always declares (shadowing in a nested block is allowed; redeclaring in the same block is `E1020`).
* `[GRM-5]` `a, b = expr` destructures a tuple/struct (pattern assignment); all names are declared or assigned consistently.
* `[GRM-6]` The optional `else` on `while`/`for` runs when the loop ends without `break` (Python semantics).
* `[GRM-7]` `defer` blocks run in reverse order at block exit, including on `return` and on panic-unwind when unwinding is enabled.

## III.5 Expressions

Precedence, lowest to highest (each row left-associative unless noted):

| Level | Operators | Notes |
|---|---|---|
| 1 | `x if c else y` | ternary, right-assoc |
| 2 | `or` | |
| 3 | `and` | |
| 4 | `not` (prefix) | |
| 5 | `==` `!=` `<` `>` `<=` `>=` `is` `is not` `in` `not in` | non-associative; chaining `a < b < c` is `E0102` (no Python chaining) |
| 6 | `..` `..=` | non-associative |
| 7 | `\|` | |
| 8 | `^` | |
| 9 | `&` | |
| 10 | `<<` `>>` | |
| 11 | `+` `-` | |
| 12 | `*` `/` `%` | |
| 13 | `-` `~` (prefix) | |
| 14 | `**` | right-assoc; binds tighter than unary minus on its left: `-2**2 == -4` |
| 15 | `?` (postfix try), `as` (cast) | |
| 16 | call `f(...)`, index `a[...]`, field `.x`, method `.m(...)`, optional chaining `?.`, generic instantiation `f[T]` | |
| 17 | atoms | |

```ebnf
expression      := ternary
ternary         := or_expr ["if" or_expr "else" ternary]
...                                                                     (* per table *)
postfix_expr    := atom {postfix}
postfix         := "(" [arg_list] ")" | "[" index_args "]" | "." identifier | "?." identifier
                 | "." int_lit (* tuple field *) | "?" | "as" type
arg_list        := arg {"," arg} [","]
arg             := expression | identifier "=" expression               (* named argument *)
index_args      := expression {"," expression}                          (* multi-dim index → Index[(A,B)] *)
atom            := literal | identifier | "self" | "(" expression ")" | tuple_lit | array_lit
                 | lambda | match_expr | block_expr | "Self" | path_expr
tuple_lit       := "(" ")" | "(" expression "," [expression {"," expression}] ")"
array_lit       := "[" [expression {"," expression} [","]] "]" | "[" expression ";" expression "]"
lambda          := ["owned"] "fn" "(" [lambda_params] ")" ["->" type] ("=>" expression | ":" block)
lambda_params   := lambda_param {"," lambda_param}
lambda_param    := [mode] identifier [":" type]
match_expr      := "match" expression ":" NEWLINE INDENT {pattern ["if" expression] "=>" expression NEWLINE} DEDENT
path_expr       := identifier "::" identifier {"::" identifier}         (* qualified module/type/namespace path *)
```

`[GRM-8]` **Generic-instantiation vs indexing.** `name[...]` in expression position is parsed as an `IndexOrInstantiate` node and resolved during name resolution: if `name` resolves to a generic function or type, it is an instantiation; otherwise an index. `[GRM-9]` A call immediately following (`f[i32](x)`) does not change this rule.

`[GRM-10]` **Statement vs expression `match`.** `match e:` followed by arms using `pattern: block` is a statement; arms using `pattern => expr` make it an expression. Mixing is `E0103`.

`[GRM-11]` Block expressions `(: ... )`: **not in v1**. Use a helper function or a `match` expression.

## III.6 Patterns

```ebnf
pattern         := alt_pattern
alt_pattern     := bind_pattern {"|" bind_pattern}
bind_pattern    := identifier "@" primary_pattern | primary_pattern
primary_pattern := "_" | literal | range_pattern | identifier            (* binding or unit variant/const *)
                 | path_type "(" [field_patterns] ")"                    (* variant/struct/tuple-struct *)
                 | "(" [pattern {"," pattern}] ")"                       (* tuple *)
                 | "[" [pattern {"," pattern} ["," ".." [identifier]]] "]"  (* slice *)
                 | "ref" ["mut"] identifier                               (* bind by reference *)
                 | "Some" "(" pattern ")" | "None" | "Ok" "(" pattern ")" | "Err" "(" pattern ")"
range_pattern   := literal ".." literal | literal "..=" literal
field_patterns  := field_pattern {"," field_pattern} ["," ".."]
field_pattern   := pattern | identifier "=" pattern                       (* positional or named *)
```

* `[GRM-12]` An identifier in pattern position resolves to a unit variant or `const` if one of that name is in scope; otherwise it is a fresh binding. The compiler warns `W1002` when a binding shadows a same-named variant in another enum to catch typos.
* `[GRM-13]` Patterns bind by value for `Copy` types and by **reference** (`ref`) otherwise when matching on a place expression that is not consumed; `match owned x:` consumes and binds by move. This mirrors Rust's default binding modes.
* `[GRM-8a]` Inside `[` `]` in expression position the parser MUST commit to `type_only_arg` when the next token is one of `ref`, `*`, `dyn`, `fn`, `extern`, `void`, `!`; otherwise it parses an `expression`. Each argument is recorded in the `IndexOrInstantiate` node as `TypeOrExpr::{Type, Expr}` (Part XVIII §2). The set is unambiguous: Ember has no prefix `*` and no prefix `!` (`not` and `~` are the operators), and the rest are keywords, so committing on those seven tokens cannot misparse an expression.
* `[GRM-8b]` Name resolution resolves the node per `[GRM-8]`. If it resolves to an **index** and any argument is a `TypeOrExpr::Type` or an `identifier "=" type` binding, it is `E2172 cannot index with a type`, naming the argument. If it resolves to an **instantiation**, each `TypeOrExpr::Expr` argument is reinterpreted as a type or a const-generic argument by the ordinary rules: an array-repeat literal `[T; N]` reinterprets as `array_type`, a tuple literal as `tuple_type`, a path expression as `path_type`; anything not reinterpretable is `E2173 not a type or const-generic argument`.
* `[GRM-8c]` `identifier "=" type` inside `[` `]` is an associated-type binding in both type and expression position; it is never a named argument and never an assignment.
* `[GRM-17]` A block-bodied lambda inside brackets whose body is not a single `small_stmt` is `E0106 a multi-statement closure cannot be written inside brackets`, with `help: bind it on a preceding line: `h = fn(e): …` then pass `h`` and `note: indentation is not significant inside brackets ([LEX-6])`.
* `[GRM-16]` `return`, `break` and `continue` are expressions of type `!` (Part IV §2), parsed at the **lowest** precedence, parallel to `ternary` and never as an `atom`: a `jump_expr` MUST be the whole of the expression in which it appears, so `a + return b` is `E0107 a jump expression may not be an operand`. Part III §4's `small_stmt` alternatives `"return" [expression]`, `"break" [label]` and `"continue" [label]` are removed; a jump written as a statement is an expression statement. `[CTL-7]`'s prohibition on `return`/`break`/`continue` leaving a `defer` block (`E2160`) is unaffected. `block_expr` is **deleted** from `atom`: `[GRM-11]` already rules it out of v1 and no production defines it.
* `[GRM-15]` `owned e` is permitted only as the iterable of a `for` and the scrutinee of a `match`, where it consumes `e` per `[CTL-1]` and `[GRM-13]`. `owned` elsewhere in expression position is `E0109`, with `help: `owned` marks a parameter, a receiver, a closure or a consumed scrutinee; to move a value, pass it to an `owned` parameter`.
* `[GRM-14]` `mut T` in a generic argument, or in a tuple type appearing as one, is an **access-mode argument**: it denotes write access to `T` rather than a distinct type. It is accepted only where the generic parameter is declared to take one, which requires a third kind of generic parameter (alongside type and const) declared `access P` — `Query[A: access…]` in `std.ecs`. Elsewhere `mut` in a type position is `E2020`. `Query[(mut Position, Velocity)]` therefore parses as a tuple of access-mode arguments, and iteration yields `ref mut Position, ref Velocity` per `[ECS-3]`.
* `[GRM-18]` `;` is **not** a statement separator (owner decision `OQ-25`). One line carries one statement. A `;` between two small statements is `E0105 `;` is not a statement separator`, whose help is to put each statement on its own line. `;` remains punctuation solely inside `[T; N]` and `[v; N]` (`array_type`, `array_lit`).
* `[GRM-19]` The pattern in a `condition` MUST be refutable. An irrefutable pattern is `E2036 this pattern always matches`, with `help: write `x = e` on the preceding line`.

## III.7 Grammar of attributes recognised by the compiler

Unknown attributes are `E0104` unless prefixed with a registered plugin namespace (`@ragev.field`). Recognised v1 attributes (full semantics where referenced):

| Attribute | Applies to | Kind |
|---|---|---|
| `@derive(A, B, …)` | struct, class, enum | code generation (Part XIV) |
| `@layout(c)` `@packed` `@align(N)` `@repr(int_type)` | struct, enum | layout contract (Part IX) |
| `@gpu_layout(std140 \| std430 \| scalar)` | struct | layout contract (Part XVII) |
| `@noalloc` `@nosync` `@nopanic`(v2) | fn | hard contract (Part X) |
| `@inline` `@noinline` `@cold` `@hot` | fn | hint |
| `@simd` `@parallel` `@unroll(N)` | for statement, fn | hint / contract (Part XII) |
| `@overflow(panic \| wrap \| saturate)` | fn, module | semantics (Part VI) |
| `@fastmath` | fn | semantics (Part VI) |
| `@must_use` `@deprecated("msg")` | fn, type | diagnostics |
| `@export("symbol")` | fn, static | ABI (Part XVI) |
| `@ffi(...)` | extern fn / overlay | FFI contract (Part XVI) |
| `@test` `@bench` `@should_panic` | fn | toolchain |
| `@view` | struct | marks a borrow-carrying type (Part VII) |
| `@move_only` | struct | disables `Copy` derivation |
| `@sync` `@thread_local` | class | overrides `Sync` derivation (Part XI) |
| `@reflect` `@serialize` | type | metadata generation (Part XIV) |
| `@static_safe` | fn | hard contract (Part X `[EFF-12]`) |
| `@noblock` | fn | hard contract (Part X) |
| `@no_runtime_checks` | fn | **reserved (v2)** — recognised, rejected with `E0104` naming the reservation, never silently ignored |
| `@borrows(param, …)` | fn | region contract (Part VII `[LT-1a]`) |
| `@assume_noalloc(expr)` | expression, inside `unsafe` | effect override (`[EFF-7]`) |
| `@allocator(Name)` | class | **reserved (v2)** (`[OBJ-4]`) |
| `@prelude` | module | import (`[MOD-3]`) |
| `@export_table("Name", protocol=N)` | struct | ABI (`[FFI-26]`, Part XXI) |
| `@non_exhaustive` | enum | FFI import (`[FFI-8]`) |
| `@component(layout=soa\|aos)` | struct | ECS storage (`[ECS-2]`) |
| `@soa(flatten)` | field | SoA column layout (`[SOA-1]`) |
| `@gpu` | fn | **reserved (v3)** (XVII §8) |
| `@allow(code, …)` | any item, and any statement admitting a statement attribute | suppresses the named `W`/`L` diagnostics within the annotated item |
| `@must_drop` | struct, class | drop is load-bearing for a borrow guarantee (`[THR-6]`) |
| `@fp(contract)` | fn | float control (`[TYP-9b]`) |
| `@deprecated(since, note)` | any item | policy (`[VER-3]`, intent) |

---
# Part IV — Type System

## IV.1 Type categories

Every type belongs to exactly one **category**, which determines storage, copy/move behaviour, and how it interacts with the borrow checker and the runtime:

| Category | Declared by | Stored | Assignment `b = a` | Destroyed |
|---|---|---|---|---|
| **Scalar** | built-in | inline | copy | never (trivial) |
| **Value** | `struct`, `enum`, tuple, `[T; N]`, closures | inline | copy if `Copy`, else move | end of scope, reverse order |
| **Reference** (handle) | `class` | pointer to counted heap object | handle copy (retain) | when strong count → 0 |
| **View** | `ref T`, `Span[T]`, `@view struct` | inline pointer(s) + compile-time region | copy (views are always `Copy`) | never (no destructor); region checked |
| **Raw** | `*T`, `*mut T`, `extern fn` | inline pointer | copy | never |
| **Existential** | `dyn I`, `Box[dyn I]`, class handle upcast | fat pointer (data + vtable) | per underlying | per underlying |
| **Unit / Never** | `void`, `!` | zero-size | trivial | trivial |

`[TYP-1]` Every concrete type has compile-time-known `size`, `align`, `is_copy`, `is_send`, `is_sync`, `needs_drop`, `is_view`, `has_niche`. The compiler computes these in the `TypeInfo` table (Part XVIII §4.3).

## IV.2 Scalar types

| Type | Size | Notes |
|---|---|---|
| `i8 i16 i32 i64 i128` | 1 2 4 8 16 | two's complement |
| `u8 u16 u32 u64 u128` | 1 2 4 8 16 | |
| `isize usize` | pointer | `usize` is the index and size type |
| `f16 f32 f64` | 2 4 8 | IEEE 754; `f16` arithmetic is performed in f32 and rounded on the C backend |
| `bool` | 1 | values 0/1 only; `[TYP-2]` producing any other bit pattern is UB and requires `unsafe` |
| `char` | 4 | Unicode scalar value; `[TYP-3]` surrogates are invalid |
| `void` | 0 | the unit type; value `()` |
| `!` | 0 | never type; coerces to every type; result of `panic`, `return`, `break`, `continue`, infinite `while true` |

`[TYP-4]` **No implicit conversions between scalar types in operators.** `i32 + i64` is `E2020`. `[TYP-5]` **Coercion sites** (assignment, argument, return, field initialiser, array element) allow **lossless widening**: `iN → iM` (M>N), `uN → uM` (M>N), `uN → iM` (M>N), `f32 → f64`, `f16 → f32`. Nothing converts to/from `bool` or `char` implicitly. `[TYP-6]` Everything else uses `as`:

* `x as T` for numeric types: truncation for narrowing integers (bit truncation), float→int saturating with NaN→0 (Rust semantics), int→float round-to-nearest.
* `x as u8` from `char`, `x as char` from `u8` only (wider ints via `char.from_u32() -> Option[char]`).
* `[TYP-7]` `as` between pointer types and between pointer and `usize` requires `unsafe`.

**Integer overflow** `[TYP-8]`: in the `debug` profile every arithmetic operation that overflows panics with `E-panic: integer overflow` and the source location. In `release`/`shipping`, `+ - *` and `<<` wrap two's-complement; `/` and `%` by zero always panic; `i32.MIN / -1` always panics. `@overflow(panic|wrap|saturate)` on a function or module overrides the profile. Explicit methods `wrapping_add`, `checked_add -> Option`, `saturating_add`, `overflowing_add -> (T, bool)` always exist.

**Floating point** `[TYP-9]`: strict IEEE semantics; no fast-math, no FMA contraction, no reassociation unless the function is `@fastmath`, in which case the C backend emits `#pragma float_control(precise, off)`/`-ffast-math`-equivalent attributes for that function only. `NaN == NaN` is false; `Ord` is not implemented for floats — use `partial_cmp` or `total_cmp`.

**Shifts** `[TYP-10]`: shift amount ≥ bit width panics in debug and is masked in release (like Rust).

## IV.3 Compound value types

**Tuples** `(A, B, C)`: value category; fields `.0 .1`; `Copy` iff all elements `Copy`. Destructured by `a, b = t`.

**Fixed arrays** `[T; N]`: inline, `N` is a const generic; index bounds-checked; `Copy` iff `T: Copy`. Literal `[0.0; 16]`. Coerces to `Span[T]`/`MutSpan[T]` at coercion sites.

**Structs**: see Part V. Field order in memory follows declaration order unless `@layout(rust)` is given (which permits reordering for size — v2). `[TYP-11]` Default layout is **C-compatible** (`@layout(c)` is implied); this is deliberate so that every plain struct can cross an FFI boundary.

**Enums**: unit-only enums have an integer discriminant (`@repr(u8)` etc., default `i32`-sized-or-smaller chosen by the compiler; `[TYP-12]` `@repr` is required for FFI). Payload enums are tagged unions; layout: `{tag, union of variants}` with the tag placed at offset 0 unless a niche makes it free.

**Niche optimisation** `[TYP-13]`: `Option[T]` where `T` is a class handle, `Box`, `ref`, `Span` (non-null pointer), `bool`, `char`, or an enum with fewer variants than its repr allows, has the same size as `T`. This is *guaranteed* for handles, `Box`, `ref` and `*fn` so that `Option[Handle]` is ABI-compatible with a nullable pointer.

## IV.4 Reference and view types

* `ref T` / `ref mut T`: a first-class reference. Non-null, aligned, points to a live `T` for the duration of its region. `Copy` for `ref T`; `ref mut T` is **move-only** (reborrowable). Auto-dereferenced: `r.field`, `r.method()`, and use of `r` in an expression of type `T` all read through. `[TYP-14]` A `ref mut` local written with `=` writes through to the referent (C++ reference semantics). Rebinding is not possible; shadow instead.
* `Span[T]` (read) and `MutSpan[T]` (read/write): pointer + length. `Copy` and move-only respectively. Bounds-checked indexing; `.len()`, `.iter()`, `.iter_mut()`, `.split_at(i)`, `.chunks(n)`, `.as_ptr()` (unsafe result).
* `str`: `Span[u8]` known to be valid UTF-8.
* `@view struct`: any struct containing a `ref`, `Span`, `MutSpan`, `str` or another view type is automatically a view type. The attribute is required on the declaration as documentation; omitting it is `E2030` with a fix-it. A view type has exactly one implicit **region parameter** (Part VII §5).

`[TYP-15]` A view-typed value MUST NOT be stored in a place whose region is not outlived by the view's region. Class fields, non-view struct fields, `static`s, `Box[T]` and `Shared[T]` contents, container elements and `owned fn` captures have no bounding region and are therefore always forbidden (`E3063 stored view may not outlive its source`, shape B12). Views MAY live in locals, parameters, return values, and in tuple, enum, `Option` and `Result` payloads. Whether a generic container instantiated at a view type may itself become a view type is reserved (OQ-19).

`[TYP-15a]` **Specialized containers of views.** `BorrowList[T]` and `ViewList[T]` MAY contain view-typed elements when all elements are bounded by one compiler-inferred region. The region is inferred from the container's construction and mutation context and MUST NOT be written by the programmer. The container's element region MUST satisfy the one-region model of `[LT-2]`; an operation that would require two independent element regions is rejected. These types are specialized borrowing containers, not ordinary owning generic containers, and they MUST NOT be used to smuggle a view into a class field, `static`, `Box`, `Shared`, or another place forbidden by `[TYP-15]`. Arbitrary owning containers instantiated at a view type (`Array[str]`, `Map[str, V]`, `Array[MutSpan[T]]`) remain rejected.

## IV.5 Raw types

`*T`, `*mut T`: nullable, unaligned-permitted, untracked. Creating one from a `ref` is safe (`ref_to_ptr`); dereferencing, offsetting, reading, writing require `unsafe`. `null[T]()` produces a null pointer. `*void` is permitted for FFI.

`extern "C" fn(i32) -> i32`: a raw function pointer with the C calling convention; `Copy`; cannot capture.

## IV.6 Class handles

A `class C` declaration introduces the type `C` whose values are **handles** (non-null pointers to counted heap objects). `Option[C]` is the nullable form. `Weak[C]` is a weak handle. Handles are `Copy` (copying retains). Upcasting a handle to a base class or to `dyn I` is implicit; downcasting uses `h as? Derived` → `Option[Derived]` (runtime type check) or `h as! Derived` (panics). Semantics in Part VIII.

## IV.7 Generics

* `[TYP-16]` Generic functions and types are **monomorphised**: every distinct instantiation produces a distinct symbol. Code size is the programmer's responsibility; the compiler deduplicates identical instantiations across modules at link time (COMDAT in the C backend via `inline`/`selectany`, weak symbols via LLVM).
* Type parameters are **bounded** by interfaces: `fn sum[T: Numeric](xs: Span[T]) -> T`. `[TYP-17]` Inside a generic body, only operations provided by the bounds (and universal operations: copy if `T: Copy`, move, drop, `size_of`) are permitted. There is no duck typing; a missing bound is `E2040` with a suggested bound.
* Const generics: `fn zero[T, const N: usize]() -> [T; N]`.
* Associated types in interfaces: `interface Iterator: type Item; fn next(mut self) -> Option[Item]`. Bindings: `Iterator[Item = i32]`.
* Default type parameters: `interface Add[Rhs = Self]`.
* `[TYP-18]` Generic parameters are inferred from arguments at call sites by unification; explicit instantiation `f[i32](x)` is allowed and required when no argument mentions the parameter (`Array[f32]()`).
* `[TYP-19]` No specialisation, no higher-kinded types, no variadic generics in v1. Overlapping `extend` impls are `E2041`.
* `[TYP-9a]` **Contraction is off by default and MUST be made off.** `[TYP-9]`'s prohibition on FMA contraction is not the default of any supported host C compiler. The C backend MUST emit `#pragma STDC FP_CONTRACT OFF` at the head of every translation unit **and** pass the corresponding flag, because GCC does not implement that pragma: Clang and GCC `-ffp-contract=off`; MSVC `/fp:precise` with `#pragma fp_contract(off)`. A toolchain on which contraction cannot be disabled MUST be rejected at configure time with `E9011`, naming the compiler and version.
* `[TYP-9b]` `@fp(contract)` on a function permits — and requires the backend to enable — FMA contraction within that function only. It permits no reassociation, no NaN/Inf assumptions and no other `@fastmath` relaxation. Mapping: Clang `#pragma clang fp contract(fast)` around the body; MSVC `#pragma fp_contract(on)` around the definition; GCC, which has no reliable per-function control, MUST emit the function into its own translation unit compiled with `-ffp-contract=fast` — and such a function is therefore **excluded from `[CG-C-3]`'s inline header on GCC**, since inlining it into a non-contracting TU would silently lose the attribute.
* `[TYP-9c]` If the host toolchain cannot honour `@fastmath` or `@fp(…)` at function granularity, the compiler MUST report `E9010` naming the toolchain and the attribute. It MUST NOT silently compile the function under the translation unit's default float control; a silently ignored float-control attribute is the worst outcome, because the programmer believes the contract holds (`[PHIL-6]`).

## IV.8 Interfaces

An `interface` declares required methods, associated types/consts, and may provide default method bodies. Types implement interfaces in their header (`struct X implements I, J:`) or in an `extend X implements I:` block. `[TYP-20]` **Coherence:** an implementation of interface `I` for type `T` may appear only in the module that declares `I` or the module that declares `T` (orphan rule), which keeps resolution local and incremental.

**Marker interfaces derived automatically** (cannot be implemented by hand except via `unsafe extend`):

| Marker | Derived when |
|---|---|
| `Copy` | struct/enum/tuple whose fields are all `Copy`, has no `drop`, and is not `@move_only`; scalars, views, raw pointers, handles, `extern fn` |
| `Send` | all fields `Send`; raw pointers are `!Send`; class handles are `Send` iff the class is `Sync` (Part XI) |
| `Sync` | all fields `Sync`; `ref mut` and interior-mutable types are `!Sync` unless synchronised |
| `Sized` | everything except `dyn I` and `str`/`[T]` (unsized types appear only behind a pointer) |
| `Drop` | type has a `fn drop(mut self)` method or a field that is `Drop` |

**Standard interfaces** that the compiler knows about (spelled as ordinary interfaces in `std.core`):

```ember
interface Clone:
    fn clone(self) -> Self

interface Drop:
    fn drop(mut self)                                      # called exactly once at end of life

interface Eq:
    fn eq(self, other: Self) -> bool                       # ==, !=

interface Ord: Eq:
    fn cmp(self, other: Self) -> Ordering                  # < > <= >=

interface PartialOrd: Eq:
    fn partial_cmp(self, other: Self) -> Option[Ordering]

interface Hash:
    fn hash(self, mut h: Hasher)

interface Default:
    fn default() -> Self

interface Display:
    fn fmt(self, mut f: Formatter) -> Result[void, FmtError]        # f"{x}"

interface Debug:
    fn fmt_debug(self, mut f: Formatter) -> Result[void, FmtError]  # f"{x:?}"

# Sub, Mul, Div, Rem, Neg, BitAnd, BitOr, BitXor, Shl, Shr and Not are declared
# exactly as Add is; each has a matching *Assign form declared as AddAssign is.
interface Add[Rhs = Self]:
    type Output
    fn add(self, rhs: Rhs) -> Output

interface AddAssign[Rhs = Self]:
    fn add_assign(mut self, rhs: Rhs)

interface Index[Idx]:
    type Output
    fn index(self, i: Idx) -> ref Output                   # a[i] read

interface IndexMut[Idx]: Index[Idx]:
    fn index_mut(mut self, i: Idx) -> ref mut Output       # a[i] write

interface Iterator:
    type Item
    fn next(mut self) -> Option[Item]

interface IntoIterator:
    type Item
    type Iter: Iterator[Item = Item]
    fn into_iter(owned self) -> Iter

interface Iterable:
    type Item
    type Iter: Iterator[Item = Item]
    fn iter(self) -> Iter                                  # for x in v

interface IterableMut: Iterable:
    type IterMut: Iterator[Item = ref mut Item]
    fn iter_mut(mut self) -> IterMut

interface Callable[Args, R]:
    fn call(self, args: Args) -> R                         # closures; compiler-implemented

interface Error: Debug + Display:
    fn source(self) -> Option[ref dyn Error]               # default None

interface From[T]:
    fn from(owned value: T) -> Self                        # `?` conversion; Into is blanket
```

`[TYP-21]` Operators desugar to these interface calls with **auto-referencing**: `a + b` calls `Add.add(a, b)` with `a` and `b` passed in the interface's declared modes (both borrowed for `Add` as declared above, i.e. `Vec3 + Vec3` does not consume). `a += b` calls `AddAssign.add_assign` if implemented, else `a = a + b`.

## IV.9 Existentials (`dyn`)

`dyn I` is an unsized type; it is used behind `ref dyn I`, `Box[dyn I]`, or a class handle whose class implements `I` (`I` alone in handle position means "any class handle implementing I"). A fat pointer is `{data*, vtable*}`; the vtable is `{drop, size, align, method0, method1, …}` in interface declaration order plus supertraits' tables. `[TYP-22]` An interface is `dyn`-compatible only if every method has a receiver, no method is generic, and no method returns `Self` by value (except in default methods marked `where Self: Sized`). Violations are `E2050` when `dyn I` is formed.

## IV.10 Type inference

`[TYP-23]` Inference is **local to a function body** and bidirectional:

1. Function signatures, struct fields, statics and consts MUST be fully annotated (return type omitted ⇒ `void`).
2. Locals declared by `x = expr` get the type of `expr`; if `expr` contains unresolved inference variables (e.g. `Array()`), they are solved by later uses within the same function; unresolved at end of body ⇒ `E2060 cannot infer type of x` with the first use highlighted.
3. Expected types flow downward (checking mode) into literals, lambdas, `Array()`, `None`, `Ok(..)`, `Err(..)`, and generic calls.
4. Lambda parameter types are inferred from the expected function type; a lambda with unannotated parameters in a context without an expected type is `E2061`.
5. Untyped literals are resolved last (`[LEX-16/17]`).
6. Method resolution on an inference variable is deferred until the variable is solved; if a method call forces resolution and multiple types are possible, `E2062`.

The algorithm is Hindley–Milner-style unification over a per-function inference table (union–find of type variables with occurs check), with interface-bound obligations collected and solved after unification (Part XVIII §4.4). There is no let-polymorphism for locals.

## IV.11 Method resolution and auto-ref

For `recv.m(args)`:

1. Determine the type `R` of `recv` after stripping `ref`/`ref mut` and class-handle indirection (auto-deref), at most one level of `Box`/`Shared` deref.
2. Look for an inherent method `m` on `R`, then on `R`'s base classes (nearest first), then in interfaces implemented by `R` that are in scope (imported), then in `dyn` vtables. `[TYP-24]` An inherent method always beats an interface method of the same name; ambiguity between two interfaces is `E2070` (disambiguate with `I.m(recv, ...)`).
3. Adjust the receiver to the method's declared mode: `self` → shared borrow (or copy of a handle), `mut self` → mutable borrow (`E3xxx` if `recv` is not mutable/not a mutable place), `owned self` → move (or handle copy for classes).

Named arguments `f(x=1, y=2)` match parameter names; positional arguments MUST precede named; `[TYP-25]` parameters with defaults may be omitted. Overloading by arity or type is **not** supported (use defaults, generics, or distinct names); `[TYP-26]` two functions with the same name in one scope is `E1030` — except operator interface impls and `extend` blocks for distinct types.

---

# Part V — Declarations and Semantics

## V.1 Modules, packages, visibility

* `[MOD-1]` A **package** is a directory tree with an `ember.toml` at its root. A **module** is one `.em` file. Module path = package name + path from `src/` with `/` → `.` and the extension removed; `src/lib.em` (library root) or `src/main.em` (binary root) is the package root module; `src/math/mod.em` is module `math` when a directory has submodules.
* `[MOD-2]` All items are private to their module unless `pub`. `pub(package)` is visible within the package; `pub` is visible to dependants. There is no `protected`; subclasses in other modules see only `pub` members.
* `[MOD-3]` `import a.b.c` binds `c` as a namespace; `from a.b import x, y as z` binds items; `import a.b.c as d` renames. `from a.b import *` is permitted only for `prelude` modules declared `@prelude` (`E1040` otherwise).
* `[MOD-4]` Import cycles within a package are allowed (name resolution is package-wide); cycles between packages are `E1041`.
* `[MOD-5]` `std.prelude` is imported implicitly into every module: `Option, Some, None, Result, Ok, Err, Array, String, str, Span, MutSpan, Box, Shared, Weak, print, println, assert, assert_eq, panic, Copy, Clone, Drop, Eq, Ord, Hash, Debug, Display, Default, Iterator, Iterable, Send, Sync`.
* `[MOD-6]` A module MAY declare `#! language "0.5"` on its first line; the package's `ember.toml` `language` key is the default. Mismatch with the compiler's supported set is `E0006`. The compiler's supported set MUST include every language version whose source it still accepts, so pinning an older version stays valid.
* `[MOD-7]` **Read-only visibility for fields.** A field declared `pub(read)` (or `pub(package, read)`) may be **read** wherever a `pub` (respectively `pub(package)`) field could be read, but may be **written only from the declaring module**. Outside the declaring module, the following are errors `E1050 field is read-only outside its module`: assignment (`h.value = x`, augmented assignment), taking `ref mut h.value`, passing `h.value` to a `mut` parameter or `mut self` method, and destructuring it with a mutable binding. Reading, copying, taking `ref h.value`, and passing it to a borrowed parameter are allowed. `read` applies to fields of structs and classes only; on any other item it is `E1051`. Because construction is a write, a `pub(read)` field does not count as `pub` for the purpose of the synthesised memberwise constructor (`[STR-1]`).

  Summary of field visibility:

  | Declaration | Read from | Write from |
  |---|---|---|
  | `value: T` | declaring module | declaring module |
  | `pub(package) value: T` | package | package |
  | `pub(package, read) value: T` | package | declaring module |
  | `pub value: T` | anywhere | anywhere |
  | `pub(read) value: T` | anywhere | declaring module |
  | `let value: T` (any visibility) | per the marker | nobody after `init`; not even the declaring module |

## V.2 Functions

```ember
pub fn name[T: Bound](a: A, mut b: B, owned c: C, d: D = default) -> R where T: Other:
    body
```

* Parameter modes `[FN-1]`:
  * `a: A` — **borrowed** (shared). The callee reads through a `ref A`. For `Copy` types smaller than 2 pointers the compiler passes by value in registers (ABI detail; semantics identical). The callee cannot mutate or move `a`.
  * `mut b: B` — **inout** (mutable borrow). The argument MUST be a mutable place; the callee may mutate; no move out (except by `mem.replace`/`take`).
  * `owned c: C` — **consumed**. The argument is moved (or copied if `Copy`; retained if a handle). The callee owns it and will drop it or move it on.
  * `[FN-2]` Missing mode is `borrowed`. There is no by-value-copy mode; if the callee wants its own copy it writes `owned` and the caller writes `f(x.clone())` or `f(x)` for `Copy` types.
* `[FN-3]` Return values are moved out; returning a `ref`/view requires that the region be tied to a parameter by elision (Part VII §5).
* `[FN-4]` The receiver `self` follows the same modes: `self` (borrow), `mut self` (mutable borrow), `owned self` (consume). Absent `self` ⇒ associated function. `self: Type` explicit form is allowed for `self: ref Self`, `self: Box[Self]`, `self: Shared[Self]` (v2 for the latter two).
* `[FN-5]` Default arguments are evaluated in the callee's scope at each call, after positional/named binding. They may reference earlier parameters. They MUST be `@noalloc`-clean if the function is `@noalloc`.
* `[FN-6]` Functions are values of a unique zero-sized function type; they coerce to `fn(A) -> R` (the generic callable bound) and, if they capture nothing and have no generic parameters, to `extern "C" fn(A) -> R` when their types are FFI-safe.
* `[FN-7]` Recursion is permitted; the compiler does not guarantee tail-call elimination in v1.
* `[FN-8]` `main` is `fn main()`, `fn main() -> Result[void, E]` (`E: Error`), or `fn main(args: Span[str])` variants. A non-`Ok` result prints the error with `Display` to stderr and exits with code 1.
* `[FN-2a]` The compiler forms the borrow the callee's declared mode requires. If the argument is not a suitable place for that mode, it reports shape **B10** naming the place required. **A call site never writes the mode** (owner decision `OQ-13`): `f(x)` is written whether `f` declares `x`, `mut x` or `owned x`, preserving Part 0 row 3's guarantee that call sites never carry a sigil. The mode is read from the callee's signature, and `ember inspect` and the editor's inlay hints (`[IDE-7]`) surface it at the argument.

## V.3 Structs

```ember
@derive(Copy, Debug, Eq)
pub struct Vec3:
    pub x: f32
    pub y: f32
    pub z: f32

    const ZERO: Vec3 = Vec3(0, 0, 0)

    fn length(self) -> f32:
        return sqrt(self.dot(self))

    fn dot(self, o: Vec3) -> f32:
        return self.x*o.x + self.y*o.y + self.z*o.z
```

* `[STR-1]` Every struct has a synthesised **memberwise constructor** `Name(field0, field1, …)` accepting positional or named arguments; fields with defaults may be omitted. It is `pub` iff all fields are `pub` (a `pub(read)` field makes it private to the declaring module, since construction is a write; `[MOD-7]`). A user-defined `fn init(mut self, …)` replaces it (Part V.5).
* `[STR-2]` Field defaults are const-evaluable expressions or calls to `Default.default()`.
* `[STR-3]` A struct with a `drop` method, or any `Drop` field, is move-only. `Copy` is derived otherwise via `@derive(Copy)` (never implicitly, so that adding a field later cannot silently change semantics — `E2080` if `@derive(Copy)` is present but a field is not `Copy`).
* `[STR-4]` Zero-sized structs are permitted (`struct Marker: pass`).
* `[STR-5]` Struct equality/ordering/hash are never implicit; `@derive(Eq, Ord, Hash)` generates field-wise implementations.

## V.4 Enums

```ember
@repr(u8)
enum RenderMode:
    Forward = 0
    Deferred = 1
    PathTrace = 2

enum Shape:
    Circle(radius: f32)
    Rect(w: f32, h: f32)
    Empty

    fn area(self) -> f32:
        match self:
            Circle(r) => return PI * r * r
            Rect(w, h) => return w * h
            Empty => return 0
```

* `[ENM-1]` Variant constructors are `Shape.Circle(1.0)` or `Shape.Circle(radius=1.0)`; with `from Shape import *`-style implicit scope inside `match`, bare `Circle(r)` patterns are accepted when the scrutinee's type is known.
* `[ENM-2]` `match` on an enum MUST be exhaustive (`E2090` lists the missing variants). `_` is the catch-all.
* `[ENM-3]` Unit-only enums implement `Copy, Eq, Hash, Debug` automatically and support `as` to their repr integer; the reverse uses `Mode.from_repr(x) -> Option[Mode]`.
* `[ENM-4]` Payload enums derive `Copy` only via `@derive(Copy)` with all payloads `Copy`.

## V.5 Classes

```ember
open class Script:
    entity: Entity
    pub(read) enabled: bool = true         # anyone may read; only this module may write

    fn init(mut self, entity: Entity):
        self.entity = entity

    virtual fn on_create(mut self): pass
    virtual fn on_update(mut self, dt: f32): pass
    fn drop(mut self): pass                    # optional destructor; non-virtual, chained automatically

class Door(Script):
    open_angle: f32 = 0.0
    hinge: Weak[Entity]

    fn init(mut self, entity: Entity, hinge: Entity):
        super.init(entity)                     # MUST be the first statement that touches self
        self.hinge = Weak(hinge)

    override fn on_update(mut self, dt: f32):
        self.open_angle = min(self.open_angle + dt, 90.0)
```

* `[CLS-1]` `class` instances live on the heap with the object header defined in Part VIII §1. `Name(args)` allocates, zero-initialises header, runs `init`, and returns a handle with strong count 1.
* `[CLS-2]` **Constructors.** `fn init(mut self, …)` is the constructor. Definite-initialisation analysis (Part XVIII §5.6) requires every field without a default to be assigned on every path before `self` is used as a whole (passed anywhere, method called, escaped). Reading a field before it is assigned is `E2100`. Multiple constructors are not supported by overloading; use defaults or associated functions `fn from_file(path: str) -> Result[Self, E]` that call `Self(...)`.
* `[CLS-3]` If no `init` is declared, the memberwise constructor is synthesised as for structs.
* `[CLS-4]` **Inheritance.** A class is `final` unless declared `open` or `abstract`. `class D(B)` requires `B` to be `open`/`abstract`. Methods are non-virtual unless declared `virtual`; overriding requires `override`; overriding a non-virtual method is `E2110`; `virtual` in a final class is `W2111`. The derived `init` MUST call `super.init(...)` exactly once before using inherited fields. Fields cannot be overridden. Base-class fields are laid out first, so a `D*` is a valid `B*` (single inheritance, no virtual bases).
* `[CLS-5]` A class may implement interfaces; interface methods are dispatched statically unless the receiver is `dyn I`.
* `[CLS-6]` `drop` on a class runs derived-first, then base; then fields are dropped in reverse declaration order; then the memory is released when the weak count is also zero.
* `[CLS-7]` `self` inside a class method is a handle (Copy); `mut self` grants a dynamically checked write access for the duration of the method (Part VIII §3). Storing `self` into another object is allowed and retains (this is how observer patterns work — beware cycles; see Part VIII §5).
* `[CLS-8]` Classes are `Sync` iff every field's type is itself `Sync` (`Atomic`, `Mutex[T]`, `RwLock[T]`, a channel end, or a deeply immutable value type) — Part XI. **`let` does not contribute to this derivation.** Since `OQ-18` a `let` field restrains only the binding, and `[CLS-9a]` permits mutation *through* it, so a `let Array[i32]` field is exactly as unsynchronised as a non-`let` one. Non-`Sync` classes use non-atomic counts and are thread-confined.
* `[CLS-9]` `let` fields: `let name: T` declares an **immutable** field, assignable only in `init`. For a field that the class mutates but outsiders may only read, use `pub(read)` (`[MOD-7]`), not `let`. Immutable fields of `Sync` types keep a class `Sync`.
* `[CLS-7a]` Inside a `drop` body, `self` MUST NOT be copied into a place that outlives the call. The compiler MUST reject the statically visible cases — storing `self`, or a handle-typed projection denoting the same object, into a field of another object, a `static`, a container, an `owned fn` capture, or a `Retained` token — as `E3016 self escapes its own drop`, with `help: a destructor may not publish a handle to the object being destroyed; move the data out with `mem.take` instead`. Cases the compiler cannot see are caught by `[OBJ-5]`.
* `[CLS-9a]` A `let` field of a non-`Copy` type MAY still be mutated *through* by a `mut self` method of the declaring class (`self.items.push(v)` is legal); only assignment to the field itself is restricted to `init`. Code that needs the value frozen must choose a type with no `mut self` API, not `let`. For "outsiders may read, only this module may write", use `pub(read)` (`[MOD-7]`).

## V.6 Interfaces and `extend`

```ember
pub interface Drawable:
    fn bounds(self) -> AABB
    fn draw(self, mut cmd: CommandList)
    fn is_visible(self, frustum: Frustum) -> bool:      # default method
        return frustum.contains(self.bounds())

extend Mesh implements Drawable:
    fn bounds(self) -> AABB: return self.aabb
    fn draw(self, mut cmd: CommandList): cmd.draw_mesh(self)

extend[T: Display] Array[T] implements Display:          # generic impl with bound
    fn fmt(self, mut f: Formatter) -> Result[void, FmtError]:
        ...
```

* `[IFC-1]` `extend T:` without `implements` adds inherent methods to `T` from any module in the same package (or the declaring package of `T`); `[IFC-2]` inherent extension of foreign types from a third package is `E2120` (avoids silent API changes); use a wrapper or an interface.
* `[IFC-3]` Interface inheritance `interface Ord: Eq` requires implementers of `Ord` to also implement `Eq`.
* `[IFC-4]` Associated consts and types are permitted; associated statics are not.

## V.7 Constants and statics

* `const NAME: T = expr` — compile-time constant, inlined at each use, no address. `expr` MUST be evaluable by the comptime interpreter (Part XIV). Type annotation optional if inferable from a literal.
* `static NAME: T = expr` — one instance per program with a stable address, initialised at compile time (comptime-evaluable expression) or lazily on first access if `T` requires runtime construction (`static lazy` v2; in v1 non-comptime statics are `E2130`). `static` values are immutable; `[STA-1]` `static mut` exists and any access requires `unsafe`. Safe mutable globals use `static COUNTER: Atomic[u64] = Atomic(0)` or `static REG: Mutex[Registry] = Mutex(Registry.new())` (both `Sync`, comptime-constructible).
* `[STA-2]` Module initialisation order is a non-issue by construction: there are no runtime initialisers, so the C++ static-initialisation-order problem cannot arise.

## V.8 Attributes on declarations

Attributes precede the declaration, one per line, and are validated by the compiler against the table in Part III §7. Attribute arguments are literals, identifiers, or nested attribute-like forms (`@derive(Serialize(rename_all="camel"))`). `[ATT-1]` An attribute not in the Part III §7 tables and not in a registered plugin namespace is `E0104`. An attribute listed as **reserved** is `E0104` with a message naming the version that will introduce it, so that a reservation is never mistaken for a typo. A derive-scoped attribute outside its derive is `E0104` naming the derive that defines it.

* `[ATT-2]` A statement attribute MUST be one of `@simd`, `@parallel`, `@unroll`, `@allow`. Any other attribute in statement position is `E0104`, naming the four that are permitted there. `[ATT-3]` A statement attribute attaches to the next `compound_stmt` and MUST NOT precede a `simple_stmt` (`E0108`). `@simd`, `@parallel` and `@unroll` additionally require that statement to be a `for_stmt` (`E0108`).
* `[ATT-4]` One attribute per line. The `attribute` production carries a mandatory `NEWLINE`, matching Part V §8 and the formatter.

---
# Part VI — Expressions and Statements

## VI.1 Evaluation order

* `[EXP-1]` Operands, arguments, array/tuple elements and struct constructor arguments are evaluated **left to right**, fully, before the operation/call. Named arguments are evaluated in source order, not parameter order.
* `[EXP-2]` Assignment `lhs = rhs` evaluates `rhs` first, then the place expression `lhs` (index/field sub-expressions), then stores. For `a[i] = f(i)`, `f(i)` runs before `i` is read for the index. Augmented assignment `a[i] += x` evaluates `x`, then the place once, then reads, ops, writes.
* `[EXP-3]` `and`/`or` short-circuit; `x if c else y` evaluates `c` first and exactly one branch.
* `[EXP-4]` Temporaries created during an expression statement are dropped at the end of that statement in reverse creation order; temporaries bound by `with`/`for`/`if`-conditions live to the end of the construct.

## VI.2 Places and values

A **place expression** denotes memory: a local, a field of a place, an index into a place, a dereference of a `ref`/`ref mut`/`Box`, a class-handle field access (`h.f` is a place inside the object), a `static`. Everything else is a value expression. `[EXP-5]` Mutation, mutable borrow and move require a place; `f(x).y = 1` is `E2140` unless `f` returns `ref mut`.

**Moves:** a place of non-`Copy` type used as a value (assignment RHS, `owned` argument, `return`, closure capture by `owned fn`) is **moved**; the source becomes uninitialised until reassigned. `[EXP-6]` Moving out of a field of a struct that has `drop` is `E3010`; moving out of an element of an array/span is `E3011` (use `mem.take`, `swap`, `pop`, `drain`); moving out of a class-object field is `E3012` (classes may be aliased); moving out of a `ref`/`ref mut` is `E3013`. Moving out of a plain struct's field is allowed and leaves the struct partially moved; it cannot be used as a whole until the field is reassigned.

## VI.3 Operators

| Expression | Desugar | Notes |
|---|---|---|
| `a + b` etc. | `Add.add(a, b)` | scalars are built-in |
| `a == b` | `Eq.eq(a, b)` | `!=` is `not (a == b)` |
| `a < b` | `Ord.cmp(a,b) == Less` or `PartialOrd` | floats use `PartialOrd`; `NaN` comparisons are `false` |
| `a is b` | handle identity (same object) | only for class handles and `ref`s; `E2150` otherwise |
| `x in coll` | `coll.contains(x)` | `Contains` interface |
| `a[i]` | `Index.index(a, i)` / `IndexMut.index_mut(a, i)` | selected by context (read vs write/mut-borrow) |
| `a[i..j]`, `a[..j]`, `a[i..]` | `a.slice(range)` → `Span`/`MutSpan` | bounds-checked; `..` and `..=` |
| `a?.f`, `a?.m()` | `match a: Some(v) => Some(v.f), None => None` | on `Option`; also `Result` (maps `Ok`) |
| `e?` | early-return on `None`/`Err(e)` with `From` conversion for errors | function return type MUST be `Option`/`Result` |
| `x as T` | numeric/pointer cast (`[TYP-6]`) | |
| `h as? D` / `h as! D` | dynamic downcast of class handle | `Option[D]` / panic |
| `**` | `pow` | integer `**` with negative exponent is `E2151` |
| `f"…{e:spec}…"` | `Formatter` calls on `Display`/`Debug` | allocates a `String`; `E4001` in `@noalloc` (use `format_to(buf, …)`) |

`[EXP-7]` Integer division truncates toward zero; `%` has the sign of the dividend (C/Rust semantics). `div_euclid`/`rem_euclid` exist.

## VI.4 Control flow

* `if`/`elif`/`else`, `while`, `for x in iterable`, labeled `break label`/`continue label`, `match`, `with`, `defer`, `return`.
* `[CTL-1]` `for pattern in expr` desugars to:
  ```
  __it = IntoIterator.into_iter(expr)   # for a place expression of non-Copy type: Iterable.iter(expr) (borrowing)
  while Some(pattern) = __it.next():
      body
  ```
  The **borrowing** rule: `for x in v` where `v` is a place borrows `v` (yields `ref T`); `for x in v.iter_mut()` yields `ref mut T`; `for x in owned v` consumes. `[CTL-2]` The iterable is borrowed for the whole loop; mutating it inside the loop is a borrow error `E3020` (the diagnostic suggests index loops or `retain`/`drain`).
* `[CTL-3]` Ranges `a..b` (`Range[T]`), `a..=b` (`RangeInclusive[T]`), `a..` (`RangeFrom`) implement `Iterator` for integer `T` and compile to a counted loop with no iterator object in memory (guaranteed by MIR lowering of `for` over range literals — `[CTL-3a]` conformance test checks the C output has no struct temporaries).
* `[CTL-4]` `while cond: … else: …` — `else` runs if the loop exits without `break`.
* `[CTL-5]` `match` semantics: arms tested top to bottom; first match wins; guards `if` evaluated after binding; exhaustiveness required (`E2090`); unreachable arm is `W2091`. Binding modes per `[GRM-13]`.
* `[CTL-6]` `with a = e1, b = e2:` binds `a`, `b` for the block and drops them (reverse order) at block exit. `with e:` without a binding evaluates `e` and keeps the temporary alive for the block (used for guards: `with lock.acquire():`).
* `[CTL-7]` `defer:` registers a block to run at scope exit (LIFO). A `defer` block cannot `return`, `break`, or `continue` out of the enclosing function (`E2160`). It may reference locals declared before it (borrowing them until scope end).
* `[CTL-8]` `return` inside `with`/`defer`-carrying scopes runs the deferred blocks and drops locals in the correct order (drops happen after `defer` blocks of the same scope).
* `[CTL-9]` `pass` is a no-op statement required for empty blocks.
* `[CTL-3b]` **Guaranteed iteration lowering.** The guarantee of `[CTL-3]` — a counted loop, no iterator object in memory, no `next` call in the emitted code — MUST extend to iteration over `Span[T]`, `MutSpan[T]`, `Array[T]`, `[T; N]`, each `SoA[T]` column and the `SoA[T].Ref`/`RefMut` proxy forms, and to `enumerate`, `zip`, `take` and `skip` composed over any of those. MIR lowering MUST rewrite these into an induction-variable loop over the underlying base pointer(s) and length(s), so the guarantee does not depend on the host compiler's inlining or scalar replacement. `rev`, `chunks`, `chunks_mut` and `Query[…]` are **not** covered in v1: `[ECS-3]` specifies `Query` iteration as "iterate the smallest storage and probe the others", which is not a contiguous base-plus-length walk and needs its own lowering rule (v0.5). Iteration over any other `Iterator` implementation retains the ordinary `next`-call lowering and no guarantee.
* `[CTL-3c]` `tests/conformance/CTL-3b/` MUST contain, for each covered form, a program whose emitted C is asserted (in M1's style) not to contain a call to that form's `next` symbol and not to declare a struct type for the iterator. Part I §5's `integrate` MUST be among them.
* `[CTL-0]` The condition of `if`, `elif`, `while`, and a `match` arm guard MUST have type `bool`. There is no truthiness conversion. `E2035 condition must be `bool`` MUST carry a type-directed fix-it: for a container, `String` or `str`, `not xs.is_empty()`; for `Option`/`Result`, `xs.is_some()`; for an integer, `x != 0`; for a raw pointer, `not p.is_null()`.

## VI.5 Closures

```ember
scale = 2.0
double = fn(x: f32) => x * scale          # borrows `scale`
adder  = fn(mut acc: Array[f32], x: f32): acc.push(x)   # block form
task   = owned fn() => process(data)      # captures `data` by move/copy/retain; may escape
```

* `[CLO-1]` A closure's type is a unique anonymous struct type implementing `Callable`. It is a **view type** if it captures anything by reference (the default). It is a plain value type if declared `owned fn` (captures by move/copy/retain) or captures nothing.
* `[CLO-2]` Capture mode is inferred per variable: read-only use ⇒ shared borrow; mutation ⇒ mutable borrow (the closure then requires a mutable place to call: `mut f`); `owned fn` ⇒ move (or copy/retain). A closure that moves a captured non-`Copy` value out of its own storage implements `CallableOnce` but not `Callable` (`[CLO-6]`).
* `[CLO-3]` Calling: `f(args)`. A parameter declared `f: fn(A) -> R` is a generic over `Callable` (static dispatch, monomorphised). A boxed dynamic closure is `Box[dyn fn(A) -> R]` (`E4001` in `@noalloc` because boxing allocates). An `extern "C" fn` parameter accepts only capture-free closures and named functions.
* `[CLO-4]` A non-`owned` closure cannot escape the scope of what it borrows: storing it, returning it, or passing it to a function whose parameter is `owned`/stored triggers the normal view-type rules (`[TYP-15]`).
* `[CLO-5]` Closures capturing class handles retain them (a strong reference) — the usual cycle caution applies (Part VIII §5).
* `[CLO-6a]` **`CallableOnce` is not `dyn`-compatible in v1**: `call_once` takes `owned self`, which `[TYP-22]` does not admit through a vtable. `[CLO-3]`'s `Box[dyn fn(A) -> R]` therefore remains a `Callable`, and a boxed once-callable payload is expressed by moving the payload into the closure's captures and having the boxed closure take it by `mem.take` from an `Option` field — the one place the `Option` dance survives, and the reason `[TYP-22]`'s by-value-self restriction is worth revisiting in v2.
* `[CLO-7]` `thread.spawn`, `jobs.submit`, `jobs.submit_after`, `Option.map`/`and_then`/`unwrap_or_else`, and `Result.map`/`map_err`/`and_then`/`unwrap_or_else` MUST declare their callable parameter `owned f:`. `[JOB-1]`'s inline job slot is unaffected: the closure remains a value moved into the slot.
* `[CLO-6]` **Once-callable closures.** `std.core` declares `interface CallableOnce[Args, R]: fn call_once(owned self, args: Args) -> R`, and `interface Callable[Args, R]: CallableOnce[Args, R]`, so every reusable closure is also once-callable. A parameter written `owned f: fn(A) -> R` is a generic bounded by `CallableOnce`; `f: fn(A) -> R` and `mut f: fn(A) -> R` are bounded by `Callable`. **No call site changes**: the mode already selects the bound, exactly as `[FN-1]`'s three modes already work for every other type. Calling a value bounded by `CallableOnce` consumes it; a second call is `E3040 use of moved value` under `[OWN-3]` and requires no additional analysis. `E3030` is emitted only when a closure implementing `CallableOnce` alone is supplied where `Callable` is required, and MUST use shape O5's help.

## VI.6 Assertions and panics

* `assert(cond)`, `assert(cond, "msg")`, `assert_eq(a, b)`, `assert_ne(a, b)`: always compiled in every profile. `debug_assert*` compiled only in `debug`.
* `panic("msg")`, `panic(f"…")`, `unreachable()`, `todo()`: type `!`.
* `[PAN-1]` Panic policy is a **package-level** setting: `abort` (default; prints message + backtrace in debug, message only in shipping, then `abort()`), or `unwind` (v2, LLVM backend only): runs `defer` blocks and drops in reverse order up to a `catch_unwind` boundary or the thread root. Foreign frames are never unwound through (`[FFI-*]`).
* `[PAN-2]` The panic message is formatted **without allocating** when the argument is a literal or `str`; with an f-string it allocates (so `@noalloc` functions may only `panic` with literals — `E4001` otherwise).
* `[PAN-3]` A panic inside `drop` while another panic is in flight aborts immediately.

---

# Part VII — Ownership, Borrowing and Lifetimes

This part specifies the **value world**. Class instances (the object world) are covered in Part VIII; the rules here apply to class *handles* as `Copy` values and to *accesses through* handles as described there.

## VII.1 Ownership

* `[OWN-1]` Every value has exactly one owner: a local variable, a field of an owned value, an element of an owned container, a temporary, or a `static`.
* `[OWN-2]` When the owner goes out of scope (block end, reverse declaration order), or is overwritten, or is a temporary at statement end, the value is **dropped**: its `drop` method (if any) runs, then its fields are dropped in reverse declaration order (arrays: elements in index order).
* `[OWN-3]` Moves transfer ownership; the source is dead. Use of a moved-from place is `E3040 use of moved value` with two labels (the move site, the use site) and a `help` suggesting `.clone()` if `Clone`. Conditional moves are handled by **drop flags** (a hidden `bool` per local that is conditionally moved), never by making the program illegal.
* `[OWN-4]` A loop body that moves a value declared outside the loop is `E3041` unless the value is reassigned before the next iteration on every path.
* `[OWN-5]` Overwriting a place that holds a live value drops the old value first (after evaluating the new value: `x = f(x)` moves `x` into `f`, then stores).
* `[OWN-6]` `mem.take(mut place) -> T` (replaces with `Default`), `mem.replace(mut place, new) -> T`, `mem.swap(mut a, mut b)`, `mem.forget(owned x)` (skips drop; leaking is not unsafe **except for `@must_drop` types, which it rejects — `[THR-6]`**).

## VII.2 Copy

* `[OWN-7]` Values of `Copy` types are duplicated bitwise on use; the source remains valid. A `Copy` type has no `drop`. Class handles are `Copy` at the language level but their copy performs a retain and their drop a release (Part VIII); they are excluded from `[TYP-15]`-style bitwise guarantees (the compiler emits the RC ops).
* `[OWN-8]` `Clone.clone(self) -> Self` is the explicit deep copy for non-`Copy` types. `@derive(Clone)` generates field-wise clones. Cloning a class handle is the same as copying it (shallow); deep-copying an object requires an explicit method.

## VII.3 Borrows

A **borrow** creates a reference (`ref T` or `ref mut T`) — or a view containing one — to a place, without transferring ownership. Borrows are created:

* implicitly, when passing a place to a `borrowed` (default) or `mut` parameter, calling a method with `self`/`mut self`, iterating with `for`, or using an operator whose interface takes borrows;
* explicitly, with the keyword forms `ref place` / `ref mut place` (needed only when initialising a `ref`-typed local or a view struct field).

The rules (`[BRW-*]`) are Rust's, restated:

* `[BRW-1]` **Aliasing XOR mutability.** At any program point, a place may have either any number of live shared borrows, or exactly one live mutable borrow, and while a mutable borrow is live the owner may not read, write, move, or drop the place; while shared borrows are live the owner may read (and copy) but not write, move, or drop.
* `[BRW-2]` **Liveness (NLL).** A borrow is live from its creation until the last use of any value derived from it (the reference itself, a reborrow, a view built from it, a return value tied to it). Scope end is irrelevant. This is what makes `n = v.len(); v.push(n)` legal.
* `[BRW-3]` **Two-phase borrows.** For `v.push(v.len())`, the mutable auto-borrow of `v` for the receiver is *reserved* first and *activated* only when the call happens; shared borrows in the arguments are permitted in between. This applies to method receivers and `mut` arguments whose argument expression is a simple place.
* `[BRW-4]` **Disjoint fields.** `ref mut a.x` and `ref mut a.y` may be live simultaneously if `x` and `y` are distinct fields of a struct/tuple (not through a method call — a method takes all of `self`). This holds through arbitrary nesting of field projections. It does **not** hold through class-handle access (`h.x` and `h.y` are accesses on an aliased object: Part VIII §3 governs) — for *conflicting accesses*. Keeping the object allocated is governed by `[RC-5]`, which applies to class-handle projections exactly as to any other place.
* `[BRW-5]` **Indices are not disjoint.** `ref mut a[i]` and `ref mut a[j]` conflict unless both indices are constants and different. `split_at_mut`, `chunks_mut`, `iter_mut` and `SoA` column borrows are the sanctioned ways to obtain multiple mutable borrows into one container.
* `[BRW-6]` **Reborrows.** From `r: ref mut T` one may create `ref r.f` (shared reborrow, freezing `r` for its duration) or `ref mut r.f` (mutable reborrow, suspending `r`). Passing a `ref mut` local to a `mut` parameter reborrows rather than moving it.
* `[BRW-7]` **No borrow of a moved or uninitialised place.** `E3050`.
* `[BRW-8]` A shared borrow of a `Copy` place may be replaced by a copy when the callee's parameter is `Copy` and ≤ 16 bytes; this is an ABI decision and never observable.

## VII.4 What a reference can point to

`ref T` points to a **place** that outlives the reference's region. Places include locals, fields, array elements, `Box` contents, class-object fields (subject to Part VIII), arena allocations, `static`s, and memory behind a `Span`. A reference can never be null, dangling, or unaligned in safe code (`[BRW-9]`).

## VII.5 Regions (lifetimes) and their inference

Ember does not have user-written lifetime names in v1. Instead:

* Every reference-typed or view-typed value has an implicit **region** — the set of program points where it must be valid.
* `[LT-1]` **Signature elision.** In a function signature with view-typed parameters and a view-typed return:
  1. if there is a `self`/`mut self` receiver that is a borrow, the return's region is the receiver's;
  2. else if exactly one parameter is view-typed, the return's region is that parameter's;
  3. else, the return's region is the **intersection** of all view-typed parameters' regions (the returned reference may point into any of them; the caller treats it as borrowing all of them). This is more permissive than Rust's elision failure and remains sound.
  `@borrows(param)` on a function overrides rule 3 to tie the return to one named parameter (e.g. `fn longest(a: str, b: str) -> str @borrows(a)`), which lets the caller keep using `b`.
* `[LT-2]` **View structs** have one region parameter. Constructing a view struct from several references gives it the intersection of their regions. A view-typed field's region is the struct's region. Multiple independent regions inside one struct are not expressible in v1; nest structs or copy data.
* `[LT-3]` **Static region.** String literals, `static` items, and `Span`s over them have the `static` region, which outlives everything and satisfies `[TYP-15]`'s storage restrictions (a `str` literal *may* be stored in a class field because its type is `str` with static region — the compiler records region `static` in the field's type; a non-static `str` cannot be stored there: `E3060 stored view may not outlive its source`).
* `[LT-4]` **Arena region.** `Arena` allocations return `ref mut T`/`MutSpan[T]` whose region is the arena's borrow; they cannot outlive the arena (`E3061`).
* `[LT-5]` **Inference.** Regions are inferred by the NLL algorithm in Part XVIII §5.7 for all locals; the programmer never writes them. Diagnostics report regions in terms of "the borrow of `x` on line N is still needed on line M".
* `[LT-1a]` **Explicit return region.** The attribute `@borrows(p₁, …, pₙ)`, written on its own line preceding the function declaration, overrides the region that rules 1–3 would assign to a view-typed return. It MUST name one or more parameters; the receiver is named as `self`. The return's region is the intersection of the named parameters' regions, and every unnamed view-typed parameter is NOT borrowed by the return, so the caller MAY continue to use it. `@borrows` overrides **all three** elision rules, **including rule 1**: a method whose result points into an argument rather than into its receiver MUST be written with `@borrows` naming that argument. Naming a parameter that is not view-typed, or writing `@borrows` on a function whose return is not view-typed, is `E2031`. If the returned value's region is not a subset of the intersection of the named parameters' regions, the body is rejected with `E3062` (shape B6).  `@borrows` is part of a function's public contract for compatibility purposes: widening it (naming fewer parameters) is a breaking change under `[VER-2]`, and an `override` of a `virtual` method MUST NOT name a superset of the base's parameters — the same inheritance rule as `[EFF-8]`.
* `[LT-1b]` **Intersection is reported at the definition.** When rule 3 assigns a view-typed return the intersection of two or more view-typed parameters' regions and the function carries no `@borrows`, the compiler emits the **opt-in lint** `L3014 return region is the intersection of N parameters` at the function's declaration, listing every parameter the caller will be unable to use while the result is live, and naming `@borrows` as the fix. Writing `@borrows` — including `@borrows` naming every view-typed parameter, which expresses the intersection deliberately — silences it. `[LT-1]`'s permissiveness is unchanged. `[LT-2a]` `L3014` is emitted likewise for a `@view struct` whose region is the intersection of two or more view-typed fields' regions.
* `[LT-7]` **Late-bound callback regions.** A callback-taking API MAY expose a callback boundary whose borrow region is chosen by the callee for each invocation. The callback's borrowed arguments are valid only for that invocation unless the API separately returns or transfers an owned value. The compiler models this boundary with one level of higher-ranked quantification: the callback is valid for any region selected by the callee at the call boundary. Region variables remain compiler-internal and MUST NOT appear in Ember source, public generic arguments, or ABI-visible type names. Implementations MUST reject any callback-local view that escapes the callback's selected region. `[THR-5]`'s and `[JOB-2]`'s scope regions are instances of this rule rather than bespoke exceptions, and `[GPU-9]`'s `pass.native(fn(cmd) => …)` is expressible under it.

`[LT-6]` Explicit named lifetimes (`fn f['a, 'b](…)`) are **reserved for v2**; the grammar reserves the `'ident` token form (`E0007` in v1 with the message "named lifetimes are not supported in this version; restructure using @borrows or a view struct"). The message MUST name `@borrows` only when a parameter exists whose region is the intended one; where the intended region is the callee's own body, the message MUST say that the construction is not expressible in v1 and name the view-struct restructuring instead.

## VII.6 Drops, `Drop` and destruction order

* `[DRP-1]` `fn drop(mut self)` is invoked exactly once per value at the end of its life. It may not be called explicitly (`E3070`); use `mem.drop(owned x)` to drop early.
* `[DRP-2]` Locals drop at the end of their block in reverse declaration order; struct fields after the struct's `drop`, in reverse declaration order; array elements in index order; enum payload of the active variant; tuple elements in reverse.
* `[DRP-3]` Temporaries drop at the end of the enclosing statement (`[EXP-4]`).
* `[DRP-4]` `drop` bodies MUST NOT panic in `abort` mode without accepting process termination; they SHOULD NOT block. `@noalloc` on a type's `drop` is honoured transitively.
* `[DRP-5]` A `drop` method's `mut self` may not move fields out (`[EXP-6]`); use `Option`/`mem.take` for fields that must be moved during destruction.
* `[DRP-6]` Drop of a `Box[T]` drops `T` then frees; drop of a class handle releases; drop of `Shared[T]` releases; drop of a view does nothing.

## VII.7 Views over containers: `Span` and `MutSpan`

```ember
fn normalize(mut xs: MutSpan[f32]):
    total = xs.iter().sum()
    for x in xs.iter_mut():
        x /= total                    # x: ref mut f32 — writes through

buf = Array[f32]([1, 2, 3])
normalize(buf.as_mut_span())          # or simply normalize(buf) — Array coerces to MutSpan at a `mut` site
left, right = buf.as_mut_span().split_at(1)   # two disjoint MutSpans
```

* `[SPN-1]` `Array[T]` coerces to `Span[T]` at borrow sites and to `MutSpan[T]` at `mut` sites; `[T; N]` likewise; `String` to `str`.
* `[SPN-2]` Indexing a `Span` is bounds-checked; `get(i) -> Option[ref T]` is the checked-without-panic form; `unsafe: s.get_unchecked(i)`.
* `[SPN-3]` `Span[T]` is `Copy`; `MutSpan[T]` is move-only and reborrowable (`s.reborrow()` or implicit at `mut` sites).

## VII.8 Interaction summary (what beginners see)

The rules above are designed so that ordinary code needs no annotations:

```ember
fn total_health(players: Array[Player]) -> f32:      # borrows the array (no `&`)
    sum = 0.0
    for p in players:                                 # p: ref Player… but Player is a class ⇒ p is a handle copy
        sum += p.health
    return sum

fn heal_all(mut players: Array[Player]):             # `mut` ⇒ inout
    for p in players:
        p.health = 100.0                              # handle access; fine (Part VIII)

fn take(owned players: Array[Player]) -> usize:      # consumes
    return players.len()                              # array dropped here (handles released)

fn main():
    ps = Array[Player]()
    ps.push(Player("a"))
    print(total_health(ps))                           # borrowed, still usable
    heal_all(ps)                                      # mutably borrowed, still usable
    n = take(ps)                                      # moved
    print(ps.len())                                   # E3040: use of moved value `ps` (moved on the line above)
```

---
# Part VIII — Classes and Reference Counting (the object world)

## VIII.1 Object representation

Every class instance is a heap block:

```
offset  size   field
0       4      strong_count   u32   (non-atomic if the class is !Sync, atomic if Sync)
4       4      weak_count     u32   (same atomicity)
8       4      access_state   u32   (dynamic exclusivity: bit31 = writer, bits0..30 = reader count; !Sync classes only)
12      4      flags          u32   (bit0 = is_deinitialising, bit1 = pinned/foreign-retained, rest reserved)
16      8      type_info      *const TypeInfo   (vtable, size, align, drop fn, class name, base chain, interface tables)
24      …      base-class fields, then own fields, each at its natural alignment
```

* `[OBJ-1]` The header is 24 bytes on 64-bit targets and is ABI-stable within a compiler minor version. Foreign code never inspects it; it goes through `ember_rt` functions.
* `[OBJ-2]` A handle is a pointer to offset 0. A `dyn I` handle to a class is *not* a fat pointer: the vtable is reached through `type_info` (this keeps `Option[Handle]` one pointer wide and makes class handles ABI-compatible with `void*`).
* `[OBJ-3]` `weak_count` starts at 1 for the "strong side"; when `strong_count` hits 0 the object is deinitialised (`drop` chain + field drops) and `weak_count` is decremented; the block is freed when `weak_count` hits 0. (Rust `Rc` scheme; avoids a separate control block.)
* `[OBJ-4]` Allocation uses the runtime's global allocator (`ember_alloc(size, align)`), default `mimalloc`; the class attribute `@allocator(Name)` selects an alternative for all instances of that class (v2).
* `[OBJ-5]` `ember_rt_deinit(obj)` MUST set `flags.is_deinitialising` before invoking the drop chain and MUST, after the drop chain returns and **again after every field's drop glue has run**, re-read `strong_count`. If it is not zero the object has been resurrected: the runtime MUST panic `E-panic: object resurrected during drop: <Class>` in every profile. No profile setting may remove this check (`[PRF-1]`). The second read is required because a field's drop glue runs after the user `drop` returns and can itself reach the object; it cannot succeed by any other route, because at `strong_count == 0` no strong handle exists and `[WK-3]` blocks `Weak.upgrade`.

## VIII.2 Reference-count operations and their elision

* `[RC-1]` Copying a handle emits `retain`; dropping a handle emits `release`; `release` reaching zero calls `ember_rt_deinit(obj)`.
* `[RC-2]` **Guaranteed elisions** (conformance-tested): (a) passing a handle to a `borrowed` parameter emits no RC ops; (b) a handle read from a place and used only within a single expression emits no RC ops when the place is not written during the expression; (c) `retain` immediately followed by `release` on the same handle with no intervening call or store is removed; (d) a handle stored into a field from a temporary is moved, not retained+released. Its lettered clauses are individually citable as `[RC-2a]`..`[RC-2d]` in the order written.
* `[RC-3]` Additional elisions (`retain` sinking, `release` hoisting across calls proven not to release the object) are permitted only when semantics-preserving; `[PHIL-5]` applies. Elisions never change *when* `drop` runs relative to observable side effects except that an object may be deinitialised **earlier** than the last syntactic use of a handle whose value is provably not needed and no loan derived from that object is in scope (`[RC-5]`) — the compiler treats a handle exactly like any other `Copy` value with a `drop`. A borrow *into* the object is a use of the handle (`[RC-5]`); `mem.keep_alive(h)` is therefore needed only to extend liveness past the last borrow, never to protect a live borrow. Programs MUST NOT depend on an object staying alive past the last use of its handles (identical to Swift; a `with h:` or `mem.keep_alive(h)` pins it explicitly).
* `[RC-4]` Non-`Sync` classes use plain loads/stores for counts; `Sync` classes use relaxed-increment / acquire-release-decrement atomics.
* `[RC-5]` A loan whose place chain contains a `Deref` of a class handle constitutes a **shared loan of the handle operand itself**. `ref h.f`, `ref mut h.f`, a `Span`/`MutSpan` derived from `h.f`, a `Ref[T]`/`RefMut[T]` guard obtained from a `RefCell` field of `h`, and any view struct built from these, all keep `h` borrowed for `[BRW-1]` purposes until the last use of any value derived from them. The compiler MUST NOT kill that loan earlier, MUST NOT sink the handle's `release` above it, and MUST report `E3060` (shape B7) if the handle's storage ends first. XVIII §4.7 **step 2** generates the extra loan when lowering a projection through a handle.
* `[RC-2e]` **Loop-borrowed handles.** Iterating a place whose element type is a class handle, `Shared[T]`, or a value type containing either — `for h in xs`, `for h in xs.iter()`, `for a, b in q` — emits **no** retain and no release on the yielded handles, provided the container is borrowed for the whole loop per `[CTL-2]` and the loop body does not store the handle into memory, return it, pass it to an `owned` parameter, or capture it in an `owned fn`. The yielded value is a borrowed handle whose region is the loop's borrow of the container, exactly as if it had been passed to a borrowed parameter under `[RC-2a]`. Where the body does one of the excluded things, the retain is emitted at that use, not at the top of the iteration. Like `[RC-2a]`..`[RC-2d]`, this elision is guaranteed and conformance-tested by counting `ember_retain` in the emitted C.
* `[RC-6]` **RC operations are optimisation barriers, and this MUST be visible.** `ember_retain` and `ember_release` write through the object pointer; a host C compiler cannot prove the header word distinct from the object's fields, so a surviving RC operation inside a loop invalidates loop-invariant loads of that object's fields and prevents vectorisation. `ember inspect` MUST list, per function, every retain and release that survives inside a loop body, with the source span, the loop, and the reason the elision analysis did not remove it, drawn from `[EFF-11]`'s vocabulary. The `Elided: RC` line of `ember inspect --safety` gains a corresponding `Surviving: RC` section.

## VIII.3 Exclusivity (aliased mutation)

Because handles alias, two references to the same object may be live at once. Ember enforces Swift's **law of exclusivity** dynamically for the accesses that can actually cause memory unsafety:

* An **instantaneous access** — reading or writing a single scalar/`Copy` field through a handle (`h.x = 1.0`, `y = h.x`) — is performed directly with no check. It cannot leave a dangling reference.
* A **long-term access** — any of: calling a `mut self` method, passing `h.field` to a `mut` parameter, taking `ref`/`ref mut` to `h.field` or to a sub-place of it, iterating `h.field` with `for` — begins an access on the object at its start and ends it at its last use (NLL), and:
  * `[EXC-1]` beginning a **write** access while any access (read or write) is active on that object panics with `E-panic: exclusivity violation: overlapping mutable access to <Class>.<field>` in every profile unless the package sets `exclusivity = "unchecked"` (shipping only, and then it is UB). The setting reaches dynamic **class** exclusivity and nothing else (`[CELL-9]`).
  * `[EXC-2]` beginning a **read** access while a write is active panics likewise;
  * `[EXC-3]` The compiler MUST NOT elide the dynamic check on the ground that all *statically visible* accesses to an object go through one handle local. An access A on object O, opened at point p and closed at point q, MAY be elided only if **both**: (a) every long-term access to O in [p, q) that the compiler can see goes through the same handle local as A, that local is not reassigned in [p, q), and conflicting accesses among them are rejected at compile time (`E3080`); **and** (b) no point in [p, q) can begin an access to O that the compiler cannot see. (b) holds when either (b1) the interval contains no call, no virtual or interface dispatch, and no call through a function value; or (b2) escape analysis (`[OPT-1]`) proves that at p, O is reachable from exactly one live handle, that handle is the local named in (a), and no statement in [p, q) stores a handle to O into memory or passes one to a call. Otherwise the compiler MUST emit `begin_access`/`end_access` even though the check in (a) succeeded.
  * `[EXC-4]` An **instantaneous** read of a `let` field — one that copies the field's value, so that no reference to it outlives the read — never begins an access. A **long-term** access to a `let` field begins an access exactly as for a non-`let` field: taking `ref`/`ref mut` to it or to a sub-place of it, iterating it, passing it to a `mut` parameter, or calling a `mut self` method on it. `let` restricts assignment to the field; it does not freeze the value the field holds (owner decision `OQ-18`, resolution A).
  * `[EXC-5]` Nested accesses through the *same* method's `mut self` are statically permitted (they are reborrows).
  * `[EXC-3a]` Every elision MUST be recorded in the safety side table (`[EFF-10]`) with reason `closed_interval` (b1) or `unique_handle` (b2). `ember inspect --safety --elided-only` MUST report it. An implementation that cannot name the condition MUST NOT elide.
  * `[EXC-6]` **Both ends of a conflict MUST be reported.** In the `debug` profile the runtime maintains a per-thread stack of active long-term accesses, each entry `{object, kind, source location}`, pushed by `begin_access` and popped by `end_access`. The panic message required by `[EXC-1]`/`[EXC-2]` MUST name the location of both the offending access and the active access it conflicts with, and MUST carry a `help`:  ``` E-panic: exclusivity violation: write access to Node.children at src/scene.em:31:9 while a read access begun at src/scene.em:22:13 is still active help: hoist the handle to a local so the accesses are ordered statically (EXC-3), or defer the mutation to a command buffer applied after the walk (see ECS-5) ```  In `release`, the mandated `help` text is required unconditionally, and the second location is required only where the implementation can carry it at **no additional per-access cost** — a single static location pointer stored alongside `access_state`, never a stack. Recording the full stack in `release` is an opt-in profile key whose cost MUST be measured under `[BEN-*]` before it is made mandatory. In `shipping` with `exclusivity = "unchecked"` nothing is recorded and no check is performed. **The header layout of `[OBJ-1]` is unchanged.**
  * `[EXC-7]` The lint `L3013 long-term access held across a call` fires when a long-term access on a class object is live across a **virtual or `dyn`** call within one `open class` hierarchy whose reachable call graph (`[EFF-1]`) is not proven free of accesses to the same class type. It names the field and the call, mirroring `L3011`. It is a lint, never an error, and is **off by default**.

Cost: an uncontended `begin_access` is a load, a compare and a store on a header word; `end_access` a decrement. **No fixed figure is normative** (owner decision `OQ-16`): the cost is re-measured under `[BEN-1]`–`[BEN-7]` and published, because `[EXC-3]`'s closed-interval restriction, `[EXC-6]`'s location recording and `[FFI-33]`'s thread assertions all add work to this path. `ember inspect --safety` reports each access as `STATIC`, `ELIDABLE` or `DYNAMIC`. Classes are not intended for inner loops; see the DOD facilities (Part XII) for those.

## VIII.4 Inheritance and dispatch

* `[DSP-1]` Static dispatch is used whenever the receiver's static type is a `final` class or the method is non-`virtual`.
* `[DSP-2]` Virtual dispatch loads `type_info->vtable[slot]`; vtable slots are assigned in declaration order, base class first; `override` reuses the base slot.
* `[DSP-3]` Interface dispatch on a class handle typed as an interface `I` loads `type_info->itable(I)` via a small per-class array of `{interface_id, table*}` pairs searched linearly (classes implement few interfaces); the compiler caches the lookup in a hidden local when the same handle is used repeatedly.
* `[DSP-4]` `h as? D` walks `type_info->base` chain; `is` for classes compares pointers.
* `[DSP-5]` Devirtualisation: with whole-program knowledge (LTO / single package) the compiler may replace a virtual call with a direct call when exactly one implementation is reachable; this is an optimisation and must be reported in `--emit-optimization-report`.

## VIII.5 Weak handles and cycles

* `Weak[C]` does not keep the object alive. `w.upgrade() -> Option[C]`; `Weak(h)` creates one; `Weak[C].empty()`.
* `[WK-1]` Reference cycles between class instances leak (they are never collected). This is a documented property, not a bug. Mitigations the language provides: `Weak` for back-pointers; the debug runtime's **cycle report** (`ember run --leak-check`), which on process exit walks all live objects (the runtime keeps an intrusive list of live objects in debug builds) and prints any strongly connected components with the field names forming the cycle; and the lint `L3001 potential cycle: field <f> of class <A> holds <B> and <B>.<g> holds <A>` for statically visible cycles.
* `[WK-2]` `drop` runs when the strong count hits zero even if weak handles remain; those weak handles then fail to upgrade.
* `[WK-3]` `Weak.upgrade` MUST return `None` when `flags.is_deinitialising` is set on the target, and `retain` on such an object MUST NOT be treated by any optimisation as producing a usable handle. This is the mechanism behind `[WK-2]`.

## VIII.6 Stack promotion (optimisation)

`[OPT-1]` If escape analysis proves that no handle to an object outlives the function that created it (no store into memory, no `owned` argument, no return, no capture by `owned fn`, no `spawn`), the compiler MAY allocate the object in the function's frame with the same header and run `drop` at scope end. This is unobservable (identity comparisons, `drop` order and exclusivity semantics are preserved).
* `[OPT-2]` **Loop bounds-check versioning.** For a counted loop whose induction variable `i` ranges over `a..b` or `a..=b` with `a` and `b` loop-invariant, whose body indexes one or more views at `i`, `i + c` or `i − c` for compile-time constant `c`, and where each such view's base and length are loop-invariant across the loop, the compiler MUST emit: a single loop-entry test that every index the loop can produce lies in `0..len` for every indexed view; an **unchecked** body with those `Assert` terminators removed, taken when the test passes; and the ordinary **checked** body otherwise. Observable behaviour is unchanged in every profile: the unchecked body is entered only when no iteration of it could have failed, so `[PHIL-5]` and `[PRF-1]` are satisfied and no profile setting is involved.
* `[OPT-3]` The rule applies whether the bound is written `s.len()` (the case §4.12 already names) or is a separate loop-invariant local — the latter is the shape `[PAR-2]`'s disjointness proof requires, and MUST be covered.
* `[OPT-2a]` `tests/conformance/OPT-2/` MUST contain the `@parallel` example of Part XI §4 and the `integrate` example of Part I §5, each asserting that the emitted C contains no call to `ember_panic_bounds` inside the loop body.

## VIII.7 Class idioms for engine code

```ember
open class Component:
    let entity: Entity                     # immutable after init
    world: Weak[World]                     # back-pointer: weak

class Health(Component):
    value: f32 = 100.0
    on_death: Option[Box[dyn fn(Entity)]] = None   # owned callback; fine to store (owned fn)

    fn apply(mut self, dmg: f32):
        self.value -= dmg
        if self.value <= 0:
            if Some(cb) = self.on_death:  # borrows the callback for the call
                cb(self.entity)
```

---

# Part IX — Memory Facilities

## IX.1 The library heap types

| Type | Semantics | Copy? | Thread |
|---|---|---|---|
| `Box[T]` | uniquely owned heap `T`; `Box(v)`, `b.get()`, deref via auto-deref; `Box[dyn I]` for existentials | move-only | `Send` iff `T: Send` |
| `Shared[T]` | reference-counted heap `T` (a struct behaving like a class instance) with `Weak[T]`; same header as classes; `s.get()`; mutation through `Shared` follows the exclusivity rules of Part VIII §3 | Copy (retain) | count atomic iff `T: Sync` |
| `Array[T]` | growable buffer `{ptr, len, cap}`; SSO not applied; `Array.with_capacity(n)`; `reserve`, `push`, `pop`, `insert`, `remove`, `swap_remove`, `retain`, `drain`, `clear`, `truncate`, `extend`, `as_span`, `as_mut_span`, `sort`, `sort_by`, `binary_search` | move-only; `Clone` if `T: Clone` | `Send`/`Sync` iff `T` |
| `String` | UTF-8 `{ptr, len, cap}` with **inline storage for ≤ 23 bytes** (SSO) | move-only | `Send + Sync` |
| `Map[K, V]` | open-addressing hash map (SwissTable-style), `K: Hash + Eq` | move-only | iff `K, V` |
| `Set[T]`, `Deque[T]`, `BitSet`, `SmallArray[T, N]` (inline up to N), `Pool[T]` (slot map with generational keys) | as named | | |

`[HEAP-1]` All of these allocate through `ember_alloc`/`ember_realloc`/`ember_free` and report allocation to the `Alloc` effect (Part X). `[HEAP-2]` Growth factor is 2× with a minimum of 4 elements; `shrink_to_fit` is explicit.

## IX.2 Arenas

```ember
frame = Arena.with_capacity(16 * MB)          # one chunk; grows by chunk (default 1 MB) if exceeded
cmd   = frame.alloc(RenderCommand(...))       # -> ref mut RenderCommand, region = borrow of `frame`
list  = frame.alloc_array[Instance](count)    # -> MutSpan[Instance], zero-initialised if T: Zeroable else Default
tmp   = frame.alloc_uninit[u8](bytes)         # -> MutSpan[MaybeUninit[u8]]
frame.reset()                                 # requires `mut frame` and no live borrows (borrow checker enforces)
```

* `[ARN-1]` `Arena` is a move-only struct. All `alloc*` methods take `self` (shared borrow) so many allocations can be outstanding; they return views with the arena's region. `reset()` and drop take `mut self`, so `[BRW-1]` guarantees no live view survives a reset.
* `[ARN-2]` Values allocated in an arena are **not dropped individually**. `[ARN-3]` Allocating a type that `needs_drop` in an arena is `E3090` unless the call is `alloc_nodrop` (explicit acknowledgement that `drop` will never run) — this keeps arenas free of destructor bookkeeping and prevents silent resource leaks of handles/GPU objects.
* `[ARN-4]` `Arena` allocation is bump allocation with alignment padding; it is `@noalloc`-clean **only** if the arena is `@noalloc`-declared (`Arena.fixed(buffer: MutSpan[u8])`, which never grows and panics on exhaustion) — a growing arena carries the `Alloc` effect on the growth path. The type `FixedArena` is provided for hot paths.
* `[ARN-5]` `ArenaArray[T]`, `ArenaMap[K,V]` are container variants whose backing storage is an arena view; they are view types (`@view`) and follow `[TYP-15]`.
* `[ARN-6]` `ScopedArena`: `Arena.scope(mut self) -> ScopedArena` takes a **mutable** borrow of the parent arena, held for the `ScopedArena`'s whole region. `with scope = frame.scope():` creates a nested mark; the block's allocations are released at block end (LIFO), giving job-local memory (Part XI §6). While a scope is live the parent MUST NOT be allocated from, reset, or dropped — `E3096 arena is scoped here`, with `help: allocate from `scope` instead, or take this allocation before opening the scope`. Nested scopes are obtained from the `ScopedArena` (`ScopedArena.scope(mut self)`), which nests marks to any depth.
* `[ARN-7]` LIFO rewind is a **safety property, not a convenience**. An implementation MUST NOT provide any operation that lowers an arena's bump pointer, invalidates a mark, or reuses arena bytes while a view whose region derives from that arena is live. Every such operation MUST take `mut self` on the arena whose bytes it reclaims, which is what makes `[BRW-1]` the enforcing rule. `[ARN-1]`'s argument extends to `scope`, `reset` and drop alike.

## IX.3 Allocators

```ember
unsafe interface Allocator:
    fn alloc(mut self, layout: Layout) -> Result[*mut u8, AllocError]
    fn dealloc(mut self, ptr: *mut u8, layout: Layout)
    fn realloc(mut self, ptr: *mut u8, old: Layout, new_size: usize) -> Result[*mut u8, AllocError]: ...default...
```

* `[ALC-1]` `Array[T, A: Allocator]`, `Map[K, V, A]`, `Box[T, A]` accept an allocator type parameter (default `Global`). Allocator instances are passed at construction (`Array.new_in(alloc)`), stored by value if zero-sized, else as a `ref`/handle.
* `[ALC-2]` Implementing `Allocator` is `unsafe` (the implementer promises the returned memory is valid, aligned and not aliased).
* `[ALC-3]` The global allocator can be replaced at link time by defining `@export("ember_global_allocator") static ALLOC: dyn Allocator` in the binary package.
* `[ALC-4]` Allocation failure is a **panic** for the standard containers (`alloc` returns `Result`, containers unwrap). `try_reserve` exists for code that must handle it.

## IX.4 Raw pointers and `unsafe`

```ember
unsafe fn read_u32_le(p: *u8) -> u32:                 # unsafe fn: caller must uphold "p points to ≥4 readable bytes"
    return (p.read() as u32) | ((p.offset(1).read() as u32) << 8) | ...

fn parse(data: Span[u8]) -> Option[u32]:
    if data.len() < 4: return None
    unsafe:                                            # SAFETY: bounds checked above
        return Some(read_u32_le(data.as_ptr()))
```

* `[UNS-1]` Operations requiring an `unsafe` context: dereferencing `*T`/`*mut T` (`read`, `write`, `deref`, `deref_mut`, index), pointer arithmetic (`offset`, `add`, `sub`), pointer casts (`[TYP-7]`), calling an `unsafe fn`, calling any `extern` function not covered by a verified contract (`[FFI-*]`), accessing `static mut`, `transmute`, `get_unchecked`, `assume_init`, implementing an `unsafe interface`, inline assembly.
* `[UNS-2]` An `unsafe:` block does not disable the borrow checker, bounds checks on safe types, or type checking; it only permits the operations above.
* `[UNS-3]` The lint `L3010 unsafe block larger than necessary` fires when statements inside an `unsafe` block need no unsafe permission.
* `[UNS-4]` Invariants safe code may assume and unsafe code MUST uphold: every `ref` is non-null, aligned, points to initialised memory of the right type, and is not aliased by a `ref mut` while live; every `Span` length is within its allocation; every class handle points to a live object with a correct header; every `str` is valid UTF-8; no `Send`/`Sync` violation. No two views (`Span`, `MutSpan`, `str`, `Ref`, `RefMut`, a `@view struct`, or a `ref`) that are simultaneously live in safe code may overlap unless both are shared. Constructing overlapping views through raw pointers, `transmute`, or a foreign call and handing them to safe code is undefined behaviour; `[SIMD-3]` and `[CG-C-4]` depend on this invariant. Unsafe code MUST NOT use a raw pointer derived from a `ref` after that reference's region has ended, nor one derived from a class-object field after the object's last live handle has been released (`[RC-5]`).
* `[UNS-5]` `MaybeUninit[T]`, `transmute[A, B]`, `ptr.copy_nonoverlapping`, `mem.zeroed[T]()` (requires `T: Zeroable`, an unsafe marker interface auto-derived for all-scalar/POD structs) are provided in `std.mem`.
* `[UNS-6]` Inline assembly: `unsafe asm("…", inputs, outputs, clobbers)` following LLVM's constraint syntax; the C backend rejects it (`E5090`) except on Clang/GCC where it emits `__asm__ volatile`. Prefer `std.cpu` intrinsics.

## IX.5 Layout attributes

* `@layout(c)` (default): C struct layout for the target ABI.
* `@packed`: no padding; field access to unaligned fields is lowered to `memcpy`-style loads/stores (never UB); taking a `ref` to an unaligned field is `E2170` (take a copy).
* `@align(N)`: raise alignment; `N` power of two ≤ 4096.
* `@repr(u8|u16|u32|i32|…)`: enum discriminant type; required for FFI enums.
* `@gpu_layout(std140|std430|scalar)`: see Part XVII; produces `comptime`-visible offsets and asserts that the CPU layout matches (`E8001` with the offending field and both offsets if not — the programmer inserts explicit padding fields).
* `size_of[T]()`, `align_of[T]()`, `offset_of[T](field)` are comptime functions.

## IX.6 Handles for resources

The standard library provides a generational handle facility used by the GPU layer and recommended for any resource manager:

```ember
@derive(Copy, Eq, Hash, Debug)
struct Handle[Tag]:                     # Tag is a phantom type: Handle[Texture] ≠ Handle[Buffer]
    index: u32                          # 20 bits index, 12 bits generation — matches RageV's ECS.Entity packing
    fn is_null(self) -> bool
    const NULL: Handle[Tag]

struct Pool[T]:                         # slot map
    fn insert(mut self, owned v: T) -> Handle[T]
    fn get(self, h: Handle[T]) -> Option[ref T]        # generation-checked
    fn get_mut(mut self, h: Handle[T]) -> Option[ref mut T]
    fn remove(mut self, h: Handle[T]) -> Option[T]
```

`[HND-1]` `Handle` is a plain `Copy` value; `[HND-2]` the index/generation split is configurable per `Pool` (`Pool[T, INDEX_BITS=20]`).

## IX.7 Interior mutability: `Cell` and `RefCell`

`[BRW-1]`'s aliasing-XOR-mutability rule is checked statically for value types. Some correct programs cannot be written that way: a shared cache, a memoised field, an observer that mutates a counter it does not own, a value reachable from two places that both need to write it. Classes solve this for object graphs (Part VIII: refcounting plus dynamically checked exclusivity). `Cell` and `RefCell` provide the equivalent escape hatch for **value types**, so that a programmer who hits the wall on a `struct` has a ladder that stays in safe code.

Both are safe types built on `unsafe` internals. Neither is a way to opt out of the rules; each moves one specific check from compile time to runtime, and says so in the type.

### `Cell[T]` — replace the whole value, no references

```ember
struct Sprite:
    frame: Cell[u32]                          # mutable through a shared borrow
    texture: TextureHandle

fn advance(s: Sprite):                        # note: `s` is borrowed, not `mut`
    s.frame.set(s.frame.get() + 1)
```

* `[CELL-1]` `Cell[T]` places any `T`. Its unconditional API is `Cell(owned v)`, `set(self, owned v: T)`, `replace(self, owned v: T) -> T`, `into_inner(owned self) -> T`, and `take(self) -> T where T: Default`; all take `self` (a shared borrow) and mutate. `get(self) -> T` is provided only where `T: Copy`, by `extend[T: Copy] Cell[T]:` — ordinary Part V §6 machinery, no specialisation implied, `[TYP-19]` unaffected. `update(self, f: fn(T) -> T)` requires `T: Default` or `T: Copy`. **`set` and `replace` MUST store the new value before dropping the old one.** A drop can run arbitrary user code that re-enters the same `Cell` (`Cell[Box[Node]]` where `Node`'s drop reaches back and reads it); a drop-then-store implementation would leave the `Cell` observably uninitialised across that window, which is a read of uninitialised memory.
* `[CELL-4]` A `Cell` field does not make its containing struct mutable in any other respect. `Cell[T]` is `Copy` when `T: Copy`, and copying such a `Cell` copies the value it holds at that moment; `Cell[T]` for a non-`Copy` `T` is move-only, and is `Drop` iff `T` is.
* `[CELL-2]` `Cell` never hands out a reference to its contents, so no aliasing rule can be violated and **no runtime check is needed**. `get` is a load; `set` is a store. There is no overhead relative to a plain field.
* `[CELL-3]` `Cell[T]` is `!Sync` (`[THR-1]`): it may be moved between threads if `T: Send`, but never shared. The `Sync` equivalent is `Atomic[T]`.

### `RefCell[T]` — dynamically checked borrows of any `T`

```ember
struct Scene:
    entities: RefCell[Array[Entity]]

fn add(s: Scene, e: Entity):
    with list = s.entities.borrow_mut():      # runtime check begins here
        list.push(e)                          # list: ref mut Array[Entity]
                                              # check ends at block exit
```

* `[CELL-5]` `RefCell[T]` holds a borrow-state counter alongside `T`. `borrow(self) -> Ref[T]` succeeds unless a mutable borrow is active; `borrow_mut(self) -> RefMut[T]` succeeds unless any borrow is active. Both **panic** on failure with `E-panic: RefCell already mutably borrowed (borrowed at <file>:<line>)` — the message names the source location of the conflicting borrow, which the runtime records in debug and release profiles.
* `[CELL-6]` `try_borrow`/`try_borrow_mut` return `Option[Ref[T]]`/`Option[RefMut[T]]` for code that must handle contention rather than panic.
* `[CELL-7]` `Ref[T]`/`RefMut[T]` are **view types** (`[TYP-15]` applies) whose region borrows the `RefCell`; their `drop` releases the borrow state. They MUST be bound by `with` or a local — the lint `L3011 RefCell guard held across a call` fires when a guard is live across a function call that could re-enter the same cell.
* `[CELL-8]` `RefCell[T]` is `!Sync`. The `Sync` equivalents are `Mutex[T]` and `RwLock[T]`, whose API is deliberately the same shape (`with g = m.lock():`) so that promoting single-threaded code to shared code is a type change and nothing else.
* `[CELL-9]` The borrow-state counter is one machine word and the check is present in **every** profile. `exclusivity = "unchecked"` (`[EXC-1]`, ADR-004) governs dynamic **class** exclusivity only; it MUST NOT affect `RefCell`, `Ref`, `RefMut`, `Cell`, `Mutex` or `RwLock`. A package that requires an interior-mutability primitive with no check uses `unsafe` (`UnsafeCell`, `[UNS-*]`), which is visible in review and in `grep`.
* `[CELL-10]` `RefCell` is not a synchronisation primitive and not a substitute for restructuring. The diagnostic for a borrow error (`[DIA-7]`, shape B4) suggests `RefCell` **only** when the conflicting accesses are provably not simultaneous in the same expression — never as a first suggestion.
* `[CELL-6a]` `try_borrow` and `try_borrow_mut` MUST return `None` on contention in every profile. No profile setting may make them infallible; doing so would change which branch of a `match` executes, which `[PRF-1]` forbids.

### Which to reach for

| Situation | Use |
|---|---|
| Mutating a `Copy` field through a shared borrow | `Cell[T]` |
| Mutating a non-`Copy` value (container, `String`) through a shared borrow, single-threaded | `RefCell[T]` |
| Object graph with aliased mutation | `class` (Part VIII) — already gives this behaviour with no wrapper type |
| Same, across threads | `Mutex[T]` / `RwLock[T]` / `Atomic[T]` |
| Two mutable views into one container | `split_at_mut`, `chunks_mut`, `columns_mut` — no wrapper needed |

`[CELL-11]` `Cell`, `RefCell`, `Ref`, `RefMut`, `Atomic`, `Mutex`, `RwLock` live in `std.cell` and `std.sync`; `Cell` and `RefCell` **are** in the prelude (owner decision `OQ-10`): `Shared[T]` is already there and is heavier on every axis this document measures, so taxing the zero-cost facility and not the expensive one inverted the gradient the earlier rule meant to create. `[DIA-9]` still forbids offering them as a *first* suggestion, which is where the visibility actually matters.

## IX.8 Establishing disjointness: `assert_disjoint` and `assume_disjoint`

`[BRW-5]` makes two mutable borrows through computed indices conflict, because the compiler cannot in general prove `i != j`. The sanctioned structural fixes (`split_at_mut`, `chunks_mut`, `columns_mut`) cover the common cases. For the rest — two spans obtained from unrelated sources that the programmer knows do not overlap — Ember provides two clearly distinct tools, matching the two arms of `[PHIL-8]`.

### `mem.assert_disjoint` — verify now, establish for the region (safe)

```ember
from std.mem import assert_disjoint

fn blend(mut dst: MutSpan[f32], src: Span[f32]):
    match assert_disjoint(dst, src):
        Some(d, s):                       # d, s are *known* disjoint from here on
            for i in 0..d.len():
                d[i] = d[i] * 0.5 + s[i] * 0.5
        None:
            fallback_overlapping_blend(dst, src)
```

* `[DSJ-1]` `assert_disjoint(a, b)` compares the two views' address ranges (`base`, `base + len * size_of[T]()`) and returns `Some((a', b'))` when they do not overlap, `None` when they do. It **consumes** `a` and `b` and returns fresh views carrying a compile-time disjointness fact. The cost is two comparisons, once, at the call.
* `[DSJ-2]` The proof is attached to the **returned values**, not to a program point. This is deliberate: a flow-sensitive fact recorded against a line silently rots when either operand is reassigned, whereas a fact carried by a value cannot be separated from the value it describes. Reassigning `d` or `s` produces ordinary views again.
* `[DSJ-3]` The returned views are treated as **non-overlapping places** by the borrow checker (`[BRW-5]` does not apply between them) and by the aliasing facts passed to the backend (`restrict` / `noalias`, `[SIMD-3]`), so a loop over both vectorises.
* `[DSJ-4]` Applicable operands: `Span[T]`, `MutSpan[T]`, `SoA` columns, and arena views — anything whose base address and byte length are recoverable. It is **not** available for arbitrary `ref mut`s to unrelated locals (`E3095`), because two single-object references have no range to compare and the borrow checker already handles the cases that arise in practice.
* `[DSJ-5]` `assert_disjoint_or_panic(a, b) -> (MutSpan[T], Span[T])` is the panicking form. Both forms record a `RuntimeCheck(Aliasing)` site with reason `establishes_static_fact` **in the contract effect set (`[EFF-15]`)** whenever the comparison is not elided by profile-independent analysis; the comparison is elided when the compiler already knows the ranges are disjoint (the common `split_at_mut` case), in which case no site is recorded.
* `[DSJ-6]` `assert_disjoint` is `@noalloc`, `@nosync`, and usable in a `@static_safe` function: it *establishes* a static fact rather than deferring a check, and once the comparison is elided or hoisted out of a loop the body contains no aliasing check at all. When the comparison itself is emitted inside a `@static_safe` function, it is permitted — `[EFF-12]` forbids checks that stand in for an unproven property, and this one proves it.

### `unsafe assume_disjoint` — assert without verification (unsafe)

```ember
unsafe:
    d, s = assume_disjoint(dst, src)     # SAFETY: caller contract guarantees distinct allocations
```

* `[DSJ-7]` `assume_disjoint` has the same signature shape as `assert_disjoint` but performs **no check in any profile** and returns the views unconditionally. It requires `unsafe` because the programmer, not the machine, is asserting the property; violating it is UB exactly like any other broken `unsafe` precondition (`[UNS-4]`).
* `[DSJ-8]` There is deliberately **no third form** that is checked in `debug` and assumed in `shipping`. Such a construct would give a program two different meanings under two profiles, would move a memory-safety property into a build setting, and would be invisible in review because it does not contain the word `unsafe`. `[PRF-1]` already forbids profiles from changing semantics; this rule names the specific temptation.
* `[DSJ-9]` `assert_disjoint_all(v1, …, vn)` and `assert_disjoint_all_or_panic` accept 2..8 view operands, perform the n(n−1)/2 pairwise range comparisons (at most 28), consume all operands and return proof-carrying views for all of them, pairwise disjoint. `[DSJ-1]`..`[DSJ-6]` apply unchanged to each pair.

The distinction in one line: **`assert_disjoint` verifies a property and establishes it; `assume_disjoint` claims a property Ember cannot verify.**

---

# Part X — Effects and Performance Contracts

## X.1 Effects

The compiler infers, for every function, an **effect set** ⊆ `{Alloc, Sync, Panic, Unsafe, FFI, Block, RuntimeCheck(k)}`:

| Effect | Introduced by |
|---|---|
| `Alloc` | any call to `ember_alloc`/`realloc`, class instantiation, `Box`, `Shared`, container growth, `String` formatting, f-strings, `Arena` growth, boxed closures |
| `Sync` | atomic RMW ops with ordering stronger than relaxed, `Mutex`/`RwLock` lock, channel send/receive, thread spawn/join, `Sync`-class retain/release (atomic) |
| `Panic` | `panic`, `assert`, bounds checks, overflow checks (debug), `unwrap`, exclusivity checks, division |
| `Unsafe` | body contains an `unsafe` block or the function is `unsafe fn` |
| `FFI` | calls to `extern` functions |
| `Block` | `Mutex.lock`, `join`, `sleep`, channel blocking receive, `File.read` — any call the runtime marks blocking |
| `RuntimeCheck(k)` | a safety check the compiler emitted rather than proved away, for kind `k ∈ {Aliasing, Bounds, Stale, Overflow}` — see §X.1.1 |

* `[EFF-1]` Effects are computed per function from its body and the (already computed) effects of its callees, over the call graph, with recursion handled by fixpoint (recursive SCCs are assumed to have the union of their members' direct effects).
* `[EFF-2]` Calls through `dyn` or function values contribute the effects declared on the interface method or function type; a method in an interface may declare `@noalloc` and implementers MUST satisfy it (`E4010`). A `fn(A) -> R` parameter type is assumed to carry all effects unless written `@noalloc fn(A) -> R`.
* `[EFF-3]` `extern` functions carry effects declared in their contract (`@ffi(effects=[FFI])` by default; `@ffi(effects=[FFI, Alloc, Block])` if the binding says so).
* `[EFF-4]` Effects are part of a function's public interface for the purpose of caching: a change to a callee's effect set invalidates callers' contract checks (Part XIX build graph).

### X.1.1 The `RuntimeCheck` effect

`RuntimeCheck` records that Safe code obtained one of its guarantees by the runtime arm of `[PHIL-8]` rather than the static arm. Its four kinds:

| Kind | Emitted for |
|---|---|
| `Aliasing` | dynamic class exclusivity (`[EXC-1/2]`), `RefCell.borrow`/`borrow_mut` state checks (`[CELL-5]`) |
| `Bounds` | an index or slice check the compiler could not prove redundant |
| `Stale` | generational handle validation (`[HND-1]`, `[GPU-1]`), `Weak.upgrade` |
| `Overflow` | checked arithmetic under `@overflow(panic)` or the `debug` profile |

* `[EFF-9]` `RuntimeCheck(k)` enters a function's **contract** effect set when a check of kind `k` in that function's own body survives the profile-independent elision passes of `[EFF-15]`, and propagates through the call graph like every other effect (`[EFF-1..3]`). It is a statement about the code the contract profile would generate, not about source syntax and not about the selected profile.
* `[EFF-10]` **The effect is coarse; the site record is not.** The effect set carries only the kinds present, so that contracts can be checked cheaply and transitively. Separately, codegen emits a **safety-check side table** (`target/<profile>/inspect/<module>.safety.json`) with one entry per emitted check: `{kind, source span, function, mechanism, reason}`, plus one entry per *elided* check with the analysis that removed it. The side table is what `ember inspect --safety` reads (`[CLI-3]`). Implementations MUST NOT push per-site data into the effect lattice.
* `[EFF-11]` **Reason codes.** Every emitted-check entry carries exactly one reason, and diagnostics MUST use its wording rather than a generic "could not prove" message:

  | Reason | Meaning | What the programmer can do |
  |---|---|---|
  | `not_provable_in_principle` | no static analysis could establish the property — the value is genuinely runtime data (an index read from a file, a handle from a scene) | nothing; the check is correct and permanent |
  | `not_proven_by_analysis` | the property may hold, but this compiler's analysis did not establish it | restructure per the hint, or file a compiler issue — this is the only reason that is a candidate for future elision |
  | `requested_by_type` | the programmer chose a dynamically checked type (`RefCell`, `Cell`-free aliasing through a class) | the check is the type's purpose; change the type to change the cost |
  | `inherent_to_mechanism` | the check is what the mechanism *is* (a generation compare in a generational handle) | use a different mechanism (a direct reference, an index) if the cost matters |
  | `establishes_static_fact` | the check verifies a property once and returns proof-carrying values, so the property is static from there on (`[DSJ-1]`, `[DSJ-5]`) | nothing; the check is what makes the code after it checkable, and `[EFF-12]` permits it under `@static_safe` |

  `[EFF-11a]` Reporting `not_proven_by_analysis` where `not_provable_in_principle` is correct is a diagnostic bug: it tells the programmer to restructure code that cannot be improved. The conformance suite fixes the expected reason for each check site in `tests/safety/reasons/`.

## X.2 Hard contracts

* `@noalloc fn`: `[EFF-5]` MUST NOT have `Alloc` in its effect set. Violations report the **full call chain** to the allocation site: `error[E4001]: @noalloc function `cull` reaches an allocation: cull → collect_visible → Array.push → ember_alloc`.
* `@nosync fn`: MUST NOT have `Sync`.
* `@noblock fn`: MUST NOT have `Block`.
* `@nopanic fn` **(v2)**: MUST NOT have `Panic` — requires proving away bounds checks; reserved.
* `@static_safe fn`: `[EFF-12]` MUST NOT have `RuntimeCheck(Aliasing)` in its effect set **except from sites whose reason code is `establishes_static_fact`** (owner decision `OQ-11`). That is: every aliasing property the function *relies on* is either proven statically or established by a verifying operation whose result carries the proof. It forbids dynamic class exclusivity, `RefCell` borrow checks, and any callee that carries a non-establishing `Aliasing` check. It does **not** forbid `RuntimeCheck(Bounds)`, `(Stale)` or `(Overflow)`: an in-bounds index and a live handle are different guarantees from aliasing, are frequently not provable in principle (`[EFF-11]`), and lumping them together would make the contract unusable. `@no_runtime_checks` — the stronger form excluding all four kinds — is **reserved for v2**, once bounds-check elimination is strong enough for it to be satisfiable.
* `[EFF-13]` **`@static_safe` is, in practice, a value-type contract.** Any long-term access through a class handle that was loaded from memory (a field, an array element, a `dyn` receiver) is dynamically checked by construction (`[EXC-3]`), so most code written against class handles cannot satisfy the contract. This is correct behaviour, not a limitation to be worked around, and the diagnostic MUST say so specifically rather than reporting a bare contract violation:

  ```
  error[E4030]: @static_safe function `integrate` performs a dynamically checked access
    --> src/systems.em:14:9
     |
  14 |         self.target.transform.position += v
     |         ^^^^^^^^^^^ handle loaded from field `self.target`, so exclusivity is checked at runtime
     |
     = help: hoist the handle to a local first, so accesses through it are statically ordered:
             `t = self.target` then `t.transform.position += v`
     = help: or operate on value types — a `Query[(mut Transform,)]` yields `ref mut Transform` with no handle indirection
     = note: accesses through a handle held in a local are proven statically (EXC-3); accesses through a
             handle re-read from memory cannot be, because another handle to the same object may exist
  ```
* `[EFF-14]` Contract attributes compose: `@static_safe @noalloc @nosync` is the inner-loop set. Each is checked independently against the same effect set.
* `[EFF-6]` A contract applies to the whole reachable call graph, including drop glue for locals, default arguments, and operator impls.
* `[EFF-6a]` `@static_safe` additionally applies to any check the *caller* would have to emit on this function's behalf — a `@static_safe` function may not take a parameter whose type forces a dynamic check at the call site (`RefCell[T]` by value, `Ref[T]`/`RefMut[T]` guards).
* `[EFF-7]` `unsafe: @assume_noalloc(expr)` overrides the analysis for one call (e.g. a C function known not to allocate); it is `unsafe` because the compiler cannot verify it.
* `[EFF-8]` Contracts are inherited by overriding methods (an `override` of a `@noalloc virtual fn` must be `@noalloc`).
* `[EFF-15]` **The contract profile.** Contract checking (`[EFF-5]`, `[EFF-12]`, `@nosync`, `@noblock`, and `@nopanic` in v2) MUST use an effect set computed once per build against the **contract profile**: bounds checks enabled, `overflow = "panic"`, `exclusivity = "checked"`, and only those elisions that are independent of profile settings (`[EXC-3]`, `[RC-2/3]`, target-independent bounds-check elimination). The contract profile pins the **set of elision passes** as well as the profile keys: adding or strengthening an elision pass changes which programs satisfy a contract, and is therefore a versioned change under `[VER-2]`. A profile setting MUST NOT remove an effect from the contract set. Consequently a program that satisfies its contracts under one profile satisfies them under all, and `E4001`/`E4030` are reported identically by `ember build --profile debug|release|shipping`.
* `[EFF-16]` **`Panic` is refined.** `Panic(Explicit)` is introduced by: `panic`, `assert`/`assert_eq`/`assert_ne` (every profile), `unwrap`, `expect`, `todo`, `unreachable`, `as!` downcast, allocation failure (`[ALC-4]`), integer division or remainder by a divisor not proven non-zero, `i32.MIN / -1` where the operands are not proven safe, and a shift whose amount is not proven in range. Every other panic is recorded by the `RuntimeCheck(k)` kind that already covers it: bounds panics by `Bounds`, overflow panics by `Overflow`, exclusivity and `RefCell` borrow failures by `Aliasing`, generational-handle and `Weak.upgrade` failures by `Stale`. `Panic`, where this document uses it unqualified, means the union. `[EFF-9]`'s post-elision rule applies to `Panic(Explicit)` unchanged, computed **under `[EFF-15]`'s contract profile**, so the set does not vary with the optimisation level.
* `[EFF-17]` **`@nopanic(explicit) fn` (v1).** MUST NOT have `Panic(Explicit)` in its contract effect set. Violations report the full call chain in `[EFF-5]`'s shape: `error[E4040]: @nopanic(explicit) function `resolve` reaches a panic: resolve → Option.unwrap → ember_panic_unwrap`. Bare `@nopanic` remains reserved for v2 and is defined as forbidding `Panic(Explicit)` together with `RuntimeCheck(Bounds)` and `RuntimeCheck(Overflow)`; `[OPT-2]` is what makes it satisfiable. `[EFF-6]`, `[EFF-8]` and `[EFF-2]` apply to it exactly as to the other contracts. `[EFF-14]`'s inner-loop set becomes `@static_safe @noalloc @nosync @nopanic(explicit)`.

## X.3 Inspection

`ember inspect path.to.fn` prints the effect set, the allocation sites reachable, inlining decisions, whether loops vectorised, the chosen ABI for each parameter, and `size_of`/`align_of`/field offsets for types. This is the primary tool for making the compiler's allocation, layout, dispatch and vectorisation decisions visible rather than implicit.

`ember inspect --safety <path>` reads the side table of `[EFF-10]` and reports every runtime safety check the function emits and every one it elided:

```
$ ember inspect --safety game.world.World.update

Function: game.world.World.update
Effects: Alloc, Panic, RuntimeCheck(Aliasing), RuntimeCheck(Bounds)

Runtime checks emitted:
  Aliasing   2
    src/world.em:88:9   dynamic exclusivity on `Entity`
                        reason: not_proven_by_analysis — handle loaded from `self.active[i]`
                        help:   hoist to a local, or iterate a Query yielding `ref mut Transform`
    src/world.em:141:5  RefCell borrow_mut on `RefCell[Array[Event]]`
                        reason: requested_by_type
  Bounds     1
    src/world.em:96:22  index `ids[j]` where `j` is read from `pending`
                        reason: not_provable_in_principle
  Stale      0
  Overflow   0

Elided:
  Aliasing   6   (EXC-3: accesses ordered through the same handle local)
  Bounds    23   (range analysis over `for i in 0..len`)
  RC        14   (RC-2a borrowed parameter, RC-2c retain/release pair)

Estimated cost of emitted checks: ~7 ns/call at the measured call count
```

* `[CLI-3]` `--safety` accepts `--json`, and `--safety --elided-only` reports just what was removed, which is the form used when investigating why a `@static_safe` function fails to compile.
* `[TOOL-1]` Each release publishes a self-contained toolchain archive per supported host (`x86_64-pc-windows-msvc`, `x86_64-unknown-linux-gnu`, `aarch64-apple-darwin`) containing `ember`, the `ember_rt` libraries, `std` sources and the editor packages, plus a one-line installer (`install.ps1`/`install.sh`) that unpacks it and places `ember` on `PATH`.
* `[TOOL-3]` When no C compiler is found the driver emits `E9001 no C compiler found`, whose `help` names the remedies verbatim for the host — on Windows, Visual Studio Build Tools with the "Desktop development with C++" workload. **A missing `cl.exe`/`clang` path MUST NOT be the primary message.**
* `[TOOL-4]` `ember --version` prints compiler version, language version, backend, and the resolved C compiler and linker, so a bug report carries its environment.  **M0 — the first hour** (Phase 0 exit criterion; `tests/milestones/m0_first_hour.md` plus a CI job on a clean image with no C toolchain beyond what `[TOOL-1]`/`[TOOL-3]` provide, no Rust and no editor): the recorded script `install → ember new hello → cd hello → ember run` MUST print `hello, world` in under five minutes wall time and at most six typed commands, on Windows and on Linux. A regression in M0 blocks a release.
* `[TOOL-2]` `ember toolchain install cc` downloads a pinned Clang + `lld` into the toolchain directory and selects it via `[build] c_compiler = "bundled"`. `lld` is the default linker where available. **A working Ember installation MUST NOT require a separately installed C toolchain.** This is a committed deliverable (owner decision `OQ-23`), not a conditional one; a system compiler MAY be selected explicitly and its full configuration is recorded in the build record.

---

# Part XI — Concurrency and Parallelism

## XI.1 Thread safety markers

* `Send`: a value may be moved to another thread. Auto-derived (`[TYP-*]`). Not `Send`: `*T`, `*mut T`, `ref`/`ref mut`/views (in v1 — scoped threads relax this, §3), non-`Sync` class handles, `Shared[T]` where `T: !Sync`.
* `Sync`: a value may be *shared* (borrowed) by multiple threads simultaneously. Auto-derived when all fields are `Sync`. `ref mut` is never `Sync`. Interior mutability primitives (`Cell[T]`, `RefCell[T]`; Part IX §7) are `!Sync`; `Atomic[T]`, `Mutex[T]`, `RwLock[T]` are `Sync` (when `T: Send`).
* `[THR-1]` A class is `Sync` iff **every** field's type is itself an interior-synchronised type (`Atomic`, `Mutex`, `RwLock`, channel end) or a deeply immutable value type. **There is no `let` exemption** (`OQ-18`): `let` restrains rebinding, not mutation through the field (`[CLS-9a]`), so exempting `let` fields would make a class `Sync` whose contents a `mut self` method can still mutate — a data race in Safe code. A class that needs a shared mutable field uses `Mutex[T]`/`RwLock[T]`; one that needs a frozen field uses a type with no `mut self` API. — because handles alias, a plain mutable field shared across threads would be a data race. `@sync class` asserts `Sync` and is `E7001` if the rule is violated; `@thread_local class` forces `!Sync` even if the fields would allow it (to get non-atomic counts).
* `[THR-2]` Class handles are `Send` iff the class is `Sync` (a sent handle can be copied on both sides).

## XI.2 Primitives (`std.sync`, `std.thread`)

```ember
t = thread.spawn(owned fn() -> R: pass)      # requires captures Send; returns JoinHandle[R]
r = t.join()                                # -> Result[R, PanicPayload]

m = Mutex[Array[i32]](Array())              # Sync
with g = m.lock():                          # g: MutexGuard (view, ref mut Array[i32] via auto-deref)
    g.push(1)

a = Atomic[u32](0)
a.fetch_add(1, Ordering.Relaxed)            # orderings: Relaxed, Acquire, Release, AcqRel, SeqCst (default SeqCst)

tx, rx = channel[Message](capacity=1024)    # bounded MPSC; unbounded via channel_unbounded
tx.send(msg)?                               # Result; Err if receiver dropped
```

* `[THR-3]` `Mutex[T]` is a struct; locking returns a guard whose region borrows the mutex; the guard is `!Send`. Poisoning is not modelled (panic = abort by default); with unwinding (v2) the lock is released during unwind.
* `[THR-4]` Deadlock detection in debug: the runtime records lock ordering per thread and reports order inversions (`W-runtime: lock order inversion`).
* `[THR-6]` A type whose `drop` is load-bearing for a borrow guarantee is marked `@must_drop` (Part III §7). A `@must_drop` value MUST NOT be (a) passed to `mem.forget`, (b) stored in a `class` field, `Shared[T]`, `Box[T]`, or any container or closure capture that can participate in a reference cycle, or (c) returned from the function that created it. Violations are `E3015 value whose drop is required may not be leaked`, whose help names the closure-scoped form. In v1 the standard library applies `@must_drop` to exactly `Scope` and `JobScope`; adding it to any other type requires an entry in `docs/DECISIONS.md`.

## XI.3 Structured (scoped) concurrency

```ember
with scope = thread.scope():                       # all spawned tasks joined at block end
    scope.spawn(fn(): process(left))               # non-owned closures allowed: borrows outlive the scope's join
    scope.spawn(fn(): process(right))
```

`[THR-5]` `thread.scope(f)` takes the scope body as a closure and returns only after every task spawned inside it has been joined, on both normal and panicking exit. The `Scope` value is a parameter of `f`; it is a view type whose region is `f`'s body, so it cannot be moved out, stored, returned, captured by an `owned fn`, or passed to `mem.forget`. Inside the scope, closures may borrow locals of the enclosing function; the borrow checker treats the scope's region as covering the spawned closures. The surface form `with scope = thread.scope():` is sugar for `thread.scope(fn(scope): …)`; the binding it introduces is not a movable place (`E3014 scope binding may not be moved`). `[JOB-2]` is the same rule for `jobs.scope`.

## XI.4 `@parallel for`

```ember
@parallel(chunk=256)
for i in 0..count:
    positions[i] += velocities[i] * dt
```

* `[PAR-1]` The loop body is compiled as a closure invoked over index chunks by the job system (Part XI §5).
* `[PAR-2]` **Iterations MUST be independent.** The body MUST be free of `break`, `continue`-to-outer and `return`; captures are checked for `Send`. For each place `P` written in the body, the compiler collects the set of index expressions through which `P` is accessed anywhere in the body, **reads included**. The loop is accepted only if: (a) every such expression has the form `i + k` with `k` a compile-time constant (`k = 0` permitted); (b) **all of them have the same `k`**; and (c) `P` is a `MutSpan` or an `SoA` column captured by the loop, not a place reached through a class handle, a `RefCell`, a `Cell`, or a raw pointer. Otherwise `E7010 parallel loop writes to a shared place`, with the suggestion set (`Atomic`, `chunks_mut`, `@parallel(reduce=…)`).
* `[PAR-3]` Reductions: `@parallel(reduce=[sum: +, best: max])` declares variables combined with an associative operator after the loop.
* `[PAR-4]` `@parallel` loops carry the `Sync` effect (job submission) and are therefore illegal in `@nosync` functions; they are `@noalloc`-clean when the job system is initialised with preallocated queues (the default).
* `[PAR-2a]` Violation of clause (b) specifically is `E7011 parallel loop has a loop-carried dependency on <place>: written at index `i + <k1>`, also accessed at index `i + <k2>``, whose help MUST name a concrete alternative: split into two `@parallel` loops separated by a join, express it as `@parallel(reduce=…)` (`[PAR-3]`), or run the loop serially.
* `[PAR-2b]` The analysis MUST run on MIR **after inlining** of the body's calls. A call in the body that the compiler cannot see through and that receives a captured `MutSpan` or `SoA` column defeats the proof for every place reachable through that argument (`E7010`).

## XI.5 Job system (`std.jobs`)

```ember
jobs = JobSystem.init(worker_count = cpu.logical_cores() - 1)   # once, in main

h1 = jobs.submit(fn(): build_visibility(scene, mut visible))    # returns JobHandle
h2 = jobs.submit_after([h1], fn(): build_commands(visible, mut cmds))
jobs.wait(h2)
```

* `[JOB-1]` Work-stealing deques per worker; jobs are `owned fn() -> void` closures stored inline in a fixed-size slot (≤ 64 bytes captured, else boxed — reported by `ember inspect`).
* `[JOB-2]` Job closures may borrow outer state **only** through a `JobScope` (`with s = jobs.scope(): s.submit(...)`), which joins at block end — same rule as `[THR-5]`.
* `[JOB-3]` **Access-set scheduling.** A job may declare `jobs.submit_with_access(reads=[positions], writes=[velocities], f)`; the scheduler orders jobs whose declared write sets intersect any other's read/write sets, giving access-set-driven parallelism without whole-program analysis. `[JOB-4]` Access sets are checked in debug builds by recording actual `Span` base pointers touched (the runtime instruments `Span` creation inside jobs when `-Cjobs-verify` is on).

## XI.6 Job-local memory

`[JOB-5]` Each worker owns a `ThreadArena`; `jobs.local_arena()` returns a `ScopedArena` view for the current job that is reset when the job completes. Allocations from it cannot escape the job (their region is the job closure's), so temporary allocations are cheap and deterministic without threading an arena parameter through every function.

## XI.7 Async

Reserved for v2. `async`/`await` keywords are reserved; the design will follow "structured async over the job system", with the rule that borrows may not cross `await` points unless the future is scoped. Nothing in v1 precludes this.

---
# Part XII — Data-Oriented Programming

## XII.1 `SoA[T]`

```ember
@derive(Copy, SoA)
struct Particle:
    position: Vec3
    velocity: Vec3
    lifetime: f32

particles = SoA[Particle].with_capacity(100_000)
particles.push(Particle(...))                          # scatters fields into columns
pos = particles.position                               # MutSpan[Vec3] over the position column (mutable if `particles` is)
p   = particles[i]                                     # a *proxy* value of type SoA[Particle].Ref — field reads/writes hit columns
particles[i].lifetime -= dt                            # writes column `lifetime` at i; no Particle temporary
particles.get(i) -> Option[Particle]                   # gathers a copy

pos, vel = particles.columns_mut(position, velocity)   # two disjoint MutSpans in one call (compile-time field names)
```

* `[SOA-1]` `@derive(SoA)` on a struct `T` generates `SoA[T]` (a struct with one `Array[FieldType]` per field), the proxy types `SoA[T].Ref`/`RefMut`, `columns_mut`, and the `Iterable`/`IterableMut` impls yielding proxies. Nested `@derive(SoA)` structs are flattened recursively (`position.x` becomes its own column) when the field is marked `@soa(flatten)`; otherwise the nested struct is one column.
* `[SOA-2]` Column borrows are **disjoint places** for the borrow checker (`[BRW-4]` applies field-wise across columns), so two systems can hold `mut` borrows of different columns simultaneously.
* `[SOA-3]` `SoA[T]` provides `swap_remove`, `retain`, `sort_by_column`, `len`, `reserve`; element order is stable across all columns.
* `[SOA-4]` `ArenaSoA[T]` is the arena-backed view variant.
* `[SOA-5]` `columns_mut(f0, f1, …)` takes **field-name arguments**: identifiers resolved against `T`'s fields at compile time rather than as expressions. It is the only construct with this argument kind; a name that is not a field of `T` is `E2020`. This is an argument *kind*, not overloading (XXII.2 unaffected).

## XII.2 SIMD

```ember
from std.simd import f32x8, Mask8

a: f32x8 = f32x8.load(xs[i..i+8])                     # explicit vectors: f32x4/8/16, i32x4/8/16, u8x16/32, ...
b = a * f32x8.splat(2.0) + c
mask = a.gt(b)                                        # Mask8
r = mask.select(a, b)
r.store(mut out[i..i+8])
lanes = f32xN                                         # N = target-preferred width (comptime const)
```

* `[SIMD-1]` Vector types are plain `Copy` structs with `@align(width)`; on the C backend they lower to a per-compiler intrinsic shim (`ember_simd.h`: SSE/AVX/AVX-512/NEON via `immintrin.h`/`arm_neon.h`, with a scalar fallback); on LLVM to vector IR.
* `[SIMD-2]` `@simd` on a `for` loop is a **hint + diagnostic contract**: the compiler attempts vectorisation and, if `--emit-optimization-report` is on, reports success or the reason for failure (aliasing not provable, non-contiguous access, call in body, early exit). It never changes semantics. `@simd(assert)` upgrades failure to `E4020`.
* `[SIMD-3]` Alias information passed to the backend MUST be **derived, never assumed**. Two views MAY be marked `restrict` (C backend) or `noalias` (LLVM) only when one of: (a) the borrow checker related them to the same owner and proved their ranges disjoint; (b) they derive from owners the compiler proved distinct in the same function; (c) they are results of `split_at_mut`, `chunks_mut`, `columns_mut`, or are distinct `SoA` columns of one container; (d) they carry the disjointness fact established by `mem.assert_disjoint`/`assert_disjoint_all` (`[DSJ-3]`) or asserted by `unsafe assume_disjoint` (`[DSJ-7]`). A view whose base pointer entered the function through an FFI contract, through `Span.from_raw_parts`, or through any `unsafe` construction is **may-alias** by default and MUST receive no annotation until (d) is applied to it. **The mere fact that two views are simultaneously live is not a proof and MUST NOT be used as one**, and alias facts MUST NOT be derived from parameter modes: two view parameters of one function are disjoint only where (a)–(d) established it in the caller and the fact travelled with the value. This is what makes `@simd` work without user annotations in cases (a), (c) and (d), and what makes `assert_disjoint` necessary otherwise.
* `[SIMD-4]` Horizontal ops (`reduce_add`, `reduce_max`), shuffles (`shuffle[...]` const-generic lane lists), gathers/scatters, `fma`, `rsqrt`/`rcp` approximations (explicitly named `_approx`) are provided.
* `[SIMD-5]` **Vectorisable form.** On every backend, `@simd(assert)` is checked against a compiler-computed property called *vectorisable form*, never against the host compiler's decision. A loop is in vectorisable form iff all of: its trip count is computable before entry; every memory access in the body is to a view at a unit-stride affine index of the induction variable, or to a local; every written place is accessed — reads included — through index expressions that all share **one** constant offset (`[PAR-2]`(b)); every written view is proven disjoint from every other accessed view per `[SIMD-3]`; the body contains no call not inlined by `[CG-C-3]` or §4.12; the body carries none of `Alloc`, `Sync`, `Block`, `FFI`, `Panic(Explicit)` (`[EFF-16]`), or any `RuntimeCheck(k)` remaining after `[OPT-2]`; the body has exactly one exit; and every floating-point reduction is absent, declared by `@parallel(reduce=…)`, or the function is `@fastmath`. `E4020` is reported when a `@simd(assert)` loop is not in vectorisable form, and the diagnostic MUST name the first violated clause and, where the clause is a `RuntimeCheck`, its `[EFF-11]` reason code.
* `[SIMD-6]` The host compiler's own vectorisation report is **corroborating evidence only**. `ember build --emit-optimization-report` MUST print it when the toolchain produces one and MUST label it as the host compiler's opinion, distinct from the vectorisable-form verdict. Its absence, its format, and its disagreement with the verdict are never errors. Part X §3's `ember inspect` line reports the vectorisable-form verdict first and the host report second. `[DSP-5]` is amended the same way: a devirtualisation performed by the compiler is reported; one performed by the host linker under LTO is not required to be.

## XII.3 ECS (`std.ecs`, a library)

The ECS is a library over `SoA`, `Pool`, and comptime reflection, designed to match RageV's existing storage shape (sparse set per component; `Entity` = 20-bit index + 12-bit generation) so that the two can share entity IDs across the FFI boundary.

```ember
@derive(Copy, Component)
struct Position:
    value: Vec3

@derive(Copy, Component)
struct Velocity:
    value: Vec3

@derive(Component)
struct Name:
    value: String

world = World()
e = world.create()
world.add(e, Position(Vec3.ZERO))
world.add(e, Velocity(Vec3(1, 0, 0)))

@noalloc
fn integrate(mut q: Query[(mut Position, Velocity)], dt: f32):
    for pos, vel in q:                                 # pos: ref mut Position, vel: ref Velocity
        pos.value += vel.value * dt

world.run(integrate, dt)                               # borrows the storages named by the query type
```

* `[ECS-1]` `Entity` is `Handle[EntityTag]` with the 20/12 split; `Entity.NULL` is all-ones (same as RageV's `Null`).
* `[ECS-2]` Storage per component type is a sparse set: `dense: SoA[T]` or `Array[T]` (chosen by `@component(layout=soa|aos)`, default AoS to match RageV), `entities: Array[Entity]`, `sparse: Array[u32]` indexed by entity index. Component type identity is a 64-bit FNV-1a hash of the fully-qualified type name (`std.ecs.type_hash[T]()`), the same scheme RageV's `TypeHash` uses, so IDs are stable across modules and DLLs.
* `[ECS-3]` `Query[(A, mut B, Option[C], Not[D])]` is a view type over the world; it iterates the smallest storage among its required components and probes the others. Iteration yields `ref`/`ref mut` tuples; component access is two loads and an index (`[ECS-2]` layout). `[ECS-4]` The query's read/write set is a comptime constant; `World.run` and `World.run_parallel([sys1, sys2])` use it for access-set scheduling (`[JOB-3]`) and reject conflicting systems at compile time when both are known (`E7020`).
* `[ECS-5]` Structural changes (`add`/`remove`/`destroy`) during iteration go through a `Commands` buffer applied after the system returns (`q.commands().destroy(e)`), which keeps iteration borrow-safe.
* `[ECS-6]` Iteration order is insertion order with swap-remove holes, i.e. deterministic for an untouched scene (RageV's requirement 3 in `ECS.h`).
* `[ECS-7]` `@derive(Component)` registers the type in a comptime component registry used by serialisation and by the editor bridge (Part XXI).

---

# Part XIII — Error Handling

* `[ERR-1]` Recoverable errors are values: `Result[T, E]` with `E: Error`. `Option[T]` for absence.
* `[ERR-2]` `expr?` in a function returning `Result[U, F]` evaluates to the `Ok` payload or returns `Err(F.from(e))`; in a function returning `Option[U]`, `?` on an `Option` returns `None`. `?` on a `Result` in a function returning `Option` is `E2180`. **When the source error type and `F` are the same type, the conversion MUST be lowered to a move with no call.**
* `[ERR-3]` `std.error.Error` is an interface (`Display + Debug + source()`); `Box[dyn Error]` is the "any error" type; `@derive(Error)` on an enum generates `Display` from `@error("message {field}")` variant attributes and `From` impls from `@from` fields.
* `[ERR-4]` `Result` and `Option` are ordinary enums in `std.core` with the full combinator set (`map`, `and_then`, `unwrap_or`, `unwrap_or_else`, `ok_or`, `expect`, `is_some`, `as_ref`, `take`, `context("msg")`).
* `[ERR-5]` `@must_use` is applied to `Result`; ignoring a `Result` value is `W2190` (error under `-Dwarnings`).
* `[ERR-6]` FFI status codes are converted at the boundary by the binding (Part XVI §6): a C function returning `VkResult` with an `@ffi(status=VkResult, ok=VK_SUCCESS)` overlay becomes a `Result[void, VkError]` in Ember.
* `[ERR-7]` **Identity conversion.** `std.core` provides `extend[T] T implements From[T]: fn from(owned v: T) -> T: return v`. It is declared in the module that declares `From`, satisfying `[TYP-20]`, and does not overlap a user impl `From[A] for B` unless `A == B`, which no user may write (`E2041`). This is what makes `?` propagate an unchanged error type.
* `[ERR-8]` **Erasure conversion.** `std.core` provides `extend[E: Error] Box[dyn Error] implements From[E]`, so a function returning `Result[T, Box[dyn Error]]` may `?` any error. **`Box[dyn Error]` MUST NOT implement `Error`**: `[ERR-7]` and `[ERR-8]` would otherwise both supply `From[Box[dyn Error]] for Box[dyn Error]` and every use of the type would be `E2041`. Consequently `Error.source()` returns `Option[ref dyn Error]` (`[ERR-3]`) and never a `Box[dyn Error]`, and an already-erased error is re-erased by move under `[ERR-7]`.

```ember
@derive(Error, Debug)
enum AssetError:
    @error("file not found: {path}")
    NotFound(path: String)
    @error("io: {0}")
    Io(@from io.Error)
    @error("bad header at byte {offset}")
    Corrupt(offset: usize)

fn load(path: str) -> Result[Texture, AssetError]:
    bytes = fs.read(path)?                     # io.Error → AssetError.Io via From
    header = parse_header(bytes).ok_or(AssetError.Corrupt(0))?
    ...
```

---

# Part XIV — Compile-Time Programming

## XIV.1 `comptime`

* `[CT-1]` `comptime:` blocks and `comptime fn` functions are executed by the **MIR interpreter** (Part XVIII §7) during compilation. Any Ember function whose transitive effect set ⊆ `{Panic}` and that uses only `comptime`-supported operations may be called at compile time — there is no separate sub-language. Supported: all arithmetic, structs, enums, `Array`/`String`/`Map` (interpreted heap), `match`, loops, recursion, `assert`; the interpreter emulates target endianness, pointer size and `@layout(c)`.
* `[CT-2]` Not supported at compile time: FFI, threads, `unsafe` raw-pointer deref into non-interpreter memory, I/O except `comptime.read_file(path)` (path relative to the package; recorded as a build dependency) and `comptime.env(name)`.
* `[CT-3]` Limits: 10^8 MIR steps and 256 MB interpreter heap per `comptime` evaluation by default (`ember.toml [comptime]`); exceeding is `E6001`.
* `[CT-4]` `comptime` values are hashed into the module cache key with their inputs; determinism is required (`E6002` if two evaluations of the same block differ — checked in `--verify-comptime` CI mode).
* `[CT-5]` Results of `comptime` blocks are materialised as `static` data (arrays, strings, structs) — no runtime initialisation code is emitted.

```ember
const SIN_TABLE: [f32; 256] = comptime:
    t = [0.0; 256]
    for i in 0..256:
        t[i] = sin(i as f32 / 256.0 * TAU)
    t

comptime:
    assert(size_of[Vertex]() == 32, "Vertex layout changed; update the shader")
```

## XIV.2 Reflection

* `[RFL-1]` `reflect[T]()` (comptime) returns a `TypeDesc`: `{name, kind, size, align, fields: [FieldDesc{name, type: TypeDesc, offset, attrs}], variants, methods (v2), attributes}`.
* `[RFL-2]` Runtime reflection is opt-in with `@reflect` on the type: the compiler emits a `TypeInfo` table entry with field descriptors accessible via `type_info_of[T]()` / `h.type_info()` for class handles. Unused runtime metadata is dead-stripped by the linker (each table is its own section/COMDAT).
* `[RFL-3]` `@reflect` fields may carry user attributes readable at runtime (`@ragev.field(range=(0, 1), tooltip="…")`) — the editor bridge in Part XXI uses this exactly as RageV's `RVShowInEditor` markers are used by `rvgen` today.

## XIV.3 Derives

`@derive(...)` invokes compiler-built-in generators. v1 set: `Copy, Clone, Debug, Display(field="…")`, `Eq, Ord, PartialOrd, Hash, Default, SoA, Component, Error, Serialize, Deserialize, Zeroable, Reflect`. `[DRV-1]` Each derive is specified as an equivalent hand-written `extend` block in `std/derive/*.em` (the reference), and the generator MUST produce the same MIR. `[DRV-2]` User-defined derives (procedural macros) are **v2**; the mechanism will be a `comptime fn derive_X(t: TypeDesc) -> Source` sandboxed in the interpreter.

## XIV.4 Serialization

`@derive(Serialize, Deserialize)` generate `fn serialize(self, mut w: dyn Writer) -> Result[void, SerError]` / `fn deserialize(mut r: dyn Reader) -> Result[Self, SerError]` using a **binary, versioned, field-tagged** format (`std.ser.binary`) and a YAML mapping (`std.ser.yaml`, to interoperate with RageV's `yaml-cpp` scene files). Field attributes: `@ser(skip)`, `@ser(rename="…")`, `@ser(default)`, `@ser(version=2)`.

---

# Part XV — Standard Library Surface (v1)

The standard library is one package `std` with the modules below. Each module's public surface is listed at the level needed to implement it; exact signatures live in `std/**/*.em` and are the normative source once written. `[STD-1]` The whole of `std.core`, `std.mem`, `std.math`, `std.simd`, `std.span`, `std.arena` MUST be `@noalloc`-clean except functions documented to allocate.

| Module | Contents |
|---|---|
| `std.core` (prelude) | `Option`, `Result`, `Cell`, `RefCell` (`OQ-10`), marker & operator interfaces, `Ordering`, `Iterator` adaptors (`map`, `filter`, `enumerate`, `zip`, `take`, `skip`, `chain`, `rev`, `sum`, `count`, `min_by`, `max_by`, `fold`, `any`, `all`, `find`, `position`, `collect[C]`), `Range*`, `print`/`println`/`eprintln`, `assert*`, `panic`, `todo`, `unreachable`, `mem.{take, replace, swap, drop, forget, size_of, align_of}` |
| `std.mem` | `MaybeUninit`, `transmute`, `zeroed`, `copy`, `copy_nonoverlapping`, `Layout`, `Allocator`, `Global`, `ptr.*` (unsafe pointer ops), `keep_alive` |
| `std.cell` | `Cell`, `RefCell`, `Ref`, `RefMut` (Part IX §7) |
| `std.mem` (cont.) | `assert_disjoint`, `assert_disjoint_or_panic`, `unsafe assume_disjoint` (Part IX §8) |
| `std.collections` | `Array`, `SmallArray`, `Deque`, `Map`, `Set`, `BitSet`, `Pool`, `Handle`, `SoA` (derive support), `ArenaArray` |
| `std.string` | `String`, `str` methods (`len`, `chars`, `bytes`, `split`, `trim`, `starts_with`, `find`, `parse[T]`, `to_upper`…), `StringBuilder`, `CString`, `cstr` |
| `std.fmt` | `Formatter`, `Display`, `Debug`, `format(…) -> String`, `format_to(mut buf: MutSpan[u8], …) -> Result[str, FmtError]` (`@noalloc`), f-string lowering targets |
| `std.math` | `Vec2/3/4`, `IVec*`, `UVec*`, `Mat2/3/4`, `Quat`, `Transform`, `AABB`, `Sphere`, `Plane`, `Ray`, `Frustum`, scalar funcs (`sin cos tan atan2 sqrt rsqrt pow exp log floor ceil round abs min max clamp lerp smoothstep`), constants; all `@derive(Copy)`, `@layout(c)`, and **layout-compatible with GLM's float types** (`Vec3` = 3×f32, 4-byte aligned; `Mat4` column-major) so they cross to RageV unchanged |
| `std.simd` | vector types, masks, intrinsics (`std.cpu`: `prefetch`, `pause`, `rdtsc`, `cores`, cache line size) |
| `std.arena` | `Arena`, `FixedArena`, `ScopedArena`, `ThreadArena` |
| `std.io` | `Read`/`Write` interfaces, `stdin/stdout/stderr`, buffered wrappers, `Error` |
| `std.fs` | `read`, `write`, `File` (move-only, `drop` closes), `metadata`, `read_dir`, `Path`/`PathBuf` |
| `std.time` | `Instant`, `Duration`, `sleep`, `SystemTime` |
| `std.thread`, `std.sync`, `std.jobs`, `std.atomic` | Part XI |
| `std.process` | `exit`, `args`, `env`, `Command` (spawn child), `abort` |
| `std.ecs` | Part XII §3 |
| `std.ser` | binary + YAML (de)serialisation |
| `std.testing` | `@test` support, `assert_approx_eq`, `expect_panic`, `bench` harness |
| `std.ffi` | `CString`, `cstr`, `c_int`… type aliases (`c_int` = target `int`), `Callback[F]`, `Retained[T]` (foreign-retained handle), `ForeignBox[T]` (owned foreign pointer with destructor fn), `NativeApiTable` helpers (Part XXI) |
| `std.gpu` | Part XVII (host-side; backend-agnostic) |
| `std.debug` | `backtrace`, `leak_report`, `alloc_stats`, `frame_profiler` markers (`zone("name")` scoped) |

`[STD-2]` `print` and friends allocate only for f-strings; `println("literal")` and `println(some_str)` are `@noalloc`.
* `[STD-3]` `std.math` MUST provide scalar `fma(a: f32, b: f32, c: f32) -> f32` (and the `f64` form) and `Vec2/3/4.mul_add`, lowered to `fmaf`/`fma`/`_mm_fmadd_ps`/`vfmaq_f32`, so that fused multiply-add is expressible as an explicit, IEEE-defined operation **with no float-control attribute at all**. These are the recommended form for `Mat*` multiply, `dot` and transform composition, and `std.math` MUST use them internally.
* `[STD-4]` `std.core` provides `NonZero[T]` for each integer `T`: a `Copy` newtype with a niche (`Option[NonZero[T]]` is `T`-sized per `[TYP-13]`), constructed by `NonZero.new(v) -> Option[NonZero[T]]` or `unsafe new_unchecked`. Division by a `NonZero` divisor carries no `Panic(Explicit)`. This is the mechanism by which a divide in a `@nopanic(explicit)` function is expressible. `debug_assert*` is permitted in `release` and `shipping`, where it compiles to nothing.

---
# Part XVI — Foreign Function Interface

FFI quality decides whether Ember is usable for RageV at all. This part is therefore specified to the level of the generated code.

## XVI.1 Principles

* `[FFI-1]` **Every foreign declaration has a contract**: for each pointer-typed parameter and return, the binding records ownership (`borrowed | owned | retained`), mutability, nullability, and for the function as a whole: effects, thread rules, and whether it can call back into Ember. The generator derives what it can from the C declaration (`const T*` ⇒ borrowed immutable; `T*` ⇒ borrowed mutable; return `T*` ⇒ **unknown**) and marks everything else `unknown`.
* `[FFI-2]` Calling a foreign function whose contract contains any `unknown` requires an `unsafe` block. Supplying the missing facts through an **overlay** (§4) makes the call safe.
* `[FFI-3]` The compiler never links against C++ mangled symbols. All C++ access goes through generated `extern "C"` thunks compiled by the project's own C++ compiler (§7).
* `[FFI-4]` No Ember panic and no C++ exception crosses an ABI boundary (§9).
* `[FFI-5]` Layout is verified, not assumed: the generator records `sizeof`/`alignof`/field offsets for every imported struct as reported by Clang for the *project's* target and flags; the Ember compiler asserts its own computed layout matches (`E5001` with both numbers otherwise).

## XVI.2 Importing C

```ember
import c "vulkan/vulkan.h" with (
    include_paths = ["RageV/vendor/Vulkan-Headers/include"],
    defines       = ["VK_NO_PROTOTYPES", "VK_USE_PLATFORM_WIN32_KHR"],
    link          = [],                        # volk loads at runtime; nothing to link
    overlay       = "overlays/vulkan.em")      # contracts, see §4
import c "./physics_bridge.c"                   # a source file: compiled by the toolchain as a C target and its
                                                # matching header (same stem) imported
```

Pipeline (`[FFI-6]`):

```
header + flags ──► libclang (clang-sys) parse with the project's target triple, language standard, defines,
                   MSVC compatibility mode on Windows (-fms-compatibility -fms-extensions, same _MSC_VER as the
                   configured MSVC), include paths
              ──► Clang AST walk: functions, structs, unions, enums, typedefs, global variables,
                   object-like macros per `[FFI-6]`, function-like macros ignored (W5001)
              ──► Binding IR (BIR): a language-neutral description with layouts computed by Clang
              ──► overlay applied (contracts, renames, hides, wrappers, and the function-like
                   macros `[FFI-6b]` exposes by declaring a signature)
              ──► serialised to .embind (§5), cached under target/<triple>/bind/
              ──► Ember compiler reads .embind as a synthetic module `c` (or the name given by `as`)
```

`[FFI-7]` The header is re-parsed only when its content hash, the flags, or the overlay change.

### Type mapping (`[FFI-8]`)

| C | Ember |
|---|---|
| `void` | `void` (return) / `*void` (pointer) |
| `_Bool`/`bool` | `bool` |
| `char` | `c_char` (= `i8` or `u8` per target; `cstr` when `const char*` with overlay `@ffi(string)`) |
| `signed char`, `short`, `int`, `long`, `long long` | `i8`, `i16`, `c_int`(=`i32`), `c_long` (target), `i64` |
| unsigned variants | `u8`, `u16`, `c_uint`, `c_ulong`, `u64` |
| `size_t`/`ptrdiff_t`/`intptr_t`/`uintptr_t` | `usize`/`isize`/`isize`/`usize` |
| `float`/`double` | `f32`/`f64` |
| `T*` | `*mut T` (raw) by default; via contract: `ref mut T`, `MutSpan[T]` (+ length param), `Option[ref mut T]` (nullable), `ForeignBox[T]` (owned) |
| `const T*` | `*T`; via contract: `ref T`, `Span[T]`, `Option[ref T]`, `cstr` |
| `T[N]` field | `[T; N]` |
| `struct S` (complete) | `@layout(c) struct S` with fields; `Copy` iff all fields POD |
| `struct S` (opaque/incomplete) | `extern type S` (unsized, only behind pointers) |
| `union U` | `@layout(c) union U` (v1: fields accessible only in `unsafe`; `union` keyword is reserved and the generator emits it) |
| `enum E` | `@repr(c_int) enum E` (or the fixed underlying type) with all enumerators; non-exhaustive: `E.from_repr` never fails → `unknown` variant is added `@non_exhaustive` |
| `typedef` | `type` alias (or newtype if `@ffi(newtype)`) |
| function pointer | `extern "C" fn(...) -> R` (nullable ⇒ `Option[extern "C" fn…]`) |
| bit-fields | accessor methods `get_x`/`set_x` on the struct; the field itself is not addressable |
| variadic `...` | callable only inside `unsafe` with explicit argument types; `printf`-style functions get a `@ffi(format=1)` overlay to type-check literals (v2) |
| `#define N 42` | `const N: c_int = 42` (typed by literal suffix rules of C) |
| `#define S "x"` | `const S: cstr = c"x"` |
| `static const` globals | `extern static` (read requires `unsafe` unless `@ffi(immutable)`) |
| `__attribute__((packed))`, `#pragma pack` | `@packed`/`@align` reproduced |
| `restrict`, `volatile` | `volatile` pointers become `Volatile[*T]` with `read_volatile`/`write_volatile` |
| `wchar_t` | `c_wchar` (`u16` on Windows, `i32` on SysV — **`wcstr` is therefore not portable and this specification says so out loud**); `const wchar_t*` + `string` ⇒ `wcstr`, with `std.ffi` gaining `WString`, `str.to_wstring()` and `wcstr.to_string() -> Result[String, Utf16Error]` |
| `char16_t` / `char32_t` | `u16` / `char` |
| anonymous `struct`/`union` member | a generated nested type `<Parent>_anon<N>` **plus transparent field access**, so `p.LowPart` resolves through anonymous members exactly as in C. The generated name is stable across runs and is printed by `--explain` |
| flexible array member | the struct is unsized and MUST NOT be declared, copied, or passed or returned by value (`E5017`); the importer generates `unsafe fn tail(self, n: usize) -> Span[T]` and `tail_mut` |
| `va_list` | `VaList`: opaque, `!Copy`, `!Send`, `!Sync`, forwardable only to another C function taking `va_list`, inside `unsafe`. Constructing one is `E5018` |
| `long double`, `_Float128` | `c_longdouble`, an opaque byte blob with the target's size and alignment; no arithmetic and no literals |
| `_Atomic T` | `Atomic[T]` when the layouts match, else `E5016` naming both |
| `__int128` | `i128` / `u128` |
| function pointer, non-default convention | `extern "<conv>" fn(…)` per `[FFI-9]`; an unsupported convention is `W5002` with the declaration skipped |
| `enum` with no fixed underlying type | `@repr(c_int)` unless an enumerator exceeds `INT_MAX`, in which case `@repr(c_uint)` — **the chosen repr is recorded in the `.embind` and asserted by `[FFI-5a]`, because the choice is implementation-defined and MSVC and GCC do not always agree** |
| C identifier colliding with an Ember keyword | `r#name` (`[LEX-14a]`) |

### Calling convention

`[FFI-9]` `extern "C"` uses the platform C ABI (SysV AMD64, Windows x64, AAPCS64). `extern "system"` = `stdcall` on 32-bit Windows, `C` elsewhere. `extern "vectorcall"`, `extern "fastcall"` supported on x86. Struct passing by value follows the ABI exactly (the C backend gets this for free; the LLVM backend implements the classification rules in `ember_abi`).

## XVI.3 Manual declarations

```ember
unsafe extern "C":
    fn calculate_damage(base: f32, mult: f32) -> f32
    @ffi(link_name="vkCreateBuffer_volk")
    static vkCreateBuffer: Option[extern "C" fn(*VkDevice, *VkBufferCreateInfo, *VkAllocationCallbacks, *mut VkBuffer) -> VkResult]
```

`[FFI-10]` Manual `extern` blocks are always `unsafe` to declare (the programmer asserts the signature) and each function is `unsafe fn` to call unless annotated with a complete `@ffi` contract, in which case the compiler generates the same safe wrapper it would from an overlay.

## XVI.4 Overlays: contracts without editing headers

An overlay is an Ember file that annotates imported declarations by name. It is the mechanism for third-party headers (Vulkan, GLFW, Jolt's C API, ImGui's cimgui) whose sources cannot be modified.

```ember
## overlays/vulkan.em
overlay c "vulkan/vulkan.h":

    ## Opaque handles: typedef struct VkBuffer_T* VkBuffer  →  a Copy handle newtype
    @ffi(handle)            type VkBuffer
    @ffi(handle)            type VkDevice

    ## Status codes: functions returning VkResult become Result[.., VkError]
    @ffi(status, ok=VK_SUCCESS)  enum VkResult

    ## Contracts on a function: parameter names refer to the C declaration's parameters
    @ffi(effects=[FFI, Alloc])
    fn vkCreateBuffer(device: borrowed, pCreateInfo: borrowed ref VkBufferCreateInfo,
                      pAllocator: nullable borrowed, pBuffer: out) -> status

    @ffi(effects=[FFI], destroys=pBuffer_of(vkCreateBuffer))
    fn vkDestroyBuffer(device: borrowed, buffer: owned, pAllocator: nullable borrowed)

    ## Pointer + length pair → Span
    fn vkCmdSetViewport(commandBuffer: borrowed, firstViewport, viewportCount: len_of(pViewports),
                        pViewports: span)

    ## Callback registration that retains the user pointer
    fn vkSetDebugUtilsObjectNameEXT(...): unsafe            # leave unsafe; too irregular

    ## Rename / hide
    rename VkPhysicalDeviceFeatures2 as PhysicalDeviceFeatures2
    hide vkAllocationFunction                                  # do not expose
```

Contract vocabulary (`[FFI-11]`):

| Contract | Meaning / generated Ember type |
|---|---|
| `borrowed` | pointer valid for the call only; `ref T` / `ref mut T` (mutability from `const`) |
| `nullable` | `Option[ref T]` |
| `owned` | callee takes ownership; Ember passes a `ForeignBox[T]`/handle by move |
| `returns_owned(destructor=f)` | return value is owned by Ember; wrapped in `ForeignBox[T]` whose `drop` calls `f` |
| `retained` | callee stores the pointer; the argument must be `Retained[T]` (pins an Ember-owned object; §8) |
| `out` | write-only out-parameter; becomes a return value (tuple/`Result` payload) |
| `span`, `len_of(p)` | pointer + length pair → `Span[T]`/`MutSpan[T]` |
| `string` | `const char*` → `cstr` (NUL-terminated, borrowed) |
| `handle` | opaque pointer typedef → `Copy` newtype with `NULL` |
| `status, ok=X` | return code enum → `Result[…, XError]` |
| `effects=[…]` | effect set (`[EFF-3]`) |
| `threads=any \| main \| creator` | which thread may call |
| `callback=borrowed \| retained \| once` | function-pointer parameter lifetime (§8) |
| `destroys=<param>` | documents pairing; used by the leak checker |
| `unsafe` | leave the function unsafe to call |

`[FFI-12]` An overlay declaration whose C signature does not match the header (wrong parameter count/names/types) is `E5010`, so overlays cannot drift silently. `[FFI-13]` Overlays may also add Ember-side **wrapper methods** (`extend VkDevice: fn create_buffer(self, info: BufferCreateInfo) -> Result[VkBuffer, VkError]: ...`) written in ordinary Ember, which is how idiomatic bindings are layered on top of raw ones.

## XVI.5 The `.embind` artefact

A `.embind` file is a CBOR document (schema versioned; `embind_version = 1`) containing: the source header path and content hash; the effective Clang configuration (target triple, `-std`, defines, include paths, MSVC version); the BIR (all declarations with computed layouts); the overlay hash; the contract for every declaration; C++ thunk list (§7). `[FFI-14]` `ember bind --emit-embind header.h` writes it; `ember bind --explain header.h::fn` prints the derived contract and the reasons. The file is a build input like any other and is content-addressed in the build cache.

## XVI.6 Strings and status codes at the boundary

* `cstr` (borrowed NUL-terminated) and `CString` (owned, NUL-terminated, allocates once). `str.to_cstring()` allocates; `cstr.to_str() -> Result[str, Utf8Error]` validates without copying. `[FFI-15]` Passing a `str` to a `cstr` parameter is a compile error (`E5020`, "not NUL-terminated") — the fix-it is `.to_cstring()`, or `c"literal"`.
* Status codes: `[FFI-16]` an `@ffi(status)` enum `E` generates `struct EError(code: E)` implementing `Error`, and every function returning `E` returns `Result[T, EError]` where `T` is the tuple of `out` parameters (or `void`).

## XVI.7 Importing C++

```ember
import cpp "RageV/src/RageV/Renderer/RHI/RHIDevice.h" with (
    project = "ragev",                       # takes flags from [cpp.ragev] in ember.toml (compiler, standard, defines, includes)
    classes = ["RageV::RHIDevice", "RageV::RHICommandList"],
    instantiate = ["std::vector<float>", "RageV::Handle<RageV::Texture>"],
    overlay = "overlays/rhi.em")
```

`[FFI-17]` The C++ importer produces, for each requested class/function:

1. A **thunk file** `target/bind/<hash>_thunks.cpp` containing `extern "C"` functions with predictable names (`em_cpp_<ns>_<class>_<method>_<sig-hash>`) that (a) call the C++ member/free function, (b) catch all exceptions and return a status + message buffer (§9), (c) never return C++ objects by value across the boundary — objects are heap-allocated (`new`) and returned as opaque owned pointers, or written into caller-provided storage when trivially copyable.
2. Ember declarations in the synthetic module: an `extern type RHIDevice`, a `struct RHIDeviceRef`/`ForeignBox[RHIDevice]` pairing, and methods on them that call the thunks. Constructors become `RHIDevice.new(...) -> ForeignBox[RHIDevice]`; the destructor is the box's `drop`. Overloads are disambiguated by suffixing parameter types (`draw_indexed_u32`) unless the overlay names them.
3. Templates are only available as **explicit instantiations** listed in `instantiate` or referenced by an imported signature; each instantiation gets its own thunks. `std::vector<T>` maps to `CppVector[T]` with `.span()`/`.span_mut()` (zero-copy, borrowing the vector), `.to_array()` (copy), `push_back`, `len`; `std::string` to `CppString` with `.as_str()`; `std::string_view` to `str`; `std::unique_ptr<T>` to `ForeignBox[T]`; `std::shared_ptr<T>` to `CppShared[T]` (calls the C++ control block through thunks); `std::optional<T>` to `Option[T]` for trivially copyable `T`; `std::function` is not importable (`E5030`) — use a C callback.
4. **Compiler matching** `[FFI-18]`: the thunks are compiled by the *same* compiler, standard and flags as the project (`[cpp.ragev]` section), so they share the C++ ABI with the engine (MSVC ABI when the engine is built with MSVC; clang-cl also targets the MSVC ABI). libclang parses the headers in MSVC-compatibility mode with the same `_MSC_VER`. `[FFI-19]` If the project's compiler is MSVC and libclang cannot parse a header (MSVC-specific extension), the importer reports the exact diagnostic and the declaration is skipped with `W5031`; the programmer then binds it manually through a small C shim.
5. Inheritance and virtual functions: a C++ class hierarchy is imported as opaque types with **upcast thunks** only; Ember cannot subclass C++ classes in v1. Overriding a C++ virtual from Ember is done through a generated C++ "trampoline subclass" listed in `overlay` with `@ffi(trampoline)` (v2).
6. Namespaces map to Ember module paths under the import name: `cpp.RageV.RHIDevice`.
7. `constexpr`/`const` integral constants and unscoped/scoped enums import like C.

`[FFI-20]` Everything an importer cannot represent is reported once with the reason (`ember bind --report`), never silently dropped.

## XVI.8 Callbacks and foreign retention

* `[FFI-21]` A C parameter of function-pointer type accepts: a capture-free Ember `fn` (coerced to `extern "C" fn`), or an `extern "C" fn` value. Capturing closures are rejected (`E5040`) unless the API has a `void* user_data` parameter matched by the overlay contract `callback=…, user_data=<param>`, in which case the generator produces the trampoline: it boxes the closure, passes the box as `user_data`, and generates the `extern "C"` shim that unboxes and calls it.
* `callback=borrowed`: the closure is valid for the call; the box is freed after return. `callback=retained`: the API stores it; Ember returns a `Retained[Callback]` token that must be kept alive by the caller (dropping it unregisters via the paired `destroys=` function if declared, else `E5041 retained callback needs a release function in the overlay`). `callback=once`: freed by the shim after the first invocation.
* `[FFI-22]` **Foreign threads.** Any `extern "C"` function exported from Ember (§XVI.10) or any generated trampoline begins with `ember_rt_thread_attach()` (idempotent, cheap after the first call: a TLS flag) so that thread-local allocators and panic state exist. Detachment happens automatically on thread exit via a TLS destructor. Class handles must not be passed to foreign threads unless the class is `Sync` (`[THR-2]` is enforced at the trampoline's capture check).
* `[FFI-23]` `Retained[T]`: to hand a pointer to an Ember-owned object to C that will keep it, wrap it: `tok = Retained.pin(obj)` (for a class handle: bumps the strong count and sets the header `pinned` flag; for a `Box`: takes ownership into the token). `tok.ptr()` is the raw pointer. Dropping the token releases. This is the only mechanism for foreign retention of Ember-owned memory; there are no GC-style pinning handles.

## XVI.9 Errors, panics and exceptions at the boundary

* `[FFI-24]` Generated C++ thunks have the shape:
  ```cpp
  extern "C" int32_t em_cpp_X_method(X* self, Args..., EmberCppError* err) noexcept {
      try { self->method(args...); return 0; }
      catch (const std::exception& e) { ember_cpp_error_set(err, typeid(e).name(), e.what()); return 1; }
      catch (...)                     { ember_cpp_error_set(err, "unknown", ""); return 1; }
  }
  ```
  The Ember side returns `Result[T, CppError]` for every C++ call unless the overlay says `@ffi(noexcept)`, in which case the thunk is `noexcept` and a throw terminates (documented).
* `[FFI-25]` Ember functions exported to C run under a **panic boundary**: with `panic=abort` a panic aborts (as anywhere); with `panic=unwind` (v2) the exported wrapper catches, logs, and returns the declared error value (`@export(on_panic=return -1)`).

## XVI.10 Exporting Ember to C and embedding the runtime

```ember
@export("rv_script_on_update")                          # stable C symbol, C ABI
pub extern "C" fn on_update(entity: u64, dt: f32) -> i32: pass

@export_table("RvScriptApi", protocol=3)                # a function-pointer table struct, RageV style (§Part XXI)
pub struct ScriptApi:
    on_create: extern "C" fn(u64) -> i32
    on_update: extern "C" fn(u64, f32) -> i32
```

* `[FFI-26]` `ember build --emit-header` writes `<package>.h` with prototypes for every `@export`, C typedefs for every `@layout(c)` type used in them, and `#define <PACKAGE>_PROTOCOL_VERSION n`. `--emit-header` MUST emit the thread contract of each `@export` as a doc comment on its prototype, so the host's C++ author reads the rule at the call site.
* `[FFI-27]` The Ember runtime (`ember_rt`) is a C11 static library with no global constructors. Embedding API: `ember_rt_init`/`ember_rt_shutdown` are the *process* lifecycle used by an executable or a `staticlib` host. A `cdylib` host uses `[FFI-31]` instead and MUST NOT call `ember_rt_shutdown` directly.
  ```c
  ember_rt_config cfg = ember_rt_config_default();
  cfg.alloc = my_malloc; cfg.free = my_free;            // optional: route allocations to the host allocator
  cfg.log = my_log;                                     // panic/log sink
  ember_rt_init(&cfg);                                  // once per process; idempotent
  ember_rt_thread_attach();                             // per thread that will call Ember (idempotent)
  ...                                                   // call exported functions / tables
  ember_rt_shutdown();                                  // runs at-exit hooks, leak report in debug
  ```
* `[FFI-28]` An Ember **package built as `kind = "cdylib"`** produces a DLL/.so exporting only `@export` symbols plus `ember_module_init(const ember_host_api*)` and `ember_module_protocol()`; built as `kind = "staticlib"` it produces a `.lib/.a` plus header, suitable for linking into RageV's static-library build directly.
* `[FFI-31]` **Module lifecycle for `kind = "cdylib"`.** A `cdylib` links a private copy of `ember_rt`. In addition to `[FFI-28]`'s symbols it MUST export `int ember_module_init(const ember_host_api*)` (configures and calls this module's `ember_rt_init`; idempotent) and `void ember_module_shutdown(void)`, which the host MUST call before unloading. `ember_module_shutdown` MUST run the module's at-exit hooks, detach every thread this module attached, release every thread-local allocator, arena and TLS slot the module owns, and emit the leak report in `debug` (ADR-007's report is load-bearing, and this is where a plugin's runs). After it returns, no code or data of the module may be reachable from any thread.
* `[FFI-31a]` `[FFI-22]` is amended: when the package kind is `cdylib`, automatic detachment MUST NOT be implemented with a TLS destructor whose code resides in the module. The runtime keeps an intrusive list of attached threads and detaches them in `ember_module_shutdown`.
* `[FFI-31b]` **No owning Ember value may cross a module boundary.** Because each `cdylib` owns a private allocator and type-info table, class handles, `Box`, `Shared`, `Weak`, `String`, `Array`, `Map`, and any type with drop glue MUST NOT appear in an `@export` or `@export_table` signature, nor be reachable through a pointer in one. Only `@layout(c)` value types, scalars, opaque handles, `cstr` (copied at the boundary) and `extern "C" fn` may cross. Violation is `E5015`, naming the offending type and the reason. This makes Part XXI's `[RV-2]` a language rule rather than a plan convention, and follows directly from XXII.2's "no stable Ember-to-Ember ABI".
* `[FFI-31c]` A host that must share one runtime between several Ember modules links `ember_rt` as a shared library and builds each module with `[build] runtime = "shared"`. This is the only configuration in which `[FFI-31b]` may be relaxed, and it is **v2**; v1 packages are always privately linked.
* `[FFI-33]` **Thread contracts on exported functions.** `@export` and `@export_table` accept `threads = any | main | creator` with `[FFI-11]`'s meanings, defaulting to `any`; a `@export_table` MAY set it per field; a module MAY declare a default with `#! threads main` (which is what Part XXI §1 assumes). `threads = any`: the compiler checks the exported function's reachable call graph as if it ran on an arbitrary thread — every `static` it reaches MUST be `Sync`, and no non-`Sync` class handle may be reachable from a `static`, a parameter or a captured value; violation is `E7010` naming the reached item and why it is not `Sync`, with `#! threads main` named as the fix. `threads = main`: the exported wrapper asserts, in `debug` and `release`, that the calling thread is the one that called `ember_module_init`/`ember_rt_init`, and panics `ember_panic_thread` otherwise; in exchange the body MAY touch non-`Sync` statics and handles. `threads = creator`: as `main`, but the asserted thread is the one that created the value the call is dispatched on.
* `[FFI-33a]` `[FFI-22]` is amended: `ember_rt_thread_attach()` establishes thread-local runtime state and confers **no** right to touch thread-confined data. In the `debug` profile, under the existing `debug_objects` profile key, an extended object header records the attaching thread id, and `ember_retain`/`ember_release` on a non-`Sync` object from a different thread panics naming the class and both thread ids. **`[OBJ-1]`'s release header layout is unchanged**: the 24 bytes are fully occupied and a thread id does not fit, so this is a debug-only extension, never a change to the ABI `[VER-4]` freezes.
* `[FFI-33b]` `returns_owned(destructor=f)`, `[FFI-23]`'s `Retained[T]` and `Callback[F]` MAY carry `threads=`. A `ForeignBox[T]` whose destructor is `threads = creator` records the creating thread on construction and panics in `debug` when dropped elsewhere; this is what makes GPU, GL-context and COM handles safe to hold in ordinary Ember values. `ember bind --report` lists every foreign destructor with no thread contract.
* `[FFI-20a]` `[FFI-20]` applies to the **C** importer exactly as to the C++ importer. Every declaration the C importer cannot represent MUST be recorded in the `.embind` with the construct that defeated it and a suggested workaround, and reported by `ember bind --report` as `W5002 declaration not imported: <name> — <construct> — <workaround>`. No declaration is ever silently absent from a synthetic module. **A reference to a name that was skipped MUST produce a diagnostic saying it was skipped and why, not "unknown identifier"** — which requires carrying the skip list into the synthetic module's namespace as tombstones.
* `[FFI-29]` **Generated shim translation unit.** For every `import c`, the importer MUST emit `target/<triple>/bind/<hash>_shim.c`, compiled by the project's C compiler with that import's own filtered flags (`[BLD-FFI-1a]`) and linked into the package, containing: (a) for every function with internal linkage or no external definition (`static`, `static inline`, `inline`, `__forceinline`) reachable from the header, an external wrapper `<ret> em_inl_<name>(<params>) { return <name>(<args>); }` — the binding refers to `em_inl_<name>`, and `ember bind --explain` prints the wrapping; (b) the function-like macro wrappers of `[FFI-6b]`; (c) the layout assertions of `[FFI-5a]`.
* `[FFI-29a]` The importer MUST NOT emit a binding that names a symbol with internal linkage. A binding that resolves to no external symbol is a defect: it produces a link error with no Ember diagnostic and no source location.
* `[FFI-29b]` **Single-header libraries.** `import c "miniaudio.h" with (implementation = ["MINIAUDIO_IMPLEMENTATION"])` causes the toolchain to emit `target/<triple>/bind/<hash>_impl.c` containing the `#define`s followed by the `#include`, compiled with the import's flags and linked **exactly once per package**. Two packages in one build requesting the same implementation macro for the same header is `E5014`, naming both; the resolution is for one to expose the library and the other to depend on it.
* `[FFI-29c]` A wrapped call costs one non-inlined call unless the shim TU participates in LTO. `[FFI-9]`'s and M5's "ABI-direct call, zero extra instructions" property applies to externally-defined functions only; `ember inspect` MUST report a wrapped foreign function as `wrapped (static inline)` so the cost is visible. A variadic `static inline` function cannot be wrapped and falls to `W5002`.
* `[FFI-30]` **Identity of imported entities.** Two imported declarations denote the same Ember entity iff they have the same **C identity** — Clang's USR for the declaration, resolved through typedefs to the underlying tag — and the same resolved BIR layout. Identity is independent of the importing module, package, alias and overlay. Two imports of the same C identity with different resolved layouts in one build are `E5011`, listing both flag sets and the first differing field, **with help naming the route out: give one import a distinct alias, accepting that its entities are then distinct and values do not interchange.** (Compiling one library twice with different `-D` sets in one build is legitimate.)
* `[FFI-30a]` An overlay is a **view, not a type constructor.** It changes the signatures, safety and names of imported *functions* and may add wrapper items (`[FFI-13]`), but it MUST NOT change the identity, layout or field set of an imported *type*. `@ffi(handle)`, `@ffi(newtype)` and `rename` produce Ember-side aliases over the same underlying entity. Two packages may therefore carry different overlays for the same header and still exchange `VkDevice` values.
* `[FFI-30b]` `pub import c "…" as vk` re-exports the synthetic module. `[MOD-2]`'s visibility rules apply to a synthetic module exactly as to any other, so a package MAY expose imported types in its public API and a dependant reaches them as `ember_vulkan.vk.VkDevice` without re-importing the header. `[BLD-2]`'s interface hash MUST include the imported entity's layout, so a header change invalidates dependants.
* `[FFI-30c]` **Distributable, composable overlays.** A package MAY distribute an FFI overlay for an imported foreign module, and multiple overlays for the same foreign module MAY be composed in a declared left-to-right order. An overlay MAY add Ember-side aliases, wrappers, metadata, or other permitted bindings and MAY use explicit `override` where the overlay contract allows replacement. Composition MUST diagnose incompatible definitions rather than silently selecting one, and MUST NOT change the identity, layout, field set, or foreign ABI identity of an imported foreign type. The composed overlay identity MUST participate in the build/interface identity used to invalidate dependants when the overlay changes (`[BLD-2]`, `[BLD-3]`).
* `[FFI-32]` **C++ value types.** A C++ class or struct that is standard-layout and trivially copyable, and every one of whose non-static data members maps under `[FFI-8]`, MUST be imported as an Ember `@layout(c) struct` carrying `@derive(Copy)` — **not** as an `extern type`. Public data members become `pub` fields; non-public members become private fields of the same type and offset, so the Ember type is bit-identical. Every such type carries a `[FFI-5a]` layout assertion. Values of it cross thunks by value with no allocation and satisfy `[TYP-11]`. `[FFI-32a]` A class that is standard-layout but **not** trivially copyable is imported as an opaque `extern type`, as today. An overlay MAY force a value mirror with `@ffi(value) class N::C` when the programmer asserts Ember only reads its bits; the mirror is `!Drop` and constructing one is `E5032`. `[FFI-32b]` `const T&` maps to Ember `T` (borrowed mode) when `T` is a value type, and to `ref T` over the opaque type otherwise; `T&` maps to `ref mut T`/`mut` mode; a class parameter **by value** maps to `owned T` for value types and is `E5031` otherwise; `T&&` is `E5031` in v1. `[FFI-32c]` **Default arguments.** Where a C++ default argument is a constant expression, the importer MUST map it to an Ember default argument (`[FN-5]`) with the same value. Otherwise it MUST emit one thunk per arity reachable by omitting trailing defaulted parameters, exposed as Ember default arguments forwarding to the shorter thunk, so a call omitting the argument receives **C++'s own default expression evaluated on the C++ side**. Thunks MAY be emitted lazily for arities Ember actually calls. `[FFI-32d]` Template instantiations follow the same rule: an instantiation satisfying `[FFI-32]` (e.g. `RageV::Handle<RageV::Texture>`) is a value type. `[FFI-17]` item 3's STL mappings are unchanged and take precedence for the named STL types. `[FFI-32e]` `ember bind --report` MUST list, for every class in `classes = […]`, whether it imported as a value type or as opaque, and for opaque types the specific disqualifying property (`non-trivial destructor`, `virtual base`, `member of unmapped type T`), because that determines the cost of every call that touches it.
* `[FFI-11]` Every pointer-typed parameter or return that a contract makes safe MUST carry a **count** axis:
* `[FFI-11a]` `borrowed` alone no longer implies `one`. An overlay supplying ownership, mutability and nullability for a pointer parameter but no count axis is `E5012 pointer contract has no count`. The diagnostic MUST list the five count contracts and MUST name any sibling parameter whose name or type suggests a length (a `uint32_t`/`size_t` parameter, or one whose name ends `Count`, `Len`, `Size`, `N`).
* `[FFI-11b]` The **two-call enumeration idiom** has a named contract: `count: inout_count` paired with `items: span(len_of(count), nullable)`. The generated signature is `fn f(…, items: Option[MutSpan[T]]) -> Result[u32, E]`; the wrapper passes `null` and forwards the caller's count when `items` is `None`, and passes `items.as_mut_ptr()` with `items.len()` otherwise, returning the count the callee wrote or requires.
* `[FFI-2a]` Contracts the generator **derives** from the C declaration are verified facts and MUST NOT require `unsafe`: `const T*` ⇒ borrowed immutable, `T*` ⇒ borrowed mutable, struct/union layouts asserted per `[FFI-5]`, enum underlying types, `@packed`/`@align` reproduction, calling convention. Only `unknown`-filling contracts require it: `owned`, `returns_owned`, `retained`, `span`, `len_of`, `string`, `nullable`, `handle`, `status`, `threads=`, `effects=`, `noexcept`, `callback=`, and `noalias` (`[SIMD-3]`). An overlay containing none of these needs no `unsafe`.
* `[FFI-11c]` `TODO(count)`, `TODO(nullable)`, `TODO(ownership)` are part of the **overlay language**, not merely of the tool. `[FFI-11]`'s vocabulary and `[FFI-12]`'s signature check MUST accept them, and `[FFI-2]` MUST treat a declaration carrying one as **uncontracted** — its calls stay `unsafe`. This is what lets a team adopt a header on day one, run everything behind `unsafe:`, and buy safety incrementally rather than writing 2000 lines before the first call compiles.
* `[FFI-6]` **Macro import.** The importer MUST import an object-like macro whose replacement list, **after full macro expansion**, is a constant expression of integer, floating, string-literal, **null-pointer-constant, or pointer/handle-cast** type as evaluated by Clang under the import's own configuration (a probe TU of one `static const` or `enum` per candidate macro, parsed with the same configuration and cached in the `.embind`). The macro's Ember type is the C type of the evaluated expression after the usual arithmetic conversions. This covers `(~0U)`, `(-1)`, `(1 << 3)`, `sizeof(T)`, `0xffffffffffffffffULL`, `((VkBuffer)0)` — which maps to the imported handle newtype's NULL per `[FFI-11]`'s `handle` contract — and object-like macros whose bodies expand through function-like macros.
* `[FFI-6a]` A macro that does not evaluate to such a constant is skipped with `W5001 macro not imported: <reason>`, where `<reason>` is Clang's evaluation failure — never a blanket category. The macro, its body and its reason MUST appear in the `.embind` and in `ember bind --report`.
* `[FFI-6b]` A **function-like** macro MAY be exposed as a function when an overlay declares its signature (`@ffi(macro_fn) fn VK_MAKE_API_VERSION(variant: u32, …) -> u32`). The importer emits `uint32_t em_mac_VK_MAKE_API_VERSION(…) { return VK_MAKE_API_VERSION(…); }` into the generated shim translation unit (`[FFI-29]`, RFC-043) and binds `em_mac_*` as an ordinary `extern "C"` function. **This adds no macro facility to Ember: the imported entity is a C function.** Where all arguments are compile-time constants the importer MAY fold the call through `[FFI-6]`'s probe TU, so `VK_MAKE_API_VERSION(0,1,3,0)` is usable in a `const` initialiser.
* `[FFI-5a]` In addition to the libclang cross-check, the importer MUST emit into the generated shim TU (`[FFI-29]`) or thunk TU (`[FFI-17]`) a `_Static_assert`/`static_assert` for `sizeof` and `_Alignof` of every imported aggregate that Ember constructs, passes or returns by value, or embeds, and for `offsetof` of every field Ember reads or writes. **The layout contract is therefore verified by the compiler that builds the project, on the user's machine, on every build**, and a mismatch is a build error naming the type, the field and both offsets (`E5001`). XVIII §10's cross-compiler tests remain and cover the compiler's own fixtures.

## XVI.11 Build integration

* `[BLD-FFI-1]` `ember.toml` `[cpp.<project>]` records `compiler` (`msvc | clang-cl | clang | gcc`), `standard`, `defines`, `include_paths`, `flags`, `libs`, and optionally `cmake = { build_dir = "build", target = "RageV" }` from which the toolchain reads `compile_commands.json` / the CMake File API to obtain the exact flags of the target — this is the mechanism that guarantees `[FFI-18]`.
* `[BLD-FFI-2]` `cmake/EmberModule.cmake` (shipped with the toolchain) provides `ember_add_library(name SOURCES ... KIND staticlib|cdylib)` and `ember_add_executable(...)`, which invoke `ember build` with `--cc-flags-from-target <cmake-target>` and add the produced C files (C backend) as an OBJECT library to the CMake target graph so that MSVC/clang compile them with the same flags, LTO and debug settings as the rest of the engine. With the LLVM backend it instead adds the produced `.obj`/`.o`.
* `[BLD-FFI-3]` Existing static/shared libraries are consumed by listing them in `link = [...]` or by inheriting the CMake target's link interface.
* `[BLD-FFI-1a]` **Flag inheritance is defined, not copied.** When flags come from a CMake target, the toolchain takes the command line of a representative TU and applies this filter. **On Windows the CMake File API is normative**; `compile_commands.json` is used only when the generator produces it. *Dropped:* precompiled-header switches (`/Yc`, `/Yu`, `/Fp`, `/FI`, `-include`, `-include-pch`); output and dependency paths (`/Fo`, `/Fd`, `-o`, `-MD`, `-MF`, `/showIncludes`); whole-program and PGO switches (`/GL`, `/LTCG`, `-flto`, `/analyze`); warning and diagnostic switches; the source operand. ***Every other switch is inherited*** — compilers add switches faster than a specification is revised, so the drop-list is the closed set and the default is inheritance. Explicitly inherited and individually load-bearing: `-D`/`/D`; include paths; `-std`/`/std`; architecture and ISA; the MSVC runtime library (`/MD`, `/MDd`, `/MT`, `/MTd`); the exception model; RTTI; `/Zc:*`; `-fms-compatibility-version`/`_MSC_VER`; structure packing; `-fshort-enums`; `char` signedness. `E9020` if the TUs of one target disagree on any inherited flag — **with an escape hatch, because real targets legitimately carry per-file `-D` overrides: an explicit `[cpp.<project>] flags` entry in `ember.toml` wins over inference, and `E9020`'s help MUST name it.**
* `[BLD-FFI-1b]` The MSVC runtime-library switch and the effective values of `_DEBUG` and `_ITERATOR_DEBUG_LEVEL` MUST be inherited byte-for-byte, because they change the layout of `std::string`/`std::vector` and the identity of the CRT heap. If the toolchain cannot determine them it MUST fail with `E9021`, never default.

---

# Part XVII — GPU Host Model and Shader Interface

Ember does not compile shaders in v1. It specifies the host-side facilities that RageV's renderer needs, as a library (`std.gpu`) over an abstract device interface that the engine implements in C++ (its existing RHI) and exposes through the FFI, or that a future Ember-native backend implements directly. Nothing here is engine-specific; everything maps one-to-one onto RageV's RHI shape (`BeginFrame → record → EndFrame`, per-frame-in-flight resource sets, deferred destruction).

## XVII.1 Handles

```ember
type TextureHandle  = Handle[gpu.Texture]
type BufferHandle   = Handle[gpu.Buffer]
type PipelineHandle = Handle[gpu.Pipeline]
type ResourceSetHandle = Handle[gpu.ResourceSet]
```

`[GPU-1]` All GPU objects are referred to by generational handles (Part IX §6); physical objects live in the device's pools. Handles are `Copy`, `Send`, and safe to store anywhere; a stale handle is detected by generation (`E-panic: stale TextureHandle (gen 7, current 9)` in debug/release, unchecked in shipping only with `gpu.validate = false`).

## XVII.2 Device and frame lifecycle

```ember
interface Device:                                     # implemented by the RHI bridge
    fn begin_frame(mut self) -> Option[Frame]         # None ⇒ skip this frame (resized/minimised); do NOT call end_frame
    fn end_frame(mut self, owned frame: Frame)
    fn create_texture(mut self, desc: TextureDesc) -> Result[TextureHandle, GpuError]
    fn create_buffer(mut self, desc: BufferDesc) -> Result[BufferHandle, GpuError]
    fn destroy(mut self, h: TextureHandle)            # logical destruction; physical destruction is deferred (§4)
    fn frames_in_flight(self) -> u32
    fn caps(self) -> DeviceCaps
    ...

struct Frame:                                         # move-only; @view over the device
    index: u32                                        # frame-in-flight slot
    fn command_list(mut self) -> ref mut CommandList
    fn arena(self) -> ref Arena                       # the frame arena (reset at end_frame)
```

* `[GPU-2]` `begin_frame` returning `None` is normal; because `Frame` is move-only and `end_frame` consumes it, the RageV trap "calling EndFrame after a null BeginFrame" is a type error rather than a runtime bug.
* `[GPU-3]` `Frame.arena()` is the per-frame transient allocator; its region is the `Frame`, so nothing allocated from it can survive `end_frame`.

## XVII.3 Command recording and resource access states

```ember
with cmd = frame.command_list():
    cmd.begin_render_pass(RenderPassBegin(target=None))   # None ⇒ swapchain
    cmd.bind_pipeline(pipeline)
    cmd.bind_resource_set(0, set)
    cmd.bind_vertex_buffer(0, vbo)
    cmd.bind_index_buffer(ibo, IndexType.U32)
    cmd.draw_indexed(index_count)
    cmd.end_render_pass()
```

Each resource carries a runtime **access state** tracked by the device:

```
CpuOwned ──(cmd.read/write, bind)──► Recording(frame N) ──(end_frame)──► InFlight(frame N)
   ▲                                                                         │
   └──────────────────(frame N's fence signalled at begin_frame(N + frames_in_flight))──┘
Retired: logically destroyed while InFlight ⇒ physical destruction deferred until CpuOwned
```

* `[GPU-4]` Uploading to or mapping a resource (`device.write_buffer(h, data)`, `device.map(h)`) while it is `InFlight` is an error `E-panic: buffer is in flight (frame 42); use a per-frame ring or wait` in debug/release. The engine's existing per-frame-in-flight duplication (`RHIResourceSet` holds one set per frame) is expressed as `Ring[T, N]` in `std.gpu` (`ring.current(frame)`).
* `[GPU-5]` `cmd.read(h)`/`cmd.write(h)` are optional explicit declarations that let the device derive barriers/transitions (declarative state tracking); binding a resource implies the appropriate declaration. Explicit `cmd.barrier(...)` remains available, and `unsafe: cmd.native()` returns the raw backend command buffer for vendor extensions (`VkCommandBuffer`), i.e. Level 1 of the three-level model.

## XVII.4 Deferred destruction

`[GPU-6]` `device.destroy(h)` invalidates the handle immediately (generation bump) and queues the physical object on the retirement list of the current frame; the runtime destroys it when that frame's fence has been observed signalled. This is RageV's `VulkanDevice::DeferDestruction` made a standard primitive. Dropping the device flushes all retirement lists after a full wait-idle; `[GPU-7]` device drop while handles are still live logs each leaked handle with its creation site in debug.

## XVII.5 Temporal history

```ember
history = gpu.History[TextureHandle].new(mut device, desc, count=2)
prev, cur = history.pair(frame)         # prev: read this frame, cur: write this frame; swapped at end_frame
```

`[GPU-8]` `History[T]` owns `count` handles, exposes `(prev, cur)` per frame, and forbids `cur` being read or `prev` being written in the same frame (checked through the access-state machine). It is registered with the device so its resources are `InFlight`-tracked like any other.

## XVII.6 Render graph (optional layer)

`std.gpu.graph` provides `Graph`, `Pass`, `graph.transient_texture(desc)`, `pass.read(h)`, `pass.write(h)`, `pass.native(fn(cmd) => ...)` and `graph.compile()` which derives first/last use per transient, aliases transients with disjoint lifetimes into shared physical allocations, orders passes by dependencies, and inserts transitions. `[GPU-9]` The graph is a library over `CommandList`; direct recording and native passes remain available inside a graph. Its implementation is Phase 7 and designed to be replaced by RageV's own `RenderGraph`/`FrameGraphBuilder` through the FFI first.

## XVII.7 Shader interface generation

```
foo.rvshader ─glslang─► foo.spv ─(SPIRV-Cross reflection, as RageV's shaderinfo already does)─► foo.reflect.json
                                                                                                    │
                                                        ember shader-bind foo.reflect.json ─► foo_shader.em
```

`[GPU-10]` The generated module contains: a `@gpu_layout(std140|std430)` struct per uniform/storage block with **comptime layout assertions**, a `ShaderInterface` struct with typed binding slots (`set`, `binding`, kind, array size), push-constant struct, and vertex-input layout constants. Binding a `ResourceSet` built from a mismatched interface is a type error, not a validation-layer message. Shader language remains GLSL (or anything that yields SPIR-V); `[GPU-11]` Ember never assumes a shader language.

## XVII.8 Kernel language (v3, reserved)

`@gpu fn` kernels compiling to SPIR-V are out of scope for v1/v2. The keyword `@gpu` is reserved; `std.gpu.kernel` is reserved as a module name.

---
# Part XVIII — Compiler Architecture

## XVIII.1 Overview

```
 .em files ─► Lexer ─► Parser ─► AST ─► Name resolution ─► Type checking/inference ─► HIR (typed, desugared)
                                                                                         │
                          ┌──────────────────────────────────────────────────────────────┘
                          ▼
   HIR ─► MIR lowering ─► Definite-init ─► Borrow check (NLL) ─► Drop elaboration
                                                                        │
                          ┌─────────────────────────────────────────────┘
                          ▼
   Monomorphisation ─► MIR optimisations (RC elision, inlining of tiny fns, SROA, const-prop, bounds-check elim,
                       exclusivity-check elision) ─► Effect analysis (under the contract profile, `[EFF-15]`)
                          │
            ┌─────────────┴─────────────┐
            ▼                           ▼
     C11 backend (v1)             LLVM backend (v2)
            │                           │
     project C compiler           LLVM opt + codegen
            └─────────────┬─────────────┘
                          ▼
                      linker (lld / link.exe / ld)  + ember_rt.lib
```

Effect analysis runs **after** monomorphisation and after the target-independent subset of MIR optimisation, because `[EFF-9]`'s `RuntimeCheck(k)` describes generated code: a check that elision removes must not appear in the effect set. It runs under the **contract profile** (`[EFF-15]`), so a contract has one verdict for a given source rather than one per build profile.

`[CMP-1]` The compiler is a Rust workspace (`emberc`). Each stage is a crate with a documented input/output type and a `--emit=<stage>` flag so intermediate representations can be dumped and snapshot-tested. `[CMP-2]` No stage after parsing may report a diagnostic without a source span.

### Crate layout

```
compiler/
  ember_span         FileId, Span, SourceMap (line/column mapping, UTF-8 aware)
  ember_diag         Diagnostic model, rendering (ariadne-style), error-code registry (Part XIX §6), JSON output
  ember_lexer        tokens, indentation algorithm, literal decoding
  ember_ast          AST types (§2), visitor, pretty-printer (used by the formatter)
  ember_parser       recursive descent + Pratt; error recovery; produces AST + parse diagnostics
  ember_resolve      module graph, scopes, DefIds, import resolution, IndexOrInstantiate disambiguation
  ember_types        Ty interner, TypeInfo table, layout computation, unification, interface obligations
  ember_typeck       bidirectional type checking of function bodies → HIR
  ember_hir          HIR types (§3)
  ember_mir          MIR types (§4), lowering from HIR, verifier
  ember_analysis     definite-init, NLL borrow checker, drop elaboration, effects, exclusivity analysis
  ember_mono         monomorphisation, instantiation cache, drop-glue synthesis
  ember_opt          MIR-level optimisations
  ember_interp       comptime MIR interpreter
  ember_ffi          libclang import (clang-sys), BIR, overlays, .embind read/write, C++ thunk generation
  ember_abi          C ABI classification for the LLVM backend; C-backend type mapping
  ember_codegen_c    MIR → C11
  ember_codegen_llvm MIR → LLVM IR (inkwell)             [v2]
  ember_build        ember.toml, lockfile, build graph, caching, CMake file-API reader, C/C++ toolchain driver
  ember_driver       `ember` CLI
tools/
  ember_fmt, ember_lint, ember_lsp (v2), ember_shader_bind
runtime/
  ember_rt/          C11 runtime (§9)
std/                 standard library in Ember
tests/               conformance suite (Part XIX §5)
```

## XVIII.2 AST

The AST is a faithful, span-carrying tree. Node kinds (Rust enum names given; fields abbreviated):

```
Item      = Fn(FnDecl) | Struct(StructDecl) | Class(ClassDecl) | Enum(EnumDecl) | Interface(IfaceDecl)
          | Extend(ExtendDecl) | Const | Static | TypeAlias | ExternBlock | Comptime(Block) | Import(ImportDecl)
FnDecl    { attrs, vis, is_unsafe, dispatch: None|Virtual|Override, name, generics, params: Vec<Param>, ret: Option<TypeExpr>,
            where_: Vec<Bound>, body: Option<Block>, doc }
Param     { mode: Borrow|Mut|Owned, pat: Ident|SelfKw, ty: TypeExpr, default: Option<Expr> }
TypeExpr  = Path{segments, generic_args} | Ref{mutable, inner} | Ptr{mutable, inner} | Tuple(Vec) | Fn{abi, params, ret}
          | Dyn(bounds) | Array{elem, len: Expr} | SelfTy | Void | Never | Infer
Stmt      = Decl{pat, ty, init} | Assign{targets, op, value} | Expr(Expr) | Return(Option<Expr>) | Break(label) | Continue(label)
          | Pass | If{...} | While{...} | For{pat, iter, body, else_} | Match{scrutinee, arms} | With{items, body}
          | Defer(Block) | Unsafe(Block) | Comptime(Block) | Labeled{label, stmt}
Expr      = Lit | Path | Field{base, name} | TupleField{base, idx} | Index{base, args} | IndexOrInstantiate{base, args}
          | Call{callee, args: Vec<Arg>} | MethodCall{recv, name, generic_args, args} | Unary | Binary | Logical
          | Ternary | Range{lo, hi, inclusive} | Cast{expr, ty} | TryOp(expr) | OptChain{base, name}
          | Lambda{owned, params, ret, body: Block|Expr} | MatchExpr | Tuple | ArrayLit | ArrayRepeat | FString{parts}
          | RefOf{mutable, place} | SelfExpr | Paren
Pattern   = Wild | Lit | Range | Bind{name, by_ref, mutable, sub: Option} | Path(path) | TupleStruct{path, fields}
          | Struct{path, named_fields, rest} | Tuple | Slice | Or(Vec)
```

`[AST-1]` Every node has `span: Span` and an `id: NodeId` (dense, per file) used by side tables (types, resolutions). `[AST-2]` The parser recovers at statement boundaries (skip to next `NEWLINE` at the current indentation) and at item boundaries; it never produces fewer than one diagnostic for a malformed region and never a cascade of more than 3 for one region (tested by the `parser/recovery` suite).

## XVIII.3 HIR

HIR is the AST after name resolution, type checking and desugaring. Differences from AST:

* All names are `DefId`s or `LocalId`s; paths are resolved.
* Every expression carries its `Ty`.
* Desugared: `for` → `while` + iterator calls (`[CTL-1]`); operators → interface method calls (scalars keep intrinsic ops); `?` → `match`; `?.` → `match`; f-strings → `Formatter` calls; `with` → block + explicit drops; augmented assignment → method call or binary op; `elif` → nested `if`; ternary → `if`; named/default arguments → positional with default expressions inserted; auto-ref/deref adjustments made explicit (`Adjust::Borrow`, `Adjust::Deref`, `Adjust::Coerce(widen)`); method calls resolved to `Callee::Static(DefId, generic_args)` or `Callee::Virtual(slot)` or `Callee::Dyn(iface, slot)` or `Callee::Closure`.
* Patterns are compiled to a **decision tree** (Maranget's algorithm) with exhaustiveness/redundancy results attached.
* Closures are lifted to synthetic struct types with a capture list `{local, mode: ByRef|ByMutRef|ByValue}`.

`[HIR-1]` HIR is the input to the comptime interpreter's MIR lowering and to the formatter's semantic lints. `[HIR-2]` `--emit=hir` prints a stable textual form used in snapshot tests.

## XVIII.4 MIR

MIR is a control-flow graph of basic blocks over **places** and **operands**, in the style of Rust MIR, with explicit borrows, moves, drops and RC operations.

### 4.1 Grammar

```
Body      := { locals: Vec<LocalDecl{ty, name, kind: Arg|Temp|User|Ret}>, blocks: Vec<BasicBlock>, arg_count }
BasicBlock:= { stmts: Vec<Stmt>, terminator: Term }
Place     := Local(LocalId) . Proj*           Proj := Field(i) | Index(LocalId) | ConstIndex(n) | Deref | Downcast(variant) | Column(field)   (Column: SoA)
Operand   := Copy(Place) | Move(Place) | Const(Const)
Rvalue    := Use(Operand) | Ref{mut, Place} | RawPtr{mut, Place} | BinaryOp(op, Operand, Operand) | CheckedBinaryOp(..)
           | UnaryOp | Cast(kind, Operand, Ty) | Aggregate(kind, Vec<Operand>)   kind = Tuple|Struct(def)|Enum(def, variant)|Array|Closure(def)
           | Len(Place) | Discriminant(Place) | NullaryOp(SizeOf|AlignOf, Ty) | ShallowInitBox
Stmt      := Assign(Place, Rvalue) | SetDiscriminant(Place, variant) | StorageLive(Local) | StorageDead(Local)
           | Retain(Operand) | Release(Place) | BeginAccess{place, kind: Read|Write, token: Local} | EndAccess(token)
           | FakeRead(Place) (* for borrowck of match scrutinees *) | Nop
Term      := Goto(bb) | SwitchInt{discr: Operand, targets: [(u128, bb)], otherwise: bb} | Return | Unreachable
           | Call{func: Operand, args: Vec<Operand>, dest: Place, next: bb, unwind: Option<bb>}
           | Drop{place, next: bb, unwind}      (* replaced by calls to drop glue during elaboration; kept in MIR for borrowck *)
           | Assert{cond: Operand, expected: bool, msg: AssertKind, next: bb}   (* bounds, overflow, div-by-zero, exclusivity *)
           | Panic{msg}
```

### 4.2 Invariants checked by the MIR verifier (`[MIR-*]`)

* `[MIR-1]` Every `Place` is well-typed; `Deref` is applied only to `ref`, `ref mut`, `Box`, raw pointers, or class handles (`Deref` of a handle yields the object type; field projection of an object requires the header offset added at codegen).
* `[MIR-2]` `Move(p)` appears at most once per path for `p` and no `Copy(p)` of a non-`Copy` type exists.
* `[MIR-3]` `Ref{mut}` of a place inside a class object is always bracketed by `BeginAccess/EndAccess` unless annotated `elided` by the exclusivity analysis.
* `[MIR-4]` Every block ends with exactly one terminator; every `Call` with an unwind edge is inside a function with `unwind` policy (v2).
* `[MIR-5]` `Retain`/`Release` appear only for class handles and `Shared`; MIR lowering inserts them at every handle copy and drop; `ember_opt` may remove pairs per `[RC-2]`/`[RC-3]`.

### 4.3 TypeInfo table

`ember_types` maintains, per interned `Ty`: `size`, `align`, `layout` (fields with offsets; enum tag placement and niche), `is_copy`, `needs_drop`, `is_view` (+ region parameter slot), `is_send`, `is_sync`, `is_zeroable`, `has_niche(Ty)`, `drop_glue: Option<DefId>`, `vtable(iface)`, `ffi_safe: bool`. Layout of `@layout(c)` structs is computed by the same algorithm the C ABI uses (natural alignment, trailing padding), and is cross-checked against libclang for imported types (`[FFI-5]`).

### 4.4 Type checking algorithm (`ember_typeck`)

1. **Collect** item signatures for the whole package (types, function signatures, interface impls, associated types) into `ember_types`. Cycles in struct definitions by value are `E2200 infinite size`.
2. **Per function**: allocate inference variables for each local declared without a type and for each generic argument at call sites; walk the HIR in **checking mode** where an expected type exists (`check(expr, ty)`) and **synthesis mode** otherwise (`synth(expr) -> ty`); unify with union–find; record coercions at coercion sites (`[TYP-5]`, `Array→Span`, `String→str`, handle upcast, `!`→any, closure→fn type); collect obligations `Ty: Interface`.
3. **Solve obligations** by searching impls (inherent + interface impls in scope, then blanket `extend[T: B]` impls) with unification; ambiguity is `E2062`; unsatisfied is `E2040`.
4. **Method resolution** per `[TYP-24]` with auto-ref/deref adjustments recorded.
5. **Default literals** (`[LEX-16/17]`), then **report** any unresolved variables (`E2060`).
6. **Pattern compilation** and exhaustiveness.
7. Emit HIR.

### 4.5 MIR lowering

Standard: expressions lowered to temporaries; places preserved; short-circuit ops to branches; `match` decision trees to `SwitchInt`; `with`/`defer` to explicit block structure; every local gets `StorageLive`/`StorageDead`; every non-`Copy` local gets a `Drop` terminator at scope exit on every path (including the `return` path); temporaries get drops at statement end; class-handle copies get `Retain`; class-handle drops get `Release`; class-field long-term accesses get `BeginAccess`/`EndAccess` (`[EXC-*]`); bounds checks become `Assert` before `Index` projections; overflow checks become `CheckedBinaryOp` + `Assert` when the profile/attribute asks for it.

### 4.6 Definite initialisation

Forward dataflow over MIR with a lattice per local (`Uninit | Init | Maybe`), also per field path for structs (partial moves/inits) and for `self` fields inside `init` methods (`[CLS-2]`). Reading `Uninit`/`Maybe` is `E3050`/`E2100`; `Maybe` at a drop point introduces a **drop flag**.

### 4.7 Borrow checking (NLL)

The algorithm is Rust's NLL (RFC 2094) restricted by the v1 lifetime rules:

1. **Regions.** Every reference/view-typed local, temporary and projection gets a region variable `'r`. A function's signature regions are per `[LT-1]`: `'self`, one `'p_i` per view-typed parameter, `'ret` = the elided/`@borrows` choice, plus `'static`. Struct view fields use the struct's single region.
2. **Constraints** are generated by walking MIR: assignment `a = b` of reference types yields `'b: 'a` (b outlives a, i.e. `points('a) ⊆ points('b)`); calls instantiate the callee's signature regions with fresh variables and add its constraints; `Ref{place}` creates a **loan** `L = (place, mut?, region)`; reborrows add constraints from the base reference's region.
3. **Liveness.** Compute the set of CFG points at which each local is live (used later on some path). A region `'r` includes every point at which any local with a type mentioning `'r` is live, and `[LT-*]` closure under constraints (fixpoint).
4. **Loan scope.** Loan `L` is **in scope** at point `P` iff `P ∈ points(region(L))` and `L` has not been **killed** (its base local was overwritten or went out of scope before `P` on that path).
5. **Access check.** At each statement, for each place `P'` accessed with kind `K ∈ {Read, Write, Move, ShallowWrite, Drop, BorrowShared, BorrowMut}`, for each loan `L` in scope with place `P` such that `P` and `P'` **overlap** (one is a prefix of the other, with `Index` projections assumed overlapping unless constant-disjoint, `Column` and `Field` projections disjoint when names differ, `Deref` of a class handle assumed overlapping with any other deref of the same class type — handled by exclusivity instead), report an error if `K` conflicts with `L.mut` per `[BRW-1]`. Two-phase borrows: a `Ref{mut}` marked `two_phase` is a shared loan until its **activation** point (the call).
6. **Region errors.** A loan whose region extends beyond the borrowed place's storage (`StorageDead`/`Drop` of a local while a loan on it is in scope) is `E3060 borrowed value does not live long enough`; a return of a reference whose region is not a subset of `'ret`'s allowed region is `E3062 returned reference does not derive from a parameter`.
7. **Diagnostics** name (a) the borrow site, (b) the conflicting access, (c) the later use that keeps the borrow alive ("borrow later used here"), and (d) a fix suggestion chosen from: shorten with a block, clone, use `split_at_mut`/`columns_mut`, use an index loop, use `Weak`.

The implementation is expected to be ~4–6k lines; the Rust compiler's `rustc_borrowck` is the reference for edge cases (drop-check, closures, two-phase, `match` fake reads). Polonius-style location-sensitive reasoning is not required for v1.

### 4.8 Exclusivity analysis

For class-object accesses, a lightweight pass over MIR: for each `BeginAccess(place=h.deref.f…)`, if all other `BeginAccess` on the *same handle local* (same `LocalId`, not reassigned in between) are statically ordered so that their intervals do not overlap conflictingly, mark it `elided`; overlapping accesses through the *same* local are `E3080` (compile-time, like a borrow error). Accesses through distinct locals keep the runtime check.

### 4.9 Drop elaboration

Replace `Drop{place}` terminators with: nothing (if `!needs_drop`), a call to the type's drop glue (a synthesised function that calls user `drop` then drops fields), a `Release` (handles/`Shared`), or a conditional on the drop flag. Partial moves produce per-field drops. This pass makes MIR ready for optimisation and codegen.

### 4.10 Effects

Per Part X: compute a bottom-up fixpoint over the call graph SCCs of the monomorphised program; store the effect set on each function instance; check contracts; produce chains for diagnostics. Before monomorphisation, generic functions are checked once with their bounds' declared effects.

### 4.11 Monomorphisation

Collect instantiation roots (`main`, `@export`s, `@test`s, statics); walk MIR bodies substituting generic arguments; instantiate on demand with a `(DefId, substs)` cache; synthesise drop glue, vtables (`dyn` and class), and closure bodies. `[MONO-1]` Instantiations are named deterministically (§8) so that separate compilation units dedupe at link time.

### 4.12 MIR optimisations (v1 set)

`RC pair elision` (`[RC-2]`), `SROA` (scalar replacement of `Copy` struct temporaries), `const propagation`, `copy propagation`, `dead-store/dead-code`, `bounds-check elimination` (range analysis for `for i in 0..len(a)` patterns — guaranteed by `[CTL-3a]`-style tests), `inline` of functions ≤ 8 MIR statements or marked `@inline`, `drop-flag elimination`, `tail-temporary merging`. All are optional for correctness; the C compiler/LLVM does the heavy lifting.

## XVIII.5 Comptime interpreter (`ember_interp`)

Executes MIR directly over an interpreter heap with typed allocations (each allocation knows its `Ty` and layout). Supports every MIR construct except `Call` into `extern` functions and raw-pointer deref outside interpreter allocations (`E6010`). Provides intrinsics: `size_of`, `align_of`, `offset_of`, `reflect`, `read_file`, `env`, `target()`. Results are converted back to `Const` values (including aggregate constants and byte strings) for embedding as statics. Step and memory limits per `[CT-3]`.

## XVIII.6 C backend (`ember_codegen_c`)

Emits one `.c` file per Ember module plus `ember_types.h` (all struct/enum/vtable definitions, topologically sorted) and `ember_decls.h` (prototypes).

**Type mapping** (`[CG-C-*]`):

| Ember | C |
|---|---|
| scalars | `int8_t … uint64_t`, `__int128`/`_BitInt(128)` (fallback struct on MSVC: `ember_i128` with helper ops), `float`, `double`, `_Float16`/`uint16_t` bit-pattern, `bool`, `uint32_t` (char), `size_t`/`ptrdiff_t` |
| `void` | `void` for returns; `ember_unit` (empty struct) as a value |
| `!` | `void` + `__builtin_unreachable()`/`__assume(0)` |
| struct/tuple/closure env | `struct em_<mangled> { ... }` with `_Alignas`; `@packed` → `#pragma pack(push,1)`/`__attribute__((packed))` |
| enum (unit) | `typedef <repr> em_<name>;` + `enum` constants |
| enum (payload) | `struct { tag; union { struct variant0; ... } u; }` (niche-optimised forms special-cased: `Option[ptr-like]` → the pointer) |
| `ref T` / `ref mut T` | `const T*` / `T*`; with `restrict` on locals proven disjoint (`[SIMD-3]`) |
| `Span`/`MutSpan` | `struct { const T* ptr; size_t len; }` / non-const |
| class handle | `struct em_obj_<Class>*` (object struct begins with `ember_obj_header`) |
| `Box[T]` | `T*` |
| `dyn I` ref | `struct { void* data; const em_vt_<I>* vt; }` |
| `[T; N]` | `struct { T a[N]; }` (so it is a value) |
| `fn` / `extern fn` | function pointer typedefs |
| generic instance | mangled distinct C type/function |

**Code shapes**: each MIR basic block is a C label; terminators become `goto`/`switch`/`return`/calls; `Assert` becomes `if (unlikely(!cond)) ember_panic_<kind>(file, line, ...)`; `Retain/Release` become inline functions from `ember_rt.h`; `BeginAccess/EndAccess` likewise; checked arithmetic uses `__builtin_*_overflow` on Clang/GCC and `ember_ck_*` helpers on MSVC; `@inline` → `static inline __attribute__((always_inline))`/`__forceinline`; `@cold` → `__attribute__((cold))`/`__declspec(noinline)`; SIMD via `ember_simd.h`.

**Debug info**: `#line` directives mapping every emitted statement to the Ember source; locals keep their Ember names where legal. `[CG-C-1]` The emitted C MUST compile warning-free under `-std=c11 -Wall -Wextra` (Clang/GCC) and `/W3` (MSVC), be free of UB by construction (no signed-overflow arithmetic without checks: wrapping ops use unsigned arithmetic and cast back), and not depend on compiler extensions except through `ember_rt.h` macros that have portable fallbacks.

`[CG-C-2]` The generated C is deterministic for identical input (stable ordering, no pointer-based hashing), so the build cache and `diff`-based review work.
* `[CG-C-4]` **Aliasing facts in emitted C.** `restrict` in C qualifies a pointer *object*; the pointer inside a `Span`/`MutSpan` is not such an object at the point of use. For every loop body and every `@simd` region, the C backend MUST hoist the base pointer of each view accessed in the region **whose base and length are loop-invariant across the region** into a local of type `T* restrict` / `const T* restrict`, and its length into a `size_t` local, before the region, and index those locals in the body. A view reassigned inside the body is not hoisted and receives no annotation. Two such locals MUST be declared `restrict` together only where `[SIMD-3]` establishes them disjoint.
* `[CG-C-5]` Every `ember_panic_*` declaration MUST carry `_Noreturn` and a cold marker (`__attribute__((cold))` on Clang/GCC; on MSVC, `__declspec(noreturn)` with the call placed in a basic block outside the loop body), and the C backend MUST emit the panic call in its own block reached by a forward branch, so the host compiler can sink it away from the fast path.
* `[CG-C-6]` For every loop in vectorisable form the C backend MUST emit the host compiler's vectorisation pragma immediately before it: `#pragma clang loop vectorize(enable)`, `#pragma GCC ivdep`, or `#pragma loop(ivdep)` (MSVC). These pragmas assert the absence of a loop-carried dependence, which is why `[SIMD-5]`'s single-constant-offset clause is a precondition rather than an optimisation.
* `[CG-C-3]` **Cross-translation-unit inlining.** Because `[BLD-1]` emits one translation unit per module, a call to a function defined in another module is opaque to the host C compiler unless the callee's definition is visible in the caller's translation unit. Extending `[BLD-1]`'s existing COMDAT-style mechanism from monomorphised instantiations to non-generic definitions, the C backend MUST emit, per package, an **inline header** `target/<profile>/c/<package>_inline.h`, included by every emitted `.c` file of that package and of every package that depends on it, holding a `static inline` definition of every function that is: (a) annotated `@inline`; (b) an impl of an operator interface (`Add`, `Sub`, `Mul`, `Div`, `Neg`, `Index`, `IndexMut`, `Eq`, `Ord`, `*Assign`, …) on a type whose `size_of` ≤ 64 bytes; (c) `len`, `is_empty`, `as_span`, `as_mut_span`, `iter`, `iter_mut`, `next`, a field accessor, or a `Deref`/`Iterator` method of a `std` view or container type; or (d) any other function whose MIR body after §4.12 is ≤ 40 statements and whose effect set does not contain `FFI`. A function emitted this way MUST NOT also be emitted with external linkage in its defining module unless it is `@export`ed or its address is taken, so `[MONO-1]`'s link-time dedup is unaffected.
* `[CG-C-3a]` `@inline` is **binding on the C backend**, not a hint: such a function MUST be emitted per `[CG-C-3]` and MUST carry `__forceinline` (MSVC) or `__attribute__((always_inline))` (Clang/GCC). Part III §7's table and the glossary entry for "Contract" are amended accordingly, resolving the existing contradiction with the `[CG-C-*]` code shapes. `@noinline` remains a hint.
* `[CG-C-3b]` A package MUST publish the MIR bodies of every function selected by `[CG-C-3]` in its build artefact, and `[BLD-2]`'s interface hash MUST cover them — `[BLD-2]` already names "inline bodies"; this rule fixes which bodies those are. A change to such a body invalidates dependent modules' codegen.
* `[CG-C-8]` **Step fidelity.** A `#line` directive MUST precede every emitted statement and MUST name the span of the *source construct that produced it*, never the declaration span of a place it mentions. All C statements lowered from one Ember statement MUST carry the same `#line`, so "step over" advances exactly one Ember statement. Desugared constructs (`for`, `?`, `?.`, `with`, f-strings, operator calls — XVIII §3) MUST attribute to the source syntax, not to the desugaring. **This requires MIR `Stmt` and `Term` to carry a source span**, and the MIR verifier checks that every statement has one.
* `[CG-C-7]` **Names.** Every MIR local carrying a user name MUST be emitted with that name as its C identifier, transliterated per `[MNG-3]`, suffixed `_<n>` only on collision with a C keyword, a reserved identifier, or another local in the same C scope. Parameters keep their Ember names; compiler temporaries keep `_<index>`. A profile MAY set `debug_names = false`; no default profile does. Deterministic naming keeps `[CG-C-2]`'s diff-based review intact.
* `[CG-C-9]` **Debugger visualisers.** `ember build` MUST emit, beside the binary, `target/<profile>/<package>.natvis` (passed with `/NATVIS:` on MSVC) and `<package>-gdb.py` / `<package>-lldb.py`, generated from the same `TypeInfo` table (§4.3) the backend already walks. They MUST render at minimum: `Option[T]` as `None`/`Some(v)` including every niche form; `Result[T,E]`; each payload enum as `Variant(fields)`; `String`/`str` as text including the SSO form; `Array`/`Span`/ `MutSpan` as `len` elements; `Box`, `Shared`, `Weak` as their pointee plus counts; a class handle as `Class { fields }` with the header hidden; `Handle[Tag]` as `index:generation`; `SoA[T]` as reconstructed `T` values.
* `[CG-C-10]` **Stacks.** `[RT-4]`'s panic output and `ember_backtrace_print` MUST print Ember function paths and Ember `file:line:col`, demangled from `[MNG-1]`'s scheme, never raw C symbols. The toolchain ships `ember demangle` (a stdin/stdout filter) so MSVC and GDB stacks can be read.

## XVIII.7 LLVM backend (v2)

Maps MIR to LLVM IR through `inkwell`: the same type mapping; `noalias`/`readonly`/`dereferenceable(N)`/`nonnull` attributes from the borrow checker's facts; `!nontemporal`, `!alias.scope` for `@simd` loops; DWARF/CodeView debug info with Ember type names; PGO via LLVM instrumentation; ThinLTO. It becomes the default when it passes the full conformance suite plus the performance suite (Part XX §4).

## XVIII.8 Name mangling

```
em_<pkg>_<module path with '_'>_<item>[__g<hash of generic args>][__v<vtable>]      e.g. em_std_math_Vec3_length, em_game_ecs_integrate__g3f2a1c
```

`[MNG-1]` Hash = first 12 hex digits of BLAKE3 of the canonical type string of the generic arguments. `[MNG-2]` `@export("name")` overrides the symbol entirely. `[MNG-3]` Identifiers are transliterated to ASCII (`_uXXXX_` for non-ASCII). `[MNG-4]` Class object structs are `em_obj_<mangled class>`; vtables `em_vt_<mangled>`; type infos `em_ti_<mangled>`.
* `[MNG-5]` The mangled prefix `em_` of `[MNG-1]`/`[MNG-4]` is derived from `EMBER_SYMBOL_PREFIX` and constructed in exactly one function.

## XVIII.9 Runtime ABI (`ember_rt`, C11)

Header `ember_rt.h`, ABI version macro `EMBER_RT_ABI = 1`. Everything below is `[RT-*]` normative.

```c
/* memory */
void* ember_alloc(size_t size, size_t align);            /* never returns NULL: panics on OOM */
void* ember_realloc(void* p, size_t old_size, size_t new_size, size_t align);
void  ember_free(void* p, size_t size, size_t align);
void* ember_try_alloc(size_t size, size_t align);        /* NULL on failure */

/* objects */
typedef struct ember_obj_header { uint32_t strong; uint32_t weak; uint32_t access; uint32_t flags; const ember_type_info* ti; } ember_obj_header;
static inline void ember_retain(ember_obj_header* o);               /* non-atomic or atomic per ti->flags & EMBER_TI_SYNC */
static inline void ember_release(ember_obj_header* o);              /* calls ember_rt_deinit when strong hits 0 */
void  ember_rt_deinit(ember_obj_header* o);                         /* runs ti->drop chain, drops fields (ti->drop_fields), then weak-release */
static inline void ember_weak_retain/release(...);
static inline ember_obj_header* ember_weak_upgrade(ember_obj_header* o);   /* NULL if strong == 0 */
static inline void ember_access_begin_read(ember_obj_header*, const char* what, ember_loc);  /* panics on conflict */
static inline void ember_access_begin_write(...);
static inline void ember_access_end_read/write(...);
void* ember_downcast(ember_obj_header* o, const ember_type_info* target);   /* NULL if not a subclass */

/* panics & diagnostics */
_Noreturn void ember_panic(const char* msg, size_t len, ember_loc loc);
_Noreturn void ember_panic_bounds(size_t index, size_t len, ember_loc);
_Noreturn void ember_panic_overflow(const char* op, ember_loc);
_Noreturn void ember_panic_div_zero(ember_loc);
_Noreturn void ember_panic_exclusivity(const char* what, ember_loc);
_Noreturn void ember_panic_unwrap(const char* what, ember_loc);
void ember_backtrace_print(void);

/* threads */
void ember_rt_thread_attach(void);   void ember_rt_thread_detach(void);
ember_thread* ember_thread_spawn(void (*f)(void*), void* arg, size_t stack_size);
int ember_thread_join(ember_thread*);
/* mutex, rwlock, condvar, atomics wrappers (C11 atomics / Win32 SRWLOCK), tls key API */

/* arenas, jobs (jobs in v1.1), time, env, fs, io: thin wrappers over the OS with UTF-8 paths on Windows */

/* runtime lifecycle & embedding */
typedef struct ember_rt_config { void*(*alloc)(size_t,size_t); void(*free)(void*,size_t,size_t); void(*log)(int,const char*,size_t);
                                 void(*on_panic)(const char*,size_t); uint32_t flags; } ember_rt_config;
ember_rt_config ember_rt_config_default(void);
int  ember_rt_init(const ember_rt_config*);   void ember_rt_shutdown(void);
uint32_t ember_rt_abi_version(void);

/* debug facilities (compiled out in shipping) */
void ember_debug_leak_report(FILE*);          /* live objects + cycles (intrusive live list enabled by EMBER_DEBUG_OBJECTS) */
void ember_debug_alloc_stats(ember_alloc_stats*);
```

* `[RT-1]` `ember_rt` has no dependencies beyond libc and the OS; `mimalloc` is vendored and used as the default allocator unless `cfg.alloc` is set.
* `[RT-2]` No global constructors; `ember_rt_init` is explicit (the generated `main` calls it) and idempotent.
* `[RT-3]` `ember_type_info` layout: `{ uint32_t size, align; uint32_t flags; const char* name; const ember_type_info* base; void (*drop)(void*); void (*drop_fields)(void*); const ember_vtable* vtable; const ember_itable_entry* itables; uint32_t itable_count; const ember_field_desc* fields; uint32_t field_count; }` — the reflection fields are present only for `@reflect` types.
* `[RT-4]` Panics print `panic at <file>:<line>:<col>: <message>` followed by a backtrace in debug/release, then call `cfg.on_panic` (if set) and `abort()`.
* `[RT-5]` Every runtime symbol, macro and header name is **generated** from a single build constant `EMBER_SYMBOL_PREFIX` (default `ember`), defined in exactly one place in `ember_rt` and one in the compiler. `ember_rt.h` is a **generation output** carrying literal identifiers, not a header of macro concatenations — it is the interface document C embedders read, and it must stay readable; `ember_rt.c` defines exactly those identifiers and carries them likewise. **Those two files aside**, no file in the compiler, runtime, CMake module, examples or test corpus may hard-code the symbol prefix, the CLI name, the manifest file name, or the source and binding-cache extensions; each is read from a single `branding` module. The exemption is exactly the runtime's own header and implementation, because a symbol has to be spelled where it is declared and defined, and nowhere else. `tools/check_branding.py` fails CI on any hard-coded occurrence.

## XVIII.10 Compiler correctness strategy

* Snapshot tests per stage (`--emit=tokens|ast|hir|mir|mir-opt|c`) with `insta`.
* The MIR verifier runs after every pass in debug builds of the compiler.
* Differential testing: every `run-pass` test is executed through both backends (once LLVM exists) and through the comptime interpreter where applicable; outputs must match.
* Fuzzing: `cargo-fuzz` targets for the lexer, parser, type checker (well-typed program generator), and borrow checker (random mutation of accepted programs must either still pass or produce an error with a span).
* FFI layout tests: a generated C program asserts `sizeof`/`offsetof` for every type crossing the boundary and is compiled with each supported compiler in CI.
* Performance regression suite (Part XX §4) gates releases.

---
# Part XIX — Toolchain

## XIX.1 CLI

```
ember new <name> [--lib | --bin | --cdylib]        scaffold a package
ember build [--profile debug|release|shipping] [--target <triple>] [--backend c|llvm] [--emit tokens|ast|hir|mir|c|obj]
            [--emit-header] [--emit-optimization-report] [--explain-performance] [-Dwarnings]
ember run [args...]        build + run the binary
ember test [filter] [--doc]      run @test functions (and doc tests)
ember bench [filter]
ember check                type-check + borrow-check without codegen
ember fmt [--check]
ember lint
ember inspect <path.to.item>       effects, layout, allocation sites, inlining, ABI
ember explain <EXXXX>              print the error's reference page
ember explain --borrow <file>:<line>   why a borrow at that line is still live (region as line ranges)
ember inspect --safety <path> [--elided-only] [--json]   runtime safety checks emitted and elided
ember bind <header> [--emit-embind | --explain <symbol> | --report | --emit-cpp]
ember shader-bind <reflect.json>
ember doc
ember clean
ember toolchain {list|install|default}
```

`[CLI-1]` Every command supports `--json` for machine-readable output (diagnostics, inspect, report) and exits non-zero on error. `[CLI-2]` `ember build --emit=c --out-dir <dir>` writes the C sources without invoking a C compiler (used by the CMake integration).

* `[CLI-4]` `ember run <file.em>` and `ember build <file.em>` MUST accept a single source file with no `ember.toml`, synthesising a package named after the file (`kind = "bin"`, entry = the file, default profile, no dependencies beyond `std`). **A first program MUST NOT require a manifest.**

## XIX.2 Manifest (`ember.toml`)

```toml
[package]
name = "ragev_scripts"
version = "0.1.0"
language = "0.2"
kind = "cdylib"                     # bin | lib | staticlib | cdylib
edition_lints = "strict"           # "strict" turns W-level into errors listed in [lints]

[dependencies]
std = { version = "0.2" }           # implicit; may pin
ragev_api = { path = "../ragev_api" }
some_lib = { git = "https://…", rev = "…" }
math_ext = "1.2"                    # registry (v2)

[build]
entry = "src/lib.em"
backend = "c"                       # c | llvm
c_compiler = "auto"                 # auto: msvc on Windows if cl.exe found, else clang
target = "native"

[profiles.debug]
opt = 0
overflow = "panic"
bounds_checks = true
exclusivity = "checked"
debug_objects = true                # live object list, leak/cycle report
sanitizers = ["address", "undefined"]   # when the C compiler supports them

[profiles.release]
opt = 2
overflow = "wrap"
bounds_checks = true
exclusivity = "checked"
lto = "thin"

[profiles.shipping]
inherits = "release"
opt = 3
exclusivity = "unchecked"           # UB on violation; only allowed here
strip = true
panic = "abort"

[cpp.ragev]                          # a C++ project Ember interoperates with
compiler = "msvc"                    # msvc | clang-cl | clang | gcc
standard = "c++20"
cmake = { build_dir = "../RageV/build", target = "RageV" }   # flags read from the CMake File API / compile_commands.json
defines = ["RV_PLATFORM_WINDOWS"]
include_paths = ["../RageV/RageV/src", "../RageV/RageV/vendor/glm"]

[comptime]
max_steps = 100000000
max_heap_mb = 256

[lints]
unused = "warn"
potential_cycle = "warn"
large_copy = { level = "warn", threshold = 256 }
```

`[MAN-1]` Unknown keys are errors. `[MAN-2]` `ember.lock` records the resolved dependency graph with content hashes; `ember build --locked` fails if the lock would change.
* `[MAN-3]` Every key in `[lints]` MUST name a lint the compiler defines (`E9010` otherwise). XIX §2's `unused` key is `L1001`.
* `[MAN-4]` The manifest file name, the source extension and the `.embind` extension are constants of the same `branding` module; the test harness and the build system discover them rather than spelling them.

* `[VER-1]` Three version numbers exist and are independent: the **language version** (`language` in `ember.toml`, `#! language`), the **compiler version**, and the **runtime ABI version** (`EMBER_RT_ABI`). Every release states all three.
* `[VER-4]` **The runtime ABI is stable within a major version.** `[OBJ-1]` is amended accordingly: the object header layout, `ember_type_info`, and every entry point in `ember_rt.h` MUST NOT change within a major version. Any change bumps `EMBER_RT_ABI` and is a major release. `ember_rt_init` MUST compare `ember_rt_abi_version()` against the value its caller was compiled with and fail initialisation with a diagnostic naming both versions rather than proceeding. Fields MAY be appended to `ember_rt_config` only at the end, guarded by a leading size field. Debug-only extensions behind a profile key (`[FFI-33a]`) are not part of the frozen layout.
* `[VER-5]` Packages use semantic versioning. A `[dependencies]` requirement is caret by default (`"1.2"` means `>= 1.2.0, < 2.0.0`); resolution selects the highest version satisfying all requirements; two different majors of one package MAY coexist in a graph, **their symbols distinguished by a major-version component in `[MNG-1]`'s package segment**. `ember.lock` records the result and `ember update [pkg]` recomputes it.

**The 1.0 compatibility promise** (owner decision `OQ-22`; normative from 1.0). `[VER-2]` source compatibility within a major language version, with breaking changes only in a new major that a package opts into by editing `language`, and a compiler accepting every language version of its own major series. `[VER-3]` deprecation in `1.n` via `@deprecated` and a `W`-code naming the replacement and the removing version, with removal no earlier than the next major. `[VER-7]` 1.0 means `[VER-2]` and `[VER-4]` come into force, the conformance suite passes on every supported host, `docs/errors/EXXXX.md` exists for every code, and the user guide exists.


## XIX.3 Build graph and caching

* `[BLD-1]` Unit of compilation and caching: the **module** (one `.em` file) for front-end stages; the **package** for monomorphisation and codegen (one C file per module is emitted, but instantiations are placed in the module that first requests them, with COMDAT-style `static inline`/weak linkage to dedupe).
* `[BLD-2]` Cache key of a module's front-end artefact: BLAKE3 of (source, compiler version, language version, package config, transitive **interface hashes** of imported modules — the hash of exported signatures, types, layouts, effect sets and inline bodies, not of private bodies). A change to a private function body recompiles only its module's codegen and any callers' *effect checks* if its effect set changed (`[EFF-4]`).
* `[BLD-3]` The `.embind` cache key remains header hash + flags + overlay-list hash, but the **entity identities** it records are `[FFI-30]`'s C identities, so two cache entries for the same header and flags describe the same entities under different views.
* `[BLD-4]` The C compiler and linker are invoked through a Ninja file generated per build (`target/<profile>/build.ninja`) so that incremental C compilation is handled by Ninja; MSVC is driven with `/showIncludes`, Clang/GCC with `-MD`.
* `[BLD-5]` Output layout: `target/<profile>/{bin,lib,c,obj,bind,inspect}`.
* `[BLD-6]` **Link-time optimisation.** `profiles.<p>.lto` takes `"off" | "on" | "thin"`, mapped by the C backend to MSVC/clang-cl `/GL` + `/LTCG`, Clang `-flto=thin` / `-flto=full`, GCC `-flto`. Where a value is unsupported the toolchain MUST substitute the nearest supported value and record the substitution in the build record. **LTO MUST NOT be required to satisfy `[CG-C-3]` and MUST NOT be required for any correctness property** (XXII.2).
* `[BLD-7]` Front-end stages MUST run in parallel across modules by default (`-j`, default = physical cores), and MUST NOT take a global lock on the symbol interner or the type interner on the hot path.
* `[BLD-8]` **Item-granular re-checking.** Within a module, the front end MUST record, per item, the signatures it read, so that editing one function body re-runs type checking, borrow checking and effect analysis **for that item alone**. Module-level name resolution re-runs only if the edit changed an item's signature or the set of names the module declares. `[BLD-2]`'s interface hash is unchanged; this is a finer key beneath it.
* `[BLD-9]` `ember build --timings` and `ember check --timings` write `target/<profile>/timings.json` plus an HTML summary, per stage and per module.
* `[BLD-10]` **Latency budget.** `tests/perf/compile/` contains generated packages of 10k, 50k and 200k lines with a realistic mix (≈30 % generic code, 20 % classes, 10 % FFI declarations). Budgets are measured on the reference machine recorded in the suite, at the `debug` profile, for: `ember check` cold; `ember check` after one function body is edited; `ember build` after one function body is edited; `ember build` cold. **The budgets are release gates with the same status as XX §4's runtime thresholds.** The concrete figures are calibrated once the generator exists and recorded in the suite, not in this document; **the existence of the gate is normative, the numbers are not.**

## XIX.4 Profiles

The profile controls: optimisation level, overflow policy, bounds checks (never disableable in safe code — `bounds_checks = false` only affects `unsafe`-opted `get_unchecked` hints and is honoured by no v1 profile), exclusivity checks, `debug_objects`, sanitizers, LTO, `panic` policy, `strip`. `[PRF-1]` A profile MUST NOT change program semantics except: (1) overflow policy (`[TYP-8]`); (2) panic-vs-UB for dynamic **class** exclusivity under `exclusivity = "unchecked"` (`[EXC-1]`, ADR-004); and (3) presence of debug facilities. In particular no profile may change which branch of a `match`, `if`, `?` or short-circuit operator is taken, nor the value of any expression observable to safe code, nor whether a program is accepted (`[EFF-15]`).

## XIX.5 Test harness and conformance suite layout

```
tests/
  conformance/<rule-id>/*.em          one directory per normative rule ID, e.g. tests/conformance/BRW-3/two_phase_push.em
  compile-pass/*.em                   must compile, not necessarily run
  compile-fail/*.em                   must fail with the annotated errors
  run-pass/*.em                       compile, run, compare stdout/exit code
  run-fail/*.em                       compile, run, expect a panic with a given message
  ui/*.em + *.stderr                  full diagnostic text snapshots
  ffi/**                              C/C++ interop fixtures (headers + .em + expected)
  perf/**                             benchmark programs with C reference implementations and thresholds
  std/**                              standard library unit tests (@test functions)
```

Annotation format inside test files (`[TST-*]`):

```ember
#$ test: compile-fail
#$ rules: BRW-1, OWN-3
fn main():
    a = Array[i32]()
    b = a
    a.push(1)          #$ error[E3040]: use of moved value `a`
                       #$ note: value moved here @ line-1
```

Annotations begin with `#$`. To the compiler that is an ordinary line comment (`[LEX-10]`), so an annotation never affects compilation and an annotated file is a valid compilation unit. `$` is the only ASCII symbol with no other meaning in Ember — it appears in no operator, no literal form and no identifier rule — so an annotation cannot collide with source, now or after any future grammar change.

* `[TST-0]` The test harness MUST read annotations from the **raw source text**, never from the token stream, because a `compile-fail` test may be expected to fail at the lexer (`E0001` invalid UTF-8, `E0002` a tab in indentation) and its expectations must be readable when the file does not tokenise at all.
* `[TST-1]` `#$ error[EXXXX]: <message-substring>` on a line asserts an error with that code whose primary span starts on that line; `#$ warning[...]`; `#$ note` for secondary labels (`@ line±n` for relative positioning). Unexpected diagnostics fail the test; missing expected ones fail the test.
* `[TST-2]` `run-pass` files may contain `#$ stdout:` followed by `#$` lines of expected output, and `#$ exit: N`. `#$ assert-c: contains("…")` and `#$ assert-c: !contains("…")` assert against the emitted C.
* `[TST-3]` `@test` functions in `std/` and user packages run in-process in the test binary, each in isolation (panics captured with `abort` replaced by a longjmp-based harness in the test runner build only; with the unwind policy, normal catch).
* `[TST-4]` Every rule ID in this document appears in `tests/conformance/INDEX.md`, generated by `tools/rule_index.py` from the spec, which fails CI if a rule has no test directory.
* `[TST-6]` `docs/spec-source/appendix-a.em` is a conformance fixture, and Appendix A's code block is generated from it by `tools/spec_check.py --emit-appendix`, so the quick reference cannot drift away from something that parses. The fixture's directive is the current language version; `...` bodies are `pass`; the statement tail is wrapped in `fn demo():`; `match` arms use the `=>` form, since `[GRM-16]` makes a jump an expression. It is annotated `#$ test: compile-pass` once `std` supplies the types the appendix names — `Entity`, `CommandList`, `Formatter`, `SoA`, `Arena`, `Mutex` — and until then `#$ test: syntax-pass`, held to `ember check --syntax-only`, which is what `[TST-7]` runs over every block in this document.
* `[TST-7]` `tools/spec_check.py` extracts every fenced ` ```ember ` block from Parts I–XVII and Appendix A and runs `ember check --syntax-only` over each. A block that does not parse fails CI. A block MAY opt out with ` ```ember,ignore ` and MUST then carry a one-line reason on the preceding line (permitted reasons: a `std` signature sketch, foreign-language source, or a deliberate error example). The gate ships with a **recorded baseline** of `,ignore` blocks and fails only on *new* failures until RFC-010 and RFC-022 have closed the grammar gaps it exists to expose.
* `[CLI-9]` `ember check --syntax-only <file>` lexes and parses the file and reports only `E00xx` and `E01xx` diagnostics. It does not resolve names, so an example naming undeclared types still passes.
* `[CLI-10]` `ember build --report=engine` is a **reporting mode, not a compilation profile**. It MUST NOT change type checking, safety, program acceptance, generated semantics, or optimisation legality — `[PRF-1]` governs profiles and this is not one. The report summarises allocations, retain/release traffic, dynamic class accesses, surviving runtime checks with their `[EFF-11]` reason codes, arena usage, vectorisable loops and the clause that blocked each one, FFI wrapper costs, inlining decisions, and candidate class-to-struct migrations. A migration candidate is advisory only and is never applied automatically.
* `[CLI-5]` `ember bind --init <header> [--out overlays/<stem>.em]` writes a **starter overlay** containing, for every declaration the header exports: its derived contract; every `unknown` fact written as a `TODO(count)` / `TODO(nullable)` / `TODO(ownership)` marker with the original C declaration in a trailing comment; every skipped declaration as a commented `W5002` line with its reason; and a `hide` section. The emitted file MUST compile as an overlay unchanged. `--init` never overwrites an existing file; with `--merge` it adds only declarations absent from it, preserving hand-written contracts and comments.
* `[CLI-6]` `ember bind --report [--json] [--baseline <file>]` prints every declaration that did not import, with its construct and reason (`[FFI-20a]`), and every declaration still `unsafe`, with the facts still `unknown`. It exits non-zero when anything is skipped, so a project can gate CI on a header continuing to import; with `--baseline` it exits non-zero only on *new* skips, and `--write-baseline` records the current state.
* `[CLI-7]` `ember bind --check <overlay>` verifies `[FFI-12]` without building, and prints the counts of declarations that are safe, still `unsafe`, and skipped — so the state of an adoption is a number a team can watch.
* `[TST-8]` **First-draft corpus.** `tests/firstweek/` holds at least 24 programs of 30–200 lines, each the *first draft* of a task a newcomer plausibly attempts in week one — a text adventure with rooms holding items, a particle fountain, a CSV summariser, a scene graph with parent pointers, an observer/event bus, an inventory with equipment slots, a tile-map path finder, a config loader, a ring buffer of chat messages, a small hand-rolled ECS. Each is contributed by someone who has read only Part I and Appendix A, MUST be committed unmodified, and records its author's experience level.
* `[TST-9]` Each program carries `#$ firstdraft: accepted` or `#$ firstdraft: rejected(<shape>)` naming a §XIX.6.1 or §XIX.6.2 shape. **A rejection whose shape is in no catalogue is a release blocker** — it is `[DIA-7]`'s unclassified case in the only corpus not written by people who already know the answer. The companion obligation — that the mandated `help`, applied literally, produces a program that compiles — is `[PHIL-8a]` (RFC-011) and applies to the whole catalogue, not only to this corpus; the corrected program is committed beside the draft as `<name>.fixed.em`.
* `[TST-11]` **v0.5 regression obligations.** The conformance suite MUST contain, in addition to the per-rule directories `[TST-4]` requires: for `[EXC-4]`, instantaneous scalar reads, `Copy` field reads, container iteration through `let` fields, `ref`/`ref mut` projections, mutating methods through `let` containers, and aliasing mutation during iteration; for `[LT-7]`, callback-local borrows, escape rejection, internal-only region inference, and nested callback-region separation; for `[TYP-15]`/`[TYP-15a]`, rejection of arbitrary owning view containers, acceptance of `BorrowList[T]`/`ViewList[T]` only under a single inferred region, rejection where two independent regions would be required, and escape rejection through class fields, `static`s, `Box` and `Shared`; for `[FFI-30c]`, overlay composition order, explicit `override`, incompatible-definition diagnostics, foreign type identity stability, and build invalidation on overlay change; for `[EFF-17]`, rejection of every `Panic(Explicit)` source and acceptance of division by `NonZero[T]`; and for `[PRF-1]`, that one source receives the same contract verdict under `debug`, `release` and `shipping`, and that `--report=engine` does not alter that verdict.
* `[TST-10]` The harness computes the **first-draft acceptance rate** (accepted ÷ total) into `target/firstweek.json` and the release notes publish it. It is a **tracked number, not a gate** — the gate is `[TST-9]`'s blocker — and a release that lowers it MUST say why.
* `[TST-5]` `tests/debug/` runs, on every supported host in CI, a scripted debugger session (`cdb` on Windows, `lldb`/`gdb` elsewhere) against a `debug` build: set a breakpoint by Ember `file:line` and verify it binds; step three Ember statements and verify the reported line advances by one construct each; print four locals by their Ember names and match the rendered text against a snapshot. A host on which `[TST-5]` cannot run is reported as **debug-unverified** and requires a recorded waiver; it does not thereby cease to be a supported host, since debugger automation is a property of the CI image.

## XIX.6 Diagnostics

Format (human):

```
error[E3040]: use of moved value `ps`
  --> src/main.em:12:11
   |
10 |     n = take(ps)
   |              -- value moved into `take` here
11 |     print(n)
12 |     print(ps.len())
   |           ^^ value used here after move
   |
   = help: if you need `ps` afterwards, pass a clone: `take(ps.clone())`
   = note: `Array[Player]` is not Copy because it owns heap memory
```

* `[DIA-1]` Every diagnostic has: code, one primary span, zero or more secondary spans with labels, optional `help` (with a machine-applicable fix-it when possible) and `note`. `--json` emits the same structure.
* `[DIA-2]` Message style: lowercase first letter, no trailing period, name the thing (`` `ps` ``), say what is wrong and, in `help`, what to do. Never say "you"; never blame the user.
* `[DIA-3]` Borrow/ownership diagnostics MUST include the "later used here" label (`[BRW-2]` explanation) and a concrete fix drawn from the catalogue in §XIX.6.1 (`[DIA-7]`).
* `[DIA-4]` Contract diagnostics MUST print the full call chain (`[EFF-5]`).
* `[DIA-5]` FFI diagnostics name the header and the C declaration (`vkCreateBuffer in vulkan_core.h:1234`).
* `[DIA-7]` **Every ownership or borrow error MUST be classified into one of the shapes in §XIX.6.1 and MUST emit that shape's required `help` line.** An error the classifier cannot place emits the generic explanation and is recorded in `target/<profile>/unclassified-borrow-errors.log`; CI fails if the conformance suite produces any unclassified borrow error. This rule exists because an unexplained rejection is the single largest usability cost of static aliasing rules, and it is cheaper to specify the fixes than to discover them one bug report at a time.
* `[DIA-8]` `ember explain --borrow <file>:<line>` prints, for each loan live at that line: where it was created, the line range of its region, and the specific later use that extends it — in the form `borrow of `v` created at 12:9, live through 19, because `s` is used at 19:14`. This is generated from the borrow checker's own loan/region tables (Part XVIII §4.7), not reconstructed.
* `[DIA-9]` Diagnostics MUST NOT suggest `unsafe`, `Cell`, `RefCell`, `Shared`, or `clone()` as the *first* suggestion when a structural fix exists for the shape (per the catalogue's ordering). `clone()` is suggested first only for shape O1 (use after move of a `Clone` type).
* `[DIA-6a]` The error-code registry is **exhaustive and normative, in both directions**: every code named anywhere in this document MUST have a registry entry (code, kind, subsystem, title, the rule it enforces) and a `docs/errors/EXXXX.md` page, **and** every registry entry MUST cite a rule id that exists in this document. `tools/rule_index.py` MUST fail CI on either violation, in the same pass that checks `[TST-4]`.
* `[DIA-7a]` **Every code is keyed.** The table below maps every **ownership or borrow** error code in `E3000–E3499` to the shape whose `help` it MUST emit. Such a code absent from this table MUST NOT be emitted. A code in the range that is neither — `E3100`, `[UNS-1]`'s "requires an `unsafe` block", which no shape in §XIX.6.1 describes — is outside the classifier's scope per `[DIA-7]`, and an implementation MUST carry the exceptions as a named list rather than by omission. `tools/rule_index.py` fails the build if `ember_diag::codes` contains an E3xxx ownership code with no row here.  | Code | Shape | Code | Shape | |---|---|---|---| | `E3010`–`E3013` | O2 | `E3050` | O6 | | `E3014`, `E3015` | O8 | `E3060` | B7 | | `E3016` | O9 | `E3061`, `E3090`, `E3096` | A1 | | `E3020` | B2 | `E3062` | B6 | | `E3021` | B3 | `E3063` | B12 | | `E3022` | B1 | `E3064` | B13 | | `E3023` | B4 | `E3070` | O7 | | `E3024` | B5 | `E3080` | X1 | | `E3025` | B8 | `E3095` | B11 | | `E3026` | B9 | `E4030` | S1 | | `E3027` | B10 | `E3030` | O5 | | `E3040` | O1 | `E3041` | O3 | | `E3042` | O4 | | |
* `[DIA-12]` Every diagnostic in `E1000–E1499` and `E2000–E2499` MUST be classified into one of the shapes below and MUST emit that shape's required `help`. An unclassifiable error is written to `target/<profile>/unclassified-basic-errors.log`; CI fails if the conformance suite produces any. This is `[DIA-7]` applied to the errors a newcomer meets first.
* `[DIA-13]` Every shape in §6.1 and §6.2 has a rendered snapshot under `tests/ui/`, plus the `.fixed.em` companion `[PHIL-8a]` requires; `[TST-4]`'s index generator fails CI for a shape with none.
* `[DIA-14]` No diagnostic may be emitted about an expression of type `Ty::Error` or a path bound to `Def::Error` (`[IDE-3]`): the first error of a cascade is reported and the rest suppressed. `[AST-2]`'s three-per-region cascade cap applies to the whole front end, not the parser alone.
* `[DIA-15]` Suggestion candidates are computed only from data the compiler already holds (scope tables, `TypeInfo`, and a by-name index over `[BLD-2]`'s module export hashes). A suggestion requiring speculative type checking is not required.
* `[DIA-16]` **World changes disclose their cost.** When a diagnostic's `help` proposes moving a value type into the object world (`class`, `Shared[T]`) or wrapping it for dynamic checking (`RefCell[T]`), it MUST attach a `note:` naming the cost in the terms the specification already defines: ``` = note: a class instance is a heap allocation with a 24-byte header (OBJ-1) and reference counting (RC-1); long-term accesses through it are checked at runtime (EXC-1), so a function containing them cannot be @static_safe (EFF-13) ``` The note is required because the object world's costs are permanent and structural while the value-world fixes offered alongside are local; a diagnostic presenting them as equivalent misinforms.
* `[DIA-17]` **Python-form fix-its.** `x is None` / `x is not None` on a non-handle operand suggests `x.is_none()` / `x.is_some()`; `a < b < c` suggests `a < b and b < c` and notes that `b` is then evaluated twice; `xs[-1]` on a `usize`-indexed container suggests `xs.last()`; `let x = e` in statement position suggests `x = e` and notes that `let` marks an immutable **field** (`[CLS-9]`); `with e as x:` suggests `with x = e:`.

### XIX.6.1 Required diagnostic catalogue for ownership and borrow errors

Each shape below MUST be recognised by the borrow checker's diagnostic classifier and MUST produce the listed `help`. Suggestions are ordered; the first applicable one is primary, the rest appear as additional `help` lines. Each shape has a directory in `tests/ui/borrow/<shape>/` with a snapshot of the exact rendered output.

| Shape | Trigger | Required primary suggestion |
|---|---|---|
| **O1** use after move | a place is read after being moved into a call, assignment or `owned` binding | if the type is `Clone`: pass `x.clone()`; else: restructure so the last use precedes the move, or borrow instead of consuming (name the parameter whose mode is `owned`) |
| **O2** move out of a container | move from an index, `Span` element, or a field of a `Drop` type | `mem.take`/`mem.replace`/`swap` for a field; `pop`, `swap_remove`, `remove`, `drain` for a container element |
| **O3** move in a loop | a value declared outside the loop is moved inside it | move the declaration inside the loop, clone per iteration, or use `mem.take` if the value is being replaced each pass |
| **O4** partial move then whole use | a struct is used as a whole after one field moved out | reassign the moved field, or destructure the whole struct up front |
| **B1** two mutable indices | `ref mut a[i]` and `ref mut a[j]` both live | `a.split_at_mut(k)`, `chunks_mut`, `iter_mut`, or `columns_mut` for `SoA` — name the one that fits the access pattern; if the operands come from unrelated sources, `mem.assert_disjoint(a, b)` (`[DSJ-1]`) |
| **B2** mutate while iterating | a container is mutated inside `for x in container` | collect the indices or values first; use `retain`, `drain`, or `for i in 0..len` with indexed access; for the ECS, `q.commands()` |
| **B3** shared and mutable overlap | a shared borrow is live across a mutating call | shorten the shared borrow with a block, copy the value out if `Copy`, or reorder so the read happens before the mutation — cite the exact "later used here" line |
| **B4** aliased mutation of a value type | the same place needs two writers, not simultaneous within one expression | restructure to a single owner and pass `mut`; if genuinely shared, `Cell[T]` (`Copy` payload) or `RefCell[T]`; if it has identity, make it a `class` |
| **B11** disjointness not provable | two views whose ranges the compiler cannot relate are both borrowed mutably | `mem.assert_disjoint` to verify and establish it (two comparisons); the structural fix from B1 if the views share a parent |
| **B5** self-referential struct | a struct field would borrow another field of the same struct | store an index or a `Handle` instead of a reference; split into two structs; for object graphs use a `class` with `Weak` back-pointers |
| **B6** returned reference not derived from a parameter | `[LT-1]` elision cannot tie the return region to an input | return an owned value, take the destination as a `mut` parameter, or add `@borrows(param)` naming the input it points into |
| **B7** borrow outlives its source | a loan is live past the `StorageDead` of the borrowed local | move the source to an outer scope, return an owned value, or bind the temporary with `with` so it lives to the end of the block |
| **B8** method takes all of `self` | disjoint-field access (`[BRW-4]`) is defeated by a method call | inline the field access, take the two fields as separate parameters, or split the method |
| **B9** closure outlives its captures | a non-`owned` closure escapes | declare it `owned fn` (captures by move/copy/retain), or use `thread.scope()`/`jobs.scope()` so the borrow is joined before it ends |
| **B10** `mut` argument is not a mutable place | a temporary, a `Copy` of a field, or an immutable binding is passed to a `mut` parameter | bind to a local first, or name the owner (`f(obj.field)` rather than `f(obj.get_field())`) |
| **X1** static exclusivity conflict (`E3080`) | two overlapping long-term accesses through the same class-handle local | shorten the first access with a block; take the field's value out if `Copy`; split the method so the two accesses do not nest |
| **S1** `@static_safe` violated (`E4030`) | a `@static_safe` function carries `RuntimeCheck(Aliasing)` | name the exact access and why it is dynamic (`[EFF-13]`): hoist the handle to a local so `[EXC-3]` applies; switch to value types / a `Query` yielding `ref mut`; establish disjointness with `assert_disjoint`; or drop the contract if the check is `not_provable_in_principle` |
| **R1** region unexplained | any of the above where the reason a borrow is still live is more than 5 lines from the error | append: `run `ember explain --borrow <file>:<line>` to see why this borrow is still live` (`[DIA-8]`) |
| **O5** closure moves out a capture (`E3030`) | a closure body moves a captured non-`Copy` value into a call, a return, or an assignment | declare the parameter `owned f:` so the closure is `CallableOnce` (`[CLO-6]`, RFC-023); or `mem.take` the capture if the type is `Default`; `clone()` last. *If RFC-023 is declined, the required text is instead: name the type, state that it has no `Default`, and suggest `Option[T]` + `mem.take` + `expect` while naming the panic path introduced* |
| **O6** borrow of a moved or uninitialised place (`E3050`) | a `ref`/`ref mut` is taken of a place not initialised on every path | name the path on which it is uninitialised; initialise before the borrow, or restructure so the borrow follows the initialisation |
| **O7** explicit `drop` call (`E3070`) | `x.drop()` in source | `mem.drop(x)` to end the value's life early, or let it fall out of scope |
| **O8** leaked `@must_drop` value (`E3014`, `E3015`) | a scope guard moved out of its binding, forgotten, or stored where it may cycle | the closure-scoped form (`[THR-5]`) |
| **O9** `self` escapes its own drop (`E3016`) | a `drop` body publishes a handle to the object being destroyed | `mem.take` the data out; for pooling, keep the count above zero rather than resurrecting |
| **A1** arena lifetime (`E3061`, `E3090`, `E3096`) | an arena view outlives the arena; a `needs_drop` type is arena-allocated; the parent is used while a scope is live | move the `Arena` to an outer scope, or copy the value out before `reset`; for `E3090`, `alloc_nodrop` with the acknowledgement that `drop` will not run; for `E3096`, allocate from `scope`, or take the allocation before opening it |
| **B12** view stored in a place that outlives it (`E3063`) | a `ref`, `Span`, `str` or `@view struct` is stored in a class field, a non-view struct field, a `static`, a container element, a `Box`/`Shared`, or an `owned fn` capture | store an owned copy (`String` for `str`, `Array[T]` for `Span[T]`) **and name the per-element allocation cost**; or store a `u32` index / `Handle[T]` **and name the container it indexes**; for object graphs, `Weak` |
| **B13** two independent regions in one view struct (`E3064`) | a `@view struct` would need its fields' regions to differ (`[LT-2]`) | pass the two views as separate parameters rather than bundling them; or copy the shorter-lived data into an owned field |

`[DIA-10]` Shapes B1, B2, B4, B5 and B9 account for most rejections in practice; their `help` text MUST name a concrete API or construct (`split_at_mut`, `retain`, `Weak`, `owned fn`), never a category ("consider restructuring", "use interior mutability").

`[DIA-11]` For shape S1 the diagnostic MUST report the reason code of the offending check (`[EFF-11]`). When the reason is `not_provable_in_principle` or `inherent_to_mechanism`, the primary suggestion is to **remove the contract or change the data structure** — never to restructure the code, because no restructuring will help. Suggesting an impossible fix is the failure mode this rule exists to prevent.

Worked example of the required rendering for shape B2:

```
error[E3020]: cannot mutate `enemies` while it is borrowed by this loop
  --> src/game.em:41:13
   |
39 |     for e in enemies:
   |              ------- `enemies` borrowed here for the whole loop
40 |         if e.health <= 0:
41 |             enemies.remove(e.id)
   |             ^^^^^^^^^^^^^^^^^^^^ mutable borrow here
   |
   = help: collect first, then mutate: `dead = enemies.iter().filter(fn(e) => e.health <= 0).collect[Array[Id]]()`
   = help: or remove in place without a second pass: `enemies.retain(fn(e) => e.health > 0)`
   = note: a `for` loop borrows the container until it ends (see BRW-2)
```

### Error-code registry

| Range | Subsystem |
|---|---|
| E0000–E0099 | lexer / indentation / directives |
| E0100–E0499 | parser |
| E1000–E1499 | name resolution, modules, visibility (incl. `E1050`/`E1051`, `pub(read)`) |
| E2000–E2499 | types, inference, interfaces, generics, patterns |
| E3000–E3499 | ownership, moves, borrows, regions, exclusivity (static), drops |
| E4000–E4499 | effects and contracts (`@noalloc`, `@static_safe` `E4030` …), SIMD/parallel contracts |
| E5000–E5499 | FFI (import, overlays, layout mismatch, callbacks, C++) |
| E6000–E6499 | comptime |
| E7000–E7499 | concurrency (`Send`/`Sync`, parallel loops, ECS scheduling) |
| E8000–E8499 | layout attributes, GPU layout |
| E9000–E9499 | build system, manifest, toolchain |
| W-codes | same ranges, warnings |
| L-codes | lints (`ember lint`) |
| E-panic | runtime panics (documented messages, not compile-time codes) |

`[DIA-6]` `docs/errors/EXXXX.md` exists for every code with an example and its fix; `ember explain E3040` prints it.
* `[DIA-18]` A diagnostic that rejects a foreign call because its contract contains `unknown` facts MUST name each missing fact and print the exact overlay line that supplies it: ``` error[E5002]: `vkGetPhysicalDeviceQueueFamilyProperties` requires `unsafe`: its contract is incomplete note: `pQueueFamilyProperties` has no count contract and no nullability help: add to `overlays/vulkan.em`: unsafe fn vkGetPhysicalDeviceQueueFamilyProperties( physicalDevice: borrowed, pQueueFamilyPropertyCount: inout_count, pQueueFamilyProperties: span(len_of(pQueueFamilyPropertyCount), nullable)) ```

### XIX.6.2 Required diagnostic catalogue for the errors of the first hour

The shapes in §XIX.6.1 are the errors of week two. These are the errors of the first hour, and they decide whether a newcomer stays. Each MUST be recognised and MUST produce the listed `help`.

| Shape | Trigger | Required primary suggestion |
|---|---|---|
| **N1** unknown name | a path resolves to nothing | the in-scope binding, field, method or item within Damerau–Levenshtein distance ≤ 2 (or ≤ ⅓ of the name's length), named exactly; at most 3, ranked by distance then declaration proximity |
| **N2** name exists elsewhere | nothing in scope matches, but items of that name are exported by modules in the dependency graph | the import line to add, verbatim, as a machine-applicable fix-it (`from std.math import Transform`); with several candidates, up to 5 with module paths |
| **N3** unknown method | the receiver's type has no such member | the nearest member by N1's rule; if the name exists after one auto-deref or on `Self`'s base (`[TYP-24]`), name that; if it exists on an interface the type does not implement, name the interface and the `extend` block that would supply it |
| **N4** scalar type mismatch | `[TYP-4]` rejects an operator, or a coercion site is not a `[TYP-5]` widening | the exact `as` cast on the operand that loses nothing (`xs.len() as f32`); the note MUST state which side supplied the expected type |
| **N5** literal does not fit (`E2010`) | `[LEX-16]` | the suffix or annotation that makes it fit |
| **N6** arity mismatch | too few/many arguments | the callee signature rendered with parameter names and modes; for a missing defaulted argument, the named-argument form |
| **N7** mutability | a `mut` method or parameter receives a non-mutable place | the declaration to change, named and located ("the binding `p` at 12:5 is not mutable") — never a bare "cannot borrow as mutable" |
| **N8** wrong argument mode | an `owned` parameter receives a borrow or vice versa | the parameter and its mode in the callee's signature, and the one-token fix at the call site |
| **N9** missing bound (`E2040`) | `[TYP-17]` | the bound to add, spelled — **and, because `[TYP-20]` permits the implementation in only one place, the single module in which `extend T implements I:` may legally be written, or a statement that neither module is under the programmer's control and a wrapper type is required** |
| **N10** `dyn` incompatibility (`E2050`) | `[TYP-22]` | the offending method and which clause it violates (no receiver / generic / returns `Self` by value), never a bare "interface is not object safe" |
| **N11** ambiguous method (`E2070`) | two candidates | the `I.m(recv, …)` disambiguation for each candidate interface |
| **N12** indentation (`E0002`–`E0004`) | | the exact column expected and the line whose indentation established it |

## XIX.7 Formatter

`ember fmt` is deterministic and configuration-free except line width (default 100). Rules: 4-space indentation; one blank line between methods, two between items; trailing commas in multi-line argument/element lists; spaces around binary operators, none around `**` when both operands are atoms; `x: T = v` spacing; imports sorted (`std` first, then dependencies, then local) and merged; attributes one per line; the formatter preserves comments and blank-line groups (max 2 consecutive); `pass` inserted for empty blocks; long conditions broken after `and`/`or`. `[FMT-1]` `fmt(fmt(x)) == fmt(x)` and `parse(fmt(x)) ≡ parse(x)` are tested over the whole test corpus.

* `[FMT-2]` The formatter never produces a block-bodied lambda inside brackets: it emits the `=>` form when the body is a single expression, and otherwise leaves the programmer's named binding alone.
* `[FMT-3]` The formatter never emits `;` outside `[T; N]` and `[v; N]`. Since `[GRM-18]` makes `;` illegal as a statement separator there is nothing to split: one line already carries one statement.

## XIX.8 Linter (`ember lint`, L-codes)

v1 lints: `L2001 unnecessary clone` (source not used again), `L2002 large Copy` (> threshold bytes passed by value), `L3001 potential cycle` (statically visible class cycles), `L3002 borrow held longer than necessary` (a borrow whose last use is far before its scope end and blocks a later access — suggests a block), `L3010 unsafe block larger than necessary`, `L3011 RefCell guard held across a call` (a `Ref`/`RefMut` guard live across a call that could re-enter the same cell, `[CELL-7]`), `L4001 allocation in hot loop` (allocation inside a loop of a `@simd`/`@parallel` body or inside functions named in `[lints.hot_paths]`), `L4002 dynamic dispatch on final type` (redundant `dyn`), `L5001 unsafe extern without contract`, `L5002 FFI copy` (conversion at the boundary copying > threshold bytes), `L7001 lock held across call that may block`.

* `[LNT-1]` **`L1001 unused binding`.** A local introduced by `x = expr` or `x: T = expr` and never read on any path is reported at `warn` by default. Names beginning `_` are exempt. Liveness comes from the borrow checker's existing analysis (XVIII §4.7 step 3).
* `[LNT-2]` **`L1002 assignment declares a new binding`.** When `L1001` fires for a binding whose name is within Damerau–Levenshtein distance ≤ 2 of a mutable binding in scope at that point which is *not* read between the declaration and the end of its scope, the diagnostic names the near miss, carries a machine-applicable fix-it rewriting the name, and carries `note: `x = e` declares when `x` is not in scope and assigns when it is (GRM-4)`. `L1002` is `deny` under `edition_lints = "strict"`.
* `[LNT-3]` `L1001` and `L1002` are emitted by `ember build` and `ember check`, not only by `ember lint`. A diagnostic that fires only on a separate command does not close the footgun ADR-002 names, which is this rule's entire purpose.

## XIX.9 Documentation

`ember doc` renders `##` doc comments to HTML/Markdown; every public function's page shows: signature with modes, **effects**, `@noalloc`-cleanliness, thread rules (`Send`/`Sync` of parameters and result), allocation behaviour (from effect analysis), and for FFI wrappers the underlying C declaration and contract. Doc examples in fenced ```` ```ember ```` blocks are compiled and run as tests by `ember test --doc`.

---

* `[DOC-1]` **Error pages ship with their errors.** Each phase's exit criteria include `docs/errors/EXXXX.md` for every code that phase introduces. A page contains: a minimal program that triggers the error; the rendered diagnostic; one paragraph on **why the rule exists** (not a restatement of the rule); and the fix as compilable code. Every fenced `ember` block under `docs/errors/` is built by `ember test --doc`: the failing example MUST fail with that exact code and the fixed example MUST compile. `[TST-4]`'s index generator fails CI for a code with no page.
* `[DOC-2]` **The user guide is a 1.0 artefact**, not v1.1. `docs/book/` MUST exist, MUST be the landing page with the specification linked from it as the reference, and every sample in it MUST be compiled and run by `ember test --doc`. Its "getting started" chapter is identical to milestone M0 and is tested against it; it MUST include a "coming from Python" chapter carrying the three-column table of silent and diagnosed differences (`/`, `%`, the `f32` literal default, `if xs:`, `is None`, chained comparison, `xs[-1]`, `let`, `with … as`, multi-statement closures) **and the short list of things that behave exactly as in Python**, because the absence of a row is not reassurance.
* `[DOC-3]` `ember doc` moves from Phase 8 to **Phase 4**, whose effects and derives it depends on; from that point each phase documents its own surface as it lands.
* `[DOC-4]` Guide, error pages and specification are published together each release.

## XIX.10 Editor and language-server architecture

* `[IDE-3]` **Resilience.** Every stage after the parser MUST produce a complete result for a file containing errors. Name resolution binds an unresolvable path to `Def::Error`; type checking assigns `Ty::Error` to any expression it cannot type; `Ty::Error` unifies with every type. A file that does not parse MUST still yield hover, completion and document symbols over the regions `[AST-2]` recovered.
* `[IDE-4]` **No codegen in the request path.** Every editor capability MUST be answerable from AST + resolution + HIR alone; none may require MIR, monomorphisation, borrow checking or a C compiler. Borrow- and effect-derived information (`[DIA-8]`, `[EFF-10]`) is surfaced as a **best-effort overlay on a completed `check`** and MUST NOT block a response when none exists. (This is also what keeps the editor independent of ADR-006's backend choice.)
* `[IDE-6]` **Session lifetime.** No compiler crate may rely on process exit to reclaim memory. Interned symbols, interned types, `TypeInfo` and instantiation caches MUST live in an explicitly owned `Session` that can be dropped and rebuilt; `ember_span::Symbol`'s leaked interner is a **v1 defect, not a v1 licence**.

# Part XX — Implementation Plan
* `[IDE-1]`, `[IDE-2]`, `[IDE-5]`, `[IDE-7]`..`[IDE-10]` are **reserved** for the language server itself, a named milestone before 1.0 rather than a v0.4 commitment: `ember lsp` (stdio, LSP 3.17) with `tests/ide/` in the conformance suite; the v1 capability list (diagnostics, hover including the effect set, definition, references, symbols, completion, signature help with parameter modes, rename, formatting, code actions for every machine-applicable `[DIA-1]` fix-it, semantic tokens, inlay hints); incrementality keyed on `[BLD-2]`'s interface hash within `[BLD-7]`'s budget; parity between `ember check` and `ember lsp` tested over the conformance corpus; and editor client packages in `editors/`. `[IDE-7]` **cost inlay hints** is the item worth specifying even if it lands late: the inferred type of each `x = expr`; the parameter mode at each call argument where it is not written; an `alloc` marker at each expression contributing the `Alloc` effect; at each site in the `[EFF-10]` side table, the check kind and its `[EFF-11]` reason code; and each inserted `Retain`/`Release`. That is the per-line form of what Part X mandates be visible.

## XX.1 Ground rules for the implementing agent

1. Work in the order of the phases below. Do not start a phase's optional items before its **exit criteria** pass.
2. Every phase adds tests before code: write the `tests/conformance/<rule>/` cases from the spec text, then implement until they pass.
3. Keep `docs/DECISIONS.md` (ADR format: context, decision, consequences, spec rule affected). Any deviation from this document requires an ADR and a spec patch in `docs/spec-errata.md`.
   * The single-file specification under `docs/spec-source/` is the **normative source**. `docs/spec/` is generated by `tools/split_spec.py` and MUST NOT be hand-edited; an errata ruling is applied to the source document and the split is regenerated. A revision authored from an unpatched copy silently reverts every ruling that exists only in the generated files — which is how 0.3 lost four of them.
4. Keep `docs/HANDOFF.md` in the RageV style: current state, what is done, what is not, load-bearing invariants, traps paid.
5. `cargo test` must be green at every commit; `ember test` on `std/` must be green from Phase 3 onward.
6. Prefer boring implementations. No novel algorithms where a textbook one exists (Pratt parsing, union–find inference, Maranget pattern compilation, NLL).
7. Do not build the LLVM backend, async, named lifetimes, procedural derives, or the kernel language in v1 even if they seem easy at the time.

## XX.2 Phases

### Phase 0 — Skeleton (exit: `hello.em` compiles via C and runs on Windows + Linux)

* Workspace, crates, CI (GitHub Actions: windows-latest with MSVC + clang-cl, ubuntu-latest with clang + gcc).
* `ember_span`, `ember_diag` (rendering + JSON), `ember_lexer` with the indentation algorithm (`[LEX-*]` tests), `ember_parser` for functions, calls, literals, `if`/`while`/`for`, structs (no generics), `ember_ast` pretty-printer.
* Minimal `ember_types`/`ember_typeck` for scalars, structs, `void`, function calls, locals.
* Straight-line MIR lowering (no borrowck yet), `ember_codegen_c`, `ember_rt` with `ember_alloc`, `ember_panic`, `println` for scalars/`str`.
* `ember build/run` driving `cl.exe`/`clang`.
* **Milestone test**: M1 (§XX.3; `Vec3` add) compiles to C with no heap allocation (assert by grepping the C for `ember_alloc`) and prints `5`.

### Phase 1 — Core language (exit: conformance for Parts II–VI except closures/generics)

* Full grammar (`[GRM-*]`), error recovery, formatter (`[FMT-1]`).
* Enums, `match` with decision trees + exhaustiveness, tuples, fixed arrays, `Option`/`Result` as real enums, `?`, ranges, `while/for/else`, labeled break, `with`, `defer`, f-strings (allocating `format`).
* Interfaces (no generics yet beyond `Self`), operator interfaces, method resolution, `extend`, visibility, modules/imports across files, `const`, `static` (comptime-initialised only via literal for now).
* Integer/float semantics (`[TYP-4..10]`), `as`, overflow policies, `Assert` lowering.
* `String`/`str` with SSO, `Array[T]` implemented as a compiler-known type temporarily (replaced by generic std implementation in Phase 2).
* Definite-init analysis (`[CLS-2]`-style for locals).

### Phase 2 — Ownership (exit: all `[OWN-*]`, `[BRW-*]`, `[LT-*]`, `[DRP-*]`, `[SPN-*]`, `[CELL-*]`, `[DIA-7..10]` tests; milestone M2; **zero unclassified borrow errors across the whole test corpus**)

* Generics with bounds, monomorphisation, associated types, `Iterator`/`Iterable` and the adaptor set; `Array`, `Span`, `MutSpan`, `Box`, `Map` written in Ember (`std/collections`).
* Moves, `Copy`, drop elaboration with drop flags, `Drop` interface, `mem.*`.
* NLL borrow checker (§XVIII.4.7) incl. two-phase borrows, disjoint fields, reborrows, view structs `@view`, elision rules, `@borrows`.
* Closures (`[CLO-*]`), `Callable`, `fn(A)->R` generic parameters.
* `Arena`, `FixedArena`, `ScopedArena` (`[ARN-*]`), `unsafe`, raw pointers, `MaybeUninit`, `transmute`.
* `Cell[T]`, `RefCell[T]`, `Ref`/`RefMut` guards (`[CELL-*]`), `std.cell`; `assert_disjoint`/`assume_disjoint` (`[DSJ-*]`) with the proof-carrying return and the backend aliasing facts.
* Diagnostics quality pass on borrow errors: the full shape catalogue of §XIX.6.1 with a `ui/borrow/<shape>/` snapshot each (`[DIA-3]`, `[DIA-7]`, `[DIA-10]`), the classifier, and `ember explain --borrow` (`[DIA-8]`).

### Phase 3 — Objects (exit: `[OBJ-*]`, `[RC-*]`, `[EXC-*]`, `[DSP-*]`, `[WK-*]` tests; leak/cycle report works)

* Classes: header, `init` with definite-init, inheritance, `virtual`/`override`, `abstract`, `let` fields, `is`, `as?`/`as!`, `dyn` for classes and structs (vtables, itables).
* Retain/release insertion; `[RC-2]` guaranteed elisions as MIR passes with tests that count `ember_retain` in emitted C.
* Exclusivity: static analysis (§4.8) + runtime checks; `Shared[T]`, `Weak[T]`.
* `debug_objects` live list, `ember run --leak-check` cycle report, `L3001`.
* Stack promotion (`[OPT-1]`) — optional, behind `-Zstack-promote`.

### Phase 4 — Effects, comptime, derives (exit: `[EFF-*]`, `[CT-*]`, `[RFL-*]`, `[DRV-*]`, `tests/safety/reasons/` fixed for every check site)

* Effect inference + `@noalloc`/`@nosync`/`@noblock` with chain diagnostics; `ember inspect`.
* `RuntimeCheck(k)` effect, the safety-check side table with reason codes (`[EFF-9..11a]`), `@static_safe` (`[EFF-12..14]`) with the S1 diagnostic, and `ember inspect --safety` (`[CLI-3]`). Requires Phase 3's exclusivity analysis, so this item lands after it.
* MIR interpreter, `comptime` blocks/functions, `const` evaluation, comptime-initialised `static`s, `reflect`, `size_of`/`offset_of`, `@gpu_layout` assertions (`[GPU-10]` layout part).
* Derives: `Copy, Clone, Debug, Display, Eq, Ord, PartialOrd, Hash, Default, Zeroable, Reflect, Error`.
* `std.ser.binary`, `std.ser.yaml` with `Serialize/Deserialize` derives.

### Phase 5 — C FFI (exit: `[FFI-1..16, 21..28]` tests; Vulkan header imports; milestone M5)

* `ember_ffi` with `clang-sys`: BIR, type mapping table, macros, bit-fields, unions, opaque types; `.embind` CBOR cache; MSVC-compat parsing.
* Overlays (`[FFI-11..13]`), safe wrappers, `status`/`handle`/`span`/`out` contracts, `cstr`/`CString`, callbacks with `user_data` trampolines, `Retained`, thread attach.
* `@export`, `--emit-header`, `cdylib`/`staticlib` kinds, embedding API (`[FFI-27]`), `@export_table`.
* `cmake/EmberModule.cmake`, CMake File API flag import, Ninja-driven C compilation.
* Fixtures: `vulkan/vulkan.h` (with `VK_NO_PROTOTYPES`), `GLFW/glfw3.h`, `miniaudio.h` (single header, macro-heavy: the stress test), `cgltf.h`.

### Phase 6 — Concurrency and DOD (exit: `[THR-*]`, `[JOB-*]`, `[PAR-*]`, `[SOA-*]`, `[SIMD-*]`, `[ECS-*]`; milestone M6 with benchmark vs C++)

* `Send`/`Sync` derivation, `thread`, `Mutex`, `Atomic`, channels, scoped threads.
* Job system with work stealing, access-set scheduling, job-local arenas.
* `@parallel for` lowering + disjointness proof; reductions.
* `@derive(SoA)`, `columns_mut`, proxies; `std.simd` types + `ember_simd.h`; `restrict` emission from borrow facts; `@simd` diagnostics.
* `std.ecs` per Part XII §3.
* Performance suite (§4) established with C++ references.

### Phase 7 — C++ FFI, and the interpreter as a supported mode (exit: `[FFI-17..20, 24]`, `[CT-*]` differential testing)

* C++ importer: thunk generation, MSVC ABI matching, template instantiation lists, STL views, exceptions → `Result[..,CppError]`, `ember bind --report`.
* *(v0.6)* `std.gpu`: handles, `Device` interface, `Frame`, `Ring`, access states, deferred destruction, `History`, `ShaderInterface`, `ember shader-bind` from SPIRV-Cross JSON — implementing Part XVII's existing `[GPU-*]` rules, and Part XXI's `[RV-*]` rules for RageV Stages 2–3.
* The MIR interpreter becomes a **supported restricted execution mode** (`ember run --interp`) within a declared intrinsic and file-I/O capability set; native FFI is not implicitly available. Differential execution against the native backend is mandatory for deterministic programs. WebAssembly is treated as an implementation target of that mode, never as a semantic dependency of the language.
* **The GPU host model is scheduled, not respecified.** Part XVII's `[GPU-*]` rules and Part XXI's `[RV-*]` rules remain the sole normative source for that surface; implementing them is **v0.6 work** and is not a v1 exit criterion. v0.5 MUST NOT make an architectural decision that would require redesigning them — the compatibility surface that must survive is: Vulkan-capable C/C++ FFI, imported value types and opaque handles, `unsafe overlay` boundaries, callback-bound command signatures (which `[LT-7]` now supplies), frame and arena lifetime primitives, deferred-destruction API shapes, a stable runtime ABI boundary, and the metadata shader reflection needs. If v0.6 requires a genuinely new GPU rule it takes an unused id after the existing Part XVII range; an existing `[GPU-*]` id MUST NOT be reused for a different meaning.

### Phase 8 — Hardening and 1.0 (exit: full conformance + perf suite + two external packages)

* `ember lint` full set, `ember doc`, doc tests, `ember explain`.
* Fuzzing targets, differential testing against the interpreter.
* Package registry client (v1.1), LSP (v1.1), LLVM backend (v2), PGO/ThinLTO (v2).

## XX.3 Milestone acceptance tests (must exist verbatim in `tests/milestones/`)

**M1 — value code has no runtime cost** (Phase 0/1):
```ember
struct Vec3:
    x: f32
    y: f32
    z: f32

fn add(a: Vec3, b: Vec3) -> Vec3:
    return Vec3(a.x + b.x, a.y + b.y, a.z + b.z)

fn main():
    c = add(Vec3(1, 2, 3), Vec3(4, 5, 6))
    println(c.x)
#$ stdout: 5
#$ assert-c: !contains("ember_alloc")
```

**M2 — the borrow checker catches escapes** (Phase 2):
```ember
#$ test: compile-fail
fn first(xs: Array[i32]) -> ref i32: return xs[0]          # ok: tied to xs
fn bad() -> ref i32:
    xs = Array[i32]([1, 2])
    return xs[0]                                              #$ error[E3060]: borrowed value does not live long enough
```

**M3 — objects are deterministic** (Phase 3):
```ember
class Res:
    fn drop(mut self): println("drop")

fn main():
    a = Res()
    b = a
    mem.drop(a)          # only the handle
    println("mid")
    mem.drop(b)          # object dies here
    println("end")
#$ stdout: mid\ndrop\nend
```

**M4 — contracts are transitive** (Phase 4):
```ember
#$ test: compile-fail
fn helper(mut xs: Array[i32]): xs.push(1)

@noalloc
fn hot(mut xs: Array[i32]): helper(xs)                      #$ error[E4001]: @noalloc function `hot` reaches an allocation: hot -> helper -> Array.push -> ember_alloc
```

**M5 — C is free** (Phase 5): import a header with `int calculate(int)`; the emitted C for the call site is a direct call (assert-c contains `calculate(` and no wrapper symbol).

**M6 — DOD matches C++** (Phase 6): `SoA[Particle]` integrate over 1M particles ≤ 1.10× the time of the reference `perf/particles.cpp` compiled with the same C++ compiler at `/O2` or `-O2`.

**M7 — C++ classes bind** (Phase 7): construct/call/destroy a C++ class through generated thunks; a thrown `std::runtime_error` surfaces as `Err(CppError)`; MSVC and clang-cl both pass.

## XX.4 Performance suite

`tests/perf/` holds paired `.em`/`.cpp` programs and thresholds; `ember bench --compare` runs both with the same C++ compiler, 10 iterations, reports median. Gates: scalar/tight loops ≤ 1.05×, SoA/SIMD ≤ 1.10×, FFI call overhead = 0 extra instructions for ABI-direct calls (asm diff), RC-heavy object code ≤ 1.3× of equivalent Swift-style hand-written C++ with `shared_ptr` (informational, not gating), allocation counts asserted exactly for `@noalloc` paths.

* `[BEN-1]` **Protocol.** Each benchmark runs ≥ 3 untimed warm-up repetitions followed by ≥ 30 timed repetitions, in a process pinned to a single core, with the timing loop's working set stated. The harness reports the median and the 95 % bootstrap confidence interval of the median for each side and for their ratio.
* `[BEN-2]` **Gating.** A gate is failed only when the **lower** bound of the ratio's 95 % confidence interval exceeds the threshold. A point estimate above the threshold whose interval includes it is reported *inconclusive* and re-run, never failed.
* `[BEN-3]` **Noise floor.** Every run additionally measures the C++ reference against a second, independently linked copy of itself. If that self-ratio's interval excludes 1.00 ± 0.02, the run is **void** — neither pass nor fail — and the machine is reported unfit for gating.
* `[BEN-4]` **Deterministic signal.** Every benchmark records retired instructions, and cycles where the platform provides them. Instruction count is deterministic for a fixed binary and input; a change beyond ± 0.5 % against the recorded baseline is a hard failure independent of `[BEN-2]`, and is the signal used in CI where `[BEN-3]` voids the timing gate.
* `[BEN-5]` **Configuration equality.** Both sides MUST be built with the same optimisation level, the same LTO setting (`[BLD-6]`), the same floating-point model (`[TYP-9a]`), and the same target-CPU flags. The harness records all four; a mismatch voids the run. M6 and every entry of §XX.4 are subject to this rule.
* `[BEN-6]` **Baselines.** Gates: scalar/tight loops ≤ 1.05×; SoA/SIMD ≤ 1.10×; `@noalloc` paths assert allocation counts exactly. FFI call overhead is gated on **retired instructions for the call sequence** being equal to the C++ reference's, not on an assembly diff. RC-heavy object code is compared against a **non-atomic intrusive reference count** in C++ when the Ember class under test is `!Sync` (gate ≤ 1.15×), and against `std::shared_ptr` only for `Sync` classes (informational). Comparing a non-atomic Ember count against `shared_ptr` is forbidden.
* `[BEN-7]` XXI.6's pass criteria and ADR-008's Stage 2 decision are decided by `[BEN-1]`..`[BEN-6]`; a void or inconclusive run is not evidence for either shape.

## XX.5 Repository layout

```
ember/
  Cargo.toml                 workspace
  compiler/…                 crates per §XVIII.1
  runtime/ember_rt/          C11 runtime, CMakeLists.txt, vendored mimalloc
  std/                       ember.toml + src/**.em
  tools/                     fmt, lint, shader-bind, rule_index.py, perf harness
  cmake/EmberModule.cmake
  tests/                     per §XIX.5
  docs/
    spec/                    THIS DOCUMENT split by part, plus spec-errata.md
    DECISIONS.md  HANDOFF.md  errors/EXXXX.md  book/ (user guide, v1.1)
  examples/                  hello, particles, ecs_demo, vulkan_triangle (via RageV RHI or raw volk)
```

---
# Part XXI — RageV Integration Plan

RageV today: a C++ static library (`RageV`) linked into `RageVEditor.exe`/`RageVRuntime.exe`; a backend-agnostic RHI over Vulkan 1.3 (dynamic rendering, synchronization2, `volk`, VMA) and OpenGL 4.5 DSA; `.rvshader` → glslang → SPIR-V (+ SPIRV-Cross for GL and reflection); a sparse-set ECS with `Entity = 20-bit index | 12-bit version`; a render/frame graph with temporal history, RT/GI signal passes; Jolt, miniaudio, ImGui, yaml-cpp, cgltf; C# scripting hosted through `DotNetHost` where **managed → native is a fixed-order table of `__cdecl` function pointers (`NativeApi`) with a protocol version, and native → managed is a set of static blittable entry points**; every crossing value is blittable, entities cross as `uint64_t` UUIDs, strings as UTF-8 copied at the boundary. Editor-visible script fields are declared with `RVShowInEditor` markers read by `rvgen`.

The plan replaces nothing at first and adds Ember beside C#, matching the existing boundary exactly.

## XXI.1 Stage 0 — Ember as a second scripting language (needs Phases 0–5)

**Goal:** an Ember package `ragev_scripts` builds to `ragev_scripts.dll` (`kind = "cdylib"`) that the engine loads next to (or instead of) the managed assembly, using the same `NativeApi` table.

1. **API package.** `ragev_api` (an Ember library) imports `RageV/src/RageV/Managed/Interop.h` via `import c` with an overlay that turns `NativeApi` into a typed table and wraps each entry:

```ember
## ragev_api/src/lib.em
import c "RageV/src/RageV/Managed/Interop.h" with (project="ragev", overlay="overlays/interop.em")

@derive(Copy, Eq, Hash, Debug)
struct Entity:                                         # UUID across the boundary; never a pointer (play-mode restore)
    id: u64
    const NULL: Entity = Entity(0)

static API: Atomic[*const c.NativeApi] = Atomic(null())   # set once by ember_module_init; read relaxed

fn api() -> ref c.NativeApi:
    unsafe: return API.load(Relaxed).deref()             # SAFETY: set before any script runs; table lives for the process

pub fn log(level: LogLevel, msg: str):
    with c = msg.to_cstring():                           # copied at the boundary, as the C# side does
        unsafe: (api().Log.unwrap())(level as i32, c.as_ptr())

pub fn position(e: Entity) -> Option[Vec3]:
    out: Vec3
    ok = unsafe: (api().GetPosition.unwrap())(e.id, ref mut out as *mut c.Vector3)   # Vec3 is layout-identical to Vector3
    return Some(out) if ok != 0 else None
```

   `ember_module_init(const ember_host_api* host)` receives a struct `{ uint32_t protocol; const NativeApi* api; }`; `[RV-1]` the module refuses to load (`return -1` + log) if `protocol != RAGEV_PROTOCOL_VERSION` baked at build time — the same guard `Interop.ProtocolVersion` provides for C#.

2. **Script model.** A script is a class deriving from `ragev.Script` with the same lifecycle names the C# `ScriptHost` exposes (`on_create`, `on_update(dt)`, `on_fixed_update(dt)`, `on_destroy`, `on_collision(e: CollisionEvent)`), registered by `@derive(Script)`:

```ember
@derive(Script)
class Swing(Script):
    @field(range=(0.0, 2.0))           # editor-visible, serialised — the `RVShowInEditor` equivalent
    amplitude: f32 = 0.34
    @field
    target: Entity = Entity.NULL
    t: f32 = 0.0                       # not a field: runtime-only

    override fn on_update(mut self, dt: f32):
        self.t += dt
        if Some(p) = position(self.entity):
            set_position(self.entity, p + Vec3(0, sin(self.t) * self.amplitude, 0))

    @callable                          # nameable by a UI button, the `RVCallable` equivalent
    fn ring(mut self): log(Info, "ring")
```

   `@derive(Script)` generates: a comptime registry entry `{name, create: extern "C" fn(u64) -> *void, field descriptors (name, type tag, offset, attrs), callables}`, and the class is `Sync`-unconstrained (scripts are main-thread only; the module declares `threads = main`).

3. **Exported table.** The module exports one table (RageV's own style, "order is the ABI"):

```ember
@export_table("RvEmberScriptApi", protocol = 1)
pub struct ScriptModuleApi:
    script_count:        extern "C" fn() -> u32
    script_name:         extern "C" fn(u32) -> cstr
    script_field_count:  extern "C" fn(u32) -> u32
    script_field_desc:   extern "C" fn(u32, u32, *mut FieldDesc) -> i32
    instance_create:     extern "C" fn(u32 /*script*/, u64 /*entity*/) -> u64 /*instance id*/
    instance_destroy:    extern "C" fn(u64) -> void
    instance_set_field:  extern "C" fn(u64, u32, *const u8, u32) -> i32     # raw bytes from the scene file, per type tag
    instance_get_field:  extern "C" fn(u64, u32, *mut u8, u32) -> i32
    instance_call:       extern "C" fn(u64, cstr) -> i32
    on_create/on_update/on_fixed_update/on_destroy/on_collision: extern "C" fn(u64, ...) -> i32
    reload_begin:        extern "C" fn(*mut u8, u32) -> u32      # serialise all instance fields (hot reload)
    reload_end:          extern "C" fn(*const u8, u32) -> i32    # restore after a new DLL is loaded
```

   Instances are addressed by a `u64` from a `Pool[Box[dyn Script]]` (generational; never a pointer), so a stale instance id fails safely. `[RV-2]` No Ember object pointer ever crosses the boundary.

4. **Engine side** (C++, small): `EmberHost.cpp` next to `DotNetHost.cpp`: `LoadLibrary`, resolve `ember_module_init` and `RvEmberScriptApi`, register script types into `ScriptRegistry` exactly as managed scripts are registered, forward lifecycle calls, and drive hot reload (`reload_begin` → unload → rebuild → load → `reload_end`). Ember-side `ember_rt_init` uses `cfg.log = RageV::Log`, `cfg.alloc` left default (mimalloc) or routed to the engine allocator.

5. **Build.** `RageVScripts/CMakeLists.txt` uses `ember_add_library(ragev_scripts KIND cdylib SOURCES …)` with `--cc-flags-from-target RageV`; it is optional like the .NET SDK (`find_program(EMBER ember)` → skip with a status message).

**Exit criteria:** the SampleProject's C# sample scripts ported line-for-line to Ember; play mode, stop/restore, editor field editing, hot reload all work with either language; per-call overhead of `position()` measured ≤ the C# function-pointer path.

## XXI.2 Stage 1 — Engine C API and idiomatic bindings (Phase 5)

Grow `Interop.h` (or a new `RageVCApi.h` generated by extending `rvgen`) into a C surface for: scene queries, component get/set (blittable components), input, audio, physics queries (Jolt via the engine), asset handles, debug draw, and RHI-independent renderer settings. Every entry gets an overlay contract; `ragev_api` exposes idiomatic Ember (`Result`, `Option`, `Span`) over it. `[RV-3]` Component types shared with the engine are `@layout(c)` Ember structs mirrored from the C++ headers with comptime layout assertions against the `.embind` layouts (`[FFI-5]`).

## XXI.3 Stage 2 — Systems in Ember over the engine ECS (Phase 6)

Two options, decided by measurement:

* **(a) Bridge:** the engine exposes `ECS_Pool_Data(type_hash) -> {entities*, count, components*, stride}`; Ember wraps it as `Query` over borrowed `Span`s (zero-copy). Systems written in Ember run inside `Scene::Update` between engine systems. Requires `[ECS-2]`'s identical `TypeHash`.
* **(b) Ember-owned ECS:** `std.ecs.World` becomes the store; the engine's C++ systems read through the same C API in reverse. Larger change; only if (a) shows unacceptable overhead.

Target systems for Ember first: particle simulation (`SoA`, `@parallel`, `@simd`), CPU frustum/occlusion pre-pass feeding `GpuCull`, animation pose evaluation, scene graph transform walk (`Get<T>` two-loads invariant preserved via `[ECS-2]`).

## XXI.4 Stage 3 — Renderer orchestration (Phase 7)

Bind `RHIDevice`, `RHICommandList`, `RHIResourceSet`, `RHIPipeline` through the C++ importer (`import cpp … classes=[…]`) with an overlay that maps the engine's raw pointers to `std.gpu` handles (`Handle[Texture]` ↔ `RHITexture*` via a `Pool` on the Ember side; the engine's own deferred destruction stays authoritative). Then move **frame orchestration** — `FrameGraphBuilder`-level decisions: which passes run, at what resolution, with which history, and the `TemporalHistory` bookkeeping — into Ember, using `std.gpu.History` and access-state checks to make "GI signal read by nobody" and "history refused by validity flag" (the traps recorded in `HANDOFF.md`) into compile-time or debug-time errors: a `History` whose `cur` is never bound in a frame is reported by the device in debug (`W-runtime: history 'gi' written but not read this frame`). Shader interfaces come from `ember shader-bind` on the existing `shaderinfo` reflection so that binding 16's dual meaning (`RayRates.w` bit 24) becomes a typed enum in the interface rather than a comment.

`[RV-4]` The Vulkan and OpenGL backends, `volk`, VMA, swapchain and window code remain C++ ("native islands") indefinitely; nothing in Ember requires their rewrite. Vendor extensions remain reachable through `import c "vulkan/vulkan.h"` + `unsafe: cmd.native()`.

## XXI.5 Stage 4 — RT/GI signal scheduling and tools (v1.1+)

Once Stage 3 is stable: the signal-processing chains (trace → guidance downsample → contract → upsample) are pass sequences whose resolution, ordering and buffer aliasing are orchestration — the kind of code where Ember's `@noalloc` frame arenas, `History`, and the graph's automatic transient lifetimes pay off. Offline tools (`diff_still`, `terrain_lod` replicas currently in Python) become `ember` programs sharing the engine's math and serialisation types.

## XXI.6 Evaluation matrix (measure before moving each area)

| Area | Stage | Ember mechanism | Pass criterion |
|---|---|---|---|
| Gameplay scripts | 0 | classes, `@derive(Script)`, hot reload | feature parity with C#; ≤ C# per-call cost |
| Editor fields/callables | 0 | `@reflect` + `@field` | inspector parity |
| Particles / SoA sims | 2 | `SoA`, `@parallel`, `@simd` | ≤ 1.10× C++ |
| ECS systems | 2 | `Query`, access sets | `Get<T>` two loads; iteration order stable |
| Transform walk | 2 | `Query`, `ref mut` | ≤ 1.05× C++ |
| Frame orchestration | 3 | `std.gpu`, `History`, `Frame` | zero validation errors; histories statically paired |
| Shader interface | 3 | `ember shader-bind` | mismatched binding = compile error |
| Vulkan/GL backends | never | native island | n/a |
| Platform/window | never | native island | n/a |

---

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

# Appendix A — Syntax quick reference

Generated from `docs/spec-source/appendix-a.em`, a conformance fixture that compiles (`[TST-6]`). Every
construct below parses under Part III; nothing here is illustrative shorthand.

```ember
#! language "0.5"
import std.io                              # namespace import
from std.math import Vec3, sin as sine     # item import
import c "vulkan/vulkan.h" with (overlay="overlays/vulkan.em")

const MAX: u32 = 1024                      # compile-time constant
static HITS: Atomic[u64] = Atomic(0)       # global (Sync)

@derive(Copy, Debug, Eq)                   # derives
@layout(c)
pub struct Vec3:                           # value type
    pub x: f32
    pub y: f32
    pub z: f32 = 0.0                       # field default
    fn length(self) -> f32: pass           # borrowed receiver
    fn scale(mut self, k: f32): pass       # mutable receiver
    fn into_array(owned self) -> [f32; 3]: pass   # consuming receiver

pub open class Script:                     # reference type, subclassable
    let entity: Entity                     # immutable field
    pub(read) health: f32 = 100.0          # public read, private write
    fn init(mut self, entity: Entity): self.entity = entity
    virtual fn on_update(mut self, dt: f32): pass

class Door(Script):                        # single inheritance
    angle: f32 = 0.0
    override fn on_update(mut self, dt: f32): self.angle += dt

enum Shape:                                # tagged union
    Circle(r: f32)
    Rect(w: f32, h: f32)
    Empty

interface Drawable:
    fn draw(self, mut cmd: CommandList)

extend Vec3 implements Display:            # external impl
    fn fmt(self, mut f: Formatter) -> Result[void, FmtError]: pass

fn area(s: Shape) -> f32:                  # params: borrowed by default
    return match s:                        # expression `match`: arms are expressions
        Circle(r) => PI * r * r
        Rect(w, h) => w * h
        Empty => 0.0

fn fill(mut buf: MutSpan[f32], v: f32):    # `mut` = inout
    for x in buf.iter_mut(): x = v         # ref mut local writes through

fn consume(owned xs: Array[i32]) -> usize: return xs.len()   # `owned` = move

fn parse(s: str) -> Result[i32, ParseError]:
    n = s.trim().parse[i32]()?             # ? propagates
    return Ok(n)

@noalloc @simd
fn integrate(mut p: SoA[Particle], dt: f32):        # contract + hint
    for i in 0..p.len(): p.velocity[i] += GRAVITY * dt

## Statements live in a function: `file` admits only items (`[GRM-2]`).
fn demo(n: usize, out: MutSpan[f32], inp: Span[f32], m: Mutex[Array[i32]]):
    @parallel(chunk=256)                   # statement attribute (`[ATT-2]`, `[ATT-3]`)
    for i in 0..n: out[i] = f(inp[i])      # parallel loop

    frame = Arena.with_capacity(16 * MB)   # arena
    cmds  = frame.alloc_array[Cmd](count)  # MutSpan tied to frame

    double = fn(x: f32) => x * 2.0         # closure (borrowing)
    task   = owned fn() => run(job)        # escaping closure

    with lock = m.lock(): lock.push(1)     # scoped guard
    defer: cleanup()                       # runs at scope exit
    unsafe: ptr.write(0)                   # unsafe block
    comptime: assert(size_of[Vec3]() == 12)  # compile-time check
    x = a if cond else b                   # ternary
    h: Option[Player] = None
    if Some(p) = h: p.health -= 1          # pattern condition
```

*End of specification.*
