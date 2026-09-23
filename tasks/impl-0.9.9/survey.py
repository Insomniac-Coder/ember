"""Survey the test corpus against the current compiler, fast (`ember check` only, no C).

Lists every file that should compile but is rejected, and every compile-fail file whose
expected codes no longer appear. Usage: python tasks/survey.py [substring-filter]
"""
import os, re, subprocess, sys
from concurrent.futures import ThreadPoolExecutor

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
EMBER = os.environ.get('EMBER') or os.path.join(ROOT, 'target', 'debug', 'ember.exe')
FILTER = sys.argv[1] if len(sys.argv) > 1 else ''


def files():
    for top in ('run-pass', 'run-fail', 'compile-pass', 'compile-fail', 'conformance', 'milestones'):
        for dirpath, _, names in os.walk(os.path.join(ROOT, 'tests', top)):
            for n in names:
                if n.endswith('.em') and '_support' not in dirpath:
                    p = os.path.join(dirpath, n)
                    if FILTER in p:
                        yield p


def survey(path):
    text = open(path, encoding='utf-8').read()
    kind = (re.search(r'^#\$ test:\s*(\S+)', text, re.M) or [None, ''])[1]
    if kind in ('parse-fail', 'parse-pass'):
        return None
    # Leading (`#$ error[...]`) and trailing (`code  #$ error[...]`) forms.
    codes = re.findall(r'#\$ error\[(\w+)\]', text)
    rel = os.path.relpath(path, ROOT)
    r = subprocess.run([EMBER, 'check', rel], cwd=ROOT, capture_output=True, text=True,
                       encoding='utf-8', errors='replace')
    out = r.stdout + r.stderr
    errs = re.findall(r'^error\[(\w+)\]: (.*)$', out, re.M)
    if kind == 'compile-fail' or codes:
        missing = [c for c in codes if c not in out]
        if r.returncode == 0 or missing:
            return (rel, 'compile-fail lost ' + ','.join(missing or ['rejection']), errs[:3])
        return None
    if r.returncode != 0:
        # No `error[...]` line means the compiler did not report: show why.
        detail = '' if errs else f' (exit {r.returncode}: {out.strip()[:200]!r})'
        return (rel, 'rejected' + detail, errs[:3])
    return None


def main():
    paths = sorted(files())
    with ThreadPoolExecutor(max_workers=8) as pool:
        results = [x for x in pool.map(survey, paths) if x]
    by_code = {}
    for rel, what, errs in results:
        print(f'{rel}: {what}')
        for code, msg in errs:
            print(f'    {code}: {msg[:110]}')
            by_code[code] = by_code.get(code, 0) + 1
    print(f'\nfiles {len(paths)}, failing {len(results)}')
    print('first-error codes:', sorted(by_code.items(), key=lambda kv: -kv[1]))


if __name__ == '__main__':
    main()
