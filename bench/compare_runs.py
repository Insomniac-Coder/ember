"""Ember's time in one benchmark run against another, per program and C compiler.

usage: python bench/compare_runs.py <old windows logs> <new windows logs> [<old gcc log> <new gcc log>]
(several Windows logs separated by commas: a later one overrides an earlier one's programs)

A Windows log has each harness's MSVC block, then its clang block: a program's first line is
MSVC's, its second clang's. A gcc log has one line per program. Prints every pair, the ratio of
the new Ember time to the old (and the new Ember/C ratio), and flags each new/old above 1.05."""
import re
import sys

LINE = re.compile(r'^(\S+)\s+ember\s+([0-9.]+)s\s+\|\s+(\S+)\s+([0-9.]+)s\s+\|\s+ember/ref\s+([0-9.]+)x')


def read(path, compilers):
    seen = {}
    out = {}
    for line in open(path, encoding='utf-8', errors='replace'):
        m = LINE.match(line.strip())
        if not m:
            continue
        name = m.group(1)
        k = seen.get(name, 0)
        seen[name] = k + 1
        if k >= len(compilers):
            continue
        out[(compilers[k], name)] = (float(m.group(2)), float(m.group(4)), float(m.group(5)))
    return out


def report(old, new, limit=1.05):
    slower = []
    for key in sorted(new, key=lambda k: (k[0], k[1])):
        if key not in old:
            print(f'{key[0]:6} {key[1]:32} new only: ember {new[key][0]:.3f}s')
            continue
        (o, _, _), (n, c, r) = old[key], new[key]
        ratio = n / o if o else float('inf')
        flag = '  SLOWER' if ratio > limit else ''
        print(f'{key[0]:6} {key[1]:32} old {o:.3f}s new {n:.3f}s  new/old {ratio:.2f}  ember/C {r:.2f}{flag}')
        if ratio > limit:
            slower.append((key, o, n, ratio))
    return slower


def read_all(paths, compilers):
    out = {}
    for path in paths.split(','):
        out.update(read(path, compilers))
    return out


if __name__ == '__main__':
    old = read_all(sys.argv[1], ['msvc', 'clang'])
    new = read_all(sys.argv[2], ['msvc', 'clang'])
    slower = report(old, new)
    if len(sys.argv) > 4:
        slower += report(read(sys.argv[3], ['gcc']), read(sys.argv[4], ['gcc']))
    print('slower than before (new/old > 1.05):', len(slower))
    for (cc, name), o, n, ratio in slower:
        print(f'  {cc} {name}: {o:.3f}s -> {n:.3f}s ({ratio:.2f}x)')
