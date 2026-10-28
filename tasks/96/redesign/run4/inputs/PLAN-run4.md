# Run 4 of the memory-model rework: the plan (draft, 2026-10-25)

**Draft for the owner. Not approved, not started.** Owner, 2026-10-25, on "first revise the proposal with
today's decisions and fix the 5 known holes, then run the safety search on it": "yes sir".

## Goal

Revise run 3's proposal (`run3/combined-v4.md`) with the owner's decisions of 2026-10-25, close its known
problems by simplifying the rules rather than patching them, and then do the safety search that run 3
never did: search the whole revised proposal for programs it wrongly accepts.

## What changes from run 3, and why

1. **Every form has "finished: yes or no" and the reason.** "No" is retried once; a second "no" stops
   the run and says why. (Run 3: both safety agents were stopped at the start and their empty forms were
   read as "no problems".)
2. **The safety agents' instructions say plainly what the work is:** checking the owner's own language
   design for bugs before it is built, by writing small test programs the rules should refuse and running
   their C translations under the memory checker. They are called "safety checkers", not "attackers".
3. **Fixes are not patches.** A fix that needs a new special case in a rule is not made; the rule is
   rewritten as a whole, or the problem goes to the owner. Every fix is checked for new holes before it
   is kept. (Run 3: fixes found 48, then 12, then 11 problems, and opened 9 holes.)
4. **The rules are written clean:** no "Was: ..." history inside a rule; history goes in a separate
   changes file. (Run 3's rule R2 had grown to a page.)
5. Kept from run 3: the shared brief, frozen inputs, automatic checks, power watcher, keep-awake, the
   smoke test, all of a phase at once, no pauses.

## Inputs

`JOB.md` (with the owner's verdicts of 2026-10-25), `run3/combined-v4.md`, `run3/vocabulary-final.md`,
`FINDINGS-run3.md`, run 3's open problems (G-01 to G-11 in `run3/22-re-attacker.md`), and the
faithfulness reviewer's two model contradictions (`run3/24-faithfulness.md`: the test on every `weak`
step in W2; one box per captured variable in W4).

The owner's decisions to apply (all in `JOB.md`):
- a call that changes an argument says so: `bump(ref mut a)`;
- no `self`: inside a method the object is `this`; a method that changes its object says `ref mut this`;
- a new word for a changeable link that others may share (owner's suggestions: `sharedref mut`, `sref mut`);
- `if Some(ref mut me) = ui:` reworked into something readable;
- agreed rows 3 to 9 kept, rows 3 and 7 without `self`.

## Phase 1: Revise (7 agents)

1. **Rule writer** (Opus 5.5, xhigh): rewrites the rules (run 3's R1 to R17) cleanly with the owner's
   decisions; settles the new spellings (`this`, the shared changeable link, the replacement for
   `if Some(...)`); redesigns the rules behind the 5 open holes (R2, R10, R11, R12) as a whole so that
   none of the 5 programs is accepted; writes `run4/rules.md`, `run4/vocabulary.md`, `run4/changes.md`.
2. **Five area updaters**, one per area A to E (Opus 5.5, high): bring every answer of their area (84 in
   all) to the new rules and words; mark every answer whose meaning the new rules change; keep the
   eight-part form.
3. **Readability checker** (Opus 5.5, high): the owner's tests on every NEW block, the error-message
   rule, the agreed rows, and the owner's readability verdicts.

Automatic checks: all 84 answers present; agreed rows unchanged; each of run 3's 5 hole programs listed
as refused, with the rule that refuses it.

## Phase 2: Safety search (6 agents, then fixes)

1. **Six safety checkers** (Opus 5.5, xhigh), each on the whole rules plus one part: areas A to E, and
   one on how the areas interact (sharing across areas, threads). Each writes test programs the rules
   should refuse, and proves every hole it finds with the memory checker.
2. **Fixer** (Opus 5.5, xhigh), then **fix checker** (Opus 5.5, xhigh): at most 2 rounds; fixes follow
   change 3 above; whatever is still open after round 2 goes to the owner.

## Phase 3: Measure again (3 agents, 4 at worst; nothing else running)

C writer, faithfulness reviewer (and the C writer again once if needed), timer: as in run 3, on W1 to W4,
because the rules behind `weak` links and captured variables change in phase 1.

## Phase 4: Write-up (the lead)

## Size

Phase 1: 7. Phase 2: 6 + 2, and 2 more at worst. Phase 3: 3, and 1 more at worst. Smoke test: 1.
**19 agents, 22 at worst.**

## Not in this run

The error-message audit (run 3's readers listed about 300 messages that do not say what to write
instead) is its own job.
