# Owner decision queue

Questions an agent may not answer, in the format `SPEC-FEED-0.8.5-H1` §28 asks
for. **Nothing here has been resolved silently.** Each entry says why it cannot
be settled without choosing between competing semantics or reversing an owner
ruling.

Closed entries stay, with the ruling, because a closed question is evidence
about what kind of question this project generates.

---

## ODR-001 — `UnsafeCell`'s API surface: the feed and a prior ruling disagree

    ID:        ODR-001
    Location:  [UNS-10], Part IX §4; docs/spec-amendments.md S2; ADR-022
    Status:    OPEN

**Existing wording.** `[UNS-10]` names the API: *"`UnsafeCell(owned v: T)`,
`get(self) -> *mut T` — which needs an `unsafe` context, being a raw pointer
under `[UNS-1]` — and `into_inner(owned self) -> T`, which is safe because the
cell is consumed and nothing is shared."*

**Conflict.** `SPEC-FEED-0.8.5-H1` §8 states that *"the current specification
deliberately leaves exact raw-pointer API naming for a later specification
revision"* and instructs: *"Do NOT invent API names during this feed."* The
document does name them. It is not an invention — the API surface and the module
were put to the owner as an explicit question on 2026-09-10, precisely because
both change the accepted program set, and the owner chose **`std.mem` with a
raw-pointer accessor**. So the feed's premise and the prior ruling describe
different documents.

**Possible interpretations.**

1. **The 2026-09-10 ruling stands and §8's premise is stale.** The names are
   owner-chosen, `[UNS-10]` is normative as written, and nothing moves.
2. **The names are illustrative, not normative.** `[UNS-10]` keeps the semantics
   as normative and marks the specific spellings non-normative until a later
   revision fixes them.
3. **The names are withdrawn** and `[UNS-10]` states semantics only, with the
   surface deferred.

**Semantic impact.** Real under 2 and 3. The API surface determines which
programs compile; withdrawing or demoting it makes `UnsafeCell` unimplementable
until a later revision, which affects `[TST-4]` coverage and any package
planning to build on it.

**Recommended.** Interpretation **1**. The ruling was explicit, recent, and was
obtained by asking rather than inferring — which is the route this project
requires. §8 appears to have been written without that exchange in view.

**Why an agent may not settle it.** Choosing 2 or 3 would reverse a decision the
owner made four hours earlier; choosing 1 would dismiss a written instruction in
the feed. Either way an agent would be picking between two owner statements.

---

## ODR-002 — six rule definitions the checker cannot see

    ID:        ODR-002
    Location:  [TYP-26], [IFC-2], [HND-2], [GPU-7] (each the second rule on a
               line shared with its predecessor); [VER-7] (inline in a
               paragraph); [CTL-3a] (inline in a parenthetical)
    Status:    OPEN — raised 2026-09-10, deliberately not acted on

**Existing wording.** Each of the six states its rule in full. `[TYP-26]`, for
example: *"two functions with the same name in one scope is `E1030` — except
operator interface impls and `extend` blocks for distinct types."*

**Conflict.** `rule_index.py` recognises a definition structurally: the id must
open a bullet, and **only the first id on a bullet counts**. Its own comment
says why — *"a later one is cited by it, even when the citation reads like a
rule. This is `[XXII.4]`'s 'a reference is not a definition' made mechanical."*
So six properly-stated rules read to the tool as dangling references and sit in
a baseline.

**Possible interpretations.**

1. **Split each onto its own bullet, text verbatim.** `EDITORIAL REPAIR`,
   changes no meaning, and the six leave the baseline.
2. **Teach the detector the inline and granting-sentence forms.** No document
   edit, but it admits false negatives — a genuinely undefined rule could then
   pass, which is worse than a known-good one sitting in a baseline.
3. **Leave both.** The inventory in withdrawn ERR-042 records what they are, and
   the baseline entries stay.

**Semantic impact.** None under any option. This is discoverability.

**Recommended.** **1** if the owner is willing to have prose relaid out; **3**
otherwise. Not **2** — the strictness is deliberate and correct.

**Why an agent may not settle it.** Option 1 is a layout edit to the owner's own
prose for a tool's benefit, which is the wrong direction by this project's
cardinal rule that the tool moves and the document does not.

---

## ODR-003 — the `[FFI-17]` numbered list: keep in step, or delete the duplicates

    ID:        ODR-003
    Location:  Part XVI §7a, the numbered list under `[FFI-17]`
    Status:    OPEN — the document raises it against itself

**Existing wording.** The list is already marked *"`NON-NORMATIVE` under
`[CAT-1]`: where it and a rule disagree, the rule governs, and the rule is named
in each item"*, and it records that it *"has been the site of four
contradictions with the rules beside it (`std::function`, `std::string_view`,
C++ inheritance, and the CRT device attributed to `[FFI-30]`), because a prose
restatement of a rule drifts from it and nothing detects that."* It then says:
*"A future revision should delete from the list every claim a rule already makes
rather than keep two copies in step."*

**Conflict.** None outstanding — the drift is neutralised by the demotion, so
`SPEC-FEED-0.8.5-H1` §14's requirement ("the tables and explicit FFI rules are
authoritative") is already met. The open question is the document's own
recommendation to delete the duplicated prose.

**Semantic impact.** None if done correctly; the risk is losing explanatory
material that is not duplicated, which is only visible item by item.

**Recommended.** Defer to a revision that can do it item by item with the tables
open beside it. It is not a consistency fix, it is an editorial project.

**Why an agent may not settle it.** Deleting normative-adjacent prose in a pass
whose first constraint is "do not silently redesign" trades a known-safe state
for an information-loss risk, on roughly thirty judgement calls.

---

## Closed

| ID | Question | Ruling |
|---|---|---|
| — | `RefCell[T]` and `Copy` | **Not `Copy`, move-only**, 2026-09-10. `[CELL-12]`, S3, ADR-021 |
| — | Cut `Hardened_2` or defer E5 | **Cut it**, 2026-09-10. Then superseded by 0.8.5 |
| — | ERR-043, `UnsafeCell` | **Retained as the lowest-level primitive**, 2026-09-10. S2, ADR-022 |
| — | ERR-041 / D5, `[FN-1]` | **The worked example governs**, 2026-09-10. `[FN-1a]`, S4. D5 closed, no code moved |
| — | ERR-042, nine undefined rule ids | **Inventory ordered**; it found zero gaps and the entry was withdrawn |
| — | Scope changes | **Permitted when justified, must be reported**, 2026-09-10. HANDOFF §0.0 I |
