ODR-020 FIX — 0.9.7_Hardened_4

We are working from Ember 0.9.7_Hardened_3.

Close ODR-020 completely.

This is a diagnostic/name-resolution hardening change only. It MUST NOT change:
- source-language validity;
- module/import semantics;
- visibility semantics;
- overload resolution;
- type checking;
- ownership/lifetime semantics;
- ABI;
- runtime behavior.

ODR-019 MUST NOT be reopened or modified by this work.

Problem
-------
Consider:

    import support.io as io

    fn main():
        io::pritn(1)

where `support.io` exports:

    pub fn print(...)

The qualified path resolves through the namespace alias `io`, but the final segment `pritn` is unresolved.

The current specification does not define how N1 applies to qualified paths:
- whether spelling similarity is computed against the whole path or only the unresolved final segment;
- whether the suggestion is displayed as `print` or `io::print`;
- whether the fix-it replaces the whole path or only the misspelled segment;
- how visibility under `[MOD-2]` constrains the candidate set.

Owner decision
--------------
Adopt the following policy:

1. For an unresolved qualified path, N1 computes spelling similarity ONLY against the unresolved final identifier.
2. The namespace/module prefix remains fixed.
3. The candidate set is restricted to items actually visible through that resolved namespace/module path under the existing module/import/visibility rules.
4. The user-facing suggestion is rendered with the full qualified path.
5. The machine-applicable fix-it replaces ONLY the unresolved final identifier.

Canonical example:

    io::pritn(1)

suggests:

    io::print(1)

The fix-it replaces:

    pritn

with:

    print

It MUST NOT replace:

    io::pritn

with:

    io::print

by rewriting the complete path.

The existing `io` alias therefore remains untouched.

Qualified-path resolution and N1
---------------------------------
Define a qualified-path N1 case without creating a second diagnostic system.

For a path of the form:

    <prefix>::<name>

where `<prefix>` resolves successfully but `<name>` does not:

1. Resolve `<prefix>` using the existing module/namespace lookup rules.
2. Enumerate only names that are eligible for lookup within that resolved namespace under existing visibility rules.
3. Apply the existing N1 spelling-distance threshold to `<name>`, not to the complete textual path.
4. Rank candidates using the existing N1 ranking rules, including the ODR-019 cross-file behavior if that behavior is already present in the source revision being amended.
5. Render each suggestion using the original resolved prefix plus the candidate final name.
6. Generate a fix-it that replaces only the unresolved final identifier token.

Do NOT compare:

    io::pritn

against:

    io::print

as complete strings.

Compare:

    pritn

against:

    print

and then render the result as:

    io::print

Visibility
----------
The candidate set MUST respect `[MOD-2]` and all existing visibility/import rules.

For:

    import support.io as io

only names that the current program is legally allowed to access through `io` may be considered N1 candidates.

In particular:

- private items in `support.io` MUST NOT appear as typo suggestions;
- `pub(package)` items MUST NOT appear to external packages;
- `pub(package)` items MAY appear where existing visibility rules already permit them;
- public exported items remain eligible;
- the implicit prelude namespace continues to follow the existing prelude rules;
- candidate discovery MUST reuse ordinary name-resolution visibility rather than inventing diagnostic-specific visibility.

An inaccessible symbol MUST NOT leak merely because its spelling is close to the typo.

Namespace-prefix behavior
--------------------------
If the prefix itself resolves successfully and the final segment is unknown:

    io::pritn

the diagnostic is attached to the unresolved final segment:

    pritn

not to:

    io

and not to the entire path.

The source span for the primary unresolved-name diagnostic MUST cover the final identifier token only.

If the prefix itself fails to resolve:

    ioo::print

then this is an ordinary unknown-name/path resolution failure for the prefix.

Do NOT attempt qualified-final-name suggestions until the prefix has successfully resolved.

Example:

    ioo::print

must diagnose `ioo`, not search every module in the program for a possible `print`.

Suggestion rendering
---------------------
For a successfully resolved prefix:

    io::pritn

and candidate:

    print

the primary suggestion MUST be rendered as:

    io::print

The diagnostic may additionally state that only the final component is being replaced, but the canonical machine-applicable replacement MUST be token-local.

For nested qualified paths, preserve the same rule:

    support::io::pritn

becomes:

    support::io::print

with only `pritn` replaced.

Do not flatten or rewrite the complete path.

Aliases
-------
An import alias is part of the source path and MUST be preserved exactly.

Example:

    import support.io as io

    io::pritn()

suggests:

    io::print()

not:

    support::io::print()

The suggestion is expressed in the source spelling visible to the programmer.

The canonical declaration/module identity may be used internally for ranking and diagnostics, but it MUST NOT replace the user's alias in the fix-it.

`from` imports
--------------
When an item is imported directly:

    from support.io import print

and the source contains:

    pritn()

the existing unqualified N1 behavior continues to apply.

Do NOT force a qualification such as:

    support::io::print

merely because the declaration originated in another module.

ODR-020 applies to the unresolved final component of an already-qualified namespace path.

Candidate ranking
-----------------
Do not create a new ranking algorithm.

Qualified-name N1 MUST reuse the existing N1 ordering after candidate discovery.

The only qualified-path-specific behavior is:

    resolved prefix + final-segment candidate set + final-segment spelling comparison + final-segment fix-it

Everything else remains inherited from N1.

If ODR-019 has already been applied to the active source revision, its same-file/cross-file ordering rules MUST continue to govern tied candidates.

