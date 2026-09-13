# Ember 0.9.6 — Simplicity Consolidation & Complexity Reduction RFC

**Document type:** Owner/design RFC — Revision 4
**Target:** `Ember v0.9.6_Hardened_1` (design basis only; not yet adopted)
**Immediate frozen predecessor:** `Ember v0.9.5_Hardened_10`
**Status:** Revised design basis; adoption gates not yet passed

---

## 0. Purpose and Decision Boundary

This RFC defines how Ember can recover simplicity after the 0.9.5 hardening line **without weakening any established safety or semantic guarantee**.

The RFC is intentionally **not itself the normative language specification**. It is the design basis for a future `0.9.6_Hardened_1` document. The currently adopted and normative language source remains `docs/spec-source/ember-spec.md`, corresponding to the adopted `0.8.5_Hardened_1` contract. `Ember_v0.9.5_Hardened_10.md` is the frozen development target and immediate construction source for `0.9.6_Hardened_1`. `0.9.6_Hardened_1` becomes normative only after its adoption gates pass.

The central owner principle is:

> **Fewer mechanisms, not fewer guarantees.**

The compiler may become sophisticated internally. The programmer-facing language and the runtime representation should remain comparatively simple.

This RFC therefore consolidates related implementation mechanisms while preserving the accepted/rejected program sets, observable behavior, memory-safety properties, ownership/destruction behavior, ABI behavior, and diagnostic identities established by the frozen predecessor.

---

# Source-of-Truth Hierarchy

The project authority order is:

```text
ADOPTED / NORMATIVE
    docs/spec-source/ember-spec.md
    adopted 0.8.5 lineage

FROZEN DEVELOPMENT TARGET
    Ember_v0.9.5_Hardened_10.md

DESIGN BASIS
    this RFC

PROPOSED TARGET
    Ember_v0.9.6_Hardened_1.md

ADOPTION
    only after all required adoption gates pass
```

No RFC, development target, generated draft, or compiler behavior silently replaces the adopted normative source.

---

# 1. Non-Negotiable Goals

### 1.1 Semantic preservation

The consolidation MUST preserve, unless separately changed by an explicit owner-approved language revision:

- the accepted source-program set;
- the rejected source-program set;
- observable runtime behavior;
- ownership and destruction semantics;
- borrow and region semantics;
- initialization validity;
- unsafe obligations;
- FFI exception/ownership behavior;
- ABI and layout behavior;
- coroutine safety behavior;
- determinism guarantees;
- diagnostic IDs and classifications.

### 1.2 Compiler-owned complexity

Information that can be derived soundly MUST remain implicit in ordinary source code.

This applies especially to:

- borrow provenance;
- region relationships;
- field provenance;
- drop state;
- initialization state;
- callable access information;
- redundant class-exclusivity checks.

### 1.3 Runtime minimalism

Compiler-only metadata MUST NOT become runtime state merely because the compiler internally represents it.

In particular, ordinary runtime values MUST NOT gain hidden fields for:

- lifetime/region IDs;
- borrow capabilities;
- callable access summaries;
- initialization proofs.

### 1.4 Conservative failure

When exact information cannot be proven, the compiler MUST preserve the established conservative behavior rather than inventing precision.

### 1.5 No semantic invention

A compiler convenience, refactoring, or shared internal representation MUST NOT silently create a new source-language rule.

Any accepted-program-set change requires the normal owner-review/versioning process.

---

# 2. The Smaller Ember Conceptual Model

The language should be explainable through a small set of concepts:

```text
Value
Object
Resource
Ownership
Borrow
Region
Access authority
Initialization state
Callable/interface contract
Native/unsafe boundary
```

`Span`, `MutSpan`, `@view`, `Arena`, `Cell`, `RefCell`, `UnsafeCell`, `std.ecs`, C FFI, C++ FFI, and coroutines remain distinct public constructs. They are **applications of these concepts**, not separate ownership/lifetime universes.

