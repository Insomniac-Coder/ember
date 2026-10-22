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

### The incremental front end: `ember check` (`[BLD-1]`, `[BLD-2]`, `[BLD-8]`, ODR-257)

`compiler/ember_driver/src/incremental.rs` (its module comment is the design). After a check that
reported no error, each module's checked form is kept in `target/<profile>/incremental/<package>/`,
one record per module, in **parts**: each function with a body of its own (with its closures),
each generic instance its bodies reach, and the rest of the module's check. A part holds its MIR as
the checks leave it (drops elaborated, region summaries installed), each body's region contract,
store summary and invalid-result flag, what its body asked of the program (`ember_typeck::Kept`:
the instances it calls, the `dyn` values it formats), the warnings its type check and its MIR checks
reported, and **the facts of each body it calls as they were** (`[BLD-2]`'s facts: the region
contract — a view result's sources, a method's field set, what it stores — and whether its result
can be followed).

A later build keeps a part while:

- the key holds: compiler identity, command, profile, manifest, and the **open-world declarations**
  (every `extend` and every class, by module): a subclass or an implementation elsewhere changes what
  a virtual call or a bound reaches;
- the module's **declarations** (its source with its ordinary functions' bodies cut out) and those
  of everything it imports, directly or not, are unchanged;
- the **text each of its spans points at is unchanged, wherever it now is**: each file a record
  points into is cut into pieces — the text between bodies, each ordinary body (its block, then the
  rest of its line and any blank and comment lines up to the next line of code) — and a record names
  the pieces it points into, a body's with its hash. Equal declarations mean the same pieces in the
  same order; a piece that moved is relocated on reading (`ember_span::codec::Relocation`), one whose
  body changed is not used. So an edit in one function moves every later one, and only the edited
  one is checked again;
- the facts it recorded for each body it calls are those bodies' facts this build: a build assumes
  so, and once the bodies checked again are, every kept part whose recorded facts changed is checked
  again by another attempt (`Attempts::forced_parts`).

A module whose source is unchanged and all of whose parts hold is kept whole; one with equal
declarations is checked in part: the type checker skips the bodies whose parts hold
(`ember_typeck::Reuse::partial`, each function's part of the check tracked by
`ember_typeck::FunctionCheck`), and the rest of the module is checked; a module whose declarations
changed is checked whole, as is every importer of a changed declaration. A kept form's types are
found again by name; a closure's environment only where the body it is written in is kept (one in
a generic's body, which every build checks, or in a body checked again, is made by that check).
Generic instances are always type-checked again (the package is the unit of monomorphisation); a
kept instance's MIR replaces the fresh one's when its part holds. Kept warnings are reported again
where a clean build reports them: the type check's after the type check, the MIR checks' after the
MIR checks (a type error stops a clean check before the latter). A check that asks the
compile-time evaluator for a value is made whole.

`--timings` gains `modules kept`, `modules checked` (whole or in part) and `functions checked` (in
the modules checked in part).

Tests: `compiler/ember_driver/tests/incremental.rs` (`[BLD-1]`, `[BLD-2]`, `[BLD-8]`): a package
of four modules through ten edits, each check compared with a clean check of the same sources
(identical report and status) and the modules and functions checked again asserted — a body: that
function alone; a body above a generic an importer instantiates: that function alone (the importer's
kept instance relocated); a body changing a borrow fact: it, its caller in its module and its caller
in the importer; a leaf's signature: the leaf and its importer; a type: every importer; a subclass:
every module; a type error and a borrow error in one module (the kept warnings reported as a clean
check reports them); the error mended (nothing checked again). `ember_span`'s codec tests
(relocation, scoped extents). Two equivalence runs over every program of `tests/conformance`,
`compile-fail`, `run-pass`, `compile-pass`, `milestones` and `run-fail` (3,355 programs), each
comparing an incremental check with a clean one: everything kept (0 differ), and an edit of the
first function's body, every later function moved down a line (3,171 edited, 0 differ).

Defects found and fixed on the way (all in this branch):

- **An ICE in the interface cache (`[LT-40]`)**: adding a subclass that overrides a virtual method
  of another module's class made the next build fail with "cached callable-region metadata
  disagrees with the current verified summary" (main has it too: `build/ember-base.exe` reproduces
  it). A module's callable summaries read the program's class hierarchy, which its cache key did not
  hold; the key now holds the open-world declarations' hash.
- Two nondeterministic borrow diagnostics (the culprit of an `E3063`, the "keeps it alive" help of a
  conflict) chose among equals by hash order; they choose by source order now.
- A test's subclass was of a final class (it did not compile, which the test did not notice); the
  test now asserts each step's status.

## Plan (in progress)

`[BLD-4]`/ODR-244: `ember build` through the kept forms, one C unit per module in `debug`, Ninja.
