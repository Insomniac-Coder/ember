"""Compile every fenced `ember` block in the specification (`[TST-7]`).

Part XIX §5 requires that every ```ember block in Parts I–XVII and Appendix A
passes `ember check --syntax-only`. A block that does not parse fails CI. A
block may opt out with ```ember,ignore and must then carry a one-line reason on
the preceding line; the permitted reasons are a `std` signature sketch, foreign
source, or a deliberate error example.

Many blocks in the document are fragments — a few statements without an
enclosing function, or a signature sketch — and cannot parse as a compilation
unit. `[TST-7]` therefore ships with a **recorded baseline** and fails only on
*new* failures, until the grammar gaps it exists to expose are closed.

    python tools/spec_check.py                  # check, non-zero on new failures
    python tools/spec_check.py --write-baseline
    python tools/spec_check.py --report         # list every failing block
"""

import argparse
import json
import re
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SPEC = ROOT / "docs" / "spec-source" / "ember-spec.md"
BASELINE = ROOT / "tools" / "spec_check_baseline.json"
NEWLINE = chr(10)

# Parts I-XVII and Appendix A. Part XVIII onward describes the compiler, whose
# examples are Rust, C and shell.
START = re.compile(r"^# Part I\b")
STOP = re.compile(r"^# Part XVIII\b")
APPENDIX = re.compile(r"^# Appendix A\b")

PERMITTED_IGNORE_REASONS = ("signature sketch", "foreign", "deliberate error")


def blocks():
    """(identifier, first line number, source, ignored, reason) per block."""
    lines = SPEC.read_text(encoding="utf-8").split("\n")
    inside_range = False
    part = "?"
    out = []
    n = 0
    while n < len(lines):
        line = lines[n]
        if START.match(line) or APPENDIX.match(line):
            inside_range = True
        if STOP.match(line):
            inside_range = False
        if line.startswith("# Part ") or line.startswith("# Appendix "):
            part = line[2:].split("—")[0].strip()
        if inside_range and line.startswith("```ember"):
            ignored = line.strip() == "```ember,ignore"
            reason = lines[n - 1].strip() if n > 0 else ""
            body, start = [], n + 1
            n += 1
            while n < len(lines) and not lines[n].startswith("```"):
                body.append(lines[n])
                n += 1
            out.append(
                {
                    "id": f"{part}:{start + 1}",
                    "line": start + 1,
                    "source": "\n".join(body) + "\n",
                    "ignored": ignored,
                    "reason": reason,
                }
            )
        n += 1
    return out


def ember_binary():
    for candidate in (
        ROOT / "target" / "debug" / "ember.exe",
        ROOT / "target" / "debug" / "ember",
        ROOT / "target" / "release" / "ember.exe",
        ROOT / "target" / "release" / "ember",
    ):
        if candidate.exists():
            return candidate
    return None


def check_block(binary, block, workdir):
    path = Path(workdir) / "block.em"
    path.write_text(block["source"], encoding="utf-8", newline="\n")
    result = subprocess.run(
        [str(binary), "check", "--syntax-only", str(path)],
        capture_output=True,
        text=True,
    )
    return result.returncode == 0, (result.stdout + result.stderr).strip()


APPENDIX_FIXTURE = ROOT / "docs" / "spec-source" / "appendix-a.em"


def emit_appendix():
    """`[TST-6]` — rewrite Appendix A's code block from the fixture.

    The fixture is the source of truth: it is the file that is checked, so the
    quick reference cannot drift away from something that parses. `#$` lines
    belong to the harness and are stripped on the way in.
    """
    fixture = APPENDIX_FIXTURE.read_text(encoding="utf-8")
    body = [l for l in fixture.split(NEWLINE) if not l.lstrip().startswith("#$")]
    while body and not body[0].strip():
        body.pop(0)
    while body and not body[-1].strip():
        body.pop()

    spec = SPEC.read_text(encoding="utf-8")
    lines = spec.split(NEWLINE)
    start = next(n for n, l in enumerate(lines) if APPENDIX.match(l))
    open_at = next(n for n in range(start, len(lines)) if lines[n].startswith("```ember"))
    close_at = next(n for n in range(open_at + 1, len(lines)) if lines[n].startswith("```"))

    new = NEWLINE.join(lines[: open_at + 1] + body + lines[close_at:])
    if new == spec:
        print("Appendix A already matches the fixture")
        return 0
    SPEC.write_text(new, encoding="utf-8", newline=NEWLINE)
    print(f"Appendix A rewritten from {APPENDIX_FIXTURE.name} ({len(body)} lines)")
    print("run: python tools/split_spec.py docs/spec-source/ember-spec.md docs/spec")
    return 0


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--write-baseline", action="store_true")
    ap.add_argument("--report", action="store_true")
    ap.add_argument("--emit-appendix", action="store_true")
    args = ap.parse_args()

    if args.emit_appendix:
        return emit_appendix()

    binary = ember_binary()
    if binary is None:
        print("ember is not built; run `cargo build` first", file=sys.stderr)
        return 2

    all_blocks = blocks()
    known = json.loads(BASELINE.read_text(encoding="utf-8")) if BASELINE.exists() else []
    known = set(known)

    failing, unexplained_ignores = [], []
    with tempfile.TemporaryDirectory() as workdir:
        for block in all_blocks:
            if block["ignored"]:
                # `[TST-7]` — an opt-out must say why.
                if not any(r in block["reason"].lower() for r in PERMITTED_IGNORE_REASONS):
                    unexplained_ignores.append(block["id"])
                continue
            ok, output = check_block(binary, block, workdir)
            if not ok:
                failing.append((block["id"], output))

    print(f"{len(all_blocks)} ember blocks in Parts I-XVII and Appendix A")
    print(f"{len(all_blocks) - len(failing)} parse, {len(failing)} do not")

    if args.write_baseline:
        BASELINE.write_text(
            json.dumps(sorted(i for i, _ in failing), indent=2) + "\n",
            encoding="utf-8",
            newline="\n",
        )
        print(f"baseline written: {len(failing)} blocks")
        return 0

    if args.report:
        for ident, output in failing:
            first = next((l for l in output.splitlines() if l.startswith("error")), "")
            print(f"  {ident}: {first}")
        return 0

    new = [(i, o) for i, o in failing if i not in known]
    for ident in unexplained_ignores:
        print(f"  {ident}: `,ignore` with no permitted reason on the line above")

    if new or unexplained_ignores:
        print(f"\n{len(new)} newly failing block(s):\n")
        for ident, output in new:
            first = next((l for l in output.splitlines() if l.startswith("error")), "")
            print(f"  {ident}: {first}")
        return 1

    print(f"no new failures ({len(known)} known)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
