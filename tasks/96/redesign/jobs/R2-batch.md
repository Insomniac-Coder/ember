# Round 2 (2026-11-11): four workers, 10 to 20 tasks each

The owner's rules for rounds:
- each worker gets 10 to 20 tasks;
- at most 4 agents run at once;
- the lead collects, tests and combines each batch before the next one is given out.

**The rules of `N-batch.md`, "Rules for every job", apply to every task here.** Branch from `main`.
Commit each task separately.

A task that cannot be done as the spec says is not bent to fit. Report it with the rule, the program,
what the compiler does, and what the spec says. **Do not run the full quick check:** run it on the
folders you touched only. The lead runs the whole suite on the combined tree.

**Never leave files outside your worktree**, above all in `%TEMP%` itself. A stray
`%TEMP%\ember.toml` makes every temporary folder a package; it broke the error-page gate in round 1.

## W1: error pages (13 tasks)

The 13 codes the compiler emits that round 1 did not reach:

E1041 E1050 E1051 E2010 E2020 E2030 E2031 E2035 E2036 E2040 E2041 E2060 E2061

Same method as `L-batch.md`'s L-A. After each page, run `tools/error_pages.py` and
`tools/rule_index.py --write-baseline` (closing only).

## W2: worker D's 40 uncommitted tests, reviewed and committed (20 tasks)

They sit in `C:\Users\ism19\Code\ember\.claude\worktrees\agent-af0eb72abdb1e39be\tests\conformance\`,
untracked. Take the first 20 of these, in this order:

DIA-2 DIA-7 DIA-9 DIA-10 ATT-2 ATT-3 ATT-4 LNT-2 LNT-3 LNT-4 TYP-9a TYP-9b TYP-11 TYP-12 TYP-25
TYP-26 IFC-1 FN-2 LT-5 STA-2

For each:
1. Copy the folder into your worktree.
2. Read the rule's whole text in the spec. Check that the test shows what the rule says, not only
   what the compiler does today. A test that only pins today's behaviour where the rule says
   something else is rewritten to the rule. If the compiler fails it, the folder is left out and the
   rule is reported.
3. Run it, remove the rule from the baseline's `rules_without_tests`, and commit.

## W3: deterministic maths, and four rules found not built (12 tasks)

1. `tools/det_model.py` models the 11 kernels round 1 added to `std/src/math/det.em`: `asin` `acos`
   `atan` `sinh` `cosh` `tanh` `exp2` `log2` `log10` `cbrt` `hypot`, one task each. `tools/check_det.py`
   checks each, `f64` and `f32`, bit for bit. A difference between the model and the Ember kernel is
   resolved by the more accurate one, checked against mpmath; report each.
2. `[STR-3]`: `@derive(Copy)` on a struct that has a `drop` is `E2080`, with the rule's message.

## W4: defects found in round 1 (12 tasks)

1. `[CTL-7]` `break` or `continue` in a `defer` inside a loop is `E2160`, not `E1010`.
2. `[GRM-20]`, `[ATT-2]` an attribute not permitted on a statement is `E0108`, as the code table
   says, not `E0104`.
3. `[LT-1a]` under `-> ref int from owner`, `return ref this.at` is `E3062` with its two-way help,
   not `E3060`.
4. `[RNG-9]` `T.new_unchecked(v)` asserts the range in `debug`.
5. `[STD-6]` a package with `layers = ["core"]` that names `Result[int]` is `E1042`, naming the
   alloc layer.
6. `[DET-3]` a call through a `dyn` method that is not `@deterministic`, inside a `@deterministic`
   function, is `E4070`.
7. `[DET-4]` `E4070`'s help for a `std.math` call is `det`'s function of the same name.
8. `[OBJ-2]` `E2173`: as the rule says, for whom and with which message.
9. `[DIA-2]` a type the program writes `int` is named `int` in messages, not `i64`. Rewrite the
   tests that expect `i64` where the program wrote `int`.
10. `[DIA-3]` an ownership error's label says "used here".
11. `LEX-2`'s CRLF test is stored LF (`.gitattributes` says `eol=lf`). Give that file `eol=crlf` in
    `.gitattributes`, or write the CRLF bytes from a driver test, so that it tests CRLF.
12. `[ATT-4]` two attributes on one line: what the rule says.
