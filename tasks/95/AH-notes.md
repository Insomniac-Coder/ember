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
  MIR writes each piece as `Builtin::PrintPiece { stderr, first, last }`, the first taking the
  stream (`ember_print_hold`, the C library's own recursive lock: `_lock_file` / `flockfile`) and
  the last letting it go (`ember_print_release`), each written with a `_held` printer that does
  not take the lock again (`_fwrite_nolock`, `fwrite_unlocked` on Linux; elsewhere the plain call,
  which takes the held lock again): one lock a statement where there was one a piece, so a
  `println(a, b)` takes one lock, not four, and the statement's blocks are as many as before (a
  hold of its own made a loop that prints two blocks longer, which cost gcc's written-out steps of
  a text loop, `milestones` `progress_reduction_…`). A `Display` a piece runs may print (the lock
  is recursive). `ember_print_T`/`ember_println_T` hold the stream for their own piece (`input`'s
  prompt uses it). The evaluator runs one thread: a piece is written as itself. A piece runs no
  Ember code (`L3011` and check hoisting read it as a print). Test:
  `STD-9/accept_a_print_is_one_write_between_threads.em` (four threads print one 37-piece line six
  times each; every line whole; `assert-c` for the hold; with the compiler before the change its
  lines mixed). C gate: clang, MSVC.
