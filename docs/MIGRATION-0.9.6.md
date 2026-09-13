# Migration intake — Ember 0.9.6_Hardened_4

**State as of 2026-09-13.** This file is the implementation and adoption map
for the current frozen development target:

    docs/spec-source/Ember_v0.9.6_Hardened_4.md

It is not a second specification. The target document governs its own
requirements; `docs/spec-source/ember-spec.md` remains the repository's sole
normative source (`0.8.5_Hardened_1`) until H4 passes every adoption gate and
the owner explicitly installs it.

## 1. Authority and custody

| Artifact | Role | Integrity |
|---|---|---|
| `docs/spec-source/ember-spec.md` | adopted normative source | unchanged by this cut |
| `docs/spec-source/Ember_v0.9.5_Hardened_10.md` | immutable architecture-line predecessor | 728,498 bytes; 7,765 lines; SHA-256 `F3FFF9C14AA3C80E57949479A1BC917FE7DF62B551B7CB8B17669D19FC5C4B14` |
| `docs/spec-source/as-received/Ember_0.9.5_Simplicity_Consolidation_RFC_Revision_5.md` | byte-for-byte owner-supplied design basis | SHA-256 `593E86A61738B32DCFAAB14875C12EA3A7BBF1A16BAD5286260EE1BAFD803923` |
| `docs/spec-source/Ember_v0.9.6_Hardened_1.md` | immutable architecture-line predecessor | 749,833 bytes; 8,007 lines; SHA-256 `8863E0088F172573B7ACF319A42F44434E8B121C2EADD8394613FCBE79C83BBA` |
| `docs/spec-source/as-received/ODR-011_Arena-backed_container_API_and_semantics.md` | byte-for-byte owner-supplied Arena collection ruling | SHA-256 `7B37DB5980911EF12DDED19144ECF65F210F5A62C4334D5E69C13D91A78DBDBE` |
| `docs/spec-source/Ember_v0.9.6_Hardened_2.md` | immutable immediate predecessor | 760,674 bytes; 8,185 lines; SHA-256 `454C38796BE0CC6A6A8E7E3F2F3C976E981D4A86F02838CA9B1FD8D1385545BF` |
| `docs/spec-source/as-received/ODR-012_Hasher_public_API_and_hashing_contract.md` | preserved owner-approved hashing ruling and canonical-mode reconciliation | SHA-256 `98337E1A382E921B74F375792C025C55E462E04C98C58E5C8ACE959DE221E367` |
| `docs/spec-source/Ember_v0.9.6_Hardened_3.md` | immutable immediate predecessor; carries the ODR-013 finding | 769,660 bytes; 8,292 lines; SHA-256 `7544B1EBB9DADE8790F500DA5D77900064BC7DE0B20E3DC48674CD2F37422396` |
| `docs/spec-source/as-received/ODR-013_Hash_static_Hasher_parameter.md` | preserved owner-approved static Hasher ruling and transport normalization | SHA-256 `20970F5C72AACD53F33584448A1AEC800A5B0126F2F63CACFFAD281F6EF60D14` |
| `docs/spec-source/Ember_v0.9.6_Hardened_4.md` | current frozen development target | 772,027 bytes; 8,319 lines; SHA-256 `D32B945BAE77A612A4664DB9FCDA7F136E741599EBBB3EEC6171C9541DC57B5E` |
| `docs/spec-source/as-received/Ember_0.9.6_Simplicity_RFC_Revision_5.md` | byte-for-byte owner-approved post-H4 architecture/process RFC; not a specification amendment | 27,488 bytes; 1,138 lines; SHA-256 `0AB0F9C2F4F52E10492A0CD94897BBD9C8B47C08D86E7246EA2FA29F18007525` |

The owner approved Revision 5 as the design basis, selected the new 0.9.6
architecture line, and separately approved the final `[ARN-10]` wording. The
direct owner ruling governs where the RFC's pending `[ARN-10]` proposal and the
final wording differ. ADR-030, HC-096-01, ERR-051, and closed ODR-010 preserve
that chain. H10 was not edited. ODR-011, ADR-031, HC-096-02, and ERR-052
preserve the subsequent fixed-capacity Arena-collection ruling and exact
completion approval. H1 was not edited.
ODR-012, ADR-032, HC-096-03, and ERR-053 preserve the subsequent public
hashing-protocol ruling. H2 was not edited.
ODR-013, ADR-033, HC-096-04, and ERR-054 preserve the subsequent static generic
Hasher-call ruling. H3 was not edited.

