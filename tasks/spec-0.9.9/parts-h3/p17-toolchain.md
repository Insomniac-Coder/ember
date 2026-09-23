---

# Part XVII — Toolchain, Diagnostics and Conformance

## XVII.1 The `ember` command

```text
ember new <name> [--lib | --bin | --cdylib]    create a package
ember run [file.em] [args…] [--hot | --interp] build and run; a single file needs no manifest
ember build [--profile <name>] [--target <triple>] [--backend c]
            [--emit tokens|ast|hir|mir|c|obj|header] [--out-dir <dir>]
            [--emit-optimization-report] [--report=engine|instantiations] [--timings[=json]]
            [--build-id] [--reload [--explain]] [-D warnings]
ember check [--syntax-only] [--timings]        type, borrow and effect checks without code generation
ember test [filter] [--doc] [--no-leak-check]  run @test functions and documentation examples
ember bench [filter] [--compare]               benchmarks; --compare runs the paired C programs
ember fmt [--check | --migrate]
ember lint
ember doc
ember update [package]                         re-resolve dependencies and rewrite ember.lock
ember explain <E-code | rule-id>               the error page or rule text
ember explain --borrow <file>:<line>           why each borrow live at that line is still live
ember explain --cycle <path> <Class[.field]>   one cycle-capable class or field
ember inspect <item> [--safety [--elided-only] | --alloc | --cost | --deterministic | --cycle
                      | --expand]
ember why --alloc|--block|--io|--lock|--sync|--unsafe|--ffi <item>   the call chain to that effect
ember calls --foreign <item>                   every foreign function the item can reach
ember bind <header> [--init [--merge] | --report [--baseline <file> | --write-baseline]
                     | --check <overlay> | --explain <fn> | --emit-embind]
ember shader-bind <reflection.json>            Annex D
ember tcb [--module <m>]                       the trusted-base report (Annex C)
ember toolchain list|install|default           manage installed C toolchains
ember clean
ember --help | --version [--matrix]
```

* `[CLI-1]` Every command accepts `--json` and exits non-zero on error.
* `[CLI-4]` *(changed in 0.9.9)* `ember run file.em` and `ember build file.em` accept a single file
  with no manifest, as a package named after the file with default settings. A first program needs no
  manifest and no `main`: `println("hello")` on its own is a complete program (`[FN-8]`).
* `[CLI-2]` `ember build --emit c --out-dir <dir>` writes the C sources without invoking a C compiler,
  for build systems that compile them themselves. `--emit tokens|ast|hir|mir|obj` write the other
  stages, for tools and for reporting compiler defects.
* `[CLI-9]` `ember check --syntax-only` lexes and parses only, reporting `E00xx` and `E01xx`.
* `[CLI-12]` `ember why --alloc <item>` (and `--block`, `--io`, `--lock`, `--sync`, `--unsafe`, `--ffi`)
  prints the shortest call chain from the item to an operation with that effect, whether or not the
  item carries a contract; with `--unsafe` it prints the obligation of every unsafe function on the
  chain.
* `[CLI-13]` `ember calls --foreign <item>` lists every foreign function the item can reach, with the
  contract of each.
* `[CLI-15]` `ember --help` lists every command and flag of this section; one missing from it, or from
  §XVII.1's listing, is a defect.
* `[CLI-20]` *(new in 0.9.9)* `ember fmt --migrate` rewrites source written for 0.9.8 into 0.9.9 where
  the change is mechanical — `::` to `.`, integer `/` to `//`, `@view` and `with_views` removal,
  `Box[dyn fn]` to an owned callable type, `for k, v in m:` over a `Map` to `for k, v in m.items():`
  (`[CTL-1]`) — and lists what it could not rewrite.
* `[CLI-21]` *(new in 0.9.9)* `ember run --interp` runs a program on the compile-time evaluator instead
  of compiling it: no C compiler is needed; console and file I/O work; foreign calls and threads are
  unavailable (`E6010`); results equal a compiled run's.
* `[TOOL-2]` `ember toolchain install cc` installs a pinned Clang and linker and selects it with `[build]
  c_compiler = "bundled"`; `ember toolchain list` and `default` show and choose among installed ones.
  A working Ember installation never requires a separately installed C toolchain; a system compiler may
  be chosen explicitly, and its configuration is recorded with each build.
* `[TOOL-3]` When no C compiler is found, `E9002`'s message says so and its help names the remedy for
  the host (on Windows, the Visual Studio Build Tools C++ workload, or `ember toolchain install cc`);
  a bare missing path is never the primary message.
* `[CLI-11]` `ember audit` prints a one-page summary of a package's safety surface: `unsafe` blocks
  and their notes, the effect totals, the foreign boundary and its contracts, and the trusted-base
  report of Annex C.
