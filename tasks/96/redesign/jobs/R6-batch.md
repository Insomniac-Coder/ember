# Round 6: four helpers (three Opus, one Sonnet), 13 to 16 tasks each

Sizes: **D (Opus deputy) 14 (done: `0.9.20_Hardened_13`, N171 to N175), H1 (Opus) 13, H2 (Opus) 16, W1 (Sonnet) 18**
(61 tasks in all; W1 grew from 14 to 18 with four of Hardened_13's new debts). The lead
coordinates: reviews, merges, the push gate and the bench. W2's 12 tasks (below) are queued for round 7, so that
only three helpers build at once.

The owner's rules for this round (2026-11-12):
- up to 4 helpers at once;
- Sonnet does light and medium development and never rules a spec question: it reports the question with
  options, and the lead or an Opus agent decides;
- Opus takes the hard problems, reviews, merges and the spec rulings (ODRs);
- each agent gets 10 to 30 tasks; the lead collects, tests and combines each batch;
- the laptop rests 10 to 15 minutes after each batch; use the CPU with care.

**The rules of `N-batch.md`, "Rules for every job", apply to every task here**, with these changes:
- Use `CARGO_BUILD_JOBS=4 RUST_TEST_THREADS=4`: three helpers and the lead build on the same machine.
- Branch from local `main`. Your worktree may start from `origin/main`: run `git reset --hard main`, then
  check that `git log -1` shows the base commit the lead gives you.
- Build in your worktree's own `target/`, and delete it at the end.
- Commit each task separately, with `--author="Gunslinger <ism1988@live.com>"`.
- The target is `docs/spec-source/Ember_v0.9.20_Hardened_13.md` (cut this round by the deputy; `decisions.md` §24). Write no `Box` anywhere: one owner in a block
  of its own is `owns T`.

A task that cannot be done as the spec says is reported with its rule, the program, what the compiler
does and what the spec says. **Never leave files outside your worktree.**

**M4's files now (H2's; nobody else touches them):**
- `ember_types/src/lib.rs`'s `Link` kinds, `Shared`, `SyncShared` and `Weak` (`counted_of`, `weak_of` and
  their callers in `ember_typeck`), and `ember_typeck`'s `synth_link`, `read_link_through`, `open_link`;
- `ember_hir`'s and `ember_mir`'s `SharedNew`, `WeakNew`, `WeakUpgrade`, `WeakEmpty`, `LinkMake`,
  `LinkTarget`, `LinkOpen`;
- `uncounted_handles.rs` and `temporary_owners.rs`;
- class objects' copy and retain (A1-9).

**The deputy's files now (do not touch):** `tasks/spec-0.9.9/`, `docs/spec-source/`,
`tasks/96/redesign/compiler-0.9.15/decisions.md`, `docs/AUDIT-0.9.20.md`, `docs/OWNER-QUEUE.md`.

**Ruled in Hardened_13 (N171 to N175), built where a job below says so:** `E2021` for arguments that do not fit
(`[TYP-25]`, `[STR-1]`, `[CLS-3]`); `[DSJ-5]`'s stop; a `unique_ptr`'s deleter as the one destructor
(`[FFI-17a]`, `[FFI-36]`, `[FFI-36a]`); B4's `new_n` (`[BRW-14]`); which types are foreign (`[FFI-36a]`).

## D (Opus deputy): the spec findings, ruled, and `0.9.20_Hardened_13` (14 tasks)

Started first, during the push gate, on its own branch. No builds. Its brief is the lead's message to it:
the 14 findings above, each classified as an implementation gap, an ODR it rules, or a question OPEN for
the owner; then the cut by the recipe in `docs/HANDOFF.md`. The lead reviews the branch and runs
`check_examples` before the merge.

## H1 (Opus): `[TYP-47]`, typed variadic generics and functions (13 tasks)

Verdicts 115, 116 and 117 (`tasks/96/redesign/JOB.md`). Read whole first: `[TYP-47]`, `[TYP-27]`, `[MONO-1]`,
the grammar lines that say "pack" (`generic_param`, `param`, the type arguments, `tuple_elem`,
`fn_type_param`, `arg`), `[STD-9]`, `[STD-18]`'s `format_to`, the `zip` line of the built-ins list, and the
mangling rules (`[MNG-*]`). Today nothing of it is built.

