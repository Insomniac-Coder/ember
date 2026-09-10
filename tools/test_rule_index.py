"""Tests for the rule-definition extractor in `tools/rule_index.py`.

Covers RIDX-1 (`docs/BACKLOG.md`, ODR-002, design in
`docs/RFC-rule-extraction.md`): six rules stated in non-bullet forms must be
recognised as defined, while citations sitting in the same positions must
still be refused. The second half is the one that matters — a genuinely
undefined rule slipping through is worse than a known-good one sitting in a
baseline — so most of this file is false definitions the extractor must
reject.

Run:  python tools/test_rule_index.py
"""

import json
import unittest
from pathlib import Path

import rule_index

# Verbatim lines from docs/spec-source/ember-spec.md (line numbers at the
# time of writing: 1402, 1556, 1628, 1834, 2031, 3452, 4490). Quoted rather
# than read so the test pins the exact wording the extractor handles: if the
# owner rewords a rule, this breaks loud and forces a re-examination instead
# of silently re-baselining.
TYP_LINE = (
    "Named arguments `f(x=1, y=2)` match parameter names; positional arguments MUST precede named; "
    "`[TYP-25]` parameters with defaults may be omitted. Overloading by arity or type is **not** "
    "supported (use defaults, generics, or distinct names); `[TYP-26]` two functions with the same "
    "name in one scope is `E1030` \u2014 except operator interface impls and `extend` blocks for "
    "distinct types."
)
IFC_LINE = (
    "* `[IFC-1]` `extend T:` without `implements` adds inherent methods to `T` from any module in "
    "the same package (or the declaring package of `T`); `[IFC-2]` inherent extension of foreign "
    "types from a third package is `E2120` (avoids silent API changes); use a wrapper or an interface."
)
CTL_LINE = (
    "* `[CTL-3]` Ranges `a..b` (`Range[T]`), `a..=b` (`RangeInclusive[T]`), `a..` (`RangeFrom`) "
    "implement `Iterator` for integer `T` and compile to a counted loop with no iterator object in "
    "memory (guaranteed by MIR lowering of `for` over range literals \u2014 `[CTL-3a]` conformance "
    "test checks the C output has no struct temporaries)."
)
RC_LINE = (
    "* `[RC-2]` **Guaranteed elisions** (conformance-tested): (a) passing a handle to a `borrowed` "
    "parameter emits no RC ops; (b) a handle read from a place and used only within a single "
    "expression emits no RC ops when the place is not written during the expression; (c) `retain` "
    "immediately followed by `release` on the same handle with no intervening call or store is "
    "removed; (d) a handle stored into a field from a temporary is moved, not retained+released. "
    "Its lettered clauses are individually citable as `[RC-2a]`..`[RC-2d]` in the order written."
)
HND_LINE = (
    "`[HND-1]` `Handle` is a plain `Copy` value; `[HND-2]` the index/generation split is "
    "configurable per `Pool` (`Pool[T, INDEX_BITS=20]`)."
)
GPU_LINE = (
    "`[GPU-6]` `device.destroy(h)` invalidates the handle immediately (generation bump) and queues "
    "the physical object on the retirement list of the current frame; the runtime destroys it when "
    "that frame's fence has been observed signalled. This is RageV's `VulkanDevice::DeferDestruction` "
    "made a standard primitive. Dropping the device flushes all retirement lists after a full "
    "wait-idle; `[GPU-7]` device drop while handles are still live logs each leaked handle with its "
    "creation site in debug."
)
VER_LINE = (
    "**The 1.0 compatibility promise** (owner decision `OQ-22`; normative from 1.0). `[VER-2]` source "
    "compatibility within a major language version, with breaking changes only in a new major that a "
    "package opts into by editing `language`, and a compiler accepting every language version of its "
    "own major series. `[VER-3]` deprecation in `1.n` via `@deprecated` and a `W`-code naming the "
    "replacement and the removing version, with removal no earlier than the next major. `[VER-7]` 1.0 "
    "means `[VER-2]` and `[VER-4]` come into force, the conformance suite passes on every supported "
    "host, `docs/errors/EXXXX.md` exists for every code, and the user guide exists."
)