* **D-899 `[CLI-21]`, `[ECS-4]` — the evaluator's "invalid downcast" in `run_parallel`.** Not a
  null storage: the storage's handle was right and of the right class, but the evaluator read an
  `Assert`'s condition as a `bool`, its low byte, and a `Downcast`'s condition is the class handle,
  which held at `0x10c00` (the program's static content only moved where the object landed). The
  condition is now read at its own type (`Terminator::Assert` in `ember_interp`): a handle holds
  when it is not null, as C's `if (!raw)` reads it. Tests:
  `GRM-30/accept_an_asserted_downcast_holds_at_any_address.em` (300 objects, boxes between some, so
  several lie at addresses ending in a zero byte; the evaluator before the change panicked there),
  `ECS-4/accept_two_lambda_systems_capturing_one_sync_object.em` (AD's repro, reduced).

## Verification (at the last commit)

* The whole conformance tree (every `tests/conformance/*`, `tests/run-pass`, `compile-pass`,
  `compile-fail`, `run-fail`; the runner compares every run test with the evaluator): failing 0.
* `ember_driver`'s `milestones` (175, the conformance suite with its C assertions), `ui`,
  `firstweek`, `reload`, `interp`: all pass; the unit tests of `ember_typeck`, `ember_parser`,
  `ember_ast`, `ember_hir`, `ember_mir`, `ember_analysis`, `ember_codegen_c`, `ember_interp`,
  `ember_driver`: all pass.
* The C gate (warnings as errors): `GRM-28`, `STD-9` under MSVC and clang (Windows), and
  `GRM-28`, `STD-9`, `GRM-30`, `ECS-4`, `CLI-19`, `RNG-4` under gcc in WSL (a copy of this tree,
  built there; the same directories' run tests pass compiled by gcc); the runtime itself under gcc
  and clang on Linux, `-std=c11 -Wall -Wextra -Werror`.

## Not done

* **ODR-266 `[ECS-4]`, `[ECS-5]` — not built: the ruling's mechanism cannot meet the brief's
  constraint.** The ruling hands a system `run_parallel` runs "a commands type of its own whose
  `add` requires `Send`". A system reaches its commands through its query (`q.commands()`), and a
  function written for `World.run`, `fn move(mut q: Query[(Mut[Pos], Vel)])`, names its query's
  type, `Query[Q]`, and so its `Commands`: for `run_parallel` to hand it another commands type,
  its parameter would have to be another type (`Query[Q, ParCommands]`, `ParQuery[Q]`), or generic
  over one (`fn move[C: Queue](mut q: Query[(Mut[Pos], Vel), C])`) — the function rewritten,
  which the constraint forbids. (A hidden default type argument does not help: the function's
  parameter is one instance, not generic over it.) Nothing is changed: `Commands.add` keeps AD's
  check, a panic before the change is queued (`type_is_send[T]()` is a constant per instance, so
  for a `Send` component the test folds away; `World.run`'s systems pay a flag test only when
  they add a component that is not `Send`). Options:
  (a) **an effect-style check where a system is made** (`std.ecs.system(body, name)`, which
  `run_parallel`'s literal calls): the components `T` of every `Commands.add[T]` the body can
  reach, through the call graph the effect analysis already walks (`ember_analysis::effects`,
  with D-877's resolution of calls through `fn` values, closures by their bodies, `dyn` by its
  tables), must be `Send`, else `E7004` at the system naming the component and the call that adds
  it. As one more bit of the effect set ("adds a component that is not `Send`", set by the
  instances of `Commands.add` whose `T` is not `Send`, propagated as `Alloc` is) it costs nothing
  at run time and nothing to a program that makes no system; system functions and `World.run`
  are unchanged; the panic and `Commands.parallel` go. A call the analysis cannot follow counts as
  adding anything, as an indirect call counts as every effect today: the error then names it.
  (b) the ruling's commands type of its own: system functions rewritten (or written generic) to
  be passed to `run_parallel`.
  (c) as built (AD): a panic before the change is queued.
  (d) `T: Send` on `Commands.add` everywhere: `World.run`'s systems may no longer add a class
  handle.
  Recommendation: (a) — the only static form that keeps a system written for `World.run` usable
  in `run_parallel` as written, and free where unused. It needs a ruling (it replaces the
  ruling's mechanism), so it is not built.
* **D-901 `[EFF-1]`, `[STD-19]` — written up, not built.** A capture-free lambda or a function used
  as a value has the shape type `fn(A) -> R` (ODR-102), so an adapter holding it
  (`Map[I, fn(int) -> int]`) is one instance for every such lambda, its `next` calls through the
  field, and since D-877 that call has the effects of every function of the shape the program
  uses as a value: one allocating lambda anywhere makes every such adapter allocate. The precise
  forms:
  (a) **a type of its own for each function used as a value** (`[CLO-1]`: "a lambda … has a unique
  anonymous type"), as C++ and Rust give each closure: a zero-sized function-item type that
  coerces to `fn(A) -> R` where a value of that type is wanted (a field, an `Array`, `extern "C"`,
  ODR-102's owned callable value), and that a generic argument keeps: `map(f)` is then
  `Map[I, item f]`, an instance per lambda whose `next` calls `f` directly (inlined by the C
  compiler; no pointer in the adapter), and its effects are `f`'s. Cost: code size, one small
  instance per lambda an adapter holds (the C++/Rust trade, usually smaller once inlined), and a
  new kind of type through the checker, the backends and the evaluator. An ADR of its own.
  (b) effects per field: what an adapter's field can hold, followed from where each adapter is
  built (a field-sensitive points-to over constructors): narrower than the shape, still one set for
  every adapter of that instance (two `map`s of one shape, one allocating, still mix).
  (c) leave it (D-877's precision).
  Recommendation: (a). Not built: it changes what a function value's type is (ODR-102), which is
  not a defect fix's to decide.
* `[GRM-28]` — left as AD and the brief name it: a local function with a foreign ABI (`E0900`),
  and a generic local function calling itself with other type arguments (`E2020`, AD's).
* Local generators — limits known:
  * A call inside a closure is made once per check of that closure (its two capture probes and
    its body), each with that check's environment, so one or two instances more than run: dead
    `static` functions the C compiler drops; compile time only. (Generic local functions do the
    same.)
  * `Self` in a local generator inside an interface's default method: the lifted body has no
    `Self` to write out (the implementation's), so its instance reads `Self` as the frame; the
    declaration's check reads it right, and reports.
  * A local generator never called is not `L1001` (its name holds a marker no call reads).

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
* `ember_hir`, `ember_mir` (lowering), `ember_analysis` (effects, borrows, check hoisting, access),
  `ember_codegen_c`, `ember_interp`, `ember_driver`: `Builtin::PrintPiece`; the runtime's
  `ember_print_hold`/`ember_print_release`, the `_held` printers, `EMBER_OUT_*` (templates,
  regenerated).
* `ember_interp`: an `Assert`'s condition read at its own type.

## Defects found

## Questions for the owner

1. ODR-266 — the ruling's commands type of its own cannot be handed to a system function written
   for `World.run` without rewriting it (above). Options (a) an effect-style check where a system
   is made, (b) the ruling's mechanism with system functions rewritten, (c) the panic as built,
   (d) `T: Send` on every `add`. Recommend (a).
2. D-901 — a function used as a value given a type of its own (a zero-sized function item,
   `[CLO-1]`'s unique type), so an adapter is made per lambda and calls it directly, its effects
   the lambda's. Options (a) that, (b) effects per adapter field, (c) leave it. Recommend (a).
