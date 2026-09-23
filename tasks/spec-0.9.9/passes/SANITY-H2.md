# Sanity check — 0.9.9_Hardened_2

**Question:** does Hardened_2 hold together after the 44 changes of `CHANGES-H2.md`: is every rule
well-formed and cited correctly, does every added or changed rule agree with the rules it did not
touch, and did applying the list open a new hole?

**Document:** `docs/spec-source/Ember_v0.9.9_Hardened_2.md` (7.6k lines, 926 rules), built from
`tasks/spec-0.9.9/parts/`. Hardened_1 is frozen (`parts-h1/`, `Ember_v0.9.9_Hardened_1.md`).

## 1. End checks

| Check | Tool | Result |
|---|---|---|
| rule ids: each defined once, never reused from 0.9.8 for a new meaning, new ids marked | `check_ids.py` | 926 defined, 0 problems |
| every cited rule id is defined | `check_citations.py` | 0 undefined |
| no rule defined in the middle of another bullet | `split_inline.py` | clean |
| diagnostic codes: every code named is registered, every registered rule exists | `code_registry.py` | 238 live, 12 retired, 0 problems |
| findings F-001..F-214 each resolved, cited rules exist | `check_findings.py` | 214 mapped, 0 problems |
| every ` ```ember ` block parses (0.9.8-era parser after `--desugar`) | `check_examples.py --desugar` | 46 blocks, 0 failing |
| every rule whose text differs from Hardened_1 is marked *(new in 0.9.9)* or *(changed in 0.9.9)* | the marker check (`appx_h._bullets`) | 0 unmarked |
| Appendix H §H.4 lists each change | `appx_h.py` | 13 rules added, 78 changed, 0 removed |

`check_examples.py` gained three desugarings for syntax new in Hardened_2 (`safe fn`, generator
expressions, top-level statements lifted into `main`) and Annex A gained a generator expression, so
each new construct is exercised by at least one block.

## 2. Read-through

Method: every added and changed rule (the §H.4 list) was read against each rule that cites it or
that it cites, and against the rules on the same subject that it did not change: the object layout,
the thread rules, the closure rules, the prelude, the non-goals, the tiers and Appendix E. 14 defects
were found. All are fixed in the built document.

| Id | Sev | Defect | Fix |
|---|---|---|---|
| SH-01 | S1 | `[TIER-1]` names an `unsafe overlay` boundary, but since Hardened_1 the overlay grammar had no `unsafe` (lost from 0.9.8's `[FFI-2a]`). With C-05 making an overlay entry assert "safe to call", an overlay could assert safety with no `unsafe` marker at all | `[GRM-35]`: `["unsafe"] "overlay" ("c" \| "cpp")`; an overlay stating any fact MUST be `unsafe overlay` (`E5067`); only `rename`/`hide` overlays need none; `[FFI-2]`, `[TIER-1]` and the Vulkan example updated |
| SH-02 | S1 | per-field access state (C-30) let a base class's `mut self` method mark only the base's fields; a derived override reached from it covers its own field accesses (`[EXC-15]`), so a write to a derived field through another handle went unchecked — a use-after-free the whole-object word of Hardened_1 prevented | `[EXC-19]`: `mut self` marks every field word of the object's **dynamic** class, through the access-word list in the type information for an `open` class; a `mut self` call on `self` is a reborrow (`[EXC-5]`) |
| SH-03 | S2 | C-19 named `debug_assert` in `[PRF-1]` only; `[PHIL-13]` still said no profile changes whether a program panics | `[PHIL-13]` names `debug_assert` as the one check a profile turns off |
| SH-04 | S2 | `[CLO-4]` (a borrowing lambda passed to an `owned` parameter follows the view rules) contradicted C-38's `[CLO-15]`; `[CLO-7]` still told callers to write `owned fn` | `[CLO-4]` excepts `[CLO-15]`; `[CLO-7]` points at it |
| SH-05 | S2 | `SyncShared` (C-02) was missing where `Shared` is listed: `Weak` of it (`[HEAP-7]`, `[WK-11]`), atomic counts (`[RC-4]`, the header layout), `@must_drop` storage (`[THR-6]`) | added to each; `[THR-6]` also cites `[CORO-13]` |
| SH-06 | S3 | `thread.spawn(f: owned fn() -> R)` and `jobs.submit(f: owned fn())` are not grammatical (`fn_type` has no `owned`) | `owned f: fn() -> R` |
| SH-07 | S3 | per-object wording left after C-30: Part I's safety table and example note, `[THR-1]`'s "header access word", `[CORO-12]`, `[CLS-7]` | reworded per field |
| SH-08 | S3 | `[MOD-3]`'s "a module of the package with the same first name" cannot occur, since module paths start with the package name (`[MOD-1]`) | the package itself or a dependency of that name wins, with `W1003` |
| SH-09 | S3 | `[SIMD-7]`'s MUST clause was spliced into the wrong place of the sentence | rewritten |
| SH-10 | S3 | C-06 proposed `E2221`, which already reports a borrow held across `yield` | `E2231`; `CHANGES-H2.md` and `PASS1` updated |
| SH-11 | S3 | Appendix G still resolved F-123 with "scalar-only functions are safe to call" | now "`safe fn` with a complete contract" |
| SH-12 | S3 | `[CLS-10]`, `[EXC-15]`, `[STD-19]` were marked *changed* though 0.9.8 never had them; `[TIER-1]`, `[JOB-2]`, `[THR-6]`, `[RC-4]`, `[WK-11]`, `[HEAP-7]` changed without a marker | markers corrected; the marker check is now part of the end checks |
| SH-13 | S3 | the callable-field row of §VIII.3 covered every callable field, but a capture-free `extern "C" fn` field is `Copy` and has no access word | the row covers owned callable values |
| SH-14 | S3 | scripts (C-32) left `return` and `yield` at file scope unspecified, which the Silence rule would turn into `E0901` | `E0100`, as in Python; `process.exit(code)` ends a script early |

## 3. Checked and consistent

* The thread routes of `[THR-13]` — `@sync` objects, `SyncShared`, statics, scoped borrows, moved
  values — now match `[THR-8]`/`[THR-9]`, `[HEAP-10]`, `[THR-10]`, `[JOB-2]` and `[FFI-22]`.
* Two-phase construction (`[CLS-2]`, `[CLS-4]`, `[CLS-10]`, `[CLS-11]`) against the C++ trampoline
  constructors of Annex C (`[FFI-39]`, `[FFI-39b]`): the order is compatible.
* `T` → `Option[T]` (`[TYP-5]` rule 11) never nests, never applies at operators, and composes only
  after rules 1–4; `range`'s optional `stop` relies on it as intended.
* The prelude functions (`[STD-26]`) against `[STD-13]`, `[DIA-21]`, the non-goals (no overloading,
  no variadics: every form is default parameters) and Appendix E.
* `Map` key iteration (`[CTL-1]`, `[STD-16]`) against `k in m`, `[STD-8]`, comprehensions, `[TYP-39]`
  and the migration list of `[CLI-20]`.
* The foreign-call safety chain: `[FFI-1]` (safe-to-call is a fact) → `[FFI-2]` (never derived) →
  `[FFI-10]` (`safe fn`) / `[GRM-35]` (`unsafe overlay`) → `[TCB-1]` (asserted grade) → `[TIER-1]`
  (exactly three boundaries).
* The cost table (`[COST-5]`): each implicit cost this revision adds has a row — per-field access
  words, stack probes, contraction off, 64-bit `int`, float sorting.

## 4. Left open, by design

* **Implementation.** Every construct new in Hardened_2 is specified, not implemented: the examples
  pass the 0.9.8-era parser only after desugaring. The implementation work is tracked outside this
  document (`[CLI-19]` lists what a compiler does not yet implement, as `E0900`).
* **Hot reload and per-field words.** Adding access words changes a class's layout. Annex B's layout
  rules already treat that as the compiler's layout, which is migrated like any field change, but
  they were not re-derived for it here.

**Verdict.** Hardened_2 is consistent as far as the end checks and this read-through reach, and closes
the Pass 1 holes without opening new ones: the two it would have opened (SH-01, SH-02) are fixed.
