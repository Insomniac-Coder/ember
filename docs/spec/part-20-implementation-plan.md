# Part XX — Implementation Plan

## XX.1 Ground rules for the implementing agent

1. Work in the order of the phases below. Do not start a phase's optional items before its **exit criteria** pass.
2. Every phase adds tests before code: write the `tests/conformance/<rule>/` cases from the spec text, then implement until they pass.
3. Keep `docs/DECISIONS.md` (ADR format: context, decision, consequences, spec rule affected). Any deviation from this document requires an ADR and a spec patch in `docs/spec-errata.md`.
4. Keep `docs/HANDOFF.md` in the RageV style: current state, what is done, what is not, load-bearing invariants, traps paid.
5. `cargo test` must be green at every commit; `ember test` on `std/` must be green from Phase 3 onward.
6. Prefer boring implementations. No novel algorithms where a textbook one exists (Pratt parsing, union–find inference, Maranget pattern compilation, NLL).
7. Do not build the LLVM backend, async, named lifetimes, procedural derives, or the kernel language in v1 even if they seem easy at the time.

## XX.2 Phases

### Phase 0 — Skeleton (exit: `hello.em` compiles via C and runs on Windows + Linux)

* Workspace, crates, CI (GitHub Actions: windows-latest with MSVC + clang-cl, ubuntu-latest with clang + gcc).
* `ember_span`, `ember_diag` (rendering + JSON), `ember_lexer` with the indentation algorithm (`[LEX-*]` tests), `ember_parser` for functions, calls, literals, `if`/`while`/`for`, structs (no generics), `ember_ast` pretty-printer.
* Minimal `ember_types`/`ember_typeck` for scalars, structs, `void`, function calls, locals.
* Straight-line MIR lowering (no borrowck yet), `ember_codegen_c`, `ember_rt` with `ember_alloc`, `ember_panic`, `println` for scalars/`str`.
* `ember build/run` driving `cl.exe`/`clang`.
* **Milestone test**: M1 (§XX.3; `Vec3` add) compiles to C with no heap allocation (assert by grepping the C for `ember_alloc`) and prints `5`.

### Phase 1 — Core language (exit: conformance for Parts II–VI except closures/generics)

* Full grammar (`[GRM-*]`), error recovery, formatter (`[FMT-1]`).
* Enums, `match` with decision trees + exhaustiveness, tuples, fixed arrays, `Option`/`Result` as real enums, `?`, ranges, `while/for/else`, labeled break, `with`, `defer`, f-strings (allocating `format`).
* Interfaces (no generics yet beyond `Self`), operator interfaces, method resolution, `extend`, visibility, modules/imports across files, `const`, `static` (comptime-initialised only via literal for now).
* Integer/float semantics (`[TYP-4..10]`), `as`, overflow policies, `Assert` lowering.
* `String`/`str` with SSO, `Array[T]` implemented as a compiler-known type temporarily (replaced by generic std implementation in Phase 2).
* Definite-init analysis (`[CLS-2]`-style for locals).

### Phase 2 — Ownership (exit: all `[OWN-*]`, `[BRW-*]`, `[LT-*]`, `[DRP-*]`, `[SPN-*]`, `[CELL-*]`, `[DIA-7..10]` tests; milestone M2; **zero unclassified borrow errors across the whole test corpus**)

* Generics with bounds, monomorphisation, associated types, `Iterator`/`Iterable` and the adaptor set; `Array`, `Span`, `MutSpan`, `Box`, `Map` written in Ember (`std/collections`).
* Moves, `Copy`, drop elaboration with drop flags, `Drop` interface, `mem.*`.
* NLL borrow checker (§XVIII.4.7) incl. two-phase borrows, disjoint fields, reborrows, view structs `@view`, elision rules, `@borrows`.
* Closures (`[CLO-*]`), `Callable`, `fn(A)->R` generic parameters.
* `Arena`, `FixedArena`, `ScopedArena` (`[ARN-*]`), `unsafe`, raw pointers, `MaybeUninit`, `transmute`.
* `Cell[T]`, `RefCell[T]`, `Ref`/`RefMut` guards (`[CELL-*]`), `std.cell`.
* Diagnostics quality pass on borrow errors: the full shape catalogue of §XIX.6.1 with a `ui/borrow/<shape>/` snapshot each (`[DIA-3]`, `[DIA-7]`, `[DIA-10]`), the classifier, and `ember explain --borrow` (`[DIA-8]`).