ADR-034 preserves the separately approved post-H4 simplicity RFC. It directs
implementation and process toward fewer public concepts, reused authoritative
facts, derived-only caches/aggregates, and conservative unknown handling while
retaining every H4 semantic distinction needed for safety. It does not amend
the language, adopt H4, edit H4, or create H5.

Any correction discovered after this freeze becomes
`0.9.6_Hardened_5`; do not amend H4 in place or silently change its identity.

## 2. Change classification

0.9.6 is an owner-selected architecture-line revision. Apart from accepting
the additive `#! language "0.9.6"` selector, it preserves 0.9.5/H10's ordinary
source accepted/rejected sets and observable semantics. It changes the required
reference-compiler and conformance architecture, not the meaning of existing
ownership, borrowing, lifetime, initialization, unsafe, FFI, ABI, reload,
coroutine, determinism, or diagnostic contracts.

The H1 architecture cut makes these changes:

1. `[IMP-7]` names one canonical semantic-fact set: `TypeIdentity`,
   `BorrowCapability`, `OwnershipGraph`, `AccessContract`,
   `InitializationState`, `LayoutDescriptor`, and `EffectSet`.
2. `BorrowCapability` separates provenance, storage identity, projection,
   region, access permission, representation kind, ownership, acquisition or
   runtime checking, unsafe authority, synchronization, validity, and escape
   constraints. Region equality is not an overlap, disjointness, or `noalias`
   proof.
3. Initial MIR is the safety-analysis substrate. Initialization, ownership,
   borrowing, and regions consume MIR; verified MIR is the code-generation
   boundary. Safety-critical transformations are verified.
4. `[MIR-REG-1]` gives callable field-access and field-to-region summaries an
   exact / audited-declared / conservative-unknown contract, complete dynamic
   target coverage, coupled invalidation, stale-metadata failure, and runtime
   erasure.
5. Definite initialization uses shared `InitializationState` facts while
   keeping `MaybeUninit`'s safe-write/unsafe-assertion contract exact.
6. MIR lowering and verification preserve three intentionally different paths:

       ordinary assignment        evaluate new -> drop old -> store new
       Cell.set                   store new -> drop old
       MaybeUninit.write          store new -> do not drop prior bytes

7. `[ARN-10]` now matches `[PAN-1]`: v1 `T.default()` has no recoverable
   failure channel, panic terminates through `abort()`, and no post-panic state
   or unwinding is observable. Rollback exists only where a separate API
   explicitly defines both recoverable failure and transactional rollback.
8. `[MOD-6]`/`[MOD-6a]` add the `0.9.6` selector while retaining older exact
   contracts.

No RFC `SIMP-*` or capability-rule family was copied into the rule index.
Existing rule IDs were amended in place, and no `[ARN-10a]` was introduced.

The H2 cut completes the previously named `[ARN-5]` surface without changing
the architecture line: fixed-capacity `ArenaArray`/`ArenaMap`, one
constructor allocation, empty initial state, exact Arena provenance,
`CapacityError.Full`, `!needs_drop` contents, ordinary borrow conflicts,
named associated-type iterators, deterministic iteration boundaries, and
`[TST-24]`. H1 remains the immutable diff base.

The H3 cut completes the hashing protocol already referenced by H2: public
`Hash`/`Hasher`, concrete `DefaultHasher`, deterministic feeding and Eq/hash
coherence, consuming finalization, non-retention of byte spans, read-only
resident keys, and an implementation-defined mixing algorithm. H2 remains the
immutable diff base.

The H4 cut closes H3's remaining callable/interface boundary: `Hash.hash` is
statically generic over `H: Hasher`, concrete hasher types are inferred and
normally monomorphized, `DefaultHasher implements Hasher`, and no implicit
dynamic/interface coercion is introduced. H3 remains the immutable diff base.

## 3. What is implemented now

H4 is a specification target, not an implementation claim. The inherited
repository baseline currently has executable evidence for:

- the existing 0.8.5 parser/type/ownership/borrow/drop subset;
- compiler-known `Cell[T]` and `RefCell[T]`, including guard lifetimes and
  runtime borrow checks for every currently reachable profile path;
