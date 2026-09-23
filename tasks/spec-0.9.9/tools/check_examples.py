"""End check 4: run `ember check --syntax-only` on every ```ember block of the built document.

Prints each failing block with its heading and the source line of each diagnostic, so a failure can be
classified as syntax new in 0.9.9 (which the 0.9.8-era parser lacks) or a mistake in the example.

--desugar rewrites the constructs new in 0.9.9 into 0.9.8-parser equivalents first, so the rest of each
block is checked instead of hiding behind the first new construct.

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
NEW_SYNTAX = [
    (re.compile(r' // '), ' / '),                                   # floor division
    (re.compile(r'-> some '), '-> '),                               # opaque return
    (re.compile(r'comptime\('), '('),                               # comptime(e)
    (re.compile(r'^(import c .*\)) as \w+'), r'\1'),                # import c … as name
    (re.compile(r'^\) as \w+'), ')'),
    (re.compile(r'\[[^\[\]]* for [^\[\]]*\]'), '[]'),               # list comprehension
    (re.compile(r'\{[^{}]* for [^{}]*\}'), 'Map()'),                # map/set comprehension
    (re.compile(r'= \{\}'), '= Map()'),                             # empty map literal
    (re.compile(r'\(\{\}\)'), '(Map())'),
    (re.compile(r'= \{[^{}:]*\}'), '= Set()'),                      # set literal
    (re.compile(r'(' + OPERAND + r') (<=|<|>=|>) (' + OPERAND + r') (<=|<|>=|>) (' + OPERAND + r')'),
     r'\1 \2 \3 and \3 \4 \5'),                                     # chained comparison
    (re.compile(r'^(\s+)safe fn '), r'\1fn '),                      # safe fn in an extern block
    (re.compile(r'\(([^()]*\([^()]*\))*[^()]* for [^()]*\)'), '([])'),  # generator expression
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


def desugar(body):
    out = []
    for line in body.split('\n'):
        code, sep, comment = line.partition('  #')
        for pat, rep in NEW_SYNTAX:
            code = pat.sub(rep, code)
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


def main():
    bl = blocks(open(DOC, encoding='utf-8').read())
    tmp = tempfile.mkdtemp()
    fails = 0
    for n, (heading, tag, body) in enumerate(bl):
        if tag in ('ignore', 'overlay'):
            continue
        if '--desugar' in sys.argv:
            body = desugar(body)
        path = os.path.join(tmp, f'b{n:02d}.em')
        open(path, 'w', encoding='utf-8', newline='\n').write(body)
        r = subprocess.run([EMBER, 'check', '--syntax-only', path], capture_output=True, text=True,
                           encoding='utf-8')
        if r.returncode == 0:
            continue
        fails += 1
        src = body.split('\n')
        print(f'--- block {n} ({tag}) under "{heading}"')
        msg = None
        for line in (r.stdout + r.stderr).split('\n'):
            if line.startswith('error['):
                msg = line
            m = LOC.search(line)
            if m and msg:
                k = int(m.group(1))
                print(f'  {msg}\n    L{k}: {src[k - 1] if k <= len(src) else ""}')
                msg = None
    print(f'blocks {len(bl)}, failing {fails}')


if __name__ == '__main__':
    main()
