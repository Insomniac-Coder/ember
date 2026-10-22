# Helper X — `@borrows(self.f)`: an iterator's items borrow what it reads (D-819, ODR-196), 2026-10-22

Branch `worktree-agent-aedbc0be8cabc99ae`, from `main` at `216698e3`.

## Done (tests)

* **`[LT-1a]` (ODR-196) `@borrows(self.f)`** — built in the checker, the analysis and the
  interface artifact; general (any struct receiver, plain, generic, or in a generic extension; any
  mode), no special case for iterators or `std`.
  * `tests/conformance/LT-1a/accept_borrows_names_a_view_field_of_the_receiver.em` (run-pass): a
    user-written view iterator in a plain type, a generic type and a generic extension; an item kept
    while `next` is called again, two items held at once, an item outliving the iterator it came
    from; a receiver taken `owned` and one passed as a copy. Without the attribute the iterator
    part is `E3025` ×2 and `E3060` ×2.
  * `reject_borrows_names_a_field_but_returns_the_receiver.em` — `E3062`, a reference to the
    receiver's own field under `@borrows(self.owner)`.
  * `reject_borrows_names_a_mut_ref_field.em` — `E3062`, a result read through a `ref mut` field
    (the reborrow continues through the unique reference, so it borrows `self` too).
  * `reject_borrows_names_a_field_that_is_not_a_view.em` — `E2031`: a field that is not a view
    (plain and generic type), a field the type lacks, an enum receiver, no receiver.
* **`[CTL-1]`**:
  * `tests/conformance/CTL-1/accept_a_kept_map_key_borrows_the_map.em` (run-pass): the D-819
    program with `for k in m:` and `m.keys()`, a `Set`, `m.values()`, `m.items()`, a key returned
    out of a function whose iterator is a local, `split` parts, and a program's own `Iterator`
    value in a `for`.
  * `reject_writing_a_map_while_a_kept_key_lives.em`: after the loop `E3021` (the kept key's
    borrow, "`best` is what keeps it alive"); inside the loop `E3020`.
* **`std`** — `@borrows(self.owner)` on `next` of `MapKeys`, `MapValues`, `MapItems`, `SetIter`,
  `FixedArrayIter` (`std/src/collections.em`); `@borrows(self.rest)` on `Lines`, `Split`,
  `SplitWhitespace` (`std/src/string.em`; `Split`'s parts no longer borrow its separator). Not
  `MapValuesMut` (a `ref mut` owner: the body check refuses it, as it should). `Chars`,
  `CharIndices`, `Bytes` yield values, not views. The `Array`/`Span` iterators' `next` is built in
  (`SpanIterNext`, already tied through the view, D-470); `ArenaArrayIter`/`ArenaMapIter` likewise
  built in (checked: an item kept past `arena.reset()` is refused).
* **Interface artifact** — schema 12 → 13: `@borrows` is recorded per source as a parameter and an
  optional field. `compiler/ember_build/src/interface.rs`,
  `a_borrowed_field_round_trips_and_is_not_its_receiver` (round trip; `self.f` and `self` give an
  importer different keys; a parameter both whole and by a field is refused).
* **First week** — the G5 workaround (`best: String` + `clone()`) taken out of
  `tests/firstweek/helped/02_csv_summariser.em` and `08_word_frequencies.em`, which check clean
  with the idiom as written; their chains in `marks.json` lose the `gap:E3021` step (`02` R4, `08`
  R5). No original's mark changed and the rate did not move (0 of 30, 3 of 30: both programs meet
  other uncatalogued rejections), so the README has no new line.

* **Runs** (`target\ci\ember.exe` of the last commit): the whole conformance tree with
  `tests/run-pass`, `compile-pass`, `compile-fail`, `run-fail` (`annotations.py`): failing 0;
  `--test ui` 10/10, `--test milestones` 171/171, `--test firstweek` 1/1 (0 of 30, 3 of 30),
  `--test reload` 49/49; `cargo test -p ember_analysis -p ember_build -p ember_typeck` all pass.

## Not done

* Iterator adapters (`Filter`, `Map`, … in `std/src/core.em`) and `QueryIter` (`std/src/ecs.em`)
  still tie their items to the adapter: `for k in m.keys().filter(f): best = k` is `E3025` and
  `E3060` naming `__it`. Their field is `inner: I` and their result `Option[I.Item]`, neither a view
  in the declaration, so `@borrows(self.inner)` is `E2031` twice. A ruling is needed (question 1).

