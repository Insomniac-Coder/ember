"""Runs the pilot's proposed tests on TODAY's compiler (target/release/ember.exe), one at a time, and says
for each whether today's compiler does what the test expects. The tests state the outcome after the
proposed edits, so a test the edit changes (a new refusal) is expected to differ today; the summary
splits the differences by kind so that a wrong program (a parse or name error) shows apart from a gap.

Usage: python -X utf8 deep/v3/run_tests_today.py deep/v3/results/pilot-tests
Copies each test into target/v3-pilot-run/<cluster>/ (never %TEMP%), runs `ember check`, and for a
run-pass test that checks clean, `ember run` with a time limit (a hung build or program is killed with
its whole process tree). Writes deep/v3/results/<dir>-today.json and prints the summary.
"""
import json
import os
import re
import shutil
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.normpath(os.path.join(HERE, '..', '..', '..', '..', '..', '..'))
EMBER = os.path.join(REPO, 'target', 'release', 'ember.exe')
WORK = os.path.join(REPO, 'target', 'v3-pilot-run')
CODE = re.compile(r'\bE\d{4}\b')
LIMIT = 180


def run(args, cwd):
    p = subprocess.Popen(args, cwd=cwd, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    try:
        out, _ = p.communicate(timeout=LIMIT)
        return p.returncode, out.decode('utf-8', 'replace')
    except subprocess.TimeoutExpired:
        subprocess.run(['taskkill', '/T', '/F', '/PID', str(p.pid)], capture_output=True)
        out, _ = p.communicate()
        return None, out.decode('utf-8', 'replace')


def headers(text):
    h = {'test': None, 'error': None, 'note': None, 'stdout': []}
    for line in text.split('\n'):
        if not line.startswith('#$'):
            continue
        body = line[2:].strip()
        if body.startswith('test:'):
            h['test'] = body[5:].strip()
        elif body.startswith('error['):
            h['error'] = body[6:body.index(']')]
        elif body.startswith('note:'):
            h['note'] = body[5:].strip()
        elif body.startswith('stdout:'):
            h['stdout'].append(body[7:].strip())
    return h


def classify(h, check_rc, check_out, run_rc, run_out):
    codes = sorted(set(CODE.findall(check_out))) if check_rc else []
    if check_rc is None:
        return 'hung (check)', codes
    if 'E0900' in codes:
        return 'not built yet (E0900)', codes
    if h['test'] == 'compile-fail':
        if not check_rc:
            return 'accepted today (the test expects a refusal)', codes
        if h['error'] and h['error'] in codes:
            return 'agrees', codes
        if not h['error']:
            return 'refused today (the test names no code)', codes
        return 'refused today with another code', codes
    if check_rc:
        return 'refused today (the test expects it to compile)', codes
    if h['test'] == 'compile-pass':
        return 'agrees', codes
    if run_rc is None:
        return 'hung (build or run)', codes
    got = [l.strip() for l in run_out.replace('\r', '').split('\n') if l.strip()]
    lines = [l for l in got if not l.startswith('note: created')]
    if run_rc != 0:
        return 'stopped while running (exit %s)' % run_rc, codes
    if lines == h['stdout']:
        return 'agrees', codes
    return 'runs, other output', codes


def main():
    src = os.path.abspath(sys.argv[1])
    results = []
    for cid in sorted(os.listdir(src)):
        cdir = os.path.join(src, cid)
        wdir = os.path.join(WORK, cid)
        os.makedirs(wdir, exist_ok=True)
        for name in sorted(os.listdir(cdir)):
            if not name.endswith('.em'):
                continue
            text = open(os.path.join(cdir, name), encoding='utf-8').read()
            h = headers(text)
            shutil.copy(os.path.join(cdir, name), os.path.join(wdir, name))
            check_rc, check_out = run([EMBER, 'check', name], wdir)
            run_rc = run_out = None
            if h['test'] == 'run-pass' and check_rc == 0:
                run_rc, run_out = run([EMBER, 'run', name], wdir)
            verdict, codes = classify(h, check_rc, check_out, run_rc, run_out or '')
            results.append({'cluster': cid, 'test': name, 'kind': h['test'], 'expected_code': h['error'],
                            'expected_stdout': h['stdout'], 'verdict': verdict, 'codes_today': codes,
                            'check_output': check_out[-1500:], 'run_output': (run_out or '')[-1500:]})
            print(f"{verdict:48} {cid[:3]} {name}", flush=True)
    out = src.rstrip('\\/') + '-today.json'
    json.dump(results, open(out, 'w', encoding='utf-8'), ensure_ascii=False, indent=1)
    print('\nSUMMARY')
    by = {}
    for r in results:
        by.setdefault(r['verdict'], 0)
        by[r['verdict']] += 1
    for v, n in sorted(by.items(), key=lambda x: -x[1]):
        print(f'{n:4}  {v}')
    print('wrote', out)


if __name__ == '__main__':
    main()
