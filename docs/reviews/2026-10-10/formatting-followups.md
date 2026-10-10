# Formatting and comparison follow-ups — 2026-10-10

The three requested follow-ups are confirmed and fixed as D-542, D-543 and
D-544. The baseline is 562fe85, which passed all five jobs in CI run 38060826982
and was promoted to main. The pinned specification is unchanged.

## Formatting errors (D-542)

Unbounded direct/projected f-strings reported E1010; aggregates containing
unbounded parameters reported E0900. Debug conversion or a width spec could
also add E2250. STD-9, LEX-19 and TYP-17 require a missing-capability E2040.

The existing capability walk now returns its first unformattable component.
Both printing and f-strings use it for a shared diagnostic, so missing bounds
inside arrays, tuples, Option and user structs can name their cause. Explicit
Debug opt-outs retain their explanation. Transparent Box/Cell/reference values
retain Display-or-Debug advice; aggregate elements and fields recommend Debug.
Unsupported concrete formatters retain E0900. Existing acceptance rules are
preserved; this batch does not redesign Display versus Debug dispatch.

Format-spec checking still runs with an error type after a missing capability.
This suppresses a dependent E2250 without losing an independent malformed-spec
E0100. Tests caught and corrected an interim version that skipped all spec
checking. ADR-147 explains the shared traversal.

## Imported projections (D-543)

This concern was reproduced, not inferred merely from index-based code.
Imported TextValue[S] contains a public S.Item field with text bounds. A free
function could format it, but a generic-owner method rejected plain, Debug and
padded formatting because its parameter slots differed. The analogous imported
DecimalValue[S] lost its declared Float bound and rejected `+ 1.0`.

Formatting predicates and float_param now use existing generic_bounds(index,
name) recovery. Tests execute both free and owner methods with concrete text,
int, f32 and f64 values. A rejection control proves that text bounds on unrelated
owner/caller parameters do not grant a projection those capabilities.

## Right-hand generic comparisons (D-544)

Comparison permission is checked for generic operands on either side. Missing
Eq/Ord produces one E2040; literal-left Float controls retain IEEE comparisons,
NaN behavior and signed zero. When an operand already has the error type, the
bound check adds no cascade. Unknown-name tests cover both operand orders.
Concrete comparisons bypass the generic-bound lookup.

## Verification and scope

Thirteen new test programs plus one imported support fixture are included.
The [recorded results](formatting-followups-results.json) show eight failing
baseline regressions and five passing controls, then all thirteen passing with
the candidate. The final surrounding sweep checked 143 programs without a
compiler-case failure, but flagged the newly created DIA-14 directory's missing
accept case. A meaningful bounded-comparison/formatting acceptance control was
added, and the corrected DIA-14 pair passed: 144 distinct nearby programs are
covered in total. No test was disabled or weakened.

All twelve repository gates passed, including specification examples and error
pages; Appendix generation left the specification bytes unchanged. Two scoped
read-only reviews checked bound recovery, traversal equivalence, diagnostic
cascades and transparent-wrapper advice. Their actionable findings have tests.
The full workspace suite runs in CI before promotion, per the owner's request;
no local full conformance sweep was launched.

All 50 existing release benchmarks emit byte-identical C with the baseline and
candidate. Runtime and standard-library sources are unchanged, so no runtime
work was added to those benchmarks. This is code-identity evidence, not a new
timing claim; README figures retain their existing measurements. Raw checks,
compiler snapshots, generated C and gate logs are under
`build/formatting-20261010/`.