- D-035 overwrite destruction and the deliberate `[OWN-5]` / `[CELL-1]`
  ordering distinction;
- partial-move/drop-path correctness through D-042 and owned-parameter cleanup
  through D-043;
- implicit `String` to `str` borrowing;
- Arena core: growing/fixed/scoped arenas, aligned bump allocation, reset,
  nested LIFO scope rewind, drop-free enforcement, diagnostics, generic
  substitution checks, and H9 `@borrows(arena)` wrapper provenance;
- the first H1 fact slice: canonical aliases for type/layout identity, shared
  initialization state, and explicit borrow-capability facts that distinguish
  Arena provenance from allocation storage identity;
- explicit method type arguments through AST, formatting, compiler-known
  Arena dispatch, nested type resolution, and diagnostics;
- source-declared generic methods and associated functions through the common
  generic-call path, including explicit/inferred arguments, bounds, interface
  conformance and defaults, callable expectations, monomorphisation, recursive
  substitution through views, and generic-owner return provenance;
- compiler-known `MaybeUninit[T]`, scalar and span initialization transitions,
  `Arena.alloc_uninit[T]`, the conservative built-in `Zeroable` branch, and
  real source-level per-element `Default` construction for
  `Arena.alloc_array[T]`;
- receiver-less interface members with concrete and bound associated calls,
  full interface-signature conformance, `Cell.take`, and `Cell.update`'s
  move-only `T: Default` arm;
- `[OWN-6]` `std.mem.take`, `replace`, and `swap` through ordinary generic,
  mutable-place, borrow, move, and `Default` checking, plus an executable O2
  diagnostic repair; and
- exact `0.9`, `0.9.5`, and `0.9.6` source-selector recognition, while unknown
  patch contracts remain E0006.

The H4 Arena-collection implementation checkpoint is `825eac5`. It has 183
Rust tests green and a green conformance runner over 95 top-level rule
directories and 305 `.em` files including support modules. The last ordinary
`cargo build` was warning-free;
the test build retains one pre-existing test-name style warning. All six
adopted-source gates are green. These figures are repository evidence, not H4
conformance counts.

The subsequent architecture checkpoint is `66d0d43`. It raises the workspace
total to 188 tests. The UnsafeCell checkpoint `a02c0a5` retains that Rust-test
count and raises executable coverage to 98 conformance directories and 320
Ember source files. Definite initialization now produces one retained
`InitializationFacts` fixpoint, verifies its entry seeds, block transfers, and
predecessor joins, then drives diagnostics from that record. The C backend now
accepts only an opaque `VerifiedMir` that binds the exact body slice and type
table checked by unconditional structural and view-provenance verification
after final body pruning. This is an incremental `ARCH-096-1` checkpoint, not
complete H4 adoption.

The subsequent diagnostic/ownership checkpoint is `c520a32`. It raises the
workspace total to 189 tests and executable coverage to 99 conformance
directories and 328 Ember source files. O2 now has an exact rendered snapshot
and compiling `mem.take` repair; OWN-6 probes cover non-`Copy` replacement,
take, swap, destructor counts, missing `Default`, live-borrow conflicts, and
same-place mutable overlap. Windows aborting tests no longer invoke an
interactive WerFault report, so the full conformance runner completes
non-interactively while preserving `[PAN-1]` termination and panic text.

The subsequent call-diagnostic checkpoint is `8f16a7f`. It retains 189 Rust
tests and raises executable coverage to 100 conformance directories and 330
Ember source files. `[FN-2a]` now emits the specified E3027/B10 identity and
bind-to-local/owner-place repair when a `mut` argument has no mutable place;
the owner-approved `[FN-1a]` mutable-view-value case remains accepted. B10 now
has an exact rendered snapshot, a compiling fixed companion, positive and
negative conformance, and an executable error page.

The subsequent memory-surface checkpoint is `17ee5d1`. It retains 189 Rust
tests and raises executable coverage to 100 conformance directories and 332
Ember source files. `mem.forget` now consumes through ordinary move semantics
without creating a drop owner, including for expression temporaries and owning
Arrays; a mutation probe that borrowed instead of moving reintroduced all
three forbidden drops and made the conformance case fail. Public
`std.mem.size_of[T]`/`align_of[T]` resolve through the module, and `align_of`
folds only after generic substitution from the canonical `LayoutDescriptor`.
Program and runtime C pass Clang C11 `-pedantic -Wall -Wextra -Werror`.
`[THR-6]`'s future `@must_drop` rejection remains assigned to its later
mechanism and was not simulated.

