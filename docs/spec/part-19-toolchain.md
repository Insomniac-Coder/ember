# Part XIX — Toolchain

## XIX.1 CLI

```
ember new <name> [--lib | --bin | --cdylib]        scaffold a package
ember build [--profile debug|release|shipping] [--target <triple>] [--backend c|llvm] [--emit tokens|ast|hir|mir|c|obj]
            [--emit-header] [--emit-optimization-report] [--explain-performance] [-Dwarnings]
ember run [args...]        build + run the binary
ember test [filter] [--doc]      run @test functions (and doc tests)
ember bench [filter]
ember check                type-check + borrow-check without codegen
ember fmt [--check]
ember lint
ember inspect <path.to.item>       effects, layout, allocation sites, inlining, ABI
ember bind <header> [--emit-embind | --explain <symbol> | --report | --emit-cpp]
ember shader-bind <reflect.json>
ember doc
ember clean
ember toolchain {list|install|default}
```

`[CLI-1]` Every command supports `--json` for machine-readable output (diagnostics, inspect, report) and exits non-zero on error. `[CLI-2]` `ember build --emit=c --out-dir <dir>` writes the C sources without invoking a C compiler (used by the CMake integration).

## XIX.2 Manifest (`ember.toml`)

```toml
[package]
name = "ragev_scripts"
version = "0.1.0"
language = "0.2"
kind = "cdylib"                     # bin | lib | staticlib | cdylib
edition_lints = "strict"           # "strict" turns W-level into errors listed in [lints]

[dependencies]
std = { version = "0.2" }           # implicit; may pin
ragev_api = { path = "../ragev_api" }
some_lib = { git = "https://…", rev = "…" }
math_ext = "1.2"                    # registry (v2)

[build]
entry = "src/lib.em"
backend = "c"                       # c | llvm
c_compiler = "auto"                 # auto: msvc on Windows if cl.exe found, else clang
target = "native"

[profiles.debug]
opt = 0
overflow = "panic"
bounds_checks = true
exclusivity = "checked"
debug_objects = true                # live object list, leak/cycle report
sanitizers = ["address", "undefined"]   # when the C compiler supports them

[profiles.release]
opt = 2
overflow = "wrap"
bounds_checks = true
exclusivity = "checked"
lto = "thin"

[profiles.shipping]
inherits = "release"
opt = 3
exclusivity = "unchecked"           # UB on violation; only allowed here
strip = true
panic = "abort"

[cpp.ragev]                          # a C++ project Ember interoperates with
compiler = "msvc"                    # msvc | clang-cl | clang | gcc
standard = "c++20"
cmake = { build_dir = "../RageV/build", target = "RageV" }   # flags read from the CMake File API / compile_commands.json
defines = ["RV_PLATFORM_WINDOWS"]
include_paths = ["../RageV/RageV/src", "../RageV/RageV/vendor/glm"]

[comptime]
max_steps = 100000000
max_heap_mb = 256

[lints]
unused = "warn"
potential_cycle = "warn"
large_copy = { level = "warn", threshold = 256 }
```

`[MAN-1]` Unknown keys are errors. `[MAN-2]` `ember.lock` records the resolved dependency graph with content hashes; `ember build --locked` fails if the lock would change.

## XIX.3 Build graph and caching

* `[BLD-1]` Unit of compilation and caching: the **module** (one `.em` file) for front-end stages; the **package** for monomorphisation and codegen (one C file per module is emitted, but instantiations are placed in the module that first requests them, with COMDAT-style `static inline`/weak linkage to dedupe).
* `[BLD-2]` Cache key of a module's front-end artefact: BLAKE3 of (source, compiler version, language version, package config, transitive **interface hashes** of imported modules — the hash of exported signatures, types, layouts, effect sets and inline bodies, not of private bodies). A change to a private function body recompiles only its module's codegen and any callers' *effect checks* if its effect set changed (`[EFF-4]`).
* `[BLD-3]` `.embind` files are cache entries keyed on header hash + flags + overlay hash.
* `[BLD-4]` The C compiler and linker are invoked through a Ninja file generated per build (`target/<profile>/build.ninja`) so that incremental C compilation is handled by Ninja; MSVC is driven with `/showIncludes`, Clang/GCC with `-MD`.
* `[BLD-5]` Output layout: `target/<profile>/{bin,lib,c,obj,bind,inspect}`.

## XIX.4 Profiles

The profile controls: optimisation level, overflow policy, bounds checks (never disableable in safe code — `bounds_checks = false` only affects `unsafe`-opted `get_unchecked` hints and is honoured by no v1 profile), exclusivity checks, `debug_objects`, sanitizers, LTO, `panic` policy, `strip`. `[PRF-1]` A profile MUST NOT change program semantics except: overflow policy, panic-vs-UB for `exclusivity = "unchecked"`, and presence of debug facilities.

## XIX.5 Test harness and conformance suite layout

```
tests/
  conformance/<rule-id>/*.em          one directory per normative rule ID, e.g. tests/conformance/BRW-3/two_phase_push.em
  compile-pass/*.em                   must compile, not necessarily run
  compile-fail/*.em                   must fail with the annotated errors
  run-pass/*.em                       compile, run, compare stdout/exit code
  run-fail/*.em                       compile, run, expect a panic with a given message
  ui/*.em + *.stderr                  full diagnostic text snapshots
  ffi/**                              C/C++ interop fixtures (headers + .em + expected)
  perf/**                             benchmark programs with C reference implementations and thresholds
  std/**                              standard library unit tests (@test functions)
```

Annotation format inside test files (`[TST-*]`):

```ember
#$ test: compile-fail
#$ rules: BRW-1, OWN-3
fn main():
    a = Array[i32]()
    b = a
    a.push(1)          #$ error[E3040]: use of moved value `a`
                       #$ note: value moved here @ line-1
```