The implementation SHOULD therefore reuse the same internal facts whenever two features establish the same kind of semantic relationship.

---

# 3. Canonical Capability Model

The earlier simplicity RFC incorrectly treated permission, ownership, proof method, and synchronization as one access-mode list. Revision 3 explicitly separates these axes.

## 3.1 Canonical internal capability

Every borrowed or access-controlled value MUST be representable through one canonical internal capability with conceptually these fields:

```text
provenance_root
source_place?                 # absent for static/non-place sources
storage_identity
projection_path
region
access_permission
reference_kind
ownership_relation
acquisition_or_check
safety_authority
synchronization_domain
validity_interval
escape_constraints
```

The concrete compiler structure is implementation-defined.

The semantic separation is not.

## 3.2 Provenance root

`provenance_root` answers:

> **Where does the lifetime/provenance originate?**

Examples include:

```text
Array storage
Arena
class field
SoA column
foreign storage
callback-local storage
static storage
```

A provenance root does not have to be a local source place.

A static literal can therefore have a `static` region without having a local `source_place`.

## 3.3 Storage identity

`storage_identity` answers:

> **Which concrete storage object or storage slice is being accessed?**

This is deliberately separate from the provenance root and region.

For example, two allocations:

```text
arena.alloc(A)
arena.alloc(B)
```

may share:

```text
provenance_root = the same Arena
region          = the same Arena lifetime
```

while having:

```text
storage_identity(A) != storage_identity(B)
```

They MUST NOT therefore be considered overlapping merely because they share an Arena region.

## 3.4 Access permission

`access_permission` describes what access is being requested:

```text
shared
mut
```

Raw-pointer representation is expressed by `reference_kind`; a raw pointer may carry either shared/read-only or mutable access permission.

## 3.5 Reference kind

`reference_kind` describes the value/access representation:

```text
reference
raw_pointer
value
view
handle
runtime_guard
```

The exact set may be extended where existing Ember constructs require it, but the implementation MUST NOT overload `access_permission` with representation kind.

## 3.6 Ownership relation

`ownership_relation` describes ownership rather than access permission:

```text
owned
borrowed
observing
```

`owned` is not a borrow mode.

## 3.7 Acquisition/enforcement mechanism

`acquisition_or_check` records how access authority was obtained or validated, conceptually:

```text
static
runtime_checked
lock_acquired
```

This is distinct from safety authority.

## 3.8 Safety authority

`safety_authority` records the trust boundary:

```text
safe
unsafe
```

`unsafe` is **not a proof algorithm**. It means the code has entered an authority boundary where the programmer is responsible for satisfying the applicable unsafe contract.

## 3.9 Synchronization domain

`synchronization_domain` records whether access is additionally governed by synchronization:

```text
none
thread_confined
synchronized
```

A mutex guard can therefore be represented as mutable + runtime-acquired + synchronized without creating a new ownership model.

## 3.10 Examples of composed capabilities

```text
RefCell mutable guard:
    access_permission    = mut
    reference_kind       = runtime_guard
    ownership_relation   = borrowed
    acquisition_or_check = runtime_checked
    safety_authority     = safe
    synchronization      = none

mutable raw pointer:
    access_permission    = mut
    reference_kind       = raw_pointer
    ownership_relation   = borrowed/observing as applicable
    safety_authority     = unsafe

Mutex guard:
    access_permission    = mut
    reference_kind       = runtime_guard
    acquisition_or_check = lock_acquired
    synchronization      = synchronized
    safety_authority     = safe
```

These are combinations of orthogonal facts, not separate language mechanisms.

## 3.11 Projection path

`projection_path` identifies accesses such as:

```text
v.positions
v.transform.rotation
array[i]
```

The same field/projection machinery should be reused by borrow checking, callable access analysis, disjointness, and optimization.

## 3.12 Escape constraints

Escape MUST be represented as a set/predicate rather than a single mutually-exclusive enum.

Relevant constraints include:

```text
may_return
may_store
may_capture
may_publish_ffi
may_survive_statement
may_survive_block
may_cross_yield
```

A single value may satisfy several of these constraints along different control-flow paths.

## 3.13 Validity interval

The capability carries the interval over which its use is valid.

This is the place where region/lifetime reasoning belongs. It must not be merged with storage identity or ownership.

## 3.14 Disjointness

Storage identity plus projection may establish that accesses refer to distinct storage. Region equality alone cannot.

Valid sources of disjointness include existing proven mechanisms such as:

- distinct allocation identities;
- non-overlapping Arena allocation ranges;
- `split_at`-style proofs;
- disjoint fields;
- SoA column disjointness;
- existing proof-carrying disjointness APIs.

Only an actual disjointness proof may justify simultaneous mutable access or `noalias`.

---

# 4. Multi-Region Views

The 0.9.5 multi-region view design remains intact.

A `@view struct` is conceptually an aggregate of ordinary borrow capabilities:

```text
ViewValue
    field A -> capability -> R1
    field B -> capability -> R2
    field C -> capability -> R1
```

There is no separate runtime lifetime subsystem.

## 4.1 One-region compatibility

A one-region view is the degenerate case in which all relevant fields resolve to the same region.

The compiler SHOULD use the same internal implementation path for one-region and multi-region views.

## 4.2 Field projection

Access to `view.field` requires the field's capability and whatever additional source-place constraints the existing borrow rules require.

It MUST NOT unnecessarily require unrelated regions.

## 4.3 Whole-value operations

Operations treating the composite as a whole require all relevant capabilities, including:

- whole-value move;
- whole-value copy;
- unrestricted mutable borrowing;
- whole-value serialization;
- whole-value comparison/hashing;
- calls whose access is unknown or all-fields.

## 4.4 Region inequality

Distinct regions are lifetime facts only. They MUST NOT imply memory disjointness.

---

# 5. Callable Access Analysis

The 0.9.5 callable model is retained, but its implementation is simplified into three certainty levels.

## 5.1 Exact access

When MIR is available, the compiler SHOULD derive field-access and field-to-region provenance directly from MIR.

Ordinary same-module source does not require an explicit summary annotation.

## 5.2 Trusted declared access

A separately compiled or opaque callable MAY carry an access contract only when it comes from:

1. compiler-derived metadata produced from verified MIR; or
2. an existing audited `unsafe`/FFI contract whose obligations explicitly cover the declared accesses and retention behavior.

Safe source code MUST NOT be able to lie about its access summary.

## 5.3 Unknown access

If neither exact nor trusted declared metadata is available, the compiler MUST conservatively assume:

```text
all relevant fields
all relevant regions permitted by the callable's type
```

“Safe” here means safe under the existing conservative H10 restrictions. It is not permission to ignore region validity.

## 5.4 Generics

Monomorphised generic calls SHOULD use exact facts after substitution.

Shared/separately compiled generic calls MUST use trusted declared facts or conservative facts.

## 5.5 Dynamic dispatch

Dynamic/interface dispatch MUST use the union of the access and provenance contracts of **every contractually valid target**, including:

- current targets;
- separately compiled implementations;
- valid hot-reload implementations under the same interface/ABI contract.

If the set cannot be bounded soundly, use the conservative all-fields/all-relevant-regions result.

## 5.6 Summary/provenance invalidation

Both of these are compiler dependencies:

```text
field-access summary
field-to-region provenance summary
```

A change to either MUST trigger rechecking of affected callers and invalidate affected generic/interface/cache dependencies.

Stale or incompatible metadata MUST be a hard compiler error.

These facts are compile-time metadata and MUST NOT enter runtime ABI, serialization, or hot-reload schemas.

---

# 6. Callable Parameter Modes

Callable parameter modes remain a compile-time mode vector rather than a new ownership system.

The supported conceptual callable forms remain:

```text
fn(T) -> R
fn(mut T) -> R
fn(owned T) -> R
```

An omitted mode remains shared/borrowed according to the ordinary parameter rules.

