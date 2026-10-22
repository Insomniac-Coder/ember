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

## `[IMP-11]` (in progress)

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