Annotations begin with `#$`. To the compiler that is an ordinary comment
(`[LEX-10]`), so an annotation never affects compilation and an annotated file
is a valid compilation unit. `$` carries no other meaning anywhere in Ember,
which is why it was chosen: no annotation can ever be confused with source.

* `[TST-0]` The test harness reads annotations from the **raw source text**, not
  from the token stream. A `compile-fail` test may be expected to fail at the
  lexer, so its expectations must be readable even when the file does not
  tokenise.
* `[TST-1]` `#$ error[EXXXX]: <message-substring>` on a line asserts an error with that code whose primary span starts on that line; `#$ warning[...]`; `#$ note` for secondary labels (`@ line±n` for relative positioning). Unexpected diagnostics fail the test; missing expected ones fail the test.
* `[TST-2]` `run-pass` files may contain `#$ stdout:` followed by `#$` lines of expected output, and `#$ exit: N`. `#$ assert-c: contains("…")` and `#$ assert-c: !contains("…")` assert against the emitted C.
* `[TST-3]` `@test` functions in `std/` and user packages run in-process in the test binary, each in isolation (panics captured with `abort` replaced by a longjmp-based harness in the test runner build only; with the unwind policy, normal catch).
* `[TST-4]` Every rule ID in this document appears in `tests/conformance/INDEX.md`, generated by `tools/rule_index.py` from the spec, which fails CI if a rule has no test directory.

## XIX.6 Diagnostics

Format (human):

```
error[E3040]: use of moved value `ps`
  --> src/main.em:12:11
   |
10 |     n = take(ps)
   |              -- value moved into `take` here
11 |     print(n)
12 |     print(ps.len())
   |           ^^ value used here after move
   |
   = help: if you need `ps` afterwards, pass a clone: `take(ps.clone())`
   = note: `Array[Player]` is not Copy because it owns heap memory
```

* `[DIA-1]` Every diagnostic has: code, one primary span, zero or more secondary spans with labels, optional `help` (with a machine-applicable fix-it when possible) and `note`. `--json` emits the same structure.
* `[DIA-2]` Message style: lowercase first letter, no trailing period, name the thing (`` `ps` ``), say what is wrong and, in `help`, what to do. Never say "you"; never blame the user.
* `[DIA-3]` Borrow/ownership diagnostics MUST include the "later used here" label (`[BRW-2]` explanation) and a concrete fix (Part XVIII §4.7 step 7).
* `[DIA-4]` Contract diagnostics MUST print the full call chain (`[EFF-5]`).
* `[DIA-5]` FFI diagnostics name the header and the C declaration (`vkCreateBuffer in vulkan_core.h:1234`).

### Error-code registry

| Range | Subsystem |
|---|---|
| E0000–E0099 | lexer / indentation / directives |
| E0100–E0499 | parser |
| E1000–E1499 | name resolution, modules, visibility |
| E2000–E2499 | types, inference, interfaces, generics, patterns |
| E3000–E3499 | ownership, moves, borrows, regions, exclusivity (static), drops |
| E4000–E4499 | effects and contracts (`@noalloc` …), SIMD/parallel contracts |
| E5000–E5499 | FFI (import, overlays, layout mismatch, callbacks, C++) |
| E6000–E6499 | comptime |
| E7000–E7499 | concurrency (`Send`/`Sync`, parallel loops, ECS scheduling) |
| E8000–E8499 | layout attributes, GPU layout |
| E9000–E9499 | build system, manifest, toolchain |
| W-codes | same ranges, warnings |
| L-codes | lints (`ember lint`) |
| E-panic | runtime panics (documented messages, not compile-time codes) |

`[DIA-6]` `docs/errors/EXXXX.md` exists for every code with an example and its fix; `ember explain E3040` prints it.

## XIX.7 Formatter

`ember fmt` is deterministic and configuration-free except line width (default 100). Rules: 4-space indentation; one blank line between methods, two between items; trailing commas in multi-line argument/element lists; spaces around binary operators, none around `**` when both operands are atoms; `x: T = v` spacing; imports sorted (`std` first, then dependencies, then local) and merged; attributes one per line; the formatter preserves comments and blank-line groups (max 2 consecutive); `pass` inserted for empty blocks; long conditions broken after `and`/`or`. `[FMT-1]` `fmt(fmt(x)) == fmt(x)` and `parse(fmt(x)) ≡ parse(x)` are tested over the whole test corpus.

## XIX.8 Linter (`ember lint`, L-codes)

v1 lints: `L2001 unnecessary clone` (source not used again), `L2002 large Copy` (> threshold bytes passed by value), `L3001 potential cycle` (statically visible class cycles), `L3002 borrow held longer than necessary` (a borrow whose last use is far before its scope end and blocks a later access — suggests a block), `L3010 unsafe block larger than necessary`, `L4001 allocation in hot loop` (allocation inside a loop of a `@simd`/`@parallel` body or inside functions named in `[lints.hot_paths]`), `L4002 dynamic dispatch on final type` (redundant `dyn`), `L5001 unsafe extern without contract`, `L5002 FFI copy` (conversion at the boundary copying > threshold bytes), `L7001 lock held across call that may block`.

## XIX.9 Documentation

`ember doc` renders `##` doc comments to HTML/Markdown; every public function's page shows: signature with modes, **effects**, `@noalloc`-cleanliness, thread rules (`Send`/`Sync` of parameters and result), allocation behaviour (from effect analysis), and for FFI wrappers the underlying C declaration and contract. Doc examples in fenced ```` ```ember ```` blocks are compiled and run as tests by `ember test --doc`.

---

