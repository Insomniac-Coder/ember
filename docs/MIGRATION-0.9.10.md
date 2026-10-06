# Ember 0.9.10: counts and numbers for every iterator

**State as of 2026-10-06.** The owner's design for G8-4 part 2
(`docs/proposals/G8-4-counts-and-positions.md`, decided 2026-10-05) changes what Ember hands back
when it counts or numbers an iterator's items. The language version moves to 0.9.10 (the owner's
rule for this change). The specification is `docs/spec-source/Ember_v0.9.10_Hardened_2.md`:
Hardened_1 was the design as first built (Appendix H §H.6 lists the rules it changed), and
Hardened_2 adds the owner's rulings of 2026-10-06 on three of the audit's decisions (§H.7): `parse`
reads the 256-bit counts, the known-numbers rule, and `enumerate` numbers that never overflow. This
file says what changes for a program and how to update one, then records the implementation.

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

**The known-numbers rule** (Hardened_2, the owner's ruling of 2026-10-06). Where `len` or `count`
is called on an iterator, the count is in the smallest type that holds every count it can have,
read from the numbers the compiler knows, so most code keeps its `int`. It knows a number written as
one; a name set once and never changed anywhere in its function (no assignment, `+=`, mutable
borrow, `mut` argument or `mut self` call of it) holds what its value held; a range or an `Array`
literal so set has its bounds or its length; a loop's counter holds its range's numbers; `+`, `-`,
`*`, `//` and `%` of known numbers are known; a list's length is at most what its items' bytes allow
(`[HEAP-8]`); `take`, `skip`, `step_by`, `zip` and `chain` give what their known arguments allow.
Anything else may be any value of its type. So `(0..n).iter().len()` with `n: int` is an `int`
(the count is at most `n`), and with `n = 2000000` and `round` counting 0 to 299,
`(round..n).iter().len()` is an `int` too. Hardened_1 read only numbers written in place (the
visible-numbers rule): a variable counted as any value of its type.

Before, every count was an `int`, and a range of more than `int.MAX` values panicked: "a range of
more than int.MAX values has no length", and `len(r)` "the range has more values than an `int` can
hold".

### Numbers: `enumerate`

| Looped over | `enumerate`'s numbers, where nothing else is known |
|---|---|
| containers, ranges of 8-, 16- or 32-bit numbers | `int` (unchanged) |
| ranges of 64-bit numbers | `i128` |
| ranges of 128-bit numbers | `i256` |

The start stays an `int` and may be negative. Where `enumerate` is called, its numbers are in the
smallest of `int`, `u64`, `i128`, `u128` and `i256` that holds every number it can give, from
`start` to `start + n - 1` over at most `n` items, read by the known-numbers rule above:
`(0..10).iter().enumerate()` and `(0..n).iter().enumerate()` with `n: int` number in `int`,
`(0..n).iter().enumerate()` with `n: u64` a parameter in `u64`, and `(m..n).iter().enumerate()`
with `m, n: int` parameters in `i128`.

**No number of an `enumerate` overflows** (Hardened_2, the owner's ruling of 2026-10-06). Where the
numbers could pass the top of the iterator's own type, they step up: `ys.enumerate(start=int.MAX - 1)`
over a list of three items numbers in `u64` (…806, …807, …808), forwards, backwards
(`.rev()`, …808 first) and when `skip` leaves items out. `xs.enumerate()` and `xs.enumerate(start=1)`
stay `int` (a list cannot hold more items than that); a start the compiler knows nothing of may be
any `int`, so over a list it numbers in `i128`. Hardened_1 panicked at a number past `int`'s top
("integer overflow in `+`"), the owner's decision F, which this ruling replaces.

Before 0.9.10, every number was an `int`, and a backwards `enumerate` over a long range panicked.

### Leaving items out

`DoubleEndedIterator.nth_back(n)`, `skip_back(n)` and `Iterator.skip_front(n)` take a `Count`. A
range leaves out any number of items at once, and `rev`, `copied`, `take`, `skip`, `zip`, `chain`
and `enumerate` (from the back) pass it down, so `take(3).rev()`, `skip(n)`, `nth(n)` and `zip`
backwards over a gigantic range finish at once. `take(n)`, `skip(n)`, `nth(n)`, `step_by(k)` and
`enumerate(start)` still take an `int`.

### Whole numbers convert where they fit

(Hardened_2, the owner's ruling of 2026-10-06.) A whole number (`int`, `i8` to `i256`, `u8` to
`u256`, `isize`, `usize`) goes by itself into another whole-number type wherever the compiler knows,
by the known-numbers rule, that every value it can have fits: at an assignment, a declaration, an
argument, a return, and in an operator, where the side that fits the other side's type converts to
it (to `int` where each would). After `n = 5`, `y: u64 = n` is accepted; `y: u64 = x` for a
parameter `x: int` still needs `x as u64` (it may be negative), and `b: u8 = 300` is still refused.
Floats, `bool` and `char` keep their rules. In a function written for any type, where the type of
`enumerate`'s numbers depends on the item type, the lines using them are checked for each use: a
use they do not fit is refused there, with "in `labels` used with `T` = `u8`".

### `u256` and `i256`, the 256-bit counts

Built-in integers that only count: they add, subtract, compare, print, format, hash, and convert to
and from the other integers with `as`; the narrower integers widen to them, and they implement the
operator interfaces of what they do (`Add`, `Sub`, `Neg`, `AddAssign`, `SubAssign`). Multiplying,
dividing, bit operations, the integer methods, `MIN`/`MAX`, float conversions and ranges of them
are `E2020`. `parse` reads them over their whole range (Hardened_2, the owner's ruling of
2026-10-06): `"115792089237316195423570985008687907853269984665640564039457584007913129639935"
.parse[u256]()` is `Ok`, one more is `Err(ParseError.Overflow)`; Hardened_1 refused it (`E2020`).

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
* **Fewer `as`** (Hardened_2). A conversion the compiler can now prove safe needs no `as`; an
  existing `as` still works.
* **A count or number now narrower** (Hardened_2). Where the compiler knows the numbers, a count or
  a number that was a `u64` or an `i128` may now be an `int`: `kept = 0 .. n` then
  `e: u64 = kept.iter().len()` is `E2020` (an `int`); write `e: int` or `e = …`. Two conformance
  cases changed so (`STD-19/accept_known_numbers_pick_the_smallest_type`,
  `STD-19/accept_a_chain_counts_a_level_above_its_sides`).
* **A number now wider** (Hardened_2). `enumerate`'s numbers over a list with a start that could
  put them past `int`'s top (a start near the top, or one the compiler knows nothing of) are `u64`s
  or `i128`s: `a: Array[(int, int)]` gathering them is `E2020`; write the wider type
  (`CTL-3b/accept_an_enumerate_under_two_revs_numbers_forwards`).
* **A test that expected a number past `int`'s top to panic** now runs: the number is there. Six
  conformance cases became `accept_` cases (`STD-19/accept_enumerate_past_int_max_steps_up`,
  `STD-19/accept_a_reversed_enumerate_past_the_top_steps_up`, and four in `CTL-3b/`).
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
* Hardened_2 (ADR-140, the owner's rulings of 2026-10-06 on three of the audit's decisions):
  `parse` reads `u256` and `i256` (eight 32-bit pieces in the runtime, `ember_parse_u256_*` and
  `ember_parse_i256_*`; `TXT-10/accept_parse_256_bit_counts`); the known-numbers rule (a pass over
  a function's body learns what each name set once holds and assumes it is never changed; a name a
  count or number relied on that the body changes, read from the checked body's assignments and
  mutable borrows, sends the body through another pass knowing nothing of it, as `[TYP-23]`'s open
  locals do; a closure's body, checked more than once already, carries the names its earlier check
  found changed; `STD-19/accept_known_numbers_follow_names_set_once`,
  `STD-19/reject_a_name_the_body_changes_is_not_known`); and `enumerate` numbers that step up
  instead of overflowing, so a counted loop computes them with no check and no check past a top
  (the fused loop's number checks, `fused_numbers`, are gone) and `Enumerate.skip_back` numbers
  nothing.
