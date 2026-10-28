---

# Part XVII — Toolchain, Diagnostics and Conformance

## XVII.1 The `ember` command

```text
ember new <name> [--lib | --bin | --cdylib]    create a package
ember run [file.em] [args…] [--hot | --interp] build and run; a single file needs no manifest
ember build [--profile <name>] [--target <triple>] [--backend c|llvm]
            [--emit tokens|ast|hir|mir|c|obj|header] [--out-dir <dir>]
            [--emit-optimization-report] [--report=engine|instantiations] [--timings[=json]]
            [--build-id] [--reload [--explain]] [-D warnings]
ember check [--syntax-only] [--timings]        every check of a build, without code generation
ember test [filter] [--doc] [--no-leak-check]  run @test functions and documentation examples
ember bench [filter] [--compare]               benchmarks; --compare runs the paired C programs
ember fmt [--check | --migrate]
ember lint
ember doc
ember update [package]                         re-resolve dependencies and rewrite ember.lock
ember explain <E-code | rule-id>               the error page or rule text
ember explain --view <file>:<line>             why each view in use at that line is still in use
ember explain --cycle <path> <Class[.field]>   one class or field through which owners may loop
ember inspect <item> [--safety [--elided-only] | --alloc | --cost | --deterministic | --cycle
                      | --expand]
ember why --alloc|--block|--io|--lock|--sync|--unsafe|--ffi <item>   the call chain to that effect
ember calls --foreign <item>                   every foreign function the item can reach
ember bind <header> [--init [--merge] | --report [--baseline <file> | --write-baseline]
                     | --check <overlay> | --explain <fn> | --emit-embind | --outcomes
                     | --emit-cpp [--out <file>]] [--flag <flag>] [--instantiate <template-id>]
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
* `[CLI-2]` *(changed in 0.9.15)* `ember build --emit c --out-dir <dir>` writes the C sources without
  invoking a C compiler, for build systems that compile them themselves, and beside them
  `<dir>/cflags.txt`, the C compiler flags those sources need for Ember's guarantees on the target,
  the stack-probe flag of `[RT-12]` among them; a build that compiles them without those flags does
  not have Ember's guarantees. `--emit tokens|ast|hir|mir|obj` write the other stages, for tools and
  for reporting compiler defects.
* `[CLI-9]` `ember check --syntax-only` lexes and parses only, reporting `E00xx` and `E01xx`.
* `[CLI-12]` `ember why --alloc <item>` (and `--block`, `--io`, `--lock`, `--sync`, `--unsafe`, `--ffi`)
  prints the shortest call chain from the item to an operation with that effect, whether or not the
  item carries a contract; with `--unsafe` it prints the obligation of every unsafe function on the
  chain.
* `[CLI-13]` `ember calls --foreign <item>` lists every foreign function the item can reach, with the
  contract of each.
* `[CLI-15]` `ember --help` lists every command and flag of this section; one missing from it, or from
  §XVII.1's listing, is a defect.
* `[CLI-20]` *(new in 0.9.9)* *(changed in 0.9.15)* `ember fmt --migrate` rewrites source written for
  an earlier version into the current one where the change is mechanical, and lists, with the file
  and line, what it could not rewrite. From 0.9.8: `::` to `.`, integer `/` to `//`, `with_views`
  removal, `for k, v in m:` over a `Map` to `for k, v in m.items():` (`[CTL-1]`). From 0.9.10:
  `Box[T]` and `ForeignBox[T]` to `owns T`, `Box[T, A]` to `owns T using A`, `Array[T, A]` to
  `Array[T] using A`, `Weak[T]` to `weakref T`, the mode `owned` to `owns`, `mut x: T` on a
  parameter to `ref mut x: T` with `ref mut` written before each argument given to it, `self` to
  `this` with the object removed from a method's parameters (`fn reset(mut self):` to `fn reset():`,
  `owned self` to `owns this` after the parameters), a function in a type body with no object to
  `static fn`, `str` to `StrView`, `cstr` to `CStrView`, `x.clone()` to `copy of x`, `.copied()` to
  `.copy_each()`, `if Some(x) = e:` to `if e exists as x:`, `w.upgrade()` opened in an `if` to
  `if w exists as p:`, `cast x if D` and `as?` to `if x is D as d:`, `as!` to `cast x as D`,
  `@borrows(p)` to `from p`, `@static_safe` to `@no_alias_checks`, `@fp(contract)` to
  `@fp(allow_fma)`, `assert_disjoint` to `check_disjoint` and `assert_disjoint_or_panic` to
  `assert_disjoint`, a function's type parameters in brackets after its name to a line of their own
  above it (`fn f[T: Add](x: T)` to `[T: Add]` above `fn f(x: T)`, `[GRM-40]`), an `x: ref T`
  parameter to `x: T`, a `T: Sync` bound to `T: ThreadShared`, and `for x in xs.iter_mut():` to
  `for ref mut x in xs:`. It does not rewrite what needs a choice the source does not say: `Shared[T]`
  (`shares T` or `shares mut T`), `RefCell`, `SyncShared` and `@sync` classes (`thread_shared` with
  `guarded by` fields or `Atomic`), `owned fn` (the capture words), and `mem.forget`.
* `[CLI-21]` *(new in 0.9.9)* `ember run --interp` runs a program on the compile-time evaluator instead
  of compiling it: no C compiler is needed; console and file I/O work; foreign calls and threads are
  unavailable (`E6010`); results equal a compiled run's.
* `[TOOL-2]` *(changed in 0.9.10)* Ember's toolchain archive (`[TOOL-1]`) carries LLVM's Clang and its
  linker LLD, pinned and selected with `[build] c_compiler = "bundled"`, which compile the generated C
  and the C and C++ glue: a working Ember installation never requires a separately installed C
  toolchain, and there is no separate Clang download. On Windows the system libraries, which their
  licence keeps out of the archive, are the one part it does not carry. `ember toolchain list` and
  `default` show and choose among installed ones; a system compiler may be chosen explicitly, and its
  configuration is recorded with each build.
* `[TOOL-3]` When no C compiler is found, `E9002`'s message says so and its help names the remedy for
  the host (on Windows, the Visual Studio Build Tools C++ workload, or `ember toolchain install cc`);
  a bare missing path is never the primary message.
* `[CLI-11]` `ember audit` prints a one-page summary of a package's safety surface: `unsafe` blocks
  and their notes, the effect totals, the foreign boundary and its contracts, and the trusted-base
  report of Annex C.
* `[TOOL-1]` *(changed in 0.9.10)* Each release publishes, on the GitHub Releases page of the Ember
  repository, a self-contained toolchain archive per supported host (Windows x64, Linux x64, macOS
  arm64) with `ember`, the runtime, `std` and editor support, and a one-line installer that puts
  `ember` on `PATH`. The release workflow (`.github/workflows/release.yml`) builds them, the macOS
  archive built and tested on GitHub's macOS machines.
