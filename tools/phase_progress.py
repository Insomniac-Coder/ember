"""The README's phase table, computed: how much of each phase's rules is fulfilled.

A rule counts as done only when
  1. a passing test exercises it: a test under `tests/` whose `#$ rules:` line names it, or a Rust
     test (a file under a crate's `tests/`, a crate's `#[cfg(test)]` module, or a runtime test
     template) whose text cites `[RULE]`; a test that expects `E0900` (not built yet) is evidence
     only for the rules about refusing what is not built, and a parse-only test only for the
     lexer and grammar families; and
  2. `docs/phase-status.json` does not list it under `unfinished`; and
  3. for a rule the development target marks `*(new in 0.9.15)*` or `*(changed in 0.9.15)*`,
     `docs/phase-status.json` lists it under `built_for_0915`: its 0.9.15 behaviour is built and
     tested. The tests written for 0.9.10 still name such a rule, so they are no evidence of it
     (the owner, 2026-10-28). A milestone's commit adds the rules it builds.
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
    '9': 'The LLVM backend (ODR-223)',
}
ORDER = ['1', '2', '3', '4', '5', '6', '7', '7a', '8', '9']
RULE = re.compile(r'\[([A-Z]+(?:-[A-Z]+)?-[0-9]+[a-z0-9]*)\]')
TABLE_HEAD = '| Phase | Scope | Done |'


def phase_of(rule, entry):
    family = entry['family']
    if family in STATUS['post_1_0']['families']:
        return 'post'
    placed = STATUS['placed_by_part_or_rule']
    if rule in placed or entry['part'] in placed:
        return placed.get(rule) or placed[entry['part']]
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
    built = {rule for rule in STATUS.get('built_for_0915', []) if not rule.startswith('_')}
    stale = sorted(rule for rule in unfinished | built if rule not in rules)
    if stale:
        sys.exit(f'docs/phase-status.json names rules the target does not define: {stale}')
    reworked = rules_reworked_in_0915((ROOT / target).read_text(encoding='utf-8'))
    unworked = sorted(built - reworked)
    if unworked:
        sys.exit(f'docs/phase-status.json lists under built_for_0915 rules 0.9.15 did not change: {unworked}')
    phases = collections.defaultdict(lambda: {'rules': 0, 'words': 0, 'done_rules': 0, 'done_words': 0, 'untested': [], 'unfinished': []})
    phases['done'] = set()
    for rule, entry in rules.items():
        phase = phase_of(rule, entry)
        if phase is None:
            sys.exit(f'{rule} is in no phase: give family {entry["family"]} one in docs/phase-status.json')
        row = phases[phase]
        row['rules'] += 1
        row['words'] += entry['words']
        if rule in unfinished or (rule in reworked and rule not in built):
            row['unfinished'].append(rule)
        elif rule not in tested:
            row['untested'].append(rule)
        else:
            row['done_rules'] += 1
            row['done_words'] += entry['words']
            phases['done'].add(rule)
    return phases


# `[CONF-2]`–`[CONF-5]` (ODR-151): what each profile adds to the one before it.
PROFILES = [
    ('core', ['II', 'III', 'IV', 'V', 'VI', 'VII', 'X', 'XIII', 'XV.1', 'XV.3', 'XV.4', 'XV.5', 'XV.6']),
    ('systems', ['VIII', 'IX', 'XI', 'XII', 'XIV', 'XV.2', 'XV.7']),
    ('native', ['XVI']),
    ('dynamic', ['Annex B']),
]
CONFORMANCE = ROOT / 'docs' / 'conformance.json'
DECLARED = re.compile(r'const PROFILE: &str = "(\w+)";')


def rules_reworked_in_0915(text):
    """The rules the development target marks `*(new in 0.9.15)*` or `*(changed in 0.9.15)*`."""
    reworked = set()
    for line in text.split('\n'):
        found = re.match(r'^\* `\[([A-Za-z0-9-]+)\]`(.*)$', line)
        if found and ('*(new in 0.9.15)*' in found.group(2) or '*(changed in 0.9.15)*' in found.group(2)):
            reworked.add(found.group(1))
    return reworked


def rule_places(doc):
    """Each rule's Part, Part XV's section (`XV.3`), or annex (`Annex B`)."""
    places, place = {}, 'front'
    for line in doc.split('\n'):
        heading = re.match(r'^# (Part ([IVXL]+)|Annex [A-Z]|Appendix [A-Z])\b', line)
        if heading:
            place = heading.group(2) or heading.group(1)
            continue
        section = re.match(r'^## (XV\.\d+) ', line)
        if section:
            place = section.group(1)
            continue
        rule = re.match(r'^\* `\[([A-Z]+(?:-[A-Z]+)?-[0-9]+[a-z0-9]*)\]`', line)
        if rule:
            places.setdefault(rule.group(1), place)
    return places


def conformance(done):
    """`[CONF-1]` — the profile the evidence earns: the last one, each holding the one before it, all
    of whose rules are done; `none` below Ember Core."""
    target = json.loads((ROOT / 'docs' / 'spec-source' / 'development-target.json').read_text(encoding='utf-8'))['path']
    places = rule_places((ROOT / target).read_text(encoding='utf-8'))
    earned, held, rows, short = 'none', set(), [], False
    for name, adds in PROFILES:
        held |= set(adds)
        rules = sorted(rule for rule, place in places.items() if place in held)
        missing = [rule for rule in rules if rule not in done]
        short = short or bool(missing)
        if not short:
            earned = name
        rows.append({'profile': name, 'adds': adds, 'rules': len(rules), 'done': len(rules) - len(missing), 'missing': missing})
    return {'earned': earned, 'profiles': rows}


# README's lines naming the specification being implemented: kept to the pinned development target.
SPEC_NAME = re.compile(r'Ember_v\d+\.\d+\.\d+_Hardened_\d+')
SPEC_LINES = ('| Language version being implemented |', '| Pinned development target |')


def readme_spec_lines(text):
    """`text` with the specification lines naming the development target's document."""
    target = json.loads((ROOT / 'docs' / 'spec-source' / 'development-target.json').read_text(encoding='utf-8'))['path']
    name = pathlib.Path(target).stem
    lines = text.split('\n')
    for index, line in enumerate(lines):
        if line.startswith(SPEC_LINES):
            lines[index] = SPEC_NAME.sub(name, line)
    return '\n'.join(lines)


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
    profile = conformance(phases['done'])
    evidence = json.dumps(profile, indent=2) + '\n'
    declared = DECLARED.search((ROOT / 'compiler' / 'ember_driver' / 'src' / 'lib.rs').read_text(encoding='utf-8')).group(1)
    if '--check' in sys.argv:
        if text[start:end] != new:
            print('README.md\'s phase table is not what tools/phase_progress.py computes; run it with --write.')
            print(new)
            return 1
        if readme_spec_lines(text) != text:
            print('README.md names a specification other than the pinned development target; run it with --write.')
            return 1
        if not CONFORMANCE.exists() or CONFORMANCE.read_text(encoding='utf-8') != evidence:
            print('docs/conformance.json is not what tools/phase_progress.py computes; run it with --write.')
            return 1
        if declared != profile['earned']:
            print(f"[CONF-1] the compiler claims profile `{declared}` and the evidence earns `{profile['earned']}`: "
                  'set PROFILE in compiler/ember_driver/src/lib.rs to it')
            return 1
        print(f"README phase table and docs/conformance.json match the evidence; profile `{declared}` [CONF-1]")
        return 0
    if '--write' in sys.argv:
        readme.write_text(readme_spec_lines(text[:start] + new + text[end:]), encoding='utf-8', newline='\n')
        CONFORMANCE.write_text(evidence, encoding='utf-8', newline='\n')
    print(new)
    print()
    for phase in ORDER + ['post']:
        row = phases[phase]
        print(f"phase {phase}: {row['done_rules']}/{row['rules']} rules done, {row['done_words']}/{row['words']} words; "
              f"{len(row['untested'])} untested, {len(row['unfinished'])} unfinished")
    print(f"\nprofile earned: {profile['earned']}, claimed: {declared} [CONF-1]")
    for row in profile['profiles']:
        print(f"  {row['profile']}: {row['done']}/{row['rules']} rules done")
    return 0


if __name__ == '__main__':
    sys.exit(main())
