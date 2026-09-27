# Phases 1–3 closure working ledger

Working notes only. This is not a phase-exit ruling and does not revise the
project's recorded phase percentages. Entries distinguish implementation
leads from verified defects and bounded test evidence from full phase closure.

## Phase 2 — ownership

The older audit table is stale for several rows. Its `not yet probed` status is
not evidence that coverage is absent:

| Area | Current evidence / remaining question |
|---|---|
| `OWN-6` | The audit's `not yet probed` label is stale ([AUDIT-0.9.9.md](AUDIT-0.9.9.md#L100)). Seven cases in [`tests/conformance/OWN-6`](../tests/conformance/OWN-6/) cover take/replace/swap/forget, `Default`, aliasing, use-after-forget, and class-handle use after `mem.drop`. The accepted test calls ordinary `mem.drop`, but no dedicated plain non-class moved-from-use case isolates it. |
| `BRW-4` | The audit row is stale ([AUDIT-0.9.9.md](AUDIT-0.9.9.md#L107)). Four cases in [`tests/conformance/BRW-4`](../tests/conformance/BRW-4/) cover distinct and nested field projections, same-field conflict, and method conflict. Class-field access is expressly governed by Part VIII, not this rule. |
| `LT-4` | The audit's `not yet probed` label is stale ([AUDIT-0.9.9.md](AUDIT-0.9.9.md#L108)). The accept/reject pair in [`tests/conformance/LT-4`](../tests/conformance/LT-4/) checks an arena allocation within its region and a returned allocation that outlives its arena; ARN-1/ARN-7 also test reset conflicts while views remain live. |
| `LT-6` | The audit's `not yet probed` label is stale ([AUDIT-0.9.9.md](AUDIT-0.9.9.md#L109)). [`tests/conformance/LT-6`](../tests/conformance/LT-6/) rejects named-lifetime syntax and accepts a character literal as non-lifetime syntax. Clause guidance for restructuring uninferrable relationships (owned result, index, view struct, `@borrows`) is not individually asserted by this pair; map related LT-1/LT-1a cases before claiming clause-complete coverage. |
| `SPN-5` | The audit's `not yet probed` label is stale ([AUDIT-0.9.9.md](AUDIT-0.9.9.md#L112)). Eight cases in [`tests/conformance/SPN-5`](../tests/conformance/SPN-5/) cover mutable splitting through a `MutSpan`, iterator regions/disjoint items, reuse, parent conflicts, and the rejected `Array.split_at_mut` spelling. Thin direct coverage remains for `Span.split_at`, simultaneous use of both halves returned by `MutSpan.split_at`, and the specified `i > len` panic. `get_pair_mut` is cross-referenced to BRW-5 and should be mapped there. |
| `DRP-4` | The audit's `not yet probed` label is accurate ([AUDIT-0.9.9.md](AUDIT-0.9.9.md#L111)); there is no DRP-4 conformance directory or test for `@noalloc` on a type's drop/panic inside drop. The handoff records the `@noalloc` promise as Phase-4-effects-gated ([HANDOFF.md](HANDOFF.md#L7848)); this is deferred scope, not a confirmed implementation defect. The SHOULD NOT block wording is advisory. |
| `BRW-11` | H39 requires all call borrows to overlap; two overlapping `mut` arguments must produce E3022, while disjoint fields are allowed (H39 lines 2085–2088). The direct overlap-reject and disjoint-field-accept cases both pass focused validation. |
| `DRP-7` | Paired probes found D-368 (missing local/counter storage ends) and D-369 (unrelated String cleanup incorrectly prolonging a plain-view borrow). The fixes track scope ends and infer destructor region reads from the actual fields being destroyed. Focused direct, generic payload, source-before-borrower, return, break/continue, counter and guard cases pass. Mutation break-tests reproduce each defect when its fix is removed and pass after restoring it. Final MSVC/clang-cl workspace suites, 18 gates, H39 examples, and full annotation sweep pass; D-368 and D-369 are fixed (see [DEFECTS.md](DEFECTS.md#L59-L60)). |
| Cell/thread boundaries | The former CELL-3 concern is not simply “no thread support”: `THR-1/reject_cell_field_even_when_let.em` checks a `Cell` field rejected by thread-property analysis. Remaining transfer/runtime thread-boundary work belongs to its current phase obligations, not this single type-property test. |

### Phase 2 audit follow-up

1. BRW-11 focused cases pass. DRP-7 focused cases and mutation break-tests pass
   after restoring both fixes. Final MSVC and clang-cl workspace suites exit 0,
   all 18 gates pass, H39 examples report 46 with `failing 0`, and the
   287-directory annotation sweep reports `failing 0`.
2. Add focused SPN-5 cases for both split halves, `Span.split_at`, and out-of-range panic; consider isolating ordinary non-class `mem.drop` use-after-move for OWN-6.
3. Reconcile CELL/thread-boundary evidence and keep DRP-4 scoped to its explicit Phase-4 effects dependency.
4. Continue the full rule-level closure audit; do not recompute percentages until the parent reviews evidence against exit criteria.

This remains a bounded list; an exhaustive phase-exit audit is still required.

## Phase 1 — bounded lead-confirmed gaps

This list is not a complete Phase 1 audit. It records concrete current
blockers or investigation leads identified during the present closure work;
no estimate changes follow from it.

| Area | Finding |
|---|---|
| `FN-5` defaults | All current focused default-argument cases pass, including earlier-parameter, generic, generic-owner, method-default, declaration-scope, borrowed-identity, mutable-handle write-back, and reservation/conflict cases. Reverting alias normalization wrongly accepts `reject_default_borrow_live_at_mut_activation.em`; restoring it recovers the expected E3021 call-site diagnostic. Full MSVC/clang-cl suites, 18 gates, and annotations pass. The declaration arithmetic-policy boundary remains pending ODR review; focused behavior and this working ledger do not resolve it or close Phase 1. |
| `TYP-8` overflow attributes | `@overflow(saturate)` is rejected with E0104 by the attribute applicability/implementation gate (`ATT-1`/`ATT-6`), not E0900. Saturation remains unimplemented pending the core-language review. |
| `TYP-9` float controls | `@fastmath` and `@fp(contract)` remain in the unbuilt function-attribute gate (`compiler/ember_typeck/src/lib.rs:35243`); attribute acceptance/semantics remain open. |
| `CTL-3b` counted loops | `step_by` and composed counted-loop adapters are absent. This needs dependency/iterator design; do not treat existing range-loop tests as that coverage. |
| `EXP-4` temporary iterable | An indirect borrowed iterable temporary is an investigation lead only; no current reproduction or defect has been confirmed. |

## Phase 3 — lead audit pending

The current handoff records partial coverage for object layout, per-field
runtime access, exception safety, dispatch, and weak handles, but also leaves
the complete `OBJ`/`RC`/`EXC`/`DSP`/`WK` matrix and static/runtime cycle
diagnostics open ([HANDOFF.md](HANDOFF.md#L10689-L10691)). This entry is a
pointer for the lead's audit, not an independent status update; no Phase 3
rules are reclassified here.

Current bounded evidence:

* `WK-8`: runtime milestones pass 4/4, including the three-object SCC
  and two independent SCCs in one run. They check object counts, internal
  edges, separate reports, and static prediction. No compiler change or defect
  was needed for these coverage gaps; the final MSVC/clang-cl workspace suites
  and all gates pass.
* `RC-3`: raw-pointer and other liveness-boundary probes remain open.
* `RC-4`: atomic selection alone does not prove concurrent retain/release or
  weak-upgrade CAS behavior; these runtime probes remain open.
* `EXC-10`: current loop hoisting covers canonical single loops; nested-loop
  access reuse remains an implementation gap.