## 4. Known implementation gaps

**Phase accounting:** exactly **1 of 9 phases is complete**. Phase 2 is active
and substantial but has not passed every exit criterion; Phases 3–9 have not
passed their exit gates. Historical material that separately numbered a
preparatory "Phase 0" is not part of this current nine-phase count.

### Immediate H1 architecture gaps

- **`ARCH-096-1`:** the compiler does not yet exchange the complete canonical
  H1 fact set or use verified MIR as the common boundary for all named
  producers/consumers. Existing working structures are evidence to preserve,
  not permission for a big-bang rewrite.
- **`VER-096-1` is complete:** selector recognition is implemented and tested.
  This does not adopt H1 or enable a newer contract for older selectors.
- H1's exact callable access/provenance summary, invalidation, dynamic target,
  stale-metadata, and runtime-erasure requirements are not fully implemented.
- The H1 equivalence matrix across references, spans, view fields, Arena
  results, RefCell guards, and FFI views has no complete conformance evidence.

### Inherited H10 and 0.9.5 gaps

- **`GEN-METHOD-1` is complete:** source-declared generic methods and associated
  functions now share ordinary generic-call inference, bounds, callable
  expectations, interface conformance/defaults, monomorphisation, and
  diagnostics. D-048 through D-050 pin explicit-argument precedence, solved
  types flowing into callable expectations, and recursive substitution through
  `Span`/`MutSpan`, including generic-owner return provenance.
- **`ARN-INIT-1` is complete:** `MaybeUninit`, `alloc_uninit`, the conservative
  compiler-proven `Zeroable` predicate, both `alloc_array` branches, real
  `Default` associated-function dispatch, per-element construction, and the
  complete `[TST-23]` matrix are executable. This includes `[ARN-10]` success,
  abort, and no-continuation evidence. General/manual or derived `Zeroable`
  support remains assigned to its later phase; unsupported validity is never
  guessed from zero bytes or field resemblance.
- **`ARN-COLL-1` is complete at `825eac5`:** ODR-011 is closed in H2 and its
  public hashing dependency is closed by ODR-012 in H3 and ODR-013 in H4.
  Fixed-capacity, single-allocation Arena views, `CapacityError.Full`,
  `!needs_drop` contents, the minimum operations, named iterators, and Map
  key/duplicate/order behavior are all explicit. ArenaArray, built-in-key
  ArenaMap, named iterator, allocation-count, capacity, provenance, and borrow
  paths have executable evidence. The source-backed public `Hash`/`Hasher`
  interfaces, move-only `DefaultHasher`, static generic `H: Hasher` dispatch,
  custom `Eq + Hash` key selection, read-only resident keys, and move-only Map
  replacement/removal/compaction now have positive and adversarial evidence.
  The linear fixed-capacity Map is permitted not to call the hasher, and the
  concrete mixer remains implementation-defined. General derive-generated
  `Hash` and ordinary `Map`/`Set` remain later-phase work.
- **`ARN-LATE-1`:** effect, `@must_drop`, `Send`/`Sync`, and `ThreadArena`
  obligations remain assigned to their later phases.
- 0.9.5 inferred multi-region view structs, callable field/provenance summaries,
  and their full conformance matrix are not implemented merely because H1
  consolidates their architecture.
- **`UnsafeCell` is complete at `a02c0a5`.** It remains distinct from
  compiler-known `Cell`/`RefCell` and from Arena. Its `!Sync` and conditional
  `Send` behavior remains blocked only on the later threading-trait machinery,
  an intentional dependency gap rather than a compiler defect.
- Phase 2 UI/diagnostic snapshots and other long-standing exit work remain.
  Sixteen of 25 ownership shapes now have executable snapshots; the nine
  remaining shapes currently await their later semantic producers rather than
  fabricated test-only diagnostics.
- **`MEM-API-1` is complete at `17ee5d1`:** `[OWN-6]` `mem.forget` and Part
  XV's `align_of` now join the existing `drop`/`take`/`replace`/`swap`/
  `size_of` subset. Only `[THR-6]`'s later `@must_drop` integration remains.

The four open deviations remain D1–D4. D5 is closed and D6 withdrawn. There is
no open compiler defect and no open owner semantic/API decision. ODR-011
through ODR-013 are closed; ODR-003 remains deferred editorial work with no
semantic impact.