The compiler's canonical callable representation is conceptually:

```text
Callable
    parameters[]:
        type
        mode
    return_type
```

`Callable[Args, R]` MUST preserve these modes through generic bounds, monomorphisation, and borrow checking.

Mode metadata MUST NOT become runtime bookkeeping.

The existing `with_views2_mut`, `with_views3_mut`, and `with_views4_mut` helpers use explicit `mut` callback parameters. The `_mut` suffix identifies the helper family; the `mut` mode expresses mutation authority.

---

# 7. Arena Simplification

Arena remains a normal provenance source rather than a separate lifetime system.

## 7.1 Arena provenance

An Arena-backed result is represented through the same capability model:

```text
provenance_root = Arena
region          = Arena borrow region
storage_identity = concrete allocation/range
```

## 7.2 `@borrows(arena)`

A user function MAY name an `Arena` parameter in `@borrows` only when its returned view is actually derived from storage owned by that Arena.

Example:

```ember
@borrows(arena)
fn make_buffer(arena: Arena, n: usize) -> MutSpan[u8]:
    return arena.alloc_array[u8](n)
```

This annotation records provenance. It does not extend the Arena lifetime or transfer ownership.

Arbitrary non-view, non-Arena parameters remain forbidden as `@borrows` sources.

## 7.3 Allocation families

The Arena API is conceptually divided into:

```text
initialized allocation
uninitialized allocation
```

The public API remains the established H10 API rather than introducing a new family of aliases for every internal distinction.

---

# 8. Unified Initialization Model

The programmer-facing model should be:

```text
initialized storage
uninitialized storage
```

not a large family of unrelated memory states.

## 8.1 `MaybeUninit`

`MaybeUninit[T]` represents storage for `T` without asserting that a valid initialized `T` exists.

It has exactly the size and alignment of `T`.

## 8.2 Safe initialization transition

The canonical safe transition is:

```text
MaybeUninit[T]
    --write(valid T)-->
initialized T storage
```

## 8.3 Unsafe transition

The canonical unchecked transition is:

```text
MaybeUninit[T]
    --unsafe assume_init-->
T
```

The caller is responsible for the initialization precondition.

## 8.4 Span initialization

For uninitialized Arena storage:

```text
MutSpan[MaybeUninit[T]]
```

supports safe per-element `write_at`.

Conversion to:

```text
MutSpan[T]
```

through `assume_init` is unsafe and requires every exposed element to be initialized.

## 8.5 Destruction

`MaybeUninit[T]` never implicitly runs `T`'s destructor merely because its storage has T's layout.

Once an initialized `T` is moved out, ownership transfers and the source slot becomes uninitialized without running a second destructor.

## 8.6 Initialization tracking

The compiler MAY track exact per-element/range initialization for diagnostics and optimization.

The language's soundness MUST NOT depend on one implementation-specific tracking precision.

---

# 9. `Zeroable` and `Default`

`Zeroable` is a validity claim, not a general construction trait.

## 9.1 `Zeroable`

`Zeroable` means exactly:

> An all-zero object representation is a valid initialized representation of `T`.

It does not imply:

- `Copy`;
- `Send`;
- `Sync`;
- FFI safety;
- ownership properties;
- absence of `Drop`.

## 9.2 Automatic derivation

Automatic `Zeroable` derivation is permitted only when the compiler proves the all-zero validity property recursively.

Invalid examples remain rejected where zero is not a valid representation, such as inappropriate references, ranges excluding zero, or invalid enum layouts.

## 9.3 Arena selection

The established `alloc_array[T]` behavior remains:

```text
T: Zeroable -> zero initialization
otherwise T: Default -> construct each element
otherwise -> E2040
```

`Zeroable` takes precedence when both are available.

## 9.4 Drop boundary

Ordinary `Arena.alloc_array[T]` remains `!needs_drop(T)` because Arena reclamation does not individually destroy elements.

---

# 10. Interior Mutability

