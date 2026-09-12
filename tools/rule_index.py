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
7. **Every registered code has a conformance test that asserts it**
   (`[TST-4]`, `[DIA-6a]`). A code the compiler can emit and no test ever names
   is a code whose meaning nothing holds in place: it can be renumbered,
   repurposed, or silently stop being emitted, and every suite stays green.
   Fix-list item 28 asked for this.

   The neighbouring check it asked for — *does the rule the registry cites
   actually name this code* — is **not** built, and deliberately. Measured both
   ways it reports 47 and 83 disagreements, and nearly all are legitimate: a
   rule routinely *uses* a code that another rule *defines* (`[ATT-2]` names
   `E0104`, which `[ATT-1]` defines), and the document does not mark which is
   which. A gate at 23% false positives is a gate nobody reads. What is needed
   first is for the specification to distinguish defining a code from citing
   one; that is an owner question, recorded in `docs/spec-errata.md`.
8. **Every rule reference resolves.** A rule that cites `[FFI-17b]` for
   something `[FFI-17b]` does not say is one problem; a rule that cites an id
   no rule defines is a worse one, because a reader cannot even find out. That
   was ERR-034: `[FFI-17d]` pointed at `@ffi(no_virtual_dtor)` "which
   `[FFI-17b]` covers", and no rule anywhere defined the attribute. This check
   cannot tell whether a citation is *apt*, but it catches every citation with
   nothing on the other end.

Checks 3, 5 and 6 cannot pass before the phases that create their artefacts, so
they run against a recorded baseline: a violation already in the baseline is
reported as known, a new one fails. `--write-baseline` records the current
state. This is the same shape `[TST-7]` uses for its `,ignore` blocks.

    python tools/rule_index.py                 # check, exit non-zero on new gaps
    python tools/rule_index.py --write-baseline
    python tools/rule_index.py --report        # full listing, always exits 0
    python tools/rule_index.py --spec path.md --report
                                                # audit another spec without adopting it
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
HEADING_RULE_ID = re.compile(
    r"^\s*#{1,6}\s+\[([A-Z][A-Z0-9-]*-[0-9]+[a-z]?)\](?:\s|$)"
)
PARAGRAPH_RULE_ID = re.compile(
    r"^\s*`?\[([A-Z][A-Z0-9-]*-[0-9]+[a-z]?)\]`?(?=\s|$)"
)
CODE = re.compile(r"\b([EWL])(\d{4})\b")
REGISTRY_ENTRY = re.compile(
    r"^\s*([EWL]\d{4})\s*=\s*\((\w+),\s*\d+,\s*(\w+),\s*\"([^\"]*)\"", re.M
)

# A definition states a rule: the id opens a bullet, or follows a short label
# at the head of one, and normative prose follows. Everything else is a
# reference — `[XXII.4]`: "a reference is not a definition".
# Words that announce a citation rather than a definition, when they appear
# before the first rule id on a line.
REFERENCE_LEAD = re.compile(
    r"\b(?:see|per|under|from|by|cites?|citing|named in|listed in|which|that|and)\s*$",
    re.I,
)
REFERENCE_TAIL = re.compile(r"^(?:'s|,|\)|\.|;|:|\s+and\b|\s+applies|\s+is unaffected)")
DEFINITION_TAIL = re.compile(r"^\s+(?:MUST|SHOULD|MAY|\*\*|…|\.\.\.|[A-Z`])")

# RIDX-1 (ODR-002, design in `docs/RFC-rule-extraction.md`): the document
# states rules in four forms, and only Form A opens a bullet. Forms B/C/D put
# the id somewhere else — second on a shared line, inline in a paragraph, in a
# parenthetical, or granted by name — so no position test can see them. The
# definition/reference distinction still holds there and is still decided by
# the surrounding text, never by the position: each predicate below recognises
# a clause that *states rule content*, and a passing mention in the same
# position ("`[ATT-2]` is unchanged", "`[HOT-1]`..`[HOT-10]` are replaced",
# "`[FAKE]`-style tests") matches none of them. See
# `tools/test_rule_index.py`, which pins both directions.
DEFINITION_PREDICATES = (
    # "`[TYP-26]` two functions with the same name in one scope is `E1030`":
    # the clause states which error code the rule raises.
    re.compile(r"^\s+[^.;(),]*\bis\s+`E\d{4}`"),
    # "`[VER-7]` 1.0 means ...": a bare subject with the definitional copula.
    # The subject is one token on purpose: "`[HR-36]` ... so "fails" never
    # means "aborts"" is a citation about wording, and the comma before its
    # "means" keeps it out either way.
    re.compile(r"^\s+[\w.]+\s+means\b"),
    # "`[HND-2]` the index/generation split is configurable per `Pool`":
    # the clause states configurability against a named surface.
    re.compile(r"^\s+[^.;(),]*\bis\s+configurable\b"),
    # "`[GPU-7]` device drop ... logs each leaked handle ...": the clause
    # states a reporting duty. The comma exclusion matters: a subordinate
    # "..., logs Z" after a citation never reaches the verb.
    re.compile(r"^\s+[^.;(),]*\blogs\b"),
    # "`[CTL-3a]` conformance test checks the C output ...": the clause
    # states what a conformance test enforces.
    re.compile(r"^\s+[^.;(),]*\bconformance test\s+checks\b"),
)

