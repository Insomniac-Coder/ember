# Helper AE — the incremental build, the editor's last parts, the release tooling (2026-10-23)

Rules: `[BLD-1]`, `[BLD-2]`, `[BLD-4]`, `[BLD-7]`, `[BLD-8]` (ODR-244, ODR-257, ODR-258); `[BUD-*]`
(ODR-242) and `[HR-1]` (ODR-261); `[IDE-6]` (ODR-256), `[IDE-3]`; `[HR-9]`, `[BEN-6]`; `[TOOL-1]`,
`[TOOL-2]`, `[TOOL-4]` (ODR-222, ODR-223, ODR-258), `[DOC-4]`; D-882. Branch
`worktree-agent-a3e943ca5ca791ff8`, from main `3922e110`.

## Measuring on a shared machine

The lead's other helpers build and test on this laptop at the same time, so wall times move by 2x
from run to run. Every figure below is the median of several runs of the compiler's own CPU time
(user + kernel, `GetProcessTimes`), which another process's load moves far less, with the wall time
beside it; "before" is the release compiler of `3922e110` (`build/ember-base.exe`), "after" this
branch's, each run on the same package after the same edits.

## Done

### The compiler's fixed costs on `ember check` (ODR-242)

`bench/bigpkg` (400 modules, 48,586 lines), `ember check` after a one-function-body edit, release
compiler: 4.46 s wall / 4.34 s CPU before, 3.11 s / 2.98 s after these four changes:

- The desugarings (`@derive`, `@reflect`, serialisation, `gen fn`, `where`, blanket extensions,
  `extern` blocks, status errors) each cloned every module's tree when any module needed it
  (~200 ms on bigpkg, most of it the 20,000-line C header's module). `LoadedModule.module` is an
  `Arc<ast::Module>` now, and each desugaring copies only the modules it rewrites
  (`Arc::make_mut`): ~20 ms.
- `ember check` ran the passes that place, hoist and remove the dynamic exclusivity checks
  (`[HEAP-5]`, `[EXC-3]`..`[EXC-19]`, `[DSJ-5]`): they change only the C, and the one thing a check
  reports of them is `@static_safe`'s verdict (`[EFF-13]`). They run in a check only when some
  function is `@static_safe` (~700 ms). No contract forbids an aliasing check (`[EFF-16]`), so the
  effect verdicts do not depend on them.
- `ember check` built the `[EFF-10]` safety side table it never writes (~80 ms).
- `[VERIFY-3]`'s re-derivation of every callable-region summary, a check of the compiler itself,
  ran in release compilers too (~680 ms, as long as deriving them): it runs in the compiler's own
  debug builds (the `ci` profile every test uses), as the MIR verifier does (`[IMP-11]`).

### The incremental front end: `ember check` (`[BLD-1]`, `[BLD-2]`, `[BLD-8]`, ODR-257)

`compiler/ember_driver/src/incremental.rs` (its module comment is the design). After a check that
reported no error, each module's checked form is kept in `target/<profile>/incremental/<package>/`,
one record per module, in **parts**: each function with a body of its own (with its closures),
each generic instance its bodies reach, and the rest of the module's check. A part holds its MIR as
the checks leave it (drops elaborated, region summaries installed), each body's region contract,
store summary and invalid-result flag, what its body asked of the program (`ember_typeck::Kept`:
the instances it calls, the `dyn` values it formats), the warnings its type check and its MIR checks
reported, and **the facts of each body it calls as they were** (`[BLD-2]`'s facts: the region
contract — a view result's sources, a method's field set, what it stores — and whether its result
can be followed).

A later build keeps a part while:

- the key holds: compiler identity, command, profile, manifest, and the **open-world declarations**
  (every `extend` and every class, by module): a subclass or an implementation elsewhere changes what
  a virtual call or a bound reaches;
- the module's **declarations** (its source with its ordinary functions' bodies cut out) and those
  of everything it imports, directly or not, are unchanged;
- the **text each of its spans points at is unchanged, wherever it now is**: each file a record
  points into is cut into pieces — the text between bodies, each ordinary body (its block, then the
  rest of its line and any blank and comment lines up to the next line of code) — and a record names
  the pieces it points into, a body's with its hash. Equal declarations mean the same pieces in the
  same order; a piece that moved is relocated on reading (`ember_span::codec::Relocation`), one whose
  body changed is not used. So an edit in one function moves every later one, and only the edited
  one is checked again;
- the facts it recorded for each body it calls are those bodies' facts this build: a build assumes
  so, and once the bodies checked again are, every kept part whose recorded facts changed is checked
  again by another attempt (`Attempts::forced_parts`).

A module whose source is unchanged and all of whose parts hold is kept whole; one with equal
declarations is checked in part: the type checker skips the bodies whose parts hold
(`ember_typeck::Reuse::partial`, each function's part of the check tracked by
`ember_typeck::FunctionCheck`), and the rest of the module is checked; a module whose declarations
changed is checked whole, as is every importer of a changed declaration. A kept form's types are
found again by name; a closure's environment only where the body it is written in is kept (one in
a generic's body, which every build checks, or in a body checked again, is made by that check).
Generic instances are always type-checked again (the package is the unit of monomorphisation); a
kept instance's MIR replaces the fresh one's when its part holds. Kept warnings are reported again
where a clean build reports them: the type check's after the type check, the MIR checks' after the
MIR checks (a type error stops a clean check before the latter). A check that asks the
compile-time evaluator for a value is made whole.

