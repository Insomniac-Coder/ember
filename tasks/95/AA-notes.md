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

## Defects found

* **AA1 — fixed.** A block-bodied lambda that assigns a capture and reads another
  (`f = fn(x: int):` / `seen += k`) was `E1010` "cannot find `k`": on the discovery pass the
  target `seen` is a placeholder, the assignment stopped at "not a place" before reading the
  right side, so `k` was never recorded as a capture. Covered by the GRM-17 case above.

## Questions
