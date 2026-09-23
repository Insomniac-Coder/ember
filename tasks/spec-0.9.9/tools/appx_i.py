"""Writes parts/p27-appx-i.md: every rule id, where it is defined, and the start of its text."""
import re, glob, os, collections

HERE = os.path.dirname(os.path.abspath(__file__))
PART = {'p00': '0', 'p01': 'I', 'p02': 'II', 'p03': 'III', 'p04': 'IV', 'p05': 'V', 'p06': 'VI',
        'p07': 'VII', 'p08': 'VIII', 'p09': 'IX', 'p10': 'X', 'p11': 'XI', 'p12': 'XII', 'p13': 'XIII',
        'p14': 'XIV', 'p15': 'XV', 'p16': 'XVI', 'p17': 'XVII', 'p18': 'XVIII', 'p19': 'Annex A',
        'p20': 'Annex B', 'p21': 'Annex C', 'p22': 'Annex D', 'p23': 'Appx E', 'p24': 'Appx F'}
DEF = re.compile(r'^\* `\[([A-Z][A-Z0-9-]*-[0-9]+[a-z0-9]*)\]`((?:[^\n]|\n  )*)', re.M)
MARK = re.compile(r'\*\((new|changed) in 0\.9\.9\)\*\s*')
PIPE = '\\|'


def main():
    rows = collections.defaultdict(list)
    for p in sorted(glob.glob(os.path.join(HERE, '..', 'parts', 'p*.md'))):
        key = os.path.basename(p)[:3]
        if key not in PART:
            continue
        for m in DEF.finditer(open(p, encoding='utf-8').read()):
            text = re.sub(r'\s+', ' ', m.group(2)).strip()
            text = MARK.sub('', text).replace('**', '').replace('|', PIPE)
            if len(text) > 70:
                text = text[:70].rsplit(' ', 1)[0] + ' …'
            fam = re.sub(r'-[0-9]+[a-z0-9]*$', '', m.group(1))
            rows[fam].append((m.group(1), PART[key], text))
    num = lambda r: (int(re.search(r'-([0-9]+)[a-z0-9]*$', r[0]).group(1)), r[0])
    out = ['---', '', '# Appendix I — Rule Index', '',
           'Every rule of this document, by family, with the Part that defines it.', '']
    total = 0
    for fam in sorted(rows):
        out += [f'## {fam}', '', '| Rule | Part | Begins |', '|---|---|---|']
        for rid, part, text in sorted(rows[fam], key=num):
            out.append(f'| `[{rid}]` | {part} | {text} |')
            total += 1
        out.append('')
    open(os.path.join(HERE, '..', 'parts', 'p27-appx-i.md'), 'w', encoding='utf-8', newline='\n').write('\n'.join(out))
    print('indexed', total, 'rules in', len(rows), 'families')


if __name__ == '__main__':
    main()
