# Ember — handoff

## Read this before anything else: a new specification is coming

**The owner is supplying a new design document, v0.5.** Said on 2026-09-08,
after the v0.2 memory-safety amendment had been merged: it "makes some
substantial changes to the design so a few areas will have to be revisited."

**It has not arrived yet.** Nothing in this file describes it, because nothing
about it is known beyond that sentence. When it arrives, apply it before
writing any more Phase 2 code — the procedure and the things it must not
overwrite are in "Applying a new specification version" below.

**Do not start block E, F, I or the rest of G while waiting.** The owner did
not say to stop, so this is judgement rather than an instruction, and an
explicit request for work overrides it. The reasoning: the NLL borrow checker
is the largest single piece left in Phase 2, the v0.2 amendment already changed
what it has to produce (`[DIA-7]`'s classifier needs the borrow checker's own
loan and region tables), and a v0.5 that revisits ownership at all would mean
building it a second time. Waiting costs nothing; the work is committed and
pushed.

**One question is with the owner and unanswered:** is v0.5 the *whole*
document, or another partial replacement? v0.2 was one 2,671-line file;
the 2026-09-08 amendment replaced six parts wholesale and left the rest alone.
The answer decides whether `docs/spec/` is regenerated or patched.

**Everything is committed and pushed** (`58cb057` on
`origin/phase-1-core-language`), so a rewrite costs time and nothing else.

### Where the owner's source documents live

Outside the repository, in `C:\Users\ism19\Downloads\`:

| File | What it is |
|---|---|
| `Ember_Design_Document_v0.2_Implementation_Spec.md` | the original 2,671-line specification; `docs/spec/` is this, split |
| `Ember_Updated_Parts_Memory_Safety.md` | the 2026-09-08 amendment: Parts IX, XI, XV, XIX, XX, XXII |
| `Ember_Part_XIX_Toolchain.md` | an earlier Part XIX the owner withdrew — **superseded, do not apply** |

None of them is in the repository. If that folder is cleared, the ability to
re-split or to diff `docs/spec/` against what the owner actually sent goes with
it. Worth proposing to the owner that they be vendored under `docs/spec/source/`;
not done unasked, because what belongs in the repository is their call.

## Applying a new specification version

The procedure that worked for the 2026-09-08 amendment, in order:

1. **Split the owner's file per part** into a scratch directory —
   `awk` on `^# Part ` / `^# Appendix ` headings, or `tools/split_spec.py` if
   the document is complete. Never overwrite `docs/spec/` from it directly.
2. **Diff each part against the committed one** before changing anything,
   ignoring blank lines and trailing space. The diff is the specification of
   the work; read all of it before applying any of it.
3. **Apply, then diff again** and confirm the only remaining differences are
   ones you can name.
4. **Record every difference you deliberately kept** in `docs/spec-errata.md`,
   against the entry that decided it. A future reader will diff the owner's
   file against `docs/spec/` and must find an explanation for each one.
5. **Watch the line endings.** `.gitattributes` pins LF, `core.autocrlf` is
   `true` locally, and most working-tree files are CRLF while every committed
   blob is LF. A scripted edit can leave a file *mixed*, which is the state
   that makes later scripted edits fail silently. Normalise anything you touch
   and check `git diff --stat` shows only real changes.

**Owner rulings a new document will probably contradict, because they postdate
it.** Each is *decided*; keep the ruling, not the document, and say so in the
errata:

- **`#$`, not `#!`, for test annotations** (ERR-006). `#!` is `[MOD-6]`'s
  language directive and the lexer cannot tell them apart. Every `.em` file
  under `tests/` uses `#$`, as do Part XIX §5 and Part XX §3's milestones.
  `[TST-0]` and the `assert-c` annotation came in with it.
- **`return` is a statement, so `Circle(r) => return ...` does not parse**
  (ERR-008). Appendix A's `match` example is corrected to the `:` form.
- **`E0102`/`E0103`/`E0104`**, not `E0010`/`E0011`/`E0020` (ERR-001).
- **A doc comment that documents nothing is silent**, no `W0001` (ERR-007).

## Start here

**Phases 0 and 1 of nine are complete. Phase 2 is in progress.** All seven of
Phase 1's blocks are done. Phase 2's blocks A to C — moves, drops and drop
flags, generics, and iterators — are done, and so is the core of block G:
`unsafe` blocks, raw pointers and the memory builtins. That is what unblocked
block D, which is now writable but not written. The borrow checker and
closures are not started.

**The owner amended the specification on 2026-09-08** — the v0.2 memory-safety
update. It adds `Cell`/`RefCell` and a mandatory borrow-diagnostic catalogue,
and both land *inside* Phase 2, which grew two exit criteria. Read
"The 2026-09-08 spec amendment" below before planning any more of this phase.

