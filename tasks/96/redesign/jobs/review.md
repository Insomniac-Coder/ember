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

## Spec findings for the next cut (Hardened_13)

1. The `W0001` row (above).