SIX = {"TYP-26", "IFC-2", "HND-2", "GPU-7", "VER-7", "CTL-3a"}
GRANTED = {"RC-2a", "RC-2b", "RC-2c", "RC-2d"}
# Baseline entries that must never be "recognised": deliberately reserved by
# Part XXI and a historical citation (withdrawn ERR-042 inventory).
MUST_STAY_DANGLING = {"HOT-10", "IDE-2", "IDE-5", "IDE-10"}


class FormRecognition(unittest.TestCase):
    def test_form_b_second_on_shared_bullet(self):
        stated = rule_index.stated_anywhere([TYP_LINE, IFC_LINE])
        self.assertIn("TYP-26", stated)
        self.assertIn("IFC-2", stated)

    def test_form_b_shared_paragraph_line(self):
        stated = rule_index.stated_anywhere([HND_LINE, GPU_LINE])
        self.assertIn("HND-2", stated)
        self.assertIn("GPU-7", stated)

    def test_form_c_inline_in_paragraph(self):
        self.assertIn("VER-7", rule_index.stated_anywhere([VER_LINE]))

    def test_form_d_parenthetical(self):
        self.assertIn("CTL-3a", rule_index.stated_anywhere([CTL_LINE]))

    def test_form_d_granted_by_name(self):
        stated = rule_index.stated_anywhere([RC_LINE])
        for rid in GRANTED:
            self.assertIn(rid, stated)