1. **The grammar.** `...` after a type parameter's name (`[Ts...: needs Display]`), after a parameter's type
   (`values: Ts...`), among type arguments (`View[Cs...]`), tuple elements (`(Entity, Cs...)`), function
   type parameters (`fn(Ts...)`), and among a call's arguments (`log(values...)`).
2. **Where a pack may be:** the last type parameter, and its values the last positional parameter; a
   parameter after it has a default and is passed by name only. Anywhere else, two packs, and a type
   formed over each type of a pack (`Option[Cs]...`) are `E0100`, with the rule's help.
3. **Spelled out.** A type written over a pack is formed once the pack is spelled out: a tuple of two or
   more elements is a tuple, of one is that type, of none is `void` (`[TYP-27]`).
4. **The needs** apply to each of the pack's types.
5. **Calls.** Any number of arguments for the pack's values, each of its own type, each checked as an
   ordinary parameter's is, its word included (`owns values: Ts...` takes each over); `values...` passes
   them on; `values.len()` is a number known when compiling.
6. **`for v in values:`** is written out once per value, each `v` of its own type; nothing runs for the
   loop itself.
7. **A `struct` or `class` over a pack** (its last type parameter, `[Cs...]` above `struct View:`).
8. **`extend (A, B, Ts...) implements I:`**, a tuple's interfaces written once from two elements up.
9. **Instances and names.** Each use is made for its exact types (`[MONO-1]`), with the symbols `[MNG-*]`
   gives; C, LLVM and the evaluator agree.
10. **`Printable`** (§IV.8, in the prelude): the compiler implements it for every type with `Display` or
    `Debug`, and no program can write it by hand; a type with neither is `E2040` with the rule's help.
11. **`print`, `println`, `eprint`, `eprintln` and `format_to` as library functions over a pack** (verdict
    117), not known to the compiler by name, with the output byte for byte as today. Measure a
    print-heavy program before and after (instruction counts, or the emitted C): no slowdown is
    acceptable; report any difference with its cause.
12. **`zip(a, b, …)`** of any number of iterables, stopping at the shortest; `zip(xs)` gives each item
    plainly.
13. **Tests and pages.** The rule's own examples as conformance tests (accept, reject, run), the `E0100`
    cases, mixed types and words; error pages for any code that gains a case; `rule_index` and
    `error_pages` green.

**Effort.** Opus at the lead's level. Report every choice the spec does not force, and every spec question
(do not rule it: the lead or the deputy does).

## H2 (Opus): M4, the rest of slice A and slice B (16 tasks)

Branch from `lead-m4` (`21cf9ab7`: steps 1 to 3 plus `main` merged in). Read whole first:
`tasks/96/redesign/compiler-0.9.15/M4-plan.md`, `gap/03-sharing-links.md` §3 (D1 to D10), and the rules `[RC-1]`,
`[RC-7]`, `[OWN-13]`, `[WK-11]` to `[WK-18]`, `[TYP-45]`, `[OBJ-1]`, `[RT-7]`. Steps 1 to 3 are in the branch's
commits (`0f9b3baf`, `979328a8`, `3032c550`, `087ac833`): read their diffs. Keep `M4-plan.md`'s progress
section current as each step lands.

0. **The merge first:** build, the quick check and `cargo test -p ember_typeck -p ember_mir -p ember_codegen_c`
   on `21cf9ab7`; fix what the merge broke, and report it.

Slice A, the rest (one thread):
1. **`E2122`** with `[RC-1]`'s messages: `shares`/`shares mut` of a list element, a field, a plain parameter,
   or `this` in a method (its own message).
2. **`E3023`** for `shares mut x` from a read-only name.
3. **`E2119`** (`[WK-11]`, `[WK-16]`): `weakref` of a value held `owns`, of a part, of a new value; `weakref`
   of a value held `dyn I`, `owns dyn I` or `some I`, and the type `weakref dyn I`, each with its message.
4. **`E2020`** (`[WK-16]`): a `weakref` link used as its target without opening it; an object stored into a
   `weakref` place without `weakref`; the two link kinds never convert (`[TYP-45]`).
