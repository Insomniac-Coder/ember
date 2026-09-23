"""End check 2: every cited rule id is defined in the parts (Appendix G/H excluded as sources)."""
import re, glob, os, sys, collections
HERE = os.path.dirname(os.path.abspath(__file__))
parts = sorted(glob.glob(os.path.join(HERE, '..', 'parts', 'p*.md')))
DEF = re.compile(r'^\* `\[([A-Z][A-Z0-9-]*-[0-9]+[a-z0-9]*)\]`', re.M)
CIT = re.compile(r'\[([A-Z][A-Z0-9]*(?:-[A-Z]+)?-[0-9]+[a-z0-9]*)\]')
defined = set()
for p in parts:
    defined.update(DEF.findall(open(p, encoding='utf-8').read()))
bad = collections.defaultdict(list)
for p in parts:
    if p.endswith(('p25-appx-g.md', 'p26-appx-h.md')):
        continue
    for n, line in enumerate(open(p, encoding='utf-8'), 1):
        for rid in CIT.findall(line):
            if rid not in defined:
                bad[rid].append(f'{os.path.basename(p)}:{n}')
for rid in sorted(bad):
    print(rid, ' '.join(bad[rid]))
print('undefined cited ids:', len(bad))
sys.exit(1 if bad else 0)
