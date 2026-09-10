# RFC — rule extraction: teach the tool the forms the document already uses

**Status: open, tooling work. `RIDX-1` in `docs/BACKLOG.md`.**
**Raised by:** owner resolution of ODR-002, 2026-09-10.
**Semantic impact: none.** Nothing in this RFC changes Ember. It changes how a
tool reads the specification.

---

## The problem, stated precisely

`tools/rule_index.py` decides whether a rule id is *defined* or merely
*referenced*. It does this structurally: the id must open a bullet, and **only
the first id on a bullet counts**. Its own comment gives the reason, and the
reason is good:

> Only the FIRST rule id on a bullet can be the rule that bullet states. A later
> one is cited by it, even when the citation reads like a rule. This is
> `[XXII.4]`'s "a reference is not a definition" made mechanical.

Six rules are fully and normatively stated in forms that test does not
recognise, so they report as dangling references and sit in
`tools/rule_index_baseline.json`:

| Rule | Form it is stated in |
|---|---|
| `[TYP-26]` | second rule on a line shared with `[TYP-25]`, after a semicolon |
| `[IFC-2]` | second rule on a line shared with `[IFC-1]` |
| `[HND-2]` | second rule on a line shared with `[HND-1]` |
| `[GPU-7]` | second rule on a line shared with `[GPU-6]` |
| `[VER-7]` | stated inline in a paragraph, no bullet |
| `[CTL-3a]` | stated inline in a parenthetical |

**These are not undefined rules.** `[TYP-26]`, for example, states in full:
*"two functions with the same name in one scope is `E1030` — except operator
interface impls and `extend` blocks for distinct types."* ERR-042 investigated
them as an undefined-rule problem and **was withdrawn as wrong**; the inventory
it produced is what established that all six are complete.

---

## The owner's ruling — which component is defective

> *"If a tool cannot correctly recognize a valid normative rule, the tool is the
> defective component, assuming the specification's structure is itself valid
> and intentional."*

The dependency that must **not** exist:

    Specification
          ↓  must conform to
    current tooling implementation

The architecture that must:

                     ┌── compiler
                     ├── conformance suite
    Specification ───┼── rule extractor
                     ├── diagnostics tooling
                     └── documentation tooling

The specification is the source of truth. Tools consume it.

**Two approaches are rejected outright**, and both are tempting:

1. **Rewriting the six rules to satisfy the extractor.** This inverts the
   dependency above. It is a layout edit to the owner's prose for a tool's
   convenience.
2. **Weakening the extractor so ambiguous definitions are accepted.** The
   strictness is deliberate. Loosening it admits false negatives, and a
   genuinely undefined rule passing unnoticed is worse than a known-good one
   sitting in a baseline.

---

## What to build

Support the four forms the document legitimately uses:

**Form A — canonical.** The id opens a bullet.

    * `[RULE-1]` ...

**Form B — sharing a structural context.** A second rule stated on the same
bullet or line as the first, typically after a semicolon.

    * `[RULE-A]` ...; `[RULE-B]` ...

**Form C — inline in a paragraph.** No bullet at all.

    `[RULE-C]` requires ...

**Form D — parenthetical, or granted by name.** Includes `[RC-2]`'s sentence
*"Its lettered clauses are individually citable as `[RC-2a]`..`[RC-2d]` in the
order written"*, which grants four ids explicitly and which no structural test
can currently see.

    ... (`[RULE-D]` ...)

### The constraint that makes this hard, and it is the whole point

> **Recognising additional syntactic forms must not weaken the
> definition/reference distinction.**

The extractor must decide whether the surrounding text *defines* the rule or
merely *mentions* it — not whether the id appears in a newly permitted position.
Form B is exactly where that bites: `[TYP-25]` ... `[TYP-26]` is two definitions
on one line, while `[ATT-2]` naming `E0104` which `[ATT-1]` defines is a
reference. Both look alike structurally.

Existing signal worth reusing: `REFERENCE_LEAD`, `REFERENCE_TAIL` and
`DEFINITION_TAIL` in `rule_index.py` already classify what follows an id.
Extending those to the new positions is more likely to be right than relaxing
the position test.

**Acceptance:** the six leave `dangling_references` in
`rule_index_baseline.json` (the ratchet shrinks, which is always permitted),
and **no id currently classified as a reference becomes a definition.** The
second half is the part to test hardest — introduce a deliberate false
definition and confirm the extractor still refuses it.

---

## The longer-term direction

Relying entirely on Markdown structure to carry normative ownership is the root
cause. A future revision could give the specification an explicit
machine-readable annotation, so rule ownership is unambiguous without changing
the human-readable text:

    <!-- ember-rule: TYP-26 -->

or equivalent structured metadata.

**This is a tooling and documentation evolution and must not be forced into
0.8.5 to close a queue item.** It touches every Part, it needs its own design
pass on what the annotation covers (a rule, a clause, an amendment), and doing
it under time pressure to make a baseline shrink is precisely the wrong reason.

---

## Until then

The six stay exactly as written. The baseline records them. Withdrawn ERR-042
carries the full twelve-item inventory of what every dangling reference actually
is — four in Form B, two in Form C, two granted by `[RC-2]`, three deliberately
reserved by Part XXI for the language server, and one historical citation to a
rule family a later revision replaced.

**Nothing here blocks implementation, conformance, or a specification freeze.**
