# Ember 0.9.10: counts and numbers for every iterator

**State as of 2026-10-06.** The owner's design for G8-4 part 2
(`docs/proposals/G8-4-counts-and-positions.md`, decided 2026-10-05) changes what Ember hands back
when it counts or numbers an iterator's items. The language version moves to 0.9.10 (the owner's
rule for this change), and the specification is `docs/spec-source/Ember_v0.9.10_Hardened_1.md`,
whose Appendix H §H.6 lists every rule it changes. This file says what changes for a program and how
to update one, then records the implementation.

## 1. What changes for a program

### Counts: `len()`, `count()`, `position()`

An iterator counts its items in its `Count`, the smallest type that holds every count it can have:

| Looped over | count type |
|---|---|
| arrays, lists, text, maps, sets, and adapters over them | `int` (unchanged) |
| ranges of 8-, 16- or 32-bit numbers | `int` (unchanged) |
| `a..b` of 64-bit numbers (`int`, `i64`, `u64`, `isize`, `usize`) | `u64` |
| `a..=b` of 64-bit numbers | `u128` |
| `a..b` of 128-bit numbers | `u128` |
| `a..=b` of 128-bit numbers | `u256` |

`r.len()`, `len(r)` and `r.iter().len()` agree. An adapter that never gives more items than it
takes (`take`, `skip`, `step_by`, `rev`, `zip`, `copied`, `cloned`, `map`, `filter`, `filter_map`,
`take_while`, `skip_while`, `inspect`, `peekable`, `enumerate`) counts as what it wraps.

`chain` counts its two sides together, stepping up only as far as the values need: two `int`
counts stay an `int` (stored items and small ranges cannot come near its top); otherwise the count
is one level above the wider side's: `u64`, then `u128`, then `u256`. `flatten` and `flat_map`
count in the level that holds the outer count times the inner one (an `int` by an `int` stays an
`int`). `u256` is the limit. In generic code, these are the counts of the types involved, filled in
for each use.

**Where the range's numbers are visible** (written in place: `(0..10).iter().len()`, `len(0..n)` with
`n: int`), the count is in the smallest type that holds it, so most code keeps its `int`:
`(0..n).iter().len()` with `n: int` is an `int`, since the count is at most `n`. `take`, `skip` and
`step_by` written with numbers lower the count: `(m..n).step_by(2).enumerate()` with `m, n: int`
gives at most 2^63 items and numbers in `int`.

Before, every count was an `int`, and a range of more than `int.MAX` values panicked: "a range of
more than int.MAX values has no length", and `len(r)` "the range has more values than an `int` can
hold".

### Numbers: `enumerate`

| Looped over | `enumerate`'s numbers |
|---|---|
| containers, ranges of 8-, 16- or 32-bit numbers | `int` (unchanged) |
| ranges of 64-bit numbers | `i128` |
| ranges of 128-bit numbers | `i256` |

The start stays an `int` and may be negative. Where the range's numbers are visible, the numbers
are in the smallest type that holds every one the loop gives: `(0..10).iter().enumerate()` and
`(0..n).iter().enumerate()` with `n: int` number in `int`, `(0..n).iter().enumerate()` with
`n: u64` in `u64`, and `(m..n).iter().enumerate()` with `m, n: int` in `i128`. An iterator kept in a
variable shows no numbers, so it numbers in the table's type.

Before, every number was an `int`, and a backwards `enumerate` over a long range panicked.

### Leaving items out

`DoubleEndedIterator.nth_back(n)`, `skip_back(n)` and `Iterator.skip_front(n)` take a `Count`. A
range leaves out any number of items at once, and `rev`, `copied`, `take`, `skip`, `zip`, `chain`
and `enumerate` (from the back) pass it down, so `take(3).rev()`, `skip(n)`, `nth(n)` and `zip`
backwards over a gigantic range finish at once. `take(n)`, `skip(n)`, `nth(n)`, `step_by(k)` and
`enumerate(start)` still take an `int`.

### `u256` and `i256`, the 256-bit counts

Built-in integers that only count: they add, subtract, compare, print, format, hash, and convert to
and from the other integers with `as`; the narrower integers widen to them, and they implement the
operator interfaces of what they do (`Add`, `Sub`, `Neg`, `AddAssign`, `SubAssign`). Multiplying,
dividing, bit operations, the integer methods, `MIN`/`MAX`, float conversions, ranges of them and
`parse` are `E2020`.

### Ranges and indices (the audit of the delegated rulings)

