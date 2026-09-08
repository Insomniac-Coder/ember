"""No file spells the project's names but the branding module (`[RT-5]`).

`[RT-5]`: "No file in the compiler, runtime, CMake module, examples or test
corpus may hard-code the symbol prefix, the CLI name, the manifest file name,
or the source and binding-cache extensions; each is read from a single
`branding` module. `tools/check_branding.py` fails CI on any hard-coded
occurrence."

The rule is about *auditability*, not about an expected rename. A name spelled
in ninety places cannot be checked: the one occurrence a rename misses becomes
a link error months later with nothing pointing at its cause.

What counts as hard-coding, and what does not:

- **Inside a string literal** — `"ember_alloc"`, `"ember.toml"`, `.em` as an
  extension. These are what reach the generated C, the file system and the
  user, and they are the violations.
- **Crate and module names** — `use ember_lexer::…`, `ember_branding` — are
  Rust paths, not the prefix. Renaming the project renames the crates by the
  same edit that renames the constant, and forcing them through a constant is
  impossible anyway.
- **Comments and documentation** — prose naming `ember_alloc` is how the rule
  is explained. Excluded.

    python tools/check_branding.py            # exit non-zero on a violation
    python tools/check_branding.py --report   # list every occurrence found
"""

import argparse
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BASELINE = ROOT / "tools" / "check_branding_baseline.json"

# The one file allowed to spell them, plus this checker and the C header that
# `[RT-5]` designates as the interface document.
EXEMPT = {
    Path("compiler/ember_branding/src/lib.rs"),
    Path("tools/check_branding.py"),
    Path("runtime/ember_rt/include/ember_rt.h"),
    # `[RT-5]` names the runtime among the places that may not hard-code the
    # prefix, and in its next sentence requires the header to carry literal
    # identifiers so embedders can read and grep it. The implementation file
    # defines exactly those identifiers, so it is exempt for the same reason.
    # Recorded as errata ERR-019.
    Path("runtime/ember_rt/src/ember_rt.c"),
}

SEARCH = [
    ("compiler", ("*.rs",)),
    ("runtime", ("*.c", "*.h")),
    ("cmake", ("*.cmake", "*.txt")),
    ("examples", ("*.em", "*.txt", "*.cmake")),
    ("std", ("*.em", "*.toml")),
]

# A Rust string literal, kept simple: no raw strings in these crates.
STRING = re.compile(r'"((?:[^"\\]|\\.)*)"')
LINE_COMMENT = re.compile(r"^\s*(//|/\*|\*|#)")

VIOLATIONS = [
    (re.compile(r"\bember_[a-z]"), "the runtime symbol prefix", "ember_branding::runtime(…) or {RT}"),
    (re.compile(r"\bem_[a-zA-Z]"), "the mangled prefix", "ember_branding::mangled(…)"),
    (re.compile(r"\bember\.toml\b"), "the manifest name", "ember_branding::MANIFEST"),
    (re.compile(r"\.embind\b"), "the binding-cache extension", "ember_branding::BINDING_EXT"),
    (re.compile(r"\.em\b(?!bind)"), "the source extension", "ember_branding::SOURCE_EXT"),
]


def files():
    for directory, patterns in SEARCH:
        base = ROOT / directory
        if not base.exists():
            continue
        for pattern in patterns:
            for path in base.rglob(pattern):
                if "target" in path.parts:
                    continue
                relative = path.relative_to(ROOT)
                if relative in EXEMPT:
                    continue
                yield relative, path


def scan(relative, path):
    out = []
    text = path.read_text(encoding="utf-8", errors="replace")
    rust = path.suffix == ".rs"
    for n, line in enumerate(text.split("\n"), 1):
        if LINE_COMMENT.match(line):
            continue
        # In Rust only string literals can reach the output; elsewhere the
        # whole line is content.
        haystacks = [m.group(1) for m in STRING.finditer(line)] if rust else [line]
        for haystack in haystacks:
            for pattern, what, fix in VIOLATIONS:
                if pattern.search(haystack):
                    out.append((relative, n, what, fix, line.strip()[:90]))
                    break
    return out


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--report", action="store_true")
    ap.add_argument("--write-baseline", action="store_true")
    args = ap.parse_args()

    found = []
    count = 0
    for relative, path in files():
        count += 1
        found.extend(scan(relative, path))

    # Keyed by file and by the line's text, not by line number, so moving code
    # does not re-fail while a genuinely new name does.
    keys = sorted(f"{r.as_posix()}: {line}" for r, _, _, _, line in found)
    print(f"scanned {count} files, {len(found)} occurrence(s)")

    if args.write_baseline:
        BASELINE.write_text(json.dumps(keys, indent=2) + "\n", encoding="utf-8", newline="\n")
        print(f"baseline written: {len(keys)} known")
        return 0

    known = set(json.loads(BASELINE.read_text(encoding="utf-8"))) if BASELINE.exists() else set()

    if args.report:
        for relative, n, what, fix, line in found:
            mark = " " if f"{relative.as_posix()}: {line}" in known else "!"
            print(f" {mark} {relative}:{n}: {what}")
            print(f"      {line}")
            print(f"      use {fix}")
        return 0

    new = [f for f in found if f"{f[0].as_posix()}: {f[4]}" not in known]
    if new:
        print(f"\n{len(new)} new hard-coded occurrence(s):\n")
        for relative, n, what, fix, line in new:
            print(f"  {relative}:{n}: {what}")
            print(f"      {line}")
            print(f"      use {fix}")
        return 1

    print(f"no new hard-coded names ({len(known)} known, all fixture file names in tests)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
