"""Method 2 inputs: one file per group from deep/phase2-4-partial.json (method 1's results), and the
length cap per group (the first design's rule + spec_text, in characters), printed for the run's args."""
import json
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
partial = json.load(open(os.path.join(HERE, '..', 'phase2-4-partial.json'), encoding='utf-8'))
groups = sys.argv[1:] or sorted(partial['groups'])
cap = {}
for gid in groups:
    rounds = partial['groups'][gid]['rounds']
    first = rounds[0]['design']
    holes = []
    for r in rounds:
        for lens, b in r['breaks'].items():
            if not b:
                continue
            for h in b['holes']:
                holes.append({'id': f'H{len(holes) + 1}', 'against': 'first design' if r['round'] == 0 else 'method-1 repair',
                              'lens': lens, **h})
            if not b['finished']:
                holes.append({'id': f'H{len(holes) + 1}', 'against': 'first design' if r['round'] == 0 else 'method-1 repair',
                              'lens': lens, 'kind': 'not_finished', 'severity': 'note', 'program': '',
                              'what_goes_wrong': 'this breaker was cut off and did not finish: ' + b['notes'][:400], 'fix_hint': ''})
    out = {'gid': gid, 'first_design': first,
           'method1_repair': rounds[1]['design'] if len(rounds) > 1 else None,
           'holes': holes}
    json.dump(out, open(os.path.join(HERE, 'inputs', f'{gid}.json'), 'w', encoding='utf-8'), ensure_ascii=False, indent=1)
    cap[gid] = len(first['rule']) + len(first['spec_text'])
    n = sum(h['severity'] in ('high', 'medium') for h in holes)
    print(f'{gid}: {len(holes)} holes ({n} high/medium), cap {cap[gid]} chars')
print(json.dumps(cap))
