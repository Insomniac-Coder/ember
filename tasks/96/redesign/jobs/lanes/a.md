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
- **State:** slice A and B built (H2, round 6); slice C measured only (581 copies in 289 programs, 65 in std;
  `count_handle_copies.py`); slices D (`copy of`) and E (`thread_shared`) not started.
- **Traps:** a `shares` value reaching a thread is E7004 where the spec says E7015 (slice E);
  `[HR-15]`'s reload test is ignored until `weakref thread_shared`.
