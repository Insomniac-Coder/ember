"""Apply A2-3's checker migrations ([BRW-12], [BRW-15], [OWN-7]) to the files given, and only
those: each round runs `ember check --json` and applies the edits of the migration suggestions this
job's diagnostics carry, until none is left. Other migrations stay for their own milestones."""
import json
import os
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor

EMBER = os.environ['EMBER']
MINE = (
    "`ref mut` before a part a name changes",
    "`ref mut` before a `MutSpan` a name writes through",
    "`copy of` where 0.9.10's `=` copied",
    "`copy of` before a part a name changes",
)
# Other migrations of the lane are named on the command line, after the list of files.
MINE = tuple(sys.argv[2:]) or MINE


def edits_for(path):
    run = subprocess.run([EMBER, 'check', path, '--json'], capture_output=True, text=True, encoding='utf-8', errors='replace')
    found = set()
    for line in run.stdout.splitlines():
        line = line.strip()
        if not line.startswith('{'):
            continue
        try:
            diagnostic = json.loads(line)
        except ValueError:
            continue
        for suggestion in diagnostic.get('suggestions') or []:
            if not suggestion.get('migration') or not suggestion['message'].startswith(MINE):
                continue
            for edit in suggestion['edits']:
                span = edit['span']
                if os.path.normcase(os.path.abspath(span['file'])) != os.path.normcase(os.path.abspath(path)):
                    continue
                found.add((span['byte_start'], span['byte_end'], edit['replacement']))
    return found


def migrate(path):
    changed = 0
    for _ in range(6):
        edits = edits_for(path)
        if not edits:
            break
        data = open(path, 'rb').read()
        for start, end, text in sorted(edits, reverse=True):
            data = data[:start] + text.encode('utf-8') + data[end:]
        open(path, 'wb').write(data)
        changed += len(edits)
    return path, changed


if __name__ == '__main__':
    files = [line.strip() for line in open(sys.argv[1], encoding='utf-8') if line.strip()]
    with ThreadPoolExecutor(max_workers=int(os.environ.get('RUST_TEST_THREADS') or 4)) as pool:
        for path, changed in pool.map(migrate, files):
            if changed:
                print(f'{changed:3} {path}')
