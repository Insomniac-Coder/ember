# Benchmarks

The programs behind the README's speed tables: each one written in Ember (`programs/`) and by
hand in C or C++ (`c/`, the same name), timed against each other with the same C compiler and
flags. The README shows the latest run, whose logs are in `results/`.
Every clang and GCC twin uses `-ffp-contract=off -fno-fast-math`, matching
Ember's strict floating-point policy across all program sets.

| Path | What it is |
|---|---|
| `programs/` | The 50 Ember programs. The first letter is the set: `p` objects and views, `a` everyday work, `b` and `w` list loops, `s` iterator chains, `t` text. |
| `c/` | Each program's hand-written twin. |
| `descriptions.py` | What each program does, as the README's tables say it, and the footnote marks. |
| `run_windows.py` | The Windows run: MSVC, then clang, on the 8 performance cores, on mains power only. |
| `gcc/run_gcc.py`, `gcc/make_tree.sh` | The gcc run in WSL, against a Linux copy of the compiler. |
| `run_all.sh` | Both runs, in order, with the logs in `build/bench/logs/`. |
| `update_readme.py` | The README's three charts (`docs/benchmarks/*.svg`) and its count of slow programs, from a run's logs. |
| `compare_runs.py` | Ember's times in one run against another, flagging anything more than 5% slower. |
| `bigpkg/generate.py` | `[BUD-1]`'s `bench/bigpkg`: a generated package of 400 modules, about 50,000 lines and 15 % generic code, with one C header import of Vulkan's size, 10,000 live instances and the export a host calls each frame; written under `build/` (500 MB once built: delete it after). |
| `bigpkg/measure_reload_build.py` | `[BUD-2]`'s B2 over it, the build half of `[HR-1]`: `ember build --reload` after a one-body edit, the median of 15 runs after 3 warm-ups, with each run's stage split. Runs nothing it builds. |
| `reload_overhead.py` | `[HR-9]`: each program built plain and reloadable (`[build] reload = "all"`), both timed on the performance cores; the suite's overhead (geometric mean) with its configuration, failing above 3 %. `--build-only` builds both forms and runs nothing. |
| `results/` | The logs of the run the README shows. |
| `probes/` | Ember programs with no twin, kept for one investigation: `text_loop_repeated_text.em` (one text of 0-60 bytes, millions of calls) and `text_loop_varied_text.em` (random English or Chinese text, or 10000 short random strings) price the text loop's ASCII step (D-498, D-550); `text_loop_scripts.em` (Russian, Greek, Arabic, Hindi, Japanese, Korean and emoji text, long and in short and medium strings) has a C twin and prices ADR-199 over other writing systems. Each reads `<mode> <count>` from standard input. `generator_pipeline` (a generator expression and a stepped range, as values) and `stepped_range_value` have C twins (`.c`) and price D-592 and D-593. `display_format` (a written `Display` formatted into a new `String`) has one and prices ADR-186: x0.62 of C on MSVC, x0.59 on clang (2026-10-16). `parallel_sum` (a `@parallel` reduction over 400 million turns) has one, the same loop split by hand into one thread per logical processor, and prices ADR-190: x0.99 of C on MSVC, x1.00 on clang (2026-10-16, after ADR-191 gave a chunk function its loop's range). |

## Running

From Git Bash in the repository, with `target/release/ember.exe` built:

    bash bench/run_all.sh

The owner's rules for a README run: one fresh run of the final code, on mains power, Windows on
the performance cores; a row a disturbance clearly hit may be timed again alone
(`python bench/run_windows.py <log> 21 --only <name>`, or `run_gcc.py 21 <name>` in WSL after
`make_tree.sh`), and that row replaces its line. Then:

    python bench/compare_runs.py bench/results/<old>-windows.log build/bench/logs/<new>-windows.log bench/results/<old>-gcc.log build/bench/logs/<new>-gcc.log
    cp build/bench/logs/<new>-windows.log build/bench/logs/<new>-gcc.log bench/results/
    python bench/update_readme.py bench/results/<new>-gcc.log bench/results/<new>-windows.log

`python bench/run_windows.py --list` shows each program and its twin without building anything.
Builds go to `build/bench/` (ignored by git); the WSL copy is deleted at the end of a run.

## Stable reference buffers

The `b16_checked` and `b16_wrapped` C twins use page-aligned buffers with offsets
0, 1024 and 2048 bytes. They allocate the same three logical arrays and execute the
same loops. On the benchmark laptop, ordinary allocation placement made MSVC's
unchanged interchanged loop vary from about 0.5 to 7 seconds. Controlled equal
page offsets reproduced the slow case; distinct offsets gave about 0.46 seconds,
with the same inner-loop instructions and checksum. This is consistent with
[Intel's documented false load/store address-alias stalls](https://www.intel.com/content/www/us/en/docs/vtune-profiler/user-guide/2024-2/cpu-metrics-reference.html).
The separated offsets give the comparison the faster, stable C reference;
they are applied under all three C compilers, with their usual flags.
The gaps also keep pending stores away from following loads in clang/GCC's
streaming loop; the initially tested 64-byte gaps slowed that loop and were
rejected.
