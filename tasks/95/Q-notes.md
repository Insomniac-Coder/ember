# Helper Q — Parts XVI/XVII builds and ABI (2026-10-22)

Rules: BLD-1, BLD-4, BLD-6, BLD-7, BLD-8, BLD-10, ABI-4, ABI-5. The 40-minute box was lifted by
the lead: build what is missing, commit after each rule.

## Done

- **ABI-4** (f1abeba6) — `docs/RELEASE-NOTES.md` (new) notes every version of each compiler
  protocol and why it changed: runtime ABI 1, 2 (8ece921b, `ember_rt_config.size` + the caller's
  version), 3 (b42a5386, `ember_type_info.reflect`); module protocol 1; hot-reload protocol 1. An
  export table's protocol (`Name_vN`, `[FFI-26]`) is the package author's, so the package's
  release notes it. Test: `compiler/ember_driver/tests/cdylib.rs`
  `every_protocol_version_is_in_the_release_notes` reads `EMBER_RUNTIME_ABI`,
  `EMBER_MODULE_PROTOCOL` and `EMBER_RELOAD_PROTOCOL` from `ember_rt.h` and fails when any version
  up to the current one has no row with a reason, or a row names a version above it: a version
  bump without its note fails CI.
- **ABI-5** (f1abeba6) — already true in the compiler; test `compiler/ember_driver/tests/reload.rs`
  `an_exports_c_abi_is_the_same_under_every_protocol`: a program's `@export("game_frame")` is the
  same C declaration (`int64_t game_frame(int64_t _1, double _2);`) with reload off and on, in
  debug, development and release; only the reloadable build names `ember_reload_protocol_vN`, N
  read from the runtime header (the protocol's own number, not the language's).
