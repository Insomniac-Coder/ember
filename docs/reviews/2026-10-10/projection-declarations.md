# Projection declaration and method rebasing — 2026-10-10

This batch fixes D-540 and the independently reproduced D-541 under the
existing IFC-4/TYP-16/TYP-17/TYP-18 contract. The baseline is cb9d2df, promoted
after all five jobs in CI run 38024872799 succeeded. No specification or
runtime/standard-library source changes are included.

D-540 resolved `Build[S.Item]` before S.Item's hidden parameter existed.
Written parameter slots are now reserved first; positional-argument projections
are declared when needed. Later written parameters retain their indices.
Equality-binding payloads and callable bounds retain deferred resolution.

The generic-owner method test exposed D-541: recipe registration substituted
the method's type values but did not move its projection metadata. A hidden
S.Item then referred to the slot occupied by another parameter. Updating its
base by the same owner-to-caller offset fixes both failed inference and the
misleading `expected S.Item, found S.Item` diagnostic. The independent
`Holder[T].item[S: Source](...) -> S.Item` test reproduces this on the baseline
without using a positional projection bound. ADR-146 records the implementation.

All six regression files fail against the saved baseline and pass against the
candidate, including native output checks. They cover int/str calls, nested
Option, sibling projections, later written parameters, a generic method's
owner prefix, equality/callable coexistence, and one diagnostic per invalid
member. The nearby IFC-4/TYP-16/TYP-17/TYP-18 sweep passed 152 files before the
last independent D-541 case was added; that case then passed in the six-file
focused run. An initial command also named the absent GRM-8c directory; its
empty-directory error was an invocation mistake, not a test failure, and the
corrected four-directory sweep is the retained result.

All twelve repository gates passed, including specification examples and error
pages; Appendix generation left the specification bytes unchanged.

The [machine-readable evidence](projection-declarations-results.json) includes
baseline/final focused results and all 50 release benchmark C hashes. Every
benchmark's C is byte-identical before/after. This protects the existing runtime
code without claiming new timing measurements or changing README numbers.
The local compiler snapshots, intermediate failures, C outputs and gate logs
remain in `build/d540-20261010/`. The full workspace matrix runs in CI before
main promotion; the owner requested avoiding another local full conformance run.

Read-only review checked stable slots, deferred equality/callable handling,
and the method rebase against the existing interface-method pattern. Formatting
diagnostic follow-ups remain open as recorded in the previous review.