* `[TOOL-4]` `ember --version` prints the compiler version, the language version, the backend, and the
  resolved C compiler and linker, so a bug report carries its environment. **The first hour:** on a
  clean Windows or Linux machine, installing Ember, `ember new hello`, `cd hello` and `ember run` print
  `hello, world` in under five minutes and at most six typed commands; a regression blocks a release.
* `[CLI-19]` *(new in 0.9.9)* **The implementation matrix.** `ember --version --matrix` prints, for
  every Part, rule family, command and flag of this document, whether this compiler implements it.
  A command, flag, attribute or construct the compiler does not implement is rejected with `E0900`
  naming it (`[PHIL-12]`), never ignored.
* `[CLI-5]` *(changed in 0.9.10)* `ember bind --init <header>` writes a starter overlay, `<stem>.overlay.em` beside the header
  (ODR-148): every derived contract, every unknown
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
language = "0.9.15"
kind = "bin"                      # bin | lib | staticlib | cdylib
layers = ["core", "alloc", "sync", "io"]
edition_lints = "strict"          # warnings listed in [lints] as "error" fail the build

[dependencies]
geometry = { path = "../geometry" }
parser = { git = "https://example.org/parser.git", rev = "4f2a9c1" }
std = "0.9.15"                    # implicit; may be pinned

[build]
entry = "src/main.em"
target = "native"
backend = "c"                     # c | llvm (`[CG-LL-1]`)
c_compiler = "auto"               # auto | bundled | msvc | clang | gcc
runtime = "static"                # static | shared (shared is required by hot reload)
max_instantiations = 200

[profiles.development]
lto = "thin"

