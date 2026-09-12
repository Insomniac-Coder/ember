# Ember Programming Language — Design & Implementation Specification

**Version:** 0.9.5_Hardened_9
**Status:** Frozen development target; implementation/conformance evidence is not claimed and normative repository adoption still requires the recorded adoption gates
**Immediate predecessor:** 0.9.5_Hardened_8
**Versioning model:** A hardening increments `Hardened_N` when it only adds implementation detail,
invariants, tooling requirements, or editorial clarification without changing accepted language
semantics. A language revision is required when the accepted-program set or another normative
language semantic changes; implementation completion by itself does not move the language version.
The 0.9.5 language revision
introduced inferred multi-region view structs; this revision is a subsequent hardening that
consolidates the inherited 0.8.5 contract, makes supersession explicit, repairs mechanical
extraction defects, recovers the owner-supplied Hardened 14 callback-composition rules, resolves
their mutable API family, callable-type mode syntax, helper input modes, `Callable` bridge, and the
narrow Arena-backed return-provenance boundary by owner decision, and records evidence status honestly. No implementation or conformance result is
asserted merely because the specification requires one.

> **Authority boundary:** This frozen hardening records the owner-selected 0.9.5 development target, but
> `docs/spec-source/ember-spec.md` remains the repository's sole normative source until this
> target passes the adoption gates and is explicitly installed. The immutable received file
> is preserved under `docs/spec-source/as-received/`.

> **SOURCE RECOVERY AND CALLABLE-MODE BOUNDARY COMPLETED:** on 2026-09-12 the owner supplied the missing
> Hardened 14 definitions (H5), selected explicit all-mutable `_mut` variants (H6), and then extended
> callable types with borrowed/default, `mut`, and `owned` parameter modes (H7). The owner then ruled
> that mutable helper inputs are `mut` reborrows and that the existing `Callable[Args, R]` abstraction
> preserves the full mode signature as compiler-known type metadata (H8). H4 through H8 remain
> frozen predecessor records. This revision consistently uses Ember's canonical type name
> `MutSpan[T]`; the rulings' `SpanMut[T]` spelling was normalized to the type already defined
> throughout the language. ODR-004 through ODR-008 are closed. No helper consumes its input view,
> callable modes are compile-time-only, and neither ruling creates a separate ownership system.

> **ARENA RETURN PROVENANCE COMPLETED:** on 2026-09-12 the owner resolved the boundary between
> `[LT-1a]` and `[LT-4]`. A user-defined wrapper may name an `Arena` parameter in `@borrows` only
> when its returned view derives from storage owned by that arena. H8 remains frozen; this H9 adds
> `[LT-4a]`, `[LT-4b]`, and `[TST-22]`. The exception neither makes `Arena` a view type nor permits
> arbitrary non-view parameters in `@borrows`.

# 0.9_Hardened_12 Owner-Decision Resolution Cycle (historical record)

**Revision:** `0.9_Hardened_13`
**Predecessor:** `0.9_Hardened_12`
**Classification:** hardening; no new source-language construct is introduced.

This cycle explicitly exercises the owner-decision authority requested for the Ember project. The design direction is treated as established: Ember is a safe-by-default systems language optimized for C/C++ migration, game-engine workloads, predictable cost, value-oriented data processing, explicit native islands, deterministic destruction, and first-class hot reload. Where an earlier question was deliberately deferred to measurement but the surrounding architecture already makes one option clearly superior, this revision records that choice rather than leaving the implementation permanently provisional.

## OQ-8 Resolution — RageV ECS architecture

`OQ-8` is resolved in favor of **shape (a): a library-level ECS built over Ember's existing `SoA`, `Pool`, `Handle`, access-set, borrow and comptime-reflection primitives**. Ember does **not** acquire a compiler-integrated ECS.

### Rationale

This is the architectural choice that best matches Ember's core philosophy:

- ECS is a data structure / library domain, not a new ownership category.
- `SoA`, `Query`, `Pool`, access sets and borrow checking already provide the necessary semantics.
- Compiler integration would create a privileged subsystem and violate the principle that the programmer's type determines the storage/lifetime model.
- A library implementation keeps the language useful outside games and avoids baking RageV's ECS representation into Ember's core ABI.
- Comptime reflection can generate efficient query/storage code without making `World`, `Query`, or `Component` compiler magic.
- RageV can retain its existing sparse-set layout and migrate orchestration incrementally.

### [OQ8-1] ECS is library-owned

`std.ecs` MUST implement ECS semantics as ordinary Ember library code over the language's existing containers, views, handles, comptime reflection and synchronization primitives. The compiler MUST NOT contain ECS-specific semantic rules.

### [OQ8-2] RageV-compatible sparse-set layout

The reference `std.ecs` implementation MUST support the RageV-compatible representation:

```text
Entity = 20-bit index | 12-bit generation
dense   = SoA[T] or Array[T]
entities = Array[Entity]
sparse  = Array[u32]
```

The exact storage container remains selected by `@component(layout=soa|aos)`, with AoS as the default where the existing specification requires it.

### [OQ8-3] Stable query iteration

A `Query` MUST expose deterministic iteration order for a fixed world state. Adding unrelated component types MUST NOT silently reorder an existing query. Mutation that changes membership MAY change subsequent iteration order according to the documented sparse-set operation, but a single query traversal has one stable order.

### [OQ8-4] Access-set enforcement

`Query` access declarations MUST be represented as ordinary borrow/access facts. The ECS runtime MUST NOT provide a hidden fourth aliasing mechanism. Read/write conflicts are rejected or synchronized using the same Ember ownership/concurrency model as non-ECS code.

### [OQ8-5] Comptime specialization without language specialization

`std.ecs` MAY use comptime reflection and monomorphisation to generate specialized query/storage code. This is not language-level specialization and MUST NOT introduce overlapping implementation precedence or a second coherence model.

### [OQ8-6] FFI interoperability

The ECS API MUST provide a zero-copy interoperability path for RageV-compatible component storage where the underlying layout satisfies the canonical `Layout[T]` contract. Entity IDs MUST cross the boundary as their canonical integer representation.

### [OQ8-7] No ECS compiler magic

A conforming implementation that does not ship `std.ecs` remains a valid Ember implementation, provided its declared conformance profile does not claim the standard ECS library. The language itself remains ECS-agnostic.

## Owner-decision closure policy

The following are now closed and MUST NOT be reopened without new evidence that demonstrates a contradiction with an existing normative rule or a concrete failure of the selected architecture:

| Decision | Resolution | Status |
|---|---|---|
| `OQ-8` | ECS shape (a): library over `SoA`/`Pool`/access sets/comptime reflection | **Closed** |
| `OQ-1..OQ-7` | Existing ADR decisions retained | **Closed** |
| `OQ-9..OQ-27` | Existing decisions retained | **Closed** |
| `OQ-28..OQ-32` | Closed/moot as recorded | **Closed** |

An implementation agent MUST NOT convert a closed decision back into an open implementation choice merely because another architecture appears locally convenient. A new proposal requires an explicit ADR and evidence.

## Cycle acceptance

This cycle is accepted when:

1. `OQ-8` is marked `decided` rather than `deferred`;
2. `ADR-008` records the selected library-level ECS architecture;
3. `[BEN-7]` is changed from “decides ADR-008” to “validates the already-selected architecture”;
4. XXII.6 no longer presents two competing ECS architectures;
5. RageV migration guidance uses `std.ecs` rather than compiler-integrated ECS semantics;
6. no language compiler pass gains an ECS-specific dependency.

For Hardened 14, acceptance additionally requires:

7. `[WK-5]`–`[WK-10]` are specified as diagnostic-only ownership-cycle analysis and do not alter ARC semantics;
8. `[EXC-8]`–`[EXC-14]` are specified and generated-code tests MUST demonstrate safe loop hoisting without weakening checked semantics;
9. `[LT-8]`–`[LT-13]` define the recovered `std.borrow.with_views2/3/4` helper surface; their Hardened 14 single-region context does not restore the former single-region `@view struct` restriction, which `0.9.5_Hardened_1` independently supersedes;
10. `[TST-14]`–`[TST-16]` and `[BUD-7]` are present in the conformance/performance plan;
11. the current revision's immediate predecessor is `0.9_Hardened_13`.

---

## 0.9 Implementation-Cycle Amendment

This revision is the result of a specification audit and consolidation pass over `Ember 0.8.5_Hardened_1` and the subsequent 0.9/0.9.5 hardening lineage. The process was intentionally iterative:

1. implement against the specification;
2. record every inconsistency, missing invariant, or implementation blocker;
3. classify it as semantic, implementation, ABI, runtime, tooling, documentation, or RageV integration;
4. resolve implementation-detail gaps without inventing language semantics;
5. when an existing language feature required completion or material semantic improvement, increment the language version and reset the hardening counter;
6. re-audit the complete resulting specification;
7. repeat until additional passes produce no material implementation-readiness improvements.

The resulting lineage is:

```text
0.8.5_Hardened_1
    ↓ hardening
0.8.5_Hardened_2
    ↓ hardening
0.8.5_Hardened_3
    ↓ hardening
0.8.5_Hardened_4
    ↓ feature completion
0.9_Hardened_1
    ↓ hardening
0.9_Hardened_2
    ↓ hardening
0.9_Hardened_3
    ↓ hardening
0.9_Hardened_4
    ↓ hardening
0.9_Hardened_5
    ↓ hardening
0.9_Hardened_6
    ↓ hardening
0.9_Hardened_7
    ↓ hardening
0.9_Hardened_8
    ↓ hardening
0.9_Hardened_9
    ↓ hardening
0.9_Hardened_10
    ↓ hardening
0.9_Hardened_11
    ↓ hardening
0.9_Hardened_12
    ↓ hardening
0.9_Hardened_13
    ↓ hardening
0.9_Hardened_14
    ↓ feature revision
0.9.5_Hardened_1
    ↓ hardening
0.9.5_Hardened_2
```

### Cycle classes

- **Hardenings:** implementation completeness, compiler invariants, ABI precision, runtime memory ordering, deterministic execution, tooling, and conformance.
- **Language revision:** completion/conformance of `UnsafeCell[T]` and `RefCell[T]`, which were already owner-approved in 0.8.5 but explicitly recorded there as unbuilt.
- **No silent owner decisions:** if implementation requires a choice about language meaning that is not already determined by this specification, the implementation MUST stop at that point, record an `OWNER DECISION REQUIRED` entry, and MUST NOT manufacture semantics by analogy.
- **0.9.5 feature boundary:** inferred multi-region views are an explicit owner-approved source-language extension; implementation agents MUST treat `[LT-14]`–`[LT-34]` as normative and MUST NOT replace them with explicit lifetime syntax or a runtime region mechanism.

---

# 0.9_Hardened_14 Hardening Cycle — Ergonomic and Hot-Path Hardening (predecessor)

This cycle addresses three weaknesses identified in the 0.9_Hardened_13 design review:

1. accidental ARC reference cycles;
2. repeated dynamic class-exclusivity checks in hot loops;
3. ergonomic composition of views with independent source lifetimes.

This cycle was a **hardening-only predecessor**. It did not add a tracing collector, change ARC ownership,
permit multiple independent regions inside a storable `@view struct`, or weaken class
exclusivity. Its `std.borrow.with_views2/3/4` helpers remain valid and useful, and their recovered
`[LT-8]`–`[LT-13]` definitions are reproduced in §H14.3.
The multi-view composition limitation they worked around is independently superseded by the
source-language feature in `0.9.5_Hardened_1`. The ARC and exclusivity hardenings remain normative
unchanged.

## H14.1 ARC-cycle prevention and diagnosis

The existing `Weak` model remains normative: strong reference cycles are still not collected.
The goal of this cycle is to move cycle detection earlier and make the diagnostic actionable.

* `[WK-5]` **Whole-program-visible ownership-cycle analysis.** The compiler MUST construct a
  directed ownership graph for class fields whose declared type is a class handle, `Shared[T]`,
  or a container whose element/value ownership is statically known to retain a class handle.
  `Weak[T]`, raw pointers, `Span`, `MutSpan`, and foreign opaque handles are non-owning edges and
  MUST NOT participate as strong edges. The graph is computed per package and, when LTO is enabled,
  may include imported package summaries. This analysis is diagnostic only and MUST NOT alter
  program acceptance or object lifetime semantics.

* `[WK-6]` **Strong-SCC warning.** If the ownership graph contains a strongly connected component
  containing only strong edges, the compiler MUST emit `L3001` at the field declaration that closes
  the shortest statically visible cycle. The diagnostic MUST identify the complete cycle, distinguish
  strong from weak/non-owning edges, and name the first edge whose type can be changed to `Weak[T]`
  without changing the ownership direction of the other edges. The lint remains a warning by default
  and follows the package's ordinary lint policy, so projects MAY promote it to an error.

* `[WK-7]` **Cycle analysis is conservative.** A possible cycle is sufficient for `L3001`; the compiler
  MUST NOT claim that a runtime cycle definitely exists unless runtime evidence establishes one.
  Generic instantiations MUST be analysed after substitution where concrete ownership is known. An
  opaque foreign edge MUST be reported as `unknown` rather than assumed weak or strong.

* `[WK-8]` **Runtime cycle report is retained and strengthened.** `ember run --leak-check` MUST report
  runtime strongly connected components using the same edge vocabulary as `[WK-5]`. For each SCC it
  MUST show: object identities in debug mode, class/field edge names, strong-edge count, whether the
  SCC was statically predicted, and a suggested weak edge when one exists. Runtime discovery remains
  diagnostic only; no collector or automatic mutation of the object graph is permitted.

* `[WK-9]` **Cycle explanation command.** `ember explain --cycle <Class[.field]>` MUST print the
  statically visible ownership path(s) that make the field cycle-prone, including the exact field
  declarations responsible. When the cycle crosses a generic container, the instantiated ownership
  type MUST be shown. When no static cycle can be established, the command MUST say that the edge
  is dynamically cycle-capable rather than pretending it has proved a cycle.

* `[WK-10]` **IDE/fix-it guidance.** Where a cycle-closing field has a valid `Weak[T]` replacement,
  the diagnostic MUST provide a machine-applicable suggestion equivalent to replacing the field's
  strong class handle with `Weak[T]`. The fix MUST NOT be offered when doing so would change a
  declared ownership contract that the compiler cannot establish as non-owning. No automatic fix
  may rewrite `Shared[T]` or foreign ownership types.

These rules do not make cycles impossible. They make the common accidental-cycle failure mode
visible at declaration time, at static analysis time, and at runtime.

## H14.2 Dynamic exclusivity check hoisting

The semantic law of exclusivity is unchanged. The hardening targets the performance criticism that
an otherwise safe class-heavy loop could execute an uncontended header check on every iteration.

* `[EXC-8]` **Loop-invariant exclusivity regions.** When a loop contains repeated long-term accesses
  to the same class object and the compiler proves that the object identity, access kind, and
  conflict set are invariant for the loop, the compiler MUST represent the access as one loop-level
  exclusivity region rather than one begin/end pair per iteration. The runtime check occurs in the
  loop preheader; the loop body executes under an internal access token. This is a semantics-preserving
  lowering of `[EXC-1]`/`[EXC-2]`, not a new aliasing mechanism.

* `[EXC-9]` **Safe hoisting conditions.** `[EXC-8]` is legal only when all of the following hold:
  (a) the receiver identity is loop-invariant;
  (b) the long-term access does not escape the loop body;
  (c) no operation in the loop can replace, publish, or otherwise invalidate the receiver identity;
  (d) no call/virtual/interface dispatch in the protected interval can begin an unmodelled conflicting
  access; or the compiler has an `[OPT-1]`/effect proof that it cannot do so; and
  (e) the access's start and end are equivalent to the original NLL boundaries. If any condition is
  unknown, the implementation MUST retain the ordinary dynamic checks.

* `[EXC-10]` **Nested-loop reuse.** For nested loops, an outer exclusivity token MAY be reused by an
  inner loop when the inner access is a subset of the outer region and `[BRW-*]` permits the nested
  access. A second runtime check MUST NOT be emitted merely because the source contains a nested
  loop. A conflicting nested access still produces the normal static error or runtime panic.

* `[EXC-11]` **Token is compiler-internal.** The exclusivity token used by `[EXC-8]` MUST NOT be
  representable as an Ember value, stored by user code, passed through FFI, or observed through
  reflection. It is a MIR/code-generation construct and does not create a fourth aliasing mechanism.

* `[EXC-12]` **Observable reporting.** `ember inspect --safety` MUST distinguish `DYNAMIC_PER_ACCESS`
  from `DYNAMIC_HOISTED_LOOP`, naming the loop and the proof that justified hoisting. The safety side
  table MUST record one runtime check at the preheader and the protected access interval, rather than
  fabricating one entry for every iteration.

* `[EXC-13]` **Performance conformance gate.** The conformance/performance suite MUST contain class-
  handle loops with: (1) one stable receiver; (2) a receiver loaded from a loop-invariant local;
  (3) an escaping receiver that defeats hoisting; (4) a virtual call that defeats the proof; and
  (5) two distinct receivers that must not be merged. The generated code MUST show one check for
  cases (1) and (2), and per-access checking for cases (3)–(5). The benchmark report MUST publish
  emitted-check counts and runtime measurements; no fixed nanosecond figure is made normative.

* `[EXC-14]` **No shipping-only semantic bypass is introduced.** `[PRF-1]`'s existing shipping
  exception for `exclusivity = "unchecked"` remains the only profile-controlled removal of class
  exclusivity checks. `[EXC-8]`–`[EXC-13]` are optimisations of checked semantics and apply independently
  of that setting.

The intended hot-loop lowering is therefore:

```text
before:
    loop:
        begin_access(object)
        body
        end_access(object)

Hardened 14:
    begin_access(object)       # one check in the preheader
    loop:
        body                    # protected by compiler-internal token
    end_access(object)         # one release in the postheader
```

The second form is permitted only when `[EXC-9]` proves equivalence. It does not turn a dynamic
property into an assumption.

## H14.3 Independent-lifetime view composition — predecessor mechanism

The owner supplied the following Hardened 14 definitions on 2026-09-12. They remain normative
language/library contracts in this revision.

* `[LT-8]` **Canonical `with_views` helpers.** `std.borrow` MUST provide canonical,
  allocation-free helpers for two, three and four borrowed views, with both shared and mutable
  variants. The helper inputs themselves use the ordinary Ember parameter-mode rules: shared
  helpers receive their `Span[T]` inputs as shared (`borrowed`) parameters; mutable helpers receive
  their `MutSpan[T]` inputs as `mut` parameters; and neither family consumes (`owned`) an input view.
  The callback receives the corresponding view values using the callable parameter modes shown
  below. The normative signatures are structurally:

  ```text
  with_views2[A, B, R](a: Span[A], b: Span[B],
                       f: fn(Span[A], Span[B]) -> R) -> R
  with_views3[A, B, C, R](a: Span[A], b: Span[B], c: Span[C],
                          f: fn(Span[A], Span[B], Span[C]) -> R) -> R
  with_views4[A, B, C, D, R](a: Span[A], b: Span[B], c: Span[C], d: Span[D],
                             f: fn(Span[A], Span[B], Span[C], Span[D]) -> R) -> R

  with_views2_mut[A, B, R](mut a: MutSpan[A], mut b: MutSpan[B],
                           f: fn(mut MutSpan[A], mut MutSpan[B]) -> R) -> R
  with_views3_mut[A, B, C, R](mut a: MutSpan[A], mut b: MutSpan[B],
                              mut c: MutSpan[C],
                              f: fn(mut MutSpan[A], mut MutSpan[B],
                                    mut MutSpan[C]) -> R) -> R
  with_views4_mut[A, B, C, D, R](mut a: MutSpan[A], mut b: MutSpan[B],
                                 mut c: MutSpan[C], mut d: MutSpan[D],
                                 f: fn(mut MutSpan[A], mut MutSpan[B],
                                       mut MutSpan[C], mut MutSpan[D]) -> R) -> R
  ```

  The exact public spelling MAY use the module's overload convention, but the arity, mutability, and
  type relationships MUST remain these. Their callback boundary uses `[LT-7]` late-bound regions,
  so each supplied view receives its own invocation-local region. The callback MUST complete before
  those regions end, and the helpers MUST return only values that do not contain those borrowed
  regions. The helpers are ordinary library APIs; they are not compiler-special ownership types.

* `[LT-8a]` **Callbacks retain callable modes.** The callback parameter of each `with_views` helper
  MUST use `[FN-6a]`'s canonical callable representation. Shared helpers require corresponding
  shared (`borrowed`) callback parameters; `_mut` helpers require corresponding `mut` callback
  parameters. The compiler MUST reject an incompatible callable and MUST NOT erase callback modes
  merely because the callable passes through `Callable[Args, R]`.

* `[LT-9]` **Independent regions are preserved.** A `with_views` invocation MUST NOT force all
  input views into the same region merely because they are passed to one callback. Each callback
  argument retains the region of its corresponding source for the duration of the invocation.
  This is an application of `[LT-7]`, not an extension of `[LT-2]`.

* `[LT-10]` **No escape.** A callback passed to `with_views` MUST NOT return, store, capture into an
  `owned fn`, or otherwise publish a value whose type contains one of the invocation-local view
  regions. Such an escape is diagnosed with the existing view/region diagnostics. An owned result
  is permitted. The helper does not acquire ownership of the underlying storage: passing a view to
  either helper family creates or uses only the borrow represented by that view. A helper MUST NOT
  extend the source storage lifetime beyond the invocation or convert a borrowed view into an owned
  value.

* `[LT-11]` **Mutability follows ordinary borrowing.** `with_views` does not grant overlapping
  mutable access. The `_mut` variants accept `MutSpan[T]` and their callbacks explicitly declare
  the corresponding parameters as `mut`; therefore they remain subject to the ordinary exclusive-
  borrow and mutable-place rules. Mutable inputs whose sources may alias are rejected by the
  ordinary borrow checker; structurally disjoint inputs are expressible through `split_at_mut`,
  `columns_mut`, `SoA`, or an established disjointness fact.

* `[LT-11a]` The compiler MUST reject a mutable `with_views*_mut` callback whose callable type omits
  the required `mut` parameter mode when the callback body requires mutable access. The diagnostic
  MUST identify the callback parameter-mode mismatch rather than treating `MutSpan[T]` as an
  implicit ownership mode.

* `[TST-20]` Conformance MUST cover:

  1. shared `with_views2/3/4` callbacks;
  2. mutable `with_views2/3/4_mut` callbacks using `mut` parameters;
  3. rejection of aliased mutable inputs;
  4. rejection of callbacks whose parameter modes do not permit the required mutation;
  5. preservation of ordinary `mut`/`owned` parameter semantics inside callable types.

  This resolution supersedes any wording that treats `_mut` as implicitly encoding parameter
  ownership or as requiring a special ownership rule for `MutSpan[T]`. `_mut` identifies the API
  family; `mut` in the callable type expresses the actual parameter mode.

* `[TST-21]` Conformance MUST additionally cover:

  1. shared helper inputs using the default shared parameter mode;
  2. mutable helper inputs using `mut`;
  3. rejection of an `owned` helper-input contract and preservation of caller usability after the
     helper returns;
  4. preservation of `mut` callback parameters through `Callable[Args, R]`;
  5. rejection when `Callable[Args, R]` would erase or mismatch a required parameter mode;
  6. generic forwarding of a mode-bearing callable without loss of its borrow or mutability
     semantics.

  This closes the two formerly deferred boundaries: helper input modes follow ordinary Ember
  parameter rules, and `Callable[Args, R]` preserves modes as part of the canonical callable type
  rather than erasing them. The supplied mnemonic `[TST-LT-MODE]` is normalized to this next unused
  numeric test-rule ID so the normative rule index can enforce it.

* `[TST-22]` Conformance MUST cover Arena-backed return provenance:

  1. a direct arena allocation returned by a function carrying `@borrows(arena)`;
  2. both `ref mut T` and mutable-span results backed by the named arena;
  3. rejection of an arena-backed view returned without the required provenance relationship;
  4. rejection when the returned view outlives the caller's arena;
  5. rejection of `@borrows` naming an arbitrary non-view parameter; and
  6. nested wrappers that each preserve `@borrows(arena)` provenance.

  The owner supplied the mnemonic `[TST-LT-ARENA-RETURN]`; it is normalized to this next unused
  numeric test-rule ID so the normative rule index can enforce it.

* `[LT-12]` **No hidden allocation.** The canonical helpers MUST be `@noalloc` and MUST NOT allocate
  a wrapper object, closure environment, heap box, or temporary container solely to represent the
  view bundle. A conforming implementation MAY inline the helper completely.

* `[LT-13]` **Persistent multi-owner aggregates remain explicit.** When a program genuinely needs
  a long-lived aggregate containing independently-lived views, v1 continues to require one of the
  existing designs: nested view structs, separate fields at the owning abstraction boundary,
  indices/handles, or an owned copy. `with_views` is not a back door around `[TYP-15]` or `[LT-2]`.

**0.9.5 interaction.** The recovered `[LT-13]` sentence records the pre-0.9.5 design boundary.
`0.9.5_Hardened_1` subsequently and intentionally generalized `[LT-2]` to inferred multi-region
`@view struct`s. That later language revision supersedes only the old requirement to encode every
persistent independent-view aggregate through the listed alternatives. It does not weaken
`[LT-10]`, `[TYP-15]`, `[TYP-15a]`, ordinary borrow checking, or the rule that `with_views` is an
ordinary allocation-free library API rather than a compiler intrinsic.

> **API-surface resolution:** the owner resolved ODR-005 on 2026-09-12 by selecting distinct
> all-mutable `_mut` helpers rather than making the shared names magically generic over `Span` and
> `MutSpan`, then resolved the callback side of ODR-006 by making `mut` explicit in each callable
> type. The owner completed ODR-006 by selecting `mut` rather than `owned` for each mutable helper
> input, and closed ODR-007 by preserving callable modes as compile-time canonical type metadata
> through the existing `Callable[Args, R]` abstraction. Mixed shared/mutable overloads are not
> specified. Both families use the same ordinary region and borrow machinery; the mutable family
> creates no new aliasing rule.

## H14.4 Regression and implementation obligations

* `[TST-14]` The conformance suite MUST add tests for `[WK-5]`–`[WK-10]`, including direct class-field
  cycles, three-node cycles, generic-container cycles, weak edges, foreign unknown edges, and a
  runtime cycle that static analysis could not predict.

* `[TST-15]` The suite MUST add `[EXC-8]`–`[EXC-14]` tests that inspect MIR/safety metadata and
  generated C/LLVM to verify preheader/postheader hoisting and preservation of dynamic checks where
  the proof is defeated.

* `[TST-16]` The suite MUST add `[LT-8]`–`[LT-13]` tests for two/three/four-view callbacks,
  independent source regions, mutable alias rejection, escaping-result rejection, `@noalloc`, and
  interaction with `@borrows`.

* `[BUD-7]` The build/performance harness MUST publish three separate measurements for the hardened
  mechanisms: static-cycle analysis time, exclusivity checks emitted per hot loop, and `with_views`
  call overhead. Cycle analysis MUST be bounded by package dependency summaries and MUST NOT require
  whole-program knowledge for correctness. `with_views` MUST be eligible for complete inlining.

* `[CLI-17]` `ember inspect --cycle <path>` MUST report the same ownership graph used by `[WK-5]`,
  including strong/weak/unknown edge classification and the shortest cycle when one is statically
  visible.

* `[CLI-16]` `ember inspect --safety` MUST report the new exclusivity classes from `[EXC-12]` and
  identify whether a loop check was hoisted, retained per access, or removed by the existing static
  proof rules.

## H14.5 Design boundary — what this cycle deliberately does not do

Hardened 14 deliberately does **not**:

- add a tracing or cycle collector;
- automatically rewrite strong fields into `Weak[T]`;
- make cycle-free ownership a compile-time safety guarantee;
- remove dynamic exclusivity checks merely because they are expensive;
- add a fourth aliasing/safety mechanism;
- permit multiple regions in storable `@view struct`s **within the then-current 0.9 semantics**; this limitation is intentionally superseded by `0.9.5_Hardened_1`;
- introduce named lifetimes into Ember source;
- make `with_views` a compiler intrinsic or special type;
- change `[WK-1]` or the core ownership model; `[LT-2]` and `[TYP-15]` are subsequently amended by the `0.9.5_Hardened_1` feature rules below.

These boundaries are intentional. A future revision may revisit multi-region storable views only as
a language-version change with a complete region-model design, diagnostics, ABI implications, and
conformance evidence.

---

# 0.9.5_Hardened_1 Language Revision — Inferred Multi-Region Views

This revision adds one source-language capability: a storable `@view struct` may preserve
multiple independently inferred borrow regions. The feature is intentionally designed as an
**inference-first extension** rather than a Rust-style named-lifetime system. Programmers do
not write lifetime names, region arguments, or region bounds in ordinary Ember source. The
compiler tracks region provenance internally and erases it before ABI/code generation.

The design goal is exactly Ember's established gradient: **C-like speed, Python-like source
readability, and Rust-like memory safety without requiring Rust's explicit lifetime bookkeeping
for ordinary code.** The feature closes a genuine expressiveness gap for RageV's ECS/SoA/data-
oriented workloads while preserving the existing aliasing XOR mutability discipline.

`0.9.5` is a conservative language revision: every program accepted under the earlier 0.9
contract remains accepted with the same meaning when compiled under `0.9.5`, unless it relied on
an implementation bug or on behaviour the earlier specification marked undefined. Programs must
opt into `0.9.5` explicitly when they want multi-region `@view struct` semantics.

## MRV-1 — Core model

* `[LT-14]` **Inferred multi-region view structs.** A `@view struct` MAY contain any finite number
  of borrowed fields whose source regions are independent. Each constructed view value carries a
  compiler-internal **region vector** with one slot for each borrowed field (or field group proven
  to share the same source region). The region vector is not an Ember source-level type argument.

* `[LT-15]` **No named lifetime syntax.** Region slots MUST be inferred from borrow provenance.
  The programmer MUST NOT be required to write `'a`, `'b`, region generic arguments, `outlives`
  clauses, or equivalent lifetime annotations to construct or use a multi-region view. `[LT-6]`
  remains unchanged: named lifetimes stay outside the 0.9.5 language surface.

* `[LT-16]` **Provenance, not intersection.** When a `@view struct` is constructed from several
  borrowed fields, the compiler MUST preserve each field's originating region instead of replacing
  all regions with their intersection. Fields that demonstrably originate from the same region MAY
  share a region slot; unrelated fields MUST remain independently constrained.

* `[LT-17]` **Validity is conjunctive.** A multi-region view is valid only while every region slot
  required by a field that may be accessed remains valid. A program cannot use one field after its
  region ends merely because another field's region is still valid. This is the central safety rule
  of the feature.

* `[LT-18]` **Ordinary borrow rules remain authoritative.** Every field borrow is still a normal
  Ember loan governed by `[BRW-1]`, `[BRW-2]`, two-phase borrowing, reborrowing, disjoint-field
  analysis, drop checking, and class-handle retention rules. Multi-region views do not create a
  fourth aliasing mechanism and do not weaken any existing conflict check.

## MRV-2 — Region inference and value flow

* `[LT-19]` **Region-slot inference.** For every borrowed field of a `@view struct`, the compiler
  records the source place and its inferred region. A region slot may be unified with another slot
  only when the normal borrow/region solver proves the same provenance relationship. Mere type
  equality, simultaneous liveness, equal lengths, or programmer intent is not sufficient.

* `[LT-20]` **Field-sensitive flow.** Moving, copying, destructuring, passing, returning, and
  storing a multi-region view preserve the region constraint of each borrowed field. Projection of
  one field carries only that field's region (plus any region required to keep its source place alive),
  rather than unnecessarily keeping unrelated fields' regions live. This is the key ergonomics
  improvement over the former intersection model.

* `[LT-21]` **Field replacement is checked as a normal borrow operation.** Assigning a new borrowed
  value into a view field is permitted only when the new field's region is valid for the containing
  value's current use. The compiler MUST recompute the field's region provenance at the assignment
  and MUST reject an assignment that would make a later field use outlive its new source. No runtime
  region tracking is introduced.

* `[LT-22]` **Returns are inferred, not annotated.** A function returning a multi-region `@view struct`
  MUST expose an internal region-provenance summary mapping each borrowed result field to the input
  region(s) from which it is derived. The caller uses that summary during normal region inference.
  This summary is part of the compiler's interface/type-checking metadata and is not written as a
  lifetime parameter in Ember source. If the implementation cannot infer a sound field-to-source
  relationship, it MUST reject the function body rather than widen the result to an intersection or
  invent a `'static` region.

* `[LT-23]` **`@borrows` remains useful but does not collapse regions.** Existing `@borrows(p, …)`
  remains normative for ordinary single-region view returns. It MUST NOT be used to force a
  multi-region return into one artificial region. For a multi-region return, the compiler uses the
  inferred field provenance from `[LT-22]`; `@borrows` MAY appear only where its existing contract
  is independently meaningful and MUST NOT make an unsafe field escape legal.

* `[LT-24]` **Non-escaping inference.** A region slot MAY be shortened by NLL when its corresponding
  field is no longer used. The existence of another live field MUST NOT keep that slot alive unless
  the value flow actually preserves a relationship requiring it. This permits the common RageV
  pattern of using one borrowed SoA/ECS column briefly while another column remains live.

## MRV-3 — Storage, containers, and escape rules

The canonical multi-region wording of `[TYP-15]` and `[TYP-15a]` appears in Part IV §4 below.
Those in-place definitions preserve the 0.8.4 static-region decision while generalising the
storage check to every carried region slot. This addendum does not duplicate them.

* `[LT-25]` **No self-referential views.** This feature does not permit a view field to borrow another
  field of the same containing value, nor does it make coroutine self-references legal. `[B5]`,
  `[CORO-6]`, and the existing self-reference restrictions remain unchanged. Multi-region means
  multiple external provenance relationships, not internal reference graphs.

* `[LT-26]` **No hidden ownership extension.** Constructing, copying, or storing a multi-region view
  MUST NOT retain, move, pin, or otherwise extend the lifetime of its source objects. A view remains
  a non-owning borrow. In particular, a multi-region view over two class fields keeps the relevant
  class handles alive through the existing `[RC-5]` loan mechanism, but the view itself does not add
  an ownership count.

## MRV-4 — Mutability and safety invariants

* `[LT-27]` **Independent does not mean overlapping.** Independent region slots do not imply that
  their memory ranges are disjoint. Two fields may carry different regions and still alias the same
  underlying storage through unsafe/foreign provenance. The backend MUST use `[DSJ-*]` rules, not
  region inequality, to establish `noalias`.

* `[LT-28]` **Mutable multi-region views remain exclusive.** Two `MutSpan` fields from distinct
  regions are both mutable only if the ordinary borrow checker permits both borrows. Distinct region
  variables are never themselves a proof of disjointness. If aliasing cannot be proven absent, the
  construction is rejected or must use an existing sanctioned mechanism such as `Cell`, `RefCell`,
  `assert_disjoint`, or an explicit `unsafe` proof where the existing rules permit it.

* `[LT-29]` **Drop and destruction.** Dropping a multi-region view performs no source destruction
  and no runtime lifetime action. It only ends the value's loans according to NLL. A source cannot be
  dropped while any field-derived loan remains live.

## MRV-5 — ABI, layout, hot reload, and FFI

* `[LT-30]` **Regions are compile-time only.** Region slots MUST NOT occupy bytes in the runtime
  representation. Two instantiations of the same `@view struct` with different inferred regions MUST
  have identical size, alignment, field offsets, padding, and calling convention.

* `[LT-31]` **Canonical `Layout[T]` is region-erased.** `[LAY-1]` MUST describe the runtime layout of
  a multi-region view exactly as it would describe the same pointer/span fields without region metadata.
  Region provenance is not a layout field, discriminant, niche, or ABI class.

* `[LT-31a]` **Region identity is not nominal type identity.** Two values of the same `@view struct`
  with different inferred region vectors have the same nominal type and the same runtime ABI. Region
  constraints are checked at each use site and through function interface summaries; they MUST NOT
  create distinct overloads, duplicate runtime vtables, separate symbol names, or separate layout
  descriptors. This keeps the feature from causing monomorphisation or ABI bloat merely because two
  call sites borrow from different owners.

* `[LT-32]` **FFI erases regions, never safety facts.** A multi-region view may cross a foreign boundary
  only where the existing FFI rules already permit the underlying runtime representation. The ABI
  carries no region identifiers. A foreign call MUST NOT be allowed to retain a borrowed view unless
  an existing explicit foreign retention/lifetime contract establishes that every relevant region
  remains valid. Unknown foreign retention remains unsafe under `[FFI-35]`/`[FFI-11d]`.

* `[LT-33]` **Hot reload erases regions.** Reload schemas, persistent function identity, state migration,
  pointer relocation, and epoch reclamation MUST treat region metadata as compile-time information.
  A reload MUST NOT serialize, compare, migrate, or otherwise depend on region identifiers. Existing
  reload safety rules remain unchanged.

* `[LT-34]` **Legacy one-region source compatibility.** Any `@view struct` whose fields all derive from
  one region has exactly the same borrow validity, storage eligibility, layout, ABI and generated-code
  obligations it had before 0.9.5. The feature MUST NOT make a previously valid one-region program
  invalid merely because the implementation internally represents its region as a one-slot vector.

* `[LT-35]` **Field-access summaries are part of callable contracts.** Every function, method, interface
  implementation, closure body, coroutine resume body, and foreign-safe wrapper that can receive a
  multi-region `@view struct` MUST have an internal access summary for each such parameter. The summary
  records, by field path, which operations may occur: `read`, `write`, `borrow_shared`, `borrow_mut`,
  `move`, `return`, or `publish`. A call site MUST require only the region slots needed by the fields
  that the callee can actually access, plus any slots required by the callee's returned or published
  provenance. If no sound field-sensitive summary is available (including an unresolved dynamic or
  foreign dispatch), the implementation MUST conservatively require all slots and MUST NOT guess a
  smaller set. The summary is compiler metadata, not source syntax and not an ABI-visible type.

* `[LT-36]` **Whole-value operations are distinguished from field projection.** A projection such as
  `v.positions` requires only the region slot(s) attached to that field path. An operation that treats
  the view as an undivided value — passing it to a callable whose access summary is `all_fields`, taking
  a whole-value mutable borrow, serialising/debugging/hashing/comparing all fields, or otherwise
  requiring unrestricted field access — requires every region slot to be valid. Purely type-level
  operations such as `size_of[T]()` do not require a region to be live. A whole-value copy or move is
  permitted only while all carried region slots are valid; it transfers/duplicates the corresponding
  loan constraints and never manufactures a longer region.

* `[LT-37]` **Method, interface, generic, and dynamic dispatch.** A method or interface call on a
  multi-region view uses the callee's field-access summary. Virtual/interface dispatch MUST use a
  conservative summary covering every field that any valid dynamic target may access; when that set
  cannot be bounded soundly, the call is treated as `all_fields`. A generic function receives the
  instantiated summary when monomorphisation proves it; before instantiation and for shared generics it
  MUST use its declared conservative summary. This rule prevents dispatch or generic abstraction from
  bypassing field-level region checks.

* `[LT-38]` **Partial moves and destructuring are field-sensitive.** Moving or destructuring one borrowed
  field transfers only that field's ownership/loan constraint and leaves unrelated field slots governed
  by their own provenance. A whole-value use after a partial move follows the existing definite-
  initialisation/partial-move rules. A moved field cannot be accessed again, and a field whose source
  region has expired cannot be moved into another view or otherwise published. View destruction itself
  remains non-accessing and may end the remaining loans without dereferencing their sources.

* `[LT-39]` **Unknown field selection is conservative.** Pattern matching, reflection, indexing through a
  field descriptor, or any other operation whose concrete field path is not statically known MUST require
  every region slot that could be accessed by that operation. A runtime value MUST NOT be used to select a
  field and thereby evade a region constraint. When control-flow analysis proves a specific field in a
  branch, the narrower slot set MAY be used inside that branch.

* `[LT-40]` **Provenance summaries are invalidated with interfaces.** A change to a function or method's
  field-access or field-to-region provenance summary MUST participate in the same interface hash,
  incremental-cache key, and generic-instantiation dependency checks that already protect signature/type
  changes. A caller MUST be rechecked when a summary changes. These summaries are compile-time facts and
  MUST NOT become runtime hot-reload schema fields or ABI compatibility requirements; `[LT-33]` remains
  authoritative that region metadata is erased from reload state. A stale summary MUST be treated as a
  compiler error, never as an optimisation hint.

* `[LT-41]` **No hidden narrowing at FFI or opaque calls.** A foreign call, function-pointer call, callback
  registration, reflection API, or `unsafe` boundary that does not carry a field-level retention/access
  contract is conservatively treated as accessing and potentially retaining every region slot in the
  supplied multi-region view. Safe code therefore cannot pass a partially valid view through an opaque
  boundary merely because the implementation happens not to touch one field at runtime. Existing FFI
  evidence may establish a narrower contract, but the contract MUST name the relevant field paths or
  explicitly state that all fields are retained/accessed.

* `[LT-42]` **Closure capture preserves field provenance.** A non-`owned` closure capturing a
  multi-region view records the field-level slots it can access. An `owned fn` closure is subject to the
  existing `[TYP-15]` storage rule and therefore requires all captured view slots to be `static`. A
  closure that conditionally captures or accesses only a field MAY carry only that field's slot, but an
  opaque closure call uses the conservative summary required by `[LT-35]`.

* `[LT-43]` **Coroutine boundaries remain strict.** Multi-region provenance may be tracked within a
  coroutine only where the existing coroutine rules permit the relevant view to remain live. The feature
  MUST NOT allow a view field from region A to survive a `yield` merely because another field from region
  B remains valid. If `[CORO-6]` rejects the borrow, the corresponding field borrow remains rejected;
  multi-region inference does not create a coroutine exception.

## MRV-6 — Ergonomics and RageV/DOD guidance

The intended source experience is deliberately lifetime-light:

```ember
@view struct TransformPhysicsView:
    positions: MutSpan[Vec3]
    rotations: Span[Quat]
    velocities: Span[Vec3]

fn integrate(v: TransformPhysicsView, dt: f32):
    for i in 0..min(v.positions.len(), v.velocities.len()):
        v.positions[i].x += v.velocities[i].x * dt
```

If `positions` and `rotations` originate from one `TransformSoA` borrow while `velocities` originates
from a separately-owned `PhysicsSoA`, the compiler may infer conceptually:

```text
TransformPhysicsView<R_transform, R_physics>
```

but the programmer writes neither region name. The runtime representation remains the ordinary
three spans. This is the intended Ember trade: the compiler performs the bookkeeping that makes the
program safe while source code stays close to Python-like data manipulation and the generated code
remains C-like.

For `std.ecs`, query-produced multi-region views MUST use the same rules. ECS access sets remain
ordinary borrow facts under `[OQ8-4]`; `std.ecs` does not acquire compiler magic. A query MAY return a
view whose component columns have different region provenance, and the compiler MUST preserve those
relationships without turning ECS into a privileged ownership subsystem.

A field-sensitive callable may then consume only the region it actually needs:

```ember
fn velocity_x(v: TransformPhysicsView) -> f32:
    return v.velocities[0].x
```

If `positions` and `rotations` no longer have live source regions at the call site but `velocities`
does, `[LT-35]` permits the call because the callable summary names only `velocities`. A callable that
may inspect the whole `TransformPhysicsView` requires all three slots. The compiler MUST derive these
requirements from the callable's verified summary, never from the fact that the nominal parameter type is
the same.

## MRV-7 — Explicit non-goals

This feature deliberately does **not**:

- introduce named lifetimes;
- introduce runtime region objects or reference-counted regions;
- permit self-referential structs;
- permit borrows across `yield` where `[CORO-6]` rejects them;
- make distinct regions a no-alias proof;
- weaken `unsafe` boundaries or foreign retention contracts;
- make arbitrary owning containers of views safe;
- require whole-program analysis for ordinary correctness;
- make multi-region views a compiler-only ECS feature;
- add GC, tracing, cycle collection, pinning, or hidden ownership.

## MRV-8 — Compatibility and implementation gates

* `[TST-17]` The conformance suite MUST cover: one-region legacy view structs; two/four/eight-field
  multi-region structs; fields sharing a region; fields from independent owners; field projection
  shortening; copy/move/destructure; return provenance; generic instantiations; `Option`/`Result`/tuple
  payloads; rejection of expired individual fields; mutable overlap rejection; `assert_disjoint`
  interaction; class-field retention; FFI rejection for unknown retention; and coroutine/yield rejection.

* `[TST-18]` Negative tests MUST include attempts to use a field after only that field's source dies,
  attempts to store a multi-region view in a class/static/Box/Shared destination with a non-static
  slot, attempts to treat region inequality as `noalias`, and attempts to manufacture a `'static`
  region by inference failure.

* `[TST-19]` Callable-access tests MUST cover: a function accessing one field of a multi-region view;
  a function accessing all fields; method and interface dispatch with different field summaries;
  generic monomorphisation and shared-generic conservative summaries; opaque function-pointer calls;
  reflection-selected fields; closure captures; whole-value copy/move after a slot expires; partial
  field moves; and stale interface-summary invalidation. Every case MUST include an accept and, where
  the rule can reject, a reject case.

* `[BUD-8]` The performance suite MUST compare multi-region views against equivalent hand-written
  separate `Span` parameters and against the legacy one-region representation where applicable. The
  generated code MUST be representation-equivalent after inlining where the optimizer can prove the
  same facts; no region metadata or access-summary dispatch may survive into runtime code for statically
  resolved calls. The suite MUST separately measure interface-summary and provenance-analysis cost so
  zero-runtime-cost safety does not hide pathological incremental-build cost.

* `[VERIFY-3]` The MIR verifier MUST verify that every multi-region view projection carries the
  correct region slot, that field replacement updates provenance, that callable access summaries agree
  with actual field accesses, that call sites require every slot named by the callee summary, that
  unknown summaries conservatively require all slots, that no region slot is widened to `'static` without
  a static source fact, and that code generation cannot emit a runtime operation corresponding solely to
  region bookkeeping.

* `[IMP-10]` The implementation matrix MUST track parser/resolver changes (expected: none for the
  region feature), typechecker/borrowck/MIR/verifier/codegen changes, diagnostics, FFI probes, and
  RageV conformance separately. The feature MUST NOT be marked `CONFORMANT` until all `[MRV-*]` rules
  and `[TST-17]`/`[TST-18]`/`[TST-19]` pass.

### Safety proof sketch

The safety argument is intentionally small: every borrowed field still corresponds to an ordinary
loan; the only semantic change is that a `@view struct` no longer merges independent loans into one
intersection region. A field may be used only while its own loan is valid. Every callable that can
receive the composite has a field-access summary, so the caller checks exactly the required region
slots, or conservatively all slots when the summary is unknown. Whole-value operations require all
slots. No operation may obtain a reference to a field without checking that field's region. No runtime
region metadata is needed, so there is no second runtime safety mechanism that could disagree with the
borrow checker. This preserves the existing Safe Ember guarantee while removing false coupling
between unrelated borrows.

### Why this is not a Rust-style lifetime tax

The feature deliberately takes the compiler complexity instead of charging the programmer for it.
Rust's MIR borrow checker similarly performs region inference and constraint propagation internally,
while explicit lifetime parameters become visible in signatures when required by its type system.
Ember's 0.9.5 rule is stricter about keeping region variables out of source-level type arguments:
ordinary users see provenance-aware views, not lifetime syntax. This is consistent with Ember's
existing rule that regions are inferred and programmer-written named lifetimes are reserved.

# 0.9 Hardened Implementation Contract

This section is normative. It supplements the existing Parts I–XXIII below.

## [IMP-1] Specification-to-implementation traceability

Every normative rule MUST have:

1. a stable rule identifier;
2. a canonical specification location;
3. at least one positive or negative conformance test;
4. a compiler/runtime/toolchain owner;
5. a diagnostic mapping when violation is diagnosable;
6. an implementation-completeness state.

No compiler component may treat an unowned rule as optional merely because its implementation has not yet been written.

## [IMP-2] No semantic invention

When the specification is silent on language semantics, the compiler MUST NOT infer semantics from C, C++, Rust, Swift, Python, or another language. It MUST emit an `OWNER DECISION REQUIRED` record and block semantic finalization.

Implementation details that do not affect observable language semantics MAY be selected by the implementation when the choice is the simplest sound and predictable one.

## [IMP-3] Canonical artifacts

The following artifacts are canonical:

| Artifact | Authority |
|---|---|
| Language semantics | This specification |
| Public std API signatures | `std/**/*.em` plus normative library contracts |
| Compiler IR invariants | Part XIX and this amendment |
| Runtime ABI | Part XIX plus canonical ABI descriptors |
| Diagnostics | Error registry + diagnostic rules |
| Conformance | `tests/conformance/**` |
| Build/toolchain behavior | Part XX |
| RageV integration | Part XXII |
| Owner decisions | `docs/DECISIONS.md` and accepted owner amendments |

Generated artifacts MUST NOT redefine any of the above.

## [IMP-4] Implementation completeness

A feature is not considered implemented merely because source code parses or code generation succeeds. A feature is complete only when its implementation-completeness record has:

```text
SPECIFIED
PARSED
TYPED
MIR
VERIFIED
CODEGEN
RUNTIME          (when applicable)
ABI              (when applicable)
DIAGNOSTICS      (when applicable)
CONFORMANCE
DOCUMENTED
```

All applicable states MUST be complete before the feature can be declared stable.

## [IMP-5] Canonical type identity

The compiler MUST provide one canonical `TypeIdentity` representation for every type.

`TypeIdentity` MUST distinguish, at minimum:

- primitive types;
- nominal types;
- generic instantiations;
- const-generic instantiations;
- associated types;
- function and closure types;
- coroutine frame types;
- dynamic interface types;
- `Self`;
- aliases after canonicalization;
- foreign types;
- opaque/generated types.

The same canonical identity MUST be used wherever semantic type identity affects:

- generic solving;
- monomorphisation;
- symbol mangling;
- ABI descriptors;
- reflection;
- incremental compilation;
- build hashing;
- hot-reload schemas;
- debugger type identity.

No subsystem MAY invent an incompatible type-name canonicalization.

## [IMP-6] MIR source provenance

Every MIR statement, terminator, compiler-generated safety check, borrow operation, drop operation, and runtime-check emission MUST retain source provenance sufficient to produce an accurate diagnostic.

Optimisation passes MUST preserve provenance or explicitly merge it according to a documented rule.

## [IMP-7] Compiler pass contracts

Every compiler pass MUST declare:

```text
input invariants
output invariants
semantic preservation obligations
diagnostic obligations
source-provenance obligations
```

The verifier MUST run after every safety-critical MIR transformation.


## [IMP-8] One consolidated normative source

Within this development target, the current addenda and the consolidated Parts I–XXIII body form
one normative contract. The front-matter authority boundary determines whether that target has
been adopted as the repository's normative specification.
Frozen predecessor files remain historical evidence outside this document. A rule ID MUST have
exactly one definition in the consolidated source; a later revision replaces wording in place or
uses a new, previously unused rule ID. Historical change-log prose is descriptive and MUST NOT be
extracted as a current definition. Rule-index, grammar, version-selection, owner-decision, and
conformance tools consume this one source directly; no duplicated registry or position-based
supersession algorithm is a second source of truth.

## [IMP-9] Canonical standard-library definition ownership

A public standard-library symbol MUST have exactly one defining module. Re-exports MUST be explicitly marked as re-exports and MUST NOT create a second canonical definition. For interior mutability, `std.cell` is the defining owner of `Cell`, `RefCell`, `Ref`, and `RefMut`; prelude exposure is re-export only.


---

# Compiler Pipeline Addendum

The reference compiler pipeline is:

```text
source
  ↓
lexer
  ↓
parser
  ↓
AST
  ↓
name resolution
  ↓
HIR
  ↓
type/interface solving
  ↓
borrow/lifetime checking
  ↓
drop checking
  ↓
MIR construction
  ↓
MIR verification
  ↓
drop elaboration
  ↓
MIR verification
  ↓
monomorphisation
  ↓
optimisation
  ↓
MIR verification
  ↓
backend lowering
  ├── C11
  └── LLVM
  ↓
link/runtime
```

## [COMP-1] Drop-check phase

Drop checking MUST be a distinct compiler phase.

It MUST verify that destruction cannot observe references whose lifetime has ended.

The analysis MUST account for:

- struct fields;
- classes;
- generic fields;
- closures;
- coroutine frames;
- `Shared`;
- `RefCell` guards;
- arena-backed values;
- handles;
- FFI-retained objects;
- destructor paths generated by partial moves.

---

# Generic Coherence Addendum

## [GEN-COH-1] Concrete implementation ownership

For an interface implementation of the form:

```text
extend Type: Interface { ... }
```

the implementation has exactly one package owner.

Two packages MUST NOT provide overlapping concrete implementations for the same canonical `(Type, Interface)` pair.

## [GEN-COH-2] Blanket overlap

Blanket implementations MUST be rejected when the compiler cannot prove that their applicable type sets are disjoint.

Ember does not implicitly introduce specialization semantics.

If specialization is introduced in a later language version, it MUST be a language-version change rather than a compiler extension.

---

# Closure ABI Addendum

## [CLO-ABI-1] Closure environment

A capturing closure MUST be represented as an environment object/value containing its captured state.

The compiler MUST define:

- capture ordering;
- field alignment;
- field destruction order;
- zero-sized captures;
- capture-free representation;
- closure identity;
- calling convention;
- escaping/non-escaping lowering.

## [CLO-ABI-2] Capture-free closures

A capture-free closure MAY lower directly to a function pointer when its callable interface permits it.

A capturing closure MUST NOT be represented solely as a function pointer.

---

# Coroutine ABI Addendum

## [COR-ABI-1] Coroutine frame

Every coroutine frame MUST have a stable compiler-defined layout containing sufficient state for:

- resume;
- completion;
- destruction;
- suspended state;
- active state;
- debug inspection.

The frame layout MUST be represented in the canonical layout descriptor.

## [COR-ABI-2] Destruction

Destroying a suspended coroutine MUST destroy live frame fields according to normal Ember destruction and lifetime rules.

---

# Runtime Addendum

## [RT-6] Canonical runtime ABI name

The canonical runtime ABI identifier is:

```c
EMBER_RUNTIME_ABI
```

If compatibility requires the historical spelling:

```c
EMBER_RT_ABI
```

it MAY exist only as an alias and MUST NOT represent a second ABI version.

## [RT-7] Reference-count overflow

Reference counters MUST NOT wrap.

Overflow is a fatal runtime error.

The implementation MUST ensure that a wrapped counter can never create a falsely live or falsely dead object.

## [RT-8] Atomic ordering matrix

Where reference-counted objects are thread-shareable, the reference implementation MUST use the following minimum ordering contract:

| Operation | Required ordering |
|---|---|
| strong retain | Relaxed |
| strong release | Acquire-Release |
| weak retain | Relaxed |
| weak release | Acquire-Release |
| publication of initialized object | Release |
| acquisition of published object | Acquire |
| reload publication | Release |
| reload entry | Acquire |

An implementation MAY use stronger ordering but MUST NOT use weaker ordering where that would violate the specified happens-before relation.

## [RT-9] Handle generation exhaustion

Generational handles MUST detect stale generations.

A generation counter MUST NOT wrap into a previously valid generation.

When a slot exhausts its generation space, the slot MUST permanently retire or otherwise enter a state in which an old handle can never become valid again.

The implementation MUST document its generation width.

---

# Layout Contract

## [LAY-1] Canonical layout descriptor

Every layout-sensitive type MUST have a canonical descriptor:

```text
Layout[T] {
    size
    alignment
    field offsets
    field sizes
    padding
    discriminant information
    niche information, when applicable
    ABI classification
}
```

The descriptor MUST be consumed consistently by:

- C backend;
- LLVM backend;
- C FFI;
- C++ FFI;
- GPU upload/layout generation;
- reflection;
- debugger metadata;
- serialization;
- hot-reload migration.

No subsystem may independently invent a conflicting layout.

---

# FFI Implementation Contract

## [FFI-IMPL-1] Canonical ABI descriptor

Every imported/exported FFI entity MUST have a canonical ABI descriptor containing, where applicable:

```text
target triple
calling convention
size
alignment
parameter ABI classes
return ABI class
aggregate classification
exception policy
ownership
nullability
effects
C++ ABI identity
```

## [FFI-IMPL-2] C++ ABI fingerprint

A C++ binding fingerprint MUST include all inputs capable of changing the externally observable ABI, including:

- target;
- compiler;
- compiler version;
- C++ language mode;
- standard library implementation;
- CRT;
- RTTI configuration;
- exception configuration;
- iterator/debug ABI configuration;
- relevant preprocessor definitions;
- packing/alignment configuration;
- calling convention;
- binding-relevant compiler flags.

A binding cache MUST be invalidated when a relevant fingerprint component changes.

## [FFI-IMPL-3] Generated thunk tests

Every generated C/C++ thunk MUST have ABI-level conformance coverage for:

- parameter ABI;
- return ABI;
- ownership;
- destruction;
- exception boundary;
- virtual dispatch where applicable;
- layout-sensitive aggregates.

---

# Foreign Callback Safety

## [FFI-CB-1] Callback generation identity

A retained foreign callback MUST carry sufficient identity to reject invocation after the associated object or binding generation becomes invalid.

The identity MUST include, as applicable:

```text
object identity
generation
retention state
callback ABI identity
hot-reload generation
```

A stale callback MUST fail according to the established foreign-boundary error policy rather than dereferencing reclaimed Ember state.

---

# Hot Reload Implementation Contract

## [HR-IMPL-1] Reload state machine

The reference runtime MUST model a reload transaction through these logical states:

```text
RUNNING
  ↓
REQUESTED
  ↓
QUIESCING
  ↓
QUIESCENT
  ↓
PLANNED
  ↓
PREPARING
  ↓
COMMITTING
  ↓
PUBLISHED
  ↓
RECLAIMING
  ↓
RUNNING
```

Each transition MUST define:

- permitted Ember execution;
- permitted foreign execution;
- allocation behavior;
- synchronization requirements;
- failure behavior;
- publication visibility.

## [HR-IMPL-2] Epoch publication

Hot-reload publication MUST use an epoch/generation model.

Each participating thread MUST expose its observed Ember execution epoch.

An old image MUST NOT be reclaimed until every participating thread has:

1. advanced to the new epoch; or
2. detached from the runtime.

## [HR-IMPL-3] Transaction failure

Failure before publication MUST leave the previously published program image operational.

Under the v1 transaction contract, failure after `PUBLISHED` is an unreachable state: all fallible preparation, migration, validation, and synchronization operations MUST complete before publication. If an implementation nevertheless detects an invariant violation after publication, it MUST treat that condition as a runtime/implementation fatal error and MUST NOT silently expose a partially committed image. v1 does not require a post-publication rollback mechanism.

---

# Determinism Addendum

## [DET-IMPL-1] Floating-point environment

A deterministic execution context MUST establish a defined floating-point environment.

The implementation contract MUST specify:

- rounding mode;
- floating-point exception behavior;
- flush-to-zero behavior;
- denormals-are-zero behavior;
- FPU precision where applicable;
- SIMD floating-point mode;
- FMA contraction;
- NaN behavior;
- signed-zero behavior;
- subnormal handling.

The deterministic profile MUST NOT rely on the ambient host process floating-point environment.

## [DET-IMPL-2] Differential determinism oracle

For programs eligible for deterministic execution, the conformance system SHOULD execute the same test through:

```text
Ember interpreter
C backend
LLVM backend
```

and compare observable results.

For bit-exact deterministic operations, comparison MUST be bit-exact.

---

# Standard Library Completeness Contract

## [STD-IMPL-1] Normative API/source correspondence

Every public standard-library API referenced by a normative language rule MUST have one canonical declaration in `std/**/*.em`.

The specification defines its semantics; the source defines its canonical callable signature.

## [STD-IMPL-2] Standard-library conformance

The following categories MUST have dedicated semantic/conformance coverage before 1.0:

- `Option`;
- `Result`;
- arrays;
- strings;
- spans/views;
- maps;
- sets;
- iterators;
- `Cell`;
- `RefCell`;
- synchronization primitives;
- atomics;
- arenas;
- handles;
- SoA containers;
- ECS support types.

Container invalidation, ownership, destruction, hashing, ordering, allocation and failure semantics MUST be explicitly covered where observable.

---

# Compiler Self-Verification

## [VERIFY-1] MIR verification

The verifier MUST execute after every MIR transformation that can affect:

- ownership;
- borrowing;
- lifetimes;
- regions;
- drops;
- type correctness;
- bounds;
- effects;
- coroutine state;
- FFI boundaries.

## [VERIFY-2] Pass preservation

A compiler pass MUST NOT be accepted into the reference compiler until its output satisfies the verifier and its semantic-preservation tests.

---

# Implementation Completeness Matrix

The repository MUST maintain a machine-readable implementation matrix with one row per normative rule.

Minimum schema:

```text
rule_id
spec_location
feature
parser_status
resolver_status
typechecker_status
borrowck_status
dropck_status
mir_status
verifier_status
codegen_status
runtime_status
abi_status
diagnostic_status
positive_tests
negative_tests
docs_status
ragev_status
owner_decision
implementation_notes
```

Allowed status values:

```text
NOT_STARTED
SPECIFIED
IN_PROGRESS
IMPLEMENTED
VERIFIED
CONFORMANT
BLOCKED
OWNER_DECISION_REQUIRED
NOT_APPLICABLE
```

A feature MUST NOT be marked `CONFORMANT` if any applicable prerequisite remains incomplete.

---

# Differential Conformance Architecture

The reference implementation SHOULD maintain three semantic oracles:

```text
             Ember source
                  │
       ┌──────────┼──────────┐
       ↓          ↓          ↓
  interpreter    C11       LLVM
       │          │          │
       └──────────┼──────────┘
                  ↓
             oracle check
```

For deterministic programs, the result MUST be equivalent and, where specified, bit-identical.

A mismatch is a compiler conformance failure until proven to originate in undefined/implementation-defined behavior permitted by the specification.

---

# Implementation Plan Revision

The implementation plan is divided into:

## Phase 7a — Tier 1 runtime integration

Includes:

- C++ FFI;
- interpreter;
- bodies-only hot reload;
- determinism;
- coroutine support;
- generic-instantiation controls;
- core RageV migration.

## Phase 7b — Tier 2 full hot reload

Includes:

- persistent function identity;
- reload schemas;
- state migration;
- pointer relocation;
- retained foreign objects;
- GPU/resource migration;
- epoch reclamation;
- transactional publication.

## Phase 8 — 1.0 hardening

Includes:

- complete conformance matrix;
- MIR verifier coverage;
- differential backend testing;
- full FFI ABI probes;
- deterministic execution validation;
- LSP;
- debugger integration;
- formatter;
- diagnostics;
- RageV production migration;
- performance gates.

LSP is a 1.0 tooling requirement. LLVM remains an implementation backend and MAY follow the C backend bootstrap sequence; a package registry is not a 1.0 language requirement.

---

# RageV Final Audit

Before Ember 1.0, RageV MUST be used as a reference workload.

The audit MUST cover:

```text
core math
engine types
ECS
inferred multi-region ECS/SoA views
resource handles
job system
renderer
render graph
GPU resources
Vulkan boundary
OpenGL boundary
C++ engine APIs
Jolt
ImGui
yaml-cpp
cgltf
editor
scripting
serialization
hot reload
debugging
determinism
performance
```

The audit MUST record:

- code migrated;
- code remaining in C++;
- FFI boundary count;
- unsafe boundary count;
- allocations;
- RC traffic;
- runtime checks;
- compile times;
- incremental compile times;
- binary size;
- hot-reload latency;
- memory consumption;
- rendering performance;
- conformance failures.

---

# 0.9 Hardening Cycle Log

| Revision | Classification | Primary result |
|---|---|---|
| 0.8.5_Hardened_2 | Hardening | version selection, runtime naming, dynamic-interface signatures, std ownership, MIR provenance |
| 0.8.5_Hardened_3 | Hardening | canonical type identity, coherence, drop checking, closure/coroutine ABI |
| 0.8.5_Hardened_4 | Hardening | runtime representation, atomic ordering, RC overflow, handle generations |
| 0.9_Hardened_1 | Owner-selected release-line boundary | carried the specified `UnsafeCell` / `RefCell` implementation contract; implementation evidence remains external, and completion alone is not a semantic change |
| 0.9_Hardened_2 | Hardening | C/C++ ABI descriptors, fingerprints, thunk verification, callback lifetime |
| 0.9_Hardened_3 | Hardening | hot-reload state machine and epoch reclamation |
| 0.9_Hardened_4 | Hardening | deterministic floating-point environment |
| 0.9_Hardened_5 | Hardening | canonical layout descriptor |
| 0.9_Hardened_6 | Hardening | standard-library semantic completeness |
| 0.9_Hardened_7 | Hardening | compiler self-verification and differential oracle |
| 0.9_Hardened_8 | Hardening | implementation-plan Phase 7a/7b completion |
| 0.9_Hardened_9 | Hardening | 1.0 tooling/gate reconciliation |
| 0.9_Hardened_10 | Hardening | complete RageV implementation/conformance audit |
| 0.9_Hardened_11 | Hardening | implementation-contract hardening and consistency reconciliation |
| 0.9_Hardened_12 | Hardening + owner resolution | resolved OQ-8 to library-level `std.ecs`; closed owner-decision ambiguity |
| 0.9_Hardened_13 | Hardening | corrected revision identity, historical boundary, self-consistency contract, hot-reload failure wording, std ownership, amendment namespace, and owner-decision guidance |
| 0.9_Hardened_14 | Hardening | ARC-cycle analysis/diagnostics, loop-hoisted class exclusivity, independent-view callback composition, and conformance/performance gates |
| 0.9.5_Hardened_1 | **Language revision** | inferred multi-region `@view` structs with implicit region provenance, field-sensitive lifetime tracking, region-erased ABI, and RageV/SoA conformance gates |
| 0.9.5_Hardened_2 | Hardening | callable field-access summaries, whole-value versus projection validity, opaque-call conservatism, summary invalidation, and ECS-plan reconciliation |
| 0.9.5_Hardened_3 | Hardening | honest evidence status, inherited-contract consolidation, and explicit E3065 artifact requirements |
| 0.9.5_Hardened_4 | Hardening | repaired source authority, missing-rule accounting, rule-ID collisions, single-source extraction, examples, and inherited editorial corruption |
| 0.9.5_Hardened_5 | Hardening/source recovery | restored owner-supplied `[LT-8]`–`[LT-13]` callback-composition rules, preserved their 0.9.5 supersession boundary, and exposed the unprovided mutable-overload surface as ODR-005 |
| 0.9.5_Hardened_6 | Owner-approved hardening clarification | resolved ODR-005 with explicit all-mutable `with_views2_mut/3_mut/4_mut` helpers using canonical `MutSpan[T]`; mixed-mutable overloads remain outside the contract |
| 0.9.5_Hardened_7 | Owner-approved hardening clarification | added borrowed/default, `mut`, and `owned` callable-type parameter modes; applied explicit `mut` callback modes to the all-mutable helpers; added `[LT-11a]` and `[TST-20]` |
| 0.9.5_Hardened_8 | Owner-approved hardening clarification | made mutable helper inputs explicit `mut` reborrows; preserved callable mode vectors as compiler-known metadata through `Callable[Args, R]`; added `[FN-6a]`, `[LT-8a]`, and `[TST-21]` |
| 0.9.5_Hardened_9 | Owner-approved hardening clarification | permitted the narrow `@borrows(arena)` return-provenance contract for Arena-backed views; added `[LT-4a]`, `[LT-4b]`, and `[TST-22]` |

---

## 0.9_Hardened_14 Audit Record

| ID | Classification | Resolution |
|---|---|---|
| AUD-014-01 | High | Accidental ARC cycles now receive package-level SCC analysis, actionable `L3001` diagnostics, runtime correlation, and `ember explain/inspect --cycle`; ownership semantics remain unchanged. |
| AUD-014-02 | High | Repeated dynamic class-exclusivity checks in provably stable loops are lowered to one preheader/postheader check with an internal MIR token; failed proofs retain the original checked path. |
| AUD-014-03 | High | The historical record says independent view lifetimes remained forbidden in storable `@view struct`s while canonical `std.borrow.with_views` helpers provided allocation-free late-bound callback composition; the exact helper definitions were recovered in H5 from the owner-supplied Hardened 14 text. |
| AUD-014-04 | Medium | Added conformance and performance gates for cycle analysis, exclusivity hoisting, and multi-view composition. |
| AUD-014-05 | Medium | Corrected the revision lineage so `0.9_Hardened_14` names `0.9_Hardened_13` as its immediate predecessor. |

## 0.9.5_Hardened_1 Audit Record

| ID | Classification | Resolution |
|---|---|---|
| AUD-095-01 | Critical | Replaced the single-region `@view struct` restriction with inferred per-field region slots while preserving ordinary borrow checking. |
| AUD-095-02 | Critical | Added conjunctive validity: every accessed field remains tied to its own source region; no region is widened to `'static` and no region inequality is treated as disjointness. |
| AUD-095-03 | High | Kept region variables compiler-internal; no named lifetimes, source-level region arguments, runtime region objects, or ABI-visible region metadata were introduced. |
| AUD-095-04 | High | Preserved `[TYP-15]` as an all-slots escape rule and retained rejection of non-static multi-region views in unbounded storage. |
| AUD-095-05 | High | Preserved FFI/hot-reload/layout semantics by erasing regions before ABI and serialization; foreign retention still requires explicit lifetime evidence. |
| AUD-095-06 | High | Added field-sensitive MIR provenance, verifier checks, negative conformance tests, and a C-equivalence performance gate. |
| AUD-095-07 | Medium | RageV ECS remains library-owned; multi-region views are a general language feature consumed by `std.ecs`, not compiler-integrated ECS semantics. |

## 0.9.5_Hardened_2 Audit Record

| ID | Classification | Resolution |
|---|---|---|
| AUD-095-08 | Critical | Closed the callable-contract gap: field-sensitive access summaries are mandatory for functions, methods, interfaces, closures, generics, coroutines, and opaque calls; unknown summaries conservatively require all slots. |
| AUD-095-09 | Critical | Defined whole-value versus field-projection validity so a partially expired composite cannot be copied, moved, published, or passed through an unrestricted call while valid fields remain independently usable. |
| AUD-095-10 | High | Summary changes participate in interface hashing, incremental compilation, generic-instantiation dependencies, and hot-reload compatibility; stale summaries are compiler failures. |
| AUD-095-11 | High | Added partial-move, dynamic-field-selection, closure-capture, interface-dispatch, and opaque-call rules so language abstraction cannot bypass region provenance. |
| AUD-095-12 | High | Added dedicated callable-summary conformance tests and compile-time performance measurement while retaining region-erased runtime representation. |
| AUD-095-13 | High | Reconciled the RageV integration plan with the closed library-level `std.ecs` architecture by removing the stale competing ECS ownership path. |
| AUD-095-14 | Medium | Updated the generated syntax fixture to target `0.9.5` rather than the stale `0.5` directive. |
| AUD-095-15 | Medium | Recorded the callable-provenance changes under amendment `HC-095-02` and extended the specification-consistency gate to the new hardening. |
| AUD-095-16 | High | Reconciled compile-time provenance-summary invalidation with `[LT-33]`: summaries affect compilation caches but never runtime hot-reload schemas or ABI compatibility. |

**Amendment:** `HC-095-02` — callable field-access summaries, whole-value validity, opaque-call conservatism, provenance-summary invalidation, and associated conformance/performance obligations.

## 0.9.5_Hardened_4 Audit Record

| ID | Classification | Resolution |
|---|---|---|
| AUD-095-17 | Source custody | Preserved the received H3 file byte-for-byte and kept 0.8.5 authoritative during intake. |
| AUD-095-18 | Source-recovery blocker | Corrected the unsupported claim that LT-8 through LT-13 were recovered; all six definitions remain required before adoption. |
| AUD-095-19 | Rule-ID integrity | Preserved inherited CLI-15, RT-1 through RT-4, and DIA-18; assigned the new rules CLI-17, RT-6 through RT-9, and DIA-19. |
| AUD-095-20 | Single-source extraction | Removed the duplicated normative rule registry; heading definitions are extracted directly and the navigation index is non-normative. |
| AUD-095-21 | Consolidation | Kept the 0.8.5-derived body normative with 0.9.5 replacements applied in place; no historical-boundary rule discards the base language. |
| AUD-095-22 | Editorial/source recovery | Repaired the previously decided TST-11 merge corruption and the mutable-span example without changing language semantics. |
| AUD-095-23 | Evidence discipline | Retained H3's distinction between SPECIFIED, IMPLEMENTED, VERIFIED, and CONFORMANT. |
| AUD-095-24 | Version-selection clarity | Removed the circular statement that `MOD-6a` adds `0.9.5` beyond a `MOD-6` set that already contains it; `0.9` and `0.9.5` remain distinct contracts in one exact supported set. |

**Amendment:** `HC-095-03` — non-semantic H4 consolidation, source-accounting, ID-integrity,
tooling-structure, and inherited editorial repairs. At H4 freeze, the only unresolved item was the
exact normative text of LT-8 through LT-13; it was not supplied by that amendment and is recovered
by H5 below.

## 0.9.5_Hardened_5 Audit Record

| ID | Classification | Resolution |
|---|---|---|
| AUD-095-25 | Source recovery | Restored the owner-supplied Hardened 14 definitions of `[LT-8]`–`[LT-13]`; normalized transport-only Markdown corruption without inferring rule substance. |
| AUD-095-26 | Supersession integrity | Preserved the callback helpers and their no-escape/allocation/borrowing contract while making explicit that 0.9.5's later multi-region `[LT-2]` supersedes only Hardened 14's former aggregate-workaround boundary. |
| AUD-095-27 | Owner boundary | Did not invent `MutSpan` overload signatures from `[LT-11]`; the exact mutable helper surface is isolated as ODR-005 because it can affect accepted programs. |
| AUD-095-28 | Frozen lineage | Preserved H4 unchanged as the immediate predecessor and issued the source repair as H5 under the repository hardening protocol. |

**Amendment:** `HC-095-04` — owner-supplied `[LT-8]`–`[LT-13]` source recovery, 0.9.5
supersession reconciliation, and explicit mutable-helper API boundary. This is a hardening/source
recovery amendment, not a language-semantic change.

## 0.9.5_Hardened_6 Audit Record

| ID | Classification | Resolution |
|---|---|---|
| AUD-095-29 | Owner-approved API clarification | Closed ODR-005 by adding explicit all-mutable two-, three-, and four-view helper signatures to `[LT-8]`. |
| AUD-095-30 | Terminology consistency | Used the existing canonical `MutSpan[T]` type name; the owner ruling's `SpanMut[T]` spelling was normalized rather than introducing a duplicate type. |
| AUD-095-31 | Borrow safety | Tightened `[LT-11]` to bind the `_mut` family to ordinary exclusive-borrow rules; no helper creates a new aliasing mechanism. |
| AUD-095-32 | API boundary | Kept shared and all-mutable families explicit and did not invent mixed `Span`/`MutSpan` overloads. |
| AUD-095-33 | Implementation-readiness boundary | Recorded ODR-006 because `[FN-1]` defaults unmarked parameters to shared borrowing and `fn_type` currently carries types but no parameter modes; H6 does not guess the mutable callback-mode representation. |

**Amendment:** `HC-095-05` — owner-approved resolution of the mutable `with_views` API boundary.
Because H5 was already frozen, ADR-023 requires this clarification to be issued as H6. The adopted
0.8.5 program set is unchanged; H6 closes the overload-family ambiguity in the not-yet-adopted
0.9.5 target while ODR-006 retains the separate callback-mode question.

## 0.9.5_Hardened_7 Audit Record

| ID | Classification | Resolution |
|---|---|---|
| AUD-095-34 | Owner-approved grammar clarification | Extended `fn_type` with omitted/borrowed, `mut`, and `owned` parameter modes using ordinary `[FN-1]`/`[FN-2]` semantics. |
| AUD-095-35 | Mutable callback contract | Applied explicit `mut` modes to every callback parameter in the all-mutable `_mut` helper family. |
| AUD-095-36 | Conformance/diagnostic boundary | Added `[LT-11a]` and `[TST-20]` so mode mismatch, mutable aliasing, and ordinary callable-mode behavior are testable. The supplied mnemonic `[TST-LT-MUT]` was normalized to the next unused numeric test-rule ID because the normative rule-ID grammar requires a numeric suffix. |
| AUD-095-37 | Terminology consistency | Continued to normalize the ruling's `SpanMut[T]` spelling to Ember's existing `MutSpan[T]` type. |
| AUD-095-38 | Remaining owner boundaries | Narrowed ODR-006 to the helper functions' own unmarked inputs and opened ODR-007 for the mode-vector bridge to `Callable[Args, R]`; neither is inferred. |

**Amendment:** `HC-095-06` — owner-approved callable parameter-mode grammar and mutable-callback
contract. Because H6 was already frozen, ADR-023 requires this clarification to be issued as H7.
The adopted 0.8.5 program set remains unchanged; H7 is still a not-yet-adopted development target.

## 0.9.5_Hardened_8 Audit Record

| ID | Classification | Resolution |
|---|---|---|
| AUD-095-39 | Owner-approved helper-mode clarification | Closed the remaining half of ODR-006: shared inputs remain borrowed, every mutable helper input is `mut`, and neither helper family consumes an input view. |
| AUD-095-40 | Owner-approved callable abstraction clarification | Closed ODR-007 by preserving the full parameter-mode vector as compiler-known canonical type metadata through the existing `Callable[Args, R]` abstraction. |
| AUD-095-41 | Borrow/region boundary | Added `[LT-8a]` and tightened `[LT-10]`: forwarding through `Callable` cannot erase modes, and helpers neither own source storage nor extend its lifetime. |
| AUD-095-42 | Conformance integrity | Added `[TST-21]`; normalized the supplied non-indexable mnemonic `[TST-LT-MODE]` to the next unused numeric test-rule ID without changing its obligations. |
| AUD-095-43 | Terminology consistency | Continued to normalize `SpanMut[T]` to Ember's canonical `MutSpan[T]`; no second mutable-span type or alias was introduced. |

**Amendment:** `HC-095-07` — owner-approved helper input modes and callable-mode preservation.
Because H7 was already frozen, ADR-023 requires this clarification to be issued as H8. The adopted
0.8.5 program set remains unchanged; H8 is still a not-yet-adopted development target and makes no
implementation or conformance claim.

## 0.9.5_Hardened_9 Audit Record

| ID | Classification | Resolution |
|---|---|---|
| AUD-095-44 | Owner-approved region-provenance clarification | Permitted `@borrows(arena)` as a narrow exception for a returned view proven to derive from storage owned by the named `Arena`; `Arena` remains a non-view type. |
| AUD-095-45 | Wrapper contract | Added `[LT-4a]` so direct and nested user-defined wrappers preserve the arena-to-result relationship in each public signature. |
| AUD-095-46 | Safety boundary | Added `[LT-4b]`; arbitrary non-view parameters, unrelated results, lifetime extension, ownership transfer, and borrow-checker bypass remain forbidden. |
| AUD-095-47 | Conformance integrity | Added `[TST-22]`; normalized the supplied non-indexable mnemonic `[TST-LT-ARENA-RETURN]` to the next unused numeric test-rule ID. |
| AUD-095-48 | API consistency | Used the already-specified `alloc` and `alloc_array` surfaces in examples rather than introducing the ruling's otherwise-undefined `alloc_span`/`alloc_mut_span` spellings. |

**Amendment:** `HC-095-08` — owner-approved Arena-backed return provenance.
Because H8 was already frozen, ADR-023 requires this clarification to be issued as H9. The adopted
0.8.5 source remains unchanged; H9 is the new frozen, not-yet-adopted development target and makes
no blanket implementation or conformance claim.

### 0.9.5_Hardened_9 language-conformance condition

`0.9.5_Hardened_9` MUST NOT be declared conformant unless `[LT-8]`–`[LT-13]`,
`[LT-8a]`, `[LT-11a]`, `[FN-6a]`, `[TST-20]`, `[TST-21]`,
`[LT-4a]`, `[LT-4b]`, `[TST-22]`, `[LT-14]`–`[LT-43]` including `[LT-31a]`, `[MIR-REG-1]`,
`[TYP-15]`, `[TYP-15a]`, `[VERIFY-3]`, `[TST-17]`–`[TST-19]`, and `[BUD-8]` all have
passing conformance evidence; the MIR verifier proves region-slot preservation and checks callable
access summaries against actual MIR field accesses; unknown calls conservatively require all
slots; stale summaries invalidate compilation dependents; generated C/LLVM contains no runtime
region bookkeeping or summary-dispatch machinery; and the RageV workload demonstrates that
multi-region ECS/SoA views introduce no measurable overhead versus equivalent separate `Span`
parameters after equivalent optimisation.


This revision incorporates the Hardened_14 hardening record including the owner-supplied
`[LT-8]`–`[LT-13]` source, the accepted 0.9.5 multi-region-view language revision, and the
callable-provenance hardening of Hardened_2. The current revision identity, consolidated-body
boundary, feature lineage, owner-decision guidance, standard-library ownership, hot-reload
transaction contract, and multi-region view safety contract are explicit.

### Acceptance condition

`0.9.5_Hardened_9` MUST NOT be declared clean unless:
1. the document has one current version identity;
2. the consolidated Parts I–XXIII body and the current addenda form one normative source;
3. the active revision lineage ends at `0.9.5_Hardened_9`;
4. the current consistency checklist targets `0.9.5_Hardened_9`;
5. `std.cell` is the sole defining owner of interior-mutability types;
6. post-publication hot-reload failure has no unspecified recovery semantics;
7. closed owner decisions are not presented as open implementation choices;
8. multi-region view provenance is verified by the MIR verifier and all negative escape tests pass;
9. no runtime region bookkeeping appears in C/LLVM output for conforming multi-region views.
10. the source-recovery record for `[LT-8]`–`[LT-13]` is traceable to the owner-supplied text and H4 remains unchanged;
11. ODR-005 is closed by the explicit all-mutable `_mut` signatures, and no mixed-mutability overload is inferred.
12. ODR-006 is closed by explicit borrowed/shared and `mut` helper input modes; no helper consumes an input view;
13. ODR-007 is closed by compile-time preservation of the full mode vector through `Callable[Args, R]`, with no runtime mode bookkeeping.
14. `[LT-4a]` and `[LT-4b]` permit only Arena-backed `@borrows(arena)` provenance and do not make `Arena` or arbitrary non-view parameters view-typed.

---

## 0.9.5_Hardened_9 Evidence Status — No Implementation Claim

The current document is a specification artifact, not a report of repository state.

1. Every current 0.9/0.9.5 rule is treated as `SPECIFIED` unless separately backed by repository
   evidence.
2. No rule is promoted to `IMPLEMENTED`, `VERIFIED`, or `CONFORMANT` merely because the specification
   describes the required implementation.
3. The conformance gate remains closed until the required compiler, runtime, tooling, diagnostic,
   and test artifacts exist and pass.
4. The implementation-completeness matrix MUST distinguish `SPECIFIED` from `IMPLEMENTED`,
   `VERIFIED`, and `CONFORMANT`.
5. Missing implementation artifacts are reported as gaps; the specification does not simulate their
   existence.

# 1.0 Readiness Gate

Ember MUST NOT be declared 1.0 merely because the reference compiler compiles Ember programs.

The following gates MUST pass:

### Language correctness

- all normative rules have stable IDs;
- all rule IDs have conformance coverage;
- no unresolved semantic owner decisions remain in the 1.0 language surface;
- no duplicated or conflicting normative definitions remain.

### Compiler

- parser complete;
- type system complete;
- borrow checker complete;
- drop checker complete;
- MIR verifier complete;
- code generation complete;
- diagnostic mappings complete.

### Runtime

- ownership/destruction verified;
- RC verified;
- `Cell`/`RefCell`/`UnsafeCell` verified;
- synchronization verified;
- handle generation verified;
- coroutine runtime verified;
- hot reload verified.

### ABI / FFI

- C ABI verified;
- C++ ABI verified;
- layout probes verified;
- thunk golden tests verified;
- callback lifetime verification verified.

### Determinism

- deterministic FP environment verified;
- interpreter/C/LLVM differential tests pass;
- reproducibility tests pass.

### Tooling

- formatter;
- package/build tooling;
- test runner;
- structured diagnostics;
- LSP;
- debugger integration;
- rule-index tooling;
- implementation-completeness tooling.

### RageV

The production reference workload MUST successfully exercise the complete supported migration path.

---

# End-State Principle

The goal of the 0.9 hardening cycle is not to make Ember contain more mechanisms.

The goal is to make every mechanism already chosen by the language **mechanically implementable, testable, diagnosable, ABI-stable, and predictable**.

The final progression is therefore:

```text
0.8.5
  │
  │ specification gaps
  ↓
0.8.5 hardened
  │
  │ compiler/runtime implementation
  ↓
0.9
  │
  │ implementation convergence
  ↓
0.9 hardened
  │
  │ conformance + RageV
  ↓
1.0
```

The language is considered converged when repeated implementation passes stop producing material semantic ambiguity, missing invariants, ABI contradictions, runtime-safety gaps, or conformance blockers.

Only at that point should Ember's design be considered complete enough for a stable 1.0 implementation.

---


---

## 0.9.5_Hardened_9 consolidation record

The body below originated in `0.8.5_Hardened_1`, incorporates the accepted 0.9/0.9.5 changes in
place, and is normative within this target together with the current addenda above. Frozen predecessor files are
the historical diff bases. Existing rule IDs retain their meanings; newly added rules use unused
IDs. In particular, the 0.8.5 `CLI-15`, `RT-1`–`RT-4`, and `DIA-18` definitions remain intact;
the new cycle-inspection, runtime-hardening, and E3065-artifact rules are `CLI-17`, `RT-6`–`RT-9`,
and `DIA-19` respectively. The 0.9.5 wording of `LT-2`, `TYP-15`, and `TYP-15a` appears once, in
place, in the consolidated body.

If a rule ID, diagnostic, grammar production, attribute, ABI field, or standard-library API is
referenced but has no recoverable definition in the complete lineage, it is a `SPEC_GAP` and
blocks conformance. It MUST NOT be fabricated from another language's behavior. The former
LT-8-through-LT-13 source gap is closed by the owner-supplied definitions in §H14.3, and ODR-005
is closed by the owner-approved explicit all-mutable `_mut` helper family in H6.

## Non-normative heading index

This is a navigation aid, not a registry and not a second definition site. The rule-index tool
recognises an ID immediately following a Markdown heading marker as the definition at that heading.
The unbracketed names below deliberately cannot be extracted as rule definitions.

* `OQ8-1` — ECS is library-owned.
* `OQ8-2` — RageV-compatible sparse-set layout.
* `OQ8-3` — stable query iteration.
* `OQ8-4` — access-set enforcement.
* `OQ8-5` — comptime specialization without language specialization.
* `OQ8-6` — FFI interoperability.
* `OQ8-7` — no ECS compiler magic.
* `IMP-1` through `IMP-9` — implementation-contract headings.
* `COMP-1`, `GEN-COH-1`, and `GEN-COH-2` — drop-check and coherence headings.
* `CLO-ABI-1`/`CLO-ABI-2` and `COR-ABI-1`/`COR-ABI-2` — callable ABI headings.
* `RT-6` through `RT-9` and `LAY-1` — runtime and layout headings.
* `FFI-IMPL-1` through `FFI-IMPL-3` and `FFI-CB-1` — foreign-interface headings.
* `HR-IMPL-1` through `HR-IMPL-3` — hot-reload implementation headings.
* `DET-IMPL-1`/`DET-IMPL-2` — determinism implementation headings.
* `STD-IMPL-1`/`STD-IMPL-2` — standard-library implementation headings.
* `VERIFY-1`/`VERIFY-2` — verifier headings.

`tools/rule_index.py` MUST extract these heading definitions directly and MUST continue to
distinguish a definition from a citation.

## CONSOLIDATED_NORMATIVE_BODY_BEGIN — 0.8.5 lineage

**Consolidated normative body:** the still-valid 0.8.5 language contract is incorporated below,
with owner-approved 0.9/0.9.5 replacements applied in place. This is current normative text inside
the H4 target, not a verbatim historical snapshot. The front-matter authority boundary controls
repository adoption, and the exact frozen 0.8.5 source remains a separate repository file.

**Historical lineage:** 0.8.5 is additive over 0.8.4, which is additive over 0.8.3. S1 made
0.8.4; owner-approved S2, S3, and S4 made 0.8.5. This is ancestry, not the immediate predecessor
of the current H4 document, which is identified in the front matter.

**Compatibility:** 0.9.5 retains the supported 0.9 and 0.8.x source contracts except where a
language revision explicitly selected new semantics. Version selection is governed by `[MOD-6]`
and `[MOD-6a]`; an older source version does not silently acquire 0.9.5 multi-region behavior.

**Authority when adopted:** the consolidated source is the sole normative contract. During intake,
the front-matter authority notice governs and the repository's existing 0.8.5 source remains
normative.
**Audience:** Implementing agents and engineers. This document is written to be executed against, not read for inspiration.
**Reference workload:** [RageV](https://github.com/Insomniac-Coder/RageV) (Windows C++ engine; Vulkan 1.3 + OpenGL 4.5 RHI; sparse-set ECS; render graph; C# scripting via a function-pointer table)
**Principle:** safe by default, provable by request, native when necessary.

---

## How to use this document

1. **Part 0** records the foundational design decisions, the alternatives that were rejected, and why. Read it first; it explains the shape of everything after it.
2. **Parts I–XVII** are the *language* specification. Normative rules carry stable identifiers in the form `[XXX-n]` (e.g. `[OWN-4]`). Every rule with an identifier MUST have at least one test in the conformance suite that references it (see Part XX §5).
3. **Part XIX** specifies the compiler internals (IRs, passes, algorithms, ABI, mangling, backends).
4. **Part XX** specifies the toolchain (CLI, manifest, build graph, test harness, diagnostics format, error-code registry).
5. **Part XXI** is the implementation plan: phases, milestones, acceptance tests, repository layout, and operating instructions for an implementing agent.
6. **Part XXII** is the RageV integration plan, stage by stage, with concrete ABI sketches.
7. **Part XXIII** lists questions that require an owner decision. An implementing agent MUST NOT silently choose an answer to these; it records a provisional choice in `docs/DECISIONS.md` and flags it.

The words **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, **MAY** are normative (RFC 2119). "v1" means the first stable release; "v2/v3" mean later releases. Anything marked **(v2)** or **(v3)** is specified so that v1 does not preclude it, but is not required for v1.

**Guiding sentence for the implementer:** when the specification is silent **on a matter of language semantics**, an implementation MUST NOT infer the answer by analogy with another language. It records a provisional decision in `docs/DECISIONS.md`, marks it as filling a specification gap, and raises the gap — because two implementations reasoning independently from "what a C programmer would expect" reach two languages, and an analogy is not a rule anyone can check. For matters the specification leaves genuinely open — a diagnostic's wording, an internal data structure, an optimisation's aggressiveness — choose the option that is (a) simplest to implement soundly and (b) most predictable at runtime, in that order, and write the choice down.

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

## Change log — 0.8.5_Hardened_1

**0.8.5 carries three owner rulings of 2026-09-10. All three are additive: no
program valid under 0.8.4 becomes invalid.**

**`[UNS-10]`, `[UNS-10a]`, `[UNS-10b]` — `UnsafeCell[T]` becomes a real
primitive.** ERR-043 recorded that `UnsafeCell` was named exactly once, in
`[CELL-9]`, and defined by no rule — so `std` could be given interior
mutability the compiler knows about while no third-party package could build
its own. The owner ruled that the distinction is not acceptable and that
`UnsafeCell` is retained as the language's lowest-level interior-mutability
primitive, in `std.mem`, with deliberately narrow semantics: it permits
mutation through shared access **only from `unsafe` code**, hands out no safe
reference, checks nothing at run time, synchronises nothing, suspends neither
`[BRW-1]` nor lifetime, region, type or bounds checking, and creates no further
safety tier. It completes a hierarchy rather than opening a hole — `Cell`,
`RefCell`, `Mutex`/`RwLock`, `UnsafeCell` — and the author of an abstraction
built on it carries `[UNS-4]`'s obligations. This is the change that forced the
language number rather than a hardening.

**`[CELL-12]` — `RefCell[T]` is never `Copy`.** `[CELL-4]` derives `Cell`'s
`Copy`-ness from its field, and read mechanically the same derivation would
make a `RefCell` `Copy` whenever `T` is. The owner ruled that it must not:
a `RefCell` carries mutable runtime borrow state, and duplicating it would give
two cells inconsistent knowledge of one storage, which is exactly the invariant
`[CELL-5]`..`[CELL-8]` rest on. `RefCell` is an explicit exception to
`[CELL-4]`, and the rule now says so rather than leaving it to be inferred.

**`[FN-1a]` — a `mut` parameter at a view type accepts a view value.** ERR-041
recorded that `[FN-1]`'s "the argument MUST be a mutable place", read literally,
rejects Part VII §7's own worked example `normalize(buf.as_mut_span())`. The
owner ruled that the example governs: where the parameter's declared type is
itself a view, the mutable-place requirement applies to the place the view was
**taken of**. This admits no arbitrary temporary and bypasses no mutability
check — the borrow relationship of the view-producing expression is preserved
and checked. The compiler already behaved this way, so deviation D5 closes with
no code moving and `tests/conformance/FN-1a/` pins it.

**What has no tests yet, and why.** `[CELL-12]`, `[UNS-10]`, `[UNS-10a]` and
`[UNS-10b]` describe types the compiler does not implement — `RefCell` and
`UnsafeCell` are both unbuilt. They are in `tools/rule_index_baseline.json`
under `[TST-4c]`'s one permitted reason: a new specification revision opened
the gap. `E3105` is registered in `ember_diag` ahead of its emitter so that
`[DIA-6a]` holds. `[FN-1a]` has its case today because the behaviour already
existed.

---

## Change log — 0.8.4_Hardened_2

**Hardened_2 is one editorial repair on Hardened_1. No rule changes meaning and
the accepted program set is identical.**

**E5 — `[EFF-18]`'s effect set gains `Nondet`.** Part X §1 defines the set with
ten members and carries a table row for the effect; `[DET-1]` contracts over it
and `[DET-2]` enumerates its sources exhaustively. `[EFF-18]` was written for
0.6, which added `Io` and `Lock`; `Nondet` arrived in 0.6.3 with `[DET-*]`,
which added it to X.1 without revisiting `[EFF-18]`'s parenthetical list. That
list is not exclusive by its own sentence — it "does not remove an effect
previously attached to any operation; it refines the effect model" — so adding
the missing member brings a stale enumeration in line with the section that
defines the set. It admits no new program and forbids no old one. ERR-028
recorded the decision; `docs/spec-amendments.md` E5 carries the reasoning.

**Why a hardening and not a revision.** The owner ruled explicitly: E5 is an
editorial repair, so the hardening number moves and the language version does
not. Between Hardened_1 and this cut the file carried the provisional header
`0.8.4_Hardened_1 + E5, pending a version decision` — the `207c69f` shape,
stating what the file was rather than claiming to be a cut it no longer
matched. That wording is now gone: the file is Hardened_2 and says so.

**Nothing else moved.** ERR-042 was inventoried in the same pass and produced
no edit to this document — see `docs/spec-errata.md`, where the entry is
withdrawn.

---

## Change log — 0.8.4_Hardened_1

**0.8.4 is one semantic change; Hardened_1 is everything else.**

The semantic change is **S1**, the owner's resolution of ERR-044: `[TYP-15]`
forbade a view in any storage with no bounding region, `[LT-3]` said a `str`
literal may be stored in a class field because its region is `static`, and
`[TYP-15]`'s own principle — a view may be stored where its region outlives the
destination — sided with `[LT-3]`. The enumeration was what overreached. It is
additive: no 0.8.3 program becomes invalid, and `[TYP-15a]`'s prohibition on
owning containers at view types is untouched. That change, and only that change,
is why the language number moved.

Everything below is the hardening, and of it: **no rule changed meaning, no
feature was added.** Every entry closes a gap found while building
the compiler against 0.8.3 — a mechanism a rule states without saying how, a name a
rule uses and never declares, a production for syntax the document already writes,
or an editorial instruction pasted in instead of carried out.

Each edit is marked where it sits — *(clarified …)*, *(head recovered verbatim …)*,
*(editorial instruction carried out …)*, *(0.6.2 leftover removed …)* — so a reader
can tell the owner's text from an implementer's addition without consulting anything
else.

**Every edit declares its class, and a hardening admits only four of the five:**

    SEMANTICALLY NEUTRAL CLARIFICATION   permitted
    IMPLEMENTATION INVARIANT             permitted
    SOURCE RECOVERY                      permitted
    EDITORIAL REPAIR                     permitted
    OWNER-APPROVED SEMANTIC CHANGE       NOT permitted — forces a language revision

The fifth is the line. The moment an edit answers *what Ember means* rather than *how
to implement what Ember already means*, it stops being a hardening. Two drafts of this
one crossed it and were withdrawn: A4, which wrote a determination about the historical pre-0.9.5 diagnostic `E3064` into
`[LT-2]`, and A6, which wrote an unruled reading of `[FN-1]` into the rule. Both are
open questions for the owner and neither side moves until they are answered. `docs/spec-amendments.md` records, for every entry, the defect its absence
caused and three flags: whether Ember's semantics changed (never), whether the
compiler had to move, and whether the text was found or written.

**Thirteen clarifications and one recovery.** Each clarification is an addition to an
existing rule and none coins a rule id.

*On the rule counts, which do not agree and should not be made to.* 0.8.3's own
change log speaks of **826 rules**; `tools/rule_index.py` reports **833**. They are
different measurements, not a discrepancy to reconcile: the tool counts every distinct
rule id appearing anywhere in the document, which includes ids only *cited* — 14 of
them, nine genuinely undefined (ERR-042) and three surviving only in an earlier
revision's change log — while 826 is the owner's count of normative rules at authoring
time, by a method not recorded here. Of the tool's 833, **819 are stated as rules**.
What matters for this hardening is that all three numbers are the same before and
after it, because it coined no id.

| # | Rule | What was missing | What its absence cost |
|---|---|---|---|
| A1 | `[SPN-1]` | that the coercion **takes a borrow**, and that the borrow must be explicit in an implementation's IR | a use-after-free reachable from Safe Ember: `v: Span[i32] = a` then `a.push(…)` compiled, the push reallocated, and `v` read freed memory |
| A2 | `[BRW-1]` | that a reference local is not re-seatable — `r = e` writes *through* it | a write through a shared `ref` passed every check and was caught only by the C backend emitting `const` |
| A3 | `[RNG-3]` | that `RangeError` is a prelude type, resolvable while signatures are collected | the rule's own worked example did not compile |
| A5 | `[CLO-3]` | that `fn(A) -> R` is a **bound**, not a representation, and MUST NOT be a function pointer | every capturing lambda was rejected |
| A7 | `[FFI-17d]` | any definition of `@ffi(no_virtual_dtor)` | an attribute named by a rule, cited to a rule about templates, and defined nowhere |
| A8 | `fn_header` | `["extern" string_lit]` | XVI.10's `pub extern "C" fn on_update(…)` did not parse |
| A9 | `extern_class` | the production | `[FFI-39]` rested on syntax Part III did not define |
| A10 | `item_body` | admitting A9 | — |
| A11 | `[THR-2]` | what `Sync` claims, as against what it is tested by | a rule that reads as a guarantee about mutation that it does not give |
| A12 | `[STD-8]` | that `a not in b` is one negation of one call | a second search and a second evaluation of the operands both admissible |
| A13 | `[CELL-2]` | that `Cell`, `RefCell` and `Arena` are one capability under three policies | three unrelated special cases in any implementation |
| A15 | `[BLD-3]` | what "flags" covers in the `.embind` cache key, and the test that generates the set | a binding surviving a configuration change that alters its ABI |

**One recovery.** `[RNG-8]` opened mid-sentence, on an ellipsis and a lowercase
"and", wrapped in quote marks — the only rule in the document with that shape, and
named in no change-log row, so a truncation rather than a deletion. Both 0.6 sources
carry it complete and identical, and they overlap the surviving text exactly at *"and
crosses an FFI boundary as its representation (`[FFI-5]`)"*, which locates the cut.
The recovered head — *"A range type is `Copy` when its representation is, has the
layout of its representation,"* — is restored **verbatim**, spliced to 0.8.3's own
revised tail at the words they already share. No 0.8.3 addition is disturbed.

**Four editorial instructions, carried out and deleted.** `[RNG-7]` (the niche
restriction, quoted rather than substituted), `[FFI-33b]` (when a foreign box records
its creating thread), `[FFI-2a]` (a literal *After "…", insert: "…"*), and `[BLD-2]`,
whose instruction was to add the `[verify]` package-config section to the cache key —
**not** carried out, because 0.6.2 removed that layer and performing it would have
resurrected it.

**Four leftovers of the contract and verification layer 0.6.2 removed**, and only
those: `RuntimeCheck` has the four kinds `[EFF-16]` assigns and `[EFF-22]` permits,
not five (`[EFF-18]`, `[EFF-17]`); `[STD-6]` loses the `verify` layer; `[UNS-7]`
loses its recommendation to carry `@requires` beside `@safety`. Ember's *current*
contracts — `@noalloc`, `@nosync`, `@noblock`, `@nopanic(explicit)` — are untouched.
Each removal is settled by the document's own closed owner questions, OQ-28..OQ-32.

**One keyword status.** `yield` appeared in both the v1 keyword table and the
reserved-for-future list. `[LEX-15b]` already makes it a v1 keyword and says its
count supersedes `[LEX-15]`'s, so the rule governed and the table is brought to
match: 49 entries.

**Deliberately not done, and recorded instead.** `[FN-1]` says a `mut` argument "MUST
be a mutable place" and Part VII's own worked example passes `buf.as_mut_span()`, a call
result — so read literally the document's own example is `E2140`, and `split_at` cannot
be called on its own result either. A reading that resolves it was drafted, and putting
it into the rule would have been this hardening answering *what Ember means* rather than
how to implement what it already means. The rule stands as written; the question is
ERR-041 and belongs to the owner. `[EFF-18]` stated the effect set
without `Nondet` at Hardened_1 time, while §X.1 — the section that defines the set — already included it, with a
table row defining the effect. That disagreement was recorded as an owner decision (ERR-028: X.1 governs) rather than a hardening, and the enumeration has since been brought in line with X.1; see `docs/spec-amendments.md`.
`[FFI-17]`'s numbered list contradicts XVI.7a's tables on `std::function` and
`std::optional<T>`; the list is already `NON-NORMATIVE` under `[CAT-1]` and the
document asks a future revision to delete the drifted claims — a deletion from owner
prose, which a hardening may not make. Nine rule ids are cited and defined by no
rule, the whole `IDE-*` family among them (ERR-042).

---

## Change log — 0.8.3

0.8.3 is a precision pass. No new language feature; seven items, each closing an
ambiguity that an implementer would otherwise have to resolve by guessing. It follows
a review whose central recommendation — freeze features, sharpen what exists, then
build — is adopted: the dominant risk has moved from whether the design works to
whether 826 rules can be implemented.

**This change log is exhaustive for normative text: a revision that alters a rule without a row
here is a defect.**

| # | Change | Where |
|---|---|---|
| 1 | **The reload transaction no longer has a hole.** `[HR-39]` permitted a `throws = "noexcept"` foreign call inside `migrate_from` and let it terminate the process, calling that "a known consequence". It was the wrong default — it opened by omission the one gap in a transaction whose entire value is having none. Such a call is now `E2227`, and `[HR-43]`'s `@allow_reload_terminate` is the explicit opt-in, reported by `ember tcb` beside the `unsafe` surface and counted rather than forbidden by `[GATE-3]`. | XVIII.4a |
| 2 | **Hot reload gains a memory model** (`[HR-42]`, `[HR-42a]`, new §XVIII.4b). `[HR-3]` established that no thread is inside Ember at COMMIT; that is a precondition, not an ordering. The depth counter is now the synchronising object — acquire on entry, release on exit, acquire scan then release publish on the reload side — which answers when new addresses become visible (at the thread's next entry), which threads may run (any not inside Ember), and what happens to a thread executing old code (it cannot exist; `[HR-3]` makes the scenario unreachable, not merely survivable). `[HR-42a]` forbids making entry take a lock. | XVIII.4b |
| 3 | **`str` membership is defined** (`[STD-8b]`). 0.8.2c said "`str` and `String`, as a substring test" and left `char in str` and `Span[u8] in str` open. Matching is by Unicode scalar: `Contains[char]` and `Contains[str]` only, `Span[u8] in str` is `E2226`, a `char` needle can never match a continuation byte and a `str` needle never a split codepoint. Byte search stays on `Span[u8]`, where the question is well-posed. | XV |
| 4 | **`Sync` says what it does not mean** (`[THR-7]`). It permits handles to cross threads and makes the count atomic; it does not make fields race-free. `[PHIL-10]`'s data-race guarantee holds because the ordinary exclusivity rules apply to a `Sync` class unchanged, not because `Sync` waives them — which is also why a `let` field is not exempt from the derivation. | XI |
| 5 | **`@nopanic(explicit)` says what it does not mean** (`[EFF-22]`). It forbids the panics the programmer writes and permits four `RuntimeCheck` kinds, every one of which can abort. The name is kept — renaming is churn against a distinction the effect set makes precisely — and the price is that every diagnostic naming it MUST state the permitted checks. | X |
| 6 | **The silence fallback no longer invites analogy.** "Closest to what a C programmer would expect" is removed as a rule for language semantics: two implementations reasoning independently from it reach two languages, and an analogy is not checkable. An implementation now records a provisional decision, marks it as a specification gap, and raises it. Simplest-and-most-predictable survives for the things genuinely left open — diagnostics' wording, internal structures, optimisation aggressiveness. | How to use this document |
| 7 | **The C backend does not define the language** (`[CMP-3]`). Where C cannot express a guarantee — aliasing, function identity across a reload, exclusivity, coroutine frames, `[COST-3]`'s elisions — the backend implements it by other means and MUST NOT narrow it. A rule may not be justified by "the C backend cannot do otherwise"; the correct outcome is a recorded gap and an LLVM-only capability, never a quieter guarantee. | XIX |

Not adopted: moving the change log's historical material out of this document. The
rationale prose inside rules could go to an ADR, but the exhaustive change log stays —
it is the mechanism that has caught the most defects here, including the `proven`
contradiction, which surfaced *because* a change-log row claimed something the body
did not do. A separate history file is optional reading, and optional is how that
stops working.

## Change log — 0.8.2c

0.8.2c finishes one operator the document had asserted without specifying. A syntax
review proposed adding `in`; checking found it already in VI.3's operator table and
**nowhere else** — `Contains` occurred exactly once in 5,269 lines, with no interface
declaration, no grammar production, no precedence, no `not in`, no cost row and no
diagnostic. An operator in the table and nothing behind it is worse than an absent
one, because a reader takes the table at its word.

**This change log is exhaustive for normative text: a revision that alters a rule without a row
here is a defect.**

| # | Change | Where |
|---|---|---|
| 1 | **`in` and `not in` are specified** (`[GRM-23]`, `[STD-8]`, `[STD-8a]`, `E2226`). A production at comparison precedence, non-associative — `a in b in c` is rejected, because Ember has no chained comparison and the Python reading would mislead. `not in` is one operator, so `x not in xs` cannot misparse as `(not x) in xs`. The two existing uses of the token, `for p in e` and `[RNG-1]`'s `type T = R in a ..= b`, are disambiguated by enclosing construct rather than by lookahead. | VI.3, III, XV |
| 2 | **`Contains` is declared and its cost is disclosed** (`[STD-8]`, `[STD-8a]`, and a `[COST-3]` row as `[COST-5]` requires). `x in coll` lowers to a **declared bound**, never to a compiler-synthesised scan: `Map`/`Set` answer in O(1) and by **key**, `Array`/`Span`/`str` in O(n), `ember inspect --cost` names which, and a type with no `Contains` is `E2226` rather than a silent linear walk. This is what keeps the operator from becoming the hidden cost the review's own §22.6 rejects. | XV, X.4 |
| 3 | **Rule-index correction.** `[GRM-20]`..`[GRM-23]` live in Part V beside the declaration rules they constrain, while the index listed `GRM` as Part III only. The index now says both. This drift arrived with 0.6.3's coroutine productions and had gone unnoticed through five revisions. | XXIII.4 |
| 4 | Not changed: `from … import` (already present, with `as` renaming and `*`), `with … as` (Ember binds with `=`; a respelling would break every existing example), and comprehensions (the review defers them itself, correctly). The review's §23 "recommended baseline" was not adopted as written: `player: mut Player` is `E2020` and contradicts the same document's §4, `string` is not an Ember type (`[TXT-1]`), `with lock = … as guard:` mixes two forms, and `if Some(p) = …` has no if-let to parse into — pasted into this document, `[TST-7]` would reject the block. Its `-> void` was correct and my objection to it was wrong: Part IV names `void` the unit type with value `()`. | — |

## Change log — 0.8.2b

0.8.2b repairs three contradictions in the C++ boundary, all three of which were
one fact stated in two places with one copy left stale, and adds the evidence the
C++ design has never had. Two of the three were introduced by earlier revisions of
this document fixing one copy and not the other.

**This change log is exhaustive for normative text: a revision that alters a rule without a row
here is a defect.**

| # | Change | Where |
|---|---|---|
| 1 | **`std::string_view` no longer maps to `str` in two different ways.** `[FFI-17a]`'s table said `Span[u8]`; `[FFI-17]` item 3 still said `str`. 0.7.2 fixed the table and left the prose, so the exact hole `[TXT-2]` exists to close — foreign bytes acquiring Ember's UTF-8 invariant without validation — was still open in one copy. Item 3 now says `Span[u8]`, and `CppString.as_str()` is corrected to the fallible `.to_str()`. | XVI.7 |
| 2 | **C++ inheritance is no longer both supported and unsupported.** `[FFI-17]` item 5 read "Ember cannot subclass C++ classes in v1" and dated the trampoline to v2, contradicting `[FFI-39]`'s forty lines of v1 specification. 0.7.2 fixed the copy in XXIII.2's non-goals and missed this one. Item 5 now defers to `[FFI-39]` and states what remains unsupported. | XVI.7 |
| 3 | **The `proven` grade is reinstated in the record, because it never left.** The 0.6.2 change log said the grade went with the prover, while the grammar production, `[TCB-1]`, `[FFI-37*]` and `ember tcb`'s own sample output all kept using it. The change-log row was the false one: `[TCB-1]`'s `proven` means *discharged by analysis of the generated adapter*, which has nothing to do with the contract prover. The row is corrected, `proven` is defined precisely as non-prover evidence, and the grammar and the prose are required to agree. | Part 0, XX.6a |
| 4 | **`[FFI-17]`'s numbered list is demoted to `NON-NORMATIVE`.** It has now been the site of four contradictions with the rules beside it. A prose restatement of a rule drifts from the rule and nothing detects it; the list now says the rules govern, names them, and records that a future revision should delete the duplicated claims rather than keep two copies in step. | XVI.7 |
| 5 | **`instrumented` carries how completely the run observed the fact** (`[FFI-37f]`): observed, partially observed, unobserved, or structurally unavailable. The last covers a custom pool, a pre-installed arena, a statically linked allocator or a VMA allocation, and is reported as **evidence of a gap, not as evidence** — `[PHIL-5]` forbids treating "the run could not have falsified this" as support for it. | XVI.9 |
| 6 | **A normative C++ compatibility table** (`[FFI-44]`, new §XVI.7b): twenty-four constructs classified automatic / overlay / native island / rejected, each citing the rule that decides it. It answers "can Ember consume this header" without reading Part XVI. | XVI.7b |
| 7 | **The unsupported-construct list becomes a rule** (`[FFI-48]`). Demoting `[FFI-17]`'s prose in row 4 left its item 14 — the only statement of what C++ Ember refuses — without a normative home, a gap this revision created. `[FFI-48]` is now thirteen constructs, each with the reason it is unsupported and the thing to do instead, and `E5034` names the row. The 0.7 review had already found that the prose version listed six constructs while claiming fourteen, and gave a reason *and* a workaround for one of them. | XVI.7b |
| 8 | **A C++ importer corpus and migration gate** (`[CXX-1]`..`[CXX-7]`, new §XX.13). The corpus includes real RageV headers and a third-party header-only library, because a fixture the importer's author wrote tests the importer against its own assumptions; generated thunks are golden-tested; every entry runs under both compilers, both CRTs, RTTI on and off and both iterator-debug levels, and disagreement must fail deterministically rather than bind and differ at runtime. `[CXX-6]`'s nine conditions are incorporated into `[GATE-4]`. `[CXX-7]` makes a dependency's `noexcept` change an API change, since it changes the Ember signature. | XX.13, XXI.6 |

## Change log — 0.8.1

0.8.1 makes conformance coverage mean something, and repairs one rule whose text
carried an unapplied editorial instruction. Two rows; no semantics change.

**This change log is exhaustive for normative text: a revision that alters a rule without a row
here is a defect.**

| # | Change | Where |
|---|---|---|
| 1 | **A rule needs a test that fails, not only one that passes** (`[TST-4a]`..`[TST-4c]`). `[TST-4]` required a directory to exist; a directory with one happy-path case passes against a compiler that never enforces the rule, which is the likeliest way an implementation looks conforming and is not. Which rules need a reject case is decided mechanically from `[DIA-6a]`'s existing rule→code map — a rule naming a diagnostic needs a reject case per code, a rule naming none is waived automatically — so no list is maintained by hand. `[TST-4c]` ships a recorded baseline in `[TST-7]`'s idiom that may shrink and never grow, and `[GATE-1]` requires it empty for 1.0. This is the narrow form of the maturity review's traceability proposal; the eight-field matrix stays deferred, since its remaining fields point at the reference compiler's layout and `[CAT-3]` exists to keep the language rules uncoupled from it. | XX.5, XXI.6 |
| 2 | **`[TST-7]` repaired.** Its text ended with an unapplied RFC instruction — *"add to the permitted opt-out reasons: … Fence XVI.4's block … as ```ember,ignore"* — pasted in rather than carried out. The reason is now stated as a fourth permitted opt-out and the two blocks are described as carrying it. This is the third time this document family has shipped editorial text inside a normative rule (`[STD-7]` and `[FFI-17]` were the earlier two), which is why `[DIA-6a]`'s completeness pass should grow a check for imperative second-person prose in rule bodies. | XX.5 |

## Change log — 0.8

0.8 changes no semantics. Every addition states something the document already meant
but had left the reader to reconstruct, or turns an intention into a number. It adds
one diagnostic code and no language feature. The prompt for it was a maturity review
whose two highest-priority items were already present in 0.7.2 — the Safe Ember
invariant (`[PHIL-10]`) and ABI versioning (`[ABI-*]`) — and whose C++ migration
scorecard contradicted `[FFI-17a]` on four rows; what survived verification is below.

**This change log is exhaustive for normative text: a revision that alters a rule without a row
here is a defect.**

| # | Change | Where |
|---|---|---|
| 1 | **The 1.0 release gate is quantitative** (`[GATE-1]`..`[GATE-8a]`, new §XXI.6). Phase 8's exit criterion was one line — "full conformance + perf suite + two external packages" — with no numbers. Every row now names an instrument the document already specifies: `[TST-4]` coverage, `[DIA-6a]` both directions, fuzzing hours, differential execution against `[COST-3]`'s implementation-defined rows, `[BUD-2]`/`[BEN-1..8]`, a first-hour test on a clean machine, and a `[CONF-1]` declaration produced by the test run rather than written by hand. `[GATE-8a]` gives every open question a status, and an `open` one blocks 1.0. | XXI.6 |
| 2 | **"Zero-cost" is defined** (`[COST-1]`, new §X.4). It appeared in the document as a phrase and nowhere as a definition, which for a language whose first pillar is the speed of C is the first thing a reviewer challenges. The definition is per *use*, not per abstraction, and "the equivalent C" means C upholding the same invariant — so an emitted bounds check is the price of a guarantee, not a failure of the claim. | X.4 |
| 3 | **Every implicit cost is classified** (`[COST-2]`..`[COST-5]`). Five classes — guaranteed elidable, guaranteed required, conditionally elidable, implementation-defined, not observable — applied to twenty mechanisms from bounds checks to coroutine resumes to reload thunks. `ember inspect --cost` reports the resolved class per item, and `[COST-5]` makes a new implicit cost without a row a CI failure. | X.4 |
| 4 | **A storage-selection table** (`[SEL-1]`, `[SEL-2]`, new §IX.0). Twelve mechanisms with ownership, aliasing, destruction, thread model and when to reach for each. Part 0 row 2 makes storage the programmer's choice; this is the first place the document helps them make it. `[SEL-2]` and `E5065` make the `Shared[T]` ↔ `CppShared[T]` confusion a diagnostic, since the two use different reference counts and conflating them double-frees. | IX.0 |
| 5 | **The hot-reload failure matrix** (`[HR-41]`). Sixteen rows covering every way a reload can fail and what each leaves behind, including the two rows that read "cannot occur" — a panic in `migrate_from`, forbidden at compile time by `[HR-35]`, and a failure during COMMIT, forbidden by `[HR-2]`. It restates nine rules as one auditable table. | XVIII.5 |
| 6 | **Every rule carries a requirement category** (`[CAT-1]`..`[CAT-5]`, new §XXIII.5): language, ABI, toolchain, reference-implementation, RageV, or non-normative. This separates what Ember *means* from how this compiler happens to work, which matters the moment a second implementation exists. `[CAT-3]` flags a language rule justified by a reference-implementation one — either the language rule is wrong or its real reason is unstated. Categorisation is descriptive: it records what each rule already was. | XXIII.5 |
| 7 | Not taken from the review: its formal-guarantees section (`[PHIL-10]`/`[PHIL-11]`, added in 0.7.2), its guarantee matrix (the same two rules as a table), its ABI examples (`[ABI-1..5]`, 0.7.2), and its migration scorecard, which marked `shared_ptr → Shared`, `weak_ptr → Weak` and arbitrary templates as automatic against `[FFI-17a]` and `[FFI-17b]`. Its traceability proposal is a genuine extension of `[TST-4]`/`[DIA-6a]` and is deferred rather than rejected. | — |

## Change log — 0.7.2

0.7.2 is a correctness and coherence release. It closes one soundness hole that
0.7.1 asserted its way past, states the guarantee the rest of the document exists to
provide, and adds the small high-value items from the 0.7.2 feedback review. It adds
no new language feature and rejects the feedback's proposals to split the effect
lattice and to require an explicit access block for aliased class mutation — the
first would break `@deterministic`, `@nosync` and `[RC-1]`, and the second is the
ceremony this language exists to avoid.

**This change log is exhaustive for normative text: a revision that alters a rule without a row
here is a defect.**

| # | Change | Where |
|---|---|---|
| 1 | **The reload transaction failure model is fixed** (`[HR-34]`..`[HR-40]`, new §XVIII.4a). 0.7.1's `[HR-2]` promised that a failed reload leaves the process on the old image, while `[PAN-1]` makes panic **abort** and defers unwinding to v2, and `[ALC-1]`'s allocator panics on exhaustion — so a panic in `migrate_from` killed the process and the guarantee was unachievable. 0.7.2 makes PREPARE **incapable of panicking** rather than catching panics: every operation in it excludes the `Panic` effect or is fallible-by-value, allocation goes through `try_alloc` into a `ReloadTransactionArena` released wholesale on failure, and `migrate_from` returns `Result[Self, ReloadError]` with a panic-free body (`E2225`). No unwinder is introduced. | XVIII.4a, XVIII.1, XVIII.4 |
| 2 | **The Safe Ember invariant is stated** (`[PHIL-10]`, `[PHIL-11]`, new §I.3a). One paragraph saying what a program with no `unsafe` and no unbacked foreign fact cannot do, and one enumerating what remains possible — exhaustion, deadlock, logical races, cycles, foreign bugs, deliberate termination. The guarantee was previously distributed across ownership, borrowing, classes, effects, FFI and runtime checks and stated nowhere. | I.3a |
| 3 | **Conformance profiles** (`[CONF-1]`..`[CONF-6]`, new §XX.11): Core, Systems, Native, Dynamic, cumulative, declared by `ember --version`, and claimable only when every conformance test for the rules they name passes. This addresses the real process risk in a specification this size — not a wrong rule, but a partial implementation becoming the de facto language. | XX.11 |
| 4 | **ABI protocol versions** (`[ABI-1]`..`[ABI-5]`, new §XX.12). `EMBER_C_ABI`, `EMBER_RUNTIME_ABI`, `EMBER_RELOAD_ABI` and `EMBER_CPP_BINDING_ABI` are versioned independently and checked before any code runs — as a link error where the boundary links, and as `E9037` on image load where it does not. | XX.12 |
| 5 | **Every imported C++ function carries an explicit exception policy** (`[FFI-43]`, `[FFI-43a]`, new §XVI.7a). `throws = "translate"` produces `Result[T, CppError]`; `throws = "noexcept"` terminates at the boundary and is applied automatically to a header-declared `noexcept`. An unstated policy is `E5061` and a contradictory one `E5062`; there is no default, because a silent assumption here is undefined behaviour crossing an ABI. | XVI.7a |
| 6 | **A normative string and text model** (`[TXT-1]`..`[TXT-8]`, new §XV.4a). Four types, one UTF-8 guarantee held by two of them, and **no unvalidated route into `str`** (`E5063`) — `std::string` and `std::string_view` now map to `CppString` and `Span[u8]`, from which conversion is fallible. This closes the hole by which a `yaml-cpp` scalar, a `cgltf` name or an ImGui buffer entered safe Ember as a `str` and was walked by a UTF-8 decoder. Also fixes null termination, slicing at a codepoint boundary, stated conversion costs, and the C ABI representation. | XV.4a |
| 7 | **The leak reporter names the cycle** (`[WK-4]`, new §VIII.5a): the shortest strong cycle through each leaked object, as `Type.field` edges, with the suggested edge to weaken (`L3017`). `[WK-1]`'s statement that cycles leak is unchanged and no collector is added. | VIII.5a |
| 8 | **Unsafe blocks carry a machine-readable reason** (`[UNS-9]`, `[UNS-9a]`, new §IX.5a) from a closed set, aggregated by `ember tcb` (`L3018`). It changes no check and no codegen; it makes the unsafe surface of a large engine answerable. | IX.5a |
| 9 | **Non-goals contradiction repaired.** XXIII.2 still read "no C++ subclassing from Ember" while `[FFI-39]` specified it. The bounded form is now a goal; unrestricted subclassing — multiple inheritance, virtual bases, unnamed virtuals — remains a non-goal. | XXIII.2 |
| 10 | Codes added: `E2225`, `E5061`–`E5064`, `E9037`, `L3017`, `L3018`. Attribute added: `@ffi(throws = …)`. CLI added: `ember inspect --alloc`. | V §8, XX §1, XX §6 |

## Change log — 0.7.1

0.7.1 is 0.6.3 with hot reload rebuilt as a first-class language feature, the
compile-time budget adopted, and the C++ boundary completed. It takes the design of
the 0.7 draft for all three and none of its text: 0.7 was authored from v0.3 and
silently reverted 239 rules settled in 0.4 through 0.6.2, so its contribution is
carried onto this base rather than the other way round. Everything 0.6.3 guarantees
is unchanged, and no rule below rejects source that 0.6.3 accepted.

**This change log is exhaustive for normative text: a revision that alters a rule without a row
here is a defect.**

| # | Change | Where |
|---|---|---|
| 1 | **Hot reload is a Part, not a toolchain section** (new Part XVIII, `[HR-1]`..`[HR-33]`). 0.6.3's `[HOT-1]`..`[HOT-10]` are replaced entirely. The new part keeps 0.7's design decisions — package-granularity image swap over binary patching, name-keyed slots, refusal as a transactional invariant, live state migrated rather than serialised — and repairs the seven defects that made it unimplementable. It is host-agnostic: `[HR-30]` states a four-call contract any program can satisfy, and `ember run --hot` is one host among others rather than the mechanism. | XVIII |
| 2 | **Permanent thunks replace 0.7's `{table_id, slot}` function values** (`[HR-6]`, `[HR-6a]`). A reloadable function's address, everywhere it can be observed, is a fixed thunk address. `fn` values, closure code pointers, vtable slots, witness tables and `extern "C" fn` values therefore keep the representation Part IV §9 and `[OBJ-2]` already give them, and `std`, `ember_rt`, non-reloadable packages and foreign code need no knowledge that reload exists. This deletes an entire class of defect — 0.7's widened vtable slot was read as a raw code pointer by `std` and by the C11 runtime — and makes `[HR-21]`'s stable `@export` address fall out for free rather than needing a second mechanism. | XVIII.2 |
| 3 | **Migration is four phases, and only two of them can fail** (`[HR-2]`, `[HR-2a]`). PLAN decides every refusal from the schemas before anything is touched; PREPARE allocates and runs `migrate_from` without mutating, moving from, or dropping any live instance; COMMIT is specified as infallible and allocation-free; RECLAIM runs the drops afterwards. 0.7 asserted a transactional guarantee while specifying a single pass that ran user `drop` and `migrate_from` code with no rollback and no unwinding, so a panic partway through aborted having already destroyed part of the live set. | XVIII.1, XVIII.4 |
| 4 | **The safe point is a depth counter, not an attach flag** (`[HR-3]`, `[HR-3a]`). 0.7 reused `[FFI-22]`'s thread-attach state, which records whether a thread has *ever* entered Ember; in any host with a long-lived render or worker thread it is permanently non-zero and reload never fires. Ember-spawned threads and job workers are registered too. | XVIII.1 |
| 5 | **Schemas cover enum variants, statics' initialisers, and recursion** (`[HR-11]`, `[HR-11a]`, `[HR-14]`, `[HR-17a]`). 0.7's schema recorded fields only, so no enum change was representable and no rule renumbered a stored discriminant; it also kept a static's old value when the initialiser changed, silently discarding the edit. Range-typed fields are refused rather than migrated, because `[RNG-10]`'s construction set is closed and migration is not in it. | XVIII.3, XVIII.4 |
| 6 | **Relocation covers `Weak[C]` and registered interior references** (`[HR-15]`), and **`std` containers over reloadable element types migrate** (`[HR-13a]`) — 0.7's rule could not fire, because `std` is not reloadable and the registration was placed there rather than in the instantiating package. | XVIII.3, XVIII.4 |
| 7 | **The 40-byte header is a whole-process property with a link-time guard** (`[HR-12]`, `[HR-12a]`), and **the runtime is shared, not duplicated per image** (`[HR-29]`). 0.7 made the header a per-package key with no guard, so two packages in one process disagreed about field offsets; and it never said where `ember_rt` lives, so a reloadable `cdylib` statically linking its own copy would load with an empty live set and a separate heap. | XVIII.3, XVIII.9 |
| 8 | **Bodies-only is a specified tier** (`[HR-19]`), carrying forward what 0.6.3's now-deleted bodies-only rule guaranteed: no schemas, no live-instance list, the 24-byte header, and every non-body change refused. It is what Phase 7a ships before migration is trusted. `[HR-10a]` also drops 0.7's rule that a `@noreload` function may not call a reloadable one, which rejected the ordinary shape of an opted-out hot loop and was unsatisfiable for `std` generic instances. | XVIII.5, XVIII.2 |
| 9 | **The compile-time budget is adopted, with the gates made measurable** (`[BUD-1]`..`[BUD-6]`, XX). Absolute and regression gates are separated and both stated; CI normalises against a calibration workload instead of gating a laptop figure on a hosted runner; a run whose own spread exceeds the gate is inconclusive rather than failing; `bench/bigpkg` grows with the compiler so Phase 0 does not gate against a package it cannot build; `[BUD-5]` names one row (B3) instead of three; and `[BUD-5a]` applies the rule to hot reload and interop themselves. | XX |
| 10 | **A foreign base can be inherited** (`[FFI-39]`..`[FFI-39e]`, `[FFI-17c]`, `[FFI-17d]`). This is 0.7's headline interop feature and the real gap in 0.6.3. The repairs: an `@ffi(trampoline)` `extern class` is a declared, sized, implicitly `open` base that satisfies `[CLS-4]` rather than an opaque unsized `extern type`; `super.init` selects a C++ base constructor, so a base like `Layer(const std::string&)` is constructible; destruction is derived-first per `[CLS-6]` rather than 0.7's inverted order; ownership is declared in both directions rather than fixed at "Ember owns", which is what makes `PushLayer(Layer*)` expressible; re-entrant callbacks do not trip `[EXC-1]`; and the inbound path has a panic boundary. | XVI.7 |
| 11 | **Member mapping completed** (`[FFI-40]`..`[FFI-42a]`): `this`-qualifiers and reference mapping, statics, nested types and namespaces, operators, and `Iterable` from `begin`/`end`. Iterator invalidation is a `MUST` with a `ref mut` borrow rather than 0.7's `SHOULD`, which left a use-after-free reachable from safe Ember; and a `const` member of a type with `mutable` state is assumed to invalidate unless the overlay says otherwise, because `const` is not an aliasing guarantee in C++. | XVI.7 |
| 12 | **Cross-language LTO and a generated C++ header** (`[BLD-FFI-4]`, `[BLD-FFI-5]`). `[BLD-FFI-5a]` generates a copyable C++ smart handle only for a `Sync` class; 0.7 generated one for every class, inlining a non-atomic refcount that any C++ worker thread could tear. | XVI.11 |
| 13 | Not taken from 0.7: its FFI section (49 rules short of this one, and missing `[FFI-35]`/`[FFI-36]`/`adopt`, the trust grades and `[TCB-*]`), its open-question list (which re-asks ten decisions settled in 0.4/0.5), and its 24-byte-header, range-type, `[CG-C-*]`, `[DIA-*]` and `[BEN-*]` regressions. `[FFI-17e]`, `[FFI-29]` and `[FFI-5a]` already covered 0.7's bridge-type declarations, header-only shims and layout verification, and cover them better. | — |
| 14 | Attributes added: `@reloadable`, `@noreload`, `@renamed_from`, `@reinit_on_reload`, and `trampoline`/`virtuals`/`owner`/`invalidates` as arguments of `@ffi`. Codes added: `E2224`, `E5056`–`E5060`, `E9035`, `E9036`, `W5033`, `W5034`, `L2005`. `[MAN-6]` gains `[MAN-7]`. | V §8, XX §6 |

## Change log — 0.6.3

0.6.3 adds four features aimed at the day-to-day experience of writing a game in
Ember rather than at the safety model: the iteration loop (hot reload), the
readability of frame-spanning gameplay code (coroutines), the guarantee lockstep
networking and replay need (determinism), and visibility plus control over what
monomorphisation costs (the instantiation budget). It removes no guarantee,
reopens no confirmed ADR. It rejects 0.6.2 source in exactly one way, recorded in
row 2 and nowhere else: `yield` becomes a reserved word, so a 0.6.2 program using it
as an identifier no longer parses. Per the 0.6 convention for source-rejecting
changes, the diagnostic (`E0104`) MUST name `r#yield` as the mechanical fix and
`ember fmt --migrate` MUST apply it.

**This change log is exhaustive for normative text: a revision that alters a rule without a row
here is a defect.**

| # | Change | Where |
|---|---|---|
| 1 | **Hot reload** (`[HOT-1]`..`[HOT-10]`, new section XX.11). `ember run --hot` swaps changed function bodies into a running process at a declared quiescent point. Bodies only: any change to a signature, layout, effect set, export set or `const` is rejected with `E9030` and the process keeps running the old code, which is what lets `[HOT-5]` promise that live state — class instances, refcounts, arenas, ECS storage, open resources — survives untouched. Reloadability is opt-in per module (`@hot_reload`), so the call indirection `[HOT-3]` needs is paid only where iteration speed is wanted, and exists in no profile but `dev`. | XX.11, XX.4 |
| 2 | **Coroutines** (`[CORO-1]`..`[CORO-11]`, new section VI.5a). `gen fn` and `yield` turn a frame-spanning sequence into straight-line code. The frame is a compile-time-sized value, so **nothing about a coroutine allocates** and a `gen fn` may be called from `@noalloc` (`[CORO-5]`). One restriction carries the safety argument: no borrow may be held across a `yield` (`[CORO-6]`, `E2221`). `yield` becomes fully reserved and `gen` contextual, taking `[LEX-15]`'s reserved set from 48 to 49 (`[LEX-15b]`). | VI.5a, II, III, XIX.4.10a |
| 3 | **Determinism** (`[DET-1]`..`[DET-9]`, new section X.2a). `@deterministic` is a hard contract over a new `Nondet` effect, in the manner of `@noalloc` — it constrains results, not timing (`[DET-6]`). `std.math.det` supplies bit-reproducible transcendentals (`[DET-4]`). `[BLD-13]` makes the build itself reproducible and gives lockstep peers a `--build-id` to compare, which is what `[DET-7]`'s cross-machine claim actually rests on. | X.1, X.2a, XX.3 |
| 4 | **Instantiation budget and shared instantiation** (`[MONO-2]`..`[MONO-9]`, new section XIX.4.11a). The compiler counts instantiations per generic and reports them (`ember build --report=instantiations`); a manifest ceiling makes crossing it `W2220` on the commit that crossed it. Where a generic uses `T` only to call its bounds' methods, the compiler may emit one shared function plus a witness table per type instead of one function per type — `[TYP-22]`'s `dyn`, chosen by the compiler from source that did not write it. `@always_specialize` / `@never_specialize` settle it per generic. `[PRF-1]` holds throughout: this changes what is emitted, never what is computed. | XIX.4.11a, XX.2 |
| 5 | **`[TYP-16]` is amended** to point at XIX.4.11a. Its existing sentence — code size is the programmer's responsibility — stands; 0.6.3 gives the programmer the instrument that sentence assumes. | IV.7 |
| 6 | Attributes added: `@deterministic`, `@hot_reload`, `@always_specialize`, `@never_specialize`, and `deterministic` as an argument of `@ffi`. Codes added: `E2220`–`E2223`, `E4070`–`E4073`, `E9030`–`E9034`, `W2220`, `W9030`, `L2004`. | V §8, XX.6 |

## Change log — 0.6.2

0.6.2 removes the contract and verification layer and finishes the C++ boundary. It is the
correctness pass over 0.6 (all of which is retained below) plus one scope decision.

**This change log is exhaustive for normative text: a revision that alters a rule without a row
here is a defect.**

| # | Change | Where |
|---|---|---|
| 1 | **Contracts and verification are removed.** `@requires`, `@ensures`, `@invariant`, `@decreases`, `@verified`, `@assume`, the `[CTR-*]` and `[PRV-*]` rules, sections X.2a and X.2b, proof manifests, the `--contracts` and `--verify` flags, and the `proven` trust grade all go. Ember follows the same approach as Rust: a rule is enforced by making the bad value unconstructible, not by writing the rule down and checking it. `Option`, `Result`, `.get(i)`, `NonZero[T]` and range types are that mechanism, and they are all retained. Range types (`[RNG-*]`) are unaffected — they are a type, not a contract. | X, XX |
| 2 | **The trusted-base report survives the removal.** `ember tcb` still lists unsafe blocks, foreign calls, and every foreign fact with its grade. **This row over-claimed and is corrected in 0.8.2b**: the grade set remains four — `asserted`, `checked`, `instrumented`, `proven` — because `[TCB-1]`'s `proven` never meant the contract prover. It means the fact is discharged by analysis of the *generated adapter*, which survives the prover's removal untouched. What went away with the prover is the contract-level `proven` required the prover. This is the audit surface for a C++ migration and never depended on verification. | XX.6a |
| 3 | **The standard-library mapping is stated and prioritised** (`[FFI-17a]`). `std::span` and `std::string_view` are **P0** because they appear in the signature of almost every modern C++ API: an importer that handles `std::vector` first still cannot import the header. `std::optional` maps to `Option[T]`, `std::variant` to a generated `enum`. `std::shared_ptr`/`std::weak_ptr` map to `CppShared[T]`/`CppWeak[T]` and explicitly **not** to Ember's `Shared[T]`/`Weak[T]`, which use Ember's own reference count — conflating them would double-free. | XVI.7 |
| 4 | **Templates are importable only as explicit instantiations** (`[FFI-17b]`), so libclang can supply a concrete layout and mangled name. Passing an Ember generic into a C++ template is out of scope (`E5055`). | XVI.7 |
| 5 | **`[FFI-17]` repaired.** An editorial fragment ("item 3, insert before `std::vector`") had been pasted into the rule text. | XVI.7 |
| 6 | **`OQ-28`..`OQ-32` are closed** by the removal — every one of them was a question about how contracts behave. | XXIII.1 |

## Change log — 0.6

0.6 adds a verification layer and strengthens the foreign boundary. It removes no
guarantee and reopens no confirmed ADR. Some changes are deliberate **safety-tightening
changes** and can reject source that a v0.5 implementation previously accepted; those
changes are listed below and MUST provide a diagnostic and migration path. Apart from
those explicitly listed changes, v0.5 source and semantics remain compatible. New v0.6
features are opt-in and do not appear by accident.

| # | What | Where |
|---|---|---|
| 1 | **Range and domain types.** A nominal numeric type carrying its own bounds. `type Roughness = f32 in 0.0 ..= 1.0`, distinct from every other type with the same representation. | IV.2a, `[RNG-1..7]` |
| 5 | **Trusted-base report with graded claims.** Every foreign fact is *asserted*, *checked*, *instrumented* or *proven*, and every unproven assumption is listed. | XX.6a, `[TCB-1..4]` |
| 6 | **A standard library that can be built without its allocating half**, plus fixed-capacity containers in `core`. | XV, `[STD-6..7]`, `[BLD-11..12]` |
| 7 | **`Io` and `Lock` as separate effects**, and `@realtime` as a marker for a configured set. | X.1, `[EFF-18..19]` |
| 8 | **Foreign references and pointers import unsafe** until a lifetime is written down; `borrowed` becomes the promotion, not the default. | `[FFI-35]` |
| 9 | **Ownership transfer across the C++ boundary is an explicit `adopt` at the call site.** | `[FFI-36]` |
| 10 | **Effect claims on foreign functions are checked by instrumentation**, not believed. | `[FFI-37]`, `[CLI-14]` |
| 11 | **Foreign reachability reporting** — which foreign functions a given function can reach. | `[CLI-13]` |

**Three decisions 0.6 takes, and what they protect:**

2. **A range type is declared with `in`, not a new `range` keyword.** `in` is
   already reserved; `range` is a plausible variable name. `[RNG-1]` states the
   nominal/transparent split in one sentence so `type` remains readable.

**What 0.6 does not do:** it does not make verification mandatory, does not add a
second profile axis (`[PRV-8]` makes verification a flag orthogonal to
`debug`/`release`/`shipping`), does not weaken any check a proof discharges in a
build that is not proving, and does not restate the C++ import machinery of
XVI.2–XVI.11, which is unchanged except where `[FFI-34..38]` amend it by id.

## Change log — 0.5

0.5 answers every question 0.4 put to the owner. It adds three rules, promotes three reserved
identifiers to normative text, and converts one open ambiguity into a decision. It removes no
guarantee and reopens no confirmed ADR.

**This change log is exhaustive for normative text: a revision that alters a rule without a row
here is a defect.**

| # | Change | Where |
|---|---|---|
| 1 | **All fifteen open questions answered.** Part XXIII.1 becomes a decision record; `OQ-1`..`OQ-23` keep their permanent identifiers. No question is left for an implementing agent to decide silently. | XXIII.1 |
| 2 | `[EXC-4]` **resolved** (`OQ-18`): `let` freezes the binding, not the value, so a long-term access to a `let` field registers exactly as any other field does and only an instantaneous read is exempt. This closes the aliasing hole the unconditional exemption left open, and `[CLS-9a]` states the mutate-through consequence. | VIII.3, V.5 |
| 3 | `[LT-7]` **adopted** (`OQ-17`): late-bound callback regions, one level of higher-ranked quantification at callback boundaries, with region variables compiler-internal and absent from source, generic arguments and ABI-visible names. `[THR-5]` and `[JOB-2]` stop being bespoke exceptions, and `[GPU-9]`'s `pass.native` becomes expressible. | VII.5 |
| 4 | `[TYP-15a]` **added** (`OQ-19`): `BorrowList[T]`/`ViewList[T]` may hold views under one inferred region, which the programmer never writes; arbitrary owning containers at view types stay rejected, and the new containers may not smuggle a view past `[TYP-15]`. | IV.4 |
| 5 | `[FFI-30c]` **added**: distributable, composable overlays with declared left-to-right order, explicit `override`, diagnosed conflicts, foreign type identity held stable, and the composed overlay identity folded into build invalidation. | XVI.4 |
| 6 | `Cell` and `RefCell` join the **prelude** (`OQ-10`), because `Shared[T]` was already there and heavier on every axis measured. `[DIA-9]` still bars them as a first suggestion. | IX.7, XV |
| 7 | **Call sites never write a mode** (`OQ-13`, `[FN-2a]`). Part 0 row 3's guarantee is preserved verbatim; the mode is read from the signature and surfaced by `ember inspect` and `[IDE-7]` inlay hints. | V.2 |
| 8 | `::` **kept** (`OQ-15`) with its purpose written down: the qualified module/type/namespace path separator, distinct from `.` for instance and member access. | III.5 |
| 9 | **No fixed exclusivity cost is normative** (`OQ-16`). The "~2 ns" figure is withdrawn as a promise and re-derived under `[BEN-1]`–`[BEN-7]`, because `[EXC-3]`, `[EXC-6]` and `[FFI-33]` all add work to that path. ADR-004's decision is unchanged. | VIII.3, I.4 |
| 10 | `[CLI-10]` `--report=engine` is a **reporting mode, not a profile**: it may not change type checking, acceptance, semantics or optimisation legality. `[PRF-1]` governs profiles and this is not one. | XX.1 |
| 11 | The 1.0 promises become **normative rather than intent** (`OQ-22`), and the bundled Clang + `lld` toolchain becomes a **committed deliverable** (`OQ-23`, `[TOOL-2]`). | XX.2, XX.1 |
| 12 | **Rule-ID uniqueness is a hard invariant.** `tools/rule_index.py` fails CI when an id is defined twice in the active index; a reference is not a definition. This is the defect class that reached three drafts of the 0.5 proposal undetected. | XXIII.4 |
| 13 | `[TST-11]` records the **v0.5 regression obligations** for `[EXC-4]`, `[LT-7]`, `[TYP-15]`/`[TYP-15a]`, `[FFI-30c]`, `[EFF-17]` and `[PRF-1]`, so every rule this revision adds or changes has a conformance mapping. | XX.5 |
| 17 | `;` is **no longer a statement separator** (`OQ-25`, ERR-003). `simple_stmt` becomes `small_stmt`, `a = 1; b = 2` is `E0105`, and one line carries one statement. `;` stays punctuation only inside `[T; N]` and `[v; N]`. `[LEX-9]` and `[FMT-3]` follow. | III.4, II.2, XX.7 |
| 16 | `1f32` is **confirmed legal** (`OQ-24`, ERR-002), alongside `1.0f32`. `1.f32` remains a method/field access on `1`, not a literal. | II.5 |
| 15 | `let` is **fully reserved** (`OQ-26`, ERR-004): it joins the reserved keyword set, which grows from 47 to 48 entries, and `r#let` is required to use the word as a name. `ember_lexer`'s keyword-count assertion moves from 47 to 48: `type` joins the set under ERR-009 and `from` leaves it under ERR-017. | II.4 |
| 14 | The **GPU host model is scheduled, not respecified**: Part XVII's `[GPU-*]` and Part XXII's `[RV-*]` remain the sole normative source, their implementation moves to v0.6, and v0.5 carries an explicit compatibility surface it may not break. The MIR interpreter becomes a supported restricted execution mode with mandatory differential testing. | XXI.2 |

## Change log — 0.4

0.4 makes the document agree with itself and with the compiler that implements it. It adds no pillar and
removes none: every change is a reconciliation, a contradiction closed, a memory-safety hole in Safe code,
or a rule this document already implies and failed to state.

**This change log is exhaustive for normative text: a revision that alters a rule without a row here is a
defect.** 0.3 altered eight sites without a row, which is why the sentence is now normative.

| # | Change | Where |
|---|---|---|
| 0 | Carries the owner rulings ERR-001, ERR-005, ERR-006 and ERR-007 recorded in `docs/spec-errata.md`, which 0.3 reverted by being authored from an unpatched copy. The single-file source under `docs/spec-source/` is now stated to be normative and `docs/spec/` generated. | II.3, III.5, III.7, XX.5, XXI.1, XXI.3 |
| 1 | The three still-open errata are written into the grammar: `;` joins the punctuation table (ERR-003), `let` joins the contextual keywords (ERR-004), `1f32` joins `float_lit` (ERR-002). Each is marked "implementation follows, owner ruling pending". | II.4, II.5, II.6 |
| 2 | Registry hygiene: the attribute table gains every attribute the document uses (`@static_safe`, `@borrows`, `@allow`, `@must_drop`, `@fp`, and the reserved forms); the rule index admits amendment and hyphenated ids and gains six prefixes; `type` becomes a v1 keyword. | III.7, II.4, XXIII.4 |
| 3 | Contract effect sets are computed once under a fixed **contract profile** (`[EFF-15]`), so `@static_safe` no longer means one thing in `release` and the opposite in `shipping`. | X.1, X.2 |
| 4 | `[DSJ-6]` and `[EFF-12]` are made consistent: a check that *establishes* a static fact carries the new reason code `establishes_static_fact` and does not violate `@static_safe`. | IX.8, X.1, X.2 |
| 5 | `exclusivity = "unchecked"` reaches dynamic **class** exclusivity and nothing else; `[CELL-9]` keeps `RefCell`'s check in every profile, resolving the contradiction with XXIII.2's own new non-goal. | VIII.3, IX.7, XX.4, XXIII.2 |
| 6 | Alias facts are derived, never assumed, and `[CG-C-4]` states how they reach the backend — without which `[SIMD-3]` and `[DSJ-3]` buy nothing. | IX.8, XII.2, XIX.6 |
| 7 | Seven memory-safety holes in Safe code are closed: exclusivity elision only over a closed interval (`[EXC-3]`), resurrection during `drop` (`[OBJ-5]`, `[WK-3]`), borrows keeping objects alive (`[RC-5]`), leakable scope guards (`[THR-6]`, `@must_drop`), `@parallel` disjointness as a property of the place (`[PAR-2]`), `ScopedArena` rewind (`[ARN-7]`), and what `let` exempts (`[EXC-4]`, pending `OQ-18`). | VIII, IX.2, XI |
| 8 | The escape hatches the vision rests on are made usable: `@borrows` is registered and given a position (`[LT-1a]`), `?` can propagate an error of its own type (`[ERR-7]`), type arguments are admitted in expression position (`[GRM-8a]`–`[GRM-8c]`), and once-callable closures are selected by the existing parameter mode (`[CLO-6]`). | III, IV, VI, VII, XIII |
| 9 | The C backend gets the four things "speed of C" requires: cross-translation-unit inlining (`[CG-C-3]`), loop bounds-check versioning (`[OPT-2]`), guaranteed vectorisable loop shape (`[SIMD-5]`, `[CG-C-6]`), and enforced float control (`[TYP-9a]`–`[TYP-9c]`). | XIX.6, VIII.6, XII, IV.2 |
| 10 | Grammar repairs for constructs the document uses and Part III did not define, and a rule that every fenced `ember` block in this specification is compile-checked (`[TST-7]`). | III, XX.5 |
| 11 | The errors of the first hour get a mandated catalogue of their own (§XX.6.2, shapes N1–N12), ownership shapes O5–O9, A1, B12, B13 are added, and `E3060` is split so one code no longer means two things. | XX.6.1, XX.6.2 |
| 12 | The editor becomes an architectural constraint rather than a later rewrite: an owned `Session` replacing the leaked interner, error tolerance past the parser, and item-granular re-checking (`[IDE-3]`, `[IDE-4]`, `[IDE-6]`, `[BLD-7]`–`[BLD-10]`); a new XX §10 reserves the server itself. | XIX.1, XX.10 |
| 13 | Phase-5 completeness: the C importer imports what real headers contain (`[FFI-6]`, `[FFI-8]`), a generated shim translation unit handles `static inline` and single-header libraries (`[FFI-29]`), foreign contracts carry a count axis and an `unsafe overlay` boundary (`[FFI-11]`, `[FFI-2a]`, `[TIER-1]`), and imported entities have one identity (`[FFI-30]`). | XVI |
| 14 | The plan gains instruments that measure the **user** rather than the compiler: a benchmark protocol (`[BEN-1]`), a first-run milestone (`[TOOL-1]`–`[TOOL-4]`), and a corpus written by people who have not read this document (`[TST-8]`–`[TST-10]`). | XX.1, XXI.3, XXI.4 |
| 15 | Part XXIII.1 is renumbered once, permanently, under stable `OQ-n` identifiers, and twelve new questions are recorded rather than answered. | XXIII.1 |

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
| 9 | Diagnostic shapes **B11** (disjointness not provable) and **S1** (`@static_safe` violated), plus `[DIA-11]`: when a check's reason is `not_provable_in_principle`, the suggestion must be to drop the contract or change the data structure, never to restructure. | XX.6.1 |
| 10 | Non-goals extended: no profile-dependent assumptions, **no cycle collector** (it is a tracing collector over the RC subgraph and reintroduces what Part 0 removed). | XXIII.2 |

Considered and rejected for 0.3: a fourth safety tier separating "statically proven" from "runtime enforced" — the existing tiers describe what the programmer *writes* (Safe / Contract / Unsafe), while static-vs-runtime describes how the compiler *enforces* within Safe; conflating them would imply a choice the programmer does not make. And limited RC cycle reclamation, per non-goal 10.

Every RageV-class requirement — first-class lifetime domains, borrow ergonomics for renderer code, command-scoped GPU ownership, deferred destruction, temporal history as a resource class, shader-language independence, native islands for backends and platform code, zero-cost FFI facades, transitive performance contracts, and visible allocation/effect decisions — is expressed with these primitives rather than as special cases. Part XXII.6 lists each requirement with the mechanism that satisfies it.

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
* `[PHIL-8a]` **The ladder is enforced on every revision.** Every error code that rejects a program a previous language version accepted MUST have a diagnostic shape in §XX.6.1 or §XX.6.2 whose mandated `help`, applied literally to the rejected program, produces a program that compiles. `tools/rule_index.py` MUST verify this mechanically over `tests/ui/`: for each shape, the recorded "before" program fails with that code and the "after" program — obtained by applying the mandated fix — compiles. A shape whose fix does not compile is a defect of the same kind as `[DIA-11]`'s.

`[PHIL-8]` is the sentence that distinguishes Ember from a language whose only safety mechanism is static proof. It is a rule about **how** a guarantee is met, not about *which* guarantees hold: the guarantee list never shrinks, and no profile, contract or attribute weakens it outside `unsafe`.


## I.3a The Safe Ember invariant

The guarantee Safe Ember makes is distributed across ownership, borrowing, classes,
effects, FFI and runtime checks. This section states it once, so that every other
rule can be read as a means to it.

* `[PHIL-10]` **The invariant.** A program that contains no `unsafe` block, no
  `unsafe fn`, and no unbacked foreign fact (`[FFI-35]`) cannot perform an invalid
  ownership operation, a use-after-free, a double release, a data race, an
  out-of-bounds access, an invalid range construction (`[RNG-9]`), a read of
  uninitialised memory, or an access through an invalid reference. Foreign code and
  explicitly `unsafe` operations lie outside the guarantee and are isolated by
  typed boundaries whose obligations `[UNS-7]` and `[FFI-35]` require to be
  written down and `ember tcb` requires to be enumerable.
* `[PHIL-11]` **What remains possible**, and is therefore not a defect in the
  guarantee: resource exhaustion (`[ALC-1]` panics on it); deadlock (`[THR-*]` does
  not prove liveness); a logical race between correctly synchronised operations;
  a reference cycle leaking (`[WK-1]`); a bug in a foreign library reached through
  a correctly declared boundary; a hardware fault; deliberate process termination
  through `panic`/`abort`; and any violation of a safety obligation inside an
  `unsafe` block or an inaccurate `asserted` foreign fact. `[PHIL-5]` governs the
  last of these: declining to look is not a proof, and an unbacked assertion is
  recorded as unbacked rather than treated as discharged.

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

`[PHIL-9]` **Why class instances can carry dynamic checks and value types cannot.** A class instance already has a 24-byte header (`[OBJ-1]`) holding its reference counts and type information; the existing header includes its access-state word within the fixed 24-byte header layout, and the header is invisible to foreign code because handles are always passed as opaque pointers. A `struct` has no header by design: `[TYP-11]` guarantees C-compatible layout so that every plain struct can cross an FFI boundary unchanged, and `size_of`, `align_of` and field offsets are exactly what a C compiler would produce. Adding hidden runtime state to value types would change `size_of[T]()`, break `@layout(c)`, invalidate every `[FFI-5]` layout assertion, and make `SoA` columns and GPU-uploaded buffers no longer bit-identical to their C counterparts. The dichotomy is therefore forced by the FFI and layout guarantees, not by a judgement that one kind of code deserves more freedom than the other. Value types that genuinely need aliased mutation opt into a header-carrying type explicitly: `RefCell[T]` (Part IX §7) or a `class`.

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
unsafe     virtual    void       where      while      with       yield
```

*(`yield` moved here 2026-09-09 from the list below, which is where it also
sat. `[LEX-15b]` already makes it a v1 keyword and says its count "supersedes
that [`[LEX-15]`'s 48]" — so the two lists implied two statuses for one word,
and the rule was the half that governed. 49 entries. See
`docs/spec-amendments.md`.)*

**Reserved for future use (lexed as keywords, `E0005` if used):**

```
actor  async  await  macro  move  trait  union  loop  unless
```

* `[LEX-15]` `abstract`, `final`, `lazy`, `test`, `bench` and `from` are **contextual**: they are keywords only in the positions specified in the grammar and identifiers elsewhere. `from` is a keyword only where it begins an import at item level, which is the one position an import may start; everywhere else it is an ordinary name, so that `interface From[T]` can declare `fn from(…)` as §8 writes it, and so that a user's `extend E implements From[io.Error]:` can too (owner decision, 2026-09-08; errata ERR-017). `let` is **fully reserved** (owner decision `OQ-26`), so it is a keyword everywhere and `r#let` (`[LEX-14]`) is required to use it as a name. `type` is fully reserved for the same reason: `[LEX-15a]` gives it three v1 meanings, so it cannot sit in the reserved-for-future list whose message `[LEX-14a]` requires to name a future version. `r#type` is the escape for the imported C field `[LEX-14]` names. The reserved set therefore has 48 entries.
* `[LEX-15b]` `yield` is **fully reserved** — a keyword everywhere, with `r#yield` (`[LEX-14]`) required to use the name — because `[CORO-2]` makes it an expression form, and an expression keyword cannot be contextual without ambiguity at the start of a statement. `gen` is **contextual**: a keyword only immediately before `fn`, and an ordinary identifier everywhere else, so a field or variable named `gen` is unaffected. `[LEX-15]`'s reserved set therefore has **49** entries, not 48; this rule supersedes that count and no other part of it.

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
* `[LEX-15a]` `type` is a v1 keyword: it introduces a type alias (`type_alias`), **a nominal range type where that alias carries an `in` clause (`[RNG-1]`)**, an associated type in an `interface` (`interface_member`), and an opaque foreign type in an `extern` block (`extern_item`). `union` remains reserved: v1 has no `union` declaration form in Ember source. The FFI importer materialises a C `union` into the synthetic binding module (`[FFI-8]`), which is generated rather than parsed from user source, so no hand-written program contains the token. `type` is a v1 keyword: it introduces a type alias (`type_alias`), **a nominal range type where that alias carries an `in` clause (`[RNG-1]`)**, an associated type in an `interface`, and an opaque foreign type in an `extern` block.
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
                 | extern_class | comptime_block | test_decl
attribute       := "@" identifier ["(" [attr_args] ")"] NEWLINE?
attr_args       := attr_arg {"," attr_arg}
attr_arg        := (expression | identifier "=" expression) [grade]
grade           := "@" ("asserted" | "checked" | "instrumented" | "proven")
```

## III.2 Declarations

```ebnf
fn_decl         := fn_header ":" block
                 | fn_header NEWLINE                                    (* only inside interface/extern *)
fn_header       := ["extern" string_lit] ["unsafe"] ["virtual" | "override"]
                   "fn" identifier [generic_params]
                   "(" [param_list] ")" ["->" type] [where_clause]
                 (* `extern "C" fn f(...)` at item level DEFINES a function with that
                    ABI and is what `@export` (XVI.10) attaches to; it is distinct from
                    `extern_block`, which DECLARES foreign functions. Its parameter and
                    return types MUST be FFI-safe under `[FFI-5]`, a range type among
                    them is `E5054` under `[RNG-10b]`, and a panic reaching the boundary
                    is `[FFI-20]`'s. `virtual`/`override` on one is `E0104`. *)
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
type_alias      := "type" identifier [generic_params] "=" type [range_clause] NEWLINE
range_clause    := "in" expression                                      (* a `..` or `..=` range expression *)

extern_block    := ["unsafe"] "extern" string_lit ":" NEWLINE INDENT {extern_item} DEDENT
extern_item     := {attribute} (fn_header NEWLINE | static_decl | "type" identifier NEWLINE)
extern_class    := "extern" "class" path [implements_clause] ":" type_body
                 (* `[FFI-39]`. A DECLARED foreign base: sized, of known layout, and
                    implicitly `open` so that `[CLS-4]` admits it as a base — as against
                    `extern_item`'s `type`, which is opaque, unsized, and may not be
                    inherited. Inheriting one REQUIRES `@ffi(trampoline, virtuals=[...])`
                    on the declaration; without it the class may be used and not
                    subclassed. Multiple inheritance, virtual bases and unnamed virtuals
                    remain unsupported. *)

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
fn_type         := ["extern" string_lit] "fn" "(" [fn_type_param {"," fn_type_param}] ")" ["->" type]
fn_type_param   := [("mut" | "owned")] type
dyn_type        := "dyn" bound_list
array_type      := "[" type ";" expression "]"                          (* fixed-size inline array, e.g. [f32; 16] *)
```

`[GRM-3]` `Span[T]`, `MutSpan[T]`, `Array[T]`, `Option[T]`, `Result[T, E]`, `Box[T]`, `Shared[T]`, `Weak[T]` are ordinary library types spelled with `path_type`; the grammar does not special-case them.

## III.4 Statements

```ebnf
block           := NEWLINE INDENT {statement} DEDENT | simple_stmt NEWLINE
statement       := {attribute} (simple_stmt NEWLINE | compound_stmt)
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
* `[GRM-8a]` Inside `[` `]` in expression position the parser MUST commit to `type_only_arg` when the next token is one of `ref`, `*`, `dyn`, `fn`, `extern`, `void`, `!`; otherwise it parses an `expression`. Each argument is recorded in the `IndexOrInstantiate` node as `TypeOrExpr::{Type, Expr}` (Part XIX §2). The set is unambiguous: Ember has no prefix `*` and no prefix `!` (`not` and `~` are the operators), and the rest are keywords, so committing on those seven tokens cannot misparse an expression.
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
| `@export_table("Name", protocol=N)` | struct | ABI (`[FFI-26]`, Part XXII) |
| `@non_exhaustive` | enum | FFI import (`[FFI-8]`) |
| `@component(layout=soa\|aos)` | struct | ECS storage (`[ECS-2]`) |
| `@soa(flatten)` | field | SoA column layout (`[SOA-1]`) |
| `@gpu` | fn | **reserved (v3)** (XVII §8) |
| `@allow(code, …)` | any item, and any statement admitting a statement attribute | suppresses the named `W`/`L` diagnostics within the annotated item |
| `@realtime` | fn | hard-contract set (X.1 `[EFF-19]`) |
| `@noio` `@nolock` | fn | hard contract (X.2 `[EFF-20]`, `[EFF-21]`) |
| `@safety("text")` | unsafe fn | trusted-base obligation (IX `[UNS-7]`) |
| `@must_drop` | struct, class | drop is load-bearing for a borrow guarantee (`[THR-6]`) |
| `@fp(contract)` | fn | float control (`[TYP-9b]`) |
| `@deprecated(since, note)` | any item | policy (`[VER-3]`, intent) |
| `@deterministic` | fn, module, interface method, fn type | hard contract (X.2a `[DET-1]`) |
| `@reloadable` `@noreload` | module, fn, class | hot-reload scope (`[HR-25]`) |
| `@renamed_from("name")` | field, enum variant | migration identity (`[HR-14]`, `[HR-11a]`) |
| `@reinit_on_reload` | static | re-run the initialiser on reload (`[HR-17a]`) |
| `@allow_reload_terminate` | `migrate_from` | admits a `noexcept` foreign call that may terminate the process (`[HR-43]`) |
| `@ffi(throws = "translate" \| "noexcept")` | extern fn / overlay | C++ exception policy (`[FFI-43]`) |
| `@always_specialize` `@never_specialize` | generic fn, generic type | monomorphisation control (`[MONO-7]`) |

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

`[TYP-1]` Every concrete type has compile-time-known `size`, `align`, `is_copy`, `is_send`, `is_sync`, `needs_drop`, `is_view`, `has_niche`. The compiler computes these in the `TypeInfo` table (Part XIX §4.3).

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

`[TYP-4]` **No implicit conversions between scalar types in operators.** `i32 + i64` is `E2020`. `[TYP-5]` **Coercion sites** (assignment, argument, return, field initialiser, array element) allow **lossless widening**: `iN → iM` (M>N), `uN → uM` (M>N), `uN → iM` (M>N), `f32 → f64`, `f16 → f32`; and **range erasure**: a value of a range type `T` over representation `R` (`[RNG-1]`) coerces to `R`. Range erasure is a distinct coercion step that composes with the widening rules above, so `Roughness → f32 → f64` and `Percent → u8 → u32` are coercions. **No coercion produces a range type** — construction is `[RNG-3]`/`[RNG-3a]`. Nothing converts to/from `bool` or `char` implicitly. `[TYP-6]` Everything else uses `as`:

* `x as T` for numeric types: truncation for narrowing integers (bit truncation), float→int saturating with NaN→0 (Rust semantics), int→float round-to-nearest.
* `x as u8` from `char`, `x as char` from `u8` only (wider ints via `char.from_u32() -> Option[char]`).
* `[TYP-7]` `as` between pointer types and between pointer and `usize` requires `unsafe`.

**Integer overflow** `[TYP-8]`: in the `debug` profile every arithmetic operation that overflows panics with `E-panic: integer overflow` and the source location. In `release`/`shipping`, `+ - *` and `<<` wrap two's-complement; `/` and `%` by zero always panic; `i32.MIN / -1` always panics. `@overflow(panic|wrap|saturate)` on a function or module overrides the profile. Explicit methods `wrapping_add`, `checked_add -> Option`, `saturating_add`, `overflowing_add -> (T, bool)` always exist.

**Floating point** `[TYP-9]`: strict IEEE semantics; no fast-math, no FMA contraction, no reassociation unless the function is `@fastmath`, in which case the C backend emits `#pragma float_control(precise, off)`/`-ffast-math`-equivalent attributes for that function only. `NaN == NaN` is false; `Ord` is not implemented for floats — use `partial_cmp` or `total_cmp`.

**Shifts** `[TYP-10]`: shift amount ≥ bit width panics in debug and is masked in release (like Rust).

## IV.2a Range and domain types

A range type is a nominal numeric type that carries its own bounds. Two of them
over the same representation are different types, so a roughness cannot be
passed where a metallic is wanted even though both are `f32` — which is the
point, because that confusion is not detectable in any other way.

```ember
type Roughness = f32 in 0.0 ..= 1.0
type Metallic  = f32 in 0.0 ..= 1.0
type Fov       = f32 in 1.0 ..= 179.0
type Percent   = u8  in 0 ..= 100

fn demo():
    r: Roughness = 0.5      ## in range at compile time; no check is emitted
    m: Metallic  = r        ## E2210: `Roughness` is not `Metallic`
```

A value the compiler cannot place in range is converted fallibly:

```ember
fn from_slider(x: f32) -> Result[Roughness, RangeError]:
    return Roughness.checked(x)

fn clamped(x: f32) -> Roughness:
    y = min(max(x, 0.0), 1.0)
    return Roughness.checked(y).unwrap()      ## [RNG-4] discharges the check
```

* `[RNG-1]` A `type` alias carrying an `in` clause declares a **nominal**
  numeric type over the named representation, restricted to that range. A `type`
  alias without one is transparent, exactly as `[LEX-15a]` specifies. The clause
  takes a range expression (`a .. b` or `a ..= b`) whose endpoints are constant
  expressions of the representation type.
* `[RNG-2]` Two range types are distinct types even when representation and
  range are identical (`E2210`). A range type converts to its representation
  implicitly; the reverse requires `[RNG-3]`. The implicit conversion this rule names is `[TYP-5]`'s range erasure and is admitted at `[TYP-5]`'s coercion sites only; a range type never converts implicitly in operator position except through `[RNG-5a1]`'s generated impls.
* `[RNG-3]` Construction from a value not statically known to be in range is
  `T.checked(v) -> Result[T, RangeError]`. Construction from a constant in
  range, or from a value whose known range is contained in the target's, emits
  no check. **`RangeError` is a prelude type**, in scope in every module without
  an import and nameable wherever a type may be written, so that this rule's own
  signature can be written by a program. It is not a `std` declaration: `checked`
  is a language-defined construction under `[RNG-10]` rather than a library
  function, so its error type cannot depend on a module having been imported, and
  an implementation MUST have the name resolvable while *signatures* are being
  collected and not merely once some body mentions `checked`. An implementation
  MAY represent it as a unit-only enum under `[ENM-3]`, which costs nothing in a
  `Result` that `[TYP-13]` can niche; the representation is not prescribed here. *(clarified 2026-09-09; see `docs/spec-amendments.md`)*
* `[RNG-4]` The compiler tracks a known range for every numeric expression it
  can — literals, `min`/`max`/`clamp`, the arms of an `if` or `match` that
  compared the value, and arithmetic on operands with known ranges — and uses it
  to discharge `[RNG-3]`'s check and `[TYP-8]`'s overflow check. A range fact is
  never derived from inside a `@fastmath` function. A range fact derived from an arithmetic operation is that operation's **mathematical** range only when the mathematical range is contained in the representation's range — that is, when the operation provably cannot overflow, in which case `[TYP-8]`'s check for it is discharged by the same fact. Otherwise the derived range is the representation's full range, or, under an effective `@overflow(saturate)`, the mathematical range clamped to the representation's. A range fact MUST NOT be derived from the mathematical range of an operation that can overflow, **in any profile**: `[TYP-8]`'s policy differs between `debug` and `release`, and `[PRF-1]` and `[PHIL-5]` forbid the set of checks the compiler emits from depending on that difference. **`[RNG-5a]`'s range-preserving clamp family is exempt**: `min`, `max` and `clamp` cannot overflow, so their mathematical range is always the computed one and `T.clamped` keeps its fact.
* `[RNG-5]` Arithmetic involving a range type normally yields its **representation**,
  not the range type: `r * 2.0` is `f32`. A range value MAY participate directly in
  arithmetic with an ordinary value of its representation type. Arithmetic between
  two **distinct nominal range types** is rejected (`E2214`) unless at least one
  operand is explicitly converted to its representation type. Thus `Roughness +
  Roughness` is permitted, `Roughness * 2.0` is permitted, but `Roughness +
  Metallic` is rejected. Producing a range type again is a construction and goes
  through `[RNG-3]`. This preserves the range types' nominal purpose without
  inventing a range-propagating result type for every operator.
* `[RNG-6]` Range reasoning over floats obeys `[TYP-9]`'s strict IEEE semantics.
  NaN is in no range. A range with endpoints `-0.0` and `+0.0` contains both
  zeros. `[TYP-9a]`'s prohibition on contraction is what keeps a range fact from
  being invalidated by an FMA the backend introduced.
* `[RNG-7]` A range type is a niche for `[TYP-13]`: `Option[Percent]` occupies
  one byte. **A range type supplies a niche only where its range does not exhaust its representation**: `type Full = u8 in 0 ..= 255` has no invalid value and `Option[Full]` is two bytes. The compiler MUST NOT claim a niche it does not have. *(editorial instruction carried out 2026-09-09; see `docs/spec-amendments.md`)*
* `[RNG-8]` A range type is `Copy` when its representation is, has the layout of its representation, and crosses an FFI boundary as its representation (`[FFI-5]`). A range type is not itself writable in a foreign signature (`[RNG-10b]`); a value arriving from foreign code enters at the representation type and becomes a range value only through `[RNG-3]` or `[RNG-3a]`. *(head recovered verbatim 2026-09-09; see `docs/spec-amendments.md`)*
  * `[RNG-9]` **Range validity is an invariant, not a convention.** A value of a range type whose representation does not lie in the declared range is **invalid**; producing one is undefined behaviour and requires `unsafe`, exactly as `[TYP-2]` provides for `bool`. `[RNG-4]` MAY assume validity, which is what licenses it to discharge `[TYP-8]`'s overflow check and `[OPT-2]`'s bounds check. Invalidity is not merely a wrong number: by `[RNG-7]` a range type is a niche, so an out-of-range representation is a bit pattern that is neither a payload nor a discriminant.
  * `[RNG-10]` **The construction set is closed.** In Safe code a range-typed value arises only from: (a) a constant the compiler placed in range (`[RNG-3]`); (b) `T.checked(v)`; (c) `T.clamped(v)` (`[RNG-3a]`); (d) a value whose `[RNG-4]` range is contained in the target's; (e) a copy or move of an already-valid value. Any other route is `unsafe` and is `T.new_unchecked(v)`, whose safety condition is written in its documentation and whose `debug` build MUST `debug_assert` the range. Constructing a range-typed value outside this set in Safe code is `E2215`. * `[RNG-10a]` A generated `deserialize` (`@derive(Deserialize)`, XIV.4) MUST emit `T.checked(...)` for every range-typed field, transitively through nested aggregates, and MUST map a failure to `SerError`. A derive that omits the check is a compiler defect, not a performance option. * `[RNG-10b]` A range type MUST NOT appear as a parameter type, return type or field type in an `extern` declaration, an `@ffi` overlay signature, or an imported foreign type — **nor in any aggregate transitively containing one, nor as the pointee of any pointer passed to or returned from foreign code.** It is `E5054`, whose help is to declare the representation and construct with `T.checked(...)` in the Ember-side wrapper (`[FFI-13]`). An `unsafe extern` does not discharge this: `[TIER-1]`'s boundary lets the programmer assert a signature, not a range, and `[FFI-38]`'s "the importer MUST reject rather than guess" applies. * `[RNG-10c]` `unsafe` reads through a raw pointer, `mem` reinterpretation and uninitialised storage may produce a value at a range type; that is the `unsafe` block's obligation under `[UNS-4]`, and `[UNS-*]` is unchanged.
  * `[RNG-3a]` **Total construction.** Every range type whose endpoints are finite provides `T.clamped(v: Repr) -> T`, defined as `min(max(v, lo), hi)` for an inclusive range and as the nearest representable value strictly inside a half-open one. It is **total**: it has no failure mode and introduces no `Panic` and no `RuntimeCheck(k)`, and `[EFF-16]` is amended to state that `T.clamped` introduces no `Panic(Explicit)`. For a float representation, `NaN` maps to `lo` — which the `docs/errors/` reference page MUST state — and `-0.0`/`+0.0` follow `[RNG-6]`. On the C backend it lowers to two compares or the target's `min`/`max` instruction pair, **strictly cheaper than `[RNG-3]`'s check-and-branch-to-panic**. `T.checked` remains for code that must distinguish an out-of-range input from a clamped one.
  * `[RNG-5a]` **The clamp family is range-preserving.** Where `min`, `max` or `clamp` is applied to operands of one range type `T`, or to a `T` and constants of its representation lying within `T`'s range, the result is `T`, not the representation. This is the one exception to `[RNG-5]` and is sound because the result's range is contained in `T`'s by construction; `[RNG-4]` discharges it with no emitted check. FIX-013's `[RNG-4]` amendment **exempts this case explicitly**, so `clamped` keeps the fact that makes it free. `L2003` warns where a fallible construction (`T.checked`) is written and a total one (`T.clamped`, or a literal the compiler can place in range) would do, because a `Result` the programmer immediately unwraps is a panic path that need not exist.
  * `[RNG-4a]` **Float facts come only from the true arm.** Over a float representation, a range fact is derived only from the **true** arm of a comparison. `not (x > hi)` does not establish `x <= hi`: `[RNG-6]` puts NaN in no range and NaN fails both comparisons, so the false arm of a float comparison establishes no bound. A fact derived from a float comparison MUST record whether NaN is excluded, and a fact that does not exclude NaN MUST NOT discharge a `[RNG-3]` construction.
  * `[RNG-5a1]` **Operators on range types resolve through interfaces, not through a built-in rule.** For every range type `T` over representation `R` the compiler generates, **in `T`'s declaring module**, the impls `T: Add[T, Output = R]`, `T: Add[R, Output = R]`, `R: Add[T, Output = R]` and the corresponding `Sub`, `Mul`, `Div`, `Rem`, `Neg`, `PartialEq`, `PartialOrd` forms, defined by erasing each operand to `R` (`[TYP-5]`) and applying `R`'s operator. Because they are emitted in the type's own declaring module they satisfy `[TYP-20]`'s orphan rule as written and **need no exemption**. No `*Assign` form is generated: `r += 1.0` would produce an `R` where a `T` is required and is `E2214`, whose help names `r = Roughness.clamped(r + 0.1)` or the fallible form. An operator with operands of two **distinct** range types resolves to no generated impl and is `E2214`, whose help names the explicit erasure — `Roughness + Metallic` is therefore rejected by ordinary overload resolution, not by a special case.
  * `[RNG-5a2]` **Exact impl before coercion.** `[TYP-24]`'s resolution selects an impl matching the operand types **exactly** before applying any `[TYP-5]` coercion. Without this, `Roughness + Roughness` matches both the generated `Roughness: Add[Roughness]` and, after erasure, `f32: Add[f32]`, and resolution is ambiguous.

Diagnostics: `E2210` a value of one range type where another was expected;
`E2211` a constant outside the target's range; `E2212` an `in` clause whose
endpoints are not constants of the representation, or are inverted; `E2213` an
`in` clause on a non-numeric representation.

```text
error[E2211]: 1.4 is outside `Roughness`

    roughness: Roughness = 1.4
                           ^^^ `Roughness` holds 0.0 ..= 1.0

  = help: clamp it, or take the fallible form: `Roughness.checked(1.4)`
```

## IV.3 Compound value types

**Tuples** `(A, B, C)`: value category; fields `.0 .1`; `Copy` iff all elements `Copy`. Destructured by `a, b = t`.

**Fixed arrays** `[T; N]`: inline, `N` is a const generic; index bounds-checked; `Copy` iff `T: Copy`. Literal `[0.0; 16]`. Coerces to `Span[T]`/`MutSpan[T]` at coercion sites.

**Structs**: see Part V. Field order in memory follows declaration order unless `@layout(rust)` is given (which permits reordering for size — v2). `[TYP-11]` Default layout is **C-compatible** (`@layout(c)` is implied); this is deliberate so that every plain struct can cross an FFI boundary.

**Enums**: unit-only enums have an integer discriminant (`@repr(u8)` etc., default `i32`-sized-or-smaller chosen by the compiler; `[TYP-12]` `@repr` is required for FFI). Payload enums are tagged unions; layout: `{tag, union of variants}` with the tag placed at offset 0 unless a niche makes it free.

**Niche optimisation** `[TYP-13]`: `Option[T]` where `T` is a class handle, `Box`, `ref`, `Span` (non-null pointer), `bool`, `char`, or an enum with fewer variants than its repr allows, has the same size as `T`. This is *guaranteed* for handles, `Box`, `ref` and `*fn` so that `Option[Handle]` is ABI-compatible with a nullable pointer.

## IV.4 Reference and view types

* `ref T` / `ref mut T`: a first-class reference. Non-null, aligned, points to a live `T` for the duration of its region. `Copy` for `ref T`; `ref mut T` is **move-only** (reborrowable). Auto-dereferenced: `r.field`, `r.method()`, and use of **any expression of type `ref T`/`ref mut T`** where a `T` is wanted all read through — a named reference, a call that returns one, a field read, an operand of an operator, an argument. Reading through is a property of the *type in a value context*, not of the form the reference was written in. `[TYP-14]` A `ref mut` local written with `=` writes through to the referent (C++ reference semantics). Rebinding is not possible; shadow instead.
* `Span[T]` (read) and `MutSpan[T]` (read/write): pointer + length. `Copy` and move-only respectively. Bounds-checked indexing; `.len()`, `.iter()`, `.iter_mut()`, `.split_at(i)`, `.chunks(n)`, `.as_ptr()` (unsafe result).
* `str`: `Span[u8]` known to be valid UTF-8.
* `@view struct`: any struct containing a `ref`, `Span`, `MutSpan`, `str` or another view type is automatically a view type. The attribute is required on the declaration as documentation; omitting it is `E2030` with a fix-it. A `@view struct` has an implicit **region vector** (Part VII §5); a plain `ref`/`Span`/`str` still carries one region, while a composite `@view struct` may carry multiple inferred slots.

`[TYP-15]` A view-typed value MUST NOT be stored in a place whose bounding region fails to outlive any region slot carried by the view. Class fields, non-view struct fields, `static`s, `Box[T]` and `Shared[T]` contents, container elements and `owned fn` captures have no bounding region, so they may hold only a view whose **every** region slot is `static` (`[LT-3]`). A multi-region view therefore remains non-escaping unless all of its borrowed fields are static. Any other stored view is `E3063 stored view may not outlive its source`, shape B12. This is the direct multi-region generalisation of the existing rule; it does not create a storage exception. **This does not relax `[TYP-15a]`**: arbitrary owning containers instantiated at a view type remain rejected. Views MAY live in locals, parameters, return values, and tuple, enum, `Option` and `Result` payloads, subject to every carried region remaining valid.

`[TYP-15a]` **Specialized containers of views.** `BorrowList[T]` and `ViewList[T]` MAY contain view-typed elements when one inferred container region outlives **every region slot of every stored element**. Multi-region elements are therefore accepted only when their individual region slots are all bounded by that single container region. An operation that would require a region outside the container's bound is rejected. These types remain specialized borrowing containers, not ordinary owning generic containers, and they MUST NOT be used to smuggle a view into a class field, `static`, `Box`, `Shared`, or another place forbidden by `[TYP-15]`. Arbitrary owning containers instantiated at a view type (`Array[str]`, `Map[str, V]`, `Array[MutSpan[T]]`) remain rejected.

## IV.5 Raw types

`*T`, `*mut T`: nullable, unaligned-permitted, untracked. Creating one from a `ref` is safe (`ref_to_ptr`); dereferencing, offsetting, reading, writing require `unsafe`. `null[T]()` produces a null pointer. `*void` is permitted for FFI.

`extern "C" fn(i32) -> i32`: a raw function pointer with the C calling convention; `Copy`; cannot capture.

## IV.6 Class handles

A `class C` declaration introduces the type `C` whose values are **handles** (non-null pointers to counted heap objects). `Option[C]` is the nullable form. `Weak[C]` is a weak handle. Handles are `Copy` (copying retains). Upcasting a handle to a base class or to `dyn I` is implicit; downcasting uses `h as? Derived` → `Option[Derived]` (runtime type check) or `h as! Derived` (panics). Semantics in Part VIII.

## IV.7 Generics

* `[TYP-16]` Generic functions and types are **monomorphised**: every distinct instantiation produces a distinct symbol. Code size is the programmer's responsibility; the compiler deduplicates identical instantiations across modules at link time (COMDAT in the C backend via `inline`/`selectany`, weak symbols via LLVM). XIX §4.11a gives the programmer the instrument that sentence assumes: a count, a report, a budget, and — for a generic that uses its parameter only to call its bounds' methods — the option of one shared function in place of the set.
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

The algorithm is Hindley–Milner-style unification over a per-function inference table (union–find of type variables with occurs check), with interface-bound obligations collected and solved after unification (Part XIX §4.4). There is no let-polymorphism for locals.

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
* `[MOD-6]` A module MAY declare a supported language version with `#! language "X.Y"` on its first line; the package's `ember.toml` `language` key is the default. Mismatch with the compiler's supported set is `E0006`. The compiler's supported set MUST include every language version whose source it still accepts. For this revision the supported set MUST include `"0.9.5"`, `"0.9"`, `"0.8.5"`, `"0.8.4"`, and `"0.8.3"`. Selecting an older version selects that version's accepted-program and semantic rules; it does not silently opt the module into later language semantics. *(hardened 2026-09-10; HC-095-01)*
* `[MOD-6a]` **Patch-level language revisions.** A language version MAY be written as `X.Y` or `X.Y.Z`. `0.9` remains the pre-0.9.5 0.9 language contract; `0.9.5` selects the inferred multi-region-view semantics of this revision. The compiler MUST NOT silently enable `0.9.5` semantics for a module that explicitly selects `0.9`. The supported set is exactly the set stated by `[MOD-6]`; this rule explains why `"0.9"` and `"0.9.5"` denote distinct contracts rather than adding another version entry. The package manifest and module directive MUST resolve to the same exact language contract after normalization.
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
  * `mut b: B` — **inout** (mutable borrow). The argument MUST be a mutable place, **or a mutable view value derived from one** (`[FN-1a]`); the callee may mutate; no move out (except by `mem.replace`/`take`).
  * `[FN-1a]` **A `mut` parameter whose declared type is itself a view accepts the view value.** Where `B` is a view type — `MutSpan[T]` and the rest of Part IV §1's View category — the argument may be the result of an expression that *produces* such a view, and the mutable-place requirement applies to the place that view was **taken of**, not to the final expression. This is what makes Part VII §7's own worked example `normalize(buf.as_mut_span())` valid, and it is valid. The distinction is between passing a mutable borrow **derived from a mutable place** and passing an arbitrary value merely because its type looks mutable: the borrow relationship of the view-producing expression is preserved and checked, so this admits no arbitrary temporary and bypasses no mutability check. `[SPN-3]` makes `MutSpan[T]` move-only, so there is exactly one live writable view either way. *(Owner ruling 2026-09-10 on ERR-041; see `docs/spec-amendments.md`)*
  * `owned c: C` — **consumed**. The argument is moved (or copied if `Copy`; retained if a handle). The callee owns it and will drop it or move it on.
  * `[FN-2]` Missing mode is `borrowed`. There is no by-value-copy mode; if the callee wants its own copy it writes `owned` and the caller writes `f(x.clone())` or `f(x)` for `Copy` types.
* `[FN-3]` Return values are moved out; returning a `ref`/view requires that the region be tied to a parameter by elision (Part VII §5).
* `[FN-4]` The receiver `self` follows the same modes: `self` (borrow), `mut self` (mutable borrow), `owned self` (consume). Absent `self` ⇒ associated function. `self: Type` explicit form is allowed for `self: ref Self`, `self: Box[Self]`, `self: Shared[Self]` (v2 for the latter two).
* `[FN-5]` Default arguments are evaluated in the callee's scope at each call, after positional/named binding. They may reference earlier parameters. They MUST be `@noalloc`-clean if the function is `@noalloc`.
* `[FN-6]` Functions are values of a unique zero-sized function type; they coerce to callable types
  whose parameter modes use the same syntax and semantics as ordinary function parameters:

  ```text
  fn(A) -> R
  fn(mut A) -> R
  fn(owned A) -> R
  ```

  An omitted callable-parameter mode is `borrowed`, matching `[FN-2]`. `mut A` denotes an
  inout/mutable-borrow parameter and `owned A` denotes a consumed parameter. Callable-type parameter
  modes MUST have the same ownership, borrowing, mutability, place, and move restrictions as the
  corresponding ordinary function declaration modes. This extension does not introduce a separate
  callable ownership model. A function with no captures and no generic parameters may also coerce
  to `extern "C" fn(...) -> R`; that form remains valid only when every parameter and return type,
  including the parameter modes, satisfies the existing FFI-safety rules.
* `[FN-6a]` **`Callable` preserves parameter modes.** The existing `Callable[Args, R]` abstraction
  MUST preserve whether each represented callable parameter is borrowed, `mut`, or `owned`; `Args`
  MUST NOT erase that information. Conceptually, its canonical type identity distinguishes
  `Callable[(A, B), R]`, `Callable[(mut A, mut B), R]`, and
  `Callable[(owned A, B), R]`. These forms describe canonical type identity, not an additional
  source grammar for tuple values. An omitted mode remains borrowed under `[FN-2]` and is not
  reinterpreted as a new callable ownership mode.

  The complete mode vector is compile-time metadata. It MUST survive generic callable bounds, type
  checking, borrow checking, overload resolution, and monomorphisation, but MUST NOT introduce
  runtime parameter-mode bookkeeping or alter callable ABI beyond the existing ABI rules. Where
  `Callable[Args, R]` source notation cannot directly spell the modes, the compiler carries the
  callable's full mode-bearing signature in its internal canonical callable type. This is not a
  second ownership system.
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
* `[CLS-2]` **Constructors.** `fn init(mut self, …)` is the constructor. Definite-initialisation analysis (Part XIX §5.6) requires every field without a default to be assigned on every path before `self` is used as a whole (passed anywhere, method called, escaped). Reading a field before it is assigned is `E2100`. Multiple constructors are not supported by overloading; use defaults or associated functions `fn from_file(path: str) -> Result[Self, E]` that call `Self(...)`.
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

* `[ATT-2]` A statement attribute MUST be one of `@simd`, `@parallel`, `@unroll`, `@allow`. Any other attribute in statement position is `E0104`, naming the four that are permitted there.
* `[ATT-5]` The attributes 0.6.3 adds are item attributes and never statement attributes, so `[ATT-2]` is unchanged. `@deterministic` is admitted on a function, a module, an interface method and a function type; `@reloadable` and `@noreload` on a module, a function or a class, and not both on the same item (`E0104`); `@renamed_from` on a field or an enum variant; `@reinit_on_reload` on a static; `@always_specialize` and `@never_specialize` on a generic function or generic type only, and not both on the same item (`E0104`). Each in any other position is `E0104`, naming the positions it does admit.
* `[ATT-3]` A statement attribute attaches to the next `compound_stmt` and MUST NOT precede a `simple_stmt` (`E0108`). `@simd`, `@parallel` and `@unroll` additionally require that statement to be a `for_stmt` (`E0108`). More than one statement attribute MAY precede one statement, one per line (`[ATT-4]`); their order is not significant.
* `[GRM-20]` The `{attribute}` prefix of `statement` is constrained by `[ATT-2]` and `[ATT-3]`: the grammar admits the position and Part V §8 decides which attributes may occupy it. The `{attribute}` prefix of `statement` is constrained by `[ATT-2]` and `[ATT-3]`: the grammar admits the position and Part V §8 decides which attributes may occupy it.
* `[GRM-21]` `gen_fn := "gen" fn_decl`. A `gen fn` is admitted wherever `fn_decl` is admitted except in an `extern` block (`[CORO-10]`), and its declared return type MUST be `Coroutine[R]` for some `R` (`E2222` otherwise).
* `[GRM-23]` **`in` and `not in` are binary operators.** `membership := expression ("in" | "not" "in") expression`, at **comparison precedence** — the same level as `==` and `<` — and **non-associative**: `a in b in c` is `E0104`, because Ember has no chained comparison and the Python reading would surprise a C++ programmer. `not in` is a **single operator**, so `x not in xs` parses as `not_in(x, xs)` and never as `(not x) in xs`; a `not` that is not immediately followed by `in` is the ordinary prefix operator. The token `in` is already used in two other positions and neither is ambiguous with this one, because both are reached in a context that admits no expression at that point: `for pattern in expression:` (`[GRM-*]`'s for header, where `in` follows a pattern) and `type T = R in a ..= b` (`[RNG-1]`, where `in` follows a type). A parser MUST commit on the enclosing construct, not on the token.
* `[GRM-22]` `yield_expr := "yield" [ expression ]`. `yield` occupies the grammatical position and precedence of `return`: it is an expression whose type is the coroutine's resume type (`()` unless the coroutine declares one), so it may appear anywhere an expression may, including as the right-hand side of a binding. A bare `yield` is `yield ()`.
* `[GRM-8d]` `range_clause` is admitted only where `type_alias` appears as an `item`; a `range_clause` on an `interface_member` associated type or on an `extern_item` opaque type is `E2213`. A `type_alias` carrying both `generic_params` and a `range_clause` is `E2213`: a range type is over a concrete representation. The production is LL(2) — no other `type_alias` may be followed by `in`, and a `type_alias` never appears where a `for_stmt` may begin, so `for x in …` is untouched. `range_clause` is admitted only where `type_alias` appears as an `item`; a `range_clause` on an `interface_member` associated type or on an `extern_item` opaque type is `E2213`. A `type_alias` carrying `generic_params` and a `range_clause` is `E2213`: a range type is over a concrete representation. The production is LL(2): no other `type_alias` may be followed by `in`, and `type_alias` never appears where a `for_stmt` may begin, so `for x in …` is untouched.
* `[FFI-34a]` The `grade` suffix is admitted **only** on an argument of `@ffi` in an overlay or `extern` declaration; anywhere else it is `E0104`. `@` introduces an attribute and a grade and nothing else — it is not an operator in expression position — and the four grade words are contextual, remaining ordinary identifiers everywhere else, so `[LEX-15]`'s reserved set is unchanged. The `grade` suffix is admitted **only** on an argument of `@ffi` in an overlay or `extern` declaration; anywhere else it is `E0104`. `@` introduces an attribute and a grade and nothing else: it is not an infix, prefix or suffix operator in expression position, and `[LEX-15]`'s reserved set is unchanged — the four grade words are contextual and remain ordinary identifiers everywhere else.
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
| `x in coll` | `coll.contains(x)` | `Contains` (`[STD-8]`); comparison precedence, non-associative (`[GRM-23]`) |
| `x not in coll` | `not coll.contains(x)` | one operator, not `not` applied to `in` (`[GRM-23]`) |
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
* `[CLO-3]` Calling: `f(args)`. A parameter declared `f: fn(A) -> R` is a generic over `Callable` (static dispatch, monomorphised). A boxed dynamic closure is `Box[dyn fn(A) -> R]` (`E4001` in `@noalloc` because boxing allocates). An `extern "C" fn` parameter accepts only capture-free closures and named functions. **`fn(A) -> R` is a bound, not a representation.** It denotes an implicit generic parameter bounded by `Callable[(A), R]` — or by `CallableOnce` under `[CLO-6]`'s `owned` mode — so each argument monomorphises the callee against its own type. For a mode-bearing `fn` type, this implicit bound retains the complete parameter-mode vector as canonical compile-time metadata under `[FN-6a]`; the `Args` tuple notation does not erase it. As an **implementation invariant, not an observable property a program may rely on**: a named function has a zero-sized function type, so its call becomes a direct call, and a lambda has an anonymous type whose fields are its `[CLO-2]` captures and whose `call` is its body. **A conforming implementation MUST NOT realise `fn(A) -> R` as a function pointer**, which erases the captures and so admits only capture-free lambdas; the ABI-level function pointer is `extern "C" fn(A) -> R`, and keeping the two distinct is what lets `[COST-3]` report a direct call for the ordinary case. *(clarified 2026-09-09 and 2026-09-12; see `docs/spec-amendments.md`)*
* `[CLO-4]` A non-`owned` closure cannot escape the scope of what it borrows: storing it, returning it, or passing it to a function whose parameter is `owned`/stored triggers the normal view-type rules (`[TYP-15]`).
* `[CLO-5]` Closures capturing class handles retain them (a strong reference) — the usual cycle caution applies (Part VIII §5).
* `[CLO-6a]` **`CallableOnce` is not `dyn`-compatible in v1**: `call_once` takes `owned self`, which `[TYP-22]` does not admit through a vtable. `[CLO-3]`'s `Box[dyn fn(A) -> R]` therefore remains a `Callable`, and a boxed once-callable payload is expressed by moving the payload into the closure's captures and having the boxed closure take it by `mem.take` from an `Option` field — the one place the `Option` dance survives, and the reason `[TYP-22]`'s by-value-self restriction is worth revisiting in v2.
* `[CLO-7]` `thread.spawn`, `jobs.submit`, `jobs.submit_after`, `Option.map`/`and_then`/`unwrap_or_else`, and `Result.map`/`map_err`/`and_then`/`unwrap_or_else` MUST declare their callable parameter `owned f:`. `[JOB-1]`'s inline job slot is unaffected: the closure remains a value moved into the slot.
* `[CLO-6]` **Once-callable closures.** `std.core` declares `interface CallableOnce[Args, R]: fn call_once(owned self, args: Args) -> R`, and `interface Callable[Args, R]: CallableOnce[Args, R]`, so every reusable closure is also once-callable. A parameter written `owned f: fn(A) -> R` is a generic bounded by `CallableOnce`; `f: fn(A) -> R` and `mut f: fn(A) -> R` are bounded by `Callable`. In each case `[FN-6a]` preserves the callable arguments' own parameter modes independently of the mode on `f` itself. **No call site changes**: the mode already selects the bound, exactly as `[FN-1]`'s three modes already work for every other type. Calling a value bounded by `CallableOnce` consumes it; a second call is `E3040 use of moved value` under `[OWN-3]` and requires no additional analysis. `E3030` is emitted only when a closure implementing `CallableOnce` alone is supplied where `Callable` is required, and MUST use shape O5's help.

## VI.5a Coroutines

Gameplay is full of sequences that span frames: play a sound, wait half a second,
swing the door open over two seconds, then let the player through. Written as a state
machine, five lines of intent become fifty lines of bookkeeping, and every such
sequence in the game gets the same treatment. This is the single largest reason RageV
hosts gameplay in C#.

```ember
gen fn open_door(mut self) -> Coroutine[()]:
    play_sound(self.unlock)
    yield wait(0.5)
    yield animate(self.door, 2.0)
    self.passable = true
```

* `[CORO-1]` A function declared `gen fn` is a **coroutine**. Calling it executes no part of the body; it returns a `Coroutine[R]` value holding the suspended frame. The body runs only when that value is resumed.
* `[CORO-2]` `yield e` suspends the coroutine and delivers `e` to whoever resumed it. `yield` outside a `gen fn` body is `E2220`. A `gen fn` with no `yield` in its body is accepted — it runs to completion on first resume — and `[LNT-4]` warns (`L2004`), because it is nearly always a mistake.
* `[CORO-3]` `std.core` declares `enum CoroutineState[R, Y]: Suspended(Y); Done(R)` and `interface Resumable: type Return; type Yield; fn resume(mut self) -> CoroutineState[Return, Yield]`. **`Coroutine[R]` in a `gen fn`'s return position is not that interface**: it names the concrete frame type the compiler synthesises for *that* function (`[MIR-6]`) — sized, move-only, and implementing `Resumable` — so `[TYP-22]`'s prohibition on bare unsized interface types is not engaged. Two `gen fn`s have distinct, unnameable frame types; a caller storing coroutines of different functions in one container uses `Box[dyn Resumable]` and pays `[TYP-22]`'s dispatch for it. Resuming a coroutine already in `Done` is `E-panic: coroutine resumed after completion` in `debug` and `release`, and unchecked in `shipping` under `[EXC-6]`'s policy.
* `[CORO-4]` **Lowering.** The compiler rewrites the body into a state machine over the frame: locals live across a `yield` become frame fields, locals that are not remain ordinary stack slots of `resume`, and a `state` discriminant selects the resume point. This is a MIR transformation (`[MIR-6]`) and introduces no runtime machinery beyond the frame.
* `[CORO-5]` **Nothing about a coroutine allocates.** The frame's size is a compile-time constant produced by `[CORO-4]` and reported by `size_of[Coroutine[R]]()`; `Coroutine[R]` is an ordinary move-only value type that may live in a local, a struct field, an `Array` or an `Arena`. `Box[Coroutine[R]]` is available for a caller who wants it on the heap and is the only form that allocates. A `gen fn` therefore carries `Alloc` only if its body does, and may be called from `@noalloc`.
* `[CORO-6]` **No borrow may be held across a `yield`.** A reference whose region (`[LT-*]`) spans a suspension point is `E2221`, naming the borrow, the `yield` it crosses, and the place it was taken from. The frame outlives the stack frame that created it and the borrow checker cannot see the resumer, so this restriction is what makes coroutines safe without a new analysis. Values **owned** by the frame are unrestricted; only borrows are refused, and the diagnostic's `help` names moving the value in as the fix.
* `[CORO-7]` A suspended coroutine **owns** everything moved into its frame. Dropping one runs the drops for exactly the locals live at its current suspension point, in reverse declaration order per `[DRP-1]`. The compiler synthesises one drop function per suspension point and selects on `state`; a coroutine dropped before completion is normal and leaks nothing.
* `[CORO-8]` A coroutine's effect set is the union over every path through its body, computed once at the `gen fn` rather than per resume. `@noalloc`, `@deterministic`, `@realtime` and the rest apply to a `gen fn` exactly as to any other function.
* `[CORO-9]` **Self-referential frames are not permitted in v1.** A frame holding a pointer into itself would break the moment the `Coroutine[R]` value moves. `[CORO-6]` already forbids writing one in safe code; `unsafe` code that constructs one carries the obligation, and `[UNS-7]`'s `@safety` text MUST state it.
* `[CORO-10]` A `gen fn` may not be `extern`, may not be `@export`ed, and may not be passed to C as a function pointer (`E2222`): its calling convention is not C's. Coroutines are driven from Ember and their results cross the boundary as ordinary values.
* `[CORO-11]` `std.coroutine` provides what gameplay needs on top of `[CORO-3]`, with no further compiler support: `Scheduler` (resume a set of coroutines once per frame and drop those reporting `Done`), `wait(seconds)`, `wait_frames(n)`, `wait_until(pred)`. `Scheduler` calls `hot.checkpoint()` (`[HR-33]`) between frames, which is what makes a project using it hot-reloadable without writing one.

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

* `[BRW-1]` **Aliasing XOR mutability.** At any program point, a place may have either any number of live shared borrows, or exactly one live mutable borrow, and while a mutable borrow is live the owner may not read, write, move, or drop the place; while shared borrows are live the owner may read (and copy) but not write, move, or drop. **A reference local is not re-seatable**: where `r` has a reference type, `r = e` writes *through* `r` to the place it denotes, and never points `r` somewhere new — which is what allows an implementation's regions to be insensitive to program location. It follows that `r = e` is permitted only where `r` is `ref mut T`; through a shared `ref T` it is the write this rule forbids, and is rejected at the assignment by the type alone, needing no flow analysis. Re-seating a reference local is not defined in v1. *(clarified 2026-09-09; see `docs/spec-amendments.md`)*
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
* `[LT-2]` **View structs.** A `@view struct` has an inferred region vector. Each borrowed field retains its own inferred region slot; fields proven to share a region MAY share a slot. Constructing a view struct from several references therefore preserves field provenance rather than forcing an intersection. The programmer never writes the region vector. The legacy one-region case remains a valid special case of this rule.
* `[LT-3]` **Static region.** String literals, `static` items, and `Span`s over them have the `static` region, which outlives everything and satisfies `[TYP-15]`'s storage restrictions (a `str` literal *may* be stored in a class field because its type is `str` with static region — the compiler records region `static` in the field's type; a non-static `str` cannot be stored there: `E3063 stored view may not outlive its source`, shape B12 — `E3060` is B7, a borrowed value that does not live long enough, which is a different shape *(owner decision, ERR-044, 2026-09-09; see `docs/spec-amendments.md`)*).
* `[LT-4]` **Arena region.** `Arena` allocations return `ref mut T`/`MutSpan[T]` whose region is the arena's borrow; they cannot outlive the arena (`E3061`). When an allocation is returned through a user-defined wrapper, that wrapper MUST either carry `@borrows(arena)` naming the `Arena` parameter that supplies the storage, or return an owned value that does not depend on arena storage. The compiler MUST reject a wrapper whose returned arena-backed view cannot be proven to carry the supplying arena parameter's region.
* `[LT-5]` **Inference.** Regions are inferred by the NLL algorithm in Part XIX §5.7 for all locals; the programmer never writes them. Diagnostics report regions in terms of "the borrow of `x` on line N is still needed on line M".
* `[LT-1a]` **Explicit return region.** The attribute `@borrows(p₁, …, pₙ)`, written on its own line preceding the function declaration, overrides the region that rules 1–3 would assign to a view-typed return. It MUST name one or more parameters; the receiver is named as `self`. The return's region is the intersection of the named parameters' regions, and every unnamed view-typed parameter is NOT borrowed by the return, so the caller MAY continue to use it. `@borrows` overrides **all three** elision rules, **including rule 1**: a method whose result points into an argument rather than into its receiver MUST be written with `@borrows` naming that argument. Naming a parameter that is not view-typed is `E2031`, except that `[LT-4a]` permits the narrow Arena-backed return-provenance case; writing `@borrows` on a function whose return is not view-typed remains `E2031`. If the returned value's region is not a subset of the intersection of the named parameters' regions, the body is rejected with `E3062` (shape B6).  `@borrows` is part of a function's public contract for compatibility purposes: widening it (naming fewer parameters) is a breaking change under `[VER-2]`, and an `override` of a `virtual` method MUST NOT name a superset of the base's parameters — the same inheritance rule as `[EFF-8]`.

* `[LT-4a]` **Arena-backed wrapper provenance.** An `Arena` parameter MAY be named in `@borrows` when a function returns a view whose storage is owned by that arena. The named parameter contributes the arena's borrow region to the returned value. For example:

  ```ember
  @borrows(arena)
  fn make_value(arena: Arena) -> ref mut i32:
      return arena.alloc(0)

  @borrows(arena)
  fn make_buffer(arena: Arena, n: usize) -> MutSpan[u8]:
      return arena.alloc_array[u8](n)
  ```

  The caller MUST keep the arena alive and accessible for the returned view's entire lifetime. The annotation records provenance only: it does not extend the arena's lifetime, transfer ownership, bypass ordinary borrowing, or make `Arena` itself a view type. Nested wrappers MUST repeat `@borrows(arena)` so the relationship remains part of each public signature.

* `[LT-4b]` **The Arena exception is narrow.** `@borrows` MAY name a non-view `Arena` parameter only for a returned view proven to derive from storage owned by that arena. It MUST NOT manufacture a general region relationship for an arbitrary non-view parameter, an owned return, or a view derived from some other source. Every other non-view parameter remains forbidden in `@borrows` with `E2031`.
* `[LT-1b]` **Intersection is reported at the definition.** When rule 3 assigns a view-typed return the intersection of two or more view-typed parameters' regions and the function carries no `@borrows`, the compiler emits the **opt-in lint** `L3014 return region is the intersection of N parameters` at the function's declaration, listing every parameter the caller will be unable to use while the result is live, and naming `@borrows` as the fix. Writing `@borrows` — including `@borrows` naming every view-typed parameter, which expresses the intersection deliberately — silences it. `[LT-1]`'s permissiveness is unchanged. `[LT-2a]` `L3014` applies only to legacy single-region return inference; it MUST NOT be emitted merely because a `@view struct` contains fields from multiple regions. Multi-region field provenance is the normative default under `[LT-14]`–`[LT-24]`.
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

* `[SPN-1]` `Array[T]` coerces to `Span[T]` at borrow sites and to `MutSpan[T]` at `mut` sites; `[T; N]` likewise; `String` to `str`. **This coercion takes a borrow of the source; it is not a conversion.** Coercing to `Span[T]` creates a live shared borrow of the source place and coercing to `MutSpan[T]` a mutable one, and the resulting view's region is that borrow's, under the ordinary rules of `[BRW-1]` and `[BRW-2]` — so the source may not be mutated, moved or dropped while the view is live, and the borrow ends at the view's last use and not at the end of scope. The same holds however the view is spelled: the implicit coercion, `as_span()`/`as_mut_span()`, and a view returned from a call under `[LT-1]`'s elision are one construction, and all three take the borrow. An implementation MUST make this borrow explicit in its IR rather than implied by the coercion, because an implied borrow is invisible to the borrow checker: `v: Span[i32] = a` followed by `a.push(...)` is then accepted, the push reallocates, and safe code reads freed memory. *(clarified 2026-09-09; see `docs/spec-amendments.md`)*
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
* `[RC-5]` A loan whose place chain contains a `Deref` of a class handle constitutes a **shared loan of the handle operand itself**. `ref h.f`, `ref mut h.f`, a `Span`/`MutSpan` derived from `h.f`, a `Ref[T]`/`RefMut[T]` guard obtained from a `RefCell` field of `h`, and any view struct built from these, all keep `h` borrowed for `[BRW-1]` purposes until the last use of any value derived from them. The compiler MUST NOT kill that loan earlier, MUST NOT sink the handle's `release` above it, and MUST report `E3060` (shape B7) if the handle's storage ends first. XIX §4.7 **step 2** generates the extra loan when lowering a projection through a handle.
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

## IX.0 Choosing a storage mechanism

Ember has several storage and lifetime mechanisms on purpose — Part 0 row 2 makes
storage a property of the declared type rather than something the compiler infers,
and that only works if the programmer can choose. This table is the choice, in one
place. It is **normative guidance**, not a new rule: every cell restates a rule from
the part named in the last column.

| Use | Ownership | Aliasing | Destruction | Threads | Reach for it when |
|---|---|---|---|---|---|
| `struct` / `enum` | unique, moved | borrow-checked | end of scope, reverse order | `Send`/`Sync` by field | **the default.** Data with no identity: maths, components, messages (VII) |
| `Box[T]` | unique, heap | borrow-checked | deterministic, on drop | by `T` | one owner, but the value must be on the heap — recursion, a large payload, an unsized tail (IX.1) |
| `class` | shared, counted | dynamic exclusivity | deterministic, at count 0 | atomic count iff `Sync` | the thing has **identity** and several places refer to it: a scene node, an observer, an editor panel (VIII) |
| `Shared[T]` | shared, counted | borrow-checked | deterministic, at count 0 | atomic count iff `Sync` | shared ownership of a **value** type, where `class`'s identity and header are not wanted. Advanced; prefer `class` (IX.1) |
| `Weak[C]` | none | — | never | follows `C` | breaking a cycle, or observing something you do not keep alive (VIII.5, `[WK-1]`) |
| `Arena` | region | borrow-checked | all at once, at reset | thread-confined | many values with one lifetime: a frame, a level load, a parse (IX.2) |
| `Handle[Tag]` | none — an index | `Copy` | the pool decides | plain value | a resource the engine owns and may recycle: GPU objects, ECS entities. Generation-checked, so a stale handle is caught (IX.6) |
| `Cell[T]` | the owner's | interior, whole-value | with the owner | `!Sync` | a counter or memo inside a `struct` you only have a `ref` to (IX.7) |
| `RefCell[T]` | the owner's | interior, runtime-checked | with the owner | `!Sync` | as `Cell`, but you need a reference to the inside; the check is present in every profile (`[CELL-9]`) |
| `Mutex[T]` / `RwLock[T]` | the owner's | synchronised | with the owner | `Sync` | mutation shared **across threads** (XI) |
| `ForeignBox[T]` | foreign, adopted | raw | the foreign destructor | foreign contract | an owned pointer from C or C++ (XVI, `[FFI-36]`) |
| `CppShared[T]` | foreign, `std::shared_ptr` | raw | C++'s count | C++'s rules | a `std::shared_ptr` crossing the boundary. **Not** `Shared[T]` — two independent counts over one object is a double free (`[FFI-17a]`) |

* `[SEL-1]` **The order above is the order to try.** A design that reaches for
  `class` where a `struct` would do pays a heap allocation, a header, reference
  counting and dynamic exclusivity for nothing, and `[CLS-*]`'s ergonomics are not
  a reason to skip the question. `ember inspect --alloc` reports which mechanism a
  declaration actually used.
* `[SEL-2]` **Two mechanisms are never interchangeable across the foreign
  boundary.** `Shared[T]` and `CppShared[T]` both denote shared ownership and use
  *different reference counts*; `Weak[C]` and `CppWeak[T]` likewise. Converting one
  to the other by transmute or by an overlay declaration is `E5065`, and the
  diagnostic names the double free it prevents.

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
* `[UNS-7]` **`@safety("…")` on an `unsafe fn`.** Every `pub unsafe fn` SHOULD carry at least one `@safety("<obligation>")` attribute stating in one sentence, per obligation, what the caller must guarantee. The text is normative documentation, not a checked expression; *(0.6.2 leftover removed 2026-09-09; see `docs/spec-amendments.md`)* **`std` MUST carry a `@safety` on every `unsafe fn` it exports.** A `pub unsafe fn` outside `std` with no `@safety` is `L3015 undocumented unsafe obligation`, at **warn**, whose fix-it inserts `@safety("TODO: state the caller's obligation")`; `L3016` reports a `@safety` text still reading `TODO`.
* `[UNS-8]` **An `unsafe` block records the obligations it discharges.** The compiler MUST record, per `unsafe:` block, the `unsafe fn`s called within it together with their `[UNS-7]` obligations, and MUST emit `W3012 unsafe block with no SAFETY note` at `warn` when the block is not preceded by a `## SAFETY:` doc comment. This is a reporting rule and introduces **no fourth tier**: `[TIER-1]`'s three boundaries are unchanged and neither attribute licenses any operation.
* `[UNS-10]` **`UnsafeCell[T]`** is the lowest-level interior-mutability primitive and lives in `std.mem` beside `[UNS-5]`'s facilities. It permits mutation of its contained storage through **shared** access, and **only from `unsafe` code**. Its API is `UnsafeCell(owned v: T)`, `get(self) -> *mut T` — which needs an `unsafe` context, being a raw pointer under `[UNS-1]` — and `into_inner(owned self) -> T`, which is safe because the cell is consumed and nothing is shared. It hands out **no** safe `ref T` or `ref mut T`, performs **no** runtime borrow check, and provides **no** synchronisation. `UnsafeCell[T]` is **never `Copy`**, is always `!Sync` (`[THR-1]`), and is `Send` when `T: Send`. It exists so that expert library code can implement an abstraction whose invariant cannot be expressed with ordinary borrowing, `Cell`, `RefCell` or a synchronisation primitive. `Cell` (exposes no reference), `RefCell` (runtime borrow check), `Mutex`/`RwLock` (synchronisation) and `UnsafeCell` (the author establishes the invariant) are one hierarchy, and this is its floor — which is what `[CELL-9]`'s sentence about a package needing an unchecked primitive refers to. It does **not** change the semantics of `Cell`, `RefCell`, `Mutex` or `RwLock`, and `exclusivity = "unchecked"` (`[EXC-1]`) does not change its semantics either.
* `[UNS-10a]` **`UnsafeCell` suspends nothing globally.** It does not disable `[BRW-1]`, lifetime or region checking, type checking or bounds checking, and it introduces no further safety tier: `[UNS-2]` applies to it unchanged. Unsafe code MAY temporarily violate the static aliasing proof **inside** the abstraction, and MUST NOT allow a conflicting or otherwise invalid reference to escape into Safe Ember. A safe abstraction may be built on `UnsafeCell`, but hiding an unsafe operation behind a safe signature does not make that abstraction sound: the implementation MUST establish the invariant before it exposes a safe value. The obligations are `[UNS-4]`'s, and the author is responsible for each as it applies — validity, initialisation, type correctness, alignment and non-nullness for raw access, aliasing, lifetime and region validity, correct destruction, and thread-safety. `@safety` (`[UNS-7]`) and `[UNS-8]`'s obligation record are the machinery; no separate documentation or safety system is introduced for it.
* `[UNS-10b]` **`UnsafeCell` is an expert facility and diagnostics MUST NOT suggest it.** No `[DIA-*]` shape may name it as a fix, in the spirit of `[CELL-10]`'s restraint about `RefCell`. It is **not permitted in `@static_safe` code** (`E3105`): `@static_safe` requires safety to be established statically, and `UnsafeCell` delegates it to an unsafe implementation. It introduces no representation or ABI behaviour of its own — foreign use follows the ordinary explicit representation contract — no special hot-reload semantics, and no inherent `Nondet` effect; any effect arises from the abstraction built over it.

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
* `[CELL-2]` `Cell` never hands out a reference to its contents, so no aliasing rule can be violated and **no runtime check is needed**. `get` is a load; `set` is a store. There is no overhead relative to a plain field. **`Cell`, `RefCell` and `Arena` all permit controlled mutation without exposing the unrestricted ownership model of an ordinary mutable field, and each enforces a different invariant to do it.** They are not all interior mutability: `Cell` and `RefCell` are, and `Arena` is a region allocator whose mutation happens to reach through a shared borrow. What they share is the implementation concern, not the concept. The mechanisms are distinct: `Cell[T]` replaces the whole value and hands out no reference, so nothing has to be proved and there is no runtime state; `RefCell[T]` mutates *through* a reference, so `[BRW-1]`'s question is asked at run time against a borrow counter (`[CELL-5]`..`[CELL-8]`); `Arena`'s mutation is allocation, and what it must prove is a region rather than an alias, which `[ARN-1]` does statically by giving the views the arena's region and taking `mut self` to reset. What follows for every one of them is that interior mutability never means the borrow checker stops caring: the obligation moves — to a replacement that cannot alias, to a counter, or to a region — and an implementation that satisfies any of the three by exempting a type from `[BRW-1]` has not implemented it. An implementation MAY share internal machinery between them; nothing here requires it to. *(clarified 2026-09-09; see `docs/spec-amendments.md`)*
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
* `[CELL-12]` **`RefCell[T]` is never `Copy`, whatever `T` is.** A `RefCell` carries mutable runtime borrow state, and copying the value would duplicate that state: two cells would then hold independent and inconsistent knowledge of the same storage, and the runtime borrow invariant `[CELL-5]`..`[CELL-8]` rests on would be unsound. Moving a `RefCell[T]` transfers the whole cell, borrow state included. Copying the contained `T` is a separate question and is unaffected. `[CELL-4]`'s field-derived `Copy` rule is stated for `Cell` and does **not** extend here: `RefCell` is an explicit exception, because its borrow state is semantically coupled to its storage in a way `Cell`'s payload is not.
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

The compiler infers, for every function, an **effect set** ⊆ `{Alloc, Sync, Lock, Io, Panic, Unsafe, FFI, Block, Nondet, RuntimeCheck(k)}`. Effects are orthogonal: one operation MAY introduce several effects at once.

| Effect | Introduced by |
|---|---|
| `Alloc` | any call to `ember_alloc`/`realloc`, class instantiation, `Box`, `Shared`, container growth, `String` formatting, f-strings, `Arena` growth, boxed closures |
| `Sync` | atomic RMW ops with ordering stronger than relaxed, lock acquisition, channel send/receive, thread spawn/join, `Sync`-class retain/release (atomic) |
| `Lock` | acquisition of a `Mutex`/`RwLock` or equivalent synchronisation primitive; `try_lock` and non-blocking lock acquisition carry `Lock` even when they do not carry `Block` |
| `Io` | file, socket, console and other external-I/O operations, whether or not they block |
| `Panic` | `panic`, `assert`, bounds checks, overflow checks (debug), `unwrap`, exclusivity checks, division |
| `Unsafe` | body contains an `unsafe` block or the function is `unsafe fn` |
| `FFI` | calls to `extern` functions |
| `Block` | `Mutex.lock`, `RwLock` operations that may wait, `join`, `sleep`, channel blocking receive, `File.read`, and any call the runtime marks as potentially waiting |
| `RuntimeCheck(k)` | a safety check the compiler emitted rather than proved away, for kind `k ∈ {Aliasing, Bounds, Stale, Overflow}` — see §X.1.1 |
| `Nondet` | an operation whose result may differ between two runs of the same program over the same inputs, or between two machines running the same binary — enumerated exhaustively in `[DET-2]` |

* `[EFF-1]` Effects are computed per function from its body and the (already computed) effects of its callees, over the call graph, with recursion handled by fixpoint (recursive SCCs are assumed to have the union of their members' direct effects).
* `[EFF-2]` Calls through `dyn` or function values contribute the effects declared on the interface method or function type; a method in an interface may declare `@noalloc` and implementers MUST satisfy it (`E4010`). A `fn(A) -> R` parameter type is assumed to carry all effects unless written `@noalloc fn(A) -> R`.
* `[EFF-3]` `extern` functions carry effects declared in their contract (`@ffi(effects=[FFI])` by default; `@ffi(effects=[FFI, Alloc, Block])` if the binding says so).
* `[EFF-4]` Effects are part of a function's public interface for the purpose of caching: a change to a callee's effect set invalidates callers' contract checks (Part XX build graph).

### X.1.1 The `RuntimeCheck` effect

`RuntimeCheck` records that Safe code obtained one of its guarantees by the runtime arm of `[PHIL-8]` rather than the static arm. Its four kinds:

| Kind | Emitted for |
|---|---|
| `Aliasing` | dynamic class exclusivity (`[EXC-1/2]`), `RefCell.borrow`/`borrow_mut` state checks (`[CELL-5]`) |
| `Bounds` | an index or slice check the compiler could not prove redundant |
| `Stale` | generational handle validation (`[HND-1]`, `[GPU-1]`), `Weak.upgrade` |
| `Overflow` | checked arithmetic under `@overflow(panic)` or the `debug` profile |

* `[EFF-9]` `RuntimeCheck(k)` enters a function's effect set when a check of kind `k` in that function's own body survives the profile-independent elision passes of `[EFF-15]`, and propagates through the call graph like every other effect (`[EFF-1..3]`). It is a statement about the code the contract profile would generate, not about source syntax and not about the selected profile.
* `[EFF-10]` **The effect is coarse; the site record is not.** The effect set carries only the kinds present, so that contracts can be checked cheaply and transitively. Separately, codegen emits a **safety-check side table** (`target/<profile>/inspect/<module>.safety.json`) with one entry per emitted check: `{kind, source span, function, mechanism, reason}`, plus one entry per *elided* check with the analysis that removed it. The side table is what `ember inspect --safety` reads (`[CLI-3]`). Implementations MUST NOT push per-site data into the effect lattice.
* `[EFF-11]` **Reason codes.** Every emitted-check entry carries exactly one reason, and diagnostics MUST use its wording rather than a generic "could not prove" message.

  | Reason | Meaning | What the programmer can do |
  |---|---|---|
  | `not_provable_in_principle` | no static analysis could establish the property — the value is genuinely runtime data (an index read from a file, a handle from a scene) | nothing; the check is correct and permanent |
  | `not_proven_by_analysis` | the property may hold, but this compiler's analysis did not establish it | restructure per the hint, or file a compiler issue — this is the only reason that is a candidate for future elision |
  | `requested_by_type` | the programmer chose a dynamically checked type (`RefCell`, `Cell`-free aliasing through a class) | the check is the type's purpose; change the type to change the cost |
  | `inherent_to_mechanism` | the check is what the mechanism *is* (a generation compare in a generational handle) | use a different mechanism (a direct reference, an index) if the cost matters |
  | `establishes_static_fact` | the check verifies a property once and returns proof-carrying values, so the property is static from there on (`[DSJ-1]`, `[DSJ-5]`) | nothing; the check is what makes the code after it checkable, and `[EFF-12]` permits it under `@static_safe` |

  `[EFF-11a]` Reporting `not_proven_by_analysis` where `not_provable_in_principle` is correct is a diagnostic bug: it tells the programmer to restructure code that cannot be improved. The conformance suite fixes the expected reason for each check site in `tests/safety/reasons/`.

* `[EFF-18]` **Effects are orthogonal.** `Io` (file, socket, console) and `Lock`
  (lock acquisition) are distinct from `Block` (may wait) and `Sync` (synchronisation
  semantics). One operation MAY carry several of them. In particular, a blocking
  `Mutex.lock` carries `Sync + Lock + Block`; a non-blocking `try_lock` carries
  `Sync + Lock`; a blocking file read carries `Io + Block`; and a non-blocking I/O
  operation carries `Io` without `Block`. `@noio`, `@nolock`, `@noblock` and
  `@nosync` therefore remain independent contracts. The full set is
  `{Alloc, Sync, Lock, Io, Panic, Unsafe, FFI, Block, Nondet, RuntimeCheck(k)}`. `[EFF-18]`
  does not remove an effect previously attached to any operation; it refines the
  effect model so a single operation may report all applicable effects. `RuntimeCheck(k)`'s kinds are `{Bounds, Overflow, Aliasing, Stale}` — the four `[EFF-16]` assigns and `[EFF-22]` permits. `Contract` went with the contract prover (OQ-28..OQ-32, owner decision 0.6.2). *(0.6.2 leftover removed 2026-09-09; see `docs/spec-amendments.md`)* *(ERR-028 applied 2026-09-09; see `docs/spec-amendments.md`)*
* `[EFF-19]` `@realtime` on a function expands to a configured contract set,
  by default `@noalloc @nolock @noblock @nopanic(explicit)`. It is a **marker
  for that set and not a timing guarantee**: the compiler MUST NOT state or
  imply that a `@realtime` function meets a deadline, because hardware,
  scheduling, cache behaviour and foreign code decide that and none of them is
  visible to it. The set is named in the manifest so a project can widen or
  narrow it once rather than per function.

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
* `[EFF-22]` **`@nopanic(explicit)` does not mean "cannot panic", and the diagnostics MUST say so.** It forbids `Panic(Explicit)` — the panics the programmer writes — and permits `RuntimeCheck(Bounds)`, `RuntimeCheck(Overflow)`, `RuntimeCheck(Aliasing)` and `RuntimeCheck(Stale)`, every one of which can still abort. The name is retained rather than changed, because it is threaded through `[EFF-16]`'s refinement and renaming it is churn against a distinction the effect set already makes precisely; the price of retaining it is that every diagnostic naming the attribute, and its `docs/errors/` page, MUST state the permitted checks explicitly. A function that reaches no abort at all is `@nopanic(explicit)` **and** free of `RuntimeCheck` in its effect set — two facts, reported together by `ember inspect --safety`, and claimed by no single attribute.
* `[EFF-16]` **`Panic` is refined.** `Panic(Explicit)` is introduced by: `panic`, `assert`/`assert_eq`/`assert_ne` (every profile), `unwrap`, `expect`, `todo`, `unreachable`, `as!` downcast, allocation failure (`[ALC-4]`), integer division or remainder by a divisor not proven non-zero, `i32.MIN / -1` where the operands are not proven safe, and a shift whose amount is not proven in range. Every other panic is recorded by the `RuntimeCheck(k)` kind that already covers it: bounds panics by `Bounds`, overflow panics by `Overflow`, exclusivity and `RefCell` borrow failures by `Aliasing`, generational-handle and `Weak.upgrade` failures by `Stale`. `Panic`, where this document uses it unqualified, means the union. `[EFF-9]`'s post-elision rule applies to `Panic(Explicit)` unchanged, computed **under `[EFF-15]`'s contract profile**, so the set does not vary with the optimisation level. It is therefore outside `[EFF-17]`'s `@nopanic(explicit)` and outside `[EFF-19]`'s default `@realtime` set, so a frame-path function MAY carry contracts.
* `[EFF-17]` **`@nopanic(explicit) fn` (v1).** MUST NOT have `Panic(Explicit)` in its contract effect set. Violations report the full call chain in `[EFF-5]`'s shape: `error[E4040]: @nopanic(explicit) function `resolve` reaches a panic: resolve → Option.unwrap → ember_panic_unwrap`. Bare `@nopanic` remains reserved for v2 and is defined as forbidding `Panic(Explicit)` together with `RuntimeCheck(Bounds)` and `RuntimeCheck(Overflow)`; `[OPT-2]` is what makes it satisfiable. `[EFF-6]`, `[EFF-8]` and `[EFF-2]` apply to it exactly as to the other contracts. `[EFF-14]`'s inner-loop set becomes `@static_safe @noalloc @nosync @nopanic(explicit)`. Bare `@nopanic` (v2) forbids `Panic(Explicit)`, `RuntimeCheck(Bounds)` and `RuntimeCheck(Overflow)`; it does not forbid `RuntimeCheck(Aliasing)` or `RuntimeCheck(Stale)`. *(0.6.2 leftover removed 2026-09-09; see `docs/spec-amendments.md`)* `@no_runtime_checks` (v2, reserved) excludes all five kinds.
* `[EFF-20]` **`@noio fn`**: MUST NOT have `Io` in its contract effect set. Violations report the **full call chain** in `[EFF-5]`'s shape: `error[E4041]: @noio function `tick` reaches I/O: tick → log_frame → File.write → ember_io_write`. `[EFF-6]`, `[EFF-8]` and `[EFF-2]` apply exactly as to the other contracts.
* `[EFF-21]` **`@nolock fn`**: MUST NOT have `Lock` in its contract effect set, in the same shape (`E4042`). **`@nolock` does not imply `@noblock` and `@noblock` does not imply `@nolock`**: per `[EFF-18]` a `try_lock` carries `Lock` without `Block` and a blocking channel receive carries `Block` without `Lock`, so a function needing both writes both.
* `[EFF-19a]` **A `@realtime` function's contract set is fixed by the package that declares it.** The expansion is read from the manifest of the package **containing the function**, is recorded in that package's build record and in its interface metadata, and MUST NOT be altered by a consumer. A consumer wanting its own code checked against a different set writes that set with the individual contract attributes, which `[EFF-14]` already composes. The language default, applied when the declaring package's manifest names no set, is `@noalloc @nolock @noblock @nopanic(explicit)` and is fixed by the language version under `[VER-2]`. **Widening or narrowing a package's `realtime` set is a breaking change to that package under `[VER-2]`**, for the reason `[EFF-15]` already gives: it changes which programs satisfy a contract.
* `[EFF-19b]` A diagnostic arising from a contract in `@realtime`'s expansion MUST name **both** the expanded contract that failed and `@realtime` as the source of the obligation, so a user who wrote one attribute does not receive an error about a different one.

## X.2a Determinism

Lockstep networking, deterministic replay, and golden-image tests all want the same
thing: two runs of the same program over the same inputs compute bit-identical
results, on one machine and across machines. Ember does not promise that globally —
it would forbid optimisations every other workload wants — so it is a hard contract
over a `Nondet` effect, in the manner of `[EFF-12]`'s `@static_safe`.

```ember
@deterministic
fn step(mut w: World, input: InputFrame):
    integrate(w.bodies, FIXED_DT)
    resolve_contacts(w.bodies)
    w.tick += 1
```

* `[DET-1]` `@deterministic` on a function is a **hard contract**: `Nondet` MUST NOT appear in its effect set. Violation is `E4070`, which names the offending operation and the shortest call chain that reaches it, per `[EFF-10]`.
* `[DET-2]` `Nondet` is introduced by, and only by:
  * floating-point contraction, reassociation, or any `@fastmath` relaxation (`[TYP-9]`, `[TYP-9a]`, `[TYP-9b]`);
  * a transcendental (`sin`, `cos`, `tan`, `exp`, `log`, `pow`, `atan2` and their variants) taken from the platform's math library rather than from `std.math.det`, because no two libms agree in the last bits;
  * observing a pointer or handle as an integer, and any hash whose input includes one — which is why `Hash` for `*T` and for a class handle carries it;
  * iteration over a container whose order its type does not specify (`Map`, `Set`);
  * the wall clock, the monotonic clock, the system random source, thread and job completion order, and `[JOB-*]` work-stealing order;
  * reading uninitialised or padding bytes;
  * any `extern` function not declared `@ffi(deterministic)`.
* `[DET-3]` Propagation is `[EFF-1]`'s: a `@deterministic` function may call only functions free of `Nondet`. `[EFF-2]` applies unchanged — an interface method may declare `@deterministic` and implementers MUST satisfy it (`E4010`), and a `fn(A) -> R` parameter is assumed to carry `Nondet` unless written `@deterministic fn(A) -> R`.
* `[DET-4]` `std.math.det` provides `sin`, `cos`, `tan`, `exp`, `log`, `pow`, `atan2` and `sqrt` for `f32` and `f64`, specified to return the same bits on every supported target. `sqrt` is the one exemption from software emulation: IEEE-754 requires it to be correctly rounded and every supported target's instruction is. These are slower than the platform's; that is the price of the guarantee, and `ember inspect` reports which of them a call graph reaches.
* `[DET-5]` Inside a `@deterministic` function the backend MUST NOT contract, reassociate, or introduce an FMA the source did not write. `[TYP-9a]`'s translation-unit-wide `FP_CONTRACT OFF` already achieves this; `@fp(contract)` (`[TYP-9b]`) or `@fastmath` on a `@deterministic` function is `E4072`.
* `[DET-6]` `@deterministic` constrains **results, not timing**. A deterministic function may allocate, lock, block, and take a different amount of time on every run. It composes with `@noalloc` and `@realtime` and substitutes for neither.
* `[DET-7]` **Scope of the cross-machine claim.** Two machines running *the same binary* compute identical results for a `@deterministic` call graph. Two machines running binaries built from the same source by different toolchains, or for different targets, do not — `[DET-4]`'s guarantee is tied to emitted code — and this specification does not claim otherwise. Lockstep peers MUST therefore agree on the build, which `[BLD-13]`'s `--build-id` makes checkable in one comparison before the match starts.
* `[DET-8]` `@deterministic` on a module applies to every function it declares, and an individual function may not opt out. `@deterministic` on an `extern` block is `E0104`: a foreign function's determinism is *asserted*, with `@ffi(deterministic)`, and is an `asserted` fact in the sense of `[FFI-35]` that appears in `ember tcb` as one. `[PHIL-5]` applies — declining to look is not a proof, and an unbacked assertion is recorded as unbacked.
* `[DET-9]` `ember inspect --deterministic <path>` prints whether the named item satisfies `[DET-1]`, and if not, the shortest call chain to each `Nondet` source. A `@deterministic` function whose body reaches no `Nondet` source at all, in a build where every `@ffi(deterministic)` fact is `asserted`, is reported as such rather than as verified.

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

## X.4 The cost model

Ember claims C-like performance, and a claim like that is worth nothing unless it
says what it means. This section defines "zero-cost" for this document and classifies
every implicit cost the language can introduce, so that a programmer can answer
"what does this line cost" from the specification rather than from a disassembler.

* `[COST-1]` **Zero-cost, defined.** An abstraction is **zero-cost** when, for a use
  in which every dynamic check it implies has been statically discharged, the
  emitted code contains no instruction that the equivalent hand-written C would not
  contain, under the optimisation model of the selected backend and profile. Two
  consequences follow and are part of the definition: an abstraction is zero-cost
  *for a use*, never in general — the same generic is zero-cost where its checks are
  discharged and not where they are emitted; and "the equivalent C" means C that
  upholds the same invariant, not C that omits it. A bounds check the compiler
  cannot discharge is not a failure of zero-cost, it is the cost of the guarantee,
  and `[COST-3]` classifies it as such.
* `[COST-2]` **Five classes.** Every implicit cost in this document is classified as
  exactly one of:
  * **Guaranteed elidable** — an implementation MUST NOT emit it when the stated
    condition holds; emitting it anyway is a defect, not a quality-of-implementation
    matter.
  * **Guaranteed required** — always emitted; the semantics depend on it.
  * **Conditionally elidable** — an implementation MAY elide it when it can prove
    the condition; whether it did is reportable through `ember inspect`.
  * **Implementation-defined** — the specification does not constrain it; two
    conforming implementations may differ.
  * **Not observable** — no cost exists at runtime; the construct is erased.
* `[COST-3]` **The classification.**

| Cost | Class | Condition / note |
|---|---|---|
| Bounds check on `a[i]` | conditionally elidable | discharged by `[OPT-2]`'s range analysis or a `[RNG-4]` fact; `ember inspect --safety` reports each site |
| Overflow check | conditionally elidable | `debug` only under `[TYP-8]`; `release` wraps and emits nothing |
| Retain / release pair | guaranteed elidable | `[RC-2]`'s pairing rule — a retain immediately dominated by its release over a non-escaping handle MUST be removed |
| Retain / release, surviving | guaranteed required | reported per call site by `ember inspect`; atomic iff the class is `Sync` (`[RC-1]`) |
| Dynamic exclusivity check | conditionally elidable | discharged where `[EXC-3]` proves the access static; unchecked in `shipping` under `[PRF-1]` exception (2) |
| Stale-handle (generation) check | guaranteed required | `[HND-1]`'s guarantee is the check; `get_unchecked` is the `unsafe` opt-out |
| Interface dispatch through `dyn` | guaranteed required | one indirect call per method (`[TYP-22]`) |
| Generic call, specialised | not observable | direct call, inlinable (`[TYP-16]`) |
| Generic call, shared | guaranteed required | one indirect call per bound method (`[MONO-6]`), taken only under `[MONO-3]`'s ceiling |
| Class method dispatch | conditionally elidable | devirtualised where the concrete class is known (`[DSP-*]`) |
| Reload thunk indirection | guaranteed required in a reloadable build, not observable otherwise | `[HR-6]`, ≤ 3% by `[HR-9]`; absent in `shipping` |
| FFI thunk | guaranteed required | one call; inlinable across the boundary only under `[BLD-FFI-4]`'s shared-compiler case |
| Coroutine resume | guaranteed required | one indirect jump on the state discriminant (`[CORO-4]`); the frame itself does not allocate (`[CORO-5]`) |
| `str` ← foreign bytes | guaranteed required | O(n) UTF-8 validation (`[TXT-2]`); no allocation |
| `x in coll` | conditionally elidable · else guaranteed required | one `Contains.contains` call, inlinable when the impl is small; **the complexity is the impl's, not the operator's** — O(1) for `Map`/`Set`, O(n) for `Array`/`Span`/`str`, and `ember inspect --cost` names which (`[STD-8a]`) |
| `Cell` access | not observable | a load or store |
| `RefCell` borrow | guaranteed required | one word, every profile (`[CELL-9]`) |
| Effect annotations (`@noalloc`, `@deterministic`, …) | not observable | compile-time only |
| Range type (`[RNG-1]`) | not observable | erased to the representation; construction is where the check lives |
| Arena allocation | guaranteed required | a pointer bump; the reset is one store |

* `[COST-4]` `ember inspect --cost <path>` prints, for the named item, every row of
  `[COST-3]` that applies to it with its resolved class — elided or emitted, and for
  a conditionally elidable cost, which condition decided it. This is the same
  machinery `[EFF-10]`'s chains and `ember inspect --safety` already use, presented
  per item rather than per check.
* `[COST-5]` A rule **added or amended after 0.8** that introduces an implicit cost
  MUST add a row to `[COST-3]`, and `tools/rule_index.py` fails CI on such a rule
  with no row. For rules that predate this section the check is a **report, not a
  failure**: `[COST-3]` is asserted to be complete over 0.8's rule set, and any gap
  the report finds is an erratum against this section rather than a build break —
  the table is the claim being checked, so a check that failed the build would only
  measure how confident the claim was.

---

# Part XI — Concurrency and Parallelism

## XI.1 Thread safety markers

* `Send`: a value may be moved to another thread. Auto-derived (`[TYP-*]`). Not `Send`: `*T`, `*mut T`, `ref`/`ref mut`/views (in v1 — scoped threads relax this, §3), non-`Sync` class handles, `Shared[T]` where `T: !Sync`.
* `Sync`: a value may be *shared* (borrowed) by multiple threads simultaneously. Auto-derived when all fields are `Sync`. `ref mut` is never `Sync`. Interior mutability primitives (`Cell[T]`, `RefCell[T]`; Part IX §7) are `!Sync`; `Atomic[T]`, `Mutex[T]`, `RwLock[T]` are `Sync` (when `T: Send`).
* `[THR-1]` A class is `Sync` iff **every** field's type is itself an interior-synchronised type (`Atomic`, `Mutex`, `RwLock`, channel end) or a deeply immutable value type. **There is no `let` exemption** (`OQ-18`): `let` restrains rebinding, not mutation through the field (`[CLS-9a]`), so exempting `let` fields would make a class `Sync` whose contents a `mut self` method can still mutate — a data race in Safe code. A class that needs a shared mutable field uses `Mutex[T]`/`RwLock[T]`; one that needs a frozen field uses a type with no `mut self` API. — because handles alias, a plain mutable field shared across threads would be a data race. `@sync class` asserts `Sync` and is `E7001` if the rule is violated; `@thread_local class` forces `!Sync` even if the fields would allow it (to get non-atomic counts).
* `[THR-2]` Class handles are `Send` iff the class is `Sync` (a sent handle can be copied on both sides). **What `Sync` claims is that *sharing a handle* across threads is safe** — that the reference count and any other per-object bookkeeping tolerate concurrent access, and that no field can be mutated through a shared handle without its own synchronisation. It does **not** mean that arbitrary mutation of the object's state is race-free: `[THR-1]`'s structural test is satisfied precisely because every mutable field is already an interior-synchronised type, and it is `Atomic`, `Mutex` and `RwLock` — not `Sync` — that make a particular mutation safe. A diagnostic MUST NOT describe `Sync` as making a type thread-safe. *(clarified 2026-09-09; see `docs/spec-amendments.md`)*
* `[THR-7]` **What `Sync` does and does not mean.** `Sync` on a class means two things and no more: its handles may cross threads (`[THR-2]`), and its reference count is atomic so the *lifetime* is safe under concurrent handle traffic. It does **not** mean the object's fields may be mutated concurrently. Field-level race freedom comes from the ordinary rules — a `mut self` method needs exclusive access, which across threads means a `Mutex`/`RwLock` or an `Atomic` field — and `[PHIL-10]`'s data-race guarantee holds because those rules apply to a `Sync` class exactly as to any other, not because `Sync` waives them. The derivation is written so this stays true: a `let` field is not exempt from the `Sync` requirement, precisely because `let` restrains rebinding and not mutation through the field (`[CLS-9a]`), and exempting it would produce a `Sync` class whose contents a `mut self` method could still race on.

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
* `[SOA-5]` `columns_mut(f0, f1, …)` takes **field-name arguments**: identifiers resolved against `T`'s fields at compile time rather than as expressions. It is the only construct with this argument kind; a name that is not a field of `T` is `E2020`. This is an argument *kind*, not overloading (XXIII.2 unaffected).

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
* `[ECS-7]` `@derive(Component)` registers the type in a comptime component registry used by serialisation and by the editor bridge (Part XXII).

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

* `[CT-1]` `comptime:` blocks and `comptime fn` functions are executed by the **MIR interpreter** (Part XIX §7) during compilation. Any Ember function whose transitive effect set ⊆ `{Panic}` and that uses only `comptime`-supported operations may be called at compile time — there is no separate sub-language. Supported: all arithmetic, structs, enums, `Array`/`String`/`Map` (interpreted heap), `match`, loops, recursion, `assert`; the interpreter emulates target endianness, pointer size and `@layout(c)`.
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
* `[RFL-3]` `@reflect` fields may carry user attributes readable at runtime (`@ragev.field(range=(0, 1), tooltip="…")`) — the editor bridge in Part XXII uses this exactly as RageV's `RVShowInEditor` markers are used by `rvgen` today.

## XIV.3 Derives

`@derive(...)` invokes compiler-built-in generators. v1 set: `Copy, Clone, Debug, Display(field="…")`, `Eq, Ord, PartialOrd, Hash, Default, SoA, Component, Error, Serialize, Deserialize, Zeroable, Reflect`. `[DRV-1]` Each derive is specified as an equivalent hand-written `extend` block in `std/derive/*.em` (the reference), and the generator MUST produce the same MIR. `[DRV-2]` User-defined derives (procedural macros) are **v2**; the mechanism will be a `comptime fn derive_X(t: TypeDesc) -> Source` sandboxed in the interpreter.

## XIV.4 Serialization

`@derive(Serialize, Deserialize)` generate `fn serialize(self, mut w: ref dyn Writer) -> Result[void, SerError]` / `fn deserialize(mut r: ref dyn Reader) -> Result[Self, SerError]` using a **binary, versioned, field-tagged** format (`std.ser.binary`) and a YAML mapping (`std.ser.yaml`, to interoperate with RageV's `yaml-cpp` scene files). Field attributes: `@ser(skip)`, `@ser(rename="…")`, `@ser(default)`, `@ser(version=2)`.

---

# Part XV — Standard Library Surface (v1)

The standard library is one package `std` with the modules below. Each module's public surface is listed at the level needed to implement it; exact signatures live in `std/**/*.em` and are the normative source once written. `[STD-1]` The whole of `std.core`, `std.mem`, `std.math`, `std.simd`, `std.span`, `std.arena` MUST be `@noalloc`-clean except functions documented to allocate.

| Module | Contents |
|---|---|
| `std.core` (prelude) | `Option`, `Result`, prelude re-exports of `Cell` and `RefCell`, marker & operator interfaces, `Ordering`, `Iterator` adaptors (`map`, `filter`, `enumerate`, `zip`, `take`, `skip`, `chain`, `rev`, `sum`, `count`, `min_by`, `max_by`, `fold`, `any`, `all`, `find`, `position`, `collect[C]`), `Range*`, `print`/`println`/`eprintln`, `assert*`, `panic`, `todo`, `unreachable`, `mem.{take, replace, swap, drop, forget, size_of, align_of}` |
| `std.mem` | `MaybeUninit`, `transmute`, `zeroed`, `copy`, `copy_nonoverlapping`, `Layout`, `Allocator`, `Global`, `ptr.*` (unsafe pointer ops), `keep_alive` |
| `std.cell` | **Defining owner** of `Cell`, `RefCell`, `Ref`, `RefMut` (Part IX §7) |
| `std.borrow` | canonical `with_views` helpers for 2, 3 and 4 independently borrowed views; allocation-free late-bound callback composition (`[LT-8]`..`[LT-12]`); multi-region view utilities MAY be layered here but require no compiler intrinsic |
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
| `std.ffi` | `CString`, `cstr`, `c_int`… type aliases (`c_int` = target `int`), `Callback[F]`, `Retained[T]` (foreign-retained handle), `ForeignBox[T]` (owned foreign pointer with destructor fn), `NativeApiTable` helpers (Part XXII) |
| `std.gpu` | Part XVII (host-side; backend-agnostic) |
| `std.debug` | `backtrace`, `leak_report`, `alloc_stats`, `frame_profiler` markers (`zone("name")` scoped) |

`[STD-2]` `print` and friends allocate only for f-strings; `println("literal")` and `println(some_str)` are `@noalloc`.
* `[STD-3]` `std.math` MUST provide scalar `fma(a: f32, b: f32, c: f32) -> f32` (and the `f64` form) and `Vec2/3/4.mul_add`, lowered to `fmaf`/`fma`/`_mm_fmadd_ps`/`vfmaq_f32`, so that fused multiply-add is expressible as an explicit, IEEE-defined operation **with no float-control attribute at all**. These are the recommended form for `Mat*` multiply, `dot` and transform composition, and `std.math` MUST use them internally.
* `[STD-4]` `std.core` provides `NonZero[T]` for each integer `T`: a `Copy` newtype with a niche (`Option[NonZero[T]]` is `T`-sized per `[TYP-13]`), constructed by `NonZero.new(v) -> Option[NonZero[T]]` or `unsafe new_unchecked`. Division by a `NonZero` divisor carries no `Panic(Explicit)`. This is the mechanism by which a divide in a `@nopanic(explicit)` function is expressible. `debug_assert*` is permitted in `release` and `shipping`, where it compiles to nothing.

---
* `[STD-6]` `std` is **layered**, and the layers are a build-time choice, not a
  convention: **core** (primitives, `Option`, `Result`, views, fixed-capacity
  containers, math), **alloc** (`Array`, `String`, `Box`, `Map`), **sync**
  (`Atomic`, `Mutex`, channels), **io** (files, sockets, console) and **ffi**.
  (0.6.2 removed the **verify** layer with the prover it served; OQ-32.) *(0.6.2 leftover removed 2026-09-09; see `docs/spec-amendments.md`)* A layer may depend only on the layers before
  it in that order.
* `[STD-8]` **`Contains`.** `std.core` declares `interface Contains[T]: fn contains(self, item: T) -> bool`. `x in coll` requires `typeof(coll): Contains[typeof(x)]` and is `E2226` otherwise, whose `help` names the explicit form — `coll.iter().any(|e| e == x)` — so the fix is always available and always visibly O(n). `std` implements it for: `Set[T]` and `Map[K, V]` (**by key**, matching `[STD-*]`'s iteration of a `Map` as pairs — `k in map` tests the key and never a value); `Array[T]`, `Span[T]`, `MutSpan[T]` and `[T; N]` where `T: Eq`, **as a linear scan**; `str` and `String`, as a substring test; and `Range[T]`, as two comparisons.
* `[STD-8b]` **`str` membership is by Unicode scalar, never by byte.** `str` and `String` implement `Contains[char]` — does the string contain that scalar value — and `Contains[str]`, a substring test matching only at a codepoint boundary, and **nothing else**. `Span[u8] in str` is `E2226`: a byte needle has no well-posed answer in a type whose invariant is that it is valid UTF-8 (`[TXT-1]`), and admitting it would let `x in s` be true for a needle that is not a substring of `s` as any reader sees it. A `char` needle therefore can never match a continuation byte, and a `str` needle never matches a split codepoint — `[TXT-4]`'s boundary rule applied to search rather than to slicing. Byte-level search stays available, and well-posed, on `Span[u8]`, which is what `s.as_bytes()` returns.
* **`a not in b` is defined as `not (a in b)`**, and therefore as `not b.contains(a)`. Each operand is evaluated exactly once and in the order written, `contains` is invoked exactly once, and the result is its boolean complement. An implementation MUST NOT lower it to a second search, nor to anything that evaluates either operand twice — `xs.pop() not in ys` removes one element, not two — and its side effects, cost and `[COST-3]` row are `in`'s. `E2226` applies unchanged: a type with no `Contains` impl is as much an error under `not in` as under `in`. *(clarified 2026-09-09; see `docs/spec-amendments.md`)*
* `[STD-8a]` **`in` never hides its cost.** `Contains` is an ordinary interface method, so `[COST-3]`'s row governs and `ember inspect --cost` reports the complexity of the implementation actually selected. This is the whole reason the operator lowers to a **declared bound** rather than to a compiler-synthesised scan: `id in players` on a `Map` is a hash lookup and on an `Array` is a scan, the two are told apart by the container's type, and a container that has no cheap answer either implements `Contains` and is reported as linear or does not implement it and is a type error. An implementation MUST NOT synthesise a `Contains` impl for a type that does not declare one.
* `[STD-7]` `core` provides `FixedArray[T, N]`, `FixedString[N]` and
  `RingBuffer[T, N]`: compile-time capacity, no heap allocation, and an explicit
  policy when full — `push` returns `Result`, `push_or_drop` does not. They
  take an integer capacity parameter under IV.7's `const N: usize` generic form,
  which v1 has. `[STD-7a]` records that **arbitrary const-generic expressions**
  — arithmetic over capacity parameters — are a milestone, not a gap in the
  parameter form itself.

* `[STD-7a]` General const-generic support for integer capacity parameters is a
  prerequisite for the unbounded `FixedArray[T, N]`, `FixedString[N]` and
  `RingBuffer[T, N]` forms. The implementation plan therefore contains an explicit
  **const-generic milestone before the fixed-capacity container milestone**. A
  bootstrap implementation MAY expose a finite set of capacities temporarily, but
  it MUST diagnose a non-supported capacity as a capability limitation rather than
  as a malformed type, and it MUST NOT claim `[STD-7]` complete until arbitrary
  compile-time `N` is accepted. `[GRM-8]`, `[CT-1]`, `[MONO-1]` and `[TYP-19]` govern
  parsing, constant evaluation, monomorphisation and identity of const-generic
  arguments respectively.


## XV.4a The string and text model

Strings are the highest-frequency interoperability type in a language whose purpose
is migrating a C++ codebase, and 0.7.1 spread their rules across `[STD-*]`,
`[FFI-17]`, `[UNS-4]` and Part IV. This section is normative and supersedes any
weaker statement elsewhere.

* `[TXT-1]` **Four types, one guarantee.** `str` is a borrowed `Span[u8]` **known to
  be valid UTF-8**; `String` owns a heap buffer with the same guarantee; `CppString`
  is an opaque owned `std::string` (`[FFI-17e]`); `*const c_char` is a foreign
  pointer with no guarantee at all. Only the first two carry the UTF-8 invariant,
  and `[UNS-4]` lists it among the invariants safe code may assume.
* `[TXT-2]` **Nothing becomes a `str` without validation.** Every conversion from
  foreign or untrusted bytes — `CppString.as_str()`, `std::string_view`,
  `*const c_char`, a `Span[u8]` — is fallible and returns `Result[str, Utf8Error]`,
  or its `_lossy` form which substitutes U+FFFD and is total. The importer MUST NOT
  map `std::string` or `std::string_view` to `str` directly; it maps them to
  `CppString` and `Span[u8]` respectively, from which `[TXT-2]`'s conversion is the
  only route. An unvalidated route is `E5063`. This closes the hole by which a
  `yaml-cpp` scalar, a `cgltf` name, an ImGui buffer or a CP-1252 path could enter
  safe Ember as a `str` and be walked by a UTF-8 decoder.
* `[TXT-3]` **No null termination.** `str` and `String` are pointer + length and MAY
  contain interior nulls. A C boundary needs `CStr`/`CString` (`std.ffi`), whose
  construction from a `str` is fallible on an interior null (`E5064` at compile time
  where the value is a literal, `Result` otherwise) and which is the only form the
  importer accepts where a header says `const char*`.
* `[TXT-4]` **Slicing is by byte index and rejects a split codepoint.** `s[a..b]`
  panics in `debug`/`release` when either bound is not a codepoint boundary, and
  `s.get(a..b) -> Option[str]` is the total form. Indexing by codepoint is not
  provided: it is O(n) and hides that cost.
* `[TXT-5]` **Conversion costs are stated, and `ember inspect --alloc` reports
  them.** `str → String` allocates and copies. `String → str` is free.
  `CppString → String` allocates, copies and validates. `str → CStr` allocates
  unless the source is a literal, which the compiler null-terminates statically.
  `Span[u8] → str` validates in O(n) and does not allocate.
* `[TXT-6]` **The C ABI representation of `str` is `{const u8*, usize}`**, matching
  `Span[u8]`, and it is **not** `const char*`. A `str` parameter in an `@export`ed
  function appears in the generated header as that pair; `[BLD-FFI-5]`'s C++ header
  additionally offers a `std::string_view` overload, which is the zero-copy form.
* `[TXT-7]` Encoding is UTF-8 everywhere. There is no UTF-16 or wide-string type in
  the language; a Windows API needing `wchar_t*` gets it from `std.ffi`'s
  `WideString`, an explicitly-converting owned buffer, and the conversion is visible
  at the call site rather than implicit.
* `[TXT-8]` `str` is a view type (`[TYP-15]`) and carries a region: a `str` borrowed
  from a `CppString` cannot outlive it, and the borrow checker enforces it exactly
  as for any other `Span`.

## VIII.5a Cycle diagnosis

* `[WK-4]` **The leak reporter names the cycle, not just the leak.** `[WK-1]`'s
  debug runtime reports leaked instances at shutdown; where the leaked set contains
  a strong-reference cycle it MUST additionally report the **shortest strong cycle**
  through each leaked object, as a path of `Type.field` edges, and suggest which
  edge to weaken:

```
L3017: reference cycle detected — 3 objects, 1 cycle

  World
    └─> EntityManager        World.entities
         └─> Entity          EntityManager.dense[i]
              └─> World      Entity.world          ← suggested Weak

  make `Entity.world` a `Weak[World]`; every other edge on this cycle is
  load-bearing (removing it would orphan the object)
```

  The suggestion names the edge that closes the cycle back to the object with the
  most incoming strong references, which is the owner in every ordinary graph shape.
  This is a diagnostic, not a collector: `[WK-1]`'s statement that cycles leak is
  unchanged, and no tracing pass exists in any profile.

## IX.5a Unsafe categorisation

* `[UNS-9]` **Every `unsafe` block carries a machine-readable reason category**, in
  addition to `[UNS-7]`'s prose obligation: `unsafe(reason = "ffi" | "layout" |
  "aliasing" | "intrinsic" | "performance" | "uninit")`. A block without one is
  `L3018` under `edition_lints = "strict"` and a warning otherwise; the category is
  a fixed closed set, because a free-form field would not be aggregable and the
  point of the field is the aggregate. `ember tcb` reports the distribution:

```
unsafe surface: 17 blocks in 6 modules
  ffi          8    (all in ffi/vulkan.em — the generated boundary)
  layout       3
  aliasing     2
  performance  4
```

  `[UNS-9a]` The category is advisory to the compiler and normative to the audit:
  it changes no check and no codegen, and `[TCB-*]`'s reports MUST carry it so that
  a reviewer can ask "how much of our unsafe surface is performance work we could
  delete" and get an answer.

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

`[FFI-17]` The C++ importer produces, for each requested class/function, what the
numbered list below describes. **That list is a reader's summary and is
`NON-NORMATIVE` under `[CAT-1]`**: where it and a rule disagree, the rule governs,
and the rule is named in each item. This demotion is not cosmetic — the list has
been the site of four contradictions with the rules beside it (`std::function`,
`std::string_view`, C++ inheritance, and the CRT device attributed to `[FFI-30]`),
because a prose restatement of a rule drifts from it and nothing detects that. A
future revision should delete from the list every claim a rule already makes rather
than keep two copies in step.

1. A **thunk file** `target/bind/<hash>_thunks.cpp` containing `extern "C"` functions with predictable names (`em_cpp_<ns>_<class>_<method>_<sig-hash>`) that (a) call the C++ member/free function, (b) catch all exceptions and return a status + message buffer (§9), (c) never return C++ objects by value across the boundary — objects are heap-allocated (`new`) and returned as opaque owned pointers, or written into caller-provided storage when trivially copyable.
2. Ember declarations in the synthetic module: an `extern type RHIDevice`, a `struct RHIDeviceRef`/`ForeignBox[RHIDevice]` pairing, and methods on them that call the thunks. Constructors become `RHIDevice.new(...) -> ForeignBox[RHIDevice]`; the destructor is the box's `drop`. Overloads are disambiguated by suffixing parameter types (`draw_indexed_u32`) unless the overlay names them.
3. Templates are only available as **explicit instantiations** listed in `instantiate` or referenced by an imported signature; each instantiation gets its own thunks. `std::vector<T>` maps to `CppVector[T]` with `.span()`/`.span_mut()` (zero-copy, borrowing the vector), `.to_array()` (copy), `push_back`, `len`; `std::string` to `CppString` with `.to_str() -> Result[str, Utf8Error]`; `std::string_view` to `Span[u8]` (**not** `str` — `[TXT-1]`/`[TXT-2]` govern, and a `string_view` carries no UTF-8 guarantee); `std::unique_ptr<T>` to `ForeignBox[T]`; `std::shared_ptr<T>` to `CppShared[T]` (calls the C++ control block through thunks); `std::optional<T>` to `Option[T]` for trivially copyable `T`; `std::function` is not importable (`E5030`) — use a C callback.
4. **Compiler matching** `[FFI-18]`: the thunks are compiled by the *same* compiler, standard and flags as the project (`[cpp.ragev]` section), so they share the C++ ABI with the engine (MSVC ABI when the engine is built with MSVC; clang-cl also targets the MSVC ABI). libclang parses the headers in MSVC-compatibility mode with the same `_MSC_VER`. `[FFI-19]` If the project's compiler is MSVC and libclang cannot parse a header (MSVC-specific extension), the importer reports the exact diagnostic and the declaration is skipped with `W5031`; the programmer then binds it manually through a small C shim.
5. Inheritance and virtual functions: a C++ class hierarchy is imported as opaque types with **upcast thunks**, and **`[FFI-39]` governs subclassing, which v1 supports in a bounded form** — a single foreign base declared `@ffi(trampoline, virtuals=[…])`, an enumerated virtual set, a generated trampoline subclass, `super.init` selecting a base constructor, declared ownership in either direction, derived-first destruction, and a panic boundary on the inbound path. Multiple inheritance, virtual bases and overriding a virtual not named in `virtuals=[…]` remain unsupported (`E5056`, `[FFI-48]`). An earlier draft of this item said Ember cannot subclass C++ classes in v1 and dated the trampoline to v2; that predates `[FFI-39]` and is withdrawn.
6. Namespaces map to Ember module paths under the import name: `cpp.RageV.RHIDevice`.
7. `constexpr`/`const` integral constants and unscoped/scoped enums import like C.


### Standard-library mapping and its order (`[FFI-17a]`)

* `[FFI-17a]` The importer supports the standard-library types below. **P0 types MUST be supported before
  any P1 type, and P1 before any P2**, because a header that uses a P0 type is not usable at all until that
  type maps — `std::span` and `std::string_view` appear in the signature of almost every modern C++ API,
  so an importer that handles `std::vector` first still cannot import the header. A type outside this table
  is opaque and reachable only behind a pointer, reported by `[FFI-20]`.

| C++ type | Priority | Ember mapping |
|---|---|---|
| `std::span<T>` / `std::span<const T>` | **P0** | `MutSpan[T]` / `Span[T]`, region from `[FFI-11d]`'s `from =`. Call-scoped in parameter position; in result position `from =` is required as for any returned view. Crosses a thunk as pointer-plus-length |
| `std::string_view` | **P0** | `Span[u8]`, region from `from =`; `[TXT-2]` supersedes the 0.7.1 mapping to `str`, because a `string_view` carries no UTF-8 guarantee and `[UNS-4]` lets safe code assume one. `.to_str()` is the fallible conversion. **Not NUL-terminated** — the importer MUST NOT pass it where a `cstr` is expected |
| `std::unique_ptr<T>` | P1 | `ForeignBox[T]`; the deleter must be the default or named by the overlay, and its drop calls that deleter through a thunk |
| `std::optional<T>` | P1 | `Option[T]` where `T` maps, by value across the thunk. `std::nullopt` is `None` |
| `std::vector<T>` | P1 | `CppVector[T]` with `.span()`/`.span_mut()` (zero-copy, borrowing the vector), `.to_array()` (copy), `push_back`, `len`. **Never** a layout assumption |
| `std::string` | P1 | `CppString` with `.as_str()`; conversion to `String` is an explicit copy |
| `std::shared_ptr<T>` | P2 | `CppShared[T]`, a foreign handle over the C++ control block. **It is not `Shared[T]`** — Ember's `Shared[T]` uses Ember's own count, and conflating the two would double-free |
| `std::weak_ptr<T>` | P2 | `CppWeak[T]`, with `upgrade() -> Option[CppShared[T]]` calling `lock()` through a thunk. **Not `Weak[T]`**, for the reason above |
| `std::variant<Ts...>` | P2 | an Ember `enum` generated from the alternatives, provided every alternative maps and the overlay names each one. Otherwise opaque |

* `[FFI-17b]` **Templates are available only as explicit instantiations.** A template type or function is
  importable when it is listed in `instantiate` or appears in an imported signature, so that libclang can
  supply a concrete instantiation whose layout and mangled name are known. Passing an Ember generic into a
  C++ template is out of scope for v1 (`E5055`): it would require Ember to reproduce C++ overload resolution
  and template instantiation, which Part 0's C-backend decision exists to avoid.

`[FFI-20]` Everything an importer cannot represent is reported once with the reason (`ember bind --report`), never silently dropped.

## XVI.7a Grading, adoption and instrumentation

XVI.2 to XVI.7 specify how a foreign declaration is imported and what an overlay
may say about it. This section is about **how much any of it is worth**: in 0.5 a
fact the importer derived from a header and a promise somebody typed into an
overlay are written the same way and are indistinguishable afterwards.

```ember
overlay cpp "RageV/VulkanBackend.hpp":

    ## a grade per fact; anything ungraded is `asserted`
    @ffi(effects=[FFI] @instrumented, threads=main @checked)
    fn submit(self: borrowed, cmd: borrowed) -> status
```

* `[FFI-34]` **Every fact in an overlay carries a grade** from `[TCB-1]`, written
  after it and defaulting to `asserted`. A grade above `asserted` requires its
  evidence: `checked` requires the fact to follow from the header, the ABI or the
  build configuration; `instrumented` requires a run under `[CLI-14]` to have
  observed it holding; `proven` requires the adapter itself to have been
  analysed. Claiming a grade whose evidence is absent is `E5050`. Replace "Claiming a grade whose evidence is absent is `E5050`" with: "Claiming a grade whose evidence is absent or stale is `W5050 unbacked grade`: the fact is **reported and used at grade `asserted`**, the report names the missing or stale record and the command that would produce it, and the claimed grade is never honoured. `E5050` is raised instead of the warning under `ember build --require-evidence`, `ember tcb --require` and `ember audit --require`, which is where a project that wants the gate puts it. This is a **reporting strictness**, not an input to acceptance: `[PRF-1]` and `[EFF-15]` forbid a build from accepting or rejecting a program on the strength of an artefact that may or may not exist on this machine."
* `[FFI-35]` **A foreign pointer or reference in return position imports unsafe.** A `T&`, `const T&`, `T*` or `extern type` handle that a foreign function **returns**, or writes through an `out` parameter, produces a raw foreign pointer requiring an `unsafe` block to use, until an overlay supplies both a lifetime fact (`[FFI-11d]`) and an aliasing fact (`[FFI-11e]`). A header does not record how long a returned reference lives or who else may hold it, and mapping one to a safe borrow on the strength of its spelling is how a dangling pointer acquires the borrow checker's endorsement. In that position `borrowed` becomes an unknown-filling contract and joins `[FFI-2a]`'s list, so an overlay that promotes a returned pointer MUST be an `unsafe overlay` (`[TIER-1]`). A pointer or reference in **parameter** position is unchanged: `[FFI-2a]`'s and `[FFI-32b]`'s derivations stand, `borrowed` remains a derived fact requiring no `unsafe`, and a callee that retains the pointer beyond the call is described by `retained` as it always was (`E5051`). `const T&` in parameter position maps to borrowed mode as stated; in return position `[FFI-35]` applies.
* `[FFI-36]` **Ownership transfer is explicit where the interface does not record it.** A foreign call returning a bare `*T`, `*mut T` or `extern type` handle **whose overlay carries no ownership contract** yields that raw representation; it becomes an owned Ember value only through `unsafe adopt(handle)`, which produces `ForeignBox[T]`. `adopt` is an `unsafe` operation under `[TIER-1]` boundary (1) and introduces no fourth boundary. Where the interface *does* record the transfer, the importer produces `ForeignBox[T]` directly and no `adopt` is written: (a) an overlay declaration carrying `returns_owned(destructor=f)` (`[FFI-11]`); (b) a C++ signature returning `std::unique_ptr<T>` (`[FFI-17]`.3); (c) an imported C++ constructor (`[FFI-17]`.2). In cases (a)–(c) the human or the C++ type system has already written the transfer down and a second token adds no fact the compiler lacks. `ForeignBox[T]` remains the sole standard owned representation for a foreign object; there is no `Foreign[T]` type.
* `[FFI-37]` **Declared effects are checked, not believed.** A run under
  `[CLI-14]` links a shim recording allocation, blocking, locking and I/O across
  every foreign call and reports each declared effect fact as confirmed or
  contradicted. A contradicted fact fails the run and names the overlay line
  (`E5053`). This is what moves "submit does not allocate" from a promise to a
  measurement, and it is the only route to the `instrumented` grade.
* `[FFI-38]` The importer MUST reject rather than guess. Where ownership,
  nullability, lifetime or exception behaviour cannot be established, the
  declaration imports unsafe and `[FFI-20]` reports why. Convenient-but-unsound
  is not an import mode. Strike "or exception behaviour" — `[FFI-24]`'s catch-all establishes it universally — and add: "Where the exception specification is dependent and unevaluated, `[FFI-24]`'s catching shape applies and `[FFI-20]` reports it."

**Which foreign code to replace first.** Not normative, and stated so that the
order is not re-argued each time. Good candidates have a clear boundary, few
dependencies, bugs that cost real time, no template machinery in the interface,
and ownership that is already understood — in the reference workload: frame
orchestration, gameplay, ECS systems, tools, editor logic, resource bookkeeping.
Poor candidates are macro-generated code, plugin ABIs, pervasive global state,
shared ownership with custom deleters, and template-heavy headers with
undocumented lifetimes — in the reference workload: the Vulkan and OpenGL
backends, the platform layer, and the allocators. Those stay native, and the
bridge around them is where the contracts and grades concentrate.


### XVI.7b C++ compatibility classification (`[FFI-44]`)

* `[FFI-44]` **What imports, and how.** This table is normative and is the answer to
  "can Ember consume this header". *Automatic* means the importer handles it with no
  overlay; *overlay* means it needs a declared contract; *native island* means it
  stays in C++ behind a hand-written boundary and no importer support is planned.

| C++ construct | v1 | Notes |
|---|---|---|
| `extern "C"` functions | automatic | Part XVI.1–6 |
| POD / value structs | automatic | `[FFI-5a]` verifies layout with the project's own compiler |
| standard-layout, trivially-copyable classes | automatic, by value | becomes an `@layout(c) struct` |
| other classes | automatic, opaque | behind a pointer; methods through thunks |
| `std::span<T>` | automatic | `Span[T]` / `MutSpan[T]`, zero copy |
| `std::string_view` | automatic, **as bytes** | `Span[u8]`; `[TXT-2]` makes the `str` conversion fallible |
| `std::string` | automatic | `CppString`; `.to_str()` is fallible |
| `std::vector<T>` | automatic | `CppVector[T]` with zero-copy `.span()` |
| `std::unique_ptr<T>` | automatic | `ForeignBox[T]`; the deleter must be default or named by the overlay |
| `std::shared_ptr<T>` / `weak_ptr<T>` | automatic | `CppShared[T]` / `CppWeak[T]` — **never** `Shared[T]`/`Weak[T]` (`[SEL-2]`, `E5065`) |
| `std::optional<T>` | automatic | `Option[T]` where `T` is trivial |
| `std::variant<…>` | automatic | generated enum, where every alternative maps |
| `std::function` | overlay | not importable as a parameter (`E5030`); importable as an opaque owned object, and `@ffi(std_function, signature=…)` generates the constructor |
| explicit template instantiation | automatic | named in `instantiate=[…]` (`[FFI-17b]`) |
| any other template | **native island** | Ember generics do not instantiate C++ templates (`E5055`) |
| object-like macros | automatic | constant-valued only |
| function-like macros | overlay | `@ffi(macro_fn)` (`[FFI-6b]`) |
| single inheritance from a foreign base | automatic, **bounded** | `@ffi(trampoline, virtuals=[…])` (`[FFI-39]`) |
| multiple inheritance, virtual bases | **native island** | pointer-adjustment thunks are ABI-specific (`[FFI-48]`) |
| overriding a virtual not in `virtuals=[…]` | rejected | `E5056` |
| exceptions | automatic, **declared** | `[FFI-43]`: `throws = "translate"` or `"noexcept"`, no default |
| custom allocators | overlay or native island | `[FFI-37f]` may report the fact as structurally unobservable |
| C++ metaprogramming, concepts, compiler extensions | **native island** | no importer support planned |
| ownership the header does not state | unsafe | `adopt` (`[FFI-36a]`), and the fact is `asserted` in `ember tcb` |

* `[FFI-48]` **Unsupported C++ constructs**, each with the reason it is unsupported
  and what to do instead. This list is normative; `[FFI-17]` item 14 is the
  non-normative summary of it. Encountering one of these in an imported header is
  `E5034`, naming the construct and its row.

| Construct | Why not | Instead |
|---|---|---|
| multiple inheritance | the `this`-adjustment thunk depends on the ABI's vtable layout and differs between MSVC and Itanium; getting it wrong is a silent wrong-object call | expose a single-inheritance facade in C++, import that |
| virtual base classes | the virtual-base offset table is ABI-private and not discoverable from the AST | as above |
| overriding a virtual not named in `virtuals=[…]` | the trampoline is generated from that list; an unnamed virtual has no override slot | name it in `virtuals=[…]` (`E5056`) |
| C++20 modules | libclang's module support does not expose a stable AST for a compiled module interface | parse the headers |
| exceptions propagating *through* Ember frames | Ember has no unwinder in v1 (`[PAN-1]`), so a frame crossed by an exception cannot run its drops | `[FFI-43]`'s `throws = "translate"` converts at the boundary |
| C++ coroutines | the promise type, the customisation points and the frame layout are all implementation-defined | expose a callback or a completion handle from C++ |
| overloads distinguished only by return type | not expressible; Ember has no return-type overload resolution (`[FN-*]`) | rename one in the overlay (`[FFI-13]`) |
| ABI depending on RTTI beyond the type's own vtable | `dynamic_cast` across a hierarchy the importer did not model has no Ember equivalent | do the cast in C++ and export the result |
| non-type template parameters of class type | the mangling is unstable across the supported compilers | instantiate explicitly in C++ and export a typedef |
| allocator-parameterised containers | the allocator is part of the type's identity and its behaviour is not inspectable | expose the container's span, or keep it native |
| `std::function` as a parameter | a `std::function` is constructed from a callable whose type Ember cannot name (`E5030`) | take it as an opaque owned object, or `@ffi(std_function, signature=…)` |
| compiler extensions (`__declspec`, `__attribute__` beyond layout) | there is no portable meaning to import | wrap in C++ behind a plain signature |
| anything reached only through template metaprogramming | Ember does not instantiate C++ templates (`[FFI-17b]`, `E5055`) | name the instantiation in `instantiate=[…]`, or keep it native |

## XX.13 The C++ importer corpus and migration gate

A C++ importer that has only ever met headers written to exercise it is not
evidence. This section is what turns the design of Part XVI into something
falsifiable.

* `[CXX-1]` **The corpus.** `tests/cxx-corpus/` contains, at minimum: a trivial
  class, a class with a non-trivial destructor, one with virtuals, one with a
  `mutable` member, one derived from another; each of `T*`, `T&`, `const T&`,
  `unique_ptr`, `shared_ptr`, `weak_ptr` in parameter and return position;
  `vector`, `span<T>`, `span<const T>`, `string`, `string_view`, `optional`,
  `variant`; `template<class T> T identity(T)` and `template<class T> struct Box`
  with an explicit instantiation, an unsupported generic use, a nested
  instantiation and a dependent layout; `noexcept`, `noexcept(false)` and
  `noexcept(expr)`; and the eight inheritance cases of `[FFI-39]` including a
  missing virtual declaration, a non-virtual destructor, both ownership modes and
  a re-entrant callback.
* `[CXX-2]` **Real headers, not only fixtures.** The corpus additionally binds a
  representative set of RageV's own headers and at least one third-party
  header-only library (GLM or EnTT), because a fixture the importer's author wrote
  tests the importer against its own assumptions.
* `[CXX-3]` **Every corpus entry has an expected outcome recorded**: imported as
  value, imported as opaque, imported behind an overlay, or refused with a named
  diagnostic. An entry whose outcome is "it worked" without saying which is not a
  test.
* `[CXX-4]` **Golden thunks.** The generated C++ thunk source for each entry is
  committed and diffed, so a change in what the importer emits is visible in review
  rather than discovered by a linker.
* `[CXX-5]` **The corpus runs under every supported configuration**: MSVC and
  clang-cl, debug and release CRT, RTTI on and off, `/Zc` conformance settings, and
  both `_ITERATOR_DEBUG_LEVEL` values. Where two configurations disagree the
  importer MUST fail deterministically with the diagnostic `[BLD-FFI-1b]` requires,
  never bind successfully and differ at runtime.
* `[CXX-6]` **The C++ migration gate**, which `[GATE-4]` incorporates by reference
  and which is met when all of the following hold on the corpus of `[CXX-1]`
  and `[CXX-2]`:
  1. every supported entry imports deterministically — the same input produces the
     same binding, twice, on both compilers;
  2. zero silent declarations: every entity is imported, refused with a
     diagnostic, or listed as skipped with a reason;
  3. zero accepted layout mismatches — `[FFI-5a]`'s asserts hold on every entry;
  4. zero ownership contracts inferred from evidence `[FFI-35]` does not admit;
  5. zero lifetime promotions without an explicit `from =` contract;
  6. zero exception-mode ambiguities — `[FFI-43]` leaves no import undeclared;
  7. MSVC and clang-cl produce the same semantic result on every entry;
  8. generated thunks compile under RageV's exact build configuration;
  9. FFI call overhead meets `[BEN-6]`.
* `[CXX-7]` **A dependency's exception mode is an API change.** If an upgraded
  header changes an imported function's `noexcept`-ness, its Ember signature
  changes under `[FFI-43]` — `Result[T, CppError]` appears or disappears — and
  `ember bind --report` MUST list it under API changes, not under notes. The
  `.embind` hash changes with it, so `[HR-23]` refuses a hot reload across it.

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

@export_table("RvScriptApi", protocol=3)                # a function-pointer table struct, RageV style (§Part XXII)
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
* `[FFI-31b]` **No owning Ember value may cross a module boundary.** Because each `cdylib` owns a private allocator and type-info table, class handles, `Box`, `Shared`, `Weak`, `String`, `Array`, `Map`, and any type with drop glue MUST NOT appear in an `@export` or `@export_table` signature, nor be reachable through a pointer in one. Only `@layout(c)` value types, scalars, opaque handles, `cstr` (copied at the boundary) and `extern "C" fn` may cross. Violation is `E5015`, naming the offending type and the reason. This makes Part XXII's `[RV-2]` a language rule rather than a plan convention, and follows directly from XXIII.2's "no stable Ember-to-Ember ABI".
* `[FFI-31c]` A host that must share one runtime between several Ember modules links `ember_rt` as a shared library and builds each module with `[build] runtime = "shared"`. This is the only configuration in which `[FFI-31b]` may be relaxed, and it is **v2**; v1 packages are always privately linked.
* `[FFI-33]` **Thread contracts on exported functions.** `@export` and `@export_table` accept `threads = any | main | creator` with `[FFI-11]`'s meanings, defaulting to `any`; a `@export_table` MAY set it per field; a module MAY declare a default with `#! threads main` (which is what Part XXII §1 assumes). `threads = any`: the compiler checks the exported function's reachable call graph as if it ran on an arbitrary thread — every `static` it reaches MUST be `Sync`, and no non-`Sync` class handle may be reachable from a `static`, a parameter or a captured value; violation is `E7010` naming the reached item and why it is not `Sync`, with `#! threads main` named as the fix. `threads = main`: the exported wrapper asserts, in `debug` and `release`, that the calling thread is the one that called `ember_module_init`/`ember_rt_init`, and panics `ember_panic_thread` otherwise; in exchange the body MAY touch non-`Sync` statics and handles. `threads = creator`: as `main`, but the asserted thread is the one that created the value the call is dispatched on.
* `[FFI-33a]` `[FFI-22]` is amended: `ember_rt_thread_attach()` establishes thread-local runtime state and confers **no** right to touch thread-confined data. In the `debug` profile, under the existing `debug_objects` profile key, an extended object header records the attaching thread id, and `ember_retain`/`ember_release` on a non-`Sync` object from a different thread panics naming the class and both thread ids. **`[OBJ-1]`'s release header layout is unchanged**: the 24 bytes are fully occupied and a thread id does not fit, so this is a debug-only extension, never a change to the ABI `[VER-4]` freezes.
* `[FFI-33b]` `returns_owned(destructor=f)`, `[FFI-23]`'s `Retained[T]` and `Callback[F]` MAY carry `threads=`. A `ForeignBox[T]` whose destructor is `threads = creator` records the creating thread **at the point the box is formed** — the `[FFI-36]` importer-generated wrapping, or the `adopt` call — and panics in `debug` when dropped elsewhere; this is what makes GPU, GL-context and COM handles safe to hold in ordinary Ember values. `ember bind --report` lists every foreign destructor with no thread contract. *(editorial instruction carried out 2026-09-09; see `docs/spec-amendments.md`)*
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
* `[FFI-2a]` Contracts the generator **derives** from the C declaration are verified facts and MUST NOT require `unsafe`: `const T*` ⇒ borrowed immutable, `T*` ⇒ borrowed mutable, struct/union layouts asserted per `[FFI-5]`, enum underlying types, `@packed`/`@align` reproduction, calling convention. In **return position** a pointer or reference contract is unknown-filling (`[FFI-35]`): `borrowed`, `from =` and the aliasing words join the list below. Only `unknown`-filling contracts require it: `owned`, `returns_owned`, `retained`, `span`, `len_of`, `string`, `nullable`, `handle`, `status`, `threads=`, `effects=`, `noexcept`, `callback=`, and `noalias` (`[SIMD-3]`). An overlay containing none of these needs no `unsafe`. *(editorial instruction carried out 2026-09-09; see `docs/spec-amendments.md`)* `noexcept` moves out of the unknown-filling list **when derived under `[FFI-24a]`**; an overlay-written `@ffi(noexcept)` remains unknown-filling and requires `unsafe overlay`.
* `[FFI-11c]` `TODO(count)`, `TODO(nullable)`, `TODO(ownership)` are part of the **overlay language**, not merely of the tool. `[FFI-11]`'s vocabulary and `[FFI-12]`'s signature check MUST accept them, and `[FFI-2]` MUST treat a declaration carrying one as **uncontracted** — its calls stay `unsafe`. This is what lets a team adopt a header on day one, run everything behind `unsafe:`, and buy safety incrementally rather than writing 2000 lines before the first call compiles. The marker set becomes `TODO(count)`, `TODO(nullable)`, `TODO(ownership)` and **`TODO(lifetime)`**. `[FFI-2]` MUST treat a declaration carrying any of them as uncontracted.
* `[FFI-6]` **Macro import.** The importer MUST import an object-like macro whose replacement list, **after full macro expansion**, is a constant expression of integer, floating, string-literal, **null-pointer-constant, or pointer/handle-cast** type as evaluated by Clang under the import's own configuration (a probe TU of one `static const` or `enum` per candidate macro, parsed with the same configuration and cached in the `.embind`). The macro's Ember type is the C type of the evaluated expression after the usual arithmetic conversions. This covers `(~0U)`, `(-1)`, `(1 << 3)`, `sizeof(T)`, `0xffffffffffffffffULL`, `((VkBuffer)0)` — which maps to the imported handle newtype's NULL per `[FFI-11]`'s `handle` contract — and object-like macros whose bodies expand through function-like macros.
* `[FFI-6a]` A macro that does not evaluate to such a constant is skipped with `W5001 macro not imported: <reason>`, where `<reason>` is Clang's evaluation failure — never a blanket category. The macro, its body and its reason MUST appear in the `.embind` and in `ember bind --report`.
* `[FFI-6b]` A **function-like** macro MAY be exposed as a function when an overlay declares its signature (`@ffi(macro_fn) fn VK_MAKE_API_VERSION(variant: u32, …) -> u32`). The importer emits `uint32_t em_mac_VK_MAKE_API_VERSION(…) { return VK_MAKE_API_VERSION(…); }` into the generated shim translation unit (`[FFI-29]`, RFC-043) and binds `em_mac_*` as an ordinary `extern "C"` function. **This adds no macro facility to Ember: the imported entity is a C function.** Where all arguments are compile-time constants the importer MAY fold the call through `[FFI-6]`'s probe TU, so `VK_MAKE_API_VERSION(0,1,3,0)` is usable in a `const` initialiser.
* `[FFI-5a]` In addition to the libclang cross-check, the importer MUST emit into the generated shim TU (`[FFI-29]`) or thunk TU (`[FFI-17]`) a `_Static_assert`/`static_assert` for `sizeof` and `_Alignof` of every imported aggregate that Ember constructs, passes or returns by value, or embeds, and for `offsetof` of every field Ember reads or writes. **The layout contract is therefore verified by the compiler that builds the project, on the user's machine, on every build**, and a mismatch is a build error naming the type, the field and both offsets (`E5001`). XIX §10's cross-compiler tests remain and cover the compiler's own fixtures.
* `[FFI-35a]` **What the header does not say about a parameter is retention, not lifetime.** A parameter pointee's liveness across the call is established by Ember's own borrow checker, not claimed by the header. Where an overlay declares no `retained` contract for a pointer parameter, the importer MUST record the fact "`<fn>` does not retain `<param>`" once per foreign function in `[TCB-1]`'s report at grade `asserted`. `[CLI-11]` MUST group that list by foreign module. This converts the one residual hole in parameter position from unrecorded into enumerated, which is what `[TCB-1..4]` exist for.
* `[FFI-11d]` **Return-position lifetime contract.** `[FFI-11]`'s table gains `from = self | <param> | static`, valid only on a result-position pointer, reference or span. `from = self` gives the result the receiver's region under `[LT-1]` rule 1; `from = p` gives it `p`'s region exactly as `[LT-1a]`'s `@borrows(p)` would; `from = static` gives it `[LT-3]`'s static region and MUST be used only for a pointer into `static` foreign storage. Naming more than one parameter gives the intersection, as `[LT-1a]` specifies. `span(len_of(q), from = self)` is the paired form for a pointer-plus-length result. Naming a parameter that does not exist, or naming `self` on a free function, is `E5010` under `[FFI-12]`. **No region syntax enters Ember source:** the overlay names a *parameter*, exactly as the already-shipped `@borrows(a)` does, so XXIII.2's "no named lifetimes" and `[LT-6]`'s v2 reservation are untouched and no `'a` appears anywhere.
* `[FFI-11e]` **Aliasing contract.** `exclusive` — no other live reference to the referent exists for the contracted lifetime; promotes to `ref mut T`. `aliased` — other readers may exist; promotes to `ref T` only, **never `ref mut T`, whatever the `const` spelling says**. A pointer carrying a lifetime fact and no aliasing fact is `aliased`. `[SIMD-3]`'s `noalias` is unchanged and orthogonal.
* `[FFI-11f]` A `from =` fact derived by the importer from a `[[clang::lifetimebound]]` attribute on the corresponding parameter or on `this` imports at grade **`asserted`**, not `checked`: the attribute is an unenforced source annotation and `[TCB-2]` says a declaration never raises a grade. `ember bind --report` MUST list every result-position pointer with no `from =` fact, because that list is the remaining unsafe surface of an adoption.
* `[FFI-36a]` **`adopt` is a name, not a keyword.** `std.ffi` declares `unsafe fn adopt[T](h: H) -> ForeignBox[T]`, where `H` is the foreign handle representation of `T` (`*T`, `*mut T`, or an `extern type` handle). `[LEX-15]`'s reserved set stays at 48 entries; `adopt` is resolved by the ordinary rules of Part V §1 and a user item of that name shadows it, exactly as for any other `std` item. The call is **compiler-recognised**: the destructor is not an argument and is read from the overlay contract of `T`'s foreign declaration (`[FFI-11]`'s `returns_owned(destructor=f)` or the C++ destructor `[FFI-17]` records). `adopt` carries the `FFI` effect and no other.
* `[FFI-36b]` Adopting a handle whose overlay declares `adopt = false` is `E5052`, reported **statically**. Adopting the same native object twice is not statically decidable in general: in the `debug` and `release` profiles the runtime maintains an adopted-pointer set under the existing `debug_objects` profile key, and a second adoption of a live pointer panics `ember_panic_double_adopt` naming both sites; where both `adopt` calls are on the same local in one function the compiler reports `E5052` statically. `E5052` MUST NOT be documented as catching the general case.
* `[FFI-37a]` **Observation surface.** `[FFI-37]`'s shim MUST interpose a closed, published set of symbols, and the set MUST be recorded in the evidence record (`[TCB-5]`) and printed by `ember tcb`. The v1 set is: `malloc`/`calloc`/`realloc`/`free`; `operator new`/`operator delete` in all array and sized forms; the platform heap entry points (`HeapAlloc`/`HeapFree`, `mmap`/`brk`); `pthread_mutex_*`/`SRWLock*`/ `EnterCriticalSection`; the futex/`WaitOnAddress` wait primitives; and the libc/Win32 file and socket entry points. **A foreign allocation that does not pass through this set — a bump allocation from a pool the callee reserved earlier, a custom allocator statically linked into the engine — is invisible to the shim by construction.**
* `[FFI-37b]` **Coverage is part of the evidence.** The record required by `[TCB-5]` MUST additionally contain, per graded fact: the number of times the foreign entry point was invoked under instrumentation; the number of distinct Ember call sites that reached it; and, for an effect fact, the number of invocations on which the property was actually evaluated. A fact whose invocation count is zero MUST be recorded and reported as **`unexercised`**, MUST NOT satisfy an `instrumented` claim, and MUST appear in `[TCB-3]`'s assumption list at grade `asserted`. A run that did not exercise a declared fact emits `W5054 unexercised foreign fact`, naming the overlay line, the entry point and the observed invocation count. **Silence is not confirmation.**
* `[FFI-37c]` **What the grade means.** `instrumented` MUST be defined in `[TCB-1]` and rendered by `ember tcb` as: "no counterexample was observed, on the paths exercised, through the recorded observation surface". It is **never** rendered as "the fact holds". Where `[CLI-13]`'s reachable foreign call set was not fully exercised, the report renders the fact as `instrumented (partial: N of M call sites)`. Where the declared effect set is a *negative* claim and the callee is known to use an allocator outside `[FFI-37a]`'s surface — which the overlay states with `@ffi(allocator = external)` — the grade MUST NOT exceed `asserted`, and `ember tcb` MUST print the reason.
* `[FFI-37d]` **What `instrumented` may discharge.** `instrumented` is evidence from execution, not a proof, and the toolchain MUST NOT allow it to discharge a fact whose failure is a **memory-safety** failure — ownership, nullability, lifetime, aliasing, or exception escape. Those reach `proven` only through `[TCB-1]`'s fourth grade and otherwise remain `asserted`. **Effect facts** (`Alloc`, `Block`, `Lock`, `Io`) MAY be `instrumented`, because their failure mode is a violated performance contract rather than undefined behaviour.
* `[FFI-37f]` **`instrumented` is not one thing.** A fact graded `instrumented` MUST
  additionally carry how completely the run observed it: **observed** (every
  allocation, free or call on the path passed through the interception point),
  **partially observed** (some did and the toolchain can say which did not),
  **unobserved** (the path did not execute in the run), or **structurally
  unavailable** (the mechanism cannot be intercepted at all — a custom pool, an
  arena reserved before interception was installed, a statically linked allocator,
  a VMA or driver allocation). `ember tcb` prints the label beside the grade, and
  **structurally unavailable is reported as evidence of a gap, not as evidence**:
  it means the run could not have falsified the claim, which `[PHIL-5]` forbids
  treating as support for it.
* `[FFI-37e]` **Configuration and cost.** The evidence record MUST include the C++ build configuration of the instrumented run — the `[cpp.<project>]` flag set of `[BLD-FFI-1]` and the values `[BLD-FFI-1b]` resolves — and it is an **identity** input under `[TCB-6]` as amended by FIX-020: an `instrumented` fact measured against a `debug` C++ build MUST NOT satisfy a claim in a build linking the `release` engine, and `ember tcb` names both. The shim is linked only by `[CLI-14]`'s run and MUST NOT be linked into any `ember build` output; `ember test --instrument-ffi` MUST print its own overhead so the run's slowdown is not mistaken for the program's.
* `[FFI-24a]` **Exception specification is a derived fact.** The C++ importer MUST read each imported function's exception specification and record `noexcept` at grade `checked` (`[TCB-1]`) where the declaration is non-throwing — `noexcept`, `noexcept(true)`, a `noexcept(expr)` Clang evaluates to `true`, or the implicitly non-throwing destructors and defaulted special members. Where it is potentially-throwing, or where the specification is dependent and unevaluated, the fact is `unknown` and `[FFI-24]`'s `Result[T, CppError]` shape applies.
* `[FFI-24b]` **The thunk shape follows the grade, not the assertion.** * Grade `checked` or higher: the thunk is declared `noexcept`, takes no `EmberCppError*`, and the Ember signature returns `T` rather than `Result[T, CppError]`. This is the zero-overhead path. * Grade `asserted` (an `@ffi(noexcept)` an overlay author wrote): the thunk is declared `noexcept` and MUST still contain `catch (...) { ember_panic_foreign_throw("<decl>", "<overlay file>:<line>"); }`, so a violated assertion produces a **named Ember panic** rather than an unattributed `std::terminate`. The Ember signature returns `T`. On both supported ABIs a table-based `try` region costs nothing on the non-throwing path, so this carries no runtime cost the `checked` path avoids.
* `[FFI-24c]` **Disclosure, to `[FFI-29c]`'s standard.** `ember inspect` MUST report, for every imported C++ call, its exception mode as `noexcept (checked)`, `noexcept (asserted)` or `catching (Result[T, CppError])`, and MUST state that a `catching` call is not ABI-direct. `ember bind --report` MUST list every potentially-throwing declaration on a `threads = main` or `@noalloc` path, because those are where the `Result` costs most, **and MUST flag any declaration whose exception mode changed since the previous import.**
* `[FFI-24d]` **Compatibility.** Because the fact is derived, an upstream header adding or removing `noexcept` silently changes an Ember signature between `T` and `Result[T, CppError]` — a source break in the consumer caused by a dependency edit. `[BLD-2]`'s interface hash covers it and will invalidate correctly, and the resulting diagnostic MUST name the header, the declaration and the exception-mode change as the cause rather than reporting a bare type mismatch.
* `[FFI-17e]` **The bridge types are library types.** `CppVector[T]`, `CppString` and `CppShared[T]` are declared in `std.ffi` and MUST carry explicit markers: all three are `!Copy` and `!Send`/`!Sync` in v1; `CppVector[T]` and `CppString` are `Drop` (the thunked C++ destructor) and their `.span()`/`.span_mut()`/`.as_str()` results carry the receiver's region under `[LT-1]` rule 1, so a span may not outlive the vector; `CppShared[T]` is `Drop` and its clone and drop call the C++ control block. **`CppShared[T]` is not `Shared[T]`**: `[HEAP-*]`'s `Shared[T]` has Ember's control block and `[RC-*]`'s elision rules, `CppShared[T]` has C++'s, and **the two never interconvert**. Part XV's `std.ffi` row gains all three.
* `[FFI-17f]` **Cost disclosure, to `[FFI-29c]`'s standard.** `ember inspect` MUST report every C++ bridge operation that is a non-inlinable thunk call, naming `CppShared[T]` clone/drop, `CppVector[T]` `push_back`/`len`/drop and `CppString` drop specifically, and MUST state that `[BEN-6]`'s "FFI call overhead equal to the C++ reference" gate applies to ABI-direct C calls and not to these. `[RC-6]`'s surviving-retain/release report gains a `Surviving: CppShared` section, since a `CppShared` clone inside a loop is the same optimisation barrier for the same reason.

## XVI.11 Build integration


### Extending a C++ class from Ember (`[FFI-39]`)

The case Ember must handle to be structural rather than leaf-only in an existing
engine: an engine base class with virtuals, subclassed from Ember.

```ember
import cpp "RageV/src/RageV/Core/Layer.h" with (project="ragev", overlay="overlays/layer.em")

@ffi(trampoline, virtuals=["OnAttach", "OnDetach", "OnUpdate", "OnEvent"])
extern class cpp.RageV.Layer:
    init(name: CppString)                       # names a C++ base constructor

class DebugOverlay(cpp.RageV.Layer):
    frames: u64 = 0

    init(self):
        super.init(CppString.from("DebugOverlay"))     # [FFI-39b]

    override fn OnAttach(mut self):            log("attached")
    override fn OnUpdate(mut self, dt: f32):   self.frames += 1
```

* `[FFI-39]` **A foreign base is a declared base, not an opaque type.** An
  `extern class` carrying `@ffi(trampoline, virtuals=[…])` declares a foreign type
  that satisfies `[CLS-4]`'s requirement on a base: it is implicitly `open`, it is
  sized (the importer records its layout under `[FFI-5]`/`[FFI-5a]`), and each
  name in `virtuals=[…]` is an implicitly `virtual` method an Ember subclass may
  `override`. `[FFI-8]`'s opaque-`extern type` form remains available and remains
  unsized; the two are different declarations and only this one may be inherited.
  Overriding a virtual not named in `virtuals=[…]` is `E5056`, naming the list;
  naming a method that is not virtual in the header is `E5057`.
* `[FFI-39a]` The importer emits a C++ subclass — `em_tramp_RageV_Layer` — whose
  overridden virtuals call the Ember override through its permanent thunk
  (`[HR-6]`), so overrides survive hot reload with no further machinery, and whose
  constructor stores the Ember instance handle in a member.
* `[FFI-39b]` **Base construction is explicit.** An `extern class` with
  `@ffi(trampoline)` declares one or more `init(…)` signatures naming C++ base
  constructor overloads; the derived Ember `init` MUST call `super.init(…)` exactly
  once, as `[CLS-4]` requires of every class, and the call selects the overload by
  arity and argument types. A base with no default constructor and no declared
  `init` is `E5058`. `super.init` runs the C++ base constructor before any Ember
  field is initialised, matching `[CLS-4]`'s order.
* `[FFI-17c]` **Destruction is derived-first, exactly as `[CLS-6]` specifies for
  every other class.** Dropping the last handle runs the Ember `drop`, then the
  Ember fields' drops in reverse declaration order, and only then the C++ base
  destructor. An Ember `drop` may therefore call an inherited method or pass `self`
  upcast to a foreign API, which base-first ordering would have made a
  use-after-free on a subobject whose destructor had already run.
* `[FFI-17d]` **Ownership of the trampoline object is declared, not assumed.**
  `@ffi(trampoline, owner="ember")` — the default — means the Ember handle count
  owns the pair, and the C++ subobject is destroyed when the count reaches zero.
  `@ffi(trampoline, owner="foreign")` means a foreign `delete` destroys the pair,
  and the Ember side holds a non-owning handle; this is the shape of
  `PushLayer(Layer*)` storing the pointer and deleting it in the host's destructor,
  and it is the dominant engine idiom. Under `owner="foreign"` the upcast of
  `[FFI-39c]` releases Ember's strong reference, and dropping the last Ember handle
  does **not** run the C++ destructor. A `@ffi(trampoline)` type whose base has no
  virtual destructor MUST declare `owner=`; `@ffi(no_virtual_dtor)` covers the
  remaining case, and omitting both is `E5059`. **`@ffi(no_virtual_dtor)` is
  defined here and nowhere else**: on a `@ffi(trampoline)` type it asserts that
  the programmer has established, outside the language, that no instance of it is
  ever destroyed through a base pointer, so the missing virtual destructor cannot
  be reached. It licenses no operation and grants no tier under `[TIER-1]` — it
  records an obligation the way `[UNS-7]`'s `@safety` does, and an implementation
  MUST carry its text into the `[TCB-*]` report. *(clarified 2026-09-09; see `docs/spec-amendments.md`)*
* `[FFI-39c]` **Upcasting.** `self` in an override, and an owning handle at a call
  site, upcast implicitly to the foreign base pointer for passing to foreign APIs.
  The upcast is a `[FFI-23]` retention point: under `owner="ember"` it produces a
  `Retained[Self]` token, which is what keeps the object alive while the host holds
  the raw pointer and what `[HR-20]` uses to relocate it across a reload. An upcast
  in a context that cannot hold the token — a temporary passed and immediately
  discarded — is `E5060`, naming `Retained.pin` as the fix.
* `[FFI-39d]` **Re-entrancy is expected and MUST NOT panic.** The ordinary engine
  shape is `PushLayer(Layer* l) { m_Layers.push_back(l); l->OnAttach(); }`: Ember
  calls out with a live `mut self` access and the host synchronously calls back
  into an override on the same object. The inbound trampoline path therefore
  **ends the caller's long-term access for the duration of the foreign call**
  (`[EXC-3]`'s access is closed at the call and reopened on return), so the
  override begins its own access without conflict. An implementation MUST NOT
  route this through `[EXC-1]`'s dynamic check. The borrow the caller held is not
  extended across the foreign call, so anything it observed MUST be re-read
  afterwards; this is stated in the diagnostic for `[EXC-1]` and in
  `docs/errors/`.
* `[FFI-39e]` **The inbound path has a panic boundary.** A panic inside an Ember
  override MUST NOT unwind into the C++ frame that called it: the trampoline
  catches it at the boundary exactly as `[FFI-24]` handles the outbound direction,
  reports it, and aborts under `[PAN-1]`'s package policy. Unwinding through a
  foreign frame is never permitted in either direction.

### Member mapping (`[FFI-40]`)

* `[FFI-40]` **`this`-qualifiers and references.** A `const` member function imports
  with a `self` receiver, a non-`const` one with `mut self`, an `&&`-qualified one
  is skipped (`W5033`). `T&` maps to `ref mut T`, `const T&` to `ref T`, `T&&` to an
  `owned` parameter where the type is movable and is otherwise skipped. A function
  returning `T&` returns a `ref T` whose region is elided from the receiver
  (`[LT-1]` rule 1). `[FFI-40a]` **`const` is not an aliasing guarantee in C++**, so
  a `const` member function that mutates through `mutable` state or invalidates
  iterators MUST be declared `@ffi(invalidates)` in the overlay, which gives it a
  `mut self` receiver. An importer MUST assume `@ffi(invalidates)` for any `const`
  member of a type whose header declares a `mutable` member and whose overlay is
  silent, rather than trusting `const`; the overlay may then state otherwise, and
  that statement is an `asserted` fact under `[FFI-35]` appearing in `ember tcb`.
* `[FFI-41]` **Statics, nested types and namespaces.** Static member functions
  import as associated functions (`cpp.RageV.RHIDevice.Create(…)`); static data
  members as `extern static`; nested classes and enums under the outer name
  (`cpp.RageV.RHIDevice.Desc`); namespaces as module paths under the import name.
  Anonymous-namespace entities are skipped (`W5034`), because their identity is
  per translation unit and `[FFI-30]`'s identity rule cannot hold for them.
* `[FFI-42]` **Operators and iteration.** C++ `operator==`, `<`, `+` and friends map
  onto Ember's operator interfaces where the signature allows; `operator[]` to
  `Index`/`IndexMut`; `operator*`/`operator->` on a smart-pointer-shaped type to
  auto-deref. A type with `begin()`/`end()` returning a forward iterator imports an
  `Iterable` driven by thunks, so `for x in cpp_vec:` works. `[FFI-42a]`
  **Iterator invalidation is prevented, not merely discouraged.** The imported
  `Iterable` takes `ref mut` of the container for the loop under `[CTL-2]`, so any
  method needing `mut self` — which by `[FFI-40a]` includes every `const` method
  the overlay has not cleared — is rejected inside the loop by the ordinary borrow
  rules. An earlier draft left this to a `SHOULD`, which left a use-after-free
  reachable from safe Ember; it is a `MUST`.

### Toolchain (`[BLD-FFI-4]`, `[BLD-FFI-5]`)

* `[BLD-FFI-4]` **Cross-language optimisation.** With the C backend, Ember's emitted
  C and the project's C++ are compiled by the same compiler, so the project's LTO
  setting applies to both and calls across the boundary inline like any other call.
  With the LLVM backend, `lto = "thin"` plus matching `-flto` on the C++ side
  achieves the same. `[BLD-FFI-4a]` Reloadable functions are excluded from
  cross-boundary inlining by `[HR-9a]`, and the toolchain MUST NOT enable an LTO
  configuration that would defeat it.
* `[BLD-FFI-5]` **Exporting Ember to C++ with C++ types.** `ember build
  --emit-header --cpp` writes `<package>.hpp` alongside the C header: RAII wrappers
  over the C API, `std::string_view` overloads for `str` parameters, `std::span`
  overloads for `Span`. The C header remains the ABI; the C++ header is a
  header-only convenience layer over it. `[BLD-FFI-5a]` **A smart handle is
  generated only for a `Sync` class.** `[RC-1]`'s reference count is atomic iff the
  class is `Sync`, and a generated C++ wrapper is an ordinary copyable type that
  can be put in a `std::vector`, captured by a lambda and moved to another thread —
  which for a non-`Sync` class would perform a non-atomic increment from two
  threads. For a non-`Sync` class the header emits an explicitly thread-confined
  handle type that is neither copyable nor movable across threads, documented as
  such, and `ember build --emit-header --cpp` reports which classes got which.

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
# Part XVIII — Hot Reload and Live Editing

Hot reload is a language feature in Ember, not a scripting-layer trick and not a
property of any one host. The compiler, the object model, the calling convention and
the runtime cooperate so that a source edit becomes running behaviour in a live
process with existing objects, world state and open resources preserved. This part
specifies the mechanism, the guarantees, what is refused, and the cost. Nothing in it
is specific to RageV; Part XXII shows one host wiring it up, and `[HR-30]` states the
host contract any program can satisfy.

The target is a requirement, not an aspiration: `[HR-1]` **an edit to one function
body in a 50k-line reloadable package MUST be visible in the running process within
one second** on `[BUD-1]`'s reference machine, with all live object state preserved.
`[BUD-2]`'s build budgets exist to make that reachable.

## XVIII.1 Model

The unit of reload is the **package**. A package built with reload enabled (`[HR-25]`)
becomes a *reloadable image*: a `cdylib` whose functions are reached through a
permanent thunk and whose types carry migration metadata.

```
edit .em file
  │
  ▼  ember build --reload   (only the changed module is recompiled)
new image  package.rN.dll
  │
  ▼  host calls ember_reload_poll() at a safe point (between frames)
  ┌────────────────────────────────────────────────────────────┐
  │ 1. load new image, read its reload manifest                │
  │ 2. compare schemas (types, statics, exports, enum variants)│
  │ 3. PLAN — refuse here, with a reason, or continue          │
  │ 4. PREPARE  (speculative: may fail, changes nothing)       │
  │ 5. COMMIT   (infallible: cannot fail, cannot allocate)     │
  │ 6. RECLAIM  (drop what the old image owned)                │
  │ 7. old image stays resident (never unloaded)               │
  └────────────────────────────────────────────────────────────┘
  │
  ▼  next frame runs new code
```

* `[HR-2]` **Reload is transactional, and the phase split is what makes that
  implementable.** Steps 3–4 may fail for any reason — a refused schema change, a
  a migration that returns an error, an allocation that cannot be satisfied — and
  failing there changes nothing observable: no old instance has been mutated, no
  field has been moved out of, no destructor has run, and the process continues on
  the old image with a diagnostic. **§XVIII.4a is what makes that last clause true
  rather than merely asserted**: `[HR-34]` makes PREPARE incapable of panicking and
  `[HR-36]` makes its allocation fallible, so "fails" never means "aborts". 0.7.1
  stated this guarantee while `[PAN-1]` still turned a panic in `migrate_from` into
  process death, and no phase ordering repairs that on its own. Step 5 MUST NOT be able to fail: it performs only
  pointer stores over memory reserved in step 4, and an implementation MUST NOT
  emit an allocation, a bounds check, a user callback or any other fallible
  operation inside it. Step 6 runs after the new state is live and its failures are
  ordinary runtime panics, not reload failures. A partially applied reload is a
  defect, not a permitted outcome.
* `[HR-2a]` **Phase contents are normative.** PREPARE allocates every new instance in the transaction arena of `[HR-37]`, fallibly (`[HR-36]`),
  runs every `migrate_from` (`[HR-16]`) against a read-only view of the old
  instance, and computes the full old-address → new-address relocation map. It MUST
  NOT move out of, mutate, or drop any live instance. COMMIT publishes the new
  instances, patches `type_info` pointers, applies the relocation map (`[HR-15]`),
  and repoints the thunk targets. RECLAIM drops the old instances and the fields
  removed by `[HR-14]`, running user `drop` code; a panic here is an ordinary panic
  under `[PAN-1]` and the reload has already succeeded.
* `[HR-3]` **Safe point.** A reload is applied only inside `ember_reload_poll()`,
  and only when the runtime's **Ember-depth counter is zero on every registered
  thread**. The counter is per-thread, incremented on entry to Ember code and
  decremented on exit, maintained at the same boundaries `[FFI-22]` uses for thread
  attachment and by `[THR-1]`'s thread spawn and `[JOB-1]`'s workers. It is **not**
  `[FFI-22]`'s attach flag, which records whether a thread has *ever* entered Ember
  and would be permanently non-zero in any host that calls Ember from a long-lived
  render or worker thread. A poll with any depth non-zero returns
  `EMBER_RELOAD_UNSAFE_POINT` and does nothing.
* `[HR-3a]` **Every thread that can run Ember code MUST be registered**, whether the
  host created it (`[FFI-22]`), `thread.spawn` created it (`[THR-1]`), or it is a
  job worker (`[JOB-1]`). A thread running Ember code without being registered is
  a defect in the runtime, not a permitted configuration; `debug` builds MUST
  assert on entry from an unregistered thread.
* `[HR-4]` **Old images are never unloaded.** A stale return address, a `defer`
  block captured mid-frame or a thunk target retired by a later reload therefore
  remains mapped rather than becoming a wild jump. The cost is a few hundred kB of
  resident image per reload, acceptable in a development session and absent in
  `shipping`. `[HR-4a]` A retired image's *static data* is not reused: statics live
  in the runtime-owned table of `[HR-17]`, never in image memory, so retirement
  never strands a value.

## XVIII.2 Call indirection and function addresses

* `[HR-5]` In a reloadable package, a call to a function defined in a reloadable
  package is emitted as a call to that function's **permanent thunk**. Calls to
  `std`, to `extern` functions, and to functions in non-reloadable packages are
  direct.
* `[HR-6]` **Permanent thunks are the whole mechanism.** Each reloadable function
  instance is allocated one thunk at first load, in runtime-owned executable memory,
  whose address never changes for the life of the process. The thunk loads the
  current target from the runtime's call table and tail-jumps to it; a reload
  rewrites the table entry, never the thunk. **A reloadable function's address, in
  every context that can observe one, is the address of its thunk.**
* `[HR-6a]` It follows — and this is the rule that keeps the rest of the language
  unchanged — that **`fn` values, closure code pointers, vtable slots, `dyn`
  witness tables, drop glue pointers and `extern "C" fn` values all remain ordinary
  code pointers of the shape Part IV §9 and `[OBJ-2]` already specify.** No
  representation anywhere in the language widens, and no rule elsewhere in this
  document changes. `std`, `ember_rt`, non-reloadable packages and foreign code
  therefore need no knowledge that reload exists: they load a slot and call it, and
  it is correct. A stored callback survives a reload because its address is a
  thunk, not because anything that stores it was modified.
* `[HR-7]` **Slot identity is by mangled name** (`[MNG-1]`), never by index, so
  adding, removing or reordering functions does not disturb an existing thunk. A
  function present in the old image and absent from the new one keeps its thunk,
  whose target becomes a stub that panics naming the removed function; a reload
  that would strand a *reachable* such thunk is refused by `[HR-18]` rather than
  deferred to a panic at call time.
* `[HR-8]` **The image carries an explicit reload manifest**, an image section
  listing every reloadable function instance by mangled name with its address,
  every type schema, every static, and the protocol version. The runtime reads that
  section and never the platform export directory, which on PE and ELF alike
  contains only `@export`ed symbols and would leave every ordinary method looking
  removed.
* `[HR-9]` **Cost.** One load from the call table (hot in L1) plus an indirect tail
  jump, both well predicted, and the loss of cross-package inlining for reloadable
  functions. Measured overhead on `perf/` with reload enabled MUST be reported and
  MUST NOT exceed 3%; the measurement configuration — profile, backend, host
  compiler and flags — MUST be recorded with the figure, because the number is
  meaningless without it. `[HR-9a]` The compiler MUST suppress inlining,
  devirtualisation and `[CG-C-3]`'s inline-header emission for every reloadable
  function; a reloadable function inlined into a caller cannot be swapped, and an
  implementation that inlines one has broken `[HR-1]` silently.
* `[HR-10]` `@noreload fn` opts a function out: it is called directly, may be
  inlined across package boundaries, and changing its body requires a restart (the
  reload is refused naming it). Hot loops, `@static_safe` inner functions and
  `@simd`/`@parallel` bodies SHOULD be `@noreload`. `[HR-10a]` A `@noreload`
  function MAY call a reloadable function; the call goes through the callee's
  thunk, and only the caller's own body is guaranteed direct and inlinable. An
  earlier draft of this rule forbade the call outright, which rejected the ordinary
  shape of an opted-out hot loop calling shared helpers and was unsatisfiable for
  `std` generic instances; it is not the rule. `[LNT-5]` warns (`L2005`) where a
  `@noreload` function calls a reloadable one inside a loop, because that is where
  the indirection the annotation was meant to avoid actually costs something.

## XVIII.3 Reloadable types and schemas

* `[HR-11]` Every `class`, `struct` and `enum` in a reloadable package carries a
  **schema**: the type's kind, base class, layout attributes, an ordered list of
  `{field name, field type schema hash, offset, size}`, and — for an `enum` — an
  ordered list of `{variant name, discriminant, payload schema}`. The schema hash
  is BLAKE3 over that structure, name-sensitive and offset-insensitive.
* `[HR-11a]` **Enum identity is by variant name, not by discriminant.** A reload
  that reorders variants, or inserts one, renumbers discriminants; migration MUST
  rewrite the stored discriminant of every live value of that enum to the new
  number for the same variant name. A variant renamed without
  `@renamed_from("old")` is a remove plus an add and follows `[HR-14]`'s rows.
* `[HR-12]` Class instances in a reloadable package are registered in a
  **live-instance list**: two pointers appended to the object header, which grows
  from 24 to 40 bytes **in reloadable builds only**. `[OBJ-1]`'s 24-byte guarantee
  is unchanged for `shipping` and for non-reloadable builds. This list is
  maintained by the runtime in every reloadable build, independently of `[WK-1]`'s
  debug leak reporter, which exists only in `debug` and cannot be relied on in a
  `release` build with reload enabled.
* `[HR-12a]` **The header layout is a whole-process property, and mixing is a link
  error.** Every package linked into one process MUST agree on the header size.
  The toolchain emits a symbol whose name encodes the header layout — the same
  device `[BLD-FFI-1b]` uses for the CRT and `_ITERATOR_DEBUG_LEVEL` — and a program mixing a reloadable and a
  non-reloadable package fails to link with `E9035` naming both, rather than
  writing a field at offset 24 and reading it at offset 40. `[HR-12b]` `std` is
  compiled once per header layout for the same reason, and the toolchain selects
  the matching prebuilt.
* `[HR-13]` Value-typed data (structs in `Array`s, `SoA` columns, ECS component
  storage, arena contents) is not individually registered; it is migrated through
  its **container**, which registers itself with the runtime along with its element
  type's schema. `[HR-13a]` This includes `std` containers holding a reloadable
  element type. `std` is not reloadable, but its containers are generic over types
  that are, so `Array[T]`, `Map[K, V]`, `SoA[T]`, `Pool[T]` and the ECS storages
  MUST register themselves whenever `T` originates in a reloadable package. The
  registration is emitted by the *instantiating* package, which is reloadable, so
  `std` itself needs no reload support. `[HR-13b]` Value data reachable only
  through raw pointers, foreign memory, or a `MutSpan` stored in a foreign
  structure is not reachable by migration; if such a type's schema changed the
  reload is refused (`[HR-18]`).

## XVIII.4 Migration semantics

When a type's schema hash differs between images, every live instance is migrated.
`[HR-14]` Migration is by **field name**, and the table is exhaustive over the
schema of `[HR-11]`:

| Change | Behaviour |
|---|---|
| field added with a default (`x: f32 = 1.0`) | new field initialised to that default |
| field added, type is `Default` | initialised to `Default.default()` |
| field added, no default and not `Default` | **refused** (`[HR-18]`), naming the field and suggesting a default |
| field removed | old value dropped in RECLAIM (`[HR-2a]`) |
| field renamed with `@renamed_from("old")` | value carried over |
| field renamed without the attribute | remove plus add |
| field type changed, losslessly widening (`[TYP-5]`) | converted |
| field type changed to a range type (`[RNG-1]`) | **refused** unless the value is admitted by `T.checked`; `[RNG-10]`'s construction set is closed and migration is not a member of it |
| field type changed otherwise | **refused** unless the type declares `migrate_from` (`[HR-16]`) |
| field's type is itself a migrated type | migrated depth-first; the schema hash of `[HR-11]` is recursive, so the inner change is visible in the outer hash and the inner type's own row decides the outcome |
| field reordered | value carried over (migration is by name) |
| `let` field changed | same rules; `let` does not prevent migration |
| method added, removed or changed | no instance change; witness tables rebuilt |
| class's base class changed | **refused** |
| new class added / class no longer referenced | no effect on live instances |
| class removed while instances live | **refused**, naming the class and the instance count |
| enum variant added | discriminants renumbered per `[HR-11a]`; no other effect |
| enum variant removed while a live value holds it | **refused**, naming the variant and the count |
| enum variant reordered or renamed with `@renamed_from` | discriminant rewritten per `[HR-11a]` |
| enum variant payload changed | the field rows above, applied to that payload |
| `@layout(c)`/`@packed`/`@align` changed on a type used across FFI | **refused** (a foreign structure may hold it) |
| static's initialiser changed, type unchanged | **refused** unless the static is `@reinit_on_reload`; see `[HR-17]` |

* `[HR-15]` **Relocation.** Migration preserves an instance's address wherever the
  new size fits the old allocation. Where it does not, PREPARE records the old and
  new addresses in the relocation map and COMMIT rewrites **every stored reference
  to that object that the runtime can enumerate**: strong class handles in live
  instances and registered containers, `Weak[C]` handles (which by `[OBJ-3]` point
  into the same block and are enumerated from the same live-instance list), and
  interior `ref`/`Span` values held in registered containers. `[HR-15a]` A
  reference the runtime cannot enumerate — in a foreign structure, behind a raw
  pointer, or inside `unsafe` code — cannot be rewritten; the reload is refused
  unless `[HR-20]` covers it. **`[HR-15b]` Refusal is decided in PLAN, from the
  schemas and the registered set, before PREPARE runs** — the reload never
  discovers halfway through migration that it should have refused.
* `[HR-16]` A type may define `fn migrate_from(old: ref OldSelf) -> Result[Self, ReloadError]` (`[HR-35]` fixes the signature and the panic-freedom requirement), where
  `OldSelf` is the previous schema surfaced as a generated struct whose name is
  `Old` + the type name, whose fields are the old schema's fields with their old
  types, and which is available only inside that function. Ember calls it instead
  of field-wise migration when present. It runs in PREPARE, in the new image,
  against a read-only view; it MUST NOT mutate the old instance, and a `mut`
  receiver on it is `E2224`.
* `[HR-17]` **Statics** live in a runtime-owned table keyed by mangled name and
  schema hash, never in image memory (`[HR-4a]`). A static whose type and
  initialiser are both unchanged keeps its value; one whose type changed follows
  `[HR-14]`'s rows; a new static is initialised normally, which `[STA-2]`'s
  no-runtime-initialisers rule makes exact and cheap. `[HR-17a]` **An edit to a
  static's initialiser is a visible change, not a no-op.** The schema of a static
  includes a hash of its initialiser expression, so editing `static GRAVITY: f32 =
  9.8` to `12.0` changes the hash; the reload is refused with a message naming the
  static, and `@reinit_on_reload` on the static instead re-runs the initialiser and
  discards the old value. Silently keeping 9.8 while the source reads 12.0 is the
  one outcome this rule exists to prevent.


## XVIII.4a The reload transaction

`[HR-2]` promises that a failed reload leaves the process running the old image with
a diagnostic. The four-phase split of `[HR-2a]` is what makes that *structurally*
possible — no live instance is touched until COMMIT — but it is not sufficient on its
own, and 0.7.1 overclaimed. `[PAN-1]` makes panic **abort** by default and defers
unwinding to v2, so a panic inside a user `migrate_from` would kill the process
rather than return to the old image, and `ember_alloc` panics on exhaustion. A phase
that "may fail" is worthless if failing means the process dies.

0.7.2 closes this by making PREPARE **incapable of panicking**, rather than by
catching panics — the language has no unwinder in v1 and this rule does not
introduce one. This is `[PHIL-3]`'s approach applied to reload: make the bad outcome
unconstructible instead of writing a rule against it.

* `[HR-34]` **PREPARE is panic-free by construction.** Every operation an
  implementation performs during PREPARE MUST be one whose effect set excludes
  `Panic`, or a fallible form returning `Result`. An implementation MUST NOT emit,
  in PREPARE, an operation carrying `Panic` from any source in `[EFF-*]`'s table —
  bounds check, overflow check, integer division, `unwrap`, explicit `panic`, or
  exclusivity check. `[HR-2]`'s guarantee is then discharged by construction and
  needs no unwinder.
* `[HR-35]` **`migrate_from` is fallible and panic-free.** Its signature is
  `fn migrate_from(old: ref OldSelf) -> Result[Self, ReloadError]`, and its inferred
  effect set MUST NOT contain `Panic`; a body that does is `E2225`, whose `help`
  names the total form of the offending operation. The restriction is ordinary
  effect checking, not new machinery: float arithmetic carries no `Panic` and is
  unaffected, so the worked example in XVIII.7 stands; indexing must be written
  `.get(i)`, integer division `checked_div`, and a `None` becomes a `ReloadError`
  through `?` rather than a panic. A migration that genuinely cannot express its
  failure as a value has no business running inside a transaction that promises not
  to fail.
* `[HR-36]` **Allocation inside PREPARE is fallible, and the transaction handles it,
  not the programmer.** For the duration of PREPARE the transaction arena of
  `[HR-37]` is the ambient allocator, and exhaustion in it aborts the transaction
  and surfaces as `ReloadError.Allocation` rather than panicking — so ordinary
  construction inside `migrate_from` (`e = Enemy(old.entity)`) needs no fallible
  spelling and does **not** count as a `Panic` source under `[HR-35]`, which would
  otherwise make that rule unsatisfiable for any migration that builds anything.
  An implementation MUST route PREPARE's allocation through this path and never
  through `[ALC-1]`'s `ember_alloc`, which panics on exhaustion. Exhaustion during a reload
  is a refusal, not a crash: the running program has not asked for the memory, the
  programmer has.
* `[HR-37]` **The transaction arena.** Every allocation PREPARE makes — new
  instances, the relocation map, `OldSelf` views, migration scratch — is made in a
  dedicated `ReloadTransactionArena` obtained for that reload. On a successful
  COMMIT the arena's ownership of the new instances transfers to the ordinary heap
  accounting; on a failed PREPARE the arena is released wholesale (`[ARN-2]`), which
  is a single pointer reset and cannot itself fail. No individual rollback path
  exists, and none is needed.
* `[HR-38]` **`ReloadError`.** `std.hot` declares
  `enum ReloadError: SchemaRefused(TypeName, Reason); Migration(TypeName, InstanceOrdinal, str); Allocation; Foreign(CppError); Timeout`.
  Every failure inside PLAN and PREPARE produces one; the runtime carries it into
  `[HR-18]`'s report unchanged, so the message names the type and the instance
  ordinal rather than "reload failed".
* `[HR-39]` **Foreign failure during PREPARE.** A migration that calls foreign code
  through `[FFI-24]`'s boundary receives a `Result` as it does anywhere else; a
  C++ exception is translated at the thunk (`[FFI-43]`) and becomes
  `ReloadError.Foreign`. No foreign exception and no foreign `terminate` may
  propagate through a reload, and a call to an `@ffi(throws = "noexcept")`
  import from inside `migrate_from` is **rejected at compile time** (`E2227`),
  because such a call can terminate the process and `[HR-34]` says PREPARE cannot
  fail. 0.8.2c permitted it and called the termination "a known consequence"; that
  was the wrong default — it left the one hole in a transaction whose whole value is
  that it has none, and it opened by omission rather than by choice.
  `[HR-43]` is the opt-in for a programmer who genuinely wants it.
* `[HR-40]` **What the phases may therefore contain.** PLAN reads schemas and the
  registered set: no allocation, no user code. PREPARE: fallible allocation in the
  transaction arena, `migrate_from` under `[HR-35]`, no mutation of any live
  instance, no `drop`. COMMIT: pointer stores over memory PREPARE reserved, and
  nothing else — no allocation, no user callback, no check that can fail. RECLAIM:
  user `drop` code, which may panic under `[PAN-1]` exactly as a destructor may
  anywhere else, after the reload has already succeeded. **The dividing line is
  that everything which can fail happens before anything is destroyed, and
  everything after that point cannot fail.**

* `[HR-43]` **Permitting termination is explicit.** `@allow_reload_terminate` on a
  `migrate_from` admits calls to `throws = "noexcept"` imports inside it, and states
  in one place that this migration may kill the process rather than refuse. It is
  the only route past `[HR-39]`'s rejection, it is reported by `ember tcb` beside
  the `unsafe` surface because it is the same kind of claim, and `[GATE-3]` counts
  its uses in the release notes rather than requiring zero — a project that needs it
  should say so, not hide it.

## XVIII.4b Publication and memory ordering

`[HR-3]` establishes that no thread is executing Ember code when COMMIT runs. That
is a precondition, not a memory model: the reloading thread writes thunk targets that
other threads will later read, and nothing so far says when those writes become
visible. This section answers the four questions an implementer will ask.

* `[HR-42]` **Publication.** Each thread's Ember-depth counter (`[HR-3]`) is the
  synchronising object. Entry to Ember code increments it with **acquire**; exit
  decrements with **release**. COMMIT reads every registered thread's counter with
  **acquire**, writes the new thunk targets and `type_info` pointers, and then
  publishes with a single **release** store to a reload-generation counter. A thread
  next entering Ember code acquires that generation, which orders every COMMIT write
  before any call it makes. It follows that:
  * **which threads may run during COMMIT** — any of them, provided none is inside
    Ember code; the runtime does not stop the world, and a thread doing foreign or
    OS work is unaffected;
  * **when new function addresses become visible** — at that thread's next entry to
    Ember code, never mid-call, because a thread inside Ember code is what `[HR-3]`
    excludes;
  * **a thread already executing old code** — cannot exist at COMMIT. The scenario of
    a thread entering an old function, a reload landing, and that thread returning
    through replaced code is not merely survivable but **unreachable**: its depth
    counter would be non-zero and `ember_reload_poll` would have returned
    `EMBER_RELOAD_UNSAFE_POINT`. `[HR-4]`'s resident old images cover the remaining
    case — a code address captured before the reload and called after it, which is a
    thunk under `[HR-6]` and therefore current, or a raw address that escaped, which
    `[HR-20]`/`[HR-15a]` refuse to let escape.
* `[HR-42a]` **The counter is not a lock.** An implementation MUST NOT make entry to
  Ember code take a lock, and MUST NOT let the depth counter's cost scale with the
  number of threads on the entry path: it is one relaxed increment plus one acquire
  fence, and the reload side pays the O(threads) scan because reloads are rare and
  calls are not. `[COST-3]`'s reload-indirection row covers it, and `[HR-9]`'s 3%
  ceiling is measured with it enabled.

## XVIII.5 Refusal and reporting

* `[HR-18]` When any rule says **refused**, `ember_reload_poll` returns
  `EMBER_RELOAD_REFUSED`, changes nothing, and the runtime emits a structured
  report through `cfg.log` and to `target/<profile>/reload-report.json`:

```
reload refused: 2 blocking changes

  class game.enemy.Enemy
    field `armor: ArmorKind` added with no default, and ArmorKind is not Default
    → give it a default (`armor: ArmorKind = ArmorKind.None`), derive Default,
      or add `fn migrate_from(old: ref OldEnemy) -> Enemy`
    147 live instances would be affected

  fn game.physics.step
    marked @noreload and its body changed
    → restart, or remove @noreload (costs one indirect call per invocation)

the process is still running the previous image; no state was lost
```

* `[HR-18a]` A refusal is never a crash and never partial. With `[HR-2]`'s phase
  split this is implementable rather than aspirational: every refusal is decided in
  PLAN, and every failure that can still occur after PLAN occurs in PREPARE, which
  has not touched the live set.
* `[HR-18b]` `ember build --reload --explain` reports, without running anything,
  whether the current source would reload cleanly against a named running process's
  manifest, so a programmer can check before switching to the game window.
* `[HR-19]` **Bodies-only mode.** `reload = "bodies"` restricts a package to
  changes that require no migration at all — the set `[HR-14]` marks as "no
  instance change", plus function bodies — and refuses everything else. It needs
  no schemas, no live-instance list and no 40-byte header, so `[OBJ-1]`'s 24-byte
  header and `[HR-12a]`'s link guard both stay at their `shipping` values. It is
  the tier an implementation MUST provide first (Phase 7a) and the tier a project
  can use before migration is trusted.

* `[HR-41]` **The failure matrix.** Every way a reload can fail, and what each one
  leaves behind. There is no row in which the process dies or the live set is
  partially updated; that is `[HR-2]` restated as a table so it can be audited
  without reading nine rules.

| What happened | Detected in | Old image | New image | Live state | Process |
|---|---|---|---|---|---|
| Source does not compile | before the poll | keeps running | never loaded | unchanged | continues |
| Type or borrow error | before the poll | keeps running | never loaded | unchanged | continues |
| `EMBER_RELOAD_ABI` mismatch | load | keeps running | rejected (`E9037`) | unchanged | continues |
| `.embind` ABI component changed | PLAN | keeps running | rejected (`[HR-23]`) | unchanged | continues |
| Schema change with no defined outcome | PLAN | keeps running | rejected (`[HR-14]`) | unchanged | continues |
| `@noreload` function's body changed | PLAN | keeps running | rejected (`[HR-10]`) | unchanged | continues |
| Non-relocatable reference would have to move | PLAN | keeps running | rejected (`[HR-15a]`) | unchanged | continues |
| `Retained` token with no `on_relocate` must move | PLAN | keeps running | rejected (`[HR-20]`) | unchanged | continues |
| Allocation exhausted during migration | PREPARE | keeps running | discarded | unchanged | continues (`ReloadError.Allocation`) |
| `migrate_from` returns `Err` | PREPARE | keeps running | discarded | unchanged | continues |
| Foreign call in migration throws | PREPARE | keeps running | discarded | unchanged | continues (`ReloadError.Foreign`) |
| GPU resource still in flight after the bound | PREPARE | keeps running | discarded | unchanged | continues (`[HR-24]`) |
| A panic inside `migrate_from` | **cannot occur** | — | — | — | forbidden by `[HR-34]`/`[HR-35]`, rejected at compile time (`E2225`) |
| A failure during COMMIT | **cannot occur** | — | — | — | forbidden by `[HR-2]`; an implementation emitting a fallible operation there is defective |
| Panic in a drop during RECLAIM | RECLAIM | retired | **live** | **migrated** | aborts under `[PAN-1]`, as any destructor panic does — the reload had already succeeded |
| Everything succeeded | — | retired, still mapped (`[HR-4]`) | live | migrated | continues |

## XVIII.6 Reload across the FFI boundary

* `[HR-20]` **Ember objects held by foreign code.** A `Retained[T]` token
  (`[FFI-23]`) registers the foreign holder with the runtime. `std.ffi` declares
  `Retained.pin(handle, on_relocate: Option[extern "C" fn(*void, *void)])`; the
  overlay form `callback=retained` accepts `on_relocate=<fn>` in the same contract
  vocabulary as `[FFI-11]`. On reload, an object with a live token is migrated in
  place where possible; where it must move, the runtime rewrites the token and
  invokes `on_relocate(old, new)` so the foreign side can update its own copy. A
  token with no `on_relocate` whose object must move refuses the reload in PLAN,
  naming the token's creation site.
* `[HR-21]` **Foreign function pointers into Ember** need no special rule:
  by `[HR-6]` an `@export`ed function's address is its permanent thunk, so a table
  the host captured once — RageV's `NativeApi`, or any other — stays valid across
  every reload with no re-read. `[HR-21a]` Adding, removing or changing the
  signature of an `@export` function or an `@export_table` field changes the module
  protocol and is refused; that is a rebuild of the host's contract, not a reload.
* `[HR-22]` **Foreign objects held by Ember.** `ForeignBox[T]`, `CppShared[T]` and
  opaque handles are carried across migration unchanged. Their foreign side is
  untouched by an Ember reload, which is the intent: the renderer, the physics
  world and open files survive. `[HR-22a]` If the *foreign* type's own layout
  changed, Ember cannot detect it and the `.embind` hash of `[HR-23]` is what
  refuses the reload.
* `[HR-23]` **C++ thunks.** The `.embind` hash of `[FFI-14]` is split into two
  components: an **ABI component** over the header path, header content and
  compiler configuration, and an **overlay component** over the Ember-side overlay
  file. A changed ABI component refuses the reload, naming the header and stating
  that the host itself must be rebuilt. A changed overlay component does not:
  overlay edits are Ember-side, change no C++, and are recompiled into the new
  image like any other Ember source. `[HR-23a]` Where the ABI component is
  unchanged the previous image's thunks are reused by symbol.
* `[HR-24]` **In-flight GPU work.** An object registered as borrowed by an
  in-flight frame (`[GPU-4]`) is migrated in place where the new size fits. Where
  it does not, the reload is deferred, at most `frames_in_flight + 1` polls; if it
  still cannot proceed the reload is **refused** naming the resource and its frame,
  rather than deferred again. Unbounded deferral is not a permitted outcome: with
  two or three frames always in flight it never terminates, which is the reason for
  the bound.

## XVIII.7 What the programmer writes

Nothing, in the common case.

```ember
class Enemy(Script):
    max_health: f32 = 100.0
    health: f32 = 100.0

    @renamed_from("speed")
    move_speed: f32 = 3.0

    aggro_radius: f32 = 8.0        # newly added: existing enemies get 8.0

    override fn on_update(mut self, dt: f32):
        ...                         # edit freely; live enemies keep their health


@noreload                           # direct calls, inlinable, no indirection
@static_safe @noalloc
fn integrate(mut p: SoA[Particle], dt: f32):
    ...


extend Enemy:                       # for a change field-wise migration cannot express
    fn migrate_from(old: ref OldEnemy) -> Result[Enemy, ReloadError]:
        e = Enemy(old.entity)                              # allocates in the transaction arena
        e.health = old.hp_percent * old.max_health / 100.0 # f32 math: no Panic effect
        return Ok(e)
```

* `[HR-25]` **Scope is opt-in and explicit.** `reload` is a manifest key with the
  values `"all"`, `"opt-in"`, `"bodies"` and `"none"` (`[MAN-7]`); `@reloadable`
  and `@noreload` apply at module and item level, and item level wins. Reload is
  available in `debug` and `release` and forbidden in `shipping`. Because
  `[HR-12a]` makes the header layout a whole-process property, a program's packages
  MUST agree on whether reload is enabled at all; `"opt-in"` and `"none"` differ in
  which functions get thunks, not in header layout.
* `[HR-26]` `@field`-marked script fields (Part XXII §1) migrate by this same
  mechanism, so an editor inspector and hot reload share one metadata path.

## XVIII.8 Cost and profile behaviour

| | `debug` | `release` | `shipping` |
|---|---|---|---|
| call indirection | yes | opt-in (`reload`) | **never** — all calls direct |
| object header | 40 bytes | 40 with reload, else 24 | **24** |
| live-instance list | yes | with reload | no |
| schema metadata in image | yes | with reload | stripped |
| `@noreload` functions | direct, inlinable | direct, inlinable | identical to every other function |

* `[HR-27]` A `shipping` build MUST be bit-identical whether or not the source
  contains `@noreload`, `@renamed_from`, `@reinit_on_reload` or `migrate_from`;
  reload constructs have no shipping representation and `migrate_from` bodies are
  dead-stripped. This is `[PRF-1]` applied to this part: reload changes performance
  and representation *within* a reloadable build, and changes nothing observable in
  a build without it.
* `[HR-28]` `ember inspect --safety` reports reload indirection per function
  alongside the safety checks, so the cost is visible where every other implicit
  cost is.

## XVIII.9 Host integration

`[HR-29]` The runtime lives in **one** place per process. When reload is enabled,
`ember_rt` MUST be linked dynamically and shared by every image: the call table, the
thunks, the live-instance list, the statics table and the heap all belong to it, and
a reloadable image that statically linked its own copy would load with an empty live
set and a separate heap. The toolchain MUST reject a reloadable package configured
for static runtime linkage with `E9036`.

`[HR-30]` **The host contract** is four calls, and any program that can call them at
a point where no Ember frame is live can hot-reload. There is no requirement that
the host be an engine, be C++, or run frames.

```c
/* once */
ember_reload_config rc = { .watch_dir = "src", .on_report = my_log_fn };
ember_reload_init(&rc);

/* at any point where this thread holds no Ember frame */
switch (ember_reload_poll()) {
  case EMBER_RELOAD_NONE:         break;  /* nothing pending          */
  case EMBER_RELOAD_APPLIED:      break;  /* new code is live         */
  case EMBER_RELOAD_REFUSED:      break;  /* report already emitted   */
  case EMBER_RELOAD_UNSAFE_POINT: break;  /* a thread is inside Ember */
}
```

* `[HR-31]` `ember_reload_poll` is non-blocking: compilation runs in a background
  process started by the file watcher, and the poll applies an image only once it
  is complete. A poll with nothing pending costs one atomic load.
* `[HR-32]` `ember_reload_stats` reports the last reload split into compile / load /
  plan / prepare / commit / reclaim, the live instances migrated, and the refusals,
  so `[HR-1]`'s one-second budget is measurable from inside the host.
* `[HR-33]` `ember run --hot` is the toolchain's own host: it builds under `debug`,
  runs the binary, watches the sources and calls `ember_reload_poll` from a
  supervisor thread at a point the program declares with `hot.checkpoint()` from
  `std.hot`. A program that declares no checkpoint and does not call
  `ember_reload_poll` itself never reloads, and `ember run --hot` MUST say so after
  ten seconds rather than appearing to hang.

---

# Part XIX — Compiler Architecture

## XIX.1 Overview

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

`[CMP-3]` **Ember's semantics are defined by this document, not by the C backend.** The C11 backend is an implementation target chosen for bootstrap and portability (Part 0 row 7). Where C cannot express an Ember guarantee directly — aliasing, function identity across a reload, weak references, exclusivity, panic behaviour, coroutine frames, alignment, the `[SIMD-*]` contracts, `[COST-3]`'s elision guarantees — the backend MUST implement the guarantee by other means and MUST NOT narrow it to what C makes convenient. A rule may not be justified by "the C backend cannot do otherwise": `[CAT-3]` already forbids a `LANGUAGE-NORMATIVE` rule resting on a `REFERENCE-IMPLEMENTATION` one, and this states it for the backend specifically. Where a guarantee is genuinely unimplementable on the C backend the correct outcome is a recorded gap and an LLVM-only capability, never a quieter guarantee.

`[CMP-1]` The compiler is a Rust workspace (`emberc`). Each stage is a crate with a documented input/output type and a `--emit=<stage>` flag so intermediate representations can be dumped and snapshot-tested. `[CMP-2]` No stage after parsing may report a diagnostic without a source span.

### Crate layout

```
compiler/
  ember_span         FileId, Span, SourceMap (line/column mapping, UTF-8 aware)
  ember_diag         Diagnostic model, rendering (ariadne-style), error-code registry (Part XX §6), JSON output
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
tests/               conformance suite (Part XX §5)
```

## XIX.2 AST

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

## XIX.3 HIR

HIR is the AST after name resolution, type checking and desugaring. Differences from AST:

* All names are `DefId`s or `LocalId`s; paths are resolved.
* Every expression carries its `Ty`.
* Desugared: `for` → `while` + iterator calls (`[CTL-1]`); operators → interface method calls (scalars keep intrinsic ops); `?` → `match`; `?.` → `match`; f-strings → `Formatter` calls; `with` → block + explicit drops; augmented assignment → method call or binary op; `elif` → nested `if`; ternary → `if`; named/default arguments → positional with default expressions inserted; auto-ref/deref adjustments made explicit (`Adjust::Borrow`, `Adjust::Deref`, `Adjust::Coerce(widen)`); method calls resolved to `Callee::Static(DefId, generic_args)` or `Callee::Virtual(slot)` or `Callee::Dyn(iface, slot)` or `Callee::Closure`.
* Patterns are compiled to a **decision tree** (Maranget's algorithm) with exhaustiveness/redundancy results attached.
* Closures are lifted to synthetic struct types with a capture list `{local, mode: ByRef|ByMutRef|ByValue}`.

`[HIR-1]` HIR is the input to the comptime interpreter's MIR lowering and to the formatter's semantic lints. `[HIR-2]` `--emit=hir` prints a stable textual form used in snapshot tests.

## XIX.4 MIR

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

`ember_types` maintains, per interned `Ty`: `size`, `align`, `layout` (fields with offsets; enum tag placement and niche), `is_copy`, `needs_drop`, `is_view` (+ inferred region-slot shape for view types), `is_send`, `is_sync`, `is_zeroable`, `has_niche(Ty)`, `drop_glue: Option<DefId>`, `vtable(iface)`, `ffi_safe: bool`. Layout of `@layout(c)` structs is computed by the same algorithm the C ABI uses (natural alignment, trailing padding), and is cross-checked against libclang for imported types (`[FFI-5]`).

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

1. **Regions.** Every reference/view-typed local, temporary and projection gets a region variable `'r`. A function's signature regions are per `[LT-1]`: `'self`, one `'p_i` per view-typed parameter, `'ret` = the elided/`@borrows` choice, plus `'static`. Struct view fields use their inferred region slots; the legacy single-region case is represented as a one-slot vector.

`[MIR-REG-1]` Every callable that can receive or return a multi-region view carries compiler-internal
field-access and field-to-region provenance summaries in its interface artifact. The summaries are
erased from source syntax and runtime ABI but participate in interface hashing and invalidate dependent
borrow/type checking when they change. The summaries MUST be sufficient for a caller to determine which
region slots are required by each call and where every returned view field derives from without
re-reading the callee body. An unavailable or stale summary is a hard compiler failure, not permission
to assume a narrower access set.
2. **Constraints** are generated by walking MIR: assignment `a = b` of reference types yields `'b: 'a` (b outlives a, i.e. `points('a) ⊆ points('b)`); calls instantiate the callee's signature regions with fresh variables and add its constraints; `Ref{place}` creates a **loan** `L = (place, mut?, region)`; reborrows add constraints from the base reference's region.
3. **Liveness.** Compute the set of CFG points at which each local is live (used later on some path). A region `'r` includes every point at which any local with a type mentioning `'r` is live, and `[LT-*]` closure under constraints (fixpoint).
4. **Loan scope.** Loan `L` is **in scope** at point `P` iff `P ∈ points(region(L))` and `L` has not been **killed** (its base local was overwritten or went out of scope before `P` on that path).
5. **Access check.** At each statement, for each place `P'` accessed with kind `K ∈ {Read, Write, Move, ShallowWrite, Drop, BorrowShared, BorrowMut}`, for each loan `L` in scope with place `P` such that `P` and `P'` **overlap** (one is a prefix of the other, with `Index` projections assumed overlapping unless constant-disjoint, `Column` and `Field` projections disjoint when names differ, `Deref` of a class handle assumed overlapping with any other deref of the same class type — handled by exclusivity instead), report an error if `K` conflicts with `L.mut` per `[BRW-1]`. Two-phase borrows: a `Ref{mut}` marked `two_phase` is a shared loan until its **activation** point (the call).
6. **Region errors.** A loan whose region extends beyond the borrowed place's storage (`StorageDead`/`Drop` of a local, **or of a parameter passed by value**, while a loan on it is in scope) is `E3060 borrowed value does not live long enough` — an owned parameter is a copy that lives in the callee's frame, so a borrow of it is exactly as short-lived as a borrow of a local, and only a **view-typed** parameter names storage the caller keeps; a return of a reference whose region is not a subset of `'ret`'s allowed region is `E3062 returned reference does not derive from a parameter`.
7. **Diagnostics** name (a) the borrow site, (b) the conflicting access, (c) the later use that keeps the borrow alive ("borrow later used here"), and (d) a fix suggestion chosen from: shorten with a block, clone, use `split_at_mut`/`columns_mut`, use an index loop, use `Weak`.

The implementation is expected to be ~4–6k lines; the Rust compiler's `rustc_borrowck` is the reference for edge cases (drop-check, closures, two-phase, `match` fake reads). Polonius-style location-sensitive reasoning is not required for v1.

### 4.8 Exclusivity analysis

For class-object accesses, a lightweight pass over MIR: for each `BeginAccess(place=h.deref.f…)`, if all other `BeginAccess` on the *same handle local* (same `LocalId`, not reassigned in between) are statically ordered so that their intervals do not overlap conflictingly, mark it `elided`; overlapping accesses through the *same* local are `E3080` (compile-time, like a borrow error). Accesses through distinct locals keep the runtime check.

### 4.9 Drop elaboration

Replace `Drop{place}` terminators with: nothing (if `!needs_drop`), a call to the type's drop glue (a synthesised function that calls user `drop` then drops fields), a `Release` (handles/`Shared`), or a conditional on the drop flag. Partial moves produce per-field drops. This pass makes MIR ready for optimisation and codegen.

### 4.10 Effects

Per Part X: compute a bottom-up fixpoint over the call graph SCCs of the monomorphised program; store the effect set on each function instance; check contracts; produce chains for diagnostics. Before monomorphisation, generic functions are checked once with their bounds' declared effects.

### 4.10a Coroutine lowering

`[MIR-6]` A `gen fn` (`[CORO-1]`) is lowered before monomorphisation. Liveness is computed across suspension points; every local live across at least one `yield` becomes a field of the frame type, laid out by the ordinary rules of `[STR-*]`; every other local stays a stack slot of `resume`. The body becomes a `switch` on the frame's `state` field, one arm per suspension point plus entry and completion. Drop glue is synthesised per suspension point and selected on `state`, satisfying `[CORO-7]`. The frame type is an ordinary nominal type from this point on, so monomorphisation, effect analysis and the borrow checker see nothing coroutine-specific, and `[CORO-6]`'s restriction is what makes that true.

### 4.11 Monomorphisation

Collect instantiation roots (`main`, `@export`s, `@test`s, statics); walk MIR bodies substituting generic arguments; instantiate on demand with a `(DefId, substs)` cache; synthesise drop glue, vtables (`dyn` and class), and closure bodies. `[MONO-1]` Instantiations are named deterministically (§8) so that separate compilation units dedupe at link time. §4.11a specifies the instantiation budget, the report, and the conditions under which an instantiation set is emitted as one shared function rather than one function per type.

### 4.11a Instantiation budget and shared instantiation

`[TYP-16]` makes every distinct instantiation a distinct symbol and says code size is
the programmer's responsibility. That is only a fair thing to say if the programmer
can see the cost and has something to do about it. This section supplies both. Neither
changes what a program computes: `[PRF-1]` governs, and nothing below may alter
observable behaviour.

* `[MONO-2]` **Counting.** The monomorphiser records per generic item the number of distinct instantiations produced and the time spent generating and optimising them. The count is taken after `[MONO-1]`'s dedup, so a generic instantiated identically from forty modules counts once.
* `[MONO-3]` **The budget.** `[build] max_instantiations = N` (`[MAN-6]`) sets a per-generic ceiling. It is unset by default, and unset means no ceiling and no behaviour from this section. Exceeding it is `W2220`, naming the generic, its count, the ceiling, and the three most recently added instantiations with source locations — the last of which is what makes the warning actionable on the commit that caused it rather than a year later. Under `edition_lints = "strict"` it is an error, like any W-level diagnostic. A non-integer or negative value is `E9034`.
* `[MONO-4]` **The report.** `ember build --report=instantiations` prints every generic with its count and time, descending by time, and writes the same data to `target/<profile>/instantiations.json`. It is available in every profile and costs nothing when not requested.
* `[MONO-5]` **Shareability.** A generic item is **shareable** at a type parameter `T` when every occurrence of `T` in its MIR body is the receiver of a call to a method of one of `T`'s bounds, and `T` occurs nowhere else — not in arithmetic, not as a field type, not as an operand of `size_of` or `align_of`, not by value in a signature the body depends on, and in no position `[TYP-22]` makes `dyn`-incompatible. Shareability is a property of the generic, computed once from its body, not per instantiation.
* `[MONO-6]` **Shared emission.** For a shareable generic the compiler MAY emit **one** function taking `{data*, vtable*}` in place of `T`, together with one witness table per instantiating type, and rewrite that generic's call sites to pass the table — that is, it may emit what `[TYP-22]`'s `dyn` already denotes, from source that did not write `dyn`. The decision is per generic, and is taken only when the generic is shareable **and** its `[MONO-2]` count exceeds `[MONO-3]`'s ceiling **and** no call site to it lies in a loop the compiler judges hot. With no ceiling set, the compiler MUST NOT share anything: a build that never asked for this gets exactly the code 0.6.2 produced. This is consistent with Part 0's governing principle — *the programmer's declared type determines the storage and lifetime model; the compiler may only perform optimisations that preserve that model's observable semantics* — because sharing changes neither storage nor lifetime nor any observable behaviour, only the number of function bodies emitted. Part 0 row 2 rejected letting the compiler choose a value's **storage strategy**; `[MONO-6]` chooses a **calling form** for code the programmer already wrote as a generic, and `[MONO-8]` binds it to produce identical results.
* `[MONO-7]` **The programmer has the last word.** `@always_specialize` on a generic forbids shared emission for it; `@never_specialize` requires it wherever it is legal. `@never_specialize` on a generic that is not shareable is `E2223`, naming the occurrence of `T` that prevents it — which is also the diagnostic that teaches what shareability is. Neither attribute changes the meaning of any program.
* `[MONO-8]` **Semantics are preserved exactly.** A shared instantiation MUST compute what the specialised instantiations would have computed. It costs one indirect call per bound-method call and forfeits inlining at those sites; it costs nothing else, and in particular it introduces no allocation and no `dyn`-typed value the programmer can observe. `ember inspect` reports per generic whether it was specialised or shared and which clause of `[MONO-6]` decided it. Effect analysis runs on the post-decision program, so `[EFF-9]`'s `RuntimeCheck(k)` continues to describe generated code.
* `[MONO-9]` A shared instantiation is a distinct symbol under `[MNG-*]` and participates in `[MONO-1]`'s link-time dedup like any other. An `@export`ed or `extern` item is never shared: its ABI is its signature.

### 4.12 MIR optimisations (v1 set)

`RC pair elision` (`[RC-2]`), `SROA` (scalar replacement of `Copy` struct temporaries), `const propagation`, `copy propagation`, `dead-store/dead-code`, `bounds-check elimination` (range analysis for `for i in 0..len(a)` patterns — guaranteed by `[CTL-3a]`-style tests), `inline` of functions ≤ 8 MIR statements or marked `@inline`, `drop-flag elimination`, `tail-temporary merging`. All are optional for correctness; the C compiler/LLVM does the heavy lifting.

## XIX.5 Comptime interpreter (`ember_interp`)

Executes MIR directly over an interpreter heap with typed allocations (each allocation knows its `Ty` and layout). Supports every MIR construct except `Call` into `extern` functions and raw-pointer deref outside interpreter allocations (`E6010`). Provides intrinsics: `size_of`, `align_of`, `offset_of`, `reflect`, `read_file`, `env`, `target()`. Results are converted back to `Const` values (including aggregate constants and byte strings) for embedding as statics. Step and memory limits per `[CT-3]`.

## XIX.6 C backend (`ember_codegen_c`)

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
* `[CG-C-8]` **Step fidelity.** A `#line` directive MUST precede every emitted statement and MUST name the span of the *source construct that produced it*, never the declaration span of a place it mentions. All C statements lowered from one Ember statement MUST carry the same `#line`, so "step over" advances exactly one Ember statement. Desugared constructs (`for`, `?`, `?.`, `with`, f-strings, operator calls — XIX §3) MUST attribute to the source syntax, not to the desugaring. **This requires MIR `Stmt` and `Term` to carry a source span**, and the MIR verifier checks that every statement has one.
* `[CG-C-7]` **Names.** Every MIR local carrying a user name MUST be emitted with that name as its C identifier, transliterated per `[MNG-3]`, suffixed `_<n>` only on collision with a C keyword, a reserved identifier, or another local in the same C scope. Parameters keep their Ember names; compiler temporaries keep `_<index>`. A profile MAY set `debug_names = false`; no default profile does. Deterministic naming keeps `[CG-C-2]`'s diff-based review intact.
* `[CG-C-9]` **Debugger visualisers.** `ember build` MUST emit, beside the binary, `target/<profile>/<package>.natvis` (passed with `/NATVIS:` on MSVC) and `<package>-gdb.py` / `<package>-lldb.py`, generated from the same `TypeInfo` table (§4.3) the backend already walks. They MUST render at minimum: `Option[T]` as `None`/`Some(v)` including every niche form; `Result[T,E]`; each payload enum as `Variant(fields)`; `String`/`str` as text including the SSO form; `Array`/`Span`/ `MutSpan` as `len` elements; `Box`, `Shared`, `Weak` as their pointee plus counts; a class handle as `Class { fields }` with the header hidden; `Handle[Tag]` as `index:generation`; `SoA[T]` as reconstructed `T` values.
* `[CG-C-10]` **Stacks.** `[RT-4]`'s panic output and `ember_backtrace_print` MUST print Ember function paths and Ember `file:line:col`, demangled from `[MNG-1]`'s scheme, never raw C symbols. The toolchain ships `ember demangle` (a stdin/stdout filter) so MSVC and GDB stacks can be read.

## XIX.7 LLVM backend (v2)

Maps MIR to LLVM IR through `inkwell`: the same type mapping; `noalias`/`readonly`/`dereferenceable(N)`/`nonnull` attributes from the borrow checker's facts; `!nontemporal`, `!alias.scope` for `@simd` loops; DWARF/CodeView debug info with Ember type names; PGO via LLVM instrumentation; ThinLTO. It becomes the default when it passes the full conformance suite plus the performance suite (Part XXI §4).

## XIX.8 Name mangling

```
em_<pkg>_<module path with '_'>_<item>[__g<hash of generic args>][__v<vtable>]      e.g. em_std_math_Vec3_length, em_game_ecs_integrate__g3f2a1c
```

`[MNG-1]` Hash = first 12 hex digits of BLAKE3 of the canonical type string of the generic arguments. `[MNG-2]` `@export("name")` overrides the symbol entirely. `[MNG-3]` Identifiers are transliterated to ASCII (`_uXXXX_` for non-ASCII). `[MNG-4]` Class object structs are `em_obj_<mangled class>`; vtables `em_vt_<mangled>`; type infos `em_ti_<mangled>`.
* `[MNG-5]` The mangled prefix `em_` of `[MNG-1]`/`[MNG-4]` is derived from `EMBER_SYMBOL_PREFIX` and constructed in exactly one function.

## XIX.9 Runtime ABI (`ember_rt`, C11)

Header `ember_rt.h`, ABI version macro `EMBER_RUNTIME_ABI = 1`. `EMBER_RT_ABI` is a compatibility alias only and MUST NOT denote a second protocol. Everything below is `[RT-*]` normative.

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
* `[RT-5]` Every runtime symbol, macro and header name is **generated** from a single build constant `EMBER_SYMBOL_PREFIX` (default `ember`), defined in exactly one place in `ember_rt` and one in the compiler. `ember_rt.h` is a **generation output** carrying literal identifiers, not a header of macro concatenations — it is the interface document C embedders read, and it must stay readable. No file in the compiler, runtime, CMake module, examples or test corpus may hard-code the symbol prefix, the CLI name, the manifest file name, or the source and binding-cache extensions; each is read from a single `branding` module. `tools/check_branding.py` fails CI on any hard-coded occurrence.

## XIX.10 Compiler correctness strategy

* Snapshot tests per stage (`--emit=tokens|ast|hir|mir|mir-opt|c`) with `insta`.
* The MIR verifier runs after every pass in debug builds of the compiler.
* Differential testing: every `run-pass` test is executed through both backends (once LLVM exists) and through the comptime interpreter where applicable; outputs must match.
* Fuzzing: `cargo-fuzz` targets for the lexer, parser, type checker (well-typed program generator), and borrow checker (random mutation of accepted programs must either still pass or produce an error with a span).
* FFI layout tests: a generated C program asserts `sizeof`/`offsetof` for every type crossing the boundary and is compiled with each supported compiler in CI.
* Performance regression suite (Part XXI §4) gates releases.

---
# Part XX — Toolchain

## XX.1 CLI

```
ember new <name> [--lib | --bin | --cdylib]        scaffold a package
ember build [--profile debug|release|shipping] [--target <triple>] [--backend c|llvm] [--emit tokens|ast|hir|mir|c|obj]
            [--emit-header] [--emit-optimization-report] [--explain-performance] [-Dwarnings]
            [--report=instantiations] [--build-id] [--timings[=json]]
ember build --reload [--explain]   build a reloadable image; --explain reports whether it would
                                   reload cleanly against a running process, without applying it
ember run [args...] [--hot]        build + run the binary; --hot watches and swaps (Part XVIII)
ember test [filter] [--doc]      run @test functions (and doc tests)
ember bench [filter]
ember check                type-check + borrow-check without codegen
ember fmt [--check]
ember lint
ember inspect <path.to.item>       effects, layout, allocation sites, inlining, ABI
ember explain <EXXXX>              print the error's reference page
ember explain <rule-id>            print the rule, its category ([CAT-4]) and its conformance directory
ember explain --borrow <file>:<line>   why a borrow at that line is still live (region as line ranges)
ember inspect --safety <path> [--elided-only] [--json]   runtime safety checks emitted and elided
ember inspect --alloc <path>       allocation taxonomy and conversion costs ([TXT-5])
ember inspect --cost <path>        every [COST-3] row applying to this item, elided or emitted
ember inspect --deterministic <path>   whether it satisfies [DET-1], and the chain to each Nondet source
ember bind <header> [--emit-embind | --explain <symbol> | --report | --emit-cpp]
ember shader-bind <reflect.json>
ember doc
ember clean
ember toolchain {list|install|default}
```

`[CLI-1]` Every command supports `--json` for machine-readable output (diagnostics, inspect, report) and exits non-zero on error. `[CLI-2]` `ember build --emit=c --out-dir <dir>` writes the C sources without invoking a C compiler (used by the CMake integration).

* `[CLI-4]` `ember run <file.em>` and `ember build <file.em>` MUST accept a single source file with no `ember.toml`, synthesising a package named after the file (`kind = "bin"`, entry = the file, default profile, no dependencies beyond `std`). **A first program MUST NOT require a manifest.**

## XX.2 Manifest (`ember.toml`)

```toml
[package]
name = "ragev_scripts"
version = "0.1.0"
language = "0.9"
kind = "cdylib"                     # bin | lib | staticlib | cdylib
edition_lints = "strict"           # "strict" turns W-level into errors listed in [lints]

[dependencies]
std = { version = "0.9" }           # implicit; may pin
ragev_api = { path = "../ragev_api" }
some_lib = { git = "https://…", rev = "…" }
math_ext = "1.2"                    # registry (v2)

[build]
entry = "src/lib.em"
backend = "c"                       # c | llvm
c_compiler = "auto"                 # auto: msvc on Windows if cl.exe found, else clang
target = "native"
max_instantiations = 200            # per-generic ceiling; unset means no ceiling ([MONO-3])
reload = "opt-in"                   # "all" | "opt-in" | "bodies" | "none"  ([HR-25], [MAN-7])

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
* `[MAN-3]` Every key in `[lints]` MUST name a lint the compiler defines (`E9010` otherwise). XX §2's `unused` key is `L1001`.
* `[MAN-4]` The manifest file name, the source extension and the `.embind` extension are constants of the same `branding` module; the test harness and the build system discover them rather than spelling them.
* `[MAN-5]` `ember.toml` gains `[ffi] evidence = "<path>"`, default `".ember/ffi-evidence"`. It is part of `[BLD-2]`'s package config.

* `[VER-1]` Three version numbers exist and are independent: the **language version** (`language` in `ember.toml`, `#! language`), the **compiler version**, and the **runtime ABI version** (`EMBER_RUNTIME_ABI`). Every release states all three.
* `[VER-4]` **The runtime ABI is stable within a major version.** `[OBJ-1]` is amended accordingly: the object header layout, `ember_type_info`, and every entry point in `ember_rt.h` MUST NOT change within a major version. Any change bumps `EMBER_RT_ABI` and is a major release. `ember_rt_init` MUST compare `ember_rt_abi_version()` against the value its caller was compiled with and fail initialisation with a diagnostic naming both versions rather than proceeding. Fields MAY be appended to `ember_rt_config` only at the end, guarded by a leading size field. Debug-only extensions behind a profile key (`[FFI-33a]`) are not part of the frozen layout.
* `[VER-5]` Packages use semantic versioning. A `[dependencies]` requirement is caret by default (`"1.2"` means `>= 1.2.0, < 2.0.0`); resolution selects the highest version satisfying all requirements; two different majors of one package MAY coexist in a graph, **their symbols distinguished by a major-version component in `[MNG-1]`'s package segment**. `ember.lock` records the result and `ember update [pkg]` recomputes it.

**The 1.0 compatibility promise** (owner decision `OQ-22`; normative from 1.0). `[VER-2]` source compatibility within a major language version, with breaking changes only in a new major that a package opts into by editing `language`, and a compiler accepting every language version of its own major series. `[VER-3]` deprecation in `1.n` via `@deprecated` and a `W`-code naming the replacement and the removing version, with removal no earlier than the next major. `[VER-7]` 1.0 means `[VER-2]` and `[VER-4]` come into force, the conformance suite passes on every supported host, `docs/errors/EXXXX.md` exists for every code, and the user guide exists.


* `[MAN-6]` `[build] max_instantiations` is a positive integer or absent; absent means no ceiling and disables everything in XIX §4.11a (`E9034` otherwise). `[MAN-7]` `[build] reload` is one of `"all"`, `"opt-in"`, `"bodies"` or `"none"` (default `"opt-in"` in `debug`, `"none"` otherwise); any other value is `E9031`. It is forbidden in `shipping` (`E9033`). Every package linked into one process MUST agree on whether reload is enabled at all, which `[HR-12a]`'s link-time symbol enforces.
* `[CLI-15]` `--report=instantiations` (`[MONO-4]`), `--build-id` (`[BLD-13]`), `--reload` and `--reload --explain` (`[HR-18b]`), `--hot` (`[HR-33]`), `--timings` (`[BUD-6]`) and `ember inspect --deterministic` (`[DET-9]`) are the surfaces 0.6.3 adds. Each MUST appear in `ember --help` and in XXIII §3's construct list, per `[DIA-6a]`'s completeness check.

## XX.3 Build graph and caching

* `[BLD-1]` Unit of compilation and caching: the **module** (one `.em` file) for front-end stages; the **package** for monomorphisation and codegen (one C file per module is emitted, but instantiations are placed in the module that first requests them, with COMDAT-style `static inline`/weak linkage to dedupe).
* `[BLD-2]` Cache key of a module's front-end artefact: BLAKE3 of (source, compiler version, language version, package config, transitive **interface hashes** of imported modules — the hash of exported signatures, types, layouts, effect sets, inline bodies, and compiler-internal multi-region view provenance summaries, not of private bodies). A change to a private function body recompiles only its module's codegen and any callers' *effect checks* if its effect set changed (`[EFF-4]`). *(editorial instruction carried out 2026-09-09; see `docs/spec-amendments.md`)*
* `[BLD-3]` The `.embind` cache key remains header hash + flags + overlay-list hash, but the **entity identities** it records are `[FFI-30]`'s C identities, so two cache entries for the same header and flags describe the same entities under different views. **"Flags" is not the compiler's command line.** The governing rule is: **if changing a setting can change a generated binding's semantics or its ABI, it is part of the key**, and a binding MUST be invalidated when such a setting changes. A flag that cannot do either — an optimisation level, a diagnostic switch — is not part of it. The settings that qualify are, at minimum: the compiler and its version; the language standard; the target triple and ABI; the C++ standard library implementation and its version; on MSVC the runtime-library switch and the effective `_DEBUG` and `_ITERATOR_DEBUG_LEVEL`, which `[BLD-FFI-1b]` already requires to be inherited byte-for-byte because they change the layout of `std::string`/`std::vector` and the identity of the CRT heap; RTTI and exception settings, and the `/Zc` conformance switches; every `-D`/`/D` reaching the header; the calling convention and name-mangling scheme; and the thunk generator's own version, since a change in how a thunk owns, copies or catches changes the contract without changing the header. The list is a floor, not a definition — the rule above decides. A setting the toolchain cannot determine is `E9021` and MUST NOT be defaulted (`[BLD-FFI-1b]`), for the same reason — a guess here binds successfully and differs at run time. *(clarified 2026-09-09; see `docs/spec-amendments.md`)*
* `[BLD-4]` The C compiler and linker are invoked through a Ninja file generated per build (`target/<profile>/build.ninja`) so that incremental C compilation is handled by Ninja; MSVC is driven with `/showIncludes`, Clang/GCC with `-MD`.
* `[BLD-5]` Output layout: `target/<profile>/{bin,lib,c,obj,bind,inspect}`.
* `[BLD-6]` **Link-time optimisation.** `profiles.<p>.lto` takes `"off" | "on" | "thin"`, mapped by the C backend to MSVC/clang-cl `/GL` + `/LTCG`, Clang `-flto=thin` / `-flto=full`, GCC `-flto`. Where a value is unsupported the toolchain MUST substitute the nearest supported value and record the substitution in the build record. **LTO MUST NOT be required to satisfy `[CG-C-3]` and MUST NOT be required for any correctness property** (XXIII.2).
* `[BLD-7]` Front-end stages MUST run in parallel across modules by default (`-j`, default = physical cores), and MUST NOT take a global lock on the symbol interner or the type interner on the hot path.
* `[BLD-8]` **Item-granular re-checking.** Within a module, the front end MUST record, per item, the signatures it read, so that editing one function body re-runs type checking, borrow checking and effect analysis **for that item alone**. Module-level name resolution re-runs only if the edit changed an item's signature or the set of names the module declares. `[BLD-2]`'s interface hash is unchanged; this is a finer key beneath it.
* `[BLD-9]` `ember build --timings` and `ember check --timings` write `target/<profile>/timings.json` plus an HTML summary, per stage and per module.
* `[BLD-10]` **Latency budget.** `tests/perf/compile/` contains generated packages of 10k, 50k and 200k lines with a realistic mix (≈30 % generic code, 20 % classes, 10 % FFI declarations). Budgets are measured on the reference machine recorded in the suite, at the `debug` profile, for: `ember check` cold; `ember check` after one function body is edited; `ember build` after one function body is edited; `ember build` cold. **The budgets are release gates with the same status as XXI §4's runtime thresholds.** The concrete figures are calibrated once the generator exists and recorded in the suite, not in this document; **the existence of the gate is normative, the numbers are not.**

* `[BLD-13]` **Reproducible builds.** Given the same source, manifest, dependency lock, toolchain version, target and profile, `ember build` MUST produce a byte-identical artifact. The compiler MUST NOT embed a build timestamp, an absolute path, a host name, a user name, or the iteration order of any hash container into its output; diagnostic paths are recorded relative to the package root, and `[MONO-1]` already makes symbol names deterministic. `ember build --build-id` prints a hash over exactly those inputs, so two lockstep peers can compare one string before a match rather than discover a mismatch three minutes in — which is what `[DET-7]` requires of them.

## XX.4 Profiles

The profile controls: optimisation level, overflow policy, bounds checks (never disableable in safe code — `bounds_checks = false` only affects `unsafe`-opted `get_unchecked` hints and is honoured by no v1 profile), exclusivity checks, `debug_objects`, sanitizers, LTO, `panic` policy, `strip`. `[PRF-1]` A profile MUST NOT change program semantics except: (1) overflow policy (`[TYP-8]`); (2) panic-vs-UB for dynamic **class** exclusivity under `exclusivity = "unchecked"` (`[EXC-1]`, ADR-004); and (3) presence of debug facilities. In particular no profile may change which branch of a `match`, `if`, `?` or short-circuit operator is taken, nor the value of any expression observable to safe code, nor whether a program is accepted (`[EFF-15]`).


* `[PRF-2]` **Reload is a profile-constrained build mode, not a profile.** `[MAN-7]`'s `reload` key is admitted in `debug` and `release` and forbidden in `shipping` (`E9033`). A build with reload enabled emits permanent thunks for reloadable functions, suppresses `[CG-C-3]`'s inline-header emission for them (`[HR-9a]`, since an inlined body cannot be swapped), and uses the 40-byte header of `[HR-12]`. `[PRF-1]` applies unchanged and `[HR-27]` states it for this part: a build without reload is bit-identical to one from a source containing no reload construct at all.

## XX.5 Test harness and conformance suite layout

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
* `[TST-4a]` **A directory is not coverage.** Every rule's directory MUST contain at least one **accept** case — source the rule admits, which compiles and behaves as the rule says — **and**, for a rule that can reject source, at least one **reject** case: source the rule refuses, with the expected diagnostic recorded in a `.stderr` file that the harness compares exactly. A rule tested only by its happy path is not tested: it passes against a compiler that never enforces it, which is the single most likely way a conforming-looking implementation is not one.
* `[TST-4b]` **Which rules need a reject case is decided mechanically, not by judgement.** `[DIA-6a]` already maps every diagnostic code to the rule that defines it, so a rule naming a code MUST have a reject case for each code it names, and a rule naming none — a layout guarantee, a representation statement, a rule about what the compiler MAY elide — is waived automatically. The waiver list is generated by `tools/rule_index.py` and committed, so a rule that later acquires a diagnostic loses its waiver in the same pass that adds the code, and nobody maintains a list by hand.
* `[TST-4c]` **Transition.** `[TST-4a]` ships with a **recorded baseline** of rules that currently have only an accept case, in the manner of `[TST-7]`, and fails CI only on a rule added or amended after 0.8.1 and on any rule whose entry improves and then regresses. The baseline shrinks and never grows: `tools/rule_index.py` rejects a commit that adds a rule to it. `[GATE-1]` requires the baseline to be **empty** for 1.0, which is what makes this a transition rather than a permanent exemption.
* `[TST-6]` `docs/spec-source/appendix-a.em` is a conformance fixture, and Appendix A's code block is generated from it by `tools/spec_check.py --emit-appendix`, so the quick reference cannot drift away from something that parses. The fixture's directive is the current language version; `...` bodies are `pass`; the statement tail is wrapped in `fn demo():`; `match` arms use the `=>` form, since `[GRM-16]` makes a jump an expression. It is annotated `#$ test: compile-pass`.
* `[TST-7]` `tools/spec_check.py` extracts every fenced ` ```ember ` block from Parts I–XVII and Appendix A and runs `ember check --syntax-only` over each. A block that does not parse fails CI. A block MAY opt out with ` ```ember,ignore ` and MUST then carry a one-line reason on the preceding line (permitted reasons: a `std` signature sketch, foreign-language source, or a deliberate error example). A fourth permitted reason is **overlay-language source, until Part III defines `overlay_decl`**; XVI.4's `overlay c "vulkan/vulkan.h":` block and XVI.7a's `overlay cpp "RageV/VulkanBackend.hpp":` block carry it, so the baseline is explicit rather than implied. The gate ships with a **recorded baseline** of `,ignore` blocks and fails only on *new* failures until RFC-010 and RFC-022 have closed the grammar gaps it exists to expose.
* `[CLI-9]` `ember check --syntax-only <file>` lexes and parses the file and reports only `E00xx` and `E01xx` diagnostics. It does not resolve names, so an example naming undeclared types still passes.
* `[CLI-10]` `ember build --report=engine` is a **reporting mode, not a compilation profile**. It MUST NOT change type checking, safety, program acceptance, generated semantics, or optimisation legality — `[PRF-1]` governs profiles and this is not one. The report summarises allocations, retain/release traffic, dynamic class accesses, surviving runtime checks with their `[EFF-11]` reason codes, arena usage, vectorisable loops and the clause that blocked each one, FFI wrapper costs, inlining decisions, and candidate class-to-struct migrations. A migration candidate is advisory only and is never applied automatically.
* `[CLI-5]` `ember bind --init <header> [--out overlays/<stem>.em]` writes a **starter overlay** containing, for every declaration the header exports: its derived contract; every `unknown` fact written as a `TODO(count)` / `TODO(nullable)` / `TODO(ownership)` marker with the original C declaration in a trailing comment; every skipped declaration as a commented `W5002` line with its reason; and a `hide` section. The emitted file MUST compile as an overlay unchanged. `--init` never overwrites an existing file; with `--merge` it adds only declarations absent from it, preserving hand-written contracts and comments.
* `[CLI-6]` `ember bind --report [--json] [--baseline <file>]` prints every declaration that did not import, with its construct and reason (`[FFI-20a]`), and every declaration still `unsafe`, with the facts still `unknown`. It exits non-zero when anything is skipped, so a project can gate CI on a header continuing to import; with `--baseline` it exits non-zero only on *new* skips, and `--write-baseline` records the current state.
* `[CLI-7]` `ember bind --check <overlay>` verifies `[FFI-12]` without building, and prints the counts of declarations that are safe, still `unsafe`, and skipped — so the state of an adoption is a number a team can watch.
* `[TST-8]` **First-draft corpus.** `tests/firstweek/` holds at least 24 programs of 30–200 lines, each the *first draft* of a task a newcomer plausibly attempts in week one — a text adventure with rooms holding items, a particle fountain, a CSV summariser, a scene graph with parent pointers, an observer/event bus, an inventory with equipment slots, a tile-map path finder, a config loader, a ring buffer of chat messages, a small hand-rolled ECS. Each is contributed by someone who has read only Part I and Appendix A, MUST be committed unmodified, and records its author's experience level.
* `[TST-9]` Each program carries `#$ firstdraft: accepted` or `#$ firstdraft: rejected(<shape>)` naming a §XX.6.1 or §XX.6.2 shape. **A rejection whose shape is in no catalogue is a release blocker** — it is `[DIA-7]`'s unclassified case in the only corpus not written by people who already know the answer. The companion obligation — that the mandated `help`, applied literally, produces a program that compiles — is `[PHIL-8a]` (RFC-011) and applies to the whole catalogue, not only to this corpus; the corrected program is committed beside the draft as `<name>.fixed.em`.
* `[TST-11]` **v0.5 regression obligations.** The conformance suite MUST contain, in addition to the per-rule directories `[TST-4]` requires: for `[EXC-4]`, instantaneous scalar reads, `Copy` field reads, container iteration through `let` fields, `ref`/`ref mut` projections, mutating methods through `let` containers, and aliasing mutation during iteration; for `[LT-7]`, callback-local borrows, escape rejection, internal-only region inference, and nested callback-region separation; for `[TYP-15]`/`[TYP-15a]`, rejection of arbitrary owning view containers, acceptance of `BorrowList[T]`/`ViewList[T]` only when one inferred container region outlives every region slot of each element, with rejection when that bound cannot be proven, and escape rejection through class fields, `static`s, `Box` and `Shared`; for `[FFI-30c]`, overlay composition order, explicit `override`, incompatible-definition diagnostics, foreign type identity stability, and build invalidation on overlay change; for `[EFF-17]`, rejection of every `Panic(Explicit)` source and acceptance of division by `NonZero[T]`; and for `[PRF-1]`, that one source receives the same contract verdict under `debug`, `release` and `shipping`, and that `--report=engine` does not alter that verdict. The v0.6 obligations additionally include: `[TCB-6]` evidence invalidation after a foreign identity or contract hash changes; a `std.ser.yaml` round-trip with an out-of-range field producing `SerError` rather than an invalid value; `[RNG-10b]`'s rejection of a range type in an `extern` block and behind a pointer parameter; and `Option[Full]` occupying two bytes.
* `[TST-10]` The harness computes the **first-draft acceptance rate** (accepted ÷ total) into `target/firstweek.json` and the release notes publish it. It is a **tracked number, not a gate** — the gate is `[TST-9]`'s blocker — and a release that lowers it MUST say why.
* `[TST-5]` `tests/debug/` runs, on every supported host in CI, a scripted debugger session (`cdb` on Windows, `lldb`/`gdb` elsewhere) against a `debug` build: set a breakpoint by Ember `file:line` and verify it binds; step three Ember statements and verify the reported line advances by one construct each; print four locals by their Ember names and match the rendered text against a snapshot. A host on which `[TST-5]` cannot run is reported as **debug-unverified** and requires a recorded waiver; it does not thereby cease to be a supported host, since debugger automation is a property of the CI image.

## XX.6 Diagnostics

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
* `[DIA-3]` Borrow/ownership diagnostics MUST include the "later used here" label (`[BRW-2]` explanation) and a concrete fix drawn from the catalogue in §XX.6.1 (`[DIA-7]`).
* `[DIA-4]` Contract diagnostics MUST print the full call chain (`[EFF-5]`).
* `[DIA-5]` FFI diagnostics name the header and the C declaration (`vkCreateBuffer in vulkan_core.h:1234`).
* `[DIA-7]` **Every ownership or borrow error MUST be classified into one of the shapes in §XX.6.1 and MUST emit that shape's required `help` line.** An error the classifier cannot place emits the generic explanation and is recorded in `target/<profile>/unclassified-borrow-errors.log`; CI fails if the conformance suite produces any unclassified borrow error. This rule exists because an unexplained rejection is the single largest usability cost of static aliasing rules, and it is cheaper to specify the fixes than to discover them one bug report at a time.
* `[DIA-8]` `ember explain --borrow <file>:<line>` prints, for each loan live at that line: where it was created, the line range of its region, and the specific later use that extends it — in the form `borrow of `v` created at 12:9, live through 19, because `s` is used at 19:14`. This is generated from the borrow checker's own loan/region tables (Part XIX §4.7), not reconstructed.
* `[DIA-9]` Diagnostics MUST NOT suggest `unsafe`, `Cell`, `RefCell`, `Shared`, or `clone()` as the *first* suggestion when a structural fix exists for the shape (per the catalogue's ordering). `clone()` is suggested first only for shape O1 (use after move of a `Clone` type).
* `[DIA-6a]` The error-code registry is **exhaustive and normative, in both directions**: every code named anywhere in this document MUST have a registry entry (code, kind, subsystem, title, the rule it enforces) and a `docs/errors/EXXXX.md` page, **and** every registry entry MUST cite a rule id that exists in this document. `tools/rule_index.py` MUST fail CI on either violation, in the same pass that checks `[TST-4]`. **A code MUST be defined by exactly one rule.** `tools/rule_index.py` MUST fail CI when two rules name the same code with different titles, in the same pass that enforces rule-ID uniqueness (XXIII.4). It MUST additionally fail CI when a rule defined in the active index has no row in the current revision's change log, and when an attribute, construct or CLI surface named in normative text is absent from Part III §7, `[ATT-2]`, XX.1 or XXIII.3. **This check, not more review, is what prevents a 0.7 that repeats this.**
* `[DIA-7a]` **Every code is keyed.** The table below maps every error code in `E3000–E3499` to the shape whose `help` it MUST emit. A code absent from this table MUST NOT be emitted. `tools/rule_index.py` fails the build if `ember_diag::codes` contains an E3xxx code with no row here.  | Code | Shape | Code | Shape | |---|---|---|---| | `E3010`–`E3013` | O2 | `E3050` | O6 | | `E3014`, `E3015` | O8 | `E3060` | B7 | | `E3016` | O9 | `E3061`, `E3090`, `E3096` | A1 | | `E3020` | B2 | `E3062` | B6 | | `E3021` | B3 | `E3063` | B12 | | `E3022` | B1 | `E3065` | B14 | | `E3023` | B4 | `E3070` | O7 | | `E3024` | B5 | `E3080` | X1 | | `E3025` | B8 | `E3095` | B11 | | `E3026` | B9 | `E4030` | S1 | | `E3027` | B10 | `E3030` | O5 | | `E3040` | O1 | `E3041` | O3 | | `E3042` | O4 | | |
* `[DIA-12]` Every diagnostic in `E1000–E1499` and `E2000–E2499` MUST be classified into one of the shapes below and MUST emit that shape's required `help`. An unclassifiable error is written to `target/<profile>/unclassified-basic-errors.log`; CI fails if the conformance suite produces any. This is `[DIA-7]` applied to the errors a newcomer meets first.
* `[DIA-13]` Every shape in §6.1 and §6.2 has a rendered snapshot under `tests/ui/`, plus the `.fixed.em` companion `[PHIL-8a]` requires; `[TST-4]`'s index generator fails CI for a shape with none.
* `[DIA-14]` No diagnostic may be emitted about an expression of type `Ty::Error` or a path bound to `Def::Error` (`[IDE-3]`): the first error of a cascade is reported and the rest suppressed. `[AST-2]`'s three-per-region cascade cap applies to the whole front end, not the parser alone.
* `[DIA-15]` Suggestion candidates are computed only from data the compiler already holds (scope tables, `TypeInfo`, and a by-name index over `[BLD-2]`'s module export hashes). A suggestion requiring speculative type checking is not required.
* `[DIA-16]` **World changes disclose their cost.** When a diagnostic's `help` proposes moving a value type into the object world (`class`, `Shared[T]`) or wrapping it for dynamic checking (`RefCell[T]`), it MUST attach a `note:` naming the cost in the terms the specification already defines: ``` = note: a class instance is a heap allocation with a 24-byte header (OBJ-1) and reference counting (RC-1); long-term accesses through it are checked at runtime (EXC-1), so a function containing them cannot be @static_safe (EFF-13) ``` The note is required because the object world's costs are permanent and structural while the value-world fixes offered alongside are local; a diagnostic presenting them as equivalent misinforms.
* `[DIA-17]` **Python-form fix-its.** `x is None` / `x is not None` on a non-handle operand suggests `x.is_none()` / `x.is_some()`; `a < b < c` suggests `a < b and b < c` and notes that `b` is then evaluated twice; `xs[-1]` on a `usize`-indexed container suggests `xs.last()`; `let x = e` in statement position suggests `x = e` and notes that `let` marks an immutable **field** (`[CLS-9]`); `with e as x:` suggests `with x = e:`.

## [DIA-19] E3065 diagnostic artifact completeness

`E3065` is the canonical 0.9.5 diagnostic for shape B14, multi-region result provenance.
The diagnostic registry MUST contain exactly one entry for `E3065`, with title
`multi-region result provenance`, subsystem `borrowck`, shape `B14`, and enforcing rules
`[LT-22]`, `[LT-35]`–`[LT-40]`, and `[VERIFY-3]`.

A conforming implementation MUST provide:
- `docs/errors/E3065.md`;
- `tests/ui/borrow/B14/multi_region_result_provenance.em`;
- the corresponding expected diagnostic snapshot; and
- a registry entry consumed by the diagnostic-index check.

This specification does **not** claim that those repository artifacts currently exist. Their
absence is an implementation/conformance gap, not permission to omit the diagnostic contract.

### XX.6.1 Required diagnostic catalogue for ownership and borrow errors

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
| **B13** reserved legacy diagnostic | `E3064` was the pre-0.9.5 rejection for multi-region view structs and MUST NOT be emitted by a 0.9.5 conforming compiler | no source fix is required; use `ember explain --borrow` if a related region error remains |
| **B14** multi-region result provenance | a returned `@view struct` contains a borrowed field whose source-region relationship cannot be inferred soundly | return an owned value, return the views as separate parameters/values, or make the source provenance explicit through an existing `@borrows` contract where it is semantically applicable; never force an intersection or `'static` |

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

Codes added by 0.6 and 0.6.1, each inside the range its subsystem owns: `E1021` (name not linked in this build, `[STD-6]`); `E2210`–`E2215` (range types); `E4041`/`E4042` (`@noio`, `@nolock`); `E4050`–`E4057` and `E4060`–`E4064` (contracts and verification); `E5050`–`E5054` (foreign grades, lifetimes and range types at the boundary); `E9012`/`E9013` (manifest sections); `E5055` (Ember generic passed to a C++ template). Warnings `W3012` (undocumented `unsafe`), `W4001` (`result`/`old` shadowing), `W5050` (unbacked or stale grade), `W5054` (unexercised foreign fact). Lints `L2003` (fallible construction where a total one exists), `L3015` (undocumented unsafe obligation), `L3016` (`@safety` text still reads `TODO`).

Codes added by 0.6.3, each inside the range its subsystem owns: `E2220` (`yield` outside a `gen fn`), `E2221` (borrow held across a `yield`), `E2222` (coroutine where an ordinary function is required — `extern`, `@export`, C function pointer, or a `gen fn` not returning `Coroutine[R]`), `E2223` (`@never_specialize` on a generic that is not shareable); `E4070` (`@deterministic` violated), `E4071` (reserved — `@deterministic` reaching an `extern` without `@ffi(deterministic)`, reported as `E4070` with the foreign frame named, and registered so the code is not reused), `E4072` (`@fastmath` or `@fp(contract)` inside `@deterministic`), `E4073` (reserved — a coroutine frame that allocates in `@noalloc`, which `[CORO-5]` makes unreachable in v1 and which is registered so the code is not reused); `E9030` (hot reload refused; **0.7.1 re-points this at `[HR-18]`'s refusal report, which supersedes the 0.6.3 meaning**), `E9031` (bad `reload` value, `[MAN-7]`), `E9032` (reloadable function's address escaping to foreign code — retained as a code, though `[HR-6]`'s permanent thunks make it unreachable in 0.7.1, so it is registered and never emitted), `E9033` (`reload` in `shipping`, `[PRF-2]`), `E9034` (bad `max_instantiations`). Warnings `W2220` (instantiation ceiling exceeded), `W9030` (reload requires a restart, with the reason). Lint `L2004` (`gen fn` with no `yield`).

Codes added by 0.7.1, each inside the range its subsystem owns: `E2224` (a `mut` receiver on `migrate_from`, `[HR-16]`); `E5056` (overriding a virtual not in `virtuals=[…]`), `E5057` (naming a non-virtual in `virtuals=[…]`), `E5058` (foreign base with no default constructor and no declared `init`), `E5059` (`@ffi(trampoline)` on a base with no virtual destructor and no `owner=`), `E5060` (upcast of an owning handle in a context that cannot hold the `Retained` token); `E9035` (packages linked into one process disagreeing about the object-header layout, `[HR-12a]`), `E9036` (a reloadable package configured for static runtime linkage, `[HR-29]`). `E9031` and `E9033` are re-pointed by `[MAN-7]` and `[PRF-2]` to the `reload` key and the `shipping` prohibition. Warnings `W5033` (`&&`-qualified member skipped), `W5034` (anonymous-namespace entity skipped). Lint `L2005` (`@noreload` function calling a reloadable one inside a loop, `[HR-10a]`).

Codes added by 0.8.3: `E2227` (a `throws = "noexcept"` import called from `migrate_from` without `@allow_reload_terminate`, `[HR-39]`/`[HR-43]`). Codes added by 0.8.2c: `E2226` (`in` or `not in` on a type that does not implement `Contains`, `[STD-8]`). Codes added by 0.7.2, each inside the range its subsystem owns: `E2225` (`migrate_from` whose effect set contains `Panic`, `[HR-35]`); `E5061` (imported C++ function with no exception policy), `E5062` (contradictory exception policies), `E5063` (foreign bytes reaching `str` without validation, `[TXT-2]`), `E5064` (interior null in a value converted to `CStr`, `[TXT-3]`); `E9037` (`EMBER_RELOAD_ABI` mismatch on image load, `[ABI-2]`). Lints `L3017` (reference cycle detected, `[WK-4]`), `L3018` (`unsafe` block with no reason category, `[UNS-9]`).

Codes added by 0.8: `E5065` (converting between `Shared[T]`/`Weak[C]` and `CppShared[T]`/`CppWeak[T]`, `[SEL-2]`). 0.8 adds no other code: it is a documentation and release-criteria revision, and a revision that needed a new diagnostic to state what the language already meant would not be one.

`[DIA-6]` `docs/errors/EXXXX.md` exists for every code with an example and its fix; `ember explain E3040` prints it.
* `[DIA-18]` A diagnostic that rejects a foreign call because its contract contains `unknown` facts MUST name each missing fact and print the exact overlay line that supplies it: ``` error[E5002]: `vkGetPhysicalDeviceQueueFamilyProperties` requires `unsafe`: its contract is incomplete note: `pQueueFamilyProperties` has no count contract and no nullability help: add to `overlays/vulkan.em`: unsafe fn vkGetPhysicalDeviceQueueFamilyProperties( physicalDevice: borrowed, pQueueFamilyPropertyCount: inout_count, pQueueFamilyProperties: span(len_of(pQueueFamilyPropertyCount), nullable)) ```

### XX.6.2 Required diagnostic catalogue for the errors of the first hour

The shapes in §XX.6.1 are the errors of week two. These are the errors of the first hour, and they decide whether a newcomer stays. Each MUST be recognised and MUST produce the listed `help`.

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

## XX.6a Trusted base and audit reporting

The trusted-base report lists what the compiler could **not** establish for
itself, and how strong each remaining claim is.

```text
$ ember tcb

Trusted for memory safety
  borrow checker, RC runtime, arena allocator      language

Unsafe
  unsafe blocks                                    12
  raw pointer operations                            7

Foreign
  C functions                                      23   19 checked, 4 asserted
  C++ bridge calls                                 14    2 proven, 9 instrumented, 3 asserted
  still requiring `unsafe`                          3

Assumptions relied on (7)
  VulkanBackend  the destructor does not throw                       asserted
  VulkanBackend  submit() does not allocate                          instrumented
  vkGetDevice    returns a valid device                              asserted
```

* `[TCB-1]` Every foreign fact carries one of four **grades**: **asserted** (a
  human wrote it), **checked** (tooling derived or confirmed it from the header,
  the ABI or the build), **instrumented** (a test run observed it holding), or
  **proven** (discharged by analysis of the adapter — *not* by a theorem prover,
  which v1 does not have; the analysis is of the thunk the importer itself emitted,
  so its scope is exactly what that code does and nothing about the foreign body it
  calls). A fact with no grade is asserted. The four grades are the complete set and
  the grammar production at Part III agrees with this list; a revision changing
  either MUST change both, and `tools/rule_index.py` compares them. **checked** — the fact is enforced or verified by something **outside the overlay author**: the ABI; the compiler's own layout cross-check (`[FFI-5]`); or the C++ runtime, as for `noexcept`, which `std::terminate` enforces. An **unenforced source annotation** — `const`, `restrict`, `[[nodiscard]]`, `[[clang::lifetimebound]]` — is `asserted`, however useful it is as a hint. This is what makes `[TCB-2]` true rather than aspirational.
* `[TCB-2]` The report MUST separate what the language guarantees from what an
  external component supplies. **A declaration never raises a grade**: no grade
  above `asserted` may be produced by writing it down.
* `[TCB-3]` Every assumption a proof relied on and did not discharge appears in
  the report, with its source and its grade.
* `[TCB-4]` Entries are categorised: language, compiler, runtime, standard
  library, unsafe, C FFI, C++ bridge, external library, GPU driver, hardware,
  solver, assumption. **The compiler is in the list.** Its own defects are part
  of what any claim rests on, and the report says so rather than implying the
  toolchain stands outside the question. The **Unsafe** section MUST list, per `unsafe fn` declaration and per `unsafe` block, its `[UNS-7]` obligations with grade `asserted` rather than a count alone. `ember tcb --json` MUST expose them, and `[CLI-12]`'s `ember why --unsafe` MUST print the obligation of every unsafe function on the chain it reports. The **compiler** entry `[TCB-4]` mandates ("The compiler is in the list") MUST carry a defined schema: compiler version and build hash, the backend in use (`[CG-C-*]` or LLVM), which analyses are self-hosted, and a link to the known-defect list for that version. A conforming implementation MUST NOT satisfy `[TCB-4]` by printing the word "compiler".
* `[TCB-5]` An `instrumented` fact is valid only for the exact foreign identity and
  contract it was measured against. Its evidence record MUST include: foreign library
  identity/version, header or interface content hash, effective ABI/toolchain hash,
  overlay/contract hash, instrumentation shim hash, test binary hash, test
  configuration and observation time. The record is stored under
  `target/<profile>/ffi-evidence/` and is referenced by content hash. The evidence record's default location is `.ember/ffi-evidence/` **beside the manifest, not under `target/`**, so a team may commit it and obtain reproducible grades across machines. `[MAN-5]`'s `[ffi] evidence = "<path>"` relocates it. It remains referenced by content hash.
* `[TCB-6]` The inputs `[TCB-5]` records are of two kinds. **Identity inputs** — foreign library identity/version, header or interface content hash, overlay/contract hash, instrumentation shim hash, and the effective ABI/toolchain hash — invalidate a record: a change to any of them makes it `stale`, and a stale record MUST NOT satisfy an `instrumented` claim, a verified build requirement, or `[FFI-37]`. **Environment inputs** — test binary hash, test configuration and observation time — are recorded and reported but do **not** alone make a record stale; `ember tcb` prints them so a reader can judge. A different STL or allocator genuinely changes whether a foreign function allocates, which is why the ABI/toolchain hash is an identity input and not an environment one.
* `[CLI-11]` `ember tcb [--json] [--module <m>]` prints the report. `ember audit`
  prints a one-page summary: the core safety analyses, the effect totals, the
  foreign boundary, and the verification counts.
* `[CLI-12]` `ember why --unsafe|--alloc|--block|--io|--lock|--ffi <path.to.fn>`
  prints the shortest call chain from that function to the thing named, using
  the effect sets `[EFF-1]` already computes.
* `[CLI-13]` `ember calls --foreign <path.to.fn>` lists every foreign function
  reachable from that function, with the chain to each and its grades. Run
  against a frame entry point it is the measure of how much of a migration
  remains, and it falls as subsystems are replaced.
* `[CLI-14]` `ember test --instrument-ffi` performs `[FFI-37]`'s run.
* `[BLD-11]` A package declares which `[STD-6]` layers it links. Omitting `alloc`
  removes allocating types from the build. Referring to one is a name-resolution
  diagnostic in the name-resolution namespace (`E1020`), not a build-system error
  code and not a lint. The build system remains responsible for determining the
  available symbol set; the resolver owns the user-facing missing-symbol diagnostic. — replace "(`E1020`)" with "(`E1021 name is not linked in this build`, whose `help` names the `[STD-6]` layer that supplies it and the manifest key that links it)". `E1021` is unused; `E1020` remains `[GRM-4]`'s and MUST NOT be reused. Add `docs/errors/E1021.md` under `[DIA-6]`.
* `[BLD-12]` A package that omits a layer MUST NOT depend on a package requiring
  it; the build reports the dependency path that reintroduces it.
* `[TST-12]` The conformance suite MUST contain a program per grade asserting
  the report gives it that grade, and MUST assert that a declaration alone never
  produces `checked` or higher.
* `[TST-13]` The suite MUST cover: a foreign reference imported unsafe and then
  promoted by an overlay; adoption, and double adoption; an instrumented run
  contradicting a declared effect; range construction proved and unproved;
  a contract proved, disproved and timed out; and `[CLI-13]`'s reachability
  against a known call graph. add: an instrumented run in which a declared fact is never exercised, asserting the resulting grade is `asserted` and `W5054` is emitted; and an `allocator = external` overlay whose negative effect claim is capped at `asserted`.

## XX.7 Formatter

`ember fmt` is deterministic and configuration-free except line width (default 100). Rules: 4-space indentation; one blank line between methods, two between items; trailing commas in multi-line argument/element lists; spaces around binary operators, none around `**` when both operands are atoms; `x: T = v` spacing; imports sorted (`std` first, then dependencies, then local) and merged; attributes one per line; the formatter preserves comments and blank-line groups (max 2 consecutive); `pass` inserted for empty blocks; long conditions broken after `and`/`or`. `[FMT-1]` `fmt(fmt(x)) == fmt(x)` and `parse(fmt(x)) ≡ parse(x)` are tested over the whole test corpus.

* `[FMT-2]` The formatter never produces a block-bodied lambda inside brackets: it emits the `=>` form when the body is a single expression, and otherwise leaves the programmer's named binding alone.
* `[FMT-3]` The formatter never emits `;` outside `[T; N]` and `[v; N]`. Since `[GRM-18]` makes `;` illegal as a statement separator there is nothing to split: one line already carries one statement.

## XX.8 Linter (`ember lint`, L-codes)

v1 lints: `L2001 unnecessary clone` (source not used again), `L2002 large Copy` (> threshold bytes passed by value), `L3001 potential cycle` (statically visible strong ownership cycles, including generic instantiations), `L3002 borrow held longer than necessary` (a borrow whose last use is far before its scope end and blocks a later access — suggests a block), `L3010 unsafe block larger than necessary`, `L3011 RefCell guard held across a call` (a `Ref`/`RefMut` guard live across a call that could re-enter the same cell, `[CELL-7]`), `L4001 allocation in hot loop` (allocation inside a loop of a `@simd`/`@parallel` body or inside functions named in `[lints.hot_paths]`), `L4002 dynamic dispatch on final type` (redundant `dyn`), `L5001 unsafe extern without contract`, `L5002 FFI copy` (conversion at the boundary copying > threshold bytes), `L7001 lock held across call that may block`.

* `[LNT-1]` **`L1001 unused binding`.** A local introduced by `x = expr` or `x: T = expr` and never read on any path is reported at `warn` by default. Names beginning `_` are exempt. Liveness comes from the borrow checker's existing analysis (XIX §4.7 step 3).
* `[LNT-2]` **`L1002 assignment declares a new binding`.** When `L1001` fires for a binding whose name is within Damerau–Levenshtein distance ≤ 2 of a mutable binding in scope at that point which is *not* read between the declaration and the end of its scope, the diagnostic names the near miss, carries a machine-applicable fix-it rewriting the name, and carries `note: `x = e` declares when `x` is not in scope and assigns when it is (GRM-4)`. `L1002` is `deny` under `edition_lints = "strict"`.
* `[LNT-3]` `L1001` and `L1002` are emitted by `ember build` and `ember check`, not only by `ember lint`. A diagnostic that fires only on a separate command does not close the footgun ADR-002 names, which is this rule's entire purpose.

* `[LNT-5]` `L2005`: a `@noreload` function calling a reloadable function from inside a loop (`[HR-10a]`). The call is legal and goes through the callee's thunk, but it reintroduces per-iteration indirection in exactly the code the annotation was applied to avoid; the lint names marking the callee `@noreload` as the fix.
* `[LNT-4]` `L2004`: a `gen fn` whose body contains no `yield` (`[CORO-2]`). It compiles and runs to completion on first resume, which is legal and almost never intended; the lint names `fn` as the fix.

## XX.9 Documentation

`ember doc` renders `##` doc comments to HTML/Markdown; every public function's page shows: signature with modes, **effects**, `@noalloc`-cleanliness, thread rules (`Send`/`Sync` of parameters and result), allocation behaviour (from effect analysis), and for FFI wrappers the underlying C declaration and contract. Doc examples in fenced ```` ```ember ```` blocks are compiled and run as tests by `ember test --doc`.

---

* `[DOC-1]` **Error pages ship with their errors.** Each phase's exit criteria include `docs/errors/EXXXX.md` for every code that phase introduces. A page contains: a minimal program that triggers the error; the rendered diagnostic; one paragraph on **why the rule exists** (not a restatement of the rule); and the fix as compilable code. Every fenced `ember` block under `docs/errors/` is built by `ember test --doc`: the failing example MUST fail with that exact code and the fixed example MUST compile. `[TST-4]`'s index generator fails CI for a code with no page.
* `[DOC-2]` **The user guide is a 1.0 artefact**, not v1.1. `docs/book/` MUST exist, MUST be the landing page with the specification linked from it as the reference, and every sample in it MUST be compiled and run by `ember test --doc`. Its "getting started" chapter is identical to milestone M0 and is tested against it; it MUST include a "coming from Python" chapter carrying the three-column table of silent and diagnosed differences (`/`, `%`, the `f32` literal default, `if xs:`, `is None`, chained comparison, `xs[-1]`, `let`, `with … as`, multi-statement closures) **and the short list of things that behave exactly as in Python**, because the absence of a row is not reassurance.
* `[DOC-3]` `ember doc` moves from Phase 8 to **Phase 4**, whose effects and derives it depends on; from that point each phase documents its own surface as it lands.
* `[DOC-4]` Guide, error pages and specification are published together each release.

## XX.10 Editor and language-server architecture

* `[IDE-3]` **Resilience.** Every stage after the parser MUST produce a complete result for a file containing errors. Name resolution binds an unresolvable path to `Def::Error`; type checking assigns `Ty::Error` to any expression it cannot type; `Ty::Error` unifies with every type. A file that does not parse MUST still yield hover, completion and document symbols over the regions `[AST-2]` recovered.
* `[IDE-4]` **No codegen in the request path.** Every editor capability MUST be answerable from AST + resolution + HIR alone; none may require MIR, monomorphisation, borrow checking or a C compiler. Borrow- and effect-derived information (`[DIA-8]`, `[EFF-10]`) is surfaced as a **best-effort overlay on a completed `check`** and MUST NOT block a response when none exists. (This is also what keeps the editor independent of ADR-006's backend choice.)
* `[IDE-6]` **Session lifetime.** No compiler crate may rely on process exit to reclaim memory. Interned symbols, interned types, `TypeInfo` and instantiation caches MUST live in an explicitly owned `Session` that can be dropped and rebuilt; `ember_span::Symbol`'s leaked interner is a **v1 defect, not a v1 licence**.



## Compile-time budget

Iteration speed is a specified property of Ember, not an emergent one. A language
that rebuilds in a second is a different tool from one that rebuilds in forty, and
the difference is what decides whether Part XVIII's hot reload is real. These are
requirements, tested in CI, and they constrain language and compiler design as hard
as any semantic rule. They are about **wall-clock build time**; XIX §4.11a's
`[MONO-2..9]` instantiation budget is a different instrument measuring a different
quantity, and the two compose.

* `[BUD-1]` **Reference machine and workload.** An 8-core / 16-thread x86-64 laptop
  CPU of 2020 or later (baseline: i7-1165G7 or Ryzen 7 4800H), 16 GB RAM, NVMe SSD,
  Windows 11 with MSVC, warm OS file cache, warm build cache. The workload is
  `bench/bigpkg`, a synthetic 50k-line package whose shape mirrors engine script and
  system code: 400 modules, 1,200 types, 6,000 functions, 15% generic, one C header
  import of Vulkan's size. `[BUD-1a]` **`bench/bigpkg` grows with the compiler.**
  Phase 0 lands a Parts II–VI subset that the Phase 0 compiler can build; each phase
  extends it to the features that phase adds, and only the subset a phase can
  compile is gated in that phase. Wiring a gate against a package the compiler
  cannot yet build would fail every build from Phase 0 onward.
* `[BUD-2]` **Budgets**, each a median of 15 runs after 3 discarded warm-up runs, on
  `[BUD-1]`'s machine and the largest `bench/bigpkg` subset the current phase
  compiles:

| # | Operation | Budget |
|---|---|---|
| B1 | `ember check` after a one-function-body edit | ≤ 250 ms |
| B2 | `ember build --reload` after a one-function-body edit | ≤ 1000 ms |
| B3 | `ember build` after a one-function-body edit, `debug` | ≤ 1.5 s |
| B4 | `ember build` after a signature change in a leaf module | ≤ 2.5 s |
| B5 | `ember build` after a change to a widely imported type | ≤ 8 s |
| B6 | clean `debug` build, cold cache | ≤ 40 s |
| B7 | clean `release` build | ≤ 90 s |
| B8 | `.embind` regeneration for a Vulkan-sized header | ≤ 3 s |
| B9 | compiler peak memory, clean build | ≤ 3 GB |

  **B2 is `[HR-1]`'s one second, and it is the only definition of it.** B2 measures
  edit to loadable image: compile and link of the changed module only. The load,
  plan, prepare and commit phases of Part XVIII are measured separately by
  `[HR-32]` and are not part of B2; `[HR-1]`'s end-to-end second is B2 plus those,
  and `[BEN-8]` gates the sum.
* `[BUD-2a]` **What `ember check` verifies.** B1 covers parse, resolve, type-check,
  borrow-check and the pre-monomorphisation effect check of `[EFF-*]`. It does
  **not** cover `[EFF-9]`'s `RuntimeCheck(k)` set, which XIX §4.10 defines over the
  post-optimisation program and which therefore requires codegen; `ember check`
  MUST report `RuntimeCheck` facts as unavailable rather than stale, and
  `[IDE-4]`'s editor path already depends on exactly this split.
* `[BUD-3]` **Two gates, and they are different.** (a) An **absolute** gate: a
  measured figure above its `[BUD-2]` budget fails CI. (b) A **regression** gate: a
  measured figure more than 15% above the recorded baseline for the same commit
  lineage fails CI even when it is inside budget, because a change that consumes
  the whole margin at once is the change worth catching. Both are reported;
  either failing fails the build.
* `[BUD-3a]` **CI is not the reference machine, so CI normalises.** Hosted runners
  vary by more than the gates do. Every CI run first executes
  `tests/perf/calibrate`, a fixed single-threaded and a fixed parallel workload with
  published reference times, and divides its measurements by the resulting factors
  before comparing. A run whose calibration itself varies by more than 10% across
  its own repetitions is **inconclusive**, not a failure: it is retried once and
  then reported as unmeasured. `[BUD-3b]` The gates apply only to figures whose
  measured spread is below the gate — a 15% gate over a distribution with a 20%
  interquartile range measures the runner, not the compiler — and
  `tests/perf/build/` MUST publish the observed spread beside every figure.
* `[BUD-4]` **Design constraints, recorded as consequences** so they are not
  re-argued per feature: (1) no analysis whose *correctness* requires whole-program
  knowledge — XIX §4.11a's instantiation decisions are an optimisation, taken over
  a package, and a build that skips them is still correct; (2) monomorphisation is
  cached per instantiation and reused across builds; (3) a module's interface hash
  carries the cross-module facts its dependants need; (4) codegen is per module and
  parallel; (5) `comptime` results are cached by input hash; (6) an unchanged module
  is never re-parsed. `[BUD-4a]` Constraint 3 is a statement about *what the hash
  covers*, not a claim that private bodies are irrelevant: effect sets are inferred
  from bodies (`[EFF-1]`), so a private body change does change the interface hash
  when it changes the function's effect set, and the build graph MUST treat it as
  such. A hash that ignored bodies would make `[EFF-4]`'s invalidation rule unsound.
* `[BUD-5]` **The budget has veto power.** A proposal that adds a language or
  compiler feature MUST state its measured effect on **B3**, the one-function-body
  `debug` build, and is rejected if that effect exceeds 5 percentage points or the
  measured spread of `[BUD-3b]`, whichever is larger. B3 is the row this rule
  names; B1 and B2 are reported alongside but do not gate a proposal. This is the
  principled basis on which Ember has no macros, no specialisation, no
  higher-kinded types and no unbounded trait-solver search — each is a known
  multi-second tax in a language that has it.
* `[BUD-5a]` **The rule applies to the features already accepted.** Part XVIII's
  hot reload costs `[HR-9]`'s ≤ 3% run-time indirection and no measurable B3 time,
  because thunk emission is per function and the reload manifest is written from
  data codegen already has. Part XVI's trampolines and member mapping cost
  `.embind` generation time only, gated separately by B8, and nothing at B3 for a
  package that imports no C++ header. Both figures MUST be reported by
  `tests/perf/build/` from Phase 7a, and a proposal that cannot state its figure
  is not ready to be accepted.
* `[BUD-6]` `ember build --timings` prints the per-phase breakdown for the build
  just run, and `--timings=json` writes it for tooling. It is how a programmer
  answers "why was that slow" without instrumenting the compiler.

---


## XX.11 Conformance profiles

A specification this size can be internally consistent and still be too large to
implement correctly in one pass. The danger is not a wrong rule; it is a **partial
implementation that becomes the de facto language**, with users depending on
whichever subset happened to work. Profiles make the subset a declaration rather
than an accident.

* `[CONF-1]` A compiler claiming to implement Ember MUST declare which profile it
  implements, and `ember --version` MUST print it. The profiles are cumulative:
  each includes every profile before it.
* `[CONF-2]` **Ember Core** — Parts II–VII and XIII: lexer, parser, types,
  `struct`/`enum`, ownership, borrowing, regions, functions and closures,
  `Result`/`Option`, modules, and range types (`[RNG-*]`). A Core implementation is
  a usable language for value-oriented code and is the Phase 0–2 exit.
* `[CONF-3]` **Ember Systems** — adds Parts VIII–XII and XIV: classes and RC,
  exclusivity, arenas and handles, effects, threads and jobs, SoA/SIMD/ECS, and
  compile-time reflection.
* `[CONF-4]` **Ember Native** — adds Parts XVI and XVII: the C and C++ importers,
  `.embind`, overlays, trust grades, the trampoline of `[FFI-39]`, and the GPU host
  model.
* `[CONF-5]` **Ember Dynamic** — adds Part XVIII: hot reload, schema migration,
  relocation and runtime inspection. `[HR-19]`'s bodies-only tier is the first
  half of this profile and MAY be declared as `Dynamic (bodies)`.
* `[CONF-6]` A profile is claimed only when **every** conformance test for the rules
  it names passes (`[TST-4]`). A partial profile is declared as the profile below
  it, never as the profile above with exceptions — an implementation that claims
  `Native` while failing three `[FFI-*]` tests is claiming `Systems`.

## XX.12 ABI protocol versions

The C ABI is the stable boundary, but there is more than one boundary, and a
mismatch on any of them is a silent memory-corruption bug rather than a link error.
Each therefore carries a version that is checked before any code runs.

* `[ABI-1]` Four protocols are versioned independently, each an integer that
  increments on any layout, calling-convention or symbol-set change:
  `EMBER_C_ABI` (the C header a package emits, `[FFI-28]`), `EMBER_RUNTIME_ABI`
  (package ↔ `ember_rt`), `EMBER_RELOAD_ABI` (reloadable image ↔ runtime, covering
  the manifest of `[HR-8]`, the thunk table and the schema encoding), and
  `EMBER_CPP_BINDING_ABI` (generated thunks ↔ the C++ side, `[BLD-FFI-5]`).
* `[ABI-2]` Each version is emitted as a defined symbol whose **name encodes the
  value** — the device `[BLD-FFI-1b]` and `[HR-12a]` already use — so a mismatch is
  an unresolved-symbol **link error** naming both versions, not a runtime crash.
  A protocol that cannot be checked at link time (`EMBER_RELOAD_ABI`, since images
  load at runtime) is checked on load and refuses with `E9037`.
* `[ABI-3]` `ember_module_init` verifies `EMBER_RUNTIME_ABI` before it does anything
  else and returns a distinguished failure code on mismatch, so a host embedding a
  stale package learns at initialisation rather than at first call.
* `[ABI-4]` A version increment is a **release-note obligation**: `[VER-*]`'s
  compatibility statement MUST name every protocol whose version changed and why.
* `[ABI-5]` These versions are independent of the language version. Two compilers
  claiming Ember 0.7.2 may differ in `EMBER_RELOAD_ABI` — reload metadata is
  implementation-defined — and MUST NOT be mixed in one process; they may not
  differ in `EMBER_C_ABI`, which `[FFI-28]` fixes.

## XVI.7a C++ exception policy

* `[FFI-43]` **Every imported C++ function carries an explicit exception policy**,
  and there is no default: an import whose policy the header and overlay both leave
  unstated is `E5061`, naming the two spellings. This is the one place a silent
  assumption becomes undefined behaviour crossing an ABI, so the specification
  requires the programmer to have decided.
  * `@ffi(throws = "translate")` — the generated thunk wraps the call in
    `try`/`catch (...)`, converts to `Result[T, CppError]` per `[FFI-24]`, and the
    Ember signature returns that `Result`. `CppError` carries `what()` where the
    exception derives from `std::exception` and an opaque marker otherwise.
  * `@ffi(throws = "noexcept")` — the thunk asserts the callee is `noexcept`; if it
    throws anyway the thunk calls `std::terminate` at the boundary. The Ember
    signature returns `T` directly. This is the correct declaration for a function
    the header marks `noexcept`, and the importer applies it automatically in that
    case rather than requiring it to be written.
* `[FFI-43a]` **A contradictory policy is rejected, not resolved.** An overlay
  declaring `throws = "noexcept"` for a function the header declares as potentially
  throwing, or two composed overlays (`[FFI-30c]`) declaring different policies for
  one function, is `E5062` naming both sources. An exception MUST NOT cross an
  Ember frame in either direction under any policy; `[FFI-39e]` states the inbound
  half of the same rule.


---

# Part XXI — Implementation Plan
* `[IDE-1]`, `[IDE-2]`, `[IDE-5]`, `[IDE-7]`..`[IDE-10]` are **reserved** for the language server itself, a named milestone before 1.0 rather than a v0.4 commitment: `ember lsp` (stdio, LSP 3.17) with `tests/ide/` in the conformance suite; the v1 capability list (diagnostics, hover including the effect set, definition, references, symbols, completion, signature help with parameter modes, rename, formatting, code actions for every machine-applicable `[DIA-1]` fix-it, semantic tokens, inlay hints); incrementality keyed on `[BLD-2]`'s interface hash within `[BLD-7]`'s budget; parity between `ember check` and `ember lsp` tested over the conformance corpus; and editor client packages in `editors/`. `[IDE-7]` **cost inlay hints** is the item worth specifying even if it lands late: the inferred type of each `x = expr`; the parameter mode at each call argument where it is not written; an `alloc` marker at each expression contributing the `Alloc` effect; at each site in the `[EFF-10]` side table, the check kind and its `[EFF-11]` reason code; and each inserted `Retain`/`Release`. That is the per-line form of what Part X mandates be visible.

## XXI.1 Ground rules for the implementing agent

1. Work in the order of the phases below. Do not start a phase's optional items before its **exit criteria** pass.
2. Every phase adds tests before code: write the `tests/conformance/<rule>/` cases from the spec text, then implement until they pass.
3. Keep `docs/DECISIONS.md` (ADR format: context, decision, consequences, spec rule affected). Any deviation from this document requires an ADR and a spec patch in `docs/spec-errata.md`.
   * The single-file specification under `docs/spec-source/` is the **normative source**. `docs/spec/` is generated by `tools/split_spec.py` and MUST NOT be hand-edited; an errata ruling is applied to the source document and the split is regenerated. A revision authored from an unpatched copy silently reverts every ruling that exists only in the generated files — which is how 0.3 lost four of them.
4. Keep `docs/HANDOFF.md` in the RageV style: current state, what is done, what is not, load-bearing invariants, traps paid.
5. `cargo test` must be green at every commit; `ember test` on `std/` must be green from Phase 3 onward.
6. Prefer boring implementations. No novel algorithms where a textbook one exists (Pratt parsing, union–find inference, Maranget pattern compilation, NLL).
7. Do not build the LLVM backend, async, named lifetimes, procedural derives, or the kernel language in v1 even if they seem easy at the time.

## XXI.2 Phases

### Phase 0 — Skeleton (exit: `hello.em` compiles via C and runs on Windows + Linux)

* Workspace, crates, CI (GitHub Actions: windows-latest with MSVC + clang-cl, ubuntu-latest with clang + gcc).
* `ember_span`, `ember_diag` (rendering + JSON), `ember_lexer` with the indentation algorithm (`[LEX-*]` tests), `ember_parser` for functions, calls, literals, `if`/`while`/`for`, structs (no generics), `ember_ast` pretty-printer.
* Minimal `ember_types`/`ember_typeck` for scalars, structs, `void`, function calls, locals.
* Straight-line MIR lowering (no borrowck yet), `ember_codegen_c`, `ember_rt` with `ember_alloc`, `ember_panic`, `println` for scalars/`str`.
* `ember build/run` driving `cl.exe`/`clang`.
* **Milestone test**: M1 (§XXI.3; `Vec3` add) compiles to C with no heap allocation (assert by grepping the C for `ember_alloc`) and prints `5`.

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
* NLL borrow checker (§XIX.4.7) incl. two-phase borrows, disjoint fields, reborrows, inferred multi-region `@view` structs, elision rules, `@borrows`.
* Closures (`[CLO-*]`), `Callable`, `fn(A)->R` generic parameters.
* `Arena`, `FixedArena`, `ScopedArena` (`[ARN-*]`), `unsafe`, raw pointers, `MaybeUninit`, `transmute`.
* `Cell[T]`, `RefCell[T]`, `Ref`/`RefMut` guards (`[CELL-*]`), `std.cell`; `assert_disjoint`/`assume_disjoint` (`[DSJ-*]`) with the proof-carrying return and the backend aliasing facts.
* Diagnostics quality pass on borrow errors: the full shape catalogue of §XX.6.1 with a `ui/borrow/<shape>/` snapshot each (`[DIA-3]`, `[DIA-7]`, `[DIA-10]`), the classifier, and `ember explain --borrow` (`[DIA-8]`).

### Phase 3 — Objects (exit: `[OBJ-*]`, `[RC-*]`, `[EXC-*]`, `[DSP-*]`, `[WK-*]` tests; static and runtime cycle diagnostics work)

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
* *(v0.6)* `std.gpu`: handles, `Device` interface, `Frame`, `Ring`, access states, deferred destruction, `History`, `ShaderInterface`, `ember shader-bind` from SPIRV-Cross JSON — implementing Part XVII's existing `[GPU-*]` rules, and Part XXII's `[RV-*]` rules for RageV Stages 2–3.
* The MIR interpreter becomes a **supported restricted execution mode** (`ember run --interp`) within a declared intrinsic and file-I/O capability set; native FFI is not implicitly available. Differential execution against the native backend is mandatory for deterministic programs. WebAssembly is treated as an implementation target of that mode, never as a semantic dependency of the language.
* **The GPU host model is scheduled, not respecified.** Part XVII's `[GPU-*]` rules and Part XXII's `[RV-*]` rules remain the sole normative source for that surface; implementing them is **v0.6 work** and is not a v1 exit criterion. v0.5 MUST NOT make an architectural decision that would require redesigning them — the compatibility surface that must survive is: Vulkan-capable C/C++ FFI, imported value types and opaque handles, `unsafe overlay` boundaries, callback-bound command signatures (which `[LT-7]` now supplies), frame and arena lifetime primitives, deferred-destruction API shapes, a stable runtime ABI boundary, and the metadata shader reflection needs. If v0.6 requires a genuinely new GPU rule it takes an unused id after the existing Part XVII range; an existing `[GPU-*]` id MUST NOT be reused for a different meaning.

### Phase 7a — Iteration, determinism and instantiation cost (exit: `[CORO-*]`, `[DET-*]`, `[HR-1..19]`, `[BUD-*]`, `[MONO-2..9]` tests)

* Coroutines: `[MIR-6]`'s lowering, `[CORO-6]`'s borrow restriction with its diagnostic, per-suspension-point drop glue, `std.coroutine`. Conformance includes a coroutine dropped at every suspension point with a leak check on each.
* Determinism: the `Nondet` effect through the existing `[EFF-1]` fixpoint, `std.math.det`, `[BLD-13]`'s reproducible build with a two-machine byte-comparison test, `--build-id`.
* Hot reload, in two tiers. **Tier 1 (`[HR-19]` bodies-only)** first: permanent thunks (`[HR-6]`), the reload manifest (`[HR-8]`), the depth-counter safe point (`[HR-3]`), the schema diff as a refusal check, and the watcher — no migration, no 40-byte header, no live-instance list. Acceptance: edit a gameplay function body, save, see it in the running host with the scene loaded and no entity re-created. **Tier 2 (`[HR-11..18]`, Phase 7b)** adds schemas, the live-instance list, the four-phase migration of `[HR-2]`/`[HR-2a]`, and relocation. Acceptance: add a field to a live script class with 400 instances, save, and see every instance keep its other fields — with a leak check and a forced panic in `migrate_from` proving `[HR-2]` holds.
* The compile-time budget: `bench/bigpkg` at the current phase's subset (`[BUD-1a]`), `tests/perf/calibrate`, and both gates of `[BUD-3]`.
* Instantiation budget: `[MONO-2]`'s counting, the report, `W2220`, `[MONO-5]`'s shareability analysis, and shared emission over the vtable machinery `[TYP-22]` already requires. Acceptance is that a package with a ceiling set and one shared generic produces the same test results, byte-identical `--build-id` aside, as the same package with `@always_specialize` everywhere.

### Phase 8 — Hardening and 1.0 (exit: full conformance + perf suite + two external packages)

* `ember lint` full set, `ember doc`, doc tests, `ember explain`.
* Fuzzing targets, differential testing against the interpreter.
* Package registry client (v1.1), LLVM backend (v2), PGO/ThinLTO (v2). LSP is a 1.0 tooling requirement under `[GATE-6]` and is implemented in Phase 8.

## XXI.3 Milestone acceptance tests (must exist verbatim in `tests/milestones/`)

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

## XXI.4 Performance suite

`tests/perf/` holds paired `.em`/`.cpp` programs and thresholds; `ember bench --compare` runs both with the same C++ compiler, 10 iterations, reports median. Gates: scalar/tight loops ≤ 1.05×, SoA/SIMD ≤ 1.10×, FFI call overhead = 0 extra instructions for ABI-direct calls (asm diff), RC-heavy object code ≤ 1.3× of equivalent Swift-style hand-written C++ with `shared_ptr` (informational, not gating), allocation counts asserted exactly for `@noalloc` paths.

* `[BEN-1]` **Protocol.** Each benchmark runs ≥ 3 untimed warm-up repetitions followed by ≥ 30 timed repetitions, in a process pinned to a single core, with the timing loop's working set stated. The harness reports the median and the 95 % bootstrap confidence interval of the median for each side and for their ratio.
* `[BEN-2]` **Gating.** A gate is failed only when the **lower** bound of the ratio's 95 % confidence interval exceeds the threshold. A point estimate above the threshold whose interval includes it is reported *inconclusive* and re-run, never failed.
* `[BEN-3]` **Noise floor.** Every run additionally measures the C++ reference against a second, independently linked copy of itself. If that self-ratio's interval excludes 1.00 ± 0.02, the run is **void** — neither pass nor fail — and the machine is reported unfit for gating.
* `[BEN-4]` **Deterministic signal.** Every benchmark records retired instructions, and cycles where the platform provides them. Instruction count is deterministic for a fixed binary and input; a change beyond ± 0.5 % against the recorded baseline is a hard failure independent of `[BEN-2]`, and is the signal used in CI where `[BEN-3]` voids the timing gate.
* `[BEN-5]` **Configuration equality.** Both sides MUST be built with the same optimisation level, the same LTO setting (`[BLD-6]`), the same floating-point model (`[TYP-9a]`), and the same target-CPU flags. The harness records all four; a mismatch voids the run. M6 and every entry of §XXI.4 are subject to this rule.
* `[BEN-6]` **Baselines.** Gates: scalar/tight loops ≤ 1.05×; SoA/SIMD ≤ 1.10×; `@noalloc` paths assert allocation counts exactly. FFI call overhead is gated on **retired instructions for the call sequence** being equal to the C++ reference's, not on an assembly diff. RC-heavy object code is compared against a **non-atomic intrusive reference count** in C++ when the Ember class under test is `!Sync` (gate ≤ 1.15×), and against `std::shared_ptr` only for `Sync` classes (informational). Comparing a non-atomic Ember count against `shared_ptr` is forbidden.
* `[BEN-7]` XXII.6's pass criteria validate the already-selected ADR-008 architecture. `[BEN-1]`..`[BEN-6]` may fail the implementation for performance or correctness, but a void or inconclusive benchmark MUST NOT reopen the architectural decision.
* `[BEN-8]` **End-to-end reload.** `[HR-1]`'s one second is edit-to-visible, which `[BUD-2]`'s B2 does not measure on its own: B2 ends at a loadable image, and the load, plan, prepare and commit phases of Part XVIII follow it. `bench/reload/` MUST measure the sum on `[BUD-1]`'s machine, over `bench/bigpkg` with a live set of 10,000 registered instances, for three edits — a function body, a field added with a default, and a field requiring `migrate_from` — reporting the split `[HR-32]` exposes. The first MUST meet one second; the other two are recorded and are what `[HR-1]` is measured against once Phase 7b lands, since a claim about migration cannot be tested by an edit that migrates nothing.

## XXI.5 Repository layout

```
ember/
  Cargo.toml                 workspace
  compiler/…                 crates per §XIX.1
  runtime/ember_rt/          C11 runtime, CMakeLists.txt, vendored mimalloc
  std/                       ember.toml + src/**.em
  tools/                     fmt, lint, shader-bind, rule_index.py, perf harness
  cmake/EmberModule.cmake
  tests/                     per §XX.5
  docs/
    spec/                    THIS DOCUMENT split by part, plus spec-errata.md
    DECISIONS.md  HANDOFF.md  errors/EXXXX.md  book/ (user guide, v1.1)
  examples/                  hello, particles, ecs_demo, vulkan_triangle (via RageV RHI or raw volk)
```

---
## XXI.6 The 1.0 release gate

"Phase 8 exits at full conformance" is not a release criterion, it is an intention.
This section is the criterion: every line is a number or a binary, and 1.0 ships when
all of them hold. Nothing here is new machinery — each row names an instrument the
document already specifies.

* `[GATE-1]` **Language completeness.** Every normative rule has a conformance
  directory that passes and contains both an accept and, where the rule can reject
  source, a reject case (`[TST-4]`, `[TST-4a]`), with `[TST-4c]`'s baseline **empty**; every diagnostic code has a registry entry and a
  `docs/errors/` page, in both directions (`[DIA-6a]`); zero owner decisions in
  XXIII.1 remain open for a v1 requirement (`[GATE-8]`); zero known soundness
  defects, where "known" means recorded in `docs/spec-errata.md`.
* `[GATE-2]` **Compiler correctness.** The C backend passes the whole semantic
  suite. Where the LLVM backend exists it passes the same suite, and differential
  execution across the two produces no divergence that is not explained by an
  `implementation-defined` row of `[COST-3]`. The MIR verifier passes on every
  compiler test. Fuzzing has run ≥ 72 CPU-hours on the parser and ≥ 72 on the
  type checker since the last change to either, with zero crashes and zero
  assertion failures outstanding.
* `[GATE-3]` **Safety.** Zero known memory-safety or data-race defects reachable
  from Safe Ember as `[PHIL-10]` defines it. Every `unsafe` block carries a
  `[UNS-7]` obligation and a `[UNS-9]` category. Every foreign fact is graded and
  appears in `ember tcb`; the count of `asserted` facts with no test exercising them
  is published in the release notes rather than being zero, because `[PHIL-5]`
  forbids pretending otherwise.
* `[GATE-4]` **FFI.** The C ABI suite passes on every supported target triple. The
  `[CXX-6]`'s nine C++ migration conditions are met on the corpus of `[CXX-1]`/`[CXX-2]`. The
  C++ layout suite passes on every supported `[cpp.<project>]` configuration —
  MSVC and Clang at minimum, each in debug and release CRT. Runtime ABI
  compatibility tests pass for every version pair `[ABI-1]` declares compatible, and
  every incompatible pair produces the diagnostic `[ABI-2]` requires rather than a
  crash.
* `[GATE-5]` **Performance.** `[BUD-2]`'s nine build budgets met on `[BUD-1]`'s
  machine, and both gates of `[BUD-3]` green. `[BEN-1..8]` met, including
  `[BEN-8]`'s end-to-end reload second. Runtime safety overhead is **measured and
  published per check class**, not estimated: the `[COST-3]` rows marked
  conditionally elidable each have a figure for how often they were elided on the
  RageV workload.
* `[GATE-6]` **Tooling.** The first-hour test passes: install, `ember new`, build,
  run, break something, read the diagnostic, fix it — on Windows and Linux, from a
  clean machine, without consulting this document. The formatter is idempotent over
  the whole corpus. The diagnostics snapshot suite passes. The language server
  answers `[IDE-4]`'s capabilities on a 50k-line package. `ember inspect` output is
  schema-versioned so tooling can depend on it.
* `[GATE-7]` **Conformance declaration.** The reference implementation declares
  `Ember Dynamic` under `[CONF-1]` — the full ladder — and the declaration is
  produced by the test run rather than written by hand.
* `[GATE-8]` **No requirement rests on an open question.** A v1 requirement whose
  behaviour depends on an unresolved `OQ-n` blocks 1.0. An `OQ` may remain open only
  if it is classified `deferred-v2` under `[GATE-8a]`.
* `[GATE-8a]` **Open-question status is explicit.** Every entry in XXIII.1 carries
  exactly one of: `decided` (the owner ruled; the rule text reflects it),
  `implementation-confirmed` (decided and a conformance test pins it),
  `deferred-v2` (out of scope for 1.0 and no v1 requirement depends on it), or
  `open` (blocks 1.0). The statuses are assigned by the same one-time pass as
  `[CAT-2]`, defaulting from the prose already in each entry — an entry reading
  "Closed" becomes `decided`, one naming a conformance test becomes
  `implementation-confirmed`, one naming a later version becomes `deferred-v2`, and
  anything else becomes `open` for the owner to rule on. `tools/rule_index.py`
  fails CI on an entry with no status, and on an `open` entry cited by a
  `LANGUAGE-NORMATIVE` rule, **from the commit that lands that pass**.

---

# Part XXII — RageV Integration Plan

RageV today: a C++ static library (`RageV`) linked into `RageVEditor.exe`/`RageVRuntime.exe`; a backend-agnostic RHI over Vulkan 1.3 (dynamic rendering, synchronization2, `volk`, VMA) and OpenGL 4.5 DSA; `.rvshader` → glslang → SPIR-V (+ SPIRV-Cross for GL and reflection); a sparse-set ECS with `Entity = 20-bit index | 12-bit version`; a render/frame graph with temporal history, RT/GI signal passes; Jolt, miniaudio, ImGui, yaml-cpp, cgltf; C# scripting hosted through `DotNetHost` where **managed → native is a fixed-order table of `__cdecl` function pointers (`NativeApi`) with a protocol version, and native → managed is a set of static blittable entry points**; every crossing value is blittable, entities cross as `uint64_t` UUIDs, strings as UTF-8 copied at the boundary. Editor-visible script fields are declared with `RVShowInEditor` markers read by `rvgen`.

The plan replaces nothing at first and adds Ember beside C#, matching the existing boundary exactly.

## XXII.1 Stage 0 — Ember as a second scripting language (needs Phases 0–5)

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

## XXII.2 Stage 1 — Engine C API and idiomatic bindings (Phase 5)

Grow `Interop.h` (or a new `RageVCApi.h` generated by extending `rvgen`) into a C surface for: scene queries, component get/set (blittable components), input, audio, physics queries (Jolt via the engine), asset handles, debug draw, and RHI-independent renderer settings. Every entry gets an overlay contract; `ragev_api` exposes idiomatic Ember (`Result`, `Option`, `Span`) over it. `[RV-3]` Component types shared with the engine are `@layout(c)` Ember structs mirrored from the C++ headers with comptime layout assertions against the `.embind` layouts (`[FFI-5]`).

## XXII.3 Stage 2 — `std.ecs` systems over RageV storage (Phase 6)

The architecture is fixed by ADR-008: RageV integration uses the library-owned `std.ecs` model over
Ember's existing `SoA`, `Pool`, `Handle`, access-set and borrow primitives. There is no competing
compiler-integrated ECS and no owner-level choice between two ECS architectures.

The first migration path MAY bridge RageV's existing sparse-set storage through a narrow C ABI that
exposes canonical `Layout[T]`-compatible component arrays as borrowed `Span`/`MutSpan` values. That
bridge is an implementation strategy of `std.ecs`, not a second language architecture. If storage is
later moved to Ember-owned `std.ecs.World`, the public query/access semantics remain unchanged and
no compiler pass acquires an ECS-specific dependency.

Target systems for Ember first: particle simulation (`SoA`, `@parallel`, `@simd`), CPU frustum/occlusion
pre-pass feeding `GpuCull`, animation pose evaluation, and scene graph transform walks. Multi-region
query views MUST preserve the independent component-column provenance specified by `[LT-14]`–`[LT-43]`.

## XXII.4 Stage 3 — Renderer orchestration (Phase 7)

Bind `RHIDevice`, `RHICommandList`, `RHIResourceSet`, `RHIPipeline` through the C++ importer (`import cpp … classes=[…]`) with an overlay that maps the engine's raw pointers to `std.gpu` handles (`Handle[Texture]` ↔ `RHITexture*` via a `Pool` on the Ember side; the engine's own deferred destruction stays authoritative). Then move **frame orchestration** — `FrameGraphBuilder`-level decisions: which passes run, at what resolution, with which history, and the `TemporalHistory` bookkeeping — into Ember, using `std.gpu.History` and access-state checks to make "GI signal read by nobody" and "history refused by validity flag" (the traps recorded in `HANDOFF.md`) into compile-time or debug-time errors: a `History` whose `cur` is never bound in a frame is reported by the device in debug (`W-runtime: history 'gi' written but not read this frame`). Shader interfaces come from `ember shader-bind` on the existing `shaderinfo` reflection so that binding 16's dual meaning (`RayRates.w` bit 24) becomes a typed enum in the interface rather than a comment.

`[RV-4]` The Vulkan and OpenGL backends, `volk`, VMA, swapchain and window code remain C++ ("native islands") indefinitely; nothing in Ember requires their rewrite. Vendor extensions remain reachable through `import c "vulkan/vulkan.h"` + `unsafe: cmd.native()`.

## XXII.5 Stage 4 — RT/GI signal scheduling and tools (v1.1+)

Once Stage 3 is stable: the signal-processing chains (trace → guidance downsample → contract → upsample) are pass sequences whose resolution, ordering and buffer aliasing are orchestration — the kind of code where Ember's `@noalloc` frame arenas, `History`, and the graph's automatic transient lifetimes pay off. Offline tools (`diff_still`, `terrain_lod` replicas currently in Python) become `ember` programs sharing the engine's math and serialisation types.

## XXII.6 Evaluation matrix (measure before moving each area)

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

# Part XXIII — Open Questions, Non-Goals, Glossary, Rule Index

## XXIII.1 Owner decisions (`OQ-n`)

Every question this document has put to the owner carries a **stable `OQ-n` identifier**.
Identifiers are never reused and never renumbered: 0.3 renumbered this list and thereby broke
ADR-009's citation of "#9", which is why the numbering is now fixed. **Every question 0.4 raised is answered.** Three errata rulings inherited from 0.2 remain outstanding and are
recorded below as `OQ-24`..`OQ-26`; the implementation has followed all three since Phase 0, so each is a
confirmation rather than a design choice. Part XXI §1 ground rule 3 continues to apply: an implementing agent
records a provisional choice and flags it, and does not decide silently.

`OQ-1`..`OQ-7` were confirmed on 2026-09-07 and are recorded as ADR-001..ADR-007. `OQ-8` was resolved by 0.9_Hardened_12 and is recorded as the closed ADR-008 architecture. `OQ-9`..`OQ-23`
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
| `OQ-8` | RageV ECS integration shape (a) or (b) | **Shape (a): library-level `std.ecs` over `SoA`/`Pool`/access sets/comptime reflection; no compiler-integrated ECS. — ADR-008; closed by 0.9_Hardened_12** |
| `OQ-9` | Language and CLI naming | **Keep `Ember` as the language name.** Language identity is separated from CLI, package, registry and runtime-ABI identity; the runtime symbol prefix is generated from `EMBER_SYMBOL_PREFIX` (`[RT-5]`, `[MNG-5]`, `[MAN-4]`), so a later rename of any one of them is a build-record change rather than a source break. |
| `OQ-10` | `Cell`/`RefCell` in the prelude (`[CELL-11]`) | **In the prelude.** `Shared[T]` was already there and is heavier on every axis; taxing the zero-cost facility inverted the gradient. `[DIA-9]` still bars them as a *first* suggestion. |
| `OQ-11` | `@static_safe` scope (`[EFF-12]`) | **Excludes only non-proof-establishing `RuntimeCheck(Aliasing)`.** A check whose reason is `establishes_static_fact` is permitted, because it returns proof-carrying values the borrow checker and backend consume. |
| `OQ-12` | Is ERR-005 decided? | **Confirmed.** `[LEX-11a]` is normative and unqualified. `docs/spec-source/` is authoritative; `docs/spec/` is generated and never hand-edited. |
| `OQ-13` | Does a call site write `mut`? | **No.** The callee's declaration determines the mode; `f(x)` is written for every mode (`[FN-2a]`). Part 0 row 3's "call sites never write `&`" is preserved verbatim. |
| `OQ-14` | Are `return`, `break`, `continue` expressions? | **Yes** — expressions of type `!` at the lowest precedence (`[GRM-16]`), which is what Part IV §2's `!` producer list already implied. |
| `OQ-15` | Does `::` stay? | **Stays**, as the qualified module/type/namespace path separator. `.` remains instance and member access. |
| `OQ-16` | Re-confirm ADR-004's access cost | **Decision kept; the number is not.** No fixed figure is normative. The cost is re-measured under `[BEN-1]`–`[BEN-7]` and published, and `ember inspect --safety` reports each access as `STATIC`, `ELIDABLE` or `DYNAMIC`. |
| `OQ-17` | Late-bound callback regions (`[LT-7]`) | **Adopted and extended.** Callback regions remain compiler-internal; `0.9.5_Hardened_1` additionally permits storable `@view struct` values to preserve multiple inferred regions without exposing lifetime syntax. |
| `OQ-18` | What does `let` freeze? | **The binding, not the value** (resolution A). `[EXC-4]` narrows to instantaneous reads; a long-term access to a `let` field registers exactly as for any other field. `[CLS-8]`/`[THR-1]` restated so a `let Array[T]` field does not by itself make a class `Sync`. |
| `OQ-19` | Containers of view types | **Arbitrary owning containers stay rejected.** `[TYP-15a]` admits `BorrowList[T]`/`ViewList[T]` only when one inferred container region outlives every region slot of each element; region slots remain compiler-internal. |
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
| View | a type carrying a borrow (`ref`, `Span`, `@view struct`, non-`owned` closure); `@view struct` values may carry one or more inferred regions |
| Region vector | compiler-internal set of independently inferred region slots carried by a multi-region `@view struct`; erased before ABI/code generation |
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

# Appendix A — Syntax quick reference

Generated from `docs/spec-source/appendix-a.em`, a conformance fixture that compiles (`[TST-6]`). Every
construct below parses under Part III; nothing here is illustrative shorthand.

```ember
#! language "0.9.5"
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


---

## CONSOLIDATED_NORMATIVE_BODY_END — 0.8.5 lineage

Everything between the consolidated-body markers is current normative text. Material after this
marker is specification tooling guidance for the current revision, not inherited 0.8.5 text.

# Appendix B — 0.9.5_Hardened_9 Consistency Checklist

The following checks are mandatory CI checks for the consolidated specification itself:

1. Exactly one current version declaration exists and names `0.9.5_Hardened_9`.
2. The immediate predecessor is `0.9.5_Hardened_8`.
3. `[MOD-6]` and `[MOD-6a]` name the supported `0.9.5`, `0.9`, and 0.8.x source-language contracts explicitly, and `0.9` does not silently enable `0.9.5`.
4. `EMBER_RUNTIME_ABI` is canonical; `EMBER_RT_ABI` can occur only in compatibility-alias documentation.
5. Every `dyn Writer`/`dyn Reader` serialization signature uses a legal indirection.
6. Manifest examples use current versions unless explicitly marked historical.
7. Phase 7a and Phase 7b are distinct headings.
8. Phase 8 does not classify LSP as v1.1 while `[GATE-6]` requires it for 1.0.
9. The consolidated normative body has exactly one begin marker and one end marker.
10. The current addenda and consolidated body form one normative source; frozen predecessors remain separate historical artifacts.
11. Every rule ID has exactly one definition without a duplicated registry or position-based supersession algorithm.
12. `[LT-8]`–`[LT-13]` are present as recovered canonical library rules, `[LT-8]` explicitly provides shared and all-mutable helper families using `Span` and `MutSpan`, their source and 0.9.5 supersession boundary are explicit, and they have regression coverage obligations.
13. `[LT-14]`–`[LT-43]` have both accept and reject conformance coverage where applicable.
14. `[MIR-REG-1]` is in the active rule index.
15. `[CLI-15]` retains the 0.6.3 CLI-surface completeness rule and `[CLI-17]` owns cycle inspection; `[RT-1]`–`[RT-4]` and `[DIA-18]` likewise retain their inherited meanings.
16. `E3065` has exactly one diagnostic registry entry, a required error-page path, and a required B14 fixture path.
17. No current text claims implementation/conformance merely from specification prose.
18. Every current-revision rule has an evidence status; absent evidence remains `SPECIFIED`, not `IMPLEMENTED`.
19. Every heading-defined rule is extracted directly by `tools/rule_index.py`; the non-normative heading index is only a navigation aid.
20. Rule IDs are unique and historical IDs are never silently reused for a new meaning.
21. Every canonical artifact named by `[IMP-3]` has exactly one authoritative owner.
22. A specification CI failure never changes language semantics automatically; it blocks release/conformance until reconciled.
23. `[FN-6]`, `fn_type`, `[LT-8]`, `[LT-11]`, `[LT-11a]`, and `[TST-20]` agree that `_mut` identifies the helper family while explicit `mut` carries callback mutation authority.
24. `[LT-8]` makes shared helper inputs borrowed and mutable helper inputs `mut`; neither family consumes an input view.
25. `[FN-6a]`, `[CLO-3]`, `[CLO-6]`, `[LT-8a]`, and `[TST-21]` preserve the full callable mode vector as compile-time-only canonical metadata through `Callable[Args, R]`.
26. ODR-006 and ODR-007 are closed with traceable owner authority and are not presented as implementation choices.
27. `[LT-1a]`, `[LT-4]`, `[LT-4a]`, `[LT-4b]`, and `[TST-22]` agree that `Arena` is a narrow non-view provenance source for Arena-backed returns, while arbitrary non-view parameters remain forbidden.

A failure of any checklist item is a specification CI failure, not a compiler implementation choice.
