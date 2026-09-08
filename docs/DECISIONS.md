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

---

## ADR-010 — Regions are location-insensitive, and a reference local is assigned once

**Spec rule:** Part XVIII §4.7 steps 1–4, `[LT-5]`.
**Status:** taken 2026-09-09, on the permission §4.7 gives.

**Context.** §4.7 says "Polonius-style location-sensitive reasoning is not
required for v1", so a constraint may propagate a whole point set rather than
the points reachable from where the assignment sits. What that costs is
precision in one shape: a reference local assigned twice, where the second
loan's region would absorb the first's live range and the first would look live
during it.

**Decision.** Constraints propagate whole point sets. The imprecision is
unreachable in Ember, because a reference local cannot be re-seated: `r = ref
mut m` writes *through* `r` (`[TYP-14]`), so every reference local is assigned
exactly once, at its declaration.

**Consequences.** The inference is a fixpoint over one edge list rather than a
reachability computation per constraint, which is most of why the pass is short.
**If a future revision adds a way to re-point a reference** — a `ref` field
assigned twice, a reference in a container, a loop that rebinds one — this
decision expires with it, and the checker will report a borrow as live across a
range where it is dead. The module header in
`compiler/ember_analysis/src/regions.rs` states the assumption where it is
relied on.

---

## ADR-011 — `E3060` for storage, `E3062` for elision

**Spec rule:** Part XVIII §4.7 step 6, `[LT-1]`, `[LT-1a]`, ERR-024.
**Status:** taken 2026-09-09.

**Context.** §4.7 step 6 names two codes and a returned borrow of a local
satisfies the description of both: its region outlives the borrowed place's
storage (`E3060`), and it is a return that derives from no parameter
(`E3062`). Shape B7 is keyed to the first and B6 to the second, so the choice
decides which `help` the programmer is shown.

**Decision.** The split is by *what is wrong*, not by where it was noticed.
`E3060` when the borrowed storage ends first — a local, or a parameter passed
by value (ERR-024). `E3062` when the storage outlives the call but elision does
not tie the return to it — `@borrows` naming a different parameter, or
`[LT-1]` rule 1's borrowing receiver where the result points into an argument.

**Consequences.** The two diagnostics say different things and neither is a
fallback for the other: B7 tells you the value dies too soon, B6 tells you the
signature does not permit what the body did. A program can hit both, and does
— `tests/compile-fail/a_returned_view_elision_cannot_tie.em` carries one of
each. The cost is that "returned reference does not derive from a parameter",
`E3062`'s registry title, is now narrower than its wording: a borrow of a local
derives from no parameter either and is `E3060`.

---

## ADR-012 — `E3064` stays unemitted until a program needs it

**Spec rule:** `[LT-2]`, `[DIA-7a]`, §XIX.6.1 shape B13.
**Status:** taken 2026-09-09. Revisit when `[TYP-15a]`'s `BorrowList`/`ViewList`
land, which is where two element regions could first be asked for.

**Context.** `E3064` "two independent regions in one view struct" is registered
and keyed to shape B13. `[LT-2]` says a view struct built from several
references takes the **intersection** of their regions — so construction always
has an answer, and the compiler narrows rather than refusing. No program has
been found that reaches the error.

**Decision.** Leave it unemitted and record why, rather than invent a trigger.
`docs/DEFECTS.md` carries it as an open row.

**Consequences.** `[DIA-7a]` is satisfied either way — it forbids emitting a
code that is *absent* from the shape table, not leaving a present one unused.
The risk accepted is that a program which should be rejected is quietly
narrowed instead; the intersection is sound, so it is a precision loss and not
a soundness one. **The alternative rejected:** writing a diagnostic against a
guessed trigger, which enforces the rule in a shape nobody asked for and is
harder to remove later than to add now.
