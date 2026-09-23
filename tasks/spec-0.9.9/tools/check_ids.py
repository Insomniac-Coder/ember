"""End check 0/1: rule-id hygiene against 0.9.8_Hardened_3.

- every id defined in the parts is defined once;
- an id not present in 0.9.8 must be marked *(new in 0.9.9)*;
- an id marked *(new in 0.9.9)* must not be present in 0.9.8.
"""
import re, glob, os, sys, collections
HERE = os.path.dirname(os.path.abspath(__file__))
OLD = os.path.join(HERE, '..', '..', '..', 'docs', 'spec-source', 'Ember_v0.9.8_Hardened_3.md')
old = open(OLD, encoding='utf-8').read()
oldids = set(re.findall(r'\[([A-Z][A-Z0-9]*(?:-[A-Z]+)?-[0-9]+[a-z0-9]*)\]', old))
DEF = re.compile(r'^\* `\[([A-Z][A-Z0-9-]*-[0-9]+[a-z0-9]*)\]`( \*\(new in 0\.9\.9\)\*)?', re.M)
seen = collections.defaultdict(list)
bad = 0
for p in sorted(glob.glob(os.path.join(HERE, '..', 'parts', 'p*.md'))):
    if p.endswith('p25-appx-g.md'):
        continue
    for m in DEF.finditer(open(p, encoding='utf-8').read()):
        rid, marked = m.group(1), bool(m.group(2))
        seen[rid].append(os.path.basename(p))
        if rid in oldids and marked:
            print('MARKED-NEW-BUT-OLD', rid, os.path.basename(p)); bad += 1
        if rid not in oldids and not marked:
            print('NEW-UNMARKED', rid, os.path.basename(p)); bad += 1
for rid, where in seen.items():
    if len(where) > 1:
        print('DUPLICATE', rid, where); bad += 1
print('defined', len(seen), 'problems', bad)
sys.exit(1 if bad else 0)
