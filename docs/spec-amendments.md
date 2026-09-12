# Specification amendments

Every difference between `docs/spec-source/as-received/Ember_v0.8.3_spec.md`
and `docs/spec-source/ember-spec.md`, with the reason for it.

## The hardening protocol (owner, 2026-09-09)

Each normative specification and issued development target carries **two
numbers that move independently**:
`v0.8.4_Hardened_1`, `v0.8.4_Hardened_2`, and so on.

* The **language version** moves when the set of accepted programs changes.
  When it moves, the hardening number **resets to 1**.
* The **hardening number** moves when the document gains implementation detail
  and no rule changes meaning.

A hardening may never carry a semantic change; that is precisely what forces the
language number instead. Historically, the S1 cut was `0.8.4_Hardened_1` and
not `0.8.3_Hardened_2` for exactly that reason — S1 changed what Ember accepts,
so 0.8.3 could not absorb it and the hardening count started again. The current
repository-normative source is `0.8.5_Hardened_1`; the separately frozen
development target is `0.9.5_Hardened_9`.

**When to cut one.** Whenever building the compiler turns up a detail whose
absence made the work harder or produced a defect — found in discussion with the
owner, or by research of my own — that detail is written into the document and
the hardening number goes up by one. The trigger is discovering the gap, not
finishing a feature.

**What may go in.** Only what is needed to *implement v0.8.3's own goals*: a
rule's mechanism, an invariant an implementer must hold, a name a rule uses and
never declares, a production for syntax the document already writes. A hardening
never adds a feature, never relaxes a rule to match a compiler, and never
replaces owner text — it appends to it, and each addition is marked in place.

**What may not.** A genuine contradiction is not hardened — it is recorded here
and taken to the owner. Guessing wording and presenting it as the document's own
is the specific failure this whole file exists to prevent.

Text missing from the document altogether is the one case that can go either
way. If an authoritative earlier source carries it, restoring it verbatim is a
**recovery** and belongs in a hardening (A14 is one). If no source carries it,
it needs the owner's own words and nothing may be written in their place.

Each hardening gets a section below listing what it added and, for each, the
defect or difficulty that justified it.

### Issued hardenings become immutable development targets

**Owner ruling, 2026-09-12; ADR-023.** A bounded audit may reconcile several
non-semantic findings in one hardening pass. When that pass is issued, its
`Hardened_N` number advances and the resulting artifact becomes the development
target. It is frozen under that identity. A flaw discovered after issuance is
repaired in `Hardened_(N+1)`, which then becomes the target; the predecessor is
never silently edited.

This target-selection rule is separate from normative repository adoption. A
target may be frozen while a recorded source, tooling, or conformance blocker
prevents it from replacing `docs/spec-source/ember-spec.md`. That is the current
0.9.5 state: H3 was received unchanged, the bounded correction pass is frozen
as H4, the owner-supplied `[LT-8]`–`[LT-13]` source recovery is frozen as H5,
the mutable-helper API resolution is frozen as H6, and the owner's callable-
parameter-mode ruling is frozen as H7. The helper-input and `Callable` bridge
ruling is frozen as H8. The owner-approved Arena-backed return-provenance
clarification is frozen as H9. ODR-004 through ODR-008 are closed.
`docs/MIGRATION-0.9.5.md` inventories every H3→H4 through H8→H9 correction;
this ledger continues to declare
differences in the currently adopted repository-normative source checked by
`tools/hardening_check.py`.

### HC-095-04 — H5 source recovery and supersession reconciliation

**Class: SOURCE RECOVERY**, with a **SEMANTICALLY NEUTRAL CLARIFICATION** of
revision interaction. On 2026-09-12 the owner supplied the missing Hardened 14
definitions of `[LT-8]`–`[LT-13]`. H5 restores their shared-`Span` helper
signatures, late-bound independent regions, no-escape requirement, ordinary
borrowing rule, `@noalloc` guarantee, and then-current persistent-aggregate
boundary. Transport-only HTML entities, escaped punctuation, and malformed
code fences were normalized; no rule substance was reconstructed.

The recovered `[LT-13]` predates 0.9.5 and describes the workaround boundary
under the former single-region `[LT-2]`. H5 explicitly states that the later,
owner-approved 0.9.5 multi-region-view rule supersedes only that historical
restriction. The callback helpers, `[LT-10]`, `[TYP-15]`, `[TYP-15a]`, and
ordinary borrowing remain unchanged.

**No semantic API completion by inference.** `[LT-8]` supplies only shared
`Span` signatures, while `[LT-11]` and `[TST-16]` mention mutable inputs. H5
does not invent `MutSpan` overloads. At H5 freeze, ODR-005 asked the owner for
the exact public surface because different overload matrices accept different
programs; HC-095-05 records the later resolution.

**No compiler code moved.** This amendment creates the frozen development
target `Ember_v0.9.5_Hardened_5.md`; it does not install H5 as
`docs/spec-source/ember-spec.md` and claims no implementation or conformance.

### HC-095-05 — H6 mutable `with_views` API resolution

**Class: SEMANTICALLY NEUTRAL CLARIFICATION, explicitly owner-approved.** The H5
recovery made an existing boundary visible: `[LT-8]` exposed only shared
`Span` signatures while `[LT-11]` and `[TST-16]` required mutable-input
behavior. The owner resolved ODR-005 on 2026-09-12 by keeping the shared family
and adding explicit all-mutable `with_views2_mut`, `with_views3_mut`, and
`with_views4_mut` helpers.

The mutable signatures use Ember's canonical `MutSpan[T]` spelling. The
owner's patch wrote `SpanMut[T]`; every existing type rule, grammar discussion,
and example uses `MutSpan[T]`, so H6 normalizes the name without creating a new
type. Mixed shared/mutable overloads were not approved and are not inferred.

`[LT-11]` now ties the `_mut` family explicitly to ordinary exclusive-borrow
rules. The decision adds no ownership mechanism, alias exception, hidden
allocation, or escape path. Because H5 was frozen, this owner-approved
clarification is issued as `Ember_v0.9.5_Hardened_6.md`. It closes the helper-
family question without changing the repository-normative 0.8.5 program set.
The subsequent audit found that `fn_type` cannot encode callback parameter
modes while `[FN-1]` defaults unmarked parameters to shared borrowing. ODR-006
records that separate issue; H6 does not infer its answer. No compiler code
moved and no conformance is claimed.

### HC-095-06 — H7 callable parameter-mode boundary clarification

**Class: SEMANTICALLY NEUTRAL CLARIFICATION, explicitly owner-approved.** H6
selected an all-mutable helper family, but its callback types could not state
the authority `[LT-11]` required: `fn_type` accepted only types, while
`[FN-1]` makes an unmarked parameter a shared borrow. The owner resolved the
callback half on 2026-09-12 by requiring callable types to use the same
borrowed/default, `mut`, and `owned` parameter modes as ordinary function
declarations.

