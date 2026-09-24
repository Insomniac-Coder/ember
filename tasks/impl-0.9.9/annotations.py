"""Check a directory's annotations against the compiler in seconds, rather than the full
conformance run. Matching follows the harness in `ember_driver/tests/milestones.rs`
(`[TST-1]`): each `#$ error[CODE]: text` or `#$ warning[CODE]: text` claims one diagnostic
with that code whose text (message, labels, notes or helps) contains `text`; one that trails a
line of code also needs the diagnostic's primary span on that line. A missing diagnostic and an
unclaimed one both fail. Also `#$ help:`, `#$ not-help:`, and for run tests `#$ stdout:` (with its
`#$` continuation lines), `#$ stdin:` (one input line each; stdin is otherwise empty) and
`#$ panics:`. A test uses its first listed profile.
python tasks/impl-0.9.9/annotations.py tests/conformance/DIA-12 [more directories]
"""
import glob
import json
import os
import re
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
EMBER = os.environ.get('EMBER') or os.path.join(ROOT, 'target', 'debug', 'ember.exe')


def expected_stdin(text):
    """`#$ stdin:` lines, each one line of standard input, or None."""
    lines = [line[len('#$ stdin:'):].strip() for line in text.splitlines() if line.startswith('#$ stdin:')]
    return ''.join(line + '\n' for line in lines) if lines else None


def expected_stdout(text):
    """Every `#$ stdout:` line in order, each followed by its `#$` continuations; an empty
    `#$ stdout:` only opens the block."""
    lines = text.split('\n')
    out = None
    for index, line in enumerate(lines):
        if line.startswith('#$ stdout:'):
            first = line[len('#$ stdout:'):].strip()
            out = (out or []) + ([first] if first else [])
            for more in lines[index + 1:]:
                if more == '#$':
                    out.append('')
                elif more.startswith('#$ ') and not re.match(r'#\$ [a-z-]+:', more):
                    # The harness trims every annotation line.
                    out.append(more[3:].strip())
                else:
                    break
    return out


def run(args, path, stdin=None):
    result = subprocess.run([EMBER, *args, path], cwd=ROOT, capture_output=True, text=True,
                            input=stdin if stdin is not None else '',
                            encoding='utf-8', errors='replace')
    return result.returncode, result.stdout.replace('\r\n', '\n'), result.stderr


def annotations(text):
    """(kind, code, message, line or None) for each `error[...]`/`warning[...]` annotation."""
    out = []
    for number, line in enumerate(text.split('\n'), 1):
        stripped = line.lstrip()
        if stripped.startswith('#$'):
            rest, trailing = stripped[2:], None
        else:
            at = line.find('#$')
            if at <= 0 or not line[:at][-1:].isspace():
                continue
            rest, trailing = line[at + 2:], number
        match = re.match(r'\s*(error|warning)\[(\w+)\](?::(.*))?$', rest)
        if match:
            out.append((match[1], match[2], (match[3] or '').strip(), trailing))
    return out


def produced(path, profile):
    code, out, err = run(['check', '--json', '--profile', profile], path)
    found = []
    for line in (out + '\n' + err).split('\n'):
        line = line.strip()
        if not line.startswith('{'):
            continue
        try:
            value = json.loads(line)
        except ValueError:
            continue
        if not value.get('code'):
            continue
        text = value.get('message') or ''
        primary = None
        for label in value.get('labels') or []:
            text += '\n' + (label.get('message') or '')
            if label.get('primary') and primary is None:
                primary = label['span']['line_start']
        for extra in ('notes', 'helps'):
            text += ''.join('\n' + (item or '') for item in (value.get(extra) or []))
        found.append({'code': value['code'], 'text': text, 'line': primary,
                      'rendered': value.get('rendered', '')})
    return code, found


def check(path):
    text = open(path, encoding='utf-8').read()
    kind = (re.search(r'^#\$ test:\s*(\S+)', text, re.M) or [None, ''])[1]
    if kind in ('parse-pass', 'parse-fail'):
        return []
    profile = (re.search(r'^#\$ profiles:\s*([a-z]+)', text, re.M) or [None, 'debug'])[1]
    problems = []
    exit_code, found = produced(path, profile)
    claimed = [False] * len(found)
    for _, code, message, line in annotations(text):
        index = next((i for i, d in enumerate(found)
                      if not claimed[i] and d['code'] == code and message in d['text']
                      and (line is None or d['line'] == line)), None)
        if index is None:
            problems.append(f'missing {code}: {message!r}' + (f' on line {line}' if line else ''))
        else:
            claimed[index] = True
    for d, taken in zip(found, claimed):
        if not taken:
            problems.append(f'unexpected {d["code"]} on line {d["line"]}: {d["text"].splitlines()[0]}')
    rendered = '\n'.join(d['rendered'] for d in found)
    help_lines = [line for line in rendered.split('\n') if 'help:' in line]
    helps = [h.strip() for h in re.findall(r'^\s*#\$ help:(.*)$', text, re.M)]
    absent = [h.strip() for h in re.findall(r'^\s*#\$ not-help:(.*)$', text, re.M)]
    problems += [f'missing help: {h}' for h in helps if not any(h in line for line in help_lines)]
    problems += [f'unwanted help: {h}' for h in absent if any(h in line for line in help_lines)]
    if kind == 'compile-fail' and exit_code == 0:
        problems.append('expected compilation to fail')
    if kind in ('run-pass', 'run-fail'):
        code, out, err = run(['run', '--profile', profile], path, expected_stdin(text))
        stdout = expected_stdout(text)
        if stdout is not None and out.rstrip('\n').split('\n') != stdout:
            problems.append(f'stdout {out.rstrip()!r} is not {stdout!r}')
        panics = re.search(r'^#\$ panics:\s*(.*)$', text, re.M)
        if kind == 'run-fail' and (code == 0 or (panics and panics[1].strip() not in err)):
            problems.append(f'expected a panic {panics[1].strip() if panics else ""!r}, got exit {code}: {err.strip()[:160]!r}')
        if kind == 'run-pass' and code != 0:
            problems.append(f'exit {code}: {err.strip()[:200]!r}')
    return problems


failures = 0
for directory in sys.argv[1:]:
    paths = sorted(glob.glob(os.path.join(directory, '*.em')))
    # `[TST-4a]` — the harness stops at a conformance directory with no accept case.
    if 'conformance' in os.path.normpath(directory).split(os.sep) \
            and not any(os.path.basename(p).startswith('accept_') for p in paths):
        failures += 1
        print(os.path.relpath(directory, ROOT))
        print('    no accept_* case ([TST-4a])')
    for path in paths:
        problems = check(path)
        if problems:
            failures += 1
            print(os.path.relpath(path, ROOT))
            for problem in problems:
                print('   ', problem)
print('failing', failures)
