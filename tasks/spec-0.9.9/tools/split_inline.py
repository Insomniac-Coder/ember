"""Find (and with --fix, split) rule definitions that sit mid-bullet.

A definition is `[ID]` starting a sentence inside a bullet (after '. ' or '; ', or at the start of a
continuation line after a line ending in '.'). The convention is one bullet per definition.
"""
import re, sys, glob, os

ID = r'`\[([A-Z][A-Z0-9]*(?:-[A-Z]+)?-[0-9]+[a-z0-9]*)\]`'
MID = re.compile(r'(?<=[.;]) ' + ID + r'(?=\s+[^)\s])')
CONT = re.compile(r'(?<=\.)\n  ' + ID + r'(?= )')

def process(path, fix):
    s = open(path, encoding='utf-8').read()
    hits = [(m.group(1), s.count('\n', 0, m.start()) + 1) for m in MID.finditer(s)]
    hits += [(m.group(1), s.count('\n', 0, m.start()) + 2) for m in CONT.finditer(s)]
    if fix and hits:
        s = MID.sub(lambda m: '\n* `[' + m.group(1) + ']`', s)
        s = CONT.sub(lambda m: '\n* `[' + m.group(1) + ']`', s)
        open(path, 'w', encoding='utf-8', newline='\n').write(s)
    return hits

if __name__ == '__main__':
    fix = '--fix' in sys.argv
    files = [a for a in sys.argv[1:] if a != '--fix'] or [f for f in sorted(glob.glob(os.path.join(os.path.dirname(__file__), '..', 'parts', 'p*.md'))) if not f.endswith(('p25-appx-g.md', 'p26-appx-h.md', 'p27-appx-i.md'))]
    for f in files:
        for rid, line in process(f, fix):
            print(f'{os.path.basename(f)}:{line}: {rid}')
