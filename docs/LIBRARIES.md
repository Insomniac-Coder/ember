# Libraries

> Tracked as numbered tasks in [BACKLOG.md](BACKLOG.md) — `LIB-1`..`LIB-31`.
> This file carries the reasoning; that one carries the gates and the
> acceptance criteria.

What ships with the language, and what should follow once all nine phases are
done. Nothing here is a commitment to build now; Part XX's phase order stands,
and this list exists so that the shape of the ecosystem is decided before
anyone starts filling it in.

Two things it is worth being clear about up front:

- **YAML is already in v1.** Part XV puts `std.ser` in the standard library and
  Part XIV §4 names `std.ser.yaml` specifically, to read RageV's `yaml-cpp`
  scene files. It is not a nice-to-have; it lands in Phase 4 with the
  `@derive(Serialize, Deserialize)` generators.
- **The line between `std` and a package matters more here than in most
  languages.** `[STD-1]` holds `std.core`, `std.mem`, `std.math`, `std.simd`,
  `std.span` and `std.arena` to being `@noalloc`-clean, and `[PHIL-2]` says no
  implicit allocation. A library that allocates cannot go in those modules,
  which is why several obvious candidates below are packages rather than `std`
  additions.

---

## Already specified for v1 (Part XV)

These are not proposals. They are the standard library surface the phases
build, listed so the additions below can be read against them.

| Module | What it carries |
|---|---|
| `std.core` | `Option`, `Result`, `Cell`, `RefCell`, the marker and operator interfaces, iterator adaptors, ranges, `print`/`println`, `assert*`, `panic` |
| `std.mem` | `MaybeUninit`, `transmute`, `Layout`, `Allocator`, pointer ops, `assert_disjoint` |
| `std.cell` | `Cell`, `RefCell`, `Ref`, `RefMut` |
| `std.collections` | `Array`, `SmallArray`, `Deque`, `Map`, `Set`, `BitSet`, `Pool`, `Handle`, `SoA` |
| `std.string` | `String`, `str` methods, `StringBuilder`, `CString` |
| `std.fmt` | `Formatter`, `Display`, `Debug`, `format`, `format_to` |
| `std.math` | `Vec2/3/4`, `Mat2/3/4`, `Quat`, `Transform`, `AABB`, `Ray`, `Frustum`, scalar functions — GLM-compatible layouts |
| `std.simd` | vector types, masks, `std.cpu` intrinsics |
| `std.arena` | `Arena`, `FixedArena`, `ScopedArena`, `ThreadArena` |
| `std.io`, `std.fs` | `Read`/`Write`, standard streams, `File`, `Path` |
| `std.time` | `Instant`, `Duration`, `SystemTime` |
| `std.thread`, `std.sync`, `std.jobs`, `std.atomic` | Part XI |
| `std.process` | `exit`, `args`, `env`, `Command` |
| `std.ecs` | Part XII §3 |
| **`std.ser`** | **binary and YAML (de)serialisation** |
| `std.testing` | `@test`, `assert_approx_eq`, `expect_panic`, benchmarks |
| `std.ffi` | `CString`, `c_int` aliases, `Callback`, `Retained`, `ForeignBox` |
| `std.gpu` | Part XVII, host side |
| `std.debug` | `backtrace`, `leak_report`, `alloc_stats`, profiler zones |

---

## Must have, after the phases

The ones a systems language is not credible without. Each is a package unless
marked otherwise.

### Data formats

| Library | Why |
|---|---|
| **`yaml`** — a full YAML 1.2 reader/writer | `std.ser.yaml` covers *mapping Ember types to and from YAML*, which is what scene files need. A general document API — anchors, aliases, multi-document streams, comment preservation on round trip, merge keys — is a different job and does not belong in `std`. Config files and CI descriptions need the general one. |
| **`json`** | The format everything else speaks. Wants both a `Serialize` path and a streaming reader for files that do not fit in memory. |
| **`toml`** | `ember.toml` is TOML, so the compiler already contains a reader. Publishing a real one stops that being duplicated by every tool. |
| **`csv`** | Dull and constantly needed. Wants to be `@noalloc` over a caller's buffer. |
| **`binary`** — endian-aware readers and writers over `Span[u8]` | Half of file-format work. Belongs beside `std.mem`, not inside it, because it allocates on the growing path. |

### Text

