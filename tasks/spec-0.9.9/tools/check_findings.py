"""Coverage check: every finding is mapped, and every cited rule id is defined in the parts.

Also writes parts/p25-appx-g.md from findings_map.M (--write).
"""
import glob, os, re, sys
sys.path.insert(0, os.path.dirname(__file__))
from findings_map import M

HERE = os.path.dirname(os.path.abspath(__file__))
PARTS = sorted(glob.glob(os.path.join(HERE, '..', 'parts', 'p*.md')))
TITLES = os.path.join(HERE, '..', 'titles.tsv')
DEF = re.compile(r'^\* `\[([A-Z][A-Z0-9-]*-[0-9]+[a-z0-9]*)\]`', re.M)

def defined_ids():
    ids = set()
    for p in PARTS:
        if p.endswith('p25-appx-g.md'):
            continue
        ids.update(DEF.findall(open(p, encoding='utf-8').read()))
    return ids

FIX = {
    'F-001': '`tasks/audit/TASKS.md` is cited by every reproducer and exists on no branch',
    'F-066': 'integer `**` with a run-time negative exponent',
    'F-077': 'spec examples call APIs that do not exist',
    'F-081': 'writing a field through a borrowed class-handle parameter is rejected (update to F-073)',
    'F-099': 'the attribute table omits `@nopanic(explicit)` (refines F-021)',
    'F-111': 'more syntax in examples that the grammar lacks',
    'F-153': 'the N1 suggestion threshold admits `io` -> `Eq` (the rule behind F-006)',
    'F-182': '`Option`/`Array`/`str` basics are missing',
}

def titles():
    out = {}
    for line in open(TITLES, encoding='utf-8'):
        f = line.rstrip('\n').split('\t')
        if len(f) >= 4:
            out[f[0]] = (f[1], f[2], FIX.get(f[0], f[3]))
    return out

def main():
    t = titles()
    ids = defined_ids()
    want = [f'F-{n:03d}' for n in range(1, 215)]
    bad = 0
    for fid in want:
        if fid not in M:
            print('UNMAPPED', fid); bad += 1; continue
        for rid in M[fid][2]:
            if rid not in ids:
                print('UNDEFINED', fid, rid); bad += 1
    extra = set(M) - set(want)
    for fid in sorted(extra):
        print('EXTRA', fid); bad += 1
    kinds = {}
    for k, _, _ in M.values():
        kinds[k] = kinds.get(k, 0) + 1
    print('mapped', len(M), 'kinds', kinds, 'problems', bad)
    if '--write' in sys.argv:
        write(t)
    return bad

def write(t):
    lines = ['---', '', '# Appendix G — Resolution of Findings F-001–F-214', '',
             'Every finding of the 2026-09-23 research pass (`tasks/audit/FINDINGS.md`) and how this revision',
             'resolves it. **SPEC**: the language text changed or gained a rule. **IMPL**: the rule stands or was',
             'clarified and the implementation must meet it. **GATE**: a test or CI obligation now in the text.',
             '**OUT**: outside a language specification, with the reason. **WDN**: withdrawn.', '',
             '| Finding | Kind | Resolution | Rules |', '|---|---|---|---|']
    for fid in sorted(M):
        kind, res, rules = M[fid]
        title = t.get(fid, ('', '', ''))[2].rstrip('.').replace('|', '\\|')
        r = ', '.join(f'`[{x}]`' for x in rules) or '—'
        lines.append(f'| {fid} — {title} | {kind} | {res.replace("|", chr(92) + "|")} | {r} |')
    open(os.path.join(HERE, '..', 'parts', 'p25-appx-g.md'), 'w', encoding='utf-8').write('\n'.join(lines) + '\n')
    print('wrote p25-appx-g.md')

if __name__ == '__main__':
    sys.exit(1 if main() else 0)
