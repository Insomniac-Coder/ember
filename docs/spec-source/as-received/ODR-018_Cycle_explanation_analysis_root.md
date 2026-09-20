ODR-018 FIX — 0.9.8_Hardened_2

We are working from Ember 0.9.8_Hardened_1.

Close ODR-018 completely. This is a tooling/CLI hardening change only. It MUST NOT change Ember source-language semantics, ownership semantics, accepted/rejected programs, ABI, runtime object semantics, or the existing cycle-analysis model.

Background
----------
The specification currently defines:

    ember inspect --cycle <path>

and:

    ember explain --cycle <Class[.field]>

[CLI-17] already establishes that `ember inspect --cycle <path>` reports the ownership graph used by `[WK-5]`, while `[WK-9]` defines `ember explain --cycle` as the explanation command for one class/field.

The problem is that `[WK-9]` does not specify how the command chooses the program/package to inspect. A class name alone is insufficient because the compiler needs an analysis root. The implementation must not guess by scanning the current directory, consulting an unrelated previous build, selecting an arbitrary package, or otherwise inventing a root.

Owner decision
--------------
Resolve ODR-018 with this canonical syntax:

    ember explain --cycle <path> <Class[.field]>

`<path>` is the analysis root.

The same `<path>` resolution semantics MUST be used by both:

    ember inspect --cycle <path>
    ember explain --cycle <path> <Class[.field]>

Do NOT introduce a new package-selection mechanism.

Canonical analysis-root semantics
---------------------------------
Define `<path>` as exactly one of:

1. A package directory containing `ember.toml`.
2. A single `.em` source file used under the existing standalone-file rules.

For a package directory, the command MUST load the package manifest and use the package's normal module/import closure.

For a standalone `.em` file, use the existing single-file package synthesis already defined by the CLI.

An arbitrary module file inside a package MUST NOT introduce a third interpretation such as “inspect just this module”. The command must not ambiguously interpret a module path as either a standalone source root or a package member.

The resolved analysis universe MUST therefore be deterministic:

    <path>
      -> analysis root
      -> package/module/import closure
      -> ownership graph
      -> selected cycle target
      -> explanation

No cwd scanning.
No stale build fallback.
No prior-build inference.
No arbitrary package selection.
No declaration-proximity selection.

Required specification changes
------------------------------
1. Add a new rule:

[CLI-18] Cycle explanation analysis root.

Required meaning:

`ember explain --cycle <path> <Class[.field]>` MUST construct its analysis universe from `<path>` using exactly the same package/module/import-closure resolution as `ember inspect --cycle <path>`.

The command MUST NOT scan the current directory, consult an unrelated previous build, select an arbitrary package, or infer an analysis root not determined by `<path>`.

`<Class[.field]>` is resolved only within that analysis universe.

If `<path>` cannot be resolved as a valid analysis root, the command MUST fail before ownership analysis and MUST NOT fall back to another root.

2. Amend [CLI-17] so that `<path>` explicitly resolves as the canonical cycle-analysis root defined by [CLI-18].

[CLI-17] must continue to require that `ember inspect --cycle <path>` report the same ownership graph used by `[WK-5]`, including strong/weak/unknown edges and the shortest statically visible cycle.

3. Amend [WK-9] from:

    ember explain --cycle <Class[.field]>

to:

    ember explain --cycle <path> <Class[.field]>

and make it explicitly depend on `[CLI-18]` for analysis-root selection.

`[WK-9]` must continue to require:
- statically visible ownership path(s);
- exact field declarations when a field is selected;
- the complete ownership path containing the selected field;
- instantiated generic ownership types when a generic container is crossed;
- an explicit statement that the edge is dynamically cycle-capable when no static cycle can be established;
- no claim that a runtime cycle has been proved merely because an edge is cycle-capable.

Target-name resolution
----------------------
The target syntax must support:

    Node
    Node.field
    scene::Node
    scene::Node.field

Use Ember's existing `::` qualification for module/type/namespace qualification and `.` for member/field selection.

Do not invent another qualification syntax.

Resolution rules:

- If the selected class does not exist in the resolved analysis universe, use the existing unknown-name diagnostic machinery.
- If the class exists but the selected field does not, use the existing name/member-resolution diagnostic machinery.
- If an unqualified class name is ambiguous within the analysis universe, the command MUST fail rather than choose one arbitrarily.
- The ambiguity diagnostic MUST identify the candidates and provide their qualified names.
- The existing N1-style unknown-name guidance may be reused; do not create a dedicated cycle-specific diagnostic code just for unknown names.
- The command MUST identify the resolved analysis root in an error where doing so materially improves diagnosis.

