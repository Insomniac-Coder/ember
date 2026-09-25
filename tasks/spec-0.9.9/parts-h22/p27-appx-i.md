---

# Appendix I — Rule Index

Every rule of this document, by family, with the Part that defines it.

## ABI

| Rule | Part | Begins |
|---|---|---|
| `[ABI-1]` | XVII | The runtime ABI, the hot-reload protocol, the module protocol and the … |
| `[ABI-2]` | XVII | A protocol that cannot be checked at link time (hot-reload images, … |
| `[ABI-3]` | XVII | `ember_module_init` checks the runtime ABI version before anything … |
| `[ABI-4]` | XVII | A release notes every protocol whose version changed, and why. |
| `[ABI-5]` | XVII | The protocol versions are independent of the language version: two … |

## ALC

| Rule | Part | Begins |
|---|---|---|
| `[ALC-1]` | IX | `Array[T, A: Allocator = Global]`, `Map`, `Set` and `Box` accept an … |
| `[ALC-2]` | IX | Implementing `Allocator` is `unsafe`: the implementer promises valid, … |
| `[ALC-3]` | IX | A binary package may replace the global allocator with `[build] … |
| `[ALC-4]` | IX | Allocation failure in a standard container panics (`out of memory`). … |

## ARN

| Rule | Part | Begins |
|---|---|---|
| `[ARN-1]` | IX | `Arena` is a move-only struct. Its `alloc*` methods take `self` (a … |
| `[ARN-2]` | IX | Values in an arena are never dropped individually. Allocating a type … |
| `[ARN-3]` | IX | `alloc(v) -> ref mut T` moves `v` in. `alloc_array[T](n) -> … |
| `[ARN-4]` | IX | A growing `Arena` carries `Alloc` on its growth path. `FixedArena` … |
| `[ARN-5]` | IX | `ArenaArray[T]` and `ArenaMap[K, V]` are fixed-capacity containers … |
| `[ARN-5c]` | IX | `ArenaArray[T]` provides `len`, `capacity`, `is_empty`, `get(i) -> … |
| `[ARN-5d]` | IX | `ArenaMap[K, V]` (with `K: Eq + Hash`) provides `len`, `capacity`, … |
| `[ARN-5g]` | IX | No operation of either container touches the arena after … |
| `[ARN-6]` | IX | `arena.scope() -> ScopedArena` takes a mutable borrow of the arena … |
| `[ARN-7]` | IX | No operation lowers an arena's allocation pointer or reuses its bytes … |
| `[ARN-8]` | IX | `MaybeUninit[T]` has the size and alignment of `T` and does not claim … |
| `[ARN-10]` | IX | A panic inside `T.default()` during `alloc_array` aborts (`[PAN-1]`); … |
| `[ARN-11]` | IX | `Zeroable` is an `unsafe` marker: all-zero bytes are a valid `T`. The … |

## ATT

| Rule | Part | Begins |
|---|---|---|
| `[ATT-1]` | V | An attribute that is neither in the table below nor a visible … |
| `[ATT-2]` | V | A statement may carry only `@parallel`, `@unroll`, `@simd` and … |
| `[ATT-3]` | III | A statement attribute attaches to the next compound statement, never … |
| `[ATT-4]` | V | One attribute per line. |
| `[ATT-6]` | V | Every attribute in the table has exactly the effect its rule gives. … |

## BCK

| Rule | Part | Begins |
|---|---|---|
| `[BCK-1]` | XVIII | Loans. Each borrow expression at a point creates a loan of a place, … |
| `[BCK-2]` | XVIII | Live loans, location-sensitive. A loan is live at a point when some … |
| `[BCK-3]` | XVIII | Conflicts. At each point, an access to a place is checked against the … |
| `[BCK-4]` | XVIII | Storage end. A place whose storage ends (scope exit, `StorageDead`) … |
| `[BCK-5]` | XVIII | Two-phase borrows. The mutable loan made for a method's `mut self` … |
| `[BCK-6]` | XVIII | Required acceptances. Because liveness is location-sensitive, these … |
| `[BCK-7]` | XVIII | Class handles. An access through a class handle is not a loan of the … |

## BEN

| Rule | Part | Begins |
|---|---|---|
| `[BEN-1]` | XVII | Each benchmark runs at least 3 untimed and 30 timed repetitions on … |
| `[BEN-2]` | XVII | A gate fails only when the lower bound of the ratio's interval … |
| `[BEN-3]` | XVII | Each run also measures the C program against a second copy of itself; … |
| `[BEN-4]` | XVII | Each benchmark records retired instructions; a change beyond ± 0.5 % … |
| `[BEN-5]` | XVII | Both sides are built with the same optimisation level, LTO setting, … |
| `[BEN-6]` | XVII | Thresholds: scalar and tight loops ≤ 1.05× C; SoA and SIMD code ≤ … |
| `[BEN-8]` | Annex B | The end-to-end reload benchmark times three edits — a function body, … |

## BLD

| Rule | Part | Begins |
|---|---|---|
| `[BLD-1]` | XVII | The module (one file) is the unit of front-end caching; the package … |
| `[BLD-2]` | XVII | A module's front-end result is keyed by its source, the compiler and … |
| `[BLD-4]` | XVII | The C compiler and linker run through a generated Ninja file, so C … |
| `[BLD-5]` | XVII | Output goes to `target/<profile>/{bin,lib,c,obj,bind,inspect}`. |
| `[BLD-6]` | XVII | `lto = "off" \| "thin" \| "on"`. A value the C toolchain does not … |
| `[BLD-7]` | XVII | Front-end stages run in parallel across modules by default (`-j`, … |
| `[BLD-8]` | XVII | Within a module, editing one function body re-checks that function … |
| `[BLD-9]` | XVII | `--timings` writes a per-stage, per-module timing report … |
| `[BLD-10]` | XVII | The compile-time budgets below are release gates. |
| `[BLD-11]` | XVII | A package that omits a `[STD-6]` layer cannot use it or depend on a … |
| `[BLD-13]` | XVII | Builds are reproducible: the same inputs give a byte-identical … |

## BLD-FFI

| Rule | Part | Begins |
|---|---|---|
| `[BLD-FFI-1]` | XVI | The manifest's `[c]` section names the C compiler and flags used for … |
| `[BLD-FFI-1a]` | XVI | Each `import c` is parsed and its shim compiled with the package's C … |
| `[BLD-FFI-1b]` | Annex C | The MSVC runtime-library switch, `_DEBUG` and `_ITERATOR_DEBUG_LEVEL` … |
| `[BLD-FFI-2]` | XVI | The toolchain ships a CMake module: `ember_add_library(name KIND … |
| `[BLD-FFI-3]` | XVI | Existing libraries are linked by name (`link = ["vulkan-1"]`) or … |
| `[BLD-FFI-4]` | Annex C | Ember's emitted C and the project's C++ are compiled by the same … |
| `[BLD-FFI-5]` | Annex C | `ember build --emit header` with `--cpp` also writes a C++ header … |
| `[BLD-FFI-5a]` | Annex C | The C++ header gives a copyable smart handle only to `@sync` classes, … |

## BRW

| Rule | Part | Begins |
|---|---|---|
| `[BRW-1]` | VII | Aliasing xor mutation. At every program point a place has any number … |
| `[BRW-2]` | VII | Liveness. A borrow is live from its creation to the last use of … |
| `[BRW-3]` | VII | Two-phase borrows. For a method call whose receiver is a place, or an … |
| `[BRW-4]` | VII | Disjoint fields. `ref mut a.x` and `ref mut a.y` may be live together … |
| `[BRW-5]` | VII | Indices are not disjoint. `ref mut a[i]` and `ref mut a[j]` conflict … |
| `[BRW-6]` | VII | Reborrows. From `r: ref mut T`, `ref r.f` freezes `r` while it lives … |
| `[BRW-7]` | VII | Borrowing a moved or uninitialised place is `E3050`. |
| `[BRW-8]` | VII | A borrowed parameter is passed by address: the callee reads the … |
| `[BRW-9]` | VII | A reference in Safe code is never null, dangling or unaligned. |
| `[BRW-10]` | VII | Methods borrow the fields they use. A call to a method of a struct or … |
| `[BRW-11]` | VII | All borrows a call makes are live together. The borrows formed for … |

## BUD

| Rule | Part | Begins |
|---|---|---|
| `[BUD-1]` | XVII | They are measured on a recorded reference machine (an 8-core laptop … |
| `[BUD-2]` | XVII | Each figure is the median of 15 runs after 3 warm-up runs: |
| `[BUD-3]` | XVII | A figure above its budget fails CI, and so does one more than 15 % … |
| `[BUD-3a]` | XVII | CI normalises its measurements by a fixed calibration workload, and a … |
| `[BUD-3b]` | XVII | A gate applies only to a figure whose measured spread is below the … |
| `[BUD-5]` | XVII | A proposed language or compiler feature states its measured effect on … |

## CELL

| Rule | Part | Begins |
|---|---|---|
| `[CELL-1]` | IX | `Cell[T]` holds a `T` that can be replaced through a shared borrow: … |
| `[CELL-2]` | IX | `Cell` never hands out a reference to its contents, so it needs no … |
| `[CELL-3]` | IX | `Cell[T]` is not `Sync`; it is `Send` if `T` is. |
| `[CELL-4]` | IX | `Cell[T]` is `Copy` when `T` is. |
| `[CELL-5]` | IX | `RefCell[T]` keeps a one-word borrow counter beside `T`. `borrow() -> … |
| `[CELL-6]` | IX | `try_borrow()` and `try_borrow_mut()` return `None` on conflict, in … |
| `[CELL-7]` | IX | `Ref[T]` and `RefMut[T]` are views of the cell; dropping one ends its … |
| `[CELL-8]` | IX | `RefCell[T]` is not `Sync`; `Mutex[T]` and `RwLock[T]` are its … |
| `[CELL-9]` | IX | The `RefCell` check exists in every profile. |
| `[CELL-10]` | IX | A borrow diagnostic suggests `RefCell` only after the structural … |
| `[CELL-12]` | IX | `RefCell[T]` is never `Copy`. |

## CG-C

| Rule | Part | Begins |
|---|---|---|
| `[CG-C-1]` | XVIII | The emitted C has no undefined behaviour. Checked signed arithmetic … |
| `[CG-C-2]` | XVIII | Accepted programs compile. A program Ember accepts never produces C … |
| `[CG-C-3]` | XVIII | Cross-module inlining. Because each module is one translation unit, … |
| `[CG-C-3a]` | XVIII | `@inline` is binding: the function is emitted with `__forceinline` or … |
| `[CG-C-3b]` | XVIII | The bodies it exports are part of the interface hash (`[BLD-2]`). |
| `[CG-C-4]` | XVIII | Aliasing facts. For each loop, the base pointer of every view whose … |
| `[CG-C-5]` | XVIII | Every panic function is `_Noreturn` and cold, and each check branches … |
| `[CG-C-6]` | XVIII | A loop in vectorisable form (`[SIMD-5]`) is preceded by the host … |
| `[CG-C-7]` | XVIII | Locals keep their Ember names in the C (transliterated per … |
| `[CG-C-8]` | XVIII | Every emitted statement is preceded by a `#line` naming the Ember … |
| `[CG-C-9]` | XVIII | The build emits debugger visualisers (`.natvis`, GDB and LLDB … |
| `[CG-C-10]` | XVIII | Stack traces print Ember function paths and `file:line:col`; `ember … |
| `[CG-C-11]` | XVIII | Floating-point flags. Every translation unit begins with `#pragma … |

## CLI

| Rule | Part | Begins |
|---|---|---|
| `[CLI-1]` | XVII | Every command accepts `--json` and exits non-zero on error. |
| `[CLI-2]` | XVII | `ember build --emit c --out-dir <dir>` writes the C sources without … |
| `[CLI-4]` | XVII | `ember run file.em` and `ember build file.em` accept a single file … |
| `[CLI-5]` | XVII | `ember bind --init <header>` writes a starter overlay: every derived … |
| `[CLI-6]` | XVII | `ember bind --report` lists every skipped declaration and every … |
| `[CLI-7]` | XVII | `ember bind --check <overlay>` checks `[FFI-12]` and prints how many … |
| `[CLI-9]` | XVII | `ember check --syntax-only` lexes and parses only, reporting `E00xx` … |
| `[CLI-10]` | XVII | `ember build --report=engine` is a report, not a profile: it changes … |
| `[CLI-11]` | XVII | `ember audit` prints a one-page summary of a package's safety … |
| `[CLI-12]` | XVII | `ember why --alloc <item>` (and `--block`, `--io`, `--lock`, … |
| `[CLI-13]` | XVII | `ember calls --foreign <item>` lists every foreign function the item … |
| `[CLI-15]` | XVII | `ember --help` lists every command and flag of this section; one … |
| `[CLI-18]` | VIII | For both commands `<path>` is a package directory (its manifest and … |
| `[CLI-19]` | XVII | The implementation matrix. `ember --version --matrix` prints, for … |
| `[CLI-20]` | XVII | `ember fmt --migrate` rewrites source written for 0.9.8 into 0.9.9 … |
| `[CLI-21]` | XVII | `ember run --interp` runs a program on the compile-time evaluator … |

## CLO

| Rule | Part | Begins |
|---|---|---|
| `[CLO-1]` | VI | A lambda or local function has a unique anonymous type implementing … |
| `[CLO-2]` | VI | Captures are inferred per variable: read only ⇒ shared borrow; … |
| `[CLO-3]` | VI | What `fn(A) -> R` means depends on where it is written. * As a … |
| `[CLO-4]` | VI | A non-`owned` lambda cannot outlive what it borrows: storing it, … |
| `[CLO-5]` | VI | A closure capturing a class handle holds a strong reference; the … |
| `[CLO-6]` | VI | A lambda that moves one of its captures out of itself (into an … |
| `[CLO-6a]` | VI | An owned `once fn` value, including one inside a `Box` or a … |
| `[CLO-7]` | VI | Standard-library APIs that store or send a callback (`thread.spawn`, … |
| `[CLO-10]` | VI | An owned callable value holds up to three pointer-sized words of … |
| `[CLO-11]` | VI | A call `recv.name(args)` where `recv`'s type has no method `name` but … |
| `[CLO-12]` | VI | A local function (`[GRM-28]`) is a named closure: it captures like a … |
| `[CLO-13]` | VI | A read-only capture of a `Copy` variable whose storage would end … |
| `[CLO-14]` | VI | A callable type may also be written as an explicit generic bound, `fn … |
| `[CLO-15]` | VI | A lambda written directly as the argument of a consumed callable … |

## CLS

| Rule | Part | Begins |
|---|---|---|
| `[CLS-1]` | V | A class instance lives on the heap with the header of `[OBJ-1]`. … |
| `[CLS-2]` | V | `fn init(self, …)` is the constructor. Before any `init` body runs, … |
| `[CLS-3]` | V | A class with no `init` and no base class gets a memberwise … |
| `[CLS-4]` | V | A class is final unless declared `open` or `abstract`. Methods are … |
| `[CLS-5]` | V | Interface calls on a class are dispatched statically unless the … |
| `[CLS-6]` | V | Destruction runs the derived `drop`, then the base's, then drops the … |
| `[CLS-7]` | V | Inside a class method, `self` is a handle. Any method may read and … |
| `[CLS-7a]` | V | Inside `drop`, `self` MUST NOT be stored anywhere that outlives the … |
| `[CLS-8]` | V | A class is `Sync` only as `[THR-1]` allows; every field of a `Sync` … |
| `[CLS-9]` | V | A `let` field is assignable only in `init`. |
| `[CLS-9a]` | V | A `let` field of a non-`Copy` type may still be mutated *through* … |
| `[CLS-10]` | V | A derived class with no `init` gets its base's constructor: … |
| `[CLS-11]` | V | Construction is two-phase. In a derived `init`, the code before … |

## CONF

| Rule | Part | Begins |
|---|---|---|
| `[CONF-1]` | XVII | A compiler declares the profile it implements, and claims it only … |
| `[CONF-2]` | XVII | Ember Core: Parts II–VII, X, XIII and XV's core and alloc layers. |
| `[CONF-3]` | XVII | Ember Systems: adds Parts VIII, IX, XI, XII and XIV. |
| `[CONF-4]` | XVII | Ember Native: adds Part XVI. Annex C (C++) is a separate, optional … |
| `[CONF-5]` | XVII | Ember Dynamic: adds Annex B (hot reload). |

## CORO

| Rule | Part | Begins |
|---|---|---|
| `[CORO-1]` | VI | A `gen fn` declares a generator. Calling it runs none of its body; it … |
| `[CORO-2]` | VI | `yield e` suspends the generator and produces `e`. `yield` outside a … |
| `[CORO-3]` | VI | `Generator[Y, R]` in a signature names the function's own frame type … |
| `[CORO-4]` | VI | The compiler rewrites the body into a state machine over the frame: … |
| `[CORO-5]` | VI | A generator allocates nothing. The frame's size is a compile-time … |
| `[CORO-6]` | VI | A reference or view to a local of the generator's own frame may not … |
| `[CORO-7]` | VI | Dropping a suspended generator drops exactly the locals live at its … |
| `[CORO-8]` | VI | A generator's effect set is the union over its whole body; contracts … |
| `[CORO-9]` | VI | A frame never points into itself in Safe code; `unsafe` code that … |
| `[CORO-10]` | VI | A `gen fn` may not be `extern`, `@export`ed or passed to C (`E2222`). |
| `[CORO-11]` | VI | `std.coroutine` builds gameplay sequencing on generators with no … |
| `[CORO-12]` | VI | A `gen fn` method of a class takes `self` (the frame retains the … |
| `[CORO-13]` | VI | A `yield` while a `@must_drop` value (`[THR-6]`) is live is `E2231`: … |

## COST

| Rule | Part | Begins |
|---|---|---|
| `[COST-1]` | X | Zero cost, defined. An abstraction is zero-cost *for a use* whose … |
| `[COST-2]` | X | Every implicit cost is one of: guaranteed elided (emitting it is a … |
| `[COST-3]` | X | The costs. |
| `[COST-4]` | X | `ember inspect --cost <item>` prints every row that applies to an … |
| `[COST-5]` | X | A rule that introduces an implicit cost MUST add a row to this table; … |

## CT

| Rule | Part | Begins |
|---|---|---|
| `[CT-1]` | XIV | These are evaluated during compilation: `comptime(e)`; `comptime:` … |
| `[CT-2]` | XIV | An operation that cannot run at compile time — an `extern` call, a … |
| `[CT-3]` | XIV | Each evaluation is limited to 10⁸ steps and 256 MB of evaluator heap … |
| `[CT-4]` | XIV | Compile-time evaluation is deterministic by construction: the … |
| `[CT-5]` | XIV | Where results live. A compile-time result is placed in the image as … |
| `[CT-6]` | XIV | `comptime(e)` is an expression whose value is `e` evaluated at … |
| `[CT-7]` | XIV | A `comptime:` block at item level, or as a statement, runs once … |

## CTL

| Rule | Part | Begins |
|---|---|---|
| `[CTL-0]` | VI | The condition of `if`, `elif`, `while` and a match guard MUST be a … |
| `[CTL-1]` | VI | `for pattern in e:` iterates: * a place `e` whose type is `Iterable`: … |
| `[CTL-2]` | VI | The iterated place is borrowed for the whole loop; mutating it inside … |
| `[CTL-3]` | VI | `a..b` (`Range`), `a..=b` (`RangeInclusive`) and `a..` (`RangeFrom`) … |
| `[CTL-3b]` | VI | Iteration over ranges, `Span`, `MutSpan`, `Array`, `[T; N]`, `SoA` … |
| `[CTL-4]` | VI | The `else` of `while` or `for` runs when the loop ends without … |
| `[CTL-5]` | VI | A `match` tests arms top to bottom; the first that matches runs; a … |
| `[CTL-6]` | VI | `with a = e1, b = e2:` binds `a` and `b` for the block and drops them … |
| `[CTL-7]` | VI | `defer:` registers a block to run when the enclosing block exits, … |
| `[CTL-8]` | VI | On every exit from a block, its `defer` blocks run first and then its … |
| `[CTL-9]` | VI | `pass` does nothing; an empty block is written `pass`. |
| `[CTL-10]` | VI | Names assigned in every branch. In an `if`/`elif`/`else` that has an … |

## CXX

| Rule | Part | Begins |
|---|---|---|
| `[CXX-1]` | Annex C | The conformance suite for this annex includes a corpus of real … |

## DET

| Rule | Part | Begins |
|---|---|---|
| `[DET-1]` | X | `@deterministic` on a function or module is a contract: `Nondet` MUST … |
| `[DET-2]` | X | `Nondet` is introduced by exactly: floating-point contraction, … |
| `[DET-3]` | X | A `@deterministic` function may call only `Nondet`-free functions; a … |
| `[DET-4]` | X | `std.math.det` provides `sin`, `cos`, `tan`, `exp`, `log`, `pow`, … |
| `[DET-5]` | X | Inside a `@deterministic` function the backend never contracts, … |
| `[DET-6]` | X | `@deterministic` constrains results, not timing. |
| `[DET-7]` | X | The cross-machine claim holds for one binary. `ember build … |
| `[DET-8]` | X | `@deterministic` on a module applies to every function in it, with no … |
| `[DET-9]` | X | `ember inspect --deterministic <item>` prints whether the item … |
| `[DET-10]` | X | A deterministic program runs with a defined floating-point … |

## DIA

| Rule | Part | Begins |
|---|---|---|
| `[DIA-1]` | XVII | A diagnostic has a code, one primary span, labelled secondary spans, … |
| `[DIA-2]` | XVII | Messages start lowercase, have no trailing period, name the thing, … |
| `[DIA-3]` | XVII | Ownership and borrow errors include the "later used here" label and a … |
| `[DIA-4]` | XVII | Contract errors print the whole call chain (`[EFF-6]`). |
| `[DIA-5]` | XVII | FFI errors name the header and the C declaration. |
| `[DIA-6]` | XVII | Every code has a page, `docs/errors/EXXXX.md`, with a program that … |
| `[DIA-6a]` | XVII | The code registry is exhaustive in both directions: every code this … |
| `[DIA-7]` | XVII | Every ownership and borrow error is classified into a shape of … |
| `[DIA-7a]` | XVII | Every code in `E3000`–`E3499` belongs to exactly one shape of … |
| `[DIA-8]` | XVII | `ember explain --borrow <file>:<line>` prints, for each loan live at … |
| `[DIA-9]` | XVII | A diagnostic never suggests `unsafe`, `Cell`, `RefCell`, `Shared` or … |
| `[DIA-10]` | XVII | The help of shapes B1, B2, B4, B5 and B9 names a concrete API or … |
| `[DIA-11]` | XVII | For shape S1 the diagnostic reports the check's reason (`[EFF-11]`); … |
| `[DIA-12]` | XVII | Every name and type error is classified into a shape of §XVII.6.2. An … |
| `[DIA-13]` | XVII | Every shape has a rendered snapshot under `tests/ui/` and a … |
| `[DIA-14]` | XVII | Only the first error of a cascade is reported: nothing is reported … |
| `[DIA-15]` | XVII | Suggestions are computed from data the compiler already holds (scope … |
| `[DIA-16]` | XVII | When a diagnostic suggests moving a value into a class, `Shared` or … |
| `[DIA-18]` | XVII | A foreign call rejected because its contract has unknown facts … |
| `[DIA-20]` | XVII | A run of invalid bytes or characters is one diagnostic, not one per … |
| `[DIA-21]` | XVII | Python habits. Each of these is recognised and answered with the … |
| `[DIA-22]` | XVII | A diagnostic caused by a callee's contract or signature is reported … |
| `[DIA-23]` | XVII | A run-time panic names Ember entities — the class, field, variable, … |
| `[DIA-24]` | XVII | Name suggestions (shape N1). A candidate is suggested when its … |

## DOC

| Rule | Part | Begins |
|---|---|---|
| `[DOC-1]` | XVII | Error pages ship with their errors. |
| `[DOC-2]` | XVII | The user guide (`docs/book/`), with a "coming from Python" chapter … |
| `[DOC-3]` | XVII | `ember doc` presents, for every function that produces or consumes a … |
| `[DOC-4]` | XVII | The guide, the error pages and this specification are published … |

## DRP

| Rule | Part | Begins |
|---|---|---|
| `[DRP-1]` | VII | `fn drop(mut self)` runs exactly once per value, at the end of its … |
| `[DRP-2]` | VII | Locals drop at the end of their block in reverse declaration order; … |
| `[DRP-3]` | VII | Temporaries drop at the end of the statement that created them … |
| `[DRP-4]` | VII | A panic inside `drop` aborts the process (`[PAN-1]`). A `drop` SHOULD … |
| `[DRP-5]` | VII | A `drop` body may not move fields out of `self` (`[EXP-6]`); it uses … |
| `[DRP-6]` | VII | Dropping a `Box[T]` drops the `T` and frees; dropping a class handle … |
| `[DRP-7]` | VII | A value whose `drop` may read through a reference it holds must be … |

## DRV

| Rule | Part | Begins |
|---|---|---|
| `[DRV-1]` | XIV | `@derive(…)` requests generated implementations. The built-in … |
| `[DRV-2]` | XIV | User-defined derives are not in this version; `macro` is reserved for … |

## DSJ

| Rule | Part | Begins |
|---|---|---|
| `[DSJ-1]` | IX | `assert_disjoint(a, b) -> Result[(A, B), (A, B)]` compares the two … |
| `[DSJ-2]` | IX | The fact belongs to the returned values, not to a program point; … |
| `[DSJ-3]` | IX | The borrow checker treats the returned views as non-overlapping, and … |
| `[DSJ-4]` | IX | It applies to `Span`, `MutSpan`, `SoA` columns and arena views; two … |
| `[DSJ-5]` | IX | `assert_disjoint_or_panic(a, b) -> (A, B)` panics on overlap. Both … |
| `[DSJ-6]` | IX | `assert_disjoint` is usable in `@noalloc`, `@nosync` and … |
| `[DSJ-7]` | IX | `unsafe assume_disjoint(a, b) -> (A, B)` asserts without checking; … |
| `[DSJ-8]` | IX | There is no form that checks in one profile and assumes in another … |
| `[DSJ-9]` | IX | `assert_disjoint_all(v1, …, vn)` handles 2 to 8 views pairwise. |

## DSP

| Rule | Part | Begins |
|---|---|---|
| `[DSP-1]` | VIII | A call is dispatched statically when the receiver's static type is a … |
| `[DSP-2]` | VIII | A virtual call loads its slot from the object's type table; slots are … |
| `[DSP-3]` | VIII | A call through an interface-typed handle finds the interface's table … |
| `[DSP-4]` | VIII | `h as? D` walks the base chain; `a is b` compares addresses. |
| `[DSP-5]` | VIII | With the whole program visible, a virtual call with exactly one … |

## ECS

| Rule | Part | Begins |
|---|---|---|
| `[ECS-1]` | XII | `Entity` is a 32-bit generational handle: a 20-bit index and a 12-bit … |
| `[ECS-2]` | XII | Each component type is stored as a sparse set: a dense `Array[T]` or … |
| `[ECS-3]` | XII | `Query[(A, Mut[B], Option[C], Not[D])]` is a view over a world's … |
| `[ECS-4]` | XII | A query's read and write sets are compile-time constants computed … |
| `[ECS-5]` | XII | Adding, removing and destroying during iteration go through a … |
| `[ECS-6]` | XII | Iteration order is insertion order with swap-remove holes: … |
| `[ECS-7]` | XII | `@derive(Component)` registers the type in a compile-time component … |

## EFF

| Rule | Part | Begins |
|---|---|---|
| `[EFF-1]` | X | Effects are computed from each function's body and its callees', over … |
| `[EFF-2]` | X | A call through a callable parameter (a generic bound, `[CLO-3]`) … |
| `[EFF-3]` | X | An `extern` function carries the effects its contract declares … |
| `[EFF-4]` | X | Effects are part of a function's interface for incremental builds: a … |
| `[EFF-5]` | X | Each contract in the table forbids its effect in the function's … |
| `[EFF-6]` | X | A contract is checked over the whole reachable call graph — callees, … |
| `[EFF-6a]` | X | A `@static_safe` function may not take a parameter whose type forces … |
| `[EFF-7]` | X | `unsafe: @assume_noalloc(expr)` overrides the analysis for one call … |
| `[EFF-8]` | X | An `override` inherits its base method's contracts. |
| `[EFF-9]` | X | `RuntimeCheck(k)` enters a function's effect set when a check of kind … |
| `[EFF-10]` | X | The effect set records only which kinds occur. Per-site detail — … |
| `[EFF-11]` | X | Every emitted check carries one reason, and diagnostics use it: |
| `[EFF-12]` | X | `@static_safe` permits `RuntimeCheck(Bounds)`, `(Arithmetic)` and … |
| `[EFF-13]` | X | A long-term access through a class handle loaded from memory is … |
| `[EFF-14]` | X | Contracts compose; the usual inner-loop set is `@static_safe @noalloc … |
| `[EFF-15]` | X | Effects and contract verdicts are computed once, after the … |
| `[EFF-16]` | X | `@nopanic(explicit)` forbids only the panics the programmer writes … |
| `[EFF-17]` | X | `@nopanic(explicit)` is spelled with its argument; bare `@nopanic` is … |
| `[EFF-18]` | X | Effects are independent: a blocking `Mutex.lock` has `Sync + Lock + … |
| `[EFF-19]` | X | `@realtime` names the contract set declared by the manifest of the … |
| `[EFF-20]` | X | `@noio` forbids `Io`, including console output; `println` in a … |
| `[EFF-21]` | X | `@nolock` forbids `Lock`, including `try_lock`, which never blocks … |
| `[EFF-23]` | X | A contract attribute whose checking an implementation has not built … |

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
| `[ERR-2]` | XIII | `e?` on a `Result[T, E]` in a function returning `Result[U, F]` … |
| `[ERR-3]` | XIII | `Error` is the interface of error types: `Display + Debug` with … |
| `[ERR-4]` | XIII | `Option` and `Result` provide `is_some`/`is_none`, `is_ok`/`is_err`, … |
| `[ERR-5]` | XIII | `Result` is `@must_use`: discarding one is `W2190` (an error under … |
| `[ERR-6]` | XIII | A foreign function's status code becomes a `Result` at the binding … |
| `[ERR-7]` | XIII | Identity conversion. The prelude provides `From[T]` for every `T` … |
| `[ERR-8]` | XIII | `AnyError` is the prelude's "any error" type: an owned, boxed `dyn … |
| `[ERR-9]` | XIII | The error parameter defaults to `AnyError` (§XIII.2), so `fn … |
| `[ERR-10]` | XIII | `r.context(msg)` on a `Result[T, E]` returns a `Result[T, AnyError]` … |
| `[ERR-11]` | XIII | The cost of `AnyError`. Converting an error into `AnyError` … |
| `[ERR-12]` | XIII | An `Err` returned from `main` (`[FN-8]`) prints `error: <Display>` to … |
| `[ERR-13]` | XIII | The standard library follows one convention, and user code SHOULD: an … |

## EXC

| Rule | Part | Begins |
|---|---|---|
| `[EXC-1]` | VIII | Beginning a write access to a field while any access to the same … |
| `[EXC-2]` | VIII | Beginning a read access to a field while a write access to it is … |
| `[EXC-3]` | VIII | The compiler MAY remove a check only when it proves no conflicting … |
| `[EXC-3a]` | VIII | Every removed check is recorded, with the condition that justified … |
| `[EXC-4]` | VIII | A `let` field is subject to the same rules: `let` fixes the binding, … |
| `[EXC-5]` | VIII | Accesses nested inside the same `mut self` method are reborrows and … |
| `[EXC-6]` | VIII | A panic from `[EXC-1]`/`[EXC-2]` names both the offending access and … |
| `[EXC-7]` | VIII | The opt-in lint `L3013` reports a long-term access held across a … |
| `[EXC-8]` | VIII | When a loop makes repeated long-term accesses to one object whose … |
| `[EXC-9]` | VIII | `[EXC-8]` applies only when the receiver's identity is … |
| `[EXC-10]` | VIII | An inner loop reuses an outer loop's hoisted access when its accesses … |
| `[EXC-11]` | VIII | A hoisted access is invisible to programs: it cannot be named, stored … |
| `[EXC-12]` | VIII | `ember inspect --safety` reports each check as `STATIC`, … |
| `[EXC-13]` | XVII | The performance suite contains class-handle loops with one stable … |
| `[EXC-14]` | VIII | The 0.9.8 `exclusivity = "unchecked"` setting is removed. Code that … |
| `[EXC-15]` | VIII | A `mut self` class method holds a write access to every field of the … |
| `[EXC-16]` | VIII | Assigning a value to a class field whose type is not `Copy` is a … |
| `[EXC-17]` | VIII | Instantaneous writes are not checked against long-term accesses, so a … |
| `[EXC-18]` | VIII | The dynamic access that a borrow through a class handle begins — a … |
| `[EXC-19]` | VIII | Access state is per field. Each non-`Copy` field of a class that is … |

## EXP

| Rule | Part | Begins |
|---|---|---|
| `[EXP-1]` | VI | Operands, arguments, and the elements of tuple, list, map and set … |
| `[EXP-2]` | VI | An assignment evaluates its right side first, into a temporary if it … |
| `[EXP-3]` | VI | `and` and `or` short-circuit. `x if c else y` evaluates `c` and then … |
| `[EXP-4]` | VI | A temporary created while evaluating an expression statement is … |
| `[EXP-5]` | VI | Mutation, a mutable borrow and a move require a place; `f(x).y = 1` … |
| `[EXP-6]` | VI | A place of a non-`Copy` type used as a value (assigned, passed to … |
| `[EXP-9]` | VI | `a is b` compares two class handles (or two references) for identity. … |

## FFI

| Rule | Part | Begins |
|---|---|---|
| `[FFI-1]` | XVI | Every foreign declaration has a contract: for each pointer-typed … |
| `[FFI-2]` | XVI | A call to a foreign function whose contract contains an `unknown` … |
| `[FFI-2a]` | XVI | Derived facts need no `unsafe`: `const T*` is a read-only borrow, … |
| `[FFI-3]` | Annex C | Ember never links against C++ mangled symbols. The importer generates … |
| `[FFI-4]` | XVI | No Ember panic crosses into foreign code (`[FFI-25]`). |
| `[FFI-5]` | XVI | Layouts are verified, not assumed: the importer records `sizeof`, … |
| `[FFI-5a]` | XVI | The generated shim (`[FFI-29]`) contains a `_Static_assert` for the … |
| `[FFI-6]` | XVI | `import c "header" with (…) [as name]` parses the header with … |
| `[FFI-6a]` | XVI | An object-like macro is imported when, after expansion, it is a … |
| `[FFI-6b]` | XVI | A function-like macro is exposed only when an overlay declares its … |
| `[FFI-7]` | XVI | A header is re-parsed only when its content, its overlay, or a … |
| `[FFI-8]` | XVI | Type mapping. |
| `[FFI-9]` | XVI | `extern "C"` is the platform C ABI (SysV AMD64, Windows x64, … |
| `[FFI-10]` | XVI | An `unsafe extern "C":` block declares foreign functions, statics and … |
| `[FFI-11]` | XVI | Contract vocabulary. A pointer contract has five axes; mutability … |
| `[FFI-11a]` | XVI | A pointer contract with no count word is `E5012`, which lists the … |
| `[FFI-11b]` | XVI | The two-call enumeration idiom (call once for the count, again to … |
| `[FFI-11c]` | XVI | `TODO(count)`, `TODO(nullable)`, `TODO(ownership)` and … |
| `[FFI-11f]` | Annex C | A result lifetime derived from `[[clang::lifetimebound]]` imports as … |
| `[FFI-12]` | XVI | An overlay entry whose name, parameter count or parameter names do … |
| `[FFI-13]` | XVI | An overlay may contain `extend` blocks with ordinary Ember wrapper … |
| `[FFI-14]` | XVI | The importer's result is a `.embind` file (versioned CBOR: header … |
| `[FFI-15]` | XVI | `cstr` is a borrowed, NUL-terminated C string (`c"…"` literals are … |
| `[FFI-16]` | XVI | An enum marked `@ffi(status, ok=X)` generates `struct <E>Error(code: … |
| `[FFI-17]` | Annex C | `import cpp "Header.hpp" with (project="engine", overlay="…", … |
| `[FFI-17a]` | Annex C |  |
| `[FFI-17b]` | Annex C | Templates are available only as explicit instantiations listed in … |
| `[FFI-17c]` | Annex C | Destruction of an Ember class derived from a C++ class is … |
| `[FFI-17d]` | Annex C | `@ffi(trampoline, owner="ember")`, the default, makes the Ember count … |
| `[FFI-17e]` | Annex C | The bridge types `CppVector[T]`, `CppString` and `CppShared[T]` are … |
| `[FFI-17f]` | Annex C | `ember inspect` reports each bridge operation that is a thunk call … |
| `[FFI-20a]` | XVI | A declaration the importer cannot represent is recorded with the … |
| `[FFI-21]` | XVI | A C function-pointer parameter accepts a capture-free Ember function … |
| `[FFI-22]` | XVI | Every exported function and trampoline first attaches the calling … |
| `[FFI-23]` | XVI | `Retained.pin(v)` hands an Ember-owned object to C for keeping: for a … |
| `[FFI-24]` | Annex C | Every imported C++ function has an exception policy, with a default. … |
| `[FFI-24b]` | Annex C | An overlay may assert `@ffi(noexcept)` for a function the header … |
| `[FFI-24c]` | Annex C | `ember inspect` reports each C++ call's mode (`noexcept (derived)`, … |
| `[FFI-24d]` | Annex C | A header that adds or removes `noexcept` changes the Ember signature; … |
| `[FFI-25]` | XVI | A panic in an exported function, or anywhere below it, aborts the … |
| `[FFI-26]` | XVI | `@export("symbol")` gives a function a stable C symbol with the C … |
| `[FFI-27]` | XVI | The runtime `ember_rt` is a C11 static library with no global … |
| `[FFI-28]` | XVI | A package built as `kind = "staticlib"` produces a library and header … |
| `[FFI-29]` | XVI | For every `import c` the importer emits a shim C file, compiled with … |
| `[FFI-29a]` | XVI | A binding never names a symbol with internal linkage. |
| `[FFI-29b]` | XVI | `implementation = ["MINIAUDIO_IMPLEMENTATION"]` compiles a … |
| `[FFI-29c]` | XVI | A call to a wrapped `static inline` function costs one extra call … |
| `[FFI-30]` | XVI | Two imports of one C declaration (by Clang's USR, through typedefs) … |
| `[FFI-30a]` | XVI | An overlay changes the signatures, safety and names of *functions* … |
| `[FFI-30b]` | XVI | `pub import c "…" as vk` re-exports the module under the ordinary … |
| `[FFI-30c]` | XVI | A package may distribute an overlay for a foreign module, and several … |
| `[FFI-31]` | XVI | A `cdylib` links its own copy of the runtime. The host calls … |
| `[FFI-31a]` | XVI | Its thread detachment never relies on a TLS destructor in the … |
| `[FFI-31b]` | XVI | No owning Ember value crosses a module boundary: class handles, … |
| `[FFI-31c]` | XVI | With `[build] runtime = "shared"`, several Ember modules in one … |
| `[FFI-32]` | Annex C | A C++ class that is standard-layout and trivially copyable, all of … |
| `[FFI-33]` | XVI | `@export(threads=any \| main \| creator)` states which threads may … |
| `[FFI-33a]` | XVI | Attaching a thread (`[FFI-22]`) gives it no right to touch another … |
| `[FFI-33b]` | XVI | `returns_owned`, `Retained` and callbacks may carry … |
| `[FFI-33c]` | XVI | Under `threads = main`, the exported wrapper checks, in every … |
| `[FFI-35a]` | XVI | A parameter with no `retained` word is recorded as "does not retain" … |
| `[FFI-36]` | Annex C | A C++ function returning a raw pointer with no ownership contract … |
| `[FFI-36a]` | XVI | `std.ffi.adopt[T](h) -> ForeignBox[T]` is an `unsafe fn` that takes … |
| `[FFI-36b]` | XVI | Adopting a type whose overlay says `adopt = false` is `E5052`. … |
| `[FFI-37]` | Annex C | `ember test --instrument-ffi` runs the tests with the allocation, … |
| `[FFI-37a]` | Annex C | The intercepted set is published and recorded in the evidence; an … |
| `[FFI-37b]` | Annex C | The evidence records how often each foreign function was called and … |
| `[FFI-37c]` | Annex C | `instrumented` means "no counterexample was observed on the paths … |
| `[FFI-37d]` | Annex C | Only effect facts (`Alloc`, `Block`, `Lock`, `Io`) can be … |
| `[FFI-38]` | XVI | The importer rejects rather than guesses: a fact it cannot establish … |
| `[FFI-39]` | Annex C | An Ember class may derive from a C++ class the overlay declares with … |
| `[FFI-39a]` | Annex C | The trampoline calls the overrides through their permanent thunks, so … |
| `[FFI-39b]` | Annex C | `super.init(…)` selects the C++ base constructor by arity and … |
| `[FFI-39c]` | Annex C | Passing `self` to a C++ API upcasts it and creates a `Retained` token … |
| `[FFI-39d]` | Annex C | Re-entrant calls from C++ into the same Ember object are expected. … |
| `[FFI-39e]` | Annex C | A C++ exception never crosses into Ember code, and an Ember panic … |
| `[FFI-40]` | Annex C | `const` methods import as `self`, non-`const` methods as `mut self`; … |
| `[FFI-40a]` | Annex C | `const` is not an aliasing guarantee in C++: a `const` method that … |
| `[FFI-41]` | Annex C | Static member functions import as associated functions and static … |
| `[FFI-42]` | Annex C | Overloaded operators map to Ember operator interfaces where one … |
| `[FFI-42a]` | Annex C | Iterating an imported C++ container borrows it mutably for the loop, … |
| `[FFI-43]` | Annex C | An overlay that marks a function both `noexcept` and throwing, or two … |
| `[FFI-44]` | Annex C | *Automatic* means no overlay is needed; *overlay* means a contract … |
| `[FFI-48]` | Annex C | Unsupported constructs (`E5034`, naming the row): |
| `[FFI-49]` | XVI | `@ffi(link_name="sym")` binds a declaration to a differently named … |
| `[FFI-50]` | Annex C | The manifest section `[cpp.<project>]` names the C++ project: … |

## FMT

| Rule | Part | Begins |
|---|---|---|
| `[FMT-1]` | XVII | `ember fmt` writes LF line endings, four-space indentation and at … |
| `[FMT-2]` | XVII | The formatter uses the `=>` form of a lambda whose body is one … |
| `[FMT-3]` | XVII | The formatter never emits `;` outside `[T; N]` and `[v; N]`. |

## FN

| Rule | Part | Begins |
|---|---|---|
| `[FN-1]` | V | Parameter modes. * `a: A` — borrowed (the default). The callee reads … |
| `[FN-1a]` | V | A `mut` parameter whose type is itself a view (`MutSpan[T]`) accepts … |
| `[FN-2]` | V | An omitted mode is borrowed. There is no by-copy mode; a callee that … |
| `[FN-2a]` | V | A call site never writes a mode. `f(x)` is written whatever mode `f` … |
| `[FN-3]` | V | A function returns by move. Returning a reference or view requires … |
| `[FN-4]` | V | A method's receiver follows the same modes: `self`, `mut self`, … |
| `[FN-5]` | V | Default argument expressions are evaluated at each call, after the … |
| `[FN-6]` | V | Callable types. A function is a value. A callable type is written … |
| `[FN-7]` | V | Recursion is permitted; tail calls are not guaranteed to be … |
| `[FN-8]` | V | `main` is `fn main()`, `fn main() -> Result[void, E]` for any `E: … |
| `[FN-9]` | V | A mode on a class-handle parameter governs the handle, not the … |
| `[FN-10]` | V | A function whose return type is `Result[void, E]` returns `Ok(())` … |

## GPU

| Rule | Part | Begins |
|---|---|---|
| `[GPU-1]` | Annex D | GPU objects are named by generational handles (`[HND-1]`), which are … |
| `[GPU-2]` | Annex D | `device.begin_frame() -> Option[Frame]`; `None` is normal (a … |
| `[GPU-3]` | Annex D | `frame.arena()` is the frame's transient allocator; its region is the … |
| `[GPU-4]` | Annex D | Writing or mapping a resource that the GPU may still be reading is a … |
| `[GPU-5]` | Annex D | Binding a resource declares its access; `cmd.read(h)`/`cmd.write(h)` … |
| `[GPU-6]` | Annex D | `device.destroy(h)` invalidates the handle at once and destroys the … |
| `[GPU-7]` | Annex D | In `debug`, dropping a device while handles are still live reports … |
| `[GPU-8]` | Annex D | `History[T]` owns the per-frame copies of a temporal resource … |
| `[GPU-9]` | Annex D | `std.gpu.graph` is an optional render-graph layer over `CommandList`: … |
| `[GPU-10]` | Annex D | `ember shader-bind <reflection.json>`, reading the reflection a … |
| `[GPU-11]` | Annex D | Ember assumes no shader language: anything that produces SPIR-V and … |

## GRM

| Rule | Part | Begins |
|---|---|---|
| `[GRM-1]` | III | A class has at most one base class, written in parentheses: `class … |
| `[GRM-2]` | III | A file contains imports and items and, in the entry file only (the … |
| `[GRM-3]` | III | `Array[T]`, `Map[K, V]`, `Option[T]`, `Result[T, E]`, `Span[T]`, … |
| `[GRM-4]` | III | `x = e` where no `x` is in scope declares `x` with the type of `e`; … |
| `[GRM-5]` | III | `a, b = e` destructures a tuple, a struct or a fixed array. The right … |
| `[GRM-6]` | III | The optional `else` of `while` and `for` runs when the loop ends … |
| `[GRM-7]` | III | `defer` blocks run in reverse order at the exit of the enclosing … |
| `[GRM-8]` | III | `name[…]` in expression position is resolved during name resolution: … |
| `[GRM-8a]` | III | Inside `[ ]` in expression position the parser commits to a type … |
| `[GRM-8b]` | III | An index whose argument is a type is `E2172`; an instantiation … |
| `[GRM-8c]` | III | `IDENT = type` inside `[ ]` is an associated-type binding … |
| `[GRM-8d]` | III | A `type_alias` with an `in` clause declares a range type (`[RNG-1]`). … |
| `[GRM-10]` | III | A `match` whose arms use `pattern: block` is a statement; one whose … |
| `[GRM-11]` | III | There are no block expressions. A value computed by several … |
| `[GRM-12]` | III | An identifier in pattern position names a unit variant or a `const` … |
| `[GRM-13]` | III | Matching a place that is not consumed binds `Copy` fields by value … |
| `[GRM-15]` | III | `owned e` in expression position is legal only as the iterable of a … |
| `[GRM-16]` | III | `return`, `break` and `continue` are expressions of type `Never`; … |
| `[GRM-17]` | III | A lambda with a `:` body inside brackets holds exactly one simple … |
| `[GRM-18]` | III | `;` never separates statements. `a = 1; b = 2` is `E0105`, whose help … |
| `[GRM-19]` | III | The pattern of a `condition` MUST be refutable. An irrefutable one is … |
| `[GRM-20]` | III | The attributes before a statement are limited to `@parallel`, … |
| `[GRM-21]` | III | `gen fn` declares a generator (`[CORO-1]`). It is legal wherever `fn` … |
| `[GRM-23]` | III | `x in c` and `x not in c` are membership tests at comparison … |
| `[GRM-24]` | III | Ember has one path separator, `.`. A path resolves left to right: … |
| `[GRM-25]` | III | A chain of the comparison operators `==`, `!=`, `<`, `>`, `<=`, `>=` … |
| `[GRM-26]` | III | A `{…}` atom is a map literal if its first element is followed by … |
| `[GRM-27]` | III | A comprehension is shorthand for an iterator pipeline and has exactly … |
| `[GRM-28]` | III | A `fn_decl` inside a block declares a local function. It may capture … |
| `[GRM-29]` | III | An `expr_list` of two or more expressions, or of one expression … |
| `[GRM-30]` | III | `as?` and `as!` are single tokens (`[LEX-21]`): `h as? D` is a … |
| `[GRM-31]` | III | A `some` type (`[TYP-32]`) is legal only as a function's return type, … |
| `[GRM-32]` | III | `comptime(e)` evaluates the expression `e` at compile time (`[CT-6]`). |
| `[GRM-33]` | III | A bodiless `fn_decl` is legal only in an `interface`, an … |
| `[GRM-34]` | III | `extend [T: B] Array[T] implements I:` declares a generic extension; … |
| `[GRM-35]` | XVI | Overlay grammar. An overlay is a file whose items are: |
| `[GRM-36]` | III | `ref e` and `ref mut e` borrow the place `e` (`[BRW-1]`). The operand … |
| `[GRM-37]` | III | A directive is a line beginning `#!` before the imports. `#! language … |
| `[GRM-38]` | III | A parenthesised comprehension `(e for x in it if c)` is a generator … |

## HASH

| Rule | Part | Begins |
|---|---|---|
| `[HASH-1]` | IV | `Hash.hash` is generic over `H: Hasher` and monomorphised; hashing a … |
| `[HASH-2]` | IV | `std.collections.DefaultHasher` is a fixed-seed hasher: the same keys … |
| `[HASH-3]` | IV | `Map` and `Set` MUST NOT weaken equality to compensate for an … |
| `[HASH-4]` | IV | `Map[K, V]` requires `K: Eq + Hash`. A map may call `hash` any number … |

## HEAP

| Rule | Part | Begins |
|---|---|---|
| `[HEAP-1]` | IX | Every heap type allocates through … |
| `[HEAP-2]` | IX | Growable buffers double, from a minimum of four elements; … |
| `[HEAP-3]` | IX | `Shared(value) -> Shared[T]` allocates one counted block holding … |
| `[HEAP-4]` | IX | `s.get() -> ref T` borrows the payload: it begins a checked read … |
| `[HEAP-5]` | IX | `s.get_mut() -> ref mut T` begins a checked write access (`[EXC-1]`) … |
| `[HEAP-6]` | IX | `Shared[T]` is `Copy`: copying retains, dropping releases. |
| `[HEAP-7]` | IX | `Weak[O]` exists for `O` a class handle, a `Shared[T]` or a … |
| `[HEAP-8]` | IX | Capacity arithmetic is checked: a requested capacity whose byte size … |
| `[HEAP-9]` | IX | Heap storage for elements of type `T` is aligned to at least … |
| `[HEAP-10]` | IX | `Shared[T]` is never `Send` or `Sync`: its counts and access state … |

## HND

| Rule | Part | Begins |
|---|---|---|
| `[HND-1]` | IX | A `Handle[T]` is a plain `Copy` value (index and generation). Every … |
| `[HND-2]` | IX | `Handle[T]` is a `u64`: 32 bits of index and 32 of generation. … |
| `[HND-3]` | IX | A slot whose generation is exhausted is retired and never reused, so … |

## HR

| Rule | Part | Begins |
|---|---|---|
| `[HR-1]` | Annex B | An edit to one function body in a 50k-line reloadable package MUST be … |
| `[HR-2]` | Annex B | Reload is transactional. A failure in plan or prepare leaves the … |
| `[HR-2a]` | Annex B | Plan reads schemas and the registered set and runs no user code. … |
| `[HR-3]` | Annex B | A reload is applied only inside `ember_reload_poll()`, and only when … |
| `[HR-3a]` | Annex B | Every thread that can run Ember code is registered: host threads on … |
| `[HR-4]` | Annex B | Old images are never unloaded, so a stale return address or retired … |
| `[HR-4a]` | Annex B | Statics live in a runtime-owned table, never in image memory. |
| `[HR-5]` | Annex B | A call to a reloadable function goes through its thunk. |
| `[HR-6]` | Annex B | Each reloadable function gets one thunk, at a fixed address for the … |
| `[HR-6a]` | Annex B | Hence function values, closure code pointers, method tables, drop … |
| `[HR-7]` | Annex B | Call-table slots are identified by mangled name (`[MNG-1]`), never by … |
| `[HR-8]` | Annex B | Each image carries a reload manifest section listing every reloadable … |
| `[HR-9]` | Annex B | A call through a thunk costs one load and one indirect jump; the … |
| `[HR-9a]` | Annex B | A reloadable function is never inlined, devirtualised or placed in … |
| `[HR-10]` | Annex B | `@noreload fn` is called directly and may be inlined; changing its … |
| `[HR-10a]` | Annex B | A `@noreload` function may call reloadable functions, through their … |
| `[HR-11]` | Annex B | Every type in a reloadable package has a schema: its kind, base, … |
| `[HR-11a]` | Annex B | Enum values are migrated by variant name, never by discriminant. |
| `[HR-12]` | Annex B | In a reloadable build, class instances are registered in a … |
| `[HR-12a]` | Annex B | The header size is a whole-process property: every package in a … |
| `[HR-13]` | Annex B | Value-typed data (structs in arrays, `SoA` columns, ECS storage, … |
| `[HR-13a]` | Annex B | Standard-library containers of a type from a reloadable package … |
| `[HR-13b]` | Annex B | Value data reachable only through raw pointers or foreign memory … |
| `[HR-14]` | Annex B | Migration is by name, and this table is exhaustive: |
| `[HR-15]` | Annex B | Migration keeps an instance's address where the new size fits; … |
| `[HR-15a]` | Annex B | A reference the runtime cannot enumerate — in foreign memory, behind … |
| `[HR-15b]` | Annex B | Every refusal is decided in plan, before prepare runs. |
| `[HR-16]` | Annex B | `fn migrate_from(old: ref OldSelf) -> Result[Self, ReloadError]` … |
| `[HR-17]` | Annex B | A static whose type and initialiser are unchanged keeps its value; … |
| `[HR-17a]` | Annex B | Editing a static's initialiser changes its schema: the reload is … |
| `[HR-18]` | Annex B | A refusal returns a report naming each type and change that caused … |
| `[HR-18a]` | Annex B | A refusal is never partial: nothing of the new image is live. |
| `[HR-18b]` | Annex B | `ember build --reload --explain` predicts the outcome against the … |
| `[HR-19]` | Annex B | `reload = "bodies"` admits only function-body changes and changes … |
| `[HR-20]` | Annex B | An object pinned by a `Retained` token that foreign code holds is … |
| `[HR-21]` | Annex B | A foreign table of exported function pointers stays valid across … |
| `[HR-22]` | Annex B | Foreign objects owned by Ember are carried across unchanged. |
| `[HR-23]` | Annex B | A changed C++ header refuses the reload and names it (the host must … |
| `[HR-24]` | Annex B | An object in use by an in-flight GPU frame is migrated in place when … |
| `[HR-25]` | Annex B | `reload` is `"all"`, `"opt-in"`, `"bodies"` or `"none"` (`[MAN-7]`); … |
| `[HR-27]` | Annex B | A `shipping` build is bit-identical whether or not the source uses … |
| `[HR-28]` | Annex B | `ember inspect --safety` reports the thunk indirection of each … |
| `[HR-29]` | Annex B | With reload enabled, one shared runtime serves every image in the … |
| `[HR-30]` | Annex B | Host contract: `ember_reload_init(&config)` once; … |
| `[HR-31]` | Annex B | Polling never blocks; compilation runs in the background. |
| `[HR-32]` | Annex B | `ember_reload_stats()` reports the last reload split into compile, … |
| `[HR-33]` | Annex B | `ember run --hot` is the toolchain's own host: it builds under … |
| `[HR-34]` | Annex B | Prepare cannot panic: every operation it performs is either free of … |
| `[HR-35]` | Annex B | `migrate_from` cannot abort the process. It may not contain an … |
| `[HR-36]` | Annex B | Allocation during prepare comes from the transaction arena; … |
| `[HR-37]` | Annex B | The transaction arena holds everything prepare allocates and is … |
| `[HR-38]` | Annex B | `std.hot.ReloadError` is `SchemaRefused(type, reason)`, … |
| `[HR-39]` | Annex B | A foreign call inside `migrate_from` returns its failure as … |
| `[HR-41]` | Annex B | Failure matrix. |
| `[HR-42]` | Annex B | Publication. Entering Ember code increments the thread's depth … |
| `[HR-42a]` | Annex B | The counter is not a lock: entering Ember code is one relaxed … |
| `[HR-43]` | Annex B | `@allow_reload_terminate` on a `migrate_from` admits such calls and … |

## IDE

| Rule | Part | Begins |
|---|---|---|
| `[IDE-3]` | XVII | Every compiler stage after parsing produces a complete result for a … |
| `[IDE-4]` | XVII | Every editor request is answered from parsing, name resolution and … |
| `[IDE-6]` | XVII | The compiler keeps all its state in a session object that can be … |

## IFC

| Rule | Part | Begins |
|---|---|---|
| `[IFC-1]` | V | `extend T:` without `implements` adds inherent methods to `T`; it is … |
| `[IFC-2]` | V | Adding inherent methods to a type from another package is `E2120`; … |
| `[IFC-3]` | V | `interface Ord: Eq` requires implementers of `Ord` to implement `Eq`. … |
| `[IFC-4]` | V | Interfaces may declare associated types and constants, not statics. … |

## IMP

| Rule | Part | Begins |
|---|---|---|
| `[IMP-11]` | XVIII | The reference implementation verifies its intermediate representation … |

## JOB

| Rule | Part | Begins |
|---|---|---|
| `[JOB-1]` | XI | A job's captured state is stored inline in its queue slot when it … |
| `[JOB-2]` | XI | `jobs.scope()` follows `[THR-5]` and `[THR-11]`: jobs submitted to a … |
| `[JOB-3]` | XI | `s.submit_after(deps, f)` starts `f` only after the jobs in `deps` … |
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
| `[LEX-14]` | II | A raw identifier `r#name` uses a keyword as a name (for imported C … |
| `[LEX-15]` | II | The table above is the complete reserved set. Contextual keywords are … |
| `[LEX-16]` | II | An integer literal without a suffix is an untyped integer: it takes … |
| `[LEX-17]` | II | A float literal without a suffix is an untyped float: it takes the … |
| `[LEX-17a]` | II | A float literal that receives `f32` or `f16` and has more significant … |
| `[LEX-18]` | II | `1.` followed by an identifier character is a method call on `1`; … |
| `[LEX-19]` | II | An f-string `{…}` contains a full expression. `{{` and `}}` are … |
| `[LEX-20]` | II | A string literal has type `str` with the static region. At a site … |
| `[LEX-21]` | II | Tokenisation is maximal munch: `//=` before `//` before `/`, `=` … |
| `[LEX-22]` | II | Ember has no lifetime syntax and never will. A `'` begins a character … |
| `[LEX-23]` | II | A line comment whose text begins `SAFETY:` or `SAFETY(<category>):`, … |
| `[LEX-24]` | II | A unary minus applied directly to an untyped integer literal forms a … |
| `[LEX-25]` | II | A raw string contains no escapes; a `r#"…"#` raw string may contain … |

## LNT

| Rule | Part | Begins |
|---|---|---|
| `[LNT-1]` | XVII | `L1001 unused binding`: a local never read on any path (names … |
| `[LNT-2]` | XVII | `L1002 assignment declares a new binding`: an unread new name within … |
| `[LNT-3]` | XVII | `L1001` and `L1002` are reported by `ember build` and `ember check`, … |
| `[LNT-4]` | XVII | `L2004`: a `gen fn` with no `yield`. |
| `[LNT-5]` | XVII | `L2005`: a `@noreload` function calling a reloadable one in a loop. |
| `[LNT-6]` | XVII | `ember lint` also reports, at `warn` unless `[lints]` says otherwise: … |

## LT

| Rule | Part | Begins |
|---|---|---|
| `[LT-1]` | VII | Signature elision. A source parameter (ODR-024) is: * a parameter … |
| `[LT-1a]` | VII | `@borrows(p, …)`, on its own line before the function, replaces the … |
| `[LT-1b]` | VII | The opt-in lint `L3014` reports rule 3 applying to more than one … |
| `[LT-3]` | VII | String literals, `bytes` literals, `static` items and views of them … |
| `[LT-4]` | VII | Arena allocations borrow the arena (`[ARN-1]`). |
| `[LT-5]` | VII | Regions of locals are inferred by non-lexical liveness (§XVIII.4). … |
| `[LT-6]` | VII | Named lifetimes are not part of Ember and will not be added. Where a … |
| `[LT-7]` | VII | Callable types. Each call through a value or parameter of callable … |
| `[LT-14]` | VII | A view type carries a compiler-internal region vector with one slot … |
| `[LT-16]` | VII | Constructing a view value keeps each field's region; it never … |
| `[LT-17]` | VII | Validity is conjunctive. A view value is usable only while every … |
| `[LT-18]` | VII | Every borrowed field is an ordinary loan under `[BRW-1]`–`[BRW-11]`; … |
| `[LT-20]` | VII | Moving, copying, destructuring, passing and returning a view … |
| `[LT-21]` | VII | Assigning a new view into a field recomputes that field's region; the … |
| `[LT-22]` | VII | A function returning a view type gets, from its body, a summary of … |
| `[LT-23]` | VII | `@borrows` on a function returning a multi-region view MUST NOT … |
| `[LT-24]` | VII | A region slot ends at the last use of its own field, not of the whole … |
| `[LT-25]` | VII | A view field may not borrow another field of the value that contains … |
| `[LT-26]` | VII | A view extends no lifetime: constructing, copying or storing it … |
| `[LT-27]` | VII | Different regions never prove that two views do not overlap in … |
| `[LT-30]` | VII | Regions exist only at compile time: two values of one view type with … |
| `[LT-34]` | VII | A view type whose fields all borrow from one source behaves exactly … |
| `[LT-35]` | VII | For each function that can receive a view value, the compiler knows … |
| `[LT-36]` | VII | Treating the view as a whole — passing it where every field may be … |
| `[LT-38]` | VII | Moving one field out moves only that field's constraint; the … |
| `[LT-39]` | VII | An operation that selects a field by a run-time value (reflection, a … |
| `[LT-42]` | VII | A non-`owned` closure capturing a view records the regions of the … |
| `[LT-43]` | VII | Region tracking never lets a view survive a `yield` that `[CORO-6]` … |
| `[LT-44]` | VII | A borrowed or `mut` `Arena`, `FixedArena` or `ScopedArena` parameter … |

## MAN

| Rule | Part | Begins |
|---|---|---|
| `[MAN-1]` | XVII | An invalid manifest, including one with an unknown key, is `E9001`, … |
| `[MAN-2]` | XVII | `ember.lock` records the resolved dependencies with content hashes; … |
| `[MAN-3]` | XVII | Every key in `[lints]` names a lint the compiler defines (`E9010` … |
| `[MAN-7]` | XVII | `[build] reload` is `"opt-in"`, `"all"`, `"bodies"` or `"none"`; it … |
| `[MAN-8]` | XVII | The sections are: `[package]` (`name`, `version`, `language`, `kind`, … |

## MNG

| Rule | Part | Begins |
|---|---|---|
| `[MNG-1]` | XVIII | Mangling is injective. A symbol is `em_` followed by each path … |
| `[MNG-2]` | XVIII | `@export("name")` sets the symbol exactly. |
| `[MNG-3]` | XVIII | Non-ASCII identifier characters are transliterated as `_uXXXX_` … |
| `[MNG-4]` | XVIII | Object structs, method tables and type information are named … |

## MOD

| Rule | Part | Begins |
|---|---|---|
| `[MOD-1]` | V | A package is a directory tree with an `ember.toml` at its root. A … |
| `[MOD-2]` | V | Items are private to their module unless marked. `pub(package)` makes … |
| `[MOD-3]` | V | `import a.b.c` binds the name `c` to module `a.b.c`, and `import … |
| `[MOD-4]` | V | Import cycles within a package are allowed; cycles between packages … |
| `[MOD-5]` | V | The prelude. Every module implicitly imports these names from `std`, … |
| `[MOD-7]` | V | A field declared `pub(read)` (or `pub(package, read)`) may be read … |
| `[MOD-8]` | V | `from m import *` binds every `pub` item of `m`. A name bound by two … |

## MONO

| Rule | Part | Begins |
|---|---|---|
| `[MONO-1]` | XVIII | Generic code is instantiated per distinct set of type arguments, from … |
| `[MONO-2]` | XVIII | The compiler records, per generic, how many instances it produced and … |
| `[MONO-3]` | XVIII | `[build] max_instantiations = N` sets a per-generic ceiling (unset by … |
| `[MONO-5]` | XVIII | A generic is shareable at a parameter `T` when `T` appears in its … |
| `[MONO-6]` | XVIII | Only for a shareable generic whose instance count exceeds the ceiling … |
| `[MONO-7]` | XVIII | `@always_specialize` forbids sharing for a generic; … |
| `[MONO-8]` | XVIII | A shared body computes exactly what the specialised ones would; it … |
| `[MONO-9]` | XVIII | A shared body is its own symbol and is deduplicated like any … |

## OBJ

| Rule | Part | Begins |
|---|---|---|
| `[OBJ-1]` | VIII | The header is 24 bytes on 64-bit targets and is part of the runtime … |
| `[OBJ-2]` | VIII | A handle points at offset 0. An interface-typed handle is the same … |
| `[OBJ-3]` | VIII | `weak` starts at 1 on behalf of all strong handles. When `strong` … |
| `[OBJ-4]` | VIII | Objects are allocated through the runtime allocator (`[RT-1]`). |
| `[OBJ-5]` | VIII | The runtime sets the deinitialising flag before the `drop` chain and … |

## OPT

| Rule | Part | Begins |
|---|---|---|
| `[OPT-1]` | VIII | When escape analysis proves that no handle to an object outlives the … |
| `[OPT-2]` | X | For a counted loop over `a..b` that indexes views at `i + c` for … |
| `[OPT-3]` | X | The loop bound in `[OPT-2]` may be written `s.len()`, a separate … |

## OWN

| Rule | Part | Begins |
|---|---|---|
| `[OWN-1]` | VII | Every value has exactly one owner: a local, a field of an owned … |
| `[OWN-2]` | VII | A value is dropped when its owner goes out of scope (block end, … |
| `[OWN-3]` | VII | A move transfers ownership and leaves the source uninitialised. Using … |
| `[OWN-4]` | VII | A loop body that moves a value declared outside the loop is `E3041`, … |
| `[OWN-5]` | VII | Assigning to a place that holds a live value evaluates the new value, … |
| `[OWN-6]` | VII | `mem.take(mut place: T) -> T` (leaves `Default`), `mem.replace(mut … |
| `[OWN-7]` | VII | A `Copy` value is duplicated bitwise on use and the source stays … |
| `[OWN-8]` | VII | `Clone.clone(self) -> Self` is the explicit deep copy. Cloning a … |

## PAN

| Rule | Part | Begins |
|---|---|---|
| `[PAN-1]` | VI | A panic prints `panic at <file>:<line>:<col>: <message>` (and a … |
| `[PAN-2]` | VI | Formatting a panic message allocates only for an f-string; … |
| `[PAN-3]` | VI | A panic inside a `drop` that runs while the process is already … |

## PAR

| Rule | Part | Begins |
|---|---|---|
| `[PAR-1]` | XI | A `@parallel` loop's body runs as a closure over index chunks on the … |
| `[PAR-2]` | XI | Iterations MUST be independent. The body MUST NOT contain `break`, … |
| `[PAR-2a]` | XI | A violation of clause (b) is `E7011`, `parallel loop has a … |
| `[PAR-2b]` | XI | The analysis runs after the body's calls are inlined. A call it … |
| `[PAR-3]` | XI | `@parallel(reduce=[total: +, best: max])` declares variables combined … |
| `[PAR-4]` | XI | A `@parallel` loop has the effects `Sync` and `Block` (it waits for … |
| `[PAR-5]` | XI | Parallel results do not depend on the machine. Chunk boundaries are a … |

## PHIL

| Rule | Part | Begins |
|---|---|---|
| `[PHIL-1]` | I | Two syntactically identical declarations in the same context have … |
| `[PHIL-2]` | I | No heap allocation happens that the source does not show. The … |
| `[PHIL-3]` | I | No implicit copy of a non-`Copy` value occurs. Copies of `Copy` … |
| `[PHIL-4]` | I | No implicit synchronisation occurs except through types documented to … |
| `[PHIL-5]` | I | Every safety check the compiler removes, it removes because it proved … |
| `[PHIL-6]` | I | Every expensive or dangerous conversion at an FFI boundary is … |
| `[PHIL-7]` | I | Panics are for programmer errors. Recoverable failures use `Result`. |
| `[PHIL-8]` | I | The enforcement ladder. Ember guarantees memory safety, lifetime … |
| `[PHIL-8a]` | I | Every rejection shape in §XVII.6 has a mandated `help` that, applied … |
| `[PHIL-9]` | I | Class instances carry a header and can therefore carry dynamic … |
| `[PHIL-10]` | I | A program containing no `unsafe` block, no `unsafe fn` and no false … |
| `[PHIL-11]` | I | What remains possible, and is therefore not a defect of the … |
| `[PHIL-12]` | I | No silent acceptance. Every construct a program writes — an … |
| `[PHIL-13]` | I | One meaning per program. No build profile, compiler flag, manifest … |
| `[PHIL-14]` | I | Python spelling, Python meaning. Where Ember accepts a spelling that … |
| `[PHIL-15]` | I | Costs are named. Every cost the language can insert without the … |

## PRF

| Rule | Part | Begins |
|---|---|---|
| `[PRF-1]` | XVII | A profile never changes what a program means. Every profile accepts … |
| `[PRF-2]` | XVII | Hot reload is a build mode admitted in `debug` and `release`, not a … |
| `[PRF-3]` | XVII | What a profile may set. |

## RC

| Rule | Part | Begins |
|---|---|---|
| `[RC-1]` | VIII | Copying a handle retains; dropping one releases; the release that … |
| `[RC-2]` | VIII | Guaranteed elisions. No retain or release is emitted in the cases … |
| `[RC-2a]` | VIII | Passing a handle to a borrowed parameter. |
| `[RC-2b]` | VIII | A handle read from a place and used only within one expression while … |
| `[RC-2c]` | VIII | A retain immediately followed by a release of the same handle with no … |
| `[RC-2d]` | VIII | A handle stored into a field from a temporary (a move, not a retain). |
| `[RC-2e]` | VIII | Handles yielded by a `for` over a borrowed collection, unless the … |
| `[RC-3]` | VIII | Further elisions are allowed only when semantics are preserved … |
| `[RC-4]` | VIII | A non-`Sync` class's counts, and a `Shared`'s, use plain loads and … |
| `[RC-5]` | VIII | A borrow whose place goes through a class handle or `Shared` (`ref … |
| `[RC-6]` | VIII | `ember inspect` lists every retain and release that survives inside a … |

## RFL

| Rule | Part | Begins |
|---|---|---|
| `[RFL-1]` | XIV | `reflect[T]()` at compile time returns a `TypeDesc`: name, kind, … |
| `[RFL-2]` | XIV | `@reflect` on a type emits run-time type information: … |
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
| `[RNG-5a2]` | IV | Overload resolution picks an implementation matching the operand … |
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
| `[RT-7]` | XVIII | A reference count never wraps: a retain that would overflow panics … |
| `[RT-8]` | XVIII | Counts of `@sync` objects use a relaxed increment for retain and an … |
| `[RT-10]` | XVIII | Counting is inline. The fast path of retain (one increment and an … |
| `[RT-11]` | XVIII | Allocation statistics are kept per thread, or only in `debug`; no … |
| `[RT-12]` | XVIII | Stack overflow faults. The backend compiles with stack probes … |

## SEL

| Rule | Part | Begins |
|---|---|---|
| `[SEL-1]` | IX | The order above is the order to try. `ember inspect --alloc` reports … |
| `[SEL-2]` | IX | `Shared`/`Weak` and C++'s `std::shared_ptr`/`std::weak_ptr` … |

## SER

| Rule | Part | Begins |
|---|---|---|
| `[SER-1]` | XIV | `@derive(Serialize, Deserialize)` implements `serialize[W: … |
| `[SER-2]` | XIV | Deserialising never produces an invalid value: a range-typed field is … |

## SIMD

| Rule | Part | Begins |
|---|---|---|
| `[SIMD-1]` | XII | `std.simd` provides vector types `f32x4`, `f32x8`, `f32x16`, `f64x2`, … |
| `[SIMD-2]` | XII | `@simd` on a `for` loop is a request and a report: the compiler tries … |
| `[SIMD-3]` | XII | Alias facts given to the backend (`restrict`) MUST be derived, never … |
| `[SIMD-4]` | XII | Horizontal operations (`reduce_add`, `reduce_min`, `reduce_max`), … |
| `[SIMD-5]` | XII | Vectorisable form, which `@simd(assert)` checks, is computed by Ember … |
| `[SIMD-6]` | XII | The C compiler's own vectorisation report is corroborating evidence … |
| `[SIMD-7]` | XII | Grouped overflow checks. In a loop whose body has none of `Sync`, … |
| `[SIMD-8]` | XII | Vector operators are lane-wise and follow the scalar rules: integer … |
| `[SIMD-9]` | XII | SIMD memory operations are bounds-checked like indexing. `V.load(s)` … |

## SOA

| Rule | Part | Begins |
|---|---|---|
| `[SOA-1]` | XII | `SoA[T]` is a compiler-known type constructor, like `Cell`: for any … |
| `[SOA-2]` | XII | `ps.f` names the column of field `f`. It is a place of type … |
| `[SOA-3]` | XII | `SoA[T]` provides `len`, `is_empty`, `push`, `pop`, `swap_remove`, … |
| `[SOA-4]` | XII | `ArenaSoA[T]` is the fixed-capacity, arena-backed form, under the … |
| `[SOA-6]` | XII | `ps[i]` is an element proxy: `SoARef[T]` where `ps` is read-only here … |
| `[SOA-7]` | XII | All columns of one `SoA[T]` live in one heap block, each column … |

## SPN

| Rule | Part | Begins |
|---|---|---|
| `[SPN-1]` | VII | An `Array[T]`, a `[T; N]` or a `String` converts to `Span[T]`/`str` … |
| `[SPN-2]` | VII | Indexing a view is bounds-checked; `get(i) -> Option[ref T]` does not … |
| `[SPN-3]` | VII | `Span[T]` is `Copy`; `MutSpan[T]` is move-only and reborrowable … |
| `[SPN-4]` | VII | `iter()` on a `Span` or `MutSpan` yields `ref T`; `iter_mut()` on a … |
| `[SPN-5]` | VII | `split_at(i)` on a `Span` returns two `Span`s; on a `MutSpan` it … |
| `[SPN-8]` | VII | `as_ptr()` and `as_mut_ptr()` return raw pointers; extracting one is … |

## STA

| Rule | Part | Begins |
|---|---|---|
| `[STA-1]` | V | `static NAME: T = e` is one value per program with a stable address. … |
| `[STA-2]` | V | There is no static-initialisation-order problem: statics are … |
| `[STA-3]` | V | A static whose initialiser can be evaluated at compile time is placed … |

## STD

| Rule | Part | Begins |
|---|---|---|
| `[STD-1]` | XV | Every function in `std.core`, `std.mem`, `std.math`, `std.simd` and … |
| `[STD-2]` | XV | Printing allocates only to build an f-string argument; … |
| `[STD-3]` | XV | `math.fma(a, b, c)` (and `mul_add`) is a fused multiply-add with a … |
| `[STD-4]` | XV | `NonZero[T]` for each integer `T` is a `Copy` wrapper with a niche … |
| `[STD-5]` | XV | `sum` and `product` combine elements left to right in the element … |
| `[STD-6]` | XV | `std` is layered, and each layer depends only on the ones before it: … |
| `[STD-7]` | XV | Fixed-capacity containers with no heap allocation: `FixedArray[T, … |
| `[STD-8]` | XV | `Contains`. `x in c` requires `c: Contains[typeof(x)]` (`E2226` … |
| `[STD-8a]` | XV | `ember inspect --cost` reports which `contains` a use selects and its … |
| `[STD-8b]` | XV | `str` implements `Contains[char]` and `Contains[str]` only; matching … |
| `[STD-9]` | XV | `print(a, b, …, sep=" ", end="")` and `println(a, b, …, sep=" ", … |
| `[STD-10]` | XV | `input(prompt="") -> String` prints the prompt, flushes, reads one … |
| `[STD-11]` | XV | `Map[K, V, H = DefaultHasher, A = Global]` is a hash map that … |
| `[STD-12]` | XV | Borrowed keys. Lookup methods, `m[k]` and `k in m` accept any key … |
| `[STD-13]` | XV | One naming convention. Names are `snake_case` words joined by … |
| `[STD-14]` | XV | Failure follows `[ERR-13]`: a caller's bug panics and has a … |
| `[STD-15]` | XV | `Array[T]` is a growable contiguous list (Python's `list`). `len`, … |
| `[STD-16]` | XV | `Map` operations. `m[k]` reads the value and panics when the key is … |
| `[STD-17]` | XV | Index assignment. `a[i] = v` calls `IndexSet.index_set(i, v)` when … |
| `[STD-18]` | XV | `format(value, spec="") -> String` formats one value with a format … |
| `[STD-19]` | XV | Every `Iterator` has the adapters `map`, `filter`, `filter_map`, … |
| `[STD-20]` | XV | Integer methods, built into every integer type: `abs` (a signed `MIN` … |
| `[STD-21]` | XV | `std.math` provides `PI`, `TAU`, `E`; `sin`, `cos`, `tan`, `asin`, … |
| `[STD-22]` | XV | Every operation that touches the outside world returns `Result[T, … |
| `[STD-23]` | XV | `time.Instant.now()` is monotonic and `Nondet`; `Duration` is an … |
| `[STD-24]` | XV | `process.exit(code) -> Never` flushes the standard streams and exits … |
| `[STD-25]` | XV | `random.Rng.seeded(seed)` is a deterministic generator (the same seed … |
| `[STD-26]` | XV | Python's built-in functions. The prelude has these functions, with … |
| `[STD-27]` | XV | `std.math.Number` is every number type. `std.math` adds each with an … |
| `[STD-28]` | XV | Vectors, matrices, rotations and shapes (ODR-043). `std.math` has … |

## STR

| Rule | Part | Begins |
|---|---|---|
| `[STR-1]` | V | Every struct has a memberwise constructor `Name(field0, field1, …)` … |
| `[STR-2]` | V | A field default is any expression; it is evaluated at each … |
| `[STR-3]` | V | A struct with a `drop` method, or with a field that needs drop, is … |
| `[STR-4]` | V | A struct may have no fields (`struct Marker: pass`). |
| `[STR-5]` | V | Implicit derives. A struct or enum implements `Eq`, `Debug` and … |
| `[STR-6]` | V | `fn init(self, …)` on a struct is its constructor: every field … |
| `[STR-7]` | V | Inside the body of a `struct`, `enum`, `class` or `extend` block, … |

## TCB

| Rule | Part | Begins |
|---|---|---|
| `[TCB-1]` | Annex C | Every fact an overlay states about foreign code carries a grade: … |
| `[TCB-2]` | Annex C | The report separates what the language guarantees from what external … |
| `[TCB-3]` | Annex C | Every assumption a guarantee relies on appears in the report with its … |
| `[TCB-4]` | Annex C | Entries are categorised — language, compiler, runtime, standard … |
| `[TCB-5]` | Annex C | An instrumented fact is valid only for the foreign library, header, … |
| `[TCB-6]` | Annex C | A change to any identity input makes the record stale; a change to an … |

## THR

| Rule | Part | Begins |
|---|---|---|
| `[THR-1]` | XI | A class is `Sync` only when it is declared `@sync class`. Every other … |
| `[THR-2]` | XI | Handles of a `@sync` class are `Send` and `Sync`. A handle of any … |
| `[THR-3]` | XI | `Mutex[T]` is a value type. `m.lock()` blocks and returns a … |
| `[THR-4]` | XI | In the `debug` profile the runtime records the order in which each … |
| `[THR-5]` | XI | `thread.scope()` returns a `Scope`, which MUST be bound by a `with` … |
| `[THR-6]` | XI | A type whose `drop` a safety guarantee depends on is `@must_drop`. A … |
| `[THR-7]` | XI | `@sync` means exactly two things: the class's handles may cross … |
| `[THR-8]` | XI | A type is `Send` when a value of it may be moved to another thread. … |
| `[THR-9]` | XI | A type is `Sync` when several threads may read one value of it at the … |
| `[THR-10]` | XI | `thread.spawn(owned f: fn() -> R) -> JoinHandle[R]` runs `f` on a new … |
| `[THR-11]` | XI | `scope.spawn(f: fn() -> R) -> ScopedJoinHandle[R]` accepts a closure … |
| `[THR-12]` | XI | Tasks that must run in sequence and share data use two scopes in … |
| `[THR-13]` | XI | The data-race guarantee. In Safe Ember a memory location is reachable … |
| `[THR-14]` | XI | `Atomic[T]` exists for the integer types, `bool` and raw pointers. … |
| `[THR-15]` | XI | `channel[T](capacity=n) -> (Sender[T], Receiver[T])` is a bounded … |
| `[THR-16]` | XI | When `main` returns, or `process.exit` is called, the process ends: … |

## TIER

| Rule | Part | Begins |
|---|---|---|
| `[TIER-1]` | I | Safe code MUST NOT invoke an operation with an unverifiable … |

## TOOL

| Rule | Part | Begins |
|---|---|---|
| `[TOOL-1]` | XVII | Each release publishes a self-contained toolchain archive per … |
| `[TOOL-2]` | XVII | `ember toolchain install cc` installs a pinned Clang and linker and … |
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
| `[TST-6]` | XVII | Appendix A's code is generated from a fixture that is compiled in CI. … |
| `[TST-7]` | XVII | Every ` ```ember ` block in this document is extracted and must pass … |
| `[TST-8]` | XVII | `tests/firstweek/` holds at least 24 first-draft programs a newcomer … |
| `[TST-9]` | XVII | Each is marked `accepted` or `rejected(<shape>)`; a rejection whose … |
| `[TST-10]` | XVII | The acceptance rate is published with each release; a release that … |
| `[TST-11]` | XVII | Besides the per-rule cases, the suite covers these scenarios: … |
| `[TST-27]` | XVII | The C gate. Every accepted program in the test suite is compiled … |
| `[TST-28]` | XVII | The performance gate. `tests/perf/` holds benchmark programs, each … |
| `[TST-29]` | XVII | Honest baselines. A gate with a baseline of known failures reports … |
| `[TST-30]` | XVII | The test runner reports every failure of a run, not only the first, … |

## TXT

| Rule | Part | Begins |
|---|---|---|
| `[TXT-1]` | XV | `str` is a borrowed `Span[u8]` known to be valid UTF-8; `String` owns … |
| `[TXT-2]` | XV | Nothing becomes a `str` without validation: every conversion from … |
| `[TXT-3]` | XV | `str` and `String` are a pointer and a length, not null-terminated, … |
| `[TXT-4]` | XV | Slicing a string, `s[a..b]`, is by byte offset and panics in every … |
| `[TXT-5]` | XV | `str` → `String` allocates and copies; `String` → `str` is free; … |
| `[TXT-6]` | XV | Across the C ABI a `str` is `{const uint8_t *ptr; size_t len}`. |
| `[TXT-7]` | XV | Text is UTF-8 everywhere. `std.ffi.WideString` converts, explicitly, … |
| `[TXT-8]` | XV | `str` is a view type and carries a region, like every `Span`. |
| `[TXT-9]` | XV | A string literal initialises a `String`. Wherever a `String` is … |
| `[TXT-10]` | XV | `str` operations. `len()` is the length in bytes and `char_count()` … |
| `[TXT-11]` | XV | `String` operations. Everything `str` has (by read-through), plus … |

## TYP

| Rule | Part | Begins |
|---|---|---|
| `[TYP-1]` | IV | Every concrete type has, at compile time, a size, an alignment, and … |
| `[TYP-2]` | IV | Producing a `bool` other than 0 or 1 is undefined behaviour and … |
| `[TYP-3]` | IV | Producing a `char` outside the Unicode scalar values is undefined … |
| `[TYP-4]` | IV | No value converts implicitly between scalar types in an operator. … |
| `[TYP-5]` | IV | Coercions — the complete list. At a coercion site (assignment or … |
| `[TYP-6]` | IV | `x as T` converts explicitly: * integer → integer: keeps the low bits … |
| `[TYP-7]` | IV | `as` between pointer types, or between a pointer and an integer, … |
| `[TYP-8]` | IV | Integer overflow panics in every profile. An arithmetic operation (`+ … |
| `[TYP-9]` | IV | Floating point is strict IEEE 754: no reassociation, no contraction … |
| `[TYP-9a]` | IV | Contraction is off by default and the implementation MUST turn it off … |
| `[TYP-9b]` | IV | `@fp(contract)` permits, and requires the backend to enable, fused … |
| `[TYP-9c]` | IV | A toolchain that cannot honour `@fastmath` or `@fp(…)` for one … |
| `[TYP-10]` | IV | Shifts. `a << n` and `a >> n` accept any integer type for `n`. If `n` … |
| `[TYP-11]` | IV | A struct's default layout is C's: declaration order, natural … |
| `[TYP-12]` | IV | A unit-only enum is an integer (`@repr(u8)` and friends choose it; … |
| `[TYP-13]` | IV | `Option[T]` has the size of `T` when `T` has a niche: a class handle, … |
| `[TYP-14]` | IV | A reference, and a `Box[T]`, is read through wherever a `T` is wanted … |
| `[TYP-15]` | IV | Where views may be stored. A view value may be stored only in a place … |
| `[TYP-15a]` | IV | The arena-backed containers (`ArenaArray`, `ArenaMap`, §IX.2) and the … |
| `[TYP-16]` | IV | Generic functions and types are monomorphised: each distinct … |
| `[TYP-17]` | IV | Type parameters are bounded by interfaces: `fn sum[T: Add[Output = T] … |
| `[TYP-18]` | IV | Generic arguments are inferred from the arguments of a call. Explicit … |
| `[TYP-19]` | IV | There is no specialisation, no higher-kinded type and no variadic … |
| `[TYP-20]` | IV | Coherence is per package. An implementation of interface `I` for type … |
| `[TYP-21]` | IV | Operators desugar to these interfaces for non-scalar operands; `a + … |
| `[TYP-22]` | IV | An interface is usable as `dyn` only if every method has a receiver, … |
| `[TYP-23]` | IV | Inference is local to a function body and bidirectional. Function … |
| `[TYP-24]` | IV | An interface method is found through any implementation visible in … |
| `[TYP-25]` | IV | Arguments may be positional or named; positional arguments come … |
| `[TYP-26]` | IV | There is no overloading: two functions of one name in one scope are … |
| `[TYP-27]` | IV | `()` is the value of type `void`; `Ok(())` is the success value of … |
| `[TYP-28]` | IV | Division. `/` is true division and applies to floats. `/` with two … |
| `[TYP-29]` | IV | For floats, `//` and `%` are Python's (ODR-021). `a % b` is the exact … |
| `[TYP-30]` | IV | Powers. `a  b` with integer `a` and non-negative integer `b` is … |
| `[TYP-31]` | IV | Sizes and indices are `int`. Every standard container's `len()` … |
| `[TYP-32]` | IV | `some I` in a return type means "one concrete type, chosen by the … |
| `[TYP-34]` | IV | *(replaces the 0.9.8 `@view` requirement)* A struct, enum or tuple … |
| `[TYP-35]` | IV | A type parameter need not appear in any field (a phantom parameter): … |
| `[TYP-36]` | IV | Which types implement which interfaces. The table is normative; a `—` … |
| `[TYP-37]` | IV | Floats implement `Eq` with IEEE `==` (so `NaN != NaN`) and `Ord` with … |
| `[TYP-38]` | VI | Collection literals. * A list literal `[a, b, c]` has the type its … |
| `[TYP-39]` | IV | Collections, tuples and `Option`/`Result` implement `Display` the way … |
| `[TYP-40]` | IV | Interfaces are nominal. A type implements an interface only through … |

## UNS

| Rule | Part | Begins |
|---|---|---|
| `[UNS-1]` | IX | An `unsafe` context is required to: dereference, read or write … |
| `[UNS-2]` | IX | `unsafe` permits exactly those operations. It does not turn off … |
| `[UNS-3]` | IX | `L3010` reports an `unsafe` block containing statements that need no … |
| `[UNS-4]` | IX | Unsafe code MUST uphold what safe code assumes: every reference is … |
| `[UNS-5]` | IX | `std.mem` provides `Volatile[*T]` (volatile reads and writes), … |
| `[UNS-6]` | IX | Inline assembly is `unsafe asm("…", …)` on toolchains that support it … |
| `[UNS-7]` | IX | Every `pub unsafe fn` carries `@safety("…")` stating the caller's … |
| `[UNS-8]` | IX | Every `unsafe:` block and `unsafe fn` carries a safety note … |
| `[UNS-10]` | IX | `UnsafeCell[T]` (in `std.mem`) is the primitive beneath every … |
| `[UNS-10a]` | IX | `UnsafeCell` suspends no rule globally: borrow, region, type and … |

## VER

| Rule | Part | Begins |
|---|---|---|
| `[VER-1]` | 0 | Three version numbers exist and move independently: the language … |
| `[VER-2]` | 0 | From 1.0: source compatibility within a major language version; a … |
| `[VER-3]` | 0 | From 1.0: deprecation through `@deprecated(since, note)`, removal no … |
| `[VER-4]` | 0 | The runtime ABI (object header, `ember_type_info`, every entry point … |
| `[VER-5]` | XVII | Package versions are semantic; a requirement `"1.2"` means `>= 1.2.0, … |
| `[VER-8]` | 0 | Before 1.0 there is exactly one language: the current one. A source … |
| `[VER-9]` | 0 | A language revision that changes the set of accepted programs or … |

## WK

| Rule | Part | Begins |
|---|---|---|
| `[WK-1]` | VIII | A cycle of strong handles among class instances and `Shared` payloads … |
| `[WK-2]` | VIII | An object is deinitialised when its strong count reaches zero, … |
| `[WK-3]` | VIII | `upgrade` returns `None` while the object's deinitialising flag is … |
| `[WK-4]` | VIII | The leak report names, for each leaked object on a cycle, the … |
| `[WK-5]` | VIII | The compiler builds a graph of strong ownership among class fields … |
| `[WK-6]` | VIII | A cycle of strong edges in that graph is warning `L3001` at the field … |
| `[WK-7]` | VIII | Cycle analysis is conservative: a possible cycle suffices for … |
| `[WK-8]` | VIII | The run-time report (`[WK-15]`) lists each leaked strongly connected … |
| `[WK-9]` | VIII | `ember explain --cycle <path> <Class[.field]>` explains one … |
| `[WK-11]` | VIII | `Weak(h)` creates a weak handle to a class object or a `Shared` or … |
| `[WK-12]` | VIII | `w.upgrade() -> Option[O]` returns a retained strong handle while the … |
| `[WK-13]` | VIII | A `Weak[Shared[T]]` refers to the same block as its `Shared[T]`. |
| `[WK-14]` | VIII | Ember's `Weak` and `Shared` never convert to or from C++'s … |
| `[WK-15]` | VIII | `ember run` and `ember test` in the `debug` profile report leaked … |
