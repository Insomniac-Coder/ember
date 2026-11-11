# Batch L: larger lanes, after each worker's batch-N job (2026-11-11)

The owner: "give the agents larger batches of work to do". Each worker finishes its batch-N job, then
does its lane below. **The rules of `N-batch.md`, "Rules for every job", apply to every item here.**

Commit each piece separately: one page, one code, or one rule's tests. A test or page that cannot be
made to pass because the compiler does not do what the spec says is **not** bent to fit. Leave it
out, list it in your report as a defect (the rule, the program, what the compiler does, and what the
spec says), and go on.

## L-A and L-B: error pages for codes the compiler emits

`tools/error_pages.py` checks every page in `docs/errors/`. Read three existing pages and the tool
first: `docs/errors/E0110.md` and two others.

**What a page holds:**
- a failing program (`ember,fails`) that gives the code;
- the expected message (`text`);
- `## Why`, the rule's reason, citing its id;
- `## The fix`, with a program that compiles (`ember,fixed`).

**What it must be:**
- The program and the message are what this compiler really prints today.
- The reason and the fix are what the spec says, from the code's rule in
  `docs/spec-source/Ember_v0.9.20_Hardened_12.md`, and the code table in
  `tasks/spec-0.9.9/parts/p17a-codes.md`.

After each page, run `python tools/error_pages.py` and `python tools/rule_index.py`, with
`PYTHONIOENCODING=utf-8`. When a page lands, its code comes out of `tools/rule_index_baseline.json`'s
`codes_without_pages` list; the tool tells you how.

- **L-A** (the worker of N-A): E0002 E0003 E0004 E0005 E0006 E0100 E0101 E0102 E0103 E0104 E0105 E0106
  E0107 E0108 E0109 E1010 E1020 E1030 E1041 E1050 E1051 E2010 E2020 E2030 E2031 E2035 E2036 E2040 E2041
  E2060 E2061
- **L-B** (the worker of N-B): E2070 E2080 E2090 E2130 E2131 E2140 E2160 E2172 E2173 E2180 E2212 E2213
  E2214 E3021 E3022 E3040 E3050 E3060 E3062 E3063 E9001 E9003 E9014 E9037 E9040 L1001 L1002 W0001 W2015
  W2091

## L-C: tests for codes the compiler emits that no test shows, then std's rules

1. **A test per code.** For each code below, a `tests/conformance/<its rule>/reject_….em` (or
   `accept_…` for a lint or warning) that gives it, with the message line. Find the code's rule in
   the code table, and where the compiler emits it with Grep. The code then comes out of
   `codes_without_tests` in the baseline.

   E0001 E1021 E1041 E1051 E2115 E2173 E5053 E9001 E9002 E9003 E9010 E9014 E9020 E9021 E9031 E9034 E9035
   E9036 E9037 E9040 E9041 E9042 W5054

   Several E90xx are about the manifest, the command line or the C compiler, and need a package
   folder with an `ember.toml`. Look at how `tests/` already does that (search for `ember.toml`
   there), and for driver tests in `compiler/ember_driver/tests/`.
2. **std's rules without a test**, from `tools/rule_index_baseline.json`'s `rules_without_tests`:
   - the families STD, ERR, TXT, FMT, RNG, STR, DET, PAN, HEAP, ALC, ARN;
   - for each rule, first check whether the compiler and std do what the rule says today;
   - built: a test that shows it;
   - not built: list it, and write no test.

## L-D: language rules without a test

From `rules_without_tests`, the families:
- TYP GRM LEX DIA CTL ENM CLS ATT LNT MONO DSP DSJ IFC FN LT MOD STA OPT SEL OBJ RC WK UNS THR SIMD
  CXX MNG.

For each rule:
- read it whole;
- check whether the compiler does what it says today;
- built: a test that shows it, `accept_…` and, where the rule refuses something, `reject_…` with
  the code's message line;
- not built, or partly: list it with what is missing, and write no test.

**Do not test tooling rules here:** HR, CLI, BLD, TST, GPU, BEN, BUD, CONF, DOC, TOOL, VER, TCB, IDE,
PRF, COST, CG-C and ABI. They need drivers, packages or benchmarks.