5. **Re-pointing a link** gives the old target's weak count back; a link may hold no target
   (`parent: weakref Node = None`), as a field, a local and a parameter.
6. **The limits** (`[RT-7]`): before each owner +1 and each weak +1, the compare and the stop texts
   ("too many owners of a `Player` (4,294,967,295)", "too many `weakref` links to a `Player` …").
7. **Targets that are not class objects** (D5): a value made shared lives in a block with `[OBJ-1]`'s header
   from the line that declares its local (`xs = make()` puts the result there), with the local's type the
   shared type through the whole function; an `owns` parameter shared gets its block at the function's
   entry; `shares` before a new value. C, LLVM and the evaluator, on today's `Shared[T]` block machinery.
8. **The tests:** the design tests R9 and R11 (one thread), and `[RC-1]`'s, `[RC-7]`'s and `[WK-11]` to
   `[WK-13]`'s own examples, each on both backends and the evaluator; no leak in a debug build.

Slice B, `Shared` and `Weak` removed (`SyncShared` stays until slice E builds `thread_shared`):
9. **`Shared[T]` and `Weak[T]` named anywhere are `E1010`**, whose message gives what to write
   (`[TYP-45]`: `shares T`, `shares mut T`, `weakref T`); `.upgrade()` is `E1010` with `exists as`.
10. **Migration edits**, as A1-6 made for `Box`, so that `ember __migrate --checker` rewrites them.
11. **The repository migrated** with `ember __migrate --checker`: tests (TYP-22's `Weak[Shared[…]]` files
    among them), std, examples, docs' programs. Check each rewritten test still tests its rule.
12. **`counted_of`'s and `weak_of`'s `Shared`/`Weak` paths and the builtins `SharedNew`, `WeakNew`,
    `WeakUpgrade`, `WeakEmpty` deleted** where nothing but `SyncShared` needs them; say what stays for it.
13. **Error pages** for the codes that gained a case (`tools/error_pages.py` green).

Slice C, measured only:
14. **Count** the tests and std code that keep two owners of one class object without saying so (each
    copy of a handle that slice C will make a hand-over or a refusal). Report the number and the list by
    folder; rewrite nothing. The owner hears the number before the rewrite (`M4-plan.md`).
15. **The Rust suites** of every crate touched, and the quick check, green on your branch.

**Effort.** Opus at the lead's level: the lead's own milestone. Report every choice the spec does not force,
and every spec question with its options (the lead or the deputy rules it).

## W1 (Sonnet): defects, types and messages (14 tasks)

For each defect read its row in `docs/DEFECTS.md` and the rule in the pinned spec first. Some are older than
0.9.20 and may no longer hold (a retired spelling, a rule that changed): then write the program in today's
spelling, say what happens, and propose closing it with the reason. If a fix needs a choice the spec does not
make, stop that task and report the question with the options (as code). You do not decide it.
When a defect is fixed, update its row (status **fixed**, how verified).

1. **D-720** `[TYP-12]`: an enum with no `@repr` takes the smallest *signed* type that fits.
2. **D-721** `[TYP-1]`: `E2130` only for its own condition; the other three get their own codes.
3. **D-854** `[TYP-36]`: `Array[T]` and `[T; N]` are `Ord`, lexicographic.
4. **D-853** `[DIA-14]`: no cascade "`(<error>, <error>)` does not implement `Ord`".
5. **D-852** `[GRM-17]`: `h = fn(x): total += x` on one line parses (ODR-209: the body ends at the newline).
6. **D-860** `[MOD-5]`: a program's own `struct int` (and `float`, `str`) shadows the prelude alias in a
   type position.
7. **D-835** `[DIA-14]`: an unresolved field type reported once in a reloadable build.
8. **D-707** `[STD-6]`: `std.process.args` counts as using the `io` layer.
9. **D-691** `[TCB-1]`: "type has no foreign representation" is `E5015`, not `E5050`.
10. **D-699** `[DRP-1]`: check under today's `fn drop():` and `this`; fix or propose closing.
11. **D-864** `[DSP-*]`: `Box` is gone; check that an `owns dyn I` reads as `ref dyn I` where one is wanted;
    fix or propose closing.
