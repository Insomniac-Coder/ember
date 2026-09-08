# Ember — handoff

## Start here

**Phases 0 and 1 of nine are complete. Phase 2 is in progress.** All seven of
Phase 1's blocks are done. Phase 2's blocks A to C — moves, drops and drop
flags, generics, and iterators — are done; the borrow checker, closures and
arenas are not started, and the collections in Ember are blocked on them.

Read `docs/spec/` (the specification, split by part) and `docs/DECISIONS.md`
(the nine owner decisions) before touching anything. `docs/spec-errata.md`
lists eight places where the specification is silent or contradicts itself.
Five have been ruled on by the owner and patched into `docs/spec/` (ERR-001,
ERR-005, ERR-006, ERR-007, ERR-008); the other three are still proposals.

| | |
|---|---|
| Repository | `https://github.com/Insomniac-Coder/ember.git` |
| Pushed | `95f3269` on `origin/main` — all of Phase 0 |
| Working branch | **`phase-1-core-language`** — all of Phase 1 and Phase 2 blocks A-C, pushed |
| Tests | `cargo test --workspace` → **151 passed, 0 failed** |
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

Seven blocks, dependency-ordered. All are done.

| | Block | State |
|---|---|---|
| **A** | numeric semantics `[TYP-4..10]`, `Assert` lowering, definite-init | **done** |
| **B** | tuples, fixed arrays, enums, `match` with exhaustiveness | **done** |
| **C** | `for` over ranges, loop `else`, labelled break, `with`, `defer` | **done** |
| **D** | interfaces, operator interfaces, method resolution, `extend`, visibility | **done** |
| **E** | `String`/`Array`, f-strings, `Option`/`Result`, `?` | **done** |
| **F** | modules across files, `const`, `static` | **done** |
| **G** | formatter (`[FMT-1]`) | **done** |

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
| `ember_fmt` | `[FMT-1]`'s canonical printer, comments preserved |
| `ember_driver` | `build`, `run`, `check`, `fmt`, `explain`, `--emit`, `--json`; module loading |
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

### Block E: `Option`, `Result`, `?`, `Array`, `String`, f-strings

- **`Option[T]` and `Result[T, E]` are synthesised payload enums**, one per set
  of type arguments, named `Option_i32` and so on. They reuse all of block B —
  the `{tag, union}` layout, `match`, and exhaustiveness — so nothing new was
  needed to pattern-match them. `Some`/`None`/`Ok`/`Err` take the expected type
  when there is one; a bare `None` with no expectation is `E2060`, because
  nothing says which `Option` it is.
- **`?` becomes a two-arm `match`**, one arm yielding the payload and one
  returning the failure — which is why HIR lets a `match`'s arms mix a value
  and a block even though `[GRM-10]` forbids that in source.
- `[ERR-2]`'s `F.from(e)` conversion is **not** applied: the failure types must
  match exactly, because `From` needs generics. The diagnostic says so.
- **`Array[T]` and `String` are one runtime buffer** — pointer, length,
  capacity — with the element size passed at each call. That is what lets a
  single `ember_vec_push` serve every element type without generics, which is
  exactly what Part XX.1 asks for ("a compiler-known type temporarily"). `String`
  is `Vec { elem: u8 }`, so it prints as `String` and shares every growth path.
- **`push` needed a spill.** `ember_vec_push` copies through a pointer, and
  `&10` is not C, so the pushed value is lowered into a local first.
- Indexing an `Array[T]` is bounds-checked against the runtime length, using
  the same `AssertKind::Bounds` a fixed array uses — its `len` operand was
  already an operand rather than a constant, so nothing had to change.
- **f-strings build a `String`**, appending each piece through
  `ember_fmt_*`, chosen from the value's type exactly as `println`'s printer
  is. A format spec (`{x:.2}`) is reported as unsupported rather than ignored.
- `for i in 0..xs.len()` now works: either end of a range may be an untyped
  literal, and the other end says what it should be.

**Not done:** small-string optimisation (`String` is always heap); `Array`'s
`pop`, `insert`, `remove`, iteration by `for x in xs` (that needs `Iterator`);
freeing — **nothing calls `ember_vec_free`, so every `Array` and `String`
leaks.** Drop elaboration is Phase 2's, and this is the first type that needs
it; `ember_vec_free` exists and is ready for it.

### Block F, half: `const` and `static`

`const NAME: T = literal` and `static NAME: T = literal` both become a value
substituted where the name is used. `[STA-2]` says there are no runtime
initialisers, so a non-literal is `E2130` — which is what Part XX.1's
"comptime-initialised only via literal for now" asks for. A `const` may be an
array length, which closes the gap `E2131` left in block B.

