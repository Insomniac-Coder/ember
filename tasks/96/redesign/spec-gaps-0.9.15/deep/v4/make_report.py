"""After a run of deep/v4/workflow.js: the owner's report and the proposed tests.
Usage (the lead runs it): python -I -X utf8 deep/v4/make_report.py deep/v4/results/<batch>.json
Writes <batch>-report.md and <batch>-tests/<cid>/*.em (tests that are descriptions, not code, are not
written and are listed with the reason). Prints the status table."""
import importlib.util
import json
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
_spec = importlib.util.spec_from_file_location('v3report', os.path.join(HERE, '..', 'v3', 'make_report.py'))
v3 = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(v3)          # header(), safe(), test_problem(): one implementation for both methods


QUOTED = re.compile(r'`([^`]+)`|"([^"]+)"')


def salvage(t):
    """A run test whose exact output sits inside a sentence ('prints "3 1" (no warnings)', 'prints exactly
    three lines, `0`, `1`, `2`, and exits 0 (today: ...)') keeps the quoted text after 'prints' as its
    output, with the agent's words kept in a note. Anything else stays as it is for the check."""
    if t.get('kind') != 'must_run' or not v3.test_problem(t) or 'description, not the exact text' not in v3.test_problem(t):
        return t
    text = t.get('expected', '')
    m = re.search(r'\bprints?\b(.*)', text, re.I | re.S)
    if not m:
        return t
    tail = re.split(r'\(today|;|\bwhile\b|\bthen\b', m.group(1))[0]
    lines = [a or b for a, b in QUOTED.findall(tail)]
    if not lines:
        return t
    fixed = dict(t, expected='\n'.join(lines), note_expected=text)
    return t if v3.test_problem(fixed) else fixed


def short(s, title):
    if not s:
        return []
    out = [f'**{title}: {s.get("one_line", "")}**', '', s.get('step_table', ''), '', f'Today: {s.get("today", "")}', '']
    out += [f'* {o}' for o in s.get('options', [])]
    return out + ['', f'Recommendation: {s.get("recommendation", "")}', '']


