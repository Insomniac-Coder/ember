# Ember Programming Language — Specification

**Version:** 0.9.15_Hardened_5
**Supersedes:** 0.9.15_Hardened_3 (development target), 0.9.15_Hardened_2, 0.9.15_Hardened_1 and every
earlier file: 0.9.10_Hardened_1 to
0.9.10_Hardened_35, 0.9.9_Hardened_1 to 0.9.9_Hardened_52, 0.9.8_Hardened_3 and 0.8.5_Hardened_1
(adopted). This document is the single specification of Ember; the earlier files are history.
**Status:** Consolidated language revision. Language rules are complete for the Core and Systems
profiles; the Native and Dynamic profiles are specified in annexes. Implementation and conformance
are tracked separately (§XVII.5, §XVII.8) and are not claimed by this text.
**Authority:** **0.9.15** is the owner's rework of the memory model (`tasks/96/redesign/JOB.md`, the
owner's verdicts of 2026-10-24 to 2026-10-27, with the design in `tasks/96/redesign/run4/v5/` and its
list of unsafe features); **0.9.15_Hardened_2** adds the owner's verdicts 101 to 106 and the lead's
rulings for building it (`tasks/96/redesign/compiler-0.9.15/decisions.md`); **0.9.15_Hardened_3** the
owner's verdicts 107 to 117; **0.9.15_Hardened_4** the owner's verdicts 118 to 124, from the review of
the rulings made building it (an `unsafe fn` unsafe as a whole, with `safe:` blocks; the record
`ember.overlays`; a stored function's effects by the place it is called from). What a program means is written in
words on the thing it describes: one owner unless a line says `shares`; `owns` for a hand-over and a
value held alone; `copy of` for a copy; `ref mut` on a name, and at the call, for a change of someone
else's value; `weakref` for a link that does not keep its target alive; `this` for a method's object.
What a function changes, reads and ends is worked out from its code, never written on its first line.
Every memory rule is checked when compiling; what still runs while a program runs is listed with its
cost (§X.4). It replaces `Box`, `Shared`, `Weak`, `RefCell`, `SyncShared`, `ForeignBox`, `@sync`,
`@borrows`, the mode `owned`, `self`, `mem.forget`, effect clauses on a first line and every check of
exclusive access made while the program runs. It changes the set of accepted programs, so the
language version moves (`[VER-9]`); Appendix H §H.41 to §H.44 list every change. The revisions before it were
authored at the owner's direction of 2026-09-23 with delegated authority to decide open language
questions against the project goal; Appendix H lists each one's changes (§H.1 to §H.40), and every
finding of the 2026-09-23 research pass (`tasks/audit/FINDINGS.md`, F-001–F-214) is resolved in
Appendix G.

---

## The goal, and how this document is judged

Ember is built to meet three requirements at once:

1. **Fast like C and Rust.** Code that uses values, views and plain loops compiles to what a careful
   C programmer would write, with every remaining cost named in §X.4.
2. **Memory safety like Rust's, or better, checked when compiling.** Safe Ember cannot use freed
   memory, race on data, read out of bounds, or leak through a loop of owners. Every memory rule is
   checked when compiling wherever it can be; what still runs while the program runs is listed, with
   its cost, in §X.4. Ember gets there without lifetime names, without trait-import rules, without
   orphan-rule surprises, and without asking the programmer to prove what the compiler can work out.
3. **Readable like Python.** Programs read like Python: indentation, few annotations, inference
   inside functions, literals for lists, maps and sets, f-strings, generators, comprehensions. What
   the code means is written in words on the thing it describes (`owns`, `shares`, `weakref`,
   `copy of`, `ref mut`), and the compiler never decides what the code does not say. Readable means
   how a program reads, not Python's meaning: where Python's meaning and C's differ, Ember takes
   Python's only where that costs neither safety nor speed that cannot be recovered, and says so.
   Safety is never traded for looking like Python. Error messages are part of readability: each
   names what was refused in the program's own words, says why in one line and gives the exact line
   to write, which compiles (§XVII.6).

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
| VII | Ownership: owners, hand-over, views, `ref mut`, destruction |
| VIII | Classes, `this`, shared owners (`shares`) and links (`weakref`) |
| IX | Memory facilities: values held alone (`owns`), memory managers (`using`), arenas, raw pointers, `unsafe` |
| X | Effects (worked out from the code), contracts, determinism, the cost model |
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
| Appendix H | Changes from 0.9.8_Hardened_3 (§H.1 to §H.3), in 0.9.9 (§H.4, §H.5), in 0.9.10 (§H.6 to §H.40) and in 0.9.15 (§H.41) |
| Appendix I | Rule index |

**Normative words.** MUST, MUST NOT, SHOULD, SHOULD NOT and MAY are used as in RFC 2119. Text marked
*Note* or *Example* is not normative.

**Rule identifiers.** Normative rules carry identifiers of the form `[XXX-n]`. A rule identifier is
never reused for a different meaning. A rule keeps its identifier while its meaning stays; a rule
whose meaning a revision changes is marked *(changed in 0.9.N)* with that revision's version, and a
rule a revision adds is marked *(new in 0.9.N)* and takes a number above every number its family has
used before. A rule whose subject a revision removes is listed in Appendix H and its identifier is
never used again. Appendix I lists every rule.

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
* `[VER-3]` *(changed in 0.9.10)* From 1.0: deprecation through `@deprecated(since, note)`, removal no
  earlier than the next major version. Each use of a deprecated item outside its own module is
  `W2192` (use of a deprecated item), naming `since` and `note`.
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
