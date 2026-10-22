# Helper AF — the LLVM backend (ODR-223), notes

Branch `worktree-agent-a7590733616cc8073`. The lead merges.

## Design as built

- `compiler/ember_codegen_llvm`: verified MIR + the type table's layouts → one textual `.ll`
  module. No LLVM library in the compiler; the `.ll` is a source of the same Clang command that
  compiles the runtime and links (`ember_build::compile_and_link_program`), so the profile's
  `-O` level, `-g` and LTO flags apply to it as to the C backend's C.
  - `abi.rs` — the platform C ABI's classification (Windows x64, System V x86-64), as Clang
    does it, register counting included. Every Ember function uses it, so runtime calls,
    function values, callbacks and exports are one convention. Unit test against Clang 22
    (Windows) and Clang 21 (Linux) lowering of the same prototypes.
  - `types.rs` — each type is a register scalar (`Sc`), nothing, or bytes in memory with the
    type table's size and alignment; places are byte offsets (no IR struct types, so no second
    copy of C's layout rules).
  - `runtime.rs` — the runtime entry points the IR calls, with their C prototypes; a unit test
    reads `ember_rt.h` and compares every prototype.
  - `func.rs` — bodies: every local a stack slot (promoted by `mem2reg`/SROA at `-O1`+),
    parameters passed behind a pointer used as the callee's own slot, `sret` results written to a
    fresh slot then copied (memcpyopt removes the copy where safe).
- `[CG-LL-2]`: checked arithmetic through `llvm.{s,u}{add,sub,mul}.with.overflow`; the checked
  division forms divide by 1 where `MIN / -1` would be (so no IR divides it), with the C
  helpers' flags exactly (`ck_rem` reports `MIN % -1`, `ck_floorrem` does not); shift amounts
  masked to the width (no poison); float→int through `llvm.fpto{s,u}i.sat` (`[TYP-6]`); no
  `nsw`/`nuw`/`inbounds`/`noundef`; fast-math flags only under `@fastmath`/`@fp(contract)`.
- Driver: `--backend c|llvm`, `[build] backend = "llvm"` (`compiler/ember_driver/src/llvm.rs`,
  one branch in `compile()` before the C emission). `--emit llvm-ir` prints the module. Clang is
  required (`Toolchain::detect(Some("clang"))`); a missing one is `E9002` with
  `winget install --id LLVM.LLVM` / `sudo apt install clang lld`. `--cc` other than clang is
  refused. Libraries, reload images and `ember test` are refused as not built yet.
- `ember_build`: a `.ll` source adds `-Wno-override-module -Wno-unused-command-line-argument`
  (the module names no triple; it reads none of the C flags the command shares). Nothing else
  changes for the C backend.
- `tasks/impl-0.9.9/annotations.py`: `EMBER_BACKEND=llvm` runs only the run tests, built with
  `--backend llvm`, and prints `backend llvm: N run tests, M passing`.

## Ledger (run-pass + run-fail unless said)

| date | commit | passing | of |
|---|---|---|---|
| 2026-10-23 | step 1 base | 31 | 182 |

Causes (step 1 base): `class` construction 67, interface calls 32, `Array` 19, `Box[dyn]` 14,
the rest single built-ins. No wrong results.

## Done

## Not done (and why)

## Defects found

## Spec questions