[profiles.profiling]
inherits = "development"
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
large_copy = { level = "warn", threshold = 256 }
```

* `[MAN-1]` *(changed in 0.9.9)* An invalid manifest, including one with an unknown key, is `E9001`,
  naming the key and the keys that section accepts.
* `[MAN-2]` `ember.lock` records the resolved dependencies with content hashes; `--locked` fails if it
  would change.
* `[MAN-3]` *(changed in 0.9.9)* Every key in `[lints]` names a lint the compiler defines (`E9010`
  otherwise); its value is a level (`allow`, `warn`, `error`) or a table with `level` and the lint's
  parameters.
* `[MAN-8]` *(new in 0.9.9)* *(changed in 0.9.10)* The sections are: `[package]` (`name`, `version`, `language`, `kind`,
  `layers`, `edition_lints`); `[dependencies]` (`{ path }`, `{ git, rev }`, or `{ git, version }`, `[VER-5]`);
  `[build]` (`entry`, `target`, `backend`, `c_compiler`, `runtime`, `max_instantiations` `[MONO-3]`,
  `reload` `[MAN-7]`); `[profiles.<name>]` (`[PRF-3]`); `[c]` (`compiler`, `flags`, `[BLD-FFI-1]`);
  `[link]` (`libs`); `[jobs]` (`workers`, `[JOB-1]`); `[realtime]` (`contracts`, `[EFF-19]`);
  `[comptime]` (`max_steps`, `max_heap_mb`, `[CT-3]`); `[lints]` (`[MAN-3]`); and `[cpp.<project>]`
  and `[ffi]` (`evidence`, Annex C). A key a section does not take, or a section not listed, is
  `E9001`, naming what is accepted (ODR-110). **No manifest key turns off a
  safety check or changes what a program means** (`[PRF-1]`).
* `[MAN-7]` *(changed in 0.9.10)* `[build] reload` is `"opt-in"`, `"all"`, `"bodies"` or `"none"`; a
  `release` build ignores it and prints the note `hot reload is off in release builds` (Annex B).
* `[VER-5]` *(changed in 0.9.10)* Package versions are semantic. A dependency
  `{ git = "<url>", version = "1.2" }` names the versions its repository has tagged
  `vMAJOR.MINOR.PATCH`; the requirement `"1.2"` means `>= 1.2.0, < 2.0.0`, and resolution picks the
  highest tagged version satisfying every requirement, recorded with its commit in `ember.lock`;
  `ember update [package]` recomputes it. A requirement with no source (`geometry = "1.2"`) is for a
  package registry, which this version does not have (`E0900`). Two majors of one package may coexist,
  distinguished in symbol names.

## XVII.3 Builds

* `[BLD-1]` *(changed in 0.9.10)* The module (one file) is the unit of front-end caching; the package
  is the unit of monomorphisation and code generation. A `debug` build writes one C file per module and
  recompiles only the modules that changed; a `release` build writes one C unit, so the C compiler
  optimises across modules and the shipped program loses no speed.
* `[BLD-2]` *(changed in 0.9.10)* *(changed in 0.9.15)* A module's front-end result is keyed by its
  source, the compiler and language versions, the package configuration and the **interface hashes**
  of what it imports (signatures, layouts, effect sets, inline bodies, and the facts of `[BLD-14]`),
  so editing a private body recompiles that module and re-checks its callers only if a fact they use
  changed: its effect set (`[EFF-4]`), an inline body, or a fact the compiler worked out from the body
  that a caller's check relies on (`[BLD-14]`) (ODR-058). The cache serves `ember check` as
  `ember build` does.
* `[BLD-8]` Within a module, editing one function body re-checks that function, and those of the
  module that use a fact of it that changed (`[BLD-2]`).
* `[BLD-14]` *(new in 0.9.15)* **Facts worked out at a package's edge.** A fact the compiler works out
  from a function's body never changes what a line does; it only decides whether a line compiles, or
  removes a test that would always pass. Inside a package such facts are used freely. Across a
  package's edge, where a word exists to write the fact, only the written word counts: a `pub`
  function's bounds are written (`E2040`). Every other fact is recorded in the package's interface
  file, beside its effect sets (`[EFF-4]`), and a change to it is reported as a change to the
  package's interface:
  1. what each function may change, read and end, with the chain of calls a message names (§X);
  2. for a `ref mut` list parameter, whether the function only adds at its end (`push`, `extend`) and
     reads or writes no element already in it, so that a caller's position in that list stays usable
     across the call;
  3. what the scoped tasks a function starts change;
  4. whether a function sets the parent link of an object it is given before reading it;
  5. which of a function's view parameters, `this` included, it uses after a line that may end or
     move what they view, and what it reaches by other names while a `ref mut` parameter is used, so
     that reordering two lines inside a library function (a use of `s` moved below `LOG.clear()`) is
     a change to its interface;
  6. the locks a function may take; and
  7. the function values the package stores into each function type and the callbacks it hands to C,
     which a program using the package counts wherever such a value is called and at every foreign
     call not marked `@ffi(no_callbacks)`.

  A library update that changes such a fact can make a line of a program that uses it stop compiling;
  the message names the chain of calls (`[DIA-27]`). A parent link proves itself only when no other
  package can change it or the places that own its holders. What a generic function changes through
  its type parameters' methods is worked out for each instance, whose body the instantiating package
  compiles.
* `[BLD-4]` *(changed in 0.9.10)* The C compiler and linker run through a generated Ninja file, so C
  compilation is incremental too. Ember's download carries Ninja beside Clang and LLD (`[TOOL-2]`); a
  build from a source checkout finds `ninja` on `PATH`, and without it is an error naming the command
  that installs it.
* `[BLD-5]` Output goes to `target/<profile>/{bin,lib,c,obj,bind,inspect}`.
* `[BLD-7]` Front-end stages run in parallel across modules by default (`-j`, default the number of
  physical cores).
* `[BLD-6]` *(changed in 0.9.10)* `lto = "off" | "thin" | "on"`. A value the backend's toolchain does
  not support is replaced by the nearest one it does, `on` for `thin` where the toolchain does only
  `off` and `on` (cl, gcc), and the build record, `target/<profile>/inspect/<name>.build.json`, says
  so: its `lto` gives what was asked (`asked`), what was used (`used`) and, where the two differ, why
  (`why`). Objects another link reads (a static library) take no LTO with cl or Clang, and fat LTO
  objects with gcc. No correctness property depends on LTO.
* `[BLD-9]` `--timings` writes a per-stage, per-module timing report (`--timings=json` for tools).
* `[BLD-10]` *(changed in 0.9.10)* The compiler is made as fast as it can be, and memory safe. Its
  speed is measured, not held to fixed times, since a time differs from one machine to another: the
  figures below are a release gate by comparison, never by a limit in seconds.
* `[BUD-1]` *(changed in 0.9.10)* They are measured over `bench/bigpkg`, a generated 50k-line package
  of 400 modules with 15 % generic code and one C header import of Vulkan's size (a generated header
  of 20,000 lines in Vulkan's shape: enums, structs, function-pointer types, prototypes, not counted in
  the package's lines), on one recorded machine, whose CPU, memory and storage are published with the
  figures.
* `[BUD-2]` *(changed in 0.9.10)* Each figure is the median of 15 runs after 3 warm-up runs. B6's cold
  cache is a new, empty global build cache; B7 and B9 build with the program cache off:

  | # | Operation |
  |---|---|
  | B1 | `ember check` after a one-function-body edit |
  | B2 | `ember build --reload` after a one-function-body edit |
  | B3 | `ember build` after a one-function-body edit, `debug` |
  | B4 | `ember build` after a signature change in a leaf module |
  | B5 | `ember build` after a change to a widely imported type |
  | B6 | clean `debug` build, cold cache |
  | B7 | clean `development` build |
  | B8 | `.embind` regeneration for a Vulkan-sized header |
  | B9 | compiler peak memory, clean build |

* `[BUD-3]` *(changed in 0.9.10)* A change is measured before and after on the same machine; one that
  makes a figure worse than before, beyond the figure's spread, fails CI. Every figure is published.
* `[BUD-3a]` *(changed in 0.9.10)* The before and after are measured in one run, interleaved, and the
  run is normalised by a fixed calibration workload; a run whose calibration varies by more than 10 %
  is inconclusive, not failed.
* `[BUD-3b]` *(changed in 0.9.10)* A figure's spread is its interquartile range over its median. Both
  gates of `[BUD-3]` apply only to a figure whose spread is below 15 %, and every figure is published
  with its spread.
* `[BUD-5]` A proposed language or compiler feature states its measured effect on B3 and is rejected if
  it adds more than 5 percentage points.
* `[BLD-11]` A package that omits a `[STD-6]` layer cannot use it or depend on a package that does.
* `[BLD-13]` Builds are reproducible: the same inputs give a byte-identical artefact, with no
  timestamps, absolute paths, host names or hash-order dependence. `--build-id` prints the hash of the
  inputs (`[DET-7]`).

## XVII.4 Profiles

* `[PRF-1]` *(changed in 0.9.9)* *(changed in 0.9.15)* **A profile never changes what a program
  means.** Every profile accepts the same programs, performs the same safety checks while the program
  runs (`[PRF-4]`), stops in the same places with the same messages, and computes the same results.
  The one check that differs is `debug_assert`, checked only where `[PRF-3]` says: it may not have
  side effects (`W2016`), so a program whose assertions hold computes the same results in every
  profile. Only the following differ between profiles.
* `[PRF-3]` *(new in 0.9.9)* *(changed in 0.9.10)* **What a profile may set.**

  | Setting (key) | `debug` | `development` | `release` |
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
  | Ember's own speed rewrites (`rewrites`) | off | on | on |

  The speed rewrites are what the compiler does to a program's code for speed before the C compiler
  sees it: inlining, copies of a loop, checks a proof removes, and the passes after them. Without
  them a build's code is the program as written, which a debugger follows line by line, each local
  by its name (`[TST-5]`); what the rewrites decide that a program is told (`@simd`'s verdicts,
  `[SIMD-2]`) is the same in every profile (`[PRF-1]`). A package turns them on for `debug` with
  `[profiles.debug] rewrites = true`.

  A package may define more profiles, each `inherits` one of these and overrides keys of this table.
  A key outside the table is `E9001`.
* `[PRF-2]` *(changed in 0.9.10)* Hot reload is a build mode of `debug` and `development`, not a
  profile; a `release` build is never reloadable (Annex B).
* `[PRF-4]` *(new in 0.9.15)* **The checks that run while a program runs** are the same in every
  profile, and none is compiled for `debug` only: bounds checks; the arithmetic checks (overflow,
  division by zero, shift amounts); `assert_disjoint`'s compare of two spans; the compare of a count
  of owners, and of a weak count, with its limit before each +1 (`[RC-4]`); the test of a `weakref` link's
  target when it is opened; the type compare of `cast x as T`; a `Handle`'s generation; and the
  checked trampoline's checks of a foreign callback, its thread checks included (`[FFI-33a]`,
  `[FFI-33b]`); the check against adopting one object twice (`[FFI-36b]`); and the ranges of every GPU draw
  (`[GPU-12]`). No check of exclusive access runs while a program runs: every rule about who may
  change or read a place is checked when compiling (§VII). §X.4 gives each check's cost. A check that
  stops the program says so in the program's own names (`[DIA-23]`), in every profile with the same
  text; §X.4 gives each stop's text (`[COST-6]`).

## XVII.5 Tests, gates and conformance

```ember
#$ test: compile-fail
fn main():
    a = [1, 2, 3]
    b = a
    a.push(4)          #$ error[E3040]: `a` was handed over to `b` on line 4 and is used on line 5; to keep both, write `b = copy of a` (a copy of your own) or `b = shares a` (two owners of one list)
    println(b)
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
* `[TST-11]` *(changed in 0.9.9)* *(changed in 0.9.15)* Besides the per-rule cases, the suite covers
  these scenarios: hand-over (a whole name, a part on its own, a number with `owns`, in a loop's later
  round); views and changes (a read-only name and a change of what it reads, a `ref mut` name and
  another name for its place, a position after a removal, an address after a reshape, a name opened
  from a link and a call that may end its object); walks (a list, a map, a set and a string changed
  while a loop or a generator walks it, through its own name, a call, a function value, a scoped
  task, a foreign call that may run a kept callback, and the `drop` of an object ended in the walk;
  the loop's own handle, `for ref mut x in xs:`); the call check (a callee that ends what a parameter
  views before its last use of it, `this` and `ref mut` parameters included); arguments that may
  overlap (two elements whose positions are proved different and not proved, the same whole variable
  under two names, two spans proved apart with `assert_disjoint`); function values (the real
  variable, each capture word, a function that outlives its maker, a stored function that changes its
  own state, `thread_shared fn()`); kept views (`from`, `sharedref mut`, a view leaving a lock's
  `with` block); memory managers (`using`, a value outliving its instance); `Span` and `MutSpan`
  (iteration, `ref mut`, `chunks(0)` panics, `split_at`); `shares` and `weakref` (opening after the
  target ended, opening during `drop`, both counts at their limits, atomic counts for
  `thread_shared`); loops of owners (refused, opened again in code the compiler can follow, through a
  function value's state, a `thread_shared` owner and a channel end); threads (a one-thread owner
  reaching another thread, a wait for a lock this thread holds, reductions).
