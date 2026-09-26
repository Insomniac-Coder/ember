# Ember Simplification Pass — Revised
## Package-Local Inference, Stable Public Contracts, and Targeted Semantic Repairs

**Proposal revision:** 2.0 — post-pushback revision  
**Date:** 26 September 2026  
**Language baseline:** `Ember_v0.9.9_Hardened_23.md`  
**Revises:** `Ember_Simplification_Pass_Consolidated.md`, proposal revision 1.0  
**Original proposal reviewed:** `Ember_Generic_Simplicity_Pass.md`  
**Status:** Design proposal for review; not an adopted language specification or an implementation claim.

> **Keep requirements explicit at the package boundary. Infer repetitive requirements inside the package only where a bounded, specified procedure is worthwhile. Fix semantic inconsistencies directly; do not make a compiler-architecture project a prerequisite for every simplification.**

---

## 1. What this revision changes

This is a replacement for the earlier consolidated pass, not an addendum that must be mentally combined with it. It incorporates the subsequent pushback and the revised analysis. All thirty-one `SP-` review identifiers remain traceable, but their disposition and priority have changed.

The main decisions are:

1. **Public generic contracts remain declaration-based.** Body-derived bound inference is a later feature for package-private ordinary helpers, not for APIs visible to dependent packages. The proposed `@bounds(explicit)` attribute is withdrawn.
2. **Use the smaller float-comparison repair.** Float relational operators retain IEEE meaning in concrete and generic code. Floats continue to implement `Ord`; `cmp`, `sort`, and the existing total-order `min`/`max` remain total. The proposed `PartialOrd` redesign and the requirement to rewrite ordinary float sorting are withdrawn from this pass.
3. **Separate outcomes from implementation architecture.** Correct ownership, contract checking, and dependency invalidation remain requirements. A unified compiler record, an expanded inspection product, a comprehensive API-diff system, or a new registry framework are not prerequisites for the immediate changes.
4. **Keep independent findings independent.** The `Result` contradiction, thread-sharing conditions, counted-copy rules, cleanup-bearing views, borrowed-key APIs, and other findings are not discarded merely because the inference proposal was too broad. Each gets its own scope, compatibility assessment, and tests.

**A narrower adoption set is not a reduction of Ember's capabilities.** Associated types, nominal interfaces, blanket implementations, const generics, explicit ownership modes, multi-region views, classes, generators, arenas, SoA, SIMD, ECS facilities, C/C++ interoperation, and hot reload are not removed by this document.

### What this revision does not claim

The supplied pushback reports that the compiler already distinguishes associated-type defaults from equalities and already uses IEEE float operators in generic code. Those are **reported implementation facts**, not independently verified results of this revision. Where the implementation already matches the recommended rule, the work may be specification alignment and regression tests rather than a new compiler feature.

No Ember compiler, standard-library implementation, or benchmark suite was executed for this revision. Examples are illustrative proposed examples unless stated otherwise. No example is a claim of a passing compiler test.

---

## 2. Evidence and adoption conventions

### Source labels

| Label | Source | Use |
|---|---|---|
| **H23** | `Ember_v0.9.9_Hardened_23.md` | Baseline language rules, section names, diagnostics, and implementation requirements. |
| **GSP** | `Ember_Generic_Simplicity_Pass.md` | The original generic simplification ideas. |
| **C1** | `Ember_Simplification_Pass_Consolidated.md` | The earlier thirty-one-item review proposal. |
| **P** | The user's three-part pushback in this conversation | Package-boundary inference, the smaller float fix, and separation of tooling/process. |
| **R** | The subsequent analysis accepting most of that pushback | The revised dispositions implemented in this document. |

References use filenames, rule identifiers, and sections rather than conversation-only links, so the document remains useful when copied into the project. Source byte identities are recorded in Appendix C. No additional external research is introduced in this revision.

A **baseline finding** describes supplied text. A **proposed rule** is a recommendation to adopt, not a statement that H23 already says it. A **safety concern** is reasoning from the written rules, not proof of an implementation bug. A **review gate** identifies a dependency or unresolved choice that must be settled before the corresponding behavior change is adopted.

### Disposition labels

| Label | Meaning |
|---|---|
| **Repair / clarify now** | A focused correction or semantic guarantee worth addressing without waiting for body-bound inference. Adoption still requires tests and the normal language-change process. |
| **Review independently** | A useful candidate that changes an API, lifetime policy, accepted programs, or results. It is not automatically approved by adopting the other repairs. |
| **Later: private inference** | A bounded enhancement to package-private helpers. Implement after the foundations, not as a dependency of the immediate work. |
| **Engineering backlog** | Optional organization, reporting, or workflow improvement. Its value should be demonstrated against existing implementation machinery. |
| **Split** | Retain the semantic requirement, but move the proposed architectural remedy or expanded tool to the backlog. |

These labels replace C1's broad P0/P1/P2 rollout. There is no requirement to implement every entry before shipping a useful revision.

`SP-001` through `SP-031` remain **review IDs**, not normative language-rule IDs. Do not reuse existing diagnostic numbers for new meanings. New source syntax or library names are explicitly marked as proposed. The language version is not advanced by this proposal document itself.

### Contents

