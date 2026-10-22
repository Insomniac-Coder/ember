# Helper Y — notes (2026-10-22)

Rules: `[MNG-1]`, `[MNG-3]` (ODR-202), `[MOD-1]` (ODR-201), `[FFI-42]` (D-782), `[HR-7]`, `[DIA-23]`.
Branch `worktree-agent-a9529437952259b8a`, from main `216698e3`.

## 1. ODR-202 — a transliterated component is marked in its length (done)

- `ember_branding::push_component`, the one function that writes a component, puts
  `TRANSLITERATED_MARK` (`u`) before the length of a component that had a character beyond ASCII:
  `café` is `em_u10caf_u00E9_`, the ASCII name `caf_u00E9_` is `em_10caf_u00E9_`.
- Every reader goes through two new functions, `starts_component` (a nonzero digit, or `u` and one)
  and `split_component`: `item_parts` (so `reinstance`), `demangle` (so `ember demangle`, L5002's
  check) and `symbol_path_starts_with`. `demangle` untransliterates only a marked component, and only
  when it is exactly what the compiler writes (a character beyond ASCII came back, each escape in
  canonical upper-case hex); an unmarked one is taken as written. `generated` asserts that a tag is not
  the start of a component (no tag is `u` and a digit).
- EMIF schema 12 → 13 (`compiler/ember_build/src/interface.rs`).
- Needles: none named a transliterated symbol except `MNG-3/accept_a_name_beyond_ascii_is_transliterated.em`'s
  substrings, which still hold.

Tests:
- `ember_branding` `a_name_beyond_ascii_is_transliterated_before_its_length` (the pair, a path of
  two marked components, demangle both ways, three near misses that are no item's, an instance after
  a marked component, `reinstance`, `symbol_path_starts_with`) and `demangling_round_trips`
  (`caf_u00E9_.café`, `u1.u2x` added).
- `tests/conformance/MNG-3/accept_a_transliterated_name_and_its_ascii_spelling_are_two_symbols.em`:
  `café`/`caf_u00E9_` functions and `Maß`/`Ma_u00DF_` structs in one program build and run, and the C
  names each apart. Before: both functions were `em_10caf_u00E9_`, one C function defined twice.

## 2. ODR-201 — a module and an item of one name in one scope is `E1032` (done)

