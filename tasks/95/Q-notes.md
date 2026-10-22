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

## Compiler changes

- `compiler/ember_build/src/lib.rs`, `cdylib.rs`, `reload.rs`; `compiler/ember_driver/src/main.rs`
  (manifest `lto`, `set_lto`, `write_build_record`) — BLD-6 above.

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