* `[TST-6]` *(changed in 0.9.9)* *(changed in 0.9.15)* Annex A's code is generated from a fixture that is compiled in CI.
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
* `[TST-9]` *(changed in 0.9.10)* Each is marked `accepted` or `rejected(<shape>)`, the shape named by
  its catalogue id (`O2`, `N3`, `P21`); a rejection whose shape is in no catalogue (§XVII.6) blocks the
  release, and the help of the shape, applied literally, must make the program compile.
* `[TST-10]` *(changed in 0.9.10)* The acceptance rate is published with each release; a release that
  lowers it says why. It counts the programs that compile as written or with the catalogue's helps
  applied as `[TST-9]` says; a help the catalogue gives that the compiler does not print is a defect.
* `[TST-27]` *(new in 0.9.9)* **The C gate.** Every accepted program in the test suite is compiled
  through the C compiler with warnings as errors on each supported host compiler. A C compiler error
  on an accepted program is a compiler defect and fails CI (`[CG-C-2]`).
* `[TST-28]` *(new in 0.9.9)* *(changed in 0.9.10)* **The performance gate.** `tests/perf/` holds
  benchmark programs, each with an equivalent C program, a checker of identical output, and a
  threshold (for example, within 10 % of C at `-O2`). A regression past a threshold fails the gate,
  whose verdict of record is the one measured on the recorded reference machine before a change is
  published; CI on shared machines, whose processor may change from run to run, reports the gate's
  results without failing on them. This is the instrument for "as fast as C"; no performance claim
  in this document is considered met until its benchmark exists. `ember bench --compare` runs the
  pairs, and `--report` reports without failing.
* `[BEN-1]` *(changed in 0.9.10)* Each benchmark builds each side at several code layouts, the same
  for both sides, each moving the program's code to other addresses and changing nothing else, and
  keeps each side's fastest layout, chosen by a short interleaved run; it then runs at least 3
  untimed and 30 timed repetitions of the two on one pinned core and reports the median and the 95 %
  confidence interval of the median for each side and their ratio.
* `[BEN-2]` A gate fails only when the lower bound of the ratio's interval exceeds the threshold; a
  point estimate above it whose interval includes it is inconclusive and re-run.
* `[BEN-3]` Each run also measures the C program against a second copy of itself; if that ratio's
  interval excludes 1.00 ± 0.02 the run is void and the machine unfit for gating.
* `[BEN-4]` Each benchmark records retired instructions; a change beyond ± 0.5 % against the baseline
  fails regardless of timing.
* `[BEN-5]` Both sides are built with the same optimisation level, LTO setting, floating-point model
  and target-CPU flags; a mismatch voids the run.
* `[BEN-6]` *(changed in 0.9.15)* Thresholds: scalar and tight loops ≤ 1.05× C; SoA and SIMD code
  ≤ 1.10×; `@noalloc` paths assert allocation counts exactly; a direct FFI call executes the same instructions as C's call;
  code with `shares` owners ≤ 1.15× a C++ intrusive count (non-atomic, or atomic for `thread_shared`).
* `[TST-29]` *(new in 0.9.9)* **Honest baselines.** A gate with a baseline of known failures reports the
  baseline size beside its result; the baseline only shrinks, and a gate whose baseline covers more
  than a tenth of its cases reports itself as `baselined`, not green.
* `[TST-30]` *(new in 0.9.9)* The test runner reports every failure of a run, not only the first, and a
  test that fails intermittently is quarantined by name and listed in the run summary.
* `[TST-31]` *(new in 0.9.15)* **The count limits are tested with a real counter that is easy to
  fill.** A test whose header says `#$ count_bits: 8` is compiled with an 8-bit count of owners and an
  8-bit weak count in each object, stored and compared as the 32-bit counts are, so 255 owners or
  links of a one-thread count, or 127 of a `thread_shared` count (half its range, `[RC-4]`), reach the
  limit and the next stops the program ("too many owners of a `Player` (255)", "too
  many `weakref` links to a `Player` (255)"). Only the two tests of the count limits use it; one more
  test checks that a program built without it keeps the 32-bit counts (limit 4,294,967,295). Every
  other test and every program keeps the 32-bit counts; the header changes their width and nothing
  else, and the language has no test-only shortcut.
* `[TST-5]` A scripted debugger session (breakpoint by Ember line, stepping, locals by Ember name) runs
  in CI on every supported host; a host where it cannot run is reported as debug-unverified, with a
  recorded waiver.

## XVII.6 Diagnostics

* `[DIA-1]` A diagnostic has a code, one primary span, labelled secondary spans, an optional `help`
  (with a machine-applicable fix-it where possible) and notes. `--json` gives the same structure.
* `[DIA-2]` *(changed in 0.9.15)* Messages start lowercase, have no trailing period, name the thing,
  and say what is wrong; `help` says what to do; a message never says "you" and never blames the
  programmer. Every refusal names what was refused in the program's own names (`enemies[0]`, not
  `*enemies[0]`; `line.to_lower()`, not a temporary's internal name; `int`, not `i64`; `fn()`, not an
  internal callable type; at a call, the caller's variable, not the callee's parameter), says why in
  one line, and gives the exact line to write (`[DIA-25]`). Compiler-internal facts (internal names)
  may appear only as secondary evidence.