### Phase 3 — Objects (exit: `[OBJ-*]`, `[RC-*]`, `[EXC-*]`, `[DSP-*]`, `[WK-*]` tests; leak/cycle report works)

* Classes: header, `init` with definite-init, inheritance, `virtual`/`override`, `abstract`, `let` fields, `is`, `as?`/`as!`, `dyn` for classes and structs (vtables, itables).
* Retain/release insertion; `[RC-2]` guaranteed elisions as MIR passes with tests that count `ember_retain` in emitted C.
* Exclusivity: static analysis (§4.8) + runtime checks; `Shared[T]`, `Weak[T]`.
* `debug_objects` live list, `ember run --leak-check` cycle report, `L3001`.
* Stack promotion (`[OPT-1]`) — optional, behind `-Zstack-promote`.

### Phase 4 — Effects, comptime, derives (exit: `[EFF-*]`, `[CT-*]`, `[RFL-*]`, `[DRV-*]`)

* Effect inference + `@noalloc`/`@nosync`/`@noblock` with chain diagnostics; `ember inspect`.
* MIR interpreter, `comptime` blocks/functions, `const` evaluation, comptime-initialised `static`s, `reflect`, `size_of`/`offset_of`, `@gpu_layout` assertions (`[GPU-10]` layout part).
* Derives: `Copy, Clone, Debug, Display, Eq, Ord, PartialOrd, Hash, Default, Zeroable, Reflect, Error`.
* `std.ser.binary`, `std.ser.yaml` with `Serialize/Deserialize` derives.

### Phase 5 — C FFI (exit: `[FFI-1..16, 21..28]` tests; Vulkan header imports; milestone M5)

* `ember_ffi` with `clang-sys`: BIR, type mapping table, macros, bit-fields, unions, opaque types; `.embind` CBOR cache; MSVC-compat parsing.
* Overlays (`[FFI-11..13]`), safe wrappers, `status`/`handle`/`span`/`out` contracts, `cstr`/`CString`, callbacks with `user_data` trampolines, `Retained`, thread attach.
* `@export`, `--emit-header`, `cdylib`/`staticlib` kinds, embedding API (`[FFI-27]`), `@export_table`.
* `cmake/EmberModule.cmake`, CMake File API flag import, Ninja-driven C compilation.
* Fixtures: `vulkan/vulkan.h` (with `VK_NO_PROTOTYPES`), `GLFW/glfw3.h`, `miniaudio.h` (single header, macro-heavy: the stress test), `cgltf.h`.

### Phase 6 — Concurrency and DOD (exit: `[THR-*]`, `[JOB-*]`, `[PAR-*]`, `[SOA-*]`, `[SIMD-*]`, `[ECS-*]`; milestone M6 with benchmark vs C++)

* `Send`/`Sync` derivation, `thread`, `Mutex`, `Atomic`, channels, scoped threads.
* Job system with work stealing, access-set scheduling, job-local arenas.
* `@parallel for` lowering + disjointness proof; reductions.
* `@derive(SoA)`, `columns_mut`, proxies; `std.simd` types + `ember_simd.h`; `restrict` emission from borrow facts; `@simd` diagnostics.
* `std.ecs` per Part XII §3.
* Performance suite (§4) established with C++ references.

### Phase 7 — C++ FFI and GPU host model (exit: `[FFI-17..20, 24]`, `[GPU-*]`; RageV Stage 2–3 in Part XXI)

