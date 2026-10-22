# Helper Q — Parts XVI/XVII builds and ABI (2026-10-22)

Rules: BLD-1, BLD-4, BLD-6, BLD-7, BLD-8, BLD-10, ABI-4, ABI-5.

## Done

- **ABI-4** — `docs/RELEASE-NOTES.md` (new) notes every version of each compiler protocol and why
  it changed: runtime ABI 1, 2 (8ece921b, `ember_rt_config.size` + the caller's version), 3
  (b42a5386, `ember_type_info.reflect`); module protocol 1; hot-reload protocol 1. An export
  table's protocol (`Name_vN`, `[FFI-26]`) is the package author's, so the package's release
  notes it. Test: `compiler/ember_driver/tests/cdylib.rs`
  `every_protocol_version_is_in_the_release_notes` reads `EMBER_RUNTIME_ABI`,
  `EMBER_MODULE_PROTOCOL` and `EMBER_RELOAD_PROTOCOL` from `ember_rt.h` and fails when any version
  up to the current one has no row with a reason, or a row names a version above it: a version
  bump without its note fails CI.
- **ABI-5** — already true in the compiler; test `compiler/ember_driver/tests/reload.rs`
  `an_exports_c_abi_is_the_same_under_every_protocol`: a program's `@export("game_frame")` is the
  same C declaration (`int64_t game_frame(int64_t _1, double _2);`) with reload off and on, in
  debug, development and release; only the reloadable build names `ember_reload_protocol_vN`, N
  read from the runtime header (the protocol's own number, not the language's).

## Compiler changes

None.
