# Lane a: ownership, views and borrowing (M4 to M7)

The lead's lane. Read first by every job in it; updated last.

- **Milestones:** M4 (shared owners and links, one owner for class objects, `copy of`; plan:
  `tasks/96/redesign/compiler-0.9.15/M4-plan.md`), M5 (views inside a function, loops, spans, kept views,
  generators), M6a/M6b/M6c (summaries, the run-time checks removed with the new header, alias facts), M7 (loops of
  owners). Items: `MILESTONES.md`; their files and tests: `gap/01-ownership.md`, `gap/02-views-borrowing.md`,
  `gap/03-sharing-links.md`.
- **Families:** OWN, BRW, LT, RC, WK, HEAP, CELL, BCK, DRP, CLS, STA, SPN, ARN, ALC, DSJ, HND, RT (158 unfinished
  rules, 32,615 words, on 2026-11-13).
- **Worktree:** `C:\Users\ism19\Code\ember\.claude\worktrees\lead`, branch `lead-m4`. M4 lands on `main` whole,
  after slice C.
- **Where the code is:** `ember_types/src/lib.rs` (`Link` kinds, `is_copy`/`copy_with`), `ember_typeck` (`synth_link`,
  `read_link_through`, `open_link`, `reject_change_through_reader`), `ember_analysis/src/borrows.rs` and `drops.rs`,
  `ember_mir`'s link builtins, `uncounted_handles.rs`, `temporary_owners.rs`.
- **State (2026-11-13):** slices A and B built (H2, round 6); `main` merged in twice (`999fb5ef`), the
  Printable read-through fix (`9d5377d4`), the quick check failing 0. Slice C on branch `m4-slice-c-probe`:
  C1 (a class handle is not `Copy`: 262 programs fail) and C2 (E3040 migrates `m = n` of a class object to
  `m = shares mut n`; 10 tests migrated) are committed. Slices D (`copy of`) and E (`thread_shared`) not started.
- **Slice C waits for M5's A2-3** (`M4-plan.md`, "Found by C2"): the rest of the 262 are reads of a class object
  out of a place (`x = list[0]`, `x = p.child`), which `[BRW-12]` makes views, for every type. Measured: part
  views alone give 3,344 "cannot write through a shared reference" (names written through, `[BRW-12]` clause 6's
  `x = ref mut <part>`, which is not built either: `x = ref mut b.items; x.push(3)` is E3021 today). So the next
  lane-a job is A2-3 whole (`[BRW-12]` to `[BRW-15]`), with a checker migration to `ref mut` and the deputy's
  tests in `tasks/96/redesign/jobs/ahead/M5/BRW-12..15/`, then slice C on top.
- **Traps:** a `shares` value reaching a thread is E7004 where the spec says E7015 (slice E);
  `[HR-15]`'s reload test is ignored until `weakref thread_shared`.
