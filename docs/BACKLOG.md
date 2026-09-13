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
| Total | 44 tracked items — 20 must-have, 12 nice-to-have, 12 compiler-debt items |
| Completed | 7 compiler-debt items (`LT-REG-1`, `RIDX-1`, `CELL-DEF-1`, `ARN-INIT-1`, `ARN-COLL-1`, `GEN-METHOD-1`, `VER-096-1`) |
| Started | `ARCH-096-1` |
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
| **RT-GEN-1** | Generate `ember_rt.h` and `ember_rt.c` from `EMBER_SYMBOL_PREFIX` | 0 | `[RT-5]` calls them "generation output"; ours are hand-written, so `tools/check_branding.py` exempts them. Done when the generator exists and the exemption is deleted |
| ~~**LT-REG-1**~~ | ~~Real region variables with a constraint graph~~ | — | **done 2026-09-09.** `compiler/ember_analysis/src/regions.rs`; `[LT-1]`'s elision is in at the call site and in the body (`E3062`). `[LT-2]`'s view structs and `[LT-7]`'s callback regions build on it |
| **LNT-CFG-1** | `[MAN-3]`'s `[lints]` configuration | 2 | `[LT-1b]`'s `L3014` is an opt-in lint and there is nowhere to opt in |
| **TST-6-1** | Appendix A's fixture as `compile-pass` | 4 | it is held to `--syntax-only` today because the appendix names `Entity`, `Formatter`, `SoA`, `Arena` and `Mutex`, which `std` does not yet have |
| ~~**CELL-DEF-1**~~ | ~~`Cell[T].take()`, and `update`'s `T: Default` arm~~ | — | **done 2026-09-13.** Receiver-less `Default.default()` resolves through ordinary interface identity; `take` constructs a replacement before moving out the old value, and non-`Copy` `update` parks the old value behind a default placeholder before its borrowed callback runs, then stores the result before dropping the placeholder. Positive move-only/destructor cases and negative missing-capability cases are under `tests/conformance/CELL-1/` |
| **CELL-SYNC-1** | `[CELL-3]`/`[CELL-8]` `!Sync` on `Cell[T]` and `RefCell[T]` | 4 | Both types are `!Sync`; `Cell[T]` may move between threads when `T: Send`, while RefCell's synchronized equivalents are `Mutex[T]`/`RwLock[T]`. There is no `Send`, no `Sync` and no thread in the compiler, so there is nothing for either marker to mean yet and nothing that could violate it — the restriction is unenforceable rather than unenforced. Done when `[THR-1]` exists and programs sharing either type across threads are refused |
| ~~**RIDX-1**~~ | ~~Rule extraction: teach `rule_index.py` the forms the document already uses~~ | — | **done in `6c77723`.** The six valid definitions left the baseline, no rule prose moved, and `tools/test_rule_index.py` proves both recognition and rejection of reference-shaped false definitions. ODR-002 is closed |
| **SPN-API-1** | Span/MutSpan view-method surface (`split_at`, `chunks`, `iter_mut`, `reborrow`, `as_ptr`, …) | 2 | `[SPN-3]` names `s.reborrow()`, `[BRW-5]` names the sanctioned ways and Part VII §7 works `split_at`, but only `len`, `get`, `get_unchecked`, `is_empty`, `as_span`, `as_mut_span` exist — the rest are refused by name (`E2020` "no method in this phase", pinned by `tests/conformance/SPN-3/reject_split_at_reports_not_in_this_phase.em`). Done when the rule-named surface exists and the sanctioned-ways accept case compiles |
| ~~**ARN-INIT-1**~~ | ~~Initialization types/interfaces required by Arena bulk allocation~~ | — | **done 2026-09-13.** Compiler-known `MaybeUninit[T]`, scalar/span write and consuming unsafe conversion APIs, Arena `alloc_uninit`, bounds/provenance enforcement, the conservative built-in `Zeroable` predicate, and both `alloc_array` branches are executable. `!needs_drop` is checked first; `Zeroable` precedes `Default`; the Default arm emits per-element constructor calls; E2040 handles neither; and the complete `[TST-23]` matrix covers initialization, conversion, layout, destruction, capability precedence, rejection paths, and `[ARN-10]` success plus all-profile abort/no-continuation behavior. General/manual or derived `Zeroable` support remains assigned to its later phase; completing this core predicate does not permit validity to be guessed from field shape or claim that later derive surface |
| ~~**ARN-COLL-1**~~ | ~~`[ARN-5]` `ArenaArray` and `ArenaMap`~~ | — | **done 2026-09-13 in `825eac5`.** Both are genuine fixed-capacity, single-allocation arena-backed `@view` collections with `[TYP-15]` provenance, not owning wrappers with hidden Arena pointers. Array and Map implement the H2 minimum operations, `CapacityError.Full`, `!needs_drop` boundaries, named iterators, provenance/borrow enforcement, zero/full/reuse behavior, and exact allocation-count paths. H3/H4's public `Hash`/`Hasher` protocol and move-only `DefaultHasher` use the ordinary static generic/interface pipeline; `Hash` is prelude while `Hasher`, `DefaultHasher`, and the Arena collection names are not. ArenaMap accepts compiler-known and user-defined `K: Eq + Hash`, calls the selected `Eq.eq` for custom keys, keeps resident keys read-only, and preserves move-only keys/values through replacement, removal, and compaction. `[TST-24]` and `[HASH-1]`–`[HASH-4]` carry positive and adversarial evidence. The current linear Map is permitted not to call the hasher; its exact mixing algorithm remains implementation-defined. General derive-generated `Hash` and ordinary `Map`/`Set` remain their later phase work and are not claimed here |
| **ARN-LATE-1** | Arena obligations whose mechanisms belong to later phases | 4/6 | allocation effects and `@noalloc` behavior are enforced by the effect system; scoped guards participate in the real `@must_drop` mechanism; and `[JOB-5]` `ThreadArena` is implemented only with the concurrency/job model. The current core does not claim these later-phase obligations |
| ~~**GEN-METHOD-1**~~ | ~~Explicit type arguments on method calls~~ | — | **done 2026-09-13.** The AST/parser/printer/formatter preserve `recv.method[T](...)`; compiler-known and source-declared methods share explicit/inferred type selection, bounds, diagnostics, concrete instantiation, and deterministic symbols. This covers concrete and generic owners, receiver-less associated functions, interface-bound dispatch, generic/default interface methods, callable parameters, and view-provenance signatures. The mechanism reuses ordinary generic-call rules; no Arena-specific parser or monomorphisation path was added. Adversarial coverage is under `tests/conformance/TYP-17/` and `TYP-18/` |
| **ARCH-096-1** | Implement 0.9.6's canonical semantic-fact architecture | 2/3 | **Started 2026-09-13; first verified boundary landed in `66d0d43`.** `TypeIdentity` and `LayoutDescriptor` name the existing canonical representations; `BorrowCapability`, `AccessContract`, storage/provenance, permission, representation, ownership, checking, unsafe-authority, synchronization, validity, escape, and shared `InitializationState` facts exist, and Arena loans carry distinct provenance/storage facts. Definite initialization now retains one `InitializationFacts` fixpoint, verifies its seeds/transfers/predecessor joins before consumption, and drives diagnostics from that verified record. The C backend accepts only a `VerifiedMir` pairing of the exact bodies and `TypeTable`, created after final body pruning by unconditional structural and view-provenance verification. `OwnershipGraph`, `EffectSet`, complete borrow/ownership producer-consumer migration, callable summaries, invalidation, runtime-erasure proof, and the equivalence matrix remain. Preserve all accepted/rejected 0.9.5 behavior and the three distinct write orderings; do not create placeholder facts with no real producer and consumer |
| ~~**VER-096-1**~~ | ~~Accept the `#! language "0.9.6"` selector~~ | — | **done 2026-09-13.** The parser accepts the exact `0.9`, `0.9.5`, and `0.9.6` contracts, retains every older supported selector, and rejects an unknown `0.9.7` with E0006. This is selector recognition only; H1 remains a frozen development target until the adoption gates and explicit owner action complete |

---

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
