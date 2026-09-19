# Backlog

Work that is decided but not scheduled. Nothing here starts before Part XX's
nine phases are complete — the phase order in the specification is the plan,
and this is what follows it.

Each task has a stable id, so a commit or a note can cite one. Ids are never
reused. `Gate` is the earliest phase after which the task is buildable, not a
promise about when it will be built.

Rationale for every entry, and the ones deliberately **not** planned, are in
[LIBRARIES.md](LIBRARIES.md). Read that before picking one up: several of these
cannot go in `std` at all, because `[STD-1]` holds `std.core`, `std.mem`,
`std.math`, `std.simd`, `std.span` and `std.arena` to being `@noalloc`-clean.

## Status

| | |
|---|---|
| Total | 47 tracked items — 20 must-have, 12 nice-to-have, 15 compiler-debt items |
| Completed | 11 compiler-debt items (`RT-GEN-1`, `LT-REG-1`, `RIDX-1`, `CELL-DEF-1`, `ARN-INIT-1`, `ARN-COLL-1`, `GEN-METHOD-1`, `VER-096-1`, `MEM-API-1`, `TUP-DST-1`, `SPN-API-1`) |
| Started | `ARCH-096-1`, `DIA-UI-1` |
| Blocked on phases | all library tasks; each compiler-debt row states its own gate |

---

## Must have

The ones a systems language is not credible without.

| Id | Task | Gate | Done when |
|---|---|---|---|
| **LIB-1** | `json` — `Serialize` path plus a streaming reader | 4 | round-trips the JSON test suite; the streaming reader handles a file larger than memory |
| **LIB-2** | `toml` — reader and writer | 4 | parses every `ember.toml` in the repo, and `ember_build` uses it instead of its own |
| **LIB-3** | `binary` — endian-aware readers/writers over `Span[u8]` | 2 | `@noalloc` over a caller's buffer; fuzzed against truncated input |
| **LIB-4** | `log` — levelled façade with structured fields | 4 | a `@noalloc` path for the hot case; libraries depend on the façade, never an implementation |
| **LIB-5** | `cli` — argument parsing driven by `@derive` | 4 | `ember`'s own CLI is rewritten on it |
| **LIB-6** | `yaml` — full YAML 1.2 document API | 4 | anchors, aliases, multi-document streams, merge keys, and comments preserved across a round trip. **Distinct from `std.ser.yaml`, which is in v1** |
| **LIB-7** | `regex` — DFA/backtracking hybrid | 4 | a pattern known at `comptime` allocates nothing at run time |
| **LIB-8** | `unicode` — segmentation, normalisation, case folding, width | 4 | grapheme clusters and NFD/NFKC; kept out of `std.string` because the tables are large |
| **LIB-9** | `hash` — BLAKE3, xxHash, FNV | 2 | matches the reference vectors; the `Map` hasher moves onto it |
| **LIB-10** | `random` — splitmix, PCG, seedable | 2 | reproducible for a fixed seed across platforms. Replays and tests both depend on it |
| **LIB-11** | `uuid` | 2 | v4 and v7 |
| **LIB-12** | `csv` | 2 | `@noalloc` over a caller's buffer; handles embedded newlines and quotes |
| **LIB-13** | `net` — TCP, UDP, DNS | 6 | non-blocking over the job system, not `async`, which is v2 |
| **LIB-14** | `http` — client | 6 | after LIB-13 and LIB-15. A server is a bigger commitment and waits |
| **LIB-15** | `tls` | 5 | bindings to a vetted C implementation. **Not a fresh implementation** |
| **LIB-16** | `compress` — zlib, zstd, lz4 | 5 | C FFI with an Ember streaming interface |
| **LIB-17** | `proptest` — property testing with shrinking | 4 | generative testing over the conformance corpus |
| **LIB-18** | `tracing` — span instrumentation | 4 | feeds `std.debug`'s profiler zones |
| **LIB-19** | `bench` — bootstrap confidence intervals | 4 | implements `[BEN-1]`..`[BEN-7]` properly, so `ember bench --compare` stops approximating |
| **LIB-32** | `glob` and `path-match` | 4 | file tooling; `@noalloc` matching against a caller's buffer |

## Nice to have

| Id | Task | Gate | Done when |
|---|---|---|---|
| **LIB-20** | `sqlite` | 5 | C FFI; prepared statements and a transaction guard that is `@must_drop` |
| **LIB-21** | `image` — PNG, JPEG, KTX2, DDS | 5 | decodes into a `std.gpu` format without a copy |
| **LIB-22** | `audio` — miniaudio wrapper | 5 | miniaudio is already a Phase 5 FFI fixture |
| **LIB-23** | `gltf` — cgltf wrapper | 5 | likewise a fixture |
| **LIB-24** | `font` — shaping and rasterisation | 5 | HarfBuzz and FreeType bindings, or a subset in Ember |
| **LIB-25** | `geometry` — meshes, BVH, convex hulls, spatial hashes | 6 | `std.math` deliberately stops at the primitives |
| **LIB-26** | `noise` — perlin, simplex, worley | 2 | SIMD paths under `@simd(assert)` |
| **LIB-27** | `terminal` — ANSI, raw mode, size | 4 | needed before anyone writes a TUI |
| **LIB-28** | `datetime` — calendars and time zones | 4 | `std.time` stays physics; dates are politics |
| **LIB-29** | `bigint`, `decimal` | 2 | fixed-point money that does not lose cents |
| **LIB-30** | `graph` — traversal, topological sort | 6 | over `SoA` storage |
| **LIB-31** | `state-machine` — `@derive`-driven transitions | 4 | gameplay code writes these by hand constantly |