Interior-mutability abstractions share a conceptual access-authority model but retain different enforcement semantics.

| Abstraction | Mutation authority | Runtime acquisition/enforcement | Synchronization | Unsafe implementation boundary |
|---|---|---|---|---|
| `Cell[T]` | value-only API | none | none | internal |
| `RefCell[T]` | runtime borrow guard | runtime checked | none | internal |
| `Mutex/RwLock` | guard | runtime lock state | synchronized | internal |
| `UnsafeCell[T]` | abstraction author | none | none | explicit unsafe |

The capability model represents these differences through separate axes. The compiler MUST NOT treat them as identical mechanisms.

The established invariants of each type remain authoritative.

---

# 11. ARC and Cycle Analysis

The static cycle analyser and runtime leak checker SHOULD share the same ownership-edge vocabulary and graph representation.

They MUST NOT, however, weaken any existing H10 requirement.

Where H10 requires cycle diagnostics, the requirements remain mandatory, including the required strong-cycle path/edge information and weakening suggestion.

Where H10 requires runtime `--leak-check` reporting, the observable contract remains mandatory.

The consolidation changes internal representation only; it does not introduce automatic cycle collection or silently alter ownership semantics.

The programmer-facing model remains:

```text
strong = owns
weak   = observes without owning
```

---

# 12. Dynamic Class Exclusivity

The optimization ladder applies **only** to H10's dynamic class-exclusivity mechanism.

It does not create a generic runtime borrow checker.

The established semantic distinction remains:

```text
ordinary borrow conflict -> compile-time rejection
class exclusivity        -> H10 static/elided/hoisted/dynamic policy
RefCell conflict         -> runtime borrow check
Mutex/RwLock             -> synchronization
UnsafeCell               -> unsafe obligation
```

For class exclusivity the compiler MAY choose, when H10 proof conditions justify it:

```text
static proof
loop-hoisted runtime check
reused runtime token
per-access runtime check
```

An optimization MUST never turn an unproven ordinary borrow conflict into a runtime acceptance.

---

# 13. Required Replacement and Destruction Ordering

A shared compiler storage-lowering implementation MUST preserve the different semantics already established by Ember.

```text
ordinary assignment:
    evaluate new value
    drop old live value
    store new value

Cell.set:
    evaluate/receive new value
    store new value
    drop old value

MaybeUninit.write / write_at:
    evaluate/receive new value
    store new value
    do not drop previous bytes
```

In particular:

```ember
x = f(x)
```

MUST evaluate the right-hand side while the old `x` remains available, then drop the old live value, then store the new value.

A shared lowering helper MAY implement all three paths only when the applicable ordering contract is explicit.

---

# 14. ECS and DOD

`std.ecs` remains library-owned.

The compiler does not gain a RageV-specific ownership system.

ECS access sets SHOULD be mapped to the same borrow/access capabilities used elsewhere.

`std.ecs` MAY use reflection, monomorphisation, SoA, handles, access sets, and specialization strategies already supported by Ember, but those are library implementations rather than language semantics.

The language's general multi-region view system is the mechanism ECS consumes; ECS does not define that mechanism.

---

# 15. FFI

Foreign views use the same capability model with foreign provenance.

Unknown foreign retention remains conservative.

Foreign byte strings remain byte storage until valid UTF-8 is explicitly established.

## 15.1 C++ exception policy

The existing H10 policies are preserved exactly:

```text
throws = "translate"
    -> catch at the thunk boundary and return Result[T, CppError]

throws = "noexcept"
    -> if an exception nevertheless occurs, terminate according to the existing noexcept policy
```

An exception MUST NOT cross an Ember frame.

This RFC introduces no third exception policy.

---

# 16. Coroutines

Coroutine borrows use the same capability model as ordinary borrows.

A borrow that is not permitted to survive `yield` remains prohibited.

Multi-region inference MUST NOT create a special coroutine exemption.

Data that must persist across suspension is owned by the coroutine frame under the existing coroutine ownership model.