H7 adds `fn_type_param` to the grammar and expands `[FN-6]`; the modes preserve
ordinary ownership, place, move, mutability, lifetime, and FFI-safety rules and
do not create a parallel callable ownership model. The mutable helpers'
callbacks now spell `fn(mut MutSpan[...])`. `[LT-11a]` requires a parameter-
mode mismatch diagnostic, and `[TST-20]` makes the shared family, mutable
family, alias rejection, callback authority, and ordinary callable modes
traceable to conformance. The supplied `SpanMut[T]` spelling is normalized to
the already-defined `MutSpan[T]`, as in H6. The supplied mnemonic
`[TST-LT-MUT]` is normalized to the next unused numeric test-rule ID because
the normative rule-index grammar requires a numeric suffix.

**Two owner boundaries remain explicit.** The ruling did not specify whether
the helper functions' own `MutSpan` inputs are `mut` or `owned`, so ODR-006 is
only partially resolved. It also did not define how `[CLO-3]`'s
`Callable[Args, R]` bridge preserves the newly normative mode vector; ODR-007
records that question. H7 neither invents those semantics nor claims the
affected surface implementation-ready.

Because H6 was frozen, ADR-023 requires this clarification to be issued as
`Ember_v0.9.5_Hardened_7.md`. It changes neither the adopted 0.8.5 source nor
the 0.8.5 implementation. No compiler code moved and no 0.9.5 implementation
or conformance is claimed.

### HC-095-07 — H8 helper input modes and `Callable` mode preservation

**Class: SEMANTICALLY NEUTRAL CLARIFICATION, explicitly owner-approved.** H7
made callable parameter modes explicit but left two API boundaries for the
owner. On 2026-09-12 the owner selected `mut`, not `owned`, for every mutable
helper input and selected compiler-known canonical type metadata for preserving
the complete mode vector through the existing `Callable[Args, R]` abstraction.

H8 makes shared helper inputs ordinary borrowed parameters and mutable helper
inputs explicit `mut` reborrows. Neither family consumes an input view.
`[LT-10]` states that helpers neither own source storage nor extend its
lifetime; `[LT-8a]` prohibits callback-mode erasure through `Callable`.
`[FN-6a]` preserves modes through generic bounds, type and borrow checking,
overload resolution, and monomorphisation, while requiring compile-time-only
metadata and no second ownership system or runtime mode bookkeeping.

`[TST-21]` records conformance for helper modes, caller usability, generic
forwarding, and mismatch/erasure rejection. As in H6/H7, the supplied
`SpanMut[T]` spelling is normalized to canonical `MutSpan[T]`. The supplied
mnemonic `[TST-LT-MODE]` is normalized to the next unused numeric test-rule ID
because it would otherwise be invisible to the normative rule index.

Because H7 was frozen, ADR-023 requires this ruling to be issued as
`Ember_v0.9.5_Hardened_8.md`. ODR-006 and ODR-007 close. The adopted 0.8.5
source and implementation remain unchanged; no 0.9.5 implementation or
conformance is claimed.

### HC-095-08 — H9 Arena-backed return provenance

**Class: SEMANTICALLY NEUTRAL CLARIFICATION, explicitly owner-approved.** Implementing
`[LT-4]` exposed a boundary in `[LT-1a]`: allocation results carry the borrow
of an `Arena`, but the only public return-provenance annotation rejected every
non-view parameter. The owner resolved that boundary on 2026-09-12 with one
narrow exception. `@borrows(arena)` may name a growing `Arena` parameter only
when the returned view is proven to derive from storage owned by that arena.

H9 adds `[LT-4a]` and `[LT-4b]`. `Arena` remains a non-view, arbitrary
non-view parameters remain E2031, the annotation grants no lifetime extension
or ownership transfer, and omitted or false provenance is rejected. The
owner-supplied illustrative `alloc_span`/`alloc_mut_span` names were normalized
to the already-specified `alloc`/`alloc_array` surface rather than creating a
second Arena API. The supplied mnemonic `[TST-LT-ARENA-RETURN]` becomes the
next numeric conformance rule, `[TST-22]`.

Because H8 was frozen, ADR-023 requires this ruling to be issued as
`Ember_v0.9.5_Hardened_9.md`. H8 remains unchanged. H9 becomes the frozen
development target; it does not replace the adopted 0.8.5 normative source or
claim blanket 0.9.5 conformance.

### The four kinds, and what each is allowed to do

The owner set this model on 2026-09-09. Every finding sorts into exactly one
branch, and the branch decides who moves:

| Found | What happens | Who changes |
|---|---|---|
| **Missing or ambiguous documentation** | clarify what implementing it requires | the document |
| **Compiler deviation** | fix the compiler | the compiler |
| **Genuine spec defect** | take it to the owner | neither, until ruled |
| **Recovered historical text** | restore it exactly | the document, verbatim |

The fourth is the one to be careful with, because it looks like the first.
A recovery is only a recovery when the text is *found*, in an authoritative
earlier source, and is restored as it was written. The moment it is smoothed,
modernised or completed by inference it becomes a reconstruction — and a
reconstruction presented as a recovery is the worst outcome available here,
because it enters the document wearing the owner's voice.

So each entry below declares **one class**, and a hardening admits only four of
the five:

    SEMANTICALLY NEUTRAL CLARIFICATION   permitted
    IMPLEMENTATION INVARIANT             permitted
    SOURCE RECOVERY                      permitted
    EDITORIAL REPAIR                     permitted
    OWNER-APPROVED SEMANTIC CHANGE       NOT permitted — forces a language revision

The fifth is the line, and it is drawn in one sentence: **the moment an edit
answers *what Ember means* rather than *how to implement what Ember already
means*, it stops being a hardening.**

Two drafts crossed it and were withdrawn, both after the owner caught them and
not before:

* **A4** wrote a determination about `E3064` into `[LT-2]` — that no v1 source
  can demand two independent regions. It was even a correct determination, and
  it was still the wrong place for it: a conclusion in a rule is no longer a
  conclusion, it is the rule.
* **A6** wrote an unruled reading of `[FN-1]` into the rule. The reading may
  well be right; ERR-041 is open and it is the owner's to answer.

The pattern in both is the same and worth naming: I did the analysis correctly,
then put the answer in the normative text instead of the ledger. The class
labels exist to make that mechanically visible rather than a matter of my
judgement in the moment.

Each entry also records whether the compiler had to move, because an amendment
that forced a compiler change is a different kind of claim from one that
documented what was already there.