12. **D-850** `[CLO-6]`: `owned` is gone; check an `owns f: fn(A) -> R` parameter called in a loop; only
    `once fn` is single-call. Fix or propose closing.
13. **E3013's help for a captured name (`[CLO-6]`)** is the rule's exact line, with the argument's text
    (round 5 left it without).
14. **`ember_diag::shapes`' B8, O5 and X1** are still mapped from retired codes: map them from the codes
    that replaced them (the spec's retired-codes table), or report why not.

Hardened_13's debts (`decisions.md` §24):
15. **`E2021`** (N171, shape N6) for every call whose arguments do not fit: a positional argument after a
    named one (with the line to write), too many, a parameter with no default given none, a name no parameter
    has, one parameter given twice; at any call, a memberwise constructor and `super.init(…)`. It replaces the
    `E2020` and `E1030` given today; the tests that expect those are rewritten to it (list each); its page,
    and its entry out of `tools/rule_index_baseline.json`.
16. **`W0001` retired** in `codes.rs` (its spec row is among the retired codes); `ember_diag`'s unit test
    moves to a live code.
17. **`[OBJ-2]`:** `E2173` for a bare interface name in a type position whatever implements it
    (`Array[Shape]` with a class implementer is accepted today).
18. **`[BRW-14]` shape B4** (N174): the local is `new_n` (`new_n_2` and on where taken), made by the refused
    line when it is a plain top-level `=` and no line above reads the parameter, else by a new first line;
    every later use rewritten, each line named; and BRW-14's message itself.

## Queued for round 7 — W2 (Sonnet): defects, names, C output and FFI messages (16 tasks)

The same rules as W1's.

1. **D-859** `[MNG-3]`, `[CG-C-7]`: fields `café` and `caf_u00E9_` of one struct get distinct C member names
   (ODR-213 applies to C members).
2. **D-862** `[MNG-3]`: the transliteration never merges two names inside one component (ODR-213).
3. **D-863** `[MNG-1]`, `[MOD-1]`: write the test (a root-package module and a dependency package of one
   name); fix if their symbols can meet.
4. **D-882** `[DOC-4]`: `docs/errors/E5053.md` and `W5054.md` pass `ember test --doc` (how a page's
   program asks for `--instrument-ffi`, as the rule allows).
5. **D-834** `[FFI-33b]`: `ForeignBox` is gone; check `static S: Mutex[Array[owns b.Widget]]` and fix or
   propose closing. (Leave E7004 against E7008 alone: it is under ruling.)
6. **D-693** `[FFI-10]`: a hand declaration taking `long long` compiles with gcc on Linux (WSL: see
   `docs/HANDOFF.md`'s WSL recipe; one gcc build at a time).
7. **`[FFI-33b]`'s stop names the variable**, as the rule's text does (round 5 left it naming only the type).
8. **"The line that made it"** for a `returns_owned` result is the call's line, not the declaration's
   (`[FFI-33b]`).
9. **`[FFI-35a]`'s header mark** (round 5 left it unbuilt): read the rule; build it or report what it needs.
10. **`E5060`'s meaning** in the pinned spec's code table: the compiler emits it as the rule says, or report.
11. **`JOB-5`'s two `E3063` on one line** (`[DIA-26]`: one mistake, one error).
12. **N173 in the importer:** a C++ class ended both by `delete` and by a `unique_ptr`'s `D` is `E5002`,
    naming both and giving the overlay's `hide` line (today the deleter's thunk replaces the class's
    `destroys`, so an object a constructor made with `new` is freed by `D`).
13. **N175:** `collect_foreign_results` takes only `@layout(c)` structs a contract gives a destructor;
    `returns_owned` of any other type is `E5002`; `owns` before an Ember-made foreign struct is `E2114`; that
    `owns T` is not `Clone`.
14. **`E7008`** for a thread-bound foreign owner sent to another thread (`E7004` today); `threads(main)` types
    are not `Send`.
15. **`[DIA-2]`:** a type is named as the program spells it (`int`, `[int; 3]`): the type table keeps the
    written spelling of an alias for messages.
16. **The checked helpers' `__builtin_*_overflow` calls** live in `ember_rt.h`, not in the emitted C (found in
    round 4). Find the rule that says where they belong (`[CG-C-*]`); fix or report.