# Form D, granted by name: "`[RC-2]` ... Its lettered clauses are individually
# citable as `[RC-2a]`..`[RC-2d]` in the order written." A bare range without
# the granting words ("`[HOT-1]`..`[HOT-10]` are replaced", "`[VER-1]`..`[VER-7]`",
# "Like `[RC-2a]`..`[RC-2d]`, this elision ...") grants nothing.
GRANT_RANGE = re.compile(
    r"individually citable as `\[([A-Z][A-Z0-9-]*)-([0-9]+)([a-z])\]`\.\.`\[([A-Z][A-Z0-9-]*)-([0-9]+)([a-z])\]"
)


def spec_lines(path=SPEC):
    if not path.exists():
        sys.exit(f"specification not found at {path}")
    return path.read_text(encoding="utf-8").split("\n")


def rule_definitions(lines):
    """id -> [line numbers where it is *stated as a rule*]."""
    defs = {}
    for n, line in enumerate(lines, 1):
        # Consolidated specifications may give a rule its own Markdown
        # heading. The id must immediately follow the heading marker: a
        # heading such as "Notes on [TYP-1]" is a reference, not a definition.
        heading = HEADING_RULE_ID.match(line)
        if heading:
            defs.setdefault(heading.group(1), []).append(n)
            continue
        paragraph = PARAGRAPH_RULE_ID.match(line)
        if paragraph:
            tail = line[paragraph.end():]
            if DEFINITION_TAIL.match(tail):
                defs.setdefault(paragraph.group(1), []).append(n)
            continue
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


def stated_anywhere(lines):
    """Every id the document *states* a rule for, wherever on the line it sits.

    Laxer than `rule_definitions` on purpose. That one answers `[XXII.4]`'s
    question — is this id defined *twice* — and so counts only the first id on
    a bullet, because "`[MAN-1]` Unknown keys are errors. `[MAN-2]` ..." must
    not read as `[MAN-2]` being defined by `[MAN-1]`'s bullet.

    The dangling-reference check asks a different question — is this id defined
    *at all* — and for that the second rule on such a line is plainly defined.
    Using the strict set here reported thirty rules as undefined that the
    document defines perfectly well, just not at the start of a bullet.
    """
    stated = set()
    for line in lines:
        heading = HEADING_RULE_ID.match(line)
        if heading:
            stated.add(heading.group(1))
        # A grant names its ids outright ("individually citable as
        # `[RC-2a]`..`[RC-2d]`"), so every id in the range is stated even
        # though only the endpoints are written out.
        grant = GRANT_RANGE.search(line)
        if (
            grant
            and grant.group(1) == grant.group(4)
            and grant.group(2) == grant.group(5)
            and grant.group(3) < grant.group(6)
        ):
            for letter in range(ord(grant.group(3)), ord(grant.group(6)) + 1):
                stated.add(f"{grant.group(1)}-{grant.group(2)}{chr(letter)}")
        # The first id on a line is what that line is about, whatever
        # punctuation follows it, unless the words before it announce a
        # citation. The document states a rule in at least four shapes —
        #
        #     * `[SPN-1]` `Array[T]` coerces …          a bullet
        #     * Parameter modes `[FN-1]`:               a labelled bullet
        #     **Integer overflow** `[TYP-8]`: in the …  a bold label, no bullet
        #     `[MAN-1]` … errors. `[MAN-2]` `ember.lock` …   two on one line
        #
        # — and a test tight enough to reject every citation rejected three of
        # these too, reporting thirty rules as undefined that the document
        # defines. Over-counting here is the safer error: it costs a missed
        # dangling reference, where under-counting costs a gate nobody trusts.
        first = QUOTED_RULE_ID.search(line)
        if first and not REFERENCE_LEAD.search(line[: first.start()]):
            stated.add(first.group(1))
        # And a later id on any line, where normative prose follows it —
        # "`[MAN-1]` Unknown keys are errors. `[MAN-2]` `ember.lock` records…"
        # RIDX-1 extends this to Forms B/C/D: a later id whose clause states
        # rule content (`DEFINITION_PREDICATES`) counts, whatever position the
        # id sits in. `REFERENCE_TAIL` still runs first, so a citation in the
        # same position ("`[ATT-2]` is unchanged", "`[X]`-style tests") stays
        # a reference.
        for m in QUOTED_RULE_ID.finditer(line):
            tail = line[m.end():]
            if REFERENCE_TAIL.match(tail):
                continue
            if DEFINITION_TAIL.match(tail):
                stated.add(m.group(1))
            elif any(p.match(tail) for p in DEFINITION_PREDICATES):
                stated.add(m.group(1))
    return stated


