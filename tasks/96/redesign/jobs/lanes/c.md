# Lane c: types, messages and the standard library (M11, M14, diagnostics, the core language)

Read first by every job in this lane; updated last by each.

- **Milestones:** M11 (interface values, kind tests and casts, generic bounds, `init`, `@cached`, class rules,
  contracts), M14 (std.ecs as EnTT's design, the rest of std), and the core language's rules (Parts II to VI) that no
  milestone owns. Items: `gap/07-syntax-types-diagnostics.md`, `gap/04-functions-effects.md`,
  `gap/08-std-ecs-programs.md`.
- **Families:** GRM, LEX, TXT, CTL, STR, EXP, MOD, DIA, ERR, TYP, FN, EFF, CLO, OBJ, IFC, STD, ECS, MNG, DSP, EXC, CT,
  PHIL, CLI, DET, and the small ones (281 unfinished rules, 51,221 words, on 2026-11-13).
- **Worktree:** `C:\Users\ism19\Code\ember\.claude\worktrees\lane-c`, branch `lane-c`; its `target/` stays between
  jobs.
- **Where the code is:** the lexer `ember_lexer/src/{lib,token}.rs`; the parser `ember_parser/src/{lib,decls,body,
  python}.rs`; the AST printer `ember_ast/src/print.rs`; the formatter `ember_fmt`; the checker `ember_typeck/src/`
  (`lib.rs` and its modules: `packs.rs` for `[TYP-47]`, `errors.rs`, `habits.rs`, `objects.rs`, `derives.rs`); the
  messages' codes `ember_diag/src/codes.rs`; error pages `docs/errors/`; std `std/src/`.
- **Evidence:** for GRM and LEX a parse-only test counts (`phase_progress.py`); for every other family a test that
  expects `E0900` counts only for the rules about refusing what is not built.
- **Waiting, with their rules' jobs:** `[DIA-2]`'s spelling done generally (Opus; commit `29cc91df` was a
  whole-program scan, not the rule); `[MNG-1]`/`[MNG-6]`'s `StrView` under `std.core`; DIA-10 B5's and B9's helps;
  `[TYP-26]`; `[TYP-15a]`; `[STD-26]`; `[OBJ-2]`'s respelling to `dyn I` (M11, `f0d4fa67` kept).

## W3, job 1 (GRM and three LEX rules), 2026-11-13

- **Credited (built_for_0915):** GRM-2, GRM-4, GRM-5, GRM-8a, GRM-19, GRM-27, GRM-29, GRM-37, GRM-38, GRM-47.
- **Where things are:** `[GRM-47]` is `ember_parser/src/body/owns_operand.rs` (the operand is noted when an operator is
  built, then reported with the line to write by the call or the `=` it stands in; the arguments become `owns` over a
  placeholder so `check_call_word` stays quiet: it returns early on `ExprKind::Error`). `[GRM-5]`'s mixed targets and fixed
  arrays are in `check_destructure`/`collect_destructure_leaves`/`destructure_source` (`ember_typeck/src/lib.rs`). `[GRM-15]`'s
  `x = owns req.field` is `synth_owns` (a part goes through `Builtin::HandOver`; E3010 for a `drop` owner, E3011 for an
  element are emitted there, E3013 for a class object or a borrowed parameter comes from `ember_analysis/src/drops.rs`).
  `[GRM-41]`'s E0118 and `changes`/`reads` E0100 are in `parse_fn_signature` (`decls.rs`). `[GRM-3a]`'s refusals are
  `refuse_stacked_weakref` (`decls.rs`). E0116 is `check_word_list` (`body.rs`), E0114 in the two word-before-a-type/value sites.
- **Traps:** a parse error does not stop the checker, so a node the parser reports must be left as a placeholder the checker
  ignores or it adds a second error. `annotations.py` runs only a binary inside `target/` (the runtime is found beside it); do not
  rebuild while a long run uses it. Python scripts that rewrite `.em` files must use `encoding='utf-8'`.
- **Not credited, and why (the clause, the milestone):**
  - GRM-3: `using` after a type is not parsed (E0100), `thread_shared fn()` and the owner words are E0900 (M4).
  - GRM-3a: `weakref Node = None` is E0900 (M4); the refusals are built and tested.
  - GRM-8b: a bare interface as an instantiation's argument is accepted (M11 respells it `dyn I`, OBJ-2).
  - GRM-13: list patterns (`[a, b]`, `[a, ..rest]`) parse but the checker says "not supported yet" (needs a HIR pattern kind and both backends).
  - GRM-15: the owner words made in a call (`shares mut`, `weakref this`, `thread_shared`) are E0900 (M4); `owns copy of fn` is GRM-42's.
  - GRM-17: a block after a lambda's `:` outside brackets (`h = fn(x):` then an indented block) is not yet `E0106`: about 31 older
    conformance files use one (CLO-2, RC-2e, GRM-28, STD-19, CTL-1, ...) and need a local function each. Held for the lead.
  - GRM-28: capture words after a local function's brackets are E0900 (CLO), `thread_shared log` (M4); `extern "C" fn` in a block is E0900.
  - GRM-30: `cast s if Circle` is still accepted (spec: E1010; the kind test that replaces it is M11); `as?`, `as!`, `cast s when D` are E1010.
  - GRM-34: a call of a blanket method that leaves a parameter open does not take it from a bound (`t.size()` under `T: needs Get[int]` is E2060; two bounds give E2070, the spec says E2060).
  - GRM-35: `hide`'s E1052 needs the C importer (lane b).
  - GRM-36: `ref mut cast ...` (M11).
  - GRM-41: only `mut` on `this` is untested, since no spelling of it is given (spec question).
  - GRM-42: capture words are E0900 (CLO); E0116 is built for loop names, pattern names and lambda parameters.
  - GRM-43: owner words (M4), `const ref mut` as a value word is not parsed, `using` after a value is not parsed.
  - GRM-44: `weakref` links (M4); also `if n exists as w:` for a non-Option is E1010 (spec: E2036 with a help), `exists as t` for a name in scope is not E2106, plain `p += 1` after `exists as p` is not E3023, `exists as copy of v` over `Option[ref T]` is E2116 (spec: a copy of the program's own).
  - GRM-45: the kind test is M11. GRM-46: `from` and `guarded by` after a field's type are not parsed (kept views M5, locks).
  - LEX-15: `from` and `guarded`/`by` after a field's type (GRM-46). LEX-23: `unsafe import c` and `@unsafe(...)` notes (lane b).
  - LEX-26: the spec says E0009 for a NUL in a `c"..."` literal in LEX-26 and E5064 in TXT-3 and the error table (question).

## W4, job 2 (GRM-17's block lambda and the core language), 2026-11-13

- **Credited (built_for_0915):** LEX-1a, LEX-10, LEX-11, LEX-16, LEX-18, LEX-20, LEX-21, TXT-2, TXT-3, TXT-4, TXT-8, TXT-9,
  TXT-10, TXT-11, STR-1, STR-2, STR-6, STR-7, CTL-3b, CTL-10.
- **Where things are:** `[GRM-17]`'s E0106 for `fn(x):` + INDENT is in `parse_lambda` (`ember_parser/src/body.rs`; the old
  `owned fn(x):` block is still read there, see below). `[LEX-1a]` is a scan at the top of `ember_lexer::lex` (`is_bidi_format`,
  `bidi_message`; the manifest check is in `ember_driver/src/packages.rs::read_manifest`, tested in `milestones.rs`). `[LEX-10]`'s
  message is in `parse_module` (`ember_parser/src/lib.rs`, `first_any`). `[LEX-11]`: a trailing `##` is held in `pending_docs`
  like an own-line one (it was a token in the middle of the line and broke `fn f(): ## note`); `skip_docs_before_else` and a
  `take_doc` before each match arm. `[TXT-2]`'s E5063: an export taking a `StrView` (`ember_typeck/src/lib.rs`, the export-parameter
  loop) and a `@ffi(macro_fn)` entry typed `StrView` (`ember_driver/src/cimport.rs`, `spell`). `[TXT-11]`: `String + StrView` is now
  the f-string's `FString` node over both operands (so `a` is not consumed). `[STR-1]`: `check_memberwise_constructor` (narrowest
  field: pub, pub(package) via `same_package`, module); cross-package test in `ember_driver/tests/package.rs`. `[STR-2]`:
  `finish_struct_literal` (`written_field_order`: written arguments in locals first when the field order differs) and
  `construct_through_init` (arguments hoisted before the defaults); `default_context` gives the two `E1010` messages for a default
  reading a field or `this`. `[STR-3]`: `@derive(Copy)` over a class field is E2080 (`holds_class_object`). `[STR-6]`: E2108 is in
  `parse_fn_signature` (`decls.rs`).
