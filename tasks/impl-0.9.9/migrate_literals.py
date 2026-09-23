"""Corpus migration for ODR-022 (0.9.9): annotate literal-initialised locals where a later use needs
another type, by applying the compiler's own `declare it as `x: T = ...`` help. Iterates per file
until the help stops appearing. Usage: python tasks/migrate_literals.py <file.em>...
"""
import json, os, re, subprocess, sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
EMBER = os.path.join(ROOT, 'target', 'debug', 'ember.exe')
HELP = re.compile(r'declare it as `(\w+): (.+) = \.\.\.`')


def fixes(rel):
    r = subprocess.run([EMBER, 'check', rel, '--json'], cwd=ROOT, capture_output=True, text=True,
                       encoding='utf-8', errors='replace')
    out = []
    for line in (r.stdout + r.stderr).splitlines():
        if not line.startswith('{'):
            continue
        d = json.loads(line)
        for h in d.get('helps', []):
            m = HELP.search(h)
            if not m:
                continue
            for label in d['labels']:
                if not label['primary'] and (label['message'] or '').startswith('declared from a literal'):
                    out.append((label['span']['file'], label['span']['byte_start'], m.group(1), m.group(2)))
    return out


def apply(rel):
    changed = 0
    for _ in range(20):
        todo = fixes(rel)
        if not todo:
            break
        done = set()
        for file, start, name, ty in sorted(todo, key=lambda t: -t[1]):
            if (file, start) in done:
                continue
            done.add((file, start))
            path = os.path.join(ROOT, file)
            data = open(path, 'rb').read()
            head = data[start:start + len(name) + 8].decode('utf-8', 'replace')
            m = re.match(re.escape(name) + r'(\s*)=', head)
            if not m:
                continue
            new = f'{name}: {ty} ='.encode()
            data = data[:start] + new + data[start + m.end():]
            open(path, 'wb').write(data)
            changed += 1
    return changed


if __name__ == '__main__':
    for rel in sys.argv[1:]:
        print(rel, apply(rel))
