# Migrating the repository to Ember v0.8.3

**Status:** v0.8.3 is installed as the normative specification
(`docs/spec-source/ember-spec.md`, split into `docs/spec/`). This file is the
analysis that decided the work and the ordered route through it. It is not
normative; the specification is.

**Read `docs/HANDOFF.md` first** for the live state. This file answers three
questions once, so they are not re-derived: what changed, what the compiler
already has, and in what order the rest is built.

---

## 1. What arrived

`Ember_v0.8.3_spec.md`, 5,373 lines / 578 kB, authored from 0.8.2c ← 0.8.2b ←
0.8.1 ← 0.8 ← 0.7.2 ← 0.7.1 ← 0.6.3. The repository was on **0.5**. Seven
revisions landed at once.

| | v0.5 | v0.8.3 |
|---|---|---|
| Rule ids referenced | 558 | 833 |
| Rules *stated* (bullet definitions) | 437 | 676 |
| Diagnostic codes named | 138 | 205 |
| Parts | 0–XXII + Appendix A | 0–XXIII + Appendix A |

### 1.1 The single most important finding

**No v0.5 rule id is absent from v0.8.3.** Checked mechanically over both
documents: the set difference is empty in that direction. The lineage note's
claim — that the 239 rules 0.7 silently reverted are all retained — holds.

**Nothing already implemented is invalidated.** The diffs over the Parts the
compiler implements are:

| Part | Changed lines | What |
|---|---|---|
| II Lexical | 3 | `[LEX-15b]` added; `[LEX-15a]` gains the range-type meaning of `type` |
| III Grammar | 22 | `range_clause`, `[grade]` on `attr_arg`, `{attribute}` on `statement`, 13 new attributes in the §7 table |
| IV Types | 92 | §IV.2a range types (all of it), `[TYP-5]` gains range erasure |
| V Declarations | 12 | `[ATT-5]`, `[GRM-20..23]`, `[GRM-8d]`, `[FFI-34a]` — all additive |
| VI Expressions | 31 | §VI.5a coroutines (all of it), the `not in` row |
| VII Ownership | 2 | a Part cross-reference renumber, nothing else |

Every other line of change in those Parts is a Part renumber (old Part XVIII
Compiler Architecture is now XIX, XIX Toolchain → XX, XX Implementation Plan →
XXI, XXI RageV → XXII, XXII Open Questions → XXIII; the new Part XVIII is Hot
Reload).

So "everything implemented so far has to be altered to match this spec"
resolves to a short, concrete list — §3 below — and the bulk of the 275 new
rule ids land in territory the compiler has not reached.

### 1.2 Where the 275 new rules are

    HR   63   hot reload (new Part XVIII)      RNG  16   range types
    FFI  43   C++ boundary, grades, adopt      BUD  12   compile-time budget
    CORO 11   coroutines                       DET   9   determinism
    GATE  9   the 1.0 release gate             MONO  8   instantiation budget
    TXT   8   the string and text model        CXX   7   C++ importer corpus
    EFF   7   Io/Lock, @nopanic, @realtime     CONF  6   conformance profiles
    TCB   6   trusted-base report              STD   6   layers, Contains
    ABI   5   protocol versions                CAT   5   requirement categories
    COST  5   the cost model                   SEL   2   storage selection
    …

Two of these are **not** deferrable, because `[CONF-2]` puts them inside Ember
Core: `[RNG-*]` (range and domain types) and, through `[STD-8]`, the `in`
operator's `Contains` bound.

### 1.3 Contracts are gone

0.6.2 removed `@requires`, `@ensures`, `@invariant`, `@decreases`, `@verified`,
`@assume`, the `[CTR-*]` and `[PRV-*]` rules, the proof manifest, the
`--contracts`/`--verify` flags and the SMT solver. `docs/RFC-v0.6.md` argued
for taking them from Aegis; that argument was heard and **declined** (`OQ-27`).
Range types, `Option`, `Result`, `.get(i)` and `NonZero[T]` are the mechanism
instead: make the bad value unconstructible rather than write a rule about it.

`docs/RFC-v0.6.md` is therefore **history, not a plan**. It is kept because it
records why each Aegis feature was taken or refused, and eight of its eleven
features did ship.

### 1.4 Two open defects, both closed by v0.8.3

`docs/DEFECTS.md` carried D-012 and D-013 against the owner's v0.6 revision.
Both are fixed in v0.8.3 and are closed in the ledger:

* **D-012** — `E1020` used for two errors. `[BLD-11]` now says
  `E1021 name is not linked in this build` and states explicitly that
  "`E1020` remains `[GRM-4]`'s and MUST NOT be reused".
* **D-013** — `[STD-7a]` cited `[CG-1]`..`[CG-4]`, which did not exist. It now
  cites `[GRM-8]`, `[CT-1]`, `[MONO-1]` and `[TYP-19]`, all of which do.

---

## 2. What the compiler has today

~24,000 lines of Rust across 15 crates, 161 green tests, four green gates.

| Area | State |
|---|---|
| `ember_span`, `ember_diag` | done for the surface built; 145 of 205 codes registered; shapes B*/O*/N*/A1/X1/S1 partially |
| `ember_lexer` | indentation algorithm, literals, raw identifiers, 48-keyword set, doc comments |
| `ember_ast`, `ember_parser` | the whole v0.5 grammar, error recovery, Pratt expressions, `IndexOrInstantiate` |
| `ember_types`, `ember_typeck` | scalars, `struct`, `enum`, tuples, fixed arrays, `ref`/`ref mut`, `str`, a compiler-known `Array`, generics **with monomorphisation folded into typeck**, interfaces, `extend`, bidirectional inference |
| `ember_hir`, `ember_mir` | lowering, verifier |
| `ember_analysis` | definite-init, **NLL borrow checker with a real region graph**, drop elaboration, `@borrows` validation, unused-binding lints |
| `ember_codegen_c` | C11 backend for the surface above |
| `ember_fmt` | `[FMT-1]` idempotence over the corpus |
| `ember_build`, `ember_driver` | `build` / `run` / `check` / `fmt` / `explain` |
| `runtime/ember_rt` | C11 runtime skeleton |
| `std/` | **empty — no `.em` file exists** |
| `tests/conformance/` | **empty — no rule has a directory** |

Against Part XXI's phases: **Phase 0 complete, Phase 1 complete, Phase 2 in
progress** (its block E, the borrow checker, landed on 2026-09-09).

Absent entirely: classes and RC (Part VIII), memory facilities beyond raw
pointers (IX), effects (X), concurrency (XI), DOD (XII), comptime and derives
(XIV), the standard library (XV), both importers (XVI), GPU (XVII), hot reload
(XVIII), and most of the toolchain (XX).

Deviations from Part XIX's crate layout — no `ember_resolve`, `ember_mono`,
`ember_opt`, `ember_interp`, `ember_ffi`, `ember_abi` — are permitted:
`[CAT-1]` categorises Part XIX as `REFERENCE-IMPLEMENTATION`, and `[CAT-3]`
forbids a language rule from resting on one. They are recorded, not treated as
defects.

---

## 3. The rework list — what v0.8.3 changes in code that exists

This is the whole of it. Everything else in §4 is new construction.

| # | Rule | Change | Touches |
|---|---|---|---|
| R1 | `[LEX-15b]` | `yield` moves from the reserved-for-future list into the reserved keyword set; the set becomes **49**, not 48. `gen` becomes contextual — a keyword only immediately before `fn`. | `ember_lexer::token` (the `assert_eq!(Kw::ALL.len(), 48)`), the future-word table, `E0005`'s message |
| R2 | `[DIA-6a]` | 67 codes named by the document are absent from `ember_diag::codes`, and the registry is normative in both directions. | `ember_diag::codes`, `docs/errors/` |
| R3 | `[TYP-5]` | Coercion sites gain **range erasure**: a range type coerces to its representation, composing with widening. | `ember_typeck` coercion |
| R4 | `[GRM-8d]`, `[RNG-1]` | `type_alias` gains `range_clause`; a `type` alias with an `in` clause is a nominal type, one without is transparent. | `ember_ast::TypeAlias`, `ember_parser::decls` |
| R5 | `[GRM-23]`, `[STD-8]`, `[STD-8a]`, `[STD-8b]` | `in`/`not in` parse already but type to nothing. They must lower to a **declared `Contains` bound**, never a synthesised scan; a type with no impl is `E2226`. | `ember_typeck`, `std.core` |
| R6 | `[GRM-16]` | The rule deletes `"return"`, `"break"`, `"continue"` from `small_stmt`; III.4's production still lists them. Already implemented correctly as `Jump`. | nothing — recorded so it is not "fixed" backwards |
| R7 | `[ATT-2]`/`[ATT-3]`/`[ATT-5]`, Part III §7 | 13 new attributes must be recognised, and each rejected with `E0104` naming the positions it does admit when misplaced. | `ember_parser`, attribute validation |
| R8 | `attr_arg := … [grade]`, `[FFI-34a]` | `@ffi(effects=[FFI] @instrumented)` — a `grade` suffix on an attribute argument, admitted **only** there. | `ember_ast::AttrArg`, `ember_parser` |
| R9 | `[MOD-7]` | `pub(read)` / `pub(package, read)` — `VisKind` has no `Read` variant, so `E1050`/`E1051` cannot fire. Pre-existing gap, not new in 0.8.3, but Core needs it. | `ember_ast::VisKind`, resolution, `ember_fmt` |

