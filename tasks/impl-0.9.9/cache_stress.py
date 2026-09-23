# D-189 regression check: wipe the interface cache, then check every tests/run-pass file
# from 8 workers (staggered starts, shared std modules); repeat. Counts ICEs.
import glob, os, shutil, subprocess
from concurrent.futures import ThreadPoolExecutor
root = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
exe = os.environ.get('EMBER') or os.path.join(root, 'target', 'debug', 'ember.exe')
files = sorted(glob.glob(os.path.join(root, 'tests', 'run-pass', '*.em')))
ice = 0
for round in range(int(os.environ.get('ROUNDS', '4'))):
    shutil.rmtree(os.path.join(root, 'target', 'debug', 'interface'), ignore_errors=True)
    def run(path):
        r = subprocess.run([exe, 'check', os.path.relpath(path, root)], cwd=root, capture_output=True, text=True, errors='replace')
        return 'internal compiler error' in (r.stdout + r.stderr)
    with ThreadPoolExecutor(8) as p:
        ice += sum(p.map(run, files))
print('files', len(files), 'ICEs:', ice)
