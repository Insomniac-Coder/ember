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
| ODR-004 | **CLOSED** — owner supplied `[LT-8]`–`[LT-13]` | Specification source recovery / library API | — | **No** |
| ODR-005 | **CLOSED** — explicit all-mutable `_mut` helpers | Language / standard-library API | — | **No** — ruled 2026-09-12 |
| ODR-006 | **CLOSED** — mutable helper inputs are `mut` reborrows | Language / callable API | — | **No** — ruled 2026-09-12 |
| ODR-007 | **CLOSED** — mode vector is compiler-known `Callable` metadata | Language / callable abstraction | — | **No** — ruled 2026-09-12 |
| ODR-008 | **CLOSED** — `Arena` is a narrow `@borrows` provenance source | Language / region provenance | — | **No** — ruled 2026-09-12 |

**ODR-001 through ODR-003 were resolved by the owner on 2026-09-10.** ODR-002
is now fully closed because `RIDX-1` landed; ODR-003 remains deferred editorial
work and needs no semantic decision. ODR-004 was discovered during the 0.9.5
intake and closed when the owner supplied the missing definitions on 2026-09-12.
ODR-005 was then closed by the owner's explicit all-mutable helper ruling.

ODR-001, ODR-002, and ODR-004 through ODR-008 are closed; ODR-003 is deferred
editorial work with no semantic impact. **No owner semantic decision is
currently open.** H8 records the complete helper-mode and callable-abstraction
ruling; H9 records the Arena-backed return-provenance ruling. Full H8/H9
implementation and conformance remain outstanding, although the Arena core and
H9 wrapper-provenance path now have executable evidence. Those gaps are not
owner questions and did not block the now-closed D-042 compiler work.

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

## ODR-004 — `[LT-8]`–`[LT-13]` source recovery — **CLOSED**

    ID:        ODR-004
    Status:    CLOSED — owner supplied the missing definitions; incorporated
               in H5
    Category:  SPECIFICATION SOURCE RECOVERY / LIBRARY API
    Priority:  —
    Location:  supplied Ember_v0.9.5_Hardened_3_Implementation_Ready_Spec.md,
               lines 105, 304–323, 1,683 and 4,360; frozen H4 target; H5 §H14.3

    Resolution: Owner-supplied Hardened 14 definitions of LT-8 through LT-13
    Authority:  Owner message, 2026-09-12
    Revision:   Ember 0.9.5_Hardened_5
    ADR:        ADR-024
    Result:     Closed. H5 §H14.3 is the recovered definition site.

    Semantic impact:                  NONE — source recovery
    Blocks implementation:            NO
    Blocks conformance:               NO — the source-recovery issue is closed
    Blocks H4 identity freeze:         NO — H4 remains the frozen predecessor
    Blocks normative specification adoption: NO
    Requires owner semantic decision: NO

**Existing wording.** The supplied 0.9.5 document says the
`std.borrow.with_views2/3/4` helpers were introduced by 0.9_Hardened_14, remain
normative compatibility APIs, and are governed by `[LT-8]`–`[LT-13]`.
`[TST-16]` requires conformance tests for all six. The standard-library table
also cites `[LT-8]` through `[LT-12]`.

**Historical conflict.** No definition of any of the six rules existed in either supplied
0.9.5 file. H3 added a manifest row saying they were “recovered from Hardened
14”, but the definitions themselves are still absent. The available
0.9_Hardened_12 and 0.9_Hardened_13 files also contain none of them, and no
0.9_Hardened_14 source is available in the repository or supplied downloads.
The frozen H4 development target corrected the unsupported recovery claim.
The owner then supplied the six definitions; they were normalized only for
transport-corrupted Markdown and incorporated in H5 rather than editing H4.
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

**Resolution.** The recommended source-recovery path occurred. H5 preserves the
shared-`Span` signatures, late-bound independent regions, no-escape rule,
ordinary borrowing, `@noalloc` requirement, and persistent-aggregate boundary.
It also reconciles the recovered pre-0.9.5 `[LT-13]` wording with the later,
owner-approved multi-region `[LT-2]` revision without weakening either contract.

**Why an agent could not close the original gap.** Before the owner supplied
the source, function signatures, callback-result restrictions, escape behavior,
and allocation guarantees were absent. Each affected accepted programs and
borrow behavior. H5 now carries those answers. The exact mutable overload
surface remains separately isolated as ODR-005 rather than being guessed.