* `[TOOL-1]` Each release publishes a self-contained toolchain archive per supported host (Windows
  x64, Linux x64, macOS arm64) with `ember`, the runtime, `std` and editor support, and a one-line
  installer that puts `ember` on `PATH`.
* `[TOOL-4]` `ember --version` prints the compiler version, the language version, the backend, and the
  resolved C compiler and linker, so a bug report carries its environment. **The first hour:** on a
  clean Windows or Linux machine, installing Ember, `ember new hello`, `cd hello` and `ember run` print
  `hello, world` in under five minutes and at most six typed commands; a regression blocks a release.
* `[CLI-19]` *(new in 0.9.9)* **The implementation matrix.** `ember --version --matrix` prints, for
  every Part, rule family, command and flag of this document, whether this compiler implements it.
  A command, flag, attribute or construct the compiler does not implement is rejected with `E0900`
  naming it (`[PHIL-12]`), never ignored.
* `[CLI-5]` `ember bind --init <header>` writes a starter overlay: every derived contract, every unknown
  fact as a `TODO(…)` word (`[FFI-11c]`), every skipped declaration as a comment with its reason. It
  compiles unchanged and never overwrites a file; with `--merge` it adds only the declarations missing
  from an existing overlay, keeping hand-written contracts and comments.
* `[CLI-6]` `ember bind --report` lists every skipped declaration and every declaration still
  `unsafe` with its unknown facts, and exits non-zero on a skip (`--baseline` limits that to new ones).
* `[CLI-7]` `ember bind --check <overlay>` checks `[FFI-12]` and prints how many declarations are
  safe, still `unsafe`, and skipped.
* `[CLI-10]` `ember build --report=engine` is a report, not a profile: it changes nothing about the
  program, and summarises allocations, surviving count operations and runtime checks with their
  reasons, vectorised loops and why others were not, FFI wrapper costs, inlining decisions, and classes
  that could be structs (advisory; never applied automatically).

## XVII.2 The manifest (`ember.toml`)

```toml
[package]
name = "particles"
version = "0.1.0"
language = "0.9.9"
kind = "bin"                      # bin | lib | staticlib | cdylib
layers = ["core", "alloc", "sync", "io"]
edition_lints = "strict"          # warnings listed in [lints] as "error" fail the build

[dependencies]
geometry = { path = "../geometry" }
parser = { git = "https://example.org/parser.git", rev = "4f2a9c1" }
std = "0.9.9"                     # implicit; may be pinned

[build]
entry = "src/main.em"
target = "native"
backend = "c"
c_compiler = "auto"               # auto | bundled | msvc | clang | gcc
runtime = "static"                # static | shared (shared is required by hot reload)
max_instantiations = 200

[profiles.release]
lto = "thin"

[profiles.profiling]
inherits = "release"
debug_info = "full"

[comptime]
max_steps = 100000000
max_heap_mb = 256

[jobs]
workers = "auto"

[realtime]
contracts = ["noalloc", "nolock", "noblock", "nopanic(explicit)"]

[link]
libs = ["m"]

[lints]
unused = "warn"
potential_cycle = "warn"
large_copy = { level = "warn", threshold = 256 }
```

* `[MAN-1]` *(changed in 0.9.9)* An invalid manifest, including one with an unknown key, is `E9001`,
  naming the key and the keys that section accepts.
* `[MAN-2]` `ember.lock` records the resolved dependencies with content hashes; `--locked` fails if it
  would change.
* `[MAN-3]` *(changed in 0.9.9)* Every key in `[lints]` names a lint the compiler defines (`E9010`
  otherwise); its value is a level (`allow`, `warn`, `error`) or a table with `level` and the lint's
  parameters.
* `[MAN-8]` *(new in 0.9.9)* The sections are: `[package]` (`name`, `version`, `language`, `kind`,
  `layers`, `edition_lints`); `[dependencies]` (a version requirement, `{ path }`, or `{ git, rev }`);
  `[build]` (`entry`, `target`, `backend`, `c_compiler`, `runtime`, `max_instantiations` `[MONO-3]`,
  `reload` `[MAN-7]`); `[profiles.<name>]` (`[PRF-3]`); `[c]` (`[BLD-FFI-1]`); `[link]`; `[jobs]`
  (`[JOB-1]`); `[realtime]` (`[EFF-19]`); `[comptime]` (`max_steps`, `max_heap_mb`, `[CT-3]`);
  `[lints]` (`[MAN-3]`); and `[cpp.<project>]` and `[ffi]` (Annex C). **No manifest key turns off a
  safety check or changes what a program means** (`[PRF-1]`).
