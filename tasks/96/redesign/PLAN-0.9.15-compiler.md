# Plan: the next steps toward 0.9.15_Hardened_1 (the lead's, 2026-10-27)

The owner, 2026-10-27: "Do the next steps" (after "the goal is to move to 0.9.15_H1"). The three steps:
the compiler brought to the spec, the measuring phase, and the design tests moved into
`tests/conformance`. The spec is `docs/spec-source/Ember_v0.9.15_Hardened_1.md`, now the development
target (`ab2b7b76`).

## Where things stand

- **The spec changed 521 rules** (§H.41, generated): FFI 40, TYP 24, LT 23, BRW 22, HR 21, GRM 21, EXC 20,
  THR 18, WK 17, STD 17, EFF 17, CLO 17, DIA 15, and 60 smaller families. Some rules are retired.
- **The compiler** is about 230,000 lines of Rust: the type checker (68k), the driver (39k), analysis
  (38k), the C backend (19k), LLVM (14k), MIR (12k) and the evaluator (9.5k).
- **Old spellings are everywhere.** Of 3,758 `.em` files (tests, std, examples, bench): `owned` in 334,
  `Box` 153, `Shared` 135, `Weak` 131, `@borrows` 61, `changes` 51, `@sync` 48, `RefCell` 47, `SyncShared`
  11, `mem.forget` 6. The std library is 13,008 lines, and `std/src/ecs.em` is still World/Query (the EnTT
  design is `tasks/95/ecs-entt-design.md`; no code yet).
- **The test suites.** `tests/conformance` has 3,147 tests in 589 rule folders. The design's 841 tests,
  with spec IDs since `ab2b7b76`, are in `run4/v5/conformance/`. The runner refuses unknown `#$` keys, so
  `#$ count_bits:` ([TST-31]) needs runner support. `tools/rule_index.py` at the new target reports 6 new
  problems: [COST-5] labels the compiler still prints (`RefCell` borrow, dynamic exclusivity check, ...).
- **The stopped helpers** (`tasks/96/helpers-stopped.md`): AK (compiler memory), AJ (a type for every
  function, ODR-278) and AI (the rest of the LLVM backend). They are on `stopped/*` branches, not merged.
  The note's order is AK, then AJ, then AI, each re-checked against the reworked rules. AJ's function
  types bear directly on the new whole-program reading of function values.
- **Branches.** `main` sits at this branch's base, and this branch is 112 commits ahead (design work
  only). The owner's rule is to work on `main`. Fast-forwarding now would turn main's spec-tied checks
  red. So the default is to stay on this branch until milestone 0 is green, then fast-forward `main` and
  go on there.

## Step A: the gap map (read-only workflow, one launch)

For every changed or retired rule, the gap map records four things:
- what the compiler does today (file:line), and whether that is built, partly built, not built, or to
  be removed;
- for each old test in that rule's folders, whether to keep, respell, rewrite or retire it;
- which design tests prove the rule;
- the work items, with size and dependencies.

From the nine reports I write the milestone plan (step C) for the owner's approval.

| # | Job | Why it is its own job | Model, effort |
|---|---|---|---|
| 1 | Ownership and hand-over: OWN, FN, MONO, ALC, ARN, HEAP, DRP, EXP, STA (~54 rules) | `owns`/`copy of` and every move; arenas with `using` | opus, high |
| 2 | Views and borrowing: BRW, LT, DSJ, SPN, CTL, SOA, SIMD, OPT, STR (~73) | compile-time exclusivity, the loop rule, unknown indexes (98), one-name copies (88) | opus, xhigh |
| 3 | Sharing and links: RC, WK, CELL, OBJ, BCK, HND (~54) | `shares` counts with the limit compare, `weakref`/`exists as`, E3032's owner loops, the walk | opus, xhigh |
| 4 | Functions, effects and checks: CLO, EFF, EXC, CLS, DSP, IFC, ERR (~84) | capture words, the whole-program effect reading, removing the run-time exclusivity checks | opus, xhigh |
| 5 | Threads and unsafe: THR, PAR, JOB, CORO, UNS, TIER, TCB, GPU (~45) | `thread_shared`, `thread.spawn`, reductions, `unsafe` | opus, high |
| 6 | C interface and hot reload: FFI, BLD-FFI, HR (~64) | `into_raw()`, `owned` parameters, trampolines, `@unsafe(export)` | opus, high |
| 7 | Syntax, types and diagnostics: GRM, LEX, TYP, TXT, DIA, PHIL, COST, PRF, TST and the small families (~110) | the parser's new words, type forms, codes and error pages, runner keys | opus, high |
| 8 | The std library, ECS, examples and bench: every `.em` outside the tests that uses a removed spelling; ECS to EnTT | 13k lines of std; ECS is new code | opus, high |
| 9 | The stopped helpers' branches AK, AJ, AI: what each changes, conflicts with the rework, merge order | small, separate input | sonnet, high |
| 10 | Completeness critic, after 1 to 9: rules no map covered, maps that contradict each other | catches the gaps between areas | opus, high |

