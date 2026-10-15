"""The README's phase table, computed: how much of each phase's rules is fulfilled.

A rule counts as done only when
  1. a passing test exercises it: a test under `tests/` whose `#$ rules:` line names it, or a Rust
     test (a file under a crate's `tests/`, a crate's `#[cfg(test)]` module, or a runtime test
     template) whose text cites `[RULE]`; a test that expects `E0900` (not built yet) is evidence
     only for the rules about refusing what is not built, and a parse-only test only for the
     lexer and grammar families; and
  2. `docs/phase-status.json` does not list it under `unfinished`.
A phase's figure is its done rules' words over all its rules' words, as `rule_sizes.py` sizes
them; every rule of the development target is in exactly one phase. Nothing can be marked done
by hand: a figure rises only when a rule gets a test and nothing records it unfinished.

Usage:
  python tools/phase_progress.py            print the table, and each phase's unfinished rules
  python tools/phase_progress.py --write    rewrite the table in README.md
  python tools/phase_progress.py --check    fail if README.md's table is not what this computes
"""
import collections
import json
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'tasks' / 'impl-0.9.9'))
import rule_sizes as rs  # noqa: E402

STATUS = json.loads((ROOT / 'docs' / 'phase-status.json').read_text(encoding='utf-8'))
SCOPES = {
    '1': 'Core language (Parts II–VI)',
    '2': 'Ownership, borrowing, regions',
    '3': 'Classes, reference counting, exclusivity',
    '4': 'Effects, compile time, reflection, derives',
    '5': 'C interoperability',
    '6': 'Concurrency and data-oriented design',
    '7': 'C++ interoperability and the interpreter',
    '7a': 'Iteration, determinism, cost control',
    '8': 'Hardening and 1.0',
}
ORDER = ['1', '2', '3', '4', '5', '6', '7', '7a', '8']
RULE = re.compile(r'\[([A-Z]+(?:-[A-Z]+)?-[0-9]+[a-z0-9]*)\]')
TABLE_HEAD = '| Phase | Scope | Done |'


def phase_of(entry):
    family = entry['family']
    if family in STATUS['post_1_0']['families']:
        return 'post'
    for phase, families in rs.PHASES.items():
        if family in families:
            return phase
    if entry['part'] in rs.EARLY_PARTS:
        return '1'
    return STATUS['unmapped_families'].get(family)


def tested_rules():
    """Each rule a passing test exercises, by the evidence rule 1 above admits."""
    meta = set(STATUS['meta_rules']['rules'])
    found = set()
    for path in (ROOT / 'tests').rglob('*.em'):
        text = path.read_text(encoding='utf-8', errors='replace')
        kind = re.search(r'^#\$ test: (\S+)', text, re.M)
        refusal = 'error[E0900]' in text
        parse_only = bool(kind) and kind.group(1).startswith('parse-')
        for line in text.split('\n'):
            if not line.startswith('#$ rules:'):
                continue
            for rule in (r.strip() for r in line[len('#$ rules:'):].split(',')):
                if refusal and rule not in meta:
                    continue
                if parse_only and not rule.startswith(('LEX-', 'GRM-')):
                    continue
                found.add(rule)
    sources = list((ROOT / 'compiler').glob('*/tests/**/*.rs'))
    sources += list((ROOT / 'runtime').rglob('templates/tests/*.c.in'))
    for path in sources:
        found.update(RULE.findall(path.read_text(encoding='utf-8', errors='replace')))
    for path in (ROOT / 'compiler').glob('*/src/**/*.rs'):
        for module in test_modules(path.read_text(encoding='utf-8', errors='replace')):
            found.update(RULE.findall(module))
    return found


