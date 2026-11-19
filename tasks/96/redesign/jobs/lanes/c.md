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
  reading a field or `this`. `[STR-6]`: E2108 is in
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
  - STR-3, STR-5: a field of `shares`, `weakref` or `sharedref mut` is E0900 (M4); and a class handle is `Copy` in `copy_with` (OWN-7's
    tests, RC-2e, rely on it) while the spec says a class object is not (lane a's A1-9), so `@derive(Copy)` over a class field is accepted.
  - CTL-7: a view a `defer` uses that a later reshape ends is `E3021`, the spec says `E3060` (question below).
  - CTL-11: `weakref` (M4), the kind test `is C as c` (M11), a plain `exists as p` name is read-only (`p += 1`, GRM-44, unbuilt).

## W5, job 3 (the types of Part III and LEX-10), 2026-11-18

- **Credited (built_for_0915):** LEX-10 (reading B), TYP-1, TYP-6, TYP-9, TYP-9b, TYP-12, TYP-19, TYP-21, TYP-24, TYP-25, TYP-26,
  TYP-31, TYP-35, TYP-39, TYP-47.
- **Where things are:** LEX-10: `check_indented_directive` (`ember_lexer/src/lib.rs`, called at an own-line or in-bracket `#!`).
  TYP-14's E2200: `ember_typeck/src/infinite_size.rs` (`check_infinite_size`, run after `check_view_attributes`; the refused field
  becomes an error type, or later passes overflow the stack on `struct T: a: T`); the `ref mut` name of a `dyn I` type is in
  `views.rs::repoint`. TYP-6: `synth_int_try_from` (`T.try_from`, a round trip plus the sign), the bool-to-integer cast (and
  the LLVM `bool` constant in `cast`), the enum-discriminant check and the E2020 chains in the `Cast` arm. TYP-5 rule 11 now composes
  with rule 12 (`coerces_before_option`) and an `owns` block (`check_expr_through`). TYP-21: `String` implements `Add[StrView]` and
  `AddAssign[StrView]` in `std/src/core.em` (the built-in `+`/`+=` still win: no `add_assign` in the C). TYP-25: the first
  parameter no other argument gives (`call_argument_slots`, `constructor_slots`). TYP-47: E2021 for a pack's values given to
  fixed parameters (the call site before `synth_generic_call`). TYP-23: a lambda parameter without `owns` where the callable
  type hands over is E2020 (`synth_lambda_fresh`).
- **Traps:** `owned` is a gone word (E1010), use another name for a local; a `#$` line's backslashes are read, write them with
  care; a heredoc through Bash eats `\n` in Rust, use the Edit tool; the shell's `$EMBER` does not persist between calls.
  `m = f(m, other)` for a non-Copy struct is E3021 on this base (see the report).
- **Not credited, and why:** TYP-5 (rule 7 `shares`, rule 8 class upcast and `dyn I` read in place need M4/M11), TYP-10 (the
  panic text "`shift out of range`: `1 << n` on line 9 shifts an `int` by 70; use `wrapping_shl`" is unbuilt: `AssertKind::ShiftTooLarge`
  has no operands in five backends), TYP-13 (`shares`, `thread_shared`, `sharedref mut` niches, M4), TYP-14 (`shares` owners, M4),
  TYP-17 (bound inference for a non-`pub` generic, the `pub` E2040 message, E2072, `dyn (A, B)` and `+` after `dyn`/`some`),
  TYP-22 (`dyn (A, B)`, E2050 messages, M11), TYP-23 (wording), TYP-32 (`some (A, B, Send)` is not parsed), TYP-34 (`from static`,
  M5), TYP-36 (cells: tuple and array `Default`, `Option` `Ord`, `Map`/`Set` `Display` as a bound, a class's `Clone`), TYP-38 (a literal
  followed by `using m`, TYP-46), TYP-43 (`ThreadShared` does not exist: M4), TYP-44a (type indices at link: M11).

## W6, job 4 (functions, errors, expressions, interfaces, attributes), 2026-11-19

