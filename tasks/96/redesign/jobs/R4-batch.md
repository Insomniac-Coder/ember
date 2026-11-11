# Round 4: two workers, 10 to 20 tasks each

Sizes: **W1 12, W2 15** (27 tasks in all).

The owner's rules for rounds:
- each worker gets 10 to 20 tasks;
- at most 2 to 3 agents run at once, an Opus helper included;
- the lead collects, tests and combines each batch;
- the laptop rests 10 minutes after each batch before the next one starts.

**The rules of `N-batch.md`, "Rules for every job", apply to every task here, with
`CARGO_BUILD_JOBS=6 RUST_TEST_THREADS=6`.** Branch from local `main`. Your worktree may start from `origin/main`:
run `git reset --hard main` first, and check that `git log` shows the A1-6 merge (`owns T` replacing `Box`).
Build in your worktree's own `target/`, since the tools look there, and delete it when you finish. Commit each
task separately.

A task that cannot be done as the spec says is not bent to fit. Report it with:
- the rule;
- the program;
- what the compiler does;
- what the spec says.

**Before writing a test or a page for a code, check that the compiler emits it**
(`grep codes::EXXXX compiler/`). Round 3 lost a job to codes no rule builds yet.

**Files the lead is changing now (do not touch):** `ember_mir/src/lower.rs`, `ember_analysis/src/drops.rs`,
`borrows.rs`, `regions.rs`, and `ember_typeck`'s argument passing (`check_argument_by`, `check_call_word`,
`pass_argument*`). A task that needs one of these is reported, not done.

**Never leave files outside your worktree**, above all in `%TEMP%` itself.

## W1: messages to the rules' text, and std to its rules (12 tasks)

1. `[OBJ-2]` `E2173`: the rule's message and help, now that `owns dyn` exists. The text: "`Shape` is an
   interface, so `Array[Shape]` does not say who owns the shapes; say who owns them: write `shapes:
   Array[owns dyn Shape]` …"; read the rule whole. Then commit worker D's OBJ-2 test, from round 3's
   folder (`.claude/worktrees/agent-af0eb72abdb1e39be/tests/conformance/OBJ-2/`), rewritten to the rule.
2. `[OBJ-5]` `E3016`: "`drop` ends this `Player`; an owner made from it would point at a freed object;
   keep a copy of what you need: `NAMES.push(owns copy of this.name)`". Then CLS-7a's and OBJ-5's tests
   from the same folder, rewritten to the rule's text.
3. `[CLS-9a]` `E2104`: "`items` is `let`: it is set in `init` and never changed after; to add to it,
   write the field without `let`: `items: Array[int]`". Then CLS-9a's test, rewritten to the rule.
4. `[DSJ-1]` `std.mem.check_disjoint(a, b) -> Result[void, void]`, as the rule says.
5. `[DSJ-2]` `assert_disjoint` is a statement that stops the program on overlap, as the rule says. Rewrite
   `tests/conformance/DSJ-1/`, which pins the old API, and every other use in `std/` and `tests/`.
6. `[DSJ-5]` what it says of the two, against what is built (`assert_disjoint_or_panic`).
7. DSJ-2's and DSJ-8's tests, from the same folder, rewritten to the rules.
8. A message that names an `owns` type reads "an `owns …`", not "a `owns …`". Find where messages put "a"
   before a type and choose the article from the word that follows. This is general, not a list of
   messages.
9. `[DIA-2]` a fixed array the program writes `[int; 3]` is named `[int; 3]`, as `int` itself is wherever
   it is shown. Report it instead if the type does not keep the spelling.
10. `[OBJ-3]`, `[OBJ-5]`: the other messages of these rules, checked against the compiler. Fix each one
    that differs, with a test.
11. `[CLS-1]`: what the rule says that is built, tested. Leave the hand-over of a class object by `=`
    (M4) for later.
12. `[E2123]`'s page, `docs/errors/E2123.md`, and `E2114`'s, `docs/errors/E2114.md` (A1-6 made both
    emitted), by L-A's method.

## W2: conformance tests for rules without one (15 tasks)

Rules in `tools/rule_index_baseline.json`'s `rules_without_tests` whose behaviour is built. Take them
from these families in this order: `EFF`, `MONO`, `CG-C`, `ABI`, `CONF`, `RT`, `TYP`, `UNS`, `WK`, `ALC`,
`ARN`, `PRF`, `SIMD`, `STD`, `THR`, `TXT`.

Stop at 15 rules done. For each rule:
1. Read the rule whole.
2. Check that the compiler does what it says (run a program). Skip and report a rule it does not do.
3. Write the `accept_…` and `reject_…` tests.
4. Run `python tasks/impl-0.9.9/annotations.py tests/conformance/<RULE-ID>/`.
5. Remove the rule from the baseline.
6. Commit.

A rule whose test needs a second build, command-line output or a library a host loads is skipped, as
in round 3.