def test_modules(text):
    """The bodies of a Rust file's `#[cfg(test)] mod name { … }` blocks, and nothing else: a
    `#[cfg(test)]` helper in the middle of a file must not make the code after it count."""
    for start in re.finditer(r'#\[cfg\(test\)\]\s*(?:#\[[^\]]*\]\s*)*(?:pub(?:\([^)]*\))?\s+)?mod\s+\w+\s*\{', text):
        depth, i = 1, start.end()
        while depth and i < len(text):
            raw = re.match(r'r(#*)"', text[i:i + 8]) if text[i] == 'r' and not text[i - 1].isalnum() else None
            char = re.match(r"'(\\.[^']*|[^\\'])'", text[i:i + 12]) if text[i] == "'" else None
            if text.startswith('//', i):
                i = text.find('\n', i)
                i = len(text) if i < 0 else i
            elif raw:
                end = text.find('"' + raw.group(1), i + len(raw.group(0)))
                i = len(text) if end < 0 else end + 1 + len(raw.group(1))
            elif text[i] == '"':
                i += 1
                while i < len(text) and text[i] != '"':
                    i += 2 if text[i] == '\\' else 1
                i += 1
            elif char:
                i += len(char.group(0))
            else:
                depth += {'{': 1, '}': -1}.get(text[i], 0)
                i += 1
        yield text[start.end():i]


def _check_test_modules():
    source = ('fn a() { "[A-1]" }\n#[cfg(test)]\nfn helper() {}\nfn b() { "[B-1]" }\n'
              '#[cfg(test)]\nmod tests {\n    // a } in a comment\n    const S: &str = "}{";\n'
              '    const R: &str = r#"a "}" b"#;\n    const C: char = \'}\';\n    const Q: char = \'\\\'\';\n'
              '    fn t() { let _ = "[C-1]"; }\n}\nfn after() { "[D-1]" }\n')
    found = set(RULE.findall(''.join(test_modules(source))))
    assert found == {'C-1'}, found


def compute():
    target = json.loads((ROOT / 'docs' / 'spec-source' / 'development-target.json').read_text(encoding='utf-8'))['path']
    rules = rs.rule_sizes((ROOT / target).read_text(encoding='utf-8'))
    tested = tested_rules()
    unfinished = {rule for rule in STATUS['unfinished'] if not rule.startswith('_')}
    stale = sorted(rule for rule in unfinished if rule not in rules)
    if stale:
        sys.exit(f'docs/phase-status.json names rules the target does not define: {stale}')
    phases = collections.defaultdict(lambda: {'rules': 0, 'words': 0, 'done_rules': 0, 'done_words': 0, 'untested': [], 'unfinished': []})
    for rule, entry in rules.items():
        phase = phase_of(entry)
        if phase is None:
            sys.exit(f'{rule} is in no phase: give family {entry["family"]} one in docs/phase-status.json')
        row = phases[phase]
        row['rules'] += 1
        row['words'] += entry['words']
        if rule in unfinished:
            row['unfinished'].append(rule)
        elif rule not in tested:
            row['untested'].append(rule)
        else:
            row['done_rules'] += 1
            row['done_words'] += entry['words']
    return phases


def percent(done, total):
    return 0 if total == 0 else int(100 * done / total)


def table(phases):
    lines = [TABLE_HEAD, '|---|---|---:|']
    for phase in ORDER:
        row = phases[phase]
        lines.append(f"| {phase} | {SCOPES[phase]} | {percent(row['done_words'], row['words'])}% |")
    done = sum(phases[p]['done_words'] for p in ORDER)
    total = sum(phases[p]['words'] for p in ORDER)
    lines.append(f'| | Overall (1.0) | {percent(done, total)}% |')
    post = phases['post']
    families = ', '.join(STATUS['post_1_0']['families'].values())
    lines.append(f"| | After 1.0: {families} | {percent(post['done_words'], post['words'])}% |")
    return '\n'.join(lines)


def readme_table(text):
    start = text.index(TABLE_HEAD)
    end = text.index('\n\n', start)
    return start, end


def main():
    _check_test_modules()
    phases = compute()
    new = table(phases)
    readme = ROOT / 'README.md'
    text = readme.read_text(encoding='utf-8')
    start, end = readme_table(text)
    if '--check' in sys.argv:
        if text[start:end] != new:
            print('README.md\'s phase table is not what tools/phase_progress.py computes; run it with --write.')
            print(new)
            return 1
        print('README phase table matches the evidence')
        return 0
    if '--write' in sys.argv:
        readme.write_text(text[:start] + new + text[end:], encoding='utf-8', newline='\n')
    print(new)
    print()
    for phase in ORDER + ['post']:
        row = phases[phase]
        print(f"phase {phase}: {row['done_rules']}/{row['rules']} rules done, {row['done_words']}/{row['words']} words; "
              f"{len(row['untested'])} untested, {len(row['unfinished'])} unfinished")
    return 0


if __name__ == '__main__':
    sys.exit(main())
