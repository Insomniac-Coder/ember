# Pass 0b — Feature parity of 0.9.9_Hardened_1 with 0.9.8_Hardened_3

**Question (owner, 2026-09-23):** does 0.9.9 keep every feature 0.9.8 had, unless something better
replaced it?

**Answer: yes, after the restorations below.** The first build of H1 had lost a number of features
while condensing 0.9.8's 870 KB into a language-only document. This pass found them mechanically and
by reading, and restored them into H1. What 0.9.9 no longer has is either replaced by something named
below, or is compiler internals or project planning that was never a language feature.

## How it was checked

1. **Mechanical inventories**, each diffing 0.9.8 against 0.9.9:
   * attributes, command-line commands and flags, manifest keys, standard-library names,
     grammar terminals and productions, diagnostic codes, section headings;
2. **Every retired rule id** (0.9.8 ids no longer defined): 244 non-internal ones were read in full and
   classified present / replaced / internal / restored;
3. **Every surviving rule that shrank by more than half** (212 rules): read side by side with its 0.9.8
   text for lost features, constraints or diagnostics.

## Restored into H1

| Area | Restored |
|---|---|
| CLI (§XVII.1) | `ember why --alloc/--block/--io/--lock/--sync/--unsafe/--ffi` (`[CLI-12]`); `ember calls --foreign` (`[CLI-13]`); `ember audit` (`[CLI-11]`); `ember update`; `ember toolchain list/install/default` and the no-separate-C-toolchain promise (`[TOOL-2]`); the "no C compiler" help (`[TOOL-3]`, `E9002`); `--emit tokens/ast/hir/mir/c/obj/header` and `--out-dir` (`[CLI-2]`); `--backend`; `--report=instantiations`; `--reload [--explain]`; `--hot`; `--timings=json`; `ember bench --compare`; `ember bind --init --merge`, `--write-baseline`, `--emit-embind`; `ember tcb --module`; `ember shader-bind`; `ember --help` completeness (`[CLI-15]`); what `--version` prints and the first-hour milestone (`[TOOL-4]`); release archives (`[TOOL-1]`) |
| Manifest (§XVII.2, §XVII.4) | `[build] entry/backend/c_compiler/runtime/max_instantiations`; dependency forms (path, git, version); `edition_lints`; lint thresholds; `[comptime]` keys; profile inheritance and custom profiles; `[ffi] evidence`; `[cpp.<project>]` keys (`[FFI-50]`) |
| Grammar (Part III) | the `#!` directive production, including `#! threads` (`[GRM-37]`); `import cpp`; statement-attribute placement (`[ATT-3]`); `ref e` expressions (`[GRM-36]`) |
| Attributes | `@hot`; user attributes as `@attribute` structs replace plugin namespaces (`[ATT-1]`, `[RFL-3]`) |
| Standard library (Part XV) | `Mat2`, `UVec2/3/4`, `Sphere`, `rsqrt`, `sqrt`, `min_by`, `max_by`, `stderr`, `BufReader`, `BufWriter`, `assert_approx_eq`, `expect_panic`, the bench harness, profiler `zone`, `ForeignBox`, `Retained`, `Callback`, `c_wchar`, `Volatile`, `copy_nonoverlapping`, the public iterator types, `Hasher`'s API (`[HASH-1]`), `Cell.update`, arena container APIs (`[ARN-5c]`, `[ARN-5d]`, `[ARN-5g]`), nested arena scopes |
| C FFI (Part XVI) | C type mappings for `__int128`, `_Float16`, `wchar_t`, `char16_t`, `char32_t`, `volatile`, packing pragmas, globals, string macros, other calling conventions, `va_list` (`E5018`); composable overlays (`[FFI-30c]`); "reject rather than guess" (`[FFI-38]`); thread checks on foreign objects (`[FFI-33a]`, `[FFI-33b]`, `[FFI-33c]`); `@export_table`; the shared runtime (`[FFI-31c]`); CMake integration moved from the C++ annex (`[BLD-FFI-2]`); macro-call folding; direct `ForeignBox` from `returns_owned`; double-adoption check; the `E5002` diagnostic with its overlay line |
| Diagnostics (§XVII.6, §XVII.9) | a complete code registry (232 live codes, 12 retired with reasons); codes keyed to shapes (`[DIA-7a]`), including B14 and U1; `explain --borrow` output (`[DIA-8]`); concrete-API help (`[DIA-10]`); S1's reason-code rule (`[DIA-11]`); UI snapshots (`[DIA-13]`); suggestion sources (`[DIA-15]`); FFI missing-fact help (`[DIA-18]`); the performance and style lints (`[LNT-6]`) |
| Builds and gates (§XVII.3, §XVII.5) | the compile-time budget table B1–B9 with its gates (`[BUD-1]`–`[BUD-5]`); the benchmark protocol and thresholds (`[BEN-1]`–`[BEN-6]`); the exclusivity-hoisting performance cases (`[EXC-13]`); Ninja, output layout and parallel front end (`[BLD-4]`, `[BLD-5]`, `[BLD-7]`); mechanical reject-case rule (`[TST-4b]`); the scenario coverage list (`[TST-11]`) |
| Editor, docs, conformance, ABI | resilience, no codegen in requests, session lifetime (`[IDE-3]`, `[IDE-4]`, `[IDE-6]`), `ember lsp` reserved; generated API contracts and joint publication (`[DOC-3]`, `[DOC-4]`); partial profiles (`[CONF-1]`); `[ABI-2]`–`[ABI-5]`; ABI version check at init (`[VER-4]`) |
| Implementation (Part XVIII) | inline-header criteria (`[CG-C-3]`); `restrict` details, `#line` attribution; relaxed-float functions kept out of the inline header; sharing only outside hot loops (`[MONO-6]`) and its report; `[MONO-9]`; `[RT-5]` |
| Effects and determinism (Part X) | module-level `@deterministic` and foreign determinism (`[DET-8]`, `[DET-9]`); the pinned list of proofs behind contract verdicts (`[EFF-15]`); the implementation-defined cost class (`[COST-2]`) |
| Memory and objects (Parts VII–IX) | `@borrows` naming an `Arena` (`[LT-1a]`); `UnsafeCell` suspends nothing (`[UNS-10a]`); conservative cycle analysis and the run-time report (`[WK-7]`, `[WK-8]`), cycle-analysis root (`[CLI-18]`) |
| Hot reload (Annex B) | the full migration table with `@renamed_from` and `@reinit_on_reload`; relocation limits (`[HR-15a]`, `[HR-15b]`); publication ordering (`[HR-42]`, `[HR-42a]`); the reload manifest (`[HR-8]`); removed-function stubs (`[HR-7]`); the 3 % overhead ceiling and no inlining of reloadable code (`[HR-9]`, `[HR-9a]`); container registration (`[HR-13a]`, `[HR-13b]`); export-table stability (`[HR-21]`); C++ header changes (`[HR-23]`); GPU in-flight bound (`[HR-24]`); `@allow_reload_terminate` (`[HR-43]`); foreign failure (`[HR-39]`); the failure matrix (`[HR-41]`); stats and `hot.checkpoint()` (`[HR-32]`, `[HR-33]`) |
| C++ (Annex C) | the import classification (`[FFI-44]`); unsupported constructs (`[FFI-48]`); bridge types and their costs (`[FFI-17e]`, `[FFI-17f]`); reference mappings and `@ffi(invalidates)` (`[FFI-40]`, `[FFI-40a]`); iterator-invalidation prevention (`[FFI-42a]`); smart handles only for `@sync` classes (`[BLD-FFI-5a]`); destruction order and trampoline ownership (`[FFI-17c]`, `[FFI-17d]`); base construction and upcasting (`[FFI-39a]`–`[FFI-39c]`); exception detail (`[FFI-24b]`–`[FFI-24d]`); trust grades and evidence (`[TCB-1]`–`[TCB-6]`, `[FFI-37a]`–`[FFI-37d]`); the C++ corpus (`[CXX-1]`) |
| GPU (Annex D) | the device interface, command recording, access states, `[GPU-7]` leak report, the render graph (`[GPU-9]`), `[GPU-11]`, the reserved kernel language |
| Overview and glossary | non-goals (§I.6); 18 glossary terms |

