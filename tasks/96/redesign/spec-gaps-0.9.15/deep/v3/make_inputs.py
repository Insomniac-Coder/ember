"""Method 3 inputs: one plain-text file per cluster (lines <= 900 characters, so agents read it with the
Read tool and never need a script), and the `args` object for deep/v3/workflow.js, printed as JSON.

Usage (from anywhere; the lead runs it, never an agent):
    python deep/v3/make_inputs.py K04 K05 K10 [--cap 1200] [--edits 3] [--limit N]

A cluster is named by its prefix (K04) or its full id (K04-weakref-outlives-arena). The clusters keep the
order given: inside one group they are answered in that order, each seeing the earlier answers.

Reads: phase3.json (clusters, conflicts), phase2.json (the questions), deep/phase1-groups.json (groups),
tasks/96/redesign/JOB.md (verdicts), tasks/spec-0.9.9/parts/p*.md (rule texts).
Writes: deep/v3/inputs/<cid>.md. Prints: a table (cluster, group, members, allowed rules, input size,
rules whose text was not found as a bullet) and then one line `ARGS: {...}` to paste as the Workflow
tool's `args`. It never touches the specification or any other file.
"""
import glob
import json
import os
import re
import sys
import textwrap

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.normpath(os.path.join(HERE, '..', '..'))            # spec-gaps-0.9.15
REPO = os.path.normpath(os.path.join(ROOT, '..', '..', '..', '..'))  # the repository
PARTS = os.path.join(REPO, 'tasks', 'spec-0.9.9', 'parts')
JOB = os.path.join(REPO, 'tasks', '96', 'redesign', 'JOB.md')
OUT = os.path.join(HERE, 'inputs')
RULE_ID = re.compile(r'\[([A-Z]{2,5}-\d+[a-z]?)\]')
RULE_DEF = re.compile(r'^[*-] `\[([A-Z]{2,5}-\d+[a-z]?)\]`')


def load(path):
    with open(path, encoding='utf-8') as f:
        return json.load(f)


def wrap(line):
    return textwrap.wrap(line, 900, break_long_words=True, break_on_hyphens=False, replace_whitespace=False,
                         drop_whitespace=False) or ['']


def render(value, depth, out):
    """The same shape as deep/v2/make_text.py: a dict's keys become headings or bold labels."""
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


def items_of(phase2):
    found = {}

    def walk(x):
        if isinstance(x, dict):
            if isinstance(x.get('id'), str) and 'verdict' in x:
                found.setdefault(x['id'], x)
            for v in x.values():
                walk(v)
        elif isinstance(x, list):
            for v in x:
                walk(v)
    walk(phase2)
    return found


def rule_texts():
    """Every rule defined as a bullet in the part files: id -> (part file, first line, last line, text)."""
    defs = {}
    for path in sorted(glob.glob(os.path.join(PARTS, 'p*.md'))):
        with open(path, encoding='utf-8') as f:
            lines = f.read().split('\n')
        name = os.path.basename(path)
        i = 0
        while i < len(lines):
            m = RULE_DEF.match(lines[i])
            if not m:
                i += 1
                continue
            j = i + 1
            # a bullet's continuation lines are indented; one blank line inside the bullet is kept
            while j < len(lines) and (lines[j].startswith('  ') or (lines[j] == '' and j + 1 < len(lines) and lines[j + 1].startswith('  '))):
                j += 1
            rid = m.group(1)
            if rid not in defs:                       # the first definition is the rule; later ones are quotes
                defs[rid] = (name, i + 1, j, '\n'.join(lines[i:j]))
            i = j
    return defs


def find_anywhere(rid):
    """Fallback for a rule not defined as a bullet (a table row, a heading): the first mention and 12 lines."""
    for path in sorted(glob.glob(os.path.join(PARTS, 'p*.md'))):
        with open(path, encoding='utf-8') as f:
            lines = f.read().split('\n')
        for i, line in enumerate(lines):
            if f'[{rid}]' in line:
                return (os.path.basename(path), i + 1, min(len(lines), i + 12), '\n'.join(lines[i:i + 12]))
    return None


def verdict_lines():
    """JOB.md's numbered verdicts: number -> the paragraph (the numbered line and what follows to a blank)."""
    with open(JOB, encoding='utf-8') as f:
        lines = f.read().split('\n')
    out = {}
    for i, line in enumerate(lines):
        m = re.match(r'^(\d+)\. \*\*', line)
        if not m:
            continue
        j = i + 1
        while j < len(lines) and lines[j].strip() and not re.match(r'^\d+\. \*\*', lines[j]):
            j += 1
        out[int(m.group(1))] = f"JOB.md {i + 1}-{j}: " + '\n'.join(lines[i:j])
    return out


