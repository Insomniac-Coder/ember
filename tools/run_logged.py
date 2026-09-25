"""Run a command, show its output as it comes, keep a copy in a file, and exit with the
command's status:

    python tools/run_logged.py test-output.txt cargo test --workspace --locked

CI's Test step uses this rather than `bash` and `tee`: on Windows, Git's bash puts its
own `link` ahead of the MSVC linker on PATH, and Rust then cannot link the tests.
"""
import subprocess
import sys

with open(sys.argv[1], 'w', encoding='utf-8') as log:
    process = subprocess.Popen(sys.argv[2:], stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    for line in process.stdout:
        sys.stdout.buffer.write(line)
        sys.stdout.flush()
        log.write(line.decode('utf-8', errors='replace'))
    sys.exit(process.wait())
