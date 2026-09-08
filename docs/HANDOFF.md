# Ember — handoff

## Start here

**Phase 0 of nine is complete and pushed.** Phase 1 is in progress: blocks A,
B, C and D of seven are done, blocks E to G are not started.

Read `docs/spec/` (the specification, split by part) and `docs/DECISIONS.md`
(the nine owner decisions) before touching anything. `docs/spec-errata.md`
lists eight places where the specification is silent or contradicts itself.
Five have been ruled on by the owner and patched into `docs/spec/` (ERR-001,
ERR-005, ERR-006, ERR-007, ERR-008); the other three are still proposals.

| | |
|---|---|
| Repository | `https://github.com/Insomniac-Coder/ember.git` |
| Pushed | `95f3269` on `origin/main` — all of Phase 0 |
| Working branch | **`phase-1-core-language`**, blocks A to D **uncommitted** |
| Tests | `cargo test --workspace` → **150 passed, 0 failed** |
| Build | warning-free; the emitted C is warning-free under `clang -Wall -Wextra` and MSVC `/W3`, which is what `ember_build` passes and what `[CG-C-1]` asks for |

## Hard constraint

**Never read, build or modify anything in `Code/RageV`.** Three conditions must
all hold before engine code may even be raised as a question (owner,
2026-09-07):

1. all nine phases complete — not Phase 5, when the C FFI first makes it
   technically possible;
2. the owner has inspected the work themselves and given an explicit green
   flag;
3. any engine work then happens on a separate RageV branch named
   `RageV_Ember`, never the working branch.

The specification's Part XXI is a *description* of a possible integration, not
a work queue.

## Working with this owner

- **Explain in plain words, and lead with the consequence, not the mechanism.**
  Several long explanations were rejected outright during Phase 0. What landed
  was a table of "you write X / it used to do Y / it now does Z".
- **Show, do not describe.** Running the compiler and pasting its real output
  settled arguments that four written explanations did not.
- Report at block boundaries, not per file.

## Phase 1 — where it stands

Seven blocks, dependency-ordered. A, B, C and D are done.

| | Block | State |
|---|---|---|
| **A** | numeric semantics `[TYP-4..10]`, `Assert` lowering, definite-init | **done** |
| **B** | tuples, fixed arrays, enums, `match` with exhaustiveness | **done** |
| **C** | `for` over ranges, loop `else`, labelled break, `with`, `defer` | **done** |
| **D** | interfaces, operator interfaces, method resolution, `extend`, visibility | **done** |
| E | `String`/`Array`, f-strings, `Option`/`Result`, `?` | not started |
| F | modules across files, `const`, `static` | not started |
| G | formatter (`[FMT-1]`) | not started |

### Block A, in detail

- `[TYP-8]` — `debug` panics on overflow, `release`/`shipping` wrap.
  `@overflow(panic|wrap)` overrides per function. `/` and `%` by zero always
  panic whatever the policy, as does `T.MIN / -1`.
- `[TYP-10]` — a shift amount at or past the type's width panics under `panic`
  and is masked under `wrap`.
- `Terminator::Assert` in MIR, carrying the span of the operation, reaching
  `ember_panic_*`. Checked arithmetic goes through `ember_ck_*` in
  `ember_rt.h`: `__builtin_*_overflow` on clang and gcc, exact widening
  fallbacks for MSVC.
- New crate **`ember_analysis`** with definite initialisation — the
  `Uninit | Init | Maybe` lattice of Part XVIII §4.6, reported as `E3050`.
  The NLL borrow checker (§4.7), exclusivity (§4.8), drop elaboration (§4.9)
  and effects (§4.10) belong in this crate too.
- MIR statements and terminators now carry spans (`Stmt { kind, span }`,
  `BasicBlock::terminator_span`).
- The test harness handles `tests/compile-fail/` and `tests/run-fail/`.

**Deferred, deliberately:** `@overflow(saturate)` is rejected with a clear
diagnostic rather than silently treated as `wrap`. It needs each type's bounds
as constants, and the backend cannot yet render a signed minimum without
tripping C's `-2147483648` parsing quirk (that literal is `-(2147483648)`,
which overflows `int`). Emit `(-MAX - 1)` when this is picked up.

