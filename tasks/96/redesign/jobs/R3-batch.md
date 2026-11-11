# Round 3 (2026-11-11 evening): four workers, 10 to 20 tasks each

Sizes: **W1 15, W2 20, W3 15, W4 10** (60 tasks in all).

The owner's rules for rounds:
- each worker gets 10 to 20 tasks;
- at most 4 agents run at once, the lead's helpers included;
- the lead collects, tests and combines each batch before the next one is given out.

**The rules of `N-batch.md`, "Rules for every job", apply to every task here.** Branch from local
`main` (your worktree may start from `origin/main`: run `git reset --hard main` first if
`git merge-base --is-ancestor main HEAD` fails, then check `git log -1` names the round-2 merge).
Commit each task separately. Use the build folder named in your job (`target/w1` to `target/w4`,
`CARGO_TARGET_DIR`), and delete it when you finish.

A task that cannot be done as the spec says is not bent to fit. Report it with the rule, the program,
what the compiler does, and what the spec says. **Do not run the full quick check:** run it on the
folders you touched only. The lead runs the whole suite on the combined tree.

**Never leave files outside your worktree**, above all in `%TEMP%` itself.

**Files the lead is changing now (do not touch):** `ember_typeck`'s `Box` and `owns T` paths
(`box_of`, the `Box` vocabulary errors), `ember_mir/src/lower.rs`, `ember_analysis/src/drops.rs`,
`borrows.rs`, `regions.rs`. A task that needs one of these is reported, not done.

## W1: error pages (15 tasks), build folder `target/w1`

The codes without a page (`tools/rule_index_baseline.json`, `codes_without_pages`) that the compiler
emits today. Take them in this order, skipping any the compiler never emits (say which, and why):

E0007 E0012 E0114 E0116 E0118 E0119 E0901 E2037 E2062 E2074 E2106 E2108 E2112 E2113 E2117 E2119
E2122 E2224 E2227 E3031 E4010 E4071 E4073 E5035 E5051

Stop at 15 pages. Same method as `L-batch.md`'s L-A. After each page, run `tools/error_pages.py`;
`tools/rule_index.py --write-baseline` once at the end.

## W2: worker D's other 20 tests, reviewed and committed (20 tasks), build folder `target/w2`

They sit in `C:\Users\ism19\Code\ember\.claude\worktrees\agent-af0eb72abdb1e39be\tests\conformance\`,
untracked:

CLS-1 CLS-3 CLS-6 CLS-7a CLS-9a DSJ-2 DSJ-8 DSP-2 MOD-4 MONO-1 MONO-3 MONO-5 MONO-8 MONO-9 OBJ-2
OBJ-3 OBJ-5 OPT-3 RC-1 UNS-2

The method of round 2's W2 (`R2-batch.md`). For each one:
1. Copy the folder.
2. Read the rule's whole text. A test that only pins today's behaviour where the rule says otherwise
   is rewritten to the rule. A test the compiler then fails is left out, and its rule reported.
3. Run the test, remove the rule from `rules_without_tests`, and commit.

Known: CLS-1's class objects are still counted handles (M4), and OBJ-2 needs `owns dyn` (M3).
Expect both to be left out unless their tests show only what is built.

## W3: conformance tests for C-interface rules without one (15 tasks), build folder `target/w3`

Rules of the `FFI` family in `rules_without_tests`, in this order:

FFI-3 FFI-4 FFI-5a FFI-7 FFI-13 FFI-14 FFI-17f FFI-22 FFI-24b FFI-24c FFI-24d FFI-27 FFI-28 FFI-29
FFI-29a FFI-29c FFI-30a FFI-31 FFI-31a FFI-31c

Stop at 15 rules. For each:
- read the rule whole;
- write accept and reject tests that show what it says, with any C header or source the test needs in
  its own folder (see `tests/conformance/FFI-*` and `tests/ffi/` for how imports name local files);
- if the compiler does not do what the rule says, leave the test out and report the rule.

Never put a C file anywhere but the test's folder.

## W4: defects found in rounds 1 and 2 (10 tasks), build folder `target/w4`

1. `[DIA-7]` The help for shape B4 says the line to write, not a category word (round 2's W2: the
   compiler's help is a category word where the rule wants the line). Read `[DIA-7]` and
   `[DIA-7a]`'s B4 row.
2. `[DIA-10]` The same for B2.
3. `[LEX-16]` `128i8` is `E2010` with the rule's message; today it is accepted.
4. `[TYP-34]` `E2030`'s message is the rule's text.
5. `[DIA-2]` A message that names a standard-library function names it as the program can write it,
   never an internal one. Today a hand-over says "handed over to `option_map`". Find each internal
   std name a message can show, and give the program's spelling (`map` on an `Option`) or leave the
   name out.
6. `[DIA-7a]` The first-week shape table (`tests/firstweek/`) lists shapes the 0.9.20 catalogue no
   longer has (O3, O4, O5; `E3012`, `E3041`, `E3042` are retired). Bring the table and its checker to
   the catalogue.
7. `[CTL-5]` `E2020`'s code and message where the rule gives them (round 1, worker D).
8. `[CLS-3]` A memberwise base's `init`: what the rule says, built (round 1, worker D). If it needs
   M4's class-object changes, report it instead.
9. `[GRM-20]` Every attribute the statement list does not have is `E0108`. Round 2 fixed one path;
   find the others (statement attributes in the parser, and in the checker if it has its own).
10. `[DIA-6]` `docs/errors/E1042.md` (round 2) and the `STD-6` test agree with the message the
    compiler now prints (`Result[int]` means `Result[int, AnyError]`, …), and `error_pages.py` is
    green for it.

Not for this round (the lead's or a later milestone's): `[DIA-2]`'s `int`/`i64` naming, `[OBJ-2]`
(needs `owns dyn`), `[LT-1a]`'s `from` on a field that is not a view (M5), anything in the files
listed above.
