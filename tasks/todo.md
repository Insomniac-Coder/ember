# Phase 2 and Phase 3 closure tasks

## Task 1: Direct Clone

**Acceptance criteria:** A non-`Copy` struct can implement `Clone`, calling `.clone()` returns an independent value, and `[OWN-8]` has executable conformance evidence.

**Verification:** Focused compiler tests, all-profile run-pass probe, and workspace tests.

**Dependencies:** None.

**Estimated scope:** Medium.
## Task 2: Derived Clone

**Acceptance criteria:** `@derive(Clone)` emits field-wise clones and rejects fields without `Clone`.

**Verification:** Focused compiler tests, all-profile run-pass and compile-fail probes.

**Dependencies:** Task 1.

**Estimated scope:** Medium.
