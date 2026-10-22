# The first-week corpus

`[TST-8]`: thirty programs a newcomer plausibly writes in the first week, `NN_what_it_does.em`, each
written by someone who had read only Part I and Annex A of the specification, and committed before
anything was compiled (`955d0dc7`). **They are never edited**: a change of the compiler or of the
specification changes their marks, never their text.

`[TST-9]`: `marks.json` gives each program its mark, `accepted` or `rejected`, and for a rejected one:

* `shape` — the shape (§XVII.6) of the first error `ember check` reports; `none:<code>` when it is in
  no catalogue, `gap:<code>` when the specification accepts the program and the compiler does not
  (both block a release); `DIA-21` for a row of `[DIA-21]`'s Python-habit table;
* `codes` — every error code the check reports;
* `helps` — the chain: `R<n>:<shape>` for each help applied in the `n`th check;
* `after_helps` — `accepted` when the catalogue's helps alone, applied literally, make it compile.

`helped/NN_*.em` is each rejected program after its chain: every copy checks clean. Each step, with
the compiler's output, is recorded in `tasks/95/S-notes.md`.

`compiler/ember_driver/tests/firstweek.rs` checks all of it: each program's outcome is its mark, each
helped copy compiles, and the rate below is the one the marks give.

## The acceptance rate (`[TST-10]`)

Acceptance rate: 0 of 30 as written, 3 of 30 after the helps.

A change of either number is a new line here, with its reason; a release that lowers one says why.

| Date | Compiler | As written | After the helps | Why it changed |
|---|---|---|---|---|
| 2026-10-22 | `b8c4df20` | 0 of 30 | 3 of 30 | the first marking |
