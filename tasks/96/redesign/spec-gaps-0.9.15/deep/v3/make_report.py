"""After a run of deep/v3/workflow.js: the owner's report and the proposed conformance tests, from the
value the Workflow tool returned (the lead saves it as deep/v3/results/<batch>.json first; the run's
journal.jsonl holds the same per-agent results if the tool result was lost).

Usage (the lead runs it, never an agent):
    python deep/v3/make_report.py deep/v3/results/pilot.json

Writes:
  deep/v3/results/<batch>-report.md          what the owner reads: one block per cluster in the shape that
                                             works (one line in C++ terms, step table, today, options,
                                             recommendation), then the edits as before/after, holes left,
                                             new gaps, and the agent count
  deep/v3/results/<batch>-tests/<cid>/*.em   every must-refuse / must-compile / must-run program with the
                                             `#$` header the harness reads (tests/compile-fail style); they
                                             stay here until the owner's verdict and are run by hand
Prints: the status table, the agent total, and the number of tests written. Writes nothing else.
"""
import json
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
CODE = re.compile(r'\bE\d{4}\b')


def safe(name):
    return re.sub(r'[^A-Za-z0-9_.-]+', '_', name)[:60] or 'x'


def header(test):
    kind = {'must_refuse': 'compile-fail', 'must_compile': 'compile-pass', 'must_run': 'run-pass'}[test['kind']]
    lines = [f'#$ test: {kind}']
    if test.get('rule'):
        lines.append(f"#$ rules: {test['rule']}")
    expected = test.get('expected', '')
    if kind == 'compile-fail':
        m = CODE.search(expected)
        lines.append(f'#$ error[{m.group(0)}]' if m else f'#$ note: expected (no code named yet; the lead assigns one): {expected}')
    elif kind == 'run-pass':
        for out in expected.split('\n'):
            if out.strip():
                lines.append(f'#$ stdout: {out}')
    else:
        lines.append(f'#$ note: {expected}')
    return lines


CODE_START = re.compile(r'^(import|from|fn|gen|class|struct|enum|interface|static|type|extern|unsafe|pub|const|@)\b')
DESCRIPTION = re.compile(r'verbatim|\bappears?\b|\bprinted\b|\bprints\b|\bno freeze\b|\bat most\b|\bonly after\b|'
                         r'\bevery line\b|\bthe last line\b|\be\.g\.|[a-z]\. [A-Z]', re.I)


def test_problem(t):
    """Why a breaker's test is not a test (owner, 2026-11-02, approving the check): a program written as a
    description instead of code, or a run's expected output written as a sentence instead of the text."""
    # a test may start with the C or C++ header it imports (`// renderer.hpp`), so any Ember line counts;
    # a program that only points elsewhere ("verbatim", "as in round 1") is not a test
    code = [l.strip() for l in t.get('program', '').split('\n') if l.strip() and not l.lstrip().startswith('#')]
    pointer = re.search(r'verbatim|\bas in (the answer|round|r\d|the first)', t.get('program', ''), re.I)
    if not any(CODE_START.match(l) for l in code) or pointer:
        return 'the program is a description, not code: ' + (code[0][:90] if code else '(empty)')
    if t.get('kind') == 'must_run':
        for out in t.get('expected', '').split('\n'):
            if DESCRIPTION.search(out):
                return 'the expected output is a description, not the exact text: ' + out.strip()[:90]
    return None


