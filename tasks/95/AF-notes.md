# Helper AF — the LLVM backend (ODR-223), notes

Branch `worktree-agent-a7590733616cc8073`. The lead merges.

## Design as built

- `compiler/ember_codegen_llvm`: verified MIR + the type table's layouts → one textual `.ll`
  module. No LLVM library in the compiler; the `.ll` is a source of the same Clang command that
  compiles the runtime and links (`ember_build::compile_and_link_program`), so the profile's
  `-O` level, `-g` and LTO flags (ThinLTO in `development`, full in `release`) apply to it as to
  the C backend's C.
  - `abi.rs` — the platform C ABI's classification (Windows x64, System V x86-64), as Clang
    does it, register counting included. Every Ember function uses it, so runtime calls,
    function values, callbacks and exports are one convention. Unit test against Clang 22
    (Windows) and Clang 21 (Linux) lowering of the same prototypes.
  - `types.rs` — each type is a register scalar (`Sc`), nothing, or bytes in memory with the
    type table's size and alignment; places are byte offsets (no IR struct types, so no second
    copy of C's layout rules).
  - `runtime.rs` — the runtime entry points the IR calls, with their C prototypes; a unit test
    reads `ember_rt.h` and compares every prototype (macro-made ones by their instantiation,
    function-pointer parameters by their declarator). Header-inline functions are reached
    through a generated C shim of one-line wrappers, linked beside the IR (LTO inlines them).
  - `func.rs` — bodies: every local a stack slot (promoted by `mem2reg`/SROA at `-O1`+),
    `sret` results written to a fresh slot then copied. `[GRM-39]`: a literal's temporary that
    the next literal reads once is written in that literal's field (`fold.rs`, the C backend's
    `foldable_aggregates` from the same MIR), so a value nested 256 deep has no 256 slots.
  - The rest by feature: `glue.rs` (drop/retain glue, class type information, guards of
    `RefCell`/`Mutex`/`RwLock`, arenas), `dispatch.rs` (virtual and interface calls, `dyn`
    tables, their clone entry), `closures.rs` (callable values), `fmt.rs`, `cmp.rs`,
    `arrays.rs`, `soa.rs` (`SoA`, `ArenaSoA`), `simd.rs` (`std.simd` as LLVM vectors),
    `sync.rs` (atomics as LLVM atomics, threads, scopes, jobs, `@parallel` loops), `views.rs`,
    `ranges.rs`, `statics.rs` (lazy statics, constant statics), `ffi.rs` (counted wrappers,
    callbacks and their trampolines, foreign destructors), `leaks.rs` (the leak check's
    ownership edges), `debug.rs` (line tables).
- `[CG-LL-2]`: checked arithmetic through `llvm.{s,u}{add,sub,mul}.with.overflow` (vectors too);
  the checked division forms divide by 1 where `MIN / -1` would be, with the C helpers' flags
  exactly; shift amounts masked to the width; float→int through `llvm.fpto{s,u}i.sat`
  (`[TYP-6]`); 128-bit division and conversions through the runtime's helpers (an MSVC link has
  no compiler-rt); `f16` through the runtime's exact conversions; no `nsw`/`nuw`/`inbounds`/
  `noundef`/`exact`; fast-math flags only under `@fastmath`/`@fp(contract)`, never `nnan`/`ninf`
  (poison).
- `[CG-LL-3]` as built: `nonnull` on reference parameters (never null by the type system);
  `noalias` on the element pointer of each view of a list kernel (`restrict_views`, the MIR fact
  the C backend makes `restrict`; the kernel's views are split into pointer and length) and on
  `sret`; CodeView (Windows) / DWARF line tables in every profile: each MIR body a
  `DISubprogram` named by its symbol, each instruction its statement's Ember line and column.
  Backtraces name frames from them (`[RT-4]`, `[CG-C-10]` pass).
- Driver: `--backend c|llvm`, `[build] backend = "llvm"` (`compiler/ember_driver/src/llvm.rs`,
  one branch in `compile()` before the C emission). `--emit llvm-ir` prints the module. Clang is
  required (`Toolchain::detect(Some("clang"))`); a missing one is `E9002` with
  `winget install --id LLVM.LLVM` / `sudo apt install clang lld`. `--cc` other than clang is
  refused. `import c` shims and C sources are compiled and linked (`cimport::build_objects`),
  `[link] libs` resolved as the C path does; the build record is written (`[BLD-6]`).
  Libraries, reload images and `ember test` are refused as not built yet.
- `ember bench --compare --backend llvm`: the Ember programs built by the LLVM backend, their C
  twins by Clang (`compiler/ember_driver/src/bench.rs`).
- `ember_build`: a `.ll` source adds `-Wno-override-module -Wno-unused-command-line-argument`.
  Nothing else changes for the C backend.
- `tasks/impl-0.9.9/annotations.py`: `EMBER_BACKEND=llvm` runs only the run tests, built with
  `--backend llvm`, and prints `backend llvm: N run tests, M passing`.

## Ledger

run-pass + run-fail (182 run tests):

| date | commit | passing |
|---|---|---|
| 2026-10-23 | step 1 base | 31 |
| 2026-10-23 | classes, interfaces, arrays | 135 |
| 2026-10-23 | closures, formatting, Shared | 158 → 170 → 175 |
| 2026-10-23 | 73db2bd8 ranges, 128-bit | 177 |
| 2026-10-23 | 3d2f0c19 SoA, FFI | 182 |

conformance tree (1904 run tests) + run-pass + run-fail:

| commit | conformance passing | all (2086) |
|---|---|---|
| (before 73db2bd8) | 1065 | |
| 73db2bd8 | 1507 | |
| 3d2f0c19 → 03497d80 | 1787 | |
| e2a34cd8 | | 2062 |
| (this commit) | see below | |

Causes left at e2a34cd8 (24): hot-reload images 11 (refused: `[HR-*]`), and one each of the
rest, each fixed since (backtraces, leak cycles, boxed `once fn`, export tables, `system`
convention, nesting 256 deep, reflection, float range niche, `ArenaSoA`, `dyn` clone).

## Done (each with its tests)

- Steps 1–5 of the brief: every MIR construct the conformance tree's run tests, tests/run-pass
  and tests/run-fail use, but the hot-reload image (below): scalars, control flow, calls,
  structs, tuples, enums and `match`, arrays and views, strings, classes (objects, counts, weak
  references, virtual and interface dispatch, `dyn`, downcasts), drops and moves, every check
  with the C backend's message (overflow, bounds, division, exclusivity, `RefCell`, stale
  handles, lanes), closures and callable values, generics (monomorphised MIR), generators and
  coroutines, formatting, foreign calls and exports with the C ABI's classification (counted
  wrappers, callbacks, `import c` shims and sources, foreign statics, export tables), threads,
  scopes, jobs, `@parallel` loops, atomics, locks, arenas, `SoA`, `std.simd`, `f16`, lazy and
  constant statics, reflection, the leak check, backtraces. Evidence: `EMBER_BACKEND=llvm
  python tasks/impl-0.9.9/annotations.py` over the tree (ledger above).
