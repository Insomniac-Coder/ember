# Job A — hot reload (Annex B): notes

Helper agent A, 2026-10-18. Branch `worktree-agent-a41c3ad1367298014`. Everything below is built,
tested on Windows (MSVC) and on Linux (gcc, in WSL), and committed on the branch.

## 1. What was built, feature by feature

### A1. The build mode (`[MAN-7]`, `[PRF-2]`, `[HR-25]`, `[HR-29]`)

* `[build] reload = "all" | "opt-in" | "bodies" | "none"` in `ember.toml`; any other value is `E9031`.
  `@reloadable` and `@noreload` apply to modules (`#! reloadable`) and items; the item wins, then
  its owner type, then the module, then the mode (`compiler/ember_driver/src/reload.rs`, `plan`).
* `shipping` refuses a manifest that sets `reload` (`E9033`). Decision: refuse rather than ignore,
  because a shipping build that silently drops the key would ship something the developer never ran.
  This is owner question Q2.
* `runtime = "static"` with reload is `E9036`: one shared runtime serves every image (`[HR-29]`).
* `@inline` functions are not reloadable (called directly). Owner question Q3.

### A2. Images, thunks and the shared runtime (`[HR-4]`..`[HR-9a]`, `[HR-8]`, `[HR-28]`)

* A reloadable build writes three things to `target/<profile>/bin`: the shared runtime
  (`ember_rt.dll` / `libember_rt.so`, the runtime compiled with `EMBER_RELOAD`, cached under the
  build cache by its source hash), the first image `<module>.image.0.<dll|so>`, and a toolchain host
  `<module>.exe` that calls `ember_reload_host_main`. `ember run` runs the host.
* `ember build --reload` builds the next image `<module>.image.N.<ext>` and announces it in
  `<module>.reload` (line 1 the image's file name, line 2 the build's time in ns). The host's next poll
  loads it. Images are never unloaded (`[HR-4]`).
* Windows: the runtime DLL's exports come from its object's COFF symbol table (a `.def` file), so no
  `__declspec` is needed in the runtime source. Images export nothing.
* Each image carries its manifest in a section (`.emrl` on PE, `ember_reload` on ELF), kept by
  `/INCLUDE:` on MSVC; the runtime reads the section, not the export table (`[HR-8]`). The manifest
  lists functions (mangled name, body, thunk, signature hash, body hash, flags), statics (name, type
  schema, initialiser hash, shape), type schemas, identities and the protocol version.
* A reloadable function's symbol is its thunk: one load from the image's `targets` table and one
  indirect jump; the body is `<symbol>__rl` (`[HR-5]`, `[HR-6]`). Reloadable bodies are never inlined,
  devirtualised or put in the inline header (`[HR-9a]`). Function values, closure code pointers,
  vtables and drop glue use the thunk's address, so they stay valid across reloads (`[HR-6a]`);
  function-value descriptors are made canonical across images.
* Classes and interfaces have identities compared by a canonical record per name, so a downcast or
  an interface lookup works on an object made by an older image.
* `ember inspect --safety` shows "Call: through its thunk (one load and one indirect jump)" or
  "Call: direct" per function (`[HR-28]`).
* `@noreload` functions are called directly and may be inlined; a change to one's body refuses the
  reload, naming it (`[HR-10]`). `L2005` warns on a `@noreload` loop calling a reloadable function
  (`[LNT-5]`).

### A3. The protocol and the host contract (`[HR-2]`, `[HR-3]`, `[HR-30]`..`[HR-32]`, `[HR-42]`, `[ABI-2]`)

