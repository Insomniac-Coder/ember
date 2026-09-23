# Implementing Ember 0.9.9

**State as of 2026-09-23.** On 2026-09-23 the owner directed that the compiler be moved to the 0.9.9
language: "start working on the language development, add/modify/update the current implementation",
continuing until the language is complete or the owner says stop. This document is the plan and the
progress record for that work. It is not a second specification.

## 1. Target and authority

* **Target:** `docs/spec-source/Ember_v0.9.9_Hardened_4.md` (926 rules), pinned by hash in
  `docs/spec-source/development-target.json` so `tools/rule_index.py` accepts its rule
  citations. Its sources (`parts/`, and the frozen `parts-h1/`–`parts-h3/`), the three review
  passes, the change list, the sanity check and the build tools are under `tasks/spec-0.9.9/` on
  this branch, which is their canonical copy; `tasks/spec-0.9.9/HANDOFF.md` records the spec
  process. H1–H3 are frozen; a later hardening diffs against the frozen predecessor.
* **Tools for this work:** `tasks/impl-0.9.9/survey.py` runs `ember check` over every test file
  (about a minute) and lists what a change breaks; `tasks/impl-0.9.9/migrate_literals.py` applies
  the compiler's ODR-022 annotation help to a file.
* **One language.** `[VER-8]`: before 1.0 there is exactly one language, the current one. The
  compiler already implements one evolving language (the `#! language` directive is validated but
  never consulted), so each 0.9.9 change is made directly and the conformance corpus is migrated in
  the same slice.
* **Ambiguities.** Each one found while implementing becomes an **ODR** (next: ODR-021) in
  `docs/OWNER-QUEUE.md`. For 0.9.9 the owner has delegated the ruling: the ODR records the
  conflicting text, the options and the decision, and the decision is published as the next hardening
  of 0.9.9 (Hardened_3, Hardened_4, …), listed in its Appendix H. The compiler never moves the text
  silently, and never adopts a reading because it matches what is built.
* **Process.** `docs/HANDOFF.md` §0.0 governs: probe first, the five-way sort, the four-document
  write path for a defect, break every new test once, claim discipline.
* **Branch.** Work happens on `phase-0.9.9` (CI runs on `phase-*`) in the worktree `Code/ember-099`, because the main checkout
  holds another agent's uncommitted `compiler/ember_analysis` work. Owner, 2026-09-23: commit and
  push about every five features, with CI green.

## 2. Probe results, 2026-09-23 (compiler at `761ce7c`)

| Probe | 0.9.9 rule | Result |
|---|---|---|
| `println("hello")` at file scope | `[GRM-2]`, `[FN-8]` | `E0100 expected a declaration` |
| `7 // 2`, `-7 // 2`, `-7 % 2` | `[TYP-28]` | `//` does not lex as an operator |
| `x = 7 / 2` on integers | `[TYP-28]` (`E2240`) | accepted, prints `3` |
| `x: int`, `z: float` | `[TYP-1]` | `E1010 cannot find type int` |
| comprehension, `{}` map, `{1, 2}` set, `0 <= q < 10` | `[TYP-38]`, `[GRM-25]`–`[GRM-27]` | comprehension does not parse |
| `return 1, 2`; `a, b = b, a` | `[GRM-29]` | unparenthesised tuple does not parse |
| `f"{x=} {x:>8} {name!r}"` | `[LEX-19]` | `a format spec is not supported yet` |
| `Node("a")` with a `String` field | `[TXT-9]` | `E2020 expected String, found str` |
| field default `children: Array[Node] = []` | `[TYP-38]`, `[STR-2]` | `E2060` (the field type does not reach the literal) |
| class holding `Array[Node]`, then `self.children.push(c)` | Part VIII example | **compiler stack overflow** (D-182) |
| enum `Color.Red` with `match` | `[GRM-24]` | works |

## 3. Plan

Slices are ordered so that each one makes more of the specification's own examples compile, first
the programs of `[TST-8]` (a first week of Python-style code), then the Parts in dependency order.
Every slice: probe → sort → implement → conformance cases (each broken once) → ledgers → all tests
and gates green.

**M0 — foundations**
1. D-182: recursive nominal types overflow the type walks. **Done.**
2. Baseline: the full workspace suite on the branch (green, 12m50s). **Done.** The development
   target manifest now pins 0.9.9_Hardened_3. **Done.**

**M1 — the first program** (Parts II–VI, XV basics)
1. `int`/`float` prelude aliases; untyped literals default to `int`/`float` (`[TYP-1]`, `[LEX-16]`,
   `[LEX-17]`) — migrates every case that relied on the `i32`/`f32` default. **Done** (ODR-022;
   expected types reach generic calls and constructors; `W2015` per `[LEX-17a]`).
2. `//`, floor `//` and `%` on signed integers, integer `/` is `E2240` (`[TYP-28]`), float floor
   operations (`[TYP-29]`). **Done** (ODR-021).
3. String literals initialise `String` (`[TXT-9]`). **Done.**
4. Unparenthesised tuples (`[GRM-29]`). **Done.**
5. Collection literals and comprehensions, `{}`/set literals, field defaults take the field type
   (`[TYP-38]`, `[GRM-26]`, `[GRM-27]`, `[STR-2]`); chained comparisons (`[GRM-25]`) **done**
   (a middle operand that is not re-readable is `E0900` for now: bind it to a local). `Map` and
   `Set` do not exist yet (COLD-START: later-phase work), so their literals wait for them.
6. Scripts: top-level statements in the entry file (`[GRM-2]`, `[FN-8]`). **Done** (a script
   using `?` still needs `AnyError`, so it gets `E2180` for now).
