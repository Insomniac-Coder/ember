# Cold start — read this first

State as of 2026-09-15. Read `docs/MIGRATION-0.9.7.md` for historical H1
context, then read the current H3 target and `docs/HANDOFF.md` §0 for the active
migration, process, and verified implementation state. `docs/MIGRATION-0.9.5.md`
and `docs/MIGRATION-0.8.3.md` remain historical
context for the 0.9.5 intake and original phase order.

| Ledger | Answers |
|---|---|
| `docs/DEFECTS.md` | every compiler defect, its status, **and how the fix was verified**. Its header carries the four-way sort below |
| `docs/DEVIATIONS.md` | where the compiler knowingly differs from the document, and why |
| `docs/spec-amendments.md` | every difference between the owner's file and the normative copy, each with a class |
| `docs/spec-errata.md` | defects in the *document*, and the reading taken |
| `docs/DECISIONS.md` | ADR-001..036 |
| `docs/OWNER-QUEUE.md` | **questions an agent may not answer.** ODR-001/002 and ODR-004..015 are closed; ODR-003 is deferred editorial. `@latebound` implementation is now an H1 milestone, not an owner question |

---

## 1. State

**Last committed implementation baseline:** `c4aa813` on `main`, following
`2087600` for the verified `@latebound` callable-boundary implementation,
`8433479` for callable modes and the H1 target cut, `9ade1d0` for validated
callable interface artifacts, `af7c525` for verified
in-MIR callable metadata, `90059c8` for direct callable summaries, `c913fbd`
for field-sensitive region vectors, `e0ba765` for canonical Array-loop
borrowing, and `d077563` for the H6 Span implementation. `1c6b285` is a local
checkpoint in the latebound arity matrix; `41d65ed` is the static-view
provenance checkpoint; `5332572` closes D-126 by validating the serialized
EMIF cache key at decode time; `c4fccdc` closes D-127 by rejecting an empty
explicit `@borrows` contract at the artifact boundary; `523c030` closes D-128
by rejecting empty and malformed source `@borrows` arguments. `d35e94f`,
`30836ee`, and `c4aa813` record the subsequent canonical Arena storage and
escape-constraint consumers and their final validation. Verify push state
before handing it off.
Always run `git status` and `git log -1` instead of treating this sentence as
live Git state.
`https://github.com/Insomniac-Coder/ember.git`

    210 tests green      cargo test --workspace
    0 warnings           cargo build          <- keep it there
    6 gates green:
      python tools/hardening_check.py      no undeclared change to the specification
      python tools/rule_index.py           rules, codes, references, baselines
      python tools/spec_check.py           fenced `ember` blocks parse
      python tools/error_pages.py          every documented fix compiles
      python tools/check_branding.py       no hard-coded project names
      python tools/split_spec.py --check docs/spec-source/ember-spec.md docs/spec
                                             docs/spec/ is the split of the source

 133 top-level conformance rule directories, 479 `.em` files including support
 modules. 125 defects recorded, **none open**.
**3 open deviations** (D1, D3, and D4; D2 is closed, D5 and D6 are historical).
**D-126 through D-128 are fixed:** EMIF decoding recomputes the cache key from
the artifact's identity inputs and rejects self-inconsistent metadata,
including an empty explicit `@borrows` contract, before consumption; source
checking rejects empty and malformed `@borrows` arguments with `E2031`.
**ERR-050 / ODR-009 is closed by the H10 owner rulings** on `Zeroable`,
`MaybeUninit`, and Arena bulk initialization. Their core
compiler/runtime/conformance work is now complete. **ERR-051 / ODR-010 is
closed in H1:** v1 `Default` panic
aborts, so `[ARN-10]` requires no observable post-panic rollback or unwinding;
only a separately specified recoverable transactional API creates rollback.
ERR-047 and ERR-048 were closed
by the H7/H8 owner rulings; ERR-049 was closed by the H9
Arena-provenance ruling; earlier ERR-041 and ERR-043 were
decided on 2026-09-10 and ERR-042 was withdrawn as wrong. See `HANDOFF.md`.
Ratchets in
`tools/*_baseline.json` may shrink and never grow; `--allow-growth` needs a
reason in the commit message.

**The working process is `docs/HANDOFF.md` §0.0** — authority, versions, the
five-way sort, the four-document write path, probe-first, test discipline,
claim discipline, escalation, scope reporting, and the pre-commit checklist.
Read it before starting a task, not after.

**Current continuation (2026-09-19):** D-156 rejects calls to
`where Self: Sized` defaults through `ref dyn`, including inherited defaults.
Interface formation and calls to ordinary members remain accepted. Read
`HANDOFF.md` §0.134 for the reproducer, verification, and remaining dyn work.
The owner requested a halt after this fix; do not start the next task without
a resume instruction. The frozen H3 target and adopted specification are
unchanged.

**Previous continuation (2026-09-15):** the H3 callable-mode diagnostic slice is
implemented and recorded as D-155. `E2228`/`B15` now covers direct callable
assignment, generic callable parameters, and `std.borrow.with_views*_mut`,
with an exact UI snapshot, error page, and compiling repair. The full workspace
build/test suite, focused UI test, `error_pages.py`, `rule_index.py`, and
`git diff --check` are green. This is a compiler correction against the frozen
H3 development target; the adopted v0.8.5 specification remains unchanged.