* `[MAN-7]` `[build] reload` is `"opt-in"`, `"all"`, `"bodies"` or `"none"`; it is forbidden in
  `shipping` (Annex B).
* `[VER-5]` Package versions are semantic; a requirement `"1.2"` means `>= 1.2.0, < 2.0.0`; resolution
  picks the highest version satisfying every requirement, and `ember update [package]` recomputes it.
  Two majors of one package may coexist, distinguished in symbol names.

## XVII.3 Builds

* `[BLD-1]` The module (one file) is the unit of front-end caching; the package is the unit of
  monomorphisation and code generation.
* `[BLD-2]` A module's front-end result is keyed by its source, the compiler and language versions,
  the package configuration and the **interface hashes** of what it imports (signatures, layouts,
  effect sets, inline bodies), so editing a private body recompiles that module and re-checks the
  contracts of callers only if the body's effect set changed (`[EFF-4]`).
* `[BLD-8]` Within a module, editing one function body re-checks that function alone.
* `[BLD-4]` The C compiler and linker run through a generated Ninja file, so C compilation is
  incremental too.
* `[BLD-5]` Output goes to `target/<profile>/{bin,lib,c,obj,bind,inspect}`.
* `[BLD-7]` Front-end stages run in parallel across modules by default (`-j`, default the number of
  physical cores).
* `[BLD-6]` `lto = "off" | "thin" | "on"`. A value the C toolchain does not support is replaced by the
  nearest one it does, and the build record says so. No correctness property depends on LTO.
* `[BLD-9]` `--timings` writes a per-stage, per-module timing report (`--timings=json` for tools).
* `[BLD-10]` *(changed in 0.9.9)* The compile-time budgets below are release gates.
* `[BUD-1]` They are measured on a recorded reference machine (an 8-core laptop CPU of 2020 or later,
  16 GB, NVMe, warm caches) over `bench/bigpkg`, a generated 50k-line package of 400 modules with
  15 % generic code and one C header import of Vulkan's size.
* `[BUD-2]` Each figure is the median of 15 runs after 3 warm-up runs:

  | # | Operation | Budget |
  |---|---|---|
  | B1 | `ember check` after a one-function-body edit | ≤ 250 ms |
  | B2 | `ember build --reload` after a one-function-body edit | ≤ 1 s |
  | B3 | `ember build` after a one-function-body edit, `debug` | ≤ 1.5 s |
  | B4 | `ember build` after a signature change in a leaf module | ≤ 2.5 s |
  | B5 | `ember build` after a change to a widely imported type | ≤ 8 s |
  | B6 | clean `debug` build, cold cache | ≤ 40 s |
  | B7 | clean `release` build | ≤ 90 s |
  | B8 | `.embind` regeneration for a Vulkan-sized header | ≤ 3 s |
  | B9 | compiler peak memory, clean build | ≤ 3 GB |

* `[BUD-3]` A figure above its budget fails CI, and so does one more than 15 % above the recorded
  baseline even inside its budget.
* `[BUD-3a]` CI normalises its measurements by a fixed calibration workload, and a run whose
  calibration varies by more than 10 % is inconclusive, not failed.
* `[BUD-3b]` A gate applies only to a figure whose measured spread is below the gate, and every figure
  is published with its spread.
* `[BUD-5]` A proposed language or compiler feature states its measured effect on B3 and is rejected if
  it adds more than 5 percentage points.
* `[BLD-11]` A package that omits a `[STD-6]` layer cannot use it or depend on a package that does.
* `[BLD-13]` Builds are reproducible: the same inputs give a byte-identical artefact, with no
  timestamps, absolute paths, host names or hash-order dependence. `--build-id` prints the hash of the
  inputs (`[DET-7]`).

## XVII.4 Profiles

* `[PRF-1]` *(changed in 0.9.9)* **A profile never changes what a program means.** Every profile
  accepts the same programs, performs the same safety checks (bounds, overflow, exclusivity, stale
  handles, `RefCell`), panics in the same places with the same messages, and computes the same results.
  The one check that differs is `debug_assert`, checked only where `[PRF-3]` says: it may not have
  side effects (`W2016`), so a program whose assertions hold computes the same results in every
  profile. Only the following differ between profiles.
* `[PRF-3]` *(new in 0.9.9)* **What a profile may set.**

  | Setting (key) | `debug` | `release` | `shipping` |
  |---|---|---|---|
  | optimisation (`opt`) | 0 | 2 | 3 |
  | debug information (`debug_info`: `full`, `lines`, `none`) | full | lines | none |
  | symbols stripped (`strip`) | no | no | yes |
  | backtrace on panic (`backtrace`) | yes | yes | no |
  | `debug_assert` (`debug_assert`) | checked | compiled out | compiled out |
  | leak and cycle report at exit (`leak_report`, `[WK-15]`) | on | off | off |
  | lock-order report (`lock_order`, `[THR-4]`) | on | off | off |
  | C sanitizers (`sanitizers = […]`) | allowed | allowed | not allowed |
  | link-time optimisation (`lto`) | off | thin | on |

  A package may define more profiles, each `inherits` one of these and overrides keys of this table.
  A key outside the table is `E9001`.