### Block B, part one: tuples and fixed arrays

Done end to end — written, run, and checked against the emitted C.

- **C has no tuple, and a bare C array cannot be assigned, passed or returned
  by value**, but an Ember tuple and an Ember `[T; N]` are ordinary values
  (Part IV.3). Both are therefore emitted as a **generated struct** with the
  same layout: `struct em_tup_i32_f32 { int32_t _0; float _1; };`,
  `struct em_arr_i32_4 { int32_t _0[4]; };`. C's value semantics then apply
  with no change to any copy, argument or return path in the backend. The
  alternative — bare arrays plus `memcpy` and a hidden out-parameter for
  returns — is more work *and* a worse result, and it still needs generated
  structs for tuples.
- `plan_types` in the C backend orders every type definition so that a type is
  defined after everything it holds by value, and names the generated ones
  from how they print (`[[i32; 2]; 2]` → `em_arr_i32_2_2`), made unique
  against every other name. A reference or pointer is not a containment edge,
  because everything is forward-declared.
- **Indexing is bounds-checked** (Part IV.3). The comparison and its
  `Assert` are built during MIR lowering, not in the backend, so the check is
  visible to every MIR analysis — the same choice `[TYP-8]` made. Block A had
  already built `AssertKind::Bounds` and `ember_panic_bounds`; nothing had
  generated them until now.
- `Rvalue::Repeat` keeps `[value; count]` as one statement rather than `count`
  operands, and the backend emits a `for` loop. `[0; 4096]` would otherwise be
  four thousand entries in the IR and in the C.
- **`t.0.1` did not parse**: the lexer reads `0.1` as one float literal, so
  every chained tuple index was a syntax error. `eat_tuple_index_pair` splits
  the token in the parser, reading the text from the source rather than from
  the `f64` (which no longer tells `.1` from `.10`). Three levels work, because
  `1.1.0` arrives as a float and then a separate `.0`.
- An array length must be an integer literal for now (`E2131`); Part IV.3 makes
  `N` a const generic, which needs const generics and `const` items.

**Not done, and deliberately:** Part IV.3's coercion of `[T; N]` to
`Span[T]`/`MutSpan[T]` at coercion sites. `Span` is a standard-library type and
arrives in block E.

### Block B, part two: enums and `match`

- **A unit-only enum is a `typedef` of its repr integer** (`[ENM-3]`), so it
  costs nothing and `as` to an integer is the identity. **A payload enum is
  `{tag, union}`** (`[TYP-12]`), the union holding one anonymous struct per
  variant that has fields. `@repr(u8)` names the tag type; without it the
  compiler picks the smallest of `u8`/`u16`/`u32` (or the signed ones, when a
  discriminant is negative) that holds every value, which is what Part IV.3's
  "`i32`-sized-or-smaller, chosen by the compiler" means.
- `SwitchInt`'s case values became **`i128`, not `u128`** — a negative
  discriminant rendered through `u128` prints a huge positive number into the
  emitted `switch`.
- **Exhaustiveness and redundancy are one algorithm** (Maranget's usefulness
  test, `compiler/ember_typeck/src/usefulness.rs`). An arm is `W2091` when it
  is not useful against the arms above it, and the `match` is `E2090` when a
  bare `_` still would be — so "not covered" and the witnesses it lists cannot
  disagree. A guarded arm covers nothing, because its guard may fail.
- **`match` lowering has two shapes.** When every arm names a different
  variant, with no guard and payloads only bound, the whole thing is one
  `SwitchInt` on a tag read once — a real `switch` in the C. Anything else
  (guards, wildcards, two arms on one variant, a non-enum scrutinee) falls
  back to a chain of per-arm tests, which handles every pattern form at one
  test per arm. Both were measured against the emitted C, not assumed.
- **Every alternative of an `|` pattern binds the same locals.** Checking each
  alternative separately would declare a new local per alternative, and the
  body would read whichever was declared last — so the first alternative's
  bindings are recorded and the rest reuse them (`or_bindings` in the checker).
