# 17 Safety attacker, threads and shared objects: stopped

This agent stopped before it finished its duty.

- Power: `PowerLineStatus` printed `Online` at the start.
- Read: `inputs/brief.md`, `inputs/JOB.md`, the duty section 17 of `inputs/duties.md`, and the parts of
  `RUN3/combined-v2.md` and `RUN3/vocabulary-final.md` about threads, `shared`/`thread_shared` values,
  locks, `weak` links and the deferred-drop region (sections 0, 1, 11 AN-21 and AN-30, 12 AN-48 to AN-52,
  13 AN-60 to AN-65, 14 AN-71 to AN-84).
- While it was reading last run's report on how scoped tasks change counts of the parent's objects, a
  safety classifier stopped one of its replies. Under the owner's rule (a classifier denial ends the
  work), it stopped there.
- Nothing was written under `target/redesign3/17-safety-threads/`. No C program was built or run, so no
  problem is proven and none is listed as unproven either.

For the lead: duty 17 was not done. The problem list for agent 19 has nothing from this agent; the
attack on threads and shared objects needs to be run again, or the owner told that it was not.