- **BLD-6** — built. `lto = "off" | "thin" | "on"`:
  - `ember_build::Lto`, `Profile::lto()` (`[PRF-3]`: debug off, development thin, release on),
    `[profiles.<name>] lto` in the manifest overrides it for a built-in profile (`set_lto`); any
    other word is `E9001`.
  - `Toolchain::supports_lto(lto, link)` / `Toolchain::lto(asked, link)`: the nearest kind the
    toolchain does; `Link::Here` (a program, a shared library: this build links) or
    `Link::Elsewhere` (a static library, `--emit obj`, `import c`/`import cpp` objects: another
    link may read them). cl: on, here only (`/GL` + `/LTCG`). gcc: on, here and elsewhere (fat
    objects, `-flto=auto -ffat-lto-objects`). clang-cl: thin and on, here (`-flto=thin`/`-flto`,
    already linked by `lld-link`). clang: thin and on, here, when `lld` is beside it or on `PATH`
    (`-fuse-ld=lld`; Apple's linker natively).
  - `/GL` left `msvc_flags`; every compile of the program's own C takes the build's flags
    (`lto_compile_flags`, `lto_link_flags`): the program's compile-and-link, the runtime object
    (cl with LTO compiles it with the program, as release did), library objects, cdylib links.
    Relaxed FP units never (as before). Hot-reload images and the shared runtime never: an
    image's functions are reached through thunks and never inlined (`[HR-9a]`), and the shared
    runtime's exports are read from its object's COFF symbols, which an LTCG object lacks (this
    also fixes a latent failure: a `cdylib` with `runtime = "shared"` in `release` compiled the
    shared runtime with `/GL` and could not read its symbols; its cache key moved to v2).
  - The build record: `target/<profile>/inspect/<name>.build.json`,
    `{"schema":1,"profile":…,"c_compiler":…,"lto":{"asked":…,"used":…,"why":…}}`, `why` set only
    when asked and used differ (or the C++ project's own flag decided, `[BLD-FFI-4]`).
  - Fixed on the way: an inherited `-fno-lto` (`[BLD-FFI-4]`) was passed to gcc/clang as an empty
    argument.
  - Tests: `ember_build` `link_time_optimisation_is_the_nearest_the_toolchain_does` (table and
    flags), `ember_driver/tests/package.rs`
    `link_time_optimisation_is_the_profiles_or_the_manifests_and_recorded` (each profile's own
    and the manifest's, the record, the same output each time, with cl the object really is
    LTCG exactly when the record says `on`; `E9001`). Passed with cl, clang-cl and clang; cdylib,
    staticlib, package, reload test files pass with cl. gcc is untested here (no gcc on this
    machine): CI's Linux gcc jobs now build `development` and `release` with `-flto=auto`; a
    warning gcc gives only at the LTO link would fail the `-Werror` C gate (`[TST-27]`), and would
    be a defect to fix in the emitted C.

- **BLD-10** — built: the budgets are a release gate.
  - `bench/bigpkg/generate.py` writes `[BUD-1]`'s package: 400 modules, ~48,600 lines, ~15.6 %
    generic code (generic structs, methods, bounded functions), `m000` holding the widely imported
    `Item`, a leaf module imported by `main` alone, and `import c "./vkimpl.c"` with a generated
    20,000-line header of Vulkan's shape; deterministic; `edit()` makes `[BUD-2]`'s edits (body,
    leaf signature, widely imported type), each new so no cache serves it.
  - `tools/compile_budgets.py`: B1–B9 as `[BUD-2]` lists them (`--list`), 3 warm-ups and the
    median of 15 runs, each figure published with its spread (IQR/median), a calibration workload
    before and after (more than 10 % drift: inconclusive, exit 2), times normalised to the
    reference machine's calibration (`bench/bigpkg/reference.json`, `--write-reference`), a figure
    failing above its budget or 15 % above `bench/bigpkg/baseline.json`; a figure with a spread of
    15 % or more is not gated (my reading of `[BUD-3b]`); a run past ten times its budget is ended
    and fails; exit 1 when any figure fails.
  - B9 needed the compiler's own peak memory: `--timings` / `--timings=json` now report it
    (`peak_memory_mb`: the peak working set on Windows, `VmHWM` on Linux).
  - `.github/workflows/release.yml`: on every `v*` tag (and by hand) the gate runs; release steps
    go after it.
  - Test: `milestones.rs` `the_compile_time_budgets_are_a_release_gate` (the tool's table equals
    `[BUD-2]`'s; a small bigpkg passes generous budgets and fails B1 = 0; the workflow runs it on
    tags).
  - Today the gate fails, and by far: see defect 1. B1 on the full bigpkg is ~19 s (budget
    250 ms); `ember build` of it did not finish in 15 minutes (11.5 GB).

## Not done: BLD-1, BLD-4, BLD-7, BLD-8 (each needs a decision first)

What the compiler is today: one whole-program front end (`ember_typeck::check` over every loaded
module, one `Checker` of ~800 lines of fields holding `&mut TypeTable`), then MIR passes over every
body, then **one C translation unit for the whole program** (std and dependencies included;
`ember_codegen_c::inline_bodies`: "the program is one translation unit"), compiled and linked in
one compiler invocation. `ember_build::interface` writes per-module interface records (`[BLD-2]`)
but its own header says the compiler "still checks a whole loaded module graph on every
invocation"; nothing reads a module's front-end result back to skip work.

- **BLD-4** (Ninja, incremental C). Meaningful only with more than one C unit: `[CG-C-3]` says each
  module is one translation unit with a per-package inline header; today there is one per
  program. Building it: the emitter writes a package header (types, prototypes, the inline
  bodies), one support unit (vtables, type infos, statics, the generated helpers: drop glue,
  `fmt`, `eq`, clone parts, array, SoA and thread helpers, today all `static` and emitted once) and
  one unit per module (its bodies, external linkage, namespaced in library mode); the reload,
  cdylib, staticlib, test, FFI-instrumentation and relaxed-FP paths follow; then a generated
  `build.ninja` per profile (compile edges per unit with `deps = msvc` / depfiles, link and archive
  edges), units written only when their text changes. Decisions: (a) the owner's C-speed rule —
  cross-module calls in `debug` stop being inlined by the C compiler; `development`/`release`
  rely on LTO (`[BLD-6]`, now on for cl) and the inline header; needs the laptop benchmark after;
  (b) `ninja` is only inside Visual Studio here and on no Linux runner by default: require it,
  ship it, or run the generated file with a small built-in executor. Estimate: several days.
- **BLD-1 / BLD-8** (module and function front-end caching). Skipping a clean module needs its
  front-end result back. For `ember check` that can be its diagnostics plus the facts its importers
  use (already in the interface record: signatures, effects, inline bodies, region summaries,
  layouts): clean modules' bodies need not be type-checked or borrow-checked. For `ember build` the
  back end needs every body of the package, so either (a) `check` only, `build` re-checks
  everything (then B3 cannot be met), or (b) each module's checked MIR is serialised and read back
  (types re-interned by canonical name, generic instances and closure types re-made), or (c) a
  query-based incremental front end. BLD-8 within a module additionally needs each body's
  record of which same-module facts it used (`[BLD-2]`'s rule applied per function) and the
  `Checker` able to check one function of a module (it has `body_filter` for `[TYP-32]` already).
  Decision: (a), (b) or (c). Estimate: (a) days, (b) a week or more, (c) weeks.
- **BLD-7** (parallel front end, `-j`, physical cores). Three blockers: `Symbol`'s order is its
  interning order (derived `Ord` on the index) and `BTreeMap<Symbol, _>`s decide output order, so
  parsing modules in parallel would make the output depend on thread timing (`[BLD-13]`) unless
  interning is made deterministic (lex in parallel to raw tokens, intern in module order, parse in
  parallel); `Symbol::as_str` locks one global mutex, so parallel passes would queue on it; and
  type checking is one `Checker` over all modules — checking bodies module by module in parallel
  means forking it per thread and merging new types (`TypeTable` ids), instances, lambdas and
  diagnostics back in a fixed order. The MIR passes (most of the front end's time on bigpkg) are
  per body and could run per module in parallel once the first two are solved. Decision: whether
  parallel parsing and MIR passes, with type checking still sequential, is worth building first
  (it would leave BLD-7 with an `unfinished` entry). Estimate: days for that part; the `Checker`
  split, weeks.
- Defect 1 below dominates all of these on any large function or package; it is worth fixing first.

## Compiler changes

- `compiler/ember_build/src/lib.rs`, `cdylib.rs`, `reload.rs`; `compiler/ember_driver/src/main.rs`
  (manifest `lto`, `set_lto`, `write_build_record`) — BLD-6 above.
- `compiler/ember_driver/src/main.rs` — `ember check` stops after the checks when no body has an
  `@simd(assert)` loop: the optimisation passes after them (inlining, range facts, loop versioning,
  the loop passes) report nothing but `[SIMD-2]`'s `E4020`. On a 10-module bigpkg, check went from
  14 s to 1.5 s. `--timings` marks four more stages (`inlining`, `checks the range facts remove`,
  `loop versioning`, `loop passes`) and the peak memory. The `[BLD-9]` test finds its JSON line by
  its `"timings"` key, not by its first key (serde_json orders keys).

## Defects found (for the lead to number)

1. **Per-function MIR passes are superlinear in a function's size, and single-call inlining makes
   one function the size of the program.** `ember check` of one function of N lines (repeated
   list loops; `scratchpad onefn.py`): 207 lines 1.3 s, 407 lines 4.4 s, 807 lines 113 s (loop
   versioning 98 s, borrow checks 6.9 s). In a 407-line function: `check_all_with_installed_callable_regions`
   0.83 s, `check_unused_all` 0.92 s, `insert_shared_accesses_all` 1.45 s, `install_callable_regions_all`
   0.47 s, `remove_proven_checks_all` 1.49 s, `version_bounds_checked_loops_all` 0.74 s. Then
   `inline_single_calls_all` inlines every function called once into its caller with no growth
   limit (for MSVC every such one), so `main` of bigpkg becomes the whole program: a 10-module,
   838-line bigpkg spent 2.2 s inlining, 5.5 s in range facts and 5.8 s in loop versioning; the
   400-module one did not build in 15 minutes and used 11.5 GB (17.8 GB for `ember check` before
   the change above). Needed: a growth limit on single-call inlining (as GCC's
   `max-inline-functions-called-once-insns` and `large-function-growth`), and the dataflow of
   these passes made linear-ish (sparse states). Not fixed here: it is the analysis crate's
   design, outside these rules; it blocks every bigpkg figure but B1 and B8.
2. The front end on bigpkg (no inlining): `ember check` ~19 s, of which "move, drop, borrow checks"
   8.2 s across ~2,500 small functions; `prepare_callable_interface_cache` is about 2.4 s of it.

## Spec questions (for the lead)

1. `[BLD-6]` "the nearest one it does": `thin` with a toolchain that does only `off` and `on`
   (cl, gcc) is equidistant. Options: (a) `on` (built: thin LTO's purpose is the optimisation; no
   correctness depends on it), (b) `off` (cheaper development builds), (c) per toolchain.
   Recommendation (a). Effect: MSVC `development` builds now use `/GL` + `/LTCG` (slower links).
2. `[BLD-6]` "the build record" is defined nowhere. Built as `inspect/<name>.build.json`.
   Recommendation: name it in the spec (XVII.3), with its keys.
3. `[BLD-6]` objects for a link the build does not run (a static library): built as no LTO with
   cl (a `/GL` object holds no code, binds the archive to that compiler's version and turns the
   reader's link into LTCG; before, `release` archives were `/GL`), fat objects with gcc, none with
   Clang. Option: keep `/GL` in MSVC archives (cross-boundary inlining for an LTCG consumer, at the
   cost of the above; the CMake test's `/MDd` evidence cannot be read from such an archive).
   Recommendation: as built. ADR-067's passing remark ("retaining profile checks and LTO flags")
   no longer holds for MSVC archives.
4. `[BUD-3b]` "a gate applies only to a figure whose measured spread is below the gate": built as
   "spread (IQR/median) below 15 %" for both gates. Options: per gate (the baseline gate's 15 %, the
   budget gate's distance from the budget); or a fixed spread. Recommendation: as built, and say
   which spread measure in the spec.
5. `[BUD-1]` "a 50k-line package … one C header import of Vulkan's size": the generated header is
   20,000 lines of Vulkan's shape (enums, structs, function pointer types, prototypes); the
   package's lines exclude it. `[BUD-2]` B6 "cold cache" is read as a new, empty global build
   cache (`EMBER_CACHE`), so the MSVC environment and the runtime object are made again too; B7 and
   B9 turn the program cache off (`EMBER_NO_COMPILE_CACHE`), else a clean build of an unchanged
   package is a cache hit.