**These are the only permitted edits to the document.** The standing rule is
that the specification is the contract and is never altered to make the
compiler agree with it; where the two disagree, the compiler is what changes.
An entry belongs here only when the owner has authorised it explicitly, and
each says which instruction authorised it.

`as-received/` is never edited under any circumstance. `docs/spec/` is
generated from `ember-spec.md` and `tools/split_spec.py --check` fails CI if
anyone hand-edits it.

## History

**2026-09-09, morning — fourteen edits, all reverted.** Passages had been
rewritten in place so the document would agree with the readings recorded in
`docs/spec-errata.md`. That made the document and the errata agree at the cost
of the document no longer being the owner's, which is the wrong trade. Every
one was reverted; the file was byte-identical to as-received again
(`2bdffee6510b8668cf828185266efedb`) before the amendments below were applied.
The reverted set is listed in the commit *"Revert every edit to the
specification; the document is the owner's"*.

**2026-09-09, afternoon — thirteen clarifications and one recovery, below; two
drafts withdrawn (A4, A6).** Authorised by the owner:
*"update the v0.8.3 spec doc with the missing implementation details, if there
was ambiguity that led to these or if there was any missing details that would
have made implementation much straightforward"*, together with the specific
changes named in the combined fix list (items 1, 3, 6, 7, 8 and 17).

Each amendment **adds** to a rule and removes nothing — A14 is the exception, and
what it removes is a truncation. No new rule id is introduced, so
`tools/rule_index.py` reports the same 833 rules and no new conformance directory
is owed. Each is marked in the text, *(clarified …)* for a clarification and
*(head recovered verbatim …)* for the recovery, so a reader can tell owner text
from mine at a glance.

## Owner review of Hardened_1 (2026-09-09)

The owner reviewed the four files as a set — the original, Hardened_1, this file
and `DEVIATIONS.md` — and did not declare Hardened_1 clean. Three corrections
were required and are applied:

1. **A6 removed from the normative body.** It answered an unruled question; the
   same failure as A4. Withdrawn, and the compiler's behaviour is now carried as
   D5 in `DEVIATIONS.md` instead of being implied by a rule.
2. **A13 narrowed.** The semantic relationship stays; the requirement that an
   implementation share one internal capability is gone. A hardening constrains
   observable behaviour, not architecture.
3. **Bookkeeping.** "Fourteen clarifications" was thirteen plus one recovery,
   and the 826-vs-833 rule counts are now explained rather than left as two
   authoritative-looking numbers in one lineage.

**Second review, after 0.8.4_Hardened_1 was cut.** Two more, both about the
version rather than a rule:

1. **The header contradicted itself.** It still read "Source files still declare
   `#! language "0.8.3"`, because the language did not move — only the
   document's completeness did". True of 0.8.3_Hardened_1, and false the moment
   S1 made this 0.8.4. It now says the language *did* move, why, and that both
   versions are accepted. A stale sentence about versioning, in the header of
   the document that defines versioning, is the worst place to leave one.
2. **A16**: `[MOD-6]` names the accepted set instead of leaving it to be derived.

Also adopted: **every entry declares one of five classes**, and a hardening
admits only four — the fifth forces a language revision. That makes the A6
failure mechanically visible rather than a matter of judgement. A3's
representation detail became permissive, and A15 now leads with the test that
generates its list rather than the list.

## The fourteen

| # | Rule | What was missing | What it cost |
|---|---|---|---|
| A1 | `[SPN-1]` | that the coercion **takes a borrow** | D-022 — a use-after-free reachable from Safe Ember |
| A2 | `[BRW-1]` | what assigning to a reference local means | D-026 — a write through a shared `ref`, caught only by clang |
| A3 | `[RNG-3]` | where `RangeError` lives | D-025 — the rule's own signature would not compile |
| A4 | `[LT-2]` | whether `E3064` is reachable in v1 | an open question the ledger could not close |
| A5 | `[CLO-3]` | how a `fn(A) -> R` parameter is realised | ADR-018 — a function pointer, so capturing lambdas are rejected |
| ~~A6~~ | ~~`[FN-1]`~~ | **withdrawn** — an unruled reading; ERR-041 is the owner's | — |
| A7 | `[FFI-17d]` | any definition of `@ffi(no_virtual_dtor)` | ERR-034 — an attribute named and defined nowhere |
| A8 | `fn_header` | a production for `extern "C" fn` | ERR-036 — XVI.10's example does not parse |
| A9 | `extern_class` | a production for `extern class` | ERR-037 — `[FFI-39]` rests on syntax that does not exist |
| A10 | `item_body` | admitting A9's production | — |
| A11 | `[THR-2]` | what `Sync` claims, as against what it is tested by | a rule that reads as a guarantee it does not give |
| A12 | `[STD-8]`, as a bullet beside `[STD-8b]` | what `not in` evaluates to | a second search and a second evaluation both admissible |
| A13 | `[CELL-2]` | that `Cell`/`RefCell`/`Arena` are one capability | three unrelated special cases in any implementation |
| **A14** | **`[RNG-8]`** | **its opening, lost to a truncation** | **two facts about `Copy` and layout, stated nowhere** |
| A15 | `[BLD-3]` | what "flags" means in the `.embind` cache key | a binding that survives a change that alters its ABI |
| A16 | `[MOD-6]` | which language versions this revision accepts | an implementer deriving the set from the lineage |
| ~~A4~~ | ~~`[LT-2]`~~ | **withdrawn** — the owner reopened the question | — |

---

### A1 — `[SPN-1]`: the coercion takes a borrow

    Class: SEMANTICALLY NEUTRAL CLARIFICATION + IMPLEMENTATION INVARIANT

**Authorised by** fix-list item 1, which gives the substance: *"A coercion from
`Array[T]` to `Span[T]` or `MutSpan[T]` creates a borrow of the source array.
The resulting view's region is tied to the source according to the normal
borrow rules."*

**Why it was needed.** The rule said only that `Array[T]` *coerces to*
`Span[T]`, and "coerces" is the same word the document uses for numeric
widening and range erasure, neither of which borrows anything. Implemented as
a conversion, this compiled:

```ember
a = Array[i32]([1, 2, 3])
v: Span[i32] = a
a.push(4)        # reallocates
print(v[0])      # reads freed memory
```

No `unsafe` appears in it. `[PHIL-10]` says that cannot happen, and the borrow
rules already had every mechanism needed to stop it — nothing said to use them.

The amendment also states the three spellings are one construction, and
requires the borrow to be **explicit in the implementation's IR**. That last
clause is the part that generalises: an implied borrow is invisible to the
analysis that would catch this, so "the coercion implies a borrow" is not a
sufficient instruction to an implementer.

### A2 — `[BRW-1]`: a reference local is not re-seatable

    Class: SEMANTICALLY NEUTRAL CLARIFICATION

