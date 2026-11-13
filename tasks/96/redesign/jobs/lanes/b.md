# Lane b: the C interface and threads (M12, M9)

Read first by every job in this lane; updated last by each.

- **Milestones:** M12 (the C interface: `unsafe import c`, `@unsafe(export)`, the checked trampoline, what a
  foreign call may run) and M9 (threads). M12's A6-6 (FFI-53 in the runtime) needs M8's two-word function values,
  and A6-8 (FFI-52) needs M6a's summaries: those wait. Items, files and tests: `gap/06-ffi-hot-reload.md`,
  `gap/05-threads-unsafe.md`.
- **Families:** FFI, BLD-FFI, UNS, TCB, THR, JOB, PAR, CG-C, TIER, BLD (103 unfinished rules, 27,054 words, on
  2026-11-13).
- **Worktree:** `C:\Users\ism19\Code\ember\.claude\worktrees\lane-b`, branch `lane-b`; its `target/` stays between
  jobs.
- **Where the code is (line numbers drift; search the names):**
  - the C importer: `ember_driver/src/cimport.rs`; foreign builds: `ember_driver/src/foreign.rs`;
  - `ember tcb`: `fn tcb` in `ember_driver/src/lib.rs`;
  - `@ffi` contract words: `FFI_CONTRACT_WORDS` in `ember_typeck/src/lib.rs`; export symbols and policies:
    `export_symbol`, `export_thread_policy`, `export_abort_policy` there;
  - attributes and overlays in the parser: `ember_parser/src/decls.rs`;
  - the C header writer: `ember_codegen_c/src/header.rs`; the LLVM side: `ember_codegen_llvm/src/ffi.rs`;
  - the runtime: `runtime/ember_rt/templates/ember_rt.{h,c}.in`; `std/src/ffi.em`.
- **New code** goes in lane modules (for example `ember_typeck/src/ffi.rs`), not appended to `ember_typeck/src/lib.rs`.
- **Traps:** M4's FFI items (A6-5, A6-9, A6-10, FFI-35a) are lane a's, on `lead-m4`.

## Job 1 (H3, 2026-11-13): what was found and done

