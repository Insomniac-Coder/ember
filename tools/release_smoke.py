#!/usr/bin/env python3
"""`[TOOL-1]`, `[TOOL-2]`, `[TOOL-4]` — for the release workflow: an archive works on a clean machine.

    python tools/release_smoke.py <archive>

Unpacks the archive into a new directory, then, with only its `bin` and the system's own
directories on `PATH`, times the first hour's commands: `ember --version` names the archive's own
Clang as the C compiler (`[TOOL-2]`: the bundled toolchain is found before anything else), and
`ember new hello`, `cd hello`, `ember run` print `hello, world` (`[TOOL-4]`: under five minutes,
at most six commands, installing included). Exit 1 when one does not.
"""

import os
import shutil
import subprocess
import sys
import tarfile
import tempfile
import time
import zipfile
from pathlib import Path


def main():
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    archive = Path(sys.argv[1]).resolve()
    started = time.monotonic()
    home = Path(tempfile.mkdtemp(prefix="ember-smoke-"))
    try:
        if archive.suffix == ".zip":
            with zipfile.ZipFile(archive) as zip_file:
                zip_file.extractall(home)
        else:
            with tarfile.open(archive) as tar:
                tar.extractall(home, filter="data")
        root = home / "ember"
        bin_dir = root / "bin"
        for path in bin_dir.iterdir():
            path.chmod(0o755)
        system = [r"C:\Windows\System32", r"C:\Windows"] if os.name == "nt" else ["/usr/bin", "/bin"]
        env = dict(os.environ, PATH=os.pathsep.join([str(bin_dir), *system]))
        for name in ("EMBER_STD", "EMBER_RUNTIME_DIR", "EMBER_CC"):
            env.pop(name, None)
        ember = str(bin_dir / ("ember.exe" if os.name == "nt" else "ember"))

        version = subprocess.run([ember, "--version"], env=env, capture_output=True, text=True, timeout=120)
        print(version.stdout)
        bundled = str(root / "toolchain" / "llvm" / "bin")
        if version.returncode != 0 or bundled.lower() not in version.stdout.replace("/", os.sep).lower():
            print(f"`ember --version` does not name the archive's Clang ({bundled}):\n{version.stdout}{version.stderr}")
            return 1

        work = home / "work"
        work.mkdir()
        made = subprocess.run([ember, "new", "hello"], cwd=work, env=env, capture_output=True, text=True, timeout=120)
        ran = subprocess.run([ember, "run"], cwd=work / "hello", env=env, capture_output=True, text=True, timeout=300)
        elapsed = time.monotonic() - started
        print(f"ember new hello: {made.returncode}; ember run: {ran.returncode}; {elapsed:.1f} s from unpacking")
        if made.returncode != 0 or ran.returncode != 0 or "hello, world" not in ran.stdout:
            print(made.stdout, made.stderr, ran.stdout, ran.stderr)
            return 1
        if elapsed > 300:
            print(f"the first hour took {elapsed:.0f} s, over five minutes [TOOL-4]")
            return 1
        return 0
    finally:
        shutil.rmtree(home, ignore_errors=True)


if __name__ == "__main__":
    sys.exit(main())
