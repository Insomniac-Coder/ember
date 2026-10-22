"""`[BUD-2]`'s B2 over `bench/bigpkg` (`[BUD-1]`), the build half of `[HR-1]`: `ember build --reload`
after a one-function-body edit (`mods/m200.em`'s `edit_me`), the median of the timed runs after
the warm-up ones, with each run's stage split (`--timings`). The package is generated, then built
once whole as the running program's first image; each run edits the body back and forth.

Nothing compiled runs here. `[HR-1]`'s whole figure adds the image's load, plan, prepare and commit
over 10,000 live instances, which `ember_driver/tests/reload.rs`' `[BEN-8]` test reports from a
host; on the reference machine both are taken on the performance cores and on mains power.

usage: python bench/bigpkg/measure_reload_build.py [runs] [warm-ups]   (default 15 and 3, `[BUD-2]`)
  EMBER=<path>  the compiler (default target/release/ember.exe)"""
import os
import pathlib
import re
import statistics
import subprocess
import sys
import time

HERE = pathlib.Path(__file__).resolve().parent
ROOT = HERE.parent.parent
EMBER = pathlib.Path(os.environ.get('EMBER') or ROOT / 'target' / 'release' / 'ember.exe')
BUDGET = 1.0


def main():
    runs = int(sys.argv[1]) if len(sys.argv) > 1 else 15
    warm = int(sys.argv[2]) if len(sys.argv) > 2 else 3
    package = ROOT / 'build' / 'bigpkg'
    subprocess.run([sys.executable, str(HERE / 'generate.py'), str(package)], check=True)
    out = package / 'out'
    first = subprocess.run([str(EMBER), 'build', 'main.em', '--out-dir', str(out)], cwd=package, capture_output=True, text=True)
    if first.returncode != 0:
        sys.exit(f'the first build failed:\n{first.stdout}{first.stderr}')
    body = package / 'mods' / 'm200.em'
    text = body.read_text(encoding='utf-8')
    times = []
    for turn in range(warm + runs):
        edited = text.replace('    return x + 1\n', f'    return x + {turn + 2}\n')
        body.write_text(edited, encoding='utf-8')
        start = time.perf_counter()
        result = subprocess.run([str(EMBER), 'build', '--reload', 'main.em', '--out-dir', str(out), '--timings'],
                                cwd=package, capture_output=True, text=True)
        seconds = time.perf_counter() - start
        if result.returncode != 0:
            sys.exit(f'the reload build failed:\n{result.stdout}{result.stderr}')
        if turn >= warm:
            times.append(seconds)
            stages = [line.strip() for line in (result.stdout + result.stderr).splitlines()
                      if re.match(r'\s*(check|lower|move|analyses|C generation|C compile|total)\b', line)]
            print(f'run {turn - warm + 1}: {seconds:.3f} s  ' + ' | '.join(stages), flush=True)
    median = statistics.median(times)
    spread = (max(times) - min(times)) / median if median else 0.0
    verdict = 'within' if median <= BUDGET else 'over'
    print(f'B2 over bench/bigpkg: median {median:.3f} s of {len(times)} runs (spread {spread * 100:.0f} %), {verdict} the 1 s budget [BUD-2], [HR-1]')
    sys.exit(0 if median <= BUDGET else 1)


if __name__ == '__main__':
    main()
