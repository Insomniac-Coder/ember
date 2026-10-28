# Brief: the gap map from today's compiler to spec 0.9.15_Hardened_1 (read this first)

## The goal

Ember (repository `C:\Users\ism19\Code\ember`) is a compiled language: Python-like syntax, compiled to C
(MSVC, Clang or GCC) or to LLVM IR. Between 2026-10-23 and 2026-10-27 its owner reworked the memory
model. There were three goals at once:
- as fast as C and Rust;
- as memory-safe as Rust or safer, checked when compiling;
- readable like Python, with every meaning written in words on the thing it describes.

The result is the spec **`docs/spec-source/Ember_v0.9.15_Hardened_1.md`**, now the compiler's
development target. The compiler still implements the previous spec, 0.9.10_Hardened_35. This job maps
the gap, rule by rule, so the compiler can be brought to the new spec in milestones.

The new model in one paragraph:
- **Ownership words** sit on the value and in its type:
  - `owns`: one owner, and the hand-over, for every type, numbers included;
  - `shares`, `shares mut`: several owners, counted, with a compare against the count's limit before
    each +1;
  - `thread_shared`: several owners on several threads, counted atomically;
  - `weakref`: a link that keeps nothing alive, opened with `if x exists as y:`;
  - `ref`, `ref mut`: views;
  - `copy of`: a copy of your own.
- **Removed:** `Box`, `Shared`, `Weak`, `RefCell`, `SyncShared`, `ForeignBox`, the `owned` type word,
  `@sync`, `@borrows`, `mem.forget`, and every `changes` and `reads` clause.
- **Effects are inferred.** What a function changes, reads and ends is worked out from its body, through
  the whole program, libraries included (kept in interface files across packages). Messages name the
  chain of calls.
- **Compile-time checks only.** Every rule on who may change or read a place is checked when compiling.
  The run-time exclusivity checks and the run-time index compare are gone. Run-time checks stay only
  where the property is data the program computes: an index's bound, an overflow, a `weakref` target, a
  downcast, a count's limit.
- **Loops follow Rust:** no change through a collection's own name while a loop walks it.
- **Unsafe:** a loop of owners is refused unless proven opened again (E3032). `into_raw()` and
  `@unsafe(export(...))` exist only inside `unsafe`.
- **ECS:** EnTT's design (`Registry`, sparse-set pools, `view`/`view_mut`, no scheduler).

## Where everything is

- **The new spec's parts:** `tasks/spec-0.9.9/parts/*.md`, identical to the frozen
  `tasks/spec-0.9.9/parts-0.9.15-h1/`.
- **The old spec's parts:** `tasks/spec-0.9.9/parts-0.9.10-h35/*.md`. A rule is a bullet that starts
  ``* `[FAMILY-N]` ``. Find it in both trees to compare the texts.
- **What changed:** `tasks/spec-0.9.9/parts/p26-appx-h.md`, section `## H.41`. Its table lists every
  rule whose text differs, marked `changed` (409), `added` (84) or `removed` (28).
- **Error codes:**
  - the new table is `tasks/spec-0.9.9/parts/p17a-codes.md` (code, meaning, rules; live and retired);
  - the compiler's registry is `compiler/ember_diag/src/codes.rs`;
  - the error pages are under `docs/errors/`.
