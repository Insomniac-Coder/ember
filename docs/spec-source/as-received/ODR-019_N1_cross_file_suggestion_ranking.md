ODR-019 FIX — 0.9.8_Hardened_2

We are working from Ember 0.9.8_Hardened_1.

Close ODR-019 completely.

This is a diagnostic-ranking hardening change only. It MUST NOT change:
- source-language validity;
- name lookup semantics;
- overload resolution;
- type checking;
- ownership/lifetime semantics;
- runtime behavior;
- ABI;
- module/import semantics.

The only observable behavior being specified is the deterministic ordering of N1 name suggestions.

Problem
-------
N1 currently ranks candidates by:

1. spelling similarity using Damerau–Levenshtein distance;
2. declaration proximity.

Declaration proximity is meaningful only when the candidate declaration and the unresolved name occur in the same source file.

For example:

    app.em:
        pritn(...)

    io.em:
        print(...)

    text.em:
        println(...)

A source offset in `app.em` cannot be meaningfully compared with a source offset in `io.em` or `text.em`, because each file has an independent coordinate system.

The specification therefore needs an explicit cross-file ranking rule.

Owner decision
--------------
Adopt ODR-019 option 1.

Do NOT introduce a project-wide file/module distance model.

Do NOT use filesystem ordering, import-graph distance, declaration byte offsets from different files, or any other invented cross-file proximity metric.

Canonical N1 ranking
--------------------
N1 candidate ranking MUST be defined as follows.

For each candidate:

1. Primary key: Damerau–Levenshtein spelling distance between the unresolved name and the candidate's visible name.

2. Secondary key: locality class.

   a. SAME_FILE:
      The candidate declaration is in the same source file as the unresolved name.

   b. CROSS_FILE:
      The candidate declaration is in a different source file.

   SAME_FILE candidates rank ahead of CROSS_FILE candidates when the spelling-distance key is equal.

3. Tertiary key for SAME_FILE candidates:
   Use the existing declaration-proximity ordering already defined by N1.

   The existing same-file proximity calculation MUST remain unchanged.

4. Tertiary key for CROSS_FILE candidates:
   Do NOT compute declaration proximity.

   Instead, order candidates by the canonical qualified declaration name using a deterministic locale-independent lexicographic ordering.

The canonical qualified declaration name MUST be based on the symbol's canonical module/type/member identity, not:
- the importing file's byte offset;
- filesystem traversal order;
- hash-table iteration order;
- source-file discovery order;
- import statement order;
- operating-system directory ordering;
- build/cache order.

Illustrative ranking
--------------------
Suppose the unresolved name is:

    pritn

and the visible candidates are:

    print       (same file)
    println     (io.em)
    printf      (text.em)

Spelling similarity remains the primary criterion.

Therefore:

- a candidate with a strictly better Damerau–Levenshtein distance MUST outrank a candidate with a worse distance, regardless of file;
- when candidates have equal spelling distance, a same-file candidate MUST outrank a cross-file candidate;
- equal-distance cross-file candidates MUST then use canonical qualified-name ordering.

This means the rule does NOT globally prefer local declarations over better spelling matches.

Cross-file candidates are simply a single deterministic group whose ordering is based on canonical identity rather than source position.

Important:
-----------
The phrase "declaration proximity" MUST no longer be interpreted as permitting comparison of source offsets from different files.

Declaration proximity is a same-file concept only.

Do not add:
- project-wide source offsets;
- virtual concatenated-file offsets;
- module-distance metrics;
- import-distance metrics;
- filesystem-distance metrics;
- arbitrary "locality scores".

Candidate-set semantics
-----------------------
This change MUST NOT alter which candidates N1 considers.

The existing candidate discovery, visibility, scope, imports, aliases, prelude candidates, inherited members, and other lookup rules remain authoritative.

Only candidate ordering is being specified.

Imported and aliased symbols
----------------------------
An imported or aliased symbol declared in another file is classified as CROSS_FILE.

Its cross-file tie-break MUST use the symbol's canonical qualified declaration identity, not the local import statement's source position.

Do NOT create separate ranking mechanisms for:
- imported names;
- aliases;
- inherited fields;
- prelude names.

They all use the same N1 ranking framework.

Canonical-name tie-break
------------------------
The cross-file tie-break MUST be deterministic and independent of locale and implementation traversal order.

Use the canonical qualified declaration name.

For example, conceptually:

    io::print
    text::print
    util::print

must be ordered lexicographically by the canonical qualified name.

The user-facing spelling/display name remains whatever N1 normally uses for suggestions; the canonical qualified name is used only to establish deterministic ordering when required.