`--timings` gains `modules kept`, `modules checked` (whole or in part) and `functions checked` (in
the modules checked in part).

Tests: `compiler/ember_driver/tests/incremental.rs` (`[BLD-1]`, `[BLD-2]`, `[BLD-8]`): a package
of four modules through ten edits, each check compared with a clean check of the same sources
(identical report and status) and the modules and functions checked again asserted — a body: that
function alone; a body above a generic an importer instantiates: that function alone (the importer's
kept instance relocated); a body changing a borrow fact: it, its caller in its module and its caller
in the importer; a leaf's signature: the leaf and its importer; a type: every importer; a subclass:
every module; a type error and a borrow error in one module (the kept warnings reported as a clean
check reports them); the error mended (nothing checked again). `ember_span`'s codec tests
(relocation, scoped extents). Two equivalence runs over every program of `tests/conformance`,
`compile-fail`, `run-pass`, `compile-pass`, `milestones` and `run-fail` (3,355 programs), each
comparing an incremental check with a clean one: everything kept (0 differ), and an edit of the
first function's body, every later function moved down a line (3,171 edited, 0 differ).

Defects found and fixed on the way (all in this branch):

- **An ICE in the interface cache (`[LT-40]`)**: adding a subclass that overrides a virtual method
  of another module's class made the next build fail with "cached callable-region metadata
  disagrees with the current verified summary" (main has it too: `build/ember-base.exe` reproduces
  it). A module's callable summaries read the program's class hierarchy, which its cache key did not
  hold; the key now holds the open-world declarations' hash.
- Two nondeterministic borrow diagnostics (the culprit of an `E3063`, the "keeps it alive" help of a
  conflict) chose among equals by hash order; they choose by source order now.
- A test's subclass was of a final class (it did not compile, which the test did not notice); the
  test now asserts each step's status.

`bench/bigpkg`, `ember check` after a one-function-body edit (`generate.edit(.., "body", n)`),
release compilers, 7 runs interleaved before/after after one unmeasured run of each: before
(`3922e110`) 4.75 s CPU / 4.89 s wall median; after 1.08 s CPU / 1.11 s wall. One function is
checked (`functions checked: em_4m2008work_200`). What remains of the 1 s: the C import's `.embind`
read (~240 ms), the type checker's declarations and finding the kept types again, and the generic
instances, which every build type-checks again (~220 ms; their MIR is kept).

### D-882: the error pages of the codes only an instrumented run reports (`[DOC-4]`, `[FFI-37]`)

