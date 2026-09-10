# Ember Programming Language — Design & Implementation Specification

**Version:** 0.8.5_Hardened_1 (supersedes 0.8.4; see the change log at the end of Part 0)
**Versioning:** two numbers move independently. The **language version** moves when the set of accepted programs changes — 0.8.4 exists because of one such change, S1, the owner's resolution of ERR-044, which admits a static-region view into storage that has no bounding region. The **hardening number** moves when the document gains implementation detail and no rule changes meaning; it resets to 1 with each language version. A hardening may never carry a semantic change: that is what forces the language number instead, and is why this file is 0.8.4_Hardened_1 rather than 0.8.3_Hardened_2.
**Compatibility:** 0.8.5 is additive over 0.8.4, which is additive over 0.8.3. No program valid under either becomes invalid, and a source file may still declare `#! language "0.8.3"` or `"0.8.4"`.
**Hardening:** A hardening adds implementation detail that the revision left out and changes no rule's meaning. It never moves the language version — a change that does is not a hardening, which is why S1 made this 0.8.4 rather than 0.8.3_Hardened_2. Source files may still declare `#! language "0.8.3"` for source compatibility; `#! language "0.8.4"` selects the 0.8.4 revision, and both are accepted (`[MOD-6]`). What Hardened_1 contains is the last change-log section; every edit is marked in place and recorded with its justification in `docs/spec-amendments.md`. The owner's file is preserved untouched at `docs/spec-source/as-received/`.
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