`static mut` is rejected: `[STA-1]` requires `unsafe` for any access to one.
With no mutation and no runtime initialiser, a `static` and a `const` behave
identically here; the stable address `static` promises is not observable until
references to globals exist.

### Block F, the rest: modules across files

- **Every declared name is stored qualified** — `math.ops.Point` — and each
  module carries a map from what it may write to what that means. Two modules
  may each declare a `helper`; the mangled C symbols carry the module, so
  nothing collides.
- **Names are declared for every module before any import is bound**, because
  `[MOD-4]` allows cycles inside a package: an import may name an item in a
  module that has not been walked yet.
- `[MOD-1]` maps `a.b.c` to `a/b/c.em`, or to `a/b/c/mod.em` when the directory
  has submodules. The root is the directory of the file named on the command
  line, until `ember.toml` is read.
- `[MOD-2]` is enforced: importing a private item is `E1020`.
- **`import a.b.ops` then `ops.add(x)` parses as a method call**, because the
  parser cannot know `ops` is a module — the same shape `Shape.Circle(1.0)`
  has. Both are settled in the checker.
- **A `DefId` is no longer a position in `Program::functions`.** Functions are
  gathered module by module, then methods, then interface defaults, so the
  lookup searches by identity. It is linear; if it ever shows up in a profile,
  a map is the fix.
- Functions are **no longer emitted `static`**. Modules share one translation
  unit, so a private helper another module never calls was an unused `static`
  function — a warning, and `[CG-C-1]` forbids those.

### Block G: the formatter

`ember fmt [--check|--write]`, in `compiler/ember_fmt`.

- **Comments were not the obstacle they looked like.** `[II.7]` keeps them out
  of the token stream, but Phase 0's lexer already records each one's span and
  whether it had a line to itself — written for exactly this. The printer
  flushes them in span order as it passes, so a comment above an item stays
  with that item.
- **The blank line comes before the comment**, not after: separation first,
  then whatever introduces the next item. Getting that backwards detaches every
  comment from what it describes, which is what the first version did.
