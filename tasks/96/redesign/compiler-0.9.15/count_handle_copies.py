"""M4 slice C, measured only (R6-batch.md task 14): the copies of a class object's handle that keep two
owners of one object without a line saying so.

Today a class handle is `Copy`, and each copy the compiler keeps is a +1: a statement
`ember_retain_plain((ember_obj_header*)x);` (or `_sync`, or the generic `ember_retain`) in the C a
program compiles to. The +1s a line asks for (`shares x`, `weakref x`, an owner read where its object is
wanted) are written inside the expression that makes the owner, `(ember_retain_plain(...), (T)x)`, and are
not counted. Slice C makes a class handle one owner: each counted copy becomes a hand-over (`=` and an
`owns` argument take the object) or, where the source is used again, a refusal that the program meets
with `shares`. This script counts them, by the function they are in (a function of `std` apart), for
every program of the quick check's folders and `std`'s own tests, and prints the totals by folder.

    python tasks/96/redesign/compiler-0.9.15/count_handle_copies.py [--json out.json] [folders...]

It builds each program's C (`ember build --backend c --emit c`), which neither compiles nor runs it.
"""
import glob
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
from collections import defaultdict
from concurrent.futures import ThreadPoolExecutor

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))))
EMBER = os.environ.get('EMBER') or os.path.join(ROOT, 'target', 'debug', 'ember.exe')
FOLDERS = ['tests/conformance/*/', 'tests/compile-pass', 'tests/run-pass', 'tests/run-fail', 'tests/milestones', 'tests/std', 'tests/ffi']
COPY = re.compile(r'^\s*ember_retain(?:_plain|_sync)?\(\(ember_obj_header\*\)')
FUNCTION = re.compile(r'^(?:static |extern )?[^#/\s][^;]*?\b(em_[A-Za-z0-9_]+)\([^;]*\)\s*\{\s*$')


def std_name(mangled):
    # `em_3std3ecs5World3addG<hash>` is `std.ecs.World.add`: each instance of a generic function is one
    # function of the source, which keeps the most copies any instance does.
    rest, parts = re.sub(r'G[0-9a-f]{32}$', '', mangled)[3:], []
    while rest[:1].isdigit():
        digits = re.match(r'\d+', rest).group(0)
        rest = rest[len(digits):]
        parts.append(rest[:int(digits)])
        rest = rest[int(digits):]
    return '.'.join(parts) + rest


def count(path, out_dir):
    # A build writes a manifest beside a program that has none: it builds a copy of the program's
    # folder (its modules and C files with it), so that nothing is left beside the repository's programs.
    folder = os.path.join(out_dir, 'src')
    shutil.copytree(os.path.dirname(path), folder)
    copy = os.path.join(folder, os.path.basename(path))
    try:
        run = subprocess.run([EMBER, 'build', '--backend', 'c', '--emit', 'c', '--out-dir', os.path.join(out_dir, 'out'), copy],
                             capture_output=True, cwd=folder, timeout=300)
    except subprocess.TimeoutExpired:
        return path, None
    finally:
        shutil.rmtree(out_dir, ignore_errors=True)
    if run.returncode != 0:
        return path, None
    user, std = 0, defaultdict(int)
    function = ''
    for line in run.stdout.decode('utf-8', 'replace').splitlines():
        found = FUNCTION.match(line)
        if found:
            function = found.group(1)
        elif COPY.match(line):
            if function.startswith('em_3std'):
                std[std_name(function)] += 1
            else:
                user += 1
    return path, (user, dict(std))


def main():
    args = sys.argv[1:]
    out_json = None
    if args[:1] == ['--json']:
        out_json, args = args[1], args[2:]
    folders = args or FOLDERS
    paths = sorted({path for pattern in folders for folder in glob.glob(os.path.join(ROOT, pattern)) for path in glob.glob(os.path.join(folder, '*.em'))})
    # A compile-fail test never builds, by design.
    paths = [path for path in paths if not re.search(r'^#\$ test: compile-fail', open(path, encoding='utf-8').read(600), re.M)]
    workers = int(os.environ.get('RUST_TEST_THREADS') or 4)
    # In the repository's `target/`, as every build here is (not the system's temporary folder).
    os.makedirs(os.path.join(ROOT, 'target'), exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='ember-handle-copies-', dir=os.path.join(ROOT, 'target')) as out_dir:
        with ThreadPoolExecutor(max_workers=workers) as pool:
            results = list(pool.map(lambda path: count(path, os.path.join(out_dir, str(abs(hash(path))))), paths))
    by_folder = defaultdict(lambda: [0, 0, 0])
    files = {}
    failed = []
    std_functions = {}
    for path, found in results:
        if found is None:
            failed.append(os.path.relpath(path, ROOT).replace(os.sep, '/'))
            continue
        user, std = found
        for function, copies in std.items():
            std_functions[function] = max(std_functions.get(function, 0), copies)
        rel = os.path.relpath(path, ROOT).replace(os.sep, '/')
        folder = rel.rsplit('/', 1)[0]
        if user:
            by_folder[folder][0] += user
            by_folder[folder][1] += 1
            files[rel] = user
        by_folder[folder][2] += 1
    total = sum(entry[0] for entry in by_folder.values())
    programs = sum(entry[1] for entry in by_folder.values())
    print(f'{total} counted handle copies in {programs} of {len(paths) - len(failed)} programs ({len(failed)} that do not build left out)')
    print(f'std: {sum(std_functions.values())} in {len(std_functions)} of the functions of std the programs use')
    for function, copies in sorted(std_functions.items(), key=lambda item: (-item[1], item[0])):
        print(f'{copies:6}           {function}')
    for folder, (copies, with_copies, checked) in sorted(by_folder.items(), key=lambda item: -item[1][0]):
        if copies:
            print(f'{copies:6} {with_copies:4}/{checked:<4} {folder}')
    if out_json:
        with open(out_json, 'w', encoding='utf-8') as out:
            json.dump({'total': total, 'programs': programs, 'failed': failed, 'std': std_functions,
                       'folders': {folder: entry for folder, entry in by_folder.items() if entry[0]}, 'files': files}, out, indent=1)


if __name__ == '__main__':
    main()