- **Traps:** `annotations.py` needs `PYTHONIOENCODING=utf-8` for output with non-ASCII; a per-file run says "no accept_* case", run the folder.
  A type error stops the borrow checker, so an `E3040` line in a test beside an `E2040` never fires. Bash eats backslashes in
  heredocs: write Rust with `\n`/`\u{..}` through a Python file made with the Write tool, or the Edit tool.
- **Not credited, and why:**
  - GRM-17: the 14 older tests whose lambda is `owned fn(...):` + block (CLO-10 x3, CLO-2 x2, CLO-6, FFI-21 x2, RC-2e x3, STD-19 x2,
    ...) cannot become local functions: a local function is not `owned` (`E1010`, "`owned` is gone"), and the spec's spelling,
    `fn f() owns x:` (capture words, GRM-42), is E0900 (M-CLO). So `owned fn(x):` + block keeps its block (`parse_lambda`, `is_owned`)
    until capture words exist; the other 24 tests are migrated and E0106 is built and tested for `fn(x):` and `owns fn(x):`.
  - TXT-6: the export that takes a `Span[u8]` is `E5015` today (FFI-31b, lane b); nothing else of the rule is testable.
  - STR-3, STR-5: a field of `shares`, `weakref` or `sharedref mut` is E0900 (M4); the class-field part of STR-3 is built.
  - CTL-7: a view a `defer` uses that a later reshape ends is `E3021`, the spec says `E3060` (question below).
  - CTL-11: `weakref` (M4), the kind test `is C as c` (M11), a plain `exists as p` name is read-only (`p += 1`, GRM-44, unbuilt).