Read `docs/spec/` (the specification, split by part) and `docs/DECISIONS.md`
(the nine owner decisions) before touching anything. `docs/spec-errata.md`
lists eight places where the specification is silent or contradicts itself.
Five have been ruled on by the owner and patched into `docs/spec/` (ERR-001,
ERR-005, ERR-006, ERR-007, ERR-008); the other three are still proposals.

| | |
|---|---|
| Repository | `https://github.com/Insomniac-Coder/ember.git` |
| Pushed | `95f3269` on `origin/main` — all of Phase 0 |
| Working branch | **`phase-1-core-language`** — all of Phase 1, Phase 2 blocks A-C and G's core, pushed at `58cb057`. Not merged to `main`; the owner has not decided when. |
| Tests | `cargo test --workspace` → **151 passed, 0 failed**; 31 `.em` programs under `tests/` |
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

**A statement that writes a place, but is not `Assign`, was invisible to the
move analysis.** `[TYP-8]`'s checked arithmetic lowers to
`StmtKind::CheckedBinaryOp`, which `drops.rs`'s `step` did not handle, so the
destination was never marked live and the next read of it was reported as a
use after move. Every integer `bigger = cap * 2` bound to a local hit it;
floats did not, because they take the plain `Assign` path, and nothing in the
suite wrote that shape until block G's tests did. **When a new MIR statement
writes a place, `drops.rs` has to learn it** — the analysis fails closed, and
failing closed here means rejecting a correct program.


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
  whole language rather than only the part with rules. **That catch-all is not
  as safe as it reads**, and it cost two defects later in Phase 2: it trims
  each copied line, so a statement holding a block — `unsafe:` — came out with
  its body at the parent's indentation. A new statement or item kind needs its
  own arm, not the catch-all.
- **The printer knew nothing about type parameters** until Phase 2 block G's
  tests were written: `struct Pair[A, B]` printed as `struct Pair`. Generics
  arrived in Phase 2, after this block, and nothing re-checked the formatter
  against them, so `[FMT-1]`'s round trip was false for every generic
  declaration in the corpus. `generics()` now prints them for `fn`, `struct`,
  `enum`, `interface`, `extend` and `type`.

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
| **B** | generics with bounds, monomorphisation, associated types, generic structs | **done** |
| **C** | `Iterator` and `for` over anything | **done; the adaptor set is not written** |
| D | `Array`, `Span`, `MutSpan`, `Box`, `Map` written in Ember | **unblocked; not written** |
| E | the NLL borrow checker (§4.7), two-phase borrows, `@view` | not started |
| F | closures (`[CLO-*]`), `Callable`, `fn(A)->R` parameters | not started |
| **G**, core | `unsafe:` blocks, `*T`/`*mut T`, `alloc`/`free`/`read`/`write`/`size_of` | **done** |
| G, the rest | arenas (`[ARN-*]`), `MaybeUninit`, `transmute`, pointer arithmetic, inline asm | not started |
| H | the §XIX.6.1 shape catalogue, the classifier, `ember explain --borrow` | not started |
| I | `Cell`, `RefCell`, `Ref`/`RefMut` (`[CELL-*]`), `std.cell` | not started — **added by the 2026-09-08 amendment** |

Blocks D and G were planned the other way round. G's core was pulled forward
because D could not be written without it, and the rest of G — arenas,
`MaybeUninit`, `transmute` — turned out not to be needed for D at all.
Block I did not exist until the amendment.

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

### Block G's core, and block D unblocked

`Array`, `Span`, `MutSpan`, `Box` and `Map` could not be written in Ember
without raw pointers and `unsafe`. Those now exist, so D is writable. What D
was worth in the meantime was already delivered: `for x in xs` over an
`Array[T]` works, and it is a counted loop rather than an iterator object.

**`unsafe:` is a block, and `[UNS-1]` is enforced.** Using a raw pointer, or
any of the memory builtins, outside one is `E3100`, whose `help` names the
fix. The typechecker carries an `in_unsafe` flag; `[UNS-2]` holds — nothing
inside an `unsafe` block skips type checking, bounds checking or the move
analysis.

**The builtins are `alloc`, `free`, `read`, `write` and `size_of`**, each
generic in the pointee type. MIR lowering picks `arg_ty` per builtin rather
than from the first argument, which was a real bug: `alloc`'s first argument
is a count, so taking the type from it made every allocation `usize`-sized.

**Generic structs came with it.** Only generic *functions* existed before, and
D needs `Buffer[T]`-shaped types. `StructDef` gained an `origin` — the generic
it was instantiated from, and the arguments used — because without it
`Buffer[T]` in a method signature would not unify with the `Buffer[i32]` at
the call site. `unify` and `substitute_ty` both consult it.

**D is proved unblocked, not assumed.** A growable collection is written
entirely in Ember — allocate, grow by doubling, copy across, index, free — and
runs correctly, with the runtime's allocation counter reporting 3 allocations,
3 frees and nothing live at exit.

