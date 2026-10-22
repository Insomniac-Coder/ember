# Helper P — Annex B (hot reload), 2026-10-22

Time box lifted by the coordinator; rules worked in turn and committed one by one.

## Done

* **HR-23** — built and tested. `compiler/ember_driver/tests/reload.rs`,
  `a_changed_cpp_header_refuses_the_reload_and_an_overlay_does_not`: a reload built after
  `engine.hpp` changed is refused in plan, naming the header, and the old image keeps running; one
  built after only the overlay changed applies the edited body beside it.
  `a_trampoline_reaches_an_edited_override_after_a_reload` still passes (thunks reused where the
  header is unchanged).
* **HR-14** — every row of the table built; `unfinished` entry removed. Tests in
  `compiler/ember_driver/tests/reload.rs`:
  `a_field_added_whose_type_is_default_takes_its_default` (written and derived `Default`),
  `a_field_changed_to_a_range_type_is_tested_by_checked` (passes, fails with a count, back to the
  representation, a range narrowed, a stored static, a folded static),
  `an_enum_payload_migrates_by_the_field_rows` (widened, `Default` field added, variant added
  before, `@renamed_from` on a payload field and without it, a non-`Default` field refused, a held
  variant removed refused with its count, a payload field removed dropped in reclaim, a struct in
  `Option[Point]`'s payload grown), `a_changed_layout_attribute_is_a_schema_change` (now also a
  type crossing a C-ABI signature by value: attribute change refused; a field added refused through
  the signature); plus the existing table tests. Conformance: `tests/conformance/HR-14/`
  (`accept_payload_field_renamed_from.em`, `reject_attribute_on_payload_field.em`).

* **HR-13** — built and tested. `compiler/ember_driver/tests/reload.rs`,
  `value_data_migrates_through_its_container`: structs by value in an `Array` a static holds; in an
  `Array` class instances hold; in a fixed array (in place); in an `Array` of `Array`s; unit enums
  in an `Array` (reordered: kept by name; a held variant removed: refused, "an element of a
  container"); payload enums in an `Array`; `SoA` columns (a scalar field added; a column of a
  package struct grown, migrated depth-first); ECS storage (`AosStorage` and `SoaStorage` of
  components, reached through a `World`); arena contents (a collection of an arena is a view no
  class object or static may hold, `E3063`/`E7002`, so a poll never meets one; held by a frame
  below a checkpoint, the change is refused by `[HR-3b]`). `[HR-13a]` extended: a
  standard-library class holding the package's types or references now has a schema the package
  emits.

* **HR-22** — holds as built; tested. `compiler/ember_driver/tests/reload.rs`,
  `a_foreign_object_ember_owns_is_carried_across`: a class instance holding a `ForeignBox` of a C
  widget (`import c "./widgets.c"` with an overlay's `destroys`/`returns_owned`) migrates at a
  checkpoint with the box carried unchanged (the widget read afterwards, freed once at the
  program's end by the destructor it was made with); an edit removing the field drops the box in
  reclaim, freeing the widget once.

* **HR-20** — partly built; `unfinished` entry added. Built: an object a `Retained` token pins is
  never moved: plan refuses a reload that would move one (its new size does not fit, or
  `migrate_from` builds a new one), naming the class and how many are pinned, even where every
  reference could be found and rewritten; a change that fits migrates it in place.
  `compiler/ember_driver/tests/reload.rs`, `a_pinned_object_is_never_moved`. Before this, a
  pinned object whose references were all enumerable was moved, leaving foreign code's address
  stale (a defect, fixed). Not built: `on_relocate` (spec question 4).

## Compiler changes

* HR-23: `cimport.rs` (`import_cached` returns a hash over every file the header included),
  `ember_typeck` `ForeignImport.header_hash`, `reload.rs` (driver) collects C++ headers into
  `ReloadImage.headers`, codegen emits a `headers` table in the manifest, the runtime's
  `ember_rl_manifest` gains `header_count`/`headers`, `g_rl.running` (the manifest applied last),
  and `rl_plan` refuses a changed header.
* HR-14, `Default` row: the driver writes, for each field (and payload field) declared without a
  default whose written type is not a number, `bool`, `String` or `Array`, a helper returning
  `std.core.default_of[T]()`; `ember_typeck::check` takes the helpers' spans and drops (does not
  check or build) one whose return type is not `Default`; codegen sets `EMBER_RL_FIELD_DEFAULT`
  only where the helper was built.
* HR-14, range row: codegen emits each range type's test (`T.checked`'s decision over the
  representation, 128-bit and float included) and names it in the field row (`in_range`, a new
  trailing member of `ember_rl_field`); a range type's schema hash now includes its range. The
  runtime has a `RL_NUMBER` route (into or out of a range type: same number or a lossless
  widening), tests every live value in plan (instances, nested structs, payloads, statics) and
  refuses naming the field, the type and the count.
