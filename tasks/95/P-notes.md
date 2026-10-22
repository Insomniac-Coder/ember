# Helper P — Annex B (hot reload), 2026-10-22

Time box 40 minutes; one rule built and tested.

## Done

* **HR-23** — built and tested.
  `compiler/ember_driver/tests/reload.rs`, `a_changed_cpp_header_refuses_the_reload_and_an_overlay_does_not`:
  a reload built after `engine.hpp` changed is refused in plan, naming the header, and the old
  image keeps running (`30\n30\n`); a reload built after only the overlay changed applies the
  edited body beside it (`30\n3000\n`). `a_trampoline_reaches_an_edited_override_after_a_reload`
  still passes (the previous thunks are reused where the header is unchanged).

## Compiler changes

* `compiler/ember_driver/src/cimport.rs` — `import_cached` also returns a content hash over every
  file the header included (the `.embind` record's file list, each with its content hash), on a
  cache hit and on a parse alike.
* `compiler/ember_typeck/src/lib.rs` — `ForeignImport.header_hash` (for `import cpp` only).
* `compiler/ember_driver/src/reload.rs` — `plan` collects each C++ header and its hash into
  `ReloadImage.headers`.
* `compiler/ember_codegen_c/src/reload.rs` — the manifest carries a `headers` table.
* `runtime/ember_rt/include/ember_rt.h` — `ember_rl_header`; `ember_rl_manifest` gains
  `header_count` and `headers` (appended; images and runtime are rebuilt together, and the
  manifest's `size` check already rejects a mismatch, `E9037`).
* `runtime/ember_rt/src/ember_rt.c` — `g_rl.running`, the manifest applied last; `rl_plan` refuses
  a header whose hash differs from the running image's (`refused: C++ header `X` changed; the host
  must be rebuilt against it [HR-23]`).

## Not done (out of time)

* **HR-41** — the failure matrix: every row but "C++ header ABI changed" already had a test; that
  row now has one (above). A test naming `[HR-41]` that walks the rows (or citing it beside each
  existing row test) is all that remains, plus the GPU row, which waits on HR-24.
* **HR-1** — needs `bench/bigpkg` (`[BUD-1]`: generated 50k-line package, 400 modules, 15 %
  generic, a Vulkan-sized C header) and a timing on the reference machine. Not started.
* **HR-9** — a measurement: the performance suite with reload on vs off, ≤ 3 %, reported with
  its configuration. Not started.
* **HR-13** — value-typed data in containers is refused today (`rl_plan_type`: "values of `X` sit
  in containers ... whose elements a reload cannot migrate yet [HR-13]"); container registration
  with the element schema is not built.
* **HR-14** — unchanged; the `unfinished` entry's three rows still stand.
* **HR-20** — `Retained`'s `on_relocate` callback and the pinned-object refusal are not built (no
  `on_relocate` anywhere).
* **HR-22** — not checked (a `ForeignBox` field carried across a reload; likely holds as an
  ordinary field, needs a test).
* **HR-24** — std has no GPU module or in-flight frame concept; nothing to build against.
* **HR-39**, **HR-43** — `ReloadError.Foreign` exists in `std.hot`, but a foreign call inside
  `migrate_from` is not converted to it, the `noexcept` C++ call is not rejected, and
  `@allow_reload_terminate` is only registered as an attribute (no effect, not in `ember tcb`).

## Defects found

None beyond the gaps above.

## Spec questions

1. **HR-23: what counts as a changed header.** Built: any change to the header or a file it
   includes (content hash), so a comment-only edit also refuses. Options: (a) keep this, the
   literal "a changed C++ header"; (b) refuse only when the binding changes (declarations,
   layouts, the shim), which `[HR-41]`'s "C++ header ABI changed" suggests, but an inline body the
   host compiled would then differ silently from the image's. Recommendation: (a).
2. **HR-23 for C headers.** The rule names C++ only, so a changed `import c` header is not
   refused. The same host-rebuild reason applies to C. Options: (a) C++ only, as written; (b)
   extend to `import c`. Recommendation: (b), by a spec change.
