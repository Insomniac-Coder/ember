# Counts and positions for ranges of any size (G8-4, part 2)

A design for the owner, 2026-10-05. Nothing in it is built. If it is adopted, the spec's version
becomes `0.9.10_Hardened_1` (the owner's rule for this change), with its own migration notes.

## The decided design (the owner, 2026-10-05 evening)

Every decision below is the owner's, given one at a time after the explanations of that evening.
Sections 1 to 7 are the analysis that led to it and are kept as they were written.

### A. What a count and a position are, by what is looped over

| Looped over | `len()` gives | `enumerate`'s numbers are |
|---|---|---|
| arrays, lists, text, maps, sets | `int` | `int` |
| ranges of 8-, 16- or 32-bit numbers | `int` | `int` |
| ranges of `int`s (or other 64-bit numbers) written with `..` | `u64` | `i128` |
| ranges of `int`s (or other 64-bit numbers) written with `..=` | `u128` | `i128` |
| ranges of 128-bit numbers written with `..` | `u128` | the 256-bit count |
| ranges of 128-bit numbers written with `..=` | the 256-bit count | the 256-bit count |

- **Counts are unsigned**, sized to the most a range of that kind can hold: a `..` range of 64-bit
  numbers holds at most 18,446,744,073,709,551,615 values, `u64`'s top; a `..=` one one more. The
  owner: "DID YOU FORGET THAT UINT EXISTS".
- **Positions are signed**, because `enumerate` keeps negative starts (`enumerate(xs, -3)`), and
  they reach as far as the range's last position.
- **What the caller passes stays an `int`:** `take(n)`, `skip(n)`, `nth(n)`, `step_by(k)` and
  `enumerate`'s start. (Section 5 said an `int` would widen into the count type; that holds for a
  signed count only, and `[TYP-5]` does not widen `int` to `u64`.) What changes type is what Ember
  hands back: `len()`, `count()`, `position()` and `enumerate`'s numbers.
- **The case that sets the 64-bit row** (the owner: "yes, it should work"): a gigantic range
  numbered from a negative start, walked backwards,
  `(-9223372036854775808 .. 9223372036854775807).iter().enumerate(-3).rev()`, where one `i` goes
  from 18,446,744,073,709,551,611 down to −3: no 64-bit type holds both.

### B. The visible-numbers rule (the owner: "the numbers you get in the range should help you decide")

The table gives the type when the compiler cannot see the range's numbers. Where it can, it
picks **the smallest type that holds every value the loop can give**, from what is visible at the
place the range is numbered or measured: the bounds written as numbers, the types of the bounds
(every variable has one, written or inferred: `n = 10` is an `int`, `m = f()` takes `f`'s return
type), and `enumerate`'s start.