def block(r):
    out = [f"## {r['cid']} ({r.get('gid', '?')}; {', '.join(r.get('members', []))}): **{r['status']}**, {r.get('agents', 0)} agents", '']
    final = r.get('final') or {}
    if not final:
        out.append('(no finished answer)')
        return out
    out.append(f"Outcome: `{final.get('outcome')}`. Intent: {final.get('intent', '')}")
    er = final.get('existing_rule') or {}
    if er.get('rule'):
        out.append(f"The rule that decides it: {er['rule']} ({er.get('part_file', '')} {er.get('lines', '')}): \"{er.get('words', '')}\"")
    out.append('')
    for e in final.get('edits', []):
        out += [f"### Edit: {e['rule']} ({e['kind']}, {e.get('part_file', '')} {e.get('lines', '')})", '', '**Before:**', '', '> ' + e['before'].replace('\n', '\n> '), '',
                '**After:**', '', '> ' + e['after'].replace('\n', '\n> '), '', f"Why general: {e.get('why_general', '')}", '']
    for d in final.get('decisions', []):
        s = d.get('owner_short') or {}
        who = "made by the owner's parameters" if d.get('made_by') == 'parameters' else 'for the owner'
        out += [f"### Decision ({who}): {d.get('title', '')}", '',
                f"**{s.get('one_line', '')}**", '', s.get('step_table', ''), '', f"Today: {s.get('today', '')}", '']
        for o in s.get('options', []):
            out.append(f'* {o}')
        out += ['', f"Recommendation: {s.get('recommendation', '')}", '']
        if d.get('made_by') == 'parameters':
            out += [f"Chosen: {d.get('chosen', '')}. By: {d.get('parameters_in_order', '')}", '']
        for o in d.get('options', []):
            out += [f"Option {o.get('letter', '')}:", '', '```', o.get('code', ''), '```', '', o.get('outcome', ''), '']
    rd = final.get('readability') or {}
    if rd.get('verdict'):
        out += [f"Readability: {rd['verdict']}", f"Cost: {final.get('cost', '')}", '']
    checks = [s.get('out') for s in r.get('steps', []) if s.get('step') == 'check' and s.get('out')]
    if checks:
        v = [x['verdict'] for x in checks[-1].get('verdicts', [])]
        out += [f"Final check of the last breakers' holes: {v.count('real')} confirmed real, {v.count('refuted')} refuted, "
                f"{v.count('pre_existing')} already in today's specification (only the confirmed count)", '']
    if r.get('holes_left'):
        out.append(f"### Holes left ({len(r['holes_left'])})")
        for h in r['holes_left']:
            out += ['', f"* **{h.get('kind')}**, {h.get('severity')}: {h.get('what_goes_wrong', '')}", '', '```', h.get('program', ''), '```']
        out.append('')
    if r.get('new_gaps'):
        out += [f"New gaps found on the way (not this edit's): {len(r['new_gaps'])}", '']
    rejected = [(t, test_problem(t)) for t in r.get('tests', []) if test_problem(t)]
    if rejected:
        out.append(f"Tests rejected by the script, not written ({len(rejected)}):")
        out += [f"* {t.get('name', '')}: {why}" for t, why in rejected] + ['']
    if final.get('to_verify'):
        out += ['To verify on the compiler:'] + [f'* {t}' for t in final['to_verify']] + ['']
    return out


def main():
    src = sys.argv[1]
    with open(src, encoding='utf-8') as f:
        data = json.load(f)
    clusters = data['clusters'] if isinstance(data, dict) else data
    base = os.path.splitext(src)[0]
    report = [f'# Report: {os.path.basename(base)} (made by deep/v3/make_report.py; nothing here is in the specification)', '',
              '| cluster | status | agents | decisions | holes left | tests |', '|---|---|---:|---:|---:|---:|']
    tests_written = 0
    tests_rejected = 0
    total_agents = 0
    for r in clusters:
        final = r.get('final') or {}
        tests = [t for t in r.get('tests', []) if not test_problem(t)]
        tests_rejected += len(r.get('tests', [])) - len(tests)
        report.append(f"| {r['cid']} | {r['status']} | {r.get('agents', 0)} | {len(final.get('decisions', []))} | {len(r.get('holes_left', []))} | {len(tests)} |")
        total_agents += r.get('agents', 0)
        folder = os.path.join(base + '-tests', safe(r['cid']))
        if tests or final.get('answers'):
            os.makedirs(folder, exist_ok=True)
        for i, t in enumerate(tests, 1):
            path = os.path.join(folder, f"{i:02d}_{t['kind']}_{safe(t.get('name', ''))}.em")
            with open(path, 'w', encoding='utf-8') as f:
                f.write('\n'.join(header(t)) + '\n' + t['program'].rstrip('\n') + '\n')
            tests_written += 1
        for i, a in enumerate(final.get('answers', []), 1):
            path = os.path.join(folder, f"q{i:02d}_{safe(a.get('question', ''))}.em")
            with open(path, 'w', encoding='utf-8') as f:
                f.write(f"#$ note: {a.get('question', '')}: today: {a.get('today', '')} | after: {a.get('after', '')}\n" + a['program'].rstrip('\n') + '\n')
            tests_written += 1
    report.append('')
    for r in clusters:
        report += block(r)
    with open(base + '-report.md', 'w', encoding='utf-8') as f:
        f.write('\n'.join(report) + '\n')
    print('\n'.join(report[2:4 + len(clusters)]))
    print(f'{total_agents} agents; {tests_written} test files under {base}-tests ({tests_rejected} rejected by the script: '
          f'a description where the program or the exact output should be); report {base}-report.md')


if __name__ == '__main__':
    main()