**Authorised by** the general instruction: this is a missing detail, not a
contradiction. No rule anywhere said what `r = e` means when `r` is a
reference.

**Why it was needed.** Two readings were available — re-seat `r`, or write
through it — and they differ observably. ADR-010 chose write-through, because
it is what makes regions insensitive to program location. That choice is sound
for `ref mut` and is *exactly the forbidden write* for a shared `ref`, and
because no rule stated the choice, the shared case was never checked:

```ember
r: ref i32 = ref x
r = 99            # accepted by every check in the compiler
```

It emitted `(*_2) = 99` and **clang** refused it — `read-only variable is not
assignable`. Aliasing-XOR-mutability was being upheld by the backend happening
to emit `const`, not by the language.

### A3 — `[RNG-3]`: `RangeError` is a prelude type

    Class: SEMANTICALLY NEUTRAL CLARIFICATION

**Authorised by** fix-list item 12, *"`RangeError` must become a real
user-facing type… resolvable by the compiler; usable in `Result`; documented;
testable; stable for public APIs"*, which prefers the prelude.

**Why it was needed.** The rule writes the signature
`T.checked(v) -> Result[T, RangeError]` and no rule declares `RangeError`. It
was therefore synthesised on first use — which meant it did not exist while
*signatures* were being collected, and the rule's own worked example did not
compile:

```text
error[E1010]: cannot find type `RangeError` in this scope
```

The amendment adds the two facts an implementer needs and neither of which was
derivable: that it is a prelude type rather than a `std` declaration (because
`checked` is a language construction, not a library function, so it cannot
depend on an import), and that the name must resolve during signature
collection.

### A4 — WITHDRAWN

    Class:                  would have been OWNER-APPROVED SEMANTIC CHANGE
    Implementation change:  none

    Spec semantic change:   none (reverted)
    Implementation change:  none
    Source recovery:        no

`[LT-2]` reads exactly as the owner wrote it. The text this amendment added is
removed.

**Why.** It wrote a *determination* into the rule — that `E3064` reports a
program no v1 source can express, so a conforming v1 implementation may leave
it unemitted. The owner reopened that on 2026-09-09: the evidence does not yet
distinguish "the compiler narrows something the rule means to reject" from "the
rule is stricter than it needs to be", and until it does, neither side moves.

That is the right call and the failure is instructive. Fix-list item 6 said
"determine whether the narrowing is genuinely valid… **do not immediately
change the rule**". I determined, found the narrowing consistent, and then
wrote the conclusion into the rule anyway — which converts a reading into
normative text and removes the very question that was meant to stay open.
A determination belongs in the ledger. D-011 is open again and holds it.

What *is* established, and is not in doubt, is narrower: the intersection is
enforced across a call. `pick(p, q)` returning its first argument still borrows
both, so mutating `q` while the result lives is rejected. That is a test, in
`tests/conformance/LT-2/`, not a claim about what `E3064` is for.

### A4 (original) — `[LT-2]`: `E3064` is unreachable in v1

**Authorised by** fix-list item 6, which asks for the determination to be made
and recorded, and says not to change the rule if the narrowing is correct.

**The determination: the narrowing is correct.** The rule says the intersection
is *taken*, so construction never fails; and `[LT-1]`'s elision extends the
same treatment to a returned view, which is why `pick(p, q)` returning its
first argument still borrows both. The amendment records that, and states what
`E3064` is actually for — a program that **demands** two independent regions —
and that no v1 source can express one, since named lifetimes are v2 (`[LT-6]`)
and `[TYP-15a]`'s `BorrowList`/`ViewList` are the only construct that could.

A v1 implementation is therefore conforming with the code registered and never
emitted. Without this, every implementer must re-derive the same conclusion or
leave a rule apparently unimplemented.

### A5 — `[CLO-3]`: `fn(A) -> R` is a bound, not a representation

    Class: SEMANTICALLY NEUTRAL CLARIFICATION + IMPLEMENTATION INVARIANT

**Authorised by** fix-list item 3, which is explicit that the rule stands and
the compiler must catch up, and lists the pieces.

**Why it was needed.** The rule said a `fn(A) -> R` parameter "is a generic
over `Callable` (static dispatch, monomorphised)" — a true statement of what it
*is*, with nothing about how it is realised. A C function pointer satisfies
"static dispatch" and is far easier, so that is what was built, and it rejects
every lambda that captures anything.

The amendment adds what the shortcut leaves out: the parameter is an implicit
generic bounded by `Callable[(A), R]`; a named function has a zero-sized type
so its call is direct; a lambda has an anonymous type whose fields are its
captures; and — stated as a prohibition, because that is the part an
implementer will otherwise get wrong — a conforming implementation **must not**
use a function pointer, which is what `extern "C" fn` is for.

### A6 — WITHDRAWN

    Class:                  would have been OWNER-APPROVED SEMANTIC CHANGE
    Implementation change:  none (the compiler already behaved this way)

`[FN-1]` reads exactly as the owner wrote it. The text this amendment added is
removed from the document.

**Why.** It answered a question the owner has not ruled on. The amendment file
said so in its own first line — "Not yet ruled on by the owner" — and the text
went into the normative rule anyway, which is the same failure as A4 one page
earlier. Writing "this is open" above an edit does not make the edit open; the
document does not carry the caveat, only the sentence.

**Where the reading now lives.** The compiler still behaves this way, because
the literal rule makes the specification's own worked example uncompilable, and
"comply with the letter and break the example" is not obviously better than the
reverse. So neither side moves and the gap is recorded as **D5** in
`docs/DEVIATIONS.md`, with `mut_param_ty`'s own comment saying plainly that the
exception it implements is unratified. ERR-041 holds the question.

### A6 (original) — `[FN-1]`: a `mut` parameter whose type is itself a borrow

**Not yet ruled on by the owner.** ERR-041 remains open; this records the
reading the document's own example forces, and it is the one amendment here
that the owner may wish to reverse.

**Why it was needed.** `[FN-1]` says a `mut` argument "MUST be a mutable
place". Part VII's own worked example passes `buf.as_mut_span()`, a call
result, which is not a place. Read literally the document's own example is
`E2140`, and `split_at` — which `[SPN-*]` names as the sanctioned way to obtain
two mutable borrows into one container — could not be called on its own result
either.

The reading: where the parameter's type is itself a borrow, the argument *is*
that borrow and travels by value, and the place requirement lands on whatever
the borrow was taken of. A `MutSpan[T]` already carries the exclusivity that
`ref mut` would add, and `[SPN-3]` makes it move-only precisely so there is one
of it.

**If the owner rules the other way** — that `[FN-1]` is literal — then Part
VII's three example lines change, `split_at` grows a binding before every use,
and the compiler change is one line in `mut_param_ty`.