| Loop | `i` is |
|---|---|
| `(0 .. 10).iter().enumerate()` | `int` (0 to 9) |
| `(0 .. n).iter().enumerate()`, `n: int` | `int` (at most 9,223,372,036,854,775,806) |
| `(0 .. n).iter().enumerate()`, `n: u64` | `u64` (0 up to 18,446,744,073,709,551,614, never negative) |
| `(0 .. n).iter().enumerate(-3)`, `n: u64` | `i128` (−3 up to past `int`'s top) |
| `(m .. n).iter().enumerate()`, `m, n: int` | `i128` |
| `it.enumerate()` where `it` is a range iterator in a variable | the table's row (the numbers are not visible there) |

Why it matters (the owner, after the example): an `i128` cannot be mixed with an `int` in
arithmetic (`total + i` is `E2020`, "`+` cannot be applied to `i64` and `i128`"), so everyday loops
such as `0 .. n` would otherwise need `i as int`.

### C. Loop versioning for the wide types (the owner's idea: "ship all possible versions")

Where `i` (or a count) is wider than 64 bits because the numbers are not visible, the compiler
builds the loop twice, as it already does for bounds checks (`[OPT-2]`, `[OPT-3]`,
`compiler/ember_analysis/src/loop_version.rs`): one test before the loop on the actual numbers,
a copy that computes the counter in 64 bits taken when every value the loop can give fits, and
the 128-bit (or 256-bit) loop otherwise. The test runs once per loop, not per item. So the type
the program sees follows B, and the speed follows the actual numbers: only a range that is truly
gigantic pays for wide arithmetic.

### D. The 256-bit count

Exposed to programs, but **only as a count**: it is what `len()` and `enumerate` give for 128-bit
ranges (section A), it compares, adds, subtracts, prints, and converts to and from the other
integers; there are no ranges of it (a range of 256-bit numbers would need a 512-bit count, the
same problem one size up).

### E. The library half of part 1

`DoubleEndedIterator` gains `nth_back(n)` (skip `n` items from the back): a default that steps
back one at a time, and an instant one for ranges (`end` moved by `n`). `take`, `skip`, `step_by`
and `zip` walk backwards with the wrapped iterator's count (now wide enough) and `nth_back`, so
none of them asks for a count that cannot be held, and none skips a gigantic tail one item at a
time.

### F. What it leaves

None of question 8's six crashes. An `enumerate` started near the top of its type still overflows
on the way, as `int.MAX + 1` does: ordinary arithmetic, the same for arrays.

### G. The spec

Version `0.9.10_Hardened_1` (the owner's rule for this change), with `MIGRATION-0.9.10.md`; the
owner's rulings replace ODR-089's "counts are `int`s" and ODR-091's backwards-`enumerate` panic.
Rules touched: `[STD-19]`, `[STD-26]`, `[CTL-3b]`, `[RNG-4]`, `[OPT-2]`, `[OPT-3]`, and the
256-bit count's entry beside the integer types.

## 1. What happens today

Ember holds every count in an `int`: `len()`'s answer, `enumerate`'s positions, the `n` of
`take(n)`, `skip(n)`, `nth(n)`, `step_by(k)`, and what `count()` and `position()` return. That
was my ruling ODR-089 (2026-09-29), made under the owner's delegation; ODR-091 (2026-10-01, also
mine) added that a backwards `enumerate` panics when its first position does not fit.

An `int` holds up to 9,223,372,036,854,775,807. A range does not store its items, so it can hold
more: `0 as u64 .. u64.MAX` holds 18,446,744,073,709,551,615. Measured on that range, each case
written as a `for` loop and step by step with `.next()`:

| Case | `for` loop | step by step |
|---|---|---|
| 1. `take(3).rev()` | 2, 1, 0 | panics: "a range of more than int.MAX values has no length" |
| 2. `skip(5).rev()` | 18446744073709551614, … | panics, the same |
| 3. `step_by(2).rev()` | 18446744073709551614, … | panics, the same |
| 4. `enumerate().rev()` | panics: "integer overflow in `+`" | panics, the same as 1 |
| 5. `zip(…).rev()` | panicked; **fixed by D-515 (part 1)** for a temporary iterator | panics, the same as 1 |
| 6. `.len()` | (no `for` form) | panics, the same as 1 |

## 2. The owner's requirements

1. Counts and positions handle every value the thing looped over needs, up to the highest
   unsigned value Ember has (`u128.MAX`); every loop handles every kind of integer.
2. `enumerate` keeps its negative start (`enumerate(xs, -3)` numbers −3, −2, −1, …).
3. The width is situational: big only where it has to be, never 128-bit everywhere ("why use a
   hammer for a thing which can be fixed with a screwdriver").

## 3. Facts that shape any design

- **Containers never need more than an `int`.** An array, view, string, map or set stores its
  items, so its length is bounded by memory: 18 quintillion one-byte items would need 18 million
  terabytes. Their counts and positions stay `int` under every option below.
- **Only ranges can be giant.** A range over a type of *b* bits holds up to 2^*b* values (the
  whole type, inclusive).
- **A variable's type is fixed when the program is compiled.** `len()`'s answer and `enumerate`'s
  `i` get one type per loop, chosen by the compiler; the running program cannot switch `i` from 8
  bytes to 16.
- **Ember widens integers implicitly and never narrows them** (`[TYP-5]` rule 1: `iN → iM`,
  `uN → iM` for M > N, at arguments, assignments, returns). So a wider count type costs nothing
  where an `int` is passed *in* (`take(n)` with `n: int` still works), and costs an explicit
  conversion only where a range's count or position is taken *out* into an `int`.
- **The common loops do not touch range counts.** `for i in 0..n` hands out the range's values
  (`int`), not positions. `for i, x in enumerate(xs)` numbers a container (`int`). Only `len()`,
  `enumerate`, `count()` and `position()` *on a range* would see a wider type.

## 4. What other languages do

- **Rust:** counts and positions are `usize`. Ranges over 64- and 128-bit integers do not
  implement `ExactSizeIterator`, so `len()`, `take(3).rev()`, `enumerate().rev()` on them do not
  compile: the failing cases are refused before the program runs.
- **Swift:** counts are `Int`; asking the count of a range with more values than `Int.max` traps.
- **C++20 ranges:** the count (difference) type of a range of `W` is a signed integer wider than
  `W` when one exists, otherwise a wider integer-like type of the library's (128-bit in GCC):
  situational by the element type. This is the closest to the owner's requirements.
- **Python:** integers never overflow at all, at the cost of every integer operation checking
  its size.

## 5. Options

### Option 1 — the count type follows what is looped over (C++20's rule)

Every iterator gets a **count type** (an associated type, `Count`), chosen by what it runs over:

| Looped over | Count type |
|---|---|
| an array, view, string, map, set (and adapters over them) | `int` |
| a range of 8-, 16- or 32-bit integers | `int` (holds 2^32 with room) |
| a range of 64-bit integers: `int`, `i64`, `u64`, `isize`, `usize` | `i128` |
| a range of 128-bit integers: `i128`, `u128` | a 256-bit count, a new std type |

- `len()`, `count()`, `position()`, `enumerate`'s positions and its `start` are of the count
  type; `take`, `skip`, `nth` and `step_by` take it (an `int` argument widens implicitly).
- An adapter keeps the count type of what it wraps; `zip` and `chain` take the wider of their
  two.
- **The library half of part 1** comes with it: `DoubleEndedIterator` gains `nth_back(n)` (Rust's
  name: skip `n` items from the back), with a default that steps back one at a time and an
  instant one for ranges (`end` moved by `n`). `take`, `skip`, `step_by` and `zip` walk backwards
  with `len()` (now wide enough) and `nth_back` (now instant on a range), so cases 1, 2, 3 and 5
  work step by step, and 4 and 6 work too, because a 64-bit range's positions and length fit in
  an `i128`.
- **Costs:**
  - A program that takes a 64-bit range's `len()` or `enumerate` position into an `int` writes a
    conversion (`(0..n).iter().len() as int`). That is rare code (see section 3), and the
    migration notes list it.
  - Arithmetic on those positions is 128-bit in the program's meaning. The compiler still emits
    64-bit arithmetic wherever its range facts (`[RNG-4]`) prove the values fit, as they do for
    `0..n`, so the generated C stays 64-bit there; where they cannot prove it, two machine words.
  - A 256-bit count type for 128-bit ranges: a std struct with the arithmetic operators,
    implemented in the runtime. Only 128-bit ranges use it.
- **Meets** all three requirements.

### Option 2 — Option 1 with a cap for 128-bit ranges

As Option 1, but a 128-bit range's count type is `i128` too, so no 256-bit type is added. A range
holding more than 170,141,183,460,469,231,731,687,303,715,884,105,727 values (more than half of
all 128-bit values) still cannot be counted and panics, as today's 64-bit ranges do.

- **Costs:** fewer than Option 1; but it **misses requirement 1** for those 128-bit ranges.

### Option 3 — Option 1, narrowed where the compiler can see the range is small

As Option 1, plus one rule: a range whose start is a constant at or above 0 and whose end is an
`int` (`0..n`, `1..=n`, `range(n)`) counts in `int`, because its count cannot pass `int`'s top
(its end can't). `m..n` with a variable `m` uses `i128`.

- **Gain:** `enumerate` and `len()` over the everyday `0..n` stay `int`, with no conversion.
- **Costs:** one more rule to learn, and a type that depends on whether the start is written as a
  constant: changing `0..n` to `m..n` changes `i`'s type.

### Option 4 — keep `int`, refuse the failing cases before the program runs (Rust's way)

Ranges over 64- and 128-bit integers do not offer `len()`, nor `take`, `skip`, `step_by`,
`enumerate` or `zip` backwards: those are compile errors instead of crashes.

- **Costs:** the smallest change, but `(0..n).iter().enumerate().rev()` with `n: int` stops
  compiling, and it **misses requirement 1** (loops cannot do everything).

### Option 5 — a count that grows when it must (Python's way, for counts only)

The count type is a number that stays 64-bit while it fits and becomes wider by itself when it
doesn't.

- **Costs:** a new kind of integer in the language, with a size check on each count operation;
  meets the requirements at run time rather than by type.

## 6. Recommendation

**Option 1**, with Option 3's narrowing as a choice for the owner. It is the general solution
(C++20's rule, made Ember's), correct for every range, its cost falls only on the rare code that
takes a 64-bit range's count or position into an `int`, and it keeps every container count an
`int`. Option 2 if the owner prefers no 256-bit type over requirement 1 for the largest 128-bit
ranges.

## 7. What it changes (inventory for Option 1)

- **Spec:** `[STD-19]` (counts, `len`, the adapters' parameters, `enumerate`'s start,
  `nth_back`), `[STD-26]` (`len`, `enumerate`, `range`), `[CTL-3b]` (fused loops number in the
  count type), `[RNG-4]` (the narrowing), a new std type for 256-bit counts; ODR-089 and ODR-091
  superseded by the owner's ruling; version `0.9.10_Hardened_1` and `MIGRATION-0.9.10.md`.
- **std (`core.em`):** `Iterator` (associated `Count`, default `int`), `ExactSizeIterator.len`,
  every adapter's `len` and `next_back`, `RangeIter` and `RangeInclusiveIter` (count type by
  element), `Integer.distance_to`, `take`, `skip`, `step_by`, `nth`, `enumerate`, `count`,
  `position`, the built-ins `len` and `enumerate`.
- **Compiler:** the associated type in the checker; fused loops (counters already `size_t`,
  which holds every count up to 2^64 − 1; a whole inclusive 64-bit range, 2^64 values, and the
  128-bit ranges need `u128` counters), `enumerate`'s position checks against the count type, the
  range-fact narrowing of 128-bit arithmetic, the 256-bit count's operators in the runtime.
- **Tests:** every test that reads a range's `len()` or positions; the six cases above for each
  integer width, both forms; programs that convert a count into an `int`.
