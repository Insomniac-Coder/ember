# Helper AA — language rules not yet done, 2026-10-22

Branch `helper-AA`, from `main` at `f00f2d61`. One commit per piece.

## Done (tests)

* **ODR-209 / D-852 `[GRM-17]`** — a lambda's `:` body on its own line is one simple statement
  that ends at the statement's newline: `h = fn(x): total += x` parses, and the newline is the
  enclosing statement's (`ember_parser`, `parse_lambda`).
  `tests/conformance/GRM-17/accept_a_same_line_lambda_body_ends_at_the_newline.em` (three
  profiles): an assignment, a `return` body, a call body, and one in a loop writing one capture
  while reading another (the defect below).

* **D-854 `[TYP-36]`** — `Array[T]` and `[T; N]` are `Ord` lexicographically when `T` is (the
  first elements that differ decide; a prefix is the smaller): `<`, `cmp`, `sort`, `sorted`,
  `min`/`max` and a `T: Ord` bound take them. `cmp` goes to std's `cmp_elements` over two views,
  as `==` goes to `eq_elements`. `TYP-36/accept_arrays_order_lexicographically.em` (three
  profiles; floats by totalOrder inside), `TYP-36/reject_an_array_whose_element_is_not_ord.em`.
* **D-860 `[MOD-5]`** — a program's own `struct int` (`float`, `str`) is what the name means in
  its types, constructions and constants (`int.ONE`); literals stay `i64`.
  `MOD-5/accept_a_programs_int_float_and_str_shadow_the_prelude.em` (three profiles).
* **ODR-211 / D-856 `[LT-1a]`, `[CTL-1]`** — `@borrows` may stand on a result some instance makes
  a view (`Option[I.Item]`) and name a parameter or a `self.f` field some instance makes a view
  (a field of a type parameter's type, an `owned self` of type `Self`); each instance applies it
  where those are views and borrows nothing from the rest, and its body is checked per instance.
  An interface method's `@borrows(self)` is resolved against its copies' parameter list (`self`
  first). std declares it: `@borrows(self.inner)` on `next`/`next_back` of `Take`, `Skip`,
  `StepBy`, `Enumerate`, `Copied`, `Cloned`, `Filter`, `TakeWhile`, `SkipWhile`, `Inspect`, `Rev`;
  `(self.inner, self.f)` on `Map`, `FilterMap`; `(self.a, self.b)` on `Zip`; `(self.first,
  self.other)` on `Chain`; `@borrows(self.view)` on `QueryIter.next` (`ecs.em`).
  * `CTL-1/accept_an_adapters_items_borrow_what_it_reads.em` (three profiles): the D-856 loop,
    items of a chain of adapters kept across turns and after the loop, a generic `keep_last`.
  * `CTL-1/reject_writing_a_map_while_an_adapters_kept_item_lives.em` (`E3021` ×2).
  * `LT-1a/reject_borrows_names_a_generic_field_but_an_instance_returns_another.em`: `E3062` in
    the `T = str` instance only.
  * `ECS-3/accept_a_query_item_is_kept_across_turns.em` (shown to fail without the attribute).
* **ODR-210 `[STD-19]`** — the consumers take their callable over (`[F: fn(...)] owned f: F`):
  `fold`, `any`, `all`, `find`, `position`, `max_by_key`, `min_by_key`, `max_by`, `min_by`,
  `reduce`. The ones giving an item (`find`, `max_by_key`, `min_by_key`, `max_by`, `min_by`)
  declare `@borrows(self)` (ODR-211), so an item does not keep the predicate's captures borrowed.
  `reduce` and `fold` do not: their result may be the callable's. `STD-19/accept_the_consumers_take_their_callable_over.em`
  (three profiles): each consumer with a lambda that counts its calls.
* **ODR-207 `[STD-26]`** — `sorted` of an iterator that lends references (`m.keys()`) copies or
  clones the items (`items_array`, as a collection's elements are), so `sorted(m.keys())` is an
  `Array[K]` whose items index the map; with `key=`/`reverse=` too.
  `STD-26/accept_sorted_of_an_iterator_of_references_copies_or_clones.em` (three profiles).
  **First week**: 05's `E2040` (N9, `sorted(self.items.keys())`) is gone; its marks re-recorded
  (codes `E1010 E2020`, the `R1:N9` step dropped) after replaying rounds 2–4 with this compiler,
  and `helped/05_inventory.em` keeps the idiom as written (`sorted(self.items.keys())`). The rate
  did not move (0 of 30, 3 of 30), so the README has no new line.
* **ODR-174 `[VER-3]`** — `@deprecated(since, note)` (both text, by position or by name; any
  other form `E0104`) on any item and on a type's or an extension's method; `W2192` "use of a
  deprecated item" at each use outside the item's module, naming `since` and `note`: a call, a
  function value, a type (plain, generic, through a module), a construction, a constant, a
  `static`, a type alias, an enum (its paths and patterns), an interface (an `implements`
  clause, a bound, `dyn`), a method or an associated function. Once per use (a generic body is
  checked per instance; a closure probe's discarded diagnostics do not count). Error page
  `docs/errors/W2192.md` (with a module beside it); `tools/error_pages.py` and
  `tools/rule_index.py` pass.
  * `VER-3/warning_a_use_of_a_deprecated_item_outside_its_module.em`,
    `VER-3/warning_a_deprecated_interface_static_alias_or_associated_function.em`,
    `VER-3/accept_a_deprecated_item_used_in_its_own_module.em`,
    `VER-3/reject_a_deprecated_attribute_without_its_version_and_note.em`; support module
    `VER-3/support/old.em`.

## Not done

## Compiler changes

* `ember_parser::parse_lambda`: the same-line `:` body is parsed as one simple statement without
  its newline.
* `ember_typeck` (assignment): on a closure's capture-discovery pass a captured assignment target
  is a placeholder, not a place; the right side is still read for the captures it names.

* `ember_typeck`: `implements` (`Ord` of `Array`/`[T; N]` by the element), `provides_cmp`,
  `synth_cmp_of` (sequences to `cmp_elements`). std: `extend[T: Ord] Span[T]: cmp_elements`.
* `ember_typeck`: `scalar_in_scope` (the scalar table unless the program declares a type of a
  shadowable prelude name), used by type positions, `T.CONST` and type names in expressions;
  `root_qualified` keys the root module's `int`, `float`, `str` as `root.<name>` (as D-305 did
  for `Box`), so `names_program_type` sees them.
* `ember_types::TypeTable::may_be_view`. `ember_typeck`: `check_borrows_attribute` (results and
  parameters some instance makes views; `borrows_in_copy` while an interface default is read for
  one type: keeps the named sources this type makes views), `receiver_view_field` (fields of a
  type parameter's type), `method_signature_inner` (an interface method's attribute against
  `self` first).
* `ember_diag`: `W2192`. `ember_typeck`: `Deprecation`, `read_deprecation`,
  `record_member_deprecations` (in `declare_names`), `note_item_use` / `warn_deprecated` at the
  resolution sites, `method_deprecation` read in `check_method_visible`; the attribute table marks
  `@deprecated` built (`@allow` stays `E0900`).

## Defects found

* **AA1 — fixed.** A block-bodied lambda that assigns a capture and reads another
  (`f = fn(x: int):` / `seen += k`) was `E1010` "cannot find `k`": on the discovery pass the
  target `seen` is a placeholder, the assignment stopped at "not a place" before reading the
  right side, so `k` was never recorded as a capture. Covered by the GRM-17 case above.

## Questions