* Host contract: `ember_reload_init(&config)`, `ember_reload_poll()`, `ember_reload_stats()`,
  `ember_reload_shutdown()`, `ember_reload_export(name)` (an export's permanent thunk).
* `[ABI-1]`/`[ABI-2]`: the protocol number is in linked symbol names (`ember_reload_init_v1`,
  `ember_reload_protocol_v1`) and in the manifest; an image of another protocol is refused on load
  with `E9037`.
* Every thread that runs Ember code is registered (host attach, `thread.spawn`, job workers) with a
  depth counter: entry is one relaxed increment and one acquire fence, exit one release decrement
  (`[HR-42a]`); a poll applies only when every counter is zero (`[HR-3]`); commit publishes with a
  release store of a generation counter (`[HR-42]`). Debug builds panic on entry from an
  unregistered thread (`[HR-3a]`).
* The transaction: plan (schemas only, every refusal decided here, `[HR-15b]`), prepare (new
  instances and statics in the transaction arena, 256 MB budget, under a `setjmp` guard that turns a
  panic into `ReloadError.Migration` and a budget overrun into `ReloadError.Allocation`), commit
  (pointer stores only), reclaim (old storage and removed fields dropped). A refusal leaves the old
  image running with nothing changed and a report naming each cause (`[HR-2]`, `[HR-18]`, `[HR-18a]`).
* `ember_reload_stats()` gives compile, load, plan, prepare, commit and reclaim times, instances
  migrated and the report (`[HR-32]`).

### A4. Schemas and migration (`[HR-11]`..`[HR-17a]`, `[HR-34]`..`[HR-38]`)

* Each package class, struct and enum has a schema: fields (name, `@renamed_from`, type, type schema
  hash, offset, size, what it holds) or variants (name, discriminant, payload hash and, for an enum
  with payloads, its payload fields). Standard types that hold the package's types (`Mutex[...]`,
  `Map`'s `Option[MapSlot[K, V]]` entries) get schemas from the package that uses them (`[HR-13a]`).
* Live instances are on a list in the 40-byte header of reloadable builds (`[HR-12]`); the header
  layout is a linked symbol, and an image built for the other layout is `E9035` (`[HR-12a]`).
* Migration by name (`[HR-14]`): default, zero for built-in `Default` types, `@renamed_from`,
  lossless widening, nested structs depth-first, reordered fields, unit-enum variants by name
  (`[HR-11a]`), methods-only changes retype instances in place; refusals for a field with no
  default, a changed type without `migrate_from`, a base change, a class removed while instances
  live (with the count), a removed variant a value holds (with the variant and the count), a changed
  payload enum, value data in containers (`[HR-13]`), and types a raw pointer reaches (`[HR-13b]`).
* An instance keeps its address when its new size fits; otherwise every reference the runtime can
  enumerate (strong and `Weak` handles in live instances, statics, `Array`s, structs and payload
  enums) is counted against the object's counts in plan and rewritten in commit; if any is missing
  the reload is refused, saying how many (`[HR-15]`, `[HR-15a]`).
* `migrate_from(old: ref OldSelf) -> Result[Self, ReloadError]` (`[HR-16]`): the driver writes
  `OldSelf` as a struct of the previous build's fields (from `<module>.schema.json`), under a helper
  name no source spells, so it is visible only as `OldSelf`; a wrapper turns `ReloadError` into the
  report. `panic`, `unwrap`, `expect`, `assert` in it are `E2225` with the fallible form in the help
  (`[HR-35]`); run-time checks inside it become `ReloadError.Migration` through prepare's guard.
* `std.hot.ReloadError` has the five variants (`[HR-38]`); `std.hot.checkpoint()` is the program's
  own poll.
* Statics live in the runtime's table (`[HR-4a]`); an unchanged static keeps its value, a struct or
  unit-enum static whose type changed migrates, an edited initialiser is refused unless
  `@reinit_on_reload` (`[HR-17]`, `[HR-17a]`).

### A5. Tools (`[HR-18b]`, `[HR-19]`, `[HR-33]`, `[BEN-8]`)

* `ember build --reload --explain` writes the image as `<module>.explain.N`, asks the running host
  (a request file beside the announcement), and prints "the reload would apply" or "would be
  refused" with the report; nothing is applied (`[HR-18b]`).
* `reload = "bodies"`: no schemas, the 24-byte header; body changes apply, anything needing instance
  migration is refused (`[HR-19]`).
* `ember run --hot` builds under `debug`, runs the program with unbuffered output, watches the
  package's sources, rebuilds in the background (a failed build changes nothing) and the program
  applies at `std.hot.checkpoint()`; a program with no checkpoint in ten seconds is told so
  (`[HR-31]`, `[HR-33]`).
* The reload benchmark (`[BEN-8]`): `the_reload_benchmark_times_three_edits` runs a C host over a
  program holding 10,000 live instances and times three edits from the save to the first frame
  running the new code. Windows, debug, this machine, 2026-10-18 (a tiny package, so the build
  dominates):

  | Edit | Save to running | Build | Load | Plan | Prepare | Commit | Reclaim | Migrated |
  |---|---:|---:|---:|---:|---:|---:|---:|---:|
  | a function body | 440 ms | 398 ms | 15.7 ms | 0.7 ms | 0.0 ms | 0.0 ms | 0.0 ms | 0 |
  | a field added with a default | 401 ms | 364 ms | 15.2 ms | 3.2 ms | 1.2 ms | 0.2 ms | 0.4 ms | 10,000 |
  | a field retyped through `migrate_from` | 428 ms | 389 ms | 15.5 ms | 2.3 ms | 3.7 ms | 0.0 ms | 0.4 ms | 10,000 |

### A6. Decisions worth recording (ADR-like)

1. **Thunks in C, not patched code.** A thunk is an ordinary C function loading its target from a
   per-image table the runtime fills. Portable to every C compiler, no writable code pages; the cost
   is the one load and indirect jump `[HR-9]` names.
2. **The runtime reads its own section, not exports.** Images export nothing, so two images never
   clash in the loader, and the manifest's layout is checked by size and protocol on load.
3. **Plan before prepare, values counted in plan.** Moving an instance needs every reference found;
   plan walks the roots once and compares what it found with the object's strong and weak counts,
   so the refusal is decided before anything is built (`[HR-15b]`).
4. **The guard is `setjmp`.** Prepare runs user code (defaults, `migrate_from`); a panic there
   `longjmp`s back to prepare, which discards the arena. Commit never runs user code.
5. **Checkpoints refuse what old frames could see.** At `std.hot.checkpoint()` the caller's frames
   are live and run old code, so a checkpoint applies body changes but refuses layout changes,
   signature changes and removed functions; a host poll (no frame live) applies all of them.
6. **`Old<Name>` is generated by the driver as source**, from the previous build's schema file, so
   the type checker checks `migrate_from` like any function.

## 2. Rules done, with their tests

Rust tests are in `compiler/ember_driver/tests/reload.rs` (R) and `compiler/ember_build/src/reload.rs`
(B, with the C images in `runtime/ember_rt/templates/tests/reload_image_*.c.in`).

| Rule | Tests |
|---|---|
| ABI-2 | B `the_reload_protocol_from_a_c_host` |
| BEN-8 | R `the_reload_benchmark_times_three_edits` |
| HR-2 | B `the_reload_protocol_from_a_c_host`; R `a_failing_migration_is_reported_not_fatal`, `the_migration_table_by_name` |
| HR-3, HR-3a, HR-42 | B `the_reload_protocol_from_a_c_host` (incl. mode `unregistered`); R `a_running_thread_holds_the_reload_back` |
| HR-42a | B `entering_ember_code_is_one_increment_and_one_fence` |
| HR-4, HR-4a | B protocol test; R `function_values_and_statics_outlive_their_image`; `tests/conformance/HR-4a/` |
| HR-5, HR-6, HR-8, HR-9a | `tests/conformance/HR-5/accept_a_call_goes_through_its_thunk.em`; R `an_edited_body_runs_after_the_next_checkpoint` |
| HR-6a | R `function_values_and_statics_outlive_their_image`, `objects_made_before_a_reload_run_the_new_methods` |
| HR-7 | B protocol test; R `removing_a_function_whose_address_is_held_is_refused` |
| HR-10, HR-10a | `tests/conformance/HR-10/`; R `a_noreload_body_change_is_refused_whole` |
| HR-11a | R `enum_values_migrate_by_variant_name`, `an_enum_static_keeps_its_variant` |
| HR-12, HR-12a | `tests/conformance/HR-12/`; R `an_image_for_the_other_header_is_e9035`, `removing_a_class_with_live_instances_is_refused` |
| HR-13a | R `a_field_added_with_a_default_migrates_live_instances`, `an_instance_moves_only_where_its_references_are_found` (the `Map` half) |
| HR-13b | R `a_type_behind_a_raw_pointer_does_not_migrate` |
| HR-15, HR-15a, HR-15b | R `an_instance_moves_only_where_its_references_are_found`, `weak_handles_follow_a_moved_instance` |
| HR-16 | R `migrate_from_builds_the_new_instances`; `tests/conformance/HR-35/` |
| HR-17, HR-17a | R `a_struct_static_migrates`, `an_enum_static_keeps_its_variant`, `a_static_initialiser_edit_needs_reinit_on_reload` |
| HR-18, HR-18a | R `a_noreload_body_change_is_refused_whole`, `the_migration_table_by_name` |
| HR-18b | R `explain_predicts_against_the_running_program` |
| HR-19 | `tests/conformance/HR-19/`; R `the_bodies_tier_admits_bodies_and_refuses_layouts` |
| HR-21 | R `exports_are_the_module_protocol` |
| HR-25 | `tests/conformance/HR-25/` (two tests) |
| HR-27 | R `shipping_is_the_same_with_reload_attributes` |
| HR-28 | R `inspect_safety_reports_the_thunk_indirection` |
| HR-29 | R `a_reloadable_package_shares_the_runtime` |
| HR-30, HR-32 | B protocol test; R the frame-host tests and the benchmark |
| HR-31, HR-33 | R `run_hot_applies_an_edit_at_the_next_checkpoint`, `run_hot_tells_a_program_without_a_checkpoint` |
| HR-34, HR-36, HR-37 | R `a_failing_migration_is_reported_not_fatal` |
| HR-35 | `tests/conformance/HR-35/` (accept and reject); R `a_failing_migration_is_reported_not_fatal` |
| HR-38 | `tests/conformance/HR-38/accept_reload_error_has_the_five_variants.em` |
| LNT-5 | `tests/conformance/LNT-5/accept_a_noreload_loop_calling_a_reloadable_function.em` |
| MAN-7, PRF-2 | R `reload_takes_four_values`, `reload_is_forbidden_in_shipping`; `tests/conformance/HR-25/` |

`tools/phase_progress.py` counts 50 of the sheet's 62 rules done (the sheet's 60 plus LNT-5 and FFI-39a).

