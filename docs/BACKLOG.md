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
| Total | 31 tasks — 18 must-have, 13 nice-to-have |
| Started | none |
| Blocked on phases | all |

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

## Nice to have

| Id | Task | Gate | Done when |
|---|---|---|---|
| **LIB-19** | `bench` — bootstrap confidence intervals | 4 | implements `[BEN-1]`..`[BEN-7]` properly, so `ember bench --compare` stops approximating |
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
