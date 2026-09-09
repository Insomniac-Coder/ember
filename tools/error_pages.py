"""Error reference pages, checked (`[DOC-1]`, `[DIA-6]`, `[PHIL-8a]`).

`[DOC-1]`: "Each phase's exit criteria include `docs/errors/EXXXX.md` for every
code that phase introduces. A page contains: a minimal program that triggers
the error; the rendered diagnostic; one paragraph on **why the rule exists**
(not a restatement of the rule); and the fix as compilable code. Every fenced
`ember` block under `docs/errors/` is built by `ember test --doc`: the failing
example MUST fail with that exact code and the fixed example MUST compile."

That last sentence is the one that cannot be faked, and it is `[PHIL-8a]` in
another guise: a page whose "fix" does not compile is a page that tells the
reader something untrue, and the only way to know is to run both halves.

A page marks its two programs with the fence's info string:

    ```ember,fails      the program that must be rejected with this code
    ```ember,fixed      the program that must compile

Any other `ember` block on the page is prose and is not run.

    python tools/error_pages.py                  # check
    python tools/error_pages.py --report         # list what is covered
    python tools/error_pages.py --write-baseline # record the codes with no page

`[DIA-6]`'s "a page for every code" is ratcheted the same way `[TST-4c]`'s
baseline is: the list of codes without pages may shrink and never grow.
"""

import argparse
import json
import re
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
PAGES = ROOT / "docs" / "errors"
REGISTRY_RS = ROOT / "compiler" / "ember_diag" / "src" / "codes.rs"
BASELINE = ROOT / "tools" / "error_pages_baseline.json"

# A fenced block and its info string.
FENCE = re.compile(r"^```ember,(fails|fixed)\s*$(.*?)^```\s*$", re.M | re.S)
# Codes the compiler can actually emit. A page is required for those; a code
# that is registered and never emitted (`E4071`, `E9032` — both reserved so the
# number is not reused) cannot have a failing example, and `[DOC-1]` asks for
# one.
EMITTED = re.compile(r"codes::([EWL]\d{4})")
REGISTRY_ENTRY = re.compile(r"^\s*([EWL]\d{4})\s*=\s*\(", re.M)


def emitted_codes() -> set[str]:
    found = set()
    for path in (ROOT / "compiler").rglob("*.rs"):
        found.update(EMITTED.findall(path.read_text(encoding="utf-8")))
    return found


def registered_codes() -> set[str]:
    return set(REGISTRY_ENTRY.findall(REGISTRY_RS.read_text(encoding="utf-8")))


def load_baseline() -> set[str]:
    if not BASELINE.exists():
        return set()
    return set(json.loads(BASELINE.read_text(encoding="utf-8")))


def ember() -> Path:
    for profile in ("release", "debug"):
        exe = ROOT / "target" / profile / "ember.exe"
        if exe.exists():
            return exe
        exe = ROOT / "target" / profile / "ember"
        if exe.exists():
            return exe
    sys.exit("build the compiler first: cargo build")


def run(source: str, work: Path) -> tuple[int, str]:
    path = work / "page.em"
    path.write_text(source, encoding="utf-8", newline="\n")
    result = subprocess.run(
        [str(ember()), "check", str(path)],
        capture_output=True,
        text=True,
        cwd=ROOT,
    )
    return result.returncode, (result.stdout + result.stderr).replace("\r\n", "\n")


def check_page(path: Path, work: Path) -> list[str]:
    code = path.stem
    text = path.read_text(encoding="utf-8")
    blocks = {kind: body for kind, body in FENCE.findall(text)}
    problems = []

    if "fails" not in blocks:
        problems.append(f"{code}: no ```ember,fails block — [DOC-1] wants the program that triggers it")
    else:
        status, output = run(blocks["fails"], work)
        if status == 0:
            problems.append(f"{code}: the ```ember,fails program compiled")
        elif code not in output:
            problems.append(
                f"{code}: the ```ember,fails program failed with something else:\n"
                + "\n".join("      " + line for line in output.splitlines()[:6])
            )

    if "fixed" not in blocks:
        problems.append(f"{code}: no ```ember,fixed block — [DOC-1] wants the fix as compilable code")
    else:
        status, output = run(blocks["fixed"], work)
        if status != 0:
            problems.append(
                f"{code}: the ```ember,fixed program does not compile — [PHIL-8a]:\n"
                + "\n".join("      " + line for line in output.splitlines()[:8])
            )

    # `[DOC-1]` — "one paragraph on **why the rule exists** (not a restatement
    # of the rule)". A heading is the cheapest thing that can be checked; the
    # paragraph under it is a person's job.
    if "## Why" not in text:
        problems.append(f"{code}: no `## Why` section — [DOC-1] requires one")
    return problems


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--report", action="store_true")
    ap.add_argument("--write-baseline", action="store_true")
    ap.add_argument(
        "--allow-growth",
        action="store_true",
        help="permit --write-baseline to record a code that has no page and was "
        "not already in the baseline. The list shrinks and never grows.",
    )
    args = ap.parse_args()

    PAGES.mkdir(parents=True, exist_ok=True)
    pages = sorted(p for p in PAGES.glob("*.md") if re.fullmatch(r"[EWL]\d{4}", p.stem))
    have = {p.stem for p in pages}
    emitted = emitted_codes()
    registered = registered_codes()
    missing = sorted(c for c in emitted & registered if c not in have)
    known = load_baseline()

    if args.report:
        print(f"pages: {len(pages)}   emitted codes: {len(emitted)}   without a page: {len(missing)}")
        for code in missing:
            print(f"  {code}")
        return 0

    if args.write_baseline:
        grown = sorted(set(missing) - known)
        if grown and not args.allow_growth:
            print("`[DIA-6]`'s page list may shrink and never grow.")
            for code in grown:
                print(f"  would add: {code}")
            return 1
        BASELINE.write_text(json.dumps(missing, indent=2) + "\n", encoding="utf-8", newline="\n")
        print(
            f"baseline written: {len(missing)} emitted codes without a page "
            f"({len(grown)} added, {len(known - set(missing))} closed)"
        )
        return 0

    problems = []
    with tempfile.TemporaryDirectory(prefix="ember-error-pages-") as tmp:
        work = Path(tmp)
        for page in pages:
            problems.extend(check_page(page, work))
    for code in missing:
        if code not in known:
            problems.append(f"[DIA-6] {code} is emitted by the compiler and has no docs/errors/{code}.md")

    print(f"{len(pages)} error page(s), {len(missing)} emitted code(s) without one ({len(known)} known)")
    if problems:
        print(f"\n{len(problems)} problem(s):\n")
        for problem in problems:
            print(f"  {problem}")
        return 1
    print("every page's failing example fails with its code, and every fix compiles")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
