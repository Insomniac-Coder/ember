# Part XX — Toolchain

## XX.1 CLI

```
ember new <name> [--lib | --bin | --cdylib]        scaffold a package
ember build [--profile debug|release|shipping] [--target <triple>] [--backend c|llvm] [--emit tokens|ast|hir|mir|c|obj]
            [--emit-header] [--emit-optimization-report] [--explain-performance] [-Dwarnings]
            [--report=instantiations] [--build-id] [--timings[=json]]
ember build --reload [--explain]   build a reloadable image; --explain reports whether it would
                                   reload cleanly against a running process, without applying it
ember run [args...] [--hot]        build + run the binary; --hot watches and swaps (Part XVIII)
ember test [filter] [--doc]      run @test functions (and doc tests)
ember bench [filter]
ember check                type-check + borrow-check without codegen
ember fmt [--check]
ember lint
ember inspect <path.to.item>       effects, layout, allocation sites, inlining, ABI
ember explain <EXXXX>              print the error's reference page
ember explain <rule-id>            print the rule, its category ([CAT-4]) and its conformance directory
ember explain --borrow <file>:<line>   why a borrow at that line is still live (region as line ranges)
ember inspect --safety <path> [--elided-only] [--json]   runtime safety checks emitted and elided
ember inspect --alloc <path>       allocation taxonomy and conversion costs ([TXT-5])
ember inspect --cost <path>        every [COST-3] row applying to this item, elided or emitted
ember inspect --deterministic <path>   whether it satisfies [DET-1], and the chain to each Nondet source
ember bind <header> [--emit-embind | --explain <symbol> | --report | --emit-cpp]
ember shader-bind <reflect.json>
ember doc
ember clean
ember toolchain {list|install|default}
```

`[CLI-1]` Every command supports `--json` for machine-readable output (diagnostics, inspect, report) and exits non-zero on error. `[CLI-2]` `ember build --emit=c --out-dir <dir>` writes the C sources without invoking a C compiler (used by the CMake integration).

* `[CLI-4]` `ember run <file.em>` and `ember build <file.em>` MUST accept a single source file with no `ember.toml`, synthesising a package named after the file (`kind = "bin"`, entry = the file, default profile, no dependencies beyond `std`). **A first program MUST NOT require a manifest.**

## XX.2 Manifest (`ember.toml`)

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
max_instantiations = 200            # per-generic ceiling; unset means no ceiling ([MONO-3])
reload = "opt-in"                   # "all" | "opt-in" | "bodies" | "none"  ([HR-25], [MAN-7])

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
* `[MAN-3]` Every key in `[lints]` MUST name a lint the compiler defines (`E9010` otherwise). XX §2's `unused` key is `L1001`.
* `[MAN-4]` The manifest file name, the source extension and the `.embind` extension are constants of the same `branding` module; the test harness and the build system discover them rather than spelling them.
* `[MAN-5]` `ember.toml` gains `[ffi] evidence = "<path>"`, default `".ember/ffi-evidence"`. It is part of `[BLD-2]`'s package config.

* `[VER-1]` Three version numbers exist and are independent: the **language version** (`language` in `ember.toml`, `#! language`), the **compiler version**, and the **runtime ABI version** (`EMBER_RT_ABI`). Every release states all three.
* `[VER-4]` **The runtime ABI is stable within a major version.** `[OBJ-1]` is amended accordingly: the object header layout, `ember_type_info`, and every entry point in `ember_rt.h` MUST NOT change within a major version. Any change bumps `EMBER_RT_ABI` and is a major release. `ember_rt_init` MUST compare `ember_rt_abi_version()` against the value its caller was compiled with and fail initialisation with a diagnostic naming both versions rather than proceeding. Fields MAY be appended to `ember_rt_config` only at the end, guarded by a leading size field. Debug-only extensions behind a profile key (`[FFI-33a]`) are not part of the frozen layout.
* `[VER-5]` Packages use semantic versioning. A `[dependencies]` requirement is caret by default (`"1.2"` means `>= 1.2.0, < 2.0.0`); resolution selects the highest version satisfying all requirements; two different majors of one package MAY coexist in a graph, **their symbols distinguished by a major-version component in `[MNG-1]`'s package segment**. `ember.lock` records the result and `ember update [pkg]` recomputes it.

**The 1.0 compatibility promise** (owner decision `OQ-22`; normative from 1.0). `[VER-2]` source compatibility within a major language version, with breaking changes only in a new major that a package opts into by editing `language`, and a compiler accepting every language version of its own major series. `[VER-3]` deprecation in `1.n` via `@deprecated` and a `W`-code naming the replacement and the removing version, with removal no earlier than the next major. `[VER-7]` 1.0 means `[VER-2]` and `[VER-4]` come into force, the conformance suite passes on every supported host, `docs/errors/EXXXX.md` exists for every code, and the user guide exists.


* `[MAN-6]` `[build] max_instantiations` is a positive integer or absent; absent means no ceiling and disables everything in XIX §4.11a (`E9034` otherwise). `[MAN-7]` `[build] reload` is one of `"all"`, `"opt-in"`, `"bodies"` or `"none"` (default `"opt-in"` in `debug`, `"none"` otherwise); any other value is `E9031`. It is forbidden in `shipping` (`E9033`). Every package linked into one process MUST agree on whether reload is enabled at all, which `[HR-12a]`'s link-time symbol enforces.
* `[CLI-15]` `--report=instantiations` (`[MONO-4]`), `--build-id` (`[BLD-13]`), `--reload` and `--reload --explain` (`[HR-18b]`), `--hot` (`[HR-33]`), `--timings` (`[BUD-6]`) and `ember inspect --deterministic` (`[DET-9]`) are the surfaces 0.6.3 adds. Each MUST appear in `ember --help` and in XXIII §3's construct list, per `[DIA-6a]`'s completeness check.

## XX.3 Build graph and caching

* `[BLD-1]` Unit of compilation and caching: the **module** (one `.em` file) for front-end stages; the **package** for monomorphisation and codegen (one C file per module is emitted, but instantiations are placed in the module that first requests them, with COMDAT-style `static inline`/weak linkage to dedupe).
* `[BLD-2]` Cache key of a module's front-end artefact: BLAKE3 of (source, compiler version, language version, package config, transitive **interface hashes** of imported modules — the hash of exported signatures, types, layouts, effect sets and inline bodies, not of private bodies). A change to a private function body recompiles only its module's codegen and any callers' *effect checks* if its effect set changed (`[EFF-4]`). "— the hash of exported signatures, types, layouts, effect sets, ** add the `[verify]` package-config section to the enumerated key.
* `[BLD-3]` The `.embind` cache key remains header hash + flags + overlay-list hash, but the **entity identities** it records are `[FFI-30]`'s C identities, so two cache entries for the same header and flags describe the same entities under different views. **"Flags" is not the compiler's command line: it is the set of settings that can change what a binding means**, and a binding MUST be invalidated when any of them changes. That set is, at minimum: the compiler and its version; the language standard; the target triple and ABI; the C++ standard library implementation and its version; on MSVC the runtime-library switch and the effective `_DEBUG` and `_ITERATOR_DEBUG_LEVEL`, which `[BLD-FFI-1b]` already requires to be inherited byte-for-byte because they change the layout of `std::string`/`std::vector` and the identity of the CRT heap; RTTI and exception settings, and the `/Zc` conformance switches; every `-D`/`/D` reaching the header; the calling convention and name-mangling scheme; and the thunk generator's own version, since a change in how a thunk owns, copies or catches changes the contract without changing the header. The governing test is not the length of that list: **if changing a setting can change a generated binding's semantics or its ABI, it is part of the key.** A setting the toolchain cannot determine is `E9021` and MUST NOT be defaulted (`[BLD-FFI-1b]`), for the same reason — a guess here binds successfully and differs at run time. *(clarified 2026-09-09; see `docs/spec-amendments.md`)*
* `[BLD-4]` The C compiler and linker are invoked through a Ninja file generated per build (`target/<profile>/build.ninja`) so that incremental C compilation is handled by Ninja; MSVC is driven with `/showIncludes`, Clang/GCC with `-MD`.
* `[BLD-5]` Output layout: `target/<profile>/{bin,lib,c,obj,bind,inspect}`.
* `[BLD-6]` **Link-time optimisation.** `profiles.<p>.lto` takes `"off" | "on" | "thin"`, mapped by the C backend to MSVC/clang-cl `/GL` + `/LTCG`, Clang `-flto=thin` / `-flto=full`, GCC `-flto`. Where a value is unsupported the toolchain MUST substitute the nearest supported value and record the substitution in the build record. **LTO MUST NOT be required to satisfy `[CG-C-3]` and MUST NOT be required for any correctness property** (XXIII.2).
* `[BLD-7]` Front-end stages MUST run in parallel across modules by default (`-j`, default = physical cores), and MUST NOT take a global lock on the symbol interner or the type interner on the hot path.
* `[BLD-8]` **Item-granular re-checking.** Within a module, the front end MUST record, per item, the signatures it read, so that editing one function body re-runs type checking, borrow checking and effect analysis **for that item alone**. Module-level name resolution re-runs only if the edit changed an item's signature or the set of names the module declares. `[BLD-2]`'s interface hash is unchanged; this is a finer key beneath it.
* `[BLD-9]` `ember build --timings` and `ember check --timings` write `target/<profile>/timings.json` plus an HTML summary, per stage and per module.
* `[BLD-10]` **Latency budget.** `tests/perf/compile/` contains generated packages of 10k, 50k and 200k lines with a realistic mix (≈30 % generic code, 20 % classes, 10 % FFI declarations). Budgets are measured on the reference machine recorded in the suite, at the `debug` profile, for: `ember check` cold; `ember check` after one function body is edited; `ember build` after one function body is edited; `ember build` cold. **The budgets are release gates with the same status as XXI §4's runtime thresholds.** The concrete figures are calibrated once the generator exists and recorded in the suite, not in this document; **the existence of the gate is normative, the numbers are not.**