## Compiler changes

* `ember_hir::BorrowSource { parameter, field: Option<usize> }` (re-exported by `ember_mir`):
  `@borrows` is `Option<Vec<BorrowSource>>` from the checker through HIR, MIR and the interface,
  canonical (sorted; a parameter named whole subsumes its fields).
* `ember_typeck`: `check_borrows_attribute` takes `self.f`; `receiver_view_field` (`E2031` when
  there is no receiver, the receiver is not a struct, the field is missing or not a view; the help
  lists the view fields); `receiver_fields` reads a struct's fields, or, while a generic type's or
  generic extension's methods are collected with `Self` as the receiver, `recipe_fields` (saved and
  restored around on-demand collection).
* `ember_analysis::regions`: `Elision::Named { arguments, through: Vec<(usize, Vec<Projection>)> }`
  — a tied argument's views by path, behind it when it is a reference; D-470's whole view is the
  empty path, `self.f` is `[Field(f)]`. `tied_fact` reads them.
* `ember_analysis::borrows`: `elision_of` builds the paths; `named_parameters`,
  `narrowed_parameters`; `return_region_offenders` refuses (`E3062`) a slot of a field-narrowed
  parameter outside its fields that reaches the result (`reaches_slot`), with its own message, at
  the `return` whose value does it (`Regions::result_at`).
  `loan_held_by_for_iterator` now asks whether the loop's iterator still holds the loan at the
  conflicting point, so a write after the loop that a kept item conflicts with is `E3021`
  (`[BRW-1]`), not `E3020` ("inside the loop", `[CTL-2]`).

## Defects found

* `E3062` pointed at the first `return` of the function, not at the one whose value borrows the
  parameter: a `next` returning `Some(ref mut self.owner[i])` after an early `return None` was
  reported at `None`. Fixed (`Regions::result_at`, `check_return_regions` picks the `return` whose
  value carries the offending parameter, the first one otherwise);
  `reject_borrows_names_a_mut_ref_field.em` has that shape.
* `E3020` ("cannot mutate `m` while it is borrowed by this loop") was given after the loop when an
  item kept from it (`best = k`) was what kept the loan: `[CTL-2]` makes `E3020` the error inside
  the loop. Fixed (above).

## Questions

1. **Adapters over a borrowing iterator.** `for k in m.keys().filter(f): best = k` is refused: the
   adapter's `next` ties its item to the adapter (`[LT-1]` rule 1), and neither its field (`inner:
   I`) nor its result (`Option[I.Item]`) is a view in the declaration, so `@borrows(self.inner)` is
   `E2031`.
   * (a) Leave it: items of an adapter borrow the adapter; the program keeps `clone()`.
   * (b) `@borrows(self.f)` may name a field whose declared type is a type parameter, and `@borrows`
     may stand on a result whose declared type names one; it applies in each instance where the
     field and the result are views (as ODR-048 makes a parameter a source per instance), and is
     checked there.
   * (c) An elision rule: a method of a generic type whose result's type is an associated type of
     a field's type parameter borrows that field.

   Recommendation: (b) — the same rule as ODR-196, read per instance as `[LT-1]` already is for
   sources (ODR-048); the adapters and `QueryIter` declare it, and nothing new is implicit.
2. **The receiver's mode.** The ruling names "a view field of the receiver" with no mode; I read it
   as every mode: `owned self` and a borrowed receiver passed as a copy (`[BRW-8]`) work as `mut
   self` does, since what the field borrows does not depend on how the receiver is passed (tested).
   The alternative, `E2031` unless the receiver is borrowed or `mut`, refuses sound programs.
   Recommendation: keep it, and say "in any mode" in `[LT-1a]`'s sentence.
3. **The spec text.** `[LT-1a]` in `tasks/spec-0.9.9/parts/p07-ownership.md` does not yet say
   `self.f` (not mine to edit). Proposed sentence, after "(`self` names the receiver)": "`self.f`
   names a view field `f` of the receiver's type, a field whose type is a reference or view: the
   result borrows what `f` borrows, and the call's loan on the receiver ends with the call; a
   result read through a `ref mut` field borrows `self` as well. `E2031` when there is no receiver
   or `f` is not such a field."
