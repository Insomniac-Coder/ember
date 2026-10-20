"""D-755 probe: time C variants of each benchmark on CI's Windows runner with Ember's MSVC release flags.

Each case is a folder under probe/cases/ holding `*.c` programs that print the same answer; `twin.c`
is the C reference. A program that includes ember_rt.h is compiled with the runtime, as Ember's
release build compiles it. Each is run 3 times untimed, then 15 times interleaved with the others of
its case, pinned to CPU 1. Results are annotations: median ms, interquartile range, ratio to twin.
"""
import ctypes, os, pathlib, platform, statistics, subprocess, sys, time

ROOT = pathlib.Path(__file__).resolve().parent
RT = ROOT.parent / 'rt'
OUT = ROOT.parent / 'out'
FLAGS = ['/nologo', '/std:c11', '/W3', '/fp:precise', '/O2', '/GL', '/MD', '/DEMBER_RELEASE']
WARMUP, TIMED = 3, 15


def note(title, text):
    print(f'::notice title={title}::{text}', flush=True)


def build(source):
    exe = OUT / f'{source.parent.name}_{source.stem}.exe'
    obj = OUT / f'{source.parent.name}_{source.stem}_obj'
    obj.mkdir(parents=True, exist_ok=True)
    text = source.read_text(encoding='utf-8')
    first = text.splitlines()[0] if text else ''
    extra = first[len('// probe-flags:'):].split() if first.startswith('// probe-flags:') else []
    cmd = ['cl', *FLAGS, *extra, f'/I{RT / "include"}', str(source)]
    if 'ember_rt.h' in source.read_text(encoding='utf-8'):
        cmd.append(str(RT / 'src' / 'ember_rt.c'))
    cmd += [f'/Fo{obj}\\', f'/Fe{exe}', '/link', '/Brepro', '/INCREMENTAL:NO']
    run = subprocess.run(cmd, capture_output=True, text=True)
    if run.returncode != 0:
        print(f'::error title=probe build {source.parent.name}/{source.name}::{run.stdout[-1500:]}', flush=True)
        return None
    return exe


def main():
    if os.name == 'nt':
        kernel = ctypes.windll.kernel32
        kernel.GetCurrentProcess.restype = ctypes.c_void_p
        kernel.SetProcessAffinityMask.argtypes = [ctypes.c_void_p, ctypes.c_size_t]
        cpu_index = int(os.environ.get('PROBE_CPU', '1'))
        if not kernel.SetProcessAffinityMask(kernel.GetCurrentProcess(), 1 << cpu_index):
            print(f'::error title=probe::cannot pin to CPU {cpu_index}', flush=True)
            sys.exit(1)
    OUT.mkdir(exist_ok=True)
    cpu = subprocess.run(['reg', 'query', r'HKLM\HARDWARE\DESCRIPTION\System\CentralProcessor\0', '/v', 'ProcessorNameString'],
                         capture_output=True, text=True).stdout
    cpu = next((line.split('REG_SZ')[1].strip() for line in cpu.splitlines() if 'REG_SZ' in line), platform.processor())
    cl = subprocess.run(['cl'], capture_output=True, text=True).stderr.splitlines()[0]
    note('probe machine', f'{cpu}; CPU {os.environ.get("PROBE_CPU", "1")}; {cl}')
    only = sys.argv[1:]
    for case in sorted(p for p in (ROOT / 'cases').iterdir() if p.is_dir()):
        if only and case.name not in only:
            continue
        programs = {}
        for source in sorted(case.glob('*.c')):
            exe = build(source)
            if exe:
                programs[source.stem] = exe
        outputs = {name: subprocess.run([str(exe)], capture_output=True, text=True).stdout for name, exe in programs.items()}
        reference = outputs.get('twin')
        for name, out in outputs.items():
            if out != reference:
                print(f'::error title=probe {case.name}::{name} printed {out.strip()!r}, the twin {str(reference).strip()!r}', flush=True)
        times = {name: [] for name in programs}
        for _ in range(WARMUP):
            for exe in programs.values():
                subprocess.run([str(exe)], capture_output=True)
        names = list(programs)
        for turn in range(TIMED):
            for offset in range(len(names)):
                name = names[(turn + offset) % len(names)]
                start = time.perf_counter()
                subprocess.run([str(programs[name])], capture_output=True)
                times[name].append((time.perf_counter() - start) * 1000)
        twin = statistics.median(times['twin']) if 'twin' in times else None
        lines = []
        for name in names:
            values = sorted(times[name])
            median = statistics.median(values)
            q1, q3 = values[len(values) // 4], values[(3 * len(values)) // 4]
            ratio = f'{median / twin:.3f}x twin' if twin else ''
            lines.append(f'{name}: {median:.1f} ms [{q1:.1f}, {q3:.1f}] {ratio}')
        note(f'probe {case.name}', ' | '.join(lines))


if __name__ == '__main__':
    main()