**0.9.6 status:** the owner-supplied 0.9.5 H2/H3 files and the Revision 5
simplicity-consolidation RFC are preserved unchanged under
`docs/spec-source/as-received/`. The repaired and frozen
`Ember_v0.9.5_Hardened_4.md` remains the source-gap audit record. The owner then
supplied `[LT-8]`–`[LT-13]`, frozen as H5, and resolved the mutable-helper
boundary in H6. The owner then specified ordinary borrowed/default, `mut`, and
`owned` callable parameter modes and explicit `mut` callback modes in
H7. The owner then closed the helper-input and `Callable` bridge boundaries in
`Ember_v0.9.5_Hardened_8.md`: mutable helper inputs are `mut` reborrows, no
helper consumes a view, and callable modes remain compile-time canonical type
metadata through the existing abstraction. H9 then resolved the Arena-backed
return-provenance boundary with the narrow `@borrows(arena)` exception. H10
now resolves deterministic bulk initialization and the complete `Zeroable` /
`MaybeUninit` safety/API boundary. The owner then selected the simplicity-
consolidation architecture line and approved the abort-only `[ARN-10]`
clarification. H2 then completed Arena-backed collections, and
H3 records the public hashing protocol, and
`Ember_v0.9.6_Hardened_4.md` closes its callable boundary with static generic
`H: Hasher` dispatch. The owner subsequently clarified that approval of the
post-H4 Revision 5 simplicity RFC was intended to produce the next hardened
target. `Ember_v0.9.6_Hardened_5.md` materializes that architecture,
tooling, documentation, and development contract while keeping H4 immutable.
The owner then closed ODR-014's remaining Span iterator, chunk, and raw-pointer
boundary in `Ember_v0.9.6_Hardened_6.md`, keeping H5 immutable. The owner then
closed ODR-015 with the `@latebound` callable-type boundary in
`Ember_v0.9.7_Hardened_1.md`, keeping H6 immutable. H1 was the frozen
development target, but not yet the normative repository
source. It retains the owner-
selected multi-region-view target and separate shared/all-mutable callback-
helper families; no mixed overloads are implied, and it requires one shared
semantic-fact/MIR verification architecture without merging distinct language
semantics. The post-H4 Revision 5 simplicity RFC requires reuse of authoritative
facts and abstractions, preservation of necessary semantic distinctions, and
conservative unknown information. No
0.9/0.9.5/0.9.6 implementation
or conformance is implied by the target's version label.

## 2. The rules. Read these before touching `docs/spec-source/`

Learned expensively. Two amendments were withdrawn for breaking them.

1. **The specification is the contract and is never edited to make the compiler
   agree with it.** Where the two disagree and the rule is sound, *the compiler
   moves*.
2. **Where the document contradicts itself, neither side moves** until the owner
   rules. Do not pick the reading that matches what is already built — that is
   how an implementation workaround becomes the language.
3. **`docs/spec-source/as-received/Ember_v0.8.3_spec.md` is never edited**, ever.
   md5 `2bdffee6510b8668cf828185266efedb`.
4. **Every edit to `ember-spec.md` declares one of five classes**, and only four
   are permitted in a hardening:

       SEMANTICALLY NEUTRAL CLARIFICATION   permitted
       IMPLEMENTATION INVARIANT             permitted
       SOURCE RECOVERY                      permitted
       EDITORIAL REPAIR                     permitted
       OWNER-APPROVED SEMANTIC CHANGE       forces a language version bump

   The line: **the moment an edit answers *what Ember means* rather than *how to
   implement what Ember already means*, it stops being a hardening.**
5. **`tools/hardening_check.py` enforces declaration**, not correctness. It
   refused three of my own correct-but-undeclared edits. It would not have
   caught either withdrawn amendment — both were declared. It is a floor.

**Both withdrawals had the same shape and you will be tempted the same way:**
the analysis was right, and the answer went into the normative text instead of
the ledger. A4 wrote a determination about `E3064` into `[LT-2]`. A6 wrote an
unruled reading of `[FN-1]` into the rule, with "not yet ruled on by the owner"
sitting directly above it in the ledger — *the document carries the sentence, not
the caveat.*

### Which ledger a finding belongs in

| Kind | Where | Who moves |
|---|---|---|
| rule clear, compiler wrong | `DEFECTS.md` | the compiler |
| maybe right, never proved | a conformance case | nobody |
| two rules disagree, neither governs | `spec-errata.md` | **nobody, until the owner rules** |
| rule clear, compiler knowingly differs | `DEVIATIONS.md` | the compiler, later |

## 3. Versioning

The adopted normative document is **v0.8.5_Hardened_1**; the frozen development
target is **v0.9.7_Hardened_3**. Two numbers move independently:

* **language version** — moves when the set of accepted programs changes, and
  **resets the hardening number to 1**. 0.8.4 exists for exactly one change: S1,
  the owner's resolution of ERR-044. **0.8.5 exists for three** — S2
  (`UnsafeCell` becomes a real primitive), S3 (`RefCell` is never `Copy`), S4
  (`[FN-1a]`) — all owner rulings of 2026-09-10, all additive.
* **hardening number** — moves when the document gains implementation detail and
  no rule changes meaning. **The number itself is the owner's call**: `8101389`
  records the last one that way, and `207c69f` is the shape to follow when the
  file has outrun its header and the decision has not been made.

