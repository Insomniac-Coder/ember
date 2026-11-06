# Phase 5: the whole-language check (approved by the owner, 2026-11-02 ~13:00)

Owner: "I approve the phase 5 design but the concurrent agent count is something I'll decide later, ask me
about it once the current workflow is over". So: when phases 2-4 end, ASK the concurrency first.

Why not one agent: the 28 designs are ~300k tokens together (median 42.7k chars each); 179 of the 378
group pairs touch the same spec rule; clashes between rules were ~18% of the serious holes (49 of 279 in
the first 27 breakers).

| Step | Who | What | Agents |
|---|---|---|---|
| A. Overlap map | the lead, by script | from the FINAL designs, which groups touch the same rules; pick neighbourhoods of 5-7 groups so every pair of groups sharing a rule is read together by at least one checker; show the owner the coverage before launch | 0 |
| B. Clash checkers | one per neighbourhood | read those designs in full + the spec parts they touch: two rules deciding one program differently, a word meaning two things, a case covered by neither, a verdict clash; each with the program | ~6 |
| C. Whole-program walkers | safety + engineering | read all 28 rules' spec_text (~60k tokens); write 3 realistic programs using many features at once (game frame: threads, arenas, dyn, C calls; a server; a parser with generators and errors); walk every line through every rule; check the owner's ease bar on the whole language | 2 |
| D. Fix what they find | the phase 2-4 repair prompt, the clash as its hole | only the groups in a clash; then re-check only that neighbourhood; one round; what is left goes to the owner as an open item with its program | 0 to ~10 |

Cost: 8 certain, ~18 at most; ~100k output tokens each. Same read-only rules, `finished` field, JSON
schemas and owner bar (deep/owner-bar.md) as phases 2-4.

After phase 5 (also approved): the breakers' must-refuse / must-compile programs become conformance tests;
every cost claim measured with C micro-benchmarks (laptop, mains, P-cores); then the ODRs one at a time,
each with the lead's recommendation.
