# Batch N: built code the 0.9.20 spec changed (job sheets for workers, 2026-11-11)

From `tasks/96/redesign/compiler-0.9.15/refresh-0.9.20/README.md`, section "Built code that must
change now". Each job is one worker's, in its own git worktree, branched from `main`.

## Rules for every job (read first)

- **The spec is the law.** The target is `docs/spec-source/Ember_v0.9.20_Hardened_12.md`; its
  sources are `tasks/spec-0.9.9/parts/p*.md`. Build exactly what the rule says, word for word,
  messages included. Never edit the spec, and never write a test to match the compiler where the
  spec says otherwise. A rule you cannot meet, or a spec text that seems wrong, is reported, not
  worked around.
- **General fixes only.** Never special-case a test's names or shapes. Keep the generated code as
  fast as before.
- **Tests first.** Each change gets a conformance test under `tests/conformance/<RULE-ID>/`, named
  `accept_….em` or `reject_….em`, with `#$ test:` and `#$ rules:` lines (see neighbouring files).
  An expected error is a `#$ error[CODE]: message` line under the offending line. The test must
  fail before your change and pass after it. Old tests that the rule now contradicts are rewritten
  to the rule; list each one in your report.
- **Build and check, in your worktree:**
  - Run every cargo and test command with `CARGO_BUILD_JOBS=6 RUST_TEST_THREADS=6` in its
    environment (in Bash: `export CARGO_BUILD_JOBS=6 RUST_TEST_THREADS=6`), the owner's thread cap:
    four workers share the machine.
  - `cargo build -p ember_driver --bin ember`.
  - The quick check: `python tasks/impl-0.9.9/annotations.py tests/conformance/*/ tests/compile-fail tests/compile-pass tests/run-pass tests/run-fail tests/milestones tests/std tests/ffi`.
  - `cargo test -p <the crates you touched>`.
  - With `PYTHONIOENCODING=utf-8`: `python tools/rule_index.py`, `python tools/error_pages.py`,
    `python tools/check_branding.py`.
  - Do not run the full workspace suite or benchmarks. The lead runs those after merging.
- **Never open windows.** Run no GUI program. If a test run hangs or a process misbehaves, stop it
  and report.
- **Commit in your worktree** when green: one commit per item, a message that names the rule and
  says what changed and why, ending with
  `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`. Do not push and do not merge.
- **Report:** per item, what you changed (files), the tests you added or rewrote, and the check
  results with their summary lines. Add anything you found that the spec and the code disagree on,
  and anything you were unsure of.

## Job N-A: syntax (parser)

1. **N1 `[GRM-29]`.** `(e,)`, and one expression followed by a comma in an `expr_list`
   (`return x,`), a `target_list` (`a, = e`) or a pattern (`(p,)`), is `E0100`: "a tuple has two or
   more elements; write `e`". Today `ember_parser/src/body.rs:1458` builds a one-element tuple. Also
   find the places that take a trailing comma after one element in `expr_list`, `target_list` and
   patterns.
   - Rewrite `tests/conformance/GRM-29/accept_unparenthesised_tuples.em`, whose comment still states
     the old rule.
   - Rewrite `tests/conformance/TYP-39/accept_collections_print_as_python_shows_them.em:29`.
   - Search `std/`, `tests/` and `examples/` for other one-element tuples.
2. **N9 `[GRM-2]`.** Read the rule whole: the order of imports, items and file-scope statements, and
   what is `E0100`. Check what the parser does today (`ember_parser/src/lib.rs` around 632-666, the
   item loop). Build only what the rule says that is not built.

## Job N-B: checks in the type checker

1. **N3 `[TYP-22]`.** `Self` appears in an interface's method nowhere but as `this` when the
   interface is to be used as `dyn`: not as a result by value, and not as a parameter's type or
   inside one (`fn eq(other: Self)`, `fn add(rhs: Self)`, `fn merge(owns other: Self)`). Methods
   marked `where Self: needs Sized` are excepted. Today `dyn_incompatibility_inner`
   (`ember_typeck/src/lib.rs:13424`) refuses only the result. Use the rule's code and message.
2. **N4 `[PAR-2]`.** In a `@parallel` loop's body, `?`, `yield` and a `continue` naming an outer
   loop are `E7010`, with the rule's message for `?`. Check what is refused today
   (`ember_typeck/src/lib.rs` around 1392-1449) and add what is missing.
3. **N6 `[UNS-12]`.** An `unsafe fn drop` is `E2020`, with the rule's message ("`drop` runs wherever
   a `Window` ends, in safe code, so it cannot be an `unsafe fn`; …"). Only the `drop` case: the
   override and interface cases come with milestone M11. Leave them out, and say so in a comment
   that cites `[UNS-12]`.

## Job N-C: std texts and names

1. **N5 `[ERR-4]`.**
   - `unwrap()` and `expect(msg)` on a `Result[T, E]` need `E: needs Display` (`E2040` otherwise,
     whose help is to implement `Display`). They panic with the error's `Display` text, and with
     `<msg>: <Display text>` for `expect`.
   - On an `Option` they panic with `called unwrap on None`, and with `msg` for `expect`.
   - Today: `ember_typeck/src/lib.rs:29006` and `:29061`. Read the rule whole first.
   - Check every std type and test that calls `unwrap` on a `Result` whose error has no `Display`.
2. **N7 `[ECS-2]`, `[TYP-44]`.** The spec names `std.core.type_id[T]()`, a function the compiler
   knows; `type_identity` is gone from the spec. Rename it everywhere: `std/src/core.em` (90-219),
   and in `compiler/` the typeck errors, `ember_analysis` facts and `ember_mir`. Read both rules for
   what `type_id` returns and promises, and build that.
3. **N11 `[DET-4]`.** Bring `std/src/math/det.em`'s names and its list of functions to `[DET-4]`'s
   text.

## Job N-D: names and messages

1. **N2 `[MNG-1]`.** A generic instance's symbol ends with `G` and 32 hex digits of the BLAKE3 hash
   of the canonical spelling of its arguments: 128 bits. Today it is 16 (`ember_branding/src/lib.rs:150`,
   and look at `:598`). Change it, then find every test or golden file that spells a mangled
   instance name. Check that the C and LLVM backends and the hot-reload naming agree.
2. **N8 `[FN-4]`, `[LT-1a]`.** Messages that still say `self` where the spec says `this`
   (`ember_typeck/src/lib.rs` around 37505-37510; `ember_parser/src/decls.rs:1065`). `[LT-1a]`'s
   rename help for `E2118` (typeck around 4430) and `E3062`'s help (`ember_analysis/src/borrows.rs`
   around 3320-3420): use the rules' current words.
3. **N10 `[RC-1]`, `[WK-16]`.** The registry rows in `compiler/ember_diag/src/codes.rs` (around 252,
   255 and 439) must match the spec's code table (`tasks/spec-0.9.9/parts/p17a-codes.md`). Check the
   whole registry against that table and fix every row that differs.
