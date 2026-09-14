# Migration intake — Ember 0.9.7_Hardened_1

**State as of 2026-09-14.** This file is the implementation and adoption map
for the current frozen development target:

    docs/spec-source/Ember_v0.9.7_Hardened_1.md

It is not a second specification. The target document governs its own
requirements; `docs/spec-source/ember-spec.md` remains the repository's sole
normative source (`0.8.5_Hardened_1`) until H1 passes every adoption gate and
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
| `docs/spec-source/Ember_v0.9.6_Hardened_4.md` | immutable immediate predecessor | 772,027 bytes; 8,319 lines; SHA-256 `D32B945BAE77A612A4664DB9FCDA7F136E741599EBBB3EEC6171C9541DC57B5E` |
| `docs/spec-source/as-received/Ember_0.9.6_Simplicity_RFC_Revision_5.md` | byte-for-byte owner-approved post-H4 architecture/process RFC materialized by H5 | 27,488 bytes; 1,138 lines; SHA-256 `0AB0F9C2F4F52E10492A0CD94897BBD9C8B47C08D86E7246EA2FA29F18007525` |
| `docs/spec-source/Ember_v0.9.6_Hardened_5.md` | immutable immediate predecessor | 785,022 bytes; 8,453 lines; SHA-256 `86DDA4BC12D64BDED32B57BB6C146FD1C8A724A32705E2D1BC4D3048C8AD641B` |
| `docs/spec-source/as-received/ODR-014_Span_MutSpan_API_completion.md` | byte-for-byte owner-approved Span API ruling; H6 normalizes only shared raw-pointer spelling | 5,228 bytes; 152 lines; SHA-256 `0B2BC1BCB6450F266EC9EC264035C2F6C4A469574591CD8C9BDC8B9421A7058F` |
| `docs/spec-source/Ember_v0.9.6_Hardened_6.md` | immutable immediate predecessor | 797,288 bytes; 8,628 lines; SHA-256 `0B8611BBB52B7A806FE8536CFB832A221A27B7B010E4660F078A33563C548172` |
| `docs/spec-source/Ember_v0.9.7_Hardened_1.md` | current frozen development target | 801,902 bytes; 8,684 lines; SHA-256 `86E32DDD162F7D2CE6F3D7FA66871714CAEC508E4B07400ED18F3573448E55B3` |

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

**ODR-015 / 0.9.7 boundary.** The owner selected a language revision rather
than a hardening: `@latebound` is a callable-type modifier that gives a
callback's borrowed/view parameters fresh invocation-local regions. It is a
general library-expressible boundary, not a `std.borrow` compiler special case,
and it is absent from runtime metadata, ABI bytes, reload schemas, and named
source regions. H6 remains immutable. The required implementation evidence is
parser support, canonical callable/EMIF identity, generic substitution and
incremental invalidation, higher-ranked invocation-region enforcement, and
ordinary region/escape diagnostics for return, storage, owned-capture, and FFI
publication paths. The current worktree now has a partial **IMPLEMENTED and
VERIFIED** slice: parser/type identity, callable-bound propagation, EMIF schema
7/cache invalidation, invocation-local region markers, return rejection, and
unbounded-Box storage rejection. H1 remains frozen and **not adopted**. The
precision matrix for freshness separation, FFI publication, and
separate-compilation consumers remains incomplete. The compiler now preserves
a known capture-free callback value through a reusable latebound generic
instance, consumes its one-field result summary, and accepts a statically
independent result through `with_views2`, including `Box[str]` storage; nested
scalar, view-escape, and owned-capture publication boundaries also have direct
evidence.

ADR-034 preserves the separately approved post-H4 simplicity RFC. It directs
implementation and process toward fewer public concepts, reused authoritative
facts, derived-only caches/aggregates, and conservative unknown handling while
retaining every H4 semantic distinction needed for safety. The owner's later
clarification requires those concrete architecture, tooling, documentation,
and process changes to be materialized as H5. H4 remains immutable; H5 does
not change source semantics and is not installed as the repository-normative
source merely by existing.

