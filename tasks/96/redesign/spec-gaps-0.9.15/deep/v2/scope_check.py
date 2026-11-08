"""Point 2 of the fixed workflow: an answer may edit only the spec rules its gaps were filed under.
Usage: scope_check.py <group>  (reads the group's gaps from phase2.json and its first answer from
phase2-4-partial.json)."""
import json
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.normpath(os.path.join(HERE, '..', '..'))
gid = sys.argv[1]
groups = json.load(open(os.path.join(ROOT, 'deep', 'phase1-groups.json'), encoding='utf-8'))
groups = {g['gid']: g for g in (groups['groups'] if isinstance(groups, dict) else groups)}
items = []


def walk(x):
    if isinstance(x, dict):
        if isinstance(x.get('id'), str):
            items.append(x)
        for v in x.values():
            walk(v)
    elif isinstance(x, list):
        for v in x:
            walk(v)


walk(json.load(open(os.path.join(ROOT, 'phase2.json'), encoding='utf-8')))
gaps = [it for it in items if it['id'] in groups[gid]['members']]
allowed = sorted({r for it in gaps for r in it.get('rules', [])})
answer = json.load(open(os.path.join(ROOT, 'deep', 'phase2-4-partial.json'), encoding='utf-8'))['groups'][gid]['rounds'][0]['design']['spec_text']
edited = set()
for line in answer.split('\n'):
    m = re.match(r"\s*(?:[*-]|\d+\.)?\s*(?:\*\*)?(?:NEW|CHANGED|New|Changed)?\s*`?\[([A-Z]{2,5}-\d+[a-z]?)\]", line)
    if m:
        edited.add(m.group(1))
for part in re.findall(r'(?:Adds|Changes|Replaces|Removes)\b([^.]*)', answer[:900]):
    edited |= set(re.findall(r'\[([A-Z]{2,5}-\d+[a-z]?)\]', part))
outside = sorted(edited - set(allowed))
for it in gaps:
    print(f"Gap {it['id']}: {it.get('title', '')}")
    print(f"   filed under rules: {', '.join(it.get('rules', []))}")
print(f"The answer edited:    {', '.join(sorted(edited))}")
print(f"Edited outside gaps:  {', '.join(outside) or 'none'}")
print('Result:', 'REJECTED, back to the answer agent' if outside else 'passes to the breakers')
