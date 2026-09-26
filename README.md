# Ember

Ember is an experimental, statically typed, ahead-of-time compiled language for systems and
application code. It aims for memory safety without a garbage collector, C-like speed, and
Python-like ergonomics: explicit ownership and borrowing, deterministic destruction, views that
carry their regions, reference-counted classes, generics with interfaces, and a portable C
backend. The full language also specifies effects and contracts, compile-time evaluation,
concurrency, C and C++ interoperability, data-oriented facilities, deterministic execution and hot
reload.

This repository holds the reference compiler (Rust), its C runtime, the standard library (written
in Ember), the language specification, the conformance suite, the diagnostics and their error
pages, and the records that keep them consistent.

> Ember is under active development and is not ready for production use. The compiler implements
> a large subset of the specification; the rest is rejected with `E0900` ("not implemented yet"),
> never silently accepted.

## Status

| | |
|---|---|
| Language version being implemented | **0.9.9**, specification `Ember_v0.9.9_Hardened_28` |
| Pinned development target | [`docs/spec-source/development-target.json`](docs/spec-source/development-target.json) → [`docs/spec-source/Ember_v0.9.9_Hardened_28.md`](docs/spec-source/Ember_v0.9.9_Hardened_28.md) |
| Specification sources | [`tasks/spec-0.9.9/parts/`](tasks/spec-0.9.9/parts/), one file per Part; each `Hardened_N` is their concatenation and is never edited afterwards |
| Last adopted normative specification | [`docs/spec-source/ember-spec.md`](docs/spec-source/ember-spec.md), 0.8.5_Hardened_1 (0.9.9 is adopted when its gates pass and the owner installs it) |
| Tests | 272 Rust tests; 1,543 Ember test programs, 250 conformance rule directories |
| Defects | 323 fixed, 5 open ([`docs/DEFECTS.md`](docs/DEFECTS.md)) |
| Language decisions | 70 ODRs in [`docs/OWNER-QUEUE.md`](docs/OWNER-QUEUE.md), each carried into a `Hardened_N` |
| CI | Linux (Clang, GCC) and Windows (MSVC, clang-cl); every push to `main` |

Phase estimates against 0.9.9 (2026-09-26, end of day; weighted by the size of each phase's rules;
method in [`docs/HANDOFF.md`](docs/HANDOFF.md)):

| Phase | Scope | Done |
|---|---|---:|
| 1 | Core language (Parts II–VI) | 97% |
| 2 | Ownership, borrowing, regions | 88% |
| 3 | Classes, reference counting, exclusivity | 50% |
| 4 | Effects, compile time, reflection, derives | 13% |
| 5 | C interoperability | 8% |
| 6 | Concurrency and data-oriented design | 2% |
| 7 | C++ interoperability and the interpreter | 0% |
| 7a | Iteration, determinism, cost control | 6% |
| 8 | Hardening and 1.0 | 14% |
| | Overall | 57% |

The latest language work is the owner's simplification pass
([`docs/proposals/Ember_Simplification_Pass_Revised.md`](docs/proposals/Ember_Simplification_Pass_Revised.md)),
adopted as ODR-049 to ODR-069 (ODR-070 is a nesting limit): associated-type defaults, `alloc_array` by `Default` and a separate
`alloc_zeroed`, float operators that stay IEEE in generic code, no early destruction, a
memory-safety fix for borrows through handles stored in objects, map lookups that need no key
conversion (`AsKey`/`ToKey`), `get_pair_mut`, when two const generic arguments are equal, and one
storage rule for views in every container (a `Map[str, int]` of string literals), checked at every
store.

## What works today

- The whole pipeline: lexing, parsing, name resolution, type checking, HIR, MIR, borrow and region
  checking, C generation, native compilation and running.
- Scalars, including `i128`/`u128` and `f16`; structs, enums, tuples, fixed arrays, range types,
  pattern matching, destructuring, and Python-style control flow.
- Generic functions, types and methods, interface bounds, associated types (with defaults),
  blanket implementations, operator interfaces (`Add` … `Shr`, `Index`, `IndexMut`, `IndexSet`),
  and monomorphisation; a generic body is checked once, as generic code.
- Ownership and borrowing: moves, `owned`/`mut`/borrowed parameter modes, non-lexical borrows,
  views (`ref`, `Span`, `MutSpan`, `str`) with field-sensitive regions, deterministic destruction
  and drop flags.
- Classes: reference-counted handles, inheritance, virtual dispatch, weak handles, runtime
  exclusivity checks, and module privacy for methods and fields.
- The standard library in Ember: `Array`, `String`, `Map` and `Set` (insertion-ordered),
  `Option`/`Result`, `Cell`/`RefCell`, `Box`, arenas (`alloc_array`, `alloc_zeroed`,
  `alloc_uninit`, fixed-capacity collections), `NonZero[T]`, iterators and `for x in owned e`,
  `std.math` (vectors, matrices, `KahanSum`, `Float` and `Number` generics) and `std.math.det`
  (bit-identical transcendental functions on every target).
- Diagnostics with stable codes, help and notes; `ember explain CODE` and error pages in
  [`docs/errors/`](docs/errors/).