* `[PRF-2]` Hot reload is a build mode admitted in `debug` and `release`, not a profile (Annex B).

## XVII.5 Tests, gates and conformance

```ember
#$ test: compile-fail
fn main():
    a = [1, 2, 3]
    b = a
    a.push(4)          #$ error[E3040]: use of moved value `a`
```

* `[TST-0]` Test annotations are line comments beginning `#$`, read from the raw source text, so a test
  may expect a failure in the lexer.
* `[TST-1]` `#$ error[E…]: text`, `#$ warning[…]` and `#$ note` assert a diagnostic whose primary span
  starts on that line; unexpected and missing diagnostics both fail.
* `[TST-2]` `#$ stdout:`, `#$ exit: N`, and `#$ assert-c: contains("…")` check output, exit status and the emitted C.
* `[TST-3]` `@test` functions run in the test binary, each isolated; `@should_panic` expects a panic.
* `[TST-4]` Every rule of this document has a directory `tests/conformance/<RULE-ID>/`, listed by
  `tools/rule_index.py`, which fails CI for a rule with none.
* `[TST-4a]` Each rule's directory has an **accept** case and, for each diagnostic code the rule names,
  a **reject** case with the exact expected output.
* `[TST-4b]` Which rules need a reject case is decided mechanically: a rule that names a code needs one
  per code; a rule naming none (a layout guarantee, a permission to optimise) is waived, and the
  generated waiver list is committed.
* `[TST-11]` *(changed in 0.9.9)* Besides the per-rule cases, the suite covers these scenarios:
  exclusivity (scalar reads, `Copy` field writes, iteration through `let` fields, mutation during
  iteration, non-`Copy` field assignment during a loop); loop-hoisted checks (one stable receiver, a
  loop-invariant receiver, an escaping receiver, a virtual call that defeats the proof, two receivers
  that must not merge); callables (fresh regions per call, escape rejection, mode mismatches);
  multi-region views (independent owners, one field's source dying, storage in a class or static,
  region inequality never used as `noalias`); arenas (return provenance, `CapacityError.Full`, no
  cursor movement after construction, uninitialised allocation); `Span` and `MutSpan` (iteration,
  reborrow reuse, `chunks(0)` panics, `split_at`); `Shared`/`Weak` (upgrade after release, upgrade
  during `drop`, atomic upgrade for `@sync`); cycles (direct, three-node, through generic containers,
  weak and foreign edges, a runtime cycle static analysis missed).
* `[TST-6]` *(changed in 0.9.9)* Appendix A's code is generated from a fixture that is compiled in CI.
  The fixture declares the items the printed fragment names, so the printed block stays a fragment
  (`ember,fragment`) while the code in it is compiled.
* `[TST-7]` *(changed in 0.9.9)* Every ` ```ember ` block in this document is extracted and must pass
  `ember check` (`--syntax-only` for blocks that name items they do not declare, marked
  ` ```ember,fragment `). A block may opt out as ` ```ember,ignore ` only with a stated reason: a
  signature sketch or a deliberate error example. Overlay source is marked ` ```ember,overlay ` and
  checked as an overlay (`[GRM-35]`).
* `[TST-8]` `tests/firstweek/` holds at least 24 first-draft programs a newcomer plausibly writes in
  week one (a text adventure, a CSV summariser, a scene graph with parent links, an event bus, an
  inventory, a path finder…), each written by someone who has read only Part I and Annex A and
  committed unmodified.
* `[TST-9]` Each is marked `accepted` or `rejected(<shape>)`; a rejection whose shape is in no
  catalogue (§XVII.6) blocks the release, and the help of the shape, applied literally, must make the
  program compile.
* `[TST-10]` The acceptance rate is published with each release; a release that lowers it says why.
* `[TST-27]` *(new in 0.9.9)* **The C gate.** Every accepted program in the test suite is compiled
  through the C compiler with warnings as errors on each supported host compiler. A C compiler error
  on an accepted program is a compiler defect and fails CI (`[CG-C-2]`).
* `[TST-28]` *(new in 0.9.9)* **The performance gate.** `tests/perf/` holds benchmark programs, each
  with an equivalent C program, a checker of identical output, and a threshold (for example, within
  10 % of C at `-O2`). A regression past a threshold fails CI. This is the instrument for "as fast as
  C"; no performance claim in this document is considered met until its benchmark exists.
  `ember bench --compare` runs the pairs.
