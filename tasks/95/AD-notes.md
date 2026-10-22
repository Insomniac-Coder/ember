# Helper AD — the language and std rules the owner has answered, 2026-10-23

Branch `worktree-agent-adbe030e8d426272c`, from `main` at `3922e110`. One commit per piece.
Baseline before any change: the whole conformance tree (every `tests/conformance/*`,
`tests/run-pass`, `compile-pass`, `compile-fail`, `run-fail`) failing 0.

## Done (tests)

* **ODR-254 `[STD-14]`, `[ERR-13]`** — the accessors' twins: `Mat2`/`Mat3`/`Mat4` `try_col(i)` and
  `try_row(i)` (`None` for a column or row the matrix does not have), `CppShared.try_get()`
  (`None` for an empty `std::shared_ptr`); `col`, `row` and `get` now name them in their panics
  (`; use .try_col(i) for an Option`), as plain text, so they stay `@noalloc`. The names follow
  std's twin habit for a method (`try_inverse`, `try_insert`). The other caller's bugs ODR-254
  lists (`stepped(0)`, `BitSet.insert`, `Entity.new`, `World.add`, `normalize`, `split`) are
  settled in `milestones.rs`'s list (`std_panics_name_their_twins_and_allocations_are_documented`),
  whose open list is now empty. `phase-status.json`: `STD-14` removed.
  * `STD-21/accept_a_matrix_column_or_row_has_a_non_panicking_twin.em` (three profiles),
    `STD-21/run_fail_a_matrix_column_past_the_matrix_names_its_twin.em`,
    `FFI-17a/accept_an_empty_shared_pointer_has_a_non_panicking_get.em` (`gauges.hpp`).