See `docs/MIGRATION-0.9.5.md` for the complete intake, H4 custody record, and H5
source-recovery classification.

---

## ODR-005 — exact mutable `with_views` overload surface — **CLOSED**

    ID:        ODR-005
    Status:    CLOSED — explicit all-mutable `_mut` family selected
    Category:  LANGUAGE / STANDARD-LIBRARY API
    Priority:  —
    Location:  Ember_v0.9.5_Hardened_6.md §H14.3, [LT-8] and [LT-11]; [TST-16]

    Resolution: Explicit all-mutable with_views2_mut/3_mut/4_mut helpers
    Authority:  Owner ruling, 2026-09-12
    Revision:   Ember 0.9.5_Hardened_6
    ADR:        ADR-025
    Result:     Closed. H6 [LT-8]/[LT-11] are authoritative within the
                development target.

    Semantic impact:                  Owner-approved API clarification
    Blocks implementation:            NO
    Blocks conformance:               NO — the decision is complete; evidence remains
    Blocks H5 identity freeze:         NO
    Blocks normative specification adoption: NO
    Requires owner semantic decision: NO

**Existing wording.** `[LT-8]` gives structural signatures only for shared
`Span[A]` inputs and callbacks. `[LT-11]` nevertheless requires two mutable
inputs whose sources may alias to be rejected, and `[TST-16]` requires mutable
alias-rejection coverage.

**Historical conflict.** The safety rule was clear, but the callable API that exposes mutable
inputs was not. The recovered source did not state whether mutable helpers exist,
which arguments may be `MutSpan`, whether shared/mutable combinations are one
overload family, or what exact callback types result. Inventing those signatures
would change the accepted standard-library API.

**Possible interpretations.**

1. `with_views2/3/4` have an overload family covering every `Span`/`MutSpan`
   combination, with callback mutability matching each input.
2. Only particular mutable combinations are provided; the owner supplies the
   exact matrix.
3. The public helpers are shared-`Span` only; `[LT-11]` is a general safety
   boundary for any later or implementation-private mutable form, and TST-16's
   mutable-helper requirement should be corrected.

**Resolution.** The owner selected distinct `with_views2_mut`,
`with_views3_mut`, and `with_views4_mut` helpers whose inputs and callback
parameters are all `MutSpan`. H6 uses Ember's canonical spelling `MutSpan[T]`;
the ruling's `SpanMut[T]` spelling was a terminology mismatch and did not create
a second type. The original shared helpers stay shared. No mixed `Span`/
`MutSpan` overload family is specified. `[LT-11]` binds the mutable family to
ordinary exclusive borrowing and alias rejection.

**Why owner resolution was required.** The alternatives accepted different
programs and exposed different mutation capabilities. The existing borrow rules
could determine whether a selected API was safe, but could not choose which API
Ember promised.

---

## ODR-006 — mutable helper and callback parameter modes — **CLOSED**

    ID:        ODR-006
    Status:    CLOSED — mutable helper inputs are mut reborrows
    Category:  LANGUAGE / CALLABLE API
    Priority:  —
    Location:  Ember_v0.9.5_Hardened_8.md §H14.3 [LT-8]/[LT-8a]/[LT-11];
               Part III fn_type; [FN-1]/[FN-2a]; [FN-6a]

    Resolution: Mutable helper inputs use mut; no helper input is owned
    Authority:  Owner rulings, 2026-09-12; ADR-026 / ADR-027
    Revision:   Ember 0.9.5_Hardened_8
    Result:     Closed. H8 [LT-8] is authoritative within the development
                target.

    Semantic impact:                  YES — callability and mutation authority
    Blocks implementation:            NO — implementation remains to be built
    Blocks conformance:               NO — evidence remains to be added
    Blocks H8 identity freeze:         NO
    Blocks normative specification adoption: NO
    Requires owner semantic decision: NO

**Resolved portion.** H7 extends `fn_type` with borrowed/default, `mut`, and
`owned` modes under ordinary `[FN-1]`/`[FN-2]` semantics. The `_mut` callbacks
now explicitly use `fn(mut MutSpan[A], ...)`; `[LT-11a]` and `[TST-20]`
make the mode mismatch testable. This closes the callback side of the question.