ODR-014, ADR-035, and HC-096-06 preserve the subsequent owner-approved Span
iterator, chunk, and raw-pointer completion. H5 was not edited. H6 normalizes
the ruling's C/Rust-style `*const T` to Ember's existing shared raw-pointer
spelling `*T` without changing its selected authority boundary.

Any correction discovered after this freeze becomes
`0.9.7_Hardened_2`; do not amend H1 in place or silently change its identity.

## 2. Change classification

The inherited 0.9.6 architecture line is an owner-selected revision. Apart from accepting
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

The H5 cut materializes the owner-approved post-H4 simplicity consolidation.
It requires one authority per semantic invariant, derived-only aggregates and
caches, conservative unknown handling, reuse/inference review before new source
concepts, source-semantic diagnostics, generated view-contract documentation,
complexity accounting, and separate authority/evidence ledgers. H4 remains the
immutable diff base. No source-language semantic or public API is added.

The H6 cut completes the inherited Span method surface: four named public,
non-prelude `@view` iterator/chunk types under `std.collections`; exact shared
and mutable reborrow behavior; ordinary NLL/provenance/disjointness; all-profile
panic for zero chunk sizes; and safe raw-pointer extraction with use governed
by the existing unsafe contract. H5 remains the immutable diff base. No opaque
return type or parallel iterator, ownership, lifetime, or unsafe model is added.

## 3. What is implemented now

H1 is a specification target, not an adoption claim. The inherited
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
- exact `0.9`, `0.9.5`, `0.9.6`, and `0.9.7` source-selector recognition,
  while unknown patch contracts remain E0006.
- generated C runtime outputs from the canonical `ember_branding` prefix:
  `tools/generate_runtime.py` renders the checked-in C11 header/source from
  templates, and `--check` prevents stale generated output. This is the
  completed `RT-GEN-1` tooling boundary; it changes no source-language
  semantics or runtime ABI.

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
complete H6 adoption.

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

The subsequent view-surface checkpoint is `1aaa98f`. It retains 189 Rust tests
and raises executable coverage to 100 conformance directories and 337 Ember
source files. Shared and mutable `Span.split_at` now use the same MIR bounds
shape and C half-view constructor as `Array.split_at_mut`; named mutable spans
are reborrowed, direct view-producing expressions remain valid, both halves
retain owner provenance, and generic element substitution reaches pointer
arithmetic. Removing either result provenance or the explicit reborrow makes
the new adversarial cases compile incorrectly. `SPN-API-1` remains partial:
`reborrow`, `chunks`, iterators, and raw-pointer access are still absent.

The explicit-reborrow checkpoint is `24d8fbc`. It retains 189 Rust tests and
raises executable coverage to 100 conformance directories and 341 Ember source
files. `MutSpan.reborrow()` forms an ordinary mutable receiver loan, returns the
same pointer/length representation with argument-0 provenance, works in generic
and direct view-producing contexts, and leaves its parent usable after the
child's last use. Removing either the provenance edge or the receiver loan
makes the corresponding owner/parent conflict compile, so both are
mutation-tested. Shared Span intentionally remains Copy rather than gaining a
second explicit reborrow operation.

The tuple-destructuring checkpoint is `dea6aa7`. It retains 189 Rust tests and
raises executable coverage to 101 conformance directories and 347 Ember source
files. `[GRM-5]` tuple/struct target lists, including nested and parenthesized
forms, evaluate their RHS into one statement-scoped aggregate temporary before
any destination place. Fresh names declare together, existing places assign
together, fields move once, ignored non-`Copy` residuals drop at statement end,
and ordinary overwrite, visibility, borrow, and coercion rules remain shared.
Removing the temporary-drop registration leaks the ignored destructor and
turns the dedicated run-pass case red.

