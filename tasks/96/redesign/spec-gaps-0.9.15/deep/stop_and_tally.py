"""Note down a (stopped or running) deep-review run: every finished agent's result, per group, and how many
agents are still needed to finish it. Writes deep/phase2-4-partial.json and prints the table.

Follows the run script's own logic (spec-gaps-deep-review-wf_28e6a473-206.js): design -> two breakers ->
up to two rounds of (repair + two re-breakers) while a high or medium hole is open.
"""
import json
import os
import sys

RUN = sys.argv[1] if len(sys.argv) > 1 else 'wf_28e6a473-206'
JOURNAL = (r'C:/Users/ism19/.claude/projects/C--Users-ism19-Code/5d9166d4-3321-41d7-b8ca-558a365a0258'
           rf'/subagents/workflows/{RUN}/journal.jsonl')
OUT = os.path.join(os.path.dirname(os.path.abspath(__file__)), 'phase2-4-partial.json')
GROUPS = [f'P{i:02d}' for i in range(1, 29)]

label, result = {}, {}
for line in open(JOURNAL, encoding='utf-8'):
    try:
        j = json.loads(line)
    except ValueError:
        continue
    if j.get('type') == 'started':
        label[j['key']] = j['label']
    elif j.get('type') == 'result' and j.get('result') is not None:
        result[label.get(j['key'], j['key'])] = j['result']
in_flight = sorted(v for k, v in label.items() if v not in result)


def serious(breaks):
    return [h for b in breaks for h in b['holes'] if h['severity'] != 'low']


groups, need_min, need_max = {}, 0, 0
for gid in GROUPS:
    g = {'rounds': [], 'in_flight': [x for x in in_flight if x.endswith(':' + gid)]}
    design = result.get(f'design:{gid}')
    if design is None:
        g.update(next='design', need_min=3, need_max=9)
    for r in range(3):
        if design is None:
            break
        pre = 'break' if r == 0 else f'rebreak{r}'
        breaks = {lens: result.get(f'{pre}-{lens}:{gid}') for lens in ('safety', 'engineering')}
        g['rounds'].append({'round': r, 'design': design, 'breaks': breaks})
        missing = [lens for lens, b in breaks.items() if b is None]
        if missing:
            g.update(next=f'{pre}-' + '+'.join(missing), need_min=len(missing), need_max=len(missing) + 3 * (2 - r))
            break
        holes = serious(breaks.values())
        if not holes or r == 2:
            unverified = any(not b['finished'] for b in breaks.values())
            g.update(next=None, need_min=0, need_max=0,
                     status='unverified' if unverified else ('unresolved' if holes else 'holds'), open_holes=len(holes))
            break
        design = result.get(f'repair{r + 1}:{gid}')
        if design is None:
            g.update(next=f'repair{r + 1}', open_holes=len(holes), need_min=3, need_max=3 * (2 - r))
            break
    need_min += g['need_min']
    need_max += g['need_max']
    groups[gid] = g

json.dump({'run': RUN, 'in_flight_when_noted': in_flight, 'agents_still_needed': {'at_least': need_min, 'at_most': need_max},
           'groups': groups}, open(OUT, 'w', encoding='utf-8'), ensure_ascii=False, indent=1)
print(f'finished agents {len(result)}, in flight {len(in_flight)}; still needed: {need_min} to {need_max} agents')
print('| group | rounds done | next | status | agents still needed |')
print('|---|---|---|---|---|')
for gid, g in groups.items():
    print(f"| {gid} | {max(0, len(g['rounds']) - 1)} | {g.get('next') or '-'} | {g.get('status', 'open')} | "
          f"{g['need_min']}" + (f"-{g['need_max']}" if g['need_max'] != g['need_min'] else '') + ' |')
print('wrote', OUT)