`ember test --doc` runs an error page's examples with the command that reports its code; for
`E5053` and `W5054` (`foreign::reported_by_an_instrumented_run`) that is `ember test
--instrument-ffi`, on a package whose `src/main` is the example: the failing program's run must
report the code (an error's must fail), the fixed one's must pass without it. Test:
`compiler/ember_driver/tests/guide.rs`,
`the_pages_of_the_codes_an_instrumented_run_reports_run_their_examples_instrumented`.

### The incremental build: one C unit per module, through Ninja (`[BLD-4]`, ODR-244, ODR-257, ODR-258)

- **`ember build` (and `run`) keep and reuse the checked forms as `ember check` does** (ODR-257):
  the key class is shared by `check`, `build` and `run`, so a check serves the build after it.
- **`debug` and `development` write one C unit per module** (`<program>.m.<module>.c`, beside the
  program's own `<program>.c`, which holds the entry, the runtime glue and the bodies of no module);
  **`release` writes one unit**, so the C compiler optimises across modules (ODR-244). A reload
  image (Annex B) is written the same way and compiled through Ninja too: its reloadable functions'
  thunks, the manifest and the tables are the main unit's, each body (`<symbol>__rl`) and its frame
  records its module's. A library and `--emit c` keep one unit. The development profile is not
  named in ODR-244: it follows `debug`, because it is the iteration profile (optimised but debuggable, `[PRF-2]`), the
  C-level cross-module inlining one unit would buy is mostly done by Ember's own MIR passes (they
  inline across modules before C is written, in every profile), and `release` is what ships.
- Each unit is written from what it names (`ember_codegen_c::finish_declarations`): its types,
  prototypes and helpers are those the identifiers in its own text need, in name order (aliases
  first, each after the aliases it names; structs after what they hold by value), and a structural
  type's C name is its readable stem and a hash of its structure (`instance_suffix`), never a
  counter. So a unit's text depends only on the bodies in it and what they name: an edit leaves
  every other module's unit byte for byte as it was, and a unit is written only when its text
  changed (`ember_build::ninja::write_if_changed`), keeping its time stamp.
- **Ninja** (`ember_build/src/ninja.rs`): the build writes `obj/build.ninja` (a `cc` rule with
  Ninja's own dependency tracking, `deps = msvc` with the probed `/showIncludes` prefix or
  `deps = gcc` with a depfile; the link through a response file, four hundred objects being more
  than a command line holds) and runs it; the flags are the one-call build's, split between compile
  and link. The program cache serves a program built before from the same inputs, keyed on the
  commands and the units' and objects' contents, and stores the files Ninja recorded as included.
  Ninja is found beside the compiler (an installed archive) or on `PATH`; otherwise `E9002` names
  the install command for the host (`winget install Ninja-build.Ninja`, `brew install ninja`,
  `sudo apt-get install ninja-build`) (ODR-258). CI installs it (`ci.yml`, `release.yml`).
- **Every result is the clean build's**: the bodies are put in one canonical order after the
  checks (by module, place, symbol), and codegen orders what it writes per type by name, not by the
  type table's order (which a build that keeps forms does not share: threads' and SoA helpers,
  payload zeros, classes' tables, the shared types). Tests: `compiler/ember_driver/tests/incremental.rs`
  `an_edited_package_builds_as_a_clean_one_does_and_compiles_again_what_the_edit_changed` — four
  edits; after each every C unit equals a clean build's byte for byte, both programs print the
  same, and after a body edit only that module's object is compiled again. Equivalence runs over
  every program of `tests/conformance`, `run-pass`, `compile-pass` and `milestones` (3,292): an
  incremental build with nothing kept, with everything kept, and after an edit of the first
  function's body, each compared unit by unit with a clean build — see the defects below; the last
  run (`0482a88d`): 3,292 programs, 0 differ. And a clean build on one thread against one on the
  default (`e1e35590`, the units written on the `-j` threads): 3,292 programs, 0 differ. The
  conformance tree on `0482a88d`: 0 failing.
- Defects the equivalence runs found, all fixed here:
  - a kept body's `RefCell` borrow line was a number lowering computed from its span, so a body
    moved by an edit above it kept its old line (`CELL-7`, `FN-1`): it is `Const::SourceLine`, a span
    the backend reads, now (the C and the interpreter resolve it);
  - **a method's C symbol depended on the order bodies were checked** (`ERR-3`, `ERR-8`, `ERR-12`):
    whether `Load.from` is named after its interface depended on whether a body checked before it
    had asked `Load` for some `From` (which applies std's universal `From[T]` for every `T`); an
    incremental build checking only `from` named it otherwise than the kept callers did, and the C
    did not compile. The symbol now applies the universal implementations offering the name first
    (ADR-186's `apply_universal_member`): it is the program's, in any order. This also mattered to
    `[HR-7]`, which names reload slots by symbol;
  - a kept body's `dyn` table lost its `clone` slot (`TYP-22`): an interface requiring `Clone` was
    recorded only when a body checked here formed its table; the types found again record it too;
  - **a nondeterministic order of field accesses** (`EXC-18`): a virtual call's result's accesses
    were gathered from a hash map, so two clean builds wrote them in different orders; by symbol now;
  - a body moved by an edit kept its old `#line`s right but its location constants wrong (above);
    the module units' alias order, parallel-loop helpers and job entries (found by the conformance
    run before the equivalence runs).
  - found by the milestones and the C sweeps: a response file for clang or gcc on Windows quoted
    as Windows quotes (clang reads backslashes as escapes there: the link found no objects); an
    instrumented test (`ember test --instrument-ffi`) written one unit per module counted no call
    sites in the module units (one unit now: its sites are numbered across the program); a `dyn`
    table static in the main unit, its upcast in a module's unit, unused there (`-Wunused-const-
    variable`; `EMBER_MAYBE_UNUSED` now); a kept module's interface record kept the cache key of
    what it imported as it was (the records of modules kept whole are keyed again by their imports'
    hashes now, `ember_build::interface::refresh_known_records`); and **a compile-time value read
    from the environment kept from the build before** — a program whose check asks the evaluator
    for a value keeps nothing, and drops what it kept (`Plan::forget`): a value reads what no source
    says. Such a program checks whole each time (a better design keys each module's record by its
    values' inputs, as the evaluator's own cache does).
- Figures: the gate tool's run, below (B3, `ember build` after a one-function-body edit: 8.31 s
  before, 2.13 s after; B2 10.53 s, 2.95 s). Where B3's time goes now (`--timings`, one build):
  the kept forms read and the edited function checked ~0.6 s, the analyses and passes over the
  program ~0.5 s, C generation ~0.2 s (every unit written on the threads, only the edited module's
  rewritten), compiling that unit and linking ~0.6 s.
- **A reload image (`ember build --reload`, B2 and `[HR-1]`'s build)** is written one unit per
  module and compiled through Ninja too (`ember_build::ninja::Image`: the units compiled as a shared
  library's objects with the reload definitions, linked against the shared runtime keeping the
  manifest): the main unit holds every reloadable function's thunk, the manifest and the tables;
  each module's unit its bodies (`<symbol>__rl`) with their frame records. Measuring B2 found these
  costs, each fixed for every build where it applies, not only reloadable ones:
  - `[HEAP-5]`'s shared-access insertion inferred every body's regions three times over, on one
    thread (a fixpoint of two rounds over all bodies, then the insertion): ~720 ms on bigpkg. Each
    round's bodies, and the insertion's, run on the `-j` threads now (each reads only the last
    round's summaries): ~75 ms.
  - A `debug` build (no speed rewrites, ODR-259) rewrote a copy of the whole program to decide each
    `@simd` loop's verdict, then dropped it (~1.2 s on bigpkg, whose standard library has such
    loops), though only an `@simd(assert)` loop's `E4020` and `--optimization-report` read the
    verdicts: the copy is made only when one of those does.
  - The reload manifest's writer copied the image's description (every reloadable symbol, static and
    default) once per unit before seeing that only the main unit writes it (~300 ms with 400 units);
    and each type's schema found its own type again by name among all types (quadratic).
  - A program that imports C loaded libclang (~160 ms on this machine) after reading every module,
    even when the binding comes from its `.embind` (the record's key names the library). The load
    starts on a thread of its own as soon as a module that imports C is read
    (`cimport::prefetch_libclang`), beside the reading of the rest: ~90 ms less on bigpkg.
  - The module units were written one after another (~720 ms on bigpkg, ~1.3 s for an image). Each
    is a function of the preamble and its own bodies alone, so they are shared out in runs among
    the `-j` threads, each thread's emitter writing the preamble once, while the main thread writes
    the main unit: ~200 ms (~560 ms for an image, whose main unit holds every thunk and the
    manifest). Every unit is the one-thread build's, byte for byte (below).
  - A body edit changes its module's interface hash (its inline body's hash is part of it), so
    every importer's interface record is keyed again and written (200 files on bigpkg, ~140 ms):
    the records are written on the `-j` threads.
  B2 (the gate's run, below): 10.53 s before (`3922e110`: one 18 MB unit compiled and linked
  each time), 2.95 s after: the whole front end checked again (~1.4 s; a reload build keeps no
  checked forms, below), C generation ~0.6 s (all 400 units written, the unchanged ones not
  rewritten), one module's unit compiled and the image linked ~0.6 s (`--timings`, one build).
- Not done: a reload build checks every module again: `[BLD-1]`'s kept forms are not used when the
  build writes the reload's helpers into the modules before the check (each type's field defaults
  and `Old<Name>` from the last image's schema, `reload::inject`). Keeping them needs the helpers'
  text in each module's key; the saving on bigpkg would be ~0.4 s of B2 (reading 407 kept records
  costs ~0.6 s, about what checking them does).
- Not done: a `cdylib`/`staticlib` package still compiles in one call, not through Ninja
  (`ember_build::compile_and_link_*`); it is one unit, so Ninja would add a process and save nothing
  until it too is written one unit per module (its private functions have internal linkage in one
  unit, which units per module would have to give up or namespace, as its relaxed units do).

### `[BLD-7]` — the front end on `-j` threads

`-j N` / `--jobs N` (default: the physical cores, `ember_driver/src/cores.rs`: Windows'
`GetLogicalProcessorInformationEx`, Linux's `/proc/cpuinfo`, macOS's `hw.physicalcpu`) sets the
threads of the front end's work that takes one body at a time (`ember_mir::parallel`, re-exported
as `ember_analysis::parallel`): lowering, definite initialisation, drop elaboration, each round of
the callable region summaries (each round reads only the last one's), the result-region inference,
the borrow check and `[HEAP-5]`'s shared-access inference (its rounds and its insertion). Each body
is taken by one thread with a sink of its own; results and diagnostics come back in the bodies'
order, so a program's report and its C are the same on any number of threads. The C units of the
modules are written on the same threads (in runs, beside the main unit), and the interface records
too. `-j` also sets Ninja's compiles; `--timings` names the threads and now splits
the body checks into their stages (initialisation checks, drop elaboration, callable region
summaries, borrow checks, the other body checks). Reading a symbol's text takes no lock any more
(`ember_span::symbol`: write-once slots in pages; only interning a new text locks), so the threads
never wait on one another for a name; a debug compiler's fact verifier of `[IMP-7]` runs in the
compiler's own debug builds only (`[IMP-11]`), as the MIR verifier does.

Tests: `ember_mir::parallel`'s `many_threads_report_what_one_does_in_the_same_order`;
`ember_span::symbol`'s `threads_intern_and_read_alike`; `tests/incremental.rs`
`a_package_checks_and_builds_alike_on_one_thread_and_on_several` (a check with an error and
warnings in several modules reports the same on 1 and 4 threads; a build writes the same C).
Not parallel: the type checker, one `Checker` with one type table, interning as it goes: parallel
type checking needs `[IDE-6]`'s interner per session and a type table two threads can intern into
in one order. The rule says "front-end stages ... in parallel across modules"; type checking is
the stage that is not (its unfinished entry says so). Figures (bigpkg, a clean `ember check`,
`EMBER_NO_INCREMENTAL=1`, release compiler, 7 runs interleaved, the laptop's 24 cores): 2.33 s on
one thread (`-j 1`), 1.38 s on the default.

### The compiler's speed, before and after (`[BLD-10]`, `[BUD-*]`, ODR-242; `[HR-1]`, ODR-261; `[HR-9]`)

`tools/compile_budgets.py` (reworked; its test `milestones.rs`
`the_compilers_speed_is_measured_before_and_after_a_change_and_published`):

- **No time limits** (ODR-242). `--before EXE --after EXE` run each figure through both compilers
  on one machine, the rounds interleaved (each round runs both, the first of them in turn), each
  compiler with its own work folder and global cache. A figure is **slower**, and the gate fails,
  when the after compiler took longer in so many rounds that chance would do so less than once in
  a hundred (a one-sided sign test over the paired rounds; 15 rounds after 3 warm-ups by default,
  `[BUD-2]`); `faster` and `no measurable change` pass. Every figure is published: both medians,
  both spreads (`[BUD-3b]`, the interquartile range over the median), the ratio, the verdict and
  every run, as JSON (`--json`) and a Markdown table (`--markdown`); `--judge REPORT` judges a
  published report again from its runs. Without `--before` the figures are measured and published.
- The calibration workload, the reference machine's normalisation and the recorded baseline
  (`[BUD-3]`, `[BUD-3a]`) are gone: a comparison on one machine needs none.
- **B9** (peak memory) keeps `[BUD-2]`'s 3 GB, which is no time; before/after it is published, not
  gated (ODR-242 speaks of speed). Question below.
- **`[HR-1]`** — `bench/bigpkg/generate.py --live` writes the reloadable package (`reload = "all"`)
  whose `main` keeps running with `[BEN-8]`'s 10,000 live class instances, moving each every frame,
  printing the edited function's result each time it changes and reaching `std.hot.checkpoint()`
  every frame. The figure is from the save of a one-function-body edit (the middle module's `work`,
  as B2's) through `ember build --reload` (B2) and the runtime's load, plan, prepare and commit to
  the program's first result from the new body, its instances all still there — measured by the
  tool on the running program, interleaved before/after like every figure.
- **`[HR-9]`** — `ember bench --reload [filter] [--tier all|bodies] [--dir D] [--runs N] [--core N]
  [--report]` (`compiler/ember_driver/src/bench.rs`, replacing `bench/reload_overhead.py`, which
  was Windows only and ran programs outside the harness): each program (with the files named after
  it beside it, `ffi_direct_call_mix.c`) built in `development` plain and reloadable
  (`reload = "all"`, or `"bodies"` with `--tier`), checked to print the same, timed interleaved on
  one pinned core; each ratio with its 95 % interval, and the suite's overhead (geometric mean less
  one) with the configuration, against the 3 % bound. The tool's `HR-9` figure runs it with the
  after compiler. Test: `reload.rs` `the_reload_overhead_is_measured_and_reported_with_its_configuration`.
  The tool now writes its report after each figure, so a figure that cannot be measured loses
  none measured before it (the first full run lost 87 minutes of figures to `ffi_direct_call`,
  which the bench copied without its C file).
- **`[HR-9]` measured** (the laptop: Intel Core Ultra 9 275HX on mains power, MSVC 14.44,
  `development`, core 1, 3 untimed and 30 timed runs of each form interleaved; ratios reloadable
  over plain):

  | Program | `reload = "all"` | `reload = "bodies"` |
  |---|---|---|
  | a06_enum_match | 2.099x | 1.574x |
  | a10_recursion | 1.869x | 1.100x |
  | t5_words | 1.477x | 0.981x |
  | ffi_direct_call | 1.408x | 1.009x |
  | t6_char_indices | 1.129x | 1.068x |
  | t1_chars | 1.104x | 1.023x |
  | counted_handles | 1.049x | 1.017x |
  | the other 18 | 0.78x to 1.04x | 0.79x to 1.03x |
  | **suite (geometric mean less one)** | **+9.05 %, over 3 %** | **+1.78 %, within** |

  What the two tiers differ by is `[HR-3b]`'s frame records, kept by every function of an image
  whose instances migrate (`all`, not `bodies`): on entry a call into the shared runtime for the
  thread's chain (`ember_reload_frames()`), a push of a record on the stack, a store of the call
  site before each call, a pop on return (the C of `a10_recursion`'s `fib`, `em_3fib__rl`).
  What remains in `bodies` is the thunk (one load and one indirect jump) and `[HR-9a]`: a reloadable
  function is never inlined, so `a06_enum_match`'s `area`, which the plain build inlines into the
  100-million-turn loop, is a call each turn. (`soa_update` 0.78x and `int_math` 0.93x are faster
  reloadable in both tiers: code placement, not reload.)
  Found and fixed while measuring: an image was compiled without link-time optimisation, which with
  one unit per module kept the C compiler from inlining any function of another unit (the standard
  library's, `@noreload` ones) into the package's code, as the plain build's link does. An image
  now has the profile's (none in `debug`): `reload = "all"` went from +11.91 % to +9.05 %
  (`a10_recursion` 2.56x to 1.87x, `t5_words` 1.84x to 1.48x). A reloadable body stays out of its
  reach: every call to one loads its target from the image's table.
- `release.yml`'s job publishes the figures of each release against the release before it (built
  from its tag in a worktree of its own), attached to the release; it reports and does not gate, a
  runner not being the reference laptop. The gate of record is the laptop's run before a push.
- **Figures** — the run of record here (`milestones.rs` `the_compilers_speed_over_bigpkg`, by hand:
  before `build/ember-base.exe`, the release compiler of `3922e110`; after the release compiler of
  `0482a88d`; the laptop on mains power, Intel Core Ultra 9 275HX, 24 cores, MSVC 14.44; bigpkg
  400 modules, 50,000 lines; 15 rounds after 3 warm-ups, interleaved; the report also in JSON,
  `build/budgets/report.json`, every run listed):

  | Figure | Operation | Before | After | Change | Verdict |
  |---|---|---|---|---|---|
  | B1 | `ember check` after a one-function-body edit | 4306.1 ms (spread 2.5%) | 1074.4 ms (spread 4.3%) | faster, 0.250x (15 of 15 rounds) | pass |
  | B2 | `ember build --reload` after a one-function-body edit | 10528.9 ms (6.9%) | 2952.6 ms (8.2%) | faster, 0.280x (15 of 15) | pass |
  | B3 | `ember build` after a one-function-body edit, `debug` | 8309.6 ms (1.5%) | 2134.7 ms (4.0%) | faster, 0.257x (15 of 15) | pass |
  | B4 | `ember build` after a signature change in a leaf module | 8219.5 ms (11.5%) | 2032.6 ms (11.1%) | faster, 0.247x (15 of 15) | pass |
  | B5 | `ember build` after a change to a widely imported type | 8444.6 ms (9.9%) | 7705.0 ms (5.2%) | no measurable change, 0.912x (faster in 11 of 15, p 0.06) | pass |
  | B6 | clean `debug` build, cold cache | 10177.5 ms (11.3%) | 9506.0 ms (8.5%) | faster, 0.934x (15 of 15) | pass |
  | B7 | clean `development` build | 32419.1 ms (3.6%) | 33064.8 ms (15.8%) | no measurable change, 1.020x (slower in 10 of 15, p 0.15) | pass |
  | B8 | `.embind` regeneration for a Vulkan-sized header | 249.7 ms (2.9%) | 245.9 ms (3.9%) | no measurable change, 0.985x | pass |
  | B9 | compiler peak memory, clean build | 583.3 MB (0.1%) | 447.3 MB (1.2%) | less memory, 0.767x | pass (3 GB bound) |
  | HR-1 | an edit to one function body running in the live process, 10,000 instances kept | 10914.8 ms (4.0%) | 3138.7 ms (8.4%) | faster, 0.288x (15 of 15) | pass |
  | HR-9 | the performance suite's overhead with reload on (`reload = "all"`) | — | +9.0 % | measured once | fail: over its 3 % bound (below) |

  The gate fails on HR-9 alone, which no change here made over (the first measurement of it;
  below). B7, the one figure slower in more rounds than not, is within chance (p 0.15): a
  `development` build is one unit per module now, its link optimising across them (`[BLD-6]`).

### `[IDE-3]` — the checks after type checking run on a program with errors

A program with errors is checked on (`ember_driver::failed_functions`, `ember_mir::lower_with`):
every function whose check reported an error, with the closures made in it; every function an error
was reported in or on the attributes written above it (a character the lexer rejected, an
`@borrows` naming no parameter); every function holding a placeholder expression or a value whose
type did not resolve (`TypeTable::mentions_error`); and every generic instance that only such
functions asked for (`Checker::failed_bodies`: `CLO-6`'s instance of `twice` made for `main`'s wrong
argument) — each is lowered as a body never reached (its signature, one unreachable block: no
effect, no borrow), so nothing is reported of it twice and its callers' checks find nothing of it.
The move, drop, borrow and effect checks and the MIR lints run on the rest; no interface artifact,
reload image or code is made. Test: `tests/compile-fail/the_checks_after_type_checking_run_on_what_has_no_error.em`
(a type error, a move error and an unused binding in three functions, and a closure error whose
instance is not checked: four reports, none twice). Expectations that changed, each a diagnostic
that is right: `FFI-17`, `HEAP-10`, `THR-9` (an unused binding in `main`, which no error is in),
`TYP-17` (`pick`'s instance for `str` returns a view of `u` where its elided contract says `self`),
the incremental test (the kept `geom`'s `L1001` beside `main`'s type error), and the first-week
marks (below).

**The first-week corpus's marks** (`tests/firstweek/marks.json`): 18 of the 30 rejected programs
changed their first error or their codes, none its outcome, so the published rate stands (0 of 30
as written, 23 of 30 after the helps). Two reasons: the checks after type checking now report
borrow and move errors in the functions without a type error (`03`, `05`, `09`, `15`, `22`, `30`
gain `E3012`/`E3013`/`E3021`, each a real error the chain met only in a later round); and the
diagnostics are listed in the program's order (`Sink::ordered`, from the incremental check, so a
kept module's reports land where a clean check's do), so the first error is the first in the file:
`01`'s is `False` (`P1`) on line 19 where the single-quoted string (`P24`) on line 46 came first
before. The `helps` chains are as they were (their rounds are the order the helps were applied in).

### `[IDE-6]` — the interner per session: not built (ODR-256)

ODR-256: "the identifier interner is one per session (`Symbol::as_str` borrows from it), built
with the language server". The language server is reserved for a named milestone (XVII.7a), so
this waits for it, as the answer says. What it takes, for that milestone: `Symbol::as_str(self) ->
&'static str` (about 810 uses, 480 interns, every crate) becomes a borrow of the session's interner
(`as_str(self, &Interner) -> &str`, with `Display` through the interner the formatting thread
holds), a change made in one sweep when no other branch is editing those crates (the helpers
working beside this one edit most of them). Until then each identifier the process ever sees
stays (growth by distinct identifiers only: `editor_memory.rs` shows fifty edits and queries do not
grow a session). The unfinished entry stays, saying so.

### `[BEN-6]` — allocation counts, a direct FFI call, counted objects

- **`@noalloc` paths assert allocation counts exactly**: a program run with
  `EMBER_ALLOCATIONS=1` reports at its end how many allocations it made (the runtime's own count,
  `ember_rt_shutdown`); `#$ allocations: N` on a `tests/perf/` program makes `ember bench --compare`
  fail unless the count is exactly `N`. `tests/perf/noalloc_kernel.em` (an `@noalloc` kernel run
  4,000 times over one buffer): 1 allocation, the buffer's, as its C twin's one `malloc`.
- **A direct FFI call executes the instructions C's call does**: `milestones.rs`
  `a_direct_ffi_call_is_the_c_functions_own_call` — the generated call is a plain call of an extern
  declaration bound to the C function's own symbol (`/ALTERNATENAME` for MSVC, an `asm` label for gcc
  and clang), its arguments as they are, no body under that name; `tests/perf/ffi_direct_call.em`
  times it against C's call of the same function (neither side can inline it).
- **Counted-object code within 1.15x a C++ intrusive count**: `tests/perf/counted_handles.em`
  (non-atomic) and `counted_sync_handles.em` (`@sync`, atomic), each against a `.cpp` twin with an
  intrusive count; `ember_build` compiles a `.cpp` twin with the C++ driver and standard and
  otherwise the C twin's flags (`[BEN-5]`).
- Figures (`ember bench --compare --core 1 --report`, release compiler `e1e35590`, MSVC 14.44,
  Intel Core Ultra 9 275HX on mains power, each side at its fastest of 5 code layouts, 30 timed
  runs, ratio with its 95 % interval): `noalloc_kernel` 1 allocation as asserted, 1.021 [1.019,
  1.024] of C (bound 1.05); `ffi_direct_call` 0.883 [0.869, 0.904]; `counted_handles` 1.083
  [1.076, 1.094] of the C++ intrusive count (bound 1.15), `counted_sync_handles` 1.012 [1.008,
  1.017]. The whole suite: 25 passed, 0 failed.

### The release tooling (`[TOOL-1]`, `[TOOL-2]`, `[TOOL-4]`, `[DOC-4]`; ODR-222, ODR-223, ODR-258)

Built, nothing published and no tag pushed:

- `tools/release_archive.py` — one host's self-contained archive under `target/release-archive/`
  (`ember`, `std`, the runtime's sources, LLVM's Clang, clang-cl on Windows and LLD, Clang's own
  headers, Ninja; `toolchain.json` with each file's SHA-256; reproducible: sorted entries, one fixed
  time and mode each) and its `.sha256`. It assembles only: it downloads and uploads nothing.
- The compiler finds the archive's toolchain beside itself before anything on `PATH`
  (`ember_build::find_bundled`, `c_compiler = "bundled"`; the bundled Clang links with the bundled
  LLD, `-fuse-ld=lld`, and Ninja is found beside it too). `ember --version` names the linker
  clang-cl really uses, `lld-link`, where it said `link.exe`.
- Verified here: an archive assembled from this machine's LLVM 19 and Ninja 1.12 (142 MiB),
  unpacked with only `C:\Windows` and `C:\Windows\System32` on `PATH`: `ember --version` names the
  archive's `clang-cl`, and `ember new hello` then `ember build` build the program with the
  archive's clang-cl, LLD and Ninja (the program was not run: that is `tools/release_smoke.py`'s, in
  the workflow). The archive was deleted after.