- New code `E1032` (`[MOD-1]`, "an item has the path of a module or the name of a built-in type") in
  `compiler/ember_diag/src/codes.rs`; page `docs/errors/E1032.md` (its `fails` example uses an
  ```` ```ember,file=grid.em ```` module; `tools/error_pages.py` passes it).
- `ember_typeck` `check_item_paths`, after every module's names are declared, three forms:
  - an item whose path is a module's of the build (the root module's `grid` beside module `grid`,
    module `inner`'s `cells` beside module `inner.cells`; a dependency's root module counts, its
    path being the package name), naming the module's file as the secondary place;
  - an item named like a module its module imports (`import paths.grid` beside `fn grid()`: `grid.at`
    meant the module silently), naming the import; prelude-bound `mem` is not an import, as
    `[MOD-5]` lets a declaration shadow it;
  - a type (struct, class, enum, interface, alias) named like a built-in type (`[TYP-1]`'s scalar
    table less the prelude's aliases `int`, `float`, `Never`, and `str`, which `[MOD-5]` lets a
    declaration shadow; `void` is a keyword). Checked first: the compiler accepted `struct i32`.
    A declaration `import c` wrote from a header keeps its name (`typedef float f32;` is common C).
- Only modules of the build are compared (an unimported file is not part of the program; a
  single-file package's directory holds unrelated files, as the conformance directories do).
- `E9040` of one path and one list of arguments under a path `E1032` reported is its cascade
  (`[DIA-14]`), not reported (`path_conflicts`, `under_path_conflict`); its `one_path` help no longer
  says "a module and a type", which `E1032` now catches.

Tests:
- `tests/conformance/MOD-1/accept_a_module_and_an_item_of_one_name_in_two_scopes.em` (+ `paths/`):
  `grid`, `paths.grid`, `paths.shapes.grid` in one program, three C names.
- `MOD-1/reject_an_item_and_a_module_of_one_path.em` (+ `inventory.em`, `inner/mod.em`,
  `inner/cells.em`): both path forms.
- `MOD-1/reject_an_item_named_like_an_imported_module.em`.
- `MOD-1/reject_a_type_named_like_a_built_in_type.em`: struct, enum, class, interface, alias.
- `MNG-1/reject_two_instances_of_one_symbol_are_both_named.em` became
  `MNG-1/reject_two_items_of_one_path_are_refused_before_their_instances.em`: the same program is
  `E1032` and no `E9040` (the hash's own collision stays `ember_typeck`'s unit test).
- `tests/ui/basic/E1032/item_named_like_a_module.{em,stderr,fixed.em}` (+ `grid.em`) and
  `ui.rs` `committed_item_named_like_a_module_matches_and_fixed_source_compiles` (`[DIA-13]`).

**Needs the lead:** `ember_diag` `the_registry_and_the_specification_name_the_same_codes` fails
until the development target's registry (§XVII.9, `p17a-codes.md`) has the row
`` | `E1032` | an item has the path of a module or the name of a built-in type | `[MOD-1]` | ``, and
ODR-201's text is in `[MOD-1]` (or the rule the lead picks; the code cites `[MOD-1]`).

## 3. D-782's workaround removed (done)

- `cimport.rs` `cpp_iteration`: the iterator is `<Class>_iter` (named as a member's static is,
  `Outer_name`), its symbol (`em_3cpp12h_containers8Bag_iter`) apart from the method `Bag.iter`'s
  (`em_3cpp12h_containers3Bag4iter`). Its name is claimed like every other name of the module, and
  the iterators are written after the header's own declarations and instantiations
  (`Writer::iterations`), so a header's own `Row_iter` keeps its name and `Row`'s iteration is the one
  skipped (`W5002`, "its iterator's name `Row_iter` is taken by another declaration of the header").
  Before, a header class `Bag_iter` after `Bag` would have lost its name to the generated iterator.
- C++ corpus records: `record.py` on Windows writes them unchanged (the nlohmann entry has no
  `begin()`/`end()` class); the Linux records come from the same code and have no iterator either.

Tests:
- `FFI-42/accept_a_cpp_container_with_begin_and_end_is_iterable.em`: needles for `Bag_iter`'s struct
  and `Bag.iter`'s function.
- `FFI-42/accept_a_headers_own_name_outranks_an_iterators.em` (+ `named.hpp`); break-tested: written
  in place (as before), the header's `Row_iter` is skipped and the case fails.
- `MNG-1/accept_a_struct_a_b_and_a_method_a_b_are_two_symbols.em`: D-N4-4's own probe
  (`struct Bag_iter`, `extend Bag: fn iter`), `em_8Bag_iter` and `em_3Bag4iter`.

## 4. The runtime names reload slots by their Ember paths (done)

- `runtime/ember_rt/templates/ember_rt.c.in`: `rl_path` (with `rl_path_put`, `rl_hex_digit`,
  `rl_path_untransliterated`) decodes a slot's symbol exactly as `ember_branding::demangle` does: the
  mangled prefix derived from `EMBER_SYMBOL_PREFIX` (its first two characters and `_`), components by
  their lengths, a `u`-marked one untransliterated to UTF-8, an instance as `path[#hash]` (as `ember
  demangle` writes it). An export's slot keeps its exact symbol (`[MNG-2]`); anything not exactly an
  item's, or longer than the buffer (256 bytes; a path is never longer than its symbol), is shown as
  it is. Called only where a report is written (the `[HR-7]`, `[HR-10]`, `[HR-19]`, `[HR-21]`
  refusals) and in `ember_reload_removed_called`'s panic (`[DIA-23]`: a panic names no mangled name).
  Regenerated with `tools/generate_runtime.py`.

Tests: `reload.rs` `a_noreload_body_change_is_refused_whole`, the explain test and
`removing_a_function_whose_address_is_held_is_refused` read `fixed`/`helper`; the last gained a
generic `größe[T]` removed at a checkpoint: "`größe[#<hash of [i64]>]` was removed while frames…". The
C host test (`the_reload_protocol_from_a_c_host`, slot `gone`) covers the as-is fallback.

## Test runs (this branch, MSVC)

- `annotations.py` over every `tests/conformance/*` directory and `tests/run-pass`,
  `compile-pass`, `compile-fail`, `run-fail`: failing 0.
- Every `assert-c`/`-count`/`-order` needle of `tests/conformance`, `run-pass`, `compile-pass`,
  `run-fail` against the emitted C in each listed profile: 3172 files, 0 failing.
- `ui` 11/11, `reload` 49/49, `package` 14/14, `cdylib` 10/10, `staticlib` 4/4, `ember_branding` 9/9,
  `ember_build` 46/46, `milestones` 171/171 (with `the_conformance_suite_runs`, the C gate).
- `ember_diag` `the_registry_and_the_specification_name_the_same_codes` fails, as expected, until
  the registry row above is in the development target.
- Gates: `error_pages.py` (E1032's page passes; E9040, below), `rule_index.py` (no new problems),
  `check_branding.py`, `generate_runtime.py --check`, `test_runtime_generation.py`: pass.

## Defects found

1. **Fields `café` and `caf_u00E9_` of one struct are one C member** (`int64_t caf_u00E9_;` twice),
   which the C compiler refuses. `[MNG-3]`/`[CG-C-7]`/D-775's member names; ODR-202 ruled symbols
   only. Locals are fine: the planner makes the second `x__u00E9__2`. Not fixed: field names are in
   generated headers, so the spelling is a decision (question 1). Probe:
   `struct P: café: int; caf_u00E9_: int`.
2. **A program's `struct int` loses to the prelude alias in type position** (`fn get(x: int)` is
   `i64`, "`i64` has no field `n`"), against `[MOD-5]` ("a local declaration … shadows a prelude
   name"): `scalar_named` answers before the program's types (D-305 fixed this for `Box`, `Cell`, …,
   not for `int`, `float`, `str`). Open; not touched.
3. **`E9040` has no error page**, so `tools/error_pages.py` fails (pre-existing since helper U's
   merge). After ODR-201 no program can raise it (only a 64-bit hash collision), so no page can have
   a failing example: baseline it (`--write-baseline --allow-growth`) or teach the tool a code that
   cannot be written. The lead's.
4. D-782 is fixed by `[MNG-1]` and its workaround is gone (its `docs/DEFECTS.md` row is the lead's).

## Spec questions (options, recommendation)

1. **Transliteration is not injective inside a component.** `é_u00E9_` and `éé` are both
   `u14_u00E9__u00E9_` (and one C field, defect 1). (a) In a transliterated component, an ASCII `_`
   that begins what reads as an escape is itself escaped (`_u005F_`): every name stays legal, only
   names mixing both pay; (b) an identifier spelled with `_u` + hex + `_` is an error; (c) accept.
   Recommend (a), applied to symbols and to C member names alike (fields then unique per struct).
2. **`E1032` has no shape** in §XVII.6.2 (`[DIA-12]`: every name error is classified). Recommend a
   shape N14, "an item named like a module or a built-in type", primary help "rename the item".
3. **Prelude names that are built-in types.** A root `struct str` (or `Never`, and `cstr`, which the
   spec puts in `std.ffi` but the compiler knows everywhere) has the path its built-in's methods are
   under, so their symbols can meet; `[MOD-5]` lets a declaration shadow these names, so ODR-201's
   built-in half does not cover them. (a) Make them `E1032` too (narrows `[MOD-5]`); (b) write
   built-in types' paths under `std.core` (`std.core.str`, as the compiler's generics already are),
   which frees every name and leaves `E1032`'s built-in half a matter of source clarity only;
   (c) leave. Recommend (b).
4. **The root package and a dependency of one name.** With the root package's paths having no
   package component (helper U's question 4), a root module `geometry.em` and a dependency
   `geometry`'s root module have one path; ODR-201 covers a module and an item, not two modules
   (by reading the code, not tested: the checker's path maps would keep one of the two). Recommend
   settling U's question 4 by giving the root package's items its package component, or `W1003`'s
   rule extended to a dependency's name.
5. **Which modules `E1032` compares**: built as the modules of the build (imported somewhere), not
   every file under the source root. Recommend stating it.
6. **The import-binding form** (`import paths.grid` beside `fn grid()`): built because the ruling's
   words ("a module and an item of one name in one scope") cover it, and `grid.at` was silently the
   module's. The lead should confirm it is within ODR-201 and not a widening.
