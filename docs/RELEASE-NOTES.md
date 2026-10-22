# Ember release notes

## Protocol versions (`[ABI-4]`)

`[ABI-1]` versions the compiler's protocols independently of the language (`[ABI-5]`); each
number is in a symbol's name, so a mismatch fails at link time or on load. Every version of every
protocol has a row here saying why it changed; `cdylib.rs`'s
`every_protocol_version_is_in_the_release_notes` fails when a header's number has no row. The
current numbers are the macros in `runtime/ember_rt/include/ember_rt.h`.

An export table's protocol (`@export_table("Name", protocol=N)`, `Name_vN`, `[FFI-26]`) is its
package's own: the package's release notes it.

| Protocol | Version | Symbol | Why it changed |
|---|---|---|---|
| runtime ABI (`EMBER_RUNTIME_ABI`) | 1 | `ember_rt_abi_version()` | The first runtime ABI: the object header, `ember_type_info` and the entry points of `ember_rt.h` (`[VER-4]`). |
| runtime ABI (`EMBER_RUNTIME_ABI`) | 2 | `ember_rt_init_v2` | `ember_rt_config` gained its leading `size`, and `ember_rt_init` carries the caller's version, so a mismatch is refused (`[ABI-1]`, `[ABI-3]`; 2026-10-12, 8ece921b). |
| runtime ABI (`EMBER_RUNTIME_ABI`) | 3 | `ember_rt_init_v3` | `ember_type_info` gained `reflect`, a `@reflect` class's description (`[RFL-2]`, `[RT-3]`; 2026-10-20, b42a5386). |
| module protocol (`EMBER_MODULE_PROTOCOL`) | 1 | `ember_module_protocol_v1` | The first: how a host starts and stops a shared library, `ember_module_init` and `ember_module_shutdown` (`[FFI-28]`, `[FFI-31]`, ODR-131). |
| hot-reload protocol (`EMBER_RELOAD_PROTOCOL`) | 1 | `ember_reload_protocol_v1` | The first: a host's and an image's contract for loading, polling and applying images (Annex B, `[ABI-2]`). |
