<p align="center">
  <img src="docs/brand/ember-banner.svg" alt="Ember" width="100%">
</p>

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
| Language version being implemented | **0.9.9**, specification `Ember_v0.9.9_Hardened_44` |
| Pinned development target | [`docs/spec-source/development-target.json`](docs/spec-source/development-target.json) → [`docs/spec-source/Ember_v0.9.9_Hardened_44.md`](docs/spec-source/Ember_v0.9.9_Hardened_44.md) |
| Specification sources | [`tasks/spec-0.9.9/parts/`](tasks/spec-0.9.9/parts/), one file per Part; each `Hardened_N` is their concatenation and is never edited afterwards |
| Last adopted normative specification | [`docs/spec-source/ember-spec.md`](docs/spec-source/ember-spec.md), 0.8.5_Hardened_1 (0.9.9 is adopted when its gates pass and the owner installs it) |
| Tests | Rust unit and integration suites; Ember run and conformance suites in [`tests/`](tests/) |
| Defects | Current fixed and open findings in [`docs/DEFECTS.md`](docs/DEFECTS.md) |
| Language decisions | Current owner decisions (ODRs) in [`docs/OWNER-QUEUE.md`](docs/OWNER-QUEUE.md) |
| CI | Linux (Clang, GCC) and Windows (MSVC, clang-cl); every push to `main` |

Phase estimates against 0.9.9 (2026-09-29; weighted by the size of each phase's rules;
method in [`docs/HANDOFF.md`](docs/HANDOFF.md)):

| Phase | Scope | Done |
|---|---|---:|
| 1 | Core language (Parts II–VI) | 98% |
| 2 | Ownership, borrowing, regions | 89% |
| 3 | Classes, reference counting, exclusivity | 60% |
| 4 | Effects, compile time, reflection, derives | 13% |
| 5 | C interoperability | 14% |
| 6 | Concurrency and data-oriented design | 4% |
| 7 | C++ interoperability and the interpreter | 0% |
| 7a | Iteration, determinism, cost control | 6% |
| 8 | Hardening and 1.0 | 17% |
| | Overall | 59% |

The latest language work includes the owner's simplification pass
([`docs/proposals/Ember_Simplification_Pass_Revised.md`](docs/proposals/Ember_Simplification_Pass_Revised.md));
its rulings and current status are tracked in [`docs/OWNER-QUEUE.md`](docs/OWNER-QUEUE.md). It
covers associated-type defaults, `alloc_array` by `Default` and a separate `alloc_zeroed`, float
operators that stay IEEE in generic code, no early destruction, a memory-safety fix for borrows
through handles stored in objects, map lookups that need no key conversion (`AsKey`/`ToKey`),
`get_pair_mut`, when two const generic arguments are equal, and one storage rule for views in every
container (a `Map[str, int]` of string literals), checked at every store.

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

## Benchmarks

Each program below was written twice, in Ember and by hand in C (C++ for the programs that use
objects, strings, maps or sorting), built by the same C compiler with the same optimisation flags,
and timed on one Windows x64 machine on 2026-09-28, once with MSVC and once with clang. Overflow
checks are on unless a row says they are off (`@overflow(wrap)`). "Same speed" means within 5%.

The loops over lists are compared with the C built by clang in both columns. On the hand-written C
of those loops MSVC either skips most of the rounds or does not vectorise the loop at all, which
would make Ember look faster than it is.

### As fast as C or faster

| What the program does | With MSVC | With clang |
|---|---|---|
| Map of numbers: 1 million inserts, then 5 million lookups | 2.2 times faster | 2.3 times faster |
| Build a long string (16 million appends), then count one letter in it | 31% faster | 20% faster |
| Copy a 1,000-number list into another, one item at a time, 100,000 times | 28% faster | same speed |
| Integer arithmetic: 50 million rounds of multiply, divide and remainder | 18% faster | same speed |
| Sort 5 million numbers | 18% faster | 12% faster |
| Clear and refill a list while 4 other lists are being viewed | 6% faster | same speed |
| Clear and refill a list while 1 other list is being viewed | same speed | 5% faster |
| Change every number twice, reading back what it wrote, overflow checks on | same speed | same speed |
| Change every number twice, writing to a separate list, overflow checks off | same speed | same speed |
| Change every number twice, reading back what it wrote, overflow checks off | same speed | same speed |
| Start and stop an empty program | same speed | same speed |
| Add up a list of 1,000 numbers, 300,000 times, overflow checks off | same speed | same speed |
| Change every number using a second list, 20,000 rounds, overflow checks on | same speed | same speed |
| Number every second value of a range of 2 million (`step_by`, `enumerate`), 300 times | same speed | same speed |
| Change every number using a second list, 20,000 rounds, overflow checks off | same speed | same speed |
| Move 100,000 particles 2,000 times, changing each in place | same speed | same speed |
| Change every number twice, writing to a separate list, overflow checks on | same speed | same speed |
| Decimal arithmetic: the Mandelbrot set, 1,000 × 1,000 points | same speed | same speed |
| Clear and refill a list while 16 other lists are being viewed | same speed | same speed |
| Add one list into another, 2,000 rounds, overflow checks on | same speed | same speed |
| Two lists of 100,000 decimal numbers, add one into the other 5,000 times | same speed | same speed |
| Call a function passed as a value, 200 million times | same speed | same speed |
| Add one list into another, 2,000 rounds, overflow checks off | same speed | same speed |

