"""Small helpers for exact edits of the spec parts (used while applying change lists)."""
import os

PARTS = os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', 'parts')


def _path(p):
    return p if os.path.isabs(p) else os.path.join(PARTS, p)


def read(p):
    return open(_path(p), encoding='utf-8').read()


def write(p, s):
    # LF, as .gitattributes pins: text mode on Windows would write CRLF.
    open(_path(p), 'w', encoding='utf-8', newline='\n').write(s)


def edit(p, old, new):
    """Replace exactly one occurrence of `old`; fail loudly if it is absent."""
    s = read(p)
    if old not in s:
        raise SystemExit(f'NOT FOUND in {p}: {old[:80]!r}')
    write(p, s.replace(old, new, 1))


def span(p, rid):
    """(start, end) of the bullet defining `rid`, continuation lines included."""
    s = read(p)
    i = s.index('* `[' + rid + ']`')
    lines = s[i:].split('\n')
    k = 1
    while k < len(lines) and lines[k].startswith('  '):
        k += 1
    return i, i + len('\n'.join(lines[:k]))


def bullet(p, rid):
    i, j = span(p, rid)
    return read(p)[i:j]


def replace_bullet(p, rid, text):
    i, j = span(p, rid)
    s = read(p)
    write(p, s[:i] + text.rstrip('\n') + s[j:])


def after(p, rid, text):
    i, j = span(p, rid)
    s = read(p)
    write(p, s[:j] + '\n' + text.rstrip('\n') + s[j:])