## Replaced by something better (not restored)

| 0.9.8 | 0.9.9 | Why better |
|---|---|---|
| `::` path separator | `.` everywhere (`[GRM-24]`) | one separator, as Python |
| `'a` reservation, `@latebound`, `with_views*` (`LT-8`–`LT-13`, `FN-6b`) | no lifetime syntax; callable types get fresh regions per call (`[LT-7]`) | accepts the safe programs those features rejected (F-168) |
| language-version selectors (`MOD-6`, `MOD-6a`) | one language before 1.0 (`[VER-8]`) | no test matrix of eight languages (F-046) |
| `exclusivity = "unchecked"`, `gpu.validate`, profile `overflow`/`bounds_checks` keys | no setting removes a check (`[PRF-1]`) | no profile-dependent safety (F-130) |
| `@thread_local` | classes are shareable only when `@sync` (`[THR-1]`) | explicit, and removes the data race (F-078, F-104) |
| `@move_only` | `Copy` is always explicit | the attribute had nothing left to do |
| `@serialize`, `@field` | `@derive(Serialize, Deserialize)`, `@reflect` with `@attribute` structs | typed, general mechanism |
| `@derive(SoA)`, `columns_mut`, field-name arguments (`SOA-5`) | `SoA[T]` for any struct; columns are disjoint places (`[SOA-1]`, `[SOA-2]`) | no special argument kind (F-112) |
| access-mode generic kind (`GRM-14`) | `Mut[T]` marker type (`[ECS-3]`) | no language surface for one library (F-024) |
| `unsafe(reason = …)`, reason category (`UNS-9`, `L3018`) | `# SAFETY(category):` note (`[LEX-23]`, `[UNS-8]`) | one annotation channel (F-120) |
| `Callable[…]`, `CallableOnce` interfaces | `fn(A) -> R` and `once fn(A) -> R` types (`[CLO-3]`, `[CLO-6]`, `[CLO-14]`) | one spelling |
| `Resumable`, `CoroutineState`, `Coroutine[R]` | `Generator[Y, R]` implementing `Iterator`, with `result()` (`[CORO-3]`) | Python's model; same power |
| truncating `/` and `%` (`EXP-7`) | true `/` on floats only, floor `//` and `%` (`[TYP-28]`) | no silent Python trap (F-155) |
| C++ exception policy with no default (`FFI-43` as written) | a derived policy with a default (`[FFI-24]`) | F-157 |
| `--leak-check` opt-in | leak report on by default in `debug`, `--no-leak-check` (`[WK-15]`) | F-082 |
| `--verify-comptime`, `E6002` | compile-time evaluation deterministic by construction (`[CT-4]`) | nothing left to verify |
| `--explain-performance` | `--report=engine` and `ember inspect --cost` | two focused reports |
| per-job access sets verified only in `debug` (`JOB-3`, `JOB-4`) | borrow rules across a scope; ECS sets checked at compile time (`[JOB-3]`, `[ECS-4]`) | a debug-only safety check is not a guarantee |
| `ArenaMap` in unspecified order | insertion order like `Map` (`[ARN-5d]`) | deterministic |

## Not language features (not restored)

Compiler architecture (`AST`, `HIR`, `MIR`, `MIR-REG`, `CLO-ABI`, `COR-ABI`, `CMP`, `COMP`, `DET-IMPL`,
`FFI-IMPL`, `HR-IMPL`, `HOT`, `STD-IMPL`, `VERIFY`), the implementation plan (`IMP`, `GATE`, phases,
milestones), the host-engine plan (`RV`), requirement categories (`CAT`), closed open questions (`OQ8`,
`PRV`, `CTR`), and the branding constants of the build (`MAN-4`). Each retired id is listed in
Appendix H with the reason.