The concrete Box checkpoint is `de641fb`, followed by the region-completeness
fix `0fdd9b6`. The sized/default-allocator slice now lowers `Box(owned value)`
through the runtime allocator, erases its compiler-private logical wrapper to
the required C `T*`, auto-dereferences, roots `get()` in the owner, and destroys
the payload before freeing exactly one allocation. Region storage is checked
after the existing provenance fixpoint: literal/local/static/call-produced
static views are accepted, while local-array and Arena-backed views are
E3063. Coverage is now 102 conformance directories and 359 Ember sources;
`Box[dyn I]`, custom allocators, `Alloc` effects, and `Send` remain later work.

The H6 Span checkpoint is `d077563`. `SPN-API-1` is complete: the four named,
public, non-prelude iterator/chunk `@view` types are source-declared in
`std.collections` and use the existing `Iterator` interface; shared and mutable
operations form ordinary reborrows; NLL restores the parent; iterator/chunk
cursors establish monotonic, disjoint yielded mutable ranges; zero-width
chunks take an explicit all-profile MIR assertion; and safe typed raw-pointer
extraction neither accesses storage nor retains its region. Twenty new sources
cover `[SPN-4]`–`[SPN-10]` and `[TST-25]`, including generic substitution,
provenance, parent conflicts, coexisting disjoint items/chunks, partial final
chunks, pointer authority/non-retention, and generated-C erasure. Deliberately
removing result provenance or cursor advancement makes the relevant
adversarial case fail. Coverage is now 110 conformance directories and 379
Ember sources; 189 Rust tests remain green.

The E3020/B2 checkpoint is `e0ba765`. Canonical direct Array iteration now
borrows the collection for the whole loop and yields `ref T`; it no longer
moves the Array into a hidden local. The source-loop fact survives HIR-to-MIR
desugaring and is matched through the loan's region holders, which preserves
E3021/B3 for manually held iterators while selecting E3020/B2 for mutation
under a source `for`. Exact UI, fixed-companion, CTL-1, CTL-2, generated-C, and
mutation evidence close D-070. Coverage is now 112 conformance directories and
382 Ember sources; 189 Rust tests remain green.

The first multi-region-view core checkpoint is `c913fbd`. Region inference now
allocates compile-time-only slots for borrowed fields and computes backwards
liveness over those slots. Direct struct, nested-struct, tuple, copy/move, and
same-source flows preserve field provenance; independent fields shorten
independently, matching-field conflicts remain E3021, fixed arrays
conservatively retain every element region, and RefCell guard destruction
continues to retain the source needed to release runtime borrow state.
Generated C proves the slots are erased. Three mechanism mutations—collapsing
aggregate routing, selecting every field on projection, and dropping the
fixed-array conservative edge—make their adversarial cases fail and were
reverted. Coverage is now 117 conformance directories and 392 Ember sources;
189 Rust tests remain green. This checkpoint does not claim `[LT-22]`/`[LT-35]`
callable summaries, `[LT-21]` field replacement, `E3065/B14`, invalidation,
the complete escape/storage/enum/generic matrix, or MIR summary verification.

The direct callable-summary checkpoint is `90059c8`. Direct bodies now infer
exact result-field provenance and parameter-field access to a fixpoint;
transitive wrappers preserve those relations, the known split builtins publish
exact two-field provenance, and opaque multi-region results report E3065/B14.
Unknown access remains conservative, and a one-region call assigned into one
field is classified from that exact destination projection rather than the
containing aggregate. E3064/B13 is now reserved historical identity, not H6
target behavior. Four independent mutation tests prove result routing, access
precision, E3065, and destination-projection classification; generated C
again contains no proof metadata. Coverage is now 120 conformance directories
and 399 Ember sources; 189 Rust tests remain green.

The verified-MIR checkpoint is `af7c525`. Canonical callable-region metadata
now lives on each MIR body, is normalized and deterministically fingerprinted,
drives the caller-side borrow check, and is independently rederived before
code generation. Missing, corrupt, and semantically false records are hard
internal failures. Direct methods, generic functions, and statically
monomorphized interface-bound calls have adversarial coverage. The same
checkpoint replaces the old global region-edge closure with a point-sensitive
value/provenance fixpoint, closing D-116: `[LT-21]` field replacement releases
the old source, retains the new source, merges conditional alternatives, and
publishes only return-reaching sources in wrapper summaries. Coverage is now
122 conformance directories and 410 Ember sources; 189 Rust tests remain green.

