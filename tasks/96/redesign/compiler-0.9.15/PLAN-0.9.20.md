# Building the compiler to spec 0.9.20_Hardened_12: the plan, and who does the work (2026-11-11, for the owner)

The owner: "come up with a plan to implement the language, do you think using Sonnet 5.5 workers for
coding with you as the orchestrator and decision maker/reviewer would be a good idea? or maybe using
yourself and another helper opus 5.5 to make decisions/review".

## Where things stand

- **The compiler** (`main`, 246,000 lines of Rust in 16 crates) builds 0.9.10, plus:
  - M1: the 0.9.15 spellings, landed and CI green;
  - part of M2: the whole-program summaries, slices 2 to 4;
  - the start of M3: `owns` hand-over (A1-2).

  README Overall: 13%. Nothing in the compiler has changed since `5001ef9e` (2026-11-02).
- **The spec moved on after the milestone plan was made.** `MILESTONES.md` was made against
  0.9.15_Hardened_1. Since then 359 of its rules changed and 24 are new; none was removed. The families
  with the most changes: FFI 45, TYP 25, HR 22, GRM 18, THR 16. Many of these are wording only, but
  not all. Example: IX.7's example became `owns (…)` in 0.9.20_Hardened_3.
- **Two branches.**
  - `main` holds the compiler and spec 0.9.15_Hardened_11.
  - `spec-0.9.20-hardening` holds spec 0.9.20_Hardened_1 to _12. It is 26 commits ahead of `main`;
    the last two are not pushed.
- **Open compiler defects:**
  - D-966: its fix is ready in `pending/`;
  - D-967 and D-968.

## Step 0: one base to build on (solo, about a day)

1. **One branch.** Merge `spec-0.9.20-hardening` into `main`, so the compiler is built against the
   pinned 0.9.20_Hardened_12. This needs the owner's word; he said "Not yet" to pushing.
2. **Refresh the milestone map for the 383 changed or new rules.** For each rule:
   - say whether its meaning changed or only its wording;
   - name the milestone that takes it (`MILESTONES.md`'s table, plus a column for 0.9.20).

   A script lists the word-level changes, and I read each one. Questions only the owner can answer
   go to him one at a time.
3. **The three defects:** D-966's ready fix, then D-967 and D-968.
4. **The spec-tied tests green against the new pin.** They are green now. Every-example test: fixed
   in `49926898`.

## Then the milestones, in `MILESTONES.md`'s order

- **The critical path.** M2's remaining slices, then M3, M4, M5, M6a, M6b, M6c and M7: ownership,
  views, the summaries' call checks, the new object header, and loops of owners. That is about 20
  large items (a week or more each), and each one builds on the one before.
- **After the critical path:**
  - M8, function values;
  - M9, threads;
  - M10, arenas;
  - M11, interface values and generics;
  - M12, the C interface. Hardened_8 to _12 changed a lot here;
  - M13, hot reload;
  - M14, std.ecs;
  - M15, close-out.
- **Each milestone lands as `MILESTONES.md` says:**
  - every MUST of its rules is built;
  - the full check is green: the quick check, `cargo test --workspace --profile ci`, the gates, and
    GCC in WSL;
  - the laptop speed run passes where speed can change;
  - then commit and push.

## Who does the work

### The two options

**Sonnet 5.5 workers code, I orchestrate and review.**
- **Good for side work** that is bounded, sits in its own files, and has a test that says done.
- **Wrong for the critical path**, for three reasons:
  - Those milestones live in the same few files (`borrows.rs`, `regions.rs`, `lower.rs`, the
    summaries). Two writers there collide.
  - Reviewing a borrow-checker change properly costs nearly as much as writing it.
  - A soundness bug that gets past review is the most expensive mistake a memory-safe language can
    make.

Sonnet 5.5 costs half of Opus 5.5 per token at API prices ($2 / $10 against $4 / $20 per million).
What counts is the cost per finished job: a Sonnet job that needs one redo costs as much as an Opus
job that doesn't.

**Me plus an Opus 5.5 helper to decide and review.** I would not add a second decision-maker.
- The owner said it himself on 2026-11-02: "you and your agents are the same so there is not
  guarantee". An Opus reviewer has my blind spots and costs what I cost.
- What actually catches mistakes here is checks that run:
  - the conformance tests;
  - the spec gates;
  - the MSVC, Clang and GCC suites;
  - the laptop speed run.
- Language questions go to the spec and to the owner, not to a second opinion.

### The owner's decision (2026-11-11)

"the goal is the implement 0.9.20's latest spec and any further changes will lead to new hardened
version and then that will turn into the new goal and you can use 3-4 sonnet agents".

- **The goal** is the latest hardened 0.9.20, today 0.9.20_Hardened_12. When the spec changes, the new
  hardened version becomes the goal, and the milestone map is refreshed for what it changed, as in
  Step 0.
- **I lead and write the critical path myself.** Up to 4 Sonnet 5.5 workers run at once, one per lane
  below.

**The lanes** (each job runs in its own git worktree, and none touches the critical path's files):

| Lane | What | Why it suits a worker | Done when |
|---|---|---|---|
| Tests | Respell, rewrite or retire the old tests of the rules a milestone changes (the map: 1,016 to respell, 296 to rewrite); write conformance tests for rules that have none (317 of 838) | Bounded files; the spec text is the input; I review tests faster than compiler code | Each test fails on the old behaviour and passes on the new; I check each against its rule's text |
| Error pages | The 114 codes without a page | Writing from the registry and the rules, no compiler change | `error_pages` gate green; I spot-check against the rule |
| std | std pieces in Ember (the ECS rewrite A8-3, the maths types) | Its own files; the compiler checks it | Its tests green on both backends |
| LLVM backend | Features the C backend has and LLVM lacks (phase 9: 38%) | Its own crate; the C backend's output is the answer | Same output on both backends, speed run passes |

**How each job runs:**
- **A written job sheet** in `tasks/96/redesign/jobs/`: the rules, the files it may touch, the tests
  that must pass, and what not to do.
- **Where it runs.** The worker runs at `high` effort, in the background, in its own worktree.
- **Its build folder** is `target/w1` to `target/w4`, inside the repository, since a build under
  `%TEMP%` trips Defender. Each is cleaned after its job, so the disk does not fill: C: has 143 GB
  free.
- **Machine load.** The laptop has 24 threads, so four builds at once fit.
- **Speed runs** happen only while no worker is building, so that timings stay clean.
- **After a stopped worker,** its process tree is ended, so that no MSVC dialog is left open.
- **Review.** I read every diff, run the checks myself, and merge only what passes. Since the merges
  run through me, the workers' pace is set by my review.

**Fallback.** If a lane's Sonnet jobs keep needing redos, I tell the owner, and that lane can
instead take an Opus 5.5 worker at `medium` effort. A lower effort on the stronger model can cost no
more per finished job.

### The first round is a small one

Per the owner's rule of 2026-11-02 ("pilot before scaling"), the first round is one small job per
lane, four workers at most. Examples:
- 10 error pages;
- the old tests of one rule M3 changes;
- one std piece;
- one LLVM backend feature.

For each job I record:
- the worker's tokens and my review tokens;
- what my review found;
- the redos;
- whether the checks are green.

I show the owner these numbers before the lanes take bigger jobs.

## What does not change

- The spec rules, word for word.
- C speed, measured on the laptop.
- GCC checks with every batch.
- One question to the owner at a time.
- No workers while the owner is away unless he says so (`docs/AUTOPILOT.md` §1: no fan-out
  unattended).
- Pushes only on the owner's word.