`LANGUAGE_VERSIONS` in `compiler/ember_parser/src/lib.rs` currently accepts
the exact `"0.9"`, `"0.9.5"`, `"0.9.6"`, and `"0.9.7"` selectors plus the
supported 0.8 contracts. Selector recognition is implementation evidence, not
adoption by itself.
`docs/spec-source/Ember_v0.8.5_Hardened_1.md` is the frozen adopted snapshot.
The current development lineage first diffed H1 against immutable H10, H2
against immutable H1, H3 against immutable H2, H4 against immutable H3, and
H5 against immutable H4, and H6 against immutable H5; any next 0.9.6 hardening
diffs against frozen H6, never
against an as-received file.
`Ember_v0.8.4_Hardened_1.md` and `Ember_v0.8.4_Hardened_2.md` are kept as prior
baselines. The working source and the current snapshot are **identical** right
now; where they ever differ, the working source governs for implementation and
`docs/HANDOFF.md` §0.17 is the authoritative statement of which artifact is
normative for what.

The current development target is `0.9.7_Hardened_3`, per the owner's explicit
selection of the supplied H3 specification. H3 names H2 as its immutable
immediate predecessor; that H2 artifact is not present in this checkout. H6
remains the architecture-line predecessor of H1;
H10 remains the
immutable architecture-line predecessor. H5 recovered the missing
source; H6 records the mutable-helper family; H7 records callable parameter
modes; H8 records helper input modes and compile-time mode preservation through
`Callable`; H9 records Arena-backed return provenance; H10 records the complete
Arena initialization and `MaybeUninit` contract. The 0.9.5 multi-region-view
feature itself is the owner-selected language change. 0.9.6_Hardened_1 preserves 0.9.5's
accepted/rejected ordinary-source sets while selecting the consolidated
reference-compiler/conformance architecture. H2 completes the Arena-backed
container contract, H3 completes the public hashing protocol, H4 selects
static generic Hasher dispatch, H5 binds the approved post-H4 simplicity
architecture/process contract without changing source semantics, and H6 fixes
the public Span iterator/chunk/raw-pointer contract. `0.9.7_Hardened_1` added
the owner-approved `@latebound` callable-type language boundary. H3 closes
ODR-016 and withdraws the two H2 detector findings described in its change log.
Any later hardening must become `0.9.7_Hardened_4`; another semantic change requires an
owner-selected language revision, never an in-place H1 edit.

**Do not couple a tool to a version string.** `rule_index.py` decided which
change log was current by matching `"0.8.3"` and would have silently stopped
checking the current section the moment the version moved. It was one commit
from happening.

## 4. Where the work is: finish Phase 2

**Current phase count: exactly 1 of 9 phases is complete.** Phase 2 is active
and substantial, but it has not met every exit criterion below. Phases 3–9
have not passed their exit gates. Older historical notes that counted a
separate "Phase 0" do not change this current nine-phase accounting.

Exit criteria, from Part XXI: *all `[OWN-*]`, `[BRW-*]`, `[LT-*]`, `[DRP-*]`,
`[SPN-*]`, `[CELL-*]`, `[DIA-7..10]` tests; milestone M2; zero unclassified
borrow errors across the whole corpus.*

M2 exists (`tests/milestones/`) and no unclassified log is produced, so what is
left is coverage:

    OWN   7/8    missing OWN-8 (Clone, @derive(Clone)) — an unbuilt feature,
                 not a missing test; OWN-6 is complete at MEM-API-1
    BRW   7/9    missing BRW-8 (an ABI decision, "never observable" — assert on
                 emitted C), BRW-9
    LT    6/10   missing LT-1b (L3014, an opt-in lint with no opt-in mechanism),
                 LT-2a, LT-5, LT-7 (callback regions)
    DRP   4/6    missing DRP-4 (needs effects, Phase 4),
                 DRP-6 (Box/handle/Shared — Phase 3). DRP-5 has cases since D-030
                 was fixed (drop-body moves rejected)
    SPN   3/3    done
    OWN-5        both clauses now, after D-035 — see the note below
    CELL  11/13  Cell/RefCell cases exist for every currently buildable rule;
                 CELL-3 and CELL-8 (`!Sync`) wait for Send/Sync/threads. See §5
    DIA   0/5    rule-family exit count remains incomplete; 18/25 ownership
                 shapes have exact UI snapshots, with seven blocked — see §6

**A directory named for a rule is not coverage of the rule.**
`tests/conformance/OWN-5/` existed and passed while the compiler ran no
destructor on an overwrite at all (D-035). Its case tested the sentence's
parenthetical and was written so the main clause could not fire. When a rule
states two things, count two cases.

### The method that found twelve defects — keep using it

**Probe every normative rule with a minimal executable program before changing
any implementation code.** Not "does the suite pass" — write the three-line
program the rule describes and check the number. Four of the twelve were
*silent*: a declared `drop` that never ran, temporaries that never dropped
(output byte-identical either way), an explicit `drop()` that double-freed, a
write through a shared `ref` caught only by clang's `const`.

Where behaviour cannot be observed from output, **assert on the emitted C**
(`#$ assert-c: contains("ember_vec_free")`).

## 5. The H1 callable-boundary precision matrix is next