Example:

    ember explain --cycle ./ragev_scripts Node

    ember explain --cycle ./ragev_scripts scene::Node.parent

    ember explain --cycle ./test.em Node.child

Ambiguous example:

    ember explain --cycle ./pkg Node.value

If both `scene::Node` and `ecs::Node` exist, do not select one. Report ambiguity and show the qualified candidates:

    scene::Node.value
    ecs::Node.value

Diagnostic/code policy
----------------------
Do NOT add a new diagnostic code merely for:
- unknown class;
- unknown field;
- ambiguous class;
- unresolved analysis root.

Reuse the existing name-resolution/CLI diagnostic families.

Add a new diagnostic code only if the existing registry demonstrably has no suitable semantic category for an analysis-root resolution failure. If such a code is genuinely required, place it in the correct CLI/tooling subsystem band, register it, give it an error page, fixture, and conformance mapping. Do not place it in the ownership/borrow diagnostic band.

Do not invent an implementation-specific error code simply to make the rule testable.

Conformance
-----------
Add or update the conformance coverage for this completion.

The tests MUST cover at least:

1. package-directory analysis root;
2. standalone `.em` analysis root;
3. identical ownership graph between `inspect --cycle` and `explain --cycle`;
4. class-only target;
5. class + field target;
6. qualified class target;
7. qualified class + field target;
8. missing class;
9. missing field;
10. ambiguous unqualified class;
11. multiple modules defining the same class name;
12. invalid `<path>`;
13. proof that invalid `<path>` does not fall back to cwd scanning;
14. proof that a stale/unrelated previous build is not used;
15. generic-container cycle explanation;
16. dynamic-cycle-capable edge with no statically proven cycle;
17. existing `[WK-5]`–`[WK-10]` cycle-analysis behavior remains unchanged.

The tests should verify the semantic result and diagnostic behavior, not just command exit status.

Tooling/documentation updates
-----------------------------
Update every authoritative CLI listing and command-surface description so the canonical command is:

    ember explain --cycle <path> <Class[.field]>

Do not leave the old zero-path form anywhere in the active normative CLI surface.

The updated `--help` surface, command tables, rule references, diagnostics documentation, and conformance metadata must agree.

Do not create a second source of truth for CLI semantics. The authoritative rule definitions remain authoritative; summaries and generated documentation must derive from them where the existing tooling supports that.

Change classification
---------------------
This is:

    0.9.8_Hardened_2
    hardening/tooling completion
    ODR-018 CLOSED

It MUST NOT be recorded as a language-version change.

The change log MUST explicitly state:
- ODR-018 is closed;
- `[CLI-18]` defines the canonical analysis-root resolution;
- `[CLI-17]` and `[WK-9]` are amended to use it;
- cycle ownership semantics are unchanged;
- no new ownership/lifetime mechanism is introduced;
- no source-language accepted/rejected program set changes;
- no ABI/runtime semantic change occurs.

Simplicity constraints
----------------------
Follow Ember's simplicity-consolidation rules.

Use one authoritative analysis-root resolution mechanism for both cycle commands.

Do not introduce:
- a separate cycle-analysis package resolver;
- hidden compiler state as the program-selection mechanism;
- a new source-language abstraction;
- a second ownership graph representation with independent semantics;
- a new runtime metadata mechanism.

The implementation may cache or derive analysis-root information internally, but such representations must remain derived from the single authoritative resolution result.

Evidence boundary
-----------------
Do not claim implementation or conformance merely because the specification has been amended.

Keep specification changes, implementation changes, owner-decision records, and conformance evidence separate.

Before finalizing:
------------------
Run a mechanical review for:

- stale `ember explain --cycle <Class[.field]>` syntax;
- duplicate `[CLI-18]` definitions;
- inconsistent `<path>` semantics between inspect and explain;
- missing diagnostics/conformance references;
- accidental language-semantic changes;
- accidental new diagnostic-code reuse;
- incorrect rule-category assignment;
- undocumented ambiguity behavior;
- duplicated analysis-root semantics in multiple normative locations.

Deliver:
1. the completed 0.9.8_Hardened_2 specification change;
2. updated ODR-018 decision/change-log record;
3. conformance/test mapping;
4. all required CLI/help/documentation updates;
5. a concise implementation/conformance checklist.

Do not silently make any additional language-design decisions outside this ruling.