**Historical remaining conflict.** The H7 helper functions' own inputs were written
`a: MutSpan[A]`, not `mut a: MutSpan[A]` or `owned a: MutSpan[A]`. `[FN-1]`
makes that an ordinary shared borrow which cannot itself be mutably borrowed or
moved when the helper invokes a callback requiring `mut MutSpan[A]`. The owner
ruling changed callback modes but did not state the helper input modes.

**Possible interpretations.**

1. Spell every helper input `mut a: MutSpan[A]`. This is the direct application
   of `[FN-1]`/`[FN-1a]` and is recommended if the helper reborrows the caller's
   mutable view for the callback.
2. Spell every helper input `owned a: MutSpan[A]`, consuming the view capability
   into the helper before it passes/reborrows it. This changes caller usability.
3. Define a narrow rule that an unmarked `MutSpan` parameter may mutate its
   referent. This conflicts with the general shared-parameter rule and risks an
   aliasing exception; it is not recommended.

**Former recommended owner action.** Select `mut` or `owned` for the helper's own
`MutSpan` inputs. Prefer `mut` if the intent is invocation-scoped reborrowing
that leaves the caller's view usable afterward. Do not choose option 3 without
explicitly amending `[FN-1]` and the safety argument.

**Why this cannot be resolved safely by an agent.** The alternatives change
ownership, post-call usability, and which arguments can be passed. The H7
callback-mode ruling does not select the helper's own mode.

**Owner resolution.** Every mutable helper input is explicitly `mut`; shared
helper inputs retain the default borrowed mode; neither family consumes a view.
This is invocation-scoped reborrowing, so caller usability resumes when the
helper returns. H8 applies the ruling and `[TST-21]` makes it testable. The
supplied `SpanMut[T]` and `[TST-LT-MODE]` spellings are normalized to canonical
`MutSpan[T]` and the next unused numeric rule ID, `[TST-21]`.

---

## ODR-007 — mode-bearing `fn` types and `Callable[Args, R]` — **CLOSED**

    ID:        ODR-007
    Status:    CLOSED — mode vector is compiler-known canonical metadata
    Category:  LANGUAGE / CALLABLE ABSTRACTION
    Priority:  —
    Location:  Ember_v0.9.5_Hardened_8.md [FN-6a], [CLO-3], [CLO-6];
               std.core Callable/CallableOnce declarations

    Resolution: Preserve modes internally through existing Callable[Args, R]
    Authority:  Owner ruling, 2026-09-12; ADR-027
    Revision:   Ember 0.9.5_Hardened_8
    Result:     Closed. H8 [FN-6a] is authoritative within the development
                target.

    Semantic impact:                  YES — callable compatibility and dispatch
    Blocks implementation:            NO — implementation remains to be built
    Blocks conformance:               NO — TST-20/TST-21 evidence remains
    Blocks H8 identity freeze:         NO
    Blocks normative specification adoption: NO
    Requires owner semantic decision: NO

**Existing wording.** H7 makes parameter modes part of callable types and says
they have ordinary ownership semantics. `[CLO-3]` still says `fn(A) -> R`
denotes an implicit generic bound `Callable[(A), R]`, while `std.core` declares
`Callable[Args, R]` and `CallableOnce[Args, R]`. `Args` is an ordinary tuple of
types and carries no mode vector.

**Conflict.** `fn(A)`, `fn(mut A)`, and `fn(owned A)` must be distinguishable
for compatibility and invocation, but the documented `Callable[Args, R]` bridge
maps all three to the same `Args = (A)` surface. The owner explicitly rejected
a separate callable ownership model, so an agent cannot add an unrelated
parallel interface ad hoc.

**Possible interpretations.**

1. Make the mode vector compiler-known metadata on the implicit callable bound;
   `Callable[Args, R]` remains the source spelling but `[CLO-3]` defines its
   mode-bearing use precisely.
2. Change the public interface shape so the complete `fn(...) -> R` signature,
   including modes, is the generic argument to `Callable`/`CallableOnce`.
3. Add a separate mode-vector generic parameter or family of callable
   interfaces. This is explicit but expands the public standard-library model.

