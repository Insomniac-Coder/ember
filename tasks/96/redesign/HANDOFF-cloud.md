# Handoff: the memory-model rework, run 4 step 2 done (2026-10-26, cloud session)

The owner paused the work on the laptop on 2026-10-26 and a cloud session picked it up from the branch
`pause/redesign-run4-step2`. That session finished step 2 (phases 0 to 4), asked the owner the open
questions (verdicts 35 to 52), and reported. Nothing is half-edited; everything is committed and pushed.

## Read first, in this order

1. `JOB.md`: the goal and every owner verdict. The newest are "Owner verdicts on run 4's open
   questions", items 1 to 52 (25 is replaced by 27). Verdicts are binding.
2. This file.
3. `SAFETY-VALIDATION-TASK.md`: the owner's task for after step 2 ("both, b first"), the next work.
4. `run4/v3/changes.md`: what step 2 changed, verdict by verdict (sections 1, 2 and 10a).
5. `PLAN-run4-step2.md`: the plan step 2 followed ("all at once, approved").

## Where things stand

- Run 3 (26 agents): `FINDINGS-run3.md`, `run3/`. Run 4's revise phase: `run4/*.md`. Step 1 applied
  verdicts 1 to 20: `run4/v2/` (its readability check `run4/v2/readability.md` lists 14 failures).