### A7 — `[FFI-17d]`: `@ffi(no_virtual_dtor)` is defined

    Class: EDITORIAL REPAIR

**Authorised by** fix-list item 17, which asks for the cross-reference to point
at the rule that defines the attribute.

**Why the fix is a definition rather than a re-pointing.** `[FFI-17d]` cited
`[FFI-17b]`, which is about templates being importable only as explicit
instantiations and says nothing about destructors. Following the citation to
the right rule was impossible: **no rule defines the attribute at all**. So the
amendment defines it where the concept lives, and says what it does not do —
it licenses no operation and grants no tier, recording an obligation the way
`[UNS-7]`'s `@safety` does.

Item 17 also asks for a rule-index check that every rule reference resolves.
That is not built yet and is tracked as follow-up work.

### A8, A9, A10 — the two missing productions

    Class: SEMANTICALLY NEUTRAL CLARIFICATION (grammar the document already uses)
    Implementation change:  yes — both productions are now parsed

**Authorised by** fix-list items 7 and 8.

`pub extern "C" fn on_update(...)` appears in XVI.10 as the shape of the
embedding story, and `extern class` carries the whole of `[FFI-39]`'s bounded
foreign-inheritance model. Neither had a production, so neither could be
parsed by an implementation following Part III.

A8 adds `["extern" string_lit]` to `fn_header`, with a note distinguishing it
from `extern_block`: this **defines** a function with a foreign ABI, where the
block **declares** foreign functions. It carries the constraints that follow
from rules already written — FFI-safe types under `[FFI-5]`, `E5054` for a
range type under `[RNG-10b]`, `[FFI-20]`'s panic boundary.

A9 adds `extern_class`, with the distinction `[FFI-39]` already draws: an
`extern class` is declared, sized and implicitly `open`, where `extern type` is
opaque, unsized and not inheritable. Per item 8 it does **not** make arbitrary
foreign classes inheritable — inheriting one requires
`@ffi(trampoline, virtuals=[...])`, and multiple inheritance, virtual bases and
unnamed virtuals stay unsupported.

A10 admits A9 in `item_body`.

---

### A11 — `[THR-2]`: what `Sync` claims

    Class: SEMANTICALLY NEUTRAL CLARIFICATION
    Implementation change:  none — threads are not implemented

`[THR-1]` gives a structural test for `Sync` and never says what passing it
means, so the name reads as a guarantee about mutation. It is not one: `Sync`
says that *sharing a handle* is safe, and `[THR-1]`'s test is satisfied
precisely because every mutable field already carries its own synchronisation.
`Atomic`, `Mutex` and `RwLock` are what make a particular mutation safe. The
amendment adds that, and forbids a diagnostic from calling a `Sync` type
thread-safe.

### A12 — `[STD-8]`: `not in` is one negation of one call

    Class: SEMANTICALLY NEUTRAL CLARIFICATION
    Implementation change:  none — `Contains` is not implemented

`[GRM-23]` makes `not in` a single operator and stops there, which leaves a
second search admissible. It is `not b.contains(a)`: each operand evaluated
once, in the order written, `contains` invoked once. The test worth keeping is
`xs.pop() not in ys`, which must remove one element and not two.

### A13 — `[CELL-2]`: what `Cell`, `RefCell` and `Arena` share

    Class: SEMANTICALLY NEUTRAL CLARIFICATION
    Implementation change:  none — none of the three is implemented

Each rule describes its own mechanism and none says what the three have in
common, so an implementer meets them as three unrelated features. They share one
semantic property — mutation is permitted through an otherwise shared access
path — and differ in the mechanism that makes it safe: whole-value replacement
with no reference handed out, a runtime borrow counter, or a region proved
statically. What follows for all three is the part worth stating: interior
mutability never means the borrow checker stops caring, the obligation moves.

**Narrowed after owner review.** The first draft said the three "are one
capability under three policies, not three mechanisms, and an implementation
SHOULD build them that way", which prescribes an architecture rather than a
semantics. An implementation may share machinery between them and nothing here
requires it to; the observable behaviour is what the rule constrains.

### A14 — `[RNG-8]` source recovery

    Class: SOURCE RECOVERY
    Implementation change:  none — both recovered clauses were already true of
                            the compiler and were checked against it

**Approved by the owner, 2026-09-09.** Not a change to `[RNG-8]`: a restoration
of the part of it that a truncation removed.

**What was wrong.** The rule opened mid-sentence, on an ellipsis and a
lowercase "and", wrapped in quote marks:

> `[RNG-8]` "…and crosses an FFI boundary as its representation (`[FFI-5]`). …"

It is the **only rule in the document with that shape**, and no change-log row
mentions `[RNG-8]`, so the loss was an editing accident rather than a deletion.

**Where the text came from.** Both v0.6 sources in this repository — the draft
and the owner's revision of it — carry the rule complete and **identical**, and
v0.8.3 descends from 0.6.3 by its own lineage note:

> `[RNG-8]` A range type is `Copy` when its representation is, has the layout of
> its representation, and crosses an FFI boundary as its representation
> (`[FFI-5]`); a value arriving from foreign code is **not** assumed in range
> and needs `[RNG-3]`.

The two texts **overlap exactly** at *"and crosses an FFI boundary as its
representation (`[FFI-5]`)"*, which locates the cut precisely. Everything before
those words is what was lost:

> **A range type is `Copy` when its representation is, has the layout of its
> representation,**

The *tail* was deliberately revised between 0.6 and 0.8.3 — strengthened with
`[RNG-10b]` and `[RNG-3a]` — and the head was dropped in that same edit. So the
restoration is the recovered head joined to the newer tail, spliced at the words
they already share. No 0.8.3 addition is disturbed.

**Why this entry exists at all.** The head had previously been filled in by me,
with *"A range type erases to its representation at every coercion site
(`[TYP-5]`) and"*. That was plausible, consistent with the neighbouring rules —
and **wrong**. The rule's actual opening is about `Copy` and layout, not about
coercion-site erasure. Had it stayed, the document would have carried my
sentence in the owner's voice, and the two facts the rule really states would
have been silently absent. Everything in this file exists because of that.

**The wording is verbatim, including its awkwardness.** The owner prefers
"`Copy` when its representation is `Copy`" for the canonical text, and that
clarification is deliberately **not** applied here: modernising a sentence
inside its own recovery is how provenance is lost. It is a separate amendment
if it is wanted.

**Nothing in the compiler moved**, and both recovered clauses were checked
against it: `a: Roughness = 0.5; b = a` uses both, so the type is `Copy` because
`f32` is; and the emitted C carries no `Roughness` at all — `float em_take(float
_1)` — so it has its representation's layout.

**On `Copy` and the niche.** The owner confirmed these do not conflict:
`Roughness` has `f32`'s layout, while `Option[Roughness]` may exploit an invalid
representation as `None`. `[RNG-7]` already draws that line — a range type
supplies a niche only where its range does not exhaust its representation — and
the two statements are at different levels.

