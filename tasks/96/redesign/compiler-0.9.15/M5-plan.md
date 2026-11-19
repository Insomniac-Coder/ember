# M5: views inside a function, loops, spans, kept views (the lead's plan, 2026-11-19)

The items are `gap/02-views-borrowing.md`'s A2-2 to A2-8 and A2-15 (MILESTONES.md's M5 row). The tests written for
them before the build are `tasks/96/redesign/jobs/ahead/M5/` (INDEX.md: 18 rules, 115 files); a rule's folder moves
to `tests/conformance/<RULE>/` when it is built.

## Where it stands

- **A2-2 done** (`e60c31d8`): a view returned where it would outlive what it points into is `E3063`.
- **A2-4a, reshapes, done** (2026-11-19): a built-in that adds, removes, reorders or replaces what its first
  argument points at (`summaries::reshaping_builtin`), and a write of a whole value that owns memory over a viewed
  place inside it (`reshaped_by_write`), end a read-only view: `E3060` at its later use (`borrows.rs`
  `reshapes_at`). A loop's walk keeps `E3020`, an arena's `reset` and a view a `dyn` call gives keep `E3021`
  (E3021's row of the code table, `[ALC-5]`). 44 older tests moved from `E3021` at the change to `E3060` at the
  use; the ui case moved from B3 to B7. Calls of written functions that reshape wait for the summaries (M6a). The
  bank's 23 passing files are in `tests/conformance/`; BRW-2 credited.
- **A2-4b, `E3021`'s text, done for a named view** (2026-11-19): "`enemies[0].hp = 1` (line 24) changes the enemy
  `boss` reads (`boss = enemies[0]`, line 22), and `boss` is used on line 25; change it after `boss`'s last use, or
  write `boss = ref mut enemies[0]` and change it through `boss`: `boss.hp = 1`" (`ember_diag::line_code_mark`
  quotes a line's code). A view a call made (a temporary borrower) keeps the old text. Left: one mistake given two
  `E3021`s (`xs[0] = 9` while `m = xs.as_mut_span()` is used: a read and a write of `xs`, `[DIA-26]`).
- **A2-3 mostly done** (round 7's H4, `lanes/a.md`): part names are views, re-pointing, names that keep their place,
  `const ref mut`, BRW-13 credited. Left: the `Cell` refusal (BRW-15.1), a `ref mut dst: MutSpan` parameter by
  address (BRW-14.8), BRW-1's `E3021` text for a part name, a part of a `packed` struct.
- **The bank against today's compiler** (after M4, 2026-11-19): 20 of 86 files pass. The failures, by cause:

| Cause | Files | Item |
|---|---|---|
| a reshape (push, remove, clear, insert, replace, hand-over) ends a view: `E3060` at the view's later use; today `E3021` at the change, or nothing | BRW-1 x1, BRW-16 x3, BRW-17 x5, BCK-4 x1, SPN-1 x1, LT-18 x1, LT-4 x1 | A2-4 |
| `E3021`'s text names the change, the view, its line and its next use (BRW-1, BRW-12, LT-5) | 4 | A2-4 |
| two element names proved different (`if i != j:`, `for j in i + 1..`, constants) | BRW-1 x2, BRW-5 x3 | A2-4 |
| a position survives a push of its list (BRW-16), and lowers to the list and its index | BRW-1 x1, BRW-16 x2 | A2-4, A2-5 |
| `+=` through a `ref mut i64` element view; `ref mut` before a span made in the call; `MutSpan` from `ref mut Array` | BRW-5 x3, SPN-1 x1, BRW-16 x1 | A2-7 |
| `sharedref mut` | BRW-17 x2 | A2-8 |
| `static mut` with no `unsafe` (A1-17, M2's half not built) | BRW-1, BRW-8 x3, BRW-16, BRW-17 x2 | A1-17 |
| kind tests (M11), capture words (M8), `Lock` (M9), `using` and `Allocator` (M10) | CTL-11 x5, BRW-14, BRW-15, LT-4 x6, BRW-17 | later milestones |

## The order

1. **A2-4a, reshapes.** The `Loan` gets a kind (position or address) and an access gets "may reshape" apart from
   "write" (`borrows.rs` `Loan`, `check_point`). A call that may reshape its receiver or argument (`push`, `remove`,
   `insert`, `clear`, `pop`, `swap_remove`, `retain`, a whole replacement `xs = …`, a hand-over) ends an address loan
   into what it reshapes; the error is `E3060` at the loan's next use, naming the line that reshaped it and how to fix
   it (`[BRW-17]`'s text). A change that does not reshape (`xs[0].hp = 1`) while a read-only view reads the place is
   `E3021` at the change. What a std method may reshape comes from its body (`[EFF-1]`); until the summaries (M6a),
   std's collection methods carry it. Done when the reshape files of the bank pass.
2. **A2-4b, the texts** of `E3021` and `E3022` (shape B3, B1).
3. **A2-4c, index proofs** (`overlaps`: constants, `for j in i + 1..`, under `if i != j:`; SoA columns, split halves).
4. **A2-5, positions at run time**: a view of an element of a list that may grow is a position (the list and the
   index), re-read where an add may have come between.
5. **A2-6, loops**: `for ref mut x in xs`, `E3020` for a change of the walked collection, a plain loop name changes
   nothing (`E3023`).
6. **A2-7, spans and SoA**; **A2-8, `sharedref mut`** and kept views; **A2-15**, frame objects.

Each step: the bank's files of its rules move to `tests/conformance/` as they pass; the older tests that pinned
0.9.10's answers follow the spec; one full quick check before each merge to `main`.
