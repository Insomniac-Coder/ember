"""The README's Windows benchmark run: each program in `programs/` built by Ember (release) and its
hand-written twin in `c/` built by the same C compiler, MSVC and then clang, set by set. Both must
print the same thing; then they run interleaved, and each line gives the median times and the
peak memory. This process and every process it starts stay on the 8 performance cores (logical
CPUs 0, 1, 10-13, 22, 23); a run stops at once on battery, where timings are invalid.

usage: python bench/run_windows.py <log> [runs] [--only <name-part>] [--list]
  runs     how many timed runs of each program (the README uses 11; default 7)
  --only   only the programs whose name contains this (to time a disturbed row again)
  --list   print each program and its twin, build nothing
  CCS=msvc,clang  which C compilers, in order (the default)
Builds go to build/bench/<compiler>/ in the repository."""
import ctypes
import ctypes.wintypes as wt
import os
import pathlib
import statistics
import subprocess
import sys
import time

from power_guard import require_mains

HERE = pathlib.Path(__file__).resolve().parent
ROOT = HERE.parent
EMBER = ROOT / 'target' / 'release' / 'ember.exe'
CLANG_BIN = pathlib.Path(r'C:\Program Files\LLVM\bin')
PCORES = 0xC03C03
# The sets in the order the README's logs hold them, by the programs' first letter.
SETS = ['p', 'a', 'b', 'w', 's', 't']
# The sets whose clang twins are built with Ember's floating-point flags; the `p` and `t` sets'
# twins never were (their work is integers and text).
FP_FLAG_SETS = {'a', 'b', 'w', 's'}


class Counters(ctypes.Structure):
    _fields_ = [('cb', wt.DWORD), ('PageFaultCount', wt.DWORD),
                ('PeakWorkingSetSize', ctypes.c_size_t), ('WorkingSetSize', ctypes.c_size_t),
                ('QuotaPeakPagedPoolUsage', ctypes.c_size_t), ('QuotaPagedPoolUsage', ctypes.c_size_t),
                ('QuotaPeakNonPagedPoolUsage', ctypes.c_size_t), ('QuotaNonPagedPoolUsage', ctypes.c_size_t),
                ('PagefileUsage', ctypes.c_size_t), ('PeakPagefileUsage', ctypes.c_size_t)]


def programs(only=''):
    """(set letter, program, twin) for each program, in the README's order."""
    out = []
    for letter in SETS:
        for program in sorted(HERE.glob(f'programs/{letter}*.em')):
            if only not in program.stem:
                continue
            twins = [p for p in (HERE / 'c').glob(f'{program.stem}.*') if p.suffix in ('.c', '.cpp')]
            if len(twins) != 1:
                sys.exit(f'{program.name}: expected one twin in c/, found {[t.name for t in twins]}')
            out.append((letter, program, twins[0]))
    return out


def build_ember(program, cc, out):
    target = out / 'ember' / program.stem
    result = subprocess.run([str(EMBER), 'build', str(program), '--profile', 'release', '--cc', cc,
                             '--out-dir', str(target)], capture_output=True, text=True, timeout=300)
    if result.returncode != 0:
        sys.exit(f'ember build failed: {program.name}\n{result.stdout}{result.stderr}')
    return target / 'release' / 'bin' / f'{program.stem}.exe'


def build_twin(letter, source, cc, out):
    exe = out / 'ref' / f'{source.stem}.exe'
    exe.parent.mkdir(parents=True, exist_ok=True)
    fp = ['-ffp-contract=off', '-fno-fast-math'] if letter in FP_FLAG_SETS else []
    if cc == 'msvc':
        command = ['cmd', '/c', str(HERE / 'cl_build.bat'), str(source), str(exe)]
    elif source.suffix == '.cpp':
        command = [str(CLANG_BIN / 'clang++.exe'), '-O2', *fp, '-std=c++17', str(source), '-o', str(exe)]
    else:
        command = [str(CLANG_BIN / 'clang.exe'), '-O2', *fp, '-std=c11', str(source), '-o', str(exe)]
    result = subprocess.run(command, capture_output=True, text=True, timeout=300)
    if result.returncode != 0 or not exe.is_file():
        sys.exit(f'twin build failed: {source.name}\n{result.stdout}{result.stderr}')
    return exe


def run(exe):
    start = time.perf_counter()
    process = subprocess.Popen([str(exe)], stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    stdout, stderr = process.communicate(timeout=120)
    seconds = time.perf_counter() - start
    counters = Counters()
    counters.cb = ctypes.sizeof(counters)
    ctypes.windll.psapi.GetProcessMemoryInfo(int(process._handle), ctypes.byref(counters), counters.cb)
    if process.returncode != 0:
        sys.exit(f'{exe} exited {process.returncode}: {stderr.decode(errors="replace")}')
    return seconds, counters.PeakPagefileUsage, stdout.decode().strip()


def main():
    args = sys.argv[1:]
    listing = '--list' in args
    only = ''
    if '--only' in args:
        at = args.index('--only')
        only = args[at + 1]
        del args[at:at + 2]
    args = [a for a in args if a != '--list']
    chosen = programs(only)
    if listing:
        for letter, program, twin in chosen:
            print(f'{letter}  {program.name:34} {twin.name}')
        print(len(chosen), 'programs')
        return
    if not args:
        sys.exit(__doc__)
    log = pathlib.Path(args[0])
    runs = int(args[1]) if len(args) > 1 else 7
    k = ctypes.windll.kernel32
    k.GetCurrentProcess.restype = ctypes.c_void_p
    k.SetProcessAffinityMask.argtypes = [ctypes.c_void_p, ctypes.c_size_t]
    assert k.SetProcessAffinityMask(k.GetCurrentProcess(), PCORES)
    log.parent.mkdir(parents=True, exist_ok=True)
    with log.open('w', encoding='utf-8') as out_log:
        for letter in SETS:
            for cc in os.environ.get('CCS', 'msvc,clang').split(','):
                require_mains(f'set {letter} {cc}')
                out = ROOT / 'build' / 'bench' / cc
                for _, program, twin in (c for c in chosen if c[0] == letter):
                    exes = {'ember': build_ember(program, cc, out), 'ref': build_twin(letter, twin, cc, out)}
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
                    line = (f'{program.stem:30} ember {ember_s:6.3f}s | {twin.suffix[1:]:3} {ref_s:6.3f}s | '
                            f'ember/ref {ember_s / ref_s:5.2f}x | MB {ember_mb:6.1f} / {ref_mb:6.1f}')
                    print(line, flush=True)
                    out_log.write(line + '\n')
                    out_log.flush()
    require_mains('the end')
    print('done', log)


if __name__ == '__main__':
    main()