- **Three defects found and fixed on the way**, all pre-existing:
  - `println(10.0)` printed `1e+01`. `write_shortest_f64` took the first
    *precision* that round-trips, and `%.1g` of 10.0 is `"1e+01"`, which does.
    It now keeps the shortest *text*, so `10` wins and `1e+20` still beats
    twenty-one digits.
  - `n = match d:` with indented arms did not parse: the match consumed the
    line's end along with the block's `Dedent`, and the enclosing statement
    then demanded a newline that was gone.
  - An unused pattern binding tripped `-Wunused-but-set-variable`, breaking
    `[CG-C-1]`. Locals that are written and never read are now discarded once
    with `(void)`.

**Not done:** slice patterns and range patterns (both parse, both are reported
as unsupported); `from_repr` (`[ENM-3]`) needs `Option`, which is block E;
`[TYP-13]`'s niche optimisation, which Phase 3 gives something to optimise.

### Block C: `for`, loop `else`, labels, `with`, `defer`

- **`for i in a..b` is a counted loop and nothing else** (`[CTL-3]`). It is not
  desugared through `Iterator` — that needs interfaces — so `hir::Stmt::ForRange`
  carries the two ends directly and MIR builds the counter. The emitted C holds
  two `int32_t` and a `bool`, which is what `[CTL-3a]` asks for; the test
  asserts no range struct appears.
- **The continue target is not the loop head.** A counted loop's `continue`
  must run the increment or it spins forever, so the loop stack records where
  `continue` goes separately from where `break` goes.
- **Loop `else` needs no flag** (`[CTL-4]`). Falling out of the condition goes
  to the `else` block; `break` jumps to a second block past it. The two exits
  are different basic blocks, so nothing is tested at run time.
- **A label is resolved to a depth while checking**, so MIR never sees a name —
  `break outer` becomes `break 1`, and the lowering indexes its loop stack.
- **`defer` runs on every way out** (`[CTL-7]`, `[CTL-8]`): the end of the
  block, a `return`, and a `break` or `continue` that leaves the scope. The
  pending list is *not* shortened when a `return` emits it, because another
  branch of the same block still has to run the same blocks on its own way out.
  Each loop records how many defers were pending when it opened, which is what
  `break` and `continue` unwind to.
- `[CTL-7]`'s restriction is checked: `return` inside a `defer` is `E2160`, and
  the loops outside are hidden while checking one, so `break` cannot reach them.

**Not done:** `for` over anything but a range (needs `Iterator`, so block D or
E); `a..` with no end; destructors at `with`-block exit, which is what makes
`with lock.acquire():` a guard — until drop elaboration exists, `with` binds
for the block and no more. **The inclusive form stops short of the type's
maximum**: `0..=i32.MAX` would wrap at the increment, and a checked step would
cost a branch in every counted loop. Worth a decision when `for` is revisited.

## Crate state

A note on the specification: **Appendix A's `match` example is not what the
parser accepts.** It writes `Circle(r) => return PI * r * r`, but `return` is a
statement and `=>` takes an expression; the statement form is `Circle(r):` and
an indented block. Worth an erratum.

| Crate | State |
|---|---|
| `ember_span` | `FileId`, `Span`, `SourceMap`, `Symbol` interner |
| `ember_diag` | model, renderer matching XIX §6, JSON, 90-code registry |
| `ember_lexer` | all of Part II |
| `ember_ast` | full Part XVIII §2 tree, structural dump (prints doc comments) |
| `ember_parser` | **the whole v1 grammar** |
| `ember_types` | interner, C layout, `Copy`/`needs_drop`/`is_view`, `OverflowPolicy` |
| `ember_hir` | typed, desugared tree; scalar and struct cases populated |
| `ember_typeck` | bidirectional checking, literal defaulting, coercion sites, patterns, `match` exhaustiveness |
| `ember_mir` | CFG with spans, lowering, pruning, verifier, checked arithmetic |
| `ember_analysis` | definite initialisation |
| `ember_codegen_c` | MIR to C11, `#line`, shortest round-trip floats |
| `ember_build` | MSVC via captured `vcvars64`, clang, gcc |
| `ember_driver` | `build`, `run`, `check`, `explain`, `--emit`, `--json` |
| `runtime/ember_rt` | C11: alloc, panics, checked arithmetic, printers, embedding API |

