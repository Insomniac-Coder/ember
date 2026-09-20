# Implementation Plan: Phase 2 and Phase 3 closure

## Overview

Complete the remaining Phase 2 ownership exits and Phase 3 object/runtime exits in small, independently verified commits. Phase 2 prerequisites scheduled later are intentionally pulled forward under the user's authorization.

## Architecture Decisions

- Preserve the frozen language target; implementation and conformance evidence change, not the specification.
- Start with `[OWN-8]`: direct `Clone` and field-wise `@derive(Clone)` are the smallest unbuilt Phase 2 blocker.
- Each slice must add an adversarial source test, pass focused tests and a runtime probe, then be committed and pushed before the next slice.

## Task List

### Phase 2 prerequisites

- [ ] Task 1: Implement direct `Clone.clone` for non-`Copy` structs and `[OWN-8]` conformance.
- [ ] Task 2: Implement field-wise `@derive(Clone)`, including rejection when a field is not `Clone`.
- [ ] Checkpoint: Run focused ownership/type-checking tests and the full workspace suite.
- [ ] Task 3: Add manifest `[lints]` configuration and opt-in `L3014` coverage for `[LT-1b]` and `[LT-2a]`.
- [ ] Task 4: Implement the remaining phase-required effects, thread traits, and semantic diagnostic producers in dependency order.
- [ ] Checkpoint: All Phase 2 rule-family, UI, M2, and unclassified-borrow-error gates pass.

### Phase 3 closure

- [ ] Task 5: Complete remaining owned dynamic-interface payload and dispatch cases.
- [ ] Task 6: Complete managed class ownership, exclusivity, generics, and conformance cases.
- [ ] Checkpoint: All Phase 3 object/runtime exit gates pass.

## Risks and Mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| Later-phase prerequisites widen the work | High | Keep each feature independently testable and committed. |
