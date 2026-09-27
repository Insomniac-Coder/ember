# Multi-agent work: the owner's rules

Copied from the owner's `lean-agent-workflow` skill (2026-09-25) so any session on this repository has it.
**Owner update, 2026-09-27:** delegation is the default for separable implementation,
tests, documentation, routine fixes, test runs/checks, and failure triage. This
supersedes the earlier solo-work, per-plan approval, and mandatory phase pauses below.
Root uses Astra for architecture, ODRs, review, and subagent guidance. Choose Sol or
Luna dynamically by task complexity: Sol for substantive compiler behavior and deeper
debugging; Luna for bounded docs, probes, routine runs, and small, well-specified
mechanical code changes. Reassess and escalate when task complexity changes. The lead
owns requirements, architecture/design, review, integration, and final
verification. Keep jobs bounded and one owner per file and run. The runner reports
failures; the lead assigns fixes by complexity. Default to 2–3 concurrent agents without
asking again for routine splits.

# Lean agent workflow

The owner set these rules on 2026-09-24 after the RageV RT-24 constants audit, which they judged
the most efficient workflow so far (13 agents across two phases, max 2 running, ~0.9M subagent
tokens for 262 values classified and 53 re-checked). Follow them for every multi-agent task.

## The five rules

1. **Understand the requirement, then split it into jobs that make sense.** A job is a piece of work
   with its own input and its own output that one agent can hold in full: one file, one subsystem,
   one question. Read enough yourself first (sizes, file list, what already exists) to split well.

2. **Justify the count.** Find the sweet spot between coverage and cost. For every split, ask:
   - Is this split actually needed, or is it two jobs that one agent could do as well?
   - Would the investigation quality drop if the agent count went down? If not, merge.
   - How hard is each job? Small or easy targets merge; big or hard ones stay alone.
   - Does a job have any input at all? Drop agents with nothing to do (e.g. a file with zero
     flagged items gets no verifier).

3. **Size each agent to its job: model and effort.**
   - Effort: the lowest level that does the job well - low / medium for small mechanical reads,
     high for normal files, xhigh ("extra") for large or subtle ones. Unless the owner sets a cap,
     never go past what the job needs; state each agent's effort in the plan.
   - Model: use the harness's available names and match current task complexity. Luna
     suits bounded mapping, probes, documentation, and routine checks; Sol suits compiler
     implementation and deeper debugging; Astra owns difficult architecture, ODRs,
     review, and delegation guidance. Reassess and escalate as work changes.

4. **Keep concurrency at 2-3.** When the agent count is high, do not run many at once. Put a
   limiter in the script (a small worker pool; the harness default is far higher).
   Reduce concurrency when independent useful work is unavailable.

5. **State the split and proceed.** The owner authorized the lead to choose
   routine job/model assignments. Keep the user informed; pause only when the
   owner requests it or a decision requires authority not already delegated.

## Brief plan to report (template)

- **Goal**, one line, in plain words.
- **Jobs**: a table - job, why it is its own job, agents, model, effort, read-only or not.
- **Phases** in order, each ending in a progress report. Worst-case agent count per phase
  and in total.
- **Concurrency**: use 2-3 as the default limit, fewer when sufficient.
- **Outputs**: where each phase's results are saved and what the owner gets at the end.
- **What happens next**, unless the owner has requested a pause.

## Practices that made the reference run efficient

- **Bounded phases.** Keep clear completion points so work can be reviewed and
  resumed without repeating investigation. Report progress and continue.
- **Structured output.** Give every agent a JSON schema; enums for categories; require evidence with
  line numbers and an honest confidence.
- **Save each phase's results to a file** (e.g. `build/<task>/phase1.json`) and let the next phase's
  agents read it, instead of pasting large arguments into prompts.
- **Adversarial second phase.** Verifiers try to disprove each finding and default to "refuted" when
  the evidence is not there; they may add a clearly marked "new" item.
- **Self-contained prompts.** Agents do not see the conversation: give the context, the owner's rules
  that apply (e.g. general solutions only), the exact target and what not to do.
- **Read-only unless the job is to change things.** A designated verification
  agent may own builds and checks, using the Windows safeguards in AUTOPILOT.
  Do not launch competing builds or mutate files during the full suite. Give
  the runner the command, timeout, output path and reporting expectations;
  process sessions belong to the agent that launched them.
- **Biggest jobs first** in the pool so the longest agent is not the last to start.
- **Never kill a running phase on the owner's behalf**; if they ask to stop part of it and that is
  not possible alone, say so and let them choose.