## What the language can do today

Structs with C layout, memberwise constructors, field access, functions with
the three parameter modes, locals with inference and definite-init checking,
`if`/`elif`/`else`, `while`, `break`/`continue`, scalar arithmetic with
`[TYP-4]`'s no-implicit-conversion rule and `[TYP-8]`/`[TYP-10]`'s checks,
`[TYP-5]` widening at coercion sites, untyped-literal defaulting, short-circuit
`and`/`or`, casts, and `println` for every scalar and `str`.

## Load-bearing invariants

**`Span` offsets are always absolute byte offsets into the normalised file.**
`ember_span::normalise` strips a BOM and folds CRLF to LF before anything else
sees the text. `lex_range` (f-string interpolations) truncates the source at
`end` rather than slicing from `start`, precisely so spans stay absolute.

**The MIR builder tracks `current_span`**, set by `Builder::at` at the top of
`lower_into`, `lower_rvalue` and `lower_operand`. `push` and `terminate` both
read it. If a new lowering entry point forgets to call `at`, its statements
inherit the previous statement's span and diagnostics silently point at the
wrong line — which is exactly the bug that made `E3050` blame declarations.

**`referenced_blocks` in the C backend must agree exactly with
`emit_terminator`'s fallthrough rule.** A jump to the next block in order emits
no `goto`, so it must not mark that block referenced. If they drift, the C
either has an unreferenced label (a warning, breaking `[CG-C-1]`) or a `goto`
to a label that was never emitted (a hard error).

**MIR lowering creates a fresh block after every `return`, `break` and
`continue`.** Most are unreachable and `prune_unreachable` removes them before
codegen. Without it the C is full of dead labels.

**The parser's cascade limit is per *region*, not per file** (`[AST-2]`). If a
recovery path fails to advance the cursor, the limit silently swallows every
later diagnostic — so every recovery loop checks
`if self.pos == before { self.bump() }`.

**The `Symbol` interner leaks, on purpose.** `Box::leak` buys a `&'static str`
with no unsafe code and no lifetime plumbing. A compiler process interns a
bounded set of names and exits.

**`workspace.lints.rust` denies `unsafe_code`.**

**`ember check` must run the MIR analyses.** Part XIX §1 defines it as
"type-check + borrow-check without codegen". It returned right after type
checking at first, so definite-init never ran and every `compile-fail` test
passed vacuously.

## Traps paid for

**Writing Rust or C source containing `\n`, `\\`, `\"` or a line-continuation
backslash through a bash heredoc into a Python string destroys the escapes.**
They become real newlines and tabs, and the damage is invisible in a diff — it
looks like the file was always that way. This cost four separate rewrites
(`scan_escape` twice, the `ember_rt.h` macros, a `replace` call in the C
backend). **Write such files with the Write or Edit tool directly, never
through a shell.** If a script is genuinely needed, write the script to a file
with Write and then run it.

**A panicking program reported exit code 0.** Windows delivers `abort()` as
`0xC0000409`, which arrives as a large negative `i32`; `clamp(0, 255)` turned
it into a clean exit. Anything not representable as `u8` is now a plain
failure. Every `run-fail` test would otherwise have passed vacuously.

**An unrecognised `#$` key in a test is silently ignored.** A `run-fail` test
written with `#$ panic:` instead of `#$ panics:` passes without ever checking
the panic — the same vacuous pass as the exit-code trap above. And only
`milestones_pass` asserts that it found any files; `run-pass`, `run-fail` and
`compile-fail` all pass on an empty or missing directory. **After adding a
test, break it on purpose once and watch the suite go red.** A `compile-fail`
expectation is a substring of the rendered message, not the registry title.

**Part II §4's keyword table has 47 entries, not 46.** `ember_lexer::token`
asserts the count so the table and the enum cannot drift.

**`Range ..` is a prefix of `Range ..=`.** Substring counts over the AST dump
over-count; match to end of line.

**`cl.exe` cannot find its own headers without `INCLUDE` and `LIB`, and no flag
substitutes.** `ember_build` runs `vcvars64.bat` through `cmd /c … && set` and
captures the environment. A capture missing `INCLUDE` means the batch file did
not really run.