**The first `@latebound` slice is implemented and verified in the current
worktree.** Continue according to `MIGRATION-0.9.7.md`, before starting
unrelated Phase 2 matrices. Preserve the ordinary-library status of
`std.borrow.with_views*`; extend the compiler-only callable-boundary fact
through the remaining freshness, FFI, and separate-compilation cases. Static-
independent capture-free callback results now have direct positive coverage,
and owned-capture publication has direct negative coverage. Separate sequential
invocation reuse, local callback-value escape, and all shared/mutable helper
arities are now also covered. Use existing escape
diagnostics. Do not invent named lifetimes,
runtime/ABI metadata, global callback inference, or a name-specific compiler
special case.

The LT-40 matrix now also has a reachable imported `@latebound` callable
consumer (`5aa804d`): a root module imports a support-module helper with an
`@latebound fn(Span[i32]) -> i32` parameter, and the helper is exercised through
the validated EMIF round trip. This proves the cross-module artifact path but
not independent package compilation; the current driver still checks the
loaded module graph together. FFI publication remains phase-limited because
foreign declarations and `extern fn` value types are not yet usable consumers.

The latebound matrix also accepts a callback result sourced from a named
static and stores it in `Box[str]` (`41d65ed`), preserving static provenance
without treating all callback results as invocation-local.

**The direct callable-summary slice is complete at `90059c8`.** Building on
`c913fbd`, analysis infers exact result-field provenance and parameter-field
access for direct bodies to a fixpoint. Transitive wrappers preserve the
relation, known split builtins publish both result fields as borrowing their
source, and an opaque multi-region result is `E3065/B14` rather than an
invented intersection or `static` relation. Calls use only the fields their
summary accesses; unresolved calls remain conservative. Destination
projection, not the containing local's full region vector, decides whether a
call needs a multi-region result summary. This keeps a one-region result
assigned into one field legal. E3064/B13 is retained only as a historical,
reserved diagnostic identity.

Mutation checks separately proved result routing, access precision, E3065,
and projected-destination handling are load-bearing. `af7c525` moves the
canonical record into MIR, computes a deterministic fingerprint, makes borrow
checking consume the installed record, independently rederives it from MIR,
and rejects missing/corrupt/false metadata at the final code-generation
boundary. Direct methods, generic functions, and statically monomorphized
interface bounds are covered. Generated C for the two-Span aggregate contains
no provenance metadata.

`9ade1d0` completes the smallest real cross-build `[LT-40]` slice. `EMIF`
schema-1 artifacts canonically encode and validate every installed direct
callable summary; malformed, noncanonical, trailing, stale-fingerprint, or
summary/body disagreement is a hard compiler error, never an unknown-call
fallback. BLAKE3 interface hashes and cache keys include source, compiler,
language/configuration, and transitive dependency-interface identity. The
cache is committed only after semantic validation. The LT-40 integration test
changes only an imported callable's result relation and proves the unchanged
importer's key changes; the conformance case crosses a module boundary and
asserts the metadata is absent from generated C.

This is deliberately not full `[BLD-2]`: the artifact conservatively contains
every currently compiled direct body rather than an exact export-only public
interface, the compiler still rechecks the whole loaded program, and there is
no item-granular or generic-instantiation reuse. Virtual/dynamic/FFI/closure/
coroutine and the complete escape/storage matrices remain open.

`7bcca7f` narrows schema-1's conservative callable section to the precise
import-visible boundary that current source modules can establish without a
second visibility model. `pub` and `pub(package)` top-level functions,
non-private members of visible named items, and non-private extension members
are serialized; module-private bodies retain their freshly derived MIR facts
only for the current whole-program check. A private summary change now changes
that module's source cache key but preserves both its interface hash and an
importer's key. A `pub(package)` or `pub` summary change changes both.
Schema-1 entries are recognized as incompatible tooling cache data and
invalidated before use; malformed, stale, or semantically false current-schema
records remain hard errors. The next missing public interface data is
signatures/types, then layouts/effects/inline bodies and actual reuse.

`6ad9834` supplies the first source-declaration signature slice in schema 4.
Every visible top-level function now contributes a canonical declaration
contract whether or not it has an emitted MIR body. Its resolved parameter and
result type identities, exact parameter modes, `@borrows` positions, unsafe
boundary, ABI, generic parameter positions, resolved interface bounds, and
implicit static Callable/CallableOnce bound are serialized with the existing
callable-region contract. A generic declaration that has not been instantiated
therefore has `metadata = None` rather than an invented MIR summary; a concrete
top-level body receives its independently verified summary in the same atom.
Changing a public generic bound invalidates an unchanged importer's cache key.
Private body changes retain the existing local-only behavior.

`6e37063` completes that declaration-first path for every method form the
current compiler lowers and moves `EMIF` to schema 5. Visible struct/enum
members, generic struct members, interface members, and extensions now carry
source contracts. Generic owner contracts retain a canonical symbolic receiver
identity and owner binders before method binders; interface `Self` is
interface-scoped. Concrete direct bodies attach independently verified
metadata, whereas generic/interface declarations retain `metadata = None`.
Schema-4 artifacts are incompatible tooling data and invalidate before use;
the regression proves a private member body change remains outside the helper
interface and importer cache keys.

This is still not complete public-interface or incremental-compilation work:
class-member lowering is not implemented, callable types nested inside a type
identity still need their own mode-preserving representation, and layouts,
effects, inline eligibility, non-direct targets, and actual reuse have no
approved producer / consumer path yet.