- **The design behind the spec** (read when a rule's intent is unclear):
  - `tasks/96/redesign/run4/v5/rules.md` (rules R1 to R18; section 2 is the cost table);
  - `vocabulary.md` (every word, and what replaced each removed one);
  - `changes.md`;
  - `tasks/96/redesign/JOB.md` (the owner's verdicts 1 to 100: **decided; never re-open them**).
- **The design's 841 tests:** `tasks/96/redesign/run4/v5/conformance/R1..R18/`. Each test's `#$ rules:`
  names spec IDs. `tasks/96/redesign/run4/v5/conformance-rule-map.md` lists them by test.
- **The compiler:** `compiler/` (Rust workspace crates):

  | crate | what it holds | size |
  |---|---|---|
  | `ember_lexer` | lexer | |
  | `ember_parser` | parser | |
  | `ember_ast` | syntax tree | |
  | `ember_hir` | high-level IR | |
  | `ember_types` | types | |
  | `ember_typeck` | the checker | 68k lines |
  | `ember_analysis` | borrow, drop and effect analyses, optimisations | 38k |
  | `ember_mir` | MIR and lowering | |
  | `ember_codegen_c` | C backend | 19k |
  | `ember_codegen_llvm` | LLVM backend | |
  | `ember_interp` | the evaluator | |
  | `ember_driver` | CLI, build and tests | 39k; tests in `compiler/ember_driver/tests/*.rs` |
  | `ember_diag` | diagnostics | |
  | `ember_build` | build | |
  | `ember_fmt` | formatter | |

- **The runtime:** `runtime/`.
- **The std library:** `std/src/*.em`, 13k lines.
- **Old tests:**
  - `tests/conformance/<RULE-ID>/*.em`: 3,147 files in 589 folders, one folder per rule;
  - `tests/{compile-fail,compile-pass,run-pass,run-fail,ui,firstweek,perf}/`.

  A test's `#$` lines carry its expectations: `#$ test: compile-fail|run-pass|run-fail`,
  `#$ error[E1234]: message` on the offending line, `#$ stdout:`, `#$ rules:`. The runner that reads
  them is `compiler/ember_driver/tests/milestones.rs`; it refuses unknown keys.
- **Inline Ember programs** in Rust tests (`compiler/*/tests/*.rs`, `compiler/*/src/**` unit tests) also
  use the old spellings.

## The owner's rules that apply to you

- **The compiler must abide by the spec.** Every MUST of a rule gets built completely. Never propose
  building part of a rule, recording a "deviation" instead, or editing the spec to fit the compiler. If
  a rule seems unbuildable or too costly as written, say so as a question for the owner.
- **General solutions only.** Nothing shaped to a benchmark or a single test. Fast as C: say what each
  check costs, and prefer proving a check away to paying for it.
- **Verdicts are settled.** The owner's verdicts in JOB.md are decided. A question for the owner is only
  for a spec text that is unclear, contradicts itself, or cannot be built. Check JOB.md before raising
  one.
- **Read-only.**
  - You may run read-only commands: `grep`/`rg`, `find`, `git log`/`git show`/`git diff` on any branch,
    and small Python scripts that only read.
  - You may write only your own report file (and scratch files under
    `target/gap-0.9.15/<your job number>/`).
  - Never build or run anything: no `cargo`, no `ember`, no C compiler, no program.
  - No git writes. Never use `cmd /c start`.
  - Do not touch `D:/Code_secondary/RageV` or anything outside this repository.

## What you produce

Write your report to `tasks/96/redesign/compiler-0.9.15/gap/<NN>-<area>.md`; your prompt gives the name.

**1. Rule table.** One row per rule of your families that the H.41 table lists, plus any rule you find
the compiler implements under a removed name.

| rule | H.41 kind | what the spec now requires (one line) | today in the compiler | status | what must change | old tests | design tests |
|---|---|---|---|---|---|---|---|

- **today in the compiler:** `crate/src/file.rs:line`, and what it does.
- **status:** one of `built` (the compiler already does what the new text says), `partly`, `not built`,
  `remove` (the compiler has a feature the spec dropped), `text only` (wording change, no behaviour
  change).
- **old tests:** for the rule's folder `tests/conformance/<RULE>/`, counts of `keep`, `respell`,
  `rewrite` and `retire`, with the file names of every non-keep.
  - `keep`: valid under 0.9.15 as written.
  - `respell`: the same meaning, only spellings change (say which).
  - `rewrite`: the expectation changes because the rule's meaning changed.
  - `retire`: it tests something the spec removed.
- **design tests:** how many of the 841 name the rule, and in which `R` folders.

**2. Work items.** Ordered. Each item gives:
- an id (`A<job>-<n>`, e.g. `A3-7`) and a title;
- the crates and files it changes;
- its size: S (under a day), M (days) or L (a week or more);
- the rules it completes;
- the tests it makes pass (design tests by folder and count; old tests to update);
- what it depends on, items of other areas included (name the area: "needs the `shares` type
  representation, area 3").

Group the items into the smallest slices that each leave the whole compiler consistent: built, its
tests green, nothing half-removed.

**3. Shared decisions.** Representations other areas will depend on, e.g.:
- the count header's layout;
- the weak count;
- what an effect summary holds and where it lives;
- how a function value's possible targets are found.

Say what you propose and why, with its cost against C.

**4. Questions for the owner.** Only as defined above, each with the rule text and why it blocks.

**5. Risks.** Places where the existing code fights the new model, the size of the unknowns, and
anything you could not verify.

Be honest about confidence. Cite file:line for every claim about the compiler. Where you
classified old tests by a script, say which ones you read by hand. When done, return the summary your
prompt's schema asks for.