- `tools/fetch_toolchain.py` + `tools/toolchain-pins.json` (LLVM 19.1.7, Ninja 1.12.1, each host's
  URL): the workflow downloads the pinned files and refuses one whose SHA-256 is not recorded; the
  hashes are empty, so **the first release run fails printing the hashes it found**, to be checked
  against LLVM's and Ninja's own release pages and recorded (on purpose: no hash is recorded that
  nobody checked).
- `tools/release_smoke.py` — on each host, the archive unpacked on a clean `PATH`: `ember
  --version` names the archive's Clang; `ember new hello`, `ember run` print `hello, world` within
  `[TOOL-4]`'s five minutes.
- `tools/install.sh` (Linux x64, macOS arm64) and `tools/install.ps1` (Windows x64): the one-line
  installers (download the host's archive from the release, check its `.sha256`, unpack into
  `~/.ember` / `%LOCALAPPDATA%\Ember`, put `bin` on `PATH`; `EMBER_VERSION` picks a release).
- `.github/workflows/release.yml` — on a tag the owner pushes (`v*`), or by hand (which publishes
  nothing): the guide, the error pages and the specification, every example run first, archived
  together and attached (`[DOC-4]`); each host's archive built and smoke-tested on its own runner
  (Windows, Linux, macOS arm64); the compiler's speed against the release before (above); and, for
  a tag only, the archives, checksums, installers and figures attached to the release. `ci.yml`
  installs Ninja where a job builds programs.

