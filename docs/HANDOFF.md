# Ember — handoff

## Start here

**Phase 0 of nine is complete.** Its exit criterion — `hello.em` compiles via C
and runs — passes, and milestone M1 passes as an automated test. Phase 1 (the
core language: enums, `match`, interfaces, modules, `String`/`Array`, overflow
semantics, the formatter) has not been started.

Read `docs/spec/` (the specification, split by part) and `docs/DECISIONS.md`
(the nine owner decisions) before touching anything. `docs/spec-errata.md`
lists seven places where the specification is silent or contradicts itself, and
what the implementation does about each. Four have been ruled on by the owner
and patched into `docs/spec/` (ERR-001, ERR-005, ERR-006, ERR-007); the other
three are still proposals.

Committed and pushed: `aa38277` on `origin/main` at
`https://github.com/Insomniac-Coder/ember.git`.

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

## State

| Crate | State |
|---|---|
| `ember_span` | `FileId`, `Span`, `SourceMap`, `Symbol` interner |
| `ember_diag` | model, renderer matching XIX §6, JSON, 90-code registry |
| `ember_lexer` | all of Part II |
| `ember_ast` | full Part XVIII §2 tree, structural dump |
| `ember_parser` | **full v1 grammar** |
| `ember_types` | interner, C layout, `Copy`/`needs_drop`/`is_view`/FFI safety |
| `ember_hir` | typed, desugared tree; scalar and struct cases populated |
| `ember_typeck` | bidirectional checking, literal defaulting, coercion sites |
| `ember_mir` | CFG, lowering, unreachable-block pruning, verifier |
| `ember_codegen_c` | MIR to C11, `#line` directives, shortest round-trip floats |
| `ember_build` | MSVC (via captured `vcvars64`) / clang / gcc, `target/` layout |
| `ember_driver` | `build`, `run`, `check`, `explain`, `--emit`, `--json` |
| `runtime/ember_rt` | C11: alloc, panics, printers, embedding API |

`cargo test --workspace`: **146 passed, 0 failed.** `cargo build --workspace`
is warning-free.

The emitted C compiles warning-free under `clang -std=c11 -Wall -Wextra` and
under MSVC `/W4` — one level stricter than `[CG-C-1]`'s `/W3`.

## What works today

```bash
ember run examples/hello.em
ember build file.em --emit tokens|ast|hir|mir|c
ember check file.em
ember explain E3040
```

Structs with C layout, memberwise constructors, field access, functions with
the three parameter modes, locals with inference, `if`/`elif`/`else`, `while`,
`break`/`continue`, scalar arithmetic with `[TYP-4]`'s no-implicit-conversion
rule, `[TYP-5]` widening at coercion sites, untyped-literal defaulting,
short-circuit `and`/`or`, casts, and `println` for every scalar and `str`.

## What was over-delivered, deliberately

Phase 0 asked for "`ember_parser` for functions, calls, literals, `if`/`while`/
`for`, structs (no generics)". The parser instead implements the **whole v1
grammar** — classes, enums, interfaces, `extend`, generics, `where` clauses,
patterns, `match`, `with`, `defer`, closures, f-strings, the full precedence
table. Writing a subset now and the rest in Phase 1 is the same work twice, and
Phase 1's exit criteria demand the full grammar anyway. The parser tests
already parse the specification's own example program from Part I §4 unchanged.

The lexer is likewise complete rather than "the indentation algorithm plus
enough to get by", and MIR exists as a real CFG with a verifier rather than
being skipped in favour of generating C straight from HIR.

## Load-bearing invariants

**`Span` offsets are always absolute byte offsets into the normalised file.**
`ember_span::normalise` strips a BOM and folds CRLF to LF before anything else
sees the text. `lex_range` (used for f-string interpolations) truncates the
source at `end` rather than slicing from `start`, precisely so that spans stay
absolute and a diagnostic inside `f"{x.y()}"` lands on the real line.

