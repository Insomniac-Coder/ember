# Benchmarks

The programs behind the README's speed tables: each one written in Ember (`programs/`) and by
hand in C or C++ (`c/`, the same name), timed against each other with the same C compiler and
flags. The README shows the latest run, whose logs are in `results/`.

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
| `results/` | The logs of the run the README shows. |

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