- [3. Revised change register](#register)
- [Part I — Targeted repairs and semantic guarantees](#part-i)
- [Part II — Independently reviewable simplifications](#part-ii)
- [Part III — Later package-private bound inference](#part-iii)
- [Part IV — Optional engineering work](#part-iv)
- [4. Adoption sequence and migration](#adoption)
- [5. Focused conformance matrix](#tests)
- [Appendix A — Disposition of the original Generic Simplicity Pass](#appendix-a)
- [Appendix B — Preserved choices and withdrawn mechanisms](#appendix-b)
- [Appendix C — Source identities and verification limits](#appendix-c)

<a id="register"></a>
## 3. Revised change register

| ID | Revised subject | Disposition | Change from C1 |
|---|---|---|---|
| SP-001 | Bounded body inference for package-private helpers | Later: private inference | Narrow scope; no public inferred requirements; no ambitious recursive solver initially. |
| SP-002 | Identify nominal operations before inferring requirements | Later: private inference; retain current nominal rules now | Keep the safeguard, not program-wide method-name guessing. |
| SP-003 | Associated defaults are not equality guarantees | Repair / clarify now | Independent of inference; align the spec with the reported compiler behavior after verification. |
| SP-004 | Explicit contracts at the package boundary | Later boundary policy; public discipline retained now | Replace selectable contract modes; withdraw `@bounds(explicit)`. |
| SP-005 | Stable fallback selection | Clarify existing behavior; later inference extension | Keep existing fallbacks; do not infer alternative protocols speculatively. |
| SP-006 | Associated projection identity and limited coercion inference | Clarify existing identities; later inference extension | No new projection syntax or coercion-search system. |
| SP-007 | Concrete const evaluation versus symbolic proof | Review independently | Preserve useful computation; avoid expanding the symbolic solver. |
| SP-008 | Type formation, derives, and implementation contracts | Clarify now; later inference boundaries | No body-derived implementation applicability or exported helper requirements. |
| SP-009 | General `Result`; operation-specific bounds | Repair / clarify now | Retain independently. |
| SP-010 | Counted `Copy`, owned arguments, and access interactions | Repair / clarify now, with a separate access/layout review gate | Do not bundle a class-access redesign into a wording patch. |
| SP-011 | Thread transfer, shared access, and destruction | Repair / clarify now | Prioritize the `RwLock` condition; audit other cases individually. |
| SP-012 | Views can carry cleanup | Repair / clarify now | Correct the model without adding runtime region machinery. |
| SP-013 | One storage-region policy | Review independently | Retain the static-view consistency proposal; no generalized heap borrowing. |
| SP-014 | Observable destruction boundaries | Review independently | Explicit lifetime-policy change, not automatic adoption. |
| SP-015 | Structural safety-critical cleanup restrictions | Repair / clarify now | Retain without broadening legal scope movement. |
| SP-016 | Separate borrowed matching from key construction | Review independently | Narrow library change, independent of inference. |
| SP-017 | Defaults, zero initialization, and numeric identities | Review independently; clarify meanings now | Preserve the distinction; migrate changed arena initialization deliberately. |
| SP-018 | IEEE float operators; total-order algorithms | Repair / clarify now | Replace the `PartialOrd` redesign; keep convenient float sorting and total `min`/`max`. |
| SP-019 | Same selected numeric operation at compile time and runtime | Repair / clarify now | Keep the explicit deterministic-math fix. |
| SP-020 | Effects of machinery, callbacks, and cleanup | Split | Keep correct effect composition; defer elaborate phase reporting. |
| SP-021 | Shared resolved-operation representation | Engineering backlog | No prescribed internal architecture for the immediate pass. |
| SP-022 | Unified semantic-signature inspection | Engineering backlog | Useful later; not required to understand a public API's bounds. |
| SP-023 | Correct invalidation of semantic dependencies | Split | Required invalidation; optional comprehensive API-diff product. |
| SP-024 | Formatting versus semantic refactoring | Repair / clarify now | No bound deletion or contract rewriting in ordinary formatting. |
| SP-025 | Existing borrow-contract and provenance rules | Split | Preserve and serialize relevant facts; defer representation consolidation. |
| SP-026 | Concrete specification drift and registry reuse | Split | Fix contradictions directly; extend rather than replace existing tooling. |
| SP-027 | Semantic inputs, policy, and resource limits | Repair / clarify now | Clarify guarantees without a new configuration framework. |
| SP-028 | Semantic verdicts versus backend decisions | Split | Preserve profile-independent acceptance; defer a broad proof-phase refactor. |
| SP-029 | Versioned library interfaces and layer organization | Engineering backlog with a layer-correctness review | Do not adopt the earlier `Result` alias restructuring automatically. |
| SP-030 | Targeted interaction tests and teaching | Extend existing practice | Tests follow adopted changes; no parallel conformance bureaucracy. |
| SP-031 | Checked mutable access to two runtime-selected elements | Review independently | Retain the focused library API; not a compiler-inference dependency. |

---

<a id="part-i"></a>
# Part I — Targeted repairs and semantic guarantees

These entries can be addressed without public body-derived bounds, a new comparison interface, or a wholesale compiler redesign. Several still contain real semantic changes and must be recorded as such.

## SP-003 — Associated-type defaults do not create universal equalities

**Disposition:** Repair / clarify now.  
**Sources:** H23 §III.2 `assoc_type`, `[IFC-4]`, §IV.7; GSP §§8–10; C1 SP-003; P §1.

### Baseline finding

GSP combines two claims: an interface can default `Output` to `Self`, and a bare `T: Add` normally guarantees `Output = T`. An overridable implementation default cannot supply that universal guarantee. H23's grammar admits an associated-type default, while the interface semantics need a complete account of it.

### Proposed rule

> An associated-type default supplies an omitted associated binding in an implementation. A generic bound acquires only the associated equalities it states or that follow from its declared requirements. An overridable default is not such an equality.

For an interface with a default `Output = Self`:

| Form | Meaning |
|---|---|
| An implementation omits `Output` | Its default is resolved and checked against the associated type's bounds. |
| An implementation specifies another `Output` | The explicit binding is used if valid. |
| `T: Add` | The operation returns the interface's associated output; it is not necessarily `T`. |
| `T: Add[Output = T]` | The result-type equality is part of the contract. |

Default method bodies must not assume an overridable associated default. Diagnose cycles and invalid defaults. Do not add a second omission policy for `dyn` types in this pass.

The rule is independent of whether std adds defaults to more operator interfaces. Adding such defaults is an implementation-author convenience, not a prerequisite for this correction and not a replacement for public associated-output constraints.

**Implementation note:** P reports that the current compiler already observes this distinction. Verify that behavior, then update the normative prose and regression cases; do not describe this as an unimplemented inference feature merely because C1 grouped it with inference.

**Tests:** Default used, default overridden, non-self result, explicit result equality, inherited associated member, invalid/default cycle, and a default method that incorrectly assumes the default cannot be overridden.

## SP-009 — Make `Result` a general success/failure container

**Disposition:** Repair / clarify now.  
**Sources:** H23 `[ERR-1]`, `[ERR-2]`, `[ERR-7]`–`[ERR-12]`, `[FN-8]`, `[STD-15]`, `[DSJ-1]`; C1 SP-009.

### Baseline finding

H23 requires `E: Error` for `Result[T, E = AnyError]`, but explicitly excludes `AnyError` from `Error`. Other specified APIs use non-error failure payloads, including `Result[int, int]` and the views returned on a failed disjointness check.

### Proposed rule

Remove the universal `E: Error` formation bound. `Result[T, E]` is an ordinary enum with success and failure payloads. Put requirements on the operations that need them.

| Operation | Requirement |
|---|---|
| Construct, move, match, or transform a payload | Only the requirements of the payload operation or callback. |
| Propagate an unchanged failure with `?` | No `Error` requirement; transfer the failure payload as specified. |
| Convert `E` to `F` with `?` | The existing `F: From[E]` relationship. |
| Format the failure | The formatting capability the API uses. |
| Traverse a source chain | `Error`, or the specified inherent reporting operation of `AnyError`. |
| Box a general error into `AnyError` | Its existing conversion requirement, including `E: Error` for the blanket error-boxing conversion. |

Keep `AnyError` outside `Error` under the existing `From` design. Do not introduce overlapping conversions or specialization. Propagating one `AnyError` into another remains an identity transfer, not another box allocation.

Express `context` and `with_context` in terms of the conversion they actually need. Do not require `Error` on every `Result` solely to support those methods. Similarly, an `unwrap` that formats an error needs the relevant formatting bound; the container does not.

### Entry-point reporting

Reconcile the `E: Display` permission in `[FN-8]` with the source-chain wording in `[ERR-12]`. Preserve the earlier recommendation as a **specific entry-point adapter rule**: a concrete `AnyError` uses its inherent reporting chain; another concrete `Error` uses `source`; a remaining displayable failure prints its message without a source chain. This is not a generalized overload-priority mechanism or a new `Error` implementation for `AnyError`.

The adapter's exact lowering must be specified with the entry point and tested. Ordinary generic functions still cannot assume that a `Display` bound supplies `source`.

**Compatibility:** This removes a contradictory restriction. New method-specific diagnostics may replace type-formation failures. Keep the default error convenience in allocation-enabled use; core-layer organization is separately reviewed under SP-029.

**Tests:** `Result[T]`, non-`Error` failure payloads, same-error and converting `?`, no repeated boxing, error-specific methods, and all three entry-point reporting cases.

## SP-010 — Give counted copies and owned arguments a compositional meaning

**Disposition:** Repair / clarify now; access/layout changes require a separate review gate.  
**Sources:** H23 `[PHIL-3]`, `[OWN-7]`, `[STR-3]`, `[DRV-1]`, `[FN-1]`, `[FN-9]`, `[RC-1]`–`[RC-3]`, `[HEAP-6]`, `[WK-11]`, `[EXC-17]`, `[EXC-19]`; C1 SP-010.

### Baseline finding

The universal bitwise-copy/no-drop descriptions conflict with counted handles and with aggregates containing them. H23 also describes an owned handle argument as copied without a retain while the source-level `Copy` rule leaves the source usable.

### Proposed rule

> A `Copy` value may be implicitly duplicated while its source remains usable. Duplication performs the type's specified copy operation, including required ownership bookkeeping.

Keep these properties distinct, whether or not the implementation stores them in one table:

| Property | Example |
|---|---|
| Can be implicitly duplicated | A scalar or a counted class handle. |
| Can be duplicated trivially, without ownership bookkeeping | A plain scalar; not a counted handle in the general case. |
| Requires cleanup | A handle release, an aggregate containing owners, or a user destructor. |
| Carries borrows | A span, reference, or borrowing aggregate. |

Aggregate duplication applies the component operations to the live components; cleanup releases the owners that aggregate contains. A user-defined destructor still prevents `Copy` under the stated derive rules. Generated retain/release cleanup is not treated as proof that every aggregate containing a handle must be move-only.

Do not add arbitrary user-defined implicit copy hooks. Custom duplication remains explicit through `Clone`.

### Owned parameter rule

An owned non-`Copy` argument transfers ownership. An owned argument taken from a `Copy` lvalue that remains usable supplies a duplicate, including any necessary retain. A newly produced owned temporary can transfer its existing ownership. Proven counting elisions remain valid.

The unsupported general combination is: source remains usable, callee owns and releases a second owner, and no extra ownership count or valid transfer/elision accounts for that owner. Remove blanket no-retain wording that implies this combination.

### Separate access and layout review

Do not infer from `Copy` that every access is non-reentrant or that a reference into the value needs no protection. Releasing an overwritten counted component can invoke a destructor. H23's unchecked `Copy`-field access rules must therefore be reviewed alongside its ordinary-reference and provenance guarantees.

This revision **does not silently approve adding access words to every class field**. Preserve the safety issue as a separate review gate: establish the reference and reentrancy invariant, show a concrete accepted/rejected case, choose a compatible enforcement strategy, and review any layout/runtime impact. A proof or the existing field-access mechanism is preferable to inventing a hidden heap side table, but this document does not assert that one local wording edit completes that review.

**Tests:** Scalars, strong and weak handles, tuples and fixed arrays of handles, derived aggregates, counted `Cell` payloads, owned calls followed by source use, overwrites with reentrant destructors, and address-taken class fields. Validate ownership counts and cleanup order, not just final values.

## SP-011 — Separate transfer, shared access, and destruction requirements

**Disposition:** Repair / clarify now; resolve each capability case on its own evidence.  
**Sources:** H23 §IV.8 marker table, `[THR-8]`, `[THR-9]`, `[THR-1]`, `[THR-3]`, `[THR-15]`, `[HEAP-10]`, `[CELL-3]`; C1 SP-011.

### Immediate correction: shared readers need shareable payloads

`Mutex[T]` and `RwLock[T]` must not have the same `Sync` condition merely because both are locks. A mutex exposes one protected access at a time; a read lock can expose several shared accesses simultaneously.

| Capability | Proposed requirement |
|---|---|
| `Mutex[T]: Send` | `T: Send`. |
| `Mutex[T]: Sync` | `T: Send`. |
| `RwLock[T]: Send` | `T: Send`. |
| `RwLock[T]: Sync` | `T: Send + Sync`. |

The written rules otherwise admit the concerning combination `RwLock[Cell[int]]`: `Cell[int]` can be transferred, but shared readers can mutate it without the read lock excluding each other. Require `Sync` for the payload's simultaneous shared access.

### Other findings retained for the same capability audit

Reconcile the summary marker table with `[THR-8]`'s conditional transfer of references and spans. Preserve the existing conservative rule about mutable references not themselves being `Sync`; this pass does not change it to imitate another language.

For `SyncShared[T]` and its transferable weak form, the proposed shared-ownership condition is `T: Send + Sync`: shared access must be valid, and destruction of the payload must be valid on the thread performing final release. For `@sync` classes, the field and destruction obligations must similarly justify both transfer and shared access. A channel that transfers payloads must retain the payload's `Send` requirement and the receiver's single-consumer restrictions.

These are access/destruction requirements, not proof that every corresponding compiler defect exists. Compare the actual std declarations and compiler capability rules before changing implementation behavior.

### Preserve provenance and foreign restrictions

A view into a thread-confined owner's field must not lose restrictions relevant to conflicting mutation or keeping its owner alive simply because the field's scalar type is `Sync`. Either the existing borrow/access analysis establishes safe transfer, or the transfer is rejected. No new lifetime syntax is needed.

A foreign owner confined to a thread must retain that restriction through wrappers and containers. Debug checks can diagnose violations, but cannot be the only protection for an all-profile Safe guarantee.

**Tests:** Accept appropriate exclusive use of `Mutex[Cell[int]]`; reject sharing `RwLock[Cell[int]]`; exercise valid shared readers, last-release destruction, payload transfer through channels, affine foreign wrappers, and views obtained from thread-confined class fields.

## SP-012 — Define views by borrows, not by absence of cleanup

**Disposition:** Repair / clarify now.  
**Sources:** H23 §IV.1, `[TYP-34]`, `[CELL-7]`, `[THR-3]`, `[DRP-6]`, `[DRP-7]`; C1 SP-012.

H23 calls views destructor-free, but also describes lock and cell guards as views with release/unlock cleanup. Replace the universal no-cleanup statement with a compositional account.

A plain span carries a region and no value destructor. A mutex guard carries a region and unlock cleanup. A borrowing aggregate may carry both regions and component cleanup. These are independent properties, not exceptions requiring a new category for each library type.

Preserve the distinction between a loan's end and a value's destruction. Compiler-managed access cleanup remains attached to the loan where H23 requires it; a guard's destructor runs at its specified ownership boundary. Any destructor that reads borrowed data requires the relevant region to remain valid until that read.

Do not add runtime region objects, relax heap-storage rules, or classify every lock guard as safety-critical `@must_drop`. A leak that can deadlock and a leak that can let borrowed storage die before a task finishes are different guarantees.

**Tests:** No-cleanup spans, cleanup-bearing guards, guards inside aggregates, early explicit drop, moves, and destructors that read borrowed fields. Sources must outlive their actual cleanup obligations.

## SP-015 — Do not hide safety-critical cleanup inside wrappers

**Disposition:** Repair / clarify now.  
**Sources:** H23 `[THR-5]`, `[THR-6]`, `[CORO-13]`, `[OWN-6]`; C1 SP-015.

The existing restrictions on scope owners must remain effective through ownership composition. A scope obligation cannot disappear because a value is put inside an enum, tuple, aggregate, or generic wrapper.

Where such composition is otherwise legal, propagate the safety-critical cleanup property through owned components and use it when checking forgetting, storage, escape, return, and suspension. A borrowed reference to a scope is not ownership of the scope's join obligation; ordinary region rules govern the reference.

This is **not permission to move a `Scope` that H23 already makes immovable**. An attempt to wrap one can be rejected at the move itself. Structural tracking closes otherwise available routes; it does not first legalize forbidden operations merely so a later check can reject them.

Keep the existing `@must_drop` and `with` forms. No general linear-type syntax, new annotation, or arbitrary user-destructor proof system is introduced.

**Tests:** Direct violations and every representable nested route; attempted scope moves; returns, captures, `?`, and `yield`; normal and early exits that join before borrowed storage ends. A wrapper with a custom destructor must not erase the enforced obligation.

## SP-018 — Keep float operators IEEE and keep total-order algorithms convenient

**Disposition:** Repair / clarify now; replaces C1's comparison-system redesign.  
**Sources:** H23 `[TYP-9]`, `[TYP-36]`, `[TYP-37]`, the `Ord` interface sketch, §VI.3, `[DRV-1]`, `[STD-15]`, `[STD-8]`, `[STD-19]`, `[STD-27]`, `[MONO-5]`–`[MONO-9]`; C1 SP-018; P §2; R §2.

### Revised decision

Keep the existing `Eq` and `Ord` interfaces. Do not introduce `PartialOrd`, remove floats from `Ord`, require a float wrapper, or require rewriting `xs.sort()` to `sort_by(math.total_cmp)`.

Adopt this policy:

> **For floating-point operands, `==`, `!=`, `<`, `<=`, `>`, and `>=` retain their specified IEEE meanings in concrete and generic code alike. A generic `Ord` bound permits the relational operation but does not change a float operator into a total-order comparison. An explicit `cmp` call, default sorting, and the standard library's specified total-order consumers use `Ord.cmp`.**

The guarantee compares uses with the same operand type and the same applicable numeric policy. It does not cancel an explicitly requested `@fastmath` relaxation or change the comparison policy of an unrelated user-defined type.

### Public behavior

| Use | Required relation |
|---|---|
| Relational operator on bare floats, whether concrete or generic | IEEE numeric comparison. |
| Float `==` / `!=` | Existing IEEE equality/inequality. |
| Float `cmp` | Existing IEEE totalOrder-based comparison. |
| Default `sort`, `sorted`, and default ordered search | `Ord.cmp` consistently. |
| Existing total-order `min` and `max` | `Ord.cmp`; do not bundle a new NaN policy into this repair. |
| Comparator-taking algorithms | The supplied comparator throughout that algorithm. |
| Equality-based membership or search | The documented equality operation, not a hidden total-order comparison. |
| Non-scalar user-defined comparisons | Their declared language/protocol behavior, preserved across concrete and generic uses. |

The generic function below must use IEEE `<` when instantiated for floats:

```ember
# Illustrative contract and body; not a compiled test result.
pub fn less[T: Ord](a: T, b: T) -> bool:
    return a < b
```

An explicit `a.cmp(b)` is intentionally a different operation. Keep the type's ordinary nominal capability checking; do not introduce unrestricted structural comparison inference.

### The deliberate tradeoff

The relations need not agree for floats. Under the revised policy:

```text
-0.0 == +0.0                 true
-0.0 < +0.0                  false
(-0.0).cmp(+0.0)             Less
```

A NaN is unequal to itself under IEEE equality, while comparing the same supported NaN representation with itself under the total comparator has the comparator's equality result. Relational IEEE comparisons involving an unordered operand are false; do not implement `<=` as `not (>)` for floats.

Accordingly, `Ord: Eq` means that the type supplies both operations; it does not establish a universal equivalence between `cmp == Equal` and `==` for all Ember types. Do not use such an equivalence as an optimizer assumption or generic algorithm law when the documented float implementation is admitted. This consequence already matters for H23's mixed IEEE-equality/total-order design; the repair makes it explicit rather than adding a new interface to eliminate it.

### Algorithms must choose one relation completely

A total-order algorithm must use its comparator for ordering decisions **and comparator equivalence**. It must not navigate using `cmp` and then test a match with `==`, or advance through lexicographic fields using an IEEE equality shortcut.

The companion library audit must cover:

- Default sort, key-based sort, and ordered search using compatible ordering throughout.
- Derived and aggregate `cmp`, including field progression based on `cmp == Equal`.
- Default scalar and iterator `min`/`max`, retaining their intended total-order policy.
- Equality-based membership, `index_of`, and any equality-based `dedup` behavior, kept separate from comparator equivalence.

For illustration, an array containing only `+0.0` can match `-0.0` under IEEE-equality membership but have no total-order-equivalent `-0.0` entry for binary search. That is an intentional consequence of those two API contracts, not evidence of an implementation test run.

The supplied prose does not determine every tie policy, duplicate-match index, or full std signature. Consult the normative `std/**/*.em` for those details and preserve an existing specified choice. Do not silently invent a new tie/NaN policy as part of this revision. Where neither source specifies a necessary detail, record and decide that detail before adopting the affected API rule.

### Required patch locations

Update `[TYP-37]`, but also update the `Ord` sketch's comment, §VI.3's operator mapping, relevant generic numeric descriptions, and algorithm documentation. No remaining table may say that a generic float relational operator necessarily lowers through the total `Ord.cmp`.

The rule is one semantic policy, not literally a one-line editing task.

### Shared generics and lowering

The compiler must preserve the distinction between an ordinary relational operation and `cmp` when selecting intrinsics and when sharing generic bodies. A total `cmp` result alone does not encode IEEE unordered comparison behavior.

Do not re-run overload/fallback selection at monomorphization. Resolve the semantic operation first, then implement that operation correctly for its operand type. Optional body sharing may retain specialization when its existing representation cannot preserve the operation. A forced-sharing directive must follow the stated shareability/support rules; it cannot authorize a changed comparison result. No new public comparison protocol or unreviewed runtime ABI is mandated here.

**Compatibility:** P reports that the compiler already behaves this way for generic operators. Verify that claim. If true, this chiefly aligns conflicting prose; otherwise generic IEEE cases change relative to H23's total-comparison sentence and need a language-change entry. Existing float `sort`, total `min`/`max`, and `Float`'s capabilities are retained.

**Tests:** Signed zeros in both orders, finite values, infinities, quiet NaNs and supported total-order NaN representations; all six operators in concrete and explicit-generic code; default sort followed by binary search; comparator equality versus IEEE membership; derived lexicographic comparison; floating range types; optimized/unoptimized and specialized/shared lowering. Later private inference must pass the same cases.

## SP-019 — Do not substitute a different numeric function at compile time

**Disposition:** Repair / clarify now.  
**Sources:** H23 `[CT-1]`, `[CT-4]`, `[DET-2]`, `[DET-4]`, Part XIV's sine-table example, `[STD-27]`; C1 SP-019.

H23 excludes nondeterministic platform math from compile-time evaluation, but describes the evaluator as using deterministic math and shows a compile-time example importing ordinary `sin`. Clarify which source operation is actually selected.

The recommended narrow rule is: code requiring deterministic compile-time transcendental computation names `std.math.det` in source. The evaluator executes that selected operation with the target's semantics. It does not silently replace an ordinary platform operation with a different deterministic implementation because the call happens at compile time.

Update the sine-table example to call `det.sin`. Retain explicit `comptime.read_file` and `comptime.env` inputs and their dependency recording. Do not create a new compile-time dialect or relax the existing effect restrictions.

**Compatibility:** Examples or code relying on an implicit substitution need an explicit deterministic call. This is a semantic clarification/change, not permission to silently alter runtime platform math.

**Tests:** The same deterministic helper evaluated at compile time and runtime, target arithmetic and failure equivalence, explicit rejection of inadmissible platform operations, and invalidation for recorded compile-time inputs.

## SP-020 — Keep full effect composition; defer richer reporting

**Disposition:** Split: semantic correction now, reporting architecture later.  
**Sources:** H23 `[EFF-1]`–`[EFF-6]`, `[EFF-15]`, `[EFF-16]`, `[PHIL-11]`, `[STD-18]`, `[STD-19]`, `[CELL-1]`, `[CELL-2]`, `[CORO-1]`, `[CORO-5]`, `[CORO-7]`, `[CORO-8]`; C1 SP-020; P §3.

### Keep now

An operation's full effect set includes its machinery, the selected callbacks/interface methods it executes, copy/drop bookkeeping, and argument/default-expression evaluation. H23 already requires transitive effects; apply that requirement consistently instead of presenting it as an entirely new effect system.

“`format_to` needs no allocation for its buffering machinery” does not prove that a user `Display` implementation called by it cannot allocate. Likewise, a lazy adapter can allocate no frame/storage of its own while its later callback allocates, and reading a counted payload from a cell can require a retain.

Where execution is already split into phases, count effects at their actual execution sites: generator construction, resumption, and destruction are not the same event. Preserve the existing conservative whole-generator contract scope unless a separately reviewed rule changes it. Do not infer an unconditional guarantee from only the concrete instantiations observed so far.

Narrow absolute “cannot abort in any way” wording to what the modeled effects establish. Keep the resource-exhaustion, foreign-code, and hardware limitations already listed by `[PHIL-11]`.

### Defer

A unified symbolic-effects product, detailed per-phase inspection views, and callback/job storage refactors are engineering work to pursue as needed. They are not prerequisites for fixing an incorrect `@noalloc` verdict or an overbroad cost statement.

**Tests:** Allocating formatter callbacks, counted cell payloads, deferred iterator effects, a never-resumed generator with cleanup, and existing interface/callable effect contracts. No claim of “zero overhead” may ignore user code or required cleanup.

## SP-023 — Preserve correct dependency invalidation without requiring a new API product

**Disposition:** Split: correctness requirement now; comprehensive API diffs later.  
**Sources:** H23 `[BLD-2]`, `[BLD-8]`, `[EFF-4]`, `[LT-22]`, `[LT-35]`, `[CG-C-3b]`; C1 SP-023; P §1 and §3.

### Keep now

A body edit that changes a semantic fact used by a caller must cause that caller to be checked again. H23 already exposes effects and some body-derived borrow/access summaries. Explicit public generic bounds do not eliminate those dependencies.

Keep three purposes distinct without mandating three new data structures:

| Changed fact | Required consequence |
|---|---|
| Caller-visible type, effect, borrow, or access contract | Recheck affected callers. |
| Generic body or inline body needed for code generation | Regenerate affected code even when caller type checking is unchanged. |
| Diagnostic/report-only detail | Do not automatically call it an API break. |

When private inference is introduced, changed helper requirements must invalidate same-package callers, including callers in other modules and public wrapper bodies. A public wrapper's contract is checked, not silently enlarged.

An existing dependency mechanism or conservative rechecking can satisfy correctness. Finer hashes are an optimization, subject to the existing build-time gates; they are not the only permitted architecture. Amend blanket “body edit checks only that body” wording where callers consume changed facts.

### Defer

A comprehensive baseline/diff command, stable JSON contract schema, and release-policy automation are optional engineering work. Ordinary public generic requirements should remain readable in source without those products.

**Tests:** Body-only changes to effects, borrow provenance, and inline bodies; later helper-bound changes across modules; correct rechecks on warm incremental builds; no stale semantic success after a dependency changed. Compare incremental outcomes against clean builds.

## SP-024 — Ordinary formatting must not change contracts

**Disposition:** Repair / clarify now.  
**Sources:** GSP §26; H23 `[FMT-1]`–`[FMT-3]`, `[CLI-20]`; C1 SP-024.

Ordinary `ember fmt` remains presentation-only. It must not delete bounds, add bounds, change ownership/capture policy, insert clones, choose another ordering, or rewrite borrow relationships.

Public constraints are part of the published source contract. A later private inferred constraint is still not a license to delete an intentional explicit requirement. Do not warn by default that all repeated or inferable bounds are mistakes.

Deliberate refactorings can materialize helper bounds, make a helper public with the necessary declaration changes, or migrate an approved language change. Such actions must show semantic consequences. Where adding a local binding around a consumed callback requires `owned fn` to preserve capture behavior, the refactoring must preserve that behavior.

Use the existing explicit migration route where applicable. There is no need for a new command merely to draw the boundary between formatting and migration. The withdrawn fixed-bound attribute must not be inserted by either path.

**Tests:** Formatting idempotence and unchanged signatures/selected operations; a helper's explicit stronger bound; callback extraction; public promotion; migration cases requiring human intent.

## SP-025 — Preserve existing borrow contracts and relevant summaries

**Disposition:** Split: existing semantic obligations retained; consolidation optional.  
**Sources:** H23 `[LT-1]`, `[LT-1a]`, `[LT-22]`, `[LT-23]`, `[LT-35]`, `[BRW-8]`, `[BRW-10]`, ODR-048; C1 SP-025.

Keep declaration-based result elision, explicit `@borrows`, multi-region result provenance, and private-method field precision in their established roles. The package boundary used for generic bounds does not silently rewrite all existing borrow-summary rules.

In particular, retain `[BRW-10]`'s package-private precision and conservative public ordinary receiver behavior. Preserve the public multi-region summaries H23 already exposes, recording and invalidating them as necessary. Do not replace every public borrow contract with the shortest body-derived one or collapse independent result regions.

Retain ODR-048's declaration-level parameter passing. Evaluating a declared source-parameter rule for an instance whose type becomes a view is not re-selecting that rule from the instantiated body.

A common internal call-contract representation may be helpful, but is not a new language requirement or a prerequisite for another targeted repair.

**Tests:** Generic `Holder[str]`-style cases, independent result fields, `@borrows`, opaque calls, private-to-public method changes, and incremental caller checks. No named lifetimes are introduced.

## SP-026 — Fix concrete specification drift; reuse the existing checks

**Disposition:** Split: direct corrections now; generalized registry generation optional.  
**Sources:** H23 `[LEX-2]`, `[FMT-1]`, `[MAN-8]`, `[RT-10]`, `[WK-12]`, `[TST-4]`, `[DIA-6]`, `[DIA-6a]`, §V.8 and §XVII.9; C1 SP-026; P §3.

H23 already requires a rule index, diagnostic registry, error pages, and conformance checks. Do not describe those as missing infrastructure or require parallel replacements.

Apply the concrete corrections directly:

| Finding | Revised action |
|---|---|
| `[LEX-2]` describes configurable CRLF output while `[FMT-1]` requires LF and the manifest surface omits that option. | Prefer the existing always-LF formatter policy; accept both LF and CRLF input. Remove the stale option promise rather than invent another setting. Check actual supported manifests before migration. |
| `[RT-10]` describes weak upgrade as potentially starting from zero, while `[WK-12]` requires failure at zero. | Make the runtime description agree with non-resurrection: upgrade must not acquire a strong reference from a zero strong count. |
| The status paragraph points implementation/conformance readers to the diagnostic registry. | Correct the cross-reference to the actual implementation-matrix and conformance sections. |
| An attribute, callable contract, command, or special call form is mentioned outside its authoritative surface description. | Check the existing grammar, table, registry, and implementation. Add the missing specified entry or state the limitation; never accept syntax while ignoring its meaning. |

Retain the earlier follow-up checks for user attributes, error/serialization field annotations, callable-type contracts, the trusted `@assume_noalloc` form, and compiler-known call forms. They are checks of identified inconsistencies, not permission to broaden the grammar by analogy.

A structured registry for every language surface is optional. Extend current scripts where that is enough. A concrete one-line conflict does not have to wait for that larger project.

**Tests:** Existing registry/error-page checks plus focused cases for each edited surface. Confirm weak upgrade after final release fails; no profile-specific resurrection behavior. Report implementation support honestly.

## SP-027 — Distinguish semantic inputs, build policy, and resource exhaustion

**Disposition:** Repair / clarify now.  
**Sources:** H23 `[PHIL-13]`, `[MAN-8]`, `[PRF-1]`, `[CT-3]`, `[EFF-19]`, `[STD-6]`; C1 SP-027.

Keep the guarantee that optimization profiles do not change a program's language semantics or disable required safety checks. Clarify the overly broad claim that no configuration can affect acceptance in any sense.

A language revision, target layout, resolved dependency, enabled library layer, warning-as-error policy, and compile-time work limit are not interchangeable inputs. A missing feature, failed build policy, exhausted budget, and language error should not be described as the same failure.

`@realtime` is expanded using the declaring package's existing manifest policy. Preserve that meaning when another package imports the function; the caller must not reinterpret the callee's annotation under its own policy. Relevant changes participate in dependency checking.

No new setting categories need to become source syntax, and no new configuration subsystem is prescribed. This is a correction to the guarantees and failure descriptions, not a permission to weaken them.

**Tests:** Identical semantic inputs across profiles; warning-policy failure; evaluator budget exhaustion; unavailable library layer; changed declaring-package realtime expansion; no option that bypasses alias, region, bounds, or lifetime enforcement.

## SP-028 — Semantic acceptance must not depend on host optimization decisions

**Disposition:** Split: semantic guarantee now; proof-architecture refactor optional.  
**Sources:** H23 `[EFF-15]`, `[PAR-2b]`, `[SIMD-3]`, `[SIMD-5]`, `[CG-C-3]`, `[CG-C-3a]`, `[MONO-3]`–`[MONO-9]`; C1 SP-028.

Clarify “inlined” where it is a precondition for semantic analysis. A source-level proof based on specified call expansion or summaries is not the same as the host C compiler deciding to inline emitted code.

A host optimizer's decision must not retroactively make a borrow, effect, parallel-independence, or vectorizable-form judgment succeed or fail. Keep the current mandatory lowering guarantees and the distinction between Ember's verdict and the host vectorization report.

If a recursive or opaque call cannot be handled by the specified proof rule, use the rule's documented conservative behavior. Do not make acceptance depend on unlimited expansion or an undocumented heuristic budget. Defining any missing expansion boundary is a targeted rule decision, not a requirement for a new global compiler architecture.

Generic sharing happens after semantic operations and contracts are established. A sharing threshold, hotness heuristic, or inlining hint cannot alter comparison policy, selected fallback, parameter mode, or result provenance. Retain existing specialization/sharing directive names and explicitly settle their precedence when necessary.

**Tests:** Equal semantic verdicts with optimization off/on, host inlining changed, and sharing heuristics changed. Preserve the difference between an Ember legality verdict and an implementation's emitted-code report. Include the float comparison cases of SP-018.

---

<a id="part-ii"></a>
# Part II — Independently reviewable simplifications

These are retained proposals, not automatic consequences of accepting Part I. Each should be adopted only with its own compatibility decision. None depends on body-bound inference or a comprehensive tooling overhaul.

## SP-007 — Keep const computation useful and symbolic proof requirements small

**Disposition:** Review independently; no new general type-level solver.  
**Sources:** H23 §IV.7, §XIV.1, `[PHIL-13]`; GSP §19; C1 SP-007.

Distinguish evaluating an expression with concrete compile-time inputs from proving a symbolic type equality for every possible generic argument.

Preserve ordinary deterministic compile-time functions, loops, and calculations that produce concrete array sizes, capacities, dimensions, and layout constants. Do not arbitrarily restrict const arguments to literals.

For symbolic equality, specify the finite set of supported normalization and proof rules. Substitution of known constants, identity of an expression/parameter, and explicitly admitted arithmetic transformations can be supported without requiring inversion of arbitrary functions or solving nonlinear constraints.

Do not let a backend optimizer's ability to prove an identity determine whether a type checks. For example, treating `N + 1` and `1 + N` as equivalent must follow a stated, target-appropriate semantic rule, not whichever optimization happens to run.

This is not a request to remove existing accepted const-generic functionality without review. First identify the current documented and implemented cases. Unsupported symbolic proofs should receive an actionable annotation or shared-expression suggestion; resource exhaustion must not be mislabeled as proof of inequality.

**Tests:** Concrete computed sizes, symbolic identities already guaranteed by the language, substitution, unsupported symbolic relationships, range/overflow checks for sizes, and deterministic resource-limit diagnostics.

## SP-013 — Use the same storage-region policy for all owning containers

**Disposition:** Review independently; proposed consistency improvement.  
**Sources:** H23 `[TYP-15]`, `[TYP-15a]`, `[TYP-38]`, `[STD-11]`; C1 SP-013.

The general storage rule allows static-region views in unbounded owning storage, while the map/set prose bans all view payloads. Prefer the general rule and remove the unexplained collection-specific prohibition.

> A stored value's regions must outlive its destination. Unbounded owning storage admits only static-region views; region-bounded containers may admit views valid for their own bounded lifetime.

This would allow an explicitly typed map of static text views:

```ember
# Proposed allowed case after the storage-rule consolidation.
labels: Map[str, int] = {"start": 1, "stop": 2}
```

It does not allow a locally borrowed `String` slice to escape into that map. Preserve default owning `String` keys/elements for unannotated map/set literals. Do not infer borrowed literal storage merely to save allocation or extend a local borrow to `static`.

Keep the existing arena-container and `BorrowList` mechanisms. This proposal does not make ordinary heap containers region-polymorphic, remove cleanup restrictions, or permit storage forbidden by `@must_drop`.

**Compatibility:** Primarily adds static-view cases, but compiler representation and nested-region checks must be reviewed. Keep the source's explicit ownership choice.

**Tests:** Static keys and values, non-static rejection, aggregates with several regions, bounded arena cases, unchanged default literal typing, and no implicit cloning or lifetime extension.

## SP-014 — Preserve observable destruction boundaries

**Disposition:** Review independently; explicit lifetime-policy decision.  
**Sources:** H23 `[OWN-2]`, `[DRP-1]`, `[DRP-2]`, `[RC-3]`, `[OPT-1]`, `[PHIL-13]`; C1 SP-014.

C1 recommended a stronger relationship between source ownership and observable destruction. Retain that recommendation, but do not call it a harmless clarification: H23 expressly permits early counted-object deinitialization in some circumstances and points users to `with` or `keep_alive`.

The proposed policy is:

> Ownership defines an observable destruction boundary. Counting optimizations may remove bookkeeping, but cannot move destructor effects, weak-upgrade outcomes, or foreign-resource release across that boundary unless observational equivalence is established.

The last use of a borrow can still end its loan early. That does not itself end the owner's lifetime. `with`, explicit destruction, and foreign lifetime declarations retain their roles. Do not require ordinary Safe resource code to use `keep_alive` solely to prevent an optimizer from changing an observable destructor's timing under the proposed policy.

Check the interaction between SP-010's counted-copy rule and explicit `mem.drop`: the specified ownership-ending operation must not be implemented as merely duplicating a counted handle and dropping the duplicate. Its existing ownership-primitive semantics must be made clear wherever ordinary owned-parameter copying would otherwise obscure them.

**Tradeoff:** Objects may remain alive longer than under aggressive early release. Measure peak live memory and surviving count operations. This proposal promises neither a speedup nor performance neutrality.

**Migration:** Preserve tests showing old observable lifetime behavior, mark the changed policy explicitly, and review users relying on early release. Do not silently rewrite those programs or remove raw-pointer lifetime obligations.

**Tests:** Destructor event order, weak upgrade around the boundary, overwrites, branches, explicit destruction, stack promotion, foreign release, and unchanged non-lexical borrow shortening.

## SP-016 — Separate borrowed-key matching from key construction

**Disposition:** Review independently; targeted library-interface change.  
**Sources:** H23 `[STD-12]`, `[STD-16]`, `[STD-17]`, `[GRM-34]`; C1 SP-016.

`AsKey[K]` currently contains both lookup matching and `to_key`, whose same-key implementation clones. Looking up an existing key should not need the ability to create an owned duplicate.

The retained candidate is to keep matching in `AsKey` and place conversion in a separate **proposed library interface**, `ToKey`:

```ember
# Proposed interface sketch, not a verified std declaration.
interface AsKey[K]: Hash:
    fn is_key(self, key: K) -> bool

interface ToKey[K]: AsKey[K]:
    fn to_key(self) -> K
```

| Use | Required capability |
|---|---|
| `get`, `get_mut`, `contains_key`, `remove`, read indexing, and membership | Matching only: `AsKey[K]`. |
| Mutable indexing of an existing entry, including compound assignment | Matching only, plus the operation's mutable-access requirements. |
| Subscript assignment that may insert a missing key | Key construction: `ToKey[K]`. |
| `insert` or `entry` taking an owned `K` | Ownership of the supplied key; no duplicate-key `Clone` requirement. |

A `K: Eq + Hash` can match itself without `Clone`. Duplicating a borrowed key for insertion requires the appropriate conversion/clone implementation. `str` can match `String` without allocating, while conversion is available when a new owned text key is needed.

Keep conversion conditional on absence: updating an existing key through subscript assignment must not construct a redundant key. The type-level conversion requirement still exists even when a particular runtime lookup happens to find an entry.

Define matching compatibility independently of conversion: borrowed hashing and matching must agree with the stored key's hash/equality relation. Where `ToKey` exists, its result must represent that same logical key. Invalid user implementations remain logical errors subject to the library's memory-safety guarantees.

**Compatibility:** Split affected explicit and blanket implementations normally. Review the actual std definitions and overlap rules; no special priority or specialization escape hatch is allowed. `ToKey` is an optional adopted library API, not a new compiler primitive.

**Tests:** Non-`Clone` key lookup and owned insertion, matching-only custom query types, allocation-free text lookup, existing-key updates, missing-key conversion exactly once, and consistent hashing.

## SP-017 — Separate defaults, valid zero bits, and additive identity

**Disposition:** Clarify meanings now; review arena API/behavior changes independently.  
**Sources:** H23 `[ARN-3]`, `[ARN-11]`, `[ARN-10]`, `[DRV-1]`, `Default`, `[STD-5]`; GSP §§3 and 22; C1 SP-017.

| Concept | Meaning |
|---|---|
| `Default` | The type's chosen default value/initialization operation. |
| `Zeroable` | An all-zero representation is valid for the type. |
| Additive identity | A property of a specified numeric operation, or an initial value supplied by the caller. |

Neither validity of zero bytes nor `Default` establishes the other concept. A type can have a valid zero representation and a deliberate nonzero default.

The retained API recommendation is to make ordinary `alloc_array[T](n)` use `Default` and introduce an explicit **proposed** `alloc_zeroed[T](n)` for zero initialization. Keep `alloc_uninit` for explicitly uninitialized storage. Existing arena no-drop and validity restrictions apply to every form.

This changes H23's current zero-first initialization policy. It is therefore **not enabled automatically by this revised pass**. Review call sites, including examples that currently obtain zeroed structs without deriving `Default`. Migrate known zero-initialization intent to the named zeroed operation; do not silently change produced values.

A bulk-zero optimization of default initialization needs proof of equivalence, including effects and initialization order. `T: Zeroable` alone is not that proof.

For generic accumulation, use an explicit initial value rather than assuming arbitrary `Default` means zero. Preserve the documented numeric identities for the numeric `sum`/`product` APIs; no new numeric identity interface is required just to fix the example.

**Tests:** Zeroable type with nonzero default, effectful default, non-zeroable defaultable type, empty allocations, prohibited cleanup types, and migration of zero-dependent examples.

## SP-031 — Checked access to two runtime-selected mutable elements

**Disposition:** Review independently; focused library ergonomics.  
**Sources:** H23 `[BRW-5]`, `[BRW-11]`, `[SPN-5]`, `[DSJ-1]`–`[DSJ-6]`; C1 SP-031.

Retain a **proposed** `get_pair_mut` operation for suitable contiguous containers:

```text
get_pair_mut(mut self, i: int, j: int)
    -> Option[(ref mut T, ref mut T)]
```

The operation borrows the parent once, checks both indices and that they select different elements, then creates the two references. `None` represents an invalid index or equal indices and leaves the container unchanged. The successful result borrows the receiver.

Do not first create possibly overlapping references and then check them. The audited library primitive establishes validity/disjointness before exposing exclusive references. Reuse existing region/loan enforcement and an ordinary unsafe implementation boundary; do not introduce runtime borrow cells or user-written lifetimes for this API.

Preserve the general rule that independently written `a[i]` and `a[j]` do not become disjoint merely because their values are expected to differ. Keep `swap`, `split_at`, and chunking as the recommended operations when they directly fit the task.

Its own work needs no allocation. For zero-sized elements, distinct logical indices do not imply distinct numerical addresses; derive backend alias facts only where valid.

**Tests:** Both index orders, either index invalid, equal indices, empty container, zero-sized elements, references attempting to outlive the receiver, and reuse after the pair's last use. Confirm arithmetic performed by the caller retains its normal overflow policy.

---

<a id="part-iii"></a>
# Part III — Later package-private bound inference

This is a bounded future enhancement, not the immediate organizing project. Explicit generic bounds remain usable everywhere and remain required at public contract boundaries. These entries replace C1's broad inferred/fixed-mode design.

Implementing Part I or adopting an individual Part II API must not depend on completing this part.

## SP-001 — Infer only supported requirements of package-private helpers

**Disposition:** Later: private inference.  
**Sources:** H23 `[TYP-17]`, `[BRW-10]`, `[MOD-2]`; GSP §§2–5 and 13; C1 SP-001; P §1; R §1.

### Revised rule

> An ordinary generic function not visible outside its package may omit requirements derivable by the specified bounded procedure from its declared types, written bounds, and nominally identified operations. Its package-local callers are checked against the resulting contract. A public declaration does not acquire additional body-derived bounds.

This preserves the useful helper case while making the publication boundary readable in source. It is not a promise to infer every relationship that could be considered unambiguous by an arbitrarily powerful solver.

Initial scope: nominal interface obligations, associated-type bindings and equalities, declared superinterface consequences, well-formedness obligations already supported by the type checker, and requirements propagated from resolved callees. Do not infer disjunctions, negative bounds, arbitrary coercion paths, or structural interfaces from method names.

Illustrative private helper after adoption:

```ember
fn accumulate[T](xs: Span[T], owned initial: T) -> T:
    total = initial
    for x in xs:
        total = total + x
    return total
```

The proposed canonical bound is `T: Add[Output = T]`. The owned initial value avoids inventing `Clone`, `Copy`, or `Default` solely to initialize the accumulator. This example uses plain addition, not `+=` fallback inference.

### Initial recursion boundary

Start with requirements propagated across acyclic helper dependencies. For a recursive dependency group, use explicit generic contracts as the initial supported boundary: the bodies are checked against those contracts, rather than recursively growing their requirements.

This is a deliberate initial design restriction, not a claim that recursion makes inference impossible. A later bounded fixed-point implementation can support more cases after its acceptance rules and costs are reviewed. It is not required for the first helper-inference release.

An explicit recursive contract does not waive infinite monomorphization, invalid type recursion, or resource checks. Never report resource exhaustion as a successful proof or as an unrelated missing-interface error.

### Ownership and cost boundary

Do not change parameter modes, insert clones, choose a heap representation, synchronize a value, or change callback capture policy to make inference succeed. An ownership error is not permission to infer different ownership semantics.

The complete local requirement set must be available to diagnostics and callers. That does not require the comprehensive inspection product proposed in C1. A bounded requirement chain in the existing error/hover machinery is sufficient initially.

**Tests:** Direct anchored operations, resolved helper calls across modules, explicit-plus-inferred private requirements, supported associated equalities, recursive groups requiring explicit contracts, and deterministic output across declaration ordering and build profiles. Measure compiler cost before expanding scope.

## SP-002 — Identify the nominal abstraction before deriving its requirements

**Disposition:** Later inference safeguard; existing nominal rules remain unchanged now.  
**Sources:** H23 `[TYP-40]`, `[TYP-24]`, §IV.11; GSP §§4 and 15; C1 SP-002.

Valid anchors include a fixed operator protocol, an explicit bound, a qualified interface call, a resolved free function, or a known nominal receiver resolved by existing rules.

For an unbounded `x.finish()`, do not search the program for whichever interface happens to have `finish`. Give a diagnostic requesting an explicit abstraction or qualified call. Visible candidates may inform the diagnostic, but are not semantic guesses.

The same applies to `T.default()` and `T.Real`: familiar spelling alone does not identify a nominal owner. Keep the convenience that a method on a known implementing type does not need an interface import; that is a different problem from discovering a bound on an unconstrained type variable.

**Tests:** Add an unrelated same-named interface, reorder imports, introduce an inherent method on an unrelated type, and compare the inferred contract. Inference must not gain requirements from unrelated declarations.

## SP-004 — Use the package boundary instead of a second contract mode

**Disposition:** Retain public explicit discipline now; apply this boundary when private inference is adopted.  
**Sources:** H23 `[MOD-2]`, `[BRW-10]`, `[TYP-17]`, `[IFC-3]`; GSP §§6–7 and 25; C1 SP-004; P §1; R §1.

### Declaration policy

| Declaration | Requirement policy |
|---|---|
| Private ordinary generic function | May infer the supported missing bounds after helper inference is adopted. |
| `pub(package)` ordinary generic function | Same package-local inference policy. |
| Generic function/method visible to dependants | Its declared contract must suffice; the body cannot silently add caller obligations. |
| Interface method, abstract/virtual contract, bodiless declaration, or implementation of an existing contract | Check against the declared contract, regardless of an implementation body's visibility. |
| Generic type and implementation applicability headers | Keep explicit declaration-level requirements; do not infer applicability from member bodies. |

“Public” refers to the existing Ember package visibility rules, not merely C `@export`. Existing rules for legal access and publication remain authoritative. Do not introduce a separate ad hoc visibility calculation just to infer bounds.

Explicit means **sufficient**, not maximally repetitive. Requirements implied by declared types, associated bindings, and superinterfaces need not be redundantly spelled. Ordinary local type inference and inference of type arguments at call sites continue unchanged.

### Public wrapper example

After private inference is available:

```ember
# Private helper: its body can establish Add[Output = T].
fn combine_impl[T](a: T, b: T) -> T:
    return a + b

# Public API: its source declares the requirement.
pub fn combine[T: Add[Output = T]](a: T, b: T) -> T:
    return combine_impl(a, b)
```

If the helper later requires `Hash`, the public body must fail checking unless that obligation already follows from the public contract. The compiler must not append `Hash` to the public signature. Report the helper call and requirement chain, and let the author change the body or explicitly change the public API.

Changing a helper to `pub` can therefore require materializing missing bounds. Provide a targeted diagnostic or optional code action; never make normal formatting publish inferred obligations.

### Private written bounds

For eligible private helpers, written bounds remain intentional requirements. Supported additional bounds may be inferred and combined with them. The complete contract is local to the package. A programmer needing a reviewed internal fixed contract can continue to write and test it; this pass does not add a second source mode to freeze it against all future internal edits.

Before helper inference is adopted, H23's explicit checking continues to apply to private helpers too. No compiler may silently accept a not-yet-supported inferred form.

### Withdrawn mechanism

**Do not add `@bounds(explicit)` or `@bounds(inferred)`.** No public-inference opt-in attribute or manifest escape hatch is introduced. No automatic conversion of existing declarations to another mode is required.

**Tests:** Missing public requirement rejected at its body use; valid transitive implications accepted; public wrapper unchanged after a helper strengthens; private-to-public promotion; interface implementations unable to add bounds; cross-module private inference with correct invalidation.

## SP-005 — Preserve fallback choices without speculative inference

**Disposition:** Clarify existing selection as needed; extend only for later private inference.  
**Sources:** H23 `[TYP-21]`, `[STD-17]`, `[STD-9]`, `[LEX-19]`, `[EXP-2]`; GSP §§3 and 15; C1 SP-005.

Keep the existing distinctions: `AddAssign` versus addition-and-assignment, `IndexSet` versus assignment through `IndexMut`, and formatting through `Display` versus its documented fallback.

Do not infer every candidate, require both alternatives, or choose a protocol by inspecting whichever concrete implementations appear during monomorphization. For a generic declaration, first establish written requirements and the supported unambiguous requirements of its resolved operations/callees. Select the fallback in that established environment, and preserve the selected operation in code generation.

If neither alternative is established for an eligible private helper, ask for the protocol choice or a more explicit expression. For example, `total = total + x` identifies addition; unbounded `total += x` alone does not uniquely say whether to require `AddAssign` or `Add`.

Public declarations must already have sufficient requirements. Their bodies cannot resolve ambiguity by exporting newly inferred alternatives.

Preserve evaluation order and one-time evaluation of targets. A fallback for `a[index()] += rhs()` is not a textual rewrite that duplicates `index()` or changes H23's right-side-first augmented-assignment order. Preserve borrow reservation/activation and drop behavior too.

**No fallback removal is proposed in this revision.** Replacing all augmented assignment with a compulsory `AddAssign` protocol, or all index assignment with `IndexSet`, would be a separate compatibility decision. Do not implement it indirectly with an overlapping blanket implementation.

**Tests:** Types with only each alternative and both with distinct behavior; explicit and later inferred forms; side-effecting indices/receivers; no concrete-instantiation fallback switch.

## SP-006 — Preserve associated identities and avoid guessing coercion paths

**Disposition:** Clarify existing semantics; bounded extension for later private inference.  
**Sources:** H23 `[IFC-4]`, `[TYP-5]`, `[STD-21]`, `[STD-27]`; GSP §4; C1 SP-006.

An associated type is identified by its receiver, nominal interface and arguments, and member. A member name such as `Output` is not a globally unique type variable. Keep those identities distinct even when user-facing shorthand is the same.

For the original `sqrt(x * x)` example, the obligations concern the multiplication result as well as the input:

```text
T implements Mul with output U.
U supplies the numeric capability required by sqrt.
sqrt returns U.Real.
The function's declared result must accept that result.
```

This does not automatically imply `U = T` or `U.Real = T.Real`. Require a sufficient declaration now; later helper inference may solve supported equalities, not invent a nominal owner for an unresolved projection.

For the initial helper solver, resolve unknown outputs by documented equality rules. Do not enumerate every output type that might widen, borrow, upcast, or wrap into the expected result. Apply the existing enumerated coercions once their types are known. A chosen canonical equality is not claimed to be the mathematically weakest possible bound.

No new projection punctuation, generic overload system, or arbitrary conversion-search mechanism is introduced.

**Tests:** Distinct same-named associated members, non-self operator outputs, known widening, permitted option coercion, incompatible return relationships, and diagnostics for unresolved projection owners.

## SP-008 — Keep formation, method requirements, and implementations separate

**Disposition:** Clarify existing contracts now; preserve these boundaries under later inference.  
**Sources:** H23 §V.3, §V.6, `[TYP-19]`, `[TYP-20]`, `[STR-5]`, `[IFC-4]`; C1 SP-008.

A generic type's formation requirements come from its declaration and representation, not from the bodies of every method attached to it. Adding a constrained `sort` helper must not make every instance of a collection require `Ord`.

A method has its own permitted contract. Public methods state their required bounds; eligible private ordinary helpers may later infer them. A method implementing an interface contract cannot add obligations beyond the implementation header and interface signature, regardless of visibility.

Keep ordinary and blanket implementation applicability explicit. Do not infer a blanket implementation's domain from its method bodies or use priorities to resolve overlap. Keep associated types and conditional derives rather than replacing them with formation-wide restrictions. Derive behavior must follow the adopted declaration/derive rules, not silently strengthen an unrelated type header.

The private-inference model no longer needs a mechanism for automatically publishing private helper requirements. A public caller must prove such a requirement from its declared contract or fail locally; it cannot expose an inaccessible helper abstraction by inference. Public declarations still follow the language's visibility rules for the types and interfaces they actually name.

**Tests:** Adding an unused constrained method does not change type formation; implementation methods cannot strengthen interface contracts; conditional derive cases remain conditional; a helper requirement does not leak into a public signature.

---

<a id="part-iv"></a>
# Part IV — Optional engineering work

The following are useful implementation ideas, not a mandatory language-simplification work package. Reuse what the compiler already has. Keep any semantic obligations described in the earlier parts even when these architectural changes are deferred.

## SP-021 — Reuse resolved operation decisions across compiler stages

**Disposition:** Engineering backlog.  
**Sources:** H23 `[TYP-21]`, `[STD-17]`, `[FN-5]`, `[CG-C-1]`, `[CG-C-8]`, `[IMP-11]`; C1 SP-021.

A shared resolved-operation representation can reduce duplicated decisions across type checking, borrowing, effects, the evaluator, and code generation. Useful information includes the selected nominal operation or intrinsic, types, parameter modes, evaluation order, coercions, borrow behavior, numeric policy, and source location.

This is one implementation strategy, not a prescribed compiler redesign. Existing structures may already carry the needed facts. Refactor only where duplicated decisions create a concrete defect or maintenance burden.

Keep documented intrinsic exceptions visible: consuming string concatenation, scalar shifts, `NonZero` divisors, and the distinct float relational/total-comparison operations must not be forced into an incompatible universal signature merely to simplify an internal table.

**Keep regardless of architecture:** Every stage must implement the same selected semantics. Tests can enforce that without requiring one particular record layout.

## SP-022 — Add signature inspection incrementally when it pays for itself

**Disposition:** Engineering backlog; not required for reading public bounds.  
**Sources:** GSP §14; H23 `[DOC-3]`, `[CLI-12]`, `[COST-4]`, `[IDE-3]`, `[IDE-4]`; C1 SP-022.

Retain `ember inspect --signature` as a possible future extension, not a required new command in the immediate pass. Public generic contracts now remain in declarations; users do not need this tool to discover missing body-derived public bounds.

For later private inference, start with a clear error/hover explanation of the effective requirement and the operation that introduced it. Expand inspection to ownership, borrow provenance, effects, and machine-readable output only where existing metadata supports it or a concrete workflow needs it.

A good explanation distinguishes written requirements from inferred helper requirements and assumed foreign facts from proved facts. A stale or incomplete editor snapshot must not be presented as a verified package interface. Preserve the existing responsive-editor policy.

**Optional example of future reporting:** “`T: Add[Output = T]`, required by `total + x` and its assignment to `total`.” No fixed-mode attribute is included in the output because this revision withdraws that feature.

## SP-029 — Review library layering before adding a publication framework

**Disposition:** Engineering backlog with an independent layer-correctness review.  
**Sources:** H23 Part XV introduction, `[STD-6]`, §XV.1's module table, `[ERR-8]`, `[ERR-11]`; C1 SP-029.

H23 gives exact std declarations authority where its prose leaves signatures open. Those `std/**/*.em` implementation files were not supplied here. This pass must not claim to have verified their signatures or reorganize them based only on the module summary.

A generated versioned interface artifact could make library modes, bounds, effects, and layer membership easier to inspect. Produce it from existing authoritative declarations when worthwhile; do not maintain another manually duplicated signature source.

Retain the concrete layer question: a core-only explicit-error `Result[T, E]` must not require allocating `AnyError` boxing solely because the result type also has a convenience default elsewhere. Allocating conversions must expose their allocation dependency.

**Revision from C1:** Do not automatically adopt a new core result name, defaulted alias, or prelude restructuring. C1's alias arrangement is one possible implementation design, not the approved smallest repair. First inspect actual layer enforcement, type/default availability, and the no-allocation use case; select a minimal compatible solution if a problem exists.

**Tests when relevant:** Core-only explicit-error results, unavailable allocation-dependent defaults/conversions, no duplicate incompatible result types, and agreement between published and compiled std declarations.

## SP-030 — Extend the existing tests and explanations per adopted change

**Disposition:** Extend existing practice; no new process framework.  
**Sources:** H23 `[TST-4]`–`[TST-11]`, `[TST-27]`–`[TST-30]`, `[DIA-6]`, `[DIA-13]`, `[DOC-2]`, `[BUD-*]`; C1 SP-030; P §3.

Reuse per-rule conformance directories, diagnostic before/after cases, the newcomer corpus, and existing performance checks. Add the small number of interaction tests necessary for each adopted change instead of creating a parallel all-or-nothing suite.

For later private inference, compare an inferred helper with an explicitly annotated version of the same helper under the **same visibility, modes, operation choices, and build settings**. Materializing an equivalent requirement set must preserve behavior. This is not a promise that every visibility change or stronger annotation is a semantics-preserving refactoring.

Explain the operation and needed capability before exposing solver terminology. A fix that adds `Clone`, allocates, changes ownership, or strengthens a public contract is a semantic choice, not a typo repair. Validate diagnostic fixes through the existing test mechanism where applicable; do not claim that one fix resolves unrelated errors elsewhere.

Teach concrete values and ordinary parameter modes first; introduce associated types and blanket implementations when needed for reusable infrastructure. Keep the full formal specification and the tested beginner guide distinct without deleting features.

Collect actual check/build time, code size, peak memory, allocation, retain/release, and relevant runtime-check data. “Already implemented” is a claim to verify with a focused test, not a reason to duplicate implementation work or omit a regression case.

---

<a id="adoption"></a>
## 4. Adoption sequence and migration

### 4.1 Recommended sequence

| Stage | Work | Exit evidence |
|---|---|---|
| **A — Establish the actual baseline** | Identify compiler/std revisions; turn reported existing behavior and each concrete discrepancy into small cases. | Classify each finding as aligned, prose mismatch, implementation mismatch, or unresolved design choice. Do not infer code behavior from the spec alone. |
| **B — Repair focused inconsistencies** | Associated defaults, `Result`, the lock-sharing condition, view cleanup, structural cleanup restrictions, the smaller float policy, and direct specification drift. | Updated authoritative rules and focused regression tests. No new inference or tool framework is needed. |
| **C — Resolve interacting safety details** | Counted copy/owned calls, destructor effects, source provenance, and any class-field access/layout implications. Check existing invalidation and semantic verdicts where affected. | Agreed invariant, implementation evidence, and compatibility/layout notes. Do not broaden `Copy` while leaving contradictory unchecked-access assumptions in place. |
| **D — Adopt independent improvements selectively** | Static-view container consistency, borrowed-key separation, explicit initialization APIs, mutable-pair access, and any chosen destruction-policy change. | Each selected API/policy has its own decision, tests, migration, and cost evidence. Unselected proposals remain pending. |
| **E — Consider private helper inference last** | Implement the bounded, initially non-recursive inference scope with explicit public contracts. | Useful savings demonstrated on Ember code; correct same-package invalidation; explicit recursive fallback; no hidden public requirement changes. |
| **As needed — Engineering backlog** | Operation-record refactors, extended signature inspection, API diffs, registry generation, and std interface publication. | A demonstrated need and improvement over existing machinery, not a blanket dependency on the language changes. |

The ordering expresses risk and dependencies, not a promised schedule. A particular safety correction may require a companion change from Stage C before it can ship; do not split such a pair just to match the table.

### 4.2 Dependencies that must not be lost

**Counted copy and cleanup:** SP-010 interacts with SP-011, SP-012, SP-015, and the existing class-access rules. Correct logical ownership is not sufficient if another rule still treats counted `Copy` as incapable of cleanup or reentrancy.

**Float comparison and consumers:** SP-018 includes its documentation updates and relation-consistency tests. Keeping `sort()` convenient does not permit an ordered search to mix `cmp` navigation with IEEE equality.

**Private inference and callers:** Part III depends on correct requirement propagation and same-package invalidation under SP-023. It does not depend on the API-diff product or the optional complete signature view.

**Library changes and regions:** SP-013 and SP-031 must use valid existing borrow/provenance enforcement. The API's convenience cannot bypass source-lifetime or exclusive-access requirements.

**Existing public semantic summaries:** Public explicit bounds do not make effects, borrowed-result summaries, or inline bodies irrelevant to dependency checking. Preserve those existing contracts rather than treating all body changes as private.

### 4.3 Migration categories

| Category | Examples | Migration treatment |
|---|---|---|
| Prose alignment with verified existing behavior | Associated defaults; reported generic IEEE comparison; corrected registry/cross-reference text. | Update the authoritative rule and add the regression test. Do not invent a compiler rewrite when none is needed. |
| Safety correction | Invalid lock-sharing capability; unaccounted counted owner; lost cleanup/provenance restriction. | Record any newly rejected programs and explain the safe alternative. Do not preserve an unsound path for compatibility. |
| Additive library capability | Checked mutable pairs; compatible static-view storage. | Add only after the exact API and safety contract are reviewed. Preserve existing defaults. |
| API split | Matching versus key construction. | Review and migrate explicit/blanket implementations normally; no special coherence exemption. |
| Behavior-changing policy | Default versus zero initialization; observable early destruction; generic float operators in an implementation that followed the conflicting H23 sentence. | Identify affected uses and intended behavior. Do not silently choose a meaning in normal formatting. |
| Later inference | Omitted bounds on private helpers. | Existing explicit code remains valid. New forms require actual compiler support; public bounds are not automatically removed. |

Keep an adoption record using H23's existing change-log and versioning rules. Any changed runtime layout, metadata format, or independently versioned protocol needs its own compatibility review. A new language document number alone does not make old binaries compatible.

### 4.4 What is not required to ship the immediate revision

The immediate revision does not require public bound inference, `@bounds(explicit)`, `PartialOrd`, a new total-order wrapper, compulsory `sort_by` on floats, a global recursive constraint solver, a unified compiler IR refactor, a comprehensive API-diff command, or wholesale registry replacement.

It also does not automatically require the optional `ToKey`, `alloc_zeroed`, or `get_pair_mut` APIs. Those are separately reviewable standard-library improvements, not hidden prerequisites for the corrections.

### 4.5 Definition of done for an adopted item

An item is implemented only when its chosen semantics, affected authoritative rules, exact library surface where relevant, support status, regression cases, and migration consequences agree. Use existing registries and gates. Keep unresolved cases identified rather than claiming blanket completion.

The final release report should list **adopted**, **already matched and clarified**, **deferred**, and **not adopted** items separately. Do not describe all thirty-one proposals as implemented because the immediate subset is complete.

<a id="tests"></a>
## 5. Focused conformance matrix

This matrix is a checklist for the corresponding adopted changes, not a demand to build every optional feature before a release. Use H23's existing test layout and measurement methodology.

| Family | Cases | Required property | Applies to |
|---|---|---|---|
| **G01 — Associated defaults** | Defaulted/overridden output, explicit equality, invalid/default cycles, default method assumptions. | Implementation defaults do not create universal generic equalities. | SP-003. |
| **G02 — Public boundary** | Public wrapper calls a helper whose requirements strengthen; private method promoted to public. | Public bounds do not silently expand; existing visibility-related borrow rules remain respected. | SP-004, SP-025; full helper case when inference is adopted. |
| **G03 — Nominal discovery** | Same-named unrelated interfaces, qualified calls, unresolved associated owner. | No program-wide guessing of an abstraction. | SP-002, SP-006. |
| **G04 — Private equivalence** | Same-visibility helper with inferred versus exactly materialized bounds. | Same selected operations, modes, relevant effects, and behavior. | Part III. |
| **G05 — Fallbacks** | Add-only, AddAssign-only, both; indexing and formatting counterparts. | Declaration-level choice, not a different operation chosen during instantiation. | SP-005. |
| **G06 — Recursion boundary** | Explicitly bounded recursive groups; omitted requirements in an unsupported recursive group. | Existing explicit code works; unsupported inference requests receive a precise explicit-contract diagnostic. | SP-001. |
| **G07 — Formation and derives** | Add an unused constrained method; conditional derive; interface implementation requiring too much. | Requirements stay on the declaration that needs them. | SP-008. |
| **O01 — Ownership counts** | Strong/weak handles, tuples, arrays, owned calls, source reused after the call. | Every logical owner is accounted for and cleanup is balanced. | SP-010. |
| **O02 — Cleanup and reentrancy** | Counted overwrite, reentrant destructor, address-taken class field. | No invalid access or assumption that `Copy` means no user-code cleanup. | SP-010 and its access review. |
| **O03 — Borrowing cleanup** | Span, mutex/cell guard, borrowing aggregate, early drop. | Regions remain valid for actual cleanup uses; no unwanted span runtime state. | SP-012. |
| **O04 — Safety-critical scope** | Direct and nested escape/forget attempts; forbidden move; early exit and suspension. | Wrapping does not evade scope joining or lifetime restrictions. | SP-015. |
| **O05 — Storage regions** | Static map keys/values; local text borrow rejected; bounded container. | The selected storage rule applies consistently without implicit lifetime extension. | SP-013 if adopted. |
| **O06 — Observable lifetime** | Destructor log, weak upgrade, foreign release, explicit destruction, stack promotion. | The adopted lifetime policy is identical across optimization profiles. | SP-014 if adopted. |
| **T01 — Lock capabilities** | Mutex of `Cell`, read-sharing a RwLock of `Cell`, immutable shareable payload. | Exclusive access and simultaneous readers have different necessary bounds. | SP-011. |
| **T02 — Transfer and final release** | Shared owned payload destroyed on another thread, channel payload, affine wrapper. | Access and destruction are both justified by the capabilities. | SP-011. |
| **T03 — View provenance** | Transfer a view from a thread-confined class versus an appropriately scoped exclusive slice. | Pointee type alone does not erase relevant source restrictions. | SP-011. |
| **R01 — General results** | Non-Error payload, default AnyError, same-type `?`, conversion, reporting. | Container formation is general; specialized methods enforce their own needs. | SP-009. |
| **L01 — Borrowed lookup** | Non-Clone key, matching-only query, text lookup/update, owned insertion. | Lookup does not need conversion; missing-key construction occurs only as specified. | SP-016 if adopted. |
| **L02 — Initialization** | Zeroable/nonzero-default type, effectful default, empty/no-drop cases. | Default and explicit zero policies are not silently interchangeable. | SP-017 if adopted. |
| **N01 — Operator stability** | Signed zeros, infinities, NaNs; concrete and generic comparisons. | Float operators stay IEEE under the same applicable numeric policy. | SP-018. |
| **N02 — Total-order consumers** | Sort then search; iterator min/max; derived cmp; membership versus ordered equivalence. | Each algorithm consistently uses its chosen relation. | SP-018. |
| **N03 — Numeric execution phase** | Same deterministic math in evaluator and compiled runtime. | No hidden substitution of another source operation. | SP-019. |
| **E01 — Complete effects** | Allocating formatter, counted cell copy, lazy callback, generator cleanup. | Full effects include invoked code and ownership work at the relevant phase. | SP-020. |
| **B01 — Invalidation** | Changed effect, view summary, inline body; later private helper requirements. | Incremental and clean builds agree; relevant callers/code regenerate. | SP-023, SP-025. |
| **B02 — Proof policy** | Host inline decision, optimization profile, sharing threshold. | Semantic acceptance and operation meaning do not depend on backend heuristics. | SP-028, SP-018. |
| **S01 — Surface drift** | Formatter line endings, known attribute positions, CLI/help parity, weak upgrade after zero. | The edited normative descriptions and existing registries agree. | SP-026. |
| **S02 — Configuration** | Warning policy, work budget, unavailable layer, realtime expansion. | Different failure categories are reported accurately; no disabled safety invariant. | SP-027. |
| **S03 — Core errors** | Core-only explicit error result and an allocation-dependent error conversion. | Core use has no undeclared boxing dependency; no duplicate Result identity. | SP-029 review. |
| **U01 — Formatting/refactoring** | Intentional bounds, callback extraction, public promotion, ordering-sensitive code. | Ordinary formatting changes no semantics; refactorings expose real contract changes. | SP-024. |
| **U02 — Mutable pair** | Equal/out-of-range/reversed indices, zero-size elements, lifetime escape. | References are valid and disjoint before exposure; no allocation machinery is introduced. | SP-031 if adopted. |

### Performance and implementation evidence

For a semantic clarification reported as already implemented, establish a regression test rather than assuming the implementation agrees. For a new feature, measure actual effects on existing check/build budgets and representative source programs.

Compare relevant emitted operations, retains/releases, allocation counts, runtime checks, code size, and peak memory under matched settings. Do not infer speed from shorter source, and do not promise that inferred generics always produce direct calls when the existing sharing policy permits indirect dispatch.

A correctness repair can invalidate an unsound optimization. Report its cost, then recover performance with valid proofs; a performance target does not justify keeping an unsafe path.

---

<a id="appendix-a"></a>
# Appendix A — Disposition of the original Generic Simplicity Pass

| Original item | Revised treatment |
|---|---|
| Core principle: infer every unambiguous relationship | Replace with a bounded package-private helper feature. Explicit public contracts are deliberate API documentation, not redundant ceremony. |
| Proposal A: body-bound inference | Defer; adopt only the supported private-helper subset first. |
| Explicit bounds plus inferred union | Allow only within eligible private helpers after inference is adopted. Public and declared interface contracts cannot silently strengthen. |
| Proposal B: common associated defaults | Keep the implementation-default concept; reject the inference that bare `T: Add` guarantees `Output = T`. Adding defaults to std operators is a separate convenience decision. |
| Proposal C: keep associated types | Retain. Do not replace associated outputs with extra generic parameters merely to shorten the spec. |
| Proposal D: blanket implementations | Retain explicit applicability and ordinary coherence. No priorities, specialization, negative bounds, or implementation-order selection. |
| Proposal E: transitive requirements | Later: propagate through resolved same-package helper contracts; a public caller proves them from its declaration or fails locally. |
| Proposal F: signature tooling | Optional incremental extension. A complete public bound contract is already in source. |
| Proposal G: preserve generic borrowing | Retain H23's established rules, including ODR-048 and existing multi-region summaries. |
| Proposal H: separate indexing protocols | Retain `Index`, `IndexMut`, and `IndexSet`; do not merge read, mutation, and insertion. |
| Proposal I: operator protocols | Retain nominal protocols and documented scalar/intrinsic exceptions; clarify operation selection without prescribing an IR redesign. |
| Proposal J: const generics | Preserve useful concrete computation and keep symbolic proof requirements bounded and specified. |
| Optional phantom-field cleanup | Do not adopt. Keep `[TYP-35]` without mandatory `_tag: Phantom[T]` fields. |
| Automatic formatter removal of bounds | Reject for normal formatting. Keep deliberate, reviewed refactorings separate. |
| Success criteria | Evaluate the adopted subset against real ergonomics, semantic consistency, safety, compiler-time, and cost evidence. No blanket completion claim. |

## How the pushback is incorporated

**P §1 — Inference:** Accepted in direction and boundary. Public body-derived bounds and the fixed-mode attribute are removed from the proposed adoption set. Private inference moves last. Nominal-resolution, fallback, recursion, and dependency rules remain necessary, but the initial scope avoids an expansive solver. Associated-default clarification is independent.

**P §2 — Floats:** Accepted as the immediate policy. No `PartialOrd` redesign or compulsory float-sort rewrite. Retain the documented total-order consumers. Complete the companion operator-table edits and algorithm tests; the relations' equality differences are explicitly acknowledged.

**P §3 — Process:** Accepted as an adoption/scope correction. Reuse existing rule-index, diagnostic, and conformance tools. Optional architecture and reporting move to the backlog. Concrete semantic guarantees and invalidation obligations remain, without prescribing a large infrastructure project to enforce them.

The two qualifications from R are retained: private inference does not eliminate all dependency tracking, and the smaller float policy requires more than replacing one sentence in isolation.

<a id="appendix-b"></a>
# Appendix B — Preserved choices and withdrawn mechanisms

## Features not removed

Keep Ember's current capability set and layered profiles. This revision is not a recommendation to shrink the language into a proof of concept, remove associated types or blanket implementations, replace classes with only value types, abandon multi-region views, or defer existing systems/interop capabilities merely because they make the full specification long.

Keep explicit storage choices, parameter modes, ownership semantics, source-level arithmetic policies, the single path separator, current literal typing, and the existing distinctions between panic and recoverable failure. No generalized structural typing, lifetime-parameter syntax, higher-kinded types, negative bounds, or specialization is added.

Keep phantom parameters as specified by H23. Introducing an obligatory marker field would add authoring and layout considerations without eliminating the underlying type-identity question.

Keep context-dependent callable representation as currently specified; do not bundle a new callable syntax into this revision. Refactorings must preserve capture, ownership, and dispatch effects rather than assuming that introducing a local is always semantics-neutral.

## Mechanisms withdrawn from C1's recommended adoption set

| Earlier mechanism | Revised disposition |
|---|---|
| Public body-derived generic bounds by default | Withdrawn from this pass. |
| `@bounds(explicit)` | Withdrawn; not new syntax. |
| `@bounds(inferred)` or a public-inference opt-in | Not introduced. |
| `PartialOrd` and its derive as the float repair | Withdrawn from this pass. |
| Removing ordinary floats from `Ord` | Not adopted. |
| Compulsory `sort_by(math.total_cmp)` for float sorting | Not adopted; `xs.sort()` remains convenient. |
| Changing float `min`/`max` to a new ordinary-comparison NaN policy | Not adopted by this repair. |
| Mandatory total-order wrapper | Not introduced. An explicit custom comparator remains available through existing APIs. |
| Mandatory unified semantic record, global API-diff product, or new registry framework | Moved to optional engineering work, while preserving semantic correctness obligations. |
| Automatic `Result` alias/prelude/layer restructuring | Deferred pending actual std/layer review. |
| Broad recursive body-bound solver as an initial requirement | Replaced with an initially explicit recursive-contract boundary. |
| Removing generic bounds during ordinary formatting | Rejected. |

The separately proposed library names `ToKey`, `alloc_zeroed`, and `get_pair_mut` remain candidates, not automatically adopted language features. An implementation claiming support for them must first have an approved exact surface and tests.

## Scope boundary

This is a revised simplification proposal grounded in the supplied H23 and review materials. It is not a new complete audit of the C++ annex, hot-reload protocol, GPU host layer, or compiler repository. Adopted ownership, metadata, or layout changes must still be integration-tested with those existing capabilities.

<a id="appendix-c"></a>
# Appendix C — Source identities and verification limits

## Source files

The following SHA-256 digests identify the supplied source bytes used for this revision, not compiler versions or implementation commits.

| Label | File | SHA-256 |
|---|---|---|
| H23 | `Ember_v0.9.9_Hardened_23.md` | `d42823462dc090adbf9a5a4fd99bb9ca7d0affa690c7f1f924a0adcd3414d9bd` |
| GSP | `Ember_Generic_Simplicity_Pass.md` | `301b90821b4e5eb2b8f2240b83c1873d27bfa382c60e2a0bba6800908ff3081d` |
| C1 | `Ember_Simplification_Pass_Consolidated.md` | `89db3108d9bcb3896f7a013de9c0f7d512a91eefb337c5bbcf39e759b91f3374` |

P and R are the supplied pushback and the subsequent analysis in this conversation. They establish the requested revision direction. Claims in P about implemented behavior remain attributed until confirmed against the actual compiler.

## What was checked for this artifact

This revision uses the supplied source rules, the earlier thirty-one-item pass, the pushback, and the revised analysis. The document preserves the review identifiers and records whether each recommendation remains a repair, an independently reviewable change, a later inference feature, or optional engineering work.

The source files were not overwritten. This is a separate revised proposal.

No Ember compiler was run, no performance claim was measured, and no unsupplied current std declarations or compiler internals were inspected. The test matrix and adoption sequence specify the evidence required before implementation claims can be made. Where the supplied source leaves a question to `std/**/*.em`, that limitation remains explicit.

---

## Final adoption principle

> **Prefer the smallest rule change that fixes the demonstrated problem. Keep public contracts explicit, let package-private inference earn its implementation cost, preserve the distinction between IEEE comparison and total ordering, and treat safety repairs, optional library improvements, and compiler engineering as separate adoption decisions.**