**`clang.exe` is not on `PATH`** even with LLVM installed;
`Toolchain::find_on_path` falls back to `C:\Program Files\LLVM\bin`.

### Block D: methods, interfaces, `extend`, operators

- **`mut` parameters did not work at all before this block.** `fn bump(mut n:
  i32)` was compiled by value, so `bump(x)` left `x` alone — parsed, stored in
  the signature, and ignored by every later stage. A `mut` parameter is now a
  `ref mut T` inside the function (which is what Part V's mode table says it
  is), every mention of its name reads through it, and the call site passes the
  address. `mut self` is the same mechanism, so it had to be fixed before any
  method could mutate its receiver.
- **A method is a function whose first parameter is the receiver.** There is no
  vtable and no dynamic dispatch: `recv.m(args)` resolves at compile time to
  one symbol, `em_<Type>_<method>`. `dyn` is what needs vtables, and it is not
  in this block.
- **`[TYP-24]` is enforced through one table**, `methods: (Ty, Symbol) ->
  entry`, which records whether an entry came from an interface. An inherent
  method registered later replaces an interface one; two interfaces offering a
  name is `E2070` at the call site, not at the declaration, because it is only
  ambiguous when someone calls it.
- **Required-method checks run after all collection**, not while walking each
  type: a type may declare `implements I` in its header and define the methods
  in a later `extend` block. Checking in place reported methods as missing that
  were defined ten lines further down.
- **An interface default body becomes one real method per implementing type**,
  registered during collection so that a call in any function body resolves,
  and checked afterwards with `self` bound to the concrete type.
- `[TYP-21]` — an operator whose left operand has the matching method calls it:
  `a + b` becomes `a.add(b)`. The check is on the method's presence, not on a
  declared `Add` interface, so `extend Vec2 implements Add:` and a plain
  `extend Vec2:` both work.
- An unused parameter now gets the same `(void)` discard an unused local does.
  `fn sides(self) -> i32: return 4` ignores its receiver, which is ordinary
  Ember and `-Wunused-parameter` under `[CG-C-1]`.

**Not done:** `dyn I` and vtables (`[TYP-22]`); generics, so a generic
interface such as `Add[Rhs]` is only usable at one concrete type; associated
types and consts (`[IFC-4]`); `[TYP-20]`'s orphan rule and `[IFC-2]`'s
cross-package restriction, which need modules — block F; `pub`/`pub(read)`
visibility, which is unenforceable while everything is one module. `a += b`
still expands to `a = a + b` rather than looking for `add_assign`.

A note on the specification, now ERR-008: **Appendix A's `match` example did
not parse.** It wrote `Circle(r) => return PI * r * r`, but Part III's grammar
makes `return` a statement while `[GRM-10]`'s `=>` takes an expression — and
Part IV §2 gives `return` the type `!`, siding with the appendix. The owner
ruled the grammar wins (2026-09-08); the appendix is corrected and the entry
records what would change if that is ever revisited.

## Next: Phase 1 block E

`String`/`Array`, f-strings, `Option`/`Result`, `?`. `Option` and `Result` are
ordinary payload enums now that payload enums work, and `?` lowers to a `match`
per `[ERR-2]`. `String` and `Array` need allocation, which `ember_rt` already
provides.

Then F (modules across files, `const`, `static`) and G (the formatter).

Two things from earlier blocks land naturally in E: `[T; N]` coercing to
`Span[T]`, and `for` over anything other than a range — both need the
standard-library types this block introduces, plus the `Iterator` interface
that block D's machinery can now express.

## Environment

- Rust 1.98.1 stable, MSVC toolchain. Installed 2026-09-07 via winget.
- LLVM 22.1.8 at `C:\Program Files\LLVM`; `libclang.dll` in its `bin`. Not
  needed until Phase 5, but installed so that phase does not stall.
- MSVC 14.44 under
  `C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools`.
- `cmake` and `ninja` exist only under the Visual Studio install, not on
  `PATH` — the same trap RageV's build has.
- `cargo` is on `PATH` only in shells started after the install; PowerShell
  calls in this project prepend the machine and user `Path` values.
