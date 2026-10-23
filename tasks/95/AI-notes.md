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
  reloadable programs among them; tests/conformance/*, tests/run-pass, tests/run-fail); with 1b's
  cases, **2143 of 2143**, and the whole tree under the C backend with the evaluator: 0 failing.
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
* **D-913 (both backends and the evaluator, fixed under 1b)**: a store into a class field holding a
  `Box[dyn I]` whose bracket is *not* back to back (the box made by a call: `h = Holder(Box(m))`,
  `keeper.keep = Some(Holder(Box(m)))`) began and ended its access through the box's table: the
  begin read the field before the store (an access violation when it is the constructor's first
  write, both backends; the compiler before this branch crashes the same way), the end the new
  box's object.

### The thunk of a function taking a `byval` argument (System V)

On Linux two reload tests crashed under LLVM (a reloadable function taking a struct of more than 16
bytes). The thunk forwarded its `byval` argument with `musttail`; LLVM lowers that, at `-O0`, by
copying the argument into the caller's argument area through `(%rsp)`, which at that point holds
the thunk's return address: the jump then returned into the copied bytes. Seen in the assembly of a
three-line IR file (`target/ai/byval_thunk.ll`, Clang 21.1.8): `musttail` gives the copy loop over
`(%rsp)`; `tail` gives `jmpq *%rax` alone, the argument left where the caller put it (a `byval`
argument is already the callee's stack copy). A thunk with a `byval` parameter is now a `tail` call;
every other thunk keeps `musttail` (Windows passes such a value by address, so it never has one).
With it the 50 reload tests pass on Linux under LLVM.

### Linux (WSL, Ubuntu, Clang 21.1.8, gcc)

* Reload tests: 50 of 50 under LLVM, 50 of 50 under the C backend (gcc).
* `HR-*` conformance under LLVM, every profile: 13 of 13.
* Every run test under LLVM: 2128 of 2129 before the fix below; the one failure was
  `FFI-8/accept_variadic_calls_and_va_list`, also failing on the base commit da5f4101 (built in WSL
  from `git archive`: a segmentation fault in every profile). **D-914 (LLVM, System V, fixed)**
  `[FFI-8]`: a `VaList` parameter had no representation the ABI classifier could read (8 bytes, no
  scalar inside), which System V classifies `NO_CLASS`: the parameter vanished from the signature
  (`define void @sink(ptr %p0, ptr %p1)`, C's `log_sink` takes three). Windows passed it as an
  `i64`, by luck the right register. A `va_list` parameter is a pointer on both x86-64 ABIs
  (Windows' `char*`; System V's array of one `__va_list_tag` decays to its address): `Types::repr`
  says so, and both targets pass a `ptr`, as Clang's IR of the same C does. With it: every run test
  under LLVM on Linux, **2143 of 2143**, and the 50 reload tests.

## 1b. The lead's ports, tried on both backends — DONE

`git diff 9a870998 HEAD -- compiler/ember_codegen_llvm` read through; each port tried with a
program built by the C backend (MSVC, with the evaluator agreeing) and by `EMBER_BACKEND=llvm` in
every profile, each kept as a conformance case (both backends pass every one):

| Port | Case |
| --- | --- |
| ODR-253/246 (`ffi.rs`): a nullable retained callback | `FFI-21/accept_a_nullable_retained_callback_keeps_an_argument_and_the_result` (C keeps an argument and a result across the call) |
| a newtype as its representation | `FFI-8/accept_a_newtype_in_a_struct_crosses_by_value` (a struct holding a newtype, by value both ways) |
| D-907 `Box[dyn]` of an interface requiring `Clone` and `Eq` | `TYP-22/accept_a_box_dyn_of_an_interface_requiring_clone_and_eq` (methods after the entries, a class concrete) |
| D-908 a boxed callable moved into its call | `CLO-6/accept_a_boxed_callable_moved_into_its_call_ends_once` (`Box[fn]` and `Box[once fn]`) |
| D-909/D-910 `Box[dyn]` lent | `TYP-22/accept_a_box_dyn_lends_its_value_as_a_ref_dyn` |
| D-900 print pieces | `STD-9/accept_a_held_print_formats_aggregates_and_runs_code` (aggregates and code inside one held statement) |

No difference between the backends in the ports. Trying them found **D-913**, in both backends and
the evaluator alike (above): fixed in `ember_analysis` (the EXC-18 access to the object behind a
`Box[dyn]` field is the field's place with a `Deref` after it: `behind_dyn`), the C backend
(`access_call`), the LLVM backend (`access`, `has_field_word`) and the evaluator (`object.rs`
`access`): a store into the field takes the field's own word; an access to the object behind it
takes the object's. Cases: `EXC-19/accept_a_dyn_field_is_stored_through_its_word` (the constructor's
first write of a box a call made; a replacement whose concrete is a class, then a struct) and
`EXC-18/run_fail_a_view_through_a_dyn_field_holds_the_object` (a view through the field holds every
field of the object until its last use).

ODR-276's question was settled here too, with cases (section 4 uses them):
`RC-5/accept_a_callee_ending_the_owner_a_borrow_points_into`,
`RC-5/accept_a_callee_replacing_the_field_a_borrow_went_through`,
`RC-5/accept_a_callee_dropping_one_owner_of_a_borrowed_shared`,
`HEAP-5/run_fail_a_callee_replacing_the_shared_a_borrow_points_into`,
`EXC-2/run_fail_a_callee_clearing_the_list_an_element_borrow_came_from`,
`EXC-2/run_fail_a_callee_replacing_the_list_an_element_borrow_came_from`.

## 2. Libraries under the LLVM backend (`cdylib`, `staticlib`, `[FFI-28]`, `[FFI-31]`) — DONE

### Design

* **What a library and a reloadable image share is built once.** `ember_build::cdylib::link_cdylib`
  takes the package's units (its C, or its IR and the IR's C shim), each compiled into an object
  named as an image's are (`unit_object`: `name.o`, `name.1.o`); the flags Clang needs for a `.ll`
  unit are one helper (`ir_flags`) every compile path takes (a program, an image, a library's
  object, an archive's). The driver's library tail is one function both backends call
  (`build_library`: a `cdylib`'s link with the export list, a `staticlib`'s archive beside the
  runtime's, the headers), the export list one function (`cdylib_exports`).
* **The module protocol in IR, as the C backend writes it:** `ember_module_protocol_v1` (1);
  `ember_module_init`, which compares `ember_rt_abi_version()` with the ABI it was built for before
  anything else (`EMBER_MODULE_INIT_ABI_MISMATCH`), returns 0 when already initialised, copies as
  much of the host's config as its `size` says over the defaults (`llvm.umin` + `memcpy`), sets
  `EMBER_RT_DEBUG_OBJECTS` in `debug`, calls `ember_rt_module_init_v3`, registers the leak check's
  edges in `debug`, and records the creator thread; `ember_module_shutdown`. An export with a
  `main`/`creator` contract checks the module's creator (`ember_rt_check_module_thread`), not the
  process's main thread. The config's layout and the protocol numbers are named once
  (`runtime::config`, `RUNTIME_ABI`, `MODULE_PROTOCOL`, `MODULE_INIT_ABI_MISMATCH`), and the unit
  test that compares the runtime table with the header now checks each of them against
  `ember_rt.h` (the program entry used the bare numbers 48, 40, 32, 1 and 3).
* **The header is the C backend's.** `ember_codegen_c::library_interface` runs the C emitter's type
  plan and class-method tables without writing C (`emit_with(…, interface_only)`, which returns
  before any body is emitted) and gives the header and the `@sync` classes' functions it declares
  (`ClassExport`, `MethodExport`, `Crossed`: how each parameter crosses). `VerifiedMir` is `Copy`
  (two shared references), so the LLVM build reads it twice. A test asserts the LLVM library's
  header equals `--emit header`'s byte for byte.
* **`[BLD-FFI-5a]` in IR** (`ffi::emit_class_exports`): a handle's retain and release, and each
  method: the thread attached, a kept handle retained, a view built from its address and length, a
  borrowed value passed by its address, the body called directly or through the object's method
  table (`virtual_call_values`).
* **A library's shims are named under its package's namespace** (`ember_ll_em_pkg_<hex>_str_eq`):
  the IR reaches the runtime's inline functions through one-line C functions, which were external
  symbols of the same names in every library. Two LLVM static libraries in one host failed to link
  (`lld-link: duplicate symbol: ember_ll_str_eq`, 12 of them; reproduced by the test below with the
  namespace switched off). The C backend's library has internal linkage throughout, so it never had
  this.

### Tests

* `ember_driver/tests/cdylib.rs`: 11 of 11 with `EMBER_BACKEND=llvm` and with the C backend. Its
  helpers pass `--backend` from `EMBER_BACKEND` (as the reload tests' do) and link the host with
  Clang under LLVM. `[ABI-3]` under LLVM: the module's IR with the ABI number its `module_init`
  checks moved on (the line carries `; EMBER_RUNTIME_ABI`), linked with this runtime, is refused
  first and leaves the runtime untouched. Under LLVM the relaxed-function test reads the fast-math
  flags in the IR (one module, no relaxed unit).
* `ember_driver/tests/staticlib.rs`: 6 of 6 under both backends, two of them new:
  `a_sync_class_crosses_a_static_librarys_exports` (a C host calls every crossing kind, a virtual
  method reaching the override, retain/release; under LLVM the header is the C backend's) and
  `static_libraries_calling_the_same_runtime_functions_link_side_by_side`.
* The IR of every run test without reload: identical to the base compiler's but for D-914's
  program (`format_list(…, i64)` became `ptr`), 1945 of 1946.

### Defects

* **D-915 (C header, both backends, fixed)** `[BLD-FFI-5a]`: a `@sync` class's method taking a
  C-laid-out struct by reference was declared with the struct's C name, but the header never
  declared the struct (its reachable-types walk visited exports and export tables, not the class
  methods): MSVC `C2146` in any host including it. The walk now visits each declared method's
  borrowed and by-value types and its result (`header.rs`). Test: `a_sync_class_crosses_a_static_
  librarys_exports` (`worlds_World_sum(World*, Pair)`).

### Spec question (for the owner)

`[FFI-28]` says a `cdylib` exports *only* its `@export` symbols, its tables, the two entry points and
the marker; `[BLD-FFI-5a]`'s header declares a `@sync` class's retain, release and methods, which a
`cdylib` therefore does not export (both backends, as the rule says): a host that loads the
library cannot call what its header declares. Options: (a) a `cdylib` also exports its classes'
functions (`[FFI-28]`'s list gains them); (b) a `cdylib`'s header leaves them out, so a `cdylib`'s
host gets handles it can only pass back; (c) as now. Recommendation: (a) — the functions exist for
the host, and they carry the package's name as their prefix, so they cannot clash.

## Merge notes for the lead

* Helper AE's branch changes the C backend's `reload.rs` too (the manifest's unit check,
  `reload_captures`, the layouts' tuple with `Option<ClassId>`): here those decisions moved into
  `ember_codegen_c::image` (`Planner`), which plans the layouts by class id already, so AE's change
  goes into the planner, and `reload.rs` keeps only the C text. AE's `frames.rs` change
  (`frames_on` for `Some(FpMode::Strict)`) applies to what is left of `frames.rs` as is.
* AE's `ember_driver::reload::build` takes units and ninja: here it takes a list of units already
  (an LLVM image is the `.ll` and its shim); a ninja rule for a `.ll` unit needs the IR flags
  (`ember_build::ir_flags`).
