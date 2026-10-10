# Associated-type and generic-bound fixes — 2026-10-10

The eight October 9 probes were checked against the pinned
`Ember_v0.9.10_Hardened_2.md`. They identify seven implementation defects
(D-533 through D-539): Result resolution and nested equality substitution
share the incomplete constructor traversal. No specification change is needed.

The Float expectation required correction. STD-27 grants numeric operations,
Copy, Eq, Ord and Default; it does not grant Display/Debug. TYP-17 permits only
bound-provided capabilities, and STD-9 requires E2040 when text capability is
missing. The fix rejects implicit text capability, reports the missing bound,
and preserves explicit Display/Debug for both f32 and f64. Float's implied
interfaces also use their canonical names, so a program's Eq/Default interface
is not acquired by sharing the standard interface's short name.

## Implementation and coverage

- Generic operator lookup requires the nominal standard interface, using the
  existing recovered bound lookup. Ordinary same-named methods remain callable.
  Indexed assignment retains its nominal IndexSet path.
- Generic equality and ordering require Eq/Ord before representation selection.
  Float comparisons retain their IEEE behavior, including NaN.
- Concrete associated values meet equality requirements through the specific
  required interface instance, resolving the wanted type through the outer owner.
  Incomplete declarations and opaque instances retain existing deferral rules.
- Interface instantiation substitutes associated positional bounds, equality
  instances and equality payloads. Two cached instances (int/str) are exercised.
- Bound methods resolve their own positional constraints through the receiver.
- Parameter substitution and both associated traversals share the existing
  constructor rebuild logic (ADR-145), preserving canonical nominal definitions.
  Tests exercise both Result payloads, nested Option, records, enums, classes,
  Box, Cell, arrays and tuples with concrete calls.

Eighteen new conformance files provide acceptance/rejection pairs and concrete
execution checks. The saved baseline is the compiler from `c428d11`, copied
before rebuilding. [Final baseline results](baseline-results.json) reproduce 13 failing regression
files and five passing controls. All 18 pass with the candidate. The initial
RED run and intermediary slice checks are retained in `build/assoc-20261010/`.
[Focused final results](focused-results.json) record the annotations check.
The older Pair default-method test now explicitly requires Eq, which its
comparison needed under TYP-17; its behavior assertions are retained.

Validation: the focused files and all 101 files in the nearby IFC-4/TYP-17/STD-27/TYP-21/TYP-30/
STD-9 rule groups, eight type-checker unit tests, and twelve local repository
gates. The full workspace suite is delegated to the four-platform CI matrix
as the owner requested. CI must pass before main is advanced.

## Existing benchmark protection

All 50 release benchmark programs produce byte-identical C with the baseline
and candidate compilers. [The hashes](benchmark-generated-c.json) record both
successful compilation and exact output equality. Runtime and standard-library
sources are unchanged in this batch, so the existing benchmark execution code
has no added work. This is a code-identity check, not a new timing claim; README
numbers remain the measured October 9 results. Local evidence and generated C
are preserved under `build/assoc-20261010/`.

## Follow-up findings

D-540 is a separate declaration-order defect: a positional bound such as
`C: Build[S.Item]` resolves S.Item before hidden projection parameters are
created. The [minimal reproduction](../2026-10-09/probes/positional_projection_bound.em)
reports E2040 on both the baseline and this batch. The method-bound regression
uses a shared explicit T to isolate the repaired receiver-substitution path.

Read-only review also identified follow-up diagnostic paths to reproduce:
f-string and aggregate printing may still use E1010/E0900 for missing generic
formatting bounds. Some formatting predicates still index the current generic
list directly; no imported/displaced-parameter producer was established.
Literal-left comparisons with an unconstrained right operand reject via type
mismatch; a symmetric missing-bound diagnostic remains a possible improvement.
These are not claimed fixed by this batch.
