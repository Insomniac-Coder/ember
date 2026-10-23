# Helper AI — the LLVM backend, the rest of phase 9, 2026-10-23

Branch `worktree-agent-acfe222401995f390`, from `main` at da5f4101 (after the lead's D-907..D-910 and
helper AH's merge). The lead merges.

## 1. Hot-reload images under the LLVM backend (`[HR-*]`, `[CG-LL-1]`) — DONE

### Design

* **What an image says of itself is decided once, for both backends.** The C backend's reload code
  (`reload.rs`, `frames.rs`) mixed the decisions (which types have schemas, what each field holds,
  which tables describe a tuple's or a closure's contents, each body's frame dependencies) with
  their C text. They now live in `ember_codegen_c::image` (public): a `Planner` decides and a
  `Writer` writes. The C backend's writer (`reload.rs`, `CWriter`) prints the C it printed before,
  **byte for byte**: every run test of the tree (1957 programs) built with `ember build --reload
  --emit c` in `debug` and in `development`, before and after: all identical but one, a
  nondeterminism the old compiler shows too (D-912 below, fixed). The LLVM backend's writer
  (`ember_codegen_llvm/src/reload.rs`, `LlvmWriter`) writes the same rows as IR constants, its
  numbers from the type table's layouts (`Num`: a size, an offset, an alignment, which the C writer
  prints as `sizeof`/`offsetof`/`_Alignof`). The runtime's record types are IR struct types, member
  for member (`RECORDS`; a unit test reads `ember_rt.h` and compares each, the magic, the protocol
  and every `EMBER_RL_*` constant's value: `hr_8_reload_records_are_the_headers`).
* **The image's own code, as the C backend writes it:** each reloadable body under `sym__rl`
  (internal), its thunk under `sym` loading the target from `em_rl_targets[i]` and jumping with
  `musttail` (one load and one indirect jump in every profile, `-O0` too, where the C backend relies
  on the C compiler making the tail call); the tables the runtime fills (`targets`, `addresses`,
  `descriptors`, `static_data`, `static_state`); a reloadable function's value through the
  runtime's descriptor (`em_rl_descriptors[i]`); lazy and constant statics, and `@reflect`
  descriptions, in the runtime's storage (`[HR-4a]`); the class identity records and the type
  information's last member (`identity`); interface identities as `ember_rl_identity` records
  listed in the manifest, and the `itable` lookup comparing their canonical records
  (`ember_same_identity`); a callable's descriptor with its `captures` table (`once` at 32);
  `reload_enter`/`reload_leave` around an export's body, a function value's C entry and a
  callback's trampoline (`[HR-42]`); the frame record of every body in an image whose instances
  migrate, the site stored before each call and popped at each return (`[HR-3b]`); the 40-byte
  object header (`Types::header`, `TypeTable::class_field_offsets_after`, which also lifts the old
  ceiling of a 32-byte-aligned field); no devirtualisation, no `inlinehint`, no split views for a
  reloadable function, `CppOverrides` true (`[HR-9a]`); the image's entry `em_rl_entry`; the
  manifest in `.emrl` (PE) / `ember_reload` (ELF), kept by `@llvm.used`. `std.hot.checkpoint()` was
  refused by this backend in every program; it is the runtime's `reload_checkpoint` now.
* **Driver:** `llvm.rs` builds an image from the IR and its C shim (compiled with the reload
  definitions, so the header's inline functions see the 40-byte header); the C path's tail (shared
  runtime, image link, host, announcement, `--explain`, the schema the next build reads) is one
  function both backends call (`build_reload_program`); `ember_build::reload::link_image` takes the
  image's units (it took one C file). `ember inspect`'s tables are written for an LLVM build too
  (`write_inspect`; `safety_json` is public), so `[HR-28]`'s report works on it.
* **Windows, Clang:** a shared library and what loads it now share one C runtime: Clang's driver
  takes `-fms-runtime-lib=dll` (the switch `msvc_runtime_switch()` names, as MSVC's `/MD`) for every
  unit and link of a reloadable program (runtime, images, host) and of a `cdylib`. Without it each
  DLL had a static C runtime of its own, so a foreign C object's `printf` in the image and the
  runtime's `println` wrote through two buffers (`a_foreign_object_ember_owns_is_carried_across` saw
  "widget 7 freed" first). The shared runtime's cache key is `v3` (a runtime built before is not
  reused).

### Tests

* Every run test under `EMBER_BACKEND=llvm EMBER_PROFILES=all`: **2129 of 2129** (the 11 refused
  reloadable programs among them; tests/conformance/*, tests/run-pass, tests/run-fail).
* `ember_driver/tests/reload.rs`, all **50** with `EMBER_BACKEND=llvm` and all 50 with the C backend.
  The helper `ember()` adds `--backend` from `EMBER_BACKEND` (the harness's variable) to a build or a
  run; `link_host` links the test's own host against the runtime the build placed (Clang's under
  LLVM). The tests that read the C read the IR under LLVM: the thunk is one load and one `musttail`
  jump (`a_thunk_is_one_load_and_one_indirect_jump`); every type has a schema, with the numbers
  (Mote: 72 bytes, `id` at 40, `x` at 56, `pair` at 64: the 40-byte header, `pair`'s access word
  and its pad; `every_type_has_a_schema_in_ir`); a release build's IR is the same with reload
  attributes (`[HR-27]`); an export's IR signature is the same under every protocol (`[ABI-5]`).
* The IR of a build without reload is byte-identical before and after (every run test, `debug` and
  `release`; the only difference the nondeterministic EXC-18 program, D-912).
* `ember_codegen_llvm` unit tests (`reload::tests`), `ember_codegen_c`, `ember_types`,
  `ember_build` (`link_image`'s own test now passes a list), `ember_driver --test llvm`.

### Defects found

* **D-911 (LLVM only, fixed)** `[EXC-19]`, `[EXC-18]`: the back-to-back check of a store into a class
  field holding a `Box[dyn I]` went through the box's table (`access_check` refused the merge for a
  `dyn` place), reading the box the store had not yet written: an access violation for
  `Holder(boxed)` in the constructor's field initialisation (the reload test
  `a_closure_holding_an_instance_follows_it_when_it_moves`, and without reload). The C backend's
  `field_check_call` checks the field's own word there; this backend now does the same.
* **D-912 (both, fixed)** `[BLD-*]` reproducible output: `EXC-18/accept_a_view_from_a_virtual_getter_
  holds_only_its_fields` gave two different C (and IR) texts on two runs: which of `Person.name` and
  `Nick.alias` a view's access began first came from a `HashMap` of virtual slots
  (`ember_analysis::borrows::VirtualSlots`), now a `BTreeMap`. 8 runs, one text.
* **D-913 (both, OPEN — see 1b)**: a store into a class field holding a `Box[dyn I]` whose bracket is
  *not* back to back (the box made by a call: `h = Holder(Box(m))`, `keeper.keep =
  Some(Holder(Box(m)))`) begins and ends its access through the box's table: the begin reads the
  field before the store (an access violation when it is the constructor's first write, both
  backends), the end the new box's object. Recorded under 1b with what it needs.