* `[BLD-13]` **Reproducible builds.** Given the same source, manifest, dependency lock, toolchain version, target and profile, `ember build` MUST produce a byte-identical artifact. The compiler MUST NOT embed a build timestamp, an absolute path, a host name, a user name, or the iteration order of any hash container into its output; diagnostic paths are recorded relative to the package root, and `[MONO-1]` already makes symbol names deterministic. `ember build --build-id` prints a hash over exactly those inputs, so two lockstep peers can compare one string before a match rather than discover a mismatch three minutes in — which is what `[DET-7]` requires of them.

## XX.4 Profiles

The profile controls: optimisation level, overflow policy, bounds checks (never disableable in safe code — `bounds_checks = false` only affects `unsafe`-opted `get_unchecked` hints and is honoured by no v1 profile), exclusivity checks, `debug_objects`, sanitizers, LTO, `panic` policy, `strip`. `[PRF-1]` A profile MUST NOT change program semantics except: (1) overflow policy (`[TYP-8]`); (2) panic-vs-UB for dynamic **class** exclusivity under `exclusivity = "unchecked"` (`[EXC-1]`, ADR-004); and (3) presence of debug facilities. In particular no profile may change which branch of a `match`, `if`, `?` or short-circuit operator is taken, nor the value of any expression observable to safe code, nor whether a program is accepted (`[EFF-15]`).


* `[PRF-2]` **Reload is a profile-constrained build mode, not a profile.** `[MAN-7]`'s `reload` key is admitted in `debug` and `release` and forbidden in `shipping` (`E9033`). A build with reload enabled emits permanent thunks for reloadable functions, suppresses `[CG-C-3]`'s inline-header emission for them (`[HR-9a]`, since an inlined body cannot be swapped), and uses the 40-byte header of `[HR-12]`. `[PRF-1]` applies unchanged and `[HR-27]` states it for this part: a build without reload is bit-identical to one from a source containing no reload construct at all.

## XX.5 Test harness and conformance suite layout

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

Annotations begin with `#$`. To the compiler that is an ordinary line comment (`[LEX-10]`), so an annotation never affects compilation and an annotated file is a valid compilation unit. `$` is the only ASCII symbol with no other meaning in Ember — it appears in no operator, no literal form and no identifier rule — so an annotation cannot collide with source, now or after any future grammar change.

