# Helper U — notes (2026-10-22)

Rule: `[MNG-1]` (D-809), with `[MNG-3]`, `[MNG-4]`, `[MNG-2]`, `[HR-7]`, `[CG-C-10]`, `[BLD-2]`, `[CLO-3]`.
Branch `worktree-agent-accf3b9cfc35cc2ac`, from main `ac4d75ac`.

## MNG-1 — done (built)

The scheme as built:

- An item's symbol is `em_` and each component of its path as its length in decimal and then its
  text, transliterated first (`[MNG-3]`): module `lib/m_x.em`'s `f` is `em_3lib3m_x1f`, module
  `lib/m.em`'s `x_f` is `em_3lib1m3x_f`. A root-package item has no package component (as the
  specification's example; see question 4), a dependency's item its package name
  (`em_8geometry6shapes6Square`), std's `std` (`em_3std4core…`). The root module's `main` is
  `em_4main` (`ember_branding::entry_symbol`).
- A method's path is its type's path and its name (`em_8geometry4Grid2at`); a method told apart by
  the interface it implements (`[TYP-24]`, D-440) has the interface's path between
  (`em_5Point3std3fmt7Display3fmt`). A built-in type's methods are under its keyword
  (`em_3i328floordiv`); a compiler-made generic is the prelude's, `std.core.<Name>`
  (`std.core.Option`); a root module's type named like one (`root.Box`, D-305) is `Box`; a
  structural type (tuple, fixed array, function, pointer, `dyn`) as an owner is `std.core.<kind>`
  with its whole spelling as its arguments.
