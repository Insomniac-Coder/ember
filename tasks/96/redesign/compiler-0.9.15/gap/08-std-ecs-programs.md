# Gap map 08: the std library, the ECS, and every Ember program outside `tests/conformance`

Agent 08, 2026-10-27. Read-only. Nothing was built or run. Scratch scripts and their outputs are in
`target/gap-0.9.15/08/`:
- `scan.py`: the spelling scanner;
- `loops.py`: loops that change what they walk;
- `staticfn.py`: type-body functions with no object;
- `gen.py` and `tests_table.md`: the per-file classification;
- `scan_std.json`, `scan_tests.json`, `scan_conf.json`.

## 0. Method and how far to trust it

- **Spellings.** `scan.py` strips comments, then matches each line against two lists:
  - **migratable**: what `[CLI-20]`'s `ember fmt --migrate` rewrites, plus the other plain renames of
    0.9.15 (`[GRM-40]` bracket lines, `x: ref T` parameters, `T: Sync` bounds,
    `for v in xs.iter_mut():`);
  - **not migratable**: what `[CLI-20]` names as needing a choice (`Shared`, `RefCell`, `SyncShared`,
    `@sync`, `owned fn`, `mem.forget`), and the old ECS API.

  A file that has only migratable spellings is `respell`. A file with a non-migratable spelling, a
  retired code, or a meaning change I found by hand is `rewrite` or `retire`.
- **Read by hand:**
  - every file in `tests/conformance/ECS-*` (27);
  - the STD files the scan or the rule diffs flagged: STD-9 (two), STD-11/reject_a_borrowed_view_as_a_key,
    STD-15/reject_sorted_needs_clone, STD-19 (the four ODR-240 files and reject_copied_of_values_that_are_not_copy),
    STD-26/accept_sorted_of_an_iterator_of_references_copies_or_clones;
  - in `tests/`: `run-pass/shared_construction_and_get`, `compile-fail/weak_nonclass`,
    `ui/borrow/O8/a_guard_forgotten`, `ui/borrow/B2`'s golden;
  - in `bench/programs/`: `p2_view_held`, `p4_views_alive_04`, `p7_mut_self_calls_a_function`;
  - the 9 design tests that use an ECS shape.

  Everything else was classified by the script.
- **Known limits of the scan:**
  - The `R` tag (`x: ref T`) over-counts. A struct field typed `ref T` stays legal (a kept view, R7).
    Only a parameter's is refused (E0117).
  - The scan cannot see that **class objects now have one owner** (`p2 = player` hands the object
    over), and cannot see call sites that now need `ref mut`.
  - 196 non-conformance test files declare a class. The 151 `respell` files among them are marked
    `†`: their meaning holds only if no handle is copied with `=`, and each needs a look by hand.
- **Counts versus the prompt.** The prompt said "83 test files and 4 bench programs". My narrow scan
  counted 42 files with `Box`, `Shared`, `Weak`, `RefCell`, `SyncShared`, `ForeignBox`, `mem.forget`,
  `@sync` or `@borrows`, and 79 once `owned` is added (75 tests, 4 bench). The prompt's figure
  probably counted comments. My final classification covers **all 511 test files and 69 bench
  programs**, so nothing depends on that figure.

## 1. Rule table

H.41 lists 17 STD rows (16 changed, STD-29 added) and 8 ECS rows (6 changed, ECS-8 and ECS-9
added). Two more rows are added below:
- ECS-6, unchanged, because its implementation and tests move with the rest;
- `[DRV-1]`'s `Component` row, a derive the compiler implements under a name the spec removed.

Design tests: none of the 841 names an STD or ECS rule in its `#$ rules:` line (searched by ID in
every `#$` line of `run4/v5/conformance/`). Nine use the ECS's shape:
- R14 (6): a local `Registry`/`Pool` sketch, consistent with `[ECS-2]`;
- R4 (2) and R8 (1): written against the **removed** `World`/`Query`/`Commands` API (risk 5.3).

