# Helper AE — the incremental build, the editor's last parts, the release tooling (2026-10-23)

Rules: `[BLD-1]`, `[BLD-2]`, `[BLD-4]`, `[BLD-7]`, `[BLD-8]` (ODR-244, ODR-257, ODR-258); `[BUD-*]`
(ODR-242) and `[HR-1]` (ODR-261); `[IDE-6]` (ODR-256), `[IDE-3]`; `[HR-9]`, `[BEN-6]`; `[TOOL-1]`,
`[TOOL-2]`, `[TOOL-4]` (ODR-222, ODR-223, ODR-258), `[DOC-4]`; D-882. Branch
`worktree-agent-a3e943ca5ca791ff8`, from main `3922e110`.

## Measuring on a shared machine

The lead's other helpers build and test on this laptop at the same time, so wall times move by 2x
from run to run. Every figure below is the median of several runs of the compiler's own CPU time
(user + kernel, `GetProcessTimes`), which another process's load moves far less, with the wall time
beside it; "before" is the release compiler of `3922e110` (`build/ember-base.exe`), "after" this
branch's, each run on the same package after the same edits.

## Done

### The compiler's fixed costs on `ember check` (ODR-242)

`bench/bigpkg` (400 modules, 48,586 lines), `ember check` after a one-function-body edit, release
compiler: 4.46 s wall / 4.34 s CPU before, 3.11 s / 2.98 s after these four changes:

- The desugarings (`@derive`, `@reflect`, serialisation, `gen fn`, `where`, blanket extensions,
  `extern` blocks, status errors) each cloned every module's tree when any module needed it
  (~200 ms on bigpkg, most of it the 20,000-line C header's module). `LoadedModule.module` is an
  `Arc<ast::Module>` now, and each desugaring copies only the modules it rewrites
  (`Arc::make_mut`): ~20 ms.
- `ember check` ran the passes that place, hoist and remove the dynamic exclusivity checks
  (`[HEAP-5]`, `[EXC-3]`..`[EXC-19]`, `[DSJ-5]`): they change only the C, and the one thing a check
  reports of them is `@static_safe`'s verdict (`[EFF-13]`). They run in a check only when some
  function is `@static_safe` (~700 ms). No contract forbids an aliasing check (`[EFF-16]`), so the
  effect verdicts do not depend on them.
- `ember check` built the `[EFF-10]` safety side table it never writes (~80 ms).
- `[VERIFY-3]`'s re-derivation of every callable-region summary, a check of the compiler itself,
  ran in release compilers too (~680 ms, as long as deriving them): it runs in the compiler's own
  debug builds (the `ci` profile every test uses), as the MIR verifier does (`[IMP-11]`).

## Plan (in progress)

`[BLD-1]`/`[BLD-2]`/`[BLD-8]`: the module's checked form kept and reused. See "The incremental
front end" below once built.