**D-116 is fixed in the same checkpoint.** `[LT-21]` field replacement now
uses a forward point-sensitive value/provenance fixpoint: an assignment
replaces the destination fact and a CFG join conservatively merges possible
facts. Old sources are released, new sources remain constrained, conditional
paths remain safe, and returned wrapper summaries contain only the sources
that can actually reach each field at `return`. Six adversarial LT-21 cases
pin these boundaries; changing replacement back into union reproduces the
original false E3021. The specification was already correct and was not
changed.

**The first multi-region core is complete at `c913fbd`.** Analysis allocates a
compile-time-only region slot for each borrowed field, preserves slots through
direct struct/nested-struct/tuple construction and copies, and computes NLL
over field slots rather than whole locals. Independent source fields shorten
independently; same-source and active matching-field conflicts remain E3021;
fixed arrays conservatively retain every element region; guard destruction
still retains RefCell state; and generated C erases all region metadata.
Mutation probes prove aggregate routing, field selection, and fixed-array
provenance are load-bearing. This remains the lower-level foundation for the
`90059c8` direct-call slice; neither checkpoint by itself proves complete
multi-region conformance.

**E3020/B2 is complete at `e0ba765`.** Direct Array iteration now borrows the
Array through the existing shared-Span producer, yields `ref T`, and releases
the borrow after the loop. A source-loop semantic marker survives HIR/MIR
desugaring and is interpreted through region holders, so conflicting mutation
receives the dedicated B2 diagnostic while a manually held iterator remains
ordinary E3021/B3. The exact UI snapshot and compiling fixed companion are
present; removing either source-loop marker makes the case red. D-070 records
the two pre-fix manifestations and the generated-C evidence.

**`SPN-API-1` is complete at `d077563`.** Four public non-prelude
`std.collections` iterator/chunk `@view` types use ordinary `Iterator` and
shared/mutable reborrows. Their monotonic cursors permit only disjoint yielded
mutable items/chunks; NLL restores the parent; partial final chunks work;
zero-width chunks assert in every profile; and safe typed raw-pointer
extraction adds no region-retention edge. Twenty SPN-4..10/TST-25 sources cover
public identity, generics, provenance, conflicts/coexistence, failure, pointer
authority, and generated-C erasure. Provenance-edge and cursor-advance
mutations both made adversarial cases fail before being restored.

**`Cell[T]` and `RefCell[T]` are both built.** The remaining buildable
Cell/RefCell coverage closed on 2026-09-12: `[CELL-6a]` now runs both fallible
borrow paths in debug, release, and shipping; `[CELL-9]` pins the one-word
counter in generated C and the every-profile runtime check; `[CELL-10]` pins
ordered positive and forbidden negative suggestions. The class-only
`exclusivity = "unchecked"` interaction has no reachable trigger until classes
and package exclusivity exist; it is a dependency gap, not a current compiler
defect. `[CELL-3]`/`[CELL-8]` remain correctly blocked on `CELL-SYNC-1`.

**D-042 is fixed:** drop elaboration now carries recursive per-field move paths,
splits partial aggregate cleanup into live-field drops, uses per-path flags for
conditional moves, preserves disjoint siblings, and emits `E3042` for a whole-
value use after a partial move. Six adversarial `[EXP-6]` cases pin exact drop
counts, conditional and call-terminator flags, nesting, reinitialisation and the
diagnostic. This is the ownership foundation 0.9.5 `[LT-38]` needs; it does not
itself implement region vectors or any 0.9.5 view rule.

The same probe found and closed **D-043**: `owned` parameters were not in the
callee's destruction scope and leaked when not moved onward. `[OWN-2]` now has
a direct move-onward/not-moved parameter test. Neither fix changed the
specification or frozen H8 target.

The `[CELL-10]` probe found and closed **D-044**: default-mode value parameters
lost their shared-borrow provenance at type checking, so writes compiled and
changed only a private ABI copy. Parameter modes now remain explicit type-
checker state across assignment, `ref mut`, mutable-view, `mut self`, and
`mut`-argument paths. E3023/B4 leads with the structural `mut` repair and only
offers costed interior mutability where non-simultaneity is proven. The rule was
already clear; no specification or ADR changed.

**What follows in Gate B:**

1. ~~**D-038:** implicit `String`→`str` coercion.~~ **Fixed.** The coercion
   reuses `view_of`, region elision, MIR view verification, and the D-037
   backend path. Three mutation-tested `[SPN-1]` cases pin assignment, argument,
   NLL, mutation, and return-region behavior.
2. **`Arena` core (`[ARN-1]`–`[ARN-4]`, `[ARN-6]`, `[ARN-7]`, `[LT-4]`).**
   **Implemented.** Growing, fixed, and scoped compiler-known arenas have
   stable aligned bump allocation, reset/rewind, nested LIFO scopes, drop-free
   enforcement, `E3090`/`E3096`, and H9 wrapper provenance. Generic
   instantiations re-check `[ARN-3]`, so `alloc(Array[T]())` cannot hide behind
   an opaque type parameter.
3. **Canonical-fact foundation.** **Started.** The smallest shared
   `TypeIdentity`/`BorrowCapability`/`AccessContract`/
   `InitializationState`/`LayoutDescriptor`/`EffectSet` representation needed
   by the next Gate B work, and route existing behavior through it without a
   big-bang rewrite. Preserve exact diagnostics, current accepted/rejected
   programs, runtime erasure, and the three different replacement orderings.
   This is `ARCH-096-1`; prove each migrated path with differential and
   adversarial tests before deleting parallel state.