---

## Compiler debt

Not libraries. Work the specification requires that the implementation does not
do yet, found while applying v0.5.

| Id | Task | Gate | Done when |
|---|---|---|---|
| ~~**RT-GEN-1**~~ | ~~Generate `ember_rt.h` and `ember_rt.c` from `EMBER_SYMBOL_PREFIX`~~ | — | **done 2026-09-14 in `d941511`.** `tools/generate_runtime.py` reads the canonical `symbol_prefix!` branding macro, renders the C11 header/source from `.in` templates, and supports deterministic `--check` verification. The checked-in outputs carry a generated-file marker, are validated by the generator rather than treated as hand-authored branding, and the branding checker no longer exempts them. A custom-prefix regression proves the emitted identifiers and include name change together; strict C11 compilation and the workspace/conformance suites remain green |
| **OBJ-RT-1** | Phase 3 object header and reference-count runtime boundary | 3 | **Started 2026-09-14 in `14d55fe`; compiler foundations in `a63ff1d`, `6a281f3`, `9205a94`, `4e2b63f`, `1ea9af3`, `74f15c3`, `97d8131`; class-handle ownership in `0f0c555`; empty/scalar-field construction in `fea9663`/`a289b2b`; the supported custom-`init` constructor subset, including branch-joined `if`, exhaustive `match`, conservative `while`/`for` analysis, state-aware whole-`self` use, and derived construction through direct `super.init(...)`, is added in the current working checkpoint.** The generated C11 runtime exposes the `[OBJ-1]` 24-byte header and `[RT-3]` type-info layout, object allocation, plain/atomic strong and weak retain/release, weak upgrade, deinitialisation with the two `[OBJ-5]` resurrection checks, base-chain downcast, and the !Sync exclusivity access word. `ember_types` has nominal `ClassId`/`ClassDef`/`TyKind::Class` identity and pointer-sized counted-handle properties. The type checker now collects non-generic class fields, openness, and single-inheritance identity; inherited field reads and read-only methods lower through generated C object structs with base fields first. Class-handle copies and class upcasts now retain, and class drop points release, with branding-derived runtime spellings. Empty and non-inheriting memberwise classes now construct through the existing object allocator, initialize fields through visible MIR assignments, and class fields that need destruction are released by generated field-drop glue. A source `drop(mut self)` in this narrow no-base/no-`init` subset is adapted to the runtime callback ABI through a compiler-generated handle-slot adapter and runs before field destruction. Receiver-rooted class-field writes from `mut self` methods now use the narrow dynamic access slice; non-mutating or distinct-object writes remain explicitly rejected. Static access elision and complete dynamic exclusivity remain open. Strict Clang C11 compilation, focused type/type-checker/MIR/codegen tests, compile-pass/run-pass coverage, and branding/generator regressions pass. Defaulted derived fields, inherited drop chaining, dynamic exclusivity, generic classes, and the complete Phase 3 conformance matrix remain outstanding; this is a foundation, not class implementation completion. The CI regression at `6415303` was a test-fixture correction only: the inherited-drop case now asserts one derived destructor followed by one base destructor and uses adapter-specific order anchors. The current checkpoint also admits inherited `mut self` method calls through a borrow-preserving derived-to-base receiver cast (`ClassUpcastBorrowed`), with generated-C no-retain coverage; virtual/override dispatch is implemented, while indexed class-object access, interface/dyn dispatch, static elision, generic classes, and the complete Phase 3 matrix remain open |
`OBJ-RT-1` continuation: mutable arguments rooted in indexed class handles
now use one evaluated MIR place for both the mutable reference and the
dynamic exclusivity access interval. The type checker no longer rejects a
valid place such as `increment(items[next_index(state)].value)`, and lowering
preserves the index's ordinary bounds check and source evaluation order. The
side-effecting-index regression is recorded as D-136. Virtual/override
dispatch is now implemented for class vtables; static access elision,
interface/dyn dispatch, generic classes, and the complete Phase 3 conformance
matrix remain open.

The compiler now also lowers class-handle identity checks end to end. `is` and
`is not` remain distinct from value equality, accept equal or related class
handles through the existing upcast path, and compare the pointer
representation in generated C. Non-class operands are rejected rather than
being silently treated as value comparisons. The focused identity fixtures
pass in debug, release, and shipping. Virtual/override dispatch is now
implemented separately below; dynamic downcasts, static access elision,
generic classes, and the complete Phase 3 conformance matrix remain open.

