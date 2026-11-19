"""`ember bench --compare`, stop tolerant: the pairs of tests/perf one at a time, each one's result kept under
target/bench-resume/<key>/, where the key is the compiler's bytes, the backend, the core and tests/perf's files.
Run it again after a stop and it measures only the pairs with no result yet; a pair the machine was unfit for
(`void`) or that was `inconclusive` is measured again. A new compiler build, another backend or core, or a changed
benchmark starts over; --fresh starts over anyway.

    python tools/bench_resume.py [--core 1] [--backend c|llvm] [--ember target/release/ember.exe] [--fresh]

Exit 0 when every pair passed.
"""
import hashlib
import os
import pathlib
import re
import shutil
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
PERF = ROOT / 'tests' / 'perf'
SOURCE_EXT = '.em'


def arg(name, default):
    return sys.argv[sys.argv.index(name) + 1] if name in sys.argv else default


def digest(paths):
    h = hashlib.sha256()
    for path in paths:
        h.update(path.name.encode())
        h.update(path.read_bytes())
    return h.hexdigest()


def main():
    core = arg('--core', '1')
    backend = arg('--backend', os.environ.get('EMBER_BACKEND', 'llvm'))
    ember = pathlib.Path(arg('--ember', str(ROOT / 'target' / 'release' / ('ember.exe' if os.name == 'nt' else 'ember'))))
    # The folder's own files, which the bench reads; not what it builds below them (target/).
    files = sorted(p for p in PERF.iterdir() if p.is_file())
    key = hashlib.sha256(f'{digest([ember])}|{backend}|{core}|{digest(files)}'.encode()).hexdigest()[:16]
    base = ROOT / 'target' / 'bench-resume'
    state = base / key
    if '--fresh' in sys.argv and state.exists():
        shutil.rmtree(state)
    state.mkdir(parents=True, exist_ok=True)
    for other in base.iterdir():
        if other.is_dir() and other.name not in (key, 'work'):
            shutil.rmtree(other, ignore_errors=True)

    # A pair: `name.em` and its twin, `name.c` or `name.cpp`; everything else in the folder is shared.
    pairs = sorted(p.stem for p in PERF.glob('*' + SOURCE_EXT)
                   if p.with_suffix('.c').exists() or p.with_suffix('.cpp').exists())
    paired = {f'{n}{ext}' for n in pairs for ext in (SOURCE_EXT, '.c', '.cpp')}
    shared = [p for p in PERF.iterdir() if p.is_file() and p.name not in paired]
    # `--only a,b`: those pairs alone (to try the script, or to measure one again with --fresh).
    if '--only' in sys.argv:
        only = set(arg('--only', '').split(','))
        pairs = [name for name in pairs if name in only]

    env = dict(os.environ, EMBER_BACKEND=backend, PYTHONIOENCODING='utf-8')
    results = {}
    for name in pairs:
        kept = state / f'{name}.txt'
        if kept.exists():
            results[name] = kept.read_text(encoding='utf-8')
            print(f'kept  {name}: {verdict(results[name])}')
            continue
        work = base / 'work' / name
        shutil.rmtree(work, ignore_errors=True)
        work.mkdir(parents=True)
        for path in shared + [p for p in (PERF / f'{name}{ext}' for ext in (SOURCE_EXT, '.c', '.cpp')) if p.exists()]:
            shutil.copy2(path, work / path.name)
        # Into a file, not a pipe: a process the linker leaves running (MSVC's `mspdbsrv`) keeps an
        # inherited pipe open, and reading it to its end would wait for ever.
        log = base / 'work' / f'{name}.log'
        with open(log, 'w', encoding='utf-8') as out:
            subprocess.run([str(ember), 'bench', '--compare', '--core', core, '--dir', str(work)],
                           stdout=out, stderr=subprocess.STDOUT, stdin=subprocess.DEVNULL, env=env, cwd=ROOT)
        text = log.read_text(encoding='utf-8', errors='replace')
        log.unlink()
        shutil.rmtree(work, ignore_errors=True)
        results[name] = text
        print(f'ran   {name}: {verdict(text)}')
        # Kept only when it says something about the code: a void or inconclusive run is measured again.
        if verdict(text) in ('pass', 'fail'):
            tmp = kept.with_suffix('.tmp')
            tmp.write_text(text, encoding='utf-8')
            tmp.replace(kept)

    counts = {}
    for name, text in results.items():
        counts.setdefault(verdict(text), []).append(name)
    print(f'\n{backend} on core {core}: ' + ', '.join(f'{len(v)} {k}' for k, v in sorted(counts.items())))
    for name in counts.get('fail', []) + counts.get('void', []) + counts.get('inconclusive', []) + counts.get('error', []):
        detail = [l for l in results[name].splitlines() if l.startswith(name) or 'ratio' in l][:2]
        print(f'  {name}: ' + ' | '.join(l.strip() for l in detail))
    return 0 if set(counts) == {'pass'} else 1


def verdict(text):
    m = re.search(r'(\d+) passed, (\d+) failed, (\d+) inconclusive, (\d+) void', text)
    if not m:
        return 'error'
    passed, failed, inconclusive, void = map(int, m.groups())
    if failed:
        return 'fail'
    if void:
        return 'void'
    if inconclusive:
        return 'inconclusive'
    return 'pass' if passed else 'error'


if __name__ == '__main__':
    sys.exit(main())