* `[BEN-1]` Each benchmark runs at least 3 untimed and 30 timed repetitions on one pinned core and
  reports the median and the 95 % confidence interval of the median for each side and their ratio.
* `[BEN-2]` A gate fails only when the lower bound of the ratio's interval exceeds the threshold; a
  point estimate above it whose interval includes it is inconclusive and re-run.
* `[BEN-3]` Each run also measures the C program against a second copy of itself; if that ratio's
  interval excludes 1.00 ± 0.02 the run is void and the machine unfit for gating.
* `[BEN-4]` Each benchmark records retired instructions; a change beyond ± 0.5 % against the baseline
  fails regardless of timing.
* `[BEN-5]` Both sides are built with the same optimisation level, LTO setting, floating-point model
  and target-CPU flags; a mismatch voids the run.
* `[BEN-6]` Thresholds: scalar and tight loops ≤ 1.05× C; SoA and SIMD code ≤ 1.10×; `@noalloc` paths
  assert allocation counts exactly; a direct FFI call executes the same instructions as C's call;
  counted-object code ≤ 1.15× a C++ intrusive count (non-atomic, or atomic for `@sync` classes).
* `[EXC-13]` The performance suite contains class-handle loops with one stable receiver, a
  loop-invariant receiver, an escaping receiver, a virtual call that defeats the proof and two
  receivers that must not merge, and asserts one hoisted check for the first two and per-access checks
  for the rest (`[EXC-8]`).
* `[TST-29]` *(new in 0.9.9)* **Honest baselines.** A gate with a baseline of known failures reports the
  baseline size beside its result; the baseline only shrinks, and a gate whose baseline covers more
  than a tenth of its cases reports itself as `baselined`, not green.
* `[TST-30]` *(new in 0.9.9)* The test runner reports every failure of a run, not only the first, and a
  test that fails intermittently is quarantined by name and listed in the run summary.
* `[TST-5]` A scripted debugger session (breakpoint by Ember line, stepping, locals by Ember name) runs
  in CI on every supported host; a host where it cannot run is reported as debug-unverified, with a
  recorded waiver.

## XVII.6 Diagnostics

* `[DIA-1]` A diagnostic has a code, one primary span, labelled secondary spans, an optional `help`
  (with a machine-applicable fix-it where possible) and notes. `--json` gives the same structure.
* `[DIA-2]` Messages start lowercase, have no trailing period, name the thing, and say what is wrong;
  `help` says what to do; a message never says "you" and never blames the programmer. The primary
  message explains the failure in terms of the source program; compiler-internal facts (region
  numbers, internal names) may appear only as secondary evidence.
* `[DIA-3]` Ownership and borrow errors include the "later used here" label and a concrete fix from the
  catalogue (§XVII.6.1).
* `[DIA-4]` Contract errors print the whole call chain (`[EFF-6]`).
* `[DIA-5]` FFI errors name the header and the C declaration.
* `[DIA-6]` Every code has a page, `docs/errors/EXXXX.md`, with a program that triggers it, the
  rendered diagnostic, why the rule exists and the fix; `ember explain` prints it. A code without a
  page fails CI, and each page's examples are built by `ember test --doc`: the failing one must fail
  with that code and the fixed one must compile.
* `[DIA-6a]` The code registry is exhaustive in both directions: every code this document names is
  registered, and every registered code is named here (§XVII.9). Each code is defined by exactly one
  rule.
* `[DIA-7]` Every ownership and borrow error is classified into a shape of §XVII.6.1 and emits that
  shape's help.
* `[DIA-7a]` Every code in `E3000`–`E3499` belongs to exactly one shape of §XVII.6.1; a code with no
  row is never emitted, and `tools/rule_index.py` fails CI on one.
* `[DIA-8]` `ember explain --borrow <file>:<line>` prints, for each loan live at that line, where it was
  created, the lines its region covers and the later use that keeps it alive: `borrow of v created at
  12:9, live through 19, because s is used at 19:14`. It reads the borrow checker's own loans
  (`[BCK-1]`).
* `[DIA-10]` The help of shapes B1, B2, B4, B5 and B9 names a concrete API or construct (`split_at`,
  `retain`, `Weak`, `owned fn`), never a category such as "consider restructuring".
* `[DIA-11]` For shape S1 the diagnostic reports the check's reason (`[EFF-11]`); when it is
  `not_provable_in_principle` or `inherent_to_mechanism`, the suggestion is to remove the contract or
  change the data structure, never to restructure code that cannot be improved.
* `[DIA-13]` Every shape has a rendered snapshot under `tests/ui/` and a `.fixed.em` companion showing
  the help applied, which must compile.