`9ade1d0` adds the first real interface/cache boundary. Its versioned `EMIF`
artifact serializes canonical callable metadata, verifies canonical decoding
and the stored fingerprint, and is installed before caller-side checking.
BLAKE3 interface/cache identity includes source, compiler, language/profile
configuration, and the sorted transitive interface-hash closure. Cache writes
occur only after semantic validation; malformed, stale, or body-disagreeing
records are hard errors. An integration test changes only an imported
callable's summary and proves the importer cache key changes, while a
cross-module LT-40 conformance case proves runtime C erasure.

This still does not complete H1's summary architecture. The current artifact
conservatively serializes all compiled direct bodies rather than an
export-precise public interface, and the compiler still whole-program
rechecks: no item/generic-instantiation reuse follows from this checkpoint.
Non-direct dispatch and the full escape/storage/enum/generic matrix also
remain.

`7bcca7f` makes the callable section import-precise under the existing
`[MOD-2]` AST visibility authority. It serializes `pub` and `pub(package)`
callables (including eligible visible members), leaves module-private records
fresh and local to the current whole-program verification, and moves the
artifact to schema 2. A schema-1 file is incompatible tooling cache data and
is invalidated before it can be consumed; malformed, stale, and
body-disagreeing schema-2 records remain hard errors. The regression proves a
private relation change preserves the helper interface/importer key while
package-visible and public changes invalidate both. This completes only the
callable-summary visibility boundary: public signatures/types, layouts,
effects, inline bodies, and item-granular reuse still need real producers and
consumers.

`6ad9834` advances that same artifact to schema 4 without adopting H6 or
changing language semantics. The type checker now emits resolved declaration
facts for every visible top-level function; `EMIF` records canonical parameter
and result types, `borrowed`/`mut`/`owned` modes, `@borrows`, unsafe/ABI facts,
generic parameter positions, resolved bounds, and implicit Callable/
CallableOnce shapes. This is declaration-first: a public generic is serialized
even with no emitted specialization, and has no fake callable-region metadata.
The regression changes a public generic bound and proves the importer key
changes; direct mode/ABI/unsafe and ordinary public signature changes are also
round-tripped. Existing visible-member records remain body-derived pending the
same source-declaration collector for members, generic methods, interfaces,
and extensions. Layout/effect/inline sections and safe reuse remain open.

`6e37063` completes that collector for every method form currently lowered and
advances `EMIF` to schema 5, without adopting H6 or changing language
semantics. Visible struct/enum members, generic struct members, interface
members, and extensions now serialize declaration-first contracts. Generic
owner contracts use canonical symbolic receiver identity and retain owner
binder facts before method binders; interface `Self` is interface-scoped.
Concrete direct bodies attach independently verified callable metadata, while
generic and interface declarations use no fabricated summary. Schema-4 cache
records invalidate before use. The regression proves a generic-owner-bound
change affects the helper interface and an unchanged importer, while a private
member-body change remains local. Class-member lowering is not currently
implemented; layouts/effects/inline sections and safe reuse remain open.

The preceding H6-target implementation advances `EMIF` to schema 6
and closes the compiler's nested callable-mode erasure: each function-type
parameter now has canonical `{ type, borrowed|mut|owned }` identity through
parsing, substitution, generic `Callable`/`CallableOnce` bounds, expected
lambda checking, indirect calls, monomorphisation, generated C, and interface
serialization. Schema-5 records do not contain the complete bound contract and
are invalidated before use. A cross-module regression changes an implicit
generic bound from `fn(mut i32) -> i32` to `fn(i32) -> i32` and proves the
helper interface hash and importer cache key both change. This implements an
owner-approved frozen-target contract; it does not install H6 as the adopted
specification, create runtime mode metadata, or complete `with_views`.