## 5. Alternate-target audit result

Running `tools/rule_index.py` explicitly against H1 and H10 produced the same
historical top-level inventory:

    960 rule IDs
    854 stated definitions
    207 named diagnostics
    208 registered diagnostics
    0 duplicate definitions
    0 orphaned amendments

H1 adds no rule-reference candidate. The extractor's four inherited raw
candidates are `IDE-2`, `IDE-5`, `IDE-10`, and `HOT-10`; ERR-042's completed
inventory classifies the first three as deliberate reservations and the last
as a historical superseded-family citation, not dangling normative references.
The classified dangling-normative-reference count is therefore zero. The inherited named-code discrepancies
reported by the alternate audit are `E3065`, `E4050`, `E4057`, `E4060`,
`E4064`, and `W4001`; the tool summary labels five as non-registry gaps because
its accepted baseline already classifies one historical case. These are not
silently resolved by this cut.

Running the same audit directly against frozen H2 reports:

    968 rule IDs
    862 stated definitions
    207 named diagnostics
    208 registered diagnostics
    0 duplicate definitions
    0 orphaned amendments

H2 adds exactly `[ARN-5a]` through `[ARN-5g]` and `[TST-24]` over H1. Their
conformance directories and executable Arena-collection matrix now exist under
the completed `ARN-COLL-1` checkpoint. Other H1–H4 rules still lack complete
adoption evidence, so this progress is not evidence that the frozen target has
been adopted.

H2 intentionally has many rules without conformance directories. In
particular, Arena collections and the H1 architecture/equivalence requirements
lack complete executable evidence. Their absence is an
implementation/conformance gap, not a reason to weaken the specification.
Against the adopted-source ratchet, the alternate rule-index gate reports
exactly **120 new adoption problems**: 119 rule IDs without conformance
directories and the inherited H10 diagnostic name E3065 without a registry
entry. The inventory/definition audit is sound, but the alternate conformance/
diagnostic gate is intentionally not green yet.

Running the same audit directly against frozen H3 reports:

    972 rule IDs
    866 stated definitions
    207 named diagnostics
    208 registered diagnostics
    0 duplicate definitions
    0 orphaned amendments

H3 adds exactly `[HASH-1]` through `[HASH-4]` over H2. The classified dangling
reference and named-code inventories are unchanged. Against the adopted-source
ratchet, the current working tree reports exactly **120 new adoption problems**:
119 rule IDs without conformance directories (including the four new hashing
rules) and inherited E3065 without a registry entry. Missing hashing evidence
is implementation/conformance work, not permission to weaken the public
protocol or freeze a mixing algorithm.

The H1 cut-time fenced-source audit found 44 Ember blocks: 19 parsed and 25 did
not. Its one new failure relative to the adopted-source baseline was Appendix
A's `0.9.6` selector. `VER-096-1` has since removed that parser failure; the
full alternate-source audit still must be rerun and recorded before adoption,
and selector support alone is not H1 conformance.

## 6. Implementation order

Avoid a flag-day compiler rewrite. Migrate one producer/consumer boundary at a
time and keep old and new decisions differentially checked until parity is
proved.

1. **Inventory and pin current facts.** Map canonical types, places/projections,
   loans/regions, access summaries, initialization/drop flags, layouts, and
   effects. Add adversarial tests for every behavior that a representation
   migration could silently change.
2. **In progress — introduce the minimal canonical fact spine.** Start with `TypeIdentity`,
   place/storage identity, `BorrowCapability`, and `InitializationState` needed
   by the active Arena work. Keep provenance distinct from storage overlap and
   keep proof metadata compile-time-only.
3. **Completed — `GEN-METHOD-1`.** Generic method syntax, inference,
   obligations, monomorphisation, and diagnostics use the same generic
   machinery as direct calls; Arena was not special-cased.
4. **Completed — `Default`, core `Zeroable`, and `MaybeUninit`.** Enforce all-zero
   validity, exact layout/drop behavior, safe initialization transitions,
   unsafe consuming assertions, and no second conversion API. Preserve the
   no-old-drop storage mode.
5. **Completed — `alloc_array`, `alloc_uninit`, and `[TST-23]`.** Preserve Arena
   provenance and `!needs_drop`; test successful default construction and
   abort-only panic without inventing unwinding or observing post-panic state.
