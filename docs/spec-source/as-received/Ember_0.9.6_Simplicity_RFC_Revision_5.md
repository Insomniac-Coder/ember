# Ember 0.9.6 — Simplicity Consolidation RFC
## Revision 5 — Final Editorial Corrections

**Document type:** Owner/design RFC  
**Target:** Ember `0.9.6_Hardened_N`  
**Basis:** frozen `Ember_v0.9.6_Hardened_4.md`  
**Status:** architecture/process RFC; not a normative specification amendment and not an implementation-status ledger

---

# 0. Purpose

This RFC proposes a second, more conservative simplicity pass for Ember 0.9.6.

Revision 1 correctly identified opportunities to reduce duplicated compiler and library concepts, but several proposals risked simplifying away distinctions that H4 deliberately requires.

Revision 5 therefore adopts a stricter principle:

> **A simplification is acceptable only when it reduces duplication without collapsing a semantic distinction that H4 relies on.**

The objective is not to make Ember smaller by removing expressive power.

The objective is to make Ember:

```text
simple at the source level
        +
small in conceptual vocabulary
        +
shared internally where semantics permit
        +
strictly conservative at uncertainty
        +
unchanged in safety guarantees
```

H4 remains the reference design. No proposal in this RFC overrides H4 merely by being written here.

---

# 1. Authority and Evidence Boundary

## 1.1 Current authority

The repository's currently adopted normative language source remains:

```text
docs/spec-source/ember-spec.md
```

corresponding to the adopted 0.8.5 lineage.

`Ember_v0.9.6_Hardened_4.md` is a frozen development target and design reference,
not the repository's adopted normative source.

This RFC is an architecture/process proposal derived from H4. It does not amend
the language merely by existing.

## 1.2 Implementation status

This RFC MUST NOT act as an implementation-status ledger.

Implementation state belongs in the project's existing implementation,
conformance, and owner ledgers.

The RFC may describe required architecture, design guidance, and acceptance
conditions, but MUST NOT claim:

```text
implemented
verified
conformant
passing
```

unless that evidence exists in the appropriate repository ledger.

## 1.3 Normative boundary

For today's adopted language, `docs/spec-source/ember-spec.md` is authoritative. For the proposed 0.9.6 target architecture, the frozen `Ember_v0.9.6_Hardened_4.md` document is the authoritative target design until the 0.9.6 target revision is formally adopted.

This RFC may recommend:

```text
compiler organization
internal representations
documentation generation
tooling
process gates
```

but it MUST NOT silently:

```text
change accepted-program semantics
change rejected-program semantics
weaken a MUST requirement
create a competing source of truth
```

## 1.3.1 RFC modality scope

Throughout this RFC, uppercase `MUST`, `MUST NOT`, `SHOULD`, and `MAY` specify
requirements for **architecture/process adoption of this RFC**. They are not,
by themselves, additions to the source-language conformance contract.

A source-language requirement becomes normative only when it is incorporated into
the adopted specification through the project's normal owner-review, versioning,
and conformance process.

## 1.4 No automatic new hardening revision

Adoption of this RFC alone does NOT create `0.9.6_Hardened_5`.

A new hardened revision is justified only when a concrete specification
clarification, consistency repair, implementation-contract change, or other
approved modification is actually materialized and passes the project's normal
gates.

# 2. Design Principles

## [SIMPL-P1] Minimize programmer-visible concepts

Ember should minimize programmer-visible concepts while preserving the full
safety and performance contract.

The normal programmer model should remain close to:

```text
values
ownership
borrows
mutation
interfaces/generics
views
containers
```

Compiler-internal proofs need not become source-level concepts merely because
they exist.

## [SIMPL-P2] One authoritative representation per invariant

Ember SHOULD maintain one authoritative representation of every semantic
invariant.

For example:

```text
borrow safety      -> established borrow/region rules
call access        -> AccessContract
effects             -> EffectSet
initialization      -> InitializationState
ownership           -> OwnershipGraph
layout              -> LayoutDescriptor
```

Derived forms are permitted:

```text
indexes
caches
summaries
serialized compiler metadata
backend-specific forms
```

provided they are mechanically derived from the authoritative facts, validated
against them, and never become an independent source of semantic truth.

## [SIMPL-P3] Preserve orthogonality

Simplification MUST NOT collapse independent semantic facts merely because they
are stored together internally.

At minimum, Ember keeps distinct:

```text
provenance
storage identity
access permission
reference representation
ownership relationship
acquisition/check mechanism
safety authority
synchronization domain
initialization state
effects
validity
escape constraints
```

## [SIMPL-P4] Reuse before abstraction

Before adding a new language or library abstraction, the design MUST ask:

```text
Can an existing type/interface/parameter mode/compiler fact express this?
```

A new abstraction is justified only when existing mechanisms cannot express the
required semantics without loss of safety, clarity, ABI guarantees, or required
performance.

## [SIMPL-P5] Conservative uncertainty

Absence of proof grants no capability.

The implementation uses:

> **the least-permissive sound fact for the domain.**

Examples:

```text
unknown borrow overlap
    -> no disjointness/noalias proof

unknown callable access
    -> all relevant fields

unknown foreign retention
    -> all relevant regions

unknown dynamic target set
    -> union when soundly bounded; otherwise conservative

unknown initialization
    -> Maybe / not proven initialized
```

Unknown initialization MUST NOT be silently treated as definitely uninitialized.

## [SIMPL-P6] No hidden weakening

A simplification is invalid if it converts an existing compile-time safety
guarantee into a runtime best effort.

In particular:

```text
ordinary borrow conflict
    -> compile-time rejection

class exclusivity
    -> existing H4 lowering policy

RefCell conflict
    -> runtime borrow checking

Mutex/RwLock
    -> synchronization

UnsafeCell
    -> unsafe author responsibility
```

## [SIMPL-P7] Compiler sophistication is allowed

Compiler analyses MAY use specialized internal structures, caches, summaries,
and optimized representations when all are derived from authoritative semantic
facts and mutually checked.

The prohibition is against **independent semantic authority**, not against
implementation complexity.

## [SIMPL-P8] The overarching rule

> **Ember should minimize programmer-visible concepts and maintain one authoritative
> representation of every semantic invariant. Compiler analyses may use
> sophisticated, specialized, or cached representations, provided they are
> derived from those invariants, remain mutually consistent, and do not create
> additional language abstractions.**

# 3. Proposal A — Compact Conceptual Vocabulary

## Problem

H4's compiler necessarily tracks more facts than an ordinary programmer should
need to name.

Those facts can be grouped for explanation without being collapsed semantically.

## Proposal

Present the internal model in separate conceptual groups:

```text
PROVENANCE
    provenance root
    source place
    region

STORAGE
    storage identity
    projection

AUTHORITY
    permission
    reference kind
    ownership
    acquisition/check
    safety authority
    synchronization

VALIDITY
    lifetime interval
    escape constraints

INITIALIZATION
    Uninit | Maybe | Init
    field/element/range state

EFFECTS
    EffectSet
```

`INITIALIZATION` and `EFFECTS` are intentionally separate. They are independent
facts and MUST remain independently authoritative.

This grouping is documentation/architecture organization only. The compiler
retains all H4 distinctions needed for soundness.

## Classification

**Architecture/documentation hardening.**

No accepted-program or rejected-program set change.

# 4. Proposal B — `CallInfo` as a Derived Aggregate

## Problem

H4 already has independent compiler facts for:

```text
AccessContract
EffectSet
field-to-region provenance
retention
```

A shared call-level view can reduce duplicated plumbing, but it must not replace
those authorities.

## Proposal

Introduce, if useful to the compiler, an internal derived aggregate:

```text
CallInfo
    access_contract
    effect_set
    provenance
    retention
```

`CallInfo` is a derived aggregate/cache only.

It MUST NOT:

- replace `AccessContract`;
- replace `EffectSet`;
- redefine provenance;
- redefine retention;
- create a new semantic authority;
- enter the runtime ABI;
- become a separately authored source of truth.