4. **Remaining Arena surface.** `MaybeUninit`, `alloc_uninit`, the conservative
   core `Zeroable` predicate, both `alloc_array` branches, receiver-less
   `Default` dispatch, the complete `[TST-23]` matrix, and `[ARN-10]`
   success/abort evidence are implemented. Source-declared generic methods,
   inference, bounds, interface conformance/defaults, callable expectations,
   monomorphisation, and generic-owner return provenance are also complete
   under `GEN-METHOD-1`. `ArenaArray`/`ArenaMap` are complete under
   **`ARN-COLL-1`** at implementation commit `825eac5`. ODR-011 is closed in H2:
   the owner selected fixed-capacity,
   single-allocation views; specified their operations, failure direction,
   `!needs_drop` boundary, and Map behavior; placed the six public types in
   `std.collections` without prelude exports; defined unit-only
   `CapacityError.Full`; selected named `@view` iterators using
   `Iterator[Item = ...]`; and required empty initial construction. Allocation
   effects and `ThreadArena`
   wait for the effects/concurrency phases. **ODR-009 is closed:** H10 defines
   the zero-bit validity set, `MaybeUninit` state transitions, canonical APIs,
   and exact bulk-allocation fallback/drop behavior. H1 `[ARN-10]` clarifies
   that v1 panic aborts and requires no observable recovery or unwinding;
   rollback exists only for a separately specified recoverable transactional
   API. Built-in and user-defined `Eq + Hash` key behavior, move-only
   replacement/removal/compaction, the source-backed prelude boundary, and the
   surrounding `[TST-24]`/`[HASH-*]` matrix are executable. H4's static generic
   Hasher boundary is implemented without mandatory dynamic dispatch or a
   frozen mixing algorithm. General automatic/derived `Hash` generation and
   ordinary `Map`/`Set` remain later-phase work rather than part of this closed
   Arena checkpoint.
   Implement the H10 contract exactly; do not substitute ad-hoc initialized
   bytes or implementation-defined conversion APIs.

Arena is **not a third interior-mutability primitive**. It is a region
allocator, and amendment A13 records that it shares implementation machinery
with `Cell`/`RefCell`, not their semantic concept.

**Blocked, correctly:** `[CELL-3]`/`[CELL-8]` (`!Sync`) on `CELL-SYNC-1`.
`CELL-DEF-1` is complete: `Cell.take` and the move-only `Default` update arm are
implemented without softening the rule.

**Open compiler defects:** none.

**`UnsafeCell` is implemented at `a02c0a5`.** `[UNS-10]`/`[UNS-10a]`/
`[UNS-10b]` now cover its public `std.mem` identity, move-only one-field
representation, unsafe shared raw access, consuming extraction, ordinary drop,
view-storage prohibition, `@static_safe` exclusion, and diagnostic restraint.
It does not weaken ordinary borrowing or create runtime borrow state. The
`!Sync` and conditional `Send` statements remain an intentional dependency gap
under `CELL-SYNC-1` until threading traits exist. **Do not rebuild `RefCell` on
it**; ADR-019's compiler-known route stands.

**`MEM-API-1` is complete at `17ee5d1`.** `mem.forget` consumes locals,
temporaries, and owning containers through ordinary move semantics while
suppressing their destructor; `std.mem.align_of[T]` uses the canonical layout
descriptor after generic substitution. The later `[THR-6]` `@must_drop`
rejection remains a dependency gap, and this slice does not invent that marker.

**`SPN-API-1` advanced at `1aaa98f` and `24d8fbc`, then completed at
`d077563`.** `Span` and `MutSpan` first implemented the
specified checked `split_at`; mutable named receivers are reborrowed, direct
view-producing expressions remain valid, and both result halves retain the
original owner provenance. Array and Span splits share one MIR check shape and
one C result constructor. Explicit `MutSpan.reborrow()` now uses the same
receiver-borrow and provenance facts without copying or moving the parent.
H6 checkpoint subsequently added the named iterator/chunk families and typed
raw-pointer extraction described in §5.

**Concrete `Box[T]` is complete at `de641fb`, with the final region correction
at `0fdd9b6`.** The implemented slice is deliberately bounded to sized payloads
and the default allocator. It has the specified `T*` C representation, runtime
allocation, move-only ownership, auto-deref and owner-rooted `get`, exact
payload-drop-before-free behavior, nested/overwrite coverage, and strict C11
evidence. `[TYP-15]` is checked after region inference: static provenance
survives locals, named statics, and calls, while ordinary and Arena-backed
non-static views report E3063. D-067 through D-069 record the defects found by
testing representation and provenance rather than trusting smoke tests.

**`TUP-DST-1` is complete at `dea6aa7`.** Tuple/struct target lists now use one
explicit HIR destructuring operation and one statement-scoped aggregate
temporary. The RHS runs once before destination places; nested projections,
fresh declarations, existing assignments, wildcard residual destruction,
ordinary overwrite ordering, and non-`Copy` moves all pass adversarial cases.
Mixed declaration/assignment modes, wrong arity, and non-aggregate RHS values
are rejected. Removing the statement-temporary drop registration makes the
ignored-field destructor disappear, proving that the test observes the
required lifetime rather than merely the final values.

