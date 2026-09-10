# Owner decision queue

Questions an agent may not answer, in the format `SPEC-FEED-0.8.5-H1` §28 asks
for. **Nothing here is resolved silently.**

**This queue is not a second specification.** `docs/spec-source/ember-spec.md`
remains the sole normative language contract; an entry here describes a
*question*, and where it quotes the specification the specification governs.

Closed entries stay, with the authority that resolved them, because a closed
question is evidence about what kind of question this project generates — and
because a future agent who cannot find where a decision was made will reopen it.

*(Referred to as `OWNERQUEUE.md` in owner feedback; the file is
`docs/OWNER-QUEUE.md`.)*

---

## Queue summary

| ID | Status | Category | Priority | Semantic decision required |
|---|---|---|---|---|
| ODR-001 | **CLOSED** | Language / API | — | **No** — already ruled |
| ODR-002 | OPEN | Tooling / document structure | **P2** | **No** |
| ODR-003 | OPEN | Editorial / documentation | **P3** | **No** |

**Nothing in this queue blocks anything.** No open entry blocks implementation,
conformance, a specification freeze, or requires an owner semantic decision.

Priorities: **P1** blocks a language or implementation decision · **P2** changes
no language semantics but affects conformance or tooling confidence · **P3**
editorial cleanup that can safely wait.

---

## ODR-001 — `UnsafeCell`'s API surface — **CLOSED**

    ID:        ODR-001
    Status:    CLOSED — superseded by owner ruling S2 / ADR-022
    Category:  Language / API
    Location:  [UNS-10], Part IX §4; docs/spec-amendments.md S2; ADR-022

    Resolution: Owner-approved UnsafeCell API surface
    Authority:  Owner ruling 2026-09-10; amendment S2; ADR-022
    Revision:   Ember 0.8.5_Hardened_1
    ADR:        ADR-022
    Result:     Closed. `[UNS-10]` is authoritative and needs no further
                owner decision.

    Blocks implementation:            NO
    Blocks conformance:               NO
    Blocks specification freeze:      NO
    Requires owner semantic decision: NO

**What the specification says, and it is settled.** `[UNS-10]` defines the
surface: `UnsafeCell(owned v: T)`, `get(self) -> *mut T` — which needs an
`unsafe` context, being a raw pointer under `[UNS-1]` — and
`into_inner(owned self) -> T`, which is safe because the cell is consumed and
nothing is shared. It lives in `std.mem`, and it is the language's lowest-level
interior-mutability primitive with the safety boundary `[UNS-10a]` and
`[UNS-10b]` state.

**None of the following is an open question, and this entry must not be read as
though any of them were:** whether `UnsafeCell` exists; whether it lives in
`std.mem`; whether it has a raw-pointer accessor; whether that accessor requires
`unsafe`; whether `into_inner` exists; whether it is the lowest-level primitive.
All six are decided and normative in 0.8.5.

### Historical context — kept because the conflict is instructive

This entry was opened because `SPEC-FEED-0.8.5-H1` §8 stated that *"the current
specification deliberately leaves exact raw-pointer API naming for a later
specification revision"* and instructed *"Do NOT invent API names during this
feed."* The document did name them, which looked like a contradiction.

It was not one, and the sequence is the point:

1. **The feed's wording was stale.** It described the specification as it stood
   before the API surface was settled.
2. **The owner ruled.** The surface and the module were put to the owner as an
   explicit question on 2026-09-10 — asked precisely *because* both change the
   accepted program set — and the owner chose `std.mem` with a raw-pointer
   accessor. The names were never invented.
3. **The current specification incorporates that ruling** as `[UNS-10]`,
   declared as amendment S2, class `OWNER-APPROVED SEMANTIC CHANGE`, which is
   what made the language version 0.8.5.
4. **No further owner decision is required.**

The entry was raised rather than resolved because settling it either way would
have meant overriding one owner statement with another — the feed or the ruling.
That was the correct call at the time, and the owner has since confirmed the
ruling stands.

---

## ODR-002 — six rule definitions the extraction tool cannot see

    ID:        ODR-002
    Status:    OPEN — raised 2026-09-10, deliberately not acted on
    Category:  TOOLING / DOCUMENT STRUCTURE
    Priority:  P2
    Location:  [TYP-26], [IFC-2], [HND-2], [GPU-7] — each the second rule on a
               line shared with its predecessor
               [VER-7]  — stated inline in a paragraph
               [CTL-3a] — stated inline in a parenthetical

    Semantic impact:                  NONE
    Blocks implementation:            NO
    Blocks conformance:               NO
    Blocks specification freeze:      NO
    Requires owner semantic decision: NO

> **These are not believed to be undefined language rules. They are rules whose
> current document structure is not recognised as a definition by the extraction
> tool.**

That distinction is the whole entry. **ERR-042 investigated this as an
undefined-rule problem and was withdrawn as wrong**; the inventory it produced
is what established that all six are fully stated. Do not resurrect it. Reading
this entry as "six rules are missing" would repeat the exact error that entry
was withdrawn for.

