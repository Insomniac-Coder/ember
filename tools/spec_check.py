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

`[TST-29]`: the baseline only shrinks (`--write-baseline` refuses to add a block
unless `--allow-growth` is given), and the result names its size and whether the
gate is `green` or `baselined`.
"""

import argparse
import hashlib
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

# `[TST-7]` names four: a `std` signature sketch, foreign-language source, a
# deliberate error example, and overlay-language source until Part III defines
# `overlay_decl`.
PERMITTED_IGNORE_REASONS = ("signature sketch", "foreign", "deliberate error", "overlay")


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
            # Keyed by content, not by line: an edit elsewhere in the
            # document shifts every line number and would invalidate the
            # whole baseline, which is exactly when a gate must not cry wolf.
            source = "\n".join(body) + "\n"
            digest = hashlib.blake2s(source.encode("utf-8"), digest_size=6).hexdigest()
            out.append(
                {
                    "id": f"{part}:{digest}",
                    "line": start + 1,
                    "source": source,
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
# The development target's Annex A, in the part every `Hardened_N` is assembled from.
ANNEX_A = ROOT / "tasks" / "spec-0.9.9" / "parts" / "p19-annex-a.md"


def appendix_block(fixture):
    """`[TST-6]` — the lines of the fixture the quick reference prints: not the harness's `#$`
    lines, and not the declarations between `#$ begin fixture` and `#$ end fixture`, which give the
    printed fragment's names something to name so that the fixture compiles."""
    body, hidden = [], False
    for line in fixture.split(NEWLINE):
        marker = line.strip()
        if marker == "#$ begin fixture":
            hidden = True
        elif marker == "#$ end fixture":
            hidden = False
        elif not hidden and not line.lstrip().startswith("#$"):
            body.append(line)
    while body and not body[0].strip():
        body.pop(0)
    while body and not body[-1].strip():
        body.pop()
    return body


def emit_appendix():
    """`[TST-6]` — rewrite Annex A's code block from the fixture.

    The fixture is the source of truth: it is the file that is compiled, so the
    quick reference cannot drift away from something that compiles. The next
    `Hardened_N` cut assembles the part into the development target.
    """
    body = appendix_block(APPENDIX_FIXTURE.read_text(encoding="utf-8"))
    part = ANNEX_A.read_text(encoding="utf-8")
    lines = part.split(NEWLINE)
    start = next(n for n, l in enumerate(lines) if l.startswith("# Annex A"))
    open_at = next(n for n in range(start, len(lines)) if lines[n].startswith("```ember"))
    close_at = next(n for n in range(open_at + 1, len(lines)) if lines[n].startswith("```"))

    new = NEWLINE.join(lines[: open_at + 1] + body + lines[close_at:])
    if new == part:
        print("Annex A already matches the fixture")
        return 0
    ANNEX_A.write_text(new, encoding="utf-8", newline=NEWLINE)
    print(f"Annex A rewritten from {APPENDIX_FIXTURE.name} ({len(body)} lines)")
    return 0


def gate_state(cases, baselined):
    """`[TST-29]` — a gate whose baseline covers more than a tenth of its cases is `baselined`, not green."""
    return "baselined" if baselined * 10 > cases else "green"


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--write-baseline", action="store_true")
    ap.add_argument(
        "--allow-growth",
        action="store_true",
        help="permit --write-baseline to record a block the baseline does not "
        "already hold. `[TST-29]`: the baseline shrinks and never grows.",
    )
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
        grown = sorted(i for i, _ in failing if i not in known)
        if grown and not args.allow_growth:
            print("`[TST-29]`: the baseline may shrink and never grow.")
            for ident in grown:
                print(f"  would add: {ident}")
            return 1
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

    checked = sum(1 for block in all_blocks if not block["ignored"])
    print(f"no new failures; baseline {len(known)} of {checked} ({gate_state(checked, len(known))}) [TST-29]")
    return 0


if __name__ == "__main__":
    sys.exit(main())