- **Credited (built_for_0915):** ERR-13, ATT-2, FN-1a, FN-2, FN-4, FN-5, ERR-2, ERR-9, EXP-5, EXP-10.
- **Where things are:** `[ERR-13]`: `ember_panic_bounds` (`runtime/ember_rt/templates/ember_rt.c.in`, then `python tools/generate_runtime.py`)
  and the interpreter's `bounds_message` (`ember_interp/src/lib.rs`): "...; use .get(i) for an Option" on every bounds panic. `[FN-2a]`'s E3027
  for `ref mut` before a name that only reads: `reject_borrowed_parameter_write_as` (`ember_typeck/src/lib.rs`, uses `ref_mut_argument`, set in
  `check_argument_by`) for a parameter, `read_only_change` (`views.rs`) for a loop, pattern or view name; a constructor's `init` `ref mut`
  parameter takes the word (`constructor_arg`, `check_call_word`). `[FN-5]`: `check_declared_defaults` (E3027 for a default on `ref mut`, an
  `owns` default is checked as an argument so an earlier plain parameter cannot be handed over). `[FN-8]`: `main`'s `E` must be `Display`
  (`check_main_form`); a script's `?` makes the implicit `main` `Result[void, AnyError]` and appends `Ok(())` (`parse_module`,
  `script_try`/`script_scope`/`script_nesting` in `ember_parser`); `return`/`yield` at file scope are E0100 (`script_jump`, `body.rs`); the
  E1010 help for a script name used in a function (`script_locals`). `[FN-4]`: `static_build_help` (the help that builds the value). `[EXP-5]`:
  a method that changes a temporary as a whole statement is recorded in `pass_receiver_to` (`discarded_receiver`) and refused in
  `decide_implicit_objects` (`objects.rs`), once the method is known to change its object. `[EXP-4]`: `lower_match` (`ember_mir/src/lower.rs`)
  ends a match subject's temporaries before the arms only when the subject's value is plain data (`is_plain_data`): any other value broke 71 tests (a value that owns heap storage may share it with the dropped temporary), so `match owns JOBS.lock().pop():` (N101) still needs its own analysis.
- **Traps:** a default on a `ref mut` parameter (E3027) contradicted TYP-9's old test (rewritten with `owns`); a method on a temporary that views a place (`q.commands().add(..)`) is not E2140. After a Python edit of a Rust file, `touch` it if `cargo build` says it is up to date; NEVER `cd ..` from a lane's `compiler/`
  directory (it lands in the main checkout): every command starts with `cd <lane root>`. A debug `eprintln!` in `ember_typeck` is not shown
  by `ember check`; write to a file. The shell tool resets its directory to the main checkout between calls.
