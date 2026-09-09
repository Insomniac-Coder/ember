#! language "0.8.3"
## `std.core` — Part XV's prelude module.
##
## Part IV §8's standard interfaces, spelled as ordinary interfaces exactly as
## that section writes them. `Option`, `Result`, `Array` and `String` are still
## compiler-known (Part XX.1 makes them so "until Phase 2's generics let the
## standard library write them"); this module holds what can already be
## written, and grows as the compiler does.

## `[ENM-3]` — a unit-only enum, so `Copy`, `Eq`, `Hash` and `Debug` come free
## and it converts to its repr with `as`.
pub enum Ordering:
    Less
    Equal
    Greater

pub interface Eq:
    fn eq(self, other: Self) -> bool

## Part IV §8 declares `Ord: Eq` with `cmp(self, other: Self) -> Ordering`.
## `[TYP-9]` — `Ord` is deliberately **not** implemented for floats: `NaN`
## makes the ordering partial, so a float uses `PartialOrd` and the comparison
## operators, which return `false` on either side of a `NaN`.
pub interface Ord: Eq:
    fn cmp(self, other: Self) -> Ordering

## Part IV §8 also declares `Clone`, `Hash`, `Default`, `Display`, `Debug`, the
## operator interfaces and `Iterator`. `Default` and the rest of the
## associated-function forms wait for interface members with no `self`
## receiver, which this phase does not have; declaring one here would be a
## name taken by something that cannot be implemented.
