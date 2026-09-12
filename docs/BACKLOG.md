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
| Total | 40 tracked items — 20 must-have, 12 nice-to-have, 8 compiler-debt items |
| Completed | 2 compiler-debt items (`LT-REG-1`, `RIDX-1`) |
| Started | none of the remaining items |
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
| **CELL-DEF-1** | `Cell[T].take()`, and `update`'s `T: Default` arm | 2 | `[CELL-1]` gives `take(self) -> T where T: Default` and lets `update` take `T: Default` **or** `T: Copy`. There is no `Default` interface in the compiler at all, so only the `Copy` arm of `update` is built and `take` reports the gap by name rather than reading as a missing method (`tests/conformance/CELL-1/reject_take_needs_default.em` pins the message). Done when `Default` exists and both are written as the rule states them. **The rule is not softened**: what is missing is the interface, not the requirement |
| **CELL-SYNC-1** | `[CELL-3]`/`[CELL-8]` `!Sync` on `Cell[T]` and `RefCell[T]` | 4 | Both types are `!Sync`; `Cell[T]` may move between threads when `T: Send`, while RefCell's synchronized equivalents are `Mutex[T]`/`RwLock[T]`. There is no `Send`, no `Sync` and no thread in the compiler, so there is nothing for either marker to mean yet and nothing that could violate it — the restriction is unenforceable rather than unenforced. Done when `[THR-1]` exists and programs sharing either type across threads are refused |
| ~~**RIDX-1**~~ | ~~Rule extraction: teach `rule_index.py` the forms the document already uses~~ | — | **done in `6c77723`.** The six valid definitions left the baseline, no rule prose moved, and `tools/test_rule_index.py` proves both recognition and rejection of reference-shaped false definitions. ODR-002 is closed |
| **SPN-API-1** | Span/MutSpan view-method surface (`split_at`, `chunks`, `iter_mut`, `reborrow`, `as_ptr`, …) | 2 | `[SPN-3]` names `s.reborrow()`, `[BRW-5]` names the sanctioned ways and Part VII §7 works `split_at`, but only `len`, `get`, `get_unchecked`, `is_empty`, `as_span`, `as_mut_span` exist — the rest are refused by name (`E2020` "no method in this phase", pinned by `tests/conformance/SPN-3/reject_split_at_reports_not_in_this_phase.em`). Done when the rule-named surface exists and the sanctioned-ways accept case compiles |

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