def block(r):
    d = r.get('final') or {}
    out = [f"## {r['cid']} ({r.get('gid', '?')}; {', '.join(r.get('members', []))}): **{r['status']}**, {r.get('agents', 0)} agents", '']
    if not d:
        return out + ['(no change written)', '']
    cause = d.get('cause') or {}
    out += [f"Outcome: `{d.get('outcome')}`. Intent: {d.get('intent', '')}", '',
            f"Where the cause is: {cause.get('where', '')} (reported at: {cause.get('reported_at', '')}). Why there: {cause.get('why_there', '')}", '']
    arb = r.get('arbiter') or {}
    out += short(arb.get('owner_short'), 'In short')
    if arb.get('summary'):
        out += [f"Arbiter: {arb['summary']}", '']
    for e in d.get('edits', []):
        out += [f"### Edit: {e['rule']} ({e['kind']}, {e.get('part_file', '')} {e.get('lines', '')})", '', '**Before:**', '',
                '> ' + (e.get('before') or '(new rule)').replace('\n', '\n> '), '', '**After:**', '', '> ' + e['after'].replace('\n', '\n> '), '',
                f"Why here: {e.get('why_here', '')}", '', f"Why general: {e.get('why_general', '')}", '']
    imp = d.get('impact') or {}
    rd = imp.get('readability') or {}
    out += ['### Impact', '', f"Rules changed: {imp.get('rules_changed', '')}", '', f"Readability: {rd.get('verdict', '')}", '',
            f"Cost: {imp.get('cost', '')}", '']
    for v in imp.get('verdicts', []):
        out.append(f"* verdict {v.get('verdict')}: {v.get('kept_or_departs')} ({v.get('how', '')})")
    for p in imp.get('programs_changed', []):
        out.append(f"* a program changes: {p.get('before', '')} -> {p.get('after', '')}")
    out.append('')
    for x in d.get('decisions', []):
        who = "made by the owner's parameters" if x.get('made_by') == 'parameters' else 'for the owner'
        out += short(x.get('owner_short'), f"Decision ({who}): {x.get('title', '')}")
        if x.get('made_by') == 'parameters':
            out += [f"Chosen: {x.get('chosen', '')}. By: {x.get('parameters_in_order', '')}", '']
    if r.get('blockers'):
        out.append(f"### What blocks it ({len(r['blockers'])})")
        ids = {o['id']: o for o in r.get('open', [])}
        for b in r['blockers']:
            o = ids.get(b.get('id') or b['objection'], {})
            out += ['', f"* {b.get('id') or b['objection']} ({o.get('block')}, {o.get('kind')}): {o.get('impact', '')}", f"  Arbiter: {b.get('evidence', '')}", '', '```', o.get('program', ''), '```']
        out.append('')
    if r.get('owner_must_see'):
        out.append('### Trade-offs the owner must see')
        for t in r['owner_must_see']:
            if 'justified' in t:          # an arbiter's ruling on a trade-off
                out.append(f"* {t['objection']}: {'justified' if t.get('justified') else 'NOT justified'}: {t.get('why', '')}")
            else:                         # a weighed objection still open; no arbiter was needed (nothing blocked)
                out.append(f"* {t.get('id')} ({t.get('kind')}, {t.get('severity')}): {t.get('impact', '')}")
        out.append('')
    if arb.get('rulings'):
        v = [x['verdict'] for x in arb['rulings']]
        out += [f"Arbiter's rulings on the open objections: {v.count('real')} real, {v.count('refuted')} refuted, {v.count('pre_existing')} already in today's specification", '']
    if r.get('closed'):
        out += [f"Objections closed in the debate: {len(r['closed'])} ({sum(1 for c in r['closed'] if c.get('closed_as') == 'resolved')} fixed, {sum(1 for c in r['closed'] if c.get('closed_as') == 'withdrawn')} withdrawn after a rebuttal)", '']
    if r.get('new_gaps'):
        out += [f"Gaps found on the way that exist without the change: {len(r['new_gaps'])}", '']
    if r.get('unjudged'):
        out += [f"Objections the arbiter did not judge ({len(r['unjudged'])}):"] + [f"* {o.get('id')} ({o.get('block')}, {o.get('kind')}): {o.get('impact', '')}" for o in r['unjudged']] + ['']
    notes = [n for s in r.get('steps', []) for n in s.get('notes', [])]
    if notes:
        out += ['Corrected or noted by the script, not rejected:'] + [f'* {n}' for n in notes] + ['']
    rejected = [(t, v3.test_problem(t)) for t in map(salvage, r.get('tests', [])) if v3.test_problem(t)]
    if rejected:
        out += [f'Tests rejected by the script ({len(rejected)}):'] + [f"* {t.get('name', '')}: {why}" for t, why in rejected] + ['']
    return out


def main():
    src = sys.argv[1]
    data = json.load(open(src, encoding='utf-8'))
    clusters = data['clusters'] if isinstance(data, dict) else data
    base = os.path.splitext(src)[0]
    rep = [f'# Report: {os.path.basename(base)} (method 4; nothing here is in the specification)', '',
           '| question | status | agents | decisions | open objections | tests kept |', '|---|---|---:|---:|---:|---:|']
    written = rejected = agents = 0
    for r in clusters:
        d = r.get('final') or {}
        tests = [t for t in map(salvage, r.get('tests', [])) if not v3.test_problem(t)]
        rejected += len(r.get('tests', [])) - len(tests)
        agents += r.get('agents', 0)
        rep.append(f"| {r['cid']} | {r['status']} | {r.get('agents', 0)} | {len(d.get('decisions', []))} | {len(r.get('open', []))} | {len(tests)} |")
        folder = os.path.join(base + '-tests', v3.safe(r['cid']))
        if tests:
            os.makedirs(folder, exist_ok=True)
        for i, t in enumerate(tests, 1):
            with open(os.path.join(folder, f"{i:02d}_{t['kind']}_{v3.safe(t.get('name', ''))}.em"), 'w', encoding='utf-8') as f:
                note = [f"#$ note: exact output taken from the agent's words: {' '.join(t['note_expected'].split())}"] if t.get('note_expected') else []
                f.write('\n'.join(v3.header(t) + note) + '\n' + t['program'].rstrip('\n') + '\n')
            written += 1
    rep.append('')
    for r in clusters:
        rep += block(r)
    with open(base + '-report.md', 'w', encoding='utf-8') as f:
        f.write('\n'.join(rep) + '\n')
    print('\n'.join(rep[2:4 + len(clusters)]))
    print(f'{agents} agents; {written} tests written, {rejected} rejected by the script; report {base}-report.md')


if __name__ == '__main__':
    main()
