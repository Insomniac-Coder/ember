"""The README's gcc benchmark run, in WSL: each program in `../programs/` built by Ember (release,
gcc) and its twin in `../c/` built by gcc/g++ with the flags of Ember's release build
(-O2 -fvect-cost-model=cheap -funroll-loops). Both must print the same thing; then they run
interleaved on one CPU, and each line gives the median times and the peak memory, in the format
of the Windows log, so update_readme.py reads it.

usage (in WSL): python3 bench/gcc/run_gcc.py [runs] [name-part]
  EMBER_TREE  a Linux copy of the compiler built by make_tree.sh (default ~/ember-bench-tree)
Builds go to ~/ember-bench-out, on Linux's own disk."""
import os
import pathlib
import statistics
import subprocess
import sys
import time

BENCH = pathlib.Path(__file__).resolve().parent.parent
TREE = pathlib.Path(os.environ.get('EMBER_TREE', str(pathlib.Path.home() / 'ember-bench-tree')))
EMBER = TREE / 'target' / 'release' / 'ember'
os.environ.setdefault('EMBER_STD', str(TREE / 'std' / 'src'))
os.environ.setdefault('EMBER_RUNTIME_DIR', str(TREE / 'runtime' / 'ember_rt'))
FLAGS = ['-O2', '-fvect-cost-model=cheap', '-funroll-loops']


def build_ember(program, out):
    target = out / 'ember' / program.stem
    result = subprocess.run([str(EMBER), 'build', str(program), '--profile', 'release', '--cc', 'gcc',
                             '--out-dir', str(target)], capture_output=True, text=True, timeout=600)
    if result.returncode != 0:
        sys.exit(f'ember build failed: {program.name}\n{result.stdout}{result.stderr}')
    return target / 'release' / 'bin' / program.stem


def build_twin(source, out):
    exe = out / 'ref' / source.stem
    exe.parent.mkdir(parents=True, exist_ok=True)
    if source.suffix == '.cpp':
        command = ['g++', *FLAGS, '-std=c++17', str(source), '-o', str(exe)]
    else:
        command = ['gcc', *FLAGS, '-std=c11', str(source), '-o', str(exe), '-lm']
    result = subprocess.run(command, capture_output=True, text=True, timeout=600)
    if result.returncode != 0 or not exe.is_file():
        sys.exit(f'twin build failed: {source.name}\n{result.stdout}{result.stderr}')
    return exe


def run(exe):
    start = time.perf_counter()
    process = subprocess.Popen([str(exe)], stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    stdout = process.stdout.read()
    stderr = process.stderr.read()
    _, status, usage = os.wait4(process.pid, 0)
    seconds = time.perf_counter() - start
    code = os.waitstatus_to_exitcode(status)
    if code != 0:
        sys.exit(f'{exe} exited {code}: {stderr.decode(errors="replace")}')
    return seconds, usage.ru_maxrss * 1024, stdout.decode().strip()


def main():
    runs = int(sys.argv[1]) if len(sys.argv) > 1 else 11
    only = sys.argv[2] if len(sys.argv) > 2 else ''
    os.sched_setaffinity(0, {0})
    out = pathlib.Path.home() / 'ember-bench-out'
    out.mkdir(parents=True, exist_ok=True)
    for program in sorted(p for p in (BENCH / 'programs').glob('*.em') if only in p.stem):
        twins = [p for p in (BENCH / 'c').glob(f'{program.stem}.*') if p.suffix in ('.c', '.cpp')]
        if len(twins) != 1:
            sys.exit(f'{program.name}: expected one twin in c/, found {[t.name for t in twins]}')
        twin = twins[0]
        exes = {'ember': build_ember(program, out), 'ref': build_twin(twin, out)}
        first = {name: run(exe) for name, exe in exes.items()}
        if first['ember'][2] != first['ref'][2]:
            sys.exit(f'{program.stem}: outputs differ: ember {first["ember"][2]!r} vs twin {first["ref"][2]!r}')
        times = {name: [] for name in exes}
        peaks = {name: [] for name in exes}
        for _ in range(runs):
            for name, exe in exes.items():
                seconds, peak, _ = run(exe)
                times[name].append(seconds)
                peaks[name].append(peak)
        ember_s, ref_s = statistics.median(times['ember']), statistics.median(times['ref'])
        ember_mb, ref_mb = max(peaks['ember']) / 1e6, max(peaks['ref']) / 1e6
        print(f'{program.stem:30} ember {ember_s:6.3f}s | {twin.suffix[1:]:3} {ref_s:6.3f}s | '
              f'ember/ref {ember_s / ref_s:5.2f}x | MB {ember_mb:6.1f} / {ref_mb:6.1f}', flush=True)


if __name__ == '__main__':
    main()