* C++ importer: thunk generation, MSVC ABI matching, template instantiation lists, STL views, exceptions → `Result[..,CppError]`, `ember bind --report`.
* `std.gpu`: handles, `Device` interface, `Frame`, `Ring`, access states, deferred destruction, `History`, `ShaderInterface`, `ember shader-bind` from SPIRV-Cross JSON.
* `std.gpu.graph` (may be deferred to v1.1 if RageV's own graph is used through FFI).

### Phase 8 — Hardening and 1.0 (exit: full conformance + perf suite + two external packages)

* `ember lint` full set, `ember doc`, doc tests, `ember explain`.
* Fuzzing targets, differential testing against the interpreter.
* Package registry client (v1.1), LSP (v1.1), LLVM backend (v2), PGO/ThinLTO (v2).

## XX.3 Milestone acceptance tests (must exist verbatim in `tests/milestones/`)

**M1 — value code has no runtime cost** (Phase 0/1):
```ember
struct Vec3:
    x: f32
    y: f32
    z: f32

fn add(a: Vec3, b: Vec3) -> Vec3:
    return Vec3(a.x + b.x, a.y + b.y, a.z + b.z)

fn main():
    c = add(Vec3(1, 2, 3), Vec3(4, 5, 6))
    println(c.x)
#$ stdout: 5
#$ assert-c: !contains("ember_alloc")
```

**M2 — the borrow checker catches escapes** (Phase 2):
```ember
#$ test: compile-fail
fn first(xs: Array[i32]) -> ref i32: return xs[0]          # ok: tied to xs
fn bad() -> ref i32:
    xs = Array[i32]([1, 2])
    return xs[0]                                              #$ error[E3060]: borrowed value does not live long enough
```

**M3 — objects are deterministic** (Phase 3):
```ember
class Res:
    fn drop(mut self): println("drop")

fn main():
    a = Res()
    b = a
    mem.drop(a)          # only the handle
    println("mid")
    mem.drop(b)          # object dies here
    println("end")
#$ stdout: mid\ndrop\nend
```

**M4 — contracts are transitive** (Phase 4):
```ember
#$ test: compile-fail
fn helper(mut xs: Array[i32]): xs.push(1)

@noalloc
fn hot(mut xs: Array[i32]): helper(xs)                      #$ error[E4001]: @noalloc function `hot` reaches an allocation: hot -> helper -> Array.push -> ember_alloc
```

**M5 — C is free** (Phase 5): import a header with `int calculate(int)`; the emitted C for the call site is a direct call (assert-c contains `calculate(` and no wrapper symbol).

**M6 — DOD matches C++** (Phase 6): `SoA[Particle]` integrate over 1M particles ≤ 1.10× the time of the reference `perf/particles.cpp` compiled with the same C++ compiler at `/O2` or `-O2`.

**M7 — C++ classes bind** (Phase 7): construct/call/destroy a C++ class through generated thunks; a thrown `std::runtime_error` surfaces as `Err(CppError)`; MSVC and clang-cl both pass.

## XX.4 Performance suite

`tests/perf/` holds paired `.em`/`.cpp` programs and thresholds; `ember bench --compare` runs both with the same C++ compiler, 10 iterations, reports median. Gates: scalar/tight loops ≤ 1.05×, SoA/SIMD ≤ 1.10×, FFI call overhead = 0 extra instructions for ABI-direct calls (asm diff), RC-heavy object code ≤ 1.3× of equivalent Swift-style hand-written C++ with `shared_ptr` (informational, not gating), allocation counts asserted exactly for `@noalloc` paths.

## XX.5 Repository layout

```
ember/
  Cargo.toml                 workspace
  compiler/…                 crates per §XVIII.1
  runtime/ember_rt/          C11 runtime, CMakeLists.txt, vendored mimalloc
  std/                       ember.toml + src/**.em
  tools/                     fmt, lint, shader-bind, rule_index.py, perf harness
  cmake/EmberModule.cmake
  tests/                     per §XIX.5
  docs/
    spec/                    THIS DOCUMENT split by part, plus spec-errata.md
    DECISIONS.md  HANDOFF.md  errors/EXXXX.md  book/ (user guide, v1.1)
  examples/                  hello, particles, ecs_demo, vulkan_triangle (via RageV RHI or raw volk)
```

---
