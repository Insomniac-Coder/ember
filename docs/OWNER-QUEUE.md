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
| ODR-001 | **CLOSED** — retain the API exactly | Language / API | — | **No** — ruled 2026-09-10 |
| ODR-002 | **CLOSED** — `RIDX-1` landed | Tooling / document structure | — | **No** |
| ODR-003 | **DEFERRED EDITORIAL CLEANUP** — open for a future revision | Editorial / documentation | **P3** | **No** |
| ODR-004 | **OPEN** — recover missing `[LT-8]`–`[LT-13]` definitions | Specification source recovery / library API | **P1** | **Conditional** — yes if the omitted source cannot be recovered |

**ODR-001 through ODR-003 were resolved by the owner on 2026-09-10.** ODR-002
is now fully closed because `RIDX-1` landed; ODR-003 remains deferred editorial
work and needs no semantic decision. ODR-004 was discovered during the 0.9.5
intake and remains the only conditional owner question.

ODR-001 through ODR-003 do not block anything. **ODR-004 blocks normative
adoption of the 0.9.5 target and conformance for the `std.borrow.with_views`
surface.** It does not block H4's frozen identity or unrelated compiler work
such as D-042.

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
That was the correct call at the time.

### The owner's rationale, 2026-09-10 — why the API stays exactly as it is

> *"There is no benefit in reopening this."*

The design direction Ember has accepted is **safe by default, but capable of
expressing low-level systems mechanisms when the programmer explicitly crosses a
well-defined unsafe boundary.** `UnsafeCell` fits that precisely. It is not an
abstraction for ordinary Ember code; it is the floor underneath the higher-level
interior-mutability mechanisms:

    Safe code
       │
       ├── Cell
       ├── RefCell
       ├── Mutex
       └── RwLock
              │
              ▼
          UnsafeCell
              │
              ▼
         unsafe / raw pointer

**`UnsafeCell` does not weaken Ember's global safety model.** It is a narrowly
scoped escape hatch whose obligations stay explicit — which is what `[UNS-10a]`
and `[UNS-10b]` exist to state. Removing or demoting the API would make the
language *less* implementation-ready with no corresponding design benefit.

**No further action required.**

---

## ODR-002 — six rule definitions the extraction tool could not see — **CLOSED**

    ID:        ODR-002
    Status:    CLOSED — RIDX-1 landed in 6c77723
               No owner semantic decision required; no language change made.
    Category:  TOOLING / DOCUMENT STRUCTURE
    Priority:  P2
    Tracking:  RIDX-1 in docs/BACKLOG.md; design in docs/RFC-rule-extraction.md
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

### The owner's resolution, 2026-09-10 — the tool is the defective component

> *"If a tool cannot correctly recognize a valid normative rule, the tool is the
> defective component, assuming the specification's structure is itself valid
> and intentional."*

**Keep the six rules exactly as they are. Do not restructure their normative
wording. Do not weaken the extractor. Improve the tool instead**, as separate
work: `RIDX-1`, designed in `docs/RFC-rule-extraction.md`.

**Resolution evidence.** `tools/rule_index.py` now recognizes the four
documented definition forms, and `tools/test_rule_index.py` includes positive
cases plus a `FalseDefinitionsRefused` suite. The dangling-reference baseline
fell from 12 to 4; the remaining four are genuine references/reservations, not
definitions. The specification did not move. Commit `6c77723` landed the
completed implementation and its tests.

The dependency that must not exist, and the architecture that must:

    Specification              ✗                    Specification ───┬── compiler
          ↓  must conform to                                         ├── conformance suite
    current tooling                                                  ├── rule extractor
                                                                     ├── diagnostics tooling
                                                                     └── documentation tooling

The RFC records the four legitimate forms the extractor should learn — canonical
bullet, a rule sharing a structural context, inline in a paragraph, and
parenthetical or granted by name — under one constraint that is the whole
difficulty: **recognising additional syntactic forms must not weaken the
definition/reference distinction.** The tool must judge whether the surrounding
text *defines* the rule, not whether the id sits in a newly permitted position.

Longer term, an explicit machine-readable annotation (`<!-- ember-rule: TYP-26 -->`
or equivalent) would carry rule ownership without relying on Markdown structure
at all. **That is a tooling and documentation evolution and must not be forced
into 0.8.5 to close a queue item.**

---

## ODR-003 — the `[FFI-17]` numbered list: keep in step, or delete the duplicates

    ID:        ODR-003
    Status:    DEFERRED EDITORIAL CLEANUP (owner, 2026-09-10).
               No semantic change required for 0.8.5.
               Technically OPEN for a future revision.
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