The dynamic downcast slice is now implemented as well. `as?` accepts related
class handles and returns `Option[Target]` after one runtime base-chain query;
success retains the returned handle and failure constructs `None`. `as!` uses
the same query and aborts through `[PAN-1]` when it returns null. Unrelated
classes are rejected statically. The positive, negative, and forced-failure
fixtures pass, and generated C shows the expected single query and owning
retain. Virtual/override dispatch is implemented; indexed class-object access,
static access elision, interface/dyn dispatch, generic classes, and the
complete Phase 3 conformance matrix remain open.

The class virtual-dispatch slice is now implemented under D-153. Type checking
assigns deterministic base-first slots and carries the declaring class and slot
through HIR/MIR. The C backend emits per-class vtable layouts, derived-prefix
compatibility, override adapters for nominal receiver types, and type-info
vtable pointers. Inherent `extend Class:` blocks now contribute virtual and
override declarations in source/module order as well, with the same override
diagnostic boundary. A base-typed call therefore selects the derived override
at runtime; the focused run-pass fixtures cover class-body and extension
declarations in all three profiles. This does not claim interface/dyn
dispatch, devirtualisation reporting, static-access elision, generic classes,
or Phase 3 completion.

The earlier row's “defaulted derived fields” wording is superseded by the
2026-09-15 D-140 continuation: supported explicit derived constructors now
materialize inherited and declared defaults in physical base-first order.
Inherited drop chaining remains implemented; dynamic exclusivity, dispatch,
generic classes, and the complete Phase 3 conformance matrix remain open.

