# The first-week corpus

`[TST-8]`: thirty programs a newcomer plausibly writes in the first week, `NN_what_it_does.em`, each
written by someone who had read only Part I and Annex A of the specification, and committed before
anything was compiled (`955d0dc7`). **They are never edited**: a change of the compiler or of the
specification changes their marks, never their text.

`[TST-9]`: `marks.json` gives each program its mark, `accepted` or `rejected`, and for a rejected one:

* `shape` — the shape (§XVII.6) of the first error `ember check` reports; `none:<code>` when it is in
  no catalogue, `gap:<code>` when the specification accepts the program and the compiler does not
  (both block a release); `P1`… for a row of `[DIA-21]`'s Python habits (ODR-179), which the
  diagnostic names (`--json`'s `"shape"`);
* `codes` — every error code the check reports;
* `helps` — the chain: `R<n>:<shape>` for each help applied in the `n`th check;
* `after_helps` — `accepted` when the catalogue's helps alone, applied literally, make it compile.

`helped/NN_*.em` is each rejected program after its chain: every copy checks clean. Each step, with
the compiler's output, is recorded in `tasks/95/S-notes.md`.

`compiler/ember_driver/tests/firstweek.rs` checks all of it: each program's outcome is its mark, each
helped copy compiles, and the rate below is the one the marks give.

## The acceptance rate (`[TST-10]`)

Acceptance rate: 0 of 30 as written, 30 of 30 after the helps.

A change of either number is a new line here, with its reason; a release that lowers one says why.

| Date | Compiler | As written | After the helps | Why it changed |
|---|---|---|---|---|
| 2026-10-22 | `b8c4df20` | 0 of 30 | 3 of 30 | the first marking |
| 2026-10-22 | `d3e170c6` | 0 of 30 | 21 of 30 | helper V: `[DIA-21]`'s Python habits P1..P30 answered, most with fix-its, the rejections in no catalogue (A1..A10) given shapes, and the helps that did not compile (B1..B6) fixed; the 9 left need a compiler gap closed (G1..G6, ODR-181: helper W, G5 the lead's) or, in 16, a move-only element of a comprehension, which no help of O2's moves |
| 2026-10-22 | merge of V | 0 of 30 | 23 of 30 | helpers V, W and X together on main: W closed G1..G4, G6 and G7 (string patterns, a temporary's iterator, a stored consumed callable, `m[k].iter_mut()`, tuples `Ord`, a one-statement lambda) and X the iterator half of G5 (`@borrows(self.f)`); the 7 left (01, 04, 05, 08, 10, 16, 23) still need a workaround after the catalogue's helps (their chains in `marks.json`) |
| 2026-10-23 | `f6a5ec2d` | 0 of 30 | 30 of 30 | helper AG: every chain replayed round by round with this compiler, each help applied literally. Five (01, 05, 08, 10, 23) compile with the catalogue's helps alone now that W's and X's gaps are closed (their chains re-recorded); 04's `E3021` (a stored callable called through a `for` loop's shared reference) gives B3's help, `.iter_mut()`, not the rebinding help; 16's `ref dyn Shape` parameter takes a loop's `ref Box[dyn Shape]` (`[TYP-14]`, whose C now compiles), and its comprehension of boxes takes `for s in owned shapes` |
| 2026-10-23 | merge of AE | 0 of 30 | 30 of 30 | helper AE: diagnostics are listed in program order (ODR-284) and the checks after type checking run on a program with errors (`[IDE-3]`): the first error of 01, 04, 08, 10 and 23 is another one now, so their shapes are re-recorded, and 05 also reports its two moves out of a reference (`E3013`, lines 34 and 39), real mistakes the borrow check used to stop short of; AG's chains are kept, every helped copy still checking clean, not replayed again |
