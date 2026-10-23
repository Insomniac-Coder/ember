# Helper AK — the compiler holds only what the module it compiles needs (ODR-281), 2026-10-23

Branch `worktree-agent-ade8cf47fe54e8adc`, from `main` at `3f2fce5b` (after helper AE's merge).
Rules: `[BUD-1]`, `[BUD-2]`, `[BUD-3]` and `[BLD-1]`, `[BLD-2]`, `[BLD-7]`, as ODR-281 rules them.

## Summary

ODR-281: the compiler's peak memory is set by the biggest module, not by the project; the proof is
a flat curve of peak memory against project size. **The curve is not flat yet.** This branch:

* finds what the compiler holds, stage by stage and structure by structure (a counting allocator
  behind a cargo feature; `--timings` then gives each stage's live and peak megabytes);
* removes what grew faster than the project (three structures grew with its square) and every
  copy and every list's slack that no later stage reads, every result byte-identical (the C of
  every unit, the diagnostics, `-j 1` against `-j N`, incremental against clean);
* builds the curve (`tools/memory_curve.py`; B9-curve in `tools/compile_budgets.py`, which passes
  only when it is flat), removes B9's 3 GB and gates B9 before/after as `[BUD-3]` gates the times;
* names what still grows with the project, and why: three stages read every body of the program at
  once — the type check (one `Checker` over the whole program), the MIR passes after it, and C
  emission — so at each of their boundaries the whole program is held in that stage's form. Flat
  needs those stages to work module by module from summaries, which decides some of the generated
  C: question Q1.

TBD: the figures.