**What is actually true.** Each of the six states its rule in full. `[TYP-26]`,
for example: *"two functions with the same name in one scope is `E1030` — except
operator interface impls and `extend` blocks for distinct types."* That is a
complete rule. `rule_index.py` does not see it because it recognises a
definition structurally — the id must open a bullet, and **only the first id on
a bullet counts** — and its own comment says why: *"a later one is cited by it,
even when the citation reads like a rule. This is `[XXII.4]`'s 'a reference is
not a definition' made mechanical."* All six sit in `rule_index_baseline.json`.

### Neither of these is permitted without a separate tooling review

1. **Changing the six rules' wording or layout merely to satisfy the
   extractor.** The specification is authoritative; a tool that cannot read it
   is the thing that is wrong. Relaying out owner prose for a tool's benefit is
   the wrong direction by this project's cardinal rule.
2. **Weakening the extractor so that it accepts ambiguous rule definitions.**
   The strictness is deliberate. Loosening it admits false negatives, and a
   genuinely undefined rule slipping through is worse than a known-good one
   sitting in a baseline.

**The preferred long-term direction is likely to improve the extractor** — to
teach it the inline and granting-sentence forms without loosening what counts as
a definition — but that is a tooling decision to be taken on its own terms, with
the false-negative risk priced, and not a side effect of a documentation pass.

---

## ODR-003 — the `[FFI-17]` numbered list: keep in step, or delete the duplicates

    ID:        ODR-003
    Status:    OPEN — the document raises it against itself
    Category:  EDITORIAL / DOCUMENTATION
    Priority:  P3
    Location:  Part XVI §7a, the numbered list under [FFI-17]

    Semantic impact:                  NONE
    Blocks implementation:            NO
    Blocks conformance:               NO
    Blocks specification freeze:      NO
    Requires owner semantic decision: NO

**There is no current semantic contradiction.** The list is already marked
*"`NON-NORMATIVE` under `[CAT-1]`: where it and a rule disagree, the rule
governs, and the rule is named in each item"*. So the normative FFI tables and
rules are authoritative today, and `SPEC-FEED-0.8.5-H1` §14's requirement is met
as the document stands.

**What remains is the document's own recommendation.** It records that the list
*"has been the site of four contradictions with the rules beside it
(`std::function`, `std::string_view`, C++ inheritance, and the CRT device
attributed to `[FFI-30]`), because a prose restatement of a rule drifts from it
and nothing detects that"*, and concludes: *"A future revision should delete from
the list every claim a rule already makes rather than keep two copies in step."*

**Explicitly not blocking:** compiler implementation, FFI conformance, ABI
implementation, RageV migration, or a language-version freeze. It is a future
cleanup opportunity and nothing more.

**Why it was not done in the 0.8.5 pass.** It is roughly thirty judgement calls
about which prose duplicates a rule and which carries explanation found nowhere
else. Deleting normative-adjacent prose in a pass whose first constraint is *do
not silently redesign* trades a known-safe state for an information-loss risk.
It wants a revision that can work item by item with the tables open beside it.

---

## Closed — the audit trail

Kept so that a future agent can find where each decision was made rather than
reopening it.

| Question | Resolution | Authority | Revision | Result |
|---|---|---|---|---|
| **ODR-001** — `UnsafeCell`'s API surface | Owner-approved: `std.mem`, raw-pointer accessor | Owner ruling 2026-09-10; **S2 / ADR-022** | 0.8.5_Hardened_1 | Closed; `[UNS-10]` is authoritative |
| `RefCell[T]` and `Copy` | Not `Copy`; move-only whatever `T` is | Owner ruling 2026-09-10; **S3 / ADR-021** | 0.8.5_Hardened_1 | `[CELL-12]` normative; `[CELL-4]` explicitly does not reach `RefCell` |
| Cut `Hardened_2`, or defer E5 | Cut it — E5 is editorial repair, not a revision | Owner ruling 2026-09-10 | 0.8.4_Hardened_2 | Cut, then superseded by 0.8.5 |
| ERR-043 — `UnsafeCell` undefined | Retained as the lowest-level interior-mutability primitive | Owner ruling 2026-09-10; **S2 / ADR-022** | 0.8.5_Hardened_1 | `[UNS-10]`/`[UNS-10a]`/`[UNS-10b]`; specified, unbuilt |
| ERR-041 / deviation D5 — `[FN-1]` | Part VII §7's worked example governs | Owner ruling 2026-09-10; **S4** | 0.8.5_Hardened_1 | `[FN-1a]`; D5 closed, **no code moved** |
| ERR-042 — nine "undefined" rule ids | Inventory ordered; it found **zero** gaps | Owner ruling 2026-09-10 | — | Entry **withdrawn as wrong**; see ODR-002 for what is actually true |
| Scope changes by an implementation agent | Permitted when justified; **must be reported** | Owner ruling 2026-09-10 | — | `docs/HANDOFF.md` §0.0 I |