6. **Completed — custom hashing and Arena collections.** ODR-011 through
   ODR-013 are closed; `825eac5` implements the static generic `H: Hasher`
   protocol, concrete move-only `DefaultHasher`, and custom `Eq + Hash` key
   dispatch while preserving fixed capacity, read-only keys, and an
   implementation-defined mixer.
7. **Completed — implement `UnsafeCell`.** `a02c0a5` provides the public
   `std.mem` identity, move-only ordinary representation, unsafe raw-access
   boundary, consuming extraction, `@static_safe` exclusion, and adversarial
   `[UNS-10]`–`[UNS-10b]` coverage without bypassing ordinary borrowing or
   retrofitting compiler-known Cell/RefCell.
8. **Complete multi-region and callable summaries through H1 facts.** Cover
   exact, audited-declared, unknown, generic, separate-compilation, dynamic,
   and hot-reload cases with coupled invalidation and runtime erasure.
9. **Add the version selector and run adoption validation.** `VER-096-1` may
   land earlier for testing, but H4 becomes normative only after every gate
   below passes and the owner explicitly adopts it.

At each step, use minimal adversarial programs, mutate each new test red once,
inspect generated C where order/erasure is not safely source-observable, and
record findings under the five-way classification before changing behavior.

## 7. Adoption gates

H4 must not replace `ember-spec.md` until all of these are true:

1. **Custody:** H10, H1, H2, H3, and all as-received owner sources match this
   record; H4 has exactly one version identity and H3 as its predecessor.
2. **Specification integrity:** alternate-source rule/index, grammar/fence,
   diagnostic, cross-reference, and version-lineage audits show no unintended
   regression.
3. **Implementation matrix:** every H4 requirement is marked `SPECIFIED`,
   `IMPLEMENTED`, `VERIFIED`, or `CONFORMANT` from repository evidence; no
   version label is treated as proof.
4. **Compiler/runtime:** all required H10, H1, H2, H3, and H4 mechanisms exist, including
   selector support, initialization APIs, canonical facts, summaries,
   invalidation, verification, and runtime erasure.
5. **Conformance:** every H10/H1/H2 condition remains green, the `[TST-24]`
   collection matrix passes, and `[HASH-1]`–`[HASH-4]` have adversarial
   custom-key, coherence, consuming-finalization, non-retention, and
   read-only-key evidence.
6. **Repository gates:** `cargo build` is warning-free, `cargo test --workspace`
   passes, and all six CI specification gates pass against the adopted source.
7. **Generated output:** C11 inspection proves portable identifiers, all three
   storage orderings, no proof-metadata ABI leakage, and required runtime
   checks/erasures.
8. **Owner action:** the owner explicitly authorizes installing H4 as
   `docs/spec-source/ember-spec.md`; generated `docs/spec/` is then regenerated,
   never hand-edited.

## 8. Standard work protocol

Before implementation, read in this order:

1. `docs/spec-source/Ember_v0.9.6_Hardened_4.md`;
2. this migration intake and `docs/HANDOFF.md` §0;
3. `docs/DECISIONS.md`, `docs/DEFECTS.md`, `docs/DEVIATIONS.md`,
   `docs/BACKLOG.md`, `docs/spec-errata.md`, and `docs/OWNER-QUEUE.md`;
4. the relevant compiler analyses and tests.

When behavior disagrees with a rule: reproduce it minimally, identify the exact
normative rule, classify compiler defect versus test defect versus intentional
dependency gap versus genuine ambiguity, and inspect whether the named test
actually exercises the rule. If semantics are clear, fix the compiler. If an
accepted-program, safety, lifetime, ABI, reload, effect, or observable-behavior
choice remains, stop and create an owner question. Never change the language to
make an implementation or current test easier.

## 9. Exact next task

Implement **`[DIA-7..10]` and `[DIA-13]` rendered diagnostic-shape coverage in
`tests/ui/`**. Use the existing catalogue as the contract: each required shape
gets a failing source/snapshot and a compilable `.fixed.em` companion, and any
wrong-code or wrong-help result is classified as a compiler or test defect
rather than papered over. Continue `ARCH-096-1` incrementally through real
producers and consumers. Do not adopt H4, freeze the hash mixer, broaden
`Zeroable` or `Hash` by inference, or add a second unsafe tier.
