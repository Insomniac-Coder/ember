"""Split the single-file Ember specification into one file per Part.

`docs/spec-source/ember-spec.md` is the **normative** document (Part XX §1
ground rule 3). `docs/spec/` is generated from it by this script and MUST NOT
be hand-edited: an errata ruling is applied to the source and the split is
regenerated. Editing the split instead is how 0.3 silently reverted four owner
rulings.

Output is written LF-only. `.gitattributes` pins LF, and a scripted edit that
leaves a file mixed makes the *next* scripted edit fail silently.
"""
import re
import sys
from pathlib import Path

ROMAN = {
    "0": 0, "I": 1, "II": 2, "III": 3, "IV": 4, "V": 5, "VI": 6, "VII": 7,
    "VIII": 8, "IX": 9, "X": 10, "XI": 11, "XII": 12, "XIII": 13, "XIV": 14,
    "XV": 15, "XVI": 16, "XVII": 17, "XVIII": 18, "XIX": 19, "XX": 20,
    "XXI": 21, "XXII": 22,
}
HEADING = re.compile(r"^# (Part ([0-9IVX]+)|Appendix ([A-Z]))\b")


def slug(text):
    text = text.lower()
    text = re.sub(r"[^a-z0-9]+", "-", text)
    return text.strip("-")[:60]


def write(path, text):
    """Always LF, on every host."""
    path.write_text(text, encoding="utf-8", newline="\n")


def main(source, out_dir):
    lines = Path(source).read_text(encoding="utf-8").splitlines(keepends=True)
    out_dir = Path(out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)

    version = "unknown"
    for line in lines[:20]:
        m = re.match(r"\*\*Version:\*\* *([0-9.]+)", line)
        if m:
            version = m.group(1)
            break

    cuts = []  # (line index, filename stem, title)
    for i, line in enumerate(lines):
        m = HEADING.match(line)
        if not m:
            continue
        title = line[2:].strip()
        if m.group(2) is not None:
            n = ROMAN[m.group(2)]
            stem = f"part-{n:02d}-{slug(title.split('—')[-1])}"
        else:
            stem = f"appendix-{m.group(3).lower()}-{slug(title.split('—')[-1])}"
        cuts.append((i, stem, title))

    if not cuts:
        sys.exit("no Part headings found in " + source)

    written = []
    front = "".join(lines[: cuts[0][0]])
    write(out_dir / "part-00-preface.md", front)
    written.append(("part-00-preface.md", "Preface — how to use this document"))

    for idx, (start, stem, title) in enumerate(cuts):
        end = cuts[idx + 1][0] if idx + 1 < len(cuts) else len(lines)
        name = stem + ".md"
        write(out_dir / name, "".join(lines[start:end]))
        written.append((name, title))

    index = [f"# Ember specification (v{version}), by part\n",
             "\n**Generated** by `tools/split_spec.py` from"
             " `docs/spec-source/ember-spec.md`, which is the normative"
             " document. Do not hand-edit these files: apply the ruling to the"
             " source, record it in `docs/spec-errata.md`, and regenerate.\n",
             "\n| File | Part |\n|---|---|\n"]
    for name, title in written:
        index.append(f"| [{name}]({name}) | {title} |\n")
    write(out_dir / "README.md", "".join(index))

    print(f"wrote {len(written)} files to {out_dir}")
    for name, title in written:
        print(f"  {name}")


def check(source, out_dir):
    """`docs/spec/` is exactly what the source splits into.

    Part XX §1 ground rule 3 forbids hand-editing the split. A hand edit is
    invisible in review — it looks like the specification — so it is caught
    here instead: split to a scratch directory and compare.
    """
    import filecmp
    import shutil
    import tempfile

    tmp = Path(tempfile.mkdtemp(prefix="ember-spec-check-"))
    try:
        main(source, tmp)
        out_dir = Path(out_dir)
        expected = {p.name for p in tmp.glob("*.md")}
        actual = {p.name for p in out_dir.glob("*.md")}
        problems = []
        for name in sorted(expected - actual):
            problems.append(f"missing from {out_dir}: {name}")
        for name in sorted(actual - expected):
            problems.append(f"not produced by the split, so hand-added: {name}")
        for name in sorted(expected & actual):
            if not filecmp.cmp(tmp / name, out_dir / name, shallow=False):
                problems.append(f"differs from the source: {name}")
        if problems:
            print("\ndocs/spec/ is not the split of docs/spec-source/:")
            for p in problems:
                print(f"  {p}")
            print("\nrun: python tools/split_spec.py docs/spec-source/ember-spec.md docs/spec")
            return 1
        print(f"docs/spec/ matches the source ({len(expected)} files)")
        return 0
    finally:
        shutil.rmtree(tmp, ignore_errors=True)


if __name__ == "__main__":
    if "--check" in sys.argv:
        argv = [a for a in sys.argv[1:] if a != "--check"]
        sys.exit(check(argv[0], argv[1]))
    main(sys.argv[1], sys.argv[2])
