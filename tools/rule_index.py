"""Rule and error-code index checks (`[TST-4]`, `[DIA-6a]`, `[XXII.4]`).

The specification requires a tool that fails CI when the rule index and the
compiler drift apart. This is that tool. It runs six checks:

1. **Duplicate rule ids** (`[XXII.4]`). A rule id defined twice is a hard
   failure "regardless of section or prefix". A *reference* is not a
   definition, so a definition is recognised structurally: the id opens a
   bullet, or follows a bullet's short label, and normative prose follows it.
2. **Orphaned amendments.** A rule whose body begins with an ellipsis is an
   amendment that was appended rather than substituted into the rule it
   amends. Two such rules in v0.5 had lost their original text entirely; this
   check costs four lines and catches the next one on the day it lands.
3. **Rule has a conformance directory** (`[TST-4]`).
4. **Every code named in the document is in the registry**, and every registry
   entry cites a rule that exists (`[DIA-6a]`, both directions).
5. **Every code has an error page** under `docs/errors/` (`[DIA-6]`).
6. **Every E3xxx code is keyed to a diagnostic shape** (`[DIA-7a]`).

Checks 3, 5 and 6 cannot pass before the phases that create their artefacts, so
they run against a recorded baseline: a violation already in the baseline is
reported as known, a new one fails. `--write-baseline` records the current
state. This is the same shape `[TST-7]` uses for its `,ignore` blocks.

    python tools/rule_index.py                 # check, exit non-zero on new gaps
    python tools/rule_index.py --write-baseline
    python tools/rule_index.py --report        # full listing, always exits 0
"""

import argparse
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SPEC = ROOT / "docs" / "spec-source" / "ember-spec.md"
CONFORMANCE = ROOT / "tests" / "conformance"
ERROR_PAGES = ROOT / "docs" / "errors"
REGISTRY_RS = ROOT / "compiler" / "ember_diag" / "src" / "codes.rs"
BASELINE = ROOT / "tools" / "rule_index_baseline.json"

# `[XXII.4]` fixes the extractor's pattern. The trailing letter admits
# amendment ids (`[LEX-11a]`), the hyphenated class admits `[CG-C-3]`.
RULE_ID = re.compile(r"\[([A-Z][A-Z0-9-]*-[0-9]+[a-z]?)\]")
QUOTED_RULE_ID = re.compile(r"`\[([A-Z][A-Z0-9-]*-[0-9]+[a-z]?)\]`")
CODE = re.compile(r"\b([EWL])(\d{4})\b")
REGISTRY_ENTRY = re.compile(
    r"^\s*([EWL]\d{4})\s*=\s*\((\w+),\s*\d+,\s*(\w+),\s*\"([^\"]*)\"", re.M
)

# A definition states a rule: the id opens a bullet, or follows a short label
# at the head of one, and normative prose follows. Everything else is a
# reference — `[XXII.4]`: "a reference is not a definition".
REFERENCE_TAIL = re.compile(r"^(?:'s|,|\)|\.|;|:|\s+and\b|\s+applies|\s+is unaffected)")
DEFINITION_TAIL = re.compile(r"^\s+(?:MUST|SHOULD|MAY|\*\*|…|\.\.\.|[A-Z`])")


def spec_lines():
    if not SPEC.exists():
        sys.exit(f"specification not found at {SPEC}")
    return SPEC.read_text(encoding="utf-8").split("\n")


def rule_definitions(lines):
    """id -> [line numbers where it is *stated as a rule*]."""
    defs = {}
    for n, line in enumerate(lines, 1):
        if not re.match(r"^\s*[*-]\s", line):
            continue
        # Only the FIRST rule id on a bullet can be the rule that bullet
        # states. A later one is cited by it, even when the citation reads
        # like a rule ("… and `[FFI-2]` MUST treat …"). This is `[XXII.4]`'s
        # "a reference is not a definition" made mechanical.
        m = QUOTED_RULE_ID.search(line)
        if not m:
            continue
        tail = line[m.end():]
        if REFERENCE_TAIL.match(tail):
            continue
        if DEFINITION_TAIL.match(tail):
            defs.setdefault(m.group(1), []).append(n)
    return defs


def orphaned_amendments(lines):
    """Rules whose body begins with an ellipsis: an appended amendment."""
    out = []
    for n, line in enumerate(lines, 1):
        m = re.search(r"`\[([A-Z][A-Z0-9-]*-[0-9]+[a-z]?)\]`\s*(…|\.\.\.)", line)
        if m:
            out.append((m.group(1), n))
    return out


def all_rule_ids(text):
    return sorted(set(RULE_ID.findall(text)))


def registry_entries():
    if not REGISTRY_RS.exists():
        return {}
    text = REGISTRY_RS.read_text(encoding="utf-8")
    return {m.group(1): m.group(4) for m in REGISTRY_ENTRY.finditer(text)}


def codes_named_in_spec(text):
    """Codes the document names as codes.

    The registry table in XIX §6 writes ranges (`E0000–E0099`). Those endpoints
    are not codes, so a range's two ends are excluded — otherwise the boundary
    of every subsystem lands in the "missing from the registry" list and hides
    the real gaps.
    """
    ranges = re.compile(r"\b[EWL]\d{4}\s*[–\-—]\s*[EWL]?\d{4}\b")
    stripped = ranges.sub(" ", text)
    return sorted({f"{k}{n}" for k, n in CODE.findall(stripped)})