The same checkpoint adds the ordinary `std.borrow.with_views2/3/4` and
`with_views2_mut/3_mut/4_mut` forwarding families. Their fixed callback modes,
reborrow behavior, generic result inference, and ordinary mutable-alias
rejection have executable LT-8 evidence. `0.9.7_Hardened_1` resolves the
previous declaration gap with `@latebound`; its first implementation slice is
now present in the worktree. The parser accepts the marker only on callable
types; callable identity, generic bounds/substitution, expected-callable
coercion, HIR/MIR call edges, canonical spelling, EMIF schema 7, and cache
invalidation preserve it. Region analysis attaches a compiler-only
invocation-specific origin to view results crossing the boundary, and the
existing return and Box-storage checks reject escaped results with E3062 or
E3063. Unknown indirect results remain conservative and no runtime/ABI fact is
introduced. Preserve this general callable-boundary fact through the existing
pipeline rather than replacing it with compiler recognition of `std.borrow`
names.

The storage conformance case is intentionally reachable from `main`: an early
version put the violating operation only in an uninstantiated implicit generic
body, which produced no MIR and falsely passed. D-125 records that test defect.
This is the standing coverage rule in executable form: a directory named after
a rule does not prove that the rule is exercised. Nested scalar composition,
nested view escape, and a statically independent callback result are also
covered directly. Separate sequential invocations now prove that callback-local
borrows do not leak into later source reuse, and a local explicitly typed
`@latebound` function-value escape reaches `E3062`. FFI and separate-compilation
precision cases remain implementation work.

The helper matrix now also has direct shared and all-mutable positive coverage
for arities two, three, and four, including generated-C erasure and sequential
source reuse. No callable-region metadata is introduced into runtime output.

The LT-40 matrix additionally contains a reachable imported latebound helper
consumer. Its `@latebound` callable parameter is declared in the support
module, serialized through the validated EMIF artifact path, and called from
the root module; the run-pass case prints `33` and checks generated-C erasure.
This is cross-module artifact evidence, not independent package compilation.
Foreign declarations and `extern fn` value types remain phase-limited, so FFI
publication is still an explicit future implementation boundary.

The matrix also covers a callback returning a `str` from named static storage
and storing it in `Box[str]`. This positive case confirms that latebound
analysis preserves genuinely static provenance instead of conservatively
tainting every view result with the callback-local region.

`5332572` also closes D-126 at the EMIF boundary. Artifact decoding now
recomputes the cache key from the serialized source/compiler/language/configuration
and dependency inputs, rejecting a self-inconsistent key before any interface
contract is consumed. This is an implementation-integrity fix under `[BLD-2]`
and `[LT-40]`; it does not change the language specification or claim that
independent package compilation is complete.

`c4fccdc` closes D-127 at the same boundary: an explicit `@borrows` vector
must contain at least one parameter under `[LT-1a]`, and EMIF contract
validation now rejects an empty vector before it can be serialized or consumed.
This is another fail-closed compiler-integrity correction; it does not alter
the language contract.

`523c030` closes D-128 at the source boundary. The type checker now rejects
`@borrows()` and every malformed/non-name argument form with the existing
`E2031` contract diagnostic instead of silently ignoring the argument or
falling through to a downstream provenance error. Three LT-1a conformance cases
cover empty, numeric, and named forms. The artifact-level D-127 validation
remains defense in depth; no specification or owner ruling changed.

## 4. Known implementation gaps

**Phase accounting:** exactly **1 of 9 phases is complete**. Phase 2 is active
and substantial but has not passed every exit criterion; Phases 3–9 have not
passed their exit gates. Historical material that separately numbered a
preparatory "Phase 0" is not part of this current nine-phase count.

### Architecture gaps inherited from 0.9.6_Hardened_1

- **`ARCH-096-1`:** the compiler does not yet exchange the complete canonical
  H1 fact set or use verified MIR as the common boundary for all named
  producers/consumers. Existing working structures are evidence to preserve,
  not permission for a big-bang rewrite.