- **Done and credited (14):** UNS-2, UNS-3, UNS-5, UNS-10, UNS-10a, UNS-11, FFI-26, FFI-25, BLD-FFI-1,
  FFI-29b, FFI-29, FFI-11a, FFI-36, FFI-33b (each commit's message lists the tests).
- **Where the new code is:**
  - `ember_typeck/src/ffi.rs` (new, this lane's module): `@unsafe(…)`'s form (`check_unsafe_attribute_form`:
    the plain `@export`/`@export_table` is E3100, `@unsafe(other)` E0104) and the names an export may not
    take (`check_export_name_taken`: the C11 library list, the runtime prefix, the module-protocol names;
    `main` goes through `exports_main` to the old entry-point check in `check_program`).
  - `ast::Attribute::written_unsafe` and `UnsafeSiteKind::{Attribute, Import}`; `ImportKind::Foreign::is_unsafe`.
    The parser pushes both sites; `check_safety_notes` (driver) gives W3012 for them with the note only on the
    line above (`[LEX-23]`); `ember tcb` lists them in its unsafe section (`UnsafeEntry::written`).
  - `cimport::refuse_compiled_in`: E3100 for a `.c` source or `implementation = [...]` without `unsafe`;
    `Known::implementation_requests`: E5014 when a second package requests one header's implementation.
  - `@ffi(no_callbacks)`: accepted (`Checker::check_no_callbacks` in `ffi.rs`, E5069 on a function that
    takes a callback or a function pointer), listed by `ember tcb` (`foreign.rs`); the walk that uses it is
    FFI-52's (A6-8). `ffi::contract_words_shown` keeps `unsafe` and `TODO` out of E0104's help.
  - `std/src/mem.em`: `copy`, `copy_nonoverlapping`, `transmute`, `zeroed`; the E3100 naming what a copy
    writes is `UnsafeStep::CopyInto` (`check_mem_copy_safety`).
  - FFI-36: `ffi.rs`'s `refuse_no_public_destructor` (E5002 for `adopt` of a C++ class with no public
    destructor), called from `synth_foreign_adopt`.
  - FFI-33b (option c): `Builtin::ForeignCallAt`, emitted by MIR (`lower.rs`, the `Call` arm) before a call
    whose `returns_owned` result is bound to its thread; each backend's drop of a named local of
    `ThreadForeignOwner` calls `ember_ffi_end_name`; the runtime's two thread-locals
    (`g_ffi_call_at`, `g_ffi_end_name`) feed `ember_ffi_adopt` and `ember_ffi_release`.
- **Traps found:**
  - `milestones.rs`'s `the_conformance_suite_runs` runs every conformance test with the binary built at the
    start: editing tests while it runs pollutes its result. Run milestones with `-- --skip
    the_conformance_suite_runs` and use `annotations.py` for the tree; `every_example_of_the_specification_checks`
    keeps a baseline of failing spec examples by heading, which a fix must lower.
  - `bench/bigpkg/generate.py` writes a package that imports C (the speed test builds it).
  - The Bash tool sometimes turns a doubled backslash into a single one in a command's text: a heredoc
    Python edit that writes Rust or Python with backslashes can come out wrong. Write such scripts with the
    Write tool, and edit Rust with Edit.
  - The scratchpad is the lead's too: keep scripts in a folder of the lane's own (`scratchpad/H3/`).
  - A run test also runs on the evaluator: a pointer cast between types (`(p as *B).read()`) fails there
    ("a 1-byte value written to a 24-byte `String`"); a byte copy between `MaybeUninit`s works on both.
  - A generic std body's `unsafe:` is judged per instance by L3010: a `MaybeUninit` of a type parameter that
    is not `Copy` now counts as one that may need drop.
- **Left, with what each needs** (none credited):
  - FFI-51's header analysis (what a header leaves in the program, `#pragma comment(lib)`, GCC's assembly
    text, the libraries' own names), BLD-FFI-3 (`unsafe_libs`, `[link.<system>]`, `/DEFAULTLIB`, unused
    libraries), and with them TCB-4's external-library lines; BLD-FFI-1a needs MAN-10's dependency flags.
  - FFI-33 and FFI-33c need `static mut`, `Lock` and `guarded by` (STA-1, THR-20: E7012); FFI-33a needs
    E7015 (no test anywhere emits it).
  - FFI-11 and FFI-11b need A6-4's last part: `ref mut p: MutSpan[T]` (and `ref mut p: Option[MutSpan[T]]`)
    for a count `exclusive` parameter C writes, the call marking `ref mut` (`ref mut p: T` for `one exclusive`
    is built). Today such a parameter is a plain `p: MutSpan[T]` (`Mode::Borrow`, the `Count` arm of the
    carrier check in `ember_typeck`); the wrappers read `FfiAbiParam::SpanPointer` by value in both
    backends, so either they learn a by-address span or the signature keeps `Borrow` and the call word is
    checked apart. 12 declarations in 9 test files, and the importer's 8 `MutSpan` sites; `no_callbacks`
    is recorded now.
    FFI-11's `bind --report` clause (each `from(static)`/`from(p)` result listed) is FFI-35a's (lane a).
  - TCB-4: its unsafe section now lists `unsafe import c` and `@unsafe(…)`; left: the external library a
    header names (FFI-51's analysis), `@ffi(safe)` (FFI-56), and an `unsafe fn`'s entry shows "no safety note
    (W3012)" where its note is its `@safety` (`[UNS-7]`).
  - FFI-56 and FFI-8 (C function pointers as `unsafe extern "C" fn`, `Option` only where `nullable` or in a
    struct field, `@ffi(safe)` typedef entries, E5013's message, the handle rules); FFI-31b (an export's
    `Span`/`ref mut` carriers, E5063, the `@non_exhaustive` enum, the header's per-parameter contract).
  - FFI-36a: N173/N180 in the importer and typeck (every place compared at once: a C++ class's public
    constructors and by-value results name `delete`, a `std::unique_ptr<T, D>` names `D`; a hidden
    declaration counts only where a wrapper calls it; one E5002 per type naming every place, with its
    `hide` line or the contract to write). Today the importer compares only the `std::unique_ptr`s
    (`Importer::deleters`), and typeck emits one E5002 per conflicting `returns_owned` result
    (`note_result_destructor`). `FFI-17a/accept_a_unique_ptr_with_its_own_deleter` (a public constructor
    beside `std::unique_ptr<Widget, Recycler>`) is then a reject case.
  - FFI-17a also needs `std::function` as `unsafe fn(A…) -> R` (E3100 at a call, the message the rule
    gives) unless `safe` is beside `std_function` (`[FFI-2]`'s limit, E5002, tcb), and `no_callbacks` there.
