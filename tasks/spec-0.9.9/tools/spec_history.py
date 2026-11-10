"""The spec's history as the first copy plus one diff per cut (the owner, 2026-11-10: "keep the first copy
and then consecutively create diff files for each cut and keep the final spec that is being used").

    spec_history.py build            write docs/spec-source/history/ from the full copies in docs/spec-source
    spec_history.py verify           rebuild every cut from the history and compare it with its copy (and,
                                     where a frozen parts folder exists, each part file)
    spec_history.py at <cut> <dir>   write one cut (e.g. 0.9.20_Hardened_5) to <dir>, and its parts folder
                                     beside it when the layout records one
    spec_history.py add <file> [<parts folder>]
                                     record a new cut (after cutting Hardened_N): append its diff from the
                                     last cut, and its parts layout

Tools read an old cut with `spec_history.text(name)` or `spec_history.parts(name)`.

history/000_<first>.md is the first copy, whole; history/NNN_<cut>.diff turns cut NNN-1 into cut NNN;
history/checksums.tsv holds each cut's SHA-256 as it was cut, which `verify` checks every rebuild against;
history/parts-layout.tsv gives, for each cut that had a frozen parts folder, its files and their lengths in
characters (a part may end without a newline, so lines would not add up).
"""
import difflib, glob, hashlib, os, re, sys

ROOT = os.path.normpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', '..', '..'))
SRC = os.path.join(ROOT, 'docs', 'spec-source')
HIST = os.environ.get('SPEC_HISTORY') or os.path.join(SRC, 'history')
PARTS = os.path.join(ROOT, 'tasks', 'spec-0.9.9')
NAME = re.compile(r'^Ember_v([0-9.]+)(?:_Hardened_(\d+)|_(spec|owner_revision))\.md$')
NO_EOL = '\\ No newline at end of file\n'


def cuts():
    """Every full copy, oldest first: by version, then the plain spec, the owner's revision, Hardened 1, 2, ..."""
    out = []
    for p in glob.glob(os.path.join(SRC, 'Ember_v*.md')):
        m = NAME.match(os.path.basename(p))
        if m:
            v, n, kind = m.groups()
            out.append((tuple(int(x) for x in v.split('.')), {'spec': -2, 'owner_revision': -1}.get(kind, int(n or 0)), p))
    return [p for *_, p in sorted(out)]


def cut_name(path):
    return os.path.basename(path)[len('Ember_v'):-len('.md')]


def folder_of(name):
    """The frozen parts folder of a cut, if one was kept: 0.9.9 cuts are parts-hN, later ones parts-<v>-hN."""
    m = re.match(r'^([0-9.]+)_Hardened_(\d+)$', name)
    if not m:
        return None
    v, n = m.groups()
    d = os.path.join(PARTS, f'parts-h{n}' if v == '0.9.9' else f'parts-{v}-h{n}')
    return d if os.path.isdir(d) else None


def read(path):
    return open(path, encoding='utf-8', newline='').read().splitlines(keepends=True)


def diff(a, b, name_a, name_b):
    out = []
    for line in difflib.unified_diff(a, b, name_a, name_b, n=3):
        out.append(line if line.endswith('\n') else line + '\n' + NO_EOL)
    return ''.join(out)


def apply(a, patch):
    """Apply a unified diff made by diff() to the lines a; every context and removed line must match."""
    lines = patch.splitlines(keepends=True)
    out, pos, i = [], 0, 0
    while i < len(lines) and not lines[i].startswith('@@'):
        i += 1
    while i < len(lines):
        m = re.match(r'@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@\n$', lines[i])
        if not m:
            raise ValueError(f'not a hunk header: {lines[i][:60]!r}')
        old_n = int(m.group(2) or 1)
        new_n = int(m.group(4) or 1)
        start = int(m.group(1)) - (0 if old_n == 0 else 1)
        if start < pos:
            raise ValueError('hunks out of order')
        out += a[pos:start]
        pos = start
        i += 1
        seen_old = seen_new = 0
        while seen_old < old_n or seen_new < new_n:
            tag, text = lines[i][0], lines[i][1:]
            if tag not in ' -+':
                raise ValueError(f'not a diff line: {lines[i][:60]!r}')
            if i + 1 < len(lines) and lines[i + 1] == NO_EOL:
                text = text[:-1]
                i += 1
            if tag in ' -':
                if pos >= len(a) or a[pos] != text:
                    raise ValueError(f'context does not match at line {pos + 1}')
                pos += 1
                seen_old += 1
            if tag in ' +':
                out.append(text)
                seen_new += 1
            i += 1
        if seen_old != old_n or seen_new != new_n:
            raise ValueError('hunk line counts do not match its header')
    return out + a[pos:]