### Close to C: up to 10% slower with one of the compilers

| What the program does | With MSVC | With clang |
|---|---|---|
| 1 million objects with a name and a list, each read 40 times | 6% slower | 5% slower |
| Add two lists plus the round number into a third, overflow checks off | 6% slower | same speed |
| 1 million objects holding 6 lists, each read 40 times | 9% slower | 9% slower |
| Walk two 1-million-number lists side by side, changing one (`zip`), 300 times | same speed | 9% slower |
| Every third number of a list, after skipping some, at most 300,000 of them (`skip`, `step_by`, `take`), 300 times | same speed | 5% slower |
| Number the items of a list past a starting point, as copies (`copied`, `enumerate`, `skip`), 300 times | 8% slower | 5% slower |
| Number each item of a 1-million-number list while changing it (`enumerate`), 300 times | 8% slower | 6% slower |

### More than 10% slower than C with at least one compiler

> Development is in progress, and attempts will be made to speed the language up in these areas.

| What the program does | With MSVC | With clang |
|---|---|---|
| Recursion: Fibonacci of 38, the slow way | 5% slower | 1.7 times slower |
| Add up a list of 1,000 numbers, 300,000 times, overflow checks on | 35% slower | 1.6 times slower |
| A generic function ("larger of two"), 200 million times | same speed | 35% slower |
| Calls through an interface: area of 1 million shapes, 20 times | 26% slower | 21% slower |
| Call a method that changes an object taken from a list, 100 million times (object holding 6 lists) | 23% slower | same speed |
| Call a method that changes an object taken from a list, 100 million times (object holding 1 list) | 19% slower | same speed |
| Call a method that changes an object taken from a list, and the method calls another function, 100 million times | 19% slower | same speed |
| Enum with `match`: area of 1 million shapes, 100 times | 15% faster | 13% slower |

## Examples

Every standalone program example below compiles and runs with the current compiler.

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

### Embed a static library from C

Build a C-compatible library for embedding in a C or C++ host. The full
MSVC and clang-cl suites, linked C/C++ archive tests, and repository gates
passed; see the [handoff](docs/HANDOFF.md) for details. In a package directory,
add `ember.toml` and use the default `src/lib.em` entry:

```toml
[package]
name = "demo_api"
version = "0.1.0"
language = "0.9.9"
kind = "staticlib"
```

```ember
@export("answer")
fn answer() -> i32:
    return 42
```

Run `ember build` from the package directory. With GNU-style toolchains, the
debug profile emits `target/debug/lib/libdemo_api.a` and `demo_api.h`, plus
`target/debug/lib/ember_runtime/libember_rt.a` and `ember_rt.h`. MSVC and
clang-cl use `demo_api.lib` and `ember_rt.lib` at the corresponding paths.

```c
#include "demo_api.h"
#include "ember_runtime/ember_rt.h"

int main(void) {
    (void)ember_rt_init(NULL);
    int result = answer();
    ember_rt_shutdown();
    return result == 42 ? 0 : 1;
}
```

On Linux, link the package archive before its runtime archive:

```sh
cc host.c -I target/debug/lib \
  -L target/debug/lib -ldemo_api \
  -L target/debug/lib/ember_runtime -lember_rt -lm -o host
./host
```

From a Windows Developer Command Prompt, use the matching toolchain and
profile:

```bat
cl /MD /std:c11 /I target\debug\lib host.c target\debug\lib\demo_api.lib target\debug\lib\ember_runtime\ember_rt.lib
host.exe
```

Use matching profile flags when linking; shipping builds retain their LTO
settings. The same archives can be linked from C++; use the C++ driver
(`c++`/`clang++` or `cl`) for a C++ host. `ember build --emit c` emits C, and
`ember build --emit header` emits only the header without invoking a C
compiler. `cdylib`, shared-runtime, and native-library packaging are not
implemented.

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
docs/brand/          the logo: mark, icon, lockups and banner
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