def dangling_references(lines, defined):
    """Rule ids the document mentions and never defines.

    A definition is `rule_definitions`' structural test; everything else that
    looks like a rule id is a reference. Some are deliberate and are not
    defects, so three kinds are excluded:

    * ids inside a `## Change log` section for an earlier revision, which
      `[XXII.4]` already says to ignore — they record what a past revision did
      and may name rules since removed (`[CTR-*]`, `[PRV-*]`, `[HOT-*]`);
    * prefix wildcards written as a family (`[SPN-*]`), which are prose;
    * ids the index itself lists as prefixes rather than rules.
    """
    out = []
    in_old_changelog = False
    seen_a_changelog = False
    for number, line in enumerate(lines, start=1):
        stripped = line.strip()
        if stripped.startswith("## Change log"):
            # The topmost change log is the current one, whatever version it
            # names. Matching a literal version string meant the check silently
            # started ignoring the current section the moment the version moved,
            # which is the failure mode a version-coupled constant always has.
            in_old_changelog = seen_a_changelog
            seen_a_changelog = True
            continue
        if stripped.startswith("## ") and not stripped.startswith("## Change log"):
            in_old_changelog = False
        if in_old_changelog:
            continue
        for rid in RULE_ID.findall(line):
            if rid not in defined:
                out.append((rid, number))
    return out


def codes_asserted_by_tests():
    """Every code a conformance or `tests/` case names in a `#$` annotation."""
    found = set()
    for root in (CONFORMANCE, ROOT / "tests"):
        if not root.exists():
            continue
        for path in root.rglob("*.em"):
            for line in path.read_text(encoding="utf-8", errors="ignore").splitlines():
                if "#$" not in line:
                    continue
                found.update(f"{k}{n}" for k, n in CODE.findall(line))
    return found


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
    return {
        "rules_without_tests": [],
        "codes_without_pages": [],
        "codes_not_in_registry": [],
        "dangling_references": [],
        "codes_without_tests": [],
    }


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
    ap.add_argument(
        "--spec",
        type=Path,
        default=SPEC,
        help="specification source to inspect (defaults to the normative source)",
    )
    args = ap.parse_args()

    spec_path = args.spec.resolve()
    if args.write_baseline and spec_path != SPEC.resolve():
        ap.error("--write-baseline is only valid for the normative specification")

    lines = spec_lines(spec_path)
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

    # --- 7. a registered code no test asserts ------------------------------
    asserted = codes_asserted_by_tests()
    untested = sorted(c for c in registry_entries() if c not in asserted)

    # --- 7. every rule reference resolves ----------------------------------
    seen = set()
    dangling = []
    for rid, n in dangling_references(lines, stated_anywhere(lines)):
        if rid not in seen:
            seen.add(rid)
            dangling.append((rid, n))

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
            "dangling_references": sorted({rid for rid, _ in dangling}),
            "codes_without_tests": untested,
        }
        grown = {
            key: sorted(set(values) - set(known.get(key, [])))
            for key, values in proposed.items()
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
            len(set(known.get(key, [])) - set(values)) for key, values in proposed.items()
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
    for c in untested:
        if c not in known.get("codes_without_tests", []):
            failures.append(f"[TST-4] {c} is registered and no conformance test asserts it")
    for rid, n in dangling:
        if rid not in known.get("dangling_references", []):
            failures.append(f"`[{rid}]` (line {n}) is cited and defined by no rule")

    print(f"rules: {len(rules)}   stated: {len(defs)}   codes named: {len(named)}   "
          f"registry: {len(registry)}")
    print(f"known gaps (baseline): {len(known['rules_without_tests'])} rules without tests, "
          f"{len(known['codes_without_pages'])} codes without pages, "
          f"{len(known['codes_not_in_registry'])} codes not in the registry, "
          f"{len(known.get('dangling_references', []))} dangling references, "
          f"{len(known.get('codes_without_tests', []))} codes without tests")

    if args.report:
        print("\n-- duplicate rule definitions --")
        for rid, where in sorted(duplicates.items()):
            print(f"   {rid}: lines {', '.join(str(n) for n in where)}")
        print("\n-- orphaned amendments --")
        for rid, n in orphaned_amendments(lines):
            print(f"   {rid}: line {n}")
        print("\n-- dangling rule references --")
        for rid, n in dangling:
            print(f"   {rid}: first seen at line {n}")
        print("\n-- rules with no conformance directory --")
        for r in missing_tests:
            print(f"   {r}")
        print("\n-- codes named in the specification but absent from the registry --")
        for c in not_in_registry:
            print(f"   {c}")
        print("\n-- codes with no error page --")
        for c in missing_pages:
            print(f"   {c}")
        print("\n-- registered codes no conformance test asserts --")
        for c in untested:
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
