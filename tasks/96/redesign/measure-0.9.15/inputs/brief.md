# Brief: measuring the costs spec 0.9.15_Hardened_1 left "not measured" (read this first)

## The goal

Ember (repository `C:\Users\ism19\Code\ember`) is a compiled language: Python-like syntax, compiled to C
or LLVM IR. Its owner reworked the memory model. There were three goals at once:
- as fast as C and Rust;
- as memory-safe as Rust or safer, checked when compiling;
- readable like Python.

The result is the spec `docs/spec-source/Ember_v0.9.15_Hardened_1.md`. Section 2 of the design's rules,
`tasks/96/redesign/run4/v5/rules.md` (the table "what each safety costs"), lists every run-time cost the
new model keeps. Several rows say **not measured**. Your job is to write the programs that measure your
share of them, faithfully and fairly: what plain C does, what Ember will emit, and what Rust does.

The owner's rule for a cost that falls short (`tasks/96/redesign/JOB.md`, "before considering anything
as a failure"):
- first ask whether the idea is good and only the implementation is weak;
- try to fix that: a better layout, a check proven away, a cheaper mechanism for the same guarantee;
- measure the fixed version too;
- keep the good parts of anything discarded.

So besides the model of the rule as written, write the **salvage variants** you can see. Each must keep
the same safety; say exactly how.

## Where things are

- **The cost table:** section 2 of `tasks/96/redesign/run4/v5/rules.md`. The rules behind each row are
  R1 to R18 in the same file. The words are in `run4/v5/vocabulary.md`. The owner's verdicts 1 to 100
  are in `tasks/96/redesign/JOB.md`; they are **decided, never re-open them**.
- **The spec's text for each mechanism:**
  - `tasks/spec-0.9.9/parts/*.md`; the rule IDs are in the cost table's rows and in the H.41 table of
    `p26-appx-h.md`;
  - the cost tables are in the COST family (`[COST-3]`, `[COST-6]`);
  - the count limits are in `[TST-31]` and the RC/WK families.
- **Earlier measurements to stay comparable with:**
  - run 3's timer report `tasks/96/redesign/run3/26-timer.md` and `run3/combined-v4.md` (the rows the
    cost table cites);
  - run 3's models `target/redesign3/25-c-writer-fix/w1.c` .. `w4.c` and the build/run scripts in
    `target/redesign3/26-timer/`;
  - the earlier workloads in `tasks/96/redesign/artifacts/` (`bench/plain/`, `bench/D1..D3/`,
    `redesign2/bench/rust_w2.rs`).
- **The harness:** `tasks/96/redesign/artifacts/bench/harness.h` (Windows x64). `h_start()` first,
  `h_report(name, checksum)` last. It prints `<name> ms=<wall ms> peak_kb=<peak working set KB>
  checksum=<u64>`, and its `h_rand()` is the one random source. Copy it into your folder, unchanged. A
  Rust program prints the same line format itself, with the same random generator ported exactly.

## What you write

All sources go under `tasks/96/redesign/measure-0.9.15/<your job>/`.

**1. For each cost row of your job, a workload.** It must be:
- big enough to run 100 to 1000 ms;
- deterministic, with one checksum every program of the workload must print;
- realistic for the rule: run 3's workloads where they fit, so the numbers compare.

Each workload has these programs:
- `plain_<wl>.c`: what a careful C programmer writes, with no safety checks;
- `model_<wl>.c`: exactly what Ember emits under the rule as written. Every check, count, compare, test
  and word must be there, each commented with the rule (spec ID and R-rule) that requires it. Leave out
  anything the spec lets the compiler prove away in this workload; if the proof is not certain, write
  both forms;
- `rust_<wl>.rs`: idiomatic safe Rust (`Rc`, `Weak`, `Arc`, a hand-written arena; no external crates:
  plain `rustc`, no cargo), plus an `unsafe` form where it is informative;
- `salvage_<wl>_<idea>.c`: your salvage variants.

**2. `build.txt`.** The exact build line of every program:
- C: `clang -O2` (the LLVM at `C:\Program Files\LLVM\bin\clang.exe`);
- Rust: `rustc -C opt-level=2 --edition 2021`, as run 3 did;
- output into `target/measure-0.9.15/<your job>/bin/`.

The lead also builds the C with MSVC `cl /O2`, so keep the C portable to MSVC: no GCC-only extensions,
no VLAs; `__declspec(noinline)` / `__attribute__((noinline))` behind a macro.

**3. `runs.txt`.** One line per program: the binary name, its arguments, and the expected checksum.

**4. Your report, `tasks/96/redesign/measure-0.9.15/<your job>.md`.** For each workload:
- the question it answers;
- each program, and what it models (with rule references);
- why the comparison is fair: same work, same layout, or why a layout differs;
- what result would mean "as fast as C", and what would mean "falls short";
- the salvage variants, and the safety argument for each;
- memory: what each program's peak should be and why (the owner judges speed **and** memory).

You may compile your programs with clang and rustc to make sure they build (into the bin folder above).
**Never run any program you build**: the lead runs every timing, one at a time, on the laptop's fast
cores and mains power. Never use `cmd /c start`. Write nowhere but your two folders and your report. No
git writes, no cargo, no `ember`. Do not touch anything outside this repository.

When done, return the summary your prompt's schema asks for.