- **`VER-096-1` is complete:** selector recognition is implemented and tested.
  This does not adopt H1 or enable a newer contract for older selectors.
- H1's exact direct callable access/provenance behavior is implemented at
  `90059c8`, including E3065/B14 and runtime erasure. `af7c525` installs and
  verifies canonical fingerprinted MIR metadata and closes point-sensitive
  `[LT-21]` replacement. `9ade1d0` serializes/validates that metadata and
  couples imported-summary changes to BLAKE3 cache identity. `7bcca7f`
  narrows the callable section to `pub`/`pub(package)` import-visible records
  and safely invalidates legacy schema-1 cache data. Public signatures/types,
  layouts/effects/inline bodies, actual item/generic reuse, dynamic targets,
  and the wider dispatch matrix remain incomplete.
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
- 0.9.5 inferred multi-region view structs are partially implemented through
  `c913fbd`, `90059c8`, and `af7c525`: direct aggregate slots, field projection,
  field-sensitive NLL, point-sensitive replacement, direct callable
  field/provenance summaries, canonical verified MIR metadata, and E3065/B14
  are executable. D-117 additionally makes a directly invoked non-`owned`
  closure preserve only the multi-region capture paths its verified body
  accesses; its environment identity and capture paths remain compiler-internal
  and absent from generated C. Interface serialization/invalidation,
  non-direct dispatch, and the full conformance matrix remain; H1's
  architecture text is not itself implementation evidence.
- **`UnsafeCell` is complete at `a02c0a5`.** It remains distinct from
  compiler-known `Cell`/`RefCell` and from Arena. Its `!Sync` and conditional
  `Send` behavior remains blocked only on the later threading-trait machinery,
  an intentional dependency gap rather than a compiler defect.
- Phase 2 UI/diagnostic snapshots and other long-standing exit work remain.
  `eeddb90` adds the reachable O5 `CallableOnce` snapshot; 18 of 25 ownership
  shapes now have executable snapshots. The seven remaining shapes currently
  await their later semantic producers rather than fabricated test-only
  diagnostics.
- **`MEM-API-1` is complete at `17ee5d1`:** `[OWN-6]` `mem.forget` and Part
  XV's `align_of` now join the existing `drop`/`take`/`replace`/`swap`/
  `size_of` subset. Only `[THR-6]`'s later `@must_drop` integration remains.

The four open deviations remain D1–D4. D5 is closed and D6 withdrawn. There is
no open compiler defect and no open owner semantic/API decision. ODR-011
through ODR-014 are closed; ODR-003 remains deferred editorial work with no
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
the completed `ARN-COLL-1` checkpoint. Other H1–H6 rules still lack complete
adoption evidence, so this progress is not evidence that the frozen target has
been adopted.

H2 intentionally has many rules without conformance directories. Arena
collection evidence now exists under `ARN-COLL-1`; the H1 architecture and
equivalence requirements still lack complete executable evidence. Their
absence is an implementation/conformance gap, not a reason to weaken the
specification. The alternate conformance/diagnostic gate is intentionally not
green yet.

Running the same audit directly against frozen H3 reports:

    972 rule IDs
    866 stated definitions
    207 named diagnostics
    208 registered diagnostics
    0 duplicate definitions
    0 orphaned amendments

H3 adds exactly `[HASH-1]` through `[HASH-4]` over H2. The classified dangling
reference and named-code inventories are unchanged. Hashing evidence now
exists for the implemented ArenaMap surface. Missing later general hashing
evidence remains implementation/conformance work, not permission to weaken the
public protocol or freeze a mixing algorithm.