class FalseDefinitionsRefused(unittest.TestCase):
    """Citations in the newly recognised positions must stay references.

    Each line puts a FAKE id where a definition may now sit (second on a
    line, inline, parenthetical, range) but clothes it as a citation. Every
    one of these passes on the pre-RIDX-1 extractor too: the point is that
    recognising the six did not buy a single false negative.

    Every FAKE id sits behind a `[ZZZ-n]` anchor on purpose: the first id on
    a line is always counted as stated (pre-existing behaviour, guarded by
    the `REFERENCE_LEAD` citation-lead test), so only a non-first id
    exercises the definition/reference distinction these tests pin.
    """

    def assert_refused(self, *lines, ids):
        stated = rule_index.stated_anywhere(list(lines))
        for rid in ids:
            self.assertNotIn(rid, stated, f"`[{rid}]` was accepted as a definition")

    def test_citation_after_semicolon(self):
        self.assert_refused(
            "* `[ZZZ-1]` MUST flush buffers; see `[FAKE-1]` for the flush order.",
            ids=("FAKE-1",),
        )

    def test_error_code_nearby_but_not_stated(self):
        self.assert_refused(
            "* `[ZZZ-1]` MUST report coverage; the table lists `[FAKE-2]` violations as `E9999`.",
            ids=("FAKE-2",),
        )

    def test_bare_mention_is_unchanged(self):
        # Shape of `[ATT-2]` at Part IV: a passing mention, not a rule.
        self.assert_refused(
            "Overloading is unsupported; `[ZZZ-2]` stays; `[FAKE-3]` is unchanged.",
            ids=("FAKE-3",),
        )

    def test_range_endpoint_stays_a_reference(self):
        # Shape of `[HOT-1]`..`[HOT-10]` in the change history.
        self.assert_refused(
            "0.6.3's `[ZZZ-3]`..`[FAKE-4]` are replaced entirely.",
            ids=("FAKE-4",),
        )

    def test_reserved_range_stays_reserved(self):
        # Shape of the Part XXI reservation line.
        self.assert_refused(
            "* `[ZZZ-4]`, `[FAKE-5]`..`[FAKE-6]` are **reserved** for the tool itself.",
            ids=("FAKE-5", "FAKE-6"),
        )

    def test_style_citation_in_parenthetical(self):
        # Shape of the `[CTL-3a]`-style mention: a use of the rule, not its text.
        self.assert_refused(
            "`[ZZZ-3]` ranges lower as counted loops \u2014 guaranteed by `[FAKE-7]`-style "
            "tests), `inline` of the rest.",
            ids=("FAKE-7",),
        )

    def test_range_without_granting_words_grants_nothing(self):
        self.assert_refused(
            "* `[ZZZ-1]` MUST hold; its clauses are citable as `[FAKE-8a]`..`[FAKE-8d]` in order.",
            ids=("FAKE-8a", "FAKE-8d"),
        )

    def test_grant_with_mismatched_prefix_grants_nothing(self):
        self.assert_refused(
            "* `[ZZZ-1]` MUST hold; its clauses are individually citable as "
            "`[FAKE-9a]`..`[OTHER-9d]` in the order written.",
            ids=("FAKE-9a", "OTHER-9d"),
        )

    def test_grant_with_reversed_range_grants_nothing(self):
        self.assert_refused(
            "* `[ZZZ-1]` MUST hold; its clauses are individually citable as "
            "`[FAKE-10d]`..`[FAKE-10a]` in the order written.",
            ids=("FAKE-10d", "FAKE-10a"),
        )

    def test_means_in_subordinate_clause(self):
        # Shape of `[HR-36]`: "means" inside a subordinate clause about wording.
        self.assert_refused(
            "`[ZZZ-5]` MUST hold; `[FAKE-11]` makes allocation fallible, "
            'so "fails" never means "aborts".',
            ids=("FAKE-11",),
        )

    def test_label_then_citation_after_semicolon(self):
        # Shape of `[RC-2]` at Part XX: a label, then a use of the rule.
        self.assert_refused(
            "* Retain/release insertion; `[ZZZ-2]` MUST hold and `[FAKE-12]` guaranteed "
            "elisions as passes.",
            ids=("FAKE-12",),
        )

    def test_bare_description_without_copula(self):
        # Shape of `[TYP-25]`'s mention on the `[TYP-26]` line: descriptive
        # prose that states no code, no meaning, no duty.
        self.assert_refused(
            "`[ZZZ-7]` MUST precede named; `[FAKE-13]` parameters with defaults may be omitted.",
            ids=("FAKE-13",),
        )


class ScopePins(unittest.TestCase):
    """Acceptance against the live specification: exactly the six (+ the
    `[RC-2]` grants) moved; reserved and historical entries stayed."""

    @classmethod
    def setUpClass(cls):
        cls.stated = rule_index.stated_anywhere(rule_index.spec_lines())
        baseline_path = Path(rule_index.__file__).resolve().parent / "rule_index_baseline.json"
        cls.baseline = json.loads(baseline_path.read_text(encoding="utf-8"))

    def test_six_are_stated(self):
        for rid in SIX:
            self.assertIn(rid, self.stated)

    def test_grants_are_stated(self):
        for rid in GRANTED:
            self.assertIn(rid, self.stated)

    def test_reserved_and_historical_stay_dangling(self):
        for rid in MUST_STAY_DANGLING:
            self.assertNotIn(rid, self.stated)

    def test_baseline_lists_only_genuine_gaps(self):
        for rid in self.baseline["dangling_references"]:
            self.assertNotIn(rid, self.stated, f"baselined `[{rid}]` is actually stated")

    def test_fixed_ids_left_the_baseline(self):
        for rid in SIX | {"RC-2a", "RC-2d"}:
            self.assertNotIn(rid, self.baseline["dangling_references"])

    def test_no_duplicate_rule_ids(self):
        defs = rule_index.rule_definitions(rule_index.spec_lines())
        duplicates = {k: v for k, v in defs.items() if len(v) > 1}
        self.assertEqual({}, duplicates)


if __name__ == "__main__":
    unittest.main()
