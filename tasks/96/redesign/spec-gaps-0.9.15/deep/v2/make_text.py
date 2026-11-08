"""Plain-text copies of the review's JSON inputs, so agents read them with the Read tool and never need a
script (owner, 2026-11-02: "b, keep the rule strict"). Every line is at most 900 characters (the Read tool
cuts longer lines). Writes deep/v2/text/<name>.md."""
import json
import os
import sys
import textwrap

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.normpath(os.path.join(HERE, '..', '..'))
OUT = os.path.join(HERE, 'text')
os.makedirs(OUT, exist_ok=True)


def wrap(line):
    return textwrap.wrap(line, 900, break_long_words=True, break_on_hyphens=False, replace_whitespace=False,
                         drop_whitespace=False) or ['']


def render(value, depth, out):
    if isinstance(value, dict):
        for k, v in value.items():
            out.append(f"{'#' * min(depth, 6)} {k}" if not isinstance(v, (str, int, float, bool)) or v is None
                       else f"**{k}:**")
            render(v, depth + 1, out)
    elif isinstance(value, list):
        for i, v in enumerate(value, 1):
            out.append(f"{'#' * min(depth, 6)} [{i}]")
            render(v, depth + 1, out)
    else:
        for line in str(value).split('\n'):
            out.extend(wrap(line))
        out.append('')


def convert(src, name):
    data = json.load(open(src, encoding='utf-8'))
    out = [f'# {name} (plain-text copy of {os.path.relpath(src, ROOT)}; same content)', '']
    render(data, 2, out)
    path = os.path.join(OUT, name + '.md')
    open(path, 'w', encoding='utf-8').write('\n'.join(out) + '\n')
    assert max(len(x) for x in out) <= 900, path
    return path


groups = sys.argv[1:]
paths = [convert(os.path.join(ROOT, 'deep', 'phase1-groups.json'), 'phase1-groups'),
         convert(os.path.join(ROOT, 'phase2.json'), 'phase2'),
         convert(os.path.join(ROOT, 'phase3.json'), 'phase3')]
paths += [convert(os.path.join(HERE, 'inputs', f'{g}.json'), g) for g in groups]
for p in paths:
    print(os.path.relpath(p, ROOT), os.path.getsize(p))