| ~~**LT-REG-1**~~ | ~~Real region variables with a constraint graph~~ | — | **done 2026-09-09.** `compiler/ember_analysis/src/regions.rs`; `[LT-1]`'s elision is in at the call site and in the body (`E3062`). `[LT-2]`'s view structs and `[LT-7]`'s callback regions build on it |
| **LNT-CFG-1** | `[MAN-3]`'s `[lints]` configuration | 2 | `[LT-1b]`'s `L3014` is an opt-in lint and there is nowhere to opt in |
| **TST-6-1** | Appendix A's fixture as `compile-pass` | 4 | it is held to `--syntax-only` today because the appendix names `Entity`, `Formatter`, `SoA`, `Arena` and `Mutex`, which `std` does not yet have |
| ~~**CELL-DEF-1**~~ | ~~`Cell[T].take()`, and `update`'s `T: Default` arm~~ | — | **done 2026-09-13.** Receiver-less `Default.default()` resolves through ordinary interface identity; `take` constructs a replacement before moving out the old value, and non-`Copy` `update` parks the old value behind a default placeholder before its borrowed callback runs, then stores the result before dropping the placeholder. Positive move-only/destructor cases and negative missing-capability cases are under `tests/conformance/CELL-1/` |
| **CELL-SYNC-1** | `[CELL-3]`/`[CELL-8]`/`[UNS-10]` threading traits for interior-mutability cells | 4 | `Cell[T]`, `RefCell[T]`, and `UnsafeCell[T]` are `!Sync`; `Cell[T]` and `UnsafeCell[T]` may move between threads when `T: Send`, while RefCell's synchronized equivalents are `Mutex[T]`/`RwLock[T]`. There is no `Send`, no `Sync` and no thread in the compiler, so there is nothing for these markers to mean yet and nothing that could violate them — this is an intentional dependency gap, not a compiler defect. Done when `[THR-1]` exists and programs sharing these types across threads are refused |
| ~~**RIDX-1**~~ | ~~Rule extraction: teach `rule_index.py` the forms the document already uses~~ | — | **done in `6c77723`.** The six valid definitions left the baseline, no rule prose moved, and `tools/test_rule_index.py` proves both recognition and rejection of reference-shaped false definitions. ODR-002 is closed |
| ~~**BOX-OWN-1**~~ | ~~Concrete sized/default-allocator `Box[T]` ownership slice~~ | — | **done 2026-09-13 in `de641fb` and `0fdd9b6`.** `Box(owned value)` allocates through `ember_alloc`, emits the required `T*` representation, auto-dereferences for fields/methods/indexing, exposes an owner-rooted shared `get`, remains move-only independently of `T`, and drops `T` before freeing. Region inference—not expression syntax—decides `[TYP-15]`: static views survive bindings/statics/calls; local and Arena-backed views are rejected with E3063. Eight DRP-6 cases plus four TYP-15 cases cover explicit generics, nested/free counts, overwrite, temporary lifetime, moves/borrows, and both region directions under strict C11. `Box[dyn I]`, `Box[T, A]`, `Alloc` effects, and conditional `Send` remain later dependent work and are not claimed here |
| ~~**SPN-API-1**~~ | ~~Span/MutSpan view-method surface (`split_at`, `chunks`, `iter_mut`, `reborrow`, `as_ptr`, …)~~ | — | **done 2026-09-13 in `d077563`, building on `352ea64`, `1aaa98f`, and `24d8fbc`.** The four public, non-prelude `std.collections` iterator/chunk `@view` types use ordinary `Iterator`, shared/mutable reborrows, transitive owner provenance, NLL, and cursor-based `[BRW-5]` disjointness. Shared and mutable chunks include partial final chunks and an all-profile MIR zero-width assertion. Safe `as_ptr`/`as_mut_ptr` extraction returns typed raw pointers without extending the source lifetime; pointer use remains unsafe. Twenty new SPN-4..10/TST-25 sources cover public identity, generics, borrowing, coexistence/conflicts, zero width, raw-pointer authority/non-retention, and generated-C erasure. Removing the iterator provenance edge or chunk cursor advance makes the corresponding adversarial case fail. The source declarations own the public types; compiler-known lowering is a bootstrap implementation detail, not a parallel abstraction |
| ~~**TUP-DST-1**~~ | ~~Tuple destructuring assignment~~ | — | **done 2026-09-13 in `dea6aa7`.** `[GRM-5]` target lists now destructure tuples and structs, including parenthesized and nested forms. One explicit HIR operation retains a statement-scoped aggregate temporary, so the RHS is evaluated once before destination places, ignored non-`Copy` fields drop at statement end, and projected fields move exactly once. Fresh names declare together, existing places assign together, mixed modes and duplicate fresh names are rejected, ordinary overwrite/visibility/borrow/type rules are reused, and arity/non-aggregate failures are checked. A deliberate removal of the temporary-drop registration leaks the ignored destructor and makes the conformance case fail |
| **DIA-UI-1** | `[DIA-7..10]`, `[DIA-12]`, `[DIA-13]`, and `[PHIL-8a]` rendered diagnostic catalogue | 2 | **Started 2026-09-13.** `compiler/ember_driver/tests/ui.rs` checks exact `.stderr` output, verifies the emitted code maps to the directory's §XX.6.1 shape, checks a shape-specific primary-help construct, rejects orphan companions, and compiles every `.fixed.em`. `eeddb90` adds the reachable O5 `CallableOnce` snapshot, exact repair, and compiling companion; 18 of the 25 code-keyed ownership shapes now have executable cases. `e0ba765` adds E3020/B2 with canonical Array borrowing, an exact snapshot, a compiling repair, and mutation evidence; B1 has a compiling `Array.split_at_mut` repair, O2 compiler-known `mem.take`, and B10 the `[FN-2a]` bind-to-local repair. The seven remaining ownership shapes await closure, class, disjointness, effect, or concurrency mechanisms and must not be fabricated. The R1 overlay, all N1–N12 basic shapes, `ember explain --borrow`, and the final `rule_index.py` completeness gate also remain. D-060 through D-066 and D-070 were exposed and fixed by these cases and their adversarial follow-ups rather than normalized into snapshots |
| ~~**MEM-API-1**~~ | ~~Complete the remaining `std.mem` ownership/layout surface~~ | — | **done 2026-09-13 in `17ee5d1`.** `mem.forget(owned value)` now uses the ordinary move pipeline and deliberately creates no replacement owner, so locals, temporaries, and owning `Array` values are not dropped; subsequent use of a non-`Copy` source is E3040. Public `std.mem.size_of[T]` and `align_of[T]` paths exist, and `align_of` folds only after generic substitution through the canonical `LayoutDescriptor`. Generated C is portable C11 and contains neither hidden destructor/free calls nor backend-side `_Alignof` recomputation for the tested layout. `[THR-6]`'s future `@must_drop` rejection remains in the later threading/capability phase; this completion does not invent that marker or claim it early |
| ~~**ARN-INIT-1**~~ | ~~Initialization types/interfaces required by Arena bulk allocation~~ | — | **done 2026-09-13.** Compiler-known `MaybeUninit[T]`, scalar/span write and consuming unsafe conversion APIs, Arena `alloc_uninit`, bounds/provenance enforcement, the conservative built-in `Zeroable` predicate, and both `alloc_array` branches are executable. `!needs_drop` is checked first; `Zeroable` precedes `Default`; the Default arm emits per-element constructor calls; E2040 handles neither; and the complete `[TST-23]` matrix covers initialization, conversion, layout, destruction, capability precedence, rejection paths, and `[ARN-10]` success plus all-profile abort/no-continuation behavior. General/manual or derived `Zeroable` support remains assigned to its later phase; completing this core predicate does not permit validity to be guessed from field shape or claim that later derive surface |
| ~~**ARN-COLL-1**~~ | ~~`[ARN-5]` `ArenaArray` and `ArenaMap`~~ | — | **done 2026-09-13 in `825eac5`.** Both are genuine fixed-capacity, single-allocation arena-backed `@view` collections with `[TYP-15]` provenance, not owning wrappers with hidden Arena pointers. Array and Map implement the H2 minimum operations, `CapacityError.Full`, `!needs_drop` boundaries, named iterators, provenance/borrow enforcement, zero/full/reuse behavior, and exact allocation-count paths. H3/H4's public `Hash`/`Hasher` protocol and move-only `DefaultHasher` use the ordinary static generic/interface pipeline; `Hash` is prelude while `Hasher`, `DefaultHasher`, and the Arena collection names are not. ArenaMap accepts compiler-known and user-defined `K: Eq + Hash`, calls the selected `Eq.eq` for custom keys, keeps resident keys read-only, and preserves move-only keys/values through replacement, removal, and compaction. `[TST-24]` and `[HASH-1]`–`[HASH-4]` carry positive and adversarial evidence. The current linear Map is permitted not to call the hasher; its exact mixing algorithm remains implementation-defined. General derive-generated `Hash` and ordinary `Map`/`Set` remain their later phase work and are not claimed here |
| **ARN-LATE-1** | Arena obligations whose mechanisms belong to later phases | 4/6 | allocation effects and `@noalloc` behavior are enforced by the effect system; scoped guards participate in the real `@must_drop` mechanism; and `[JOB-5]` `ThreadArena` is implemented only with the concurrency/job model. The current core does not claim these later-phase obligations |
| ~~**GEN-METHOD-1**~~ | ~~Explicit type arguments on method calls~~ | — | **done 2026-09-13.** The AST/parser/printer/formatter preserve `recv.method[T](...)`; compiler-known and source-declared methods share explicit/inferred type selection, bounds, diagnostics, concrete instantiation, and deterministic symbols. This covers concrete and generic owners, receiver-less associated functions, interface-bound dispatch, generic/default interface methods, callable parameters, and view-provenance signatures. The mechanism reuses ordinary generic-call rules; no Arena-specific parser or monomorphisation path was added. Adversarial coverage is under `tests/conformance/TYP-17/` and `TYP-18/` |
| **ARCH-096-1** | Implement 0.9.6's canonical semantic-fact architecture | 2/3 | **Started 2026-09-13; verified boundaries landed in `66d0d43`, `c913fbd`, `90059c8`, `af7c525`, `9ade1d0`, `7bcca7f`, `6ad9834`, `6e37063`, `5332572`, `c4fccdc`, `523c030`, and `7291248`.** `TypeIdentity` and `LayoutDescriptor` name the existing canonical representations; `BorrowCapability`, `AccessContract`, storage/provenance, permission, representation, ownership, checking, unsafe-authority, synchronization, validity, escape, and shared `InitializationState` facts exist, and Arena loans carry distinct provenance/storage facts. Definite initialization retains one verified `InitializationFacts` fixpoint, and the C backend accepts only an exact `VerifiedMir` body/type pairing. The region solver retains compile-time-only per-field slots, field-sensitive NLL, conservative fixed-array provenance, and runtime erasure. It now uses one point-sensitive value/provenance fixpoint so `[LT-21]` assignment replaces a field fact while CFG joins merge alternatives. Direct bodies infer exact parameter-field access and result-field provenance summaries; canonical metadata is installed on MIR, deterministically fingerprinted, consumed by callers, independently rederived, and rejected before code generation when missing, corrupt, or semantically stale. `9ade1d0` serializes that canonical metadata into validated `EMIF` artifacts and derives BLAKE3 transitive dependency/cache identities. `5332572` additionally makes artifact decoding recompute and validate the serialized cache key before any decoded contract is consumed. `c4fccdc` rejects an empty explicit `@borrows` vector at EMIF contract validation, and `523c030` rejects empty or malformed source `@borrows` arguments, preserving `[LT-1a]`'s one-or-more parameter-name invariant at both the source and artifact boundaries. `7291248` routes the loan-side RefCell guard consumer through canonical `BorrowCapability.reference_kind`; the place-based guard classifier remains only for prospective accesses, with RefCell regression behavior unchanged. `7bcca7f` scopes the callable section to `[MOD-2]` import-visible `pub`/`pub(package)` bodies, preserves private MIR facts locally, invalidates schema-1 cache records safely, and proves private vs package/public summary changes affect cache identity correctly without leaking metadata into C. `6ad9834` adds schema-4 source declaration contracts for visible top-level functions, including generic bounds and declaration-only generic records, plus exact direct parameter modes, ABI, unsafe status, `@borrows`, and resolved canonical type spelling. `6e37063` advances EMIF to schema 5 and adds source declaration contracts for every method form currently lowered: visible struct/enum members, generic struct members, interface members, and extensions. Generic owners retain canonical symbolic receiver identity and owner binders before method binders; concrete direct bodies attach independently verified metadata while generic/interface declarations have no fabricated summary. Schema 4 invalidates before use, and a private member change remains local. Class-member lowering, layouts, effects, inline eligibility, real item/generic reuse, non-direct dispatch, remaining enum/escape matrices, `OwnershipGraph`, `EffectSet`, complete borrow/ownership producer-consumer migration, and the equivalence matrix remain. Preserve all accepted/rejected 0.9.5 behavior and the three distinct write orderings; do not create placeholder facts with no real producer and consumer |
| ~~**VER-096-1**~~ | ~~Accept the `#! language "0.9.6"` selector~~ | — | **done 2026-09-13.** The parser accepts the exact `0.9`, `0.9.5`, and `0.9.6` contracts, retains every older supported selector, and rejects an unknown `0.9.7` with E0006. This is selector recognition only; the current H6 target remains frozen and non-normative until its adoption gates and explicit owner action complete. The later 0.9.7 selector is covered by the H1 implementation checkpoint, not retroactively folded into this historical task |

