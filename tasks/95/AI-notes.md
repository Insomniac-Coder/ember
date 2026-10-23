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

## 3. ODR-277: `nsw`/`nuw` from a proved-away check — MEASURED, NOT BUILT (no program changes)

* **What was measured.** An experiment, never committed: under `EMBER_LLVM_NSW`, the LLVM backend
  wrote `nsw` (signed) or `nuw` (unsigned) on each `add`/`sub`/`mul` whose overflow check a pass
  removed (the operation's span is a `RemovedCheck` of kind `Overflow`), which is the IR the MIR fact
  would give. The scripts are `target/ai/nsw_experiment.py on|off`, `nsw_count.py`, `nsw_asm.py`,
  `nsw_exe.py`.
* **What it marks.** 15 of `tests/perf`'s 21 programs get marked operations (1 to 10 each: e.g.
  `int_math` 5, `b15_checked` 10, `a10_recursion` 2), and 33 of `bench/programs`' 50.
* **Result: the machine code does not change.** Each program's release IR compiled by Clang 22 at
  `-O3` (the release flags) gives the same instructions with and without the flags, for all 50
  `bench/programs` and all 21 `tests/perf` programs (`b15_checked.plain.s` and `.nsw.s` are
  byte-identical files while their IR differs in ten `nsw`s). The release executables, linked with
  link-time optimisation by the build itself, are byte-identical too for all 21 `tests/perf`
  programs (the link is reproducible, `/Brepro`). Identical code cannot be faster, so no program
  gets faster beyond noise, and the MIR fact is not built (the brief's condition).
* **Why not.** The loops' induction variables are already 64-bit (no sign extension for `nsw` to
  remove), and where a check was proved away the range facts that proved it already bound the
  values LLVM reasons about. `a10_recursion`'s gap is the remaining check on `+` itself (ODR-277),
  which no flag on the other operations touches.
* **A timing round was taken and is not used.** One round of `ember bench --compare --backend llvm
  --core 1` per arm (`target/ai/bench_*_0.txt`): the `nsw` round ran while the machine was loaded by
  other work (CPU 37 % before the run; `s2_zip_and`'s Ember time 1003 ms against 120 ms, intervals
  up to [0.38, 1.82]), so its ratios say nothing; the identical code makes a second round needless.
* **`tests/perf/int_math.em` fixed** (ODR-277 found the defect; its rewrite, `60588521`, reached only
  `bench/programs/a01_int_math.em`): the gate's program added to `total` three times a pass where
  its C twin adds once. Written as the twin is: 0.948x C [0.934, 0.963] on core 1 (26.71 ms against
  28.18 ms; the `nsw` arm's run, its code identical to the plain build's), where it was 1.281x
  [1.244, 1.336] before (35.53 ms against 27.74 ms) — the same loop as the twin's.

## 4. `readonly` (ODR-275) and `dereferenceable(N)` (ODR-276) — DONE

### ODR-275: what changes through a shared reference is in an `UnsafeCell`

* `std.sync`: `Mutex[T]` and `RwLock[T]` hold `value: UnsafeCell[T]` and `word: UnsafeCell[usize]`;
  `Atomic[T]` holds `value: UnsafeCell[T]` (`std/src/sync.em`). The compiler-built `RefCell[T]`
  holds its value, its borrow count and the location of its last borrow each in an `UnsafeCell`
  (`refcell_of`); `RefCell(v)` and the `std.sync` constructors put what they are given into the cell
  (`in_unsafe_cell`). `Once` holds an `Atomic`. `Cell` stays the other primitive, recognised by its
  name as before: it is `Copy` where an `UnsafeCell` never is (`[UNS-10]`), so it cannot hold one.
* The code that reaches that state goes through the cell: MIR lowering of `lock`, `read`/`write`,
  the atomics and `borrow`/`borrow_mut` (`state_ty`, `state_place`), the C backend's guard ends
  (`state_member`: `->word.value`, `->borrow.value`), the LLVM backend's and the evaluator's
  (`TypeTable::cell_held`), the borrow checker's guard loans (`guarded_parent`).