def main():
    argv = sys.argv[1:]
    cap, edits, limit = 1200, 3, None
    wanted = []
    i = 0
    while i < len(argv):
        if argv[i] == '--cap':
            cap = int(argv[i + 1]); i += 2
        elif argv[i] == '--edits':
            edits = int(argv[i + 1]); i += 2
        elif argv[i] == '--limit':
            limit = int(argv[i + 1]); i += 2
        else:
            wanted.append(argv[i]); i += 1
    assert wanted, __doc__

    phase3 = load(os.path.join(ROOT, 'phase3.json'))
    phase2 = items_of(load(os.path.join(ROOT, 'phase2.json')))
    phase1 = {}

    def walk1(x):
        if isinstance(x, dict):
            if isinstance(x.get('id'), str) and 'rules' in x:
                phase1.setdefault(x['id'], x)
            for v in x.values():
                walk1(v)
        elif isinstance(x, list):
            for v in x:
                walk1(v)
    walk1(load(os.path.join(ROOT, 'phase1.json')))
    groups = load(os.path.join(ROOT, 'deep', 'phase1-groups.json'))
    groups = groups['groups'] if isinstance(groups, dict) else groups
    clusters = phase3['clusters'] if isinstance(phase3, dict) else phase3
    conflicts = phase3.get('conflicts', []) if isinstance(phase3, dict) else []
    defs = rule_texts()
    verdicts = verdict_lines()
    os.makedirs(OUT, exist_ok=True)

    args = {'limit': limit, 'cap_chars': cap, 'max_edits': edits, 'known_rules': sorted(defs), 'clusters': []}
    rows = []
    for want in wanted:
        matches = [c for c in clusters if c['cid'] == want or c['cid'].startswith(want + '-')]
        assert len(matches) == 1, f'{want}: {len(matches)} clusters match'
        cl = matches[0]
        members = cl['members']
        questions = [phase2[m] for m in members if m in phase2]
        assert len(questions) == len(members), f"{cl['cid']}: a member is missing from phase2.json"
        group = next((g for g in groups if cl['cid'] in g.get('clusters', []) or any(m in g.get('members', []) for m in members)), None)
        assert group, f"{cl['cid']}: no group in phase1-groups.json holds it"
        allowed = set()
        for q in questions:
            # the rules the question was filed under: phase2's copy, or, where phase2 dropped them, the
            # finding itself in phase1.json (J8-08, J8-09, J6-16, J8-04 have none in phase2)
            allowed |= set(q.get('rules') or phase1.get(q['id'], {}).get('rules', []))
            allowed |= set(RULE_ID.findall(q.get('spec_today', '')))
        allowed = sorted(allowed)
        texts, located, missing = {}, {}, []
        for rid in allowed:
            d = defs.get(rid) or find_anywhere(rid)
            if d is None:
                missing.append(rid)
                continue
            texts[rid] = d[3]
            located[rid] = f'{d[0]} lines {d[1]}-{d[2]}'
        verdict_nums = []
        for v in group.get('verdicts_involved', []):
            m = re.match(r'^(\d+)', str(v))
            if m:
                verdict_nums.append(int(m.group(1)))
        related_conflicts = [c for c in conflicts if any(m in c.get('ids', []) for m in members)]

        doc = {
            'cluster': cl,
            'questions': questions,
            'group': group,
            'conflicts_naming_a_member': related_conflicts,
            'verdicts_involved': {str(n): verdicts.get(n, f'verdict {n}: not found as a numbered line in JOB.md') for n in verdict_nums},
            'allowed_rules': allowed,
            'rule_text': {rid: f"{located[rid]}\n{t}" for rid, t in texts.items()},
            'rules_not_found': missing,
        }
        out = [f"# {cl['cid']} (input for method 3; made by deep/v3/make_inputs.py from phase2.json, phase3.json, deep/phase1-groups.json, JOB.md and the part files)", '']
        render(doc, 2, out)
        assert max(len(x) for x in out) <= 900
        path = os.path.join(OUT, cl['cid'] + '.md')
        with open(path, 'w', encoding='utf-8') as f:
            f.write('\n'.join(out) + '\n')
        args['clusters'].append({'cid': cl['cid'], 'gid': group['gid'], 'members': members, 'allowed_rules': allowed,
                                 'rule_text': texts, 'input': path})
        rows.append((cl['cid'], group['gid'], ', '.join(members), ', '.join(allowed), os.path.getsize(path), ', '.join(missing) or '-'))

    print('| cluster | group | members | allowed rules | input bytes | rule text not found |')
    print('|---|---|---|---|---:|---|')
    for r in rows:
        print('| ' + ' | '.join(str(x) for x in r) + ' |')
    print(f'{len(defs)} rules known as bullets in the part files; cap {cap} characters, {edits} edits; limit {limit or "(the owner sets it at launch)"}')
    print('ARGS: ' + json.dumps(args, ensure_ascii=False))


if __name__ == '__main__':
    main()
