# Ember

Ember is an experimental, statically typed, ahead-of-time compiled language
for systems and application code. Its design combines explicit ownership,
borrowing, deterministic destruction, region-aware views, generics, and
safe-by-default APIs with a portable C backend. The long-term language also
specifies effects, concurrency, C and C++ interoperability, data-oriented
facilities, deterministic execution, and first-class hot reload.

This repository contains the reference compiler, runtime, standard-library
sources, language specifications, conformance suite, diagnostics, and the
engineering records used to keep those pieces consistent.

> Ember is under active development and is not ready for production use.
> Implemented behavior is only a subset of the full specification.

## Project status

The authority levels matter:

| Role | Current artifact |
|---|---|
| Adopted normative specification | [`docs/spec-source/ember-spec.md`](docs/spec-source/ember-spec.md), Ember v0.8.5_Hardened_1 |
| Frozen development target | [`docs/spec-source/Ember_v0.9.6_Hardened_6.md`](docs/spec-source/Ember_v0.9.6_Hardened_6.md) |
| Immediate target predecessor | Ember v0.9.6_Hardened_5, kept immutable |
| Implementation phase | Phase 2, ownership |
| Completed phases | Exactly 1 of 9 |

The v0.9.6_Hardened_6 target is not yet adopted as the normative source. It is
the contract the implementation is working toward. A version is adopted only
after its implementation, conformance, documentation, and regression gates
pass and the owner explicitly installs it.

At the latest verified checkpoint:

- `cargo build --workspace` is warning-free;
- all 189 Rust tests pass;
- the conformance runner passes across 120 rule directories and 399 Ember
  source files;
- all six adopted-specification gates pass;
- 85 recorded compiler defects are closed;
- four known deviations remain open (D1–D4);
- no owner semantic/API decision is currently open; ODR-014 is closed by H6.

`ARN-COLL-1` is complete at implementation commit `825eac5`: the public static
`Hash`/`Hasher` protocol, move-only `DefaultHasher`, built-in and user-defined
`Eq + Hash` ArenaMap keys, read-only resident keys, named iterators, capacity
handling, Arena provenance, ordinary ownership, and single-allocation behavior
are executable. The first `ARCH-096-1` slice is complete at `66d0d43`:
definite-initialization now produces verified shared facts, and the C backend
can consume only a structurally and provenance-verified MIR/type pair. The
`UnsafeCell` is complete at `a02c0a5`: its narrow unsafe raw-pointer boundary,
move-only representation, ordinary destruction, `@static_safe` exclusion, and
diagnostic restraint are executable. The remaining architecture migration
continues incrementally around real feature work.

The concrete, sized, default-allocator `Box[T]` slice is complete at `de641fb`,
with region-complete stored-view checking at `0fdd9b6`. It allocates through
the runtime allocator, is emitted as the specified `T*`, auto-dereferences,
keeps `get()` tied to its owner, rejects invalid moves and non-static stored
views, and destroys `T` before freeing exactly one allocation. Existential
`Box[dyn I]`, custom allocators, allocation-effect accounting, and `Send`
integration remain assigned to their dependent phases.

The H6 Span surface is complete at `d077563`: four public named iterator/chunk
view types use the existing `Iterator`, borrow, region, and NLL machinery;
mutable items and chunks retain disjoint storage ranges; zero-width chunks
panic in every profile; and safe typed raw-pointer extraction does not extend
the source lifetime. The public source declarations live in `std.collections`;
compiler-known lowering is only the current bootstrap implementation.

For-loop borrowing and the dedicated mutate-while-iterating diagnostic are
complete at `e0ba765`: direct Array iteration borrows for the whole loop,
yields shared references, leaves the Array usable afterward, and reports
E3020/B2 for conflicting mutation rather than moving the collection or falling
back to a generic overlapping-loan diagnostic.

The direct-call multi-region slice is complete at `90059c8`, building on
`c913fbd`'s field-sensitive region vectors. Direct function summaries now
retain exact result-field provenance and parameter-field access, transitive
wrappers reach a fixpoint, known split operations publish exact result
relations, and opaque multi-region results fail with `E3065/B14` rather than
being widened to the obsolete one-region intersection. Field replacement uses
the destination projection rather than the containing local, and generated C
contains no region metadata. Summary serialization/verification and
invalidation, non-direct dispatch, and the complete escape/storage matrix
remain incomplete; unknown calls stay conservative.