def history():
    files = sorted(os.listdir(HIST))
    first = [f for f in files if f.startswith('000_')][0]
    return first, [f for f in files if f.endswith('.diff')]


def replay(upto=None):
    """Yield (cut name, lines) for every cut in the history, oldest first, stopping after `upto`."""
    first, diffs = history()
    cur = read(os.path.join(HIST, first))
    name = first[4:-3]
    yield name, cur
    if name == upto:
        return
    for d in diffs:
        cur = apply(cur, open(os.path.join(HIST, d), encoding='utf-8', newline='').read())
        name = d[4:-5]
        yield name, cur
        if name == upto:
            return


def layout():
    out = {}
    path = os.path.join(HIST, 'parts-layout.tsv')
    for row in open(path, encoding='utf-8'):
        name, part, n = row.rstrip('\n').split('\t')
        out.setdefault(name, []).append((part, int(n)))
    return out


def split(lines, parts):
    text, i = ''.join(lines), 0
    for part, n in parts:
        yield part, [text[i:i + n]]
        i += n
    assert i == len(text), 'the layout does not cover the cut'


def sha(lines):
    return hashlib.sha256(''.join(lines).encode('utf-8')).hexdigest().upper()


def sums():
    """checksums.tsv: the SHA-256 of each cut's file as it was cut, so a rebuild is checked after its copy
    is gone."""
    path = os.path.join(HIST, 'checksums.tsv')
    return dict(row.rstrip('\n').split('\t') for row in open(path, encoding='utf-8')) if os.path.exists(path) else {}


def layout_rows(name, folder):
    return [f'{name}\t{part}\t{len("".join(read(os.path.join(folder, part))))}\n'
            for part in sorted(os.listdir(folder)) if part.endswith('.md')]


def build():
    os.makedirs(HIST, exist_ok=True)
    paths = cuts()
    rows, sumrows = [], []
    prev = None
    for k, p in enumerate(paths):
        name, lines = cut_name(p), read(p)
        if prev is None:
            open(os.path.join(HIST, f'000_{name}.md'), 'w', encoding='utf-8', newline='').write(''.join(lines))
        else:
            text = diff(prev[1], lines, f'Ember_v{prev[0]}.md', f'Ember_v{name}.md')
            open(os.path.join(HIST, f'{k:03d}_{name}.diff'), 'w', encoding='utf-8', newline='').write(text)
        folder = folder_of(name)
        if folder:
            rows += layout_rows(name, folder)
        sumrows.append(f'{name}\t{sha(lines)}\n')
        prev = (name, lines)
    open(os.path.join(HIST, 'parts-layout.tsv'), 'w', encoding='utf-8', newline='\n').write(''.join(rows))
    open(os.path.join(HIST, 'checksums.tsv'), 'w', encoding='utf-8', newline='\n').write(''.join(sumrows))
    print(f'{len(paths)} cuts: 1 first copy, {len(paths) - 1} diffs, {len(rows)} part rows')


