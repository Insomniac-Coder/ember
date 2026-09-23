# Ember Programming Language — Specification

**Version:** 0.9.9_Hardened_3
**Supersedes:** 0.9.9_Hardened_2, 0.9.9_Hardened_1, 0.9.8_Hardened_3 (development target) and 0.8.5_Hardened_1 (adopted). This document is
the single specification of Ember; the earlier files are history.
**Status:** Consolidated language revision. Language rules are complete for the Core and Systems
profiles; the Native and Dynamic profiles are specified in annexes. Implementation and conformance
are tracked separately (§XVII.9) and are not claimed by this text.
**Authority:** Authored at the owner's direction of 2026-09-23 with delegated authority to decide open
language questions against the project goal. Every decision that changes 0.9.8's meaning is listed in
Appendix H, and every finding of the 2026-09-23 research pass (`tasks/audit/FINDINGS.md`, F-001–F-214)
is resolved in Appendix G. Hardened_2 applies the memory-safety, consistency-and-speed and
ergonomics passes over Hardened_1 (`tasks/spec-0.9.9/passes/`); Appendix H §H.4 lists every change.
Hardened_3 and later record the ODRs raised while implementing 0.9.9 (from ODR-021, ruled under the
owner's delegation); Appendix H §H.5 lists them.

---

## The goal, and how this document is judged

Ember is built to meet three requirements at once:

1. **Fast like C.** Code that uses values, views and plain loops compiles to what a careful C
   programmer would write, with every remaining cost named in §X.4.
2. **Types like Python.** Programs read like Python: indentation, few annotations, inference inside
   functions, literals for lists, maps and sets, f-strings, generators, comprehensions, and errors
   that name the fix. Where Python's meaning and C's meaning differ, Ember takes Python's unless doing
   so costs speed that cannot be recovered, and says so.
3. **Memory safety like Rust, without Rust's ceremony.** Safe Ember cannot use freed memory, race on
   data, or read out of bounds. Ember gets there without lifetime names, without trait-import rules,
   without orphan-rule surprises, and without asking the programmer to prove what the compiler can
   infer.

Each goal has an instrument, and a release is judged by the instruments rather than by this prose:
the performance suite (`[TST-28]`), the first-week newcomer corpus (`[TST-8]`), and the Safe Ember
invariant (`[PHIL-10]`) with its conformance tests.

## How to read this document

| Part | Contents |
|---|---|
| I | Overview, principles, the Safe Ember invariant |
| II | Lexical structure |
| III | Grammar (complete) |
| IV | Types |
| V | Declarations: modules, functions, structs, enums, classes, interfaces, statics, attributes |
| VI | Expressions, statements, closures, generators |
| VII | Ownership, borrowing, regions, destruction, views |
| VIII | Classes and reference counting |
| IX | Memory facilities: heap types, arenas, raw pointers, interior mutability |
| X | Effects, contracts, determinism, the cost model |
| XI | Concurrency |
| XII | Data-oriented programming |
| XIII | Error handling |
| XIV | Compile-time programming |
| XV | Standard library (normative surface) |
| XVI | Foreign function interface (C) |
| XVII | Toolchain, diagnostics and conformance |
| XVIII | Requirements on implementations |
| Annex A | Syntax quick reference |
| Annex B | Hot reload (Dynamic profile) |
| Annex C | C++ interoperation (Native profile) |
| Annex D | GPU host model (library) |
| Appendix E | Coming from Python (non-normative) |
| Appendix F | Glossary |
| Appendix G | Resolution of findings F-001–F-214 |
| Appendix H | Changes from 0.9.8_Hardened_3, from 0.9.9_Hardened_1 (§H.4) and from 0.9.9_Hardened_2 (§H.5) |
| Appendix I | Rule index |

**Normative words.** MUST, MUST NOT, SHOULD, SHOULD NOT and MAY are used as in RFC 2119. Text marked
*Note* or *Example* is not normative.

**Rule identifiers.** Normative rules carry identifiers of the form `[XXX-n]`. A rule identifier is
never reused for a different meaning. Rules carried from 0.9.8 keep their identifiers; a rule whose
meaning this revision changes is marked *(changed in 0.9.9)*; new rules take numbers above every
number the family has used before. Appendix I lists every rule.

**Examples.** Every ` ```ember ` block is a valid program or sequence of items under this
specification and passes `ember check`. A block marked ` ```ember,fragment ` parses but names items it
does not declare; ` ```ember,overlay ` is overlay source (`[GRM-35]`); ` ```ember,ignore ` carries a
stated reason (`[TST-7]`). Blocks marked ` ```text ` are not Ember.

**Silence.** Where this document is silent on a matter of language meaning, an implementation MUST
NOT choose an answer by analogy with another language. It rejects the construct with the
not-yet-specified diagnostic `E0901` (`[PHIL-12]`) and the gap is raised with the owner.

## Versioning

* `[VER-1]` *(changed in 0.9.9)* Three version numbers exist and move independently: the **language
  version** (this document), the **compiler version**, and the **runtime ABI version**
  (`EMBER_RUNTIME_ABI`).
* `[VER-8]` *(new in 0.9.9)* Before 1.0 there is exactly one language: the current one. A source file does not select
  a language version, and `#! language` directives are accepted only if they name the current version
  (`E0006` otherwise, whose help says to delete the line). Selectable language versions begin at 1.0,
  under `[VER-2]`.
* `[VER-2]` From 1.0: source compatibility within a major language version; a breaking change needs a
  new major version that a package opts into with the manifest `language` key.
* `[VER-3]` From 1.0: deprecation through `@deprecated(since, note)`, removal no earlier than the next
  major version.
* `[VER-4]` The runtime ABI (object header, `ember_type_info`, every entry point in `ember_rt.h`) is
  stable within a major version; any change bumps `EMBER_RUNTIME_ABI`. `ember_rt_init` compares the
  runtime's ABI version with the one its caller was compiled against and fails, naming both, on a
  mismatch; fields may be added to `ember_rt_config` only at its end, behind a leading size field.
* `[VER-9]` *(new in 0.9.9)* A language revision that changes the set of accepted programs or their
  meaning moves the language version and resets the hardening number to 1. From 1.0, a hardening adds
  precision without changing any accepted program's meaning (`[VER-2]`). Before 1.0 there is one
  language (`[VER-8]`), so a hardening may change the language, provided its change log lists every
  such change as a change to the language and `ember fmt --migrate` rewrites every program it can. The
  change log of each revision (Appendix H for this one) lists every rule it adds, removes or changes;
  a change without a row is a defect in the document.