* HR-14, payload row: the runtime's migration works over field lists (`rl_fields`), a type's or a
  variant's payload: plan (`rl_plan_payload`), the removed-variant and range value checks, the
  migration (`rl_migrate_payload`: variant by name, tag rewritten, payload by the rows) and reclaim
  (`rl_drop_payload`). Codegen numbers payload fields across variants for an enum's `init_field`
  and `drop_field`, and gives payload rows their flags. A schema'd enum no longer makes its payload
  structs "contained". The parser now reads attributes on a payload field (the grammar's
  `variant_field := {attribute} ...`; `VariantField.attrs`), `@renamed_from` applies to a payload
  field ("payload field" target), `ember fmt` prints them.
* HR-14, FFI row: codegen marks the types that cross a C-ABI signature by value or through a
  reference (`EMBER_RL_LAYOUT_ABI`) and records a hash of a type's layout attributes alone
  (`ember_rl_type.attributes`); plan refuses a changed attribute on such a type whether or not a
  value lives.
* HR-13: an `Array`'s and a fixed array's schema hash includes its element's (a container of a
  changed type holds values to migrate). Codegen describes a container's element with a row of its
  own (`inner`, by `field_shape`, with the buffer's alignment `align`); new field kinds `FIXED`
  (elements in place) and `SOA` (`layout` describes the `SoA`'s own struct: each column pointer
  with `within`, where its value sits in a whole element, then `len` and `cap`; the type's new
  `columns` member is the image's `<SoA>_layout`). The runtime has `RL_ELEMENTS` (each element by
  its row: plan, removed-variant and range checks, migration into a buffer the transaction arena
  holds, reclaim of the old buffer) and `RL_COLUMNS` (each `SoA` element gathered into a whole old
  element, migrated by the element type's schema, scattered into a block the new image lays out;
  the old block freed by the old image's layout); the reference walk follows `FIXED` and `SOA`
  values. "Contained" (refused) structs are now only those held where a reload cannot reach (in a
  tuple, a closure's captures, a type with no schema), found by a walk down from such holders; a
  raw pointer type that only an `SoA`'s own struct holds no longer makes its pointee "foreign".
  Classes with a schema: the package's, and standard-library classes holding the package's types,
  an `SoA` of them, or references. A struct or enum with no schema that holds references is now
  described to the runtime (followed) instead of opaque.
* Runtime: the runtime C is generated from `runtime/ember_rt/templates/*.in`
  (`tools/generate_runtime.py`); every runtime change is in the templates, regenerated.

## Defects found (fixed)

1. `[HR-14]` widening into a 128-bit integer wrote only the low 8 bytes (a negative `i64` widened
   to `i128` became a large positive number), and `rl_widens` accepted `f16`/128-bit sources it
   could not read. Fixed in `rl_convert_number`/`rl_widens`.
2. `[HR-17a]` a static the compiler folds into the code (its initialiser evaluated while
   compiling) whose initialiser text is unchanged but whose value changed (a function it calls was
   edited) silently took the new value. The folded value is now part of the static's initialiser
   hash (`TypeTable::folded_statics`), so the change is refused unless `@reinit_on_reload`; and a
   folded static whose type changed with its value kept applies (it was refused as
   "type changed from `?` to `?`"). Test: `a_static_initialiser_edit_needs_reinit_on_reload`
   (folded case), `a_field_changed_to_a_range_type_is_tested_by_checked` (folded static).
3. `[HR-21]` a C-ABI function's signature hash was its C type names only, so a field added to a
   struct it takes by value was not a signature change. It now includes the crossing types'
   schemas (a class as its handle). Test: `a_changed_layout_attribute_is_a_schema_change`.
4. `[HR-13]` any tuple or `Array` type anywhere in the program holding a struct (a temporary
   `(self, default)` in `Option.unwrap_or`) made the struct "contained" and its every change
   refused. Only a holder whose values can outlive a call counts now (held by another type, a
   closure's captured state, a static).
5. `[HR-11a]`/`[HR-13]` an `Array` of unit enums whose variants were reordered kept its old
   discriminants (the `Array`'s schema hash was its name only, so nothing migrated): wrong
   variants after the reload. Fixed by element schemas; test `value_data_migrates_through_its_container`.
6. `[HR-13]` a component in an `SoA` (`SoaStorage`, `SoA[T]`) whose fields changed was neither
   migrated nor refused (the `SoA`'s struct holds column pointers, which mention no package type):
   old columns under the new layout. Fixed by `RL_COLUMNS`.
7. Codegen: `reload_layout_table` named its table before describing its members, so a member's
   own table could take the same name (`em_rl_layout_0` defined twice) once more types were
   described. Named after its members now.

## Defects suspected (not verified)

* `[FFI-33b]`/`[THR-1]`: `static S: Mutex[Array[ForeignBox[b.Widget]]] = Mutex([])` is `E7002`
  ("not Sync"), though `std.ffi` declares `unsafe extend[T] ForeignBox[T] implements Send` and
  `[FFI-33b]` says a box is `Send`. Either the marker is not applied to an imported opaque type's
  box or the rule intends a `threads` contract first; not mine to rule.
* Diagnostics: a field whose type does not resolve is reported twice in a reloadable build, the
  second time at `<reload helpers of the root module>` (the helper written for its default). The
  same holds for a declared default that does not check. Dropping the helpers' diagnostics when
  the user's own code has errors would remove the noise.

* A struct value inside a `Box[dyn I]` (an interface value) is not described to the runtime and
  not flagged contained; if its layout changes while such a value lives, the new image's methods
  would read the old layout. Worth a test.

## Spec questions

1. **HR-23: what counts as a changed header.** Built: any change to the header or a file it
   includes (content hash), so a comment-only edit also refuses. Options: (a) keep this, the
   literal "a changed C++ header"; (b) refuse only when the binding changes (declarations,
   layouts, the shim), which `[HR-41]`'s "C++ header ABI changed" suggests, but an inline body the
   host compiled would then differ silently from the image's. Recommendation: (a).
2. **HR-23 for C headers.** The rule names C++ only, so a changed `import c` header is not
   refused. The same host-rebuild reason applies to C. Options: (a) C++ only, as written; (b)
   extend to `import c`. Recommendation: (b), by a spec change.
3. **HR-14 FFI row: field changes.** The row refuses changed layout attributes on a type used
   across FFI; a field added to such a type (by value in a C-ABI signature) is refused too, through
   `[HR-21]`'s signature. Recommendation: say so in the row ("layout or fields changed on a type
   used across FFI by value").

4. **HR-20 `on_relocate`.** The target says a token "created with an `on_relocate` callback,
   which the runtime calls with the new address during commit" lets a pinned object move, but
   gives no spelling, and `[HR-2]` says commit performs "no allocation, check, callback or other
   fallible operation". 0.8.5 (docs/spec/part-18) had `Retained.pin(handle, on_relocate:
   Option[extern "C" fn(*void, *void)])`, called with the old and the new address. Options:
   (a) restore 0.8.5's spelling and call the callbacks right after commit publishes, before
   reclaim (commit stays pointer stores; the reload has succeeded, so a callback cannot undo it);
   (b) call them inside commit and add an exception to `[HR-2]` for a foreign `void` callback;
   (c) drop `on_relocate`: a pinned object never moves. Recommendation: (a), the foreign side
   told the new address once the new image is live, with `ObjectPtr[C]` for the class case
   (`[FFI-23]`, ODR-132) instead of `*void`.

## Not done yet

HR-1, HR-9, HR-20, HR-22, HR-24, HR-39, HR-41, HR-43 — in progress.

One transient failure seen once and not again (four reruns): a reload build in
`value_data_migrates_through_its_container` exited -1 with no output.