---

### A16 — `[MOD-6]`: the accepted language versions, named

    Class: SEMANTICALLY NEUTRAL CLARIFICATION
    Implementation change:  none — `LANGUAGE_VERSIONS` already carries both

`[MOD-6]` requires the compiler's supported set to "include every language
version whose source it still accepts", which is the rule and is unchanged. It
illustrates with `#! language "0.5"`, so an implementer reading it after 0.8.4
has to derive the currently accepted set from the lineage paragraph. The
amendment names it: `"0.8.4"` selects this language, `"0.8.3"` stays valid
because 0.8.4 is additive over it.

Deliberately an example rather than a rule change — the general statement
already permitted both, and rewriting it would be churn against something
already correct.

### A15 — `[BLD-3]`: what "flags" covers

    Class: SEMANTICALLY NEUTRAL CLARIFICATION

    Class: SEMANTICALLY NEUTRAL CLARIFICATION
    Implementation change:  none yet — the `.embind` cache is Phase 7

**Authorised by** fix-list item 18: "Define one authoritative ABI fingerprint
set… If changing a configuration can change generated binding semantics or ABI,
it must invalidate the binding."

**Why it was needed.** `[BLD-3]` gives the `.embind` cache key as "header hash
+ flags + overlay-list hash". Every dangerous setting is named *somewhere* —
`[BLD-FFI-1b]` requires the MSVC runtime switch, `_DEBUG` and
`_ITERATOR_DEBUG_LEVEL` to be inherited byte-for-byte, `[CXX-6]` runs the corpus
across both compilers, both CRTs, RTTI on and off and both iterator-debug
levels — but none of that says those settings are *in the cache key*, and an
implementer reading `[BLD-3]` alone would reasonably read "flags" as the
compiler's command line.

The amendment enumerates the minimum set and, more usefully, states the test
that generates it: if changing a setting can change a binding's semantics or
ABI, it is part of the key. It also names the one an enumeration would miss —
**the thunk generator's own version**, because a change in how a thunk owns,
copies or catches changes the contract without changing the header.

---

## S1 — `[TYP-15]` and `[LT-3]`: an owner semantic decision

    Class: OWNER-APPROVED SEMANTIC CHANGE
    Implementation change:  yes — a `static` may now hold a `str` literal

**This is the class a hardening may not contain**, and it is here rather than
hidden because the owner made the decision explicitly and asked for it to be
recorded as one: *"Please resolve ERR-044 as an owner semantic decision, not as
a hardening-only change."*

**Settled: this is what made the language version 0.8.4.** The protocol says
the fifth class "forces a language revision", and the owner took that route
rather than relaxing the boundary — so S1 is 0.8.4's one semantic change and
everything else in the file is Hardened_1 on top of it.

The reason to prefer that route was never the size of the change — it is tiny
and additive, and no 0.8.3 program becomes invalid — but the boundary it
protects: **hardening must never quietly become language evolution.** The
alternative was Hardened_2 with the document admitting it was no longer pure
hardening, which costs the boundary to save a digit.

`0.8.3` remains an accepted language version, so a file declaring
`#! language "0.8.3"` compiles unchanged; `0.8.4` is accepted beside it.

**What was wrong.** ERR-044. `[TYP-15]` stated a principle — a view may not be
stored "in a place whose region is not outlived by the view's region" — and then
an enumeration that said class fields, `static`s and the rest are "**always**
forbidden". `[LT-3]` said, in as many words, that a `str` literal **may** be
stored in a class field because its region is `static`. Two normative statements
requiring different things, and `[TYP-15]`'s own principle siding with `[LT-3]`,
since a static region does outlive the destination.

**The decision: `[LT-3]`'s semantics govern.** Long-lived storage is not
inherently incompatible with views; a view may be stored there when its region
outlives the destination. The enumeration was what overreached, by assuming
every listed place has no *possible* sufficient region — true of a field holding
a borrowed view, false of one holding a literal.

**The exception is on the view's region, not the destination type**, and the
pair the owner gave is the test of it:

    class Foo:
        greeting: str = "hello"        admitted: "hello" is static-region

    fn set(mut foo: Foo, s: str):
        foo.greeting = s               refused: `s` may be a caller's region

**Deliberately not widened.** `[TYP-15a]` is untouched: an owning container at a
view type — `Array[str]`, `Map[str, V]`, `Array[MutSpan[T]]` — stays rejected
whatever the region, because that rejection is at the *type* and not at the
region, and `BorrowList[T]`/`ViewList[T]` remain the specialised model. The
compiler's span-element and container-element checks are unchanged.

**The diagnostic reconciled too.** `[LT-3]` named `E3060` for the rejection;
`[DIA-7a]` keys `E3060` to shape B7 (a borrowed value that does not live long
enough) and `E3063` to B12 (a view stored in a place that outlives it). B12 is
this. `[LT-3]` now says `E3063`, which is what the compiler already emitted.

**What the compiler does now.** `[STA-2]` restricts a `static`'s initialiser to a
literal, so the region question is decidable syntactically and needs no region
graph; `has_static_region` answers it and is deliberately conservative — anything
not obviously static is treated as not static, which errs towards rejecting a
program rather than storing a view that outlives its source.

---

## The editorial repairs and leftover removals (E1–E4, V1–V4, K1)

These were carried out in the same pass and described in the change log, and
until `tools/hardening_check.py` existed they were **not entered here** — which
the gate reported on its first run as ten undeclared differences. That is the
gate working: prose in a change log is not a declaration, and an edit nobody has
to look up is an edit nobody reviews.

### E1–E4 — editorial repairs

    Class: EDITORIAL REPAIR
    Implementation change:  none

Four editorial instructions had been pasted into normative prose instead of
being carried out. Each is now performed and the instruction deleted, so a
reader is no longer doing the editor's job — differently each time.

| # | Rule | The instruction | What was done |
|---|---|---|---|
| E1 | `[RNG-7]` | a quoted block duplicating the rule and adding to it | the addition is real normative content — a range type supplies a niche only where its range does not exhaust its representation — and is now the rule's text rather than a quotation |
| E2 | `[FFI-33b]` | a quoted refinement of "on construction" | substituted: the creating thread is recorded at the point the box is formed, which is `[FFI-36]`'s wrapping or the `adopt` call |
| E3 | `[FFI-2a]` | a literal *After "…calling convention", insert: "…"* | inserted where it says; instruction removed |
| E4 | `[BLD-2]` | *add the `[verify]` package-config section to the enumerated key* | **deleted unperformed.** 0.6.2 removed that layer, so carrying it out would have resurrected it. The one place an explicit instruction in the document was overridden, and it is recorded rather than quietly skipped |