`OBJ-RT-1` current checkpoint also includes defaulted memberwise construction
for non-inheriting classes: omitted fields are materialized from the source
default expression and checked at the construction site using ordinary
expression typing and coercion. Explicit derived constructors now flatten and
materialize inherited and declared defaults in physical base-first order before
the constructor body; `super.init(...)` then preserves the existing inherited
initialization boundary. The regression
`tests/run-pass/class_inherited_defaults.em` covers a base default read inside
the base constructor and a derived default read after `super.init(...)` in all
profiles. Defaulted derived construction is no longer fail-closed; virtual or
interface dispatch, indexed access sharing, static elision, generic classes,
and the complete Phase 3 conformance matrix remain open.

`OBJ-RT-1` continuation: custom `init` now accepts default expressions on a
non-inheriting class. HIR carries the checked defaults, MIR materializes them
after allocation and before the constructor body, and definite-initialization
checking treats them as already live. An explicit assignment to such a field
uses ordinary overwrite lowering, preserving `[OWN-5]` drop-before-store
ordering. The later inherited-default continuation extends the same metadata
through the base-first object layout.

`OBJ-RT-1` continuation: inherited source-destructor chaining is now emitted
for concrete single-inheritance classes. Release invokes the most-derived
destructor followed by each base destructor, including when the derived class
declares no destructor of its own. Generated field-drop glue remains after
that chain and keeps derived-before-base field order. Defaulted derived
construction, dynamic exclusivity, dispatch, generic classes, and the complete
Phase 3 conformance matrix remain outstanding.

