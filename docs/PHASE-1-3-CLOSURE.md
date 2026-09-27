# Phases 1–3 closure working ledger

Working notes only. This is not a phase-exit ruling and does not revise the
project's recorded phase percentages. Entries distinguish implementation
leads from verified defects and bounded test evidence from full phase closure.

## Phase 2 — ownership

The older audit table is stale for several rows. Its `not yet probed` status is
not evidence that coverage is absent:

| Area | Current evidence / remaining question |
|---|---|
| `OWN-6` | The audit's `not yet probed` label is stale ([AUDIT-0.9.9.md](AUDIT-0.9.9.md#L100)). Existing cases cover take/replace/swap/forget, `Default`, aliasing, use-after-forget, and class-handle use after `mem.drop`; new [`reject_use_after_mem_drop_of_noncopy_value.em`](../tests/conformance/OWN-6/reject_use_after_mem_drop_of_noncopy_value.em) isolates ordinary non-Copy moved-from use. Focused annotations and the current full workspace suites pass. |
| `BRW-4` | The audit row is stale ([AUDIT-0.9.9.md](AUDIT-0.9.9.md#L107)). Four cases in [`tests/conformance/BRW-4`](../tests/conformance/BRW-4/) cover distinct and nested field projections, same-field conflict, and method conflict. Class-field access is expressly governed by Part VIII, not this rule. |
| `LT-4` | The audit's `not yet probed` label is stale ([AUDIT-0.9.9.md](AUDIT-0.9.9.md#L108)). The accept/reject pair in [`tests/conformance/LT-4`](../tests/conformance/LT-4/) checks an arena allocation within its region and a returned allocation that outlives its arena; ARN-1/ARN-7 also test reset conflicts while views remain live. |
| `LT-6` | The audit's `not yet probed` label is stale ([AUDIT-0.9.9.md](AUDIT-0.9.9.md#L109)). [`tests/conformance/LT-6`](../tests/conformance/LT-6/) rejects named-lifetime syntax and accepts a character literal as non-lifetime syntax. Clause guidance for restructuring uninferrable relationships (owned result, index, view struct, `@borrows`) is not individually asserted by this pair; map related LT-1/LT-1a cases before claiming clause-complete coverage. |
| `SPN-5` | The audit's `not yet probed` label is stale ([AUDIT-0.9.9.md](AUDIT-0.9.9.md#L112)). Existing cases cover mutable splitting through a `MutSpan`, iterator regions/disjoint items, reuse, parent conflicts, and the rejected `Array.split_at_mut` spelling. New cases cover `Span.split_at` values at both ends and the middle, simultaneous use/mutation of both `MutSpan` halves with writes observed through the owner, and the specified `i > len` panic on an empty span. Focused annotations and the current full workspace suites pass. `get_pair_mut` is cross-referenced to BRW-5 and should be mapped there. |
| `DRP-4` | The audit's `not yet probed` label is accurate ([AUDIT-0.9.9.md](AUDIT-0.9.9.md#L111)); there is no DRP-4 conformance directory or test for `@noalloc` on a type's drop/panic inside drop. The handoff records the `@noalloc` promise as Phase-4-effects-gated ([HANDOFF.md](HANDOFF.md#L7848)); this is deferred scope, not a confirmed implementation defect. The SHOULD NOT block wording is advisory. |
| `BRW-11` | H39 requires all call borrows to overlap; two overlapping `mut` arguments must produce E3022, while disjoint fields are allowed (H39 lines 2085–2088). The direct overlap-reject and disjoint-field-accept cases both pass focused validation. |
| `DRP-7` | Paired probes found D-368 (missing local/counter storage ends) and D-369 (unrelated String cleanup incorrectly prolonging a plain-view borrow). The fixes track scope ends and infer destructor region reads from the actual fields being destroyed. Focused direct, generic payload, source-before-borrower, return, break/continue, counter and guard cases pass. Mutation break-tests reproduce each defect when its fix is removed and pass after restoring it. Final MSVC/clang-cl workspace suites, 18 gates, H39 examples, and full annotation sweep pass; D-368 and D-369 are fixed (see [DEFECTS.md](DEFECTS.md#L59-L60)). |
| Cell/thread boundaries | The former CELL-3 concern is not simply “no thread support”: `THR-1/reject_cell_field_even_when_let.em` checks a `Cell` field rejected by thread-property analysis. Remaining transfer/runtime thread-boundary work belongs to its current phase obligations, not this single type-property test. |

### Phase 2 audit follow-up

1. BRW-11 focused cases pass. DRP-7 focused cases and mutation break-tests pass
   after restoring both fixes. The current clang-cl full workspace suite exits
   0. The initial MSVC `--no-fail-fast` run had only the stale integer-shift
   expectation; after correcting it to use `wrapping_shl`, the exact failed
   target passed 1/1, and all other MSVC targets had passed in the initial run.
   All 18 gates pass, H40 examples report 46 with `failing 0`, and the current
   annotation sweep reports `failing 0` across 288 directories and 1,794 files.
2. Reconcile CELL/thread-boundary evidence and keep DRP-4 scoped to its explicit Phase-4 effects dependency.
3. Continue the full rule-level closure audit; do not recompute percentages until the parent reviews evidence against exit criteria.

This remains a bounded list; an exhaustive phase-exit audit is still required.

## Phase 1 — bounded lead-confirmed gaps

This list is not a complete Phase 1 audit. It records concrete current
blockers or investigation leads identified during the present closure work;
no estimate changes follow from it.

| Area | Finding |
|---|---|
| `FN-5` defaults | Current focused default-argument cases pass, including earlier-parameter, generic, generic-owner, method-default, declaration-scope, borrowed-identity, mutable-handle write-back, and reservation/conflict cases. Reverting alias normalization wrongly accepts `reject_default_borrow_live_at_mut_activation.em`; restoring it recovers E3021 at the call site. ODR-084/H40 settles declaration arithmetic-policy boundaries; ADR-068's propagation is implemented. The current full verification is recorded above; this bounded evidence does not close Phase 1. |
| `TYP-8` overflow attributes | Module overflow directives and `@overflow(panic|wrap|saturate)` now have focused coverage, including saturation and const-evaluation policy. The `#! module` syntax, declaration policy, CT-4 integration, and current full validation pass; no broad Phase 1 closure is claimed. |
| `CT-4` constant evaluation | Supported arithmetic forms are covered by focused cases for lexical/module policy, imported wrapped constants, typed unsigned negation, and nested signed-minimum literals. Comptime calls and powers, plus const-generic evaluation, remain gated; do not infer CT-4 closure from this batch. Current annotation and workspace verification pass for the covered cases. |
| `TYP-10` shifts and `STD-20` fixed integer methods | Invalid shift amounts now panic regardless of overflow policy; fixed-contract methods retain checked semantics, including `MIN.rem_trunc(-1) == 0` while a zero divisor still panics. The focused fixtures and current full verification pass. |
| `TYP-9` float controls | `@fastmath` and `@fp(contract)` remain unimplemented. The next-session work has not started; map the strict/default policy boundary and separate-TU backend requirement under ODR-085 before implementation. |
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
