"""Turn a failed test run's output into one GitHub error annotation.

A job's log can only be read when signed in to GitHub; its annotations can be read
by anyone, through the check-runs API. So CI passes the Test step's output here on a
failure: cargo's `failures:` block when there is one (a test that failed), else the end
of the output (a test program that crashed, or a build error).

    python tools/ci_failure_annotation.py test-output.txt
"""
import re
import sys

text = open(sys.argv[1], encoding='utf-8', errors='replace').read()
text = re.sub(r'\x1b\[[0-9;]*m', '', text).replace('\r', '')
lines = text.split('\n')
start = next((i for i, line in enumerate(lines) if line == 'failures:'), None)
chosen = lines[start:] if start is not None else lines[-150:]
body = '\n'.join(chosen)[-60000:] or 'the test output is empty'
print('::error title=test failures::' + body.replace('%', '%25').replace('\n', '%0A'))