* `[TST-0]` The test harness MUST read annotations from the **raw source text**, never from the token stream, because a `compile-fail` test may be expected to fail at the lexer (`E0001` invalid UTF-8, `E0002` a tab in indentation) and its expectations must be readable when the file does not tokenise at all.
* `[TST-1]` `#$ error[EXXXX]: <message-substring>` on a line asserts an error with that code whose primary span starts on that line; `#$ warning[...]`; `#$ note` for secondary labels (`@ line±n` for relative positioning). Unexpected diagnostics fail the test; missing expected ones fail the test.
* `[TST-2]` `run-pass` files may contain `#$ stdout:` followed by `#$` lines of expected output, and `#$ exit: N`. `#$ assert-c: contains("…")` and `#$ assert-c: !contains("…")` assert against the emitted C.
* `[TST-3]` `@test` functions in `std/` and user packages run in-process in the test binary, each in isolation (panics captured with `abort` replaced by a longjmp-based harness in the test runner build only; with the unwind policy, normal catch).
* `[TST-4]` Every rule ID in this document appears in `tests/conformance/INDEX.md`, generated by `tools/rule_index.py` from the spec, which fails CI if a rule has no test directory.
* `[TST-4a]` **A directory is not coverage.** Every rule's directory MUST contain at least one **accept** case — source the rule admits, which compiles and behaves as the rule says — **and**, for a rule that can reject source, at least one **reject** case: source the rule refuses, with the expected diagnostic recorded in a `.stderr` file that the harness compares exactly. A rule tested only by its happy path is not tested: it passes against a compiler that never enforces it, which is the single most likely way a conforming-looking implementation is not one.
* `[TST-4b]` **Which rules need a reject case is decided mechanically, not by judgement.** `[DIA-6a]` already maps every diagnostic code to the rule that defines it, so a rule naming a code MUST have a reject case for each code it names, and a rule naming none — a layout guarantee, a representation statement, a rule about what the compiler MAY elide — is waived automatically. The waiver list is generated by `tools/rule_index.py` and committed, so a rule that later acquires a diagnostic loses its waiver in the same pass that adds the code, and nobody maintains a list by hand.
* `[TST-4c]` **Transition.** `[TST-4a]` ships with a **recorded baseline** of rules that currently have only an accept case, in the manner of `[TST-7]`, and fails CI only on a rule added or amended after 0.8.1 and on any rule whose entry improves and then regresses. The baseline shrinks and never grows: `tools/rule_index.py` rejects a commit that adds a rule to it. `[GATE-1]` requires the baseline to be **empty** for 1.0, which is what makes this a transition rather than a permanent exemption.
* `[TST-6]` `docs/spec-source/appendix-a.em` is a conformance fixture, and Appendix A's code block is generated from it by `tools/spec_check.py --emit-appendix`, so the quick reference cannot drift away from something that parses. The fixture's directive is the current language version; `...` bodies are `pass`; the statement tail is wrapped in `fn demo():`; `match` arms use the `=>` form, since `[GRM-16]` makes a jump an expression. It is annotated `#$ test: compile-pass`.
* `[TST-7]` `tools/spec_check.py` extracts every fenced ` ```ember ` block from Parts I–XVII and Appendix A and runs `ember check --syntax-only` over each. A block that does not parse fails CI. A block MAY opt out with ` ```ember,ignore ` and MUST then carry a one-line reason on the preceding line (permitted reasons: a `std` signature sketch, foreign-language source, or a deliberate error example). A fourth permitted reason is **overlay-language source, until Part III defines `overlay_decl`**; XVI.4's `overlay c "vulkan/vulkan.h":` block and XVI.7a's `overlay cpp "RageV/VulkanBackend.hpp":` block carry it, so the baseline is explicit rather than implied. The gate ships with a **recorded baseline** of `,ignore` blocks and fails only on *new* failures until RFC-010 and RFC-022 have closed the grammar gaps it exists to expose.
* `[CLI-9]` `ember check --syntax-only <file>` lexes and parses the file and reports only `E00xx` and `E01xx` diagnostics. It does not resolve names, so an example naming undeclared types still passes.
* `[CLI-10]` `ember build --report=engine` is a **reporting mode, not a compilation profile**. It MUST NOT change type checking, safety, program acceptance, generated semantics, or optimisation legality — `[PRF-1]` governs profiles and this is not one. The report summarises allocations, retain/release traffic, dynamic class accesses, surviving runtime checks with their `[EFF-11]` reason codes, arena usage, vectorisable loops and the clause that blocked each one, FFI wrapper costs, inlining decisions, and candidate class-to-struct migrations. A migration candidate is advisory only and is never applied automatically.
* `[CLI-5]` `ember bind --init <header> [--out overlays/<stem>.em]` writes a **starter overlay** containing, for every declaration the header exports: its derived contract; every `unknown` fact written as a `TODO(count)` / `TODO(nullable)` / `TODO(ownership)` marker with the original C declaration in a trailing comment; every skipped declaration as a commented `W5002` line with its reason; and a `hide` section. The emitted file MUST compile as an overlay unchanged. `--init` never overwrites an existing file; with `--merge` it adds only declarations absent from it, preserving hand-written contracts and comments.
* `[CLI-6]` `ember bind --report [--json] [--baseline <file>]` prints every declaration that did not import, with its construct and reason (`[FFI-20a]`), and every declaration still `unsafe`, with the facts still `unknown`. It exits non-zero when anything is skipped, so a project can gate CI on a header continuing to import; with `--baseline` it exits non-zero only on *new* skips, and `--write-baseline` records the current state.
* `[CLI-7]` `ember bind --check <overlay>` verifies `[FFI-12]` without building, and prints the counts of declarations that are safe, still `unsafe`, and skipped — so the state of an adoption is a number a team can watch.
* `[TST-8]` **First-draft corpus.** `tests/firstweek/` holds at least 24 programs of 30–200 lines, each the *first draft* of a task a newcomer plausibly attempts in week one — a text adventure with rooms holding items, a particle fountain, a CSV summariser, a scene graph with parent pointers, an observer/event bus, an inventory with equipment slots, a tile-map path finder, a config loader, a ring buffer of chat messages, a small hand-rolled ECS. Each is contributed by someone who has read only Part I and Appendix A, MUST be committed unmodified, and records its author's experience level.
* `[TST-9]` Each program carries `#$ firstdraft: accepted` or `#$ firstdraft: rejected(<shape>)` naming a §XX.6.1 or §XX.6.2 shape. **A rejection whose shape is in no catalogue is a release blocker** — it is `[DIA-7]`'s unclassified case in the only corpus not written by people who already know the answer. The companion obligation — that the mandated `help`, applied literally, produces a program that compiles — is `[PHIL-8a]` (RFC-011) and applies to the whole catalogue, not only to this corpus; the corrected program is committed beside the draft as `<name>.fixed.em`.
* `[TST-11]` **v0.5 regression obligations.** The conformance suite MUST contain, in addition to the per-rule directories `[TST-4]` requires: for `[EXC-4]`, instantaneous scalar reads, `Copy` field reads, container iteration through `let` fields, `ref`/`ref mut` projections, mutating methods through `let` containers, and aliasing mutation during iteration; for `[LT-7]`, callback-local borrows, escape rejection, internal-only region inference, and nested callback-region separation; for `[TYP-15]`/`[TYP-15a]`, rejection of arbitrary owning view containers, acceptance of `BorrowList[T]`/`ViewList[T]` only under a single inferred region, rejection where two independent regions would be required, and escape rejection through class fields, `static`s, `Box` and `Shared`; for `[FFI-30c]`, overlay composition order, explicit `override`, incompatible-definition diagnostics, foreign type identity stability, and build invalidation on overlay change; for `[EFF-17]`, rejection of every `Panic(Explicit)` source and acceptance of division by `NonZero[T]`; and for `[PRF-1]`, that one source receives the same contract verdict under `debug`, `release` and `shipping`, and that `--report=engine` does not alter that verdict. The v0..6]` evidence invalidation after a foreign identity or contract hash changes.ser.yaml` with an out-of-range field producing `SerError` rather than an invalid value; `[RNG-10b]`'s rejection of a range type in an `extern` block and behind a pointer parameter; and `Option[Full]` occupying two bytes.
* `[TST-10]` The harness computes the **first-draft acceptance rate** (accepted ÷ total) into `target/firstweek.json` and the release notes publish it. It is a **tracked number, not a gate** — the gate is `[TST-9]`'s blocker — and a release that lowers it MUST say why.
* `[TST-5]` `tests/debug/` runs, on every supported host in CI, a scripted debugger session (`cdb` on Windows, `lldb`/`gdb` elsewhere) against a `debug` build: set a breakpoint by Ember `file:line` and verify it binds; step three Ember statements and verify the reported line advances by one construct each; print four locals by their Ember names and match the rendered text against a snapshot. A host on which `[TST-5]` cannot run is reported as **debug-unverified** and requires a recorded waiver; it does not thereby cease to be a supported host, since debugger automation is a property of the CI image.

