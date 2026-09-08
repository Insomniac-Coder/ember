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
ember explain <EXXXX>              print the error's reference page
ember explain --borrow <file>:<line>   why a borrow at that line is still live (region as line ranges)
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
* `[DIA-3]` Borrow/ownership diagnostics MUST include the "later used here" label (`[BRW-2]` explanation) and a concrete fix drawn from the catalogue in §XIX.6.1 (`[DIA-7]`).
* `[DIA-4]` Contract diagnostics MUST print the full call chain (`[EFF-5]`).
* `[DIA-5]` FFI diagnostics name the header and the C declaration (`vkCreateBuffer in vulkan_core.h:1234`).
* `[DIA-7]` **Every ownership or borrow error MUST be classified into one of the shapes in §XIX.6.1 and MUST emit that shape's required `help` line.** An error the classifier cannot place emits the generic explanation and is recorded in `target/<profile>/unclassified-borrow-errors.log`; CI fails if the conformance suite produces any unclassified borrow error. This rule exists because an unexplained rejection is the single largest usability cost of static aliasing rules, and it is cheaper to specify the fixes than to discover them one bug report at a time.
* `[DIA-8]` `ember explain --borrow <file>:<line>` prints, for each loan live at that line: where it was created, the line range of its region, and the specific later use that extends it — in the form `borrow of `v` created at 12:9, live through 19, because `s` is used at 19:14`. This is generated from the borrow checker's own loan/region tables (Part XVIII §4.7), not reconstructed.
* `[DIA-9]` Diagnostics MUST NOT suggest `unsafe`, `Cell`, `RefCell`, `Shared`, or `clone()` as the *first* suggestion when a structural fix exists for the shape (per the catalogue's ordering). `clone()` is suggested first only for shape O1 (use after move of a `Clone` type).

### XIX.6.1 Required diagnostic catalogue for ownership and borrow errors

Each shape below MUST be recognised by the borrow checker's diagnostic classifier and MUST produce the listed `help`. Suggestions are ordered; the first applicable one is primary, the rest appear as additional `help` lines. Each shape has a directory in `tests/ui/borrow/<shape>/` with a snapshot of the exact rendered output.

| Shape | Trigger | Required primary suggestion |
|---|---|---|
| **O1** use after move | a place is read after being moved into a call, assignment or `owned` binding | if the type is `Clone`: pass `x.clone()`; else: restructure so the last use precedes the move, or borrow instead of consuming (name the parameter whose mode is `owned`) |
| **O2** move out of a container | move from an index, `Span` element, or a field of a `Drop` type | `mem.take`/`mem.replace`/`swap` for a field; `pop`, `swap_remove`, `remove`, `drain` for a container element |
| **O3** move in a loop | a value declared outside the loop is moved inside it | move the declaration inside the loop, clone per iteration, or use `mem.take` if the value is being replaced each pass |
| **O4** partial move then whole use | a struct is used as a whole after one field moved out | reassign the moved field, or destructure the whole struct up front |
| **B1** two mutable indices | `ref mut a[i]` and `ref mut a[j]` both live | `a.split_at_mut(k)`, `chunks_mut`, `iter_mut`, or `columns_mut` for `SoA` — name the one that fits the access pattern |
| **B2** mutate while iterating | a container is mutated inside `for x in container` | collect the indices or values first; use `retain`, `drain`, or `for i in 0..len` with indexed access; for the ECS, `q.commands()` |
| **B3** shared and mutable overlap | a shared borrow is live across a mutating call | shorten the shared borrow with a block, copy the value out if `Copy`, or reorder so the read happens before the mutation — cite the exact "later used here" line |
| **B4** aliased mutation of a value type | the same place needs two writers, not simultaneous within one expression | restructure to a single owner and pass `mut`; if genuinely shared, `Cell[T]` (`Copy` payload) or `RefCell[T]`; if it has identity, make it a `class` |
| **B5** self-referential struct | a struct field would borrow another field of the same struct | store an index or a `Handle` instead of a reference; split into two structs; for object graphs use a `class` with `Weak` back-pointers |
| **B6** returned reference not derived from a parameter | `[LT-1]` elision cannot tie the return region to an input | return an owned value, take the destination as a `mut` parameter, or add `@borrows(param)` naming the input it points into |
| **B7** borrow outlives its source | a loan is live past the `StorageDead` of the borrowed local | move the source to an outer scope, return an owned value, or bind the temporary with `with` so it lives to the end of the block |
| **B8** method takes all of `self` | disjoint-field access (`[BRW-4]`) is defeated by a method call | inline the field access, take the two fields as separate parameters, or split the method |
| **B9** closure outlives its captures | a non-`owned` closure escapes | declare it `owned fn` (captures by move/copy/retain), or use `thread.scope()`/`jobs.scope()` so the borrow is joined before it ends |
| **B10** `mut` argument is not a mutable place | a temporary, a `Copy` of a field, or an immutable binding is passed to a `mut` parameter | bind to a local first, or name the owner (`f(obj.field)` rather than `f(obj.get_field())`) |
| **X1** static exclusivity conflict (`E3080`) | two overlapping long-term accesses through the same class-handle local | shorten the first access with a block; take the field's value out if `Copy`; split the method so the two accesses do not nest |
| **R1** region unexplained | any of the above where the reason a borrow is still live is more than 5 lines from the error | append: `run `ember explain --borrow <file>:<line>` to see why this borrow is still live` (`[DIA-8]`) |

`[DIA-10]` Shapes B1, B2, B4, B5 and B9 account for most rejections in practice; their `help` text MUST name a concrete API or construct (`split_at_mut`, `retain`, `Weak`, `owned fn`), never a category ("consider restructuring", "use interior mutability").

Worked example of the required rendering for shape B2:

```
error[E3020]: cannot mutate `enemies` while it is borrowed by this loop
  --> src/game.em:41:13
   |
39 |     for e in enemies:
   |              ------- `enemies` borrowed here for the whole loop
40 |         if e.health <= 0:
41 |             enemies.remove(e.id)
   |             ^^^^^^^^^^^^^^^^^^^^ mutable borrow here
   |
   = help: collect first, then mutate: `dead = enemies.iter().filter(fn(e) => e.health <= 0).collect[Array[Id]]()`
   = help: or remove in place without a second pass: `enemies.retain(fn(e) => e.health > 0)`
   = note: a `for` loop borrows the container until it ends (see BRW-2)
```

### Error-code registry

| Range | Subsystem |
|---|---|
| E0000–E0099 | lexer / indentation / directives |
| E0100–E0499 | parser |
| E1000–E1499 | name resolution, modules, visibility (incl. `E1050`/`E1051`, `pub(read)`) |
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

v1 lints: `L2001 unnecessary clone` (source not used again), `L2002 large Copy` (> threshold bytes passed by value), `L3001 potential cycle` (statically visible class cycles), `L3002 borrow held longer than necessary` (a borrow whose last use is far before its scope end and blocks a later access — suggests a block), `L3010 unsafe block larger than necessary`, `L3011 RefCell guard held across a call` (a `Ref`/`RefMut` guard live across a call that could re-enter the same cell, `[CELL-7]`), `L4001 allocation in hot loop` (allocation inside a loop of a `@simd`/`@parallel` body or inside functions named in `[lints.hot_paths]`), `L4002 dynamic dispatch on final type` (redundant `dyn`), `L5001 unsafe extern without contract`, `L5002 FFI copy` (conversion at the boundary copying > threshold bytes), `L7001 lock held across call that may block`.

## XIX.9 Documentation

`ember doc` renders `##` doc comments to HTML/Markdown; every public function's page shows: signature with modes, **effects**, `@noalloc`-cleanliness, thread rules (`Send`/`Sync` of parameters and result), allocation behaviour (from effect analysis), and for FFI wrappers the underlying C declaration and contract. Doc examples in fenced ```` ```ember ```` blocks are compiled and run as tests by `ember test --doc`.

---