R1, R2, R6, R7, R8 and R9 are mechanical. R3, R4 and R5 are the range-type and
`Contains` work and belong with §4's stage C1.

---

## 4. The route

Ordered so that each step's exit is checkable and nothing later depends on a
step that was skipped. Part XXI §1 ground rule 1 governs: no phase's optional
items before its exit criteria pass.

### Stage A — make the document the law in the repository

*No compiler change. Everything here is a gate or a record.*

* **A1** Install v0.8.3 as `ember-spec.md`, regenerate `docs/spec/`, teach
  `split_spec.py` about Part XXIII, delete the four stale part files. **Done.**
* **A2** Grow the gates to what v0.8.3 asks of them:
  * `[DIA-6a]` — a rule with no change-log row; two rules naming one code with
    different titles; an attribute, construct or CLI surface named in normative
    text but absent from Part III §7 / `[ATT-2]` / XX.1 / XXIII.3.
  * `[TST-4b]` — generate and commit the reject-case waiver list from the
    rule→code map, so no list is maintained by hand.
  * `[TST-4c]` — the accept-only baseline, which may shrink and never grow.
  * `[CAT-2]` — the one-time mechanical categorisation pass over all 676 stated
    rules, defaulting per Part and overriding the `ABI-NORMATIVE` ones.
  * `[GATE-8a]` — a status on every `OQ-n`; an `open` one cited by a
    `LANGUAGE-NORMATIVE` rule fails CI.
  * `[COST-5]` — a post-0.8 rule introducing an implicit cost with no
    `[COST-3]` row.
  * `[PHIL-8a]` — for each diagnostic shape, the recorded "before" program fails
    with that code and the "after" program compiles.
* **A3** Record the defects found in v0.8.3 (§5) in `docs/spec-errata.md` and
  `docs/DEFECTS.md`, each with the reading taken and why.
* **A4** Rewrite `docs/HANDOFF.md`'s opening section; retire `docs/RFC-v0.6.md`
  to history; refresh `DECISIONS.md`, `BACKLOG.md`, `LIBRARIES.md`.

### Stage B — the rework list

§3, items R1, R2, R7, R8, R9. R3/R4/R5 fold into C1.

### Stage C — Ember Core (`[CONF-2]`), which is Phase 2's exit

`[CONF-2]` names it exactly: **Parts II–VII and XIII, plus `[RNG-*]`**. That is
the next declarable milestone and the first one `[CONF-1]` lets the compiler
claim.

* **C1** Range and domain types — `[RNG-1..10c]`, `[TYP-5]` erasure,
  `[RNG-5a1]`'s generated operator impls in the declaring module,
  `[RNG-5a2]`'s exact-impl-before-coercion rule, `E2210`–`E2215`. The one
  genuinely new *language* feature Core gains.
* **C2** Closures — `[CLO-1..7]`, `Callable`/`CallableOnce`, capture inference,
  `owned fn`.
* **C3** `Cell` / `RefCell` / `Ref` / `RefMut` — `[CELL-1..11]`.
* **C4** `Arena`, `FixedArena`, `ScopedArena` — `[ARN-1..7]`; `assert_disjoint`
  / `assume_disjoint` — `[DSJ-1..9]`.
* **C5** `std/` as a real package: `Option`, `Result`, `Array`, `Span`,
  `MutSpan`, `Box`, `Map`, the iterator adaptor set — replacing the
  compiler-known `Array`. `[SPN-1..3]`, `[TYP-15a]`.
