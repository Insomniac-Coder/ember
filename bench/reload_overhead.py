"""`[HR-9]` — what hot reload costs a program that is not reloading: the performance suite
(`programs/`) built twice by Ember's `development` profile, once plain and once reloadable
(`[build] reload = "all"`, every function reached through its thunk, none inlined, `[HR-9a]`),
each pair checked to print the same thing and then timed interleaved. Each line gives both medians
and their ratio; the last gives the suite's overhead, the geometric mean of the ratios less one,
with the configuration it was measured on. The rule's bound is 3 %: the run exits 1 above it.

The owner's rules for a timed run hold: this process and every process it starts stay on the 8
performance cores, and a run stops at once on battery (`power_guard`).

usage: python bench/reload_overhead.py <log> [runs] [--only <name-part>] [--build-only]
  runs          how many timed runs of each program (default 7)
  --only        only the programs whose name contains this
  --build-only  build both forms of each program and stop: nothing runs
  CC=msvc|clang the C compiler (default msvc)
  EMBER=<path>  the compiler (default target/release/ember.exe, as the README's run)
Builds go to build/bench/reload/ in the repository."""
import ctypes
import math
import os
import pathlib
import platform
import shutil
import statistics
import subprocess
import sys

import run_windows as suite
from power_guard import require_mains

HERE = pathlib.Path(__file__).resolve().parent
ROOT = HERE.parent
EMBER = pathlib.Path(os.environ.get('EMBER') or suite.EMBER)
BOUND = 0.03


def build(program, cc, out, reload):
    """The program built by Ember (`development`), plain or reloadable: its executable, which for
    a reloadable build is the toolchain's host, loading the program's first image beside it."""
    form = 'reload' if reload else 'plain'
    package = out / form / program.stem
    if package.exists():
        shutil.rmtree(package)
    package.mkdir(parents=True)
    source = package / program.name
    shutil.copyfile(program, source)
    (package / 'ember.toml').write_text(
        f'[package]\nname = "{program.stem}"\n\n[build]\nreload = "{"all" if reload else "none"}"\n', encoding='utf-8')
    target = package / 'target'
    result = subprocess.run([str(EMBER), 'build', str(source), '--profile', 'development', '--cc', cc, '--out-dir', str(target)],
                            capture_output=True, text=True, timeout=600)
    if result.returncode != 0:
        sys.exit(f'ember build ({form}) failed: {program.name}\n{result.stdout}{result.stderr}')
    exe = target / 'development' / 'bin' / f'{program.stem}.exe'
    if not exe.is_file():
        sys.exit(f'{program.name}: no executable at {exe}')
    return exe


def configuration(cc):
    version = subprocess.run([str(EMBER), '--version'], capture_output=True, text=True).stdout.strip().replace('\n', '; ')
    return f'{version}; C compiler {cc}; profile development; {platform.processor() or platform.machine()}; {platform.platform()}; performance cores (mask {suite.PCORES:#x})'


def main():
    args = sys.argv[1:]
    build_only = '--build-only' in args
    args = [a for a in args if a != '--build-only']
    only = ''
    if '--only' in args:
        at = args.index('--only')
        only = args[at + 1]
        del args[at:at + 2]
    if not args and not build_only:
        sys.exit(__doc__)
    cc = os.environ.get('CC', 'msvc')
    out = ROOT / 'build' / 'bench' / 'reload' / cc
    chosen = suite.programs(only)
    if build_only:
        for _, program, _ in chosen:
            plain, reloadable = build(program, cc, out, False), build(program, cc, out, True)
            print(f'{program.stem:30} {plain.name} | {reloadable.name} (host and image)')
        print(len(chosen), 'programs built in both forms; nothing ran')
        return
    log = pathlib.Path(args[0])
    runs = int(args[1]) if len(args) > 1 else 7
    k = ctypes.windll.kernel32
    k.GetCurrentProcess.restype = ctypes.c_void_p
    k.SetProcessAffinityMask.argtypes = [ctypes.c_void_p, ctypes.c_size_t]
    assert k.SetProcessAffinityMask(k.GetCurrentProcess(), suite.PCORES)
    log.parent.mkdir(parents=True, exist_ok=True)
    ratios = []
    with log.open('w', encoding='utf-8') as out_log:
        def say(line):
            print(line, flush=True)
            out_log.write(line + '\n')
            out_log.flush()

        say(f'[HR-9] reload overhead: {configuration(cc)}; {runs} runs each')
        for _, program, _ in chosen:
            require_mains(program.stem)
            exes = {'plain': build(program, cc, out, False), 'reload': build(program, cc, out, True)}
            first = {name: suite.run(exe) for name, exe in exes.items()}
            if first['plain'][2] != first['reload'][2]:
                sys.exit(f'{program.stem}: outputs differ: plain {first["plain"][2]!r} vs reloadable {first["reload"][2]!r}')
            times = {name: [] for name in exes}
            for _ in range(runs):
                for name, exe in exes.items():
                    times[name].append(suite.run(exe)[0])
            plain_s, reload_s = statistics.median(times['plain']), statistics.median(times['reload'])
            ratio = reload_s / plain_s
            ratios.append(ratio)
            say(f'{program.stem:30} plain {plain_s:6.3f}s | reloadable {reload_s:6.3f}s | {ratio:5.3f}x')
        require_mains('the end')
        overhead = math.exp(sum(math.log(r) for r in ratios) / len(ratios)) - 1
        verdict = 'within' if overhead <= BOUND else 'over'
        say(f'suite overhead (geometric mean of {len(ratios)} programs): {overhead * 100:+.2f} %, {verdict} the 3 % bound [HR-9]')
    sys.exit(0 if overhead <= BOUND else 1)


if __name__ == '__main__':
    main()