The underlying facts remain authoritative.

### Consistency

Whenever an underlying fact changes:

```text
AccessContract
EffectSet
provenance
retention
```

the derived `CallInfo` MUST be regenerated or invalidated.

A stale derived `CallInfo` MUST NEVER be used to accept a program.

### Trust boundary

The underlying trusted facts continue to come only from existing H4-authorized
sources such as verified compiler analysis or audited unsafe/FFI contracts.

## Classification

**Compiler implementation consolidation.**

No source-language semantic change.

# 5. Proposal C — Unified Programmer Model for Initialization

## Problem

The programmer-facing explanation of `MaybeUninit` can be simple, but the compiler
must retain H4's richer dataflow model.

## Resolution

The compiler MUST retain:

```text
Uninit
Maybe
Init
```

plus field, element, and range facts wherever required.

This RFC does NOT replace that model with two states.

### Programmer-facing explanation

Programmers may use the simpler mental model:

> `MaybeUninit[T]` means storage exists, but Ember has not established that a valid
> `T` exists there.

### Compiler-facing model

A shared `InitializationState` is consumed by:

```text
MaybeUninit
Arena
local storage
partial moves
drop elaboration
MIR verification
```

The compiler may refine the state beyond the three lattice values, but all refined
facts must remain consistent with the authoritative dataflow lattice.

## Classification

**Compiler consolidation with semantic preservation.**

# 6. Proposal D — Reuse Existing Collection Interfaces

## Problem

A first simplification proposal introduced `Len` and `Capacity` interfaces.
H4 already has established collection interfaces such as:

```text
Index
IndexMut
Iterator
Iterable
IterableMut
```

Adding public `Len`/`Capacity` interfaces would enlarge the API and require a
separate semantic/API decision.

## Resolution

This RFC does NOT add those public interfaces.

Existing collections SHOULD implement existing interfaces only when their
semantics match exactly.

For example:

```text
ArenaArray
    -> existing indexing/iteration interfaces where compatible

ArenaMap
    -> existing iteration interfaces where compatible
```

Concrete `len()` and `capacity()` methods remain ordinary APIs where appropriate.

No universal collection trait is introduced.

A collection MUST NOT implement an existing interface merely to appear uniform
when its failure, ownership, mutation, iteration, or destruction semantics differ.

## Classification

**Library/documentation guidance.**

No new public API is created by this RFC.

# 7. Proposal E — Generated View Documentation

## Proposal

Every view-producing/consuming API SHOULD expose a standardized documentation
template:

```text
Ownership:
Permission:
Source:
Provenance:
Escape:
Effects:
Allocation:
```

The template SHOULD be generated from authoritative signatures and contracts
where tooling permits it.

The documentation generator MUST NOT maintain a hand-written semantic duplicate
of the API contract.

The public conceptual rule remains:

> **A view is non-owning access to existing storage.**

The underlying H4 rules continue to determine actual ownership, provenance,
effects, lifetime, and escape behavior.

## Classification

**Documentation/tooling hardening.**

# 8. Proposal F — Generic-First API Review

## Rule

Before introducing a new public type, interface, or syntactic form, perform this review:

```text
Can an existing generic parameter express it?
Can an existing interface express it?
Can an existing parameter mode express it?
Can compiler metadata express it without source syntax?
Can an existing concrete API express it without ambiguity?
```

Only if all answers are negative should a new public abstraction be proposed.

## Example

The H4 hashing design remains:

```ember
fn hash[H: Hasher](self, mut h: H)
```

not a family of static/dynamic hasher wrapper types.

H4 deliberately keeps normal hashing statically generic and normally monomorphized.

## Classification

**Design/process hardening.**

---

# 9. Proposal G — Semantic Diagnostics

## Proposal

Primary diagnostics should explain the semantic failure.

Example:

```text
`view.velocity` is borrowed from a source whose lifetime has ended.
```

rather than:

```text
Capability 17 failed region constraint R3.
```

Advanced evidence remains available through existing expert tooling:

```text
ember explain
ember inspect
IDE advanced views
```

Compiler internals may be displayed as secondary information where useful.

## Classification

**Tooling hardening.**

---

# 10. Proposal H — One Unknown-Boundary Principle

## Canonical principle

> **Absence of proof grants no capability; use the domain's least-permissive sound fact.**

This principle unifies the explanation of otherwise similar uncertainty cases.

| Domain | Unknown result |
|---|---|
| borrow overlap | no disjointness proof |
| callable access | all relevant fields |
| foreign retention | all relevant regions |
| dynamic target set | union if bounded; otherwise conservative all-fields/all-regions |
| initialization | `Maybe` / not proven initialized |
| unsafe validity claim | no automatic proof |

This is a conceptual and implementation unification only.

Existing individual H4 rules remain authoritative.

## Classification

**Hardening.**

---

# 11. Proposal I — Complexity Budget

Every future feature proposal SHOULD declare:

```text
Public concepts added:
Syntax added:
Public APIs added:
Compiler facts added:
Compiler passes added:
Runtime mechanisms added:
Existing mechanisms reused:
```

Then classify:

```text
SIMPLIFIES
NEUTRAL
ADDS COMPLEXITY — JUSTIFIED
```

An `ADDS COMPLEXITY — JUSTIFIED` classification must identify why existing mechanisms cannot safely express the requirement.

This review does not itself create a normative language rule.

## Classification

**Process hardening.**

---

# 12. Proposal J — Inference-First Gate

Before requiring a programmer annotation:

```text
1. Attempt compiler inference.
2. Reuse existing metadata.
3. Use the least-permissive sound conservative result.
4. Only then consider explicit source syntax.
```

A new annotation should be justified by a concrete inability of inference to establish the required semantic contract.

This preserves Ember's intended ergonomics:

```text
simple source
    ↓
compiler inference
    ↓
verified MIR
    ↓
native code
```

## Classification

**Process/design hardening.**

---

# 13. Proposal K — Normative Specification vs Implementation Ledger

## Problem

A large specification can become ambiguous if implementation status is embedded as another semantic layer.

## Resolution

The project has one normative semantic source and separate ledgers.

Conceptually:

```text
NORMATIVE SPECIFICATION
        │
        ├── generated rule index
        ├── generated API index
        └── generated diagnostic index

IMPLEMENTATION LEDGER
        │
        ├── implementation status
        ├── pass ownership
        └── evidence

OWNER-DECISION LEDGER
        │
        └── rulings / ODRs

CONFORMANCE LEDGER
        └── executable evidence
```

The normative specification remains the sole authority for semantics.

The implementation ledger records whether those semantics are actually implemented.

The owner ledger records decisions that authorize changes.

The conformance ledger records evidence.

## Historical record

Historical material may remain in the specification for traceability, but it must be explicitly classified as historical/non-authoritative.

Generated indexes should point to the current rule definitions rather than forcing implementers to reconstruct authority from the historical record.

## Classification

**Documentation/tooling/process hardening.**

---

# 14. Proposal L — Simpler Learning Path

The recommended learning order is:

```text
1. values
2. functions
3. ownership
4. borrowing and exclusivity
5. structs
6. classes and ARC
7. containers
8. interfaces and generics
9. views
10. advanced memory / FFI / unsafe
```

Ownership/exclusivity intentionally comes before classes.

An ordinary programmer should not need to understand:

```text
region vectors
CallInfo
storage identities
MIR capabilities
initialization lattice
```

to write normal Ember code.

Those are compiler concepts.

## Classification

**Educational/documentation hardening.**

---

# 15. Additional Simplicity Constraint — No Parallel Safety Systems

A new feature MUST first demonstrate that it cannot reuse:

```text
BorrowCapability
OwnershipGraph
AccessContract
EffectSet
InitializationState
LayoutDescriptor
```

or another already-authoritative H4 compiler fact.

Creating a new safety subsystem is the exception, not the default.

This is the most important compiler-side simplicity rule in this RFC.

---