`OBJ-RT-1` continuation: class-valued field receivers can now invoke a
`mut self` method through the ordinary mutable-place call pipeline, with the
containing object and callee receiver protected by their respective access
boundaries. The shared write checker also now enforces class `pub(read)`
visibility from outside the declaring module; the previous class branch had
made that check unreachable. These are implementation-only continuations;
static elision, indexed access sharing, dispatch, generic classes, and the
complete Phase 3 matrix remain open.

The constructor dataflow also covers `while`/`for` `else` blocks. The `else`
path is checked from the join of loop entry and body state, preserving the
zero-iteration case; a body-only initialization fact cannot make a field
readable in `else`. `break`/`continue` and other unsupported constructor exits
remain fail-closed.

`ARCH-096-1` continuation checkpoints: `d35e94f` routes Arena allocation-return
escape checking through canonical `StorageIdentity::ArenaAllocation` ownership,
`30836ee` makes the same checker honor the canonical
`EscapeConstraint::MustNotOutliveStorage` constraint, and `90ddefb` routes the
borrow scope, iterator-retention, and diagnostic keeper consumers through the
canonical `ValidityInterval`. The source-type fallback for ordinary Arena
place borrows remains. `498938b` also routes parameter-versus-local escape
diagnostic classification through canonical `ProvenanceRoot`, while retaining
the MIR local only for presentation and type lookup. These preserve behavior
and are implementation-architecture migrations under `[IMP-7]`; no new
semantic fact or placeholder consumer was introduced.

`OBJ-RT-1` continuation: class `mut self` methods now have a narrow executable
access interval. Receiver-rooted class-field writes are accepted, explicit MIR
`BeginAccess`/`EndAccess` statements lower to the existing checked runtime
counter, and non-mutating or distinct-object writes remain fail-closed. This
does not claim static exclusivity/elision, inheritance/dispatch, or complete
Phase 3 object semantics; those remain open work under the same backlog item.

The call-boundary slice also admits a direct class-field place supplied to a
`mut` parameter and emits a checked begin/end interval around the call. It is
covered in all profiles. Indexed class-object roots, mutating methods invoked
through class fields, instantaneous direct field writes, access elision, and
the complete Phase 3 exclusivity matrix remain fail-closed or outstanding.

The instantaneous class-field slice also admits direct `Copy` field writes
through a class-handle local or mutable reference without emitting a runtime
access pair. Non-`Copy`, nested/indexed class-object, and read-only forms stay
fail-closed; view-typed field arguments and full static/dynamic access
classification remain later work.

`OBJ-RT-1` continuation: direct class handles stored in a growable `Array`
now retain at the `Array.push` copy boundary, so the container owns a strong
reference independently of the source local. The indexed class-method
regression `class_indexed_mut_method.em` also confirms that an element remains
valid through mutation and final buffer destruction. Aggregate values containing
class handles, virtual/interface dispatch, indexed access sharing for dynamic
exclusivity, static elision, generic classes, and the complete Phase 3 matrix
remain open; this fix does not claim those broader cases.

`OBJ-RT-1` continuation: inherited `mut self` calls through a class-valued
field now preserve both access boundaries. The generated derived-to-base borrow
cast is unwrapped only for locating the containing class place, so the caller
opens its long-term access while the inherited base method opens the callee
object access. `class_field_inherited_mut_method.em` covers the two generated
access pairs in all profiles. Virtual/interface dispatch, indexed access
sharing, static elision, generic classes, aggregate class-handle ownership,
and the complete Phase 3 matrix remain open.

