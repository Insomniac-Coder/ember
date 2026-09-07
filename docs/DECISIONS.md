# Ember — decision record

ADR format: context, decision, consequences, spec rule affected. One entry per
decision that is not already forced by the specification.

Per Part XXII.1 the nine questions below require an owner decision and an agent
MUST NOT choose one silently. On 2026-09-07 the owner confirmed all nine as the
specification writes them. Entries marked **deferred** are confirmed as a
direction but their concrete answer waits for the evidence named in the entry.

---

## ADR-001 — Float literals default to `f32`

**Spec rule:** `[LEX-17]`, Part 0 #12.
**Status:** confirmed by the owner, 2026-09-07.

**Context.** An unsuffixed float literal has to land on some type when no
context supplies one. C, Python and Rust all land on 64-bit. The alternative
is `f64` with `f32` suffixes written throughout engine code.

**Decision.** Untyped float literals resolve to `f32` absent context.
`1.0f64` or an annotation selects `f64`.

**Consequences.** Graphics and engine arithmetic — the reference workload —
never accidentally promotes a temporary to double. Numeric code ported from
C or Python that assumes double precision will silently lose it unless
annotated; the risk is real and is accepted. `[LEX-16]`/`[LEX-17]` are the
only literal coercions, so the rule is at least uniform.

---

## ADR-002 — Block scoping, not Python function scoping

**Spec rule:** Part 0 #13, `[GRM-4]`, `[DRP-2]`.
**Status:** confirmed by the owner, 2026-09-07.

**Context.** Deterministic destruction needs a defined point at which a value
dies. Function scoping (Python) has no such point inside a function body.

**Decision.** Block scoping. Values drop at end of block in reverse
declaration order. NLL governs borrows.

**Consequences.** Required for `with`, `defer` and RAII to mean anything.
Combined with `[GRM-4]` (`x = expr` declares when `x` is not in scope) it
carries a cost worth naming: a misspelled name inside a nested block declares
a fresh local rather than failing, and the outer variable is left unwritten.
This is Python's oldest footgun in a language whose premise is that the
compiler catches such things. A lint could recover most of it — "assignment
declares a new binding that is never read" — but no such lint is specified.
Raised for the record; the rule stands.

---

## ADR-003 — `pub` for visibility

**Spec rule:** Part 0 #11, `[MOD-2]`, `[MOD-7]`.
**Status:** confirmed by the owner, 2026-09-07.

**Context.** `export`, `public` and `pub` were all candidates.

**Decision.** `pub`, with `pub(package)` and the field-only `pub(read)`.

**Consequences.** Short, reads naturally on fields. Familiar to Rust readers,
unfamiliar to the Python and C# readers who are the scripting audience.

---

## ADR-004 — Class exclusivity checks stay enabled in `release`

**Spec rule:** `[EXC-1]`, `[EXC-2]`, `[PRF-1]`.
**Status:** confirmed by the owner, 2026-09-07.

**Context.** Overlapping mutable access through aliasing class handles is
memory-unsafe. The check is a load, a compare and a store on a header word
(~2 ns uncontended). Swift enforces it in release; the alternative is
debug-only checking and undefined behaviour in release.

**Decision.** Checked in `debug` and `release`. `exclusivity = "unchecked"`
is permitted only in the `shipping` profile, where a violation is UB.

**Consequences.** Object-world code carries a small, predictable per-access
cost in shipped-but-not-shipping builds. Classes are documented as unsuitable
for inner loops; the data-oriented facilities in Part XII are the answer
there. `[PRF-1]` already lists this as one of the three permitted
profile-dependent semantic differences.

---

## ADR-005 — Generic arguments use `[]`, not `<>`

**Spec rule:** `[GRM-8]`, `[GRM-9]`.
**Status:** confirmed by the owner, 2026-09-07.

**Context.** `<>` forces the parser to disambiguate against comparison
operators (C++'s and Rust's problem; Rust pays for it with the turbofish).
`[]` collides with indexing instead.

**Decision.** `[]`. `name[...]` in expression position parses to a single
`IndexOrInstantiate` node and is resolved during name resolution by what
`name` resolves to.

