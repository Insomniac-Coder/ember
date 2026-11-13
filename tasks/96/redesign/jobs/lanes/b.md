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
