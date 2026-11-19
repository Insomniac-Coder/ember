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
- **State (2026-11-19):** slices A, B, C and D are built on branch `m4-slice-c-probe` in the lead worktree (`M4-plan.md`, Progress). Next: slice E (`thread_shared`) and `[TYP-36]`'s row for owners, then M4 goes onto `main`.
- **Slice C waits for M5's A2-3** (`M4-plan.md`, "Found by C2"): the rest of the 262 are reads of a class object
  out of a place (`x = list[0]`, `x = p.child`), which `[BRW-12]` makes views, for every type. Measured: part
  views alone give 3,344 "cannot write through a shared reference" (names written through, `[BRW-12]` clause 6's
  `x = ref mut <part>`, which is not built either: `x = ref mut b.items; x.push(3)` is E3021 today). So the next
  lane-a job is A2-3 whole (`[BRW-12]` to `[BRW-15]`), with a checker migration to `ref mut` and the deputy's
  tests in `tasks/96/redesign/jobs/ahead/M5/BRW-12..15/`, then slice C on top.
- **Traps:** a `shares` value reaching a thread is E7004 where the spec says E7015 (slice E);
  `[HR-15]`'s reload test is ignored until `weakref thread_shared`.

## M5's A2-3 (H4, job 1, 2026-11-13): what a name is (`[BRW-12]` to `[BRW-15]`)

- **Where the code is:** `ember_typeck/src/views.rs` (new): part names as read-only views (`part_as_view`,
  `is_part`, `changeable_place`), the view names a line makes (`view_names`, `ViewName`/`ViewKind`: part, `ref`/`ref
  mut`/`const ref mut`, a `const ref mut` parameter, a plain name holding a `MutSpan`), `=` on a view name (`repoint`:
  re-points, or `E3023` for `ref b` given to a `ref mut` name and any ref given to a `const ref mut` one, Q4's `rb`
  name), names that keep their place (`kept_headers`/`kept_header`: loop, `match` and `exists as` names, found by
  their pattern's span), the `E3023` texts (`read_only_change`, `span_holder_write`, `kept_name_given_a_value`, the
  plain-parameter message with each call, `report_parameter_changes`, amended at the end of `check` once every call
  is known), Q5's nouns (`value_noun`), and `[OWN-7]`'s copied `Copy` values (`CopiedValues`, reported at the end of
  `check_body`). The hooks in `lib.rs`: the `x = e` declaration and the `=` paths of `check_stmt`,
  `reject_shared_ref_access`, `reject_borrowed_parameter_write_as`, `check_call_word`, `check_match_scoped`,
  `check_for`, `check_destructure`, `take_pattern_copies` (`exists as copy of v` over a view copies what it views),
  and `RefOf`'s synthesis (`ref mut` before a call giving a tuple of `MutSpan`s). The parser: `const ref mut e`
  (`ast::ExprKind::RefOf { constant }`).
- **Re-pointing needed no analysis change:** the region solver's forward value state (`[LT-21]`) already ends the old
  loan where a view name is given another view (ADR-010 is marked expired in `docs/DECISIONS.md`).
- **The migration:** `lanes/a_migrate_views.py` (`EMBER=… python a_migrate_views.py <file list>`): it runs `ember check
  --json` and applies only this job's migrations (`ref mut` before a part a name changes, `copy of` before one a name
  that only reads changes, `ref mut` before a `MutSpan` a plain name writes through, `copy of` for `[OWN-7]`), so a
  compile-fail test's other migrations stay; applied to every `.em` of `tests/`, `docs/book/` and `examples/`: 26
  files on `main` (class handles are still `Copy` there, and a class field is written through any name (D-747), so
  slice C meets the class parts). By hand: a part name moved on is `x = owns <part>` (`[OWN-11]`: EXP-6, DRP-5,
  EFF-2, LAY-2, the ui snapshots O1 and O2); 0.9.10's write-through `r = ref mut x` on a `ref mut` name is `r = x`
  (BRW-1, SPN-4, SPN-5, ARN-5c, ARN-5d); `std`: `Iterator.fold` takes `owns init` (a plain parameter's view cannot
  become the accumulator; `sum(xs, start)` hands its start over), `get_pair_mut` and `Random.shuffle` split with
  `ref mut`; the messages old tests pinned follow BRW-15's form (CELL-10, CLO-2, DIA-7, DIA-9, FN-2, FN-9, GRM-13,
  SOA-2, ECS-3, the ui snapshot B4) and the first-week mark of `01_text_adventure` has `E3023`.
- **Done:** `[BRW-13]` credited (every clause; `tests/conformance/BRW-13`). Built but not credited: BRW-12 (12.9
  waits for M4's `weakref`; 12.2's temporary case for A1-18; 12.5 for LT-1a/LT-45; BRW-1's text for A2-4), BRW-14
  (kind tests, M11; the `MutSpan` half of 14.8; `Handle`'s no-`Clone` offer, M4), BRW-15 (`Lock`, M9; `shares mut`,
  M4; a `Cell` through a plain name; EXC-15's text needs A2-6's `for ref mut`, the parameter case M6a). The
  deputy's files that wait are listed in `ahead/M5/INDEX.md` §4.
- **Left for A2-3:** the `Cell` refusal (BRW-15.1) with its migration; a `ref mut dst: MutSpan` parameter passed by
  address (BRW-14.8); BRW-1's E3021 text for a part name (`boss = ref mut enemies[0]` … `boss.hp = 1`); a class
  object's field written through a plain name (D-747's exemption; with slice C); a part of a `packed` struct
  (`r.name` is `E2170` as a view: the tests write `owns`/`copy of`; a spec question in H4's report).
- **Traps:** the compiler reads `std/src` from disk while it runs: never edit `std/` or the quick check's folders
  while one runs (a baseline was lost so). `\u{1}`/`\u{2}` are `ember_diag`'s line marks; the provisional message of
  a plain-parameter change starts with `\u{3}`. `for ref mut x in e:` parses but is not built (A2-6), so a loop or
  pattern name whose type is `ref mut T` or `MutSpan` (`iter_mut()`, `chunks_mut`) is not refused as one that keeps
  its place yet. A `ref mut dst: MutSpan` parameter is still passed as the span itself, so giving it another span
  does not reach the caller's `d` (BRW-14.8, an ABI change).