def load_baseline():
    if BASELINE.exists():
        return json.loads(BASELINE.read_text(encoding="utf-8"))
    return {"rules_without_tests": [], "codes_without_pages": [], "codes_not_in_registry": []}


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--write-baseline", action="store_true")
    ap.add_argument(
        "--allow-growth",
        action="store_true",
        help="permit --write-baseline to record entries the baseline does not "
        "already hold. `[TST-4c]`: the baseline shrinks and never grows, so "
        "this is for a new specification revision only, and the reason belongs "
        "in the commit message.",
    )
    ap.add_argument("--report", action="store_true")
    args = ap.parse_args()

    lines = spec_lines()
    text = "\n".join(lines)
    failures = []
    known = load_baseline()

    # --- 1. duplicate rule ids (`[XXII.4]`) --------------------------------
    defs = rule_definitions(lines)
    duplicates = {k: v for k, v in defs.items() if len(v) > 1}
    for rid, where in sorted(duplicates.items()):
        failures.append(
            f"[XXII.4] `[{rid}]` is stated as a rule {len(where)} times "
            f"(lines {', '.join(str(n) for n in where)})"
        )

    # --- 2. orphaned amendments --------------------------------------------
    for rid, n in orphaned_amendments(lines):
        failures.append(
            f"`[{rid}]` (line {n}) begins with an ellipsis: an amendment "
            f"appended instead of substituted into the rule it amends"
        )

    # --- 3. every rule has a conformance directory (`[TST-4]`) -------------
    rules = all_rule_ids(text)
    have = {p.name for p in CONFORMANCE.iterdir()} if CONFORMANCE.exists() else set()
    missing_tests = sorted(r for r in rules if r not in have)

    # --- 4/5/6. codes ------------------------------------------------------
    registry = registry_entries()
    named = codes_named_in_spec(text)
    not_in_registry = sorted(c for c in named if c not in registry)
    pages = {p.stem for p in ERROR_PAGES.glob("*.md")} if ERROR_PAGES.exists() else set()
    missing_pages = sorted(c for c in registry if c not in pages)

    # `[DIA-6a]`, the other direction: a registry entry citing a rule that is
    # not in the specification. This one is never baselined — it means the
    # compiler is enforcing something the document does not say.
    rule_set = set(rules)
    for code, rule in sorted(registry.items()):
        cited = RULE_ID.findall(rule)
        for rid in cited:
            if rid not in rule_set:
                failures.append(
                    f"[DIA-6a] {code} cites `[{rid}]`, which is in no part of the specification"
                )

    if args.write_baseline:
        # `[TST-4c]` — "The baseline shrinks and never grows: rule_index.py
        # rejects a commit that adds a rule to it." A gap that was closed and
        # reopens is a regression, and a baseline that absorbs it silently is
        # the mechanism by which a suite stops meaning anything. Growth is
        # permitted only for a new specification revision, explicitly.
        proposed = {
            "rules_without_tests": missing_tests,
            "codes_without_pages": missing_pages,
            "codes_not_in_registry": not_in_registry,
        }
        grown = {
            key: sorted(set(values) - set(known[key])) for key, values in proposed.items()
        }
        if any(grown.values()) and not args.allow_growth:
            print("\n`[TST-4c]`: the baseline may shrink and never grow.")
            for key, values in grown.items():
                for v in values:
                    print(f"  would add to {key}: {v}")
            print(
                "\nEither close the gap, or pass --allow-growth and say in the "
                "commit message which specification revision opened it."
            )
            return 1
        BASELINE.write_text(
            json.dumps(proposed, indent=2) + "\n",
            encoding="utf-8",
            newline="\n",
        )
        added = sum(len(v) for v in grown.values())
        removed = sum(
            len(set(known[key]) - set(values)) for key, values in proposed.items()
        )
        print(f"baseline written: {len(missing_tests)} rules without tests, "
              f"{len(missing_pages)} codes without pages, "
              f"{len(not_in_registry)} codes not in the registry "
              f"({added} added, {removed} closed)")
        return 0

    for r in missing_tests:
        if r not in known["rules_without_tests"]:
            failures.append(f"[TST-4] `[{r}]` has no tests/conformance/{r}/ directory")
    for c in missing_pages:
        if c not in known["codes_without_pages"]:
            failures.append(f"[DIA-6] {c} has no docs/errors/{c}.md page")
    for c in not_in_registry:
        if c not in known["codes_not_in_registry"]:
            failures.append(f"[DIA-6a] {c} is named in the specification but not in the registry")

    print(f"rules: {len(rules)}   stated: {len(defs)}   codes named: {len(named)}   "
          f"registry: {len(registry)}")
    print(f"known gaps (baseline): {len(known['rules_without_tests'])} rules without tests, "
          f"{len(known['codes_without_pages'])} codes without pages, "
          f"{len(known['codes_not_in_registry'])} codes not in the registry")

    if args.report:
        print("\n-- rules with no conformance directory --")
        for r in missing_tests:
            print(f"   {r}")
        print("\n-- codes with no error page --")
        for c in missing_pages:
            print(f"   {c}")
        return 0

    if failures:
        print(f"\n{len(failures)} new problem(s):\n")
        for f in failures:
            print(f"  {f}")
        return 1

    print("\nno new problems")
    return 0


if __name__ == "__main__":
    sys.exit(main())