See [`docs/HANDOFF.md`](docs/HANDOFF.md) for the complete verified state and
[`docs/COLD-START.md`](docs/COLD-START.md) for the shortest safe route into the
compiler.

## What works today

The reference compiler currently includes substantial support for:

- lexical analysis, parsing, formatting, name resolution, type checking, HIR,
  MIR, safety analysis, C generation, native compilation, and execution;
- scalar values, structs, enums, tuples, tuple/struct destructuring assignment,
  arrays, ranges, pattern matching, and control flow;
- generic functions, generic types, generic methods, interface bounds,
  associated types, and monomorphization;
- ownership, moves, non-lexical borrows, mutable references, region-carrying
  views with field-sensitive direct aggregate provenance,
  checked shared/mutable Span splitting, explicit mutable-span
  reborrowing, named shared/mutable Span iteration and chunking, safe raw-
  pointer extraction with unsafe-only use, deterministic destruction, drop
  flags, and partial moves;
- capturing closures through statically monomorphized callable bounds;
- compiler-known `Array`, `String`, `Span`, `MutSpan`, `Box`, `Cell`, `RefCell`, and
  Arena primitives, plus the public `std.mem.UnsafeCell` boundary;
- public `std.mem` ownership/layout operations including move-preserving
  `take`, `replace`, `swap`, destructor-suppressing `forget`, `size_of`, and
  canonical-layout `align_of`;
- Arena allocation, scopes, reset/rewind, `MaybeUninit`, conservative
  `Zeroable`, `alloc_array`, `alloc_uninit`, and fixed-capacity Arena
  collections;
- the statically generic `Hash`/`Hasher` protocol, a deterministic move-only
  `DefaultHasher`, and custom `Eq + Hash` keys in `ArenaMap`;
- structured diagnostics and conformance assertions over generated C,
  including operation ordering and exact occurrence counts.

Important incomplete areas include the remainder of Phase 2 diagnostics and
rule coverage, existential/custom-allocator Box forms, general automatic/derived `Hash` generation and
ordinary `Map`/`Set`, the canonical semantic-fact migration, objects and
managed ownership, effects and comptime, complete FFI, concurrency/data-oriented
facilities, the interpreter, hot reload, and the final performance and
ecosystem hardening phases.

## Examples

### Hello, Ember

```ember
fn main():
    println("Hello from Ember")
```

### Generics and interface bounds

```ember
interface Shape:
    fn area(self) -> f32

struct Square implements Shape:
    side: f32

    fn area(self) -> f32:
        return self.side * self.side

fn total_area[T: Shape](a: T, b: T) -> f32:
    return a.area() + b.area()

fn main():
    println(total_area(Square(2.0), Square(3.0)))
```

### Explicit borrowing

```ember
fn increment(mut value: i32):
    value = value + 1

fn main():
    n: i32 = 10
    increment(n)
    println(n)  # 11

    shared: ref i32 = ref n
    println(shared)
```

Parameter modes carry the ordinary borrow at call sites, so `increment(n)`
does not require `ref mut` syntax. Explicit `ref` and `ref mut` are used when a
reference value itself is constructed.

### Fixed-capacity Arena storage

```ember
from std.collections import ArenaArray

fn main():
    arena = Arena.with_capacity(4096)
    values: ArenaArray[i32] = ArenaArray[i32].with_capacity(arena, 3)

    _first = values.push(10)
    _second = values.push(20)

    total: i32 = 0
    for value in values.iter():
        total = total + value
    println(total)  # 30
```

`ArenaArray` is a region-carrying fixed-capacity view. Construction reserves
its backing storage in one Arena allocation; it does not own the Arena or hide
an owning Arena pointer.

More executable examples live in [`examples/`](examples/),
[`tests/run-pass/`](tests/run-pass/), and
[`tests/conformance/`](tests/conformance/).

## Build and run

