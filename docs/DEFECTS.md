# Ember — defect ledger

Every defect found and what closed it. One row per defect, newest first.

**Why this exists.** Defects were recorded in prose, spread across
`docs/HANDOFF.md`'s per-block sections, where "found" and "fixed" read the
same. A reader could not tell which of them are closed. This file answers that
one question, and links to where the reasoning lives.

**How to use it.** When a defect is fixed: add a row, state how the fix was
*verified* (a program that failed before and passes now, or a test that goes red
when the fix is removed), and name the commit. If the defect was in the
**specification** rather than in the compiler, it belongs in
`docs/spec-errata.md` instead and the row here points at the errata id — the
standing rule is that an implementation gap is not a spec defect.

Status is one of **fixed**, **open**, or **won't fix** with the reason.

---

## 2026-09-09 — the region work

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-001 | A borrow copied into another local was not tracked: `s = r` killed the loan with `r`, so writing the owner afterwards compiled | `[BRW-1]`, `[LT-5]` | **fixed** | `1204a5e` |
| D-002 | A reborrow (`q = ref mut r`) did not keep the borrow it derived from alive | `[BRW-6]`, `[LT-5]` | **fixed** | `1204a5e` |
| D-003 | A call handing a reference back did not keep its argument borrowed | `[LT-1]` | **fixed** | `1204a5e` |
| D-004 | `[DIA-3]`'s "later used here" label was missing from every borrow diagnostic, and the help named the local the borrow *started* in | `[DIA-3]` | **fixed** | `1204a5e` |
| D-005 | `@borrows` was checked as a signature only: `@borrows(a)` on a function returning `b` compiled | `[LT-1a]` | **fixed** | `c1bd89c` |
| D-006 | `E3060` exempted every parameter, so a borrow of a **by-value** parameter could be returned | `[LT-1]`, XVIII §4.7 step 6 | **fixed** | `c1bd89c` |
| D-007 | `[TYP-14]`'s read-through worked only for a named local, so a call returning `ref i32` was not an operand of `+` (`E2020`) and reached `println` as a pointer | `[TYP-14]` | **fixed** | `c1bd89c` |
| D-008 | `ember fmt` deleted a doc comment on a method | `[FMT-1]` | **fixed** | `28aa05d` |
| D-009 | `ember fmt` deleted a doc comment on an enum variant | `[FMT-1]` | **fixed** | `28aa05d` |
| D-010 | The AST printer showed a variant as its bare name, so `[FMT-1]`'s round-trip test could not see D-009 | `[FMT-1]`, `[TST-1]` | **fixed** | `28aa05d` |
| D-011 | `E3064` (two independent regions in one view struct) is registered and emitted by nothing | `[LT-2]` | **open — no case found** | — |

### How each was verified

**D-001, D-002, D-003.** Three programs that write through a dangling
reference. Each compiled in silence before and is `E3021` now:

```ember
r: ref mut i32 = ref mut n
s: ref mut i32 = r      ## the loan is held by `s` now
n = 5                   ## …and this was accepted
s = 7
```

`tests/compile-fail/borrows_travel_with_the_reference.em` holds all three, and
`tests/run-pass/references_that_travel.em` holds the same shapes written
correctly, so the fix is not over-rejection.

**D-004.** Visible in the rendered diagnostic: the conflict now carries a
`borrow later used here` label at the next read, and the help names the local
that actually keeps the loan alive — `s` in the program above, where it used to
say `r`.

**D-005.** `@borrows(a) fn pick(a: ref i32, b: ref i32) -> ref i32: return b`
is `E3062`. `[LT-1]` rule 1's case — a method whose result points into an
argument rather than into its receiver — is rejected with the `@borrows` that
would fix it in the help line.
`tests/compile-fail/a_returned_view_elision_cannot_tie.em`.

**D-006.** `fn peek(self) -> ref i32: return ref self.n` is `E3060`. The
exemption is now for view-typed parameters only: what the caller owns outlives
the call, a copy in this frame does not.

**D-007.** `println(give(ref n))` emitted C passing a `const int32_t *` where
`ember_str` was expected — clang caught it, Ember did not — and
`give(ref n) + 1` was `E2020`. Both work now;
`tests/run-pass/returned_views.em` runs them.

**D-008, D-009, D-010.** `ember fmt` on a file with a documented method and a
documented variant keeps both. Verified the way the handoff asks: the fix was
removed and `formatting_is_idempotent_and_preserves_the_tree` went red, then
restored. For D-009 that test went red **only after D-010 was fixed** — the
instrument was blind to the defect it exists to catch.

**D-011 is open and deliberately unwritten.** `[LT-2]` gives a struct built
from several references the *intersection* of their regions, so the compiler
narrows rather than refusing, and no program has been found that reaches the
error. Writing the diagnostic before there is a program that needs it would
enforce the rule in a shape nobody asked for.

---

## Before 2026-09-09

Reconstructed from `docs/HANDOFF.md`, which is where the reasoning for each of
these lives. All are fixed; the ledger did not exist when they were.

| # | Defect | Rule | Status |
|---|---|---|---|
| D-101 | A `##` comment inside a function body deleted the statement under it — the parser returned "no statement" and the caller treated that as a parse failure | `ERR-007` | **fixed** |
| D-102 | `let` was dropped by the formatter, turning an immutable field into a mutable one | `[CLS-9]`, `[FMT-1]` | **fixed** |
| D-103 | Doc comments were dropped by the formatter at the item level | `[FMT-1]` | **fixed** |
| D-104 | The formatter dropped every type parameter list: `struct Pair[A, B]` printed as `struct Pair` | `[FMT-1]` | **fixed** |
| D-105 | The formatter flattened `unsafe:` blocks — the catch-all preserved text and destroyed structure | `[FMT-1]`, `[UNS-1]` | **fixed** |
| D-106 | Methods on a generic struct were silently dropped, so an instantiation had no methods | `[TYP-16]` | **fixed** |
| D-107 | `alloc`'s `arg_ty` was taken from its first argument, a count, so every allocation was `usize`-sized | `[UNS-4]` | **fixed** |
| D-108 | `StmtKind::CheckedBinaryOp` was invisible to the move analysis, so `bigger = cap * 2` read as a use after move | XVIII §4.6 | **fixed** |
| D-109 | A panicking program reported exit code 0: `abort()` arrives as `0xC0000409` and `clamp(0, 255)` made it a clean exit | `[TST-1]` | **fixed** |
| D-110 | `println(10.0)` printed `1e+01` — the shortest *precision* that round-trips, rather than the shortest text | `[FMT-2]` | **fixed** |
| D-111 | `n = match d:` with indented arms did not parse | `[GRM-11]` | **fixed** |
| D-112 | `t.0.1` did not parse: the lexer read `0.1` as one float literal | `[GRM-8]` | **fixed** |
| D-113 | An unused pattern binding tripped `-Wunused-but-set-variable`, breaking the warning-free C requirement | `[CG-C-1]` | **fixed** |
| D-114 | `interface From[T]` could not be declared because `from` was reserved | `ERR-017` | **fixed** |