- **Not credited, and why:**
  - ATT-1: `@sync` is still a working attribute (48 files; the replacement is `thread_shared`, M4) and `@component(layout=soa)` is std.ecs's storage choice
    (the target says "any type is a component, with no mark": M14).
  - FN-1: the `shares` clauses are M4, a class object is not `Copy` (A1-9: `p.heal()` through a reading class parameter and `owns p: Player` then use are not E3023/E3040).
  - FN-2a: a function from a variable lent to a callable-bounded `owns f: F` (`apply(h)`, `ref mut h` for state that changes, `owns h` E0109) needs `[CLO-18]`'s
    state analysis; today `apply(h)` is E3028 and a state-changing closure takes no word. The `shares mut game` examples are M4.
  - FN-3: the compiler says E3060 for a returned view of a local, the spec E3063 (question 2).
  - FN-6: `extern "C" fn(Span[T])` and `ref mut x: T` as C carriers, the thread-attaching C entry, and E2020 (the compiler says E5015) are FFI-31b (lane b).
  - FN-8: tested and built except "`args` decodes invalid bytes as U+FFFD": built in the runtime (`push_utf8_lossy`, Linux) but a test needs raw argv bytes (Linux only).
  - FN-9: class objects (A1-9).
  - ERR-1: `f"{e}"` and `println(e)` of an error with no `Display` compile (question 3).
  - ERR-3, ERR-8: a one-thread error through `?` into `AnyError` is not E2040 (needs `Error`'s `Send`/`ThreadShared` check at the conversion, N130) and `@from` on a bad variant is E0104/E2020, not E2080.
  - ERR-4: ERR-10's `context` and the list of methods are tested, but the 0.9.20 `once fn` bounds were not re-checked one by one.
  - EXP-4: E3063 vs E3060 for a view that outlives its temporary (question 2).
  - IFC-3: the E2040 message "`Sq.scale` takes `k: i64`, and `Scaler.scale` takes `k: f64`; write `fn scale(k: f64) -> f64:`" is `does not match the signature required by` (5 older tests pin the prefix).
  - IFC-4: an interface's associated constant (`const N: int = 3` then `A.N`) is not built.
  - IFC-5: a bare `dyn I` in a field, result or payload is E2020 "an unsized interface value", the spec says E2173 with the owner and the two lines to write.
  - IFC-6 and TYP-17: bound inference for a generic that is not `pub` is not built (`fn total[T](a: T, b: T)` with `a + b` is E2040 today).
  - TYP-10 (the shift panic's operands in five backends), TYP-23's wording (the callee is not known where the lambda is checked: `argument_of` is
    `None` there), TYP-17: not started.

## W7, job 5 (`ref` in a parameter's type, what W6 left, diagnostics), 2026-11-19

- **Credited (built_for_0915):** ERR-4 (the new test covers every method that takes a function as a `once fn`), DIA-8 (`explain_view_names_each_view_in_use_and_the_use_that_keeps_it`, `milestones.rs`: built before, exact outputs for a shared view, a changing one, a line with none).
- **Where things are:** `[FN-1]`'s `ref`/`ref mut` in a parameter's type is `E0117` in `refuse_mode_in_type` (`ember_parser/src/decls.rs`, also
  lambda parameters in `body.rs`); the diagnostic's migration moves the word to the name, so `a_migrate_views.py`'s way of applying it works
  (scratchpad `mig.py`: the same script with the suggestion's message); `ember fmt --migrate` does it too (`migrate.rs::parameter`). Generated
  Ember source: `cimport.rs` (every `{param}: ref ...` is now `ref {param}: ...`) and `reload.rs`'s wrapper. A test that returned a view of an
  `i32` parameter now uses `String`: a borrowed `Copy` parameter is passed as a copy (BRW-8), so `ref a: i32` cannot give a view back (E2031/E3060),
  where `a: ref i32` could. `[TYP-23]`: `callee_named` (the innermost call's name, set by `synth_call`/`synth_method_call`) gives the lambda-word
  message its callee. `[IFC-3]`: `signature_difference` and `interface_first_line` (`ember_typeck/src/lib.rs`, next to
  `implementation_signature_matches`); the same parameter names are now required (std.math's operator impls, `FloorDiv`/`Rem` on `iN` and `Set`'s
  operators, `Pool.index` were renamed to the interface's `rhs`/`i`, and 72 test programs). `[IFC-5]`: `bare_dyn_message` (E2173 for a bare
  `dyn I` in a field, a result or a payload; a `dyn I` parameter stays as it was). `[DIA-24]`: `name_suggestions::nearest_help` (one help, "the
  nearest name is `x`" / "the nearest names are `a`, `b` and `c`", same file first, then by qualified name; 15 tests, 4 `.stderr` snapshots and
  `ui.rs` follow). `[DIA-2]`: the "you" in six helps and the lint label are gone, `E3016`'s message names `this.<field>`.
  `[TYP-17]`: the `E0100` for `T: A + B` gives the whole line with `needs`.
- **Traps:** a Python heredoc with `'\n'` inside Rust source writes a real newline: build the character with `chr(92)+'n'` (it broke the build three
  times). `annotations.py` over 300 files waits for `%TEMP%\ember-suite.lock`, which another worktree's `cargo test` of the conformance suite holds for
  an hour or more: do the full run last, and never start it with sources newer than the binary. A killed run leaves `ember.exe` locked: kill its
  child with WMI. `cargo test -p ember_driver` runs the `milestones` suite of `tests/compile-fail`, `compile-pass`, `run-pass` and `milestones`, which
  the quick check does not see (it reported 0 failing while 8 files were wrong): run `cargo test -p ember_driver --test milestones -- compile_fail_programs_are_rejected compile_pass_programs_compile run_pass_programs_pass milestones_pass` after a change of the parser or a diagnostic.
- **Not credited, and why:**
  - **`E0117` for `ref` is for `fn` declarations only; the exemptions and the fallout** (the lead decides the open ones): a lambda's `fn(w: ref String)`
    stays (it is the callable type `fn(ref String)`; `fn(ref w: String)` is a different callable and breaks `iter().filter(...)`), `ref mut` in a lambda is E0117 as
    before; the importer's source (`ember_parser::parse_import`) and an item with an `@ffi` contract keep `x: ref T` (the pointer C takes, `[FFI-11]`), so the
    contract checks that read `ty == Ref` work. `ref a: T` is NOT the old `a: ref T`: (1) a `Copy` T (a scalar, a view struct) is passed as a copy, so a
    view of it cannot be returned (E2031/E3060) and a borrow of the caller's place is not live for the call (E3021 and the run-time exclusivity checks are not
    there): 13 tests were rewritten to `String` or a non-`Copy` struct (a_returned_view_elision_cannot_tie, a_view_struct_carries_its_region,
    borrows_travel_with_the_reference, borrows_attribute_accepted, m2_the_borrow_checker_catches_escapes, references_that_travel, returned_views, view_structs, BRW-3 x2,
    EXC-2 x2, HEAP-5, FN-5 x2, EXP-9 x2, DIA-11, TYP-5, TYP-18, LT-1, CTL-1); (2) a class handle is `Copy` until A1-9, so `ref fetch: AosStorage[T]` returned a view
    of its fields as E3060: `through_a_class_parameter` in `ember_analysis/src/borrows.rs::check_escapes` (a file outside this lane's list) lets a view through a
    borrowed class parameter's field escape; (3) a function item `fn two_fn(ref w: String)` is `fn(String)`, which `find` (`fn(ref String)`) refuses: CTL-1 now
    uses a lambda. **Still failing after this (std/ecs.em's 30 migrated lines): `ECS-4/run_fail_a_system_writes_a_storage_its_query_holds` and
    `ECS-5/run_fail_destroying_through_the_world_while_a_query_iterates` expect the run-time `exclusivity violation: write access to std.ecs.AosStorage[Hp]`,
    which is no longer raised once the view-making functions take `ref fetch: AosStorage[T]`; the lead is changing ecs.em (M4 slice E).**
  - The name rule of IFC-3 is not applied to the importer's source (`generated_files`), whose operator wrappers keep the header's parameter names.
  - TYP-10: the panic text needs the amount: `AssertKind::ShiftTooLarge` must carry `amount: Operand` (+ the expression text, the type name and
    `wrapping_shl`/`wrapping_shr`), which touches about 26 sites that list `RefCellBorrow`/`Bounds` (borrows.rs x4, coroutines.rs x4, kernels.rs x3,
    progress_reduction.rs x4, loop_version.rs x3, regions.rs x2, inline.rs, list_locals.rs, loop_access.rs, perf_lints.rs, static_safe.rs,
    strength_reduce.rs, temporary_owners.rs, unused.rs, access.rs, effects.rs, verify.rs, codegen_c x6, codegen_llvm x2, interp, driver): any one that
    does not rewrite the operand after inlining or loop versioning miscompiles. Plus `ember_panic_shift` in the runtime. Not started.
  - TYP-23: the wording and the callee are built, but a lambda whose body is `Ok(v)`/`Err(e)` given to a generic helper (`good.and_then(owns fn(owns v) => Ok(v))`)
    does not take its error type from the expected result: `Ok(v)` is `Result[i64, void]` there (`synth_wrapper`, and `synth_lambda_fresh` drops an expected
    result that has an open parameter), so the call is `E2020`. It needs the expected result to reach the body with the open parts as holes.
  - TYP-17, IFC-6: bound inference for a generic that is not `pub`, the `pub` E2040 message, E2072's message and `dyn (A, B)`/`some (A, B)` (M11) are not built;
    only the `E0100` line of `[T: A + B]` is. IFC-3: the marker parents (`Send`, `ThreadShared`: M4) and the line that compiles for a missing parent. IFC-4:
    associated constants (`const N: int` in an interface, `A.N`, `T.N`) are not built at all (parser, extend, lookup, const evaluation). IFC-5: `ref dyn I`
    over a struct and `dyn I` as a parameter given a value need M11; `shares dyn` M4.
  - ERR-1: every clause is tested (`tests/conformance/ERR-1`) except that `AnyError`'s `From` needs `Send` and `ThreadShared` of `E`: `ThreadShared` does not exist in the
    checker (M4, N130). ERR-3, ERR-8: the same, and `@from` on a variant with no field, two fields or a field that is no `Error` is E0104/E2020 and the spec says E2080.
  - FN-8: the `#[cfg(unix)]` test `main_args_replace_text_that_is_not_utf8` exists (`milestones.rs`) but WSL has no `cargo`, so it has not run on Linux; the C runtime
    alone was run there (`ember build --emit c`, then gcc in WSL with the same four argv byte strings: output as the test expects).
  - DIA-2: `int` is shown as `i64` everywhere (`types.display`), where the rule says `int` unless the program writes `i64`: the table has one type for both.
    DIA-3, 7, 8, 9, 10, 11, 16, 18, 21, 22, 23, 25: not started (no time); DIA-6a: 123 rows of the registry have a rule after the first with no reject case for the code
    (script in the report), and three codes the document names are not registered (E1040, E9012, L3019).