* `[DIA-25]` *(new in 0.9.15)* **The line a message gives compiles** under this document, in that
  program: its fix never runs into another error. Where it replaces several lines, the message names
  every line it replaces ("instead of `i = 0`, the `while` line and `i += 1`, write
  `for i in 0..idx.len():`"); a line it moves is named with where it goes ("move line 17,
  `b.click()`, below the loop"). A form that would compile but would not do what its words say is
  refused, with the line that does (`State.Walking(mut path):`, which says "may change" but not whose
  list, is `E0113`: "to change that list through `path`, write `State.Walking(ref mut path):`").
* `[DIA-26]` *(new in 0.9.15)* **One mistake gives one message**, at the program's line, on the name
  that caused it. A fault inside a function stored in a function value (it ends what its parameter
  views and then uses the parameter) is reported on the line that stores it, not on a call through
  the value.
* `[DIA-27]` *(new in 0.9.15)* **A message about a change a call makes names the chain of calls** that
  makes it ("`tick()` (line 10) adds to `LOG` (through `note`, line 4) while this loop walks `LOG`
  (line 9); call `tick()` after the loop: move line 10 below the loop"). No message asks for a word on
  a function's first line: what a function changes, reads and ends is worked out from its code (§X),
  across a package's edge from its interface file (`[BLD-14]`).
* `[DIA-28]` *(new in 0.9.15)* **Two errors on one line.** Where two errors fit one line, the one
  whose fix makes the program compile is shown: a call through a function value that may take a lock
  this thread holds, inside a walk under that lock, is `E7018` ("call `note()` after the
  `with log.lock:` block"), not `E3020`, whose fix (after the loop, still inside the lock) would still
  wait for ever; a scoped task started inside a walk of what it changes is `E3020` ("start it after
  the loop"), not `E7007`, whose fix (a lock or an `Atomic`) leaves the walk refused. Where both fixes
  compile, the rule that names the case decides: a position used after a removal is `E3060`, not
  `E3022`; a view used after a call whose callee ends what it points into, where `E3021` and `E3031`
  both fit, is `E3031`. A message says only what the compiler knows.
* `[DIA-3]` *(changed in 0.9.15)* Ownership and view errors (§XVII.6.1) label the later use that makes
  the refusal ("used here") and give the fix of their shape.
* `[DIA-4]` Contract errors print the whole call chain (`[EFF-6]`).
* `[DIA-5]` FFI errors name the header and the C declaration.
* `[DIA-6]` Every code has a page, `docs/errors/EXXXX.md`, with a program that triggers it, the
  rendered diagnostic, why the rule exists and the fix; `ember explain` prints it. A code without a
  page fails CI, and each page's examples are built by `ember test --doc`: the failing one must fail
  with that code and the fixed one must compile.
* `[DIA-6a]` The code registry is exhaustive in both directions: every code this document names is
  registered, and every registered code is named here (§XVII.9). Each code is defined by exactly one
  rule.
* `[DIA-7]` *(changed in 0.9.15)* Every ownership and view error is classified into a shape of §XVII.6.1 and emits that
  shape's help.
* `[DIA-7a]` Every code in `E3000`–`E3499` belongs to exactly one shape of §XVII.6.1; a code with no
  row is never emitted, and `tools/rule_index.py` fails CI on one.
* `[DIA-8]` *(changed in 0.9.15)* `ember explain --view <file>:<line>` prints, for each view in use at
  that line, where it was made, what it points into and the later use that keeps it in use:
  `s views v (made at 12:9), in use through 19, because s is used at 19:14`. It reads the facts the
  compiler's own checks use (`[BCK-1]`).
* `[DIA-10]` *(changed in 0.9.15)* The help of shapes B1, B2, B4, B5 and B9 names a concrete API or
  construct (`split_at`, `retain`, `for ref mut x in xs:`, `weakref`, a capture word such as
  `owns n`), written as the line to write, never a category such as "consider restructuring".
* `[DIA-11]` *(changed in 0.9.15)* For shape S1 the diagnostic names the `assert_disjoint` call and
  gives the lines that need no check while running (blending from a copy of your own), or removing
  `@no_alias_checks`; it never suggests restructuring code that cannot be improved.
* `[DIA-13]` Every shape has a rendered snapshot under `tests/ui/` and a `.fixed.em` companion showing
  the help applied, which must compile.
* `[DIA-15]` Suggestions are computed from data the compiler already holds (scope tables, type
  information, an index of exported names); none requires speculative type checking.
* `[DIA-18]` A foreign call rejected because its contract has unknown facts (`E5002`) names each missing
  fact and prints the overlay line that supplies it.
* `[DIA-12]` Every name and type error is classified into a shape of §XVII.6.2. An unclassified error
  is logged, and CI fails on the log.
* `[DIA-9]` *(changed in 0.9.15)* A diagnostic never suggests `unsafe` while a fix without it compiles
  in that program, and never suggests `shares` or `shares mut` alone where a fix that keeps one owner
  compiles.
* `[DIA-16]` *(changed in 0.9.15)* When a diagnostic suggests a `shares` or `shares mut` owner, a note
  names its cost while the program runs: one count per owner made or ended, and one compare with the
  count's maximum per owner made.
* `[DIA-14]` Only the first error of a cascade is reported: nothing is reported about an expression
  whose type is already an error, a name bound to a failed import, or a call to a failed declaration.
* `[DIA-20]` *(new in 0.9.9)* A run of invalid bytes or characters is one diagnostic, not one per byte.
* `[DIA-21]` *(new in 0.9.9)* *(changed in 0.9.10)* *(changed in 0.9.15)* **Python habits.** Each of
  these is a catalogued shape, named by its id in marks, in `--json` and in `tests/ui/` (`[TST-9]`, `[DIA-13]`), and is
  recognised and answered with the Ember form: as a machine-applicable fix-it where the help is a
  rewrite of what was written, and as text where it is advice (P18, P19, P22, P29, and P12's `try` and
  `except`):

  | Shape | Written | Codes | Help |
  |---|---|---|---|
  | P1 | `True`, `False`, `None` as a value where no `Option` is expected | `E1010`, `E2020`, `E2060` | `true`, `false`; `None` needs an `Option` type |
  | P2 | `def f():` | `E0100` | `fn f():` |
  | P3 | `str(x)`, `int(s)`, `float(s)`, `int(x)`, `float(x)` | `E1010` | by the argument's type: `x.to_string()` or `f"{x}"`; for text `s.parse[int]()?`, `s.parse[float]()?` in a function whose `Result` takes a `ParseError`, else with `.unwrap()`, giving the number as Python's does; for a number `x as int`, `x as float` |
  | P4 | `len(s)` for a string | `E2073` | `s.char_count()` (Python's count) or `s.len()` (bytes) |
  | P5 | `min(xs)`, `max(xs)` of one collection | `E2020` | `xs.iter().min().unwrap()`, `xs.iter().max().unwrap()` (Python's raises on an empty one) |
  | P6 | `xs.append(x)`, `s.strip()`, `s.startswith(p)`, `s.upper()`, … | `E2072`, `E2020` | the `[STD-13]` table (Appendix E) |
  | P7 | `xs[a:b]` | `E0100` | `xs[a..b]` |
  | P8 | `if xs:` | `E2035` | `if not xs.is_empty():` (`[CTL-0]`) |
  | P9 | `let x = 5`, `var x = 5` | `E0100` | `x = 5` |
  | P10 | `x == None` where the `Option` has no `Eq` | `E2040` | `x is None` |
  | P11 | `with open(p) as f:` | `E0100` | `with f = fs.File.open(p)?:` |
  | P12 | `raise e`, `try:`, `except E:`, `finally:` | `E0100` | `return Err(e)`, `?`, `match`, `defer:` (Part XIII) |
  | P13 | `lambda x: e` | `E0100` | `fn(x) => e` |
  | P14 | `self` in a method (`fn reset(self):`, `self.count = 0`) | `E1010` | inside a method its object is `this`, and a method does not list it: `fn reset():`, `this.count = 0` |
  | P15 | `a / b` on integers | `E2240` | `a // b`, or `a as float / b` (`[TYP-28]`) |
  | P16 | `if x = 5:` | `E2036` | `if x == 5:` |
  | P17 | `case p:` inside `match` | `E0100` | `p:` (or `p =>`), without `case` |
  | P18 | `global x`, `nonlocal x` | `E0100` | remove the line: a function changes a module variable (`static mut`) by its name, and a lambda uses the real `x` unless its capture list names it (`[CLO-2]`) |
  | P19 | `*args`, `**kwargs` in a signature | `E0100` | an `Array` or `Span` parameter, or keyword arguments with defaults; Ember has no variadic functions (§I.6) |
  | P20 | `"%d" % n`, `"{}".format(n)` | `E2020`, `E2072` | `f"{n}"` (`[LEX-19]`) |
  | P21 | `Err("…")` in a function whose error type is `AnyError` | `E2020` | `Err(AnyError.msg("…"))` (`[ERR-8]`) |
  | P22 | an interface name as a type (`shapes: Array[Shape]`) | `E2173`; `E2020` at a use | `dyn Shape` as a parameter's type, `owns dyn Shape` (or `shares dyn Shape`) in a field or a list, or `[T: Shape]` above the function (`[OBJ-2]`) |
  | P23 | `s * n`, `[x] * n` | `E2020` | `s.repeat(n)`, `[x for _ in 0..n]` (`[PHIL-14]`) |
  | P24 | `'abc'`, `''` | `E0008` | `"abc"`, `""` |
  | P25 | `len(it)`, `it[i]` on an iterator (`s.split(",")`) | `E2072`, `E2020` | `.collect[Array[…]]()` where the iterator is made |
  | P26 | a `StrView` kept where a `String` is wanted; `+=` on a `StrView` | `E2020` | `String(name)`; `name: String = …` |
  | P27 | a `char` against a `StrView` (`c == "S"`, `s + c`) | `E2020`, `E2040` | `'S'`, `c.to_string()`, `s.push(c)` |
  | P28 | a statement after `=>` in a `match` arm | `E0100` | every arm written `pattern:` with a block |
  | P29 | `s[i]` on a string | `E2020` | `s.chars()` collected, or `s.as_bytes()[i]`; there is no indexing by character (`[TXT-4]`) |
  | P30 | `p[1]` on a tuple | `E2020` | `p.1` |

* `[DIA-22]` *(new in 0.9.9)* *(changed in 0.9.15)* A diagnostic caused by a callee's contract, its
  signature or a fact the compiler worked out from its body (`[BLD-14]`) is reported at the call site
  in the programmer's code, with the callee's declaration as a secondary span.
* `[DIA-23]` *(new in 0.9.9)* *(changed in 0.9.15)* A run-time panic names Ember entities — the class, field, variable, file
  and line — never a C symbol or a mangled name (the stop texts of `[PRF-4]`).
* `[DIA-24]` *(new in 0.9.9)* *(changed in 0.9.10)* **Name suggestions (shape N1).** A candidate is
  suggested when its Damerau–Levenshtein distance from the unknown name is at most 1 for names of up
  to 4 characters and at most 2 otherwise, and it is of the right kind for the position (a type where
  a type is expected, a value where a value is, and a type where a value is only before `.`, as
  `Color` in `Color.Red`). At most three are suggested, closest first, then same file before other
  files, then by qualified name.

### XVII.6.1 Ownership and view shapes

| Shape | Situation | Codes | Primary help |
|---|---|---|---|
| O1 | a name used after it was handed over; a part handed over on its own, or its owner used whole, before the part has a new value | `E3040` | `copy of x` (a copy of your own) or `shares x` (two owners), or the use moved above the line that hands it over; for a part, a new value first or `names.push(copy of p.name)`; for a list, a note that `=` hands a list over where Python's aliases it (Appendix E) |
| O2 | a hand-over out of a place that does not own the value: a field of a type with `drop`, an array or span element, a class object's field, a read-only name, a function value's own capture where it may run twice | `E3010`, `E3011`, `E3013` | a view (`ref`, `ref mut`) first (`[DIA-9]`); then `mem.take(ref mut place)`, `mem.replace`, `mem.swap`; `pop`, `swap_remove`, `remove`, `drain`; then `copy of`; for a function value, a parameter that holds it, `owns f: once fn()` |
| O6 | a view of an uninitialised place | `E3050` | name the path on which it is uninitialised |
| O7 | explicit `x.drop()` | `E3070` | `mem.drop(x)` |
| O8 | scope guard handed over or leaked | `E3014`, `E3015` | bind it with `with` (`[THR-5]`) |
| O9 | `this` escapes its `drop`, or an owner of it is made there | `E3016` | `mem.take` the data out |
| B1 | one place changed through two names: two changeable arguments of one place, a `ref mut` name's place changed through another name while it is used, a function value stored where a variable it changes owns it | `E3022` | `if i != j:` above the call; `split_at`, `chunks_mut`, `for ref mut x in xs:`, `swap(i, j)`; `SoA` columns; for a `ref mut` name, the change made through it or after its last use |
| B2 | a collection changed while a loop or a generator walks it | `E3020` | the loop's own handle (`for ref mut x in xs:`, `for k, ref mut v in m:`); a list of your own, added after the loop; `retain` after the loop; the call that changes it moved below the loop |
| B3 | a read and a change of one place overlap: a change of what a read-only name reads while it is still used; a `ref mut` name's place read through another name; an argument that may reach into a changeable argument of the same call | `E3021` | the change moved after the read-only name's last use, or the name made `ref mut` (`boss = ref mut enemies[0]`) and the change made through it; a copy of your own for the other argument (`old = copy of tracks[src_name]`) |
| B4 | a change the name does not allow: through a plain parameter, a loop, pattern or opened name, or a plain name holding a `MutSpan`; `=` on a name that keeps its place; an owner that may change made from one that only reads | `E3023` | `ref mut` on the name and at the call (`fn tick(ref mut s: Clock):`, `tick(ref mut clock)`), on the pattern (`State.Walking(ref mut path):`) or on the loop (`for ref mut p in players:`); for a parameter given a new value, a local of your own first (`count = n`) |
| B5 | self-referential struct | `E3024` | store an index or `Handle`; split the struct; or a class whose view field says `from` a `let` field |
| B6 | a result that points into something its `from` does not name | `E3062` | a view of what `from` names, or `from` naming what the result points into |
| B7 | a view used after what it points into was ended, handed over, replaced or reshaped | `E3060` | the use moved above that line, or the place named again after it (`boss = ref mut enemies[0]`; `if cam.target exists as q:`); a value of your own (`String(...)`, `.copy_each()`) |
| B9 | a function value that may outlive a place it uses and does not own | `E3026` | a capture word (`return fn() owns n: n += 1`, `scope.spawn(fn() copy of t: ...)`), a `shares mut` owner the function keeps, or a scope (`[THR-5]`) |
| B10 | a call's word that does not match its parameter; `ref mut` before what this function may not change | `E3027`, `E3028` | the parameter's own words at the call (`bump(ref mut a)`, `blend(const ref mut d, s)`, `show(a)`), or `ref mut` on the caller's own parameter |
| B11 | two single-object references given to `assert_disjoint` | `E3095` | for two elements of one list, their positions proved apart when compiling: `if i != j:` above the call (`[DSJ-4]`) |
| B12 | a view, or a value holding views, kept or returned where it would outlive what it points into; a value made in a memory manager's instance outliving the instance | `E3063` | a value of your own (`return String(word)`, `return copy of g[0]`), an index, or a `shares` owner; the value made in an instance the caller keeps |
| B16 | a call that hands the callee a part of what its body may end or move before its last use of that parameter (a method's object included) | `E3031` | `copy of` the argument (`note(copy of LOG[0])`), or, in the callee, the use moved above the line that ends what it views |
| B17 | a loop of owners that nothing opens again before its objects lose their last owner from outside | `E3032` | the loop opened before they end (`b.next = None`), or the link back made `weakref` (`next: Option[weakref Node]`) |
| S1 | `assert_disjoint` inside `@no_alias_checks` | `E4030` | lines that need no check while running (`[DIA-11]`), or `@no_alias_checks` removed |
| A1 | arena allocation of a type with `drop`; parent used during a scope | `E3090`, `E3096` | move the arena outward, copy out, or end the scope first |
| U1 | operation needs `unsafe` | `E3100`, `E3105` | the safe API that does the same, else an `unsafe:` block with a `# SAFETY:` note |
| R1 | the reason is far from the error | — | `ember explain --view <file>:<line>` |

The ids O3, O4, O5, B8, B14, B15 and X1 are no longer used: their cases are O1 (`E3040`), O2
(`E3013`), `E2020`, or no longer arise.

### XVII.6.2 Name and type shapes

| Shape | Situation | Primary help |
|---|---|---|
| N1 | unknown name | up to three candidates (`[DIA-24]`) |
| N2 | name exported elsewhere | the import line, as a fix-it |
| N3 | unknown method or field (`E2072`) | nearest member; the interface that would supply it; a Python name per `[DIA-21]` |
| N4 | scalar mismatch | the exact lossless `as`, and which side set the expected type; for an `int` and a `float` operand (`i * 0.5`), `i as float * 0.5` as a machine-applicable fix-it |
| N5 | literal does not fit (`E2010`) | the suffix or annotation that fits |
| N6 | wrong number of arguments | the signature with names and the words in front of them |
| N7 | changing a place that never changes (a `const`, a `static`, a `let` field after `init`) | the declaration to change, with its location |
| N8 | argument does not fit the word in front of the parameter | the parameter's words in the callee's first line and the caller's fix (`copy of x`, `ref mut x` at the call, or ending the value's use before the call) (`[FN-2a]`) |
| N9 | missing bound (`E2040`) | the bound to add, and the one module where `extend` may be written (`[TYP-20]`) |
| N10 | not usable as `dyn` (`E2050`) | the method and the clause it violates |
| N11 | ambiguous method (`E2070`) | `I.m(recv, …)` for each candidate |
| N12 | indentation (`E0002`–`E0004`) | the expected column and the line that set it |
| N13 | a field assigned (`this.x = …`) but not declared (`E2072`) | `field 'x' is not declared in class C`, with a machine-applicable fix-it adding `x: <inferred type>` to the class body |
| N14 | an item has the path of a module or the name of a built-in type (`E1032`) | rename the item, or the module's file |
| N15 | a word a place requires is missing, or a word that cannot apply: a value made for an `owns` place without `owns`, `owns` before an existing name or `fn`, `owns` on a class type, `using` on a type with no heap memory of its own, `copy of` what cannot be copied as itself, a `weakref` of a part (`E2123`, `E0109`, `E2114`, `E2116`, `E2119`) | the line with the word written or removed (`mesh=owns Mesh(points=[0.0, 1.0])`, `e.mesh = m`, `a = weakref this.keep`) |
| N16 | a word or form that is not Ember: `self`, `Box`, `Shared`, `Weak`, `RefCell`, the mode `owned`, `upgrade`, `mem.forget`, `@sync`, `@borrows`, a clause after a function's brackets (`E1010`, `E0104`, `E0100`) | the line with the current word (`[CLI-20]` lists the mechanical rewrites) |
| N17 | opening an `Option` or a link, and kind tests: `Some(...)` in an `if` or `while` header, an opening or a kind test that always or never passes, a name already in use (`E0115`, `E2036`, `E2037`, `E2106`) | `if e exists as x:`, `if s is Circle as c:`, a new name, or the test removed |
| N18 | a first line, a declaration or a list of names out of form: `mut` alone on a name, `ref` in a parameter's type, parts out of order, a name with no word after a name with a word, `ref mut` in a capture list, `sharedref` without `mut`, `owns this` on `init`, `static fn` outside a type, `virtual` in a class that cannot be subclassed, a view field without a usable `from`, `@cached` where its result cannot be kept (`E0113`, `E0117`, `E0118`, `E0116`, `E0120`, `E0114`, `E2108`, `E2115`, `E2112`, `E2118`, `E2117`) | the line as it is written (`ref mut n: int`, `fn finish() owns this -> u64:`, `fn() copy of name, copy of score:`) |

### XVII.6.3 Code ranges

| Range | Area |
|---|---|
| `E0000`–`E0099` | lexing, indentation, directives, `E0900`/`E0901` (`[PHIL-12]`) |
| `E0100`–`E0499` | parsing |
| `E1000`–`E1499` | names, modules, visibility |
| `E2000`–`E2499` | types, inference, interfaces, patterns |
| `E3000`–`E3499` | ownership, hand-over, views and changes, function values, drops, loops of owners |
| `E4000`–`E4499` | effects and contracts, SIMD |
| `E5000`–`E5499` | FFI and interop |
| `E6000`–`E6499` | compile-time evaluation |
| `E7000`–`E7499` | concurrency |
| `E8000`–`E8499` | layout and GPU layout |
| `E9000`–`E9499` | build, manifest, toolchain |
| `L…`, `W…` | lints and warnings |

## XVII.7 Formatter, linter and documentation

* `[FMT-1]` *(changed in 0.9.9)* *(changed in 0.9.15)* `ember fmt` writes LF line endings, four-space
  indentation and at most 100 columns, whatever the platform. It changes layout only: never a bound,
  a word in front of a name (`owns`, `ref mut`, `copy of`, `shares`), a capture word, or anything else
  that changes what a program means; rewriting source is `ember fmt --migrate`'s,
  which lists what it changed (`[CLI-20]`) (ODR-059).
* `[FMT-2]` The formatter uses the `=>` form of a lambda whose body is one expression.
* `[FMT-3]` The formatter never emits `;` outside `[T; N]` and `[v; N]`.
* `[LNT-1]` `L1001 unused binding`: a local never read on any path (names starting `_` are exempt).
* `[LNT-2]` `L1002 assignment declares a new binding`: an unread new name within distance 2 of a
  mutable binding in scope, which is usually a typo; its fix-it rewrites the name, and it is an error
  under `edition_lints = "strict"`.
* `[LNT-3]` `L1001` and `L1002` are reported by `ember build` and `ember check`, not only `ember lint`.
* `[LNT-4]` `L2004`: a `gen fn` with no `yield`.
* `[LNT-5]` `L2005`: a `@noreload` function calling a reloadable one in a loop.
* `[LNT-6]` *(new in 0.9.9)* *(changed in 0.9.15)* `ember lint` also reports, at `warn` unless
  `[lints]` says otherwise: `L2001` unnecessary `copy of` (the original is not used after it);
  `L2002` large `Copy` value passed by value (`large_copy = { threshold = 256 }`); `L3002` a view kept
  in use longer than necessary; `L4001` allocation in a hot loop; `L4002` dynamic dispatch on a final
  type; `L4003` an `Array[int]` or `Array[float]` of more than 64 KiB indexed in a hot loop whose
  values provably fit 32 bits; `L5001` `unsafe extern` declaration with no contract; `L5002` copying
  conversion at the FFI boundary; and `L7001` lock held across a call that may block. A loop of owners
  is not a lint: it is refused (`E3032`, `[WK-6]`).
* `[DOC-1]` Error pages ship with their errors.
* `[DOC-2]` The user guide (`docs/book/`), with a "coming from Python" chapter built on Appendix E, is
  published with each release, and every sample in it is run by `ember test --doc`.
* `[DOC-3]` *(changed in 0.9.15)* `ember doc` presents, for every function, the words of its first
  line (what each parameter is handed: a view, `ref mut`, `owns`), what a view result points into
  (`from`), and what the compiler worked out from its code: what it may change, read and end, with the
  chain of calls, and its effects and allocation (`[BLD-14]`, §X). The first line does not say what a
  function changes; the documentation does.
* `[DOC-4]` The guide, the error pages and this specification are published together for each release.

## XVII.7a Editor support

* `[IDE-3]` Every compiler stage after parsing produces a complete result for a file with errors: an
  unresolvable name and an untypable expression become error placeholders that satisfy everything, so
  hover, completion and symbols keep working in a broken file.
* `[IDE-4]` *(changed in 0.9.15)* Every editor request is answered from parsing, name resolution and type checking alone,
  never from code generation or a C compiler; what the compiler works out from bodies (what a function
  changes, reads and ends, its effects, which views are in use) is shown from the last completed
  `ember check` and never blocks a response.
* `[IDE-6]` *(changed in 0.9.10)* *(changed in 0.9.15)* The compiler keeps all its state in a session object that can be
  dropped and rebuilt, so a long-running editor process does not grow without bound; the identifier
  interner is the session's too (an identifier's text is a view into it).
* `ember lsp`, a language server over stdio, is reserved for a named milestone before 1.0.

## XVII.8 Conformance profiles and ABI versions

* `[CONF-1]` *(changed in 0.9.9)* *(changed in 0.9.10)* A compiler declares the profile it implements,
  as `ember --version`'s `profile:` line (`none`, `core`, `systems`, `native` or `dynamic`), and claims
  it only when every conformance test of every rule in it passes; a partial implementation claims the
  profile below, never the one above with exceptions. Each profile holds the one before it.
* `[CONF-2]` *(changed in 0.9.10)* **Ember Core**: Parts II–VII, X, XIII and XV's core and alloc
  layers, which are XV.1 and XV.3–XV.6.
* `[CONF-3]` *(changed in 0.9.10)* **Ember Systems**: adds Parts VIII, IX, XI, XII and XIV, and XV.2
  and XV.7 (console, files, time and processes: the io layer).
* `[CONF-4]` **Ember Native**: adds Part XVI. Annex C (C++) is a separate, optional claim.
* `[CONF-5]` *(changed in 0.9.10)* **Ember Dynamic**: adds Annex B (hot reload) to Ember Native.
* `[ABI-1]` *(changed in 0.9.10)* The runtime ABI, the hot-reload protocol, the module protocol and the
  export-table protocol are versioned independently of the language; each version is a symbol whose
  name encodes the number (`ember_rt_init_vN`, `ember_reload_protocol_vN`, a module's marker
  `ember_module_protocol_vN` `[FFI-28]`, a table's `Name_vN` `[FFI-26]`), so a mismatch fails at link
  time or, for a module loaded at run time, when the host looks the name up; `ember_module_init`
  checks the runtime ABI first. A setting appended to the end of `ember_rt_config` is no new module
  protocol: the settings' `size` lets each side read only what both know.
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