## 3. What is left, and why

| Rule | State | What is missing |
|---|---|---|
| HR-2a | tested, `unfinished` entry | Reclaim drops a `migrate_from`'s old instance field by field without its class's own `drop`: that method's thunk now reaches the new image's body, built for the new layout (Q4). |
| HR-14 | tested, `unfinished` entry | Four rows are refused instead of built: a field added whose type is a user `Default` type, a change to a range type (`T.checked`), changed enum payloads, changed layout attributes (not in the schema). |
| HR-11 | untested | Schemas do not record layout attributes, and an interface is recorded only by its identity, with no schema (`EMBER_RL_TYPE_INTERFACE` is declared, never emitted). |
| HR-13 | untested | Value data in containers is not migrated through them: an `Array[Struct]` whose struct changed is refused; SoA columns, ECS storage and arenas have no registration (SoA/ECS are another job's). |
| HR-1, BUD-2 | untested | The one-second budget and B1-B9 are for `bench/bigpkg` (a generated 50k-line package) on the reference machine; neither exists yet. BEN-8's harness is the start of HR-1's measurement. |
| HR-9 | untested | The ≤ 3 % thunk overhead must be measured on the performance suite (`bench/`) with reload on; helper agents may not run programs outside the test harnesses, so the owner or coordinator should run it. |
| HR-20, HR-22 | untested | Need `Retained` tokens and `ForeignBox` (job B). |
| HR-24 | untested | GPU frames (post-1.0). |
| HR-23, HR-39, HR-43, FFI-39a | untested | C++ interop (Annex C) is not built; `@allow_reload_terminate` is `E0900`. |
| HR-41 | untested | The failure matrix's C++ and GPU rows depend on the above; the other rows are each tested. |
| ABI-1 | untested | The runtime ABI and the hot-reload protocol are linked symbols; the module protocol and export-table protocol are not versioned, and `ember_module_init` does not exist (with ABI-3, ADR-171). |

Known limits inside the done rules: references held in tuples, `Box` contents, closure captures and
`dyn` values are not walked, so an instance they hold that must move is refused (`[HR-15a]`); a
`migrate_from` type error names the struct by its helper name `__reload_Old_<Name>`.

## 4. Owner questions

* **Q1 — checkpoints and `[HR-3]`.** `[HR-3]` applies a reload only when no registered thread is in
  Ember code, but `[HR-33]` polls at `std.hot.checkpoint()`, inside Ember code. Built: a checkpoint
  allows its own thread's one level of entry and applies body changes only; layout changes, signature
  changes and removed functions are refused while old frames are live. Is that the intended reading?
* **Q2 — `reload` in a shipping build.** `E9033` refuses a manifest with `reload` under `shipping`.
  Should shipping instead ignore the key (`[HR-27]` says shipping is bit-identical with or without
  reload *attributes*, which is built)?
* **Q3 — `@inline` on a reloadable function.** `[HR-9a]` says a reloadable function is never inlined;
  built: `@inline` wins and makes the function not reloadable (called directly). Should it be an
  error instead?
* **Q4 — user `drop` of a `migrate_from`'s old instance (`[HR-2a]`).** After commit, the old class's
  `drop` thunk runs the new image's body, compiled for the new layout, so running it on the old
  instance is unsafe. Options: (a) as built, fields only; (b) emit a per-image adapter calling the old
  body directly (methods it calls on `self` still reach new code); (c) run old drops before commit,
  which breaks the transaction.
* **Q5 — `ReloadError` field types.** The spec names `SchemaRefused(type, reason)`,
  `Migration(type, instance, message)`, `Foreign(error)`; built as `String` for text and `u64` for
  `instance`. Right?
* **Q6 — what counts as a reference the runtime "can enumerate" (`[HR-15]`).** Tuples, `Box`
  contents and closure captures are not walked today (so a move is refused). Should they be required?

## 5. Defects found

* **D-681 (fixed, in `docs/DEFECTS.md`)**: a `@sync` class's `drop(self)` was called with the
  handle's address; gcc refused the C. Fixed in `emit_class_drop_adapters`; probe and test:
  `tests/conformance/CLS-8/accept_a_sync_class_drop_reads_its_fields.em`.
* Found and fixed in this job's own unmerged code (no ledger row): a struct static holding a removed
  enum variant would have been given the first variant (now refused, `an_enum_static_keeps_its_variant`);
  arrays of `Weak` handles were not walked (`weak_handles_follow_a_moved_instance`); a function
  removed at a checkpoint could be called by the old frames (now refused); a moved instance left a
  stale entry in the debug object registry.