## 6. `[DIA-7..10]`, `[DIA-13]`, and `tests/ui/`

A Phase 2 *exit* criterion now in progress. `compiler/ember_diag/src/shapes.rs`
holds every shape; `[DIA-13]` wants a rendered snapshot per shape plus
`[PHIL-8a]`'s `.fixed.em` companion. `eeddb90` adds the reachable O5
`CallableOnce` case; 18 of 25 ownership shapes now have exact snapshots and
compiling repairs. The remaining seven await real semantic producers rather
than fabricated diagnostics.

Note the pattern four defects took: **the compiler rejects the right program
under the wrong shape**, so the *help* is wrong. D-011 (`E3064` reported as B3),
D-028 (a loop move as O1 not O3), D-034 (two mutable indices as `E3021` not
`E3022`), and D-070 (loop-held mutation as generic E3021/B3 rather than
E3020/B2). `[DIA-7a]` makes the shape part of the conformance contract, so a
snapshot per shape is what stops the next one.

## 7. Owner queue status

**Everything that stood here on 2026-09-09 has been ruled on or closed.**
ERR-041 and ERR-043 were decided by the owner on 2026-09-10; ERR-042 was
withdrawn as wrong; D5 closed with the compiler right; D-030 was fixed. What
remains:

**The queue lives in `docs/OWNER-QUEUE.md`.** ODR-001, ODR-002, and ODR-004
through ODR-014 are closed. ODR-003 is deferred editorial work. **No owner
semantic/API decision is currently open.**

* **ODR-001 — CLOSED.** `[UNS-10]`'s `UnsafeCell` API stays exactly as written.
* **ODR-002 — CLOSED as tooling work, not spec work.** The six rules stayed
  untouched. `RIDX-1` landed in `6c77723`; the extractor now recognises the
  legitimate structural forms and its tests reject reference-shaped false
  definitions.
* **ODR-003 — deferred editorial cleanup.** No semantic change for 0.8.5; the
  next suitable revision classifies each `[FFI-17]` item A/B/C/D against its
  authoritative rule.
* **ODR-004 — CLOSED.** The owner supplied `[LT-8]`–`[LT-13]` on 2026-09-12;
  H5 reproduces them with transport-only Markdown normalization and preserves
  H4 unchanged.
* **ODR-005 — CLOSED.** H6 adds distinct all-mutable
  `with_views2_mut/3_mut/4_mut` helpers using canonical `MutSpan[T]`. Shared
  helpers remain shared; no mixed `Span`/`MutSpan` overloads are implied.
* **ODR-006 — CLOSED.** H8 makes mutable helper inputs `mut` reborrows; shared
  inputs stay borrowed and neither family consumes a view.
* **ODR-007 — CLOSED.** H8 retains `Callable[Args, R]` and preserves the full
  callable mode vector as compiler-known, compile-time-only canonical type
  metadata. No new public generic or runtime mode mechanism is implied.
* **ODR-008 — CLOSED.** H9 permits `@borrows(arena)` only when a returned view
  is proven to use storage owned by that growing Arena parameter. Arena remains
  non-view; arbitrary non-view parameters remain E2031.
* **ODR-009 — CLOSED.** H10 defines `Zeroable`, `MaybeUninit`, `alloc_array`,
  and `alloc_uninit`, including bit validity, canonical value/span transitions,
  `!needs_drop`, deterministic fallback, E2040, rollback, and phase ordering.
  The remaining Arena bulk surface is now ordinary implementation work.
* **ODR-010 — CLOSED.** H1 amends `[ARN-10]` directly: `Default.default()` has
  no recoverable failure path in v1, panic terminates through `[PAN-1]`
  `abort()`, and no post-panic Arena state or unwinding is promised. No
  `[ARN-10a]` was introduced. A future recoverable API receives rollback only
  from its own explicit transactional contract.
* **ODR-011 — CLOSED.** H2 records fixed-capacity, single-allocation Arena
  provenance, empty construction, operations, recoverable
  `CapacityError.Full`, `!needs_drop` contents, Map key/duplicate/order
  behavior, and the public `std.collections` surface. The owner selected named
  `@view` iterator types using `Iterator[Item = ...]`; none of the six names is
  in the prelude. `ARN-COLL-1` is complete at `825eac5`.
* **ODR-012 — CLOSED.** H3 defines public `Hash`/`Hasher`, concrete
  `DefaultHasher`, Eq/hash coherence, read-only resident Map keys, and leaves
  the exact mixing algorithm implementation-defined.
* **ODR-013 — CLOSED.** H4 uses
  `fn hash[H: Hasher](self, mut h: H)`: `H` is inferred from the concrete
  context and normally monomorphized, `DefaultHasher implements Hasher`, and
  `Hash.hash` introduces no implicit or mandatory dynamic dispatch.
* **ODR-014 — CLOSED and implemented.** H6 defines four named public, non-prelude
  `std.collections` iterator/chunk `@view` types, exact shared/mutable reborrow
  and `Iterator.Item` contracts, all-profile zero-size panic, safe raw-pointer
  extraction, and unsafe-only pointer use. `SPN-API-1` completed at `d077563`.