`OBJ-RT-1` continuation: copied aggregates now retain nested class handles at
every C copy boundary covered by the backend. The recursive walk handles
structs, tuples, fixed arrays, and active enum payloads for ordinary `Copy`
assignments, aggregate construction operands, and `Array.push`; moves keep
ownership transfer semantics. `MaybeUninit` is excluded because its storage is
not an initialized owner. `array_copy_struct_class_handle.em` proves that a
class handle nested in a copied struct survives the source scope and remains
valid in the Array. Virtual/interface dispatch, indexed access sharing, static
elision, generic classes, and the complete Phase 3 matrix remain open.

`OBJ-RT-1` continuation: memberwise classes now accept named field arguments in
the same way as memberwise structs, including out-of-order field selection and
duplicate-field diagnostics. `class_named_memberwise_construct.em` covers the
new path in all profiles. User-defined class `init` now reuses the same
named-parameter binder, including source-order evaluation metadata for
out-of-order arguments; `class_named_init_construct.em` and its compile-fail
companion cover the positive, evaluation-order, unknown-name, duplicate, and
positional-after-named boundaries. Virtual/interface dispatch, indexed access
sharing, static elision, generic classes, and the complete Phase 3 matrix
remain open.

The direct-call argument boundary is now also implemented under `[TYP-25]`:
ordinary, qualified, associated, inherited-bound, generic, and generic-method
calls bind named arguments to parameter names, evaluate expressions in source
order, and lower operands in declaration order. Unknown names and
positional-after-named arguments remain diagnostics, and duplicate parameters
are rejected. Function-value calls intentionally remain positional because
callable types do not carry source parameter names. `named_function_arguments.em`,
`named_method_arguments.em`, and the compile-fail rejection fixture provide
adversarial coverage. User-defined class-`init` construction is covered by the
class-specific positive and negative fixtures above; virtual/interface
dispatch, indexed access sharing, static access elision, generic classes, and
the complete Phase 3 conformance matrix remain open.

`RNG-4` continuation: comparison facts are now refined in `if` arms for the
existing interval lattice. Integer comparisons against constants, including
conjunctions in the true arm and safe negated bounds in the false arm, can now
prove a range construction without a redundant check. D-150 extends the same
mechanism to finite floating comparisons using conservative closed bounds for
strict inequalities. Branch facts are isolated and joined conservatively;
disjunctions remain unchanged. D-141 and D-150 record the compiler defects
and their adversarial conformance cases. No specification or owner decision
changed.

`RNG-4` continuation: canonical `std.math` `min_i32`, `max_i32`, `clamp_i32`,
and their `f32` counterparts now transfer interval facts through their resolved
direct-call definitions. `clamp` can establish its output bounds even when the
clamped value has no prior fact, provided the bound intervals prove `lo <= hi`;
unknown or mixed-representation calls remain fail-closed. D-142 records the
compiler defect and `accept_math_range_refinement.em` covers a parameter clamp
and a branch-refined min/max chain. No specification or owner decision changed.

`RNG-4` continuation: division now transfers a quotient interval when the
divisor is wholly positive or wholly negative and all endpoint computations are
representable. Zero-capable divisors and the signed `MIN / -1` overflow case
remain unknown, so the ordinary runtime checks are preserved. D-143 records the
compiler defect; `accept_division_range_refinement.em` covers integer and float
quotients, and `reject_division_unknown_divisor.em` preserves the fail-closed
boundary. No specification or owner decision changed.

`RNG-4` continuation: remainder now transfers a conservative integer interval
when its divisor is wholly non-zero, using the divisor's maximum magnitude and
the dividend's sign. It does not assume `%` is corner-monotone. Zero-capable or
unknown divisors remain unknown and preserve ordinary runtime checks. D-144
records the compiler defect; the division refinement fixture covers both the
positive and fail-closed paths. No specification or owner decision changed.

`RNG-4` continuation: shifts now transfer intervals when the amount is proven
non-negative and below the operand width. Left shifts use checked endpoint
arithmetic and therefore do not claim a fact across mathematical overflow;
right shifts use sign-aware arithmetic-shift bounds. D-145 records the compiler
defect, with `accept_shift_range_refinement.em` and
`reject_shift_unknown_amount.em` covering the positive and fail-closed paths in
all profiles. No specification or owner decision changed.

`RNG-4` continuation: exact integer bitwise constants now transfer their exact
interval, and `&` with an exact non-negative mask transfers `0 ..= mask`. General
OR/XOR, sign-bearing masks, and unknown operands remain conservative. D-146
records the compiler defect, with `accept_bitwise_range_refinement.em` and
`reject_bitwise_unproven_range.em` covering the positive and fail-closed paths.
No specification or owner decision changed.

`RNG-4` continuation: unary negation now reverses a safely negated interval,
and exact integer `~` now transfers its exact complement. Integer overflow,
non-finite float endpoints, boolean `not`, and general complements remain
conservative. D-147 records the compiler defect with
`accept_unary_range_refinement.em`. No specification or owner decision changed.