Not built yet (rejected with `E0900`): most of effects and compile-time evaluation, several
derives (`Default`, `Ord`, `Serialize`), the FFI beyond the basics, threads and jobs, SIMD and
ECS facilities, C++ interop, the interpreter, and hot reload. The open defects are listed in
[`docs/DEFECTS.md`](docs/DEFECTS.md).

## Examples

Every example below compiles and runs with the current compiler.

```ember
fn main():
    println("Hello from Ember")
```

Interfaces and generics:

```ember
interface Shape:
    fn area(self) -> f64

struct Square implements Shape:
    side: f64

    fn area(self) -> f64:
        return self.side * self.side

fn total[T: Shape](shapes: Span[T]) -> f64:
    sum = 0.0
    for s in shapes:
        sum += s.area()
    return sum

fn main():
    squares = [Square(2.0), Square(3.0)]
    println(total(squares.as_span()))    # 13.0
```

Parameter modes and views:

```ember
fn increment(mut value: int):
    value += 1

fn longest(a: str, b: str) -> str:
    return a if a.len() >= b.len() else b

fn main():
    n = 10
    increment(n)
    println(n)                                   # 11
    words = ["ember", "compiler"]
    println(longest(words[0], words[1]))         # compiler
```

A `mut` parameter borrows the caller's variable for the call; no `&mut` is written at the call
site. `longest` returns a view of one of its arguments, and the compiler keeps both borrowed for as
long as the result lives.

Classes are reference-counted handles:

```ember
class Counter:
    count: int

    fn bump(mut self):
        self.count += 1

fn main():
    a = Counter(0)
    b = a
    b.bump()
    println(a.count, a is b)    # 1 true
```

Maps keep insertion order:

```ember
fn main():
    ages: Map[String, int] = {}
    ages["ann"] = 31
    ages["bob"] = 27
    for name, age in ages.items():
        println(name, age)
    println(ages.get("ann"), "cy" in ages)    # Some(31) false
```

More programs: [`examples/`](examples/), [`tests/run-pass/`](tests/run-pass/) and
[`tests/conformance/`](tests/conformance/) (one directory per specification rule).

## Build and run

You need a stable Rust toolchain and a C compiler (Clang, GCC or MSVC).

```text
cargo build --workspace
cargo run -p ember_driver --bin ember -- run path/to/program.em
```

The `ember` command:

```text
ember run   <file.em>        compile and run
ember build <file.em>        compile to an executable (--profile debug|release|shipping)
ember check <file.em>        type-check without generating code
ember build <file.em> --emit c|mir|hir|ast|tokens
ember explain E3060          describe a diagnostic code
ember inspect --safety <path>
```

`--cc clang|gcc|msvc` (or the environment variable `EMBER_CC`) overrides C compiler detection.

Tests:

```text
EMBER_CC=clang cargo test --workspace          # everything, including the conformance suite
python3 tasks/impl-0.9.9/annotations.py tests/conformance/MOD-2/   # one directory, quickly
```

The second needs `EMBER=target/debug/ember`. Specification and repository gates live in
[`tools/`](tools/) (`spec_check.py`, `rule_index.py`, `error_pages.py`, `hardening_check.py`,
`check_det.py`, …); CI runs them all.

## Repository map

```text
compiler/            the Rust crates: parser, typeck, hir, mir, analysis, codegen_c, driver, …
runtime/             the C runtime, generated from runtime/ember_rt/templates/
std/src/             the standard library, in Ember
tests/               conformance (by rule), run-pass, run-fail, compile-fail, compile-pass, ui
tools/               specification, registry, runtime and documentation gates
tasks/spec-0.9.9/    the 0.9.9 specification's sources and their build tools
docs/spec-source/    each Hardened_N, the adopted spec, and the pinned development target
docs/errors/         one page per diagnostic code
docs/proposals/      the owner's design proposals
examples/            standalone programs
```

## Project records

- [`docs/HANDOFF.md`](docs/HANDOFF.md) — the current state, next task and recipes (§0.355,
  "Start here").
- [`docs/OWNER-QUEUE.md`](docs/OWNER-QUEUE.md) — language decisions (ODRs) and their rulings.
- [`docs/DEFECTS.md`](docs/DEFECTS.md) — every compiler defect, with its fix and test.
- [`docs/DECISIONS.md`](docs/DECISIONS.md) — implementation decisions (ADRs).
- [`docs/MIGRATION-0.9.9.md`](docs/MIGRATION-0.9.9.md) — what changed for programs, dated.

## Development protocol

The specification is the contract; compiler behaviour and passing tests never override it.

1. Reproduce the behaviour with a minimal program.
2. Find the rule that governs it. If the specification is ambiguous, record an ODR, rule it, and
   cut the next `Hardened_N`; never edit a frozen `Hardened_N` or the specification to fit the
   compiler.
3. Fix the compiler, with a conformance test that fails without the fix (check it by breaking the
   fix), and record the defect.
4. Before pushing: the full test suite and every gate; then watch CI. `main` is never left red.

## Contributing

Read the "Start here" part of [`docs/HANDOFF.md`](docs/HANDOFF.md) first. Name the rule in every conformance test (`#$ rules:`),
and give every defect its failing program and its fix.
