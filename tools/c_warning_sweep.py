"""`[CG-C-1]`: the emitted C compiles without warnings under `-std=c11 -Wall -Wextra` (Clang, GCC)
and `/W3` (MSVC). Emits the C of every program that compiles (tests, examples, benchmarks) for one
C compiler and checks it with that compiler's warnings as errors (gcc and MSVC compile at -O2,
where some of their warnings come from; clang checks the syntax).

usage: python tools/c_warning_sweep.py clang|msvc|gcc [--jobs N] [--only <path part>] [--keep] [--all]
  Uses target/release/ember (build it first). Programs that do not compile (the reject tests) are
  skipped, and so is a test whose `#$ c-compilers:` line does not name this compiler. Prints each warning kind with its count and first places (every place with --all), then the totals; exits
  non-zero when any program warns. Work files go to build/c-warnings/<compiler>/ and are deleted
  at the end unless --keep."""
import collections
import concurrent.futures
import os
import pathlib
import re
import shutil
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
EXE = '.exe' if os.name == 'nt' else ''
EMBER = ROOT / 'target' / 'release' / f'ember{EXE}'
INCLUDE = ROOT / 'runtime' / 'ember_rt' / 'include'
SOURCES = [ROOT / 'tests', ROOT / 'examples', ROOT / 'bench' / 'programs']
VCVARS = r'C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat'
CLANG = r'C:\Program Files\LLVM\bin\clang.exe' if os.name == 'nt' else 'clang'


CL_EXE = 'cl'


def msvc_environment():
    """The environment vcvars64.bat sets up, once, and `cl`'s full path in it (Windows finds a
    program on the parent's PATH, not on the one a child is given)."""
    global CL_EXE
    out = subprocess.run(f'cmd /c ""{VCVARS}" >nul && set"', capture_output=True, text=True, shell=True).stdout
    env = dict(os.environ)
    for line in out.splitlines():
        if '=' in line:
            key, value = line.split('=', 1)
            env[key] = value
    CL_EXE = shutil.which('cl', path=env.get('PATH')) or 'cl'
    return env


def check_command(cc, c_file):
    # gcc and MSVC find some warnings only while optimising (a value maybe used before it is set),
    # so they compile for real; every clang warning comes from its front end.
    if cc == 'msvc':
        return [CL_EXE, '/nologo', '/std:c11', '/W3', '/WX', '/fp:precise', '/O2', '/c',
                f'/Fo{c_file.with_suffix(".obj")}', f'/I{INCLUDE}', str(c_file)]
    output = ['-fsyntax-only'] if cc == 'clang' else ['-c', '-o', str(c_file.with_suffix('.o'))]
    compiler = CLANG if cc == 'clang' else 'gcc'
    return [compiler, '-std=c11', '-Wall', '-Wextra', '-Werror', '-ffp-contract=off', '-fno-fast-math',
            '-O2', *output, f'-I{INCLUDE}', str(c_file)]


# One warning per line: clang/gcc `file:line:col: error: text [-Wflag]`, MSVC `file(line): error C4100: text`.
GNU = re.compile(r'^(?P<file>.+?):(?P<line>\d+):\d+: (?:error|warning): (?P<text>.*?)(?: \[(?P<kind>-W[^\]]+)\])?$')
CL = re.compile(r'^(?P<file>.+?)\((?P<line>\d+)\) ?: (?:error|warning) (?P<kind>C\d+): (?P<text>.*)$')
# MSVC's `/O2` does not finish some programs (D-514, a value 256 boxes deep): past this, no verdict.
TIME_LIMIT = 180


def wanted_compilers(program):
    """A test's `#$ c-compilers: msvc, clang` line: the C compilers its C is for (a calling
    convention only some have, `[FFI-9]`); None when it names none, meaning every one."""
    for line in program.read_text(encoding='utf-8', errors='replace').splitlines()[:20]:
        line = line.strip()
        if line.startswith('#$') and line[2:].strip().startswith('c-compilers:'):
            return {name.strip() for name in line[2:].strip()[len('c-compilers:'):].split(',') if name.strip()}
    return None


