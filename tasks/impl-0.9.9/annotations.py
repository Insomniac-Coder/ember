"""Check a directory's annotations against the compiler in seconds, rather than the full
conformance run: `#$ error[CODE]: text`, `#$ help:`, `#$ not-help:`, and for run tests
`#$ stdout:` (with its `#$` continuation lines) and `#$ panics:`. A run test uses its first
listed profile. Matching follows the harness in `ember_driver/tests/milestones.rs`: an
error's text may appear anywhere in that diagnostic (message or labels), and a help's text
anywhere in a help line.
python tasks/impl-0.9.9/annotations.py tests/conformance/DIA-12 [more directories]
"""
import glob
import os
import re
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
EMBER = os.environ.get('EMBER') or os.path.join(ROOT, 'target', 'debug', 'ember.exe')


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
                    out.append(more[3:])
                else:
                    break
    return out


def run(args, path):
    result = subprocess.run([EMBER, *args, path], cwd=ROOT, capture_output=True, text=True,
                            encoding='utf-8', errors='replace')
    return result.returncode, result.stdout.replace('\r\n', '\n'), result.stderr


def diagnostics(rendered):
    """(code, text of the whole diagnostic) for each `error[...]`."""
    blocks, current = [], None
    for line in rendered.split('\n'):
        match = re.match(r'^(error|warning)\[(\w+)\]', line)
        if match:
            current = [match[2], line]
            blocks.append(current)
        elif current is not None:
            current[1] += '\n' + line
    return blocks


def check(path):
    text = open(path, encoding='utf-8').read()
    kind = (re.search(r'^#\$ test:\s*(\S+)', text, re.M) or [None, ''])[1]
    profile = (re.search(r'^#\$ profiles:\s*([a-z]+)', text, re.M) or [None, 'debug'])[1]
    problems = []
    if kind in ('run-pass', 'run-fail'):
        code, out, err = run(['run', '--profile', profile], path)
        stdout = expected_stdout(text)
        if stdout is not None and out.rstrip('\n').split('\n') != stdout:
            problems.append(f'stdout {out.rstrip()!r} is not {stdout!r}')
        panics = re.search(r'^#\$ panics:\s*(.*)$', text, re.M)
        if kind == 'run-fail' and (code == 0 or (panics and panics[1].strip() not in err)):
            problems.append(f'expected a panic {panics[1].strip() if panics else ""!r}, got exit {code}: {err.strip()[:160]!r}')
        if kind == 'run-pass' and code != 0:
            problems.append(f'exit {code}: {err.strip()[:200]!r}')
        return problems
    code, out, err = run(['check'], path)
    out = out + err
    help_lines = [line for line in out.split('\n') if 'help:' in line]
    helps = [h.strip() for h in re.findall(r'^\s*#\$ help:(.*)$', text, re.M)]
    absent = [h.strip() for h in re.findall(r'^\s*#\$ not-help:(.*)$', text, re.M)]
    errors = re.findall(r'#\$ error\[(\w+)\](?::\s*(.*))?$', text, re.M)
    problems += [f'missing help: {h}' for h in helps if not any(h in line for line in help_lines)]
    problems += [f'unwanted help: {h}' for h in absent if any(h in line for line in help_lines)]
    found = diagnostics(out)
    problems += [f'missing error[{c}]: {m}' for c, m in errors
                 if not any(code == c and m.strip() in block for code, block in found)]
    return problems


failures = 0
for directory in sys.argv[1:]:
    for path in sorted(glob.glob(os.path.join(directory, '*.em'))):
        problems = check(path)
        if problems:
            failures += 1
            print(os.path.relpath(path, ROOT))
            for problem in problems:
                print('   ', problem)
print('failing', failures)