---


# 17. Specification Simplification

## 17.0 Normative rule-index boundary

Architecture mappings in this RFC are non-normative. They MUST NOT become language-rule entries merely because an extractor recognizes bracketed labels. The eventual H1 generator MUST verify:

```text
new RFC-only architecture IDs discovered as normative = 0
duplicate selected rule IDs = 0
dangling normative references = 0
```

Any failure blocks H1 generation.



The specification should reduce conceptual duplication without sacrificing stable rule IDs.

Existing H10/0.9.5 rule IDs remain stable because they are useful for:

- conformance;
- diagnostics;
- implementation tracking;
- history;
- auditability.

However, a stable rule ID is an **address to a requirement**, not necessarily a separate compiler mechanism.

### 17.1 Supersession

When a later revision generalizes or replaces an earlier semantic statement, the old wording MUST be explicitly marked superseded or replaced.

Two contradictory formulations MUST NOT remain simultaneously normative.

### 17.2 Architecture labels

This RFC intentionally does **not** create dozens of new normative rule IDs.

Labels such as:

```text
CAPABILITY MODEL
CALLABLE ARCHITECTURE
INITIALIZATION MODEL
ECS MAPPING
FFI MAPPING
```

are organizational design headings, not entries for the normative rule index.

Only a genuinely new, independently testable implementation-contract invariant should receive a new normative ID when `0.9.6_Hardened_1` is authored.

### 17.3 Examples

Examples illustrate rules. They do not silently override rules.

If an example establishes required normative behavior, the corresponding rule must say so explicitly.

---

# 18. Public API Simplicity

The public surface should remain small and unsurprising.

### Borrowing

```text
Span[T]
MutSpan[T]
```

### Multi-view composition

```text
with_views2
with_views3
with_views4
with_views2_mut
with_views3_mut
with_views4_mut
```

### Initialization

```text
MaybeUninit.uninit
MaybeUninit.write
MaybeUninit.assume_init
MutSpan[MaybeUninit[T]].write_at
MutSpan[MaybeUninit[T]].assume_init
```

### Arena

```text
alloc
alloc_array
alloc_uninit
alloc_nodrop
reset
scope
```

The compiler MUST NOT expose an internal helper merely because two internal analysis cases happen to differ.

---

# 19. Diagnostics

Normal diagnostics should explain the semantic cause first.

For example:

```text
`v.velocities` is borrowed from a source whose lifetime has ended.
```

Expert tooling such as `ember explain` MAY expose:

```text
region
storage identity
projection
capability
access summary
MIR path
```

The internal representation is therefore available without forcing compiler implementation terminology into every compiler error.

Stable diagnostic IDs and safety classifications MUST remain stable unless a separately approved diagnostic revision changes them.

---

# 20. Compiler Architecture

The consolidated compiler should use a clear distinction between analysis MIR and codegen-ready MIR.

```text
source
  ↓
parser
  ↓
HIR
  ↓
type/interface solving
  ↓
initial MIR lowering
  ↓
callable/provenance/access-contract computation
  ↓
initialization + ownership + borrow + region analysis
  ↓
drop elaboration
  ↓
MIR verification
  ↓
optimization
  ↓
MIR verification
  ↓
backend
```

The exact implementation may interleave some steps, but the semantic boundary remains:

> **Initial MIR is an analysis substrate. Verified MIR is the code-generation boundary.**

The compiler SHOULD centralize shared semantic facts in reusable representations:

```text
TypeIdentity
BorrowCapability
OwnershipGraph
AccessContract
InitializationState
LayoutDescriptor
EffectSet
```

Feature modules consume these facts rather than independently re-solving the same problem.

Examples:

```text
Arena      -> BorrowCapability + InitializationState
ECS        -> BorrowCapability + AccessSet + LayoutDescriptor
RefCell    -> BorrowCapability + runtime borrow state
FFI        -> BorrowCapability + LayoutDescriptor + foreign contract
Coroutine  -> BorrowCapability + frame validity
Hot reload -> TypeIdentity + LayoutDescriptor
```

