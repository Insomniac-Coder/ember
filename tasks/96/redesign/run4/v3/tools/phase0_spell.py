"""Phase 0 of run 4 step 2: `holds` -> `exists as` in code only (verdict 26).

Code means: inline `...` spans anywhere, and lines inside fenced blocks whose code part (before `#`)
has `if`, `elif` or `while` before `holds`. English "holds" in sentences and comments is left alone.
`holds` never existed in today's Ember, so every code occurrence is the design's keyword.
Usage: python spell.py <v3 dir> <list file>
"""
import re
import sys
from pathlib import Path

KW = re.compile(r"\bholds\b")
SPAN = re.compile(r"`[^`\n]+`")
HEADER = re.compile(r"\b(if|elif|while)\b[^#]*\bholds\b")


def fix_code_line(line):
    code, sep, comment = line.partition("#")
    if HEADER.search(code):
        code = KW.sub("exists as", code)
    return code + sep + comment


def fix_file(path):
    out, changes, fenced = [], [], False
    for n, line in enumerate(path.read_text(encoding="utf-8").split("\n"), 1):
        new = line
        if line.lstrip().startswith("```"):
            fenced = not fenced
        else:
            if fenced:
                new = fix_code_line(new)
            new = SPAN.sub(lambda m: KW.sub("exists as", m.group(0)), new)
        if new != line:
            changes.append(f"{path.name}:{n}\n  - {line.strip()}\n  + {new.strip()}")
        out.append(new)
    path.write_text("\n".join(out), encoding="utf-8")
    return changes


def main():
    root, listing = Path(sys.argv[1]), Path(sys.argv[2])
    changes = []
    for name in ["rules.md", "vocabulary.md", "area-A.md", "area-B.md", "area-C.md", "area-D.md", "area-E.md"]:
        changes += fix_file(root / name)
    left = []
    for name in ["rules.md", "vocabulary.md", "area-A.md", "area-B.md", "area-C.md", "area-D.md", "area-E.md"]:
        for n, line in enumerate((root / name).read_text(encoding="utf-8").split("\n"), 1):
            if KW.search(line):
                left.append(f"{name}:{n}: {line.strip()}")
    listing.write_text(
        f"Phase 0: `holds` -> `exists as` in code (verdict 26). {len(changes)} lines changed.\n\n"
        + "\n".join(changes)
        + f"\n\nLines that still contain the word `holds` ({len(left)}; English, or code the script did not"
        " recognise: check yours):\n\n"
        + "\n".join(left)
        + "\n",
        encoding="utf-8",
    )
    print(len(changes), "changed;", len(left), "lines still contain holds")


if __name__ == "__main__":
    main()
