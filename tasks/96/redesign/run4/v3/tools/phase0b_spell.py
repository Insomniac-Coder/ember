"""Phase 0 of run 4 step 2, the other two spellings: verdict 27 (`if cast X if T exists as n:` ->
`if X is T as n:`) and verdict 33 (`.copied()` -> `.copy_each()`), in code only.

`phase0_spell.py` did verdict 26 (`holds` -> `exists as`) first; this runs on its output. Both old
spellings are today's Ember (`cast x if D` since ODR-280, `.copied()` in std), so unlike `holds` they
are kept wherever the text is today's Ember: an answer's part 2 (OLD) and part 5 (Means exactly), and
today's compiler output.

- Verdict 27: the full opening `cast X if T exists as n` is the v2 design's (today's Ember has no
  `exists as`), so it is respelled `X is T as n` in every inline code span and in the code part (before
  `#`) of every fenced line. A bare `cast x if D` is left: it is today's form or rule text, which the
  rule writer and the area updaters reword (verdict 27 removes the form). A line about the old form's
  "two `if`s" is left whole: respelled, it would say something false.
- Verdict 33: `.copied()` is respelled `.copy_each()` only inside an answer (a heading naming `AN-`),
  in its parts 3 (NEW), 4 (the owner's test), 6 (Still refused) and 7 (Cost). Elsewhere (`v2:` notes,
  change tables, "For the owner", the vocabulary's row) it is a statement about v2's choice to keep
  the name, for the agents to rewrite, not a spelling.

Every changed line, and every line still containing either old spelling with the reason it was left,
is appended to the list file. Usage: python phase0b_spell.py <v3 dir> <list file>
"""
import re
import sys
from pathlib import Path

FILES = ["rules.md", "vocabulary.md", "area-A.md", "area-B.md", "area-C.md", "area-D.md", "area-E.md"]
SPAN = re.compile(r"`[^`\n]+`")
CAST = re.compile(r"\bcast ([^`#]+?) if ([^\s`]+) exists as ")
COPIED = re.compile(r"\.copied\(\)")
OLD_CAST = re.compile(r"\bcast\b[^`#]*?\b(if|when)\b")
PART = re.compile(r"^\s*(?:(\d)\.\s+\*\*|\*\*(\d)\.\s)")
TWO_IFS = "two `if`s"
DESIGN_PARTS = {"3", "4", "6", "7"}
TODAY_PARTS = {"2", "5"}


def respell_cast(text):
    return CAST.sub(lambda m: f"{m.group(1)} is {m.group(2)} as ", text)


def respell_copied(text):
    return COPIED.sub(".copy_each()", text)


def in_code(line, fenced, fix):
    if fenced:
        code, sep, comment = line.partition("#")
        line = fix(code) + sep + comment
    return SPAN.sub(lambda m: fix(m.group(0)), line)


def fix_file(path):
    out, changes, left = [], [], []
    fenced, in_answer, part = False, False, None
    for n, line in enumerate(path.read_text(encoding="utf-8").split("\n"), 1):
        new = line
        stripped = line.lstrip()
        if stripped.startswith("```"):
            fenced = not fenced
        else:
            if not fenced:
                if stripped.startswith("#"):
                    in_answer, part = "AN-" in stripped, None
                elif in_answer and PART.match(line):
                    m = PART.match(line)
                    part = m.group(1) or m.group(2)
            if TWO_IFS not in line:
                new = in_code(new, fenced, respell_cast)
            if in_answer and part in DESIGN_PARTS:
                new = in_code(new, fenced, respell_copied)
        if new != line:
            changes.append(f"{path.name}:{n}\n  - {line.strip()}\n  + {new.strip()}")
        if COPIED.search(new) or OLD_CAST.search(new):
            if in_answer and part in TODAY_PARTS:
                why = "today's Ember (part " + part + ")"
            elif path.name in ("rules.md", "vocabulary.md"):
                why = "rule writer: rule text or a row to reword"
            else:
                why = "area updater: a statement or message to reword"
            left.append(f"{path.name}:{n} [{why}]: {new.strip()}")
        out.append(new)
    path.write_text("\n".join(out), encoding="utf-8")
    return changes, left


def main():
    root, listing = Path(sys.argv[1]), Path(sys.argv[2])
    changes, left = [], []
    for name in FILES:
        c, l = fix_file(root / name)
        changes += c
        left += l
    with listing.open("a", encoding="utf-8") as f:
        f.write(
            f"\n\nPhase 0, part 2: `if cast X if T exists as n:` -> `if X is T as n:` (verdict 27) and"
            f" `.copied()` -> `.copy_each()` (verdict 33), in code. {len(changes)} lines changed.\n\n"
            + "\n".join(changes)
            + "\n\nA respelled span can sit in a sentence written for the old form (`cast x if D`, \"opened"
            " with `exists as`\", \"`exists as` declares a new name\"): the rule writer (rules, vocabulary) and"
            " area D reword those sentences under verdict 27."
            + f"\n\nLines that still contain `.copied()` or `cast ... if` ({len(left)}), with the reason"
            " each was left (check yours):\n\n"
            + "\n".join(left)
            + "\n"
        )
    print(len(changes), "changed;", len(left), "left with an old spelling")


if __name__ == "__main__":
    main()
