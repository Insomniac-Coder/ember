# Workers' results, the lead's review, and what follows (2026-11-11)

## Worker B: N-B and L-B (branch `worktree-agent-a7cc99b51ca14fb6e`; 291k tokens, 51 min)

**Done:**
- N3 `[TYP-22]` (`e176194c`), N4 `[PAR-2]` (`38a4ef1d`) and N6 `[UNS-12]` (`61516321`);
- 26 error pages (`648859bf` to `6cfb3619`, baselines `0c92ea06`).

Its checks: the quick check reported 0 failing, `cargo test -p ember_typeck` was green, and
`rule_index` showed 88 codes without pages, down from 114.

**Its gaps, sent back to it as follow-ups:**
- **N3.** E2050's first line is still "interface `X` is not `dyn`-compatible". `[TYP-22]` gives one
  message, "`Same` cannot be used as `dyn Same`: …". It is to be built as the rule says, and the
  about 15 E2050 tests rewritten to it, since tests follow the spec.
- **N3, not built.** The rule for a method of a required interface (`cmp(other: Self)` through
  `dyn Shape`).
- **N4.** The message is to name the result array's real type (`Array[Result[int, ParseError]]`),
  which the compiler knows at the `?`.

**Pages not written, because nothing in a page can show them:**
- E9003, which only `ember update` emits;
- E9037, at image load;
- E9040, a hash collision.

**W0001 is a spec slip, for the next cut.** `p17a-codes.md:319` keeps `W0001` as a live row, "a
dangling doc comment is discarded in silence (`[LEX-11]`)". `[LEX-11]` says such a comment produces
no diagnostic. The row moves to the retired codes.

**Defects the pages found (spec vs compiler), each for its milestone:**
- **E3040's message** is "`items` is borrowed after it has been moved out of"; `[OWN-3]` gives its
  own text. A1-3, M3.
- **A temporary that does not live long enough.** `word = line.to_lower().trim()` gives E3060
  ("`_5` does not live long enough"); the spec gives E3063 with its message. A1-18, M5.
- **E2173's help** names `Box[dyn Shape]`; `[TYP-45]` names `owns dyn Shape`, which is not built
  yet. A1-6, M3.
- **E2090's wording, and E2070's,** are to be compared with the spec's text. A7-16, M15.

**B's follow-ups** (after the owner's cap; 414k tokens in all):
- `c78243cf`: E2050 is one message, and the required-interface case is built.
- `6bc190b3`: `[PAR-2]`'s message names the type to keep.
- A1-3 was not done; the work in progress is in the scratchpad, `item3-wip.patch`. It goes back to
  the queue.

## Worker A: N-A and 18 pages (164k tokens, 81 min, 20 items)

- N1 `[GRM-29]` (`06951db8`): one-element tuples are E0100 in four places.
- N9 `[GRM-2]` (`23674a3c`): an import below an item or a statement is E0100.
- 18 pages, E0002 to E1030.
- Not reached (cap): E1041 E1050 E1051 E2010 E2020 E2030 E2031 E2035 E2036 E2040 E2041 E2060 E2061.
- Pre-existing: the pages of E5060 and E5062 fail `error_pages.py` in the workers' runs (both
  workers saw it). To check on `main`.

## Worker C: N-C, 5 codes, 18 rules (338k tokens, 77 min, 26 items)

- N7 (`f9e3d461`): `type_identity` is `std.core.type_id[T]()`.
- N5 (`f4034ab0`): `unwrap` and `expect` need `E: needs Display`, with the panic texts. `Display` was
  added for `Utf8Error`, `NulError` and `SendError[T]`.
- N11 (`df1b0527`): `det` has every `Nondet` name of `std.math`. Ten new fdlibm-style kernels,
  checked by C against mpmath on about 35,000 arguments: worst 0.89 ulp. **Still to be spot-checked
  by the lead.**
- Tests for E1051 E2115 E2173 E0001 E1021, and for 16 rules.
- **Lead's change at integration (`28796304`):** `det.log`'s help came from a test of one qualified
  name. It is now a table, `STD_RENAMED`, which covers `std.math.log` too.

**Not built (found by C):**
- `[ARN-5]` `ArenaArray.in` (`in` is a keyword);
- `[ALC-1]` `using A`;
- `[ALC-4]` `try_push`, `try_reserve`;
- `[ALC-3]` under `--interp`;
- `[RNG-9]`'s debug assert;
- `[STR-3]`'s E2080 for `@derive(Copy)` with `drop`;
- `[STD-6]`'s E1042;
- `[DET-3]`'s E4070 through a `dyn` method;
- `[DET-4]`'s E4070 help.

## Worker D: N-D and 17 rules (345k tokens, 83 min, 20 items committed)

- N2 `[MNG-1]` (`3fa99ea9`): the instance hash has 32 hex digits. The 61 tests that spell one were
  updated from the compiler's own output.
- N8 (`7030e69f`): `this` messages, E2118's help, E3062's help.
- N10 (`874895fd`): 72 registry rows brought to the spec's table, with a unit test that compares
  every row.