The current H6 alternate-source audit reports 980 rule IDs, 874 stated
definitions, 207 named diagnostics, 209 registered diagnostics, no duplicate
definitions, and no orphaned amendments. Against the adopted-source ratchet it
reports **110 new adoption problems**, all target rules without their own
conformance directory. E3065 is now registered and has executable LT-22 plus
B14 evidence, so the former registry gap is closed; that does not imply all of
`[DIA-19]` or the wider target matrix is conformant. The four raw reference
candidates retain their established classification; no new dangling normative
reference was introduced. This alternate gate is expected to remain red until
implementation/conformance catches up and does not affect the six gates
against the adopted source.

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
8. **Completed — complete the H6 Span API.** `d077563` implements the exact
   ODR-014 iterator, chunk, reborrow, zero-size, and raw-pointer contract using
   existing Iterator/borrow/region/MIR machinery, with mutation-sensitive
   `[TST-25]` evidence and no proof metadata in generated C.
9. **In progress — complete multi-region and callable summaries through H1
   facts.** The direct region-vector/field-NLL core is complete at `c913fbd`;
   `90059c8` adds direct-call `[LT-22]`/`[LT-35]` result/access summaries, known
   split-result relations, and `E3065/B14`; `af7c525` installs canonical
   fingerprinted MIR metadata, verifies producer/consumer agreement, covers
   direct method/generic/monomorphized-interface cases, and closes `[LT-21]`
   point-sensitive replacement. `9ade1d0` adds the first real interface/cache
   artifact and `[LT-40]` identity invalidation. `7bcca7f` makes its callable
   section import-precise, `6ad9834` adds declaration-first top-level
   signatures and generic bounds, and `6e37063` completes source declaration
   contracts for every method form currently lowered. D-117 closes the narrow
   direct capturing-closure field-provenance case using that same verified
   summary boundary. The current checkpoint also gives `[LT-28]` direct
   conformance evidence: structural split proofs permit independently mutable
   fields, while two overlapping `as_mut_span` borrows remain E3022; region
   identity is not a disjointness proof. `LT-41` owns the complementary opaque
   function-value all-slot rejection. `[LT-25]` now pins the ordinary E3021
   move/borrow rejection that prevents a constructed view from borrowing an
   owner moved into its own field. D-118 makes enum discriminants control-flow
   metadata rather than payload reads: an explicit nested payload destructure
   retains only selected fields, while a whole payload binding remains
   all-slot under `[LT-36]`. The current matrix also proves `[TYP-15]`'s
   all-slots static storage boundary and `[LT-29]`'s non-owning generated-C
   destruction path. `[LT-31a]` now proves source-distinct region provenance
   does not alter a nominal direct callable's C ABI: one callable declaration
   and definition, with no region-slot ABI data. D-119 now makes `owned fn`
   capture by move/copy rather than silently lowering it as a shared borrow;
   its owned environment carries only a compiler-internal marker, requires
   static provenance for captured views, and is verifier-bound. D2's distinct
   `owned f: fn(...)` parameter boundary is now complete: a `CallableOnce`
   call explicitly moves its indirect callee and a second call is E3040 even
   for a concrete `Copy` function representation. D-121 also completes the
   separate body-level fact: an owned closure is one-shot only when its checked
   body consumes a non-`Copy` capture; plain `Callable` rejects that closure
   with E3030 while `owned f` accepts it. D-122 completes per-capture mutation:
   normal closures store only mutated captures as `ref mut`, owned closures
   mutate their moved fields through the existing `mut` mode, and `mut f`
   carries the required mutable callable place through monomorphization.
   D-123 closes the owned-callable escape boundary: a normal reference-capturing
   closure cannot cross `owned f` merely because the currently visible callee
   happens to invoke rather than store it. The current worktree additionally
   implements the first `@latebound` callable-boundary slice: parser/type
   identity, callable-bound propagation, EMIF schema 7/cache invalidation,
   compiler-only invocation origins, and return/Box publication rejection.
   D-125 records and fixes the first storage-test reachability defect. Complete
   the remaining precision and adversarial matrix—freshness, nested boundaries,
   FFI publication, and separate-compilation consumers—while preserving runtime
   erasure. Static-independent capture-free callback results now have a direct
   implementation/conformance slice. Dynamic
   dispatch, class-member lowering, layouts/effects/inline sections, and safe
   reuse remain separate prerequisites; do not publish placeholders.
