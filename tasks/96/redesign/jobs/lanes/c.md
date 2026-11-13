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