`RNG-4` continuation: `&` now also transfers two known non-negative operand
intervals as `0 ..= min(upper bounds)`. Intervals that may include negative
values and general OR/XOR remain conservative. D-148 records the compiler
defect; the pairwise case is covered by `accept_bitwise_range_refinement.em`.
No specification or owner decision changed.

`RNG-4` continuation: non-negative interval `|` and `^` operands now transfer
an all-bits upper mask derived from the larger operand upper bound. Negative-
capable intervals remain conservative. D-149 records the compiler defect; the
pairwise OR/XOR cases are covered by `accept_bitwise_range_refinement.em`. No
specification or owner decision changed.

`RNG-4` continuation: finite floating comparison facts now refine `if` arms.
The checker starts an unconstrained numeric local from its finite
representation interval, and strict comparisons widen to closed bounds because
the interval lattice has no floating predecessor/successor operation. The
result is conservative and profile-independent; non-finite bounds and
disjunctions remain fail-closed. D-150 records the compiler defect, with
`accept_float_branch_refinement.em` and
`reject_float_branch_fact_does_not_escape.em` covering acceptance and branch
isolation in all profiles. No specification or owner decision changed.

---

## Current implementation continuation

`DIA-UI-1` H3 continuation: the callable-mode boundary now has `E2228`/`B15`
coverage with an exact UI snapshot and compiling repair. The fixture requires
the expected and supplied callable signatures in the primary help, while the
conformance cases keep direct, generic, and `with_views*_mut` mode mismatches
on the same diagnostic path. This closes the current B15 implementation
slice; it does not claim the remaining diagnostic catalogue or Phase 3 matrix.

`OBJ-RT-1` Phase 3 continuation: MIR verification now propagates every
reachable `BeginAccess`/`EndAccess` interval as a LIFO state through the CFG.
Missing closes, mismatched close order, and incompatible access stacks at joins
are rejected before code generation. The C backend also writes the first
`[EFF-10]` emitted-check records to
`target/<profile>/inspect/<module>.safety.json`. This is still not static
elision in general: the conservative `unique_handle` proof now removes the
unneeded bracket for an unprojected, unescaped local class handle and records
`reason = unique_handle`; receiver-reference, indexed/projection, and escaped
cases remain dynamic. The reader validates and reports the module side table
in human or `--json` form, including `--elided-only`. Function-level filtering
is now available with `--function <name>`; the remaining `[EXC-3]` proof cases
remain pending. No specification or owner decision changed.

`[TYP-22]` continuation: `dyn I` formation now has a canonical `TyKind::Dyn`
identity, resolves interface bounds, rejects non-dyn-compatible interfaces with
the existing `E2050` diagnostic, rejects bare unsized values in parameter and
return positions, and exposes the fixed opaque `{data*, vtable*}` carrier in
generated C. This is only the semantic formation boundary. Interface-object
coercion, vtable construction, method-slot ABI adaptation, and runtime
materialization remain open under the Phase 3 object/dispatch work; do not
mark interface/dyn dispatch complete from this slice.

The next continuation adds a real `InterfaceCall` HIR/MIR boundary for method
calls through `ref dyn I`. It preserves interface slot order, callable
parameter modes, named-argument evaluation order, and the existing opaque
`{data*, vtable*}` carrier. The C backend emits a typed per-interface vtable
shape and performs the indirect slot call, while deliberately refusing to
invent concrete object-to-interface coercion or implementation tables. The
compile-pass matrix covers shared and mutable dynamic receivers. This is an
implementation-only continuation; no adopted specification, owner decision,
diagnostic identity, or language version changed. Interface-object creation,
concrete adapters, `Box[dyn I]` ownership, and the complete Phase 3 matrix
remain open.

D-156 hardens this boundary: `where Self: Sized` defaults remain compatible
with interface formation but cannot be called through `ref dyn`, including
when inherited and when their return type is not `Self`. Negative cases run
in all profiles; ordinary member calls and explicit mutable/named arguments
remain covered. Canonical full-table layout still belongs to the concrete
vtable-materialization work, not to this call-site-shape milestone.

## Build order

1. **LIB-1, LIB-2, LIB-3, LIB-4, LIB-5** — nothing else is pleasant to write
   without them, and each is small.
2. **LIB-6, LIB-7, LIB-9, LIB-10** — the next layer of ordinary work.
3. **LIB-13, LIB-15, LIB-16, LIB-20** — real dependencies, so they want the C
   FFI settled, which means after Phase 5.
4. The rest, driven by what the first real programs reach for.

**Three of the first five are what a package registry needs in order to
exist** — `toml`, `hash`, `net`. The registry is listed as v1.1 in Part XX §2,
and it cannot be built before its own dependencies are. Worth knowing before
anyone schedules it.

---

## Not in this list

Deliberately not planned, with the reasoning in
[LIBRARIES.md](LIBRARIES.md#deliberately-not-planned): an async runtime, a
garbage collector or `Gc[T]` heap, a web framework, an ORM, and a second math
library. Each is refused for a stated reason rather than by omission, so that
"why is there no X" has an answer.
