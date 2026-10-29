---

# Appendix I — Rule Index

Every rule of this document, by family, with the Part that defines it.

## ABI

| Rule | Part | Begins |
|---|---|---|
| `[ABI-1]` | XVII | *(changed in 0.9.10)* The runtime ABI, the hot-reload protocol, the … |
| `[ABI-2]` | XVII | A protocol that cannot be checked at link time (hot-reload images, … |
| `[ABI-3]` | XVII | `ember_module_init` checks the runtime ABI version before anything … |
| `[ABI-4]` | XVII | A release notes every protocol whose version changed, and why. |
| `[ABI-5]` | XVII | The protocol versions are independent of the language version: two … |

## ALC

| Rule | Part | Begins |
|---|---|---|
| `[ALC-1]` | IX | *(changed in 0.9.15)* The place names the kind, the making line names … |
| `[ALC-2]` | IX | *(changed in 0.9.10)* *(changed in 0.9.15)* `std.mem` declares `pub … |
| `[ALC-3]` | IX | A binary package may replace the global allocator with `[build] … |
| `[ALC-4]` | IX | Allocation failure in a standard container panics (`out of memory`). … |
| `[ALC-5]` | IX | *(new in 0.9.15)* An instance stays as its values need it. While a … |

## ARN

| Rule | Part | Begins |
|---|---|---|
| `[ARN-1]` | IX | *(changed in 0.9.15)* `Arena` is a struct, and not `Copy`. Its … |
| `[ARN-2]` | IX | *(changed in 0.9.15)* Values placed with `alloc*` are never dropped … |
| `[ARN-3]` | IX | *(changed in 0.9.15)* `alloc(owns v: T) -> ref mut T` hands `v` over … |
| `[ARN-4]` | IX | *(changed in 0.9.15)* A growing `Arena` carries `Alloc` on its growth … |
| `[ARN-5]` | IX | `ArenaArray[T]` and `ArenaMap[K, V]` are fixed-capacity containers … |
| `[ARN-5c]` | IX | `ArenaArray[T]` provides `len`, `capacity`, `is_empty`, `get(i) -> … |
| `[ARN-5d]` | IX | `ArenaMap[K, V]` (with `K: needs (Eq, Hash)`) provides `len`, … |
| `[ARN-5g]` | IX | No operation of either container touches the arena after … |
| `[ARN-6]` | IX | *(changed in 0.9.15)* `arena.scope() -> ScopedArena` is called … |
| `[ARN-7]` | IX | *(changed in 0.9.15)* No operation lowers an arena's allocation … |
| `[ARN-8]` | IX | *(changed in 0.9.15)* `MaybeUninit[T]` has the size and alignment of … |
| `[ARN-10]` | IX | A panic inside `T.default()` during `alloc_array` aborts (`[PAN-1]`); … |
| `[ARN-11]` | IX | `Zeroable` is an `unsafe` marker: all-zero bytes are a valid `T`. The … |

## ATT

| Rule | Part | Begins |
|---|---|---|
| `[ATT-1]` | V | *(changed in 0.9.15)* An attribute that is neither in the table below … |
| `[ATT-2]` | V | *(changed in 0.9.15)* A statement may carry only `@parallel`, … |
| `[ATT-3]` | III | A statement attribute attaches to the next compound statement, never … |
| `[ATT-4]` | V | One attribute per line. A module attribute uses a `#! module` … |
| `[ATT-6]` | V | Every attribute in the table has exactly the effect its rule gives. … |

## BCK

| Rule | Part | Begins |
|---|---|---|
| `[BCK-1]` | XVIII | *(changed in 0.9.15)* Views. Each expression that makes a view at a … |
| `[BCK-2]` | XVIII | *(changed in 0.9.15)* Live views, location-sensitive. A view is live … |
| `[BCK-3]` | XVIII | *(changed in 0.9.15)* Conflicts. At each point, a read or a change of … |
| `[BCK-4]` | XVIII | *(changed in 0.9.15)* Storage end. A place whose storage ends (scope … |
| `[BCK-5]` | XVIII | *(changed in 0.9.15)* A changeable argument takes effect when the … |
| `[BCK-6]` | XVIII | *(changed in 0.9.15)* Required acceptances. Because liveness is … |
| `[BCK-7]` | XVIII | *(changed in 0.9.15)* Links. A place reached through a link … |
| `[BCK-8]` | XVIII | *(new in 0.9.15)* Summaries. For each function the compiler works out … |

## BEN

| Rule | Part | Begins |
|---|---|---|
| `[BEN-1]` | XVII | *(changed in 0.9.10)* Each benchmark builds each side at several code … |
| `[BEN-2]` | XVII | A gate fails only when the lower bound of the ratio's interval … |
| `[BEN-3]` | XVII | Each run also measures the C program against a second copy of itself; … |
| `[BEN-4]` | XVII | Each benchmark records retired instructions; a change beyond ± 0.5 % … |
| `[BEN-5]` | XVII | Both sides are built with the same optimisation level, LTO setting, … |
| `[BEN-6]` | XVII | *(changed in 0.9.15)* Thresholds: scalar and tight loops ≤ 1.05× C; … |
| `[BEN-8]` | Annex B | The end-to-end reload benchmark times three edits — a function body, … |

## BLD

| Rule | Part | Begins |
|---|---|---|
| `[BLD-1]` | XVII | *(changed in 0.9.10)* The module (one file) is the unit of front-end … |
| `[BLD-2]` | XVII | *(changed in 0.9.10)* *(changed in 0.9.15)* A module's front-end … |
| `[BLD-4]` | XVII | *(changed in 0.9.10)* The C compiler and linker run through a … |
| `[BLD-5]` | XVII | Output goes to `target/<profile>/{bin,lib,c,obj,bind,inspect}`. |
| `[BLD-6]` | XVII | *(changed in 0.9.10)* `lto = "off" \| "thin" \| "on"`. A value the … |
| `[BLD-7]` | XVII | Front-end stages run in parallel across modules by default (`-j`, … |
| `[BLD-8]` | XVII | Within a module, editing one function body re-checks that function, … |
| `[BLD-9]` | XVII | `--timings` writes a per-stage, per-module timing report … |
| `[BLD-10]` | XVII | *(changed in 0.9.10)* The compiler is made as fast as it can be, and … |
| `[BLD-11]` | XVII | A package that omits a `[STD-6]` layer cannot use it or depend on a … |
| `[BLD-13]` | XVII | Builds are reproducible: the same inputs give a byte-identical … |
| `[BLD-14]` | XVII | *(new in 0.9.15)* Facts worked out at a package's edge. A fact the … |
| `[BLD-15]` | XVII | *(new in 0.9.15)* A link that hangs is an error (verdict 124). Each … |

## BLD-FFI

| Rule | Part | Begins |
|---|---|---|
| `[BLD-FFI-1]` | XVI | *(changed in 0.9.15)* The manifest's `[c]` section names the C … |
| `[BLD-FFI-1a]` | XVI | Each `import c` is parsed and its shim compiled with the package's C … |
| `[BLD-FFI-1b]` | Annex C | The MSVC runtime-library switch, `_DEBUG` and `_ITERATOR_DEBUG_LEVEL` … |
| `[BLD-FFI-2]` | XVI | The toolchain ships a CMake module: `ember_add_library(name KIND … |
| `[BLD-FFI-3]` | XVI | *(changed in 0.9.15)* Existing libraries are linked by name (`link = … |
| `[BLD-FFI-4]` | Annex C | Ember's emitted C and the project's C++ are compiled by the same … |
| `[BLD-FFI-5]` | Annex C | `ember build --emit header` with `--cpp` also writes a C++ header … |
| `[BLD-FFI-5a]` | Annex C | *(changed in 0.9.15)* With the C++ header, an exported function may … |

## BRW

| Rule | Part | Begins |
|---|---|---|
| `[BRW-1]` | VII | *(changed in 0.9.15)* One name changes, or any number read. At every … |
| `[BRW-2]` | VII | *(changed in 0.9.15)* Liveness. A view is in use from where it is … |
| `[BRW-3]` | VII | *(changed in 0.9.15)* Reading before the call. For a method call … |
| `[BRW-4]` | VII | *(changed in 0.9.15)* Disjoint fields. `ref mut a.x` and `ref mut … |
| `[BRW-5]` | VII | *(changed in 0.9.15)* Indices are not disjoint unless proved. `ref … |
| `[BRW-6]` | VII | *(changed in 0.9.15)* Lending a `ref mut` name. From a `ref mut` name … |
| `[BRW-7]` | VII | *(changed in 0.9.15)* Using a place that is uninitialised on some … |
| `[BRW-8]` | VII | *(changed in 0.9.15)* A plain or `ref mut` parameter is passed by … |
| `[BRW-9]` | VII | *(changed in 0.9.15)* A view in Safe code is never null, dangling or … |
| `[BRW-10]` | VII | *(changed in 0.9.15)* What a method does to its object is worked out … |
| `[BRW-11]` | VII | *(changed in 0.9.15)* Arguments do not overlap a changeable one. Two … |
| `[BRW-12]` | VII | *(new in 0.9.15)* A name for a part is a read-only view. A name is … |
| `[BRW-13]` | VII | *(new in 0.9.15)* `=` on a name that has a value puts the new value … |
| `[BRW-14]` | VII | *(new in 0.9.15)* Names that keep their place. A parameter, a loop … |
| `[BRW-15]` | VII | *(new in 0.9.15)* A plain name never changes anything. A plain … |
| `[BRW-16]` | VII | *(new in 0.9.15)* Positions. A name for a list element made by … |
| `[BRW-17]` | VII | *(new in 0.9.15)* Addresses. Every other view (a kept view; a view … |
| `[BRW-18]` | VII | *(new in 0.9.15)* Loops, generators included. While a `for` loop … |
| `[BRW-19]` | VII | *(new in 0.9.15)* Temporaries. A temporary lives until the last use … |
| `[BRW-20]` | VII | *(new in 0.9.15)* Guards. A view made through a lock's guard (`g[0]`, … |
| `[BRW-21]` | VII | *(new in 0.9.15)* The call check: a view argument and what the callee … |
| `[BRW-22]` | VII | *(new in 0.9.15)* Noalias. Every `ref mut` parameter, a `MutSpan` one … |

## BUD

| Rule | Part | Begins |
|---|---|---|
| `[BUD-1]` | XVII | *(changed in 0.9.10)* They are measured over `bench/bigpkg`, a … |
| `[BUD-2]` | XVII | *(changed in 0.9.10)* Each figure is the median of 15 runs after 3 … |
| `[BUD-3]` | XVII | *(changed in 0.9.10)* A change is measured before and after on the … |
| `[BUD-3a]` | XVII | *(changed in 0.9.10)* The before and after are measured in one run, … |
| `[BUD-3b]` | XVII | *(changed in 0.9.10)* A figure's spread is its interquartile range … |
| `[BUD-5]` | XVII | A proposed language or compiler feature states its measured effect on … |

## CELL

| Rule | Part | Begins |
|---|---|---|
| `[CELL-1]` | IX | *(changed in 0.9.15)* `Cell[T]` holds a `T` that is replaced whole: … |
| `[CELL-2]` | IX | *(changed in 0.9.15)* `Cell` never hands out a view of its contents, … |
| `[CELL-3]` | IX | *(changed in 0.9.15)* `Cell[T]` is not `ThreadShared`; it is `Send` … |
| `[CELL-4]` | IX | `Cell[T]` is `Copy` when `T` is. |
| `[CELL-13]` | IX | *(new in 0.9.15)* No change is checked while the program runs. Every … |

## CG-C

| Rule | Part | Begins |
|---|---|---|
| `[CG-C-1]` | XVIII | The emitted C has no undefined behaviour. Checked signed arithmetic … |
| `[CG-C-2]` | XVIII | Accepted programs compile. A program Ember accepts never produces C … |
| `[CG-C-3]` | XVIII | Cross-module inlining. Because each module is one translation unit, … |
| `[CG-C-3a]` | XVIII | *(changed in 0.9.10)* `@inline` is binding: the function is emitted … |
| `[CG-C-3b]` | XVIII | The bodies it exports are part of the interface hash (`[BLD-2]`). |
| `[CG-C-4]` | XVIII | *(changed in 0.9.15)* Aliasing facts. Every `ref mut` parameter, a … |
| `[CG-C-5]` | XVIII | Every panic function is `_Noreturn` and cold, and each check branches … |
| `[CG-C-6]` | XVIII | *(changed in 0.9.10)* A loop in vectorisable form (`[SIMD-5]`) is … |
| `[CG-C-7]` | XVIII | Locals keep their Ember names in the C (transliterated per … |
| `[CG-C-8]` | XVIII | Every emitted statement is preceded by a `#line` naming the Ember … |
| `[CG-C-9]` | XVIII | *(changed in 0.9.15)* The build emits debugger visualisers … |
| `[CG-C-10]` | XVIII | Stack traces print Ember function paths and `file:line:col`; `ember … |
| `[CG-C-11]` | XVIII | *(changed in 0.9.15)* Floating-point flags. Every translation unit … |

## CG-LL

| Rule | Part | Begins |
|---|---|---|
| `[CG-LL-1]` | XVIII | *(changed in 0.9.10)* Two backends over one intermediate … |
| `[CG-LL-2]` | XVIII | The LLVM backend maps MIR to LLVM IR with the C backend's type … |
| `[CG-LL-3]` | XVIII | *(changed in 0.9.15)* It passes the facts the compiler proves to LLVM … |

## CLI

| Rule | Part | Begins |
|---|---|---|
| `[CLI-1]` | XVII | Every command accepts `--json` and exits non-zero on error. |
| `[CLI-2]` | XVII | *(changed in 0.9.15)* `ember build --emit c --out-dir <dir>` writes … |
| `[CLI-4]` | XVII | `ember run file.em` and `ember build file.em` accept a single file … |
| `[CLI-5]` | XVII | *(changed in 0.9.10)* `ember bind --init <header>` writes a starter … |
| `[CLI-6]` | XVII | `ember bind --report` lists every skipped declaration and every … |
| `[CLI-7]` | XVII | *(changed in 0.9.15)* `ember bind <header> --check <overlay>` checks … |
| `[CLI-9]` | XVII | `ember check --syntax-only` lexes and parses only, reporting `E00xx` … |
| `[CLI-10]` | XVII | `ember build --report=engine` is a report, not a profile: it changes … |
| `[CLI-11]` | XVII | `ember audit` prints a one-page summary of a package's safety … |
| `[CLI-12]` | XVII | `ember why --alloc <item>` (and `--block`, `--io`, `--lock`, … |
| `[CLI-13]` | XVII | `ember calls --foreign <item>` lists every foreign function the item … |
| `[CLI-15]` | XVII | `ember --help` lists every command and flag of this section; one … |
| `[CLI-18]` | VIII | *(changed in 0.9.15)* For both commands `<path>` is a package … |
| `[CLI-19]` | XVII | The implementation matrix. `ember --version --matrix` prints, for … |
| `[CLI-20]` | XVII | *(changed in 0.9.15)* `ember fmt --migrate` rewrites source written … |
| `[CLI-21]` | XVII | `ember run --interp` runs a program on the compile-time evaluator … |

## CLO

| Rule | Part | Begins |
|---|---|---|
| `[CLO-1]` | VI | *(changed in 0.9.15)* A named function, a lambda and a local function … |
| `[CLO-2]` | VI | *(changed in 0.9.15)* What a function captures is written after its … |
| `[CLO-3]` | VI | *(changed in 0.9.10)* *(changed in 0.9.15)* A callable type is one … |
| `[CLO-4]` | VI | *(changed in 0.9.10)* *(changed in 0.9.15)* A function value does not … |
| `[CLO-5]` | VI | *(changed in 0.9.15)* A function value that captures an object with … |
| `[CLO-6]` | VI | *(changed in 0.9.15)* A function that gives one of its captures away … |
| `[CLO-6a]` | VI | *(changed in 0.9.15)* A `once fn` value the program owns, including … |
| `[CLO-7]` | VI | *(changed in 0.9.10)* *(changed in 0.9.15)* Standard-library … |
| `[CLO-10]` | VI | *(changed in 0.9.10)* *(changed in 0.9.15)* A function value is two … |
| `[CLO-11]` | VI | A call `recv.name(args)` where `recv`'s type has no method `name` but … |
| `[CLO-12]` | VI | *(changed in 0.9.15)* A local function (`[GRM-28]`) is a named … |
| `[CLO-14]` | VI | *(changed in 0.9.15)* A callable type may be written as a type … |
| `[CLO-16]` | VI | *(new in 0.9.15)* `thread_shared fn()`: a function any thread may … |
| `[CLO-17]` | VI | *(new in 0.9.15)* What a function value does is worked out from the … |
| `[CLO-18]` | VI | *(new in 0.9.15)* A function value that changes its own state. A call … |
| `[CLO-19]` | VI | *(new in 0.9.15)* A lambda's parameters say what they receive. A … |

## CLS

| Rule | Part | Begins |
|---|---|---|
| `[CLS-1]` | V | *(changed in 0.9.10)* *(changed in 0.9.15)* A class object lives in … |
| `[CLS-2]` | V | *(changed in 0.9.10)* *(changed in 0.9.15)* `fn init(…)` is the … |
| `[CLS-3]` | V | A class with no `init` and no base class gets a memberwise … |
| `[CLS-4]` | V | *(changed in 0.9.15)* A class is final unless declared `open` or … |
| `[CLS-5]` | V | *(changed in 0.9.15)* Interface calls on a class object are … |
| `[CLS-6]` | V | *(changed in 0.9.15)* Destruction runs the derived `drop`, then the … |
| `[CLS-7]` | V | *(changed in 0.9.15)* `this` in a class method. Inside a class method … |
| `[CLS-7a]` | V | *(changed in 0.9.15)* Inside `drop`, `this` MUST NOT be stored … |
| `[CLS-9]` | V | *(changed in 0.9.10)* *(changed in 0.9.15)* A `let` field is set only … |
| `[CLS-9a]` | V | *(changed in 0.9.15)* After `init`, nothing is changed through a … |
| `[CLS-10]` | V | A derived class with no `init` gets its base's constructor: … |
| `[CLS-11]` | V | *(changed in 0.9.15)* Construction is two-phase. In a derived `init`, … |
| `[CLS-12]` | V | *(new in 0.9.15)* `@cached`. `@cached` above a method with no … |

## CONF

| Rule | Part | Begins |
|---|---|---|
| `[CONF-1]` | XVII | *(changed in 0.9.10)* A compiler declares the profile it implements, … |
| `[CONF-2]` | XVII | *(changed in 0.9.10)* Ember Core: Parts II–VII, X, XIII and XV's core … |
| `[CONF-3]` | XVII | *(changed in 0.9.10)* Ember Systems: adds Parts VIII, IX, XI, XII and … |
| `[CONF-4]` | XVII | Ember Native: adds Part XVI. Annex C (C++) is a separate, optional … |
| `[CONF-5]` | XVII | *(changed in 0.9.10)* Ember Dynamic: adds Annex B (hot reload) to … |

## CORO

| Rule | Part | Begins |
|---|---|---|
| `[CORO-1]` | VI | A `gen fn` declares a generator. Calling it runs none of its body; it … |
| `[CORO-2]` | VI | `yield e` suspends the generator and produces `e`. `yield` outside a … |
| `[CORO-3]` | VI | *(changed in 0.9.15)* `Generator[Y, R]` in a signature names the … |
| `[CORO-4]` | VI | The compiler rewrites the body into a state machine over the frame: … |
| `[CORO-5]` | VI | *(changed in 0.9.15)* A generator allocates nothing. The frame's size … |
| `[CORO-6]` | VI | *(changed in 0.9.15)* A view into the generator's own frame (a view … |
| `[CORO-7]` | VI | Dropping a suspended generator drops exactly the locals live at its … |
| `[CORO-8]` | VI | A generator's effect set is the union over its whole body; contracts … |
| `[CORO-9]` | VI | A frame never points into itself in Safe code; `unsafe` code that … |
| `[CORO-10]` | VI | A `gen fn` may not be `extern`, `@export`ed or passed to C (`E2222`). |
| `[CORO-11]` | VI | `std.coroutine` builds gameplay sequencing on generators with no … |
| `[CORO-12]` | VI | *(changed in 0.9.15)* A `gen fn` method of a class has `this`, and … |
| `[CORO-13]` | VI | *(changed in 0.9.15)* A `yield` while a `@must_drop` value … |
| `[CORO-14]` | VI | *(new in 0.9.15)* A generator is a walk of what it views. A … |

## COST

| Rule | Part | Begins |
|---|---|---|
| `[COST-1]` | X | Zero cost, defined. An abstraction is zero-cost *for a use* whose … |
| `[COST-2]` | X | Every implicit cost is one of: guaranteed elided (emitting it is a … |
| `[COST-3]` | X | *(changed in 0.9.10)* *(changed in 0.9.15)* The costs. Everything the … |
| `[COST-4]` | X | `ember inspect --cost <item>` prints every row that applies to an … |
| `[COST-5]` | X | A rule that introduces an implicit cost MUST add a row to this table; … |
| `[COST-6]` | X | *(new in 0.9.15)* What stops the program, and what it says. Of the … |

## CT

| Rule | Part | Begins |
|---|---|---|
| `[CT-1]` | XIV | These are evaluated during compilation: `comptime(e)`; `comptime:` … |
| `[CT-2]` | XIV | *(changed in 0.9.10)* An operation that cannot run at compile time — … |
| `[CT-3]` | XIV | Each evaluation is limited to 10⁸ steps and 256 MB of evaluator heap … |
| `[CT-4]` | XIV | *(changed in 0.9.10)* Compile-time evaluation is deterministic by … |
| `[CT-5]` | XIV | *(changed in 0.9.15)* Where results live. A compile-time result is … |
| `[CT-6]` | XIV | `comptime(e)` is an expression whose value is `e` evaluated at … |
| `[CT-7]` | XIV | A `comptime:` block at item level, or as a statement, runs once … |

## CTL

| Rule | Part | Begins |
|---|---|---|
| `[CTL-0]` | VI | The condition of `if`, `elif`, `while` and a match guard MUST be a … |
| `[CTL-1]` | VI | *(changed in 0.9.15)* `for names in e:` walks: * a place `e` whose … |
| `[CTL-2]` | VI | *(changed in 0.9.15)* While a `for` loop walks a collection, nothing … |
| `[CTL-3]` | VI | *(changed in 0.9.10)* `a..b` (`Range`), `a..=b` (`RangeInclusive`) … |
| `[CTL-3b]` | VI | *(changed in 0.9.10)* *(changed in 0.9.15)* Iteration over ranges, … |
| `[CTL-4]` | VI | The `else` of `while` or `for` runs when the loop ends without … |
| `[CTL-5]` | VI | A `match` tests arms top to bottom; the first that matches runs; a … |
| `[CTL-6]` | VI | `with a = e1, b = e2:` binds `a` and `b` for the block and drops them … |
| `[CTL-7]` | VI | *(changed in 0.9.15)* `defer:` registers a block to run when the … |
| `[CTL-8]` | VI | On every exit from a block, its `defer` blocks run first and then its … |
| `[CTL-9]` | VI | `pass` does nothing; an empty block is written `pass`. |
| `[CTL-10]` | VI | Names assigned in every branch. In an `if`/`elif`/`else` that has an … |
| `[CTL-11]` | VI | *(new in 0.9.15)* Headers that name what they test. An `if` or … |

## CXX

| Rule | Part | Begins |
|---|---|---|
| `[CXX-1]` | Annex C | The conformance suite for this annex includes a corpus of real … |

## DET

| Rule | Part | Begins |
|---|---|---|
| `[DET-1]` | X | `@deterministic` on a function or module is a contract: `Nondet` MUST … |
| `[DET-2]` | X | *(changed in 0.9.15)* `Nondet` is introduced by exactly: … |
| `[DET-3]` | X | *(changed in 0.9.15)* A `@deterministic` function may call only … |
| `[DET-4]` | X | `std.math.det` provides `sin`, `cos`, `tan`, `exp`, `log`, `pow`, … |
| `[DET-5]` | X | *(changed in 0.9.15)* Inside a `@deterministic` function the backend … |
| `[DET-6]` | X | `@deterministic` constrains results, not timing. |
| `[DET-7]` | X | The cross-machine claim holds for one binary. `ember build … |
| `[DET-8]` | X | `@deterministic` on a module applies to every function in it, with no … |
| `[DET-9]` | X | `ember inspect --deterministic <item>` prints whether the item … |
| `[DET-10]` | X | A deterministic program runs with a defined floating-point … |

## DIA

| Rule | Part | Begins |
|---|---|---|
| `[DIA-1]` | XVII | A diagnostic has a code, one primary span, labelled secondary spans, … |
| `[DIA-2]` | XVII | *(changed in 0.9.15)* Messages start lowercase, have no trailing … |
| `[DIA-3]` | XVII | *(changed in 0.9.15)* Ownership and view errors (§XVII.6.1) label the … |
| `[DIA-4]` | XVII | Contract errors print the whole call chain (`[EFF-6]`). |
| `[DIA-5]` | XVII | FFI errors name the header and the C declaration. |
| `[DIA-6]` | XVII | Every code has a page, `docs/errors/EXXXX.md`, with a program that … |
| `[DIA-6a]` | XVII | The code registry is exhaustive in both directions: every code this … |
| `[DIA-7]` | XVII | *(changed in 0.9.15)* Every ownership and view error is classified … |
| `[DIA-7a]` | XVII | Every code in `E3000`–`E3499` belongs to exactly one shape of … |
| `[DIA-8]` | XVII | *(changed in 0.9.15)* `ember explain --view <file>:<line>` prints, … |
| `[DIA-9]` | XVII | *(changed in 0.9.15)* A diagnostic never suggests `unsafe` while a … |
| `[DIA-10]` | XVII | *(changed in 0.9.15)* The help of shapes B1, B2, B4, B5 and B9 names … |
| `[DIA-11]` | XVII | *(changed in 0.9.15)* For shape S1 the diagnostic names the … |
| `[DIA-12]` | XVII | Every name and type error is classified into a shape of §XVII.6.2. An … |
| `[DIA-13]` | XVII | Every shape has a rendered snapshot under `tests/ui/` and a … |
| `[DIA-14]` | XVII | Only the first error of a cascade is reported: nothing is reported … |
| `[DIA-15]` | XVII | Suggestions are computed from data the compiler already holds (scope … |
| `[DIA-16]` | XVII | *(changed in 0.9.15)* When a diagnostic suggests a `shares` or … |
| `[DIA-18]` | XVII | A foreign call rejected because its contract has unknown facts … |
| `[DIA-20]` | XVII | A run of invalid bytes or characters is one diagnostic, not one per … |
| `[DIA-21]` | XVII | *(changed in 0.9.10)* *(changed in 0.9.15)* Python habits. Each of … |
| `[DIA-22]` | XVII | *(changed in 0.9.15)* A diagnostic caused by a callee's contract, its … |
| `[DIA-23]` | XVII | *(changed in 0.9.15)* A run-time panic names Ember entities — the … |
| `[DIA-24]` | XVII | *(changed in 0.9.10)* Name suggestions (shape N1). A candidate is … |
| `[DIA-25]` | XVII | *(new in 0.9.15)* The line a message gives compiles under this … |
| `[DIA-26]` | XVII | *(new in 0.9.15)* One mistake gives one message, at the program's … |
| `[DIA-27]` | XVII | *(new in 0.9.15)* A message about a change a call makes names the … |
| `[DIA-28]` | XVII | *(new in 0.9.15)* Two errors on one line. Where two errors fit one … |

## DOC

| Rule | Part | Begins |
|---|---|---|
| `[DOC-1]` | XVII | Error pages ship with their errors. |
| `[DOC-2]` | XVII | The user guide (`docs/book/`), with a "coming from Python" chapter … |
| `[DOC-3]` | XVII | *(changed in 0.9.15)* `ember doc` presents, for every function, the … |
| `[DOC-4]` | XVII | The guide, the error pages and this specification are published … |

## DRP

| Rule | Part | Begins |
|---|---|---|
| `[DRP-1]` | VII | *(changed in 0.9.15)* `fn drop():` runs exactly once per value, at … |
| `[DRP-2]` | VII | Locals drop at the end of their block in reverse declaration order; … |
| `[DRP-3]` | VII | Temporaries drop at the end of the statement that created them … |
| `[DRP-4]` | VII | A panic inside `drop` aborts the process (`[PAN-1]`). A `drop` SHOULD … |
| `[DRP-5]` | VII | *(changed in 0.9.15)* A `drop` body may not take fields out of `this` … |
| `[DRP-6]` | VII | *(changed in 0.9.15)* Ending an `owns T` ends the `T` and frees its … |
| `[DRP-7]` | VII | *(changed in 0.9.15)* A value whose `drop` may read through a view it … |
| `[DRP-8]` | VII | *(new in 0.9.15)* Ending a value counts as a call of its `drop`. … |

## DRV

| Rule | Part | Begins |
|---|---|---|
| `[DRV-1]` | XIV | *(changed in 0.9.15)* `@derive(…)` requests generated … |
| `[DRV-2]` | XIV | User-defined derives are not in this version; `macro` is reserved for … |

## DSJ

| Rule | Part | Begins |
|---|---|---|
| `[DSJ-1]` | IX | *(changed in 0.9.15)* `check_disjoint(a, b) -> Result[void, void]` … |
| `[DSJ-2]` | IX | *(changed in 0.9.15)* The fact belongs to the two names, not to a … |
| `[DSJ-3]` | IX | *(changed in 0.9.15)* Where the fact holds, the two names are treated … |
| `[DSJ-4]` | IX | It applies to `Span`, `MutSpan`, `SoA` columns and arena views; two … |
| `[DSJ-5]` | IX | *(changed in 0.9.15)* `assert_disjoint(a, b)` makes the same … |
| `[DSJ-6]` | IX | *(changed in 0.9.15)* `check_disjoint` and `assert_disjoint` are … |
| `[DSJ-7]` | IX | *(changed in 0.9.15)* `assume_disjoint(a, b)`, an `unsafe fn`, gives … |
| `[DSJ-8]` | IX | There is no form that checks in one profile and assumes in another … |
| `[DSJ-9]` | IX | *(changed in 0.9.15)* `check_disjoint_all(v1, …, vn)` and … |
| `[DSJ-10]` | IX | *(new in 0.9.15)* Making a view and comparing two address ranges read … |

## DSP

| Rule | Part | Begins |
|---|---|---|
| `[DSP-1]` | VIII | *(changed in 0.9.15)* A call is dispatched statically when the … |
| `[DSP-2]` | VIII | A virtual call loads its slot from the object's type table; slots are … |
| `[DSP-3]` | VIII | *(changed in 0.9.15)* A call through a `dyn I` value of a class … |
| `[DSP-4]` | VIII | *(changed in 0.9.15)* Kind tests. A value of an interface type (`dyn … |
| `[DSP-5]` | VIII | *(changed in 0.9.15)* With the whole program visible, a virtual call … |
| `[DSP-6]` | VIII | *(new in 0.9.15)* Casts. `cast s as Circle` gives the `Circle` and … |

## ECS

| Rule | Part | Begins |
|---|---|---|
| `[ECS-1]` | XII | *(changed in 0.9.15)* `Entity` is a 32-bit generational handle: a … |
| `[ECS-2]` | XII | *(changed in 0.9.15)* Each component type is stored in a pool of its … |
| `[ECS-3]` | XII | *(changed in 0.9.15)* The registry. `Registry` (EnTT's … |
| `[ECS-4]` | XII | *(changed in 0.9.15)* Views. `reg.view[A, B]()` walks the entities … |
| `[ECS-5]` | XII | *(changed in 0.9.15)* Changing the registry while a view walks it. A … |
| `[ECS-6]` | XII | Iteration order is insertion order with swap-remove holes: … |
| `[ECS-7]` | XII | *(changed in 0.9.15)* Any type is a component, with no registration … |
| `[ECS-8]` | XII | *(new in 0.9.15)* Signals, sorting, context, groups, snapshots. … |
| `[ECS-9]` | XII | *(new in 0.9.15)* No scheduler. The library runs no systems: a … |

## EFF

| Rule | Part | Begins |
|---|---|---|
| `[EFF-1]` | X | *(changed in 0.9.15)* Effects, and the function's summary (what it … |
| `[EFF-2]` | X | *(changed in 0.9.15)* A call through a function value contributes the … |
| `[EFF-3]` | X | *(changed in 0.9.15)* An `extern` function carries the effects its … |
| `[EFF-4]` | X | *(changed in 0.9.15)* Effects and summaries are part of a function's … |
| `[EFF-5]` | X | Each contract in the table forbids its effect in the function's … |
| `[EFF-6]` | X | A contract is checked over the whole reachable call graph — callees, … |
| `[EFF-7]` | X | `@assume_noalloc(call)`, an expression admitted only inside `unsafe` … |
| `[EFF-8]` | X | An `override` inherits its base method's contracts. |
| `[EFF-9]` | X | *(changed in 0.9.15)* `RuntimeCheck(k)` enters a function's effect … |
| `[EFF-10]` | X | The effect set records only which kinds occur. Per-site detail — … |
| `[EFF-11]` | X | *(changed in 0.9.15)* Every emitted check carries one reason, and … |
| `[EFF-12]` | X | *(changed in 0.9.15)* `@no_alias_checks` permits … |
| `[EFF-14]` | X | *(changed in 0.9.15)* Contracts compose; the usual inner-loop set is … |
| `[EFF-15]` | X | *(changed in 0.9.15)* Effects and contract verdicts are computed … |
| `[EFF-16]` | X | *(changed in 0.9.15)* `@nopanic(explicit)` forbids only the panics … |
| `[EFF-17]` | X | `@nopanic(explicit)` is spelled with its argument; bare `@nopanic` is … |
| `[EFF-18]` | X | *(changed in 0.9.15)* Effects are independent: a blocking … |
| `[EFF-19]` | X | `@realtime` names the contract set declared by the manifest of the … |
| `[EFF-20]` | X | `@noio` forbids `Io`, including console output; `println` in a … |
| `[EFF-21]` | X | `@nolock` forbids `Lock`, including `try_lock`, which never blocks … |
| `[EFF-23]` | X | A contract attribute whose checking an implementation has not built … |
| `[EFF-24]` | X | *(new in 0.9.15)* What a function does is worked out from its code. … |
| `[EFF-25]` | X | *(new in 0.9.15)* The summary decides every rule that needs it, and … |
| `[EFF-26]` | X | *(new in 0.9.15)* A call that runs a function makes its changes; … |
| `[EFF-27]` | X | *(new in 0.9.15)* What a `@cached` result depends on. A `@cached` … |

## ENM

| Rule | Part | Begins |
|---|---|---|
| `[ENM-1]` | III | Inside a pattern whose scrutinee type is known, a variant may be … |
| `[ENM-2]` | V | A `match` on an enum MUST be exhaustive (`E2090` lists the missing … |
| `[ENM-3]` | V | A unit-only enum is `Copy`, `Eq`, `Ord` (declaration order), `Hash` … |
| `[ENM-4]` | V | A payload enum is `Copy` only by `@derive(Copy)` with every payload … |

## ERR

| Rule | Part | Begins |
|---|---|---|
| `[ERR-1]` | XIII | `Option[T]` is absence: `Some(v)` or `None`. `Result[T, E = … |
| `[ERR-2]` | XIII | *(changed in 0.9.15)* `e?` on a `Result[T, E]` in a function … |
| `[ERR-3]` | XIII | *(changed in 0.9.15)* `Error` is the interface of error types: … |
| `[ERR-4]` | XIII | *(changed in 0.9.15)* `Option` and `Result` provide … |
| `[ERR-5]` | XIII | `Result` is `@must_use`: discarding one is `W2190` (an error under … |
| `[ERR-6]` | XIII | *(changed in 0.9.10)* A foreign function's status code becomes a … |
| `[ERR-7]` | XIII | Identity conversion. The prelude provides `From[T]` for every `T` … |
| `[ERR-8]` | XIII | *(changed in 0.9.10)* *(changed in 0.9.15)* `AnyError` is the … |
| `[ERR-9]` | XIII | *(changed in 0.9.15)* The error parameter defaults to `AnyError` … |
| `[ERR-10]` | XIII | `r.context(msg)` on a `Result[T, E]` whose `E` converts into … |
| `[ERR-11]` | XIII | The cost of `AnyError`. Converting an error into `AnyError` … |
| `[ERR-12]` | XIII | An `Err` returned from `main` (`[FN-8]`) prints `error: <Display>` to … |
| `[ERR-13]` | XIII | *(changed in 0.9.15)* The standard library follows one convention, … |

## EXC

| Rule | Part | Begins |
|---|---|---|
| `[EXC-1]` | VIII | *(changed in 0.9.15)* A change of an object's field while a read-only … |
| `[EXC-2]` | VIII | *(changed in 0.9.15)* A read of an object's field through another … |
| `[EXC-3]` | VIII | *(changed in 0.9.15)* Which names may reach one object. Two paths may … |
| `[EXC-15]` | VIII | *(changed in 0.9.15)* A method and its object. Inside a method its … |
| `[EXC-16]` | VIII | *(changed in 0.9.15)* Giving a field that owns memory a new value … |
| `[EXC-17]` | VIII | *(changed in 0.9.15)* Writing a number or a `bool` (a field of a … |
| `[EXC-19]` | VIII | *(changed in 0.9.15)* Fields are separate places. A view of one field … |

## EXP

| Rule | Part | Begins |
|---|---|---|
| `[EXP-1]` | VI | Operands, arguments, and the elements of tuple, list, map and set … |
| `[EXP-2]` | VI | An assignment evaluates its right side first, into a temporary if it … |
| `[EXP-3]` | VI | `and` and `or` short-circuit. `x if c else y` evaluates `c` and then … |
| `[EXP-4]` | VI | *(changed in 0.9.15)* A temporary created while evaluating an … |
| `[EXP-5]` | VI | *(changed in 0.9.15)* A change, a `ref mut` name and a hand-over … |
| `[EXP-6]` | VI | *(changed in 0.9.15)* A whole name of a type that is not `Copy`, used … |
| `[EXP-9]` | VI | *(changed in 0.9.15)* `a is b` tests whether two class objects (or … |
| `[EXP-10]` | VI | *(new in 0.9.10)* *(changed in 0.9.15)* Optional chaining. `a?.f` and … |

## FFI

| Rule | Part | Begins |
|---|---|---|
| `[FFI-1]` | XVI | *(changed in 0.9.10)* *(changed in 0.9.15)* Every foreign declaration … |
| `[FFI-2]` | XVI | *(changed in 0.9.15)* A call to a foreign function whose contract … |
| `[FFI-2a]` | XVI | *(changed in 0.9.15)* Derived facts need no `unsafe`: `const T*` is a … |
| `[FFI-3]` | Annex C | Ember never links against C++ mangled symbols. The importer generates … |
| `[FFI-4]` | XVI | No Ember panic crosses into foreign code (`[FFI-25]`). |
| `[FFI-5]` | XVI | Layouts are verified, not assumed: the importer records `sizeof`, … |
| `[FFI-5a]` | XVI | The generated shim (`[FFI-29]`) contains a `_Static_assert` for the … |
| `[FFI-6]` | XVI | `import c "header" with (…) [as name]` parses the header with … |
| `[FFI-6a]` | XVI | *(changed in 0.9.10)* An object-like macro is imported when, after … |
| `[FFI-6b]` | XVI | *(changed in 0.9.10)* A function-like macro is exposed only when an … |
| `[FFI-7]` | XVI | A header is re-parsed only when its content, its overlay, or a … |
| `[FFI-8]` | XVI | *(changed in 0.9.10)* *(changed in 0.9.15)* Type mapping. |
| `[FFI-9]` | XVI | `extern "C"` is the platform C ABI (SysV AMD64, Windows x64, … |
| `[FFI-10]` | XVI | *(changed in 0.9.15)* An `unsafe extern "C":` block declares foreign … |
| `[FFI-11]` | XVI | *(changed in 0.9.10)* *(changed in 0.9.15)* Contract vocabulary. A … |
| `[FFI-11a]` | XVI | A pointer contract with no count word is `E5012`, which lists the … |
| `[FFI-11b]` | XVI | *(changed in 0.9.10)* *(changed in 0.9.15)* The two-call enumeration … |
| `[FFI-11c]` | XVI | `TODO(count)`, `TODO(nullable)`, `TODO(ownership)` and … |
| `[FFI-11f]` | Annex C | A result lifetime derived from `[[clang::lifetimebound]]` imports as … |
| `[FFI-12]` | XVI | *(changed in 0.9.15)* An overlay entry whose name, parameter count or … |
| `[FFI-13]` | XVI | An overlay may contain `extend` blocks with ordinary Ember wrapper … |
| `[FFI-14]` | XVI | The importer's result is a `.embind` file (versioned CBOR: header … |
| `[FFI-15]` | XVI | *(changed in 0.9.15)* `CStrView` is a read-only view of a … |
| `[FFI-16]` | XVI | *(changed in 0.9.10)* An enum `E` marked `@ffi(status=E, ok=X, … |
| `[FFI-17]` | Annex C | `import cpp "Header.hpp" with (project="engine", overlay="…", … |
| `[FFI-17a]` | Annex C | *(changed in 0.9.10)* *(changed in 0.9.10)* *(changed in 0.9.15)* |
| `[FFI-17b]` | Annex C | Templates are available only as explicit instantiations listed in … |
| `[FFI-17c]` | Annex C | Destruction of an Ember class derived from a C++ class is … |
| `[FFI-17d]` | Annex C | *(changed in 0.9.15)* `@ffi(trampoline, owner="ember")`, the default, … |
| `[FFI-17e]` | Annex C | *(changed in 0.9.15)* The bridge types `CppVector[T]`, `CppString` … |
| `[FFI-17f]` | Annex C | `ember inspect` reports each bridge operation that is a thunk call … |
| `[FFI-20a]` | XVI | A declaration the importer cannot represent is recorded with the … |
| `[FFI-21]` | XVI | *(changed in 0.9.10)* *(changed in 0.9.15)* A C function-pointer … |
| `[FFI-21a]` | XVI | *(new in 0.9.10)* *(changed in 0.9.15)* A parameter C passes as a … |
| `[FFI-22]` | XVI | *(changed in 0.9.15)* Every exported function and trampoline first … |
| `[FFI-23]` | XVI | *(changed in 0.9.10)* *(changed in 0.9.15)* … |
| `[FFI-24]` | Annex C | Every imported C++ function has an exception policy, with a default. … |
| `[FFI-24b]` | Annex C | An overlay may assert `@ffi(noexcept)` for a function the header … |
| `[FFI-24c]` | Annex C | *(changed in 0.9.10)* `ember inspect --cost`, which reads the built … |
| `[FFI-24d]` | Annex C | A header that adds or removes `noexcept` changes the Ember signature; … |
| `[FFI-25]` | XVI | *(changed in 0.9.15)* A panic in an exported function, or anywhere … |
| `[FFI-26]` | XVI | *(changed in 0.9.10)* *(changed in 0.9.15)* … |
| `[FFI-27]` | XVI | The runtime `ember_rt` is a C11 static library with no global … |
| `[FFI-28]` | XVI | *(changed in 0.9.10)* A package built as `kind = "staticlib"` … |
| `[FFI-29]` | XVI | For every `import c` the importer emits a shim C file, compiled with … |
| `[FFI-29a]` | XVI | A binding never names a symbol with internal linkage. |
| `[FFI-29b]` | XVI | *(changed in 0.9.15)* `implementation = ["MINIAUDIO_IMPLEMENTATION"]` … |
| `[FFI-29c]` | XVI | A call to a wrapped `static inline` function costs one extra call … |
| `[FFI-30]` | XVI | Two imports of one C declaration (by Clang's USR, through typedefs) … |
| `[FFI-30a]` | XVI | An overlay changes the signatures, safety and names of *functions* … |
| `[FFI-30b]` | XVI | `pub import c "…" as vk` re-exports the module under the ordinary … |
| `[FFI-30c]` | XVI | *(changed in 0.9.10)* *(changed in 0.9.15)* A package may distribute … |
| `[FFI-31]` | XVI | *(changed in 0.9.10)* A `cdylib` links its own copy of the runtime. … |
| `[FFI-31a]` | XVI | Its thread detachment never relies on a TLS destructor in the … |
| `[FFI-31b]` | XVI | *(changed in 0.9.15)* No owning Ember value crosses a module … |
| `[FFI-31c]` | XVI | With `[build] runtime = "shared"`, several Ember modules in one … |
| `[FFI-32]` | Annex C | *(changed in 0.9.15)* A C++ class that is standard-layout and … |
| `[FFI-33]` | XVI | *(changed in 0.9.15)* The export attribute's `threads=any \| main \| … |
| `[FFI-33a]` | XVI | *(changed in 0.9.15)* Attaching a thread (`[FFI-22]`) gives it no … |
| `[FFI-33b]` | XVI | *(changed in 0.9.15)* `returns_owned` results and callbacks may carry … |
| `[FFI-33c]` | XVI | *(changed in 0.9.15)* Under `threads = main`, the exported wrapper … |
| `[FFI-35a]` | XVI | *(changed in 0.9.15)* A parameter with no `retained` word is recorded … |
| `[FFI-36]` | Annex C | *(changed in 0.9.15)* A C++ function returning a raw pointer with no … |
| `[FFI-36a]` | XVI | *(changed in 0.9.15)* `std.ffi.adopt[T](h) -> owns T` is an `unsafe … |
| `[FFI-36b]` | XVI | Adopting a type whose overlay says `adopt = false` is `E5052`. … |
| `[FFI-37]` | Annex C | `ember test --instrument-ffi` runs the tests with the allocation, … |
| `[FFI-37a]` | Annex C | The intercepted set is published and recorded in the evidence; an … |
| `[FFI-37b]` | Annex C | The evidence records how often each foreign function was called and … |
| `[FFI-37c]` | Annex C | `instrumented` means "no counterexample was observed on the paths … |
| `[FFI-37d]` | Annex C | *(changed in 0.9.15)* Only effect facts (`Alloc`, `Block`, `Lock`, … |
| `[FFI-38]` | XVI | The importer rejects rather than guesses: a fact it cannot establish … |
| `[FFI-39]` | Annex C | *(changed in 0.9.10)* *(changed in 0.9.15)* An Ember class may derive … |
| `[FFI-39a]` | Annex C | The trampoline calls the overrides through their permanent thunks, so … |
| `[FFI-39b]` | Annex C | *(changed in 0.9.10)* `super.init(…)` selects the C++ base … |
| `[FFI-39c]` | Annex C | *(changed in 0.9.10)* *(changed in 0.9.15)* A C++ parameter that … |
| `[FFI-39d]` | Annex C | *(changed in 0.9.15)* Re-entrant calls from C++ into an Ember object … |
| `[FFI-39e]` | Annex C | A C++ exception never crosses into Ember code, and an Ember panic … |
| `[FFI-40]` | Annex C | *(changed in 0.9.15)* A method imported from C++ is a method of the … |
| `[FFI-40a]` | Annex C | *(changed in 0.9.15)* `const` is not an aliasing guarantee in C++: a … |
| `[FFI-41]` | Annex C | *(changed in 0.9.15)* Static member functions import as functions of … |
| `[FFI-42]` | Annex C | *(changed in 0.9.10)* Overloaded operators map to Ember operator … |
| `[FFI-42a]` | Annex C | *(changed in 0.9.15)* A `for` loop over an imported C++ container … |
| `[FFI-43]` | Annex C | An overlay that marks a function both `noexcept` and throwing, or two … |
| `[FFI-44]` | Annex C | *Automatic* means no overlay is needed; *overlay* means a contract … |
| `[FFI-48]` | Annex C | *(changed in 0.9.10)* Unsupported constructs (`E5034`, naming the … |
| `[FFI-49]` | XVI | `@ffi(link_name="sym")` binds a declaration to a differently named … |
| `[FFI-50]` | Annex C | The manifest section `[cpp.<project>]` names the C++ project: … |
| `[FFI-51]` | XVI | *(new in 0.9.15)* C compiled into the program is vouched for. An … |
| `[FFI-52]` | XVI | *(new in 0.9.15)* What a foreign call may run. The compiler cannot … |
| `[FFI-53]` | XVI | *(new in 0.9.15)* The checked trampoline. C never holds the address … |
| `[FFI-54]` | XVI | *(new in 0.9.15)* Giving an object to C. A C function declared as … |
| `[FFI-55]` | XVI | *(new in 0.9.15)* Every overlay is checked against the header it was … |
| `[FFI-56]` | XVI | *(new in 0.9.15)* A C function pointer is unsafe to call. A function … |

## FMT

| Rule | Part | Begins |
|---|---|---|
| `[FMT-1]` | XVII | *(changed in 0.9.15)* `ember fmt` writes LF line endings, four-space … |
| `[FMT-2]` | XVII | The formatter uses the `=>` form of a lambda whose body is one … |
| `[FMT-3]` | XVII | The formatter never emits `;` outside `[T; N]` and `[v; N]`. |

## FN

| Rule | Part | Begins |
|---|---|---|
| `[FN-1]` | V | *(changed in 0.9.15)* Parameter words. What a function may do with a … |
| `[FN-1a]` | V | *(changed in 0.9.15)* A `MutSpan[T]` parameter that writes the … |
| `[FN-2]` | V | *(changed in 0.9.15)* A parameter with no word reads. There is no … |
| `[FN-2a]` | V | *(changed in 0.9.15)* A call says `ref mut` before an argument the … |
| `[FN-3]` | V | *(changed in 0.9.15)* A function hands its result over to its caller. … |
| `[FN-4]` | V | *(changed in 0.9.15)* Methods and `this`. Inside a method its object … |
| `[FN-5]` | V | Default argument expressions are evaluated at each call, after the … |
| `[FN-6]` | V | *(changed in 0.9.15)* Callable types. A function is a value. A … |
| `[FN-7]` | V | Recursion is permitted; tail calls are not guaranteed to be … |
| `[FN-8]` | V | *(changed in 0.9.10)* *(changed in 0.9.15)* `main` is `fn main()`, … |
| `[FN-9]` | V | *(changed in 0.9.15)* A parameter of a class type takes its word as … |
| `[FN-10]` | V | A function whose return type is `Result[void, E]` returns `Ok(())` … |

## GPU

| Rule | Part | Begins |
|---|---|---|
| `[GPU-1]` | Annex D | GPU objects are named by generational handles (`[HND-1]`), which are … |
| `[GPU-2]` | Annex D | *(changed in 0.9.15)* `device.begin_frame() -> Option[Frame]`; `None` … |
| `[GPU-3]` | Annex D | *(changed in 0.9.15)* `frame.arena()` is the frame's transient memory … |
| `[GPU-4]` | Annex D | Writing or mapping a resource that the GPU may still be reading is a … |
| `[GPU-5]` | Annex D | Binding a resource declares its access; `cmd.read(h)`/`cmd.write(h)` … |
| `[GPU-6]` | Annex D | *(changed in 0.9.15)* `device.destroy(h)` invalidates the handle at … |
| `[GPU-7]` | Annex D | *(changed in 0.9.15)* In `debug`, ending a device while handles are … |
| `[GPU-8]` | Annex D | `History[T]` owns the per-frame copies of a temporal resource … |
| `[GPU-9]` | Annex D | *(changed in 0.9.15)* `std.gpu.graph` is an optional render-graph … |
| `[GPU-10]` | Annex D | `ember shader-bind <reflection.json>`, reading the reflection a … |
| `[GPU-11]` | Annex D | Ember assumes no shader language: anything that produces SPIR-V and … |
| `[GPU-12]` | Annex D | *(new in 0.9.15)* Every draw and dispatch is checked against what is … |

## GRM

| Rule | Part | Begins |
|---|---|---|
| `[GRM-1]` | III | A class has at most one base class, written in parentheses: `class … |
| `[GRM-2]` | III | A file contains imports and items and, in the entry file only (the … |
| `[GRM-3]` | III | *(changed in 0.9.15)* `Array[T]`, `Map[K, V]`, `Option[T]`, … |
| `[GRM-4]` | III | *(changed in 0.9.15)* `x = e` where no `x` is in scope declares `x` … |
| `[GRM-5]` | III | `a, b = e` destructures a tuple, a struct or a fixed array. The right … |
| `[GRM-6]` | III | The optional `else` of `while` and `for` runs when the loop ends … |
| `[GRM-7]` | III | `defer` blocks run in reverse order at the exit of the enclosing … |
| `[GRM-8]` | III | `name[…]` in expression position is resolved during name resolution: … |
| `[GRM-8a]` | III | *(changed in 0.9.15)* Inside `[ ]` in expression position the parser … |
| `[GRM-8b]` | III | *(changed in 0.9.10)* *(changed in 0.9.15)* An index whose argument … |
| `[GRM-8c]` | III | `IDENT = type` inside `[ ]` is an associated-type binding … |
| `[GRM-8d]` | III | A `type_alias` with an `in` clause declares a range type (`[RNG-1]`). … |
| `[GRM-10]` | III | A `match` whose arms use `pattern: block` is a statement; one whose … |
| `[GRM-11]` | III | There are no block expressions. A value computed by several … |
| `[GRM-12]` | III | An identifier in pattern position names a unit variant or a `const` … |
| `[GRM-13]` | III | *(changed in 0.9.15)* A name a pattern binds in a place it does not … |
| `[GRM-15]` | III | *(changed in 0.9.10)* *(changed in 0.9.15)* `owns e` hands `e` over. … |
| `[GRM-16]` | III | `return`, `break` and `continue` are expressions of type `Never`; … |
| `[GRM-17]` | III | *(changed in 0.9.10)* A lambda with a `:` body inside brackets holds … |
| `[GRM-18]` | III | `;` never separates statements. `a = 1; b = 2` is `E0105`, whose help … |
| `[GRM-19]` | III | *(changed in 0.9.15)* The pattern of a `condition` MUST be refutable. … |
| `[GRM-20]` | III | The attributes before a statement are limited to `@parallel`, … |
| `[GRM-21]` | III | `gen fn` declares a generator (`[CORO-1]`). It is legal wherever `fn` … |
| `[GRM-23]` | III | `x in c` and `x not in c` are membership tests at comparison … |
| `[GRM-24]` | III | Ember has one path separator, `.`. A path resolves left to right: … |
| `[GRM-25]` | III | A chain of the comparison operators `==`, `!=`, `<`, `>`, `<=`, `>=` … |
| `[GRM-26]` | III | A `{…}` atom is a map literal if its first element is followed by … |
| `[GRM-27]` | III | *(changed in 0.9.10)* *(changed in 0.9.15)* A comprehension is … |
| `[GRM-28]` | III | *(changed in 0.9.15)* A `fn_decl` inside a block declares a local … |
| `[GRM-29]` | III | An `expr_list` of two or more expressions, or of one expression … |
| `[GRM-30]` | III | *(changed in 0.9.15)* Casts. `cast s as Circle` gives the `Circle` … |
| `[GRM-31]` | III | *(changed in 0.9.10)* A `some` type (`[TYP-32]`) is legal only as a … |
| `[GRM-32]` | III | `comptime(e)` evaluates the expression `e` at compile time (`[CT-6]`). |
| `[GRM-33]` | III | A bodiless `fn_decl` is legal only in an `interface`, an … |
| `[GRM-34]` | III | *(changed in 0.9.10)* `extend [T: needs B] Array[T] implements I:` … |
| `[GRM-35]` | XVI | *(changed in 0.9.15)* Overlay grammar. An overlay is a file whose … |
| `[GRM-36]` | III | *(changed in 0.9.15)* `ref e`, `ref mut e` and `const ref mut e` name … |
| `[GRM-37]` | III | *(changed in 0.9.15)* A directive is a line beginning `#!` before the … |
| `[GRM-38]` | III | *(changed in 0.9.15)* A parenthesised comprehension `(e for x in it … |
| `[GRM-39]` | III | Nesting depth. Every compiler accepts expressions, statements, … |
| `[GRM-40]` | III | *(new in 0.9.15)* Type parameters go on a line of their own, directly … |
| `[GRM-41]` | III | *(new in 0.9.15)* A function's first line gives its parts in this … |
| `[GRM-42]` | III | *(new in 0.9.15)* Capture words, and lists of names without types. A … |
| `[GRM-43]` | III | *(new in 0.9.15)* Words before a value. `copy of`, `owns`, `shares`, … |
| `[GRM-44]` | III | *(new in 0.9.15)* Opening an `Option` or a `weakref` link: `exists … |
| `[GRM-45]` | III | *(new in 0.9.15)* The kind test. In an `if` or `elif` header, `s is … |
| `[GRM-46]` | III | *(new in 0.9.15)* Clauses after a field's type. `from` after a view … |

## HASH

| Rule | Part | Begins |
|---|---|---|
| `[HASH-1]` | IV | *(changed in 0.9.15)* `Hash.hash` is generic over `H: needs Hasher` … |
| `[HASH-2]` | IV | `std.collections.DefaultHasher` is a fixed-seed hasher: the same keys … |
| `[HASH-3]` | IV | *(changed in 0.9.15)* `Map` and `Set` MUST NOT weaken equality to … |
| `[HASH-4]` | IV | `Map[K, V]` requires `K: needs (Eq, Hash)`. A map may call `hash` any … |

## HEAP

| Rule | Part | Begins |
|---|---|---|
| `[HEAP-1]` | IX | *(changed in 0.9.15)* Every heap type allocates through … |
| `[HEAP-2]` | IX | Growable buffers double, from a minimum of four elements; … |
| `[HEAP-3]` | IX | *(changed in 0.9.15)* A value with several owners (`shares`, `shares … |
| `[HEAP-6]` | IX | *(changed in 0.9.15)* `shares T`, `shares mut T`, `thread_shared T` … |
| `[HEAP-7]` | IX | *(changed in 0.9.15)* `weakref T` links to a value that lives on its … |
| `[HEAP-8]` | IX | Capacity arithmetic is checked: a requested capacity whose byte size … |
| `[HEAP-9]` | IX | *(changed in 0.9.15)* Heap storage for elements of type `T` is … |
| `[HEAP-10]` | IX | *(changed in 0.9.15)* A `shares` owner, and a `weakref` link to a … |

## HND

| Rule | Part | Begins |
|---|---|---|
| `[HND-1]` | IX | A `Handle[T]` is a plain `Copy` value (index and generation). Every … |
| `[HND-2]` | IX | `Handle[T]` is a `u64`: 32 bits of index and 32 of generation. … |
| `[HND-3]` | IX | *(changed in 0.9.15)* A slot whose generation is exhausted is retired … |

## HR

| Rule | Part | Begins |
|---|---|---|
| `[HR-1]` | Annex B | An edit to one function body in a 50k-line reloadable package MUST be … |
| `[HR-2]` | Annex B | *(changed in 0.9.15)* Reload is transactional. A failure in plan or … |
| `[HR-2a]` | Annex B | *(changed in 0.9.15)* Plan reads schemas, the facts of `[HR-44]` and … |
| `[HR-3]` | Annex B | *(changed in 0.9.10)* A reload is applied only inside … |
| `[HR-3a]` | Annex B | Every thread that can run Ember code is registered: host threads on … |
| `[HR-3b]` | Annex B | *(new in 0.9.10)* *(changed in 0.9.15)* At a checkpoint the frames … |
| `[HR-4]` | Annex B | Old images are never unloaded, so a stale return address or retired … |
| `[HR-4a]` | Annex B | Statics live in a runtime-owned table, never in image memory. |
| `[HR-5]` | Annex B | A call to a reloadable function goes through its thunk. |
| `[HR-6]` | Annex B | Each reloadable function gets one thunk, at a fixed address for the … |
| `[HR-6a]` | Annex B | *(changed in 0.9.15)* Hence function values (their code word), method … |
| `[HR-7]` | Annex B | Call-table slots are identified by mangled name (`[MNG-1]`), never by … |
| `[HR-8]` | Annex B | *(changed in 0.9.15)* Each image carries a reload manifest section … |
| `[HR-9]` | Annex B | A call through a thunk costs one load and one indirect jump; the … |
| `[HR-9a]` | Annex B | *(changed in 0.9.10)* A reloadable function is never inlined, … |
| `[HR-10]` | Annex B | *(changed in 0.9.15)* `@noreload fn` is called directly and may be … |
| `[HR-10a]` | Annex B | A `@noreload` function may call reloadable functions, through their … |
| `[HR-11]` | Annex B | *(changed in 0.9.15)* Every type in a reloadable package has a … |
| `[HR-11a]` | Annex B | Enum values are migrated by variant name, never by discriminant. |
| `[HR-12]` | Annex B | *(changed in 0.9.15)* In a reloadable build, class instances are … |
| `[HR-12a]` | Annex B | The header size is a whole-process property: every package in a … |
| `[HR-13]` | Annex B | Value-typed data (structs in arrays, `SoA` columns, ECS storage, … |
| `[HR-13a]` | Annex B | Standard-library containers of a type from a reloadable package … |
| `[HR-13b]` | Annex B | Value data reachable only through raw pointers or foreign memory … |
| `[HR-14]` | Annex B | *(changed in 0.9.10)* *(changed in 0.9.15)* Migration is by name, and … |
| `[HR-15]` | Annex B | *(changed in 0.9.10)* *(changed in 0.9.15)* Migration keeps an … |
| `[HR-15a]` | Annex B | *(changed in 0.9.15)* An owner, link or view the runtime cannot … |
| `[HR-15b]` | Annex B | Every refusal is decided in plan, before prepare runs. |
| `[HR-16]` | Annex B | *(changed in 0.9.15)* `static fn migrate_from(old: OldSelf) -> … |
| `[HR-17]` | Annex B | A static whose type and initialiser are unchanged keeps its value; … |
| `[HR-17a]` | Annex B | *(changed in 0.9.15)* Editing a static's initialiser changes its … |
| `[HR-18]` | Annex B | A refusal returns a report naming each type and change that caused … |
| `[HR-18a]` | Annex B | A refusal is never partial: nothing of the new image is live. |
| `[HR-18b]` | Annex B | `ember build --reload --explain` predicts the outcome against the … |
| `[HR-19]` | Annex B | *(changed in 0.9.15)* `reload = "bodies"` admits only function-body … |
| `[HR-20]` | Annex B | *(changed in 0.9.10)* *(changed in 0.9.15)* An object C holds — … |
| `[HR-21]` | Annex B | A foreign table of exported function pointers stays valid across … |
| `[HR-22]` | Annex B | Foreign objects owned by Ember are carried across unchanged. |
| `[HR-23]` | Annex B | *(changed in 0.9.10)* A changed C or C++ header refuses the reload … |
| `[HR-24]` | Annex B | *(changed in 0.9.10)* An object in use by an in-flight GPU frame is … |
| `[HR-25]` | Annex B | `reload` is `"all"`, `"opt-in"`, `"bodies"` or `"none"` (`[MAN-7]`); … |
| `[HR-27]` | Annex B | A `release` build is bit-identical whether or not the source uses any … |
| `[HR-28]` | Annex B | `ember inspect --safety` reports the thunk indirection of each … |
| `[HR-29]` | Annex B | With reload enabled, one shared runtime serves every image in the … |
| `[HR-30]` | Annex B | Host contract: `ember_reload_init(&config)` once; … |
| `[HR-31]` | Annex B | Polling never blocks; compilation runs in the background. |
| `[HR-32]` | Annex B | `ember_reload_stats()` reports the last reload split into compile, … |
| `[HR-33]` | Annex B | `ember run --hot` is the toolchain's own host: it builds under … |
| `[HR-34]` | Annex B | Prepare cannot panic: every operation it performs is either free of … |
| `[HR-35]` | Annex B | *(changed in 0.9.15)* `migrate_from` cannot abort the process. It may … |
| `[HR-36]` | Annex B | Allocation during prepare comes from the transaction arena; … |
| `[HR-37]` | Annex B | The transaction arena holds everything prepare allocates and is … |
| `[HR-38]` | Annex B | *(changed in 0.9.10)* `std.hot.ReloadError` is … |
| `[HR-39]` | Annex B | A foreign call inside `migrate_from` returns its failure as … |
| `[HR-41]` | Annex B | *(changed in 0.9.10)* *(changed in 0.9.15)* Failure matrix. |
| `[HR-42]` | Annex B | Publication. Entering Ember code increments the thread's depth … |
| `[HR-42a]` | Annex B | The counter is not a lock: entering Ember code is one relaxed … |
| `[HR-43]` | Annex B | `@allow_reload_terminate` on a `migrate_from` admits such calls and … |
| `[HR-44]` | Annex B | *(new in 0.9.15)* The facts the compiler worked out are part of what … |
| `[HR-45]` | Annex B | *(new in 0.9.15)* A suspended generator keeps its frame. A `gen fn`'s … |
| `[HR-46]` | Annex B | *(new in 0.9.15)* Values made in a memory manager's instance. A value … |
| `[HR-47]` | Annex B | *(new in 0.9.15)* `@reinit_on_reload` ends the static's old value … |

## IDE

| Rule | Part | Begins |
|---|---|---|
| `[IDE-3]` | XVII | Every compiler stage after parsing produces a complete result for a … |
| `[IDE-4]` | XVII | *(changed in 0.9.15)* Every editor request is answered from parsing, … |
| `[IDE-6]` | XVII | *(changed in 0.9.10)* *(changed in 0.9.15)* The compiler keeps all … |

## IFC

| Rule | Part | Begins |
|---|---|---|
| `[IFC-1]` | V | `extend T:` without `implements` adds inherent methods to `T`; it is … |
| `[IFC-2]` | V | Adding inherent methods to a type from another package is `E2120`; … |
| `[IFC-3]` | V | `interface Ord: needs Eq` requires implementers of `Ord` to implement … |
| `[IFC-4]` | V | Interfaces may declare associated types and constants, not statics. … |
| `[IFC-5]` | V | *(new in 0.9.15)* Interface values say who owns them. `owns dyn I` is … |
| `[IFC-6]` | V | *(new in 0.9.15)* Bounds of a generic function. A function that is … |

## IMP

| Rule | Part | Begins |
|---|---|---|
| `[IMP-11]` | XVIII | *(changed in 0.9.15)* The reference implementation verifies its … |

## JOB

| Rule | Part | Begins |
|---|---|---|
| `[JOB-1]` | XI | A job's captured state is stored inline in its queue slot when it … |
| `[JOB-2]` | XI | *(changed in 0.9.15)* `jobs.scope()` follows `[THR-5]` and … |
| `[JOB-3]` | XI | *(changed in 0.9.15)* `s.submit_after(deps, f)` starts `f` only after … |
| `[JOB-5]` | XI | `jobs.local_arena()` returns an arena view for the current job, reset … |

## LAY

| Rule | Part | Begins |
|---|---|---|
| `[LAY-2]` | IX | Layout attributes change layout exactly as stated, in every profile … |

## LEX

| Rule | Part | Begins |
|---|---|---|
| `[LEX-1]` | II | Source files are UTF-8. A byte-order mark is accepted and ignored; … |
| `[LEX-2]` | II | Line endings are LF or CRLF, both normalised to LF before … |
| `[LEX-3]` | II | The source extension is `.em`. |
| `[LEX-4]` | II | Indentation MUST use spaces. A tab at the start of a logical line is … |
| `[LEX-5]` | II | The lexer emits `NEWLINE`, `INDENT` and `DEDENT` with Python's … |
| `[LEX-6]` | II | Inside `(`, `[` and `{`, and inside a triple-quoted string, newlines … |
| `[LEX-6a]` | II | Inside brackets, a lambda's `:` body is a single simple statement … |
| `[LEX-7]` | II | A backslash at the end of a physical line joins it to the next. The … |
| `[LEX-8]` | II | Blank lines and comment-only lines do not affect indentation. |
| `[LEX-9]` | II | An indented block MUST be introduced by a line ending in `:`. A `:` … |
| `[LEX-10]` | II | There are no block comments. A line beginning `#!` before the first … |
| `[LEX-11]` | II | A `##` comment attaches to the next declaration, ignoring blank … |
| `[LEX-11a]` | II | A `##` comment on a comment-only line is emitted after the … |
| `[LEX-12]` | II | Identifiers are NFC-normalised; two identifiers are the same iff … |
| `[LEX-13]` | II | A lone `_` is the discard pattern and never names a variable. |
| `[LEX-14]` | II | *(changed in 0.9.15)* A raw identifier `r#name` uses a keyword as a … |
| `[LEX-15]` | II | *(changed in 0.9.10)* *(changed in 0.9.15)* The table above is the … |
| `[LEX-16]` | II | An integer literal without a suffix is an untyped integer: it takes … |
| `[LEX-17]` | II | A float literal without a suffix is an untyped float: it takes the … |
| `[LEX-17a]` | II | A float literal that receives `f32` or `f16` and has more significant … |
| `[LEX-18]` | II | `1.` followed by an identifier character is a method call on `1`; … |
| `[LEX-19]` | II | An f-string `{…}` contains a full expression. `{{` and `}}` are … |
| `[LEX-20]` | II | *(changed in 0.9.15)* A string literal has type `StrView` with the … |
| `[LEX-21]` | II | *(changed in 0.9.15)* Tokenisation is maximal munch: `//=` before … |
| `[LEX-22]` | II | Ember has no lifetime syntax and never will. A `'` begins a character … |
| `[LEX-23]` | II | *(changed in 0.9.15)* A line comment whose text begins `SAFETY:` or … |
| `[LEX-24]` | II | A unary minus applied directly to an untyped integer literal forms a … |
| `[LEX-25]` | II | A raw string contains no escapes; a `r#"…"#` raw string may contain … |
| `[LEX-26]` | II | *(new in 0.9.10)* A literal is written as the grammar above gives it: … |

## LNT

| Rule | Part | Begins |
|---|---|---|
| `[LNT-1]` | XVII | `L1001 unused binding`: a local never read on any path (names … |
| `[LNT-2]` | XVII | `L1002 assignment declares a new binding`: an unread new name within … |
| `[LNT-3]` | XVII | `L1001` and `L1002` are reported by `ember build` and `ember check`, … |
| `[LNT-4]` | XVII | `L2004`: a `gen fn` with no `yield`. |
| `[LNT-5]` | XVII | `L2005`: a `@noreload` function calling a reloadable one in a loop. |
| `[LNT-6]` | XVII | *(changed in 0.9.15)* `ember lint` also reports, at `warn` unless … |

## LT

| Rule | Part | Begins |
|---|---|---|
| `[LT-1]` | VII | *(changed in 0.9.15)* Where a call's view points. A call's result … |
| `[LT-1a]` | VII | *(changed in 0.9.10)* *(changed in 0.9.15)* `from`. `-> T from name`, … |
| `[LT-3]` | VII | String literals, `bytes` literals, `static` items and views of them … |
| `[LT-4]` | VII | *(changed in 0.9.15)* A value made in a memory manager's instance … |
| `[LT-5]` | VII | *(changed in 0.9.15)* Regions of locals are inferred by non-lexical … |
| `[LT-6]` | VII | *(changed in 0.9.15)* Named lifetimes are not part of Ember and will … |
| `[LT-7]` | VII | *(changed in 0.9.15)* Function types. Each call through a value or … |
| `[LT-14]` | VII | *(changed in 0.9.15)* A view type carries a compiler-internal region … |
| `[LT-16]` | VII | Constructing a view value keeps each field's region; it never … |
| `[LT-17]` | VII | Validity is conjunctive. A view value is usable only while every … |
| `[LT-18]` | VII | *(changed in 0.9.15)* Every view field is an ordinary view under … |
| `[LT-20]` | VII | Moving, copying, destructuring, passing and returning a view … |
| `[LT-21]` | VII | Assigning a new view into a field recomputes that field's region; the … |
| `[LT-22]` | VII | *(changed in 0.9.15)* A function returning a view type gets, from its … |
| `[LT-23]` | VII | *(changed in 0.9.15)* With `from` on a function returning a … |
| `[LT-24]` | VII | A region slot ends at the last use of its own field, not of the whole … |
| `[LT-25]` | VII | *(changed in 0.9.15)* A view field may not point into another field … |
| `[LT-26]` | VII | *(changed in 0.9.15)* A view extends no lifetime: constructing, … |
| `[LT-27]` | VII | *(changed in 0.9.15)* Different regions never prove that two views do … |
| `[LT-30]` | VII | Regions exist only at compile time: two values of one view type with … |
| `[LT-34]` | VII | *(changed in 0.9.15)* A view type whose fields all point into one … |
| `[LT-35]` | VII | *(changed in 0.9.15)* For each function that can receive a view … |
| `[LT-36]` | VII | *(changed in 0.9.15)* Treating the view as a whole (passing it where … |
| `[LT-38]` | VII | Moving one field out moves only that field's constraint; the … |
| `[LT-39]` | VII | *(changed in 0.9.10)* *(changed in 0.9.15)* An operation that selects … |
| `[LT-42]` | VII | *(changed in 0.9.15)* A function value that uses a real variable … |
| `[LT-43]` | VII | Region tracking never lets a view survive a `yield` that `[CORO-6]` … |
| `[LT-44]` | VII | *(changed in 0.9.15)* A plain or `ref mut` `Arena`, `FixedArena` or … |
| `[LT-45]` | VII | *(new in 0.9.15)* Kept views. A struct that holds a view (`ref T`, … |
| `[LT-46]` | VII | *(new in 0.9.15)* The three kinds of kept view, by their word: * `ref … |
| `[LT-47]` | VII | *(new in 0.9.15)* A view put in through `ref mut`. After a call, a … |

## MAN

| Rule | Part | Begins |
|---|---|---|
| `[MAN-1]` | XVII | An invalid manifest, including one with an unknown key, is `E9001`, … |
| `[MAN-2]` | XVII | *(changed in 0.9.15)* `ember.lock` records the resolved dependencies … |
| `[MAN-3]` | XVII | Every key in `[lints]` names a lint the compiler defines (`E9010` … |
| `[MAN-7]` | XVII | *(changed in 0.9.10)* `[build] reload` is `"opt-in"`, `"all"`, … |
| `[MAN-8]` | XVII | *(changed in 0.9.10)* The sections are: `[package]` (`name`, … |
| `[MAN-9]` | XVII | *(new in 0.9.15)* Every program is a package (verdict 127). A command … |

## MNG

| Rule | Part | Begins |
|---|---|---|
| `[MNG-1]` | XVIII | *(changed in 0.9.10)* *(changed in 0.9.15)* Mangling is injective. A … |
| `[MNG-2]` | XVIII | *(changed in 0.9.15)* `@unsafe(export("name"))` sets the symbol … |
| `[MNG-3]` | XVIII | *(changed in 0.9.10)* Non-ASCII identifier characters are … |
| `[MNG-4]` | XVIII | *(changed in 0.9.10)* Object structs, method tables and type … |
| `[MNG-6]` | XVIII | *(new in 0.9.10)* *(changed in 0.9.15)* The canonical spelling of a … |

## MOD

| Rule | Part | Begins |
|---|---|---|
| `[MOD-1]` | V | *(changed in 0.9.10)* *(changed in 0.9.15)* A package is a directory … |
| `[MOD-2]` | V | Items are private to their module unless marked. `pub(package)` makes … |
| `[MOD-3]` | V | `import a.b.c` binds the name `c` to module `a.b.c`, and `import … |
| `[MOD-4]` | V | Import cycles within a package are allowed; cycles between packages … |
| `[MOD-5]` | V | *(changed in 0.9.15)* The prelude. Every module implicitly imports … |
| `[MOD-7]` | V | A field declared `pub(read)` (or `pub(package, read)`) may be read … |
| `[MOD-8]` | V | `from m import *` binds every `pub` item of `m`. A name bound by two … |
| `[MOD-9]` | V | *(new in 0.9.15)* A package's edge. What a dependant may rely on is … |

## MONO

| Rule | Part | Begins |
|---|---|---|
| `[MONO-1]` | XVIII | *(changed in 0.9.10)* Generic code is instantiated per distinct set … |
| `[MONO-2]` | XVIII | The compiler records, per generic, how many instances it produced and … |
| `[MONO-3]` | XVIII | `[build] max_instantiations = N` sets a per-generic ceiling (unset by … |
| `[MONO-5]` | XVIII | A generic is shareable at a parameter `T` when `T` appears in its … |
| `[MONO-6]` | XVIII | Only for a shareable generic whose instance count exceeds the ceiling … |
| `[MONO-7]` | XVIII | `@always_specialize` forbids sharing for a generic; … |
| `[MONO-8]` | XVIII | A shared body computes exactly what the specialised ones would; it … |
| `[MONO-9]` | XVIII | A shared body is its own symbol and is deduplicated like any … |
| `[MONO-10]` | XVIII | *(new in 0.9.15)* One whole variable under two names. A call that the … |

## OBJ

| Rule | Part | Begins |
|---|---|---|
| `[OBJ-1]` | VIII | *(changed in 0.9.15)* The header is 8 bytes: a 32-bit count and a … |
| `[OBJ-2]` | VIII | *(changed in 0.9.10)* *(changed in 0.9.15)* An owner, a link or a … |
| `[OBJ-3]` | VIII | *(changed in 0.9.15)* The weak count starts at 1 on behalf of all … |
| `[OBJ-4]` | VIII | *(changed in 0.9.10)* *(changed in 0.9.15)* Objects are allocated … |
| `[OBJ-5]` | VIII | *(changed in 0.9.15)* An ended object is never brought back. Inside a … |

## OPT

| Rule | Part | Begins |
|---|---|---|
| `[OPT-1]` | VIII | *(changed in 0.9.10)* *(changed in 0.9.15)* When escape analysis … |
| `[OPT-2]` | X | For a counted loop over `a..b` that indexes views at `i + c` for … |
| `[OPT-3]` | X | The loop bound in `[OPT-2]` may be written `s.len()`, a separate … |

## OWN

| Rule | Part | Begins |
|---|---|---|
| `[OWN-1]` | VII | *(changed in 0.9.15)* Every value has exactly one owner: a local, a … |
| `[OWN-2]` | VII | *(changed in 0.9.15)* A value is dropped (ended) when its owner goes … |
| `[OWN-3]` | VII | *(changed in 0.9.15)* `=` with a whole name hands it over. `b = a`, … |
| `[OWN-4]` | VII | *(changed in 0.9.15)* A loop body that hands over a value declared … |
| `[OWN-5]` | VII | Assigning to a place that holds a live value evaluates the new value, … |
| `[OWN-6]` | VII | *(changed in 0.9.15)* `mem.take(ref mut place: T) -> T` (leaves … |
| `[OWN-7]` | VII | *(changed in 0.9.15)* A value of a `Copy` type is duplicated bit for … |
| `[OWN-8]` | VII | *(changed in 0.9.15)* `copy of x` makes a copy of the thing itself, … |
| `[OWN-9]` | VII | *(new in 0.9.15)* `owns` hands over, for every type, numbers … |
| `[OWN-10]` | VII | *(new in 0.9.15)* `owns` on the line that makes a value for a place … |
| `[OWN-11]` | VII | *(new in 0.9.15)* A part handed over on its own. A part of a struct … |
| `[OWN-12]` | VII | *(new in 0.9.15)* Giving an object to C. A C function whose … |
| `[OWN-13]` | VII | *(new in 0.9.15)* Words before a value. Before a value, a word acts … |

## PAN

| Rule | Part | Begins |
|---|---|---|
| `[PAN-1]` | VI | A panic prints `panic at <file>:<line>:<col>: <message>` (and a … |
| `[PAN-2]` | VI | Formatting a panic message allocates only for an f-string; … |
| `[PAN-3]` | VI | A panic inside a `drop` that runs while the process is already … |

## PAR

| Rule | Part | Begins |
|---|---|---|
| `[PAR-1]` | XI | *(changed in 0.9.15)* A `@parallel` loop splits a range (`for i in … |
| `[PAR-2]` | XI | *(changed in 0.9.15)* Iterations MUST be independent. The body MUST … |
| `[PAR-2a]` | XI | A violation of clause (b) is `E7011`, `parallel loop has a … |
| `[PAR-2b]` | XI | The analysis reads through the body's calls as if inlined: each call … |
| `[PAR-3]` | XI | `@parallel(reduce=[total: +, best: max])` declares variables combined … |
| `[PAR-4]` | XI | A `@parallel` loop has the effects `Sync` and `Block` (it waits for … |
| `[PAR-5]` | XI | Parallel results do not depend on the machine. Chunk boundaries are a … |

## PHIL

| Rule | Part | Begins |
|---|---|---|
| `[PHIL-1]` | I | Two syntactically identical declarations in the same context have … |
| `[PHIL-2]` | I | *(changed in 0.9.15)* No heap allocation happens that the source does … |
| `[PHIL-3]` | I | *(changed in 0.9.15)* No implicit copy of a non-`Copy` value occurs: … |
| `[PHIL-4]` | I | *(changed in 0.9.15)* No implicit synchronisation occurs except … |
| `[PHIL-5]` | I | Every safety check the compiler removes, it removes because it proved … |
| `[PHIL-6]` | I | Every expensive or dangerous conversion at an FFI boundary is … |
| `[PHIL-7]` | I | Panics are for programmer errors. Recoverable failures use `Result`. |
| `[PHIL-8]` | I | *(changed in 0.9.15)* The enforcement ladder. Ember guarantees memory … |
| `[PHIL-8a]` | I | Every rejection shape in §XVII.6 has a mandated `help` that, applied … |
| `[PHIL-9]` | I | *(changed in 0.9.15)* Class objects and shared values carry a header … |
| `[PHIL-10]` | I | *(changed in 0.9.15)* A program containing no `unsafe` boundary … |
| `[PHIL-11]` | I | *(changed in 0.9.15)* What remains possible, and is therefore not a … |
| `[PHIL-12]` | I | No silent acceptance. Every construct a program writes — an … |
| `[PHIL-13]` | I | One meaning per program. No build profile, compiler flag, manifest … |
| `[PHIL-14]` | I | *(changed in 0.9.10)* Python spelling, Python meaning. Where Ember … |
| `[PHIL-15]` | I | *(changed in 0.9.15)* Costs are named. Every cost the language can … |
| `[PHIL-16]` | I | *(new in 0.9.15)* The words say what the code does. Whether a name … |

## PRF

| Rule | Part | Begins |
|---|---|---|
| `[PRF-1]` | XVII | *(changed in 0.9.15)* A profile never changes what a program means. … |
| `[PRF-2]` | XVII | *(changed in 0.9.10)* Hot reload is a build mode of `debug` and … |
| `[PRF-3]` | XVII | *(changed in 0.9.10)* What a profile may set. |
| `[PRF-4]` | XVII | *(new in 0.9.15)* The checks that run while a program runs are the … |

## RC

| Rule | Part | Begins |
|---|---|---|
| `[RC-1]` | VIII | *(changed in 0.9.15)* Several owners. `shares x` makes one more owner … |
| `[RC-2]` | VIII | *(changed in 0.9.15)* No count where no owner is made. No count … |
| `[RC-2a]` | VIII | *(changed in 0.9.15)* Passing a shared value to a parameter that is … |
| `[RC-2b]` | VIII | *(changed in 0.9.15)* A view of a shared value or an object: a name … |
| `[RC-2c]` | VIII | *(changed in 0.9.15)* A +1 immediately followed by a -1 of the same … |
| `[RC-2d]` | VIII | *(changed in 0.9.15)* An owner handed over (`b = a`, an `owns` … |
| `[RC-2e]` | VIII | *(changed in 0.9.15)* The loop names of a `for` over a list of … |
| `[RC-3]` | VIII | *(changed in 0.9.15)* Further removals of count operations are … |
| `[RC-4]` | VIII | *(changed in 0.9.15)* The counts of a value that is not … |
| `[RC-5]` | VIII | *(changed in 0.9.15)* A view of an object reached through a link. A … |
| `[RC-6]` | VIII | *(changed in 0.9.15)* `ember inspect` lists every count operation … |
| `[RC-7]` | VIII | *(new in 0.9.15)* Changing a shared value. A value made shared is … |

## RFL

| Rule | Part | Begins |
|---|---|---|
| `[RFL-1]` | XIV | `reflect[T]()` at compile time returns a `TypeDesc`: name, kind, … |
| `[RFL-2]` | XIV | *(changed in 0.9.15)* `@reflect` on a type emits run-time type … |
| `[RFL-3]` | XIV | User attributes are declared. A struct marked `@attribute` may be … |

## RNG

| Rule | Part | Begins |
|---|---|---|
| `[RNG-1]` | IV | A `type` alias with an `in` clause declares a nominal numeric type … |
| `[RNG-2]` | IV | Two range types are distinct even with equal representation and range … |
| `[RNG-3]` | IV | Construction from a value not known to be in range is `T.checked(v) … |
| `[RNG-3a]` | IV | Every range type with finite endpoints has `T.clamped(v) -> T`, … |
| `[RNG-4]` | IV | The compiler tracks a known range for numeric expressions — literals, … |
| `[RNG-4a]` | IV | For floats, a range fact comes only from the true arm of a comparison … |
| `[RNG-5]` | IV | Arithmetic on a range value yields its representation: `r * 2.0` is … |
| `[RNG-5a1]` | IV | The operators on range types come from compiler-generated … |
| `[RNG-5a2]` | IV | Overload resolution (`[TYP-24]`) picks an implementation matching the … |
| `[RNG-6]` | IV | NaN is in no range. A range from `-0.0` to `+0.0` contains both zeros. |
| `[RNG-7]` | IV | A range type whose range does not cover its whole representation … |
| `[RNG-8]` | IV | A range type is `Copy` when its representation is, has its … |
| `[RNG-9]` | IV | A range value outside its range is invalid; producing one is … |
| `[RNG-10]` | IV | In Safe code a range value arises only from a constant in range, … |
| `[RNG-10a]` | IV | A derived `Deserialize` checks every range-typed field with `checked` … |
| `[RNG-10b]` | IV | A range type may not appear in a foreign signature, directly or … |

## RT

| Rule | Part | Begins |
|---|---|---|
| `[RT-1]` | XVIII | The runtime `ember_rt` is C11 depending on libc and the OS only. … |
| `[RT-2]` | XVIII | The runtime has no global constructors; the generated `main` calls … |
| `[RT-3]` | XVIII | `TypeInfo` holds size, alignment, flags, name, base, drop functions, … |
| `[RT-4]` | XVIII | A panic prints `panic at <file>:<line>:<col>: <message>` naming Ember … |
| `[RT-5]` | XVIII | Every runtime symbol, macro and header name derives from one … |
| `[RT-6]` | XVIII | The runtime ABI version is `EMBER_RUNTIME_ABI`. |
| `[RT-7]` | XVIII | *(changed in 0.9.15)* A count is checked before it rolls over. The … |
| `[RT-8]` | XVIII | *(changed in 0.9.15)* A `thread_shared` value's away count and its … |
| `[RT-10]` | XVIII | *(changed in 0.9.15)* Counting is inline. A count changes only where … |
| `[RT-11]` | XVIII | Allocation statistics are kept per thread, or only in `debug`; no … |
| `[RT-12]` | XVIII | *(changed in 0.9.15)* Stack overflow faults. The backend compiles … |

## SEL

| Rule | Part | Begins |
|---|---|---|
| `[SEL-1]` | IX | *(changed in 0.9.10)* *(changed in 0.9.15)* The order above is the … |
| `[SEL-2]` | IX | *(changed in 0.9.15)* `shares` and `weakref` and C++'s … |

## SER

| Rule | Part | Begins |
|---|---|---|
| `[SER-1]` | XIV | *(changed in 0.9.15)* `@derive(Serialize, Deserialize)` implements … |
| `[SER-2]` | XIV | Deserialising never produces an invalid value: a range-typed field is … |

## SIMD

| Rule | Part | Begins |
|---|---|---|
| `[SIMD-1]` | XII | `std.simd` provides vector types `f32x4`, `f32x8`, `f32x16`, `f64x2`, … |
| `[SIMD-2]` | XII | `@simd` on a `for` loop is a request and a report: the compiler tries … |
| `[SIMD-3]` | XII | *(changed in 0.9.15)* Alias facts given to the backend (`restrict` on … |
| `[SIMD-4]` | XII | Horizontal operations (`reduce_add`, `reduce_min`, `reduce_max`), … |
| `[SIMD-5]` | XII | Vectorisable form, which `@simd(assert)` checks, is computed by Ember … |
| `[SIMD-6]` | XII | The C compiler's own vectorisation report is corroborating evidence … |
| `[SIMD-7]` | XII | Grouped overflow checks. In a loop whose body has none of `Sync`, … |
| `[SIMD-8]` | XII | Vector operators are lane-wise and follow the scalar rules: integer … |
| `[SIMD-9]` | XII | *(changed in 0.9.15)* SIMD memory operations are bounds-checked like … |

## SOA

| Rule | Part | Begins |
|---|---|---|
| `[SOA-1]` | XII | `SoA[T]` is a compiler-known type constructor, like `Cell`: for any … |
| `[SOA-2]` | XII | *(changed in 0.9.15)* `ps.f` names the column of field `f`: a view of … |
| `[SOA-3]` | XII | `SoA[T]` provides `len`, `is_empty`, `push`, `pop`, `swap_remove`, … |
| `[SOA-4]` | XII | `ArenaSoA[T]` is the fixed-capacity, arena-backed form, under the … |
| `[SOA-6]` | XII | *(changed in 0.9.15)* `ps[i]` is an element proxy: `SoARef[T]`, a … |
| `[SOA-7]` | XII | All columns of one `SoA[T]` live in one heap block, each column … |

## SPN

| Rule | Part | Begins |
|---|---|---|
| `[SPN-1]` | VII | *(changed in 0.9.15)* An `Array[T]`, a `[T; N]` or a `String` … |
| `[SPN-2]` | VII | Indexing a view is bounds-checked; `get(i) -> Option[ref T]` does not … |
| `[SPN-3]` | VII | *(changed in 0.9.15)* `Span[T]` is `Copy`; `MutSpan[T]` is not. A … |
| `[SPN-4]` | VII | *(changed in 0.9.15)* `iter()` on a `Span` or `MutSpan` gives … |
| `[SPN-5]` | VII | *(changed in 0.9.15)* `split_at(i)` on a `Span` returns two `Span`s; … |
| `[SPN-8]` | VII | `as_ptr()` and `as_mut_ptr()` return raw pointers; extracting one is … |

## STA

| Rule | Part | Begins |
|---|---|---|
| `[STA-1]` | V | *(changed in 0.9.15)* `static NAME: T = e` is one value per program … |
| `[STA-2]` | V | There is no static-initialisation-order problem: statics are … |
| `[STA-3]` | V | *(changed in 0.9.15)* A static whose initialiser can be evaluated at … |

## STD

| Rule | Part | Begins |
|---|---|---|
| `[STD-1]` | XV | Every function in `std.core`, `std.mem`, `std.math`, `std.simd` and … |
| `[STD-2]` | XV | Printing allocates only to build an f-string argument; … |
| `[STD-3]` | XV | *(changed in 0.9.15)* `math.fma(a, b, c)` (and `mul_add`) is a fused … |
| `[STD-4]` | XV | `NonZero[T]` for each integer `T` is a `Copy` wrapper with a niche … |
| `[STD-5]` | XV | `sum` and `product` combine elements left to right in the element … |
| `[STD-6]` | XV | *(changed in 0.9.15)* `std` is layered, and each layer depends only … |
| `[STD-7]` | XV | Fixed-capacity containers with no heap allocation: `FixedArray[T, … |
| `[STD-8]` | XV | *(changed in 0.9.15)* `Contains`. `x in c` requires `c: … |
| `[STD-8a]` | XV | `ember inspect --cost` reports which `contains` a use selects and its … |
| `[STD-8b]` | XV | *(changed in 0.9.15)* `StrView` implements `Contains[char]` and … |
| `[STD-9]` | XV | *(changed in 0.9.15)* `print(a, b, …, sep=" ", end="")` and … |
| `[STD-10]` | XV | `input(prompt="") -> String` prints the prompt, flushes, reads one … |
| `[STD-11]` | XV | *(changed in 0.9.15)* `Map[K, V, H = DefaultHasher]` is a hash map … |
| `[STD-12]` | XV | *(changed in 0.9.15)* Keys looked up by a view. Lookup methods, … |
| `[STD-13]` | XV | One naming convention. Names are `snake_case` words joined by … |
| `[STD-14]` | XV | *(changed in 0.9.10)* *(changed in 0.9.15)* Failure follows … |
| `[STD-15]` | XV | *(changed in 0.9.15)* `Array[T]` is a growable contiguous list … |
| `[STD-16]` | XV | *(changed in 0.9.15)* `Map` operations. `m[k]` reads the value and … |
| `[STD-17]` | XV | *(changed in 0.9.15)* Index assignment. `a[i] = v` calls … |
| `[STD-18]` | XV | *(changed in 0.9.15)* `format(value, spec="") -> String` formats one … |
| `[STD-19]` | XV | *(changed in 0.9.10)* *(changed in 0.9.15)* Every `Iterator` has the … |
| `[STD-20]` | XV | Integer methods, built into every integer type: `abs` (a signed `MIN` … |
| `[STD-21]` | XV | *(changed in 0.9.15)* `std.math` provides `PI`, `TAU`, `E`; `sin`, … |
| `[STD-22]` | XV | Every operation that touches the outside world returns `Result[T, … |
| `[STD-23]` | XV | `time.Instant.now()` is monotonic and `Nondet`; `Duration` is an … |
| `[STD-24]` | XV | `process.exit(code) -> Never` flushes the standard streams and exits … |
| `[STD-25]` | XV | *(changed in 0.9.15)* `random.Rng.seeded(seed)` is a deterministic … |
| `[STD-26]` | XV | *(changed in 0.9.10)* *(changed in 0.9.15)* Python's built-in … |
| `[STD-27]` | XV | `std.math.Number` is every number type. `std.math` adds each with an … |
| `[STD-28]` | XV | Vectors, matrices, rotations and shapes (ODR-043). `std.math` has … |
| `[STD-29]` | XV | *(new in 0.9.15)* `std.mem.Volatile[*T]`'s `read_volatile` and … |

## STR

| Rule | Part | Begins |
|---|---|---|
| `[STR-1]` | V | *(changed in 0.9.15)* Every struct has a memberwise constructor … |
| `[STR-2]` | V | A field default is any expression; it is evaluated at each … |
| `[STR-3]` | V | *(changed in 0.9.15)* A struct with a `drop` method, or with a field … |
| `[STR-4]` | V | A struct may have no fields (`struct Marker: pass`). |
| `[STR-5]` | V | *(changed in 0.9.15)* Implicit derives. A struct or enum implements … |
| `[STR-6]` | V | *(changed in 0.9.15)* `fn init(…)` on a struct is its constructor; it … |
| `[STR-7]` | V | *(changed in 0.9.15)* Inside the body of a `struct`, `enum`, `class` … |

## TCB

| Rule | Part | Begins |
|---|---|---|
| `[TCB-1]` | Annex C | Every fact an overlay states about foreign code carries a grade: … |
| `[TCB-2]` | Annex C | The report separates what the language guarantees from what external … |
| `[TCB-3]` | Annex C | Every assumption a guarantee relies on appears in the report with its … |
| `[TCB-4]` | Annex C | *(changed in 0.9.15)* Entries are categorised — language, compiler, … |
| `[TCB-5]` | Annex C | An instrumented fact is valid only for the foreign library, header, … |
| `[TCB-6]` | Annex C | A change to any identity input makes the record stale; a change to an … |

## THR

| Rule | Part | Begins |
|---|---|---|
| `[THR-1]` | XI | *(changed in 0.9.15)* A value several threads own is made … |
| `[THR-2]` | XI | *(changed in 0.9.15)* A one-thread value never reaches another … |
| `[THR-3]` | XI | *(changed in 0.9.15)* `Mutex[T]` is a value type. `m.lock()` blocks … |
| `[THR-4]` | XI | In the `debug` profile the runtime records the order in which each … |
| `[THR-5]` | XI | *(changed in 0.9.15)* `thread.scope()` returns a `Scope`, which MUST … |
| `[THR-6]` | XI | *(changed in 0.9.15)* A type whose `drop` a safety guarantee depends … |
| `[THR-7]` | XI | *(changed in 0.9.15)* `thread_shared` means exactly: the value may … |
| `[THR-8]` | XI | *(changed in 0.9.10)* *(changed in 0.9.15)* A type is `Send` when a … |
| `[THR-9]` | XI | *(changed in 0.9.15)* A type is `ThreadShared` when several threads … |
| `[THR-10]` | XI | *(changed in 0.9.10)* *(changed in 0.9.15)* `thread.spawn` runs a … |
| `[THR-11]` | XI | *(changed in 0.9.15)* `scope.spawn` takes a plain function, `[R]` … |
| `[THR-12]` | XI | *(changed in 0.9.15)* Tasks that must run in sequence and share data … |
| `[THR-13]` | XI | *(changed in 0.9.15)* The data-race guarantee. In Safe Ember every … |
| `[THR-14]` | XI | `Atomic[T]` exists for the integer types, `bool` and raw pointers. … |
| `[THR-15]` | XI | *(changed in 0.9.15)* `channel[T](capacity=n) -> (Sender[T], … |
| `[THR-16]` | XI | When `main` returns, or `process.exit` is called, the process ends: … |
| `[THR-17]` | XI | *(new in 0.9.15)* What an unscoped thread holds, and the loop's own … |
| `[THR-18]` | XI | *(new in 0.9.15)* `Lock` and `guarded by`. A `Lock` is a lock with no … |
| `[THR-19]` | XI | *(new in 0.9.15)* A wait for a lock this thread holds is refused when … |
| `[THR-20]` | XI | *(new in 0.9.15)* Module variables and threads. A `static mut` … |
| `[THR-21]` | XI | *(new in 0.9.15)* Reductions. A task's `total += e` (and `-=`, `*=`, … |

## TIER

| Rule | Part | Begins |
|---|---|---|
| `[TIER-1]` | I | *(changed in 0.9.15)* Anything that can break a safety promise is … |

## TOOL

| Rule | Part | Begins |
|---|---|---|
| `[TOOL-1]` | XVII | *(changed in 0.9.10)* Each release publishes, on the GitHub Releases … |
| `[TOOL-2]` | XVII | *(changed in 0.9.10)* Ember's toolchain archive (`[TOOL-1]`) carries … |
| `[TOOL-3]` | XVII | When no C compiler is found, `E9002`'s message says so and its help … |
| `[TOOL-4]` | XVII | `ember --version` prints the compiler version, the language version, … |

## TST

| Rule | Part | Begins |
|---|---|---|
| `[TST-0]` | XVII | Test annotations are line comments beginning `#$`, read from the raw … |
| `[TST-1]` | XVII | `#$ error[E…]: text`, `#$ warning[…]` and `#$ note` assert a … |
| `[TST-2]` | XVII | `#$ stdout:`, `#$ exit: N`, and `#$ assert-c: contains("…")` check … |
| `[TST-3]` | XVII | `@test` functions run in the test binary, each isolated; … |
| `[TST-4]` | XVII | Every rule of this document has a directory … |
| `[TST-4a]` | XVII | Each rule's directory has an accept case and, for each diagnostic … |
| `[TST-4b]` | XVII | Which rules need a reject case is decided mechanically: a rule that … |
| `[TST-5]` | XVII | A scripted debugger session (breakpoint by Ember line, stepping, … |
| `[TST-6]` | XVII | *(changed in 0.9.15)* Annex A's code is generated from a fixture that … |
| `[TST-7]` | XVII | Every ` ```ember ` block in this document is extracted and must pass … |
| `[TST-8]` | XVII | `tests/firstweek/` holds at least 24 first-draft programs a newcomer … |
| `[TST-9]` | XVII | *(changed in 0.9.10)* Each is marked `accepted` or … |
| `[TST-10]` | XVII | *(changed in 0.9.10)* The acceptance rate is published with each … |
| `[TST-11]` | XVII | *(changed in 0.9.15)* Besides the per-rule cases, the suite covers … |
| `[TST-27]` | XVII | The C gate. Every accepted program in the test suite is compiled … |
| `[TST-28]` | XVII | *(changed in 0.9.10)* The performance gate. `tests/perf/` holds … |
| `[TST-29]` | XVII | Honest baselines. A gate with a baseline of known failures reports … |
| `[TST-30]` | XVII | The test runner reports every failure of a run, not only the first, … |
| `[TST-31]` | XVII | *(new in 0.9.15)* The count limits are tested with a real counter … |

## TXT

| Rule | Part | Begins |
|---|---|---|
| `[TXT-1]` | XV | *(changed in 0.9.15)* `StrView` is a read-only view of bytes, a … |
| `[TXT-2]` | XV | *(changed in 0.9.15)* Nothing becomes a `StrView` without validation: … |
| `[TXT-3]` | XV | *(changed in 0.9.15)* `StrView` and `String` are a pointer and a … |
| `[TXT-4]` | XV | *(changed in 0.9.15)* Slicing a string, `s[a..b]`, is by byte offset … |
| `[TXT-5]` | XV | *(changed in 0.9.15)* `StrView` → `String` allocates and copies, and … |
| `[TXT-6]` | XV | *(changed in 0.9.15)* Across the C ABI a `StrView` is `{const uint8_t … |
| `[TXT-7]` | XV | Text is UTF-8 everywhere. `std.ffi.WideString` converts, explicitly, … |
| `[TXT-8]` | XV | *(changed in 0.9.15)* `StrView` is a view, like every `Span`: it … |
| `[TXT-9]` | XV | *(changed in 0.9.15)* A string literal initialises a `String`. … |
| `[TXT-10]` | XV | *(changed in 0.9.10)* *(changed in 0.9.15)* `StrView` operations. … |
| `[TXT-11]` | XV | *(changed in 0.9.15)* `String` operations. Everything `StrView` has … |

## TYP

| Rule | Part | Begins |
|---|---|---|
| `[TYP-1]` | IV | *(changed in 0.9.15)* Every concrete type has, at compile time, a … |
| `[TYP-2]` | IV | Producing a `bool` other than 0 or 1 is undefined behaviour and … |
| `[TYP-3]` | IV | Producing a `char` outside the Unicode scalar values is undefined … |
| `[TYP-4]` | IV | *(changed in 0.9.10)* No value converts implicitly between scalar … |
| `[TYP-5]` | IV | *(changed in 0.9.10)* *(changed in 0.9.15)* Coercions — the complete … |
| `[TYP-6]` | IV | *(changed in 0.9.15)* `x as T` converts explicitly: * integer → … |
| `[TYP-7]` | IV | `as` between pointer types, or between a pointer and an integer, … |
| `[TYP-8]` | IV | Integer overflow panics in every profile. An arithmetic operation (`+ … |
| `[TYP-9]` | IV | *(changed in 0.9.15)* Floating point is strict IEEE 754: no … |
| `[TYP-9a]` | IV | Contraction is off by default and the implementation MUST turn it off … |
| `[TYP-9b]` | IV | *(changed in 0.9.15)* `@fp(allow_fma)` permits, and requires the … |
| `[TYP-9c]` | IV | A toolchain that cannot honour `@fastmath` or `@fp(…)` for one … |
| `[TYP-10]` | IV | Shifts. `a << n` and `a >> n` accept any integer type for `n`. If `n` … |
| `[TYP-11]` | IV | A struct's default layout is C's: declaration order, natural … |
| `[TYP-12]` | IV | A unit-only enum is an integer (`@repr(u8)` and friends choose it; … |
| `[TYP-13]` | IV | *(changed in 0.9.15)* `Option[T]` has the size of `T` when `T` has a … |
| `[TYP-14]` | IV | *(changed in 0.9.15)* A reference, and an `owns T`, is read through … |
| `[TYP-15]` | IV | *(changed in 0.9.15)* Where views may be stored. A view may be stored … |
| `[TYP-15a]` | IV | The arena-backed containers (`ArenaArray`, `ArenaMap`, §IX.2) and the … |
| `[TYP-16]` | IV | Generic functions and types are monomorphised: each distinct … |
| `[TYP-17]` | IV | *(changed in 0.9.10)* *(changed in 0.9.15)* Type parameters are … |
| `[TYP-18]` | IV | Generic arguments are inferred from the arguments of a call. Explicit … |
| `[TYP-19]` | IV | *(changed in 0.9.15)* There is no specialisation and no higher-kinded … |
| `[TYP-20]` | IV | Coherence is per package. An implementation of interface `I` for type … |
| `[TYP-21]` | IV | *(changed in 0.9.15)* Operators desugar to these interfaces for … |
| `[TYP-22]` | IV | *(changed in 0.9.10)* *(changed in 0.9.15)* An interface is usable as … |
| `[TYP-23]` | IV | *(changed in 0.9.15)* Inference is local to a function body and … |
| `[TYP-24]` | IV | *(changed in 0.9.15)* An interface method is found through any … |
| `[TYP-25]` | IV | Arguments may be positional or named; positional arguments come … |
| `[TYP-26]` | IV | There is no overloading: two functions of one name in one scope are … |
| `[TYP-27]` | IV | `()` is the value of type `void`; `Ok(())` is the success value of … |
| `[TYP-28]` | IV | Division. `/` is true division and applies to floats. `/` with two … |
| `[TYP-29]` | IV | For floats, `a // b` is `floor(a / b)` and `a % b` is `a - b * … |
| `[TYP-30]` | IV | Powers. `a  b` with integer `a` and non-negative integer `b` is … |
| `[TYP-31]` | IV | *(changed in 0.9.10)* *(changed in 0.9.15)* Sizes and indices are … |
| `[TYP-32]` | IV | `some I` in a return type means "one concrete type, chosen by the … |
| `[TYP-34]` | IV | *(replaces the 0.9.8 `@view` requirement)* *(changed in 0.9.15)* A … |
| `[TYP-35]` | IV | *(changed in 0.9.15)* A type parameter need not appear in any field … |
| `[TYP-36]` | IV | *(changed in 0.9.10)* *(changed in 0.9.15)* Which types implement … |
| `[TYP-37]` | IV | Floats implement `Eq` with IEEE `==` (so `NaN != NaN`) and `Ord` with … |
| `[TYP-38]` | VI | *(changed in 0.9.15)* Collection literals. * A list literal `[a, b, … |
| `[TYP-39]` | IV | *(changed in 0.9.10)* *(changed in 0.9.15)* Collections, tuples and … |
| `[TYP-40]` | IV | Interfaces are nominal. A type implements an interface only through … |
| `[TYP-41]` | IV | Const arguments. A `const` generic argument, like an array length, is … |
| `[TYP-42]` | IV | *(new in 0.9.10)* The 256-bit counts. `u256` and `i256` are what the … |
| `[TYP-43]` | IV | *(new in 0.9.10)* *(changed in 0.9.15)* The marker interfaces are … |
| `[TYP-44]` | IV | *(new in 0.9.15)* Kind tests and casts. A value of an interface type … |
| `[TYP-45]` | IV | *(new in 0.9.15)* Who owns a value, in its type. A type says how its … |
| `[TYP-46]` | IV | *(new in 0.9.15)* Which memory manager. `using A` after a type whose … |
| `[TYP-47]` | IV | *(new in 0.9.15)* Any number of types, and of values (verdicts 115 … |

## UNS

| Rule | Part | Begins |
|---|---|---|
| `[UNS-1]` | IX | *(changed in 0.9.15)* An `unsafe` context is required to: … |
| `[UNS-2]` | IX | *(changed in 0.9.15)* `unsafe` permits exactly those operations. It … |
| `[UNS-3]` | IX | *(changed in 0.9.15)* `L3010` reports an `unsafe` block containing … |
| `[UNS-4]` | IX | *(changed in 0.9.15)* Unsafe code MUST uphold what safe code assumes: … |
| `[UNS-5]` | IX | `std.mem` provides `Volatile[*T]` (volatile reads and writes), … |
| `[UNS-6]` | IX | Inline assembly is `unsafe asm("…", …)` on toolchains that support it … |
| `[UNS-7]` | IX | *(changed in 0.9.15)* Every `unsafe fn` carries `@safety("…")` … |
| `[UNS-8]` | IX | *(changed in 0.9.15)* Every `unsafe:` block, and every `unsafe` … |
| `[UNS-10]` | IX | *(changed in 0.9.15)* `UnsafeCell[T]` (in `std.mem`) is the primitive … |
| `[UNS-10a]` | IX | *(changed in 0.9.15)* `UnsafeCell` suspends no rule globally: the … |
| `[UNS-11]` | IX | *(new in 0.9.15)* Only inside `unsafe`. Anything that can break a … |
| `[UNS-12]` | IX | *(new in 0.9.15)* An `unsafe fn` is unsafe as a whole (verdict 118: … |

## VER

| Rule | Part | Begins |
|---|---|---|
| `[VER-1]` | 0 | Three version numbers exist and move independently: the language … |
| `[VER-2]` | 0 | From 1.0: source compatibility within a major language version; a … |
| `[VER-3]` | 0 | *(changed in 0.9.10)* From 1.0: deprecation through … |
| `[VER-4]` | 0 | The runtime ABI (object header, `ember_type_info`, every entry point … |
| `[VER-5]` | XVII | *(changed in 0.9.10)* Package versions are semantic. A dependency `{ … |
| `[VER-8]` | 0 | Before 1.0 there is exactly one language: the current one. A source … |
| `[VER-9]` | 0 | A language revision that changes the set of accepted programs or … |

## WK

| Rule | Part | Begins |
|---|---|---|
| `[WK-1]` | VIII | *(changed in 0.9.15)* Ember has no cycle collector: a loop of owners, … |
| `[WK-2]` | VIII | *(changed in 0.9.15)* An object is ended when its count of owners … |
| `[WK-3]` | VIII | *(changed in 0.9.15)* An object is ended the moment its last owner … |
| `[WK-4]` | VIII | *(changed in 0.9.15)* The report names, for each leaked object on a … |
| `[WK-5]` | VIII | *(changed in 0.9.15)* The graph of owners. The compiler builds a … |
| `[WK-6]` | VIII | *(changed in 0.9.15)* A loop of owners is refused unless it is proved … |
| `[WK-7]` | VIII | *(changed in 0.9.15)* What counts as proof. Proved: a loop made and … |
| `[WK-8]` | VIII | *(changed in 0.9.15)* The run-time report (`[WK-15]`) lists each … |
| `[WK-9]` | VIII | *(changed in 0.9.15)* `ember explain --cycle <path> <Class[.field]>` … |
| `[WK-11]` | VIII | *(changed in 0.9.15)* `weakref` links. `weakref T` is a link that … |
| `[WK-12]` | VIII | *(changed in 0.9.15)* Opening a link. A `weakref` link is read by … |
| `[WK-13]` | VIII | *(changed in 0.9.15)* A `weakref` link to a value held `shares` names … |
| `[WK-14]` | VIII | *(changed in 0.9.15)* Ember's `weakref` and `shares` never convert to … |
| `[WK-15]` | VIII | *(changed in 0.9.15)* `ember run` and `ember test` in the `debug` … |
| `[WK-16]` | VIII | *(new in 0.9.15)* A `weakref` of a part (a field, or an element … |
| `[WK-17]` | VIII | *(new in 0.9.15)* Parent links. A parent link is an ordinary … |
| `[WK-18]` | VIII | *(new in 0.9.15)* A walk across a call that may change nodes holds a … |