10. **Run adoption validation.** `VER-096-1` may land earlier for testing, but
   H1 becomes normative only after every gate below passes and the owner
   explicitly adopts it.

At each step, use minimal adversarial programs, mutate each new test red once,
inspect generated C where order/erasure is not safely source-observable, and
record findings under the five-way classification before changing behavior.

## 7. Adoption gates

H1 must not replace `ember-spec.md` until all of these are true:

1. **Custody:** H10, H1, H2, H3, H4, H5, H6, and all as-received owner sources
   match this record; H1 has exactly one version identity and H6 as its predecessor.
2. **Specification integrity:** alternate-source rule/index, grammar/fence,
   diagnostic, cross-reference, and version-lineage audits show no unintended
   regression.
3. **Implementation matrix:** every H1 requirement is marked `SPECIFIED`,
   `IMPLEMENTED`, `VERIFIED`, or `CONFORMANT` from repository evidence; no
   version label is treated as proof.
4. **Compiler/runtime:** all required H10, H1, H2, H3, H4, H5, and H6 mechanisms exist, including
   selector support, initialization APIs, canonical facts, summaries,
   invalidation, verification, and runtime erasure.
5. **Conformance:** every H10/H1/H2 condition remains green, the `[TST-24]`
   collection matrix passes, and `[HASH-1]`–`[HASH-4]` have adversarial
   custom-key, coherence, consuming-finalization, non-retention, and
   read-only-key evidence; `[TST-25]` covers the complete Span iterator, chunk,
   reborrow, pointer, zero-size, generic-substitution, and metadata-erasure matrix.
6. **Simplicity architecture:** derived caches and aggregates are invalidated
   with authoritative facts; unknown facts grant no capability; initialization
   retains `Uninit | Maybe | Init`; no duplicate public interface or parallel
   safety system appears; and diagnostics/documentation remain derived from
   source-semantic contracts.
7. **Repository gates:** `cargo build` is warning-free, `cargo test --workspace`
   passes, and all six CI specification gates pass against the adopted source.
8. **Generated output:** C11 inspection proves portable identifiers, all three
   storage orderings, no proof-metadata ABI leakage, and required runtime
   checks/erasures.
9. **Owner action:** the owner explicitly authorizes installing H1 as
   `docs/spec-source/ember-spec.md`; generated `docs/spec/` is then regenerated,
   never hand-edited.

## 8. Standard work protocol

Before implementation, read in this order:

1. `docs/spec-source/Ember_v0.9.7_Hardened_1.md`;
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

The implementable `@latebound` precision slice is complete for the currently
available compiler boundaries. Repository evidence covers parser/type identity,
callable-bound propagation, EMIF/cache invalidation, invocation-local origin
tracking, return and mutable-return escape, unbounded-`Box` storage escape,
freshness across sequential invocations, nested-boundary separation,
owned-capture publication rejection, all shared/mutable helper arities,
statically independent capture-free callback results through `Box[str]`, and an
imported latebound consumer through the validated interface-artifact path.

The remaining `@latebound` items are explicit phase boundaries rather than
missing cases that can be completed safely in this slice:

1. foreign callback publication, which requires the later FFI function-value /
   foreign-boundary machinery; and
2. a true separate-compilation consumer, which requires independent package
   compilation and reuse rather than the current same-invocation interface
   artifact test.

Do not fabricate either boundary, add name-specific compiler behavior, expose
named lifetimes, add runtime/ABI metadata, or turn ordinary callbacks into
late-bound callbacks by inference. When one of the prerequisite phases lands,
begin its coverage with a minimal adversarial program and keep unknown indirect
results conservative. Preserve conservative all-slot behavior for unknown calls
and ordinary E3021 for precise matching-field conflicts; E3064 remains a
reserved historical identity rather than target behavior. Treat class-member
lowering and nested callable mode-preserving type identity as distinct later
prerequisites. Do not start cache reuse, layouts/effects/inline sections, or a
big-bang rewrite as part of this slice, and do not leak proof metadata into
runtime layout.