### V1–V4 — leftovers of the layer 0.6.2 removed

    Class: EDITORIAL REPAIR
    Implementation change:  none

Only the removed system's vocabulary. Ember still calls `@noalloc`, `@nosync`,
`@noblock` and `@nopanic(explicit)` contracts and all 57 mentions are untouched;
what went is `@requires`, the prover, the `verify` layer and the `Contract` kind.
Each removal is settled by the document's own closed owner questions,
OQ-28..OQ-32.

| # | Rule | Removed |
|---|---|---|
| V1 | `[EFF-18]` | `Contract` from `RuntimeCheck`'s kinds — `[EFF-16]` assigns four and `[EFF-22]` permits four |
| V2 | `[EFF-17]` | the consequent "does not forbid `RuntimeCheck(Contract)`" |
| V3 | `[STD-6]` | the **verify** layer from the std layering (OQ-32) |
| V4 | `[UNS-7]` | the recommendation to carry `@requires` beside `@safety` (OQ-29) |

### K1 — one keyword status

    Class: EDITORIAL REPAIR
    Implementation change:  none — the lexer has had 49 keywords all along

`yield` appeared in both the v1 keyword table and the reserved-for-future list,
claiming two statuses for one word. `[LEX-15b]` already makes it a v1 keyword
and says its count "supersedes" `[LEX-15]`'s, so the rule was the half that
governed and the table is brought to match. The tables carry a note saying so.

### E5 — `[EFF-18]`: the effect set gains `Nondet` (ERR-028 applied)

    Class: EDITORIAL REPAIR
    Implementation change:  none — no rule changes meaning and the compiler does not move