- Tests for 17 rules (`e4e81325` to `202c2939`).
- **Merge conflict:** both B and D added fields to `Checker`; both were kept.
- **Left in D's worktree, not merged:** passing tests for about 40 more rules, written before the
  cap. They are to be reviewed before any is taken. TYP-15a is weak, and CLS-1 has an accept test
  only.
- **LEX-2's CRLF test** is stored LF by `.gitattributes`, so it tests nothing. It needs an `eol=crlf`
  line, or another test.

**Defects found by D:**
- `[DIA-3]` the E3040 label;
- `[DIA-2]` `int` printed `i64`;
- `[CTL-7]` `break` in a `defer` gives E1010, not E2160;
- `[GRM-20]` E0104 where the table says E0108;
- `[ATT-4]` two attributes on a line;
- `[OBJ-2]` E2173's reach and message;
- `[LT-1a]` E3060 where E3062 is wanted;
- `[CLS-1]` class objects still counted handles (M4);
- `[CLS-3]` a memberwise base's `init`;
- `[MNG-2]` `@unsafe(export)`;
- `[CTL-5]` E2020's code.

**Not built (found by D):** `weakref`, `shares`, the 8/12-byte header, kind tests, `transmute` and
friends. Each is its milestone's (M3 to M6b).

## Spec findings for the next cut (Hardened_13)

1. The `W0001` row (above).

## Round 2 (R2-batch.md): 13, 20, 12 and 12 tasks

- **W1, error pages** (129k tokens, 21 min): 12 of 13 written, E1050 to E2061, one commit each; baselines
  `3db854db`. E1041 has no page: an import cycle needs two packages, and a page holds one.
  - **Spec vs compiler:**
    - `[LEX-16]`: `128i8` is accepted, where the rule says E2010.
    - `[TYP-34]`: E2030's message is not the rule's.
    - `[DIA-2]`: `int` is printed `i64` (W4's item 9).
  - **Base:** its worktree started from origin/main (`baa4ffdc`), before round 1.
- **W2, D's tests reviewed** (124k tokens, 10 min): 18 of 20 committed.
  - TYP-9b was rewritten: the old test showed nothing about `allow_fma`.
  - TYP-25's reject was removed: the spec gives no code for a positional argument after a named one.
    A spec gap, for the next cut.
  - DIA-7 and DIA-10 were left out: the compiler's helps for B4 and B2 are category words, where the
    rules want the line to write. Defects.
- **W3, deterministic maths** (145k tokens, 13 min): all 12.
  - `tools/det_model.py` models the 11 new kernels. `check_det.py` matches bit for bit: 45,045
    results. Worst error against mpmath is 0.93 ulp.
  - `[STR-3]`: `@derive(Copy)` with a `drop` is E2080 (`f9fb6aa4`).
  - **Base:** it started from origin/main too, and reset itself onto local `main`.
- **W4, defects:** running.

## Spec findings for the next cut (continued)

2. **TYP-25:** the code for a positional argument after a named one (W2).

## Round 2 collected (the lead, 2026-11-11 evening)

Merged into `main` in this order: `lead-a1-3` (A1-3), W3, W4 up to `bcaa0288` (its `[DIA-2]` commit
`29cc91df` left out), W1, W2.

**Conflicts, resolved by the lead:**
- `drops.rs`: A1-3's hand-over messages were kept, with `[DIA-3]`'s "used here" label (W4) in each.
- `E3040.md`, the O1 and O4 snapshots, and `E3042.md` (retired by A1-3): main's side, with the
  snapshots written again from the merged compiler.
- `error_pages_baseline.json`, `rule_index_baseline.json`: each side had removed other entries. The
  merge keeps both removals.

**Changed by the lead at integration:**
- **`[DET-4]` (W4).** The help rebuilt the `det` call from the program's text. It took the
  function's name from that text, and counted every comma as an argument boundary, so
  `math.atan2(f(a, b), c)` gave `det.atan2(math, f(a, b), c)` and `(a + b).sin()` gave
  `det.(…)`. Now:
  - the name and arity come from the MIR call's own builtin (`platform_function`);
  - arguments are split only at top-level commas, past brackets and literals;
  - the call's parentheses are found from its end, and `**` is found at the top level.

  A call that reaches the platform through another function gets no `det` help. Unit test:
  `det_help_tests`.
- **`[STD-6]` (W4).** The error type's name was built from the package's name by slicing the first
  byte of each part, which would panic on a non-ASCII letter. It now goes by characters.

**Reviewed and taken as they are:**
- `[CTL-7]` (a `return` after `E2160`).
- `[GRM-20]` (`E0108`).
- `[LT-1a]` (`E3062` for a view of the object itself; its help is the spec's text word for word).
- `[RNG-9]` (the debug assertion).
- `[DET-3]` (the slot's `deterministic`).
- `[LEX-2]` (`eol=crlf`).
- `[DIA-3]` (the label).
- W1's 12 pages, W2's 18 tests, W3's model.