## XX.6 Diagnostics

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
* `[DIA-3]` Borrow/ownership diagnostics MUST include the "later used here" label (`[BRW-2]` explanation) and a concrete fix drawn from the catalogue in §XX.6.1 (`[DIA-7]`).
* `[DIA-4]` Contract diagnostics MUST print the full call chain (`[EFF-5]`).
* `[DIA-5]` FFI diagnostics name the header and the C declaration (`vkCreateBuffer in vulkan_core.h:1234`).
* `[DIA-7]` **Every ownership or borrow error MUST be classified into one of the shapes in §XX.6.1 and MUST emit that shape's required `help` line.** An error the classifier cannot place emits the generic explanation and is recorded in `target/<profile>/unclassified-borrow-errors.log`; CI fails if the conformance suite produces any unclassified borrow error. This rule exists because an unexplained rejection is the single largest usability cost of static aliasing rules, and it is cheaper to specify the fixes than to discover them one bug report at a time.
* `[DIA-8]` `ember explain --borrow <file>:<line>` prints, for each loan live at that line: where it was created, the line range of its region, and the specific later use that extends it — in the form `borrow of `v` created at 12:9, live through 19, because `s` is used at 19:14`. This is generated from the borrow checker's own loan/region tables (Part XIX §4.7), not reconstructed.
* `[DIA-9]` Diagnostics MUST NOT suggest `unsafe`, `Cell`, `RefCell`, `Shared`, or `clone()` as the *first* suggestion when a structural fix exists for the shape (per the catalogue's ordering). `clone()` is suggested first only for shape O1 (use after move of a `Clone` type).
* `[DIA-6a]` The error-code registry is **exhaustive and normative, in both directions**: every code named anywhere in this document MUST have a registry entry (code, kind, subsystem, title, the rule it enforces) and a `docs/errors/EXXXX.md` page, **and** every registry entry MUST cite a rule id that exists in this document. `tools/rule_index.py` MUST fail CI on either violation, in the same pass that checks `[TST-4]`. **A code MUST be defined by exactly one rule.** `tools/rule_index.py` MUST fail CI when two rules name the same code with different titles, in the same pass that enforces rule-ID uniqueness (XXIII.4). It MUST additionally fail CI when a rule defined in the active index has no row in the current revision's change log, and when an attribute, construct or CLI surface named in normative text is absent from Part III §7, `[ATT-2]`, XX.1 or XXIII.3. **This check, not more review, is what prevents a 0.7 that repeats this.**
* `[DIA-7a]` **Every code is keyed.** The table below maps every error code in `E3000–E3499` to the shape whose `help` it MUST emit. A code absent from this table MUST NOT be emitted. `tools/rule_index.py` fails the build if `ember_diag::codes` contains an E3xxx code with no row here.  | Code | Shape | Code | Shape | |---|---|---|---| | `E3010`–`E3013` | O2 | `E3050` | O6 | | `E3014`, `E3015` | O8 | `E3060` | B7 | | `E3016` | O9 | `E3061`, `E3090`, `E3096` | A1 | | `E3020` | B2 | `E3062` | B6 | | `E3021` | B3 | `E3063` | B12 | | `E3022` | B1 | `E3064` | B13 | | `E3023` | B4 | `E3070` | O7 | | `E3024` | B5 | `E3080` | X1 | | `E3025` | B8 | `E3095` | B11 | | `E3026` | B9 | `E4030` | S1 | | `E3027` | B10 | `E3030` | O5 | | `E3040` | O1 | `E3041` | O3 | | `E3042` | O4 | | |
* `[DIA-12]` Every diagnostic in `E1000–E1499` and `E2000–E2499` MUST be classified into one of the shapes below and MUST emit that shape's required `help`. An unclassifiable error is written to `target/<profile>/unclassified-basic-errors.log`; CI fails if the conformance suite produces any. This is `[DIA-7]` applied to the errors a newcomer meets first.
* `[DIA-13]` Every shape in §6.1 and §6.2 has a rendered snapshot under `tests/ui/`, plus the `.fixed.em` companion `[PHIL-8a]` requires; `[TST-4]`'s index generator fails CI for a shape with none.
* `[DIA-14]` No diagnostic may be emitted about an expression of type `Ty::Error` or a path bound to `Def::Error` (`[IDE-3]`): the first error of a cascade is reported and the rest suppressed. `[AST-2]`'s three-per-region cascade cap applies to the whole front end, not the parser alone.
* `[DIA-15]` Suggestion candidates are computed only from data the compiler already holds (scope tables, `TypeInfo`, and a by-name index over `[BLD-2]`'s module export hashes). A suggestion requiring speculative type checking is not required.
* `[DIA-16]` **World changes disclose their cost.** When a diagnostic's `help` proposes moving a value type into the object world (`class`, `Shared[T]`) or wrapping it for dynamic checking (`RefCell[T]`), it MUST attach a `note:` naming the cost in the terms the specification already defines: ``` = note: a class instance is a heap allocation with a 24-byte header (OBJ-1) and reference counting (RC-1); long-term accesses through it are checked at runtime (EXC-1), so a function containing them cannot be @static_safe (EFF-13) ``` The note is required because the object world's costs are permanent and structural while the value-world fixes offered alongside are local; a diagnostic presenting them as equivalent misinforms.
* `[DIA-17]` **Python-form fix-its.** `x is None` / `x is not None` on a non-handle operand suggests `x.is_none()` / `x.is_some()`; `a < b < c` suggests `a < b and b < c` and notes that `b` is then evaluated twice; `xs[-1]` on a `usize`-indexed container suggests `xs.last()`; `let x = e` in statement position suggests `x = e` and notes that `let` marks an immutable **field** (`[CLS-9]`); `with e as x:` suggests `with x = e:`.

### XX.6.1 Required diagnostic catalogue for ownership and borrow errors

Each shape below MUST be recognised by the borrow checker's diagnostic classifier and MUST produce the listed `help`. Suggestions are ordered; the first applicable one is primary, the rest appear as additional `help` lines. Each shape has a directory in `tests/ui/borrow/<shape>/` with a snapshot of the exact rendered output.

| Shape | Trigger | Required primary suggestion |
|---|---|---|
| **O1** use after move | a place is read after being moved into a call, assignment or `owned` binding | if the type is `Clone`: pass `x.clone()`; else: restructure so the last use precedes the move, or borrow instead of consuming (name the parameter whose mode is `owned`) |
| **O2** move out of a container | move from an index, `Span` element, or a field of a `Drop` type | `mem.take`/`mem.replace`/`swap` for a field; `pop`, `swap_remove`, `remove`, `drain` for a container element |
| **O3** move in a loop | a value declared outside the loop is moved inside it | move the declaration inside the loop, clone per iteration, or use `mem.take` if the value is being replaced each pass |
| **O4** partial move then whole use | a struct is used as a whole after one field moved out | reassign the moved field, or destructure the whole struct up front |
| **B1** two mutable indices | `ref mut a[i]` and `ref mut a[j]` both live | `a.split_at_mut(k)`, `chunks_mut`, `iter_mut`, or `columns_mut` for `SoA` — name the one that fits the access pattern; if the operands come from unrelated sources, `mem.assert_disjoint(a, b)` (`[DSJ-1]`) |
| **B2** mutate while iterating | a container is mutated inside `for x in container` | collect the indices or values first; use `retain`, `drain`, or `for i in 0..len` with indexed access; for the ECS, `q.commands()` |
| **B3** shared and mutable overlap | a shared borrow is live across a mutating call | shorten the shared borrow with a block, copy the value out if `Copy`, or reorder so the read happens before the mutation — cite the exact "later used here" line |
| **B4** aliased mutation of a value type | the same place needs two writers, not simultaneous within one expression | restructure to a single owner and pass `mut`; if genuinely shared, `Cell[T]` (`Copy` payload) or `RefCell[T]`; if it has identity, make it a `class` |
| **B11** disjointness not provable | two views whose ranges the compiler cannot relate are both borrowed mutably | `mem.assert_disjoint` to verify and establish it (two comparisons); the structural fix from B1 if the views share a parent |
| **B5** self-referential struct | a struct field would borrow another field of the same struct | store an index or a `Handle` instead of a reference; split into two structs; for object graphs use a `class` with `Weak` back-pointers |
| **B6** returned reference not derived from a parameter | `[LT-1]` elision cannot tie the return region to an input | return an owned value, take the destination as a `mut` parameter, or add `@borrows(param)` naming the input it points into |
| **B7** borrow outlives its source | a loan is live past the `StorageDead` of the borrowed local | move the source to an outer scope, return an owned value, or bind the temporary with `with` so it lives to the end of the block |
| **B8** method takes all of `self` | disjoint-field access (`[BRW-4]`) is defeated by a method call | inline the field access, take the two fields as separate parameters, or split the method |
| **B9** closure outlives its captures | a non-`owned` closure escapes | declare it `owned fn` (captures by move/copy/retain), or use `thread.scope()`/`jobs.scope()` so the borrow is joined before it ends |
| **B10** `mut` argument is not a mutable place | a temporary, a `Copy` of a field, or an immutable binding is passed to a `mut` parameter | bind to a local first, or name the owner (`f(obj.field)` rather than `f(obj.get_field())`) |
| **X1** static exclusivity conflict (`E3080`) | two overlapping long-term accesses through the same class-handle local | shorten the first access with a block; take the field's value out if `Copy`; split the method so the two accesses do not nest |
| **S1** `@static_safe` violated (`E4030`) | a `@static_safe` function carries `RuntimeCheck(Aliasing)` | name the exact access and why it is dynamic (`[EFF-13]`): hoist the handle to a local so `[EXC-3]` applies; switch to value types / a `Query` yielding `ref mut`; establish disjointness with `assert_disjoint`; or drop the contract if the check is `not_provable_in_principle` |
| **R1** region unexplained | any of the above where the reason a borrow is still live is more than 5 lines from the error | append: `run `ember explain --borrow <file>:<line>` to see why this borrow is still live` (`[DIA-8]`) |
| **O5** closure moves out a capture (`E3030`) | a closure body moves a captured non-`Copy` value into a call, a return, or an assignment | declare the parameter `owned f:` so the closure is `CallableOnce` (`[CLO-6]`, RFC-023); or `mem.take` the capture if the type is `Default`; `clone()` last. *If RFC-023 is declined, the required text is instead: name the type, state that it has no `Default`, and suggest `Option[T]` + `mem.take` + `expect` while naming the panic path introduced* |
| **O6** borrow of a moved or uninitialised place (`E3050`) | a `ref`/`ref mut` is taken of a place not initialised on every path | name the path on which it is uninitialised; initialise before the borrow, or restructure so the borrow follows the initialisation |
| **O7** explicit `drop` call (`E3070`) | `x.drop()` in source | `mem.drop(x)` to end the value's life early, or let it fall out of scope |
| **O8** leaked `@must_drop` value (`E3014`, `E3015`) | a scope guard moved out of its binding, forgotten, or stored where it may cycle | the closure-scoped form (`[THR-5]`) |
| **O9** `self` escapes its own drop (`E3016`) | a `drop` body publishes a handle to the object being destroyed | `mem.take` the data out; for pooling, keep the count above zero rather than resurrecting |
| **A1** arena lifetime (`E3061`, `E3090`, `E3096`) | an arena view outlives the arena; a `needs_drop` type is arena-allocated; the parent is used while a scope is live | move the `Arena` to an outer scope, or copy the value out before `reset`; for `E3090`, `alloc_nodrop` with the acknowledgement that `drop` will not run; for `E3096`, allocate from `scope`, or take the allocation before opening it |
| **B12** view stored in a place that outlives it (`E3063`) | a `ref`, `Span`, `str` or `@view struct` is stored in a class field, a non-view struct field, a `static`, a container element, a `Box`/`Shared`, or an `owned fn` capture | store an owned copy (`String` for `str`, `Array[T]` for `Span[T]`) **and name the per-element allocation cost**; or store a `u32` index / `Handle[T]` **and name the container it indexes**; for object graphs, `Weak` |
| **B13** two independent regions in one view struct (`E3064`) | a `@view struct` would need its fields' regions to differ (`[LT-2]`) | pass the two views as separate parameters rather than bundling them; or copy the shorter-lived data into an owned field |

`[DIA-10]` Shapes B1, B2, B4, B5 and B9 account for most rejections in practice; their `help` text MUST name a concrete API or construct (`split_at_mut`, `retain`, `Weak`, `owned fn`), never a category ("consider restructuring", "use interior mutability").

`[DIA-11]` For shape S1 the diagnostic MUST report the reason code of the offending check (`[EFF-11]`). When the reason is `not_provable_in_principle` or `inherent_to_mechanism`, the primary suggestion is to **remove the contract or change the data structure** — never to restructure the code, because no restructuring will help. Suggesting an impossible fix is the failure mode this rule exists to prevent.

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
| E4000–E4499 | effects and contracts (`@noalloc`, `@static_safe` `E4030` …), SIMD/parallel contracts |
| E5000–E5499 | FFI (import, overlays, layout mismatch, callbacks, C++) |
| E6000–E6499 | comptime |
| E7000–E7499 | concurrency (`Send`/`Sync`, parallel loops, ECS scheduling) |
| E8000–E8499 | layout attributes, GPU layout |
| E9000–E9499 | build system, manifest, toolchain |
| W-codes | same ranges, warnings |
| L-codes | lints (`ember lint`) |
| E-panic | runtime panics (documented messages, not compile-time codes) |

Codes added by 0.6 and 0.6.1, each inside the range its subsystem owns: `E1021` (name not linked in this build, `[STD-6]`); `E2210`–`E2215` (range types); `E4041`/`E4042` (`@noio`, `@nolock`); `E4050`–`E4057` and `E4060`–`E4064` (contracts and verification); `E5050`–`E5054` (foreign grades, lifetimes and range types at the boundary); `E9012`/`E9013` (manifest sections); `E5055` (Ember generic passed to a C++ template). Warnings `W3012` (undocumented `unsafe`), `W4001` (`result`/`old` shadowing), `W5050` (unbacked or stale grade), `W5054` (unexercised foreign fact). Lints `L2003` (fallible construction where a total one exists), `L3015` (undocumented unsafe obligation), `L3016` (`@safety` text still reads `TODO`).

Codes added by 0.6.3, each inside the range its subsystem owns: `E2220` (`yield` outside a `gen fn`), `E2221` (borrow held across a `yield`), `E2222` (coroutine where an ordinary function is required — `extern`, `@export`, C function pointer, or a `gen fn` not returning `Coroutine[R]`), `E2223` (`@never_specialize` on a generic that is not shareable); `E4070` (`@deterministic` violated), `E4071` (reserved — `@deterministic` reaching an `extern` without `@ffi(deterministic)`, reported as `E4070` with the foreign frame named, and registered so the code is not reused), `E4072` (`@fastmath` or `@fp(contract)` inside `@deterministic`), `E4073` (reserved — a coroutine frame that allocates in `@noalloc`, which `[CORO-5]` makes unreachable in v1 and which is registered so the code is not reused); `E9030` (hot reload refused; **0.7.1 re-points this at `[HR-18]`'s refusal report, which supersedes the 0.6.3 meaning**), `E9031` (bad `reload` value, `[MAN-7]`), `E9032` (reloadable function's address escaping to foreign code — retained as a code, though `[HR-6]`'s permanent thunks make it unreachable in 0.7.1, so it is registered and never emitted), `E9033` (`reload` in `shipping`, `[PRF-2]`), `E9034` (bad `max_instantiations`). Warnings `W2220` (instantiation ceiling exceeded), `W9030` (reload requires a restart, with the reason). Lint `L2004` (`gen fn` with no `yield`).

Codes added by 0.7.1, each inside the range its subsystem owns: `E2224` (a `mut` receiver on `migrate_from`, `[HR-16]`); `E5056` (overriding a virtual not in `virtuals=[…]`), `E5057` (naming a non-virtual in `virtuals=[…]`), `E5058` (foreign base with no default constructor and no declared `init`), `E5059` (`@ffi(trampoline)` on a base with no virtual destructor and no `owner=`), `E5060` (upcast of an owning handle in a context that cannot hold the `Retained` token); `E9035` (packages linked into one process disagreeing about the object-header layout, `[HR-12a]`), `E9036` (a reloadable package configured for static runtime linkage, `[HR-29]`). `E9031` and `E9033` are re-pointed by `[MAN-7]` and `[PRF-2]` to the `reload` key and the `shipping` prohibition. Warnings `W5033` (`&&`-qualified member skipped), `W5034` (anonymous-namespace entity skipped). Lint `L2005` (`@noreload` function calling a reloadable one inside a loop, `[HR-10a]`).

Codes added by 0.8.3: `E2227` (a `throws = "noexcept"` import called from `migrate_from` without `@allow_reload_terminate`, `[HR-39]`/`[HR-43]`). Codes added by 0.8.2c: `E2226` (`in` or `not in` on a type that does not implement `Contains`, `[STD-8]`). Codes added by 0.7.2, each inside the range its subsystem owns: `E2225` (`migrate_from` whose effect set contains `Panic`, `[HR-35]`); `E5061` (imported C++ function with no exception policy), `E5062` (contradictory exception policies), `E5063` (foreign bytes reaching `str` without validation, `[TXT-2]`), `E5064` (interior null in a value converted to `CStr`, `[TXT-3]`); `E9037` (`EMBER_RELOAD_ABI` mismatch on image load, `[ABI-2]`). Lints `L3017` (reference cycle detected, `[WK-4]`), `L3018` (`unsafe` block with no reason category, `[UNS-9]`).

Codes added by 0.8: `E5065` (converting between `Shared[T]`/`Weak[C]` and `CppShared[T]`/`CppWeak[T]`, `[SEL-2]`). 0.8 adds no other code: it is a documentation and release-criteria revision, and a revision that needed a new diagnostic to state what the language already meant would not be one.

`[DIA-6]` `docs/errors/EXXXX.md` exists for every code with an example and its fix; `ember explain E3040` prints it.
* `[DIA-18]` A diagnostic that rejects a foreign call because its contract contains `unknown` facts MUST name each missing fact and print the exact overlay line that supplies it: ``` error[E5002]: `vkGetPhysicalDeviceQueueFamilyProperties` requires `unsafe`: its contract is incomplete note: `pQueueFamilyProperties` has no count contract and no nullability help: add to `overlays/vulkan.em`: unsafe fn vkGetPhysicalDeviceQueueFamilyProperties( physicalDevice: borrowed, pQueueFamilyPropertyCount: inout_count, pQueueFamilyProperties: span(len_of(pQueueFamilyPropertyCount), nullable)) ```

### XX.6.2 Required diagnostic catalogue for the errors of the first hour

The shapes in §XX.6.1 are the errors of week two. These are the errors of the first hour, and they decide whether a newcomer stays. Each MUST be recognised and MUST produce the listed `help`.

| Shape | Trigger | Required primary suggestion |
|---|---|---|
| **N1** unknown name | a path resolves to nothing | the in-scope binding, field, method or item within Damerau–Levenshtein distance ≤ 2 (or ≤ ⅓ of the name's length), named exactly; at most 3, ranked by distance then declaration proximity |
| **N2** name exists elsewhere | nothing in scope matches, but items of that name are exported by modules in the dependency graph | the import line to add, verbatim, as a machine-applicable fix-it (`from std.math import Transform`); with several candidates, up to 5 with module paths |
| **N3** unknown method | the receiver's type has no such member | the nearest member by N1's rule; if the name exists after one auto-deref or on `Self`'s base (`[TYP-24]`), name that; if it exists on an interface the type does not implement, name the interface and the `extend` block that would supply it |
| **N4** scalar type mismatch | `[TYP-4]` rejects an operator, or a coercion site is not a `[TYP-5]` widening | the exact `as` cast on the operand that loses nothing (`xs.len() as f32`); the note MUST state which side supplied the expected type |
| **N5** literal does not fit (`E2010`) | `[LEX-16]` | the suffix or annotation that makes it fit |
| **N6** arity mismatch | too few/many arguments | the callee signature rendered with parameter names and modes; for a missing defaulted argument, the named-argument form |
| **N7** mutability | a `mut` method or parameter receives a non-mutable place | the declaration to change, named and located ("the binding `p` at 12:5 is not mutable") — never a bare "cannot borrow as mutable" |
| **N8** wrong argument mode | an `owned` parameter receives a borrow or vice versa | the parameter and its mode in the callee's signature, and the one-token fix at the call site |
| **N9** missing bound (`E2040`) | `[TYP-17]` | the bound to add, spelled — **and, because `[TYP-20]` permits the implementation in only one place, the single module in which `extend T implements I:` may legally be written, or a statement that neither module is under the programmer's control and a wrapper type is required** |
| **N10** `dyn` incompatibility (`E2050`) | `[TYP-22]` | the offending method and which clause it violates (no receiver / generic / returns `Self` by value), never a bare "interface is not object safe" |
| **N11** ambiguous method (`E2070`) | two candidates | the `I.m(recv, …)` disambiguation for each candidate interface |
| **N12** indentation (`E0002`–`E0004`) | | the exact column expected and the line whose indentation established it |

## XX.6a Trusted base and audit reporting

The trusted-base report lists what the compiler could **not** establish for
itself, and how strong each remaining claim is.

```text
$ ember tcb

Trusted for memory safety
  borrow checker, RC runtime, arena allocator      language

Unsafe
  unsafe blocks                                    12
  raw pointer operations                            7

Foreign
  C functions                                      23   19 checked, 4 asserted
  C++ bridge calls                                 14    2 proven, 9 instrumented, 3 asserted
  still requiring `unsafe`                          3

Assumptions relied on (7)
  VulkanBackend  the destructor does not throw                       asserted
  VulkanBackend  submit() does not allocate                          instrumented
  vkGetDevice    returns a valid device                              asserted
```

* `[TCB-1]` Every foreign fact carries one of four **grades**: **asserted** (a
  human wrote it), **checked** (tooling derived or confirmed it from the header,
  the ABI or the build), **instrumented** (a test run observed it holding), or
  **proven** (discharged by analysis of the adapter — *not* by a theorem prover,
  which v1 does not have; the analysis is of the thunk the importer itself emitted,
  so its scope is exactly what that code does and nothing about the foreign body it
  calls). A fact with no grade is asserted. The four grades are the complete set and
  the grammar production at Part III agrees with this list; a revision changing
  either MUST change both, and `tools/rule_index.py` compares them. **checked** — the fact is enforced or verified by something **outside the overlay author**: the ABI; the compiler's own layout cross-check (`[FFI-5]`); or the C++ runtime, as for `noexcept`, which `std::terminate` enforces. An **unenforced source annotation** — `const`, `restrict`, `[[nodiscard]]`, `[[clang::lifetimebound]]` — is `asserted`, however useful it is as a hint. This is what makes `[TCB-2]` true rather than aspirational.
* `[TCB-2]` The report MUST separate what the language guarantees from what an
  external component supplies. **A declaration never raises a grade**: no grade
  above `asserted` may be produced by writing it down.
* `[TCB-3]` Every assumption a proof relied on and did not discharge appears in
  the report, with its source and its grade.
* `[TCB-4]` Entries are categorised: language, compiler, runtime, standard
  library, unsafe, C FFI, C++ bridge, external library, GPU driver, hardware,
  solver, assumption. **The compiler is in the list.** Its own defects are part
  of what any claim rests on, and the report says so rather than implying the
  toolchain stands outside the question. The **Unsafe** section MUST list, per `unsafe fn` declaration and per `unsafe` block, its `[UNS-7]` obligations with grade `asserted` rather than a count alone. `ember tcb --json` MUST expose them, and `[CLI-12]`'s `ember why --unsafe` MUST print the obligation of every unsafe function on the chain it reports. The **compiler** entry `[TCB-4]` mandates ("The compiler is in the list") MUST carry a defined schema: compiler version and build hash, the backend in use (`[CG-C-*]` or LLVM), which analyses are self-hosted, and a link to the known-defect list for that version. A conforming implementation MUST NOT satisfy `[TCB-4]` by printing the word "compiler".
* `[TCB-5]` An `instrumented` fact is valid only for the exact foreign identity and
  contract it was measured against. Its evidence record MUST include: foreign library
  identity/version, header or interface content hash, effective ABI/toolchain hash,
  overlay/contract hash, instrumentation shim hash, test binary hash, test
  configuration and observation time. The record is stored under
  `target/<profile>/ffi-evidence/` and is referenced by content hash. The evidence record's default location is `.ember/ffi-evidence/` **beside the manifest, not under `target/`**, so a team may commit it and obtain reproducible grades across machines. `[MAN-5]`'s `[ffi] evidence = "<path>"` relocates it. It remains referenced by content hash.
* `[TCB-6]` The inputs `[TCB-5]` records are of two kinds. **Identity inputs** — foreign library identity/version, header or interface content hash, overlay/contract hash, instrumentation shim hash, and the effective ABI/toolchain hash — invalidate a record: a change to any of them makes it `stale`, and a stale record MUST NOT satisfy an `instrumented` claim, a verified build requirement, or `[FFI-37]`. **Environment inputs** — test binary hash, test configuration and observation time — are recorded and reported but do **not** alone make a record stale; `ember tcb` prints them so a reader can judge. A different STL or allocator genuinely changes whether a foreign function allocates, which is why the ABI/toolchain hash is an identity input and not an environment one.
* `[CLI-11]` `ember tcb [--json] [--module <m>]` prints the report. `ember audit`
  prints a one-page summary: the core safety analyses, the effect totals, the
  foreign boundary, and the verification counts.
* `[CLI-12]` `ember why --unsafe|--alloc|--block|--io|--lock|--ffi <path.to.fn>`
  prints the shortest call chain from that function to the thing named, using
  the effect sets `[EFF-1]` already computes.
* `[CLI-13]` `ember calls --foreign <path.to.fn>` lists every foreign function
  reachable from that function, with the chain to each and its grades. Run
  against a frame entry point it is the measure of how much of a migration
  remains, and it falls as subsystems are replaced.
* `[CLI-14]` `ember test --instrument-ffi` performs `[FFI-37]`'s run.
* `[BLD-11]` A package declares which `[STD-6]` layers it links. Omitting `alloc`
  removes allocating types from the build. Referring to one is a name-resolution
  diagnostic in the name-resolution namespace (`E1020`), not a build-system error
  code and not a lint. The build system remains responsible for determining the
  available symbol set; the resolver owns the user-facing missing-symbol diagnostic. — replace "(`E1020`)" with "(`E1021 name is not linked in this build`, whose `help` names the `[STD-6]` layer that supplies it and the manifest key that links it)". `E1021` is unused; `E1020` remains `[GRM-4]`'s and MUST NOT be reused. Add `docs/errors/E1021.md` under `[DIA-6]`.
* `[BLD-12]` A package that omits a layer MUST NOT depend on a package requiring
  it; the build reports the dependency path that reintroduces it.
* `[TST-12]` The conformance suite MUST contain a program per grade asserting
  the report gives it that grade, and MUST assert that a declaration alone never
  produces `checked` or higher.
* `[TST-13]` The suite MUST cover: a foreign reference imported unsafe and then
  promoted by an overlay; adoption, and double adoption; an instrumented run
  contradicting a declared effect; range construction proved and unproved;
  a contract proved, disproved and timed out; and `[CLI-13]`'s reachability
  against a known call graph. add: an instrumented run in which a declared fact is never exercised, asserting the resulting grade is `asserted` and `W5054` is emitted; and an `allocator = external` overlay whose negative effect claim is capped at `asserted`.

## XX.7 Formatter

`ember fmt` is deterministic and configuration-free except line width (default 100). Rules: 4-space indentation; one blank line between methods, two between items; trailing commas in multi-line argument/element lists; spaces around binary operators, none around `**` when both operands are atoms; `x: T = v` spacing; imports sorted (`std` first, then dependencies, then local) and merged; attributes one per line; the formatter preserves comments and blank-line groups (max 2 consecutive); `pass` inserted for empty blocks; long conditions broken after `and`/`or`. `[FMT-1]` `fmt(fmt(x)) == fmt(x)` and `parse(fmt(x)) ≡ parse(x)` are tested over the whole test corpus.

* `[FMT-2]` The formatter never produces a block-bodied lambda inside brackets: it emits the `=>` form when the body is a single expression, and otherwise leaves the programmer's named binding alone.
* `[FMT-3]` The formatter never emits `;` outside `[T; N]` and `[v; N]`. Since `[GRM-18]` makes `;` illegal as a statement separator there is nothing to split: one line already carries one statement.

## XX.8 Linter (`ember lint`, L-codes)

v1 lints: `L2001 unnecessary clone` (source not used again), `L2002 large Copy` (> threshold bytes passed by value), `L3001 potential cycle` (statically visible class cycles), `L3002 borrow held longer than necessary` (a borrow whose last use is far before its scope end and blocks a later access — suggests a block), `L3010 unsafe block larger than necessary`, `L3011 RefCell guard held across a call` (a `Ref`/`RefMut` guard live across a call that could re-enter the same cell, `[CELL-7]`), `L4001 allocation in hot loop` (allocation inside a loop of a `@simd`/`@parallel` body or inside functions named in `[lints.hot_paths]`), `L4002 dynamic dispatch on final type` (redundant `dyn`), `L5001 unsafe extern without contract`, `L5002 FFI copy` (conversion at the boundary copying > threshold bytes), `L7001 lock held across call that may block`.

* `[LNT-1]` **`L1001 unused binding`.** A local introduced by `x = expr` or `x: T = expr` and never read on any path is reported at `warn` by default. Names beginning `_` are exempt. Liveness comes from the borrow checker's existing analysis (XIX §4.7 step 3).
* `[LNT-2]` **`L1002 assignment declares a new binding`.** When `L1001` fires for a binding whose name is within Damerau–Levenshtein distance ≤ 2 of a mutable binding in scope at that point which is *not* read between the declaration and the end of its scope, the diagnostic names the near miss, carries a machine-applicable fix-it rewriting the name, and carries `note: `x = e` declares when `x` is not in scope and assigns when it is (GRM-4)`. `L1002` is `deny` under `edition_lints = "strict"`.
* `[LNT-3]` `L1001` and `L1002` are emitted by `ember build` and `ember check`, not only by `ember lint`. A diagnostic that fires only on a separate command does not close the footgun ADR-002 names, which is this rule's entire purpose.

* `[LNT-5]` `L2005`: a `@noreload` function calling a reloadable function from inside a loop (`[HR-10a]`). The call is legal and goes through the callee's thunk, but it reintroduces per-iteration indirection in exactly the code the annotation was applied to avoid; the lint names marking the callee `@noreload` as the fix.
* `[LNT-4]` `L2004`: a `gen fn` whose body contains no `yield` (`[CORO-2]`). It compiles and runs to completion on first resume, which is legal and almost never intended; the lint names `fn` as the fix.

## XX.9 Documentation

`ember doc` renders `##` doc comments to HTML/Markdown; every public function's page shows: signature with modes, **effects**, `@noalloc`-cleanliness, thread rules (`Send`/`Sync` of parameters and result), allocation behaviour (from effect analysis), and for FFI wrappers the underlying C declaration and contract. Doc examples in fenced ```` ```ember ```` blocks are compiled and run as tests by `ember test --doc`.

---

* `[DOC-1]` **Error pages ship with their errors.** Each phase's exit criteria include `docs/errors/EXXXX.md` for every code that phase introduces. A page contains: a minimal program that triggers the error; the rendered diagnostic; one paragraph on **why the rule exists** (not a restatement of the rule); and the fix as compilable code. Every fenced `ember` block under `docs/errors/` is built by `ember test --doc`: the failing example MUST fail with that exact code and the fixed example MUST compile. `[TST-4]`'s index generator fails CI for a code with no page.
* `[DOC-2]` **The user guide is a 1.0 artefact**, not v1.1. `docs/book/` MUST exist, MUST be the landing page with the specification linked from it as the reference, and every sample in it MUST be compiled and run by `ember test --doc`. Its "getting started" chapter is identical to milestone M0 and is tested against it; it MUST include a "coming from Python" chapter carrying the three-column table of silent and diagnosed differences (`/`, `%`, the `f32` literal default, `if xs:`, `is None`, chained comparison, `xs[-1]`, `let`, `with … as`, multi-statement closures) **and the short list of things that behave exactly as in Python**, because the absence of a row is not reassurance.
* `[DOC-3]` `ember doc` moves from Phase 8 to **Phase 4**, whose effects and derives it depends on; from that point each phase documents its own surface as it lands.
* `[DOC-4]` Guide, error pages and specification are published together each release.

## XX.10 Editor and language-server architecture

* `[IDE-3]` **Resilience.** Every stage after the parser MUST produce a complete result for a file containing errors. Name resolution binds an unresolvable path to `Def::Error`; type checking assigns `Ty::Error` to any expression it cannot type; `Ty::Error` unifies with every type. A file that does not parse MUST still yield hover, completion and document symbols over the regions `[AST-2]` recovered.
* `[IDE-4]` **No codegen in the request path.** Every editor capability MUST be answerable from AST + resolution + HIR alone; none may require MIR, monomorphisation, borrow checking or a C compiler. Borrow- and effect-derived information (`[DIA-8]`, `[EFF-10]`) is surfaced as a **best-effort overlay on a completed `check`** and MUST NOT block a response when none exists. (This is also what keeps the editor independent of ADR-006's backend choice.)
* `[IDE-6]` **Session lifetime.** No compiler crate may rely on process exit to reclaim memory. Interned symbols, interned types, `TypeInfo` and instantiation caches MUST live in an explicitly owned `Session` that can be dropped and rebuilt; `ember_span::Symbol`'s leaked interner is a **v1 defect, not a v1 licence**.



## Compile-time budget

Iteration speed is a specified property of Ember, not an emergent one. A language
that rebuilds in a second is a different tool from one that rebuilds in forty, and
the difference is what decides whether Part XVIII's hot reload is real. These are
requirements, tested in CI, and they constrain language and compiler design as hard
as any semantic rule. They are about **wall-clock build time**; XIX §4.11a's
`[MONO-2..9]` instantiation budget is a different instrument measuring a different
quantity, and the two compose.

* `[BUD-1]` **Reference machine and workload.** An 8-core / 16-thread x86-64 laptop
  CPU of 2020 or later (baseline: i7-1165G7 or Ryzen 7 4800H), 16 GB RAM, NVMe SSD,
  Windows 11 with MSVC, warm OS file cache, warm build cache. The workload is
  `bench/bigpkg`, a synthetic 50k-line package whose shape mirrors engine script and
  system code: 400 modules, 1,200 types, 6,000 functions, 15% generic, one C header
  import of Vulkan's size. `[BUD-1a]` **`bench/bigpkg` grows with the compiler.**
  Phase 0 lands a Parts II–VI subset that the Phase 0 compiler can build; each phase
  extends it to the features that phase adds, and only the subset a phase can
  compile is gated in that phase. Wiring a gate against a package the compiler
  cannot yet build would fail every build from Phase 0 onward.
* `[BUD-2]` **Budgets**, each a median of 15 runs after 3 discarded warm-up runs, on
  `[BUD-1]`'s machine and the largest `bench/bigpkg` subset the current phase
  compiles:

| # | Operation | Budget |
|---|---|---|
| B1 | `ember check` after a one-function-body edit | ≤ 250 ms |
| B2 | `ember build --reload` after a one-function-body edit | ≤ 1000 ms |
| B3 | `ember build` after a one-function-body edit, `debug` | ≤ 1.5 s |
| B4 | `ember build` after a signature change in a leaf module | ≤ 2.5 s |
| B5 | `ember build` after a change to a widely imported type | ≤ 8 s |
| B6 | clean `debug` build, cold cache | ≤ 40 s |
| B7 | clean `release` build | ≤ 90 s |
| B8 | `.embind` regeneration for a Vulkan-sized header | ≤ 3 s |
| B9 | compiler peak memory, clean build | ≤ 3 GB |

  **B2 is `[HR-1]`'s one second, and it is the only definition of it.** B2 measures
  edit to loadable image: compile and link of the changed module only. The load,
  plan, prepare and commit phases of Part XVIII are measured separately by
  `[HR-32]` and are not part of B2; `[HR-1]`'s end-to-end second is B2 plus those,
  and `[BEN-8]` gates the sum.
* `[BUD-2a]` **What `ember check` verifies.** B1 covers parse, resolve, type-check,
  borrow-check and the pre-monomorphisation effect check of `[EFF-*]`. It does
  **not** cover `[EFF-9]`'s `RuntimeCheck(k)` set, which XIX §4.10 defines over the
  post-optimisation program and which therefore requires codegen; `ember check`
  MUST report `RuntimeCheck` facts as unavailable rather than stale, and
  `[IDE-4]`'s editor path already depends on exactly this split.
* `[BUD-3]` **Two gates, and they are different.** (a) An **absolute** gate: a
  measured figure above its `[BUD-2]` budget fails CI. (b) A **regression** gate: a
  measured figure more than 15% above the recorded baseline for the same commit
  lineage fails CI even when it is inside budget, because a change that consumes
  the whole margin at once is the change worth catching. Both are reported;
  either failing fails the build.
* `[BUD-3a]` **CI is not the reference machine, so CI normalises.** Hosted runners
  vary by more than the gates do. Every CI run first executes
  `tests/perf/calibrate`, a fixed single-threaded and a fixed parallel workload with
  published reference times, and divides its measurements by the resulting factors
  before comparing. A run whose calibration itself varies by more than 10% across
  its own repetitions is **inconclusive**, not a failure: it is retried once and
  then reported as unmeasured. `[BUD-3b]` The gates apply only to figures whose
  measured spread is below the gate — a 15% gate over a distribution with a 20%
  interquartile range measures the runner, not the compiler — and
  `tests/perf/build/` MUST publish the observed spread beside every figure.
* `[BUD-4]` **Design constraints, recorded as consequences** so they are not
  re-argued per feature: (1) no analysis whose *correctness* requires whole-program
  knowledge — XIX §4.11a's instantiation decisions are an optimisation, taken over
  a package, and a build that skips them is still correct; (2) monomorphisation is
  cached per instantiation and reused across builds; (3) a module's interface hash
  carries the cross-module facts its dependants need; (4) codegen is per module and
  parallel; (5) `comptime` results are cached by input hash; (6) an unchanged module
  is never re-parsed. `[BUD-4a]` Constraint 3 is a statement about *what the hash
  covers*, not a claim that private bodies are irrelevant: effect sets are inferred
  from bodies (`[EFF-1]`), so a private body change does change the interface hash
  when it changes the function's effect set, and the build graph MUST treat it as
  such. A hash that ignored bodies would make `[EFF-4]`'s invalidation rule unsound.
* `[BUD-5]` **The budget has veto power.** A proposal that adds a language or
  compiler feature MUST state its measured effect on **B3**, the one-function-body
  `debug` build, and is rejected if that effect exceeds 5 percentage points or the
  measured spread of `[BUD-3b]`, whichever is larger. B3 is the row this rule
  names; B1 and B2 are reported alongside but do not gate a proposal. This is the
  principled basis on which Ember has no macros, no specialisation, no
  higher-kinded types and no unbounded trait-solver search — each is a known
  multi-second tax in a language that has it.
* `[BUD-5a]` **The rule applies to the features already accepted.** Part XVIII's
  hot reload costs `[HR-9]`'s ≤ 3% run-time indirection and no measurable B3 time,
  because thunk emission is per function and the reload manifest is written from
  data codegen already has. Part XVI's trampolines and member mapping cost
  `.embind` generation time only, gated separately by B8, and nothing at B3 for a
  package that imports no C++ header. Both figures MUST be reported by
  `tests/perf/build/` from Phase 7a, and a proposal that cannot state its figure
  is not ready to be accepted.
* `[BUD-6]` `ember build --timings` prints the per-phase breakdown for the build
  just run, and `--timings=json` writes it for tooling. It is how a programmer
  answers "why was that slow" without instrumenting the compiler.

---


## XX.11 Conformance profiles

A specification this size can be internally consistent and still be too large to
implement correctly in one pass. The danger is not a wrong rule; it is a **partial
implementation that becomes the de facto language**, with users depending on
whichever subset happened to work. Profiles make the subset a declaration rather
than an accident.

* `[CONF-1]` A compiler claiming to implement Ember MUST declare which profile it
  implements, and `ember --version` MUST print it. The profiles are cumulative:
  each includes every profile before it.
* `[CONF-2]` **Ember Core** — Parts II–VII and XIII: lexer, parser, types,
  `struct`/`enum`, ownership, borrowing, regions, functions and closures,
  `Result`/`Option`, modules, and range types (`[RNG-*]`). A Core implementation is
  a usable language for value-oriented code and is the Phase 0–2 exit.
* `[CONF-3]` **Ember Systems** — adds Parts VIII–XII and XIV: classes and RC,
  exclusivity, arenas and handles, effects, threads and jobs, SoA/SIMD/ECS, and
  compile-time reflection.
* `[CONF-4]` **Ember Native** — adds Parts XVI and XVII: the C and C++ importers,
  `.embind`, overlays, trust grades, the trampoline of `[FFI-39]`, and the GPU host
  model.
* `[CONF-5]` **Ember Dynamic** — adds Part XVIII: hot reload, schema migration,
  relocation and runtime inspection. `[HR-19]`'s bodies-only tier is the first
  half of this profile and MAY be declared as `Dynamic (bodies)`.
* `[CONF-6]` A profile is claimed only when **every** conformance test for the rules
  it names passes (`[TST-4]`). A partial profile is declared as the profile below
  it, never as the profile above with exceptions — an implementation that claims
  `Native` while failing three `[FFI-*]` tests is claiming `Systems`.

## XX.12 ABI protocol versions

The C ABI is the stable boundary, but there is more than one boundary, and a
mismatch on any of them is a silent memory-corruption bug rather than a link error.
Each therefore carries a version that is checked before any code runs.

* `[ABI-1]` Four protocols are versioned independently, each an integer that
  increments on any layout, calling-convention or symbol-set change:
  `EMBER_C_ABI` (the C header a package emits, `[FFI-28]`), `EMBER_RUNTIME_ABI`
  (package ↔ `ember_rt`), `EMBER_RELOAD_ABI` (reloadable image ↔ runtime, covering
  the manifest of `[HR-8]`, the thunk table and the schema encoding), and
  `EMBER_CPP_BINDING_ABI` (generated thunks ↔ the C++ side, `[BLD-FFI-5]`).
* `[ABI-2]` Each version is emitted as a defined symbol whose **name encodes the
  value** — the device `[BLD-FFI-1b]` and `[HR-12a]` already use — so a mismatch is
  an unresolved-symbol **link error** naming both versions, not a runtime crash.
  A protocol that cannot be checked at link time (`EMBER_RELOAD_ABI`, since images
  load at runtime) is checked on load and refuses with `E9037`.
* `[ABI-3]` `ember_module_init` verifies `EMBER_RUNTIME_ABI` before it does anything
  else and returns a distinguished failure code on mismatch, so a host embedding a
  stale package learns at initialisation rather than at first call.
* `[ABI-4]` A version increment is a **release-note obligation**: `[VER-*]`'s
  compatibility statement MUST name every protocol whose version changed and why.
* `[ABI-5]` These versions are independent of the language version. Two compilers
  claiming Ember 0.7.2 may differ in `EMBER_RELOAD_ABI` — reload metadata is
  implementation-defined — and MUST NOT be mixed in one process; they may not
  differ in `EMBER_C_ABI`, which `[FFI-28]` fixes.

## XVI.7a C++ exception policy

* `[FFI-43]` **Every imported C++ function carries an explicit exception policy**,
  and there is no default: an import whose policy the header and overlay both leave
  unstated is `E5061`, naming the two spellings. This is the one place a silent
  assumption becomes undefined behaviour crossing an ABI, so the specification
  requires the programmer to have decided.
  * `@ffi(throws = "translate")` — the generated thunk wraps the call in
    `try`/`catch (...)`, converts to `Result[T, CppError]` per `[FFI-24]`, and the
    Ember signature returns that `Result`. `CppError` carries `what()` where the
    exception derives from `std::exception` and an opaque marker otherwise.
  * `@ffi(throws = "noexcept")` — the thunk asserts the callee is `noexcept`; if it
    throws anyway the thunk calls `std::terminate` at the boundary. The Ember
    signature returns `T` directly. This is the correct declaration for a function
    the header marks `noexcept`, and the importer applies it automatically in that
    case rather than requiring it to be written.
* `[FFI-43a]` **A contradictory policy is rejected, not resolved.** An overlay
  declaring `throws = "noexcept"` for a function the header declares as potentially
  throwing, or two composed overlays (`[FFI-30c]`) declaring different policies for
  one function, is `E5062` naming both sources. An exception MUST NOT cross an
  Ember frame in either direction under any policy; `[FFI-39e]` states the inbound
  half of the same rule.


---