**Authorised by** ERR-028 in `docs/spec-errata.md` (decided: the effect set has
ten members; X.1 governs; `[DET-2]` enumerates `Nondet`'s sources
exhaustively). At Hardened_1 time this disagreement was deliberately left as an
owner decision rather than a hardening (see the front matter's "Deliberately
not done" paragraph as written then); the decision it waited on is the one
ERR-028 already records, and no superseding ruling has since removed `Nondet`
from X.1, from the X.1 table, or from `[DET-*]`.

**Why this is repair and not a semantic change.** Part X §1 already defines
the set with `Nondet` and carries a table row defining the effect; `[DET-1]`
contracts over it, `[DET-2]` introduces it "by, and only by" its sources, and
`@deterministic` is meaningless without it. `[EFF-18]` was written for 0.6
(`Io`, `Lock`); `Nondet` arrived in 0.6.3 with `[DET-1]`/`[DET-2]`, which added
it to X.1 without revisiting `[EFF-18]`'s parenthetical list. `[EFF-18]`'s own
sentence — it "does not remove an effect previously attached to any operation;
it refines the effect model" — says it is not an exclusive list. Adding the
missing member brings a stale enumeration in line with the section that defines
the set; it admits no new program, forbids no old one, and changes no effect
any operation carries.

**What was done.** `[EFF-18]`'s full set reads
`{Alloc, Sync, Lock, Io, Panic, Unsafe, FFI, Block, Nondet, RuntimeCheck(k)}`,
matching X.1. Nothing else in the rule moves: `@noio`, `@nolock`, `@noblock`
and `@nosync` remain independent, which is what the rule is for. The front
matter's "Deliberately not done" paragraph now records the prior disagreement
in the past tense rather than asserting a live contradiction.

**Version standing — settled 2026-09-10.** The owner ruled: *"cut
`0.8.4_Hardened_2`. E5 is an editorial/hardening repair and does not change the
accepted program set. Therefore it belongs in the hardening-number bump rather
than a language revision."* The provisional header is gone and the file is
`0.8.4_Hardened_2`.

For the interval between, the header read *0.8.4_Hardened_1 **+ E5, pending a
version decision***, following 207c69f (which did the same for S1): a header
claiming to be an artifact the file no longer matches is a defect in its own
right, and guessing the number is not the implementer's to do — 8101389 records
the last one as "the owner's call". Both halves of that discipline held.

`docs/spec-source/Ember_v0.8.4_Hardened_1.md` stays as the previous frozen
baseline; `Ember_v0.8.4_Hardened_2.md` is the new one, and is what the next
hardening diffs against.

## S2 — `[UNS-10]`, `[UNS-10a]`, `[UNS-10b]`: `UnsafeCell[T]` becomes a real primitive

    Class: OWNER-APPROVED SEMANTIC CHANGE
    Implementation change:  yes — a new std.mem type; unbuilt as of this cut

**Owner ruling, 2026-09-10, resolving ERR-043.** The errata recorded that
`UnsafeCell` was named exactly once in 5,526 lines — in `[CELL-9]`, as *the*
primitive a package uses for unchecked interior mutability — and defined by no
rule. ADR-019 routed `std`'s own `Cell` around it, which left the language in
the position the owner named as unacceptable: *"if `RefCell` is implemented
using an internal primitive that ordinary packages cannot reproduce, then the
language would have an unexplained distinction between `std` and user
libraries."*

**The ruling, in the owner's terms.** `UnsafeCell[T]` is retained and becomes
the lowest-level interior-mutability primitive. It permits mutation through
shared access **only from `unsafe` code**; it does not globally disable
`[BRW-1]`, lifetime or region checking, type checking or bounds checking; it
gives no safe `ref`/`ref mut`, no runtime borrow check, no synchronisation, and
no further safety tier; it is never `Copy`, always `!Sync`, and `Send` when
`T: Send`. Unsafe code may temporarily violate the static aliasing proof inside
the abstraction and must not let a conflicting or invalid reference escape into
Safe Ember. `@safety` and the existing `[UNS-*]` machinery remain authoritative
and nothing new is introduced beside them. It is not permitted in
`@static_safe` code, and diagnostics must not suggest it.

**What was decided here rather than by the owner, and it was asked first.** The
ruling fixed the semantics and not the spelling. The API surface and the module
were put to the owner as a question, because both change the accepted program
set; the owner chose `std.mem` with a raw-pointer accessor. So:
`UnsafeCell(owned v: T)`, `get(self) -> *mut T` (needs `unsafe` under
`[UNS-1]`, being a raw pointer), `into_inner(owned self) -> T` (safe — the cell
is consumed). Nothing else was inferred.

**Why this is a language revision and not a hardening.** It adds a type and a
capability that no 0.8.4 program could express. The owner said so explicitly,
and contrasted it with E5 in the same message. `[UNS-10]` is why the file is
0.8.5.

**`E3105`** is registered in `ember_diag` for `[UNS-10b]`'s `@static_safe`
refusal, ahead of its emitter, so `[DIA-6a]` holds. It joins `E3100` in
`shapes.rs`'s named exemption: both sit in the ownership range and are neither
ownership nor borrow errors.

## S3 — `[CELL-12]`: `RefCell[T]` is never `Copy`

    Class: OWNER-APPROVED SEMANTIC CHANGE
    Implementation change:  yes, when RefCell is built — it must NOT inherit
                            [CELL-4]'s field-derived Copy

**Owner ruling, 2026-09-10.** `[CELL-4]` derives `Cell`'s `Copy`-ness from its
field, and `Cell[T]` gets it free because a struct's `is_copy` already reduces
to its fields'. Read mechanically, the same derivation makes a `RefCell` `Copy`
whenever `T` is — and the owner ruled that it must not, on semantic grounds:
*"copying the value would duplicate that state and therefore create two
logically independent cells with inconsistent knowledge of the same
storage/borrow state. That would make the runtime borrow invariant unsound."*

So `RefCell[T]` is move-only whatever `T` is; moving one transfers the whole
cell including its borrow state; copying the contained `T` is a separate and
unaffected question; and `[CELL-4]` does **not** extend to it. The rule now
states the exception rather than leaving an implementer to derive it.

**Provenance worth keeping.** This was raised as an owner question precisely
because an earlier revision of `docs/HANDOFF.md` asserted "`RefCell` is never
`Copy`" as though the document said so, when Part IX said nothing about it.
The reasoning was sound and the sourcing was not — the shape of both withdrawn
amendments. It was withdrawn, escalated, and is now normative because the owner
ruled, which is the only route that was ever available.

## S4 — `[FN-1a]`: a `mut` parameter at a view type accepts a view value

    Class: OWNER-APPROVED SEMANTIC CHANGE
    Implementation change:  none — the compiler already behaved this way

**Owner ruling, 2026-09-10, resolving ERR-041 and closing deviation D5.**
`[FN-1]` says a `mut` argument "MUST be a mutable place". Read literally that
rejects Part VII §7's own worked example, `normalize(buf.as_mut_span())`, whose
argument is a call result — the document forbidding the program it uses to
explain itself. Amendment A6 wrote a reading into `[FN-1]` and the owner
withdrew it, because a hardening may not answer what Ember means; neither side
moved after that.

**The ruling.** The example governs. Where a `mut` parameter's declared type is
itself a view, the argument may be an expression that produces such a view, and
the mutable-place requirement applies to the place the view was **taken of**.
The owner drew the line explicitly: this is *"passing a mutable borrow/view
value produced from a mutable place"*, not *"passing an arbitrary immutable
value merely because it happens to have a mutable-looking type"* — it admits no
arbitrary temporary and bypasses no mutability check.

**No code moved.** D5's fix plan said "one line in `mut_param_ty` if the owner
rules for the literal text; nothing if the example governs". The example
governs, so D5 closes as *the compiler was right*, and
`tests/conformance/FN-1a/` pins the behaviour that previously rested on a
deviation entry.

**Attribution anchor: `[MOD-7]`, and it is a misattribution — stated rather
than worked around.** The edit is inside `[FN-1]`'s parameter-mode list: the
`mut b: B` bullet gains "or a mutable view value derived from one", and
`[FN-1a]` is inserted beneath it. That bullet carries no rule id of its own —
`[FN-1]` is named on the list's *heading* line, not at the start of the mode
bullets — so `hardening_check.py` walks back to the nearest line that opens a
rule and finds `[MOD-7]` at line 1416, which is a different rule about
`pub(read)` fields and has nothing to do with this. COLD-START §8 records this
class of behaviour: *"a checker that knows one shape of a rule reports thirty
false positives … each time it named a real rule and the wrong one."*
`[MOD-7]` is named here because the gate reads its anchor set from this file
and the edit must be declared. **What was actually amended is `[FN-1]` and the
new `[FN-1a]`; `[MOD-7]` is untouched.** Anyone auditing this divergence should
read it that way.

---

## Not amended, and why

**The diagnostic-code collisions.** `[GRM-23]` names `E0104` where Part III's
precedence table names `E0102`, and `[MAN-3]` names `E9010`, which `[TYP-9c]`
also owns. The owner ruled on both (items 4 and 5): the normative code wins and
the compiler changes. No amendment is needed for the compiler to be correct, so
none was made — the tension with `[DIA-6a]`'s one-code-per-rule is recorded at
the registry entries in `compiler/ember_diag/src/codes.rs` and in
`docs/spec-errata.md` under ERR-026 and ERR-039.

**Fix-list items 19 and 20 — `std::function` and `std::optional<T>`.** Both are
real inconsistencies and **the document already adjudicates them**. `[FFI-17]`'s
numbered list says `std::function` "is not importable (`E5030`)" while XVI.7a's
table says "not importable *as a parameter*; importable as an opaque owned
object"; the list says `std::optional<T>` maps "for trivially copyable `T`"
while the table says "where `T` maps, by value across the thunk". `[FFI-17]`
declares its own list **`NON-NORMATIVE` under `[CAT-1]`** — "where it and a rule
disagree, the rule governs" — and names `std::function` as one of the four
contradictions that demotion exists for. So the tables govern and there is
nothing to decide.

What the document asks for is a *deletion*: "A future revision should delete
from the list every claim a rule already makes rather than keep two copies in
step." That is a revision-level edit to owner prose, not a hardening, which may
only add. Left for the owner.

**Fix-list items 21 and 22 — reload termination and COMMIT synchronisation.**
Both are **already fixed in v0.8.3**, which post-dates the review they were
raised from.

* Item 21 asked that a foreign `noexcept` call which can terminate the process
  during migration become opt-in. 0.8's change log row 1 records exactly that:
  `[HR-39]` had permitted it and called termination "a known consequence"; such
  a call is now `E2227`, and `[HR-43]`'s `@allow_reload_terminate` is the
  explicit opt-in, reported by `ember tcb` and counted rather than forbidden by
  `[GATE-3]`.
* Item 22 asked which threads may execute, whether they are quiesced, when new
  addresses become visible, what memory ordering applies, and what happens to a
  thread already inside old code. `[HR-42]` and `[HR-42a]` (§XVIII.4b) answer
  all five: the per-thread Ember-depth counter is the synchronising object,
  acquire on entry and release on exit, acquire-scan then release-publish on the
  reload side; new addresses become visible at a thread's next entry; and a
  thread executing old code cannot exist, because `[HR-3]` makes the scenario
  unreachable rather than merely survivable.

**Everything in `docs/spec-errata.md` not listed above.** Duplicated sentences,
stale counts superseded by a later rule, editorial instructions pasted into the
source, leftovers from the layer 0.6.2 removed. Each is recorded there with the
reading taken; none is edited into the document. Fix-list items 25 and 26 ask
for several of these to be repaired properly, which is follow-up work and will
be appended here when done.
