# The helpers stopped on 2026-10-23, and how to resume their work

The owner stopped all helpers on 2026-10-23: the language's memory model is to be reworked (C-like
performance, memory safety without Rust's annoying parts, Python-like syntax; the research is in
`tasks/96/redesign/`). Nothing of theirs is merged. Each helper's work, its unfinished part
included, is committed and pinned to a named branch, and its notes are copied to `tasks/95/`.

| Helper | Branch | Base on `main` | Commits past base | Notes |
|---|---|---|---|---|
| AI — the rest of the LLVM backend | `stopped/AI-llvm-backend` | `da5f4101` (before AE's merge) | 9 | `tasks/95/AI-notes.md` |
| AJ — a type of its own for every function and lambda (ODR-278) | `stopped/AJ-function-items` | `3f2fce5b` | 4 | `tasks/95/AJ-notes.md` |
| AK — the compiler's memory set by the biggest module (ODR-281) | `stopped/AK-compiler-memory` | `3f2fce5b` | 5 | `tasks/95/AK-notes.md` |

The last commit on each branch ("Stopped by the owner 2026-10-23 …") holds what was uncommitted
when it stopped: AI a comment in `ember_codegen_llvm/src/types.rs`, AJ its notes, AK its notes.

## What each one finished, and what it had not

**AI (`stopped/AI-llvm-backend`), mostly done.** Hot-reload images under the LLVM backend, the
planner shared with the C backend (`ember_codegen_c::image`), every run test under
`EMBER_BACKEND=llvm EMBER_PROFILES=all` 2129/2129 and the 50 reload tests; the lead's ports tried on
both backends (D-913); libraries (`cdylib`, `staticlib`) from the IR, 11/11 and 6/6; D-914 (a
`va_list` parameter on System V); ODR-275 (`UnsafeCell` for every std type changed through a shared
reference) and ODR-276 (`dereferenceable(N)`, `readonly`); ODR-274 (PGO); ODR-277 measured and not
built (no program changes); `extern "vectorcall"`, AAPCS64 classified (not run on AArch64). It was
writing its notes when stopped. Its notes end with "Merge notes for the lead" and one spec question
for the owner (section 2).

**AJ (`stopped/AJ-function-items`), built, measurements unfinished.** ODR-278 in the checker, MIR,
both backends, the evaluator and the codec: `TyKind::FnItem` (zero-sized, named by its symbol),
converting to `fn(A) -> R` where a callable type is wanted; the owner's `fast`/`make_list` example
builds, D-901's adapter case is accepted (`map` made per lambda). Not done: a foreign declaration's
value keeps the C pointer type (spec question 5); the C backend gives a zero-sized member one byte
(defect 3); `callable_values` kept. It was measuring the compiler's own figures when stopped.

**AK (`stopped/AK-compiler-memory`), part way.** The curve (`tools/memory_curve.py`, B9-curve in
`tools/compile_budgets.py`, B9's 3 GB removed), the counting allocator (`count-allocations`
feature), and four rounds of savings with every program's C and diagnostics byte-identical: 400k
lines, clean debug build, peak working set 4769 MB before, 1356 MB after (live heap 4294 MB to
979 MB). **The curve is not flat yet**: the type check, the MIR passes and C emission each hold the
whole program at once; flat needs them to work module by module from summaries (its question Q1).
Its notes' figures section was not written ("TBD").

## To resume

1. Merge in this order, each with the full check (`scratchpad/verify_nobench.sh`) before the next:
   AK (compiler only, smallest conflict surface), then AJ (it touches the checker, MIR and both
   backends), then AI (its base is before AE's merge: expect conflicts in `ember_codegen_llvm`,
   the driver and `bench.rs`; AE's merge resolutions are in commit `3f2fce5b`).
2. AI's and AJ's work depends on the type checker and backends, not on the memory model, so it should
   survive the rework; re-check each against the reworked rules before merging.
3. A source checkout builds through Ninja (`[BLD-4]`): put
   `C:\Users\ism19\AppData\Local\Microsoft\WinGet\Links` on PATH in Bash.
