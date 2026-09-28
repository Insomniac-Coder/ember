"""End check 4: run `ember check --syntax-only` on every ```ember block of the built document.

A block that does not compile is sorted into one of two kinds, and called by that kind:
- **unbuilt features**: it compiles once the constructs the compiler does not have yet (`NEW_SYNTAX`)
  are rewritten into ones it has, so it only shows features still to be built; they are named;
- **errors**: it still does not compile, so the example is wrong or something built is broken.
Each such block is printed with its heading and the source line of each diagnostic.

--desugar checks only the rewritten form of every block.

Usage: check_examples.py <ember.exe> [document] [--desugar]
"""
import re, os, sys, subprocess, tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ARGS = [a for a in sys.argv[1:] if not a.startswith('--')]
EMBER = ARGS[0]
DOC = ARGS[1] if len(ARGS) > 1 else os.path.join(
    HERE, '..', '..', '..', 'docs', 'spec-source', 'Ember_v0.9.9_Hardened_2.md')
LOC = re.compile(r'--> .*:(\d+):(\d+)\s*$')

OPERAND = r'[\w.\[\]()]+'
# Each construct the compiler does not build yet: its rewrite into one it has, and its name.
NEW_SYNTAX = [
    (re.compile(r' // '), ' / ', 'floor division'),
    (re.compile(r'-> some '), '-> ', 'opaque return types (`-> some I`)'),
    (re.compile(r'comptime\('), '(', 'compile-time calls (`comptime(e)`)'),
    (re.compile(r'^(import c .*\)) as \w+'), r'\1', 'C header import (`import c`)'),
    (re.compile(r'^\) as \w+'), ')', 'C header import (`import c`)'),
    (re.compile(r'\[[^\[\]]* for [^\[\]]*\]'), '[]', 'list comprehensions'),
    (re.compile(r'\{[^{}]* for [^{}]*\}'), 'Map()', 'map and set comprehensions'),
    (re.compile(r'= \{\}'), '= Map()', 'empty map literals'),
    (re.compile(r'\(\{\}\)'), '(Map())', 'empty map literals'),
    (re.compile(r'= \{[^{}:]*\}'), '= Set()', 'set literals'),
    (re.compile(r'(' + OPERAND + r') (<=|<|>=|>) (' + OPERAND + r') (<=|<|>=|>) (' + OPERAND + r')'),
     r'\1 \2 \3 and \3 \4 \5', 'chained comparisons'),
    (re.compile(r'^(\s+)safe fn '), r'\1fn ', '`safe fn` in an extern block'),
    (re.compile(r'\(([^()]*\([^()]*\))*[^()]* for [^()]*\)'), '([])', 'generator expressions'),
]

ITEM = re.compile(r'(fn|gen|once|class|open|abstract|struct|enum|interface|import|from|pub|@|#|static|'
                  r'const|type|extend|unsafe|overlay|comptime|extern)\b|[@#)\]]')


def lift_script(body):
    """Top-level statements of an entry file (0.9.9 H2) become the body of `fn main():`."""
    items, stmts, cur = [], [], None
    for line in body.split('\n'):
        if line and not line[0].isspace():
            cur = items if ITEM.match(line) else stmts
        (cur if cur is not None else items).append(line)
    if not any(s.strip() for s in stmts):
        return body
    return '\n'.join(items + ['fn main():'] + ['    ' + s if s.strip() else s for s in stmts]) + '\n'


def desugar(body, used=None, keep=None):
    """The block with every unbuilt construct rewritten, except those named `keep`; the names of
    those it had go in `used`."""
    out = []
    for line in body.split('\n'):
        code, sep, comment = line.partition('  #')
        for pat, rep, name in NEW_SYNTAX:
            if name == keep:
                continue
            code, n = pat.subn(rep, code)
            if n and used is not None and name not in used:
                used.append(name)
        out.append(code + sep + comment)
    return lift_script('\n'.join(out))


def blocks(text):
    out, heading, lines, i = [], '', text.split('\n'), 0
    while i < len(lines):
        line = lines[i]
        if line.startswith('#'):
            heading = line.strip('# ').strip()
        m = re.match(r'^(\s*)```ember(,\w+)?\s*$', line)
        if m:
            indent, tag, body = len(m.group(1)), (m.group(2) or ',full')[1:], []
            i += 1
            while not lines[i].strip().startswith('```'):
                body.append(lines[i][indent:])
                i += 1
            out.append((heading, tag, '\n'.join(body) + '\n'))
        i += 1
    return out


def check(body, path):
    open(path, 'w', encoding='utf-8', newline='\n').write(body)
    return subprocess.run([EMBER, 'check', '--syntax-only', path], capture_output=True, text=True,
                          encoding='utf-8')


def main():
    bl = blocks(open(DOC, encoding='utf-8').read())
    tmp = tempfile.mkdtemp()
    unbuilt, errors, features = 0, 0, []
    for n, (heading, tag, body) in enumerate(bl):
        if tag in ('ignore', 'overlay'):
            continue
        if '--desugar' in sys.argv:
            body = desugar(body)
        path = os.path.join(tmp, f'b{n:02d}.em')
        r = check(body, path)
        if r.returncode == 0:
            continue
        used = []
        if check(desugar(body, used), path).returncode == 0 and used:
            # Only the constructs the compiler really lacks: a rewrite the block also compiles
            # without is of something already built.
            used = [name for name in used if check(desugar(body, keep=name), path).returncode != 0]
            unbuilt += 1
            features += [name for name in used if name not in features]
            kind = 'unbuilt features: ' + ', '.join(used)
        else:
            errors += 1
            kind = 'ERROR'
        src = body.split('\n')
        print(f'--- block {n} ({tag}) under "{heading}": {kind}')
        msg = None
        for line in (r.stdout + r.stderr).split('\n'):
            if line.startswith('error['):
                msg = line
            m = LOC.search(line)
            if m and msg:
                k = int(m.group(1))
                print(f'  {msg}\n    L{k}: {src[k - 1] if k <= len(src) else ""}')
                msg = None
    print(f'blocks {len(bl)}: {len(bl) - unbuilt - errors} compile, {unbuilt} show unbuilt features'
          f' ({", ".join(features) or "none"}), {errors} errors')


if __name__ == '__main__':
    main()