**Consequences.** Keeps Python's look, and type position needs no
disambiguation at all. The cost is an AST node whose meaning is unknown until
after name resolution, so the parser cannot report "indexing a non-indexable
type" and index expressions cannot be folded pre-resolution. Accepted.

---

## ADR-006 — C11 backend first, LLVM in v2

**Spec rule:** Part 0 #7, Part XVIII.6, XVIII.7.
**Status:** confirmed by the owner, 2026-09-07.

**Context.** LLVM first delays a working toolchain and RageV integration by an
estimated one to two phases and requires an LLVM install at every consumer
site. Without a GC there are no safepoints, so C loses nothing essential.

**Decision.** `ember_codegen_c` is the v1 backend. `ember_codegen_llvm`
begins only when the C backend passes the full conformance suite, and becomes
default only when it also passes the performance suite.

**Consequences.** MSVC is a first-class target, which costs the workarounds
already named in Part XVIII.6 (`ember_i128`, `ember_ck_*` for checked
arithmetic, `_Alignas` handling). Console toolchains remain reachable.
Optimisation reports, PGO and precise vectorisation wait for v2.

---

## ADR-007 — Reference cycles leak, with tooling rather than a collector

**Spec rule:** `[WK-1]`, `[WK-2]`, `L3001`.
**Status:** confirmed by the owner, 2026-09-07.

**Context.** Reference counting cannot reclaim cycles. The alternative is a
backup cycle collector (CPython's approach), which adds a tracing pass to a
runtime that otherwise has none.

**Decision.** Cycles leak. This is documented behaviour, not a defect.
Mitigations: `Weak[C]` for back-pointers, the debug runtime's cycle report
(`ember run --leak-check`) walking the intrusive live-object list at exit and
printing strongly connected components with the field names that form them,
and lint `L3001` for statically visible cycles.

**Consequences.** The runtime stays free of tracing. Long-running processes
with programmer-authored cycles grow. The debug live-object list is what makes
this tolerable, so it is not an optional feature — it is load bearing and must
land with Phase 3.

---

## ADR-008 — RageV ECS integration shape — **deferred to measurement**

**Spec rule:** Part XXI.3, `[ECS-2]`.
**Status:** direction confirmed 2026-09-07; the answer waits for Phase 6.

**Context.** Two shapes. (a) Bridge: the engine exposes pool data
(`entities*, count, components*, stride`) and Ember wraps it as a `Query` over
borrowed `Span`s, zero-copy, with systems running inside `Scene::Update`.
(b) Ember-owned ECS: `std.ecs.World` becomes the store and the engine's C++
systems read back through the C API.

**Decision.** (a) is the default and (b) is attempted only if (a) measures
unacceptably. The measurement is the pass criterion in XXI.6: `Get<T>` in two
loads, iteration order stable, transform walk within 1.05x of C++.

**Consequences.** `[ECS-2]`'s component identity must be a 64-bit FNV-1a hash
of the fully-qualified type name, byte-identical to RageV's `TypeHash`, or (a)
is impossible. That constraint binds from Phase 6, and any earlier design that
would break it is a defect.

---

## ADR-009 — Language and CLI naming — **open, check pending**

**Spec rule:** Part XXII.1 #9.
**Status:** confirmed as a task, not as an answer. Owner asked for the check.

**Context.** The specification asks for a conflict check against crates.io,
PyPI and npm before anything is published. `ember` is also the name of a
large, long-established JavaScript framework (Ember.js), which at minimum
contends for the `ember` name on npm and for search results generally. The
names at stake are the CLI (`ember`), the compiler crate (`emberc`), the
manifest (`ember.toml`), the source extension (`.em`) and the runtime
(`ember_rt`).

**Decision.** Not yet taken. The check runs before the first published
artefact. Nothing before publication depends on the name being final, but the
cost of changing it rises with every test annotation, CMake module and
documentation page written.

**Consequences.** If the name changes, the rename touches crate names, the CLI
binary, `ember.toml`, the `.em`/`.embind` extensions, `EMBER_RT_ABI` and every
`ember_*` runtime symbol, `cmake/EmberModule.cmake`, and every `#!` test
annotation. Mechanical, but wide. Recorded here so the decision is not made by
default.