**The `Symbol` interner leaks, on purpose.** `Box::leak` buys a `&'static str`
with no unsafe code and no lifetime plumbing through every stage. A compiler
process interns a bounded set of names and exits.

**`workspace.lints.rust` denies `unsafe_code`.** The compiler enforces Ember's
safety story; it should not have memory errors of its own.

**The parser's cascade limit is per *region*, not per file** (`[AST-2]`). A
region ends when the cursor moves past where the last error was reported. If a
recovery path ever fails to advance the cursor, the limit silently swallows
every later diagnostic in the file — so every recovery loop checks
`if self.pos == before { self.bump() }`.

**`referenced_blocks` in the C backend must agree exactly with
`emit_terminator`'s fallthrough rule.** A jump to the next block in order emits
no `goto`, so it must not mark that block as referenced. If the two drift, the
emitted C either has an unreferenced label (a warning, breaking `[CG-C-1]`) or
a `goto` to a label that was never emitted (a hard error).

**MIR lowering creates a fresh block after every `return`, `break` and
`continue`.** Most are unreachable and are pruned by `prune_unreachable` before
codegen. Without the prune the C is full of dead labels.

## Traps paid for

**Shell escaping through three layers destroys Rust escape sequences.** Writing
Rust source containing `\n`, `\\` or `\"` through a bash heredoc into a Python
string mangles them into real newlines and tabs, and the damage is not visible
in a diff — it looks like the file was always that way. `scan_escape` had to be
rewritten twice. Write files containing escape sequences with the file tool
directly, never through a shell.

**Part II §4's keyword table has 47 entries, not 46.** Seven per row for six
rows, five on the seventh. `ember_lexer::token` asserts the count so the table
and the enum cannot drift apart.

**`Range ..` is a prefix of `Range ..=`.** A substring count over the AST dump
silently over-counts. Test assertions against the dump match to end of line.

**`cl.exe` cannot find its own headers without `INCLUDE` and `LIB`, and there
is no flag that substitutes for them.** `ember_build` runs `vcvars64.bat`
through `cmd /c … && set` and captures the environment. If `INCLUDE` is absent
from the captured set, the batch file did not really run and MSVC detection
reports "not found" rather than failing later with "cannot open stdio.h".

**`clang.exe` is not on `PATH`** even with LLVM installed — the Windows
installer does not add it. `Toolchain::find_on_path` falls back to
`C:\Program Files\LLVM\bin`.

## Next: Phase 1 — the core language

Exit criterion: conformance for Parts II–VI except closures and generics.

1. Enums and `match` with Maranget decision trees, exhaustiveness (`E2090`) and
   redundancy (`W2091`). `Option`/`Result` become real enums; `?` lowers.
2. Interfaces without generics, operator interfaces, method resolution
   (`[TYP-24]`), `extend`, and the auto-ref/deref adjustments HIR already has a
   slot for.
3. Modules and imports across files — the first time more than one translation
   unit exists, which is what `[BLD-1]`..`[BLD-4]`'s build graph is for.
4. Integer and float semantics in full: `[TYP-8]` overflow policy per profile,
   `[TYP-10]` shifts, division and remainder panics, `Assert` lowering in MIR
   and `ember_panic_*` in the C.
5. `String`/`str` with SSO and `Array[T]` as compiler-known types, replaced by
   the generic `std` implementations in Phase 2.
6. `with`, `defer`, labelled `break`, `for`/`while` `else`, f-strings that
   actually format.
7. The formatter, with `[FMT-1]`'s idempotence tested over the whole corpus.

## Environment

- Rust 1.98.1 stable, MSVC toolchain. Installed 2026-09-07 via winget.
- LLVM 22.1.8 at `C:\Program Files\LLVM`; `libclang.dll` in its `bin`. Not
  needed until Phase 5, but installed so that phase does not stall.
- MSVC 14.44 under
  `C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools`.
- `cmake` and `ninja` exist only under the Visual Studio install, not on
  `PATH` — the same trap RageV's build has.
- `cargo` is on `PATH` only in shells started after the install.