**Three defects, all found by writing the tests rather than by reading the
code.** None was visible while the demo programs ran:

1. **Methods on a generic struct were silently dropped.** Collection kept the
   fields and ignored every other member, so `Pair[A, B]` with a `fn left`
   produced an instantiation with no `left`, and the call site said
   `Pair_i32_f32 has no method named left` — an error at a distance from the
   cause. `GenericStruct` now stores each method resolved once with the
   parameters opaque, and `instantiate_struct` substitutes and registers them,
   queueing the bodies on `pending_methods` the way generic functions queue on
   `pending`. This mattered more than it looked: `Array[T].push` is a method,
   so block D was not really unblocked without it.
2. **The formatter dropped every type parameter list.** `struct Pair[A, B]`
   printed as `struct Pair`, and `fn make[T](...)` as `fn make(...)`. The
   formatter was written in Phase 1, before generics existed, and nothing had
   re-checked it since. `[FMT-1]`'s `parse(fmt(x)) ≡ parse(x)` was false for
   every generic declaration in the corpus.
3. **The formatter flattened `unsafe:` blocks.** `StmtKind::Unsafe` had no arm,
   so the catch-all copied the text through with each line trimmed, which put
   the body at the parent's indentation.

The third defect is the general shape of the second: the formatter's catch-all
looks safe but is not — it preserves text and destroys structure. Any new
statement or item kind needs an arm.

**Not done in G:** arenas (`[ARN-*]`), `MaybeUninit`, `transmute`, pointer
arithmetic (`offset`/`add`/`sub`), and inline assembly (`[UNS-6]`). None of
them is needed for D. `L3010 unsafe block larger than necessary` (`[UNS-3]`)
is a lint, and no lint pass exists yet.

**One limit worth knowing.** A generic struct's method body is checked only
when something instantiates it — there is no opaque `Buffer[T]` for `self` to
have, so unlike a generic function it is never checked once with the
parameters left abstract. A method nobody instantiates is never type-checked,
which is C++'s behaviour rather than Rust's. The first instantiation of each
method reports; later ones are quiet, so one mistake is one error.

**Still open from blocks A to D:** partial moves (moving one field out of a struct)
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

## The 2026-09-08 spec amendment

The owner replaced Parts IX, XI, XV, XIX, XX and XXII wholesale with the v0.2
memory-safety update. Two things were added, and both are Phase 2 work:

**1. `Cell` and `RefCell` (`[CELL-1..11]`, new Part IX §7).** The escape hatch
for value types, matching what classes already give the object world. `Cell[T]`
requires `T: Copy`, hands out no reference, and therefore needs no runtime
check at all — `get` is a load, `set` is a store. `RefCell[T]` keeps a
one-word borrow counter and panics on conflict, naming the line of the
conflicting borrow. Both are `!Sync`. Neither is in the prelude (`[CELL-11]`),
so reaching for one is a visible import — that is open question 9, awaiting the
owner. They live in `std.cell`, a new module row in Part XV.

**2. A mandatory borrow-diagnostic catalogue (`[DIA-7..10]`, new §XIX.6.1).**
Sixteen error shapes — O1–O4, B1–B10, X1, R1 — each with a *required* `help`
line naming a concrete API. `[DIA-7]` makes classification mandatory: an
ownership or borrow error the classifier cannot place is written to
`target/<profile>/unclassified-borrow-errors.log`, and CI fails if the
conformance suite produces even one. `[DIA-9]` forbids suggesting `unsafe`,
`Cell`, `RefCell`, `Shared` or `clone()` first when a structural fix exists;
`clone()` leads only for O1. `[DIA-8]` adds `ember explain --borrow
<file>:<line>`, which must be generated from the borrow checker's own loan and
region tables, not reconstructed after the fact.

**What that does to the plan.** Phase 2's exit criteria in §XX.2 now also
require all `[CELL-*]` and `[DIA-7..10]` tests, a `tests/ui/borrow/<shape>/`
snapshot for each of the sixteen shapes, and **zero unclassified borrow errors
across the whole corpus**. `[DIA-7]` is the constraint that matters: it has to
be designed into the borrow checker (block E) rather than bolted on in block H,
because the classifier needs the loan and region tables that E builds. Building
E without it means building E twice.

`L3011 RefCell guard held across a call` joined the lint list, and the error
registry's `E1000–E1499` row now names `E1050`/`E1051` — both already exist.

**Three things in the update were deliberately not taken**, all because they
predate owner decisions already recorded in `docs/spec-errata.md`: it reverts
test annotations from `#$` to `#!` in Part XIX §5 and in the §XX.3 milestones
(ERR-006), and it carries Appendix A's `match` example back to the `=>` form
that does not parse (ERR-008). It also drops `[TST-0]` and the `assert-c`
annotation, which came in with ERR-006. Diffing the owner's file against
`docs/spec/` will show exactly those differences and nothing else.

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