* `[DIA-15]` Suggestions are computed from data the compiler already holds (scope tables, type
  information, an index of exported names); none requires speculative type checking.
* `[DIA-18]` A foreign call rejected because its contract has unknown facts (`E5002`) names each missing
  fact and prints the overlay line that supplies it.
* `[DIA-12]` Every name and type error is classified into a shape of §XVII.6.2. An unclassified error
  is logged, and CI fails on the log.
* `[DIA-9]` A diagnostic never suggests `unsafe`, `Cell`, `RefCell`, `Shared` or `clone()` first when a
  structural fix exists.
* `[DIA-16]` When a diagnostic suggests moving a value into a class, `Shared` or `RefCell`, a note names the run-time cost.
* `[DIA-14]` Only the first error of a cascade is reported: nothing is reported about an expression
  whose type is already an error, a name bound to a failed import, or a call to a failed declaration.
* `[DIA-20]` *(new in 0.9.9)* A run of invalid bytes or characters is one diagnostic, not one per byte.
* `[DIA-21]` *(new in 0.9.9)* **Python habits.** Each of these is recognised and answered with the
  Ember form as a machine-applicable fix-it:

  | Written | Help |
  |---|---|
  | `True`, `False`, `None` as a value where no `Option` is expected | `true`, `false`; `None` needs an `Option` type |
  | `def f():` | `fn f():` |
  | `str(x)`, `int(s)`, `float(s)`, `int(x)` | `x.to_string()` or `f"{x}"`, `s.parse[int]()`, `s.parse[float]()`, `x as int` |
  | `len(s)` for a string (`E2073`) | `s.char_count()` (Python's count) or `s.len()` (bytes) |
  | `min(xs)`, `max(xs)` of one collection | `xs.iter().min()`, `xs.iter().max()` (an `Option`) |
  | `xs.append(x)`, `s.strip()`, `s.startswith(p)`, `s.upper()`, … | the `[STD-13]` table (Appendix E) |
  | `xs[a:b]` | `xs[a..b]` |
  | `if xs:` | `if not xs.is_empty():` (`[CTL-0]`) |
  | `let x = 5`, `var x = 5` | `x = 5` |
  | `x == None` | `x is None` |
  | `with open(p) as f:` | `with f = fs.File.open(p)?:` |
  | `raise e`, `try:`, `except E:`, `finally:` | `return Err(e)`, `?`, `match`, `defer:` (Part XIII) |
  | `lambda x: e` | `fn(x) => e` |
  | `self` missing from a method's parameters | add `self` |
  | `a / b` on integers | `a // b`, or `a as float / b` (`[TYP-28]`) |
  | `if x = 5:` | `if x == 5:` |
  | `case p:` inside `match` | `p:` (or `p =>`), without `case` |
  | `global x`, `nonlocal x` | a `static` or a class field for shared state; a closure captures `x` without a declaration (`[CLO-2]`) |
  | `*args`, `**kwargs` in a signature | an `Array` or `Span` parameter, or keyword arguments with defaults; Ember has no variadic functions (§I.6) |
  | `"%d" % n`, `"{}".format(n)` | `f"{n}"` (`[LEX-19]`) |

* `[DIA-22]` *(new in 0.9.9)* A diagnostic caused by a callee's contract or signature is reported at the
  call site in the programmer's code, with the callee's declaration as a secondary span.
* `[DIA-23]` *(new in 0.9.9)* A run-time panic names Ember entities — the class, field, variable, file
  and line — never a C symbol or a mangled name.
* `[DIA-24]` *(new in 0.9.9)* **Name suggestions (shape N1).** A candidate is suggested when its
  Damerau–Levenshtein distance from the unknown name is at most 1 for names of up to 4 characters and
  at most 2 otherwise, and it is of the right kind for the position (a type where a type is expected,
  a value where a value is). At most three are suggested, closest first, then same file before other
  files, then by qualified name.

### XVII.6.1 Ownership and borrow shapes

| Shape | Situation | Codes | Primary help |
|---|---|---|---|
| O1 | use after move | `E3040` | `x.clone()` if `Clone`; else reorder so the last use precedes the move; for a collection, a note that assignment moves a list where Python's aliases it (Appendix E) |
| O2 | move out of a container or field | `E3010`–`E3013` | `mem.take`, `mem.replace`, `swap`; `pop`, `swap_remove`, `remove`, `drain` |
| O3 | move inside a loop | `E3041` | declare inside the loop, clone per iteration, or `mem.take` |
| O4 | whole use after a partial move | `E3042` | reassign the field, or destructure up front |
| O5 | closure moves out a capture | `E3030` | declare the parameter `once fn` |
| O6 | borrow of an uninitialised place | `E3050` | name the path on which it is uninitialised |
| O7 | explicit `x.drop()` | `E3070` | `mem.drop(x)` |
| O8 | scope guard moved or leaked | `E3014`, `E3015` | bind it with `with` (`[THR-5]`) |
| O9 | `self` escapes its `drop` | `E3016` | `mem.take` the data out |
| B1 | two mutable borrows of one place, e.g. two indices | `E3022` | `split_at`, `chunks_mut`, `iter_mut`, `swap(i, j)`; `SoA` columns |
| B2 | mutation while iterating | `E3020` | `retain`, `drain`, collect first, or an index loop |
| B3 | shared and mutable overlap | `E3021` | end the shared borrow first, or copy the value out |
| B4 | two writers of one value | `E3023` | a single owner passing `mut`; `Cell` for a counter |
| B5 | self-referential struct | `E3024` | store an index or `Handle`; split the struct |
| B6 | returned view not derived from a parameter | `E3062` | return an owned value, or `@borrows(p)` |
| B7 | borrow outlives its source | `E3060` | move the source outward, return an owned value, or bind with `with` |
| B8 | a method takes all of `self` | `E3025` | `[BRW-10]` covers private methods; else take the fields as parameters |
| B9 | closure outlives its captures | `E3026` | `owned fn`, or a scope (`[THR-5]`) |
| B10 | `mut` argument that is not a mutable place | `E3027` | bind to a local first |
| B11 | disjointness not provable | `E3095` | `assert_disjoint` (`[DSJ-1]`) |
| B12 | view stored where it outlives its source | `E3063` | store an owned value or an index |
| B14 | multi-region result provenance cannot be inferred | `E3065` | return an owned value, or the views as separate results |
| B15 | callable parameter-mode mismatch | `E2228` | the expected `fn` type, spelled |
| X1 | static exclusivity conflict | `E3080` | end the first access first; copy the value out |
| S1 | `@static_safe` violated | `E4030` | the exact access and why it is dynamic (`[EFF-13]`, `[DIA-11]`) |
| A1 | arena view outlives the arena; arena allocation of a type with `drop`; parent used during a scope | `E3061`, `E3090`, `E3096` | move the arena outward, copy out, or end the scope first |
| U1 | operation needs `unsafe` | `E3100`, `E3105` | the safe API that does the same, else an `unsafe:` block with a `# SAFETY:` note |
| R1 | the reason is far from the error | — | `ember explain --borrow <file>:<line>` |

### XVII.6.2 Name and type shapes

| Shape | Situation | Primary help |
|---|---|---|
| N1 | unknown name | up to three candidates (`[DIA-24]`) |
| N2 | name exported elsewhere | the import line, as a fix-it |
| N3 | unknown method | nearest member; the interface that would supply it; a Python name per `[DIA-21]` |
| N4 | scalar mismatch | the exact lossless `as`, and which side set the expected type; for an `int` and a `float` operand (`i * 0.5`), `i as float * 0.5` as a machine-applicable fix-it |
| N5 | literal does not fit (`E2010`) | the suffix or annotation that fits |
| N6 | wrong number of arguments | the signature with names and modes |
| N7 | mutating an immutable place | the declaration to change, with its location |
| N8 | argument does not fit the parameter's mode | the mode in the callee's signature and the caller's fix (`.clone()`, a mutable local, or ending the value's use before the call); a call site never writes a mode (`[FN-2a]`) |
| N9 | missing bound (`E2040`) | the bound to add, and the one module where `extend` may be written (`[TYP-20]`) |
| N10 | not usable as `dyn` (`E2050`) | the method and the clause it violates |
| N11 | ambiguous method (`E2070`) | `I.m(recv, …)` for each candidate |
| N12 | indentation (`E0002`–`E0004`) | the expected column and the line that set it |
| N13 | a field assigned (`self.x = …`) but not declared (`E2072`) | `field 'x' is not declared in class C`, with a machine-applicable fix-it adding `x: <inferred type>` to the class body |