- `[FMT-1]`'s two guarantees — `fmt(fmt(x)) == fmt(x)` and `parse(fmt(x)) ≡
  parse(x)` — are a test over the whole corpus, not an assertion in a comment.
- **Anything the printer has no rule for is copied from the source.** A lambda,
  an f-string, a `match` expression: reproducing one from the tree risks
  changing what it means, and copying keeps parse-equivalence true for the
  whole language rather than only the part with rules.

**Not done:** width-based breaking inside expressions. A long call or condition
stays on one line; only a long parameter list breaks. `[FMT-1]` asks for
conditions broken after `and`/`or` and trailing commas in multi-line element
lists, which needs a proper Wadler-style layout algebra rather than the
string-building printer this is.

## Phase 2 — where it stands

Part XX's Phase 2 is ownership: generics, drops, the borrow checker, closures
and arenas. Split the same way Phase 1 was:

| | Block | State |
|---|---|---|
| **A** | moves, `Copy`, drop elaboration, drop flags, `E3040` | **done** |
| **B** | generics with bounds, monomorphisation, associated types | **done** |
| **C** | `Iterator` and `for` over anything | **done; the adaptor set is not written** |
| D | `Array`, `Span`, `MutSpan`, `Box`, `Map` written in Ember | **blocked on G** — see below |
| E | the NLL borrow checker (§4.7), two-phase borrows, `@view` | not started |
| F | closures (`[CLO-*]`), `Callable`, `fn(A)->R` parameters | not started |
| G | arenas, `unsafe`, raw pointers, `MaybeUninit`, `transmute` | not started |
| H | diagnostics pass on borrow errors, `ui/` snapshots | not started |

### Block A: moves and drops

**The `Array`/`String` leak is closed, and measured rather than asserted:** the
runtime's own allocation counter reports 204 allocations and 204 frees over
`tests/run-pass/drops_and_moves.em`, with nothing live at exit.

- `StmtKind::Drop { place, flag }` is inserted by lowering at every way out of
  a scope — the end of the block, a `return`, and a `break` or `continue` that
  leaves it — in reverse declaration order (`[DRP-2]`), after that scope's
  `defer` blocks (`[CTL-8]`).
- `compiler/ember_analysis/src/drops.rs` then runs the `Live | Moved | Maybe`
  dataflow: a drop of a moved value becomes a `Nop`, and a drop of a `Maybe`
  value gets `[OWN-3]`'s **drop flag**, a hidden `bool` set where the value is
  stored and cleared where it is moved away. A conditional move is never an
  error.
- **A value can arrive from a call**, whose destination is written by a
  terminator rather than a statement — `xs = Array()` is one — so the flag is
  set at the top of the block control reaches next. Missing that leaked exactly
  the values that were never moved.
- **`[FN-1]`'s borrow is the default, and reading a place for a borrowed
  argument must not consume it.** `xs.len()` was moving `xs` away, because a
  non-`Copy` place always read as a move; the drop was then removed as
  "already moved" and the buffer leaked. `lower_operand_borrowed` is the fix.
  An `owned` parameter still needs the mode threaded through — nothing uses one
  yet.
- The drop glue is written straight into the C rather than as a synthesised
  function: a struct drops its fields in reverse, an enum switches on its tag
  and drops the active variant's payload, an array drops its elements.

### Blocks B and C: generics, bounds, associated types, `for`

- **A generic body is checked once, with its parameters opaque** (`TyKind::Param`),
  which is what `[TYP-17]` demands: only what the bounds provide is available,
  and there is no duck typing. `a.area()` inside `fn f[T]` is an error even
  when every caller happens to pass something with an `area` — and the
  diagnostic names the bound that would fix it (`help: add the bound: T: Shape`).
- **Nothing is emitted for the generic body itself.** Each call instantiates
  it: the arguments are unified against the declared parameter types
  (`[TYP-18]`), the bounds are checked, and the body is re-checked with the
  parameters bound to the concrete types, producing its own `DefId` and symbol
  (`[MONO-1]`). Instantiations are cached by `(DefId, args)` and drained from a
  queue, because instantiating one can reach another.
- Errors from an instantiation go to a **discarded sink** — the generic body
  was already checked once, and reporting the same mistake per instantiation
  would bury it.
- `f[i32](x)` parses as a call on an index, which is where explicit
  instantiation is picked up. `[TYP-18]` requires it when no argument mentions
  the parameter, and that is `E2060` with the shape to write.
- **`[IFC-4]` associated types** are `TyKind::Assoc { name }`, in scope while an
  interface declaration is read and resolved once the receiver is concrete
  through `assoc_values[(type, name)]`. That is what lets `Iterator` say
  `fn next(mut self) -> Option[Item]` and an implementation say `type Item = i32`.
- **`[CTL-1]`'s `for`** now has three shapes: a range is still a counted loop
  (`[CTL-3]`); an `Array[T]` is a counted loop over its indices, so iterating a
  collection also costs no iterator object; anything else is driven through
  `next()`. Exhaustion ends the loop through its condition rather than a
  `break`, so `[CTL-4]`'s `else` still tells the two apart.

**Not done in B and C:** const generics (`E1010` with a clear message); generic
structs and enums — only functions are parameterised, so `Pair[A, B]` is not
yet writable; generic interfaces (`Add[Rhs]`); the `Iterator` adaptor set
(`map`, `filter`, …), which needs closures — block F.

### Block D is blocked on block G

`Array`, `Span`, `MutSpan`, `Box` and `Map` cannot be written in Ember until
raw pointers, `MaybeUninit` and `unsafe` exist, which is block G. Generics are
in place, so the type signatures are now expressible; the bodies are not. **Do
G before D.**

What D was worth in the meantime is delivered: `for x in xs` over an `Array[T]`
works, and it is a counted loop rather than an iterator object.

**Not done in this block:** partial moves (moving one field out of a struct)
are not tracked — only whole locals; `mem.take`/`replace`/`swap`/`forget`
(`[OWN-6]`); `[OWN-4]`'s `E3041` for a loop that moves a value declared
outside it; and user-defined `drop` methods are recognised (`has_drop`) but
their bodies are not yet called by the glue.

**What Phase 1 left open, and where each belongs:**

- **`dyn` and vtables** (`[TYP-22]`), which block D deliberately left out.
- **`[T; N]` coercing to `Span[T]`**, and **`for` over anything but a range** —
  both need `Iterator`, which block D's interface machinery can express but
  nothing has written.
- `[ERR-2]`'s `F.from(e)` conversion for `?`, which needs `From`, which needs
  generics.
- `@overflow(saturate)` (block A), `[TYP-13]`'s niche optimisation, slice and
  range patterns, `from_repr`, small-string optimisation, `a += b` looking for
  `add_assign`, and width-based breaking in the formatter. Each is written up
  in its block's section above.
- **`pub` visibility is enforced across modules but not within one.** Nothing
  checks `pub(package)` yet, because there is one package.

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
