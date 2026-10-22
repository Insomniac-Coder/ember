# Helper AC — tooling and the build (2026-10-22)

Rules: `[BLD-1]`, `[BLD-8]`, `[BLD-7]`, `[BLD-4]`, `[CLI-21]`, `[IMP-11]`, `[IDE-3]`, `[IDE-4]`,
`[IDE-6]`, `[TST-5]`, `[DOC-2]`, `[DOC-4]`. Branch `helper-AC`, from main `f00f2d61`.

Order of work: `[IMP-11]`'s pieces that need nothing else (the verifier after every pass, the
fuzzers, `[FFI-5a]` with every compiler), then `[CLI-21]` (and `[IMP-11]`'s evaluator runs), then
the editor rules, the debugger session, the guide, and the build rules.

## Tools on this machine

- `ninja`: not on `PATH`; Visual Studio Build Tools ship one
  (`C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\Common7\IDE\CommonExtensions\Microsoft\CMake\Ninja\ninja.exe`).
  WSL has `/usr/bin/ninja`.
- `cdb`: not installed. `gdb`: not installed in WSL. `lldb`: `C:\Program Files\LLVM\bin\lldb.exe`.

## `[IMP-11]` (done)

### The MIR is verified after every pass (done)

- `ember_mir::verify::Pipeline` holds a program's MIR from lowering to code generation. The bodies
  are changed only through `Pipeline::pass(name, types, |bodies, types| …)`; in a debug build of
  the compiler (`debug_assertions`: the `ci` profile the tests use, and `dev`) every body is
  verified after each pass: the structural rules (`verify`), literals decided
  (`verify_concrete_literals`), views made by a borrow (`verify_views`), interface upcasts
  complete (`verify_interface_upcasts`), and a violation's panic names the pass
  (`MIR verification failed after loop versioning: …`). Reading is `Pipeline::bodies()`; a pass
  that changes bodies outside `pass` does not compile, so a pass added later is verified too.
- `compile` (`ember_driver/src/main.rs`) runs its 40-odd passes through it, from lowering to
  `for_codegen`; so does the compile-time evaluator's program (`comptime.rs`). The scattered
  `verify_all`/`verify_views_all` calls it replaces are gone. A release compiler skips the
  checks (`for_codegen`'s unconditional boundary check is unchanged).
- Test: `ember_mir` `the_mir_is_verified_after_every_pass_and_a_bad_one_is_named` (a pass that
  jumps to no block, a pass that makes a view with a cast: each stopped, named). The whole
  conformance tree and the four test trees pass with the checks on (4 min).

### Fuzzing the lexer and the parser (done)

`compiler/ember_parser/tests/fuzz.rs`: fixed seed, fixed counts (`EMBER_FUZZ_CASES` multiplies,
`EMBER_FUZZ_SEED` changes the seed, `EMBER_FUZZ_TRACE` prints each case before it runs, for an
abort that cannot be caught):
- `the_lexer_and_parser_survive_any_text` (3,000 texts of keywords, operators, literals and near
  misses, odd whitespace and characters): no panic; every token inside the source and in order
  (indentation and documentation tokens excepted: the lexer places docs after them by design);
  lexing and parsing deterministic; every diagnostic has a code.
- `generated_programs_parse_and_round_trip_through_the_formatter` (600 programs from Part III's
  grammar: items, every statement form, every expression level): parse with no diagnostic; the
  formatter's text parses to the same tree (imports sorted, as `[FMT-1]` does) and formats to
  itself.
- `mutated_programs_never_break_the_lexer_or_parser` (2,000 programs with tokens deleted, doubled
  or replaced, as typing makes them).
Ran with 10x the cases on seven seeds: clean after the fixes below.

Defects the fuzzers found, fixed:
- **Lexer: a backslash ending the file inside a string hung the compiler** until an allocation of
  52 GB failed (`ember check` on `x = "abc\`): `scan_escape` returned `Truncated` without moving
  past the backslash, so `lex_quoted` scanned it again forever, a diagnostic each time. The
  scanner now always consumes the backslash; `decode_escapes` (triple-quoted strings) no longer
  adds a byte of its own after an error, which had dropped the character after an invalid escape.
  Test: `ember_lexer` `a_backslash_ending_the_file_in_a_string_is_reported_once`.
- **Formatter: `ember fmt` was not idempotent** (`ember fmt --check` failed on its own output). A
  comprehension, an optional chain (`a?.b(…)`), a `=>` lambda and the types `fn(…)`, `dyn`, `some`
  were copied from the source; one the formatter had broken over lines came back as broken, and
  the line around it was laid out otherwise. All are printed from the tree now (a lambda's header
  too; a generator that is a call's only argument without parentheses of its own, `[GRM-38]`).
- **Formatter: a long `with f = …:` did not parse after formatting**: it was broken as an
  assignment, with its `:` inside the parentheses. A compound statement's `:` stays after them.
  Test for both: `ember_parser` `formatting_the_formatters_output_changes_nothing`.

### Fuzzing the type checker and the borrow checker (done)

`compiler/ember_driver/tests/fuzz.rs`, through `ember check --json` (files side by side; any
internal error or panic fails the case):
- `well_typed_programs_are_accepted_and_one_wrong_type_is_found_where_it_is`: 60 programs of
  structs and functions from a typed generator (int, float, bool, `String`, `Array[int]`, `Copy`
  structs; arithmetic, casts, comparisons and chains, conditionals, fields, calls of earlier
  functions, f-strings; declarations, assignments, pushes, `if`/`for`/`while`); every function is
  well typed by construction and must be accepted; in half the programs some functions have one
  `v: T = e` given a value of another type, which must be `E2020` at that line, with no error
  elsewhere.
- `the_borrow_checker_agrees_with_a_model_of_loans_and_moves`: 40 programs of 8 functions over
  three arrays: views (`Span`), `ref mut` element references, their uses, reads, pushes,
  assignments and moves. A model of `[BRW-1]`/`[BRW-2]`/`[OWN-3]` for straight-line code (a loan
  lives to the last use of its holder; a conflicting access or a moved array is an error) builds
  sequences it accepts, and in half the functions inserts one access it rejects; the checker must
  accept the first and reject the second at that line (`E3020`–`E3022`, or `E3040` for a move).
Ran with 10x the cases on five seeds: the compiler agreed with both oracles everywhere (one
generator error fixed on the way: a variable made from a bare literal is a `str`, `[TXT-9]`).

### `[FFI-5a]`'s layout tests with every supported C compiler (done)

`ember_driver/tests/cdylib.rs` `the_layout_assertions_hold_under_every_supported_c_compiler`: a
header of padded, nested, packed (`#pragma pack`), over-aligned (`_Alignas`), enum-, `long`-,
pointer- and bit-field-holding structs, imported, built and run under each supported compiler of
the host (Windows: MSVC, clang-cl, and Clang where installed; Linux: GCC and Clang; macOS: Clang;
a missing required one fails the test): the shim asserts every struct, its assertions hold, and
Ember's sizes are what that compiler's C reports; and the non-Clang-layout probe stops the build
at its assertion under MSVC and GCC and builds under clang-cl and Clang. Passed here with MSVC,
clang-cl and Clang, and in WSL (a Linux build of this tree) with GCC and Clang. CI runs it on both
systems.

### Every run test also runs on the evaluator (done)

Every program the suite runs is run twice: compiled, and on the evaluator (`ember run --interp`,
below), and the evaluator must give the compiled run's standard output, exit status and panic
line exactly; a program it refuses with `E6010` (a foreign call, a thread) is the one exception.
Both harnesses do it: `assert_the_evaluator_agrees` in `ember_driver/tests/milestones.rs` (so
`run_pass_programs_pass`, `run_fail_programs_panic`, `milestones_pass`) and `evaluator_disagrees`
in `tasks/impl-0.9.9/annotations.py` (the whole conformance tree). The evaluator's runs add about
a minute to the conformance tree with 8 workers. **For whoever merges**: a helper's new run test
that uses a builtin the evaluator does not have yet now fails with "the evaluator stopped: the
builtin `x` is not run by the evaluator yet"; the fix is the builtin in `ember_interp`
(`builtin.rs`'s dispatcher), following the C backend's `call_expression` arm.

## `[CLI-21]` (done)

`ember run --interp <file>` runs the program on a new evaluator, `compiler/ember_interp`, with no
C compiler: the toolchain is not even looked for (a run with `EMBER_CC=no-such-cc` works).

- **What it runs**: the program's MIR as `ember check` leaves it — every check passed, drops
  elaborated, generators made state machines, the dynamic access checks inserted — before the
  passes that only make code faster (`compile`'s hook before the `@simd(assert)` reports).
- **How**: every value lives in one flat byte heap with the target's layout (`TypeTable::layout`),
  so a value is its bytes as in the compiled program, and what the C backend and the runtime do
  for a construct is done the same way: class objects with the runtime's header and access words,
  counting, drop glue and `drop` adapters, virtual and interface dispatch, callable values and
  their descriptors, niche `Option`s, the list buffer's growth, `Shared`/`Weak`, `RefCell`,
  statics (lazy, with the cycle panic), formatting (the format mini-language, Python's float
  text), parsing, ranges, SoA (`[SOA-1]`..`[SOA-7]`), arenas (`[ARN-1]`..`[ARN-12]`, the runtime's
  chunk list), SIMD lane by lane (`[SIMD-1]`..`[SIMD-9]`), atomics and locks on one thread,
  256-bit integers (`wide.rs`), files, console, `process`, arguments, the panic line, the `[RT-4]`
  backtrace outside `release`, the `debug` run's leak report (`[WK-4]`, `[WK-15]`: the runtime's
  components, edges and shortest cycles) and the debug access-conflict text (`[EXC-6]`).
- **Compile-time values**: with `--interp` the compile-time evaluator's requests run on the
  evaluator too (`ember_interp::comptime`), answering as the C program `emit_comptime` builds does
  (the same value text, exit statuses, `ember-comptime-*` lines), so `E6004`, `E6001` and
  `read_file`'s `E6010` are word for word the same (checked: every compile-fail test under
  `CT-*`, `STA-*`, `TYP-41` gives the same diagnostics both ways). Its answers are cached apart
  from the C program's, so neither hides the other.
- **Refused**: a foreign function (called directly or through its wrapper), a thread, a job, a C++
  or foreign object, a pinned object's address: `E6010` at the call, "… cannot run on the
  evaluator", with a help to use `ember run`. A lock its own thread holds, or a `wait` no thread
  can wake, stops with a fault (the compiled program would wait for ever).
- **Speed**: the hot paths read and write words without building values (`scalar_binary`, copies,
  checked arithmetic, branches), layouts and sizes are cached per type and frames per body. A
  MIR statement costs about 20 ns; the slowest run test (20 million loop turns, D-752's) takes 12 s.
- Tests: `ember_driver/tests/interp.rs` (no C compiler, console, files, a compile-time value, both
  ways; a foreign call and a thread are `E6010`; a panic and a compile-time panic are the compiled
  program's), and every run test of the suite both ways (above). Sweep: 1,864 run tests pass both
  ways, 172 refused (foreign calls, threads, jobs, C++), none differ.

### Questions and observations for the owner (`[CLI-21]`)

1. **Should the evaluator become the compile-time evaluator for every build?** It answers every
   request the C program does, word for word, without compiling and linking a program per check
   pass (on this laptop that program's build is most of a cold `ember check` of a program with
   compile-time values). Options: (a) keep the C program for builds, the evaluator for `--interp`
   (what is built; no change to builds); (b) the evaluator for every build, the C program kept as
   a cross-check in CI; (c) the evaluator only. Recommendation: (b) — faster checks, one less
   compiler run per pass, and the C program still checks the evaluator. Not done: it changes
   `[CT-4]`'s "the target's own code, built by the target's C compiler" wording, which is the
   owner's.
2. **What "equal a compiled run's" cannot mean**: an address. `{:p}`, a class's `Debug`
   (`<Node at 0x…>`) and the leak report's `Node@…` print the evaluator's addresses, which are not
   the program's (no test prints one). And `rsqrt_approx`/`rcp_approx` are exact on the evaluator,
   as on a target without SSE, where SSE's are 12-bit estimates: `[SIMD-4]` allows either. I read
   the rule as "the same output for a program whose output is defined", which is what the suite
   checks.

## `[IDE-3]`, `[IDE-4]`, `[IDE-6]` — the compiler for an editor

**`ember_driver` is a library now** (`src/lib.rs`, which was `src/main.rs`, renamed with `git mv`
so a merge carries other branches' edits of `main.rs` into it; the `ember` program is
`src/ember.rs`, three lines). **For whoever merges**: a branch that edited `src/main.rs` merges into
`src/lib.rs` by git's rename detection; a branch that *added* a `mod x;` there does too.

- **`ember_driver::session::Session`** (`[IDE-4]`, `[IDE-6]`): `set_buffer(path, text)` (an
  editor's unsaved text), `hover(path, Position)`, `complete(path, Position)`, `symbols(path)`,
  `problems(path)`, `clear()`. Each query checks the file — parsing, the modules it imports, name
  resolution, type checking, once (a compile-time value is left unevaluated) — unless the last check
  was of the same text, and answers from that: a local's or field's type, a call's signature, a
  declaration's head; a value's fields and methods after `.`, the names in scope elsewhere; every
  declaration with its members. Nothing is lowered, built or run, and no C compiler is looked for.
  The session holds one check; `clear` or dropping it frees it. (`ember lsp` stays reserved: this
  is what it would serve.) `ember_hir::visit_block`/`visit_expr` is the walk it finds things with.
- **`[IDE-3]` — every stage after parsing runs on a file with errors**: `compile` no longer stops
  after a parse error or a module not found; names and types are checked whatever came before, and
  what failed is a placeholder that satisfies everything, so each error is reported once:
  - the lexer closes brackets never closed at the next unindented declaration (`fn name`, `struct`,
    `class`, `import`, …, which no bracket can hold), with `E0101` at the opener — before, an
    unclosed `(` swallowed the rest of the file;
  - the parser makes a placeholder of what it reported: a jump as an operand (`E0107`), a chain with
    `is`/`in` (`E0102`), a multi-statement closure in brackets (`E0106`); it drops a statement
    attribute it rejected (`E0104`, `E0108`); `owned` where it may not be (`E0109`) is the value it
    marks; `a::b` (`E0100`) is read as the `a.b` it should be;
  - the type checker takes the parser's placeholders silently (an expression, a type, a
    declaration's pattern), does not report a missing `return` in a body holding a placeholder,
    checks `if x = e:` (`E2036`) without the `else` it makes unreachable (`W2091`), does not report
    a missing module again (the loader's `E1060`), and treats the names a failed import binds, and
    the names a foreign import skipped in the module it wrote, as placeholders.
  - Every compile-fail test passes unchanged with this.
- Tests: `ember_driver/tests/editor.rs` — `queries_keep_working_in_a_file_with_errors_of_every_stage`
  (`[IDE-3]`: an unclosed bracket, an unresolvable name, an untypable `+`: four errors, none
  following from another; hover, completion and symbols in and around the broken functions),
  `queries_need_no_code_generation_and_no_c_compiler` (`[IDE-4]`: a program whose compile-time
  value and build need a C compiler, and with a borrow error `ember check` reports: answered, no
  file written, no borrow error among the query's problems); `ember_driver/tests/editor_memory.rs`
  (`[IDE-6]`, a counting allocator: fifty edits and queries do not grow the session; `clear` and
  dropping it free what it held, to within 4 KB).

### What is not done, and a question (`[IDE-3]`, `[IDE-6]`)

1. **`[IDE-6]` — the identifier interner is the process's**, not the session's
   (`ember_span::Symbol`: `as_str` gives a `&'static str`, leaked). An editor that types a name a
   letter at a time interns each prefix once and keeps it: growth by the distinct identifiers the
   process ever sees, small but without bound. The fix is an interner per session, which needs
   `Symbol::as_str` to give a borrow tied to the session (as rustc's `Symbol::as_str(&self)` is to
   its session globals) instead of `&'static str` — every crate's uses of it, a change of its own.
   Options: (a) that change; (b) an editor process restarts its compiler session's process now and
   then (no change); (c) intern only names that resolve, typing's partial names held elsewhere.
   Recommendation: (a), when the language server is built (it is what needs it); until then the
   rule stays `unfinished` with this said.
2. **`[IDE-3]` — the stages after type checking** (borrow and effect checks, the lints on MIR) do
   not run on a program with a type error: `compile` stops after type checking reports one.
   Tried (the diff is kept out of the branch): each function that failed — the checker's
   `incomplete`, one holding a placeholder or a value of an unresolved type, one an error was
   reported in (its attributes' lines included) — lowered as a body that is never reached (no
   effect, no borrow), left out of the interface artifact, and the passes run on the rest. Of the
   compile-fail tests, 9 then changed: 5 gained a lint that is right (`L1001` for a binding never
   read, in a function with no error), 1 a borrow error that is right (`TYP-17`'s `pick("x")`
   returns a view of `u`), and 2 (`CLO-6`) a cascade that is not: the error is in `main`, but the
   instance of `twice` made for `main`'s wrong argument is checked as if sound. Fixing that needs
   the type checker to mark the instances a failed body asks for — and its `incomplete` set is
   also what the compile-time evaluator stubs, so it needs a set of its own. Not done; the rule
   stays `unfinished` with this said.