---

# 21. Safety Proof of the Consolidated Architecture

The simplification is acceptable only if these invariants remain demonstrably true.

### A. Provenance
Every borrow has an identifiable provenance root, which need not be a local source place.

### B. Region validity
A borrow may be used only while its region remains valid.

### C. Storage identity
Region equality does not imply storage overlap. Disjointness requires an independent proof.

### D. Access authority
Shared access cannot become exclusive without an existing sanctioned transition.

### E. Initialization
Storage layout does not imply initialization validity.

### F. Ownership
Borrowing never silently becomes ownership.

### G. FFI
Foreign code cannot extend or weaken a borrow without a verified contract.

### H. Coroutine
A forbidden borrow cannot survive suspension merely because it is stored in a multi-region value.

### I. Runtime erasure
Compiler-only metadata does not enter runtime representation, ABI, serialization, or hot-reload schemas.

### J. Destruction ordering
Shared implementation machinery never erases the distinct ordering contracts of ordinary assignment, `Cell.set`, and `MaybeUninit.write`.

---

# 22. Compatibility Contract

The consolidation is allowed only if it preserves the complete observable contract:

```text
accepted programs
rejected programs
runtime behavior
memory safety
ownership/destruction
ABI/layout
FFI behavior
diagnostic identity
```

Any intentional exception is a separate owner-approved language or diagnostic revision and MUST NOT be smuggled into a “simplicity” change.

---

# 23. Differential Validation

The incomplete current compiler is not a complete semantic oracle.

Validation MUST classify tests into three evidence classes.

### 23.1 Existing conformance-backed behavior

Where the old implementation has conformance evidence, compare old and consolidated implementation paths directly.

### 23.2 Specified but not yet implemented behavior

Where the compiler does not yet implement the complete H10/0.9.5 contract, derive positive, negative, adversarial, and verifier tests from the specification and owner decisions.

### 23.3 Divergent current behavior

When current compiler behavior and the specification differ, classify against the normative specification, not against the current compiler.

Every difference must become one of:

```text
implementation defect
specification defect
owner decision
known unimplemented area
```

A green current test does not override the specification.

---

# 24. Conformance Requirements for the Consolidation

The eventual implementation of this RFC MUST demonstrate:

1. equivalent borrow safety for equivalent capability relationships;
2. exact/declared/unknown callable access behavior;
3. trusted-contract enforcement;
4. invalidation of both access and provenance summaries;
5. initialization safety for `MaybeUninit`;
6. `Zeroable` validity checking;
7. Arena provenance preservation;
8. preservation of all H10 cycle diagnostics;
9. preservation of H10 class-exclusivity behavior;
10. all three replacement/destruction orderings;
11. no runtime region/lifetime objects;
12. no runtime callable-summary fields;
13. no compiler-only initialization tags in ordinary runtime objects;
14. preserved FFI exception semantics;
15. preserved coroutine restrictions;
16. preserved ECS library ownership;
17. complete stable diagnostic ID mapping.

The conformance suite MUST retain all previous safety regressions even when the underlying implementation path changes.

---

# 25. Performance Contract

Performance requirements should use the established measurable compiler/runtime budgets rather than vague asymptotic claims.

The implementation SHOULD:

- reuse analysis results rather than recomputing the same facts independently;
- avoid runtime dispatch for statically resolved callable access;
- erase region/capability metadata before runtime code generation;
- preserve Arena `alloc_uninit` as ordinary allocation plus its defined bounds/borrow checks;
- retain H10 exclusivity optimizations only when their proofs succeed.

Any quantitative compile-time constraint is governed by the existing compiler-budget rules. This RFC does not create a second unspecified “quadratic” budget.

---

# 26. H10 `[ARN-10]` Owner Resolution — APPROVAL REQUIRED

