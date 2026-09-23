#!/usr/bin/env python3
"""Re-run the audit reproducers in tasks/audit/repro and summarise each one.

For every program: `ember check`; if it is accepted, run it in the debug,
release and shipping profiles, then build the debug C with gcc
AddressSanitizer/UndefinedBehaviorSanitizer when gcc is available. The
summary line is what a fix should change; the header of each file states
the expected behaviour.

    python tasks/audit/run_repros.py [substring-filter]
"""

import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
REPRO = ROOT / "tasks" / "audit" / "repro"
EMBER = ROOT / "target" / "debug" / ("ember.exe" if os.name == "nt" else "ember")
INCLUDE = ROOT / "runtime" / "ember_rt" / "include"
RUNTIME = ROOT / "runtime" / "ember_rt" / "src" / "ember_rt.c"
TIMEOUT = 20


def one_line(text, limit=160):
    text = " | ".join(line.strip() for line in text.strip().splitlines() if line.strip())
    return text if len(text) <= limit else text[: limit - 3] + "..."


def run(cmd, cwd):
    try:
        done = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, timeout=TIMEOUT)
        return done.returncode, done.stdout, done.stderr
    except subprocess.TimeoutExpired:
        return None, "", "TIMEOUT"


def main():
    if not EMBER.exists():
        sys.exit(f"build the compiler first: {EMBER} is missing")
    wanted = sys.argv[1] if len(sys.argv) > 1 else ""
    gcc = shutil.which("gcc")
    work = Path(tempfile.mkdtemp(prefix="ember-repro-"))
    for path in sorted(REPRO.glob("*.em")):
        if wanted not in path.name:
            continue
        local = work / path.name
        shutil.copy(path, local)
        code, _, err = run([str(EMBER), "check", local.name], work)
        if code != 0:
            first = next((l for l in err.splitlines() if l.startswith(("error", "warning"))), err)
            print(f"{path.name}: rejected ({one_line(first, 110)})" if code is not None
                  else f"{path.name}: check TIMEOUT")
            continue
        parts = []
        for profile in ("debug", "release", "shipping"):
            code, out, err = run([str(EMBER), "run", local.name, "--profile", profile,
                                  "--out-dir", str(work / "out")], work)
            status = "TIMEOUT" if code is None else f"exit={code}"
            parts.append(f"{profile}:{status}:{one_line(out, 40)}")
        if gcc:
            code, csrc, _ = run([str(EMBER), "build", local.name, "--emit", "c"], work)
            if code == 0:
                cfile = work / (local.stem + ".c")
                cfile.write_text(csrc, encoding="utf-8")
                exe = work / (local.stem + ".asan")
                built = subprocess.run(
                    [gcc, "-std=c11", "-O0", "-g", "-fsanitize=address,undefined",
                     "-I", str(INCLUDE), str(cfile), str(RUNTIME), "-o", str(exe), "-lm"],
                    capture_output=True, text=True)
                if built.returncode == 0:
                    _, _, err = run([str(exe)], work)
                    hits = [l for l in err.splitlines()
                            if "ERROR: AddressSanitizer" in l or "runtime error:" in l
                            or "SUMMARY: AddressSanitizer" in l]
                    parts.append("asan:" + (one_line(hits[0], 90) if hits else "clean"))
        print(f"{path.name}: accepted; " + "; ".join(parts))
    shutil.rmtree(work, ignore_errors=True)


if __name__ == "__main__":
    main()
