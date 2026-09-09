"""The hardening gate: every difference from the owner's file must be declared.

`docs/spec-source/as-received/` holds the specification exactly as the owner
sent it and is never edited. `docs/spec-source/ember-spec.md` is the hardened
copy. This checks that the second differs from the first **only** where
`docs/spec-amendments.md` says it does.

    python tools/hardening_check.py                # check, non-zero on a surprise
    python tools/hardening_check.py --report       # list every difference and its anchor

## What it can and cannot do

It answers *"is this change declared?"* — mechanically, every time.

It cannot answer *"is this change semantically neutral?"* No tool can read a
sentence and tell whether it altered what Ember means; that is the review the
owner does. What this removes is the failure mode where a change reaches the
document without anyone noticing it is there at all — which is how both
withdrawn amendments (A4 and A6) got in. Each was declared in the ledger and
still went into the normative body, so this gate would not have caught them
either; what it catches is the *undeclared* edit, the one nobody reviews
because nobody knows to look.

## How a difference is attributed

Each changed line is attributed to the nearest **anchor** above it: a rule id
(`[SPN-1]`), a grammar production (`fn_header :=`), or a named section. The
anchor must then appear in the amendment ledger's declared set, which is read
from `docs/spec-amendments.md` rather than hard-coded here — so declaring an
amendment is what permits the edit, and the two cannot drift.

Wholly new sections are exempt by name: the hardening's own change log and the
version header are additions to the front matter, not edits to a rule.
"""

import argparse
import difflib
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
AS_RECEIVED = ROOT / "docs" / "spec-source" / "as-received" / "Ember_v0.8.3_spec.md"
HARDENED = ROOT / "docs" / "spec-source" / "ember-spec.md"
LEDGER = ROOT / "docs" / "spec-amendments.md"

RULE_ID = re.compile(r"`\[([A-Z][A-Z0-9-]*-[0-9]+[a-z]?)\]`")
PRODUCTION = re.compile(r"^([a-z_]+)\s*:=")
HEADING = re.compile(r"^#{1,3}\s+(.*)$")

# Front matter the hardening adds wholesale rather than editing.
NEW_SECTIONS = ("Change log — 0.8.3_Hardened_1", "**Hardening:**", "**Version:**")

# The markers the hardening writes into the text. A line that differs only by
# one of these is an annotation, not a change to what the rule says — but it is
# still attributed and still has to be declared, because the annotation marks a
# change that is right beside it.
MARKERS = (
    "*(clarified 2026-09-09;",
    "*(head recovered verbatim 2026-09-09;",
    "*(editorial instruction carried out 2026-09-09;",
    "*(0.6.2 leftover removed 2026-09-09;",
)


def declared_anchors():
    """Everything the ledger says was touched.

    Read from the amendment table and the prose, so a change with no entry has
    nowhere to hide: the ledger is the permission, not a description of what
    already happened.
    """
    text = LEDGER.read_text(encoding="utf-8")
    anchors = set(RULE_ID.findall(text))
    # Grammar productions are named in the table as bare identifiers.
    for name in re.findall(r"`([a-z_]+)`", text):
        anchors.add(name)
    # The keyword table has no rule id of its own.
    if "keyword" in text.lower():
        anchors.add("KEYWORD-TABLE")
    return anchors


# A line that *opens* a rule. Two shapes, because the document uses both — and
# the second must be pinned to column 0:
#
#     * `[SPN-1]` `Array[T]` coerces …          a bullet, at any indent
#     `[TYP-15]` A view-typed value MUST NOT …  no bullet, column 0
#
# Allowing an unbulleted opening at *any* indent reads a wrapped continuation
# line as a new rule — `[FFI-17d]`'s body wraps onto a line beginning
# "`[FFI-39c]` releases Ember's strong reference", and the change two lines
# below it was then blamed on `[FFI-39c]`. That is the fourth way this detector
# has been wrong; each time it named a real rule and the wrong one, which is
# why the gate reports rather than silently attributing.
BULLET_RULE = re.compile(r"^(?:\s*[*-]\s+|)`\[([A-Z][A-Z0-9-]*-[0-9]+[a-z]?)\]`")


def anchor_of(lines, index, inserted=()):
    """Which rule, production or section a changed line belongs to.

    Walks up to the nearest line that **opens** a rule — a bullet whose first
    token is the id — rather than to the nearest id of any kind. The difference
    is not cosmetic: `[EFF-18]`'s amended text contains the words "the four
    `[EFF-16]` assigns", so a search for any id finds `[EFF-16]` on the changed
    line itself and blames the wrong rule. Three of the first twelve reports
    were wrong that way.

    An inserted block that carries its own hardening heading is a new section,
    not an edit to whatever precedes it.
    """
    for line in inserted:
        h = HEADING.match(line)
        if h and "Hardened" in h.group(1):
            return "NEW-SECTION"
        for marker in NEW_SECTIONS:
            if marker in line:
                return "NEW-SECTION"
    for i in range(index, -1, -1):
        line = lines[i]
        for marker in NEW_SECTIONS:
            if marker in line:
                return "NEW-SECTION"
        m = PRODUCTION.match(line.strip())
        if m:
            return m.group(1)
        m = BULLET_RULE.match(line)
        if m:
            return m.group(1)
        if "Reserved keywords (v1)" in line or "Reserved for future use" in line:
            return "KEYWORD-TABLE"
        h = HEADING.match(line)
        if h and "Change log" in h.group(1):
            return "NEW-SECTION" if "Hardened" in h.group(1) else "CHANGELOG"
    return "PREAMBLE"


def differences():
    old = AS_RECEIVED.read_text(encoding="utf-8").split("\n")
    new = HARDENED.read_text(encoding="utf-8").split("\n")
    out = []
    matcher = difflib.SequenceMatcher(None, old, new, autojunk=False)
    for tag, i1, i2, j1, j2 in matcher.get_opcodes():
        if tag == "equal":
            continue
        # Attribute to the hardened side, which is where the anchor sits.
        at = j1 if j1 < len(new) else len(new) - 1
        out.append((tag, anchor_of(new, at, new[j1:j2]), j1 + 1, i2 - i1, j2 - j1))
    return out


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--report", action="store_true")
    args = ap.parse_args()

    for path in (AS_RECEIVED, HARDENED, LEDGER):
        if not path.exists():
            sys.exit(f"missing: {path}")

    declared = declared_anchors()
    diffs = differences()
    undeclared = [
        d for d in diffs
        if d[1] not in declared and d[1] not in ("NEW-SECTION", "PREAMBLE")
    ]

    print(f"{len(diffs)} difference(s) from the owner's file, "
          f"{len(declared)} anchor(s) declared in the ledger")

    if args.report:
        print("\n-- every difference, with the anchor it is attributed to --")
        for tag, anchor, line, removed, added in diffs:
            mark = " " if anchor in declared or anchor in ("NEW-SECTION", "PREAMBLE") else "!"
            print(f" {mark} {tag:<7} line {line:<6} -{removed} +{added}  {anchor}")

    if undeclared:
        print(f"\n{len(undeclared)} undeclared difference(s):\n")
        for tag, anchor, line, removed, added in undeclared:
            print(f"  line {line}: {tag} at `{anchor}`, which no amendment declares")
        print(
            "\nThe owner's file is the contract. Either revert the change, or add an "
            "amendment to docs/spec-amendments.md saying what it is and which of the "
            "four permitted classes it belongs to."
        )
        sys.exit(1)

    print("every difference is declared in the amendment ledger")


main()