def verify():
    lay, sm, n, bad = layout(), sums(), 0, 0
    for name, lines in replay():
        copy = os.path.join(SRC, f'Ember_v{name}.md')
        if sm.get(name) != sha(lines):
            print(f'MISMATCH {name}: not the checksum recorded when it was cut'); bad += 1
        if os.path.exists(copy) and sha(read(copy)) != sha(lines):
            print(f'MISMATCH {name}'); bad += 1
        folder = folder_of(name)
        if folder and name in lay:
            for part, chunk in split(lines, lay[name]):
                if sha(read(os.path.join(folder, part))) != sha(chunk):
                    print(f'MISMATCH {name} {part}'); bad += 1
        n += 1
    print(f'{n} cuts rebuilt from the history, {bad} mismatches')
    return bad == 0


def add(path, folder=None):
    """Record a new cut: its diff from the last cut in the history, and its parts folder's layout."""
    name = cut_name(path)
    *_, (last, prev) = replay()
    if name == last:
        sys.exit(f'{name} is already the last cut in the history')
    lines = read(path)
    k = len(history()[1]) + 1
    open(os.path.join(HIST, f'{k:03d}_{name}.diff'), 'w', encoding='utf-8', newline='').write(
        diff(prev, lines, f'Ember_v{last}.md', f'Ember_v{name}.md'))
    if folder:
        with open(os.path.join(HIST, 'parts-layout.tsv'), 'a', encoding='utf-8', newline='\n') as t:
            t.write(''.join(layout_rows(name, folder)))
    with open(os.path.join(HIST, 'checksums.tsv'), 'a', encoding='utf-8', newline='\n') as t:
        t.write(f'{name}\t{sha(lines)}\n')
    print(f'added {k:03d}_{name}.diff')


_at = None  # (index in the history, name, lines): replay goes on from here when a later cut is asked for


def lines_at(name):
    """One cut's lines, rebuilt from the history. Tools ask for cuts mostly oldest first, so the replay goes
    on from the last cut asked for when it can, and starts again only for an earlier one."""
    global _at
    first, diffs = history()
    names = [first[4:-3]] + [d[4:-5] for d in diffs]
    if name not in names:
        raise KeyError(f'no cut {name} in the history')
    k = names.index(name)
    if _at is None or _at[0] > k:
        _at = (0, names[0], read(os.path.join(HIST, first)))
    i, _, cur = _at
    for j in range(i + 1, k + 1):
        cur = apply(cur, open(os.path.join(HIST, diffs[j - 1]), encoding='utf-8', newline='').read())
    _at = (k, name, cur)
    return cur


def text(name):
    """A cut's text as `open(path, encoding='utf-8').read()` gave it from its copy (newlines read as \\n)."""
    return ''.join(lines_at(name)).replace('\r\n', '\n').replace('\r', '\n')


def parts(name):
    """A cut's parts folder as [(file name, text)], read as `open(path, encoding='utf-8').read()` gives a
    file (newlines as \\n)."""
    lay = layout().get(name)
    if not lay:
        raise KeyError(f'the history records no parts folder for {name}')
    return [(part, ''.join(chunk).replace('\r\n', '\n').replace('\r', '\n'))
            for part, chunk in split(lines_at(name), lay)]


def at(name, out):
    lines = lines_at(name)
    os.makedirs(out, exist_ok=True)
    open(os.path.join(out, f'Ember_v{name}.md'), 'w', encoding='utf-8', newline='').write(''.join(lines))
    lay = layout().get(name)
    if lay:
        folder = os.path.join(out, f'parts-{name}')
        os.makedirs(folder, exist_ok=True)
        for part, chunk in split(lines, lay):
            open(os.path.join(folder, part), 'w', encoding='utf-8', newline='').write(''.join(chunk))
    print(f'wrote {name} to {out}')


if __name__ == '__main__':
    cmd = sys.argv[1] if len(sys.argv) > 1 else ''
    if cmd == 'build':
        build()
    elif cmd == 'verify':
        sys.exit(0 if verify() else 1)
    elif cmd == 'at' and len(sys.argv) == 4:
        at(sys.argv[2], sys.argv[3])
    elif cmd == 'add' and len(sys.argv) in (3, 4):
        add(sys.argv[2], sys.argv[3] if len(sys.argv) == 4 else None)
    else:
        sys.exit(__doc__)