* **Historical tooling lesson from ODR-002.** Six valid rules (`[TYP-26]`,
  `[IFC-2]`, `[HND-2]`, `[GPU-7]`, `[VER-7]`, `[CTL-3a]`) used structural forms
  the old extractor could not read. The specification was not rearranged for
  the tool. `RIDX-1` taught the extractor those forms and pinned the
  definition/reference boundary with tests. Withdrawn ERR-042 retains the
  original inventory.
* **No open compiler defect.** **D-038** now routes implicit `String`→`str`
  through the same explicit-borrow producer as `as_str()`. **D-042** (partial
  moves), **D-043** (owned parameters leaked), and **D-044**
  (writes through borrowed value parameters) are fixed with adversarial
  conformance evidence. **D-041** (moves out of
  borrowed places unchecked — `x = r.inner` compiled and the value dropped
  twice) was the serious one and is **fixed** this turn: borrowed-ness is
  threaded HIR→MIR and owning moves out of borrows are `E3013`.
  **D-048–D-050** close the generic pipeline defects found while completing
  `GEN-METHOD-1`: explicit arguments outrank literal defaults, solved types
  reach callable expectations independent of argument order, and
  substitution/inference recurse through `Span`/`MutSpan` while preserving
  generic-owner return provenance. **D-054–D-058** close the H4 integration
  defects: move-only Map internals, built-in bound consistency, concrete
  monomorphization diagnostics, unreachable standard-body emission, and
  imported/prelude interface identity. The specification was already correct
  in every case and no frozen specification was edited.
* **`RT-GEN-1` is complete.** Commit `d941511` generates the runtime header and
  source from the canonical branding prefix through checked-in templates and a
  deterministic generator check. The generated outputs are validated as
  outputs, while hand-authored compiler/runtime sources remain covered by the
  branding scan. Strict C11 compilation and the full workspace regression pass.
* **Three open deviations**: D1 (`[RNG-5a1]`'s generated operator impls), D3
  (`extern class` parses and is refused), and D4 (`E9012` registered and never
  emitted). D2 (`[CLO-6]`'s `owned f`) is closed by the current compiler
  checkpoint; D5 and D6 are historical closed/withdrawn entries.

## 8. Traps paid for

* **A stale binary makes a gate lie.** `error_pages.py` preferred
  `target/release/` over `debug` and validated pages against a compiler from an
  earlier session. Newest wins now, and it says which it used.
* **An assertion can satisfy itself.** A `compile-fail` case writes its expected
  code on the line that provokes it, and the renderer echoes that line — so
  `stderr.contains("E1050")` matched the test's own annotation. Every trailing
  expectation asserted nothing, twice over, because the harness did not parse
  them either. Found by writing a case with a deliberately wrong code and
  watching it pass. **After adding a test, break it once and watch it go red.**
* **A Python line-continuation eats a Rust `\`.** Two help strings shipped with
  eighteen spaces mid-sentence. Write Rust through the Write tool, never a shell
  heredoc, and check any `\` that survives a Python string.
* **A checker that knows one shape of a rule reports thirty false positives.**
  The document states a rule in at least four shapes. `hardening_check.py`'s
  attribution was wrong four ways; each time it named a *real* rule and the
  wrong one.
* **A new value-producing form must be walked through every analysis by hand.**
  D-022 was found by asking whether the borrow checker saw the view, not by a
  test failing.
* **CRLF.** The Edit tool writes CRLF on this host while `.gitattributes` pins
  LF. Normalise every changed file before every commit.
* **A name the compiler invents ends up in the C verbatim.** `Cell`'s payload
  field was called `$value` so that no Ember program could spell it. `$` in a C
  identifier is a **compiler extension**, which `[CG-C-1]` forbids relying on —
  and it compiles clean under `-Wall -Wextra`, so nothing said so until
  `-pedantic` was tried by hand. Privacy, not spelling, is what makes a field
  unreachable. **Read the emitted C after anything that puts a new identifier
  into it.**
* **A conformance directory named for a rule can test almost none of it.**
  `tests/conformance/OWN-5/` passed for as long as the rule went unimplemented
  (D-035): its one case covered the sentence's parenthetical, and was written so
  the main clause could not fire. Before trusting a green directory, read the
  rule and count its clauses.
* **`grep -c` returning zero exits 1**, which reads as a failed command in a
  chained shell line and can be mistaken for a failing test run. Twice.
* **A gate needs its real arguments.** `python tools/split_spec.py --check`
  alone throws `IndexError`; CI passes it the source and the output directory.
  `.github/workflows/ci.yml` is the authority on how each gate is invoked.

## 9. Where the code is

    compiler/ember_typeck/src/lib.rs        largest file: ranges, spans, closures,
                                            visibility, callable generics, cells
    compiler/ember_analysis/src/borrows.rs  NLL, loans, elision, the [DIA-7] shapes
    compiler/ember_analysis/src/drops.rs    moves, drop flags, [OWN-4]'s loop shape
    compiler/ember_analysis/src/regions.rs  region variables and the constraint graph
    compiler/ember_mir/src/lower.rs         MIR lowering; statement_temps is [DRP-3]
    compiler/ember_mir/src/verify.rs        structural verifier and verify_views
    compiler/ember_diag/src/codes.rs        the code registry, one entry per code
    compiler/ember_diag/src/shapes.rs       [DIA-7a]'s shape catalogue
    tools/hardening_check.py                the gate on the specification itself