### The owner's resolution, 2026-09-10 — and the method for the next revision

**Do not simply delete the list.** Classify every item against its authoritative
rule or table, and act per category:

| | If the item | Then |
|---|---|---|
| **A** | merely restates a normative rule | **delete it** — the rule already exists |
| **B** | gives rationale, examples, migration advice or context absent from the rule | **keep**, clearly labelled explanatory / non-normative |
| **C** | is a useful quick reference | keep **only** if it can be mechanically generated from the authoritative rules, or mark it plainly as a non-authoritative summary |
| **D** | conflicts with the normative rule | **delete or rewrite immediately** — the rule wins |

The target shape, and why two representations is bad architecture even when the
second is marked non-normative:

    Normative FFI rules          rather than       Normative rules
            │ authoritative                              ↕
            ▼                                     duplicated prose
      Single source of truth                            ↕
            │                                         tables
            ├── generated summary
            └── explanatory prose (non-normative)       ⇒ drift is inevitable

That is exactly the drift the four recorded contradictions came from. A hardened
specification should eliminate the second source of truth, not keep two copies
in step — but the elimination is item-by-item work for a suitable revision, not
a consistency pass.

---

## ODR-004 — `[LT-8]`–`[LT-13]` are referenced but absent from the 0.9.5 source

    ID:        ODR-004
    Status:    OPEN — source recovery required before normative adoption;
               the repair must be issued as H5
    Category:  SPECIFICATION SOURCE RECOVERY / LIBRARY API
    Priority:  P1
    Location:  supplied Ember_v0.9.5_Hardened_3_Implementation_Ready_Spec.md,
               lines 105, 304–323, 1,683 and 4,360; frozen H4 target front matter

    Semantic impact:                  CONDITIONAL
    Blocks implementation:            YES — only the LT-8..LT-13 helper surface
    Blocks conformance:               YES — TST-16 requires the absent rules
    Blocks H4 identity freeze:         NO — H4 is frozen as the target
    Blocks normative specification adoption: YES
    Requires owner semantic decision: YES if the original rules cannot be
                                      recovered verbatim

**Existing wording.** The supplied 0.9.5 document says the
`std.borrow.with_views2/3/4` helpers were introduced by 0.9_Hardened_14, remain
normative compatibility APIs, and are governed by `[LT-8]`–`[LT-13]`.
`[TST-16]` requires conformance tests for all six. The standard-library table
also cites `[LT-8]` through `[LT-12]`.

**Conflict.** No definition of any of the six rules exists in either supplied
0.9.5 file. H3 added a manifest row saying they were “recovered from Hardened
14”, but the definitions themselves are still absent. The available
0.9_Hardened_12 and 0.9_Hardened_13 files also contain none of them, and no
0.9_Hardened_14 source is available in the repository or supplied downloads.
The frozen H4 development target corrects the unsupported recovery claim.
ODR-004 still blocks normative adoption of that target and conformance for this
helper surface; the eventual repair must be issued as H5 rather than editing H4.
This is not the former ERR-042 tooling problem: there is no hidden inline,
heading, or shared-line definition for the extractor to discover.

**Possible interpretations.**

1. The six rule definitions exist in an omitted 0.9_Hardened_14 source and
   should be recovered verbatim. **Recommended.** This is source recovery and
   creates no new semantics.
2. The references are stale and the helper surface should be removed or
   deferred. That changes the claimed standard-library contract and needs the
   owner.
3. The helpers are intended, but the six rules were never written. Defining
   their exact signatures, mutability, escape, allocation, and region behavior
   is new normative work and needs the owner.

**Semantic impact.** The difference is observable: options 2 and 3 decide
whether programs naming these helpers are accepted and what borrow/escape
behavior they have. An implementation agent cannot reconstruct six APIs from
the phrases “allocation-free late-bound callback composition” and
“two/three/four-view callbacks” without inventing semantics.

**Recommended owner action.** Supply the exact 0.9_Hardened_14 definitions of
`[LT-8]`–`[LT-13]`. If no authoritative source exists, explicitly choose
whether the helpers are removed/deferred or specify the missing rules as a new
owner amendment.

**Why this cannot be resolved safely by an agent.** Function signatures,
mutable-view combinations, callback result restrictions, failure diagnostics,
and whether the 2/3/4 helpers are separate rules are all absent. Each affects
accepted programs and borrow behavior. Guessing would violate the standing
rule that the compiler implements the language rather than defining it.

See `docs/MIGRATION-0.9.5.md` for the complete intake and the non-semantic
consolidation defects already repaired in H4 without inventing the missing
helper semantics.

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
