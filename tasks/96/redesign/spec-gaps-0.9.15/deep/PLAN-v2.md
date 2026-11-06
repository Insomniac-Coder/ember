# Deep review, method 2 (plan for the owner; nothing spawned before approval)

Why: method 1 patched (one clause per hole; designs +57..95%; holes per breaker 9 -> 9.5). Every agent
reads `deep/RULES-FOR-EVERY-AGENT.md` first and carries `rules_checked`.

## Per group, one round

| Step | Agents | Model, effort | What |
|---|---|---|---|
| 1. Check and cause | 1 | Opus 5.5, xhigh | every high/medium hole so far: real (shown under the rules as written) or refuted (default when not shown); caused by this rule or elsewhere (names the group or rule); the real ones grouped by root cause; for each cause, the existing rule or verdict that already answers it |
| 2. Redesign | 1 | Opus 5.5, xhigh | from the first design and the causes (not the hole list): one rule per cause, existing tools first, no longer than the first design's rule; an ODR where the owner's requirements cannot all hold |
| 3. Gate | script + 1 | length check in the script (free); Opus 5.5, high | rejects a clause for one case, a broken rule, an untrue `rules_checked`; one redo by the designer, then `unresolved: method` |
| 4. Break | 2 | Opus 5.5, xhigh | safety and engineering, as before; each hole marked caused-by-this-rule or elsewhere |

The next round's step 1 checks step 4's holes; after the last round a final step 1 decides the status:
**holds** (no real high or medium hole caused by the rule), **ODR** (the rest is the owner's decision),
**unresolved** (with the program), **unverified** (a breaker cut off; reported, never retried reworded).
At most 2 rounds. Holes marked "elsewhere" go to the group they name and to phase 5.

Agents per group: 6 if it holds after round 1; at most 15 (2 rounds, each with a gate redo).

## Order

1. **Pilot: P02, P03, P05** (memory managers, one value one place, where `drop` runs). They have round-1
   results of method 1, so method 2 is compared directly. 18 to 45 agents. Then a report to the owner:
   did it converge, did the rules stay one rule and no longer, what went to ODRs.
2. Only if the pilot converges: the other 25 groups in batches the owner sets (150 to 375 agents).
3. Phase 5 (`deep/phase5-plan.md`, approved).

Concurrency: the owner decides (asked before launch). Inputs: `deep/phase2-4-partial.json`. Outputs:
`deep/v2/<group>.json` per group, the report per batch.
