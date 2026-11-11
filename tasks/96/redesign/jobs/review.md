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

## Round 3 (R3-batch.md): three workers, at most three agents at once (the owner, 2026-11-11 ~19:10)

The owner cut the cap to "2-3 agent at once (including the opus helper agent)" before round 3 began, so
the round was resized:
- W1 had 15 error pages;
- W2 had 20 tests;
- W4 had 10 defects;
- W3's FFI tests moved to the next round.

When W1's job turned out empty, W1 took the FFI tests (15). The total stayed 45 tasks. "A 10-minute break
after each work batch" is the owner's rule from this round.

- **W1, error pages, then FFI tests** (168k tokens in all, 16 min).
  - **The pages: none.** None of the 25 codes in the job sheet is emitted by the compiler: their rules are
    not built. **The lead's mistake:** the sheet listed `codes_without_pages` without checking
    `error_pages.py`'s `emitted_codes()`. The error-page lane has nothing left until more rules are built
    (E9040 and W0001 are emitted; W0001 is the spec slip already queued).
  - **FFI tests: 11 rules.** FFI-3, FFI-4, FFI-5a, FFI-13, FFI-22, FFI-24b, FFI-27, FFI-29a, FFI-29, FFI-29c and
    FFI-30a are committed. 9 were skipped, each needing two builds, CLI output or a library a host loads,
    which a conformance file cannot show. They are covered by driver tests, where they are covered at all:
    FFI-7, FFI-14, FFI-17f, FFI-24c, FFI-24d, FFI-28, FFI-31, FFI-31a and FFI-31c.
  - **Spec vs compiler:**
    - `[FFI-22]`: a one-thread owner captured by a `threads(any)` callback is E7015, and a thread-bound value
      is E7008. The compiler emits neither: a class handle gives E7004 (FFI-21's text). M9/M12.
    - `[FFI-29a]`: "never names a symbol with internal linkage". The compiler calls a plain `static` C
      function through the shim's wrapper. Taken as meeting the rule (the binding names the wrapper);
      flagged in case the rule means the function must not be callable.
  - **Tool note:** `error_pages.py` and `annotations.py` find the compiler at `ROOT/target/{release,debug,ci}`,
    not at `CARGO_TARGET_DIR`. Jobs should build in the worktree's own `target/`.
- **W2, worker D's other 20 tests** (124k tokens, 6 min). 12 of 20 are committed: MOD-4, CLS-3 (its reject
  was removed: no code is given for a missing memberwise argument, a spec gap), CLS-6, DSP-2, MONO-1, MONO-3,
  MONO-5, MONO-8, MONO-9, OBJ-3, OPT-3 and UNS-2. Left out, each following the spec where the compiler does not:
  - **CLS-1:** `=` hands a class object over (M4). Its `owns Player` E2114 is now built by A1-6.
  - **CLS-7a, OBJ-5:** E3016's message is not `[OBJ-5]`'s text ("`drop` ends this `Player`; …").
  - **CLS-9a:** E2104's message is not the rule's ("`items` is `let`: …").
  - **OBJ-2:** E2173's text. It needs `owns dyn`, which A1-6 now builds.
  - **DSJ-2, DSJ-8:** `std.mem` has `assert_disjoint` returning a `Result`, and no `check_disjoint`.
    `[DSJ-1]`, `[DSJ-2]` and `[DSJ-5]` give `check_disjoint(a, b) -> Result[void, void]` and
    `assert_disjoint` as a statement. The DSJ-1 test already committed pins the wrong API.
  - **RC-1:** `shares mut` is not built (M4).
- **W4, defects** (298k tokens, 32 min). 8 of 10 done:
  - **DIA-7 B4's help.** The line to write: `ref mut clock: Clock` and `ref mut` at each call.
  - **LEX-16:** `128i8` is E2010, while `-128i8` fits.
  - **TYP-34:** E2030's message is the rule's.
  - **DIA-7a:** the first-week shape table is §XVII.6.1's, with a test that compares it row by row with the
    pinned spec.
  - **CTL-5 / GRM-13:** a list pattern that can never match is E2020.
  - **CLS-3:** a memberwise base `init` is built through `super.init(…)`. A base that itself derives is not.
  - **GRM-20:** three parser paths now give E0108.
  - **DIA-6:** E1042's page now shows the real output.

  Reported, because their files are the lead's:
  - **DIA-10 B2's help** is in `borrows.rs`.
  - **The internal std names in hand-over messages** are in `drops.rs`: a table from `option_map` to `map`
    and its kin.

  Found: E3025, E3030, E3061 and E3080 are still emitted though retired. `shapes.rs` keeps the old shapes.
  `[int; 3]` prints `[i64; 3]` (DIA-2).