### XVII.6.3 Code ranges

| Range | Area |
|---|---|
| `E0000`–`E0099` | lexing, indentation, directives, `E0900`/`E0901` (`[PHIL-12]`) |
| `E0100`–`E0499` | parsing |
| `E1000`–`E1499` | names, modules, visibility |
| `E2000`–`E2499` | types, inference, interfaces, patterns |
| `E3000`–`E3499` | ownership, borrows, regions, exclusivity, drops |
| `E4000`–`E4499` | effects and contracts, SIMD |
| `E5000`–`E5499` | FFI and interop |
| `E6000`–`E6499` | compile-time evaluation |
| `E7000`–`E7499` | concurrency |
| `E8000`–`E8499` | layout and GPU layout |
| `E9000`–`E9499` | build, manifest, toolchain |
| `L…`, `W…` | lints and warnings |

## XVII.7 Formatter, linter and documentation

* `[FMT-1]` *(changed in 0.9.9)* `ember fmt` writes LF line endings, four-space indentation and at most 100
  columns, whatever the platform.
* `[FMT-2]` The formatter uses the `=>` form of a lambda whose body is one expression.
* `[FMT-3]` The formatter never emits `;` outside `[T; N]` and `[v; N]`.
* `[LNT-1]` `L1001 unused binding`: a local never read on any path (names starting `_` are exempt).
* `[LNT-2]` `L1002 assignment declares a new binding`: an unread new name within distance 2 of a
  mutable binding in scope, which is usually a typo; its fix-it rewrites the name, and it is an error
  under `edition_lints = "strict"`.
