# Helper W — the first week's compiler gaps (D-815 … D-821), 2026-10-22

Branch `worktree-agent-a3a5ead2cac2ca5c6`, from `aae4a993`. One commit per piece. G5 (D-819, the "best
so far" loop) is the lead's and untouched; so are `regions.rs` and `borrows.rs`.

## Done, with tests

| Gap | Ledger | What the compiler does now | Tests |
|---|---|---|---|
| G1 | D-815 | `match` on text (`str` or `String`) with string literal patterns: III.6's `STRING` pattern. HIR `PatternKind::Str`; exhaustiveness has a `Str` constructor (text needs `_` or a binding, `E2090`; a repeated literal is `W2091`); a text `const` matches as its literal. MIR tests each arm with the byte comparison `==` already uses, read in place: no copy, no allocation. Text `==`/`!=` now emit `ember_str_eq` (new `static inline` in the runtime header: lengths, then `memcmp`), so against a literal the C compiler sees `s.len == 5 && memcmp(s.ptr, "north", 5) == 0`; orders still use `ember_str_cmp`. | `CTL-5/accept_a_string_literal_pattern_matches_text_by_bytes` (three profiles, `assert-c` on `ember_str_eq`, no `ember_str_cmp`), `CTL-5/reject_a_match_on_text_needs_a_wildcard`, `CTL-5/warning_a_repeated_string_pattern_can_never_match`, `DIA-14/reject_literal_patterns_on_a_failed_scrutinee_add_nothing` |
| G2 | D-816 | An `Array` that is a temporary can be viewed (`iter`, `iter_mut`, `chunks(_mut)`, `windows`, `split_at`, `get_mut`, `get`, `first`, `last`): the `E2140` "needs an Array variable" is gone. In a `for` header the existing `[EXP-4]` binding keeps the temporary to the end of the loop (dropped after it, checked with a `drop` that prints); anywhere else the borrow checker refuses a use past the statement (`E3060`). A temporary mutable view is taken by the iterator as it is (no reborrow of a place). | `EXP-4/accept_a_loop_over_an_iterator_of_a_temporary_array` (three profiles), `EXP-4/reject_an_iterator_of_a_temporary_array_past_its_statement` |
| G3 | D-817 | ODR-189: in the generic check a consumed callable parameter (`owned f: fn(A) -> R`) converts to the owned callable type of its signature; each instance converts its concrete argument by the existing rules (function or capture-free lambda by its bits, `owned fn` by `MakeCallable`, a borrowing or once-only lambda refused). A borrowed callable parameter still cannot be stored. | `CLO-3/accept_a_consumed_callable_parameter_is_stored` (three profiles), `CLO-3/reject_a_stored_callable_parameter_that_is_lent_or_called_once` |
| G4 | D-818 | A mutable view of `m[k]` goes through `index_mut`, as a `mut` receiver does (`view_of` calls `index_place_for_write`): `for v in m[k].iter_mut()`, `chunks_mut`, `get_mut`. Through a borrowed parameter it is `E3023`. | `STD-17/accept_a_mutable_view_of_a_map_entry` (three profiles), `STD-17/reject_a_mutable_view_of_a_borrowed_maps_entry` |
| G6 | D-820 | `[TYP-36]`: a tuple is `Ord` when every element is, compared lexicographically (`<`, `cmp`, `sort`, `sorted`, a `T: Ord` bound). ODR-181: `ref T` implements `Eq`, `Ord`, `Hash`, `Display` and `Debug` as `T` does (only `ref`, not `ref mut`). `cmp` is one function, `synth_cmp_of`: through a shared reference to the referent, the numbers and text by `TotalLess`, a written or derived `cmp` by its method, a tuple element by element (each side held once, the first unequal element decides). | `TYP-36/accept_tuples_order_and_references_implement_as_their_referent` (three profiles), `TYP-36/reject_a_tuple_or_reference_without_its_parts_interface` |
| G7 | D-821 | `[GRM-17]`/`[LEX-6a]`: a bracketed lambda's `:` body is one simple statement — `x op= e`, `place = e`, `x: T = e`, `pass` or an expression — ended by the closing bracket or a `,`; more is still `E0106`. std's `for_each` takes its callable over as the adapters do (`[F: fn(Item)](owned self, owned f: F)`), so `xs.iter().for_each(fn(x): total += x)` compiles (with `f: fn(Item)` borrowed, a writing closure was `E3023` inside std). | `GRM-17/accept_one_simple_statement_in_brackets` (three profiles), `FN-2a/reject_a_lambda_given_to_a_mut_callable_parameter` |

Every run-pass case above was also run in `release` by the harness (a copy with `profiles: release`
first), and the harness was shown to fail on a wrong expected line.

### The first-week marks (`tests/firstweek/`)

The rate does not move: **0 of 30 as written, 3 of 30 after the helps** (before and after), so the
README keeps its line and history. Every program that met a gap also meets a rejection in no
catalogue (`none:`), so none is accepted after the catalogue's helps alone. Re-recorded in
`marks.json`:

* 10: the first error is now `none:E2020` (`'*' * n`), codes `E1010 E2020` (was `gap:E2140` first).
* 01, 11, 23: the `gap:E2020` step (G1) is gone from `helps` (11 and 23 also from `shapes`).
* 04: `R1:none:E0100`, `R2:B3` — the B3 help (`iter_mut`) applied literally now compiles; G3 and G4
  are gone.
* 08: with tuples `Ord`, round 4 meets `p.0` moved out of the sort key's parameter (O2) together
  with round 5's O2s and G5: `R4:O2`, `R4:gap:E3021` (was `R4:gap:E2040`, `R5:O2`, `R5:gap:E3021`).
* Programs with a `gap:` step: 7 → 2 (02 and 08, both G5).

`helped/` copies keep the spec's form where a gap's workaround was applied: the string `match`es in
01, 11, 23 (with the `str`→`String` fixes their arms needed), `book.ranking().iter().enumerate()` in
10, `push(handler)` and `self.subscribers[topic].iter_mut()` in 04, and one tuple-keyed
`sort_by_key(fn(p) => (-p.1, p.0.clone()))` in 08. All 30 copies check clean. Each changed round was
replayed with this compiler (scratch copies of the original with the earlier rounds' fixes) before
the marks were written. `S-notes.md` is S's record and is not edited; its G1–G4 and G6 chain steps
are superseded by the above.

## Not done

* **`sorted(m.keys())` is still refused**, though `ref K: Ord` is built (ODR-181): `sorted(it)` gives
  `Array[T]` with `T = ref K`, an `Array` of non-static views, which `[TYP-15]` forbids (`E3063` in
  std's `to_array`). Before that is reached, indexing `m[k]` with an element of it (`ref ref K`) is
  `E2040`, "`ref K` does not implement `AsKey[K]`" (05 now reports its N9 there). Question Q-W1.
* **`[CLO-15]` for a user's registry**: a lambda that borrows, written at the call of a function that
  stores its consumed callable parameter (`bus.subscribe(fn(e) => use(total))`), is refused with
  `E3026` at the store inside the callee, where its help (`owned fn`) does not apply, instead of
  capturing by move. Question Q-W2. (thread.spawn does not capture by move either; its own test
  expects `E3026`.)
* `Array[T]` and `[T; N]` as `Ord` (the `[TYP-36]` table says lexicographic; codegen's comment "an
  `Array` is not `Ord`" predates the table) — not in G6's scope; the tuple path does not cover them.
* D1 (`Callable0` in the message for a stored borrowed callable) and the other diagnostics are
  helper V's; `CLO-3/reject_…` matches only "expected `fn(i64) -> void`".

## Compiler changes

* `ember_hir`: `PatternKind::Str`.
* `ember_typeck`: `check_literal_pattern` (strings; silent on an `<error>` scrutinee), `const_pattern`
  (text constants), `usefulness` (`Ctor::Str`); Array view methods without the place requirement,
  `reborrow_span` takes a temporary view, `view_of` writes through `IndexMut`; `coerce` (consumed
  callable parameter to its owned callable type); `implements` (`ref T`, tuples' `Ord`),
  `provides_cmp`, `synth_total_cmp` → `synth_cmp_of`; `pass_checked_lambda` (B10 for a lambda given
  to a `mut` parameter of a generic call).
* `ember_mir`: `lower_pattern_test` for `Str` (a `ValueCompare` call, then a switch).
* `ember_codegen_c`: text `==`/`!=` as `ember_str_eq`.
* `ember_parser`: the bracketed `:` lambda body.
* runtime (template + generated header): `ember_str_eq`.
* std: `Iterator.for_each`'s signature.

## Defects found

* **W1 — `owned f: fn(A) -> R` is treated as `once fn`.** The implicit callable bound sets `once:
  mode == Owned`, so an owned callable parameter called in a loop is `E3041` ("moved in a loop");
  `[CLO-6]` makes only `once fn` single-call. The std adapters avoid it with an explicit generic
  (`[F: fn(Item)] owned f: F`), and so does `for_each` now. Not fixed: G3's check reads the same flag,
  and changing it moves every owned callable parameter.
* **W2 — wrong code, fixed**: a lambda written as the argument of a generic call's `mut` callable
  parameter (`feed(fn(i) => …, 2)` with `mut f: fn(int)`) was passed by value where the callee reads a
  pointer, and the C did not compile. Now `E3027` (B10), as any other value given to a `mut`
  parameter (`[FN-2a]`); `pass_checked_lambda` serves both generic call paths. Q-W3.
* **W3** — a one-line named lambda, `note = fn(x: int): noted += x`, fails to parse ("expected the
  end of the line" on the next statement): the `:` body's `simple_stmt NEWLINE` takes the
  assignment's NEWLINE. The multi-line form, which `E0106`'s help suggests, works. Q-W4.
* **W4** — the D2 cascade "`(<error>, <error>)` does not implement `Ord`" remains (a bound check on a
  tuple holding error types; 08's round 3). `[DIA-14]`.
* Std consumers other than `for_each` (`any`, `all`, `find`, `position`, `min_by_key`, …) take a
  borrowed `fn(Item)`, so a closure that writes its captures is `E3023` inside std. Q-W5.

## Questions (options and a recommendation; not mine to rule)

* **Q-W1. `sorted` of an iterator that lends references** (`sorted(m.keys())`). (a) its items are
  cloned, giving `Array[K]`, as `sorted(m)` does (ODR-034; needs `K: Clone`); (b) `[TYP-15]` lets a
  local `Array` of views live bounded by their source; (c) refused, with a catalogued help naming
  `sorted(m)`. **Recommend (a)**: it is what ODR-181's "so `sorted(m.keys())` works" meant, and Python's
  meaning; (b) reopens `[TYP-15]`.
* **Q-W2. Which consumed callable parameters "must carry only the static region" (`[CLO-15]`) in user
  code?** (a) inferred: one its function stores as an owned callable value (ODR-189), so a lambda
  written at the call captures by move; (b) declared, by an attribute; (c) none: the caller writes
  `owned fn` (today), with the refusal moved to the call. **Recommend (a)** — `[CLO-7]` names event
  registries, and the corpus's 04 is one; (c) at least needs the diagnostic at the call.
* **Q-W3. A lambda given to a `mut` callable parameter**: (a) B10, as any value (built); (b) lent as a
  temporary, as Rust's `&mut |x| …`. **Recommend (a)** with (b) left open: `[LEX-6a]`'s own example
  (`f(fn(x): total += x, 4)`) works with a consumed parameter, which is how std now takes one.
* **Q-W4. Does a same-line `:` lambda body end at its statement's NEWLINE?** (a) yes, one NEWLINE ends
  both; (b) no, a `:` body on the same line is for brackets only. **Recommend (a)**: Python's
  `f = lambda x: …` habit, and the grammar's `block := simple_stmt NEWLINE` otherwise needs two.
* **Q-W5. Do the consumers with a callable take it over, as the adapters do?** (a) yes, all of them
  (`[F: fn(Item) -> bool] owned pred: F`): a predicate may count what it sees; (b) only `for_each`
  (built). **Recommend (a)**, as one rule with `[STD-19]`'s adapters.

## Checks run

With the `ci` build of this branch:

* `annotations.py` over every `tests/conformance/*` directory (577): `failing 0`.
* `cargo test --profile ci -p ember_driver --test ui`: 10 passed.
* `--test milestones`: 171 passed in 531 s (it includes `the_conformance_suite_runs`, so every
  profile and every `assert-c` of the new cases).
* `--test firstweek`: passed; "0 of 30 accepted as written, 3 after the helps; 24 meet a rejection
  in no catalogue or a compiler gap".
* For the lead: `tools/phase_progress.py --check` was not run; new conformance directory `CTL-5`.