- `[CG-LL-2]`: ember_driver/tests/llvm.rs (`cg_ll_2_checked_arithmetic_…`,
  `cg_ll_2_floating_point_…`, `cg_ll_2_simd_vectors_are_llvm_vectors`); ember_codegen_llvm's
  `abi::tests::cg_ll_2_matches_clang_on_both_targets`, `runtime::tests::rt_5_…`.
- `[CG-LL-3]` (part): ember_driver/tests/llvm.rs (`cg_ll_3_references_are_nonnull_…`,
  `cg_ll_3_debug_information_…`); tests/debugger.rs
  (`the_llvm_backends_program_gives_the_same_debugger_session`, `[TST-5]` with cdb).
- `[CG-LL-1]`: `cg_ll_1_an_unknown_backend_is_refused`; `ember test --backend llvm`;
  `ember bench --compare --backend llvm`.

## Not done (and why)

- Hot-reload images (`[HR-*]`, 11 conformance tests): the driver refuses `--backend llvm` for a
  reloadable program. The image is the C backend's thunks, tables and manifest over a DLL; for
  LLVM it is the same plan written as IR, a piece of its own.
- Libraries (`cdylib`, `staticlib`): refused; the IR needs the module protocol's entry points
  and the export list the C path writes.
- `[CG-LL-3]`'s `readonly` and `dereferenceable(N)`: not given, because neither is derived yet
  (spec questions AF2, AF3). PGO: no way for a build to ask for it (AF1).
- `extern "vectorcall"`: its classification (HVAs in vector registers) is not implemented.
- Another target's C ABI (AArch64): only Windows x64 and System V x86-64.
- The LLVM backend as the default: `[CG-LL-1]` makes it the default once it passes the
  performance suite on the reference laptop; see the bench results below.

## Defects found

## Spec questions

- AF1 — `[CG-LL-3]`, `[BLD-6]`: how a build asks for profile-guided optimisation. Options:
  (a) the manifest, `[profiles.<name>] pgo = "generate"` and `pgo = "use"` with
  `pgo_data = "<file>"`; (b) the command line, `ember build --pgo-generate` and
  `--pgo-use <file>`; (c) both, the command line over the manifest, as `--profile` is.
  Recommendation (c): the instrumented build is a one-off run (b), the use of a recorded
  profile belongs with the package (a); `llvm-profdata` merges the raw files, shipped with the
  bundled LLVM (ODR-223).
- AF2 — `[CG-LL-3]`'s `readonly` on a shared reference needs the fact that the referent holds
  no state a shared reference may change. `TypeTable::holds_a_cell` is that fact for
  `Cell`/`UnsafeCell` only: `RefCell`'s count, `Atomic[T]`'s value and a lock's word change
  through a shared reference without one. Options: (a) a structural walk naming std's
  interior-mutable types; (b) every such std type holds its changing state in an `UnsafeCell`,
  so one fact serves `[BRW-8]` and `readonly`. Recommendation (b).
- AF3 — `dereferenceable(N)` on a reference parameter claims the referent stays allocated for
  the whole call (LLVM may load from it speculatively anywhere in the function). Is that an
  Ember guarantee (`[LT-*]`): can a callee end the object a borrowed parameter points into, by
  releasing the last handle to a class object or clearing the list an element reference came
  from? If the borrow rules rule it out, the attribute is derived; otherwise it needs a proof
  per call.