- A generic instance (function, type, a method of an instance, a generic method's instance) ends with
  `G` and the first 16 hex digits of BLAKE3 of the canonical spelling of its arguments. The spelling
  (`TypeTable::spelling`, `arguments_spelling`): `[a, b]` per generic level that has arguments, in
  path order (owner type, interface, the method's own); a declared type by its path, never by the
  instance name the checker gave it (`Pair_i64_2` depended on what else was named); an instance with
  its arguments; scalars by keyword; `String`, `Array`, spans as `std.core.…`; tuples, arrays,
  references, pointers, function types as written; a parameter as `$T0` (name and slot, D-306); a
  function given for a callable parameter as its path after its type (`[fn(i64) -> i64 = add_one]`).
  The `__<DefId>` suffix instances had is gone: an instance's name no longer depends on the order the
  checker made things in.
- `[CLO-3]`'s copy of a function for a function passed to it (`callable_arguments.rs`, was
  `<callee>__via_0_<function>`) is an instance of the callee: `ember_branding::reinstance`, its path
  and the hash of its own hash and `[0 = <function's symbol>]`.
- `[MNG-4]`: `em_obj_`, `em_vt_`, `em_ti_` and the class's mangled form (`em_obj_6Animal`) — see
  question 3. Names the compiler makes for itself start with a letter after `em_`
  (`ember_branding::generated`: `em_closure3`, `em_drop_glue_2`, `em_main_entry`), so no item's
  (digit after `em_`) can meet one; names keyed by types write their mangled forms after `_`
  (`em_drop_fields_6Animal`, `em_vt_dyn_7Observe_for_6Reader`, `em_vt_dyn_7Observe_4Bump` for a
  composed `dyn`, `em_interface_id_7Observe`, `em_rl_fields_…`, `em_tid_…`), which read back exactly
  because a mangled form begins with a digit and says where it ends.
- `E9040`: each instance symbol made is recorded with the instance it was made for (a function
  instance's definition, a method's definition, type and name, a type); another instance with the
  symbol is `E9040` naming both, the first as the secondary place. Two definitions of one method made
  from one path and arguments (overlapping implementations) are `[TYP-19]`'s errors and not reported
  again. Type instances are checked after checking (`check_type_instance_symbols`).

Tests:
- `tests/conformance/MNG-1/accept_each_component_is_written_with_its_length.em` (+ `lib/m_x.em`,
  `lib/m.em`): the specification's pair builds and runs through the harness (prints `9 70`), the C
  holds `em_3lib3m_x1f` and `em_3lib1m3x_f`, and the frame table names each by its path. Before:
  "internal compiler error: [LT-40] callable `em_lib_m_x_f` is present in more than one interface
  artifact" (checked with ac4d75ac's compiler).
- `tests/conformance/MNG-1/reject_two_instances_of_one_symbol_are_both_named.em` (+ `grid.em`, a
  parse-pass case): `E9040` naming `grid.at[i64]` and `grid[i64].at`, a module's generic and a type's
  method of one path (question 1).
- `ember_typeck` `instance_symbol_tests::two_instances_whose_hashes_meet_are_e9040`: the detection
  given one symbol for two instances (what a 64-bit hash collision would make; nothing can write one
  in a program, and the code has no clean seam for a shortened hash): `E9040`, both named, the first
  as the secondary, the hash help; overlapping definitions of one method not reported; a function and
  a method made alike are.
- `ember_branding` tests: the spec's pair, injectivity (`m_x.f`/`m.x_f`), `[MNG-3]` before the
  length, the instance hash, `demangle` round trips (paths, instances, `__rl`, non-ASCII; generated
  names, export aliases and runtime names are no item's), `reinstance`, `foreign_alias`.
- `milestones.rs` `demangle_writes_a_c_stack_in_ember_paths` (see CG-C-10 below).
- Existing needles (`#$ assert-c*`, 162 conformance files, 67 in `tests/run-pass`/`compile-pass`)
  rewritten to the new spelling: each old symbol mapped to the new one at the same place in the C of
  ac4d75ac's compiler and of this one, aligned line by line; the 20 that named a prefix or an absent
  symbol by hand, keeping what each tests (an instance named by its arguments' hash, with a comment
  saying which arguments). Checked: every `assert-c`, `assert-c-count`, `assert-c-order` of
  `tests/conformance`, `run-pass`, `compile-pass`, `milestones`, `run-fail`, `compile-fail` against
  the emitted C in each listed profile (script beside the build, the harness's own parsing).

Compiler changes:
- `compiler/ember_branding/src/lib.rs` — `mangled_path`, `item_symbol`, `instance_symbol`,
  `instance_suffix` (BLAKE3, `INSTANCE_HASH_DIGITS = 16`, `INSTANCE_MARK = 'G'`), `entry_symbol`,
  `reinstance`, `symbol`, `generated`, `transliterated`, `demangle`/`Demangled`,
  `symbol_path_starts_with` (no allocation: the passes ask it of every body); `object_struct`,
  `vtable`, `type_info` take the mangled form; `foreign_alias` writes the declaration's path mangled
  (two declarations of one C function from `m_x.f` and `m.x_f` had one C name); `mangled` and
  `drop_function` removed, so nothing can join text into a symbol; the prefix is made once
  (`LazyLock`). Depends on `blake3` (already in the lock).
- `compiler/ember_types/src/lib.rs` — `note_instance` (each instance name's generic and arguments,
  recorded by the checker's `unique_instance_name`, the one function naming every instance),
  `type_path`, `mangled`, `mangled_struct/class/enum/interface`, `method_symbol`/`method_path`,
  `class_method_symbol`, `spelling`, `arguments_spelling`, `item_path`, `COMPILER_GENERICS`.
- `compiler/ember_typeck/src/lib.rs` — `mangle` (an item's path), `method_symbol` → the type table's,
  `method_symbol_for` with the interface and the instance's own arguments, generic function and
  generic method instances by `instance_spellings` (made in `instantiate`/`instantiate_method`),
  `instance_arguments_spelling`, `callable_spelling`, `InstanceSymbols`/`InstanceMade`,
  `instance_collision` (`E9040`), `check_type_instance_symbols`; closures, comptime functions,
  `main_entry` and EMIF's declaration-only member keys are generated names.
- `compiler/ember_codegen_c/src/lib.rs` — every type's C name from its mangled form, made once per
  type (`mangled_types`, the emitter's cache: a type is named wherever a value of it is);
  `object_struct`/`class_vtable`/`class_type_info`/`class_helper`/`class_drop_symbol`/`drop_symbol`;
  `dyn_table_identity`, `interface_id_symbol`, adapter tables, itable caches by mangled interfaces;
  constant statics are items (`em_<path>`); the planner's taken set no longer lists declared types
  (they cannot meet a structural type's `em_tup_…`); `clone_fns` found a type's clone by
  `symbol.ends_with("_clone")`, which took any method ending in `_clone` with a `self` returning its
  type (`deep_clone`) for it: now by its name, `clone`.
- `compiler/ember_codegen_c/src/reload.rs` — schema, field, drop, tag, payload, variant and migration
  tables and class identities named by mangled forms (`em_rl_fields_8geometry4Grid`); joined with `_`,
  the tables of `a_b.X` and `a.b_X` were one C name.
- `compiler/ember_analysis` — `coroutines.rs`, `effects.rs`, `loop_version.rs`: a type's `drop`,
  `next`, `result` by the type table's symbol (each crate had its own copy of the joining);
  `access.rs`, `long_access_lint.rs`: "is it std's / cpp's" by the symbol's first component (an item
  `std_dev` of the program counted as std's); `inline.rs`: `entry_symbol`; `perf_lints.rs` (L5002):
  the copying conversions by how their paths end; `callable_arguments.rs`: copies are instances.
- `compiler/ember_mir/src/lower/coroutines.rs` — the suspension points' symbols, made once; a yield
  instance is its symbol and `G…`.
- `compiler/ember_driver/src/main.rs`, `comptime.rs` — `standard_symbol` (first component `std`)
  in the seven places that matched `em_std_`; `ember demangle` (below).
- `compiler/ember_build/src/interface.rs` — EMIF schema 11 → 12 (callables keyed by the new
  symbols); a stale cache is not read.
- `tools/check_branding.py` — the mangled prefix's pattern is `em_\w`: a digit-led symbol written in
  a Rust string is caught too.

Not changed, and why: the C++ importer's thunks and wrappers (`ember_cpp_…`, their own names, and
another helper's file); `@export`/`extern "C"` names (`[MNG-2]`, exact); foreign symbols; the
structural types' C names (`em_tup_…`, `em_fn_…`: no item's, made unique by the planner); the
runtime (it names no generated symbol; the reload manifest lists slots by symbol, so `[HR-7]`
follows); the global cache of linked programs (keyed by the C's bytes, which changed).

## CG-C-10 — `ember demangle` reads the scheme (done)

- `ember demangle <file>`: a function of the program is named as its backtrace names it (a lambda as
  "a lambda", a method `geometry.Grid.at`); any other whole word that is an item's symbol is read back
  by its length prefixes (`em_3lib3m_x1f` → `lib.m_x.f`, an instance `lib.max[#<hash>]`, transliteration
  undone), and a near miss (`em_5outer2`) is left alone; `…__rl` is its function's.
- Test: `milestones.rs` `demangle_writes_a_c_stack_in_ember_paths`. It decodes the stack's
  `geometry.*` symbols structurally, so it holds in this branch's layout and in main's
  `support/geometry.em` one.

Main moved `tests/conformance/CG-C-10/geometry.em` to `support/geometry.em` (`import
support.geometry as geometry`); I left the file as at ac4d75ac. The needles of
`accept_frames_are_named_by_path_and_calls_by_place.em` in main's layout (computed with this compiler
on a copy laid out so):

```
#$ assert-c: contains("{"em_7support8geometry4Grid2at", "support.geometry.Grid.at", "support/geometry.em", 7u},")
#$ assert-c: contains("{"em_7support8geometry5probe", "support.geometry.probe", "support/geometry.em", 10u},")
#$ assert-c: contains("{"em_7support8geometry5probe", "em_7support8geometry4Grid2at", "support/geometry.em", 11u, 17u},")
#$ assert-c: contains("{"em_7support8geometry5probe", "em_7support8geometry4Grid2at", "support/geometry.em", 11u, 27u},")
#$ assert-c: contains("{"em_5outer", "em_7support8geometry5probe", "accept_frames_are_named_by_path_and_calls_by_place.em", 22u, 30u},")
```

## HR-7, BLD-2, CG-C-9, MNG-2 — keep working

- Hot reload: call-table slots are body symbols, so they follow; `reload.rs`'s tests name the
  refused functions by `item_symbol` (`em_5fixed`). The runtime's refusal reports print the slot's
  symbol (`refused: \`em_5helper\` was removed…`); a manifest field with the Ember path would read
  better (not built: a runtime and manifest change of its own).
- Interface artifacts: schema 12. Visualisers name types through the emitter (`node_name`), so they
  follow; `@export` names are exact; `extern "C"` definitions keep their names.

## Speed

Symbols are made once: a function's when its `Function` is made, an instance's spelling once per
instance, a type's mangled form once per type per emitter, the coroutine and prefix strings once per
process; `E9040`'s display strings only for a collision. `--timings` of five programs, old against new
on the performance cores (medians of 7 and 11 runs): `check` and `C generation` within run-to-run
noise (the laptop was running other helpers' suites at the time); no laptop benchmark run (the lead's).

## Defects found

1. A module and a type of one name in one module give two items one path: root `struct grid` with
   method `at` and module `grid.em` with `fn at` are both `grid.at` (question 1). Generic: `E9040`
   (tested). Non-generic: one C function defined twice, the C compiler's error, as before this change
   (`em_grid_at` then, `em_4grid2at` now).
2. Pre-existing at ac4d75ac, `tests/conformance/CG-C-10/geometry.em` has no `#$ rules:`, so
   `milestones.rs` `the_conformance_suite_runs` stops at it before running any case (main's move to
   `support/` fixes it).
3. Pre-existing at ac4d75ac (`[OPT-1]`'s stack promotion): five needles expect
   `ember_obj_new(&em_ti_…)` where the object is now made in the frame
   (`ember_obj_new_frame(&_fo1, &em_ti_…)`): `tests/compile-pass/class_construct_empty.em`,
   `tests/run-pass/class_construct_drop.em`, `class_construct_empty.em`, `class_construct_fields.em`,
   `generic_class.em`. Spelling updated only; whether each should keep a heap object (an escaping
   handle) or name the frame form is the lead's.
4. Fixed in passing: `clone_fns` took any `…_clone` method for the type's clone; the std checks took
   a program's `std_*` item for std's; L5002 took any std function ending `_to_cstring`;
   `foreign_alias` could give two declarations one C name; reload tables could meet (`a_b.X`/`a.b_X`);
   D-782 (a struct `A_b` and a method `A.b`) is gone with the scheme (the C++ importer's
   `<Class>_cpp_iter` workaround can go; not touched, another helper's file).

## Spec questions (options, recommendation)

1. **Paths are not unique.** `[MNG-1]` makes symbols injective over paths, and "Two distinct items
   therefore never share a name" holds only if two items never share a path; a module and a type of
   one name in one module (accepted today) give their members one path, and `grid.at(4)` reads either
   way in the source too. (a) A module and an item of one name in one scope is an error (`E1030`'s
   kind); (b) symbols mark a type's members apart from a module's (beyond the spec's example);
   (c) leave it (`E9040` for instances, the C compiler's redefinition otherwise). Recommend (a): the
   ambiguity is the program's, and (b) would only hide it. The same holds for a type named like a
   built-in (`struct i32`, accepted today), whose spelling as an argument is the built-in's.
2. **`[MNG-3]` is not injective.** `café` and the ASCII name `caf_u00E9_` are both `10caf_u00E9_` (and
   one C field or local name, already). (a) A component that was transliterated is marked in its
   length prefix (`u10caf_u00E9_`: a letter after a component is otherwise only `G`), one character on
   non-ASCII names only; (b) an identifier spelled like a transliteration (`_u` + four or more hex +
   `_`) is an error; (c) accept. Recommend (a) for symbols, (b) if field and local names must be
   injective too.
3. **`[MNG-4]` "followed by the mangled type"**: built as the mangled form without its `em_`
   (`em_obj_6Animal`); `em_obj_em_6Animal` is the other reading. Recommend as built; say so.
4. **The root package's component.** `[MOD-1]` puts the package name first in a module's path, but
   the specification's example (`lib/m_x.em`'s `f` is `em_3lib3m_x1f`) has none, as built: a root
   package's items have no package component, a dependency's do. Recommend stating it in `[MNG-1]`.
5. **"The canonical spelling of its arguments"** is not defined; built as above (levels in path order,
   declared types by path, compiler-made generics as `std.core.…`). Separately compiled packages
   deduplicating instances at link time (`[MONO-1]`) need every compiler to spell alike. Recommend
   writing the spelling into XVIII.2.
6. Not in the specification, built so: a method told apart by its interface (path, then the
   interface's path, then the name); built-in types' methods under their keyword; a structural owner
   as `std.core.<kind>`; `[CLO-3]` copies as instances.