* **C6** The string and text model — `[TXT-1..8]`. `str` exists as a type; the
  invariant, the fallible conversions and `[TXT-4]`'s boundary rule do not.
* **C7** `[CTL-3b]`/`[CTL-3c]` guaranteed iteration lowering, with the
  emitted-C assertions.
* **C8** The diagnostic catalogue — every shape in §XX.6.1 and §XX.6.2 with a
  `tests/ui/` snapshot and a `.fixed.em` companion, the classifier, and
  `ember explain --borrow`. `[DIA-7]`'s "zero unclassified borrow errors" is a
  Phase 2 exit criterion.
* **C9** `tests/conformance/<rule-id>/` for every Core rule, with `[TST-4a]`'s
  accept **and** reject cases.

### Stage D — the remaining phases, in Part XXI's order

Phase 3 objects → Phase 4 effects/comptime/derives → Phase 5 C FFI → Phase 6
concurrency and DOD → Phase 7 C++ FFI → Phase 7a coroutines, determinism, hot
reload, budgets → Phase 8 hardening. Each is scoped by its exit criteria in
Part XXI §2 and by the conformance profile it completes: Systems after Phase 6,
Native after Phase 7, Dynamic after Phase 7b.

---

## 5. Defects found in v0.8.3

Fifteen, all recorded in `docs/spec-errata.md` with the reading taken. None is
unresolvable; none blocks the work. They fall into four classes.

**Class 1 — a rule superseding a table that was not updated.** `[LEX-15b]`
makes `yield` a v1 keyword while II.4's reserved-for-future list still contains
it; `[GRM-16]` deletes three alternatives from `small_stmt` while III.4's
production still lists them. Both rules say explicitly that they supersede, so
the reading is forced.

**Class 2 — leftovers from the removed contract layer.** `[UNS-7]` still says
an obligation SHOULD additionally carry `@requires`; `[STD-6]` still lists a
**verify** layer; `[EFF-18]` and `[EFF-17]` still name a fifth `RuntimeCheck`
kind `Contract`; the code registry paragraph still announces `E4050`–`E4057`
and `E4060`–`E4064` and `W4001`. 0.6.2 removed all of it.

**Class 3 — an editorial instruction pasted instead of carried out.** `[BLD-2]`,
`[FFI-34]`, `[FFI-38]`, `[FFI-2a]`, `[BLD-11]`, `[TCB-5]`, `[FFI-33b]`,
`[TST-13]`, `[RNG-7]`, `[RNG-8]`. In every case the replacement text is quoted
in full beside the instruction, so the meaning is recoverable — this is the
third revision to ship it, and `[DIA-6a]`'s own completeness pass is asked to
grow a check for it.

**Class 4 — one code with two meanings.** `[GRM-23]` reports `a in b in c` as
`E0104`, which `[ATT-1]` already owns for an unknown attribute; `[DIA-6a]`
requires a code to be defined by exactly one rule. `E0102` (chained comparison)
already covers it, and `in` sits at comparison precedence by that same rule.

Plus `[TST-11]`, whose text carries a merge artefact
("The v0..6]` evidence invalidation … .ser.yaml`"); the intent is legible from
the surrounding clauses and is recorded as read.

---

## 6. Facts that decide later work

* `[CONF-2]` puts range types in **Core**, not in a later profile. C1 is not
  optional and not deferrable.
* `[HR-12]` grows the object header from 24 to **40 bytes in reloadable builds
  only**, and `[HR-12a]` makes the layout a whole-process property enforced by
  a link-time symbol. Part VIII's design must leave room for that before
  classes are built, not after.
* `[EFF-15]`'s contract profile means a contract has **one verdict per source**,
  not one per build profile. Effect analysis must run after monomorphisation
  and after the target-independent MIR optimisations, under a pinned elision
  set — Part XIX §1 says so explicitly.
* `[MONO-6]`'s shared instantiation is available **only** when a ceiling is set.
  With no `max_instantiations`, the compiler MUST NOT share anything.
* `[PRF-1]` admits exactly three profile-dependent behaviours: overflow policy,
  class exclusivity under `exclusivity = "unchecked"`, and the presence of debug
  facilities. Nothing else may vary.
* `[CMP-3]` (new in 0.8.3): a rule may not be justified by "the C backend
  cannot do otherwise". The correct outcome is a recorded gap and an LLVM-only
  capability.