If ODR-019 is not yet present in the active source revision, do not independently redefine cross-file ranking here. Record the dependency clearly rather than duplicating the policy.

Top-N
-----
The existing N1 maximum suggestion count MUST remain unchanged.

The ranking is performed over the visible candidate set for the resolved namespace, then the existing top-N limit is applied.

Do not increase the number of suggestions because the path is qualified.

Diagnostics
-----------
Do NOT create a new diagnostic code.

This remains N1 / the existing unknown-name diagnostic family.

The diagnostic MUST:

- point at the unresolved final identifier;
- display the qualified replacement;
- provide the token-local fix-it;
- respect existing N1 suggestion-count and ranking rules.

Example shape:

    error[E1060]: no member `pritn` in namespace `io`
      help: did you mean `io::print`?
      fix: replace `pritn` with `print`

The exact wording MUST follow the existing E1060/N1 diagnostic conventions rather than creating a parallel qualified-name message format.

Important:
The displayed suggestion is qualified; the replacement span is not.

Source semantics
----------------
This change MUST NOT alter name resolution itself.

A source program that previously failed because `io::pritn` does not exist MUST still fail.

A source program that previously succeeded MUST continue to succeed.

The only change is the quality and determinism of the diagnostic emitted for an already-invalid qualified name.

Specification placement
------------------------
Add the new normative text to the existing N1 diagnostic/name-resolution section.

Do not create an independent "qualified-name suggestion" subsystem.

The authoritative rule MUST specify:

- prefix resolution first;
- final-segment candidate lookup second;
- visibility filtering;
- final-segment spelling comparison;
- existing N1 ranking;
- qualified rendering;
- final-token fix-it.

Summaries, examples, help output, and generated documentation MUST reference that authoritative rule rather than restating conflicting versions of it.

Conformance
-----------
Add or update conformance coverage for:

1. qualified namespace typo:
       io::pritn
       -> io::print

2. diagnostic span covers only `pritn`;

3. fix-it replaces only `pritn`;

4. alias `io` is preserved;

5. nested qualification:
       support::io::pritn
       -> support::io::print

6. private candidate is not suggested;

7. `pub(package)` candidate is suggested only where visibility permits;

8. public candidate is suggested;

9. multiple visible candidates are ranked by existing N1 rules;

10. top-N truncation remains unchanged;

11. prefix typo:
       ioo::print
    diagnoses `ioo` and does not perform final-segment lookup;

12. no matching final-segment candidate produces the normal N1 unknown-name behavior;

13. direct `from` import continues to use ordinary unqualified N1;

14. aliases remain unchanged in suggestions and fix-its;

15. repeated compilations produce identical suggestion ordering;

16. inaccessible names never leak through diagnostic suggestion generation.

Add a regression specifically proving that an inaccessible symbol with a perfect spelling match is not suggested.

For example:

    // support.io
    private fn print(...)

must NOT cause:

    io::pritn

to suggest `io::print` when `print` is not legally visible through `io`.

Implementation constraints
--------------------------
Reuse the existing resolver's namespace/member candidate set.

Do not create a separate "diagnostic symbol table" with different visibility semantics.

Do not scan the entire dependency graph once the prefix has already resolved.

Do not use filesystem traversal as candidate ordering.

Do not use raw source offsets from unrelated files for qualified-name spelling.

The implementation may cache the qualified candidate set, but that cache MUST be derived from the existing authoritative module/name-resolution facts and invalidated with those facts.

Simplicity requirement
----------------------
Follow Ember's established simplicity principle:

- one name-resolution model;
- one N1 ranking model;
- one visibility model;
- one fix-it representation.

Do not introduce:
- a second qualified-name similarity algorithm;
- a separate imported-name diagnostic engine;
- a diagnostic-only visibility system;
- a new public language abstraction.

The qualified-path rule should be a thin specialization of N1 around the already-resolved prefix.

Versioning
----------
Record this revision as:

    0.9.7_Hardened_4

Classification:

    Hardening / diagnostic and tooling completion

Do NOT change the Ember language version.

The change log MUST explicitly state:

- ODR-020 is closed;
- qualified N1 suggestions compare only the unresolved final segment;
- candidate discovery respects existing visibility rules;
- suggestions display the full qualified source path;
- fix-its replace only the unresolved final identifier;
- namespace aliases are preserved;
- no source-language accepted/rejected program set changes;
- no module/import semantics change;
- no runtime or ABI change occurs.

Do not renumber or rewrite historical 0.9.7_Hardened_1 through _3 entries.

Mechanical review
-----------------
Before finalizing 0.9.7_Hardened_4, check for:

- stale wording that compares complete qualified paths for N1;
- diagnostics that highlight the namespace prefix instead of the final token;
- fix-its that replace the entire qualified path;
- suggestions that expose private or otherwise inaccessible symbols;
- duplicate N1 ranking definitions;
- qualified-name-specific ranking algorithms;
- documentation that changes candidate discovery semantics;
- stale version references to H2/H3 where H4 is required;
- accidental ODR-019 redefinition.

Deliver
-------
1. Completed ODR-020 specification change.
2. Updated authoritative N1 rule.
3. ODR-020 owner-decision/change-log entry.
4. Conformance/test mapping and fixtures.
5. Updated diagnostic/help documentation where required.
6. Implementation checklist.
7. Mechanical consistency report confirming no language-semantic changes.

Do not make any additional language-design decisions outside this ruling.