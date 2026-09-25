# Multi-agent work: the owner's rules

Copied from the owner's `lean-agent-workflow` skill (2026-09-25) so any session on this repository has it.
It applies only when the owner is present and has approved the plan; unattended sessions work
solo (`docs/AUTOPILOT.md` §3).

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
   - Model: any available model except **Fable**, which needs the owner's special approval and is
     reserved for extremely heavy reasoning and research. Scale by difficulty: **Opus 4.8** for
     absolutely easy jobs, **Opus 5.5** for absolutely large-scale ones, something in between as the
     job suggests. Use the model names the harness accepts; check before assuming.

4. **Keep concurrency at 2-3.** When the agent count is high, do not run many at once. Put a
   limiter in the script (a small worker pool; the harness default is far higher). Ask the owner
   whether they want everything at once or 2-3 at a time - their answer depends on how much of the
   5-hour usage limit is left.

5. **Get the plan reviewed before launching.** Present it, wait for approval or modifications, then
   start. Approving a shape is not approving a size: give the worst-case agent count in the question.

## Plan to present (template)

- **Goal**, one line, in plain words.
- **Jobs**: a table - job, why it is its own job, agents, model, effort, read-only or not.
- **Phases** in order, each ending in a pause/report (see below). Worst-case agent count per phase
  and in total.
- **Concurrency**: proposed limit (2-3) and the question "all at once or 2-3 at a time?".
- **Outputs**: where each phase's results are saved and what the owner gets at the end.
- **What happens between phases** (the owner may set a break length; they can waive it).

## Practices that made the reference run efficient

- **One phase per launch.** Each phase is its own Workflow run (or a `parallel()` barrier with a
  stop point), so it can be halted, reported and resumed from cache. Report after every phase.
- **Structured output.** Give every agent a JSON schema; enums for categories; require evidence with
  line numbers and an honest confidence.
- **Save each phase's results to a file** (e.g. `build/<task>/phase1.json`) and let the next phase's
  agents read it, instead of pasting large arguments into prompts.
- **Adversarial second phase.** Verifiers try to disprove each finding and default to "refuted" when
  the evidence is not there; they may add a clearly marked "new" item.
- **Self-contained prompts.** Agents do not see the conversation: give the context, the owner's rules
  that apply (e.g. general solutions only), the exact target and what not to do.
- **Read-only unless the job is to change things.** No builds or runs from agents on the owner's
  machine without asking (runs open windows the owner can see).
- **Biggest jobs first** in the pool so the longest agent is not the last to start.
- **Never kill a running phase on the owner's behalf**; if they ask to stop part of it and that is
  not possible alone, say so and let them choose.