**Signing** (not built; the owner's decision, since each needs an account or a certificate):

- Windows: Authenticode on `ember.exe` (and the archive's executables), e.g. `signtool sign /fd
  sha256 /tr <RFC 3161 timestamp URL> /td sha256` with an OV/EV certificate, or Azure Trusted
  Signing (a GitHub action, no certificate file in CI). Unsigned, SmartScreen warns on the first run
  of a downloaded `ember.exe`; the PowerShell installer's download is not marked, so it does not.
- macOS: `codesign --options runtime --timestamp` with a Developer ID Application certificate on
  `ember`, `clang`, `ld64.lld` and `ninja`, then `xcrun notarytool submit` for the archive. A
  `.tar.gz` cannot carry a stapled ticket, so Gatekeeper checks online; `install.sh` downloads with
  `curl`, which sets no quarantine attribute, so the installed binaries run unsigned today, while a
  browser download of the archive would be blocked.
- Everywhere, without any account: GitHub's build provenance (`actions/attest-build-provenance`,
  `id-token: write` and `attestations: write`), verifiable with `gh attestation verify`; and the
  `.sha256` files the installers already check. Recommendation: provenance now (free, no secrets),
  Authenticode and notarization when the owner has the accounts.

`[TOOL-1]`, `[TOOL-2]` and `[TOOL-4]` stay unfinished until the workflow has run on a tag: the
entries say what is built and what waits.

## Not done, and why

- `[BLD-7]` for the type checker: it runs on one thread (one `Checker`, the process's interner).
  Everything after it that takes one body at a time is on `-j` threads, lowering included.
- `[IDE-6]`'s interner per session: ODR-256 builds it with the language server (above).
- `cdylib` and `staticlib` packages compile in one call, not through Ninja (one unit each).
- A reload build (`ember build --reload`) checks every module again: the reload's helpers are
  written into the modules before the check, and the kept forms' key does not hold them (above).
- The pinned toolchain's hashes, signing, and the first run of the release workflow (above).

## Questions

1. **B9** (`[BUD-2]`, compiler peak memory ≤ 3 GB): ODR-242 removes the time limits; I kept B9's
   3 GB as a gate (it is no time) and publish its before/after without gating it. Options: (a) as
   built; (b) gate B9 before/after like the times (a change may not use more memory); (c) drop the
   3 GB too. Recommendation: (a), and (b) if the owner wants memory held as speed is.
2. **`[HR-1]`'s "within one second"**: ODR-261 makes it a published figure gated before/after; the
   rule's text still names one second on the reference machine. The figure measured here: 3.14 s
   (10.91 s before), of which `ember build --reload` is ~2.95 s (B2).
   Should the one second stay in the text as a target (not a gate), or go with ODR-242's limits?
3. **`[BUD-3]`, `[BUD-3a]`** (the baseline gate, the calibration workload) are replaced by the
   before/after comparison and are no longer built as written; their unfinished entries say so until
   the cut rewrites them.
4. **The diagnostics' order**: they are listed in the program's order now (by file, then place),
   where they were listed as found (the lexer's and parser's first). It makes a kept module's
   reports land where a clean check's do, and puts the first error of the file first; the first-week
   marks changed with it (above). Keep, or list as found and order the replayed ones otherwise?
5. **`[HR-9]`'s 3 % with `reload = "all"`** (measured above: +9.05 %; `bodies` +1.78 %). Per
   call, from the medians: `a10_recursion` computes `fib(38)`, 126,491,971 calls of `fib`; `all`
   205.95 ms, `bodies` 121.20 ms, plain 110.20 ms: the frame records cost 0.67 ns a call, the thunk
   and the call not inlined 0.09 ns. `a06_enum_match` calls `area` 100,000,000 times: `all`
   250.08 ms, `bodies` 176.95 ms, plain 112.45 ms: 0.73 ns a call for the records, 0.65 ns for the
   thunk and `area` not inlined into the loop (`[HR-9a]`). Options: (a) the bound is measured with
   `reload = "bodies"` and `all`'s overhead is reported beside it; (b) the suite's hot functions are
   `@noreload`, as `[HR-10]` advises for hot loops, and the bound holds for `all`; (c) the bound
   kept for `all` as written, `[HR-9]` unfinished until the records cost less (not tried here: what
   a record costs a call, the entry's call into the shared runtime for the thread's chain, the push,
   the site stores and the pop, has not been taken apart). Recommendation: (a): the records are the
   rule's (`[HR-3b]`), and the bound should name the tier it is measured on.