| Library | Why |
|---|---|
| **`regex`** | A DFA/backtracking hybrid with a compile-time-constructible form, so a pattern known at `comptime` costs no allocation at run time. |
| **`unicode`** | `str` is UTF-8 by construction, but grapheme clusters, normalisation beyond NFC, case folding and width are not in `std.string` and should not be — the tables are large and most programs never touch them. |
| **`glob`**, **`path-match`** | File tooling. |

### Systems and I/O

| Library | Why |
|---|---|
| **`log`** | A façade with levels and structured fields, and a `@noalloc` path for the hot case. Every other library depends on the façade rather than on an implementation. |
| **`cli`** | Argument parsing driven by `@derive`, using the same reflection Part XIV gives serialisation. |
| **`net`** | TCP, UDP, DNS. Non-blocking over the job system rather than `async`, which is v2. |
| **`http`** | Client first. A server is a bigger commitment than it looks and should wait. |
| **`tls`** | Bindings to a vetted C implementation, not a fresh one. Cryptography is the one area where writing it yourself is the wrong answer. |
| **`compress`** | zlib, zstd and lz4 through the C FFI, with an Ember-side streaming interface. |
| **`hash`** | BLAKE3, xxHash, FNV. Non-cryptographic hashing is already in `std.core` for `Map`; this is the rest. |
| **`uuid`** | Small, universally needed. |
| **`random`** | Splitmix and PCG, seedable and reproducible. Determinism matters for a game engine: replays and tests both depend on it. |

### Development

| Library | Why |
|---|---|
| **`bench`** | `std.testing` has a benchmark harness; a statistics library that does `[BEN-1]`'s bootstrap confidence intervals properly belongs outside it. |
| **`proptest`** | Property-based testing with shrinking. Given a borrow checker and a type system this strict, generative testing finds what examples miss. |
| **`tracing`** | Span-based instrumentation feeding `std.debug`'s profiler zones. |

---

## Nice to have

Worth building; nothing breaks without them.

| Library | Note |
|---|---|
| **`sqlite`** | Through the C FFI. The one embedded database everything expects. |
| **`image`** | PNG, JPEG, KTX2, DDS. Engine-adjacent, so it wants `std.gpu`'s formats. |
| **`audio`** | miniaudio is already a Phase 5 FFI fixture; an idiomatic wrapper follows. |
| **`gltf`** | cgltf is likewise a fixture. |
| **`font`** | Shaping and rasterisation, or bindings to HarfBuzz and FreeType. |
| **`geometry`** | Meshes, BVH, convex hulls, spatial hashes. `std.math` deliberately stops at the primitives. |
| **`noise`** | Perlin, simplex, worley. Small, and every procedural tool wants it. |
| **`terminal`** | ANSI, raw mode, size queries. Needed before anyone writes a TUI. |
| **`datetime`** | Calendars and time zones, which `std.time` deliberately does not carry — `Instant` and `Duration` are physics, dates are politics. |
| **`bigint`**, **`decimal`** | Arbitrary precision and fixed-point money. |
| **`graph`** | Traversal and topological sort, over `SoA` storage. |
| **`state-machine`** | `@derive`-driven transitions. Gameplay code writes these by hand constantly. |

---

## Deliberately not planned

| Not doing | Why |
|---|---|
| An async runtime | `async`/`await` are reserved for v2 and Part XI §7 fixes the design as "structured async over the job system". A library that invents its own now would have to be thrown away. |
| A garbage collector, or a `Gc[T]` heap | Part 0 row 1 rejected tracing GC and XXII.2 makes "no cycle collector" a non-goal. `Weak`, the leak report and `L3001` are the whole answer. |
| A web framework | Too early, and it would fix opinions about `http` before that library has users. |
| An ORM | Same, for `sqlite`. |
| A second math library | `std.math` is layout-compatible with GLM by requirement (`[STD-*]`, Part XXI). A competing one splits the ecosystem at its foundation. |

---

## The order to build them in

1. **`json`, `toml`, `binary`, `log`, `cli`** — nothing else is pleasant to
   write without them, and each is small.
2. **`yaml` (the full document API), `regex`, `hash`, `random`** — the next
   layer of ordinary work.
3. **`net`, `compress`, `sqlite`** — real dependencies, so they want the C FFI
   settled, which means after Phase 5.
4. Everything else, driven by what the first real programs actually reach for.

Three of the first five are what a package registry needs in order to exist
(`toml`, `hash`, `net`), which is worth noticing: the registry is listed as
v1.1 in Part XX §2, and it cannot be built before its own dependencies are.
