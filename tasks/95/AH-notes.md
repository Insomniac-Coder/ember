# Helper AH — local generators and parallel commands, 2026-10-23

Branch `worktree-agent-af62646ba2cbe7004`, from `main` at 4fd9acfb (after helper AD's merge).

## Done (tests)

* **ODR-268 `[GRM-28]`, `[GRM-21]` — a generator written in a block.** AD's design (a), as ruled.
  * The coroutine desugaring (`coroutines.rs`, `Writer::lift_in_item` … `lift_local`) finds every
    `gen fn` in a block (the parser marks a module holding one, `Module::local_generators`, so a
    program without one costs nothing), innermost first, and writes it out at module level twice,
    generic over the enclosing declarations' type parameters (owner's, function's, enclosing
    generic local functions'), a hidden `·E`, then its own: `count.local.env[…, ·E, …](owned ·env:
    ·E, n: int)` and `count.local.plain[…, ·E, …](n: int)` (whose `·E` is phantom), each with its
    frame, `Iterator` implementation and body method as any `gen fn`, `Self` written out as what it
    names where the generator is written. The statement stays in the block, its `yield`s the
    transform's calls, for the checker. (`[BLD-2]` tells callables apart by span: the second form's
    spans are the empty ones where the first's start.)
  * The checker (`check_local_generator`) checks the body there as the named closure it is
    (`synth_lambda_fresh`, own type parameters opaque, its own name a frame over an opaque `·E`):
    the check that reports, and that finds the captures and their access (`[CLO-2]`). That
    closure, and every one made in it, is never emitted. Its name holds a marker of no value; each
    captured place gets a hidden alias in the block's scope (a capture of an enclosing closure: in
    that closure's watch, `CaptureWatch::aliases`), so a later local of the same name is not it.
  * A call (`synth_local_generator_call`) makes the environment there — `ref` or `ref mut` of each
    captured place, as `[CLO-2]` decided — and instantiates the `env` form with `·E` = that
    environment (or, capturing nothing, the `plain` form with `·E = void`): so the captured locals
    are borrowed while a generator made of them lives (`[BRW-2]`), not while the name is in scope
    (a read between two generators is accepted; a write while one is still to be resumed is
    `E3021`). A closure in the block calling it captures the aliases. Its name read as a value is
    the closure `fn(params) => name(params)` (`E2060` for one with type parameters of its own).
  * An instance's body is checked per instance (`enter_local_generator_body`): its free names
    read through `·env` as a closure's through its environment, `self` its method's (the frame's
    receiver hidden), its own name the instance with the environment it was given again. Neither
    form is checked opaquely (`check_generic_type_methods` skips their extensions); an instance's
    diagnostics are those its declaration's check did not give (as a generic type's method's).
  * The frame holds what it was given: `(*self)._u00B7_env.total` is one load more than a local,
    the captured place's pointer; a generator capturing nothing has the frame a module-level one
    would (the C names the `plain` form; `assert-c`).
  * Tests (`tests/conformance/GRM-28/`, run tests in `debug`, `development` and `release`):
    `accept_a_local_generator_reads_what_it_captures.em`,
    `accept_a_local_generator_changes_what_it_captures.em` (the local read after the loop, the
    generator called again), `accept_a_local_generator_capturing_nothing_needs_no_environment.em`
    (`assert-c`, and its name as a value), `accept_a_generic_local_generator_is_made_for_each_use.em`
    (own type parameters, in a generic function, calling another local generator),
    `accept_a_local_generator_called_from_a_closure.em` (lambdas, a lambda in a lambda, a loop
    variable of a captured name), `accept_a_local_generator_calls_itself_and_nests.em` (recursion,
    one inside another, a method's `self`, `Generator[Y, R]`),
    `reject_a_local_generator_borrows_what_it_captures_while_it_lives.em` (`E3021` twice, and
    `[CORO-6]`'s `E2221` for an inner generator capturing the outer's parameter across a `yield`),
    `reject_a_local_generator_body_reports_once.em` (one report per mistake though called twice;
    `E2020`, `E2220`, `E2222`, `L2004`), `reject_the_local_function_forms_not_built.em` rewritten
    for what is not built: a local function with a foreign ABI (the parser now reads `extern "C"
    fn` in a block, so it is `E0900` by name rather than `E0100`). `CLI-19`'s unbuilt construct is
    that one now. C gate: the GRM-28 programs under clang and MSVC, no warning.
  * `phase-status.json` `GRM-28`: says what is built; left are the foreign-ABI local function and
    a generic local function calling itself with other type arguments (AD's, `E2020`).

* **D-900 `[STD-9]` — two threads' multi-argument `println`s interleaved their pieces.** Each piece
  (argument, separator, end) was its own `fwrite`, each taking the stream's lock; another thread's
  piece could come between. Now a print statement holds its stream from its first piece to its last:
  MIR brackets the pieces with `Builtin::PrintHold { stderr, start }` (`ember_print_hold`,
  `ember_print_release`: the C library's own recursive lock, `_lock_file` / `flockfile`), and each
  piece is written with a `_held` printer that does not take the lock again (`_fwrite_nolock`,
  `fwrite_unlocked` on Linux; elsewhere the plain call, which takes the held lock again): one lock
  a statement where there was one a piece, so a `println(a, b)` takes one lock, not four. A
  `Display` a piece runs may print (the lock is recursive). `ember_print_T`/`ember_println_T`
  hold the stream for their own piece (the prompt of `input` uses it). The evaluator runs one
  thread: `PrintHold` is nothing there. Test: `STD-9/accept_a_print_is_one_write_between_threads.em`
  (four threads print one 37-piece line six times each; every line whole; `assert-c` for the hold;
  with the compiler before the change its lines mixed). C gate: clang, MSVC.
* **D-899 `[CLI-21]`, `[ECS-4]` — the evaluator's "invalid downcast" in `run_parallel`.** Not a
  null storage: the storage's handle was right and of the right class, but the evaluator read an
  `Assert`'s condition as a `bool`, its low byte, and a `Downcast`'s condition is the class handle,
  which held at `0x10c00` (the program's static content only moved where the object landed). The
  condition is now read at its own type (`Terminator::Assert` in `ember_interp`): a handle holds
  when it is not null, as C's `if (!raw)` reads it. Tests:
  `GRM-30/accept_an_asserted_downcast_holds_at_any_address.em` (300 objects, boxes between some, so
  several lie at addresses ending in a zero byte; the evaluator before the change panicked there),
  `ECS-4/accept_two_lambda_systems_capturing_one_sync_object.em` (AD's repro, reduced).

## Not done

## Compiler changes

* `ember_ast`: `Module::local_generators`. `ember_parser`: the flag; `extern "C" fn` (and `unsafe
  extern "C" fn`, `extern "C" gen fn`) as a block statement.
* `ember_typeck`: `coroutines::LocalLift` and the lifting; `generator()` takes what `Self` names
  and whether to lint; the checker's `local_lifts`, `local_frames`, `local_generators`,
  `check_local_generator`, `local_generator_env`, `synth_local_generator_call`,
  `synth_local_generator_self_call`, `call_local_generator` (the environment reaches
  `synth_generic_call` as an argument already checked, `LOCAL_ENV_VALUE`),
  `local_generator_value`, `enter_local_generator_body`; `CaptureWatch::aliases`, read by
  `resolve_capture`.

* `ember_hir`, `ember_mir` (lowering), `ember_analysis` (effects), `ember_codegen_c`, `ember_interp`:
  `Builtin::PrintHold`; the runtime's `ember_print_hold`/`ember_print_release`, the `_held`
  printers, `EMBER_OUT_*` (templates, regenerated).
* `ember_interp`: an `Assert`'s condition read at its own type.

## Defects found

## Questions for the owner