7. f-string specs, `=`, `!r` (`[LEX-19]`); `print`/`println` `sep`/`end` **done**, `Display` →
   `Debug` fallback (`[STD-9]`), float `Display` shortest round trip (`[STD-20]`).
8. `Map` iterates keys, `items()`, `values()` (`[CTL-1]`, `[STD-16]`).
9. Prelude built-ins `len`, `range`, `sum`, `sorted`, `enumerate`, `zip`, `reversed`, `any`, `all`
   (`[STD-26]`); generator expressions (`[GRM-38]`); adapters on `Iterable` (`[STD-19]`).
10. `T` → `Option[T]`, `None` inference (`[TYP-5]` rule 11, `[TYP-23]`); implicit `Ok(())`
    (`[FN-10]`); `Result[T]` default error (`[ERR-1]`).

**Next batch** (from `docs/AUDIT-0.9.9.md`, defects first):
1. `-x as T` binds as `(-x) as T` (parser binding powers).
2. `==`/`!=` field-wise for `str`, `String`, structs, tuples, arrays and enums (`[STR-5]`, D-187).
3. The ternary `x if c else y`.
4. `[TYP-31]`: `len()` is `int`, any integer index, `E2011` for a negative literal index.
5. Removed constructs: `::` is `E0100` with `use '.' for paths`; `#! language` other than the
   current version is `E0006`; `@latebound` and `@thread_local` rejected, with the corpus migrated.
6. Prelude: `mem`, `panic`, `todo`, `unreachable`; `Option.unwrap`/`unwrap_or`.

**M2 — classes and exclusivity** (Part VIII): two-phase init (`[CLS-11]`), per-field access words
(`[EXC-19]`), `@sync`, `Weak` upgrade, callable fields.

**M3 onward** — effects and contracts (X), concurrency (XI), data-oriented design (XII), errors
(XIII), compile time (XIV), standard library (XV), C interop (XVI), toolchain (XVII), then the
annexes. Each Part gets its own probe sweep when it becomes current; the sweep's table is added here.

## 4. ODRs raised by this work

| ODR | Question | Ruling | Hardening |
|---|---|---|---|
| ODR-021 | float `//`/`%`: `[TYP-29]`'s formula in IEEE arithmetic, or Python's result? | Python's: exact floor modulo rounded once, consistent quotient | H3 |
| ODR-022 | does a later use change the type of `x = 0` (`[LEX-16]` vs `[TYP-23]`)? | no: the declaration fixes `int`; `E2020` help names the annotation | H3 |

The next number is ODR-023.

## 5. Progress log

* **2026-09-23 — D-182 fixed.** `TypeTable::is_generic` and `has_unsized_by_value` walked the
  fields of structs, classes and enums with no cycle guard, so any type that reaches itself through
  an owning indirection (`struct Tree: kids: Array[Tree]`, the Part VIII `Node` example) overflowed
  the compiler's stack as soon as a method was called on the `Array`. Both walks now carry a visited
  set of the nominal types being examined, and `Array`/`Box` element drops go through out-of-line
  glue. The specification was right; only the compiler moved.
* **2026-09-23 — M1 slices 1 and 2.** `int`/`float`; literals default to `i64`/`f64`; `//`, `//=`,
  floor `%` (runtime `ember_ck_floordiv_*`/`ember_ck_floorrem_*`, Python-exact float helpers);
  `E2240` with its page; `W2015` for literals given `f32`/`f16`; the overflow panic names `//`.
  Found and fixed on the way: D-183 (nested literal typing, printed `-14` for `-7.5 * 2.0`),
  D-184 (the harness ignored `#$ error[CODE]` without a message), D-185 (calling a captured
  callable). Inference: an untyped literal argument waits for the typed arguments and the expected
  result, in generic calls, generic constructors and `Arena.alloc`. Two rulings, ODR-021/022, cut
  0.9.9_Hardened_3. Corpus migration: 45 files, each through the compiler's own help or by hand
  (annotations on literal locals, `//` for integer `/`, `Array[int]` where a loop counts in
  `int`); `tasks/survey.py` runs `ember check` over the whole corpus in about a minute.
* **Known gap, not yet addressed:** `[TYP-10]` says a shift amount out of range panics in every
  profile; `@overflow(wrap)` still masks it (`tests/run-pass/integer_semantics.em` prints `2` for
  `1 << 33` under it). The rule is clear, so this is compiler work for the `[TYP-8]`/`[TYP-10]`
  slice, not an ODR.
* **2026-09-23 — M1 slices 3–5.** `[TXT-9]`: a string literal where a `String` is expected is a
  one-part f-string, so it allocates through the existing lowering. `[GRM-29]`: `return x, y`,
  `a, b = b, a`, `t = 1, 2`, `one = 5,`. `[GRM-25]`: comparison chains, desugared to `and` in the
  type checker; `E0102` now also covers `is`/`in` in a chain (0.9.9 `[GRM-23]`), and
  `GRM-23/reject_chained_comparison_keeps_its_own_code.em` was retired because the rule it tested
  no longer exists. `E0900` registered (`[CLI-19]`). The directory tests collect every failing
  case before reporting. Drop glue is out of line only for a type that owns itself.
* **2026-09-23 — M1 slices 6 and 7a.** Scripts: the parser gathers file-scope statements into a
  synthesised `fn main()` (`Module::script` records where each statement stood, so `ember fmt`
  reprints the script unchanged and idempotently); the driver rejects them outside the entry file
  (`E0100`), and a declared `main` beside them is `E1030`. `print`/`println` take several arguments,
  `sep=` (a string literal for now, else `E0900`) and `end=`: the checker produces one `print`
  builtin of pieces, and MIR evaluates every argument before printing each piece with its typed
  printer. `return a, b` makes a tuple only outside brackets, where `[LEX-6a]` lets a `,` end a
  colon-bodied lambda.