Do not use a hash value as the tie-break.

Top-N behavior
--------------
N1 MUST apply the ranking above before selecting its existing maximum number of displayed suggestions.

Do not change the existing N1 suggestion-count limit.

Therefore, the cross-file rule MUST affect which candidates make the top-N set when candidates tie on spelling distance.

Diagnostics
-----------
Do NOT add a new diagnostic code.

N1 remains the authoritative diagnostic.

Do not change:
- diagnostic wording beyond what is necessary to document deterministic candidate ordering;
- unknown-name detection;
- candidate discovery.

The fix is entirely in candidate ranking.

Specification structure
-----------------------
Add the missing rule in the existing N1 diagnostic/ranking section rather than creating a new independent diagnostic subsystem.

The new rule should make these facts explicit:

- spelling similarity is the primary ranking key;
- declaration proximity is defined only for same-file candidates;
- same-file candidates form the higher-locality class for equal spelling distance;
- cross-file candidates do not use source-position proximity;
- cross-file ties use canonical qualified-name ordering;
- candidate selection is performed only after the complete deterministic ordering.

Do not duplicate the complete N1 algorithm in multiple normative sections.

Create one authoritative definition of the ranking algorithm and reference it from summaries or tooling documentation.

Conformance
-----------
Add/update conformance tests for:

1. same-file misspelling with multiple candidates;
2. equal-distance same-file candidates;
3. equal-distance cross-file candidates;
4. same-file vs cross-file candidates with equal spelling distance;
5. cross-file candidate with better spelling distance than same-file candidate;
6. imported candidate;
7. aliased candidate;
8. inherited/member candidate where applicable;
9. prelude candidate where applicable;
10. enough candidates to verify top-N truncation;
11. multiple cross-file candidates whose canonical names determine ordering;
12. repeated builds/invocations produce identical ordering;
13. different filesystem traversal orders do not affect ordering;
14. import/source discovery order does not affect ordering.

The tests MUST verify the actual suggestion ordering, not merely that the diagnostic is emitted.

Important regression tests
--------------------------
Include a case equivalent to:

    app.em:
        pritn

    io.em:
        print

    text.em:
        println

Verify that source offsets from `io.em` and `text.em` are never compared against offsets in `app.em`.

Also include a case where:

- same-file candidate has equal spelling distance to a cross-file candidate;
- same-file candidate ranks first.

And a case where:

- cross-file candidate has better spelling distance than a same-file candidate;
- better spelling similarity still ranks first.

This confirms that locality is NOT incorrectly made the primary key.

Simplicity constraints
----------------------
This fix MUST follow Ember's simplicity principle.

Use one ranking model.

Do not create separate ranking algorithms for:
- local declarations;
- imported declarations;
- prelude declarations;
- inherited declarations.

The only semantic distinction introduced is the necessary one:

    SAME_FILE vs CROSS_FILE

because source-position proximity is meaningful only in the SAME_FILE case.

The compiler may internally represent the ranking as a tuple/key or cached structure, but that representation is derived from the single authoritative N1 ranking rule.

No new public language concept is introduced.

Versioning
----------
Record this as:

    0.9.8_Hardened_2

Classify it as diagnostic/tooling hardening.

Do NOT increment the Ember language version.

The change log MUST explicitly state:

- ODR-019 is closed;
- option 1 was adopted;
- declaration proximity applies only within the same source file;
- equal-distance same-file candidates rank before cross-file candidates;
- cross-file candidates use canonical qualified-name ordering;
- no project-wide file/module proximity model was introduced;
- candidate discovery semantics are unchanged;
- no source program changes validity;
- no runtime, ABI, ownership, or type-system semantics changed.

Mechanical review
-----------------
Before finalizing 0.9.8_Hardened_2, check for:

- any remaining statement implying source offsets can be compared across files;
- any N1 wording that leaves cross-file ties unspecified;
- duplicate definitions of N1 ranking;
- nondeterministic filesystem/import/hash iteration being used as a tie-break;
- stale documentation describing declaration proximity as universally applicable;
- tests that verify only diagnostic presence instead of ordering;
- accidental changes to candidate discovery rather than candidate ranking;
- accidental language-version change.

Deliver
-------
1. The completed ODR-019 specification change.
2. Updated N1 normative ranking rule.
3. ODR-019 owner-decision/change-log entry.
4. Conformance/test mapping and fixtures.
5. Any required diagnostic/help documentation updates.
6. A concise implementation checklist.

Do not make any additional language-design decisions outside this ruling.