def sweep_one(cc, program, work, env):
    wanted = wanted_compilers(program)
    if wanted is not None and cc not in wanted:
        return program, None
    c_file = work / (program.relative_to(ROOT).as_posix().replace('/', '__') + '.c')
    emitted = subprocess.run([str(EMBER), 'build', str(program), '--emit', 'c', '--profile', 'release', '--cc', cc],
                             capture_output=True, text=True, encoding='utf-8', errors='replace', cwd=ROOT, env=env)
    if emitted.returncode != 0 or not emitted.stdout.strip():
        return program, None
    # `--emit c` prints a program's relaxed units (`@fastmath`, `@fp(...)`, `[CG-C-11]`) after its
    # main one: each is a translation unit of its own.
    units = re.split(r'(?m)^(?=/\* \[CG-C-11\] @)', emitted.stdout)
    found = []
    for index, unit in enumerate(units):
        unit_file = c_file.with_name(c_file.stem + (f'__unit{index}' if index else '') + '.c')
        unit_file.write_text(unit, encoding='utf-8')
        try:
            checked = subprocess.run(check_command(cc, unit_file), capture_output=True, text=True, encoding='utf-8',
                                     errors='replace', cwd=work, env=env, timeout=TIME_LIMIT)
        except subprocess.TimeoutExpired:
            found.append(('timeout (no verdict)', '?', f'the C compiler ran past {TIME_LIMIT} s'))
            continue
        here = []
        for line in (checked.stdout + checked.stderr).splitlines():
            match = (CL if cc == 'msvc' else GNU).match(line.strip())
            if match and 'D9025' not in line and 'C2220' not in line:
                here.append((match.group('kind') or 'other', match.group('line'), match.group('text').strip()))
        if checked.returncode != 0 and not here:
            here.append(('compile-error', '?', (checked.stdout + checked.stderr).strip().splitlines()[-1][:200]))
        found += here
    return program, found


def main():
    args = sys.argv[1:]
    if not args or args[0] not in ('clang', 'msvc', 'gcc'):
        sys.exit(__doc__)
    cc = args[0]
    jobs = int(args[args.index('--jobs') + 1]) if '--jobs' in args else max(1, (os.cpu_count() or 2) - 2)
    only = args[args.index('--only') + 1] if '--only' in args else ''
    if not EMBER.is_file():
        sys.exit(f'{EMBER} is not built: cargo build --release -p ember_driver')
    env = msvc_environment() if cc == 'msvc' else dict(os.environ)
    work = ROOT / 'build' / 'c-warnings' / cc
    shutil.rmtree(work, ignore_errors=True)
    work.mkdir(parents=True)
    programs = sorted(p for base in SOURCES for p in base.rglob('*.em')
                      if only in p.as_posix() and 'support' not in p.relative_to(ROOT).parts)
    by_kind = collections.defaultdict(list)
    warned, emitted = set(), 0
    with concurrent.futures.ThreadPoolExecutor(jobs) as pool:
        for program, found in pool.map(lambda p: sweep_one(cc, p, work, env), programs):
            if found is None:
                continue
            emitted += 1
            for kind, line, text in found:
                by_kind[kind].append((program.relative_to(ROOT).as_posix(), line, text))
                if not kind.startswith('timeout'):
                    warned.add(program)
    for kind, places in sorted(by_kind.items(), key=lambda item: -len({p for p, _, _ in item[1]})):
        programs_with = len({p for p, _, _ in places})
        print(f'{kind}: {len(places)} warnings in {programs_with} programs')
        for program, line, text in places if '--all' in args else places[:3]:
            print(f'    {program}:{line}: {text[:150]}')
    print(f'{cc}: {len(warned)} of {emitted} programs with C warn ({len(programs) - emitted} produced no C)')
    if '--keep' not in args:
        shutil.rmtree(work, ignore_errors=True)
    sys.exit(1 if warned else 0)


if __name__ == '__main__':
    main()