**Former recommended owner action.** Prefer option 1 if preserving the current public
`Callable[Args, R]` spelling is important; specify how conformance and method
matching observe the compiler-known mode vector. Otherwise choose an exact
public interface signature. In every case, mode mismatches must remain type
errors and no runtime mode dispatch should be introduced.

**Why this cannot be resolved safely by an agent.** The options alter interface
identity, generic bounds, coherence, closure matching, dynamic callable
compatibility, and possibly ABI/interface hashes. `[FN-6]` establishes required
behavior but does not choose this bridge representation.

**Owner resolution.** Option 1 governs. The existing public
`Callable[Args, R]` abstraction remains, while the compiler's canonical
callable type carries the complete borrowed/`mut`/`owned` mode vector through
generic bounds, type and borrow checking, overload resolution, and
monomorphisation. The metadata is compile-time-only and introduces neither
runtime bookkeeping nor a second ownership system. `[FN-6a]`, `[LT-8a]`, and
`[TST-21]` record the contract in H8.

---

## ODR-008 — Arena-backed return provenance — **CLOSED**

    ID:        ODR-008
    Status:    CLOSED — `Arena` is a narrow `@borrows` provenance source
    Category:  LANGUAGE / REGION PROVENANCE
    Priority:  —
    Location:  Ember_v0.9.5_Hardened_9.md [LT-1a], [LT-4], [LT-4a], [LT-4b]

    Resolution: Permit @borrows(arena) only for Arena-backed returned views
    Authority:  Owner ruling, 2026-09-12; ADR-028
    Revision:   Ember 0.9.5_Hardened_9
    Result:     Closed. H9 [LT-4a]/[LT-4b] are authoritative within the
                development target.

    Semantic impact:                  YES — wrapper signatures and accepted programs
    Blocks implementation:            NO — ruling received and implementation started
    Blocks conformance:               NO — TST-22 evidence can now be built
    Blocks H9 identity freeze:         NO
    Blocks normative specification adoption: NO
    Requires owner semantic decision: NO

**Historical conflict.** `[LT-4]` tied every arena allocation to the Arena
borrow, while `[LT-1a]` rejected an Arena parameter in the only public
return-provenance annotation because Arena is not a view type. A safe wrapper
could therefore neither express the relationship nor omit it soundly.

**Owner resolution.** A function whose returned view is proven to derive from
storage owned by a growing `Arena` parameter writes `@borrows(arena)`. This is
a provenance-only exception. Arena stays non-view; arbitrary non-view
parameters, unrelated views, owned results, lifetime extension, ownership
transfer, and borrow-checker bypass remain forbidden. Nested wrappers repeat
the annotation. Omitted provenance is E3061; false provenance remains E3062;
an unrelated non-view parameter remains E2031.

**Normalization.** H9 uses the existing `alloc`/`alloc_array` API rather than
the ruling's otherwise-undefined illustrative `alloc_span` names, and records
the mnemonic conformance request as numeric `[TST-22]`.

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
| **ODR-004** — missing `[LT-8]`–`[LT-13]` source | Owner supplied the Hardened 14 definitions | Owner message 2026-09-12; **ADR-024** | 0.9.5_Hardened_5 | Closed; H5 §H14.3 is authoritative within the development target |
| **ODR-005** — mutable `with_views` API | Explicit all-mutable `_mut` helper family using `MutSpan[T]` | Owner ruling 2026-09-12; **ADR-025** | 0.9.5_Hardened_6 | Closed; no mixed-mutability overloads are implied |
| **ODR-006** — helper/callback modes | Mutable helper inputs are `mut` reborrows; callback modes remain explicit | Owner ruling 2026-09-12; **ADR-026 / ADR-027** | 0.9.5_Hardened_8 | Closed; no helper consumes an input view |
| **ODR-007** — `fn`/`Callable` mode bridge | Preserve the complete mode vector as compiler-known canonical type metadata | Owner ruling 2026-09-12; **ADR-027** | 0.9.5_Hardened_8 | Closed; no runtime mode bookkeeping or second ownership system |
| **ODR-008** — Arena-backed return provenance | Permit narrow `@borrows(arena)` only for a view backed by that Arena | Owner ruling 2026-09-12; **ADR-028** | 0.9.5_Hardened_9 | Closed; Arena remains non-view and arbitrary non-view parameters remain forbidden |