| rule | H.41 kind | what the spec now requires (one line) | today in the compiler | status | what must change | old tests (`tests/conformance/<RULE>/`) | design tests |
|---|---|---|---|---|---|---|---|
| ECS-1 | changed | `Entity` is a 32-bit handle (20-bit index, 12-bit generation), EnTT's `entt::entity`; `NULL` all ones | `std/src/ecs.em:20-44` (`Entity`: `new`, `index`, `generation`, `to_bits`, `is_null`, `NULL`) | text only | none to `Entity`; it moves into the new module unchanged | 1: respell 1 (`accept_an_entity_is_a_20_bit_index_and_a_12_bit_generation.em`: `World`→`Registry`, `is_alive`→`valid`) | 0 |
| ECS-2 | changed | one sparse-set pool per type, dense contiguous (not paged); identity = FNV-1a-64 of the qualified name for **any** type; `pools: Map[u64, owns dyn AnyPool]`, reached by `ref mut cast this.pools[k] as Pool[T]` | the sparse set is built: `ecs.em:97-139` `AnyStorage` (entities, sparse slot+1, swap-remove); `ecs.em:140-163` `AosStorage[T]`; `ecs.em:165-191` `SoaStorage` (SoA layout). `World.storages: Map[u64, AnyStorage]` is a class map, downcast by class `cast` (`ecs.em:194-199`, `486-490`). `type_hash[T: Component]` (`ecs.em:59`) needs the derive, whose `extend` the compiler writes (`ember_typeck/src/derives.rs:214-256`, FNV at `164-168`) | partly | drop `SoaStorage`. Pools become `owns dyn AnyPool` with a place cast. `type_hash` works for any `T` (Q2). Needs `owns dyn` (area for ownership types) and a cast of a place (`[TYP-44]`/`[DSP-4]`, area for kind tests) | 2: rewrite 1 (`accept_each_component_type_is_a_sparse_set_keyed_by_its_type_hash.em`: no SoA, no derive, `Pool` for `AosStorage`); retire 1 (`reject_a_component_layout_that_is_not_soa_or_aos.em`, `@component`, Q3) | 0 by ID; R14: 6 exercise the pool registry |
| ECS-3 | changed | `Registry` class with EnTT's methods (`create`, `create_at`, `destroy -> u32`, `valid`, `current`, `clear`, `clear_of`, `emplace…`, `patch`, `insert`, `remove`, `erase`, `get`, `get_mut`, `try_get…`, `get_or_emplace`, `all_of`, `any_of`, `orphan`, `storage[T]() -> ref mut Storage[T]`); several types are one tuple type | not built: `World` (`ecs.em:194-383`) has `create`, `is_alive`, `destroy -> bool`, `add`, `remove -> bool`, `has`, `storage -> Option[AnyStorage]`, `entities_with`, `run`, `run_parallel`, `apply` | not built | replace `World` with `Registry`. The sparse-set code is reused. `[(A, B)]` type arguments need one-element tuples for `[(A,)]` (Q1) | 9: rewrite 7 (`accept_a_query_item_is_kept_across_turns.em`, `accept_option_and_not_filter_and_items_follow_each_parameter.em`, `accept_the_smallest_required_storage_drives_the_query.em`, `accept_the_worked_example_runs_as_written.em`, `reject_iter_mut_needs_a_mut_query.em`, `reject_writing_a_component_the_query_reads.em`, `reject_writing_through_an_item_iter_yields.em`: queries become views, codes become E3023); retire 2 (`accept_a_query_with_no_required_component_visits_every_live_entity.em`: ODR-142's query with no required component is gone; `accept_an_soa_component_yields_element_proxies.em`: no SoA pools) | 0 |
| ECS-4 | changed | **views**: `view[(A, B)]()` gives read-only items, `view_mut` gives `ref mut` items; `.exclude[(C,)]()`; `each`, `get`, `contains`, `size_hint`, `front`, `back`; walks the smallest pool; blanket implementations for tuples of 1 to 8 | not built. Today's `Query` (`ecs.em:1088-1163`) and tuple `ParamOps` implementations for 2 to 8 (`ecs.em:783-1087`) are the pattern to reuse. The compiler special-cases `run_parallel`: `ember_typeck/src/lib.rs:44926-44932` (hook), `44949-44984` (`ecs_system_of`, ODR-141 coercion), `44986-45039` (E7020, E7005), `45042-45060` (`ecs_access_set`), `38951-39020` (`check_ecs_system_sendable`, E7004), `35293-35295` (`check_expr_through`), field `3221-3223`, `3846` | not built; the scheduler support is **remove** | build views. Remove every compiler line listed. Retire E7020: `ember_diag/src/codes.rs:403` takes "retired:" as `E9033` does at `:424`, and `docs/errors/E7020.md` becomes a retired page. One-element tuples (Q1) | 11: retire 11 (`accept_a_named_system_is_lent_to_run_and_kept.em`, `accept_a_run_borrows_only_the_storages_its_query_names.em`, `accept_run_parallel_runs_its_systems_at_once_on_threads.em`, `accept_run_parallel_runs_systems_whose_sets_do_not_conflict.em`, `accept_two_lambda_systems_capturing_one_sync_object.em`, `reject_a_system_or_component_run_parallel_cannot_send.em` (E7005 retired), `reject_systems_that_conflict_in_one_run_parallel.em` (E7020 retired), `run_fail_a_parallel_system_adds_a_component_that_is_not_send.em`, `run_fail_a_system_writes_a_storage_its_query_holds.em`, `run_fail_systems_passed_as_values_conflict.em`, `run_fail_systems_passed_as_values_share_a_component_that_is_not_sync.em`) | 0 by ID; R4: 2 stale (`Query`) |
| ECS-5 | changed | adding, removing, creating or destroying through the registry while a `for` walks a view is refused when compiling (`[BRW-18]`, E3020, with the given message); values change through `ref mut` items | today: a `Commands` buffer (`ecs.em:1164-1247`, an `@sync` class at `:1206`) is applied after the system. A change through the world during a query panics at run time (test `run_fail_destroying_through_the_world_while_a_query_iterates`) | not built | delete `Commands`. The refusal follows from two other areas: a view's result points into the registry (LT-1, regions area), and the loop rule refuses a change of what a loop walks (`[BRW-18]`, loops area). The library has nothing ECS-specific to do beyond returning views tied to `this` | 2: retire 1 (`accept_commands_apply_after_the_system_returns.em`); rewrite 1 (`run_fail_destroying_through_the_world_while_a_query_iterates.em` becomes compile-fail E3020) | 0 by ID; R8: 1 stale (`Commands`) |
| ECS-6 | (unchanged) | insertion order with swap-remove holes | built in `AnyStorage.swap_remove_entity` (`ecs.em:126-131`) | built | carry over to `Pool[T]` | 1: rewrite 1 (`accept_iteration_order_is_insertion_order_with_swap_remove_holes.em`: `World.run`/`Query` become a view; its `stdout` stays) | 0 |
| ECS-7 | changed | any type is a component, with no registration and no derive | `@derive(Component)`: `ember_typeck/src/derives.rs:27` (in `WRITTEN`), `40-120` (registry built, written into `std.ecs`), `122-135` (`@component` without the derive, E0104), `137-163` (`component_registry_key`), `177-213` (`component_layout`), `214-256`, `257-279` (`components()` body). `ember_driver/src/lib.rs:4198-4202` (interface-cache key). `std/src/ecs.em:45-81` (`Component`, `ComponentInfo`, `components()`) | remove | delete all of it. `type_hash` needs no derive (Q2). `@component` depends on Q3 | 1: retire 1 (`accept_derive_component_registers_the_type.em`) | 0 |
| ECS-8 | added | signals (`on_construct`/`on_update`/`on_destroy`, `connect`, `disconnect` of `fn(ref mut Registry, Entity)`, called after the change with no view in use); `sort[T](less)`, `sort_as[To, From]`; `ctx()`; `group[(A, B)]()`; `Snapshot`, `SnapshotLoader`, `ContinuousLoader`; `RuntimeView`; `Handle` | nothing | not built | library work (A8-4). Signals depend on function-value `==` (`[CLO-1]`, ODR-102, for `disconnect`). `Handle` holds a changeable link to the registry (`sharedref mut`, R7 area) | none (new tests needed) | 0 |
| ECS-9 | added | no scheduler; work run at once goes through `std.thread`/`std.jobs`; two pieces of work that may touch one pool, one changing it, are refused when compiling | today: a scheduler (`ecs.em:320-383` `run`/`run_parallel`/`apply`, `1248-1303` `System`/`system`/`run_work`/`run_staged`), checked partly at run time | remove (scheduler) + not built (the compile-time refusal) | remove the scheduler. Remove the builtins `TypeIsSend`/`TypeIsSync`, which only `std.ecs` uses: `ember_hir/src/lib.rs:2050-2053`, `2457-2458`; `ember_typeck/src/lib.rs:16805-16808`, `16837`, `16867`, `16882`, `16892`; `ember_mir/src/lower.rs:3507`; `ember_codegen_c/src/lib.rs:10961-10967`; `ember_codegen_llvm/src/builtins.rs:790-795`; `ember_interp/src/builtin.rs:643-644`; `ember_analysis/src/borrows.rs:3991-3992`; `ember_analysis/src/effects.rs:1537`. The refusal is the general thread/job rule (concurrency area, verdict 98): nothing ECS-specific | (in ECS-4's 11) | 0 |
| DRV-1 (`Component` row) | changed | the built-in derive table no longer has `Component` | `derives.rs:27` `WRITTEN` holds `"Component"` | remove | as ECS-7; `@derive(Component)` then falls to the unknown-derive error | (ECS-7's) | 0 |
| STD-3 | changed | `math.fma`; fused arithmetic is otherwise `@fp(allow_fma)` | `fma` built (`std/src/math/mod.em`). The argument is spelled `contract`: `ember_typeck/src/lib.rs:2077-2081` (`fp_names_contract`), `ember_types/src/lib.rs:3636` | partly | accept `allow_fma`; refuse `contract` with `[CLI-20]`'s rename. `FpMode::Contract` and its uses in `codegen_c/src/lib.rs:1653`, `5981`, `6040` keep their meaning | 2: keep 2 | 0 |
| STD-6 | changed | layers: `alloc` holds the heap blocks of `owns` and `shares` values (was `Box`, `Shared`) | `ember_driver/src/packages.rs:125-150` (layers) | text only | none here; the layer of the new words follows the `owns`/`shares` representation (another area) | no folder | 0 |
| STD-8 | changed | `Contains` for `StrView`/`String` (was `str`) | built; the `str` name is everywhere | partly | the `str`→`StrView` rename (types area); nothing STD-specific | 4: keep 3, respell 1 (`accept_a_scan_calls_a_written_eq.em`: `self`) | 0 |
| STD-8b | changed | `StrView` implements `Contains[char]`, `Contains[StrView]` | built under `str` | partly | the rename | no folder | 0 |
| STD-9 | changed | `print`'s `Display`→`Debug` fallback; a class object always has `Debug`; RefCell example dropped | built (ODR-113 predates 0.9.15) | text only | none | 8: keep 3; respell 3 (`accept_a_separator_that_is_not_a_literal.em` M, `accept_generic_formatting_with_text_bounds.em` G, `reject_generic_formatting_without_text_bounds.em` G, `Box[T]`→`owns T`); rewrite 2 (`accept_a_print_is_one_write_between_threads.em`: `owned fn` to `thread.spawn`; `reject_printing_a_value_with_no_printer.em`: its value is a `RefCell`) | 0 |
| STD-11 | changed | `Map[K, V, H]`, memory from `using A`; a map may hold views and lives no longer than they do (E3063 only where it would outlive them) | `Map[K, V, H]` built (`std/src/collections.em:416`), no allocator parameter. Views refused unless `static`: `ember_analysis/src/stores.rs:435` ("may not be passed to `insert` … only a `static` view") | partly | `using A` (`[ALC-1]`, memory-manager area). Replace the static-only store rule with the region rule (`[TYP-15]`, ODR-069, regions area) | 6: keep 3; respell 2 (`accept_insertion_order.em` C, `accept_static_views_as_keys_and_elements.em` V); rewrite 1 (`reject_a_borrowed_view_as_a_key.em`: storing a view of a local into a local map is now legal) | 0 |
| STD-12 | changed | keys looked up by a view; `fn is_key(key: …)`, `fn to_key()`; a key written through a `StrView` is stored as the map's own `String` | built (`AsKey`/`ToKey`, ODR-067) | text only | spellings only | 8: keep 1; respell 7 (`accept_a_key_that_cannot_be_cloned.em`, `accept_a_lookup_type_that_only_matches.em`, `accept_compact_map_lookup_keeps_checked_semantics.em`, `accept_str_keys_for_a_string_map.em`, `accept_string_and_str_keyed_maps_in_one_program.em`, `reject_index_assignment_that_would_copy_the_key.em`, `reject_inserting_through_a_type_that_only_matches.em`) | 0 |
| STD-14 | changed | callers' bugs panic with no twin, now naming `Registry.emplace` on a dead entity | built for the rest; `World.add` on a dead entity panics (`milestones.rs:5059` catalogue row) | not built (needs `Registry`) | the panic in `Registry.emplace`; the catalogue rows at `ember_driver/tests/milestones.rs:5053-5054`, `5058-5059` change (`run_parallel` and `add` rows go, `emplace` comes) | no folder | 0 |
| STD-15 | changed | `retain`, `sort_by`, `sort_by_key` take `[F: …]` + `owns f: F`; a callback that changes the list is E3022; every reshape ends views and is refused in a loop (E3020); `xs.get(i)` names a position | signatures built in today's spelling (`std/src/core.em:2318`, `2328`, `2334`: `owned keep: F`) | partly | respell; the E3022/E3020/E3060 behaviour belongs to the effects and loops areas | 26: keep 19; respell 7 (`accept_an_array_of_void.em` B,C, `accept_array_queries.em` V, `accept_more_array_methods.em` C,T, `accept_sort_by_and_sort_by_key.em`, `accept_sort_of_ord_types.em`, `accept_sort_past_one_run.em`, `reject_sorted_needs_clone.em`: `self` and the rest) | 0 |
| STD-16 | changed | `Map` ops; `for k, ref mut v in m:`; nothing changes a map while a loop walks it (E3020); `retain(keep)` with `fn(K, V)`; `into_items` `owns this`; `sorted(m)` gives views; Set ops give copies | most built. `retain[F: fn(ref K, ref V) -> bool]` (`collections.em:662`) and Set `fn(ref T)` (`:1048`) use the old parameter types. No (key, `ref mut` value) iteration, only `values_mut` (`:1002`). `sorted(m)` clones (see STD-26) | partly | new callback types; a `MapItemsMut` iterator and the `for k, ref mut v` form (loops area, comma rule R3); the loop rule (loops area); `sorted` views (STD-26) | 5: keep 5 | 0 |
| STD-17 | changed | `IndexSet`/`IndexMut`; "dropping" → "ending" | built | text only | none | 4: keep 2; respell 2 (`accept_a_mutable_view_of_a_map_entry.em` T,I, `reject_a_mutable_view_of_a_borrowed_maps_entry.em` T) | 0 |
| STD-18 | changed | `format_to(ref mut buf: MutSpan[u8], …) -> Result[StrView, FmtError] from buf`, called `format_to(ref mut buf, …)` | `std/src/fmt.em:38` `format_to(buf: MutSpan[u8]) -> Result[str, FmtError]`; compiler-known in `ember_hir/src/lib.rs:1879`, `ember_analysis/src/effects.rs:1034-1035`, `1355`, `1531`; E2250 built | partly | signature respell. The call word (E3028) and the `from` clause belong to other areas (R4, R6) | 4: keep 1; respell 3 (`accept_format_is_one_value_as_an_fstring.em`, `accept_format_to_writes_into_a_caller_s_buffer.em`, `reject_format_allocates_where_format_to_does_not.em`) | 0 |
| STD-19 | changed | `copy_each` replaces `copied`/`cloned` (E1010 with help); adapters and consumers take `owns this` and `[F: …]` + `owns f: F`; **the function is handed over**, as for any `owns` parameter (ODR-240's lending is gone); a consumer's call makes what its function changes, and an adapter's driver does | adapters built with `owned` (`core.em:758-885`). `copied`/`cloned`: `core.em:1237` and on, and `ember_typeck/src/lib.rs:11147`, `33000-33014`, `45576`, `45767`. ODR-240 lending: `lend_named_callable` `ember_typeck/src/lib.rs:40811-41019`, calls through the reference `47633`, `47646`, `56594`, state `3243` | partly; lending is **remove** | rename to `copy_each` and refuse the old names; delete `lend_named_callable` and its call paths. Effects of a given function belong to the effects area (R5/R12) | 44: keep 14; rewrite 5 (`accept_a_named_callable_is_lent_and_kept.em`, `accept_a_named_callable_is_lent_to_the_option_and_result_methods.em`, `reject_a_callable_whose_call_gives_a_capture_away_is_given_not_lent.em`, `reject_reading_what_a_lent_callable_changes_before_its_last_use.em`: lending gone, a second use of a handed-over function is E3040; `reject_copied_of_values_that_are_not_copy.em`: `copy_each` of a `String` is legal); respell 25 | 0 |
| STD-21 | changed | `std.math` functions; `[T: Number]` on its own line (`[GRM-40]`) | built | text only | respell `math/mod.em`, `math/det.em` | 12: keep 12 | 0 |
| STD-25 | changed | `rng.shuffle(ref mut span)`, parameter `ref mut s: MutSpan[T]`; each call changes `rng` | `std/src/random.em:70` `shuffle[T](mut self, mut items: MutSpan[T])` | partly | signature respell; the call word belongs to the R4 area | 2: keep 2 | 0 |
| STD-26 | changed | `sorted(it)` copies none: over views it is a list of views, ended by a reshape (E3060); a list of your own is `sorted(m.keys().copy_each())` | ODR-207 clones: `items_array` `ember_typeck/src/lib.rs:32996-33020` adds `copied`/`cloned`; `compiler_sorts` `:32613-32632` | partly | delete the cloning in `items_array`. An `Array` that holds views needs the regions area's `[TYP-15]` | 22: keep 20; rewrite 1 (`accept_sorted_of_an_iterator_of_references_copies_or_clones.em`: `names: Array[String] = sorted(m.keys(), …)` no longer type-checks); respell 1 (`accept_sorted_with_key_and_reverse.em` C) | 0 |
| STD-29 | added | `Volatile`'s `read_volatile`/`write_volatile` are `unsafe fn`s whose `@safety` asks for a valid **and aligned** pointer | `std/src/mem.em:25-40`: `unsafe fn`s; `@safety` says "valid for a read/write", no alignment | partly | add the alignment to the three `@safety` texts | no folder | 0 |

## 1b. The std modules

Totals over `std/src` (26 files, 13,008 lines):
- 3,092 lines with `self`, 311 bracket-after-name declarations (`[GRM-40]`), 156 `owned`, 106 `mut`
  parameters, 90 `str`, 44 `@borrows`, 13 `.clone()`, 10 `Box`, 4 `SyncShared`, 3 `ForeignBox`,
  2 `owned fn`, 1 `@sync`, 1 `mem.forget` declaration;
- about 310 functions in type bodies with no object, which become `static fn` (`staticfn.py`, an
  approximate count).

23 of the 26 modules need changes (3 keep). 6 need more than mechanical respelling.

The removed **std types are compiler built-ins, not std code**:
- `Box`, `Shared`, `SyncShared`, `Weak`, `Cell`, `RefCell` are resolved and built in
  `ember_typeck/src/lib.rs`: `15084-15133` (names), `19397-19981` (instance structs), `38218-38523`
  (constructors), `48133-48140`, the prelude list `1717`, and `ember_types/src/lib.rs:1825-1833`, `3518`.
- Their runtime support is in `runtime/ember_rt/templates/ember_rt.{h,c}.in`. `include/` and `src/`
  are copies generated from the templates.
  - Object header (`h.in:1418-1450`): `strong`, `weak`, `access`, `flags`, `ti`, 24 bytes.
  - `ember_weak_retain`/`release`/`upgrade` (`h.in:1484-1488`, `c.in:3400`) carry over to `weakref`.
  - The access word, `ember_access_begin_*` (`h.in:1950-1956`, `c.in:3449`), and
    `ember_panic_exclusivity` (`h.in:2646-2711`) go: there are no run-time exclusivity checks.
  - `ember_panic_refcell` (`h.in:2732`, `c.in:9353`) goes with `RefCell`.
  - The 8-byte header the spec gives a class (a 32-bit count and a 32-bit type index) belongs to the
    representation area.
- `mem.forget` is the builtin `MemForget`:
  - `ember_hir/src/lib.rs:2063`, `2467`;
  - `ember_interp/src/builtin.rs:638`;
  - `ember_codegen_c/src/lib.rs:8795`, `10136`;
  - `ember_codegen_llvm/src/builtins.rs:313`;
  - `ember_analysis/src/effects.rs:1538`.

  It stays as the hidden body of `into_raw()` (decision 3.5).
- `ForeignBox` is std code (`std/src/ffi.em:165-196`), and the C++ importer writes it into generated
  Ember: `ember_driver/src/cimport.rs:6767`, `6848`, `7209`, `7378`, `7447`, `7575-7576`.

| module | lines | spellings found (count of lines) | what it becomes beyond spelling | class |
|---|---|---|---|---|
| `arena.em` | 12 | — | — | keep |
| `lib.em` | 31 | — | — | keep |
| `process.em` | 39 | — | — | keep |
| `env.em` | 15 | `str` 2, `mut` param 1 | — | respell |
| `hot.em` | 36 | `owned` 1 | — | respell |
| `time.em` | 70 | `self` 14; 5 static fns | — | respell |
| `io.em` | 104 | `self` 5, `mut` 3, `str` 1; 2 static fns | — | respell |
| `fmt.em` | 66 | `mut` 3, `self` 2, `str` 2, G 4, `x: ref T` 2 | `format_to(ref mut buf: MutSpan[u8]) -> Result[StrView, FmtError] from buf` (STD-18); `Display.fmt`'s `mut f` becomes `ref mut f` | respell |
| `fs.em` | 241 | `str` 20, `self` 27, `mut` 6, `owned` 2 | — | respell |
| `string.em` | 245 | `self` 62, `str` 15, `@borrows` 4, `mut` 1 | `str` methods become `StrView`'s; `@borrows` becomes `from` | respell |
| `random.em` | 94 | `self` 18, G 2, `@borrows` 1, `mut` 1 | `shuffle(ref mut s: MutSpan[T])` (STD-25); `choice` result `from` | respell |
| `simd.em` | 622 | `self` 84, G 1; 30 static fns | — | respell |
| `math/mod.em` | 2,054 | `self` 678, G 23; 43 static fns | — | respell |
| `math/det.em` | 871 | `self` 40, G 8 | — | respell |
| `ser/mod.em` | 337 | `self` 57, G 30, `mut` 26, `str` 11, `owned` 3 | `[SER-1]`: `[W: ser.Writer]` above `fn serialize(ref mut w: W)` | respell |
| `ser/json.em` | 371 | `self` 182, `str` 13, G 2 | — | respell |
| `ser/binary.em` | 294 | `self` 127, `str` 7, G 2 | — | respell |
| `collections.em` | 1,814 | `self` 605, G 68, `owned` 34, `mut` 23, `x: ref T` 13, `@borrows` 8, `.clone()` 7, `str` 5, `if Some` 1; 29 static fns | STD-16: `retain` callbacks `fn(K, V)`/`fn(T)` (`:662`, `:1048`); a (key, `ref mut` value) iterator for `for k, ref mut v in m:`; `or_insert_with(owns f) owns this`; `into_items` `owns this`; Set ops return copies; `using A` (memory-manager area). The private method `forget` (`:688`, `:696`) is only a name and can stay | respell + API changes |
| `core.em` | 3,167 | `self` 851, `owned` 90, G 87, `@borrows` 30, `mut` 5, `.clone()` 5, `str` 4, `Box` 4; 74 static fns | STD-19: adapters `owns this` + `owns f: F`; `copied`/`cloned` (`:1237`) become `copy_each`; `peek`/`peek_mut`/`next_back` lose `mut self`; `FromIterator.from_iter` becomes a `static fn`. `AnyError.error: Box[dyn Error]` (`:92-98`, `:211`) and `dyn_payload(boxed: Box[dyn Error])` (`:247`) become `owns dyn Error`. Array `get_pair_mut`, `sort_by`, … respell | respell + API changes |
| `thread.em` | 40 | `owned` 1, G 3, `x: ref T` 2 | `[R]` above `fn spawn(owns f: thread_shared fn() -> R)` (`:25`): a new type word, not a rename | rewrite |
| `jobs.em` | 49 | `owned` 1, `x: ref T` 2 | `submit(owns f: thread_shared fn())` (`:23`), as `thread.spawn` (concurrency area) | rewrite |
| `coroutine.em` | 103 | `self` 19, `Box` 2, `owned` 2, G 2, `owned fn` 1 | `Array[owns dyn Sequence]` (`:53`, `:63`); `owned fn() => pred()` (`:40`) needs a capture word (`fn() owns pred: pred()`) | rewrite |
| `mem.em` | 206 | G 19, `self` 17, `owned` 7, `mut` 3, `assert_disjoint` 3, `str` 1 | delete `forget` (`:97`); rename `assert_disjoint` (`:76`) to `check_disjoint` and `assert_disjoint_or_panic` (`:85`) to `assert_disjoint`; `Volatile` `@safety` gets alignment (STD-29); `Allocator` named by `using` | rewrite |
| `sync.em` | 279 | `self` 48, G 16, `owned` 4, `SyncShared` 4, `x: ref T` 3 | `SyncShared[T]` (`:104`) and the channel states (`:168`, `:222`, `:278`) become `thread_shared ChannelState[T]`; `Lock` and `guarded by` are new (concurrency area) | rewrite |
| `ffi.em` | 545 | `self` 102, G 12, `owned` 4, `ForeignBox` 3, `str` 3, `x: ref T` 3, `mut` 1, `mem.forget` 1 | `ForeignBox[T]` (`:165-190`) becomes `owns T` on a foreign type; `adopt` returns `owns T` (`:195`); `into_raw()` only inside `unsafe` (R16), body `MemForget`. Where the creator thread and the run-time record (`[FFI-33b]`, `[FFI-36b]`) live is the FFI area's call. `CppShared`/`CppWeak` stay | rewrite |
| `ecs.em` | 1,303 | `self` 154, `x: ref T` 36, `mut` 33, G 32, `owned` 7, `str` 6, `Box` 4, `owned fn` 1, `@sync` 1, old ECS API 58 lines | replaced whole: `Registry`, `Pool[T]`/`Storage[T]`, `AnyPool`, views (ECS-1 to ECS-9) | rewrite (replace) |

## 1c. Programs outside `tests/conformance`

Totals:

| set | files | keep | respell | rewrite | retire |
|---|---|---|---|---|---|
| `tests/compile-fail` | 59 | 18 | 36 | 5 | 0 |
| `tests/compile-pass` | 13 | 1 | 11 | 1 | 0 |
| `tests/run-pass` | 212 | 44 | 146 | 22 | 0 |
| `tests/run-fail` | 5 | 5 | 0 | 0 | 0 |
| `tests/ui` (`.em`; 68 `.stderr` goldens beside them) | 137 | 72 | 11 | 50 | 4 |
| `tests/firstweek` | 60 | 1 | 57 | 2 | 0 |
| `tests/perf` (`.em`) | 25 | 16 | 8 | 1 | 0 |
| **tests total** | **511** | **157** | **269** | **81** | **4** |
| `bench/programs` | 50 | 33 | 6 | 11 | 0 |
| `bench/probes` | 19 | 8 | 7 | 4 | 0 |
| `examples/` | 1 | 1 | 0 | 0 | 0 |

- **`tests/ui/borrow/`** (27 cases, each `.em`, `.fixed.em`, `.stderr`) is the old memory-model
  message suite. Its goldens hold the old texts ("cannot mutate `values` while it is borrowed by this
  loop"). The 0.9.15 code table and the owner's message verdicts give new texts, so every
  E3xxx golden is a rewrite.
  - B8 (E3025) and B14 (E3065) retire: those codes are retired with no successor.
  - O3 (E3041 → E3040), O4 (E3042 → E3040), O5 (E3030 → E3013), B15 (E2228 → E2020) and X1
    (E3080 → E3021/E3022) are rewritten to the new code.
- **`assert-c` lines.** 106 tests assert on the generated C. 12 run-pass files assert
  `ember_object_begin_write`/`ember_field_begin_write`, run-time exclusivity calls 0.9.15 removes,
  so those assertions invert. 4 assert `ember_box_new_copy`: they hold only if `owns dyn` keeps
  `Box`'s lowering. They are listed in the table below.
- **Benchmarks.** The p-set (`p1` to `p7`, `w1`) measures the run-time access words, and is named
  after `mut self`. Under 0.9.15 the element names need `ref mut`. `p4_views_alive_*` keeps views of
  `bags[2..5]` while it changes `bags[round % 2]`, which R8 (f) refuses unless the compiler proves
  the two places different. The owner's rule "name benchmarks by what they do" means `p3_mut_self_*`
  and `p7_mut_self_*` need new names: a choice for the owner when the work is done, not a spec
  question.

The full per-file classification is section 6 (generated by `gen.py`).

## 2. Work items

Ordered; slices are in section 2b.

**A8-1. A general type-name intrinsic, so `ecs.type_hash[T]()` works for any `T`.** S.
- **Changes:**
  - `std/src/core.em` declares `type_name[T]()` beside `type_identity` (`:240-244`);
  - `ember_typeck/src/errors.rs:219-250` (where `type_identity` is known) and the builtin table
    `ember_typeck/src/lib.rs:16790-16810`;
  - `ember_hir` `Builtin`, folded to a constant in `codegen_c`, `codegen_llvm` and `interp`.
  - `std/src/ecs.em` computes FNV-1a-64 over it at compile time.
- **Completes:** part of ECS-2 and ECS-7.
- **Tests:** ECS-2's rewrite.
- **Depends on:** Q2.

**A8-2. One-element tuple types `(A,)`.** S.
- **Changes:** `ember_parser/src/decls.rs:1516-1527` (today `(A)` and `(A,)` both make
  `Tuple([A])`); `ember_typeck/src/lib.rs:14709-14715` (a tuple has two or more elements, ODR-071).
- **Completes:** the `[(A,)]` forms of ECS-3 and ECS-4.
- **Depends on:** Q1.

**A8-3. `std.ecs` as EnTT's registry, in today's spelling, with the compiler's ECS support removed.** L.
- **Changes in `std/src/ecs.em`:**
  - remove `World`, `Query`, `QueryIter*`, `Mut`, `Not`, `QueryParam`, `ParamOps`, `Read`/`Write`/
    `ReadSoa`/`WriteSoa`/`Maybe`/`Without`/`Entities`, the tuple `ParamOps` implementations,
    `SoaStorage`, `Access`, `conflict`, `shared_unsync`, `type_is_send`/`type_is_sync`, `Commands`
    and its changes, `System`, `system`, `run_*`, `Component`, `ComponentInfo`, `components()`;
  - add `Registry` (every ECS-3 method), `Pool[T]` with the Q5 name and `AnyPool`, and views
    (`view`, `view_mut`, `exclude`, `each`, `get`, `contains`, `size_hint`, `front`, `back`) with
    blanket implementations over tuples of 1 to 8.
- **Changes in the compiler:**
  - `ember_typeck`: `derives.rs` lines of ECS-7; `lib.rs` lines of ECS-4 and ECS-9;
  - the `TypeIsSend`/`TypeIsSync` builtins in every crate (ECS-9 row);
  - `ember_driver/src/lib.rs:4198-4202`;
  - `ember_diag/src/codes.rs:403` (E7020 marked retired); `docs/errors/E7020.md` (retired page);
  - `ember_driver/tests/milestones.rs:5053-5054`, `5058-5059` (catalogue rows) and `6788-6830`
    (component-key test, deleted);
  - `ember_driver/tests/reload.rs:867-875` (the hot-reload ECS program rewritten to `Registry`).
- **Completes:** ECS-1, ECS-2 (except the 0.9.15 forms), ECS-3, ECS-4, ECS-6, ECS-7, DRV-1's row,
  ECS-9's removal, and STD-14's `Registry.emplace` panic.
- **Tests:** ECS folders: 1 respell, 10 rewrite, 16 retire, and new tests for ECS-3 and ECS-4.
- **Depends on:** A8-1, A8-2, Q3, Q5. In today's spelling the pools stay a class map with a class
  `cast`, as now.

**A8-4. ECS-8.** M.
- **Builds:** signals with `connect` and `disconnect` (function-value `==`, `[CLO-1]`), `sort`,
  `sort_as`, `ctx`, owning groups, `Snapshot`, `SnapshotLoader`, `ContinuousLoader` over a generic
  archive (`std.ser`), `RuntimeView`, `Handle`.
- **Depends on:** A8-3. `Handle` needs a kept changeable link to the registry (`sharedref mut`, the
  R7 area). Signals follow decision 3.6.

**A8-5. Move `std.ecs` onto the 0.9.15 forms.** M.
- **Changes:**
  - `pools: Map[u64, owns dyn AnyPool]`;
  - `pool = ref mut cast this.pools[type_hash[T]()] as Pool[T]`;
  - results `-> ref mut T` that point into the registry;
  - `for e, ref mut a, ref mut b in reg.view_mut[(A, B)]():`.
- **Completes:** ECS-2's spelled form; ECS-5 (E3020 on a registry change inside a view loop, which
  follows from the general rules); the 3 stale design tests (A8-11) start to pass.
- **Depends on:**
  - `owns dyn` (ownership-types area);
  - a cast of a place, `[TYP-44]`/`[DSP-4]` (kind-test area, R14);
  - a method's result pointing into its object, LT-1 (regions area, R6);
  - `[BRW-18]` (loops area, R8);
  - A8-9.

**A8-6. Library renames that need no new model.** M.
- **Changes:**
  - `.copied()`/`.cloned()` become `.copy_each()`, with E1010 and `[STD-19]`'s help for the old
    names: `std/src/core.em:1237` and on; `ember_typeck/src/lib.rs:11147`, `33000-33014`, `45576`,
    `45767`;
  - `@fp(contract)` becomes `@fp(allow_fma)`: `ember_typeck/src/lib.rs:2077-2081`,
    `ember_types/src/lib.rs:3636`;
  - the `Volatile` `@safety` texts (`std/src/mem.em:26-40`).
- **Completes:** STD-3, STD-29, and STD-19's names.
- **Tests:** respell the files that use them in the same slice: STD-19 (about 15 files with E),
  `bench/programs/s4_copied_enumerate_skip.em`, and 3 of `tests/`.
- **Depends on:** nothing, except that `[CLI-20]`'s migration must know the rename.

**A8-7. STD-19, STD-26, STD-11: lending, sorting, maps of views.** M.
- **Changes:**
  - delete `lend_named_callable` (`ember_typeck/src/lib.rs:40811-41019`) and the calls through the
    lent reference (`47633`, `47646`, `56594`, `3243`), so a named function given to `owns f: F` is
    handed over;
  - delete the cloning in `items_array` (`32996-33020`) and route `sorted` over views to an `Array`
    of views;
  - replace the static-only store check `ember_analysis/src/stores.rs:435`.
- **Completes:** STD-19's hand-over, STD-26, STD-11's views.
- **Tests:** STD-19 rewrite 5, STD-26 rewrite 1, STD-11 rewrite 1.
- **Depends on:** the `owns` hand-over (ownership area); values and arrays holding views, `[TYP-15]`
  (regions area); a callback's effects counted at the consumer (effects area, R5/R12).

**A8-8. Collections API per STD-15 and STD-16.** M.
- **Changes (`std/src/collections.em`, `std/src/core.em`):**
  - `retain` callback types `fn(K, V)` and `fn(T)`;
  - a (key, `ref mut` value) map iterator for `for k, ref mut v in m:`;
  - `entry(k).or_default()` as a place;
  - `or_insert_with` and `into_items` with `owns this`;
  - Set operations returning copies.
- **Completes:** STD-15 and STD-16 (their E3020/E3022/E3060 parts come from the loops and effects
  areas).
- **Depends on:** loop names with `ref mut` and the comma rule (R3/R8 areas); A8-7.

**A8-9. Respell `std` (23 modules) and rewrite its 6 non-mechanical parts.** L.
- **Mechanical part:** `ember fmt --migrate` (`[CLI-20]`, CLI area) covers `self`→`this`,
  `owned`→`owns`, `mut x`→`ref mut x` with the call words, `str`→`StrView`, `static fn`, `@borrows`
  →`from`, `Box`→`owns`, `.clone()`→`copy of`, and the renames of A8-6.
- **By hand:**
  - `sync.em`: `SyncShared` becomes `thread_shared`;
  - `thread.em`, `jobs.em`: `thread_shared fn()`;
  - `coroutine.em`: the `owned fn` capture;
  - `ffi.em`: `ForeignBox` becomes `owns T`, with `into_raw` in `unsafe`;
  - `mem.em`: `forget` removed, the disjoint renames;
  - `core.em`: `AnyError` becomes `owns dyn Error`.
- **Completes:** spelling conformance of `std` for every rule above.
- **Tests:** the STD folders' 51 respell files and ECS-1's 1 move with it.
- **Depends on:** the new spellings parsed and checked (lexer, parser and grammar areas);
  `thread_shared`, `owns dyn`, `owns T` on foreign types (ownership, concurrency and FFI areas);
  `[CLI-20]` (CLI area); decision 3.1.

**A8-10. Respell and rewrite the programs outside `tests/conformance`.** L.
- **Respell:** 269 test files and 13 bench files, by migration.
- **Rewrite by hand:** 81 test files and 15 bench files. This includes 50 `ui` cases whose `.stderr`
  goldens are rewritten from the new message texts, and the 16 `assert-c` files.
- **Retire:** 4 `ui` files (B8, B14).
- **Also:** check the 151 `respell†` files by hand for a class handle copied with `=`; rename the
  p-set benchmarks (owner); update `bench/descriptions.py` and the README tables; rewrite
  `tests/perf/counted_sync_handles.em` to `thread_shared` (its C++ twin stays).
- **Depends on:** A8-9, and on each area whose rule a test exercises.

**A8-11. The 3 design tests written against the removed ECS API.** S.
- **Files:** `run4/v5/conformance/R4/reject_mut_in_a_query_is_now_ref_mut.em`,
  `R4/reject_ref_mut_on_a_query_item_the_query_only_reads.em`,
  `R8/accept_ecs_changes_are_queued_while_a_query_walks.em`.
- **Rewrite** them onto `Registry`/`view_mut` with the same rule meaning:
  - the R4 pair becomes `ref mut` on an item of a read-only `view`, E3027;
  - R8's becomes collect-then-change, E3020.
- **Needs:** the owner's sign-off, since these are the job's own tests.
- **Depends on:** A8-5.

**A8-12. Measure `std.ecs` against EnTT, speed and memory, on the laptop.** M.
- The design note's step 5, and the owner's rules: fast as C, prove, measure, try to break.
- Needs EnTT's headers installed. Ask the owner with the exact command, e.g.
  `vcpkg install entt` or `git clone https://github.com/skypjack/entt` into a location he picks.
- **Depends on:** A8-5.

### 2b. Slices (each leaves the compiler consistent and its tests green)

1. **A8-1 + A8-2 + A8-3**, once Q1, Q2, Q3 and Q5 are answered: the new ECS in today's spelling.
   `std.ecs` and the compiler's ECS support change in one commit, so nothing half-removed is left;
   E7020 is retired in the same commit.
2. **A8-4.**
3. **A8-6**, with the files it respells.
4. **A8-9 + A8-10 + A8-5** in one sweep, after the language areas land the new spellings. Migrate
   `std`, tests, bench, examples and the inline programs in `compiler/*/tests/*.rs` together, then
   refuse the old spellings (decision 3.1).
5. **A8-7 + A8-8**, with the regions and effects areas.
6. **A8-11, A8-12.**

## 3. Shared decisions

### 3.1 Spelling transition: both spellings parse until one migration commit, then the old ones are refused
`std` is compiled with every program, so the compiler cannot refuse `self` before `std` stops
using it, and `std` cannot use `this` before the compiler accepts it.

Proposal:
- each language area adds its new spelling **alongside** the old;
- one commit runs `[CLI-20]`'s migration over `std/`, `tests/`, `bench/`, `examples/` and the
  inline Rust-test programs, and does the by-hand rewrites;
- the next commit refuses the old spellings with `[CLI-20]`'s messages.

This is staging, not a deviation: the shipped compiler accepts only the 0.9.15 forms. Run-time
cost: none.

### 3.2 ECS pool layout (`Pool[T]`)
- **Dense part:** `dense: Array[T]` and `entities: Array[Entity]`, one contiguous array each, as
  `[ECS-2]` requires.
- **Sparse part:** I propose EnTT's **paged** sparse array (pages of 4,096 `u32` slots, each slot
  the dense index plus one, 0 for none), each page allocated when an index in it is first used.
  `[ECS-2]` forbids paging only the pool's values.
- **Memory:** 16 KB per touched page per component type. A flat sparse array costs up to 4 MB per
  type at 2^20 entities.
- **Speed:** a lookup (`contains`, `get`) costs a shift, the page load, the slot load, a bound
  check on the page count and a version compare against `entities[slot]`. That is EnTT's own
  sequence, one dependent load more than a flat array.

Measure both layouts in A8-12 and keep paging only if it costs under the owner's speed bar.
Today's `AnyStorage` is flat (`ecs.em:100-131`).

### 3.3 Reaching pools
- `pools: Map[u64, owns dyn AnyPool]` (`[ECS-2]`).
- A view looks up its pools **once**, when it is made: one hash probe and one type-index compare
  (the downcast check, `[DSP-4]`) per pool. Per entity it pays only the sparse probes of the other
  pools, as EnTT's view does.
- `reg.get[T](e)` pays the probe and the compare on every call, the cost of EnTT's
  `assure<T>()`. Hot loops use a view or a held `storage[T]()`.
- The compare cannot be proved away in general: only `storage[T]` inserts under `type_hash[T]`,
  but the compiler cannot see that across calls. It is one compare per view, not per entity.

### 3.4 Type identity
A general compiler-known `std.core.type_name[T]() -> StrView from static` gives the fully qualified
name, the same text `type_identity` (ERR-8) hashes today. `ecs.type_hash[T]()` is FNV-1a-64 over
it, evaluated when compiling: zero run-time cost. It is general (also usable by errors and
reflection), so the compiler keeps no ECS-specific support. Pending Q2.

### 3.5 `mem.forget` leaves the library surface, not the compiler
The `MemForget` builtin (`ember_hir/src/lib.rs:2063`) stays internal, as the body of `into_raw()`.
`into_raw()` is allowed only inside `unsafe` (R16). No runtime change.

### 3.6 Signals call listeners with no view open
`emplace`, `patch` and `destroy` finish the pool change, then take the listener list out of the
registry (`mem.take(ref mut this.signals…)`), call each `f(ref mut this, e)`, and put the list back.

This way no name points into `this` during the call, and `[BRW-21]` is met by construction, with
no run-time check. Cost: an `is_empty()` test when no one listens; otherwise two moves of a list
header per notification.

What happens to a listener connected during a notification is the library's call. EnTT's rule:
it is not called for that notification.

### 3.7 `TypeIsSend` and `TypeIsSync` go
Their only user is the ECS scheduler. The `Send`/`ThreadShared` checks of the concurrency area do
not need them.

## 4. Questions for the owner

1. **One-element tuple types.**
   - `[TYP-27]`: "Written as a type, `()` is `void`: a tuple type has two or more elements
     (ODR-071)."
   - `[ECS-4]` writes `.exclude[(C,)]()` and "implements its view interface for tuples of 1 to 8
     types". `[ECS-3]` writes `all_of[(A, B)](e)`, so one type is `all_of[(A,)]`.
   - `[GRM-38]` makes `(e,)` a tuple *value*, but no rule makes `(T,)` a *type*. Today
     `(A)` and `(A,)` both parse to the same one-item tuple (`ember_parser/src/decls.rs:1516-1527`).

   Blocks every one-type view. Should `(T,)` be a one-element tuple type, with `(T)` still `T`?

2. **Where `ecs.type_hash[T]()` gets the name.**
   - `[ECS-2]`: "A component type's identity is the 64-bit FNV-1a hash of its fully qualified name
     (`ecs.type_hash[T]()`)".
   - `[ECS-7]`: "Any type is a component, with no registration and no derive".
   - XII.3: "the compiler gives it no support of its own".

   Library code has no way to read a type's name: today's `type_hash` reads it from the derive's
   generated `extend`, and `std.core.type_identity` (ERR-8) gives a 128-bit hash, not the name or
   a 64-bit FNV. Proposal (decision 3.4): a general compiler-known `std.core.type_name[T]()`, which
   is not ECS support. Is that the intent, or should `type_hash` itself be compiler-known?

3. **`@component` is still in the attribute table.**
   - `p05-declarations.md:535` lists "`@component(layout=soa|aos)`, `@soa(flatten)` | struct;
     field | Part XII".
   - `[ECS-2]` no longer has SoA pools or `@component`, and `tasks/95/ecs-entt-design.md` lists
     `@component(layout=soa)` under "What goes".

   Remove the row, so `@component` is E0104? Blocks deleting `derives.rs:177-213` and decides
   ECS-2's `reject_a_component_layout_…` test.

4. **`Pool[T]` or `Storage[T]`.**
   - `[ECS-2]`: "`pool = ref mut cast this.pools[ecs.type_hash[T]()] as Pool[T]`".
   - `[ECS-3]`: "`storage[T]() -> ref mut Storage[T]`".

   Are these one type? `std.collections` already exports `Pool` and `Handle` (`[HND-1]`,
   `p15-stdlib.md:57`), and `[ECS-8]` adds an ECS `Handle`. Blocks the public names of the new
   module.

5. **Is `[CLI-20]`'s migration list complete?** (Not blocking for this area; it decides how much of
   A8-9 and A8-10 is by hand.) `[CLI-20]` rewrites "where the change is mechanical", then lists the
   changes from 0.9.10. The list leaves out four mechanical ones:
   - `[GRM-40]`'s bracket lines: 311 lines in `std`, 68 in tests;
   - `x: ref T` parameters, refused by E0117;
   - `T: Sync` bounds, which become `ThreadShared`;
   - `for v in xs.iter_mut():`, which becomes `for ref mut v in xs:`.

   Should `--migrate` do these too?

## 5. Risks

1. **Class objects now have one owner.**
   - 196 non-conformance test files and 12 benchmarks declare classes. Under 0.9.15, `x = obj`
     hands the object over, `t = things[i]` is a read-only view, and changing through it needs
     `ref mut`.
   - The scan marks 151 respell files `†`. Any of them may turn out to be a rewrite once read.
   - Size of the unknown: up to 151 files.
2. **New messages are part of the rules.** The owner's readability verdict and the code table give
   exact texts (E3020's "collect … in a list of your own …"). The 68 `ui` goldens and every
   `#$ error[...]: message` line in the respell files whose message text changed need new text.
   Code and line usually stay. I classified on code and line, so some `respell` compile-fail files
   will also need their message text changed.
3. **Stale design tests.** R4 (2) and R8 (1) import `World`, `Query`, `Component` and
   `@derive(Component)`, which 0.9.15 removes ([ECS-7], [ECS-9]). They cannot pass as written. A8-11
   proposes a rewrite with the same meaning; the owner should confirm.
4. **ECS-5 rests on three other areas.** The E3020 refusal inside a view loop exists only once a
   view's result is tied to the registry (regions area) and `[BRW-18]` (loops area) are built. Until
   then (slice 1), the new `Registry` in today's spelling relies on today's loop borrow. I did not
   verify that today's compiler refuses `reg.destroy(e)` inside a loop over a library iterator
   that borrows `reg`.
5. **The C++ importer writes old spellings.** `ember_driver/src/cimport.rs` generates `ForeignBox[...]`
   and `owned` (lines cited in 1b). Its generated text must switch to `owns T` together with
   `ffi.em` (FFI area). Generated overlays already on disk need regenerating.
6. **`sorted` over an `Array` is now a list of views** ([STD-26]: "copies none"). Programs that sort,
   then change the source, then use the sorted list, become E3060. I found none in the STD folders
   by hand, but did not search all 511 files.
7. **Std sweep size.** 3,092 lines with `self`, about 310 functions that become `static fn`, 311
   bracket lines. If `--migrate` is not complete when the sweep runs, the by-hand share grows
   (Q5).
8. **Performance is unmeasured.** The ECS layout (3.2, 3.3) follows EnTT but is not measured. The
   owner's bar needs A8-12 before any claim, and EnTT is not on this machine.
9. **Not verified:**
   - the scan's regexes; the `R` tag over-counts struct fields (section 0);
   - call sites that now need `ref mut` (E3028) or `const ref mut`, which the scan cannot see;
   - programs inside `compiler/*/tests/*.rs` and unit tests: only the ECS ones were searched
     (`milestones.rs`, `reload.rs`);
   - whether `[CLI-20]` migration exists yet (CLI area).

## 6. Per-file classification of the programs outside `tests/conformance`

Tags (the spellings found):
- `S` `self`→`this`; `O` `owned`→`owns`; `M` `mut x`→`ref mut x` plus the call word;
  `V` `str`→`StrView`; `G` brackets after the name (`[GRM-40]`); `I` `if Some(x) =`→`exists as`;
- `C` `.clone()`→`copy of`; `E` `.copied()`/`.cloned()`→`.copy_each()`; `B` `Box`→`owns`;
  `W` `Weak`/`upgrade`→`weakref`/`exists as`; `R` `x: ref T` (parameters only matter);
- `T` `for … .iter_mut()`→`for ref mut`; `D` `assert_disjoint` renames; `A` `@borrows`→`from`;
  `X` `cast … if`→`is … as`; `Y` `@static_safe`; `P` `@fp(contract)`; `K` `Sync` bound;
  `Fb` `ForeignBox`;
- `†` the file declares a class.

| file | class | spellings | why (rewrite/retire) |
|---|---|---|---|
| `bench/probes/channel_throughput.em` | rewrite |  | `owned fn` to `thread.spawn` (now `thread_shared fn()` with capture words) |
| `bench/probes/mutex_contended.em` | rewrite |  | `SyncShared` to `thread_shared`; `owned fn` |
| `bench/probes/sync_shared_copies.em` | rewrite |  | `SyncShared` copies to `thread_shared` owners (same cost model, atomic count) |
| `bench/probes/thread_spawn_join.em` | rewrite |  | `owned fn` to `thread.spawn` |
| `bench/programs/p1_read_loop.em` | rewrite† | S | class elements named with plain `=` (`t = things[i % 2]`) are now read-only views: changing them needs `ref mut`; names written for `mut self` (owner naming rule); p4 keeps views of `bags[2..5]` across changes of `bags[round % 2]`, which R8 (f) refuses unless proved different |
| `bench/programs/p2_view_held.em` | rewrite† | S | class elements named with plain `=` (`t = things[i % 2]`) are now read-only views: changing them needs `ref mut`; names written for `mut self` (owner naming rule); p4 keeps views of `bags[2..5]` across changes of `bags[round % 2]`, which R8 (f) refuses unless proved different |
| `bench/programs/p3_mut_self_1_field.em` | rewrite† | S | class elements named with plain `=` (`t = things[i % 2]`) are now read-only views: changing them needs `ref mut`; names written for `mut self` (owner naming rule); p4 keeps views of `bags[2..5]` across changes of `bags[round % 2]`, which R8 (f) refuses unless proved different |
| `bench/programs/p3_mut_self_6_fields.em` | rewrite† | S | class elements named with plain `=` (`t = things[i % 2]`) are now read-only views: changing them needs `ref mut`; names written for `mut self` (owner naming rule); p4 keeps views of `bags[2..5]` across changes of `bags[round % 2]`, which R8 (f) refuses unless proved different |
| `bench/programs/p4_views_alive_01.em` | rewrite† | S | class elements named with plain `=` (`t = things[i % 2]`) are now read-only views: changing them needs `ref mut`; names written for `mut self` (owner naming rule); p4 keeps views of `bags[2..5]` across changes of `bags[round % 2]`, which R8 (f) refuses unless proved different |
| `bench/programs/p4_views_alive_04.em` | rewrite† | S | class elements named with plain `=` (`t = things[i % 2]`) are now read-only views: changing them needs `ref mut`; names written for `mut self` (owner naming rule); p4 keeps views of `bags[2..5]` across changes of `bags[round % 2]`, which R8 (f) refuses unless proved different |
| `bench/programs/p4_views_alive_16.em` | rewrite† | S | class elements named with plain `=` (`t = things[i % 2]`) are now read-only views: changing them needs `ref mut`; names written for `mut self` (owner naming rule); p4 keeps views of `bags[2..5]` across changes of `bags[round % 2]`, which R8 (f) refuses unless proved different |
| `bench/programs/p5_million_objects.em` | rewrite† | S | class elements named with plain `=` (`t = things[i % 2]`) are now read-only views: changing them needs `ref mut`; names written for `mut self` (owner naming rule); p4 keeps views of `bags[2..5]` across changes of `bags[round % 2]`, which R8 (f) refuses unless proved different |
| `bench/programs/p6_million_objects_6_lists.em` | rewrite† | S | class elements named with plain `=` (`t = things[i % 2]`) are now read-only views: changing them needs `ref mut`; names written for `mut self` (owner naming rule); p4 keeps views of `bags[2..5]` across changes of `bags[round % 2]`, which R8 (f) refuses unless proved different |
| `bench/programs/p7_mut_self_calls_a_function.em` | rewrite† | S | class elements named with plain `=` (`t = things[i % 2]`) are now read-only views: changing them needs `ref mut`; names written for `mut self` (owner naming rule); p4 keeps views of `bags[2..5]` across changes of `bags[round % 2]`, which R8 (f) refuses unless proved different |
| `bench/programs/w1_sum_loop.em` | rewrite† | S | class elements named with plain `=` (`t = things[i % 2]`) are now read-only views: changing them needs `ref mut`; names written for `mut self` (owner naming rule); p4 keeps views of `bags[2..5]` across changes of `bags[round % 2]`, which R8 (f) refuses unless proved different |
| `tests/compile-fail/borrows_attribute.em` | rewrite | A,R | E2031 on `@borrows` misuse; `from` clause errors are E3062/E2118 |
| `tests/compile-fail/class_drop_self_escape_owned_capture.em` | rewrite† | S | `owned fn` (no migration, needs capture words); E3016 now names `this` |
| `tests/compile-fail/class_drop_self_escape_through_a_field_handle.em` | rewrite† | S | expects L3001 (retired: a loop of owners is E3032 when not proven opened again) |
| `tests/compile-fail/the_checks_after_type_checking_run_on_what_has_no_error.em` | rewrite | O | expects E3030 (retired, folded into E3013); `owned fn` needs capture words |
| `tests/compile-fail/weak_nonclass.em` | rewrite | W | `Weak(1)` E2020; `weakref` links only to an object or `shares` value: new code and text |
| `tests/compile-pass/class_final_virtual_warning.em` | rewrite† | S | expects W2111 (retired: `virtual` in a class that cannot be subclassed is E2112), so it becomes compile-fail |
| `tests/firstweek/04_event_bus.em` | rewrite† | O,S,V | `owned fn` x4 (no migration) |
| `tests/firstweek/helped/04_event_bus.em` | rewrite† | O,S,T,V | `owned fn` x4 (no migration) |
| `tests/perf/counted_sync_handles.em` | rewrite† | S | `@sync` class (no migration: `thread_shared` + `Atomic`/`guarded by`); its C++ twin stays |
| `tests/run-pass/a_lambda_in_a_generic_function_calls_through_a_projection.em` | rewrite | G | `owned fn` (no migration: `fn() owns ...`) |
| `tests/run-pass/class_field_inherited_mut_method.em` | rewrite† | S | `assert-c` on `ember_object_begin_write`, `ember_object_end_write`: 0.9.15 runs no exclusivity check while the program runs, so the assertion inverts |
| `tests/run-pass/class_field_mut_argument_access.em` | rewrite† | M | `assert-c` on `ember_field_begin_write`, `ember_object_begin_write`: 0.9.15 runs no exclusivity check while the program runs, so the assertion inverts |
| `tests/run-pass/class_field_mut_argument_indexed_object.em` | rewrite† | M | `assert-c` on `ember_field_begin_write`, `ember_field_end_write`: 0.9.15 runs no exclusivity check while the program runs, so the assertion inverts |
| `tests/run-pass/class_field_mut_method.em` | rewrite† | S | `assert-c` on `ember_object_begin_write`, `ember_object_end_write`: 0.9.15 runs no exclusivity check while the program runs, so the assertion inverts |
| `tests/run-pass/class_indexed_mut_argument_access.em` | rewrite† | M | `assert-c` on `ember_field_begin_write`, `ember_object_begin_write`: 0.9.15 runs no exclusivity check while the program runs, so the assertion inverts |
| `tests/run-pass/class_indexed_mut_method.em` | rewrite† | S | `assert-c` on `ember_object_begin_write`, `ember_object_end_write`: 0.9.15 runs no exclusivity check while the program runs, so the assertion inverts |
| `tests/run-pass/class_interface_handle_field_mut.em` | rewrite† | S | `assert-c` on `ember_object_begin_write`, `ember_object_end_write`: 0.9.15 runs no exclusivity check while the program runs, so the assertion inverts |
| `tests/run-pass/class_interface_handle_mut.em` | rewrite† | S | `assert-c` on `ember_object_begin_write`, `ember_object_end_write`: 0.9.15 runs no exclusivity check while the program runs, so the assertion inverts |
| `tests/run-pass/class_interface_handle_mut_param.em` | rewrite† | M,S | `assert-c` on `ember_object_begin_write`, `ember_object_end_write`: 0.9.15 runs no exclusivity check while the program runs, so the assertion inverts |
| `tests/run-pass/class_let_field_mutate_through.em` | rewrite† | S | `assert-c` on `ember_object_begin_write`, `ember_object_end_write`: 0.9.15 runs no exclusivity check while the program runs, so the assertion inverts |
| `tests/run-pass/class_mut_method_access.em` | rewrite† | S | `assert-c` on `ember_object_begin_write`, `ember_object_end_write`: 0.9.15 runs no exclusivity check while the program runs, so the assertion inverts |
| `tests/run-pass/class_mut_method_access_kept.em` | rewrite† | S | `assert-c` on `ember_object_begin_write`, `ember_object_end_write`: 0.9.15 runs no exclusivity check while the program runs, so the assertion inverts |
| `tests/run-pass/class_virtual_dispatch.em` | rewrite† | S | expects W2111 (retired, now E2112): write `open class` or drop `virtual` |
| `tests/run-pass/dyn_interface_box_struct.em` | rewrite | B,S | `assert-c` on `ember_box_new_copy`, the `Box` lowering: stays only if `owns dyn` keeps it (the `owns` representation, another area); recheck |
| `tests/run-pass/dyn_interface_multi_bound_class_box.em` | rewrite† | B,S | `assert-c` on `ember_box_new_copy`, the `Box` lowering: stays only if `owns dyn` keeps it (the `owns` representation, another area); recheck |
| `tests/run-pass/generic_enum_dynamic_box_imported.em` | rewrite | B | `assert-c` on `ember_box_new_copy`, the `Box` lowering: stays only if `owns dyn` keeps it (the `owns` representation, another area); recheck |
| `tests/run-pass/generic_enum_generic_interface_default_drop_imported.em` | rewrite | B | `assert-c` on `ember_box_new_copy`, the `Box` lowering: stays only if `owns dyn` keeps it (the `owns` representation, another area); recheck |
| `tests/run-pass/known_callable_values_called_directly.em` | rewrite |  | `owned fn` (no migration) |
| `tests/run-pass/recursive_types_through_array.em` | rewrite† | S | expects L3001 (retired; E3032 or proof the loop is opened) |
| `tests/run-pass/shared_construction_and_get.em` | rewrite | S | `Shared` (no migration: `shares` or `shares mut`), `.get()` gone; `assert-c` on `ember_obj_new_copy`/`ember_retain` names tied to the old header |
| `tests/run-pass/weak_shared_handle.em` | rewrite | W | `Shared` (no migration), `Weak`+`upgrade()` to `weakref`+`exists as` |
| `tests/ui/borrow/A1/arena_needs_drop.em` | rewrite |  | `.stderr` golden E3090: the message text the 0.9.15 code table and verdicts give |
| `tests/ui/borrow/A1/arena_needs_drop.fixed.em` | rewrite |  | `.stderr` golden E3090: the message text the 0.9.15 code table and verdicts give |
| `tests/ui/borrow/B1/two_mutable_indices.em` | rewrite | R | `.stderr` golden E3022: the message text the 0.9.15 code table and verdicts give |
| `tests/ui/borrow/B1/two_mutable_indices.fixed.em` | rewrite | R | `.stderr` golden E3022: the message text the 0.9.15 code table and verdicts give |
| `tests/ui/borrow/B10/mut_argument_needs_place.em` | rewrite | M | `.stderr` golden E3027: the message text the 0.9.15 code table and verdicts give |
| `tests/ui/borrow/B10/mut_argument_needs_place.fixed.em` | rewrite | M | `.stderr` golden E3027: the message text the 0.9.15 code table and verdicts give |
| `tests/ui/borrow/B11/references_have_no_range.em` | rewrite | D | `.stderr` golden E3095: the message text the 0.9.15 code table and verdicts give |
| `tests/ui/borrow/B11/references_have_no_range.fixed.em` | rewrite | D | `.stderr` golden E3095: the message text the 0.9.15 code table and verdicts give |
| `tests/ui/borrow/B12/view_in_owning_container.em` | rewrite | V | `.stderr` golden E3063: the message text the 0.9.15 code table and verdicts give |
| `tests/ui/borrow/B12/view_in_owning_container.fixed.em` | rewrite |  | `.stderr` golden E3063: the message text the 0.9.15 code table and verdicts give |
| `tests/ui/borrow/B14/multi_region_result_provenance.em` | retire |  | E3065 retired: nothing is left to infer |
| `tests/ui/borrow/B14/multi_region_result_provenance.fixed.em` | retire |  | E3065 retired |
| `tests/ui/borrow/B15/callable_parameter_mode_mismatch.em` | rewrite | M | `.stderr` golden E2228: the message text the 0.9.15 code table and verdicts give |
| `tests/ui/borrow/B15/callable_parameter_mode_mismatch.fixed.em` | rewrite | M | `.stderr` golden E2228: the message text the 0.9.15 code table and verdicts give |
| `tests/ui/borrow/B2/mutate_while_iterating.em` | rewrite |  | `.stderr` golden E3020: the message text the 0.9.15 code table and verdicts give |
| `tests/ui/borrow/B2/mutate_while_iterating.fixed.em` | rewrite |  | `.stderr` golden E3020: the message text the 0.9.15 code table and verdicts give |
| `tests/ui/borrow/B3/shared_mutable_overlap.em` | rewrite | R | `.stderr` golden E3021: the message text the 0.9.15 code table and verdicts give |
| `tests/ui/borrow/B3/shared_mutable_overlap.fixed.em` | rewrite | R | `.stderr` golden E3021: the message text the 0.9.15 code table and verdicts give |
| `tests/ui/borrow/B4/borrowed_parameter_mutation.em` | rewrite |  | `.stderr` golden E3023: the message text the 0.9.15 code table and verdicts give |
| `tests/ui/borrow/B4/borrowed_parameter_mutation.fixed.em` | rewrite | M | `.stderr` golden E3023: the message text the 0.9.15 code table and verdicts give |
| `tests/ui/borrow/B5/a_struct_holding_its_owner_and_a_view.em` | rewrite | V | `.stderr` golden E3024: the message text the 0.9.15 code table and verdicts give |
| `tests/ui/borrow/B5/a_struct_holding_its_owner_and_a_view.fixed.em` | rewrite |  | `.stderr` golden E3024: the message text the 0.9.15 code table and verdicts give |
| `tests/ui/borrow/B6/return_wrong_parameter.em` | rewrite | A | `.stderr` golden E3062: the message text the 0.9.15 code table and verdicts give |
| `tests/ui/borrow/B6/return_wrong_parameter.fixed.em` | rewrite | A | `.stderr` golden E3062: the message text the 0.9.15 code table and verdicts give |
| `tests/ui/borrow/B7/borrow_outlives_value_parameter.em` | rewrite | O,V | `.stderr` golden E3060: the message text the 0.9.15 code table and verdicts give |
| `tests/ui/borrow/B7/borrow_outlives_value_parameter.fixed.em` | rewrite | V | `.stderr` golden E3060: the message text the 0.9.15 code table and verdicts give |
| `tests/ui/borrow/B8/method_takes_whole_self.em` | retire | R,S | E3025 retired: no method claims all of its object |
| `tests/ui/borrow/B8/method_takes_whole_self.fixed.em` | retire | R,S | E3025 retired |
| `tests/ui/borrow/B9/lambda_stored_past_its_capture.em` | rewrite |  | `.stderr` golden E3026: the message text the 0.9.15 code table and verdicts give |
| `tests/ui/borrow/B9/lambda_stored_past_its_capture.fixed.em` | rewrite |  | `.stderr` golden E3026: the message text the 0.9.15 code table and verdicts give |
| `tests/ui/borrow/O1/borrow_after_move.em` | rewrite | R | `.stderr` golden E3040: the message text the 0.9.15 code table and verdicts give |
| `tests/ui/borrow/O1/borrow_after_move.fixed.em` | rewrite | R | `.stderr` golden E3040: the message text the 0.9.15 code table and verdicts give |
| `tests/ui/borrow/O1/use_after_move.em` | rewrite | O | `.stderr` golden E3040: the message text the 0.9.15 code table and verdicts give |
| `tests/ui/borrow/O1/use_after_move.fixed.em` | rewrite |  | `.stderr` golden E3040: the message text the 0.9.15 code table and verdicts give |
| `tests/ui/borrow/O2/move_out_of_drop_field.em` | rewrite | S | `.stderr` golden E3010: the message text the 0.9.15 code table and verdicts give |
| `tests/ui/borrow/O2/move_out_of_drop_field.fixed.em` | rewrite | S | `.stderr` golden E3010: the message text the 0.9.15 code table and verdicts give |
| `tests/ui/borrow/O3/move_in_loop.em` | rewrite | O | `.stderr` golden E3041: the message text the 0.9.15 code table and verdicts give |
| `tests/ui/borrow/O3/move_in_loop.fixed.em` | rewrite | O | `.stderr` golden E3041: the message text the 0.9.15 code table and verdicts give |
| `tests/ui/borrow/O4/partial_move_then_whole_use.em` | rewrite | O,S | `.stderr` golden E3042: the message text the 0.9.15 code table and verdicts give |
| `tests/ui/borrow/O4/partial_move_then_whole_use.fixed.em` | rewrite | O,S | `.stderr` golden E3042: the message text the 0.9.15 code table and verdicts give |
| `tests/ui/borrow/O5/once_callable_required.em` | rewrite | O | `.stderr` golden E3030: the message text the 0.9.15 code table and verdicts give |
| `tests/ui/borrow/O5/once_callable_required.fixed.em` | rewrite | O | `.stderr` golden E3030: the message text the 0.9.15 code table and verdicts give |
| `tests/ui/borrow/O6/borrow_of_a_maybe_unassigned_local.em` | rewrite | R | `.stderr` golden E3050: the message text the 0.9.15 code table and verdicts give |
| `tests/ui/borrow/O6/borrow_of_a_maybe_unassigned_local.fixed.em` | rewrite | R | `.stderr` golden E3050: the message text the 0.9.15 code table and verdicts give |
| `tests/ui/borrow/O7/explicit_drop_call.em` | rewrite | S | `.stderr` golden E3070: the message text the 0.9.15 code table and verdicts give |
| `tests/ui/borrow/O7/explicit_drop_call.fixed.em` | rewrite | S | `.stderr` golden E3070: the message text the 0.9.15 code table and verdicts give |
| `tests/ui/borrow/O8/a_guard_forgotten.em` | rewrite | S | `mem.forget` gone (E1010 with the spec text); `.stderr` golden rewritten |
| `tests/ui/borrow/O8/a_guard_forgotten.fixed.em` | rewrite | S | `.stderr` golden E3015: the message text the 0.9.15 code table and verdicts give |
| `tests/ui/borrow/O9/class_drop_publishes_self.em` | rewrite† | S | `.stderr` golden E3016: the message text the 0.9.15 code table and verdicts give |
| `tests/ui/borrow/O9/class_drop_publishes_self.fixed.em` | rewrite† | S | `.stderr` golden E3016: the message text the 0.9.15 code table and verdicts give |
| `tests/ui/borrow/X1/push_while_a_view_of_the_field_lives.em` | rewrite† |  | `.stderr` golden E3080: the message text the 0.9.15 code table and verdicts give |
| `tests/ui/borrow/X1/push_while_a_view_of_the_field_lives.fixed.em` | rewrite† |  | `.stderr` golden E3080: the message text the 0.9.15 code table and verdicts give |
| `tests/ui/python/P22/interface_as_a_parameter_type.em` | rewrite | S | `.stderr` golden E2173: the message text the 0.9.15 code table and verdicts give |
| `tests/ui/python/P22/interface_as_a_parameter_type.fixed.em` | rewrite | G,S | `.stderr` golden E2173: the message text the 0.9.15 code table and verdicts give |

**Respell** (only spellings `ember fmt --migrate` or a hand rename changes; † declares a class, checked by hand for a handle copied with `=`):

| file | spellings |
|---|---|
| `bench/probes/disjoint_views_blend.em` | D,M |
| `bench/probes/display_format.em` | M,S |
| `bench/probes/simd_scale.em` | M |
| `bench/probes/soa_integrate.em` | M |
| `bench/probes/text_loop_repeated_text.em` | V |
| `bench/probes/text_loop_scripts.em` | V |
| `bench/probes/text_loop_varied_text.em` | V |
| `bench/programs/a05_structs.em` | T |
| `bench/programs/a07_virtual_calls.em`† | S |
| `bench/programs/a09_generics.em` | G |
| `bench/programs/s1_enumerate_xor.em` | T |
| `bench/programs/s2_zip_and.em` | T |
| `bench/programs/s4_copied_enumerate_skip.em` | E |
| `tests/compile-fail/a_returned_view_elision_cannot_tie.em` | A,O,R,S |
| `tests/compile-fail/a_view_struct_carries_its_region.em` | A,R |
| `tests/compile-fail/aliasing_xor_mutability.em` | R |
| `tests/compile-fail/borrow_and_use_after_move.em` | O,R |
| `tests/compile-fail/borrows_travel_with_the_reference.em` | R |
| `tests/compile-fail/class_construct_fields_not_ready.em`† | S |
| `tests/compile-fail/class_construct_init_for_partial.em`† | S |
| `tests/compile-fail/class_construct_init_loop_else_read.em`† | S |
| `tests/compile-fail/class_construct_init_match_partial.em`† | S |
| `tests/compile-fail/class_construct_init_partial_if.em`† | S |
| `tests/compile-fail/class_construct_init_read_before.em`† | S |
| `tests/compile-fail/class_construct_init_while_partial.em`† | S |
| `tests/compile-fail/class_construct_init_whole_self_before.em`† | S |
| `tests/compile-fail/class_downcast_unrelated.em`† | X |
| `tests/compile-fail/class_drop_self_escape.em`† | S |
| `tests/compile-fail/class_drop_self_escape_field.em`† | S |
| `tests/compile-fail/class_extend_override_nonvirtual.em`† | S |
| `tests/compile-fail/class_inherited_field_before_super.em`† | S |
| `tests/compile-fail/class_inherited_missing_super.em`† | S |
| `tests/compile-fail/class_inherited_super_twice.em`† | S |
| `tests/compile-fail/class_interface_handle_mut_borrowed.em`† | S |
| `tests/compile-fail/class_let_field_reassign.em`† | S |
| `tests/compile-fail/class_missing_abstract_override.em`† | S |
| `tests/compile-fail/class_named_init_argument_errors.em`† | S |
| `tests/compile-fail/class_override_nonvirtual.em`† | S |
| `tests/compile-fail/generic_bounds.em` | G,S |
| `tests/compile-fail/generic_class_bounded_interface_extension_unsatisfied.em`† | G,R,S |
| `tests/compile-fail/generic_class_downcast_mismatched_args.em`† | G,X |
| `tests/compile-fail/generic_class_identity_mismatched_args.em`† | G |
| `tests/compile-fail/generic_class_inherent_extension_override_nonvirtual.em`† | G,S |
| `tests/compile-fail/generic_class_let_field_reassign.em`† | G,S |
| `tests/compile-fail/generic_class_missing_abstract_override.em`† | G,S |
| `tests/compile-fail/generic_instances_are_named_as_written.em` | G |
| `tests/compile-fail/generic_struct_deferred_missing_interface.em` | G,S |
| `tests/compile-fail/interface_obligations.em` | M,S |
| `tests/compile-fail/v05_grammar_rules.em` | O |
| `tests/compile-pass/borrows_attribute_accepted.em` | A,R |
| `tests/compile-pass/class_construct_empty.em`† | S |
| `tests/compile-pass/class_type_header.em`† | O,S |
| `tests/compile-pass/class_virtual_override.em`† | S |
| `tests/compile-pass/dyn_interface_argument_modes.em` | M,R,S |
| `tests/compile-pass/dyn_interface_inherited_vtable_layout.em` | R,S |
| `tests/compile-pass/dyn_interface_method_call.em` | R,S |
| `tests/compile-pass/dyn_interface_mut_method_call.em` | R,S |
| `tests/compile-pass/dyn_interface_sized_default.em` | R,S |
| `tests/compile-pass/dyn_interface_type_formation.em` | R,S |
| `tests/compile-pass/view_struct_declared.em` | R |
| `tests/firstweek/01_text_adventure.em`† | I,S,V |
| `tests/firstweek/02_csv_summariser.em` | S,V |
| `tests/firstweek/03_scene_graph.em`† | I,O,S,V,W |
| `tests/firstweek/05_inventory.em`† | S,V |
| `tests/firstweek/06_path_finder.em`† | S,V |
| `tests/firstweek/07_todo_list.em`† | C,S,T,V |
| `tests/firstweek/08_word_frequencies.em` | V |
| `tests/firstweek/09_bank_account.em`† | O,S,V |
| `tests/firstweek/10_grade_book.em`† | O,S,V |
| `tests/firstweek/11_rpn_calculator.em`† | S,V |
| `tests/firstweek/12_tic_tac_toe.em` | I,S,V |
| `tests/firstweek/13_game_of_life.em` | S,V |
| `tests/firstweek/14_queue_simulation.em`† | O,S |
| `tests/firstweek/15_library_catalogue.em`† | O,S,V |
| `tests/firstweek/16_shapes_interface.em` | S |
| `tests/firstweek/17_linked_list.em`† | I,S |
| `tests/firstweek/18_traffic_light.em` | S |
| `tests/firstweek/19_tokenizer.em` | V |
| `tests/firstweek/20_matrix.em` | S |
| `tests/firstweek/21_particles.em`† | S,T |
| `tests/firstweek/22_undo_stack.em`† | I,S,V |
| `tests/firstweek/23_recipe_scaler.em` | S,V |
| `tests/firstweek/24_dice_game.em`† | I,M,O,S |
| `tests/firstweek/26_game_entities.em`† | O,S |
| `tests/firstweek/27_vending_machine.em`† | S,V |
| `tests/firstweek/28_hangman.em`† | S,V |
| `tests/firstweek/29_temperature_stats.em` | V |
| `tests/firstweek/30_binary_search_tree.em`† | I,M,O,S,V |
| `tests/firstweek/helped/01_text_adventure.em`† | C,I,S,V |
| `tests/firstweek/helped/02_csv_summariser.em` | S,V |
| `tests/firstweek/helped/03_scene_graph.em`† | C,I,O,S,V,W |
| `tests/firstweek/helped/05_inventory.em`† | C,S,V |
| `tests/firstweek/helped/06_path_finder.em`† | S,V |
| `tests/firstweek/helped/07_todo_list.em`† | C,S,T,V |
| `tests/firstweek/helped/08_word_frequencies.em` | C,V |
| `tests/firstweek/helped/09_bank_account.em`† | C,O,S,V |
| `tests/firstweek/helped/10_grade_book.em`† | C,O,S,V |
| `tests/firstweek/helped/11_rpn_calculator.em`† | S,V |
| `tests/firstweek/helped/12_tic_tac_toe.em` | I,S,V |
| `tests/firstweek/helped/13_game_of_life.em` | S,V |
| `tests/firstweek/helped/14_queue_simulation.em`† | O,S |
| `tests/firstweek/helped/15_library_catalogue.em`† | C,O,S,V |
| `tests/firstweek/helped/16_shapes_interface.em` | B,O,R,S |
| `tests/firstweek/helped/17_linked_list.em`† | I,S |
| `tests/firstweek/helped/18_traffic_light.em` | S |
| `tests/firstweek/helped/19_tokenizer.em` | V |
| `tests/firstweek/helped/20_matrix.em` | S |
| `tests/firstweek/helped/21_particles.em`† | S,T |
| `tests/firstweek/helped/22_undo_stack.em`† | C,I,S,V |
| `tests/firstweek/helped/23_recipe_scaler.em` | C,S,V |
| `tests/firstweek/helped/24_dice_game.em`† | I,M,O,S |
| `tests/firstweek/helped/25_prime_generators.em` | G,O |
| `tests/firstweek/helped/26_game_entities.em`† | O,S |
| `tests/firstweek/helped/27_vending_machine.em`† | C,S,V |
| `tests/firstweek/helped/28_hangman.em`† | S,V |
| `tests/firstweek/helped/29_temperature_stats.em` | C,V |
| `tests/firstweek/helped/30_binary_search_tree.em`† | C,I,M,O,S,V |
| `tests/perf/a05_structs.em` | T |
| `tests/perf/counted_handles.em`† | S |
| `tests/perf/noalloc_kernel.em` | M |
| `tests/perf/s1_enumerate_xor.em` | T |
| `tests/perf/s2_zip_and.em` | T |
| `tests/perf/s4_copied_enumerate_skip.em` | E |
| `tests/perf/soa_update.em` | M |
| `tests/perf/w1_sum_loop.em`† | S |
| `tests/run-pass/a_generic_struct_may_name_a_generic_class_declared_after_it.em`† | G,S |
| `tests/run-pass/a_generic_struct_with_a_projection_field_may_be_named_before_it_is_declared.em` | G,S |
| `tests/run-pass/a_generic_type_names_itself_in_its_fields.em`† | B,G,S,W |
| `tests/run-pass/a_list_that_leaves_or_recurses_gets_no_frame_buffer.em`† | S |
| `tests/run-pass/a_tuple_instance_takes_its_hidden_parameters_from_a_tuple_extension.em` | G,M,R,S,T |
| `tests/run-pass/a_tuple_of_parameters_implements_an_interface.em` | G |
| `tests/run-pass/abstract_class_virtual_dispatch.em`† | S |
| `tests/run-pass/an_extension_may_name_a_generic_subclass_before_its_base_is_collected.em`† | G,S |
| `tests/run-pass/an_iterator_default_in_a_generic_body_keeps_its_own_parameters.em` | G |
| `tests/run-pass/an_soa_in_a_class_field.em`† | G,O,S |
| `tests/run-pass/array_copy_struct_class_handle.em`† | M |
| `tests/run-pass/borrows_end_at_their_last_use.em` | R |
| `tests/run-pass/class_construct_drop.em`† | S |
| `tests/run-pass/class_construct_empty.em`† | S |
| `tests/run-pass/class_construct_fields.em`† | S |
| `tests/run-pass/class_construct_init.em`† | S |
| `tests/run-pass/class_construct_init_default_literals.em`† | S |
| `tests/run-pass/class_construct_init_drop.em`† | O,S |
| `tests/run-pass/class_construct_init_field_read.em`† | S |
| `tests/run-pass/class_construct_init_for_read.em`† | S |
| `tests/run-pass/class_construct_init_if.em`† | S |
| `tests/run-pass/class_construct_init_loop_else.em`† | S |
| `tests/run-pass/class_construct_init_match.em`† | S |
| `tests/run-pass/class_construct_init_while_read.em`† | S |
| `tests/run-pass/class_construct_init_whole_self_after.em`† | S |
| `tests/run-pass/class_construct_nonliteral_defaults.em`† | S |
| `tests/run-pass/class_downcast.em`† | S,X |
| `tests/run-pass/class_identity.em`† | S |
| `tests/run-pass/class_inherited_base_drop.em`† | S |
| `tests/run-pass/class_inherited_construct.em`† | S |
| `tests/run-pass/class_inherited_defaults.em`† | S |
| `tests/run-pass/class_inherited_drop.em`† | S |
| `tests/run-pass/class_inherited_mut_method.em`† | S |
| `tests/run-pass/class_interface_handle.em`† | S |
| `tests/run-pass/class_interface_handle_base_entries.em`† | S |
| `tests/run-pass/class_interface_handle_cache.em`† | S |
| `tests/run-pass/class_interface_handle_field_cache_mut.em`† | M,S |
| `tests/run-pass/class_interface_handle_inherited.em`† | S |
| `tests/run-pass/class_interface_handle_mut_param_cache.em`† | M,S |
| `tests/run-pass/class_named_init_construct.em`† | M,S |
| `tests/run-pass/class_virtual_dispatch_extend.em`† | S |
| `tests/run-pass/copied_and_cloned_over_a_bounded_parameter.em` | E,G |
| `tests/run-pass/derived_clone_class.em`† | C,S |
| `tests/run-pass/derived_clone_generic_class.em`† | C,G,S |
| `tests/run-pass/disjoint_field_borrows.em` | R |
| `tests/run-pass/dyn_interface_box_generic_struct.em` | B,G,R,S |
| `tests/run-pass/dyn_interface_box_generic_struct_default.em` | B,G,R,S |
| `tests/run-pass/dyn_interface_box_generic_struct_default_late.em` | B,G,S |
| `tests/run-pass/dyn_interface_box_struct_mut.em` | B,S |
| `tests/run-pass/dyn_interface_class_argument_modes.em`† | M,R,S |
| `tests/run-pass/dyn_interface_class_coercion.em`† | R,S |
| `tests/run-pass/dyn_interface_class_default_method_coercion.em`† | R,S |
| `tests/run-pass/dyn_interface_class_inherited_coercion.em`† | R,S |
| `tests/run-pass/dyn_interface_class_mut_coercion.em`† | R,S |
| `tests/run-pass/dyn_interface_class_sized_default_coercion.em`† | R,S |
| `tests/run-pass/dyn_interface_multi_bound.em` | R,S |
| `tests/run-pass/dyn_interface_multi_bound_box.em` | B,S |
| `tests/run-pass/dyn_interface_multi_bound_inherited.em` | R,S |
| `tests/run-pass/dyn_interface_struct_coercion.em` | R,S |
| `tests/run-pass/generators.em` | S |
| `tests/run-pass/generic_abstract_class_virtual.em`† | G,S |
| `tests/run-pass/generic_class.em`† | G |
| `tests/run-pass/generic_class_bounded_interface_extension.em`† | G,R,S |
| `tests/run-pass/generic_class_default.em`† | G |
| `tests/run-pass/generic_class_drop.em`† | G,S |
| `tests/run-pass/generic_class_generic_base.em`† | G,S |
| `tests/run-pass/generic_class_generic_base_chain.em`† | G,S |
| `tests/run-pass/generic_class_generic_base_drop.em`† | G,S |
| `tests/run-pass/generic_class_handle_field.em`† | G,S |
| `tests/run-pass/generic_class_identity_downcast.em`† | G,S,X |
| `tests/run-pass/generic_class_inherent_extension.em`† | G,S |
| `tests/run-pass/generic_class_inherent_extension_generic_method.em`† | G,S |
| `tests/run-pass/generic_class_inherent_extension_nested.em`† | G,S |
| `tests/run-pass/generic_class_inherent_extension_reordered.em`† | G,S |
| `tests/run-pass/generic_class_inherent_extension_virtual.em`† | G,S |
| `tests/run-pass/generic_class_inherited_base.em`† | G,S |
| `tests/run-pass/generic_class_inherited_base_drop.em`† | G,S |
| `tests/run-pass/generic_class_inherited_interface_defaults.em`† | B,G,R,S |
| `tests/run-pass/generic_class_init.em`† | G,S |
| `tests/run-pass/generic_class_interface_default_dyn_box.em`† | B,G,R,S |
| `tests/run-pass/generic_class_interface_default_generic_method.em`† | G,S |
| `tests/run-pass/generic_class_interface_extension.em`† | G,R,S |
| `tests/run-pass/generic_class_interface_extension_default.em`† | B,G,R,S |
| `tests/run-pass/generic_class_interface_extension_default_generic_method.em`† | G,S |
| `tests/run-pass/generic_class_interface_extension_imported.em` | R |
| `tests/run-pass/generic_class_interface_extension_support/model.em`† | G,S |
| `tests/run-pass/generic_class_interface_extension_support/traits.em` | S |
| `tests/run-pass/generic_class_interface_handle.em`† | G,S |
| `tests/run-pass/generic_class_interface_handle_mut_param.em`† | G,M,S |
| `tests/run-pass/generic_class_method.em`† | G,S |
| `tests/run-pass/generic_class_multi_interface.em`† | G,R,S |
| `tests/run-pass/generic_class_multi_interface_extension.em`† | B,G,R,S |
| `tests/run-pass/generic_class_mut_method.em`† | G,S |
| `tests/run-pass/generic_class_sized_interface_default.em`† | B,C,G,R,S |
| `tests/run-pass/generic_class_virtual_dispatch.em`† | G,S |
| `tests/run-pass/generic_class_virtual_mut_dispatch.em`† | G,S |
| `tests/run-pass/generic_enum_derived_clone_imported.em` | C |
| `tests/run-pass/generic_enum_derived_clone_noncopy_imported.em` | C |
| `tests/run-pass/generic_enum_dynamic_box_imported_support/model.em` | S |
| `tests/run-pass/generic_enum_dynamic_box_imported_support/traits.em` | S |
| `tests/run-pass/generic_enum_generic_inherited_defaults_imported.em` | B |
| `tests/run-pass/generic_enum_generic_inherited_defaults_imported_support/traits.em` | S |
| `tests/run-pass/generic_enum_generic_interface_default_drop_imported_support/model.em` | S |
| `tests/run-pass/generic_enum_generic_interface_default_drop_imported_support/traits.em` | S |
| `tests/run-pass/generic_enum_generic_interface_default_imported.em` | B |
| `tests/run-pass/generic_enum_generic_interface_default_imported_support/traits.em` | S |
| `tests/run-pass/generic_enum_generic_interface_imported.em` | B |
| `tests/run-pass/generic_enum_generic_interface_imported_support/model.em` | S |
| `tests/run-pass/generic_enum_generic_interface_imported_support/traits.em` | S |
| `tests/run-pass/generic_enum_inherited_defaults_imported.em` | B |
| `tests/run-pass/generic_enum_inherited_defaults_imported_support/traits.em` | S |
| `tests/run-pass/generic_enum_inherited_interface_imported.em` | B |
| `tests/run-pass/generic_enum_inherited_interface_imported_support/model.em` | S |
| `tests/run-pass/generic_enum_inherited_interface_imported_support/traits.em` | S |
| `tests/run-pass/generic_enum_interface_default_imported.em` | B |
| `tests/run-pass/generic_enum_interface_default_imported_support/traits.em` | S |
| `tests/run-pass/generic_enum_interface_imported.em` | G |
| `tests/run-pass/generic_enum_interface_imported_support/model.em` | S |
| `tests/run-pass/generic_enum_interface_imported_support/traits.em` | S |
| `tests/run-pass/generic_enum_method_imported_support/model.em` | G,S |
| `tests/run-pass/generic_enum_multi_interface_imported.em` | B |
| `tests/run-pass/generic_enum_multi_interface_imported_support/model.em` | S |
| `tests/run-pass/generic_enum_multi_interface_imported_support/traits.em` | S |
| `tests/run-pass/generic_enum_mut_generic_interface_default_imported.em` | R |
| `tests/run-pass/generic_enum_mut_generic_interface_default_imported_support/traits.em` | S |
| `tests/run-pass/generic_enum_mut_interface_imported.em` | B |
| `tests/run-pass/generic_enum_mut_interface_imported_support/model.em` | S |
| `tests/run-pass/generic_enum_mut_interface_imported_support/traits.em` | S |
| `tests/run-pass/generic_enum_ref_generic_interface_default_imported.em` | R |
| `tests/run-pass/generic_enum_ref_interface_imported.em` | R |
| `tests/run-pass/generic_enum_ref_mut_interface_imported.em` | R |
| `tests/run-pass/generic_struct_interface_default_generic_method.em` | G,S |
| `tests/run-pass/generic_structs.em` | G,M,S |
| `tests/run-pass/generics_and_iterators.em` | G,S |
| `tests/run-pass/methods_and_interfaces.em` | M,S |
| `tests/run-pass/modules/ffi_static_result.em` | R |
| `tests/run-pass/named_function_arguments.em` | G,M |
| `tests/run-pass/named_method_arguments.em` | G,S |
| `tests/run-pass/references.em` | M,R |
| `tests/run-pass/references_that_travel.em` | R |
| `tests/run-pass/returned_views.em` | A,R,S |
| `tests/run-pass/two_generic_structs_may_name_each_other.em` | G,R,S |
| `tests/run-pass/view_structs.em` | R |
| `tests/run-pass/views_passed_as_two_words.em` | M,V |
| `tests/run-pass/weak_class_field.em`† | W |
| `tests/run-pass/weak_class_handle.em`† | S,W |
| `tests/ui/borrow/S1/a_class_handle_in_a_loop.em`† | M,S,Y |
| `tests/ui/borrow/S1/a_class_handle_in_a_loop.fixed.em` | M,Y |
| `tests/ui/python/P11/with_open_as.em` | V |
| `tests/ui/python/P11/with_open_as.fixed.em` | V |
| `tests/ui/python/P14/self_left_out.em`† | S |
| `tests/ui/python/P14/self_left_out.fixed.em`† | S |
| `tests/ui/python/P2/def_declares_a_function.em` | V |
| `tests/ui/python/P2/def_declares_a_function.fixed.em` | V |
| `tests/ui/python/P25/len_of_a_split.fixed.em` | V |
| `tests/ui/python/P26/str_kept_as_a_string.em` | V |
| `tests/ui/python/P26/str_kept_as_a_string.fixed.em` | V |

**Keep** (no removed or renamed spelling found):

- `bench/probes/` (8): atomic_counter.em, generator_pipeline.em, mutex_uncontended.em, once_completed.em, parallel_sum.em, soa_retain.em, stepped_range_value.em, thread_scope_spawn.em
- `bench/programs/` (33): a00_empty.em, a01_int_math.em, a02_float_math.em, a03_strings.em, a04_map_int.em, a06_enum_match.em, a08_closures.em, a10_recursion.em, a11_dod_soa.em, a12_sort.em, a13_map_scrambled.em, a14_map_gap_1024.em, a15_map_gap_1048576.em, a16_map_text.em, b13_checked.em, b13_wrapped.em, b14_checked.em, b14_wrapped.em, b15_checked.em, b15_wrapped.em, b16_checked.em, b16_wrapped.em, s3_skip_step_take.em, s5_range_step_enumerate.em, t1_chars.em, t1b_str_loop.em, t2_bytes.em, t3_lines.em, t4_split.em, t5_words.em, t6_char_indices.em, w13_checked.em, w13_int_lanes.em
- `examples/` (1): hello.em
- `tests/compile-fail/` (18): a_local_assigned_on_only_one_branch.em, a_run_of_invalid_characters_is_one_error.em, class_construct_abstract.em†, class_identity_nonclass.em, class_inherits_final.em†, condition_must_be_bool.em, defer_and_labels.em, if_equals_offers_a_comparison.em, indexing_a_non_array.em, jump_as_an_operand.em, match_is_not_exhaustive.em, named_function_argument_errors.em, numeric_mismatch_names_the_cast.em, reading_an_uninitialised_local.em, type_argument_where_an_index_belongs.em, unsafe_required.em, use_after_move.em, views_may_not_be_stored.em
- `tests/compile-pass/` (1): unused_binding_lints.em
- `tests/firstweek/` (1): 25_prime_generators.em
- `tests/perf/` (16): a06_enum_match.em, a10_recursion.em, a11_dod_soa.em, b13_checked.em, b14_checked.em, b15_checked.em, b16_checked.em, ffi_direct_call.em, float_math.em, int_math.em, s3_skip_step_take.em, s5_range_step_enumerate.em, t1_chars.em, t2_bytes.em, t5_words.em, t6_char_indices.em
- `tests/run-fail/` (5): a_shift_past_the_width_panics.em, array_out_of_bounds.em, class_downcast_forced.em†, division_by_zero_panics.em, overflow_panics_under_the_debug_profile.em
- `tests/run-pass/` (32): a_class_object_is_written_through_a_loop_variable.em†, a_deep_expression_compiles_on_every_host.em, a_plain_and_condition_branches_on_each_operand.em, an_enum_discriminant_beyond_32_bits_keeps_its_value.em, an_inner_loops_iterator_starts_again_each_turn.em, an_interface_signature_may_name_a_map_instance.em, an_soa_of_a_struct_with_no_fields_is_a_count.em, arithmetic_and_control_flow.em, cell_receiver_is_evaluated_before_its_argument.em, class_construct_default_literals.em†, class_instantaneous_field_write.em†, class_named_memberwise_construct.em†, const_and_static.em, definite_initialisation.em, drops_and_moves.em, enums_and_match.em, ffi_counted_across_modules.em, ffi_static_result_across_modules.em, generic_enum_derived_copy_imported.em, generic_enum_method_imported.em, integer_semantics.em, jump_expressions.em, loops_and_defer.em, modules_across_files.em, raw_pointer_builtins_in_any_module.em, short_lists_start_in_the_frame.em, std_math.em, stdlib_types.em, struct_values.em, tuples_and_arrays.em, two_phase_borrows.em, unsafe_and_raw_pointers.em
- `tests/run-pass/generic_enum_derived_clone_imported_support/` (1): model.em
- `tests/run-pass/generic_enum_derived_clone_noncopy_imported_support/` (1): model.em
- `tests/run-pass/generic_enum_derived_copy_imported_support/` (1): model.em
- `tests/run-pass/generic_enum_generic_inherited_defaults_imported_support/` (1): model.em
- `tests/run-pass/generic_enum_generic_interface_default_imported_support/` (1): model.em
- `tests/run-pass/generic_enum_inherited_defaults_imported_support/` (1): model.em
- `tests/run-pass/generic_enum_interface_default_imported_support/` (1): model.em
- `tests/run-pass/generic_enum_mut_generic_interface_default_imported_support/` (1): model.em
- `tests/run-pass/modules/` (2): ffi_counted.em, util.em
- `tests/run-pass/modules/geom/` (1): shapes.em
- `tests/run-pass/raw_pointer_builtins_support/` (1): pointers.em
- `tests/ui/basic/E1032/` (3): grid.em, item_named_like_a_module.em, item_named_like_a_module.fixed.em
- `tests/ui/basic/N1/` (8): inherited_class_field_typo.em†, inherited_class_field_typo.fixed.em†, local_name_typo.em, local_name_typo.fixed.em, module_item_typo.em, module_item_typo.fixed.em, struct_field_typo.em, struct_field_typo.fixed.em
- `tests/ui/python/P1/` (2): true_and_false.em, true_and_false.fixed.em
- `tests/ui/python/P10/` (2): equals_none.em, equals_none.fixed.em
- `tests/ui/python/P12/` (2): raise_an_exception.em, raise_an_exception.fixed.em
- `tests/ui/python/P13/` (2): lambda.em, lambda.fixed.em
- `tests/ui/python/P15/` (2): integer_true_division.em, integer_true_division.fixed.em
- `tests/ui/python/P16/` (2): assignment_as_a_condition.em, assignment_as_a_condition.fixed.em
- `tests/ui/python/P17/` (2): case_in_a_match.em, case_in_a_match.fixed.em
- `tests/ui/python/P18/` (2): global.em, global.fixed.em
- `tests/ui/python/P19/` (2): star_args.em, star_args.fixed.em
- `tests/ui/python/P20/` (4): percent_format.em, percent_format.fixed.em, str_format.em, str_format.fixed.em
- `tests/ui/python/P21/` (2): err_of_a_message.em, err_of_a_message.fixed.em
- `tests/ui/python/P23/` (4): list_repetition.em, list_repetition.fixed.em, string_repetition.em, string_repetition.fixed.em
- `tests/ui/python/P24/` (2): single_quoted_string.em, single_quoted_string.fixed.em
- `tests/ui/python/P25/` (1): len_of_a_split.em
- `tests/ui/python/P27/` (2): char_against_a_string.em, char_against_a_string.fixed.em
- `tests/ui/python/P28/` (2): assignment_in_an_arrow_arm.em, assignment_in_an_arrow_arm.fixed.em
- `tests/ui/python/P29/` (2): index_a_string.em, index_a_string.fixed.em
- `tests/ui/python/P3/` (4): float_of_a_number.em, float_of_a_number.fixed.em, int_of_text.em, int_of_text.fixed.em
- `tests/ui/python/P30/` (2): index_a_tuple.em, index_a_tuple.fixed.em
- `tests/ui/python/P4/` (2): len_of_a_string.em, len_of_a_string.fixed.em
- `tests/ui/python/P5/` (2): max_of_a_list.em, max_of_a_list.fixed.em
- `tests/ui/python/P6/` (8): append.em, append.fixed.em, dict_get_with_a_default.em, dict_get_with_a_default.fixed.em, isdigit.em, isdigit.fixed.em, join_from_the_separator.em, join_from_the_separator.fixed.em
- `tests/ui/python/P7/` (2): slice.em, slice.fixed.em
- `tests/ui/python/P8/` (2): truthiness_of_a_list.em, truthiness_of_a_list.fixed.em
- `tests/ui/python/P9/` (2): let_declares.em, let_declares.fixed.em
