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
  `ember test --backend llvm` builds the test entry in IR and runs the C backend's runner.
  Libraries and reload images are refused as not built yet. With the LLVM backend selected the
  MIR passes tuned for the C compiler (`c_for_msvc` at the seven pass sites) tune for LLVM
  (`tuned_for_msvc`, `tuned_for_gcc`); the C backend's choices are unchanged.
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
| 5b9400f2 | 1893 | 2075 |
| 3621cb1f | 1893 | 2075 |
| 25891563 | 1893 | 2075 |

Every listed profile (`EMBER_PROFILES=all`), after the merge of origin/main (D-903; 2088 run
tests): 2075 at first (the two defects below), 2077 with them fixed. The 11 left are the
reloadable programs (10 `HR-*`, one `LNT-5`), refused.

Causes left at e2a34cd8 (24): hot-reload images 11 (refused: `[HR-*]`), and one each of the
rest, each fixed since (backtraces, leak cycles, boxed `once fn`, export tables, `system`
convention, nesting 256 deep, reflection, float range niche, `ArenaSoA`, `dyn` clone). At
5b9400f2 the 11 hot-reload programs are all that fail.

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
- Speed, `[CG-LL-3]`: `cg_ll_3_functions_name_the_runtimes_cpu_and_elements_are_scaled_addresses`
  (every function says `"target-cpu"="x86-64"`: without it the LTO link compiled the IR for the
  linker's default CPU, a feature subset of the runtime's, and LLVM inlined no runtime inline
  function into the program; an element's address is one `getelementptr [N x i8]`, which
  LLVM's unroller counts as C's indexing, where `mul` + byte GEP unrolled `w1_sum_loop` 2x
  instead of 5x). `[SIMD-7]`: `simd_7_a_byte_sum_is_the_runtimes_sum_bytes` (the C backend's
  selection, `ember_codegen_c::byte_sum_loops`, now public; the C backend's output unchanged).
- `annotations.py`: `EMBER_PROFILES=all` with `EMBER_BACKEND=llvm` runs each test in every
  profile it lists (the C harness does); the release builds are where LTO, the MIR's loop
  versions and the byte sums apply.

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

## Performance (`ember bench --compare --backend llvm`, laptop, mains, core 1)

Twins built by Clang with the release profile's flags (`-O3`, full LTO), `[BEN-5]`. The
machine was shared with other helpers' builds; wide intervals are load, not code.

| run | build | passed | failed |
|---|---|---|---|
| 3 | 3621cb1f | 17 | a10 1.69x, int_math 1.28x, t5_words 1.08x, w1_sum_loop 1.31x |
| 4 | + CPU attribute, scaled GEPs | 19 | a10 1.79x [1.52, 2.04], int_math 1.27x [1.26, 1.28] |
| C backend, `EMBER_CC=clang` | same compiler | 19 | a10 1.70x [1.68, 1.73], int_math 1.27x [1.25, 1.28] |
| 6 | + byte sums, merge of D-903, the two defect fixes | 19 | a10 1.69x [1.65, 1.73], int_math 1.22x [1.14, 1.39] |

Run 4 against the C backend's own code under Clang (like with like): w1 20.6 / 20.4 ms, t5 56.0
/ 54.2, t2 16.0 / 10.9 (before the byte sum; its LTO assembly is now the C build's, `psadbw`),
the rest within noise. The two failures are the C backend's under Clang too:

- `a10_recursion` (1.7x): Clang turns the twin's `fib(n-1) + fib(n-2)` into a loop with an
  accumulator (reassociating the additions, which C's undefined signed overflow allows). Ember's
  `+` is checked and panics at the addition that overflows, so the same rewrite would move that
  panic; the IR's code is the C backend's. Not a backend defect; under MSVC the twin is not
  rewritten. Re-measured alone at the coordinator's request: 1.79x [1.28, 2.41] under load,
  1.69x on the quiet run.
- `int_math` (1.27x): the IR's loop at `-O3` without LTO is the twin's at `-O3` (one 64-bit
  multiply, two 32-bit). Full LTO's link-time pass over the already optimised module leaves
  the last turn's exit values computed on every turn: 38 instructions, 4 64-bit multiplies; the
  twin under the same LTO goes from 26 instructions to 29 (2 64-bit multiplies), less because
  C's `i * 7` is `nsw` and its `% 13` narrows to 32 bits. Measured on the assembly and set
  aside: `nsw`/`nuw` on the IR's arithmetic (no change; also unprovable, the MIR does not mark
  an operation whose check it proved away), signed `sdiv`/`srem` (worse), the module optimised
  at `-O3` before the LTO link (no change), the module not optimised before the link (fixes
  `int_math`, 23 instructions, but `s5_range_step_enumerate`'s loop is then not vectorised, in
  full and in thin LTO). Open: see question AF4.

## Defects found

Both in this backend's own code, found by the first run of every listed profile
(`EMBER_PROFILES=all`), both fixed:

- `[TYP-28]`, `[CG-LL-2]`: a signed `%` (floor or truncating) whose check is gone computed
  `sdiv`/`srem` of `MIN` by `-1`, undefined in LLVM though `MIN % -1` is 0: in `release` the
  program stopped printing there (`TYP-28/accept_floor_division_and_modulo`). Now only the
  operator's own division, the remainder's divisor `-1` taken as `1` (`signed_rem`).
- `[THR-14]`: with the CPU named but not `cx16`, an LTO build (`development`) left a 16-byte
  compare-exchange to `__atomic_compare_exchange_16`, which no link has
  (`THR-14/accept_128_bit_atomics`). Every function now says `"target-features"="+cx16"`, as
  the program's C gets `-mcx16`.

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
- AF4 — `[BEN-5]`, `[TST-28]` against `int_math` (above): the twin's lead is LLVM's full-LTO
  link pass plus C's undefined overflow, and it hits the C backend under Clang the same way.
  Options: (a) accept it as the twin's advantage (MSVC, the reference gate, has no gap);
  (b) a MIR fact on each arithmetic operation whose overflow check was proved away, which the
  LLVM backend gives as `nsw`/`nuw` and the C backend as signed C arithmetic (measured: alone it
  does not close this gap); (c) link the LLVM backend's release builds without LTO and reach the
  runtime's inline functions some other way (the IR's loop is then the twin's). Recommendation
  (a) for 1.0, with (b) worth its own measurement where loop narrowing matters.