- **Step 2 is done.** The current design is `run4/v3/`: `rules.md`, `vocabulary.md`, `changes.md`,
  `area-A.md` to `area-E.md` (the 84 answers), `readability.md` (agent 07's check), `phase3-messages.md`
  (every message the lead fixed), `spelling-changes.txt` (phase 0). Inputs frozen in `run4/v3/inputs/`
  (brief, duties, `JOB.md`, kept identical to `../JOB.md`).
  - **Phase 0 (the lead):** the spellings in code: `holds` -> `exists as` (88 lines,
    `tools/phase0_spell.py`), then `if cast X if T exists as n:` -> `if X is T as n:` and `.copied()` ->
    `.copy_each()` (18 lines, `tools/phase0b_spell.py`). Today's Ember (OLD code, pasted outputs, "means
    exactly") keeps the old spellings; the 33 lines left, each with its reason, are in
    `spelling-changes.txt`.
  - **Phases 1 and 2 (6 agents, `tools/step2_workflow.js`):** the rule writer (xhigh), then the five
    area updaters at once (A to D high, E xhigh), for verdicts 21 to 34. Their forms are in
    `run4/v3/step2-forms.json`.
  - **The owner's questions from phases 1 and 2, then a survey of the hardest lines:** verdicts 35 to
    52, applied by the lead to the rules, the vocabulary, `changes.md` (section 10a) and the answers
    (verdict 50's spelling by `tools/v50_template_line.py`). Evidence for them: `run4/v3/probes/`
    (C++ template against `std::function`; Rust's spawn in a loop and join inside a lock).
  - **Phase 3 (the lead, no agents):** every error-message failure of `run4/v2/readability.md` (F-01 to
    F-11 and F-13) and every message the agents flagged; F-01 was settled by verdict 42 instead of
    rewritten. Each fix is a row of `run4/v3/phase3-messages.md`.
  - **Phase 4 (one agent, `tools/step2_phase4.js`, high):** the readability checker on verdicts 1 to
    52. Result below; the lead fixed its 15 failures and the 4 of its notes that were stale text, each a
    row at the end of `phase3-messages.md`. `readability.md` is left as the checker wrote it.
- Today's compiler, unchanged: `target/redesign4/bin/ember check` (a `ci`-profile build of 703d180f; in a
  fresh cloud checkout build it with `cargo build --profile ci` and copy it there; `target/` is ignored).

## Report to the owner (given 2026-10-26)

**tl;dr:** step 2 is done. Verdicts 21 to 52 are applied to the rules, the vocabulary and all 84
answers. The independent readability check found no old spelling in any new code; it found 15 places
that did not yet match your decisions, all fixed. The compiler did not change. No new questions.

What each verdict changed:

*Ownership and names*
- 23: after `other = shares mut items`, both `items` and `other` may change it.
- 24: the field says the kind of arena; the line that makes the value says which one (`using current`).
- 32: the word in front of a parameter's name sets what the function may do with it, for every type.
- 34: a span name or parameter that writes says `ref mut`.
- 42: a `ref mut` name may be pointed at something else; a `const ref mut` name never is.
- 47: a value made in an arena keeps a pointer to its arena (8 bytes), accepted for now, not measured.

*Saying what a function changes*
- 21, 38: a library function given a function takes it as a template parameter, and the call counts as
  making that function's changes.
- 22, 35, 40: a lambda names the variables it changes; a stored list of functions names a variable it
  can see by name, and `captured` / `captured...` for one it cannot.
- 29, 44: a change made under a lock is named on the first line; `guarded by` follows `Mutex`'s rules.
- 30: starting a thread counts as making what its function changes.
- 31, 39: calling C counts as running the callbacks C keeps, so `tick` says `changes Game, LOG`, unless
  the C function is marked `@ffi(no_callbacks)`.
- 36, 37: sending on a channel, or receiving through a given channel end, is named on the first line.
- 41: a named change list, `world: changelist = (Game, LOG)`, strict.
- 45: a `=>` lambda says what it changes, as a `:` lambda does.
- 46: a `dyn` type may say what its values change.

*Spellings*
- 26: `holds` became `exists as`.
- 27: `if s is Circle as c:`; `cast x if D` and `cast x when D` are gone (25 not applied, replaced by 27).
- 33: `.copied()` became `.copy_each()`.
- 48, 49: a function type written often or long is named once with `type`.
- 50: type parameters go on their own line above every generic function, method, struct and class.
- 51: a long lambda may be written in the call or as a named local function.

*Safety*
- 28: a walk whose visit function may delete nodes checks a `weakref` every step.
- 43: a call that may take a lock the thread already holds is refused when compiling.
- 52: a thread may be started in a loop over what it changes when the loop holds that lock; joining it
  inside the lock is refused.

Error messages: every one now names what was refused in the program's own names and gives the exact
lines to write; where a refused program was never written out, it now is, so the line numbers can be
checked.

Open, but not questions for the owner yet: four points of the rules the checker noted (its section 7,
notes 2, 4, 5 and 8; listed at the end of `phase3-messages.md`), for example which of two messages
comes first when one mistake fits both (E3020 and E7018).

## Next

- **Plan (b) of `SAFETY-VALIDATION-TASK.md`:** the safety-invariant matrix and conformance-format tests
  for the `v3` rules. Show the plan to the owner before building anything; then (a), the pipeline on
  today's compiler.
- **A measure phase is still owed:** W2's parent-link counting is unmeasured; verdicts 14 and 28 ask for
  the cost to be removed by compile-time proof where possible and measured where not; verdict 47's
  arena pointer is not measured either.
- The rules' four open points above, when the design is next revised.

## Working with the owner (from the lead's notes; read before talking to them)

- **Before any agents or workflow:** present a plan (goal; a table of jobs with why each is its own
  job, model, effort; phases; worst-case agent count; concurrency) and wait for approval. Default 2-3
  agents at once unless the owner says otherwise (for step 2 they approved all five at once).
- **Standing rules:** if a safety classifier stops something, stop and report; never reword or route
  around it. In this rework: no compiler changes, `ember check` only, never run an Ember program. No
  downloads or installs; if a tool is missing, ask the owner with the exact command. Commit only when
  asked: the cloud's stop hook asks for a commit after every turn; do not commit for it, say so in one
  line. Never touch the RageV project. Error messages are part of readability.
- **Explaining (lessons that cost many hours):**
  - Answer the literal question first, fully. Never dodge or set a question aside ("you never write
    that line" got "are you going to explain it or no?").
  - The problem first, shown with code; then each option as the code you would write and what happens,
    before and after; then one recommendation; one question at a time; "okay" is not a verdict, ask for
    the letter.
  - Full code with every name defined, and show where each name comes from (for "node 1 .. node 4",
    show `main` making them). No inline-comment clutter in code.
  - Never change the example in the middle of a question; never refer to examples by number or to code
    not shown in the same message. If the owner asks for an earlier example, show that exact one again.
  - When a C++ concept exists, name it first (the owner writes C++): "a template parameter, like
    `template <typename T>`" landed after seven failed tries. Then say how Ember differs.
  - Build a complex version from the simple one, one change at a time, each with its reason.
  - Never add an untested option mid-question, least of all after the owner has chosen; if their choice
    has a risk, state the risk plainly and stop.
  - Never record an answer the owner did not give, and check a file's state before saying what was or
    was not written to it.
  - Keep replies short, tl;dr first: the owner reads long replies as clutter.
  - Never mention time, tiredness, or offer to stop or pause.
  - Plain words; short tables of "what happens" work; text-art trees do not; avoid visuals unless asked.
  - The owner's test (in `run4/v3/inputs/brief.md`) is how they judge every line.

## Repository state

- The cloud session's work is on `ccr-b1e28856-1ir3qy` (pushed): `pause/redesign-run4-step2` (e503bf04)
  plus the session's commits, documents only. `pause/redesign-run4-step2` itself is local `main`
  (561143a1, 90 commits ahead of `origin/main` and never pushed, because a push of `main` needs the full
  check and the laptop benchmark first) plus one commit with the rework's files.
- Not committed: `.claude/` (worktrees of the helpers the owner stopped on 2026-10-24; see
  `tasks/96/helpers-stopped.md`) and `target/` (ignored: compiler copies, probes, demos).