**Collected by the lead:**
- **Merged into `main`:** W1 (`072d43b7`), W2 (`ef3de060`) and W4 (`8b226a06`). The rule-index baseline
  conflict kept both sides' removals.
- **The push gate on round 2's tree (`99f21806`):**
  - The CI-style run is green.
  - The MSVC run and the C-backend run each failed `TYP-9b`'s test (round 2's W2): with MSVC the
    development build inlined `plain`, so its `assert-c` lines found nothing. Fixed with `@noinline`
    (`[CG-C-3a]`), and checked in the emitted C.
  - The C-backend run also failed two speed tests, during the owner's room switch:
    - the bench's "void: the machine is on battery power";
    - the compiler-speed test's timings.

    The owner said to ignore any dip then. Both pass when rerun on mains.

**Spec findings for the next cut (continued):**

3. **CLS-3:** a missing memberwise-constructor argument has no code (W2). W4 used E2020, as TYP-25 did.

**Queued for the next rounds:**
- the E3016 and E2104 messages (W2);
- `std.mem`'s `check_disjoint` and `assert_disjoint`, to the spec (W2);
- the internal std names in hand-over messages, `drops.rs` (W4, lead);
- DIA-10 B2's help, `borrows.rs` (W4, lead);
- E3025, E3030, E3061 and E3080, which are still emitted (W4);
- `[int; 3]` printed as `[i64; 3]` (DIA-2);
- "a `owns …`" in messages, which should read "an".

## Round 4 (R4-batch.md): two workers, 12 and 15 tasks

- **W2, tests for rules without one** (146k tokens, 12 min). 8 of 15 are committed: EFF-20, EFF-14,
  EFF-18, TYP-3, CG-C-1, CG-C-5, CG-C-8 and PRF-1. The baseline went from 241 to 233. The other 7 slots
  could not be filled.
  - **Not built:**
    - TYP-15a (the compiler is still at TYP-15's old rule; its test `reject_a_borrowed_str_pushed_into_an_array`
      pins that and contradicts TYP-15a);
    - ARN-5's `ArenaArray.in`;
    - UNS-5's `std.mem.copy` and friends;
    - WK-1 to WK-3 (`weakref`, M4);
    - ALC-1 (`using`);
    - ALC-3 (`global_allocator`);
    - ALC-4 (`try_push`).
  - **Skipped:** CLI, toolchain or host needed.
  - **Found, fixed by the lead: D-972.** The C backend emitted C's `<<` on negative signed values, which
    is undefined in C11. It now shifts in the unsigned type of the width, the same instruction.
  - **Found, open:**
    - a char literal with a surrogate escape reports `E0009` twice;
    - the checked helpers' `__builtin_*_overflow` calls live in `ember_rt.h`, not in the emitted C.
- **W1, messages and std to the rules** (303k tokens, 48 min). 5 of 12 were done:
  - **the article rule.** `ember_diag::articles` chooses "a" or "an" before a quoted name by its first
    sound. The lead added one-letter names, which are said as their letter.
  - **the E2114 and E2123 pages;**
  - **CLS-1's tests;**
  - **OBJ-2:** E2173 has the rule's message;
  - **CLS-9a:** E2104 has the rule's message.

  7 were reported:
  - **OBJ-5's E3016** lives in `drops.rs`, which is the lead's.
  - **DSJ-1, DSJ-2 and DSJ-5:** the spec's `check_disjoint` and statement `assert_disjoint` need facts kept
    on the names (`[BRW-11]`, `[DSJ-11]`) in the analysis. Today the facts ride on the returned tuple, and
    DSJ-3's `restrict` and the bench probe rely on that. **For the lead, with M5/M6c.**
  - **`[int; 3]`:** the type table keeps no spelling, since `int` is `i64`'s alias. **A design question:**
    a spelling kept per written type, or a display rule.
  - **OBJ-3:** has no message.
  - **Spec vs compiler:**
    - `[OBJ-2]`'s reach: a bare interface with a class implementer is still taken as a class handle
      (ODR-226).
    - `[CLS-9a]`: a change through a `let` field (`push`) is still accepted.
    - `[DSJ-5]`'s message has no general form, a spec slip for the next cut.