The inherited H10 contract contains an ambiguity: it describes Arena cursor rollback if `T.default()` “fails or panics”, while `Default.default()` returns `Self` and v1 panic behavior aborts rather than unwinds.

The following ruling is proposed for the 0.9.6 design and remains pending explicit owner approval:

> **In v1, `T.default()` has no recoverable construction-failure path. A panic terminates according to `[PAN-1]`; no post-panic Arena state is observable, so `[ARN-10]` does not require panic recovery or unwinding. Cursor rollback applies only to separately specified recoverable failures. Any future fallible construction or unwinding model must define its own rollback behavior.**

Therefore:

- the compiler MUST NOT invent panic unwinding to satisfy `[ARN-10]`;
- transactional rollback may apply to an actually recoverable construction failure if a separate API specifies one;
- future fallible construction/unwinding is a separate language/library design decision.

This is an explicit owner ruling, not an implementation workaround.

When `0.9.6_Hardened_1` is generated, it MUST:
1. amend `[ARN-10]` with this ruling;
2. record the ruling in the change history and owner-decision ledger;
3. add conformance coverage for the abort-only panic behavior and absence of required panic recovery;
4. contain no implementation that introduces panic unwinding solely to satisfy `[ARN-10]`.

---

# 27. H1 Generation Rules

When generating `Ember_v0.9.6_Hardened_1.md` from this RFC:

1. Start from frozen `Ember_v0.9.5_Hardened_10.md`.
2. Verify that every accepted 0.9/0.9.5 decision is already present exactly once in `Ember_v0.9.5_Hardened_10.md`. Do not replay amendments or decisions already materialized in the frozen predecessor.
3. Apply this consolidation only where it preserves the accepted/rejected program sets and existing semantics.
4. Preserve all still-valid inherited normative rules.
5. Explicitly mark superseded formulations; never leave contradictory old/new formulations simultaneously normative.
6. Do not copy this RFC's architecture headings into the normative rule index merely because they are structured labels.
7. Adopt a new normative rule ID only when it represents an independent testable invariant with a clear owner-approved reason and conformance mapping.
8. Keep implementation evidence separate from specification requirements.
9. Never claim implementation, verification, or conformance unless repository evidence exists.
10. Run the complete rule-index, specification-consistency, grammar, diagnostic, and conformance gates before adoption.
11. `0.9.5_Hardened_10` remains the active frozen development target and construction source while H1 is prepared and validated.
12. After `0.9.6_Hardened_1` passes its adoption gates, H1 becomes the normative specification target and `0.9.5_Hardened_10` remains its historical immediate predecessor.

---

# 28. Explicit Non-Goals

This RFC does not:

- add tracing garbage collection;
- remove `Weak`;
- remove borrow checking;
- expose Rust-style named lifetime syntax;
- introduce runtime region objects;
- make views owning;
- allow arbitrary non-view `@borrows` parameters;
- turn `Span`/`MutSpan` into separate ownership systems;
- make C++ exceptions cross Ember frames;
- make foreign strings implicitly valid UTF-8;
- weaken coroutine lifetime rules;
- make ECS compiler magic;
- make uninitialized bytes implicitly valid values;
- turn ordinary borrow errors into runtime checks;
- collapse `Cell`, `RefCell`, synchronization, and `UnsafeCell` into one identical mechanism.

---


# 29. Owner Position

The intended end state is:

```text
                 EMBER
                   │
       ┌───────────┼───────────┐
       ↓           ↓           ↓
     VALUE       OBJECT      RESOURCE
       │           │           │
       └───────────┼───────────┘
                   ↓
            shared compiler facts
                   ↓
          strong compile-time proof
                   ↓
               native code
```

The programmer should see the smallest useful abstraction.

The compiler should carry the detailed proof.

The runtime should carry as little bookkeeping as possible.

The design target remains:

> **C-like speed, Python-like syntax, Rust-like memory safety — with the compiler absorbing complexity that can be inferred safely.**

The simplicity pass is therefore successful only if it makes Ember **easier to understand without making it easier to break**.

