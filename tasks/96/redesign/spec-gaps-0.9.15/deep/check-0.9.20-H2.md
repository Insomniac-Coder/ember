# First check of 0.9.20_Hardened_2 against the open concerns (2026-11-03; owner: full check "next week")

Branch `origin/spec-0.9.20-hardening` (head 1a6459e8; also `ccr-f5aedc9d-5l4mab`), a cloud session's work
of 2026-11-03: 0.9.20_Hardened_1 (an audit of 0.9.15_Hardened_10) and Hardened_2 (a Fable workflow;
209 rules changed, 18 new). **Based on origin/main 5001ef9e (Hardened_10)**: it lacks the local commit
4de795c4 (Hardened_11, verdicts 161-163), the uncommitted verdicts 164-166 in JOB.md (its JOB.md ends at
160), and this folder's sweep (76 clusters, untracked). By its own record it reverses the written-manifest
half of verdict 135 (N83) and touches verdicts 101 and 114 (N87).

## The owner's verdicts 161-166 in its text

| Verdict | In 0.9.20_H2 |
|---|---|
| 161 `owns` binds as `-` | yes, `[GRM-47]`, `E0119` (H11 had `E3028`) |
| 162 `runtime` before a class type | yes, `[LT-48]` (open/abstract classes) |
| 163 `runtime` on a variable | yes, `[LT-48]` (a local declared with its type) |
| 164 `sharedref mut` neither Send nor ThreadShared | yes, `[THR-8]`, `[THR-9]` |
| 165 `thread_shared` refused where made unless ThreadShared | **no**: `[THR-1]` asks `Send` only at the making line |
| 166 a scoped task neither makes nor ends a `shares` owner | yes, p11 "A task touches no one-thread count" |

## The five questions the trials worked on

K04 fixed (`[ALC-5a]`); K05 fixed another way (`thread_shared T using A` with a non-ThreadShared `A` is
`E7015`); K07 not fixed (`[FFI-10]` still covers only a raw-pointer parameter or result); K20 not fixed
(`[ERR-3]` still `Display + Debug`, no `Send`); K10 unclear (`[RC-4]` point 3 changed; no answer found to
a queued drop run inside a held lock).

## The 76 clusters, first pass (rules changed, not "answered")

62 had at least one of their rules changed, 13 none (high: K21 C++ overloads), 1 with no rule found.

## Next week (owner)

Read each of the 76 clusters against 0.9.20_H2 (answered / not / answered differently), plus verdict 165,
K07, K20, K10, and decide how the branch and the local work (H11, JOB.md 164-166, this folder) come together.