**Worst case: 10 agents.** All are read-only and none builds or runs anything. The results go to
`tasks/96/redesign/compiler-0.9.15/gap/` (one file per job, plus `critic.md`). My `MILESTONES.md` then
comes to the owner.

## Step B: the measuring phase (source-only agents; I run the timings)

Section 2 of `run4/v5/rules.md` still marks these costs "not measured": the count's limit compare, the
weak counts on parent links, the walk's count moves, the atomic count of a `thread_shared` view, the
arena's instance word and its allocation path, the release-profile thread check in the trampoline, the
handler's self-count, `this` without noalias, and the one-name copy of a function (verdict 88). Each is
measured against plain C and against Rust doing the same work, for both speed and memory. Verdict 47
also asks for ways to drop the arena pointer.

| # | Job | Model, effort |
|---|---|---|
| M1 | Counts: `shares` plus the limit compare, atomic for `thread_shared`, a view through a `weakref`, a handler's self-count | opus, high |
| M2 | Links and walks: parent links' weak counts, the walk's header test and count moves, the deferred-drop region | opus, high |
| M3 | Arenas: allocation with `using A`, the instance word (speed and memory), and the two ways to drop it | opus, high |
| M4 | Calls: `this` without noalias, one-name copies (speed and code size), the trampoline's thread check | opus, high |

**Worst case: 4 agents.** Each writes C, C++ and Rust models with `bench/harness.h` (time, peak memory,
checksum) under `tasks/96/redesign/measure-0.9.15/`, plus a run list. Agents never run programs. I run
every timing myself, one at a time, on mains (Kernel-Power 105 checked) with P-core affinity `0xC03C03`.
WSL gcc runs on the guest's CPU 0, disclosed. Anything slower than C or Rust then gets a salvage round
under JOB.md's rule (fix the implementation first, discard only with the reason). That round would be a
separate question, with its own agent count. The numbers replace "not measured" in the rules and in the
spec's cost tables.

## Step C: the compiler, milestone by milestone (planned after A and B; its own approval)

Each milestone is one complete slice. It changes the compiler, std and tests together and keeps every
MUST of the rules it touches. It ends with the full check green (quick check, `cargo test --workspace
--profile ci`, the gates, WSL gcc), then the laptop speed gate when speed can change, then a commit.
Milestone 0 makes the spec-tied checks green at the new target: the codes registry in `codes.rs`, the
[COST-5] labels, error pages for the new codes, and the runner's `count_bits`. The helpers' branches
merge where job 9 places them. Whether helpers are used, and how many, is part of that plan.

## Step D: the tests move into `tests/conformance` (inside C)

`run4/v5/conformance/` is the queue. A test moves into `tests/conformance/<its first spec ID>/` in the
milestone that makes it pass, so the suite stays green. The 347 tests mapped only by their design rules'
main ID get their exact IDs then. In the same milestone, the old tests of a changed rule are respelled,
rewritten or retired as job 1 to 8's map says.

## What the owner gets

After A and B: the gap map's summary, the measured costs (with anything that fell short), and
`MILESTONES.md` to approve. After each milestone of C: what was built and which tests moved.