# 16. Additional Simplicity Constraint — No Semantic Restatement Without Need

An implementation RFC should prefer:

```text
existing rule
    ↓
architecture mapping
```

over copying the same semantic statement into a new rule.

If an existing rule already completely defines a behavior, the simplicity RFC should point to that rule.

This prevents:

```text
Rule A
Rule B says A again
Rule C slightly restates A
```

from becoming multiple sources of truth.

---

# 17. Safety Preservation Requirements

The consolidation MUST preserve at least these invariants.

### Provenance

Every borrow has a provenance root, even when there is no local source place.

### Storage identity

Distinct allocations may share lifetime provenance while remaining distinct storage identities.

### Disjointness

Neither equal nor unequal regions imply overlap/non-overlap.

### Access

Permissions remain distinct from reference representation and ownership.

### Callable contracts

Access, provenance, and effects remain separate authoritative facts even when aggregated into `CallInfo`.

### Initialization

The compiler retains:

```text
Uninit | Maybe | Init
```

and required finer-grained facts.

### Destruction

The existing ordering remains:

```text
ordinary assignment:
    evaluate RHS
    drop old
    store new

Cell.set:
    evaluate/receive new
    store new
    drop old

MaybeUninit.write/write_at:
    evaluate/receive new
    store new
    do not drop previous bytes
```

### Runtime erasure

Compiler-only region/capability/call-contract/initialization metadata must not enter runtime ABI or reload schemas unless an independently existing runtime contract requires the data.

---

# 18. Compatibility

This RFC is a consolidation proposal.

Unless an individual change is separately approved as a language revision, implementation MUST preserve:

```text
accepted-program set
rejected-program set
observable behavior
ownership semantics
borrow semantics
destruction semantics
ABI semantics
diagnostic identity/classification
```

A compiler refactor is not a license to reinterpret the specification.

The current compiler is not a complete semantic oracle for unimplemented H4 behavior.

---

# 19. Conformance and Validation

The implementation of this RFC must be validated through several evidence classes.

## Existing behavior

Where the old implementation already has conformance evidence:

```text
old path vs consolidated path
```

should agree.

## Specified but unimplemented behavior

Use:

```text
positive tests
negative tests
adversarial tests
MIR verifier tests
ABI tests
```

derived from the normative specification.

The specification, not current compiler behavior, is the oracle.

## Simplification-specific tests

The following should be added where applicable:

```text
CallInfo preserves independent AccessContract and EffectSet
InitializationState distinguishes Uninit/Maybe/Init
unknown initialization does not become definite Uninit
same-Arena distinct allocations retain distinct storage identities
unknown overlap does not produce noalias
no new collection interfaces are introduced by this RFC
architecture labels do not enter the normative rule index
```

---

# 20. Adoption Gate

The simplicity consolidation may be applied to a future 0.9.6 hardening revision only when:

1. no existing H4 semantic guarantee is weakened;
2. accepted and rejected program sets remain unchanged;
3. `AccessContract` and `EffectSet` remain authoritative;
4. the H4 initialization lattice remains intact;
5. existing collection interfaces are reused where appropriate;
6. no new `Len`/`Capacity` interface is introduced under this RFC;
7. unknown information follows the least-permissive sound policy;
8. implementation status remains in ledgers rather than a second normative contract;
9. historical references cannot be mistaken for current rules;
10. no new RFC architecture labels become normative rules accidentally;
11. the compiler does not add unnecessary runtime metadata;
12. generated-code behavior remains within established performance budgets;
13. adversarial safety tests pass;
14. specification/index consistency checks pass;
15. all owner decisions required for any semantic change are explicitly recorded.

---

# 21. Versioning

### 21.1 RFC adoption does not itself bump the language

Adopting this RFC as an architecture/process decision does not by itself create
`0.9.6_Hardened_5`.

A hardened revision number advances only when a concrete, approved change is
actually materialized into the specification/tooling/implementation contract
and passes the applicable gates.



The proposals in this RFC are intended to remain within the 0.9.6 hardening line when they only change:

- compiler organization;
- shared internal representations;
- tooling;
- documentation;
- conformance infrastructure;
- library implementation behind unchanged public semantics.

A proposal becomes a language revision when it changes:

- accepted-program set;
- rejected-program set;
- observable source semantics;
- ownership/lifetime behavior;
- public API semantics incompatibly;
- ABI semantics.

A hardening number must never be used to hide a semantic change.

---

# 22. Final Recommendation

Revision 5 intentionally rejects several tempting forms of "simplification":

```text
No:
    two-state initialization compiler model
    new Len/Capacity public interfaces
    replacement of AccessContract/EffectSet
    automatic dynamic borrow fallback
    implementation status as a second specification
    new rule ID for every architectural statement
```

Instead, it adopts:

```text
Yes:
    smaller conceptual grouping
    shared internal aggregation without collapsing semantics
    existing interface reuse
    compiler inference
    conservative unknown handling
    semantic diagnostics
    explicit complexity review
    separate implementation/owner/conformance ledgers
```

The desired end-state remains:

```text
                  SIMPLE EMBER SOURCE
                          │
                          ↓
             compiler-internal sophistication
                          │
          ┌───────────────┼────────────────┐
          ↓               ↓                ↓
      ownership       initialization     calls
          │               │                │
          └───────────────┼────────────────┘
                          ↓
                     MIR verifier
                          ↓
                    native runtime
```

The guiding principle is:

> **Keep the distinctions the compiler needs, hide the bookkeeping the programmer does not need, and reuse existing machinery before creating anything new.**

That is the simplicity direction for the remainder of the Ember 0.9.6 hardening line.


---

## 22.1 Final RFC status

This Revision 5 is the final editorial form of the simplicity architecture/process
RFC. It does not itself create `0.9.6_Hardened_5` or any other language-version
bump. Concrete semantic/specification changes remain subject to the ordinary
owner-decision and hardening process.

# Appendix A — Revision 5 Review Checklist

Before any future revision of this RFC is accepted:

```text
[ ] Current normative authority is stated correctly.
[ ] H4 semantic authorities are not replaced by derived aggregates.
[ ] AccessContract remains authoritative.
[ ] EffectSet remains authoritative and separate from initialization.
[ ] Initialization retains Uninit | Maybe | Init plus required refinements.
[ ] Existing collection interfaces are reused only when semantics match exactly.
[ ] No Len/Capacity interface is introduced by this RFC.
[ ] Documentation is generated from authoritative contracts where possible.
[ ] Diagnostic identities and prescribed repairs remain authoritative.
[ ] Unknown facts grant no capability.
[ ] Existing explicit proof/annotation mechanisms remain available when inference fails.
[ ] Implementation status remains in ledgers.
[ ] Derived caches/indexes never become semantic sources of truth.
[ ] Historical material remains distinguishable from current semantics.
[ ] RFC architecture labels are not accidentally indexed as normative rules.
[ ] Adoption of the RFC alone does not imply a Hardened_N version bump.
```

---

# Appendix B — Final Simplicity Test

For every proposed Ember change, ask:

```text
Does this reduce programmer-visible concepts?
Does this reuse an existing abstraction?
Does this reuse an existing authoritative compiler fact?
Does this reduce duplicated sources of truth?
Can the compiler infer it?
If inference fails, is there an existing explicit proof path?
Does it preserve the accepted/rejected program sets?
Does it preserve runtime/ABI semantics?
Does it preserve diagnostic identity?
Does it preserve all H4 safety invariants?
```

If the answer to the simplicity question is no, the design may still be correct,
but the RFC must explain why the added complexity is justified.

The ideal Ember architecture remains:

```text
                 SIMPLE SOURCE
                      │
                      ↓
            SOPHISTICATED COMPILER
                      │
             shared authoritative facts
                      │
                      ↓
                 VERIFIED MIR
                      │
                      ↓
                  NATIVE CODE
```

> **Keep semantic distinctions that are necessary. Eliminate duplicated mechanisms
> and duplicated sources of truth.**