* `for i in a..=T.MAX` ends after `T.MAX`, as `[CTL-3]` says; it ran forever.
* `a..` gives its type's maximum, in a `for` and through its iterator, and panics only when the
  value after it is asked for; before, it panicked before giving the maximum. `skip(n)` and
  `nth(n)` over it leave the values out at once.
* An index or a size wider than `usize` (`i128`, `u128`, `i256`, `u256`) that `usize` does not hold
  panics (`index … is out of bounds`); before, its low bits were used, so `xs[2^64 + 1]` read
  `xs[1]`.

### The library

* `std.core.ItemCount`: what a count or a number type does (`of`, `of_wide`, `to_wide`, `at_top`,
  `at_most`, `as_count`, `plus`, `less`, `exceeds`, `div_by`, `rem_by`, `to_int`); `int`, `u64`,
  `u128`, `u256`, `i128` and `i256` implement it. Import it (`from std.core import ItemCount`) to
  name it in a bound.
* `Iterator` has two associated types, `Count` and `Position`, both `int` unless an iterator says
  otherwise; `count()` returns `Count`, `position()` an `Option[Count]`; `ExactSizeIterator.len`
  returns `Count`.
* `Enumerate` takes its number type as a parameter: `Enumerate[I, P]`.

## 2. Updating a program

* **A number or count into an `int`.** `total += i` where `i` numbers a range of 64-bit numbers
  whose numbers are not visible (`(m..n)`, a range from a variable) is `E2020` (`i128` and `int`):
  write `i as int` where it fits, or keep the total in the wider type. The conformance case
  `CTL-3b/accept_a_value_kept_running_is_the_same_value` needed one such `as int`.
* **Generic code.** `it.count()` in a function generic over `I: Iterator` returns `I.Count`; write
  `it.count().to_int()` for an `int`, which panics past `int.MAX`.
* **A test that expected the old panic** ("a range of more than int.MAX values has no length", "the
  range has more values than an `int` can hold") now runs: the length is there. Three conformance
  cases changed so (`STD-19/accept_a_wide_range_has_its_length`,
  `STD-26/accept_len_of_a_range_too_long_for_an_int_is_its_count`,
  `STD-26/accept_len_of_an_inclusive_range_past_an_int_is_its_count`).

## 3. Implementation record

* Design: `docs/proposals/G8-4-counts-and-positions.md` (the owner's decisions A to G, 2026-10-05),
  with three refinements the owner made while it was built: `u256` beside `i256`, so counts are
  unsigned all the way; `chain`'s count steps up one level from its sides', in generic code too;
  and the rule behind both, written into `docs/AUTOPILOT.md` §4 ("step up by need; the biggest tool
  only when the values need it").
* Decision record: ADR-138 in `docs/DECISIONS.md`.
* Compiler defects found while building it, each fixed with a conformance case: D-516 to D-524
  (`docs/DEFECTS.md`).
* Tests: `tests/conformance/STD-19/` (counts, numbers, `nth_back`, skipping, `chain`, `flatten`,
  the 256-bit counts, the visible-numbers rule), `tests/conformance/IFC-4/` (the defects),
  `tests/conformance/STD-26/` (`len`).
* Design C, loop versioning: a `for` over `enumerate` whose numbers are wider than 64 bits (a
  range of 64-bit numbers whose numbers are not visible numbers in `i128`) has a copy of the loop
  computing them as `int`s, taken after one test before the loop when every number fits one; a
  read of such a number as an `int` then reads the `int` (`narrow_widened_reads_all`). Measured on
  2,000,000,000 turns of `enumerate(start=s)` over `m..n`, against the same loop numbering in
  `int` before G8-4: the 128-bit numbers alone took 1.14x its time with clang and 1.47x with MSVC;
  with the copy, 1.00x with both. `CTL-3b/accept_a_wide_enumerate_numbers_the_same_in_either_copy`.
* Found by the audit of the delegated rulings that followed (ADR-139): `a..=T.MAX` ran forever
  (D-525), `a..` stopped before its type's maximum (D-526, ODR-027), an index wider than `usize`
  read the wrong element (D-527), `parse[i256]` stopped the compiler, `i256` and `u256` lacked the
  operator interfaces of the operators they have (ODR-040), and a backwards `enumerate`'s instant
  skip did not number the items it passed over (`[CTL-3b]`). The benchmark run then found the
  visible-numbers rule ignoring `skip` and `step_by` (a stepped range numbered in `u64` where
  `int` holds every number).
