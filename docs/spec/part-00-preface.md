# Ember Programming Language — Design & Implementation Specification

**Version:** 0.8.3_Hardened_1 **+ S1, pending a version decision** (supersedes 0.8.2c; see the change log at the end of Part 0)
**Pending:** this file carries one **owner-approved semantic change** — S1, the resolution of ERR-044 — which a hardening may not contain, because it changes the set of accepted programs rather than only the document's completeness. It is additive: no program valid under 0.8.3 becomes invalid. The version is therefore **not yet settled**; it will either cut **v0.8.4**, which keeps 0.8.3 exactly as specified and leaves hardening a pure clarification mechanism, or land in **Hardened_2** with that boundary explicitly relaxed. Until then this file is "0.8.3_Hardened_1 plus S1" and says so rather than claiming to be either.
**Hardening:** A hardening adds implementation detail that the revision left out, and changes no rule's meaning and no language version. Source files still declare `#! language "0.8.3"`, because the language did not move — only the document's completeness did. What Hardened_1 contains is the last change-log section; every edit is marked in place and recorded with its justification in `docs/spec-amendments.md`. The owner's file is preserved untouched at `docs/spec-source/as-received/`.
**Lineage:** authored from **0.8.2c**, which was authored from 0.8.2b, which was authored from 0.8.1, which was authored from 0.8, which was authored from 0.7.2, which was authored from 0.7.1, which was authored from 0.6.3 taking the design — not the text — of the 0.7 draft for hot reload, the compile-time budget and the C++ boundary. The 0.7 draft was itself authored from 0.3 and silently reverted 239 rules settled in 0.4 through 0.6.2; every one of those is retained here. **A revision of this document MUST be authored from the immediately preceding revision.**
**Authority:** This document is the sole normative source for Ember. It supersedes all earlier drafts, which are not required to implement anything described here.
**Status:** Implementation-ready specification for the reference compiler, runtime, toolchain and RageV integration
**Audience:** Implementing agents and engineers. This document is written to be executed against, not read for inspiration.
**Reference workload:** [RageV](https://github.com/Insomniac-Coder/RageV) (Windows C++ engine; Vulkan 1.3 + OpenGL 4.5 RHI; sparse-set ECS; render graph; C# scripting via a function-pointer table)
**Principle:** safe by default, provable by request, native when necessary.

---

## How to use this document

1. **Part 0** records the foundational design decisions, the alternatives that were rejected, and why. Read it first; it explains the shape of everything after it.
2. **Parts I–XVII** are the *language* specification. Normative rules carry stable identifiers in the form `[XXX-n]` (e.g. `[OWN-4]`). Every rule with an identifier MUST have at least one test in the conformance suite that references it (see Part XX §5).
3. **Part XIX** specifies the compiler internals (IRs, passes, algorithms, ABI, mangling, backends).
4. **Part XX** specifies the toolchain (CLI, manifest, build graph, test harness, diagnostics format, error-code registry).
5. **Part XXI** is the implementation plan: phases, milestones, acceptance tests, repository layout, and operating instructions for an implementing agent.
6. **Part XXII** is the RageV integration plan, stage by stage, with concrete ABI sketches.
7. **Part XXIII** lists questions that require an owner decision. An implementing agent MUST NOT silently choose an answer to these; it records a provisional choice in `docs/DECISIONS.md` and flags it.

The words **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, **MAY** are normative (RFC 2119). "v1" means the first stable release; "v2/v3" mean later releases. Anything marked **(v2)** or **(v3)** is specified so that v1 does not preclude it, but is not required for v1.

**Guiding sentence for the implementer:** when the specification is silent **on a matter of language semantics**, an implementation MUST NOT infer the answer by analogy with another language. It records a provisional decision in `docs/DECISIONS.md`, marks it as filling a specification gap, and raises the gap — because two implementations reasoning independently from "what a C programmer would expect" reach two languages, and an analogy is not a rule anyone can check. For matters the specification leaves genuinely open — a diagnostic's wording, an internal data structure, an optimisation's aggressiveness — choose the option that is (a) simplest to implement soundly and (b) most predictable at runtime, in that order, and write the choice down.

---