* `[LNT-3]` `L1001` and `L1002` are reported by `ember build` and `ember check`, not only `ember lint`.
* `[LNT-4]` `L2004`: a `gen fn` with no `yield`.
* `[LNT-5]` `L2005`: a `@noreload` function calling a reloadable one in a loop.
* `[LNT-6]` *(new in 0.9.9)* `ember lint` also reports, at `warn` unless `[lints]` says otherwise:
  `L2001` unnecessary clone; `L2002` large `Copy` value passed by value (`large_copy = { threshold =
  256 }`); `L3002` borrow held longer than necessary; `L4001` allocation in a hot loop; `L4002`
  dynamic dispatch on a final type; `L4003` an `Array[int]` or `Array[float]` of more than 64 KiB
  indexed in a hot loop whose values provably fit 32 bits; `L5001` `unsafe extern` declaration with no contract; `L5002`
  copying conversion at the FFI boundary; `L7001` lock held across a call that may block; and `L3001`
  potential reference cycle (`potential_cycle`, `[WK-6]`).
* `[DOC-1]` Error pages ship with their errors.
* `[DOC-2]` The user guide (`docs/book/`), with a "coming from Python" chapter built on Appendix E, is
  published with each release, and every sample in it is run by `ember test --doc`.
* `[DOC-3]` `ember doc` presents, for every function that produces or consumes a view, its generated
  contract: ownership, what the result borrows, effects and allocation, derived from the signature.
* `[DOC-4]` The guide, the error pages and this specification are published together for each release.

## XVII.7a Editor support

* `[IDE-3]` Every compiler stage after parsing produces a complete result for a file with errors: an
  unresolvable name and an untypable expression become error placeholders that satisfy everything, so
  hover, completion and symbols keep working in a broken file.
* `[IDE-4]` Every editor request is answered from parsing, name resolution and type checking alone,
  never from code generation or a C compiler; borrow- and effect-derived information is shown from the
  last completed `ember check` and never blocks a response.
* `[IDE-6]` The compiler keeps all its state in a session object that can be dropped and rebuilt, so a
  long-running editor process does not grow without bound.
* `ember lsp`, a language server over stdio, is reserved for a named milestone before 1.0.

## XVII.8 Conformance profiles and ABI versions

* `[CONF-1]` *(changed in 0.9.9)* A compiler declares the profile it implements, and claims it only
  when every conformance test of every rule in it passes; a partial implementation claims the profile
  below, never the one above with exceptions.
* `[CONF-2]` **Ember Core**: Parts II–VII, X, XIII and XV's core and alloc layers.
* `[CONF-3]` **Ember Systems**: adds Parts VIII, IX, XI, XII and XIV.
* `[CONF-4]` **Ember Native**: adds Part XVI. Annex C (C++) is a separate, optional claim.
* `[CONF-5]` **Ember Dynamic**: adds Annex B (hot reload).
* `[ABI-1]` The runtime ABI, the hot-reload protocol, the module protocol and the export-table protocol
  are versioned independently of the language; each version is a linked symbol whose name encodes the
  number, so a mismatch fails at link time, and `ember_module_init` checks the runtime ABI first.
* `[ABI-2]` A protocol that cannot be checked at link time (hot-reload images, loaded at run time) is
  checked on load and refused with `E9037`.
* `[ABI-3]` `ember_module_init` checks the runtime ABI version before anything else and returns a
  distinct failure code on a mismatch.
* `[ABI-4]` A release notes every protocol whose version changed, and why.
* `[ABI-5]` The protocol versions are independent of the language version: two compilers of one
  language version may differ in the reload protocol (and must not be mixed in one process), never in
  the C ABI of exports.
The runtime ABI (the object header, `TypeInfo`, `ember_rt.h`) is stable within a major version
(`[VER-4]`).