The compiler is a Rust workspace using the stable toolchain. Native Ember
programs currently lower through the C backend, so a supported C compiler is
also required. CI exercises MSVC and clang-cl on Windows, and Clang and GCC on
Linux.

```text
cargo build --workspace
cargo test --workspace
```

Run a source file through the compiler without installing it:

```text
cargo run -p ember_driver --bin ember -- run path/to/program.em
```

Other useful commands are:

```text
cargo run -p ember_driver --bin ember -- check path/to/program.em
cargo run -p ember_driver --bin ember -- check path/to/program.em --syntax-only
cargo run -p ember_driver --bin ember -- build path/to/program.em --emit c
cargo run -p ember_driver --bin ember -- explain E3042
```

Run `cargo run -p ember_driver --bin ember -- --help` for the current command
line surface.

## Repository map

```text
compiler/   Rust crates for the compiler pipeline
runtime/    C runtime used by generated programs
std/        Ember standard-library source modules
tests/      run-pass, run-fail, compile-fail, milestone, and conformance tests
tools/      specification, registry, branding, and documentation gates
docs/       specifications, diagnostics, decisions, defects, backlog, and handoff
examples/   standalone Ember programs
```

The most useful project records are:

- [`docs/COLD-START.md`](docs/COLD-START.md) — current starting point and traps;
- [`docs/HANDOFF.md`](docs/HANDOFF.md) — detailed implementation snapshot;
- [`docs/DECISIONS.md`](docs/DECISIONS.md) — accepted architecture and owner rulings;
- [`docs/DEFECTS.md`](docs/DEFECTS.md) — compiler defects and verification evidence;
- [`docs/BACKLOG.md`](docs/BACKLOG.md) — implementation gaps and build order;
- [`docs/OWNER-QUEUE.md`](docs/OWNER-QUEUE.md) — questions requiring owner authority;
- [`docs/MIGRATION-0.9.6.md`](docs/MIGRATION-0.9.6.md) — H6 adoption and implementation map.

## Development protocol

The specification is the contract. Compiler behavior and passing tests do not
override it. Before changing semantics or implementation:

1. reproduce the behavior with a minimal adversarial program;
2. identify the exact normative rule;
3. classify the finding as a language decision, specification ambiguity,
   compiler defect, test defect, or implementation limitation;
4. verify that the test actually exercises every relevant clause;
5. fix the compiler when the semantics are clear;
6. stop and request an owner ruling when semantics are genuinely ambiguous;
7. record the result in the appropriate ledgers and add mutation-sensitive
   conformance evidence.

Frozen specification cuts are immutable. A newly discovered specification gap
after H6 becomes H7; H6 is never edited in place. Generated specification
parts under `docs/spec/` are never hand-edited.

A passing suite is necessary, not sufficient. Ownership and lifetime work is
checked with minimal counterexamples, adversarial cases, generated-C
inspection where source observation is unsafe or impossible, and all relevant
repository gates.

## Roadmap

The nine-phase plan proceeds from the completed core-language phase through:

1. ownership completion (current Phase 2);
2. objects and managed ownership;
3. effects, comptime, reflection, and derives;
4. C interoperability;
5. concurrency and data-oriented programming;
6. C++ interoperability and the supported interpreter;
7. iteration, determinism, hot reload, and instantiation-cost controls;
8. full conformance, performance validation, external-package validation, and
   the 1.0 hardening pass.

The immediate order is narrower: carry the direct `[LT-22]`/`[LT-35]`
field/provenance summaries into verified MIR/interface metadata and add
`[LT-40]` invalidation; then complete the remaining
multi-region/Phase 2 matrix, and continue the `[DIA-7..10]`/`[DIA-13]`
diagnostic catalogue as its currently unreachable mechanisms become real.

## Contributing

Start with `README.md`, then read `docs/COLD-START.md`, `docs/HANDOFF.md`,
`docs/DECISIONS.md`, `docs/DEFECTS.md`, and `docs/BACKLOG.md` before changing
compiler behavior. Preserve unrelated working-tree changes, use normative rule
IDs in conformance tests, and include the exact failure and verification method
in every defect record.
