# Handoff: the memory-model rework, paused 2026-10-26 (for a cloud session)

The owner had to leave and asked for all work to stop, everything noted, and the work committed and
pushed to a separate branch so a cloud session can pick it up. Nothing was lost; nothing is half-edited.

## Read first, in this order

1. `JOB.md`: the goal and every owner verdict. The newest are "Owner verdicts on run 4's open
   questions", items 1 to 34 (25 is replaced by 27). Verdicts are binding.
2. This file.
3. `PLAN-run4-step2.md`: the approved plan in progress ("all at once, approved").
4. `SAFETY-VALIDATION-TASK.md`: the owner's task for after step 2 ("both, b first").
5. `run4/v3/inputs/brief.md` and `run4/v3/inputs/duties.md`: the agents' frozen brief and duties.

## Where things stand

- Run 3 (26 agents): `FINDINGS-run3.md`, `run3/`. Run 4's revise phase: `run4/*.md`. Step 1 applied
  verdicts 1 to 20: `run4/v2/` (its readability check `run4/v2/readability.md` lists 14 failures).
- The owner then answered 12 more questions: verdicts 21 to 34 in `JOB.md`.
- **Step 2, bringing the design up to date with 21 to 34, as `run4/v3/`:**
  - **Phase 0, done (the lead):** `run4/v3/` holds copies of `v2`'s `rules.md`, `vocabulary.md` and
    `area-A.md` .. `area-E.md`, with `holds` replaced by `exists as` in code only (88 lines), made by
    `run4/v3/tools/phase0_spell.py`; every changed line and every line still containing "holds" is in
    `run4/v3/spelling-changes.txt`. Two quotes of today's compiler output (probe `q1_holds.em`) were
    restored by hand. Inputs frozen in `run4/v3/inputs/` (brief, duties, `JOB.md` with the 34 verdicts).
  - **Phase 1, stopped at its very start:** the rule writer had changed nothing (it had only made an
    empty scratch folder). `run4/v3/` is exactly the phase 0 state; there is no `run4/v3/changes.md` yet.
  - **To resume phases 1 and 2:** run `run4/v3/tools/step2_workflow.js` (a Workflow script: the rule
    writer at xhigh effort, then the five area updaters all at once, A to D high, E xhigh; every form has
    `finished`, a not-finished agent is retried once, then the run halts). Its paths, and the paths in
    `run4/v3/inputs/brief.md` and `duties.md`, are Windows paths under `C:\Users\ism19\Code\ember\`:
    replace them with the checkout's root. The brief's "today's Ember" is
    `target/redesign4/bin/ember.exe`, a copy of the `ci`-profile build of commit 703d180f (the compiler
    has not changed since; later commits are documents only). In a cloud checkout, build the compiler
    with `cargo build --profile ci` and point the brief at the built binary. The brief's power check is
    Windows-only (the laptop); drop it in the cloud.
  - **Phase 3 (the lead, no agents):** fix the error-message failures of `run4/v2/readability.md`
    (F-01 to F-11 and F-13, listed below) in the `v3` files, plus any message the area updaters flag.
  - **Phase 4 (one agent):** the readability checker, duty 07 in `run4/v3/inputs/duties.md`, writes
    `run4/v3/readability.md`.
  - **Then report to the owner:** what changed per verdict, the readability result, and any new
    question, one at a time.
- **After step 2:** plan (b), the safety-invariant matrix and conformance-format tests for the `v3`
  rules (`SAFETY-VALIDATION-TASK.md`), and show the plan to the owner before building anything; then (a),
  the pipeline on today's compiler. A measure phase is still owed (W2's parent-link counting is
  unmeasured; verdicts 14 and 28 ask for the cost to be removed by compile-time proof where possible and
  measured where not).

## Phase 3: the message fixes (from `run4/v2/readability.md` section 3; find the blocks by content)

- **F-01:** messages say "name it again: `boss = ref mut enemies[0]`" on a name already declared
  `ref mut`, which R2 refuses. Give a new name (`boss2 = ref mut enemies[0]`) or a block of its own.
  Rules E3060 row; AN-01, AN-02 (three messages), AN-16's table row, AN-61 (`h = ref mut ...`).
- **F-02:** AN-23's E2140 says `n = first_value(xs)` then `n += 1`, which still loses the change. Give
  both ways out of verdict 13: `xs[0] += 1`, or `-> ref mut int` with `first(ref mut xs) += 1` (AN-10's
  message is the model).
- **F-03:** E3026's help names a `Counter` with a field `n` the program does not have (rules row; AN-53;
  AN-57). Give the lines, including the class and the button's list type.
- **F-04:** the rules' E3063 row says `target: shares Player` for watchers that change their players;
  it must be `target: shares mut Player` (AN-52).
- **F-05:** messages use a list or a lock the program does not declare (`spawned`, `dead`, `joined`,
  `new_items`, `FRAME_LOCK`, `HITS_LOCK`): add the declaring line (`spawned: Array[Enemy] = []`,
  `static FRAME_LOCK: Lock = Lock()`); AN-72's message is the model.
- **F-06:** ways out given in words, not lines: AN-07, AN-11, AN-13, AN-16, AN-24, AN-25, AN-31, AN-42,
  AN-53, AN-58, AN-77, the rules' E3023 row. Write the exact line.
- **F-07:** AN-36's L3001 tells the programmer to make the owning list weak; name the back link and give
  the line (as AN-64 does: `best: weakref Person = None`).
- **F-08:** AN-11's E3023 calls `integrate(ref mut ps, vs, 0.5)`; the program has `p` and `v` and needs
  `ps = p.as_mut_span()` first (and, with verdict 34, `ps = ref mut p.as_mut_span()`).
- **F-09:** AN-17 asks the caller to rewrite another package's `note`; give a line for the calling
  package (call `note` after the loop).
- **F-10:** AN-24's refused example names a view struct `Handle`, a word kept for tokens; rename it.
- **F-11:** E7007 (AN-74, rules row) says `with this.lock:` in a free function; it is `with stats.lock:`.
- **F-13:** the vocabulary's `from` row still writes `from(self)`; with no `self` it needs a spelling
  (`from(this)` or another; say which).

## Working with the owner (from the lead's notes; read before talking to them)

- **Before any agents or workflow:** present a plan (goal; a table of jobs with why each is its own
  job, model, effort; phases; worst-case agent count; concurrency) and wait for approval. Default 2-3
  agents at once unless the owner says otherwise (for step 2 they approved all five at once).
- **Standing rules:** if a safety classifier stops something, stop and report; never reword or route
  around it. In this rework: no compiler changes, `ember check` only, never run an Ember program. No
  downloads or installs; if a tool is missing, ask the owner with the exact command. Commit only when
  asked. Never touch the RageV project. Error messages are part of readability.
- **Explaining (lessons that cost many hours):**
  - Answer the literal question first, fully. Never dodge or set a question aside ("you never write
    that line" got "are you going to explain it or no?").
  - The problem first, shown with code; then each option as the code you would write and what happens;
    then one recommendation; one question at a time; "okay" is not a verdict, ask for the letter.
  - Full code with every name defined, and show where each name comes from (for "node 1 .. node 4",
    show `main` making them). No inline-comment clutter in code.
  - Never change the example in the middle of a question; never refer to examples by number.
  - When a C++ concept exists, name it first (the owner writes C++): "a template parameter, like
    `template <typename T>`" landed after seven failed tries. Then say how Ember differs.
  - Build a complex version from the simple one, one change at a time, each with its reason.
  - Never add an untested option mid-question, least of all after the owner has chosen; if their choice
    has a risk, state the risk plainly and stop.
  - Never mention time, tiredness, or offer to stop or pause.
  - Plain words; short tables of "what happens" work; text-art trees do not; avoid visuals unless asked.
  - The owner's test (in `run4/v3/inputs/brief.md`) is how they judge every line.

## Repository state

- This branch is `pause/redesign-run4-step2`: local `main` (561143a1, which is 90 commits ahead of
  `origin/main` and was never pushed, because a push of `main` needs the full check and the laptop
  benchmark first) plus one commit with the rework's files (`JOB.md`, plans, findings, `run3/`,
  `run4/` with `v2/` and `v3/`, `artifacts/redesign2/split/`, this note).
- Not committed: `.claude/` (worktrees of the helpers the owner stopped on 2026-10-24; see
  `tasks/96/helpers-stopped.md`) and `target/` (ignored: compiler copies, probes, demos).