* `TypeTable::holds_a_cell` is now the one fact `[BRW-8]` and `readonly` read; the hand list ODR-275
  ruled out does not exist. `[UNS-10b]`'s `E3105` in `@static_safe` code looks into what a std type
  holds (its `T`), not at std's own cells: a `RefCell` parameter is still `[EFF-6a]`'s `E4030`, as
  the tests expect.
* Before this, `holds_a_cell(RefCell[T])`, `(Atomic[T])` and `(Mutex[T])` were false: `loop_version`
  (D-529's lent locals) treated a shared loan of one as unchanging. No test showed it.
* Hot reload: a `Mutex[Score]` static's value is now an `UnsafeCell[Score]`, which had no schema
  (`image::is_compiler_struct` listed `UnsafeCell_`), so five reload tests saw "field `value`
  changed type" where `Score` changed. An `UnsafeCell` holding the package's types now has its own
  schema, as a std struct does (`[HR-13a]`), and the reload follows into it: 50 of 50 reload tests
  under both backends again.
* `CELL-9/accept_the_counter_is_one_machine_word` pinned the C `ptrdiff_t borrow;`; the counter is
  now `UnsafeCell…G9ee56c8f11b4d4af borrow;` (`G9ee56c8f11b4d4af` is `isize`'s instance), which
  the test now pins instead (the same fact: the counter is one machine word).

### ODR-276: what a callee can end, established first

The cases (section 1b; both backends and the evaluator):

| Case | What happens |
| --- | --- |
| `RC-5/accept_a_callee_ending_the_owner_a_borrow_points_into` | the callee drops the last other handle of the class object a borrowed parameter points into: the caller's loan keeps the object until the statement ends |
| `RC-5/accept_a_callee_replacing_the_field_a_borrow_went_through` | the callee replaces the field the borrow was taken through: the old object stays alive for the loan |
| `RC-5/accept_a_callee_dropping_one_owner_of_a_borrowed_shared` | `mem.drop` of a `Shared` owner in the callee frees nothing while the borrow lives |
| `HEAP-5/run_fail_a_callee_replacing_the_shared_a_borrow_points_into` | replacing the `Shared` the borrow points into panics |
| `EXC-2/run_fail_a_callee_clearing_the_list_an_element_borrow_came_from` | clearing the list an element reference came from panics before anything is freed |
| `EXC-2/run_fail_..._replacing_the_list_...` | so does replacing it |

Safe Ember gives a callee no way to end the object a borrowed parameter points into; unsafe code
that does is outside `[UNS-*]`'s rules. So the parameter attribute is derived, everywhere: an
Ember function's reference parameter is `nonnull dereferenceable(size of the referent)`, and a
shared one whose referent holds no cell is also `readonly` (`abi::Ext::Ref`, `body_signature`). A
zero-sized referent gets none; an export's, a C-callable function value's or a callback's
parameters none (their callers are foreign). No call-site attribute is needed.

### Tests

* `ember_driver/tests/llvm.rs` `cg_ll_3_references_are_dereferenceable_and_shared_ones_without_a_cell_readonly`:
  `ref Pair` is `nonnull dereferenceable(16) readonly`; a `mut` one, and shared ones to a `Cell`,
  a `RefCell`, a struct holding an `Atomic`, a `Mutex`, have `dereferenceable` and no `readonly`;
  an export's parameter has neither.
* `tests/conformance/UNS-10/accept_what_a_shared_reference_changes_is_in_a_cell` (both backends,
  every profile): a never-inlined function changes a `Cell`, a `RefCell`, an `Atomic` in a struct and
  a `Mutex` through a shared reference, and the caller reads the new value. Tried against a broken
  build that marks every shared reference `readonly`: `development` and `release` then print the
  old `Cell` and `RefCell` values (`1 1`), so the test catches the fact being wrong.
* Every test folder using a cell or a sync type (56 folders): 0 failing under the C backend with the
  evaluator; 334 of 334 run tests under LLVM in every profile.
* After items 4 to 6, on Windows: the whole conformance tree under the C backend with the evaluator,
  0 failing; under LLVM in every profile, 2144 of 2144; ui 12, milestones 175, firstweek 1, reload 50
  (both backends), cdylib 11 and staticlib 6 (both backends), llvm 12, interp 3. On Linux (WSL,
  gcc and Clang 21): every run test under LLVM, 2144 of 2144; the cell and sync folders under the C
  backend, 0 failing; reload 50, cdylib 11, staticlib 6 under both backends; llvm 11 of 12 (PGO,
  section 5's missing runtime). The C warning gate (`tools/c_warning_sweep.py`, MSVC and Clang) over
  the folders the C changes touch (CELL, THR, FFI-10, BLD-FFI, JOB, ECS, STD-9, EXC-19): no warning
  under MSVC, Clang or (in WSL) gcc.

## 5. Profile-guided optimisation (ODR-274) — DONE

* `ember build --pgo-generate` builds a program instrumented by Clang (`-fprofile-generate=<dir>`,
  compile and link) to write its raw profile into `target/<profile>/pgo/` when it ends.
  `ember build --pgo-use <file>` optimises with a recorded profile (`-fprofile-use=`): a `.profdata`
  as it is; a `.profraw`, or a folder of them (`--pgo-generate`'s), merged first by
  `llvm-profdata merge` into `target/<profile>/pgo/<module>.profdata` (the `llvm-profdata` beside
  the Clang the build uses, its real path followed; else the one `PATH` finds).
* The manifest: `[profiles.<name>] pgo = "generate"` or `"use"` with `pgo_data = "<file>"` (from
  the manifest's folder). The command line wins over the manifest, as `--profile` does; both flags
  together are refused; `pgo = "use"` without `pgo_data` is refused; a value other than the two is
  `E9001`.
* The LLVM backend's: the C backend refuses the flags naming it. A program only: a library, a
  reloadable image or a test binary is refused (a library's instrumented code runs in its host; not
  asked for).
* The build record (`inspect/<module>.build.json`) says which (`"pgo": {"generate": dir}` /
  `{"use": file}`). The program cache keys a `use` build by the profile's content.
* Test: `cg_ll_3_profile_guided_optimisation_records_and_uses_a_profile` — the instrumented build
  (the manifest asking for a missing profile, the command line winning), its run writing a
  `.profraw`, the manifest's missing profile refused, the `use` build merging and using it, the
  merged profile's counters for `work` (`[3003, 3, …]`: its loop's 1000 + 1001 + 1002 turns and its
  three entries), a broken `.profdata` stopping the build at Clang, the C backend refusing.
* **Missing tool (Linux).** In WSL the test fails at the instrumented link: Ubuntu's `clang` package
  has no profile runtime (`cannot find …/libclang_rt.profile.a`). The owner would run
  `sudo apt install libclang-rt-21-dev`. The build now says so (a `help` line naming the package
  when an instrumented link misses `clang_rt.profile`). Windows' LLVM has it
  (`clang_rt.profile-x86_64.lib`): the test passes there.

## 6. `extern "vectorcall"` and AAPCS64 (`[FFI-9]`, `[CG-LL-2]`) — DONE (AArch64: classified, not run)

### `__vectorcall` on Windows x64

* `abi::classify_vectorcall`, Clang's `WinX86_64ABIInfo` for the convention: the first six
  arguments may take vector registers, six in all, a floating-point scalar its position's; a
  homogeneous floating-point aggregate (one to four `f32` or `f64` members, nothing else, no
  `std.simd` vector — C holds those in a union, never homogeneous) takes as many registers as it
  has members from those left once every scalar has its own, as `{ T, … } inreg`, else it goes
  behind a pointer; a result of up to four such members comes back in registers; everything else is
  the C convention's. Calls, declarations and definitions say `x86_vectorcallcc`.
* The symbol is MSVC's `name@@N` (N the parameters' bytes, each rounded up to 8, an aggregate passed
  by address counting its own size), written `"\01name@@N"` so LLVM does not decorate it again by
  the IR's parameters (it would count such an aggregate as a pointer). An export's `/INCLUDE:` uses
  it too (`function_link_symbol`).
* Off Windows x64 the convention is refused by name (the C backend's `EMBER_CC_VECTORCALL` stops a
  build there the same way); a reloadable vectorcall function and one used as a value are refused
  (`[HR-6]`'s thunk and a function value's call are the C convention's).
* **D-916 (C backend, fixed; and the LLVM backend's counted wrapper)** `[FFI-9]`: a foreign function
  of an `extern "vectorcall"` block taking a struct by value is called through its counted
  wrapper, an Ember function with no convention of its own, and its C declaration came out without
  `EMBER_CC_VECTORCALL`: MSVC linked `vc_hva2` against nothing (`LNK2019`, the C library's symbol
  being `vc_hva2@@16`), and a C compiler that does not decorate would have called it with the C
  convention. `FfiCounted` now carries the foreign function's convention (`abi`), which both
  backends' declarations and calls use.
* Tests: `abi::tests::ffi_9_vectorcall_matches_clang` (eleven signatures and symbols, each Clang
  22's IR of the same C: `target/ai/abi_probe/vectorcall.c`); `ember_driver/tests/llvm.rs`
  `ffi_9_vectorcall_passes_floating_point_aggregates_in_vector_registers` (Windows): a program
  calling C compiled by the backend's own compiler (an HVA, an HVA after scalars, eight arguments
  leaving no register for the HVA, an HVA result, a five-double struct), built by both backends;
  the IR's declarations checked; both print the same.

### AAPCS64

* `Target::Aapcs64` (`Target::host()` on AArch64): Clang's `AArch64ABIInfo` for Linux: no small
  integer extended; a floating-point aggregate as `[N x T]` with `alignstack(8)` (16 when aligned
  to 16); another aggregate of up to 16 bytes as `i64`, `[2 x i64]`, or `i128` when aligned to 16;
  a larger one behind a pointer to a copy (not `byval`); results: a floating-point aggregate as a
  struct of its members, an aggregate of up to 8 bytes as the integer of its size, of up to 16 as
  `[2 x i64]` (`i128` aligned to 16), larger into memory; `long double` is `fp128` (`Sc::F128`).
  An aggregate whose parts reach past it (a 4-byte struct passed as `i64`, a 12-byte one as
  `[2 x i64]`) is read from and written to storage padded to the parts (`parts_span`), never past
  its own. Functions name Clang's AArch64 CPU and features; the object format is Linux's (DWARF,
  hidden visibility, `ember_reload`).
* Test: `abi::tests::cg_ll_2_aapcs64_matches_clang`, every parameter and result of the probe as
  Clang 22's IR of the same C for `aarch64-linux-gnu` (`target/ai/abi_probe/aapcs.c`), registers
  running out included.
* **Not run on AArch64.** There is no AArch64 machine here and no emulator (installing one is not
  allowed). On an AArch64 Linux machine with Rust and Clang:
  `cargo build --profile ci -p ember_driver && EMBER=target/ci/ember EMBER_BACKEND=llvm EMBER_PROFILES=all python3 tasks/impl-0.9.9/annotations.py tests/conformance/* tests/run-pass tests/run-fail`.
  Apple's AArch64 differs in one point not built: Darwin extends small integers.

## Merge notes for the lead

* Helper AE's branch changes the C backend's `reload.rs` too (the manifest's unit check,
  `reload_captures`, the layouts' tuple with `Option<ClassId>`): here those decisions moved into
  `ember_codegen_c::image` (`Planner`), which plans the layouts by class id already, so AE's change
  goes into the planner, and `reload.rs` keeps only the C text. AE's `frames.rs` change
  (`frames_on` for `Some(FpMode::Strict)`) applies to what is left of `frames.rs` as is.
* ODR-275 changed `std/src/sync.em`'s `Mutex`, `RwLock` and `Atomic` fields and the compiler-built
  `RefCell`'s to `UnsafeCell`s: code on another branch reading those fields by index (a lock's
  `word`, an atomic's `value`, a `RefCell`'s `borrow`) reaches the state through the cell
  (`TypeTable::cell_held`, MIR's `state_ty`/`state_place`).
* AE's `ember_driver::reload::build` takes units and ninja: here it takes a list of units already
  (an LLVM image is the `.ll` and its shim); a ninja rule for a `.ll` unit needs the IR flags
  (`ember_build::ir_flags`).
